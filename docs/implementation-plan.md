# Implementation plan

## Phase 0 - hygiene

Goal: make the repo safe to build and discuss.

Deliverables:

```text
LICENSE
.gitignore
CHANGELOG.md
SECURITY.md
CONTRIBUTING.md
CODE_OF_CONDUCT.md
rust-toolchain.toml
.github/workflows/ci.yml
```

## Phase 1 - schema and conformance

Goal: make the canon externally checkable.

Deliverables:

```text
spec/logline-canon.schema.json
conformance/cases.json
logline canon check --schema ...
logline conformance run ...
```

Runtime receipt:

```text
schema parse -> semantic validate -> conformance report
```

## Phase 2 - if_doubt auditable simulation

Goal: make doubt productive without becoming fake execution.

Deliverables:

```text
if_doubt trace
simulation receipt
ambiguous result class
possible-world LogLine marker
```

## Phase 3 - receipts and ledger

Goal: make release and closure replayable.

Deliverables:

```text
receipt export
stdout/stderr digest
canonical tuple digest in spec
append-only ledger with prev_digest
```

## Phase 4 - identity and signatures

Goal: make `confirmed_by` stronger than local tokens.

Deliverables:

```text
Ed25519 verification
did:key or did:web mapping
quorum duplicate detection
witness scope document
```

## Phase 5 - governance release

Goal: make the Foundation claim credible.

Deliverables:

```text
LIP process ratified
canon editor role
security review trigger
signed release artifact
published conformance report
```

## Recommended next move

Implement Phase 1 and Phase 2 together in the smallest loop:

```bash
logline conformance run conformance/cases.json
```

The first meaningful proof is not a beautiful document. It is a runtime report
showing valid, invalid, and ambiguous cases, including `if_doubt` simulation
without execution.
