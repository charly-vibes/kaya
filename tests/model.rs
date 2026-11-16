//! Contract tests for `kaya.model`: the closed vocabulary, direction
//! grammar, reference resolution, influence acyclicity, and namespace
//! qualified ids.

use kaya::event::{Actor, ActorType, Event, Provenance, Subject};
use kaya::model::{EntityType, Graph, GraphBuilder, ModelError, Relationship, build};
use serde_json::{Value, json};

fn event(kind: &str, subject_type: &str, subject_id: &str, data: Value, sequence: usize) -> Event {
    Event {
        id: format!("evt-{sequence:03}"),
        schema: "kaya.event/v0".to_owned(),
        r#type: kind.to_owned(),
        timestamp: "2026-11-16T00:00:00Z".to_owned(),
        actor: Actor {
            r#type: ActorType::Tool,
            name: "model-contract-tests".to_owned(),
        },
        subject: Subject {
            r#type: subject_type.to_owned(),
            id: subject_id.to_owned(),
        },
        data,
        provenance: Provenance {
            tool: "kaya-tests".to_owned(),
            version: "0.1.0".to_owned(),
            reference: "tests/model.rs".to_owned(),
        },
        artifacts: Vec::new(),
    }
}

fn entity_event(entity_type: &str, id: &str, attributes: Value, sequence: usize) -> Event {
    event(
        "model.entity.created",
        entity_type,
        id,
        json!({ "attributes": attributes }),
        sequence,
    )
}

fn relationship_event(relationship: &str, from: &str, to: &str, sequence: usize) -> Event {
    event(
        "model.relationship.asserted",
        "Relationship",
        from,
        json!({ "relationship": relationship, "from": from, "to": to }),
        sequence,
    )
}

fn expect_graph(result: Result<Graph, ModelError>) -> Graph {
    result.expect("graph build must succeed")
}

fn expect_refusal(result: Result<Graph, ModelError>) -> ModelError {
    result.expect_err("graph build must refuse")
}

#[test]
fn unknown_entity_type_refused_naming_the_type_and_closed_set() {
    let events = [entity_event(
        "Task",
        "alpha/Task/T-001",
        json!({ "title": "not research" }),
        1,
    )];

    let error = expect_refusal(build(&events));

    assert_eq!(error.label, "kaya.model.vocabulary_closed");
    assert!(error.message.contains("Task"), "{}", error.message);
    for entity in [
        "Project",
        "Question",
        "Idea",
        "Hypothesis",
        "Model",
        "Experiment",
        "Run",
        "Artifact",
        "Observation",
        "Claim",
        "Finding",
        "Decision",
        "Source",
    ] {
        assert!(
            error.message.contains(entity),
            "closed set must name {entity}: {}",
            error.message
        );
    }
}

#[test]
fn unknown_relationship_refused_naming_the_relationship_and_closed_set() {
    let events = [
        entity_event("Decision", "alpha/Decision/DEC-001", json!({}), 1),
        entity_event("Finding", "alpha/Finding/FND-001", json!({}), 2),
        relationship_event(
            "depends_on",
            "alpha/Decision/DEC-001",
            "alpha/Finding/FND-001",
            3,
        ),
    ];

    let error = expect_refusal(build(&events));

    assert_eq!(error.label, "kaya.model.relationship_set_closed");
    assert!(error.message.contains("depends_on"), "{}", error.message);
    for relationship in [
        "investigates",
        "predicts",
        "executes",
        "observes",
        "supports",
        "challenges",
        "influences",
        "realizes",
        "derives",
    ] {
        assert!(
            error.message.contains(relationship),
            "closed set must name {relationship}: {}",
            error.message
        );
    }
}

#[test]
fn dangling_claim_to_finding_refused_naming_the_endpoint() {
    let events = [
        entity_event("Claim", "alpha/Claim/CLM-001", json!({}), 1),
        relationship_event(
            "supports",
            "alpha/Claim/CLM-001",
            "alpha/Finding/FND-404",
            2,
        ),
    ];

    let error = expect_refusal(build(&events));

    assert_eq!(error.label, "kaya.model.references_resolve");
    assert!(
        error.message.contains("alpha/Finding/FND-404"),
        "{}",
        error.message
    );
}

#[test]
fn influence_cycle_refused_naming_its_members() {
    let events = [
        entity_event("Decision", "alpha/Decision/DEC-A", json!({}), 1),
        entity_event("Decision", "alpha/Decision/DEC-B", json!({}), 2),
        relationship_event(
            "influences",
            "alpha/Decision/DEC-A",
            "alpha/Decision/DEC-B",
            3,
        ),
        relationship_event(
            "influences",
            "alpha/Decision/DEC-B",
            "alpha/Decision/DEC-A",
            4,
        ),
    ];

    let error = expect_refusal(build(&events));

    assert_eq!(error.label, "kaya.model.influence_acyclic");
    assert!(
        error.message.contains("alpha/Decision/DEC-A"),
        "{}",
        error.message
    );
    assert!(
        error.message.contains("alpha/Decision/DEC-B"),
        "{}",
        error.message
    );
}

