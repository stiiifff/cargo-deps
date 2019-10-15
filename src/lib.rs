mod dep;
mod graph;
mod project;
mod util;

pub mod config;
pub mod error;

use std::{
    fs::File,
    io::{self, BufWriter},
    path::Path,
};
use {
    config::Config,
    error::{CliError, CliResult},
    project::Project,
};

pub fn execute(cfg: Config) -> CliResult<()> {
    // Search through parent dirs for Cargo.toml.
    is_cargo_toml(&cfg.manifest_path)?;
    let manifest_path = util::find_manifest_file(&cfg.manifest_path)?;

    // Cargo.lock must be in the same directory as Cargo.toml or in a parent directory.
    let manifest = manifest_path.to_str().unwrap();
    let lock_file = format!("{}.lock", &manifest[0..manifest.len() - 5]);
    let lock_path = util::find_manifest_file(&lock_file)?;

    // Graph the project.
    let dot_file = cfg.dot_file.clone();
    let project = Project::with_config(cfg)?;
    let graph = project.graph(manifest_path, lock_path)?;

    // Render the dot file.
    match dot_file {
        None => {
            let o = io::stdout();
            let mut bw = BufWriter::new(o.lock());
            graph.render_to(&mut bw)
        }
        Some(file) => {
            let o = File::create(&Path::new(&file)).expect("Failed to create file");
            let mut bw = BufWriter::new(o);
            graph.render_to(&mut bw)
        }
    }
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
