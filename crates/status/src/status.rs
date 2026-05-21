use crate::receipt::canonical_json;
use logline_who::slot::{
    Branch, Operation, RunContext, RuntimeDecision, RuntimeSlot, Slot, SlotHouse, SlotResolution,
    StatusTransition,
};
use logline_who::{Canon, LogLine, Result};
use sha2::{Digest, Sha256};
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lifecycle {
    Draft,
    Pending,
    Doubt,
    Confirmed,
    Released,
    Failed,
    RolledBack,
    Rejected,
    Void,
    Closed,
    Domain(String),
}

impl Lifecycle {
    pub fn from_status(value: &str) -> Self {
        match value.trim() {
            "draft" => Self::Draft,
            "pending" => Self::Pending,
            "doubt" => Self::Doubt,
            "confirmed" => Self::Confirmed,
            "released" => Self::Released,
            "failed" => Self::Failed,
            "rolled_back" => Self::RolledBack,
            "rejected" => Self::Rejected,
            "void" => Self::Void,
            "closed" => Self::Closed,
            other => Self::Domain(other.to_string()),
        }
    }

    pub fn as_canonical_str(&self) -> Option<&'static str> {
        match self {
            Self::Draft => Some("draft"),
            Self::Pending => Some("pending"),
            Self::Doubt => Some("doubt"),
            Self::Confirmed => Some("confirmed"),
            Self::Released => Some("released"),
            Self::Failed => Some("failed"),
            Self::RolledBack => Some("rolled_back"),
            Self::Rejected => Some("rejected"),
            Self::Void => Some("void"),
            Self::Closed => Some("closed"),
            Self::Domain(_) => None,
        }
    }

    pub fn canonical_label(&self) -> Cow<'_, str> {
        match self.as_canonical_str() {
            Some(value) => Cow::Borrowed(value),
            None => match self {
                Self::Domain(value) => Cow::Borrowed(value.as_str()),
                _ => Cow::Borrowed(""),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LifecycleTransition {
    pub before: Lifecycle,
    pub after: Lifecycle,
}

impl LifecycleTransition {
    pub fn from_status_transition(transition: &StatusTransition) -> Self {
        Self {
            before: Lifecycle::from_status(&transition.before),
            after: Lifecycle::from_status(&transition.after),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerEntry {
    pub logline_digest: String,
    pub selected_branch: Branch,
    pub transition: LifecycleTransition,
    pub reason: String,
}

pub trait StatusLedger {
    fn append(&self, entry: &LedgerEntry) -> Result<()>;
}

pub struct CanonicalTuple<'a> {
    pub who: &'a str,
    pub did: &'a str,
    pub this_: &'a str,
    pub when: &'a str,
    pub confirmed_by: &'a str,
    pub if_ok: &'a str,
    pub if_doubt: &'a str,
    pub if_not: &'a str,
    pub status: &'a str,
}

impl<'a> CanonicalTuple<'a> {
    pub fn from_logline(logline: &'a LogLine) -> Self {
        Self {
            who: &logline.who,
            did: &logline.did,
            this_: &logline.this_,
            when: &logline.when,
            confirmed_by: &logline.confirmed_by,
            if_ok: &logline.if_ok,
            if_doubt: &logline.if_doubt,
            if_not: &logline.if_not,
            status: &logline.status,
        }
    }

}

pub fn canonical_tuple_digest(logline: &LogLine) -> String {
    let tuple = CanonicalTuple::from_logline(logline);
    let json_obj = serde_json::json!({
        "who": tuple.who,
        "did": tuple.did,
        "this": tuple.this_,
        "when": tuple.when,
        "confirmed_by": tuple.confirmed_by,
        "if_ok": tuple.if_ok,
        "if_doubt": tuple.if_doubt,
        "if_not": tuple.if_not,
        "status": tuple.status,
    });

    // Safety: json! macro output is always serializable
    let canonical = match canonical_json(&json_obj) {
        Ok(c) => c,
        Err(_) => return String::new(),
    };
    let digest = Sha256::digest(canonical.as_bytes());
    hex_lower(&digest)
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);

    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }

    output
}

pub struct StatusSlot;

impl SlotHouse for StatusSlot {
    fn slot(&self) -> Slot {
        Slot::Status
    }

    fn resolve(
        &self,
        _input: &LogLine,
        ctx: &RunContext<'_>,
        decision: &RuntimeDecision,
    ) -> RuntimeSlot {
        let reason = if self.accepts_operation(ctx.operation) {
            "registrar_lifecycle_transition"
        } else {
            "lifecycle_transition"
        };

        RuntimeSlot {
            value: decision.status.before.clone(),
            resolution: SlotResolution::Ok,
            reason: reason.to_string(),
            next: None,
            route: None,
            before: Some(decision.status.before.clone()),
            after: Some(decision.status.after.clone()),
        }
    }

    fn accepts_operation(&self, op: Operation) -> bool {
        matches!(op, Operation::Registrar)
    }
}

pub fn transition(canon: &Canon, current: &str, branch: &Branch) -> StatusTransition {
    StatusTransition {
        before: current.to_string(),
        after: next_status(canon, current, branch),
    }
}

pub fn lifecycle_transition(canon: &Canon, current: &str, branch: &Branch) -> LifecycleTransition {
    LifecycleTransition::from_status_transition(&transition(canon, current, branch))
}

fn next_status(canon: &Canon, current: &str, branch: &Branch) -> String {
    let target = match branch {
        Branch::Ok => {
            if matches!(current, "draft" | "pending" | "doubt") {
                "confirmed"
            } else {
                "released"
            }
        }
        Branch::Doubt => "doubt",
        Branch::Not => "rejected",
    };

    let Some(allowed) = canon.status_lifecycle.get(current) else {
        return target.to_string();
    };

    if allowed.iter().any(|s| s == target) {
        return target.to_string();
    }

    if matches!(branch, Branch::Ok) && allowed.iter().any(|s| s == "released") {
        return "released".to_string();
    }

    if matches!(branch, Branch::Not) && allowed.iter().any(|s| s == "void") {
        return "void".to_string();
    }

    allowed
        .first()
        .cloned()
        .unwrap_or_else(|| current.to_string())
}
