//! Physical memory cells with thermal stability modelling.
//!
//! A [`MemoryCell`] represents a single memory element with energy barriers,
//! thermal stability factors, and refresh costs. A [`PhysicalMemory`] groups
//! cells and tracks aggregate thermodynamic properties.

use crate::{BOLTZMANN, LN2};
use serde::{Deserialize, Serialize};

/// A single physical memory cell.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryCell {
    /// Current stored bit: true = One, false = Zero.
    pub stored: bool,
    /// Energy level of the cell (joules).
    pub energy_level: f64,
    /// Barrier height (joules). Higher = more thermally stable.
    pub barrier_height: f64,
    /// Attempt frequency for thermal transitions (Hz).
    /// Typical solid-state: ~1e12 Hz.
    pub attempt_frequency: f64,
}

impl MemoryCell {
    /// Create a new memory cell.
    pub fn new(stored: bool, barrier_height: f64, attempt_frequency: f64) -> Self {
        MemoryCell {
            stored,
            energy_level: if stored { barrier_height } else { 0.0 },
            barrier_height,
            attempt_frequency,
        }
    }

    /// Mean time to spontaneous (thermally-induced) bit flip, in seconds.
    /// Uses the Arrhenius relation: τ = (1/ν) · exp(E_b / kT).
    ///
    /// Returns `f64::INFINITY` if the barrier is infinite or temperature is zero.
    pub fn mean_time_to_flip(&self, temperature: f64) -> f64 {
        if temperature <= 0.0 || self.barrier_height <= 0.0 {
            return f64::INFINITY;
        }
        let exponent = self.barrier_height / (BOLTZMANN * temperature);
        // Guard against overflow
        if exponent > 700.0 {
            return f64::INFINITY;
        }
        (1.0 / self.attempt_frequency) * exponent.exp()
    }

    /// Retention energy: the energy required to keep the cell stable for
    /// `duration_seconds`. Approximated as the barrier height divided by
    /// the mean flip time, times the duration.
    pub fn retention_energy(&self, temperature: f64, duration_seconds: f64) -> f64 {
        let mttf = self.mean_time_to_flip(temperature);
        if mttf.is_infinite() || mttf <= 0.0 {
            return 0.0;
        }
        (duration_seconds / mttf) * self.barrier_height
    }

    /// Refresh cost: energy to re-write the cell to reinforce the barrier.
    /// Approximately kT·ln(2) plus the barrier energy scaled by a factor.
    pub fn refresh_cost(&self, temperature: f64) -> f64 {
        BOLTZMANN * temperature * LN2 + self.barrier_height * 0.01
    }

    /// Erase the cell to Zero and return the Landauer cost.
    pub fn erase(&mut self, temperature: f64) -> f64 {
        if self.stored {
            self.stored = false;
            self.energy_level = 0.0;
            BOLTZMANN * temperature * LN2
        } else {
            0.0
        }
    }

    /// Write a value to the cell.
    pub fn write(&mut self, value: bool, temperature: f64) -> f64 {
        let cost = if self.stored != value {
            BOLTZMANN * temperature * LN2
        } else {
            0.0
        };
        self.stored = value;
        self.energy_level = if value { self.barrier_height } else { 0.0 };
        cost
    }
}

/// A collection of physical memory cells.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalMemory {
    /// The memory cells.
    pub cells: Vec<MemoryCell>,
    /// Total energy spent on operations (joules).
    pub total_energy_spent: f64,
}

impl PhysicalMemory {
    /// Create a new physical memory with `n` cells, all Zero.
    pub fn new(n: usize, barrier_height: f64, attempt_frequency: f64) -> Self {
        let cells = (0..n)
            .map(|_| MemoryCell::new(false, barrier_height, attempt_frequency))
            .collect();
        PhysicalMemory {
            cells,
            total_energy_spent: 0.0,
        }
    }

    /// Number of cells.
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    /// True if no cells.
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    /// Erase all cells and return total cost. Iterative.
    pub fn erase_all(&mut self, temperature: f64) -> f64 {
        let mut total = 0.0;
        for cell in &mut self.cells {
            total += cell.erase(temperature);
        }
        self.total_energy_spent += total;
        total
    }

    /// Compute total refresh cost for all cells. Iterative.
    pub fn total_refresh_cost(&self, temperature: f64) -> f64 {
        let mut total = 0.0;
        for cell in &self.cells {
            total += cell.refresh_cost(temperature);
        }
        total
    }

    /// Compute total retention energy for all cells over `duration_seconds`. Iterative.
    pub fn total_retention_energy(&self, temperature: f64, duration_seconds: f64) -> f64 {
        let mut total = 0.0;
        for cell in &self.cells {
            total += cell.retention_energy(temperature, duration_seconds);
        }
        total
    }

    /// Count cells storing One.
    pub fn count_ones(&self) -> usize {
        self.cells.iter().filter(|c| c.stored).count()
    }

    /// Write a value to cell at `index`.
    pub fn write(&mut self, index: usize, value: bool, temperature: f64) {
        if index < self.cells.len() {
            let cost = self.cells[index].write(value, temperature);
            self.total_energy_spent += cost;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_new() {
        let cell = MemoryCell::new(true, 1e-19, 1e12);
        assert!(cell.stored);
        assert_eq!(cell.energy_level, 1e-19);
    }

    #[test]
    fn mean_time_increases_with_barrier() {
        let low = MemoryCell::new(true, 1e-20, 1e12);
        let high = MemoryCell::new(true, 1e-19, 1e12);
        let t = 300.0;
        assert!(high.mean_time_to_flip(t) > low.mean_time_to_flip(t));
    }

    #[test]
    fn mean_time_zero_temp_is_infinite() {
        let cell = MemoryCell::new(true, 1e-19, 1e12);
        assert!(cell.mean_time_to_flip(0.0).is_infinite());
    }

    #[test]
    fn erase_cost() {
        let mut cell = MemoryCell::new(true, 1e-19, 1e12);
        let cost = cell.erase(300.0);
        assert!(cost > 0.0);
        assert!(!cell.stored);
    }

    #[test]
    fn erase_already_zero() {
        let mut cell = MemoryCell::new(false, 1e-19, 1e12);
        let cost = cell.erase(300.0);
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn write_changes_state() {
        let mut cell = MemoryCell::new(false, 1e-19, 1e12);
        let cost = cell.write(true, 300.0);
        assert!(cost > 0.0);
        assert!(cell.stored);
    }

    #[test]
    fn physical_memory_erase_all() {
        let mut mem = PhysicalMemory::new(4, 1e-19, 1e12);
        mem.write(0, true, 300.0);
        mem.write(2, true, 300.0);
        assert_eq!(mem.count_ones(), 2);
        let cost = mem.erase_all(300.0);
        assert!(cost > 0.0);
        assert_eq!(mem.count_ones(), 0);
    }

    #[test]
    fn retention_energy_small() {
        let cell = MemoryCell::new(true, 1e-19, 1e12);
        let re = cell.retention_energy(300.0, 1.0);
        // Should be tiny for a stable cell at room temperature
        assert!(re >= 0.0);
    }

    #[test]
    fn refresh_cost_positive() {
        let cell = MemoryCell::new(true, 1e-19, 1e12);
        let rc = cell.refresh_cost(300.0);
        assert!(rc > 0.0);
    }
}
