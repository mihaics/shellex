# Bug Fixes + High-Value Enhancements Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix five confirmed bugs (config parse failure, fish output collapsing, missing request timeout, weak dangerous-command confirm, safety-pattern gaps) and ship four high-value enhancements (chat endpoint, `<think>` stripping, unquoted multi-word input, shell keybinding integration).

**Architecture:** The Rust binary swaps `ollama-rs` for a thin `reqwest` client hitting `/api/chat` (fixes the timeout bug and gains compatibility with any Ollama-compatible server). Response parsing gains a `<think>`-block stripper. The lite scripts get the `/api/chat` switch **first** (so later whole-file commits can't smuggle endpoint hunks), then targeted fish/bash fixes. New `shell/` integration scripts insert generated commands into the interactive command line via `commandline -r` (fish) / `READLINE_LINE` (bash).

**Tech Stack:** Rust (clap 4, tokio 1, reqwest 0.12 + rustls, serde/serde_json, regex, crossterm), fish, bash, GitHub Actions.

> **Amendment (during execution):** the repo default model changed from `qwen3-coder` to `gemma4:12b` (user request, own commit after Task 2). This touched `src/config.rs` defaults/tests, every lite function default, README, and SHELLEX-LITE.md. The local working tree keeps only the `OLLAMA_URL` (`52625`) overrides uncommitted; the local model-default overrides were superseded by the new committed default.

## Global Constraints

- **Never commit the local FastFlowLM defaults.** The working tree has uncommitted changes setting `qwen2.5-it:3b` / `http://localhost:52625` as defaults in `lite/`. Repo defaults must stay `qwen3-coder` / `http://localhost:11434`. Commit procedure ("default dance") for affected lite files: `sed` the defaults back to upstream, `git add <file>`, **inspect `git diff --cached` and confirm no default/FastFlowLM hunks are staged**, commit, then `sed` the local defaults back (uncommitted). Task ordering guarantees each whole-file `git add` stages only intended hunks: the endpoint switch is committed first (Task 2), so later lite commits contain only that task's fix.
- **Before EVERY staging step:** assert the index is clean with `git diff --cached --quiet || echo "STALE INDEX - STOP"`. For every lite commit additionally run the mechanical guard `git diff --cached | grep -inE '52625|qwen2\.5|FastFlowLM'` and require empty output before committing.
- No tool attribution in commit messages, PR bodies, code comments, or docs.
- Conventional commit messages (`fix:`, `feat:`, `docs:`), matching repo history.
- Rust code must pass `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check`.
- Fish files must pass `fish -n <file>`; bash file must pass `bash -n <file>`.
- Lite shell regexes must use POSIX classes (`[[:space:]]`, `[^[:space:]]`) — `\s` is a GNU extension not guaranteed in BSD/macOS `grep -E`. Rust regex crate keeps `\s`.
- Known limitation (accepted, out of scope): users with an existing generated `config.toml` keep their old `dangerous_patterns`; defaults are not migrated into existing configs.
- Accepted over-warnings (safety patterns are substring regexes, not shell-aware): `rm -rf /tmp/x` matches the pre-existing absolute-path pattern; lite's broad `chmod NNN /` pattern matches `chmod 644 /tmp/x`; `grep "> /dev/sda" file` matches the redirect pattern; `echo rm -rf $HOME` matches. All merely trigger a confirmation prompt, never a block — over-warning is the intended failure direction.

---

### Task 1: Config parses with missing fields (serde defaults)

**Files:**
- Modify: `src/config.rs:6` (derive attr) + tests

**Interfaces:**
- Produces: `Config` deserializes from partial TOML; missing fields take `Config::default()` values. No signature changes.

- [ ] **Step 1: Write the failing test** (append inside `mod tests` in `src/config.rs`)

```rust
    #[test]
    fn test_partial_config_gets_defaults() {
        // A user config with only `model` set (like the README example, which
        // omits yes_warned) must parse, taking defaults for missing fields.
        let config: Config = toml::from_str(r#"model = "gemma:7b""#).unwrap();
        assert_eq!(config.model, "gemma:7b");
        assert_eq!(config.ollama_url, "http://localhost:11434");
        assert!(!config.yes_warned);
        assert!(!config.dangerous_patterns.is_empty());
        assert!(!config.ctx_tools.is_empty());
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_partial_config_gets_defaults`
Expected: FAIL with `missing field ...`

- [ ] **Step 3: Implement** — container-level serde default:

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Config {
```

- [ ] **Step 4: Run tests** — `cargo test config` → PASS

- [ ] **Step 5: Commit**

```bash
git add src/config.rs
git commit -m "fix: parse partial config files using defaults for missing fields"
```

---

### Task 2: Adopt /api/chat in shellex-lite (keep Ollama defaults) — FIRST lite commit

**Files:**
- Modify: `lite/fish/_ollama.fish`, `lite/fish/sx.fish`, `lite/bash/shellex-lite.bash` (endpoint hunks already in working tree from local diff; genericize comments, restore upstream defaults/header for the commit)
- Modify: `SHELLEX-LITE.md:195-206`

**Interfaces:**
- Produces: lite `_ollama` helpers POST to `/api/chat` with `messages:[{system},{user}]`, parse `.message.content`. Repo defaults stay `qwen3-coder` / `http://localhost:11434`.

**Why first:** the working tree already contains these endpoint hunks. Committing them now (intentionally) means every later whole-file `git add` on lite files can only pick up that task's own changes plus the defaults — and the defaults are handled by the dance.

- [ ] **Step 1: Genericize the comments** in `lite/fish/_ollama.fish` and `lite/bash/shellex-lite.bash` — replace the two-line flm comment with:

```
# Use /api/chat with an explicit system role — honored by Ollama and other
# Ollama-compatible servers, unlike /api/generate's `system` field.
```

- [ ] **Step 2: Restore the upstream header** of `lite/bash/shellex-lite.bash` (lines 1-10): title `LLM-powered shell functions via local Ollama`, `SX_MODEL - Ollama model (default: qwen3-coder)`, `OLLAMA_URL - Ollama API endpoint (default: http://localhost:11434)`, `Requires: curl, jq, ollama running locally`.

- [ ] **Step 3: Update `SHELLEX-LITE.md`** "How it works":
  - step 2 → `POST to http://localhost:11434/api/chat with a system + user message`
  - step 3 → `Parse the response with jq -r '.message.content'`
  - Reword the "key difference from ollama run" paragraph: the chat endpoint's explicit system role dramatically improves output quality and works across Ollama-compatible servers.

- [ ] **Step 4: Default dance + commit**

```bash
sed -i 's|http://localhost:52625|http://localhost:11434|g' lite/fish/_ollama.fish lite/fish/sx.fish lite/bash/shellex-lite.bash
sed -i 's/qwen2\.5-it:3b/qwen3-coder/g' lite/fish/sx.fish lite/bash/shellex-lite.bash
fish -n lite/fish/_ollama.fish && fish -n lite/fish/sx.fish && bash -n lite/bash/shellex-lite.bash
git add lite/fish/_ollama.fish lite/fish/sx.fish lite/bash/shellex-lite.bash SHELLEX-LITE.md
git diff --cached          # MUST show: endpoint/jq/comment/doc hunks only; no 52625, no qwen2.5
git commit -m "feat: use /api/chat in shellex-lite for Ollama-compatible servers"
# re-apply local FastFlowLM defaults (stay uncommitted)
sed -i 's|http://localhost:11434|http://localhost:52625|g' lite/fish/_ollama.fish lite/fish/sx.fish
sed -i 's|OLLAMA_URL:-http://localhost:11434|OLLAMA_URL:-http://localhost:52625|' lite/bash/shellex-lite.bash
sed -i 's/qwen3-coder/qwen2.5-it:3b/g' lite/fish/sx.fish
sed -i 's/SX_MODEL:-qwen3-coder/SX_MODEL:-qwen2.5-it:3b/g' lite/bash/shellex-lite.bash
git diff lite/            # MUST show only SX_MODEL/OLLAMA_URL default value hunks
```
(bash re-apply seds target only the `${VAR:-default}` sites so header comments stay upstream; the fish `_ollama.fish` URL sed is safe — the only 11434 occurrence is the default.)

---

### Task 3: fish `ai` agent output — preserve newlines / fix summarize branch

**Files:**
- Modify: `lite/fish/ai.fish:71-93`

**Interfaces:**
- Produces: agent mode prints command output verbatim (newlines intact); the ">20 lines → summarize" branch actually triggers; `$status` of the eval'd command is preserved (pipeline semantics: last command's status, same as any shell).

**Bug:** `set -l output (eval $cmd 2>&1)` splits into a fish list; `echo "$output"` re-joins with spaces, so newlines are lost and `wc -l` always sees 1 line — the summarize branch is dead code. Piping through `string collect` would clobber `$status`, so capture to a temp file.

- [ ] **Step 1: Replace lines 71-93 (from `set -l output (eval $cmd 2>&1)` through the end of the `if/else`) with:**

```fish
  set -l tmpout (mktemp); or return 1
  eval $cmd >$tmpout 2>&1
  set -l exit_code $status

  if test $exit_code -ne 0
    set_color brblack; echo "  (exit $exit_code)" >&2; set_color normal
  end

  # Step 3: summarize or print directly
  set -l line_count (wc -l <$tmpout | string trim)
  if test $line_count -le 20
    # Short output — just print it, no need for a second LLM call
    cat $tmpout
  else
    # Long output — cap at 100 lines and let LLM summarize
    set -l capped (head -100 $tmpout | string collect)
    _ollama $model \
      "You ran a command to answer the user's question. Give a clear, concise answer based on the output. No markdown fences. If the output is short enough, include the key data directly." \
      "Question: $instruction
Command: $cmd
Exit code: $exit_code
Output (first 100 lines):
$capped"
  end
  rm -f $tmpout
```

- [ ] **Step 2: Syntax check** — `fish -n lite/fish/ai.fish` → exit 0

- [ ] **Step 3: Runtime verification with real stubs** — write to scratchpad `test-ai.fish` and run `fish /path/test-ai.fish` from the repo root:

```fish
# Shadow isatty with a function that pretends stdin IS a TTY, so agent
# mode triggers even though the test runs without one. Stub _ollama:
# first call returns the "generated command", later calls the summarizer.
set -g _ollama_calls 0
function isatty; return 0; end
function _ollama
    set -g _ollama_calls (math $_ollama_calls + 1)
    if test $_ollama_calls -eq 1
        echo "seq 30"
    else
        echo "SUMMARY-CALLED"
    end
end
source lite/fish/ai.fish

# >20 lines → summarize branch
set -l out (ai count to thirty | string collect)
string match -q "*SUMMARY-CALLED*" -- $out; and echo "PASS: summarize branch"

# <=20 lines → verbatim output with newlines intact
set -g _ollama_calls 0
function _ollama
    set -g _ollama_calls (math $_ollama_calls + 1)
    if test $_ollama_calls -eq 1; echo "seq 5"; else; echo "UNEXPECTED"; end
end
set -l out2 (ai count to five | string collect)
test (echo $out2 | count) -ge 1
string match -q "*3
4*" -- $out2; and echo "PASS: newlines preserved"
```
Expected: both `PASS:` lines print.

- [ ] **Step 4: Default dance + commit**

```bash
sed -i 's/echo "qwen2\.5-it:3b"/echo "qwen3-coder"/' lite/fish/ai.fish
git add lite/fish/ai.fish
git diff --cached      # MUST show only the output-capture hunk
git commit -m "fix: preserve newlines in fish ai agent output"
sed -i 's/echo "qwen3-coder"/echo "qwen2.5-it:3b"/' lite/fish/ai.fish
```

---

### Task 4: `ai` dangerous-command confirm requires typing "yes"

**Files:**
- Modify: `lite/fish/ai.fish:66-69`
- Modify: `lite/bash/shellex-lite.bash:96-100`

**Interfaces:**
- Produces: on dangerous match, agent mode cancels unless the user types exactly `yes` (matches the Rust binary's behavior).

- [ ] **Step 1: fish** — replace the `if test $is_dangerous -eq 1` block with:

```fish
  if test $is_dangerous -eq 1
    set_color red; echo "⚠ dangerous command — not auto-executing" >&2; set_color normal
    read -P "Type 'yes' to run anyway: " confirm
    if test "$confirm" != yes
      echo "Cancelled." >&2
      return 1
    end
  end
```

- [ ] **Step 2: bash** — replace the `if echo "$cmd" | grep -qE ...` block with:

```bash
  if echo "$cmd" | grep -qE "$danger_patterns"; then
    echo -e "\033[31m⚠ dangerous command — not auto-executing\033[0m" >&2
    local confirm
    read -r -p "Type 'yes' to run anyway: " confirm
    if [ "$confirm" != "yes" ]; then echo "Cancelled." >&2; return 1; fi
  fi
```

- [ ] **Step 3: Syntax checks** — `fish -n lite/fish/ai.fish && bash -n lite/bash/shellex-lite.bash` → exit 0

- [ ] **Step 4: Default dance + commit**

```bash
sed -i 's/echo "qwen2\.5-it:3b"/echo "qwen3-coder"/' lite/fish/ai.fish
sed -i 's/SX_MODEL:-qwen2\.5-it:3b/SX_MODEL:-qwen3-coder/g; s|OLLAMA_URL:-http://localhost:52625|OLLAMA_URL:-http://localhost:11434|' lite/bash/shellex-lite.bash
git add lite/fish/ai.fish lite/bash/shellex-lite.bash
git diff --cached      # MUST show only the confirm-block hunks
git commit -m "fix: require typing yes to run dangerous commands in ai agent"
sed -i 's/echo "qwen3-coder"/echo "qwen2.5-it:3b"/' lite/fish/ai.fish
sed -i 's/SX_MODEL:-qwen3-coder/SX_MODEL:-qwen2.5-it:3b/g; s|OLLAMA_URL:-http://localhost:11434|OLLAMA_URL:-http://localhost:52625|' lite/bash/shellex-lite.bash
```

---

### Task 5: Close safety-pattern gaps

**Files:**
- Modify: `src/config.rs:22-31` (defaults), `src/safety.rs` (tests)
- Modify: `lite/fish/ai.fish:36-54` (pattern list — also convert to POSIX classes)
- Modify: `lite/bash/shellex-lite.bash:95` (`danger_patterns` — also convert to POSIX classes)
- Modify: `README.md:126-134` (sample `dangerous_patterns` — sync with new defaults)

**Interfaces:**
- Produces: default patterns catch `> /dev/sdX` (spaced) plus nvme/mmcblk/hd/vd devices, and `rm` of `~`, `$HOME`, `"$HOME"`, `${HOME}`. Lite patterns are POSIX-portable ERE.

- [ ] **Step 1: Failing tests** (append in `src/safety.rs` `mod tests`):

```rust
    #[test]
    fn test_redirect_to_device_with_space() {
        assert!(checker().check("echo x > /dev/sda").is_dangerous());
    }

    #[test]
    fn test_redirect_to_nvme_device() {
        assert!(checker().check("cat img > /dev/nvme0n1").is_dangerous());
    }

    #[test]
    fn test_rm_rf_home_tilde() {
        assert!(checker().check("rm -rf ~").is_dangerous());
    }

    #[test]
    fn test_rm_rf_home_var() {
        assert!(checker().check("rm -rf $HOME").is_dangerous());
    }

    #[test]
    fn test_rm_rf_home_var_quoted() {
        assert!(checker().check("rm -rf \"$HOME\"").is_dangerous());
    }

    #[test]
    fn test_rm_rf_home_var_braced() {
        assert!(checker().check("rm -rf ${HOME}").is_dangerous());
    }

    #[test]
    fn test_rm_rf_subdir_of_home_safe() {
        assert!(!checker().check("rm -rf ~/old-project/build").is_dangerous());
    }

    #[test]
    fn test_rm_rf_home_subdir_var_safe() {
        assert!(!checker().check("rm -rf $HOME/old-project/build").is_dangerous());
    }
```

- [ ] **Step 2: Run** — `cargo test safety` → the 6 positive tests FAIL, negatives PASS

- [ ] **Step 3: Update defaults in `src/config.rs`:**

```rust
            dangerous_patterns: vec![
                r"rm\s+(\S+\s+)+/".to_string(),
                r#"rm\s+(\S+\s+)*["']?(~|\$\{?HOME\}?)["']?/?(\s|$)"#.to_string(),
                r"mkfs".to_string(),
                r"dd\s+.*of=/dev/".to_string(),
                r":\(\)\{.*\|.*&\}.*;:".to_string(),
                r"chmod\s+777".to_string(),
                r">\s*/dev/(sd|hd|vd|nvme|mmcblk)".to_string(),
                r"wget.*\|.*sh".to_string(),
                r"curl.*\|.*sh".to_string(),
            ],
```
Note: `$HOME/...` stays safe because after the optional closing quote the pattern requires `/?(\s|$)` — a following path component fails the match. `rm -rf ~/x` fails because `~` must be followed by optional `/` then space/end.

- [ ] **Step 4: Run** — `cargo test` → all PASS

- [ ] **Step 5: Mirror in lite (POSIX classes)** — fish `ai.fish` list becomes:

```fish
  set -l dangerous_patterns \
    'rm[[:space:]]+([^[:space:]]+[[:space:]]+)+/' \
    'rm[[:space:]]+([^[:space:]]+[[:space:]]+)*["'"'"']?(~|\$\{?HOME\}?)["'"'"']?/?([[:space:]]|$)' \
    'mkfs' \
    'dd[[:space:]]+.*of=/dev/' \
    ':\(\)\{.*\|.*&\}.*;:' \
    'chmod[[:space:]]+777' \
    '>[[:space:]]*/dev/(sd|hd|vd|nvme|mmcblk)' \
    'wget.*\|.*sh' \
    'curl.*\|.*sh' \
    'sudo[[:space:]]+rm' \
    'sudo[[:space:]]+mkfs' \
    'sudo[[:space:]]+dd' \
    'reboot' \
    'shutdown' \
    'kill[[:space:]]+-9[[:space:]]+1([[:space:]]|$)' \
    'mv[[:space:]]+/[^[:space:]]' \
    'systemctl[[:space:]]+(stop|disable|mask)' \
    '>[[:space:]]*/etc/' \
    'chmod[[:space:]]+[0-7]*[0-7][[:space:]]+/'
```
(quote gymnastics: `["'"'"']?` embeds a single quote in a single-quoted fish string; verify with the Step 7 runtime check. `\b` replaced by `([[:space:]]|$)` — `\b` is also non-portable ERE.)

Bash `danger_patterns` single-quoted string, same alternation set joined with `|`:

```bash
  local danger_patterns='rm[[:space:]]+([^[:space:]]+[[:space:]]+)+/|rm[[:space:]]+([^[:space:]]+[[:space:]]+)*["'"'"']?(~|\$\{?HOME\}?)["'"'"']?/?([[:space:]]|$)|mkfs|dd[[:space:]]+.*of=/dev/|:\(\)\{.*\|.*&\}.*;:|chmod[[:space:]]+777|>[[:space:]]*/dev/(sd|hd|vd|nvme|mmcblk)|wget.*\|.*sh|curl.*\|.*sh|sudo[[:space:]]+rm|sudo[[:space:]]+mkfs|sudo[[:space:]]+dd|reboot|shutdown|kill[[:space:]]+-9[[:space:]]+1([[:space:]]|$)|mv[[:space:]]+/[^[:space:]]|systemctl[[:space:]]+(stop|disable|mask)|>[[:space:]]*/etc/|chmod[[:space:]]+[0-7]*[0-7][[:space:]]+/'
```
**Escaped-pipe caution:** inside the bash alternation the fork-bomb and wget/curl patterns keep their `\|` escapes exactly as today.

- [ ] **Step 6: Sync README sample** — replace `dangerous_patterns` at `README.md:126-134` with the new Rust defaults, TOML-escaped (`\\s`, `\\S`, `\\{`, etc.). Example lines:

```toml
dangerous_patterns = [
    "rm\\s+(\\S+\\s+)+/",
    "rm\\s+(\\S+\\s+)*[\"']?(~|\\$\\{?HOME\\}?)[\"']?/?(\\s|$)",
    "mkfs",
    "dd\\s+.*of=/dev/",
    "chmod\\s+777",
    ">\\s*/dev/(sd|hd|vd|nvme|mmcblk)",
    "wget.*\\|.*sh",
    "curl.*\\|.*sh",
]
```

- [ ] **Step 7: Runtime check of the shell patterns** (scratchpad script, plain `grep -E`):

```bash
p='rm[[:space:]]+([^[:space:]]+[[:space:]]+)*["'"'"']?(~|\$\{?HOME\}?)["'"'"']?/?([[:space:]]|$)'
echo 'rm -rf ~'          | grep -qE "$p" && echo PASS1
echo 'rm -rf "$HOME"'    | grep -qE "$p" && echo PASS2
echo 'rm -rf ${HOME}'    | grep -qE "$p" && echo PASS3
echo 'rm -rf ~/proj/sub' | grep -qE "$p" || echo PASS4
d='>[[:space:]]*/dev/(sd|hd|vd|nvme|mmcblk)'
echo 'echo x > /dev/sda'      | grep -qE "$d" && echo PASS5
echo 'cat img > /dev/nvme0n1' | grep -qE "$d" && echo PASS6
```
Expected: PASS1..PASS6.

- [ ] **Step 8: Verify + default dance + commit**

```bash
cargo test && fish -n lite/fish/ai.fish && bash -n lite/bash/shellex-lite.bash
sed -i 's/echo "qwen2\.5-it:3b"/echo "qwen3-coder"/' lite/fish/ai.fish
sed -i 's/SX_MODEL:-qwen2\.5-it:3b/SX_MODEL:-qwen3-coder/g; s|OLLAMA_URL:-http://localhost:52625|OLLAMA_URL:-http://localhost:11434|' lite/bash/shellex-lite.bash
git add src/config.rs src/safety.rs lite/fish/ai.fish lite/bash/shellex-lite.bash README.md
git diff --cached      # MUST show only pattern hunks
git commit -m "fix: catch spaced device redirects and rm of home dir in safety patterns"
sed -i 's/echo "qwen3-coder"/echo "qwen2.5-it:3b"/' lite/fish/ai.fish
sed -i 's/SX_MODEL:-qwen3-coder/SX_MODEL:-qwen2.5-it:3b/g; s|OLLAMA_URL:-http://localhost:11434|OLLAMA_URL:-http://localhost:52625|' lite/bash/shellex-lite.bash
```

---

### Task 6: Rust client → reqwest + /api/chat + timeout + wait indicator

**Files:**
- Modify: `Cargo.toml` (drop `ollama-rs`, add `reqwest`, `serde_json`)
- Rewrite: `src/ollama.rs`
- Modify: `README.md:182-191` ("How it works" step 1)

**Interfaces:**
- Produces: `OllamaClient::new(url: &str, model: &str) -> Result<Self>` and `async fn generate(&self, system_prompt: &str, user_prompt: &str) -> Result<String>` — signatures unchanged; `main.rs` untouched.

- [ ] **Step 1: Cargo.toml** — replace the `ollama-rs` line with:

```toml
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
serde_json = "1"
```

- [ ] **Step 2: Rewrite `src/ollama.rs`** (timeout is special-cased on BOTH send and body read):

```rust
// Ollama-compatible chat API client (/api/chat works on Ollama and compatible servers)
use anyhow::{bail, Context, Result};
use serde::Deserialize;
use serde_json::json;
use std::io::{stderr, IsTerminal, Write};
use std::time::Duration;

const REQUEST_TIMEOUT_SECS: u64 = 120;

pub struct OllamaClient {
    http: reqwest::Client,
    base_url: String,
    model: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    message: ChatMessage,
}

#[derive(Deserialize)]
struct ChatMessage {
    content: String,
}

/// Shows an hourglass on stderr while a request is in flight; clears on drop
/// so the indicator disappears on both success and error paths.
struct WaitIndicator(bool);

impl WaitIndicator {
    fn start() -> Self {
        let show = stderr().is_terminal();
        if show {
            eprint!("\u{23f3}");
            stderr().flush().ok();
        }
        WaitIndicator(show)
    }
}

impl Drop for WaitIndicator {
    fn drop(&mut self) {
        if self.0 {
            eprint!("\r\x1b[K");
            stderr().flush().ok();
        }
    }
}

fn timeout_error(base_url: &str) -> anyhow::Error {
    anyhow::anyhow!(
        "Error: Request to {} timed out after {}s. Model too slow or server hung.",
        base_url,
        REQUEST_TIMEOUT_SECS
    )
}

impl OllamaClient {
    pub fn new(url: &str, model: &str) -> Result<Self> {
        let http = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .build()
            .context("Error: Failed to build HTTP client")?;
        Ok(Self {
            http,
            base_url: url.trim_end_matches('/').to_string(),
            model: model.to_string(),
        })
    }

    pub async fn generate(&self, system_prompt: &str, user_prompt: &str) -> Result<String> {
        let body = json!({
            "model": self.model,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_prompt},
            ],
            "stream": false,
        });

        let _wait = WaitIndicator::start();
        let response = match self
            .http
            .post(format!("{}/api/chat", self.base_url))
            .json(&body)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) if e.is_timeout() => return Err(timeout_error(&self.base_url)),
            Err(e) => {
                return Err(e).with_context(|| {
                    format!(
                        "Error: Cannot connect to Ollama at {}. Is it running? (ollama serve)",
                        self.base_url
                    )
                })
            }
        };

        if !response.status().is_success() {
            let status = response.status();
            // Best-effort body read: on failure we still report the status.
            let text = response.text().await.unwrap_or_default();
            bail!("Error: Server returned {}: {}", status, text.trim());
        }

        let parsed: ChatResponse = match response.json().await {
            Ok(p) => p,
            Err(e) if e.is_timeout() => return Err(timeout_error(&self.base_url)),
            Err(e) => return Err(e).context("Error: Unexpected response from /api/chat"),
        };

        if parsed.message.content.trim().is_empty() {
            bail!("Error: Model returned empty response. Try a different model or rephrase.");
        }

        Ok(parsed.message.content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    #[test]
    fn test_new_trims_trailing_slash() {
        let client = OllamaClient::new("http://localhost:11434/", "m").unwrap();
        assert_eq!(client.base_url, "http://localhost:11434");
    }

    #[test]
    fn test_chat_response_deserializes() {
        let json =
            r#"{"model":"m","message":{"role":"assistant","content":"ls -la"},"done":true}"#;
        let parsed: ChatResponse = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.message.content, "ls -la");
    }

    /// One-shot HTTP server: accepts a single connection, reads the request,
    /// writes `body` as a JSON 200 (or the raw `raw` response), then closes.
    async fn one_shot_server(raw: String) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let (mut sock, _) = listener.accept().await.unwrap();
            let mut buf = vec![0u8; 65536];
            let _ = sock.read(&mut buf).await.unwrap();
            sock.write_all(raw.as_bytes()).await.unwrap();
            sock.shutdown().await.ok();
        });
        format!("http://{}", addr)
    }

    fn http_200(json_body: &str) -> String {
        format!(
            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
            json_body.len(),
            json_body
        )
    }

    #[tokio::test]
    async fn test_generate_happy_path() {
        let body = r#"{"message":{"role":"assistant","content":"ls -la"},"done":true}"#;
        let url = one_shot_server(http_200(body)).await;
        let client = OllamaClient::new(&url, "m").unwrap();
        let out = client.generate("sys", "user").await.unwrap();
        assert_eq!(out, "ls -la");
    }

    #[tokio::test]
    async fn test_generate_empty_content_errors() {
        let body = r#"{"message":{"role":"assistant","content":"  "},"done":true}"#;
        let url = one_shot_server(http_200(body)).await;
        let client = OllamaClient::new(&url, "m").unwrap();
        let err = client.generate("sys", "user").await.unwrap_err();
        assert!(err.to_string().contains("empty response"));
    }

    #[tokio::test]
    async fn test_generate_http_error_status() {
        let raw = "HTTP/1.1 404 Not Found\r\ncontent-length: 15\r\nconnection: close\r\n\r\nmodel not found".to_string();
        let url = one_shot_server(raw).await;
        let client = OllamaClient::new(&url, "m").unwrap();
        let err = client.generate("sys", "user").await.unwrap_err();
        assert!(err.to_string().contains("404"));
    }

    #[tokio::test]
    async fn test_generate_malformed_json_errors() {
        let url = one_shot_server(http_200("not json")).await;
        let client = OllamaClient::new(&url, "m").unwrap();
        let err = client.generate("sys", "user").await.unwrap_err();
        assert!(err.to_string().contains("Unexpected response"));
    }

    #[tokio::test]
    async fn test_generate_connection_refused_errors() {
        // Bind-then-drop to get a port nothing listens on.
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        drop(listener);
        let client = OllamaClient::new(&format!("http://{}", addr), "m").unwrap();
        let err = client.generate("sys", "user").await.unwrap_err();
        assert!(err.to_string().contains("Cannot connect"));
    }
}
```

- [ ] **Step 3: Build + test + lint** — `cargo test && cargo clippy -- -D warnings && cargo fmt` → PASS. If the one-shot server tests are flaky on request-read (request larger than one read), loop the read until `\r\n\r\n` is seen; content-length of our request bodies is small, one read suffices in practice.

- [ ] **Step 4: README** — "How it works" step 1 → `1. Your intent is sent to a local Ollama-compatible server (/api/chat) with a system prompt that constrains output to a single command`

- [ ] **Step 5: Live smoke test if a server is up:**

```bash
curl -s --max-time 2 http://localhost:11434/api/tags >/dev/null && \
  cargo run --release -- --yes --dry-run "list files in current directory"
```
(also try the FastFlowLM port via `OLLAMA_URL`-equivalent config if 11434 is down: `curl -s --max-time 2 http://localhost:52625/api/tags`). If no server: skip, note in summary.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock src/ollama.rs README.md
git commit -m "feat: use /api/chat via reqwest with timeout and wait indicator"
```

---

### Task 7: Strip `<think>` blocks from model responses

**Files:**
- Modify: `src/prompt.rs` (new fn + first line of `parse_generate_response` + tests)
- Modify: `src/main.rs:67-68` (explain path)

**Interfaces:**
- Produces: `pub fn strip_think_blocks(response: &str) -> String` in `src/prompt.rs`.

- [ ] **Step 1: Failing tests** (append in `src/prompt.rs` `mod tests`):

```rust
    #[test]
    fn test_parse_response_strips_think_block() {
        let response = "<think>\nThe user wants to list files.\n</think>\nls -la";
        assert_eq!(parse_generate_response(response), "ls -la");
    }

    #[test]
    fn test_parse_response_unclosed_think_yields_empty() {
        let response = "<think>reasoning that got cut off";
        assert_eq!(parse_generate_response(response), "");
    }

    #[test]
    fn test_strip_think_noop_without_block() {
        assert_eq!(strip_think_blocks("ls -la"), "ls -la");
    }
```

- [ ] **Step 2: Run** — `cargo test prompt` → compile error/failures as expected

- [ ] **Step 3: Implement in `src/prompt.rs`:**

```rust
/// Remove reasoning-model thinking blocks. Closed `<think>...</think>` blocks
/// are deleted; an unclosed `<think>` truncates the rest (it's all reasoning).
pub fn strip_think_blocks(response: &str) -> String {
    let re = Regex::new(r"(?s)<think>.*?</think>").unwrap();
    let without_closed = re.replace_all(response, "");
    let result = match without_closed.find("<think>") {
        Some(idx) => &without_closed[..idx],
        None => &without_closed,
    };
    result.trim().to_string()
}
```
`parse_generate_response` starts with:

```rust
pub fn parse_generate_response(response: &str) -> String {
    let response = strip_think_blocks(response);
    let trimmed = response.trim();
```
(rest of fn unchanged.)

- [ ] **Step 4: Explain path** — in `src/main.rs` `run_explain`:

```rust
    let response = client.generate(&system_prompt, &segments).await?;
    println!("{}", prompt::strip_think_blocks(&response));
```

- [ ] **Step 5: Run** — `cargo test && cargo clippy -- -D warnings` → PASS

- [ ] **Step 6: Commit**

```bash
git add src/prompt.rs src/main.rs
git commit -m "feat: strip reasoning-model think blocks from responses"
```

---

### Task 8: Accept unquoted multi-word input

**Files:**
- Modify: `src/cli.rs` (input becomes `Vec<String>` + `input_text()`; update tests)
- Modify: `src/main.rs` (use `input_text()`)
- Modify: `README.md` Usage (unquoted example + quoting caveats)

**Interfaces:**
- Produces: `Args::input_text(&self) -> String` (words joined with single spaces). `args.input` is `Vec<String>`.

**Accepted trade-offs (documented, not code):** intent words that look like flags (`--force`, `-e`) need `--` or quotes; explain mode joins words with single spaces, so commands whose meaning depends on quoting/spacing must be quoted (`shellex -e 'echo "a  b"'`). No `trailing_var_arg`, no `allow_hyphen_values` — `shellex list files --verbose` must keep parsing `--verbose` as a flag.

- [ ] **Step 1: Update `src/cli.rs`:**

```rust
    /// The natural-language intent (generate mode) or command to explain (with -e)
    #[arg(required = true, num_args = 1..)]
    pub input: Vec<String>,
```

```rust
impl Args {
    /// The positional words joined into the prompt text.
    pub fn input_text(&self) -> String {
        self.input.join(" ")
    }
}
```

- [ ] **Step 2: Update tests in `src/cli.rs`** — replace `args.input == "..."` assertions with `args.input_text() == "..."`; add:

```rust
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
```

- [ ] **Step 3: Update `src/main.rs`** — `run()` explain branch:

```rust
    if args.explain {
        run_explain(&client, &args.input_text()).await
    } else {
        run_generate(&client, &args, &mut config, &config_path).await
    }
```
In `run_generate`, add near the top `let input = args.input_text();` and use `&input` for the verbose print and the `client.generate(...)` call.

- [ ] **Step 4: Run** — `cargo test && cargo clippy -- -D warnings` → PASS (including `tests/cli_test.rs::test_no_args_fails`)

- [ ] **Step 5: README Usage** — add unquoted example and caveat:

```markdown
Quotes are optional for plain intents:

    shellex find all log files modified in the last week

Quote (or use `--`) when the intent contains flag-like words or shell
metacharacters, and always quote commands passed to `-e` whose meaning
depends on exact quoting/spacing.
```

- [ ] **Step 6: Commit**

```bash
git add src/cli.rs src/main.rs README.md
git commit -m "feat: accept unquoted multi-word intent"
```

---

### Task 9: Shell integration — insert generated command into the prompt line

**Files:**
- Create: `shell/shellex.fish`
- Create: `shell/shellex.bash`
- Modify: `src/main.rs:117-144` (`--yes` block: `--dry-run` skips the first-run warning/config write AND prints dangerous commands with warning on stderr)
- Modify: `tests/cli_test.rs` (hermetic integration tests for the dry-run gate via a fake one-shot LLM server)
- Modify: `README.md` (new "Shell integration" section; update Scripting-mode + Safety sections for the dry-run exception)

**Interfaces:**
- Consumes: `shellex --yes --dry-run -- <intent>` printing the command to stdout, warnings to stderr, exit 0.
- Produces: Alt+X keybinding in fish/bash replacing the typed intent with the generated command.

- [ ] **Step 1: Rework the `--yes` block in `src/main.rs`:**

```rust
    if args.yes {
        // Dry run prints without executing: skip the execution warning and
        // don't block dangerous commands (nothing runs) — but keep the
        // warning visible on stderr so a human reviews before running.
        if args.dry_run {
            if let safety::SafetyResult::Dangerous(patterns) = &safety_result {
                eprintln!("Warning: This command matches a dangerous pattern:");
                for p in patterns {
                    eprintln!("  - {}", p);
                }
            }
            println!("{}", command);
            return Ok(());
        }

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

        interactive::print_yes_mode(&command);
        return execute_command(&command);
    }
```
**Documented behavior change:** `--yes` without `--dry-run` still exits 2 on dangerous commands; `--yes --dry-run` now prints them (warning on stderr). Both README mentions (Scripting mode ~line 102 and Safety ~line 162) must be updated in Step 5. Hermetic tests in Step 1b — no live LLM needed.

- [ ] **Step 1b: Hermetic integration tests** (append to `tests/cli_test.rs`; `tempfile` is already a dev-dependency, no async needed):

```rust
use std::io::{Read, Write};
use std::net::TcpListener;

/// One-shot fake LLM server: answers the next `n` requests with `content`
/// as the chat message, then exits. Returns the base URL.
fn fake_llm_server(content: &str, n: usize) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let body = format!(
        r#"{{"message":{{"role":"assistant","content":"{}"}},"done":true}}"#,
        content
    );
    let response = format!(
        "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    std::thread::spawn(move || {
        for _ in 0..n {
            let (mut sock, _) = listener.accept().unwrap();
            let mut buf = [0u8; 65536];
            let _ = sock.read(&mut buf);
            let _ = sock.write_all(response.as_bytes());
        }
    });
    format!("http://{}", addr)
}

fn write_test_config(dir: &tempfile::TempDir, url: &str) -> std::path::PathBuf {
    let path = dir.path().join("config.toml");
    // Partial config: relies on serde defaults for dangerous_patterns etc.
    std::fs::write(
        &path,
        format!("model = \"test\"\nollama_url = \"{}\"\n", url),
    )
    .unwrap();
    path
}

#[test]
fn test_dry_run_prints_dangerous_command_with_warning() {
    let url = fake_llm_server("rm -rf /", 1);
    let dir = tempfile::tempdir().unwrap();
    let config = write_test_config(&dir, &url);

    let output = shellex()
        .args(["--config", config.to_str().unwrap(), "--yes", "--dry-run", "anything"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "expected exit 0, stderr: {}", stderr);
    assert_eq!(stdout.trim(), "rm -rf /");
    assert!(stderr.contains("dangerous pattern"), "stderr: {}", stderr);
    // dry-run must not trip the first-run execution warning
    assert!(!stderr.contains("executes without confirmation"), "stderr: {}", stderr);
}

#[test]
fn test_yes_mode_blocks_dangerous_command_exit_2() {
    let url = fake_llm_server("rm -rf /", 1);
    let dir = tempfile::tempdir().unwrap();
    let config = write_test_config(&dir, &url);

    let output = shellex()
        .args(["--config", config.to_str().unwrap(), "--yes", "anything"])
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(2), "stderr: {}", stderr);
    assert!(stderr.contains("Use --force"), "stderr: {}", stderr);
}
```
(If a test is ever flaky because the request spans multiple TCP segments, loop the server read until `\r\n\r\n` is seen; in practice one read suffices for these small requests.)

- [ ] **Step 2: Create `shell/shellex.fish`:**

```fish
# shellex shell integration for fish
# Source from config.fish:  source /path/to/shellex/shell/shellex.fish
# Type a natural-language description at the prompt, press Alt+X, and the
# line is replaced with the generated command — edit it, run it, it lands
# in your history like any other command.
# Rebind by calling:  bind \e<key> __shellex_transform

if status is-interactive
    function __shellex_transform
        set -l intent (commandline)
        if not string match -qr '\S' -- $intent
            return
        end
        set -l cmd (shellex --yes --dry-run -- $intent)
        if test -n "$cmd"
            commandline -r -- $cmd
        end
        commandline -f repaint
    end

    bind \ex __shellex_transform
end
```

- [ ] **Step 3: Create `shell/shellex.bash`:**

```bash
# shellex shell integration for bash
# Source from ~/.bashrc:  source /path/to/shellex/shell/shellex.bash
# Type a natural-language description at the prompt, press Alt+X, and the
# line is replaced with the generated command — edit it, run it, it lands
# in your history like any other command.
# Rebind by changing the bind line at the bottom.

case $- in *i*) ;; *) return 0 2>/dev/null || exit 0 ;; esac

