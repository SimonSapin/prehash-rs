use crate::hash;
use crate::PreHash;
use std::alloc::Layout;
use std::hash::Hash;

/// Stores a `T` value together with its pre-computed hash.
///
/// This is the primary implementor of `PreHash<T>`.
/// For convenience, it also implements `Deref<Target = T>` and `AsRef<T>`.
#[repr(C)] // Ensures a layout compatible with `Layout::extend`
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

impl<T> WithHash<[T]> {
    fn new_raw_boxed_slice(input_slice: &'_ [T]) -> *mut Self
    where
        T: Copy + Hash,
    {
        let (layout, value_offset) = Self::slice_layout(input_slice.len());
        unsafe {
            // SAFETY: `layout` is not zero size since `WithHash` has a `hash: u64` field.
            let struct_ptr: *mut u8 = crate::alloc::alloc(layout);

            // SAFETY: allocated from the appropriate layout
            Self::initialize(struct_ptr, value_offset, input_slice);

            Self::as_wide_ptr(struct_ptr, input_slice.len())
        }
    }

    fn slice_layout(len: usize) -> (Layout, usize) {
        // SAFETY: must match the #[repr(C)] layout of the `WithHash` struct
        let hash_layout = Layout::new::<u64>();
        let value_layout = Layout::array::<T>(len).expect("layout computation overflow");
        let (layout, offset) = hash_layout
            .extend(value_layout)
            .expect("layout computation overflow");
        (layout.pad_to_align(), offset)
    }

    /// SAFETY: `struct_ptr` must be valid for the layout returned by `slice_layout`
    unsafe fn initialize(struct_ptr: *mut u8, value_offset: usize, input_slice: &[T])
    where
        T: Copy + Hash,
    {
        // The first field of the struct is at offset 0:
        let hash_ptr = struct_ptr.cast::<u64>();
        // SAFETY: pointer is valid (from a successful allocation)
        // and aligned (from `Layout` computation)
        unsafe { hash_ptr.write(hash(input_slice)) }
        // SAFETY: both pointers are within a successful allocation
        // `value_offset` is counted in bytes, so add it before casting.
        let value_ptr = unsafe { struct_ptr.add(value_offset) }.cast::<T>();
        // SAFETY: the ranges do not overlap (dest is a brand new allocation)
        // and are both valid.
        unsafe { value_ptr.copy_from_nonoverlapping(input_slice.as_ptr(), input_slice.len()) }
    }

    fn as_wide_ptr(data: *mut u8, slice_len: usize) -> *mut Self {
        // TODO: use `ptr::from_raw_parts_mut` when available (https://github.com/rust-lang/rust/issues/81513)

        // Until then, `slice_from_raw_parts_mut` returns a raw wide pointer with the wrong type
        // but the correct components (data pointer and length metadata).
        let raw_slice = core::ptr::slice_from_raw_parts_mut::<T>(data.cast::<T>(), slice_len);
        // This cast preserves both pointer components.
        raw_slice as *mut WithHash<[T]>
    }
}

impl<T: Copy + Hash> From<&'_ [T]> for Box<WithHash<[T]>> {
    fn from(value: &'_ [T]) -> Self {
        let ptr = WithHash::new_raw_boxed_slice(value);

        // SAFETY: points to a fully initialized allocation with the appropriate layout
        unsafe { Box::from_raw(ptr) }
    }
}

impl From<&'_ str> for Box<WithHash<str>> {
    fn from(value: &'_ str) -> Self {
        let ptr: *mut WithHash<[u8]> = WithHash::new_raw_boxed_slice(value.as_bytes());

        // The wide pointer metadata is compatible between `*[u8]` and `*str`
        // (the length as a `usize` counting bytes)
        let ptr = ptr as *mut WithHash<str>;

        // SAFETY: points to a fully initialized allocation with the appropriate layout
        unsafe { Box::from_raw(ptr) }
    }
}
