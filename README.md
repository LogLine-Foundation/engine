# LogLine engine

Rust workspace that **implements** the LogLine canon. Ten crates: nine slot crates (`who`, `did`, `this`, `when`, `confirmed_by`, `if_ok`, `if_doubt`, `if_not`, `status`) plus one shell (`logline-cli`).

```
canon defines.
conformance proves.
engine implements.
governance evolves.
```

This repository **does not**:

- define the canon (see [`LogLine-Foundation/canon`](https://github.com/LogLine-Foundation/canon))
- own conformance (see [`LogLine-Foundation/conformance`](https://github.com/LogLine-Foundation/conformance))
- govern LIPs (see [`LogLine-Foundation/governance`](https://github.com/LogLine-Foundation/governance))

## Workspace layout

```
crates/
  who/                — origin slot
  did/                — action slot
  this/               — subject slot
  when/               — time slot
  confirmed_by/       — confirmation slot
  if_ok/              — happy path slot
  if_doubt/           — doubt branch slot
  if_not/             — refusal slot
  status/             — lifecycle status slot
  logline-cli/        — `logline` binary, the shell

docs/                 — engine-specific documentation
scripts/              — release helpers
examples/             — usage examples (not vector fixtures — those live in conformance)
```

## Build

```bash
cargo build --release
cargo test
```

The resulting binary is `target/release/logline`. Downstream systems point `LOGLINE_RUNTIME_BIN` at this path.

## Conformance

This engine claims adherence to:

- `logline.receipt.v0` (current normative receipt format)
- `logline.canon.v0` (language grammar and lifecycle)

Adherence is proved by running the conformance suite from `LogLine-Foundation/conformance` against this engine's binary. Conformance is not declared by this repo — it is claimed, then proved externally.

## Discipline

- Runtime is **not sovereign**: it implements admissible behavior, it does not define it.
- Natural language never executes — only nine-slot LogLine acts cross the runtime boundary.
- Receipts beat stories — every executable claim must produce a verifiable receipt under the canon.
- LABs (the downstream machines that consume this engine) execute; they do not govern.
