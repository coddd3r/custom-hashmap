use std::{
    hash::{DefaultHasher, Hash, Hasher},
    mem,
};

const INITIAL_NBUCKETS: usize = 1;
pub struct HashMap<K, V> {
    buckets: Vec<Vec<(K, V)>>,
    //buckets: Vec<Bucket<K, V>>,
    //build_hasher: RandomState,
    items: usize,
}

impl<K, V> HashMap<K, V> {
    pub fn new() -> Self {
        Self {
            buckets: Vec::new(),
            items: 0,
        }
    }
}

impl<K, V> HashMap<K, V>
where
    K: Hash + Eq,
{
    fn get_bucket(&self, key: &K) -> usize {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() % self.buckets.len() as u64) as usize
    }

    fn resize(&mut self) {
        let target_size = match self.buckets.len() {
            0 => INITIAL_NBUCKETS,
            n => n * 2,
        };
        let mut new_buckets = Vec::with_capacity(target_size);
        new_buckets.extend((0..target_size).map(|_| Vec::new()));

        for (key, value) in self.buckets.iter_mut().flat_map(|bucket| bucket.drain(..)) {
            let mut hasher = DefaultHasher::new();
            key.hash(&mut hasher);
            let bucket: usize = (hasher.finish() % new_buckets.len() as u64) as usize;
            new_buckets[bucket].push((key, value));
        }
        let _ = mem::replace(&mut self.buckets, new_buckets);
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if self.buckets.is_empty() || self.items > 3 * self.buckets.len() / 4 {
            self.resize();
        }

        let bucket: usize = self.get_bucket(&key);
        let bucket = &mut self.buckets[bucket];

        self.items += 1;
        for &mut (ref ekey, ref mut evalue) in &mut *bucket {
            if ekey == &key {
                use std::mem;
                return Some(mem::replace(evalue, value));
            }
        }

        bucket.push((key, value));
        None
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let bucket = self.get_bucket(key);
        self.buckets[bucket]
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let bucket = self.get_bucket(key);
        let bucket = &mut self.buckets[bucket];
        //let removed = None;
        let index = bucket.iter().position(|(k, _)| k == key)?;
        Some(bucket.swap_remove(index).1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_test() {
        let mut map = HashMap::new();
        map.insert("a", 10);
        assert_eq!(map.get(&"a"), Some(&10));
        assert_eq!(map.remove(&"a"), Some(10));

        assert_eq!(map.get(&"a"), None);
    }
}
