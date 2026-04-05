use std::process::Command;

fn shellex() -> Command {
    Command::new(env!("CARGO_BIN_EXE_shellex"))
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
