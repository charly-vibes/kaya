//! The explain projection: make the publication machinery legible at the CLI.
//!
//! Purpose: answer, without reading kaya's source, why a product or
//! collection is omitted, which document kinds would back it, where those
//! kinds' content comes from, and what flips when they are populated. The
//! command is read-only: it reads the ledger, models the graph, and prints
//! envelopes; it never writes the ledger or the tree.
//!
//! Responsibilities: resolve a name against the three closed publication
//! vocabularies and the document-kind block vocabulary; report per-name
//! admission rows, graph feeders, ingestion paths, and dependents; mirror the
//! renderers' emptiness conditions so a reported status matches what `kaya
//! build` and `kaya publish` would actually emit. Rationale: the admission
//! table and feeder rules live in code, not documentation, so an author who
//! cannot see them cannot author the backing documents — this command is the
//! machine-readable form of that contract.

use serde_json::{Value, json};
use std::fs;
use std::io::ErrorKind;

use kaya::envelope::Envelope;
use kaya::event::parse_line;
use kaya::model::{self, EntityType, Graph, Relationship};
use kaya::publication::{
    COLLECTIONS, IMPLEMENTED, PRODUCTS, REASON_NO_BACKING_DOCUMENT, Tier, admission, backing_kinds,
    reader_question, resolve_name,
};

use crate::LEDGER_PATH;

/// Same failure shape as every command handler: `(label, message)`.
type Failure = (String, String);

/// Every closed document-kind block name, derived from the contract table so
/// the vocabulary has exactly one in-tree spelling.
fn all_blocks() -> Vec<&'static str> {
    KIND_CONTRACTS
        .iter()
        .map(|contract| contract.block)
        .collect()
}

/// One document kind's feeder contract, mirroring the renderer conditions.
///
/// `entity` names the node entity type(s) whose presence populates the
/// block; `attribute` names the one attribute that matters when the feeder
/// is attribute-driven (`None` for entity-presence feeders); `ingestion` is
/// the command path that produces such nodes; `seed_example` carries a
/// copy-pasteable `import model` record when no adapter or capture command
/// writes the feeder attribute.
struct KindContract {
    /// Generated block name, `family.kind`.
    block: &'static str,
    /// Bare kind name accepted as an alias for the block name.
    bare: &'static str,
    /// The feeder entity types, human-readable.
    entity: &'static str,
    /// The feeder attribute, when one drives the block.
    attribute: Option<&'static str>,
    /// The ingestion path that produces feeder nodes.
    ingestion: &'static str,
    /// A valid `import model` seed line, when no adapter covers the kind.
    seed_example: Option<&'static str>,
    /// A structural caveat the status alone would hide.
    note: Option<&'static str>,
}

