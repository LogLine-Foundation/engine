use crate::logline::LogLine;
use crate::slot::{
    self, RunContext, RuntimeDecision, RuntimeSlot, Slot, SlotHouse, SlotResolution,
};
use crate::{LogLineError, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct ActorId(String);

impl ActorId {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        Ok(Self(clean_identity("actor", value.into())?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for ActorId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for ActorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ActorId {
    type Err = LogLineError;

    fn from_str(value: &str) -> Result<Self> {
        Self::new(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct AuthorityId(String);

impl AuthorityId {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        Ok(Self(clean_identity("authority", value.into())?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for AuthorityId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for AuthorityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for AuthorityId {
    type Err = LogLineError;

    fn from_str(value: &str) -> Result<Self> {
        Self::new(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct Jurisdiction(String);

impl Jurisdiction {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        Ok(Self(clean_identity("jurisdiction", value.into())?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for Jurisdiction {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for Jurisdiction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Jurisdiction {
    type Err = LogLineError;

    fn from_str(value: &str) -> Result<Self> {
        Self::new(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct Principal(String);

impl Principal {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        Ok(Self(clean_identity("principal", value.into())?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for Principal {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for Principal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Principal {
    type Err = LogLineError;

    fn from_str(value: &str) -> Result<Self> {
        Self::new(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct WitnessKey(String);

impl WitnessKey {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        Ok(Self(clean_identity("witness_key", value.into())?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for WitnessKey {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for WitnessKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for WitnessKey {
    type Err = LogLineError;

    fn from_str(value: &str) -> Result<Self> {
        Self::new(value)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OriginKind {
    Human,
    System,
    Runtime,
    Llm,
    Unknown,
}

impl OriginKind {
    pub fn reason(self) -> &'static str {
        match self {
            OriginKind::Human => "accountable_origin_human",
            OriginKind::System => "accountable_origin_system",
            OriginKind::Runtime => "accountable_origin_runtime",
            OriginKind::Llm => "accountable_origin_llm",
            OriginKind::Unknown => "accountable_origin_unknown_kind",
        }
    }

    pub fn strong_reason(self) -> &'static str {
        match self {
            OriginKind::Human => "identificar_accountable_origin_human",
            OriginKind::System => "identificar_accountable_origin_system",
            OriginKind::Runtime => "identificar_accountable_origin_runtime",
            OriginKind::Llm => "identificar_accountable_origin_llm",
            OriginKind::Unknown => "identificar_accountable_origin_unknown_kind",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccountableOrigin {
    pub actor: ActorId,
    pub kind: OriginKind,
    pub authority: Option<AuthorityId>,
    pub jurisdiction: Option<Jurisdiction>,
}

impl AccountableOrigin {
    pub fn parse(value: &str) -> Result<Self> {
        value.parse()
    }
}

impl FromStr for AccountableOrigin {
    type Err = LogLineError;

    fn from_str(value: &str) -> Result<Self> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(LogLineError::EmptyIdentity { kind: "who" });
        }

        let Some((prefix, actor)) = trimmed.split_once(':') else {
            return Ok(Self {
                actor: ActorId::new(trimmed)?,
                kind: OriginKind::Unknown,
                authority: None,
                jurisdiction: None,
            });
        };

        let kind = match prefix.trim().to_ascii_lowercase().as_str() {
            "human" | "actor" | "user" => OriginKind::Human,
            "system" => OriginKind::System,
            "runtime" => OriginKind::Runtime,
            "llm" => OriginKind::Llm,
            _ => return Err(LogLineError::InvalidWhoOrigin(prefix.trim().to_string())),
        };

        Ok(Self {
            actor: ActorId::new(actor)?,
            kind,
            authority: None,
            jurisdiction: None,
        })
    }
}

fn clean_identity(kind: &'static str, value: String) -> Result<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(LogLineError::EmptyIdentity { kind });
    }

    Ok(trimmed.to_string())
}

pub struct WhoSlot;

impl SlotHouse for WhoSlot {
    fn slot(&self) -> Slot {
        Slot::Who
    }

    fn resolve(
        &self,
        input: &LogLine,
        ctx: &RunContext<'_>,
        _decision: &RuntimeDecision,
    ) -> RuntimeSlot {
        let origin = match AccountableOrigin::parse(&input.who) {
            Ok(origin) => origin,
            Err(_) => {
                return RuntimeSlot {
                    value: input.who.clone(),
                    resolution: SlotResolution::Doubt,
                    reason: "accountable_origin_invalid".to_string(),
                    next: Some("did".to_string()),
                    route: None,
                    before: None,
                    after: None,
                };
            }
        };

        slot::ok_for_house(
            &input.who,
            origin.kind.reason(),
            origin.kind.strong_reason(),
            "did",
            self,
            ctx,
        )
    }
}
