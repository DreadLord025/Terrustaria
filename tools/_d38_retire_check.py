# W4-d38 RETIRE phase mechanical verification (RUN, code-only mode).
# Four receipts:
#  R1: the 7-town-stand-in residual at d44 tip c863a7a5 (which ids the
#      wired hooks ACTUALLY claim vs the 68 table's WIRED cells).
#  R2: the 328 extra-card drop-hunk anchors (exact current lines).
#  R3: the GAP-1 critter gate — d38's six critters pass c8's GENERIC
#      counts_as_a_critter at af1f8ba1d (SetDefaults rows) and the
#      scan's live gate; no other d38 kind passes.
#  R4: the SEAL census pins (35 kinds / 35 entries / 35 slots / 35
#      sheets; the 216 declared == ids == vec rows).
import re
import subprocess
import sys

ROOT = "."

def read(p):
    return open(p, encoding="utf-8").read()

def git_show(rev, p):
    r = subprocess.run(["git", "show", f"{rev}:{p}"], capture_output=True)
    assert r.returncode == 0, (rev, p)
    return r.stdout.decode("utf-8")

ok = True
def check(name, cond, detail=""):
    global ok
    print(("PASS " if cond else "FAIL ") + name + (" — " + detail if detail else ""))
    if not cond:
        ok = False

# ---------------- R1: the d44-tip claim sets ----------------
D44 = "c863a7a5"
npc_d44 = git_show(D44, "crates/tr-app/src/npc.rs")
e6 = git_show(D44, "crates/tr-app/src/npc_findframe_extra6.rs")
e7 = git_show(D44, "crates/tr-app/src/npc_findframe_extra7.rs")
b = git_show(D44, "crates/tr-app/src/npc_findframe_b.rs")

def mapper_ids(src, fn):
    i = src.find(f"pub fn {fn}(")
    seg = src[i:src.find("}", src.find("_ => return None", i))]
    return {int(m) for m in re.findall(r"^\s*(\d+) =>", seg, re.M)}

s6 = mapper_ids(e6, "find_frame_kind_extra6")
s7 = mapper_ids(e7, "find_frame_kind_extra7")
check("R1 extra6 mapper == the declared 15",
      s6 == {278,279,280,281,282,283,284,285,286,287,290,291,292,293,295},
      str(sorted(s6)))
check("R1 extra7 mapper == the declared 15",
      s7 == {316,317,318,322,323,324,333,334,335,336,337,437,438,440,442},
      str(sorted(s7)))

# the hooks: claimed-kind early returns PRECEDE the body match
ff = npc_d44[npc_d44.find("pub(crate) fn find_frame(&mut self)"):]
body = ff.find("match self.kind")
h6 = ff.find("find_frame_kind_extra6_claimed(self.kind.cs_id())")
h7 = ff.find("find_frame_kind_extra7_claimed(self.kind.cs_id())")
check("R1 extra6 hook precedes the body match", 0 < h6 < body)
check("R1 extra7 hook precedes the body match", 0 < h7 < body)

# wave-B has NO cs_id hook / kind mapper anywhere on the d44 tip
check("R1 wave-B module has no mapper", "pub fn find_frame_kind" not in b)
check("R1 no wave-B hook in d44 npc.rs",
      "find_frame_wave_b" not in npc_d44 and "npc_findframe_b::find_frame_b(" not in npc_d44)

SEVEN = [287, 294, 295, 296, 322, 323, 324]
wired = sorted(k for k in SEVEN if k in s6 or k in s7)
residual = sorted(k for k in SEVEN if k not in s6 and k not in s7)
check("R1 hook-wired of the seven == [287,295,322,323,324]",
      wired == [287, 295, 322, 323, 324], str(wired))
check("R1 RESIDUAL == [294,296] (wave-B module, no hook on any tip)",
      residual == [294, 296], str(residual))

# our branch: the seven kinds' cs_id rows resolve into the claim sets
npc_ours = read("crates/tr-app/src/npc.rs")
cs = {k: int(v) for k, v in re.findall(r"NpcKind::(\w+) => (\d+),", npc_ours)}
names = {"BoneLee":287, "AngryBonesBig":294, "AngryBonesBigMuscle":295,
         "AngryBonesBigHelmet":296, "SkeletonTopHat":322,
         "SkeletonAstonaut":323, "SkeletonAlien":324}
