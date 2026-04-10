function portwtf --description "Explain what's on a port: portwtf 8080"
  set -l model (set -q SX_MODEL && echo $SX_MODEL || echo "qwen3-coder")
  if test (count $argv) -eq 0
    echo "Usage: portwtf <port>"; return 1
  end
  set -l port $argv[1]
  set -l info (ss -tlnp 2>/dev/null | grep ":$port " ; lsof -i :$port 2>/dev/null | head -10)
  if test -z "$info"
    echo "Nothing listening on port $port"; return 0
  end
  echo "$info"
  echo "---"
  _ollama $model \
    "You see output from ss/lsof about a network port. Briefly explain: what process is listening, what it likely is, and whether it looks normal. Max 4 lines. No markdown fences." \
    "Port $port:
$info"
end
