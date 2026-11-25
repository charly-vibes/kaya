//! The implemented collection pages.
//!
//! Purpose: render each implemented collection from the source map and say
//! when a collection has nothing to render. Responsibilities: keep every
//! collection body a pure projection of the source map so rebuilds are
//! byte-identical and `kaya check` compares equal bytes, parse the research
//! map's relationship and isolated sections into per-topic rows, and derive
//! the gallery and catalogue rows from the producer blocks' own sections.
//! Rationale: one renderer per implemented collection, dispatched by name,
//! keeps the partition refusal, the default declarations, and the export in
//! exact agreement about what is implemented; a collection whose rows come
//! out empty returns `None` and the caller reports the standard omission
//! instead of writing a page of empty sections.

use std::collections::{BTreeMap, BTreeSet};

use crate::model::EntityType;

use super::source::{DOCUMENT_KINDS, Documents};

/// The document kinds whose bodies back collection rows.
const MAP_KIND: &str = "orientation.research-map";
const RECORD_KIND: &str = "exploration.experiment-record";
const TECH_KIND: &str = "reuse-handoff.technical-reference";

/// The document kinds whose populated blocks back a collection.
///
/// `None` marks the collections that emit from the compiled document set
/// unconditionally (glossary renders the closed kind vocabulary) or from any
/// document at all (knowledge-index renders the census), so they never carry
/// a per-kind backing row. Explaining omissions names the same keys the
/// renderer consults, so the two can never disagree about what backs what.
pub fn backing_kinds(name: &str) -> Option<&'static [&'static str]> {
    match name {
        "api-catalogue" => Some(&[TECH_KIND]),
        "example-gallery" => Some(&[RECORD_KIND]),
        "topic-map" => Some(&[MAP_KIND]),
        _ => None,
    }
}

/// The collections this release implements, in vocabulary order. The
/// partition refusal quotes this list, so a new renderer registers here and
/// the refusal stays truthful without a second edit. The list and
/// [`collection_page`]'s dispatch must change together in both directions: a
/// name here without a dispatch arm silently renders as an omission, and a
/// dispatch arm without a name here is refused at the partition.
pub const IMPLEMENTED: [&str; 5] = [
    "api-catalogue",
    "example-gallery",
    "glossary",
    "knowledge-index",
    "topic-map",
];

/// Render one implemented collection's page body, or `None` when its rows
/// come out empty. `map_owner` is the emitted product that composes the
/// research map, if any: the topic-map then cross-references that page
/// instead of embedding the edge statements a second time.
pub(super) fn collection_page(
    name: &str,
    docs: &Documents,
    map_owner: Option<&str>,
) -> Option<String> {
    match name {
        "glossary" => Some(render_glossary()),
        "topic-map" => render_topic_map(docs, map_owner),
        "knowledge-index" => render_knowledge_index(docs),
        "example-gallery" => render_example_gallery(docs),
        "api-catalogue" => render_api_catalogue(docs),
        // Unreachable: partition_declarations refuses any other name before
        // the render, so a stray name can never silently render nothing.
        _ => None,
    }
}

/// The glossary composes its body from the compile-time closed vocabularies
/// only — the entity types, the relationship grammar, and the document kinds
/// — so the page needs no ledger read and is byte-identical from build to
/// build.
fn render_glossary() -> String {
    let mut page = String::from("## Glossary\n\n");
    page.push_str("The closed vocabulary this build's research graph and publications use.\n\n");
    page.push_str("### Entities\n\n");
    for entity in EntityType::ALL {
        page.push_str(&format!("- {}\n", entity.as_str()));
    }
    page.push_str("\n### Relationships\n\n");
    page.push_str(&format!(
        "A relationship links two entities from the closed set {set}:\n\n",
        set = crate::model::ENTITY_TYPE_SET,
    ));
    for relationship in crate::model::Relationship::ALL {
        page.push_str(&format!("- {}\n", relationship.as_str()));
    }
    page.push_str("\n### Document kinds\n\n");
    for kind in DOCUMENT_KINDS {
        page.push_str(&format!("- {kind}\n"));
    }
    page
}

