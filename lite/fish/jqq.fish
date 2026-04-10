function jqq --description "Natural language jq filter: cat data.json | jqq 'get all names where age > 30'"
  set -l model (set -q SX_MODEL && echo $SX_MODEL || echo "qwen3-coder")
  if test (count $argv) -eq 0
    echo "Usage: cat file.json | jqq 'description of what to extract'"; return 1
  end
  set -l input ""
  if not isatty stdin
    read -z input
    set input (printf '%s' "$input" | string collect)
  end
  set -l user_prompt (string join " " $argv)
  set -l sample (printf '%s' "$input" | head -20 | string collect)

  set -l filter (_ollama $model \
    "You are a jq filter generator. Output ONLY the jq filter expression, nothing else. No explanation, no markdown, no backticks." \
    "JSON sample (first 20 lines):
$sample

Query: $user_prompt" | string trim)

  if test -z "$filter"
    echo "No filter generated"; return 1
  end
  set_color yellow; echo "▶ jq '$filter'" >&2; set_color normal
  if test -n "$input"
    printf '%s' "$input" | jq "$filter"
  else
    echo "Filter: $filter"
  end
end
