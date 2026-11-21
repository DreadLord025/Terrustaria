# W4-d38 SEAL-RECEIPT phase mechanical verification (RUN, code-only mode).
# Five receipts (research/125 §§14-18):
#  S1: the {294,296} residual mapper-hook task-seed for d44f-wire-lane at
#      the CURRENT d44 tip ce2c3a24a (wire round 4) — claim sets, the
#      wave-B module arms, the cs_id resolution path, the hook boundary.
#  S2: the 328 credit-preserving-guard consumer census (the FULL panic
#      surface: kill loop + spawn_loot + net_id() + the client kill arm
#      + the cs_id sentinel) + the vanilla credit-chain cites.
#  S3: the c8 GAP-1 receipt at the c8d-crawler-retire-active tip
#      7c2de9c06 (origin == local) — the generic gate, the six d38
#      critters, the 3 kill-only UI call sites + the e23 static-list
#      delta.
#  S4: the d35 atlas-lazy compose — the 35 d38 sheets in the 562-name
#      union at 1afd327b9 (owners/dims/missing), the step-7 obligation.
#  S5: the §13b SEAL re-receipt — the census pins stand.
import json
import re
import subprocess
import sys

def read(p):
    return open(p, encoding="utf-8").read()

def git_show(rev, p):
    r = subprocess.run(["git", "show", f"{rev}:{p}"], capture_output=True)
    assert r.returncode == 0, (rev, p)
    return r.stdout.decode("utf-8-sig")

ok = True
def check(name, cond, detail=""):
    global ok
    print(("PASS " if cond else "FAIL ") + name + (" — " + detail if detail else ""))
    if not cond:
        ok = False

def line_of(src, needle):
    i = src.find(needle)
    return src[:i].count("\n") + 1 if i >= 0 else -1

# ---------------- S1: the 294/296 mapper-hook seed ----------------
D44 = "ce2c3a24a"  # wire round 4 (past the RETIRE census's c863a7a5)
npc_d44 = git_show(D44, "crates/tr-app/src/npc.rs")
b = git_show(D44, "crates/tr-app/src/npc_findframe_b.rs")
e6 = git_show(D44, "crates/tr-app/src/npc_findframe_extra6.rs")
e7 = git_show(D44, "crates/tr-app/src/npc_findframe_extra7.rs")
e8 = git_show(D44, "crates/tr-app/src/npc_findframe_extra8.rs")

def mapper_ids(src, fn):
    i = src.find(f"pub fn {fn}(")
    seg = src[i:src.find("}", src.find("_ => return None", i))]
    return {int(m) for m in re.findall(r"^\s*(\d+) =>", seg, re.M)}

claimed = (mapper_ids(e6, "find_frame_kind_extra6")
           | mapper_ids(e7, "find_frame_kind_extra7")
           | mapper_ids(e8, "find_frame_kind_extra8"))
check("S1 no pushed mapper claims 294/296 at ce2c3a24a",
      294 not in claimed and 296 not in claimed)
check("S1 wave-B still has NO cs_id mapper/hook at ce2c3a24a",
      "pub fn find_frame_kind" not in b
      and "find_frame_wave_b" not in npc_d44
      and "npc_findframe_b::find_frame_b(" not in npc_d44)
check("S1 d44 tip has NO AngryBonesBig kind (the hook is stream-inert there)",
      "NpcKind::AngryBonesBig" not in npc_d44
      and "NpcKind::AngryBonesBig" not in git_show(D44, "crates/tr-app/src/npc/data.rs"))
# the module arms: the fighter group + the air pin 0 + the row counts
check("S1 wave-B fighter group carries 294/296",
      "| FindFrameKind::AngryBonesBig\n" in b
      and "| FindFrameKind::AngryBonesBigHelmet\n" in b)
check("S1 wave-B air pin list carries 294/296 (row 0)",
      b.count("| FindFrameKind::AngryBonesBig\n") == 2
      and b.count("| FindFrameKind::AngryBonesBigHelmet\n") == 2)
check("S1 wave-B vanilla_type rows 294/296",
      "Self::AngryBonesBig => 294," in b and "Self::AngryBonesBigHelmet => 296," in b)
check("S1 wave-B frame_count 16/15 (Main.cs:67113)",
      "Self::AngryBonesBig => 16," in b and "Self::AngryBonesBigHelmet => 15," in b)
check("S1 humanoid_fighter_frame pub(crate) (the delegate seam)",
      "pub(crate) fn humanoid_fighter_frame(" in b)
