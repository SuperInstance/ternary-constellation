# ternary-constellation: Constellation pattern for grouping related crates into deployable units

## Why This Exists

A single ternary skill isn't useful in isolation. A Codespace room loads a specific set of skills — navigation, communication, sensing — that work together. This "room type" needs to be defined, validated, resolved for dependencies, and compiled into a bundle for the target hardware (ESP32, WASM, or native). This crate handles that lifecycle: declare a constellation of skills, resolve their dependencies, detect conflicts, and produce a compiled bundle.

## Core Concepts

- **SkillDescriptor**: A named skill with a version and a list of dependency names. The atomic unit that goes into a constellation.
- **Constellation**: A named group of skills with version and description. Maps to a "room type" — e.g., "navigation-room" includes nav, sensor, and mapping skills.
- **ConstellationBuilder**: Fluent builder for constructing constellations incrementally.
- **ConstellationResolver**: Static methods for dependency analysis: collecting all transitive deps, detecting duplicates and circular dependencies, finding root skills (no internal deps), and producing a topological load order.
- **ConstellationCompiler**: Validates a constellation (no conflicts) and produces a `CompiledConstellation` with target-specific size estimates and a deterministic checksum.
- **DeploymentTarget**: Where the constellation runs: `Esp32`, `Wasm`, or `Native`.
- **ConstellationRegistry**: Fleet-wide catalog of constellations by name, with lookup by constellation name or by skill name.

## Quick Start

```toml
[dependencies]
ternary-constellation = "0.1"
```

```rust
use ternary_constellation::*;

let constellation = ConstellationBuilder::new("navigation-room")
    .description("Full navigation suite for exploration rooms")
    .add_skill(SkillDescriptor::new("base-nav", "1.0"))
    .add_skill(SkillDescriptor::new("pathfinding", "1.0").with_dependency("base-nav"))
    .add_skill(SkillDescriptor::new("mapping", "1.0").with_dependency("base-nav"))
    .build();

// Resolve: check for conflicts and get load order
let conflicts = ConstellationResolver::detect_conflicts(&constellation);
assert!(conflicts.is_empty());
let order = ConstellationResolver::dependency_order(&constellation).unwrap();
// order: ["base-nav", "pathfinding", "mapping"] or ["base-nav", "mapping", "pathfinding"]

// Compile for ESP32
let compiled = ConstellationCompiler::compile(&constellation, DeploymentTarget::Esp32).unwrap();
assert_eq!(compiled.skill_count, 3);
```

## API Overview

| Type | Description |
|------|-------------|
| `SkillDescriptor` | Named skill with version and dependency list |
| `Constellation` | Named group of skills with metadata |
| `ConstellationBuilder` | Fluent builder for constellations |
| `ConstellationResolver` | Dependency analysis: conflicts, ordering, root detection |
| `ConstellationCompiler` | Validates and compiles to a target platform |
| `CompiledConstellation` | Result of compilation: target, size, checksum |
| `DeploymentTarget` | Enum: `Esp32`, `Wasm`, `Native` |
| `ConstellationRegistry` | Fleet-wide catalog with lookup by name or skill |
| `ResolutionError` | Error for dependency conflicts and cycles |

## How It Works

Dependency resolution uses standard graph algorithms. `collect_dependencies` builds a reverse map from dependency name to the skills that need it. `detect_conflicts` checks for duplicate skill names within a constellation and for direct circular dependencies (A depends on B, B depends on A). `dependency_order` performs a topological sort using Kahn's algorithm (BFS-based) — skills with no internal dependencies come first, then their dependents, etc. If the sort can't visit all nodes, there's a cycle.

Compilation is simulated: it checks for conflicts, estimates binary size based on target platform (ESP32 is smallest, native is largest), and generates a deterministic checksum from the constellation name and skill names. A real implementation would invoke cross-compilation toolchains.

## Known Limitations

- Circular dependency detection only catches direct 2-node cycles (A↔B). Longer cycles (A→B→C→A) are caught by the topological sort failure, not by `detect_conflicts`.
- Version constraints aren't enforced. Two skills could require different versions of the same dependency without triggering a conflict.
- Compilation is simulated — no actual cross-compilation occurs. Size estimates are rough multipliers.
- No support for optional or conditional skills within a constellation.
- The registry is in-memory only. No persistence or network sync.

## Use Cases

1. **Room type definition**: A "sensor-analysis" room loads a constellation containing data-collection, filtering, and anomaly-detection skills with resolved dependencies.
2. **Fleet deployment**: An ESP32 node receives a compiled constellation bundle containing exactly the skills it needs, validated for no conflicts.
3. **Skill catalog search**: Given a skill like "thermal-imaging", find which constellations across the fleet include it using `find_by_skill`.

## Ecosystem Context

Sits above individual skill crates (`ternary-nav`, `ternary-sensor`, etc.) and below the fleet orchestration layer (`ternary-swarm`, `ternary-distributed`). Uses `ternary-bridge` for interop between constellation members that speak different protocols. The registry could be backed by `ternary-registry` for persistent storage.

## License

MIT
