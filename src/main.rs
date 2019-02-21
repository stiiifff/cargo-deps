#![deny(
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unused_import_braces,
    unused_qualifications
)]

#[cfg(feature = "color")]
extern crate ansi_term;
#[macro_use]
extern crate clap;
extern crate toml;

mod config;
mod dep;
mod error;
mod fmt;
mod graph;
mod project;
mod util;

use crate::config::Config;
use crate::error::CliResult;
use crate::project::Project;
use clap::{App, Arg, ArgMatches};
use std::fs::File;
use std::io::{self, BufWriter};
use std::path::Path;

fn parse_cli<'a>() -> ArgMatches<'a> {
    App::new("cargo-deps")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .args_from_usage(
            "-I, --include-versions 'Include the dependency version on nodes'
                 --dot-file [PATH] 'Output file (Default stdout)'
                 --build-deps 'Should build deps be in the graph?'
                 --dev-deps 'Should dev deps be in the graph?'
                 --optional-deps 'Should optional deps be in the graph?'",
        )
        .args(&[
            Arg::from_usage("--lock-file [PATH] 'Specify location of .lock file'")
                .default_value("Cargo.lock")
                .validator(is_file),
            Arg::from_usage("--manifest-file [PATH] 'Specify location of manifest file'")
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
    let project = Project::with_config(&cfg)?;
    let graph = project.graph()?;

    match cfg.dot_file {
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

fn is_file(s: String) -> Result<(), String> {
    let p = Path::new(&*s);
    if p.file_name().is_none() {
        return Err(format!("'{}' doesn't appear to be a valid file name", &*s));
    }
    Ok(())
}
