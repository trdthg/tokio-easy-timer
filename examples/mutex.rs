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
    let c2 = config.clone();
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

    let start_time = std::time::Instant::now();
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_millis(1100));
        println!(
            "[{}] check: {}",
            start_time.elapsed().as_secs(),
            c2.lock().unwrap().id
        );
    });
    // scheduler.add_ext(std::time::Instant::now());
    // scheduler.add_syncbuilder(
    //     BaseJob::new()
    //         .since_every(5.seconds(), 10.seconds())
    //         .run_sync(|config: Data<Arc<Mutex<Config>>>, start: Data<Instant>| {
    //             if let Ok(config) = config.lock() {
    //                 println!("[{}] check: {}", start.elapsed().as_secs(), config.id);
    //             }
    //         }),
    // );

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
