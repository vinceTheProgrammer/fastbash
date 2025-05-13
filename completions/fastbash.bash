# fastbash tab completion for bash

_fastbash_completions() {
    local cur prev commands scripts_dir
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    commands="ls rm help create edit"

    scripts_dir="$HOME/.fastbash/scripts"
    scripts=$(ls "$scripts_dir" 2>/dev/null)

    if [[ $COMP_CWORD -eq 1 ]]; then
        COMPREPLY=( $(compgen -W "$commands $scripts" -- "$cur") )
    elif [[ $COMP_CWORD -eq 2 && $prev == "rm" || $COMP_CWORD -eq 2 && $prev == "edit" ]]; then
        COMPREPLY=( $(compgen -W "$scripts" -- "$cur") )
    fi
}

complete -F _fastbash_completions fastbash
