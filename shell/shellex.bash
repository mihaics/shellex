# shellex shell integration for bash
# Source from ~/.bashrc:  source /path/to/shellex/shell/shellex.bash
# Type a natural-language description at the prompt, press Alt+X, and the
# line is replaced with the generated command — edit it, run it, it lands
# in your history like any other command.
# Rebind by changing the bind line at the bottom.

case $- in *i*) ;; *) return 0 2>/dev/null || exit 0 ;; esac

__shellex_transform() {
  [ -z "${READLINE_LINE//[[:space:]]/}" ] && return
  local cmd err
  # Capture stderr: the in-flight indicator and warnings would otherwise
  # garble the readline display mid-edit.
  err=$(mktemp) || return
  cmd=$(shellex --yes --dry-run -- "$READLINE_LINE" 2>"$err")
  if [ -n "$cmd" ]; then
    READLINE_LINE="$cmd"
    READLINE_POINT=${#cmd}
    # Surface any warning (e.g. dangerous pattern) above the prompt.
    if [ -s "$err" ]; then printf '\n' >&2; cat "$err" >&2; fi
  else
    printf '\n' >&2; cat "$err" >&2
  fi
  rm -f "$err"
}

bind -x '"\ex": __shellex_transform'
