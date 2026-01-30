use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "dotxt")]
#[command(about = "A task management tool for TODO.md files")]
#[command(
    long_about = "dotxt is a command-line task manager that uses TODO.md as its storage format.\n\n\
    Track tasks, generate changelogs, and integrate with git commits for automatic \
    task completion detection.\n\n\
    EXAMPLES:\n  \
    dotxt add \"Fix parsing error\" --priority high --tags bug,parser\n  \
    dotxt done \"Fix parsing error\" --commit abc123f\n  \
    dotxt list --undone --tags bug\n  \
    dotxt release v0.2.3 --summary"
)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a new task
    #[command(long_about = "Add a new task to TODO.md.\n\n\
        Creates a task with optional priority and tags. Use --dry-run to preview \
        the task before adding it.\n\n\
        EXAMPLES:\n  \
        dotxt add \"Fix parsing error in format.rs\"\n  \
        dotxt add \"Implement new feature\" --priority high --tags feature,backend\n  \
        dotxt add \"Update docs\" --dry-run")]
    Add {
        /// Text of the task to add
        description: String,

        /// Priority level for the task
        #[arg(short, long, value_enum)]
        priority: Option<Priority>,

        /// Comma-separated tags (e.g., bug,frontend)
        #[arg(short, long, value_delimiter = ',')]
        tags: Option<Vec<String>>,

        /// Show what would be added without modifying TODO.md
        #[arg(long, default_value_t = false)]
        dry_run: bool,
    },

    /// Mark a task as completed
    #[command(long_about = "Mark a task as completed in TODO.md.\n\n\
        Fuzzy-matches the description against existing tasks and marks the match \
        as done. Optionally associate a git commit or version.\n\n\
        EXAMPLES:\n  \
        dotxt done \"Fix parsing error\"\n  \
        dotxt done \"parsing error\" --commit abc123f\n  \
        dotxt done \"Fix bug\" --version v0.2.3 --dry-run")]
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
    },

    /// Display tasks
    #[command(long_about = "Display tasks with optional filtering and formatting.\n\n\
        View all tasks, or filter by completion status, tags, or priority. \
        Output as human-readable text or JSON.\n\n\
        EXAMPLES:\n  \
        dotxt list\n  \
        dotxt list --undone\n  \
        dotxt list --tags bug,parser --priority high\n  \
        dotxt list --done --json")]
    List {
        /// Show all tasks (done and undone)
        #[arg(long, conflicts_with_all = ["done", "undone"])]
        all: bool,

        /// Show only completed tasks
        #[arg(long, conflicts_with_all = ["all", "undone"])]
        done: bool,

        /// Show only incomplete tasks
        #[arg(long, conflicts_with_all = ["all", "done"])]
        undone: bool,

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
    #[command(long_about = "Assign a version to all completed tasks without a version.\n\n\
        Useful for release management - tags all completed tasks with the specified \
        version identifier.\n\n\
        EXAMPLES:\n  \
        dotxt release v0.2.3\n  \
        dotxt release v1.0.0 --summary\n  \
        dotxt release v0.2.4 --dry-run")]
    Release {
        /// Version string to assign (e.g., v0.2.3)
        version: String,

        /// Show what would be assigned without modifying tasks
        #[arg(long, default_value_t = false)]
        dry_run: bool,

        /// Show number of tasks assigned to this version
        #[arg(long, default_value_t = false)]
        summary: bool,
    },

    /// Generate a changelog
    #[command(long_about = "Generate a changelog from completed tasks.\n\n\
        Create a changelog for a version range or all versions. Optionally write \
        to CHANGELOG.md or use a custom template.\n\n\
        EXAMPLES:\n  \
        dotxt changelog\n  \
        dotxt changelog --from v0.7.2\n  \
        dotxt changelog --from v0.2.2 --to v0.2.3")]
    Changelog {
        /// Start version/date range
        #[arg(long)]
        from: Option<String>,

        /// End version/date range
        #[arg(long)]
        to: Option<String>,
    },

    /// Detect completed tasks from git commits
    #[command(long_about = "Scan git commit messages to automatically detect completed tasks.\n\n\
        Uses fuzzy matching to find tasks that may have been completed based on \
        commit messages. Can run automatically or prompt for confirmation.\n\n\
        EXAMPLES:\n  \
        dotxt scan\n  \
        dotxt scan --auto\n  \
        dotxt scan --threshold 0.65 --dry-run")]
    Scan {
        /// Automatically mark matches as done without prompting
        #[arg(long, default_value_t = false)]
        auto: bool,

        /// Fuzzy match confidence threshold (0.0-1.0)
        #[arg(long, value_parser = parse_threshold, default_value_t = 0.85)]
        threshold: f64,

        /// Show suggested matches without modifying TODO.md
        #[arg(long, default_value_t = false)]
        dry_run: bool,
    },

    /// Open TODO.md in the default editor
    #[command(long_about = "Open the TODO.md file for manual editing.\n\n\
        Uses the editor specified in config or falls back to $EDITOR environment \
        variable, then common editors like vim, nano, etc.\n\n\
        EXAMPLE:\n  \
        dotxt edit")]
    Edit,

    /// Archive old tasks
    #[command(long_about = "Archive old completed tasks to keep TODO.md clean.\n\n\
        Move old or released tasks to an archive file. Use filters to control \
        which tasks get archived.\n\n\
        EXAMPLES:\n  \
        dotxt prune\n  \
        dotxt prune --older-than 90d\n  \
        dotxt prune --released-only --dry-run")]
    Prune {
        /// Archive tasks older than specified duration (e.g., 30d, 90d)
        #[arg(long)]
        older_than: Option<String>,

        /// Only archive tasks that have a version assigned
        #[arg(long, default_value_t = false)]
        released_only: bool,

        /// Show what would be archived without modifying files
        #[arg(long, default_value_t = false)]
        dry_run: bool,
    },

    /// Manage preferences
    #[command(long_about = "View and modify dotxt configuration.\n\n\
        Configuration is stored in .dotxt/config.toml and includes settings like \
        default priority, editor preferences, and changelog templates.\n\n\
        EXAMPLES:\n  \
        dotxt config set default_priority medium\n  \
        dotxt config get changelog_template\n  \
        dotxt config list")]
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Set a configuration value
    #[command(long_about = "Set one or more configuration values.\n\n\
        Use dot notation for nested keys if needed. Values are stored in \
        .dotxt/config.toml.\n\n\
        EXAMPLES:\n  \
        dotxt config set default_priority medium\n  \
        dotxt config set editor vim")]
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
        dotxt config get default_priority\n  \
        dotxt config get changelog_template")]
    Get {
        /// Configuration key to retrieve
        key: String,
    },

    /// List all configuration keys and values
    #[command(long_about = "Display all configuration settings.\n\n\
        Shows all current configuration keys and their values.\n\n\
        EXAMPLE:\n  \
        dotxt config list")]
    List,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Priority {
    High,
    Medium,
    Low,
}

fn parse_threshold(s: &str) -> Result<f64, String> {
    let val: f64 = s
        .parse()
        .map_err(|_| format!("'{}' is not a valid number", s))?;

    if !(0.0..=1.0).contains(&val) {
        return Err(format!("threshold must be between 0.0 and 1.0, got {}", val));
    }

    Ok(val)
}
