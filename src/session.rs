use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct SessionEntry {
    pub id: String,
    pub tool: String,
    pub project_name: String,
    pub project_path: String,
    pub last_prompt: String,
    pub message_count: u32,
    pub created: String,
    pub modified: String,
    pub git_branch: Option<String>,
    pub file_path: PathBuf,
}

pub struct ConversationLine {
    pub role: String,
    pub text: String,
}

pub fn scan_all_sessions(claude_dir: Option<&str>, codex_dir: Option<&str>) -> Vec<SessionEntry> {
    let home = match std::env::var("HOME") {
        Ok(h) => PathBuf::from(h),
        Err(_) => return vec![],
    };

    let mut sessions = Vec::new();

    let claude_path = claude_dir
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".claude").join("projects"));
    scan_claude_sessions_dir(&claude_path, &mut sessions);

    let codex_path = codex_dir
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".codex").join("sessions"));
    if codex_path.exists() {
        scan_codex_dir(&codex_path, &mut sessions);
    }

    sessions
}

pub fn session_file_path(entry: &SessionEntry) -> Option<PathBuf> {
    if entry.file_path.exists() {
        Some(entry.file_path.clone())
    } else {
        None
    }
}

pub fn delete_session(entry: &SessionEntry) -> io::Result<()> {
    if !entry.file_path.exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "Session file not found"));
    }

    std::fs::remove_file(&entry.file_path)?;

    // Clean up associated directory for Claude Code sessions
    if entry.tool == "Claude Code" {
        if let Some(parent) = entry.file_path.parent() {
            let dir_path = parent.join(&entry.id);
            if dir_path.is_dir() {
                if let (Ok(canonical_dir), Ok(canonical_parent)) =
                    (dir_path.canonicalize(), parent.canonicalize())
                {
                    if canonical_dir.starts_with(&canonical_parent) {
                        std::fs::remove_dir_all(&dir_path)?;
                    }
                }
            }
            if !parent.is_symlink() && is_dir_empty(parent) {
                let _ = std::fs::remove_dir(parent);
            }
        }
    }

    // Clean up empty parent directories for Codex sessions
    if entry.tool == "Codex" {
        let home = std::env::var("HOME")
            .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "HOME not set"))?;
        let sessions_dir = PathBuf::from(home).join(".codex").join("sessions");
        if let Ok(sessions_dir_canonical) = sessions_dir.canonicalize() {
            let mut parent = entry.file_path.parent();
            while let Some(p) = parent {
                let p_canonical = match p.canonicalize() {
                    Ok(c) => c,
                    Err(_) => break,
                };
                if p_canonical == sessions_dir_canonical
                    || !p_canonical.starts_with(&sessions_dir_canonical)
                {
                    break;
                }
                if is_dir_empty(p) {
                    if std::fs::remove_dir(p).is_err() {
                        break;
                    }
                    parent = p.parent();
                } else {
                    break;
                }
            }
        }
    }

    Ok(())
}

pub fn read_conversation(entry: &SessionEntry, max_lines: usize) -> Vec<ConversationLine> {
    let path = match session_file_path(entry) {
        Some(p) => p,
        None => return vec![],
    };
    let file = match std::fs::File::open(&path) {
        Ok(f) => f,
        Err(_) => return vec![],
    };
    let reader = BufReader::new(file);
    let mut lines = Vec::new();

    match entry.tool.as_str() {
        "Claude Code" => read_claude_conversation(reader, &mut lines, max_lines),
        "Codex" => read_codex_conversation(reader, &mut lines, max_lines),
        _ => {}
    }

    lines
}

fn read_claude_conversation(
    reader: BufReader<std::fs::File>,
    lines: &mut Vec<ConversationLine>,
    max_lines: usize,
) {
    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        let v: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let msg_type = match v.get("type").and_then(|t| t.as_str()) {
            Some(t) => t,
            None => continue,
        };

        let role = match msg_type {
            "user" => "user",
            "assistant" => "assistant",
            _ => continue,
        };

        let content = &v["message"]["content"];
        let text = extract_text_content(content);
        let trimmed = text.trim();
        if trimmed.is_empty() || trimmed.starts_with('<') {
            continue;
        }

        lines.push(ConversationLine {
            role: role.to_string(),
            text: truncate_str(trimmed, 500),
        });
        if lines.len() > max_lines {
            lines.remove(0);
        }
    }
}

