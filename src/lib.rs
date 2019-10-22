//! This library is the backend for the `cargo-deps` command-line program, containing the actual
//! logic.
//!
//! This library provides the following functionality:
//!
//! + Getting the dependency graph of a crate in its full intermediate representation.
//! + Getting the final graphviz representation of a crate's dependencies.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod config;
mod dep;
mod error;
mod graph;
mod project;
mod util;

pub use config::Config;
pub use error::{Error, Result};

use std::{io::BufWriter, path::Path};

use graph::DepGraph;
use project::Project;

/// Gets the full representation of the dependency graph, without converting it to graphviz output.
///
/// Pass the result of this function to `render_dep_graph` for the graphviz string.
pub fn get_dep_graph(cfg: Config) -> Result<DepGraph> {
    // Search through parent dirs for Cargo.toml.
    is_cargo_toml(&cfg.manifest_path)?;
    let manifest_path = util::find_manifest_file(&cfg.manifest_path)?;

    // Cargo.lock must be in the same directory as Cargo.toml or in a parent directory.
    let manifest = manifest_path.to_str().unwrap();
    let lock_file = format!("{}.lock", &manifest[0..manifest.len() - 5]);
    let lock_path = util::find_manifest_file(&lock_file)?;

    // Graph the project.
    let project = Project::with_config(cfg)?;
    project.graph(manifest_path, lock_path)
}

/// Converts the dependency graph representation into a graphviz string.
pub fn render_dep_graph(graph: DepGraph) -> Result<String> {
    let mut bytes: Vec<u8> = Vec::new();
    let mut writer = BufWriter::new(&mut bytes);
    graph.render_to(&mut writer)?;
    drop(writer);

    String::from_utf8(bytes).map_err(|err| Error::Generic(err.to_string()))
}

// Check that the manifest file name is "Cargo.toml".
fn is_cargo_toml(file_name: &str) -> Result<()> {
    let path = Path::new(file_name);

    if let Some(file_name) = path.file_name() {
        if file_name != "Cargo.toml" {
            return Err(Error::Toml(
                "The manifest-path must be a path to a Cargo.toml file".into(),
            ));
        }
    } else {
        return Err(Error::Toml("The manifest path is not a valid file".into()));
    }

    Ok(())
}
