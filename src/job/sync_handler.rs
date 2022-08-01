use crate::extensions::{Data, Extensions};

pub trait SyncHandler<Args> {
    fn call(&self, e: &Extensions);
}

macro_rules! impl_handler {
    ($( $P:ident ),*) => {
        impl<F, $($P,)*> SyncHandler<($(Data<$P>,)*)> for F
        where
            $( $P: Clone + 'static + Send + Sync, )*
            F: Fn($(Data<$P>,)*),
        {
            fn call(&self, _e: &Extensions) {
                self($(_e.get_data::<$P>(),)*);
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
