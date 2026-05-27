use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize)]
pub struct FridaEnvelope {
    #[serde(rename = "type")]
    pub kind: String,
    pub payload: ArgusEvent,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ArgusEvent {
    pub schema: String,
    pub time: String,
    pub event: ArgusEventKind,
    pub tag: String,
    pub subject: Subject,
    #[serde(default)]
    pub data: Value,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Subject {
    pub name: String,
    pub address: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ArgusEventKind {
    Init,
    Register,
    Collect,
    Triggered,
    Skip,
    Error,

    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize)]
pub struct ArgsData {
    pub args: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct ChangeData {
    pub original: std::collections::BTreeMap<String, String>,
    pub current: std::collections::BTreeMap<String, String>,
}
