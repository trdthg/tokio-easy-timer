# tokio-easy-timer

tokio-easy-timer is a tokio-based task scheduler, with a user-friendly API.

Inspired by [clokwerk](https://github.com/mdsherry/clokwerk)

## Features

- Easy: use job builder to build corn expression and task func
- Clean: use extension map to manage data
- Async: support both async and sync job
- Cron Expressions: support for using standard corn expressions

## Examples

```rs
use std::sync::Arc;
use parking_lot::Mutex;
use tokio_easy_timer::prelude::*;

struct Config {
    id: i32,
}

#[tokio::main]
async fn main() {
    let mut cheduler = Scheduler::new();

    // register some data that will be used later in task
    let config = Arc::new(Mutex::new(Config { id: 1 }));
    cheduler.add_ext(config);
    cheduler.add_ext("a".to_string());
    cheduler.add_ext(1);

    // add 10000 write tasks
    for _ in 0..10000 {
        cheduler.add(AsyncJob::new().every(1.minutes()).run(
            |config: Data<Arc<Mutex<Config>>>| async move {
                let mut config = config.lock();
                config.id += 1;
            },
        ));
    }

    // add one read tasks
    cheduler.add(AsyncJob::new().every(1.minutes()).run(
        |config: Data<Arc<Mutex<Config>>>| async move {
            let mut config = config.lock();
            config.id += 1;
            println!("{}", config.id);
        },
    ));

    // start
    cheduler.start().await;
}
```

## Tips

Sync job will be spawn by `tokio::task::spawn_blocking`

## Contributing