/// One topic's row facts: its entity type and the relationship types that
/// touch it in the research map.
type Topic = (String, BTreeSet<String>);

/// The topic-map composes the research map: one row per distinct topic id
/// with its entity type and relationship types, and the edge statements
/// embedded exactly when no emitted product carries the body.
fn render_topic_map(docs: &Documents, map_owner: Option<&str>) -> Option<String> {
    let map = docs.0.get(MAP_KIND)?;
    let mut topics: BTreeMap<&str, Topic> = BTreeMap::new();
    let mut edges: Vec<(&str, &str, &str)> = Vec::new();

    let relationships = section(&map.body, "### Relationships");
    for line in relationships.lines() {
        let Some(rest) = line.trim().strip_prefix("- ") else {
            continue;
        };
        let Some((from, rest)) = rest.split_once("-[") else {
            continue;
        };
        let Some((relationship, to)) = rest.split_once("]->") else {
            continue;
        };
        // Labeled rows carry a human title after the id (`kaya/Decision/keep
        // (keep the gate)`); node ids contain neither whitespace nor parens,
        // so the label suffix is stripped before the id is indexed.
        let from = from.split(" (").next().unwrap_or(from).trim();
        let to = to.split(" (").next().unwrap_or(to).trim();
        let relationship = relationship.trim();
        edges.push((from, relationship, to));
        let (_, rels) = topic_row(&mut topics, from);
        rels.insert(relationship.to_owned());
        let (_, rels) = topic_row(&mut topics, to);
        rels.insert(relationship.to_owned());
    }

    for line in section(&map.body, "### Isolated").lines() {
        let Some(rest) = line.trim().strip_prefix("- ") else {
            continue;
        };
        let id = rest.split(" — ").next().unwrap_or(rest).trim();
        topic_row(&mut topics, id);
    }

    if topics.is_empty() {
        return None;
    }

    let mut page = String::from("## Topic map\n\n");
    page.push_str("### Topics\n\n");
    for (id, (kind, rels)) in &topics {
        if rels.is_empty() {
            page.push_str(&format!("- {id} — {kind}\n"));
        } else {
            let rels = rels.iter().cloned().collect::<Vec<_>>().join(", ");
            page.push_str(&format!("- {id} — {kind} — {rels}\n"));
        }
    }
    page.push_str("\n### Relationships\n\n");
    match map_owner {
        Some(owner) => {
            let anchor = super::render::heading_anchor(MAP_KIND);
            page.push_str(&format!("See [{owner}]({owner}.md#{anchor}).\n"));
        }
        None => {
            for (from, relationship, to) in &edges {
                let statement = format!("- {from} -[{relationship}]-> {to}\n");
                if !page.contains(&statement) {
                    page.push_str(&statement);
                }
            }
        }
    }
    Some(page)
}

