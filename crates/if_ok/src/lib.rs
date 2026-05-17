#![forbid(unsafe_code)]

use logline_who::canon::{Canon, RouteClass};
use logline_who::slot::{
    self, Operation, RunContext, RuntimeDecision, RuntimeSlot, Slot, SlotHouse, SlotResolution,
};
use logline_who::{LogLine, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseKind {
    Execute,
    Commit,
    EmitLogLine,
    ReleaseToTower,
    EnterStrongGrammar,
    Domain(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReleaseRoute {
    pub raw: String,
    pub kind: ReleaseKind,
}

impl ReleaseRoute {
    pub fn parse(value: &str) -> Self {
        let raw = value.trim().to_string();
        let kind = match raw.as_str() {
            "execute" => ReleaseKind::Execute,
            "commit" | "commit_status" => ReleaseKind::Commit,
            "emit_logline" | "emit_candidate_logline" | "emit_normalized_logline" => {
                ReleaseKind::EmitLogLine
            }
            "release_to_tower" | "release_admissible_logline" => ReleaseKind::ReleaseToTower,
            "enter_strong_grammar" => ReleaseKind::EnterStrongGrammar,
            _ => ReleaseKind::Domain(raw.clone()),
        };

        Self { raw, kind }
    }

    pub fn selected_reason(&self) -> &'static str {
        match self.kind {
            ReleaseKind::Execute => "selected_release_execute",
            ReleaseKind::Commit => "selected_release_commit",
            ReleaseKind::EmitLogLine => "selected_release_emit_logline",
            ReleaseKind::ReleaseToTower => "selected_release_to_tower",
            ReleaseKind::EnterStrongGrammar => "selected_release_enter_strong_grammar",
            ReleaseKind::Domain(_) => "selected_release_domain",
        }
    }
}

pub trait ReleaseAdapter {
    type Receipt;
    type Error;

    fn release(&self, route: &ReleaseRoute) -> std::result::Result<Self::Receipt, Self::Error>;
}

pub struct IfOkSlot;

impl SlotHouse for IfOkSlot {
    fn slot(&self) -> Slot {
        Slot::IfOk
    }

    fn resolve(
        &self,
        input: &LogLine,
        _ctx: &RunContext<'_>,
        decision: &RuntimeDecision,
    ) -> RuntimeSlot {
        let route = ReleaseRoute::parse(&input.if_ok);
        let selected_here = decision.selected.position == "if_ok";

        if selected_here {
            return RuntimeSlot {
                value: input.if_ok.clone(),
                resolution: SlotResolution::Selected,
                reason: route.selected_reason().to_string(),
                next: Some("status".to_string()),
                route: Some(decision.selected.route.clone()),
                before: None,
                after: None,
            };
        }

        slot::branch("if_ok", &input.if_ok, &decision.selected)
    }

    fn accepts_operation(&self, op: Operation) -> bool {
        matches!(op, Operation::Acionar)
    }
}

pub fn route_class(canon: &Canon, input: &LogLine) -> Result<RouteClass> {
    canon.classify_route_field("if_ok", &input.if_ok)
}
