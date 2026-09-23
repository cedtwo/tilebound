use bitvec::prelude::*;
use macroquad::prelude::*;
use tilebound::ops::array_index_to_tile_pos;
use tilebound_solid::prelude::*;

type Sc = ConSc<32>;

#[macroquad::main("Example")]
async fn main() {
    #[rustfmt::skip]
    let map = BitMap::new(bits![
        0, 0, 1, 1, 1, 1, 0, 0, 0, 0,
        0, 0, 0, 0, 1, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 1, 1, 0, 0, 1, 1, 1,
        0, 0, 0, 0, 0, 0, 0, 1, 1, 1,
        1, 0, 0, 0, 0, 0, 0, 0, 1, 1,
        1, 0, 0, 0, 0, 0, 0, 0, 0, 1,
        1, 0, 0, 1, 1, 0, 0, 0, 0, 0,
        1, 0, 0, 1, 1, 0, 0, 0, 0, 0,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    ].into(), (10, 10));

    let mut ctx: Context<Sc, _> = Context::new(&map, AxisMask::ALL);

    // Bounding box variables.
    let mut rect = BoundBox::new((0.0, 0.0), (18.0, 25.0), AxisMask::NONE);

    // Velocity
    let mut vel_x;
    let mut vel_y;

    // Move at five tiles per second.
    const V: f32 = Sc::SCALE * 5.0;

    loop {
        if is_key_pressed(KeyCode::Enter) {
            match ctx.scene().map_bounds.any() {
                true => {
                    // Force recheck collisions.
                    rect.attmask_mut().clear();
                    *ctx.map_bounds_mut() = AxisMask::NONE;
                }
                false => *ctx.map_bounds_mut() = AxisMask::ALL,
            }
        }

        match (is_key_down(KeyCode::A), is_key_down(KeyCode::D)) {
            (true, false) => vel_x = -V * get_frame_time(),
            (false, true) => vel_x = V * get_frame_time(),
            _ => vel_x = 0.0,
        }
        match (is_key_down(KeyCode::W), is_key_down(KeyCode::S)) {
            (true, false) => vel_y = -V * get_frame_time(),
            (false, true) => vel_y = V * get_frame_time(),
            _ => vel_y = 0.0,
        }

        // Displace `rect`.
        ctx.sweep_by::<AxisX, _>(&mut rect, vel_x);
        ctx.sweep_by::<AxisY, _>(&mut rect, vel_y);

        match ctx.map_bounds().any() {
            true => clear_background(GRAY),
            false => clear_background(WHITE),
        }

        draw_map(&map);

        // Draw the collider.
        draw_rectangle(rect.pos().x, rect.pos().y, rect.len().x, rect.len().y, RED);

        draw_text(
            "Move the collider with WASD",
            20.0,
            (map.size().y as f32 + 1.0) * Sc::SCALE,
            20.0,
            BLACK,
        );
        draw_text(
            "Press ENTER to toggle map bounds",
            20.0,
            (map.size().y as f32 + 1.5) * Sc::SCALE,
            20.0,
            BLACK,
        );

        next_frame().await
    }
}

fn draw_map(map: &BitMap) {
    // Draw the map.
    map.store.iter().enumerate().for_each(|(i, is_solid)| {
        let tl = array_index_to_tile_pos::<Sc, _>(i, map);
        draw_rectangle(
            tl.x,
            tl.y,
            Sc::SCALE,
            Sc::SCALE,
            if *is_solid { BLACK } else { WHITE },
        )
    });
}
