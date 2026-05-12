use crossterm::event::KeyEvent;

pub enum Event {
    Key(KeyEvent),
    Resize,
    FetchOk(FetchPayload),
    FetchErr(String),
}

pub enum FetchPayload {
    ResourceGroups(Vec<serde_json::Value>),
    ResourcesInGroup { rg: String, items: Vec<serde_json::Value> },
    Subscriptions(Vec<serde_json::Value>),
}
