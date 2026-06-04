#![forbid(unsafe_code)]

//! Constellation pattern for grouping related ternary crates into deployable units.
//!
//! A `Constellation` is a named group of skills (crates) with declared dependencies.
//! `ConstellationBuilder` constructs them, `ConstellationResolver` resolves dependencies
//! and detects conflicts, `ConstellationCompiler` bundles them for deployment targets,
//! and `ConstellationRegistry` catalogs constellations across a fleet. Maps to the
//! "room type" concept — a Codespace room loads a specific constellation.

use std::collections::{HashMap, HashSet};

// ── SkillDescriptor ────────────────────────────────────────────────────────

/// A skill (crate) that can be part of a constellation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillDescriptor {
    pub name: String,
    pub version: String,
    pub dependencies: Vec<String>,
}

impl SkillDescriptor {
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            dependencies: Vec::new(),
        }
    }

    pub fn with_dependency(mut self, dep: &str) -> Self {
        self.dependencies.push(dep.to_string());
        self
    }

    pub fn with_dependencies(mut self, deps: &[&str]) -> Self {
        self.dependencies = deps.iter().map(|s| s.to_string()).collect();
        self
    }
}

// ── Constellation ──────────────────────────────────────────────────────────

/// A named group of skills with metadata.
#[derive(Debug, Clone)]
pub struct Constellation {
    pub name: String,
    pub version: String,
    pub description: String,
    pub skills: Vec<SkillDescriptor>,
}

impl Constellation {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            description: String::new(),
            skills: Vec::new(),
        }
    }

    /// Get all skill names in this constellation.
    pub fn skill_names(&self) -> Vec<&str> {
        self.skills.iter().map(|s| s.name.as_str()).collect()
    }

    /// Check if a skill is included.
    pub fn has_skill(&self, name: &str) -> bool {
        self.skills.iter().any(|s| s.name == name)
    }

    /// Get the total dependency count (unique).
    pub fn unique_dependency_count(&self) -> usize {
        let mut deps = HashSet::new();
        for skill in &self.skills {
            for dep in &skill.dependencies {
                deps.insert(dep.clone());
            }
        }
        deps.len()
    }
}

// ── ConstellationBuilder ───────────────────────────────────────────────────

/// Builds a constellation incrementally.
#[derive(Debug)]
pub struct ConstellationBuilder {
    name: String,
    version: String,
    description: String,
    skills: Vec<SkillDescriptor>,
}

impl ConstellationBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            description: String::new(),
            skills: Vec::new(),
        }
    }

    pub fn version(mut self, v: &str) -> Self {
        self.version = v.to_string();
        self
    }

    pub fn description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    pub fn add_skill(mut self, skill: SkillDescriptor) -> Self {
        self.skills.push(skill);
        self
    }

    pub fn build(self) -> Constellation {
        Constellation {
            name: self.name,
            version: self.version,
            description: self.description,
            skills: self.skills,
        }
    }
}

// ── ResolutionError ────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolutionError {
    pub message: String,
}

impl ResolutionError {
    pub fn new(msg: &str) -> Self {
        Self { message: msg.to_string() }
    }
}

impl std::fmt::Display for ResolutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ResolutionError: {}", self.message)
    }
}

// ── ConstellationResolver ──────────────────────────────────────────────────

/// Resolves dependencies and detects conflicts within a constellation.
pub struct ConstellationResolver;

impl ConstellationResolver {
    /// Collect all transitive dependencies from a constellation's skills.
    /// Returns a map from dependency name to the set of skills that need it.
    pub fn collect_dependencies(constellation: &Constellation) -> HashMap<String, HashSet<String>> {
        let mut dep_map: HashMap<String, HashSet<String>> = HashMap::new();
        for skill in &constellation.skills {
            for dep in &skill.dependencies {
                dep_map.entry(dep.clone())
                    .or_default()
                    .insert(skill.name.clone());
            }
        }
        dep_map
    }

    /// Detect conflicts: skills that depend on each other (circular) or
    /// the same skill appearing multiple times.
    pub fn detect_conflicts(constellation: &Constellation) -> Vec<ResolutionError> {
        let mut errors = Vec::new();

        // Check for duplicate skill names
        let mut seen = HashSet::new();
        for skill in &constellation.skills {
            if !seen.insert(&skill.name) {
                errors.push(ResolutionError::new(&format!(
                    "Duplicate skill: {}", skill.name
                )));
            }
        }

        // Check for circular dependencies (skill A depends on skill B which depends on A)
        let skill_names: HashSet<&str> = constellation.skills.iter().map(|s| s.name.as_str()).collect();
        for skill in &constellation.skills {
            for dep in &skill.dependencies {
                if skill_names.contains(dep.as_str()) {
                    // dep is another skill in the constellation; check if it depends back
                    if let Some(dep_skill) = constellation.skills.iter().find(|s| s.name == *dep) {
                        if dep_skill.dependencies.contains(&skill.name) {
                            errors.push(ResolutionError::new(&format!(
                                "Circular dependency: {} <-> {}", skill.name, dep
                            )));
                        }
                    }
                }
            }
        }

        errors
    }

