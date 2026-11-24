# 229 — W4-d36 NPC kinds 141-210: census, claims, id-band plan

> Renumbered 109 -> 229 at integration/plan-B22 merge per the collision registry (150-research-number-registry, sec 3/4, d54 tip); winner keeps 109.

Status: CENSUS + PLAN (research phase; the port batches follow).
Branch: `task/W4-d36x-npc-kinds-141-210-exec` (isolated executor worktree
`TR-W4-d36x`, base `0eb4fee9` = the d36 claim+census tip), because the
original `TR-W4-d36` worktree was being written concurrently by another
d36 session (three foreign in-flight states preserved as stashes there).
All numbers re-derived by script from the working tree + `git show` of the
pushed tips (no compiler invoked; code-only mode).

## 0b. E4 RE-PIN (authoritative — supersedes §3's 731-768 / 38-kind plan)

The d37 reconcile EXTEND (`bdd59afba`, research/108 §E4) re-scanned **all
354** pushed `task/W4-*` refs (the §0 six-tip read missed 13 claimants) and
re-pinned the unpushed lanes: the canonical small-lane band is **731-761**
(c10/c7/c8/c9/d125/d23/e26/g8), so **d36 moves to 762-811** (above the
canonical max 761). A fresh scripted claim scan of every pushed tip's
`NpcKind` enum + `cs_id` + bestiary rows (this session, post-fetch) confirms
**21** of the 49 missing band ids are claimed elsewhere — the §2 list of 11
was incomplete (it predated the g8 batch-3/4, d120 and g5 pushes):

- g8 (`82c93cec`, batches 2-4): 150 IceBat, 151 Lavabat, 152 GiantFlyingFox,
  156 RedDevil, 158 VampireBat, 164 WallCreeper, 165 WallCreeperWall,
  169 IceElemental, 174 Herpling, 177 Derpling, 179 CrimsonAxe,
  182 FloatyGross, 195 LostGirl, 196 Nymph (14).
- d120 (town-chatter): 142 SantaClaus, 160 Truffle, 178 Steampunker,
  209 Cyborg (4).
- c4 (`22cec16f`): 157 Arapaima. d23 (`19109424`): 175 AngryTrapper.
  g5 (style7-caster-doc): 172 RuneWizard (3).

**28 kinds remain for d36** (proto **762-789**, ascending vanilla-id order;
render_key **550-577**, the base render_key match tops out at 549 and the
550+ band is free — script-verified): 141 ToxicSludge, 148 Penguin,
149 PenguinBlack, 153 GiantTortoise, 154 IceTortoise, 155 Wolf, 159 Vampire,
162 Frankenstein, 163 BlackRecluse, 166 SwampThing, 167 UndeadViking,
168 CorruptPenguin, 170 PigronCorruption, 171 PigronHallow, 173 Crimera,
176 MossHornet, 180 PigronCrimson, 181 FaceMonster, 185 SnowFlinx,
197 ArmoredViking, 198 Lihzahrd, 199 LihzahrdCrawler, 201 HeadacheSkeleton,
202 MisassembledSkeleton, 203 PantlessSkeleton, 204 SpikedJungleSlime,
205 Moth, 206 IcyMerman.

