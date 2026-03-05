use std::time::SystemTime;

use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::{App, Mode};
use crate::tree::{SortMode, TreeNode, SessionStats};

pub fn draw(frame: &mut Frame, app: &App) {
    let outer = Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).split(frame.area());

    let main_area = outer[0];
    let status_area = outer[1];

    let chunks = Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(main_area);

    draw_tree(frame, app, chunks[0]);
    draw_preview(frame, app, chunks[1]);
    draw_status_bar(frame, app, status_area);

    match app.mode {
        Mode::Confirm => draw_confirm_popup(frame),
        Mode::Stats => draw_stats_popup(frame, &app.tree.stats()),
        Mode::Help => draw_help_popup(frame),
        _ => {}
    }
}

fn draw_tree(frame: &mut Frame, app: &App, area: Rect) {
    let title = if app.mode == Mode::Search {
        format!("Sessions [/ {}]", app.search_input)
    } else {
        let arrow = if app.tree.sort_ascending { "\u{25b2}" } else { "\u{25bc}" };
        format!("Sessions [sort: {} {}]", app.tree.sort_mode.label(), arrow)
    };

    let block = Block::default().title(title).borders(Borders::ALL);

    let items: Vec<ListItem> = app
        .tree
        .nodes()
        .iter()
        .map(|node| {
            let line = match node {
                TreeNode::Tool {
                    name,
                    session_count,
                    expanded,
                } => {
                    let arrow = if *expanded { "\u{25be}" } else { "\u{25b8}" };
                    Line::from(Span::styled(
                        format!("{arrow} {name} ({session_count})"),
                        Style::default().add_modifier(Modifier::BOLD),
                    ))
                }
                TreeNode::Project {
                    name,
                    session_count,
                    expanded,
                    ..
                } => {
                    let arrow = if *expanded { "\u{25be}" } else { "\u{25b8}" };
                    Line::from(Span::styled(
                        format!("  {arrow} {name} ({session_count})"),
                        Style::default().fg(Color::Yellow),
                    ))
                }
                TreeNode::Session { entry } => {
                    let is_flat = app.tree.sort_mode != SortMode::ByProject;
                    let indent = if is_flat { "  " } else { "    " };
                    let prompt = truncate_display(&entry.last_prompt, if is_flat { 30 } else { 40 });
                    let rel = relative_time(&entry.modified);
                    let mut spans = vec![
                        Span::raw(indent),
                        Span::styled("\u{25cf} ", Style::default().fg(Color::Cyan)),
                    ];
                    if is_flat && !entry.project_name.is_empty() {
                        spans.push(Span::styled(
                            format!("[{}] ", entry.project_name),
                            Style::default().fg(Color::Yellow),
                        ));
                    }
                    spans.push(Span::raw(prompt));
                    spans.push(Span::styled(
                        format!("  {rel}  {}msg", entry.message_count),
                        Style::default().fg(Color::DarkGray),
                    ));
                    Line::from(spans)
                }
            };
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White))
        .highlight_symbol("");

    frame.render_stateful_widget(
        list,
        area,
        &mut ratatui::widgets::ListState::default().with_selected(Some(app.tree.cursor)),
    );
}

fn draw_preview(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().title("Preview").borders(Borders::ALL);

    let text = match app.tree.selected_session() {
        Some(entry) => {
            let branch = entry.git_branch.as_deref().unwrap_or("-");
            let mut lines = vec![
                Line::from(format!("Project:  {}", entry.project_name)),
                Line::from(format!("Path:     {}", entry.project_path)),
                Line::from(format!("Branch:   {branch}")),
                Line::from(format!("Created:  {}", entry.created)),
                Line::from(format!("Messages: {}", entry.message_count)),
                Line::from(""),
                Line::from(Span::styled(
                    "\u{2500}\u{2500} Last Prompt \u{2500}\u{2500}",
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Line::from(entry.last_prompt.clone()),
                Line::from(""),
                Line::from(Span::styled(
                    "\u{2500}\u{2500} Recent Conversation \u{2500}\u{2500}",
                    Style::default().add_modifier(Modifier::BOLD),
                )),
            ];

            for cl in &app.conversation_cache {
                let style = match cl.role.as_str() {
                    "user" => Style::default().fg(Color::Cyan),
                    "assistant" => Style::default().fg(Color::Green),
                    _ => Style::default(),
                };
                lines.push(Line::from(Span::styled(
                    format!("[{}] {}", cl.role, cl.text),
                    style,
                )));
            }

            lines
        }
        None => vec![Line::from("Select a session to preview")],
    };

    let paragraph = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((app.preview_scroll, 0));

    frame.render_widget(paragraph, area);
}

fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let text = match app.mode {
        Mode::Normal => "? help  / search  s sort  i stats  d delete  Enter resume  r refresh  q quit",
        Mode::Help => "Keybindings (Esc/?/q to close)",
        Mode::Search => "Type to search... (Esc cancel, Enter confirm)",
        Mode::Confirm => "Delete session? (y/Enter confirm, n/Esc cancel)",
        Mode::Stats => "Session Statistics (Esc/i/q to close)",
    };

    let paragraph = Paragraph::new(Line::from(Span::styled(
        text,
        Style::default().fg(Color::DarkGray),
    )));

    frame.render_widget(paragraph, area);
}

fn draw_confirm_popup(frame: &mut Frame) {
    let area = frame.area();
    let w = 40u16.min(area.width);
    let h = 5u16.min(area.height);
    let x = (area.width.saturating_sub(w)) / 2;
    let y = (area.height.saturating_sub(h)) / 2;
    let popup_area = Rect::new(x, y, w, h);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title("Confirm")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Red));

    let text = Paragraph::new("Delete this session? (y/n)")
        .block(block)
        .style(Style::default().fg(Color::White));

    frame.render_widget(text, popup_area);
}

