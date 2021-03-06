//! A dynamically-sized view into a contiguous sequence, `[T]`.
//!
//! *[See also the slice primitive type](../../std/primitive.slice.html).*
//!
//! Slices are a view into a block of memory represented as a pointer and a
//! length.
//!
//! ```
//! // slicing a Vec
//! let vec = vec![1, 2, 3];
//! let int_slice = &vec[..];
//! // coercing an array to a slice
//! let str_slice: &[&str] = &["one", "two", "three"];
//! ```
//!
//! Slices are either mutable or shared. The shared slice type is `&[T]`,
//! while the mutable slice type is `&mut [T]`, where `T` represents the element
//! type. For example, you can mutate the block of memory that a mutable slice
//! points to:
//!
//! ```
//! let x = &mut [1, 2, 3];
//! x[1] = 7;
//! assert_eq!(x, &[1, 7, 3]);
//! ```
//!
//! Here are some of the things this module contains:
//!
//! ## Structs
//!
//! There are several structs that are useful for slices, such as [`Iter`], which
//! represents iteration over a slice.
//!
//! ## Trait Implementations
//!
//! There are several implementations of common traits for slices. Some examples
//! include:
//!
//! * [`Clone`]
//! * [`Eq`], [`Ord`] - for slices whose element type are [`Eq`] or [`Ord`].
//! * [`Hash`] - for slices whose element type is [`Hash`].
//!
//! ## Iteration
//!
//! The slices implement `IntoIterator`. The iterator yields references to the
//! slice elements.
//!
//! ```
//! let numbers = &[0, 1, 2];
//! for n in numbers {
//!     println!("{} is a number!", n);
//! }
//! ```
//!
//! The mutable slice yields mutable references to the elements:
//!
//! ```
//! let mut scores = [7, 8, 9];
//! for score in &mut scores[..] {
//!     *score += 1;
//! }
//! ```
//!
//! This iterator yields mutable references to the slice's elements, so while
//! the element type of the slice is `i32`, the element type of the iterator is
//! `&mut i32`.
//!
//! * [`.iter`] and [`.iter_mut`] are the explicit methods to return the default
//!   iterators.
//! * Further methods that return iterators are [`.split`], [`.splitn`],
//!   [`.chunks`], [`.windows`] and more.
//!
//! [`Clone`]: ../../std/clone/trait.Clone.html
//! [`Eq`]: ../../std/cmp/trait.Eq.html
//! [`Ord`]: ../../std/cmp/trait.Ord.html
//! [`Iter`]: struct.Iter.html
//! [`Hash`]: ../../std/hash/trait.Hash.html
//! [`.iter`]: ../../std/primitive.slice.html#method.iter
//! [`.iter_mut`]: ../../std/primitive.slice.html#method.iter_mut
//! [`.split`]: ../../std/primitive.slice.html#method.split
//! [`.splitn`]: ../../std/primitive.slice.html#method.splitn
//! [`.chunks`]: ../../std/primitive.slice.html#method.chunks
//! [`.windows`]: ../../std/primitive.slice.html#method.windows
#![stable(feature = "rust1", since = "1.0.0")]

// Many of the usings in this module are only used in the test configuration.
// It's cleaner to just turn off the unused_imports warning than to fix them.
#![cfg_attr(test, allow(unused_imports, dead_code))]

use core::cmp::Ordering::{self, Less};
use core::mem::size_of;
use core::mem;
use core::ptr;
use core::{u8, u16, u32};

use borrow::{Borrow, BorrowMut, ToOwned};
use boxed::Box;
use vec::Vec;

#[stable(feature = "rust1", since = "1.0.0")]
pub use core::slice::{Chunks, Windows};
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::slice::{Iter, IterMut};
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::slice::{SplitMut, ChunksMut, Split};
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::slice::{SplitN, RSplitN, SplitNMut, RSplitNMut};
#[stable(feature = "slice_rsplit", since = "1.27.0")]
pub use core::slice::{RSplit, RSplitMut};
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::slice::{from_raw_parts, from_raw_parts_mut};
#[stable(feature = "from_ref", since = "1.28.0")]
pub use core::slice::{from_ref, from_mut};
#[stable(feature = "slice_get_slice", since = "1.28.0")]
pub use core::slice::SliceIndex;
#[stable(feature = "chunks_exact", since = "1.31.0")]
pub use core::slice::{ChunksExact, ChunksExactMut};
#[stable(feature = "rchunks", since = "1.31.0")]
pub use core::slice::{RChunks, RChunksMut, RChunksExact, RChunksExactMut};

