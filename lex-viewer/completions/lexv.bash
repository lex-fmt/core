_lexv() {
    local i cur prev opts cmd
    COMPREPLY=()
    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
        cur="$2"
    else
        cur="${COMP_WORDS[COMP_CWORD]}"
    fi
    prev="$3"
    cmd=""
    opts=""

    for i in "${COMP_WORDS[@]:0:COMP_CWORD}"
    do
        case "${cmd},${i}" in
            ",$1")
                cmd="lexv"
                ;;
            *)
                ;;
        esac
    done

    case "${cmd}" in
        lexv)
            opts="-h -V --help --version"
            if [[ ${cur} == -* ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    # Complete file paths for the path argument
                    COMPREPLY=( $(compgen -f -- "${cur}") )
                    ;;
            esac
            return 0
            ;;
    esac
}

if [[ "${BASH_VERSINFO[0]}" -eq 4 && "${BASH_VERSINFO[1]}" -ge 4 || "${BASH_VERSINFO[0]}" -gt 4 ]]; then
    complete -F _lexv -o nosort -o bashdefault -o default -o filenames lexv
else
    complete -F _lexv -o bashdefault -o default -o filenames lexv
fi
