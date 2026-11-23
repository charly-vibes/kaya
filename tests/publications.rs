//! Contract tests for the publication layer: closed product/collection/package
//! vocabularies with exactly one tier per name, refusal of inadmissible
//! inclusions and family members before any write, one rendering of each fact
//! with cross-references elsewhere, omission (not stubbing) of products without
//! an admissible backing document, and one source projected to Markdown pages
//! and a structured export that agree on event ids and rebuild byte-identically.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde_json::{Value, json};
use tempfile::TempDir;

use kaya::projection::{BlockKind, scan};
use kaya::publication::{
    COLLECTIONS, Decl, DocumentRef, Documents, PACKAGES, PRODUCTS, Tier, admission, build,
    read_documents, reader_question, resolve_name,
};

const OVERVIEW_KIND: &str = "orientation.project-overview";
const MAP_KIND: &str = "orientation.research-map";
const TECH_KIND: &str = "reuse-handoff.technical-reference";
const SPEC_KIND: &str = "exploration.experiment-specification";
const FINDINGS_KIND: &str = "understanding.findings-and-analysis";
const DECISION_KIND: &str = "understanding.decision-record";

/// The heading anchors cross-references point at: the document kind with every
/// character other than an alphanumeric, hyphen, or underscore removed.
const OVERVIEW_ANCHOR: &str = "orientationproject-overview";
const FINDINGS_ANCHOR: &str = "understandingfindings-and-analysis";
const DECISION_ANCHOR: &str = "understandingdecision-record";

const OVERVIEW_BODY: &str = "Kaya publishes client-facing documents from one source.";
const MAP_BODY: &str = "The research map links each fact to its event.";
const TECH_BODY: &str = "Reuse begins from the recorded handoff.";
const SPEC_BODY: &str = "The experiment specification states the intended change.";
const FINDINGS_BODY: &str = "The findings record what the runs taught the project.";
const DECISION_BODY: &str = "The decision record pins the shipped cache revision.";

/// The closed document vocabulary an admission row may name.
const DOCUMENT_KINDS: [&str; 11] = [
    "orientation.project-overview",
    "orientation.research-map",
    "exploration.exploration-brief",
    "exploration.experiment-specification",
    "exploration.experiment-record",
    "understanding.findings-and-analysis",
    "understanding.decision-record",
    "reuse-handoff.technical-reference",
    "reuse-handoff.reproduction-package",
    "reuse-handoff.knowledge-brief",
    "reuse-handoff.handoff",
];

/// The fixed reader-question binding, verbatim from the capability spec.
const QUESTIONS: [(&str, &str); 12] = [
    ("overview", "what is this and why does it matter"),
    ("getting-started", "how do I reach a first result"),
    ("tutorial", "can you teach me through a guided outcome"),
    ("how-to", "how do I accomplish this task"),
    ("concept", "why does it work this way"),
    ("reference", "what exactly is this"),
    ("example", "what can I do with it"),
    ("troubleshooting", "why is this broken and how do I fix it"),
    ("faq", "what do people keep asking"),
    ("status-compatibility", "what can I rely on"),
    ("release-migration", "what changed and what must I do"),
    ("architecture-design", "how is it structured and why"),
];

/// The full admission table, pinned verbatim from the capability spec and
/// `docs/src/document-taxonomy.md`: each product admits exactly these document
/// kinds, in this order.
const ADMISSIONS: [(&str, &[&str]); 12] = [
    ("overview", &["orientation.project-overview"]),
    (
        "getting-started",
        &[
            "orientation.project-overview",
            "reuse-handoff.reproduction-package",
        ],
    ),
    (
        "tutorial",
        &[
            "exploration.experiment-record",
            "exploration.exploration-brief",
        ],
    ),
    (
        "how-to",
        &[
            "reuse-handoff.technical-reference",
            "reuse-handoff.reproduction-package",
        ],
    ),
    (
        "concept",
        &[
            "understanding.decision-record",
            "understanding.findings-and-analysis",
        ],
    ),
    (
        "reference",
        &[
            "reuse-handoff.technical-reference",
            "orientation.research-map",
        ],
    ),
    ("example", &["exploration.experiment-record"]),
    ("troubleshooting", &["understanding.findings-and-analysis"]),
    (
        "faq",
        &[
            "understanding.findings-and-analysis",
            "understanding.decision-record",
        ],
    ),
    (
        "status-compatibility",
        &[
            "orientation.project-overview",
            "exploration.experiment-record",
        ],
    ),
    (
        "release-migration",
        &[
            "orientation.project-overview",
            "understanding.decision-record",
        ],
    ),
    (
        "architecture-design",
        &["understanding.decision-record", "orientation.research-map"],
    ),
];

fn kind_label(tier: Tier) -> &'static str {
    match tier {
        Tier::Product => "product",
        Tier::Collection => "collection",
        Tier::Package => "package",
    }
}

/// The document source keyed by generated-block name (the document kind).
fn documents(entries: &[(&str, &str, &[&str])]) -> Documents {
    let mut map = BTreeMap::new();
    for (kind, body, event_ids) in entries {
        map.insert(
            (*kind).to_owned(),
            DocumentRef {
                kind: (*kind).to_owned(),
                body: (*body).to_owned(),
                event_ids: event_ids.iter().map(|id| (*id).to_owned()).collect(),
            },
        );
    }
    Documents(map)
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

fn product(name: &str, includes: &[&str]) -> Decl {
    Decl {
        tier: Tier::Product,
        name: name.to_owned(),
        includes: strings(includes),
        members: Vec::new(),
    }
}

fn package(name: &str, members: &[&str]) -> Decl {
    Decl {
        tier: Tier::Package,
        name: name.to_owned(),
        includes: Vec::new(),
        members: strings(members),
    }
}

/// Every regular file under `root`, keyed by its relative path.
fn read_tree(root: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(root: &Path, dir: &Path, files: &mut BTreeMap<String, Vec<u8>>) {
        let mut paths: Vec<_> = fs::read_dir(dir)
            .expect("publication tree is readable")
            .map(|entry| entry.expect("directory entry is readable").path())
            .collect();
        paths.sort();
        for path in paths {
            if path.is_dir() {
                walk(root, &path, files);
            } else {
                let relative = path
                    .strip_prefix(root)
                    .expect("tree paths stay under the output root")
                    .to_string_lossy()
                    .into_owned();
                files.insert(relative, fs::read(&path).expect("tree file is readable"));
            }
        }
    }

    let mut files = BTreeMap::new();
    walk(root, root, &mut files);
    files
}

/// The Markdown pages of a built tree, keyed by relative path.
fn markdown(tree: &BTreeMap<String, Vec<u8>>) -> BTreeMap<String, String> {
    tree.iter()
        .filter(|(path, _)| path.ends_with(".md"))
        .map(|(path, bytes)| {
            (
                (*path).clone(),
                String::from_utf8((*bytes).clone()).expect("generated pages are UTF-8"),
            )
        })
        .collect()
}

fn assert_no_writes(out: &Path) {
    assert!(
        !out.join("publication.json").exists(),
        "a refused build must not write the export"
    );
    let entries: Vec<_> = fs::read_dir(out)
        .expect("the pre-created output directory is readable")
        .collect();
    assert!(
        entries.is_empty(),
        "a refused build must not write a single page"
    );
}

/// The body of the single generated block named `block` in `document`.
fn block_body(document: &str, block: &str) -> String {
    let mut bodies = scan(document)
        .expect("a written page has a valid managed-block grammar")
        .into_iter()
        .filter_map(|entry| match entry.kind {
            BlockKind::Generated(name) if name == block => Some(entry.body),
            _ => None,
        });
    let body = bodies
        .next()
        .unwrap_or_else(|| panic!("the page carries the {block} block: {document}"));
    assert!(
        bodies.next().is_none(),
        "the page carries exactly one {block} block: {document}"
    );
    body
}

/// The bytes `replace_generated` produces for a new page with `body`.
fn wrapped_page(block: &str, body: &str) -> String {
    format!("<!-- kaya:begin generated:{block} -->\n{body}<!-- kaya:end generated:{block} -->\n")
}

/// The `### <kind>` sections of a rendered product block body.
fn rendered_sections(body: &str) -> BTreeMap<String, String> {
    let mut sections = BTreeMap::new();
    let mut current: Option<String> = None;
    let mut buffer = String::new();
    for line in body.lines() {
        if let Some(kind) = line.strip_prefix("### ") {
            if let Some(name) = current.take() {
                sections.insert(name, std::mem::take(&mut buffer));
            }
            current = Some(kind.trim().to_owned());
        } else if current.is_some() {
            buffer.push_str(line);
            buffer.push('\n');
        }
    }
    if let Some(name) = current {
        sections.insert(name, buffer);
    }
    sections
}

/// The `kaya/...` node ids named in a body, sorted and deduplicated.
fn body_node_ids(body: &str) -> Vec<String> {
    let mut ids: Vec<String> = body
        .split(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '/' | '-' | '_' | '.')))
        .filter(|token| token.starts_with("kaya/"))
        .map(str::to_owned)
        .collect();
    ids.sort();
    ids.dedup();
    ids
}

