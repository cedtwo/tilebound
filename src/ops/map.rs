//! Tilemap view helper functions.
use crate::ctx::scene::Scene;
use crate::ctx::state::State;
use crate::plane::axis::Axis;
use crate::plane::scale::Scale;
use crate::topology::vertex::Vertex;
use crate::view::index::{SliceIndex, SliceIndexError};
use crate::view::{inspect::TileMapInspect, tilemap::TileMapView};

/// Initialize an intersecting [`SliceIndex`] from a vertex, returning a [`TileMapInspect`] type or
/// `None` if out of bounds.
pub fn inspect_intersected<A, R, Sc, Map, Ins>(
    vertex: Vertex<A>,
    state: &State<R>,
    scene: &Scene<Sc, Map>,
) -> Result<Ins, SliceIndexError<A::T>>
where
    A: Axis,
    Sc: Scale,
    Map: TileMapView<A::T>,
    Ins: TileMapInspect<A::T, Map>,
{
    let (t_index, range) = (vertex.index(), state.index_range::<A::T, Sc>());
    SliceIndex::<A::T>::try_new(t_index, range, !vertex.endpoint(), scene.size())
        .map(|index| Ins::inspect(&scene.map, &index))
}

/// Initialize an intersecting [`SliceIndex`] from the next tiles after a vertex, returning a
/// [`TileMapInspect`] type or `None` if out of bounds.
pub fn inspect_next_intersected<A, R, Sc, Map, Ins>(
    vertex: Vertex<A>,
    state: &State<R>,
    scene: &Scene<Sc, Map>,
) -> Result<Ins, SliceIndexError<A::T>>
where
    A: Axis,
    Sc: Scale,
    Map: TileMapView<A::T>,
    Ins: TileMapInspect<A::T, Map>,
{
    let (t_index, range) = (
        vertex.endpoint().incr(vertex.index()),
        state.index_range::<A::T, Sc>(),
    );
    SliceIndex::<A::T>::try_new(t_index, range, !vertex.endpoint(), scene.size())
        .map(|index| Ins::inspect(&scene.map, &index))
}
