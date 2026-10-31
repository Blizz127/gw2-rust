#!/bin/bash
# Freeze a received GW2 client copy. Run ONLY after Devbox says the copy is complete
# and stable (no rsync still writing).
#
#   tools/freeze-client.sh <name> [src]
#     name  frozen directory name, e.g. build-209026 or steam-23352508
#     src   default /home/blizz/gw2-client/incoming (Devbox names which folder;
#           expected /home/blizz/gw2-client/arenanet-download)
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
# Any process (installer, wine, rsync...) with an open file or cwd under src blocks the freeze.
# Only processes visible to this user are seen; root-owned (e.g. docker) ones are not.
real_src="$(realpath "$src")"
busy="$( (find /proc/[0-9]*/cwd /proc/[0-9]*/fd -maxdepth 1 \( -lname "$real_src" -o -lname "$real_src/*" \) 2>/dev/null || true) |
        cut -d/ -f3 | sort -u | while read -r pid; do printf '%s:%s ' "$pid" "$(cat /proc/$pid/comm 2>/dev/null)"; done)"
[[ -z "$busy" ]] || { echo "processes still using $src: $busy; refusing"; exit 2; }

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
