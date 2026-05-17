# Canon vs Runtime

LogLine is canon first. A runtime is an implementation that claims adherence to
a specific canon version.

## Canon

The canon defines the normative shape:

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
```

The canon owns:

- required positions;
- textual and JSON shape;
- route classes;
- lifecycle vocabulary;
- evidence expectations;
- conformance cases;
- versioned normative changes.

## Runtime

A runtime executes, validates, walks, optimizes, and reports. It may be written
in Rust, Python, JavaScript, or another language.

A runtime owns:

- parser implementation;
- validator implementation;
- evidence adapters;
- CLI or SDK surface;
- performance strategy;
- storage strategy;
- receipts emitted by that implementation.

It does not own the norm.

## Conformance

An implementation should be able to say:

```text
I implement LogLine canon version X.
I ran conformance suite Y.
These receipts prove the observed result.
```

Conformance is measured against published artifacts, not against hidden behavior
inside the Rust runtime.

## Boundary

The Rust runtime is the reference implementation. It is important, but not
sovereign.

The authority chain is:

```text
canon -> conformance corpus -> receipts -> implementation report
```

The runtime sits in the implementation layer.
