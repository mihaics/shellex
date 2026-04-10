function ai --description "LLM agent: ai 'what linux version am I running' or echo text | ai 'summarize'"
  set -l model (set -q SX_MODEL && echo $SX_MODEL || echo "qwen3-coder")
  if test (count $argv) -eq 0
    echo "Usage: ai 'question'  OR  echo text | ai 'instruction'"; return 1
  end
  set -l instruction (string join " " $argv)

  # Pipe mode: transform text
  if not isatty stdin
    set -l input (cat)
    _ollama $model \
      "Apply the user's instruction to the provided text. Output ONLY the result. No explanation, no markdown fences." \
      "Instruction: $instruction

Text:
$input"
    return
  end

  # Agent mode: run commands to answer questions
  # Step 1: ask LLM what command to run
  set -l cmd (_ollama $model \
    "You are a shell command generator. The user asks a question. You MUST output a shell command to answer it. Always prefer running a real command over guessing. Output ONLY the command — no explanation, no markdown, no backticks. Only output NONE if the question is purely conceptual (e.g. 'what is a monad')." \
    "OS: "(uname -s)" Shell: $SHELL
Question: $instruction" | string trim | head -1 \
    | string replace -r '^```\w*' '' | string replace -r '```$' '' \
    | string replace -r '^`' '' | string replace -r '`$' '' | string trim)

  if test -z "$cmd" -o "$cmd" = "NONE"
    _ollama $model "Answer concisely. No markdown fences." "$instruction"
    return
  end

  # Step 2: safety check — blacklist dangerous patterns
  set -l dangerous_patterns \
    'rm\s+(\S+\s+)+/' \
    'mkfs' \
    'dd\s+.*of=/dev/' \
    ':\(\)\{.*\|.*&\}.*;:' \
    'chmod\s+777' \
    '>/dev/sd' \
    'wget.*\|.*sh' \
    'curl.*\|.*sh' \
    'sudo\s+rm' \
    'sudo\s+mkfs' \
    'sudo\s+dd' \
    'reboot' \
    'shutdown' \
    'kill\s+-9\s+1\b' \
    'mv\s+/\S' \
    'systemctl\s+(stop|disable|mask)' \
    '>\s*/etc/' \
    'chmod\s+[0-7]*[0-7]\s+/'

  set -l is_dangerous 0
  for pattern in $dangerous_patterns
    if echo "$cmd" | grep -qE "$pattern"
      set is_dangerous 1
      break
    end
  end

  set_color brblack; echo "→ $cmd" >&2; set_color normal

  if test $is_dangerous -eq 1
    set_color red; echo "⚠ dangerous command — not auto-executing" >&2; set_color normal
    read -P "[Enter=run anyway, Ctrl-C=cancel] " confirm
  end

  set -l output (eval $cmd 2>&1)
  set -l exit_code $status

  if test $exit_code -ne 0
    set_color brblack; echo "  (exit $exit_code)" >&2; set_color normal
  end

  # Step 3: summarize or print directly
  set -l line_count (echo "$output" | wc -l | string trim)
  if test $line_count -le 20
    # Short output — just print it, no need for a second LLM call
    echo "$output"
  else
    # Long output — cap at 100 lines and let LLM summarize
    set -l capped (echo "$output" | head -100)
    _ollama $model \
      "You ran a command to answer the user's question. Give a clear, concise answer based on the output. No markdown fences. If the output is short enough, include the key data directly." \
      "Question: $instruction
Command: $cmd
Exit code: $exit_code
Output (first 100 lines):
$capped"
  end
end
