use serde_json::Value;
use std::error::Error;
use std::io;
use std::process::{Command, Output};

fn run_logline(args: &[&str]) -> Result<Output, Box<dyn Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_logline"))
        .args(args)
        .output()?;

    if output.status.success() {
        return Ok(output);
    }

    Err(io::Error::other(format!("logline command failed: {:?}", args)).into())
}

fn json_output(args: &[&str]) -> Result<Value, Box<dyn Error>> {
    let output = run_logline(args)?;
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn string_array_contains(value: &Value, key: &str, expected: &str) -> Result<bool, Box<dyn Error>> {
    let items = value[key]
        .as_array()
        .ok_or_else(|| io::Error::other(format!("{key} must be an array")))?;

    Ok(items.iter().any(|item| item.as_str() == Some(expected)))
}

#[test]
fn version_probe_reports_runtime_identity() -> Result<(), Box<dyn Error>> {
    let value = json_output(&["--version"])?;

    assert_eq!(value["runtime"], "logline-runtime-rs");
    assert_eq!(value["binary"], "logline");
    assert_eq!(value["package"], "logline-cli");
    assert_eq!(value["canon_version"], "0.2.0-draft");
    assert_eq!(value["external_effects"], false);
    assert!(string_array_contains(
        &value,
        "features",
        "canonical_tuple_digest"
    )?);
    assert!(string_array_contains(
        &value,
        "features",
        "receipt_encoding_profile"
    )?);
    assert!(string_array_contains(
        &value,
        "features",
        "adapter_protocol"
    )?);
    assert!(string_array_contains(&value, "lip_support", "LIP-0004")?);
    assert!(string_array_contains(&value, "lip_support", "LIP-0005")?);
    assert!(string_array_contains(&value, "lip_support", "LIP-0006")?);
    assert!(string_array_contains(&value, "lip_support", "LIP-0007")?);

    Ok(())
}

#[test]
fn version_subcommand_matches_probe_shape() -> Result<(), Box<dyn Error>> {
    let value = json_output(&["version"])?;

    assert_eq!(value["runtime"], "logline-runtime-rs");
    assert_eq!(value["external_effects"], false);

    Ok(())
}

#[test]
fn status_reports_safe_local_runtime() -> Result<(), Box<dyn Error>> {
    let value = json_output(&["status"])?;

    assert_eq!(value["runtime_available"], true);
    assert_eq!(value["authoritative_runtime"], true);
    assert_eq!(value["runtime"], "logline-runtime-rs");
    assert_eq!(value["binary"], "logline");
    assert_eq!(value["canon_version"], "0.2.0-draft");
    assert_eq!(value["adapter_protocol_supported"], true);
    assert_eq!(value["receipt_encoding_profile_supported"], true);
    assert_eq!(value["canonical_tuple_digest_supported"], true);
    assert_eq!(value["external_effects"], false);

    Ok(())
}
