use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use asm_core::{self, ConversationLine, ScanMode, SessionEntry};

use crate::config::Config;
use crate::tree::TreeState;

#[derive(PartialEq)]
pub enum Mode {
    Normal,
    Search,
    Confirm,
    Stats,
    Help,
}

pub struct App {
    pub tree: TreeState,
    pub mode: Mode,
    pub search_input: String,
    pub should_quit: bool,
    pub resume_command: Option<String>,
    pub conversation_cache: Vec<ConversationLine>,
    pub preview_scroll: u16,
    claude_dir: Option<PathBuf>,
    codex_dir: Option<PathBuf>,
    skip_permissions: bool,
}

impl App {
    pub fn new(config: Config) -> Self {
        let sort_mode = config.sort_mode();
        let claude_dir = config.claude_projects_dir.map(PathBuf::from);
        let codex_dir = config.codex_sessions_dir.map(PathBuf::from);
        let sessions = asm_core::scan_all_sessions(
            claude_dir.as_deref(),
            codex_dir.as_deref(),
            ScanMode::Full,
        );
        let tree = TreeState::new(sessions, sort_mode, config.default_expanded);
        let mut app = App {
            tree,
            mode: Mode::Normal,
            search_input: String::new(),
            should_quit: false,
            resume_command: None,
            conversation_cache: Vec::new(),
            preview_scroll: 0,
            claude_dir,
            codex_dir,
            skip_permissions: config.skip_permissions.unwrap_or(true),
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
            let skip = if skip_permissions { " --dangerously-skip-permissions" } else { "" };
            format!("claude --resume {}{}", entry.id, skip)
        }
        "Codex" => {
            let skip = if skip_permissions { " --dangerously-bypass-approvals-and-sandbox" } else { "" };
            format!("codex resume {}{}", entry.id, skip)
        }
        _ => return String::new(),
    };
    if entry.project_path.is_empty() {
        resume
    } else {
        format!("cd '{}' && {}", entry.project_path.replace('\'', "'\\''"), resume)
    }
}
