//! Classical bit state, bit registers, and erasure operations.
//!
//! A [`Bit`] is either `Zero` or `One`. A [`BitRegister`] holds a collection
//! of bits and exposes `erase()`, `reset()`, and `measure()` operations that
//! carry concrete Landauer energy costs.

use serde::{Deserialize, Serialize};

/// Classical bit state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Bit {
    /// Logical zero.
    Zero,
    /// Logical one.
    One,
}

impl Bit {
    /// Flip the bit.
    pub fn flip(self) -> Self {
        match self {
            Bit::Zero => Bit::One,
            Bit::One => Bit::Zero,
        }
    }

    /// Convert to 0 or 1.
    pub fn to_u8(self) -> u8 {
        match self {
            Bit::Zero => 0,
            Bit::One => 1,
        }
    }

    /// Erase to a known state (default: Zero). Returns the Landauer cost
    /// in joules given temperature `t_kelvin`.
    ///
    /// If the bit was already in the target state, the cost is zero
    /// (no information was destroyed).
    pub fn erase_to(self, target: Bit, t_kelvin: f64, k: f64) -> f64 {
        if self == target {
            0.0
        } else {
            k * t_kelvin * std::f64::consts::LN_2
        }
    }
}

/// A register of classical bits with thermodynamic bookkeeping.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BitRegister {
    /// The bits in this register.
    pub bits: Vec<Bit>,
    /// Cumulative energy spent on erasures (joules).
    pub total_erasure_energy: f64,
}

impl BitRegister {
    /// Create a new register of `n` bits, all zero.
    pub fn new(n: usize) -> Self {
        BitRegister {
            bits: vec![Bit::Zero; n],
            total_erasure_energy: 0.0,
        }
    }

    /// Create a register from a slice of bits.
    pub fn from_bits(bits: &[Bit]) -> Self {
        BitRegister {
            bits: bits.to_vec(),
            total_erasure_energy: 0.0,
        }
    }

    /// Number of bits.
    pub fn len(&self) -> usize {
        self.bits.len()
    }

    /// True if the register is empty.
    pub fn is_empty(&self) -> bool {
        self.bits.is_empty()
    }

    /// Erase all bits to Zero at temperature `t_kelvin`. Returns the
    /// energy cost and accumulates it into `total_erasure_energy`.
    ///
    /// Uses iterative computation — no recursion.
    pub fn erase(&mut self, t_kelvin: f64, k: f64) -> f64 {
        let mut cost = 0.0;
        for bit in &mut self.bits {
            cost += bit.erase_to(Bit::Zero, t_kelvin, k);
            *bit = Bit::Zero;
        }
        self.total_erasure_energy += cost;
        cost
    }

    /// Reset every bit to Zero *without* tracking cost.
    /// This is a logical reset, not a thermodynamic erasure.
    pub fn reset(&mut self) {
        for bit in &mut self.bits {
            *bit = Bit::Zero;
        }
    }

    /// Measure (read) the register: returns a clone of the bit vector.
    /// Measurement here is idealised and costs no energy.
    pub fn measure(&self) -> Vec<Bit> {
        self.bits.clone()
    }

    /// Count the number of One bits.
    pub fn count_ones(&self) -> usize {
        self.bits.iter().filter(|&&b| b == Bit::One).count()
    }

    /// Count the number of Zero bits.
    pub fn count_zeros(&self) -> usize {
        self.bits.len() - self.count_ones()
    }

    /// Set bit at `index` to `value`.
    pub fn set(&mut self, index: usize, value: Bit) {
        if index < self.bits.len() {
            self.bits[index] = value;
        }
    }

    /// Flip bit at `index`.
    pub fn flip(&mut self, index: usize) {
        if index < self.bits.len() {
            self.bits[index] = self.bits[index].flip();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bit_flip() {
        assert_eq!(Bit::Zero.flip(), Bit::One);
        assert_eq!(Bit::One.flip(), Bit::Zero);
    }

    #[test]
    fn bit_to_u8() {
        assert_eq!(Bit::Zero.to_u8(), 0);
        assert_eq!(Bit::One.to_u8(), 1);
    }

    #[test]
    fn erase_to_same_state_costs_zero() {
        let k = 1.38064852e-23;
        assert_eq!(Bit::Zero.erase_to(Bit::Zero, 300.0, k), 0.0);
        assert_eq!(Bit::One.erase_to(Bit::One, 300.0, k), 0.0);
    }

    #[test]
    fn erase_to_different_state_costs_kt_ln2() {
        let k = 1.38064852e-23;
        let t = 300.0;
        let expected = k * t * std::f64::consts::LN_2;
        let cost = Bit::One.erase_to(Bit::Zero, t, k);
        assert!((cost - expected).abs() < 1e-35);
    }

    #[test]
    fn register_new() {
        let reg = BitRegister::new(8);
        assert_eq!(reg.len(), 8);
        assert_eq!(reg.count_zeros(), 8);
        assert_eq!(reg.count_ones(), 0);
    }

    #[test]
    fn register_from_bits() {
        let bits = [Bit::One, Bit::Zero, Bit::One, Bit::One];
        let reg = BitRegister::from_bits(&bits);
        assert_eq!(reg.len(), 4);
        assert_eq!(reg.count_ones(), 3);
    }

    #[test]
    fn register_erase_accumulates_cost() {
        let k = 1.38064852e-23;
        let mut reg = BitRegister::from_bits(&[Bit::One, Bit::One, Bit::Zero, Bit::One]);
        let cost = reg.erase(300.0, k);
        assert!(cost > 0.0);
        assert!((reg.total_erasure_energy - cost).abs() < 1e-45);
        assert_eq!(reg.count_zeros(), 4);
    }

    #[test]
    fn register_set_and_flip() {
        let mut reg = BitRegister::new(4);
        reg.set(2, Bit::One);
        assert_eq!(reg.bits[2], Bit::One);
        reg.flip(2);
        assert_eq!(reg.bits[2], Bit::Zero);
    }

    #[test]
    fn register_reset_no_cost() {
        let k = 1.38064852e-23;
        let mut reg = BitRegister::from_bits(&[Bit::One, Bit::One]);
        reg.erase(300.0, k);
        let before = reg.total_erasure_energy;
        reg.reset();
        assert_eq!(reg.total_erasure_energy, before);
    }
}
