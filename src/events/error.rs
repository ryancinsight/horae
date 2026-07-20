use core::fmt;

/// Invalid ordering in a borrowed event schedule.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ScheduleError {
    /// An event precedes its immediate predecessor.
    Unsorted {
        /// Index of the first event that violates nondecreasing order.
        index: usize,
    },
}

impl fmt::Display for ScheduleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsorted { index } => {
                write!(formatter, "event schedule is unsorted at index {index}")
            }
        }
    }
}

impl core::error::Error for ScheduleError {}
