#!/bin/sh
set -eu

REPOSITORY="git-ksk/reasoning-harness"
DEFAULT_VERSION="0.4.2"
VERSION="$DEFAULT_VERSION"
BIN_DIR=""

usage() {
  cat <<'USAGE'
Install the native Reason CLI from a published GitHub Release.

Usage:
  install.sh [--version VERSION|TAG] [--bin-dir DIR]

Options:
  --version VERSION|TAG  CLI version or release tag (default: 0.4.2)
                         Examples: 0.4.2, v0.4.2, 0.5.0, reason-v0.5.0
  --bin-dir DIR          Install directory (default: $HOME/.local/bin)
  -h, --help             Show this help.

The installer downloads the platform archive and SHA256SUMS from the same
GitHub Release, verifies the archive checksum, verifies `reason --version`,
and only then replaces the destination binary.
USAGE
}

fail() {
  printf 'reason installer: %s\n' "$*" >&2
  exit 1
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --version)
      [ "$#" -ge 2 ] || fail "--version requires a value"
      VERSION="$2"
      shift 2
      ;;
    --bin-dir)
      [ "$#" -ge 2 ] || fail "--bin-dir requires a value"
      BIN_DIR="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      fail "unknown argument: $1"
      ;;
  esac
done

if [ -z "$BIN_DIR" ]; then
  [ -n "${HOME:-}" ] || fail "HOME is not set; pass --bin-dir explicitly"
  BIN_DIR="$HOME/.local/bin"
fi

case "$VERSION" in
  reason-v*)
    TAG="$VERSION"
    VERSION=${VERSION#reason-v}
    ;;
  v*)
    TAG="$VERSION"
    VERSION=${VERSION#v}
    ;;
  [0-9]*.[0-9]*.[0-9]*)
    MAJOR=${VERSION%%.*}
    REST=${VERSION#*.}
    MINOR=${REST%%.*}
    case "$MAJOR:$MINOR" in
      0:0|0:1|0:2|0:3|0:4) TAG="v$VERSION" ;;
      *) TAG="reason-v$VERSION" ;;
    esac
    ;;
  *)
    fail "invalid version/tag: $VERSION"
    ;;
esac

printf '%s\n' "$VERSION" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+([+-][0-9A-Za-z.-]+)?$' \
  || fail "invalid semantic version: $VERSION"

command -v curl >/dev/null 2>&1 || fail "curl is required"
command -v tar >/dev/null 2>&1 || fail "tar is required"

OS=$(uname -s 2>/dev/null || true)
ARCH=$(uname -m 2>/dev/null || true)
case "$OS:$ARCH" in
  Linux:x86_64|Linux:amd64)
    ASSET="linux-x86_64"
    ;;
  Darwin:arm64|Darwin:aarch64)
    ASSET="macos-aarch64"
    ;;
  Darwin:x86_64|Darwin:amd64)
    ASSET="macos-x86_64"
    ;;
  *)
    fail "unsupported platform: ${OS:-unknown}/${ARCH:-unknown}; supported: Linux x86_64, macOS arm64, macOS x86_64. On Windows use install.ps1"
    ;;
esac

ARCHIVE="reason-v${VERSION}-${ASSET}.tar.gz"
BASE_URL="https://github.com/${REPOSITORY}/releases/download/${TAG}"

TMP_DIR=$(mktemp -d 2>/dev/null || mktemp -d -t reason-install)
STAGED=""
cleanup() {
  [ -z "$STAGED" ] || rm -f "$STAGED"
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT HUP INT TERM

printf 'Installing Reason CLI %s for %s...\n' "$VERSION" "$ASSET"
curl -fL --retry 3 --retry-delay 1 -o "$TMP_DIR/$ARCHIVE" "$BASE_URL/$ARCHIVE" \
  || fail "failed to download $ARCHIVE from $TAG"
curl -fL --retry 3 --retry-delay 1 -o "$TMP_DIR/SHA256SUMS" "$BASE_URL/SHA256SUMS" \
  || fail "failed to download SHA256SUMS from $TAG"

EXPECTED=$(awk -v name="$ARCHIVE" '$2 == name || $2 == "*" name { print $1 }' "$TMP_DIR/SHA256SUMS")
[ -n "$EXPECTED" ] || fail "$ARCHIVE is missing from SHA256SUMS"
case "$EXPECTED" in
  *[!0-9a-fA-F]*|'') fail "invalid SHA-256 entry for $ARCHIVE" ;;
esac
[ "${#EXPECTED}" -eq 64 ] || fail "invalid SHA-256 length for $ARCHIVE"

if command -v sha256sum >/dev/null 2>&1; then
  ACTUAL=$(sha256sum "$TMP_DIR/$ARCHIVE" | awk '{print $1}')
elif command -v shasum >/dev/null 2>&1; then
  ACTUAL=$(shasum -a 256 "$TMP_DIR/$ARCHIVE" | awk '{print $1}')
else
  fail "sha256sum or shasum is required to verify the release archive"
fi

[ "$ACTUAL" = "$EXPECTED" ] || fail "checksum mismatch for $ARCHIVE; refusing to install"

tar -tzf "$TMP_DIR/$ARCHIVE" >/dev/null || fail "release archive is not a valid tar.gz"
case "$(tar -tzf "$TMP_DIR/$ARCHIVE" | grep -E '(^/|(^|/)\.\.(/|$))' || true)" in
  '') ;;
  *) fail "release archive contains an unsafe path" ;;
esac

tar -xzf "$TMP_DIR/$ARCHIVE" -C "$TMP_DIR"
SOURCE="$TMP_DIR/reason-v${VERSION}-${ASSET}/reason"
[ -f "$SOURCE" ] || fail "release archive does not contain the expected reason binary"
[ ! -L "$SOURCE" ] || fail "release archive reason binary must not be a symlink"
chmod +x "$SOURCE"

VERSION_OUTPUT=$($SOURCE --version 2>/dev/null || true)
[ "$VERSION_OUTPUT" = "reason $VERSION" ] \
  || fail "downloaded binary reported '$VERSION_OUTPUT', expected 'reason $VERSION'"

mkdir -p "$BIN_DIR" || fail "cannot create install directory: $BIN_DIR"
STAGED=$(mktemp "$BIN_DIR/.reason.install.XXXXXX") || fail "cannot stage binary in $BIN_DIR"
cp "$SOURCE" "$STAGED"
chmod 755 "$STAGED"
mv -f "$STAGED" "$BIN_DIR/reason"
STAGED=""

printf 'Installed %s\n' "$BIN_DIR/reason"
if "$BIN_DIR/reason" --version >/dev/null 2>&1; then
  "$BIN_DIR/reason" --version
else
  fail "installed binary failed its version smoke check"
fi

case ":${PATH:-}:" in
  *":$BIN_DIR:"*) ;;
  *)
    printf '\nAdd this directory to PATH to run reason from anywhere:\n  %s\n' "$BIN_DIR"
    ;;
esac

printf '\nIntegrity: SHA-256 verified against the published release checksum.\n'
