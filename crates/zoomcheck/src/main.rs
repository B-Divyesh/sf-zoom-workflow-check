mod browser;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use zoomcheck::{RunReport, Step, Workflow, report};

#[derive(Parser)]
#[command(
    name = "zoomcheck",
    version,
    about = "Replay a keyboard workflow at high browser zoom",
    long_about = "Record a real keyboard path, replay it using Chromium's native desktop page zoom at 200% and 400%, and write local visual evidence. This is an engineering check, not WCAG certification."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Record keyboard steps in a visible Chromium window (finish with Alt+Shift+S)
    Record {
        url: String,
        #[arg(long)]
        name: String,
        #[arg(long, default_value = ".zoomcheck/workflow.json")]
        out: PathBuf,
        #[arg(long)]
        browser: Option<PathBuf>,
        #[arg(long, default_value_t = 120)]
        timeout: u64,
    },
    /// Replay a workflow and write report.json plus a visual index.html
    Check {
        #[arg(default_value = ".zoomcheck/workflow.json")]
        workflow: PathBuf,
        #[arg(long, action=clap::ArgAction::Append, default_values_t=[200_u16,400_u16])]
        zoom: Vec<u16>,
        #[arg(long, default_value = "zoomcheck-report")]
        out: PathBuf,
        #[arg(long)]
        browser: Option<PathBuf>,
        #[arg(long)]
        headful: bool,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        quiet: bool,
    },
    /// Write an editable starter workflow
    Init {
        #[arg(long, default_value = ".zoomcheck/workflow.json")]
        out: PathBuf,
    },
    /// Run the bundled checkout sample in an isolated temporary folder
    Demo {
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long, action=clap::ArgAction::Append, default_values_t=[200_u16,400_u16])]
        zoom: Vec<u16>,
        #[arg(long)]
        browser: Option<PathBuf>,
        #[arg(long)]
        headful: bool,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        quiet: bool,
    },
}

fn main() -> ExitCode {
    match execute(Cli::parse()) {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("zoomcheck: {error:#}");
            ExitCode::from(2)
        }
    }
}

fn execute(cli: Cli) -> Result<u8> {
    match cli.command {
        Command::Init { out } => {
            init(&out)?;
            Ok(0)
        }
        Command::Record {
            url,
            name,
            out,
            browser,
            timeout,
        } => {
            let browser_path = browser::find_browser(browser.as_deref())?;
            browser::record(
                &url,
                &name,
                &out,
                &browser_path,
                Duration::from_secs(timeout),
            )?;
            Ok(0)
        }
        Command::Check {
            workflow,
            zoom,
            out,
            browser,
            headful,
            json,
            quiet,
        } => {
            let document: Workflow = serde_json::from_slice(
                &fs::read(&workflow).with_context(|| format!("read {}", workflow.display()))?,
            )
            .context("parse workflow JSON")?;
            check(document, zoom, out, browser, headful, json, quiet)
        }
        Command::Demo {
            out,
            zoom,
            browser,
            headful,
            json,
            quiet,
        } => demo(out, zoom, browser, headful, json, quiet),
    }
}

fn check(
    document: Workflow,
    zoom: Vec<u16>,
    out: PathBuf,
    browser: Option<PathBuf>,
    headful: bool,
    json: bool,
    quiet: bool,
) -> Result<u8> {
    document.validate()?;
    if zoom.is_empty() {
        anyhow::bail!("provide at least one --zoom value");
    }
    if zoom
        .iter()
        .any(|z| ![100, 110, 125, 150, 175, 200, 250, 300, 400].contains(z))
    {
        anyhow::bail!(
            "zoom must be a Chromium level: 100, 110, 125, 150, 175, 200, 250, 300, or 400"
        );
    }
    fs::create_dir_all(&out)?;
    let browser_path = browser::find_browser(browser.as_deref())?;
    let mut runs = Vec::new();
    for percent in zoom {
        if !quiet {
            eprintln!("Replaying {:?} at {percent}%…", document.name);
        }
        runs.push(browser::run_zoom(
            &document,
            percent,
            &out,
            &browser_path,
            !headful,
        )?);
    }
    let failures = runs.iter().map(|r| r.failures).sum();
    let warnings = runs.iter().map(|r| r.warnings).sum();
    let generated = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let result = RunReport {
        tool_version: env!("CARGO_PKG_VERSION").into(),
        workflow: document.name,
        url: document.url,
        generated_at: format!("Unix timestamp {generated}"),
        passed: failures == 0,
        runs,
        failures,
        warnings,
        disclaimer:
            "This report is focused engineering evidence, not legal advice or WCAG certification."
                .into(),
    };
    report::write_report(&result, &out)?;
    if json {
        println!("{}", serde_json::to_string(&result)?);
    } else if !quiet {
        eprintln!(
            "{} · {} failures · {} warnings · {}",
            if result.passed { "PASS" } else { "CHECK" },
            failures,
            warnings,
            out.join("index.html").display()
        );
    }
    Ok(if result.passed { 0 } else { 1 })
}

fn demo(
    out: Option<PathBuf>,
    zoom: Vec<u16>,
    browser: Option<PathBuf>,
    headful: bool,
    json: bool,
    quiet: bool,
) -> Result<u8> {
    let root = match out {
        Some(path) => {
            fs::create_dir_all(&path)?;
            path
        }
        None => tempfile::Builder::new()
            .prefix("zoomcheck-demo-")
            .tempdir()?
            .keep(),
    };
    let sample_dir = root.join("sample");
    fs::create_dir_all(&sample_dir)?;
    let html = sample_dir.join("checkout-flyout.html");
    fs::write(&html, include_str!("../examples/checkout-flyout.html"))?;
    let document = Workflow {
        version: 1,
        name: "Checkout flyout sample".into(),
        url: file_url(&html),
        settle_ms: 80,
        steps: vec![
            Step {
                key: "Tab".into(),
                expect: Some("#delivery-note".into()),
                note: Some("Reach the delivery note".into()),
            },
            Step {
                key: "Tab".into(),
                expect: Some("#continue-checkout".into()),
                note: Some("Reach the continue button".into()),
            },
            Step {
                key: "Tab".into(),
                expect: Some("#pay-securely".into()),
                note: Some("Reach the payment button".into()),
            },
        ],
    };
    let workflow_path = sample_dir.join("workflow.json");
    fs::write(&workflow_path, serde_json::to_vec_pretty(&document)?)?;
    if !quiet {
        eprintln!(
            "Demo sample and report are isolated at {}. Nothing is uploaded or saved to your workflow files.",
            root.display()
        );
    }
    check(document, zoom, root, browser, headful, json, quiet)
}

fn file_url(path: &Path) -> String {
    let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let text = path
        .to_string_lossy()
        .replace('%', "%25")
        .replace('#', "%23")
        .replace(' ', "%20");
    format!("file://{text}")
}

fn init(out: &Path) -> Result<()> {
    if out.exists() {
        anyhow::bail!("{} already exists", out.display());
    }
    let workflow = Workflow {
        version: 1,
        name: "Open the primary action".into(),
        url: "http://localhost:4173".into(),
        settle_ms: 180,
        steps: vec![
            Step {
                key: "Tab".into(),
                expect: Some("#primary-action".into()),
                note: Some("First meaningful focus stop".into()),
            },
            Step {
                key: "Enter".into(),
                expect: None,
                note: Some("Activate it".into()),
            },
        ],
    };
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(out, serde_json::to_vec_pretty(&workflow)?)?;
    eprintln!("Created {}", out.display());
    Ok(())
}
