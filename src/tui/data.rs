use tokio::sync::mpsc;

use crate::auth::TokenProvider;

use super::app::{App, View, VmOp, VmssOp, VmssScope};
use super::event::{Event, FetchPayload};

pub fn spawn_fetch_current(app: &App, tx: mpsc::Sender<Event>) {
    match app.current_view() {
        View::ResourceGroups => spawn_fetch_rgs(app, tx),
        View::ResourcesInGroup { rg } => spawn_fetch_resources(app, rg.clone(), tx),
        View::AccountPicker => spawn_fetch_subscriptions(app, tx),
        View::VmDetail { rg, name } => spawn_fetch_vm_detail(app, rg.clone(), name.clone(), tx),
        View::VmssDetail { rg, name } => spawn_fetch_vmss_detail(app, rg.clone(), name.clone(), tx),
        View::VmssInstanceDetail { rg, vmss, .. } => spawn_fetch_vmss_detail(app, rg.clone(), vmss.clone(), tx),
        View::PimPanel => {
            let sub = app.subscription_id.clone();
            spawn_fetch_pim(app, sub, tx);
        }
    }
}

pub fn spawn_fetch_rgs(app: &App, tx: mpsc::Sender<Event>) {
    let client = app.client.clone();
    tokio::spawn(async move {
        match client.list_resource_groups().await {
            Ok(rgs) => {
                let items: Vec<serde_json::Value> = rgs.into_iter()
                    .map(|rg| serde_json::to_value(rg).unwrap_or(serde_json::Value::Null))
                    .collect();
                let _ = tx.send(Event::FetchOk(FetchPayload::ResourceGroups(items))).await;
            }
            Err(e) => {
                let _ = tx.send(Event::FetchErr(format!("{e:#}"))).await;
            }
        }
    });
}

pub fn spawn_fetch_resources(app: &App, rg: String, tx: mpsc::Sender<Event>) {
    let client = app.client.clone();
    tokio::spawn(async move {
        match client.list_resources_in_group(&rg).await {
            Ok(items) => {
                let _ = tx.send(Event::FetchOk(FetchPayload::ResourcesInGroup { rg, items })).await;
            }
            Err(e) => {
                let _ = tx.send(Event::FetchErr(format!("{e:#}"))).await;
            }
        }
    });
}

pub fn spawn_fetch_vm_detail(app: &App, rg: String, name: String, tx: mpsc::Sender<Event>) {
    let client = app.client.clone();
    tokio::spawn(async move {
        match crate::commands::vm::show::execute(&client, &rg, &name, true).await {
            Ok(value) => {
                let _ = tx.send(Event::FetchOk(FetchPayload::VmDetail { rg, name, value })).await;
            }
            Err(e) => {
                let _ = tx.send(Event::FetchErr(format!("{e:#}"))).await;
            }
        }
    });
}

pub fn spawn_vm_action(app: &App, op: VmOp, rg: String, name: String, tx: mpsc::Sender<Event>) {
    let client = app.client.clone();
    tokio::spawn(async move {
        let res = match op {
            VmOp::Start => client.start_vm(&rg, &name).await,
            VmOp::Deallocate => client.stop_vm(&rg, &name, true).await,
            VmOp::PowerOff => client.stop_vm(&rg, &name, false).await,
            VmOp::Restart => client.vm_post_action(&rg, &name, "restart").await,
            VmOp::Delete => client.vm_delete(&rg, &name, false).await,
        };
        match res {
            Ok(()) => {
                let _ = tx.send(Event::ActionOk(format!("{} {} done", op.verb_ing(), name))).await;
            }
            Err(e) => {
                let _ = tx.send(Event::ActionErr(format!("{} {} failed: {e:#}", op.verb_ing(), name))).await;
            }
        }
    });
}

