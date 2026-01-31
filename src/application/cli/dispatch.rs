use anyhow::Result;

use crate::application::cli::arguments::{Cli, Commands, ConfigAction};
use crate::application::commands::{self, cmd_config_get};
use crate::utils::project_paths::ProjectPaths;

impl Cli {
    pub fn run(self) -> Result<()> {
        match self.command {
            Commands::Init => commands::cmd_init(),
            Commands::Add {
                description,
                priority,
                tags,
                dry_run,
            } => commands::cmd_add(description, priority, tags, dry_run),
            Commands::Done {
                description,
                commit,
                version,
                dry_run,
            } => commands::cmd_done(description, commit, version, dry_run),
            Commands::List {
                tags,
                priority,
                json,
            } => commands::cmd_list(tags, priority, json),
            Commands::Release {
                version,
                dry_run,
                summary,
            } => commands::cmd_release(version, dry_run, summary),
            Commands::Changelog { from, to } => commands::cmd_changelog(from, to),
            Commands::Scan { auto, dry_run } => commands::cmd_scan(auto, dry_run),
            Commands::Edit => commands::cmd_edit(),
            Commands::Config { action } => match action {
                ConfigAction::List => commands::cmd_config_list(),
                ConfigAction::Set { key, value } => commands::cmd_config_set(key, value),
                ConfigAction::Get { key } => cmd_config_get(key),
            },
        }
    }
}
