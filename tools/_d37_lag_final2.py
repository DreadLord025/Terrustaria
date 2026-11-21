# W4-d37 LAGGARD-EXEC: build the FINAL adjudicated plan (v2) at the LIVE
# tips. SEAL final + union additions + live-wave verdicts:
#  - d43 research/244 executed-adjudication ACCEPTED (688/692 flip to d43,
#    LibrarianSkeleton to g5, 18 d43 drops; 665/666 keep-d43 rationale read)
#  - VileSpitEaterOfWorlds==VileSpitEoW alias: d43's full-name kind keeps
#    cs==proto 666 (kinds-lane rule), d23's abbreviation drops
#  - d23's style-14 wave dups drop to the SEAL owners (g8/d34/d35/d37);
#    its two UNIQUE kinds SporeBat 407 / QueenSlimeMinionPurple 409 flip to
#    cs==proto 634/660 (d43 drops its stand-ins)
#  - d34 phase-4: Crab/DungeonGuardian re-pin 791/792 (d38 owns 282/283);
#    Antlion/PinkJellyfish/SpikeBall drop to g8 (wired axis machines)
#  - e21 PartyBunny 237 ACCEPT (free after d125 executed 237->742)
#  - g8 Vampire 388->762 (d37 owns 388)
# Then re-derive owed moves/drops + clash gates. Read-only.
import re, subprocess, json

REPO = r"C:\Users\DreadLord\Desktop\TR-W4-d37"
DATA = "crates/tr-app/src/npc/data.rs"

def git(*a):
    r = subprocess.run(["git", "-C", REPO] + list(a), capture_output=True)
    return r.returncode, r.stdout.decode("utf8", "replace")

def show(rev, path):
    rc, out = git("show", f"{rev}:{path}")
    return out if rc == 0 else ""

def fn_body(src, sig):
    i = src.index(sig); j = src.index("{", i); d = 0; k = j
    while True:
        c = src[k]
        if c == "{": d += 1
        elif c == "}":
            d -= 1
            if d == 0: return src[j:k+1]
        k += 1

def arms(rev):
    src = show(rev, DATA)
    if not src: return {}
    return {m[0]: int(m[1]) for m in re.findall(
        r"NpcKind::(\w+) => (\d+),", fn_body(src, "pub fn to_u16"))}

seal = json.load(open(REPO + r"\tools\_d37_seal_final.json"))
plan = {k: [v, seal["owners"][k][0]] for k, v in seal["final"].items()}
union = json.load(open(REPO + r"\tools\_d37_lag_union.json"))
for k, v in union["additions"].items():
    plan[k] = [v["id"], v["lane"]]

def setp(k, i, lane, why):
    plan[k] = [i, lane]
    verdicts.append({"kind": k, "id": i, "lane": lane, "why": why})
verdicts = []

# d36 live tip: five re-pins EXECUTED + three new free-band kinds
for k, i in [("ToxicSludge",767),("Penguin",768),("PenguinBlack",769),
             ("GiantTortoise",770),("IceTortoise",771)]:
    setp(k, i, "d36", "EXECUTED re-pin (was 762-766; R6 towns + Vampire 762)")
for k, i in [("Crimera",781),("MossHornet",782),("Moth",793)]:
    setp(k, i, "d36", "ACCEPT new free-band kind as-pushed")
# d41 batch 4-6: ACCEPT as-pushed (vanilla ids)
for k, i in [("SolarGoop",519),("TombCrawlerHead",513),("TombCrawlerBody",514),
             ("TombCrawlerTail",515),("AncientLight",522),("AncientDoom",523),
             ("DesertGhoul",524),("DesertGhoulCorruption",525),
             ("DesertGhoulCrimson",526),("DesertGhoulHallow",527),
             ("DesertLamiaLight",528),("DesertLamiaDark",529),
             ("DesertScorpionWalk",530),("DesertScorpionWall",531)]:
    setp(k, i, "d41", "ACCEPT as-pushed vanilla band")
# d43 research/244 verdicts (the lane's own retire doc, cross-checked here)
setp("Pufferfish", 688, "d43", "research/148 conditional FIRED: kinds-lane owns; c4 retires + folds just_hit fix")
setp("Orca", 692, "d43", "same conditional; c4 retires + folds wet-anchor + research/144 sheet census")
setp("LibrarianSkeleton", 693, "g5", "g5 carries the LIVE caster wiring; d43 casts silently (244 sec1)")
# 666 alias: d43's full-name kind wins cs==proto (kinds-lane), d23 abbrev drops
setp("VileSpitEaterOfWorlds", 666, "d43", "alias of d23 VileSpitEoW@411; kinds-lane cs==proto wins; d23 drops")
del plan["VileSpitEoW"]
# d23's two unique style-14 kinds -> cs==proto, d43 stand-ins drop
setp("SporeBat", 634, "d23", "d23 wires the 634 flap; re-pin 407->634 cs==proto; d43 stand-in drops")
setp("QueenSlimeMinionPurple", 660, "d23", "d23 wires the 660 eye-group; re-pin 409->660; d43 stand-in drops")
# d34 phase-4
setp("Crab", 791, "d34", "RENUMBER 282->791 (d38 owns RaggedCasterOpenCoat@282)")
setp("DungeonGuardian", 792, "d34", "RENUMBER 283->792 (d38 owns Necromancer@283)")
# e21
setp("PartyBunny", 237, "e21", "ACCEPT 237 (free after d125 executed 237->742; f18 dup drops)")
# g8 Vampire (from union additions, restated for the verdict log)
setp("Vampire", 762, "g8", "RENUMBER 388->762 (d37 owns RustyArmoredBonesAxe@388)")

