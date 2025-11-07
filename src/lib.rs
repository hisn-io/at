#![no_std]
#![warn(clippy::pedantic)]
#![deny(missing_docs)]
#![allow(clippy::inline_always)]
#![doc = include_str!("../README.md")]

#[cfg(feature = "unsafe-unchecked")]
use core::hint::unreachable_unchecked;

extern crate alloc;
use alloc::vec::Vec;

mod private {
	pub trait ToIndex: TryInto<isize> + TryInto<usize> + core::fmt::Debug + Copy {}
	impl<T: TryInto<isize> + TryInto<usize> + core::fmt::Debug + Copy> ToIndex for T {}
}

// Trait alias for TryInto<isize> + TryInto<usize> + core::fmt::Debug + Copy
use private::ToIndex;

#[inline(always)]
fn check_index(idx: impl ToIndex, len: usize) -> Option<usize> {
	let resolved = if let Ok(unsigned_index) = idx.try_into() {
		unsigned_index
	} else {
		let signed_index = idx.try_into().ok()?;
		// If this overflows, the index is guaranteed invalid (this is handled at the end of this function).
		// Proof: `signed_index` must be negative; otherwise, the previous branch would have succeeded.
		// Thus `signed_index` is any negative number in `isize::MIN..0`. After the addition,
		// `resolved` is in `len + isize::MIN..len`. If the length is extremely large, that is `len > isize::MAX`,
		// (only possible for ZST slices) overflow does not occur. Otherwise, the wrapped range
		// is `len + isize::MIN..0` which becomes `len + isize::MAX + 1..=usize::MAX`. But since we
		// know that `len` is at most `isize::MAX` in this case, the wrapped range is always invalid.
		// Therefore, we use the wrapping method to discourage the compiler from adding pointless runtime checks.
		len.wrapping_add_signed(signed_index)
	};

	(resolved < len).then_some(resolved)
}

#[cfg(not(feature = "unsafe-unchecked"))]
#[inline(never)]
fn panic_bounds_check(idx: impl ToIndex, len: usize) -> ! {
	panic!("index out of bounds: the len is {len} but the index is {idx:?}")
}

/// This trait provides the `at`, `ref_at`, and `mut_at` methods for slices
/// as well as any type that can be deferenced to a slice.
pub trait At {
	/// Access a particular index of a `Copy` type. Panics if the index is out of bounds.
	///
	/// # Examples
	/// ```
	/// use at::At;
	/// let a = [1, 2, 3];
	///
	/// assert_eq!(a.at(2), 3);
	/// assert_eq!(a.at(-2), 2);
	/// ```
	#[inline(always)]
	fn at<T>(&self, idx: impl ToIndex) -> T
	where
		Self: AsRef<[T]>,
		T: Copy,
	{
		let slice = self.as_ref();
		let len = slice.len();

		match check_index(idx, len) {
			Some(i) => slice[i],
			#[cfg(feature = "unsafe-unchecked")]
			None => unsafe { unreachable_unchecked() },
			#[cfg(not(feature = "unsafe-unchecked"))]
			None => panic_bounds_check(idx, len),
		}
	}

	/// Access a particular index by reference. Panics if the index is out of bounds.
	///
	/// # Examples
	/// ```
	/// use at::At;
	/// let a = [1, 2, 3];
	///
	/// assert_eq!(a.ref_at(2), &3);
	/// assert_eq!(a.ref_at(-2), &2);
	/// ```
	#[inline(always)]
	fn ref_at<T>(&self, idx: impl ToIndex) -> &T
	where
		Self: AsRef<[T]>,
	{
		let slice = self.as_ref();
		let len = slice.len();

		match check_index(idx, len) {
			Some(i) => &slice[i],
			#[cfg(feature = "unsafe-unchecked")]
			None => unsafe { unreachable_unchecked() },
			#[cfg(not(feature = "unsafe-unchecked"))]
			None => panic_bounds_check(idx, len),
		}
	}

	/// Access a particular index by mutable reference. Panics if the index is out of bounds.
	///
	/// # Examples
	/// ```
	/// use at::At;
	/// let mut a = [1, 2, 3];
	///
	/// assert_eq!(a.mut_at(2), &mut 3);
	/// assert_eq!(a.mut_at(-2), &mut 2);
	/// ```
	#[inline(always)]
	fn mut_at<T>(&mut self, idx: impl ToIndex) -> &mut T
	where
		Self: AsMut<[T]>,
	{
		let slice = self.as_mut();
		let len = slice.len();

		match check_index(idx, len) {
			Some(i) => &mut slice[i],
			#[cfg(feature = "unsafe-unchecked")]
			None => unsafe { unreachable_unchecked() },
			#[cfg(not(feature = "unsafe-unchecked"))]
			None => panic_bounds_check(idx, len),
		}
	}

