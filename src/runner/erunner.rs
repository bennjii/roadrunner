use crate::exec::Executor;
use crate::exec::ExecutorBuilder;
use crate::lang::Languages;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use uuid::Uuid;
use warp::ws::Message;

use tokio::sync::mpsc::UnboundedSender;

pub type Locked<T> = Arc<Mutex<T>>;

#[derive(Clone)]
pub struct Client {
    pub id: Uuid,
    pub job_history: Vec<Runner>,
    pub sender: UnboundedSender<Message>,
}

impl Client {
    pub fn new(sender: UnboundedSender<Message>) -> Self {
        Client {
            id: Uuid::new_v4(),
            job_history: vec![],
            sender,
        }
    }
}

pub struct GlobalState {
    pub task_queue: Locked<VecDeque<Locked<Executor>>>,
    pub runners: Locked<HashMap<String, Runner>>,
    pub clients: Locked<HashMap<String, Client>>,
    pub runtime: Locked<Runtime>,
}

impl GlobalState {
    pub fn initialize() -> Self {
        GlobalState {
            task_queue: Arc::new(Mutex::new(VecDeque::new())),
            runners: Arc::new(Mutex::new(HashMap::new())),
            clients: Arc::new(Mutex::new(HashMap::new())),
            runtime: Arc::new(Mutex::new(Runtime::new().unwrap())),
        }
    }
}

#[derive(Clone)]
pub struct Runner {
    pub id: Uuid,
    pub nonce: String,

    pub source: String,
    pub language: Languages,

    pub commandline_arguments: String,
    pub standard_input: String,

    pub requestee: Uuid,
    pub executor: Option<Uuid>, // Id
}

impl Runner {
    pub fn batch(self) -> Executor {
        ExecutorBuilder::new()
            .language(self.language)
            .input(self.standard_input)
            .src_file(self.source)
            .arguments(self.commandline_arguments)
            .nonce(self.nonce)
            .build(self.requestee)
    }
}

#[derive(Serialize, Deserialize)]
pub struct ExecutePacket {
    pub source: String,
    pub language: String,

    pub nonce: String,

    pub commandline_arguments: Option<String>,
    pub standard_input: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct RunnerBuilder {
    pub id: Uuid,
    pub nonce: Option<String>,

    pub source: Option<String>,
    pub language: Option<Languages>,

    pub commandline_arguments: Option<String>,
    pub standard_input: Option<String>,

    pub requestee: Option<Uuid>,
    pub executor: Option<Uuid>, // Id
}

impl RunnerBuilder {
    pub fn new() -> Self {
        RunnerBuilder {
            id: Uuid::new_v4(),
            nonce: None,
            source: None,
            language: None,
            commandline_arguments: None,
            standard_input: None,
            requestee: None,
            executor: None,
        }
    }

    pub fn source(mut self, source: String) -> Self {
        self.source = Some(source);
        self
    }

    pub fn nonce(mut self, nonce: String) -> Self {
        self.nonce = Some(nonce);
        self
    }

    pub fn language(mut self, language: &str) -> Self {
        self.language = Some(Languages::from_string(language));
        self
    }

    pub fn arguments(mut self, commandline_arguments: Option<String>) -> Self {
        self.commandline_arguments = commandline_arguments;
        self
    }

    pub fn input(mut self, input: Option<String>) -> Self {
        self.standard_input = input;
        self
    }

    pub fn build(self, requestee: Uuid) -> Runner {
        Runner {
            id: self.id,
            nonce: self.nonce.unwrap_or(String::new()),

            source: self
                .source
                .expect("[RUNNER-BUILD]: Expected value \"source\" to be non-null"),
            language: self
                .language
                .expect("[RUNNER-BUILD]: Expected value \"language\" to be non-null"),

            commandline_arguments: self.commandline_arguments.unwrap_or(String::new()),
            standard_input: self.standard_input.unwrap_or(String::new()),

            requestee,
            executor: None, // Has not been assigned an executor yet!
        }
    }
}