__shellex_transform() {
  [ -z "${READLINE_LINE//[[:space:]]/}" ] && return
  local cmd err
  # Capture stderr: the in-flight indicator and warnings would otherwise
  # garble the readline display mid-edit.
  err=$(mktemp) || return
  cmd=$(shellex --yes --dry-run -- "$READLINE_LINE" 2>"$err")
  if [ -n "$cmd" ]; then
    READLINE_LINE="$cmd"
    READLINE_POINT=${#cmd}
    # Surface any warning (e.g. dangerous pattern) above the prompt.
    if [ -s "$err" ]; then printf '\n' >&2; cat "$err" >&2; fi
  else
    printf '\n' >&2; cat "$err" >&2
  fi
  rm -f "$err"
}

bind -x '"\ex": __shellex_transform'
```

- [ ] **Step 4: Checks**

```bash
fish -n shell/shellex.fish && bash -n shell/shellex.bash
# non-interactive source must be a no-op, not an error or shell-exit:
fish -c 'source shell/shellex.fish; echo FISH-SOURCED-OK'
bash -c 'source shell/shellex.bash && echo BASH-SOURCED-OK'
# `--` handling end-to-end (errors before reaching the server are fine;
# it must NOT be a clap "unexpected argument" error):
cargo run -- --yes --dry-run -- delete the --force flag docs 2>&1 | head -3
```
Expected: syntax checks pass, both `SOURCED-OK` lines print, clap accepts the `--force` word.

- [ ] **Step 5: README** — insert after "Scripting mode":

```markdown
### Shell integration (recommended)

Instead of letting shellex execute commands, put the generated command on
your prompt line — you get your shell's real line editor, completions, and
history.

    # fish (~/.config/fish/config.fish)
    source /path/to/shellex/shell/shellex.fish

    # bash (~/.bashrc)
    source /path/to/shellex/shell/shellex.bash

Type what you want, press **Alt+X**, review/edit the command, hit Enter.
```
Update Scripting-mode and Safety sections: dangerous commands exit 2 under `--yes`, **except** `--yes --dry-run`, which prints the command (warning on stderr) since nothing executes.

- [ ] **Step 6: Full suite** — `cargo test && cargo clippy -- -D warnings && cargo fmt --check` → PASS

- [ ] **Step 7: Commit**

```bash
git add shell/shellex.fish shell/shellex.bash src/main.rs README.md
git commit -m "feat: add fish and bash keybinding integration (Alt+X inserts command)"
```

---

### Task 10: Final verification sweep

- [ ] **Step 1:** `cargo test && cargo clippy -- -D warnings && cargo fmt --check`
- [ ] **Step 2:** `fish -c 'for f in lite/fish/*.fish shell/shellex.fish; fish -n $f; or echo "FAIL $f"; end'` and `bash -n lite/bash/shellex-lite.bash && bash -n shell/shellex.bash`
- [ ] **Step 3:** `git log --oneline -12` — review commit list. `git status --short` must show ONLY the FastFlowLM default hunks in `lite/` (`git diff lite/` to confirm: `SX_MODEL`/`OLLAMA_URL` default values only) plus untracked `.codex`.
- [ ] **Step 4:** If a local LLM server responds (try `http://localhost:11434` then `http://localhost:52625`), run end-to-end: `target/release/shellex --yes --dry-run find large files` (with config pointed at the live server) and the stubbed fish `ai` test from Task 3.
- [ ] **Step 5:** Commit the plan doc: `git add docs/superpowers/plans/2026-07-07-bugfixes-and-enhancements.md && git commit -m "docs: add bugfix and enhancement plan"`
