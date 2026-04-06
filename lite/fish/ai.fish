function ai --description "General pipe transformer: echo text | ai 'translate to Romanian'"
  set -l model (set -q SX_MODEL && echo $SX_MODEL || echo "qwen2.5-coder:7b")
  if test (count $argv) -eq 0
    echo "Usage: echo text | ai 'instruction'  OR  ai 'question'"; return 1
  end
  set -l instruction (string join " " $argv)
  set -l input ""
  if not isatty stdin
    set input (cat)
  end
  if test -n "$input"
    _ollama $model \
      "Apply the user's instruction to the provided text. Output ONLY the result. No explanation, no markdown fences." \
      "Instruction: $instruction

Text:
$input"
  else
    _ollama $model \
      "Answer concisely. No markdown fences." \
      "$instruction"
  end
end
