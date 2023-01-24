#![feature(min_specialization)]

use crate::alloc::Allocator;
use crate::co_alloc::CoAllocPref;
use crate::vec;
use core::iter::TrustedLen;
use core::slice;

use super::VecDeque;

// Specialization trait used for VecDeque::extend
pub(super) trait SpecExtend<T, I> {
    fn spec_extend(&mut self, iter: I);
}

#[allow(unused_braces)]
impl<T, I, A: Allocator, const CO_ALLOC_PREF: CoAllocPref> SpecExtend<T, I>
    for VecDeque<T, A, CO_ALLOC_PREF>
where
    I: Iterator<Item = T>,
    [(); { crate::meta_num_slots!(A, CO_ALLOC_PREF) }]:,
{
    default fn spec_extend(&mut self, mut iter: I) {
        // This function should be the moral equivalent of:
        //
        // for item in iter {
        //     self.push_back(item);
        // }

        // May only be called if `deque.len() < deque.capacity()`
        unsafe fn push_unchecked<T, A: Allocator, const CO_ALLOC_PREF: CoAllocPref>(
            deque: &mut VecDeque<T, A, CO_ALLOC_PREF>,
            element: T,
        ) where
            [(); { crate::meta_num_slots!(A, CO_ALLOC_PREF) }]:,
        {
            // SAFETY: Because of the precondition, it's guaranteed that there is space
            // in the logical array after the last element.
            unsafe { deque.buffer_write(deque.to_physical_idx(deque.len), element) };
            // This can't overflow because `deque.len() < deque.capacity() <= usize::MAX`.
            deque.len += 1;
        }

        while let Some(element) = iter.next() {
            let (lower, _) = iter.size_hint();
            self.reserve(lower.saturating_add(1));

            // SAFETY: We just reserved space for at least one element.
            unsafe { push_unchecked(self, element) };

            // Inner loop to avoid repeatedly calling `reserve`.
            while self.len < self.capacity() {
                let Some(element) = iter.next() else {
                    return;
                };
                // SAFETY: The loop condition guarantees that `self.len() < self.capacity()`.
                unsafe { push_unchecked(self, element) };
            }
        }
    }
}

#[allow(unused_braces)]
impl<T, I, A: Allocator, const CO_ALLOC_PREF: CoAllocPref> SpecExtend<T, I>
    for VecDeque<T, A, CO_ALLOC_PREF>
where
    I: TrustedLen<Item = T>,
    [(); { crate::meta_num_slots!(A, CO_ALLOC_PREF) }]:,
{
    default fn spec_extend(&mut self, iter: I) {
        // This is the case for a TrustedLen iterator.
        let (low, high) = iter.size_hint();
        if let Some(additional) = high {
            debug_assert_eq!(
                low,
                additional,
                "TrustedLen iterator's size hint is not exact: {:?}",
                (low, high)
            );
            self.reserve(additional);

            let written = unsafe {
                self.write_iter_wrapping(self.to_physical_idx(self.len), iter, additional)
            };

            debug_assert_eq!(
                additional, written,
                "The number of items written to VecDeque doesn't match the TrustedLen size hint"
            );
        } else {
            // Per TrustedLen contract a `None` upper bound means that the iterator length
            // truly exceeds usize::MAX, which would eventually lead to a capacity overflow anyway.
            // Since the other branch already panics eagerly (via `reserve()`) we do the same here.
            // This avoids additional codegen for a fallback code path which would eventually
            // panic anyway.
            panic!("capacity overflow");
        }
    }
}

#[allow(unused_braces)]
impl<T, A: Allocator, const CO_ALLOC_PREF: CoAllocPref> SpecExtend<T, vec::IntoIter<T>>
    for VecDeque<T, A, CO_ALLOC_PREF>
where
    [(); { crate::meta_num_slots!(A, CO_ALLOC_PREF) }]:,
{
    fn spec_extend(&mut self, mut iterator: vec::IntoIter<T>) {
        let slice = iterator.as_slice();
        self.reserve(slice.len());

        unsafe {
            self.copy_slice(self.to_physical_idx(self.len), slice);
            self.len += slice.len();
        }
        iterator.forget_remaining_elements();
    }
}

#[allow(unused_braces)]
impl<'a, T: 'a, I, A: Allocator, const CO_ALLOC_PREF: CoAllocPref> SpecExtend<&'a T, I>
    for VecDeque<T, A, CO_ALLOC_PREF>
where
    I: Iterator<Item = &'a T>,
    T: Copy,
    [(); { crate::meta_num_slots!(A, CO_ALLOC_PREF) }]:,
{
    default fn spec_extend(&mut self, iterator: I) {
        self.spec_extend(iterator.copied())
    }
}

#[allow(unused_braces)]
impl<'a, T: 'a, A: Allocator, const CO_ALLOC_PREF: CoAllocPref>
    SpecExtend<&'a T, slice::Iter<'a, T>> for VecDeque<T, A, CO_ALLOC_PREF>
where
    T: Copy,
    [(); { crate::meta_num_slots!(A, CO_ALLOC_PREF) }]:,
{
    fn spec_extend(&mut self, iterator: slice::Iter<'a, T>) {
        let slice = iterator.as_slice();
        self.reserve(slice.len());

        unsafe {
            self.copy_slice(self.to_physical_idx(self.len), slice);
            self.len += slice.len();
        }
    }
}
