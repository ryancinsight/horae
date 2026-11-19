use super::{NANOS_PER_SECOND, UtcTimestamp};
use crate::CivilTimeError;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

impl UtcTimestamp {
    /// Read the current UTC timestamp from the system clock.
    ///
    /// # Errors
    ///
    /// Returns an error if the clock reports a year outside 0000 through 9999.
    pub fn now() -> Result<Self, CivilTimeError> {
        SystemTime::now().try_into()
    }
}

impl TryFrom<SystemTime> for UtcTimestamp {
    type Error = CivilTimeError;

    fn try_from(time: SystemTime) -> Result<Self, Self::Error> {
        match time.duration_since(UNIX_EPOCH) {
            Ok(duration) => {
                let seconds =
                    i64::try_from(duration.as_secs()).map_err(|_| CivilTimeError::OutOfRange)?;
                Self::from_unix_parts(seconds, duration.subsec_nanos())
            }
            Err(error) => {
                let duration = error.duration();
                let seconds =
                    i64::try_from(duration.as_secs()).map_err(|_| CivilTimeError::OutOfRange)?;
                let fraction = duration.subsec_nanos();
                if fraction == 0 {
                    Self::from_unix_parts(-seconds, 0)
                } else {
                    // -(s + f) = (-s - 1) + (1 - f) for 0 < f < 1.
                    Self::from_unix_parts(-seconds - 1, NANOS_PER_SECOND - fraction)
                }
            }
        }
    }
}

impl TryFrom<UtcTimestamp> for SystemTime {
    type Error = CivilTimeError;

    fn try_from(time: UtcTimestamp) -> Result<Self, Self::Error> {
        if time.is_leap_second() {
            return Err(CivilTimeError::LeapSecondOnSystemClock);
        }
        let seconds = time.unix_seconds();
        let nanos = time.subsec_nanoseconds();
        let clock = if seconds >= 0 {
            UNIX_EPOCH.checked_add(Duration::new(seconds.unsigned_abs(), nanos))
        } else if nanos == 0 {
            UNIX_EPOCH.checked_sub(Duration::new(seconds.unsigned_abs(), 0))
        } else {
            UNIX_EPOCH.checked_sub(Duration::new(
                seconds.unsigned_abs() - 1,
                NANOS_PER_SECOND - nanos,
            ))
        };
        clock.ok_or(CivilTimeError::OutOfRange)
    }
}
