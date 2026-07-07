# shellex shell integration for fish
# Source from config.fish:  source /path/to/shellex/shell/shellex.fish
# Type a natural-language description at the prompt, press Alt+X, and the
# line is replaced with the generated command — edit it, run it, it lands
# in your history like any other command.
# Rebind by calling:  bind \e<key> __shellex_transform

if status is-interactive
    function __shellex_transform
        set -l intent (commandline)
        if not string match -qr '\S' -- $intent
            return
        end
        set -l cmd (shellex --yes --dry-run -- $intent)
        if test -n "$cmd"
            commandline -r -- $cmd
        end
        commandline -f repaint
    end

    bind \ex __shellex_transform
end
