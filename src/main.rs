mod extensions;
pub mod interval;
mod job;
mod scheduler;
use extensions::Data;
use interval::{Interval, TimeUnits};
use job::SyncJob;
use parking_lot::Mutex;
use scheduler::Scheduler;
use std::{fmt::Debug, str::FromStr, sync::Arc};

#[derive(Default, Debug)]
struct Config {
    id: i32,
}

#[derive(Default, Debug)]
struct Database;

#[tokio::main]
async fn main() {
    let cheduler = Scheduler::new();

    let config = Arc::new(Mutex::new(Config { id: 1 }));
    cheduler.add_ext(config);
    cheduler.add_ext("a".to_string());
    cheduler.add_ext(1);
    cheduler.add_ext(Database {});

    cheduler
        // .add(
        //     SyncJob::new()
        //         .every(Interval::Thursday)
        //         .at_time(8, 0, 0)
        //         .repeat(3, 30.seconds())
        //         .next()
        //         .every(Interval::Wednesday)
        //         .at_time(20, 45, 30)
        //         .run(|config: Data<Arc<Mutex<Config>>>| {
        //             let mut config = config.lock();
        //             config.id += 1;
        //             println!("{}", config.id);
        //         }),
        // )
        // .add(
        //     SyncJob::new()
        //         .every(Interval::Saturday)
        //         .at_time(14, 13, 0)
        //         .repeat(3, 5.seconds())
        //         .next()
        //         .every(Interval::Wednesday)
        //         .at_time(20, 45, 30)
        //         .run(|config: Data<Arc<Mutex<Config>>>| {
        //             let mut config = config.lock();
        //             config.id += 1;
        //             println!("{}", config.id);
        //         }),
        // )
        .add(
            SyncJob::new()
                .every(Interval::Saturday) // 每周六，0:0:0 执行一次
                // .every(Interval::Sunday) // 每周六，0:0:0 执行一次
                .next()
                // .every(Interval::Saturday) // 每周六，每隔 2 小时，运行一次 0:0:0 0:02:0 0:04:00
                // .every(2.hour())
                // .next()
                // .every(Interval::Saturday) // 每周六，每隔 2 小时，每分钟运行一次，持续 1 小时 0:0:0 0:0:0 0:1:0 0:2:0 2:0:0 2:1:0 2:2:0
                // .every(2.hour())
                // .every(1.minutes())
                // .next()
                // .every(Interval::Saturday) // 每周六，从 14 点 13 分 后，每隔 10 秒运行一次
                // .since_time(14, 13, 0)
                // .every(1.hour())
                // .every(10.minutes())
                // .repeat(2, 3.seconds())
                // .next()
                // .every(Interval::Saturday) // 每周六，从 14 点 13 分 后，每隔 90 秒，运行一次
                // .since_time(14, 13, 0) // 需要两个表达式
                // .every(1.hour())
                // .since_every(0.minutes(), 3.minutes()) // 从 0 开始，每隔 3 分钟，运行一次 0:00 3:00 6:00
                // .next()
                // .every(1.hour())
                // .since_every(1.minutes(), 3.minutes()) // 从 1 开始，每隔 3 分种，运行一次 1:30 4:30 7:30
                // .at(30.seconds())
                .run(|config: Data<Arc<Mutex<Config>>>| {
                    let mut config = config.lock();
                    config.id += 1;
                    println!("{}", config.id);
                }),
        )
        .start()
        .await;
    // for _ in 0..100000 {
    //     cheduler = cheduler.add(
    //         SyncJob::new()
    //             .every(1.minutes())
    //             .repeat(3)
    //             .run(|config: Data<Arc<Mutex<Config>>>| {
    //                 let mut config = config.lock();
    //                 config.id += 1;
    //             })
    //             .build(),
    //     );
    // }
    // cheduler.start().await;

    // .every(Interval::Saturday) // 每周六，每隔 2 小时，下一个小时内 每 10 分钟运行一轮，每轮每 10 秒运行一次，持续 1 分钟 0:0:0 0:0:1 0:1:0 0:2:0 2:0:0 2:1:0 2:2:0
    // .every(2.hour())
    // .every(10.minutes())
    // .every(10.seconds())
}

#[test]
fn test() {
    let s = "0 0 0 * * 7,1 *";
    let s = cron::Schedule::from_str(s);
    for i in s.unwrap().upcoming(chrono::Local).take(100) {
        println!("{}", i);
    }
}
