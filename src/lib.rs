//! # landauer
//!
//! Information thermodynamics: Landauer's principle and the physics of erasure.
//!
//! This crate provides concrete, iterative computations for the thermodynamic
//! costs of information processing. No external physics engine required — just
//! `f64` arithmetic and the laws of thermodynamics.
//!
//! ## Modules
//!
//! - [`bit`] — Classical bit state, registers, erasure with Landauer cost
//! - [`landauer`] — Landauer's bound: minimum energy to erase a bit
//! - [`memory`] — Physical memory cells with thermal stability
//! - [`carnot`] — Carnot engines and Szilard information-to-work conversion
//! - [`reversible`] — Reversible logic gates (Toffoli, Fredkin, CNOT)
//! - [`thermodynamics`] — Thermodynamic systems, laws, equilibrium

pub mod bit;
pub mod carnot;
pub mod landauer;
pub mod memory;
pub mod reversible;
pub mod thermodynamics;

/// Boltzmann constant in J/K.
pub const BOLTZMANN: f64 = 1.38064852e-23;

/// Natural log of 2.
pub const LN2: f64 = std::f64::consts::LN_2;
