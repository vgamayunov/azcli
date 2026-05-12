use tokio::sync::mpsc;

use crate::auth::TokenProvider;

use super::app::{App, View, VmOp};
use super::event::{Event, FetchPayload};

pub fn spawn_fetch_current(app: &App, tx: mpsc::Sender<Event>) {
    match app.current_view() {
        View::ResourceGroups => spawn_fetch_rgs(app, tx),
        View::ResourcesInGroup { rg } => spawn_fetch_resources(app, rg.clone(), tx),
        View::AccountPicker => spawn_fetch_subscriptions(app, tx),
        View::VmDetail { rg, name } => spawn_fetch_vm_detail(app, rg.clone(), name.clone(), tx),
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
