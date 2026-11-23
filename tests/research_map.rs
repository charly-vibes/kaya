//! Contract tests for the `orientation.research-map` projection: typed edges
//! naming both endpoints, isolated entities, the closed block vocabulary, and
//! deterministic, idempotent rendering.

use std::fs;
use std::path::Path;

use kaya::model::{EntityType, Graph, GraphBuilder, Relationship};
use kaya::projection::BEGIN_PREFIX;
use kaya::projection::index::{RESEARCH_MAP_BLOCK, build};
use kaya::projection::report::{Kind, LABEL_LEGACY_BLOCK};
use serde_json::{Value, json};
use tempfile::tempdir;

/// One row per declared edge in [`edge_graph`], in canonical sort order.
const EDGE_ROWS: [&str; 4] = [
    "- p/Finding/f1 -[challenges]-> p/Hypothesis/h1",
    "- p/Finding/f1 -[derives]-> p/Observation/o1",
    "- p/Run/r1 -[executes]-> p/Experiment/e1",
    "- p/Run/r1 -[supports]-> p/Hypothesis/h1",
];

fn graph_of(nodes: &[(&str, EntityType, Value)]) -> Graph {
    let mut builder = GraphBuilder::new();
    for (id, entity, attributes) in nodes {
        builder
            .entity(*id, *entity, attributes.clone())
            .expect("test node must be admitted");
    }
    builder.build().expect("test graph must build")
}

/// A graph carrying one edge of every relationship kind the map must render.
fn edge_graph() -> Graph {
    let mut builder = GraphBuilder::new();
    for (id, entity, title) in [
        ("p/Run/r1", EntityType::Run, "Try it"),
        ("p/Experiment/e1", EntityType::Experiment, "Run it"),
        ("p/Hypothesis/h1", EntityType::Hypothesis, "It holds"),
        ("p/Finding/f1", EntityType::Finding, "It follows"),
        ("p/Observation/o1", EntityType::Observation, "Saw it"),
    ] {
        builder
            .entity(id, entity, json!({"title": title}))
            .expect("test node must be admitted");
    }
    for (relationship, from, to) in [
        (Relationship::Supports, "p/Run/r1", "p/Hypothesis/h1"),
        (Relationship::Challenges, "p/Finding/f1", "p/Hypothesis/h1"),
        (Relationship::Derives, "p/Finding/f1", "p/Observation/o1"),
        (Relationship::Executes, "p/Run/r1", "p/Experiment/e1"),
    ] {
        builder
            .relationship(relationship, from, to)
            .expect("relationship direction follows the grammar");
    }
    builder.build().expect("test graph must build")
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).expect("projection file must be readable")
}

/// The bytes of the generated research-map block, between its markers.
fn body_of(path: &Path) -> String {
    let document = read(path);
    let begin = format!("{BEGIN_PREFIX}{RESEARCH_MAP_BLOCK} -->\n");
    let end = format!("<!-- kaya:end generated:{RESEARCH_MAP_BLOCK} -->");
    let start = document.find(&begin).unwrap_or_else(|| {
        panic!("the research map must write a {RESEARCH_MAP_BLOCK} block; document:\n{document}")
    }) + begin.len();
    let stop = document[start..].find(&end).unwrap_or_else(|| {
        panic!("the {RESEARCH_MAP_BLOCK} block must be terminated; document:\n{document}")
    }) + start;
    document[start..stop].to_owned()
}

/// The rows `- …` of one `### <heading>` section of the rendered block.
fn section(body: &str, heading: &str) -> Vec<String> {
    let mut lines = body.lines().skip_while(|line| *line != heading);
    assert!(
        lines.next().is_some(),
        "the research map must render a {heading} section; body:\n{body}"
    );
    lines
        .take_while(|line| !line.is_empty() && !line.starts_with('#'))
        .filter(|line| line.starts_with("- "))
        .map(str::to_owned)
        .collect()
}

#[test]
fn every_declared_edge_renders_once_with_both_endpoints_and_type() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("index.md");

    build(&edge_graph(), &path, 4).expect("the research map must build");

    let body = body_of(&path);
    let rows = section(&body, "### Relationships");
    assert_eq!(
        rows.len(),
        EDGE_ROWS.len(),
        "every declared edge renders exactly one row; body:\n{body}"
    );

    for expected in EDGE_ROWS {
        let occurrences = rows.iter().filter(|row| row.as_str() == expected).count();
        assert_eq!(
            occurrences, 1,
            "edge \"{expected}\" must render exactly once; body:\n{body}"
        );
    }

    for row in &rows {
        assert!(
            EDGE_ROWS.contains(&row.as_str()),
            "row \"{row}\" names no declared edge; body:\n{body}"
        );
    }

    assert!(
        !body.contains("### Isolated"),
        "every node here is an edge endpoint, so no Isolated section may render; body:\n{body}"
    );
}

