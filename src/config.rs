use crate::error::CliResult;
use clap::ArgMatches;

#[derive(Clone, Debug)]
pub struct Config {
    pub dot_file: Option<String>,
    pub filter: Option<Vec<String>>,
    pub include_vers: bool,
    pub manifest_path: String,

    pub regular_deps: bool,
    pub build_deps: bool,
    pub dev_deps: bool,
    pub optional_deps: bool,
}

impl Config {
    pub fn from_matches(m: &ArgMatches) -> CliResult<Self> {
        let all_deps = m.is_present("all-deps");

        Ok(Config {
            dot_file: m.value_of("dot-file").map(|s| s.into()),
            filter: m
                .values_of("filter")
                .map(|deps| deps.map(|dep| dep.into()).collect()),
            include_vers: m.is_present("include-versions"),
            manifest_path: m.value_of("manifest-path").unwrap().into(),

            regular_deps: !m.is_present("no-regular-deps"),
            build_deps: all_deps || m.is_present("build-deps"),
            dev_deps: all_deps || m.is_present("dev-deps"),
            optional_deps: all_deps || m.is_present("optional-deps"),
        })
    }
}
