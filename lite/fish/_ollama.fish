function _ollama
  # Usage: _ollama <model> <system_prompt> <user_prompt>
  set -l model $argv[1]
  set -l sys $argv[2]
  set -l prompt $argv[3]
  set -l url (set -q OLLAMA_URL && echo $OLLAMA_URL || echo "http://localhost:11434")
  printf '\e[90m⏳\e[0m' >&2
  jq -n --arg m $model --arg s "$sys" --arg p "$prompt" \
    '{model:$m, system:$s, prompt:$p, stream:false}' \
    | curl -s --max-time 60 "$url/api/generate" -d @- 2>/dev/null \
    | jq -r '.response // empty'
  printf '\r\e[K' >&2
end
