pub mod invoke;
pub mod list;
pub mod show;
pub mod create;
pub mod update;
pub mod delete;

pub fn parse_params(params: &[String]) -> Vec<serde_json::Value> {
    params.iter().filter_map(|kv| {
        kv.split_once('=').map(|(k, v)| serde_json::json!({ "name": k, "value": v }))
    }).collect()
}
