//! Read the instant a timestamp represents.

use super::{NANOS_PER_SECOND, UtcTimestamp};

impl UtcTimestamp {
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
