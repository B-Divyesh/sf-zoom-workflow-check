use anyhow::{Context, Result, bail};
use base64::Engine;
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    env,
    ffi::OsStr,
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};
use tempfile::TempDir;
use tungstenite::{Error as WebSocketError, Message, WebSocket, connect, stream::MaybeTlsStream};
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

struct BrowserSession {
    _profile: TempDir,
    child: Child,
    cdp: Cdp,
}

impl Drop for BrowserSession {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn launch(path: &Path, headless: bool, zoom: u16) -> Result<BrowserSession> {
    let profile = tempfile::tempdir().context("create temporary browser profile")?;
    write_zoom_preference(profile.path(), zoom)?;
    let listener = TcpListener::bind("127.0.0.1:0").context("reserve Chromium debugging port")?;
    let port = listener.local_addr()?.port();
    drop(listener);
    let mut command = Command::new(path);
    command
        .arg(format!("--user-data-dir={}", profile.path().display()))
        .arg(format!("--remote-debugging-port={port}"))
        .arg("--remote-debugging-address=127.0.0.1")
        .arg("--remote-allow-origins=*")
        .arg("--disable-search-engine-choice-screen")
        .arg("--no-first-run")
        .arg("--hide-scrollbars")
        .arg("--window-size=1280,900")
        .arg("--no-sandbox")
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if headless {
        command.arg("--headless=new");
    }
    let child = command.spawn().context("start Chromium")?;
    let cdp = Cdp::connect(port).context("connect to Chromium DevTools")?;
    Ok(BrowserSession {
        _profile: profile,
        child,
        cdp,
    })
}

fn write_zoom_preference(profile: &Path, zoom: u16) -> Result<()> {
    let default_dir = profile.join("Default");
    fs::create_dir_all(&default_dir)?;
    // Chrome stores its desktop page zoom as a logarithmic zoom level in the
    // profile. This is the same setting changed by Chrome's View > Zoom menu.
    let level = (zoom as f64 / 100.0).ln() / 1.2_f64.ln();
    let preferences = json!({
        "partition": { "default_zoom_level": { "x": level } }
    });
    fs::write(
        default_dir.join("Preferences"),
        serde_json::to_vec(&preferences)?,
    )?;
    Ok(())
}

pub fn record(
    url: &str,
    name: &str,
    out: &Path,
    browser_path: &Path,
    timeout: Duration,
) -> Result<()> {
    let mut browser = launch(browser_path, false, 100)
        .context("open recording browser (is a desktop display available?)")?;
    browser
        .cdp
        .call("Runtime.addBinding", json!({ "name": "__zoomcheckSend" }))?;
    browser.cdp.call(
        "Page.addScriptToEvaluateOnNewDocument",
        json!({ "source": RECORDER_JS }),
    )?;
    browser.cdp.navigate(url)?;
    eprintln!("Recording {name:?}. Use the page with the keyboard; press Alt+Shift+S to save.");
    let started = Instant::now();
    let mut recorded = Vec::<Step>::new();
    loop {
        if started.elapsed() > timeout {
            bail!(
                "recording timed out after {} seconds; press Alt+Shift+S to finish",
                timeout.as_secs()
            );
        }
        let Some(event) = browser.cdp.next_event(Duration::from_millis(120))? else {
            continue;
        };
        if event["method"] != "Runtime.bindingCalled"
            || event["params"]["name"] != "__zoomcheckSend"
        {
            continue;
        }
        let Some(payload) = event["params"]["payload"].as_str() else {
            continue;
        };
        let Ok(message) = serde_json::from_str::<Value>(payload) else {
            continue;
        };
        match message["type"].as_str() {
            Some("step") => {
                if let Ok(step) = serde_json::from_value::<Step>(message["step"].clone()) {
                    recorded.push(step);
                }
            }
            Some("done") => {
                let workflow = Workflow {
                    version: 1,
                    name: name.into(),
                    url: url.into(),
                    settle_ms: 180,
                    steps: recorded,
                };
                workflow.validate()?;
                if let Some(parent) = out.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(out, serde_json::to_vec_pretty(&workflow)?)?;
                eprintln!("Saved {} steps to {}", workflow.steps.len(), out.display());
                return Ok(());
            }
            _ => {}
        }
    }
}

pub fn run_zoom(
    workflow: &Workflow,
    zoom: u16,
    out: &Path,
    browser_path: &Path,
    headless: bool,
) -> Result<ZoomRun> {
    let mut browser = launch(browser_path, headless, zoom)?;
    browser.cdp.navigate(&workflow.url)?;
    thread::sleep(Duration::from_millis(workflow.settle_ms));
    browser
        .cdp
        .evaluate("document.body && document.body.focus(); window.scrollTo(0,0)")?;
    let mut results = Vec::new();
    for (index, step) in workflow.steps.iter().enumerate() {
        browser
            .cdp
            .press(&step.key)
            .with_context(|| format!("send {:?} at step {}", step.key, index + 1))?;
        thread::sleep(Duration::from_millis(workflow.settle_ms));
        let raw: BrowserSnapshot = eval_value(&mut browser.cdp, SNAPSHOT_JS)
            .with_context(|| format!("inspect focus after step {}", index + 1))?;
        results.push(classify(index + 1, step, raw));
    }
    let viewport: Viewport = eval_value(
        &mut browser.cdp,
        "JSON.stringify({width:innerWidth,height:innerHeight,dpr:devicePixelRatio})",
    )?;
    let screenshot_name = format!("zoom-{zoom}.png");
    let png = browser.cdp.screenshot()?;
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

fn eval_value<T: for<'de> Deserialize<'de>>(cdp: &mut Cdp, js: &str) -> Result<T> {
    let value = cdp.evaluate(js)?;
    let encoded = value
        .as_str()
        .context("browser expression did not return JSON text")?;
    serde_json::from_str(encoded).context("decode browser result")
}

struct Cdp {
    socket: WebSocket<MaybeTlsStream<TcpStream>>,
    next_id: u64,
    queued_events: Vec<Value>,
}

impl Cdp {
    fn connect(port: u16) -> Result<Self> {
        let deadline = Instant::now() + Duration::from_secs(12);
        loop {
            let last_error = match http_request(port, "GET", "/json/list") {
                Ok(body) => {
                    let url = serde_json::from_slice::<Value>(&body)
                        .ok()
                        .and_then(|targets| {
                            targets
                                .as_array()
                                .and_then(|items| items.first())
                                .and_then(|target| target["webSocketDebuggerUrl"].as_str())
                                .map(str::to_owned)
                        });
                    if let Some(url) = url {
                        let (socket, _) = connect(&url).context("open DevTools websocket")?;
                        return Ok(Self {
                            socket,
                            next_id: 1,
                            queued_events: Vec::new(),
                        });
                    }
                    "DevTools listed no debuggable page".to_owned()
                }
                Err(error) => error.to_string(),
            };
            if Instant::now() >= deadline {
                bail!(
                    "Chromium did not open its DevTools endpoint within 12 seconds: {}",
                    last_error
                );
            }
            thread::sleep(Duration::from_millis(80));
        }
    }

    fn call(&mut self, method: &str, params: Value) -> Result<Value> {
        let id = self.next_id;
        self.next_id += 1;
        self.socket.send(Message::Text(
            json!({ "id": id, "method": method, "params": params }).to_string(),
        ))?;
        loop {
            let value = self.read_value()?;
            if value["id"].as_u64() == Some(id) {
                if let Some(error) = value.get("error") {
                    bail!(
                        "Chromium {method} failed: {}",
                        error["message"]
                            .as_str()
                            .unwrap_or("unknown protocol error")
                    );
                }
                return Ok(value["result"].clone());
            }
            self.queued_events.push(value);
        }
    }

    fn navigate(&mut self, url: &str) -> Result<()> {
        let result = self.call("Page.navigate", json!({ "url": url }))?;
        if let Some(error) = result["errorText"].as_str() {
            bail!("could not load {url}: {error}");
        }
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            let state = self.evaluate("document.readyState")?;
            if matches!(state.as_str(), Some("interactive") | Some("complete")) {
                return Ok(());
            }
            if Instant::now() >= deadline {
                bail!("timed out loading {url}");
            }
            thread::sleep(Duration::from_millis(40));
        }
    }

    fn evaluate(&mut self, expression: &str) -> Result<Value> {
        let result = self.call(
            "Runtime.evaluate",
            json!({
                "expression": expression,
                "returnByValue": true,
                "awaitPromise": true
            }),
        )?;
        if let Some(details) = result.get("exceptionDetails") {
            bail!(
                "page expression failed: {}",
                details["text"].as_str().unwrap_or("unknown exception")
            );
        }
        Ok(result["result"]["value"].clone())
    }

    fn press(&mut self, key: &str) -> Result<()> {
        let (key, code, key_code, modifiers) = match key {
            "Shift+Tab" => ("Tab", "Tab", 9, 8),
            "Tab" => ("Tab", "Tab", 9, 0),
            "Enter" => ("Enter", "Enter", 13, 0),
            "Space" => (" ", "Space", 32, 0),
            "ArrowUp" => ("ArrowUp", "ArrowUp", 38, 0),
            "ArrowDown" => ("ArrowDown", "ArrowDown", 40, 0),
            "ArrowLeft" => ("ArrowLeft", "ArrowLeft", 37, 0),
            "ArrowRight" => ("ArrowRight", "ArrowRight", 39, 0),
            "Escape" => ("Escape", "Escape", 27, 0),
            "Home" => ("Home", "Home", 36, 0),
            "End" => ("End", "End", 35, 0),
            _ => bail!("unsupported key {key:?}"),
        };
        let mut down = json!({ "key": key, "code": code, "windowsVirtualKeyCode": key_code, "nativeVirtualKeyCode": key_code, "modifiers": modifiers });
        down["type"] = Value::String("keyDown".into());
        let mut up = down.clone();
        up["type"] = Value::String("keyUp".into());
        self.call("Input.dispatchKeyEvent", down)?;
        self.call("Input.dispatchKeyEvent", up)?;
        Ok(())
    }

    fn screenshot(&mut self) -> Result<Vec<u8>> {
        let result = self.call(
            "Page.captureScreenshot",
            json!({ "format": "png", "fromSurface": true }),
        )?;
        let data = result["data"]
            .as_str()
            .context("Chromium did not return a screenshot")?;
        base64::engine::general_purpose::STANDARD
            .decode(data)
            .context("decode screenshot")
    }

    fn next_event(&mut self, timeout: Duration) -> Result<Option<Value>> {
        if !self.queued_events.is_empty() {
            return Ok(Some(self.queued_events.remove(0)));
        }
        if let MaybeTlsStream::Plain(stream) = self.socket.get_mut() {
            stream.set_read_timeout(Some(timeout))?;
        }
        let result = match self.read_value() {
            Ok(value) => Ok(Some(value)),
            Err(error) if error.downcast_ref::<WebSocketError>().is_some_and(|websocket| matches!(websocket, WebSocketError::Io(io) if matches!(io.kind(), std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock))) => Ok(None),
            Err(error) => Err(error),
        };
        if let MaybeTlsStream::Plain(stream) = self.socket.get_mut() {
            stream.set_read_timeout(None)?;
        }
        result
    }

    fn read_value(&mut self) -> Result<Value> {
        loop {
            match self.socket.read()? {
                Message::Text(text) => {
                    return serde_json::from_str(&text).context("decode DevTools message");
                }
                Message::Binary(bytes) => {
                    return serde_json::from_slice(&bytes).context("decode DevTools message");
                }
                Message::Ping(payload) => self.socket.send(Message::Pong(payload))?,
                Message::Close(_) => bail!("Chromium closed its DevTools connection"),
                _ => {}
            }
        }
    }
}

fn http_request(port: u16, method: &str, path: &str) -> Result<Vec<u8>> {
    let mut stream = TcpStream::connect(("127.0.0.1", port))?;
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    write!(
        stream,
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"
    )?;
    let mut response = Vec::new();
    let mut buffer = [0_u8; 8192];
    let (boundary, content_length) = loop {
        let count = stream.read(&mut buffer)?;
        if count == 0 {
            bail!("DevTools closed an incomplete HTTP response");
        }
        response.extend_from_slice(&buffer[..count]);
        if let Some(boundary) = response.windows(4).position(|window| window == b"\r\n\r\n") {
            let head = std::str::from_utf8(&response[..boundary])?;
            let content_length = head
                .lines()
                .find_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    name.eq_ignore_ascii_case("content-length")
                        .then_some(value.trim())
                })
                .context("DevTools response has no content length")?
                .parse::<usize>()?;
            break (boundary, content_length);
        }
    };
    while response.len() < boundary + 4 + content_length {
        let count = stream.read(&mut buffer)?;
        if count == 0 {
            bail!("DevTools closed an incomplete HTTP response");
        }
        response.extend_from_slice(&buffer[..count]);
    }
    let head = std::str::from_utf8(&response[..boundary])?;
    if !head.starts_with("HTTP/1.1 200") {
        bail!(
            "DevTools endpoint returned {}",
            head.lines().next().unwrap_or("an invalid response")
        );
    }
    Ok(response[(boundary + 4)..].to_vec())
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
