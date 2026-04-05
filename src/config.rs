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
            model: "gemma".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            yes_warned: false,
            dangerous_patterns: vec![
                r"rm\s+(\S+\s+)+/".to_string(),
                r"mkfs".to_string(),
                r"dd\s+.*of=/dev/".to_string(),
                r":\(\)\{.*\|.*&\}.*;:".to_string(),
                r"chmod\s+777".to_string(),
                r">/dev/sd".to_string(),
                r"wget.*\|.*sh".to_string(),
                r"curl.*\|.*sh".to_string(),
            ],
            ctx_tools: vec![
                "git".into(),
                "docker".into(),
                "kubectl".into(),
                "systemctl".into(),
                "npm".into(),
                "python3".into(),
                "pip".into(),
                "cargo".into(),
                "go".into(),
                "apt".into(),
                "dnf".into(),
                "pacman".into(),
                "brew".into(),
                "jq".into(),
                "ripgrep".into(),
                "fd".into(),
                "tmux".into(),
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
        let content = fs::read_to_string(path).with_context(|| {
            format!(
                "Error: Invalid config at {}. Delete it to regenerate defaults.",
                path.display()
            )
        })?;
        let config: Config = toml::from_str(&content).with_context(|| {
            format!(
                "Error: Invalid config at {}. Delete it to regenerate defaults.",
                path.display()
            )
        })?;
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
    use super::*;
    use std::io::Write;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.model, "gemma");
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
        assert_eq!(
            config.dangerous_patterns.len(),
            deserialized.dangerous_patterns.len()
        );
        assert_eq!(config.ctx_tools.len(), deserialized.ctx_tools.len());
    }

    #[test]
    fn test_load_from_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let mut file = std::fs::File::create(&path).unwrap();
        write!(
            file,
            r#"
model = "gemma:7b"
ollama_url = "http://myhost:11434"
yes_warned = true
dangerous_patterns = ["rm -rf"]
ctx_tools = ["git"]
custom_prompt = "prefer ripgrep"
"#
        )
        .unwrap();

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
        assert_eq!(config.model, "gemma");
        assert!(path.exists());
    }

    #[test]
    fn test_config_path_default() {
        let path = Config::default_path();
        assert!(path.ends_with("shellex/config.toml"));
    }
}
