use super::{SECONDS_PER_DAY, UtcTimestamp};
use crate::CivilDate;
use core::fmt;

impl fmt::Display for UtcTimestamp {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (days, seconds) = (
            self.unix_seconds.div_euclid(SECONDS_PER_DAY),
            self.unix_seconds.rem_euclid(SECONDS_PER_DAY),
        );
        let date = CivilDate::from_unix_days(days).map_err(|_| fmt::Error)?;
        let hour = seconds / 3600;
        let minute = seconds / 60 % 60;
        let second = seconds % 60 + i64::from(self.is_leap_second());
        if formatter.alternate() {
            return write!(formatter, "{date} {hour:02}:{minute:02}:{second:02} UTC");
        }
        write!(formatter, "{date}T{hour:02}:{minute:02}:{second:02}")?;

        // Preserve the precision convention of existing reports: print zero,
        // three, six, or nine digits without changing the represented value.
        let nanos = self.subsec_nanoseconds();
        if nanos != 0 {
            if nanos.is_multiple_of(1_000_000) {
                write!(formatter, ".{:03}", nanos / 1_000_000)?;
            } else if nanos.is_multiple_of(1_000) {
                write!(formatter, ".{:06}", nanos / 1_000)?;
            } else {
                write!(formatter, ".{nanos:09}")?;
            }
        }
        formatter.write_str("Z")
    }
}
