use core::fmt;

use crate::time::TimeError;

use super::AdaptiveParameter;

/// Invalid adaptive-controller configuration or observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum AdaptiveError {
    /// A named controller parameter violates its finite range.
    InvalidParameter(AdaptiveParameter),
    /// An observed absolute error or reference scale is not finite.
    NonFiniteObservation,
    /// Mixed absolute-relative tolerance overflowed the scalar range.
    NonFiniteTolerance,
    /// The const-generic method order is zero or exceeds `u32`.
    InvalidOrder,
    /// The suggested scaled step is not representable.
    SuggestedStep(TimeError),
}

impl fmt::Display for AdaptiveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidParameter(parameter) => {
                write!(formatter, "invalid adaptive parameter: {parameter:?}")
            }
            Self::NonFiniteObservation => {
                formatter.write_str("adaptive error observations must be finite")
            }
            Self::NonFiniteTolerance => {
                formatter.write_str("adaptive mixed tolerance must remain finite")
            }
            Self::InvalidOrder => {
                formatter.write_str("adaptive method order must be in 1..=u32::MAX")
            }
            Self::SuggestedStep(error) => write!(formatter, "invalid suggested step: {error}"),
        }
    }
}

impl core::error::Error for AdaptiveError {}
