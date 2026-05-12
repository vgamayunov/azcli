use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};

use super::app::{App, CapacityPrompt, ListState, PendingConfirm, ResourceSort, RgSort, View};

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

    if let Some(ref pc) = app.pending_confirm {
        render_confirm(f, area, pc);
    }

    if let Some(ref prompt) = app.capacity_prompt {
        render_capacity_prompt(f, area, prompt);
    }

    if app.help_visible {
        render_help(f, area);
    }

    if let Some(ref text) = app.log_modal {
        render_log_modal(f, area, text);
    }
}

fn render_header(f: &mut Frame, area: Rect, app: &App) {
    let sub_label = active_account_label(app);
    let view_label = match app.current_view() {
        View::ResourceGroups => "Resource Groups".to_string(),
        View::ResourcesInGroup { rg } => format!("Resources / {rg}"),
        View::VmDetail { rg, name } => format!("VM / {rg} / {name}"),
        View::VmssDetail { rg, name } => format!("VMSS / {rg} / {name}"),
        View::VmssInstanceDetail { rg, vmss, instance_id } => format!("VMSS / {rg} / {vmss} / instance {instance_id}"),
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
        View::ResourceGroups => render_rg_list(f, area, &app.rg_list, app.rg_sort),
        View::ResourcesInGroup { .. } => render_resource_list(f, area, &app.resource_list, app.resource_sort),
        View::VmDetail { .. } => render_vm_detail(f, area, app),
        View::VmssDetail { .. } => render_vmss_detail(f, area, app),
        View::VmssInstanceDetail { .. } => render_vmss_instance_detail(f, area, app),
        View::AccountPicker => {}
    }
}

fn render_rg_list(f: &mut Frame, area: Rect, list: &ListState, sort: RgSort) {
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

    let title = format!("Resource Groups ({})  [sort: {}]", list.items.len(), sort.label());
    let widget = List::new(items).block(Block::default().borders(Borders::ALL).title(title));
    f.render_widget(widget, area);
}

fn render_resource_list(f: &mut Frame, area: Rect, list: &ListState, sort: ResourceSort) {
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

    let title = format!("Resources ({})  [sort: {}]", list.items.len(), sort.label());
    let widget = List::new(items).block(Block::default().borders(Borders::ALL).title(title));
    f.render_widget(widget, area);
}

fn render_footer(f: &mut Frame, area: Rect, app: &App) {
    let hints = match app.current_view() {
        View::ResourceGroups => format!(
            "↑↓/jk move  Enter drill  r refresh  o sort: {}  s switch-sub  ? help  q quit",
            app.rg_sort.label()
        ),
        View::ResourcesInGroup { .. } => format!(
            "↑↓/jk move  Enter (VM/VMSS)  Esc back  r refresh  o sort: {}  s switch-sub  ? help",
            app.resource_sort.label()
        ),
        View::VmDetail { .. } => "S start  D deallocate  O power-off  T restart  r refresh  Esc back  ? help".to_string(),
        View::VmssDetail { .. } => {
            let sel = app.vmss_detail.selected.len();
            let scope = if sel == 0 { "ALL".to_string() } else { format!("{sel} selected") };
            let del_hint = if sel == 0 { String::new() } else { format!("  X delete-{sel}") };
            format!("↑↓/jk move  Space select  Enter instance  S/D/O/T → {scope}{del_hint}  C scale  Esc back  ? help")
        }
        View::VmssInstanceDetail { .. } => "S start  D deallocate  O power-off  T restart  X DELETE  r refresh  Esc back  ? help".to_string(),
        View::AccountPicker => "↑↓/jk move  Enter select  r refresh  Esc cancel".to_string(),
    };
    let mut lines = vec![Line::from(Span::styled(hints, Style::default().fg(Color::DarkGray)))];
    if let Some(ref msg) = app.action_in_progress {
        lines.push(Line::from(Span::styled(format!("⏳ {msg}..."), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))));
    } else if !app.status.is_empty() {
        lines.push(Line::from(Span::styled(app.status.clone(), Style::default().fg(Color::Yellow))));
    }
    let p = Paragraph::new(lines).block(Block::default().borders(Borders::TOP));
    f.render_widget(p, area);
}

