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
    let items = unwrap_items(value);

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

    if obj.contains_key("resourceType")
        && obj.contains_key("capabilities")
        && obj.contains_key("locationInfo")
    {
        return vec![
            "@resourceType".to_string(),
            "@locations".to_string(),
            "@name".to_string(),
            "@zones".to_string(),
            "@restrictions".to_string(),
        ];
    }

    if obj.contains_key("roleName") && obj.contains_key("roleDefinitionId") && obj.contains_key("scope") {
        if obj.contains_key("principalType") {
            return vec![
                "roleName".to_string(),
                "principalType".to_string(),
                "principalId".to_string(),
                "@scopeShort".to_string(),
            ];
        }
        let mut cols = vec!["roleName".to_string()];
        if obj.contains_key("state") {
            cols.push("state".to_string());
        }
        if obj.contains_key("assignmentType") {
            cols.push("assignmentType".to_string());
        }
        cols.push("@scopeShort".to_string());
        cols.push("startDateTime".to_string());
        if obj.contains_key("endDateTime") {
            cols.push("endDateTime".to_string());
        }
        return cols;
    }

    if obj.contains_key("roleName") && obj.contains_key("type") && obj.contains_key("assignableScopes") {
        return vec![
            "roleName".to_string(),
            "type".to_string(),
            "name".to_string(),
            "description".to_string(),
        ];
    }

    if obj.contains_key("isDefault") && obj.contains_key("tenantId") && obj.contains_key("id") {
        let mut cols = vec![
            "name".to_string(),
            "id".to_string(),
            "tenantId".to_string(),
            "isDefault".to_string(),
        ];
        if obj.contains_key("state") {
            cols.insert(3, "state".to_string());
        }
        return cols;
    }

    if obj.contains_key("regionalDisplayName") && obj.contains_key("displayName") {
        return vec![
            "name".to_string(),
            "displayName".to_string(),
            "regionalDisplayName".to_string(),
        ];
    }

    if let Some(t) = obj.get("type").and_then(|v| v.as_str()) {
        if t.eq_ignore_ascii_case("Microsoft.Compute/images") {
            return vec![
                "name".to_string(),
                "@resourceGroup".to_string(),
                "location".to_string(),
                "properties.hyperVGeneration".to_string(),
                "properties.provisioningState".to_string(),
            ];
        }
        if t.eq_ignore_ascii_case("Microsoft.VirtualMachineImages/imageTemplates") {
            return vec![
                "name".to_string(),
                "@resourceGroup".to_string(),
                "location".to_string(),
                "properties.provisioningState".to_string(),
                "properties.lastRunStatus.runState".to_string(),
            ];
        }
        if t.eq_ignore_ascii_case("Microsoft.Compute/galleries") {
            return vec![
                "name".to_string(),
                "@resourceGroup".to_string(),
                "location".to_string(),
                "properties.provisioningState".to_string(),
                "properties.sharingProfile.permissions".to_string(),
            ];
        }
        if t.eq_ignore_ascii_case("Microsoft.Compute/galleries/images") {
            return vec![
                "name".to_string(),
                "@resourceGroup".to_string(),
                "location".to_string(),
                "properties.osType".to_string(),
                "properties.hyperVGeneration".to_string(),
                "properties.provisioningState".to_string(),
            ];
        }
        if t.eq_ignore_ascii_case("Microsoft.Compute/galleries/images/versions") {
            return vec![
                "name".to_string(),
                "@resourceGroup".to_string(),
                "location".to_string(),
                "properties.publishingProfile.excludeFromLatest".to_string(),
                "properties.provisioningState".to_string(),
            ];
        }
        if t.eq_ignore_ascii_case("microsoft.compute/locations/communitygalleries") {
            return vec![
                "name".to_string(),
                "location".to_string(),
                "properties.publicNames".to_string(),
            ];
        }
        if t.eq_ignore_ascii_case("Microsoft.Network/virtualNetworks") {
            return vec![
                "name".to_string(),
                "@resourceGroup".to_string(),
                "location".to_string(),
                "properties.addressSpace.addressPrefixes".to_string(),
                "properties.provisioningState".to_string(),
            ];
        }
        if t.eq_ignore_ascii_case("Microsoft.Network/virtualNetworks/subnets") {
            return vec![
                "name".to_string(),
                "properties.addressPrefix".to_string(),
                "properties.provisioningState".to_string(),
            ];
        }
        if t.eq_ignore_ascii_case("Microsoft.Network/virtualNetworks/virtualNetworkPeerings") {
            return vec![
                "name".to_string(),
                "properties.peeringState".to_string(),
                "properties.peeringSyncLevel".to_string(),
                "properties.provisioningState".to_string(),
            ];
        }
        if t.eq_ignore_ascii_case("Microsoft.Network/networkSecurityGroups") {
            return vec![
                "name".to_string(),
                "@resourceGroup".to_string(),
                "location".to_string(),
                "properties.provisioningState".to_string(),
            ];
        }
        if t.eq_ignore_ascii_case("Microsoft.Network/networkSecurityGroups/securityRules") {
            return vec![
                "name".to_string(),
                "properties.priority".to_string(),
                "properties.direction".to_string(),
                "properties.access".to_string(),
                "properties.protocol".to_string(),
                "properties.destinationPortRange".to_string(),
            ];
        }
        if t.eq_ignore_ascii_case("Microsoft.Network/publicIPAddresses") {
            return vec![
                "name".to_string(),
                "@resourceGroup".to_string(),
                "location".to_string(),
                "properties.ipAddress".to_string(),
                "properties.publicIPAllocationMethod".to_string(),
                "properties.provisioningState".to_string(),
            ];
        }
        if t.eq_ignore_ascii_case("Microsoft.Network/networkInterfaces") {
            return vec![
                "name".to_string(),
                "@resourceGroup".to_string(),
                "location".to_string(),
                "properties.macAddress".to_string(),
                "properties.primary".to_string(),
                "properties.provisioningState".to_string(),
            ];
        }
        if t.eq_ignore_ascii_case("Microsoft.Network/networkInterfaces/ipConfigurations") {
            return vec![
                "name".to_string(),
                "properties.privateIPAddress".to_string(),
                "properties.privateIPAllocationMethod".to_string(),
                "properties.primary".to_string(),
                "properties.provisioningState".to_string(),
            ];
        }
        if t.eq_ignore_ascii_case("Microsoft.Network/privateEndpoints") {
            return vec![
                "name".to_string(),
                "@resourceGroup".to_string(),
                "location".to_string(),
                "properties.provisioningState".to_string(),
            ];
        }
    }

    if obj.contains_key("identifier") && obj.contains_key("uniqueId") && obj.get("location").is_some() {
        return vec![
            "name".to_string(),
            "location".to_string(),
            "identifier.uniqueId".to_string(),
        ];
    }

    if resolve_path(sample, "properties.addressPrefix").is_some()
        && resolve_path(sample, "properties.privateEndpointNetworkPolicies").is_some()
    {
        return vec![
            "name".to_string(),
            "properties.addressPrefix".to_string(),
            "properties.provisioningState".to_string(),
        ];
    }

    if resolve_path(sample, "properties.peeringState").is_some() {
        return vec![
            "name".to_string(),
            "properties.peeringState".to_string(),
            "properties.peeringSyncLevel".to_string(),
            "properties.provisioningState".to_string(),
        ];
    }

    if resolve_path(sample, "properties.priority").is_some()
        && resolve_path(sample, "properties.direction").is_some()
        && resolve_path(sample, "properties.access").is_some()
    {
        return vec![
            "name".to_string(),
            "properties.priority".to_string(),
            "properties.direction".to_string(),
            "properties.access".to_string(),
            "properties.protocol".to_string(),
            "properties.destinationPortRange".to_string(),
        ];
    }

    if resolve_path(sample, "properties.privateIPAddress").is_some()
        && resolve_path(sample, "properties.privateIPAllocationMethod").is_some()
    {
        return vec![
            "name".to_string(),
            "properties.privateIPAddress".to_string(),
            "properties.privateIPAllocationMethod".to_string(),
            "properties.primary".to_string(),
            "properties.provisioningState".to_string(),
        ];
    }

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
    match path {
        "@resourceType" => resolve_path(item, "resourceType")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        "@name" => resolve_path(item, "name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        "@locations" => join_string_array(item.get("locations")),
        "@zones" => item
            .get("locationInfo")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .map(|li| join_string_array(li.get("zones")))
            .unwrap_or_default(),
        "@restrictions" => summarize_restrictions(item.get("restrictions")),
        "@scopeShort" => short_scope(item.get("scope").and_then(|v| v.as_str()).unwrap_or("")),
        "@resourceGroup" => extract_resource_group(item),
        _ => match resolve_path(item, path) {
            Some(serde_json::Value::String(s)) => s.clone(),
            Some(serde_json::Value::Null) => String::new(),
            Some(serde_json::Value::Bool(b)) => b.to_string(),
            Some(serde_json::Value::Number(n)) => n.to_string(),
            Some(other) => other.to_string(),
            None => String::new(),
        },
    }
}

