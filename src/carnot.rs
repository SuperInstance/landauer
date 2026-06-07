//! Carnot heat engine and Szilard information-to-work conversion.
//!
//! A [`CarnotEngine`] operates between hot and cold reservoirs at maximum
//! theoretical efficiency η = 1 − T_cold/T_hot. The [`SzilardEngine`]
//! demonstrates that one bit of information can be converted to kT·ln 2
//! joules of work — the exact Landauer bound in reverse.

use crate::{BOLTZMANN, LN2};
use serde::{Deserialize, Serialize};

/// An ideal Carnot heat engine between two thermal reservoirs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarnotEngine {
    /// Hot reservoir temperature (K).
    pub t_hot: f64,
    /// Cold reservoir temperature (K).
    pub t_cold: f64,
}

impl CarnotEngine {
    /// Create a new Carnot engine. Asserts t_hot > t_cold > 0.
    pub fn new(t_hot: f64, t_cold: f64) -> Self {
        assert!(t_hot > t_cold, "t_hot must exceed t_cold");
        assert!(t_cold > 0.0, "temperatures must be positive");
        CarnotEngine { t_hot, t_cold }
    }

    /// Carnot efficiency: η = 1 − T_cold / T_hot.
    pub fn efficiency(&self) -> f64 {
        1.0 - self.t_cold / self.t_hot
    }

    /// Maximum work extractable from `q_hot` joules of heat absorbed
    /// from the hot reservoir.
    pub fn work_from_heat(&self, q_hot: f64) -> f64 {
        q_hot * self.efficiency()
    }

    /// Heat rejected to the cold reservoir when `q_hot` is absorbed.
    pub fn heat_rejected(&self, q_hot: f64) -> f64 {
        q_hot - self.work_from_heat(q_hot)
    }

    /// Heat that must be absorbed to produce `work` joules of output.
    pub fn heat_for_work(&self, work: f64) -> f64 {
        let eta = self.efficiency();
        if eta <= 0.0 {
            return f64::INFINITY;
        }
        work / eta
    }

    /// Entropy change of the universe for one cycle with `q_hot` absorbed:
    /// ΔS = −Q_hot/T_hot + Q_cold/T_cold ≥ 0 (Carnot equality).
    pub fn entropy_change(&self, q_hot: f64) -> f64 {
        let w = self.work_from_heat(q_hot);
        let q_cold = q_hot - w;
        -q_hot / self.t_hot + q_cold / self.t_cold
    }
}

/// Szilard engine: converts information into work.
///
/// In Szilard's thought experiment (1929), knowing the position of a
/// single molecule in a box allows extracting kT·ln 2 joules of work.
/// This is the information-theoretic dual of Landauer's principle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SzilardEngine {
    /// Operating temperature (K).
    pub temperature: f64,
    /// Boltzmann constant (J/K).
    pub k: f64,
}

impl SzilardEngine {
    /// Create a Szilard engine at `temperature`.
    pub fn new(temperature: f64) -> Self {
        SzilardEngine {
            temperature,
            k: BOLTZMANN,
        }
    }

    /// Work extractable from knowing the state of one bit of information.
    pub fn work_per_bit(&self) -> f64 {
        self.k * self.temperature * LN2
    }

    /// Work extractable from `n` bits of information.
    /// Iterative accumulation.
    pub fn work_from_bits(&self, n: usize) -> f64 {
        self.work_per_bit() * (n as f64)
    }

    /// Information-to-work conversion efficiency relative to a Carnot engine
    /// between `t_hot` and `t_cold`. The Szilard engine operates at the
    /// temperature of the cold reservoir (information is "free" energy).
    pub fn conversion_efficiency(&self, t_hot: f64) -> f64 {
        if t_hot <= self.temperature {
            return 0.0;
        }
        let carnot_eff = 1.0 - self.temperature / t_hot;
        // The Szilard engine extracts kT·ln2 per bit at temperature T.
        // Relative to Carnot, this is the ratio of information work to
        // the maximum possible work.
        (self.work_per_bit()) / (BOLTZMANN * t_hot * LN2 * carnot_eff)
    }

    /// How many bits of information are needed to produce `target_work` joules?
    pub fn bits_for_work(&self, target_work: f64) -> usize {
        let per_bit = self.work_per_bit();
        if per_bit <= 0.0 {
            return 0;
        }
        (target_work / per_bit).ceil() as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn carnot_efficiency() {
        let engine = CarnotEngine::new(600.0, 300.0);
        assert!((engine.efficiency() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn carnot_work_from_heat() {
        let engine = CarnotEngine::new(600.0, 300.0);
        let w = engine.work_from_heat(100.0);
        assert!((w - 50.0).abs() < 1e-10);
    }

    #[test]
    fn carnot_heat_rejected() {
        let engine = CarnotEngine::new(600.0, 300.0);
        let q_cold = engine.heat_rejected(100.0);
        assert!((q_cold - 50.0).abs() < 1e-10);
    }

    #[test]
    fn carnot_entropy_change_is_zero() {
        // Carnot cycle: reversible, so ΔS_universe = 0
        let engine = CarnotEngine::new(600.0, 300.0);
        let ds = engine.entropy_change(100.0);
        assert!(ds.abs() < 1e-10);
    }

    #[test]
    fn carnot_heat_for_work() {
        let engine = CarnotEngine::new(600.0, 300.0);
        let q = engine.heat_for_work(50.0);
        assert!((q - 100.0).abs() < 1e-10);
    }

    #[test]
    fn szilard_work_per_bit() {
        let engine = SzilardEngine::new(300.0);
        let expected = BOLTZMANN * 300.0 * LN2;
        assert!((engine.work_per_bit() - expected).abs() < 1e-30);
    }

    #[test]
    fn szilard_work_from_bits() {
        let engine = SzilardEngine::new(300.0);
        let w10 = engine.work_from_bits(10);
        let w1 = engine.work_per_bit();
        assert!((w10 - 10.0 * w1).abs() < 1e-35);
    }

    #[test]
    fn szilard_bits_for_work() {
        let engine = SzilardEngine::new(300.0);
        let w = engine.work_per_bit();
        let bits = engine.bits_for_work(w);
        assert_eq!(bits, 1);
    }

    #[test]
    #[should_panic]
    fn carnot_panics_on_invalid_temps() {
        CarnotEngine::new(300.0, 300.0);
    }
}
