# Release discipline

## Core law

```text
natural language cannot execute
LLM cannot release
if_ok names route but does not execute by itself
Tower/adapters only after release
```

## Branch boundaries

### if_ok

`if_ok` is the branch of real release. It may authorize a consequence route when
`confirmed_by` has collapsed sufficient evidence.

`if_ok` still does not mean execution has occurred. It means the runtime has
released the route to the next authorized layer.

### if_doubt

`if_doubt` is the branch of possible-world simulation, clarification,
suspension, and ghost recording.

It may simulate. It may ask. It may suspend. It may propose. It must not execute
consequence.

### if_not

`if_not` records falsification, denial, rejection, rollback, or prohibition.
It is not a fallback for inconvenience. It is the branch for contradiction or
forbidden shape.

## Adapter boundary

The runtime walk admits and routes. Adapters execute. A conforming system should
not hide adapter execution inside grammar validation.

## Receipt discipline

Receipts must name scope:

```text
simulation receipt: proves simulation only
release receipt: proves release only
execution receipt: proves adapter execution only
closure receipt: proves final status transition only
```

No receipt should be stretched beyond its scope.

## Receipt encoding

A receipt is itself a LogLine-shaped act:

```text
who did this when confirmed_by if_ok if_doubt if_not status
```

with a `hashes` object containing `tuple_hash`, `content_hash`, and `algorithm`
(LIP-0007). The `id` field equals `content_hash`.

A receipt MUST NOT contain top-level `result`, `evidence`, or `transport` fields.
Those are forbidden by `logline.receipt.v0`. The `envelope_hash` lives only on
the Envelope wrapper, never inside the receipt.

The receipt does not contain a nested LogLine. Its nine canonical slots remain
top-level fields.

Release authorization, adapter execution, and closure remain separate receipt
scopes.
