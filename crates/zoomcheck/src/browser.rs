use anyhow::{Context, Result, bail};
use headless_chrome::{
    Browser, LaunchOptions,
    browser::tab::ModifierKey,
    protocol::cdp::{
        Emulation::SetDeviceMetricsOverride,
        Page::{AddScriptToEvaluateOnNewDocument, CaptureScreenshotFormatOption},
    },
};
use serde::Deserialize;
use std::{
    env,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};
use tempfile::TempDir;
use zoomcheck::model::{Finding, FindingKind, Rect, Severity, Step, StepResult, Workflow, ZoomRun};

pub fn find_browser(explicit: Option<&Path>) -> Result<PathBuf> {
    if let Some(path) = explicit {
        return existing(path);
    }
    if let Ok(path) = env::var("ZOOMCHECK_BROWSER") {
        return existing(Path::new(&path));
    }
    for path in [
        "/usr/bin/chromium",
        "/usr/bin/chromium-browser",
        "/usr/bin/google-chrome",
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    ] {
        if Path::new(path).is_file() {
            return Ok(path.into());
        }
    }
    if let Ok(root) = env::var("PLAYWRIGHT_BROWSERS_PATH") {
        if let Some(path) = find_playwright_chrome(Path::new(&root)) {
            return Ok(path);
        }
    }
    bail!(
        "Chromium was not found; pass --browser or set ZOOMCHECK_BROWSER / PLAYWRIGHT_BROWSERS_PATH"
    )
}

fn existing(path: &Path) -> Result<PathBuf> {
    if path.is_file() {
        Ok(path.to_path_buf())
    } else {
        bail!("browser does not exist at {}", path.display())
    }
}

fn find_playwright_chrome(root: &Path) -> Option<PathBuf> {
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(dir).ok()?.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if matches!(
                path.file_name().and_then(OsStr::to_str),
                Some("chrome" | "headless_shell")
            ) {
                return Some(path);
            }
        }
    }
    None
}

fn launch(path: &Path, headless: bool) -> Result<(TempDir, Browser)> {
    let profile = tempfile::tempdir().context("create temporary browser profile")?;
    let flags = [
        OsStr::new("--disable-search-engine-choice-screen"),
        OsStr::new("--no-first-run"),
        OsStr::new("--hide-scrollbars"),
    ];
    let options = LaunchOptions::default_builder()
        .path(Some(path.to_path_buf()))
        .user_data_dir(Some(profile.path().to_path_buf()))
        .headless(headless)
        .sandbox(false)
        .window_size(Some((1280, 900)))
        .args(flags.to_vec())
        .build()
        .map_err(|e| anyhow::anyhow!("invalid browser options: {e}"))?;
    let browser = Browser::new(options).context("start Chromium")?;
    Ok((profile, browser))
}

pub fn record(
    url: &str,
    name: &str,
    out: &Path,
    browser_path: &Path,
    timeout: Duration,
) -> Result<()> {
    let (_profile, browser) = launch(browser_path, false)
        .context("open recording browser (is a desktop display available?)")?;
    let tab = browser.new_tab()?;
    let recorded = Arc::new(Mutex::new(Vec::<Step>::new()));
    let finished = Arc::new(AtomicBool::new(false));
    let recorded_for_browser = Arc::clone(&recorded);
    let finished_for_browser = Arc::clone(&finished);
    tab.expose_function(
        "__zoomcheckSend",
        Arc::new(move |payload: serde_json::Value| {
            let Some(payload) = payload.as_str() else {
                return;
            };
            let Ok(envelope) = serde_json::from_str::<serde_json::Value>(payload) else {
                return;
            };
            let Some(message) = envelope["args"].get(0).and_then(|value| value.as_str()) else {
                return;
            };
            let Ok(message) = serde_json::from_str::<serde_json::Value>(message) else {
                return;
            };
            match message["type"].as_str() {
                Some("done") => finished_for_browser.store(true, Ordering::SeqCst),
                Some("step") => {
                    if let Ok(step) = serde_json::from_value::<Step>(message["step"].clone()) {
                        recorded_for_browser.lock().unwrap().push(step);
                    }
                }
                _ => {}
            }
        }),
    )?;
    tab.call_method(AddScriptToEvaluateOnNewDocument {
        source: RECORDER_JS.into(),
        world_name: None,
        include_command_line_api: None,
        run_immediately: None,
    })?;
    tab.navigate_to(url)?.wait_until_navigated()?;
    ensure_page_loaded(&tab, url)?;
    eprintln!("Recording {name:?}. Use the page with the keyboard; press Alt+Shift+S to save.");
    let started = Instant::now();
    loop {
        if started.elapsed() > timeout {
            bail!(
                "recording timed out after {} seconds; press Alt+Shift+S to finish",
                timeout.as_secs()
            );
        }
        if finished.load(Ordering::SeqCst) {
            let workflow = Workflow {
                version: 1,
                name: name.into(),
                url: url.into(),
                settle_ms: 180,
                steps: recorded.lock().unwrap().clone(),
            };
            workflow.validate()?;
            if let Some(parent) = out.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(out, serde_json::to_vec_pretty(&workflow)?)?;
            eprintln!("Saved {} steps to {}", workflow.steps.len(), out.display());
            return Ok(());
        }
        thread::sleep(Duration::from_millis(120));
    }
}

