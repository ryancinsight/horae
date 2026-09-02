//! Explicit Runge--Kutta integration, fixed-step and embedded.
//!
//! [`step_into`] advances one step with a plain tableau. [`step_embedded_into`]
//! advances one step with an embedded pair, writing the primary result and the
//! componentwise local-error estimate into the two slices of an
//! [`EmbeddedOutputs`], which is what [`crate::adaptive`] assesses.

mod error;
mod report;
mod stepper;
pub mod tableau;
mod workspace;

pub use error::{SliceRole, StepError, WorkspaceError};
pub use report::StepReport;
pub use stepper::{EmbeddedOutputs, step_embedded_into, step_into};
pub use workspace::StepWorkspace;