////////////////////////////////////////////////////////////////////////////////
// Basic slice extension methods
////////////////////////////////////////////////////////////////////////////////

// HACK(japaric) needed for the implementation of `vec!` macro during testing
// NB see the hack module in this file for more details
#[cfg(test)]
pub use self::hack::into_vec;

// HACK(japaric) needed for the implementation of `Vec::clone` during testing
// NB see the hack module in this file for more details
#[cfg(test)]
pub use self::hack::to_vec;

// HACK(japaric): With cfg(test) `impl [T]` is not available, these three
// functions are actually methods that are in `impl [T]` but not in
// `core::slice::SliceExt` - we need to supply these functions for the
// `test_permutations` test
mod hack {
    use boxed::Box;
    use core::mem;

    #[cfg(test)]
    use string::ToString;
    use vec::Vec;

    pub fn into_vec<T>(mut b: Box<[T]>) -> Vec<T> {
        unsafe {
            let xs = Vec::from_raw_parts(b.as_mut_ptr(), b.len(), b.len());
            mem::forget(b);
            xs
        }
    }

    #[inline]
    pub fn to_vec<T>(s: &[T]) -> Vec<T>
        where T: Clone
    {
        let mut vector = Vec::with_capacity(s.len());
        vector.extend_from_slice(s);
        vector
    }
}

#[lang = "slice_alloc"]
#[cfg(not(test))]
impl<T> [T] {
    /// Sorts the slice.
    ///
    /// This sort is stable (i.e., does not reorder equal elements) and `O(n log n)` worst-case.
    ///
    /// When applicable, unstable sorting is preferred because it is generally faster than stable
    /// sorting and it doesn't allocate auxiliary memory.
    /// See [`sort_unstable`](#method.sort_unstable).
    ///
    /// # Current implementation
    ///
    /// The current algorithm is an adaptive, iterative merge sort inspired by
    /// [timsort](https://en.wikipedia.org/wiki/Timsort).
    /// It is designed to be very fast in cases where the slice is nearly sorted, or consists of
    /// two or more sorted sequences concatenated one after another.
    ///
    /// Also, it allocates temporary storage half the size of `self`, but for short slices a
    /// non-allocating insertion sort is used instead.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut v = [-5, 4, 1, -3, 2];
    ///
    /// v.sort();
    /// assert!(v == [-5, -3, 1, 2, 4]);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn sort(&mut self)
        where T: Ord
    {
        merge_sort(self, |a, b| a.lt(b));
    }

    /// Sorts the slice with a comparator function.
    ///
    /// This sort is stable (i.e., does not reorder equal elements) and `O(n log n)` worst-case.
    ///
    /// The comparator function must define a total ordering for the elements in the slice. If
    /// the ordering is not total, the order of the elements is unspecified. An order is a
    /// total order if it is (for all a, b and c):
    ///
    /// * total and antisymmetric: exactly one of a < b, a == b or a > b is true; and
    /// * transitive, a < b and b < c implies a < c. The same must hold for both == and >.
    ///
    /// For example, while [`f64`] doesn't implement [`Ord`] because `NaN != NaN`, we can use
    /// `partial_cmp` as our sort function when we know the slice doesn't contain a `NaN`.
    ///
    /// ```
    /// let mut floats = [5f64, 4.0, 1.0, 3.0, 2.0];
    /// floats.sort_by(|a, b| a.partial_cmp(b).unwrap());
    /// assert_eq!(floats, [1.0, 2.0, 3.0, 4.0, 5.0]);
    /// ```
    ///
    /// When applicable, unstable sorting is preferred because it is generally faster than stable
    /// sorting and it doesn't allocate auxiliary memory.
    /// See [`sort_unstable_by`](#method.sort_unstable_by).
    ///
    /// # Current implementation
    ///
    /// The current algorithm is an adaptive, iterative merge sort inspired by
    /// [timsort](https://en.wikipedia.org/wiki/Timsort).
    /// It is designed to be very fast in cases where the slice is nearly sorted, or consists of
    /// two or more sorted sequences concatenated one after another.
    ///
    /// Also, it allocates temporary storage half the size of `self`, but for short slices a
    /// non-allocating insertion sort is used instead.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut v = [5, 4, 1, 3, 2];
    /// v.sort_by(|a, b| a.cmp(b));
    /// assert!(v == [1, 2, 3, 4, 5]);
    ///
    /// // reverse sorting
    /// v.sort_by(|a, b| b.cmp(a));
    /// assert!(v == [5, 4, 3, 2, 1]);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn sort_by<F>(&mut self, mut compare: F)
        where F: FnMut(&T, &T) -> Ordering
    {
        merge_sort(self, |a, b| compare(a, b) == Less);
    }

