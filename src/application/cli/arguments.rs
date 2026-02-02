use clap::{Parser, Subcommand};

use crate::models::common::Priority;

#[derive(Parser)]
#[command(name = "tally")]
#[command(about = "A task management tool for TODO.md files")]
#[command(
    long_about = "tally is a command-line task manager that uses TODO.md as its storage format.\n\n\
    Track tasks, generate changelogs, and integrate with git commits for automatic \
    task completion detection.\n\n\
    EXAMPLES:\n  \
    tally add \"Fix parsing error\" --priority high --tags bug,parser\n  \
    tally done \"Fix parsing error\" --commit abc123f\n  \
    tally list --tags bug\n  \
    tally release v0.2.3 --summary"
)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(long_about = "Initialize tally in the CWD.")]
    Init,
    /// Add a new task
    #[command(long_about = "Add a new task to TODO.md.\n\n\
        Creates a task with optional priority and tags. Use --dry-run to preview \
        the task before adding it.\n\n\
        EXAMPLES:\n  \
        tally add \"Fix parsing error in format.rs\"\n  \
        tally add \"Implement new feature\" --priority high --tags feature,backend\n  \
        tally add \"Update docs\" --dry-run")]
    Add {
        /// Text of the task to add
        description: String,

        /// Priority level for the task
        #[arg(short, long, value_enum, default_value_t = Priority::Medium)]
        priority: Priority,

        /// Comma-separated tags (e.g., bug,frontend)
        #[arg(short, long, value_delimiter = ',')]
        tags: Option<Vec<String>>,

        /// Show what would be added without modifying TODO.md
        #[arg(long, default_value_t = false)]
        dry_run: bool,

        /// Automatically commit TODO.md after adding a task
        #[arg(long, default_value_t = false)]
        auto: bool,
    },

    /// Mark a task as completed
    #[command(long_about = "Mark a task as completed in TODO.md.\n\n\
        Fuzzy-matches the description against existing tasks and marks the match \
        as done. Optionally associate a git commit or version.\n\n\
        EXAMPLES:\n  \
        tally done \"Fix parsing error\"\n  \
        tally done \"parsing error\" --commit abc123f\n  \
        tally done \"Fix bug\" --version v0.2.3 --dry-run")]
    Done {
        /// Text to fuzzy-match against existing tasks
        description: String,

        /// Git commit hash associated with completion
        #[arg(short, long)]
        commit: Option<String>,

        /// Release version (e.g., v0.2.3)
        #[arg(short, long)]
        version: Option<String>,

        /// Show changes without writing to TODO.md
        #[arg(long, default_value_t = false)]
        dry_run: bool,

        /// Automatically commit TODO.md after completing task
        #[arg(long, default_value_t = false)]
        auto: bool,
    },

    /// Display tasks
    #[command(
        long_about = "Display tasks with optional filtering and formatting.\n\n\
        View all tasks, or filter by tags or priority. \
        Output as human-readable text or raw JSON.\n\n\
        EXAMPLES:\n  \
        tally list\n  \
        tally list --tags bug,parser --priority high\n  \
        tally list --json"
    )]
    List {
        /// Filter by tags (comma-separated)
        #[arg(short, long, value_delimiter = ',')]
        tags: Option<Vec<String>>,

        /// Filter by priority level
        #[arg(short, long, value_enum)]
        priority: Option<Priority>,

        /// Output in JSON format
        #[arg(long, default_value_t = false)]
        json: bool,
    },

    /// Assign version to completed tasks
    #[command(
        long_about = "Assign a version to all completed tasks without a version.\n\n\
        Additionally sets the project version in the TODO list itself.\n\n\
        EXAMPLES:\n  \
        tally semver v0.2.3\n  \
        tally semver v1.0.0 --summary\n  \
        tally semver v0.2.4 --dry-run"
    )]
    Semver {
        /// Version string to assign (e.g., v0.2.3)
        version: String,

        /// Show what would be assigned without modifying tasks
        #[arg(long, default_value_t = false)]
        dry_run: bool,

        /// Show number of tasks assigned to this version
        #[arg(long, default_value_t = false)]
        summary: bool,

        /// Automatically commit TODO.md after setting version
        #[arg(long, default_value_t = false)]
        auto: bool,
    },

    /// Release and create a git tag
    #[command(long_about = "Assign a version to completed tasks and create a git tag.\n\n\
        Runs 'tally release' then commits both the todo and command history before creating a git tag. \n
        The tag name will always be prefixed with 'v' if not already.\n\n\
        EXAMPLES:\n  \
        tally tag v0.2.3\n  \
        tally tag v1.0.0 --summary\n  \
        tally tag v0.2.3 --message \"First stable release\"\n  \
        tally tag v0.2.4 --dry-run")]
    Tag {
        /// Version string (e.g., v0.2.3)
        version: String,
        /// Custom tag message (defaults to "Release v0.2.3")
        #[arg(short, long)]
        message: Option<String>,
        /// Show what would happen without making changes
        #[arg(long, default_value_t = false)]
        dry_run: bool,
        /// Show tasks assigned to this version
        #[arg(long, default_value_t = false)]
        summary: bool,
        /// Automatically commit TODO.md after semver
        #[arg(long, default_value_t = false)]
        auto: bool,
    },

    /// Generate a changelog
    #[command(long_about = "Generate a changelog from completed tasks.\n\n\
        Create a changelog for a version range or until the current version.\n\n\
        EXAMPLES:\n  \
        tally changelog\n  \
        tally changelog --from v0.7.2\n  \
        tally changelog --from v0.2.2 --to v0.2.3")]
    Changelog {
        /// Start version/date range
        #[arg(long)]
        from: Option<String>,

        /// End version/date range
        #[arg(long)]
        to: Option<String>,
    },

    /// Remove a task entirely
    #[command(long_about = "Remove a task from TODO.md.\n\n\
        Fuzzy-matches the description. If the task is completed, it will be \
        saved to history.json before removal so it still appears in changelogs.\n\n\
        EXAMPLES:\n  \
        tally remove \"Fix parsing error\"\n  \
        tally remove \"old task\" --dry-run")]
    Remove {
        /// Text to fuzzy-match against existing tasks
        description: String,
        /// Show what would be removed without modifying TODO.md
        #[arg(long, default_value_t = false)]
        dry_run: bool,
        /// Automatically commit TODO.md after removing a task
        #[arg(long, default_value_t = false)]
        auto: bool,
    },

    /// Prune old completed tasks
    #[command(long_about = "Remove completed tasks older than a threshold.\n\n\
        Pruned tasks are saved to history.json before removal so they \
        still appear in changelogs. Days and hours combine if both are given.\n\n\
        EXAMPLES:\n  \
        tally prune                      # default: 30 days\n  \
        tally prune --days 7\n  \
        tally prune --hours 12\n  \
        tally prune --days 1 --hours 12  # 1.5 days\n  \
        tally prune --dry-run")]
    Prune {
        /// Number of days
        #[arg(long)]
        days: Option<u32>,
        /// Number of hours
        #[arg(long)]
        hours: Option<u32>,
        /// Show what would be pruned without modifying TODO.md
        #[arg(long, default_value_t = false)]
        dry_run: bool,
        /// Automatically commit TODO.md after pruning tasks
        #[arg(long, default_value_t = false)]
        auto: bool,
    },

    /// Detect completed tasks from git commits
    #[command(
        long_about = "Scan git commit messages to automatically detect completed tasks.\n\n\
        Uses fuzzy matching to find tasks that may have been completed based on \
        commit messages. Can run automatically or prompt for confirmation.\n\n\
        EXAMPLES:\n  \
        tally scan\n  \
        tally scan --auto\n  \
        tally scan --dry-run"
    )]
    Scan {
        /// Automatically mark matches as done without prompting
        #[arg(long, default_value_t = false)]
        auto: bool,

        /// Show suggested matches without modifying TODO.md
        #[arg(long, default_value_t = false)]
        dry_run: bool,
    },

    /// Manage preferences
    #[command(long_about = "View and modify tally configuration.\n\n\
        Configuration is stored in .tally/config.toml and includes settings like \
        default priority, editor preferences, and changelog templates.\n\n\
        EXAMPLES:\n  \
        tally config set default_priority medium\n  \
        tally config get changelog_template\n  \
        tally config list")]
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    #[command(long_about = "Display a summary dashboard of your project's tasks.\n\n\
        Shows overall progress, open tasks by priority, tag usage, and version \
        statistics. Provides a quick overview without needing to piece together \
        multiple commands.\n\n\
        EXAMPLE:\n  \
        tally status")]
    Status,
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Set a configuration value
    #[command(long_about = "Set one or more configuration values.\n\n\
        Use dot notation for nested keys if needed. Values are stored in \
        .tally/config.toml.\n\n\
        EXAMPLES:\n  \
        tally config set default_priority medium\n  \
        tally config set editor vim")]
    Set {
        /// Configuration key
        key: String,

        /// Configuration value
        value: String,
    },

    /// Get a configuration value
    #[command(long_about = "Retrieve a configuration value.\n\n\
        Displays the current value for the specified configuration key.\n\n\
        EXAMPLES:\n  \
        tally config get default_priority\n  \
        tally config get changelog_template")]
    Get {
        /// Configuration key to retrieve
        key: String,
    },

    /// List all configuration keys and values
    #[command(long_about = "Display all configuration settings.\n\n\
        Shows all current configuration keys and their values.\n\n\
        EXAMPLE:\n  \
        tally config list")]
    List,
}
