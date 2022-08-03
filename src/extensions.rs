mod data;
mod type_key;
pub use data::Data;
use parking_lot::RwLock;
use std::{any::Any, collections::HashMap, sync::Arc};
use type_key::TypeKey;

pub trait AsAny: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Any + 'static> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Default, Clone)]
pub struct Extensions {
    map: Arc<RwLock<HashMap<TypeKey, Box<dyn AsAny + Send + Sync>>>>,
}

impl Extensions {
    /// insert a type to the map, if already exists, then replace
    pub(crate) fn insert<T>(&self, data: T)
    where
        T: 'static + Send + Sync,
    {
        let key = TypeKey::of::<T>();
        self.map.write().insert(key, Box::new(Data::new(data)));
    }

    /// this will panic if the required type doesn't exist
    pub(crate) fn get_data<T>(&self) -> Data<T>
    where
        T: 'static + Send + Sync,
    {
        let key = TypeKey::of::<T>();
        let data = self.map.read();
        let res = data.get(&key).unwrap();
        let res = (**res).as_any().downcast_ref::<Data<T>>().unwrap();
        let res = res.clone();
        res
    }
}
