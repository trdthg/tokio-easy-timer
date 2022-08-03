use std::sync::{Arc, Mutex};

use tokio_easy_timer::prelude::*;

struct Config {
    id: i32,
}

#[tokio::main]
async fn main() {
    let mut cheduler = Scheduler::new();
    let config = Arc::new(Mutex::new(Config { id: 0 }));
    cheduler.add_ext(config);

    for _ in 0..10000 {
        cheduler.add(
            SyncJob::new()
                .every(20.seconds())
                .run(|config: Data<Arc<Mutex<Config>>>| {
                    if let Ok(mut config) = config.lock() {
                        config.id += 1;
                    }
                }),
        );
    }

    cheduler.add(SyncJob::new().since_every(10.seconds(), 20.seconds()).run(
        |config: Data<Arc<Mutex<Config>>>| {
            if let Ok(config) = config.lock() {
                println!("{}", config.id);
            }
        },
    ));

    cheduler.run_pending().await;
}
