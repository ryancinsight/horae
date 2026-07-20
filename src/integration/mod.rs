//! Fixed-step explicit integration.

mod error;
mod report;
mod stepper;
pub mod tableau;
mod workspace;

pub use error::{SliceRole, StepError, WorkspaceError};
pub use report::StepReport;
pub use stepper::step_into;
pub use workspace::StepWorkspace;