# the hook insertion boundary: after the extra8 world-free block, before num2
ff = npc_d44[npc_d44.find("pub(crate) fn find_frame(&mut self)"):]
h8 = ff.find("find_frame_kind_extra8_claimed(self.kind.cs_id())")
num2 = ff.find("let num2 = if self.ai_action == 0")
check("S1 extra8 hook precedes the num2 body (the insertion gap)", 0 < h8 < num2)
base = npc_d44.find("pub(crate) fn find_frame(&mut self)")
check("S1 insertion boundary lines",
      line_of(npc_d44, "let num2 = if self.ai_action == 0") == 7198,
      f"num2 at {line_of(npc_d44, 'let num2 = if self.ai_action == 0')}")
check("S1 find_frame_world falls through to find_frame (no wave-B world arm needed)",
      "self.find_frame();\n    }" in npc_d44)
# our branch: the cs_id resolution path (NO hand row — the ENTRIES fallback)
data = read("crates/tr-app/src/npc/data.rs")
best = read("crates/tr-app/src/bestiary.rs")
npc_ours = read("crates/tr-app/src/npc.rs")
cs_seg = data[data.find("pub(crate) fn cs_id(self)"):]
cs_seg = cs_seg[:cs_seg.find("\n    }")]
check("S1 cs_id has NO 294/296 hand row (the fallback resolves them)",
      "NpcKind::AngryBonesBig =>" not in cs_seg
      and "NpcKind::AngryBonesBigHelmet =>" not in cs_seg)
check("S1 cs_id fallback reads try_entry_for_npc net_id (data.rs:3363-3370)",
      "match crate::bestiary::try_entry_for_npc(self, SlimeVariant::Blue)" in cs_seg)
check("S1 ENTRIES rows 294/296 exist (the fallback's source)",
      "net_id: 294," in best and "net_id: 296," in best,
      f"lines {line_of(best, 'net_id: 294,')}/{line_of(best, 'net_id: 296,')}")
check("S1 to_u16 hand rows 294/296 (protocol unchanged)",
      "NpcKind::AngryBonesBig => 294," in data
      and "NpcKind::AngryBonesBigHelmet => 296," in data)
check("S1 the stand-in arms at npc.rs:9514-9519",
      line_of(npc_ours, "NpcKind::AngryBonesBig => self.humanoid_town_frame(),") == 9515
      and line_of(npc_ours, "NpcKind::AngryBonesBigHelmet => self.humanoid_town_frame(),") == 9519)
check("S1 our humanoid_frame_count 16/15 agrees with the module",
      "NpcKind::AngryBonesBig => 16," in data
      and "NpcKind::AngryBonesBigHelmet => 15," in data)
check("S1 extra_frames_count 0 for both (the full-count wrap)",
      "AngryBonesBig" not in data[data.find("pub fn extra_frames_count"):
                                   data.find("pub fn behind_tiles")])
check("S1 the FINAL stand-in witness pins 294/296 (the re-pin target)",
      "w4_d38_final_the_seven_town_standins_are_the_wiring_gap" in npc_ours)

# ---------------- S2: the 328 guard consumer census ----------------
sim = read("crates/tr-app/src/world_sim/npcs.rs")
check("S2 entry_for_npc ends in .expect (the panic)",
      '.expect("every ported NPC kind has a bestiary entry")' in best,
      f"line {line_of(best, '.expect(')}")
check("S2 kill-credit loop :5283 rides entry_for_npc",
      line_of(sim, "let entry = crate::bestiary::entry_for_npc(npc.kind, npc.slime);") == 5283)
check("S2 spawn_loot :6840 ALSO rides entry_for_npc (the second panic site)",
      line_of(sim, "let entry = crate::bestiary::entry_for_npc(npc.kind, npc.slime);",
              ) == 5283 and sim.count("crate::bestiary::entry_for_npc(npc.kind, npc.slime)") == 2)
check("S2 net_id() rides entry_for_npc (data.rs:1270-1272 — draw/save/lang/clicks)",
      "crate::bestiary::entry_for_npc(self, SlimeVariant::Blue).net_id" in data)
ent = read("crates/tr-app/src/render/entities.rs")
check("S2 texture_net_id fallback rides kind.net_id() (entities.rs:1234)",
      "_ => kind.net_id() as u16," in ent)
check("S2 the blade draw arm rides texture_net_id (entities.rs:3371-3373)",
      "NpcKind::PumpkingBlade => {\n                let ty = texture_net_id(n.kind);" in ent)
