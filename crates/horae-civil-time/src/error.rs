use core::fmt;

/// A civil date or timestamp that cannot be represented.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CivilTimeError {
    /// Year, month, and day do not form a representable Gregorian date.
    InvalidDate,
    /// The timestamp has invalid syntax, clock components, or offset.
    InvalidTimestamp,
    /// The fraction is outside the timestamp's nanosecond precision.
    InvalidFraction,
    /// The date or timestamp is outside the four-digit UTC year range.
    OutOfRange,
    /// A leap second has no unique representation on the system clock.
    LeapSecondOnSystemClock,
}

impl fmt::Display for CivilTimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidDate => "invalid four-digit Gregorian date",
            Self::InvalidTimestamp => "invalid RFC 3339 timestamp",
            Self::InvalidFraction => "timestamp fraction must fit nanosecond precision",
            Self::OutOfRange => "civil time is outside the four-digit UTC year range",
            Self::LeapSecondOnSystemClock => "the system clock cannot represent a leap second",
        })
    }
}

impl core::error::Error for CivilTimeError {}
