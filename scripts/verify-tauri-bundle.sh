#!/usr/bin/env bash
# verify-tauri-bundle.sh — prove a built SeatLoom.app actually CONTAINS its frontend.
#
# WHY THIS SCRIPT EXISTS. A `pnpm tauri build` can print `Bundling SeatLoom.app`,
# exit 0, and still produce an app that shows a white window. Tauri v2 embeds the
# frontend assets BROTLI-COMPRESSED INTO THE EXECUTABLE — it does not put them in
# Contents/Resources. So `ls Contents/Resources` shows only icon.icns even for a
# perfectly good app, and a check written against that directory reports a false
# failure. Conversely, `grep isComposing <binary>` finds nothing, because the bytes
# are compressed — that check reports a false PASS *and* a false FAIL depending on
# which way you squint at it.
#
# Both of those checks are wrong, and both have already misled someone
# (AEGIS-2026-10-26-tauri-bundle-missing-frontend-v1: a working app was reported
# broken, and a binary grep was recorded as "the criterion is broken").
#
# HOW THIS ONE WORKS, and why it is a statement about content:
#   1. Decompress every blob tauri-codegen wrote under target/*/build/*/out/.
#   2. For every file in ui/dist, find the blob that decompresses back to it — a
#      content match, which is stronger than the digest-name shortcut (the names are
#      blake3, which is not installed here, and a name match alone would not prove
#      the blob holds this file's bytes).
#   3. Check that blob's COMPRESSED bytes appear verbatim in the app executable.
#      tauri emits `include_bytes!(compressed)`, so that is direct evidence THIS app
#      carries THIS asset.
#   4. Report which strings prove what. `isComposing` and `proposeDimensions` come
#      from @xterm/addon-fit (a ui/package.json dependency) — they prove xterm is
#      bundled, NOT that SeatLoom's code shipped. SeatLoom's own strings do.
#   5. Check the HTML for references to files ui/dist does not contain (a 404 on
#      every launch, which is how the missing favicon was found).
#
# USAGE
#   scripts/verify-tauri-bundle.sh [path-to-SeatLoom.app] [path-to-ui/dist]
#
# Exit codes: 0 = the frontend is genuinely embedded; 1 = it is not.
set -uo pipefail

APP="${1:-target/release/bundle/macos/SeatLoom.app}"
DIST="${2:-ui/dist}"

command -v brotli >/dev/null || { echo "FAIL: brotli not on PATH (brew install brotli)" >&2; exit 1; }
command -v python3 >/dev/null || { echo "FAIL: python3 not on PATH" >&2; exit 1; }
[ -d "$APP" ] || { echo "FAIL: no such app bundle: $APP" >&2; exit 1; }
[ -d "$DIST" ] || { echo "FAIL: no such frontend dist: $DIST" >&2; exit 1; }

python3 - "$APP" "$DIST" <<'PY'
import glob, os, subprocess, sys

app, dist = sys.argv[1], sys.argv[2]
binpath = os.path.join(app, 'Contents/MacOS/seatloom-tauri')

# Where to look for the blobs. The natural home is the repo that owns the dist
# being verified (<dist>/../../target), because that is where the build that
# produced it wrote them; fall back to the cwd's target/ for the common case of
# verifying the build you just made. Searching only one of them silently reports
# a healthy app as broken when you point the script at another checkout.
roots = [os.path.abspath(os.path.join(dist, '..', '..', 'target')), os.path.abspath('target')]
roots = list(dict.fromkeys(r for r in roots if os.path.isdir(r)))

def die(msg):
    print(f"FAIL: {msg}", file=sys.stderr); sys.exit(1)

if not os.path.isfile(binpath):
    die(f"no executable at {binpath} (is this a macOS SeatLoom.app?)")

print("=== 1. what the bundle directory actually holds ===")
res = os.path.join(app, 'Contents/Resources')
listing = sorted(os.listdir(res)) if os.path.isdir(res) else []
print(f"  Contents/Resources: {' '.join(listing) if listing else '(absent)'}")
print("  (Tauri v2 embeds assets in the executable; an empty Resources is NOT a defect)")

