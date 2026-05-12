use tokio::sync::mpsc;

use crate::arm_client::ArmClient;
use crate::auth::TokenProvider;
use crate::auth::cache::CachedAccount;

use super::event::{Event, FetchPayload};

#[derive(Clone)]
pub enum View {
    ResourceGroups,
    ResourcesInGroup { rg: String },
    VmDetail { rg: String, name: String },
    AccountPicker,
}

pub enum Action {
    Quit,
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    End,
    Enter,
    Back,
    Refresh,
    OpenAccountPicker,
    SelectAccount,
    ToggleHelp,
    VmStart,
    VmDeallocate,
    VmPowerOff,
    VmRestart,
    ConfirmYes,
    ConfirmNo,
}

#[derive(Clone)]
pub enum VmOp {
    Start,
    Deallocate,
    PowerOff,
    Restart,
}

impl VmOp {
    pub fn label(&self) -> &'static str {
        match self {
            VmOp::Start => "start",
            VmOp::Deallocate => "deallocate (stop + release compute)",
            VmOp::PowerOff => "power off (stop, keep compute)",
            VmOp::Restart => "restart",
        }
    }
    pub fn verb_ing(&self) -> &'static str {
        match self {
            VmOp::Start => "starting",
            VmOp::Deallocate => "deallocating",
            VmOp::PowerOff => "powering off",
            VmOp::Restart => "restarting",
        }
    }
}

pub struct PendingConfirm {
    pub op: VmOp,
    pub rg: String,
    pub name: String,
}

pub struct VmDetailState {
    pub value: Option<serde_json::Value>,
    pub loading: bool,
    pub error: Option<String>,
}

impl VmDetailState {
    pub fn new() -> Self { Self { value: None, loading: true, error: None } }
}

pub struct ListState {
    pub items: Vec<serde_json::Value>,
    pub cursor: usize,
    pub loading: bool,
    pub error: Option<String>,
}

impl ListState {
    pub fn new() -> Self {
        Self { items: Vec::new(), cursor: 0, loading: true, error: None }
    }

    pub fn move_by(&mut self, delta: isize) {
        if self.items.is_empty() {
            self.cursor = 0;
            return;
        }
        let len = self.items.len() as isize;
        let mut c = self.cursor as isize + delta;
        if c < 0 { c = 0; }
        if c >= len { c = len - 1; }
        self.cursor = c as usize;
    }

    pub fn selected(&self) -> Option<&serde_json::Value> {
        self.items.get(self.cursor)
    }
}

pub struct App {
    pub provider: TokenProvider,
    pub client: ArmClient,
    pub subscription_id: String,
    pub view_stack: Vec<View>,
    pub rg_list: ListState,
    pub resource_list: ListState,
    pub subs_list: ListState,
    pub vm_detail: VmDetailState,
    pub pending_confirm: Option<PendingConfirm>,
    pub action_in_progress: Option<String>,
    pub help_visible: bool,
    pub status: String,
    pub log_modal: Option<String>,
}

impl App {
    pub fn new(provider: TokenProvider, client: ArmClient, subscription_id: String) -> Self {
        Self {
            provider,
            client,
            subscription_id,
            view_stack: Vec::new(),
            rg_list: ListState::new(),
            resource_list: ListState::new(),
            subs_list: ListState::new(),
            vm_detail: VmDetailState::new(),
            pending_confirm: None,
            action_in_progress: None,
            help_visible: false,
            status: String::new(),
            log_modal: None,
        }
    }

    pub fn current_view(&self) -> &View {
        self.view_stack.last().expect("view stack never empty after push_view")
    }

    pub fn push_view(&mut self, v: View) {
        self.view_stack.push(v);
    }

    pub fn pop_view(&mut self) {
        if self.view_stack.len() > 1 {
            self.view_stack.pop();
        }
    }

    pub fn current_list_mut(&mut self) -> Option<&mut ListState> {
        match self.current_view() {
            View::ResourceGroups => Some(&mut self.rg_list),
            View::ResourcesInGroup { .. } => Some(&mut self.resource_list),
            View::AccountPicker => Some(&mut self.subs_list),
            View::VmDetail { .. } => None,
        }
    }

