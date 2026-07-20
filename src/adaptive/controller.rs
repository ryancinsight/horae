use eunomia::{FloatElement, NumericElement};

use crate::time::StepSize;

use super::{AdaptiveError, AdaptiveParameter, AdaptiveReport, StepDecision};

/// Mixed absolute-relative adaptive step controller.
///
/// The caller supplies an aggregate absolute error and reference-state scale;
/// Horae applies `error / (absolute + relative * scale)`. It does not own an
/// embedded estimator or a state norm.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AdaptiveController<T> {
    absolute_tolerance: T,
    relative_tolerance: T,
    safety_factor: T,
    minimum_scale: T,
    maximum_scale: T,
}

impl<T> AdaptiveController<T>
where
    T: FloatElement,
{
    /// Validate controller tolerances and scaling limits.
    ///
    /// Absolute tolerance, safety, and both scale limits must be finite and
    /// positive. Relative tolerance may be zero. Safety and minimum scale must
    /// not exceed one; maximum scale must be at least one.
    ///
    /// # Errors
    ///
    /// Returns [`AdaptiveError::InvalidParameter`] naming the first invalid
    /// input.
    pub fn new(
        absolute_tolerance: T,
        relative_tolerance: T,
        safety_factor: T,
        minimum_scale: T,
        maximum_scale: T,
    ) -> Result<Self, AdaptiveError> {
        let zero = <T as NumericElement>::ZERO;
        let one = <T as NumericElement>::ONE;
        require(
            absolute_tolerance.is_finite() && absolute_tolerance > zero,
            AdaptiveParameter::AbsoluteTolerance,
        )?;
        require(
            relative_tolerance.is_finite() && relative_tolerance >= zero,
            AdaptiveParameter::RelativeTolerance,
        )?;
        require(
            safety_factor.is_finite() && safety_factor > zero && safety_factor <= one,
            AdaptiveParameter::SafetyFactor,
        )?;
        require(
            minimum_scale.is_finite() && minimum_scale > zero && minimum_scale <= one,
            AdaptiveParameter::MinimumScale,
        )?;
        require(
            maximum_scale.is_finite() && maximum_scale >= one,
            AdaptiveParameter::MaximumScale,
        )?;
        require(
            minimum_scale <= maximum_scale,
            AdaptiveParameter::MaximumScale,
        )?;
        Ok(Self {
            absolute_tolerance,
            relative_tolerance,
            safety_factor,
            minimum_scale,
            maximum_scale,
        })
    }

    /// Assess a trial step for a method of const-generic formal `ORDER`.
    ///
    /// The suggested factor is `safety / normalized_error^(1/(ORDER + 1))`,
    /// clamped to the configured scale interval. A zero error selects the
    /// maximum scale directly.
    ///
    /// # Errors
    ///
    /// Returns [`AdaptiveError::NonFiniteObservation`] for NaN or infinite
    /// observations, [`AdaptiveError::NonFiniteTolerance`] when their mixed
    /// scale overflows, [`AdaptiveError::InvalidOrder`] for an unsupported
    /// order, or [`AdaptiveError::SuggestedStep`] when scaling cannot produce
    /// a valid positive finite step.
    pub fn assess<const ORDER: usize>(
        &self,
        step: StepSize<T>,
        absolute_error: T,
        reference_scale: T,
    ) -> Result<AdaptiveReport<T>, AdaptiveError> {
        if !absolute_error.is_finite() || !reference_scale.is_finite() {
            return Err(AdaptiveError::NonFiniteObservation);
        }
        let order = u32::try_from(ORDER).map_err(|_| AdaptiveError::InvalidOrder)?;
        if order == 0 || order == u32::MAX {
            return Err(AdaptiveError::InvalidOrder);
        }

        let error = absolute_error.abs();
        let scale_reference = reference_scale.abs();
        let tolerance = self
            .relative_tolerance
            .scalar_fmadd(scale_reference, self.absolute_tolerance);
        if !tolerance.is_finite() {
            return Err(AdaptiveError::NonFiniteTolerance);
        }
        let normalized_error = error / tolerance;
        let one = <T as NumericElement>::ONE;
        let decision = if normalized_error <= one {
            StepDecision::Accept
        } else {
            StepDecision::Reject
        };
        let scale = if normalized_error == <T as NumericElement>::ZERO {
            self.maximum_scale
        } else {
            let exponent_denominator = T::from_f64(f64::from(order + 1));
            let raw_scale = self.safety_factor / normalized_error.powf(one / exponent_denominator);
            raw_scale
                .max_scalar(self.minimum_scale)
                .min_scalar(self.maximum_scale)
        };
        let suggested_step = step.scaled(scale).map_err(AdaptiveError::SuggestedStep)?;
        Ok(AdaptiveReport::new(
            decision,
            normalized_error,
            scale,
            suggested_step,
        ))
    }
}

fn require(condition: bool, parameter: AdaptiveParameter) -> Result<(), AdaptiveError> {
    if condition {
        Ok(())
    } else {
        Err(AdaptiveError::InvalidParameter(parameter))
    }
}
