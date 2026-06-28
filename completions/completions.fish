# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_tally_global_optspecs
	string join \n h/help V/version
end

function __fish_tally_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_tally_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_tally_using_subcommand
	set -l cmd (__fish_tally_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c tally -n "__fish_tally_needs_command" -s h -l help -d 'Print help'
complete -c tally -n "__fish_tally_needs_command" -s V -l version -d 'Print version'
complete -c tally -n "__fish_tally_needs_command" -f -a "add" -d 'Add a new task to TODO.md'
complete -c tally -n "__fish_tally_needs_command" -f -a "done" -d 'Mark a task as completed using fuzzy description matching'
complete -c tally -n "__fish_tally_needs_command" -f -a "list" -d 'List tasks with optional filters'
complete -c tally -n "__fish_tally_needs_command" -f -a "semver" -d 'Move completed unversioned tasks into CHANGELOG.md under a version'
complete -c tally -n "__fish_tally_needs_command" -f -a "remove" -d 'Remove a task by fuzzy description match from TODO.md or a released entry'
complete -c tally -n "__fish_tally_needs_command" -f -a "yank" -d 'Yank a changelog entry back into TODO as completed and unversioned'
complete -c tally -n "__fish_tally_needs_command" -f -a "scan" -d 'Scan for task updates from git commits and/or source TODO markers'
complete -c tally -n "__fish_tally_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c tally -n "__fish_tally_using_subcommand add" -s p -l priority -d 'Priority for the new task' -r -f -a "low\t''
medium\t''
high\t''"
complete -c tally -n "__fish_tally_using_subcommand add" -s t -l tags -d 'Comma-separated tags to attach' -r
complete -c tally -n "__fish_tally_using_subcommand add" -l dry-run -d 'Show what would be added without writing TODO.md'
complete -c tally -n "__fish_tally_using_subcommand add" -l auto -d 'Auto-commit updated files after adding'
complete -c tally -n "__fish_tally_using_subcommand add" -l json -d 'Output result as JSON'
complete -c tally -n "__fish_tally_using_subcommand add" -s h -l help -d 'Print help'
complete -c tally -n "__fish_tally_using_subcommand done" -s c -l commit -d 'Commit hash to associate with completion' -r
complete -c tally -n "__fish_tally_using_subcommand done" -s v -l version -d 'Release version to attach at completion time' -r
complete -c tally -n "__fish_tally_using_subcommand done" -l dry-run -d 'Show what would be changed without writing TODO.md'
complete -c tally -n "__fish_tally_using_subcommand done" -l auto -d 'Auto-commit updated files after completion'
complete -c tally -n "__fish_tally_using_subcommand done" -l json -d 'Output result as JSON'
complete -c tally -n "__fish_tally_using_subcommand done" -s h -l help -d 'Print help'
complete -c tally -n "__fish_tally_using_subcommand list" -s t -l tags -d 'Filter by one or more comma-separated tags' -r
complete -c tally -n "__fish_tally_using_subcommand list" -s p -l priority -d 'Filter by priority' -r -f -a "low\t''
medium\t''
high\t''"
complete -c tally -n "__fish_tally_using_subcommand list" -s r -l released -d 'List released tasks from CHANGELOG.md for a specific version' -r
complete -c tally -n "__fish_tally_using_subcommand list" -l done -d 'Show only completed tasks'
complete -c tally -n "__fish_tally_using_subcommand list" -l json -d 'Output results as JSON'
complete -c tally -n "__fish_tally_using_subcommand list" -s h -l help -d 'Print help'
complete -c tally -n "__fish_tally_using_subcommand semver" -l dry-run -d 'Show what would be moved without writing files'
complete -c tally -n "__fish_tally_using_subcommand semver" -l summary -d 'Print a summary of tasks moved for this version'
complete -c tally -n "__fish_tally_using_subcommand semver" -l auto -d 'Auto-commit updated files after semver move'
complete -c tally -n "__fish_tally_using_subcommand semver" -l json -d 'Output result as JSON'
complete -c tally -n "__fish_tally_using_subcommand semver" -s h -l help -d 'Print help'
complete -c tally -n "__fish_tally_using_subcommand remove" -s r -l released -d 'Remove from CHANGELOG.md in a specific version instead of TODO.md' -r
complete -c tally -n "__fish_tally_using_subcommand remove" -s t -l tags -d 'Filter candidate tasks by one or more comma-separated tags before matching' -r
complete -c tally -n "__fish_tally_using_subcommand remove" -l dry-run -d 'Show what would be removed without writing TODO.md'
complete -c tally -n "__fish_tally_using_subcommand remove" -l auto -d 'Auto-commit updated files after removal'
complete -c tally -n "__fish_tally_using_subcommand remove" -l json -d 'Output result as JSON'
complete -c tally -n "__fish_tally_using_subcommand remove" -s h -l help -d 'Print help'
complete -c tally -n "__fish_tally_using_subcommand yank" -s t -l tags -d 'Optional tag filter to narrow released-task matching' -r
complete -c tally -n "__fish_tally_using_subcommand yank" -l dry-run -d 'Show what would be yanked without writing files'
complete -c tally -n "__fish_tally_using_subcommand yank" -l auto -d 'Auto-commit updated files after yank'
complete -c tally -n "__fish_tally_using_subcommand yank" -l json -d 'Output result as JSON'
complete -c tally -n "__fish_tally_using_subcommand yank" -s h -l help -d 'Print help'
complete -c tally -n "__fish_tally_using_subcommand scan" -l auto -d 'Auto-accept git-based done matches without prompting'
complete -c tally -n "__fish_tally_using_subcommand scan" -l dry-run -d 'Show what would change without writing files'
complete -c tally -n "__fish_tally_using_subcommand scan" -l git -d 'Include git commit scanning'
complete -c tally -n "__fish_tally_using_subcommand scan" -l todo -d 'Include source TODO scanning'
complete -c tally -n "__fish_tally_using_subcommand scan" -l done -d 'Include source DONE scanning'
complete -c tally -n "__fish_tally_using_subcommand scan" -l json -d 'Output result as JSON'
complete -c tally -n "__fish_tally_using_subcommand scan" -s h -l help -d 'Print help'
complete -c tally -n "__fish_tally_using_subcommand help; and not __fish_seen_subcommand_from add done list semver remove yank scan help" -f -a "add" -d 'Add a new task to TODO.md'
complete -c tally -n "__fish_tally_using_subcommand help; and not __fish_seen_subcommand_from add done list semver remove yank scan help" -f -a "done" -d 'Mark a task as completed using fuzzy description matching'
complete -c tally -n "__fish_tally_using_subcommand help; and not __fish_seen_subcommand_from add done list semver remove yank scan help" -f -a "list" -d 'List tasks with optional filters'
complete -c tally -n "__fish_tally_using_subcommand help; and not __fish_seen_subcommand_from add done list semver remove yank scan help" -f -a "semver" -d 'Move completed unversioned tasks into CHANGELOG.md under a version'
complete -c tally -n "__fish_tally_using_subcommand help; and not __fish_seen_subcommand_from add done list semver remove yank scan help" -f -a "remove" -d 'Remove a task by fuzzy description match from TODO.md or a released entry'
complete -c tally -n "__fish_tally_using_subcommand help; and not __fish_seen_subcommand_from add done list semver remove yank scan help" -f -a "yank" -d 'Yank a changelog entry back into TODO as completed and unversioned'
complete -c tally -n "__fish_tally_using_subcommand help; and not __fish_seen_subcommand_from add done list semver remove yank scan help" -f -a "scan" -d 'Scan for task updates from git commits and/or source TODO markers'
complete -c tally -n "__fish_tally_using_subcommand help; and not __fish_seen_subcommand_from add done list semver remove yank scan help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
