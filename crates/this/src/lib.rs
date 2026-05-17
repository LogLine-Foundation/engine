#![forbid(unsafe_code)]

use logline_who::slot::{
    self, Mode, RunContext, RuntimeDecision, RuntimeSlot, Slot, SlotHouse, SlotResolution,
};
use logline_who::LogLine;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MatterRef {
    ObjectId(String),
    Url(Url),
    Path(String),
    Digest(String),
    JsonPayload(String),
    Wildcard,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct MatterDigest(String);

impl MatterDigest {
    pub fn sha256_bytes(bytes: &[u8]) -> Self {
        let digest = Sha256::digest(bytes);
        Self(format!("sha256:{digest:x}"))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for MatterDigest {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for MatterDigest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocatedMatter {
    pub raw: String,
    pub reference: MatterRef,
    pub digest: Option<MatterDigest>,
}

impl LocatedMatter {
    pub fn parse(value: &str) -> Self {
        let raw = value.trim().to_string();
        let reference = MatterRef::parse(&raw);
        let digest = match &reference {
            MatterRef::JsonPayload(payload) => Some(MatterDigest::sha256_bytes(payload.as_bytes())),
            _ => None,
        };

        Self {
            raw,
            reference,
            digest,
        }
    }

    pub fn reason(&self) -> &'static str {
        match self.reference {
            MatterRef::ObjectId(_) => "object_matter",
            MatterRef::Url(_) => "url_matter",
            MatterRef::Path(_) => "path_matter",
            MatterRef::Digest(_) => "digest_matter",
            MatterRef::JsonPayload(_) => "json_payload_matter",
            MatterRef::Wildcard => "wildcard_matter",
        }
    }

    pub fn strong_reason(&self) -> &'static str {
        match self.reference {
            MatterRef::ObjectId(_) => "localizar_object_matter",
            MatterRef::Url(_) => "localizar_url_matter",
            MatterRef::Path(_) => "localizar_path_matter",
            MatterRef::Digest(_) => "localizar_digest_matter",
            MatterRef::JsonPayload(_) => "localizar_json_payload_matter",
            MatterRef::Wildcard => "localizar_wildcard_matter",
        }
    }

    pub fn is_unbounded_for_mode(&self, mode: Mode) -> bool {
        self.raw.is_empty()
            || (matches!(self.reference, MatterRef::Wildcard)
                && matches!(mode, Mode::Operational | Mode::Closure))
    }
}

impl MatterRef {
    pub fn parse(raw: &str) -> Self {
        if raw == "*" {
            return Self::Wildcard;
        }

        if is_sha256_digest(raw) {
            return Self::Digest(raw.to_ascii_lowercase());
        }

        if let Ok(url) = Url::parse(raw) {
            if matches!(url.scheme(), "http" | "https") {
                return Self::Url(url);
            }
        }

        if is_json_payload(raw) {
            return Self::JsonPayload(raw.to_string());
        }

        if is_path_like(raw) {
            return Self::Path(raw.to_string());
        }

        Self::ObjectId(raw.to_string())
    }
}

fn is_sha256_digest(raw: &str) -> bool {
    let Some(hex) = raw.strip_prefix("sha256:") else {
        return false;
    };

    hex.len() == 64 && hex.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn is_json_payload(raw: &str) -> bool {
    let trimmed = raw.trim();
    (trimmed.starts_with('{') && trimmed.ends_with('}'))
        || (trimmed.starts_with('[') && trimmed.ends_with(']'))
}

fn is_path_like(raw: &str) -> bool {
    raw.starts_with("./")
        || raw.starts_with("../")
        || raw.starts_with('/')
        || raw.contains('/')
        || raw.contains('\\')
}

pub struct ThisSlot;

impl SlotHouse for ThisSlot {
    fn slot(&self) -> Slot {
        Slot::This
    }

    fn resolve(
        &self,
        input: &LogLine,
        ctx: &RunContext<'_>,
        _decision: &RuntimeDecision,
    ) -> RuntimeSlot {
        let matter = LocatedMatter::parse(&input.this_);

        if matter.is_unbounded_for_mode(ctx.mode) {
            return RuntimeSlot {
                value: input.this_.clone(),
                resolution: SlotResolution::Doubt,
                reason: "unbounded_matter".to_string(),
                next: Some("when".to_string()),
                route: None,
                before: None,
                after: None,
            };
        }

        slot::ok_for_house(
            &input.this_,
            matter.reason(),
            matter.strong_reason(),
            "when",
            self,
            ctx,
        )
    }
}
