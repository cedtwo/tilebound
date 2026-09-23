## Tilebound-Halfslope

A tilemap collision libary supporting half-tile slopes.

### Summary

`tilebound_halfslope` is an implementation of [`tilebound`] supporting half-tile 45 degree
slopes, and sliding and attaching behaviour. It offers the following features:
- Half tile slopes of any direction,
- One-way tiles of any direction,
- The choice to either slide or attach (stop displacement) on colliding with a hypotenuse,
- Axis seperate *sweep* displacement that only checks tiles on-demand (when moving into a new
tile, or off a ledge),
- Attach/detach behaviour (for asserting collisions) when moving on/off surfaces,
- Tile range inspection on collision (eg. Check where the first halfslope tile is when colliding
with a ledge),
- Perfect tile-bound alignment (no fractional displacement for an edge on a tile bound),
- Support tiles as small as half a byte in size.

A large number of these features are provided by the implementing library, [`tilebound`].

## Usage

The following demonstrates `tilebound_halfslope` usage.

```rust
// A 2*2 map with a single tile in the top-right triangle tile (represented as a u8 for this example).
const MAP: ArrayMap<2, 2, 4, u8> = ArrayMap::new_unchecked([
    00, 13,
    00, 00
]);

// Declare a constant tile size (a 16 unit square).
type Sc = ConSc<16>;
// Declare a `Context`, passing in the map and declaring any solid map bounds.
let ctx = Context::<Sc, _>::new(&MAP, AxisMask::NONE);

// Create a mutable bounding box. Here we define a box at the top left of `(8.0 * 8.0)` units in size.
let mut rect = BoundBox::new_with_res((0.0, 0.0), (8.0, 8.0), AxisMask::NONE, VertexMask::NONE);

// Displace 16.0 units to the right (one exact tile) using the `Slide` handler (slides on intersecting a triangle hypotenuse).
let brk = ctx.slide_handler().sweep_by::<AxisX, _>(&mut rect, 16.0);

assert_matches!(brk, Break::ReachedTarget); // Assert we reached the target.
assert_eq!(rect.pos(), (16.0, 8.0).into()); // Assert we displaced downward on reaching the triangle tile.
assert_eq!(rect.attmask(), AxisMask::NONE); // Assert we are not colliding (colliding prevents displacement).

// Displace 8.0 units up (half a tile) using the `Attach` handler (attaches on intersecting a triangle hypotenuse).
let brk = ctx.attach_handler().sweep_by::<AxisY, _>(&mut rect, -8.0);

assert_matches!(brk, Break::Collision(Collision::TriangleHypotenuse)); // Assert this time we collided with the triangle hypotenuse.
assert_eq!(rect.pos(), (16.0, 8.0).into()); // Assert we didn't displace at all (we were already on the hypotenuse).
assert_eq!(rect.attmask(), AxisMask::TOP); // Assert we are colliding at the top.
```

### features

`tilebound_halfslope` supports the following features:

Feature | Description | Exposed Type(s)
---|---|---
`arraymap` | Exports tilemap implementations built on `std` types. | [`ArrayMap`](crate::prelude::ArrayMap), [`VecMap`](crate::prelude::VecMap)

### Example

Two examples are provided using `macroquad` as a backend. `bevy` uses a different axis
orientation so variables will either need to be manually oriented, or another rendering backend
(eg. `bevy_framebuffer`) will be needed.

Example | Description | Command
---|---|---
`minimal` | A minimal example with input and rendering. | `cargo run --example minimal --features="arraymap"`
`platformer` | A minimal example with gravity and jumping. | `cargo run --example platformer --features="arraymap"`

License: MIT OR Apache-2.0