fn render_vm_detail(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default().borders(Borders::ALL).title("VM");
    if app.vm_detail.loading && app.vm_detail.value.is_none() {
        let p = Paragraph::new("Loading...").block(block);
        f.render_widget(p, area);
        return;
    }
    if let Some(err) = &app.vm_detail.error {
        let p = Paragraph::new(format!("Error: {err}")).block(block).wrap(Wrap { trim: false });
        f.render_widget(p, area);
        return;
    }
    let Some(value) = &app.vm_detail.value else {
        f.render_widget(block, area);
        return;
    };

    let s = |key: &str| -> String {
        value.get(key).and_then(|v| v.as_str()).unwrap_or("").to_string()
    };

    let power = s("powerState");
    let power_span = match power.as_str() {
        "running" => Span::styled(power.clone(), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        "stopped" | "deallocated" => Span::styled(power.clone(), Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        "starting" | "stopping" | "deallocating" => Span::styled(power.clone(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        _ if power.is_empty() => Span::styled("(unknown)", Style::default().fg(Color::DarkGray)),
        _ => Span::raw(power.clone()),
    };

    let field = |label: &'static str, val: String| -> Line<'static> {
        Line::from(vec![
            Span::styled(format!("{label:<14}"), Style::default().fg(Color::DarkGray)),
            Span::raw(val),
        ])
    };

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(vec![
        Span::styled(format!("{:<14}", "power"), Style::default().fg(Color::DarkGray)),
        power_span,
    ]));
    lines.push(field("name", s("name")));
    lines.push(field("vmSize", s("vmSize")));
    lines.push(field("location", s("location")));
    lines.push(field("osType", s("osType")));
    lines.push(field("provisioning", s("provisioningState")));
    lines.push(field("privateIps", s("privateIps")));
    lines.push(field("publicIps", s("publicIps")));
    lines.push(field("fqdns", s("fqdns")));
    lines.push(Line::from(""));

    if let Some(statuses) = value.pointer("/instanceView/statuses").and_then(|v| v.as_array()) {
        lines.push(Line::from(Span::styled("statuses", Style::default().add_modifier(Modifier::BOLD))));
        for st in statuses {
            let code = st.get("code").and_then(|v| v.as_str()).unwrap_or("");
            let disp = st.get("displayStatus").and_then(|v| v.as_str()).unwrap_or("");
            let level = st.get("level").and_then(|v| v.as_str()).unwrap_or("Info");
            let color = match level {
                "Error" => Color::Red,
                "Warning" => Color::Yellow,
                _ => Color::Reset,
            };
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(format!("{code:<34}"), Style::default().fg(Color::DarkGray)),
                Span::styled(disp.to_string(), Style::default().fg(color)),
            ]));
        }
    }

    let p = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });
    f.render_widget(p, area);
}

fn render_vmss_detail(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(7), Constraint::Min(0)])
        .split(area);

    render_vmss_summary(f, chunks[0], app);
    render_vmss_instance_table(f, chunks[1], app);
}

