use crate::graph::{DepGraph, Node};
use std::io::{Result, Write};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DepKind {
    Regular,
    Build,
    Dev,
    Optional,
    Unknown,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RootCrate {
    pub name: String,
    pub ver: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedDep {
    pub name: String,
    pub ver: String,
    pub force_write_ver: bool,

    pub is_regular: bool,
    pub is_build: bool,
    pub is_dev: bool,
    pub is_optional: bool,

    pub children: Vec<Node>,
    pub parents: Vec<Node>,
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

            children: vec![],
            parents: vec![],
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

    pub fn label<W: Write>(&self, w: &mut W, dg: &DepGraph) -> Result<()> {
        let name = if self.force_write_ver || dg.cfg.include_vers {
            format!("{} v{}", self.name, self.ver)
        } else {
            self.name.clone()
        };

        let shape = if dg.root_deps_map.contains_key(&self.name) {
            ", shape=box"
        } else {
            ""
        };

        match self.kind() {
            DepKind::Regular => writeln!(w, " [label=\"{}\"{}];", name, shape),
            DepKind::Build => writeln!(w, " [label=\"{}\", color=purple];", name),
            DepKind::Dev => writeln!(w, " [label=\"{}\", color=blue];", name),
            DepKind::Optional => writeln!(w, " [label=\"{}\", color=red];", name),
            _ => writeln!(w, " [label=\"{}\", color=orange];", name),
        }
    }
}
