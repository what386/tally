use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub preferences: Preferences,
    pub git: Git,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preferences {
    pub auto_commit_todo: bool,
    pub auto_complete_tasks: bool
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Git {
    pub done_prefix: String
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            preferences: Preferences::default(),
            git: Git::default(),
        }
    }
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            auto_commit_todo: false,
            auto_complete_tasks: true
        }
    }
}

impl Default for Git {
    fn default() -> Self {
        Self {
            done_prefix: "done:".to_string(),
        }
    }
}
