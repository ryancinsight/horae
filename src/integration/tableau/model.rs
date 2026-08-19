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

/// Embedded lower-order weights sharing an explicit Runge--Kutta tableau.
///
/// [`ExplicitTableau::B`] is the primary result and
/// [`Self::B_EMBEDDED`] produces the error-estimation result from the same
/// stage derivatives. The difference between the two results is the local
/// error estimate supplied by [`crate::integration::step_embedded_into`].
pub trait EmbeddedExplicitTableau<const STAGES: usize>: ExplicitTableau<STAGES> {
    /// Formal order of the embedded result.
    const EMBEDDED_ORDER: usize;
    /// Output weights for the lower-order embedded result.
    const B_EMBEDDED: [f64; STAGES];
}

pub(crate) mod private {
    pub trait Sealed {}
}