fn join_string_array(value: Option<&serde_json::Value>) -> String {
    let arr = match value.and_then(|v| v.as_array()) {
        Some(a) => a,
        None => return String::new(),
    };
    let mut parts: Vec<&str> = arr.iter().filter_map(|v| v.as_str()).collect();
    parts.sort_unstable();
    parts.join(",")
}

fn extract_resource_group(item: &serde_json::Value) -> String {
    if let Some(rg) = item.get("resourceGroup").and_then(|v| v.as_str()) {
        return rg.to_string();
    }
    let id = match item.get("id").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return String::new(),
    };
    let lower = id.to_ascii_lowercase();
    let needle = "/resourcegroups/";
    let start = match lower.find(needle) {
        Some(i) => i + needle.len(),
        None => return String::new(),
    };
    let rest = &id[start..];
    rest.split('/').next().unwrap_or("").to_string()
}

fn summarize_restrictions(value: Option<&serde_json::Value>) -> String {
    let arr = match value.and_then(|v| v.as_array()) {
        Some(a) if !a.is_empty() => a,
        _ => return "None".to_string(),
    };
    arr.iter()
        .map(|r| {
            let reason = r.get("reasonCode").and_then(|v| v.as_str()).unwrap_or("Restricted");
            let rtype = r.get("type").and_then(|v| v.as_str()).unwrap_or("");
            let info = r.get("restrictionInfo");
            let locs = info
                .and_then(|i| i.get("locations"))
                .map(|v| join_string_array(Some(v)))
                .unwrap_or_default();
            let zones = info
                .and_then(|i| i.get("zones"))
                .map(|v| join_string_array(Some(v)))
                .unwrap_or_default();
            let mut parts = vec![reason.to_string()];
            if !rtype.is_empty() {
                parts.push(format!("type: {}", rtype));
            }
            if !locs.is_empty() {
                parts.push(format!("locations: {}", locs));
            }
            if !zones.is_empty() {
                parts.push(format!("zones: {}", zones));
            }
            parts.join(", ")
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn display_name(path: &str) -> &str {
    match path {
        "@resourceType" => "ResourceType",
        "@name" => "Name",
        "@locations" => "Locations",
        "@zones" => "Zones",
        "@restrictions" => "Restrictions",
        "@scopeShort" => "Scope",
        "@resourceGroup" => "ResourceGroup",
        "roleName" => "Role",
        "state" => "State",
        "assignmentType" => "Type",
        "startDateTime" => "Start",
        "endDateTime" => "End",
        "principalType" => "PrincipalType",
        "principalId" => "PrincipalId",
        "type" => "Type",
        "name" => "Name",
        "description" => "Description",
        "tenantId" => "TenantId",
        "isDefault" => "IsDefault",
        "displayName" => "DisplayName",
        "regionalDisplayName" => "Region",
        "id" => "Id",
        _ => path.rsplit('.').next().unwrap_or(path),
    }
}

fn print_tsv(value: &serde_json::Value) -> anyhow::Result<()> {
    let items = unwrap_items(value);

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

fn unwrap_items(value: &serde_json::Value) -> Vec<serde_json::Value> {
    match value {
        serde_json::Value::Array(arr) => arr.clone(),
        serde_json::Value::Object(map) => {
            if let Some(arr) = map.get("value").and_then(|v| v.as_array()) {
                return arr.clone();
            }
            let has_eligible = map.get("eligible").and_then(|v| v.as_array()).is_some();
            let has_active = map.get("active").and_then(|v| v.as_array()).is_some();
            if has_eligible || has_active {
                let mut out = Vec::new();
                if let Some(arr) = map.get("eligible").and_then(|v| v.as_array()) {
                    for item in arr {
                        out.push(tag_state(item.clone(), "Eligible"));
                    }
                }
                if let Some(arr) = map.get("active").and_then(|v| v.as_array()) {
                    for item in arr {
                        out.push(tag_state(item.clone(), "Active"));
                    }
                }
                return out;
            }
            vec![value.clone()]
        }
        _ => vec![value.clone()],
    }
}

fn tag_state(mut item: serde_json::Value, state: &str) -> serde_json::Value {
    if let Some(obj) = item.as_object_mut() {
        obj.insert("state".to_string(), serde_json::Value::String(state.to_string()));
    }
    item
}

fn short_scope(scope: &str) -> String {
    let parts: Vec<&str> = scope.split('/').filter(|s| !s.is_empty()).collect();
    let mut sub = "";
    let mut rg: Option<&str> = None;
    let mut leaf: Option<&str> = None;
    let mut i = 0;
    while i < parts.len() {
        match parts[i].to_ascii_lowercase().as_str() {
            "subscriptions" if i + 1 < parts.len() => {
                sub = parts[i + 1];
                i += 2;
            }
            "resourcegroups" if i + 1 < parts.len() => {
                rg = Some(parts[i + 1]);
                i += 2;
            }
            "providers" if i + 3 < parts.len() => {
                leaf = Some(parts[parts.len() - 1]);
                i = parts.len();
            }
            _ => i += 1,
        }
    }
    let sub_short = sub.split('-').next().unwrap_or(sub);
    match (rg, leaf) {
        (None, _) => format!("sub:{}", sub_short),
        (Some(r), None) => format!("sub:{}/{}", sub_short, r),
        (Some(r), Some(l)) => format!("sub:{}/{}/.../{}", sub_short, r, l),
    }
}
