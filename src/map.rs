use crate::PreHash;
use hashbrown::hash_map::RawEntryMut;
use hashbrown::Equivalent;
use hashbrown::HashMap;

pub struct PreHashMap<K, V>
where
    K: PreHash,
{
    hashbrown: HashMap<K, V, NotAHasher>,
}

/// This does *not* implement `BuildHasher`. We never want Hashbrown to do the hashing.
struct NotAHasher;

impl<K, V> PreHashMap<K, V>
where
    K: PreHash,
    K::Hashed: Eq,
{
    pub fn new() -> Self {
        Self {
            hashbrown: HashMap::with_hasher(NotAHasher),
        }
    }

    fn raw_entry<Q>(&self, key: &Q) -> Option<(&K, &V)>
    where
        Q: PreHash,
        Q::Hashed: Equivalent<K::Hashed>,
    {
        let hash = PreHash::precomputed_hash(key);
        self.hashbrown.raw_entry().from_hash(hash, |candidate| {
            PreHash::hashed_value(key).equivalent(PreHash::hashed_value(candidate))
        })
    }

    fn raw_entry_mut<Q>(&mut self, key: &Q) -> RawEntryMut<'_, K, V, NotAHasher>
    where
        Q: PreHash,
        Q::Hashed: Equivalent<K::Hashed>,
    {
        let hash = PreHash::precomputed_hash(key);
        self.hashbrown.raw_entry_mut().from_hash(hash, |candidate| {
            PreHash::hashed_value(key).equivalent(PreHash::hashed_value(candidate))
        })
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        Q: PreHash,
        Q::Hashed: Equivalent<K::Hashed>,
    {
        self.raw_entry(key).map(|(_key, value)| value)
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        match self.raw_entry_mut(&key) {
            RawEntryMut::Occupied(mut entry) => Some(entry.insert(value)),
            RawEntryMut::Vacant(entry) => {
                let hash = PreHash::precomputed_hash(&key);
                entry.insert_with_hasher(hash, key, value, PreHash::precomputed_hash);
                None
            }
        }
    }
}
