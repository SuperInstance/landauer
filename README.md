# landauer

**Information thermodynamics: Landauer's principle and the physics of erasure.**

A Rust crate for computing the thermodynamic costs of information processing. Every byte you erase has a physical cost — `kT·ln(2)` joules per bit, as proven by Rolf Landauer in 1961. This crate makes that cost concrete and computable.

No physics engine. No external dependencies beyond `serde`. Just `f64` arithmetic and the laws of thermodynamics.

## The Core Idea

> "Information is physical." — Rolf Landauer, 1961

Erasing information is not free. When you reset a bit to zero without knowing its previous state, you destroy information. Thermodynamics demands that this destruction produces at least `kT·ln(2)` joules of heat:

```
E_min = k_B × T × ln(2)
```

Where:
- `k_B = 1.38064852 × 10⁻²³` J/K (Boltzmann constant)
- `T` = temperature in kelvin
- `ln(2) ≈ 0.6931`

At room temperature (300 K), this is approximately `2.87 × 10⁻²¹` joules per bit — tiny, but fundamental. It's the absolute thermodynamic floor. No technology, no matter how advanced, can erase a bit for less.

## Installation

```toml
[dependencies]
landauer = "0.1"
```

## Quick Start

```rust
use landauer::landauer::{LandauerBound, landauer_cost};

fn main() {
    // The cost of erasing one bit at room temperature
    let bound = LandauerBound::new(300.0);
    println!("One bit erasure: {:.3e} J", bound.bound_joules());

    // Erasing 1 gigabyte (8 × 10⁹ bits)
    let cost = landauer_cost(8_000_000_000, 300.0);
    println!("Erasing 1 GB: {:.3e} J", cost);

    // How many bits can we erase with 1 microjoule?
    let bits = bound.bits_from_energy(1e-6);
    println!("Bits from 1 μJ: {:.3e}", bits as f64);
}
```

## Module Overview

| Module | Description | Key Types |
|--------|-------------|-----------|
| [`bit`] | Classical bit state, registers, erasure | `Bit`, `BitRegister` |
| [`landauer`] | Landauer's bound computation | `LandauerBound` |
| [`memory`] | Physical memory cells, thermal stability | `MemoryCell`, `PhysicalMemory` |
| [`carnot`] | Carnot engine, Szilard information-to-work | `CarnotEngine`, `SzilardEngine` |
| [`reversible`] | Reversible logic gates (Toffoli, Fredkin, CNOT) | `ReversibleGate`, `ReversibleCircuit` |
| [`thermodynamics`] | Thermodynamic systems, laws of thermodynamics | `ThermodynamicSystem` |

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                   THERMODYNAMIC SYSTEM                   │
│                                                         │
│  ┌──────────┐    ┌──────────┐    ┌───────────────────┐  │
│  │  HOT      │    │  COLD    │    │  INFORMATION      │  │
│  │ RESERVOIR │    │ RESERVOIR│    │  PROCESSING       │  │
│  │  T_H      │    │  T_C     │    │                   │  │
│  └────┬──────┘    └─────┬────┘    │  ┌─────────────┐  │  │
│       │                 │         │  │ Bit Register│  │  │
│       │   ┌─────────┐   │         │  │ [0][1][0].. │  │  │
│       └──►│  CARNOT  │◄──┘         │  └──────┬──────┘  │  │
│           │  ENGINE  │             │         │         │  │
│           │ η=1-Tc/Th│             │    erase │         │  │
│           └─────┬────┘             │         ▼         │  │
│                 │                  │  ┌──────────────┐  │  │
│                 │ W = Q·η          │  │ Landauer     │  │  │
│                 │                  │  │ Cost = kT·ln2│  │  │
│           ┌─────▼────┐             │  └──────────────┘  │  │
│           │   WORK   │             │                   │  │
│           │ EXTRACTED│◄────────────│  ┌─────────────┐  │  │
│           └──────────┘  Szilard    │  │ Reversible  │  │  │
│                              Engine│  │ Gates (0 J) │  │  │
│                                    │  └─────────────┘  │  │
│                                    └───────────────────┘  │
│                                                         │
│  First Law:  ΔU = Q - W     (energy conservation)       │
│  Second Law: ΔS ≥ 0         (entropy non-decrease)      │
│  Landauer:   E ≥ kT·ln(2)   (cost of erasure)           │
│  Carnot:     η ≤ 1 - Tc/Th  (max efficiency)            │
│  Szilard:    W = kT·ln(2)   (work from information)     │
└─────────────────────────────────────────────────────────┘
```

## Examples

### Example 1: Computing Erasure Costs at Different Temperatures

```rust
use landauer::landauer::LandauerBound;

