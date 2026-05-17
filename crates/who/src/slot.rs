use crate::canon::Canon;
use crate::logline::LogLine;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Slot {
    Who,
    Did,
    This,
    When,
    ConfirmedBy,
    IfOk,
    IfDoubt,
    IfNot,
    Status,
}

impl Slot {
    pub fn as_str(self) -> &'static str {
        match self {
            Slot::Who => "who",
            Slot::Did => "did",
            Slot::This => "this",
            Slot::When => "when",
            Slot::ConfirmedBy => "confirmed_by",
            Slot::IfOk => "if_ok",
            Slot::IfDoubt => "if_doubt",
            Slot::IfNot => "if_not",
            Slot::Status => "status",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Operation {
    Identificar,
    Nomear,
    Localizar,
    Simular,
    Questionar,
    Confirmar,
    Verificar,
    Acionar,
    Despachar,
    MarcarProblema,
    Registrar,
}

impl Operation {
    pub fn home(self) -> Slot {
        match self {
            Operation::Identificar => Slot::Who,
            Operation::Nomear => Slot::Did,
            Operation::Localizar => Slot::This,
            Operation::Simular => Slot::When,
            Operation::Questionar | Operation::Confirmar | Operation::Verificar => {
                Slot::ConfirmedBy
            }
            Operation::Acionar => Slot::IfOk,
            Operation::Despachar => Slot::IfDoubt,
            Operation::MarcarProblema => Slot::IfNot,
            Operation::Registrar => Slot::Status,
        }
    }

    pub fn phase(self) -> Phase {
        match self {
            Operation::Identificar
            | Operation::Nomear
            | Operation::Localizar
            | Operation::Simular => Phase::Input,
            Operation::Questionar | Operation::Confirmar | Operation::Verificar => Phase::Pivot,
            Operation::Acionar | Operation::Despachar | Operation::MarcarProblema => Phase::Output,
            Operation::Registrar => Phase::Closure,
        }
    }
}

impl FromStr for Operation {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.trim().to_ascii_lowercase().as_str() {
            "identificar" | "assinar" | "identify" | "sign" => Ok(Operation::Identificar),
            "nomear" | "name" => Ok(Operation::Nomear),
            "localizar" | "locate" => Ok(Operation::Localizar),
            "simular" | "simulate" => Ok(Operation::Simular),
            "questionar" | "ask" | "question" => Ok(Operation::Questionar),
            "confirmar" | "confirm" => Ok(Operation::Confirmar),
            "verificar" | "verify" => Ok(Operation::Verificar),
            "acionar" | "activate" | "trigger" => Ok(Operation::Acionar),
            "despachar" | "dispatch" => Ok(Operation::Despachar),
            "marcar_problema" | "marcar-problema" | "mark_problem" | "mark-problem" => {
                Ok(Operation::MarcarProblema)
            }
            "registrar" | "register" | "record" => Ok(Operation::Registrar),
            _ => Err(format!("unknown operation: {input}")),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Input,
    Pivot,
    Output,
    Closure,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Pocket,
    Grammar,
    Algebra,
    Operational,
    Closure,
}

impl FromStr for Mode {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.trim().to_ascii_lowercase().as_str() {
            "pocket" => Ok(Self::Pocket),
            "grammar" | "strong_grammar" | "strong-grammar" => Ok(Self::Grammar),
            "algebra" | "movement" | "movements" => Ok(Self::Algebra),
            "operational" | "operation" => Ok(Self::Operational),
            "closure" | "status" => Ok(Self::Closure),
            _ => Err(format!("unknown mode: {input}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Branch {
    Ok,
    Doubt,
    Not,
}

impl Branch {
    pub fn position(&self) -> &'static str {
        match self {
            Branch::Ok => "if_ok",
            Branch::Doubt => "if_doubt",
            Branch::Not => "if_not",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SlotResolution {
    Ok,
    Doubt,
    Not,
    Selected,
    NotSelected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeSlot {
    pub value: String,
    pub resolution: SlotResolution,
    pub reason: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub route: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SelectedBranch {
    pub branch: Branch,
    pub position: String,
    pub route: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StatusTransition {
    pub before: String,
    pub after: String,
}

#[derive(Debug, Clone)]
pub struct RunContext<'a> {
    pub canon: &'a Canon,
    pub mode: Mode,
    pub operation: Operation,
    pub provided_evidence: Vec<String>,
    pub allow_no_confirmation: bool,
}

impl<'a> RunContext<'a> {
    pub fn new(canon: &'a Canon, operation: Operation) -> Self {
        Self {
            canon,
            mode: Mode::Algebra,
            operation,
            provided_evidence: Vec::new(),
            allow_no_confirmation: false,
        }
    }

    pub fn has_evidence(&self, token: &str) -> bool {
        self.provided_evidence
            .iter()
            .any(|evidence| evidence == token)
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeDecision {
    pub selected: SelectedBranch,
    pub status: StatusTransition,
}

pub trait SlotHouse {
    fn slot(&self) -> Slot;

    fn resolve(
        &self,
        input: &LogLine,
        ctx: &RunContext<'_>,
        decision: &RuntimeDecision,
    ) -> RuntimeSlot;

    fn accepts_operation(&self, op: Operation) -> bool {
        self.slot() == op.home()
    }
}

pub fn ok(value: &str, reason: &str, next: &str) -> RuntimeSlot {
    RuntimeSlot {
        value: value.to_string(),
        resolution: SlotResolution::Ok,
        reason: reason.to_string(),
        next: Some(next.to_string()),
        route: None,
        before: None,
        after: None,
    }
}

pub fn ok_for_house(
    value: &str,
    default_reason: &str,
    strong_reason: &str,
    next: &str,
    house: &dyn SlotHouse,
    ctx: &RunContext<'_>,
) -> RuntimeSlot {
    let reason = if house.accepts_operation(ctx.operation) {
        strong_reason
    } else {
        default_reason
    };

    ok(value, reason, next)
}

pub fn branch(position: &str, value: &str, selected: &SelectedBranch) -> RuntimeSlot {
    let selected_here = selected.position == position;

    RuntimeSlot {
        value: value.to_string(),
        resolution: if selected_here {
            SlotResolution::Selected
        } else {
            SlotResolution::NotSelected
        },
        reason: if selected_here {
            selected.reason.clone()
        } else {
            "branch_not_chosen".to_string()
        },
        next: if selected_here {
            Some("status".to_string())
        } else {
            None
        },
        route: if selected_here {
            Some(selected.route.clone())
        } else {
            None
        },
        before: None,
        after: None,
    }
}
