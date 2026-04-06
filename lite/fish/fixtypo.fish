function fixtypo --description "Fix typos in piped text: echo 'teh quikc fox' | fixtypo"
  set -l model (set -q SX_MODEL && echo $SX_MODEL || echo "qwen2.5-coder:7b")
  set -l input (cat)
  if test -z "$input"
    echo "Usage: echo 'text with typos' | fixtypo"; return 1
  end
  _ollama $model \
    "Fix all typos and grammar errors in the text. Output ONLY the corrected text. Do not explain changes. Preserve the original formatting, line breaks, and meaning." \
    "$input"
end
