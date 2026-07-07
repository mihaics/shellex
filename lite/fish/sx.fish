function sx
  set -l model (set -q SX_MODEL && echo $SX_MODEL || echo "gemma4:12b")
  set -l url (set -q OLLAMA_URL && echo $OLLAMA_URL || echo "http://localhost:11434")
  set -l sys "You are a shell command generator. Output ONLY the command, no explanation, no markdown, no backticks. One single command or pipeline. OS: "(uname -s)" Shell: $SHELL"
  set -l user_prompt (string join " " $argv)

  set -l json (jq -n --arg m $model --arg s "$sys" --arg p "$user_prompt" \
    '{model:$m, messages:[{role:"system",content:$s},{role:"user",content:$p}], stream:false}')
  set -l cmd (curl -s --max-time 300 "$url/api/chat" -d "$json" 2>/dev/null \
    | jq -r '.message.content // empty' | string trim | head -1 \
    | string replace -r '^```\w*' '' | string replace -r '```$' '' \
    | string replace -r '^`' '' | string replace -r '`$' '' | string trim)

  if test -z "$cmd"
    echo "No command generated"; return 1
  end

  set_color green; echo "▶ $cmd"; set_color normal
  read -P "[Enter=run, Ctrl-C=cancel] " confirm
  eval $cmd
end
