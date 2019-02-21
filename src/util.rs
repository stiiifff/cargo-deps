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

pub fn find_manifest_file(file: &PathBuf) -> CliResult<PathBuf> {
    let pwd = env::current_dir()?;
    let manifest = pwd.join(file);
    let file_name = manifest.file_name().unwrap();
    let mut dir = manifest.parent().unwrap().to_path_buf();

    loop {
        let manifest = dir.join(file_name);

        if let Ok(metadata) = fs::metadata(&manifest) {
            if metadata.is_file() {
                return Ok(manifest);
            }
        }

        dir = match dir.parent() {
            None => {
                return Err(CliError::Generic(format!(
                    "Could not find `{}` in `{}` or any \
                     parent directory",
                    file.to_str().unwrap(),
                    pwd.display()
                )));
            }
            Some(ref dir) => dir.to_path_buf(),
        };
    }
}
