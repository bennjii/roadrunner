mod lang;
mod runner;
mod exec;
mod pool;

pub use chrono;
use pool::Pool;
use runner::{GlobalState, Locked};

use std::sync::Arc;
use tokio::sync::{Mutex};

#[tokio::main]
async fn main() {
    let config: Locked<GlobalState> = Arc::new(
            Mutex::new(
                GlobalState::initialize()
        )
    );

    Pool::new()
        .begin(config)
        .await
}