s59 = read("crates/tr-app/src/npc_ai_style59.rs")
check("S2 the blade's silent despawn sets hp = 0 (style59:255)",
      "npc.hp = 0;" in s59)
nm = read("crates/tr-app/src/app/netmodules.rs")
check("S2 client kill arm gates on kind_from_net_id (netmodules.rs:457-460)",
      "let Some(kind) = crate::npc::kind_from_net_id(bk.npc_net_id as i32) else {" in nm)
check("S2 kind_from_net_id reads ENTRIES (None for 328 post-drop)",
      "crate::bestiary::ENTRIES\n        .iter()\n        .find(|e| e.net_id == net_id)" in data)
check("S2 cs_id fallback would sentinel 328 post-drop (the hand-row need)",
      "_ => u16::MAX," in cs_seg)
# the fold-table asymmetry: a bare try_entry_for_npc swap is WRONG
try_seg = best[best.find("pub fn try_entry_for_npc"):best.find("pub fn entry_for_npc")]
ent_seg = best[best.find("pub fn entry_for_npc"):]
check("S2 try_entry_for_npc LACKS the town-sleeper folds (bare swap loses credits)",
      "NpcKind::SleepingAngler => NpcKind::Angler" in ent_seg
      and "NpcKind::SleepingAngler" not in try_seg
      and "NpcKind::ForceBubble => NpcKind::MartianOfficer" in ent_seg
      and "NpcKind::ForceBubble" not in try_seg)
# the vanilla credit chain (the guard's faithfulness source)
npc_cs = read("decompiled/Terraria/NPC.cs")
check("S2 GetBestiaryCreditId reads ContentSamples (NPC.cs:95139-95142)",
      "return ContentSamples.NpcBestiaryCreditIdsByNpcNetIds[this.netID];" in npc_cs)
cs_samples = read("decompiled/Terraria/ID/ContentSamples.cs")
check("S2 the credit id is the vanilla NAME (ContentSamples.cs:868-870)",
      "string name = NPCID.Search.GetName(nPC.netID);" in cs_samples
      and "ContentSamples.NpcBestiaryCreditIdsByNpcNetIds[i] = name;" in cs_samples)
nbm = read("decompiled/Terraria/GameContent/NetModules/NetBestiaryModule.cs")
check("S2 the client counts via ContentSamples.NpcsByNetId (NetBestiaryModule.cs:52)",
      "ContentSamples.NpcsByNetId[key3].GetBestiaryCreditId()" in nbm)
kt = read("decompiled/Terraria/GameContent/Bestiary/NPCKillsTracker.cs")
check("S2 RegisterKill needs no entry (NPCKillsTracker.cs:23-33)",
      "string bestiaryCreditId = npc.GetBestiaryCreditId();" in kt)
npcid = read("decompiled/Terraria/ID/NPCID.cs")
check("S2 NPCID.PumpkingBlade = 328 (the name row)",
      "public const short PumpkingBlade = 328;" in npcid)
check("S2 no 328 fold in entry_for_npc today",
      "NpcKind::PumpkingBlade =>" not in ent_seg.split(".expect")[0])

# ---------------- S3: the c8 GAP-1 receipt at 7c2de9c06 ----------------
C8 = "7c2de9c06"  # c8d-crawler-retire-active (origin == local)
c8b = git_show(C8, "crates/tr-app/src/bestiary.rs")
check("S3 generic gate at the retire tip (bestiary.rs:2280)",
      line_of(c8b, "fn counts_as_a_critter(kind: NpcKind) -> bool {") == 2280)
check("S3 gate reads max_hp/contact_damage/594/686",
      re.search(r"kind\.max_hp\(\) <= 5\s*&& kind\.contact_damage\(\) == 0\s*&& kind\.cs_id\(\) != 594\s*&& kind\.cs_id\(\) != 686", c8b) is not None)
check("S3 is_critter_entry = gate + the 549 Hide exclusion (:2303)",
      line_of(c8b, "entry.kind != NpcKind::DD2LanePortal && counts_as_a_critter(entry.kind)") == 2304)
check("S3 unlock_state_world dispatch (:2412)",
      line_of(c8b, "pub fn unlock_state_world(entry: &BestiaryEntryDef, bestiary: &WorldBestiary)") == 2412)
check("S3 completion_percent_world (:2449)",
      "pub fn completion_percent_world(bestiary: &WorldBestiary) -> f32 {" in c8b)
