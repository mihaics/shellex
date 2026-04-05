# shellex — Design Spec

**Date:** 2026-04-05
**Status:** Draft

## One-Sentence Pitch

Translates a natural-language intent into a shell command for your specific environment, or explains a given command — powered by a local LLM via Ollama.

## Core Constraints

- Two modes: generate and explain. No "and."
- One-shot only. Not a shell replacement, not a REPL, no multi-step workflows.
- No persistent state. Does not learn from corrections.
- Commands are never auto-executed. User must confirm (unless `--yes`).
- No data leaves the machine (Ollama is local).

---

## 1. CLI Interface

Two mutually exclusive modes via clap:

```
# Generate mode (default) — positional argument
shellex "find PNG files larger than 5MB"
shellex --ctx "list failed systemd services"

# Explain mode — -e/--explain flag
shellex -e "tar czf - /var/log | ssh backup@remote 'cat > /tmp/x.tar.gz'"

# Global flags
--model <name>       # Override default model
--ctx                # Gather environment context (generate mode only)
--yes                # Skip confirmation, execute immediately (scripting)
--dry-run            # With --yes: print command to stdout, don't execute
--force              # Allow --yes to proceed even on dangerous commands
--config <path>      # Custom config file path
--verbose            # Show the prompt sent to the LLM (debug)
```

`--yes` prints a one-time warning on first use: `"Warning: --yes mode executes without confirmation. You accept full responsibility."` Tracked via `yes_warned: bool` in config.

Generate and explain are mutually exclusive. If `-e` is present, the positional arg is treated as a command string. If absent, it's treated as natural-language intent.

---

## 2. Module Structure

```
src/
├── main.rs          # Entry point: parse args, load config, dispatch
├── cli.rs           # Clap derive structs, arg validation
├── config.rs        # Load/create ~/.config/shellex/config.toml
├── context.rs       # Environment detection (uname, shell, installed tools)
├── prompt.rs        # Build system/user prompts for generate & explain modes
├── safety.rs        # Dangerous command blocklist, pattern matching
├── ollama.rs        # ollama-rs wrapper: send prompt, stream response
├── interactive.rs   # Confirmation UX: display command, inline edit, run/cancel
└── explain.rs       # Tokenize command into segments for structured explanation
```

Single binary, single crate. No workspace, no library split.

### Data Flow — Generate Mode

1. `cli.rs` parses args, merges with `Config` from file
2. If `--ctx`: `context.rs` gathers env info, injected into system prompt
3. `prompt.rs` builds system + user prompt from intent + context
4. `ollama.rs` sends to model, streams response, extracts the command
5. `safety.rs` checks extracted command against blocklist
6. If top-level command found: `which <command>` validation (warn if missing)
7. `interactive.rs` presents command with inline-edit confirmation
8. On confirm: `std::process::Command::new(shell).arg("-c").arg(command)` with inherited stdio
9. Exit with child process's exit code

### Data Flow — Explain Mode

1. `cli.rs` parses args, command string captured as raw `&str` (never evaluated)
2. `explain.rs` tokenizes command into segments (pipes, redirects, subcommands)
3. `prompt.rs` builds prompt with tokenized segments asking LLM to annotate each
4. `ollama.rs` sends, streams explanation to stdout

---

## 3. Config File

Location: `~/.config/shellex/config.toml` (respects `$XDG_CONFIG_HOME`). Created with defaults on first run.

```toml
# Model to use with Ollama
model = "llama3"

# Ollama server URL
ollama_url = "http://localhost:11434"

# Skip --yes warning (set to true after first acknowledgment)
yes_warned = false

# Commands/patterns that trigger a red warning before confirmation
# Regex patterns matched against the generated command
dangerous_patterns = [
    "rm\\s+(-[^\\s]*)?\\s*/",
    "mkfs",
    "dd\\s+.*of=/dev/",
    ":(\\)\\{.*\\|.*&\\}.*;:)",
    "chmod\\s+777",
    ">\\/dev\\/sd",
    "wget.*\\|.*sh",
    "curl.*\\|.*sh",
]

# Tools to check for during --ctx environment detection
ctx_tools = [
    "git", "docker", "kubectl", "systemctl",
    "npm", "python3", "pip", "cargo", "go",
    "apt", "dnf", "pacman", "brew",
    "jq", "ripgrep", "fd", "tmux",
]

# Extra lines appended to the system prompt
# Example: "Always prefer ripgrep over grep. Use fd instead of find."
custom_prompt = ""
```

