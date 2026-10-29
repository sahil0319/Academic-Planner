#!/usr/bin/env bash
# Writes the updater manifest (latest.json) for a release to stdout (docs/spec.md §13.1, R27,
# R28; specs/007-m7-updater/contracts/release-workflow.md).
#
# Usage: scripts/updater-manifest.sh <tag> <dir> <notes-file>
#   <tag>         release tag, e.g. v0.2.0 (the version is the tag without the leading "v")
#   <dir>         folder holding the signed updater artifacts and their .sig files
#   <notes-file>  release notes shown in the app's install confirmation
#
# The .sig text is written without trailing whitespace: the updater decodes it as strict
# base64, where a trailing newline would make every signature check fail (R28).
#
# Every platform needs exactly one artifact and its .sig file in <dir>; otherwise the script
# exits 1 so the release is not published with a manifest that cannot be verified.
set -euo pipefail

if [ "$#" -ne 3 ]; then
  echo "usage: $0 <tag> <dir> <notes-file>" >&2
  exit 1
fi

tag=$1
dir=$2
notes_file=$3
repo_url="https://github.com/sahil0319/Academic-Planner"

if [ ! -d "$dir" ]; then
  echo "error: $dir is not a folder" >&2
  exit 1
fi
if [ ! -f "$notes_file" ]; then
  echo "error: release notes $notes_file not found" >&2
  exit 1
fi

# Prints the single file in $dir matching the glob suffix $1, or fails with a message.
find_one() {
  local pattern=$1
  local matches=()
  local f
  shopt -s nullglob
  for f in "$dir"/*"$pattern"; do
    matches+=("$f")
  done
  shopt -u nullglob
  if [ "${#matches[@]}" -ne 1 ]; then
    echo "error: expected exactly one *$pattern in $dir, found ${#matches[@]}" >&2
    return 1
  fi
  printf '%s\n' "${matches[0]}"
}

# Prints the JSON platform entry for the artifact matching $1 with updater format $2.
platform() {
  local pattern=$1 format=$2 file name
  file=$(find_one "$pattern") || return 1
  if [ ! -f "$file.sig" ]; then
    echo "error: missing signature $file.sig" >&2
    return 1
  fi
  name=$(basename "$file")
  jq -n \
    --arg url "$repo_url/releases/download/$tag/$name" \
    --rawfile signature "$file.sig" \
    --arg format "$format" \
    '{url: $url, signature: ($signature | sub("\\s+$"; "")), format: $format}'
}

macos=$(platform ".app.tar.gz" app)
windows=$(platform "-setup.exe" nsis)
linux=$(platform ".AppImage" appimage)

jq -n \
  --arg version "${tag#v}" \
  --rawfile notes "$notes_file" \
  --arg pub_date "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --argjson macos "$macos" \
  --argjson windows "$windows" \
  --argjson linux "$linux" \
  '{
    version: $version,
    notes: $notes,
    pub_date: $pub_date,
    platforms: {
      "macos-aarch64": $macos,
      "windows-x86_64": $windows,
      "linux-x86_64": $linux
    }
  }'