check("S3 the scan at the update_npcs head (world_sim/npcs.rs:3950)",
      line_of(git_show(C8, "crates/tr-app/src/world_sim/npcs.rs"),
              "crate::bestiary::scan_world_for_critter_sights(app);") == 3950)
# the six critters pass on their SetDefaults rows (the R3 extraction)
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
cs = {k: int(v) for k, v in re.findall(r"NpcKind::(\w+) => (\d+),", npc_ours)}
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
check("S3 exactly the SIX d38 critters pass the generic gate",
      passing == SIX, str(sorted(passing)))
# our 3 kill-only UI call sites + the plumbing the fold touches
uib = read("crates/tr-app/src/ui/bestiary.rs")
sites = [m.start() for m in re.finditer(r"crate::bestiary::unlock_state\(entry, tracker\)", uib)]
check("S3 exactly 3 kill-only unlock_state call sites",
      len(sites) == 3,
      str([uib[:s].count("\n") + 1 for s in sites]))
check("S3 the sites are :510/:537/:1803",
      [uib[:s].count("\n") + 1 for s in sites] == [510, 537, 1803])
check("S3 the KillTracker params :363/:476/:532/:1797",
      [m.start() for m in re.finditer(r"tracker: &KillTracker,", uib)]
      and [uib[:m.start()].count("\n") + 1
           for m in re.finditer(r"tracker: &KillTracker,", uib)] == [363, 476, 532, 1797])
check("S3 completion_percent kill-only at :412",
      line_of(uib, "let pct = crate::bestiary::completion_percent(tracker);") == 412)
check("S3 kill_progress rides the bare tracker at :1926",
      line_of(uib, "let (kills, _needed) = crate::bestiary::kill_progress(entry, tracker);") == 1926)
uim = read("crates/tr-app/src/ui/mod.rs")
check("S3 ui::draw kill_tracker param :683",
      line_of(uim, "kill_tracker: &tr_core::bestiary::KillTracker,") == 683)
dr = read("crates/tr-app/src/app/frame/draw.rs")
check("S3 the caller passes &self.bestiary.kills (draw.rs:1684)",
      line_of(dr, "&self.bestiary.kills,") == 1684)
check("S3 app.bestiary is ALREADY WorldBestiary (app/mod.rs:276)",
      "pub(crate) bestiary: tr_core::bestiary::WorldBestiary," in read("crates/tr-app/src/app/mod.rs"))
# c8's own folded form (the precedent the fold copies)
c8ui = git_show(C8, "crates/tr-app/src/ui/bestiary.rs")
check("S3 c8 folded its 3 UI sites onto unlock_state_world",
      c8ui.count("crate::bestiary::unlock_state_world(entry, tracker)") == 3)
check("S3 c8 folded completion + kill_progress + the caller",
      "completion_percent_world(tracker)" in c8ui
      and "kill_progress(entry, &tracker.kills)" in c8ui
      and "&self.bestiary," in git_show(C8, "crates/tr-app/src/app/frame/draw.rs"))
# the e23 static-list delta (the compose hazard)
E23 = "7a7779690"
e23d = git_show(E23, "crates/tr-app/src/bestiary_data.rs")
check("S3 e23 CRITTER_NET_IDS is the STATIC [55,549] list (bestiary_data.rs:82)",
      "const CRITTER_NET_IDS: &[i32] = &[55, 549];" in e23d)
check("S3 e23 gate dispatches the list (bestiary_data.rs:227)",
      "if CRITTER_NET_IDS.contains(&net_id) {" in e23d)
check("S3 e23 unlock_state already takes WorldBestiary (the c37 shape)",
      "pub fn unlock_state(entry: &BestiaryEntryDef, tracker: &WorldBestiary) -> UnlockState {"
      in git_show(E23, "crates/tr-app/src/bestiary.rs"))
check("S3 none of the six is in e23's static list (they would ride the kill ladder)",
      all(str(cs[k]) not in ("55", "549") for k in SIX))

# the S2 recommendation (b) safety sweep: no 328 in the cs_id-keyed tables
fb = read("crates/tr-app/src/npc_ai_fighter_scan.rs")
door_seg = fb[fb.find("const DOOR_BASH_TYPES"):fb.find("];", fb.find("const DOOR_BASH_TYPES"))]
check("S2 DOOR_BASH_TYPES has no 328 (the cs_id hand row changes nothing)",
      "328" not in door_seg)
t3 = read("crates/tr-app/src/npc_ai3_speeds.rs")
check("S2 the ai3 ladder tables carry no 328 row (style-59 never rides them)",
      not re.search(r"^\s*328 =>", t3, re.M))