/// A JSON object's keys, sorted.
fn object_keys(value: &Value) -> Vec<&str> {
    let mut keys: Vec<&str> = value
        .as_object()
        .unwrap_or_else(|| panic!("expected a JSON object: {value}"))
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort();
    keys
}

// 1. kaya.publications/Publication tiers
#[test]
fn tier_resolved_for_every_name_and_unknown_refused() {
    for (name, tier) in PRODUCTS
        .into_iter()
        .map(|name| (name, "product"))
        .chain(COLLECTIONS.into_iter().map(|name| (name, "collection")))
        .chain(PACKAGES.into_iter().map(|name| (name, "package")))
    {
        let (resolved_tier, resolved_name) = match resolve_name(name) {
            Ok(resolved) => resolved,
            Err(error) => panic!(
                "{name} must resolve to one tier: {}: {}",
                error.label, error.message
            ),
        };
        assert_eq!(
            kind_label(resolved_tier),
            tier,
            "{name} resolves to exactly one tier"
        );
        assert_eq!(resolved_name, name, "{name} resolves to its own spelling");
    }

    // `experiment-report` is a plausible product name outside the closed
    // product vocabulary; `blog` is a plausible fourth tier name.
    for value in ["experiment-report", "blog"] {
        let error = match resolve_name(value) {
            Ok(_) => panic!("{value} is outside every closed vocabulary and must be refused"),
            Err(error) => error,
        };
        assert_eq!(
            error.label, "kaya.publications.tier_resolved",
            "an unknown name is a labeled tier refusal: {}",
            error.message
        );
        assert!(
            error.message.contains(&format!("\"{value}\""))
                || error.message.contains(&format!("'{value}'")),
            "the refusal quotes the offending value `{value}`: {}",
            error.message
        );
        for entry in PRODUCTS.into_iter().chain(COLLECTIONS).chain(PACKAGES) {
            assert!(
                error.message.contains(entry),
                "the refusal lists the closed vocabularies, missing {entry}: {}",
                error.message
            );
        }
    }

    for product in PRODUCTS {
        assert!(
            QUESTIONS.iter().any(|(name, _)| *name == product),
            "product {product} must declare a reader question"
        );
        assert!(
            reader_question(product).is_some(),
            "product {product} binds exactly one reader question"
        );
    }
    for (product, question) in QUESTIONS {
        assert_eq!(
            reader_question(product),
            Some(question),
            "product {product} binds its fixed reader question"
        );
    }
    assert!(
        reader_question("experiment-report").is_none(),
        "a name outside the product vocabulary has no reader question"
    );

    for product in PRODUCTS {
        let row = admission(product);
        assert!(
            !row.is_empty(),
            "product {product} admits at least one document kind"
        );
        for kind in row {
            assert!(
                DOCUMENT_KINDS.contains(kind),
                "the admission row for {product} names a kind outside the document vocabulary: {kind}"
            );
        }
    }
}

// 1b. kaya.publications/Reader-question binding — the admission table is fixed
#[test]
fn admission_table_is_pinned_for_every_product() {
    for (product, row) in ADMISSIONS {
        assert_eq!(
            admission(product),
            row,
            "the admission row for {product} is fixed, in declaration order"
        );
        let mut seen = BTreeSet::new();
        for kind in row {
            assert!(
                DOCUMENT_KINDS.contains(kind),
                "{product} admits only document-vocabulary kinds, found {kind}"
            );
            assert!(
                seen.insert(*kind),
                "{product} admits {kind} once, never twice"
            );
        }
    }

    assert!(
        admission("experiment-report").is_empty(),
        "an unknown product admits nothing, so every inclusion is refused"
    );
}

// 2. kaya.publications.inadmissible_refused
#[test]
fn inadmissible_inclusion_is_refused() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    fs::create_dir(&out).expect("output directory");

    let decls = [product("faq", &[FINDINGS_KIND, SPEC_KIND])];
    let docs = documents(&[
        (FINDINGS_KIND, FINDINGS_BODY, &["evt-findings"]),
        (SPEC_KIND, SPEC_BODY, &["evt-spec"]),
    ]);

    let error = match build(&decls, &docs, &out) {
        Ok(_) => panic!("an include outside the faq admission row must be refused"),
        Err(error) => error,
    };

    assert_eq!(
        admission("faq"),
        &[
            "understanding.findings-and-analysis",
            "understanding.decision-record"
        ][..],
        "the faq admission row is fixed"
    );
    assert_eq!(
        error.label, "kaya.publications.inadmissible_refused",
        "an inadmissible inclusion is a labeled refusal: {}",
        error.message
    );
    assert!(
        error.message.contains("faq"),
        "the refusal names the product: {}",
        error.message
    );
    assert!(
        error.message.contains(SPEC_KIND),
        "the refusal names the offending kind: {}",
        error.message
    );
    for kind in admission("faq") {
        assert!(
            error.message.contains(*kind),
            "the refusal names the admission table row, missing {kind}: {}",
            error.message
        );
    }

    assert!(!out.join("faq.md").exists(), "no page is written");
    assert_no_writes(&out);
}

// 3. kaya.publications/Publication tiers — Package cannot list a family
#[test]
fn package_cannot_list_a_family() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    fs::create_dir(&out).expect("output directory");

    let decls = [package("project-site", &["exploration.experiment-record"])];
    let docs = documents(&[(OVERVIEW_KIND, OVERVIEW_BODY, &["evt-1"])]);

    let error = match build(&decls, &docs, &out) {
        Ok(_) => panic!("a package may only list products, never a document family"),
        Err(error) => error,
    };

    assert_eq!(
        error.label, "kaya.publications.package_members",
        "a non-product member is a labeled package refusal: {}",
        error.message
    );
    assert!(
        error.message.contains("exploration.experiment-record"),
        "the refusal names the family reference: {}",
        error.message
    );
    assert!(
        error.message.contains("package"),
        "the refusal names the package tier: {}",
        error.message
    );

    assert_no_writes(&out);
}

// 4. kaya.publications.fact_once
#[test]
fn shared_fact_is_rendered_once_with_a_cross_reference() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");

    let decls = [
        product("overview", &[OVERVIEW_KIND]),
        product("getting-started", &[OVERVIEW_KIND]),
        package("project-site", &["overview", "getting-started"]),
    ];
    let docs = documents(&[(OVERVIEW_KIND, OVERVIEW_BODY, &["evt-1"])]);

    let built = match build(&decls, &docs, &out) {
        Ok(built) => built,
        Err(error) => panic!(
            "the composition must build: {}: {}",
            error.label, error.message
        ),
    };

    let names: Vec<&str> = built.pages.iter().map(|page| page.name.as_str()).collect();
    assert_eq!(
        names,
        ["getting-started", "overview", "project-site"],
        "product and package pages are sorted by name"
    );

    let tree = read_tree(&out);
    let pages = markdown(&tree);
    let occurrences: usize = pages
        .values()
        .map(|text| text.matches(OVERVIEW_BODY).count())
        .sum();
    assert_eq!(
        occurrences, 1,
        "the shared fact body is rendered exactly once across the published tree"
    );

    let holder_file = pages
        .iter()
        .find(|(_, text)| text.contains(OVERVIEW_BODY))
        .map(|(path, _)| path.clone())
        .expect("one product page carries the fact body");
    let holder = holder_file.trim_end_matches(".md").to_owned();
    assert!(
        holder == "overview" || holder == "getting-started",
        "the fact body lives on a product page, not the package: {holder}"
    );
    let other = if holder == "overview" {
        "getting-started"
    } else {
        "overview"
    };

    assert!(
        pages[&holder_file].contains(&format!("### {OVERVIEW_KIND}")),
        "the rendering product carries the document block: {}",
        pages[&holder_file]
    );
    let other_page = pages
        .get(&format!("{other}.md"))
        .unwrap_or_else(|| panic!("{other} is emitted"));
    assert!(
        other_page.contains(&format!("See [{holder}]({holder}.md#{OVERVIEW_ANCHOR}).")),
        "the second product cross-references the first instead of repeating the body: {other_page}"
    );
    assert!(
        !other_page.contains(OVERVIEW_BODY),
        "the second product does not duplicate the shared fact body"
    );

    let package_page = pages
        .get("project-site.md")
        .expect("the package page is written");
    assert!(
        !package_page.contains(OVERVIEW_BODY),
        "the package embeds no product body text: {package_page}"
    );
    let package_body = block_body(package_page, "publication.package.project-site");
    let rows: Vec<&str> = package_body
        .lines()
        .filter(|line| !line.is_empty())
        .collect();
    assert_eq!(
        rows.len(),
        3,
        "the package block is a heading and one row per member: {package_page}"
    );
    assert!(
        rows[0].starts_with("## "),
        "the package block opens with a heading: {package_page}"
    );
    assert_eq!(rows[1], "- [overview](overview.md)");
    assert_eq!(rows[2], "- [getting-started](getting-started.md)");
}

