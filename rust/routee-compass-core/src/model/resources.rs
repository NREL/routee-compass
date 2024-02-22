use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

pub trait Resource: Any + Send + Sync {}

#[derive(Default)]
pub struct Resources {
    resources: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl Resources {
    pub fn add_resource<T: Resource + 'static>(&mut self, resource: T) {
        self.resources.insert(
            TypeId::of::<T>(),
            Arc::new(resource) as Arc<dyn Any + Send + Sync>,
        );
    }

    pub fn get_resource<T: Resource + 'static>(&self) -> Option<Arc<T>> {
        self.resources
            .get(&TypeId::of::<T>())
            .and_then(|arc_any| arc_any.downcast_ref::<Arc<T>>().cloned())
    }
}
