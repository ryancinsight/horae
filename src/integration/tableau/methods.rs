use super::{EmbeddedExplicitTableau, ExplicitTableau, model::private::Sealed};

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

/// Dormand--Prince fifth/fourth-order embedded tableau marker.
///
/// The fifth-order result is the primary step and the fourth-order result is
/// retained only as a local error estimate. The seven stages are shared by
/// both results, so an adaptive caller evaluates the system once per stage.
/// The coefficient set follows Dormand and Prince, “A family of embedded
/// Runge--Kutta formulae”, DOI
/// <https://doi.org/10.1016/0771-050X(80)90013-3>.
#[derive(Clone, Copy, Debug, Default)]
pub struct DormandPrince;

impl Sealed for DormandPrince {}

impl ExplicitTableau<7> for DormandPrince {
    const ORDER: usize = 5;
    const A: [[f64; 7]; 7] = [
        [0.0; 7],
        [1.0 / 5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        [3.0 / 40.0, 9.0 / 40.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        [44.0 / 45.0, -56.0 / 15.0, 32.0 / 9.0, 0.0, 0.0, 0.0, 0.0],
        [
            19372.0 / 6561.0,
            -25360.0 / 2187.0,
            64448.0 / 6561.0,
            -212.0 / 729.0,
            0.0,
            0.0,
            0.0,
        ],
        [
            9017.0 / 3168.0,
            -355.0 / 33.0,
            46732.0 / 5247.0,
            49.0 / 176.0,
            -5103.0 / 18656.0,
            0.0,
            0.0,
        ],
        [
            35.0 / 384.0,
            0.0,
            500.0 / 1113.0,
            125.0 / 192.0,
            -2187.0 / 6784.0,
            11.0 / 84.0,
            0.0,
        ],
    ];
    const B: [f64; 7] = [
        35.0 / 384.0,
        0.0,
        500.0 / 1113.0,
        125.0 / 192.0,
        -2187.0 / 6784.0,
        11.0 / 84.0,
        0.0,
    ];
    const C: [f64; 7] = [0.0, 1.0 / 5.0, 3.0 / 10.0, 4.0 / 5.0, 8.0 / 9.0, 1.0, 1.0];
}

impl EmbeddedExplicitTableau<7> for DormandPrince {
    const EMBEDDED_ORDER: usize = 4;
    const B_EMBEDDED: [f64; 7] = [
        5179.0 / 57600.0,
        0.0,
        7571.0 / 16695.0,
        393.0 / 640.0,
        -92097.0 / 339_200.0,
        187.0 / 2100.0,
        1.0 / 40.0,
    ];
}
