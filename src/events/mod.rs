//! Borrowed event-time scheduling and authoritative event-boundary clipping.

mod error;
mod report;
mod schedule;

pub use error::ScheduleError;
pub use report::EventClip;
pub use schedule::EventSchedule;
