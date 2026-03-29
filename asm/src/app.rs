use std::path::PathBuf;
use std::time::SystemTime;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use asm_core::{self, ConversationLine, ScanMode, SessionEntry};

use crate::ui::parse_iso_timestamp;

use crate::config::Config;
use crate::tree::TreeState;

#[derive(PartialEq)]
pub enum Mode {
    Normal,
    Search,
    Confirm,
    Stats,
    Help,
    BulkCleanup,
    BulkCleanupConfirm,
    Settings,
    SettingsEdit,
}

pub const SETTINGS_COUNT: usize = 5;

pub struct SettingsState {
    pub cursor: usize,
    pub edit_buf: String,
}

pub struct App {
    pub tree: TreeState,
    pub mode: Mode,
    pub search_input: String,
    pub should_quit: bool,
    pub resume_command: Option<String>,
    pub conversation_cache: Vec<ConversationLine>,
    pub preview_scroll: u16,
    pub bulk_days_input: String,
    pub bulk_target_count: usize,
    pub bulk_target_size: u64,
    pub settings: SettingsState,
    pub config: Config,
    claude_dir: Option<PathBuf>,
    codex_dir: Option<PathBuf>,
    skip_permissions: bool,
}

impl App {
    pub fn new(config: Config) -> Self {
        let sort_mode = config.sort_mode();
        let claude_dir = config.claude_projects_dir.as_deref().map(PathBuf::from);
        let codex_dir = config.codex_sessions_dir.as_deref().map(PathBuf::from);
        let sessions = asm_core::scan_all_sessions(
            claude_dir.as_deref(),
            codex_dir.as_deref(),
            ScanMode::Full,
        );
        let tree = TreeState::new(sessions, sort_mode, config.default_expanded);
        let skip_permissions = config.skip_permissions.unwrap_or(true);
        let mut app = App {
            tree,
            mode: Mode::Normal,
            search_input: String::new(),
            should_quit: false,
            resume_command: None,
            conversation_cache: Vec::new(),
            preview_scroll: 0,
            bulk_days_input: "30".to_string(),
            bulk_target_count: 0,
            bulk_target_size: 0,
            settings: SettingsState {
                cursor: 0,
                edit_buf: String::new(),
            },
            config,
            claude_dir,
            codex_dir,
            skip_permissions,
        };
        app.update_preview_cache();
        app
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match self.mode {
            Mode::Normal => self.handle_normal(key),
            Mode::Search => self.handle_search(key),
            Mode::Confirm => self.handle_confirm(key),
            Mode::Stats => self.handle_stats(key),
            Mode::Help => self.handle_help(key),
            Mode::BulkCleanup => self.handle_bulk_cleanup(key),
            Mode::BulkCleanupConfirm => self.handle_bulk_cleanup_confirm(key),
            Mode::Settings => self.handle_settings(key),
            Mode::SettingsEdit => self.handle_settings_edit(key),
        }
    }

