# shellex-lite

14 shell functions that bring LLM superpowers to your terminal — powered by a local [Ollama](https://ollama.com) instance. No binary to install, just shell functions that call the Ollama API via `curl` and `jq`.

Think of it as the lightweight, zero-dependency companion to [shellex](README.md).

## Prerequisites

- [Ollama](https://ollama.com) running locally
- `curl` and `jq` installed
- A model pulled: `ollama pull qwen3-coder`

## Quick start

### Fish

Copy the function files from [`lite/fish/`](lite/fish/) into `~/.config/fish/functions/`. They auto-load — no sourcing needed.

```bash
cp lite/fish/*.fish ~/.config/fish/functions/
```

### Bash

Source [`lite/bash/shellex-lite.bash`](lite/bash/shellex-lite.bash) from your `~/.bashrc`:

```bash
echo 'source /path/to/shellex/lite/bash/shellex-lite.bash' >> ~/.bashrc
source ~/.bashrc
```

## Configuration

All functions respect these environment variables:

| Variable | Default | Description |
|---|---|---|
| `SX_MODEL` | `qwen3-coder` | Ollama model to use |
| `OLLAMA_URL` | `http://localhost:11434` | Ollama API endpoint |

```bash
# Use a different model
export SX_MODEL=gemma

# Use a remote Ollama instance
export OLLAMA_URL=http://gpu-server:11434
```

## Functions

### sx — Natural language to shell command

```bash
sx list files sorted by size
sx find all TODO comments in python files
sx compress this directory and scp to myserver
```

Shows the generated command in green, then waits for confirmation before running.

### wtf — Explain errors

```bash
make 2>&1 | wtf
cargo build 2>&1 | wtf
docker compose up 2>&1 | wtf
```

Pipe any error output and get a concise explanation + suggested fix.

### tldr — Summarize verbose output

```bash
kubectl describe pod my-pod | tldr
systemctl status nginx | tldr
git log --stat -20 | tldr
```

Distills long command output into 3-5 bullet points.

### ai — LLM agent + text transformer

```bash
# Agent mode (no pipe) — runs commands to answer your questions
ai "show me the content of Cargo.toml"
ai "which version of linux am i running"
ai "how much free disk space do i have"
ai "what is the largest file in this directory"

# Pipe mode — transforms text
echo "hello world" | ai "translate to Romanian"
cat README.md | ai "summarize in 3 bullets"
git diff | ai "explain what changed"
```

Two modes in one function. Without a pipe, it acts as a **mini agent**: asks the LLM what command to run, executes it, then summarizes the result. With a pipe, it transforms text. The executed command is shown in gray on stderr.

### rx — Natural language to regex

```bash
rx "email addresses"
rx "IPv4 addresses"
rx "dates in YYYY-MM-DD format"

# Use it inline
grep -E $(rx "email addresses") contacts.txt
```

Outputs a single ERE-compatible regex pattern.

### jqq — Natural language jq queries

```bash
cat data.json | jqq "get all names where age > 30"
curl -s api.example.com/users | jqq "count by country"
```

Reads a JSON sample, generates the jq filter, shows it, and runs it.

### gitm — AI commit messages

```bash
git add -p
gitm
```

Reads your staged diff and generates a conventional commit message. Press Enter to commit, `e` to edit in your `$EDITOR` first.

### how — Quick terminal how-to

```bash
how tar extract gz
how resize an image to 50% with imagemagick
how find and kill a process by name
```

1-5 line answers with the exact commands you need. OS and shell aware.

### eli5 — Explain like I'm 5

```bash
eli5 TCP three-way handshake
eli5 iptables DNAT
eli5 "what is a monad"
```

Plain-English explanations with analogies, no jargon.

### manq — Ask a man page

```bash
manq ffmpeg "convert mp4 to gif at 10fps"
manq tar "extract specific files from archive"
manq rsync "sync but exclude node_modules"
```

Feeds the actual man page to the LLM so it gives you the exact flags.

### fixtypo — Fix typos and grammar

```bash
echo "teh quikc brwon fox" | fixtypo
cat draft.txt | fixtypo > clean.txt
```

Corrects typos and grammar while preserving meaning and formatting.

### rename-files — Suggest file renames

```bash
ls *.jpg | rename-files "date-based naming"
ls *.mp3 | rename-files "artist - title"
```

Generates `mv` commands, shows them, and asks for confirmation before running.

### diagnose — System health check

```bash
diagnose
```

Collects load, memory, disk, top CPU/MEM processes, and recent kernel messages, then reports anything that looks wrong.

### portwtf — Explain what's on a port

```bash
portwtf 8080
portwtf 3000
portwtf 5432
```

Shows what's listening and explains the process.

## How it works

All functions share a common pattern:

1. Build a JSON payload with `jq -n` (safe escaping of user input)
2. POST to `http://localhost:11434/api/generate` with a focused system prompt
3. Parse the response with `jq -r '.response'`
4. Strip LLM artifacts (markdown fences, backticks)

The key difference from `ollama run`: the API supports a proper `system` prompt field, which dramatically improves output quality.

Fish functions use a shared `_ollama` helper. Bash functions are self-contained.

## Full function reference

See the source files for the complete implementation:

- **Fish**: [`lite/fish/`](lite/fish/) — one file per function, auto-loaded
- **Bash**: [`lite/bash/shellex-lite.bash`](lite/bash/shellex-lite.bash) — all functions in one file
