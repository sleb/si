use assert_fs::prelude::*;

use std::process::Command;
use tempfile::tempdir;

const BINARY_NAME: &str = "si";

fn get_binary_path() -> std::path::PathBuf {
    // First try to find the binary in the target directory
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let target_dir = std::path::Path::new(&manifest_dir)
        .join("target")
        .join("debug")
        .join(BINARY_NAME);

    if target_dir.exists() {
        return target_dir;
    }

    // Fallback: try to find it relative to the test executable
    std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("deps")
        .join(BINARY_NAME)
}

#[test]
fn test_cli_help() {
    let mut cmd = Command::new(get_binary_path());
    cmd.arg("--help");

    let output = cmd.output().expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("A CLI for the Si (see) AI image generator"));
    assert!(stdout.contains("model"));
    assert!(stdout.contains("config"));
    assert!(stdout.contains("image"));
}

#[test]
fn test_cli_version() {
    let mut cmd = Command::new(get_binary_path());
    cmd.arg("--version");

    let output = cmd.output().expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("0.1.0"));
}

#[test]
fn test_model_help() {
    let mut cmd = Command::new(get_binary_path());
    cmd.args(["model", "--help"]);

    let output = cmd.output().expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Model-related operations"));
    assert!(stdout.contains("list"));
    assert!(stdout.contains("download"));
    assert!(stdout.contains("delete"));
    assert!(stdout.contains("show"));
}

#[test]
fn test_config_help() {
    let mut cmd = Command::new(get_binary_path());
    cmd.args(["config", "--help"]);

    let output = cmd.output().expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Configuration management"));
    assert!(stdout.contains("show"));
    assert!(stdout.contains("set"));
    assert!(stdout.contains("get"));
    assert!(stdout.contains("reset"));
}

#[test]
fn test_image_help() {
    let mut cmd = Command::new(get_binary_path());
    cmd.args(["image", "--help"]);

    let output = cmd.output().expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Image-related operations"));
    assert!(stdout.contains("generate"));
}

#[test]
fn test_model_list_no_models() {
    let mut cmd = Command::new(get_binary_path());
    cmd.args(["model", "list"]);

    // Set a temporary directory as the data directory
    let temp_dir = tempdir().unwrap();
    cmd.env("XDG_DATA_HOME", temp_dir.path());

    let output = cmd.output().expect("Failed to execute command");

    // Should fail because there's no model index
    assert!(!output.status.success());
}

#[test]
fn test_config_show() {
    let mut cmd = Command::new(get_binary_path());
    cmd.args(["config", "show"]);

    let output = cmd.output().expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Showing current configuration"));
}

#[test]
fn test_config_set() {
    let mut cmd = Command::new(get_binary_path());
    cmd.args(["config", "set", "test_key", "test_value"]);

    let output = cmd.output().expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Setting config: test_key = test_value"));
}

#[test]
fn test_config_get() {
    let mut cmd = Command::new(get_binary_path());
    cmd.args(["config", "get", "test_key"]);

    let output = cmd.output().expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Getting config value for: test_key"));
}

#[test]
fn test_config_reset() {
    let mut cmd = Command::new(get_binary_path());
    cmd.args(["config", "reset"]);

    let output = cmd.output().expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Resetting configuration to defaults"));
}

#[test]
fn test_model_delete() {
    let mut cmd = Command::new(get_binary_path());
    cmd.args(["model", "delete", "test-model"]);

    let output = cmd.output().expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Deleting model: test-model"));
}

#[test]
fn test_model_show() {
    let mut cmd = Command::new(get_binary_path());
    cmd.args(["model", "show", "test-model"]);

    let output = cmd.output().expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Showing details for model: test-model"));
}

#[test]
fn test_image_generate() {
    let temp_dir = assert_fs::TempDir::new().unwrap();
    let input_file = temp_dir.child("input.jpg");
    let output_file = temp_dir.child("output.png");

    // Create a dummy input file
    input_file.write_binary(b"fake image data").unwrap();

    let mut cmd = Command::new(get_binary_path());
    cmd.args([
        "image",
        "generate",
        "A beautiful sunset",
        "--model",
        "test-model",
        "--input",
        input_file.path().to_str().unwrap(),
        "--output",
        output_file.path().to_str().unwrap(),
    ]);

    let output = cmd.output().expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Generating image with prompt: A beautiful sunset"));
    assert!(stdout.contains("Using model: test-model"));
}

#[test]
fn test_invalid_command() {
    let mut cmd = Command::new(get_binary_path());
    cmd.arg("invalid-command");

    let output = cmd.output().expect("Failed to execute command");

    assert!(!output.status.success());
}

#[test]
fn test_model_download_missing_name() {
    let mut cmd = Command::new(get_binary_path());
    cmd.args(["model", "download"]);

    let output = cmd.output().expect("Failed to execute command");

    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("required") || stderr.contains("missing"));
}

#[test]
fn test_image_generate_missing_arguments() {
    let mut cmd = Command::new(get_binary_path());
    cmd.args(["image", "generate"]);

    let output = cmd.output().expect("Failed to execute command");

    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("required") || stderr.contains("missing"));
}

#[test]
fn test_config_set_missing_value() {
    let mut cmd = Command::new(get_binary_path());
    cmd.args(["config", "set", "key"]);

    let output = cmd.output().expect("Failed to execute command");

    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("required") || stderr.contains("missing"));
}

#[test]
fn test_config_get_missing_key() {
    let mut cmd = Command::new(get_binary_path());
    cmd.args(["config", "get"]);

    let output = cmd.output().expect("Failed to execute command");

    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("required") || stderr.contains("missing"));
}
