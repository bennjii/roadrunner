use crate::lang::Languages;
use crate::exec::Executor;
use crate::exec::ExecutorBuilder;
use std::collections::{VecDeque, HashMap};
use tokio::sync::Mutex;
use std::sync::Arc;
use warp::ws::Message;
use uuid::Uuid;

use tokio::sync::mpsc::UnboundedSender;

pub type Locked<T> = Arc<Mutex<T>>;

pub struct GlobalState {
    pub task_queue: Locked<VecDeque<Locked<Executor>>>,
    pub runners: Locked<HashMap<String, Runner>>,
}

impl GlobalState {
    pub fn initialize() -> Self {
        GlobalState {
            task_queue: Arc::new(Mutex::new(VecDeque::new())),
            runners: Arc::new(Mutex::new(HashMap::new()))
        }
    }
}

pub struct Runner {
    pub id: Uuid,

    pub source: String,
    pub language: Languages,

    pub commandline_arguments: String,
    pub standard_input: String,

    pub requestee: String,
    pub executor: String, // Id

    pub sender: Option<UnboundedSender<Message>>
}

impl Runner {
    pub fn batch(mut self) -> Executor {
        let executor = ExecutorBuilder::new()
            .language(self.language)
            .input(self.standard_input)
            .src_file(self.source)
            .arguments(self.commandline_arguments)
            .build(self.id);

        executor
    }
}