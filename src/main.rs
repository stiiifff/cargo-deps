#![deny(
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unused_import_braces,
    unused_qualifications
)]

extern crate toml;
#[macro_use]
extern crate clap;
#[cfg(feature = "color")]
extern crate ansi_term;

mod config;
mod dep;
mod error;
mod fmt;
mod graph;
mod project;
mod util;

use std::fs::File;
use std::io::{self, BufWriter};
use std::path::Path;

use clap::{App, Arg, ArgMatches};

use crate::config::Config;
use crate::error::CliResult;
use crate::project::Project;

static LINE_STYLES: [&'static str; 3] = ["solid", "dotted", "dashed"];
static COLORS: [&'static str; 8] = [
    "blue", "black", "yellow", "purple", "green", "red", "white", "orange",
];
static DEP_SHAPES: [&'static str; 4] = ["box", "round", "diamond", "triangle"];

fn parse_cli<'a>() -> ArgMatches<'a> {
    App::new("cargo-deps")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .args_from_usage("
                            -I, --include-versions 'Include the dependency version on nodes'
                                --dot-file [PATH] 'Output file (Default stdout)'
                                --dev-deps [true|false] 'Should dev deps be included in the graph? (Default false, also allows yes|no)'
                                --build-deps [true|false] 'Should build deps be in the graph? (Default true, also allows yes|no)'
                                --optional-deps [true|false] 'Should optional deps be in the graph? (Default true, also allows yes|no)'
                        ")
        .args(&[
            Arg::from_usage("--lock-file [PATH] 'Specify location of .lock file'")
                .default_value("Cargo.lock")
                .validator(is_file),
            Arg::from_usage("--manifest-file [PATH] 'Specify location of manifest file'")
                .default_value("Cargo.toml")
                .validator(is_file),
            Arg::from_usage("--build-line-style [STYLE] 'Line style for build deps'")
                .default_value("solid")
                .possible_values(&LINE_STYLES),
            Arg::from_usage("--build-line-color [COLOR] 'Line color for regular deps'")
                .default_value("black")
                .possible_values(&COLORS),
            Arg::from_usage("--build-shape [SHAPE] 'Shape for regular deps'")
                .default_value("round")
                .possible_values(&DEP_SHAPES),
            Arg::from_usage("--build-color [COLOR] 'Color for regular deps'")
                .default_value("black")
                .possible_values(&COLORS),
            Arg::from_usage("--optional-line-style [STYLE] 'Line style for optional deps'")
                .default_value("solid")
                .possible_values(&LINE_STYLES),
            Arg::from_usage("--optional-line-color [COLOR] 'Line color for optional deps'")
                .default_value("black")
                .possible_values(&COLORS),
            Arg::from_usage("--optional-shape [SHAPE] 'Shape for optional deps'")
                .default_value("round")
                .possible_values(&DEP_SHAPES),
            Arg::from_usage("--optional-color [COLOR] 'Color for optional deps'")
                .default_value("black")
                .possible_values(&COLORS),
            Arg::from_usage("--dev-line-style [STYLE] 'Line style for dev deps'")
                .default_value("solid")
                .possible_values(&LINE_STYLES),
            Arg::from_usage("--dev-line-color [COLOR] 'Line color for dev deps'")
                .default_value("black")
                .possible_values(&COLORS),
            Arg::from_usage("--dev-shape [SHAPE] 'Shape for dev deps'")
                .default_value("round")
                .possible_values(&DEP_SHAPES),
            Arg::from_usage("--dev-color [COLOR] 'Color for dev deps'")
                .default_value("black")
                .possible_values(&COLORS)])
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
