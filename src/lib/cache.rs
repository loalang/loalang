use crate::*;
use std::collections::hash_map::RandomState;
use std::hash::Hash;

pub struct Cache<K, V> {
    entries: HashMap<K, V>,
}

impl<K: Hash + Eq + Clone, V> Cache<K, V> {
    pub fn new() -> Cache<K, V> {
        Cache {
            entries: HashMap::new(),
        }
    }

    pub fn set(&mut self, k: K, v: V) {
        self.entries.insert(k, v);
    }

    pub fn get(&self, k: &K) -> Option<&V> {
        self.entries.get(k)
    }

    pub fn get_mut(&mut self, k: &K) -> Option<&mut V> {
        self.entries.get_mut(k)
    }

    pub fn cache<F: FnOnce(&mut Self) -> V>(&mut self, k: K, f: F) -> &V {
        if !self.entries.contains_key(&k) {
            let v = f(self);

            self.entries.insert(k.clone(), v);
        }
        self.entries.get(&k).unwrap()
    }
}

impl<K: Hash + Eq, V> From<HashMap<K, V>> for Cache<K, V> {
    fn from(entries: HashMap<K, V, RandomState>) -> Self {
        Cache { entries }
    }
}

#[test]
fn cache() {
    let mut cache = Cache::new();
    let mut called_times = 0;

    for _ in 0..3 {
        cache.cache(12, || {
            called_times += 1;
            String::from("twelve")
        });
    }

    assert_eq!(called_times, 1);
    assert_eq!(cache.get(&12).unwrap(), "twelve");
}
