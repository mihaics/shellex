// Environment detection for --ctx flag

use std::env;
use std::process::Command;
use tokio::task::JoinSet;

pub fn detect_os() -> String {
    // Try /etc/os-release first (Linux) — parse PRETTY_NAME
    if let Ok(contents) = std::fs::read_to_string("/etc/os-release") {
        let pretty_name = contents
            .lines()
            .find(|line| line.starts_with("PRETTY_NAME="))
            .and_then(|line| line.strip_prefix("PRETTY_NAME="))
            .map(|val| val.trim_matches('"').to_string());

        if let Some(name) = pretty_name {
            let kernel = Command::new("uname")
                .arg("-r")
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .unwrap_or_default()
                .trim()
                .to_string();

            let arch = Command::new("uname")
                .arg("-m")
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .unwrap_or_default()
                .trim()
                .to_string();

            if kernel.is_empty() && arch.is_empty() {
                return name;
            } else if kernel.is_empty() {
                return format!("{} ({})", name, arch);
            } else if arch.is_empty() {
                return format!("{} (Linux {})", name, kernel);
            } else {
                return format!("{} (Linux {} {})", name, kernel, arch);
            }
        }
    }

    // Try sw_vers on macOS
    if let Ok(output) = Command::new("sw_vers").arg("-productVersion").output() {
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !version.is_empty() {
                return format!("macOS {}", version);
            }
        }
    }

    // Fallback to `uname -a`
    if let Ok(output) = Command::new("uname").arg("-a").output() {
        if output.status.success() {
            let info = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !info.is_empty() {
                return info;
            }
        }
    }

    // Final fallback
    "Unknown OS".to_string()
}

pub fn detect_shell() -> String {
    env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
}

pub fn detect_package_manager() -> String {
    let managers = ["apt", "dnf", "pacman", "brew", "zypper", "apk"];
    for manager in &managers {
        if let Ok(output) = Command::new("which").arg(manager).output() {
            if output.status.success() {
                return manager.to_string();
            }
        }
    }
    "unknown".to_string()
}

pub async fn check_tools(tools: &[&str]) -> Vec<String> {
    let mut set: JoinSet<Option<String>> = JoinSet::new();

    for tool in tools {
        let tool_owned = tool.to_string();
        set.spawn(async move {
            let output = tokio::process::Command::new("which")
                .arg(&tool_owned)
                .output()
                .await;
            match output {
                Ok(o) if o.status.success() => Some(tool_owned),
                _ => None,
            }
        });
    }

    let mut available = Vec::new();
    while let Some(result) = set.join_next().await {
        if let Ok(Some(tool)) = result {
            available.push(tool);
        }
    }

    available.sort();
    available
}

pub fn format_context_block(
    os: &str,
    shell: &str,
    package_manager: &str,
    tools: &[&str],
) -> String {
    let mut lines = Vec::new();
    lines.push(format!("OS: {}", os));
    lines.push(format!("Shell: {}", shell));
    lines.push(format!("Package manager: {}", package_manager));
    if !tools.is_empty() {
        lines.push(format!("Available tools: {}", tools.join(", ")));
    }
    lines.join("\n")
}

pub async fn gather_context(ctx_tools: &[String]) -> String {
    let os = detect_os();
    let shell = detect_shell();
    let package_manager = detect_package_manager();

    let tool_refs: Vec<&str> = ctx_tools.iter().map(|s| s.as_str()).collect();
    let available_tools = check_tools(&tool_refs).await;
    let tool_refs_available: Vec<&str> = available_tools.iter().map(|s| s.as_str()).collect();

    format_context_block(&os, &shell, &package_manager, &tool_refs_available)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_os_returns_something() {
        let os = detect_os();
        assert!(!os.is_empty());
    }

    #[test]
    fn test_detect_shell_returns_something() {
        let shell = detect_shell();
        assert!(!shell.is_empty());
    }

    #[test]
    fn test_detect_package_manager_linux() {
        let pm = detect_package_manager();
        assert!(!pm.is_empty());
    }

    #[test]
    fn test_format_context_block() {
        let block = format_context_block(
            "Ubuntu 24.04 (Linux 6.8.0 x86_64)",
            "/bin/bash",
            "apt",
            &["git", "docker", "cargo"],
        );
        assert!(block.contains("OS: Ubuntu 24.04"));
        assert!(block.contains("Shell: /bin/bash"));
        assert!(block.contains("Package manager: apt"));
        assert!(block.contains("Available tools: git, docker, cargo"));
    }

    #[test]
    fn test_format_context_block_no_tools() {
        let block = format_context_block("Linux", "/bin/sh", "apt", &[]);
        assert!(block.contains("OS: Linux"));
        assert!(!block.contains("Available tools"));
    }

    #[tokio::test]
    async fn test_check_tools_finds_at_least_one() {
        // "sh" should exist on any Unix system
        let available = check_tools(&["sh", "nonexistent_tool_xyz123"]).await;
        assert!(available.contains(&"sh".to_string()));
        assert!(!available.contains(&"nonexistent_tool_xyz123".to_string()));
    }
}
