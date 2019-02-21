use crate::error::{CliError, CliResult};
use std::env;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use toml::{self, Value};

pub fn toml_from_file<P: AsRef<Path>>(p: P) -> CliResult<Value> {
    let mut f = File::open(p.as_ref())?;

    let mut s = String::new();
    f.read_to_string(&mut s)?;

    let toml: Value = toml::from_str(&s)?;
    Ok(toml)
}

pub fn find_manifest_file(file: &str) -> CliResult<PathBuf> {
    let pwd = env::current_dir()?;
    let mut dir = pwd.clone();

    loop {
        let manifest = dir.join(file);
        if let Ok(metadata) = fs::metadata(&manifest) {
            if metadata.is_file() {
                return Ok(manifest);
            }
        }

        let parent = dir.parent();
        if parent.is_none() {
            break;
        }
        dir = parent.unwrap().to_path_buf();
    }

    Err(CliError::Generic(format!(
        "Could not find `{}` in `{}` or any \
         parent directory",
        file,
        pwd.display()
    )))
}
