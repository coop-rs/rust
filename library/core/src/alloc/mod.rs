//! Memory allocation APIs

#![stable(feature = "alloc_module", since = "1.28.0")]

mod global;
mod layout;

#[stable(feature = "global_alloc", since = "1.28.0")]
pub use self::global::GlobalAlloc;
#[stable(feature = "alloc_layout", since = "1.28.0")]
pub use self::layout::Layout;
#[stable(feature = "alloc_layout", since = "1.28.0")]
#[deprecated(
    since = "1.52.0",
    note = "Name does not follow std convention, use LayoutError",
    suggestion = "LayoutError"
)]
#[allow(deprecated, deprecated_in_future)]
pub use self::layout::LayoutErr;

#[stable(feature = "alloc_layout_error", since = "1.50.0")]
pub use self::layout::LayoutError;

use crate::error::Error;
use crate::fmt;
use crate::ptr::{self, NonNull};

/// The `AllocError` error indicates an allocation failure
/// that may be due to resource exhaustion or to
/// something wrong when combining the given input arguments with this
/// allocator.
#[unstable(feature = "allocator_api", issue = "32838")]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct AllocError;

#[unstable(
    feature = "allocator_api",
    reason = "the precise API and guarantees it provides may be tweaked.",
    issue = "32838"
)]
impl Error for AllocError {}

// (we need this for downstream impl of trait Error)
#[unstable(feature = "allocator_api", issue = "32838")]
impl fmt::Display for AllocError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("memory allocation failed")
    }
}

/// (Non-Null) Pointer and coallocation metadata.
#[unstable(feature = "global_co_alloc_meta", issue = "none")]
#[derive(Clone, Copy, Debug)]
pub struct PtrAndMeta<M: ~const CoAllocMetaBase> {
    pub ptr: NonNull<u8>,
    pub meta: M,
}

/// (NonNull) Slice and coallocation metadata.
#[unstable(feature = "global_co_alloc_meta", issue = "none")]
#[derive(Clone, Copy, Debug)]
/// Used for results (from `CoAllocator`'s functions, where applicable).
pub struct SliceAndMeta<M: ~const CoAllocMetaBase> {
    pub slice: NonNull<[u8]>,
    pub meta: M,
}

/// `Result` of `SliceAndMeta` or `AllocError`.
#[unstable(feature = "global_co_alloc_meta", issue = "none")]
pub type SliceAndMetaResult<M> = Result<SliceAndMeta<M>, AllocError>;

#[unstable(feature = "global_co_alloc_meta", issue = "none")]
#[const_trait]
pub trait CoAllocMetaBase: Clone + Copy {
    /// NOT for public use. This MAY BE REMOVED or CHANGED.
    ///
    /// For EXPERIMENTATION only.
    const ZERO_METAS: [Self; 0];
    const ONE_METAS: [Self; 1];

    /// NOT for public use. This MAY BE REMOVED or CHANGED.
    ///
    /// For EXPERIMENTATION only.
    fn new_plain() -> Self;
}

#[unstable(feature = "global_co_alloc_meta", issue = "none")]
#[derive(Clone, Copy, Debug)]
pub struct CoAllocMetaPlain {}

const CO_ALLOC_META_PLAIN: CoAllocMetaPlain = CoAllocMetaPlain {};

#[unstable(feature = "global_co_alloc_meta", issue = "none")]
impl const CoAllocMetaBase for CoAllocMetaPlain {
    const ZERO_METAS: [Self; 0] = [];
    const ONE_METAS: [Self; 1] = [CO_ALLOC_META_PLAIN];

    fn new_plain() -> Self {
        CO_ALLOC_META_PLAIN
    }
}

/// Whether an `Allocator` implementation supports coallocation.
///
/// This type WILL CHANGE (once ``#![feature(generic_const_exprs)]` and
/// `#![feature(adt_const_params)]` are stable) to a dedicated struct/enum. Hence:
/// - DO NOT mix this/cast this with/to `u8`, `u16`, (nor any other integer); and
/// - DO NOT hard code any values, but use `CO_ALLOCATOR_SUPPORTS_META_YES` and `CO_ALLOCATOR_SUPPORTS_META_NO`.
// @FIXME Once ICE is fixed: Change to `u32` (or any other unused unsinged integer type, and other
// than `usize`, so we can't mix it up with `usize`).
#[unstable(feature = "global_co_alloc_meta", issue = "none")]
pub type CoAllocatorMetaNumSlots = usize;

