#!/usr/bin/env bash
# shellex-lite: LLM-powered shell functions via local Ollama
# Source this file from your ~/.bashrc:
#   source /path/to/shellex-lite.bash
#
# Config via environment variables:
#   SX_MODEL     - Ollama model (default: qwen3-coder)
#   OLLAMA_URL   - Ollama API endpoint (default: http://localhost:11434)
#
# Requires: curl, jq, ollama running locally

_ollama() {
  local model="$1" sys="$2" prompt="$3"
  local url="${OLLAMA_URL:-http://localhost:11434}"
  echo -ne "\033[90m⏳\033[0m" >&2
  local result
  result=$(jq -n --arg m "$model" --arg s "$sys" --arg p "$prompt" \
    '{model:$m, system:$s, prompt:$p, stream:false}' \
    | curl -s --max-time 60 "$url/api/generate" -d @- 2>/dev/null \
    | jq -r '.response // empty')
  echo -ne "\r\033[K" >&2
  echo "$result"
}

# sx - Natural language to shell command
sx() {
  local model="${SX_MODEL:-qwen3-coder}"
  local sys="You are a shell command generator. Output ONLY the command, no explanation, no markdown, no backticks. One single command or pipeline. OS: $(uname -s) Shell: $SHELL"
  local cmd
  cmd=$(_ollama "$model" "$sys" "$*" | head -1 \
    | sed 's/^```[a-z]*//;s/```$//;s/^`//;s/`$//' | xargs)
  if [ -z "$cmd" ]; then echo "No command generated"; return 1; fi
  echo -e "\033[32m▶ $cmd\033[0m"
  read -p "[Enter=run, Ctrl-C=cancel] "
  eval "$cmd"
}

# wtf - Explain errors
wtf() {
  if [ -t 0 ]; then echo "Usage: some_command 2>&1 | wtf"; return 1; fi
  local model="${SX_MODEL:-qwen3-coder}"
  local input; input=$(cat | head -100)
  if [ -z "$input" ]; then echo "No input received."; return 1; fi
  _ollama "$model" \
    "You are a concise error diagnostician. Explain what went wrong and suggest a fix. Be brief — max 5 lines. No markdown fences." \
    "$input"
}

# tldr - Summarize verbose output
tldr() {
  if [ -t 0 ]; then echo "Usage: some_command | tldr"; return 1; fi
  local model="${SX_MODEL:-qwen3-coder}"
  local input; input=$(cat | head -100)
  if [ -z "$input" ]; then echo "No input received."; return 1; fi
  _ollama "$model" \
    "Summarize this command output in 3-5 bullet points. Focus on what matters — status, errors, key values. No markdown fences. Use • for bullets." \
    "$input"
}