// 5. kaya.publications.stub_free
#[test]
fn product_without_backing_document_is_omitted_without_a_page() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");

    let decls = [
        product("tutorial", &["exploration.experiment-record"]),
        package("project-site", &["tutorial"]),
    ];
    let docs = documents(&[(OVERVIEW_KIND, OVERVIEW_BODY, &["evt-1"])]);

    let built = match build(&decls, &docs, &out) {
        Ok(built) => built,
        Err(error) => panic!(
            "the omission must build: {}: {}",
            error.label, error.message
        ),
    };

    let names: Vec<&str> = built.pages.iter().map(|page| page.name.as_str()).collect();
    assert_eq!(
        names,
        ["project-site"],
        "an unbacked product gets no page; only the package is emitted"
    );
    assert!(
        !out.join("tutorial.md").exists(),
        "no stub page exists for a product without a backing document"
    );

    let omitted = built
        .omitted
        .iter()
        .find(|entry| entry.name == "tutorial")
        .expect("the omission of tutorial is reported");
    assert_eq!(
        omitted.reason, "no-backing-document",
        "the omission names its reason"
    );
    assert!(
        omitted.tier.contains("product"),
        "the omission reports the product tier: {}",
        omitted.tier
    );

    let tree = read_tree(&out);
    let pages = markdown(&tree);
    let package_page = pages
        .get("project-site.md")
        .expect("the package page is written");
    let package_body = block_body(package_page, "publication.package.project-site");
    let rows: Vec<&str> = package_body
        .lines()
        .filter(|line| !line.is_empty())
        .collect();
    assert_eq!(
        rows.len(),
        2,
        "the package block is a heading and one row per member: {package_page}"
    );
    assert!(
        rows[0].starts_with("## "),
        "the package block opens with a heading: {package_page}"
    );
    assert_eq!(
        rows[1], "- tutorial — unavailable",
        "the package names the omitted product unavailable instead of linking a missing page"
    );
}

// 6. kaya.publications.format_attribution
#[test]
fn markdown_and_export_share_one_source_and_rebuild_identically() {
    let tmp = TempDir::new().expect("temp dir");
    let first = tmp.path().join("first");
    let second = tmp.path().join("second");

    let decls = [
        product("overview", &[OVERVIEW_KIND]),
        product("reference", &[TECH_KIND, MAP_KIND]),
        package("project-site", &["overview", "reference"]),
    ];
    let docs = documents(&[
        (TECH_KIND, TECH_BODY, &["evt-tech"]),
        (MAP_KIND, MAP_BODY, &["evt-map"]),
    ]);

    for out in [&first, &second] {
        match build(&decls, &docs, out) {
            Ok(_) => {}
            Err(error) => panic!(
                "the project site must build: {}: {}",
                error.label, error.message
            ),
        }
    }

    let first_tree = read_tree(&first);
    let second_tree = read_tree(&second);
    assert_eq!(
        first_tree.keys().collect::<Vec<_>>(),
        second_tree.keys().collect::<Vec<_>>(),
        "both builds project the same files"
    );
    for (path, bytes) in &first_tree {
        assert_eq!(
            bytes, &second_tree[path],
            "{path} rebuilds byte-identically from unchanged inputs"
        );
    }

    match build(&decls, &docs, &first) {
        Ok(_) => {}
        Err(error) => panic!(
            "the project site must rebuild over its own output: {}: {}",
            error.label, error.message
        ),
    }
    assert_eq!(
        first_tree,
        read_tree(&first),
        "a rebuild over unchanged inputs leaves every already-written byte in place"
    );

    let pages = markdown(&first_tree);
    let reference_page = pages
        .get("reference.md")
        .expect("the reference product page is written");
    assert!(
        reference_page.contains(MAP_BODY),
        "the reference page carries the research-map body: {reference_page}"
    );
    assert!(
        reference_page.contains(TECH_BODY),
        "the reference page carries the technical-reference body: {reference_page}"
    );

    let export_bytes = first_tree
        .get("publication.json")
        .expect("the structured export is written");
    let export: Value =
        serde_json::from_slice(export_bytes).expect("the export is a JSON document");
    let mut objects = Vec::new();
    collect_objects(&export, &mut objects);

    assert_eq!(
        exported_kind(&objects, MAP_KIND).get("event_ids"),
        Some(&json!(["evt-map"])),
        "the export attributes the research-map fact to its event ids"
    );
    assert_eq!(
        exported_kind(&objects, TECH_KIND).get("event_ids"),
        Some(&json!(["evt-tech"])),
        "the export attributes the technical-reference fact to its event ids"
    );

    assert!(
        objects.iter().any(|object| {
            object.get("question").and_then(Value::as_str) == Some("what exactly is this")
        }),
        "the export binds the reference product to its reader question: {export}"
    );

    assert_eq!(
        exported_member(&objects, "overview")
            .get("emitted")
            .and_then(Value::as_bool),
        Some(false),
        "the export reports the overview member as not emitted"
    );
    assert_eq!(
        exported_member(&objects, "reference")
            .get("emitted")
            .and_then(Value::as_bool),
        Some(true),
        "the export reports the reference member as emitted"
    );
    assert!(
        objects.iter().any(|object| {
            object.get("name").and_then(Value::as_str) == Some("project-site")
                && object.get("members").is_some_and(Value::is_array)
        }),
        "the export carries the project-site package with its members: {export}"
    );
}

/// The exported document attribution naming `kind`.
fn exported_kind<'a>(objects: &[&'a Value], kind: &str) -> &'a Value {
    objects
        .iter()
        .copied()
        .find(|object| object.get("kind").and_then(Value::as_str) == Some(kind))
        .unwrap_or_else(|| panic!("the export attributes every source kind, missing {kind}"))
}

/// The exported package member naming `name`; members carry only a name and
/// an `emitted` flag, unlike product rows which also carry `documents`.
fn exported_member<'a>(objects: &[&'a Value], name: &str) -> &'a Value {
    objects
        .iter()
        .copied()
        .find(|object| {
            object.get("name").and_then(Value::as_str) == Some(name)
                && object.get("emitted").is_some()
                && object.get("documents").is_none()
                && object.get("question").is_none()
        })
        .unwrap_or_else(|| panic!("the export lists package member {name}"))
}

/// Collect every JSON object reachable from `value`, whatever its nesting.
fn collect_objects<'a>(value: &'a Value, objects: &mut Vec<&'a Value>) {
    match value {
        Value::Object(map) => {
            objects.push(value);
            for child in map.values() {
                collect_objects(child, objects);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_objects(item, objects);
            }
        }
        _ => {}
    }
}

