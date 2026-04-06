function gitm --description "Auto-generate commit message from staged changes"
  set -l model (set -q SX_MODEL && echo $SX_MODEL || echo "qwen2.5-coder:7b")
  set -l diff (git diff --cached --stat 2>/dev/null)
  if test -z "$diff"
    echo "Nothing staged. Use 'git add' first."; return 1
  end
  set -l full_diff (git diff --cached 2>/dev/null | head -200)

  set -l msg (_ollama $model \
    "You are a git commit message generator. Write a concise conventional commit message (type: description). One line, max 72 chars. No backticks, no quotes, no markdown. Types: feat, fix, refactor, docs, style, test, chore, ci." \
    "Staged changes:
$full_diff" | string trim | head -1)

  if test -z "$msg"
    echo "No message generated"; return 1
  end
  set_color green; echo "▶ $msg"; set_color normal
  read -P "[Enter=commit, e=edit, Ctrl-C=cancel] " choice
  switch $choice
    case e E
      git commit -e -m "$msg"
    case '*'
      git commit -m "$msg"
  end
end
