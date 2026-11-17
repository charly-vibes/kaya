//! Contract tests for the developer-guide products: each of the six emits only
//! from its admission row, binds its fixed reader question, refuses an
//! inadmissible include naming the product, the offending kind, and the row,
//! omits itself (never stubs itself) when no admissible document is compiled,
//! renders a shared fact once with a cross-reference elsewhere, and appears in
//! the developer-guide package as a link or an unavailable marker.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use tempfile::TempDir;

use kaya::projection::{BlockKind, scan};
use kaya::publication::{Decl, DocumentRef, Documents, Tier, admission, build};

const OVERVIEW_KIND: &str = "orientation.project-overview";
const REPRODUCTION_KIND: &str = "reuse-handoff.reproduction-package";
const TECH_KIND: &str = "reuse-handoff.technical-reference";
const DECISION_KIND: &str = "understanding.decision-record";
const FINDINGS_KIND: &str = "understanding.findings-and-analysis";
const RECORD_KIND: &str = "exploration.experiment-record";

/// The heading anchors cross-references point at: the document kind with every
/// character other than an alphanumeric, hyphen, or underscore removed.
const REPRODUCTION_ANCHOR: &str = "reuse-handoffreproduction-package";
const FINDINGS_ANCHOR: &str = "understandingfindings-and-analysis";
const DECISION_ANCHOR: &str = "understandingdecision-record";

const OVERVIEW_BODY: &str = "Kaya publishes client-facing documents from one source.";
const REPRODUCTION_BODY: &str = "The reproduction package records how to repeat the result.";
const TECH_BODY: &str = "The technical reference names the interface a reader can rely on.";
const DECISION_BODY: &str = "The decision record pins the shipped cache revision.";
const FINDINGS_BODY: &str = "The findings record what the runs taught the project.";
const RECORD_BODY: &str = "The experiment record lists the runs that were executed.";

/// The closed document vocabulary, so a page can be checked against every kind
/// it might have leaked.
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

/// The developer-guide products, in the package's member order.
const DEVELOPER_PRODUCTS: [&str; 6] = [
    "getting-started",
    "how-to",
    "concept",
    "example",
    "troubleshooting",
    "faq",
];

/// Each product's admission row, verbatim from the capability spec and
/// `docs/src/document-taxonomy.md`: a product admits exactly these document
/// kinds, in this order.
const ADMISSIONS: [(&str, &[&str]); 6] = [
    ("getting-started", &[OVERVIEW_KIND, REPRODUCTION_KIND]),
    ("how-to", &[TECH_KIND, REPRODUCTION_KIND]),
    ("concept", &[DECISION_KIND, FINDINGS_KIND]),
    ("example", &[RECORD_KIND]),
    ("troubleshooting", &[FINDINGS_KIND]),
    ("faq", &[FINDINGS_KIND, DECISION_KIND]),
];

/// The fixed reader question bound to each developer product, verbatim from the
/// capability spec.
const QUESTIONS: [(&str, &str); 6] = [
    ("getting-started", "how do I reach a first result"),
    ("how-to", "how do I accomplish this task"),
    ("concept", "why does it work this way"),
    ("example", "what can I do with it"),
    ("troubleshooting", "why is this broken and how do I fix it"),
    ("faq", "what do people keep asking"),
];

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

/// The pinned admission row for a developer product.
fn row(product: &str) -> &'static [&'static str] {
    ADMISSIONS
        .iter()
        .find(|(name, _)| *name == product)
        .map(|(_, row)| *row)
        .unwrap_or_else(|| panic!("{product} is a developer-guide product"))
}