/// The feeder contract for every closed document kind, in vocabulary order.
///
/// Each row mirrors one renderer's emptiness condition: `project-overview`
/// mirrors `report::render_overview`, `research-map` the index projection's
/// whole-graph render, `exploration-brief` and `experiment-specification`
/// the brief/specification renders, `experiment-record` the anchored record
/// render, `findings-and-analysis` the findings render, `decision-record`
/// the decision render, and the four `reuse-handoff` rows mirror
/// `reuse::inputs` plus the flat renderers over its selections.
const KIND_CONTRACTS: &[KindContract] = &[
    KindContract {
        block: "orientation.project-overview",
        bare: "project-overview",
        entity: "Project, Question, or Finding",
        attribute: None,
        ingestion: "kaya capture question | kaya capture finding",
        seed_example: None,
        note: None,
    },
    KindContract {
        block: "orientation.research-map",
        bare: "research-map",
        entity: "any modelled node",
        attribute: None,
        ingestion: "any kaya import or kaya capture",
        seed_example: None,
        note: None,
    },
    KindContract {
        block: "exploration.exploration-brief",
        bare: "exploration-brief",
        entity: "Question, Idea, or Hypothesis",
        attribute: None,
        ingestion: "kaya capture question; `import model` for Idea and Hypothesis",
        seed_example: None,
        note: Some(
            "the brief renders to docs/research/brief.md, which the publication reader set does \
             not read, so it cannot back tutorial until that contract changes",
        ),
    },
    KindContract {
        block: "exploration.experiment-specification",
        bare: "experiment-specification",
        entity: "Question or Hypothesis joined to an Experiment (investigates / predicts)",
        attribute: None,
        ingestion: "`import model` records - an Experiment entity plus model.relationship.asserted edges; no capture command creates the join",
        seed_example: None,
        note: None,
    },
    KindContract {
        block: "exploration.experiment-record",
        bare: "experiment-record",
        entity: "Run joined to an Experiment by executes",
        attribute: None,
        ingestion: "`import model` records - an Experiment entity plus a model.relationship.asserted executes edge; capture note/finding enter only through run-anchored chains",
        seed_example: None,
        note: None,
    },
    KindContract {
        block: "understanding.findings-and-analysis",
        bare: "findings-and-analysis",
        entity: "Finding, Observation, or Claim",
        attribute: None,
        ingestion: "kaya capture finding | kaya capture note; `import model` for Claim",
        seed_example: None,
        note: None,
    },
    KindContract {
        block: "understanding.decision-record",
        bare: "decision-record",
        entity: "Decision",
        attribute: None,
        ingestion: "kaya capture decision",
        seed_example: None,
        note: None,
    },
    KindContract {
        block: "reuse-handoff.technical-reference",
        bare: "technical-reference",
        entity: "Source",
        attribute: Some("spec"),
        ingestion: "no adapter or capture command writes `spec`; seed with `kaya import model`",
        note: None,
        seed_example: Some(
            r#"{"id":"seed-spec-1","schema":"kaya.event/v0","type":"model.entity.created","timestamp":"2026-11-24T00:00:00Z","actor":{"type":"agent","name":"docs"},"subject":{"type":"Source","id":"kaya/Source/spec-kaya-promotion"},"data":{"attributes":{"spec":"kaya-promotion"}},"provenance":{"tool":"kaya","version":"0.4.0","reference":"import model"},"artifacts":[]}"#,
        ),
    },
    KindContract {
        block: "reuse-handoff.reproduction-package",
        bare: "reproduction-package",
        entity: "Source (path and commit) or Run (command)",
        attribute: Some("path, commit"),
        ingestion: "`import model` seeds Sources (path+commit) and Run commands; `kaya import test` writes Run counts only, no command attribute",
        seed_example: None,
        note: None,
    },
    KindContract {
        block: "reuse-handoff.knowledge-brief",
        bare: "knowledge-brief",
        entity: "Source",
        attribute: Some("entry"),
        ingestion: "kaya import turu",
        seed_example: None,
        note: None,
    },
    KindContract {
        block: "reuse-handoff.handoff",
        bare: "handoff",
        entity: "Artifact (kind handoffs)",
        attribute: Some("kind"),
        ingestion: "kaya import wai (artifacts under a handoffs/ directory)",
        seed_example: None,
        note: None,
    },
];

/// Run `kaya explain [name]` against the ledger graph without writing.
pub fn run(name: Option<&str>) -> Result<Envelope, Failure> {
    let graph = read_graph()?;
    let Some(name) = name else {
        return Ok(bare_explain(&graph));
    };

    // Products and collections resolve through the pinned tier vocabularies;
    // everything else is tried as a document kind, bare or fully qualified.
    if let Ok((Tier::Product, product)) = resolve_name(name) {
        return Ok(product_explain(&graph, product));
    }
    if let Ok((Tier::Collection, collection)) = resolve_name(name) {
        return Ok(collection_explain(&graph, collection));
    }
    if let Some(contract) = KIND_CONTRACTS
        .iter()
        .find(|contract| contract.block == name || contract.bare == name)
    {
        return Ok(kind_explain(&graph, contract));
    }
    Err((
        "kaya.explain.unknown_name".to_owned(),
        format!(
            "unknown name \"{name}\"; explain resolves product names ({}), collection names ({}), \
             and document kinds by bare or block name ({}); package names are navigation-only \
             and are not explained",
            PRODUCTS.join(", "),
            COLLECTIONS.join(", "),
            KIND_CONTRACTS
                .iter()
                .map(|contract| contract.block)
                .collect::<Vec<_>>()
                .join(", "),
        ),
    ))
}

