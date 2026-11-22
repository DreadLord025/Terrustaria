# W4-d37 E8-SEAL: emit the refreshed per-lane renumber-INSTRUCTION tables
# (old->new) + exact drop-hunks, from the verified plan + accounting.
import json

REPO = r"C:\Users\DreadLord\Desktop\TR-W4-d37"
census = json.load(open(REPO + r"\tools\_d37_e8s_census.json"))["lanes"]
plan = json.load(open(REPO + r"\tools\_d37_e8s_plan.json"))
acct = json.load(open(REPO + r"\tools\_d37_e8s_acct.json"))
owners = {k: tuple(v) for k, v in plan["owners"].items()}
final = plan["final"]

def cs_of(l, k):
    return census[l]["cs"].get(k)
def net_of(l, k):
    return census[l]["new_entries"].get(k)

lines = []
lines.append("# E8-RECONCILE-SEAL per-lane renumber tables (refreshed at current tips)\n")
lines.append("Each shifting lane executes its table on ITS OWN branch: change the listed")
lines.append("`to_u16` arms AND the mirrored `from_u16` arms, update witness pins/comments")
lines.append("naming the old protocol id, and DROP the listed duplicate ports (variant +")
lines.append("to/from arms + ALL_KINDS row + ENTRIES row + stat arms). `cs_id`, ENTRIES")
lines.append("`net_id`, sheets, render keys and every cs/net-keyed seam DO NOT CHANGE.\n")
for l in sorted(acct):
    a = acct[l]
    if not a["moves"] and not a["drops"]:
        lines.append(f"## {l} — {len(a['keeps'])} owned, 0 renumber, 0 drop (KEEP receipt)\n")
        continue
    lines.append(f"## {l} — {len(a['moves'])+len(a['keeps'])} owned, "
                 f"{len(a['moves'])} renumber, {len(a['drops'])} drop\n")
    if a["moves"]:
        lines.append("| Kind | old | new | cs_id | net_id |")
        lines.append("|---|---|---|---|---|")
        for k, pid, fid in a["moves"]:
            lines.append(f"| {k} | {pid} | **{fid}** | {cs_of(l,k)} | {net_of(l,k)} |")
        lines.append("")
    if a["keeps"] and a["moves"]:
        lines.append("Keep as pushed: " + ", ".join(f"{k}@{pid}" for k, pid, _ in a["keeps"]) + "\n")
    if a["drops"]:
        lines.append("DROP duplicate ports (keep canonical owner's row+wiring):")
        for k, pid, keeper in a["drops"]:
            if keeper == "collapsed-name":
                keeper = {"HallowBoss": "d125 EmpressOfLight@636 (same vanilla 636)",
                          "EmpressButterfly": "d125 PrismaticLacewing@661 (same vanilla 661)",
                          "VileSpitEoW": "d43 VileSpitEaterOfWorlds@666 (same vanilla 666)"}.get(k, keeper)
            lines.append(f"- {k}: drop {l}@{pid} -> keep {keeper}")
        lines.append("")
open(REPO + r"\tools\_d37_e8s_tables.md", "w", encoding="utf-8").write("\n".join(lines))
print("tables written:", len(lines), "lines")
# summary of renumber owed per lane
for l in sorted(acct):
    if acct[l]["moves"]:
        print(l, "moves:", [(k, p, f) for k, p, f in acct[l]["moves"]])
