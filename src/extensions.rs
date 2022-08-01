mod data;
mod type_key;
pub use data::Data;
use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};
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
    pub fn insert<T>(&self, data: T)
    where
        T: 'static + Send + Sync,
    {
        let key = TypeKey::of::<T>();
        println!("key: {:?}", key);

        self.map.write().insert(key, Box::new(Data::new(data)));
    }

    pub fn get<T: Send + Sync + AsAny + 'static>(&self) -> MappedRwLockReadGuard<'_, T> {
        RwLockReadGuard::map(self.map.read(), |m| {
            m.get(&TypeKey::of::<T>())
                .and_then(|x| (*x).as_any().downcast_ref())
                .unwrap()
        })
    }

    pub fn get_mut<T>(&self) -> MappedRwLockWriteGuard<'_, T>
    where
        T: Send + Sync + AsAny + 'static,
    {
        RwLockWriteGuard::map(self.map.write(), |m| {
            m.get_mut(&TypeKey::of::<T>())
                .and_then(|x| (**x).as_any_mut().downcast_mut())
                .unwrap()
        })
    }

    /// check whether a type exists
    fn has_type<T: AsAny + 'static + Send + Sync>(&self) -> bool {
        match self.map.read().get(&TypeKey::of::<T>()) {
            Some(_) => true,
            None => false,
        }
    }

    /// this will panic if the required type doesn't exist
    pub fn get_data<T>(&self) -> Data<T>
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
