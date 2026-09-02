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

`step_embedded_into` takes an [`EmbeddedOutputs`] pair, writes the fifth-order
result to its primary slice, and writes `output - embedded_output` to its
error-estimate slice:

```rust
# extern crate aequitas;
# extern crate horae;
use aequitas::systems::si::quantities::Time;
use horae::{
    integration::{step_embedded_into, tableau::DormandPrince, EmbeddedOutputs, StepWorkspace},
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
    EmbeddedOutputs::new(&mut output, &mut error_estimate),
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

## Why seven evaluations, not six

The pair satisfies the *first same as last* condition: `C[6] == 1` and the
final stage row `A[6]` equals the primary weights `B`, so stage seven is
evaluated at `t + h` with exactly the state the step returns. Its derivative is
therefore the next step's first stage, and an accepted step could carry it
forward for six evaluations instead of seven — a seventh less work per step,
where the right-hand side dominates. `dormand_prince_satisfies_the_fsal_condition`
asserts the property, so this section cannot quietly describe a tableau that no
longer has it.

Horae does not exploit it, deliberately.

`step_into` and `step_embedded_into` are pure functions of their arguments: the
same state, step and start instant produce the same output and the same
estimate, which `tests/multi_step_march.rs` asserts by splitting a march and
comparing bit-for-bit. Reuse breaks that. The saved derivative is only valid
for a step that *begins where the previous one ended and was accepted*, so a
caller holding one must be prevented from using it after a rejected step, a
changed step size, or an event that moved the state. Threading it as a plain
extra argument would make a silent wrong answer reachable from safe code, and
threading it as a typed continuation the previous step returns is a public API
change to the whole stepping surface.

That trade is worth making only against a measurement showing the seventh
evaluation is a material share of a real workload's cost, and for stiff or
expensive right-hand sides it usually is. Nothing here measures it yet, and the
adaptive loop rejects steps often enough at tight tolerances that the reuse does
not apply to every step. Until such a measurement exists, seven evaluations per
step is the honest cost, and `StepReport::evaluations()` reports it.
