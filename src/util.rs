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

pub fn find_manifest_file(file: &PathBuf, is_lock: bool) -> CliResult<PathBuf> {
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
        if is_lock || parent.is_none() {
            return Err(CliError::Generic(format!(
                "Could not find `{}` in `{}` or any \
                 parent directory",
                file.to_str().unwrap(),
                pwd.display()
            )));
        }
        dir = parent.unwrap().to_path_buf();
    }
}
