//! Reversible logic gates and circuits.
//!
//! Reversible computation (Bennett 1973) avoids information erasure, and
//! therefore avoids Landauer's energy cost. This module provides:
//!
//! - [`ReversibleGate`]: Toffoli, Fredkin, CNOT
//! - [`ReversibleCircuit`]: composition of gates with garbage-bit tracking
//! - [`Uncompute`]: the UNCOMPUTE pattern for cleaning ancilla bits

use serde::{Deserialize, Serialize};

/// A reversible logic gate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ReversibleGate {
    /// Toffoli (CCNOT): flips target if both controls are 1.
    /// Fields: (control_a, control_b, target)
    Toffoli(usize, usize, usize),
    /// Fredkin (CSWAP): swaps targets if control is 1.
    /// Fields: (control, target_a, target_b)
    Fredkin(usize, usize, usize),
    /// CNOT: flips target if control is 1.
    /// Fields: (control, target)
    CNOT(usize, usize),
    /// NOT (X gate): flips a single bit.
    /// Fields: (target)
    Not(usize),
}

impl ReversibleGate {
    /// Apply the gate to a bit vector in-place.
    /// Bits are represented as bools: true = 1, false = 0.
    pub fn apply(&self, bits: &mut [bool]) {
        match self {
            ReversibleGate::Toffoli(ca, cb, t) => {
                if *ca < bits.len() && *cb < bits.len() && *t < bits.len() && bits[*ca] && bits[*cb]
                {
                    bits[*t] = !bits[*t];
                }
            }
            ReversibleGate::Fredkin(c, ta, tb) => {
                if *c < bits.len() && *ta < bits.len() && *tb < bits.len() && bits[*c] {
                    bits.swap(*ta, *tb);
                }
            }
            ReversibleGate::CNOT(c, t) => {
                if *c < bits.len() && *t < bits.len() && bits[*c] {
                    bits[*t] = !bits[*t];
                }
            }
            ReversibleGate::Not(t) => {
                if *t < bits.len() {
                    bits[*t] = !bits[*t];
                }
            }
        }
    }

    /// Returns true — reversible gates produce zero Landauer cost.
    pub fn is_lossless(&self) -> bool {
        true
    }

    /// The inverse (self-inverse for all our gates).
    pub fn inverse(&self) -> Self {
        self.clone()
    }
}

/// A reversible circuit: an ordered sequence of gates applied to a bit register.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReversibleCircuit {
    /// The gates, in order of application.
    pub gates: Vec<ReversibleGate>,
    /// Number of bits in the circuit.
    pub width: usize,
    /// Indices of garbage output bits (ancilla that must be cleaned).
    pub garbage_bits: Vec<usize>,
}

impl ReversibleCircuit {
    /// Create a new circuit of `width` bits.
    pub fn new(width: usize) -> Self {
        ReversibleCircuit {
            gates: Vec::new(),
            width,
            garbage_bits: Vec::new(),
        }
    }

    /// Add a gate to the circuit.
    pub fn add_gate(&mut self, gate: ReversibleGate) {
        self.gates.push(gate);
    }

    /// Mark a bit index as garbage.
    pub fn mark_garbage(&mut self, index: usize) {
        if !self.garbage_bits.contains(&index) {
            self.garbage_bits.push(index);
        }
    }

    /// Number of gates.
    pub fn gate_count(&self) -> usize {
        self.gates.len()
    }

    /// Run the circuit forward on a bit vector. Returns a new vector.
    pub fn run(&self, input: &[bool]) -> Vec<bool> {
        let mut bits = input.to_vec();
        for gate in &self.gates {
            gate.apply(&mut bits);
        }
        bits
    }

    /// Run the circuit in reverse (apply gates in reverse order).
    /// Since all gates are self-inverse, this is straightforward.
    pub fn run_inverse(&self, input: &[bool]) -> Vec<bool> {
        let mut bits = input.to_vec();
        for gate in self.gates.iter().rev() {
            gate.inverse().apply(&mut bits);
        }
        bits
    }

    /// Compute the Landauer cost of this circuit: always zero for
    /// reversible computation, but we return it explicitly.
    pub fn landauer_cost(&self) -> f64 {
        0.0
    }

