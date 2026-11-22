//! Contract tests for `kaya.model`: the closed vocabulary, direction
//! grammar, reference resolution, influence acyclicity, and namespace
//! qualified ids.

use kaya::event::{Actor, ActorType, Event, Provenance, Subject};
use kaya::ledger::Ledger;
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
fn capture_events_create_artifact_and_run_nodes_with_attributes() {
    let events = [
        event(
            "capture.artifact.observed",
            "Artifact",
            "alpha/Artifact/ART-001",
            json!({ "path": "target/report.json", "kind": "json" }),
            1,
        ),
        event(
            "capture.commit.observed",
            "Artifact",
            "alpha/Artifact/ART-002",
            json!({ "sha": "abc123", "message": "fix cache" }),
            2,
        ),
        event(
            "capture.artifact.observed",
            "Artifact",
            "alpha/Artifact/ART-003",
            json!({ "path": "target/only-path.log" }),
            3,
        ),
        event(
            "experiment.run.completed",
            "Run",
            "alpha/Run/RUN-001",
            json!({ "passed": 10, "failed": 1, "ignored": 0, "total": 11 }),
            4,
        ),
    ];

    let graph = expect_graph(build(&events));

    assert_eq!(graph.nodes().count(), 4);
    assert_eq!(graph.edges().count(), 0);

    let artifact = graph
        .node("alpha/Artifact/ART-001")
        .expect("artifact node present");
    assert_eq!(artifact.entity, EntityType::Artifact);
    assert_eq!(
        artifact.attributes,
        json!({ "path": "target/report.json", "kind": "json" })
    );

    let commit = graph
        .node("alpha/Artifact/ART-002")
        .expect("commit artifact node present");
    assert_eq!(commit.entity, EntityType::Artifact);
    assert_eq!(
        commit.attributes,
        json!({ "sha": "abc123", "message": "fix cache" })
    );

    let partial = graph
        .node("alpha/Artifact/ART-003")
        .expect("partial artifact node present");
    assert_eq!(
        partial.attributes,
        json!({ "path": "target/only-path.log" })
    );

    let run = graph.node("alpha/Run/RUN-001").expect("run node present");
    assert_eq!(run.entity, EntityType::Run);
    assert_eq!(
        run.attributes,
        json!({ "passed": 10, "failed": 1, "ignored": 0, "total": 11 })
    );
}

