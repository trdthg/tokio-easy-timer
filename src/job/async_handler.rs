use crate::extensions::{Data, Extensions};
use async_trait::async_trait;
use std::{fmt::Debug, future::Future};
#[async_trait]
pub trait AsyncHandler<Args> {
    async fn call(&self, e: &Extensions);
}

#[async_trait]
impl<F, Fut> AsyncHandler<()> for F
where
    Fut: Future<Output = ()>,
    F: Fn() -> Fut + Send + Sync + 'static,
{
    async fn call(&self, _e: &Extensions) {
        self();
    }
}

// #[async_trait]
// impl<F, P1> AsyncHandler<Data<P1>> for F
// where
//     P1: Clone + 'static + Debug,
//     F: Fn(Data<P1>) + Send + Sync,
// {
//     async fn call(&self, e: &Extensions) {
//         let p1 = e.get_data::<P1>();
//         self(p1)
//     }
// }

// #[async_trait]
// impl<F, P1, P2> AsyncHandler<(Data<P1>, Data<P2>)> for F
// where
//     P1: Clone + 'static + Debug + Send + Sync,
//     P2: Clone + 'static + Debug + Send + Sync,
//     F: Fn(Data<P1>, Data<P2>),
// {
//     async fn call(&self, e: &Extensions) {
//         let p1 = e.get_data::<P1>();
//         let p2 = e.get_data::<P2>();
//         self(p1, p2)
//     }
// }
