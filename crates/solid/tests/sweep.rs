use tilebound::ctx::brk::Break;
use tilebound_solid::prelude::*;

type Sc = ConSc<16>;

#[test]
fn cant_sweep_by_zero() {
    let ctx = Context::<Sc, _>::new(ArrayMap::<4, 4, 16, _>([false; 16]), AxisMask::NONE);
    let mut payload = BoundBox::new((0.0, 0.0), (Sc::SCALE, Sc::SCALE), AxisMask::NONE);

    let result = ctx.sweep_by::<AxisX, _>(&mut payload, 0.0);

    assert_eq!(result, Break::None);
    assert_eq!(payload.pos(), (0.0, 0.0).into());
}

#[test]
fn cant_sweep_to_intersected() {
    let ctx = Context::<Sc, _>::new(ArrayMap::<4, 4, 16, _>([false; 16]), AxisMask::NONE);
    let mut payload = BoundBox::new((0.0, 0.0), (Sc::SCALE, Sc::SCALE), AxisMask::NONE);

    let result = ctx.sweep_to::<AxisX, _>(&mut payload, 8.0);

    assert_eq!(result, Break::None);
    assert_eq!(payload.pos(), (0.0, 0.0).into());
}

#[test]
fn can_sweep_to_inner_map_bound() {
    let ctx = Context::<Sc, _>::new(ArrayMap::<4, 4, 16, _>([false; 16]), AxisMask::ALL);
    let mut payload_l = BoundBox::new((16.0, 0.0), (Sc::SCALE, Sc::SCALE), AxisMask::NONE);
    let mut payload_r = payload_l.clone();

    let l_bound_result = ctx.sweep_to::<AxisX, _>(&mut payload_l, -16.0);
    let r_bound_result = ctx.sweep_to::<AxisX, _>(&mut payload_r, 76.0);

    assert_eq!(l_bound_result, Break::Collision(Collision::MapBound));
    assert_eq!(r_bound_result, Break::Collision(Collision::MapBound));
    assert_eq!(payload_l.pos(), (0.0, 0.0).into());
    assert_eq!(payload_r.pos(), (48.0, 0.0).into());
}

#[test]
fn can_sweep_to_outer_map_bound() {
    let ctx = Context::<Sc, _>::new(ArrayMap::<4, 4, 16, _>([false; 16]), AxisMask::ALL);
    let mut payload_l = BoundBox::new((-32.0, 0.0), (16.0, 16.0), AxisMask::NONE);
    let mut payload_r = BoundBox::new((76.0, 0.0), (16.0, 16.0), AxisMask::NONE);

    let l_bound_result = ctx.sweep_to::<AxisX, _>(&mut payload_l, 0.0);
    let r_bound_result = ctx.sweep_to::<AxisX, _>(&mut payload_r, 64.0);

    assert_eq!(l_bound_result, Break::ReachedTarget);
    assert_eq!(r_bound_result, Break::ReachedTarget);
    assert_eq!(payload_l.pos(), (-16.0, 0.0).into());
    assert_eq!(payload_r.pos(), (64.0, 0.0).into());
}

#[test]
fn can_sweep_out_of_bound() {
    let ctx = Context::<Sc, _>::new(ArrayMap::<4, 4, 16, _>([false; 16]), AxisMask::ALL);
    let mut payload_l = BoundBox::new((-16.0, 0.0), (16.0, 16.0), AxisMask::NONE);
    let mut payload_r = BoundBox::new((64.0, 0.0), (16.0, 16.0), AxisMask::NONE);

    let l_bound_result = ctx.sweep_to::<AxisX, _>(&mut payload_l, -128.0);
    let r_bound_result = ctx.sweep_to::<AxisX, _>(&mut payload_r, 128.0);

    assert_eq!(l_bound_result, Break::ReachedTarget);
    assert_eq!(r_bound_result, Break::ReachedTarget);
    assert_eq!(payload_l.pos(), (-128.0, 0.0).into());
    assert_eq!(payload_r.pos(), (112.0, 0.0).into());
}

/// Assert that [`Context::sweep_by`] and [`Context::sweep_to`] produce equivalent results.
#[test]
fn sweep_to_and_sweep_by_are_equivalent() {
    let ctx = Context::<Sc, _>::new(ArrayMap::<4, 4, 16, _>([false; 16]), AxisMask::NONE);
    let payload = BoundBox::new((0.0, 0.0), (16.0, 16.0), AxisMask::NONE);
    let mut payload_to_neg = payload.clone();
    let mut payload_by_neg = payload.clone();
    let mut payload_to_pos = payload.clone();
    let mut payload_by_pos = payload.clone();

    let to_16_neg = ctx.sweep_to::<AxisX, _>(&mut payload_to_neg, -16.0);
    let by_16_neg = ctx.sweep_by::<AxisX, _>(&mut payload_by_neg, -16.0);
    let to_32_pos = ctx.sweep_to::<AxisX, _>(&mut payload_to_pos, 32.0);
    let by_16_pos = ctx.sweep_by::<AxisX, _>(&mut payload_by_pos, 16.0);

    assert_eq!(payload_to_neg, payload_by_neg);
    assert_eq!(payload_to_pos, payload_by_pos);
    assert_eq!(to_16_neg, by_16_neg);
    assert_eq!(to_32_pos, by_16_pos);
    assert_eq!(payload_to_neg.pos(), (-16.0, 0.0).into());
    assert_eq!(payload_to_pos.pos(), (16.0, 0.0).into());
}
