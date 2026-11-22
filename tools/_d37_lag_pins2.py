# W4-d37 LAGGARD-EXEC: POST-DROP count pins. Map every ENTRIES row / sheet
# / slot / cue to its KIND, keep only the surviving-owner rows (the FINAL
# union + the drop verdicts), then count. core = SEAL roster + Vampire;
# grand = the lag_union 373-kind plan.
import re, subprocess, json

REPO = r"C:\Users\DreadLord\Desktop\TR-W4-d37"

def show(rev, path):
    r = subprocess.run(["git", "-C", REPO, "show", f"{rev}:{path}"],
                       capture_output=True)
    return r.stdout.decode("utf8", "replace") if r.returncode == 0 else ""

def entries(rev):
    src = show(rev, "crates/tr-app/src/bestiary.rs")
    m = re.search(r"const ENTRIES[^=]*=\s*&?\[(.*?)\n\];", src, re.S)
    if not m: return []
    body = re.sub(r"//[^\n]*", "", m.group(1))
    rows = []
    for r in re.split(r"BestiaryEntryDef \{", body)[1:]:
        c = re.search(r'credit_id:\s*"([^"]+)"', r)
        n = re.search(r"net_id:\s*(-?\d+|crate::loot_db::\w+)", r)
        k = re.search(r"kind:\s*NpcKind::(\w+)", r)
        if c and n and k:
            rows.append((c.group(1), n.group(1), k.group(1)))
    return rows

union = json.load(open(REPO + r"\tools\_d37_lag_union.json"))
grand = union["grand"]
seal = json.load(open(REPO + r"\tools\_d37_seal_final.json"))
core = dict(seal["final"])
core["Vampire"] = 762

# alias fold: dropped variant kind -> canonical kind
ALIAS = dict(union["aliases"])

TIPS = {
 "c3": "origin/task/W4-c3-style10-cursed-skull",
 "c4": "origin/task/W4-c4-style44-flying-fish",
 "c6": "origin/task/W4-c6-style59-pumpking-blade",
 "c7": "origin/task/W4-c7-style64-fireflies",
 "c8": "origin/task/W4-c8-style66-crawling-critters",
 "c9": "origin/task/W4-c9-style67-snails",
 "c10": "origin/task/W4-c10-style68-waterfowl",
 "d23": "origin/task/W4-d23-npc-ai-styles-9-14",
 "d33": "origin/task/W4-d33-npc-ai-styles-95-127",
 "d34": "origin/task/W4-d34-npc-kinds-1-70",
 "d35": "origin/task/W4-d35-npc-kinds-71-140",
 "d37": "750d8e446",
 "d38": "origin/task/W4-d38-npc-kinds-281-350",
 "d40": "origin/task/W4-d40-npc-kinds-421-490",
 "d42": "origin/task/W4-d42-npc-kinds-561-630",
 "d120": "origin/task/W4-d120-town-chatter",
 "d125": "origin/task/W4-d125-empress-r7-drain",
 "e26": "origin/task/W4-e26-luck-system",
 "g8": "origin/task/W4-g8-npc-kind-wire-1",
 "d36": "origin/task/W4-d36-npc-kinds-141-210",
 "d39": "origin/task/W4-d39-npc-kinds-351-420",
 "d41": "origin/task/W4-d41-npc-kinds-491-560",
 "d43": "origin/task/W4-d43-npc-kinds-631-697",
 "g10": "origin/task/W4-g10-npc-kind-wire-3",
 "d9": "origin/task/W4-d9-proj-ai-81-90",
 "g5": "origin/task/W4-g5-npc-style7-caster-doc",
 "f42": "origin/task/W4-f42-sandstorm-2",
}
owners = {k: v[0] for k, v in seal["owners"].items()}
owners["Vampire"] = "g8"
for k, v in union["additions"].items():
    owners[k] = v["lane"]

out = {}
for tag, roster in [("core", core), ("grand", grand)]:
    lanes = sorted({owners[k] for k in roster if k in owners})
    keep_entries = {}   # (credit, net) -> lane
    kinds_with_entry = set()
    for lane in lanes:
        for credit, net, kind in entries(TIPS[lane]):
            canon = ALIAS.get(kind, kind)
            if canon not in roster: continue
            if owners.get(canon) != lane: continue
            keep_entries[(credit, net)] = lane
            kinds_with_entry.add(canon)
    no_entry = sorted(k for k in roster if k in owners and k not in kinds_with_entry)
    out[tag] = {"kinds": len(roster), "entries": len(keep_entries),
                "owned_no_entry": len(no_entry),
                "all_kinds_pin": 270 + len(roster),
                "max_id": max(roster.values())}
    if tag == "core": out["core_no_entry_list"] = no_entry

json.dump(out, open(REPO + r"\tools\_d37_lag_pins2.json", "w"), indent=1, sort_keys=True)
print(json.dumps(out, indent=1))
