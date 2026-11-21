# W4-d35 ATLAS-REPACK: emit the union sheet list with owner branches + dims.
# Reads _d35_pushed_sheets.json (re-extracted vs base a725acb0), resolves each
# name's REAL .xnb dims via the pinned harness's sheetDims (LZX first-frame),
# and writes _d35_union_list.json + a markdown table sorted by padded area
# desc (the packing priority order: tallest-first is the packer's sort, but
# area-desc is the merge-priority view — biggest consumers first).
import json, subprocess, os, re

HERE = os.path.dirname(os.path.abspath(__file__))
pushed = json.load(open(os.path.join(HERE, "_d35_pushed_sheets.json")))

D40_IDS = [430,431,432,433,434,435,436,437,438,440,442,443,444,445,446,
  447,448,449,450,451,452,453,454,455,456,457,458,459,460,461,462,463,464,465,
  466,467,468,469,470,472,473,474,475,476,477,478,479,480,481,482,483,484,485,
  486,487,488]
D35_DEFERRED = [79,80,84,85,86,103,104,109,110,120,122,140]

owners = {}
for b, names in pushed.items():
    for n in names:
        owners.setdefault(n, []).append(b.replace("origin/task/", ""))
for i in D40_IDS:
    owners.setdefault(f"NPC_{i}", []).append("W4-d40-npc-kinds-421-490 (FORECAST: data-only today, sheets phase pending)")
for i in D35_DEFERRED:
    owners.setdefault(f"NPC_{i}", []).append("W4-d35-npc-kinds-71-140 (DEFERRED: packed only if a repack admits it)")
for n in owners:
    owners[n] = sorted(set(owners[n]))

# dims via node one-shot over the pinned harness head
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
r = subprocess.run(["node", "-e", js, json.dumps(names)], capture_output=True, text=True, cwd=HERE)
if r.returncode != 0:
    raise SystemExit(r.stderr)
dims = json.loads(r.stdout.strip().splitlines()[-1])

rows = []
for n in names:
    d = dims.get(n)
    rows.append({
        "name": n, "owners": owners[n],
        "w": d["w"] if d else None, "h": d["h"] if d else None,
        "padded_area": (d["w"]+2)*(d["h"]+2) if d else 0,
        "missing": d is None,
    })
rows.sort(key=lambda r: (-r["padded_area"], r["name"]))
json.dump(rows, open(os.path.join(HERE, "_d35_union_list.json"), "w"), indent=1)

total = sum(r["padded_area"] for r in rows)
missing = [r["name"] for r in rows if r["missing"]]
print(f"union names: {len(rows)} | with dims: {len(rows)-len(missing)} | missing: {len(missing)}")
print(f"total padded area: {total/1e6:.2f} Mpx -> {total/(8192-2):.0f} px of width-8192 bin")
if missing: print("missing:", ", ".join(missing))
print("\ntop 25 by padded area:")
for r in rows[:25]:
    print(f"  {r['name']:40s} {r['w']}x{r['h']}  {r['padded_area']/1e3:.1f} kpx  {','.join(r['owners'])}")
