use anyhow::Result;

use crate::application::cli::arguments::{Cli, Commands};
use crate::application::commands;

impl Cli {
    pub fn run(self) -> Result<()> {
        match self.command {
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
                released,
                json,
            } => commands::cmd_list(tags, priority, done, released, json),

            Commands::Semver {
                version,
                dry_run,
                summary,
                auto,
            } => commands::cmd_semver(version, dry_run, summary, auto),

            Commands::Remove {
                description,
                released,
                tags,
                dry_run,
                auto,
            } => commands::cmd_remove(join_words(description), released, tags, dry_run, auto),

            Commands::Yank {
                description,
                tag,
                dry_run,
                auto,
            } => commands::cmd_yank(join_words(description), tag, dry_run, auto),

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
