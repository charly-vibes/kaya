//! Contract tests for `kaya.projection` and the `orientation.research-map`
//! projection: managed-block grammar, byte preservation, deterministic
//! rendering, and incremental rebuild reporting.

use std::fs;
use std::path::Path;

use kaya::model::{EntityType, Graph, GraphBuilder, Relationship};
use kaya::projection::index;
use kaya::projection::{BEGIN_PREFIX, Block, BlockKind, replace_generated, scan};
use serde_json::{Value, json};
use tempfile::tempdir;

const BLOCK: &str = index::RESEARCH_MAP_BLOCK;

fn begin_block() -> String {
    format!("{BEGIN_PREFIX}{BLOCK} -->")
}

fn end_block() -> String {
    format!("<!-- kaya:end generated:{BLOCK} -->")
}

fn graph_of(nodes: &[(&str, EntityType, Value)]) -> Graph {
    graph_with_edges(nodes, &[])
}

fn graph_with_edges(
    nodes: &[(&str, EntityType, Value)],
    edges: &[(Relationship, &str, &str)],
) -> Graph {
    let mut builder = GraphBuilder::new();
    for (id, entity, attributes) in nodes {
        builder
            .entity(*id, *entity, attributes.clone())
            .expect("test node must be admitted");
    }
    for (relationship, from, to) in edges {
        builder
            .relationship(*relationship, *from, *to)
            .expect("test edge must be admitted");
    }
    builder.build().expect("test graph must build")
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).expect("projection file must be readable")
}

/// The `- …` rows of one `### <heading>` section, empty when it is absent.
fn section_rows(document: &str, heading: &str) -> Vec<String> {
    let mut lines = document.lines().skip_while(|line| *line != heading);
    if lines.next().is_none() {
        return Vec::new();
    }
    lines
        .take_while(|line| !line.is_empty() && !line.starts_with('#'))
        .filter(|line| line.starts_with("- "))
        .map(str::to_owned)
        .collect()
}

