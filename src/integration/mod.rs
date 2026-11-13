//! One-step time integration: explicit Runge--Kutta, fixed-step and embedded,
//! and implicit stepping.
//!
//! [`step_into`] advances one step with a plain tableau. [`step_embedded_into`]
//! advances one step with an embedded pair, writing the primary result and the
//! componentwise local-error estimate into the two slices of an
//! [`EmbeddedOutputs`], which is what [`crate::adaptive`] assesses.
//! [`step_implicit_into`] advances one step with an [`ImplicitMethod`] such as
//! [`BackwardEuler`] for a system that supplies its Jacobian, solving the
//! step's nonlinear equation by Newton iteration in an [`ImplicitWorkspace`].

mod error;
mod implicit;
mod report;
mod stepper;
pub mod tableau;
mod workspace;

pub use error::{SliceRole, StepError, WorkspaceError};
pub use implicit::{
    BackwardEuler, ImplicitMethod, ImplicitStepError, ImplicitWorkspace, SolveError,
    step_implicit_into,
};
pub use report::StepReport;
pub use stepper::{EmbeddedOutputs, step_embedded_into, step_into};
pub use workspace::StepWorkspace;
