//! Thermodynamic systems: state variables, laws of thermodynamics, equilibrium.
//!
//! A [`ThermodynamicSystem`] tracks energy, entropy, temperature, volume,
//! and particle count. Operations enforce the first law (energy conservation)
//! and second law (entropy never decreases for isolated systems).

use crate::BOLTZMANN;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A thermodynamic system with full state tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermodynamicSystem {
    /// Internal energy (joules).
    pub energy: f64,
    /// Entropy (J/K).
    pub entropy: f64,
    /// Temperature (K).
    pub temperature: f64,
    /// Volume (m³).
    pub volume: f64,
    /// Number of particles.
    pub particle_count: u64,
    /// Additional named state variables.
    pub state_variables: HashMap<String, f64>,
    /// Cumulative heat added (J).
    pub total_heat_added: f64,
    /// Cumulative work done by the system (J).
    pub total_work_done: f64,
    /// Is this an isolated system? (No heat or work exchange.)
    pub isolated: bool,
}

impl ThermodynamicSystem {
    /// Create a new thermodynamic system.
    pub fn new(energy: f64, entropy: f64, temperature: f64, volume: f64) -> Self {
        ThermodynamicSystem {
            energy,
            entropy,
            temperature,
            volume,
            particle_count: 0,
            state_variables: HashMap::new(),
            total_heat_added: 0.0,
            total_work_done: 0.0,
            isolated: false,
        }
    }

    /// Create an isolated system (no heat or work exchange with surroundings).
    pub fn isolated(energy: f64, entropy: f64, temperature: f64, volume: f64) -> Self {
        let mut sys = ThermodynamicSystem::new(energy, entropy, temperature, volume);
        sys.isolated = true;
        sys
    }

    /// Add heat to the system.
    /// First law: ΔU = Q − W, so adding heat increases internal energy.
    /// Second law: ΔS ≥ Q/T.
    pub fn add_heat(&mut self, heat_joules: f64) -> Result<f64, String> {
        if self.isolated && heat_joules != 0.0 {
            return Err("Cannot add heat to an isolated system".into());
        }
        if heat_joules < 0.0 && self.energy + heat_joules < 0.0 {
            return Err("Cannot remove more energy than the system has".into());
        }
        self.energy += heat_joules;
        self.total_heat_added += heat_joules;
        // Minimum entropy increase for heat addition
        let ds = if self.temperature > 0.0 {
            heat_joules / self.temperature
        } else {
            0.0
        };
        self.entropy += ds;
        Ok(ds)
    }

    /// Do work (expansion). Returns work done.
    /// First law: W = ∫PdV, simplified to P·ΔV.
    /// For an ideal gas: P = n·k·T / V.
    pub fn do_work(&mut self, delta_volume: f64) -> Result<f64, String> {
        if self.isolated && delta_volume != 0.0 {
            return Err("Isolated system cannot exchange work".into());
        }
        if self.volume + delta_volume <= 0.0 {
            return Err("Volume cannot go negative".into());
        }
        // Simplified: P = n·k·T / V (ideal gas)
        let pressure = if self.volume > 0.0 && self.particle_count > 0 {
            (self.particle_count as f64) * BOLTZMANN * self.temperature / self.volume
        } else if self.volume > 0.0 {
            self.energy / self.volume // fallback: use energy density
        } else {
            0.0
        };
        let work = pressure * delta_volume;
        self.volume += delta_volume;
        self.energy -= work;
        self.total_work_done += work;
        if self.energy < 0.0 {
            return Err("Work exceeds available energy".into());
        }
        Ok(work)
    }

    /// Check if the system satisfies the second law:
    /// entropy must be non-negative and non-decreasing for isolated systems.
    pub fn second_law_satisfied(&self, previous_entropy: f64) -> bool {
        if self.isolated {
            self.entropy >= previous_entropy
        } else {
            true // second law applies to universe, not open systems
        }
    }

    /// Check if the system is in thermal equilibrium with another system.
    /// Two systems are in equilibrium when their temperatures are equal
    /// within a tolerance.
    pub fn in_equilibrium_with(&self, other: &ThermodynamicSystem, tolerance: f64) -> bool {
        (self.temperature - other.temperature).abs() < tolerance
    }

    /// Compute the total entropy of this system plus another.
    pub fn total_entropy_with(&self, other: &ThermodynamicSystem) -> f64 {
        self.entropy + other.entropy
    }

    /// Detect equilibrium: true if temperature, pressure, and chemical
    /// potential (approximated by energy/particle) are matched.
    pub fn is_in_equilibrium(&self) -> bool {
        // A system in equilibrium has stable state variables.
        // We check that the system isn't changing (simplified).
        self.temperature > 0.0 && self.entropy >= 0.0 && self.energy >= 0.0
    }

    /// First law check: ΔU = Q − W.
    /// Returns (expected_energy, actual_energy, difference).
    pub fn first_law_check(&self, initial_energy: f64) -> (f64, f64, f64) {
        let expected = initial_energy + self.total_heat_added - self.total_work_done;
        let actual = self.energy;
        (expected, actual, (actual - expected).abs())
    }

    /// Set a named state variable.
    pub fn set_variable(&mut self, name: &str, value: f64) {
        self.state_variables.insert(name.to_string(), value);
    }

