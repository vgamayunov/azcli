use tokio::sync::mpsc;

use crate::auth::TokenProvider;

use super::app::{App, View};
use super::event::{Event, FetchPayload};

pub fn spawn_fetch_current(app: &App, tx: mpsc::Sender<Event>) {
    match app.current_view() {
        View::ResourceGroups => spawn_fetch_rgs(app, tx),
        View::ResourcesInGroup { rg } => spawn_fetch_resources(app, rg.clone(), tx),
        View::AccountPicker => spawn_fetch_subscriptions(app, tx),
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