`dangerous_patterns` is defense-in-depth. Matching commands get a red warning and require explicit `yes` confirmation. Users can add/remove patterns.

---

## 4. Prompt Engineering

Two system prompts as string constants in `prompt.rs`.

### Generate Mode System Prompt

```
You are a shell command generator. Output ONLY the command, no explanation,
no markdown, no backticks. One single command or pipeline.

OS: {os}
Shell: {shell}
{context_block}
{custom_prompt}
```

`{context_block}` is empty unless `--ctx`, then:
```
Package manager: apt
Available tools: git, docker, systemctl, npm, cargo, jq, ripgrep
```

User message: the raw intent string verbatim.

### Explain Mode System Prompt

```
You are a shell command explainer. The user will provide a command broken into
numbered segments. For each segment, explain what it does in plain English.
Then provide a one-sentence overall summary at the top.

Format:
Summary: <one sentence>
Breakdown:
  [1] <segment> — <explanation>
  [2] <segment> — <explanation>
  ...
```

User message: pre-tokenized segments from `explain.rs`.

### Response Parsing (Generate Mode)

1. Strip leading/trailing whitespace
2. Strip markdown code fences if present (regex: `^```\w*\n?` and `\n?```$`)
3. If result contains newlines, take first line only (model gave multiple commands)

---

## 5. Safety & Execution

### Dangerous Command Detection (`safety.rs`)

- Compiled `regex::RegexSet` from `dangerous_patterns` in config (single-pass matching)
- Returns `SafetyResult` enum: `Safe` or `Dangerous(Vec<String>)` with matched pattern descriptions
- Checked after LLM response, before interactive prompt

### Warning UX

```
Warning: This command matches a dangerous pattern:
  - Recursive deletion from root
shellex generated: rm -rf /tmp/old_builds/

Type 'yes' to proceed, anything else cancels: _
```

Dangerous commands get this separate prompt — no inline editing. Accept verbatim or cancel.

### Command Existence Check

- Extract first token of generated command (before pipes/spaces)
- Run `which <token>` — if not found, warn: `"Warning: Command 'foo' not found on this system."` but still show confirmation (may be alias, may be valid elsewhere)

### Execution

- `std::process::Command::new(shell).arg("-c").arg(command)` where `shell = $SHELL` (fallback `/bin/sh`)
- Inherit stdin/stdout/stderr
- Exit with child's exit code

### `--yes` Mode

