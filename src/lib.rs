mod config;
mod dep;
mod error;
mod graph;
mod project;
mod util;

pub use config::Config;
pub use error::{CliError, CliResult};

use std::{io::BufWriter, path::Path};

use graph::DepGraph;
use project::Project;

pub fn get_dep_graph(cfg: Config) -> CliResult<DepGraph> {
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

pub fn render_dep_graph(graph: DepGraph) -> CliResult<String> {
    let mut v: Vec<u8> = Vec::new();
    let mut bw = BufWriter::new(&mut v);
    graph.render_to(&mut bw)?;
    drop(bw);

    String::from_utf8(v).map_err(|err| CliError::Generic(err.to_string()))
}

// Check that the manifest file name is "Cargo.toml".
fn is_cargo_toml(file_name: &str) -> CliResult<()> {
    let path = Path::new(file_name);

    if let Some(file_name) = path.file_name() {
        if file_name != "Cargo.toml" {
            return Err(CliError::Toml(
                "The manifest-path must be a path to a Cargo.toml file".into(),
            ));
        }
    } else {
        return Err(CliError::Toml(
            "The manifest path is not a valid file".into(),
        ));
    }

    Ok(())
}
