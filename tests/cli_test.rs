use assert_cmd::Command;
use serde_json::Value;
use std::path::Path;
use tempfile::TempDir;

fn runtime_envs(base: &Path) -> [(&'static str, String); 4] {
    [
        (
            "QRCODE_AGENT_CLI_CONFIG_DIR",
            base.join("config").display().to_string(),
        ),
        (
            "QRCODE_AGENT_CLI_DATA_DIR",
            base.join("data").display().to_string(),
        ),
        (
            "QRCODE_AGENT_CLI_STATE_DIR",
            base.join("state").display().to_string(),
        ),
        (
            "QRCODE_AGENT_CLI_CACHE_DIR",
            base.join("cache").display().to_string(),
        ),
    ]
}

#[test]
fn top_level_invocation_shows_help() {
    Command::cargo_bin("qrcode-agent-cli")
        .expect("binary exists")
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "Generate QR codes as terminal text or PNG images",
        ));
}

#[test]
fn help_run_can_be_rendered_as_json() {
    let output = Command::cargo_bin("qrcode-agent-cli")
        .expect("binary exists")
        .args(["help", "run", "--format", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let payload: Value = serde_json::from_slice(&output).expect("valid json");
    assert_eq!(
        payload["command_path"],
        serde_json::json!(["qrcode-agent-cli", "run"])
    );
}

#[test]
fn run_text_emits_qr_blocks() {
    let output = Command::cargo_bin("qrcode-agent-cli")
        .expect("binary exists")
        .args(["run", "hello from test", "--render", "text"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).expect("utf8 output");
    assert!(stdout.lines().count() > 2, "expected multiline qr output");
}

#[test]
fn run_image_writes_png_and_returns_metadata() {
    let temp_dir = TempDir::new().expect("temp dir");
    let image_path = temp_dir.path().join("qr.png");

    let output = Command::cargo_bin("qrcode-agent-cli")
        .expect("binary exists")
        .args([
            "run",
            "hello image",
            "--render",
            "image",
            "--output",
            image_path.to_str().expect("utf8 path"),
            "--format",
            "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    assert!(image_path.exists(), "expected png file to be created");
    let payload: Value = serde_json::from_slice(&output).expect("valid json");
    assert_eq!(payload["render"], "image");
    assert_eq!(payload["image_path"], image_path.display().to_string());
}

#[test]
fn missing_content_returns_structured_error() {
    let output = Command::cargo_bin("qrcode-agent-cli")
        .expect("binary exists")
        .args(["run", "--render", "image", "--format", "json"])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let payload: Value = serde_json::from_slice(&output).expect("valid json");
    assert_eq!(payload["code"], "missing_content");
}

#[test]
fn context_can_be_persisted_and_shown() {
    let temp_dir = TempDir::new().expect("temp dir");
    let envs = runtime_envs(temp_dir.path());

    let mut set_command = Command::cargo_bin("qrcode-agent-cli").expect("binary exists");
    for (key, value) in &envs {
        set_command.env(key, value);
    }
    set_command
        .args([
            "context", "set", "--render", "image", "--size", "512", "--format", "json",
        ])
        .assert()
        .success();

    let mut show_command = Command::cargo_bin("qrcode-agent-cli").expect("binary exists");
    for (key, value) in &envs {
        show_command.env(key, value);
    }
    let output = show_command
        .args(["context", "show", "--format", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let payload: Value = serde_json::from_slice(&output).expect("valid json");
    assert_eq!(payload["active_context"]["default_render"], "image");
    assert_eq!(payload["active_context"]["default_image_size"], 512);
}

#[test]
fn paths_supports_toml_output() {
    let output = Command::cargo_bin("qrcode-agent-cli")
        .expect("binary exists")
        .args(["paths", "--format", "toml"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let parsed: toml::Value =
        toml::from_str(&String::from_utf8(output).expect("utf8 output")).expect("valid toml");
    assert!(parsed.get("runtime_directories").is_some());
}
