// System/user prompt construction for both modes
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