# ---- gates: unique ids, zero base clash
base_ids = set(arms("a725acb04").values())
ids = [v[0] for v in plan.values()]
dups = {i for i in ids if ids.count(i) > 1}
assert not dups, ("id collision", [(i, [k for k in plan if plan[k][0] == i]) for i in dups])
clash = {k: v for k, v in plan.items() if v[0] in base_ids}
assert not clash, ("base clash", clash)

# ---- owed derivation at LIVE tips
LANES = {
 "c3":"origin/task/W4-c3-style10-cursed-skull","c4":"origin/task/W4-c4-style44-flying-fish",
 "c6":"origin/task/W4-c6-style59-pumpking-blade","c7":"origin/task/W4-c7-style64-fireflies",
 "c8":"origin/task/W4-c8-style66-crawling-critters","c9":"origin/task/W4-c9-style67-snails",
 "c10":"origin/task/W4-c10-style68-waterfowl","d23":"origin/task/W4-d23-npc-ai-styles-9-14",
 "d33":"origin/task/W4-d33-npc-ai-styles-95-127","d34":"origin/task/W4-d34-npc-kinds-1-70",
 "d35":"origin/task/W4-d35-npc-kinds-71-140","d36":"origin/task/W4-d36-npc-kinds-141-210",
 "d37":"750d8e446","d38":"origin/task/W4-d38-npc-kinds-281-350",
 "d39":"origin/task/W4-d39-npc-kinds-351-420","d40":"origin/task/W4-d40-npc-kinds-421-490",
 "d41":"origin/task/W4-d41-npc-kinds-491-560","d42":"origin/task/W4-d42-npc-kinds-561-630",
 "d43":"origin/task/W4-d43-npc-kinds-631-697","d120":"origin/task/W4-d120-town-chatter",
 "d125":"origin/task/W4-d125-empress-r7-drain","e26":"origin/task/W4-e26-luck-system",
 "g8":"origin/task/W4-g8-npc-kind-wire-1","g10":"origin/task/W4-g10-npc-kind-wire-3",
 "d9":"origin/task/W4-d9-proj-ai-81-90","d11":"origin/task/W4-d11-proj-ai-101-110",
 "g5":"origin/task/W4-g5-npc-style7-caster-doc","f42":"origin/task/W4-f42-sandstorm-2",
 "f18":"origin/task/W4-f18-npc-kinds-verify-2","f47":"origin/task/W4-f47-liquid-shimmer-2",
 "e21":"origin/task/W4-e21-wiring-device-arms-3","g62":"origin/task/W4-g62-npc-hit-dust",
}
base = set(arms("a725acb04"))
ALIAS = {"HallowBoss":"EmpressOfLight","EmpressButterfly":"PrismaticLacewing",
         "VileSpitEoW":"VileSpitEaterOfWorlds"}
tips, cur = {}, {}
for lane, rev in LANES.items():
    rc, t = git("rev-parse", "--short", rev)
    tips[lane] = t.strip()
    cur[lane] = {k: v for k, v in arms(rev).items() if k not in base}
moves, drops, done = {}, {}, {}
for lane, kinds in cur.items():
    for k, id_ in kinds.items():
        if k in ALIAS:
            drops.setdefault(lane, []).append(
                {"kind":k,"id":id_,"keeper":plan[ALIAS[k]][1],"why":"alias"})
            continue
        if k not in plan:
            drops.setdefault(lane, []).append(
                {"kind":k,"id":id_,"keeper":"?","why":"UNPLANNED"})
            continue
        target, owner = plan[k]
        if lane == owner:
            if id_ == target: done.setdefault(lane, []).append(k)
            else: moves.setdefault(lane, []).append({"kind":k,"old":id_,"new":target})
        else:
            drops.setdefault(lane, []).append(
                {"kind":k,"id":id_,"keeper":owner,"why":"dup"})
nm = sum(len(v) for v in moves.values()); nd = sum(len(v) for v in drops.values())
out = {"tips":tips,"plan":plan,"plan_size":len(plan),"max_id":max(ids),
       "verdicts":verdicts,"moves_total":nm,"drops_total":nd,
       "moves":moves,"drops":drops,"done":done}
json.dump(out, open(REPO+r"\tools\_d37_lag_final2.json","w"), indent=1, sort_keys=True)
print(f"FINAL v2: {len(plan)} kinds, max {max(ids)}; OWED {nm} moves / {nd} drops")
for lane in sorted(set(moves)|set(drops)):
    print(f"  {lane:6s} tip={tips[lane]} moves={len(moves.get(lane,[])):2d} drops={len(drops.get(lane,[])):2d}")
unp=[(l,d) for l,ds in drops.items() for d in ds if d["why"]=="UNPLANNED"]
print("UNPLANNED:", unp)
print("ALL GATES PASSED (unique ids, zero base clash)")
