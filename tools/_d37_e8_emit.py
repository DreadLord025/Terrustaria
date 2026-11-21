# Emit the E8 per-lane renumber-INSTRUCTION tables (markdown) from the
# verified canonical plan. Output goes to tools/_d37_e8_tables.md for
# splicing into research/108.
import json, collections
REPO = r"C:\Users\DreadLord\Desktop\TR-W4-d37"
maps = json.load(open(REPO+r"\tools\_d37_receipts_maps.json"))["lanes"]
plan = json.load(open(REPO+r"\tools\_d37_receipts_final.json"))
own = {k: tuple(v) for k,v in plan["owners"].items()}
fin = plan["final"]; drops = plan["drops"]

by_lane = collections.defaultdict(list)
for k,(l,pid) in own.items(): by_lane[l].append((pid,k))
drop_by_lane = collections.defaultdict(list)
for k,l,pid,keep in drops: drop_by_lane[l].append((pid,k,keep))

lines=[]
for l in sorted(by_lane):
    rows=sorted(by_lane[l]); moved=[(p,k) for p,k in rows if fin[k]!=p]
    dl=sorted(drop_by_lane[l])
    lines.append(f"### {l} - {len(rows)} owned, {len(moved)} renumber, {len(dl)} drop\n")
    if moved:
        lines.append("| Kind | old `to_u16`/`from_u16` | new id | cs_id |")
        lines.append("|---|---|---|---|")
        for p,k in moved:
            cs=maps[l]["cs"].get(k)
            lines.append(f"| {k} | {p} | **{fin[k]}** | {cs if cs is not None else '(fold/sentinel)'} |")
        lines.append("")
    keeps=[(p,k) for p,k in rows if fin[k]==p]
    if keeps and moved:
        lines.append("Keep as pushed: " + ", ".join(f"{k}@{p}" for p,k in keeps) + ".\n")
    if dl:
        lines.append("DROP duplicate ports (keep canonical owner's row+wiring):")
        for p,k,keep in dl:
            kp = own[k][1]; kf = fin[k]
            lines.append(f"- {k}: drop {l}@{p} -> keep {keep}@{kf}" + (f" (renumbered from {kp})" if kf!=kp else ""))
        lines.append("")
open(REPO+r"\tools\_d37_e8_tables.md","w",encoding="utf-8").write("\n".join(lines))
print("\n".join(lines[:40]))
print(f"... {len(lines)} lines total")
