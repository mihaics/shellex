# shellex

Translate natural-language intent into shell commands, or explain existing commands — powered by a local LLM via [Ollama](https://ollama.com). No data leaves your machine.

```bash
$ shellex "find all PNG files larger than 5MB in home directory"
> find ~/ -type f -name "*.png" -size +5M
  [Enter] Run  [Tab] Edit  [Esc] Cancel
```

```bash
$ shellex -e "tar czf - /var/log | ssh backup@remote 'cat > /backups/logs.tar.gz'"

Summary: Creates a gzipped tar archive of /var/log and streams it over SSH to a remote server.
Breakdown:
  [1] tar czf - -- create gzipped archive to stdout
  [2] /var/log -- directory to archive
  [3] | -- pipe to next command
  [4] ssh backup@remote 'cat > /backups/logs.tar.gz' -- receive on remote, write to file
```

## shellex-lite

Don't want to install a binary? **[shellex-lite](SHELLEX-LITE.md)** is a collection of 14 shell functions (fish + bash) that call the Ollama API directly via `curl`. Zero dependencies beyond `curl`, `jq`, and a running Ollama instance.

Includes `sx` (command generation), `wtf` (error explainer), `gitm` (AI commit messages), `ai` (general text transformer), `diagnose` (system health check), and more.

## Why

`man` pages are comprehensive but slow. `tldr` is static and can't adapt to your OS, your installed tools, or your specific intent. shellex uses a local LLM to generate commands instantly, offline, tailored to your environment.

## Install

### From source (requires Rust toolchain)

```bash
cargo install --git https://github.com/mihaics/shellex.git
```

### From release binaries

Download the latest release for your platform from [Releases](https://github.com/mihaics/shellex/releases), extract, and place `shellex` in your PATH.

### Prerequisites

[Ollama](https://ollama.com) must be installed and running:

```bash
# Install Ollama
curl -fsSL https://ollama.com/install.sh | sh

# Pull a model (gemma is the default)
ollama pull gemma

# Start the server (if not already running)
ollama serve
```

## Usage

### Generate a command

```bash
shellex "find all log files modified in the last week"
shellex "compress this directory and upload via scp to myserver"
shellex "show disk usage sorted by size"
```

The generated command is shown with an interactive prompt:
- **Enter** - run the command
- **Tab** - edit the command inline before running
- **Esc** - cancel

### Explain a command

```bash
shellex -e "find / -name '*.log' -mtime -7 | xargs grep -l ERROR"
shellex -e "awk '{sum += $5} END {print sum}' access.log"
```

The command is tokenized and each segment is explained.

### Environment-aware generation

```bash
shellex --ctx "install htop"
# Detects your OS, shell, package manager, and installed tools
# Ubuntu: apt install htop
# Fedora: dnf install htop
# macOS:  brew install htop
```

### Scripting mode

```bash
# Execute without confirmation
shellex --yes "list running docker containers"

# Get just the command text (no execution)
shellex --yes --dry-run "count lines in all python files"

# Dangerous commands are blocked in --yes mode (exit code 2)
# Use --force to override
shellex --yes --force "remove all temp files from root"
```

### Other flags

```bash
shellex --model gemma4:e2b "query"         # Use a different model
shellex --verbose "query"                   # Show the full LLM prompt
shellex --config /path/to/config.toml "q"  # Custom config file
```

## Configuration

Config lives at `~/.config/shellex/config.toml` (or `$XDG_CONFIG_HOME/shellex/config.toml`). Created with defaults on first run.

```toml
# Model to use with Ollama
model = "qwen3-coder"

# Ollama server URL
ollama_url = "http://localhost:11434"

# Regex patterns that trigger a safety warning
dangerous_patterns = [
    "rm\\s+(\\S+\\s+)+/",
    "mkfs",
    "dd\\s+.*of=/dev/",
    "chmod\\s+777",
    "wget.*\\|.*sh",
    "curl.*\\|.*sh",
]

# Tools to check for with --ctx
ctx_tools = [
    "git", "docker", "kubectl", "systemctl",
    "npm", "python3", "cargo", "go",
    "apt", "dnf", "pacman", "brew",
    "jq", "ripgrep", "fd", "tmux",
]

# Extra instructions for the LLM
custom_prompt = ""
```

### Recommended models

| Model | Size | Notes |
|-------|------|-------|
| **qwen3-coder** (default) | 18 GB | MoE (3.3B active), fast, best tool-calling |
| **gemma4:e2b** | 3 GB | Compact, fast, good accuracy |
| **gemma4:26b** | 17 GB | Strong reasoning, clean output |

## Safety

shellex never auto-executes commands. Generated commands must be confirmed by the user.

Additionally, a configurable regex blocklist detects dangerous patterns (`rm -rf /`, `mkfs`, `dd of=/dev/`, `curl | sh`, etc.). Matching commands require typing `yes` to proceed — no inline editing allowed.

In `--yes` scripting mode, dangerous commands exit with code 2 unless `--force` is also passed.

The explain mode tokenizer operates on raw bytes and never evaluates shell expressions. `$(rm -rf /)` passes through as a literal string for the LLM to explain.

## Build from source

```bash
git clone https://github.com/mihaics/shellex.git
cd shellex
cargo build --release
# Binary at target/release/shellex
```

### Run tests

```bash
cargo test                        # Unit + integration tests
cargo test --ignored              # Ollama smoke tests (requires running Ollama)
```

## How it works

1. Your intent is sent to a local Ollama model with a system prompt that constrains output to a single command
2. The response is parsed (markdown fences and backticks stripped, first line extracted)
3. The command is checked against a safety blocklist
4. Top-level command existence is verified via `which` (shell builtins are skipped)
5. You confirm, edit, or cancel via the interactive prompt
6. On confirmation, the command is executed via your `$SHELL`

For explain mode, the command is pre-tokenized into segments (pipes, operators, redirects) on the Rust side, then sent to the LLM for annotation — reducing hallucination.

## License

MIT
