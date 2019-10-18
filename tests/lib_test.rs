extern crate cargo_deps;

use cargo_deps::{get_dep_graph, render_dep_graph, Config};

// Note: these are really just smoke tests to ensure we can use cargo-deps as a lib

#[test]
fn get_dep_graph_self() {
    let mut cfg = Config::default();
    cfg.manifest_path = "../Cargo.toml".to_string();
    let graph = get_dep_graph(cfg).unwrap();
    assert!(graph.nodes.iter().any(|d| d.name == "clap"));
    assert!(graph.nodes.iter().any(|d| d.name == "toml"));
}

#[test]
fn render_dep_graph_self() {
    let mut cfg = Config::default();
    cfg.manifest_path = "../Cargo.toml".to_string();
    let graph = get_dep_graph(cfg).unwrap();
    let out = render_dep_graph(graph).unwrap();
    assert!(out.starts_with("digraph dependencies"));
}
