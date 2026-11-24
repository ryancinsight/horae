#![cfg_attr(not(feature = "std"), no_std)]
//! Gregorian dates and UTC timestamps for reports and calendar validation.
//!
//! Dates cover the four-digit Gregorian years. Timestamp formatting uses
//! RFC 3339, while clock access and serialization are opt-in capabilities.

mod calendar;
mod error;
mod timestamp;

pub use calendar::CivilDate;
pub use error::CivilTimeError;
pub use timestamp::UtcTimestamp;
