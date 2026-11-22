use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppEvent {
    pub id: i64,
    pub ts: i64,
    pub repo_id: i64,
    pub kind: String,
    pub payload: serde_json::Value,
}
