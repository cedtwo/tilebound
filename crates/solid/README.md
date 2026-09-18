## Tilebound-Solid

A small boolean/bit tilemap collision libary.

### Summary

`tilebound_solid` is a simle implementation of [`tilebound`] supporting arrays of simple
solid/empty tiles. It offers the following features:
- Axis seperate *sweep* displacement that only checks tiles on-demand (when moving into a new
tile),
- Attach/detach behaviour (for asserting collisions) when moving on/off surfaces,
- Tile range inspection on collision (eg. Check where the first solid tile is when colliding
with a ledge),
- Perfect tile-bound alignment (no fractional displacement for an edge on a tile bound),
- Support for `bool`, `u8` (0 or 1) and `bitvec` backed tile arrays.

A large number of these features are provided by the implementing library, [`tilebound`].

### features

`tilebound_solid` supports the following features:

Feature | Description | Exposed Type(s)
---|---|---
`arraymap` | Exports tilemap implementations built on `std` types. | [`ArrayMap`](crate::prelude::ArrayMap), [`VecMap`](crate::prelude::VecMap)
`bitmap` | Exports a `bitvec` backed tilemap (where each tile represents one bit). | [`BitMap`](crate::prelude::BitMap)

### Example

A minimal example is provided using `macroquad` as a backend. `bevy` uses a different axis
orientation so variables will either need to be manually oriented, or another rendering backend
(such as `bevy_framebuffer`) will be needed.

```bash
cargo run --example minimal --features="bitmap"
```

License: MIT OR Apache-2.0
