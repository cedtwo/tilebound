use std::fmt::Debug;
use std::marker::PhantomData;

use crate::plane::axis::{AxisMask, AxisVec};

/// # BoundBoxView
///
/// Bounding box variables access and mutation.
///
/// `BoundBoxView` provides access to references, mutable references and copies of bounding box
/// variables. This type is implemented on [`BoundBox`] and underlying data tuples. Higher level
/// abstractions may implement this trait to limit [`BoundBox`] variants depending on a resource.
///
/// `BoundBoxView` offers the flexibility in instantiate a [`BoundBox`] from either owned variables,
/// or mutable references. See [`BoundBox`] for more.
pub trait BoundBoxView: Sized {
    /// The external resource type, if any.
    type R: Clone;

    /// Return the top-left position of the bounding-box.
    fn pos(&self) -> AxisVec<f32>;

    /// Return a mutable reference to the top-left position of the bounding-box.
    fn pos_mut(&mut self) -> &mut AxisVec<f32>;

    /// Return the bounding-box length on each axis.
    fn len(&self) -> AxisVec<f32>;

    /// Return the bounding-box attachment [`AxisMask`].
    fn attmask(&self) -> AxisMask;

    /// Return a mutable reference to the bounding-box attachment [`AxisMask`].
    fn attmask_mut(&mut self) -> &mut AxisMask;

    /// Return a reference to the inner resource `R` (if any).
    fn res(&self) -> &Self::R;

    /// Return a mutable reference to the inner resource `R` (if any).
    fn res_mut(&mut self) -> &mut Self::R;
}

/// # BoundBox
///
/// Bounding box variable payload.
///
/// `BoundBox` wraps owned or mutable bounding box variables for displacement operations. `BoundBox`
/// can be created with [`BoundBox::new`] or [`BoundBox::from_mut`] with the base bounding box
/// variables, or [`BoundBox::new_with_res`] or [`BoundBox::from_mut_with_res`] where additional
/// resourced need be supplied. Variables are accessed and mutated via the [`BoundBoxView`] trait.
///
/// `BoundBox` variables are intended to be mutated via an intermediate [`State`], committing
/// changes back to `BoundBox` via [`State::apply`].
///
/// ## Example
///
/// ```
/// # use tilebound::ctx::state::State;
/// # use tilebound::ctx::bnd_box::{BoundBox, BoundBoxView};
/// # use tilebound::plane::scale::ConSc;
/// # use tilebound::plane::endpoint::Endpoint;
/// # use tilebound::plane::axis::{AxisVec, AxisMask};
/// # use tilebound::topology::vertex::Vertex;
/// # type Sc = ConSc<16>;
/// /// Instantiate from owned variables.
/// let mut bounding_box = BoundBox::new((0.0, 0.0), (16.0, 16.0), AxisMask::NONE);
///
/// // Set the right of the box to `32.0` and commit changes.
/// let mut state = State::new::<Sc, _>(&bounding_box);
/// state.set_vertex(Vertex::from_pos::<Sc>(32.0, Endpoint::RIGHT));
/// state.apply::<Sc, _>(&mut bounding_box);
///
/// assert_eq!(bounding_box.pos(), AxisVec::new(16.0, 0.0));
///
/// /// Instantiate from mutable references.
/// let mut pos = AxisVec::new(0.0, 0.0);
/// let mut attmask = AxisMask::NONE;
/// let mut bounding_box = BoundBox::from_mut(&mut pos, (16.0, 16.0), &mut attmask);
///
/// // Set the left of the box to `-16.0` and commit changes.
/// let mut state = State::new::<Sc, _>(&bounding_box);
/// state.set_vertex(Vertex::from_pos::<Sc>(-16.0, Endpoint::LEFT));
/// state.apply::<Sc, _>(&mut bounding_box);
///
/// assert_eq!(pos, AxisVec::new(-16.0, 0.0));
/// ```
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BoundBox<'a, P>(pub P, PhantomData<&'a P>);

impl<'a> BoundBox<'a, OwnedTuple> {
    /// Create a new payload from owned bounding box variables..
    pub fn new(
        pos: impl Into<AxisVec<f32>>,
        len: impl Into<AxisVec<f32>>,
        attmask: AxisMask,
    ) -> Self {
        Self((pos.into(), len.into(), attmask, ()), PhantomData)
    }
}

