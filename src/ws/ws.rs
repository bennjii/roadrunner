use std::sync::Arc;

use crate::{
    lang::RuntimeError,
    runner::{Client, ExecutePacket, GlobalState, Locked, RunnerBuilder},
};
use futures::{SinkExt, StreamExt};
use tokio::sync::{
    mpsc::{self},
    Mutex,
};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::{
    ws::{Message, WebSocket},
    Rejection, Reply,
};

type Result<T> = std::result::Result<T, Rejection>;

pub async fn ws_handler(ws: warp::ws::Ws, config: Locked<GlobalState>) -> Result<impl Reply> {
    Ok(ws.on_upgrade(move |socket| client_connection(socket, config)))
}

async fn client_connection(ws: WebSocket, config: Locked<GlobalState>) {
    let (mut client_ws_sender, mut client_ws_rcv) = ws.split();
    let (client_sender, client_rcv) = mpsc::unbounded_channel();

    let mut client_rcv = UnboundedReceiverStream::new(client_rcv);
    // Now that we have a client connection, we can listen
    // for requests and push to the pool of jobs.

    let client = Client::new(client_sender);
    let id = &client.id.to_string();

    config
        .lock()
        .await
        .clients
        .lock()
        .await
        .insert(id.to_string(), client.clone());

    tokio::spawn(async move {
        while let Some(value) = client_rcv.next().await {
            client_ws_sender.send(value).await.unwrap();
        }
    });

    while let Some(result) = client_ws_rcv.next().await {
        let msg = match result {
            Ok(msg) => {
                if !msg.is_close() {
                    msg
                } else {
                    continue;
                }
            }
            Err(e) => {
                println!("[err]: Receiving message for id {}: {:?}", id, e);
                continue;
            }
        };

        client_msg(client.clone(), msg, &config).await;
    }

    if std::fs::remove_dir_all(format!("jobs/{}", id)).is_ok() {
        println!("[POOL]: Cleaned Directory for user-leave")
    }

    config.lock().await.clients.lock().await.remove(id);
}

async fn client_msg(client: Client, msg: Message, config: &Locked<GlobalState>) {
    // Expect message to be of type -> Insert Runner
    let string = msg.to_str().unwrap();

    let packet: ExecutePacket = match serde_json::from_str(string) {
        Ok(val) => val,
        Err(err) => {
            println!("[WS]: Error parsing input, {}", err);
            client
                .sender
                .send(Message::text(
                    serde_json::to_string(&RuntimeError::ParseInput(err.to_string())).unwrap(),
                ))
                .unwrap();

            return;
        }
    };

    let runner = RunnerBuilder::new()
        .arguments(packet.commandline_arguments)
        .input(packet.standard_input)
        .language(&packet.language)
        .source(packet.source)
        .nonce(packet.nonce)
        .build(client.id);

    let executor = runner.batch();
    config
        .lock()
        .await
        .task_queue
        .lock()
        .await
        .push_back(Arc::new(Mutex::new(executor)));
}