/// The rows of the `### Isolated` section, empty when it renders no section.
fn isolated_rows(document: &str) -> Vec<String> {
    section_rows(document, "### Isolated")
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
    let original = format!(
        "{before}{}\n- stale line\n{}{after}",
        begin_block(),
        end_block()
    );
    fs::write(&path, &original).unwrap();

    let graph = graph_of(&[(
        "project/Question/Q-1",
        EntityType::Question,
        json!({"title": "Why?"}),
    )]);
    let summary = index::build(&graph, &path, 3).unwrap();
    assert_eq!(summary.recomputed, vec![BLOCK.to_owned()]);

    let rebuilt = read(&path);
    let begin = rebuilt.find(BEGIN_PREFIX).unwrap();
    let end_marker = end_block();
    let end = rebuilt.find(&end_marker).unwrap();

    assert_eq!(&rebuilt[..begin], before, "prefix bytes must not change");
    assert_eq!(
        &rebuilt[end + end_marker.len()..],
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
    assert!(
        rebuilt.contains("### Isolated\n- project/Question/Q-1 — Question — Why?\n"),
        "an edgeless graph lists its node as isolated: {rebuilt}"
    );
}

#[test]
fn identical_graphs_build_byte_identical_files() {
    let dir = tempdir().unwrap();
    let first = dir.path().join("first.md");
    let second = dir.path().join("second.md");

    let graph = graph_with_edges(
        &[
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
        ],
        &[(Relationship::Supports, "p/Experiment/E-1", "p/Finding/F-1")],
    );

    index::build(&graph, &first, 3).unwrap();
    index::build(&graph, &second, 3).unwrap();

    assert_eq!(read(&first), read(&second));

    let document = read(&first);
    let edge = "- p/Experiment/E-1 -[supports]-> p/Finding/F-1";
    assert_eq!(
        document.matches(edge).count(),
        1,
        "every declared edge renders once with both endpoints and its type: {document}"
    );
    assert!(
        document.contains("### Isolated\n- p/Question/Q-1 — Question — Q-1\n"),
        "the edgeless node is listed once as isolated: {document}"
    );
    let isolated = isolated_rows(&document);
    assert_eq!(
        isolated,
        vec!["- p/Question/Q-1 — Question — Q-1".to_owned()],
        "only the edgeless node renders in the Isolated section: {document}"
    );
    for endpoint in ["p/Experiment/E-1", "p/Finding/F-1"] {
        assert!(
            !isolated.iter().any(|row| row.contains(endpoint)),
            "edge endpoint {endpoint} appears nowhere in the Isolated section: {document}"
        );
    }
    assert!(
        document.find("### Relationships").unwrap() < document.find("### Isolated").unwrap(),
        "relationships precede the isolated rows: {document}"
    );
    assert!(
        document.contains("_3 nodes from 3 events._"),
        "the summary counts the three distinct rendered node ids: {document}"
    );
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

    let first = index::build(&graph, &path, 3).unwrap();
    assert_eq!(first.recomputed, vec![BLOCK.to_owned()]);
    let bytes = read(&path);
    let modified = fs::metadata(&path).unwrap().modified().unwrap();

    let second = index::build(&graph, &path, 3).unwrap();
    assert!(second.recomputed.is_empty());
    assert_eq!(second.unchanged, vec![BLOCK.to_owned()]);
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
    index::build(&first_graph, &path, 3).unwrap();
    let first = read(&path);
    let begin = first.find(BEGIN_PREFIX).unwrap();
    let end_marker = end_block();
    let end = first.find(&end_marker).unwrap();
    let prefix = &first[..begin];
    let suffix = &first[end + end_marker.len()..];

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
    let summary = index::build(&second_graph, &path, 4).unwrap();
    assert_eq!(summary.recomputed, vec![BLOCK.to_owned()]);
    assert!(summary.unchanged.is_empty());

    let second = read(&path);
    assert_ne!(first, second);
    assert!(second.starts_with(prefix));
    assert!(second.ends_with(suffix));
    assert!(
        second.contains("### Isolated\n"),
        "a graph without declared edges lists every node as isolated: {second}"
    );
    assert!(
        !second.contains("### Relationships"),
        "no edge is invented for an edgeless graph: {second}"
    );
    assert!(second.contains("- p/Experiment/E-1 — Experiment — Second"));
    assert!(second.contains("- p/Question/Q-1 — Question — First"));
}

#[test]
fn unterminated_generated_block_refuses_naming_it_and_leaves_file_untouched() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("index.md");
    let original = format!("# Index\n\n{}\n- stale\n", begin_block());
    fs::write(&path, &original).unwrap();

    let graph = graph_of(&[("p/Question/Q-1", EntityType::Question, json!({}))]);
    let error = index::build(&graph, &path, 3).unwrap_err();

    assert_eq!(error.label, "kaya.projection.managed_grammar");
    assert!(error.message.contains(BLOCK), "{}", error.message);
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

    let summary = index::build(&graph, &path, 3).unwrap();
    let document = read(&path);
    let rendered: Vec<&str> = document
        .lines()
        .filter_map(|line| line.strip_prefix("- "))
        .map(|rest| rest.split(" — ").next().unwrap())
        .collect();

    assert_eq!(
        rendered,
        vec![
            "p/Decision/D-1",
            "p/Idea/I-1",
            "p/Question/Q-1",
            "p/Question/Q-2"
        ],
        "the isolated rows are ordered by id: {document}"
    );
    assert!(
        document.contains("### Isolated\n"),
        "an edgeless graph renders its nodes as isolated rows: {document}"
    );
    assert!(
        !document.contains("### Relationships"),
        "an edgeless graph renders no relationship section: {document}"
    );
    let sources = &summary.sources[BLOCK];
    for id in &rendered {
        assert!(
            sources.iter().any(|source| source.as_str() == *id),
            "{id} missing from sources"
        );
    }
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

    let summary = index::build(&graph, &path, 3).unwrap();
    let document = read(&path);

    assert!(
        !document.contains("### Isolated"),
        "edgeless Artifact and Run nodes render only in their detail sections: {document}"
    );
    assert_eq!(
        section_rows(&document, "### Artifacts"),
        vec![
            "- p/Artifact/A-1 — abc123".to_owned(),
            "- p/Artifact/A-2 — out/report.pdf".to_owned(),
            "- p/Artifact/A-3 — A-3".to_owned(),
        ],
        "each edgeless artifact renders once, ordered by id, in Artifacts: {document}"
    );
    assert_eq!(
        section_rows(&document, "### Runs"),
        vec!["- p/Run/R-1 — passed 4, failed 1, ignored 2".to_owned()],
        "each edgeless run renders once, in Runs: {document}"
    );
    assert_eq!(
        document.matches("p/Artifact/A-2").count(),
        1,
        "an edgeless artifact is not duplicated outside its detail row: {document}"
    );
    assert_eq!(
        document.matches("p/Run/R-1").count(),
        1,
        "an edgeless run is not duplicated outside its detail row: {document}"
    );
    assert!(!document.contains("### Relationships"));
    assert!(document.contains("### Artifacts\n"));
    assert!(document.contains("- p/Artifact/A-1 — abc123"));
    assert!(document.contains("- p/Artifact/A-2 — out/report.pdf"));
    assert!(document.contains("- p/Artifact/A-3 — A-3"));
    assert!(document.contains("### Runs\n"));
    assert!(document.contains("- p/Run/R-1 — passed 4, failed 1, ignored 2"));
    assert!(
        document.contains("_4 nodes from 3 events._"),
        "distinct node ids are counted once even when a node has two rows: {document}"
    );

    let sources = &summary.sources[BLOCK];
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
fn edge_endpoint_with_a_detail_section_renders_once_per_section_and_is_never_isolated() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("index.md");
    let graph = graph_with_edges(
        &[
            (
                "p/Artifact/A-1",
                EntityType::Artifact,
                json!({"path": "out/report.pdf"}),
            ),
            (
                "p/Run/R-1",
                EntityType::Run,
                json!({"passed": 2, "failed": 0, "ignored": 1}),
            ),
            (
                "p/Hypothesis/h1",
                EntityType::Hypothesis,
                json!({"title": "It holds"}),
            ),
            (
                "p/Observation/o1",
                EntityType::Observation,
                json!({"title": "Saw it"}),
            ),
            (
                "p/Decision/d1",
                EntityType::Decision,
                json!({"title": "Ship it"}),
            ),
        ],
        &[
            (Relationship::Supports, "p/Artifact/A-1", "p/Hypothesis/h1"),
            (Relationship::Observes, "p/Observation/o1", "p/Run/R-1"),
        ],
    );

    index::build(&graph, &path, 4).unwrap();
    let document = read(&path);

    assert_eq!(
        section_rows(&document, "### Relationships"),
        vec![
            "- p/Artifact/A-1 -[supports]-> p/Hypothesis/h1".to_owned(),
            "- p/Observation/o1 -[observes]-> p/Run/R-1".to_owned(),
        ],
        "each endpoint's edge renders exactly once in Relationships: {document}"
    );
    assert_eq!(
        section_rows(&document, "### Artifacts"),
        vec!["- p/Artifact/A-1 — out/report.pdf".to_owned()],
        "the edge endpoint artifact renders exactly once in its detail section: {document}"
    );
    assert_eq!(
        section_rows(&document, "### Runs"),
        vec!["- p/Run/R-1 — passed 2, failed 0, ignored 1".to_owned()],
        "the edge endpoint run renders exactly once in its detail section: {document}"
    );
    assert_eq!(
        isolated_rows(&document),
        vec!["- p/Decision/d1 — Decision — Ship it".to_owned()],
        "only the edgeless non-detail node is isolated: {document}"
    );

    for endpoint in ["p/Artifact/A-1", "p/Run/R-1"] {
        assert!(
            !isolated_rows(&document)
                .iter()
                .any(|row| row.contains(endpoint)),
            "edge endpoint {endpoint} appears nowhere in the Isolated section: {document}"
        );
        assert_eq!(
            document.matches(endpoint).count(),
            2,
            "{endpoint} appears once in Relationships and once in its detail section: {document}"
        );
    }
    assert!(
        document.contains("_5 nodes from 4 events._"),
        "the summary counts each distinct node once across sections: {document}"
    );
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

    index::build(&graph, &path, 3).unwrap();
    let document = read(&path);
    assert!(!document.contains("### Artifacts"));
    assert!(!document.contains("### Runs"));
    assert!(!document.contains("### Relationships"));
    assert!(
        document.contains("### Isolated\n- p/Question/Q-1 — Question — Why?\n"),
        "the map still lists the edgeless node: {document}"
    );
    assert!(
        document.contains("_1 nodes from 3 events._"),
        "only the isolated row is counted: {document}"
    );
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
