//! Contract tests for `kaya::promotion`.

use serde_json::json;

use kaya::event::{Actor, ActorType, Event, Provenance, SCHEMA_V0, Subject};
use kaya::promotion::{
    Assertion, PromotionState, Status, approve, cap_proposals, challenge, propose,
};

fn actor(r#type: ActorType, name: &str) -> Actor {
    Actor {
        r#type,
        name: name.to_string(),
    }
}

fn tool() -> Actor {
    actor(ActorType::Tool, "cargo-test")
}

fn agent() -> Actor {
    actor(ActorType::Agent, "analysis-agent")
}

fn human(name: &str) -> Actor {
    actor(ActorType::Human, name)
}

fn event(event_type: &str, actor: Actor, data: serde_json::Value) -> Event {
    Event {
        id: kaya::event::new_id(),
        schema: SCHEMA_V0.to_string(),
        r#type: event_type.to_string(),
        timestamp: "2026-11-16T00:00:00Z".to_string(),
        actor,
        subject: Subject {
            r#type: "record".to_string(),
            id: "kaya/record/x".to_string(),
        },
        data,
        provenance: Provenance {
            tool: "test".to_string(),
            version: "0".to_string(),
            reference: "test".to_string(),
        },
        artifacts: Vec::new(),
    }
}

fn make_assertion(id: &str, status: Status, cited: usize) -> Assertion {
    Assertion {
        id: id.to_string(),
        text: format!("interpretation {id}"),
        status,
        machine_proposed: false,
        event_ids: (0..cited).map(|index| format!("{id}-ev-{index}")).collect(),
        author: "alice".to_string(),
    }
}

#[test]
fn tool_cannot_propose() {
    let error = propose(
        &tool(),
        "the counts contradict H-001",
        &["ev-1".to_string()],
    )
    .expect_err("tool assertions are not admissible");
    assert_eq!(error.label, "kaya.promotion.tools_emit_events_only");
    assert!(
        error.message.contains("events only"),
        "refusal names the tool-writable categories: {}",
        error.message
    );
}

