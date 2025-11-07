# At

Various utility functions for indexing slices.
This crate provides three methods for indexing slices: `at`, `ref_at`, and `mut_at`.
These methods offer a few benefits over standard indexing:

- They work for any integer type, rather than just `usize`[^0]
- They support Pythonesque negative indices; for example, `nums.at(-1)` returns the last element[^1]
- You explicitly specify whether you're indexing by value (for Copy types), reference, or mutable reference,
  rather than the compiler "magically" choosing the right kind of access
- You can disable *all* bounds checks across the entire program by activating the `unsafe-unchecked` feature;
  this is not recommended unless you absolutely need the performance gains
- If the `fallible` feature is enabled there are 4 additional methods provided that return `Option` instead of panicking on out-of-bounds access: `get_at`, `get_ref_at`, `get_mut_at`, and `extract_at` (implemented only for `Vec`).

All this happens with zero runtime overhead compared to standard indexing.
However, note that checking the validity of signed types is slightly more complex
than for a `usize` due to negative indexing. Signed indexing does not incur any
overhead when the index is known at compile time.

## Examples

```rs
use at::At;

let mut v = vec![8, 2, 1, 0];
assert_eq!(v.at(-1), 0);
assert_eq!(v.ref_at(2), &1);
assert_eq!(v.mut_at(-3), &mut 2);

// With the `fallible` feature enabled:
assert_eq!(v.get_at(-1), Some(0));
assert_eq!(v.get_ref_at(2), Some(&1));
assert_eq!(v.get_mut_at(-3), Some(&mut 2));
assert_eq!(v.get_at(4), None);
assert_eq!(v.get_ref_at(-5), None);
assert_eq!(v.get_mut_at(10), None);
assert_eq!(v.extract_at(-2), Some(1));
assert_eq!(v.get_at(-2), Some(2));
assert_eq!(v.extract_at(-10), None);
```

[^0]: Specifically, the trait bound is `TryInto<isize> + TryInto<usize> + Debug + Copy`.
[^1]: Negative indices are converted into an `isize`, so they cannot be smaller than `isize::MIN`.
     Therefore, `[(); usize::MAX].at(-(usize::MAX as i128))` will panic, even though you might
     expect it to successfully return the first element of the slice.
