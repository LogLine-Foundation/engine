#![forbid(unsafe_code)]

use logline_status::{
    is_form_require_directive, parse_logline, run_with_context, validate_against_canon, Canon,
    LogLine, Mode, Operation, RunContext,
};
use std::str::FromStr;

const RUNTIME_NAME: &str = "logline-runtime-rs";
const RUNTIME_BINARY: &str = "logline";
const CANON_VERSION: &str = "0.2.0-draft";
const SUPPORTED_LIPS: [&str; 4] = ["LIP-0004", "LIP-0005", "LIP-0006", "LIP-0007"];
const SUPPORTED_FEATURES: [&str; 3] = [
    "canonical_tuple_digest",
    "receipt_encoding_profile",
    "adapter_protocol",
];

fn main() -> logline_status::Result<()> {
    let mut args = std::env::args().skip(1);
    let Some(command) = args.next() else {
        usage_and_exit();
    };

    match command.as_str() {
        "--version" | "-V" | "version" => {
            println!("{}", serde_json::to_string_pretty(&runtime_probe())?);
        }
        "status" => {
            println!("{}", serde_json::to_string_pretty(&runtime_status())?);
        }
        "check-canon" => {
            let Some(canon_path) = args.next() else {
                usage_and_exit();
            };
            let _canon = Canon::from_json_path(canon_path)?;
            println!("{{\"ok\":true,\"checked\":\"canon\"}}");
        }
        "parse-logline" => {
            let Some(logline_path) = args.next() else {
                usage_and_exit();
            };
            let text = std::fs::read_to_string(logline_path)?;
            for line in text.lines().filter(|line| !line.trim().is_empty()) {
                if is_form_require_directive(line.trim())? {
                    continue;
                }
                let logline = parse_logline(line)?;
                println!("{}", serde_json::to_string_pretty(&logline)?);
            }
        }
        "validate" => {
            let Some(canon_path) = args.next() else {
                usage_and_exit();
            };
            let Some(logline_path) = args.next() else {
                usage_and_exit();
            };
            let canon = Canon::from_json_path(canon_path)?;
            let logline = read_logline(logline_path)?;
            validate_against_canon(&canon, &logline)?;
            println!("{{\"ok\":true,\"validated\":\"logline\"}}");
        }
        "run" | "walk" => {
            let run_args = parse_run_args(args.collect());
            let canon = Canon::from_json_path(run_args.canon_path)?;
            let logline = read_logline(run_args.input_path)?;
            let mut ctx = RunContext::new(&canon, run_args.operation);
            ctx.mode = run_args.mode;
            ctx.provided_evidence = run_args.evidence;
            ctx.allow_no_confirmation = run_args.allow_no_confirmation;
            let result = run_with_context(&logline, ctx)?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        _ => usage_and_exit(),
    }

    Ok(())
}

struct RunArgs {
    canon_path: String,
    input_path: String,
    mode: Mode,
    operation: Operation,
    evidence: Vec<String>,
    allow_no_confirmation: bool,
}

fn parse_run_args(raw: Vec<String>) -> RunArgs {
    let mut mode = Mode::Algebra;
    let mut operation = Operation::Confirmar;
    let mut evidence = Vec::new();
    let mut allow_no_confirmation = false;
    let mut canon_path = None;
    let mut input_path = None;

    let mut iter = raw.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--op" => {
                let Some(value) = iter.next() else {
                    usage_and_exit();
                };
                operation = match Operation::from_str(&value) {
                    Ok(operation) => operation,
                    Err(error) => {
                        eprintln!("{error}");
                        usage_and_exit();
                    }
                };
            }
            "--mode" => {
                let Some(value) = iter.next() else {
                    usage_and_exit();
                };
                mode = match Mode::from_str(&value) {
                    Ok(mode) => mode,
                    Err(error) => {
                        eprintln!("{error}");
                        usage_and_exit();
                    }
                };
            }
            "--evidence" => {
                let Some(value) = iter.next() else {
                    usage_and_exit();
                };
                evidence.push(value);
            }
            "--allow-no-confirmation" => {
                allow_no_confirmation = true;
            }
            "--canon" => {
                canon_path = iter.next();
            }
            "--input" => {
                input_path = iter.next();
            }
            _ if canon_path.is_none() => canon_path = Some(arg),
            _ if input_path.is_none() => input_path = Some(arg),
            _ => usage_and_exit(),
        }
    }

    let Some(canon_path) = canon_path else {
        usage_and_exit();
    };
    let Some(input_path) = input_path else {
        usage_and_exit();
    };

    RunArgs {
        canon_path,
        input_path,
        mode,
        operation,
        evidence,
        allow_no_confirmation,
    }
}

fn runtime_probe() -> serde_json::Value {
    serde_json::json!({
        "runtime": RUNTIME_NAME,
        "binary": RUNTIME_BINARY,
        "package": env!("CARGO_PKG_NAME"),
        "package_version": env!("CARGO_PKG_VERSION"),
        "canon_version": CANON_VERSION,
        "git_commit": git_commit(),
        "lip_support": SUPPORTED_LIPS,
        "features": SUPPORTED_FEATURES,
        "external_effects": false
    })
}

fn runtime_status() -> serde_json::Value {
    serde_json::json!({
        "runtime_available": true,
        "authoritative_runtime": true,
        "runtime": RUNTIME_NAME,
        "binary": RUNTIME_BINARY,
        "canon_version": CANON_VERSION,
        "adapter_protocol_supported": true,
        "receipt_encoding_profile_supported": true,
        "canonical_tuple_digest_supported": true,
        "external_effects": false
    })
}

fn git_commit() -> &'static str {
    match option_env!("LOGLINE_GIT_COMMIT") {
        Some(value) if !value.is_empty() => value,
        _ => "unknown",
    }
}

fn read_logline(path: String) -> logline_status::Result<LogLine> {
    let text = std::fs::read_to_string(path)?;
    let trimmed = text.trim();

    if trimmed.starts_with('{') {
        Ok(serde_json::from_str(trimmed)?)
    } else {
        parse_logline(trimmed)
    }
}

fn usage_and_exit() -> ! {
    eprintln!("usage:");
    eprintln!("  logline --version");
    eprintln!("  logline version");
    eprintln!("  logline status");
    eprintln!("  logline check-canon <canon.json>");
    eprintln!("  logline parse-logline <canon.logline|act.logline>");
    eprintln!("  logline validate <canon.json> <act.json|act.logline>");
    eprintln!(
        "  logline run [--mode algebra] [--op confirmar] [--evidence token] <canon.json> <act.json|act.logline>"
    );
    eprintln!("  logline run --canon <canon.json> --input <act.json|act.logline>");
    eprintln!("  logline walk <canon.json> <act.json|act.logline>  # diagnostic alias");
    std::process::exit(2)
}
