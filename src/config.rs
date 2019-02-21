use crate::error::CliResult;
use clap::ArgMatches;

#[derive(Clone, Debug)]
pub struct Config {
    pub dot_file: Option<String>,
    pub include_vers: bool,
    pub manifest_path: String,
    pub no_color: bool,

    pub build_deps: bool,
    pub dev_deps: bool,
    pub optional_deps: bool,
}

impl Config {
    pub fn from_matches(m: &ArgMatches) -> CliResult<Self> {
        Ok(Config {
            dot_file: m.value_of("dot-file").map(|s| s.into()),
            include_vers: m.is_present("include-versions"),
            manifest_path: m.value_of("manifest-path").unwrap().into(),
            no_color: m.is_present("no_color"),

            build_deps: !m.is_present("no-build-deps"),
            dev_deps: m.is_present("dev-deps"),
            optional_deps: m.is_present("optional-deps"),
        })
    }
}
