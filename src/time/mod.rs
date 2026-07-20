//! Typed simulation-time values.

mod error;
mod instant;
mod step_size;

pub use error::TimeError;
pub use instant::Instant;
pub use step_size::StepSize;