fn main() {
    let temperatures: Vec<(f64, &str)> = vec![
        (4.0,     "liquid helium"),
        (77.0,    "liquid nitrogen"),
        (300.0,   "room temperature"),
        (1000.0,  "hot filament"),
        (5800.0,  "surface of the Sun"),
    ];

    println!("Temperature (K)  │ Context              │ Cost per bit (J)");
    println!("─────────────────┼──────────────────────┼─────────────────");

    for (temp, context) in &temperatures {
        let bound = LandauerBound::new(*temp);
        println!(
            "{:>15.1} │ {:<20} │ {:.6e}",
            temp, context, bound.bound_joules()
        );
    }

    // Show the signal-to-thermal ratio (always ln2)
    let bound = LandauerBound::new(300.0);
    println!(
        "\nSignal-to-thermal ratio: {:.4} (always ln2)",
        bound.signal_to_thermal_ratio()
    );

    // Thermal noise floor at room temperature
    println!(
        "Thermal noise floor at 300K: {:.6e} J",
        bound.thermal_noise_threshold()
    );
}
```

### Example 2: Reversible vs Irreversible Computation

```rust
use landauer::bit::BitRegister;
use landauer::reversible::{ReversibleCircuit, ReversibleGate};
use landauer::{BOLTZMANN, LN2};

fn main() {
    // ── Irreversible computation ──
    // Erase 64 bits (a 64-bit register)
    let mut reg = BitRegister::from_bits(
        &(0..64).map(|i| if i % 3 == 0 {
            landauer::bit::Bit::One
        } else {
            landauer::bit::Bit::Zero
        }).collect::<Vec<_>>()
    );

    let irreversible_cost = reg.erase(300.0, BOLTZMANN);
    println!("Irreversible: erasing 64 bits costs {:.6e} J", irreversible_cost);

    // ── Reversible computation ──
    // Build a circuit that does useful work without erasing
    let mut circuit = ReversibleCircuit::new(4);
    circuit.add_gate(ReversibleGate::CNOT(0, 1));       // Copy (reversible)
    circuit.add_gate(ReversibleGate::Toffoli(0, 1, 2)); // AND into ancilla
    circuit.add_gate(ReversibleGate::CNOT(0, 3));       // Copy to output

    let input = [true, false, false, false];
    let output = circuit.run(&input);

    println!("Reversible circuit cost: {:.6e} J", circuit.landauer_cost());
    println!("Input:  {:?}", input);
    println!("Output: {:?}", output);

    // Reversible computation is FREE (Landauer cost = 0)
    assert_eq!(circuit.landauer_cost(), 0.0);

    // But we must eventually uncompute garbage bits
    let cleaned = circuit.uncompute(1); // Keep last gate's outputs
    println!("After uncompute: {} gates", cleaned.gate_count());
}
```

### Example 3: Szilard Engine and Information-to-Work Conversion

```rust
use landauer::carnot::{CarnotEngine, SzilardEngine};
use landauer::thermodynamics::ThermodynamicSystem;

