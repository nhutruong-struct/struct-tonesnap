#!/usr/bin/env bash
# Signs, notarizes and packages ToneSnap.app for a macOS release. It runs nothing but Apple's command
# line tools, so the secrets it sees never reach a build script or a dependency.
#
#   scripts/release/sign-macos.sh <ToneSnap.app> <version> <output dir>
#
# It writes ToneSnap_<version>_aarch64.dmg and ToneSnap_<version>_aarch64.app.tar.gz (the file the in-app
# updater downloads) to the output directory.
#
# Environment:
#   APPLE_SIGNING_IDENTITY       The certificate name, e.g. "Developer ID Application: Name (TEAMID)". Required.
#   APPLE_CERTIFICATE            The certificate with its key as a base64 .p12. When set (CI), it is imported
#   APPLE_CERTIFICATE_PASSWORD   into a temporary keychain that is deleted on exit; otherwise the identity must
#                                already be in a keychain (a developer's own Mac).
#   NOTARY_PROFILE               A `notarytool store-credentials` profile (a developer's own Mac), or
#   APPLE_API_KEY_ID             an App Store Connect API key: its id, its issuer and the contents of its
#   APPLE_API_ISSUER_ID          .p8 file (CI).
#   APPLE_API_KEY_P8
#   SKIP_NOTARIZE=1              Sign and package only: a trial with a development certificate.

set -euo pipefail

if [ "$#" -ne 3 ]; then
  echo "usage: sign-macos.sh <ToneSnap.app> <version> <output dir>" >&2
  exit 2
fi
APP="$1"
VERSION="$2"
OUT="$3"
: "${APPLE_SIGNING_IDENTITY:?APPLE_SIGNING_IDENTITY is not set}"
[ -d "$APP" ] || { echo "$APP is not an app bundle" >&2; exit 1; }

WORK="$(mktemp -d)"
KEYCHAIN=""
cleanup() {
  if [ -n "$KEYCHAIN" ]; then security delete-keychain "$KEYCHAIN" >/dev/null 2>&1 || true; fi
  rm -rf "$WORK"
}
trap cleanup EXIT

mkdir -p "$OUT"
OUT="$(cd "$OUT" && pwd)"

# A temporary keychain for the certificate, so a CI runner's own keychains stay untouched.
if [ -n "${APPLE_CERTIFICATE:-}" ]; then
  KEYCHAIN="$WORK/tonesnap-signing.keychain-db"
  KEYCHAIN_PASSWORD="$(uuidgen)"
  printf '%s' "$APPLE_CERTIFICATE" | base64 --decode > "$WORK/certificate.p12"
  security create-keychain -p "$KEYCHAIN_PASSWORD" "$KEYCHAIN"
  security set-keychain-settings -lut 3600 "$KEYCHAIN"
  security unlock-keychain -p "$KEYCHAIN_PASSWORD" "$KEYCHAIN"
  security import "$WORK/certificate.p12" -k "$KEYCHAIN" -P "${APPLE_CERTIFICATE_PASSWORD:-}" -T /usr/bin/codesign
  security set-key-partition-list -S apple-tool:,apple: -s -k "$KEYCHAIN_PASSWORD" "$KEYCHAIN" >/dev/null
  # shellcheck disable=SC2046 -- the list holds quoted paths without spaces
  security list-keychains -d user -s "$KEYCHAIN" $(security list-keychains -d user | tr -d '"')
  rm -f "$WORK/certificate.p12"
fi

# Submits a file to Apple's notary service and waits; fails with Apple's log unless it is accepted.
notarize() {
  local file="$1" result="$WORK/notary.json"
  local -a auth
  if [ -n "${NOTARY_PROFILE:-}" ]; then
    auth=(--keychain-profile "$NOTARY_PROFILE")
  else
    : "${APPLE_API_KEY_ID:?set NOTARY_PROFILE or the APPLE_API_* variables}"
    : "${APPLE_API_ISSUER_ID:?APPLE_API_ISSUER_ID is not set}"
    : "${APPLE_API_KEY_P8:?APPLE_API_KEY_P8 is not set}"
    printf '%s' "$APPLE_API_KEY_P8" > "$WORK/key.p8"
    chmod 600 "$WORK/key.p8"
    auth=(--key "$WORK/key.p8" --key-id "$APPLE_API_KEY_ID" --issuer "$APPLE_API_ISSUER_ID")
  fi
  xcrun notarytool submit "$file" "${auth[@]}" --wait --output-format json > "$result"
  if ! grep -q '"status" *: *"Accepted"' "$result"; then
    cat "$result" >&2
    id="$(sed -n 's/.*"id" *: *"\([^"]*\)".*/\1/p' "$result" | head -n 1)"
    [ -n "$id" ] && xcrun notarytool log "$id" "${auth[@]}" >&2 || true
    echo "notarization of $(basename "$file") was not accepted" >&2
    exit 1
  fi
}

echo "Signing the app"
# The hardened runtime is required for notarization; the app needs no entitlements.
codesign --force --options runtime --timestamp --sign "$APPLE_SIGNING_IDENTITY" "$APP"
codesign --verify --strict --verbose=2 "$APP"

if [ "${SKIP_NOTARIZE:-}" != "1" ]; then
  echo "Notarizing the app"
  ditto -c -k --keepParent "$APP" "$WORK/ToneSnap.zip"
  notarize "$WORK/ToneSnap.zip"
  xcrun stapler staple "$APP"
  spctl --assess --type exec --verbose=2 "$APP"
fi

echo "Packaging the updater archive"
# COPYFILE_DISABLE keeps extended attributes out of the archive as `._` files.
COPYFILE_DISABLE=1 tar -czf "$OUT/ToneSnap_${VERSION}_aarch64.app.tar.gz" -C "$(dirname "$APP")" "$(basename "$APP")"

echo "Building the disk image"
STAGE="$WORK/dmg"
mkdir -p "$STAGE"
ditto "$APP" "$STAGE/ToneSnap.app"
ln -s /Applications "$STAGE/Applications"
DMG="$OUT/ToneSnap_${VERSION}_aarch64.dmg"
rm -f "$DMG"
hdiutil create -volname "ToneSnap" -srcfolder "$STAGE" -ov -format UDZO "$DMG" >/dev/null
codesign --force --timestamp --sign "$APPLE_SIGNING_IDENTITY" "$DMG"
if [ "${SKIP_NOTARIZE:-}" != "1" ]; then
  echo "Notarizing the disk image"
  notarize "$DMG"
  xcrun stapler staple "$DMG"
  spctl --assess --type open --context context:primary-signature --verbose=2 "$DMG"
fi

ls -l "$OUT"
