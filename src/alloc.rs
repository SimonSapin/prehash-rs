use std::alloc::Layout;

/// Allocate and handle allocation errors.
///
/// SAFETY: `layout` must have non-zero size
pub(crate) unsafe fn alloc(layout: Layout) -> *mut u8 {
    let ptr: *mut u8 = unsafe { std::alloc::alloc(layout) };
    if ptr.is_null() {
        std::alloc::handle_alloc_error(layout)
    }
    ptr
}
