use serde::{Deserialize, Serialize};

use crate::models::common::Priority;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    defaults: Defaults,
    git: Git,
    commands: Prune,

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Defaults {
    priority: Priority,
    tags: Vec<String>,
    editor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Git {
    done_prefix: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prune {
    prune_after_days: i64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            defaults: Defaults::default(),
            git: Git::default(),
            commands: Prune::default(),
        }
    }
}

impl Default for Defaults {
    fn default() -> Self {
        Self {
            priority: Priority::Medium,
            tags: Vec::new(),
            editor: "nvim".to_string(),
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

impl Default for Prune {
    fn default() -> Self {
        Self {
            prune_after_days: 30,
        }
    }
}
