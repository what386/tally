#compdef tally

autoload -U is-at-least

_tally() {
    typeset -A opt_args
    typeset -a _arguments_options
    local ret=1

    if is-at-least 5.2; then
        _arguments_options=(-s -S -C)
    else
        _arguments_options=(-s -C)
    fi

    local context curcontext="$curcontext" state line
    _arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
'-V[Print version]' \
'--version[Print version]' \
":: :_tally_commands" \
"*::: :->tally" \
&& ret=0
    case $state in
    (tally)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:tally-command-$line[1]:"
        case $line[1] in
            (add)
_arguments "${_arguments_options[@]}" : \
'-p+[Priority for the new task]:PRIORITY:(low medium high)' \
'--priority=[Priority for the new task]:PRIORITY:(low medium high)' \
'*-t+[Comma-separated tags to attach]:TAGS:_default' \
'*--tags=[Comma-separated tags to attach]:TAGS:_default' \
'--dry-run[Show what would be added without writing TODO.md]' \
'--auto[Auto-commit updated files after adding]' \
'--json[Output result as JSON]' \
'-h[Print help]' \
'--help[Print help]' \
'*::description -- Task text to add:_default' \
&& ret=0
;;
(done)
_arguments "${_arguments_options[@]}" : \
'-c+[Commit hash to associate with completion]:COMMIT:_default' \
'--commit=[Commit hash to associate with completion]:COMMIT:_default' \
'-v+[Release version to attach at completion time]:VERSION:_default' \
'--version=[Release version to attach at completion time]:VERSION:_default' \
'--dry-run[Show what would be changed without writing TODO.md]' \
'--auto[Auto-commit updated files after completion]' \
'--json[Output result as JSON]' \
'-h[Print help]' \
'--help[Print help]' \
'*::description -- Task text to match:_default' \
&& ret=0
;;
(list)
_arguments "${_arguments_options[@]}" : \
'*-t+[Filter by one or more comma-separated tags]:TAGS:_default' \
'*--tags=[Filter by one or more comma-separated tags]:TAGS:_default' \
'-p+[Filter by priority]:PRIORITY:(low medium high)' \
'--priority=[Filter by priority]:PRIORITY:(low medium high)' \
'-r+[List released tasks from CHANGELOG.md for a specific version]:VERSION:_default' \
'--released=[List released tasks from CHANGELOG.md for a specific version]:VERSION:_default' \
'--done[Show only completed tasks]' \
'--json[Output results as JSON]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(semver)
_arguments "${_arguments_options[@]}" : \
'--dry-run[Show what would be moved without writing files]' \
'--summary[Print a summary of tasks moved for this version]' \
'--auto[Auto-commit updated files after semver move]' \
'--json[Output result as JSON]' \
'-h[Print help]' \
'--help[Print help]' \
':version -- Version to assign (for example\: 1.2.3 or v1.2.3):_default' \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
'-r+[Remove from CHANGELOG.md in a specific version instead of TODO.md]:VERSION:_default' \
'--released=[Remove from CHANGELOG.md in a specific version instead of TODO.md]:VERSION:_default' \
'*-t+[Filter candidate tasks by one or more comma-separated tags before matching]:TAGS:_default' \
'*--tags=[Filter candidate tasks by one or more comma-separated tags before matching]:TAGS:_default' \
'--dry-run[Show what would be removed without writing TODO.md]' \
'--auto[Auto-commit updated files after removal]' \
'--json[Output result as JSON]' \
'-h[Print help]' \
'--help[Print help]' \
'*::description -- Task text to match:_default' \
&& ret=0
;;
(yank)
_arguments "${_arguments_options[@]}" : \
'*-t+[Optional tag filter to narrow released-task matching]:TAGS:_default' \
'*--tags=[Optional tag filter to narrow released-task matching]:TAGS:_default' \
'--dry-run[Show what would be yanked without writing files]' \
'--auto[Auto-commit updated files after yank]' \
'--json[Output result as JSON]' \
'-h[Print help]' \
'--help[Print help]' \
'*::description -- Released task text to match:_default' \
&& ret=0
;;
(scan)
_arguments "${_arguments_options[@]}" : \
'--auto[Auto-accept git-based done matches without prompting]' \
'--dry-run[Show what would change without writing files]' \
'--git[Include git commit scanning]' \
'--todo[Include source TODO scanning]' \
'--done[Include source DONE scanning]' \
'--json[Output result as JSON]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_tally__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:tally-help-command-$line[1]:"
        case $line[1] in
            (add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(done)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(semver)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(yank)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(scan)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
}

(( $+functions[_tally_commands] )) ||
_tally_commands() {
    local commands; commands=(
'add:Add a new task to TODO.md' \
'done:Mark a task as completed using fuzzy description matching' \
'list:List tasks with optional filters' \
'semver:Move completed unversioned tasks into CHANGELOG.md under a version' \
'remove:Remove a task by fuzzy description match from TODO.md or a released entry' \
'yank:Yank a changelog entry back into TODO as completed and unversioned' \
'scan:Scan for task updates from git commits and/or source TODO markers' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'tally commands' commands "$@"
}
(( $+functions[_tally__subcmd__add_commands] )) ||
_tally__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'tally add commands' commands "$@"
}
(( $+functions[_tally__subcmd__done_commands] )) ||
_tally__subcmd__done_commands() {
    local commands; commands=()
    _describe -t commands 'tally done commands' commands "$@"
}
(( $+functions[_tally__subcmd__help_commands] )) ||
_tally__subcmd__help_commands() {
    local commands; commands=(
'add:Add a new task to TODO.md' \
'done:Mark a task as completed using fuzzy description matching' \
'list:List tasks with optional filters' \
'semver:Move completed unversioned tasks into CHANGELOG.md under a version' \
'remove:Remove a task by fuzzy description match from TODO.md or a released entry' \
'yank:Yank a changelog entry back into TODO as completed and unversioned' \
'scan:Scan for task updates from git commits and/or source TODO markers' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'tally help commands' commands "$@"
}
(( $+functions[_tally__subcmd__help__subcmd__add_commands] )) ||
_tally__subcmd__help__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'tally help add commands' commands "$@"
}
(( $+functions[_tally__subcmd__help__subcmd__done_commands] )) ||
_tally__subcmd__help__subcmd__done_commands() {
    local commands; commands=()
    _describe -t commands 'tally help done commands' commands "$@"
}
(( $+functions[_tally__subcmd__help__subcmd__help_commands] )) ||
_tally__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'tally help help commands' commands "$@"
}
(( $+functions[_tally__subcmd__help__subcmd__list_commands] )) ||
_tally__subcmd__help__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'tally help list commands' commands "$@"
}
(( $+functions[_tally__subcmd__help__subcmd__remove_commands] )) ||
_tally__subcmd__help__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'tally help remove commands' commands "$@"
}
(( $+functions[_tally__subcmd__help__subcmd__scan_commands] )) ||
_tally__subcmd__help__subcmd__scan_commands() {
    local commands; commands=()
    _describe -t commands 'tally help scan commands' commands "$@"
}
(( $+functions[_tally__subcmd__help__subcmd__semver_commands] )) ||
_tally__subcmd__help__subcmd__semver_commands() {
    local commands; commands=()
    _describe -t commands 'tally help semver commands' commands "$@"
}
(( $+functions[_tally__subcmd__help__subcmd__yank_commands] )) ||
_tally__subcmd__help__subcmd__yank_commands() {
    local commands; commands=()
    _describe -t commands 'tally help yank commands' commands "$@"
}
(( $+functions[_tally__subcmd__list_commands] )) ||
_tally__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'tally list commands' commands "$@"
}
(( $+functions[_tally__subcmd__remove_commands] )) ||
_tally__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'tally remove commands' commands "$@"
}
(( $+functions[_tally__subcmd__scan_commands] )) ||
_tally__subcmd__scan_commands() {
    local commands; commands=()
    _describe -t commands 'tally scan commands' commands "$@"
}
(( $+functions[_tally__subcmd__semver_commands] )) ||
_tally__subcmd__semver_commands() {
    local commands; commands=()
    _describe -t commands 'tally semver commands' commands "$@"
}
(( $+functions[_tally__subcmd__yank_commands] )) ||
_tally__subcmd__yank_commands() {
    local commands; commands=()
    _describe -t commands 'tally yank commands' commands "$@"
}

if [ "$funcstack[1]" = "_tally" ]; then
    _tally "$@"
else
    compdef _tally tally
fi