// 7. kaya.publications/Publication tiers — Product and package composed
#[test]
fn project_site_over_overview_and_reference() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");

    let decls = [
        product("overview", &[OVERVIEW_KIND]),
        product("reference", &[TECH_KIND, MAP_KIND]),
        package("project-site", &["overview", "reference"]),
    ];
    let docs = documents(&[
        (TECH_KIND, TECH_BODY, &["evt-tech"]),
        (MAP_KIND, MAP_BODY, &["evt-map"]),
    ]);

    let built = match build(&decls, &docs, &out) {
        Ok(built) => built,
        Err(error) => panic!(
            "the project site must build: {}: {}",
            error.label, error.message
        ),
    };

    let names: Vec<&str> = built.pages.iter().map(|page| page.name.as_str()).collect();
    assert_eq!(
        names,
        ["project-site", "reference"],
        "the unbacked overview is omitted; the package and reference are emitted"
    );

    let reference = built
        .pages
        .iter()
        .find(|page| page.name == "reference")
        .expect("the reference product is emitted");
    assert_eq!(
        reference.question,
        Some("what exactly is this"),
        "the reference page names its reader question"
    );
    assert_eq!(
        reference.documents,
        [TECH_KIND, MAP_KIND],
        "the reference page admits its declared documents in declaration order"
    );
    assert!(
        reference.path.ends_with("reference.md"),
        "the reference page is written as reference.md"
    );

    let omitted = built
        .omitted
        .iter()
        .find(|entry| entry.name == "overview")
        .expect("the omission of overview is reported");
    assert_eq!(omitted.reason, "no-backing-document");
    assert!(
        omitted.tier.contains("product"),
        "the omission reports the product tier: {}",
        omitted.tier
    );

    assert!(
        !out.join("overview.md").exists(),
        "no overview page exists without a backing document"
    );

    let tree = read_tree(&out);
    let pages = markdown(&tree);
    let reference_page = pages
        .get("reference.md")
        .expect("the reference page is written");
    let expected = format!(
        "## Reference\n\nReader question: what exactly is this\n\n\
         ### {TECH_KIND}\n\n{TECH_BODY}\n\n\
         ### {MAP_KIND}\n\n{MAP_BODY}\n\n"
    );
    assert_eq!(
        block_body(reference_page, "publication.product.reference").trim_end_matches('\n'),
        expected.trim_end_matches('\n'),
        "the reference page is the exact product block over its admitted documents"
    );

    let package_page = pages
        .get("project-site.md")
        .expect("the package page is written");
    let package_body = block_body(package_page, "publication.package.project-site");
    let rows: Vec<&str> = package_body
        .lines()
        .filter(|line| !line.is_empty())
        .collect();
    assert_eq!(
        rows.len(),
        3,
        "the package block is a heading and one row per member: {package_page}"
    );
    assert!(
        rows[0].starts_with("## "),
        "the package block opens with a heading: {package_page}"
    );
    assert_eq!(
        rows[1], "- overview — unavailable",
        "the package names the omitted overview unavailable"
    );
    assert_eq!(
        rows[2], "- [reference](reference.md)",
        "the package links the emitted reference"
    );
    assert!(
        !package_page.contains(MAP_BODY) && !package_page.contains(TECH_BODY),
        "the package embeds neither product's content: {package_page}"
    );
}

// 8. kaya.publications.io — read_documents keeps the vocabulary, first wins
#[test]
fn read_documents_keeps_vocabulary_kinds_first_wins_across_files() {
    let tmp = TempDir::new().expect("temp dir");
    let research = tmp.path().join("research");
    fs::create_dir_all(research.join("experiments")).expect("research tree is creatable");

    let overview = format!(
        "<!-- kaya:begin generated:{OVERVIEW_KIND} -->\n\
         {OVERVIEW_BODY} cites kaya/Overview/o-one\n\
         <!-- kaya:end generated:{OVERVIEW_KIND} -->\n"
    );
    fs::write(research.join("overview.md"), overview).expect("overview is writable");

    let index = format!(
        "# Research index\n\n\
         <!-- kaya:begin generated:{MAP_KIND} -->\n\
         index map cites kaya/Orientation/m-two, kaya/Orientation/m-one and kaya/Orientation/m-one\n\
         <!-- kaya:end generated:{MAP_KIND} -->\n\n\
         <!-- kaya:begin generated:orientation.other-block -->\n\
         outside the vocabulary, cites kaya/Orientation/ignored\n\
         <!-- kaya:end generated:orientation.other-block -->\n"
    );
    fs::write(research.join("index.md"), index).expect("index is writable");
    let handoff = format!(
        "<!-- kaya:begin generated:{MAP_KIND} -->\n\
         handoff map cites kaya/Orientation/h-one\n\
         <!-- kaya:end generated:{MAP_KIND} -->\n"
    );
    fs::write(research.join("handoff.md"), handoff).expect("handoff is writable");

    let docs = read_documents(&research).expect("the compiled documents are readable");

    assert_eq!(
        docs.0.len(),
        2,
        "only document-vocabulary kinds are kept: {:?}",
        docs.0.keys().collect::<Vec<_>>()
    );
    let map = &docs.0[MAP_KIND];
    assert_eq!(map.kind, MAP_KIND);
    assert!(
        map.body.contains("index map cites"),
        "the first compiled kind in read order wins: {}",
        map.body
    );
    assert!(
        !map.body.contains("handoff map cites"),
        "a duplicate kind in a later file never overrides the first: {}",
        map.body
    );
    assert_eq!(
        map.event_ids,
        ["kaya/Orientation/m-one", "kaya/Orientation/m-two"],
        "event ids are sorted and deduplicated from the winning body"
    );
    let overview = &docs.0[OVERVIEW_KIND];
    assert_eq!(overview.kind, OVERVIEW_KIND);
    assert!(
        overview.body.contains(OVERVIEW_BODY),
        "the overview document is read from overview.md: {}",
        overview.body
    );
    assert_eq!(
        overview.event_ids,
        ["kaya/Overview/o-one"],
        "event ids are sorted and deduplicated from the overview body"
    );
    assert!(
        !docs.0.contains_key("orientation.other-block"),
        "a generated block outside the document vocabulary is not a document"
    );
    assert!(
        !research.join("reproduction-package.md").exists(),
        "missing compiled files are tolerated, not fabricated"
    );

    // The document read from overview.md is the overview product's backing.
    let out = tmp.path().join("out");
    let decls = [
        product("overview", &[OVERVIEW_KIND]),
        package("project-site", &["overview"]),
    ];
    let built = build(&decls, &docs, &out).expect("the overview document backs its product");
    assert!(
        built.pages.iter().any(|page| page.name == "overview"),
        "the overview product is emitted: {:?}",
        built.pages
    );
    assert!(
        out.join("overview.md").exists(),
        "the backed overview page is written"
    );
    let page = fs::read_to_string(out.join("overview.md")).expect("the overview page is readable");
    assert!(
        page.contains(OVERVIEW_BODY),
        "the page renders the compiled overview body: {page}"
    );
    assert!(
        !built.omitted.iter().any(|entry| entry.name == "overview"),
        "the overview document leaves the product unomitted: {:?}",
        built.omitted
    );
}

// 9. kaya.publications.duplicate_block — one file, one block per name
#[test]
fn read_documents_refuses_duplicate_generated_blocks_in_one_file() {
    let tmp = TempDir::new().expect("temp dir");
    let research = tmp.path().join("research");
    fs::create_dir_all(&research).expect("research tree is creatable");
    let index = format!(
        "<!-- kaya:begin generated:{MAP_KIND} -->\n\
         first map\n\
         <!-- kaya:end generated:{MAP_KIND} -->\n\n\
         <!-- kaya:begin generated:{MAP_KIND} -->\n\
         second map\n\
         <!-- kaya:end generated:{MAP_KIND} -->\n"
    );
    fs::write(research.join("index.md"), index).expect("index is writable");

    let error =
        read_documents(&research).expect_err("a repeated block in one file must be refused");
    assert_eq!(
        error.label, "kaya.publications.duplicate_block",
        "a repeated generated block is a labeled refusal: {}",
        error.message
    );
    assert!(
        error.message.contains("index.md"),
        "the refusal names the file: {}",
        error.message
    );
    assert!(
        error.message.contains(MAP_KIND),
        "the refusal names the repeated block: {}",
        error.message
    );
}

// 10. kaya.publications.io — malformed markup is refused with the file name
#[test]
fn read_documents_refuses_malformed_markup_naming_the_file() {
    let tmp = TempDir::new().expect("temp dir");
    let research = tmp.path().join("research");
    fs::create_dir_all(&research).expect("research tree is creatable");
    fs::write(
        research.join("index.md"),
        "<!-- kaya:begin generated:orientation.research-map -->\nbody without a closing marker\n",
    )
    .expect("index is writable");

    let error =
        read_documents(&research).expect_err("broken managed-block grammar must be refused");
    assert_eq!(
        error.label, "kaya.publications.io",
        "a malformed compiled document is an IO refusal: {}",
        error.message
    );
    assert!(
        error.message.contains("index.md"),
        "the refusal names the malformed file: {}",
        error.message
    );
}

