//! The research graph over kaya's closed vocabulary.
//!
//! `kaya.model` turns admitted `model.entity.created` and
//! `model.relationship.asserted` events into a typed graph. The entity and
//! relationship sets are closed, node ids are namespace-qualified as
//! `<project>/<TYPE>/<key>`, every edge endpoint must resolve, relationship
//! direction follows a fixed grammar, and the influence/derives subgraph
//! stays acyclic. Every violation is a labeled [`ModelError`].

use std::collections::BTreeMap;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::event::Event;

/// Event type that creates a research node.
pub const ENTITY_CREATED_EVENT: &str = "model.entity.created";
/// Event type that asserts a typed relationship between two nodes.
pub const RELATIONSHIP_ASSERTED_EVENT: &str = "model.relationship.asserted";

const ENTITY_TYPE_SET: &str = "{Project, Question, Idea, Hypothesis, Model, Experiment, Run, Artifact, Observation, Claim, Finding, Decision, Source}";
const RELATIONSHIP_SET: &str = "{investigates, predicts, executes, observes, supports, challenges, influences, realizes, derives}";

/// A labeled refusal produced while building the research graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelError {
    /// Failure class, for example `kaya.model.vocabulary_closed`.
    pub label: String,
    /// Human-readable explanation naming the offending value.
    pub message: String,
}

impl ModelError {
    /// Construct a labeled refusal.
    pub fn new(label: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.label, self.message)
    }
}

impl std::error::Error for ModelError {}

/// The thirteen node types of the research vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityType {
    Project,
    Question,
    Idea,
    Hypothesis,
    Model,
    Experiment,
    Run,
    Artifact,
    Observation,
    Claim,
    Finding,
    Decision,
    Source,
}

impl EntityType {
    /// Every admitted node type, in vocabulary order.
    pub const ALL: [Self; 13] = [
        Self::Project,
        Self::Question,
        Self::Idea,
        Self::Hypothesis,
        Self::Model,
        Self::Experiment,
        Self::Run,
        Self::Artifact,
        Self::Observation,
        Self::Claim,
        Self::Finding,
        Self::Decision,
        Self::Source,
    ];

    /// The variant name, used as the `TYPE` segment of a node id and on the
    /// wire.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Project => "Project",
            Self::Question => "Question",
            Self::Idea => "Idea",
            Self::Hypothesis => "Hypothesis",
            Self::Model => "Model",
            Self::Experiment => "Experiment",
            Self::Run => "Run",
            Self::Artifact => "Artifact",
            Self::Observation => "Observation",
            Self::Claim => "Claim",
            Self::Finding => "Finding",
            Self::Decision => "Decision",
            Self::Source => "Source",
        }
    }
}

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for EntityType {
    type Err = ModelError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .iter()
            .copied()
            .find(|entity| entity.as_str() == raw)
            .ok_or_else(|| {
                ModelError::new(
                    "kaya.model.vocabulary_closed",
                    format!(
                        "unknown entity type \"{raw}\"; the closed thirteen-entity set is {ENTITY_TYPE_SET}"
                    ),
                )
            })
    }
}

/// The nine admitted relationship kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Relationship {
    Investigates,
    Predicts,
    Executes,
    Observes,
    Supports,
    Challenges,
    Influences,
    Realizes,
    Derives,
}

impl Relationship {
    /// Every admitted relationship, in vocabulary order.
    pub const ALL: [Self; 9] = [
        Self::Investigates,
        Self::Predicts,
        Self::Executes,
        Self::Observes,
        Self::Supports,
        Self::Challenges,
        Self::Influences,
        Self::Realizes,
        Self::Derives,
    ];

    /// The lowercase relationship name used on the wire and in event data.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Investigates => "investigates",
            Self::Predicts => "predicts",
            Self::Executes => "executes",
            Self::Observes => "observes",
            Self::Supports => "supports",
            Self::Challenges => "challenges",
            Self::Influences => "influences",
            Self::Realizes => "realizes",
            Self::Derives => "derives",
        }
    }

    /// The grammar's required `(from, to)` entity pair, when the
    /// relationship's direction is fixed.
    fn fixed_direction(self) -> Option<(EntityType, EntityType)> {
        match self {
            Self::Executes => Some((EntityType::Run, EntityType::Experiment)),
            Self::Observes => Some((EntityType::Observation, EntityType::Run)),
            Self::Derives => Some((EntityType::Finding, EntityType::Observation)),
            Self::Influences => Some((EntityType::Decision, EntityType::Finding)),
            Self::Realizes => Some((EntityType::Model, EntityType::Hypothesis)),
            Self::Investigates | Self::Predicts | Self::Supports | Self::Challenges => None,
        }
    }

    /// Whether the relationship participates in the acyclic subgraph.
    fn is_acyclic_edge(self) -> bool {
        matches!(self, Self::Influences | Self::Derives)
    }
}

impl std::fmt::Display for Relationship {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Relationship {
    type Err = ModelError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .iter()
            .copied()
            .find(|relationship| relationship.as_str() == raw)
            .ok_or_else(|| {
                ModelError::new(
                    "kaya.model.relationship_set_closed",
                    format!(
                        "unknown relationship \"{raw}\"; the closed nine-relationship set is {RELATIONSHIP_SET}"
                    ),
                )
            })
    }
}

