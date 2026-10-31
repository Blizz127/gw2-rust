#!/usr/bin/env python3
"""Match Gw2.dat mapc files to web-API maps by rectangle.

usage: match-map-rects.py <mapc-rects.tsv> <data/api/<build>/maps.json> [out.tsv]

The rects TSV comes from `gw2-dat-scan --map-rects` (MapParam.rect). A file
matches an API map when its rect equals the map's `map_rect` exactly. Several API
maps can share one file (instances and story copies), so every match is kept.
"""
import json
import sys
from collections import defaultdict

rects_path, maps_path = sys.argv[1], sys.argv[2]
out_path = sys.argv[3] if len(sys.argv) > 3 else None

files = defaultdict(list)
for line in open(rects_path):
    fid, *r = line.split("\t")
    files[tuple(round(float(x)) for x in r)].append(int(fid))

rows, unmatched = [], []
for m in json.load(open(maps_path)):
    mr = m.get("map_rect")
    if not mr:
        continue
    key = (round(mr[0][0]), round(mr[0][1]), round(mr[1][0]), round(mr[1][1]))
    hits = files.get(key, [])
    if hits:
        for fid in hits:
            rows.append((m["id"], m["name"], m.get("type", ""), fid, len(hits)))
    else:
        unmatched.append(m["id"])

lines = ["api_map_id\tname\ttype\tmapc_file_id\tcandidates"] + [
    f"{a}\t{n}\t{t}\t{f}\t{c}" for a, n, t, f, c in sorted(rows)
]
if out_path:
    open(out_path, "w").write("\n".join(lines) + "\n")
unique = sum(1 for r in rows if r[4] == 1)
print(f"api maps with rect matches: {len({r[0] for r in rows})}, unique-file matches: {unique}, unmatched api maps: {len(unmatched)}")
for r in rows:
    if r[1] in ("Queensdale", "Divinity's Reach", "Kessex Hills", "Shaemoor"):
        print("\t".join(map(str, r)))
