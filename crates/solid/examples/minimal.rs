use bitvec::prelude::*;
use macroquad::prelude::*;
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

    // Collider position.
    let mut x = 0.0;
    let mut y = 0.0;
    // Collider size.
    let len_x = 18.0;
    let len_y = 25.0;
    // Velocity
    let mut vel_x;
    let mut vel_y;
    // AttMask (collisions).
    let mut attmask = AxisMask::NONE;

    // Move at five tiles per second.
    const V: f32 = Sc::SCALE * 5.0;

    loop {
        if is_key_pressed(KeyCode::Enter) {
            match ctx.scene().map_bounds.any() {
                true => {
                    // Force recheck collisions.
                    attmask = AxisMask::NONE.into();
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

        let mut state = State::new::<Sc>(((x, y), (len_x, len_y), attmask));

        ctx.sweep_by::<AxisX>(&mut state, vel_x);
        ctx.sweep_by::<AxisY>(&mut state, vel_y);

        state.apply::<Sc>(((&mut x, &mut y), &mut attmask));

        match ctx.map_bounds().any() {
            true => clear_background(GRAY),
            false => clear_background(WHITE),
        }

        draw_map(&map);

        // Draw the collider.
        draw_rectangle(x, y, len_x, len_y, RED);

        draw_text(
            "Move the collider with WASD",
            20.0,
            (map.size().y() as f32 + 1.0) * Sc::SCALE,
            20.0,
            BLACK,
        );
        draw_text(
            "Press ENTER to toggle map bounds",
            20.0,
            (map.size().y() as f32 + 1.5) * Sc::SCALE,
            20.0,
            BLACK,
        );

        next_frame().await
    }
}

fn draw_map(map: &BitMap) {
    // Draw the map.
    map.store.iter().enumerate().for_each(|(i, is_solid)| {
        draw_rectangle(
            (i as i32 % map.size().y() as i32 * Sc::SCALE_INT) as f32,
            (i as i32 / map.size().y() as i32 * Sc::SCALE_INT) as f32,
            Sc::SCALE,
            Sc::SCALE,
            if *is_solid { BLACK } else { WHITE },
        )
    });
}
