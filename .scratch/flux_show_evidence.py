import json, os, sys

# stdin: the MCP stdio transcript's last line (the tools/call result).
# Prints one line per evidence pointer with whether the path is on disk.
raw = sys.stdin.read().strip().splitlines()[-1]
r = json.loads(raw)["result"]
assert not r["isError"], r["content"][0]["text"]
d = json.loads(r["content"][0]["text"])
for e in d["evidence"]:
    ondisk = os.path.isfile(e["id"]) if e["id"].startswith("/") else "n/a"
    print("    kind=%-11s onDisk=%-5s id=%s" % (e["kind"], str(ondisk), e["id"]))
    print("      detail: %s" % (e["detail"] or ""))
print("    noRecord: %s" % (d["noRecord"]["statement"] if d["noRecord"] else None))
