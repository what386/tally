use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct AppConfig {
    pub preferences: Preferences,
    pub git: Git,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct Preferences {
    pub auto_commit_todo: bool,
    pub auto_complete_tasks: bool,
    pub editor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Git {
    pub done_prefix: String,
}



impl Default for Git {
    fn default() -> Self {
        Self {
            done_prefix: "done:".to_string(),
        }
    }
}
