use std::collections::HashMap;

use crate::session::SessionEntry;

#[derive(Clone, Copy, PartialEq)]
pub enum SortMode {
    ByDate,
    ByProject,
    ByMessageCount,
}

impl SortMode {
    pub fn next(self) -> Self {
        match self {
            SortMode::ByDate => SortMode::ByProject,
            SortMode::ByProject => SortMode::ByMessageCount,
            SortMode::ByMessageCount => SortMode::ByDate,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            SortMode::ByDate => "date",
            SortMode::ByProject => "project",
            SortMode::ByMessageCount => "messages",
        }
    }
}

pub struct SessionStats {
    pub total: usize,
    pub by_tool: Vec<(String, usize)>,
    pub by_project: Vec<(String, usize)>,
}

#[derive(Clone)]
pub enum TreeNode {
    Tool {
        name: String,
        session_count: u32,
        expanded: bool,
    },
    Project {
        name: String,
        path: String,
        session_count: u32,
        expanded: bool,
    },
    Session {
        entry: SessionEntry,
    },
}

impl TreeNode {
    #[allow(dead_code)]
    pub fn depth(&self) -> usize {
        match self {
            TreeNode::Tool { .. } => 0,
            TreeNode::Project { .. } => 1,
            TreeNode::Session { .. } => 2,
        }
    }

    pub fn is_expandable(&self) -> bool {
        matches!(self, TreeNode::Tool { .. } | TreeNode::Project { .. })
    }
}

pub struct TreeState {
    nodes: Vec<TreeNode>,
    all_sessions: Vec<SessionEntry>,
    pub cursor: usize,
    pub filter: String,
    pub sort_mode: SortMode,
    pub sort_ascending: bool,
    default_expanded: bool,
    tool_expanded: HashMap<String, bool>,
    project_expanded: HashMap<String, bool>,
}

impl TreeState {
    pub fn new(sessions: Vec<SessionEntry>, sort_mode: SortMode, default_expanded: Option<bool>) -> Self {
        let expanded = default_expanded.unwrap_or(true);
        let mut state = TreeState {
            nodes: Vec::new(),
            all_sessions: sessions,
            cursor: 0,
            filter: String::new(),
            sort_mode,
            sort_ascending: false,
            default_expanded: expanded,
            tool_expanded: HashMap::new(),
            project_expanded: HashMap::new(),
        };
        state.build_tree();
        state
    }

    pub fn cycle_sort(&mut self) {
        self.sort_mode = self.sort_mode.next();
        self.build_tree();
    }

    pub fn toggle_sort_order(&mut self) {
        self.sort_ascending = !self.sort_ascending;
        self.build_tree();
    }

    pub fn stats(&self) -> SessionStats {
        let total = self.all_sessions.len();
        let mut tool_counts: HashMap<String, usize> = HashMap::new();
        let mut proj_counts: HashMap<String, usize> = HashMap::new();
        for s in &self.all_sessions {
            *tool_counts.entry(s.tool.clone()).or_default() += 1;
            let name = if s.project_name.is_empty() { "(unknown)".to_string() } else { s.project_name.clone() };
            *proj_counts.entry(name).or_default() += 1;
        }
        let mut by_tool: Vec<_> = tool_counts.into_iter().collect();
        by_tool.sort_by(|a, b| b.1.cmp(&a.1));
        let mut by_project: Vec<_> = proj_counts.into_iter().collect();
        by_project.sort_by(|a, b| b.1.cmp(&a.1));
        by_project.truncate(10);
        SessionStats { total, by_tool, by_project }
    }