fn main() {
    // Szilard's thought experiment: one bit of information
    // yields kT·ln(2) joules of work
    let szilard = SzilardEngine::new(300.0);
    println!("Work from 1 bit: {:.6e} J", szilard.work_per_bit());
    println!("Work from 1 byte: {:.6e} J", szilard.work_from_bits(8));
    println!("Work from 1 KB: {:.6e} J", szilard.work_from_bits(8192));

    // How many bits to produce 1 nanojoule?
    let bits = szilard.bits_for_work(1e-9);
    println!("Bits needed for 1 nJ: {:.3e}", bits as f64);

    // Compare with Carnot efficiency
    let carnot = CarnotEngine::new(600.0, 300.0);
    println!("\nCarnot engine (600K → 300K):");
    println!("  Efficiency: {:.1}%", carnot.efficiency() * 100.0);
    println!("  Work from 100 J heat: {:.1} J", carnot.work_from_heat(100.0));
    println!("  Heat for 50 J work: {:.1} J", carnot.heat_for_work(50.0));

    // Thermodynamic system: track energy and entropy
    let mut system = ThermodynamicSystem::new(1000.0, 5.0, 300.0, 0.001);
    system.set_variable("pressure", 101325.0);
    system.set_variable("enthalpy", 1005.0);

    let ds = system.add_heat(100.0).unwrap();
    println!("\nAfter adding 100 J:");
    println!("  Energy: {:.1} J", system.energy);
    println!("  Entropy change: {:.6e} J/K", ds);
    println!("  Total entropy: {:.6e} J/K", system.entropy);

    // First law verification
    let (expected, actual, error) = system.first_law_check(1000.0);
    println!("\nFirst law check: ΔU = Q - W");
    println!("  Expected: {:.1} J, Actual: {:.1} J, Error: {:.2e}", expected, actual, error);
}
```

## Theory

### Landauer's Principle

In 1961, Rolf Landauer demonstrated that any logically irreversible operation — one where the output does not uniquely determine the input — must dissipate at least `kT·ln(2)` energy per bit of information lost. This is not a limitation of any particular technology but a consequence of the second law of thermodynamics.

Formally, if a system has Ω possible microstates, its entropy is:

```
S = k_B · ln(Ω)
```

Erasing a bit reduces the number of microstates by a factor of 2, so:

```
ΔS = k_B · ln(Ω/2) - k_B · ln(Ω) = -k_B · ln(2)
```

The second law requires that the total entropy of the universe cannot decrease, so this entropy must appear elsewhere as heat:

```
Q ≥ k_B · T · ln(2)
```

### The Szilard Engine

Leó Szilard's 1929 thought experiment (refining Maxwell's demon) showed the flip side: knowing one bit of information about a thermodynamic system allows extracting exactly `kT·ln(2)` joules of useful work. The Szilard engine places a single molecule in a box with a partition. Knowing which side the molecule is on lets you extract work as it pushes the partition out.

This establishes a precise equivalence:

```
1 bit of information ⟺ kT·ln(2) joules of energy
```

### Carnot Efficiency

The Carnot engine represents the maximum theoretical efficiency of any heat engine operating between two temperatures:

```
η_Carnot = 1 - T_cold / T_hot
```

The work extractable from Q joules of heat:

```
W = Q · η = Q · (1 - T_cold / T_hot)
```

This connects information theory to thermodynamics: the Szilard engine's efficiency is bounded by the Carnot limit.

### Reversible Computation

Charles Bennett (1973) showed that computation can be performed reversibly — without erasing information and therefore without any Landauer cost. The key insights:

1. **Reversible gates** (Toffoli, Fredkin, CNOT) are logically invertible
2. **UNCOMPUTE pattern**: after producing outputs, reverse the intermediate computation to clean up garbage bits
3. **Cost is zero** for reversible operations; cost is only incurred when information is irreversibly discarded

### The Laws of Thermodynamics

**First Law** (energy conservation):

```
ΔU = Q - W
```

Where ΔU is the change in internal energy, Q is heat added to the system, and W is work done by the system.

**Second Law** (entropy non-decrease for isolated systems):

```
ΔS ≥ 0
```

For a heat transfer Q at temperature T:

```
ΔS ≥ Q / T
```

### Thermal Stability of Memory

Physical memory cells have energy barriers that prevent spontaneous bit flips. The mean time to a thermally-induced flip follows the Arrhenius relation:

```
τ = (1/ν) · exp(E_b / k_B T)
```

Where ν is the attempt frequency (~10¹² Hz for solids) and E_b is the barrier height. Higher barriers mean longer retention times but also higher write energies.

## Design Decisions

### Why `f64` everywhere?

Physics is messy. Real energies span dozens of orders of magnitude. `f64` gives us ~15 decimal digits of precision, which is more than enough for any practical computation involving thermodynamic quantities. We don't need arbitrary precision; we need correctness.

### Why iterative, not recursive?

Computation on millions of bits should not blow the stack. Every loop that processes bits or memory cells uses `for` loops, not recursion. This is a deliberate design choice for robustness.

### Why no abstract physics traits?

This crate is *concrete*. `Bit` is an enum, not a trait. `ThermodynamicSystem` is a struct with `f64` fields, not an abstraction over arbitrary state spaces. You can understand the entire implementation by reading the source. No indirection, no generics maze, no existential crisis about what a "Temperature" trait should look like.

### Why only `serde` as a dependency?

All public types derive `Serialize` and `Deserialize` because thermodynamic state should be persistable, loggable, and transmittable. Serde is the standard way to do this in Rust. Everything else uses `std`.

### Thermal noise threshold

At any temperature T, thermal fluctuations have characteristic energy kT. Any signal below this threshold is thermally indistinguishable from noise. The Landauer bound sits at `ln(2) ≈ 0.693` times this threshold — information erasure lives in the thermal noise.

## Constants

| Constant | Value | Unit |
|----------|-------|------|
| Boltzmann constant (k_B) | 1.38064852 × 10⁻²³ | J/K |
| ln(2) | 0.693147180559945... | dimensionless |
| Landauer bound at 300 K | 2.872 × 10⁻²¹ | J/bit |

## References

1. **Landauer, R.** (1961). "Irreversibility and heat generation in the computing process." *IBM Journal of Research and Development*, 5(3), 183–191. — The original paper establishing the minimum energy cost of bit erasure.

2. **Bennett, C. H.** (1982). "The thermodynamics of computation — a review." *International Journal of Theoretical Physics*, 21(12), 905–940. — Shows that reversible computation avoids Landauer's cost.

3. **Szilard, L.** (1929). "Über die Entropieverminderung in einem thermodynamischen System bei Eingriffen intelligenter Wesen." *Zeitschrift für Physik*, 53, 840–856. — The thought experiment connecting information to work extraction.

4. **Feynman, R. P.** (1996). *Feynman Lectures on Computation.* Addison-Wesley. — Accessible treatment of the physics of computation, including Landauer's principle and reversible computing.

5. **Nielsen, M. A. & Chuang, I. L.** (2010). *Quantum Computation and Quantum Information.* Cambridge University Press. — The standard reference; Chapter 12 covers entropy and information in quantum systems.

6. **Bérut, A., Arakelyan, A., Petrosyan, A., Ciliberto, S., Dillenschneider, R. & Lutz, E.** (2012). "Experimental verification of Landauer's principle linking information and thermodynamics." *Nature*, 483, 187–189. — First experimental confirmation of the Landauer bound.

## API Reference

### `bit` Module

```rust
use landauer::bit::{Bit, BitRegister};

