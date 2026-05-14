use anyhow::Result;

use crate::application::cli::arguments::{Cli, Commands};
use crate::application::commands;

impl Cli {
    pub fn run(self) -> Result<()> {
        match self.command {
            Commands::Edit => commands::cmd_edit(),

            Commands::Add {
                description,
                priority,
                tags,
                dry_run,
                auto,
            } => commands::cmd_add(join_words(description), priority, tags, dry_run, auto),

            Commands::Done {
                description,
                commit,
                version,
                dry_run,
                auto,
            } => commands::cmd_done(join_words(description), commit, version, dry_run, auto),

            Commands::List {
                tags,
                priority,
                done,
                json,
            } => commands::cmd_list(tags, priority, done, json),

            Commands::Semver {
                version,
                dry_run,
                summary,
                auto,
            } => commands::cmd_semver(version, dry_run, summary, auto),

            Commands::Remove {
                description,
                dry_run,
                auto,
            } => commands::cmd_remove(join_words(description), dry_run, auto),

            Commands::Changelog { from, to } => commands::cmd_changelog(from, to),

            Commands::Released { version, query } => {
                commands::cmd_released(version, query.map(join_words))
            }

            Commands::Unrelease {
                description,
                version,
                dry_run,
                auto,
            } => commands::cmd_unrelease(join_words(description), version, dry_run, auto),

            Commands::Scan {
                auto,
                dry_run,
                git,
                source,
            } => commands::cmd_scan(auto, dry_run, git, source),
        }
    }
}

fn join_words(words: Vec<String>) -> String {
    words.join(" ")
}
