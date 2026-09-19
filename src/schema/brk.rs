use std::ops::ControlFlow;

/// # Break
///
/// [`ControlFlow::Break`] variant for displacement operations. Accepts a generic `T` for returning
/// additional tile bound collision data. Consider implementing `From<T> for ControlFlow<Break<T>>`
/// for convenience.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Break<T> {
    /// No attempted displacement due to a colliding edge, an intersected target, or a delta of `0.0`.
    None,
    /// Displaced to the target vertex without colliding.
    ReachedTarget,
    /// Collided at some point prior to the target.
    Collision(T),
}

impl<T> From<Break<T>> for ControlFlow<Break<T>> {
    fn from(brk: Break<T>) -> Self {
        ControlFlow::Break(brk)
    }
}