    /// UNCOMPUTE pattern: append the inverse of all gates except
    /// the last `keep` gates. This cleans garbage bits back to their
    /// initial state while preserving the desired outputs.
    ///
    /// Returns a new circuit with the uncompute appended.
    pub fn uncompute(&self, keep: usize) -> ReversibleCircuit {
        let mut uncomputed = self.clone();
        // Append inverse of gates[0..gates.len()-keep] in reverse order
        let n = self.gates.len().saturating_sub(keep);
        for i in (0..n).rev() {
            uncomputed.gates.push(self.gates[i].inverse());
        }
        uncomputed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cnot_basic() {
        let gate = ReversibleGate::CNOT(0, 1);
        let mut bits = [true, false];
        gate.apply(&mut bits);
        assert_eq!(bits, [true, true]);
    }

    #[test]
    fn cnot_no_flip_when_control_zero() {
        let gate = ReversibleGate::CNOT(0, 1);
        let mut bits = [false, false];
        gate.apply(&mut bits);
        assert_eq!(bits, [false, false]);
    }

    #[test]
    fn toffoli_basic() {
        let gate = ReversibleGate::Toffoli(0, 1, 2);
        let mut bits = [true, true, false];
        gate.apply(&mut bits);
        assert_eq!(bits, [true, true, true]);
    }

    #[test]
    fn toffoli_no_flip() {
        let gate = ReversibleGate::Toffoli(0, 1, 2);
        let mut bits = [true, false, false];
        gate.apply(&mut bits);
        assert_eq!(bits, [true, false, false]);
    }

    #[test]
    fn fredkin_swaps_when_control_set() {
        let gate = ReversibleGate::Fredkin(0, 1, 2);
        let mut bits = [true, true, false];
        gate.apply(&mut bits);
        assert_eq!(bits, [true, false, true]);
    }

    #[test]
    fn fredkin_no_swap_when_control_unset() {
        let gate = ReversibleGate::Fredkin(0, 1, 2);
        let mut bits = [false, true, false];
        gate.apply(&mut bits);
        assert_eq!(bits, [false, true, false]);
    }

    #[test]
    fn not_gate() {
        let gate = ReversibleGate::Not(0);
        let mut bits = [false];
        gate.apply(&mut bits);
        assert_eq!(bits, [true]);
    }

    #[test]
    fn circuit_forward_and_reverse() {
        let mut circuit = ReversibleCircuit::new(3);
        circuit.add_gate(ReversibleGate::CNOT(0, 1));
        circuit.add_gate(ReversibleGate::Toffoli(0, 1, 2));
        let input = [true, false, false];
        let output = circuit.run(&input);
        let recovered = circuit.run_inverse(&output);
        assert_eq!(recovered, input);
    }

    #[test]
    fn circuit_landauer_cost_zero() {
        let mut circuit = ReversibleCircuit::new(4);
        circuit.add_gate(ReversibleGate::CNOT(0, 1));
        circuit.add_gate(ReversibleGate::Fredkin(2, 0, 1));
        assert_eq!(circuit.landauer_cost(), 0.0);
    }

    #[test]
    fn uncompute_pattern() {
        let mut circuit = ReversibleCircuit::new(2);
        circuit.add_gate(ReversibleGate::CNOT(0, 1));
        circuit.add_gate(ReversibleGate::Not(0));
        let uncomputed = circuit.uncompute(0);
        // Should have original gates + their inverses
        assert_eq!(uncomputed.gate_count(), 4);
        // Running forward then inverse should return to original
        let input = [true, false];
        let _mid = circuit.run(&input);
        let final_bits = uncomputed.run(&input);
        // After full uncompute, should return to input (since all gates are self-inverse)
        assert_eq!(final_bits, input);
    }

    #[test]
    fn gates_are_lossless() {
        assert!(ReversibleGate::CNOT(0, 1).is_lossless());
        assert!(ReversibleGate::Toffoli(0, 1, 2).is_lossless());
        assert!(ReversibleGate::Fredkin(0, 1, 2).is_lossless());
        assert!(ReversibleGate::Not(0).is_lossless());
    }

    #[test]
    fn garbage_bits_tracking() {
        let mut circuit = ReversibleCircuit::new(4);
        circuit.mark_garbage(2);
        circuit.mark_garbage(3);
        circuit.mark_garbage(2); // duplicate
        assert_eq!(circuit.garbage_bits.len(), 2);
    }
}
