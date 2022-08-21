use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use chrono::TimeZone;
use tokio_easy_timer::{prelude::*, BaseJob};

struct Config {
    id: i32,
}

#[tokio::main]
async fn main() {
    let mut scheduler = scheduler::JobScheduler::new();
    let config = Arc::new(Mutex::new(Config { id: 0 }));
    scheduler.add_ext(config);

    for _ in 0..800000 {
        scheduler.add(BaseJob::new().every(10.seconds()).run_async(
            |config: Data<Arc<Mutex<Config>>>| async move {
                if let Ok(mut config) = config.lock() {
                    config.id += 1;
                }
            },
        ));
    }

    scheduler.add_ext(std::time::Instant::now());
    scheduler.add(
        BaseJob::new()
            .since_every(5.seconds(), 10.seconds())
            .run_async(
                |config: Data<Arc<Mutex<Config>>>, start: Data<Instant>| async move {
                    if let Ok(config) = config.lock() {
                        println!("[{}] check: {}", start.elapsed().as_secs(), config.id);
                    }
                },
            ),
    );

    println!("{}", "开始");

    scheduler.run_pending().await;
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
