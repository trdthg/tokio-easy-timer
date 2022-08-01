use std::sync::Arc;

use parking_lot::Mutex;
use tokio_sheduler::prelude::*;

#[derive(Default, Debug)]
struct Config {
    id: i32,
}

struct Database;

#[tokio::main]
async fn main() {
    let mut cheduler = Scheduler::with_tz(chrono::FixedOffset::east(8 * 3600));

    let config = Arc::new(Mutex::new(Config { id: 1 }));
    cheduler.add_ext(config);
    cheduler.add_ext("a".to_string());
    cheduler.add_ext(1);
    cheduler.add_ext(Database {});

    cheduler
        .add(
            SyncJob::new()
                .every(Interval::Monday)
                .repeat_seq(3, 1.seconds())
                .run(|| {
                    // Some havey job, like read/write from DB
                }),
        )
        .add(AsyncJob::new().every(10.seconds()).run(
            |config: Data<Arc<Mutex<Config>>>| async move {
                let config = config.lock();
                println!("{}", config.id);
            },
        ));
    // for _ in 0..10000 {
    //     cheduler.add(
    //         AsyncJob::new()
    //             .every(1.minutes())
    //             .repeat_unblocking(3, 1.seconds())
    //             .run(|config: Data<Arc<Mutex<Config>>>| async move {
    //                 // let mut config = config.lock();
    //                 // config.id += 1;
    //             }),
    //     );
    // }
    cheduler.start().await;

    // .every(Interval::Saturday) // 每周六，每隔 2 小时，下一个小时内 每 10 分钟运行一轮，每轮每 10 秒运行一次，持续 1 分钟 0:0:0 0:0:1 0:1:0 0:2:0 2:0:0 2:1:0 2:2:0
    // .every(2.hour())
    // .every(10.minutes())
    // .every(10.seconds())
}
