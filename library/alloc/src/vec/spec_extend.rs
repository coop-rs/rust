use core::alloc;
use crate::alloc::Allocator;
use core::iter::TrustedLen;
use core::slice::{self};

use super::{IntoIter, Vec};

// Specialization trait used for Vec::extend
pub(super) trait SpecExtend<T, I> {
    fn spec_extend(&mut self, iter: I);
}

impl<T, I, A: Allocator> SpecExtend<T, I> for Vec<T, A>
where
    I: Iterator<Item = T>,
    [(); alloc::co_alloc_metadata_num_slots::<A>()]:
{
    default fn spec_extend(&mut self, iter: I) {
        self.extend_desugared(iter)
    }
}

impl<T, I, A: Allocator> SpecExtend<T, I> for Vec<T, A>
where
    I: TrustedLen<Item = T>,
    [(); alloc::co_alloc_metadata_num_slots::<A>()]:
{
    default fn spec_extend(&mut self, iterator: I) {
        self.extend_trusted(iterator)
    }
}

impl<T, A: Allocator> SpecExtend<T, IntoIter<T>> for Vec<T, A>
where [(); alloc::co_alloc_metadata_num_slots::<A>()]: {
    fn spec_extend(&mut self, mut iterator: IntoIter<T>) {
        unsafe {
            self.append_elements(iterator.as_slice() as _);
        }
        iterator.forget_remaining_elements();
    }
}

impl<'a, T: 'a, I, A: Allocator + 'a> SpecExtend<&'a T, I> for Vec<T, A>
where
    I: Iterator<Item = &'a T>,
    T: Clone,
    [(); alloc::co_alloc_metadata_num_slots::<A>()]:
{
    default fn spec_extend(&mut self, iterator: I) {
        self.spec_extend(iterator.cloned())
    }
}

impl<'a, T: 'a, A: Allocator + 'a> SpecExtend<&'a T, slice::Iter<'a, T>> for Vec<T, A>
where
    T: Copy,
    [(); alloc::co_alloc_metadata_num_slots::<A>()]:
{
    fn spec_extend(&mut self, iterator: slice::Iter<'a, T>) {
        let slice = iterator.as_slice();
        unsafe { self.append_elements(slice) };
    }
}
