use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub version: u8,
    pub name: String,
    pub url: String,
    #[serde(default = "default_settle")]
    pub settle_ms: u64,
    pub steps: Vec<Step>,
}

fn default_settle() -> u64 {
    180
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expect: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl Workflow {
    pub fn validate(&self) -> Result<()> {
        if self.version != 1 {
            bail!("unsupported workflow version {}; expected 1", self.version);
        }
        if self.name.trim().is_empty() {
            bail!("workflow name cannot be empty");
        }
        if !(self.url.starts_with("http://")
            || self.url.starts_with("https://")
            || self.url.starts_with("file://"))
        {
            bail!("workflow URL must use http://, https://, or file://");
        }
        if self.steps.is_empty() {
            bail!("workflow has no steps; record a path or add at least one key step");
        }
        if self.steps.len() > 200 {
            bail!("workflow has {} steps; maximum is 200", self.steps.len());
        }
        for (i, step) in self.steps.iter().enumerate() {
            if !is_allowed_key(&step.key) {
                bail!("step {} uses unsupported key {:?}", i + 1, step.key);
            }
        }
        Ok(())
    }
}

pub fn is_allowed_key(key: &str) -> bool {
    matches!(
        key,
        "Tab"
            | "Shift+Tab"
            | "Enter"
            | "Space"
            | "ArrowUp"
            | "ArrowDown"
            | "ArrowLeft"
            | "ArrowRight"
            | "Escape"
            | "Home"
            | "End"
    )
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Pass,
    Warning,
    Failure,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FindingKind {
    FocusOrder,
    FocusName,
    FocusVisible,
    ViewportClipping,
    AncestorClipping,
    Obscured,
    Runtime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub severity: Severity,
    pub kind: FindingKind,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StepResult {
    pub number: usize,
    pub key: String,
    pub selector: String,
    pub element: String,
    pub accessible_name: String,
    pub rect: Rect,
    pub viewport_width: f64,
    pub viewport_height: f64,
    pub scroll_x: f64,
    pub scroll_y: f64,
    pub scrollable: bool,
    pub findings: Vec<Finding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZoomRun {
    pub zoom: u16,
    pub viewport_css: String,
    pub screenshot: String,
    pub steps: Vec<StepResult>,
    pub failures: usize,
    pub warnings: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunReport {
    pub tool_version: String,
    pub workflow: String,
    pub url: String,
    pub generated_at: String,
    pub passed: bool,
    pub runs: Vec<ZoomRun>,
    pub failures: usize,
    pub warnings: usize,
    pub disclaimer: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_documented_keys() {
        for key in ["Tab", "Shift+Tab", "Enter", "Space", "ArrowDown", "Escape"] {
            assert!(is_allowed_key(key));
        }
        assert!(!is_allowed_key("Control+L"));
    }

    #[test]
    fn rejects_empty_workflow() {
        let workflow = Workflow {
            version: 1,
            name: "Empty".into(),
            url: "https://example.com".into(),
            settle_ms: 180,
            steps: vec![],
        };
        assert!(
            workflow
                .validate()
                .unwrap_err()
                .to_string()
                .contains("no steps")
        );
    }
}
