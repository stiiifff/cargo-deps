#![forbid(unsafe_code)]

#[macro_use]
extern crate clap;

use std::{
    fs::File,
    io::{self, Write},
    path::Path,
    str::FromStr,
};

use cargo_deps::{get_dep_graph, render_dep_graph, Config};
use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

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
    let args = parse_cli();

    if let Some(arg) = args.subcommand_matches("deps") {
        let cfg = Config::from_matches(&arg).unwrap_or_else(|e| e.exit());
        let dot_file = cfg.dot_file.clone();

        // Get dependency graph & render it
        let out = get_dep_graph(cfg)
            .and_then(render_dep_graph)
            .map_err(|e| e.exit())
            .unwrap();

        // Output to stdout or render the dot file
        match dot_file {
            None => Box::new(io::stdout()) as Box<dyn Write>,
            Some(file) => Box::new(File::create(&Path::new(&file)).expect("Failed to create file")),
        }
        .write_all(&out.into_bytes())
        .expect("Unable to write graph");
    }
}