impl<'a, R> BoundBox<'a, OwnedResTuple<R>> {
    /// Create a new payload from owned bounding box variables, including the given resource `R`.
    pub fn new_with_res(
        pos: impl Into<AxisVec<f32>>,
        len: impl Into<AxisVec<f32>>,
        attmask: AxisMask,
        res: R,
    ) -> Self {
        Self((pos.into(), len.into(), attmask, res), PhantomData)
    }
}

impl<'a> BoundBox<'a, MutTuple<'a>> {
    /// Create a new payload from mutable references to bounding box variables.
    pub fn from_mut(
        pos: &'a mut AxisVec<f32>,
        len: impl Into<AxisVec<f32>>,
        attmask: &'a mut AxisMask,
    ) -> Self {
        Self((pos, len.into(), attmask, ()), PhantomData)
    }
}

impl<'a, R> BoundBox<'a, MutResTuple<'a, R>> {
    /// Create a new payload from mutable references to bounding box variables, including the given
    /// resource `R`.
    pub fn from_mut_with_res(
        pos: &'a mut AxisVec<f32>,
        len: impl Into<AxisVec<f32>>,
        attmask: &'a mut AxisMask,
        res: &'a mut R,
    ) -> Self {
        Self((pos, len.into(), attmask, res), PhantomData)
    }
}

impl<'a, P: BoundBoxView> BoundBoxView for BoundBox<'a, P> {
    type R = P::R;

    fn pos(&self) -> AxisVec<f32> {
        P::pos(&self.0)
    }

    fn pos_mut(&mut self) -> &mut AxisVec<f32> {
        P::pos_mut(&mut self.0)
    }

    fn len(&self) -> AxisVec<f32> {
        P::len(&self.0)
    }

    fn attmask(&self) -> AxisMask {
        P::attmask(&self.0)
    }

    fn attmask_mut(&mut self) -> &mut AxisMask {
        P::attmask_mut(&mut self.0)
    }

    fn res(&self) -> &Self::R {
        P::res(&self.0)
    }

    fn res_mut(&mut self) -> &mut Self::R {
        P::res_mut(&mut self.0)
    }
}

impl<'a, P: BoundBoxView<R: Debug>> Debug for BoundBox<'a, P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KinematicBox")
            .field("pos", &self.pos())
            .field("len", &self.len())
            .field("attmask", &self.attmask())
            .field("res", &self.res())
            .finish()
    }
}

/// A tuple of owned bounding box variables.
pub type OwnedTuple = (AxisVec<f32>, AxisVec<f32>, AxisMask, ());

/// A tuple of owned bounding box variables including a generic resource `R`.
pub type OwnedResTuple<R> = (AxisVec<f32>, AxisVec<f32>, AxisMask, R);

impl<'a, R: Clone> BoundBoxView for OwnedResTuple<R> {
    type R = R;

    fn pos(&self) -> AxisVec<f32> {
        self.0
    }

    fn pos_mut(&mut self) -> &mut AxisVec<f32> {
        &mut self.0
    }

    fn len(&self) -> AxisVec<f32> {
        self.1
    }

    fn attmask(&self) -> AxisMask {
        self.2
    }

    fn attmask_mut(&mut self) -> &mut AxisMask {
        &mut self.2
    }

    fn res(&self) -> &Self::R {
        &self.3
    }

    fn res_mut(&mut self) -> &mut Self::R {
        &mut self.3
    }
}

/// A tuple of mutable bounding box variables.
pub type MutTuple<'a> = (&'a mut AxisVec<f32>, AxisVec<f32>, &'a mut AxisMask, ());

impl<'a> BoundBoxView for MutTuple<'a> {
    type R = ();

    fn pos(&self) -> AxisVec<f32> {
        *self.0
    }

    fn pos_mut(&mut self) -> &mut AxisVec<f32> {
        &mut self.0
    }

    fn len(&self) -> AxisVec<f32> {
        self.1
    }

    fn attmask(&self) -> AxisMask {
        *self.2
    }

    fn attmask_mut(&mut self) -> &mut AxisMask {
        &mut self.2
    }

    fn res(&self) -> &Self::R {
        &self.3
    }

    fn res_mut(&mut self) -> &mut Self::R {
        &mut self.3
    }
}