    fn handle_normal(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.tree.move_down();
                self.preview_scroll = 0;
                self.update_preview_cache();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.tree.move_up();
                self.preview_scroll = 0;
                self.update_preview_cache();
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                if let Some(node) = self.tree.selected_node() {
                    if node.is_expandable() {
                        self.tree.toggle_expand();
                        self.update_preview_cache();
                    } else if let Some(entry) = self.tree.selected_session() {
                        self.resume_command = Some(resume_cmd_for(entry, self.skip_permissions));
                        self.should_quit = true;
                    }
                }
            }
            KeyCode::Char('d') => {
                if self.tree.selected_session().is_some() {
                    self.mode = Mode::Confirm;
                }
            }
            KeyCode::Char('/') => {
                self.mode = Mode::Search;
                self.search_input.clear();
            }
            KeyCode::Char('s') => {
                self.tree.cycle_sort();
                self.update_preview_cache();
            }
            KeyCode::Char('S') => {
                self.tree.toggle_sort_order();
                self.update_preview_cache();
            }
            KeyCode::Char('i') => {
                self.mode = Mode::Stats;
            }
            KeyCode::Char('D') => {
                self.bulk_days_input = "30".to_string();
                self.compute_bulk_targets();
                self.mode = Mode::BulkCleanup;
            }
            KeyCode::Char('c') => {
                self.settings.cursor = 0;
                self.mode = Mode::Settings;
            }
            KeyCode::Char('?') => {
                self.mode = Mode::Help;
            }
            KeyCode::Char('r') => {
                self.refresh();
            }
            KeyCode::Char('q') | KeyCode::Esc => {
                if !key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.should_quit = true;
                }
            }
            _ if key.modifiers.contains(KeyModifiers::CONTROL) => match key.code {
                KeyCode::Char('d') => {
                    self.preview_scroll = self.preview_scroll.saturating_add(5);
                }
                KeyCode::Char('u') => {
                    self.preview_scroll = self.preview_scroll.saturating_sub(5);
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn handle_search(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.search_input.clear();
                self.tree.set_filter(String::new());
                self.mode = Mode::Normal;
                self.update_preview_cache();
            }
            KeyCode::Enter => {
                self.mode = Mode::Normal;
                self.update_preview_cache();
            }
            KeyCode::Backspace => {
                self.search_input.pop();
                self.tree.set_filter(self.search_input.clone());
                self.update_preview_cache();
            }
            KeyCode::Char(c) => {
                self.search_input.push(c);
                self.tree.set_filter(self.search_input.clone());
                self.update_preview_cache();
            }
            _ => {}
        }
    }

    fn handle_help(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') => {
                self.mode = Mode::Normal;
            }
            _ => {}
        }
    }

    fn handle_stats(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('i') | KeyCode::Char('q') => {
                self.mode = Mode::Normal;
            }
            _ => {}
        }
    }

    fn handle_confirm(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('y') | KeyCode::Enter => {
                if let Some(entry) = self.tree.selected_session() {
                    let _ = asm_core::delete_session(entry);
                }
                self.refresh();
                self.mode = Mode::Normal;
            }
            KeyCode::Char('n') | KeyCode::Esc => {
                self.mode = Mode::Normal;
            }
            _ => {}
        }
    }

    fn handle_bulk_cleanup(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(c) if c.is_ascii_digit() => {
                self.bulk_days_input.push(c);
                self.compute_bulk_targets();
            }
            KeyCode::Backspace => {
                self.bulk_days_input.pop();
                self.compute_bulk_targets();
            }
            KeyCode::Enter => {
                if self.bulk_target_count > 0 {
                    self.mode = Mode::BulkCleanupConfirm;
                }
            }
            KeyCode::Esc => {
                self.mode = Mode::Normal;
            }
            _ => {}
        }
    }

    fn handle_bulk_cleanup_confirm(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('y') => {
                self.execute_bulk_cleanup();
                self.refresh();
                self.mode = Mode::Normal;
            }
            KeyCode::Char('n') | KeyCode::Esc => {
                self.mode = Mode::BulkCleanup;
            }
            _ => {}
        }
    }

    fn compute_bulk_targets(&mut self) {
        let days: u64 = self.bulk_days_input.parse().unwrap_or(0);
        if days == 0 {
            self.bulk_target_count = 0;
            self.bulk_target_size = 0;
            return;
        }

        let now_secs = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let threshold = days * 86400;

        let mut count = 0usize;
        let mut size = 0u64;
        for s in self.tree.all_sessions() {
            let mod_secs = parse_iso_timestamp(&s.modified).unwrap_or(0);
            if mod_secs > 0 && now_secs.saturating_sub(mod_secs) > threshold {
                count += 1;
                size += s.file_size;
            }
        }
        self.bulk_target_count = count;
        self.bulk_target_size = size;
    }

    fn execute_bulk_cleanup(&mut self) {
        let days: u64 = self.bulk_days_input.parse().unwrap_or(0);
        if days == 0 {
            return;
        }

        let now_secs = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let threshold = days * 86400;

        let targets: Vec<SessionEntry> = self
            .tree
            .all_sessions()
            .iter()
            .filter(|s| {
                let mod_secs = parse_iso_timestamp(&s.modified).unwrap_or(0);
                mod_secs > 0 && now_secs.saturating_sub(mod_secs) > threshold
            })
            .cloned()
            .collect();

        for entry in &targets {
            let _ = asm_core::delete_session(entry);
        }
    }

    fn handle_settings(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                if self.settings.cursor + 1 < SETTINGS_COUNT {
                    self.settings.cursor += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.settings.cursor = self.settings.cursor.saturating_sub(1);
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.apply_settings_action();
            }
            KeyCode::Left => {
                if self.settings.cursor == 0 {
                    self.cycle_sort_setting(false);
                }
            }
            KeyCode::Right => {
                if self.settings.cursor == 0 {
                    self.cycle_sort_setting(true);
                }
            }
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('c') => {
                self.mode = Mode::Normal;
            }
            _ => {}
        }
    }

    fn handle_settings_edit(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Settings;
            }
            KeyCode::Enter => {
                let val = self.settings.edit_buf.clone();
                let val_opt = if val.is_empty() { None } else { Some(val) };
                match self.settings.cursor {
                    3 => {
                        self.config.claude_projects_dir = val_opt;
                        self.claude_dir = self
                            .config
                            .claude_projects_dir
                            .as_deref()
                            .map(PathBuf::from);
                    }
                    4 => {
                        self.config.codex_sessions_dir = val_opt;
                        self.codex_dir =
                            self.config.codex_sessions_dir.as_deref().map(PathBuf::from);
                    }
                    _ => {}
                }
                self.config.save();
                self.refresh();
                self.mode = Mode::Settings;
            }
            KeyCode::Backspace => {
                self.settings.edit_buf.pop();
            }
            KeyCode::Char(c) => {
                self.settings.edit_buf.push(c);
            }
            _ => {}
        }
    }

    fn apply_settings_action(&mut self) {
        match self.settings.cursor {
            0 => self.cycle_sort_setting(true),
            1 => {
                let new_val = !self.config.default_expanded.unwrap_or(false);
                self.config.default_expanded = Some(new_val);
                self.tree.set_default_expanded(new_val);
                self.config.save();
            }
            2 => {
                let new_val = !self.config.skip_permissions.unwrap_or(true);
                self.config.skip_permissions = Some(new_val);
                self.skip_permissions = new_val;
                self.config.save();
            }
            3 => {
                self.settings.edit_buf =
                    self.config.claude_projects_dir.clone().unwrap_or_default();
                self.mode = Mode::SettingsEdit;
            }
            4 => {
                self.settings.edit_buf = self.config.codex_sessions_dir.clone().unwrap_or_default();
                self.mode = Mode::SettingsEdit;
            }
            _ => {}
        }
    }

    fn cycle_sort_setting(&mut self, forward: bool) {
        let current = self.config.sort_mode();
        let next = if forward {
            current.next()
        } else {
            current.prev()
        };
        self.config.default_sort = Some(next.label().to_string());
        self.tree.set_sort(next);
        self.config.save();
        self.update_preview_cache();
    }

    fn update_preview_cache(&mut self) {
        self.conversation_cache = match self.tree.selected_session() {
            Some(entry) => asm_core::read_conversation(entry, 50),
            None => Vec::new(),
        };
    }

    fn refresh(&mut self) {
        let sessions = asm_core::scan_all_sessions(
            self.claude_dir.as_deref(),
            self.codex_dir.as_deref(),
            ScanMode::Full,
        );
        self.tree.refresh(sessions);
        self.preview_scroll = 0;
        self.update_preview_cache();
    }
}

fn resume_cmd_for(entry: &SessionEntry, skip_permissions: bool) -> String {
    let resume = match entry.tool.as_str() {
        "Claude Code" => {
            let skip = if skip_permissions {
                " --dangerously-skip-permissions"
            } else {
                ""
            };
            format!("claude --resume {}{}", entry.id, skip)
        }
        "Codex" => {
            let skip = if skip_permissions {
                " --dangerously-bypass-approvals-and-sandbox"
            } else {
                ""
            };
            format!("codex resume {}{}", entry.id, skip)
        }
        _ => return String::new(),
    };
    if entry.project_path.is_empty() {
        resume
    } else {
        format!(
            "cd '{}' && {}",
            entry.project_path.replace('\'', "'\\''"),
            resume
        )
    }
}
