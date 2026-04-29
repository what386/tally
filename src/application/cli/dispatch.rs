use anyhow::Result;

use crate::application::cli::arguments::{Cli, Commands};
use crate::application::commands;

impl Cli {
    pub fn run(self) -> Result<()> {
        match self.command {
            Commands::Init => commands::cmd_init(),
            Commands::Edit => commands::cmd_edit(),

            Commands::Add {
                description,
                priority,
                tags,
                dry_run,
                auto,
            } => commands::cmd_add(description, priority, tags, dry_run, auto),

            Commands::Done {
                description,
                commit,
                version,
                dry_run,
                auto,
            } => commands::cmd_done(description, commit, version, dry_run, auto),

            Commands::List {
                tags,
                priority,
                done,
                semver,
                json,
            } => commands::cmd_list(tags, priority, done, semver, json),

            Commands::Semver {
                version,
                dry_run,
                summary,
                auto,
            } => commands::cmd_semver(version, dry_run, summary, auto),

            Commands::Prune {
                days,
                hours,
                dry_run,
                auto,
            } => commands::cmd_prune(days, hours, dry_run, auto),

            Commands::Remove {
                description,
                dry_run,
                auto,
            } => commands::cmd_remove(description, dry_run, auto),

            Commands::Changelog { from, to } => commands::cmd_changelog(from, to),

            Commands::Scan { auto, dry_run } => commands::cmd_scan(auto, dry_run),
        }
    }
}
