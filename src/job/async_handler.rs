use crate::extensions::{Data, DebugAny, Extensions};
use async_trait::async_trait;
use std::future::Future;
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

#[async_trait]
impl<F, Fut, P1> AsyncHandler<Data<P1>> for F
where
    P1: Clone + 'static + DebugAny + Send + Sync,
    Fut: Future<Output = ()> + Send + Sync,
    F: Fn(Data<P1>) -> Fut + Send + Sync,
{
    async fn call(&self, e: &Extensions) {
        let p1 = e.get_data::<P1>();
        self(p1).await
    }
}

#[async_trait]
impl<F, Fut, P1, P2> AsyncHandler<(Data<P1>, Data<P2>)> for F
where
    P1: Clone + 'static + DebugAny + Send + Sync,
    P2: Clone + 'static + DebugAny + Send + Sync,
    Fut: Future<Output = ()> + Send + Sync,
    F: Fn(Data<P1>, Data<P2>) -> Fut + Send + Sync,
{
    async fn call(&self, e: &Extensions) {
        let p1 = e.get_data::<P1>();
        let p2 = e.get_data::<P2>();
        self(p1, p2).await
    }
}
