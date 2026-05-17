#![forbid(unsafe_code)]

use logline_who::slot::{
    self, Branch, Operation, RunContext, RuntimeDecision, RuntimeSlot, SelectedBranch, Slot,
    SlotHouse, SlotResolution,
};
use logline_who::LogLine;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DoubtKind {
    Ask,
    Clarify,
    Suspend,
    Ghost,
    ManualReview,
    Dispatch,
    Simulate,
    EmitSimulationReceipt,
    EmitDoubtTrace,
    ProposePossibleWorld,
    AskForEvidence,
    Domain(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DoubtRoute {
    pub raw: String,
    pub kind: DoubtKind,
}

impl DoubtRoute {
    pub fn parse(value: &str) -> Self {
        let raw = value.trim().to_string();
        let normalized = raw.to_ascii_lowercase();
        let kind = match normalized.as_str() {
            "ask" => DoubtKind::Ask,
            "clarify" | "ask_or_suspend" => DoubtKind::Clarify,
            "suspend" => DoubtKind::Suspend,
            "ghost" | "ghost_and_clarify" => DoubtKind::Ghost,
            "manual_review" | "manual-review" => DoubtKind::ManualReview,
            "dispatch" | "despachar" | "return_to_logline" => DoubtKind::Dispatch,
            "simulate" => DoubtKind::Simulate,
            "emit_simulation_receipt" | "emit-simulation-receipt" => {
                DoubtKind::EmitSimulationReceipt
            }
            "emit_doubt_trace" | "emit-doubt-trace" => DoubtKind::EmitDoubtTrace,
            "propose_possible_world" | "propose-possible-world" => DoubtKind::ProposePossibleWorld,
            "ask_for_evidence" | "ask-for-evidence" => DoubtKind::AskForEvidence,
            _ => DoubtKind::Domain(raw.clone()),
        };

        Self { raw, kind }
    }

    pub fn is_clarifying(&self) -> bool {
        matches!(
            self.kind,
            DoubtKind::Ask | DoubtKind::Clarify | DoubtKind::Ghost | DoubtKind::AskForEvidence
        )
    }

    pub fn is_suspending(&self) -> bool {
        matches!(self.kind, DoubtKind::Suspend) || self.raw.eq_ignore_ascii_case("ask_or_suspend")
    }

    pub fn is_simulating(&self) -> bool {
        matches!(
            self.kind,
            DoubtKind::Simulate
                | DoubtKind::EmitSimulationReceipt
                | DoubtKind::ProposePossibleWorld
        )
    }

    pub fn is_trace_emitting(&self) -> bool {
        matches!(
            self.kind,
            DoubtKind::EmitDoubtTrace
                | DoubtKind::Simulate
                | DoubtKind::EmitSimulationReceipt
                | DoubtKind::ProposePossibleWorld
                | DoubtKind::AskForEvidence
        )
    }

    pub fn is_releasing(&self) -> bool {
        false
    }

    pub fn selected_reason(&self) -> String {
        match &self.kind {
            DoubtKind::Ask => "question_required".to_string(),
            DoubtKind::Clarify => "clarification_required".to_string(),
            DoubtKind::Suspend => "suspended_under_doubt".to_string(),
            DoubtKind::Ghost => "ghost_present".to_string(),
            DoubtKind::ManualReview => "manual_review_required".to_string(),
            DoubtKind::Dispatch => "dispatch_under_doubt".to_string(),
            DoubtKind::Simulate => "simulation_requested".to_string(),
            DoubtKind::EmitSimulationReceipt => "simulation_receipt_requested".to_string(),
            DoubtKind::EmitDoubtTrace => "doubt_trace_requested".to_string(),
            DoubtKind::ProposePossibleWorld => "possible_world_requested".to_string(),
            DoubtKind::AskForEvidence => "evidence_missing".to_string(),
            DoubtKind::Domain(value) => format!("domain_doubt_route:{value}"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DoubtTrace {
    pub branch: Branch,
    pub reason: String,
    pub simulated_route: String,
    pub released: bool,
}

impl DoubtTrace {
    pub fn from_selected_branch(selected: &SelectedBranch) -> Option<Self> {
        if selected.branch != Branch::Doubt {
            return None;
        }

        Some(Self {
            branch: Branch::Doubt,
            reason: selected.reason.clone(),
            simulated_route: selected.route.clone(),
            released: false,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SimulationReceipt {
    pub receipt_kind: String,
    pub logline_digest: String,
    pub branch: Branch,
    pub reason: String,
    pub simulated_route: String,
    pub executed: bool,
    pub released: bool,
}

impl SimulationReceipt {
    pub fn from_selected_branch(
        selected: &SelectedBranch,
        logline_digest: impl Into<String>,
    ) -> Option<Self> {
        if selected.branch != Branch::Doubt {
            return None;
        }

        let route = DoubtRoute::parse(&selected.route);
        if !route.is_simulating() {
            return None;
        }

        Some(Self {
            receipt_kind: "simulation".to_string(),
            logline_digest: logline_digest.into(),
            branch: Branch::Doubt,
            reason: selected.reason.clone(),
            simulated_route: route.raw,
            executed: false,
            released: false,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClarificationRequest {
    pub route: DoubtRoute,
    pub question: String,
    pub missing_slots: Vec<Slot>,
}

impl ClarificationRequest {
    pub fn new(route: DoubtRoute, question: impl Into<String>, missing_slots: Vec<Slot>) -> Self {
        Self {
            route,
            question: question.into(),
            missing_slots,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SuspensionRecord {
    pub route: DoubtRoute,
    pub reason: String,
}

impl SuspensionRecord {
    pub fn new(route: DoubtRoute, reason: impl Into<String>) -> Self {
        Self {
            route,
            reason: reason.into(),
        }
    }
}

pub struct IfDoubtSlot;

impl SlotHouse for IfDoubtSlot {
    fn slot(&self) -> Slot {
        Slot::IfDoubt
    }

    fn resolve(
        &self,
        input: &LogLine,
        _ctx: &RunContext<'_>,
        decision: &RuntimeDecision,
    ) -> RuntimeSlot {
        let route = DoubtRoute::parse(&input.if_doubt);
        let selected_here = decision.selected.position == "if_doubt";

        if selected_here {
            return RuntimeSlot {
                value: input.if_doubt.clone(),
                resolution: SlotResolution::Selected,
                reason: route.selected_reason(),
                next: Some("status".to_string()),
                route: Some(decision.selected.route.clone()),
                before: None,
                after: None,
            };
        }

        slot::branch("if_doubt", &input.if_doubt, &decision.selected)
    }

    fn accepts_operation(&self, op: Operation) -> bool {
        matches!(op, Operation::Despachar)
    }
}
