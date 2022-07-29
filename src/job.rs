use std::{fmt::Debug, time::Duration};
mod async_handler;
mod handler;

use crate::{
    ext_map::{Data, DebugAny, Extensions},
    interval::Interval,
};

use self::async_handler::AsyncHandler;
use self::handler::Handler;

pub struct Job {
    extensions: Extensions,
    interval: Interval,
}

impl Job {
    pub fn new(interval: Interval, extensions: Extensions) -> Self {
        Self {
            interval,
            extensions,
        }
    }

    pub fn run<Params, F>(&self, f: F)
    where
        Params: Clone + 'static + Debug,
        F: Handler<Params>,
    {
        f.call(&self.extensions);
    }

    pub fn run_async<Params, F>(&self, f: F)
    where
        Params: Clone + 'static + Debug,
        F: Clone + Copy + Send + Sync + 'static + AsyncHandler<Params>,
    {
        let f = f.clone();
        let ext = self.extensions.clone();
        tokio::spawn(async move {
            loop {
                let f = f.clone();
                f.call(&ext).await;
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
        });
    }
}