/// One research node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    /// Namespace-qualified id `<project>/<TYPE>/<key>`.
    pub id: String,
    /// Entity type, matching the id's `TYPE` segment.
    pub entity: EntityType,
    /// Entity payload; `{}` when the ledger event carried none.
    pub attributes: Value,
}

impl Node {
    /// A node with empty attributes.
    pub fn new(id: impl Into<String>, entity: EntityType) -> Self {
        Self {
            id: id.into(),
            entity,
            attributes: Value::Object(Map::new()),
        }
    }
}

/// One directed, typed relationship between two nodes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Edge {
    pub relationship: Relationship,
    pub from: String,
    pub to: String,
}

/// The built research graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Graph {
    pub nodes: BTreeMap<String, Node>,
    pub edges: Vec<Edge>,
}

impl Graph {
    /// All nodes, ordered by id.
    pub fn nodes(&self) -> impl ExactSizeIterator<Item = &Node> {
        self.nodes.values()
    }

    /// All asserted relationships, in assertion order.
    pub fn edges(&self) -> impl ExactSizeIterator<Item = &Edge> {
        self.edges.iter()
    }

    /// The node with `id`, when present.
    pub fn node(&self, id: &str) -> Option<&Node> {
        self.nodes.get(id)
    }
}

/// Incremental construction of a [`Graph`].
#[derive(Debug, Clone, Default)]
pub struct GraphBuilder {
    nodes: BTreeMap<String, Node>,
    edges: Vec<Edge>,
}

impl GraphBuilder {
    /// An empty graph.
    pub fn new() -> Self {
        Self::default()
    }

    /// Admit a node. The id must be namespace-qualified and its `TYPE`
    /// segment must name `entity`.
    pub fn entity(
        &mut self,
        id: impl Into<String>,
        entity: EntityType,
        attributes: Value,
    ) -> Result<&mut Self, ModelError> {
        let id = id.into();
        let id_type = parse_namespaced_id(&id)?;
        if id_type != entity {
            return Err(ModelError::new(
                "kaya.model.namespaced_ids",
                format!(
                    "node id \"{id}\" names type {id_type} but the node is declared as {entity}"
                ),
            ));
        }
        self.nodes.insert(
            id.clone(),
            Node {
                id,
                entity,
                attributes,
            },
        );
        Ok(self)
    }

    /// Assert a relationship. Direction is checked immediately when both
    /// endpoints are already present and again at [`Self::build`], when
    /// forward references have resolved.
    pub fn relationship(
        &mut self,
        relationship: Relationship,
        from: impl Into<String>,
        to: impl Into<String>,
    ) -> Result<&mut Self, ModelError> {
        let from = from.into();
        let to = to.into();
        let from_entity = self.nodes.get(&from).map(|node| node.entity);
        let to_entity = self.nodes.get(&to).map(|node| node.entity);
        if let (Some(from_entity), Some(to_entity)) = (from_entity, to_entity) {
            check_direction(relationship, &from, from_entity, &to, to_entity)?;
        }
        self.edges.push(Edge {
            relationship,
            from,
            to,
        });
        Ok(self)
    }

    /// Resolve endpoints, enforce direction, and refuse influence cycles.
    pub fn build(self) -> Result<Graph, ModelError> {
        for edge in &self.edges {
            let from = self
                .nodes
                .get(&edge.from)
                .ok_or_else(|| dangling_reference(edge, &edge.from))?;
            let to = self
                .nodes
                .get(&edge.to)
                .ok_or_else(|| dangling_reference(edge, &edge.to))?;
            check_direction(
                edge.relationship,
                &edge.from,
                from.entity,
                &edge.to,
                to.entity,
            )?;
        }
        check_acyclic(&self.edges)?;
        Ok(Graph {
            nodes: self.nodes,
            edges: self.edges,
        })
    }
}

/// Build a graph from ledger events.
///
/// `model.entity.created` events create nodes; `model.relationship.asserted`
/// events add edges. Other event types are ignored — the model interprets
/// only the two events that describe its own structure.
pub fn build(events: &[Event]) -> Result<Graph, ModelError> {
    let mut builder = GraphBuilder::new();
    for event in events {
        match event.r#type.as_str() {
            ENTITY_CREATED_EVENT => entity_from_event(event, &mut builder)?,
            RELATIONSHIP_ASSERTED_EVENT => relationship_from_event(event, &mut builder)?,
            _ => {}
        }
    }
    builder.build()
}

