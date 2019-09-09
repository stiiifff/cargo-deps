#![forbid(unsafe_code)]

#[macro_use]
extern crate clap;
extern crate toml;

mod config;
mod dep;
mod error;
mod graph;
mod project;
mod util;

use crate::{
    config::Config,
    error::{CliError, CliResult},
    project::Project,
};
use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use std::{
    fs::File,
    io::{self, BufWriter},
    path::Path,
    str::FromStr,
};

const USAGE: &str = "\
cargo-deps writes a graph in dot format to standard output.

    Typical usage is `cargo deps | dot -Tpng > graph.png`.";

// TODO: remove this and uncomment the next occurrence.
#[rustfmt::skip]
fn parse_cli<'a>() -> ArgMatches<'a> {
    App::new("cargo-deps")
        .version(crate_version!())
        .bin_name("cargo")
        .settings(&[AppSettings::GlobalVersion, AppSettings::SubcommandRequired])
        .global_setting(AppSettings::ColoredHelp)
        .subcommand(
            SubCommand::with_name("deps")
                .author(crate_authors!())
                .about(crate_description!())
                .usage(USAGE)
                .args_from_usage(
                    // #[rustfmt::skip]
                    "
                    -o --dot-file [PATH] 'Output file, or stdout if not specified'
                       --filter [DEPNAMES] ... 'Only display provided deps'
                       --include-orphans 'Don't purge orphan nodes (yellow). \
                                          This is useful in some workspaces'
                    -I --include-versions 'Include the dependency version on nodes'
                       --subgraph [DEPNAMES] ... 'Group provided deps in their own subgraph'

                      --all-deps 'Include all dependencies in the graph. \
                                  Can be used with --no-regular-deps'
                       --no-regular-deps 'Exclude regular dependencies from the graph'
                       --build-deps 'Include build dependencies in the graph (purple)'
                       --dev-deps 'Include dev dependencies in the graph (blue)'
                       --optional-deps 'Include optional dependencies in the graph (red)'
                       --no-transitive-deps 'Filter out edges that point to a transitive \
                                             dependency'
                    ",
                )
                .args(&[
                    Arg::from_usage("-d --depth [DEPTH] 'The maximum dependency depth to display. \
                                                         The default is no limit'")
                        .validator(|v| usize::from_str(&v)
                                   .map(|_| ())
                                   .map_err(|e| format!("{}: {}", v, e))
                        ),
                    Arg::from_usage("--manifest-path [PATH] 'Specify location of manifest file'")
                        .default_value("Cargo.toml"),
                    Arg::from_usage("--subgraph-name [NAME] 'Optional name of subgraph'")
                        .requires("subgraph"),
                ]),
        )
        .get_matches()
}

fn main() {
    let m = parse_cli();

    if let Some(m) = m.subcommand_matches("deps") {
        let cfg = Config::from_matches(&m).unwrap_or_else(|e| e.exit());
        execute(cfg).map_err(|e| e.exit()).unwrap();
    }
}

fn execute(cfg: Config) -> CliResult<()> {
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
