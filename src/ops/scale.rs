use crate::{
    plane::{axis::AxisVec, scale::Scale},
    view::tilemap::TileMap,
};

/// Return a *one-dimensional* raw `f32` position from an `i32` tile index. This method is exposed
/// as it is a very common operation, however [`Vertex`](crate::topology::vertex::Vertex) should be
/// preferred for position/tile index conversion during displacement.
#[inline]
pub fn index_to_raw_pos<Sc: Scale>(index: i32) -> f32 {
    index as f32 * Sc::SCALE
}

/// Return a *one-dimensional* `i32` tile index from a raw `f32` position. This method is exposed as
/// it is a very common operation, however [`Vertex`](crate::topology::vertex::Vertex) should be
/// preferred for position/tile index conversion during displacement.
#[inline]
pub fn raw_pos_to_index<Sc: Scale>(pos: f32) -> i32 {
    (pos / Sc::SCALE).floor() as i32
}

/// Return a two-dimensional `AxisVec<i32>` tile index from a contiguous array index.
pub fn array_index_to_tile_index<Map: TileMap>(i: usize, map: Map) -> AxisVec<i32> {
    let map_size = map.size();
    AxisVec::new((i % map_size.y) as i32, (i / map_size.y) as i32)
}

/// Return a two-dimensional `AxisVec<f32>` tile position from a contiguous array index. Returns the
/// top-left position of the tile.
pub fn array_index_to_tile_pos<Sc: Scale, Map: TileMap>(i: usize, map: Map) -> AxisVec<f32> {
    let map_size = map.size();
    AxisVec::new(
        (i % map_size.y) as f32 * Sc::SCALE,
        (i / map_size.y) as f32 * Sc::SCALE,
    )
}
