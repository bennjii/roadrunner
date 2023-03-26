use tokio::sync::broadcast::{Sender, Receiver};
use tokio::sync::broadcast;
use crate::lang::Languages;
use chrono::offset::Utc;
use chrono::DateTime;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Clone)]
pub struct Arguments {
    pub argument_count: i32,
    pub arguments: Vec<String>
}

impl Arguments {
    pub fn parse(argument_string: String) -> Self {
        let arguments: Vec<String> = argument_string.split(" ").into_iter().map(|e| format!("{}", e)).collect();

        Arguments {
            argument_count: arguments.len().try_into().unwrap(),
            arguments
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum TerminalStream {
    StandardInput(String),
    StandardOutput(String),
    StandardError(String),
    EndOfOutput
}

//impl Serialize for TerminalStream {
//    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//    where
//        S: Serializer,
//    {
//        match *self {
//            TerminalStream::StandardInput(x) => serializer.serialize_str(&x),
//            TerminalStream::StandardOutput(x) => serializer.serialize_str(&x),
//            TerminalStream::StandardError(x) => serializer.serialize_str(&x),
//        }
//    }
//}

#[derive(Clone, Debug)]
pub struct TerminalFeed {
    pub std_cout: Vec<TerminalStream>,
    pub std_cin: Vec<TerminalStream>,
    pub std_err: Vec<TerminalStream>
}

#[derive(Clone, Copy)]
pub struct Timing {
    pub time_sent: Option<DateTime<Utc>>,
    pub time_recieved: Option<DateTime<Utc>>,
    pub time_executed: Option<DateTime<Utc>>,
    pub time_completed: Option<DateTime<Utc>>
}

pub struct ExecutorBuilder {
    language: Option<Languages>, // Language
    standard_input: Option<String>, // STDIN
    arguments: Option<String>, // Command-line Arguments
    src_file: Option<String>, // Sourcefile
}

pub struct Executor {
    pub id: Uuid,

    pub language: Languages,
    pub src_file: String,
    pub allocated_dir: String,

    pub terminal_feed: TerminalFeed,
    pub commandline_arguments: Arguments,

    pub timings: Timing,
    pub broadcast: (Sender<TerminalStream>, Receiver<TerminalStream>),

    pub sender_id: Uuid
}

impl ExecutorBuilder {
    pub fn new() -> Self {
        ExecutorBuilder {
            language: None,
            standard_input: None,
            arguments: None,
            src_file: None
        }
    }

    pub fn language(mut self, language: Languages) -> Self {
        self.language = Some(language);
        self
    }

    pub fn src_file(mut self, input: String) -> Self {
        self.src_file = Some(input);
        self
    }

    pub fn arguments(mut self, arguments: String) -> Self {
        self.arguments = Some(arguments);
        self
    }

    pub fn input(mut self, standard_input: String) -> Self {
        self.standard_input = Some(standard_input);
        self
    }

    pub fn build(self, sender_id: Uuid) -> Executor {
        let throughput = broadcast::channel::<TerminalStream>(100);
        let id = Uuid::new_v4();

        Executor {
            id: id,
            broadcast: throughput,
            language: self.language.expect("[BUILDER]: Could not retrieve language, value not set."),
            src_file: self.src_file.expect("[BUILDER]: Could not retrieve source file, value not set."),
            terminal_feed: TerminalFeed {
                std_cout: vec![],
                std_cin: vec![TerminalStream::StandardInput(self.standard_input.unwrap_or(format!("")))],
                std_err: vec![]
            },
            timings: Timing {
                time_sent: None,
                time_recieved: None,
                time_executed: None,
                time_completed: None
            },
            sender_id,
            allocated_dir: format!("{}/{}", sender_id.to_string(), id.to_string()),
            commandline_arguments: Arguments::parse(self.arguments.unwrap_or(format!("")))
        }
    }
}