/// A tuple of mutable bounding box variables including a generic resource `R`.
pub type MutResTuple<'a, R> = (
    &'a mut AxisVec<f32>,
    AxisVec<f32>,
    &'a mut AxisMask,
    &'a mut R,
);

impl<'a, R: Clone> BoundBoxView for MutResTuple<'a, R> {
    type R = R;

    fn pos(&self) -> AxisVec<f32> {
        *self.0
    }

    fn pos_mut(&mut self) -> &mut AxisVec<f32> {
        &mut self.0
    }

    fn len(&self) -> AxisVec<f32> {
        self.1
    }

    fn attmask(&self) -> AxisMask {
        *self.2
    }

    fn attmask_mut(&mut self) -> &mut AxisMask {
        &mut self.2
    }

    fn res(&self) -> &Self::R {
        &self.3
    }

    fn res_mut(&mut self) -> &mut Self::R {
        &mut self.3
    }
}

#[cfg(test)]
mod tests {
    use crate::ctx::state::State;
    use crate::plane::endpoint::Endpoint;
    use crate::plane::scale::ConSc;
    use crate::topology::vertex::Vertex;

    use super::*;

    type Sc = ConSc<16>;

    #[test]
    fn owned_payload() {
        let mut payload = BoundBox::new((0.0, 0.0), (0.0, 0.0), AxisMask::NONE);
        let mut state = State::new::<Sc, _>(&payload);

        state.set_vertex(Vertex::from_pos::<Sc>(16.0, Endpoint::LEFT));
        state.set_vertex(Vertex::from_pos::<Sc>(32.0, Endpoint::TOP));
        state.attach(Endpoint::RIGHT);
        state.apply::<Sc, _>(&mut payload);

        assert_eq!(payload.pos(), AxisVec::new(16.0, 32.0));
        assert_eq!(payload.attmask(), AxisMask::RIGHT);
    }

    #[test]
    fn owned_with_res() {
        let mut payload = BoundBox::new_with_res((0.0, 0.0), (0.0, 0.0), AxisMask::NONE, 0);
        let mut state = State::new::<Sc, _>(&payload);

        state.set_vertex(Vertex::from_pos::<Sc>(16.0, Endpoint::LEFT));
        state.set_vertex(Vertex::from_pos::<Sc>(32.0, Endpoint::TOP));
        state.attach(Endpoint::RIGHT);
        *state.res_mut() = 1;
        state.apply::<Sc, _>(&mut payload);

        assert_eq!(payload.pos(), AxisVec::new(16.0, 32.0));
        assert_eq!(payload.attmask(), AxisMask::RIGHT);
        assert_eq!(*payload.res(), 1);
    }

    #[test]
    fn mut_payload() {
        let mut pos = (0.0, 0.0).into();
        let mut attmask = AxisMask::NONE;
        let mut payload = BoundBox::from_mut(&mut pos, (0.0, 0.0), &mut attmask);
        let mut state = State::new::<Sc, _>(&payload);

        state.set_vertex(Vertex::from_pos::<Sc>(16.0, Endpoint::LEFT));
        state.set_vertex(Vertex::from_pos::<Sc>(32.0, Endpoint::TOP));
        state.attach(Endpoint::RIGHT);
        state.apply::<Sc, _>(&mut payload);

        assert_eq!(payload.pos(), AxisVec::new(16.0, 32.0));
        assert_eq!(payload.attmask(), AxisMask::RIGHT);
    }

    #[test]
    fn mut_with_res() {
        let mut pos = (0.0, 0.0).into();
        let mut attmask = AxisMask::NONE;
        let mut int = 0;
        let mut payload = BoundBox::from_mut_with_res(&mut pos, (0.0, 0.0), &mut attmask, &mut int);
        let mut state = State::new::<Sc, _>(&payload);

        state.set_vertex(Vertex::from_pos::<Sc>(16.0, Endpoint::LEFT));
        state.set_vertex(Vertex::from_pos::<Sc>(32.0, Endpoint::TOP));
        state.attach(Endpoint::RIGHT);
        *state.res_mut() = 1;
        state.apply::<Sc, _>(&mut payload);

        assert_eq!(payload.pos(), AxisVec::new(16.0, 32.0));
        assert_eq!(payload.attmask(), AxisMask::RIGHT);
        assert_eq!(*payload.res(), 1);
    }
}