check("R1 our cs_id rows == the seven ids",
      all(cs[k] == v for k, v in names.items()))
check("R1 our branch has NO extra6/extra7 module (drop is merge-time)",
      "find_frame_kind_extra6" not in npc_ours and "find_frame_kind_extra7" not in npc_ours)

# the fighter delegates carry air_row0 exactly for 295 (287 keeps row 1)
check("R1 extra6 fighter air_row0 == AngryBonesBigMuscle only",
      "let air_row0 = kind == FindFrameExtra6Kind::AngryBonesBigMuscle;" in e6)
check("R1 extra7 fighter delegate air_row0=false idle_row1=false",
      re.search(r"SkeletonAlien => \{.*?humanoid_fighter_frame\(\s*&mut bs,\s*&bi,\s*kind\.frame_count\(\),\s*false,\s*false,\s*\)", e7, re.S) is not None)
check("R1 wave-B air pin list carries 294/296",
      "| FindFrameKind::AngryBonesBig\n" in b and "| FindFrameKind::AngryBonesBigHelmet\n" in b)

# ---------------- R2: the 328 drop-hunk anchors ----------------
best = read("crates/tr-app/src/bestiary.rs")
assets = read("crates/tr-app/src/assets.rs")
uib = read("crates/tr-app/src/ui/bestiary.rs")

anchors = [
    ("ENTRIES 328 row", best, 'credit_id: "PumpkingBlade",'),
    ("slot id 328", assets, "333, 334, 335, 336, 337, 318, 322, 323, 324, 328,"),
    ("vec row NPC_328", assets, 'atlas.sheet("NPC_328").copied(),'),
    ("portrait arm 328", uib, "(NpcKind::PumpkingBlade, _) => {"),
    ("strips witness 328 row", uib, "(328, 106, 86, 1),"),
    ("GAP-3 pin", best, 'assert!(ENTRIES.iter().any(|e| e.net_id == 328),'),
]
for name, src, needle in anchors:
    n = src.count(needle)
    ln = src[:src.find(needle)].count("\n") + 1 if n else -1
    check(f"R2 {name} unique", n == 1, f"line {ln}")

# the kill credit path is entry_for_npc().expect — the drop REQUIRES a
# fold or guard (the blade despawns hp=0 via the missing-carrier arm)
check("R2 entry_for_npc panics on a missing row",
      '.expect("every ported NPC kind has a bestiary entry")' in best)
sim = read("crates/tr-app/src/world_sim/npcs.rs")
check("R2 kill-credit loop rides entry_for_npc for every dead npc",
      "let entry = crate::bestiary::entry_for_npc(npc.kind, npc.slime);" in sim)
s59 = read("crates/tr-app/src/npc_ai_style59.rs")
check("R2 the blade's silent despawn sets hp = 0", "npc.hp = 0;" in s59)
check("R2 no 328 fold in entry_for_npc today",
      "NpcKind::PumpkingBlade =>" not in best.split("pub fn entry_for_npc")[1].split(".expect")[0])

# ---------------- R3: the critter gate coverage ----------------
C8 = "af1f8ba1d"  # the c8 LOCAL FINAL tip (origin pushes stop at 3705670d3)
c8b = git_show(C8, "crates/tr-app/src/bestiary.rs")
check("R3 generic gate exists at c8 final tip",
      "fn counts_as_a_critter(kind: NpcKind) -> bool {" in c8b)
check("R3 gate reads max_hp/contact_damage/594/686",
      re.search(r"kind\.max_hp\(\) <= 5\s*&& kind\.contact_damage\(\) == 0\s*&& kind\.cs_id\(\) != 594\s*&& kind\.cs_id\(\) != 686", c8b) is not None)
check("R3 dispatch rides the generic gate + the 549 Hide exclusion",
      "entry.kind != NpcKind::DD2LanePortal && counts_as_a_critter(entry.kind)" in c8b)
check("R3 scan gate is the live-row twin",
      "if n.max_hp > 5 || n.kind.contact_damage() != 0 {" in c8b)

data = read("crates/tr-app/src/npc/data.rs")
def fn_rows(src, header):
    i = src.find(header)
    depth, began, rows = 0, False, {}
    for line in src[i:].splitlines():
        depth += line.count("{") - line.count("}")
        if "{" in line:
            began = True
        if "=>" in line:
            m = re.search(r"=>\s*(-?\d+)", line)
            if m:
                for k in re.findall(r"NpcKind::(\w+)", line):
                    rows[k] = int(m.group(1))
        if began and depth == 0:
            return rows
    return rows