/// The omission census: every product and collection that would not emit,
/// each with its reason and the kinds that would back it.
fn bare_explain(graph: &Graph) -> Envelope {
    let mut omitted = Vec::new();
    let mut emitted = Vec::new();
    for product in PRODUCTS {
        push_state(
            &mut omitted,
            &mut emitted,
            product,
            "product",
            admission(product),
            product_emitted(graph, product),
        );
    }
    // Only the implemented collections can appear in a built tree; the rest
    // refuse at declaration, so they are never omissions to explain.
    for collection in IMPLEMENTED {
        // The knowledge-index is backed by any document at all, so its
        // omission row admits the whole closed kind vocabulary.
        let blocks = all_blocks();
        let kinds: &[&str] = if collection == "knowledge-index" {
            &blocks
        } else {
            backing_kinds(collection).unwrap_or(&[])
        };
        push_state(
            &mut omitted,
            &mut emitted,
            collection,
            "collection",
            kinds,
            collection_emitted(graph, collection),
        );
    }
    Envelope::ok(
        "explain",
        json!({
            "omitted": omitted,
            "emitted": emitted,
        }),
    )
}

/// One product's view: its reader question, admission row, and status.
fn product_explain(graph: &Graph, product: &'static str) -> Envelope {
    let kinds = admission(product);
    let emitted = product_emitted(graph, product);
    let mut data = json!({
        "name": product,
        "tier": "product",
        "question": reader_question(product),
        "admits": kinds,
        "status": if emitted { "emitted" } else { "omitted" },
    });
    if !emitted {
        data["reason"] = json!(REASON_NO_BACKING_DOCUMENT);
    }
    // Carry any structural caveat an admitted kind attaches, so the product
    // view explains the same trap the kind view does.
    let notes: Vec<&str> = KIND_CONTRACTS
        .iter()
        .filter(|contract| kinds.contains(&contract.block))
        .filter_map(|contract| contract.note)
        .collect();
    if let [note] = notes.as_slice() {
        data["note"] = json!(note);
    } else if notes.len() > 1 {
        data["notes"] = json!(notes);
    }
    Envelope::ok("explain", data)
}

/// One collection's view: its backing kinds and status.
fn collection_explain(graph: &Graph, collection: &'static str) -> Envelope {
    if !IMPLEMENTED.contains(&collection) {
        return Envelope::ok(
            "explain",
            json!({
                "name": collection,
                "tier": "collection",
                "status": "unimplemented",
                "reason": "the collection is in the closed vocabulary but this release implements only glossary, topic-map, knowledge-index, example-gallery, and api-catalogue",
            }),
        );
    }
    let kinds = backing_kinds(collection).unwrap_or(&[]);
    let emitted = collection_emitted(graph, collection);
    let mut data = json!({
        "name": collection,
        "tier": "collection",
        "backs": kinds,
        "status": if emitted { "emitted" } else { "omitted" },
    });
    if !emitted {
        data["reason"] = json!(REASON_NO_BACKING_DOCUMENT);
    }
    Envelope::ok("explain", data)
}

