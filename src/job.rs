use std::fmt::Debug;

use crate::{
    ext_map::{Data, DebugAny, Extensions},
    interval::Interval,
};

pub trait Handler<Args> {
    fn call(&self, e: &Extensions);
}

impl<F> Handler<()> for F
where
    F: Fn(),
{
    fn call(&self, _e: &Extensions) {
        self()
    }
}

impl<F, P1> Handler<Data<P1>> for F
where
    P1: Clone + 'static + Debug,
    F: Fn(Data<P1>),
{
    fn call(&self, e: &Extensions) {
        let p1 = e.get_data::<P1>();
        self(p1)
    }
}

impl<F, P1, P2> Handler<(Data<P1>, Data<P2>)> for F
where
    P1: Clone + 'static + Debug + Send + Sync,
    P2: Clone + 'static + Debug + Send + Sync,
    F: Fn(Data<P1>, Data<P2>),
{
    fn call(&self, e: &Extensions) {
        let p1 = e.get_data::<P1>();
        let p2 = e.get_data::<P2>();
        self(p1, p2)
    }
}

pub struct Job {
    pub extensions: Extensions,
    pub interval: Interval,
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
}
