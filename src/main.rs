mod ext_map;
mod job;
use std::fmt::Debug;

use ext_map::{Data, DebugAny, Extensions};
use interval::{Interval, TimeUnits};
use job::Job;
use parking_lot::Mutex;
pub mod interval;

struct Sheduler {
    jobs: Vec<Box<Job<Box<dyn DebugAny>>>>,
    extensions: Extensions,
}
impl Sheduler {
    pub fn new() -> Self {
        Self {
            extensions: Extensions::default(),
            jobs: vec![],
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

fn main() {
    let mut sheduler = Sheduler::new();
    let config = Mutex::new(Config { id: 1 });
    sheduler.add_ext(config);
    sheduler.add_ext(Database {});

    Job::new(1.hours(), sheduler.extensions.clone()).run(
        |config: Data<(Mutex<Config>, String)>| {
            let mut config = config.0.lock();
            config.id += 1;
        },
    );

    Job::new(1.hours(), sheduler.extensions.clone()).run(|config: Data<Mutex<Config>>| {
        let config = config.lock();
        println!("{}", config.id)
    });

    // sheduler
    //     .every(1.hours())
    //     .run(|config: Data<Mutex<Config>>| {
    //         let mut config = config.lock();
    //         config.id += 1;
    //         println!("1");
    //     });

    sheduler.start();

    // sheduler.run(|config: Data<Mutex<Config>>| {
    //     let config = config.lock();
    //     println!("{}", config.id)
    // });
}