pub fn spawn_fetch_vmss_detail(app: &App, rg: String, name: String, tx: mpsc::Sender<Event>) {
    let client = app.client.clone();
    tokio::spawn(async move {
        let vmss = match client.show_vmss(&rg, &name).await {
            Ok(v) => v,
            Err(e) => {
                let _ = tx.send(Event::FetchErr(format!("{e:#}"))).await;
                return;
            }
        };
        let vmss_value = vmss.to_flattened_value();

        let orchestration = vmss_value.get("orchestrationMode")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let is_flex = orchestration.eq_ignore_ascii_case("Flexible");
        let vmss_id = vmss_value.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();

        let instances_result: Result<Vec<serde_json::Value>, anyhow::Error> = if is_flex {
            match client.list_vmss_flex_instances(&rg, &vmss_id).await {
                Ok(vms) => {
                    let mut joinset = tokio::task::JoinSet::new();
                    for vm in vms.into_iter() {
                        let client = client.clone();
                        let rg = rg.clone();
                        joinset.spawn(async move {
                            let vm_name = vm.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            let iv = if vm_name.is_empty() {
                                None
                            } else {
                                client.vm_get_instance_view(&rg, &vm_name).await.ok()
                            };
                            normalize_flex_instance(vm, iv)
                        });
                    }
                    let mut out = Vec::new();
                    while let Some(joined) = joinset.join_next().await {
                        if let Ok(v) = joined { out.push(v); }
                    }
                    out.sort_by(|a, b| {
                        a.get("name").and_then(|v| v.as_str()).unwrap_or("")
                            .cmp(b.get("name").and_then(|v| v.as_str()).unwrap_or(""))
                    });
                    Ok(out)
                }
                Err(e) => Err(e),
            }
        } else {
            client.list_vmss_instances(&rg, &name, Some("instanceView")).await
                .map(|items| items.iter().map(|i| i.to_flattened_value()).collect())
        };

        match instances_result {
            Ok(instances) => {
                let _ = tx.send(Event::FetchOk(FetchPayload::VmssDetail {
                    rg, name, vmss: vmss_value, instances, is_flex,
                })).await;
            }
            Err(e) => {
                let _ = tx.send(Event::FetchErr(format!("{e:#}"))).await;
            }
        }
    });
}

fn normalize_flex_instance(vm: serde_json::Value, instance_view: Option<serde_json::Value>) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    let name = vm.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    map.insert("name".into(), serde_json::Value::String(name.clone()));
    let inst_id = vm.pointer("/properties/instanceId")
        .or_else(|| vm.get("instanceId"))
        .and_then(|v| v.as_str())
        .unwrap_or(&name)
        .to_string();
    map.insert("instanceId".into(), serde_json::Value::String(inst_id));

    if let Some(prov) = vm.pointer("/properties/provisioningState").and_then(|v| v.as_str()) {
        map.insert("provisioningState".into(), serde_json::Value::String(prov.to_string()));
    }
    if let Some(iv) = instance_view {
        map.insert("instanceView".into(), iv);
    }
    map.insert("latestModelApplied".into(), serde_json::Value::Null);
    serde_json::Value::Object(map)
}

pub fn spawn_vmss_action(app: &App, op: VmssOp, scope: VmssScope, is_flex: bool, rg: String, name: String, tx: mpsc::Sender<Event>) {
    let client = app.client.clone();
    tokio::spawn(async move {
        let res = match (&scope, is_flex) {
            (VmssScope::All, _) => {
                match op {
                    VmssOp::Start => client.vmss_start(&rg, &name, None).await,
                    VmssOp::Deallocate => client.vmss_deallocate(&rg, &name, None).await,
                    VmssOp::PowerOff => client.vmss_stop(&rg, &name, None).await,
                    VmssOp::Restart => client.vmss_restart(&rg, &name, None).await,
                    VmssOp::Delete => Err(anyhow::anyhow!("delete with scope=ALL is not allowed via TUI")),
                }
            }
            (VmssScope::Selected(targets), false) => {
                let instance_ids: Vec<String> = targets.iter().map(|t| t.instance_id.clone()).collect();
                match op {
                    VmssOp::Start => client.vmss_start(&rg, &name, Some(&instance_ids)).await,
                    VmssOp::Deallocate => client.vmss_deallocate(&rg, &name, Some(&instance_ids)).await,
                    VmssOp::PowerOff => client.vmss_stop(&rg, &name, Some(&instance_ids)).await,
                    VmssOp::Restart => client.vmss_restart(&rg, &name, Some(&instance_ids)).await,
                    VmssOp::Delete => client.vmss_delete_instances(&rg, &name, &instance_ids).await,
                }
            }
            (VmssScope::Selected(targets), true) => {
                let mut joinset = tokio::task::JoinSet::new();
                for target in targets.iter().cloned() {
                    let client = client.clone();
                    let rg = rg.clone();
                    let op = op.clone();
                    joinset.spawn(async move {
                        match op {
                            VmssOp::Start => client.start_vm(&rg, &target.vm_name).await,
                            VmssOp::Deallocate => client.stop_vm(&rg, &target.vm_name, true).await,
                            VmssOp::PowerOff => client.stop_vm(&rg, &target.vm_name, false).await,
                            VmssOp::Restart => client.vm_post_action(&rg, &target.vm_name, "restart").await,
                            VmssOp::Delete => client.vm_delete(&rg, &target.vm_name, false).await,
                        }.map_err(|e| (target.vm_name, e))
                    });
                }
                let mut failures: Vec<String> = Vec::new();
                while let Some(joined) = joinset.join_next().await {
                    match joined {
                        Ok(Ok(())) => {}
                        Ok(Err((vm, e))) => failures.push(format!("{vm}: {e:#}")),
                        Err(e) => failures.push(format!("join error: {e:#}")),
                    }
                }
                if failures.is_empty() {
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("{} of {} instance ops failed:\n{}",
                        failures.len(),
                        match &scope { VmssScope::Selected(t) => t.len(), _ => 0 },
                        failures.join("\n")))
                }
            }
        };

        match res {
            Ok(()) => {
                let _ = tx.send(Event::ActionOk(format!("{} {} accepted", op.verb_ing(), name))).await;
            }
            Err(e) => {
                let _ = tx.send(Event::ActionErr(format!("{} {} failed: {e:#}", op.verb_ing(), name))).await;
            }
        }
    });
}