// Create a bit
let b = Bit::One;
assert_eq!(b.flip(), Bit::Zero);
assert_eq!(b.to_u8(), 1);

// Erasure cost
let k = landauer::BOLTZMANN;
let cost = Bit::One.erase_to(Bit::Zero, 300.0, k);
assert!(cost > 0.0);

// Bit register
let mut reg = BitRegister::new(8);
reg.set(0, Bit::One);
reg.set(3, Bit::One);
assert_eq!(reg.count_ones(), 2);
let cost = reg.erase(300.0, k);
assert_eq!(reg.count_zeros(), 8);
```

### `landauer` Module

```rust
use landauer::landauer::LandauerBound;

let bound = LandauerBound::new(300.0);

// Per-bit cost
println!("{:.6e} J", bound.bound_joules());

// Batch erasure
println!("{:.6e} J", bound.erasure_cost(1024));

// Reverse: how many bits from an energy budget?
println!("{} bits", bound.bits_from_energy(1e-15));
```

### `memory` Module

```rust
use landauer::memory::{MemoryCell, PhysicalMemory};

let cell = MemoryCell::new(true, 1e-19, 1e12);
let mttf = cell.mean_time_to_flip(300.0);
println!("Mean time to flip: {:.3e} s", mttf);

let mut mem = PhysicalMemory::new(1024, 1e-19, 1e12);
mem.write(0, true, 300.0);
mem.write(511, true, 300.0);
let refresh = mem.total_refresh_cost(300.0);
println!("Refresh cost for 1 KB: {:.6e} J", refresh);
```

### `carnot` Module

```rust
use landauer::carnot::{CarnotEngine, SzilardEngine};

let engine = CarnotEngine::new(600.0, 300.0);
assert!((engine.efficiency() - 0.5).abs() < 1e-10);

let szilard = SzilardEngine::new(300.0);
println!("Work per bit: {:.6e} J", szilard.work_per_bit());
```

### `reversible` Module

```rust
use landauer::reversible::{ReversibleGate, ReversibleCircuit};

let mut circuit = ReversibleCircuit::new(3);
circuit.add_gate(ReversibleGate::CNOT(0, 1));
circuit.add_gate(ReversibleGate::Toffoli(0, 1, 2));

let input = [true, false, false];
let output = circuit.run(&input);
let recovered = circuit.run_inverse(&output);
assert_eq!(input, recovered.as_slice());
assert_eq!(circuit.landauer_cost(), 0.0);
```

### `thermodynamics` Module

```rust
use landauer::thermodynamics::ThermodynamicSystem;

