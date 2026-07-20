//! Const-generic explicit Runge--Kutta tableaus.

mod methods;
mod model;

pub use methods::{Euler, Midpoint, Rk4};
pub use model::ExplicitTableau;
