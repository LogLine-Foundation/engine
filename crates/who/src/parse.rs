use crate::error::{LogLineError, Result};
use crate::logline::LogLine;
use std::collections::BTreeMap;

pub fn parse_logline(input: &str) -> Result<LogLine> {
    let trimmed = input.trim();

    if trimmed.contains('=') {
        parse_keyed_logline(trimmed)
    } else {
        parse_marked_logline(trimmed)
    }
}

pub fn parse_keyed_logline(input: &str) -> Result<LogLine> {
    let mut found: BTreeMap<String, String> = BTreeMap::new();
    let mut order: Vec<String> = Vec::new();

    for token in input.split_whitespace() {
        let Some((key, value)) = token.split_once('=') else {
            return Err(LogLineError::UnparsedMaterial(token.to_string()));
        };

        if !LogLine::POSITIONS.contains(&key) {
            return Err(LogLineError::UnknownPosition(key.to_string()));
        }

        if found.insert(key.to_string(), unquote(value)).is_some() {
            return Err(LogLineError::DuplicatePosition(key.to_string()));
        }

        order.push(key.to_string());
    }

    for (idx, expected) in LogLine::POSITIONS.iter().enumerate() {
        if order.get(idx).map(String::as_str) != Some(*expected) {
            return Err(LogLineError::PositionOrder);
        }
    }

    object_from_map(&found)
}

pub fn parse_marked_logline(input: &str) -> Result<LogLine> {
    let tokens: Vec<&str> = input.split_whitespace().collect();

    let confirmed_idx = find_marker_after(&tokens, "confirmed_by", 4)?;
    let if_ok_idx = find_marker_after(&tokens, "if_ok", confirmed_idx + 1)?;
    let if_doubt_idx = find_marker_after(&tokens, "if_doubt", if_ok_idx + 1)?;
    let if_not_idx = find_marker_after(&tokens, "if_not", if_doubt_idx + 1)?;
    let status_idx = find_marker_after(&tokens, "status", if_not_idx + 1)?;

    if confirmed_idx < 4
        || !(confirmed_idx < if_ok_idx
            && if_ok_idx < if_doubt_idx
            && if_doubt_idx < if_not_idx
            && if_not_idx < status_idx)
    {
        return Err(LogLineError::PositionOrder);
    }

    let who = join_tokens(&tokens[0..1]);
    let did = join_tokens(&tokens[1..2]);
    let this_ = join_tokens(&tokens[2..3]);
    let when = join_tokens(&tokens[3..confirmed_idx]);
    let confirmed_by = join_tokens(&tokens[(confirmed_idx + 1)..if_ok_idx]);
    let if_ok = join_tokens(&tokens[(if_ok_idx + 1)..if_doubt_idx]);
    let if_doubt = join_tokens(&tokens[(if_doubt_idx + 1)..if_not_idx]);
    let if_not = join_tokens(&tokens[(if_not_idx + 1)..status_idx]);
    let status = join_tokens(&tokens[(status_idx + 1)..]);

    Ok(LogLine::new(
        who,
        did,
        this_,
        when,
        confirmed_by,
        if_ok,
        if_doubt,
        if_not,
        status,
    ))
}

fn find_marker_after(tokens: &[&str], marker: &'static str, start: usize) -> Result<usize> {
    tokens
        .iter()
        .enumerate()
        .skip(start)
        .find_map(|(idx, token)| (*token == marker).then_some(idx))
        .ok_or(LogLineError::MissingMarker(marker))
}

fn object_from_map(map: &BTreeMap<String, String>) -> Result<LogLine> {
    Ok(LogLine::new(
        required(map, "who")?,
        required(map, "did")?,
        required(map, "this")?,
        required(map, "when")?,
        required(map, "confirmed_by")?,
        required(map, "if_ok")?,
        required(map, "if_doubt")?,
        required(map, "if_not")?,
        required(map, "status")?,
    ))
}

fn required(map: &BTreeMap<String, String>, key: &'static str) -> Result<String> {
    map.get(key)
        .cloned()
        .ok_or(LogLineError::MissingPosition(key))
}

fn join_tokens(tokens: &[&str]) -> String {
    tokens.join(" ")
}

fn unquote(value: &str) -> String {
    let bytes = value.as_bytes();
    if bytes.len() >= 2
        && ((bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\''))
    {
        value[1..value.len() - 1].to_string()
    } else {
        value.to_string()
    }
}
