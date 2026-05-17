#![forbid(unsafe_code)]

use logline_who::slot::{
    self, Operation, RunContext, RuntimeDecision, RuntimeSlot, Slot, SlotHouse, SlotResolution,
};
use logline_who::LogLine;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Verb {
    Operation(Operation),
    CanonLaw(String),
    Domain(String),
    Unknown(String),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VerbClass {
    RuntimeOperation,
    CanonicalLaw,
    DomainAct,
    Unknown,
}

impl VerbClass {
    pub fn reason(self) -> &'static str {
        match self {
            VerbClass::RuntimeOperation => "runtime_operation_verb",
            VerbClass::CanonicalLaw => "canonical_law_verb",
            VerbClass::DomainAct => "domain_act_verb",
            VerbClass::Unknown => "unknown_verb",
        }
    }

    pub fn strong_reason(self) -> &'static str {
        match self {
            VerbClass::RuntimeOperation => "nomear_runtime_operation",
            VerbClass::CanonicalLaw => "nomear_canonical_law_verb",
            VerbClass::DomainAct => "nomear_domain_act_verb",
            VerbClass::Unknown => "nomear_unknown_verb",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DidResolution {
    pub raw: String,
    pub verb: Verb,
    pub class: VerbClass,
}

impl DidResolution {
    pub fn parse(value: &str) -> Self {
        let raw = value.trim().to_string();

        if raw.is_empty() {
            return Self {
                raw,
                verb: Verb::Unknown(String::new()),
                class: VerbClass::Unknown,
            };
        }

        if let Ok(operation) = Operation::from_str(&raw) {
            return Self {
                raw,
                verb: Verb::Operation(operation),
                class: VerbClass::RuntimeOperation,
            };
        }

        if is_canonical_law_verb(&raw) {
            return Self {
                raw: raw.clone(),
                verb: Verb::CanonLaw(raw),
                class: VerbClass::CanonicalLaw,
            };
        }

        Self {
            raw: raw.clone(),
            verb: Verb::Domain(raw),
            class: VerbClass::DomainAct,
        }
    }
}

fn is_canonical_law_verb(value: &str) -> bool {
    matches!(
        value,
        "define"
            | "require"
            | "walk"
            | "hold"
            | "compile"
            | "model"
            | "operate"
            | "close"
            | "execute"
            | "become"
            | "bypass"
            | "release"
            | "express_or_author"
            | "translate"
    )
}

pub struct DidSlot;

impl SlotHouse for DidSlot {
    fn slot(&self) -> Slot {
        Slot::Did
    }

    fn resolve(
        &self,
        input: &LogLine,
        ctx: &RunContext<'_>,
        _decision: &RuntimeDecision,
    ) -> RuntimeSlot {
        let resolution = DidResolution::parse(&input.did);

        if resolution.class == VerbClass::Unknown {
            return RuntimeSlot {
                value: input.did.clone(),
                resolution: SlotResolution::Doubt,
                reason: "unknown_verb".to_string(),
                next: Some("this".to_string()),
                route: None,
                before: None,
                after: None,
            };
        }

        slot::ok_for_house(
            &input.did,
            resolution.class.reason(),
            resolution.class.strong_reason(),
            "this",
            self,
            ctx,
        )
    }
}
