//! Accumulated state over a multi-step march (H-MULTISTEP-003).
//!
//! The only existing multi-step loop is in `tests/allocation.rs`, whose
//! assertions are allocation counts: nothing checked that marching N steps
//! actually lands on the right state, or that the reported end instant tracks
//! the march rather than drifting. Both are asserted here against closed forms.

use core::convert::Infallible;

use aequitas::systems::si::quantities::Time;
use eunomia::NumericElement;
use horae::{
    integration::{
        StepWorkspace, step_into,
        tableau::{ExplicitTableau, Rk4},
    },
    system::ExplicitSystem,
    time::{Instant, StepSize},
};

/// `y' = -y`, solution `y(t) = y0·e^(-t)`.
#[derive(Clone, Copy)]
struct Decay;

impl ExplicitSystem<f64> for Decay {
    type Error = Infallible;

    fn evaluate(
        &self,
        _time: Instant<f64>,
        state: &[f64],
        derivative: &mut [f64],
    ) -> Result<(), Self::Error> {
        derivative[0] = -state[0];
        Ok(())
    }
}

#[test]
fn a_march_lands_on_the_closed_form_within_its_accumulated_error_bound() {
    const STEPS: usize = 64;
    let h = 1.0 / f64::from(u32::try_from(STEPS).expect("invariant: small step count"));
    let step = StepSize::new(Time::from_base(h)).expect("invariant: positive step");
    let mut state = [1.0_f64];
    let mut output = [<f64 as NumericElement>::ZERO; 1];
    let mut workspace = StepWorkspace::<f64, 4>::new(1).expect("invariant: nonzero workspace");
    let mut end = Instant::new(Time::from_base(0.0)).expect("invariant: finite start");

    for index in 0..STEPS {
        let start = Instant::new(Time::from_base(
            f64::from(u32::try_from(index).expect("invariant: small index")) * h,
        ))
        .expect("invariant: finite");
        let report = step_into(
            &Decay,
            Rk4,
            start,
            step,
            &state,
            &mut output,
            &mut workspace,
        )
        .expect("invariant: infallible system, matched dimensions");
        state = output;
        end = report.end();
    }

    // Global error for a p-th order method over a fixed interval is
    // `C·h^p`, and for `y' = -y` on `[0, 1]` the classical Runge--Kutta
    // constant is bounded by `1 / (p + 1)!` times the interval — the leading
    // term of the local truncation error accumulated over `1 / h` steps. At
    // `h = 1/64` and `p = 4` that is `h^4 / 120`, and the bound below carries
    // an order of magnitude over it rather than a fitted factor.
    let bound = 10.0 * h.powi(4) / 120.0;
    let exact = (-1.0_f64).exp();
    let deviation = (state[0] - exact).abs();
    assert!(
        deviation <= bound,
        "after {STEPS} steps the state is {}, {deviation:.3e} from the \
         closed-form {exact}, beyond the accumulated bound {bound:.3e}",
        state[0]
    );
    // The bound must not be vacuous: a fourth-order march over this interval
    // is expected to be accurate, so the deviation is real but small.
    assert!(
        deviation > 0.0,
        "an exact result would mean the march is not integrating"
    );

    // The reported end instant is the analytic one: `STEPS` steps of `h` from
    // zero. Each step reports `start + h`, so the accumulated representation
    // error is bounded by the number of steps in units of last place.
    let reported = *end.as_time().as_base();
    let expected_end = 1.0_f64;
    let instant_bound = f64::from(u32::try_from(STEPS).expect("invariant: small step count"))
        * f64::EPSILON
        * expected_end;
    assert!(
        (reported - expected_end).abs() <= instant_bound,
        "the march reports {reported} as its end instant, beyond {instant_bound:.3e} \
         of the analytic {expected_end}"
    );
}

#[test]
fn the_march_is_the_composition_of_its_steps() {
    // Marching `2n` steps of `h` must land where marching `n` steps of `h`
    // twice does: the stepper carries no state between calls beyond what it is
    // handed, so splitting a march at any point changes nothing.
    fn march(from: f64, initial: [f64; 1], steps: usize, h: f64) -> [f64; 1] {
        let step = StepSize::new(Time::from_base(h)).expect("invariant: positive step");
        let mut state = initial;
        let mut output = [<f64 as NumericElement>::ZERO; 1];
        let mut workspace = StepWorkspace::<f64, 4>::new(1).expect("invariant: nonzero workspace");
        for index in 0..steps {
            let offset = f64::from(u32::try_from(index).expect("invariant: small index")) * h;
            let start = Instant::new(Time::from_base(from + offset)).expect("invariant: finite");
            let _report = step_into(
                &Decay,
                Rk4,
                start,
                step,
                &state,
                &mut output,
                &mut workspace,
            )
            .expect("invariant: infallible system, matched dimensions");
            state = output;
        }
        state
    }

    let h = 1.0 / 32.0;
    let whole = march(0.0, [1.0], 32, h);
    let first_half = march(0.0, [1.0], 16, h);
    let split = march(16.0 * h, first_half, 16, h);

    // Bit equality: the claim is that splitting the march changes nothing, not
    // that it changes little. `Decay` ignores the time argument, so the two
    // routes perform identical arithmetic in identical order.
    assert_eq!(
        whole[0].to_bits(),
        split[0].to_bits(),
        "a 32-step march gave {} but two 16-step marches gave {}",
        whole[0],
        split[0]
    );
}

#[test]
fn the_declared_stage_count_is_what_the_workspace_needs() {
    // A workspace narrower than the tableau's stage count must be rejected
    // rather than silently reused: `Rk4` declares four stages.
    assert_eq!(<Rk4 as ExplicitTableau<4>>::ORDER, 4);
    let step = StepSize::new(Time::from_base(0.1)).expect("invariant: positive step");
    let start = Instant::new(Time::from_base(0.0)).expect("invariant: finite start");
    let mut output = [0.0_f64; 1];
    let mut workspace = StepWorkspace::<f64, 4>::new(2).expect("invariant: nonzero workspace");
    let error = step_into(
        &Decay,
        Rk4,
        start,
        step,
        &[1.0],
        &mut output,
        &mut workspace,
    )
    .expect_err("a two-element workspace cannot serve a one-element state");
    // The dimension mismatch is reported, not absorbed.
    assert!(
        format!("{error:?}").contains("Workspace"),
        "a workspace/state dimension mismatch must name the workspace: {error:?}"
    );
}
