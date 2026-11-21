# 244 — KINDS-631-697 RETIRE (W4-d43 claims-table + dedupe drop-hunks)

Lane: `task/W4-d43-npc-kinds-631-697`, worktree `TR-W4-d43`.
Retire round `d43s`, base tip `0253bde90` (batch 13/13, the full band landed).
**Code-only mode — no cargo/tests/rustfmt/graphify invoked; the witnesses below
are WRITTEN NOT RUN.** Number **244** per the registry-150 rule (150 = the
collision registry, 151–236 reserved for its losers, 237+ free; a fresh
all-`task/W4-*`-tip `ls-tree docs/research/` scan at retire time shows 237–243
taken — e23/g48@237, c8/d42/g61@238, d42/f21/g56@239, d53/g58@240,
g58@241/242/243 — so 244 is the next free key).

Companion contracts read: research/108 (d37 roster master + E8 + SEAL),
research/148 (c4e fish-family declaration — the 688/692 conditional),
research/150 (the number registry), the swarm ledger v429/v434/v435 dedupe
flags. This doc is the d43 side of the d37d-E8 adjudication: it publishes the
keep-one verdicts + the EXACT drop-hunks for the losing d43 rows, verifies the
d33-style activation contract, and declares the band complete.

---

## 1. The claims-table (d37d-E8 dedupe) — keep-one verdicts

The d43 full-band landing (13 batches, 63 kinds) overlaps six sibling lanes.
The E3 ownership rule ("kinds-lane owns over wiring-lane") is the default, but
it is OVERRIDDEN where the sibling carries the faithful MACHINE + the wiring
(the g5/g10/d125/c3/c7/c9 cases) — a bare kind row must not shadow a wired
machine. Verdicts:

