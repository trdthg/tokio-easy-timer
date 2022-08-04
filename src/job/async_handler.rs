use crate::extensions::{Data, Extensions};
use std::{future::Future, pin::Pin};

pub trait AsyncHandler<Args> {
    fn call(self, e: &Extensions) -> Pin<Box<dyn Future<Output = ()> + Send>>;
}

macro_rules! impl_handler {
    ($( $P:ident ),*) => {
        impl<F, Fut, $($P,)*> AsyncHandler<($(Data<$P>,)*)> for F
        where
            $( $P: Clone + 'static + Send + Sync, )*
            Fut: Future<Output = ()> + Send + 'static,
            F: Fn($(Data<$P>,)*) -> Fut + Send + Copy,
        {
            fn call(self, _e: &Extensions) -> Pin<Box<dyn Future<Output = ()> + Send>> {
                let f = self.to_owned();
                let f = f($(_e.get_data::<$P>(),)*);
                Box::pin(
                    async {
                        f.await;
                    }
                )
        }
        }
    };
}

impl_handler!();
impl_handler!(P1);
impl_handler!(P1, P2);
impl_handler!(P1, P2, P3);
impl_handler!(P1, P2, P3, P4);
impl_handler!(P1, P2, P3, P4, P5);
impl_handler!(P1, P2, P3, P4, P5, P6);
impl_handler!(P1, P2, P3, P4, P5, P6, P7);
impl_handler!(P1, P2, P3, P4, P5, P6, P7, P8);
impl_handler!(P1, P2, P3, P4, P5, P6, P7, P8, P9);
