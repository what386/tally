use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub preferences: Preferences,
    #[serde(default)]
    pub git: Git,
    #[serde(default)]
    pub auto_commit: AutoCommit,
    #[serde(default)]
    pub scan: Scan,
    #[serde(default)]
    pub matching: Matching,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Preferences {
    #[serde(default)]
    pub auto_commit_todo: bool,
    #[serde(default)]
    pub auto_complete_tasks: bool,
    #[serde(default)]
    pub editor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Git {
    #[serde(default = "default_done_prefix")]
    pub done_prefix: String,
    #[serde(default)]
    pub track_created_files: TrackCreatedFiles,
}

impl Default for Git {
    fn default() -> Self {
        Self {
            done_prefix: default_done_prefix(),
            track_created_files: TrackCreatedFiles::Prompt,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TrackCreatedFiles {
    Always,
    Never,
    #[default]
    Prompt,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AutoCommit {
    #[serde(default)]
    pub add: bool,
    #[serde(default)]
    pub done: bool,
    #[serde(default)]
    pub remove: bool,
    #[serde(default)]
    pub semver: bool,
    #[serde(default)]
    pub yank: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scan {
    #[serde(default = "default_git_log_limit")]
    pub git_log_limit: usize,
    #[serde(default = "default_todo_markers")]
    pub todo_markers: Vec<String>,
    #[serde(default = "default_done_markers")]
    pub done_markers: Vec<String>,
}

impl Default for Scan {
    fn default() -> Self {
        Self {
            git_log_limit: default_git_log_limit(),
            todo_markers: default_todo_markers(),
            done_markers: default_done_markers(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Matching {
    #[serde(default = "default_min_score")]
    pub task_min_score: f64,
    #[serde(default = "default_min_score")]
    pub source_done_min_score: f64,
    #[serde(default = "default_min_score")]
    pub released_min_score: f64,
}

impl Default for Matching {
    fn default() -> Self {
        Self {
            task_min_score: default_min_score(),
            source_done_min_score: default_min_score(),
            released_min_score: default_min_score(),
        }
    }
}

impl AppConfig {
    pub fn auto_commit_add(&self) -> bool {
        self.preferences.auto_commit_todo || self.auto_commit.add
    }

    pub fn auto_commit_done(&self) -> bool {
        self.preferences.auto_commit_todo || self.auto_commit.done
    }

    pub fn auto_commit_remove(&self) -> bool {
        self.preferences.auto_commit_todo || self.auto_commit.remove
    }

    pub fn auto_commit_semver(&self) -> bool {
        self.preferences.auto_commit_todo || self.auto_commit.semver
    }

    pub fn auto_commit_yank(&self) -> bool {
        self.preferences.auto_commit_todo || self.auto_commit.yank
    }
}

fn default_done_prefix() -> String {
    "done:".to_string()
}

fn default_git_log_limit() -> usize {
    50
}

fn default_todo_markers() -> Vec<String> {
    vec!["TODO:".to_string()]
}

fn default_done_markers() -> Vec<String> {
    vec!["DONE:".to_string()]
}

fn default_min_score() -> f64 {
    50.0
}
