# Runtime declaration

## Purpose

A runtime declaration prevents vague compatibility claims. It tells users which
canon version, features, and conformance level are actually supported.

## Minimum shape

```json
{
  "runtime": "logline-runtime-rs",
  "runtime_version": "0.1.x",
  "canon_version": "0.2.0-draft",
  "conformance_status": "partial",
  "supports": [
    "nine_slot_tuple",
    "canonical_tuple_digest",
    "branch_routing",
    "status_lifecycle"
  ]
}
```

## Binary probe

The local runtime binary artifact is `logline`.

After:

```bash
cargo build --release
```

the binary exists at:

```text
target/release/logline
```

Downstream systems may set:

```bash
LOGLINE_RUNTIME_BIN=/absolute/path/to/target/release/logline
```

The safe probe command is:

```bash
target/release/logline --version
```

It emits JSON with `runtime`, `binary`, `canon_version`, `git_commit`,
`lip_support`, `features`, and `external_effects`.

The safe status command is:

```bash
target/release/logline status
```

These commands do not execute adapters, persist receipts, fetch network state,
or mutate the world.

## if_doubt addendum extension

```json
{
  "runtime": "logline-runtime-rs",
  "runtime_version": "0.1.x",
  "canon_version": "0.2.0-draft",
  "conformance_status": "partial",
  "supports": [
    "nine_slot_tuple",
    "schema_before_semantics",
    "canonical_tuple_digest",
    "valid_invalid_ambiguous",
    "if_doubt_trace",
    "simulation_receipt",
    "receipt_encoding_v0",
    "jcs_rfc8785_receipt_hash",
    "conformance_runner"
  ]
}
```

## Status values

```text
none       runtime has not run conformance
partial    runtime supports a declared subset
full       runtime passed the canonical suite for the declared version
failed     runtime attempted conformance and failed
unknown    runtime did not provide evidence
```

## Rule

A runtime MUST NOT report `full` unless it can attach a conformance report for
the declared canon version.

A runtime MUST NOT claim receipt encoding conformance unless it can compute the
existing nine-slot `tuple_hash`, JCS `result_hash`, and JCS `receipt_hash`
according to `spec/receipt-encoding.md`.
