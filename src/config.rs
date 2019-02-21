use crate::error::CliResult;
use clap::ArgMatches;

#[derive(Debug)]
pub struct Config<'a> {
    pub dot_file: Option<&'a str>,
    pub include_vers: bool,
    pub lock_file: &'a str,
    pub manifest_file: &'a str,

    pub build_deps: bool,
    pub dev_deps: bool,
    pub optional_deps: bool,
}

impl<'a> Config<'a> {
    pub fn from_matches(m: &'a ArgMatches) -> CliResult<Self> {
        Ok(Config {
            dot_file: m.value_of("dot-file"),
            include_vers: m.is_present("include-versions"),
            lock_file: m.value_of("lock-file").unwrap(),
            manifest_file: m.value_of("manifest-file").unwrap(),

            build_deps: m.is_present("build-deps"),
            dev_deps: m.is_present("dev-deps"),
            optional_deps: m.is_present("optional-deps"),
        })
    }
}
