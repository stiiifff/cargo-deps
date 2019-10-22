extern crate cargo_deps;

use cargo_deps::{get_dep_graph, render_dep_graph, Config};

// Note: these are really just smoke tests to ensure we can use cargo-deps as a lib

#[test]
fn get_dep_graph_self() {
    let cfg = Config::default();
    let graph = get_dep_graph(cfg).unwrap();
    assert!(graph.nodes.iter().any(|d| d.name == "clap"));
    assert!(graph.nodes.iter().any(|d| d.name == "toml"));
}

// TODO: remove this and uncomment the next occurrence.
#[rustfmt::skip]
#[test]
fn render_dep_graph_self() {
    let mut cfg = Config::default();
    cfg.depth = Some(1);
    let graph = get_dep_graph(cfg).unwrap();
    let out = render_dep_graph(graph).unwrap();
    assert_eq!(
        out,
        // #[rustfmt::skip]
        "digraph dependencies {\n\
         \tn6 [label=\"cargo-deps\", shape=box];\n\
         \tn7 [label=\"clap\"];\n\tn8 [label=\"toml\"];\n\n\
         \tn6 -> n7;\n\
         \tn6 -> n8;\n\
         }\n"
    );
}
