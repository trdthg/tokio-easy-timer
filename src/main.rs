mod ext_map;
use std::{fmt::Debug, sync::Arc};

use ext_map::{Data, DebugAny, Extensions};
use parking_lot::Mutex;

struct Sheduler {
    extensions: Extensions,
}
impl Sheduler {
    pub fn new() -> Self {
        Self {
            extensions: Extensions::default(),
        }
    }

    pub fn add_ext<T>(&self, ext: T)
    where
        T: DebugAny + 'static,
    {
        self.extensions.insert(ext);
    }

    pub fn run<T, F>(&mut self, f: F)
    where
        T: Send + Sync + Debug + 'static + Default,
        F: FnOnce(Data<T>),
    {
        let args = self.extensions.get_data::<T>();
        f(args);
    }
}

#[derive(Default, Debug)]
struct Config {
    id: i32,
}

#[derive(Default, Debug)]
struct Database;

fn main() {
    let mut app = Sheduler::new();
    let config = Mutex::new(Config { id: 1 });
    app.add_ext(config);
    app.add_ext(Database {});

    app.run(|config: Data<Mutex<Config>>| {
        let mut config = config.lock();
        config.id += 1;
    });

    app.run(|config: Data<Mutex<Config>>| {
        let config = config.lock();
        println!("{}", config.id)
    });
}
