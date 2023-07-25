use crate::hash;
use crate::PreHash;
use std::hash::Hash;

/// Stores a `T` value together with its pre-computed hash.
///
/// This is the primary implementor of `PreHash<T>`.
/// For convenience, it also implements `Deref<Target = T>` and `AsRef<T>`.
#[derive(Clone, Copy, Debug)]
pub struct WithHash<T: ?Sized> {
    hash: u64,

    /// Invariant: do not give out `&mut T`
    value: T,
}

/// Computes the hash of `value` and returns a new [`WithHash`] struct.
impl<T: Hash> From<T> for WithHash<T> {
    fn from(value: T) -> Self {
        Self {
            hash: hash(&value),
            value,
        }
    }
}

impl<T: ?Sized> std::ops::Deref for WithHash<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: ?Sized> AsRef<T> for WithHash<T> {
    fn as_ref(&self) -> &T {
        &self.value
    }
}

impl<T: ?Sized> PreHash for WithHash<T> {
    type Hashed = T;

    fn precomputed_hash(self_: &Self) -> u64 {
        self_.hash
    }

    fn hashed_value(self_: &Self) -> &Self::Hashed {
        &self_.value
    }
}

impl<T: ?Sized> PreHash for &'_ WithHash<T> {
    type Hashed = T;

    fn precomputed_hash(self_: &Self) -> u64 {
        self_.hash
    }

    fn hashed_value(self_: &Self) -> &Self::Hashed {
        &self_.value
    }
}

impl<T: ?Sized> PreHash for &'_ mut WithHash<T> {
    type Hashed = T;

    fn precomputed_hash(self_: &Self) -> u64 {
        self_.hash
    }

    fn hashed_value(self_: &Self) -> &Self::Hashed {
        &self_.value
    }
}

impl<T: ?Sized> PreHash for Box<WithHash<T>> {
    type Hashed = T;

    fn precomputed_hash(self_: &Self) -> u64 {
        self_.hash
    }

    fn hashed_value(self_: &Self) -> &Self::Hashed {
        &self_.value
    }
}

impl<T: ?Sized> PreHash for std::rc::Rc<WithHash<T>> {
    type Hashed = T;

    fn precomputed_hash(self_: &Self) -> u64 {
        self_.hash
    }

    fn hashed_value(self_: &Self) -> &Self::Hashed {
        &self_.value
    }
}

impl<T: ?Sized> PreHash for std::sync::Arc<WithHash<T>> {
    type Hashed = T;

    fn precomputed_hash(self_: &Self) -> u64 {
        self_.hash
    }

    fn hashed_value(self_: &Self) -> &Self::Hashed {
        &self_.value
    }
}