	#[cfg(feature = "fallible")]
	/// Get a particular index of a `Copy` type. Returns `None` if the index is out of bounds.
	///
	/// # Examples
	/// ```
	/// use at::At;
	/// let a = [1, 2, 3];
	///
	/// assert_eq!(a.get_at(2), Some(3));
	/// assert_eq!(a.get_at(-2), Some(2));
	/// assert_eq!(a.get_at(5), None);
	/// ```
	#[inline(always)]
	fn get_at<T>(&self, idx: impl ToIndex) -> Option<T>
	where
		Self: AsRef<[T]>,
		T: Copy,
	{
		let slice = self.as_ref();
		let len = slice.len();

		check_index(idx, len).map(|i| slice[i])
	}

	#[cfg(feature = "fallible")]
	/// Get a particular index by reference. Returns `None` if the index is out of bounds.
	///
	/// # Examples
	/// ```
	/// use at::At;
	/// let a = [1, 2, 3];
	///
	/// assert_eq!(a.get_ref_at(2), Some(&3));
	/// assert_eq!(a.get_ref_at(-2), Some(&2));
	/// assert_eq!(a.get_ref_at(5), None);
	/// ```
	#[inline(always)]
	fn get_ref_at<T>(&self, idx: impl ToIndex) -> Option<&T>
	where
		Self: AsRef<[T]>,
	{
		let slice = self.as_ref();
		let len = slice.len();

		check_index(idx, len).map(|i| &slice[i])
	}

	#[cfg(feature = "fallible")]
	/// Get a particular index by mutable reference. Returns `None` if the index is out of bounds.
	///
	/// # Examples
	/// ```
	/// use at::At;
	/// let mut a = [1, 2, 3];
	///
	/// assert_eq!(a.get_mut_at(2), Some(&mut 3));
	/// assert_eq!(a.get_mut_at(-2), Some(&mut 2));
	/// assert_eq!(a.get_mut_at(5), None);
	/// ```
	#[inline(always)]
	fn get_mut_at<T>(&mut self, idx: impl ToIndex) -> Option<&mut T>
	where
		Self: AsMut<[T]>,
	{
		let slice = self.as_mut();
		let len = slice.len();

		check_index(idx, len).map(|i| &mut slice[i])
	}

	#[cfg(feature = "fallible")]
	/// Extracts the element at the given index from the slice, shifting all elements after it
	/// to the left. Returns `None` if the index is out of bounds.
	///
	/// # Examples
	/// ```
	/// use at::At;
	/// let mut a = vec![1, 2, 3, 4];
	///
	/// assert_eq!(a.extract_at(1), Some(2));
	/// assert_eq!(a, vec![1, 3, 4]);
	/// assert_eq!(a.extract_at(-1), Some(4));
	/// assert_eq!(a, vec![1, 3]);
	/// assert_eq!(a.extract_at(5), None);
	/// ```
	#[inline(always)]
	fn extract_at<T>(&mut self, idx: impl ToIndex) -> Option<T>
	where
		Self: AsMut<Vec<T>>,
	{
		let slice = self.as_mut();
		let len = slice.len();

		check_index(idx, len).map(|i| slice.remove(i))
	}
}

impl<T> At for T {}

#[cfg(test)]
mod test {
	use super::At;
	extern crate alloc;
	use alloc::vec;

	#[test]
	fn test_positive() {
		let mut v = vec![1, 2, 3];
		assert_eq!(v.at(0u8), 1);
		assert_eq!(v.ref_at(1i128), &2);
		assert_eq!(v.mut_at(2isize), &mut 3);
		#[cfg(feature = "fallible")]
		{
			assert_eq!(v.get_at(0), Some(1));
			assert_eq!(v.get_ref_at(1), Some(&2));
			assert_eq!(v.get_mut_at(2), Some(&mut 3));
			assert_eq!(v.get_at(3), None);
			assert_eq!(v.get_ref_at(3), None);
			assert_eq!(v.get_mut_at(3), None);
			assert_eq!(v.extract_at(2), Some(3));
			assert_eq!(v.get_at(3), None);
			assert_eq!(v.extract_at(10), None);
		}
	}

	#[test]
	fn test_negative() {
		let mut v = vec![4, 5, 6];
		assert_eq!(v.at(-1i8), 6);
		assert_eq!(v.ref_at(-2i128), &5);
		assert_eq!(v.mut_at(-3isize), &mut 4);
		#[cfg(feature = "fallible")]
		{
			assert_eq!(v.get_at(-1), Some(6));
			assert_eq!(v.get_ref_at(-2), Some(&5));
			assert_eq!(v.get_mut_at(-3), Some(&mut 4));
			assert_eq!(v.get_at(-10), None);
			assert_eq!(v.get_ref_at(-11), None);
			assert_eq!(v.get_mut_at(-12), None);
			assert_eq!(v.extract_at(-2), Some(5));
			assert_eq!(v.get_at(-2), Some(4));
			assert_eq!(v.extract_at(-10), None);
		}
	}

	#[test]
	#[should_panic(expected = "index out of bounds: the len is 1 but the index is -2")]
	fn test_panic() {
		let s = ["hi"];
		let _ = s.at(-2);
	}

	#[test]
	fn test_zst() {
		let giant = [(); usize::MAX];
		giant.at(-1);
		giant.at(usize::MAX - 1);
		giant.at(isize::MIN);
	}
}