pub fn run_zoom(
    workflow: &Workflow,
    zoom: u16,
    out: &Path,
    browser_path: &Path,
    headless: bool,
) -> Result<ZoomRun> {
    let (_profile, browser) = launch(browser_path, headless)?;
    let tab = browser.new_tab()?;
    tab.navigate_to(&workflow.url)?.wait_until_navigated()?;
    ensure_page_loaded(&tab, &workflow.url)?;
    apply_browser_zoom(&tab, zoom)?;
    thread::sleep(Duration::from_millis(workflow.settle_ms));
    tab.evaluate(
        "document.body && document.body.focus(); window.scrollTo(0,0)",
        false,
    )?;
    let mut results = Vec::new();
    for (index, step) in workflow.steps.iter().enumerate() {
        press(&tab, &step.key)
            .with_context(|| format!("send {:?} at step {}", step.key, index + 1))?;
        thread::sleep(Duration::from_millis(workflow.settle_ms));
        let raw: BrowserSnapshot = eval_value(&tab, SNAPSHOT_JS)
            .with_context(|| format!("inspect focus after step {}", index + 1))?;
        results.push(classify(index + 1, step, raw));
    }
    let viewport: Viewport = eval_value(
        &tab,
        "JSON.stringify({width:innerWidth,height:innerHeight,dpr:devicePixelRatio})",
    )?;
    let screenshot_name = format!("zoom-{zoom}.png");
    let png = tab.capture_screenshot(CaptureScreenshotFormatOption::Png, None, None, true)?;
    fs::write(out.join(&screenshot_name), png)?;
    let failures = results
        .iter()
        .filter(|step| {
            step.findings
                .iter()
                .any(|finding| finding.severity == Severity::Failure)
        })
        .count();
    let warnings = results
        .iter()
        .filter(|step| {
            step.findings
                .iter()
                .any(|finding| finding.severity == Severity::Warning)
        })
        .count();
    Ok(ZoomRun {
        zoom,
        viewport_css: format!(
            "{:.0} × {:.0} CSS px · {:.2} dppx",
            viewport.width, viewport.height, viewport.dpr
        ),
        screenshot: screenshot_name,
        steps: results,
        failures,
        warnings,
    })
}

fn ensure_page_loaded(tab: &std::sync::Arc<headless_chrome::Tab>, requested: &str) -> Result<()> {
    let current = tab.get_url();
    if current.starts_with("chrome-error://") {
        bail!("could not load {requested}; check the URL, server, and network connection");
    }
    Ok(())
}

fn apply_browser_zoom(tab: &std::sync::Arc<headless_chrome::Tab>, zoom: u16) -> Result<()> {
    // Browser zoom narrows the CSS viewport while increasing device pixels per CSS pixel.
    // Applying both through Chromium's desktop device metrics preserves responsive layout,
    // fixed positioning, overflow, and high-density rendering; this is not CSS `zoom`.
    let factor = zoom as f64 / 100.0;
    tab.call_method(SetDeviceMetricsOverride {
        width: (1280.0 / factor).round() as u32,
        height: (900.0 / factor).round() as u32,
        device_scale_factor: factor,
        mobile: false,
        scale: Some(1.0),
        screen_width: Some(1280),
        screen_height: Some(900),
        position_x: None,
        position_y: None,
        dont_set_visible_size: None,
        screen_orientation: None,
        viewport: None,
        display_feature: None,
        device_posture: None,
    })?;
    Ok(())
}

