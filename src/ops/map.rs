//! Tilemap view helper functions.
use tilebound_plane::axis::Axis;
use tilebound_plane::scale::Scale;
use tilebound_schema::scene::Scene;
use tilebound_schema::state::State;
use tilebound_topology::vertex::Vertex;
use tilebound_view::index::{SliceIndex, SliceIndexError};
use tilebound_view::{inspect::TileMapInspect, tilemap::TileMapView};

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
