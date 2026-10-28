import sys, json

d = json.loads(json.load(sys.stdin)["result"]["content"][0]["text"])
rows = d["projects"]
print("projectCount:", d["projectCount"], "sessionTotal:", d["sessionTotal"],
      "messageTotal:", d["messageTotal"], "corpusLoaded:", d["corpusLoaded"])
for i, p in enumerate(rows, 1):
    print("%3d. %4d sess %6d msg  unclassified=%-5s %s" % (
        i, p["sessionCount"], p["messageCount"], p["unclassified"], p["id"]))
print()
print("is the unclassified/root group LAST in the returned order?",
      rows[-1]["unclassified"] if rows else None)
print("positions of unclassified rows:", [i for i, p in enumerate(rows, 1) if p["unclassified"]])
print("sessionCount sequence is descending?",
      all(rows[i]["sessionCount"] >= rows[i + 1]["sessionCount"] for i in range(len(rows) - 1)))
