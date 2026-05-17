#![forbid(unsafe_code)]

use logline_who::slot::{
    Branch, Operation, RunContext, RuntimeDecision, RuntimeSlot, Slot, SlotHouse, SlotResolution,
};
use logline_who::LogLine;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceResolution {
    Confirmed,
    Missing,
    Contradicted,
    Expired,
    Insufficient,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    Token,
    Signature,
    Digest,
    Receipt,
    Quorum,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceRequirement {
    pub raw: String,
    pub kind: EvidenceKind,
    pub threshold: Option<usize>,
    pub label: Option<String>,
}

impl EvidenceRequirement {
    pub fn parse(value: &str) -> Self {
        let raw = value.trim().to_string();

        if raw.starts_with("receipt:") && raw.len() > "receipt:".len() {
            return Self {
                raw,
                kind: EvidenceKind::Receipt,
                threshold: None,
                label: None,
            };
        }

        if raw.starts_with("digest:") && raw.len() > "digest:".len() {
            return Self {
                raw,
                kind: EvidenceKind::Digest,
                threshold: None,
                label: None,
            };
        }

        if raw.starts_with("signature:") && raw.len() > "signature:".len() {
            return Self {
                raw,
                kind: EvidenceKind::Signature,
                threshold: None,
                label: None,
            };
        }

        if let Some((threshold, label)) = parse_quorum(&raw) {
            return Self {
                raw,
                kind: EvidenceKind::Quorum,
                threshold: Some(threshold),
                label: Some(label),
            };
        }

        Self {
            raw,
            kind: EvidenceKind::Token,
            threshold: None,
            label: None,
        }
    }

    pub fn is_missing_marker(&self) -> bool {
        matches!(
            self.raw.trim().to_ascii_lowercase().as_str(),
            "none" | "missing" | "unknown" | "unconfirmed" | "ghost" | "null" | ""
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceReceipt {
    pub id: String,
    pub confirms: String,
    pub issued_by: String,
    pub digest: Option<String>,
}

impl EvidenceReceipt {
    pub fn token(value: impl Into<String>) -> Self {
        let id = value.into().trim().to_string();
        Self {
            confirms: id.clone(),
            id,
            issued_by: "provided_evidence".to_string(),
            digest: None,
        }
    }

    pub fn parse_provided(value: &str) -> Option<Self> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return None;
        }

        Some(Self::token(trimmed))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceView {
    pub receipts: Vec<EvidenceReceipt>,
}

impl EvidenceView {
    pub fn from_tokens(tokens: &[String]) -> Self {
        let receipts = tokens
            .iter()
            .filter_map(|token| EvidenceReceipt::parse_provided(token))
            .collect();

        Self { receipts }
    }

    pub fn contains(&self, requirement: &EvidenceRequirement) -> bool {
        match requirement.kind {
            EvidenceKind::Quorum => match requirement.threshold {
                Some(threshold) => self.quorum_count(requirement) >= threshold,
                None => false,
            },
            _ => self
                .receipts
                .iter()
                .any(|receipt| receipt.confirms == requirement.raw),
        }
    }

    pub fn quorum_count(&self, requirement: &EvidenceRequirement) -> usize {
        let Some(label) = requirement.label.as_deref() else {
            return 0;
        };
        let prefix = format!("quorum:{label}:");
        let mut witnesses = BTreeSet::new();

        for receipt in &self.receipts {
            let Some(witness) = receipt.confirms.strip_prefix(&prefix) else {
                continue;
            };
            let witness = witness.trim();
            if !witness.is_empty() {
                witnesses.insert(witness.to_string());
            }
        }

        witnesses.len()
    }

    pub fn quorum_witnesses(&self) -> BTreeMap<String, Vec<String>> {
        let mut grouped: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

        for receipt in &self.receipts {
            let Some(rest) = receipt.confirms.strip_prefix("quorum:") else {
                continue;
            };
            let Some((label, witness)) = rest.split_once(':') else {
                continue;
            };
            let label = label.trim();
            let witness = witness.trim();
            if label.is_empty() || witness.is_empty() {
                continue;
            }
            grouped
                .entry(label.to_string())
                .or_default()
                .insert(witness.to_string());
        }

        grouped
            .into_iter()
            .map(|(label, witnesses)| (label, witnesses.into_iter().collect()))
            .collect()
    }
}

fn parse_quorum(raw: &str) -> Option<(usize, String)> {
    let rest = raw.strip_prefix("quorum:")?;
    let (threshold, label) = rest.split_once(':')?;
    let threshold = threshold.parse::<usize>().ok()?;
    let label = label.trim();

    if threshold == 0 || label.is_empty() {
        return None;
    }

    Some((threshold, label.to_string()))
}

pub struct ConfirmedBySlot;

impl SlotHouse for ConfirmedBySlot {
    fn slot(&self) -> Slot {
        Slot::ConfirmedBy
    }

    fn resolve(
        &self,
        input: &LogLine,
        _ctx: &RunContext<'_>,
        decision: &RuntimeDecision,
    ) -> RuntimeSlot {
        let (resolution, reason) = match decision.selected.branch {
            Branch::Ok => (SlotResolution::Ok, decision.selected.reason.as_str()),
            Branch::Doubt => (SlotResolution::Doubt, decision.selected.reason.as_str()),
            Branch::Not => (SlotResolution::Not, decision.selected.reason.as_str()),
        };

        RuntimeSlot {
            value: input.confirmed_by.clone(),
            resolution,
            reason: reason.to_string(),
            next: Some(decision.selected.position.clone()),
            route: None,
            before: None,
            after: None,
        }
    }

    fn accepts_operation(&self, op: Operation) -> bool {
        matches!(
            op,
            Operation::Questionar | Operation::Confirmar | Operation::Verificar
        )
    }
}

pub fn pivot(input: &LogLine, ctx: &RunContext<'_>) -> (Branch, &'static str) {
    let requirement = EvidenceRequirement::parse(&input.confirmed_by);
    let required = requirement.raw.trim();

    if requirement.is_missing_marker() {
        if required.eq_ignore_ascii_case("none") && ctx.allow_no_confirmation {
            return (Branch::Ok, "canon_allows_no_confirmation");
        }

        return match ctx.operation {
            Operation::Questionar => (Branch::Doubt, "question_pending"),
            _ => (Branch::Doubt, "receipt_missing"),
        };
    }

    if has_required_evidence(ctx, &requirement) {
        return match ctx.operation {
            Operation::Questionar => (Branch::Doubt, "question_still_open"),
            Operation::Verificar => (Branch::Ok, "receipt_verified"),
            _ => (Branch::Ok, "receipt_found"),
        };
    }

    match ctx.operation {
        Operation::Questionar => (Branch::Doubt, "question_requires_answer"),
        Operation::Verificar => (Branch::Doubt, "receipt_not_found_for_verification"),
        _ => (Branch::Doubt, "receipt_missing"),
    }
}

pub fn is_missing(input: &LogLine) -> bool {
    EvidenceRequirement::parse(&input.confirmed_by).is_missing_marker()
}

fn has_required_evidence(ctx: &RunContext<'_>, requirement: &EvidenceRequirement) -> bool {
    EvidenceView::from_tokens(&ctx.provided_evidence).contains(requirement)
}