    /// Sorts the slice with a key extraction function.
    ///
    /// This sort is stable (i.e., does not reorder equal elements) and `O(m n log(m n))`
    /// worst-case, where the key function is `O(m)`.
    ///
    /// When applicable, unstable sorting is preferred because it is generally faster than stable
    /// sorting and it doesn't allocate auxiliary memory.
    /// See [`sort_unstable_by_key`](#method.sort_unstable_by_key).
    ///
    /// # Current implementation
    ///
    /// The current algorithm is an adaptive, iterative merge sort inspired by
    /// [timsort](https://en.wikipedia.org/wiki/Timsort).
    /// It is designed to be very fast in cases where the slice is nearly sorted, or consists of
    /// two or more sorted sequences concatenated one after another.
    ///
    /// Also, it allocates temporary storage half the size of `self`, but for short slices a
    /// non-allocating insertion sort is used instead.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut v = [-5i32, 4, 1, -3, 2];
    ///
    /// v.sort_by_key(|k| k.abs());
    /// assert!(v == [1, 2, -3, 4, -5]);
    /// ```
    #[stable(feature = "slice_sort_by_key", since = "1.7.0")]
    #[inline]
    pub fn sort_by_key<K, F>(&mut self, mut f: F)
        where F: FnMut(&T) -> K, K: Ord
    {
        merge_sort(self, |a, b| f(a).lt(&f(b)));
    }

    /// Sorts the slice with a key extraction function.
    ///
    /// During sorting, the key function is called only once per element.
    ///
    /// This sort is stable (i.e., does not reorder equal elements) and `O(m n + n log n)`
    /// worst-case, where the key function is `O(m)`.
    ///
    /// For simple key functions (e.g., functions that are property accesses or
    /// basic operations), [`sort_by_key`](#method.sort_by_key) is likely to be
    /// faster.
    ///
    /// # Current implementation
    ///
    /// The current algorithm is based on [pattern-defeating quicksort][pdqsort] by Orson Peters,
    /// which combines the fast average case of randomized quicksort with the fast worst case of
    /// heapsort, while achieving linear time on slices with certain patterns. It uses some
    /// randomization to avoid degenerate cases, but with a fixed seed to always provide
    /// deterministic behavior.
    ///
    /// In the worst case, the algorithm allocates temporary storage in a `Vec<(K, usize)>` the
    /// length of the slice.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(slice_sort_by_cached_key)]
    /// let mut v = [-5i32, 4, 32, -3, 2];
    ///
    /// v.sort_by_cached_key(|k| k.to_string());
    /// assert!(v == [-3, -5, 2, 32, 4]);
    /// ```
    ///
    /// [pdqsort]: https://github.com/orlp/pdqsort
    #[unstable(feature = "slice_sort_by_cached_key", issue = "34447")]
    #[inline]
    pub fn sort_by_cached_key<K, F>(&mut self, f: F)
        where F: FnMut(&T) -> K, K: Ord
    {
        // Helper macro for indexing our vector by the smallest possible type, to reduce allocation.
        macro_rules! sort_by_key {
            ($t:ty, $slice:ident, $f:ident) => ({
                let mut indices: Vec<_> =
                    $slice.iter().map($f).enumerate().map(|(i, k)| (k, i as $t)).collect();
                // The elements of `indices` are unique, as they are indexed, so any sort will be
                // stable with respect to the original slice. We use `sort_unstable` here because
                // it requires less memory allocation.
                indices.sort_unstable();
                for i in 0..$slice.len() {
                    let mut index = indices[i].1;
                    while (index as usize) < i {
                        index = indices[index as usize].1;
                    }
                    indices[i].1 = index;
                    $slice.swap(i, index as usize);
                }
            })
        }

        let sz_u8    = mem::size_of::<(K, u8)>();
        let sz_u16   = mem::size_of::<(K, u16)>();
        let sz_u32   = mem::size_of::<(K, u32)>();
        let sz_usize = mem::size_of::<(K, usize)>();

        let len = self.len();
        if len < 2 { return }
        if sz_u8  < sz_u16   && len <= ( u8::MAX as usize) { return sort_by_key!( u8, self, f) }
        if sz_u16 < sz_u32   && len <= (u16::MAX as usize) { return sort_by_key!(u16, self, f) }
        if sz_u32 < sz_usize && len <= (u32::MAX as usize) { return sort_by_key!(u32, self, f) }
        sort_by_key!(usize, self, f)
    }