hp = fn_rows(data, "pub(crate) fn max_hp(self)")
dmg = fn_rows(data, "pub(crate) fn contact_damage(self)")
K35 = ["RaggedCaster","RaggedCasterOpenCoat","Necromancer","NecromancerArmored",
       "DiabolistRed","DiabolistWhite","BoneLee","GiantCursedSkull","Paladin",
       "SkeletonSniper","TacticalSkeleton","SkeletonCommando","AngryBonesBig",
       "AngryBonesBigMuscle","AngryBonesBigHelmet","BirdBlue","BirdRed","Squirrel",
       "Mouse","Raven","SlimeMasked","BunnySlimed","HoppinJack","Ghost",
       "DemonEyeOwl","DemonEyeSpaceship","SkeletonTopHat","SkeletonAstonaut",
       "SkeletonAlien","PumpkingBlade","SlimeRibbonWhite","SlimeRibbonYellow",
       "SlimeRibbonGreen","SlimeRibbonRed","BunnyXmas"]
SIX = {"BirdBlue","BirdRed","Squirrel","Mouse","BunnySlimed","BunnyXmas"}
passing = {k for k in K35 if hp.get(k, 99) <= 5 and dmg.get(k, 1) == 0
           and cs.get(k) not in (594, 686)}
check("R3 exactly the SIX d38 critters pass the generic gate",
      passing == SIX, str(sorted(passing)))
for k in sorted(SIX):
    print(f"     {k:12s} cs_id={cs[k]:3d} max_hp={hp[k]} contact_damage={dmg[k]}")

# credit ids == the scan's entry_for_npc credit (no fold for the six)
for k, cid in [("BirdBlue","BirdBlue"),("BirdRed","BirdRed"),("Squirrel","Squirrel"),
               ("Mouse","Mouse"),("BunnySlimed","BunnySlimed"),("BunnyXmas","BunnyXmas")]:
    check(f"R3 {k} credit_id row", f'credit_id: "{cid}",' in best)
check("R3 no gold peer of the six in GOLD_PEER_TABLE keys",
      all(f"({cs[k]}, " not in c8b for k in SIX))

# ---------------- R4: the SEAL census pins ----------------
i = best.find("ENTRIES")
entries_seg = best[best.find("pub static ENTRIES") if "pub static ENTRIES" in best else i:]
entries_seg = entries_seg[:entries_seg.find("\n];")]
n_entries = len(re.findall(r"BestiaryEntryDef \{", entries_seg))
d38_entry_rows = len(re.findall(r"// W4-d38 b\d — BestiaryEntry\.", best))
check("R4 35 d38 ENTRIES rows", d38_entry_rows == 35, str(d38_entry_rows))
check("R4 base ENTRIES total 284 (249 + 35)", n_entries == 284, str(n_entries))

i = assets.find("pub const BESTIARY_NPC_SHEET_IDS")
seg = assets[i:assets.find("];", i)]
declared = int(re.search(r"\[u16; (\d+)\]", seg).group(1))
ids = [int(x) for x in re.findall(r"\d+", re.sub(r"//[^\n]*", "", seg).split("[", 2)[2])]
i = assets.find("bestiary_npc_sheets: vec![")
vec = assets[i:assets.find("],", i)]
n_vec = vec.count("atlas.sheet")
check("R4 slots declared==ids==vec == 216", declared == len(ids) == n_vec == 216,
      f"{declared}/{len(ids)}/{n_vec}")
d38_ids = [281,282,283,284,285,286,287,289,290,291,292,293,294,295,296,
           297,298,299,300,301,302,303,304,316,317,318,322,323,324,328,
           333,334,335,336,337]
check("R4 all 35 d38 ids hold slots", all(x in ids for x in d38_ids))
check("R4 all 35 d38 sheets in STATIC_SHEET_NAMES",
      all(f'"NPC_{x}",' in assets for x in d38_ids))
# post-drop arithmetic (the R2 hunk applied): 215 slots / 283 entries
check("R4 post-drop arithmetic 216-1=215 / 284-1=283 / 323-1=322",
      216 - 1 == 215 and 284 - 1 == 283 and 323 - 1 == 322)

print()
print("ALL PASS" if ok else "FAILURES PRESENT")
sys.exit(0 if ok else 1)