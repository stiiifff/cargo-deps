use crate::{
    config::Config,
    dep::{DepKind, ResolvedDep},
    error::{CliError, CliResult},
    project::RootDepsMap,
};
use std::{collections::HashMap, fmt, io::Write};

pub type Node = usize;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub struct Edge(pub Node, pub Node);

impl Edge {
    pub fn label<W: Write>(&self, w: &mut W, dg: &DepGraph) -> CliResult<()> {
        use crate::dep::DepKind::{Build, Dev, Optional, Regular, Unknown};

        let parent = dg.get(self.0).unwrap();
        let child = dg.get(self.1).unwrap();

        // Special case: always color edge from root to root dep by its actual root dependency kind.
        // Otherwise, the root dep could also be a dep of a regular dep which will cause the root ->
        // root dep edge to appear regular, which is misleading as it is not regular in Cargo.toml.
        let child_kind = if let Some(dep_kinds_map) = &dg.root_deps_map.get(&parent.name) {
            if let Some(kinds) = dep_kinds_map.get(&child.name) {
                if kinds.contains(&Regular) {
                    Regular
                } else if kinds.contains(&Build) {
                    Build
                } else if kinds.contains(&Dev) {
                    Dev
                } else if kinds.contains(&Optional) {
                    Optional
                } else {
                    Unknown
                }
            } else {
                return Err(CliError::Generic(format!(
                    "Crate '{}' is not a dependency of a root crate. This is probably a logic \
                     error.",
                    child.name
                )));
            }
        } else {
            child.kind()
        };

        match (parent.kind(), child_kind) {
            (Regular, Regular) => writeln!(w, ";")?,
            (Build, _) | (Regular, Build) => writeln!(w, " [color=purple, style=dashed];")?,
            (Dev, _) | (Regular, Dev) => writeln!(w, " [color=blue, style=dashed];")?,
            (Optional, _) | (Regular, Optional) => writeln!(w, " [color=red, style=dashed];")?,
            _ => writeln!(w, " [color=orange, style=dashed];")?,
        }

        Ok(())
    }
}

impl fmt::Display for Edge {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let &Edge(il, ir) = self;
        write!(f, "n{} -> n{}", il, ir)
    }
}

#[derive(Debug)]
pub struct DepGraph {
    /// Vector of nodes containing resolved dependency information as well as the indices of parent
    /// and children nodes.
    pub nodes: Vec<ResolvedDep>,
    pub edges: Vec<Edge>,
    pub root_deps_map: RootDepsMap,
    pub cfg: Config,
}

impl DepGraph {
    pub fn new(cfg: Config) -> Self {
        DepGraph {
            nodes: vec![],
            edges: vec![],
            root_deps_map: HashMap::new(),
            cfg,
        }
    }