    /// Get skills that have no unresolved dependencies (ready to load first).
    pub fn root_skills(constellation: &Constellation) -> Vec<&SkillDescriptor> {
        let skill_names: HashSet<&str> = constellation.skills.iter().map(|s| s.name.as_str()).collect();
        constellation.skills.iter()
            .filter(|s| !s.dependencies.iter().any(|d| skill_names.contains(d.as_str())))
            .collect()
    }

    /// Topological sort of skills by dependency order.
    /// Returns skills in load order (dependencies first), or an error if cycles exist.
    pub fn dependency_order(constellation: &Constellation) -> Result<Vec<String>, ResolutionError> {
        let skill_names: HashSet<&str> = constellation.skills.iter().map(|s| s.name.as_str()).collect();
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();

        for skill in &constellation.skills {
            in_degree.entry(&skill.name).or_insert(0);
            adj.entry(&skill.name).or_default();
        }

        for skill in &constellation.skills {
            for dep in &skill.dependencies {
                if skill_names.contains(dep.as_str()) {
                    *in_degree.entry(&skill.name).or_insert(0) += 1;
                    adj.entry(dep.as_str()).or_default().push(&skill.name);
                }
            }
        }

        let mut queue: Vec<&str> = in_degree.iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&name, _)| name)
            .collect();

        let mut result = Vec::new();
        while let Some(name) = queue.pop() {
            result.push(name.to_string());
            if let Some(neighbors) = adj.get(name) {
                for &neighbor in neighbors {
                    if let Some(deg) = in_degree.get_mut(neighbor) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push(neighbor);
                        }
                    }
                }
            }
        }

        if result.len() != constellation.skills.len() {
            return Err(ResolutionError::new("Circular dependency detected in topological sort"));
        }

        Ok(result)
    }
}

// ── DeploymentTarget ───────────────────────────────────────────────────────

/// Target platform for constellation compilation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeploymentTarget {
    /// ESP32 microcontroller.
    Esp32,
    /// WebAssembly.
    Wasm,
    /// Native binary (Linux/macOS/Windows).
    Native,
}

impl std::fmt::Display for DeploymentTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeploymentTarget::Esp32 => write!(f, "esp32"),
            DeploymentTarget::Wasm => write!(f, "wasm"),
            DeploymentTarget::Native => write!(f, "native"),
        }
    }
}

// ── CompiledConstellation ──────────────────────────────────────────────────

/// A constellation compiled for a specific target.
#[derive(Debug, Clone)]
pub struct CompiledConstellation {
    pub constellation_name: String,
    pub target: DeploymentTarget,
    pub skill_count: usize,
    pub estimated_size_bytes: usize,
    pub checksum: String,
}

// ── ConstellationCompiler ──────────────────────────────────────────────────

/// Compiles a constellation into a bundle for a deployment target.
pub struct ConstellationCompiler;

impl ConstellationCompiler {
    /// Compile a constellation for a given target.
    /// In a real implementation this would invoke cross-compilation toolchains.
    /// Here we simulate it with size estimates and checksum generation.
    pub fn compile(
        constellation: &Constellation,
        target: DeploymentTarget,
    ) -> Result<CompiledConstellation, ResolutionError> {
        let conflicts = ConstellationResolver::detect_conflicts(constellation);
        if !conflicts.is_empty() {
            return Err(ResolutionError::new(&format!(
                "Cannot compile: {} conflicts detected", conflicts.len()
            )));
        }

        // Simulated size estimate
        let base_size = match target {
            DeploymentTarget::Esp32 => 4096,
            DeploymentTarget::Wasm => 2048,
            DeploymentTarget::Native => 8192,
        };
        let estimated_size = base_size * constellation.skills.len().max(1);

        // Simple checksum from constellation name + skill names
        let mut hash_val: u64 = 0;
        for byte in constellation.name.bytes() {
            hash_val = hash_val.wrapping_mul(31).wrapping_add(byte as u64);
        }
        for skill in &constellation.skills {
            for byte in skill.name.bytes() {
                hash_val = hash_val.wrapping_mul(31).wrapping_add(byte as u64);
            }
        }
        let checksum = format!("{:016x}", hash_val);

        Ok(CompiledConstellation {
            constellation_name: constellation.name.clone(),
            target,
            skill_count: constellation.skills.len(),
            estimated_size_bytes: estimated_size,
            checksum,
        })
    }
}

