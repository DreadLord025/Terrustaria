# W4-d35 ATLAS-REPACK: render the union list as the doc's markdown table.
import json, os
HERE = os.path.dirname(os.path.abspath(__file__))
rows = json.load(open(os.path.join(HERE, "_d35_union_list.json")))
lines = ["| # | sheet | w×h | padded kpx | owner branch(es) |",
         "|---|-------|-----|-----------|------------------|"]
for i, r in enumerate(rows, 1):
    own = ", ".join(o.replace("W4-", "").replace("-npc-kinds", "").replace(" (FORECAST: data-only today, sheets phase pending)", "†").replace(" (DEFERRED: packed only if a repack admits it)", "‡") for o in r["owners"])
    lines.append(f"| {i} | `{r['name']}` | {r['w']}×{r['h']} | {r['padded_area']/1e3:.1f} | {own} |")
out = "\n".join(lines) + "\n"
open(os.path.join(HERE, "_d35_union_table.md"), "w", encoding="utf-8").write(out)
print(f"{len(rows)} rows written")
