use tilebound::ctx::brk::Break;
use tilebound_solid::prelude::*;

type Sc = ConSc<16>;

#[test]
fn cant_sweep_by_zero() {
    let ctx = Context::<Sc, _>::new(ArrayMap::<4, 4, 16, _>([false; 16]), AxisMask::NONE);
    let mut state = State::new::<Sc>(((0.0, 0.0), (Sc::SCALE, Sc::SCALE), AxisMask::NONE));

    let result = ctx.sweep_by::<AxisX>(&mut state, 0.0);

    assert_eq!(result, Break::None);
    assert_eq!(state.pos_vec::<Sc>(), (0.0, 0.0).into());
}

#[test]
fn cant_sweep_to_intersected() {
    let ctx = Context::<Sc, _>::new(ArrayMap::<4, 4, 16, _>([false; 16]), AxisMask::NONE);
    let mut state = State::new::<Sc>(((0.0, 0.0), (Sc::SCALE, Sc::SCALE), AxisMask::NONE));

    let result = ctx.sweep_to::<AxisX>(&mut state, 8.0);

    assert_eq!(result, Break::None);
    assert_eq!(state.pos_vec::<Sc>(), (0.0, 0.0).into());
}

#[test]
fn can_sweep_to_inner_map_bound() {
    let ctx = Context::<Sc, _>::new(ArrayMap::<4, 4, 16, _>([false; 16]), AxisMask::ALL);
    let state = State::new::<Sc>(((16.0, 0.0), (Sc::SCALE, Sc::SCALE), AxisMask::NONE));
    let mut state_l = state.clone();
    let mut state_r = state.clone();

    let l_bound_result = ctx.sweep_to::<AxisX>(&mut state_l, -16.0);
    let r_bound_result = ctx.sweep_to::<AxisX>(&mut state_r, 76.0);

    assert_eq!(l_bound_result, Break::Collision(Collision::MapBound));
    assert_eq!(r_bound_result, Break::Collision(Collision::MapBound));
    assert_eq!(state_l.pos_vec::<Sc>(), (0.0, 0.0).into());
    assert_eq!(state_r.pos_vec::<Sc>(), (48.0, 0.0).into());
}

#[test]
fn can_sweep_to_outer_map_bound() {
    let ctx = Context::<Sc, _>::new(ArrayMap::<4, 4, 16, _>([false; 16]), AxisMask::ALL);
    let mut state_l = State::new::<Sc>(((-32.0, 0.0), (16.0, 16.0), AxisMask::NONE));
    let mut state_r = State::new::<Sc>(((76.0, 0.0), (16.0, 16.0), AxisMask::NONE));

    let l_bound_result = ctx.sweep_to::<AxisX>(&mut state_l, 0.0);
    let r_bound_result = ctx.sweep_to::<AxisX>(&mut state_r, 64.0);

    assert_eq!(l_bound_result, Break::ReachedTarget);
    assert_eq!(r_bound_result, Break::ReachedTarget);
    assert_eq!(state_l.pos_vec::<Sc>(), (-16.0, 0.0).into());
    assert_eq!(state_r.pos_vec::<Sc>(), (64.0, 0.0).into());
}

#[test]
fn can_sweep_out_of_bound() {
    let ctx = Context::<Sc, _>::new(ArrayMap::<4, 4, 16, _>([false; 16]), AxisMask::ALL);
    let mut state_l = State::new::<Sc>(((-16.0, 0.0), (16.0, 16.0), AxisMask::NONE));
    let mut state_r = State::new::<Sc>(((64.0, 0.0), (16.0, 16.0), AxisMask::NONE));

    let l_bound_result = ctx.sweep_to::<AxisX>(&mut state_l, -128.0);
    let r_bound_result = ctx.sweep_to::<AxisX>(&mut state_r, 128.0);

    assert_eq!(l_bound_result, Break::ReachedTarget);
    assert_eq!(r_bound_result, Break::ReachedTarget);
    assert_eq!(state_l.pos_vec::<Sc>(), (-128.0, 0.0).into());
    assert_eq!(state_r.pos_vec::<Sc>(), (112.0, 0.0).into());
}

/// Assert that [`Context::sweep_by`] and [`Context::sweep_to`] produce equivalent results.
#[test]
fn sweep_to_and_sweep_by_are_equivalent() {
    let ctx = Context::<Sc, _>::new(ArrayMap::<4, 4, 16, _>([false; 16]), AxisMask::NONE);
    let state = State::new::<Sc>(((0.0, 0.0), (16.0, 16.0), AxisMask::NONE));
    let mut state_to_neg = state.clone();
    let mut state_by_neg = state.clone();
    let mut state_to_pos = state.clone();
    let mut state_by_pos = state.clone();

    let to_16_neg = ctx.sweep_to::<AxisX>(&mut state_to_neg, -16.0);
    let by_16_neg = ctx.sweep_by::<AxisX>(&mut state_by_neg, -16.0);
    let to_32_pos = ctx.sweep_to::<AxisX>(&mut state_to_pos, 32.0);
    let by_16_pos = ctx.sweep_by::<AxisX>(&mut state_by_pos, 16.0);

    assert_eq!(state_to_neg, state_by_neg);
    assert_eq!(state_to_pos, state_by_pos);
    assert_eq!(to_16_neg, by_16_neg);
    assert_eq!(to_32_pos, by_16_pos);
    assert_eq!(state_to_neg.pos_vec::<Sc>(), (-16.0, 0.0).into());
    assert_eq!(state_to_pos.pos_vec::<Sc>(), (16.0, 0.0).into());
}
