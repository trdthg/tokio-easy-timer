use std::sync::{Arc, Mutex};

use tokio_easy_timer::prelude::*;

struct Config {
    id: i32,
}

#[tokio::main]
async fn main() {
    let mut sheduler = HeapScheduler::new();
    let config = Arc::new(Mutex::new(Config { id: 0 }));
    sheduler.add_ext(config);

    let job =
        AsyncJob::new()
            .every(10.seconds())
            .run(|config: Data<Arc<Mutex<Config>>>| async move {
                if let Ok(mut config) = config.lock() {
                    config.id += 1;
                }
            });

    for _ in 0..1000000 {
        sheduler.add(job.box_clone());
    }

    sheduler.add(SyncJob::new().since_every(5.seconds(), 10.seconds()).run(
        |config: Data<Arc<Mutex<Config>>>| {
            if let Ok(config) = config.lock() {
                println!("check: {}", config.id);
            }
        },
    ));

    println!("{}", "开始");

    sheduler.run_pending().await;
}

#[test]
fn add_to_1000000() {
    let sum = Mutex::new(0);
    for _ in 0..10 {
        for i in 0..1000000 {
            *sum.lock().unwrap() += 1;
        }
    }
    println!("{}", *sum.lock().unwrap());
}