/// The six developer products declared with their admission rows, plus the
/// developer-guide package over all six.
fn developer_declarations() -> Vec<Decl> {
    let mut decls: Vec<Decl> = DEVELOPER_PRODUCTS
        .iter()
        .map(|name| product(name, row(name)))
        .collect();
    decls.push(package("developer-guide", &DEVELOPER_PRODUCTS));
    decls
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

/// The product block body the renderer must produce: a heading, the fixed
/// reader question, and one section per composed kind — the document body for
/// the page that owns it, a cross-reference for every later page.
fn expected_page(title: &str, question: &str, sections: &[(&str, &str)]) -> String {
    let mut body = format!("## {title}\n\nReader question: {question}\n\n");
    for (kind, content) in sections {
        body.push_str(&format!("### {kind}\n\n{content}\n\n"));
    }
    body
}

// 1. kaya.publications/Justified emission — a product emits its admission row
// and nothing else
#[test]
fn developer_products_emit_only_from_admission_rows() {
    for (name, expected) in ADMISSIONS {
        assert_eq!(
            admission(name),
            expected,
            "the {name} admission row is fixed, in declaration order"
        );
    }

    let docs = documents(&[
        (OVERVIEW_KIND, OVERVIEW_BODY, &["kaya/Question/q-one"]),
        (
            REPRODUCTION_KIND,
            REPRODUCTION_BODY,
            &["kaya/ReproductionPackage/r-one"],
        ),
        (TECH_KIND, TECH_BODY, &["kaya/TechnicalReference/t-one"]),
        (DECISION_KIND, DECISION_BODY, &["kaya/Decision/d-one"]),
        (FINDINGS_KIND, FINDINGS_BODY, &["kaya/Finding/f-one"]),
        (RECORD_KIND, RECORD_BODY, &["kaya/Experiment/exp-one"]),
    ]);

    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    let built = match build(&developer_declarations(), &docs, &out) {
        Ok(built) => built,
        Err(error) => panic!(
            "the developer products must build: {}: {}",
            error.label, error.message
        ),
    };

    assert!(
        built.omitted.is_empty(),
        "every developer product has an admissible backing document: {:?}",
        built.omitted
    );

    let names: Vec<&str> = built.pages.iter().map(|page| page.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "concept",
            "developer-guide",
            "example",
            "faq",
            "getting-started",
            "how-to",
            "troubleshooting",
        ],
        "the product and package pages are sorted by name"
    );

    for page in &built.pages {
        let Some((_, question)) = QUESTIONS
            .iter()
            .find(|(name, _)| *name == page.name.as_str())
        else {
            assert_eq!(
                page.name, "developer-guide",
                "only the package page is not a product"
            );
            assert_eq!(page.question, None, "a package binds no reader question");
            assert!(
                page.documents.is_empty(),
                "a package composes no documents: {:?}",
                page.documents
            );
            continue;
        };
        assert_eq!(
            page.question,
            Some(*question),
            "{} binds its fixed reader question",
            page.name
        );
        assert_eq!(
            page.documents,
            strings(row(&page.name)),
            "{} composes exactly its present admitted kinds, in row order",
            page.name
        );
    }

    let pages = markdown(&read_tree(&out));
    let reproduction_ref =
        format!("See [getting-started](getting-started.md#{REPRODUCTION_ANCHOR}).");
    let findings_ref = format!("See [concept](concept.md#{FINDINGS_ANCHOR}).");
    let decision_ref = format!("See [concept](concept.md#{DECISION_ANCHOR}).");

    for (name, expected) in [
        (
            "concept",
            expected_page(
                "Concept",
                "why does it work this way",
                &[
                    (DECISION_KIND, DECISION_BODY),
                    (FINDINGS_KIND, FINDINGS_BODY),
                ],
            ),
        ),
        (
            "example",
            expected_page(
                "Example",
                "what can I do with it",
                &[(RECORD_KIND, RECORD_BODY)],
            ),
        ),
        (
            "faq",
            expected_page(
                "Faq",
                "what do people keep asking",
                &[
                    (FINDINGS_KIND, findings_ref.as_str()),
                    (DECISION_KIND, decision_ref.as_str()),
                ],
            ),
        ),
        (
            "getting-started",
            expected_page(
                "Getting-started",
                "how do I reach a first result",
                &[
                    (OVERVIEW_KIND, OVERVIEW_BODY),
                    (REPRODUCTION_KIND, REPRODUCTION_BODY),
                ],
            ),
        ),
        (
            "how-to",
            expected_page(
                "How-to",
                "how do I accomplish this task",
                &[
                    (TECH_KIND, TECH_BODY),
                    (REPRODUCTION_KIND, reproduction_ref.as_str()),
                ],
            ),
        ),
        (
            "troubleshooting",
            expected_page(
                "Troubleshooting",
                "why is this broken and how do I fix it",
                &[(FINDINGS_KIND, findings_ref.as_str())],
            ),
        ),
    ] {
        let page = pages
            .get(&format!("{name}.md"))
            .unwrap_or_else(|| panic!("the {name} page is written"));
        assert_eq!(
            block_body(page, &format!("publication.product.{name}")),
            expected,
            "the {name} page renders exactly its admitted kinds: {page}"
        );
        let sections: Vec<String> =
            rendered_sections(&block_body(page, &format!("publication.product.{name}")))
                .into_keys()
                .collect();
        let mut expected_sections = strings(row(name));
        expected_sections.sort();
        assert_eq!(
            sections, expected_sections,
            "the {name} page has exactly one section per admitted kind"
        );
    }

    for (name, admitted) in ADMISSIONS {
        let page = &pages[&format!("{name}.md")];
        for kind in DOCUMENT_KINDS {
            if !admitted.contains(&kind) {
                assert!(
                    !page.contains(kind),
                    "{name} admits no {kind}, so its page must never name it: {page}"
                );
            }
        }
    }

    for body in [
        OVERVIEW_BODY,
        REPRODUCTION_BODY,
        TECH_BODY,
        DECISION_BODY,
        FINDINGS_BODY,
        RECORD_BODY,
    ] {
        let occurrences: usize = pages.values().map(|text| text.matches(body).count()).sum();
        assert_eq!(
            occurrences, 1,
            "each fact body is rendered exactly once across the tree: {body:?}"
        );
    }
}

