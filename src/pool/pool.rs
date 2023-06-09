use crate::runner::{Locked, GlobalState};
use chrono::Utc;
use futures_timer::Delay;
use tokio::sync::mpsc::UnboundedSender;
use std::time::Duration;
use crate::exec::{Executor, TerminalStream, TerminalFeed, TerminalStreamType};
use crate::lang::{Languages};
use warp::ws::Message;

#[derive(Copy, Clone)]
pub struct Pool {

}

impl Pool {
    pub fn new() -> Self {
        Pool {  }
    }

    pub async fn begin(self, config: Locked<GlobalState>) {
        tokio::spawn(async move {
            loop {
                let conf_clone = config.clone();

                match tokio::spawn(async move {
                    let config_lock = conf_clone.lock().await;
                    let mut task_queue = config_lock.task_queue.lock().await;

                    if let Some(task) = task_queue.pop_front() {
                        // Have some task to perform.
                        println!("[POOL]: Found task to perform");

                        drop(task_queue);
                        
                        let sid = task.lock().await.sender_id.clone();
                        let sender = config_lock
                            .clients.lock().await
                            .get(&sid.to_string()).unwrap()
                            .sender.clone();

                        println!("[POOL]: Got sender, starting!");

                        let res = self.execute(task.clone(), sender).await;
                        println!("[POOL]: Ended with output, {:?}", res);
                        task.lock().await.terminal_feed = res;
                    }else {
                        // Sleep Queue
                        Delay::new(Duration::from_millis(1000)).await;
                        println!("{}: Completed wait.", Utc::now().to_rfc3339());
                    }
                }).await {
                    Ok(_a) => {}
                    Err(_e) => {}
                }
            }
        }).await.unwrap();
    }

    pub async fn execute(&self, locked_task: Locked<Executor>, sender: UnboundedSender<Message>) -> TerminalFeed {
        let mut tx2 = locked_task.lock().await.broadcast.0.clone().subscribe();
        println!("[EXEC]: Performing task from sender");

        let file_dir = locked_task.lock().await.allocated_dir.clone();
        // Template create all the directories necessary
        match std::fs::create_dir_all(&file_dir) {
            Ok(_) => {},
            Err(err) => println!("[POOL]: Failed to create directory, {}", err.to_string()),
        }

        let sender_id = locked_task.lock().await.sender_id.clone();
        drop(sender_id);

        tokio::spawn(async move {
            let unlkd = locked_task.lock().await;
            let bcst = unlkd.broadcast.0.clone();
            let nonce = unlkd.nonce.clone();
            let name = unlkd.id.clone();

            let opt = match Languages::run(unlkd) {
                Ok(val) => {
                    println!("[PROG:{}]: Completed Execution.", name);
                    bcst.send(TerminalStream::new_output(TerminalStreamType::EndOfOutput, val, nonce)).unwrap();
                    Ok(val)
                }
                Err(err) => {
                    println!("[PROG:{}]: Runtime Error {:?}", name, err);
                    bcst.send(TerminalStream::new(TerminalStreamType::EndOfOutput, err.clone().to_string(), nonce)).unwrap();
                    Err(err)
                }
            };

            opt
        });

        // We can listen to the stream of inputs/outputs
        // let feed_handler = tokio::spawn(async move {
        let mut feed = TerminalFeed {
            std_cout: vec![],
            std_cin: vec![],
            std_err: vec![],
            output: vec![]
        };

        loop {
            match tx2.recv().await {
                Ok(terminal_stream) => {
                    // Send to websocket listener
                    let as_string = serde_json::to_string(&terminal_stream).unwrap();
                    println!("[STREAM]: Sending value, '{}' to ws", as_string);
                    sender.send(Message::text(as_string)).unwrap();

                    // Push into logs
                    match terminal_stream.terminal_type {
                        TerminalStreamType::StandardOutput => {
                            feed.std_cout.push(terminal_stream);
                        },
                        TerminalStreamType::StandardError => {
                            feed.std_err.push(terminal_stream);
                        },
                        TerminalStreamType::StandardInput => {
                            feed.std_cin.push(terminal_stream);
                        },
                        TerminalStreamType::EndOfOutput => {
                            feed.output.push(terminal_stream);
                            break;
                        }
                    };
                }
                Err(_) => {
                    println!("[TERM]: Poor receive.");
                    // No new input
                }
            };
        }

        match std::fs::remove_dir_all(file_dir) {
            Ok(_) => println!("[POOL]: Cleaned Directory after execution"),
            Err(_) => {}
        }

        feed
    }
}