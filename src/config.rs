use crate::error::Result;
use clap::ArgMatches;
use std::str::FromStr;

/// Configuration options.
///
/// Create this object with `Default::default()` for the configuration equivalent to running without
/// any command-line arguments.
///
/// Please refer to the help menu for information about each option.
#[derive(Clone, Debug)]
#[allow(missing_docs)]
pub struct Config {
    pub depth: Option<usize>,
    pub dot_file: Option<String>,
    pub filter: Option<Vec<String>>,
    pub include_orphans: bool,
    pub include_versions: bool,
    /// Default: "Cargo.toml".
    pub manifest_path: String,
    pub subgraph: Option<Vec<String>>,
    pub subgraph_name: Option<String>,

    /// Default: true.
    pub regular_deps: bool,
    /// Default: false.
    pub build_deps: bool,
    /// Default: false.
    pub dev_deps: bool,
    /// Default: false.
    pub optional_deps: bool,
    /// Default: true.
    pub transitive_deps: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            depth: None,
            dot_file: None,
            filter: None,
            include_orphans: false,
            include_versions: false,
            manifest_path: "Cargo.toml".into(),
            subgraph: None,
            subgraph_name: None,

            regular_deps: true,
            build_deps: false,
            dev_deps: false,
            optional_deps: false,
            transitive_deps: true,
        }
    }
}

impl Config {
    /// Creates a config object from command line arguments.
    pub fn from_matches(m: &ArgMatches) -> Result<Self> {
        let all_deps = m.is_present("all-deps");

        Ok(Self {
            depth: m
                .value_of("depth")
                .map(|depth| usize::from_str(depth).unwrap()),
            dot_file: m.value_of("dot-file").map(|s| s.into()),
            filter: m
                .values_of("filter")
                .map(|deps| deps.map(|dep| dep.into()).collect()),
            include_orphans: m.is_present("include-orphans"),
            include_versions: m.is_present("include-versions"),
            manifest_path: m.value_of("manifest-path").unwrap_or("Cargo.toml").into(),
            subgraph: m
                .values_of("subgraph")
                .map(|deps| deps.map(|dep| dep.into()).collect()),
            subgraph_name: m.value_of("subgraph-name").map(|s| s.into()),

            regular_deps: !m.is_present("no-regular-deps"),
            build_deps: all_deps || m.is_present("build-deps"),
            dev_deps: all_deps || m.is_present("dev-deps"),
            optional_deps: all_deps || m.is_present("optional-deps"),
            transitive_deps: !m.is_present("no-transitive-deps"),
        })
    }
}
