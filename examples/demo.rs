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
                    // Some havey job, like complex calculate or read/write from DB.
                }),
        )
        .add(
            AsyncJob::new()
                // Runs every 90 minutes after 14:13:00 every Saturday
                .every(Interval::Saturday)
                .every(Interval::Sunday)
                .and()
                // Runs every two hours on Saturdays
                .every(Interval::Sunday)
                .every(2.hour())
                .and()
                // Runs every 90 minutes after 14:13:00 every Saturday
                .every(Interval::Sunday)
                .since_time(14, 13, 0)
                .every(1.hour())
                .every(10.minutes())
                .repeat_async(2, 3.seconds())
                .and()
                // Runs every 90 seconds
                .since_every(0.minutes(), 3.minutes()) // Runs every 3 minutes starting at 0:00
                .and()
                .since_every(1.minutes(), 3.minutes()) // Runs every 3 minutes starting at 1:30
                .at(30.seconds())
                .run(|config: Data<Arc<Mutex<Config>>>| async move {
                    let mut config = config.lock();
                    config.id += 1;
                    println!("{}", config.id);
                }),
        )
        .run_pending()
        .await;
}
