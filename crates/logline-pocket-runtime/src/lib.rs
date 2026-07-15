//! LogLine Pocket Runtime — small guard beside the LLM Translator.
//!
//! The Pocket Runtime sits **before** canon. Its single talent is to reduce
//! the entropy of a translation draft enough that downstream canon emission
//! does not have to invent shape. It does **not** emit canonical receipts,
//! it does **not** admit runtime actions, it does **not** execute, and it
//! does **not** call any LLM.
//!
//! See `LogLine-Foundation/governance/lips/LIP-0008-llm-tier-discipline-and-dossier-discipline.md`
//! for the doctrine that motivates this crate.
//!
//! # Three outcomes
//!
//! - [`PocketRuntimeRuling::AcceptCandidate`] — all nine slots are present,
//!   non-empty strings, free of obviously hostile content, and pass the
//!   shared `validate_shape` from `logline-who`. The normalized draft is
//!   returned for the next stage (canon emission).
//!
//! - [`PocketRuntimeRuling::Ghost`] — one or more required slots are
//!   missing or empty. The partial draft is preserved so the caller (UI,
//!   Translator session loop, etc.) can ask for clarification without
//!   losing what was provided.
//!
//! - [`PocketRuntimeRuling::Reject`] — slots are present but malformed
//!   (wrong JSON type, forbidden characters, oversized, or rejected by
//!   the shared `validate_shape`). The Reject is structured per-slot so
//!   the caller can render a precise correction request.
//!
//! `aux` is preserved across all three outcomes and never affects the
//! nine-slot decision.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use logline_who::{validate_shape, LogLine};

/// Pocket Runtime soft length cap for a single slot value. Slots over this
/// limit are rejected; canon emission would also choke on them, and the
/// Pocket Runtime exists precisely to refuse such drafts cheaply.
pub const MAX_SLOT_LEN: usize = 4096;

/// Names of the nine canonical slots, in canonical order.
pub const SLOTS: [&str; 9] = [
    "who",
    "did",
    "this",
    "when",
    "confirmed_by",
    "if_ok",
    "if_doubt",
    "if_not",
    "status",
];

/// A candidate LogLine draft as the LLM Translator might emit it. Each
/// slot is `Option<Value>` because partial drafts are first-class; the
/// Pocket Runtime decides whether the draft can move on, must ghost, or
/// must be rejected.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct LogLineDraft {
    pub who: Option<Value>,
    pub did: Option<Value>,
    pub this: Option<Value>,
    pub when: Option<Value>,
    pub confirmed_by: Option<Value>,
    pub if_ok: Option<Value>,
    pub if_doubt: Option<Value>,
    pub if_not: Option<Value>,
    pub status: Option<Value>,
    /// Free-form auxiliary fields the Translator may have inferred. Never
    /// affects the nine-slot decision; preserved verbatim across all
    /// outcomes.
    pub aux: Option<Value>,
}

/// Non-fatal observation about a slot value that was accepted after a
/// normalizing step (e.g. surrounding whitespace trimmed away).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PocketWarning {
    pub slot: String,
    pub kind: String,
    pub detail: String,
}

/// A slot whose absence prevented acceptance. The caller may treat the
/// set of ghosts as a clarification request to the human or the
/// Translator.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PocketGhost {
    pub slot: String,
    pub reason: String,
}

/// A slot whose presence violates a structural rule (wrong type,
/// forbidden character, oversized, shape rejected by `validate_shape`).
/// Carrying the slot name and the offence keeps the Reject directly
/// actionable.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PocketError {
    pub slot: String,
    pub kind: String,
    pub detail: String,
}

/// The Pocket Runtime's decision about a draft. Always one of three;
/// never a free-form string.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "ruling", rename_all = "snake_case")]
pub enum PocketRuntimeRuling {
    /// All nine slots are present and pass shape and forbidden-char
    /// checks. The Translator (or any caller) may proceed to canon
    /// emission with `normalized`.
    AcceptCandidate {
        normalized: LogLineDraft,
        warnings: Vec<PocketWarning>,
    },
    /// One or more required slots are missing or empty. The draft cannot
    /// proceed to canon, but the partial slots are returned so the
    /// caller can refine the request without losing context.
    Ghost {
        ghosts: Vec<PocketGhost>,
        partial: LogLineDraft,
    },
    /// One or more slots are present but malformed. The draft is
    /// rejected; the caller must produce a new draft. Errors enumerate
    /// every offending slot rather than failing on the first.
    Reject { errors: Vec<PocketError> },
}

