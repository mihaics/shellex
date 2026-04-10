function fixtypo --description "Fix typos in piped text: echo 'teh quikc fox' | fixtypo"
  if isatty stdin
    echo "Usage: echo 'text with typos' | fixtypo"; return 1
  end
  set -l model (set -q SX_MODEL && echo $SX_MODEL || echo "qwen3-coder")
  read -z input
  set -l input (printf '%s' "$input" | head -100 | string collect)
  if test -z "$input"
    echo "No input received."; return 1
  end
  _ollama $model \
    "Fix all typos and grammar errors in the text. Output ONLY the corrected text. Do not explain changes. Preserve the original formatting, line breaks, and meaning." \
    "$input"
end
