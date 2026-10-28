#!/usr/bin/env bash
# configure-wash-provider.sh — register the data-washing pi providers on THIS machine.
#
# Why this is a script and not an inline command: the action's shape (read a secret
# file -> write it into a config) is exactly what the auto-mode permission classifier
# blocks, and rightly so in general. Narrowing the escape hatch to one auditable
# script means .claude/settings.local.json can allow this single command instead of a
# broad `Bash(python3 *)` pattern that would pre-approve anything.
#
# What it does: deep-copies the existing `stepplan` / `stepplan-anthropic` provider
# entries in ~/.pi/agent/models.json and re-registers them under `stepplan-wash` /
# `stepplan-wash-anthropic` with the unlimited-budget key. Copying rather than
# hand-writing the entries is deliberate: baseUrl, api shape and the model list stay
# byte-identical to the working providers, so the wash providers cannot drift from them.
#
# The key NEVER leaves this machine: it is read from a gitignored file, written only to
# ~/.pi/agent/models.json (chmod 600), and never echoed, logged, or passed as an
# argument (argv is visible to every process via `ps`).
#
# Idempotent: re-running overwrites the two wash entries and touches nothing else.
# A timestamped backup of models.json is taken first.
#
# Usage: bash scripts/configure-wash-provider.sh
set -uo pipefail

KEY_FILE="${SEATLOOM_WASH_KEY_FILE:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/key/stepplan_key_unlimit.txt}"
MODELS="$HOME/.pi/agent/models.json"

[ -r "$KEY_FILE" ] || { echo "key file not readable: $KEY_FILE" >&2; exit 1; }
[ -r "$MODELS" ]   || { echo "pi models.json not readable: $MODELS" >&2; exit 1; }

cp -p "$MODELS" "$MODELS.bak-$(date +%Y%m%d-%H%M%S)" || { echo "backup failed" >&2; exit 1; }

KEY_FILE="$KEY_FILE" MODELS="$MODELS" python3 - <<'PY'
import copy, json, os, sys

models = os.environ["MODELS"]
key = open(os.environ["KEY_FILE"]).read().strip()
if not key:
    sys.exit("key file is empty")

doc = json.load(open(models))
providers = doc.get("providers")
if not isinstance(providers, dict):
    sys.exit("models.json has no providers object")

PAIRS = [
    ("stepplan", "stepplan-wash",
     "StepFun StepPlan WASH (OpenAI) - unlimited budget, data-washing only"),
    ("stepplan-anthropic", "stepplan-wash-anthropic",
     "StepFun StepPlan WASH (Anthropic) - unlimited budget, data-washing only"),
]

for src, dst, label in PAIRS:
    if src not in providers:
        sys.exit(f"source provider {src!r} is missing; refusing to invent one")
    entry = copy.deepcopy(providers[src])
    entry["name"] = label
    entry["apiKey"] = key
    providers[dst] = entry
    ids = [m.get("id", m) if isinstance(m, dict) else m for m in entry.get("models", [])]
    # Deliberately prints no key material, only its length.
    print(f"registered [{dst}] api={entry.get('api')} baseUrl={entry.get('baseUrl')} "
          f"models={ids} keylen={len(key)}")

with open(models, "w") as fh:
    json.dump(doc, fh, indent=2, ensure_ascii=False)
    fh.write("\n")
PY
rc=$?
chmod 600 "$MODELS" 2>/dev/null
[ "$rc" -eq 0 ] && echo "ok: wash providers registered in $MODELS (mode 600)"
exit "$rc"
