//! Landauer's Principle: the minimum energy cost of irreversible bit erasure.
//!
//! Landauer (1961) showed that erasing one bit of information dissipates at
//! least **kT ln 2** joules of energy as heat. This module provides the
//! [`LandauerBound`] struct for computing this bound at any temperature.

use crate::{BOLTZMANN, LN2};
use serde::{Deserialize, Serialize};

/// The Landauer bound for a given temperature.
///
/// At temperature T, erasing one bit of information costs at least
/// kT·ln(2) joules. This struct pre-computes the bound and provides
/// methods to scale it to many bits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LandauerBound {
    /// Temperature in kelvin.
    pub temperature: f64,
    /// Boltzmann constant (J/K).
    pub k: f64,
}

impl LandauerBound {
    /// Create a new bound at `temperature` K using the standard Boltzmann constant.
    pub fn new(temperature: f64) -> Self {
        LandauerBound {
            temperature,
            k: BOLTZMANN,
        }
    }

    /// Create with a custom Boltzmann constant (useful for testing).
    pub fn with_k(temperature: f64, k: f64) -> Self {
        LandauerBound { temperature, k }
    }

    /// The single-bit Landauer bound in joules: kT·ln(2).
    pub fn bound_joules(&self) -> f64 {
        self.k * self.temperature * LN2
    }

    /// The single-bit bound expressed in units of kT.
    pub fn bound_kt(&self) -> f64 {
        LN2
    }

    /// Energy cost to erase `n` bits at this temperature.
    /// Iterative: O(n).
    pub fn erasure_cost(&self, n: usize) -> f64 {
        let per_bit = self.bound_joules();
        per_bit * (n as f64)
    }

    /// How many bits can be erased with `energy_joules` of available energy?
    /// Returns the floor (never exceeds the energy budget).
    pub fn bits_from_energy(&self, energy_joules: f64) -> usize {
        let per_bit = self.bound_joules();
        if per_bit <= 0.0 {
            return 0;
        }
        (energy_joules / per_bit).floor() as usize
    }

    /// Thermal noise threshold: the energy scale of thermal fluctuations
    /// at this temperature. Roughly kT. Anything below this is "drowned"
    /// in thermal noise.
    pub fn thermal_noise_threshold(&self) -> f64 {
        self.k * self.temperature
    }

    /// Compute the Landauer bound at a different temperature.
    pub fn at_temperature(&self, new_temp: f64) -> LandauerBound {
        LandauerBound {
            temperature: new_temp,
            k: self.k,
        }
    }

    /// Ratio of the bound to the thermal noise energy.
    /// Always ln(2) ≈ 0.693 for the standard bound.
    pub fn signal_to_thermal_ratio(&self) -> f64 {
        self.bound_joules() / self.thermal_noise_threshold()
    }
}

/// Compute the Landauer cost for `n` bits at `temperature` K.
/// Convenience function.
pub fn landauer_cost(n: usize, temperature: f64) -> f64 {
    LandauerBound::new(temperature).erasure_cost(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bound_at_room_temp() {
        let lb = LandauerBound::new(300.0);
        let expected = BOLTZMANN * 300.0 * LN2;
        assert!((lb.bound_joules() - expected).abs() < 1e-30);
    }

    #[test]
    fn bound_kt_is_ln2() {
        let lb = LandauerBound::new(300.0);
        assert!((lb.bound_kt() - LN2).abs() < 1e-15);
    }

    #[test]
    fn erasure_cost_scales_linearly() {
        let lb = LandauerBound::new(300.0);
        let one = lb.erasure_cost(1);
        let ten = lb.erasure_cost(10);
        assert!((ten - 10.0 * one).abs() < 1e-35);
    }

    #[test]
    fn bits_from_energy_roundtrip() {
        let lb = LandauerBound::new(300.0);
        let cost = lb.erasure_cost(100);
        let recovered = lb.bits_from_energy(cost);
        assert!(recovered <= 100);
        assert!(recovered >= 99);
    }

    #[test]
    fn thermal_noise_threshold() {
        let lb = LandauerBound::new(300.0);
        let expected = BOLTZMANN * 300.0;
        assert!((lb.thermal_noise_threshold() - expected).abs() < 1e-30);
    }

    #[test]
    fn signal_to_thermal_ratio() {
        let lb = LandauerBound::new(300.0);
        assert!((lb.signal_to_thermal_ratio() - LN2).abs() < 1e-15);
    }

    #[test]
    fn at_temperature_changes_temp() {
        let lb = LandauerBound::new(300.0);
        let cold = lb.at_temperature(4.0);
        assert!((cold.temperature - 4.0).abs() < 1e-10);
        assert!(cold.bound_joules() < lb.bound_joules());
    }

    #[test]
    fn convenience_function() {
        let cost = landauer_cost(1, 300.0);
        let expected = BOLTZMANN * 300.0 * LN2;
        assert!((cost - expected).abs() < 1e-30);
    }
}
