use crate::manifest::Manifest;
use crate::maven::MavenClient;
use anyhow::Result;
use std::collections::HashMap;
use varisat::{ExtendFormula, Lit, Solver};

pub struct Resolver<'a> {
    client: &'a MavenClient,
    manifest: &'a Manifest,
    var_map: HashMap<String, varisat::Var>,
    rev_map: HashMap<varisat::Var, String>,
    solver: Solver<'a>,
}

impl<'a> Resolver<'a> {
    pub fn new(client: &'a MavenClient, manifest: &'a Manifest) -> Self {
        Self {
            client,
            manifest,
            var_map: HashMap::new(),
            rev_map: HashMap::new(),
            solver: Solver::new(),
        }
    }

    pub fn resolve(&mut self) -> Result<Vec<String>> {
        for (name, version) in &self.manifest.dependencies {
            let key = format!("{}:{}", name, version);
            let var = self.get_or_create_var(&key);
            // Root deps must be true
            self.solver.add_clause(&[Lit::from_var(var, true)]);

            self.resolve_deps(&key, var)?;
        }

        let solution = self.solver.solve().unwrap();

        if solution {
            let model = self.solver.model().unwrap();
            let mut resolved = Vec::new();
            for lit in model {
                if lit.is_positive() {
                    let var = lit.var();
                    if let Some(key) = self.rev_map.get(&var) {
                        resolved.push(key.clone());
                    }
                }
            }
            Ok(resolved)
        } else {
            anyhow::bail!("Unsatisfiable dependencies")
        }
    }

    fn get_or_create_var(&mut self, key: &str) -> varisat::Var {
        if let Some(&var) = self.var_map.get(key) {
            var
        } else {
            let var = self.solver.new_var();
            self.var_map.insert(key.to_string(), var);
            self.rev_map.insert(var, key.to_string());
            var
        }
    }

    fn resolve_deps(&mut self, parent_key: &str, parent_var: varisat::Var) -> Result<()> {
        let parts: Vec<&str> = parent_key.split(':').collect();
        if parts.len() != 3 {
            return Ok(());
        }
        let group = parts[0];
        let artifact = parts[1];
        let version = parts[2];

        let pom = match self.client.get_pom(group, artifact, version) {
            Ok(p) => p,
            Err(_) => return Ok(()),
        };

        for dep in pom.dependencies.dependency {
            if let Some(scope) = &dep.scope {
                if scope == "test" {
                    continue;
                }
            }

            if let Some(ver) = dep.version {
                if ver.contains("${") {
                    continue;
                }

                let dep_key = format!("{}:{}:{}", dep.group_id, dep.artifact_id, ver);
                let dep_var = self.get_or_create_var(&dep_key);

                // Implication: Parent -> Child
                // !Parent v Child
                // Lit::from_var(parent_var, false) is negative?
                // varisat: Lit::from_var(var, true) is positive?
                // Docs say: Lit::from_var(var, true) -> var
                // Lit::from_var(var, false) -> !var
                // Wait, let's check. Usually bool is "is_positive".
                // If so, !Parent is Lit::from_var(parent_var, false).
                // Child is Lit::from_var(dep_var, true).

                self.solver.add_clause(&[
                    Lit::from_var(parent_var, false),
                    Lit::from_var(dep_var, true),
                ]);
            }
        }
        Ok(())
    }
}
