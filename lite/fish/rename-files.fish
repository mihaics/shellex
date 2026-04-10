function rename-files --description "Suggest file renames: ls *.jpg | rename-files 'date-based naming'"
  set -l model (set -q SX_MODEL && echo $SX_MODEL || echo "qwen3-coder")
  if test (count $argv) -eq 0
    echo "Usage: ls files | rename-files 'naming convention'"; return 1
  end
  set -l input (cat)
  if test -z "$input"
    echo "Pipe file list via stdin"; return 1
  end
  set -l convention (string join " " $argv)
  set -l cmds (_ollama $model \
    "Generate mv commands to rename these files according to the convention. Output ONLY the mv commands, one per line. No explanation, no markdown." \
    "Files:
$input

Convention: $convention")

  echo "$cmds"
  echo ""
  set_color yellow; echo "Run these commands? [Enter=yes, Ctrl-C=no]" >&2; set_color normal
  read confirm
  eval "$cmds"
end