#[test]
fn reversed_executes_refused_naming_the_direction_rule() {
    let events = [
        entity_event("Experiment", "alpha/Experiment/EXP-001", json!({}), 1),
        entity_event("Run", "alpha/Run/RUN-001", json!({}), 2),
        relationship_event(
            "executes",
            "alpha/Experiment/EXP-001",
            "alpha/Run/RUN-001",
            3,
        ),
    ];

    let error = expect_refusal(build(&events));

    assert_eq!(error.label, "kaya.model.direction_fixed");
    assert!(error.message.contains("executes"), "{}", error.message);
    assert!(
        error.message.contains("Run -> Experiment"),
        "{}",
        error.message
    );
    assert!(
        error.message.contains("alpha/Experiment/EXP-001"),
        "{}",
        error.message
    );
    assert!(
        error.message.contains("alpha/Run/RUN-001"),
        "{}",
        error.message
    );
}

#[test]
fn graph_round_trips_identically() {
    let mut builder = GraphBuilder::new();
    builder
        .entity(
            "alpha/Experiment/EXP-001",
            EntityType::Experiment,
            json!({ "title": "does the cache help?" }),
        )
        .expect("entity")
        .entity(
            "alpha/Run/RUN-001",
            EntityType::Run,
            json!({ "attempt": 1 }),
        )
        .expect("entity")
        .entity(
            "alpha/Observation/OBS-001",
            EntityType::Observation,
            json!({ "p95_ms": 42 }),
        )
        .expect("entity")
        .entity(
            "alpha/Claim/CLM-001",
            EntityType::Claim,
            json!({ "statement": "the cache helps" }),
        )
        .expect("entity")
        .entity("alpha/Finding/FND-001", EntityType::Finding, json!({}))
        .expect("entity")
        .relationship(
            Relationship::Executes,
            "alpha/Run/RUN-001",
            "alpha/Experiment/EXP-001",
        )
        .expect("edge")
        .relationship(
            Relationship::Observes,
            "alpha/Observation/OBS-001",
            "alpha/Run/RUN-001",
        )
        .expect("edge")
        .relationship(
            Relationship::Supports,
            "alpha/Claim/CLM-001",
            "alpha/Finding/FND-001",
        )
        .expect("edge");

    let graph = expect_graph(builder.build());
    assert_eq!(graph.nodes().count(), 5);
    assert_eq!(graph.edges().count(), 3);

    let encoded = serde_json::to_string_pretty(&graph).expect("serialize graph");
    let decoded: Graph = serde_json::from_str(&encoded).expect("deserialize graph");

    assert_eq!(graph, decoded);
    assert_eq!(
        serde_json::to_string_pretty(&decoded).expect("reserialize graph"),
        encoded
    );
}

#[test]
fn namespaced_ids_keep_projects_distinct() {
    let events = [
        entity_event(
            "Experiment",
            "alpha/Experiment/EXP-001",
            json!({ "project": "alpha" }),
            1,
        ),
        entity_event(
            "Experiment",
            "beta/Experiment/EXP-001",
            json!({ "project": "beta" }),
            2,
        ),
    ];

    let graph = expect_graph(build(&events));

    assert_eq!(graph.nodes().count(), 2);
    let alpha = graph
        .node("alpha/Experiment/EXP-001")
        .expect("alpha node present");
    let beta = graph
        .node("beta/Experiment/EXP-001")
        .expect("beta node present");
    assert_ne!(alpha.id, beta.id);
    assert_eq!(alpha.attributes["project"], "alpha");
    assert_eq!(beta.attributes["project"], "beta");
}

#[test]
fn forward_grammar_builds_and_missing_attributes_default_to_an_object() {
    let events = [
        event(
            "model.entity.created",
            "Run",
            "alpha/Run/RUN-001",
            json!({}),
            1,
        ),
        entity_event(
            "Experiment",
            "alpha/Experiment/EXP-001",
            json!({ "title": "the experiment" }),
            2,
        ),
        relationship_event(
            "executes",
            "alpha/Run/RUN-001",
            "alpha/Experiment/EXP-001",
            3,
        ),
    ];

    let graph = expect_graph(build(&events));

    assert_eq!(graph.nodes().count(), 2);
    assert_eq!(graph.edges().count(), 1);
    let run = graph.node("alpha/Run/RUN-001").expect("run node present");
    assert_eq!(run.attributes, json!({}));
}

#[test]
fn ids_must_be_namespaced_and_match_the_entity_type() {
    let mut builder = GraphBuilder::new();

    let unqualified = builder
        .entity("EXP-001", EntityType::Experiment, json!({}))
        .expect_err("unqualified id must refuse");
    assert_eq!(unqualified.label, "kaya.model.namespaced_ids");
    assert!(
        unqualified.message.contains("EXP-001"),
        "{}",
        unqualified.message
    );

    let mismatched = builder
        .entity("alpha/Run/EXP-001", EntityType::Experiment, json!({}))
        .expect_err("type mismatch must refuse");
    assert_eq!(mismatched.label, "kaya.model.namespaced_ids");
    assert!(
        mismatched.message.contains("alpha/Run/EXP-001"),
        "{}",
        mismatched.message
    );
}