/// Pocket Runtime entry point. Pure function; no I/O, no LLM call, no
/// canon emission, no admission decision.
pub fn check(draft: LogLineDraft) -> PocketRuntimeRuling {
    let mut ghosts: Vec<PocketGhost> = Vec::new();
    let mut errors: Vec<PocketError> = Vec::new();
    let warnings: Vec<PocketWarning> = Vec::new();

    // Per-slot pass. Collect every issue rather than fail on first.
    let raw: [(&'static str, &Option<Value>); 9] = [
        ("who", &draft.who),
        ("did", &draft.did),
        ("this", &draft.this),
        ("when", &draft.when),
        ("confirmed_by", &draft.confirmed_by),
        ("if_ok", &draft.if_ok),
        ("if_doubt", &draft.if_doubt),
        ("if_not", &draft.if_not),
        ("status", &draft.status),
    ];

    // Strings extracted slot-by-slot (only populated when the slot passed
    // the per-slot pass cleanly). Used to construct the canonical LogLine
    // for `validate_shape` once every slot is good.
    let mut strings: [String; 9] = Default::default();

    for (i, (slot, val)) in raw.iter().enumerate() {
        match val {
            None => {
                ghosts.push(PocketGhost {
                    slot: (*slot).to_string(),
                    reason: "slot_missing".to_string(),
                });
            }
            Some(Value::Null) => {
                ghosts.push(PocketGhost {
                    slot: (*slot).to_string(),
                    reason: "slot_null".to_string(),
                });
            }
            Some(Value::String(s)) => {
                if s.trim().is_empty() {
                    ghosts.push(PocketGhost {
                        slot: (*slot).to_string(),
                        reason: "slot_empty_string".to_string(),
                    });
                } else if has_forbidden_chars(s) {
                    errors.push(PocketError {
                        slot: (*slot).to_string(),
                        kind: "forbidden_char".to_string(),
                        detail: "slot value contains a null byte or unescaped control character"
                            .to_string(),
                    });
                } else if s.len() > MAX_SLOT_LEN {
                    errors.push(PocketError {
                        slot: (*slot).to_string(),
                        kind: "oversized".to_string(),
                        detail: format!(
                            "slot value exceeds {} chars (got {})",
                            MAX_SLOT_LEN,
                            s.len()
                        ),
                    });
                } else {
                    strings[i] = s.clone();
                }
            }
            Some(other) => {
                errors.push(PocketError {
                    slot: (*slot).to_string(),
                    kind: "wrong_type".to_string(),
                    detail: format!("slot must be a JSON string, got {}", json_type_name(other)),
                });
            }
        }
    }

    // Decision precedence: Reject > Ghost > AcceptCandidate.
    // Errors (malformed values) win over ghosts (missing values) because
    // a malformed slot signals an upstream bug, not just under-specified
    // intent.
    if !errors.is_empty() {
        return PocketRuntimeRuling::Reject { errors };
    }
    if !ghosts.is_empty() {
        return PocketRuntimeRuling::Ghost {
            ghosts,
            partial: draft,
        };
    }

    // All nine slots are non-empty strings free of forbidden content.
    // Construct the canonical LogLine and ask the shared validator to
    // double-check shape. `validate_shape` today re-checks non-empty,
    // but it is the authoritative shape gate; if it ever tightens, the
    // Pocket Runtime inherits the tightening for free.
    let logline = LogLine::new(
        &strings[0],
        &strings[1],
        &strings[2],
        &strings[3],
        &strings[4],
        &strings[5],
        &strings[6],
        &strings[7],
        &strings[8],
    );
    if let Err(e) = validate_shape(&logline) {
        return PocketRuntimeRuling::Reject {
            errors: vec![PocketError {
                slot: "(shape)".to_string(),
                kind: "shape_invalid".to_string(),
                detail: e.to_string(),
            }],
        };
    }

    PocketRuntimeRuling::AcceptCandidate {
        normalized: draft,
        warnings,
    }
}

/// Forbidden characters in a slot string value. The list is intentionally
/// short and conservative: control bytes that downstream canon, JCS
/// canonicalization, or transport encoders would have to either escape or
/// reject. The Pocket Runtime refuses them cheaply at the door.
fn has_forbidden_chars(s: &str) -> bool {
    s.chars()
        .any(|c| c == '\0' || c == '\n' || c == '\r' || c == '\t' || c.is_control())
}

fn json_type_name(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}
