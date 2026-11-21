# W4-d35 ATLAS FINAL: emit Appendix B (the 83-name delta between the
# original 407-name appendix and the FINAL 490-name union list).
import json, os, re

HERE = os.path.dirname(os.path.abspath(__file__))
rows = json.load(open(os.path.join(HERE, "_d35_union_list.json")))
doc = open(os.path.join(HERE, "..", "docs", "research",
                        "113-atlas-repack-plan.md"), encoding="utf-8").read()
app = doc.split("## Appendix A")[1].split("## Appendix B")[0]
old = set(re.findall(r"\| `([^`]+)` \|", app))
print("old appendix names:", len(old))
new = [r for r in rows if r["name"] not in old]
print("delta:", len(new))
lines = ["| # | sheet | w×h | padded kpx | owner branch(es) |",
         "|---|-------|-----|-----------|------------------|"]
for i, r in enumerate(new, 1):
    own = ", ".join(
        o.replace("W4-", "").replace("-npc-kinds", "")
         .replace(" (FORECAST: data-only today, sheets phase pending)", "†")
         .replace(" (DEFERRED: packed only if a repack admits it)", "‡")
         .replace(" (sheets tip 2dfe026d, local ahead of origin)", " (tip 2dfe026d, local)")
        for o in r["owners"])
    lines.append(f"| {i} | `{r['name']}` | {r['w']}×{r['h']} | "
                 f"{r['padded_area']/1e3:.1f} | {own} |")
out = "\n".join(lines) + "\n"
open(os.path.join(HERE, "_d35_delta_table.md"), "w", encoding="utf-8").write(out)
print("wrote _d35_delta_table.md")
