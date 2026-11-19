use crate::CivilTimeError;
use core::fmt;

const MONTH_STARTS: [i64; 12] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
const YEAR_LIMIT: i64 = 10_000;
const UNIX_EPOCH_DAY: i64 = days_before_year(1970);

pub(super) const MIN_UNIX_DAY: i64 = -UNIX_EPOCH_DAY;
pub(super) const MAX_UNIX_DAY: i64 = days_before_year(YEAR_LIMIT) - UNIX_EPOCH_DAY - 1;

const fn is_leap_year(year: u16) -> bool {
    year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
}

const fn days_in_month(year: u16, month: u8) -> Option<u8> {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => Some(31),
        4 | 6 | 9 | 11 => Some(30),
        2 if is_leap_year(year) => Some(29),
        2 => Some(28),
        _ => None,
    }
}

// Counts leap years in [0, year). A cycle contributes 97 leap days per 400
// years, so this function is strictly increasing and gives a unique year
// interval for each representable absolute day.
const fn days_before_year(year: i64) -> i64 {
    365 * year + (year + 3) / 4 - (year + 99) / 100 + (year + 399) / 400
}

fn days_before_month(year: u16, month: u8) -> i64 {
    MONTH_STARTS[usize::from(month - 1)] + i64::from(month > 2 && is_leap_year(year))
}

/// A Gregorian date with a four-digit year, including year 0000.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CivilDate {
    year: u16,
    month: u8,
    day: u8,
}

impl CivilDate {
    /// Construct a validated date.
    ///
    /// # Errors
    ///
    /// Returns an error if the year has more than four digits, the month is
    /// outside 1 through 12, or the day does not exist in that month.
    pub const fn new(year: u16, month: u8, day: u8) -> Result<Self, CivilTimeError> {
        let Some(last_day) = days_in_month(year, month) else {
            return Err(CivilTimeError::InvalidDate);
        };
        if year >= 10_000 || day == 0 || day > last_day {
            return Err(CivilTimeError::InvalidDate);
        }
        Ok(Self { year, month, day })
    }

    /// Return the year, month, and day in that order.
    #[must_use]
    pub const fn components(self) -> (u16, u8, u8) {
        (self.year, self.month, self.day)
    }

    /// Return the signed day offset from 1970-01-01.
    #[must_use]
    pub fn unix_days(self) -> i64 {
        days_before_year(i64::from(self.year)) - UNIX_EPOCH_DAY
            + days_before_month(self.year, self.month)
            + i64::from(self.day)
            - 1
    }

    /// Convert the signed day offset from 1970-01-01 to a Gregorian date.
    ///
    /// # Errors
    ///
    /// Returns an error if the day lies outside years 0000 through 9999.
    pub fn from_unix_days(days: i64) -> Result<Self, CivilTimeError> {
        if !(MIN_UNIX_DAY..=MAX_UNIX_DAY).contains(&days) {
            return Err(CivilTimeError::OutOfRange);
        }
        let absolute_day = days + UNIX_EPOCH_DAY;

        let mut start = 0;
        let mut end = YEAR_LIMIT;
        while end - start > 1 {
            let midpoint = i64::midpoint(start, end);
            if days_before_year(midpoint) <= absolute_day {
                start = midpoint;
            } else {
                end = midpoint;
            }
        }

        let year = u16::try_from(start).map_err(|_| CivilTimeError::OutOfRange)?;
        let ordinal = absolute_day - days_before_year(start);
        let month = (2..=12)
            .rev()
            .find(|&month| days_before_month(year, month) <= ordinal)
            .unwrap_or(1);
        let day = u8::try_from(ordinal - days_before_month(year, month) + 1)
            .map_err(|_| CivilTimeError::OutOfRange)?;
        Self::new(year, month, day)
    }
}

impl fmt::Display for CivilDate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (year, month, day) = self.components();
        write!(formatter, "{year:04}-{month:02}-{day:02}")
    }
}
