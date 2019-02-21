use crate::config::Config;
use crate::dep::{DeclaredDep, DepKind};
use crate::error::{CliError, CliResult};
use crate::graph::DepGraph;
use crate::util;
use std::path::PathBuf;
use toml::Value;

#[derive(Debug)]
pub struct Project {
    cfg: Config,
}

impl Project {
    pub fn with_config(cfg: Config) -> CliResult<Self> {
        Ok(Project { cfg })
    }

    pub fn graph(self, manifest_path: PathBuf, lock_path: PathBuf) -> CliResult<DepGraph> {
        let (root_deps, root_name, root_version) = self.parse_root_deps(&manifest_path)?;

        let mut dg = self.parse_lock_file(lock_path)?;

        // Set node 0 to be the root.
        if !dg.set_root(&root_name, &root_version) {
            return Err(CliError::Toml("Missing name or version".into()));
        }

        // Set the kind of dependency on each dep.
        dg.set_resolved_kind(&root_deps);

        if !self.cfg.include_vers {
            dg.show_version_on_duplicates();
        }

        Ok(dg)
    }

    /// Builds a graph of the resolved dependencies declared in the lock file.
    fn parse_lock_file(&self, lock_path: PathBuf) -> CliResult<DepGraph> {
        let lock_toml = util::toml_from_file(lock_path)?;

        let mut dg = DepGraph::new(self.cfg.clone());

        if let Some(root) = lock_toml.get("root") {
            parse_package(&mut dg, root);
        }

        if let Some(&Value::Array(ref packages)) = lock_toml.get("package") {
            for pkg in packages {
                parse_package(&mut dg, pkg);
            }
        }

        Ok(dg)
    }

    /// Builds a list of the dependencies declared in the manifest file.
    pub fn parse_root_deps(
        &self,
        manifest_path: &PathBuf,
    ) -> CliResult<(Vec<DeclaredDep>, String, String)> {
        let manifest_toml = util::toml_from_file(manifest_path)?;

        let mut declared_deps = vec![];
        let mut v = vec![];

        // Get the name and version of the root project.
        let (root_name, root_version) = {
            if let Some(table) = manifest_toml.get("package") {
                if let Some(table) = table.as_table() {
                    if let (Some(&Value::String(ref n)), Some(&Value::String(ref v))) =
                        (table.get("name"), table.get("version"))
                    {
                        (n.to_string(), v.to_string())
                    } else {
                        return Err(CliError::Toml("No name for 'package'".into()));
                    }
                } else {
                    return Err(CliError::Toml(
                        "Could not parse 'package' as a table".into(),
                    ));
                }
            } else {
                return Err(CliError::Toml("No 'package' table found".into()));
            }
        };

        if let Some(table) = manifest_toml.get("dependencies") {
            if let Some(table) = table.as_table() {
                for (name, dep_table) in table.iter() {
                    if let Some(&Value::Boolean(true)) = dep_table.get("optional") {
                        declared_deps.push(DeclaredDep::with_kind(name.clone(), DepKind::Optional));
                    } else {
                        declared_deps.push(DeclaredDep::with_kind(name.clone(), DepKind::Build));
                    }
                    v.push(name.clone());
                }
            }
        }

        if let Some(table) = manifest_toml.get("dev-dependencies") {
            if let Some(table) = table.as_table() {
                for (name, _) in table.iter() {
                    declared_deps.push(DeclaredDep::with_kind(name.clone(), DepKind::Dev));
                    v.push(name.clone());
                }
            }
        }

        Ok((declared_deps, root_name, root_version))
    }
}

fn parse_package(dg: &mut DepGraph, pkg: &Value) {
    let name = pkg
        .get("name")
        .expect("no 'name' field in Cargo.lock [package] or [root] table")
        .as_str()
        .expect(
            "'name' field of [package] or [root] table in Cargo.lock was not a \
             valid string",
        )
        .to_owned();
    let ver = pkg
        .get("version")
        .expect("no 'version' field in Cargo.lock [package] or [root] table")
        .as_str()
        .expect(
            "'version' field of [package] or [root] table in Cargo.lock was not a \
             valid string",
        )
        .to_owned();

    let id = dg.find_or_add(&*name, &*ver);

    if let Some(&Value::Array(ref deps)) = pkg.get("dependencies") {
        for dep in deps {
            let dep_vec = dep.as_str().unwrap_or("").split(' ').collect::<Vec<_>>();
            let dep_string = dep_vec[0].to_owned();
            let ver = dep_vec[1];
            dg.add_child(id, &*dep_string, ver);
        }
    }
}
