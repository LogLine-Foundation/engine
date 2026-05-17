# Evidence model

## Purpose

`confirmed_by` is the pivot of the LogLine runtime. It determines whether a
candidate act has enough evidence to move toward `if_ok`, must remain in
`if_doubt`, or should be rejected by `if_not`.

## Evidence kinds

### none

No evidence was supplied.

Rules:

- MUST NOT release consequence by default.
- MAY enter `if_doubt` for clarification, suspension, or simulation.
- MAY be allowed only by explicit override or narrow canon rule.

### token

A named witness or local capability token.

Example:

```text
ana
operator:dan
```

Rules:

- Useful for local workflows.
- Not sufficient for strong public verification unless mapped to authority.

### receipt

A prior runtime artifact proving that a bounded observation occurred.

Example:

```text
receipt:sha256:abc...
```

Rules:

- SHOULD be dereferenceable or reproducible.
- MUST define scope.
- MUST NOT be generalized beyond its observed claim.
- MAY be referenced by `receipt_hash` in `evidence.input_receipts`.
- MAY become `confirmed_by` evidence for a later LogLine when its scope matches
  the later claim.
- MUST keep the canonical slots at top level when encoded as a LogLine receipt.

### digest

A cryptographic digest of a tuple, artifact, stdout, input, or receipt.

Example:

```text
digest:sha256:abc...
```

Rules:

- Identifies bytes or canonical tuple identity.
- Does not by itself prove authority, intent, or execution.

### signature

A verifiable signature over a canonical message.

Example:

```text
signature:ed25519:...
```

Rules:

- Requires key, algorithm, message, and verification result.
- SHOULD be linked to DID or authority document in future versions.

### quorum

A threshold witness pattern.

Example:

```text
quorum:2:release_committee
```

Rules:

- Duplicate witnesses MUST NOT count twice.
- Quorum scope MUST be declared.
- Quorum result SHOULD produce a receipt.

## Evidence collapse

Evidence collapse means the runtime has enough scoped evidence to route toward
`if_ok`. It does not mean broad truth, production safety, or human-signature
approval unless those are explicitly part of the claim and receipt.

## Receipt hashes

Receipt encoding distinguishes three identities:

```text
tuple_hash    identity of exactly the nine-slot LogLine tuple
result_hash   identity of the produced result
receipt_hash  identity of the historical receipt emission
```

`tuple_hash` uses the LogLine length-prefixed tuple profile.

`result_hash` and `receipt_hash` use JCS / RFC 8785 canonical JSON with SHA-256
in v0.

Changing transport changes `receipt_hash`, but not `result_hash`.
