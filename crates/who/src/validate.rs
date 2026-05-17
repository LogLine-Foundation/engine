use crate::canon::{Canon, RouteClass};
use crate::error::{LogLineError, Result};
use crate::logline::LogLine;

pub fn validate_canon(canon: &Canon) -> Result<()> {
    if canon.canon != "logline" {
        return Err(LogLineError::InvalidCanonName);
    }

    let expected_walk: Vec<String> = LogLine::POSITIONS.iter().map(|s| s.to_string()).collect();
    if canon.runtime_walk != expected_walk {
        return Err(LogLineError::InvalidRuntimeWalk);
    }

    let mut actual: Vec<&str> = canon.form.keys().map(String::as_str).collect();
    actual.sort();

    let mut expected = LogLine::POSITIONS.to_vec();
    expected.sort();

    if actual != expected {
        return Err(LogLineError::InvalidCanonForm);
    }

    for (idx, law) in canon.law.iter().enumerate() {
        validate_shape(law).map_err(|e| LogLineError::InvalidLawEntry(idx, e.to_string()))?;
        validate_routes_for_canon_logline(canon, law)
            .map_err(|e| LogLineError::InvalidLawEntry(idx, e.to_string()))?;
    }

    for (idx, prohibition) in canon.prohibition.iter().enumerate() {
        validate_shape(prohibition)
            .map_err(|e| LogLineError::InvalidProhibitionEntry(idx, e.to_string()))?;
        validate_routes_for_canon_logline(canon, prohibition)
            .map_err(|e| LogLineError::InvalidProhibitionEntry(idx, e.to_string()))?;
    }

    Ok(())
}

pub fn validate_shape(logline: &LogLine) -> Result<()> {
    for position in LogLine::POSITIONS {
        let value = logline
            .get(position)
            .ok_or(LogLineError::UnknownPosition(position.to_string()))?;

        if value.is_empty() {
            return Err(LogLineError::EmptyField(position));
        }
    }

    Ok(())
}

pub fn validate_against_canon(canon: &Canon, logline: &LogLine) -> Result<()> {
    validate_shape(logline)?;

    if is_prohibited(canon, logline) {
        return Err(LogLineError::Prohibited);
    }

    validate_routes_for_act(canon, logline)?;
    validate_status(canon, &logline.status)?;

    Ok(())
}

pub fn is_prohibited(canon: &Canon, logline: &LogLine) -> bool {
    canon
        .prohibition
        .iter()
        .any(|pattern| matches_pattern(pattern, logline))
}

fn matches_pattern(pattern: &LogLine, logline: &LogLine) -> bool {
    for position in LogLine::POSITIONS {
        let Some(expected) = pattern.get(position) else {
            return false;
        };
        let Some(actual) = logline.get(position) else {
            return false;
        };

        if expected == "*" {
            continue;
        }

        if expected != actual {
            return false;
        }
    }

    true
}

fn validate_routes_for_act(canon: &Canon, logline: &LogLine) -> Result<()> {
    validate_route_field(canon, "if_ok", &logline.if_ok, RouteClass::Ok)?;
    validate_route_field(canon, "if_doubt", &logline.if_doubt, RouteClass::Doubt)?;
    validate_route_field(canon, "if_not", &logline.if_not, RouteClass::Not)?;
    Ok(())
}

fn validate_routes_for_canon_logline(canon: &Canon, logline: &LogLine) -> Result<()> {
    validate_route_field(canon, "if_ok", &logline.if_ok, RouteClass::Ok)?;
    validate_route_field(canon, "if_doubt", &logline.if_doubt, RouteClass::Doubt)?;
    validate_route_field(canon, "if_not", &logline.if_not, RouteClass::Not)?;
    Ok(())
}

fn validate_route_field(
    canon: &Canon,
    field: &'static str,
    token: &str,
    expected: RouteClass,
) -> Result<()> {
    let actual = canon.classify_route_field(field, token)?;
    if matches!(actual, RouteClass::None) || actual == expected {
        return Ok(());
    }

    Err(LogLineError::RouteClassMismatch {
        token: token.to_string(),
        actual: route_class_name(actual),
        expected: route_class_name(expected),
    })
}

fn route_class_name(class: RouteClass) -> &'static str {
    match class {
        RouteClass::Ok => "ok",
        RouteClass::Doubt => "doubt",
        RouteClass::Not => "not",
        RouteClass::None => "none",
    }
}

fn validate_status(canon: &Canon, status: &str) -> Result<()> {
    const SPECIAL_STATUSES: &[&str] = &[
        "source",
        "translator",
        "canonical_act",
        "compiler",
        "formal",
        "judge",
        "operator",
        "closure",
        "forbidden",
        "canonical",
    ];

    if SPECIAL_STATUSES.contains(&status) {
        return Ok(());
    }

    if canon.status_lifecycle.contains_key(status) {
        return Ok(());
    }

    Err(LogLineError::UnknownStatus(status.to_string()))
}
