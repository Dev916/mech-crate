#!/usr/bin/env bash
#
# Package a release tarball from already-built binaries.
#
# Usage:
#   scripts/package.sh <triple> <version> <bin-source-dir>
#
#   triple          target triple, e.g. universal-apple-darwin,
#                   x86_64-unknown-linux-musl, aarch64-unknown-linux-musl
#   version         version string without leading "v", e.g. 0.1.1
#   bin-source-dir  directory containing the built mx and mx-mcp
#
# Outputs:
#   dist/mx-v<version>-<triple>.tar.gz
#   dist/mx-v<version>-<triple>.tar.gz.sha256
#
# Tarball layout (extracted as mx-v<version>/):
#   bin/{mx,mx-mcp}              signed/notarized binaries (on macOS)
#   bin/lib/*.sh                 runtime bash helpers (sourced by mx-mcp)
#   templates/                   project templates copied by `mx init`
#   scripts/                     repo scripts (mech_crate_root() marker)
#   LICENSE-APACHE, LICENSE-MIT
#   VERSION

set -euo pipefail

triple="${1:?triple required (e.g. universal-apple-darwin)}"
version="${2:?version required (e.g. 0.1.1, no leading v)}"
bin_src="${3:?bin source directory required}"

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
stage_root="$repo_root/dist/stage"
stage="$stage_root/mx-v${version}"
out_dir="$repo_root/dist"
out_tar="$out_dir/mx-v${version}-${triple}.tar.gz"

# Sanity checks
for bin in mx mx-mcp; do
  if [[ ! -x "$bin_src/$bin" ]]; then
    echo "error: missing binary $bin_src/$bin" >&2
    exit 1
  fi
done
for asset in bin/lib templates scripts LICENSE-APACHE LICENSE-MIT; do
  if [[ ! -e "$repo_root/$asset" ]]; then
    echo "error: missing repo asset $asset" >&2
    exit 1
  fi
done

rm -rf "$stage"
mkdir -p "$stage/bin" "$out_dir"

# Binaries
for bin in mx mx-mcp; do
  install -m 0755 "$bin_src/$bin" "$stage/bin/$bin"
done

# Bundled assets
cp -R "$repo_root/bin/lib"   "$stage/bin/lib"
cp -R "$repo_root/templates" "$stage/templates"
cp -R "$repo_root/scripts"   "$stage/scripts"
cp    "$repo_root/LICENSE-APACHE" "$stage/LICENSE-APACHE"
cp    "$repo_root/LICENSE-MIT"    "$stage/LICENSE-MIT"
echo "$version" > "$stage/VERSION"

# Tar (deterministic-ish: sort + reproducible mtime)
tar \
  --format=ustar \
  -C "$stage_root" \
  -czf "$out_tar" \
  "mx-v${version}"

# Checksum
( cd "$out_dir" && shasum -a 256 "$(basename "$out_tar")" > "$(basename "$out_tar").sha256" )

echo "Built: $out_tar"
echo "       $out_tar.sha256"
