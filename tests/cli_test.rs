use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::Command;

fn shellex() -> Command {
    Command::new(env!("CARGO_BIN_EXE_shellex"))
}

/// One-shot fake LLM server: answers the next `n` requests with `content`
/// as the chat message, then exits. Returns the base URL.
fn fake_llm_server(content: &str, n: usize) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let body = format!(
        r#"{{"message":{{"role":"assistant","content":"{}"}},"done":true}}"#,
        content
    );
    let response = format!(
        "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    std::thread::spawn(move || {
        for _ in 0..n {
            let (mut sock, _) = listener.accept().unwrap();
            let mut buf = [0u8; 65536];
            let _ = sock.read(&mut buf);
            let _ = sock.write_all(response.as_bytes());
        }
    });
    format!("http://{}", addr)
}

fn write_test_config(dir: &tempfile::TempDir, url: &str) -> std::path::PathBuf {
    let path = dir.path().join("config.toml");
    // Partial config: relies on serde defaults for dangerous_patterns etc.
    std::fs::write(
        &path,
        format!("model = \"test\"\nollama_url = \"{}\"\n", url),
    )
    .unwrap();
    path
}

#[test]
fn test_dry_run_prints_dangerous_command_with_warning() {
    let url = fake_llm_server("rm -rf /", 1);
    let dir = tempfile::tempdir().unwrap();
    let config = write_test_config(&dir, &url);

    let output = shellex()
        .args([
            "--config",
            config.to_str().unwrap(),
            "--yes",
            "--dry-run",
            "anything",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "expected exit 0, stderr: {}",
        stderr
    );
    assert_eq!(stdout.trim(), "rm -rf /");
    assert!(stderr.contains("dangerous pattern"), "stderr: {}", stderr);
    // dry-run must not trip the first-run execution warning
    assert!(
        !stderr.contains("executes without confirmation"),
        "stderr: {}",
        stderr
    );
}

#[test]
fn test_yes_mode_blocks_dangerous_command_exit_2() {
    let url = fake_llm_server("rm -rf /", 1);
    let dir = tempfile::tempdir().unwrap();
    let config = write_test_config(&dir, &url);

    let output = shellex()
        .args(["--config", config.to_str().unwrap(), "--yes", "anything"])
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(2), "stderr: {}", stderr);
    assert!(stderr.contains("Use --force"), "stderr: {}", stderr);
}

#[test]
fn test_help_flag() {
    let output = shellex().arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("shellex"));
    assert!(stdout.contains("--explain"));
    assert!(stdout.contains("--ctx"));
    assert!(stdout.contains("--yes"));
}

#[test]
fn test_version_flag() {
    let output = shellex().arg("--version").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("shellex"));
}

#[test]
fn test_no_args_fails() {
    let output = shellex().output().unwrap();
    assert!(!output.status.success());
}

#[test]
fn test_fish_sx_respects_documented_ollama_url() {
    let sx = std::fs::read_to_string("lite/fish/sx.fish").unwrap();
    assert!(sx.contains("OLLAMA_URL"));
    assert!(!sx.contains("SX_OLLAMA_URL"));
}
