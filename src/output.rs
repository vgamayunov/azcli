use std::io::{self, Write};

use crate::models::OutputFormat;

pub fn print_output(value: &serde_json::Value, format: OutputFormat) -> anyhow::Result<()> {
    match format {
        OutputFormat::None => Ok(()),
        OutputFormat::Json => print_json(value),
        OutputFormat::Jsonc => print_jsonc(value),
        OutputFormat::Table => print_table(value),
        OutputFormat::Tsv => print_tsv(value),
        OutputFormat::Yaml | OutputFormat::Yamlc => print_yaml(value),
    }
}

fn print_json(value: &serde_json::Value) -> anyhow::Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

fn print_jsonc(value: &serde_json::Value) -> anyhow::Result<()> {
    let pretty = serde_json::to_string_pretty(value)?;
    for line in pretty.lines() {
        println!("{}", colorize_json_line(line));
    }
    Ok(())
}

fn colorize_json_line(line: &str) -> String {
    let trimmed = line.trim_start();

    if trimmed.starts_with('"') {
        if let Some(colon_pos) = trimmed.find("\":") {
            let indent = &line[..line.len() - trimmed.len()];
            let key = &trimmed[..colon_pos + 1];
            let rest = &trimmed[colon_pos + 1..];
            return format!("{indent}\x1b[34m{key}\x1b[0m{}", colorize_json_value(rest));
        }
    }

    if trimmed.starts_with('"') {
        return format!(
            "{}\x1b[32m{}\x1b[0m",
            &line[..line.len() - trimmed.len()],
            trimmed
        );
    }

    if trimmed.starts_with("true") || trimmed.starts_with("false") {
        return format!(
            "{}\x1b[33m{}\x1b[0m",
            &line[..line.len() - trimmed.len()],
            trimmed
        );
    }

    if trimmed.starts_with("null") {
        return format!(
            "{}\x1b[90m{}\x1b[0m",
            &line[..line.len() - trimmed.len()],
            trimmed
        );
    }

    if trimmed
        .bytes()
        .next()
        .map_or(false, |b| b.is_ascii_digit() || b == b'-')
    {
        return format!(
            "{}\x1b[36m{}\x1b[0m",
            &line[..line.len() - trimmed.len()],
            trimmed
        );
    }

    line.to_string()
}

fn colorize_json_value(s: &str) -> String {
    let trimmed = s.trim_start_matches(": ").trim_end_matches(',');

    if trimmed.starts_with('"') {
        let suffix = if s.ends_with(',') { "," } else { "" };
        return format!(": \x1b[32m{trimmed}\x1b[0m{suffix}");
    }

    if trimmed == "true" || trimmed == "false" {
        let suffix = if s.ends_with(',') { "," } else { "" };
        return format!(": \x1b[33m{trimmed}\x1b[0m{suffix}");
    }

    if trimmed == "null" {
        let suffix = if s.ends_with(',') { "," } else { "" };
        return format!(": \x1b[90m{trimmed}\x1b[0m{suffix}");
    }

    if trimmed
        .bytes()
        .next()
        .map_or(false, |b| b.is_ascii_digit() || b == b'-')
    {
        let suffix = if s.ends_with(',') { "," } else { "" };
        return format!(": \x1b[36m{trimmed}\x1b[0m{suffix}");
    }

    s.to_string()
}

fn print_yaml(value: &serde_json::Value) -> anyhow::Result<()> {
    let yaml = serde_yaml::to_string(value)?;
    print!("{yaml}");
    Ok(())
}

fn print_table(value: &serde_json::Value) -> anyhow::Result<()> {
    let items = match value {
        serde_json::Value::Array(arr) => arr.clone(),
        serde_json::Value::Object(map) => {
            if let Some(arr) = map.get("value").and_then(|v| v.as_array()) {
                arr.clone()
            } else {
                vec![value.clone()]
            }
        }
        _ => vec![value.clone()],
    };

    if items.is_empty() {
        return Ok(());
    }

    let columns = pick_table_columns(&items[0]);

    if columns.is_empty() {
        return print_json(value);
    }

    let rows: Vec<Vec<String>> = items
        .iter()
        .map(|item| columns.iter().map(|col| extract_field(item, col)).collect())
        .collect();

    let mut widths: Vec<usize> = columns.iter().map(|c| display_name(c).len()).collect();
    for row in &rows {
        for (i, cell) in row.iter().enumerate() {
            widths[i] = widths[i].max(cell.len());
        }
    }

    let stdout = io::stdout();
    let mut out = stdout.lock();

    for (i, col) in columns.iter().enumerate() {
        if i > 0 {
            write!(out, "  ")?;
        }
        write!(out, "{:<width$}", display_name(col), width = widths[i])?;
    }
    writeln!(out)?;

    for (i, w) in widths.iter().enumerate() {
        if i > 0 {
            write!(out, "  ")?;
        }
        write!(out, "{}", "-".repeat(*w))?;
    }
    writeln!(out)?;

    for row in &rows {
        for (i, cell) in row.iter().enumerate() {
            if i > 0 {
                write!(out, "  ")?;
            }
            write!(out, "{:<width$}", cell, width = widths[i])?;
        }
        writeln!(out)?;
    }

    Ok(())
}

fn pick_table_columns(sample: &serde_json::Value) -> Vec<String> {
    let obj = match sample.as_object() {
        Some(o) => o,
        None => return vec![],
    };

    let preferred = [
        "name",
        "location",
        "resourceGroup",
        "provisioningState",
        "properties.provisioningState",
        "hardwareProfile.vmSize",
        "sku.name",
        "properties.dnsName",
    ];

    let mut cols = Vec::new();
    for &p in &preferred {
        if resolve_path(sample, p).is_some() {
            cols.push(p.to_string());
        }
    }

    if cols.is_empty() {
        for key in obj.keys().take(6) {
            cols.push(key.clone());
        }
    }

    cols
}

fn resolve_path<'a>(value: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value> {
    let mut current = value;
    for segment in path.split('.') {
        current = current.get(segment)?;
    }
    Some(current)
}

fn extract_field(item: &serde_json::Value, path: &str) -> String {
    match resolve_path(item, path) {
        Some(serde_json::Value::String(s)) => s.clone(),
        Some(serde_json::Value::Null) => String::new(),
        Some(serde_json::Value::Bool(b)) => b.to_string(),
        Some(serde_json::Value::Number(n)) => n.to_string(),
        Some(other) => other.to_string(),
        None => String::new(),
    }
}

fn display_name(path: &str) -> &str {
    path.rsplit('.').next().unwrap_or(path)
}

fn print_tsv(value: &serde_json::Value) -> anyhow::Result<()> {
    let items = match value {
        serde_json::Value::Array(arr) => arr.clone(),
        serde_json::Value::Object(map) => {
            if let Some(arr) = map.get("value").and_then(|v| v.as_array()) {
                arr.clone()
            } else {
                vec![value.clone()]
            }
        }
        _ => vec![value.clone()],
    };

    if items.is_empty() {
        return Ok(());
    }

    let columns = pick_table_columns(&items[0]);

    let stdout = io::stdout();
    let mut out = stdout.lock();

    for row in &items {
        let cells: Vec<String> = columns.iter().map(|col| extract_field(row, col)).collect();
        writeln!(out, "{}", cells.join("\t"))?;
    }

    Ok(())
}
