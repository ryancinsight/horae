use super::{ExplicitTableau, model::private::Sealed};

/// Forward Euler tableau marker.
#[derive(Clone, Copy, Debug, Default)]
pub struct Euler;

impl Sealed for Euler {}

impl ExplicitTableau<1> for Euler {
    const ORDER: usize = 1;
    const A: [[f64; 1]; 1] = [[0.0]];
    const B: [f64; 1] = [1.0];
    const C: [f64; 1] = [0.0];
}

/// Explicit midpoint tableau marker.
#[derive(Clone, Copy, Debug, Default)]
pub struct Midpoint;

impl Sealed for Midpoint {}

impl ExplicitTableau<2> for Midpoint {
    const ORDER: usize = 2;
    const A: [[f64; 2]; 2] = [[0.0, 0.0], [0.5, 0.0]];
    const B: [f64; 2] = [0.0, 1.0];
    const C: [f64; 2] = [0.0, 0.5];
}

/// Classical four-stage, fourth-order Runge--Kutta tableau marker.
#[derive(Clone, Copy, Debug, Default)]
pub struct Rk4;

impl Sealed for Rk4 {}

impl ExplicitTableau<4> for Rk4 {
    const ORDER: usize = 4;
    const A: [[f64; 4]; 4] = [
        [0.0, 0.0, 0.0, 0.0],
        [0.5, 0.0, 0.0, 0.0],
        [0.0, 0.5, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
    ];
    const B: [f64; 4] = [1.0 / 6.0, 1.0 / 3.0, 1.0 / 3.0, 1.0 / 6.0];
    const C: [f64; 4] = [0.0, 0.5, 0.5, 1.0];
}
