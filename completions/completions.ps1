
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'tally' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'tally'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'tally' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add a new task to TODO.md')
            [CompletionResult]::new('done', 'done', [CompletionResultType]::ParameterValue, 'Mark a task as completed using fuzzy description matching')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List tasks with optional filters')
            [CompletionResult]::new('semver', 'semver', [CompletionResultType]::ParameterValue, 'Move completed unversioned tasks into CHANGELOG.md under a version')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Remove a task by fuzzy description match from TODO.md or a released entry')
            [CompletionResult]::new('yank', 'yank', [CompletionResultType]::ParameterValue, 'Yank a changelog entry back into TODO as completed and unversioned')
            [CompletionResult]::new('scan', 'scan', [CompletionResultType]::ParameterValue, 'Scan for task updates from git commits and/or source TODO markers')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'tally;add' {
            [CompletionResult]::new('-p', '-p', [CompletionResultType]::ParameterName, 'Priority for the new task')
            [CompletionResult]::new('--priority', '--priority', [CompletionResultType]::ParameterName, 'Priority for the new task')
            [CompletionResult]::new('-t', '-t', [CompletionResultType]::ParameterName, 'Comma-separated tags to attach')
            [CompletionResult]::new('--tags', '--tags', [CompletionResultType]::ParameterName, 'Comma-separated tags to attach')
            [CompletionResult]::new('--dry-run', '--dry-run', [CompletionResultType]::ParameterName, 'Show what would be added without writing TODO.md')
            [CompletionResult]::new('--auto', '--auto', [CompletionResultType]::ParameterName, 'Auto-commit updated files after adding')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'tally;done' {
            [CompletionResult]::new('-c', '-c', [CompletionResultType]::ParameterName, 'Commit hash to associate with completion')
            [CompletionResult]::new('--commit', '--commit', [CompletionResultType]::ParameterName, 'Commit hash to associate with completion')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Release version to attach at completion time')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Release version to attach at completion time')
            [CompletionResult]::new('--dry-run', '--dry-run', [CompletionResultType]::ParameterName, 'Show what would be changed without writing TODO.md')
            [CompletionResult]::new('--auto', '--auto', [CompletionResultType]::ParameterName, 'Auto-commit updated files after completion')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'tally;list' {
            [CompletionResult]::new('-t', '-t', [CompletionResultType]::ParameterName, 'Filter by one or more comma-separated tags')
            [CompletionResult]::new('--tags', '--tags', [CompletionResultType]::ParameterName, 'Filter by one or more comma-separated tags')
            [CompletionResult]::new('-p', '-p', [CompletionResultType]::ParameterName, 'Filter by priority')
            [CompletionResult]::new('--priority', '--priority', [CompletionResultType]::ParameterName, 'Filter by priority')
            [CompletionResult]::new('-r', '-r', [CompletionResultType]::ParameterName, 'List released tasks from CHANGELOG.md for a specific version')
            [CompletionResult]::new('--released', '--released', [CompletionResultType]::ParameterName, 'List released tasks from CHANGELOG.md for a specific version')
            [CompletionResult]::new('--done', '--done', [CompletionResultType]::ParameterName, 'Show only completed tasks')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output results as JSON')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'tally;semver' {
            [CompletionResult]::new('--dry-run', '--dry-run', [CompletionResultType]::ParameterName, 'Show what would be moved without writing files')
            [CompletionResult]::new('--summary', '--summary', [CompletionResultType]::ParameterName, 'Print a summary of tasks moved for this version')
            [CompletionResult]::new('--auto', '--auto', [CompletionResultType]::ParameterName, 'Auto-commit updated files after semver move')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'tally;remove' {
            [CompletionResult]::new('-r', '-r', [CompletionResultType]::ParameterName, 'Remove from CHANGELOG.md in a specific version instead of TODO.md')
            [CompletionResult]::new('--released', '--released', [CompletionResultType]::ParameterName, 'Remove from CHANGELOG.md in a specific version instead of TODO.md')
            [CompletionResult]::new('-t', '-t', [CompletionResultType]::ParameterName, 'Filter candidate tasks by one or more comma-separated tags before matching')
            [CompletionResult]::new('--tags', '--tags', [CompletionResultType]::ParameterName, 'Filter candidate tasks by one or more comma-separated tags before matching')
            [CompletionResult]::new('--dry-run', '--dry-run', [CompletionResultType]::ParameterName, 'Show what would be removed without writing TODO.md')
            [CompletionResult]::new('--auto', '--auto', [CompletionResultType]::ParameterName, 'Auto-commit updated files after removal')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'tally;yank' {
            [CompletionResult]::new('-t', '-t', [CompletionResultType]::ParameterName, 'Optional tag filter to narrow released-task matching')
            [CompletionResult]::new('--tags', '--tags', [CompletionResultType]::ParameterName, 'Optional tag filter to narrow released-task matching')
            [CompletionResult]::new('--dry-run', '--dry-run', [CompletionResultType]::ParameterName, 'Show what would be yanked without writing files')
            [CompletionResult]::new('--auto', '--auto', [CompletionResultType]::ParameterName, 'Auto-commit updated files after yank')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'tally;scan' {
            [CompletionResult]::new('--auto', '--auto', [CompletionResultType]::ParameterName, 'Auto-accept git-based done matches without prompting')
            [CompletionResult]::new('--dry-run', '--dry-run', [CompletionResultType]::ParameterName, 'Show what would change without writing files')
            [CompletionResult]::new('--git', '--git', [CompletionResultType]::ParameterName, 'Include git commit scanning')
            [CompletionResult]::new('--todo', '--todo', [CompletionResultType]::ParameterName, 'Include source TODO scanning')
            [CompletionResult]::new('--done', '--done', [CompletionResultType]::ParameterName, 'Include source DONE scanning')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'tally;help' {
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add a new task to TODO.md')
            [CompletionResult]::new('done', 'done', [CompletionResultType]::ParameterValue, 'Mark a task as completed using fuzzy description matching')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List tasks with optional filters')
            [CompletionResult]::new('semver', 'semver', [CompletionResultType]::ParameterValue, 'Move completed unversioned tasks into CHANGELOG.md under a version')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Remove a task by fuzzy description match from TODO.md or a released entry')
            [CompletionResult]::new('yank', 'yank', [CompletionResultType]::ParameterValue, 'Yank a changelog entry back into TODO as completed and unversioned')
            [CompletionResult]::new('scan', 'scan', [CompletionResultType]::ParameterValue, 'Scan for task updates from git commits and/or source TODO markers')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'tally;help;add' {
            break
        }
        'tally;help;done' {
            break
        }
        'tally;help;list' {
            break
        }
        'tally;help;semver' {
            break
        }
        'tally;help;remove' {
            break
        }
        'tally;help;yank' {
            break
        }
        'tally;help;scan' {
            break
        }
        'tally;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
