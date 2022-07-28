mod data;
mod type_key;
pub use data::Data;
use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};
use std::{any::Any, collections::HashMap, fmt::Debug};
use type_key::TypeKey;

pub trait DebugAny: Any + Debug {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Any + Debug + 'static> DebugAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Default)]
pub struct Extensions {
    map: RwLock<HashMap<TypeKey, Box<dyn DebugAny>>>,
}
impl Extensions {
    pub fn insert<T>(&self, data: T)
    where
        T: DebugAny + 'static,
    {
        self.map
            .write()
            .insert(TypeKey::of::<T>(), Box::new(Data::new(data)));
    }

    pub fn get<T: Send + Sync + Debug + 'static>(&self) -> MappedRwLockReadGuard<'_, T>
    where
        T: Default,
    {
        self.ensure::<T>();
        RwLockReadGuard::map(self.map.read(), |m| {
            m.get(&TypeKey::of::<T>())
                .and_then(|x| (*x).as_any().downcast_ref())
                .unwrap()
        })
    }

    pub fn get_mut<T>(&self) -> MappedRwLockWriteGuard<'_, T>
    where
        T: Send + Sync + Debug + 'static + Default,
    {
        self.ensure::<T>();
        RwLockWriteGuard::map(self.map.write(), |m| {
            m.get_mut(&TypeKey::of::<T>())
                .and_then(|x| (**x).as_any_mut().downcast_mut())
                .unwrap()
        })
    }

    fn ensure<T: Send + Sync + Default + Debug + 'static>(&self) {
        if self.map.read().get(&TypeKey::of::<T>()).is_none() {
            self.insert(T::default());
        }
    }

    pub fn get_data<T>(&self) -> Data<T>
    where
        T: Send + Sync + Debug + 'static + Default,
    {
        let key = TypeKey::of::<T>();
        self.ensure::<T>();
        let data = self.map.read();
        let res = data.get(&key).unwrap();
        let res = (**res).as_any().downcast_ref::<Data<T>>().unwrap();
        let res = res.clone();
        res
    }
}
