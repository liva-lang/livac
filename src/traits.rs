/// Trait system for generic constraints
/// Maps Liva operators and features to Rust standard library traits

use std::collections::{HashMap, HashSet};

/// Trait definition with Rust mapping
#[derive(Debug, Clone, PartialEq)]
pub struct TraitDef {
    pub name: String,
    pub rust_path: String,
    pub operators: Vec<String>,
    pub requires: Vec<String>,  // Other traits required by this trait
}

/// Built-in traits supported by Liva generics
pub struct TraitRegistry {
    traits: HashMap<String, TraitDef>,
}

impl TraitRegistry {
    pub fn new() -> Self {
        let mut registry = TraitRegistry {
            traits: HashMap::new(),
        };
        registry.register_builtin_traits();
        registry
    }

    fn register_builtin_traits(&mut self) {
        // Arithmetic operators
        // Note: We use Copy bound to ensure T can be returned by value
        self.register(TraitDef {
            name: "Add".to_string(),
            rust_path: "std::ops::Add<Output=T> + Copy".to_string(),
            operators: vec!["+".to_string()],
            requires: vec![],
        });

        self.register(TraitDef {
            name: "Sub".to_string(),
            rust_path: "std::ops::Sub<Output=T> + Copy".to_string(),
            operators: vec!["-".to_string()],
            requires: vec![],
        });

        self.register(TraitDef {
            name: "Mul".to_string(),
            rust_path: "std::ops::Mul<Output=T> + Copy".to_string(),
            operators: vec!["*".to_string()],
            requires: vec![],
        });

        self.register(TraitDef {
            name: "Div".to_string(),
            rust_path: "std::ops::Div<Output=T> + Copy".to_string(),
            operators: vec!["/".to_string()],
            requires: vec![],
        });

        self.register(TraitDef {
            name: "Rem".to_string(),
            rust_path: "std::ops::Rem<Output=T> + Copy".to_string(),
            operators: vec!["%".to_string()],
            requires: vec![],
        });

        // Unary operators
        self.register(TraitDef {
            name: "Neg".to_string(),
            rust_path: "std::ops::Neg<Output=T> + Copy".to_string(),
            operators: vec!["unary-".to_string()],
            requires: vec![],
        });

        self.register(TraitDef {
            name: "Not".to_string(),
            rust_path: "std::ops::Not<Output=T> + Copy".to_string(),
            operators: vec!["!".to_string()],
            requires: vec![],
        });

        // Comparison operators
        self.register(TraitDef {
            name: "Eq".to_string(),
            rust_path: "std::cmp::PartialEq + Copy".to_string(),
            operators: vec!["==".to_string(), "!=".to_string()],
            requires: vec![],
        });

        self.register(TraitDef {
            name: "Ord".to_string(),
            rust_path: "std::cmp::PartialOrd + Copy".to_string(),
            operators: vec![">".to_string(), "<".to_string(), ">=".to_string(), "<=".to_string()],
            requires: vec!["Eq".to_string()],  // Ord requires Eq
        });

        // Utility traits
        self.register(TraitDef {
            name: "Clone".to_string(),
            rust_path: "Clone".to_string(),
            operators: vec![],
            requires: vec![],
        });

        self.register(TraitDef {
            name: "Display".to_string(),
            rust_path: "std::fmt::Display".to_string(),
            operators: vec![],
            requires: vec![],
        });

        self.register(TraitDef {
            name: "Debug".to_string(),
            rust_path: "std::fmt::Debug".to_string(),
            operators: vec![],
            requires: vec![],
        });

        self.register(TraitDef {
            name: "Copy".to_string(),
            rust_path: "Copy".to_string(),
            operators: vec![],
            requires: vec!["Clone".to_string()],  // Copy requires Clone
        });

        self.register(TraitDef {
            name: "Default".to_string(),
            rust_path: "Default".to_string(),
            operators: vec![],
            requires: vec![],
        });
    }

    fn register(&mut self, trait_def: TraitDef) {
        self.traits.insert(trait_def.name.clone(), trait_def);
    }

