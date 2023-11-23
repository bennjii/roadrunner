use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::runner::{Runner, RunnerBuilder};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Configuration {
    command: String,
    restart_on_panic: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    restart_limit: Option<i8>
}

impl Configuration {
    pub fn into_exec(self) -> Runner  {
        RunnerBuilder::new()
            .source(self.command)
            .language("shell")
            .build(Uuid::new_v4())
    }
}

#[derive(Debug)]
pub struct ConfigurationGroup {
    pub name: String,
    pub services: HashMap<String, Runner>
}

impl ConfigurationGroup {
    pub fn new(name: &str) -> Self {
        ConfigurationGroup {
            name: name.to_string(),
            services: HashMap::new()
        }
    }

    pub fn add_service(&mut self, name: &str, configuration: Configuration) {
        self.services.insert(name.to_string(), configuration.into_exec());
    }
}

pub type ProgramConfiguration = HashMap<String, Vec<HashMap<String, Configuration>>>;