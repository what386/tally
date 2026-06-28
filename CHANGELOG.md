# Changelog — tally

*Generated on 2026-06-28*

## 0.15.0 — 2026-06-28

### Changes

- feat: emit json for more commands
- feat: make fuzzy match stricter
- feat: try to preserve manual changelog sections


## 0.14.0 — 2026-06-21

### Changes

- feat: pagination for long command outputs
- feat: natural tag/priority processing from input
- feat: source scan imports TODO markers as tasks
- feat: source DONE markers complete matching tasks
- feat: configurable scan markers and git log limit
- feat: configurable matching thresholds
- feat: command-specific auto-commit settings
- fix: yank literal release prefixes and full semver releases


## 0.13.0 — 2026-05-23

### Changes

- make scan able to init tally


## 0.12.0 — 2026-05-14

### Changes

- unify tag filtering
- seperate scan flags


## 0.10.0 — 2026-05-14

### Changes

- feat: scan for ' DONE: ' to complete tasks in source code
- remove more subcommands


## 0.9.0 — 2026-05-14

### Changes

- reformat to use CHANGELOG as storage and TODO as scratchpad
- add source scanner function
- remove some commands


## 0.8.0 — 2026-04-29

### Changes

- trim features


## 0.7.0 — 2026-04-07

### High Priority

- Add --auto and config auto-commit support to changelog command `feature`
- Add --done and --semver filters to tally list `feature`, `ux`


## 0.6.0 — 2026-02-24

### High Priority

- Add tally edit command to open TODO.md in configured editor `feature`, `ux`
- Add cross-project registry and aggregated project status command `feature`, `workspace`
- Fix changelog duplicate entries `bug`
- Add high-priority regression tests for version/config/scan parsing `test`
- Add publish script to create git tag from provided version `feature`, `release`
- Add tally edit command to open TODO.md in configured editor `feature`, `ux`
- Add cross-project registry and aggregated project status command `feature`, `workspace`
- Fix changelog duplicate entries `bug`
- Add high-priority regression tests for version/config/scan parsing `test`
- Add publish script to create git tag from provided version `feature`, `release`
- Fix clippy warnings across command, model, and storage modules `cleanup`
- Remove dead code methods flagged by clippy `cleanup`
- Fix clippy warnings across command, model, and storage modules `cleanup`
- Remove dead code methods flagged by clippy `cleanup`


## 0.5.0 — 2026-02-02

### High Priority

- github doing! ([`9ba2c4a`])


## 0.4.0 — 2026-02-02

### Minor Changes

- config support `feature`


## 0.3.2 — 2026-02-01

### High Priority

- fix duplication in history ([`a556fb5`])


## 0.3.1 — 2026-02-01

### Minor Changes

- git hook for tags? ([`3c1b958`])


## 0.1.1 — 2026-01-31

### High Priority

- make a .tally/history.json file to track tasks (even if deleted by tally prune) for changelogs

### Changes

- implement 'tally prune [--days] [--hours]' to remove old tasks `feature`
- add 'tally remove' to remove tasks (instead of marking them as completed)

### Minor Changes

- modify changelog to use history.json instead of parsing TODO.md


