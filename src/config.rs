use std::path::PathBuf;

use crate::tree::SortMode;

#[derive(Default)]
pub struct Config {
    pub default_sort: Option<String>,
    pub default_expanded: Option<bool>,
    pub claude_projects_dir: Option<String>,
    pub codex_sessions_dir: Option<String>,
    pub skip_permissions: Option<bool>,
}

impl Config {
    pub fn load() -> Self {
        let paths = config_paths();
        for path in paths {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(table) = content.parse::<toml::Table>() {
                    return Self::from_table(&table);
                }
            }
        }
        Config::default()
    }

    fn from_table(table: &toml::Table) -> Self {
        Config {
            default_sort: table.get("default_sort").and_then(|v| v.as_str()).map(|s| s.to_string()),
            default_expanded: table.get("default_expanded").and_then(|v| v.as_bool()),
            claude_projects_dir: table.get("claude_projects_dir").and_then(|v| v.as_str()).map(|s| s.to_string()),
            codex_sessions_dir: table.get("codex_sessions_dir").and_then(|v| v.as_str()).map(|s| s.to_string()),
            skip_permissions: table.get("skip_permissions").and_then(|v| v.as_bool()),
        }
    }

    pub fn sort_mode(&self) -> SortMode {
        match self.default_sort.as_deref() {
            Some("project") => SortMode::ByProject,
            Some("messages") => SortMode::ByMessageCount,
            _ => SortMode::ByDate,
        }
    }
}

fn config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(config_dir) = std::env::var("XDG_CONFIG_HOME").ok().map(PathBuf::from).or_else(|| {
        std::env::var("HOME").ok().map(|h| PathBuf::from(h).join(".config"))
    }) {
        paths.push(config_dir.join("asm").join("config.toml"));
    }
    if let Ok(home) = std::env::var("HOME") {
        paths.push(PathBuf::from(home).join(".asm.toml"));
    }
    paths
}