/// Indicating that an Allocator supports coallocation (if a type of the allocated instances supports it, too).
#[unstable(feature = "global_co_alloc_meta", issue = "none")]
pub const CO_ALLOCATOR_SUPPORTS_META_YES: CoAllocatorMetaNumSlots = 1;

/// Indicating that an Allocator does not support coallocation.
#[unstable(feature = "global_co_alloc_meta", issue = "none")]
pub const CO_ALLOCATOR_SUPPORTS_META_NO: CoAllocatorMetaNumSlots = 0;

/// An implementation of `Allocator` can allocate, grow, shrink, and deallocate arbitrary blocks of
/// data described via [`Layout`][].
///
/// `Allocator` is designed to be implemented on ZSTs, references, or smart pointers because having
/// an allocator like `MyAlloc([u8; N])` cannot be moved, without updating the pointers to the
/// allocated memory.
///
/// Unlike [`GlobalAlloc`][], zero-sized allocations are allowed in `Allocator`. If an underlying
/// allocator does not support this (like jemalloc) or return a null pointer (such as
/// `libc::malloc`), this must be caught by the implementation.
///
/// ### Currently allocated memory
///
/// Some of the methods require that a memory block be *currently allocated* via an allocator. This
/// means that:
///
/// * the starting address for that memory block was previously returned by [`allocate`], [`grow`], or
///   [`shrink`], and
///
/// * the memory block has not been subsequently deallocated, where blocks are either deallocated
///   directly by being passed to [`deallocate`] or were changed by being passed to [`grow`] or
///   [`shrink`] that returns `Ok`. If `grow` or `shrink` have returned `Err`, the passed pointer
///   remains valid.
///
/// [`allocate`]: Allocator::allocate
/// [`grow`]: Allocator::grow
/// [`shrink`]: Allocator::shrink
/// [`deallocate`]: Allocator::deallocate
///
/// ### Memory fitting
///
/// Some of the methods require that a layout *fit* a memory block. What it means for a layout to
/// "fit" a memory block means (or equivalently, for a memory block to "fit" a layout) is that the
/// following conditions must hold:
///
/// * The block must be allocated with the same alignment as [`layout.align()`], and
///
/// * The provided [`layout.size()`] must fall in the range `min ..= max`, where:
///   - `min` is the size of the layout most recently used to allocate the block, and
///   - `max` is the latest actual size returned from [`allocate`], [`grow`], or [`shrink`].
///
/// [`layout.align()`]: Layout::align
/// [`layout.size()`]: Layout::size
///
/// # Safety
///
/// * Memory blocks returned from an allocator must point to valid memory and retain their validity
///   until the instance and all of its clones are dropped,
///
/// * cloning or moving the allocator must not invalidate memory blocks returned from this
///   allocator. A cloned allocator must behave like the same allocator, and
///
/// * any pointer to a memory block which is [*currently allocated*] may be passed to any other
///   method of the allocator.
///
/// [*currently allocated*]: #currently-allocated-memory
#[unstable(feature = "allocator_api", issue = "32838")]
#[const_trait]
pub unsafe trait Allocator {
    /// NOT for public use. MAY CHANGE.
    const CO_ALLOC_META_NUM_SLOTS: CoAllocatorMetaNumSlots = CO_ALLOCATOR_SUPPORTS_META_NO;

    /// Type to store coallocation metadata (if both the allocator and the heap-based type support
    /// coallocation, and if coallocation is used).
    ///
    /// If this is any type with non-zero size, then the actual `Allocator` implementation supports
    /// cooperative functions (`co_*`) as first class citizens. NOT for public use. The default
    /// value MAY be REMOVED or CHANGED.
    ///
    /// @FIXME Validate (preferrable at compile time, otherwise as a test) that this type's
    /// alignment <= `usize` alignment.
    type CoAllocMeta: ~const CoAllocMetaBase = CoAllocMetaPlain;