- Skips interactive prompt
- Prints the generated command to stderr, then executes it (stdout is the command's own output)
- Still runs safety check — dangerous commands print warning to stderr and exit 2 (distinct code) unless `--force` also passed
- To capture just the command text without executing: `shellex --yes --dry-run "intent"` (prints command to stdout, does not execute)

### `--dry-run` Flag

- Only meaningful with `--yes`. Prints the generated command to stdout and exits 0. Does not execute.
- Without `--yes`, has no effect (the interactive prompt already shows the command without executing).

---

## 6. Interactive Prompt (`interactive.rs`)

Uses `crossterm` raw terminal mode for inline editing. No full TUI framework.

### Normal Flow

```
$ shellex "find PNG files larger than 5MB in home"
> find ~/ -name "*.png" -size +5M -type f
  [Enter] Run  [Tab] Edit  [Esc] Cancel
```

### Edit Mode

Pressing Tab puts cursor into the command line as an editable input:
- Arrow keys to move cursor
- Home/End for line start/end
- Backspace/Delete
- Enter to confirm edited command (runs it)
- Esc to cancel

Implemented with `crossterm` raw mode for full control over the single-line editing UX.

### Verbose Mode

`--verbose` prints the full prompt above the result:
```
$ shellex --verbose --ctx "list failed services"
[system] You are a shell command generator...
[user] list failed services
> systemctl list-units --state=failed
  [Enter] Run  [Tab] Edit  [Esc] Cancel
```

---

## 7. Environment Detection (`context.rs`)

Gathered when `--ctx` is used. All checks are local, no network.

### Always Gathered

- `uname -a` output (OS, kernel, arch)
- `$SHELL` value
- Distro: `/etc/os-release` on Linux, `sw_vers` on macOS
- Package manager: inferred from distro (Ubuntu->apt, Fedora->dnf, Arch->pacman, macOS->brew)

### Tool Availability

- Iterate `ctx_tools` from config
- Run `which` for each via `tokio::spawn` (parallel async subprocesses)
- Collect available tools into a list; only available tools injected into prompt

### Performance

Parallel `which` checks stay under 100ms wall-clock for 20 tools.

### Output Format (injected as `{context_block}`)

```
OS: Ubuntu 24.04 LTS (Linux 6.8.0 x86_64)
Shell: /usr/bin/fish
Package manager: apt
Available tools: git, docker, systemctl, npm, cargo, jq, ripgrep, fd
```

Flat and terse — every token costs inference time on a local model.

---

## 8. Explain Mode Tokenizer (`explain.rs`)

Best-effort heuristic splitter, not a full shell parser.

### Splitting Rules

1. Split on pipes `|`
2. Split on logical operators `&&`, `||`
3. Split on semicolons `;`
4. Within each segment, separate redirections (`>`, `>>`, `2>&1`, `<`)
5. Keep quoted strings intact (single, double, backticks)
6. Keep `$(...)` subshells as single tokens

### Example

```
Input:  tar czf - /var/log | ssh backup@remote 'cat > /backups/logs-$(date +%F).tar.gz'
Output:
  [1] tar czf -
  [2] /var/log
  [3] |
  [4] ssh backup@remote
  [5] 'cat > /backups/logs-$(date +%F).tar.gz'
```

### Safety

The tokenizer operates on the string as bytes. Never invokes a shell, never evaluates `$(...)`, never expands backticks. `$(rm -rf /)` passes through as a literal token.

### Edge Cases

- Unmatched quotes: treat rest of string as one token
- Nested `$(...)`: count parens for matching
- Heredocs (`<<EOF`): grab everything until delimiter as one token

Doesn't need to be perfect — it's a hint to reduce LLM hallucination.

---

## 9. Dependencies

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
ollama-rs = "0.2"
serde = { version = "1", features = ["derive"] }
toml = "0.8"
regex = "1"
crossterm = "0.28"
dirs = "5"
anyhow = "1"
```

9 crates total (including `anyhow`). No bloat.

---

## 10. Error Handling

`anyhow::Result<()>` everywhere, `?` propagation, user-friendly messages to stderr.

| Condition | Message | Exit Code |
|-----------|---------|-----------|
| Ollama not running | `"Error: Cannot connect to Ollama at {url}. Is it running? (ollama serve)"` | 1 |
| Model not found | `"Error: Model '{model}' not found. Run 'ollama pull {model}' first."` | 1 |
| Config parse error | `"Error: Invalid config at {path}: {details}. Delete it to regenerate defaults."` | 1 |
| Empty LLM response | `"Error: Model returned empty response. Try a different model or rephrase."` | 1 |
| `--yes` + dangerous (no `--force`) | Warning to stderr | 2 |

No retries. One-shot tool, one inference call. If it fails, the user reruns.

No panics anywhere. Every failure path is a clean error message.

---

## 11. Testing Strategy

### Unit Tests (in-module `#[cfg(test)]`)

- **`config.rs`** — default generation, TOML round-trip, merge with CLI overrides, XDG path resolution
- **`safety.rs`** — each default pattern: at least 3 positive matches, 3 negative matches. RegexSet compiles without error.
- **`explain.rs`** — tokenizer: pipes, quotes, nested `$(...)`, heredocs, unmatched quotes
- **`prompt.rs`** — template substitution, context block injection, markdown fence stripping

### Integration Tests (`tests/`)

- **`cli_test.rs`** — clap rejects invalid combos (`-e` + `--ctx`), `--help` works, `--yes` without intent fails
- **`context_test.rs`** — detects OS and shell on any CI runner

### Manual / Ignored Tests

- `#[ignore]` test that requires running Ollama instance — sends a real prompt, validates response is non-empty and doesn't contain markdown fences. Run with `cargo test --ignored`.

### No Ollama Mocking

The Ollama interaction is thin (send prompt, get string). Testing prompt construction and response parsing covers the logic.

---

## Non-Goals

- Not a shell replacement
- Not an interactive REPL
- Does not execute multi-step workflows
- Does not learn from corrections (no persistent state)
- Does not handle piped input
- Does not validate individual flags (only top-level command existence)
- No Windows support at MVP (Linux/macOS only)
- No provider abstraction (Ollama-only)