/// One kind's view: feeder, ingestion path, dependents, and status.
fn kind_explain(graph: &Graph, contract: &KindContract) -> Envelope {
    let backs: Vec<&str> = PRODUCTS
        .iter()
        .copied()
        .filter(|product| admission(product).contains(&contract.block))
        .collect();
    let collections: Vec<&str> = COLLECTIONS
        .iter()
        .copied()
        .filter(|collection| {
            backing_kinds(collection)
                .unwrap_or(&[])
                .contains(&contract.block)
        })
        .collect();
    let populated = populated(graph, contract.block);
    let mut data = json!({
        "name": contract.bare,
        "tier": "document-kind",
        "kind": contract.block,
        "feeder": {
            "entity": contract.entity,
            "attribute": contract.attribute,
        },
        "ingestion": contract.ingestion,
        "backs": backs,
        "collections": collections,
        "status": if populated { "populated" } else { "unpopulated" },
    });
    if let Some(seed) = contract.seed_example {
        data["seed_example"] = json!(seed);
    }
    if let Some(note) = contract.note {
        data["note"] = json!(note);
    }
    Envelope::ok("explain", data)
}

/// Append one name to the omitted or emitted side of the census.
fn push_state(
    omitted: &mut Vec<Value>,
    emitted: &mut Vec<Value>,
    name: &str,
    tier: &str,
    kinds: &[&str],
    is_emitted: bool,
) {
    if is_emitted {
        emitted.push(json!(name));
        return;
    }
    omitted.push(json!({
        "name": name,
        "tier": tier,
        "reason": REASON_NO_BACKING_DOCUMENT,
        "admits": kinds,
    }));
}

/// Whether a product emits: at least one admissible kind is populated.
///
/// `exploration.exploration-brief` never counts: its block renders to
/// `docs/research/brief.md`, outside the publication reader set, so it can
/// never appear as a backing document (see the contract note).
fn product_emitted(graph: &Graph, product: &str) -> bool {
    admission(product)
        .iter()
        .filter(|kind| **kind != "exploration.exploration-brief")
        .any(|kind| populated(graph, kind))
}

/// Whether a collection emits, mirroring `collection.rs`'s render guards:
/// per-kind collections need their kind populated; the glossary emits
/// unconditionally and the knowledge-index needs any document at all.
fn collection_emitted(graph: &Graph, collection: &str) -> bool {
    match collection {
        // The topic-map composes the map's relationship rows: with no nodes
        // the map body carries no rows and the page omits.
        "topic-map" => graph.nodes().next().is_some(),
        _ => match backing_kinds(collection) {
            Some(kinds) => kinds.iter().any(|kind| populated(graph, kind)),
            None => collection == "glossary" || graph.nodes().next().is_some(),
        },
    }
}

