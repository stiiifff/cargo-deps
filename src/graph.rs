use crate::config::Config;
use crate::dep::{DeclaredDep, DepKind, ResolvedDep};
use crate::error::CliResult;
use std::collections::HashMap;
use std::fmt;
use std::io::{self, Write};

pub type Node = usize;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub struct Edge(pub Node, pub Node);

impl Edge {
    pub fn label<W: Write>(&self, w: &mut W, dg: &DepGraph) -> io::Result<()> {
        use crate::dep::DepKind::{Build, Dev, Optional};

        let parent = dg.get(self.0).unwrap().kind();
        let child = dg.get(self.1).unwrap().kind();

        match (parent, child) {
            (Build, Build) => writeln!(w, "[label=\"\"];"),
            (Dev, _) | (Build, Dev) => writeln!(w, "[label=\"\",color=blue,style=dashed];"),
            (Optional, _) | (Build, Optional) => {
                writeln!(w, "[label=\"\",color=red,style=dashed];")
            }
            _ => writeln!(w, "[label=\"\",color=purple,style=dashed];"),
        }
    }
}

impl fmt::Display for Edge {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let &Edge(il, ir) = self;
        write!(f, "N{} -> N{}", il, ir)
    }
}

#[derive(Debug)]
pub struct DepGraph {
    pub nodes: Vec<ResolvedDep>,
    pub edges: Vec<Edge>,
    cfg: Config,
}

impl DepGraph {
    pub fn new(cfg: Config) -> Self {
        DepGraph {
            nodes: vec![],
            edges: vec![],
            cfg,
        }
    }

    /// Sets the kind of dependency on each dependency based on how the dependencies are declared in
    /// the manifest.
    pub fn set_resolved_kind(&mut self, declared_deps: &[DeclaredDep]) {
        let declared_deps_map = declared_deps
            .iter()
            .map(|dd| (&*dd.name, dd.kind))
            .collect::<HashMap<_, _>>();

        self.nodes[0].is_build = true;

        // Make sure to process edges from the root node first.
        // Sorts by ID of first node first, then by second node.
        self.edges.sort();

        // FIXME: We repeat the following step several times to ensure that the kinds are propogated
        // to all nodes. The surefire way to handle this would be to do a proper topological sort.
        for _ in 0..10 {
            for ed in self.edges.iter() {
                if ed.0 == 0 {
                    // If this is an edge from the root node,
                    // set the kind based on how the dependency is declared in the manifest file.
                    if let Some(kind) = declared_deps_map.get(&*self.nodes[ed.1].name) {
                        match *kind {
                            DepKind::Build => self.nodes[ed.1].is_build = true,
                            DepKind::Dev => self.nodes[ed.1].is_dev = true,
                            DepKind::Optional => self.nodes[ed.1].is_optional = true,
                            _ => (),
                        }
                    }
                } else {
                    // If this is an edge from a dependency node, propagate the kind. This is a set of
                    // flags because a dependency can appear several times in the graph, and the kind of
                    // dependency may vary based on the path to that dependency. The flags start at
                    // false, and once they become true, they stay true. ResolvedDep::kind() will pick a
                    // kind based on their priority.

                    if self.nodes[ed.0].is_build {
                        self.nodes[ed.1].is_build = true;
                    }

                    if self.nodes[ed.0].is_dev {
                        self.nodes[ed.1].is_dev = true;
                    }

                    if self.nodes[ed.0].is_optional {
                        self.nodes[ed.1].is_optional = true;
                    }
                }
            }
        }

        // Remove the nodes that the user doesn't want.
        // Start at 1 to keep the root node.
        for id in (1..self.nodes.len()).rev() {
            let kind = self.nodes[id].kind();
            if (kind == DepKind::Build && !self.cfg.build_deps)
                || (kind == DepKind::Dev && !self.cfg.dev_deps)
                || (kind == DepKind::Optional && !self.cfg.optional_deps)
            {
                self.remove(id);
            }
        }

        self.remove_orphans();
    }

    /// Forces the version to be displayed on dependencies that have the same name (but a different
    /// version) as another dependency.
    pub fn show_version_on_duplicates(&mut self) {
        // Build a list of node IDs, sorted by the name of the dependency on that node.
        let dep_ids_sorted_by_name = {
            let mut deps = self.nodes.iter().enumerate().collect::<Vec<_>>();
            deps.sort_by_key(|dep| &*dep.1.name);
            deps.iter().map(|dep| dep.0).collect::<Vec<_>>()
        };

        for (i, &dep_id_i) in dep_ids_sorted_by_name
            .iter()
            .enumerate()
            .take(dep_ids_sorted_by_name.len() - 1)
        {
            // Find other nodes with the same name.
            // We need to iterate one more time after the last node to handle the break.
            for (j, &dep) in dep_ids_sorted_by_name
                .iter()
                .enumerate()
                .take(dep_ids_sorted_by_name.len() + 1)
                .skip(i + 1)
            {
                // Stop once we've found a node with a different name or reached the end of the
                // list.
                if j >= dep_ids_sorted_by_name.len()
                    || self.nodes[dep_id_i].name != self.nodes[dep].name
                {
                    // If there are at least two nodes with the same name
                    if j >= i + 2 {
                        // Set force_write_ver = true on all nodes
                        // from dep_ids_sorted_by_name[i] to dep_ids_sorted_by_name[j - 1].
                        // Remember: j is pointing on the next node with a *different* name!
                        // Remember also: i..j includes i but excludes j.
                        for &dep_id_k in dep_ids_sorted_by_name.iter().take(j).skip(i) {
                            self.nodes[dep_id_k].force_write_ver = true;
                        }
                    }

                    break;
                }
            }
        }
    }

