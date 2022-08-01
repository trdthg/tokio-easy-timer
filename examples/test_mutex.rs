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

    for _ in 0..1000000 {
        cheduler.add(AsyncJob::new().every(1.minutes()).run(
            |config: Data<Arc<Mutex<Config>>>| async move {
                let mut config = config.lock();
                config.id += 1;
            },
        ));
    }

    cheduler.add(AsyncJob::new().every(1.minutes()).run(
        |config: Data<Arc<Mutex<Config>>>| async move {
            let mut config = config.lock();
            config.id += 1;
            println!("{}", config.id);
        },
    ));

    cheduler.start().await;
}
