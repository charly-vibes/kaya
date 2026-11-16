//! Contract tests for `kaya.projection` and the `research-index` projection:
//! managed-block grammar, byte preservation, deterministic rendering, and
//! incremental rebuild reporting.

use std::fs;
use std::path::Path;

use kaya::model::{EntityType, Graph, GraphBuilder};
use kaya::projection::index;
use kaya::projection::{BEGIN_PREFIX, Block, BlockKind, replace_generated, scan};
use serde_json::{Value, json};
use tempfile::tempdir;

const END_BLOCK: &str = "<!-- kaya:end generated:research-index -->";

fn graph_of(nodes: &[(&str, EntityType, Value)]) -> Graph {
    let mut builder = GraphBuilder::new();
    for (id, entity, attributes) in nodes {
        builder
            .entity(*id, *entity, attributes.clone())
            .expect("test node must be admitted");
    }
    builder.build().expect("test graph must build")
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).expect("projection file must be readable")
}

#[test]
fn authored_and_foreign_bytes_survive_recomputation() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("index.md");

    let before = "# Research\n\nIntro prose.\n\n\
        <!-- kaya:authored -->\n\
        Human interpretation.\n\
        <!-- kaya:end authored -->\n\n\
        <!-- wai:begin managed -->\n\
        foreign bytes\n\
        <!-- wai:end managed -->\n\n";
    let after = "\nTail prose.\n";
    let original =
        format!("{before}{BEGIN_PREFIX}research-index -->\n- stale line\n{END_BLOCK}{after}");
    fs::write(&path, &original).unwrap();

    let graph = graph_of(&[(
        "project/Question/Q-1",
        EntityType::Question,
        json!({"title": "Why?"}),
    )]);
    let summary = index::build(&graph, &path).unwrap();
    assert_eq!(summary.recomputed, vec!["research-index".to_owned()]);

    let rebuilt = read(&path);
    let begin = rebuilt.find(BEGIN_PREFIX).unwrap();
    let end = rebuilt.find(END_BLOCK).unwrap();

    assert_eq!(&rebuilt[..begin], before, "prefix bytes must not change");
    assert_eq!(
        &rebuilt[end + END_BLOCK.len()..],
        after,
        "suffix bytes must not change"
    );
    assert!(
        rebuilt
            .contains("<!-- kaya:authored -->\nHuman interpretation.\n<!-- kaya:end authored -->")
    );
    assert!(
        rebuilt.contains("<!-- wai:begin managed -->\nforeign bytes\n<!-- wai:end managed -->")
    );
    assert!(!rebuilt.contains("- stale line"));
}

#[test]
fn identical_graphs_build_byte_identical_files() {
    let dir = tempdir().unwrap();
    let first = dir.path().join("first.md");
    let second = dir.path().join("second.md");

    let graph = graph_of(&[
        (
            "p/Finding/F-1",
            EntityType::Finding,
            json!({"title": "It holds"}),
        ),
        ("p/Question/Q-1", EntityType::Question, json!({})),
        (
            "p/Experiment/E-1",
            EntityType::Experiment,
            json!({"title": "Try it"}),
        ),
    ]);

    index::build(&graph, &first).unwrap();
    index::build(&graph, &second).unwrap();

    assert_eq!(read(&first), read(&second));
}

#[test]
fn second_build_reports_unchanged_and_does_not_rewrite() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("index.md");
    let graph = graph_of(&[(
        "p/Decision/D-1",
        EntityType::Decision,
        json!({"title": "Choose A"}),
    )]);

    let first = index::build(&graph, &path).unwrap();
    assert_eq!(first.recomputed, vec!["research-index".to_owned()]);
    let bytes = read(&path);
    let modified = fs::metadata(&path).unwrap().modified().unwrap();

    let second = index::build(&graph, &path).unwrap();
    assert!(second.recomputed.is_empty());
    assert_eq!(second.unchanged, vec!["research-index".to_owned()]);
    assert_eq!(read(&path), bytes);
    assert_eq!(fs::metadata(&path).unwrap().modified().unwrap(), modified);
}

#[test]
fn new_node_recomputes_only_the_generated_block() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("index.md");

    let first_graph = graph_of(&[(
        "p/Question/Q-1",
        EntityType::Question,
        json!({"title": "First"}),
    )]);
    index::build(&first_graph, &path).unwrap();
    let first = read(&path);
    let begin = first.find(BEGIN_PREFIX).unwrap();
    let end = first.find(END_BLOCK).unwrap();
    let prefix = &first[..begin];
    let suffix = &first[end + END_BLOCK.len()..];

    let second_graph = graph_of(&[
        (
            "p/Question/Q-1",
            EntityType::Question,
            json!({"title": "First"}),
        ),
        (
            "p/Experiment/E-1",
            EntityType::Experiment,
            json!({"title": "Second"}),
        ),
    ]);
    let summary = index::build(&second_graph, &path).unwrap();
    assert_eq!(summary.recomputed, vec!["research-index".to_owned()]);
    assert!(summary.unchanged.is_empty());

    let second = read(&path);
    assert_ne!(first, second);
    assert!(second.starts_with(prefix));
    assert!(second.ends_with(suffix));
    assert!(second.contains("- p/Experiment/E-1 — Experiment — Second"));
    assert!(second.contains("- p/Question/Q-1 — Question — First"));
}

#[test]
fn unterminated_generated_block_refuses_naming_it_and_leaves_file_untouched() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("index.md");
    let original = format!("# Index\n\n{BEGIN_PREFIX}research-index -->\n- stale\n");
    fs::write(&path, &original).unwrap();

    let graph = graph_of(&[("p/Question/Q-1", EntityType::Question, json!({}))]);
    let error = index::build(&graph, &path).unwrap_err();

    assert_eq!(error.label, "kaya.projection.managed_grammar");
    assert!(
        error.message.contains("research-index"),
        "{}",
        error.message
    );
    assert_eq!(read(&path), original);
}