// ── ConstellationRegistry ──────────────────────────────────────────────────

/// Fleet-wide catalog of constellations.
#[derive(Debug)]
pub struct ConstellationRegistry {
    constellations: HashMap<String, Constellation>,
}

impl ConstellationRegistry {
    pub fn new() -> Self {
        Self { constellations: HashMap::new() }
    }

    /// Register a constellation.
    pub fn register(&mut self, constellation: Constellation) {
        self.constellations.insert(constellation.name.clone(), constellation);
    }

    /// Look up a constellation by name.
    pub fn get(&self, name: &str) -> Option<&Constellation> {
        self.constellations.get(name)
    }

    /// Remove a constellation from the registry.
    pub fn unregister(&mut self, name: &str) -> bool {
        self.constellations.remove(name).is_some()
    }

    /// List all registered constellation names.
    pub fn list_names(&self) -> Vec<&str> {
        self.constellations.keys().map(|s| s.as_str()).collect()
    }

    /// Find constellations that contain a given skill.
    pub fn find_by_skill(&self, skill_name: &str) -> Vec<&Constellation> {
        self.constellations.values()
            .filter(|c| c.has_skill(skill_name))
            .collect()
    }

    /// Total number of registered constellations.
    pub fn count(&self) -> usize {
        self.constellations.len()
    }
}

