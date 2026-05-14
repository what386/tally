use clap::{Parser, Subcommand};

use crate::models::common::Priority;

#[derive(Parser)]
#[command(name = "tally")]
#[command(about = "A task management tool for TODO.md files")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Edit,

    Add {
        #[arg(required = true, num_args = 1..)]
        description: Vec<String>,
        #[arg(short, long, value_enum, default_value_t = Priority::Medium)]
        priority: Priority,
        #[arg(short, long, value_delimiter = ',')]
        tags: Option<Vec<String>>,
        #[arg(long, default_value_t = false)]
        dry_run: bool,
        #[arg(long, default_value_t = false)]
        auto: bool,
    },

    Done {
        #[arg(required = true, num_args = 1..)]
        description: Vec<String>,
        #[arg(short, long)]
        commit: Option<String>,
        #[arg(short, long)]
        version: Option<String>,
        #[arg(long, default_value_t = false)]
        dry_run: bool,
        #[arg(long, default_value_t = false)]
        auto: bool,
    },

    List {
        #[arg(short, long, value_delimiter = ',')]
        tags: Option<Vec<String>>,
        #[arg(short, long, value_enum)]
        priority: Option<Priority>,
        #[arg(long, default_value_t = false)]
        done: bool,
        #[arg(long, default_value_t = false)]
        json: bool,
    },

    Semver {
        version: String,
        #[arg(long, default_value_t = false)]
        dry_run: bool,
        #[arg(long, default_value_t = false)]
        summary: bool,
        #[arg(long, default_value_t = false)]
        auto: bool,
    },

    Changelog {
        #[arg(long)]
        from: Option<String>,
        #[arg(long)]
        to: Option<String>,
    },

    Released {
        #[arg(long)]
        version: Option<String>,
        #[arg(num_args = 1..)]
        query: Option<Vec<String>>,
    },

    Unrelease {
        #[arg(required = true, num_args = 1..)]
        description: Vec<String>,
        #[arg(long)]
        version: Option<String>,
        #[arg(long, default_value_t = false)]
        dry_run: bool,
        #[arg(long, default_value_t = false)]
        auto: bool,
    },

    Remove {
        #[arg(required = true, num_args = 1..)]
        description: Vec<String>,
        #[arg(long, default_value_t = false)]
        dry_run: bool,
        #[arg(long, default_value_t = false)]
        auto: bool,
    },

    Scan {
        #[arg(long, default_value_t = false)]
        auto: bool,
        #[arg(long, default_value_t = false)]
        dry_run: bool,
        #[arg(long, default_value_t = false)]
        git: bool,
        #[arg(long, default_value_t = false)]
        source: bool,
    },
}