fn draw_stats_popup(frame: &mut Frame, stats: &SessionStats) {
    let area = frame.area();
    let w = 50u16.min(area.width);
    let h = 20u16.min(area.height);
    let x = (area.width.saturating_sub(w)) / 2;
    let y = (area.height.saturating_sub(h)) / 2;
    let popup_area = Rect::new(x, y, w, h);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(format!("Session Statistics (total: {})", stats.total))
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));

    let mut lines = vec![
        Line::from(Span::styled(
            "By Tool",
            Style::default().add_modifier(Modifier::BOLD),
        )),
    ];
    for (name, count) in &stats.by_tool {
        lines.push(Line::from(format!("  {name}: {count}")));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Top Projects",
        Style::default().add_modifier(Modifier::BOLD),
    )));
    for (name, count) in &stats.by_project {
        lines.push(Line::from(format!("  {name}: {count}")));
    }

    let paragraph = Paragraph::new(lines)
        .block(block)
        .style(Style::default().fg(Color::White));

    frame.render_widget(paragraph, popup_area);
}

fn draw_help_popup(frame: &mut Frame) {
    let area = frame.area();
    let w = 50u16.min(area.width);
    let h = 19u16.min(area.height);
    let x = (area.width.saturating_sub(w)) / 2;
    let y = (area.height.saturating_sub(h)) / 2;
    let popup_area = Rect::new(x, y, w, h);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title("Keybindings")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));

    let keys: &[(&str, &str)] = &[
        ("j / k", "Move down / up"),
        ("Enter", "Resume session or toggle folder"),
        ("Space", "Toggle folder expand/collapse"),
        ("d", "Delete session (with confirmation)"),
        ("/", "Search / filter sessions"),
        ("s", "Cycle sort mode (date/project/messages)"),
        ("S", "Toggle sort order (asc/desc)"),
        ("i", "Show session statistics"),
        ("r", "Refresh session list"),
        ("Ctrl+d", "Scroll preview down"),
        ("Ctrl+u", "Scroll preview up"),
        ("?", "Show this help"),
        ("q / Esc", "Quit"),
    ];

    let lines: Vec<Line> = keys
        .iter()
        .map(|(key, desc)| {
            Line::from(vec![
                Span::styled(format!("  {key:<10}"), Style::default().fg(Color::Yellow)),
                Span::raw(*desc),
            ])
        })
        .collect();

    let paragraph = Paragraph::new(lines)
        .block(block)
        .style(Style::default().fg(Color::White));

    frame.render_widget(paragraph, popup_area);
}

fn truncate_display(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max).collect();
        format!("{truncated}...")
    }
}

fn relative_time(modified: &str) -> String {
    let now_secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let mod_secs = parse_iso_timestamp(modified).unwrap_or(0);
    if mod_secs == 0 {
        return String::new();
    }

    let diff = now_secs.saturating_sub(mod_secs);

    if diff < 60 {
        "just now".to_string()
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else {
        format!("{}d ago", diff / 86400)
    }
}

fn parse_iso_timestamp(s: &str) -> Option<u64> {
    if s.len() < 19 {
        return None;
    }
    let year: u64 = s.get(0..4)?.parse().ok()?;
    let month: u64 = s.get(5..7)?.parse().ok()?;
    let day: u64 = s.get(8..10)?.parse().ok()?;
    let hour: u64 = s.get(11..13)?.parse().ok()?;
    let min: u64 = s.get(14..16)?.parse().ok()?;
    let sec: u64 = s.get(17..19)?.parse().ok()?;

    let days = date_to_days(year, month, day)?;
    Some(days * 86400 + hour * 3600 + min * 60 + sec)
}

fn date_to_days(year: u64, month: u64, day: u64) -> Option<u64> {
    let (y, m) = if month <= 2 {
        (year - 1, month + 9)
    } else {
        (year, month - 3)
    };
    let era = y / 400;
    let yoe = y - era * 400;
    let doy = (153 * m + 2) / 5 + day - 1;
    let doe = 365 * yoe + yoe / 4 - yoe / 100 + doy;
    let days = era * 146097 + doe;
    days.checked_sub(719468)
}
