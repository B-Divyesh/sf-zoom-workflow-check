use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn help_describes_the_real_workflow() {
    Command::cargo_bin("zoomcheck")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Record a real keyboard path"));
}

#[test]
fn init_writes_a_valid_document() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("flow.json");
    Command::cargo_bin("zoomcheck")
        .unwrap()
        .args(["init", "--out", out.to_str().unwrap()])
        .assert()
        .success();
    let value: serde_json::Value = serde_json::from_slice(&std::fs::read(out).unwrap()).unwrap();
    assert_eq!(value["version"], 1);
    assert!(value["steps"].as_array().unwrap().len() >= 2);
}
