# tally

**tally** is a lightweight, Git-friendly task manager for projects that live in a `TODO.md` file.

It lets you track tasks, mark them complete, associate them with git commits or releases, and automatically generate changelogs — all from the command line, without a database or daemon.

---

## Table of Contents

1. [Features](#features)
2. [Installation](#installation)
   - [Install via Cargo](#install-via-cargo)
   - [Build from Source](#build-from-source)
3. [Getting Started](#getting-started)
4. [Usage](#usage)
   - [Initialize](#initialize)
   - [Add Tasks](#add-tasks)
   - [List Tasks](#list-tasks)
   - [Complete Tasks](#complete-tasks)
   - [Release Management](#release-management)
   - [Changelog Generation](#changelog-generation)
   - [Task Cleanup](#task-cleanup)
   - [Git Integration](#git-integration)
   - [Editing Tasks](#editing-tasks)
5. [Configuration](#configuration)
6. [Storage Format](#storage-format)

---

## Features

- Uses a plain `TODO.md` file as storage
- Fuzzy-matching to find tasks by description
- Tags and priorities for better organization
- Automatic changelog generation from completed tasks
- Associate tasks with git commits or release versions
- Prune or archive old completed tasks
- Scan git history to detect completed tasks

---

## Installation

### Install via Cargo

`tally` is written in Rust and can be installed with Cargo:

```bash
cargo install tally
```

Ensure Cargo’s bin directory is in your `PATH`:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

To update:

```bash
cargo install --force tally
```

---

### Build from Source

```bash
git clone https://github.com/yourname/tally.git
cd tally
cargo build --release
```

Binary location:

```text
target/release/tally
```

---

## Getting Started

Initialize `tally` in your project directory:

```bash
tally init
```

This creates:

- `TODO.md` — your task list
- `.tally/` — configuration and history files

---

## Usage

### Git Commit Integration

`tally` can automatically detect completed tasks by scanning git commit messages.

This works by looking for a special **`done:` section** in commit messages and fuzzy-matching the listed items against tasks in `TODO.md`.

#### Commit Message Format

Add a `done:` section anywhere in your commit message:

```text
do thing

done:
fix parsing issue
```

Each line under `done:` is treated as a potential completed task.

#### Lists Are Supported

You can use plain lines, dash lists, or bullet lists:

```text
refactor parser

done:
- fix parsing issue
- handle edge cases
```

```text
cleanup

done:
* update docs
* remove unused code
```

`tally` strips common list markers (`-`, `*`) before matching.

#### Fuzzy Matching

Task names do **not** need to match exactly.

For example, this task in `TODO.md`:

```text
- Fix parsing error in format.rs
```

Will match any of the following commit entries:

```text
fix parsing issue
parsing error
fix parser
```

#### Scanning Commits

To scan git history for completed tasks:

```bash
tally scan
```

Preview matches without making changes:

```bash
tally scan --dry-run
```

Automatically mark all detected matches as done:

```bash
tally scan --auto
```

When a task is marked complete via a commit scan, the commit hash is automatically associated with the task.

---

### Initialize

```bash
tally init
```

Initializes `tally` in the current directory.

---

### Add Tasks

```bash
tally add "Fix parsing error"
```

With priority and tags:

```bash
tally add "Implement feature" --priority high --tags feature,backend
```

Preview without writing:

```bash
tally add "Update docs" --dry-run
```

---

### List Tasks

```bash
tally list
```

Filter by tags and priority:

```bash
tally list --tags bug,parser --priority high
```

Output as JSON:

```bash
tally list --json
```

---

### Complete Tasks

Mark a task as completed using fuzzy matching:

```bash
tally done "Fix parsing error"
```

Associate a git commit:

```bash
tally done "Fix parsing error" --commit abc123f
```

Associate a release version:

```bash
tally done "Fix parsing error" --version v0.2.3
```

Preview changes:

```bash
tally done "Fix parsing error" --dry-run
```

---

### Release Management

Assign a version to all completed, unversioned tasks:

```bash
tally release v0.2.3
```

Show a summary:

```bash
tally release v1.0.0 --summary
```

Dry run:

```bash
tally release v0.2.4 --dry-run
```

---

### Changelog Generation

Generate a changelog from completed tasks:

```bash
tally changelog
```

From a specific version:

```bash
tally changelog --from v0.2.2
```

Between versions:

```bash
tally changelog --from v0.2.2 --to v0.2.3
```

---

### Task Cleanup

Remove a task entirely:

```bash
tally remove "Old task"
```

Completed tasks are archived to `history.json` before removal.

Prune completed tasks older than a threshold:

```bash
tally prune            # default: 30 days
tally prune --days 7
tally prune --days 1 --hours 12
tally prune --dry-run
```

---

### Git Integration

Scan git commit messages for completed tasks:

```bash
tally scan
```

Automatically mark matches as done:

```bash
tally scan --auto
```

Preview matches:

```bash
tally scan --dry-run
```

---

### Editing Tasks

Open `TODO.md` in your editor:

```bash
tally edit
```

Uses:

1. Editor set in `tally config`
2. `$EDITOR`
3. Common fallbacks (vim, nano, etc.)

---

## Configuration

Configuration is stored in:

```text
.tally/config.toml
```

Manage configuration via:

```bash
tally config <action>
```

Available actions:

| Action | Description                    |
| ------ | ------------------------------ |
| `set`  | Set a configuration value      |
| `get`  | Retrieve a configuration value |
| `list` | List all configuration keys    |

Examples:

```bash
tally config set default_priority medium
tally config get changelog_template
tally config list
```

---

## Storage Format

- **`TODO.md`** — active tasks
- **`.tally/history.json`** — archived completed tasks
- **`.tally/config.toml`** — user preferences

All data is human-readable and git-friendly.

---

## Philosophy

`tally` is designed to:

- Stay out of your way
- Work naturally with git
- Avoid lock-in and opaque formats
- Make changelogs easy
