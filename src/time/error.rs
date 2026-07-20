use core::fmt;

/// Failure to construct a valid simulation-time value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TimeError {
    /// A time value is NaN or infinite.
    NonFinite,
    /// A step size is zero or negative.
    NonPositiveStep,
}

impl fmt::Display for TimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFinite => formatter.write_str("simulation time must be finite"),
            Self::NonPositiveStep => formatter.write_str("step size must be greater than zero"),
        }
    }
}

impl core::error::Error for TimeError {}