pub fn spawn_vmss_scale(app: &App, rg: String, name: String, capacity: i64, tx: mpsc::Sender<Event>) {
    let client = app.client.clone();
    tokio::spawn(async move {
        match client.vmss_scale(&rg, &name, capacity).await {
            Ok(()) => {
                let _ = tx.send(Event::ActionOk(format!("scaling {name} to capacity {capacity} accepted"))).await;
            }
            Err(e) => {
                let _ = tx.send(Event::ActionErr(format!("scaling {name} failed: {e:#}"))).await;
            }
        }
    });
}

pub fn spawn_fetch_subscriptions(app: &App, tx: mpsc::Sender<Event>) {
    let sub_override = Some(app.subscription_id.clone());
    tokio::spawn(async move {
        let mut provider = match TokenProvider::load(sub_override) {
            Ok(p) => p,
            Err(e) => {
                let _ = tx.send(Event::FetchErr(format!("Reload provider failed: {e:#}"))).await;
                return;
            }
        };
        match crate::commands::account::list::execute(&mut provider).await {
            Ok(serde_json::Value::Array(rows)) => {
                let _ = tx.send(Event::FetchOk(FetchPayload::Subscriptions(rows))).await;
            }
            Ok(_) => {
                let _ = tx.send(Event::FetchErr("Unexpected non-array from account list".into())).await;
            }
            Err(e) => {
                let _ = tx.send(Event::FetchErr(format!("{e:#}"))).await;
            }
        }
    });
}

pub fn spawn_fetch_pim(app: &App, subscription_id: String, tx: mpsc::Sender<Event>) {
    let client = app.client.clone();
    tokio::spawn(async move {
        match crate::commands::role::pim::list::execute(&client, None, None).await {
            Ok(value) => {
                let eligible = value.get("eligible").and_then(|v| v.as_array()).cloned().unwrap_or_default();
                let active = value.get("active").and_then(|v| v.as_array()).cloned().unwrap_or_default();
                let _ = tx.send(Event::FetchOk(FetchPayload::PimRoles { subscription_id, eligible, active })).await;
            }
            Err(e) => {
                let _ = tx.send(Event::FetchErr(format!("{e:#}"))).await;
            }
        }
    });
}

pub fn spawn_pim_activate(app: &App, role_name: String, justification: String, duration: String, _role_scope: String, tx: mpsc::Sender<Event>) {
    let client = app.client.clone();
    tokio::spawn(async move {
        match crate::commands::role::pim::activate::execute(&client, &role_name, &justification, &duration, None, None).await {
            Ok(_) => {
                let _ = tx.send(Event::ActionOk(format!("PIM '{role_name}' activate accepted"))).await;
            }
            Err(e) => {
                let _ = tx.send(Event::ActionErr(format!("PIM '{role_name}' activate failed: {e:#}"))).await;
            }
        }
    });
}

pub fn spawn_pim_deactivate(app: &App, role_name: String, _role_scope: String, tx: mpsc::Sender<Event>) {
    let client = app.client.clone();
    tokio::spawn(async move {
        match crate::commands::role::pim::deactivate::execute(&client, &role_name, None, None).await {
            Ok(_) => {
                let _ = tx.send(Event::ActionOk(format!("PIM '{role_name}' deactivate accepted"))).await;
            }
            Err(e) => {
                let _ = tx.send(Event::ActionErr(format!("PIM '{role_name}' deactivate failed: {e:#}"))).await;
            }
        }
    });
}
