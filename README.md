# tokio-easy-timer

tokio-easy-timer is a tokio-based task scheduler, with a user-friendly API.

Inspired by [clokwerk](https://github.com/mdsherry/clokwerk)

## Features

- Easy: use job builder to build corn expression and task func
- Clean: use extension map to manage data
- Async: support both async and sync job
- Cron Expressions: support for using standard corn expressions

## Examples

### Basic Usage

```rs
use std::sync::{Arc, Mutex};

use tokio_easy_timer::prelude::*;

struct Config {
    id: i32,
}

#[tokio::main]
async fn main() {
    let mut cheduler = Scheduler::new();
    let config = Arc::new(Mutex::new(Config { id: 0 }));
    cheduler.add_ext(config);

    for _ in 0..10000 {
        cheduler.add(
            SyncJob::new()
                .every(20.seconds())
                .run(|config: Data<Arc<Mutex<Config>>>| {
                    if let Ok(mut config) = config.lock() {
                        config.id += 1;
                    }
                }),
        );
    }

    cheduler.add(SyncJob::new().since_every(10.seconds(), 20.seconds()).run(
        |config: Data<Arc<Mutex<Config>>>| {
            if let Ok(config) = config.lock() {
                println!("{}", config.id);
            }
        },
    ));

    cheduler.run_pending().await;
}
```

### More Example

```rs
use std::sync::Arc;

use parking_lot::Mutex;
use tokio_easy_timer::prelude::*;

struct Config {
    id: i32,
}

#[tokio::main]
async fn main() {
    let mut cheduler = Scheduler::with_tz(chrono::FixedOffset::east(8 * 3600));

    // add whatever you want to the map
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
                .and() // use and to start build another corn expression
                // Runs every two hours on Saturdays
                .every(Interval::Sunday)
                .every(2.hour())
                .and()
                // Runs every 90 minutes after 14:13:00 every Saturday
                .every(Interval::Sunday)
                .since_time(14, 13, 0)
                .every(10.minutes())
                .repeat_async(2, 3.seconds()) // repeat
                .and()
                // Runs every 90 seconds, this needs two corn expression
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
```

## Contributing
