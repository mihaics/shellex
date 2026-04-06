function rx --description "Natural language to regex: rx 'email addresses'"
  set -l model (set -q SX_MODEL && echo $SX_MODEL || echo "qwen2.5-coder:7b")
  if test (count $argv) -eq 0
    echo "Usage: rx 'description of pattern'"; return 1
  end
  set -l user_prompt (string join " " $argv)
  _ollama $model \
    "You are a regex generator. Output ONLY the regex pattern, nothing else. No explanation, no markdown, no backticks. One single regex. Use extended regex syntax (ERE) compatible with grep -E." \
    "$user_prompt"
end
