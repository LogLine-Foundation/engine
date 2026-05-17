# Security Policy

This policy covers the LogLine canon materials and conformance artifacts.

## Scope

Security-sensitive areas include:

- evidence and receipt formats;
- signature and digest rules;
- ledger and append-only history rules;
- conformance claims;
- ambiguity between runtime errors and governed LogLine branches.

## Reporting

Until a public security contact exists, record suspected issues as private
maintainer notes before publishing exploit details.

Reports should include:

- affected canon version;
- affected artifact;
- expected behavior;
- observed behavior;
- proof of impact;
- suggested mitigation, if known.

## Non-Scope

Implementation-specific bugs belong to the affected runtime unless they reveal
ambiguity or weakness in the canon itself.

## Principle

No implementation should infer confirmation from absence. `confirmed_by = none`
must not silently become success unless explicitly permitted by the target canon
and declared by the implementation.