    /// Attempts to allocate a block of memory.
    ///
    /// On success, returns a [`NonNull<[u8]>`][NonNull] meeting the size and alignment guarantees of `layout`.
    ///
    /// The returned block may have a larger size than specified by `layout.size()`, and may or may
    /// not have its contents initialized.
    ///
    /// # Errors
    ///
    /// Returning `Err` indicates that either memory is exhausted or `layout` does not meet
    /// allocator's size or alignment constraints.
    ///
    /// Implementations are encouraged to return `Err` on memory exhaustion rather than panicking or
    /// aborting, but this is not a strict requirement. (Specifically: it is *legal* to implement
    /// this trait atop an underlying native allocation library that aborts on memory exhaustion.)
    ///
    /// Clients wishing to abort computation in response to an allocation error are encouraged to
    /// call the [`handle_alloc_error`] function, rather than directly invoking `panic!` or similar.
    ///
    /// [`handle_alloc_error`]: ../../alloc/alloc/fn.handle_alloc_error.html
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError>;

    fn co_allocate(&self, _layout: Layout, _result: &mut SliceAndMetaResult<Self::CoAllocMeta>) {
        panic!("FIXME")
    }

    /// Behaves like `allocate`, but also ensures that the returned memory is zero-initialized.
    ///
    /// # Errors
    ///
    /// Returning `Err` indicates that either memory is exhausted or `layout` does not meet
    /// allocator's size or alignment constraints.
    ///
    /// Implementations are encouraged to return `Err` on memory exhaustion rather than panicking or
    /// aborting, but this is not a strict requirement. (Specifically: it is *legal* to implement
    /// this trait atop an underlying native allocation library that aborts on memory exhaustion.)
    ///
    /// Clients wishing to abort computation in response to an allocation error are encouraged to
    /// call the [`handle_alloc_error`] function, rather than directly invoking `panic!` or similar.
    ///
    /// [`handle_alloc_error`]: ../../alloc/alloc/fn.handle_alloc_error.html
    fn allocate_zeroed(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let ptr = self.allocate(layout)?;
        // SAFETY: `alloc` returns a valid memory block
        unsafe { ptr.as_non_null_ptr().as_ptr().write_bytes(0, ptr.len()) }
        Ok(ptr)
    }

    fn co_allocate_zeroed(
        &self,
        layout: Layout,
        mut result: &mut SliceAndMetaResult<Self::CoAllocMeta>,
    ) {
        self.co_allocate(layout, &mut result);
        if let Ok(SliceAndMeta { slice, .. }) = result {
            // SAFETY: `alloc` returns a valid memory block
            unsafe { slice.as_non_null_ptr().as_ptr().write_bytes(0, slice.len()) }
        }
    }

