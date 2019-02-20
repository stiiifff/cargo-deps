use crate::error::{CliErrorKind, CliResult};
use std::env;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use toml::{self, Value};

pub fn toml_from_file<P: AsRef<Path>>(p: P) -> CliResult<Value> {
    let mut f = File::open(p.as_ref())?;

    let mut s = String::new();
    f.read_to_string(&mut s)?;

    Ok(Value::try_from(s)?)
}

pub fn find_manifest_file(file: &str) -> CliResult<PathBuf> {
    let mut pwd = env::current_dir()?;

    loop {
        let manifest = pwd.join(file);
        if let Ok(metadata) = fs::metadata(&manifest) {
            if metadata.is_file() {
                return Ok(manifest);
            }
        }

        let pwd2 = pwd.clone();
        let parent = pwd2.parent();
        if parent.is_none() {
            break;
        }
        pwd = parent.unwrap().to_path_buf();
    }

    Err(From::from(CliErrorKind::Generic(format!(
        "Could not find `{}` in `{}` or any \
         parent directory, or it isn't a valid \
         lock-file",
        file,
        pwd.display()
    ))))
}
