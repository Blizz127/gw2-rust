#!/bin/bash
# Freeze a received GW2 client copy. Run ONLY after Devbox says the copy is complete
# and stable (no rsync still writing).
#
#   tools/freeze-client.sh <name> [src]
#     name  frozen directory name, e.g. steam-23352508 (the game build id is
#           recorded separately once REA/the client reports it)
#     src   default /home/blizz/gw2-client/incoming
#
# It moves src (same filesystem, no copy) to /home/blizz/gw2-client/<name>, makes it
# read-only, and writes <name>.manifest.txt next to it (SHA256 of Gw2-64.exe, Gw2.dat
# and every other file). It never deletes anything and never runs the client.
set -euo pipefail
ROOT="${GW2_CLIENT_ROOT:-/home/blizz/gw2-client}"
name="${1:?usage: freeze-client.sh <name> [src]}"
src="${2:-$ROOT/incoming}"
dst="$ROOT/$name"
manifest="$ROOT/$name.manifest.txt"

[[ "$name" =~ ^[A-Za-z0-9._-]+$ ]] || { echo "bad name $name"; exit 2; }
[[ -d "$src" ]] || { echo "no source dir $src"; exit 2; }
[[ ! -e "$dst" ]] || { echo "$dst already exists; refusing to overwrite"; exit 2; }
[[ -f "$src/Gw2-64.exe" && -f "$src/Gw2.dat" ]] || { echo "missing Gw2-64.exe or Gw2.dat in $src"; exit 2; }
if pgrep -af rsync | grep -F -- "$src" >/dev/null; then
  echo "an rsync touching $src is still running; refusing"; exit 2
fi
for f in "$src"/Gw2.dat "$src"/Gw2-64.exe; do
  if command -v fuser >/dev/null && fuser "$f" >/dev/null 2>&1; then
    echo "$f is open by a process; refusing"; exit 2
  fi
done

mv -T "$src" "$dst"
chmod -R a-w "$dst"

{
  echo "# GW2 client freeze: $name"
  echo "frozen_at: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "path: $dst"
  echo "total_bytes: $(du -sb "$dst" | cut -f1)"
  echo
  echo "## key files"
  (cd "$dst" && sha256sum Gw2-64.exe Gw2.dat)
  echo
  echo "## all files (sha256, bytes, path)"
  (cd "$dst" && find . -type f ! -name Gw2.dat -print0 | sort -z |
     while IFS= read -r -d '' f; do
       printf '%s  %s  %s\n' "$(sha256sum "$f" | cut -d' ' -f1)" "$(stat -c %s "$f")" "$f"
     done)
} > "$manifest"
chmod a-w "$manifest"
sed -n '1,12p' "$manifest"
echo "manifest: $manifest"
