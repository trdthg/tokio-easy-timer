#![feature(async_closure)]

mod ext_map;
mod job;
use std::{fmt::Debug, sync::Arc, time::Duration};

use ext_map::{Data, DebugAny, Extensions};
use interval::{Interval, TimeUnits};
use job::Job;
use parking_lot::Mutex;
pub mod interval;

struct Sheduler {
    // jobs: Vec<Box<dyn Handler>>,
    extensions: Extensions,
}
impl Sheduler {
    pub fn new() -> Self {
        Self {
            extensions: Extensions::default(),
            // jobs: vec![],
        }
    }

    pub fn add_ext<T>(&self, ext: T)
    where
        T: DebugAny + 'static + Send + Sync,
    {
        self.extensions.insert(ext);
    }

    pub fn add(&mut self, job: Job) -> &mut Self {
        self
    }

    pub fn every(&self, interval: Interval) -> Job {
        Job::new(interval, self.extensions.clone())
    }

    pub fn start(&mut self) {}
}

#[derive(Default, Debug)]
struct Config {
    id: i32,
}

#[derive(Default, Debug)]
struct Database;

#[tokio::main]
async fn main() {
    let sheduler = Sheduler::new();

    let config = Arc::new(Mutex::new(Config { id: 1 }));
    sheduler.add_ext(config);
    sheduler.add_ext("a".to_string());
    sheduler.add_ext(1);
    sheduler.add_ext(Database {});

    sheduler
        .every(1.hours())
        .run(|config: Data<Arc<Mutex<Config>>>| {
            let mut config = config.lock();
            config.id += 1;
        });

    sheduler
        .every(1.hours())
        .run(|config: Data<Arc<Mutex<Config>>>| {
            println!("{}", config.lock().id);
        });

    sheduler.every(1.hours()).run(|| {});
    sheduler.every(1.hours()).run(|a: Data<i32>| {
        println!("{}", a.into_inner());
    });
    sheduler.every(1.hours()).run(|a: Data<i32>, b: Data<i32>| {
        println!("{}", a.into_inner());
        println!("{}", b.into_inner());
    });

    sheduler
        .every(1.hours())
        .run(|config: Data<Arc<Mutex<Config>>>| {
            let mut config = config.lock();
            config.id += 1;
            println!("1");
        });

    sheduler.every(1.hours()).run_async(|| async move {
        tokio::time::sleep(Duration::from_secs(5)).await;
    });

    // sheduler.start();
}
