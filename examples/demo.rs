use std::sync::Arc;

use parking_lot::Mutex;
use tokio_easy_timer::prelude::*;

struct Config {
    id: i32,
}

#[tokio::main]
async fn main() {
    let mut cheduler = Scheduler::with_tz(chrono::FixedOffset::east(8 * 3600));

    let config = Arc::new(Mutex::new(Config { id: 1 }));
    cheduler.add_ext(config);
    cheduler.add_ext("a".to_string());
    cheduler.add_ext(1);

    cheduler
        .add(
            SyncJob::new()
                .every(Interval::Monday)
                .repeat_seq(3, 1.seconds())
                .run(|| {
                    // Some havey job, like read/write from DB
                }),
        )
        .add(
            AsyncJob::new()
                .every(Interval::Thursday)
                .at_time(8, 0, 0)
                .repeat_async(3, 30.seconds())
                .and()
                .every(Interval::Wednesday)
                .at_time(20, 45, 30)
                .run(|config: Data<Arc<Mutex<Config>>>| async move {
                    let mut config = config.lock();
                    config.id += 1;
                    println!("{}", config.id);
                }),
        )
        .add(
            AsyncJob::new()
                .every(Interval::Saturday)
                .at_time(14, 13, 0)
                .repeat_async(3, 5.seconds())
                .and()
                .every(Interval::Wednesday)
                .at_time(20, 45, 30)
                .run(|config: Data<Arc<Mutex<Config>>>| async move {
                    let mut config = config.lock();
                    config.id += 1;
                    println!("{}", config.id);
                }),
        )
        .add(
            AsyncJob::new()
                .every(Interval::Saturday) // 每周六，0:0:0 执行一次
                .every(Interval::Sunday) // 每周六，0:0:0 执行一次
                .and()
                .every(Interval::Sunday) // 每周六，每隔 2 小时，运行一次 0:0:0 0:02:0 0:04:00
                .every(2.hour())
                .and()
                .every(Interval::Sunday) // 每周六，每隔 2 小时，每分钟运行一次，持续 1 小时 0:0:0 0:0:0 0:1:0 0:2:0 2:0:0 2:1:0 2:2:0
                .every(2.hour())
                .every(1.minutes())
                .and()
                .every(Interval::Sunday) // 每周六，从 14 点 13 分 后，每隔 10 分钟运行一次
                .since_time(14, 13, 0)
                .every(1.hour())
                .every(10.minutes())
                .repeat_async(2, 3.seconds())
                .and()
                .every(Interval::Sunday) // 每周六，从 14 点 13 分 后，每隔 90 秒，运行一次
                .since_time(14, 13, 0) // 需要两个表达式
                .every(1.hour())
                .since_every(0.minutes(), 3.minutes()) // 从 0 开始，每隔 3 分钟，运行一次 0:00 3:00 6:00
                .and()
                .every(1.hour())
                .since_every(1.minutes(), 3.minutes()) // 从 1 开始，每隔 3 分种，运行一次 1:30 4:30 7:30
                .at(30.seconds())
                .run(|config: Data<Arc<Mutex<Config>>>| async move {
                    let mut config = config.lock();
                    config.id += 1;
                    println!("{}", config.id);
                }),
        );
    cheduler.start().await.pending().await;
    println!("a");
}

mod test {
    use std::str::FromStr;

    #[test]
    fn a() {
        let s = "0 0 2 31 12 7 2100";
        let s = cron::Schedule::from_str(s);
        println!("{:?}", s.unwrap().upcoming(chrono::Local).next());
    }
}
