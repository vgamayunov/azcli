use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};

use super::app::{App, ListState, View};

pub fn render(f: &mut Frame, app: &App) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .split(area);

    render_header(f, chunks[0], app);
    render_body(f, chunks[1], app);
    render_footer(f, chunks[2], app);

    if matches!(app.current_view(), View::AccountPicker) {
        render_account_picker(f, area, app);
    }

    if app.help_visible {
        render_help(f, area);
    }
}

fn render_header(f: &mut Frame, area: Rect, app: &App) {
    let sub_label = active_account_label(app);
    let view_label = match app.current_view() {
        View::ResourceGroups => "Resource Groups".to_string(),
        View::ResourcesInGroup { rg } => format!("Resources / {rg}"),
        View::AccountPicker => "Switch Subscription".to_string(),
    };

    let line1 = Line::from(vec![
        Span::styled(" azcli tui ", Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled(view_label, Style::default().add_modifier(Modifier::BOLD)),
    ]);
    let line2 = Line::from(vec![
        Span::styled("account: ", Style::default().fg(Color::DarkGray)),
        Span::raw(sub_label),
    ]);

    let p = Paragraph::new(vec![line1, line2])
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(p, area);
}

fn render_body(f: &mut Frame, area: Rect, app: &App) {
    match app.current_view() {
        View::ResourceGroups => render_rg_list(f, area, &app.rg_list),
        View::ResourcesInGroup { .. } => render_resource_list(f, area, &app.resource_list),
        View::AccountPicker => {}
    }
}

fn render_rg_list(f: &mut Frame, area: Rect, list: &ListState) {
    if let Some(msg) = status_line(list) {
        let p = Paragraph::new(msg).block(Block::default().borders(Borders::ALL).title("Resource Groups"));
        f.render_widget(p, area);
        return;
    }

    let inner_w = area.width.saturating_sub(2) as usize;
    let loc_w = 18usize;
    let gap = 2usize;
    let name_w = inner_w.saturating_sub(loc_w + gap).max(8);

    let visible = visible_window(list, area.height.saturating_sub(2) as usize);
    let items: Vec<ListItem> = visible.iter().map(|(idx, v)| {
        let name = v.get("name").and_then(|s| s.as_str()).unwrap_or("?");
        let loc = v.get("location").and_then(|s| s.as_str()).unwrap_or("");
        let line = format!("{}  {}", pad(&fit(name, name_w), name_w), fit(loc, loc_w));
        let style = if *idx == list.cursor {
            Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        ListItem::new(line).style(style)
    }).collect();

    let title = format!("Resource Groups ({})", list.items.len());
    let widget = List::new(items).block(Block::default().borders(Borders::ALL).title(title));
    f.render_widget(widget, area);
}

fn render_resource_list(f: &mut Frame, area: Rect, list: &ListState) {
    if let Some(msg) = status_line(list) {
        let p = Paragraph::new(msg).block(Block::default().borders(Borders::ALL).title("Resources"));
        f.render_widget(p, area);
        return;
    }

    let inner_w = area.width.saturating_sub(2) as usize;
    let loc_w = 14usize;
    let type_w = 36usize;
    let gap = 2usize;
    let name_w = inner_w.saturating_sub(type_w + loc_w + gap * 2).max(8);

    let visible = visible_window(list, area.height.saturating_sub(2) as usize);
    let items: Vec<ListItem> = visible.iter().map(|(idx, v)| {
        let name = v.get("name").and_then(|s| s.as_str()).unwrap_or("?");
        let ty = v.get("type").and_then(|s| s.as_str()).unwrap_or("");
        let loc = v.get("location").and_then(|s| s.as_str()).unwrap_or("");
        let line = format!(
            "{}  {}  {}",
            pad(&fit(name, name_w), name_w),
            pad(&fit(ty, type_w), type_w),
            fit(loc, loc_w),
        );
        let style = if *idx == list.cursor {
            Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        ListItem::new(line).style(style)
    }).collect();

    let title = format!("Resources ({})", list.items.len());
    let widget = List::new(items).block(Block::default().borders(Borders::ALL).title(title));
    f.render_widget(widget, area);
}

fn render_footer(f: &mut Frame, area: Rect, app: &App) {
    let hints = match app.current_view() {
        View::ResourceGroups => "↑↓/jk move  Enter drill  r refresh  s switch-sub  ? help  q quit",
        View::ResourcesInGroup { .. } => "↑↓/jk move  Esc/h back  r refresh  s switch-sub  ? help  q quit",
        View::AccountPicker => "↑↓/jk move  Enter select  r refresh  Esc cancel",
    };
    let mut lines = vec![Line::from(Span::styled(hints, Style::default().fg(Color::DarkGray)))];
    if !app.status.is_empty() {
        lines.push(Line::from(Span::styled(app.status.clone(), Style::default().fg(Color::Yellow))));
    }
    let p = Paragraph::new(lines).block(Block::default().borders(Borders::TOP));
    f.render_widget(p, area);
}

fn render_account_picker(f: &mut Frame, area: Rect, app: &App) {
    let w = area.width.saturating_sub(8).max(60).min(120);
    let h = area.height.saturating_sub(4).max(8);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let rect = Rect { x, y, width: w, height: h };

    f.render_widget(Clear, rect);

    let list = &app.subs_list;
    if let Some(msg) = status_line(list) {
        let p = Paragraph::new(msg)
            .block(Block::default().borders(Borders::ALL).title(" Switch Subscription "));
        f.render_widget(p, rect);
        return;
    }

    let capacity = rect.height.saturating_sub(2) as usize;
    let visible = visible_window(list, capacity);

    let inner_w = rect.width.saturating_sub(2) as usize;
    let active_w = 2usize;
    let profile_w = 10usize;
    let short_id_w = 8usize;
    let gap = 2usize;
    let fixed = active_w + profile_w + short_id_w + gap * 3;
    let remaining = inner_w.saturating_sub(fixed).max(20);
    let name_w = (remaining * 4 / 7).max(12);
    let tenant_w = remaining.saturating_sub(name_w).max(8);

    let items: Vec<ListItem> = visible.iter().map(|(idx, v)| {
        let profile = v.get("profile").and_then(|s| s.as_str()).unwrap_or("-");
        let name = v.get("name").and_then(|s| s.as_str()).unwrap_or("");
        let tenant = v.get("tenantDisplayName").and_then(|s| s.as_str())
            .or_else(|| v.get("tenantDefaultDomain").and_then(|s| s.as_str()))
            .unwrap_or("");
        let sub_id = v.get("id").and_then(|s| s.as_str()).unwrap_or("");
        let short_id = if sub_id.len() >= short_id_w { &sub_id[..short_id_w] } else { sub_id };
        let active = if sub_id == app.subscription_id { "*" } else { " " };
        let line = format!(
            "{} {}  {}  {}  {}",
            active,
            pad(&fit(profile, profile_w), profile_w),
            pad(&fit(name, name_w), name_w),
            pad(&fit(tenant, tenant_w), tenant_w),
            short_id,
        );
        let style = if *idx == list.cursor {
            Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        ListItem::new(line).style(style)
    }).collect();

    let title = format!(" Switch Subscription ({}) ", list.items.len());
    let widget = List::new(items).block(Block::default().borders(Borders::ALL).title(title));
    f.render_widget(widget, rect);
}

fn render_help(f: &mut Frame, area: Rect) {
    let w = 70u16.min(area.width.saturating_sub(4));
    let h = 18u16.min(area.height.saturating_sub(4));
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let rect = Rect { x, y, width: w, height: h };

    f.render_widget(Clear, rect);

    let body = "\
Navigation
  ↑ k         Move cursor up
  ↓ j         Move cursor down
  g / Home    First row
  G / End     Last row
  Ctrl-u/d    Page up / down
  Enter / l   Drill into selection
  Esc / h     Go back

Actions
  r           Refresh current view
  s           Switch account
  ?  F1       Toggle this help
  q  Ctrl-C   Quit
";
    let p = Paragraph::new(body)
        .block(Block::default().borders(Borders::ALL).title(" Help "))
        .wrap(Wrap { trim: false });
    f.render_widget(p, rect);
}

fn status_line(list: &ListState) -> Option<String> {
    if let Some(e) = &list.error { return Some(format!("Error: {e}")); }
    if list.loading { return Some("Loading...".into()); }
    if list.items.is_empty() { return Some("(empty)".into()); }
    None
}

fn visible_window(list: &ListState, capacity: usize) -> Vec<(usize, &serde_json::Value)> {
    if list.items.is_empty() || capacity == 0 { return Vec::new(); }
    let start = list.cursor.saturating_sub(capacity / 2)
        .min(list.items.len().saturating_sub(capacity).max(0));
    list.items.iter().enumerate().skip(start).take(capacity).collect()
}

fn fit(s: &str, max: usize) -> String {
    let count = s.chars().count();
    if count <= max { return s.to_string(); }
    if max == 0 { return String::new(); }
    if max == 1 { return "…".into(); }
    let mut out: String = s.chars().take(max - 1).collect();
    out.push('…');
    out
}

fn pad(s: &str, width: usize) -> String {
    let count = s.chars().count();
    if count >= width { return s.to_string(); }
    let mut out = String::with_capacity(s.len() + (width - count));
    out.push_str(s);
    for _ in 0..(width - count) { out.push(' '); }
    out
}

fn active_account_label(app: &App) -> String {
    if let Some(acc) = app.provider.cache().accounts.iter()
        .find(|a| a.subscription_id.as_deref() == Some(&app.subscription_id))
    {
        let profile = acc.profile.as_deref().unwrap_or("-");
        let name = acc.subscription_name.as_deref().unwrap_or(&app.subscription_id);
        format!("{profile} / {name}")
    } else {
        app.subscription_id.clone()
    }
}
