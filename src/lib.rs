use std::collections::hash_map::RandomState;
use std::hash::BuildHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::sync::OnceLock;

mod alloc;
mod map;
mod rc;
mod with_hash;

pub use self::map::PreHashMap;
pub use self::rc::Rc;
pub use self::with_hash::WithHash;

/// Computes and returns the hash of `value`,
/// with a hasher configured randomly once per process.
///
/// This is not as much DoS protection as the standard library’s default `HashMap`
/// where each map gets its own `RandomState`.
pub fn hash<T: ?Sized + Hash>(value: &T) -> u64 {
    static SHARED_RANDOM: OnceLock<RandomState> = OnceLock::new();
    let mut hasher = SHARED_RANDOM.get_or_init(RandomState::new).build_hasher();
    value.hash(&mut hasher);
    hasher.finish()
}

/// Associated functions of this trait are not methods taking `&self`
/// to avoid shadowing potential methods of the target type when implementing this trait
/// on something that implements `Deref`.
pub trait PreHash {
    type Hashed: ?Sized;

    /// Returns the stored result of [`hash`] for the [`hashed_value()`][Self::hashed_value].
    ///
    /// Multiple calls on the same `&Self` value must return the same `u64` hash.
    fn precomputed_hash(self_: &Self) -> u64;

    /// Returns a reference to the value whose hash is stored.
    fn hashed_value(self_: &Self) -> &Self::Hashed;
}
