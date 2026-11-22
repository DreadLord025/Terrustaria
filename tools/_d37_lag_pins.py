# W4-d37 LAGGARD-EXEC: FINAL grand count-pin re-derive at CURRENT tips.
# core (SEAL 19-lane roster) vs grand (+d36/d39/d41/d43/g10/d9/g5/f42).
# sheets/slots/cues unions; ENTRIES per-owner dedupe; ALL_KINDS arithmetic.
import re, subprocess, json

REPO = r"C:\Users\DreadLord\Desktop\TR-W4-d37"

def show(rev, path):
    r = subprocess.run(["git", "-C", REPO, "show", f"{rev}:{path}"],
                       capture_output=True)
    return r.stdout.decode("utf8", "replace") if r.returncode == 0 else ""

def sheets(rev):
    m = re.search(r"STATIC_SHEET_NAMES[^=]*=\s*&?\[(.*?)\n\];",
                  show(rev, "crates/tr-app/src/assets.rs"), re.S)
    if not m: return set()
    return set(re.findall(r'"(NPC_[^"]+)"', re.sub(r"//[^\n]*", "", m.group(1))))

def slots(rev):
    m = re.search(r"BESTIARY_NPC_SHEET_IDS:\s*\[u16;\s*(\d+)\]\s*=\s*\[(.*?)\];",
                  show(rev, "crates/tr-app/src/assets.rs"), re.S)
    if not m: return []
    return [int(x) for x in re.findall(r"\b(\d+)\b", re.sub(r"//[^\n]*", "", m.group(2)))]

def cues(rev):
    src = show(rev, "crates/tr-app/src/audio.rs")
    m = re.search(r"SFX_CUES[^=]*=\s*&?\[(.*?)\n\];", src, re.S)
    if not m: return set()
    return set(re.findall(r'"(NPC_[^"]+)"', re.sub(r"//[^\n]*", "", m.group(1))))

def entries(rev):
    src = show(rev, "crates/tr-app/src/bestiary.rs")
    m = re.search(r"const ENTRIES[^=]*=\s*&?\[(.*?)\n\];", src, re.S)
    if not m: return []
    body = re.sub(r"//[^\n]*", "", m.group(1))
    return [(c, int(n)) for c, n in re.findall(
        r'credit_id:\s*"([^"]+)",\s*net_id:\s*(-?\d+)', body)]

CORE = {  # lane -> current tip (SEAL core roster owners)
 "base": "a725acb04",
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
}
GRAND_EXTRA = {
 "d36": "origin/task/W4-d36-npc-kinds-141-210",
 "d39": "origin/task/W4-d39-npc-kinds-351-420",
 "d41": "origin/task/W4-d41-npc-kinds-491-560",
 "d43": "origin/task/W4-d43-npc-kinds-631-697",
 "g10": "origin/task/W4-g10-npc-kind-wire-3",
 "d9": "origin/task/W4-d9-proj-ai-81-90",
 "g5": "origin/task/W4-g5-npc-style7-caster-doc",
 "f42": "origin/task/W4-f42-sandstorm-2",
}

out = {}
u_core, u_grand = set(), set()
s_core, s_grand = set(), set()
c_core, c_grand = set(), set()
bsl = slots("a725acb04")
e_core, e_grand = {}, {}
for tag, group in [("core", CORE), ("grand", {**CORE, **GRAND_EXTRA})]:
    us, ss, cs, ee = set(), set(bsl), set(), {}
    for lane, rev in group.items():
        us |= sheets(rev); ss |= set(slots(rev)); cs |= cues(rev)
        for credit, net in entries(rev):
            ee[(credit, net)] = lane
    out[tag] = {"sheets": len(us), "slots_distinct": len(ss),
                "cues": len(cs), "entries_distinct": len(ee)}
    if tag == "core": u_core, s_core, c_core, e_core = us, ss, cs, ee
    else: u_grand, s_grand, c_grand, e_grand = us, ss, cs, ee

out["grand_new_sheets"] = sorted(u_grand - u_core)
out["grand_new_cues"] = sorted(c_grand - c_core)
json.dump(out, open(REPO + r"\tools\_d37_lag_pins.json", "w"), indent=1, sort_keys=True)
print(json.dumps({k: v for k, v in out.items() if not k.startswith("grand_new")}, indent=1))
print("grand-new sheets:", len(out["grand_new_sheets"]))
print("grand-new cues:", out["grand_new_cues"])
