#!/bin/sh
#
# mx installer — https://mechcrate.dev/install.sh
#
#   curl -fsSL https://mechcrate.dev/install.sh | sh
#
# Downloads the latest published mx release for this machine, verifies its
# sha256, extracts it, and hands the extracted bundle to the bundled binary:
#
#   mx self-update --from-dir <bundle> --yes
#
# which is the single writer of the install layout (~/.mech-crate/releases,
# the ~/.mech-crate/current symlink, and the ~/.local/bin shims). The script
# never uses sudo and never writes outside a temp dir and your home.
#
# Environment:
#   MX_VERSION       install this version instead of the latest (e.g. 0.1.2)
#   MX_RELEASES_API  GitHub API base for the release channel (tests/mirrors)
#
# Contributors building from a checkout: `make install-local` instead.
#
# Style: docs/development/SHELL_SCRIPTING_GUIDE.md — POSIX sh, set -eu, pure
# helpers separated from the effectful main, every failure explicit.

set -eu

RELEASES_API="${MX_RELEASES_API:-https://api.github.com/repos/unyform-ai/mech-crate-releases}"

# ── pure helpers ──────────────────────────────────────────────────────────

# Map uname output to a published target triple, or print nothing.
triple_for() {
    os="$1"
    arch="$2"
    case "$os" in
        Darwin) printf 'universal-apple-darwin' ;;
        Linux)
            case "$arch" in
                x86_64|amd64) printf 'x86_64-unknown-linux-musl' ;;
                aarch64|arm64) printf 'aarch64-unknown-linux-musl' ;;
                *) printf '' ;;
            esac
            ;;
        *) printf '' ;;
    esac
}

# Pull "tag_name": "vX.Y.Z" out of a GitHub release JSON body (no jq).
tag_from_json() {
    sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -n 1
}

# First 64-hex token of a .sha256 sidecar.
digest_from_sidecar() {
    tr -d '\r' | awk 'NF { print tolower($1); exit }' | grep -E '^[0-9a-f]{64}$' || true
}

# ── effectful helpers ─────────────────────────────────────────────────────

log()  { printf '  \033[36m→\033[0m %s\n' "$*"; }
ok()   { printf '  \033[32m✓\033[0m %s\n' "$*"; }
die()  { printf '  \033[31m✗\033[0m %s\n' "$*" >&2; exit "${2:-1}"; }

fetch() {
    # fetch <url> <dest>   (curl preferred, wget fallback)
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL --retry 3 -o "$2" "$1"
    elif command -v wget >/dev/null 2>&1; then
        wget -q -O "$2" "$1"
    else
        die "need curl or wget to download mx"
    fi
}

sha256_of() {
    if command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$1" | awk '{ print tolower($1) }'
    elif command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | awk '{ print tolower($1) }'
    else
        die "need shasum or sha256sum to verify the download"
    fi
}

# ── main ──────────────────────────────────────────────────────────────────

main() {
    printf '\n  \033[36m🦝 mx installer\033[0m\n\n'

    triple="$(triple_for "$(uname -s)" "$(uname -m)")"
    [ -n "$triple" ] || die "no mx release is published for $(uname -s)/$(uname -m); build from source: https://github.com/Dev916/mech-crate" 2

    tmp="$(mktemp -d)"
    trap 'rm -rf "$tmp"' EXIT INT TERM

    if [ -n "${MX_VERSION:-}" ]; then
        version="${MX_VERSION#v}"
        release_url="$RELEASES_API/releases/tags/v$version"
    else
        release_url="$RELEASES_API/releases/latest"
    fi
    log "Resolving release from $release_url"
    fetch "$release_url" "$tmp/release.json" || die "could not reach the release channel ($release_url)"
    tag="$(tag_from_json < "$tmp/release.json")"
    [ -n "$tag" ] || die "release channel returned no tag_name (is the release published?)"
    version="${tag#v}"
    ok "mx $version ($triple)"

    asset="mx-v$version-$triple.tar.gz"
    base="$(sed -n 's/.*"browser_download_url"[[:space:]]*:[[:space:]]*"\([^"]*\/\)'"$asset"'".*/\1/p' "$tmp/release.json" | head -n 1)"
    [ -n "$base" ] || die "release $tag has no asset $asset"

    log "Downloading $asset"
    fetch "$base$asset" "$tmp/$asset" || die "download failed: $base$asset"
    fetch "$base$asset.sha256" "$tmp/$asset.sha256" || die "download failed: $base$asset.sha256"

    expected="$(digest_from_sidecar < "$tmp/$asset.sha256")"
    [ -n "$expected" ] || die "$asset.sha256 does not contain a sha256 digest"
    actual="$(sha256_of "$tmp/$asset")"
    if [ "$expected" != "$actual" ]; then
        die "checksum mismatch for $asset: expected $expected, got $actual"
    fi
    ok "sha256 verified"

    log "Extracting"
    tar -xzf "$tmp/$asset" -C "$tmp"
    bundle="$tmp/mx-v$version"
    [ -x "$bundle/bin/mx" ] || die "$asset did not contain mx-v$version/bin/mx"

    log "Installing via the bundled mx"
    "$bundle/bin/mx" self-update --from-dir "$bundle" --yes

    printf '\n'
}

main "$@"
