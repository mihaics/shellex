/// This test requires a running Ollama instance.
/// Run with: cargo test --test ollama_test -- --ignored
#[tokio::test]
#[ignore]
async fn test_generate_returns_nonempty_response() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_shellex"))
        .args(["--yes", "--dry-run", "list files in current directory"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !stdout.trim().is_empty() || !stderr.trim().is_empty(),
        "Expected non-empty output. stdout: '{}', stderr: '{}'",
        stdout,
        stderr
    );

    assert!(
        !stdout.contains("```"),
        "Response should not contain markdown fences"
    );
}

#[tokio::test]
#[ignore]
async fn test_explain_returns_explanation() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_shellex"))
        .args(["-e", "ls -la /tmp"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.trim().is_empty(), "Expected non-empty explanation");
}