// 2. kaya.publications.inadmissible_refused — the row, not the product's wishes
#[test]
fn inadmissible_developer_include_is_refused() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    fs::create_dir(&out).expect("output directory");

    let decls = [product("example", &[RECORD_KIND, FINDINGS_KIND])];
    let docs = documents(&[
        (RECORD_KIND, RECORD_BODY, &["kaya/Experiment/exp-one"]),
        (FINDINGS_KIND, FINDINGS_BODY, &["kaya/Finding/f-one"]),
    ]);

    let error = match build(&decls, &docs, &out) {
        Ok(_) => panic!("an include outside the example admission row must be refused"),
        Err(error) => error,
    };

    assert_eq!(
        admission("example"),
        &[RECORD_KIND][..],
        "the example admission row is fixed"
    );
    assert_eq!(
        error.label, "kaya.publications.inadmissible_refused",
        "an inadmissible inclusion is a labeled refusal: {}",
        error.message
    );
    assert!(
        error.message.contains("example"),
        "the refusal names the product: {}",
        error.message
    );
    assert!(
        error.message.contains(FINDINGS_KIND),
        "the refusal names the offending kind: {}",
        error.message
    );
    assert!(
        error.message.contains(RECORD_KIND),
        "the refusal names the admission row: {}",
        error.message
    );

    assert!(
        !out.join("example.md").exists(),
        "a refused build writes no page for the offending product"
    );
    assert_no_writes(&out);
}

// 3. kaya.publications.stub_free — no backing document, no page
#[test]
fn developer_products_without_backing_are_omitted() {
    let docs = documents(&[(FINDINGS_KIND, FINDINGS_BODY, &["kaya/Finding/f-one"])]);

    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    let built = match build(&developer_declarations(), &docs, &out) {
        Ok(built) => built,
        Err(error) => panic!(
            "the omission must build: {}: {}",
            error.label, error.message
        ),
    };

    let names: Vec<&str> = built.pages.iter().map(|page| page.name.as_str()).collect();
    assert_eq!(
        names,
        ["concept", "developer-guide", "faq", "troubleshooting"],
        "only the products the finding backs get a page, beside the package"
    );

    let omitted: Vec<(&str, &str, &str)> = built
        .omitted
        .iter()
        .map(|entry| (entry.tier, entry.name.as_str(), entry.reason))
        .collect();
    assert_eq!(
        omitted,
        [
            ("product", "example", "no-backing-document"),
            ("product", "getting-started", "no-backing-document"),
            ("product", "how-to", "no-backing-document"),
        ],
        "the unbacked products are omitted with their reason, sorted by name"
    );

    for name in ["example", "getting-started", "how-to"] {
        assert!(
            !out.join(format!("{name}.md")).exists(),
            "an omitted product writes no stub page: {name}"
        );
    }

    let pages = markdown(&read_tree(&out));
    let findings_ref = format!("See [concept](concept.md#{FINDINGS_ANCHOR}).");
    assert_eq!(
        block_body(&pages["concept.md"], "publication.product.concept"),
        expected_page(
            "Concept",
            "why does it work this way",
            &[(FINDINGS_KIND, FINDINGS_BODY)],
        ),
        "the concept page renders the one present admitted kind"
    );
    assert_eq!(
        block_body(&pages["faq.md"], "publication.product.faq"),
        expected_page(
            "Faq",
            "what do people keep asking",
            &[(FINDINGS_KIND, findings_ref.as_str())],
        ),
        "the faq page cross-references the shared finding instead of repeating it"
    );
    assert_eq!(
        block_body(
            &pages["troubleshooting.md"],
            "publication.product.troubleshooting"
        ),
        expected_page(
            "Troubleshooting",
            "why is this broken and how do I fix it",
            &[(FINDINGS_KIND, findings_ref.as_str())],
        ),
        "the troubleshooting page cross-references the shared finding"
    );

    let occurrences: usize = pages
        .values()
        .map(|text| text.matches(FINDINGS_BODY).count())
        .sum();
    assert_eq!(
        occurrences, 1,
        "the one compiled finding body renders exactly once: {pages:?}"
    );
}

