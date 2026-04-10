function tldr --description "Pipe verbose output to get a summary: kubectl describe pod foo | tldr"
  if isatty stdin
    echo "Usage: some_command | tldr"; return 1
  end
  set -l model (set -q SX_MODEL && echo $SX_MODEL || echo "qwen3-coder")
  read -z input
  set -l input (printf '%s' "$input" | head -100 | string collect)
  if test -z "$input"
    echo "No input received."; return 1
  end
  _ollama $model \
    "Summarize this command output in 3-5 bullet points. Focus on what matters — status, errors, key values. No markdown fences. Use • for bullets." \
    "$input"
end
