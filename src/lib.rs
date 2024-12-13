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

    pub fn len(&self) -> usize {
        self.items
    }

    pub fn is_empty(&self) -> bool {
        self.items == 0
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

    pub fn contains_key(&self, key: &K) -> bool {
        let bucket = self.get_bucket(key);
        self.buckets[bucket].iter().any(|(k, _)| k == key)
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let bucket = self.get_bucket(key);
        let bucket = &mut self.buckets[bucket];
        //let removed = None;
        let index = bucket.iter().position(|(k, _)| k == key)?;
        self.items -= 1;
        Some(bucket.swap_remove(index).1)
    }
}

pub struct Iter<'a, K, V> {
    map: &'a HashMap<K, V>,
    curr_bucket: usize,
    at: usize, // index inside a specific bucket
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.map.buckets.get(self.curr_bucket) {
                Some(bucket) => match bucket.get(self.at) {
                    Some((k, v)) => {
                        self.at += 1;
                        break Some((k, v));
                    }
                    None => {
                        self.curr_bucket += 1;
                        self.at = 0;
                        continue;
                    }
                },
                None => break None,
            }
        }
    }
}

impl<'a, K, V> IntoIterator for &'a HashMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;
    fn into_iter(self) -> Iter<'a, K, V> {
        Iter {
            curr_bucket: 0,
            at: 0,
            map: self,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_test() {
        let mut map = HashMap::new();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
        map.insert("a", 10);
        assert_eq!(map.get(&"a"), Some(&10));
        assert!(!map.is_empty());
        assert_eq!(map.len(), 1);
        assert!(map.contains_key(&"a"));
        assert_eq!(map.remove(&"a"), Some(10));
        assert_eq!(map.len(), 0);
        assert!(map.is_empty());

        assert_eq!(map.get(&"a"), None);
    }

    #[test]
    fn iter_test() {
        let mut map = HashMap::new();
        assert!(map.is_empty());
        map.insert("a", 10);
        map.insert("b", 20);
        map.insert("c", 30);
        map.insert("d", 40);
        assert!(!map.is_empty());
        assert_eq!(map.len(), 4);

        map.into_iter().for_each(|(k, v)| match *k {
            "a" => assert_eq!(*v, 10),
            "b" => assert_eq!(*v, 20),
            "c" => assert_eq!(*v, 30),
            "d" => assert_eq!(*v, 40),
            _ => unreachable!(),
        });
    }
}