fn entity_from_event(event: &Event, builder: &mut GraphBuilder) -> Result<(), ModelError> {
    let entity = EntityType::from_str(&event.subject.r#type)?;
    let attributes = match event.data.get("attributes") {
        None => Value::Object(Map::new()),
        Some(value) if value.is_object() => value.clone(),
        Some(value) => {
            return Err(ModelError::new(
                "kaya.model.attributes_object",
                format!(
                    "entity event \"{}\" carries data.attributes as {} rather than a JSON object",
                    event.id,
                    value_type(value)
                ),
            ));
        }
    };
    builder.entity(event.subject.id.clone(), entity, attributes)?;
    Ok(())
}

fn relationship_from_event(event: &Event, builder: &mut GraphBuilder) -> Result<(), ModelError> {
    let data = event.data.as_object().ok_or_else(|| {
        ModelError::new(
            "kaya.model.relationship_shape",
            format!(
                "relationship event \"{}\" carries {} data rather than a JSON object",
                event.id,
                value_type(&event.data)
            ),
        )
    })?;
    let raw = data
        .get("relationship")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            ModelError::new(
                "kaya.model.relationship_shape",
                format!(
                    "relationship event \"{}\" is missing string data.relationship",
                    event.id
                ),
            )
        })?;
    let relationship = Relationship::from_str(raw)?;
    let from = data.get("from").and_then(Value::as_str).ok_or_else(|| {
        ModelError::new(
            "kaya.model.relationship_shape",
            format!(
                "relationship event \"{}\" is missing string data.from",
                event.id
            ),
        )
    })?;
    let to = data.get("to").and_then(Value::as_str).ok_or_else(|| {
        ModelError::new(
            "kaya.model.relationship_shape",
            format!(
                "relationship event \"{}\" is missing string data.to",
                event.id
            ),
        )
    })?;
    builder.relationship(relationship, from, to)?;
    Ok(())
}

fn parse_namespaced_id(id: &str) -> Result<EntityType, ModelError> {
    let segments: Vec<&str> = id.split('/').collect();
    if segments.len() != 3 || segments.iter().any(|segment| segment.is_empty()) {
        return Err(ModelError::new(
            "kaya.model.namespaced_ids",
            format!("node id \"{id}\" is not namespace-qualified as <project>/<TYPE>/<key>"),
        ));
    }
    EntityType::from_str(segments[1])
}

fn check_direction(
    relationship: Relationship,
    from_id: &str,
    from_entity: EntityType,
    to_id: &str,
    to_entity: EntityType,
) -> Result<(), ModelError> {
    if let Some((required_from, required_to)) = relationship.fixed_direction()
        && (from_entity, to_entity) == (required_to, required_from)
    {
        return Err(ModelError::new(
            "kaya.model.direction_fixed",
            format!(
                "relationship {} is directed {} -> {}; edge \"{from_id}\" -> \"{to_id}\" runs it backwards",
                relationship.as_str(),
                required_from.as_str(),
                required_to.as_str()
            ),
        ));
    }
    Ok(())
}

fn dangling_reference(edge: &Edge, endpoint: &str) -> ModelError {
    ModelError::new(
        "kaya.model.references_resolve",
        format!(
            "relationship {} edge \"{}\" -> \"{}\" references absent node \"{endpoint}\"",
            edge.relationship.as_str(),
            edge.from,
            edge.to
        ),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mark {
    Temporary,
    Permanent,
}

fn check_acyclic(edges: &[Edge]) -> Result<(), ModelError> {
    let mut adjacency: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for edge in edges
        .iter()
        .filter(|edge| edge.relationship.is_acyclic_edge())
    {
        adjacency
            .entry(edge.from.as_str())
            .or_default()
            .push(edge.to.as_str());
    }

    let mut marks: BTreeMap<&str, Mark> = BTreeMap::new();
    let mut path: Vec<&str> = Vec::new();

    for start in adjacency.keys().copied() {
        if marks.get(start) == Some(&Mark::Permanent) {
            continue;
        }
        marks.insert(start, Mark::Temporary);
        path.push(start);
        let mut frames: Vec<(&str, usize)> = vec![(start, 0)];

        while let Some(&(node, index)) = frames.last() {
            let targets = adjacency
                .get(node)
                .map(|targets| targets.as_slice())
                .unwrap_or(&[]);
            if index < targets.len() {
                let target = targets[index];
                if let Some(frame) = frames.last_mut() {
                    frame.1 += 1;
                }
                match marks.get(target).copied() {
                    Some(Mark::Temporary) => {
                        let position = path.iter().position(|seen| *seen == target).unwrap_or(0);
                        let mut members: Vec<&str> = path[position..].to_vec();
                        members.push(target);
                        let rendered = members
                            .iter()
                            .map(|id| format!("\"{id}\""))
                            .collect::<Vec<_>>()
                            .join(" -> ");
                        return Err(ModelError::new(
                            "kaya.model.influence_acyclic",
                            format!("influence/derives edges form a cycle: {rendered}"),
                        ));
                    }
                    Some(Mark::Permanent) => {}
                    None => {
                        marks.insert(target, Mark::Temporary);
                        path.push(target);
                        frames.push((target, 0));
                    }
                }
            } else {
                marks.insert(node, Mark::Permanent);
                frames.pop();
                path.pop();
            }
        }
    }
    Ok(())
}

fn value_type(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "a boolean",
        Value::Number(_) => "a number",
        Value::String(_) => "a string",
        Value::Array(_) => "an array",
        Value::Object(_) => "an object",
    }
}
