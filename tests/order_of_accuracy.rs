//! Observed order of accuracy for each sealed tableau (H-ORDER-001).
//!
//! `AdaptiveController::assess::<ORDER>` consumes the `ORDER` constant each
//! tableau declares, and nothing measured it: a wrong constant would silently
//! mis-scale every step proposal. These clauses recover the order from a
//! timestep-refinement study and compare it to the declaration.
//!
//! The oracle is derived, not tuned. With the global error obeying
//! `e(h) = C·h^p + D·h^(p+1) + …`, the two-level estimate
//! `p_k = log2(e(h_k)/e(h_k/2))` carries a defect `K·h_k + M·h_k² + …` that
//! shrinks with each refinement, so `2·p_fine − p_coarse` cancels the linear
//! term and leaves `O(h²)`. Four levels give two such extrapolants, and the
//! distance between them measures the term the second one still neglects —
//! that measured distance, plus floating-point noise propagated through the
//! same logarithms, is the tolerance. Every number in it comes from the run;
//! none is chosen to make one pass.

use core::{convert::Infallible, fmt};

use aequitas::systems::si::quantities::Time;
use eunomia::{NumericElement, RealField};
use horae::{
    integration::{
        StepWorkspace, step_into,
        tableau::{DormandPrince, Euler, ExplicitTableau, Midpoint, Rk4},
    },
    system::ExplicitSystem,
    time::{Instant, StepSize},
};

/// `y' = −y`, whose solution `y(t) = y0·e^(−t)` is smooth and exact in closed
/// form. No tableau here integrates it exactly, so each shows its own order —
/// a constant or linear-in-`t` rate would be integrated exactly by several of
/// them and measure nothing.
#[derive(Clone, Copy)]
struct Decay;

impl<T: RealField> ExplicitSystem<T> for Decay {
    type Error = Infallible;

    fn evaluate(
        &self,
        _time: Instant<T>,
        state: &[T],
        derivative: &mut [T],
    ) -> Result<(), Self::Error> {
        for (d, s) in derivative.iter_mut().zip(state) {
            *d = -*s;
        }
        Ok(())
    }
}

/// Marches `steps` uniform steps from `t = 0` to `t = 1` and returns the
/// absolute error against `e^(−1)`.
fn global_error<Method, const STAGES: usize>(method: Method, steps: usize) -> f64
where
    Method: ExplicitTableau<STAGES> + Copy,
{
    let h = 1.0 / f64::from(u32::try_from(steps).expect("invariant: step counts fit u32"));
    let step = StepSize::new(Time::from_base(h)).expect("invariant: positive step");
    let mut state = [1.0_f64];
    let mut output = [<f64 as NumericElement>::ZERO; 1];
    let mut workspace = StepWorkspace::<f64, STAGES>::new(1).expect("invariant: nonzero workspace");

    for index in 0..steps {
        let elapsed = f64::from(u32::try_from(index).expect("invariant: step index fits u32")) * h;
        let start = Instant::new(Time::from_base(elapsed)).expect("invariant: finite");
        let _report = step_into(
            &Decay,
            method,
            start,
            step,
            &state,
            &mut output,
            &mut workspace,
        )
        .expect("invariant: infallible system, matched dimensions");
        state = output;
    }

    (state[0] - (-1.0_f64).exp()).abs()
}

/// The rounding floor on one global error: `steps` steps each commit a
/// relative rounding error of order `eps` on a state of magnitude at most one.
fn rounding_floor(steps: usize) -> f64 {
    f64::from(u32::try_from(steps).expect("invariant: step counts fit u32")) * f64::EPSILON
}

/// Propagates the rounding floor through `log2(e_coarse / e_fine)`:
/// `d(log2 x) = dx / (x·ln 2)`, and the two errors contribute independently.
fn order_uncertainty(
    error_coarse: f64,
    steps_coarse: usize,
    error_fine: f64,
    steps_fine: usize,
) -> f64 {
    (rounding_floor(steps_coarse) / error_coarse + rounding_floor(steps_fine) / error_fine)
        / core::f64::consts::LN_2
}

/// Asserts the tableau's declared order against a three-level refinement.
fn assert_declared_order<Method, const STAGES: usize>(method: Method, name: &str, coarsest: usize)
where
    Method: ExplicitTableau<STAGES> + Copy + fmt::Debug,
{
    let steps = [coarsest, 2 * coarsest, 4 * coarsest, 8 * coarsest];
    let errors = steps.map(|n| global_error(method, n));

    // Truncation error must dominate rounding at every level, or the study
    // measures the floor instead of the method.
    for (error, n) in errors.iter().zip(steps) {
        assert!(
            *error > 100.0 * rounding_floor(n),
            "{name}: e({n} steps) = {error:.3e} is not clear of the rounding floor              {:.3e}; the refinement range is too fine to measure an order",
            rounding_floor(n)
        );
    }

    let pairs = [
        (errors[0] / errors[1]).log2(),
        (errors[1] / errors[2]).log2(),
        (errors[2] / errors[3]).log2(),
    ];
    // Richardson on the order estimate itself: the linear defect halves with
    // each refinement, so this cancels it and leaves O(h²).
    let coarse_extrapolant = 2.0 * pairs[1] - pairs[0];
    let extrapolated = 2.0 * pairs[2] - pairs[1];

    let declared = f64::from(u32::try_from(Method::ORDER).expect("invariant: orders are small"));
    // The distance between successive extrapolants is the size of the term the
    // finer one still neglects; the logarithms also carry the rounding floor.
    let neglected_term = (extrapolated - coarse_extrapolant).abs();
    let noise = 2.0 * order_uncertainty(errors[2], steps[2], errors[3], steps[3])
        + order_uncertainty(errors[1], steps[1], errors[2], steps[2]);
    let tolerance = neglected_term + noise;

    assert!(
        (extrapolated - declared).abs() <= tolerance,
        "{name}: declared ORDER = {declared}, but refinement recovers          {extrapolated:.5} (pairs {:.4}, {:.4}, {:.4}; errors {:.3e}, {:.3e},          {:.3e}, {:.3e}) outside the derived tolerance {tolerance:.3e}          (neglected term {neglected_term:.3e}, rounding {noise:.3e})",
        pairs[0],
        pairs[1],
        pairs[2],
        errors[0],
        errors[1],
        errors[2],
        errors[3],
    );

    // The extrapolation is only meaningful if the raw sequence is converging:
    // each pair estimate must sit closer to the declared order than the last.
    assert!(
        (pairs[2] - declared).abs() < (pairs[1] - declared).abs(),
        "{name}: the order estimate is not converging — pairs {:.4}, {:.4},          {:.4} against declared {declared}",
        pairs[0],
        pairs[1],
        pairs[2],
    );
}

#[test]
fn euler_recovers_its_declared_first_order() {
    assert_declared_order(Euler, "Euler", 64);
}

#[test]
fn midpoint_recovers_its_declared_second_order() {
    assert_declared_order(Midpoint, "Midpoint", 32);
}

#[test]
fn rk4_recovers_its_declared_fourth_order() {
    assert_declared_order(Rk4, "Rk4", 8);
}

#[test]
fn dormand_prince_recovers_its_declared_fifth_order() {
    assert_declared_order(DormandPrince, "DormandPrince", 4);
}
