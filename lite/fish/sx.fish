function sx
  set -l model (set -q SX_MODEL && echo $SX_MODEL || echo "qwen3-coder")
  set -l url (set -q SX_OLLAMA_URL && echo $SX_OLLAMA_URL || echo "http://localhost:11434")
  set -l sys "You are a shell command generator. Output ONLY the command, no explanation, no markdown, no backticks. One single command or pipeline. OS: "(uname -s)" Shell: $SHELL"
  set -l user_prompt (string join " " $argv)

  set -l json (jq -n --arg m $model --arg s "$sys" --arg p "$user_prompt" \
    '{model:$m, system:$s, prompt:$p, stream:false}')
  set -l cmd (curl -s "$url/api/generate" -d "$json" 2>/dev/null \
    | jq -r '.response // empty' | string trim | head -1 \
    | string replace -r '^```\w*' '' | string replace -r '```$' '' \
    | string replace -r '^`' '' | string replace -r '`$' '' | string trim)

  if test -z "$cmd"
    echo "No command generated"; return 1
  end

  set_color green; echo "▶ $cmd"; set_color normal
  read -P "[Enter=run, Ctrl-C=cancel] " confirm
  eval $cmd
end