#[test]
fn relationship_rows_are_sorted_and_deduplicated() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("index.md");
    let mut builder = GraphBuilder::new();
    for (id, entity, title) in [
        ("p/Finding/f1", EntityType::Finding, "It follows"),
        ("p/Hypothesis/h1", EntityType::Hypothesis, "It holds"),
        ("p/Observation/o1", EntityType::Observation, "Saw it"),
        ("p/Run/r1", EntityType::Run, "Try it"),
        ("p/Experiment/e1", EntityType::Experiment, "Run it"),
    ] {
        builder
            .entity(id, entity, json!({"title": title}))
            .expect("test node must be admitted");
    }
    // Declared out of canonical order, with the same triple repeated
    // non-adjacently: the renderer must sort and collapse it.
    for (relationship, from, to) in [
        (Relationship::Supports, "p/Finding/f1", "p/Hypothesis/h1"),
        (Relationship::Executes, "p/Run/r1", "p/Experiment/e1"),
        (Relationship::Challenges, "p/Finding/f1", "p/Hypothesis/h1"),
        (Relationship::Derives, "p/Finding/f1", "p/Observation/o1"),
        (Relationship::Supports, "p/Finding/f1", "p/Hypothesis/h1"),
    ] {
        builder
            .relationship(relationship, from, to)
            .expect("relationship direction follows the grammar");
    }
    let graph = builder.build().expect("test graph must build");
    assert_eq!(
        graph.edges().len(),
        5,
        "the graph keeps both assertions of the same triple"
    );

    build(&graph, &path, 5).expect("the research map must build");

    let body = body_of(&path);
    assert_eq!(
        section(&body, "### Relationships"),
        vec![
            "- p/Finding/f1 -[challenges]-> p/Hypothesis/h1".to_owned(),
            "- p/Finding/f1 -[derives]-> p/Observation/o1".to_owned(),
            "- p/Finding/f1 -[supports]-> p/Hypothesis/h1".to_owned(),
            "- p/Run/r1 -[executes]-> p/Experiment/e1".to_owned(),
        ],
        "rows are sorted by (from, relationship, to) and the repeated triple renders once; body:\n{body}"
    );
    assert!(
        body.contains("_5 nodes from 5 events._"),
        "the repeated triple does not inflate the distinct node count; body:\n{body}"
    );
}

#[test]
fn self_edge_renders_once_and_node_is_not_isolated() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("index.md");
    let mut builder = GraphBuilder::new();
    builder
        .entity(
            "p/Hypothesis/h1",
            EntityType::Hypothesis,
            json!({"title": "It holds"}),
        )
        .expect("test node must be admitted");
    builder
        .relationship(Relationship::Supports, "p/Hypothesis/h1", "p/Hypothesis/h1")
        .expect("supports is free-direction, so a self edge is legal");
    let graph = builder.build().expect("test graph must build");

    build(&graph, &path, 1).expect("the research map must build");

    let body = body_of(&path);
    let row = "- p/Hypothesis/h1 -[supports]-> p/Hypothesis/h1";
    assert_eq!(
        section(&body, "### Relationships"),
        vec![row.to_owned()],
        "the self edge renders exactly one row; body:\n{body}"
    );
    assert_eq!(
        body.matches(row).count(),
        1,
        "the self edge appears exactly once in the whole block; body:\n{body}"
    );
    assert!(
        !body.contains("### Isolated"),
        "a node incident to its own edge is not isolated; body:\n{body}"
    );
    assert!(
        body.contains("_1 nodes from 1 events._"),
        "the self edge's node is counted once; body:\n{body}"
    );
}

#[test]
fn empty_graph_renders_only_the_count_line() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("index.md");
    let graph = GraphBuilder::new().build().expect("test graph must build");

    build(&graph, &path, 0).expect("the research map must build");

    let body = body_of(&path);
    assert_eq!(
        body, "\n_0 nodes from 0 events._\n",
        "an empty graph renders the count line and nothing else"
    );
    assert!(
        !body.contains('#'),
        "an empty graph renders no section heading; body:\n{body}"
    );
}