    /// Get trait definition by name
    pub fn get_trait(&self, name: &str) -> Option<&TraitDef> {
        self.traits.get(name)
    }

    /// Find trait required for an operator
    pub fn trait_for_operator(&self, op: &str) -> Option<&TraitDef> {
        self.traits.values().find(|t| t.operators.contains(&op.to_string()))
    }

    /// Get all traits required for a given trait (transitive dependencies)
    pub fn get_required_traits(&self, trait_name: &str) -> HashSet<String> {
        let mut required = HashSet::new();
        let mut to_visit = vec![trait_name.to_string()];
        
        while let Some(current) = to_visit.pop() {
            if required.contains(&current) {
                continue;
            }
            required.insert(current.clone());
            
            if let Some(trait_def) = self.traits.get(&current) {
                for req in &trait_def.requires {
                    to_visit.push(req.clone());
                }
            }
        }
        
        required
    }

    /// Check if a constraint is valid
    pub fn is_valid_constraint(&self, constraint: &str) -> bool {
        self.traits.contains_key(constraint)
    }

    /// Get all trait names
    pub fn all_trait_names(&self) -> Vec<String> {
        let mut names: Vec<_> = self.traits.keys().cloned().collect();
        names.sort();
        names
    }

    /// Generate Rust trait bounds for a type parameter
    pub fn generate_rust_bounds(&self, constraints: &[String]) -> String {
        if constraints.is_empty() {
            return String::new();
        }

        // Expand constraints to include required traits
        let mut all_traits = HashSet::new();
        for constraint in constraints {
            all_traits.extend(self.get_required_traits(constraint));
        }

        // Remove redundant traits (if Ord is present, remove Eq)
        if all_traits.contains("Ord") {
            all_traits.remove("Eq");
        }
        if all_traits.contains("Copy") {
            all_traits.remove("Clone");
        }

        // Convert to sorted list for deterministic output
        let mut trait_list: Vec<_> = all_traits.iter().collect();
        trait_list.sort();

        // Generate Rust bounds
        let bounds: Vec<String> = trait_list
            .iter()
            .filter_map(|name| {
                self.traits.get(*name).map(|t| t.rust_path.clone())
            })
            .collect();

        if bounds.is_empty() {
            String::new()
        } else {
            format!(": {}", bounds.join(" + "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trait_registry() {
        let registry = TraitRegistry::new();
        
        // Test operator lookup
        assert!(registry.trait_for_operator("+").is_some());
        assert_eq!(registry.trait_for_operator("+").unwrap().name, "Add");
        assert_eq!(registry.trait_for_operator("%").unwrap().name, "Rem");
        
        // Test constraint validation
        assert!(registry.is_valid_constraint("Add"));
        assert!(registry.is_valid_constraint("Ord"));
        assert!(!registry.is_valid_constraint("InvalidTrait"));
    }

    #[test]
    fn test_required_traits() {
        let registry = TraitRegistry::new();
        
        // Ord requires Eq
        let ord_traits = registry.get_required_traits("Ord");
        assert!(ord_traits.contains("Ord"));
        assert!(ord_traits.contains("Eq"));
        
        // Add requires nothing
        let add_traits = registry.get_required_traits("Add");
        assert!(add_traits.contains("Add"));
        assert_eq!(add_traits.len(), 1);
    }

    #[test]
    fn test_rust_bounds_generation() {
        let registry = TraitRegistry::new();
        
        // Single constraint
        let bounds = registry.generate_rust_bounds(&["Add".to_string()]);
        assert_eq!(bounds, ": std::ops::Add<Output=Self>");
        
        // Multiple constraints
        let bounds = registry.generate_rust_bounds(&["Add".to_string(), "Sub".to_string()]);
        assert!(bounds.contains("Add"));
        assert!(bounds.contains("Sub"));
        
        // Ord automatically includes Eq
        let bounds = registry.generate_rust_bounds(&["Ord".to_string()]);
        assert!(bounds.contains("PartialOrd"));
        assert!(bounds.contains("PartialEq"));
    }
}
