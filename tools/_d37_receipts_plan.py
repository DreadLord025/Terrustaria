# W4-d37 receipts phase: build the CANONICAL final assignment from the
# re-censused maps, verify collision-free, emit per-lane renumber tables.
import json, collections

P = r"C:\Users\DreadLord\Desktop\TR-W4-d37\tools\_d37_receipts_maps.json"
maps = json.load(open(P))["lanes"]

# ---- dedupe: keep canonical owner (108 E3, with the c4-verdict correction
# for CrimsonGoldfish: keep c4@465's CORRECT impl, retire d40's kb-0.4 row;
# the id stays 465 inside d40's frozen band).
KEEP = {
 "BartenderUnconscious": ("d42",), "BloodFeeder": ("d37",),
 "BlueJellyfish": ("d34",), "Buggy": ("d40",), "Clinger": ("d35",),
 "CorruptGoldfish": ("d34",), "CrimsonGoldfish": ("c4",),
 "CursedHammer": ("d35",), "CursedSkull": ("d34",), "Demon": ("d34",),
 "Dolphin": ("d42",), "Duck": ("c10",), "EnchantedNightcrawler": ("d40",),
 "EnchantedSword": ("d35",), "EyeballFlyingFish": ("d42",),
 "FlyingAntlion": ("d42",), "FlyingFish": ("d37",), "FlyingSnake": ("d37",),
 "FungiBulb": ("d37",), "Gastropod": ("d35",), "GiantBat": ("d35",),
 "GiantCursedSkull": ("d38",), "GiantFungiBulb": ("d37",),
 "GoldGoldfish": ("d42",), "GoldLadyBug": ("d42",), "GoldWorm": ("d40",),
 "Grebe": ("d42",), "Grebe2": ("d42",), "GreenJellyfish": ("d35",),
 "Grubby": ("d40",), "Harpy": ("d34",), "IlluminantBat": ("d35",),
 "LadyBug": ("d42",), "Maggot": ("d42",), "ManEater": ("d34",),
 "Mimic": ("d35",), "Pixie": ("d35",), "PumpkingBlade": ("d38",),
 "Pupfish": ("d42",), "Seagull": ("d42",), "Seagull2": ("d42",),
 "Shimmerfly": ("c7",), "Slimer": ("d35",), "Sluggy": ("d40",),
 "Snatcher": ("d34",), "Squid": ("d37",), "Vulture": ("d34",),
 "Wraith": ("d35",),
 # NEW at receipts time: d33 PUSHED the fully-wired aiStyle-114 dragonflies
 # 595-601 (e71010d3) after the E-census saw them local-only; d42's cs
 # 578-630 band also ports them. Both land at the SAME final ids (595-601
 # vanilla-free); d33 is the superset impl (axis arm + Hide folds + Gold's
 # own card + catch chain) -> canonical owner d33, d42 drops its rows.
 "BlackDragonfly": ("d33",), "BlueDragonfly": ("d33",),
 "GreenDragonfly": ("d33",), "OrangeDragonfly": ("d33",),
 "RedDragonfly": ("d33",), "YellowDragonfly": ("d33",),
 "GoldDragonfly": ("d33",),
}
# NOTE: the fish five (Dolphin/GoldGoldfish/Pupfish/EyeballFlyingFish/
# FlyingAntlion): E3's kinds-lane rule keeps d42 ownership; d42's renumber
# to cs 578-630 lands them at EXACTLY c4's pushed vanilla-free ids
# (581/587/592/607/615), so the c4 verdict's "c4 stands" and E3 agree on
# the final id. c4's duplicate ports drop; c4's style-16/44 wiring stays
# (E3's dropped-lane-keeps-wiring rule). CrimsonGoldfish is the EXCEPTION:
# the c4 verdict 318e28709 proved d40's row defective (kb 0.4 vs C# 1f,
# missing vanilla_type arm -> unreachable! panic) -> keep c4's IMPL at the
# shared id 465 (E3 correction R2).

owners = {}   # kind -> (lane, pushed_id)
drops = []    # (kind, lane, pushed_id, keeper)
for lane, d in maps.items():
    if lane == "base": continue
    for k, pid in sorted(d["new"].items(), key=lambda x: x[1]):
        if k in owners:
            kl, kp = owners[k]
            keep = KEEP.get(k, (None,))[0]
            if keep is None:
                raise SystemExit(f"UNADJUDICATED duplicate: {k} {kl}@{kp} vs {lane}@{pid}")
            if keep == lane:
                drops.append((k, kl, kp, lane)); owners[k] = (lane, pid)
            else:
                assert keep == kl, f"{k}: keep {keep} but owners {kl}/{lane}"
                drops.append((k, lane, pid, kl))
        else:
            owners[k] = (lane, pid)

# ---- canonical renumber plan (108 E2/E3 + receipts corrections)
final = {}    # kind -> final id
lane_order = lambda lane, pid: pid
# keep lanes untouched:
for k, (lane, pid) in owners.items():
    final[k] = pid
# d42 -> vanilla-free cs (578-630), only its owned kinds
for k, (lane, pid) in owners.items():
    if lane == "d42":
        cs = maps["d42"]["cs"][k]
        assert cs is not None and 578 <= cs <= 630, (k, cs)
        final[k] = cs
# d35 -> 698-730 order-preserving (+269)
for k, (lane, pid) in owners.items():
    if lane == "d35":
        final[k] = pid + 269
# g8 owned -> 745-761 order-preserving over pushed ids
g8_owned = sorted([(pid, k) for k, (lane, pid) in owners.items() if lane == "g8"])
SMALL = {  # explicit per-kind small-lane shifts (108 E3, exact kind names)
 ("c10","Duck"):731, ("c10","Duck2"):732, ("c10","DuckWhite"):733, ("c10","DuckWhite2"):734,
 ("c7","Firefly"):735, ("c7","LightningBug"):736,
 ("c8","Worm"):737, ("c8","TruffleWorm"):738, ("c8","TruffleWormDigger"):739,
 ("c9","Snail"):740, ("c9","GlowingSnail"):741,
 ("d125","PrismaticLacewing"):742,
 ("d23","AngryTrapper"):743,
 ("e26","Frog"):744,
}
for (lane,k),nid in SMALL.items():
    assert owners.get(k,(None,None))[0]==lane, f"SMALL key not owned: {lane}/{k}"
    final[k]=nid

print("g8 owned kinds (pushed order):")
for i,(pid,k) in enumerate(g8_owned):
    print(f"  {k:22s} {pid} -> {745+i}")
    final[k] = 745+i

# c4 secondary-flag renumber (c4 verdict 318e28709: CorruptGoldfish/Arapaima
# squat reserved cs holes 229/237; CorruptGoldfish is dropped to d34@277).
# Arapaima stays c4-owned -> needs a genuinely free port-local id.
# d125 EmpressOfLight 229 also squats the Pirate cs hole (d37 Pirate@361,
# cs 229) -> same correction class.
print("\nc4/d125 reserved-hole owners:")
for k,(lane,pid) in owners.items():
    if pid in (229,237,241,245):
        print(f"  {k:22s} {lane}@{pid} cs={maps[lane]['cs'][k]}")

json.dump({"owners": owners, "final": final, "drops": drops},
          open(r"C:\Users\DreadLord\Desktop\TR-W4-d37\tools\_d37_receipts_plan_raw.json","w"),
          indent=1, sort_keys=True)
print("\nowned:", len(owners), "drops:", len(drops))
