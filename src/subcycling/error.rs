use core::fmt;

use crate::time::TimeError;

/// Invalid const-generic subcycle ratio or derived child step.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SubcycleError {
    /// A zero ratio cannot partition a parent step.
    ZeroRatio,
    /// Ratios above `u32::MAX` cannot be converted exactly to coefficient
    /// metadata by this policy.
    RatioTooLarge,
    /// The child step underflowed or otherwise became invalid.
    ChildStep(TimeError),
}

impl fmt::Display for SubcycleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroRatio => formatter.write_str("subcycle ratio must be nonzero"),
            Self::RatioTooLarge => formatter.write_str("subcycle ratio exceeds u32::MAX"),
            Self::ChildStep(error) => write!(formatter, "invalid child step: {error}"),
        }
    }
}

impl core::error::Error for SubcycleError {}
