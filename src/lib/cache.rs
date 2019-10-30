use crate::*;
use std::hash::Hash;

#[derive(Clone)]
pub struct Cache<K, T> {
    mutex: Arc<Mutex<HashMap<K, T>>>,
}

impl<K, T> Cache<K, T>
where
    K: Hash + Eq,
{
    pub fn new() -> Cache<K, T> {
        Cache {
            mutex: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl<K, T> Cache<K, T>
where
    T: Default + Clone,
    K: Hash + Eq + Clone,
{
    pub fn gate<F: FnOnce() -> T>(&self, key: &K, f: F) -> T {
        {
            if let Ok(mut cache) = self.mutex.lock() {
                if let Some(type_) = cache.get(key) {
                    return type_.clone();
                }

                cache.insert(key.clone(), Default::default());
            }
        }

        let result = f();

        {
            if let Ok(mut cache) = self.mutex.lock() {
                cache.insert(key.clone(), result.clone());
            }
        }

        result
    }
}

impl<K, T> Cache<K, T>
where
    T: Clone,
    K: Hash + Eq + Clone,
{
    pub fn gate_not_loop_safe<F: FnOnce() -> T>(&self, key: &K, f: F) -> T {
        {
            if let Ok(cache) = self.mutex.lock() {
                if let Some(type_) = cache.get(key) {
                    return type_.clone();
                }
            }
        }

        let result = f();

        {
            if let Ok(mut cache) = self.mutex.lock() {
                cache.insert(key.clone(), result.clone());
            }
        }

        result
    }
}
