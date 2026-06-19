# tally

**tally** is a lightweight, Git-friendly task manager for projects that live in a `TODO.md` file.

It tracks tasks, marks them complete, associates completions with git commits or release versions, moves released work into `CHANGELOG.md`, and can scan source files or git history for task updates.

---

## Features

- Plain-text `TODO.md` task storage
- Plain-text `CHANGELOG.md` release storage
- Fuzzy task matching for `done`, `remove`, `scan`, and `yank`
- Tags and priorities
- Git commit scanning from configurable `done:` sections
- Source scanning from configurable `TODO:` / `DONE:` markers
- Release management with `semver`
- Yank released changelog entries back into TODO
- Optional auto-commit per command
- Prompt, always, or never policy for tracking newly created tally files with git
- Pager support for large command output

---

## Installation

Install from crates.io:

```bash
cargo install tally-todo
```

Update an existing install:

```bash
cargo install --force tally-todo
```

Build from source:

```bash
git clone https://github.com/what386/tally.git
cd tally
cargo build --release
```

The built binary is:

```text
target/release/tally
```

---

## Getting Started

Create the first task in a project directory:

```bash
tally add "Fix parsing error"
```

If `TODO.md` does not exist, tally creates it in the current directory. Release operations create `CHANGELOG.md` when needed.

When auto-commit is enabled and tally creates `TODO.md` or `CHANGELOG.md`, it can prompt to `git add` those files before committing. This behavior is controlled by `git.track_created_files`.

---

## Commands

### Add Tasks

```bash
tally add "Fix parsing error"
```

Set priority and tags:

```bash
tally add "Implement parser recovery" --priority high --tags parser,feature
```

You can also include TODO-style priority and tags in the task text:

```bash
tally add "Implement a new backend (high) #backend #improvement"
```

Explicit flags override metadata parsed from the task text:

```bash
tally add "Implement a new backend (low) #backend" --priority high --tags api
```

Preview without writing:

```bash
tally add "Update docs" --dry-run
```

Auto-commit this add regardless of config:

```bash
tally add "Update docs" --auto
```

### List Tasks

```bash
tally list
```

Filter active TODO entries:

```bash
tally list --tags bug,parser
tally list --priority high
tally list --done
```

List released changelog entries for an exact version:

```bash
tally list --released v0.6.0
```

Output JSON:

```bash
tally list --json
tally list --released v0.6.0 --json
```

Large list output is paged automatically when stdout is an interactive terminal.

### Complete Tasks

Mark a task complete using fuzzy matching:

```bash
tally done "Fix parsing"
```

Attach completion metadata:

```bash
tally done "Fix parsing" --commit abc123f
tally done "Fix parsing" --version v0.2.3
```

Preview:

```bash
tally done "Fix parsing" --dry-run
```

`matching.task_min_score` controls how confident a fuzzy match must be for task commands.

### Release Completed Work

Move completed, unversioned tasks into `CHANGELOG.md`:

```bash
tally semver v0.2.3
```

Preview or print the moved task summary:

```bash
tally semver v0.2.3 --dry-run
tally semver v0.2.3 --summary
```

### Remove Tasks

Remove a TODO entry by fuzzy match:

```bash
tally remove "Old task"
```

Remove a released changelog entry from a specific version:

```bash
tally remove "Old task" --released v0.2.3
```

Narrow candidates by tag:

```bash
tally remove "Old task" --tags cleanup
```

Preview:

```bash
tally remove "Old task" --dry-run
```

### Yank Released Entries

Move a released changelog entry back into `TODO.md` as completed and unversioned:

```bash
tally yank "Fix parsing"
```

You can include a release version tag in the match text to restrict the search:

```bash
tally yank v0.2.3 "Fix parsing"
```

A version-only yank works when that release has exactly one matching entry:

```bash
tally yank v0.2.3
```

Use tags or dry-run:

```bash
tally yank "Fix parsing" --tags parser
tally yank v0.2.3 "Fix parsing" --dry-run
```

`matching.released_min_score` controls fuzzy matching for released changelog entries.

### Scan Git and Source

Scan git commits and tracked source files:

```bash
tally scan
```

Limit scan modes:

```bash
tally scan --git
tally scan --todo
tally scan --done
```

Preview or auto-accept git commit matches:

```bash
tally scan --dry-run
tally scan --auto
```

Git scan looks for a configurable `done:` section in recent commit messages:

```text
implement parser recovery

done:
- fix parsing error
- handle quoted strings
```

Source scan looks at git-tracked files and ignores `TODO.md` / `CHANGELOG.md`. By default it recognizes comment markers like:

```rust
// TODO: write parser recovery tests
// DONE: remove old parser workaround
```

Configured source markers can include alternatives such as `FIXME:` or `SHIPPED:`.

---

## Configuration

Configuration is stored at:

```text
~/.config/tally/config.toml
```

Tally creates missing config directories and fills missing sections with defaults, so older config files continue to load.

Example:

```toml
[preferences]
auto_commit_todo = false
auto_complete_tasks = false
editor = "nvim"

[auto_commit]
add = false
done = true
remove = false
semver = true
yank = true

[git]
done_prefix = "done:"
track_created_files = "prompt" # prompt | always | never

[scan]
git_log_limit = 100
todo_markers = ["TODO:", "FIXME:"]
done_markers = ["DONE:", "SHIPPED:"]

[matching]
task_min_score = 50.0
source_done_min_score = 50.0
released_min_score = 50.0
```

### Config Reference

`preferences.auto_commit_todo`

Legacy broad auto-commit switch. When true, all write commands that support commits will auto-commit. The command-specific `[auto_commit]` keys are preferred for new configs.

`preferences.auto_complete_tasks`

Auto-accept git-based scan matches without prompting.

`preferences.editor`

Reserved for editor preference support.

`auto_commit.add`, `auto_commit.done`, `auto_commit.remove`, `auto_commit.semver`, `auto_commit.yank`

Enable auto-commit for individual write commands.

`git.done_prefix`

Commit-message section header used by `tally scan --git`. Default: `done:`.

`git.track_created_files`

Controls what happens when auto-commit sees newly created `TODO.md` or `CHANGELOG.md`:

- `prompt`: ask before `git add`
- `always`: add them automatically
- `never`: fail instead of tracking them

`scan.git_log_limit`

Number of recent commits to inspect. Default: `50`.

`scan.todo_markers`, `scan.done_markers`

Markers used by source scanning. Defaults: `["TODO:"]` and `["DONE:"]`.

`matching.task_min_score`

Minimum fuzzy score for task matching in `done`, `remove`, and git scan completion matching.

`matching.source_done_min_score`

Minimum fuzzy score for matching source `DONE:` markers to existing TODO tasks.

`matching.released_min_score`

Minimum fuzzy score for released changelog entry matching in `remove --released` and `yank`.

---

## Storage Format

- `TODO.md`: active and completed-but-unreleased tasks
- `CHANGELOG.md`: released entries grouped by semver
- `~/.config/tally/config.toml`: user preferences and matching/scan behavior

All project data is human-readable and git-friendly.

---

## Philosophy

`tally` is designed to:

- Stay out of your way
- Work naturally with git
- Avoid lock-in and opaque formats
- Keep TODO and changelog history editable as plain text