fn read_codex_conversation(
    reader: BufReader<std::fs::File>,
    lines: &mut Vec<ConversationLine>,
    max_lines: usize,
) {
    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        let v: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        if v.get("type").and_then(|t| t.as_str()) != Some("event_msg") {
            continue;
        }

        let payload = match v.get("payload") {
            Some(p) => p,
            None => continue,
        };
        let payload_type = payload.get("type").and_then(|t| t.as_str()).unwrap_or("");

        let role = match payload_type {
            "user_message" => "user",
            "assistant_message" => "assistant",
            _ => continue,
        };

        let text = payload
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("");
        let trimmed = text.trim();
        if trimmed.is_empty() {
            continue;
        }

        lines.push(ConversationLine {
            role: role.to_string(),
            text: truncate_str(trimmed, 500),
        });
        if lines.len() > max_lines {
            lines.remove(0);
        }
    }
}

fn extract_text_content(content: &serde_json::Value) -> String {
    if let Some(s) = content.as_str() {
        return s.to_string();
    }
    if let Some(arr) = content.as_array() {
        // Find the last text block that looks like actual user input
        let mut best = String::new();
        for item in arr {
            if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                let trimmed = text.trim();
                if !trimmed.is_empty() && !is_system_text(trimmed) {
                    best = trimmed.to_string();
                }
            }
        }
        if !best.is_empty() {
            return best;
        }
        // Fallback: return first text block if nothing else
        return arr
            .iter()
            .find_map(|item| item.get("text").and_then(|t| t.as_str()))
            .unwrap_or("")
            .to_string();
    }
    String::new()
}

fn is_system_text(s: &str) -> bool {
    s.starts_with("<system-reminder>")
        || s.starts_with("<local-command-caveat>")
        || s.starts_with("<local-command-stdout>")
        || s.starts_with("<teammate-message")
        || s.starts_with("<available-deferred-tools>")
        || s.starts_with("[Request interrupted")
}

// --- Claude sessions ---

fn scan_claude_sessions_dir(projects_dir: &Path, sessions: &mut Vec<SessionEntry>) {
    let read_dir = match std::fs::read_dir(projects_dir) {
        Ok(d) => d,
        Err(_) => return,
    };

    for project_entry in read_dir.flatten() {
        let project_path = project_entry.path();
        if !project_path.is_dir() {
            continue;
        }

        let files = match std::fs::read_dir(&project_path) {
            Ok(d) => d,
            Err(_) => continue,
        };

        for file_entry in files.flatten() {
            let path = file_entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
                continue;
            }
            if let Some(entry) = parse_claude_session(&path) {
                sessions.push(entry);
            }
        }
    }
}

fn parse_claude_session(path: &Path) -> Option<SessionEntry> {
    let session_id = path.file_stem()?.to_str()?.to_string();

    let file = std::fs::File::open(path).ok()?;
    let reader = BufReader::new(file);

    let mut cwd = String::new();
    let mut git_branch: Option<String> = None;
    let mut timestamp = String::new();
    let mut last_prompt = String::new();
    let mut found_metadata = false;
    let mut msg_count: u32 = 0;

    for (i, line) in reader.lines().enumerate() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        let v: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        if !found_metadata && i < 50 {
            if let Some(sid) = v.get("sessionId").and_then(|v| v.as_str()) {
                if !sid.is_empty() {
                    found_metadata = true;

                    if v.get("isSidechain")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
                    {
                        return None;
                    }

                    cwd = v
                        .get("cwd")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    git_branch = v
                        .get("gitBranch")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    timestamp = v
                        .get("timestamp")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                }
            }
        }

        let msg_type = v.get("type").and_then(|t| t.as_str());
        match msg_type {
            Some("user") => {
                msg_count += 1;
                let content = &v["message"]["content"];
                let text = extract_text_content(content);
                let trimmed = text.trim();
                if !trimmed.is_empty() && !trimmed.starts_with('<') && !trimmed.starts_with("[Request interrupted") {
                    last_prompt = trimmed.to_string();
                }
            }
            Some("assistant") => {
                msg_count += 1;
            }
            _ => {}
        }
    }

    if !found_metadata || last_prompt.is_empty() {
        return None;
    }

    let last_prompt = truncate_str(&last_prompt, 200);
    let project_name = PathBuf::from(&cwd)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let modified = file_modified_time(path);

    Some(SessionEntry {
        id: session_id,
        tool: "Claude Code".to_string(),
        project_name,
        project_path: cwd,
        last_prompt,
        message_count: msg_count,
        created: timestamp,
        modified,
        git_branch,
        file_path: path.to_path_buf(),
    })
}