let mut sys = ThermodynamicSystem::new(1000.0, 5.0, 300.0, 0.001);
sys.add_heat(100.0).unwrap();

let (expected, actual, error) = sys.first_law_check(1000.0);
assert!(error < 1e-10);

// State variables
sys.set_variable("pressure", 101325.0);
assert_eq!(sys.get_variable("pressure"), Some(101325.0));
```

## Numerical Examples

| Scenario | Bits | Temperature (K) | Cost (J) |
|----------|------|------------------|----------|
| Single bit erase | 1 | 300 | 2.87 × 10⁻²¹ |
| Byte erase | 8 | 300 | 2.30 × 10⁻²⁰ |
| 1 KB erase | 8,192 | 300 | 2.35 × 10⁻¹⁷ |
| 1 MB erase | 8,388,608 | 300 | 2.41 × 10⁻¹⁴ |
| 1 GB erase | 8.59 × 10⁹ | 300 | 2.47 × 10⁻¹¹ |
| 1 TB erase | 8.80 × 10¹² | 300 | 2.53 × 10⁻⁸ |
| Supercomputer wipe (10 PB) | 8.0 × 10¹⁶ | 300 | 2.30 × 10⁻⁴ |
| Single bit at 4 K | 1 | 4 | 3.83 × 10⁻²³ |
| Single bit at 5800 K | 1 | 5,800 | 5.55 × 10⁻²⁰ |

For comparison: a single chemical bond (C–C) is about 3.6 × 10⁻¹⁹ J. The Landauer cost of erasing one bit at room temperature is roughly **1/126** of a single chemical bond — incredibly small, yet fundamentally unavoidable.

## Physical Memory Considerations

### DRAM Refresh Energy vs Landauer Limit

Modern DRAM operates far above the Landauer limit. A typical DRAM cell refresh requires ~10⁻¹⁴ J — about **3.5 million times** the theoretical minimum. This gap represents the engineering headroom for future low-power memory technologies.

### Thermal Stability

For a memory cell with barrier height E_b = 10⁻¹⁹ J (about 0.6 eV) at room temperature:

```
E_b / kT = 10⁻¹⁹ / (1.38 × 10⁻²³ × 300) ≈ 24.15

τ = (1/10¹²) × exp(24.15) ≈ 3.0 × 10⁻² s
```

That's only ~30 ms! Real DRAM uses barriers of ~0.8–1.0 eV, giving retention times of milliseconds to seconds — hence the need for constant refreshing.

For non-volatile memory (Flash), barriers are ~3–5 eV:

```
E_b / kT ≈ 115–190
τ ≈ 10¹⁵⁰ years (effectively infinite)
```

### The Reversible Computing Pathway

The path to zero-energy computation:

1. **Current**: Irreversible, ~10⁻¹⁴ J per operation
2. **Near-term**: Reversible architectures, reducing erasures
3. **Theoretical**: Fully reversible (Bennett), 0 J per operation
4. **Caveat**: Only the *computation* is free; setup, measurement, and final erasure still cost energy

## Glossary

| Term | Definition |
|------|------------|
| **Bit** | Binary digit: 0 or 1. The fundamental unit of information. |
| **Landauer bound** | Minimum energy kT·ln(2) to irreversibly erase one bit. |
| **Entropy** | Measure of microscopic disorder; S = k·ln(Ω). |
| **Carnot efficiency** | Maximum theoretical efficiency of a heat engine: 1 - Tc/Th. |
| **Szilard engine** | Extracts kT·ln(2) work from one bit of information. |
| **Toffoli gate** | Reversible 3-bit gate (CCNOT); universal for reversible computation. |
| **Fredkin gate** | Reversible 3-bit gate (CSWAP); conserves the number of 1-bits. |
| **CNOT gate** | Controlled-NOT: flips target if control is 1. |
| **UNCOMPUTE** | Running a circuit in reverse to clean ancilla (garbage) bits. |
| **Thermal stability** | Mean time for a memory cell to spontaneously flip due to thermal noise. |
| **Boltzmann constant** | k_B = 1.38 × 10⁻²³ J/K. Relates temperature to energy. |
| **Isolated system** | System that cannot exchange heat or work with surroundings. |

## Contributing

Contributions are welcome. Please ensure:

- All new types derive `Serialize` + `Deserialize`
- All computation is iterative (no recursive bit processing)
- `cargo test`, `cargo fmt`, `cargo clippy` all pass cleanly
- New modules follow the existing concrete style (f64, HashMap, no abstract traits)

## License

MIT
