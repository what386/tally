use anyhow::Result;

use crate::application::cli::arguments::{Cli, Commands, ConfigAction};
use crate::application::features;

impl Cli {
    pub async fn run(self) -> Result<()> {
        match self.command {
            Commands::Add { description, priority, tags, dry_run } => unimplemented!("add"),
            Commands::Done { description, commit, version, dry_run } => unimplemented!("done"),
            Commands::List { tags, priority, json } => unimplemented!("list"),
            Commands::Release { version, dry_run, summary } => unimplemented!("release"),
            Commands::Changelog { from, to } => unimplemented!("changelog"),
            Commands::Scan { auto, threshold, dry_run } => unimplemented!("scan"),
            Commands::Edit => unimplemented!("edit"),
            Commands::Config { action } => match action {
                ConfigAction::List => unimplemented!("config-list"),
                ConfigAction::Set { key, value } => unimplemented!("config-set"),
                ConfigAction::Get { key } => unimplemented!("config-get"),
            }
        }
    }
}