fn render_vmss_summary(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default().borders(Borders::ALL).title("VMSS");
    if app.vmss_detail.loading && app.vmss_detail.vmss.is_none() {
        let p = Paragraph::new("Loading...").block(block);
        f.render_widget(p, area);
        return;
    }
    if let Some(err) = &app.vmss_detail.error {
        let p = Paragraph::new(format!("Error: {err}")).block(block).wrap(Wrap { trim: false });
        f.render_widget(p, area);
        return;
    }
    let Some(vmss) = &app.vmss_detail.vmss else {
        f.render_widget(block, area);
        return;
    };

    let name = vmss.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let location = vmss.get("location").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let sku_name = vmss.pointer("/sku/name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let sku_tier = vmss.pointer("/sku/tier").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let capacity = vmss.pointer("/sku/capacity").and_then(|v| v.as_i64()).map(|c| c.to_string()).unwrap_or_default();
    let provisioning = vmss.get("provisioningState").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let upgrade_mode = vmss.pointer("/upgradePolicy/mode").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let orchestration = vmss.get("orchestrationMode").and_then(|v| v.as_str()).unwrap_or("").to_string();

    let field = |label: &'static str, val: String| -> Line<'static> {
        Line::from(vec![
            Span::styled(format!("{label:<14}"), Style::default().fg(Color::DarkGray)),
            Span::raw(val),
        ])
    };

    let lines = vec![
        Line::from(Span::styled(name, Style::default().add_modifier(Modifier::BOLD))),
        field("sku", format!("{sku_name}  ({sku_tier})  capacity={capacity}")),
        field("location", location),
        field("provisioning", provisioning),
        field("upgradeMode", format!("{upgrade_mode}   orchestration={orchestration}")),
    ];

    let p = Paragraph::new(lines).block(block);
    f.render_widget(p, area);
}

fn render_vmss_instance_table(f: &mut Frame, area: Rect, app: &App) {
    let n_sel = app.vmss_detail.selected.len();
    let title = if n_sel > 0 {
        format!("Instances ({} total, {} selected)", app.vmss_detail.instances.len(), n_sel)
    } else {
        format!("Instances ({})", app.vmss_detail.instances.len())
    };
    let block = Block::default().borders(Borders::ALL).title(title);
    if app.vmss_detail.loading && app.vmss_detail.instances.is_empty() {
        let p = Paragraph::new("Loading...").block(block);
        f.render_widget(p, area);
        return;
    }
    if app.vmss_detail.instances.is_empty() {
        let p = Paragraph::new("(no instances)").block(block);
        f.render_widget(p, area);
        return;
    }

    let items = &app.vmss_detail.instances;

    let max_id = items.iter()
        .map(|i| i.get("instanceId").and_then(|v| v.as_str()).unwrap_or("").chars().count())
        .max().unwrap_or(0);
    let max_name = items.iter()
        .map(|i| i.get("name").and_then(|v| v.as_str()).unwrap_or("").chars().count())
        .max().unwrap_or(0);
    let any_latest = items.iter().any(|i| i.get("latestModelApplied").map(|v| !v.is_null()).unwrap_or(false));

    let id_w = max_id.clamp(4, 18);
    let name_w = max_name.clamp(10, 36);
    let power_w = 14usize;
    let prov_w = 14usize;
    let latest_w = if any_latest { 4usize } else { 0 };

    let capacity = area.height.saturating_sub(2) as usize;
    let cursor = app.vmss_detail.cursor;
    let start = cursor.saturating_sub(capacity / 2)
        .min(items.len().saturating_sub(capacity).max(0));
    let visible: Vec<(usize, &serde_json::Value)> = items.iter().enumerate().skip(start).take(capacity).collect();

    let list_items: Vec<ListItem> = visible.iter().map(|(idx, inst)| {
        let iid = inst.get("instanceId").and_then(|v| v.as_str()).unwrap_or("");
        let iname = inst.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let prov = inst.get("provisioningState").and_then(|v| v.as_str()).unwrap_or("");
        let latest = inst.get("latestModelApplied").and_then(|v| v.as_bool())
            .map(|b| if b { "yes" } else { "no" })
            .unwrap_or("");
        let power = extract_power(inst);
        let selected = app.vmss_detail.selected.contains(iid);

        let marker = if selected {
            Span::styled("● ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))
        } else {
            Span::raw("  ")
        };

        let mut spans = vec![
            marker,
            Span::raw(pad(&fit(iid, id_w), id_w)),
            Span::raw("  "),
            Span::raw(pad(&fit(iname, name_w), name_w)),
            Span::raw("  "),
            power_span(&power, power_w),
            Span::raw("  "),
            Span::raw(pad(&fit(prov, prov_w), prov_w)),
        ];
        if latest_w > 0 {
            spans.push(Span::raw("  "));
            spans.push(Span::raw(pad(&fit(latest, latest_w), latest_w)));
        }

        let style = if *idx == cursor {
            Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        ListItem::new(Line::from(spans)).style(style)
    }).collect();

    let widget = List::new(list_items).block(block);
    f.render_widget(widget, area);
}

fn extract_power(inst: &serde_json::Value) -> String {
    if let Some(statuses) = inst.pointer("/instanceView/statuses").and_then(|v| v.as_array()) {
        for s in statuses {
            if let Some(code) = s.get("code").and_then(|v| v.as_str()) {
                if let Some(rest) = code.strip_prefix("PowerState/") {
                    return rest.to_string();
                }
            }
        }
    }
    String::new()
}

fn power_span<'a>(power: &str, width: usize) -> Span<'a> {
    let text = if power.is_empty() { "-".to_string() } else { power.to_string() };
    let padded = pad(&fit(&text, width), width);
    let style = match power {
        "running" => Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        "stopped" | "deallocated" => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        "starting" | "stopping" | "deallocating" => Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        "" => Style::default().fg(Color::DarkGray),
        _ => Style::default(),
    };
    Span::styled(padded, style)
}

fn render_vmss_instance_detail(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default().borders(Borders::ALL).title("VMSS Instance");
    if let Some(err) = &app.vmss_instance_detail.error {
        let p = Paragraph::new(format!("Error: {err}")).block(block).wrap(Wrap { trim: false });
        f.render_widget(p, area);
        return;
    }
    let Some(inst) = &app.vmss_instance_detail.instance else {
        let p = Paragraph::new("(no data)").block(block);
        f.render_widget(p, area);
        return;
    };

    let s = |key: &str| -> String {
        inst.get(key).and_then(|v| v.as_str()).unwrap_or("").to_string()
    };

    let name = s("name");
    let instance_id = s("instanceId");
    let prov = s("provisioningState");
    let power = extract_power(inst);

    let power_disp = match power.as_str() {
        "" => Span::styled("(unknown)", Style::default().fg(Color::DarkGray)),
        "running" => Span::styled(power.clone(), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        "stopped" | "deallocated" => Span::styled(power.clone(), Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        "starting" | "stopping" | "deallocating" => Span::styled(power.clone(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        _ => Span::raw(power.clone()),
    };

    let field = |label: &'static str, val: String| -> Line<'static> {
        Line::from(vec![
            Span::styled(format!("{label:<14}"), Style::default().fg(Color::DarkGray)),
            Span::raw(val),
        ])
    };

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled(name.clone(), Style::default().add_modifier(Modifier::BOLD))));
    lines.push(field("instanceId", instance_id));
    lines.push(Line::from(vec![
        Span::styled(format!("{:<14}", "power"), Style::default().fg(Color::DarkGray)),
        power_disp,
    ]));
    lines.push(field("provisioning", prov));
    lines.push(Line::from(""));

    if let Some(statuses) = inst.pointer("/instanceView/statuses").and_then(|v| v.as_array()) {
        lines.push(Line::from(Span::styled("statuses", Style::default().add_modifier(Modifier::BOLD))));
        for st in statuses {
            let code = st.get("code").and_then(|v| v.as_str()).unwrap_or("");
            let disp = st.get("displayStatus").and_then(|v| v.as_str()).unwrap_or("");
            let level = st.get("level").and_then(|v| v.as_str()).unwrap_or("Info");
            let color = match level {
                "Error" => Color::Red,
                "Warning" => Color::Yellow,
                _ => Color::Reset,
            };
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(format!("{code:<34}"), Style::default().fg(Color::DarkGray)),
                Span::styled(disp.to_string(), Style::default().fg(color)),
            ]));
        }
    }

    let p = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });
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
    let h = 38u16.min(area.height.saturating_sub(4));
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
  Esc / h     Go back (clears selection first if any)

Actions
  r           Refresh current view
  o           Cycle sort
              · RGs:       name → location
              · Resources: name → type → location
  s           Switch subscription
  ?  F1       Toggle this help
  q  Ctrl-C   Quit

VM Detail view
  S           Start
  D           Deallocate (stop + release compute)
  O           Power off (stop, keep compute)
  T           Restart

VMSS Detail view
  Space       Toggle instance selection
  a           Clear selection
  Enter       Open instance detail
  S D O T     Start / Deallocate / PowerOff / Restart
              → selected if any, else ALL instances
  X           DELETE selected instances (selection required)
  C           Set capacity (scale)

VMSS Instance Detail view
  S D O T     Same as VM Detail, on this single instance
  X           DELETE this instance
";
    let p = Paragraph::new(body)
        .block(Block::default().borders(Borders::ALL).title(" Help "))
        .wrap(Wrap { trim: false });
    f.render_widget(p, rect);
}

