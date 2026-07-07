use clap::Parser;
use std::path::PathBuf;

/// Translate natural-language intent to shell commands, or explain existing commands.
#[derive(Parser, Debug)]
#[command(name = "shellex", version, about)]
pub struct Args {
    /// The natural-language intent (generate mode) or command to explain (with -e)
    #[arg(required = true, num_args = 1..)]
    pub input: Vec<String>,

    /// Explain mode: interpret the input as a shell command and explain it
    #[arg(short = 'e', long = "explain")]
    pub explain: bool,

    /// Gather environment context (OS, shell, installed tools) for better results
    #[arg(long, conflicts_with = "explain")]
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

impl Args {
    /// The positional words joined into the prompt text.
    pub fn input_text(&self) -> String {
        self.input.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_mode_basic() {
        let args = Args::parse_from(["shellex", "find large files"]);
        assert_eq!(args.input_text(), "find large files");
        assert!(!args.explain);
        assert!(!args.ctx);
        assert!(!args.yes);
        assert!(args.model.is_none());
    }

    #[test]
    fn test_explain_mode() {
        let args = Args::parse_from(["shellex", "-e", "tar czf - /var/log"]);
        assert!(args.explain);
        assert_eq!(args.input_text(), "tar czf - /var/log");
    }

    #[test]
    fn test_ctx_flag() {
        let args = Args::parse_from(["shellex", "--ctx", "list services"]);
        assert!(args.ctx);
        assert_eq!(args.input_text(), "list services");
    }

    #[test]
    fn test_generate_mode_unquoted_multiword() {
        let args = Args::parse_from(["shellex", "find", "large", "files"]);
        assert_eq!(args.input_text(), "find large files");
    }

    #[test]
    fn test_flags_still_parse_after_words() {
        let args = Args::parse_from(["shellex", "list", "files", "--verbose"]);
        assert!(args.verbose);
        assert_eq!(args.input_text(), "list files");
    }

    #[test]
    fn test_double_dash_allows_flaglike_words() {
        let args = Args::parse_from(["shellex", "--yes", "--", "remove", "the", "--force", "flag"]);
        assert!(args.yes);
        assert_eq!(args.input_text(), "remove the --force flag");
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
