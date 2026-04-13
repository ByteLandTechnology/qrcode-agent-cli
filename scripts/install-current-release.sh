#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
CONFIG_PATH="${REPO_ROOT}/release/skill-release.config.json"

if [[ ! -f "${CONFIG_PATH}" ]]; then
  echo "Missing ${CONFIG_PATH}. The release asset pack must be configured before install." >&2
  exit 1
fi

if ! command -v node >/dev/null 2>&1; then
  echo "Node.js is required to read ${CONFIG_PATH}." >&2
  exit 1
fi

mapfile -t RELEASE_INFO < <(
  node --input-type=module -e '
    import fs from "node:fs";
    const config = JSON.parse(fs.readFileSync(process.argv[1], "utf8"));
    const repo =
      process.env.GITHUB_REPOSITORY ||
      config.githubRelease?.ownerRepository ||
      config.sourceRepository ||
      "";
    console.log(config.sourceSkillId);
    console.log(repo);
  ' "${CONFIG_PATH}"
)

SKILL_NAME="${RELEASE_INFO[0]:-}"
OWNER_REPOSITORY="${RELEASE_INFO[1]:-}"
VERSION="${1:-}"
INSTALL_DIR="${INSTALL_DIR:-${REPO_ROOT}/.local/bin}"
PLATFORM="$(uname -s)"
ARCH="$(uname -m)"

if [[ -z "${SKILL_NAME}" || -z "${OWNER_REPOSITORY}" || "${OWNER_REPOSITORY}" == *"REPLACE_WITH_OWNER/REPO"* ]]; then
  echo "release/skill-release.config.json must define the repository owner/name and skill id before install." >&2
  exit 1
fi

if [[ -z "${VERSION}" ]] && command -v git >/dev/null 2>&1; then
  VERSION="$(git -C "${REPO_ROOT}" describe --tags --exact-match 2>/dev/null || true)"
  VERSION="${VERSION#v}"
fi

if [[ -z "${VERSION}" ]]; then
  echo "Unable to determine release version. Check out a released tag or pass the version explicitly." >&2
  exit 1
fi

case "${PLATFORM}:${ARCH}" in
  Linux:x86_64)
    TARGET="x86_64-unknown-linux-gnu"
    ;;
  Darwin:arm64)
    TARGET="aarch64-apple-darwin"
    ;;
  *)
    echo "Unsupported platform ${PLATFORM}:${ARCH}. See the repo release notes for supported targets." >&2
    exit 1
    ;;
esac

ARCHIVE_NAME="${SKILL_NAME}-${VERSION}-${TARGET}.tar.gz"
RELEASE_URL="https://github.com/${OWNER_REPOSITORY}/releases/tag/v${VERSION}"
DOWNLOAD_URL="${RELEASE_URL}/download/${ARCHIVE_NAME}"
TMP_DIR="$(mktemp -d)"
ARCHIVE_PATH="${TMP_DIR}/${ARCHIVE_NAME}"

cleanup() {
  rm -rf "${TMP_DIR}"
}
trap cleanup EXIT

mkdir -p "${INSTALL_DIR}"

echo "Downloading ${DOWNLOAD_URL}"
curl --fail --location --silent --show-error "${DOWNLOAD_URL}" -o "${ARCHIVE_PATH}"
tar -xzf "${ARCHIVE_PATH}" -C "${TMP_DIR}"
install -m 0755 "${TMP_DIR}/${SKILL_NAME}" "${INSTALL_DIR}/${SKILL_NAME}"

echo "Installed ${SKILL_NAME} ${VERSION} to ${INSTALL_DIR}/${SKILL_NAME}"
echo "Verify with: ${INSTALL_DIR}/${SKILL_NAME} --version"