    /// Copies `self` into a new `Vec`.
    ///
    /// # Examples
    ///
    /// ```
    /// let s = [10, 40, 30];
    /// let x = s.to_vec();
    /// // Here, `s` and `x` can be modified independently.
    /// ```
    #[rustc_conversion_suggestion]
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn to_vec(&self) -> Vec<T>
        where T: Clone
    {
        // NB see hack module in this file
        hack::to_vec(self)
    }

    /// Converts `self` into a vector without clones or allocation.
    ///
    /// The resulting vector can be converted back into a box via
    /// `Vec<T>`'s `into_boxed_slice` method.
    ///
    /// # Examples
    ///
    /// ```
    /// let s: Box<[i32]> = Box::new([10, 40, 30]);
    /// let x = s.into_vec();
    /// // `s` cannot be used anymore because it has been converted into `x`.
    ///
    /// assert_eq!(x, vec![10, 40, 30]);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn into_vec(self: Box<Self>) -> Vec<T> {
        // NB see hack module in this file
        hack::into_vec(self)
    }

    /// Creates a vector by repeating a slice `n` times.
    ///
    /// # Panics
    ///
    /// This function will panic if the capacity would overflow.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// #![feature(repeat_generic_slice)]
    ///
    /// fn main() {
    ///     assert_eq!([1, 2].repeat(3), vec![1, 2, 1, 2, 1, 2]);
    /// }
    /// ```
    ///
    /// A panic upon overflow:
    ///
    /// ```should_panic
    /// #![feature(repeat_generic_slice)]
    /// fn main() {
    ///     // this will panic at runtime
    ///     b"0123456789abcdef".repeat(usize::max_value());
    /// }
    /// ```
    #[unstable(feature = "repeat_generic_slice",
               reason = "it's on str, why not on slice?",
               issue = "48784")]
    pub fn repeat(&self, n: usize) -> Vec<T> where T: Copy {
        if n == 0 {
            return Vec::new();
        }

        // If `n` is larger than zero, it can be split as
        // `n = 2^expn + rem (2^expn > rem, expn >= 0, rem >= 0)`.
        // `2^expn` is the number represented by the leftmost '1' bit of `n`,
        // and `rem` is the remaining part of `n`.

        // Using `Vec` to access `set_len()`.
        let mut buf = Vec::with_capacity(self.len().checked_mul(n).expect("capacity overflow"));

        // `2^expn` repetition is done by doubling `buf` `expn`-times.
        buf.extend(self);
        {
            let mut m = n >> 1;
            // If `m > 0`, there are remaining bits up to the leftmost '1'.
            while m > 0 {
                // `buf.extend(buf)`:
                unsafe {
                    ptr::copy_nonoverlapping(
                        buf.as_ptr(),
                        (buf.as_mut_ptr() as *mut T).add(buf.len()),
                        buf.len(),
                    );
                    // `buf` has capacity of `self.len() * n`.
                    let buf_len = buf.len();
                    buf.set_len(buf_len * 2);
                }

                m >>= 1;
            }
        }

        // `rem` (`= n - 2^expn`) repetition is done by copying
        // first `rem` repetitions from `buf` itself.
        let rem_len = self.len() * n - buf.len(); // `self.len() * rem`
        if rem_len > 0 {
            // `buf.extend(buf[0 .. rem_len])`:
            unsafe {
                // This is non-overlapping since `2^expn > rem`.
                ptr::copy_nonoverlapping(
                    buf.as_ptr(),
                    (buf.as_mut_ptr() as *mut T).add(buf.len()),
                    rem_len,
                );
                // `buf.len() + rem_len` equals to `buf.capacity()` (`= self.len() * n`).
                let buf_cap = buf.capacity();
                buf.set_len(buf_cap);
            }
        }
        buf
    }
}

