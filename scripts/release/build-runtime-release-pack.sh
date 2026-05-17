#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
cd "${REPO_ROOT}"

REPO_SLUG="LogLine-Foundation/canon"
SOURCE_URL="https://github.com/${REPO_SLUG}"
BINARY_NAME="logline"
BINARY_PATH="target/release/${BINARY_NAME}"
CANON_VERSION="0.2.0-draft"
EXPECTED_RUNTIME="logline-runtime-rs"

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required command: $1" >&2
    exit 1
  fi
}

json_quote() {
  python3 -c 'import json,sys; print(json.dumps(sys.argv[1]))' "$1"
}

detect_platform() {
  local os arch
  os="$(uname -s | tr '[:upper:]' '[:lower:]')"
  arch="$(uname -m)"

  case "${os}" in
    darwin) os="darwin" ;;
    linux) os="linux" ;;
    *) os="${os}" ;;
  esac

  case "${arch}" in
    arm64|aarch64) arch="arm64" ;;
    x86_64|amd64) arch="x86_64" ;;
    *) arch="${arch}" ;;
  esac

  printf '%s-%s' "${os}" "${arch}"
}

sha256_file() {
  local path="$1"
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "${path}" | awk '{print $1}'
  elif command -v sha256sum >/dev/null 2>&1; then
    sha256sum "${path}" | awk '{print $1}'
  else
    echo "missing shasum or sha256sum" >&2
    exit 1
  fi
}

require_command git
require_command cargo
require_command python3

COMMIT="$(git rev-parse HEAD)"
COMMIT_SHORT="$(git rev-parse --short=12 HEAD)"
TAG="${LOGLINE_RUNTIME_RELEASE_TAG:-logline-runtime-v0.1.0-${COMMIT_SHORT}}"
PLATFORM="$(detect_platform)"
ASSET_NAME="logline-runtime-${PLATFORM}"
DIST_DIR="dist/runtime-release/${TAG}"
ASSET_PATH="${DIST_DIR}/${ASSET_NAME}"
SHA_PATH="${ASSET_PATH}.sha256"
MANIFEST_PATH="${DIST_DIR}/logline-runtime-manifest.json"
NOTES_PATH="${DIST_DIR}/RELEASE_NOTES.md"
VERSION_PROBE_PATH="${DIST_DIR}/probe.version.json"
STATUS_PROBE_PATH="${DIST_DIR}/probe.status.json"
WORKTREE_STATUS="$(git status --short)"

if [[ -n "${WORKTREE_STATUS}" ]]; then
  echo "warning: working tree is dirty; pack will report worktree_dirty=true" >&2
  echo "${WORKTREE_STATUS}" >&2
fi

cargo build --release

if [[ ! -x "${BINARY_PATH}" ]]; then
  echo "expected runtime binary missing or not executable: ${BINARY_PATH}" >&2
  exit 1
fi

rm -rf "${DIST_DIR}"
mkdir -p "${DIST_DIR}"

"${BINARY_PATH}" --version > "${VERSION_PROBE_PATH}"
"${BINARY_PATH}" status > "${STATUS_PROBE_PATH}"

cp "${BINARY_PATH}" "${ASSET_PATH}"
chmod 0755 "${ASSET_PATH}"

SHA256="$(sha256_file "${ASSET_PATH}")"
printf '%s  %s\n' "${SHA256}" "${ASSET_NAME}" > "${SHA_PATH}"

export REPO_SLUG SOURCE_URL COMMIT COMMIT_SHORT TAG CANON_VERSION EXPECTED_RUNTIME
export BINARY_NAME ASSET_NAME SHA256 VERSION_PROBE_PATH STATUS_PROBE_PATH MANIFEST_PATH
export WORKTREE_DIRTY="false"
if [[ -n "${WORKTREE_STATUS}" ]]; then
  export WORKTREE_DIRTY="true"
fi

python3 - <<'PY'
import json
import os
from pathlib import Path


def load_probe(path: str):
    text = Path(path).read_text()
    try:
        return json.loads(text)
    except json.JSONDecodeError:
        return text


