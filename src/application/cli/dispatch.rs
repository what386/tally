use anyhow::Result;

use crate::application::cli::arguments::{Cli, Commands, ConfigAction, ProjectsAction};
use crate::application::commands::{self, cmd_config_get};

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

            Commands::Status => commands::cmd_status(),

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
                json,
            } => commands::cmd_list(tags, priority, json),

            Commands::Semver {
                version,
                dry_run,
                summary,
                auto,
            } => commands::cmd_semver(version, dry_run, summary, auto),

            Commands::Tag {
                version,
                message,
                dry_run,
                summary,
                auto,
            } => commands::cmd_tag(version, message, dry_run, summary, auto),

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

            Commands::Config { action } => match action {
                ConfigAction::List => commands::cmd_config_list(),
                ConfigAction::Set { key, value } => commands::cmd_config_set(key, value),
                ConfigAction::Get { key } => cmd_config_get(key),
            },

            Commands::Projects { action } => match action {
                ProjectsAction::List => commands::cmd_projects_list(),
                ProjectsAction::Status => commands::cmd_projects_status(),
                ProjectsAction::Prune => commands::cmd_projects_prune(),
            },
        }
    }
}
