use crate::error::{LogLineError, Result};
use crate::logline::LogLine;
use crate::parse::parse_logline;
use crate::validate::{validate_canon, validate_shape};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Canon {
    pub canon: String,
    pub version: String,
    pub form: BTreeMap<String, String>,
    pub runtime_walk: Vec<String>,
    pub route_kinds: RouteKinds,
    pub status_lifecycle: StatusLifecycle,

    #[serde(default)]
    pub law: Vec<LogLine>,

    #[serde(default)]
    pub prohibition: Vec<LogLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteKinds {
    pub ok: Vec<String>,
    pub doubt: Vec<String>,

    #[serde(rename = "not")]
    pub not_: Vec<String>,

    pub none: Vec<String>,
}

pub type StatusLifecycle = BTreeMap<String, Vec<String>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteClass {
    Ok,
    Doubt,
    Not,
    None,
}

impl Canon {
    pub fn from_json_path(path: impl AsRef<Path>) -> Result<Self> {
        let text = std::fs::read_to_string(path)?;
        Self::from_json_str(&text)
    }

    pub fn from_json_str(text: &str) -> Result<Self> {
        let canon: Canon = serde_json::from_str(text)?;
        validate_canon(&canon)?;
        Ok(canon)
    }

    pub fn loglines_from_path(path: impl AsRef<Path>) -> Result<Vec<LogLine>> {
        let text = std::fs::read_to_string(path)?;
        Self::loglines_from_str(&text)
    }

    pub fn loglines_from_str(text: &str) -> Result<Vec<LogLine>> {
        let mut lines = Vec::new();

        for line in text.lines().map(str::trim).filter(|line| !line.is_empty()) {
            if is_form_require_directive(line)? {
                continue;
            }

            let logline = parse_logline(line)?;
            validate_shape(&logline)?;
            lines.push(logline);
        }

        Ok(lines)
    }

    pub fn classify_route(&self, token: &str) -> Result<RouteClass> {
        if token == "*" {
            return Ok(RouteClass::None);
        }

        if self.route_kinds.ok.iter().any(|t| t == token)
            || self.law.iter().any(|line| line.if_ok == token)
            || self.prohibition.iter().any(|line| line.if_ok == token)
        {
            return Ok(RouteClass::Ok);
        }
        if self.route_kinds.doubt.iter().any(|t| t == token)
            || self.law.iter().any(|line| line.if_doubt == token)
            || self.prohibition.iter().any(|line| line.if_doubt == token)
        {
            return Ok(RouteClass::Doubt);
        }
        if self.route_kinds.not_.iter().any(|t| t == token)
            || self.law.iter().any(|line| line.if_not == token)
            || self.prohibition.iter().any(|line| line.if_not == token)
        {
            return Ok(RouteClass::Not);
        }
        if self.route_kinds.none.iter().any(|t| t == token) {
            return Ok(RouteClass::None);
        }

        Err(LogLineError::UnknownRouteToken(token.to_string()))
    }

    pub fn classify_route_field(&self, field: &'static str, token: &str) -> Result<RouteClass> {
        match self.classify_route(token) {
            Ok(class) => Ok(class),
            Err(LogLineError::UnknownRouteToken(_)) => match field {
                "if_ok" => Ok(RouteClass::Ok),
                "if_doubt" => Ok(RouteClass::Doubt),
                "if_not" => Ok(RouteClass::Not),
                _ => Err(LogLineError::UnknownPosition(field.to_string())),
            },
            Err(error) => Err(error),
        }
    }
}

pub fn is_form_require_directive(line: &str) -> Result<bool> {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    if !tokens.starts_with(&["form", "require"]) {
        return Ok(false);
    }

    let required = &tokens[2..];
    if required == LogLine::POSITIONS {
        return Ok(true);
    }

    Err(LogLineError::InvalidCanonForm)
}
