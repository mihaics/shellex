function how --description "Quick how-to: how tar extract gz"
  set -l model (set -q SX_MODEL && echo $SX_MODEL || echo "qwen2.5-coder:7b")
  if test (count $argv) -eq 0
    echo "Usage: how <question>"; return 1
  end
  set -l user_prompt (string join " " $argv)
  _ollama $model \
    "You are a concise terminal assistant. Answer in 1-5 lines. Show the command(s) needed. No markdown fences. If multiple approaches exist, show the simplest one. OS: "(uname -s)" Shell: $SHELL" \
    "$user_prompt"
end
