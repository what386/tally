---
name: tally
description: Manage project work tracked in repository TODO.md and CHANGELOG.md files using the tally CLI. Use when adding tasks, completing tasks, removing tasks, releasing completed work, yanking changelog entries back into TODO.md, scanning git/source markers, reviewing pending/completed/released work, or safely previewing changes before writes.
---

# Tally

Use this skill to drive task tracking in repos that use `tally`, `TODO.md`, and optionally `CHANGELOG.md`.

`tally` is a Git-friendly task manager for plain-text project task files. It supports active tasks, completed-but-unreleased tasks, released changelog entries, fuzzy matching, tags, priorities, source/git scanning, JSON output, and optional auto-commits.

## Quick Workflow

1. Add work items with clear descriptions and metadata.
2. Review active/completed/released work with `tally list`.
3. Complete work items with fuzzy matching and optional commit/version linkage.
4. Move completed unreleased work into `CHANGELOG.md` with `tally semver`.
5. Use `--dry-run` before broad, fuzzy, destructive, or low-confidence operations.
6. After any write action, verify with `tally list`, `tally list --done`, or `tally list --released VERSION`.

## Add Tasks

Prefer short, action-oriented descriptions. Prefer explicit flags for metadata when possible.

```bash
tally add "Fix parsing error"
tally add "Implement parser recovery" --priority high --tags parser,feature
tally add "Update docs" --dry-run
tally add "Update docs" --dry-run --json
```

Inline metadata is also supported:

```bash
tally add "Implement feature flag (high) #feature #backend"
```

Guidance:

* Prefer `--priority low|medium|high` over inline priority when generating commands.
* Prefer `--tags tag1,tag2` over inline hashtags when generating commands.
* Use inline `(priority)` and `#tags` when preserving user-provided task wording.
* Use `--dry-run` before adding many tasks or when unsure how metadata will be parsed.
* Use `--json` when another tool or script will consume the result.
* Use `--auto` only when the user wants tally to auto-commit the change.

## List and Inspect Tasks

Use `tally list` to verify repository task state.

```bash
tally list
tally list --done
tally list --tags bug,parser
tally list --priority high
tally list --released v0.6.0
tally list --json
```

Guidance:

* Use `tally list` for active TODO entries.
* Use `tally list --done` for completed-but-unreleased entries.
* Use `tally list --released VERSION` for entries already moved to `CHANGELOG.md`.
* Use `--json` when exact structured output is needed.

## Mark Tasks Done

Use fuzzy matching on existing task text. Start with the most specific recognizable phrase available.

```bash
tally done "Fix parsing error"
tally done "Fix parsing error" --commit abc123f
tally done "Fix parsing error" --version v0.2.3
tally done "Fix parsing error" --dry-run
tally done "Fix parsing error" --dry-run --json
```

Guidance:

* Prefer exact or near-exact task wording to reduce ambiguous matches.
* Use `--commit` when completion corresponds to a known git commit.
* Use `--version` when completion belongs directly to a release.
* Use `--dry-run` when match confidence may be low.
* If multiple tasks may match, inspect first with `tally list` or use a more specific phrase.
* After writing, verify with `tally list --done`.

## Release Completed Work

Use `semver` to move completed, unversioned tasks from `TODO.md` into `CHANGELOG.md` under a release version.

```bash
tally semver v0.2.3
tally semver v0.2.3 --dry-run
tally semver v0.2.3 --summary
tally semver v0.2.3 --json
```

Guidance:

* Run `tally list --done` before release operations when reviewing what will move.
* Prefer `tally semver VERSION --dry-run` before writing a release.
* Use `--summary` when the user wants a release summary.
* Use `--auto` only when the user wants tally to auto-commit the release-file changes.
* Verify released entries with `tally list --released VERSION`.

## Remove Tasks

Use `remove` to delete a task by fuzzy match from `TODO.md`, or from a released changelog version.

```bash
tally remove "Old task"
tally remove "Old task" --tags cleanup
tally remove "Old task" --released v0.2.3
tally remove "Old task" --dry-run
```

Guidance:

* Treat removal as destructive; prefer `--dry-run` first.
* Use `--tags` to narrow candidates before fuzzy matching.
* Use `--released VERSION` when removing from `CHANGELOG.md`.
* Verify with `tally list`, `tally list --done`, or `tally list --released VERSION`.

## Yank Released Entries

Use `yank` to move a released changelog entry back into `TODO.md` as completed and unversioned.

```bash
tally yank "Fix parsing"
tally yank v0.2.3 "Fix parsing"
tally yank v0.2.3
tally yank "Fix parsing" --tags parser
tally yank v0.2.3 "Fix parsing" --dry-run
```

Guidance:

* Use a version in the match text when the release is known.
* A version-only yank is safe only when that release has exactly one matching entry.
* Use `--tags` to narrow released-task matching.
* Prefer `--dry-run` before yanking.
* Verify with `tally list --done` and `tally list --released VERSION`.

## Scan Git and Source Markers

Use `scan` to detect task updates from git commits and/or source markers.

```bash
tally scan
tally scan --git
tally scan --todo
tally scan --done
tally scan --dry-run
tally scan --json --dry-run
```

Git scan detects completed work from commit messages using the configured done section, usually:

```text
done:
- fix parsing error
- handle quoted strings
```

Source scanning detects configured markers such as:

```rust
// TODO: Implement parser recovery (high) #parser
// DONE: Remove old parser workaround
```

Guidance:

* Use `tally scan --dry-run` before applying detected changes.
* Use `--git` to scan commit messages.
* Use `--todo` to add source TODO markers as tasks.
* Use `--done` to match source DONE markers against existing tasks.
* Use `--auto` on `scan` only when the user wants git-based done matches auto-accepted without prompting.
* Use `--json` when results must be consumed by another tool.

## Auto-Commit Behavior

Many write commands support `--auto`:

```bash
tally add "Update docs" --auto
tally done "Fix parsing" --auto
tally remove "Old task" --auto
tally semver v0.2.3 --auto
tally yank "Fix parsing" --auto
```

Guidance:

* For write commands, `--auto` requests an auto-commit of updated files.
* For `scan`, `--auto` means auto-accept git-based done matches.
* Do not add `--auto` unless the user explicitly wants tally to make the related automated action.
* Be aware that config may also enable auto-commit behavior per command.

## Safety Rules

* Prefer `--dry-run` before destructive, broad, fuzzy, release, yank, remove, or low-confidence operations.
* Prefer exact recognizable task wording for fuzzy-match commands.
* Use `--tags` and `--released VERSION` to narrow candidates when possible.
* Use `--json` for machine-readable inspection or automation.
* After any write action, run a relevant verification command:

  * `tally list`
  * `tally list --done`
  * `tally list --released VERSION`
* Do not silently choose among ambiguous fuzzy matches. Inspect first or use a more specific phrase.

