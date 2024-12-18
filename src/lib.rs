use std::{
    borrow::Borrow,
    hash::{DefaultHasher, Hash, Hasher},
    mem,
    ops::Index,
};

const INITIAL_NBUCKETS: usize = 1;

#[derive(Debug)]
pub struct HashMap<K, V> {
    buckets: Vec<Vec<(K, V)>>,
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

impl<K, V> Default for HashMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> Clone for HashMap<K, V>
where
    K: Clone + Hash + Eq,
    V: Clone + Hash + Eq,
{
    fn clone(&self) -> HashMap<K, V> {
        let mut cloned_buckets = HashMap::new();
        for x in self.buckets.iter() {
            for (key, value) in x {
                cloned_buckets.insert(key.clone(), value.clone());
            }
        }
        cloned_buckets
    }
}

pub struct OccupiedEntry<'a, K, V> {
    element: &'a mut (K, V), //ref to the present key,val pair
}

pub struct VacantEntry<'a, K, V> {
    key: K,
    map: &'a mut HashMap<K, V>,
    bucket: usize,
}

impl<'a, K, V> VacantEntry<'a, K, V> {
    pub fn insert(self, value: V) -> &'a mut V
    where
        K: Hash + Eq,
        V: Hash + Eq,
    {
        self.map.buckets[self.bucket].push((self.key, value));
        self.map.items += 1;
        &mut self.map.buckets[self.bucket].last_mut().unwrap().1
    }
}

pub enum Entry<'a, K, V> {
    Occupied(OccupiedEntry<'a, K, V>),
    Vacant(VacantEntry<'a, K, V>),
}

impl<'a, K, V> Entry<'a, K, V>
where
    K: Hash + Eq,
    V: Hash + Eq,
{
    pub fn or_insert(self, val: V) -> &'a mut V {
        match self {
            Entry::Occupied(e) => &mut e.element.1, //ifoccupied give back value
            Entry::Vacant(e) => e.insert(val),
        }
    }

    pub fn or_insert_with<F>(self, maker: F) -> &'a mut V
    where
        F: FnOnce() -> V,
    {
        match self {
            Entry::Occupied(e) => &mut e.element.1, //ifoccupied give back value
            Entry::Vacant(e) => e.insert(maker()),
        }
    }

    pub fn or_default(self) -> &'a mut V
    where
        V: Default,
    {
        self.or_insert_with(Default::default)
    }

    pub fn and_modify<F>(self, modifier: F) -> Self
    where
        F: FnOnce(&mut V),
    {
        match self {
            Entry::Occupied(e) => {
                modifier(&mut e.element.1);
                Entry::Occupied(e)
            }
            Entry::Vacant(e) => Entry::Vacant(e),
        }
    }
}

impl<K, V> HashMap<K, V>
where
    K: Hash + Eq,
{
    fn get_bucket<Q>(&self, key: &Q) -> usize
    where
        K: Borrow<Q>,
        Q: Eq + Hash + ?Sized,
    {
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

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Eq + Hash + ?Sized,
    {
        let bucket = self.get_bucket(key);
        self.buckets[bucket]
            .iter()
            .find(|(k, _)| k.borrow() == key)
            .map(|(_, v)| v)
    }

    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Eq + Hash + ?Sized,
    {
        let bucket = self.get_bucket(key);
        self.buckets[bucket].iter().any(|(k, _)| k.borrow() == key)
    }

    pub fn entry(&mut self, key: K) -> Entry<K, V> {
        if self.buckets.is_empty() || self.items > 3 * self.buckets.len() / 4 {
            self.resize();
        }
        let bucket = self.get_bucket(&key);

        //get around double borrow
        if self.buckets[bucket].iter().any(|(k, _)| k == &key) {
            let e = self.buckets[bucket].iter_mut().find(|(k, _)| k == &key);
            return Entry::Occupied(OccupiedEntry {
                element: e.unwrap(),
            });
        } else {
            Entry::Vacant(VacantEntry {
                key,
                bucket,
                map: self,
            })
        }
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Eq + Hash + ?Sized,
    {
        let bucket = self.get_bucket(key);
        let bucket = &mut self.buckets[bucket];
        let index = bucket.iter().position(|(k, _)| k.borrow() == key)?;
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

pub struct IntoIter<K, V> {
    map: HashMap<K, V>,
    curr_bucket: usize,
}

impl<K, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.map.buckets.get_mut(self.curr_bucket) {
                Some(bucket) => match bucket.pop() {
                    Some(x) => break Some(x),
                    None => {
                        self.curr_bucket += 1;
                        continue;
                    }
                },
                None => break None,
            }
        }
    }
}

impl<K, V> IntoIterator for HashMap<K, V> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;
    fn into_iter(self) -> IntoIter<K, V> {
        IntoIter {
            curr_bucket: 0,
            map: self,
        }
    }
}

impl<K, V> FromIterator<(K, V)> for HashMap<K, V>
where
    K: Hash + Eq,
{
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (K, V)>,
    {
        let mut ret: HashMap<K, V> = HashMap::new();
        iter.into_iter().for_each(|(k, v)| {
            ret.insert(k, v);
        });
        ret
    }
}

impl<K, Q, V> Index<&Q> for HashMap<K, V>
where
    K: Eq + Hash + Borrow<Q>,
    Q: Eq + Hash + ?Sized,
{
    type Output = V;
    fn index(&self, key: &Q) -> &V {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);

        let bucket = (hasher.finish() % self.buckets.len() as u64) as usize;
        self.buckets[bucket]
            .iter()
            .find(|(k, _)| k.borrow() == key)
            .map(|(_, v)| v)
            .unwrap()
    }
}

impl<K, V, const N: usize> From<[(K, V); N]> for HashMap<K, V>
where
    K: Eq + Hash,
{
    fn from(value: [(K, V); N]) -> Self {
        let mut ret: HashMap<K, V> = HashMap::new();
        value.into_iter().for_each(|(k, v)| {
            ret.insert(k, v);
        });
        ret
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

        map.into_iter().for_each(|(k, v)| match k {
            "a" => assert_eq!(v, 10),
            "b" => assert_eq!(v, 20),
            "c" => assert_eq!(v, 30),
            "d" => assert_eq!(v, 40),
            _ => unreachable!(),
        });
    }
}
