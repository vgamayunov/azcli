mod app;
mod data;
mod event;
mod keys;
mod stderr_capture;
mod ui;

use std::io::IsTerminal;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::{Context, Result, bail};
use crossterm::event::{self as cevent, KeyEventKind};
use tokio::sync::mpsc;

use crate::arm_client::ArmClient;
use crate::auth::TokenProvider;

use app::{App, View};
use event::Event;
use stderr_capture::StderrCapture;

pub async fn run(subscription_override: Option<String>) -> Result<()> {
    if !std::io::stdout().is_terminal() {
        bail!("azcli tui requires a terminal on stdout (cannot pipe/redirect)");
    }

    let mut provider = TokenProvider::load(subscription_override.clone())
        .context("Failed to load token cache")?;

    if provider.cache().accounts.is_empty() {
        bail!("No cached accounts. Run `azcli login` first.");
    }

    let token = provider.get_access_token().await
        .context("Failed to acquire access token")?;
    let sub_id = provider.get_subscription_id_or_fallback().await
        .context("Failed to resolve subscription")?;

    let client = ArmClient::new(token, sub_id.clone());

    let stderr_capture = StderrCapture::install()
        .context("Failed to install stderr capture")?;

    let (event_tx, mut event_rx) = mpsc::channel::<Event>(64);
    let stop = Arc::new(AtomicBool::new(false));

    let key_tx = event_tx.clone();
    let key_stop = stop.clone();
    let key_handle = tokio::task::spawn_blocking(move || {
        loop {
            if key_stop.load(Ordering::Relaxed) {
                return;
            }
            match cevent::poll(std::time::Duration::from_millis(80)) {
                Ok(true) => {
                    let ev = match cevent::read() {
                        Ok(e) => e,
                        Err(_) => return,
                    };
                    let send = match ev {
                        cevent::Event::Key(k) if k.kind == KeyEventKind::Press => Some(Event::Key(k)),
                        cevent::Event::Resize(_, _) => Some(Event::Resize),
                        _ => None,
                    };
                    if let Some(e) = send {
                        if key_tx.blocking_send(e).is_err() {
                            return;
                        }
                    }
                }
                Ok(false) => continue,
                Err(_) => return,
            }
        }
    });

    let mut app = App::new(provider, client, sub_id);
    app.push_view(View::ResourceGroups);
    data::spawn_fetch_current(&app, event_tx.clone());

    let tick_tx = event_tx.clone();
    let tick_stop = stop.clone();
    let tick_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(150));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            if tick_stop.load(Ordering::Relaxed) { return; }
            if tick_tx.send(Event::Tick).await.is_err() { return; }
        }
    });

    let mut terminal = ratatui::init();
    let result = event_loop(&mut terminal, &mut app, &mut event_rx, &event_tx, &stderr_capture).await;
    ratatui::restore();

    stop.store(true, Ordering::Relaxed);
    let _ = key_handle.await;
    tick_handle.abort();

    if let Some(leftover) = stderr_capture.take() {
        eprint!("{leftover}");
    }
    drop(stderr_capture);

    result
}