    pub fn apply_fetch(&mut self, payload: FetchPayload) {
        match payload {
            FetchPayload::ResourceGroups(items) => {
                self.rg_list.items = items;
                self.rg_list.loading = false;
                self.rg_list.error = None;
                if self.rg_list.cursor >= self.rg_list.items.len() {
                    self.rg_list.cursor = 0;
                }
            }
            FetchPayload::ResourcesInGroup { rg, items } => {
                if let Some(View::ResourcesInGroup { rg: cur }) = self.view_stack.last() {
                    if *cur == rg {
                        self.resource_list.items = items;
                        self.resource_list.loading = false;
                        self.resource_list.error = None;
                        self.resource_list.cursor = 0;
                    }
                }
            }
            FetchPayload::Subscriptions(items) => {
                self.subs_list.items = items;
                self.subs_list.loading = false;
                self.subs_list.error = None;
                self.subs_list.cursor = self.subs_list.items.iter()
                    .position(|v| v.get("id").and_then(|s| s.as_str()) == Some(&self.subscription_id))
                    .unwrap_or(0);
            }
            FetchPayload::VmDetail { rg, name, value } => {
                if let Some(View::VmDetail { rg: cur_rg, name: cur_name }) = self.view_stack.last() {
                    if *cur_rg == rg && *cur_name == name {
                        self.vm_detail.value = Some(value);
                        self.vm_detail.loading = false;
                        self.vm_detail.error = None;
                    }
                }
            }
        }
    }

    pub fn apply_error(&mut self, msg: String) {
        match self.current_view() {
            View::ResourceGroups => {
                self.rg_list.loading = false;
                self.rg_list.error = Some(msg);
            }
            View::ResourcesInGroup { .. } => {
                self.resource_list.loading = false;
                self.resource_list.error = Some(msg);
            }
            View::AccountPicker => {
                self.subs_list.loading = false;
                self.subs_list.error = Some(msg);
            }
            View::VmDetail { .. } => {
                self.vm_detail.loading = false;
                self.vm_detail.error = Some(msg);
            }
        }
    }

    pub fn clear_caches(&mut self) {
        self.rg_list = ListState::new();
        self.resource_list = ListState::new();
    }
}

pub async fn handle_action(app: &mut App, action: Action, event_tx: &mpsc::Sender<Event>) -> bool {
    if app.action_in_progress.is_some() {
        return false;
    }

    if app.pending_confirm.is_some() {
        match action {
            Action::ConfirmYes => confirm_yes(app, event_tx),
            Action::ConfirmNo | Action::Back | Action::Quit => { app.pending_confirm = None; }
            _ => {}
        }
        return false;
    }

    match action {
        Action::Quit => return true,
        Action::Up => { if let Some(l) = app.current_list_mut() { l.move_by(-1); } }
        Action::Down => { if let Some(l) = app.current_list_mut() { l.move_by(1); } }
        Action::PageUp => { if let Some(l) = app.current_list_mut() { l.move_by(-10); } }
        Action::PageDown => { if let Some(l) = app.current_list_mut() { l.move_by(10); } }
        Action::Home => { if let Some(l) = app.current_list_mut() { l.cursor = 0; } }
        Action::End => { if let Some(l) = app.current_list_mut() { if !l.items.is_empty() { l.cursor = l.items.len() - 1; } } }
        Action::Enter => enter(app, event_tx),
        Action::Back => back(app),
        Action::Refresh => refresh(app, event_tx),
        Action::OpenAccountPicker => open_account_picker(app, event_tx),
        Action::SelectAccount => select_account(app, event_tx).await,
        Action::ToggleHelp => app.help_visible = !app.help_visible,
        Action::VmStart => request_vm_op(app, VmOp::Start),
        Action::VmDeallocate => request_vm_op(app, VmOp::Deallocate),
        Action::VmPowerOff => request_vm_op(app, VmOp::PowerOff),
        Action::VmRestart => request_vm_op(app, VmOp::Restart),
        Action::ConfirmYes | Action::ConfirmNo => {}
    }
    false
}

fn enter(app: &mut App, event_tx: &mpsc::Sender<Event>) {
    match app.current_view() {
        View::ResourceGroups => {
            let Some(sel) = app.rg_list.selected() else { return; };
            let Some(name) = sel.get("name").and_then(|v| v.as_str()) else { return; };
            let rg = name.to_string();
            app.resource_list = ListState::new();
            app.push_view(View::ResourcesInGroup { rg: rg.clone() });
            super::data::spawn_fetch_resources(app, rg, event_tx.clone());
        }
        View::ResourcesInGroup { rg } => {
            let rg = rg.clone();
            let Some(sel) = app.resource_list.selected() else { return; };
            let ty = sel.get("type").and_then(|v| v.as_str()).unwrap_or("");
            let name = match sel.get("name").and_then(|v| v.as_str()) {
                Some(n) => n.to_string(),
                None => return,
            };
            if ty.eq_ignore_ascii_case("Microsoft.Compute/virtualMachines") {
                app.vm_detail = VmDetailState::new();
                app.push_view(View::VmDetail { rg: rg.clone(), name: name.clone() });
                super::data::spawn_fetch_vm_detail(app, rg, name, event_tx.clone());
            }
        }
        View::VmDetail { .. } => {}
        View::AccountPicker => {}
    }
}

