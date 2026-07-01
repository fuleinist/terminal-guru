# terminal-guru zsh plugin
# Source this in your .zshrc:
#   source /path/to/shell-plugins/zsh.sh

TGURU_BIN="${TGURU_BIN:-tguru}"

# Suggest after every 10th command
__tguru_prompt_counter=0

tguru_precmd() {
    ((__tguru_prompt_counter++))
    if (( __tguru_prompt_counter >= 10 )); then
        __tguru_prompt_counter=0
        # Silently check for suggestions in background
        ($TGURU_BIN suggest --json 2>/dev/null | $TGURU_BIN -q list --unapplied 2>/dev/null) &
    fi
}

# Add to precmd hooks
autoload -Uz add-zsh-hook
add-zsh-hook precmd tguru_precmd

# Convenience aliases
alias tg='tguru suggest'
alias tgs='tguru stats'
alias tgdaemon='tguru daemon'
