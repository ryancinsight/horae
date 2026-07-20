/// Compile-time Butcher tableau for an explicit Runge--Kutta method.
///
/// `STAGES` is structural: coefficient arrays and workspace storage have the
/// same compile-time stage count. Coefficients are metadata converted once to
/// the selected Eunomia scalar at the operation boundary.
pub trait ExplicitTableau<const STAGES: usize>: private::Sealed {
    /// Formal order of the method.
    const ORDER: usize;
    /// Strictly lower-triangular stage coefficients.
    const A: [[f64; STAGES]; STAGES];
    /// Output weights.
    const B: [f64; STAGES];
    /// Stage-time fractions.
    const C: [f64; STAGES];
}

pub(crate) mod private {
    pub trait Sealed {}
}
