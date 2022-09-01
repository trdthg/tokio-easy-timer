use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use tokio_easy_timer::prelude::*;

struct Config {
    id: i32,
}

#[tokio::main]
async fn main() {
    let mut scheduler = scheduler::TimingWheelScheduler::new();
    let config = Arc::new(Mutex::new(Config { id: 0 }));
    scheduler.add_ext(config);

    let builder = BaseJob::new().every(10.seconds()).run_async(
        |config: Data<Arc<Mutex<Config>>>| async move {
            if let Ok(mut config) = config.lock() {
                config.id += 1;
            }
        },
    );

    for i in 0..100000 {
        if i % 10000 == 0 {
            println!("{i}");
        }
        scheduler.add_asyncbuilder(builder.clone());
    }

    scheduler.add_ext(std::time::Instant::now());
    scheduler.add_syncbuilder(
        BaseJob::new()
            .since_every(5.seconds(), 10.seconds())
            .run_sync(|config: Data<Arc<Mutex<Config>>>, start: Data<Instant>| {
                if let Ok(config) = config.lock() {
                    println!("[{}] check: {}", start.elapsed().as_secs(), config.id);
                }
            }),
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
