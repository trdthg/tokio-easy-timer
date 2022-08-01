use crate::extensions::{Data, Extensions};
use async_trait::async_trait;
use std::future::Future;
#[async_trait]
pub trait AsyncHandler<Args> {
    async fn call(&self, e: &Extensions);
}
macro_rules! impl_handler {
    ($( $P:ident ),*) => {
        #[async_trait]
        impl<F, Fut, $($P,)*> AsyncHandler<($(Data<$P>,)*)> for F
        where
            $( $P: Clone + 'static + Send + Sync, )*
            Fut: Future<Output = ()> + Send + Sync,
            F: Fn($(Data<$P>,)*) -> Fut + Send + Sync,
        {
            async fn call(&self, _e: &Extensions) {
                self($(_e.get_data::<$P>(),)*).await
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
