# Dormand--Prince Embedded Pair

The Dormand--Prince pair uses one seven-stage explicit Runge--Kutta
recurrence to produce two approximations at the same endpoint. The primary
weights have order five; the embedded weights have order four. Their
componentwise difference is a local error estimate, so adaptive control does
not require a second right-hand-side traversal.

The coefficient set is the method reported by Dormand and Prince in
[“A family of embedded Runge--Kutta formulae”](https://doi.org/10.1016/0771-050X(80)90013-3).
Horae stores the coefficients as compile-time metadata and converts them to
the selected `T: eunomia::FloatElement` at the arithmetic boundary.

## One embedded step

`step_embedded_into` writes the fifth-order result to `output` and
`output - embedded_output` to `error_estimate`:

```rust
# extern crate aequitas;
# extern crate horae;
use aequitas::systems::si::quantities::Time;
use horae::{
    integration::{step_embedded_into, tableau::DormandPrince, StepWorkspace},
    system::ExplicitSystem,
    time::{Instant, StepSize},
};

struct FourthDegreeRate;

impl ExplicitSystem<f64> for FourthDegreeRate {
    type Error = core::convert::Infallible;

    fn evaluate(
        &self,
        time: Instant<f64>,
        _state: &[f64],
        derivative: &mut [f64],
    ) -> Result<(), Self::Error> {
        derivative[0] = (*time.as_time().as_base()).powi(4);
        Ok(())
    }
}

# fn main() -> Result<(), Box<dyn core::error::Error>> {
let start = Instant::new(Time::from_base(0.0))?;
let step = StepSize::new(Time::from_base(0.5))?;
let mut output = [0.0];
let mut error_estimate = [0.0];
let mut workspace = StepWorkspace::<f64, 7>::new(1)?;
let report = step_embedded_into(
    &FourthDegreeRate,
    DormandPrince,
    start,
    step,
    &[0.0],
    &mut output,
    &mut error_estimate,
    &mut workspace,
)?;

assert_eq!(report.evaluations(), 7);
// The fifth-order primary result integrates t^4 from 0 to 0.5 exactly in
// exact arithmetic; this bound covers stage and coefficient rounding.
let exact = 0.5_f64.powi(5) / 5.0;
assert!((output[0] - exact).abs() <= 128.0 * f64::EPSILON);
assert!(error_estimate[0].is_finite());
# Ok(())
# }
```

The caller supplies the state norm and tolerances to
[`AdaptiveController`](https://docs.rs/horae/latest/horae/adaptive/struct.AdaptiveController.html).
For a scalar state, `error_estimate[0].abs()` is an absolute observation; for
vector states, the caller chooses the norm appropriate to its equations and
units. The pair does not impose a norm or silently accept a step.