#[lang = "slice_u8_alloc"]
#[cfg(not(test))]
impl [u8] {
    /// Returns a vector containing a copy of this slice where each byte
    /// is mapped to its ASCII upper case equivalent.
    ///
    /// ASCII letters 'a' to 'z' are mapped to 'A' to 'Z',
    /// but non-ASCII letters are unchanged.
    ///
    /// To uppercase the value in-place, use [`make_ascii_uppercase`].
    ///
    /// [`make_ascii_uppercase`]: #method.make_ascii_uppercase
    #[stable(feature = "ascii_methods_on_intrinsics", since = "1.23.0")]
    #[inline]
    pub fn to_ascii_uppercase(&self) -> Vec<u8> {
        let mut me = self.to_vec();
        me.make_ascii_uppercase();
        me
    }

    /// Returns a vector containing a copy of this slice where each byte
    /// is mapped to its ASCII lower case equivalent.
    ///
    /// ASCII letters 'A' to 'Z' are mapped to 'a' to 'z',
    /// but non-ASCII letters are unchanged.
    ///
    /// To lowercase the value in-place, use [`make_ascii_lowercase`].
    ///
    /// [`make_ascii_lowercase`]: #method.make_ascii_lowercase
    #[stable(feature = "ascii_methods_on_intrinsics", since = "1.23.0")]
    #[inline]
    pub fn to_ascii_lowercase(&self) -> Vec<u8> {
        let mut me = self.to_vec();
        me.make_ascii_lowercase();
        me
    }
}

////////////////////////////////////////////////////////////////////////////////
// Extension traits for slices over specific kinds of data
////////////////////////////////////////////////////////////////////////////////
#[unstable(feature = "slice_concat_ext",
           reason = "trait should not have to exist",
           issue = "27747")]
/// An extension trait for concatenating slices
///
/// While this trait is unstable, the methods are stable. `SliceConcatExt` is
/// included in the [standard library prelude], so you can use [`join()`] and
/// [`concat()`] as if they existed on `[T]` itself.
///
/// [standard library prelude]: ../../std/prelude/index.html
/// [`join()`]: #tymethod.join
/// [`concat()`]: #tymethod.concat
pub trait SliceConcatExt<T: ?Sized> {
    #[unstable(feature = "slice_concat_ext",
               reason = "trait should not have to exist",
               issue = "27747")]
    /// The resulting type after concatenation
    type Output;

    /// Flattens a slice of `T` into a single value `Self::Output`.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(["hello", "world"].concat(), "helloworld");
    /// assert_eq!([[1, 2], [3, 4]].concat(), [1, 2, 3, 4]);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    fn concat(&self) -> Self::Output;

    /// Flattens a slice of `T` into a single value `Self::Output`, placing a
    /// given separator between each.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(["hello", "world"].join(" "), "hello world");
    /// assert_eq!([[1, 2], [3, 4]].join(&0), [1, 2, 0, 3, 4]);
    /// ```
    #[stable(feature = "rename_connect_to_join", since = "1.3.0")]
    fn join(&self, sep: &T) -> Self::Output;

    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_deprecated(since = "1.3.0", reason = "renamed to join")]
    fn connect(&self, sep: &T) -> Self::Output;
}

#[unstable(feature = "slice_concat_ext",
           reason = "trait should not have to exist",
           issue = "27747")]
impl<T: Clone, V: Borrow<[T]>> SliceConcatExt<T> for [V] {
    type Output = Vec<T>;

    fn concat(&self) -> Vec<T> {
        let size = self.iter().map(|slice| slice.borrow().len()).sum();
        let mut result = Vec::with_capacity(size);
        for v in self {
            result.extend_from_slice(v.borrow())
        }
        result
    }

    fn join(&self, sep: &T) -> Vec<T> {
        let mut iter = self.iter();
        let first = match iter.next() {
            Some(first) => first,
            None => return vec![],
        };
        let size = self.iter().map(|slice| slice.borrow().len()).sum::<usize>() + self.len() - 1;
        let mut result = Vec::with_capacity(size);
        result.extend_from_slice(first.borrow());

        for v in iter {
            result.push(sep.clone());
            result.extend_from_slice(v.borrow())
        }
        result
    }

