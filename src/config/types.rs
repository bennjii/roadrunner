use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Configuration {
    command: String,
    restart_on_panic: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    restart_limit: Option<i8>
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ConfigurationGroup {
    name: String,
    services: Vec<HashMap<String, Configuration>>
}

pub type ProgramConfiguration = HashMap<String, Vec<HashMap<String, Configuration>>>;