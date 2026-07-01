# terminal-guru fish plugin
# Source this in your config.fish:
#   source /path/to/shell-plugins/fish.fish

set -q TGURU_BIN; or set -g TGURU_BIN tguru

function __tguru_prompt --on-event fish_prompt
    set -g __tguru_counter (math $__tguru_counter + 1)
    if test $__tguru_counter -ge 10
        set -g __tguru_counter 0
        $TGURU_BIN suggest --json 2>/dev/null | $TGURU_BIN list --unapplied 2>/dev/null &
    end
end

# Convenience aliases
alias tg='tguru suggest'
alias tgs='tguru stats'
alias tgdaemon='tguru daemon'