fn render_confirm(f: &mut Frame, area: Rect, pc: &PendingConfirm) {
    let w = 64u16.min(area.width.saturating_sub(4));
    let h = 7u16.min(area.height.saturating_sub(4));
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let rect = Rect { x, y, width: w, height: h };

    f.render_widget(Clear, rect);

    let body = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::raw(format!("Confirm: {} ", pc.label())),
            Span::styled(pc.name.clone(), Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(Span::styled("  Press [y] to confirm, anything else to cancel",
            Style::default().fg(Color::DarkGray))),
    ];

    let p = Paragraph::new(body)
        .block(Block::default().borders(Borders::ALL).title(Span::styled(" Confirm ",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))));
    f.render_widget(p, rect);
}

fn render_capacity_prompt(f: &mut Frame, area: Rect, prompt: &CapacityPrompt) {
    let w = 64u16.min(area.width.saturating_sub(4));
    let h = 9u16.min(area.height.saturating_sub(4));
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let rect = Rect { x, y, width: w, height: h };

    f.render_widget(Clear, rect);

    let mut body: Vec<Line> = Vec::new();
    body.push(Line::from(""));
    body.push(Line::from(vec![
        Span::raw("  "),
        Span::raw(format!("Set VMSS '{}' capacity (current: {})", prompt.vmss, prompt.current_capacity)),
    ]));
    body.push(Line::from(""));
    body.push(Line::from(vec![
        Span::raw("  new capacity: "),
        Span::styled(prompt.input.clone(), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::styled("█", Style::default().fg(Color::Green).add_modifier(Modifier::SLOW_BLINK)),
    ]));
    if let Some(err) = &prompt.error {
        body.push(Line::from(Span::styled(format!("  {err}"), Style::default().fg(Color::Red))));
    } else {
        body.push(Line::from(Span::styled("  Enter to confirm, Esc to cancel",
            Style::default().fg(Color::DarkGray))));
    }

    let p = Paragraph::new(body)
        .block(Block::default().borders(Borders::ALL).title(Span::styled(" Scale VMSS ",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))));
    f.render_widget(p, rect);
}

fn render_log_modal(f: &mut Frame, area: Rect, text: &str) {
    let w = area.width.saturating_sub(4).max(40);
    let h = area.height.saturating_sub(4).max(8);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let rect = Rect { x, y, width: w, height: h };

    f.render_widget(Clear, rect);

    let trimmed = text.trim_end();
    let max_lines = rect.height.saturating_sub(3) as usize;
    let lines: Vec<&str> = trimmed.lines().collect();
    let shown: Vec<&str> = if lines.len() > max_lines {
        lines[lines.len() - max_lines..].to_vec()
    } else {
        lines
    };
    let body = shown.join("\n");

    let p = Paragraph::new(body)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(" Captured stderr (press any key to dismiss) ",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))))
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