impl Default for ConstellationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_descriptor_builder() {
        let skill = SkillDescriptor::new("navigation", "1.0")
            .with_dependencies(&["math", "sensor"]);
        assert_eq!(skill.name, "navigation");
        assert_eq!(skill.dependencies.len(), 2);
    }

    #[test]
    fn test_constellation_new() {
        let c = Constellation::new("room-alpha");
        assert_eq!(c.name, "room-alpha");
        assert!(c.skills.is_empty());
    }

    #[test]
    fn test_constellation_has_skill() {
        let mut c = Constellation::new("room-alpha");
        c.skills.push(SkillDescriptor::new("nav", "1.0"));
        assert!(c.has_skill("nav"));
        assert!(!c.has_skill("comms"));
    }

    #[test]
    fn test_constellation_skill_names() {
        let mut c = Constellation::new("room-alpha");
        c.skills.push(SkillDescriptor::new("nav", "1.0"));
        c.skills.push(SkillDescriptor::new("comms", "1.0"));
        assert_eq!(c.skill_names(), vec!["nav", "comms"]);
    }

    #[test]
    fn test_builder_basic() {
        let c = ConstellationBuilder::new("room-beta")
            .version("2.0")
            .description("Test room")
            .add_skill(SkillDescriptor::new("nav", "1.0"))
            .add_skill(SkillDescriptor::new("sensor", "1.0").with_dependency("nav"))
            .build();
        assert_eq!(c.name, "room-beta");
        assert_eq!(c.version, "2.0");
        assert_eq!(c.skills.len(), 2);
    }

    #[test]
    fn test_resolver_collect_dependencies() {
        let c = ConstellationBuilder::new("test")
            .add_skill(SkillDescriptor::new("a", "1.0").with_dependencies(&["math", "log"]))
            .add_skill(SkillDescriptor::new("b", "1.0").with_dependencies(&["math"]))
            .build();
        let deps = ConstellationResolver::collect_dependencies(&c);
        assert_eq!(deps.len(), 2); // math, log
        assert_eq!(deps.get("math").unwrap().len(), 2); // a and b need math
    }

    #[test]
    fn test_resolver_no_conflicts() {
        let c = ConstellationBuilder::new("test")
            .add_skill(SkillDescriptor::new("a", "1.0"))
            .add_skill(SkillDescriptor::new("b", "1.0"))
            .build();
        let conflicts = ConstellationResolver::detect_conflicts(&c);
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_resolver_duplicate_conflict() {
        let c = ConstellationBuilder::new("test")
            .add_skill(SkillDescriptor::new("dup", "1.0"))
            .add_skill(SkillDescriptor::new("dup", "1.0"))
            .build();
        let conflicts = ConstellationResolver::detect_conflicts(&c);
        assert_eq!(conflicts.len(), 1);
        assert!(conflicts[0].message.contains("Duplicate"));
    }

    #[test]
    fn test_resolver_circular_conflict() {
        let c = ConstellationBuilder::new("test")
            .add_skill(SkillDescriptor::new("a", "1.0").with_dependency("b"))
            .add_skill(SkillDescriptor::new("b", "1.0").with_dependency("a"))
            .build();
        let conflicts = ConstellationResolver::detect_conflicts(&c);
        assert!(conflicts.iter().any(|e| e.message.contains("Circular")));
    }

    #[test]
    fn test_resolver_root_skills() {
        let c = ConstellationBuilder::new("test")
            .add_skill(SkillDescriptor::new("base", "1.0"))
            .add_skill(SkillDescriptor::new("derived", "1.0").with_dependency("base"))
            .build();
        let roots = ConstellationResolver::root_skills(&c);
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].name, "base");
    }

    #[test]
    fn test_resolver_dependency_order() {
        let c = ConstellationBuilder::new("test")
            .add_skill(SkillDescriptor::new("c", "1.0").with_dependency("b"))
            .add_skill(SkillDescriptor::new("b", "1.0").with_dependency("a"))
            .add_skill(SkillDescriptor::new("a", "1.0"))
            .build();
        let order = ConstellationResolver::dependency_order(&c).unwrap();
        let a_pos = order.iter().position(|n| n == "a").unwrap();
        let b_pos = order.iter().position(|n| n == "b").unwrap();
        let c_pos = order.iter().position(|n| n == "c").unwrap();
        assert!(a_pos < b_pos);
        assert!(b_pos < c_pos);
    }

    #[test]
    fn test_resolver_dependency_order_cycle_fails() {
        let c = ConstellationBuilder::new("test")
            .add_skill(SkillDescriptor::new("a", "1.0").with_dependency("b"))
            .add_skill(SkillDescriptor::new("b", "1.0").with_dependency("a"))
            .build();
        assert!(ConstellationResolver::dependency_order(&c).is_err());
    }

    #[test]
    fn test_compiler_esp32() {
        let c = ConstellationBuilder::new("room-x")
            .add_skill(SkillDescriptor::new("nav", "1.0"))
            .add_skill(SkillDescriptor::new("comms", "1.0"))
            .build();
        let compiled = ConstellationCompiler::compile(&c, DeploymentTarget::Esp32).unwrap();
        assert_eq!(compiled.target, DeploymentTarget::Esp32);
        assert_eq!(compiled.skill_count, 2);
        assert!(compiled.estimated_size_bytes > 0);
    }

    #[test]
    fn test_compiler_wasm() {
        let c = ConstellationBuilder::new("room-y")
            .add_skill(SkillDescriptor::new("skill", "1.0"))
            .build();
        let compiled = ConstellationCompiler::compile(&c, DeploymentTarget::Wasm).unwrap();
        assert_eq!(compiled.target, DeploymentTarget::Wasm);
        assert!(compiled.estimated_size_bytes < 100000); // reasonable
    }

    #[test]
    fn test_compiler_conflict_fails() {
        let c = ConstellationBuilder::new("bad")
            .add_skill(SkillDescriptor::new("dup", "1.0"))
            .add_skill(SkillDescriptor::new("dup", "1.0"))
            .build();
        assert!(ConstellationCompiler::compile(&c, DeploymentTarget::Native).is_err());
    }

    #[test]
    fn test_registry_register_and_get() {
        let mut reg = ConstellationRegistry::new();
        let c = ConstellationBuilder::new("room-a")
            .add_skill(SkillDescriptor::new("nav", "1.0"))
            .build();
        reg.register(c);
        assert!(reg.get("room-a").is_some());
        assert!(reg.get("room-b").is_none());
    }

    #[test]
    fn test_registry_find_by_skill() {
        let mut reg = ConstellationRegistry::new();
        reg.register(ConstellationBuilder::new("room-a")
            .add_skill(SkillDescriptor::new("nav", "1.0"))
            .build());
        reg.register(ConstellationBuilder::new("room-b")
            .add_skill(SkillDescriptor::new("comms", "1.0"))
            .build());
        let found = reg.find_by_skill("nav");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].name, "room-a");
    }

    #[test]
    fn test_registry_unregister() {
        let mut reg = ConstellationRegistry::new();
        reg.register(Constellation::new("room-x"));
        assert!(reg.unregister("room-x"));
        assert!(!reg.unregister("room-x"));
        assert_eq!(reg.count(), 0);
    }

    #[test]
    fn test_compiled_checksum_deterministic() {
        let c = ConstellationBuilder::new("test")
            .add_skill(SkillDescriptor::new("x", "1.0"))
            .build();
        let c1 = ConstellationCompiler::compile(&c, DeploymentTarget::Native).unwrap();
        let c2 = ConstellationCompiler::compile(&c, DeploymentTarget::Native).unwrap();
        assert_eq!(c1.checksum, c2.checksum);
    }
}