#[test]
fn sources_name_every_rendered_node_and_order_is_deterministic() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("index.md");
    let graph = graph_of(&[
        (
            "p/Question/Q-2",
            EntityType::Question,
            json!({"title": "Second"}),
        ),
        ("p/Decision/D-1", EntityType::Decision, json!({})),
        (
            "p/Question/Q-1",
            EntityType::Question,
            json!({"title": "First"}),
        ),
        (
            "p/Idea/I-1",
            EntityType::Idea,
            json!({"title": "Not indexed"}),
        ),
    ]);

    let summary = index::build(&graph, &path).unwrap();
    let document = read(&path);
    let rendered: Vec<&str> = document
        .lines()
        .filter_map(|line| line.strip_prefix("- "))
        .map(|rest| rest.split(" — ").next().unwrap())
        .collect();

    assert_eq!(
        rendered,
        vec!["p/Decision/D-1", "p/Question/Q-1", "p/Question/Q-2"]
    );
    let sources = &summary.sources["research-index"];
    for id in &rendered {
        assert!(
            sources.iter().any(|source| source.as_str() == *id),
            "{id} missing from sources"
        );
    }
    assert!(!document.contains("p/Idea/I-1"));
}

#[test]
fn artifact_and_run_sections_render_attributes_and_feed_sources() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("index.md");
    let graph = graph_of(&[
        (
            "p/Artifact/A-2",
            EntityType::Artifact,
            json!({"path": "out/report.pdf"}),
        ),
        (
            "p/Run/R-1",
            EntityType::Run,
            json!({"passed": 4, "failed": 1, "ignored": 2}),
        ),
        (
            "p/Artifact/A-1",
            EntityType::Artifact,
            json!({"sha": "abc123"}),
        ),
        ("p/Artifact/A-3", EntityType::Artifact, json!({})),
    ]);

    let summary = index::build(&graph, &path).unwrap();
    let document = read(&path);

    assert!(document.contains("### Artifacts\n"));
    assert!(document.contains("- p/Artifact/A-1 — abc123"));
    assert!(document.contains("- p/Artifact/A-2 — out/report.pdf"));
    assert!(document.contains("- p/Artifact/A-3 — A-3"));
    assert!(document.contains("### Runs\n"));
    assert!(document.contains("- p/Run/R-1 — passed 4, failed 1, ignored 2"));

    let sources = &summary.sources["research-index"];
    for id in [
        "p/Artifact/A-1",
        "p/Artifact/A-2",
        "p/Artifact/A-3",
        "p/Run/R-1",
    ] {
        assert!(
            sources.iter().any(|source| source.as_str() == id),
            "{id} missing from sources"
        );
    }
}

#[test]
fn empty_artifact_and_run_sections_are_skipped() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("index.md");
    let graph = graph_of(&[(
        "p/Question/Q-1",
        EntityType::Question,
        json!({"title": "Why?"}),
    )]);

    index::build(&graph, &path).unwrap();
    let document = read(&path);
    assert!(!document.contains("### Artifacts"));
    assert!(!document.contains("### Runs"));
}

#[test]
fn scan_returns_managed_blocks_with_bodies_between_markers() {
    let document = "head\n\
        <!-- kaya:authored -->\n\
        keep\n\
        <!-- kaya:end authored -->\n\
        <!-- kaya:begin generated:g -->\n\
        old\n\
        <!-- kaya:end generated:g -->\n\
        tail\n";

    let blocks = scan(document).unwrap();
    assert_eq!(
        blocks,
        vec![
            Block {
                kind: BlockKind::Authored,
                body: "keep\n".to_owned(),
            },
            Block {
                kind: BlockKind::Generated("g".to_owned()),
                body: "old\n".to_owned(),
            },
        ]
    );
}

#[test]
fn replace_generated_preserves_outside_bytes_and_appends_with_a_blank_line() {
    let document = "head\n\
        <!-- kaya:authored -->\n\
        keep\n\
        <!-- kaya:end authored -->\n\
        <!-- kaya:begin generated:g -->\n\
        old\n\
        <!-- kaya:end generated:g -->\n\
        tail\n";

    let replaced = replace_generated(document, "g", "new\n").unwrap();
    let expected_prefix = "head\n\
        <!-- kaya:authored -->\n\
        keep\n\
        <!-- kaya:end authored -->\n\
        <!-- kaya:begin generated:g -->\n";
    assert!(replaced.starts_with(expected_prefix));
    assert!(replaced.ends_with("<!-- kaya:end generated:g -->\ntail\n"));
    assert!(
        replaced.contains("<!-- kaya:begin generated:g -->\nnew\n<!-- kaya:end generated:g -->")
    );

    assert_eq!(
        replace_generated("hello\n", "g", "body\n").unwrap(),
        "hello\n\n<!-- kaya:begin generated:g -->\nbody\n<!-- kaya:end generated:g -->\n"
    );
    assert_eq!(
        replace_generated("", "g", "body\n").unwrap(),
        "<!-- kaya:begin generated:g -->\nbody\n<!-- kaya:end generated:g -->\n"
    );
}

#[test]
fn mismatched_end_marker_is_refused_naming_the_block() {
    let document = "<!-- kaya:begin generated:alpha -->\n\
        x\n\
        <!-- kaya:end generated:beta -->\n";

    let error = scan(document).unwrap_err();
    assert_eq!(error.label, "kaya.projection.managed_grammar");
    assert!(error.message.contains("alpha"), "{}", error.message);
    assert!(error.message.contains("beta"), "{}", error.message);
}
