use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;

pub struct PluginContext {
    services: RefCell<HashMap<&'static str, Box<dyn Any + Send + Sync>>>,
}

impl Default for PluginContext {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginContext {
    pub fn new() -> Self {
        PluginContext {
            services: RefCell::new(HashMap::new()),
        }
    }

    pub fn register<T: 'static + Send + Sync>(&self, key: &'static str, val: T) {
        self.services.borrow_mut().insert(key, Box::new(val));
    }

    pub fn get<T: 'static + Clone>(&self, key: &str) -> Option<T> {
        self.services
            .borrow()
            .get(key)
            .and_then(|v| v.downcast_ref::<T>())
            .cloned()
    }

    pub fn unregister(&self, key: &'static str) {
        self.services.borrow_mut().remove(key);
    }
}