    /// Deallocates the memory referenced by `ptr`.
    ///
    /// # Safety
    ///
    /// * `ptr` must denote a block of memory [*currently allocated*] via this allocator, and
    /// * `layout` must [*fit*] that block of memory.
    ///
    /// [*currently allocated*]: #currently-allocated-memory
    /// [*fit*]: #memory-fitting
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout);

    unsafe fn co_deallocate(&self, _ptr_and_meta: PtrAndMeta<Self::CoAllocMeta>, _layout: Layout) {
        panic!("FIXME")
    }

    /// Attempts to extend the memory block.
    ///
    /// Returns a new [`NonNull<[u8]>`][NonNull] containing a pointer and the actual size of the allocated
    /// memory. The pointer is suitable for holding data described by `new_layout`. To accomplish
    /// this, the allocator may extend the allocation referenced by `ptr` to fit the new layout.
    ///
    /// If this returns `Ok`, then ownership of the memory block referenced by `ptr` has been
    /// transferred to this allocator. Any access to the old `ptr` is Undefined Behavior, even if the
    /// allocation was grown in-place. The newly returned pointer is the only valid pointer
    /// for accessing this memory now.
    ///
    /// If this method returns `Err`, then ownership of the memory block has not been transferred to
    /// this allocator, and the contents of the memory block are unaltered.
    ///
    /// # Safety
    ///
    /// * `ptr` must denote a block of memory [*currently allocated*] via this allocator.
    /// * `old_layout` must [*fit*] that block of memory (The `new_layout` argument need not fit it.).
    /// * `new_layout.size()` must be greater than or equal to `old_layout.size()`.
    ///
    /// Note that `new_layout.align()` need not be the same as `old_layout.align()`.
    ///
    /// [*currently allocated*]: #currently-allocated-memory
    /// [*fit*]: #memory-fitting
    ///
    /// # Errors
    ///
    /// Returns `Err` if the new layout does not meet the allocator's size and alignment
    /// constraints of the allocator, or if growing otherwise fails.
    ///
    /// Implementations are encouraged to return `Err` on memory exhaustion rather than panicking or
    /// aborting, but this is not a strict requirement. (Specifically: it is *legal* to implement
    /// this trait atop an underlying native allocation library that aborts on memory exhaustion.)
    ///
    /// Clients wishing to abort computation in response to an allocation error are encouraged to
    /// call the [`handle_alloc_error`] function, rather than directly invoking `panic!` or similar.
    ///
    /// [`handle_alloc_error`]: ../../alloc/alloc/fn.handle_alloc_error.html
    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        debug_assert!(
            new_layout.size() >= old_layout.size(),
            "`new_layout.size()` must be greater than or equal to `old_layout.size()`"
        );

        let new_ptr = self.allocate(new_layout)?;

        // SAFETY: because `new_layout.size()` must be greater than or equal to
        // `old_layout.size()`, both the old and new memory allocation are valid for reads and
        // writes for `old_layout.size()` bytes. Also, because the old allocation wasn't yet
        // deallocated, it cannot overlap `new_ptr`. Thus, the call to `copy_nonoverlapping` is
        // safe. The safety contract for `dealloc` must be upheld by the caller.
        unsafe {
            ptr::copy_nonoverlapping(ptr.as_ptr(), new_ptr.as_mut_ptr(), old_layout.size());
            self.deallocate(ptr, old_layout);
        }

        Ok(new_ptr)
    }

    unsafe fn co_grow(
        &self,
        ptr_and_meta: PtrAndMeta<Self::CoAllocMeta>,
        old_layout: Layout,
        new_layout: Layout,
        mut result: &mut SliceAndMetaResult<Self::CoAllocMeta>,
    ) {
        debug_assert!(
            new_layout.size() >= old_layout.size(),
            "`new_layout.size()` must be greater than or equal to `old_layout.size()`"
        );

        self.co_allocate(new_layout, &mut result);

        if let Ok(SliceAndMeta { slice, .. }) = result {
            // SAFETY: because `new_layout.size()` must be greater than or equal to
            // `old_layout.size()`, both the old and new memory allocation are valid for reads and
            // writes for `old_layout.size()` bytes. Also, because the old allocation wasn't yet
            // deallocated, it cannot overlap `new_slice_and_meta.slice`. Thus, the call to `copy_nonoverlapping` is
            // safe. The safety contract for `dealloc` must be upheld by the caller.
            unsafe {
                ptr::copy_nonoverlapping(
                    ptr_and_meta.ptr.as_ptr(),
                    slice.as_mut_ptr(),
                    old_layout.size(),
                );
                self.co_deallocate(ptr_and_meta, old_layout);
            }
        }
    }

    /// Behaves like `grow`, but also ensures that the new contents are set to zero before being
    /// returned.
    ///
    /// The memory block will contain the following contents after a successful call to
    /// `grow_zeroed`:
    ///   * Bytes `0..old_layout.size()` are preserved from the original allocation.
    ///   * Bytes `old_layout.size()..old_size` will either be preserved or zeroed, depending on
    ///     the allocator implementation. `old_size` refers to the size of the memory block prior
    ///     to the `grow_zeroed` call, which may be larger than the size that was originally
    ///     requested when it was allocated.
    ///   * Bytes `old_size..new_size` are zeroed. `new_size` refers to the size of the memory
    ///     block returned by the `grow_zeroed` call.
    ///
    /// # Safety
    ///
    /// * `ptr` must denote a block of memory [*currently allocated*] via this allocator.
    /// * `old_layout` must [*fit*] that block of memory (The `new_layout` argument need not fit it.).
    /// * `new_layout.size()` must be greater than or equal to `old_layout.size()`.
    ///
    /// Note that `new_layout.align()` need not be the same as `old_layout.align()`.
    ///
    /// [*currently allocated*]: #currently-allocated-memory
    /// [*fit*]: #memory-fitting
    ///
    /// # Errors
    ///
    /// Returns `Err` if the new layout does not meet the allocator's size and alignment
    /// constraints of the allocator, or if growing otherwise fails.
    ///
    /// Implementations are encouraged to return `Err` on memory exhaustion rather than panicking or
    /// aborting, but this is not a strict requirement. (Specifically: it is *legal* to implement
    /// this trait atop an underlying native allocation library that aborts on memory exhaustion.)
    ///
    /// Clients wishing to abort computation in response to an allocation error are encouraged to
    /// call the [`handle_alloc_error`] function, rather than directly invoking `panic!` or similar.
    ///
    /// [`handle_alloc_error`]: ../../alloc/alloc/fn.handle_alloc_error.html
    unsafe fn grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        debug_assert!(
            new_layout.size() >= old_layout.size(),
            "`new_layout.size()` must be greater than or equal to `old_layout.size()`"
        );

        let new_ptr = self.allocate_zeroed(new_layout)?;

        // SAFETY: because `new_layout.size()` must be greater than or equal to
        // `old_layout.size()`, both the old and new memory allocation are valid for reads and
        // writes for `old_layout.size()` bytes. Also, because the old allocation wasn't yet
        // deallocated, it cannot overlap `new_ptr`. Thus, the call to `copy_nonoverlapping` is
        // safe. The safety contract for `dealloc` must be upheld by the caller.
        unsafe {
            ptr::copy_nonoverlapping(ptr.as_ptr(), new_ptr.as_mut_ptr(), old_layout.size());
            self.deallocate(ptr, old_layout);
        }

        Ok(new_ptr)
    }

    unsafe fn co_grow_zeroed(
        &self,
        ptr_and_meta: PtrAndMeta<Self::CoAllocMeta>,
        old_layout: Layout,
        new_layout: Layout,
        mut result: &mut SliceAndMetaResult<Self::CoAllocMeta>,
    ) {
        debug_assert!(
            new_layout.size() >= old_layout.size(),
            "`new_layout.size()` must be greater than or equal to `old_layout.size()`"
        );

        self.co_allocate_zeroed(new_layout, &mut result);

        if let Ok(SliceAndMeta { slice, .. }) = result {
            // SAFETY: because `new_layout.size()` must be greater than or equal to
            // `old_layout.size()`, both the old and new memory allocation are valid for reads and
            // writes for `old_layout.size()` bytes. Also, because the old allocation wasn't yet
            // deallocated, it cannot overlap `new_slice_and_meta.slice`. Thus, the call to `copy_nonoverlapping` is
            // safe. The safety contract for `dealloc` must be upheld by the caller.
            unsafe {
                ptr::copy_nonoverlapping(
                    ptr_and_meta.ptr.as_ptr(),
                    slice.as_mut_ptr(),
                    old_layout.size(),
                );
                self.co_deallocate(ptr_and_meta, old_layout);
            }
        }
    }

    /// Attempts to shrink the memory block.
    ///
    /// Returns a new [`NonNull<[u8]>`][NonNull] containing a pointer and the actual size of the allocated
    /// memory. The pointer is suitable for holding data described by `new_layout`. To accomplish
    /// this, the allocator may shrink the allocation referenced by `ptr` to fit the new layout.
    ///
    /// If this returns `Ok`, then ownership of the memory block referenced by `ptr` has been
    /// transferred to this allocator. Any access to the old `ptr` is Undefined Behavior, even if the
    /// allocation was shrunk in-place. The newly returned pointer is the only valid pointer
    /// for accessing this memory now.
    ///
    /// If this method returns `Err`, then ownership of the memory block has not been transferred to
    /// this allocator, and the contents of the memory block are unaltered.
    ///
    /// # Safety
    ///
    /// * `ptr` must denote a block of memory [*currently allocated*] via this allocator.
    /// * `old_layout` must [*fit*] that block of memory (The `new_layout` argument need not fit it.).
    /// * `new_layout.size()` must be smaller than or equal to `old_layout.size()`.
    ///
    /// Note that `new_layout.align()` need not be the same as `old_layout.align()`.
    ///
    /// [*currently allocated*]: #currently-allocated-memory
    /// [*fit*]: #memory-fitting
    ///
    /// # Errors
    ///
    /// Returns `Err` if the new layout does not meet the allocator's size and alignment
    /// constraints of the allocator, or if shrinking otherwise fails.
    ///
    /// Implementations are encouraged to return `Err` on memory exhaustion rather than panicking or
    /// aborting, but this is not a strict requirement. (Specifically: it is *legal* to implement
    /// this trait atop an underlying native allocation library that aborts on memory exhaustion.)
    ///
    /// Clients wishing to abort computation in response to an allocation error are encouraged to
    /// call the [`handle_alloc_error`] function, rather than directly invoking `panic!` or similar.
    ///
    /// [`handle_alloc_error`]: ../../alloc/alloc/fn.handle_alloc_error.html
    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        debug_assert!(
            new_layout.size() <= old_layout.size(),
            "`new_layout.size()` must be smaller than or equal to `old_layout.size()`"
        );

        let new_ptr = self.allocate(new_layout)?;

        // SAFETY: because `new_layout.size()` must be lower than or equal to
        // `old_layout.size()`, both the old and new memory allocation are valid for reads and
        // writes for `new_layout.size()` bytes. Also, because the old allocation wasn't yet
        // deallocated, it cannot overlap `new_ptr`. Thus, the call to `copy_nonoverlapping` is
        // safe. The safety contract for `dealloc` must be upheld by the caller.
        unsafe {
            ptr::copy_nonoverlapping(ptr.as_ptr(), new_ptr.as_mut_ptr(), new_layout.size());
            self.deallocate(ptr, old_layout);
        }

        Ok(new_ptr)
    }

    unsafe fn co_shrink(
        &self,
        ptr_and_meta: PtrAndMeta<Self::CoAllocMeta>,
        old_layout: Layout,
        new_layout: Layout,
        mut result: &mut SliceAndMetaResult<Self::CoAllocMeta>,
    ) {
        debug_assert!(
            new_layout.size() <= old_layout.size(),
            "`new_layout.size()` must be smaller than or equal to `old_layout.size()`"
        );

        self.co_allocate(new_layout, &mut result);

        if let Ok(SliceAndMeta { slice, .. }) = result {
            // SAFETY: because `new_layout.size()` must be lower than or equal to
            // `old_layout.size()`, both the old and new memory allocation are valid for reads and
            // writes for `new_layout.size()` bytes. Also, because the old allocation wasn't yet
            // deallocated, it cannot overlap `new_slice_and_meta.slice`. Thus, the call to `copy_nonoverlapping` is
            // safe. The safety contract for `dealloc` must be upheld by the caller.
            unsafe {
                ptr::copy_nonoverlapping(
                    ptr_and_meta.ptr.as_ptr(),
                    slice.as_mut_ptr(),
                    new_layout.size(),
                );
                self.co_deallocate(ptr_and_meta, old_layout);
            }
        }
    }

    /// Creates a "by reference" adapter for this instance of `Allocator`.
    ///
    /// The returned adapter also implements `Allocator` and will simply borrow this.
    #[inline(always)]
    fn by_ref(&self) -> &Self
    where
        Self: Sized,
    {
        self
    }
}

// @FIXME
#[unstable(feature = "allocator_api", issue = "32838")]
unsafe impl<A> Allocator for &A
where
    A: Allocator + ?Sized,
{
    #[inline]
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        (**self).allocate(layout)
    }

    #[inline]
    fn allocate_zeroed(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        (**self).allocate_zeroed(layout)
    }

    #[inline]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        // SAFETY: the safety contract must be upheld by the caller
        unsafe { (**self).deallocate(ptr, layout) }
    }

    #[inline]
    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        // SAFETY: the safety contract must be upheld by the caller
        unsafe { (**self).grow(ptr, old_layout, new_layout) }
    }

    #[inline]
    unsafe fn grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        // SAFETY: the safety contract must be upheld by the caller
        unsafe { (**self).grow_zeroed(ptr, old_layout, new_layout) }
    }

    #[inline]
    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        // SAFETY: the safety contract must be upheld by the caller
        unsafe { (**self).shrink(ptr, old_layout, new_layout) }
    }
}
