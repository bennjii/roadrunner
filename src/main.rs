mod exec;
mod lang;
mod pool;
mod runner;
mod ws;

pub use chrono;
use pool::Pool;
use runner::{GlobalState, Locked};
use serde_json::from_str;

use std::{convert::Infallible, sync::Arc};
use tokio::sync::Mutex;
use warp::Filter;
use clap::{Parser};
#[derive(Parser, Debug)]
#[command(name = "roadrunner", about = "Code execution and orchestration engine")]
struct Args {
    /// Sets the configuration file (Relative or Absolute Path)
    #[clap(short, long, value_name = "FILE")]
    config: Option<String>,

    /// Blacklist categories (Comma Separated)
    #[clap(long, value_name = "CATEGORIES")]
    exclude: Option<Vec<String>>,

    /// Whitelist categories (Comma Separated)
    #[clap(long, value_name = "CATEGORIES")]
    include: Option<Vec<String>>,
}

#[tokio::main]
async fn main() {
    let config: Locked<GlobalState> = Arc::new(Mutex::new(GlobalState::initialize()));
    dotenv::dotenv().ok();

    let args = Args::parse();

    if args.config.is_some() {
        let config_file = args.config;

        println!("cfg; {}", config_file.unwrap());

        if let Some(excluded_categories) = args.exclude {
            println!("Excluding categories: {:?}", excluded_categories);
            // Handle excluding categories
        }

        if let Some(included_categories) = args.include {
            println!("Including categories: {:?}", included_categories);
            // Handle including categories
        }

        println!("Loading configuration.");
        loop {}
    } else {
        start_webserver(config).await;
    }
}

async fn start_webserver(config: Locked<GlobalState>) {
    let ws_route = warp::path::path("ws")
        .and(warp::ws())
        .and(with_config(config.clone()))
        .and_then(ws::ws_handler);

    let echo_route = warp::path::end().and(warp::get()).and_then(ws::echo);

    tokio::spawn(async move { Pool::new().begin(config).await });

    let routes = ws_route
        .or(echo_route)
        .with(warp::cors().allow_any_origin());

    let port: u16 = from_str::<u16>(&dotenv::var("PORT").unwrap()).unwrap();

    println!("Deploying on 0.0.0.0:{}", port);

    warp::serve(routes)
        .run(([0, 0, 0, 0], port))
        .await;
}

fn with_config(
    config: Locked<GlobalState>,
) -> impl Filter<Extract = (Locked<GlobalState>,), Error = Infallible> + Clone {
    warp::any().map(move || config.clone())
}
