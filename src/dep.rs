use crate::config::Config;
use std::io::{Result, Write};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DepKind {
    Regular,
    Build,
    Dev,
    Optional,
    Unknown,
}

#[derive(Debug)]
pub struct DeclaredDep {
    pub name: String,
    pub kind: DepKind,
}

impl DeclaredDep {
    pub fn with_kind(name: String, kind: DepKind) -> Self {
        DeclaredDep { name, kind }
    }
}

#[derive(Debug, PartialEq)]
pub struct ResolvedDep {
    pub name: String,
    pub ver: String,
    pub force_write_ver: bool,

    pub is_regular: bool,
    pub is_build: bool,
    pub is_dev: bool,
    pub is_optional: bool,
}

impl ResolvedDep {
    pub fn new(name: String, ver: String) -> Self {
        ResolvedDep {
            name,
            ver,
            force_write_ver: false,

            is_regular: false,
            is_build: false,
            is_dev: false,
            is_optional: false,
        }
    }

    pub fn kind(&self) -> DepKind {
        if self.is_regular {
            DepKind::Regular
        } else if self.is_build {
            DepKind::Build
        } else if self.is_dev {
            DepKind::Dev
        } else if self.is_optional {
            DepKind::Optional
        } else {
            DepKind::Unknown
        }
    }

    pub fn label<W: Write>(&self, w: &mut W, cfg: &Config, i: usize) -> Result<()> {
        let name = if self.force_write_ver || cfg.include_vers {
            format!("{} v{}", self.name, self.ver)
        } else {
            self.name.clone()
        };

        let shape = if i == 0 { ", shape=box" } else { "" };

        match self.kind() {
            DepKind::Regular => writeln!(w, " [label=\"{}\"{}];", name, shape),
            DepKind::Build => writeln!(w, " [label=\"{}\", color=purple];", name),
            DepKind::Dev => writeln!(w, " [label=\"{}\", color=blue];", name),
            DepKind::Optional => writeln!(w, " [label=\"{}\", color=red];", name),
            _ => writeln!(w, " [label=\"{}\", color=orange];", name),
        }
    }
}
