#![forbid(unsafe_code)]

use logline_who::slot::{
    self, Mode, RunContext, RuntimeDecision, RuntimeSlot, Slot, SlotHouse, SlotResolution,
};
use logline_who::LogLine;
use serde::{Deserialize, Serialize};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TemporalBinding {
    Timestamp(String),
    Phase(String),
    Window { starts_at: String, ends_at: String },
    Relative(String),
    Wildcard,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TemporalResolution {
    pub raw: String,
    pub binding: TemporalBinding,
}

impl TemporalResolution {
    pub fn parse(value: &str) -> Self {
        let raw = value.trim().to_string();

        if raw == "*" || raw == "any_time" {
            return Self {
                raw,
                binding: TemporalBinding::Wildcard,
            };
        }

        if let Some((starts_at, ends_at)) = parse_window(&raw) {
            return Self {
                raw,
                binding: TemporalBinding::Window { starts_at, ends_at },
            };
        }

        if is_rfc3339_timestamp(&raw) {
            return Self {
                raw: raw.clone(),
                binding: TemporalBinding::Timestamp(raw),
            };
        }

        if is_phase_token(&raw) {
            return Self {
                raw: raw.clone(),
                binding: TemporalBinding::Phase(raw),
            };
        }

        Self {
            raw: raw.clone(),
            binding: TemporalBinding::Relative(raw),
        }
    }

    pub fn reason(&self) -> &'static str {
        match self.binding {
            TemporalBinding::Timestamp(_) => "timestamp_binding",
            TemporalBinding::Phase(_) => "phase_binding",
            TemporalBinding::Window { .. } => "window_binding",
            TemporalBinding::Relative(_) => "relative_time_unresolved",
            TemporalBinding::Wildcard => "wildcard_temporal_binding",
        }
    }

    pub fn strong_reason(&self) -> &'static str {
        match self.binding {
            TemporalBinding::Timestamp(_) => "simular_timestamp_binding",
            TemporalBinding::Phase(_) => "simular_phase_binding",
            TemporalBinding::Window { .. } => "simular_window_binding",
            TemporalBinding::Relative(_) => "simular_relative_time_unresolved",
            TemporalBinding::Wildcard => "simular_wildcard_temporal_binding",
        }
    }

    pub fn is_unresolved_for_mode(&self, mode: Mode) -> bool {
        matches!(self.binding, TemporalBinding::Relative(_))
            && matches!(mode, Mode::Operational | Mode::Closure)
    }
}

pub trait ClockView {
    fn now_utc(&self) -> &str;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FixedClock {
    now: String,
}

impl FixedClock {
    pub fn new(now: impl Into<String>) -> Self {
        Self {
            now: now.into().trim().to_string(),
        }
    }
}

impl ClockView for FixedClock {
    fn now_utc(&self) -> &str {
        &self.now
    }
}

fn parse_window(raw: &str) -> Option<(String, String)> {
    let (starts_at, ends_at) = raw.split_once("..")?;
    let starts_at = starts_at.trim();
    let ends_at = ends_at.trim();

    if starts_at.is_empty() || ends_at.is_empty() {
        return None;
    }

    if is_rfc3339_timestamp(starts_at) && is_rfc3339_timestamp(ends_at) {
        return Some((starts_at.to_string(), ends_at.to_string()));
    }

    None
}

fn is_rfc3339_timestamp(raw: &str) -> bool {
    OffsetDateTime::parse(raw, &Rfc3339).is_ok()
}

fn is_phase_token(raw: &str) -> bool {
    raw.starts_with("before_")
        || raw.starts_with("after_")
        || raw.starts_with("at_")
        || raw.starts_with("during_")
        || raw.starts_with("phase:")
}

pub struct WhenSlot;

impl SlotHouse for WhenSlot {
    fn slot(&self) -> Slot {
        Slot::When
    }

    fn resolve(
        &self,
        input: &LogLine,
        ctx: &RunContext<'_>,
        _decision: &RuntimeDecision,
    ) -> RuntimeSlot {
        let temporal = TemporalResolution::parse(&input.when);

        if temporal.is_unresolved_for_mode(ctx.mode) {
            return RuntimeSlot {
                value: input.when.clone(),
                resolution: SlotResolution::Doubt,
                reason: "relative_time_unresolved".to_string(),
                next: Some("confirmed_by".to_string()),
                route: None,
                before: None,
                after: None,
            };
        }

        slot::ok_for_house(
            &input.when,
            temporal.reason(),
            temporal.strong_reason(),
            "confirmed_by",
            self,
            ctx,
        )
    }
}
