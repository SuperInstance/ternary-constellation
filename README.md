# Ternary Constellation — Grouping Ternary Crates into Deployable Units with Dependency Resolution

**Ternary Constellation** implements the *constellation pattern*: grouping related ternary skill crates into named, versioned deployable units with declared dependencies. It provides dependency resolution, conflict detection, cross-compilation for deployment targets, and a fleet-wide registry. A constellation maps to the "room type" concept — a Codespace room loads a specific constellation of skills.

## Why It Matters

Modern distributed systems are composed of dozens to hundreds of micro-packages. Without a dependency-aware grouping mechanism, deployment becomes a manual exercise in tracking which packages go together. The constellation pattern solves this by treating a group of related ternary crates as a single atomic deployment unit. Dependency resolution ensures all transitive dependencies are satisfied; conflict detection prevents two crates from requiring incompatible versions of the same dependency. This is analogous to Helm charts for Kubernetes, but specialized for the ternary ecosystem where each skill operates on {-1, 0, +1} logic.

## How It Works

### Skill Descriptors

Each skill (crate) in a constellation is described by a `SkillDescriptor` carrying its name, version, and dependency list. Dependencies are expressed as string names, enabling loose coupling and version ranges.

### Constellation Construction

A `Constellation` is built incrementally via `ConstellationBuilder`. Skills are added one at a time, and the builder validates that each skill's name is unique within the constellation. The builder pattern allows fluent composition:

```
Constellation::new("fleet-inference")
    .add_skill("ternary-core", "0.1.0")
    .add_skill("ternary-inference", "0.1.0", deps=["ternary-core"])
```

### Dependency Resolution

The `ConstellationResolver` performs topological sort over the dependency graph to determine load order. Cycle detection runs in O(V + E) using DFS coloring (white/gray/black). If a cycle is found, resolution fails with the participating nodes identified. Version conflicts (two skills requiring different versions of the same dependency) are detected during resolution.

### Compilation and Bundling

The `ConstellationCompiler` bundles resolved constellations for specific deployment targets (GPU architecture, embedded device, etc.). It produces a self-contained package with all skills and their dependencies, ready for deployment.

### Registry

The `ConstellationRegistry` catalogs constellations across a fleet. It supports lookup by name, version, and tag. The registry is itself a CRDT — fleet nodes merge their local registries to maintain a globally consistent catalog.

## Quick Start

```rust
use ternary_constellation::{SkillDescriptor, Constellation, ConstellationBuilder};

let core = SkillDescriptor::new("ternary-core", "0.1.0");
let inference = SkillDescriptor::new("ternary-inference", "0.1.0")
    .with_dependency("ternary-core");

let mut constellation = Constellation::new("fleet-ai");
constellation.skills = vec![core, inference];

assert!(constellation.has_skill("ternary-core"));
assert_eq!(constellation.unique_dependency_count(), 1);
```

```bash
cargo add ternary-constellation
```

## API

| Type / Function | Description |
|---|---|
| `SkillDescriptor` | `{ name, version, dependencies }` with builder methods |
| `Constellation` | Named group of skills with `skill_names()`, `has_skill()` |
| `ConstellationBuilder` | Fluent builder for constellations |
| `ConstellationResolver` | Topological sort + cycle detection (O(V+E)) |
| `ConstellationCompiler` | Bundles for deployment targets |
| `ConstellationRegistry` | Fleet-wide CRDT catalog |

## Architecture Notes

Constellations are the deployment primitive in **SuperInstance**. Each room type (Codespace room) loads a specific constellation — for example, an inference room loads the "fleet-inference" constellation containing ternary-inference, ternary-core, and ternary-compression. The γ + η = C conservation law is maintained at the constellation level: the total compute budget γ plus dependency overhead η must not exceed the fleet capacity C. See [Architecture](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md).

## References

- Taylor, R. N. & Medvidovic, N. et al. "Software Architecture: Foundations, Theory, and Practice," *SEI*, 2009 — component dependency management.
- Shapiro, Marc et al. "Conflict-free Replicated Data Types," *SSS*, 2011 — CRDT merge semantics for registries.
- Helm, Eric. "Helm: The Package Manager for Kubernetes," *CNCF*, 2019 — dependency-aware packaging patterns.

## License

MIT
