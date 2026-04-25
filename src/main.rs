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

enum FinalAction {
    RunSafe(String),
    RunDangerous(String, Vec<String>),
    Cancel,
}

fn classify_final_action(action: UserAction, checker: &safety::SafetyChecker) -> FinalAction {
    match action {
        UserAction::Run(cmd) => match checker.check(&cmd) {
            safety::SafetyResult::Safe => FinalAction::RunSafe(cmd),
            safety::SafetyResult::Dangerous(patterns) => FinalAction::RunDangerous(cmd, patterns),
        },
        UserAction::Cancel => FinalAction::Cancel,
    }
}

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
    let config_path = args.config.clone().unwrap_or_else(Config::default_path);
    let mut config = Config::load_or_create(&config_path)?;

    // Resolve model (CLI flag overrides config)
    let model = args.model.clone().unwrap_or_else(|| config.model.clone());

    // Create Ollama client
    let client = ollama::OllamaClient::new(&config.ollama_url, &model)?;

    if args.explain {
        run_explain(&client, &args.input).await
    } else {
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
    let os_info = context::detect_os();
    let shell_info = context::detect_shell();

    let context_block = if args.ctx {
        Some(context::gather_context(&config.ctx_tools).await)
    } else {
        None
    };

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
    let initially_dangerous = safety_result.is_dangerous();
    let action = if initially_dangerous {
        if let safety::SafetyResult::Dangerous(ref patterns) = safety_result {
            interactive::prompt_dangerous(&command, patterns)?
        } else {
            unreachable!()
        }
    } else {
        interactive::prompt_command(&command)?
    };

    let final_action = if initially_dangerous {
        match action {
            UserAction::Run(cmd) => FinalAction::RunSafe(cmd),
            UserAction::Cancel => FinalAction::Cancel,
        }
    } else {
        classify_final_action(action, &checker)
    };

    match final_action {
        FinalAction::RunSafe(cmd) => execute_command(&cmd),
        FinalAction::RunDangerous(cmd, patterns) => {
            match interactive::prompt_dangerous(&cmd, &patterns)? {
                UserAction::Run(cmd) => execute_command(&cmd),
                UserAction::Cancel => {
                    eprintln!("Cancelled.");
                    Ok(())
                }
            }
        }
        FinalAction::Cancel => {
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
    let Some(first_token) = command_lookup_token(command) else {
        return;
    };

    // Shell builtins won't be found by `which` — skip them
    const BUILTINS: &[&str] = &[
        "cd", "export", "source", "alias", "unalias", "set", "unset", "echo", "printf", "read",
        "eval", "exec", "exit", "return", "break", "continue", "shift", "trap", "type", "hash",
        "ulimit", "umask", "wait", "jobs", "fg", "bg", "disown", "pushd", "popd", "dirs",
        "builtin", "command", "declare", "local", "readonly", "let", "test", "[", ".",
    ];

    if BUILTINS.contains(&first_token) {
        return;
    }

    let result = tokio::process::Command::new("which")
        .arg(first_token)
        .output()
        .await;

    if let Ok(output) = result {
        if !output.status.success() {
            eprintln!(
                "Warning: Command '{}' not found on this system.",
                first_token
            );
        }
    }
}

fn command_lookup_token(command: &str) -> Option<&str> {
    let mut tokens = command.split_whitespace().peekable();

    while let Some(token) = tokens.next() {
        if token.contains('=') && !token.starts_with('-') {
            let mut parts = token.splitn(2, '=');
            let name = parts.next().unwrap_or_default();
            if !name.is_empty()
                && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                && !name.chars().next().unwrap().is_ascii_digit()
            {
                continue;
            }
        }

        match token {
            "sudo" | "doas" | "command" | "builtin" | "exec" => continue,
            "env" => {
                while let Some(next) = tokens.peek().copied() {
                    if next.starts_with('-') {
                        tokens.next();
                        continue;
                    }
                    if next.contains('=') {
                        tokens.next();
                        continue;
                    }
                    break;
                }
                continue;
            }
            _ => return Some(token),
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn checker() -> safety::SafetyChecker {
        safety::SafetyChecker::new(&Config::default().dangerous_patterns).unwrap()
    }

    #[test]
    fn final_safe_run_stays_safe() {
        let result = classify_final_action(UserAction::Run("ls -la".to_string()), &checker());
        assert!(matches!(result, FinalAction::RunSafe(cmd) if cmd == "ls -la"));
    }

    #[test]
    fn final_edited_dangerous_command_requires_confirmation() {
        let result = classify_final_action(UserAction::Run("rm -rf /".to_string()), &checker());
        assert!(matches!(
            result,
            FinalAction::RunDangerous(cmd, patterns)
                if cmd == "rm -rf /" && patterns.iter().any(|p| p.contains("rm"))
        ));
    }

    #[test]
    fn final_cancel_stays_cancelled() {
        let result = classify_final_action(UserAction::Cancel, &checker());
        assert!(matches!(result, FinalAction::Cancel));
    }

    #[test]
    fn command_lookup_skips_environment_assignments() {
        assert_eq!(
            command_lookup_token("FOO=1 BAR=baz make test"),
            Some("make")
        );
    }

    #[test]
    fn command_lookup_unwraps_common_wrappers() {
        assert_eq!(command_lookup_token("sudo apt install htop"), Some("apt"));
        assert_eq!(command_lookup_token("env FOO=1 cargo test"), Some("cargo"));
        assert_eq!(command_lookup_token("command git status"), Some("git"));
    }
}