fn back(app: &mut App) {
    match app.current_view() {
        View::AccountPicker => app.pop_view(),
        View::VmDetail { .. } => app.pop_view(),
        View::ResourcesInGroup { .. } => app.pop_view(),
        View::ResourceGroups => {}
    }
}

fn refresh(app: &mut App, event_tx: &mpsc::Sender<Event>) {
    match app.current_view() {
        View::ResourceGroups => {
            app.rg_list.loading = true;
            app.rg_list.error = None;
            super::data::spawn_fetch_rgs(app, event_tx.clone());
        }
        View::ResourcesInGroup { rg } => {
            let rg = rg.clone();
            app.resource_list.loading = true;
            app.resource_list.error = None;
            super::data::spawn_fetch_resources(app, rg, event_tx.clone());
        }
        View::VmDetail { rg, name } => {
            let rg = rg.clone();
            let name = name.clone();
            app.vm_detail.loading = true;
            app.vm_detail.error = None;
            super::data::spawn_fetch_vm_detail(app, rg, name, event_tx.clone());
        }
        View::AccountPicker => {
            app.subs_list = ListState::new();
            super::data::spawn_fetch_subscriptions(app, event_tx.clone());
        }
    }
}

fn request_vm_op(app: &mut App, op: VmOp) {
    let (rg, name) = match app.current_view() {
        View::VmDetail { rg, name } => (rg.clone(), name.clone()),
        _ => return,
    };
    app.pending_confirm = Some(PendingConfirm { op, rg, name });
}

fn confirm_yes(app: &mut App, event_tx: &mpsc::Sender<Event>) {
    let Some(pc) = app.pending_confirm.take() else { return; };
    app.action_in_progress = Some(format!("{} {}", pc.op.verb_ing(), pc.name));
    super::data::spawn_vm_action(app, pc.op, pc.rg, pc.name, event_tx.clone());
}

fn open_account_picker(app: &mut App, event_tx: &mpsc::Sender<Event>) {
    app.push_view(View::AccountPicker);
    if app.subs_list.items.is_empty() && app.subs_list.error.is_none() {
        app.subs_list = ListState::new();
        super::data::spawn_fetch_subscriptions(app, event_tx.clone());
    }
}

async fn select_account(app: &mut App, event_tx: &mpsc::Sender<Event>) {
    let Some(sub_row) = app.subs_list.selected().cloned() else { return; };
    let Some(new_sub_id) = sub_row.get("id").and_then(|v| v.as_str()).map(str::to_string) else {
        app.status = "Selected row has no id".into();
        return;
    };
    let new_sub_name = sub_row.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let new_tenant_id = sub_row.get("tenantId").and_then(|v| v.as_str()).unwrap_or("").to_string();

    if new_sub_id == app.subscription_id {
        app.pop_view();
        return;
    }

    let existing = app.provider.cache().accounts.iter()
        .find(|a| a.subscription_id.as_deref() == Some(&new_sub_id))
        .cloned();

    if existing.is_none() {
        let home = match app.provider.cache().active_account().cloned() {
            Some(a) => a,
            None => { app.status = "No active account to clone from".into(); return; }
        };
        let new_account = CachedAccount {
            auth_method: home.auth_method.clone(),
            tenant_id: if new_tenant_id.is_empty() { home.tenant_id.clone() } else { new_tenant_id },
            subscription_id: Some(new_sub_id.clone()),
            subscription_name: Some(new_sub_name),
            profile: None,
            access_token: None,
            refresh_token: home.refresh_token.clone(),
            expires_at: None,
            client_id: home.client_id.clone(),
            client_secret: home.client_secret.clone(),
            client_certificate_path: home.client_certificate_path.clone(),
            managed_identity_client_id: home.managed_identity_client_id.clone(),
        };
        app.provider.cache_mut().set_account(new_account);
    } else {
        app.provider.cache_mut().default_subscription = Some(new_sub_id.clone());
    }

    if let Err(e) = app.provider.save_cache() {
        app.status = format!("Save cache failed: {e}");
        return;
    }

    let mut new_provider = match TokenProvider::load(Some(new_sub_id.clone())) {
        Ok(p) => p,
        Err(e) => { app.status = format!("Reload failed: {e}"); return; }
    };

    let token = match new_provider.get_access_token().await {
        Ok(t) => t,
        Err(e) => { app.status = format!("Token refresh failed: {e:#}"); return; }
    };

    app.provider = new_provider;
    app.client = ArmClient::new(token, new_sub_id.clone());
    app.subscription_id = new_sub_id;
    app.pop_view();
    app.clear_caches();
    super::data::spawn_fetch_rgs(app, event_tx.clone());
}
