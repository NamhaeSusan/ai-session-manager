use std::path::PathBuf;

use crate::tree::SortMode;

#[derive(Default)]
pub struct Config {
    pub default_sort: Option<String>,
    pub default_expanded: Option<bool>,
    pub claude_projects_dir: Option<String>,
    pub codex_sessions_dir: Option<String>,
    pub skip_permissions: Option<bool>,
    /// The path from which this config was loaded (used for saving back).
    pub loaded_from: Option<PathBuf>,
}

impl Config {
    pub fn load() -> Self {
        let paths = config_paths();
        for path in paths {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(table) = content.parse::<toml::Table>() {
                    let mut cfg = Self::from_table(&table);
                    cfg.loaded_from = Some(path);
                    return cfg;
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
            loaded_from: None,
        }
    }

    pub fn sort_mode(&self) -> SortMode {
        match self.default_sort.as_deref() {
            Some("project") => SortMode::ByProject,
            Some("messages") => SortMode::ByMessageCount,
            _ => SortMode::ByDate,
        }
    }

    pub fn save_path(&self) -> PathBuf {
        if let Some(ref p) = self.loaded_from {
            return p.clone();
        }
        // Default: ~/.config/asm/config.toml
        let config_dir = std::env::var("XDG_CONFIG_HOME")
            .ok()
            .map(PathBuf::from)
            .or_else(|| std::env::var("HOME").ok().map(|h| PathBuf::from(h).join(".config")))
            .unwrap_or_else(|| PathBuf::from("."));
        config_dir.join("asm").join("config.toml")
    }

    pub fn save(&self) {
        let path = self.save_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let mut table = toml::Table::new();
        if let Some(ref s) = self.default_sort {
            table.insert("default_sort".into(), toml::Value::String(s.clone()));
        }
        if let Some(b) = self.default_expanded {
            table.insert("default_expanded".into(), toml::Value::Boolean(b));
        }
        if let Some(ref s) = self.claude_projects_dir {
            table.insert("claude_projects_dir".into(), toml::Value::String(s.clone()));
        }
        if let Some(ref s) = self.codex_sessions_dir {
            table.insert("codex_sessions_dir".into(), toml::Value::String(s.clone()));
        }
        if let Some(b) = self.skip_permissions {
            table.insert("skip_permissions".into(), toml::Value::Boolean(b));
        }
        let _ = std::fs::write(&path, table.to_string());
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
