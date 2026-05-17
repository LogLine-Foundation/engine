# LogLine Canon Changelog

This changelog tracks changes to the living canon package. Runtime-only changes
belong near the crate or implementation that owns them.

## Unreleased

- Standardized the `logline` binary as the local runtime artifact for
  `LOGLINE_RUNTIME_BIN`.
- Added safe `logline --version`, `logline version`, and `logline status`
  probes with runtime/canon identity and `external_effects: false`.
- Added runtime binary artifact documentation for pinned local builds.
- Added a local runtime release pack script that builds the binary, writes a
  checksum, emits a manifest, prepares release notes, and prints manual GitHub
  release commands without publishing by default.
- Added LIP-0003 for LogLine receipt encoding and content addressing.
- Added the receipt encoding spec, JSON Schema, examples, and conformance cases.
- Clarified that `tuple_hash` uses the existing length-prefixed nine-slot tuple
  profile while `result_hash` and `receipt_hash` use JCS / RFC 8785 JSON.
- Added draft LIP-0004, LIP-0005, and LIP-0006 for the lateral adapter
  protocol, adapter declarations, and adapter conformance.
- Added lateral adapter protocol LogLine sources, specs, examples, and
  conformance cases without adding runtime anatomy.
- Reorganized the canon as a single package root with `Cargo.toml`, `crates/`,
  `source/`, `spec/`, `conformance/`, `docs/`, `lips/`, and `proposals/`.
- Promoted the Rust slot runtime workspace to the package root.
- Renamed canonical source files to `source/logline.canon.json` and
  `source/logline.canon.logline`.
- Integrated the if-doubt auditable simulation package as specs, docs, LIPs,
  proposal notes, and conformance material.
- Kept `if_doubt` as a branch of auditable simulation, not execution.

## Initial Canon

- Published the nine-position LogLine source in JSON and `.logline` forms.
- Established `confirmed_by` as the admissibility pivot.
- Established the invariant runtime body:

```text
who did this when confirmed_by if_ok if_doubt if_not status
```
