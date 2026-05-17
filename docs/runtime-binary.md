# Runtime Binary Artifact

## Purpose

This canon package produces a local runtime binary artifact for downstream
systems that need to invoke LogLine without vendoring canon source code or
reimplementing the runtime.

The v0 artifact is:

```text
target/release/logline
```

Downstream systems may point:

```bash
export LOGLINE_RUNTIME_BIN=/absolute/path/to/target/release/logline
```

## Build

Build with normal Cargo:

```bash
cargo build --release
```

The binary path is:

```text
target/release/logline
```

The same source also supports debug builds:

```text
target/debug/logline
```

Release builds are preferred for pinned downstream use.

## Safe Probe

The runtime binary has a safe probe command:

```bash
target/release/logline --version
```

It prints JSON containing runtime identity, canon version, git commit if known,
declared LIP support, supported features, and `external_effects: false`.

Equivalent:

```bash
target/release/logline version
```

The safe status command is:

```bash
target/release/logline status
```

It reports that the local runtime is available and that the probe itself has no
external effects.

## Pinning Rule

External systems SHOULD build this binary from a pinned git commit.

External adapters MUST NOT run `git pull`, fetch network dependencies, or update
canon source in hot runtime paths. A production integration should select a
commit, build the binary, record the commit, and then invoke that local artifact.

## Downstream Rule

Downstream systems SHOULD NOT vendor canon source code. They should either:

- depend on the published canon package as source for review and builds; or
- call a pinned local runtime binary via `LOGLINE_RUNTIME_BIN`.

For v0, npm publication is not required.

## Local GitHub Release Pack

The canon repo can produce a local release pack for manual upload to a GitHub
Release:

```bash
scripts/release/build-runtime-release-pack.sh
```

The script:

```text
builds target/release/logline
runs the safe version/status probes
copies the binary into dist/runtime-release/<tag>/
writes a SHA-256 checksum
writes logline-runtime-manifest.json
writes RELEASE_NOTES.md
prints manual gh release commands
```

It does not publish to GitHub by default.

To override the suggested tag:

```bash
LOGLINE_RUNTIME_RELEASE_TAG=logline-runtime-v0.1.0-<commit> \
  scripts/release/build-runtime-release-pack.sh
```

By default the suggested tag is:

```text
logline-runtime-v0.1.0-<commit_short>
```

The local pack is intended for this model:

```text
local canon repo
  -> cargo build --release
  -> local dist/runtime-release/<tag>/ pack
  -> human-created GitHub Release
  -> downstream provisioning downloads the asset
  -> runtime hot path calls LOGLINE_RUNTIME_BIN
```

Runtime hot paths MUST NOT download from GitHub, run `git pull`, or update canon
source. GitHub release assets are provisioning material, not runtime behavior.

For v0:

```text
GitHub Actions is not required.
npm is not required.
Canon source is not vendored downstream.
The release asset is a local runtime binary plus manifest/checksum/notes.
```

## Scope

The runtime binary is allowed to parse, validate, walk, report status, and print
receipts or reports already supported by the canon crates.

It MUST NOT implement world-specific adapters here.

Out of scope for this artifact:

```text
Minilab admission
passport or visa product logic
Gateway
Supabase persistence
MCP execution
Cloudflare integration
LAB Runner
receipt persistence runtime
external effects
if_ok release execution into the real world
```

Adapters remain world contact. The runtime binary remains the local LogLine
mouth for the nine-slot walk.
