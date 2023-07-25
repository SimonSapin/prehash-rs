//! Like `std::rc::Rc` but:
//!
//! * Does not have weak references
//! * Supports dynamically-sized conversion to `Rc<WithHash<str>>` or `Rc<WithHash<[T]>>`

use crate::WithHash;
use std::alloc::Layout;
use std::cell::Cell;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ptr::NonNull;

#[repr(C)]
struct RcBox<T: ?Sized> {
    refcount: Cell<usize>,
    value: T,
}

fn rcbox_layout_and_value_offset(value_layout: Layout) -> (Layout, usize) {
    let (layout, value_offset) = Layout::new::<Cell<usize>>().extend(value_layout).unwrap();
    (layout.pad_to_align(), value_offset)
}

pub struct Rc<T: ?Sized> {
    ptr: NonNull<RcBox<T>>,
    phantom: PhantomData<RcBox<T>>,
}

impl<T: ?Sized> Rc<T> {
    #[inline(always)]
    fn inner(&self) -> &RcBox<T> {
        // SAFETY: While this Rc is alive we’re guaranteed that the inner pointer is valid.
        unsafe { self.ptr.as_ref() }
    }
}

// Implicit `T: Sized`
impl<T> Rc<T> {
    pub fn new(value: T) -> Self {
        Self {
            ptr: Box::leak(Box::new(RcBox {
                refcount: Cell::new(1),
                value,
            }))
            .into(),
            phantom: PhantomData,
        }
    }
}

impl<T: ?Sized> Drop for Rc<T> {
    fn drop(&mut self) {
        let new_count = self.inner().refcount.get() - 1;
        if new_count != 0 {
            self.inner().refcount.set(new_count);
        } else {
            unsafe {
                let layout = Layout::for_value(self.ptr.as_ref());
                std::ptr::drop_in_place(&mut self.ptr.as_mut().value);
                std::alloc::dealloc(self.ptr.cast().as_ptr(), layout);
            }
        }
    }
}

impl<T: ?Sized> Clone for Rc<T> {
    fn clone(&self) -> Self {
        let new_count = self.inner().refcount.get().checked_add(1).unwrap();
        self.inner().refcount.set(new_count);
        Self {
            ptr: self.ptr,
            phantom: PhantomData,
        }
    }
}

impl<T: ?Sized> std::ops::Deref for Rc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner().value
    }
}

impl<T: ?Sized> AsRef<T> for Rc<T> {
    fn as_ref(&self) -> &T {
        &self.inner().value
    }
}

// Implicit `T: Sized`
impl<T> From<T> for Rc<T> {
    fn from(value: T) -> Self {
        Rc::new(value)
    }
}

fn new_dynamically_sized_rcbox(value_layout: Layout) -> (NonNull<u8>, usize) {
    let (layout, rcvalue_offset) = rcbox_layout_and_value_offset(value_layout);
    unsafe {
        let rcbox_ptr: NonNull<u8> = crate::alloc::alloc(layout);

        let refcount_ptr = rcbox_ptr.cast::<Cell<usize>>();
        refcount_ptr.as_ptr().write(Cell::new(1));

        (rcbox_ptr, rcvalue_offset)
    }
}

impl<T> RcBox<WithHash<[T]>> {
    fn new_withhash_slice(input_slice: &[T]) -> NonNull<Self>
    where
        T: Copy + Hash,
    {
        let (withhash_layout, slice_offset) = WithHash::slice_layout_and_value_offset(input_slice);
        let (rcbox_ptr, rcvalue_offset) = new_dynamically_sized_rcbox(withhash_layout);
        unsafe {
            let withhash_ptr = rcbox_ptr.as_ptr().add(rcvalue_offset);
            WithHash::initialize(withhash_ptr, slice_offset, input_slice);

            // TODO: use `ptr::from_raw_parts_mut` when available (https://github.com/rust-lang/rust/issues/81513)

            // Until then, `slice_from_raw_parts_mut` returns a raw wide pointer with the wrong type
            // but the correct components (data pointer and length metadata).
            let raw_slice = core::ptr::slice_from_raw_parts_mut::<T>(
                rcbox_ptr.as_ptr().cast::<T>(),
                input_slice.len(),
            );
            // This cast preserves both pointer components.
            NonNull::new_unchecked(raw_slice as *mut RcBox<WithHash<[T]>>)
        }
    }
}

impl<T: Copy + Hash> From<&'_ [T]> for Rc<WithHash<[T]>> {
    fn from(input_slice: &'_ [T]) -> Self {
        Self {
            ptr: RcBox::new_withhash_slice(input_slice),
            phantom: PhantomData,
        }
    }
}