    pub fn add_child(&mut self, parent: usize, dep_name: &str, dep_ver: &str) -> usize {
        let idr = self.find_or_add(dep_name, dep_ver);
        self.edges.push(Edge(parent, idr));
        idr
    }

    pub fn get(&self, id: usize) -> Option<&ResolvedDep> {
        if id < self.nodes.len() {
            return Some(&self.nodes[id]);
        }
        None
    }

    pub fn remove(&mut self, id: usize) {
        self.nodes.remove(id);
        // Remove edges of the removed node.
        self.edges = self
            .edges
            .iter()
            .filter(|e| !(e.0 == id || e.1 == id))
            .cloned()
            .collect();
        self.shift_edges_after_node(id);
    }

    fn shift_edges_after_node(&mut self, id: usize) {
        enum Side {
            Left,
            Right,
        }
        let mut to_upd = vec![];
        for c in id..self.nodes.len() {
            for (eid, &Edge(idl, idr)) in self.edges.iter().enumerate() {
                if idl == c {
                    to_upd.push((eid, Side::Left, c - 1));
                }
                if idr == c {
                    to_upd.push((eid, Side::Right, c - 1));
                }
            }
        }
        for (eid, side, new) in to_upd {
            match side {
                Side::Left => self.edges[eid].0 = new,
                Side::Right => self.edges[eid].1 = new,
            }
        }
    }

    pub fn remove_orphans(&mut self) {
        let len = self.nodes.len();
        self.edges.retain(|&Edge(idl, idr)| idl < len && idr < len);
        loop {
            let mut removed = false;
            let mut used = vec![false; self.nodes.len()];
            used[0] = true;
            for &Edge(_, idr) in &self.edges {
                used[idr] = true;
            }

            for (id, &u) in used.iter().enumerate() {
                if !u {
                    self.nodes.remove(id);

                    // Remove edges originating from the removed node
                    self.edges.retain(|&Edge(origin, _)| origin != id);
                    // Adjust edges to match the new node indexes
                    for edge in self.edges.iter_mut() {
                        if edge.0 > id {
                            edge.0 -= 1;
                        }
                        if edge.1 > id {
                            edge.1 -= 1;
                        }
                    }
                    removed = true;
                    break;
                }
            }
            if !removed {
                break;
            }
        }
    }

    fn remove_self_pointing(&mut self) {
        loop {
            let mut found = false;
            let mut self_p = vec![false; self.edges.len()];
            for (eid, &Edge(idl, idr)) in self.edges.iter().enumerate() {
                if idl == idr {
                    found = true;
                    self_p[eid] = true;
                    break;
                }
            }

            for (id, &u) in self_p.iter().enumerate() {
                if u {
                    self.edges.remove(id);
                    break;
                }
            }
            if !found {
                break;
            }
        }
    }

    pub fn set_root(&mut self, name: &str, ver: &str) -> bool {
        let root_id = if let Some(i) = self.find(name, ver) {
            i
        } else {
            return false;
        };

        if root_id == 0 {
            return true;
        }

        // Swap with 0
        self.nodes.swap(0, root_id);

        // Adjust edges
        for edge in self.edges.iter_mut() {
            if edge.0 == 0 {
                edge.0 = root_id;
            } else if edge.0 == root_id {
                edge.0 = 0;
            }
            if edge.1 == 0 {
                edge.1 = root_id;
            } else if edge.1 == root_id {
                edge.1 = 0;
            }
        }
        true
    }

    pub fn find(&self, name: &str, ver: &str) -> Option<usize> {
        for (i, d) in self.nodes.iter().enumerate() {
            if d.name == name && d.ver == ver {
                return Some(i);
            }
        }
        None
    }

    pub fn find_or_add(&mut self, name: &str, ver: &str) -> usize {
        if let Some(i) = self.find(name, ver) {
            return i;
        }
        self.nodes
            .push(ResolvedDep::new(name.to_owned(), ver.to_owned()));
        self.nodes.len() - 1
    }

    pub fn render_to<W: Write>(mut self, output: &mut W) -> CliResult<()> {
        self.edges.sort();
        self.edges.dedup();
        self.remove_orphans();
        self.remove_self_pointing();
        writeln!(output, "digraph dependencies {{")?;
        for (i, dep) in self.nodes.iter().enumerate() {
            write!(output, "\tN{}", i)?;
            dep.label(output, &self.cfg)?;
        }
        for ed in &self.edges {
            write!(output, "\t{}", ed)?;
            ed.label(output, &self)?;
        }
        writeln!(output, "}}")?;
        Ok(())
    }
}
