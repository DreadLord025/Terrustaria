# W4-d37 LAGGARD-EXEC: count pins for the FINAL v2 plan (389 kinds).
# ENTRIES/sheets/slots/cues unions restricted to the surviving-owner rows.
import re, subprocess, json

REPO = r"C:\Users\DreadLord\Desktop\TR-W4-d37"
def show(rev, p):
    r = subprocess.run(["git", "-C", REPO, "show", f"{rev}:{p}"], capture_output=True)
    return r.stdout.decode("utf8", "replace") if r.returncode == 0 else ""

def sheets(rev):
    m = re.search(r"STATIC_SHEET_NAMES[^=]*=\s*&?\[(.*?)\n\];", show(rev, "crates/tr-app/src/assets.rs"), re.S)
    return set(re.findall(r'"(NPC_[^"]+)"', re.sub(r"//[^\n]*", "", m.group(1)))) if m else set()
def slots(rev):
    m = re.search(r"BESTIARY_NPC_SHEET_IDS:\s*\[u16;\s*(\d+)\]\s*=\s*\[(.*?)\];", show(rev, "crates/tr-app/src/assets.rs"), re.S)
    return [int(x) for x in re.findall(r"\b(\d+)\b", re.sub(r"//[^\n]*", "", m.group(2)))] if m else []
def cues(rev):
    m = re.search(r"SFX_CUES[^=]*=\s*&?\[(.*?)\n\];", show(rev, "crates/tr-app/src/audio.rs"), re.S)
    return set(re.findall(r'"(NPC_[^"]+)"', re.sub(r"//[^\n]*", "", m.group(1)))) if m else set()
def entries(rev):
    src = show(rev, "crates/tr-app/src/bestiary.rs")
    m = re.search(r"const ENTRIES[^=]*=\s*&?\[(.*?)\n\];", src, re.S)
    if not m: return []
    body = re.sub(r"//[^\n]*", "", m.group(1)); rows = []
    for r in re.split(r"BestiaryEntryDef \{", body)[1:]:
        c = re.search(r'credit_id:\s*"([^"]+)"', r); n = re.search(r"net_id:\s*(-?\d+|crate::loot_db::\w+)", r); k = re.search(r"kind:\s*NpcKind::(\w+)", r)
        if c and n and k: rows.append((c.group(1), n.group(1), k.group(1)))
    return rows

f2 = json.load(open(REPO + r"\tools\_d37_lag_final2.json"))
plan = f2["plan"]; tips = f2["tips"]
ALIAS = {"HallowBoss":"EmpressOfLight","EmpressButterfly":"PrismaticLacewing","VileSpitEoW":"VileSpitEaterOfWorlds"}
REV = {l: ("750d8e446" if l == "d37" else "origin/task/W4-" + B) for l, B in {
 "c3":"c3-style10-cursed-skull","c4":"c4-style44-flying-fish","c6":"c6-style59-pumpking-blade",
 "c7":"c7-style64-fireflies","c8":"c8-style66-crawling-critters","c9":"c9-style67-snails",
 "c10":"c10-style68-waterfowl","d23":"d23-npc-ai-styles-9-14","d33":"d33-npc-ai-styles-95-127",
 "d34":"d34-npc-kinds-1-70","d35":"d35-npc-kinds-71-140","d36":"d36-npc-kinds-141-210",
 "d38":"d38-npc-kinds-281-350","d39":"d39-npc-kinds-351-420","d40":"d40-npc-kinds-421-490",
 "d41":"d41-npc-kinds-491-560","d42":"d42-npc-kinds-561-630","d43":"d43-npc-kinds-631-697",
 "d120":"d120-town-chatter","d125":"d125-empress-r7-drain","e26":"e26-luck-system",
 "g8":"g8-npc-kind-wire-1","g10":"g10-npc-kind-wire-3","d9":"d9-proj-ai-81-90",
 "g5":"g5-npc-style7-caster-doc","f42":"f42-sandstorm-2","e21":"e21-wiring-device-arms-3"}.items()}
REV["d37"] = "750d8e446"

owners = {k: v[1] for k, v in plan.items()}
lanes = sorted(set(owners.values()))
keep_entries = {}; kinds_with_entry = set()
us, ss, cs = set(), set(slots("a725acb04")), set()
for lane in lanes:
    rev = REV[lane]
    us |= sheets(rev); ss |= set(slots(rev)); cs |= cues(rev)
    for credit, net, kind in entries(rev):
        canon = ALIAS.get(kind, kind)
        if canon not in plan or owners.get(canon) != lane: continue
        keep_entries[(credit, net)] = lane; kinds_with_entry.add(canon)
no_entry = sorted(k for k in plan if k not in kinds_with_entry)
out = {"plan_kinds": len(plan), "all_kinds_pin": 270 + len(plan),
       "entries": len(keep_entries), "owned_no_entry": len(no_entry),
       "no_entry_list": no_entry, "sheets_union": len(us),
       "slots_distinct": len(ss), "cues": len(cs), "max_id": f2["max_id"]}
json.dump(out, open(REPO + r"\tools\_d37_lag_pins3.json", "w"), indent=1, sort_keys=True)
print(json.dumps({k: v for k, v in out.items() if k != "no_entry_list"}, indent=1))
print("no_entry:", no_entry)
