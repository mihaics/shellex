function manq --description "Ask about a man page: manq ffmpeg 'convert mp4 to gif at 10fps'"
  set -l model (set -q SX_MODEL && echo $SX_MODEL || echo "qwen3-coder")
  if test (count $argv) -lt 2
    echo "Usage: manq <command> <question>"; return 1
  end
  set -l cmd $argv[1]
  set -l question (string join " " $argv[2..])
  set -l manpage (man $cmd 2>/dev/null | col -bx | head -300)
  if test -z "$manpage"
    echo "No man page for '$cmd'"; return 1
  end
  _ollama $model \
    "You are reading a man page and answering a specific question about it. Give the exact command with the right flags. Be concise — max 5 lines. No markdown fences." \
    "Man page for $cmd (first 300 lines):
$manpage

Question: $question"
end