// 4. kaya.publications.fact_once — concept and faq share findings and decision
#[test]
fn shared_developer_fact_renders_once() {
    let docs = documents(&[
        (FINDINGS_KIND, FINDINGS_BODY, &["kaya/Finding/f-one"]),
        (DECISION_KIND, DECISION_BODY, &["kaya/Decision/d-one"]),
    ]);

    let decls = [
        product("concept", row("concept")),
        product("faq", row("faq")),
        package("developer-guide", &["concept", "faq"]),
    ];

    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    let built = match build(&decls, &docs, &out) {
        Ok(built) => built,
        Err(error) => panic!(
            "the shared composition must build: {}: {}",
            error.label, error.message
        ),
    };
    assert!(
        built.omitted.is_empty(),
        "both products have an admissible backing document: {:?}",
        built.omitted
    );

    let pages = markdown(&read_tree(&out));
    let findings_ref = format!("See [concept](concept.md#{FINDINGS_ANCHOR}).");
    let decision_ref = format!("See [concept](concept.md#{DECISION_ANCHOR}).");

    assert_eq!(
        block_body(&pages["concept.md"], "publication.product.concept"),
        expected_page(
            "Concept",
            "why does it work this way",
            &[
                (DECISION_KIND, DECISION_BODY),
                (FINDINGS_KIND, FINDINGS_BODY),
            ],
        ),
        "the first product in page order carries both shared fact bodies"
    );
    assert_eq!(
        block_body(&pages["faq.md"], "publication.product.faq"),
        expected_page(
            "Faq",
            "what do people keep asking",
            &[
                (FINDINGS_KIND, findings_ref.as_str()),
                (DECISION_KIND, decision_ref.as_str()),
            ],
        ),
        "the faq page only cross-references the bodies concept owns"
    );

    for body in [FINDINGS_BODY, DECISION_BODY] {
        let occurrences: usize = pages.values().map(|text| text.matches(body).count()).sum();
        assert_eq!(
            occurrences, 1,
            "a shared fact body renders exactly once across the pages: {body:?}"
        );
        assert!(
            !pages["developer-guide.md"].contains(body),
            "the package embeds no product body text: {body:?}"
        );
    }
}

// 5. kaya.publications.package_members — the package links or marks unavailable
#[test]
fn developer_guide_package_links_emitted_products() {
    let docs = documents(&[
        (FINDINGS_KIND, FINDINGS_BODY, &["kaya/Finding/f-one"]),
        (RECORD_KIND, RECORD_BODY, &["kaya/Experiment/exp-one"]),
    ]);

    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    let built = match build(&developer_declarations(), &docs, &out) {
        Ok(built) => built,
        Err(error) => panic!("the package must build: {}: {}", error.label, error.message),
    };

    let omitted: Vec<&str> = built
        .omitted
        .iter()
        .map(|entry| entry.name.as_str())
        .collect();
    assert_eq!(
        omitted,
        ["getting-started", "how-to"],
        "the finding and the experiment record back the other four products"
    );

    let pages = markdown(&read_tree(&out));
    let package_page = pages
        .get("developer-guide.md")
        .expect("the package page is written");
    assert_eq!(
        block_body(package_page, "publication.package.developer-guide"),
        concat!(
            "## Developer-guide\n\n",
            "- getting-started — unavailable\n",
            "- how-to — unavailable\n",
            "- [concept](concept.md)\n",
            "- [example](example.md)\n",
            "- [troubleshooting](troubleshooting.md)\n",
            "- [faq](faq.md)\n",
        ),
        "the package names all six members in declaration order, linking the emitted ones: {package_page}"
    );

    for body in [FINDINGS_BODY, RECORD_BODY] {
        assert!(
            !package_page.contains(body),
            "the package page carries navigation, never a product body: {body:?}"
        );
    }

    let package_built = built
        .pages
        .iter()
        .find(|page| page.name == "developer-guide")
        .expect("the package page is reported");
    assert_eq!(
        package_built.question, None,
        "a package binds no reader question"
    );
    assert!(
        package_built.documents.is_empty(),
        "a package composes no documents: {:?}",
        package_built.documents
    );
}