// 11. kaya.publications.stub_free — an empty block is no backing document
#[test]
fn whitespace_only_block_is_treated_as_absent() {
    let tmp = TempDir::new().expect("temp dir");
    let research = tmp.path().join("research");
    fs::create_dir_all(&research).expect("research tree is creatable");
    let index = format!(
        "<!-- kaya:begin generated:{OVERVIEW_KIND} -->\n   \n\n\
         <!-- kaya:end generated:{OVERVIEW_KIND} -->\n"
    );
    fs::write(research.join("index.md"), index).expect("index is writable");

    let docs = read_documents(&research).expect("the compiled documents are readable");
    assert!(
        !docs.0.contains_key(OVERVIEW_KIND),
        "a whitespace-only generated block backs no document"
    );

    let out = tmp.path().join("out");
    let decls = [
        product("overview", &[OVERVIEW_KIND]),
        package("project-site", &["overview"]),
    ];
    let built = build(&decls, &docs, &out).expect("the omission must build");
    assert!(
        !out.join("overview.md").exists(),
        "an empty backing document produces no stub page"
    );
    assert!(
        built
            .omitted
            .iter()
            .any(|entry| { entry.name == "overview" && entry.reason == "no-backing-document" }),
        "the empty-backing product is reported omitted: {:?}",
        built.omitted
    );
    let package_page =
        fs::read_to_string(out.join("project-site.md")).expect("the package page is written");
    assert!(
        block_body(&package_page, "publication.package.project-site")
            .contains("- overview — unavailable"),
        "the package names the omitted product unavailable: {package_page}"
    );
}

// 12. kaya.publications.format_attribution — build writes one managed block
#[test]
fn build_writes_one_managed_block_and_preserves_surrounding_bytes() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    fs::create_dir(&out).expect("output directory");

    let block = "publication.product.reference";
    let reference_path = out.join("reference.md");
    let authored = format!(
        "# Authored intro\n\nKeep this byte for byte.\n\n\
         <!-- kaya:begin generated:{block} -->\n\
         stale body\n\
         <!-- kaya:end generated:{block} -->\n\n\
         Authored footer stays.\n"
    );
    fs::write(&reference_path, &authored).expect("the authored page is writable");

    let decls = [product("reference", &[MAP_KIND])];
    let docs = documents(&[(MAP_KIND, MAP_BODY, &["evt-map"])]);
    let built = build(&decls, &docs, &out).expect("the page must build over authored bytes");
    let page = fs::read_to_string(&reference_path).expect("the page is readable");

    assert!(
        page.starts_with("# Authored intro\n\nKeep this byte for byte.\n\n"),
        "authored bytes before the block survive: {page}"
    );
    assert!(
        page.ends_with("\n\nAuthored footer stays.\n"),
        "authored bytes after the block survive: {page}"
    );
    assert_eq!(
        page.matches(&format!("kaya:begin generated:{block}"))
            .count(),
        1,
        "the page carries exactly one begin marker: {page}"
    );
    assert_eq!(
        page.matches(&format!("kaya:end generated:{block}")).count(),
        1,
        "the page carries exactly one end marker: {page}"
    );
    let expected = format!(
        "## Reference\n\nReader question: what exactly is this\n\n\
         ### {MAP_KIND}\n\n{MAP_BODY}\n\n"
    );
    assert_eq!(
        block_body(&page, block).trim_end_matches('\n'),
        expected.trim_end_matches('\n'),
        "build writes the rendered page into the managed block, not around it"
    );
    assert!(
        !page.contains("stale body"),
        "the old block body is replaced: {page}"
    );
    assert!(
        built.summary.recomputed.iter().any(|name| name == block),
        "the page block is reported recomputed: {:?}",
        built.summary.recomputed
    );
    assert!(
        built.summary.unchanged.is_empty(),
        "nothing was already current: {:?}",
        built.summary.unchanged
    );
}

// 13. kaya.publications.stub_free — a disappeared backing document removes the page
#[test]
fn stale_page_is_removed_when_its_backing_document_disappears() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    let decls = [
        product("overview", &[OVERVIEW_KIND]),
        package("project-site", &["overview"]),
    ];

    let backed = documents(&[(OVERVIEW_KIND, OVERVIEW_BODY, &["evt-overview"])]);
    let first = build(&decls, &backed, &out).expect("the backed product must build");
    assert!(
        first.pages.iter().any(|page| page.name == "overview"),
        "the backed product is emitted: {:?}",
        first.pages
    );
    assert!(out.join("overview.md").exists(), "the backed page exists");

    let second = build(&decls, &Documents::default(), &out).expect("the omission must rebuild");
    assert!(
        !out.join("overview.md").exists(),
        "a page whose backing document disappeared is removed, not left stale"
    );
    assert!(
        !second.pages.iter().any(|page| page.name == "overview"),
        "an unbacked product leaves no page entry: {:?}",
        second.pages
    );
    let omitted = second
        .omitted
        .iter()
        .find(|entry| entry.name == "overview")
        .expect("the omission is reported");
    assert_eq!(omitted.reason, "no-backing-document");
    assert!(omitted.tier.contains("product"));
    let package_page =
        fs::read_to_string(out.join("project-site.md")).expect("the package page is written");
    assert!(
        block_body(&package_page, "publication.package.project-site")
            .contains("- overview — unavailable"),
        "the package row flips to unavailable: {package_page}"
    );
}

// 14. kaya.publications.tier_resolved — declared tier must be the resolved tier
#[test]
fn declared_tier_that_differs_from_the_resolved_tier_is_refused() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    fs::create_dir(&out).expect("output directory");
    let decls = [Decl {
        tier: Tier::Package,
        name: "overview".to_owned(),
        includes: Vec::new(),
        members: Vec::new(),
    }];

    let error = build(&decls, &Documents::default(), &out)
        .expect_err("a product name declared as a package must be refused");
    assert_eq!(
        error.label, "kaya.publications.tier_resolved",
        "a declared/resolved tier mismatch is a tier refusal: {}",
        error.message
    );
    assert!(
        error.message.contains("overview"),
        "the refusal names the offending publication: {}",
        error.message
    );
    assert_no_writes(&out);
}

// 15. kaya.publications.duplicate_declaration — one declaration per name
#[test]
fn duplicate_declarations_are_refused_before_any_write() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    fs::create_dir(&out).expect("output directory");
    let decls = [
        product("overview", &[OVERVIEW_KIND]),
        product("overview", &[OVERVIEW_KIND]),
    ];

    let error = build(&decls, &Documents::default(), &out)
        .expect_err("the same publication declared twice must be refused");
    assert_eq!(
        error.label, "kaya.publications.duplicate_declaration",
        "a repeated declaration is a labeled refusal: {}",
        error.message
    );
    assert!(
        error.message.contains("overview"),
        "the refusal names the duplicate publication: {}",
        error.message
    );
    assert_no_writes(&out);
}

// 16. kaya.publications.tier_fields — each tier owns exactly one field
#[test]
fn product_with_members_and_package_with_includes_are_refused() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    fs::create_dir(&out).expect("output directory");

    let mut product_with_members = product("overview", &[OVERVIEW_KIND]);
    product_with_members.members = strings(&["reference"]);
    let error = build(&[product_with_members], &Documents::default(), &out)
        .expect_err("a product does not list package members");
    assert_eq!(
        error.label, "kaya.publications.tier_fields",
        "a stray field on a product is a labeled refusal: {}",
        error.message
    );
    for needle in ["overview", "product", "member"] {
        assert!(
            error.message.contains(needle),
            "the refusal names the publication, tier and stray field ({needle}): {}",
            error.message
        );
    }
    assert_no_writes(&out);

    let mut package_with_includes = package("project-site", &["overview"]);
    package_with_includes.includes = strings(&[OVERVIEW_KIND]);
    let docs = documents(&[(OVERVIEW_KIND, OVERVIEW_BODY, &["evt-1"])]);
    let decls = [product("overview", &[OVERVIEW_KIND]), package_with_includes];
    let error = build(&decls, &docs, &out).expect_err("a package does not compose document kinds");
    assert_eq!(
        error.label, "kaya.publications.tier_fields",
        "a stray field on a package is a labeled refusal: {}",
        error.message
    );
    for needle in ["project-site", "package", "include"] {
        assert!(
            error.message.contains(needle),
            "the refusal names the publication, tier and stray field ({needle}): {}",
            error.message
        );
    }
    assert_no_writes(&out);
}

