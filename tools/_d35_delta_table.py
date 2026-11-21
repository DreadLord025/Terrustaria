# W4-d35 ATLAS RETIRE: emit the RETIRE delta (the new names between the
# FINAL 490-name union = Appendix A + Appendix B, and the RETIRE 552-name
# union list). The prior FINAL delta (83 names) stays in Appendix B; this
# writes the RETIRE append (Appendix B-2).
import json, os, re

HERE = os.path.dirname(os.path.abspath(__file__))
rows = json.load(open(os.path.join(HERE, "_d35_union_list.json")))
doc = open(os.path.join(HERE, "..", "docs", "research",
                        "113-atlas-repack-plan.md"), encoding="utf-8").read()
# the prior 490-name union = Appendix A + Appendix B (everything before B-2)
head = doc.split("## Appendix B-2")[0]
appAB = head.split("## Appendix A")[1]
old = set(re.findall(r"\| `([^`]+)` \|", appAB))
print("prior union names (A+B):", len(old))
new = [r for r in rows if r["name"] not in old]
print("RETIRE delta:", len(new))
lines = ["| # | sheet | w×h | padded kpx | owner branch(es) |",
         "|---|-------|-----|-----------|------------------|"]
for i, r in enumerate(new, 1):
    own = ", ".join(
        o.replace("W4-", "").replace("-npc-kinds", "")
         .replace(" (FORECAST: data-only today, sheets phase pending)", "†")
         .replace(" (DEFERRED: packed only if a repack admits it)", "‡")
         .replace(" (sheets tip 86e3adfd3, local ahead of origin)", " (tip 86e3adfd3, local)")
        for o in r["owners"])
    lines.append(f"| {i} | `{r['name']}` | {r['w']}×{r['h']} | "
                 f"{r['padded_area']/1e3:.1f} | {own} |")
out = "\n".join(lines) + "\n"
open(os.path.join(HERE, "_d35_retire_delta_table.md"), "w", encoding="utf-8").write(out)
print("wrote _d35_retire_delta_table.md")
