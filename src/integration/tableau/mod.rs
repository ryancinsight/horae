//! Const-generic explicit Runge--Kutta tableaus.

mod methods;
mod model;

pub use methods::{DormandPrince, Euler, Midpoint, Rk4};
pub use model::{EmbeddedExplicitTableau, ExplicitTableau};
