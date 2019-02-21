use std::fmt;
use std::io::{self, Write};

use crate::config::Config;
use crate::dep::ResolvedDep;
use crate::error::CliResult;

pub type Node = usize;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub struct Edge(pub Node, pub Node);

impl Edge {
    pub fn label<W: Write>(&self, w: &mut W, dg: &DepGraph) -> io::Result<()> {
        use crate::dep::DepKind::{Build, Dev, Optional};

        let parent = dg.get(self.0).unwrap().kind();
        let child = dg.get(self.1).unwrap().kind();

        match (parent, child) {
            (_, Build) => writeln!(w, "[label=\"\",color=black,style=dashed];"),
            (_, Dev) => writeln!(w, "[label=\"\",color=red,style=dashed];"),
            (_, Optional) => writeln!(w, "[label=\"\",color=orange,style=dotted];"),
            _ => writeln!(w, "[label=\"\"];"),
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
pub struct DepGraph<'c, 'o>
where
    'o: 'c,
{
    pub nodes: Vec<ResolvedDep>,
    pub edges: Vec<Edge>,
    cfg: &'c Config<'o>,
}

impl<'c, 'o> DepGraph<'c, 'o> {
    pub fn new(cfg: &'c Config<'o>) -> Self {
        DepGraph {
            nodes: vec![],
            edges: vec![],
            cfg,
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
            dep.label(output, self.cfg)?;
        }
        for ed in &self.edges {
            write!(output, "\t{}", ed)?;
            ed.label(output, &self)?;
        }
        writeln!(output, "}}")?;
        Ok(())
    }
}