fn press(tab: &std::sync::Arc<headless_chrome::Tab>, key: &str) -> Result<()> {
    match key {
        "Shift+Tab" => {
            tab.press_key_with_modifiers("Tab", Some(&[ModifierKey::Shift]))?;
        }
        "Space" => {
            tab.press_key(" ")?;
        }
        key => {
            tab.press_key(key)?;
        }
    }
    Ok(())
}

fn eval_value<T: for<'de> Deserialize<'de>>(
    tab: &std::sync::Arc<headless_chrome::Tab>,
    js: &str,
) -> Result<T> {
    let object = tab.evaluate(js, false)?;
    let value = object
        .value
        .context("browser expression returned no value")?;
    let encoded = value
        .as_str()
        .context("browser expression did not return JSON text")?;
    serde_json::from_str(encoded).context("decode browser result")
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Viewport {
    width: f64,
    height: f64,
    dpr: f64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BrowserSnapshot {
    selector: String,
    element: String,
    accessible_name: String,
    rect: Rect,
    viewport_width: f64,
    viewport_height: f64,
    scroll_x: f64,
    scroll_y: f64,
    scrollable: bool,
    focus_visible: bool,
    viewport_clipped: bool,
    ancestor_clipped: bool,
    obscured: bool,
}

fn classify(number: usize, step: &Step, raw: BrowserSnapshot) -> StepResult {
    let mut findings = Vec::new();
    if raw.element == "body" || raw.element == "html" {
        findings.push(failure(
            FindingKind::FocusOrder,
            "Key did not reach an interactive element",
        ));
    } else if let Some(expected) = &step.expect {
        if expected != &raw.selector && !raw.selector.contains(expected) {
            findings.push(failure(
                FindingKind::FocusOrder,
                &format!("Expected {expected}, reached {}", raw.selector),
            ));
        }
    }
    if raw.accessible_name.trim().is_empty() && !matches!(raw.element.as_str(), "body" | "html") {
        findings.push(warning(
            FindingKind::FocusName,
            "Focused control has no detectable accessible name",
        ));
    }
    if !raw.focus_visible {
        findings.push(warning(
            FindingKind::FocusVisible,
            "No outline or box-shadow focus treatment was detected",
        ));
    }
    if raw.viewport_clipped {
        findings.push(failure(
            FindingKind::ViewportClipping,
            "Focused control is partly or fully outside the zoomed viewport",
        ));
    }
    if raw.ancestor_clipped {
        findings.push(failure(
            FindingKind::AncestorClipping,
            "An overflow ancestor clips the focused control",
        ));
    }
    if raw.obscured {
        findings.push(failure(
            FindingKind::Obscured,
            "Another element covers the focused control's sampled points",
        ));
    }
    if findings.is_empty() {
        findings.push(Finding {
            severity: Severity::Pass,
            kind: FindingKind::FocusOrder,
            message: "Focus is named, visible, and reachable".into(),
        });
    }
    StepResult {
        number,
        key: step.key.clone(),
        selector: raw.selector,
        element: raw.element,
        accessible_name: raw.accessible_name,
        rect: raw.rect,
        viewport_width: raw.viewport_width,
        viewport_height: raw.viewport_height,
        scroll_x: raw.scroll_x,
        scroll_y: raw.scroll_y,
        scrollable: raw.scrollable,
        findings,
    }
}

fn failure(kind: FindingKind, message: &str) -> Finding {
    Finding {
        severity: Severity::Failure,
        kind,
        message: message.into(),
    }
}
fn warning(kind: FindingKind, message: &str) -> Finding {
    Finding {
        severity: Severity::Warning,
        kind,
        message: message.into(),
    }
}

const RECORDER_JS: &str = r#"(()=>{const selector=e=>{if(!e||e===document.body)return'e-body';if(e.id)return'#'+CSS.escape(e.id);for(const a of ['data-testid','name','aria-label'])if(e.hasAttribute(a))return e.tagName.toLowerCase()+'['+a+'="'+CSS.escape(e.getAttribute(a))+'"]';let s=e.tagName.toLowerCase();const p=e.parentElement;if(p){const same=[...p.children].filter(x=>x.tagName===e.tagName);if(same.length>1)s+=`:nth-of-type(${same.indexOf(e)+1})`;}return s};const allowed=new Set(['Tab','Enter',' ','ArrowUp','ArrowDown','ArrowLeft','ArrowRight','Escape','Home','End']);addEventListener('keyup',e=>{if(e.altKey&&e.shiftKey&&e.code==='KeyS'){e.preventDefault();window.__zoomcheckSend(JSON.stringify({type:'done'}));document.querySelector('#__zwc_hint')?.remove();return}if(e.ctrlKey||e.metaKey||e.altKey||!allowed.has(e.key))return;const key=e.key===' '?'Space':(e.key==='Tab'&&e.shiftKey?'Shift+Tab':e.key);window.__zoomcheckSend(JSON.stringify({type:'step',step:{key,expect:selector(document.activeElement)}}))},true);const mount=()=>{if(document.querySelector('#__zwc_hint'))return;const d=document.createElement('div');d.id='__zwc_hint';d.setAttribute('role','status');d.textContent='ZOOM CHECK · recording keys · Alt+Shift+S to save';Object.assign(d.style,{position:'fixed',zIndex:2147483647,right:'12px',bottom:'12px',padding:'12px 16px',background:'#18221d',color:'#fff9ec',font:'700 14px monospace',border:'3px solid #f2c94c',pointerEvents:'none'});document.documentElement.append(d)};document.readyState==='loading'?addEventListener('DOMContentLoaded',mount,{once:true}):mount()})()"#;

const SNAPSHOT_JS: &str = r#"JSON.stringify((()=>{const e=document.activeElement||document.body,r=e.getBoundingClientRect(),s=getComputedStyle(e),vw=innerWidth,vh=innerHeight;let ancestorClipped=false,scrollable=document.documentElement.scrollHeight>vh||document.documentElement.scrollWidth>vw;for(let p=e.parentElement;p;p=p.parentElement){const ps=getComputedStyle(p),pr=p.getBoundingClientRect(),clip=/(hidden|clip|auto|scroll)/.test(ps.overflow+ps.overflowX+ps.overflowY);if(clip&&(r.left<pr.left-1||r.right>pr.right+1||r.top<pr.top-1||r.bottom>pr.bottom+1))ancestorClipped=true;if(p.scrollHeight>p.clientHeight+1||p.scrollWidth>p.clientWidth+1)scrollable=true}const points=[[r.left+r.width/2,r.top+r.height/2],[r.left+2,r.top+2],[r.right-2,r.bottom-2]].filter(([x,y])=>x>=0&&y>=0&&x<vw&&y<vh);const obscured=points.length>0&&!points.some(([x,y])=>{const t=document.elementFromPoint(x,y);return t&&(t===e||e.contains(t))});const id=x=>{if(!x||x===document.body)return'e-body';if(x.id)return'#'+CSS.escape(x.id);for(const a of ['data-testid','name','aria-label'])if(x.hasAttribute(a))return x.tagName.toLowerCase()+'['+a+'="'+CSS.escape(x.getAttribute(a))+'"]';let q=x.tagName.toLowerCase(),p=x.parentElement;if(p){const same=[...p.children].filter(n=>n.tagName===x.tagName);if(same.length>1)q+=`:nth-of-type(${same.indexOf(x)+1})`}return q};const labelled=e.getAttribute('aria-labelledby'),label=labelled&&document.getElementById(labelled);const name=e.getAttribute('aria-label')||(label&&label.textContent)||e.getAttribute('alt')||e.getAttribute('title')||e.value||e.innerText||'';return{selector:id(e),element:e.tagName.toLowerCase(),accessibleName:String(name).trim().replace(/\s+/g,' ').slice(0,160),rect:{x:r.x,y:r.y,width:r.width,height:r.height},viewportWidth:vw,viewportHeight:vh,scrollX:scrollX,scrollY:scrollY,scrollable,focusVisible:(s.outlineStyle!=='none'&&parseFloat(s.outlineWidth)>0)||s.boxShadow!=='none',viewportClipped:r.width<=0||r.height<=0||r.left<0||r.top<0||r.right>vw+1||r.bottom>vh+1,ancestorClipped,obscured}})())"#;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn classifies_a_clipped_control_as_failure() {
        let raw = BrowserSnapshot {
            selector: "#go".into(),
            element: "button".into(),
            accessible_name: "Go".into(),
            rect: Rect::default(),
            viewport_width: 640.0,
            viewport_height: 450.0,
            scroll_x: 0.0,
            scroll_y: 0.0,
            scrollable: true,
            focus_visible: true,
            viewport_clipped: true,
            ancestor_clipped: false,
            obscured: false,
        };
        let result = classify(
            1,
            &Step {
                key: "Tab".into(),
                expect: Some("#go".into()),
                note: None,
            },
            raw,
        );
        assert!(result.findings.iter().any(|f| f.kind == FindingKind::ViewportClipping && f.severity == Severity::Failure));
    }
}
