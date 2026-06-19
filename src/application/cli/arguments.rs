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
    /// Add a new task to TODO.md.
    Add {
        /// Task text to add.
        #[arg(required = true, num_args = 1..)]
        description: Vec<String>,
        /// Priority for the new task.
        #[arg(short, long, value_enum)]
        priority: Option<Priority>,
        /// Comma-separated tags to attach.
        #[arg(short, long, value_delimiter = ',')]
        tags: Option<Vec<String>>,
        /// Show what would be added without writing TODO.md.
        #[arg(long, default_value_t = false)]
        dry_run: bool,
        /// Auto-commit updated files after adding.
        #[arg(long, default_value_t = false)]
        auto: bool,
    },

    /// Mark a task as completed using fuzzy description matching.
    Done {
        /// Task text to match.
        #[arg(required = true, num_args = 1..)]
        description: Vec<String>,
        /// Commit hash to associate with completion.
        #[arg(short, long)]
        commit: Option<String>,
        /// Release version to attach at completion time.
        #[arg(short, long)]
        version: Option<String>,
        /// Show what would be changed without writing TODO.md.
        #[arg(long, default_value_t = false)]
        dry_run: bool,
        /// Auto-commit updated files after completion.
        #[arg(long, default_value_t = false)]
        auto: bool,
    },

    /// List tasks with optional filters.
    List {
        /// Filter by one or more comma-separated tags.
        #[arg(short, long, value_delimiter = ',')]
        tags: Option<Vec<String>>,
        /// Filter by priority.
        #[arg(short, long, value_enum)]
        priority: Option<Priority>,
        /// Show only completed tasks.
        #[arg(long, default_value_t = false)]
        done: bool,
        /// List released tasks from CHANGELOG.md for a specific version.
        #[arg(short = 'r', long, value_name = "VERSION")]
        released: Option<String>,
        /// Output results as JSON.
        #[arg(long, default_value_t = false)]
        json: bool,
    },

    /// Move completed unversioned tasks into CHANGELOG.md under a version.
    Semver {
        /// Version to assign (for example: 1.2.3 or v1.2.3).
        version: String,
        /// Show what would be moved without writing files.
        #[arg(long, default_value_t = false)]
        dry_run: bool,
        /// Print a summary of tasks moved for this version.
        #[arg(long, default_value_t = false)]
        summary: bool,
        /// Auto-commit updated files after semver move.
        #[arg(long, default_value_t = false)]
        auto: bool,
    },

    /// Remove a task by fuzzy description match from TODO.md or a released entry.
    Remove {
        /// Task text to match.
        #[arg(required = true, num_args = 1..)]
        description: Vec<String>,
        /// Remove from CHANGELOG.md in a specific version instead of TODO.md.
        #[arg(short = 'r', long, value_name = "VERSION")]
        released: Option<String>,
        /// Filter candidate tasks by one or more comma-separated tags before matching.
        #[arg(short, long, value_delimiter = ',')]
        tags: Option<Vec<String>>,
        /// Show what would be removed without writing TODO.md.
        #[arg(long, default_value_t = false)]
        dry_run: bool,
        /// Auto-commit updated files after removal.
        #[arg(long, default_value_t = false)]
        auto: bool,
    },

    /// Yank a changelog entry back into TODO as completed and unversioned.
    Yank {
        /// Released task text to match.
        #[arg(required = true, num_args = 1..)]
        description: Vec<String>,
        /// Optional tag filter to narrow released-task matching.
        #[arg(short, long, value_delimiter = ',')]
        tags: Option<Vec<String>>,
        /// Show what would be yanked without writing files.
        #[arg(long, default_value_t = false)]
        dry_run: bool,
        /// Auto-commit updated files after yank.
        #[arg(long, default_value_t = false)]
        auto: bool,
    },

    /// Scan for task updates from git commits and/or source TODO markers.
    Scan {
        /// Auto-accept git-based done matches without prompting.
        #[arg(long, default_value_t = false)]
        auto: bool,
        /// Show what would change without writing files.
        #[arg(long, default_value_t = false)]
        dry_run: bool,
        /// Include git commit scanning.
        #[arg(long, default_value_t = false)]
        git: bool,
        /// Include source TODO scanning.
        #[arg(long, default_value_t = false)]
        todo: bool,
        /// Include source DONE scanning.
        #[arg(long, default_value_t = false)]
        done: bool,
    },
}
