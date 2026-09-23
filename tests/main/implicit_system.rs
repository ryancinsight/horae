//! The implicit-system seam: a linear system exposes its Jacobian.

use aequitas::systems::si::quantities::Time;
use horae::{
    system::{ExplicitSystem, ImplicitSystem},
    time::Instant,
};

/// `y' = A y` with `A = [[-1, 2], [-3, -4]]`; the Jacobian is `A` at every
/// state, so the written matrix is exactly `A` in row-major order.
struct Linear {
    a: [[f64; 2]; 2],
}

impl Linear {
    const fn new() -> Self {
        Self {
            a: [[-1.0, 2.0], [-3.0, -4.0]],
        }
    }
}

impl ExplicitSystem<f64> for Linear {
    type Error = core::convert::Infallible;

    fn evaluate(
        &self,
        _time: Instant<f64>,
        state: &[f64],
        derivative: &mut [f64],
    ) -> Result<(), Self::Error> {
        for (out, row) in derivative.iter_mut().zip(&self.a) {
            let mut sum = 0.0;
            for (value, coefficient) in state.iter().zip(row) {
                sum += value * coefficient;
            }
            *out = sum;
        }
        Ok(())
    }
}

impl ImplicitSystem<f64> for Linear {
    fn jacobian(
        &self,
        _time: Instant<f64>,
        state: &[f64],
        jacobian: &mut [f64],
    ) -> Result<(), Self::Error> {
        let n = state.len();
        for (i, row) in self.a.iter().enumerate() {
            for (j, coefficient) in row.iter().enumerate() {
                jacobian[i * n + j] = *coefficient;
            }
        }
        Ok(())
    }
}

#[test]
#[expect(
    clippy::float_cmp,
    reason = "the hand-multiplied result is exact integer arithmetic"
)]
fn evaluate_matches_hand_multiplication() {
    let system = Linear::new();
    let time = Instant::new(Time::from_base(0.5)).expect("invariant: finite fixture");
    let state = [1.0, 2.0];
    let mut derivative = [0.0_f64; 2];
    system
        .evaluate(time, &state, &mut derivative)
        .expect("evaluates");

    // [-1·1 + 2·2, -3·1 + -4·2] = [3, -11].
    assert_eq!(derivative, [3.0, -11.0]);
}

#[test]
#[expect(
    clippy::float_cmp,
    reason = "the Jacobian coefficients are exact binary literals"
)]
fn jacobian_is_row_major() {
    let system = Linear::new();
    let time = Instant::new(Time::from_base(0.5)).expect("invariant: finite fixture");
    let state = [1.0, 2.0];
    let mut jacobian = [0.0_f64; 4];
    system
        .jacobian(time, &state, &mut jacobian)
        .expect("Jacobian forms");

    assert_eq!(jacobian, [-1.0, 2.0, -3.0, -4.0]);
}