// 17. kaya.publications.tier_unimplemented — collections are not silently dropped
#[test]
fn collection_declaration_is_refused_as_unimplemented() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    fs::create_dir(&out).expect("output directory");
    let decls = [Decl {
        tier: Tier::Collection,
        name: "glossary".to_owned(),
        includes: Vec::new(),
        members: Vec::new(),
    }];

    let error = build(&decls, &Documents::default(), &out)
        .expect_err("a collection declaration must not be silently dropped");
    assert_eq!(
        error.label, "kaya.publications.tier_unimplemented",
        "an unimplemented tier is a labeled refusal: {}",
        error.message
    );
    assert!(
        error.message.contains("glossary"),
        "the refusal names the collection: {}",
        error.message
    );
    assert!(
        error.message.contains("implemented"),
        "the refusal states collections are not implemented yet: {}",
        error.message
    );
    assert_no_writes(&out);
}

// 18. kaya.publications.package_members — members are declared products
#[test]
fn package_member_must_be_a_declared_product() {
    let tmp = TempDir::new().expect("temp dir");

    for member in ["glossary", "project-site", "experiment-report"] {
        let out = tmp.path().join(format!("out-{member}"));
        fs::create_dir(&out).expect("output directory");
        let decls = [package("project-site", &[member])];
        let error = build(&decls, &Documents::default(), &out)
            .expect_err("only products may be package members");
        assert_eq!(
            error.label, "kaya.publications.package_members",
            "a non-product member is a labeled refusal: {}",
            error.message
        );
        assert!(
            error.message.contains(member),
            "the refusal names the offending member: {}",
            error.message
        );
        assert!(
            error.message.contains("project-site"),
            "the refusal names the package: {}",
            error.message
        );
        assert_no_writes(&out);
    }

    // A name in the product vocabulary is still refused when the build never
    // declared that product.
    let out = tmp.path().join("out-undeclared");
    fs::create_dir(&out).expect("output directory");
    let decls = [
        product("overview", &[OVERVIEW_KIND]),
        package("project-site", &["reference"]),
    ];
    let error = build(&decls, &Documents::default(), &out)
        .expect_err("a package cannot reference an undeclared product");
    assert_eq!(
        error.label, "kaya.publications.package_members",
        "an undeclared product member is a labeled refusal: {}",
        error.message
    );
    assert!(
        error.message.contains("reference"),
        "the refusal names the undeclared member: {}",
        error.message
    );
    assert!(
        error.message.contains("project-site"),
        "the refusal names the package: {}",
        error.message
    );
    assert_no_writes(&out);
}

// 19. kaya.publications.document_kind — map key and DocumentRef agree
#[test]
fn document_kind_mismatch_is_refused_before_any_write() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    fs::create_dir(&out).expect("output directory");

    let mut map = BTreeMap::new();
    map.insert(
        OVERVIEW_KIND.to_owned(),
        DocumentRef {
            kind: MAP_KIND.to_owned(),
            body: OVERVIEW_BODY.to_owned(),
            event_ids: Vec::new(),
        },
    );
    let docs = Documents(map);
    let decls = [product("overview", &[OVERVIEW_KIND])];

    let error = build(&decls, &docs, &out)
        .expect_err("a document whose kind differs from its map key must be refused");
    assert_eq!(
        error.label, "kaya.publications.document_kind",
        "a kind/key mismatch is a labeled refusal: {}",
        error.message
    );
    assert!(
        error.message.contains(OVERVIEW_KIND),
        "the refusal names the map key: {}",
        error.message
    );
    assert!(
        error.message.contains(MAP_KIND),
        "the refusal names the differing DocumentRef kind: {}",
        error.message
    );
    assert_no_writes(&out);
}

// 20. kaya.publications.stub_free — only present kinds get a heading
#[test]
fn partial_includes_render_only_present_kinds() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    let decls = [product("reference", &[TECH_KIND, MAP_KIND])];
    let docs = documents(&[(MAP_KIND, MAP_BODY, &["evt-map"])]);

    build(&decls, &docs, &out).expect("a partially backed product still builds");
    let page = fs::read_to_string(out.join("reference.md")).expect("the page is written");
    let body = block_body(&page, "publication.product.reference");

    assert!(
        body.contains(&format!("### {MAP_KIND}")),
        "the present kind renders: {body}"
    );
    assert!(
        !body.contains(TECH_KIND),
        "the absent kind renders nothing: {body}"
    );
    assert_eq!(
        body.matches("### ").count(),
        1,
        "no empty heading is left for an absent kind: {body}"
    );
}

// 21. kaya.publications.fact_once — a duplicated include renders once
#[test]
fn duplicate_include_renders_the_body_once() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    let decls = [product("reference", &[MAP_KIND, MAP_KIND])];
    let docs = documents(&[(MAP_KIND, MAP_BODY, &["evt-map"])]);

    let built = build(&decls, &docs, &out).expect("the duplicated include still builds");
    assert!(
        built.pages.iter().any(|page| page.name == "reference"),
        "the reference product is emitted"
    );
    let page = fs::read_to_string(out.join("reference.md")).expect("the page is written");
    let body = block_body(&page, "publication.product.reference");
    assert_eq!(
        body.matches(&format!("### {MAP_KIND}")).count(),
        1,
        "the duplicated include gets one heading: {body}"
    );
    assert_eq!(
        body.matches(MAP_BODY).count(),
        1,
        "the duplicated include renders its body once: {body}"
    );
}

// 22. kaya.publications.format_attribution — exact export shape
#[test]
fn export_reports_emitted_products_with_exact_keys() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    let decls = [
        product("overview", &[OVERVIEW_KIND]),
        product("reference", &[MAP_KIND]),
        package("project-site", &["overview", "reference"]),
    ];
    let docs = documents(&[(MAP_KIND, MAP_BODY, &["evt-map"])]);

    build(&decls, &docs, &out).expect("the project site must build");
    let export: Value = serde_json::from_str(
        &fs::read_to_string(out.join("publication.json")).expect("the export is written"),
    )
    .expect("the export is JSON");

    let products = export["products"]
        .as_array()
        .expect("the export lists products");
    assert_eq!(
        products.len(),
        2,
        "only declared products are exported: {export}"
    );
    assert_eq!(products[0]["name"], json!("overview"));
    assert_eq!(products[1]["name"], json!("reference"));
    for row in products {
        assert_eq!(
            object_keys(row),
            ["documents", "emitted", "name", "question"],
            "a product row carries exactly its name, question, emitted flag and documents: {row}"
        );
    }

    assert_eq!(
        products[0]["emitted"],
        json!(false),
        "an omitted product is exported with emitted=false"
    );
    assert_eq!(
        products[0]["documents"],
        json!([]),
        "an omitted product keeps an empty documents list"
    );
    assert_eq!(
        products[0]["question"],
        json!("what is this and why does it matter")
    );
    assert_eq!(
        products[1]["emitted"],
        json!(true),
        "an emitted product is exported with emitted=true"
    );
    let documents = products[1]["documents"]
        .as_array()
        .expect("the emitted product lists its documents");
    assert_eq!(documents.len(), 1);
    assert_eq!(
        object_keys(&documents[0]),
        ["event_ids", "kind"],
        "a document row carries exactly its kind and event ids: {}",
        documents[0]
    );
    assert_eq!(documents[0]["kind"], json!(MAP_KIND));
    assert_eq!(documents[0]["event_ids"], json!(["evt-map"]));

    let packages = export["packages"]
        .as_array()
        .expect("the export lists packages");
    assert_eq!(
        packages.len(),
        1,
        "only declared packages are exported: {export}"
    );
    assert_eq!(object_keys(&packages[0]), ["members", "name"]);
    let members = packages[0]["members"]
        .as_array()
        .expect("the package lists its members");
    assert_eq!(members.len(), 2);
    for member in members {
        assert_eq!(
            object_keys(member),
            ["emitted", "name"],
            "a package member row carries exactly its name and emitted flag: {member}"
        );
    }
    assert_eq!(members[0]["name"], json!("overview"));
    assert_eq!(members[0]["emitted"], json!(false));
    assert_eq!(members[1]["name"], json!("reference"));
    assert_eq!(members[1]["emitted"], json!(true));
}

