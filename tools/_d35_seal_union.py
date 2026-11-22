# W4-d35 SEAL-VERIFY: regenerate the union list at the SEAL re-extraction
# (65 branches / 630 pushed names -> 678 union) with dims + owners, WITHOUT
# touching the committed _d35_union_list.json (the retire witness). Writes
# _d35_seal_union.json + the Appendix C table body _d35_seal_delta_table.md.
import json, subprocess, os

HERE = os.path.dirname(os.path.abspath(__file__))
pushed = json.load(open(os.path.join(HERE, "_d35_pushed_sheets.json")))
delta = json.load(open(os.path.join(HERE, "_d35_seal_delta.json")))

D40_IDS = [430,431,432,433,434,435,436,437,438,440,442,443,444,445,446,
  447,448,449,450,451,452,453,454,455,456,457,458,459,460,461,462,463,464,465,
  466,467,468,469,470,472,473,474,475,476,477,478,479,480,481,482,483,484,485,
  486,487,488]
D35_DEFERRED = [79,80,84,85,86,103,104,109,110,120,122,140]
D125_TIP = ["NPC_636", "NPC_661", "Extra_188"]
D125_DEFERRED = ["Extra_157", "Extra_158", "Extra_159", "Extra_160", "Extra_187"]
# d42 FINAL 32-kind band (research/239 §4), NO-PACK verdict (research/238:
# eager 8382 +190 OVER) - forecast rows like d40's 56; 580/630 pushed
# by f42 and 619 by d11 already, these 29 are the unpacked remainder.
D42_FORECAST = [578,579,582,583,584,585,586,590,591,593,594,610,611,612,613,
  614,616,617,618,620,621,622,623,624,625,626,627,628,629]

owners = {}
for b, names in pushed.items():
    for n in names:
        owners.setdefault(n, []).append(b.replace("origin/task/", ""))
for n in D125_TIP:
    owners.setdefault(n, []).append("W4-d125-empress-r7-drain (sheets tip, now PUSHED)")
for n in D125_DEFERRED:
    owners.setdefault(n, []).append("W4-d125-empress-r7-drain (DEFERRED)")
for i in D40_IDS:
    owners.setdefault(f"NPC_{i}", []).append("W4-d40-npc-kinds-421-490 (FORECAST)")
for i in D35_DEFERRED:
    owners.setdefault(f"NPC_{i}", []).append("W4-d35-npc-kinds-71-140 (DEFERRED)")
for i in D42_FORECAST:
    owners.setdefault(f"NPC_{i}", []).append("W4-d42-npc-kinds-561-630 (FORECAST: NO-PACK verdict, research/238)")
for n in owners:
    owners[n] = sorted(set(owners[n]))

names = sorted(owners)
js = """
const fs=require('fs'),path=require('path');
const src=fs.readFileSync(path.join(__dirname,'_d35_atlas_sim_pinned.js'),'utf8');
const head=src.slice(0,src.indexOf('function main()'));
const fn=new Function('require','__dirname','names',head+`
  const out={};
  for(const n of names){const d=sheetDims(n);out[n]=d;}
  return out;`);
console.log(JSON.stringify(fn(require,__dirname,JSON.parse(process.argv[1]))));
"""
r = subprocess.run(["node", "-e", js, json.dumps(names)], capture_output=True,
                   text=True, cwd=HERE)
if r.returncode != 0:
    raise SystemExit(r.stderr)
dims = json.loads(r.stdout.strip().splitlines()[-1])

rows = []
for n in names:
    d = dims.get(n)
    rows.append({"name": n, "owners": owners[n],
                 "w": d["w"] if d else None, "h": d["h"] if d else None,
                 "padded_area": (d["w"]+2)*(d["h"]+2) if d else 0,
                 "missing": d is None})
rows.sort(key=lambda r: (-r["padded_area"], r["name"]))
json.dump(rows, open(os.path.join(HERE, "_d35_seal_union.json"), "w"), indent=1)
total = sum(r["padded_area"] for r in rows)
missing = [r["name"] for r in rows if r["missing"]]
print(f"union names: {len(rows)} | with dims: {len(rows)-len(missing)} | missing: {len(missing)}")
print(f"total padded area: {total/1e6:.2f} Mpx -> {total/(8192-2):.0f} px of width-8192 bin")
if missing: print("missing:", ", ".join(missing))

# Appendix C table: the genuinely-new 116 pushed + 29 d42 forecast, area-desc
newset = set(delta["genuinelyNew"]) | set(delta["d42Forecast"])
newrows = [r for r in rows if r["name"] in newset]
lines = ["| # | name | w x h | padded kpx | owners |", "|---|---|---|---|---|"]
for i, r in enumerate(newrows, 1):
    ow = ", ".join(o.replace("W4-", "") for o in r["owners"])
    wh = f'{r["w"]}x{r["h"]}' if not r["missing"] else "MISSING"
    lines.append(f'| {i} | `{r["name"]}` | {wh} | {r["padded_area"]/1e3:.1f} | {ow} |')
open(os.path.join(HERE, "_d35_seal_delta_table.md"), "w").write("\n".join(lines) + "\n")
print("wrote _d35_seal_union.json + _d35_seal_delta_table.md")
npc = [r for r in newrows if r["name"].startswith("NPC_") and r["name"][4:].isdigit()]
print("Appendix C rows:", len(newrows), "| NPC-band", len(npc),
      "| non-NPC", len(newrows)-len(npc))
print("non-NPC new area Mpx:", sum(r["padded_area"] for r in newrows
      if r not in npc)/1e6)
print("d42-forecast area Mpx:", sum(r["padded_area"] for r in newrows
      if r["name"] in set(delta["d42Forecast"]))/1e6)
