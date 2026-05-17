# Crates

This directory contains 9 runtime crates and 1 shell crate.

```text
who
did
this
when
confirmed_by
if_ok
if_doubt
if_not
status
logline-cli
```

The rule:

```text
who through status = runtime body
logline-cli        = shell / mouth
```

These crates are intentionally thin. They are the spine of the canon, not the
final clothing of any one product or domain.

## Runtime Crates

Every runtime crate corresponds to exactly one LogLine position.

No runtime crate may be added unless the canon itself changes its 9-slot body.

```text
who            accountable origin
did            movement or act
this           bounded matter
when           time, phase, simulation condition
confirmed_by   evidence pivot
if_ok          release route
if_doubt       doubt, simulation, clarification route
if_not         rejection or falsification route
status         lifecycle closure
```

## Shell Crate

`logline-cli` exists because Rust needs a binary package.

It is allowed to parse arguments, load files, call the walk, and print output.
It must not own runtime judgment.

## Scaling Rule

Add modes, adapters, receipts, stores, SDKs, policies, and domain vocabularies
around these crates.

Do not add new runtime anatomy.

