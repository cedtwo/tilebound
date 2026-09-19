use bitvec::prelude::*;
use macroquad::prelude::*;
use tilebound_solid::prelude::*;

type Sc = ConSc<32>;

#[macroquad::main("Example")]
async fn main() {
    #[rustfmt::skip]
    let map = BitMap::new(bits![
        0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0,
        0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1,
        0, 0, 0, 1, 1, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1,
        0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1,
        0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1,
        1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
        1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    ].into(), (10, 15));

    let mut ctx: Context<Sc, _> = Context::new(&map, AxisMask::ALL);

    // Collider position.
    let mut x = 0.0;
    let mut y = 0.0;
    // Collider size.
    let len_x = 18.0;
    let len_y = 25.0;
    // Velocity
    let mut vel_x;
    let mut vel_y = 0.0;
    // AttMask (collisions).
    let mut attmask = AxisMask::NONE;

    // Movement (left/right), gravity and jumping velocities.
    const MOVE: f32 = Sc::SCALE * 3.0;
    const GRAV: f32 = 10.0;
    const JUMP: f32 = -Sc::SCALE * 9.0;

    loop {
        if is_key_pressed(KeyCode::Enter) {
            match ctx.map_bounds().any() {
                true => {
                    // Force recheck collisions when turning off map-bound (rechecks on entering a new tile).
                    attmask = AxisMask::NONE;
                    *ctx.map_bounds_mut() = AxisMask::NONE;
                }
                false => *ctx.map_bounds_mut() = AxisMask::ALL,
            }
        }

        match (is_key_down(KeyCode::A), is_key_down(KeyCode::D)) {
            (true, false) => vel_x = -MOVE * get_frame_time(),
            (false, true) => vel_x = MOVE * get_frame_time(),
            _ => vel_x = 0.0,
        }

        // Return the attached (colliding) endpoint for the Y axis. Note that only one endpoint will
        // ever be set at a time.
        match attmask.first_end::<AxisY>() {
            // Colliding with the ceiling. See also `AxisMask::top`.
            Some(Endpoint::Lower) => {
                // Prevent upward velocity when colliding with the ceiling.
                vel_y = f32::max(vel_y, 0.0);
                vel_y += GRAV * get_frame_time();
            }
            // Colliding with the floor. See also `AxisMask::bottom`.
            Some(Endpoint::Upper) => {
                // Set velocity to zero or jump.
                if is_key_pressed(KeyCode::Space) {
                    vel_y = JUMP * get_frame_time();
                } else {
                    vel_y = 0.0
                }
            }
            // Falling.
            None => {
                vel_y += GRAV * get_frame_time();
            }
        }

        let mut state = State::new::<Sc>(((x, y), (len_x, len_y), attmask));

        // Slide along slopes on axis x, while stopping on axis y.
        ctx.sweep_by::<AxisX>(&mut state, vel_x);
        ctx.sweep_by::<AxisY>(&mut state, vel_y);

        state.apply::<Sc>(((&mut x, &mut y), &mut attmask));

        match ctx.map_bounds().any() {
            true => clear_background(GRAY),
            false => clear_background(WHITE),
        }

        draw_map(&map);

        // Draw the collider.
        draw_rectangle(
            x,
            y,
            len_x,
            len_y,
            if attmask.bottom() { BLUE } else { RED },
        );

        draw_text(
            "Move the collider with WASD and press SPACE to jump",
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
