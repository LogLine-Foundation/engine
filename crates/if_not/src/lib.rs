#![forbid(unsafe_code)]

use logline_who::slot::{
    self, Branch, Operation, RunContext, RuntimeDecision, RuntimeSlot, SelectedBranch, Slot,
    SlotHouse, SlotResolution,
};
use logline_who::LogLine;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RejectionKind {
    Reject,
    Deny,
    ProhibitedByCanon,
    ContradictedEvidence,
    InvalidShape,
    ForbiddenRoute,
    Void,
    MarkFailed,
    Rollback,
    Domain(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RejectionRoute {
    pub raw: String,
    pub kind: RejectionKind,
}

impl RejectionRoute {
    pub fn parse(value: &str) -> Self {
        let raw = value.trim().to_string();
        let normalized = raw.to_ascii_lowercase();
        let kind = match normalized.as_str() {
            "reject" => RejectionKind::Reject,
            "deny" => RejectionKind::Deny,
            "prohibited_by_canon" => RejectionKind::ProhibitedByCanon,
            "contradicted_evidence" => RejectionKind::ContradictedEvidence,
            "invalid_shape" | "reject_shape" => RejectionKind::InvalidShape,
            "forbid" | "do_not_execute" => RejectionKind::ForbiddenRoute,
            "void" => RejectionKind::Void,
            "mark_failed" => RejectionKind::MarkFailed,
            "rollback" | "rolled_back" => RejectionKind::Rollback,
            _ => RejectionKind::Domain(raw.clone()),
        };

        Self { raw, kind }
    }

    pub fn selected_reason(&self) -> &'static str {
        match self.kind {
            RejectionKind::Reject | RejectionKind::Deny | RejectionKind::ProhibitedByCanon => {
                "selected_problem_reject"
            }
            RejectionKind::ContradictedEvidence => "selected_problem_contradicted_evidence",
            RejectionKind::InvalidShape => "selected_problem_invalid_shape",
            RejectionKind::ForbiddenRoute => "selected_problem_forbid",
            RejectionKind::Void => "selected_problem_void",
            RejectionKind::MarkFailed => "selected_problem_mark_failed",
            RejectionKind::Rollback => "selected_problem_rollback",
            RejectionKind::Domain(_) => "selected_problem_domain",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProblemRecord {
    pub slot: Slot,
    pub kind: RejectionKind,
    pub reason: String,
    pub route: String,
}

impl ProblemRecord {
    pub fn from_selected_branch(selected: &SelectedBranch) -> Option<Self> {
        if selected.branch != Branch::Not {
            return None;
        }

        let route = RejectionRoute::parse(&selected.route);

        Some(Self {
            slot: Slot::IfNot,
            kind: route.kind,
            reason: selected.reason.clone(),
            route: route.raw,
        })
    }
}

pub struct IfNotSlot;

impl SlotHouse for IfNotSlot {
    fn slot(&self) -> Slot {
        Slot::IfNot
    }

    fn resolve(
        &self,
        input: &LogLine,
        _ctx: &RunContext<'_>,
        decision: &RuntimeDecision,
    ) -> RuntimeSlot {
        let route = RejectionRoute::parse(&input.if_not);
        let selected_here = decision.selected.position == "if_not";

        if selected_here {
            return RuntimeSlot {
                value: input.if_not.clone(),
                resolution: SlotResolution::Selected,
                reason: route.selected_reason().to_string(),
                next: Some("status".to_string()),
                route: Some(decision.selected.route.clone()),
                before: None,
                after: None,
            };
        }

        slot::branch("if_not", &input.if_not, &decision.selected)
    }

    fn accepts_operation(&self, op: Operation) -> bool {
        matches!(op, Operation::MarcarProblema)
    }
}
