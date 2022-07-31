use crate::extensions::{Data, DebugAny, Extensions};

pub trait SyncHandler<Args> {
    fn call(&self, e: &Extensions);
}

impl<F> SyncHandler<()> for F
where
    F: Fn(),
{
    fn call(&self, _e: &Extensions) {
        self()
    }
}

impl<F, P1> SyncHandler<Data<P1>> for F
where
    P1: Clone + 'static + DebugAny + Send + Sync,
    F: Fn(Data<P1>),
{
    fn call(&self, e: &Extensions) {
        let p1 = e.get_data::<P1>();
        self(p1)
    }
}

impl<F, P1, P2> SyncHandler<(Data<P1>, Data<P2>)> for F
where
    P1: Clone + 'static + DebugAny + Send + Sync,
    P2: Clone + 'static + DebugAny + Send + Sync,
    F: Fn(Data<P1>, Data<P2>),
{
    fn call(&self, e: &Extensions) {
        let p1 = e.get_data::<P1>();
        let p2 = e.get_data::<P2>();
        self(p1, p2)
    }
}
