/// Named adaptive-controller parameter for validation diagnostics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum AdaptiveParameter {
    /// Absolute error tolerance.
    AbsoluteTolerance,
    /// Relative error tolerance.
    RelativeTolerance,
    /// Conservative scale multiplier.
    SafetyFactor,
    /// Smallest permitted step scale.
    MinimumScale,
    /// Largest permitted step scale.
    MaximumScale,
}
