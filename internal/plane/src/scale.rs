use std::fmt::Debug;

use marker_value::MarkerValue;

/// # Scale
///
/// An integer representation of a tile size (for both axes).
///
/// A single [`Scale`] variable should be a persistant across **all** operations and should
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