print("=== 2. decompress every blob tauri-codegen wrote ===")
blobs = []  # (path, compressed_bytes, decompressed_bytes)
dirs = []
for root in roots:
    for d in glob.glob(os.path.join(root, '**', 'tauri-codegen-assets'), recursive=True):
        dirs.append(d)
        for f in sorted(glob.glob(os.path.join(d, '*'))):
            if not os.path.isfile(f):
                continue
            comp = open(f, 'rb').read()
            r = subprocess.run(['brotli', '-d', '-c', f], capture_output=True)
            if r.returncode == 0 and r.stdout:
                blobs.append((f, comp, r.stdout))
print(f"  searched: {', '.join(roots)}")
print(f"  {len(blobs)} blobs decompressed from {len(dirs)} build director(ies)")
if not blobs:
    die("no tauri-codegen-assets blobs found under target/ — run a build first")

binary = open(binpath, 'rb').read()
print(f"  executable: {len(binary)} bytes")

print("=== 3. every ui/dist file: is its content embedded in THIS executable? ===")
missing, total = [], 0
for src in sorted(glob.glob(os.path.join(dist, '**', '*'), recursive=True)):
    if not os.path.isfile(src):
        continue
    total += 1
    rel = os.path.relpath(src, dist)
    content = open(src, 'rb').read()
    # content match first: which blob decompresses to exactly this file
    match = [(p, c) for p, c, out in blobs if out == content]
    if not match:
        print(f"  NOT EMBEDDED: {rel} — no blob in any build holds its content")
        missing.append(rel); continue
    # the blob's compressed bytes must be inside THIS executable
    in_bin = [p for p, c in match if c in binary]
    if not in_bin:
        print(f"  STALE: {rel} — embedded in some build, but NOT in this executable (older app)")
        missing.append(rel); continue
    print(f"  OK: {rel} ({len(content)} bytes; compressed bytes present in the executable)")

print("=== 4. which strings prove what ===")
# The largest embedded JS blob is the app bundle. Read it back out of the binary so
# the check is about what the APP carries, not about what is on disk.
main = None
for p, comp, out in blobs:
    if p.endswith('.js') and comp in binary and (main is None or len(out) > len(main)):
        main = out
if main:
    for s, who in (('isComposing', '@xterm/addon-fit — NOT SeatLoom'),
                   ('proposeDimensions', '@xterm/addon-fit — NOT SeatLoom')):
        print(f"  {s:20} x{main.count(s.encode()):<3} <- {who}")
    print(f"  {'等你裁定':20} x{main.count('等你裁定'.encode()):<3} <- SeatLoom's own 020 code; THIS is the one that proves it shipped")
else:
    print("  (no JS bundle found inside this executable)")

print("=== 5. dangling references in the HTML ===")
dangling = 0
for h in ('index.html', 'supervisor.html'):
    p = os.path.join(dist, h)
    if not os.path.isfile(p):
        continue
    import re
    for ref in sorted(set(re.findall(rb'(?:src|href)="(/[^"]*)"', open(p, 'rb').read()))):
        ref = ref.decode()
        if ref.endswith(('.js', '.css')):
            continue  # hashed build outputs; covered by step 3
        if not os.path.exists(os.path.join(dist, ref.lstrip('/'))):
            print(f"  DANGLING: {h} references {ref}, which ui/dist does not contain (404 on every launch)")
            dangling += 1
if dangling == 0:
    print("  none")

print("=== 6. verdict ===")
if missing:
    die(f"{len(missing)} of {total} frontend files are not embedded in THIS executable "
        f"({', '.join(missing)}) — the built app would show an empty window")
print(f"PASS: all {total} frontend files are embedded in this executable, verified byte-for-byte.")
print("")
print("Next step (the only one that proves a human sees it): open the app and look.")
print(f"  open {app}")
print("Do NOT substitute a directory listing or a binary grep for that step — neither")
print("can tell a compressed asset from an absent one.")
PY
