//! # Min SmallVec
//! A collection that knows its own minimum value.

use std::ptr::NonNull;

use smallvec::SmallVec;

/// A collection with a known minimum value backed by a [SmallVec].
///
/// Comparisons and equality on the type are delegated to comparisons and equality
/// on the minimum value.
///
/// This allows one to create a tree of [MinSmallVec]s like so:
/// ```rust
/// struct MinTree<
///     T: PartialOrd,
///     const OS: usize, // outer size
///     const IS: usize, // inner size
/// >(MinSmallVec<MinSmallVec<T, IS>, OS>);
/// ```
///
/// which reduces the cost of computing the minimum value logarithmically.
#[derive(Debug)]
pub struct MinSmallVec<T: PartialOrd, const S: usize> {
    inner: SmallVec<[T; S]>,
    /// Min value of the contained array is [None] if the array is empty
    /// or [PartialOrd::partial_cmp] has returned [None]
    min: Option<NonNull<T>>,
}

fn slice_min<T: PartialOrd>(slice: &[T]) -> Option<NonNull<T>> {
    let first = slice.first()?;

    slice[1..]
        .iter()
        .try_fold(first, |min, val| {
            min.partial_cmp(val).map(|ord| match ord {
                std::cmp::Ordering::Greater => val,
                _ => min,
            })
        })
        .map(|refer: &T| refer.into())
}

impl<T: Ord, const S: usize> MinSmallVec<T, S> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: SmallVec::with_capacity(capacity),
            min: None,
        }
    }

    pub fn new() -> Self {
        Default::default()
    }

    pub fn from_slice(slice: &[T]) -> Self
    where
        T: Copy,
    {
        Self {
            min: slice_min(slice),
            inner: SmallVec::from_slice(slice),
        }
    }

    /// Get a reference to the minimum value.
    pub fn get_min(&self) -> Option<&T> {
        // SAFETY: min always points to a value in self
        self.min.map(|ptr| unsafe { ptr.as_ref() })
    }

    /// Applies a modification function to `self` and recalculates the min value after
    /// using a linear scan
    pub fn modify(&mut self, mut func: impl FnMut(&mut SmallVec<[T; S]>)) {
        func(&mut self.inner);
        self.min = slice_min(&self.inner);
    }

    /// Pushes a value. This is faster than using [MinBucket::modify]
    pub fn push(&mut self, value: T) {
        self.inner.push(value);
        let pushed = unsafe { self.inner.last().unwrap_unchecked() };

        // recalculate the minimum value with a simple comparison
        if !self.get_min().is_some_and(|min| min <= pushed) {
            self.min = Some(pushed.into());
        }
    }
}

impl<T: Ord, const S: usize> Default for MinSmallVec<T, S> {
    fn default() -> Self {
        Self {
            inner: SmallVec::default(),
            min: None,
        }
    }
}

impl<T: Ord, const S: usize> PartialOrd for MinSmallVec<T, S> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.get_min().zip(other.get_min()).map(|(s, o)| s.cmp(o))
    }
}

impl<T: Ord + Eq, const S: usize> PartialEq for MinSmallVec<T, S> {
    fn eq(&self, other: &Self) -> bool {
        self.get_min().eq(&other.get_min())
    }
}

impl<T: Ord + Eq, const S: usize> Eq for MinSmallVec<T, S> {}

impl<T: Ord + Eq, const S: usize> FromIterator<T> for MinSmallVec<T, S> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let inner = SmallVec::from_iter(iter);

        Self {
            min: slice_min(&inner),
            inner,
        }
    }
}
