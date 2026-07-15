//! LogLine Pocket Runtime — outcome tests for `check`.
//!
//! These tests pin the three-outcome contract (AcceptCandidate / Ghost /
//! Reject), the precedence rule (Reject > Ghost > Accept), and the aux
//! preservation rule.

use logline_pocket_runtime::{check, LogLineDraft, PocketRuntimeRuling};
use serde_json::{json, Value};

fn s(v: &str) -> Option<Value> {
    Some(Value::String(v.to_string()))
}

fn complete_valid_draft() -> LogLineDraft {
    LogLineDraft {
        who: s("dan"),
        did: s("update_host_runtime"),
        this: s("lab512"),
        when: s("2026-05-18T15:00:00Z"),
        confirmed_by: s("dan"),
        if_ok: s("emit_logline"),
        if_doubt: s("ghost_and_clarify"),
        if_not: s("refuse"),
        status: s("candidate"),
        aux: None,
    }
}

#[test]
fn complete_valid_draft_accepts_candidate() {
    let ruling = check(complete_valid_draft());
    match ruling {
        PocketRuntimeRuling::AcceptCandidate {
            normalized,
            warnings,
        } => {
            assert!(warnings.is_empty(), "no warnings expected on clean draft");
            assert_eq!(normalized.who, s("dan"));
            assert_eq!(normalized.status, s("candidate"));
            assert!(normalized.aux.is_none());
        }
        other => panic!("expected AcceptCandidate, got {other:?}"),
    }
}

#[test]
fn missing_when_ghosts() {
    let mut draft = complete_valid_draft();
    draft.when = None;
    let ruling = check(draft);
    match ruling {
        PocketRuntimeRuling::Ghost { ghosts, partial } => {
            assert_eq!(ghosts.len(), 1);
            assert_eq!(ghosts[0].slot, "when");
            assert_eq!(ghosts[0].reason, "slot_missing");
            // Partial preserves what the caller did provide.
            assert_eq!(partial.who, s("dan"));
            assert!(partial.when.is_none());
        }
        other => panic!("expected Ghost, got {other:?}"),
    }
}

#[test]
fn missing_if_doubt_ghosts() {
    let mut draft = complete_valid_draft();
    draft.if_doubt = None;
    let ruling = check(draft);
    match ruling {
        PocketRuntimeRuling::Ghost { ghosts, partial: _ } => {
            assert_eq!(ghosts.len(), 1);
            assert_eq!(ghosts[0].slot, "if_doubt");
            assert_eq!(ghosts[0].reason, "slot_missing");
        }
        other => panic!("expected Ghost, got {other:?}"),
    }
}

#[test]
fn missing_multiple_slots_ghosts() {
    let mut draft = complete_valid_draft();
    draft.when = None;
    draft.if_ok = None;
    draft.if_not = None;
    let ruling = check(draft);
    match ruling {
        PocketRuntimeRuling::Ghost { ghosts, partial: _ } => {
            let slots: Vec<&str> = ghosts.iter().map(|g| g.slot.as_str()).collect();
            assert!(slots.contains(&"when"));
            assert!(slots.contains(&"if_ok"));
            assert!(slots.contains(&"if_not"));
            assert_eq!(ghosts.len(), 3, "all three missing slots must be reported");
        }
        other => panic!("expected Ghost, got {other:?}"),
    }
}

#[test]
fn invalid_status_rejects() {
    // Status contains an embedded newline — a forbidden control char.
    let mut draft = complete_valid_draft();
    draft.status = Some(Value::String("candidate\ntampered".to_string()));
    let ruling = check(draft);
    match ruling {
        PocketRuntimeRuling::Reject { errors } => {
            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0].slot, "status");
            assert_eq!(errors[0].kind, "forbidden_char");
        }
        other => panic!("expected Reject, got {other:?}"),
    }
}

#[test]
fn slot_validator_failure_rejects() {
    // `did` is a number rather than a string — wrong JSON type.
    let mut draft = complete_valid_draft();
    draft.did = Some(json!(42));
    let ruling = check(draft);
    match ruling {
        PocketRuntimeRuling::Reject { errors } => {
            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0].slot, "did");
            assert_eq!(errors[0].kind, "wrong_type");
            assert!(errors[0].detail.contains("number"));
        }
        other => panic!("expected Reject, got {other:?}"),
    }
}

#[test]
fn aux_is_preserved_but_not_required() {
    // aux = None on a complete draft → still AcceptCandidate.
    let no_aux = check(complete_valid_draft());
    assert!(matches!(
        no_aux,
        PocketRuntimeRuling::AcceptCandidate { .. }
    ));

    // aux = Some(rich object) preserved on AcceptCandidate.
    let mut draft = complete_valid_draft();
    draft.aux = Some(json!({
        "duration_hours": 8,
        "subjective_state": "rested",
        "nested": { "context": "self_report", "tags": ["rest", "minilab"] }
    }));
    let ruling = check(draft);
    match ruling {
        PocketRuntimeRuling::AcceptCandidate {
            normalized,
            warnings: _,
        } => {
            let aux = normalized.aux.expect("aux must be preserved verbatim");
            assert_eq!(aux["duration_hours"], json!(8));
            assert_eq!(aux["nested"]["tags"], json!(["rest", "minilab"]));
        }
        other => panic!("expected AcceptCandidate with preserved aux, got {other:?}"),
    }

    // aux as a non-object (string) does NOT affect 9-slot validity.
    let mut draft = complete_valid_draft();
    draft.aux = Some(json!("free text aux"));
    let ruling = check(draft);
    assert!(matches!(
        ruling,
        PocketRuntimeRuling::AcceptCandidate { .. }
    ));

    // Conversely: aux must not rescue a missing 9-slot. With aux present
    // and `when` missing, the draft still Ghosts.
    let mut draft = complete_valid_draft();
    draft.when = None;
    draft.aux = Some(json!({"trying": "to rescue"}));
    let ruling = check(draft);
    assert!(matches!(ruling, PocketRuntimeRuling::Ghost { .. }));
}
