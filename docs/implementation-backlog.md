# Implementation backlog - schema and conformance foundation

Status: Not fully applied  
Purpose: Make LIP-0001 concrete enough to implement.

This is execution backlog, not a normative LIP. The normative proposal lives in
`lips/LIP-0001-canon-hardening-and-conformance.md`.

## Required states

```text
Proposed -> Accepted -> Implemented -> Superseded | Rejected | Withdrawn
```

## Backlog

### 1. Schema before semantics

Add CLI support:

```bash
logline canon check --schema spec/logline-canon.schema.json source/logline.canon.json
```

Order:

1. parse JSON;
2. validate JSON Schema;
3. run semantic canon validation;
4. emit receipt/report.

### 2. Conformance runner

Add CLI support:

```bash
logline conformance run conformance/cases.json
```

The runner should process:

```text
valid
invalid
ambiguous
```

and produce a machine-readable report.

### 3. Canon version declaration

Runtime output should include:

```json
{
  "runtime": "logline-runtime-rs",
  "runtime_version": "0.1.x",
  "canon_version": "0.2.0-draft",
  "conformance_status": "partial"
}
```

### 4. Receipt export

Minimum receipt shape:

```json
{
  "receipt_id": "...",
  "canon_version": "0.2.0-draft",
  "logline_digest": "sha256:...",
  "claim": "...",
  "observed": "...",
  "stdout_digest": "sha256:..."
}
```

### 5. Institutional hygiene

Add or confirm:

```text
CONTRIBUTING.md
CODE_OF_CONDUCT.md
SECURITY.md
CHANGELOG.md
rust-toolchain.toml
.github/workflows/ci.yml
```

### 6. CI gates

Minimum CI:

```text
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
schema parse
conformance fixture shape
mythology check
runtime audit
```

## Definition of implemented

LIP-0001 is implemented only when the schema, conformance runner, runtime
declaration, and CI gates exist and produce receipts or reports. A document
alone is not implementation.