# ai - LLM agent: answers questions by running commands, or transforms piped text
ai() {
  local model="${SX_MODEL:-qwen3-coder}"
  if [ $# -eq 0 ]; then echo "Usage: ai 'question'  OR  echo text | ai 'instruction'"; return 1; fi
  local instruction="$*"

  # Pipe mode: transform text
  if [ ! -t 0 ]; then
    local input
    input=$(cat)
    _ollama "$model" \
      "Apply the user's instruction to the provided text. Output ONLY the result. No explanation, no markdown fences." \
      "Instruction: $instruction

Text:
$input"
    return
  fi

  # Agent mode: run commands to answer questions
  local cmd
  cmd=$(_ollama "$model" \
    "You are a shell command generator. The user asks a question. You MUST output a shell command to answer it. Always prefer running a real command over guessing. Output ONLY the command — no explanation, no markdown, no backticks. Only output NONE if the question is purely conceptual." \
    "OS: $(uname -s) Shell: $SHELL
Question: $instruction" | head -1 \
    | sed 's/^```[a-z]*//;s/```$//;s/^`//;s/`$//' | xargs)

  if [ -z "$cmd" ] || [ "$cmd" = "NONE" ]; then
    _ollama "$model" "Answer concisely. No markdown fences." "$instruction"
    return
  fi

  # Safety check — blacklist dangerous patterns
  local danger_patterns='rm\s+(\S+\s+)+/|mkfs|dd\s+.*of=/dev/|:\(\)\{.*\|.*&\}.*;:|chmod\s+777|>/dev/sd|wget.*\|.*sh|curl.*\|.*sh|sudo\s+rm|sudo\s+mkfs|sudo\s+dd|reboot|shutdown|kill\s+-9\s+1\b|mv\s+/\S|systemctl\s+(stop|disable|mask)|>\s*/etc/|chmod\s+[0-7]*[0-7]\s+/'
  echo -e "\033[90m→ $cmd\033[0m" >&2
  if echo "$cmd" | grep -qE "$danger_patterns"; then
    echo -e "\033[31m⚠ dangerous command — not auto-executing\033[0m" >&2
    read -p "[Enter=run anyway, Ctrl-C=cancel] "
  fi

  local output exit_code
  output=$(eval "$cmd" 2>&1)
  exit_code=$?
  [ $exit_code -ne 0 ] && echo -e "\033[90m  (exit $exit_code)\033[0m" >&2

  # Short output — print directly; long output — cap and summarize
  local line_count; line_count=$(echo "$output" | wc -l)
  if [ "$line_count" -le 20 ]; then
    echo "$output"
  else
    local capped; capped=$(echo "$output" | head -100)
    _ollama "$model" \
      "You ran a command to answer the user's question. Give a clear, concise answer based on the output. No markdown fences. If the output is short enough, include the key data directly." \
      "Question: $instruction
Command: $cmd
Exit code: $exit_code
Output (first 100 lines):
$capped"
  fi
}

# rx - Natural language to regex
rx() {
  local model="${SX_MODEL:-qwen3-coder}"
  if [ $# -eq 0 ]; then echo "Usage: rx 'description of pattern'"; return 1; fi
  _ollama "$model" \
    "You are a regex generator. Output ONLY the regex pattern, nothing else. No explanation, no markdown, no backticks. One single regex. Use extended regex syntax (ERE) compatible with grep -E." \
    "$*"
}

# jqq - Natural language jq queries
jqq() {
  local model="${SX_MODEL:-qwen3-coder}"
  if [ $# -eq 0 ]; then echo "Usage: cat file.json | jqq 'description of what to extract'"; return 1; fi
  local input=""
  if [ ! -t 0 ]; then input=$(cat); fi
  local sample
  sample=$(echo "$input" | head -20)
  local filter
  filter=$(_ollama "$model" \
    "You are a jq filter generator. Output ONLY the jq filter expression, nothing else. No explanation, no markdown, no backticks." \
    "JSON sample (first 20 lines):
$sample

Query: $*" | xargs)
  if [ -z "$filter" ]; then echo "No filter generated"; return 1; fi
  echo -e "\033[33m▶ jq '$filter'\033[0m" >&2
  if [ -n "$input" ]; then
    echo "$input" | jq "$filter"
  else
    echo "Filter: $filter"
  fi
}

# gitm - AI commit messages
gitm() {
  local model="${SX_MODEL:-qwen3-coder}"
  local diff
  diff=$(git diff --cached --stat 2>/dev/null)
  if [ -z "$diff" ]; then echo "Nothing staged. Use 'git add' first."; return 1; fi
  local full_diff
  full_diff=$(git diff --cached 2>/dev/null | head -200)
  local msg
  msg=$(_ollama "$model" \
    "You are a git commit message generator. Write a concise conventional commit message (type: description). One line, max 72 chars. No backticks, no quotes, no markdown. Types: feat, fix, refactor, docs, style, test, chore, ci." \
    "Staged changes:
$full_diff" | head -1 | xargs)
  if [ -z "$msg" ]; then echo "No message generated"; return 1; fi
  echo -e "\033[32m▶ $msg\033[0m"
  read -p "[Enter=commit, e=edit, Ctrl-C=cancel] " choice
  case "$choice" in
    e|E) git commit -e -m "$msg" ;;
    *) git commit -m "$msg" ;;
  esac
}

# how - Quick terminal how-to
how() {
  local model="${SX_MODEL:-qwen3-coder}"
  if [ $# -eq 0 ]; then echo "Usage: how <question>"; return 1; fi
  _ollama "$model" \
    "You are a concise terminal assistant. Answer in 1-5 lines. Show the command(s) needed. No markdown fences. If multiple approaches exist, show the simplest one. OS: $(uname -s) Shell: $SHELL" \
    "$*"
}

# eli5 - Explain like I'm 5
eli5() {
  local model="${SX_MODEL:-qwen3-coder}"
  if [ $# -eq 0 ]; then echo "Usage: eli5 <concept>"; return 1; fi
  _ollama "$model" \
    "Explain this concept in plain, simple English. Use analogies. No jargon. No markdown fences. Keep it under 8 lines." \
    "$*"
}

# manq - Ask a man page
manq() {
  local model="${SX_MODEL:-qwen3-coder}"
  if [ $# -lt 2 ]; then echo "Usage: manq <command> <question>"; return 1; fi
  local cmd="$1"; shift
  local question="$*"
  local manpage
  manpage=$(man "$cmd" 2>/dev/null | col -bx | head -300)
  if [ -z "$manpage" ]; then echo "No man page for '$cmd'"; return 1; fi
  _ollama "$model" \
    "You are reading a man page and answering a specific question about it. Give the exact command with the right flags. Be concise — max 5 lines. No markdown fences." \
    "Man page for $cmd (first 300 lines):
$manpage

Question: $question"
}

# fixtypo - Fix typos and grammar
fixtypo() {
  if [ -t 0 ]; then echo "Usage: echo 'text with typos' | fixtypo"; return 1; fi
  local model="${SX_MODEL:-qwen3-coder}"
  local input; input=$(cat | head -100)
  if [ -z "$input" ]; then echo "No input received."; return 1; fi
  _ollama "$model" \
    "Fix all typos and grammar errors in the text. Output ONLY the corrected text. Do not explain changes. Preserve the original formatting, line breaks, and meaning." \
    "$input"
}

# rename-files - Suggest file renames
rename-files() {
  local model="${SX_MODEL:-qwen3-coder}"
  if [ $# -eq 0 ]; then echo "Usage: ls files | rename-files 'naming convention'"; return 1; fi
  local input
  input=$(cat)
  if [ -z "$input" ]; then echo "Pipe file list via stdin"; return 1; fi
  local cmds
  cmds=$(_ollama "$model" \
    "Generate mv commands to rename these files according to the convention. Output ONLY the mv commands, one per line. No explanation, no markdown." \
    "Files:
$input

Convention: $*")
  echo "$cmds"
  echo ""
  echo -e "\033[33mRun these commands? [Enter=yes, Ctrl-C=no]\033[0m" >&2
  read
  eval "$cmds"
}

# diagnose - System health check
diagnose() {
  local model="${SX_MODEL:-qwen3-coder}"
  echo "Collecting system state..." >&2
  local info=""
  info+="=== LOAD ===\n$(uptime 2>/dev/null)\n\n"
  info+="=== MEMORY ===\n$(free -h 2>/dev/null)\n\n"
  info+="=== DISK ===\n$(df -h / /home 2>/dev/null)\n\n"
  info+="=== TOP CPU ===\n$(ps aux --sort=-%cpu 2>/dev/null | head -6)\n\n"
  info+="=== TOP MEM ===\n$(ps aux --sort=-%mem 2>/dev/null | head -6)\n\n"
  info+="=== DMESG (last 20) ===\n$(dmesg --time-format reltime 2>/dev/null | tail -20)"
  _ollama "$model" \
    "You are a Linux sysadmin. Analyze this system snapshot. Report ONLY things that look wrong or concerning — high load, low memory, disk nearly full, suspicious processes, kernel errors. If everything looks fine, say so briefly. No markdown fences. Use • for bullets." \
    "$(echo -e "$info")"
}

# portwtf - Explain what's on a port
portwtf() {
  local model="${SX_MODEL:-qwen3-coder}"
  if [ $# -eq 0 ]; then echo "Usage: portwtf <port>"; return 1; fi
  local port="$1"
  local info
  info=$(ss -tlnp 2>/dev/null | grep ":$port " ; lsof -i :"$port" 2>/dev/null | head -10)
  if [ -z "$info" ]; then echo "Nothing listening on port $port"; return 0; fi
  echo "$info"
  echo "---"
  _ollama "$model" \
    "You see output from ss/lsof about a network port. Briefly explain: what process is listening, what it likely is, and whether it looks normal. Max 4 lines. No markdown fences." \
    "Port $port:
$info"
}