/// Insert or touch one topic row, taking its entity type from the id's type
/// segment; an id outside `kaya/<Type>/...` contributes no row.
fn topic_row<'a, 'b>(topics: &'b mut BTreeMap<&'a str, Topic>, id: &'a str) -> &'b mut Topic {
    topics.entry(id).or_insert_with(|| {
        (
            id.split('/').nth(1).unwrap_or("Unknown").to_owned(),
            BTreeSet::new(),
        )
    })
}

/// The knowledge-index composes the census: each closed entity type with the
/// count and the node ids the compiled tree names. Ids whose type segment is
/// outside the closed set contribute nothing.
fn render_knowledge_index(docs: &Documents) -> Option<String> {
    let mut census: BTreeMap<&str, BTreeSet<&str>> = EntityType::ALL
        .iter()
        .map(|entity| (entity.as_str(), BTreeSet::new()))
        .collect();
    for document in docs.0.values() {
        for id in &document.event_ids {
            if let Some(kind) = id.split('/').nth(1)
                && let Some(bucket) = census.get_mut(kind)
            {
                bucket.insert(id.as_str());
            }
        }
    }
    let named: usize = census.values().map(|bucket| bucket.len()).sum();
    if named == 0 {
        return None;
    }

    let mut page = String::from("## Knowledge index\n\n");
    page.push_str("The compiled tree names these topics, per closed entity type.\n\n");
    for entity in EntityType::ALL {
        let bucket = &census[entity.as_str()];
        page.push_str(&format!(
            "### {kind} — {count}\n\n",
            kind = entity.as_str(),
            count = bucket.len(),
        ));
        if bucket.is_empty() {
            page.push_str("_(none)_\n\n");
        } else {
            for id in bucket {
                page.push_str(&format!("- {id}\n"));
            }
            page.push('\n');
        }
    }
    Some(page)
}

/// The example-gallery composes the experiment records: one row per record
/// document carrying the run, artifact, and observation ids that record
/// names, omitting the groups it does not name.
fn render_example_gallery(docs: &Documents) -> Option<String> {
    let mut rows: Vec<String> = Vec::new();
    for document in docs.0.values().filter(|d| d.kind == RECORD_KIND) {
        let runs = first_id(&document.body, "### Runs");
        let artifacts = first_id(&document.body, "### Artifacts");
        let observations = first_id(&document.body, "### Observations");
        if runs.is_none() && artifacts.is_none() && observations.is_none() {
            continue;
        }
        let mut groups: Vec<String> = Vec::new();
        if let Some(runs) = runs {
            groups.push(format!("runs {runs}"));
        }
        if let Some(artifacts) = artifacts {
            groups.push(format!("artifacts {artifacts}"));
        }
        if let Some(observations) = observations {
            groups.push(format!("observations {observations}"));
        }
        rows.push(groups.join(" — "));
    }
    if rows.is_empty() {
        return None;
    }

    let mut page = String::from("## Example gallery\n\n");
    page.push_str("### Recorded experiments\n\n");
    for row in &rows {
        page.push_str(&format!("- {row}\n"));
    }
    Some(page)
}

/// The api-catalogue composes the technical references: one row per spec id
/// with the node id that spec pins.
fn render_api_catalogue(docs: &Documents) -> Option<String> {
    let mut rows: Vec<String> = Vec::new();
    for document in docs.0.values().filter(|d| d.kind == TECH_KIND) {
        for line in section(&document.body, "### Specs").lines() {
            let Some(rest) = line.trim().strip_prefix("- ") else {
                continue;
            };
            let Some((node, spec)) = rest.split_once(" — ") else {
                continue;
            };
            let node = node.trim();
            let spec = spec.trim();
            if node.starts_with("kaya/") && !spec.is_empty() {
                rows.push(format!("- {spec} — {node}\n"));
            }
        }
    }
    if rows.is_empty() {
        return None;
    }

    let mut page = String::from("## Api catalogue\n\n");
    page.push_str("### Specs\n\n");
    for row in &rows {
        page.push_str(row);
    }
    Some(page)
}

/// The first `kaya/...` id a section lists, if any.
fn first_id(body: &str, heading: &str) -> Option<String> {
    for line in section(body, heading).lines() {
        let Some(rest) = line.trim().strip_prefix("- ") else {
            continue;
        };
        let token = rest.split(" — ").next().unwrap_or(rest).trim();
        if token.starts_with("kaya/") {
            return Some(token.to_owned());
        }
    }
    None
}

/// The text under one `### <heading>` section, up to the next heading.
///
/// The heading is matched at the start of a line, so prose that merely
/// mentions the heading text never mis-slices the body. A section the body
/// does not carry contributes no lines, so a renderer never depends on
/// sections the producer did not write.
fn section<'a>(body: &'a str, heading: &str) -> &'a str {
    let start = if body.starts_with(heading) {
        0
    } else {
        let needle = format!("\n{heading}");
        let Some(found) = body.find(&needle) else {
            return "";
        };
        found + 1
    };
    let after = &body[start + heading.len()..];
    let end = after
        .find("\n### ")
        .map(|found| found + 1)
        .unwrap_or(after.len());
    &after[..end]
}