async fn event_loop(
    terminal: &mut ratatui::DefaultTerminal,
    app: &mut App,
    event_rx: &mut mpsc::Receiver<Event>,
    event_tx: &mpsc::Sender<Event>,
    stderr_capture: &StderrCapture,
) -> Result<()> {
    loop {
        if app.log_modal.is_none() && stderr_capture.peek_nonempty() {
            if let Some(text) = stderr_capture.take() {
                app.log_modal = Some(text);
            }
        }

        terminal.draw(|f| ui::render(f, app)).context("draw failed")?;

        let event = match event_rx.recv().await {
            Some(e) => e,
            None => return Ok(()),
        };

        match event {
            Event::Key(key) => {
                if app.log_modal.is_some() {
                    app.log_modal = None;
                    continue;
                }
                if app.capacity_prompt.is_some() {
                    handle_capacity_key(app, key);
                    continue;
                }
                if app.pim_activate_prompt.is_some() {
                    handle_pim_activate_key(app, key, event_tx).await;
                    continue;
                }
                if let Some(action) = keys::dispatch(app, key) {
                    if app::handle_action(app, action, event_tx).await {
                        return Ok(());
                    }
                }
            }
            Event::Resize => {}
            Event::Tick => {
                let should_refresh = app.pim_state.as_ref()
                    .and_then(|p| p.last_fetched.map(|t| t.elapsed() >= std::time::Duration::from_secs(60)))
                    .unwrap_or(false);
                let pim_sub_matches = app.pim_state.as_ref()
                    .map(|p| p.subscription_id == app.subscription_id)
                    .unwrap_or(false);
                if should_refresh && pim_sub_matches {
                    if let Some(p) = app.pim_state.as_mut() { p.last_fetched = Some(std::time::Instant::now()); }
                    let sub = app.subscription_id.clone();
                    data::spawn_fetch_pim(app, sub, event_tx.clone());
                }
            }
            Event::FetchOk(payload) => app.apply_fetch(payload),
            Event::FetchErr(err) => app.apply_error(err),
            Event::ActionOk(msg) => {
                app.action_in_progress = None;
                app.status = msg;
                match app.current_view().clone() {
                    app::View::VmDetail { rg, name } => {
                        app.vm_detail.loading = true;
                        data::spawn_fetch_vm_detail(app, rg, name, event_tx.clone());
                    }
                    app::View::VmssDetail { rg, name } => {
                        app.vmss_detail.loading = true;
                        data::spawn_fetch_vmss_detail(app, rg, name, event_tx.clone());
                    }
                    app::View::VmssInstanceDetail { rg, vmss, .. } => {
                        app.vmss_detail.loading = true;
                        data::spawn_fetch_vmss_detail(app, rg, vmss, event_tx.clone());
                    }
                    app::View::PimPanel => {
                        if let Some(p) = app.pim_state.as_mut() { p.loading = true; }
                        let sub = app.subscription_id.clone();
                        data::spawn_fetch_pim(app, sub, event_tx.clone());
                    }
                    _ => {}
                }
            }
            Event::ActionErr(msg) => {
                app.action_in_progress = None;
                app.status = msg;
            }
        }
    }
}

fn handle_capacity_key(app: &mut App, key: crossterm::event::KeyEvent) {
    use crossterm::event::KeyCode;
    let Some(prompt) = app.capacity_prompt.as_mut() else { return; };
    match key.code {
        KeyCode::Esc => { app.capacity_prompt = None; }
        KeyCode::Enter => app::submit_capacity_prompt(app),
        KeyCode::Backspace => {
            prompt.input.pop();
            prompt.error = None;
        }
        KeyCode::Char(c) if c.is_ascii_digit() => {
            if prompt.input.len() < 6 {
                prompt.input.push(c);
                prompt.error = None;
            }
        }
        _ => {}
    }
}

async fn handle_pim_activate_key(app: &mut App, key: crossterm::event::KeyEvent, event_tx: &tokio::sync::mpsc::Sender<Event>) {
    use crossterm::event::KeyCode;
    match key.code {
        KeyCode::Esc => { app::handle_action(app, app::Action::PimCancelActivate, event_tx).await; }
        KeyCode::Enter => { app::handle_action(app, app::Action::PimSubmitActivate, event_tx).await; }
        KeyCode::Tab | KeyCode::BackTab => { app::handle_action(app, app::Action::PimTabField, event_tx).await; }
        KeyCode::Backspace => { app::handle_action(app, app::Action::PimInputBackspace, event_tx).await; }
        KeyCode::Char(c) => { app::handle_action(app, app::Action::PimInputChar(c), event_tx).await; }
        _ => {}
    }
}