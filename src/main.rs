mod ws;
mod lang;
mod runner;
mod exec;
mod pool;

pub use chrono;
use pool::Pool;
use runner::{GlobalState, Locked};
use serde_json::from_str;

use std::{sync::Arc, convert::Infallible};
use tokio::sync::{Mutex};
use warp::{Filter};

#[tokio::main]
async fn main() {
    let config: Locked<GlobalState> = Arc::new(
            Mutex::new(
                GlobalState::initialize()
        )
    );

    let ws_route = warp::path::path("ws")
        .and(warp::ws())
        .and(with_config(config.clone()))
        .and_then(ws::ws_handler);

    let echo_route = warp::path::end().and(warp::get()).and_then(ws::echo);

    tokio::spawn(async move {
        Pool::new()
            .begin(config)
            .await
    });

    let routes = ws_route
        .or(echo_route)
        .with(warp::cors().allow_any_origin());

    dotenv::dotenv().ok();
    let certificate = dotenv::var("CERTIFICATE").unwrap();
    let private_key = dotenv::var("PRIVATE_KEY").unwrap();
    let port: u16 = from_str::<u16>(&dotenv::var("PORT").unwrap()).unwrap();

    warp::serve(routes)
        .tls()
        .cert(certificate)
        .key(private_key)
        // .cert_path("/run/secrets/certificate")
        // .key_path("/run/secrets/private_key")
        .run(([0, 0, 0, 0], port))
        .await;
}

fn with_config(
    config: Locked<GlobalState>,
) -> impl Filter<Extract = (Locked<GlobalState>,), Error = Infallible> + Clone {
    warp::any().map(move || config.clone())
}