| cs id | kind | d43 row | sibling claim | KEEPER | rationale |
| --- | --- | --- | --- | --- | --- |
| 693 | LibrarianSkeleton | batch 12 (`8fc608bc`) | **g5** `96f07495` phase 4/5 | **g5** | g5 ports the full kind + the LIVE caster wiring (`npc_ai_bat.rs` +778: `AI_FindNearbyBook` `:62308-62364`, `AI_AttemptToFindTeleportSpotNearBooks` `:19009-19085`, the 1092 book shot + the 129 RuneBlast). d43's 693 casts SILENTLY (book-search unplumbed — its own documented skip). Keep the wired caster. |
| 636 | HallowBoss (Empress of Light) | batch 1 (`510abc9d`) | **d125** `EmpressOfLight`@229→636 (R4) | **d125** | d125 ports the AI_120 adapter (`ai_120_empress`, the 9999 day-enrage one-shot, the IsDamageDodgeable stamp, the owned bolt drains), `empress_draw.rs` (the glow/halo/arm-frame geometry), the 919/923/872 special hitboxes, BossNoCheese, the .trwld downedEmpress byte. d43's 636 is a `{}` no-op stand-in. Keep the wired boss. |
| 661 | EmpressButterfly (Prismatic Lacewing) | batch 6 (`ee665ab6`) | **d125** `PrismaticLacewing`@237→742 (R5/E8) | **d125** | same kind, different variant name. d125 ports the whole AI_065 661 branch (`npc_ai_lacewing.rs`, the release/vanish/17-draw census/re-aim) + the death→SpawnBoss(636) chain + the kind-guarded `65 if` axis arm. d43's 661 is a `{}` stand-in. |
| 637/638/656/670/678/679/680/681/682/683/684 | TownCat/TownDog/TownBunny/TownSlimeBlue/Green/Old/Purple/Rainbow/Red/Yellow/Copper (11 pets) | batches 2/5/9/10/11 | **g10** `5dee2865` (cs==proto) | **g10** | g10 wires the pets to the landed pet-aware town machine: the kind-guarded `7 if self.kind.is_town_pet()` axis arm (`ai_007_town_pet_family`), `npc_findframe_pet.rs` (489 ln), the `is_town_pet`/`is_town_slime` gates, the ExtraFramesCount/AttackFrameCount rows, `spawn_guide`. d43's pets ride `update_guide` (the town-NPC stand-in) + `{}` FindFrame. Keep the wired pets. **The brief named 637/638; the overlap is actually ELEVEN** (g10 grew to TownSlimeCopper 684 in batch 2). |
| 694 | WaterBoltMimic | batch 13 (`0253bde9`) | **c3** `9299046f` (style-10 lane) | **c3** | c3 ports the kind + the `10 =>` axis claim + the FULL 694 anchor (`npc_anchor_nudge`'s `num45` X term + `num46` Y term, `Main.cs:23096-23113`, the ai[3]==3/4 offsets) + the book-strip FindFrame. d43's 694 rides `update_water_bolt_mimic` (a kind-keyed style-10 call) with NO anchor terms. Keep c3's wired skull-family member. |
| 654 | Lavafly | batch 5 (`600c61f2`) | **c7** `c2cc6ec6` (style-64 lane) | **c7** | c7 ports the kind + the `64 =>` axis claim (`update_rng_style64_solo`) + the firefly FindFrame. d43's 654 is a `{}` stand-in. |
| 655 | MagmaSnail | batch 5 (`600c61f2`) | **c9** `2042d77f` (style-67 lane) | **c9** | c9 ports the kind + the `67 =>` axis claim (`ai_067_snail`) + the snail FindFrame + the shimmer_immune buff-353 set. d43's 655 is a `{}` stand-in. |
| 677 | Shimmerfly | batch 9 (`418336473`) | **c7** `c2cc6ec6` (E3: keep c7@677, drop c9@677) | **c7** | c7 owns 677 per the E3 adjudication (c9's dup drops). c7 wires the `64 =>` axis. d43's 677 is a `{}` stand-in. |
| 688 | Pufferfish | batch 11 (`fd6a544f`) | c4 `0df6964b` (conditional) | **d43** | c4e research/148's conditional FIRED: d43 rowed 688 before the merge → E3 kinds-lane ownership. **c4's 688 variant retires onto d43's row.** d43 folds c4's keeper implementations (§3). |
| 692 | Orca | batch 12 (`8fc608bc`) | c4 `0df6964b` (conditional) | **d43** | same conditional. **c4's 692 variant retires onto d43's row.** d43 folds c4's keeper implementations (§3). |

**Net d43 drop count: 18 kinds** (693, 636, 661, the 11 g10 pets, 694, 654,
655, 677). **d43 keeps 45** of its 63 (63 − 18), INCLUDING 688/692 (the c4
conditional flipped them to d43). Post-drop d43 contribution: 45 kinds;
`ALL_KINDS` 333 → 315 on the d43 branch after the drops execute (the merge
wave re-derives the grand-union pin per research/108).

**NOT a dup — d43 keeps:** 665 ChaosBallTim / 666 VileSpitEaterOfWorlds. The
base FOLDS 665 into ChaosBall 30 ("identical stats and name"), but the C#
bestiary gives 665/666 their OWN entries (`BestiaryDatabaseNPCsPopulator.cs:951`
/`:954` attach info to `FindEntryByNPCID(665/666)`). d43's separate kinds +
own bestiary credits are MORE faithful than the base fold — keep them (the
base's fold comment is the stale claim, not d43's rows).

---

## 2. The EXACT drop-hunks (the losing d43 rows)

**These execute AT MERGE, not on the d43 branch.** The d43 rows stay on-branch
so the exhaustive `NpcKind` matches compile (the BloodFeeder treatment, c4e
research/148). The merger drops the d43 variant + every arm below and keeps the
sibling's. Line numbers are d43-tip (`0253bde90` + the retire fixes) anchors;
the SYMBOL is the durable key (lines drift).

### 2a. 693 LibrarianSkeleton → keep g5

Drop the d43 `NpcKind::LibrarianSkeleton` variant + these arms (keep g5's):
- `npc.rs`: enum variant `LibrarianSkeleton` (`:1120`); kb `:3799`; defense
  `:4242`; value `:4677`; kind-AI arm `:6754` (`update_caster`); FindFrame arm
  `:10899`; render_key `:20688`; ALL_KINDS `:26098`; batch-12 stat pin `:26508`.
- `npc/data.rs`: to_u16 `:596`; from_u16 `:989`; hit_sound `:1965`; death_sound
  `:2329`; ai_style `:2770`; size `:3367`; cs_id `:3506`; max_hp `:4136`;
  contact_damage `:4563`.
- `npc_buffs.rs:494` (`&[20,31]`); `npc_ai_bat.rs:80` (the `dungeon_wall_gated`
  `| NpcKind::LibrarianSkeleton` — g5's own gate stays); `app/netglue.rs:3588`
  (the spawn_cave_walker arm — **g5 uses spawn_hover, keep g5's**);
  `bestiary.rs:2231` (the ENTRIES row); `ui/bestiary.rs:1635` (the portrait);
  `render/entities.rs:3429` (the draw arm).
- **KEEP (base/shared, not d43-added):** `creative_sacrifices.rs:3121`,
  `itemdata.rs:7075`, `lang_tables.rs:770`, `projectile_id.rs:1104` (the 1092
  book name) — these are base rows g5 also reads.
- assets: `NPC_693` stays packed (g5 uses it); the bestiary grid slot 693 is
  g5's (d43's append dedupes onto g5's).

### 2b. 636 HallowBoss + 661 EmpressButterfly → keep d125

Drop d43 `NpcKind::HallowBoss` + `NpcKind::EmpressButterfly`; keep d125
`EmpressOfLight` (636) + `PrismaticLacewing` (661, proto 742 after R5):
- `npc.rs`: HallowBoss enum `:1340`, EmpressButterfly enum (batch 6); boss flag
  `:3240`; dontTakeDamage `:3290`; kb `:3854`/`:3829`; defense `:4297`/`:4272`;
  value `:4732`/`:4707`; kind-AI `{}` arms `:6845`/`:6812`; FindFrame `:9662`
  (636) + the 661 arm; render_key `:20636`/`:20661`; ALL_KINDS
  `:26077`/`:26102`; stat pins `:26198`/`:26353`.
- `npc/data.rs`: to_u16/from_u16/cs_id/ai_style/size/max_hp/damage/sound rows
  for both (the 636/661 cs_id rows `:3525`/`:3536` region).
- `npc_buffs.rs:549` (636 `&[31]`), `:524` (661 `&[31]`); `app/netglue.rs` (the
  636/661 spawn arms — d125's `spawn_hover` stays); `bestiary.rs:2671` (HallowBoss
  ENTRIES) + `:2471` (EmpressButterfly ENTRIES); `ui/bestiary.rs` portraits;
  `render/entities.rs` draw arms.
- **d125's R4/R5 renumber rides its own lane** (229→636, 237→742); the merger
  takes d125's protocol ids, NOT d43's 636/661 to_u16. The bestiary net_id 636/661
  is shared (both lanes key the same vanilla credit) — one ENTRIES row survives.

### 2c. The 11 g10 town pets → keep g10

Drop d43 `TownCat/TownDog/TownBunny/TownSlimeBlue/TownSlimeGreen/TownSlimeOld/
TownSlimePurple/TownSlimeRainbow/TownSlimeRed/TownSlimeYellow/TownSlimeCopper`;
keep g10's (cs==proto, the kind-guarded `7 if` axis + `npc_findframe_pet`):
- `npc.rs`: the 11 enum variants; kb/defense/value rows (`:3845`/`:3846` etc.);
  the `update_guide` kind-AI arms (`:6834`/`:6835`/`:6755`/`:6762-6766`/
  `:6777`/`:6781`/`:6823`); the `{}` FindFrame arms (`:9672`/`:9677` etc.);
  render_key rows; ALL_KINDS entries; the batch stat pins.
- `npc/data.rs`: to_u16/from_u16/cs_id/ai_style(7)/size/max_hp/damage/sound rows.
- `npc_buffs.rs:495-511`/`:529`/`:540-541` (the `&[31]` rows); `app/netglue.rs`
  (the spawn_cave_walker arms — **g10 uses spawn_guide, keep g10's**);
  `bestiary.rs` ENTRIES; `ui/bestiary.rs` portraits; `render/entities.rs` draws.
- **g10's `is_town_pet`/`is_town_slime` gates + `npc_findframe_pet.rs` +
  `world_sim/clock.rs` pet-spawn + `app/saveglue.rs` pet-license rows are the
  keeper wiring** — d43 has none of them.

### 2d. 694 WaterBoltMimic → keep c3

Drop d43 `NpcKind::WaterBoltMimic`; keep c3's (the `10 =>` axis + the full anchor):
- `npc.rs`: enum variant; kb `:3792`; defense `:4235`; value `:4670`; kind-AI
  `:6739` (`update_water_bolt_mimic`); the `update_water_bolt_mimic` fn
  `:12723-12740` (RETIRE wholesale — c3's axis arm is the superset); FindFrame
  `:10919`; render_key `:20692`; ALL_KINDS `:26133`; stat pin `:26566`.
- `npc/data.rs`: to_u16/from_u16/cs_id `:3499`/ai_style(10) `:2763`/size/max_hp/
  damage/sound rows.
- `npc_buffs.rs:487` (`&[20,31]`); `app/netglue.rs` spawn arm; `bestiary.rs:2175`
  ENTRIES; `ui/bestiary.rs:1598` portrait; `render/entities.rs:3432` draw.
- **c3's `npc_anchor_nudge` 694 arm (num45 X + num46 Y) is the keeper** — d43
  has NO 694 anchor (the generic draw would mis-place it). The merger keeps c3's.

### 2e. 654 Lavafly / 655 MagmaSnail / 677 Shimmerfly → keep c7/c9/c7

Drop d43 `Lavafly`/`MagmaSnail`/`Shimmerfly`; keep c7's 654/677 + c9's 655:
- `npc.rs`: enum variants; kb `:3832`/`:3833`/`:3813`; defense
  `:4275`/`:4276`/`:4256`; value `:4710`/`:4711`/`:4691`; the `{}` kind-AI arms
  `:6819`/`:6822`/`:6776`; FindFrame arms; render_key `:20654`/`:20655`/`:20675`;
  ALL_KINDS `:26095`/`:26096`/`:26116`; stat pins `:26320`/`:26321`/`:26445`;
  the CS_STYLES pin `:27321` (Shimmerfly).
- `npc/data.rs`: to_u16/from_u16/cs_id/ai_style(64/67/64)/size/max_hp/damage/
  sound rows.
- `npc_buffs.rs:527`/`:528`/`:508` (`&[31]`); `app/netglue.rs` spawn arms;
  `bestiary.rs` ENTRIES; `ui/bestiary.rs` portraits; `render/entities.rs` draws.
- **c7's `64 =>` axis + c9's `67 =>` axis + their FindFrame modules are the
  keeper wiring.** c9's Shimmerfly 677 dup drops per E3 (keep c7@677).

### 2f. Count-pin re-derivation (the merge wave)

After the 18 drops: d43 `ALL_KINDS` 333 → 315, `s1_f_3` gate 333 → 315,
`to_u16`/`from_u16` 333 → 315 arms. The bestiary ENTRIES lose the 18 dropped
kinds' rows (the 689 OwlMimic row STAYS — it is d43's, Hide-folded, §4). The
grand-union pins (research/108's ALL_KINDS-550 / ENTRIES-486 / sheets-455) are
re-derived by the merge wave AFTER all lanes' drops + renumbers land — this doc
does not re-derive them (the d43 side only).

---

## 3. The c4e keeper folds APPLIED to d43 (688/692)

c4e research/148's conditional fired (d43 rowed 688/692) → d43 owns them and
folds c4's keeper implementations. Applied this retire round (code, witnesses
WRITTEN NOT RUN):

1. **The puff one-tick flag** (`npc.rs::update_swimmer`): `just_hit` now reads
   `self.just_hit` (the one-tick strike flag, set `:81274` / cleared `:90516`),
   NOT `self.hurt_cd > 0.0` (the cooldown stand-in that re-puffs every tick of
   the hurt window). The `ai_055_creeper_family` precedent. Witness
   `w4_d43_the_pufferfish_puff_reads_the_one_tick_flag_not_the_cooldown`.
2. **The Pufferfish 688 anchor** (`render/entities.rs::npc_anchor_nudge`): the
   base arm was MIS-KEYED `DD2GoblinT1 => num46 += 10.0` (a copy-paste of the
   -2 row above; `Main.cs:23114-23116` is the Pufferfish's). Re-keyed to
   `Pufferfish => num46 += 10.0`. This is a BASE bug the new kind exposes.
3. **The Orca 692 wet-anchor** (`npc_anchor_nudge`): added
   `Orca => num46 = if n.wet { 6.0 } else { 10.0 }` (`Main.cs:23071-23073`).
   Witness `w4_d43_the_pufferfish_and_orca_anchor_nudges`.

**NOT folded (c4 defects, d43's rows are C#-correct):**
- c4's `npc_buffs.rs:252` `Pufferfish | Orca => &[]` is WRONG — the C#
  `NPCID.cs:10046-10057` gives BOTH `SpecificallyImmuneTo = {31}` (Confused).
  d43's `&[31]` rows (`npc_buffs.rs:493`/`:499`) are correct; keep them, drop
  c4's `&[]`.
- The Orca NPC_692 sheet (136×992): c4 atlas-DEFERRED it (overflow); d43 packed
  it eagerly (`assets.rs:1284`). The atlas crisis (research/113, d35c/d35e)
  owns the lazy-page decision — d43's eager pack is flagged for the repack wave
  (it may move to the lazy NPC page per research/144). NOT a behaviour change.

**c4's 688/692 retire-hunks (the merger drops c4's variants):** c4's
`NpcKind::Pufferfish`/`Orca` enum + data rows + the `style16_find_frame` 688/692
cases (`npc_ai_style16.rs:783`/`:818`) + the bestiary entries + the
`npc_anchor_nudge` Orca arm (d43 now carries it) retire onto d43's rows. c4's
`vanilla_type` 688/692 arms (`npc_ai_style16.rs:128-129`) are IDENTICAL to d43's
(`:108-109`) — one survives. c4's puff `just_hit` fix `17a241f6a` is the same
fold d43 applied (§3.1) — dedupe.

---

## 4. The 689 OwlMimic Hide-fold (task 3 — APPLIED)

The brief: "689-folds-credit-611-Hide". Verified against the C#:
- `NPCID.cs:755-761`: 689's `NPCBestiaryDrawModifiers` carries `Hide = true`.
- `ContentSamples.cs:1054`: `npcBestiaryCreditIdsByNpcNetIds[689] =
  npcBestiaryCreditIdsByNpcNetIds[611]` — the kill credit folds onto the Owl 611.

d43's batch-12 OwlMimic entry was UNFOLDED (a standalone visible "Owl" card) —
a gap. Applied this retire round (the d33 hidden-dragonfly precedent):
- `bestiary.rs::entry_for_npc`: added `NpcKind::OwlMimic => NpcKind::Owl` (the
  credit fold). `try_entry_for_npc` stays UNFOLDED (the s1_f_3 cs_id==net
  invariant — the 689 entry keeps net_id 689 == cs_id 689).
- The 689 ENTRIES row STAYS (own credit_id "OwlMimic" + cs_id 689 for the
  catch/release chain's `kind_from_net_id`).
- `assets.rs`: 689 DROPS from `BESTIARY_NPC_SHEET_IDS` (244 → 243) + the parallel
  `bestiary_npc_sheets` vec (the Hide=true kind takes no grid slot). The
  `NPC_689` world-draw sheet STAYS packed (`STATIC_SHEET_NAMES`).
- Witness `w4_d43_the_owl_mimic_folds_its_credit_and_takes_no_grid_slot`.

**Merge dependency:** the fold TARGET `NpcKind::Owl` (611) rides d42's band
(561-630). On the d43 branch alone `NpcKind::Owl` does not exist — the fold arm
compiles only after d42 lands (the merge wave). This is the documented
cross-lane dependency (the d33 dragonfly fold resolved the same way). The
671-675 macaws/toucan/cockatiels are NOT Hide (own visible cards, d43 keeps
them); their FindFrame (`NPC.cs:73654-73677`) + catch items (5212/5300/5311/
5312/5313, `bug_net.rs:188-192`) + frame counts (14 each) are verified faithful.

---

## 5. The d33-style activation contract (task 2) — VERIFIED

The brief: "657-QueenSlime/686-balloon/690-StatueMimic/695/696-Pals/662-
PirateGhost wire-to-d33-style-modules-121/122/125/126/127 (d33-UNWIRED — your
kinds rowed, activation-contract: d33-style-wiring merges → your kinds
auto-activate? verify axis-routing)."

**Axis-routing verified:** `npc.rs::update_rng`'s style axis (`match
self.kind.ai_style()`, `:5100`) dispatches FIRST; the `_ => update_kind_ai`
fallback (`:5382`) runs the kind match only for UNCLAIMED styles. d43's six
kinds carry the correct ai_style rows (`npc/data.rs`): QueenSlimeBoss 657→121,
PirateGhost 662→122, BoundTownSlimePurple 686→125, StatueMimic 690→126,
PalworldCattiva/Foxsparks 695/696→127. Their kind-AI arms are `{}` no-op
stand-ins (`:6806`/`:6793`/`:6759`/`:6751`/`:6742`/`:6745`).

**d33's modules are UNWIRED** (`npc_ai_style121/122/125/126/127.rs`, pure fns
`styleNNN_step(npc, env) -> Outcome` + FindFrame helpers, NO axis arm, NO sim
caller — the style73 precedent). **So the kinds DO NOT auto-activate when d33
merges.** The module alone is inert; the axis arm + the env adapter are the
missing wiring.

**The activation contract (the merge wave's work order):**
1. d33's `styleNNN_step` modules land (pure fns, no behaviour change alone).
2. The merger adds the axis arms `121 => …`, `122 => …`, `125 => …`,
   `126 => …`, `127 => …` to `update_rng`, each filling the module's
   `StyleNNNEnv` seam (world/rng/players/net_mode + the per-style extras:
   121's QueenShot/QueenMinionSpawn drains, 122's GhostNeighbor pool, 125's
   balloon collision box, 126's gen_rand statue-style latch, 127's npc_type
   pal picker) — the `ai_120_empress` adapter precedent (d125).
3. The `{}` kind-AI stand-ins retire to `unreachable!` guards (the axis claims
   the style first — the S1-D-4c one-live-arm rule; a duplicate arm is a
   compile error under `#[deny(unreachable_patterns)]`).
4. The FindFrame helpers (`queen_find_frame`/`ghost_find_frame`/`pal_find_frame`)
   replace d43's inline FindFrame arms for those kinds (d43's 657/662/695/696
   arms are the C#-faithful row decodes; d33's helpers are the same machines —
   dedupe onto d33's, the 686/690 single-frame kinds have no FindFrame case).

**Until step 2 lands, the six kinds hold still** (the `{}` stand-in) — this is
the documented UNWIRED state, NOT a bug. d43's contribution is the kind rows +
the ai_style routing + the SetDefaults/FindFrame/bestiary data; the MACHINE
wiring is d33's lane + the merge wave. **No auto-activation.**

---

## 6. 664 TorchGod (task 4) — d43 KEEPS the kind row, f33 owns the event

The brief: "664-TorchGod-kind-row (f33-event-owns — your row vs f33 keep-one)."

**Verified: NOT a collision.** f33's torch-god event spawns PROJECTILE 949
(`ProjectileID.TorchGod = 949`, `torch_god.rs:59` / `projectile_ai_torch_god.rs`),
NOT NPC 664. The C# NPC 664 (`NPCID.TorchGod = 664`) has a SetDefaults row
(`NPC.cs:17398-17402`: `lifeMax = 20; boss = true;` — width/height 0, aiStyle 0)
+ a FindFrame case (`NPC.cs:74837`, the shared 634/662/664 flap) but NO
`NewNPC(664)` spawn site in the decompile (the event is projectile-driven).

**Verdict: d43 KEEPS the NPC 664 kind row** (the data-only coverage of the
vanilla NPC id — the SetDefaults/FindFrame/bestiary rows are faithful);
**f33 owns the EVENT** (the projectile-949 machine + the favor-use + the
persist). They are complementary, not duplicates. d43's `TorchGod => {}`
kind-AI arm (`:6796`) is the correct stationary stand (aiStyle 0, the
BoundGoblin precedent — the NPC never moves; the event is the projectile).
No drop-hunk. The 0×0 size is the C#'s own (the row sets no width/height).

---

## 7. The band-631-697 declaration + bijection-check (task 5)

**BAND COMPLETE.** d43 landed 63 kinds covering NPCID 631-696 minus the three
base-owned ids (633 Zoologist, 663 Princess, 668 Deerclops — all on
`origin/main`). 697 is `NPCID.Count` (the array length, `NPCID.cs:12581`), NOT
a kind — the band's last real id is 696. Scripted verification at retire time:

- **cs_id coverage:** 63 d43 rows in 631-696 (the 65 rows the scan finds include
  base 633/663); `Compare-Object` against `631..696 ∖ {633,663,668}` = EMPTY
  (no gap, no extra).
- **Bijection:** `ALL_KINDS` decl 333 == 333 arms, no dup kinds; `to_u16` 333
  arms / `from_u16` 333 arms, id sets IDENTICAL (`Compare-Object` = EMPTY);
  max protocol id **696 ≤ i16::MAX** (the `23` body's i16 kind field is safe —
  the `the_protocol_id_space_round_trips_every_kind` witness pins it).
- **Frame counts:** `NPC_FRAME_COUNT[631..696]` == the C# `Main.npcFrameCount`
  literal (`Main.cs:67081-67153`) for EVERY id in the band (scripted
  element-wise compare = ALL MATCH). The 689 OwlMimic = 13, 692 Orca = 16,
  694 WaterBoltMimic = 30, 695/696 Pals = 3/1, 657 QueenSlime = 16, 662/664 = 4.
- **s1_f_3 gate:** 333 (the wire-roster size pin, `npc.rs:30118`).

**Post-drop projection (§2):** 18 kinds retire onto siblings → d43 contributes
45; the merge wave re-derives the grand-union pins. The bijection holds at
every step (each drop removes the variant + its to_u16/from_u16 arm pair).

---

## 8. Witnesses (WRITTEN NOT RUN — code-only mode)

Added this retire round (no compiler invoked):
- `npc.rs::w4_d43_the_pufferfish_puff_reads_the_one_tick_flag_not_the_cooldown`
  — the §3.1 puff-flag fold (strike tick puffs, cooldown-only tick does not).
- `render/entities.rs::w4_d43_the_pufferfish_and_orca_anchor_nudges` — the §3.2/
  §3.3 anchor folds (Pufferfish +10, Orca 10 dry / 6 wet, the mis-keyed
  DD2GoblinT1 arm no longer fires).
- `bestiary.rs::w4_d43_the_owl_mimic_folds_its_credit_and_takes_no_grid_slot`
  — the §4 Hide-fold source side (own entry + cs_id 689, no grid slot, world
  sheet packed). The fold TARGET (Owl 611) resolves at the d42 merge.

The 13 batch stat-pin witnesses (`w4_d43_batchN_the_defaults_match_the_csharp_rows`)
+ the bijection witness stay as landed. The drop-hunks (§2) are merge-wave
work orders, not d43-branch edits (the rows stay so the exhaustive matches
compile).

---

## 9. Verification record

- C# re-reads: `NPC.cs:17398-17402` (664 SetDefaults), `:74835-74884` (the
  634/662/664 FindFrame), `:73654-73677` (the 671-675 bird FindFrame),
  `:23071-23073`/`:23114-23116` (the 692/688 anchors, Main.cs); `NPCID.cs:755-761`
  (689 Hide), `:10046-10057` (688/692 {31} immunity), `:12581` (Count 697);
  `ContentSamples.cs:1054` (the 689→611 credit fold);
  `BestiaryDatabaseNPCsPopulator.cs:951/954` (665/666 own entries).
- Sibling tips read: g5 `96f07495`, g10 `5dee2865`, d125 `7a0482302`, c3
  `9299046f`, c4 `0df6964b` (+ research/148), c7 `c2cc6ec6`, c9 `2042d77f`,
  d33 `b08acf463`, f33 `83387628`, d37 research/108 `a341daa12`.
- Scripted: the band cs_id coverage, the ALL_KINDS/to_u16/from_u16 bijection,
  the NPC_FRAME_COUNT band compare, the registry-150 free-number scan.
- **No cargo/tests/rustfmt/graphify invoked** (code-only mode). No junction
  recursion (direct paths only). Edits via the edit tool only.
