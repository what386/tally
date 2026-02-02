use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub defaults: Defaults,
    pub git: Git,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Defaults {
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Git {
    pub done_prefix: String
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            defaults: Defaults::default(),
            git: Git::default(),
        }
    }
}

impl Default for Defaults {
    fn default() -> Self {
        Self {
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