    fn connect(&self, sep: &T) -> Vec<T> {
        self.join(sep)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Standard trait implementations for slices
////////////////////////////////////////////////////////////////////////////////

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> Borrow<[T]> for Vec<T> {
    fn borrow(&self) -> &[T] {
        &self[..]
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> BorrowMut<[T]> for Vec<T> {
    fn borrow_mut(&mut self) -> &mut [T] {
        &mut self[..]
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: Clone> ToOwned for [T] {
    type Owned = Vec<T>;
    #[cfg(not(test))]
    fn to_owned(&self) -> Vec<T> {
        self.to_vec()
    }

    #[cfg(test)]
    fn to_owned(&self) -> Vec<T> {
        hack::to_vec(self)
    }

    fn clone_into(&self, target: &mut Vec<T>) {
        // drop anything in target that will not be overwritten
        target.truncate(self.len());
        let len = target.len();

        // reuse the contained values' allocations/resources.
        target.clone_from_slice(&self[..len]);

        // target.len <= self.len due to the truncate above, so the
        // slice here is always in-bounds.
        target.extend_from_slice(&self[len..]);
    }
}

////////////////////////////////////////////////////////////////////////////////
// Sorting
////////////////////////////////////////////////////////////////////////////////

/// Inserts `v[0]` into pre-sorted sequence `v[1..]` so that whole `v[..]` becomes sorted.
///
/// This is the integral subroutine of insertion sort.
fn insert_head<T, F>(v: &mut [T], is_less: &mut F)
    where F: FnMut(&T, &T) -> bool
{
    if v.len() >= 2 && is_less(&v[1], &v[0]) {
        unsafe {
            // There are three ways to implement insertion here:
            //
            // 1. Swap adjacent elements until the first one gets to its final destination.
            //    However, this way we copy data around more than is necessary. If elements are big
            //    structures (costly to copy), this method will be slow.
            //
            // 2. Iterate until the right place for the first element is found. Then shift the
            //    elements succeeding it to make room for it and finally place it into the
            //    remaining hole. This is a good method.
            //
            // 3. Copy the first element into a temporary variable. Iterate until the right place
            //    for it is found. As we go along, copy every traversed element into the slot
            //    preceding it. Finally, copy data from the temporary variable into the remaining
            //    hole. This method is very good. Benchmarks demonstrated slightly better
            //    performance than with the 2nd method.
            //
            // All methods were benchmarked, and the 3rd showed best results. So we chose that one.
            let mut tmp = mem::ManuallyDrop::new(ptr::read(&v[0]));

            // Intermediate state of the insertion process is always tracked by `hole`, which
            // serves two purposes:
            // 1. Protects integrity of `v` from panics in `is_less`.
            // 2. Fills the remaining hole in `v` in the end.
            //
            // Panic safety:
            //
            // If `is_less` panics at any point during the process, `hole` will get dropped and
            // fill the hole in `v` with `tmp`, thus ensuring that `v` still holds every object it
            // initially held exactly once.
            let mut hole = InsertionHole {
                src: &mut *tmp,
                dest: &mut v[1],
            };
            ptr::copy_nonoverlapping(&v[1], &mut v[0], 1);

            for i in 2..v.len() {
                if !is_less(&v[i], &*tmp) {
                    break;
                }
                ptr::copy_nonoverlapping(&v[i], &mut v[i - 1], 1);
                hole.dest = &mut v[i];
            }
            // `hole` gets dropped and thus copies `tmp` into the remaining hole in `v`.
        }
    }

    // When dropped, copies from `src` into `dest`.
    struct InsertionHole<T> {
        src: *mut T,
        dest: *mut T,
    }

    impl<T> Drop for InsertionHole<T> {
        fn drop(&mut self) {
            unsafe { ptr::copy_nonoverlapping(self.src, self.dest, 1); }
        }
    }
}

/// Merges non-decreasing runs `v[..mid]` and `v[mid..]` using `buf` as temporary storage, and
/// stores the result into `v[..]`.
///
/// # Safety
///
/// The two slices must be non-empty and `mid` must be in bounds. Buffer `buf` must be long enough
/// to hold a copy of the shorter slice. Also, `T` must not be a zero-sized type.
unsafe fn merge<T, F>(v: &mut [T], mid: usize, buf: *mut T, is_less: &mut F)
    where F: FnMut(&T, &T) -> bool
{
    let len = v.len();
    let v = v.as_mut_ptr();
    let v_mid = v.add(mid);
    let v_end = v.add(len);

    // The merge process first copies the shorter run into `buf`. Then it traces the newly copied
    // run and the longer run forwards (or backwards), comparing their next unconsumed elements and
    // copying the lesser (or greater) one into `v`.
    //
    // As soon as the shorter run is fully consumed, the process is done. If the longer run gets
    // consumed first, then we must copy whatever is left of the shorter run into the remaining
    // hole in `v`.
    //
    // Intermediate state of the process is always tracked by `hole`, which serves two purposes:
    // 1. Protects integrity of `v` from panics in `is_less`.
    // 2. Fills the remaining hole in `v` if the longer run gets consumed first.
    //
    // Panic safety:
    //
    // If `is_less` panics at any point during the process, `hole` will get dropped and fill the
    // hole in `v` with the unconsumed range in `buf`, thus ensuring that `v` still holds every
    // object it initially held exactly once.
    let mut hole;

    if mid <= len - mid {
        // The left run is shorter.
        ptr::copy_nonoverlapping(v, buf, mid);
        hole = MergeHole {
            start: buf,
            end: buf.add(mid),
            dest: v,
        };

        // Initially, these pointers point to the beginnings of their arrays.
        let left = &mut hole.start;
        let mut right = v_mid;
        let out = &mut hole.dest;

        while *left < hole.end && right < v_end {
            // Consume the lesser side.
            // If equal, prefer the left run to maintain stability.
            let to_copy = if is_less(&*right, &**left) {
                get_and_increment(&mut right)
            } else {
                get_and_increment(left)
            };
            ptr::copy_nonoverlapping(to_copy, get_and_increment(out), 1);
        }
    } else {
        // The right run is shorter.
        ptr::copy_nonoverlapping(v_mid, buf, len - mid);
        hole = MergeHole {
            start: buf,
            end: buf.add(len - mid),
            dest: v_mid,
        };

        // Initially, these pointers point past the ends of their arrays.
        let left = &mut hole.dest;
        let right = &mut hole.end;
        let mut out = v_end;

        while v < *left && buf < *right {
            // Consume the greater side.
            // If equal, prefer the right run to maintain stability.
            let to_copy = if is_less(&*right.offset(-1), &*left.offset(-1)) {
                decrement_and_get(left)
            } else {
                decrement_and_get(right)
            };
            ptr::copy_nonoverlapping(to_copy, decrement_and_get(&mut out), 1);
        }
    }
    // Finally, `hole` gets dropped. If the shorter run was not fully consumed, whatever remains of
    // it will now be copied into the hole in `v`.

    unsafe fn get_and_increment<T>(ptr: &mut *mut T) -> *mut T {
        let old = *ptr;
        *ptr = ptr.offset(1);
        old
    }

    unsafe fn decrement_and_get<T>(ptr: &mut *mut T) -> *mut T {
        *ptr = ptr.offset(-1);
        *ptr
    }

    // When dropped, copies the range `start..end` into `dest..`.
    struct MergeHole<T> {
        start: *mut T,
        end: *mut T,
        dest: *mut T,
    }

    impl<T> Drop for MergeHole<T> {
        fn drop(&mut self) {
            // `T` is not a zero-sized type, so it's okay to divide by its size.
            let len = (self.end as usize - self.start as usize) / mem::size_of::<T>();
            unsafe { ptr::copy_nonoverlapping(self.start, self.dest, len); }
        }
    }
}

/// This merge sort borrows some (but not all) ideas from TimSort, which is described in detail
/// [here](http://svn.python.org/projects/python/trunk/Objects/listsort.txt).
///
/// The algorithm identifies strictly descending and non-descending subsequences, which are called
/// natural runs. There is a stack of pending runs yet to be merged. Each newly found run is pushed
/// onto the stack, and then some pairs of adjacent runs are merged until these two invariants are
/// satisfied:
///
/// 1. for every `i` in `1..runs.len()`: `runs[i - 1].len > runs[i].len`
/// 2. for every `i` in `2..runs.len()`: `runs[i - 2].len > runs[i - 1].len + runs[i].len`
///
/// The invariants ensure that the total running time is `O(n log n)` worst-case.
fn merge_sort<T, F>(v: &mut [T], mut is_less: F)
    where F: FnMut(&T, &T) -> bool
{
    // Slices of up to this length get sorted using insertion sort.
    const MAX_INSERTION: usize = 20;
    // Very short runs are extended using insertion sort to span at least this many elements.
    const MIN_RUN: usize = 10;

    // Sorting has no meaningful behavior on zero-sized types.
    if size_of::<T>() == 0 {
        return;
    }

    let len = v.len();

    // Short arrays get sorted in-place via insertion sort to avoid allocations.
    if len <= MAX_INSERTION {
        if len >= 2 {
            for i in (0..len-1).rev() {
                insert_head(&mut v[i..], &mut is_less);
            }
        }
        return;
    }

    // Allocate a buffer to use as scratch memory. We keep the length 0 so we can keep in it
    // shallow copies of the contents of `v` without risking the dtors running on copies if
    // `is_less` panics. When merging two sorted runs, this buffer holds a copy of the shorter run,
    // which will always have length at most `len / 2`.
    let mut buf = Vec::with_capacity(len / 2);

    // In order to identify natural runs in `v`, we traverse it backwards. That might seem like a
    // strange decision, but consider the fact that merges more often go in the opposite direction
    // (forwards). According to benchmarks, merging forwards is slightly faster than merging
    // backwards. To conclude, identifying runs by traversing backwards improves performance.
    let mut runs = vec![];
    let mut end = len;
    while end > 0 {
        // Find the next natural run, and reverse it if it's strictly descending.
        let mut start = end - 1;
        if start > 0 {
            start -= 1;
            unsafe {
                if is_less(v.get_unchecked(start + 1), v.get_unchecked(start)) {
                    while start > 0 && is_less(v.get_unchecked(start),
                                               v.get_unchecked(start - 1)) {
                        start -= 1;
                    }
                    v[start..end].reverse();
                } else {
                    while start > 0 && !is_less(v.get_unchecked(start),
                                                v.get_unchecked(start - 1)) {
                        start -= 1;
                    }
                }
            }
        }

        // Insert some more elements into the run if it's too short. Insertion sort is faster than
        // merge sort on short sequences, so this significantly improves performance.
        while start > 0 && end - start < MIN_RUN {
            start -= 1;
            insert_head(&mut v[start..end], &mut is_less);
        }

        // Push this run onto the stack.
        runs.push(Run {
            start,
            len: end - start,
        });
        end = start;

        // Merge some pairs of adjacent runs to satisfy the invariants.
        while let Some(r) = collapse(&runs) {
            let left = runs[r + 1];
            let right = runs[r];
            unsafe {
                merge(&mut v[left.start .. right.start + right.len], left.len, buf.as_mut_ptr(),
                      &mut is_less);
            }
            runs[r] = Run {
                start: left.start,
                len: left.len + right.len,
            };
            runs.remove(r + 1);
        }
    }

    // Finally, exactly one run must remain in the stack.
    debug_assert!(runs.len() == 1 && runs[0].start == 0 && runs[0].len == len);

    // Examines the stack of runs and identifies the next pair of runs to merge. More specifically,
    // if `Some(r)` is returned, that means `runs[r]` and `runs[r + 1]` must be merged next. If the
    // algorithm should continue building a new run instead, `None` is returned.
    //
    // TimSort is infamous for its buggy implementations, as described here:
    // http://envisage-project.eu/timsort-specification-and-verification/
    //
    // The gist of the story is: we must enforce the invariants on the top four runs on the stack.
    // Enforcing them on just top three is not sufficient to ensure that the invariants will still
    // hold for *all* runs in the stack.
    //
    // This function correctly checks invariants for the top four runs. Additionally, if the top
    // run starts at index 0, it will always demand a merge operation until the stack is fully
    // collapsed, in order to complete the sort.
    #[inline]
    fn collapse(runs: &[Run]) -> Option<usize> {
        let n = runs.len();
        if n >= 2 && (runs[n - 1].start == 0 ||
                      runs[n - 2].len <= runs[n - 1].len ||
                      (n >= 3 && runs[n - 3].len <= runs[n - 2].len + runs[n - 1].len) ||
                      (n >= 4 && runs[n - 4].len <= runs[n - 3].len + runs[n - 2].len)) {
            if n >= 3 && runs[n - 3].len < runs[n - 1].len {
                Some(n - 3)
            } else {
                Some(n - 2)
            }
        } else {
            None
        }
    }

    #[derive(Clone, Copy)]
    struct Run {
        start: usize,
        len: usize,
    }
}
