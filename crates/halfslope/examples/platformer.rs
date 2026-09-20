use macroquad::prelude::*;
use tilebound_halfslope::prelude::*;

type Sc = ConSc<32>;

#[macroquad::main("Example")]
async fn main() {
    #[rustfmt::skip]
    let map = VecMap::<VertexPattern>::new([
        00, 00, 15, 15, 15, 15, 15, 15, 15, 15, 15, 07, 00, 00, 00,
        00, 00, 00, 13, 15, 07, 00, 00, 00, 00, 00, 00, 00, 00, 00,
        00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00,
        00, 00, 13, 15, 07, 05, 05, 15, 15, 15, 15, 15, 15, 15, 15,
        00, 00, 00, 00, 00, 00, 00, 13, 15, 15, 15, 15, 15, 15, 15,
        00, 00, 00, 00, 00, 05, 05, 05, 13, 15, 15, 15, 15, 15, 15,
        00, 05, 05, 05, 05, 05, 05, 05, 00, 00, 00, 00, 00, 13, 15,
        15, 11, 00, 05, 05, 05, 05, 00, 00, 00, 00, 00, 00, 00, 13,
        15, 15, 11, 00, 00, 00, 00, 14, 15, 11, 00, 00, 00, 00, 14,
        15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
    ].map(|idx| VertexPattern::try_from(idx).unwrap()).to_vec(), (10, 15));

    let mut ctx: Context<Sc, _> = Context::new(&map, AxisMask::ALL);

    // Position.
    let mut x = 0.0;
    let mut y = 0.0;
    // Size.
    let len_x = 18.0;
    let len_y = 25.0;
    // Velocity
    let mut vel_x;
    let mut vel_y = 0.0;
    // AttMask (collisions) and trimask (bounding box intersecting triangle tile state).
    let mut attmask = AxisMask::NONE;
    let mut trimask = VertexMask::NONE.into();

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

        let mut state = State::new::<Sc>(((x, y), (len_x, len_y), attmask, trimask));

        // Slide along slopes on axis x, while stopping on axis y.
        ctx.slide_handler().sweep_by::<AxisX>(&mut state, vel_x);
        ctx.attach_handler().sweep_by::<AxisY>(&mut state, vel_y);

        state.apply::<Sc>(((&mut x, &mut y), &mut attmask, &mut trimask));

        match ctx.map_bounds().any() {
            true => clear_background(GRAY),
            false => clear_background(WHITE),
        }

        draw_map(&map);

        // Draw the rectangle (bounding box).
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

fn draw_map(map: &VecMap<VertexPattern>) {
    let map_size = map.size();
    // Draw white under the map.
    draw_rectangle(
        0.0,
        0.0,
        map_size.y() as f32 * Sc::SCALE,
        map_size.x() as f32 * Sc::SCALE,
        WHITE,
    );
    map.vec.iter().enumerate().for_each(|(i, tile)| {
        let tl = Vec2::new(
            (i as i32 % map_size.y() as i32 * Sc::SCALE_INT) as f32,
            (i as i32 / map_size.y() as i32 * Sc::SCALE_INT) as f32,
        );
        let tr = || tl + Vec2::new(Sc::SCALE, 0.0);
        let bl = || tl + Vec2::new(0.0, Sc::SCALE);
        let br = || tl + Vec2::new(Sc::SCALE, Sc::SCALE);

        match tile {
            VertexPattern::None => {}
            VertexPattern::Full => draw_rectangle(tl.x, tl.y, Sc::SCALE, Sc::SCALE, BROWN),
            VertexPattern::TopOneway => {
                let tr = tr();
                draw_line(tl.x, tl.y, tr.x, tr.y, 2.0, BROWN);
                draw_triangle(tl, tr, tl + Vec2::new(Sc::SCALE / 2.0, 8.0), BROWN);
            }
            VertexPattern::LeftOneway => unreachable!(), // Not used in example.
            VertexPattern::BottomOneway => unreachable!(), // Not used in example.
            VertexPattern::RightOneway => unreachable!(), // Not used in example.
            VertexPattern::TopLeftTri => draw_triangle(tl, tr(), bl(), BROWN),
            VertexPattern::BottomLeftTri => draw_triangle(tl, bl(), br(), BROWN),
            VertexPattern::TopRightTri => draw_triangle(tl, tr(), br(), BROWN),
            VertexPattern::BottomRightTri => draw_triangle(tr(), bl(), br(), BROWN),
        }
    });
}