// 23. kaya.publications.format_attribution — a rebuild rewrites nothing
#[test]
fn rebuild_reports_unchanged_blocks_and_leaves_bytes_and_mtime_alone() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    let decls = [
        product("reference", &[MAP_KIND]),
        package("project-site", &["reference"]),
    ];
    let docs = documents(&[(MAP_KIND, MAP_BODY, &["evt-map"])]);

    let first = build(&decls, &docs, &out).expect("the project site must build");
    let reference_block = "publication.product.reference";
    let package_block = "publication.package.project-site";
    for name in [reference_block, package_block, "publication.json"] {
        assert!(
            first
                .summary
                .recomputed
                .iter()
                .any(|candidate| candidate == name),
            "a fresh build recomputes {name}: {:?}",
            first.summary.recomputed
        );
    }
    assert!(
        first.summary.unchanged.is_empty(),
        "a fresh build has nothing already current: {:?}",
        first.summary.unchanged
    );

    let reference_path = out.join("reference.md");
    let package_path = out.join("project-site.md");
    let export_path = out.join("publication.json");
    let expected_package = wrapped_page(
        package_block,
        "## Project-site\n\n- [reference](reference.md)\n",
    );
    assert_eq!(
        fs::read_to_string(&package_path).expect("the package page is written"),
        expected_package,
        "the package page is exactly one managed block with the navigation body"
    );
    let mtimes = [
        fs::metadata(&reference_path)
            .expect("page metadata")
            .modified()
            .expect("page mtime"),
        fs::metadata(&package_path)
            .expect("page metadata")
            .modified()
            .expect("page mtime"),
        fs::metadata(&export_path)
            .expect("export metadata")
            .modified()
            .expect("export mtime"),
    ];

    let second = build(&decls, &docs, &out).expect("the rebuild must build");
    assert!(
        second.summary.recomputed.is_empty(),
        "an unchanged rebuild recomputes nothing: {:?}",
        second.summary.recomputed
    );
    for name in [reference_block, package_block, "publication.json"] {
        assert!(
            second
                .summary
                .unchanged
                .iter()
                .any(|candidate| candidate == name),
            "an unchanged rebuild reports {name} unchanged: {:?}",
            second.summary.unchanged
        );
    }
    assert_eq!(
        fs::read_to_string(&package_path).expect("the package page is readable"),
        expected_package,
        "an unchanged rebuild leaves the page bytes alone"
    );
    let rebuilt_mtimes = [
        fs::metadata(&reference_path)
            .expect("page metadata")
            .modified()
            .expect("page mtime"),
        fs::metadata(&package_path)
            .expect("page metadata")
            .modified()
            .expect("page mtime"),
        fs::metadata(&export_path)
            .expect("export metadata")
            .modified()
            .expect("export mtime"),
    ];
    assert_eq!(
        rebuilt_mtimes, mtimes,
        "unchanged files are not rewritten, so their mtimes stay put"
    );

    let tree = read_tree(&out);
    let refused = [product("faq", &[SPEC_KIND])];
    let error =
        build(&refused, &docs, &out).expect_err("faq cannot include an experiment specification");
    assert_eq!(error.label, "kaya.publications.inadmissible_refused");
    assert_eq!(
        read_tree(&out),
        tree,
        "a refusal leaves an existing tree byte-identical"
    );
}

// 24. kaya.publications.format_attribution — pages and export agree per document
#[test]
fn export_and_page_attribution_agree_after_one_document_changes() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    let decls = [product("reference", &[TECH_KIND, MAP_KIND])];

    let first = documents(&[
        (
            TECH_KIND,
            "Technical reference cites kaya/Evidence/t-one see above",
            &["kaya/Evidence/t-one"],
        ),
        (
            MAP_KIND,
            "Research map cites kaya/Evidence/m-one see above",
            &["kaya/Evidence/m-one"],
        ),
    ]);
    build(&decls, &first, &out).expect("the first build succeeds");

    let second = documents(&[
        (
            TECH_KIND,
            "Technical reference cites kaya/Evidence/t-one see above",
            &["kaya/Evidence/t-one"],
        ),
        (
            MAP_KIND,
            "Research map now cites kaya/Evidence/m-two see above",
            &["kaya/Evidence/m-two"],
        ),
    ]);
    build(&decls, &second, &out).expect("the changed document rebuilds");

    let page = fs::read_to_string(out.join("reference.md")).expect("the page is written");
    let sections = rendered_sections(&block_body(&page, "publication.product.reference"));
    assert_eq!(
        sections.len(),
        2,
        "both declared documents render after the rebuild: {sections:?}"
    );

    let export: Value = serde_json::from_str(
        &fs::read_to_string(out.join("publication.json")).expect("the export is written"),
    )
    .expect("the export is JSON");
    let reference = export["products"]
        .as_array()
        .expect("the export lists products")
        .iter()
        .find(|product| product["name"] == json!("reference"))
        .expect("the export names the reference product");
    let documents = reference["documents"]
        .as_array()
        .expect("the reference product lists its documents");
    assert_eq!(documents.len(), 2);
    for document in documents {
        let kind = document["kind"]
            .as_str()
            .expect("every document kind is a string");
        let exported: Vec<String> = document["event_ids"]
            .as_array()
            .expect("every document lists its event ids")
            .iter()
            .map(|id| {
                id.as_str()
                    .expect("every exported event id is a string")
                    .to_owned()
            })
            .collect();
        let section = sections
            .get(kind)
            .unwrap_or_else(|| panic!("the page renders {kind}: {sections:?}"));
        assert_eq!(
            body_node_ids(section),
            exported,
            "the page and export name the same event ids for {kind}"
        );
    }
    let map = documents
        .iter()
        .find(|document| document["kind"] == json!(MAP_KIND))
        .expect("the export names the changed research map");
    assert_eq!(
        map["event_ids"],
        json!(["kaya/Evidence/m-two"]),
        "the rebuilt attribution carries the changed document's new ids"
    );
}

// 25. kaya.publications.fact_once — the understanding document backs its
// products, and each document body renders once across their pages
#[test]
fn understanding_document_backs_concept_faq_and_troubleshooting_once() {
    let tmp = TempDir::new().expect("temp dir");
    let research = tmp.path().join("research");
    fs::create_dir_all(&research).expect("research tree is creatable");
    let findings = format!(
        "<!-- kaya:begin generated:{FINDINGS_KIND} -->\n\
         ### Findings\n\n\
         - kaya/Finding/f-hit-rate — {FINDINGS_BODY}\n\
         <!-- kaya:end generated:{FINDINGS_KIND} -->\n"
    );
    let decision = format!(
        "<!-- kaya:begin generated:{DECISION_KIND} -->\n\
         ### Decisions\n\n\
         - kaya/Decision/d-ship-cache — {DECISION_BODY}\n\
         <!-- kaya:end generated:{DECISION_KIND} -->\n"
    );
    fs::write(
        research.join("understanding.md"),
        format!("# Understanding\n\n{findings}\n{decision}"),
    )
    .expect("understanding is writable");

    let docs = read_documents(&research).expect("the compiled documents are readable");
    let kinds: Vec<&str> = docs.0.keys().map(String::as_str).collect();
    assert_eq!(
        kinds,
        vec![DECISION_KIND, FINDINGS_KIND],
        "both understanding blocks are compiled documents"
    );
    assert_eq!(
        docs.0[FINDINGS_KIND].event_ids,
        ["kaya/Finding/f-hit-rate"],
        "the findings document attributes its rendered id"
    );
    assert!(
        docs.0[FINDINGS_KIND].body.contains(FINDINGS_BODY),
        "the findings document keeps its body: {}",
        docs.0[FINDINGS_KIND].body
    );
    assert_eq!(
        docs.0[DECISION_KIND].event_ids,
        ["kaya/Decision/d-ship-cache"],
        "the decision document attributes its rendered id"
    );
    assert!(
        docs.0[DECISION_KIND].body.contains(DECISION_BODY),
        "the decision document keeps its body: {}",
        docs.0[DECISION_KIND].body
    );

    let out = tmp.path().join("out");
    let decls = [
        product("concept", &[DECISION_KIND, FINDINGS_KIND]),
        product("faq", &[FINDINGS_KIND, DECISION_KIND]),
        product("troubleshooting", &[FINDINGS_KIND]),
    ];
    let built = build(&decls, &docs, &out).expect("the understanding products must build");
    assert!(
        built.omitted.is_empty(),
        "every declared product has an admissible backing document: {:?}",
        built.omitted
    );

    let pages = markdown(&read_tree(&out));
    for name in ["concept.md", "faq.md", "troubleshooting.md"] {
        assert!(
            pages.contains_key(name),
            "a backed product writes its page, missing {name}: {:?}",
            pages.keys().collect::<Vec<_>>()
        );
    }

    // Each document body is rendered once, in the first product page that
    // admits it; every later page cross-references it.
    for (kind, anchor, body) in [
        (FINDINGS_KIND, FINDINGS_ANCHOR, FINDINGS_BODY),
        (DECISION_KIND, DECISION_ANCHOR, DECISION_BODY),
    ] {
        let occurrences: usize = pages.values().map(|page| page.matches(body).count()).sum();
        assert_eq!(
            occurrences, 1,
            "{body} is rendered exactly once across the product pages: {pages:?}"
        );
        assert!(
            pages["concept.md"].contains(&format!("### {kind}")),
            "the concept page carries the {kind} section: {}",
            pages["concept.md"]
        );
        assert!(
            pages["faq.md"].contains(&format!("See [concept](concept.md#{anchor}).")),
            "the faq page cross-references the {kind} body: {}",
            pages["faq.md"]
        );
    }
    let troubleshooting = &pages["troubleshooting.md"];
    assert!(
        troubleshooting.contains(&format!("See [concept](concept.md#{FINDINGS_ANCHOR}).")),
        "the troubleshooting page cross-references the findings body: {troubleshooting}"
    );
    assert!(
        !troubleshooting.contains(DECISION_BODY),
        "troubleshooting admits no decision record: {troubleshooting}"
    );
}

