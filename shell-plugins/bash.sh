# terminal-guru bash plugin
# Source this in your .bashrc:
#   source /path/to/shell-plugins/bash.sh

TGURU_BIN="${TGURU_BIN:-tguru}"

__tguru_prompt_counter=0

__tguru_prompt() {
    ((__tguru_prompt_counter++))
    if (( __tguru_prompt_counter >= 10 )); then
        __tguru_prompt_counter=0
        ($TGURU_BIN suggest --json 2>/dev/null | $TGURU_BIN list --unapplied 2>/dev/null) &
    fi
}

# Hook into PROMPT_COMMAND
if [[ -z "$PROMPT_COMMAND" ]]; then
    PROMPT_COMMAND="__tguru_prompt"
else
    PROMPT_COMMAND="__tguru_prompt;${PROMPT_COMMAND}"
fi

# Convenience aliases
alias tg='tguru suggest'
alias tgs='tguru stats'
alias tgdaemon='tguru daemon'
