use crate::error::{Error, Result};
use std::{
    env,
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
};
use toml::{self, Value};

pub fn toml_from_file<P: AsRef<Path>>(p: P) -> Result<Value> {
    let mut f = File::open(p.as_ref())?;

    let mut s = String::new();
    f.read_to_string(&mut s)?;

    let toml: Value = toml::from_str(&s)?;
    Ok(toml)
}

// TODO: replace with `find_root_manifest_for_wd` in the `cargo` crate?
pub fn find_file_search_parent_dirs(file: &str) -> Result<PathBuf> {
    let pwd = env::current_dir()?;
    let input_manifest_path = pwd.join(file);
    let file_name = input_manifest_path.file_name().unwrap();
    // Canonicalize the directory to get rid of things like `..`.
    let mut current_dir = input_manifest_path.parent().unwrap().to_path_buf();
    current_dir = current_dir
        .canonicalize()
        .map_err(|e| Error::Generic(format!("Could not canonicalize {:?}: {}", current_dir, e)))?;
    let mut first_try = true;

    loop {
        let try_manifest = current_dir.join(file_name);

        if let Ok(metadata) = fs::metadata(&try_manifest) {
            if metadata.is_file() {
                if !first_try {
                    eprintln!("Found {:?} in {:?}.", file_name, current_dir.display());
                }

                return Ok(try_manifest);
            }
        }

        if first_try {
            eprintln!(
                "Could not find {:?} in {:?}, searching parent directories.",
                file_name,
                current_dir.display()
            );
            first_try = false;
        }

        current_dir = match current_dir.parent() {
            None => {
                return Err(Error::Generic(format!(
                    "Could not find {:?} in {:?} or any parent directory",
                    file, pwd,
                )));
            }
            Some(ref dir) => dir.to_path_buf(),
        };
    }
}
