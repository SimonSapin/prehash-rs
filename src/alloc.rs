use std::alloc::Layout;
use std::ptr::NonNull;

/// Allocate and handle allocation errors.
///
/// SAFETY: `layout` must have non-zero size
pub(crate) unsafe fn alloc(layout: Layout) -> NonNull<u8> {
    let ptr: *mut u8 = unsafe { std::alloc::alloc(layout) };
    NonNull::new(ptr).unwrap_or_else(|| std::alloc::handle_alloc_error(layout))
}
