//! `gitx graph` (docs/21 Stage 6, docs/02 V1 "stronger architecture graph"):
//! the shared HEAD-graph builder must emit deterministic file nodes,
//! directory containment, and call edges between files.

use gitx_git::Repository;
use petgraph::visit::EdgeRef;

#[path = "../common/mod.rs"]
mod common;
use common::FixtureRepo;

fn rust_fixture() -> Option<FixtureRepo> {
    let repo = FixtureRepo::new("graph")?;
    repo.write("src/lib.rs", "mod util;\nfn run() { util::helper(); }\n");
    repo.write("src/util.rs", "pub fn helper() {}\n");
    repo.commit("feat: modules");
    Some(repo)
}

#[test]
fn graph_emits_call_edges() {
    let Some(repo) = rust_fixture() else {
        eprintln!("skipping: git CLI unavailable");
        return;
    };
    let gix = Repository::discover(repo.path()).unwrap();
    let graph = gitx_graph::graph::build_head_code_graph(&gix).unwrap();

    let mut edges = Vec::new();
    for e in graph.graph.edge_references() {
        let from = graph.graph[e.source()].path.display().to_string();
        let to = graph.graph[e.target()].path.display().to_string();
        let kind = format!("{:?}", e.weight().edge_type).to_lowercase();
        edges.push((from, to, kind, e.weight().weight));
    }

    // lib.rs calls helper(), which lives in util.rs.
    assert!(
        edges
            .iter()
            .any(|(a, b, t, _)| t == "calls" && a.ends_with("lib.rs") && b.ends_with("util.rs")),
        "call edge lib.rs -> util.rs, got {edges:?}"
    );
    // No self-call edges (functions are only attributed to their own file).
    assert!(
        !edges.iter().any(|(a, b, t, _)| t == "calls" && a == b),
        "self call edges must not be emitted"
    );
}

#[test]
fn module_summary_counts_files_per_directory() {
    let Some(repo) = rust_fixture() else {
        eprintln!("skipping: git CLI unavailable");
        return;
    };
    let gix = Repository::discover(repo.path()).unwrap();
    let summary = gitx_graph::graph::module_summary(&gix).unwrap();

    let src = summary
        .iter()
        .find(|(d, _, _, _)| d == "src")
        .expect("src directory in summary");
    assert!(src.1 >= 2, "src contains lib.rs + util.rs, got {src:?}");
}