bn = read("crates/tr-app/src/bug_net.rs")
check("S2 the catch tables carry no 328 (the blade is not catchable)",
      not re.search(r"^\s*328 =>", bn, re.M))
chat = read("crates/tr-app/src/npc_chat.rs")
check("S2 the chat cs_id match carries no 328",
      not re.search(r"^\s*328 =>", chat, re.M))
check("S2 npc_target no_aggro rides the cs_id index (the row keeps it in range)",
      "let id = kind.cs_id() as usize;" in read("crates/tr-app/src/npc_target.rs"))

# ---------------- S4: the d35 atlas-lazy compose ----------------
D35 = "1afd327b9"
union = json.loads(git_show(D35, "tools/_d35_union_list.json"))
check("S4 the union is 562 names", len(union) == 562, str(len(union)))
d38rows = [r for r in union if any("d38" in o for o in r["owners"])]
D38_IDS = [281,282,283,284,285,286,287,289,290,291,292,293,294,295,296,
           297,298,299,300,301,302,303,304,316,317,318,322,323,324,328,
           333,334,335,336,337]
want = {f"NPC_{i}" for i in D38_IDS}
check("S4 all 35 d38 sheets present in the union",
      {r["name"] for r in d38rows} == want and len(d38rows) == 35)
check("S4 every d38 row missing=False (the real dims measured)",
      all(not r["missing"] for r in d38rows))
shared = sorted(r["name"] for r in d38rows if len(r["owners"]) > 1)
check("S4 the shared-owner rows are exactly NPC_289/NPC_328 (the E3 dedupes)",
      shared == ["NPC_289", "NPC_328"], str(shared))
padded = sum(r["padded_area"] for r in d38rows)
check("S4 the 35-sheet padded contribution 717824 px", padded == 717824, str(padded))
# the union dims == the §4b strips witness row-for-row
u = {r["name"]: r for r in union}
uib0 = read("crates/tr-app/src/ui/bestiary.rs")
i0 = uib0.find("(287, 44, 896, 16)")
seg0 = uib0[uib0.rfind("[", 0, i0):uib0.find("];", i0)]
strips = re.findall(r"\((\d+), (\d+), (\d+), (\d+)\)", seg0)
bad = [(n, w, h) for n, w, h, _ in strips
       if u.get("NPC_" + n) is None
       or (u["NPC_" + n]["w"], u["NPC_" + n]["h"]) != (int(w), int(h))]
check("S4 the union dims == the 35-row strips witness (zero mismatch)",
      len(strips) == 35 and not bad, str(bad))
assets = read("crates/tr-app/src/assets.rs")
check("S4 all 35 in STATIC_SHEET_NAMES (the eager superset until step 7)",
      all(f'"NPC_{x}",' in assets for x in D38_IDS))
d35doc = git_show(D35, "docs/research/113-atlas-repack-plan.md")
check("S4 d35 §16.1 cross-receipts d38d at 1ba0395c7",
      "### 16.1 d38d (`task/W4-d38-npc-kinds-281-350`, tip `1ba0395c7`)" in d35doc)
check("S4 the 328 card drop does NOT touch NPC_328's STATIC row (§12 item 7)",
      '"NPC_328",' in assets)

# ---------------- S5: the §13b SEAL re-receipt ----------------
doc = read("docs/research/125-d38-crossnote-atlas-addendum.md")
check("S5 §13b SEAL declaration present", "### 13b. The SEAL declaration" in doc)
i = assets.find("pub const BESTIARY_NPC_SHEET_IDS")
seg = assets[i:assets.find("];", i)]
declared = int(re.search(r"\[u16; (\d+)\]", seg).group(1))
ids = [int(x) for x in re.findall(r"\d+", re.sub(r"//[^\n]*", "", seg).split("[", 2)[2])]
i = assets.find("bestiary_npc_sheets: vec![")
vec = assets[i:assets.find("],", i)]
check("S5 slots declared==ids==vec == 216", declared == len(ids) == vec.count("atlas.sheet") == 216)
check("S5 35 d38 ENTRIES rows",
      len(re.findall(r"// W4-d38 b\d — BestiaryEntry\.", best)) == 35)
check("S5 the RETIRE receipt 44/44 stands (committed output)",
      "ALL PASS" in read("tools/_d38_retire_check.txt"))

print()
print("ALL PASS" if ok else "FAILURES PRESENT")
sys.exit(0 if ok else 1)
