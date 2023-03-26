use crate::runner::{Locked, GlobalState};
use futures_timer::Delay;
use std::time::Duration;
use crate::exec::{Executor, TerminalStream, TerminalFeed};
use crate::lang::Languages;
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
                        let res = self.execute(task.clone(), conf_clone.clone()).await;
                        task.lock().await.terminal_feed = res;
                    }else {
                        // Sleep Queue
                        Delay::new(Duration::from_millis(100)).await;
                    }
                }).await {
                    Ok(_a) => {}
                    Err(_e) => {}
                }
            }
        }).await.unwrap();
    }

    pub async fn execute(&self, locked_task: Locked<Executor>, config: Locked<GlobalState>) -> TerminalFeed {
        let mut tx2 = locked_task.lock().await.broadcast.0.subscribe();

        let sender_id = locked_task.lock().await.sender_id.clone();
        let sender = config.lock().await.runners.lock().await.get(&sender_id.to_string()).unwrap().sender.clone();

        drop(sender_id);

        let spwn = tokio::spawn(async move {
            let unlkd = locked_task.lock().await;
            let name = unlkd.id.clone();

            match Languages::run(unlkd) {
                Ok(_) => {
                    println!("[PROG:{}]: Completed Execution.", name);
                }
                Err(err) => {
                    println!("[PROG:{}]: Runtime Error {:?}", name, err);
                }
            }
        });

        // We can listen to the stream of inputs/outputs
        let result_feed: TerminalFeed = tokio::spawn(async move {
            let mut feed = TerminalFeed {
                std_cout: vec![],
                std_cin: vec![],
                std_err: vec![]
            };

            while !spwn.is_finished() {
                match tx2.recv().await {
                    Ok(terminal_stream) => {
                        // Send to websocket listener
                        let as_string = serde_json::to_string(&terminal_stream).unwrap();
                        sender.clone().unwrap().send(Message::text(as_string)).unwrap();

                        // Push into logs
                        match terminal_stream {
                            TerminalStream::StandardOutput(val) => {
                                feed.std_cout.push(TerminalStream::StandardOutput(val))
                            }
                            TerminalStream::StandardError(val) => {
                                feed.std_err.push(TerminalStream::StandardError(val))
                            }
                            TerminalStream::StandardInput(val) => {
                                feed.std_cin.push(TerminalStream::StandardInput(val))
                            }
                        };
                    }
                    Err(_) => {
                        println!("[TERM]: Poor recieve.")
                        // No new input
                    }
                }
            }

            feed
        }).await.unwrap();

        result_feed
    }
}