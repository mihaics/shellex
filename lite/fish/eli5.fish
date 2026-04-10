function eli5 --description "Explain like I'm 5: eli5 iptables DNAT"
  set -l model (set -q SX_MODEL && echo $SX_MODEL || echo "qwen3-coder")
  if test (count $argv) -eq 0
    echo "Usage: eli5 <concept>"; return 1
  end
  set -l user_prompt (string join " " $argv)
  _ollama $model \
    "Explain this concept in plain, simple English. Use analogies. No jargon. No markdown fences. Keep it under 8 lines." \
    "$user_prompt"
end
