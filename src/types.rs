use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Var {
    pub scope: usize,
    pub body: serde_json::Value,
}
