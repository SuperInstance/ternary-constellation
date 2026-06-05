# ternary-constellation

**Constellation pattern for grouping related ternary crates into deployable units**

[![ternary](https://img.shields.io/badge/ecosystem-ternary-blue)](https://github.com/orgs/SuperInstance/repositories?q=ternary)
[![tests](https://img.shields.io/badge/tests-19-green)]()

## Overview

Constellation pattern for grouping related ternary crates into deployable units.

A `Constellation` is a named group of skills (crates) with declared dependencies.
`ConstellationBuilder` constructs them, `ConstellationResolver` resolves dependencies
and detects conflicts, `ConstellationCompiler` bundles them for deployment targets,
and `ConstellationRegistry` catalogs constellations across a fleet. Maps to the
"room type" concept — a Codespace room loads a specific constellation.

## Architecture

- **`SkillDescriptor`** — core data structure
- **`Constellation`** — core data structure
- **`ConstellationBuilder`** — core data structure
- **`ResolutionError`** — core data structure
- **`ConstellationResolver`** — core data structure
- **`CompiledConstellation`** — core data structure
- **`ConstellationCompiler`** — core data structure
- **`ConstellationRegistry`** — core data structure
- **`DeploymentTarget`** — state enumeration

### Key Functions

- `new()`
- `with_dependency()`
- `with_dependencies()`
- `new()`
- `skill_names()`
- `has_skill()`
- `unique_dependency_count()`
- `new()`
- `version()`
- `description()`
- ... and 15 more

## Why Ternary?

The balanced ternary system {-1, 0, +1} (also known as Z₃) is the mathematically optimal discrete encoding:
- **More expressive than binary**: three states capture positive, neutral, and negative
- **Natural for decisions**: accept/reject/abstain, buy/hold/sell, agree/disagree/neutral
- **Self-balancing**: the 0 state acts as a universal screen, preventing pathological lock-in
- **Z₃ cyclic dynamics**: rock-paper-scissors is the only natural coordination mechanism

## Stats

| Metric | Value |
|--------|-------|
| Lines of Rust | 601 |
| Test count | 19 |
| Public types | 9 |
| Public functions | 25 |

## Ecosystem

This crate is part of the **[SuperInstance Ternary Fleet](https://github.com/orgs/SuperInstance/repositories?q=ternary)**:

- **[ternary-core](https://github.com/SuperInstance/ternary-core)** — shared traits and Z₃ arithmetic
- **[ternary-grid](https://github.com/SuperInstance/ternary-grid)** — spatial grid with {-1, 0, +1} cells
- **[ternary-graph](https://github.com/SuperInstance/ternary-graph)** — ternary-weighted graph algorithms
- **[ternary-automata](https://github.com/SuperInstance/ternary-automata)** — three-state cellular automata
- **[ternary-compiler](https://github.com/SuperInstance/ternary-compiler)** — expression compiler and optimizer

200+ crates. 4,300+ tests. One pattern.

## Research Context

The ternary approach connects to several active research areas:
- **Ternary Neural Networks** (TNNs): weights constrained to {-1, 0, +1} for efficient inference
- **Huawei's ternary chip**: 7nm ternary silicon with 60% less power consumption
- **Active inference**: free energy minimization naturally maps to ternary action selection
- **Cyclic dominance**: RPS dynamics maintain biodiversity in spatial ecology
- **Z₃ group theory**: the only algebraic group on three elements is cyclic addition mod 3

## Usage

```toml
[dependencies]
ternary-constellation = "0.1.0"
```

```rust
use ternary_constellation;
```

## License

MIT