// 26. kaya.publications.io — the block name, not the file name, names the kind
#[test]
fn read_documents_reads_a_vocabulary_kind_from_a_non_canonical_file() {
    let tmp = TempDir::new().expect("temp dir");
    let research = tmp.path().join("research");
    fs::create_dir_all(&research).expect("research tree is creatable");
    // A reuse-handoff kind compiled into index.md — the file a reader keyed on
    // canonical per-kind paths would never consult.
    let index = format!(
        "# Research index\n\n\
         <!-- kaya:begin generated:{TECH_KIND} -->\n\
         {TECH_BODY} cites kaya/Document/technical-reference\n\
         <!-- kaya:end generated:{TECH_KIND} -->\n\n\
         <!-- kaya:begin generated:reuse-handoff.other-block -->\n\
         outside the vocabulary, cites kaya/Document/ignored\n\
         <!-- kaya:end generated:reuse-handoff.other-block -->\n"
    );
    fs::write(research.join("index.md"), index).expect("index is writable");

    let docs = read_documents(&research).expect("the compiled documents are readable");

    assert_eq!(
        docs.0.len(),
        1,
        "only document-vocabulary kinds are kept: {:?}",
        docs.0.keys().collect::<Vec<_>>()
    );
    let technical = &docs.0[TECH_KIND];
    assert_eq!(
        technical.kind, TECH_KIND,
        "the DocumentRef names the kind its block declares"
    );
    assert!(
        technical.body.contains(TECH_BODY),
        "the block is read from index.md even though no technical-reference.md exists: {}",
        technical.body
    );
    assert_eq!(
        technical.event_ids,
        ["kaya/Document/technical-reference"],
        "event ids come from the block body"
    );
    assert!(
        !docs.0.contains_key("reuse-handoff.other-block"),
        "a generated block outside the document vocabulary is not a document"
    );
    assert!(
        !research.join("technical-reference.md").exists(),
        "the kind is not restricted to a canonical filename"
    );

    // The reference product takes its backing from the block in index.md.
    let out = tmp.path().join("out");
    let decls = [
        product("reference", &[TECH_KIND]),
        package("project-site", &["reference"]),
    ];
    let built = build(&decls, &docs, &out).expect("the non-canonical file backs its product");
    assert!(
        built.pages.iter().any(|page| page.name == "reference"),
        "the reference product is emitted: {:?}",
        built.pages
    );
    assert!(
        !built.omitted.iter().any(|entry| entry.name == "reference"),
        "the technical-reference block leaves the product unomitted: {:?}",
        built.omitted
    );
    let page = fs::read_to_string(out.join("reference.md")).expect("the reference page is written");
    assert!(
        page.contains(TECH_BODY),
        "the page renders the compiled technical-reference body: {page}"
    );
}

// 12. kaya.publications.block_vocabulary — a publication page's managed block
// name is inside the publication vocabulary, and a stale or misspelled one is
// refused before anything is written.
#[test]
fn publication_page_with_a_misspelled_block_is_refused() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    fs::create_dir(&out).expect("output directory");

    let stale = "publication.produkt.reference";
    let page = format!(
        "<!-- kaya:begin generated:{stale} -->\n\
         stale body from a misspelled tier\n\
         <!-- kaya:end generated:{stale} -->\n"
    );
    let page_path = out.join("reference.md");
    fs::write(&page_path, &page).expect("the page is writable");

    let decls = [product("reference", &[MAP_KIND])];
    let docs = documents(&[(MAP_KIND, MAP_BODY, &["evt-map"])]);

    let error = build(&decls, &docs, &out)
        .expect_err("a page outside the publication vocabulary is refused");
    assert_eq!(
        error.label, "kaya.publications.block_vocabulary",
        "a block outside the publication vocabulary is a labeled refusal: {}",
        error.message
    );
    assert!(
        error.message.contains(stale),
        "the refusal names the offending block: {}",
        error.message
    );
    assert!(
        error.message.contains("reference.md"),
        "the refusal names the page it refused: {}",
        error.message
    );
    assert_eq!(
        fs::read_to_string(&page_path).expect("the page is readable"),
        page,
        "a refused build leaves the page byte-identical"
    );
}

#[test]
fn publication_page_with_a_document_block_is_refused() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    fs::create_dir(&out).expect("output directory");

    let foreign = MAP_KIND;
    let page = format!(
        "<!-- kaya:begin generated:{foreign} -->\n\
         a research document block inside a publication page\n\
         <!-- kaya:end generated:{foreign} -->\n"
    );
    let page_path = out.join("reference.md");
    fs::write(&page_path, &page).expect("the page is writable");

    let decls = [product("reference", &[MAP_KIND])];
    let docs = documents(&[(MAP_KIND, MAP_BODY, &["evt-map"])]);

    let error = build(&decls, &docs, &out)
        .expect_err("a research document block inside a publication page is refused");
    assert_eq!(
        error.label, "kaya.publications.block_vocabulary",
        "a document-vocabulary block in a publication page is refused: {}",
        error.message
    );
    assert!(
        error.message.contains(foreign),
        "the refusal names the offending block: {}",
        error.message
    );
    assert_eq!(
        fs::read_to_string(&page_path).expect("the page is readable"),
        page,
        "a refused build leaves the page byte-identical"
    );
}

#[test]
fn publication_page_with_a_name_outside_its_tier_is_refused() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    fs::create_dir(&out).expect("output directory");

    // "glossary" is a collection name, so it does not belong under the product
    // tier: the vocabulary is tier-scoped, not a flat name list.
    let wrong_tier = "publication.product.glossary";
    let page = format!(
        "<!-- kaya:begin generated:{wrong_tier} -->\n\
         a collection name under the product tier\n\
         <!-- kaya:end generated:{wrong_tier} -->\n"
    );
    let page_path = out.join("reference.md");
    fs::write(&page_path, &page).expect("the page is writable");

    let decls = [product("reference", &[MAP_KIND])];
    let docs = documents(&[(MAP_KIND, MAP_BODY, &["evt-map"])]);

    let error =
        build(&decls, &docs, &out).expect_err("a name outside the tier's vocabulary is refused");
    assert_eq!(
        error.label, "kaya.publications.block_vocabulary",
        "a name outside the tier vocabulary is refused: {}",
        error.message
    );
    assert!(
        error.message.contains(wrong_tier),
        "the refusal names the offending block: {}",
        error.message
    );
}