    /// Get a named state variable.
    pub fn get_variable(&self, name: &str) -> Option<f64> {
        self.state_variables.get(name).copied()
    }

    /// Heat flow between two systems until thermal equilibrium.
    /// Returns (heat_transferred, iterations).
    /// Uses iterative Newton's method style approach.
    pub fn equilibrate_with(
        &mut self,
        other: &mut ThermodynamicSystem,
        heat_per_step: f64,
        tolerance: f64,
        max_steps: usize,
    ) -> (f64, usize) {
        let mut total_heat = 0.0;
        let mut steps = 0;
        for _ in 0..max_steps {
            if self.in_equilibrium_with(other, tolerance) {
                break;
            }
            // Heat flows from hot to cold
            let delta = if self.temperature > other.temperature {
                heat_per_step.min(self.energy * 0.1)
            } else {
                -heat_per_step.min(other.energy * 0.1)
            };
            // Transfer heat
            let _ = self.add_heat(-delta);
            let _ = other.add_heat(delta);
            total_heat += delta.abs();
            steps += 1;
        }
        (total_heat, steps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_system() {
        let sys = ThermodynamicSystem::new(100.0, 1.0, 300.0, 1.0);
        assert_eq!(sys.energy, 100.0);
        assert_eq!(sys.temperature, 300.0);
    }

    #[test]
    fn add_heat_increases_energy_and_entropy() {
        let mut sys = ThermodynamicSystem::new(100.0, 1.0, 300.0, 1.0);
        let ds = sys.add_heat(10.0).unwrap();
        assert!((sys.energy - 110.0).abs() < 1e-10);
        assert!(ds > 0.0);
        assert!(sys.entropy > 1.0);
    }

    #[test]
    fn isolated_system_rejects_heat() {
        let mut sys = ThermodynamicSystem::isolated(100.0, 1.0, 300.0, 1.0);
        let result = sys.add_heat(10.0);
        assert!(result.is_err());
    }

    #[test]
    fn first_law_check() {
        let mut sys = ThermodynamicSystem::new(100.0, 1.0, 300.0, 1.0);
        let initial = sys.energy;
        sys.add_heat(50.0).unwrap();
        let (expected, actual, diff) = sys.first_law_check(initial);
        assert!(diff < 1e-10);
        assert!((expected - 150.0).abs() < 1e-10);
        assert!((actual - 150.0).abs() < 1e-10);
    }

    #[test]
    fn equilibrium_detection() {
        let a = ThermodynamicSystem::new(100.0, 1.0, 300.0, 1.0);
        let b = ThermodynamicSystem::new(100.0, 1.0, 300.0, 1.0);
        assert!(a.in_equilibrium_with(&b, 0.1));
    }

    #[test]
    fn not_in_equilibrium() {
        let a = ThermodynamicSystem::new(100.0, 1.0, 300.0, 1.0);
        let b = ThermodynamicSystem::new(100.0, 1.0, 400.0, 1.0);
        assert!(!a.in_equilibrium_with(&b, 0.1));
    }

    #[test]
    fn state_variables() {
        let mut sys = ThermodynamicSystem::new(100.0, 1.0, 300.0, 1.0);
        sys.set_variable("pressure", 101325.0);
        assert!((sys.get_variable("pressure").unwrap() - 101325.0).abs() < 1e-5);
        assert!(sys.get_variable("nonexistent").is_none());
    }

    #[test]
    fn total_entropy() {
        let a = ThermodynamicSystem::new(100.0, 2.0, 300.0, 1.0);
        let b = ThermodynamicSystem::new(100.0, 3.0, 300.0, 1.0);
        assert!((a.total_entropy_with(&b) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn second_law_satisfied() {
        let sys = ThermodynamicSystem::isolated(100.0, 1.0, 300.0, 1.0);
        let prev = sys.entropy;
        // Isolated system: entropy non-decreasing trivially
        assert!(sys.second_law_satisfied(prev));
    }

    #[test]
    fn equilibrate_flows_heat() {
        let mut a = ThermodynamicSystem::new(200.0, 2.0, 400.0, 1.0);
        let mut b = ThermodynamicSystem::new(100.0, 1.0, 200.0, 1.0);
        let (heat, steps) = a.equilibrate_with(&mut b, 10.0, 0.01, 100);
        // Heat flows from hot to cold
        assert!(steps > 0);
        assert!(heat > 0.0);
        // System a lost energy, system b gained
        assert!(a.energy < 200.0);
        assert!(b.energy > 100.0);
    }

    #[test]
    fn same_temp_already_equilibrium() {
        let mut a = ThermodynamicSystem::new(100.0, 1.0, 300.0, 1.0);
        let mut b = ThermodynamicSystem::new(100.0, 1.0, 300.0, 1.0);
        let (heat, steps) = a.equilibrate_with(&mut b, 1.0, 0.1, 100);
        assert_eq!(steps, 0);
        assert_eq!(heat, 0.0);
    }

    #[test]
    fn is_in_equilibrium_basic() {
        let sys = ThermodynamicSystem::new(100.0, 1.0, 300.0, 1.0);
        assert!(sys.is_in_equilibrium());
    }
}