#[test]
fn agent_assertion_lands_proposed_and_machine_flagged() {
    let recorded = propose(
        &agent(),
        "this result contradicts H-001",
        &["ev-7".to_string()],
    )
    .expect("agents may propose");
    assert_eq!(recorded.r#type, "promotion.assertion.recorded");
    assert_eq!(recorded.data["status"], "proposed");
    assert_eq!(recorded.data["machine_proposed"].as_bool(), Some(true));

    let state = PromotionState::from_events(&[recorded]).expect("a well-formed proposal replays");
    assert_eq!(state.pending_count(), 1);
    assert_eq!(state.assertions.len(), 1);
    let assertion = &state.assertions[0];
    assert_eq!(assertion.status, Status::Proposed);
    assert!(assertion.machine_proposed);
    assert_eq!(assertion.text, "this result contradicts H-001");
    assert_eq!(assertion.event_ids, vec!["ev-7".to_string()]);
    assert!(
        state
            .assertions
            .iter()
            .all(|assertion| assertion.status != Status::Established),
        "an agent proposal never becomes a conclusion"
    );
}

#[test]
fn human_assertion_lands_proposed_with_author_and_event_references() {
    let recorded = propose(
        &human("alice"),
        "this result contradicts H-001",
        &["ev-1".to_string()],
    )
    .expect("humans may propose");
    assert_eq!(recorded.r#type, "promotion.assertion.recorded");
    assert_eq!(recorded.data["status"], "proposed");
    assert_eq!(recorded.data["event_ids"], json!(["ev-1"]));
    assert_eq!(recorded.data["machine_proposed"].as_bool(), Some(false));

    let state = PromotionState::from_events(&[recorded]).expect("a proposal replays");
    let assertion = &state.assertions[0];
    assert_eq!(assertion.status, Status::Proposed);
    assert_eq!(assertion.author, "alice");
    assert_eq!(assertion.event_ids, vec!["ev-1".to_string()]);
}

#[test]
fn approval_requires_human_and_records_provenance() {
    let recorded = propose(&agent(), "counts fell", &["ev-1".to_string()]).expect("agent proposes");
    let assertion_id = recorded.subject.id.clone();

    let refused = approve(&agent(), &assertion_id).expect_err("agents cannot approve");
    assert_eq!(refused.label, "kaya.promotion.approval_required");
    assert!(
        refused.message.contains("human"),
        "refusal names the missing human approval: {}",
        refused.message
    );

    let promotion = approve(&human("alice"), &assertion_id).expect("humans approve");
    assert_eq!(promotion.r#type, "promotion.conclusion.established");
    assert_eq!(promotion.data["assertion"], assertion_id);
    assert_eq!(promotion.data["approver"], "alice");

    let events = vec![recorded, promotion];
    let state = PromotionState::from_events(&events).expect("approval replays");
    assert_eq!(
        state.displayed_status(&assertion_id),
        Some(Status::Established)
    );
    assert_eq!(state.pending_count(), 0);
}

#[test]
fn unapproved_conclusion_is_refused() {
    let recorded =
        propose(&human("alice"), "counts fell", &["ev-1".to_string()]).expect("human proposes");
    let assertion_id = recorded.subject.id.clone();
    let forged = event(
        "promotion.conclusion.established",
        agent(),
        json!({"assertion": assertion_id, "approver": "analysis-agent"}),
    );

    let error = PromotionState::from_events(&[recorded, forged])
        .expect_err("non-human promotion events are refused");
    assert_eq!(error.label, "kaya.promotion.approval_required");
    assert!(
        error.message.contains("human"),
        "refusal names the missing approval: {}",
        error.message
    );
}

#[test]
fn approval_provenance_is_required() {
    let recorded =
        propose(&human("alice"), "counts fell", &["ev-1".to_string()]).expect("human proposes");
    let assertion_id = recorded.subject.id.clone();
    let forged = event(
        "promotion.conclusion.established",
        human("alice"),
        json!({"assertion": assertion_id}),
    );

    let error = PromotionState::from_events(&[recorded, forged])
        .expect_err("promotion without an approver is refused");
    assert_eq!(error.label, "kaya.promotion.approval_provenance");
}

#[test]
fn challenge_demotes_displayed_status_without_deleting_the_record() {
    let recorded = propose(&human("alice"), "H-001 is falsified", &["ev-1".to_string()])
        .expect("human proposes");
    let assertion_id = recorded.subject.id.clone();
    let established = approve(&human("alice"), &assertion_id).expect("human approves");
    let counter = challenge(&agent(), &assertion_id, &["ev-2".to_string()])
        .expect("counter-assertions attach");

    assert_eq!(counter.r#type, "promotion.conclusion.challenged");
    assert_eq!(counter.data["conclusion"], assertion_id);
    assert_eq!(counter.data["counter_event_ids"], json!(["ev-2"]));

    let ledger = vec![recorded.clone(), established.clone(), counter];
    let before = ledger.clone();
    let state = PromotionState::from_events(&ledger).expect("challenge replays");
    assert_eq!(
        state.displayed_status(&assertion_id),
        Some(Status::Challenged)
    );
    assert_eq!(ledger, before, "the original records stay byte-intact");
    assert_eq!(ledger[0], recorded);
    assert_eq!(ledger[1], established);
}

#[test]
fn challenge_requires_an_established_conclusion() {
    let recorded =
        propose(&human("alice"), "maybe", &["ev-1".to_string()]).expect("human proposes");
    let assertion_id = recorded.subject.id.clone();
    let counter = challenge(&human("alice"), &assertion_id, &["ev-2".to_string()])
        .expect("a challenge event can be emitted");

    let error = PromotionState::from_events(&[recorded, counter])
        .expect_err("proposals cannot be challenged");
    assert_eq!(error.label, "kaya.promotion.challenge_target");
}

#[test]
fn tools_cannot_challenge() {
    let error = challenge(&tool(), "assertion-1", &["ev-2".to_string()])
        .expect_err("tools do not run promotion commands");
    assert_eq!(error.label, "kaya.promotion.tools_emit_events_only");
}

#[test]
fn from_events_replays_state_in_order() {
    let first = propose(&human("alice"), "first", &["ev-1".to_string()]).expect("proposal");
    let first_id = first.subject.id.clone();
    let second = propose(&agent(), "second", &["ev-2".to_string()]).expect("proposal");
    let second_id = second.subject.id.clone();
    let established = approve(&human("alice"), &first_id).expect("approval");
    let demoted = challenge(&human("bob"), &first_id, &["ev-3".to_string()]).expect("challenge");
    let resolved = approve(&human("alice"), &first_id).expect("resolution approval");

    let state = PromotionState::from_events(&[first, second, established, demoted, resolved])
        .expect("replay");
    assert_eq!(
        state.displayed_status(&first_id),
        Some(Status::Established),
        "a later human approval resolves a challenge"
    );
    assert_eq!(
        state.displayed_status(&second_id),
        Some(Status::Proposed),
        "an unapproved proposal remains proposed"
    );
    assert_eq!(state.pending_count(), 1);
}

#[test]
fn cap_proposals_ranks_then_caps_at_budget() {
    let assertions = vec![
        make_assertion("a", Status::Proposed, 1),
        make_assertion("b", Status::Proposed, 3),
        make_assertion("c", Status::Proposed, 2),
        make_assertion("d", Status::Proposed, 3),
        make_assertion("z", Status::Established, 99),
    ];

    let capped = cap_proposals(&assertions, 2);
    let ids: Vec<&str> = capped
        .iter()
        .map(|assertion| assertion.id.as_str())
        .collect();
    assert_eq!(ids, vec!["b", "d"], "more cited events first, then id");

    assert!(cap_proposals(&assertions, 0).is_empty());
    assert_eq!(cap_proposals(&assertions, 10).len(), 4);
}

#[test]
fn proposals_never_expire_or_settle_silently() {
    let recorded = propose(&agent(), "still proposed", &["ev-1".to_string()]).expect("proposal");
    let events = vec![recorded];
    let first = PromotionState::from_events(&events).expect("replay");
    let again = PromotionState::from_events(&events).expect("replay");
    assert_eq!(first, again);
    assert_eq!(first.pending_count(), 1);
    assert_eq!(first.assertions[0].status, Status::Proposed);
    let capped = cap_proposals(&first.assertions, 1);
    assert_eq!(capped.len(), 1);
    assert_eq!(first.assertions[0].status, Status::Proposed);
}

#[test]
fn epistemic_categories_are_closed() {
    let foreign = event(
        "observation.recorded",
        tool(),
        json!({"category": "opinion", "note": "not an epistemic category"}),
    );
    let error =
        PromotionState::from_events(&[foreign]).expect_err("foreign categories are refused");
    assert_eq!(error.label, "kaya.promotion.category_closed");
    assert!(
        error.message.contains("event, assertion, conclusion"),
        "refusal names the closed categories: {}",
        error.message
    );
}

#[test]
fn proposal_channel_requires_status_author_and_event_references() {
    let malformed = event(
        "promotion.assertion.recorded",
        human("alice"),
        json!({"text": "missing the channel fields"}),
    );
    let error =
        PromotionState::from_events(&[malformed]).expect_err("an incomplete proposal is refused");
    assert_eq!(error.label, "kaya.promotion.proposal_channel");
    assert!(error.message.contains("status"));
    assert!(error.message.contains("event_ids"));
}
