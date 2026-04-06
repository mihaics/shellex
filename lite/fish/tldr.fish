function tldr --description "Pipe verbose output to get a summary: kubectl describe pod foo | tldr"
  set -l model (set -q SX_MODEL && echo $SX_MODEL || echo "qwen2.5-coder:7b")
  set -l input (cat)
  if test -z "$input"
    echo "Usage: some_command | tldr"; return 1
  end
  _ollama $model \
    "Summarize this command output in 3-5 bullet points. Focus on what matters — status, errors, key values. No markdown fences. Use • for bullets." \
    "$input"
end