#[test]
fn isolated_entity_is_listed_once_and_no_edge_is_invented() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("index.md");
    let mut builder = GraphBuilder::new();
    for (id, entity, title) in [
        ("p/Run/r1", EntityType::Run, "Try it"),
        ("p/Experiment/e1", EntityType::Experiment, "Run it"),
        ("p/Decision/d1", EntityType::Decision, "Ship it"),
    ] {
        builder
            .entity(id, entity, json!({"title": title}))
            .expect("test node must be admitted");
    }
    builder
        .relationship(Relationship::Executes, "p/Run/r1", "p/Experiment/e1")
        .expect("run executes its experiment");
    let graph = builder.build().expect("test graph must build");

    build(&graph, &path, 2).expect("the research map must build");

    let body = body_of(&path);
    let isolated = section(&body, "### Isolated");
    assert_eq!(
        isolated,
        vec!["- p/Decision/d1 — Decision — Ship it".to_owned()],
        "a node without edges is listed once in Isolated; body:\n{body}"
    );
    assert!(
        !isolated[0].contains("-["),
        "an isolated row invents no edge: {}",
        isolated[0]
    );
    assert_eq!(
        body.matches("p/Decision/d1").count(),
        1,
        "an isolated node appears exactly once in the map; body:\n{body}"
    );

    let edges = section(&body, "### Relationships");
    assert_eq!(
        edges,
        vec!["- p/Run/r1 -[executes]-> p/Experiment/e1".to_owned()],
        "only the declared edge renders; body:\n{body}"
    );
    for row in &edges {
        assert!(
            !row.contains("p/Decision/d1"),
            "no edge may be invented for an isolated node: {row}"
        );
    }
}

#[test]
fn legacy_block_in_the_index_is_refused_before_writing() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("index.md");
    let original = "# Research\n\nIntro prose.\n\n\
        <!-- kaya:authored -->\n\
        Human interpretation.\n\
        <!-- kaya:end authored -->\n\n\
        <!-- kaya:begin generated:experiment-report -->\n\
        - legacy row\n\
        <!-- kaya:end generated:experiment-report -->\n";
    fs::write(&path, original).unwrap();

    let error = build(
        &graph_of(&[(
            "p/Question/q1",
            EntityType::Question,
            json!({"title": "Why?"}),
        )]),
        &path,
        1,
    )
    .expect_err("a generated block outside the vocabulary must be refused");

    assert_eq!(
        error.label, LABEL_LEGACY_BLOCK,
        "a legacy block is refused with its own label; error: {error}"
    );
    assert!(
        error.message.contains("experiment-report"),
        "the refusal names the offending block; error: {error}"
    );
    assert_eq!(
        fs::read(&path).unwrap(),
        original.as_bytes(),
        "a refused build must not write to the document"
    );
}

#[test]
fn legacy_block_alongside_a_valid_map_is_refused() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("index.md");
    let original = "# Research\n\nIntro prose.\n\n\
        <!-- kaya:begin generated:orientation.research-map -->\n\
        ### Relationships\n\
        - stale row\n\
        <!-- kaya:end generated:orientation.research-map -->\n\n\
        <!-- kaya:begin generated:experiment-report -->\n\
        - legacy row\n\
        <!-- kaya:end generated:experiment-report -->\n";
    fs::write(&path, original).unwrap();

    let error = build(
        &graph_of(&[(
            "p/Question/q1",
            EntityType::Question,
            json!({"title": "Why?"}),
        )]),
        &path,
        1,
    )
    .expect_err("one legacy block must refuse the whole document");

    assert_eq!(
        error.label, LABEL_LEGACY_BLOCK,
        "the legacy block is refused with its own label even when the map block is valid; error: {error}"
    );
    assert!(
        error.message.contains("experiment-report"),
        "the refusal names the offending block rather than the map; error: {error}"
    );
    assert_eq!(
        fs::read(&path).unwrap(),
        original.as_bytes(),
        "refusing the legacy block must leave the valid map block untouched"
    );
}

#[test]
fn map_block_name_is_the_vocabulary_name() {
    assert_eq!(RESEARCH_MAP_BLOCK, Kind::ResearchMap.block_name());
    assert_eq!(Kind::ResearchMap.block_name(), "orientation.research-map");
}

#[test]
fn map_is_deterministic_and_idempotent() {
    let dir = tempdir().unwrap();
    let first_path = dir.path().join("first.md");
    let second_path = dir.path().join("second.md");
    let graph = edge_graph();

    let first = build(&graph, &first_path, 4)
        .expect("the research map must build")
        .summary;
    let second = build(&graph, &second_path, 4)
        .expect("the research map must build")
        .summary;

    assert_eq!(
        read(&first_path),
        read(&second_path),
        "the same graph must render byte-identical maps"
    );
    assert_eq!(first.recomputed, vec![RESEARCH_MAP_BLOCK.to_owned()]);
    assert_eq!(second.recomputed, vec![RESEARCH_MAP_BLOCK.to_owned()]);

    let bytes = read(&first_path);
    let modified = fs::metadata(&first_path).unwrap().modified().unwrap();

    let third = build(&graph, &first_path, 4)
        .expect("the research map must rebuild")
        .summary;

    assert!(
        third.recomputed.is_empty(),
        "an unchanged graph recomputes nothing; summary: {third:?}"
    );
    assert_eq!(third.unchanged, vec![RESEARCH_MAP_BLOCK.to_owned()]);
    assert_eq!(read(&first_path), bytes, "a rebuild must not rewrite");
    assert_eq!(
        fs::metadata(&first_path).unwrap().modified().unwrap(),
        modified,
        "a rebuild must not touch the document"
    );
}