    pub fn build_tree(&mut self) {
        self.nodes.clear();

        let filtered: Vec<&SessionEntry> = if self.filter.is_empty() {
            self.all_sessions.iter().collect()
        } else {
            let f = self.filter.to_lowercase();
            self.all_sessions
                .iter()
                .filter(|s| {
                    s.last_prompt.to_lowercase().contains(&f)
                        || s.project_name.to_lowercase().contains(&f)
                        || s.id.to_lowercase().contains(&f)
                })
                .collect()
        };

        // Group by tool
        let mut tool_map: HashMap<String, Vec<&SessionEntry>> = HashMap::new();
        for s in &filtered {
            tool_map.entry(s.tool.clone()).or_default().push(s);
        }

        let mut tools: Vec<String> = tool_map.keys().cloned().collect();
        tools.sort();

        for tool_name in tools {
            let tool_sessions = &tool_map[&tool_name];
            let tool_total = tool_sessions.len() as u32;
            let tool_expanded = *self.tool_expanded.get(&tool_name).unwrap_or(&self.default_expanded);

            self.nodes.push(TreeNode::Tool {
                name: tool_name.clone(),
                session_count: tool_total,
                expanded: tool_expanded,
            });

            if !tool_expanded {
                continue;
            }

            let asc = self.sort_ascending;

            match self.sort_mode {
                SortMode::ByDate => {
                    let mut sorted: Vec<&SessionEntry> = tool_sessions.clone();
                    sorted.sort_by(|a, b| {
                        if asc { a.modified.cmp(&b.modified) } else { b.modified.cmp(&a.modified) }
                    });
                    for s in sorted {
                        self.nodes.push(TreeNode::Session { entry: s.clone() });
                    }
                }
                SortMode::ByMessageCount => {
                    let mut sorted: Vec<&SessionEntry> = tool_sessions.clone();
                    sorted.sort_by(|a, b| {
                        if asc { a.message_count.cmp(&b.message_count) } else { b.message_count.cmp(&a.message_count) }
                    });
                    for s in sorted {
                        self.nodes.push(TreeNode::Session { entry: s.clone() });
                    }
                }
                SortMode::ByProject => {
                    let mut proj_map: HashMap<String, Vec<&SessionEntry>> = HashMap::new();
                    for s in tool_sessions {
                        proj_map.entry(s.project_path.clone()).or_default().push(s);
                    }

                    let mut proj_list: Vec<(&String, &Vec<&SessionEntry>)> = proj_map.iter().collect();
                    proj_list.sort_by(|a, b| {
                        let name_a = a.1.first().map(|s| s.project_name.to_lowercase()).unwrap_or_default();
                        let name_b = b.1.first().map(|s| s.project_name.to_lowercase()).unwrap_or_default();
                        if asc { name_a.cmp(&name_b) } else { name_b.cmp(&name_a) }
                    });

                    for (proj_path, sess_list) in proj_list {
                        let proj_name = sess_list
                            .first()
                            .map(|s| s.project_name.clone())
                            .unwrap_or_default();
                        let proj_key = format!("{tool_name}:{proj_path}");
                        let proj_expanded = *self.project_expanded.get(&proj_key).unwrap_or(&self.default_expanded);

                        self.nodes.push(TreeNode::Project {
                            name: proj_name,
                            path: proj_path.clone(),
                            session_count: sess_list.len() as u32,
                            expanded: proj_expanded,
                        });

                        if !proj_expanded {
                            continue;
                        }

                        let mut sorted: Vec<&SessionEntry> = sess_list.clone();
                        sorted.sort_by(|a, b| b.modified.cmp(&a.modified));

                        for s in sorted {
                            self.nodes.push(TreeNode::Session { entry: s.clone() });
                        }
                    }
                }
            }
        }

        if self.cursor >= self.nodes.len() {
            self.cursor = self.nodes.len().saturating_sub(1);
        }
    }

    pub fn toggle_expand(&mut self) {
        if let Some(node) = self.nodes.get(self.cursor) {
            match node {
                TreeNode::Tool { name, expanded, .. } => {
                    self.tool_expanded.insert(name.clone(), !expanded);
                }
                TreeNode::Project { path, expanded, .. } => {
                    let tool_name = self.find_parent_tool_name(self.cursor);
                    let key = format!("{tool_name}:{path}");
                    self.project_expanded.insert(key, !expanded);
                }
                TreeNode::Session { .. } => return,
            }
            self.build_tree();
        }
    }

    fn find_parent_tool_name(&self, idx: usize) -> String {
        for i in (0..idx).rev() {
            if let TreeNode::Tool { name, .. } = &self.nodes[i] {
                return name.clone();
            }
        }
        String::new()
    }

    pub fn move_up(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    pub fn move_down(&mut self) {
        if !self.nodes.is_empty() {
            self.cursor = std::cmp::min(self.cursor + 1, self.nodes.len() - 1);
        }
    }

    pub fn selected_node(&self) -> Option<&TreeNode> {
        self.nodes.get(self.cursor)
    }

    pub fn selected_session(&self) -> Option<&SessionEntry> {
        match self.selected_node()? {
            TreeNode::Session { entry } => Some(entry),
            _ => None,
        }
    }

    pub fn set_filter(&mut self, filter: String) {
        self.filter = filter;
        self.cursor = 0;
        self.build_tree();
    }

    pub fn refresh(&mut self, sessions: Vec<SessionEntry>) {
        self.all_sessions = sessions;
        self.build_tree();
    }

    #[allow(dead_code)]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn nodes(&self) -> &[TreeNode] {
        &self.nodes
    }
}
