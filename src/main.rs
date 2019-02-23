#![deny(
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unused_import_braces,
    unused_qualifications
)]

#[macro_use]
extern crate clap;
extern crate toml;

mod config;
mod dep;
mod error;
mod graph;
mod project;
mod util;

use crate::config::Config;
use crate::error::{CliError, CliResult};
use crate::project::Project;
use clap::{App, Arg, ArgMatches};
use std::fs::File;
use std::io::{self, BufWriter};
use std::path::Path;
use std::path::PathBuf;

fn parse_cli<'a>() -> ArgMatches<'a> {
    App::new("cargo-deps")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .args_from_usage(
            "
                 --dot-file [PATH] 'Output file (Default stdout)'
                 --filter [DEPNAMES] ... 'Only display provided deps'
                 --include-orphans 'Don't purge orphan nodes (useful in some workspaces)'
             -I, --include-versions 'Include the dependency version on nodes'

                 --all-deps 'Include all dependencies in the graph. \
                 Can be used with --no-regular-deps'
                 --no-regular-deps 'Exclude regular dependencies from the graph'
                 --build-deps 'Include build dependencies in the graph (purple)'
                 --dev-deps 'Include dev dependencies in the graph (blue)'
                 --optional-deps 'Include optional dependencies in the graph (red)'
            ",
        )
        .args(&[
            Arg::from_usage("--manifest-path [PATH] 'Specify location of manifest file'")
                .default_value("Cargo.toml")
                .validator(is_file),
        ])
        .get_matches()
}

fn main() {
    let m = parse_cli();

    let cfg = Config::from_matches(&m).unwrap_or_else(|e| e.exit());

    execute(cfg).map_err(|e| e.exit()).unwrap();
}

fn execute(cfg: Config) -> CliResult<()> {
    // Check the manifest file name.
    let manifest_path = PathBuf::from(&cfg.manifest_path);
    if let Some(file_name) = manifest_path.file_name() {
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

    // Search through parent dirs for Cargo.toml.
    let manifest_path = util::find_manifest_file(&manifest_path)?;

    // Cargo.lock must be in the same directory as Cargo.toml or in a parent directory.
    let manifest = manifest_path.to_str().unwrap();
    let lock_file = format!("{}.lock", &manifest[0..manifest.len() - 5]);
    let lock_path = util::find_manifest_file(&PathBuf::from(lock_file))?;

    // Graph the project.
    let dot_file = cfg.dot_file.clone();
    let project = Project::with_config(cfg)?;
    let (graph, root_deps_map) = project.graph(manifest_path, lock_path)?;

    // Render the dot file.
    match dot_file {
        None => {
            let o = io::stdout();
            let mut bw = BufWriter::new(o.lock());
            graph.render_to(&mut bw, &root_deps_map)
        }
        Some(file) => {
            let o = File::create(&Path::new(&file)).expect("Failed to create file");
            let mut bw = BufWriter::new(o);
            graph.render_to(&mut bw, &root_deps_map)
        }
    }
}

fn is_file(s: String) -> Result<(), String> {
    let p = Path::new(&*s);
    if p.file_name().is_none() {
        return Err(format!("'{}' doesn't appear to be a valid file name", &*s));
    }
    Ok(())
}
