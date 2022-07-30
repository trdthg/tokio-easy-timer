mod extensions;
pub mod interval;
mod job;
mod scheduler;
use extensions::Data;
use interval::TimeUnits;
use job::SyncJob;
use parking_lot::Mutex;
use scheduler::Sheduler;
use std::{fmt::Debug, sync::Arc};

#[derive(Default, Debug)]
struct Config {
    id: i32,
}

#[derive(Default, Debug)]
struct Database;

#[tokio::main]
async fn main() {
    let mut sheduler = Sheduler::new();

    let config = Arc::new(Mutex::new(Config { id: 1 }));
    sheduler.add_ext(config);
    sheduler.add_ext("a".to_string());
    sheduler.add_ext(1);
    sheduler.add_ext(Database {});

    sheduler = sheduler
        .add(
            SyncJob::new()
                .every(2.hour())
                .repeat(3)
                .run(|a: Data<i32>, b: Data<i32>| {
                    // println!("{}", a.into_inner());
                    // println!("{}", b.into_inner());
                    // println!("结束了？");
                })
                .build(),
        )
        .add(
            SyncJob::new()
                .every(1.hour())
                .repeat(3)
                .run(|config: Data<Arc<Mutex<Config>>>| {
                    // println!("P");
                    let config = config.lock();
                    println!("{}", config.id);
                })
                .build(),
        );
    for _ in 0..100000 {
        sheduler = sheduler.add(
            SyncJob::new()
                .every(1.hour())
                .repeat(3)
                .run(|config: Data<Arc<Mutex<Config>>>| {
                    let mut config = config.lock();
                    config.id += 1;
                })
                .build(),
        );
    }
    sheduler.start().await;

    // sheduler
    //     .every(1.hours())
    // .run(|config: Data<Arc<Mutex<Config>>>| {
    //     let mut config = config.lock();
    //     config.id += 1;
    // });

    // sheduler
    //     .every(1.hours())
    //     .run(|config: Data<Arc<Mutex<Config>>>| {
    //         println!("{}", config.lock().id);
    //     });

    // sheduler.every(1.hours()).run(|| {});
    // sheduler.every(1.hours()).run(|a: Data<i32>| {
    //     println!("{}", a.into_inner());
    // });
    // sheduler.every(1.hours()).run(|a: Data<i32>, b: Data<i32>| {
    //     println!("{}", a.into_inner());
    //     println!("{}", b.into_inner());
    // });

    // sheduler
    //     .every(1.hours())
    //     .run(|config: Data<Arc<Mutex<Config>>>| {
    //         let mut config = config.lock();
    //         config.id += 1;
    //         println!("1");
    //     });

    // sheduler.every(1.hours()).run_async(|| async move {
    //     tokio::time::sleep(Duration::from_secs(5)).await;
    // });

    // sheduler.start();
}
