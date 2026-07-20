/// Acceptance decision for an error-controlled trial step.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum StepDecision {
    /// Commit the trial state.
    Accept,
    /// Discard the trial state and retry with the suggested step.
    Reject,
}