/// Whether a kind's block would render non-empty, mirroring each renderer's
/// emptiness condition (see [`KIND_CONTRACTS`]).
fn populated(graph: &Graph, block: &str) -> bool {
    let any = |wanted: &[EntityType]| graph.nodes().any(|node| wanted.contains(&node.entity));
    match block {
        "orientation.project-overview" => any(&[
            EntityType::Project,
            EntityType::Question,
            EntityType::Finding,
        ]),
        // The research-map block always renders: even an empty graph emits
        // the summary line, so the publication source map never omits it.
        "orientation.research-map" => true,
        "exploration.exploration-brief" => any(&[
            EntityType::Question,
            EntityType::Idea,
            EntityType::Hypothesis,
        ]),
        "exploration.experiment-specification" => {
            // Dossier anchors are the graph's Experiment nodes; the
            // specification renders only a Question joined by investigates
            // or a Hypothesis joined by predicts into one of them.
            let experiments: Vec<&str> = graph
                .nodes()
                .filter(|node| node.entity == EntityType::Experiment)
                .map(|node| node.id.as_str())
                .collect();
            graph.edges().any(|edge| {
                let touches_experiment = experiments.contains(&edge.from.as_str())
                    || experiments.contains(&edge.to.as_str());
                let kind_joined = |kind: EntityType, relationship: Relationship| {
                    edge.relationship == relationship
                        && [&edge.from, &edge.to]
                            .iter()
                            .any(|end| graph.node(end).is_some_and(|node| node.entity == kind))
                };
                touches_experiment
                    && (kind_joined(EntityType::Question, Relationship::Investigates)
                        || kind_joined(EntityType::Hypothesis, Relationship::Predicts))
            })
        }
        "exploration.experiment-record" => {
            // Executes is fixed Run -> Experiment, so one executes edge is
            // exactly the record renderer's emptiness condition:
            // observations and findings enter only through run-anchored
            // chains, and no other relationship anchors a run to an
            // experiment.
            graph
                .edges()
                .any(|edge| edge.relationship == Relationship::Executes)
        }
        "understanding.findings-and-analysis" => any(&[
            EntityType::Finding,
            EntityType::Observation,
            EntityType::Claim,
        ]),
        "understanding.decision-record" => any(&[EntityType::Decision]),
        "reuse-handoff.technical-reference" => source_with(graph, "spec"),
        "reuse-handoff.reproduction-package" => {
            let revisions = graph.nodes().any(|node| {
                node.entity == EntityType::Source
                    && non_blank(node, "path")
                    && non_blank(node, "commit")
            });
            let commands = graph.nodes().any(|node| {
                node.entity == EntityType::Run
                    && (non_blank(node, "command") || commands_array(node))
            });
            revisions || commands
        }
        "reuse-handoff.knowledge-brief" => source_with(graph, "entry"),
        "reuse-handoff.handoff" => graph.nodes().any(|node| {
            node.entity == EntityType::Artifact
                && node
                    .attributes
                    .get("kind")
                    .and_then(Value::as_str)
                    .is_some_and(|kind| kind == "handoffs")
                && non_blank(node, "path")
        }),
        _ => false,
    }
}

/// Whether any Source node carries a non-blank value for `attribute`.
fn source_with(graph: &Graph, attribute: &str) -> bool {
    graph
        .nodes()
        .any(|node| node.entity == EntityType::Source && non_blank(node, attribute))
}

/// Whether a node carries a non-blank string attribute.
fn non_blank(node: &kaya::model::Node, attribute: &str) -> bool {
    node.attributes
        .get(attribute)
        .and_then(Value::as_str)
        .is_some_and(|value| !value.trim().is_empty())
}

/// Whether a node's `commands` attribute is a non-empty string array.
fn commands_array(node: &kaya::model::Node) -> bool {
    node.attributes
        .get("commands")
        .and_then(Value::as_array)
        .is_some_and(|entries| {
            entries.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|command| !command.trim().is_empty())
            })
        })
}

/// Read the ledger and model its graph; explain never writes either.
fn read_graph() -> Result<Graph, Failure> {
    let bytes = fs::read(LEDGER_PATH).map_err(|err| match err.kind() {
        ErrorKind::NotFound => (
            "missing-ledger".to_owned(),
            format!("missing ledger {LEDGER_PATH}: run `kaya init` to create it"),
        ),
        _ => (
            "ledger-io".to_owned(),
            format!("failed to read {LEDGER_PATH}: {err}"),
        ),
    })?;
    let mut events = Vec::new();
    for (index, raw) in bytes.split_inclusive(|byte| *byte == b'\n').enumerate() {
        let text = std::str::from_utf8(raw.strip_suffix(b"\n").unwrap_or(raw)).map_err(|err| {
            (
                "kaya.explain.ledger".to_owned(),
                format!("ledger line {} is not UTF-8: {err}", index + 1),
            )
        })?;
        if text.is_empty() {
            continue;
        }
        let event = parse_line(text).map_err(|err| {
            (
                err.label.to_owned(),
                format!("ledger line {}: {}", index + 1, err.message),
            )
        })?;
        events.push(event);
    }
    model::build(&events).map_err(|err| {
        (
            err.label.to_owned(),
            format!("failed to model the ledger graph: {}", err.message),
        )
    })
}
