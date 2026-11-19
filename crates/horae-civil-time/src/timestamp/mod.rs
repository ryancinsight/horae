use crate::{CivilTimeError, calendar};

#[cfg(feature = "std")]
mod clock;
mod format;
mod parsing;
#[cfg(feature = "serde")]
mod serde_impl;

const SECONDS_PER_DAY: i64 = 24 * 60 * 60;
const NANOS_PER_SECOND: u32 = 1_000_000_000;
const MIN_SECOND: i64 = calendar::MIN_UNIX_DAY * SECONDS_PER_DAY;
const MAX_SECOND: i64 = (calendar::MAX_UNIX_DAY + 1) * SECONDS_PER_DAY - 1;

/// A UTC instant with nanosecond precision and a four-digit Gregorian year.
///
/// The display and string conversion use RFC 3339. Alternate display uses
/// the human-readable UTC report label with second precision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UtcTimestamp {
    unix_seconds: i64,
    // A value in [10^9, 2*10^9) is the fraction within a leap second
    // directly after unix_seconds. Derived ordering then places it between
    // that ordinary second and the next Unix second.
    nanos: u32,
}

impl UtcTimestamp {
    /// Build a timestamp from Unix seconds and a nonnegative nanosecond fraction.
    ///
    /// # Errors
    ///
    /// Returns an error if the seconds lack a four-digit UTC year or the
    /// fraction is at least one full second.
    pub const fn from_unix_parts(unix_seconds: i64, nanos: u32) -> Result<Self, CivilTimeError> {
        if unix_seconds < MIN_SECOND || unix_seconds > MAX_SECOND {
            return Err(CivilTimeError::OutOfRange);
        }
        if nanos >= NANOS_PER_SECOND {
            return Err(CivilTimeError::InvalidFraction);
        }
        Ok(Self {
            unix_seconds,
            nanos,
        })
    }

    /// Return the Unix second, using the preceding second for a leap second.
    #[must_use]
    pub const fn unix_seconds(self) -> i64 {
        self.unix_seconds
    }

    /// Return the nanosecond fraction within the represented second.
    #[must_use]
    pub const fn subsec_nanoseconds(self) -> u32 {
        self.nanos % NANOS_PER_SECOND
    }

    /// Return whether this timestamp represents a leap second.
    #[must_use]
    pub const fn is_leap_second(self) -> bool {
        self.nanos >= NANOS_PER_SECOND
    }
}
