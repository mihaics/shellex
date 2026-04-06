function wtf --description "Pipe error output to get an explanation: make 2>&1 | wtf"
  set -l model (set -q SX_MODEL && echo $SX_MODEL || echo "qwen2.5-coder:7b")
  set -l input (cat)
  if test -z "$input"
    echo "Usage: some_command 2>&1 | wtf"; return 1
  end
  _ollama $model \
    "You are a concise error diagnostician. Explain what went wrong and suggest a fix. Be brief — max 5 lines. No markdown fences." \
    "$input"
end
