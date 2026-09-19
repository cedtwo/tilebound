use macroquad::prelude::*;
use tilebound_halfslope::prelude::*;

type Sc = ConSc<32>;

#[macroquad::main("Example")]
async fn main() {
    #[rustfmt::skip]
    let map = VecMap::new([
        00, 00, 15, 15, 15, 15, 07, 00, 00, 00,
        00, 00, 00, 00, 03, 00, 00, 00, 00, 00,
        00, 00, 00, 14, 11, 00, 00, 00, 00, 00,
        00, 00, 00, 15, 15, 00, 00, 15, 15, 11,
        00, 00, 00, 13, 15, 00, 14, 15, 15, 07,
        15, 00, 00, 00, 00, 00, 13, 07, 00, 00,
        15, 00, 00, 00, 00, 00, 00, 00, 00, 00,
        15, 00, 00, 14, 11, 00, 00, 00, 00, 00,
        15, 00, 00, 15, 15, 00, 00, 14, 15, 15,
        15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
    ].map(|idx| VertexPattern::try_from(idx).unwrap()).to_vec(), (10, 10));

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
    // AttMask (collisions) and trimask (bounding box intersecting triangle tile state).
    let mut attmask = AxisMask::NONE;
    let mut trimask = VertexMask::NONE.into();

    // Move at five tiles per second.
    const V: f32 = Sc::SCALE * 5.0;

    let mut slide = true;

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

        if is_key_pressed(KeyCode::Tab) {
            slide.toggle();
            // Force recheck collisions.
            attmask = AxisMask::NONE.into();
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

        let mut state = State::new::<Sc>(((x, y), (len_x, len_y), attmask, trimask));

        match slide {
            true => {
                ctx.slide_handler().sweep_by::<AxisX>(&mut state, vel_x);
                ctx.slide_handler().sweep_by::<AxisY>(&mut state, vel_y);
            }
            false => {
                ctx.attach_handler().sweep_by::<AxisX>(&mut state, vel_x);
                ctx.attach_handler().sweep_by::<AxisY>(&mut state, vel_y);
            }
        }

        state.apply::<Sc>(((&mut x, &mut y), &mut attmask, &mut trimask));

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
        draw_text(
            format!("Press TAB to toggle sliding (Enabled: {slide})"),
            20.0,
            (map.size().y() as f32 + 2.0) * Sc::SCALE,
            20.0,
            BLACK,
        );

        next_frame().await
    }
}

fn draw_map(map: &VecMap<VertexPattern>) {
    let map_size = map.size();
    draw_rectangle(
        0.0,
        0.0,
        map_size.x() as f32 * Sc::SCALE,
        map_size.y() as f32 * Sc::SCALE,
        WHITE,
    );
    map.vec.iter().enumerate().for_each(|(i, tile)| {
        // The top-left position of a tile.
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
            VertexPattern::TopOneway => unreachable!(), // Not used in example.
            VertexPattern::LeftOneway => {
                let bl = bl();
                draw_line(tl.x, tl.y, bl.x, bl.y, 4.0, BROWN);
            }
            VertexPattern::BottomOneway => unreachable!(), // Not used in example.
            VertexPattern::RightOneway => unreachable!(),  // Not used in example.
            VertexPattern::TopLeftTri => draw_triangle(tl, tr(), bl(), BROWN),
            VertexPattern::BottomLeftTri => draw_triangle(tl, bl(), br(), BROWN),
            VertexPattern::TopRightTri => draw_triangle(tl, tr(), br(), BROWN),
            VertexPattern::BottomRightTri => draw_triangle(tr(), bl(), br(), BROWN),
        }
    });
}
