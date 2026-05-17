# Engine spec/

Normative-leaning artifacts that the engine implements but the canon has not yet **frozen**.

The canon (`LogLine-Foundation/canon`) holds only what is content-frozen and citable as a stable identifier. Things here are **proposed** behavior the engine honors today, pending promotion to canon via a future LIP (see `LogLine-Foundation/governance/lips/`).

| file | LIP |
|---|---|
| `logline.adapter.v0` | LIP-0004 (adapter protocol) |
| `logline.adapter-declaration.v0` + `.schema.json` | LIP-0005 (declaration profile) |
| `logline.adapter-conformance.v0` | LIP-0006 (adapter conformance) |
| `logline.if-doubt.v0` | LIP-0002 (if-doubt simulation) |
| `logline.if-doubt-simulation.v0` | LIP-0002 |
| `logline.if-doubt-grammar.v0` | LIP-0002 |

Grammar (EBNF for LogLine language proper) is intentionally not here yet — it is planned to live with the `constitutional-runtime` crate in the Minilab workspace, where it is consumed by the IR layer.
