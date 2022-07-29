use std::{fmt::Debug, future::Future, marker::PhantomData};

use crate::{
    ext_map::{Data, DebugAny, Extensions},
    interval::Interval,
};

// pub trait Handler<Args>: 'static {
//     fn call(&self, args: Args);
// }
pub struct Job<Args> {
    pub extensions: Extensions,
    pub interval: Interval,
    pub handler: Option<Box<dyn Fn(Data<Args>)>>,
    _phantom: PhantomData<Args>,
}

// type BoxedFnOnce = Box<dyn FnOnce(Box<&dyn DebugAny>)>;

// impl<Args: 'static + DebugAny> Handler<Args> for Box<dyn FnOnce(Args)> {
//     fn call(&self, args: Args) {
//         self.as_mut()(args)
//     }
// }

impl<Args> Job<Args> {
    pub fn new(interval: Interval, extensions: Extensions) -> Self {
        Self {
            interval,
            extensions,
            handler: None,
            _phantom: PhantomData,
        }
    }

    pub fn run<F>(&mut self, handler: F)
    where
        F: Fn(Data<Args>) + 'static + Clone,
        Args: Send + Sync + 'static + DebugAny + Default,
    {
        self.handler = Some(Box::new(handler));
        let args = self.extensions.get_data::<Args>();
        let f = self.handler.as_deref().unwrap();
        f(args)

        // .map(|f| f.as_ref()(args));
        // self.handler.map(|handler| {
        //     //
        //     let handler = handler.as_ref().clone();
        //     handler(args)
        // });
    }
}
