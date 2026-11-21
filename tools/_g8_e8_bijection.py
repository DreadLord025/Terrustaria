# W4-g8 E8 SEAL — the scripted id-space bijection census (code-only lane;
# no cargo — this parses the match arms directly). Pins:
#   to_u16 305 arms / from_u16 305 arms / id sets IDENTICAL / zero
#   duplicates / max 762; the E8 band 745-761 + Vampire 762 present;
#   the freed old ids absent; the 17 DROP-PENDING dups keep pushed ids;
#   render_key mirrors the 35 g8 kinds; ALL_KINDS carries 305 rows.
# Evidence: d37 research/108 E8 g8 table (lines 1176-1214) + the SEAL
# amendment (fbb948172 line 1408: 788-790 RESERVED for d39 — the
# in-flight Vampire-788 draft was the defect; CORRECT = 762, absent
# from tools/_d37_seal_final.json's core/grand maps).
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DATA = (ROOT / "crates/tr-app/src/npc/data.rs").read_text(encoding="utf-8")
NPC = (ROOT / "crates/tr-app/src/npc.rs").read_text(encoding="utf-8")

def slice_fn(src, header):
    i = src.index(header)
    j = src.index("\n    }", i)
    return src[i:j]

to_src = slice_fn(DATA, "pub fn to_u16(")
from_src = slice_fn(DATA, "pub fn from_u16(")

to_arms = re.findall(r"NpcKind::(\w+) => (\d+),", to_src)
from_arms = re.findall(r"(\d+) => NpcKind::(\w+),", from_src)

to_map = {}
from_map = {}
dup_to, dup_from = [], []
for kind, sid in to_arms:
    if kind in to_map:
        dup_to.append(kind)
    to_map[kind] = int(sid)
for sid, kind in from_arms:
    if int(sid) in from_map:
        dup_from.append(int(sid))
    from_map[int(sid)] = kind

errors = []
def check(cond, msg):
    if not cond:
        errors.append(msg)

check(len(to_arms) == 305, f"to_u16 arms {len(to_arms)} != 305")
check(len(from_arms) == 305, f"from_u16 arms {len(from_arms)} != 305")
check(not dup_to, f"to_u16 duplicate kinds: {dup_to}")
check(not dup_from, f"from_u16 duplicate ids: {dup_from}")
ids_to = set(to_map.values())
ids_from = set(from_map.keys())
check(ids_to == ids_from, f"id sets differ: to-only {sorted(ids_to - ids_from)}, from-only {sorted(ids_from - ids_to)}")
check(all(from_map.get(v) == k for k, v in to_map.items()), "to/from not inverse")
check(len(set(to_map.values())) == len(to_map), "to_u16 ids not distinct")
check(max(ids_to) == 762, f"max id {max(ids_to)} != 762")

# The E8 g8 move-table (d37 research/108 lines 1176-1188), verbatim.
MOVES = {
    "PinkJellyfish": (356, 745), "Antlion": (357, 746), "SpikeBall": (358, 747),
    "WallCreeper": (366, 748), "WallCreeperWall": (367, 749),
    "IceElemental": (368, 750), "Herpling": (369, 751), "Derpling": (370, 752),
    "CrimsonAxe": (371, 753), "FloatyGross": (372, 754), "LostGirl": (373, 755),
    "Nymph": (374, 756), "IceBat": (380, 757), "Lavabat": (381, 758),
    "GiantFlyingFox": (382, 759), "VampireBat": (383, 760), "RedDevil": (387, 761),
}
for kind, (old, new) in MOVES.items():
    check(to_map.get(kind) == new, f"{kind}: to_u16 {to_map.get(kind)} != {new}")
    check(old not in ids_to, f"{kind}: old id {old} not freed")
DUPS = {
    "Vulture": 354, "BlueJellyfish": 355, "Pixie": 359, "Wraith": 360,
    "CursedHammer": 361, "EnchantedSword": 362, "Mimic": 363,
    "GreenJellyfish": 364, "Gastropod": 365, "CursedSkull": 375, "Demon": 376,
    "GiantBat": 377, "Slimer": 378, "IlluminantBat": 379, "FlyingSnake": 384,
    "Squid": 385, "Harpy": 386,
}
for kind, sid in DUPS.items():
    check(to_map.get(kind) == sid, f"{kind}: drop-pending id {to_map.get(kind)} != {sid}")
check(to_map.get("Vampire") == 762, f"Vampire {to_map.get('Vampire')} != 762")
for reserved in (388, 788, 789, 790):
    check(reserved not in ids_to, f"{reserved} must stay free (d37/d39)")

# render_key mirrors the g8 kinds (npc.rs's WorldFrameKey hash match).
rk_i = NPC.index("NpcKind::Vulture => 354,")
rk_j = NPC.index("} << 5;", rk_i)
rk_arms = dict((k, int(v)) for k, v in re.findall(r"NpcKind::(\w+) => (\d+),", NPC[rk_i:rk_j]))
for kind in list(MOVES) + list(DUPS) + ["Vampire"]:
    check(rk_arms.get(kind) == to_map.get(kind),
          f"render_key {kind}: {rk_arms.get(kind)} != to_u16 {to_map.get(kind)}")

# ALL_KINDS: 305 rows, one per to_u16 kind.
ak_i = NPC.index("const ALL_KINDS:")
ak_j = NPC.index("\n    ];", ak_i)
ak = re.findall(r"NpcKind::(\w+),", NPC[ak_i:ak_j])
check("[NpcKind; 305]" in NPC[ak_i:ak_i + 120], "ALL_KINDS decl not 305")
check(len(ak) == 305, f"ALL_KINDS rows {len(ak)} != 305")
check(set(ak) == set(to_map), "ALL_KINDS kinds != to_u16 kinds")
check(len(set(ak)) == len(ak), "ALL_KINDS has duplicates")

if errors:
    print("BIJECTION FAIL")
    for e in errors:
        print(" -", e)
    sys.exit(1)
print(f"BIJECTION OK: to_u16 {len(to_arms)} arms / from_u16 {len(from_arms)} arms / "
      f"sets identical / zero duplicates / max {max(ids_to)}; "
      f"E8 band 745-761 + Vampire 762 present; 17 dups drop-pending; "
      f"388/788-790 free (d37/d39); render_key + ALL_KINDS(305) mirror.")
