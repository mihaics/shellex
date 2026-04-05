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
