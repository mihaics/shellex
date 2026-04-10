function wtf --description "Pipe error output to get an explanation: make 2>&1 | wtf"
  if isatty stdin
    echo "Usage: some_command 2>&1 | wtf"; return 1
  end
  set -l model (set -q SX_MODEL && echo $SX_MODEL || echo "qwen3-coder")
  set -l input (cat | head -100)
  if test -z "$input"
    echo "No input received."; return 1
  end
  _ollama $model \
    "You are a concise error diagnostician. Explain what went wrong and suggest a fix. Be brief — max 5 lines. No markdown fences." \
    "$input"
end
