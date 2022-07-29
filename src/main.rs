mod ext_map;
mod job;
use std::{fmt::Debug, sync::Arc};

use ext_map::{Data, DebugAny, Extensions};
use interval::{Interval, TimeUnits};
use job::{Handler, Job};
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
        T: DebugAny + 'static,
    {
        self.extensions.insert(ext);
    }

    // pub fn every<Args>(&mut self, interval: Interval) -> &mut Job<Box<dyn DebugAny>>
    // where
    //     Args: DebugAny + 'static,
    // {
    //     let job: Job<Box<dyn DebugAny>> = Job::new(interval);
    //     self.jobs.push(Box::new(job));

    //     let last_index = self.jobs.len() - 1;
    //     self.jobs[last_index].as_mut()
    // }

    pub fn start(&mut self) {}
}

#[derive(Default, Debug)]
struct Config {
    id: i32,
}

#[derive(Default, Debug)]
struct Database;

fn a(config: Data<Mutex<Config>>) {
    let mut config = config.lock();
    config.id += 1;
}

fn main() {
    let sheduler = Sheduler::new();

    let config = Arc::new(Mutex::new(Config { id: 1 }));
    sheduler.add_ext(config);
    sheduler.add_ext("a".to_string());
    sheduler.add_ext(1);
    sheduler.add_ext(Database {});

    Job::new(1.hours(), sheduler.extensions.clone()).run(|config: Data<Arc<Mutex<Config>>>| {
        let mut config = config.lock();
        config.id += 1;
    });

    Job::new(1.hours(), sheduler.extensions.clone()).run(|config: Data<Arc<Mutex<Config>>>| {
        println!("{}", config.lock().id);
    });

    Job::new(1.hours(), sheduler.extensions.clone()).run(|| {});
    Job::new(1.hours(), sheduler.extensions.clone()).run(|a: Data<i32>| {
        println!("{}", a.into_inner());
    });
    Job::new(1.hours(), sheduler.extensions.clone()).run(|a: Data<i32>, b: Data<i32>| {
        println!("{}", a.into_inner());
        println!("{}", b.into_inner());
    });

    // sheduler
    //     .every(1.hours())
    //     .run(|config: Data<Mutex<Config>>| {
    //         let mut config = config.lock();
    //         config.id += 1;
    //         println!("1");
    //     });

    // sheduler.start();

    // sheduler.run(|config: Data<Mutex<Config>>| {
    //     let config = config.lock();
    //     println!("{}", config.id)
    // });
}