Bestiary folds among the 28 (ContentSamples.cs:1036/1052/1053):
- 163 BlackRecluse → 238 BlackRecluseWall (d37's kind, NOT on this branch):
  the fold target is absent, so 163 keeps a hand `cs_id` 163 and NO own
  bestiary entry on this branch (the s1_f_3 sentinel path); the fold arm
  lands at merge when d37's 238 row exists. `net_id()` reads need the
  sentinel-safe path (the kind is not spawnable from a bestiary row here).
- 198 Lihzahrd → 199 LihzahrdCrawler (both on this branch): 199 keeps its
  OWN entry (net 199, the credit owner — the populator adds Temple info to
  199 at :1395 and 198 folds onto it), 198 folds onto 199's credit. Both
  keep their own cs_id + draw sheet (the d37 GoldfishWalker precedent).
- 159 Vampire: own entry (158 VampireBat folds onto it, but 158 is g8's).
- 201/202/203: own entries — the fold TARGETS of d40's 450/451/452
  (cross-note: this wave UNBLOCKS those d40 folds).

Count pins (this branch, post-wave): ALL_KINDS 270→298; to_u16/from_u16
298 arms bijection; s1_f_3 covered+sentinel 298; ENTRIES 241→267 (+26:
28 kinds − the 163 no-entry − the 198 fold); BESTIARY_NPC_SHEET_IDS
181→207 (+26 visible); D36_SHEETS 28 draw rows (163/198 keep own sheets).
The §3/§9 numbers below (731-768 / 38 kinds / 308) are SUPERSEDED by this
section; they are kept as the historical first-pass record.

## E4 BATCH 2 LANDED — the fighters + the style-5 tail wiring (this pass)

The R7-band ascending slots after the Wolf (772) and the concurrent
writer's Frankenstein (773, tip `85ee24abb`): **163 BlackRecluse → 774,
166 SwampThing → 775, 167 UndeadViking → 776**. (159 Vampire is g8-claimed
— their pushed tip's `Vampire => 388`, machine round 3 — skipped; the
brief's "159-ALREADY-g8" confirmed against `origin/task/W4-g8-npc-kind-wire-1`.)

1. **The three fighters** ride the aiStyle-3 walker family on the ladder's
   per-type rows (163 the cap-2 group `NPC.cs:57554`, 166 the cap-3 group
   `:57520` + its own `:55938-55971` dwell pre-arm, 167 the num83 base
   `:57606`). FindFrame: 163 case `:73227-73267` (the -12 air counter, the
   |vx|*1.1 bands), 166 case `:75618-75664` (the idle 0..5 / walk 7..12 /
   air 6 bands), 167 the humanoid-group `:68572` (air row 0 — `:69647`
   lists 167). All 12 hard sites + the wildcard rows + the sheets/portraits.
2. **The 163 bestiary fold**: credit → 238 BlackRecluseWall
   (`ContentSamples.cs:1036`, Hide=true GetLeinforsEntries); the row is
   emitted at net 238 keyed on the wall crawler's credit (d37's kind is
   not on this branch), the DRAW sheet stays 163 (`texture_net_id`'s own
   row, the Sharkron 372 precedent). The portrait samples NPC_238's 4-row
   strip. Immunities: 163/166 {20}, 167 {20,44,324} (`NPCID.cs:7074`/
   `:7095`/`:7102`); cues NPC_Hit_29/NPC_Killed_32 (163), NPC_Hit_2/
   NPC_Killed_2 (167), NPC_Hit_1/NPC_Killed_2 (166).
3. **The style-5 tail WIRED** (the brief's item 4): the concurrent writer's
   `npc_ai_style5::style5_tail` module (research/110 §3, the more complete
   port — the Hornet 42/231-235 family + getGoodWorld third draw + the
   integer-half anchor + the stealth gate) is now called from
   `ai_005_hover_family` (the rng thread, the `ai_009_sphere_family`
   signature precedent); the shot drains to `wants_hostile_shot` (the sim
   timed list's `timeLeft = 300`), the Item17 cue to `wants_item_sounds` →
   the sim drain (the style75 `out.sounds` shape). The frozen A/B
   (`s1_d_2_the_style_axis_keeps_the_hover_behaviour`) legacy twin calls
   the tail too, so the axis-vs-kind-dispatch equivalence holds (the
   Hornet 42's charge rolls + volley are the NEW behaviour both share).
4. **Pins after batch 2:** ALL_KINDS/to/from/s1_f_3 **283**; ENTRIES
   **254 numeric-net rows** (+3: 238/166/167); BESTIARY_NPC_SHEET_IDS
   **194**; D36_SHEETS 13 rows; bijection ALL PINS HOLD (283/283, max id
   793). Witnesses WRITTEN NOT RUN (code-only):
   `w4_d36_fighters_the_defaults_match_the_csharp_rows`,
   `w4_d36_fighters_the_frame_clocks_match_the_csharp_cases` (+ the
   style5 module's own 7). **Kinds remaining: 15** (CorruptPenguin 168,
   the pigron trio 170/171/180, FaceMonster 181, SnowFlinx 185,
   ArmoredViking 197, Lihzahrd 198, LihzahrdCrawler 199, the bone trio
   201/202/203, SpikedJungleSlime 204, IcyMerman 206).
5. **The writer race** (fourth recurrence): the concurrent E4 writer
   landed Wolf@772 (`a11f61850`) + Frankenstein@773 (`85ee24abb`) and
   preserved this pass's earlier 4-kind WIP as
   `tools/_d36_inflight_batch2{,b,c}.patch` (NOT landed). This pass kept
   their Frankenstein, added the three remaining fighters + wired their
   style5 module (the disjoint item-4 seam they left for the walker
   batch's completion). No stash touched.

## SUPERSEDED AGAIN — d37-R7 re-pin (research/108 `a341daa12`, latest)

The d37 RECEIPTS pass re-pins d36 to **767-816** (R7): the R5/R6 moves
(c4 Arapaima 237→**762**, d120 towns 241-244→**763-766**) extend the
canonical max to 766, so the E4 762-811 band's head is occupied. Executed
on this branch:

1. **Batch 0 renumbered 762-766 → 767-771** (ToxicSludge 767, Penguin
   768, PenguinBlack 769, GiantTortoise 770, IceTortoise 771). Only the
   `to_u16`/`from_u16` arms moved — `cs_id`, ENTRIES `net_id`, sheets,
   render keys untouched (the E8 seam rule).
2. **The hover batch landed** (the in-flight stash@{0}'s unique work,
   COMPLETED): 173 Crimera → **781**, 176 MossHornet → **782**, 205 Moth
   → **793** (the R7-band ascending-vanilla slots). The stash shipped
   HoverParams presets + family arms WITHOUT the enum variants (never
   compiled); this pass adds the variants + all hard/wildcard sites and
   FIXES two preset bugs against the C#:
   - MOSS_HORNET `double_add` false→**true** (`NPC.cs:50447`'s flag5
     exclusion set {173,6,42,231-235,94,139,619} does NOT list 176);
   - CRIMERA `no_tile_collide` true→**false** (SetDefaults case 173
     `:11168-11193` writes noGravity ONLY; the ×0.4 bounce `:50659-50661`
     exists for the colliding 6/173 pair). The base EATER_OF_SOULS row's
     `no_tile_collide: true` claim is likewise unsupported by case 6
     (`:8733-8762`) — left untouched (frozen behaviour), flagged for the
     Lead.
   - The MossHornet stinger volley (`:50754-50810`, proj 55 dmg 30) and
     the Moth wet-arm (`:50726-50738`) are the documented remainders
     (the hostile-shot seam, the Probe precedent).
3. **Pins after the hover batch:** ALL_KINDS/to/from/s1_f_3 **278**;
   ENTRIES **249 numeric-net rows** (+3, no folds in this batch);
   BESTIARY_NPC_SHEET_IDS **189**; D36_SHEETS 8 rows; bijection script
   ALL PINS HOLD (278/278, max id 793). Witnesses WRITTEN NOT RUN
   (code-only): `w4_d36_hover_the_defaults_match_the_csharp_rows`,
   `w4_d36_hover_the_frame_clocks_match_the_csharp_cases`, the extended
   `the_hover_presets_match_the_csharp_table` (7 rows + the gate pins).
4. **A FOREIGN writer raced this worktree again** (npc.rs batch-1 WIP:
   Wolf/Vampire/Frankenstein/BlackRecluse/SwampThing enum + stat arms,
   NO data.rs arms, comment claims "E4 band 767-771" — which under R7
   belongs to batch 0). Preserved verbatim as stash `d36-inflight-4`,
   NOT landed; its band claim needs Lead adjudication against R7.
Branch: `task/W4-d36-npc-kinds-141-210`, base `a725acb0` (claim `65e481f9`).
All numbers re-derived by script from the working tree + `git show` of the
pushed tips (no compiler invoked; code-only mode).

## SUPERSEDED IN PART — Lead ruling (d37c research/108-E `bdd59afb`)

The 108-E full-wave re-census (18 pushed claimants, not the 6-tip read this
doc's §2 scan predates) is AUTHORITATIVE where it conflicts:

1. **Protocol band: 762-811, NOT §3's 731-768.** 108-E4 re-pins d36 to
   **762-811** because the canonical small-lane shifts occupy 731-761
   (g8's 17 owned kinds → 745-761 alone collide with §3's whole 731-768
   plan). 28 kinds ride 762-789 contiguous, ascending vanilla order.
2. **Skip-list: 21 claimed, NOT §2's 11 → this wave ports 28, NOT 38.**
   The scan missed: g8 batch-1's 150 IceBat / 151 Lavabat / 152
   GiantFlyingFox / 156 RedDevil / 158 VampireBat (108-E2 item 4 lists all
   five in g8's 17-kind owned set → 745-761); d120's 142 SantaClaus /
   160 Truffle / 178 Steampunker / 209 Cyborg (108-E0: proto 241-244,
   kept vanilla-free); g5's **172 RuneWizard** (pushed tip `feb78f109`
   "wire the two unclaimed aiStyle-8 casters", `RuneWizard => 354` —
   not in 108-E's tables but a pushed kind-port; ownership rule E3
   gives it to the pushed lane). 14 g8 + 4 d120 + c4-157 + d23-175 +
   g5-172 = 21 skips; 49 missing − 21 = **28**.
3. **The 28-kind list** (§2's 38 minus the 10 newly-found claims):
   141 ToxicSludge, 148 Penguin, 149 PenguinBlack, 153 GiantTortoise,
   154 IceTortoise, 155 Wolf, 159 Vampire, 162 Frankenstein,
   163 BlackRecluse, 166 SwampThing, 167 UndeadViking, 168 CorruptPenguin,
   170 PigronCorruption, 171 PigronHallow, 173 Crimera, 176 MossHornet,
   180 PigronCrimson, 181 FaceMonster, 185 SnowFlinx, 197 ArmoredViking,
   198 Lihzahrd, 199 LihzahrdCrawler, 201 HeadacheSkeleton,
   202 MisassembledSkeleton, 203 PantlessSkeleton, 204 SpikedJungleSlime,
   205 Moth, 206 IcyMerman → proto **762-789**.
4. **Count pins (§9) recompute:** ALL_KINDS 270→298; to/from_u16 298 arms;
   s1_f_3 298; ENTRIES 241→268 (+27: 28 kinds − 1 fold 198→199; 158 is
   skipped so its fold row is g8's problem); BESTIARY_NPC_SHEET_IDS
   181→208 (+27); D36_SHEETS 28 rows (the 198 fold keeps its own sheet).
   §1's census, §4-§8's folds/immunities/frames/AI-routing stand for the
   28 kept kinds (the skipped rows are their owners' now).

## 0. Foreign-writer incident (preserved, not discarded)

At 08:07 local the worktree carried **uncommitted W4-d36 edits this session
did not make** (a parallel d36 agent writing the same directory): the first
five kinds (ToxicSludge/SantaClaus/Penguin/PenguinBlack/IceBat) on a
**354-358 protocol band** — which collides with d37's pushed 354-399 band
(research/108 §2a) and with g8's pushed 364-374. Preserved verbatim as
`stash@{0}` ("d36-inflight: FOREIGN parallel d36 agent …"), the
`stash@{1}` d52-inflight precedent. This session proceeds on the clean
`65e481f9` claim with the research/108-compliant band (§3).

## 1. Census (NPC.cs SetDefaults 141-210 vs the base roster)

`SetDefaults` (`NPC.cs:8426-17859`) carries **69** type arms in 141-210
(the 69 `else if (this.type == N)` blocks; no range arms in the band).
Base coverage = the `cs_id` hand rows (`npc/data.rs:2992`) ∪ the bestiary
`ENTRIES` net ids (`bestiary.rs`, the `kind:` field's kind):

- covered (20): 143 SnowmanGangsta, 144 SnowBalla (credit "MisterStabby"),
  145 SnowBalla/MysterySnowman, 147 IceSlime, 161 ZombieEskimo,
  183 Crimslime, 184 SpikedIceSlime, 186-189 the four armed-zombie twins,
  190-194 the five eye variants, 200 FemaleZombie, 207 DyeTrader,
  208 PartyGirl, 210 Bee.
- **missing: 49** — 141, 142, 148-160, 162-179, 180-182, 185, 195-199,
  201-206, 209 (the exact list with NPCID names + SetDefaults line spans
  in `tools/_d36_census.json`).

This matches research/108 §2c's "d36 141-210: 50" ±1 (their count included
one bestiary-net-only row; the cs_id ∪ net union here is the precise 49).

## 2. Claim scan (every pushed tip, scripted)

A strict scan of all **352** pushed `origin/task/*` tips (enum-variant diff
vs base + `cs_id` hand rows + bestiary `kind:`/`net_id` rows) found these
band claims:

| id | kind | owner tip | evidence |
|---|---|---|---|
| 157 | Arapaima | `W4-c4-style44-flying-fish` `22cec16f` (phase 3+4) | cs_id 157, bestiary net 157, proto 237 |
| 164 | WallCreeper | `W4-g8-npc-kind-wire-1` `0a4b76e3` (batch 2) | cs_id 164, proto 366 |
| 165 | WallCreeperWall | g8 batch 2 | cs_id 165, proto 367 |
| 169 | IceElemental | g8 batch 2 | cs_id 169, proto 368 |
| 174 | Herpling | g8 batch 2 | cs_id 174, proto 369 |
| 175 | AngryTrapper | `W4-d23-npc-ai-styles-9-14` `19109424` | bestiary net 175, proto 357 |
| 177 | Derpling | g8 batch 2 | cs_id 177, proto 371 |
| 179 | CrimsonAxe | g8 batch 2 | cs_id 179, proto 372 |
| 182 | FloatyGross | g8 batch 2 | cs_id 182, proto 373 |
| 195 | LostGirl | g8 batch 2 | cs_id 195, proto 374 |
| 196 | Nymph | g8 batch 2 | cs_id 196, proto 374-fold |

**11 claimed → SKIP.** The brief named 157 (c4b) + 175 (d23b); the scan
adds g8's nine (the brief's "g9 kinds 315/329/…" are out-of-band; g8's
batch-2 nine are IN-band and were not in the brief — the script-check the
brief demanded caught them). d35's 71-140 and d37's 211-280 bands do not
touch 141-210. The 201/202/203 bone-thrower fold TARGETS are unclaimed
(d40's 450/451/452 fold ONTO them — `ContentSamples.cs:1040-1042`); this
wave ports them and UNBLOCKS those folds (cross-note to d40).

**This wave ports 38 kinds:**
141 ToxicSludge, 142 SantaClaus, 148 Penguin, 149 PenguinBlack, 150 IceBat,
151 Lavabat, 152 GiantFlyingFox, 153 GiantTortoise, 154 IceTortoise,
155 Wolf, 156 RedDevil, 158 VampireBat, 159 Vampire, 160 Truffle,
162 Frankenstein, 163 BlackRecluse, 166 SwampThing, 167 UndeadViking,
168 CorruptPenguin, 170 PigronCorruption, 171 PigronHallow, 172 RuneWizard,
173 Crimera, 176 MossHornet, 178 Steampunker, 180 PigronCrimson,
181 FaceMonster, 185 SnowFlinx, 197 ArmoredViking, 198 Lihzahrd,
199 LihzahrdCrawler, 201 HeadacheSkeleton, 202 MisassembledSkeleton,
203 PantlessSkeleton, 204 SpikedJungleSlime, 205 Moth, 206 IcyMerman,
209 Cyborg.

## 3. Protocol band: 731-768 port-local (research/108 §3)

The vanilla 141-210 corridor is **densely base-owned in protocol space**
(base `to_u16` carries all of 141-210 — the DD2/martian/town port-local
bands; script-verified). research/108 §3 assigns future-d36 **731-780**;
this wave's 38 kinds take **731-768** contiguous, ascending in vanilla-id
order (ToxicSludge 731 … Cyborg 768). `cs_id`/bestiary `net_id` carry the
vanilla type. Free on every pushed tip (max proto: base 353, d34 275,
d37 399, d38 337, d40 488, d35 461→698-730 per the plan; g8 374). i16-safe
(768 ≪ 32767). `render_key` rides the protocol id (the d37 precedent) —
injective, no collision with the base's cs-space render keys.

## 4. Bestiary folds (the Hide=true rows, ContentSamples.cs)

- 158 VampireBat → 159 Vampire (`ContentSamples.cs:1053`, Hide
  `NPCID.cs` GetRedigitEntries) — fold, own cs_id + draw sheet.
- 163 BlackRecluse → 238 BlackRecluseWall (`:1036`, Hide GetLeinforsEntries).
  238 is d37's kind (unpushed-to-main); the fold target row is added by
  this wave as a **net-238 entry keyed on the wall crawler's credit** only
  if d37's kind is absent — see the batch-4 note (the port keeps the fold
  arm pointing at the d37 kind name once merged; on this branch the entry
  is emitted with `kind: NpcKind::BlackRecluse` folded to a 238-net row).
- 199 LihzahrdCrawler → own entry (Hide=true BUT the populator registers
  199's info at `BestiaryDatabaseNPCsPopulator.cs:1395` and 198 folds onto
  199 at `ContentSamples.cs:1052` — the crawler is the CREDIT owner).
  So **198 Lihzahrd folds onto 199 LihzahrdCrawler**, the reverse of the
  usual head-first shape; 199 keeps its own entry despite Hide=true (the
  Hide flag suppresses the automated row, the manual credit fold makes
  199 the visible credit — the Sharkron 372 precedent).
- 201/202/203 HeadacheSkeleton/MisassembledSkeleton/PantlessSkeleton:
  own entries (the populator adds Caverns info to each,
  `:1469-1471`); they are the fold TARGETS of d40's 450/451/452.
- The town NPCs 142/160/178/209: `BestiaryEntry.TownNPC` rows
  (`:815-820`), own entries.

## 5. DebuffImmunitySets (NPCID.cs:6075+, script-extracted)

141 {20,70,395}; 142/148/149/160/173/178/209 {31}; 150/154/185/206 {44,324};
151 {24,323}; 156 {24,31,153,323}; 163/166/181/198/199/201/202/203/204 {20};
167/197 {20,44,324}; 172/176/205 {20,31}; null (no row): 152/153/155/158/
159/162/168/170/171/180. (`tools/_d36_debuffs.json`.)

## 6. FindFrame cases (NPC.cs, script-located)

Humanoid-group case list (`:68568-68633`): 142/159/160/162/167/178/181/185/
197/198/201/202/203/209. Own cases: 141 (`:72977`), 148/149/168 (`:76392`,
the penguin localAI[0] offset), 150/151/152/158 (`:74786`, the bat flap),
153/154 (NO case — the aiStyle-39 post-switch tail `:77292`), 155
(`:74311`), 156 (`:73903`), 163 (`:73227`), 166 (`:75618`), 170/171/180
(`:73418`), 172 (`:77022`, the caster pose group), 173 (`:76967`), 176
(`:74958`), 199 (`:75030`), 204 (`:70810`, the slime group), 205
(`:74935`), 206 (`:76060`). Frame counts cross-checked vs
`NPC_FRAME_COUNT` (entities.rs:1002, all 697 == Main.cs:67082).

## 7. Exhaustive-match sites (the c8/d37/d40 pattern)

Hard (compiler-forced): `to_u16`/`from_u16`/`ai_style`/`size`/`max_hp`/
`contact_damage` (data.rs), `defense`/`value`/`update_kind_ai`/`render_key`
(npc.rs), `debuff_immune_ids` (npc_buffs.rs), `npc_seed` (netglue.rs),
`portrait_data` (ui/bestiary.rs — 270 kinds, no wildcard), `draw_npcs`
(entities.rs — 270 kinds, no wildcard), `entry_for_npc`/`try_entry_for_npc`
(bestiary.rs), `ALL_KINDS` + the s1_f_3 pin (npc.rs tests).
Wildcard (script-walked per kind): `cs_id`/`hit_sound`/`death_sound`/
`no_gravity`/`ai3_walk_speed`/`humanoid_frame_count`/`extra_frames_count`/
`is_town_resident`/`is_town_npc_like`/`is_friendly` (data.rs),
`knock_back_resist`/`find_frame`/alpha-ctor (npc.rs), `texture_net_id` +
the `npc_sheets` chain (entities.rs), `STATIC_SHEET_NAMES` +
`BESTIARY_NPC_SHEET_IDS` + the parallel vec (assets.rs), `SFX_CUES`
(audio.rs), `fighter_walker_kind` (ambient_sound.rs).

## 8. AI routing (the landed machines, no new god-file)

- aiStyle 1 (141/204): `update_slime`; FindFrame the slime group.
- aiStyle 7 (142/160/178/209 town + 148/149 critters): `update_guide`
  (the d37 Pirate/GoldfishWalker precedent); the critters deal 0 contact.
- aiStyle 14 (150/151/152/156/158): `update_cave_bat` (the faithful
  `npc_ai_style14::bat_ai`; the 158 Transform(159)/156 trident are the
  module's cited unported branches — they land with the bat wiring).
- aiStyle 39 (153/154): the `npc_ai_style39` module is UNWIRED on main
  (the SolarSroller live arm keeps running); the tortoises hold the
  documented stand-in (`update_solar_sroller`'s family or the still hold)
  + the FindFrame style-39 tail ported verbatim.
- aiStyle 26 (155 Wolf): `npc_ai_unicorn::unicorn_step` (the 155 hop-over
  + 0.95 turnaround are already in the module, `:62558`/`:62621`).
- aiStyle 3 (159/162/163/166/167/168/181/185/197/198/199/201/202/203/206):
  `update_skeleton`/`update_zombie` on the `npc_ai3_speeds` ladder (the
  159 spin/199 cap-4/166 cap-3/163+162+197 cap-2/167+168+185+198+181 num83/
  201 1.1/202 0.9/203 1.2 rows are ALREADY in the ladder; 206 is in the
  shooter-exclusion list → its own arm).
- aiStyle 8 (172 RuneWizard): `update_caster` (the npc_ai_bat module; the
  172 alpha ramp/trigger arms are its cited unported branches).
- aiStyle 5 (173/176/205): the style axis `ai_005_hover_family` — new
  HoverParams CRIMERA (4/0.02, bounce 0.4, Aim), MOSS_HORNET (4/0.017,
  VelocityX), MOTH (3.25/0.018, Aim) per `NPC.cs:50304/50364/50369`.
- aiStyle 2 (170/171/180 pigrons): `update_eye` (the aiStyle-2 chase; the
  pigron phase-through-walls is the documented remainder).

## 9. Count pins (this branch, post-wave)

ALL_KINDS 270→308; to_u16/from_u16 308 arms, bijection 0 dups; s1_f_3
covered+sentinel 308; bestiary ENTRIES 241→277 (+36: 38 kinds − 2 folds
158→159, 198→199); BESTIARY_NPC_SHEET_IDS 181→217 (+36 visible); the
draw sheets D36_SHEETS 38 rows (the 2 folds keep their own draw sheet).
Sound cues: +NPC_Hit_24/26/27/29, +NPC_Killed_27/29/30/32 (8 new; the
rest already banked).
"""
