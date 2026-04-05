// Command tokenizer for explain mode

/// Tokenize a shell command string into logical segments for explanation.
///
/// Splits on pipes `|`, logical operators `&&`/`||`, semicolons `;`, and
/// redirections `>`/`>>`. Quoted strings (single, double, backtick) and
/// `$(...)` subshells are kept intact. The pattern `2>&1` is kept with
/// its command rather than split on `>`. This is a best-effort heuristic,
/// not a full shell parser, and never evaluates or expands any input.
pub fn tokenize(input: &str) -> Vec<String> {
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();

    let mut tokens: Vec<String> = Vec::new();
    let mut buf = String::new();

    // Quote state: None, Single, Double, Backtick
    #[derive(PartialEq)]
    enum Quote {
        None,
        Single,
        Double,
        Backtick,
    }

    let mut quote = Quote::None;
    let mut subshell_depth: usize = 0;
    let mut i = 0;

    // Helper: flush buffer as a trimmed token if non-empty
    macro_rules! flush {
        () => {
            let trimmed = buf.trim().to_string();
            if !trimmed.is_empty() {
                tokens.push(trimmed);
            }
            buf.clear();
        };
    }

    while i < len {
        let ch = chars[i];

        // Inside a quoted region — only watch for the matching close quote
        if quote != Quote::None {
            match (&quote, ch) {
                (Quote::Single, '\'') => {
                    buf.push(ch);
                    quote = Quote::None;
                }
                (Quote::Double, '"') => {
                    buf.push(ch);
                    quote = Quote::None;
                }
                (Quote::Backtick, '`') => {
                    buf.push(ch);
                    quote = Quote::None;
                }
                // Backslash escape inside double quotes
                (Quote::Double, '\\') if i + 1 < len => {
                    buf.push(ch);
                    buf.push(chars[i + 1]);
                    i += 2;
                    continue;
                }
                _ => {
                    buf.push(ch);
                }
            }
            i += 1;
            continue;
        }

        // Inside a $(...) subshell — track nesting, watch for open/close parens
        if subshell_depth > 0 {
            match ch {
                '\'' => {
                    buf.push(ch);
                    quote = Quote::Single;
                }
                '"' => {
                    buf.push(ch);
                    quote = Quote::Double;
                }
                '`' => {
                    buf.push(ch);
                    quote = Quote::Backtick;
                }
                '(' => {
                    subshell_depth += 1;
                    buf.push(ch);
                }
                ')' => {
                    subshell_depth -= 1;
                    buf.push(ch);
                }
                _ => {
                    buf.push(ch);
                }
            }
            i += 1;
            continue;
        }

        // Normal (unquoted, not in subshell) state
        match ch {
            // Start of a $(...) subshell
            '$' if i + 1 < len && chars[i + 1] == '(' => {
                buf.push('$');
                buf.push('(');
                subshell_depth += 1;
                i += 2;
                continue;
            }

            // Open quotes
            '\'' => {
                buf.push(ch);
                quote = Quote::Single;
            }
            '"' => {
                buf.push(ch);
                quote = Quote::Double;
            }
            '`' => {
                buf.push(ch);
                quote = Quote::Backtick;
            }

            // `&&` logical AND
            '&' if i + 1 < len && chars[i + 1] == '&' => {
                flush!();
                tokens.push("&&".to_string());
                i += 2;
                continue;
            }

            // `||` logical OR (two chars, checked before single `|`)
            '|' if i + 1 < len && chars[i + 1] == '|' => {
                flush!();
                tokens.push("||".to_string());
                i += 2;
                continue;
            }

            // `|` pipe (single)
            '|' => {
                flush!();
                tokens.push("|".to_string());
            }

            // `;` semicolon
            ';' => {
                flush!();
                tokens.push(";".to_string());
            }

            // `>>` append redirect (two chars, checked before single `>`)
            '>' if i + 1 < len && chars[i + 1] == '>' => {
                // `2>>` stays with the command (fd redirect)
                if buf.trim_end().ends_with('2') {
                    buf.push('>');
                    buf.push('>');
                    i += 2;
                    continue;
                }
                flush!();
                tokens.push(">>".to_string());
                i += 2;
                continue;
            }

            // `>` redirect — but `2>&1` must stay with its command
            '>' => {
                // Check for `2>&1` pattern: current buffer ends with '2'
                // and next chars are `>&1` (i.e. chars[i..i+3] == ">&1")
                let buf_trimmed_ends_with_2 = buf.trim_end().ends_with('2');

                if buf_trimmed_ends_with_2 {
                    // Consume `>&1` (or `>&<digit>`) as part of the token
                    buf.push(ch); // '>'
                    i += 1;
                    // consume the rest of `&1` if present
                    while i < len && chars[i] != ' ' && chars[i] != '\t' {
                        buf.push(chars[i]);
                        i += 1;
                    }
                    continue;
                }

                flush!();
                tokens.push(">".to_string());
            }

            // Any other character
            _ => {
                buf.push(ch);
            }
        }

        i += 1;
    }

    // Flush remaining buffer (handles unmatched quotes too)
    {
        let trimmed = buf.trim().to_string();
        if !trimmed.is_empty() {
            tokens.push(trimmed);
        }
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
        let tokens =
            tokenize("tar czf - /var/log | ssh backup@remote 'cat > /backups/logs.tar.gz'");
        assert_eq!(
            tokens,
            vec![
                "tar czf - /var/log",
                "|",
                "ssh backup@remote 'cat > /backups/logs.tar.gz'",
            ]
        );
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
        let tokens = vec![
            "tar czf -".to_string(),
            "|".to_string(),
            "gzip".to_string(),
        ];
        let formatted = format_segments(&tokens);
        assert_eq!(formatted, "[1] tar czf -\n[2] |\n[3] gzip");
    }
}
