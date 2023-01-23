use core::iter::{FusedIterator, TrustedLen};
use core::{alloc, fmt};

use crate::alloc::{Allocator, Global};

use super::VecDeque;

/// An owning iterator over the elements of a `VecDeque`.
///
/// This `struct` is created by the [`into_iter`] method on [`VecDeque`]
/// (provided by the [`IntoIterator`] trait). See its documentation for more.
///
/// [`into_iter`]: VecDeque::into_iter
/// [`IntoIterator`]: core::iter::IntoIterator
#[derive(Clone)]
#[stable(feature = "rust1", since = "1.0.0")]
pub struct IntoIter<
    T,
    #[unstable(feature = "allocator_api", issue = "32838")] A: Allocator = Global,
    const COOP_PREFERRED: bool = true,
> where
    [(); alloc::co_alloc_metadata_num_slots_with_preference::<A>(COOP_PREFERRED)]:,
{
    inner: VecDeque<T, A, COOP_PREFERRED>,
}

impl<T, A: Allocator, const COOP_PREFERRED: bool> IntoIter<T, A, COOP_PREFERRED>
where
    [(); core::alloc::co_alloc_metadata_num_slots_with_preference::<A>(COOP_PREFERRED)]:,
{
    pub(super) fn new(inner: VecDeque<T, A, COOP_PREFERRED>) -> Self {
        IntoIter { inner }
    }

    pub(super) fn into_vecdeque(self) -> VecDeque<T, A, COOP_PREFERRED> {
        self.inner
    }
}

#[stable(feature = "collection_debug", since = "1.17.0")]
impl<T: fmt::Debug, A: Allocator, const COOP_PREFERRED: bool> fmt::Debug for IntoIter<T, A, COOP_PREFERRED>
where
    [(); core::alloc::co_alloc_metadata_num_slots_with_preference::<A>(COOP_PREFERRED)]:,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("IntoIter").field(&self.inner).finish()
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T, A: Allocator, const COOP_PREFERRED: bool> Iterator for IntoIter<T, A, COOP_PREFERRED>
where
    [(); core::alloc::co_alloc_metadata_num_slots_with_preference::<A>(COOP_PREFERRED)]:,
{
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        self.inner.pop_front()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.inner.len();
        (len, Some(len))
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T, A: Allocator, const COOP_PREFERRED: bool> DoubleEndedIterator for IntoIter<T, A, COOP_PREFERRED>
where
    [(); core::alloc::co_alloc_metadata_num_slots_with_preference::<A>(COOP_PREFERRED)]:,
{
    #[inline]
    fn next_back(&mut self) -> Option<T> {
        self.inner.pop_back()
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T, A: Allocator, const COOP_PREFERRED: bool> ExactSizeIterator for IntoIter<T, A, COOP_PREFERRED>
where
    [(); core::alloc::co_alloc_metadata_num_slots_with_preference::<A>(COOP_PREFERRED)]:,
{
    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

#[stable(feature = "fused", since = "1.26.0")]
impl<T, A: Allocator, const COOP_PREFERRED: bool> FusedIterator for IntoIter<T, A, COOP_PREFERRED>
where
    [(); core::alloc::co_alloc_metadata_num_slots_with_preference::<A>(COOP_PREFERRED)]:,
{
}

#[unstable(feature = "trusted_len", issue = "37572")]
unsafe impl<T, A: Allocator, const COOP_PREFERRED : bool> TrustedLen for IntoIter<T, A, COOP_PREFERRED>
where
    [(); core::alloc::co_alloc_metadata_num_slots_with_preference::<A>(COOP_PREFERRED)]:,
{
}
