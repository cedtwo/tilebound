use std::fmt::Debug;

use marker_value::MarkerValue;

/// # Scale
///
/// An integer representation of a tile size (for both axes).
///
/// `Scale` defines the length of a (square) tile on each dimension. This has an influence on all
/// declared sizes and deltas in relation to the given tile size. For example, a bounding box with a
/// size of `8.0` on one axis occupies `8` tiles where the scale is `1`, but half a tile where the
/// scale is `16`.
///
/// A single `Scale` variable should be a presisted across **all** operations and should
/// preferably a power of two. See [`ConSc`].
pub trait Scale: MarkerValue<u32> {
    /// Get the scale as an `f32` float.
    const SCALE: f32 = Self::VALUE as f32;
    /// Get the scale as a `i32` integer.
    const SCALE_INT: i32 = Self::VALUE as i32;
}

/// # ConSc (Constant Scale)
///
/// A constant integer scale value. See [`Scale`].
#[derive(MarkerValue, Clone, Copy)]
pub struct ConSc<const N: u32>;

impl<const N: u32> Scale for ConSc<N> {}

impl<const N: u32> Debug for ConSc<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Scale").field(&N).finish()
    }
}
