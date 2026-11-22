# W4-d35 SEAL-VERIFY: probe the named claimant branches at their CURRENT tips.
# (2) NEW claimants since the retire 562-union: d42/d43 (npc bands, claim NO
# pack), f29 (map-overlay-2, 109 overlay sheets claim), e23 (MapBGOverlay
# 7759px claim), g84 (Extra_260/Glow_294), c8/c9 (crawler sheets).
# (3) the in-union lanes: d38 (35 sheets), d40 (roster), g10/g11 (pets).
# Comment-tolerant row regex (the RETIRE fix). Read-only: no writes to git.
import subprocess, re, json, os

HERE = os.path.dirname(os.path.abspath(__file__))
BASE = "a725acb0"
def git(*a):
    return subprocess.run(["git", *a], capture_output=True, text=True,
                          encoding="utf-8", errors="replace").stdout

union = {r["name"] for r in json.load(open(os.path.join(HERE, "_d35_union_list.json")))}
print("committed union size:", len(union))
row = re.compile(r'^\+\s+"([^"{} ]+)",?\s*(//.*)?$')

SLUGS = ["d42-npc-kinds-561-630", "d43-npc-kinds-631-697",
         "f29-map-overlay-2", "e23-bestiary-ui-grid",
         "g84-plantera-bulb-draw", "c8-style66-crawling-critters",
         "c9-style67-snails", "d38-npc-kinds-281-350",
         "d40-npc-kinds-421-490", "g10-npc-kind-wire-3",
         "g11-dust-main-sites-1"]
out = {}
for slug in SLUGS:
    b = "origin/task/W4-" + slug
    tip = git("rev-parse", "--short=9", b).strip()
    d = git("diff", BASE + ".." + b, "--", "crates/tr-app/src/assets.rs")
    added = [m.group(1) for l in d.splitlines() if (m := row.match(l))]
    names = sorted(set(added))
    notin = [n for n in names if n not in union]
    out[slug] = {"tip": tip, "rows": names, "notInUnion": notin}
    print("\n%s tip %s: %d rows, %d NOT in 562-union" %
          (slug, tip, len(names), len(notin)))
    if notin:
        print("  not-in-union:", notin[:40], "..." if len(notin) > 40 else "")
json.dump(out, open(os.path.join(HERE, "_d35_seal_probe.json"), "w"), indent=1)
print("\nwrote _d35_seal_probe.json")