#[test]
fn capture_events_still_validate_namespaced_subject_ids() {
    let unqualified = [event(
        "capture.artifact.observed",
        "Artifact",
        "ART-001",
        json!({ "path": "target/report.json" }),
        1,
    )];

    let error = expect_refusal(build(&unqualified));
    assert_eq!(error.label, "kaya.model.namespaced_ids");
    assert!(error.message.contains("ART-001"), "{}", error.message);

    let mismatched_type = [event(
        "experiment.run.completed",
        "Artifact",
        "alpha/Artifact/ART-001",
        json!({ "passed": 1 }),
        1,
    )];

    let error = expect_refusal(build(&mismatched_type));
    assert_eq!(error.label, "kaya.model.namespaced_ids");
    assert!(
        error.message.contains("alpha/Artifact/ART-001"),
        "{}",
        error.message
    );
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

#[test]
fn wrong_entity_pairs_refused_for_each_exact_pair_relationship() {
    let cases = [
        (
            "executes",
            "Run",
            "alpha/Run/RUN-R1",
            "Run",
            "alpha/Run/RUN-R2",
            "Run -> Experiment",
        ),
        (
            "executes",
            "Hypothesis",
            "alpha/Hypothesis/HYP-001",
            "Experiment",
            "alpha/Experiment/EXP-001",
            "Run -> Experiment",
        ),
        (
            "observes",
            "Claim",
            "alpha/Claim/CLM-001",
            "Run",
            "alpha/Run/RUN-001",
            "Observation -> Run",
        ),
        (
            "derives",
            "Finding",
            "alpha/Finding/FND-1",
            "Finding",
            "alpha/Finding/FND-2",
            "Finding -> Observation",
        ),
        (
            "realizes",
            "Run",
            "alpha/Run/RUN-001",
            "Hypothesis",
            "alpha/Hypothesis/HYP-001",
            "Model -> Hypothesis",
        ),
    ];

    for (relationship, from_type, from_id, to_type, to_id, rule) in cases {
        let events = [
            entity_event(from_type, from_id, json!({}), 1),
            entity_event(to_type, to_id, json!({}), 2),
            relationship_event(relationship, from_id, to_id, 3),
        ];

        let error = expect_refusal(build(&events));

        assert_eq!(
            error.label, "kaya.model.direction_fixed",
            "{relationship} {from_type} -> {to_type} must be refused"
        );
        assert!(error.message.contains(relationship), "{}", error.message);
        assert!(error.message.contains(rule), "{}", error.message);
        assert!(error.message.contains(from_id), "{}", error.message);
        assert!(error.message.contains(to_id), "{}", error.message);
    }
}

#[test]
fn cites_relationship_joins_the_closed_set() {
    let decision = "alpha/Decision/DEC-001";
    let source = "alpha/Source/SRC-001";
    let events = [
        entity_event("Decision", decision, json!({}), 1),
        entity_event(
            "Source",
            source,
            json!({ "path": "src/cache.rs", "commit": "abc123" }),
            2,
        ),
        relationship_event("cites", decision, source, 3),
    ];

    let graph = expect_graph(build(&events));

    let edge = graph
        .edges
        .iter()
        .find(|edge| edge.from == decision && edge.to == source)
        .expect("the cites edge is admitted into the graph");
    assert_eq!(
        edge.relationship.as_str(),
        "cites",
        "the wire name is the lowercase verb: {}",
        edge.relationship.as_str()
    );
}

#[test]
fn unknown_relationship_refusal_names_cites_in_the_closed_set() {
    let events = [
        entity_event("Decision", "alpha/Decision/DEC-001", json!({}), 1),
        entity_event("Source", "alpha/Source/SRC-001", json!({}), 2),
        relationship_event(
            "depends_on",
            "alpha/Decision/DEC-001",
            "alpha/Source/SRC-001",
            3,
        ),
    ];

    let error = expect_refusal(build(&events));

    assert_eq!(error.label, "kaya.model.relationship_set_closed");
    assert!(
        error.message.contains("cites"),
        "the closed set must name cites: {}",
        error.message
    );
}

#[test]
fn cites_direction_is_decision_to_source_exact_pair() {
    let cases = [
        (
            "Source",
            "alpha/Source/SRC-001",
            "Decision",
            "alpha/Decision/DEC-001",
        ),
        ("Run", "alpha/Run/RUN-001", "Source", "alpha/Source/SRC-002"),
        (
            "Decision",
            "alpha/Decision/DEC-002",
            "Decision",
            "alpha/Decision/DEC-001",
        ),
        (
            "Finding",
            "alpha/Finding/FND-001",
            "Source",
            "alpha/Source/SRC-003",
        ),
    ];

    for (from_type, from_id, to_type, to_id) in cases {
        let events = [
            entity_event(from_type, from_id, json!({}), 1),
            entity_event(to_type, to_id, json!({}), 2),
            relationship_event("cites", from_id, to_id, 3),
        ];

        let error = expect_refusal(build(&events));

        assert_eq!(
            error.label, "kaya.model.direction_fixed",
            "cites {from_type} -> {to_type} must be refused by the grammar"
        );
        assert!(error.message.contains("cites"), "{}", error.message);
        assert!(
            error.message.contains("Decision -> Source"),
            "the refusal names the grammar's rule: {}",
            error.message
        );
        assert!(error.message.contains(from_id), "{}", error.message);
        assert!(error.message.contains(to_id), "{}", error.message);
    }
}

#[test]
fn cites_to_an_unknown_node_is_refused_as_dangling() {
    let events = [
        entity_event("Decision", "alpha/Decision/DEC-001", json!({}), 1),
        relationship_event(
            "cites",
            "alpha/Decision/DEC-001",
            "alpha/Source/SRC-404",
            2,
        ),
    ];

    let error = expect_refusal(build(&events));

    assert_eq!(error.label, "kaya.model.references_resolve");
    assert!(
        error.message.contains("alpha/Source/SRC-404"),
        "{}",
        error.message
    );
}

#[test]
fn self_loop_influence_refused_by_acyclicity_not_direction() {
    let decision = "alpha/Decision/DEC-001";
    let events = [
        entity_event("Decision", decision, json!({}), 1),
        relationship_event("influences", decision, decision, 2),
    ];

    let error = expect_refusal(build(&events));

    assert_eq!(error.label, "kaya.model.influence_acyclic");
    assert!(error.message.contains(decision), "{}", error.message);
}

#[test]
fn mixed_derives_and_influence_cycle_refused() {
    let decision = "alpha/Decision/DEC-001";
    let finding = "alpha/Finding/FND-001";
    let observation = "alpha/Observation/OBS-001";
    let events = [
        entity_event("Decision", decision, json!({}), 1),
        entity_event("Finding", finding, json!({}), 2),
        entity_event("Observation", observation, json!({}), 3),
        // Decision influences Finding, Finding derives Observation, and
        // Observation influences Decision closes a mixed relationship cycle.
        relationship_event("influences", decision, finding, 4),
        relationship_event("derives", finding, observation, 5),
        relationship_event("influences", observation, decision, 6),
    ];

    let error = expect_refusal(build(&events));

    assert_eq!(error.label, "kaya.model.influence_acyclic");
    for id in [decision, finding, observation] {
        assert!(error.message.contains(id), "{}", error.message);
    }
}

#[test]
fn forward_relationship_assertion_resolves_once_entities_arrive() {
    // Direction can only be checked once both endpoints are known, so a
    // relationship asserted before its entities is held and resolved at
    // build time rather than refused at assertion time.
    let run = "alpha/Run/RUN-001";
    let experiment = "alpha/Experiment/EXP-001";
    let events = [
        relationship_event("executes", run, experiment, 1),
        entity_event("Experiment", experiment, json!({}), 2),
        entity_event("Run", run, json!({}), 3),
    ];

    let graph = expect_graph(build(&events));

    assert_eq!(graph.edges().count(), 1);
    let edge = graph.edges().next().expect("edge present");
    assert_eq!(edge.relationship, Relationship::Executes);
    assert_eq!(edge.from, run);
    assert_eq!(edge.to, experiment);
}

#[test]
fn repeated_entity_definitions_merge_object_attributes() {
    let run = "alpha/Run/RUN-001";
    let events = [
        entity_event("Run", run, json!({ "attempt": 1 }), 1),
        event(
            "experiment.run.completed",
            "Run",
            run,
            json!({ "passed": 10, "failed": 0, "ignored": 0, "total": 10 }),
            2,
        ),
    ];

    let graph = expect_graph(build(&events));

    let node = graph.node(run).expect("run node present");
    assert_eq!(
        node.attributes,
        json!({ "attempt": 1, "passed": 10, "failed": 0, "ignored": 0, "total": 10 })
    );
}

#[test]
fn non_object_attributes_replace_rather_than_merge() {
    let run = "alpha/Run/RUN-001";
    let mut builder = GraphBuilder::new();

    builder
        .entity(run, EntityType::Run, json!({ "attempt": 1 }))
        .expect("entity")
        .entity(run, EntityType::Run, json!("replaced"))
        .expect("entity replacement")
        .entity(run, EntityType::Run, json!({ "attempt": 2 }))
        .expect("entity replacement");

    let graph = expect_graph(builder.build());
    assert_eq!(
        graph.node(run).expect("run node present").attributes,
        json!({ "attempt": 2 })
    );
}

#[test]
fn corrected_run_counts_reach_the_model_through_the_ledger() {
    let run = "kaya/Run/RUN-001";
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("ledger.jsonl");
    let mut ledger = Ledger::new(&path);

    let original = event(
        "experiment.run.completed",
        "Run",
        run,
        json!({ "passed": 3, "failed": 1, "ignored": 0, "total": 4 }),
        1,
    );
    let correction = event(
        "experiment.run.corrected",
        "Run",
        run,
        json!({
            "passed": 4,
            "failed": 0,
            "ignored": 1,
            "total": 5,
            "supersedes": original.id.clone(),
        }),
        2,
    );

    ledger.append(&original).expect("append original");
    ledger.append(&correction).expect("append correction");
    let (events, _recovery) = ledger.read_recovered().expect("read ledger");

    let graph = expect_graph(build(&events));

    let node = graph.node(run).expect("run node present");
    assert_eq!(
        node.attributes,
        json!({ "passed": 4, "failed": 0, "ignored": 1, "total": 5 })
    );
}

#[test]
fn corrected_relationship_event_updates_the_edge() {
    let run = "alpha/Run/RUN-001";
    let first = "alpha/Experiment/EXP-001";
    let second = "alpha/Experiment/EXP-002";
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("ledger.jsonl");
    let mut ledger = Ledger::new(&path);

    let original = relationship_event("executes", run, first, 1);
    let correction = event(
        "model.relationship.corrected",
        "Relationship",
        run,
        json!({
            "relationship": "executes",
            "from": run,
            "to": second,
            "supersedes": original.id.clone(),
        }),
        2,
    );

    ledger.append(&original).expect("append original");
    ledger.append(&correction).expect("append correction");
    let (events, _recovery) = ledger.read_recovered().expect("read ledger");

    let entities = [
        entity_event("Run", run, json!({}), 3),
        entity_event("Experiment", first, json!({}), 4),
        entity_event("Experiment", second, json!({}), 5),
    ];
    let mut ledger_events = entities.to_vec();
    ledger_events.extend(events);

    let graph = expect_graph(build(&ledger_events));

    assert_eq!(graph.edges().count(), 1);
    let edge = graph.edges().next().expect("edge present");
    assert_eq!(edge.relationship, Relationship::Executes);
    assert_eq!(edge.from, run);
    assert_eq!(edge.to, second);
}