manifest = {
    "artifact_kind": "logline_runtime_binary",
    "binary_name": os.environ["BINARY_NAME"],
    "asset_name": os.environ["ASSET_NAME"],
    "repo": os.environ["REPO_SLUG"],
    "source_url": os.environ["SOURCE_URL"],
    "commit": os.environ["COMMIT"],
    "commit_short": os.environ["COMMIT_SHORT"],
    "suggested_tag": os.environ["TAG"],
    "canon_version": os.environ["CANON_VERSION"],
    "package": "logline-cli",
    "package_version": "0.1.0",
    "runtime": os.environ["EXPECTED_RUNTIME"],
    "lip_support": ["LIP-0003", "LIP-0004", "LIP-0005", "LIP-0006"],
    "features": [
        "canonical_tuple_digest",
        "receipt_encoding_profile",
        "adapter_protocol",
    ],
    "external_effects": False,
    "built_locally": True,
    "github_actions_required": False,
    "npm_package": False,
    "vendored": False,
    "worktree_dirty": os.environ["WORKTREE_DIRTY"] == "true",
    "sha256": os.environ["SHA256"],
    "probe_version_output": load_probe(os.environ["VERSION_PROBE_PATH"]),
    "probe_status_output": load_probe(os.environ["STATUS_PROBE_PATH"]),
}

Path(os.environ["MANIFEST_PATH"]).write_text(
    json.dumps(manifest, indent=2, sort_keys=True) + "\n"
)
PY

if ! python3 -m json.tool "${MANIFEST_PATH}" >/dev/null; then
  echo "manifest is not valid JSON: ${MANIFEST_PATH}" >&2
  exit 1
fi

if [[ "$(sha256_file "${ASSET_PATH}")" != "${SHA256}" ]]; then
  echo "checksum verification failed for ${ASSET_PATH}" >&2
  exit 1
fi

cat > "${NOTES_PATH}" <<EOF_NOTES
# LogLine Runtime ${TAG}

This is a local build artifact from pinned LogLine Canon source.

## Source

- Repository: ${SOURCE_URL}
- Commit: ${COMMIT}
- Commit short: ${COMMIT_SHORT}
- Worktree dirty during pack build: ${WORKTREE_DIRTY}

## Binary

- Asset: ${ASSET_NAME}
- SHA-256: ${SHA256}
- Canon version: ${CANON_VERSION}
- Runtime: ${EXPECTED_RUNTIME}
- External effects in probe: false

## Probe

\`\`\`bash
./${ASSET_NAME} --version
./${ASSET_NAME} status
\`\`\`

## Downstream Usage

\`\`\`bash
export LOGLINE_RUNTIME_BIN=/absolute/path/to/${ASSET_NAME}
"\${LOGLINE_RUNTIME_BIN}" --version
"\${LOGLINE_RUNTIME_BIN}" status
\`\`\`

## Distribution Notes

- GitHub Actions is not required.
- npm is not required.
- Canon source is not vendored into downstream repositories.
- Downstream runtime hot paths must not run \`git pull\`.
- Download or provision this release asset outside the hot runtime path.
- This release does not include Minilab adapters, Gateway logic, Supabase
  persistence, receipt persistence runtime, or world-specific adapters.
EOF_NOTES

TAG_JSON="$(json_quote "${TAG}")"
ASSET_JSON="$(json_quote "${ASSET_PATH}")"
SHA_JSON="$(json_quote "${SHA_PATH}")"
MANIFEST_JSON="$(json_quote "${MANIFEST_PATH}")"
NOTES_JSON="$(json_quote "${NOTES_PATH}")"

cat <<EOF_SUMMARY

LogLine runtime release pack created.

Directory:
  ${DIST_DIR}

Asset:
  ${ASSET_PATH}

SHA-256:
  ${SHA256}

Manifest:
  ${MANIFEST_PATH}

Release notes:
  ${NOTES_PATH}

Suggested manual GitHub commands:

  git tag -a ${TAG_JSON} -m "LogLine runtime binary ${TAG}"
  git push origin main
  git push origin ${TAG_JSON}
  gh release create ${TAG_JSON} \\
    ${ASSET_JSON} \\
    ${SHA_JSON} \\
    ${MANIFEST_JSON} \\
    --title "LogLine Runtime ${TAG}" \\
    --notes-file ${NOTES_JSON} \\
    --repo ${REPO_SLUG}

EOF_SUMMARY

if [[ "${LOGLINE_RUNTIME_RELEASE_PUBLISH:-false}" == "true" ]]; then
  require_command gh
  gh release create "${TAG}" \
    "${ASSET_PATH}" \
    "${SHA_PATH}" \
    "${MANIFEST_PATH}" \
    --title "LogLine Runtime ${TAG}" \
    --notes-file "${NOTES_PATH}" \
    --repo "${REPO_SLUG}"
else
  echo "GitHub release publish skipped. Set LOGLINE_RUNTIME_RELEASE_PUBLISH=true to run gh release create."
fi
