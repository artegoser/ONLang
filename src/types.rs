use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Var {
    pub scope: usize,
    pub var_type: VarTypes,
    pub body: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum VarTypes {
    Variable,
    Function,
}