// --- Codex sessions ---

fn scan_codex_dir(dir: &Path, sessions: &mut Vec<SessionEntry>) {
    let read_dir = match std::fs::read_dir(dir) {
        Ok(d) => d,
        Err(_) => return,
    };

    for entry in read_dir.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_codex_dir(&path, sessions);
        } else if path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
            if let Some(entry) = parse_codex_session(&path) {
                sessions.push(entry);
            }
        }
    }
}

fn parse_codex_session(path: &Path) -> Option<SessionEntry> {
    let file = std::fs::File::open(path).ok()?;
    let reader = BufReader::new(file);

    let mut session_id = String::new();
    let mut cwd = String::new();
    let mut git_branch: Option<String> = None;
    let mut created = String::new();
    let mut last_prompt = String::new();
    let mut found_meta = false;
    let mut msg_count: u32 = 0;

    for (i, line) in reader.lines().enumerate() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        let v: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let msg_type = v.get("type").and_then(|t| t.as_str()).unwrap_or("");

        if msg_type == "session_meta" && !found_meta && i < 50 {
            found_meta = true;
            if let Some(payload) = v.get("payload") {
                session_id = payload
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                cwd = payload
                    .get("cwd")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                created = payload
                    .get("timestamp")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                git_branch = payload
                    .get("git")
                    .and_then(|g| g.get("branch"))
                    .and_then(|b| b.as_str())
                    .map(|s| s.to_string());
            }
        }

        if msg_type == "event_msg" {
            if let Some(payload) = v.get("payload") {
                let payload_type = payload.get("type").and_then(|t| t.as_str()).unwrap_or("");
                match payload_type {
                    "user_message" => {
                        msg_count += 1;
                        let text = payload
                            .get("message")
                            .and_then(|m| m.as_str())
                            .unwrap_or("");
                        if !text.trim().is_empty() {
                            last_prompt = text.trim().to_string();
                        }
                    }
                    "assistant_message" => {
                        msg_count += 1;
                    }
                    _ => {}
                }
            }
        }
    }

    if !found_meta {
        return None;
    }

    let last_prompt = truncate_str(&last_prompt, 200);
    let project_name = PathBuf::from(&cwd)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let modified = file_modified_time(path);

    Some(SessionEntry {
        id: session_id,
        tool: "Codex".to_string(),
        project_name,
        project_path: cwd,
        last_prompt,
        message_count: msg_count,
        created,
        modified,
        git_branch,
        file_path: path.to_path_buf(),
    })
}

// --- Helpers ---

fn truncate_str(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_chars).collect();
        format!("{truncated}...")
    }
}

fn file_modified_time(path: &Path) -> String {
    std::fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| {
            t.duration_since(SystemTime::UNIX_EPOCH)
                .ok()
                .map(|d| time_from_epoch_secs(d.as_secs()))
        })
        .unwrap_or_default()
}

fn time_from_epoch_secs(secs: u64) -> String {
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;
    let (year, month, day) = days_to_date(days);
    format!("{year:04}-{month:02}-{day:02}T{hours:02}:{minutes:02}:{seconds:02}Z")
}

fn days_to_date(days: u64) -> (u64, u64, u64) {
    let z = days + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

fn is_dir_empty(path: &Path) -> bool {
    std::fs::read_dir(path)
        .map(|mut d| d.next().is_none())
        .unwrap_or(false)
}