    /// Performs a topological sort on the edges.
    pub fn topological_sort(&mut self) -> CliResult<()> {
        let mut graph_nodes = self.nodes.clone();
        let mut l: Vec<Node> = vec![]; // Will contain indices of the nodes in sorted order.
        let mut s: Vec<Node> = vec![]; // Set of nodes with no incoming edges.

        // Populate initial list of start nodes which have no incoming edges.
        for (i, node) in self.nodes.iter().enumerate() {
            if node.parents.is_empty() {
                s.push(i);
            }
        }

        while let Some(n) = s.pop() {
            l.push(n);

            while let Some(child) = graph_nodes[n].children.pop() {
                assert_ne!(n, child);

                // Remove the edge from n -> child.
                let e_index = self
                    .edges
                    .iter()
                    .position(|Edge(a, b)| a == &n && b == &child)
                    .unwrap();
                self.edges.remove(e_index);
                let n_index = graph_nodes[child]
                    .parents
                    .iter()
                    .position(|node| *node == n)
                    .unwrap();
                graph_nodes[child].parents.remove(n_index);

                // If child has no other parents, it is in the next topological level.
                if graph_nodes[child].parents.is_empty() {
                    s.push(child);
                }
            }
        }

        if self.edges.is_empty() {
            // Add back the edges, this time in topological order.
            for n in l.iter() {
                'child_loop: for child in self.nodes[*n].children.iter() {
                    // push an edge for each child, unless filtering of
                    // transitive deps is enabled, in which case skip to the
                    // next child if a transitive dependency exists
                    if !self.cfg.transitive_deps {
                        for c in self.nodes[*n].children.iter().filter(|c| *c != child) {
                            if self.transitive_dep(*c, *child) {
                                continue 'child_loop;
                            }
                        }
                    }
                    self.edges.push(Edge(*n, *child));
                }
            }

            Ok(())
        } else {
            Err(CliError::Generic(
                "Cycle detected in dependency graph".into(),
            ))
        }
    }

    /// Sets the kind of each dependency based on how the dependencies are declared in the manifest.
    pub fn set_resolved_kind(&mut self) -> CliResult<()> {
        // Set regular kind for all root nodes.
        for node in self.nodes.iter_mut() {
            if self.root_deps_map.contains_key(&node.name) {
                node.is_regular = true;
            }
        }

        // Iterate over edges in topologically-sorted order to propogate the kinds.
        for ed in self.edges.iter() {
            let (parent_name, parent_regular, parent_build, parent_dev, parent_optional) = {
                let parent = &self.nodes[ed.0];
                (
                    parent.name.to_string(),
                    parent.is_regular,
                    parent.is_build,
                    parent.is_dev,
                    parent.is_optional,
                )
            };
            let mut child = &mut self.nodes[ed.1];

            if let Some(dep_kinds_map) = self.root_deps_map.get(&parent_name) {
                // If this is an edge from the root node,
                // set the kind based on how the dependency is declared in the manifest file.
                if let Some(kinds) = dep_kinds_map.get(&child.name) {
                    for kind in kinds {
                        match *kind {
                            DepKind::Regular => child.is_regular = true,
                            DepKind::Build => child.is_build = true,
                            DepKind::Dev => child.is_dev = true,
                            DepKind::Optional => child.is_optional = true,
                            _ => (),
                        }
                    }
                } else {
                    return Err(CliError::Generic(format!(
                        "Crate '{}' is not a dependency of a root crate. This is probably a logic \
                         error.",
                        child.name
                    )));
                }
            } else {
                // If this is an edge from a dependency node, propagate the kind. This is a set
                // of flags because a dependency can appear several times in the graph, and the
                // kind of dependency may vary based on the path to that dependency. The flags
                // start at false, and once they become true, they stay true.
                // ResolvedDep::kind() will pick a kind based on their priority.

                if parent_regular {
                    child.is_regular = true;
                }
                if parent_build {
                    child.is_build = true;
                }
                if parent_dev {
                    child.is_dev = true;
                }
                if parent_optional {
                    child.is_optional = true;
                }
            }
        }

        Ok(())
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

    pub fn add_child(&mut self, parent: usize, dep_name: &str, dep_ver: &str) {
        let child = self.find_or_add(dep_name, dep_ver);

        if parent == child {
            return;
        }

        self.edges.push(Edge(parent, child));

        self.nodes[parent].children.push(child);
        self.nodes[child].parents.push(parent);
    }

    pub fn get(&self, id: usize) -> Option<&ResolvedDep> {
        if id < self.nodes.len() {
            return Some(&self.nodes[id]);
        }
        None
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

    pub fn render_to<W: Write>(self, output: &mut W) -> CliResult<()> {
        // Keep track of added nodes.
        let mut nodes_added = vec![];

        writeln!(output, "digraph dependencies {{")?;

        // Output all non-subgraph nodes.
        for (i, dep) in self.nodes.iter().enumerate() {
            // Skip subgraph nodes, will be declared in the subgraph.
            if let Some(sub_deps) = &self.cfg.subgraph {
                if sub_deps.contains(&dep.name) {
                    continue;
                }
            }

            // Skip orphan nodes.
            // Orphan nodes will still be output later if specified in a subgraph.
            if !self.cfg.include_orphans {
                if let DepKind::Unknown = dep.kind() {
                    continue;
                }
            }

            write!(output, "\tn{}", i)?;
            dep.label(output, &self)?;

            nodes_added.push(i);
        }
        writeln!(output)?;

        // Output any subgraph nodes.
        if let Some(sub_deps) = &self.cfg.subgraph {
            writeln!(output, "\tsubgraph cluster_subgraph {{")?;
            if let Some(sub_name) = &self.cfg.subgraph_name {
                writeln!(output, "\t\tlabel=\"{}\";", sub_name)?;
            }
            writeln!(output, "\t\tcolor=brown;")?;
            writeln!(output, "\t\tstyle=dashed;")?;
            writeln!(output)?;

            for (i, dep) in self.nodes.iter().enumerate() {
                if sub_deps.contains(&dep.name) {
                    write!(output, "\t\tn{}", i)?;
                    dep.label(output, &self)?;

                    nodes_added.push(i);
                }
            }

            writeln!(output, "\t}}\n")?;
        }

        // Output edges.
        for ed in &self.edges {
            // Only add edges if both nodes exist in the graph.
            if !(nodes_added.contains(&ed.0) && nodes_added.contains(&ed.1)) {
                continue;
            }

            write!(output, "\t{}", ed)?;
            ed.label(output, &self)?;
        }

        writeln!(output, "}}")?;

        Ok(())
    }

    // TODO: make this function more efficient with memoization:
    // ahead of time, generate a list of all dependencies (direct and indirect)
    // for all nodes, and then this function can simply check if child is in
    // parent's list of dependencies (or possibly remove this function
    // entirely)
    fn transitive_dep(&self, parent: usize, child: usize) -> bool {
        for c in self.nodes[parent].children.iter() {
            if *c == child || self.transitive_dep(*c, child) {
                return true;
            }
        }
        false
    }
}
