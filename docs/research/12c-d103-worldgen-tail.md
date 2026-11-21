# 12c — W4-d103 tail: the WorldGen POST-pass census (classic chain exhausted)

> Wave 4 · lane `task/W4-d103-worldgen-passes-3` (continuation after 9/9,
> tip 8be04cb0) · claimed 2026-11-21.
> Parent census: `12b-d101-passes-1.md` (the 42 unexecuted at d101's base)
> and the `12-worldgen.md` lane-split section. All pushed worldgen branches
> re-read after `git fetch --prune` (d101-d108, f24/f25, g36-g39, d1, g42).

## Part 1 — the classic unexecuted-pass list: REMAINING = ZERO (all 51 claimed)

The 51 classic (`gate == ""`) unexecuted `VANILLA_CHAIN` passes at base
a725acb0 (the d101/d102/g37/g39 scripted diffs agree) are covered by the
union of the pushed claims below. No pass is unclaimed; no lane may port a
new classic pass without colliding. Landed-vs-pending per claimant:

| # (C# order) | pass | anchor | claimant(s) | landed on a pushed branch |
| --- | --- | --- | --- | --- |
| 1-9 | Sand Patches … Generate Ice Biome, Granite | 11689-12839 | d101 | **9/9** (4194acf8 + Granite split) |
| 10-14 | Silt, Webs, Slush, Mountain Caves, Gems | 13143-15050 | d102 | **5/5** (28ec5561) |
| 15-18 | Shimmer, Clean Up Dirt, Dirt Rock Wall Runner, Wood Tree Walls | 15195-15717 | d102 + g36 A1-A4 | **4/4** (g36 aa3ee000; d102 29e8859b duplicates — merger reconciles) |
| 19 | Settle Liquids | 16132 | f25 (orphan adopt) | **landed** (0ed1f3cf) |
| 20-27 | Oasis … Cave Walls, Floating Island Houses | 16243-17859 | d103 (this lane) + g37 B1-B5 | **9/9** (d103 eb018138; g37 overlaps — merger reconciles) |
| 28-31 | Quick Cleanup, Hellforge, Spreading Grass, Surface Ore and Stone | 17875-18447 | d104 + g37 B6-B9 | **4/4** (d104 19bc132b; g37 94c365ee duplicates) |
| 32 | Place Fallen Log | 18513 | d104 + g38 C1 | **landed** (d104 2fdc8b96 / g38 8c9b7b60) |
| 33 | Traps | 18657 | d104 + g38 C2 (claim) | **landed** (d104 7c768356 + fix 341f4280) |
| 34-35 | Piles, Spawn Point | 18788, 19460 | d104 + g38 C3/C4 | pending (both claim) |
| 36 | Grass Wall | 19589 | d106 R1 (orphan adopt) | **landed** (ee0fcbef) |
| 37-40 | Guide, Sunflowers, Herbs, Dye Plants | 19685-19981 | d105 + g38 C5-C8 | pending (both claim) |
| 41-45 | Flowers, Mushrooms, Random Gems, Moss Grass, Larva | 20447-20868 | d105 | pending |
| 46 | Settle Liquids Again | 21246 | d106 R2 | **landed** (08aa8289) |
| 47-50 | Tile Cleanup, Stalac, Remove Broken Traps, Final Cleanup | 21520-22141 | d106 R3-R6 | pending |

Gated registrations (not classic): Skyblock `:11511` + the skyblock Jungle
re-registration + Dual Dungeons Dither Snake `:14755` — f24 landed all
three (9cbbf9de / 993864e1). d107 owns the seed-branch arms of executed
passes; d108 the temple seed branches. g39 landed the H1-H15 placement
helpers (bab25558 close). **The dispatch's exhaustion rule fires: this lane
takes the WorldGen POST-passes — the hardmode/growth functions outside
`AddPasses`.**

## Part 2 — the POST-pass census (WorldGen.cs hardmode family vs landed Rust)

| # | C# function | anchor | Rust today | gap |
| --- | --- | --- | --- | --- |
| P0 | `GenerateHellevator` | — | — | **does not exist in this decompilation (1.4.5.8)** — no `Hellevator`/`Elevator` symbol anywhere in `WorldGen.cs`; nothing to port under that name |
| P1 | `StartHardmode` | 31701-31719 | `tr-app world_sim/growth.rs::start_hardmode` | landed (netMode gate, chat, progression 9); `force` param unused (no caller forces) |
| P2 | `initializeHardMode` head | 31570-31661 | `spread.rs::initialize_hard_mode` is the runner half ONLY (fixed ±width/7 starts, `:31659-31660` slice) | **the anchor draws + the dungeon-side bias are missing** (Next(300,400)/Next(200,300)/Next(2), num3/num4/num5, num6) — THIS LANE |
| P3 | `initializeHardMode` remix arm | 31616-31656 | none | **missing** (the width/7 + width/14 hallow band, the per-column Next(-1,2) start) — THIS LANE |
| P4 | `GERunner` | 76169-76441 | `spread.rs::gerunner` (approximation: no shimmer pre-scan, no 20-margin box, wrong wall sets, fixed velocity, no exit band) | **verbatim rewrite** — THIS LANE |
| P5 | `initializeHardMode` moss-wall tail | 31663-31698 | none | **missing** (25·width/4200 placements, RandomWorldPoint, the Crimson/Corrupt/Hallow wall-family roll, ShapeFloodFill(1000) + OnlyWalls pocket + OuterOutline wall ring) — THIS LANE |
| P6 | `SmashAltar` | 48927-49109 | `ores.rs::smash_altar` (classic tiers/bands/blobs landed) | drunk tier-flip arms `:48939-48948`/`:48976-48985`/`:49008-49017`, the remix band `:49056-49067`, the anniversary strength `:49069-49071` — THIS LANE (seed arms); the wraith spawn `:49078-49105` parks (no NPC.SpawnOnPlayer port) |
| P7 | `OreRunner` | 48831-48925 | `ores.rs::ore_runner` (shape walk landed) | the 59-above-sky stop `:48843-48846`, the velocity drift + clamp `:48915-48923`, the runtime paint-reset + SquareTileFrame `:48887-48895`, the wallType/stayInArea/onlyReplace parameters — THIS LANE |
| P8 | `hardUpdateWorld` | 69600-69955 | `spread.rs::hard_update_world_gated` + `growth.rs::grow_crystal_shards` | **LANDED (P8 slice)** — the hardModeWorldUpdates gate `:69602`, the 60-grass chlorophyte seeding arm `:69652-69660`, the SINGLE type-9 purge pick `:69689-69700` (the C# `while(flag)` never re-arms), ChlorophyteDefense `:69456-69462` + Convert types 9/10 `:55671-55763`, the sunflower retry semantics (`continue`, not break), the walls:false Converts |
| P9 | Moss conversion micro-passes | 72371-72430 (overground), 73699-73755 (underground) | `growth/grass.rs::grow_moss_arm` / `grow_moss_arm_underground` | **LANDED (P9 slice)** — the underground arm (no NextDouble gate, the flag9 net gate parks, the shared `moss_arm_body` + the bare-write PlaceTile(184) spray approximation) |
| P10 | `MossConversion` | 71457-71493 | `growth/grass.rs::moss_conversion` | landed verbatim (DW-2.9 witnesses) |
| P11 | `CanChlorophyteGrow` | 69464-69515 | `spread.rs::can_chlorophyte_grow` | **LANDED (P8 slice)** — the remix arm (+5/×15/−4/−8) + the above-rock halving of BOTH caps and ×1.5 of BOTH boxes |
| P12 | `DoStartInHardmode` | 1899-1903 | `micro_surface.rs::start_in_hardmode` | landed (seed-gated); rides the P2-P5 rewrite automatically |
| P13 | `hardModeWorldUpdates` set | 71501 | `growth.rs::hard_mode_world_updates` | **LANDED (P8 slice)** — `hardMode || (remix && getGood && !anniversary)`, read by both dispatchers + the app growth tick |
| P14 | `TransformWorldOnBackgroundThread` | 31721-31742 | inline (documented deviation in `start_hardmode`) | stays (single-threaded port; no behavioural gap) |

Not this lane (runtime/event helpers, own lanes): `TryProtectingSpawnedItems`/
`UndoSpawnedItemProtection`, `Netplay.ResetSections`, the altar wraith spawn
(NPC spawn lane), `ClearUnbreakableWallsWithPaintUpTo` (d108 temple paint).

## This lane's slices (one per commit)

1. **(this doc)** the tail census + claim. → `fe9a565f`
2. **P2+P3+P4** — `initializeHardMode` head verbatim + `GERunner` verbatim
   rewrite (the two are one draw chain; splitting them would leave the head
   drawing anchors nobody consumes). → `04637c12`
3. **P5** — the moss-wall tail (the flood-fill/outline framework inline,
   the `wall_variety.rs` precedent). → `23f55212`
4. **P6+P7** — SmashAltar seed arms + OreRunner's missing halves.
   → `339d9f6d`
5. **P8+P11+P13** — hardUpdateWorld's gates + the chlorophyte arms +
   Convert types 9/10 + ChlorophyteDefense. → landed (this slice):
   - `hard_update_world_gated` (`:69600-69955`) verbatim: the
     `hardModeWorldUpdates` gate (`growth.rs::hard_mode_world_updates`,
     `:71501`), the actuated skip, the shard head gates moved to the call
     site (`grow_crystal_shards` = the placement body only), the 60-grass
     1/300 seeding, the 211/346 growth + the **single** type-9 purge pick
     (the C# `while (flag)` at `:69689` clears `flag` and never re-arms —
     one `Next(-6,7)` pair per source update, NOT a loop-until-air), the
     post-Plantera `Next(2)` halving, the three infection loops with the
     `continue`-retry / defence-break semantics and `walls: false`
     Converts (the old wall-converts-with-the-tile test assertion was the
     deviation — fixed).
   - `nearby_chlorophyte` (`:69517-69558`): the 211+346 ±5 count, the ≥3
     no-draw short-circuit, the border margin, the remix `Next(-1,3)` roll.
   - `ChlorophyteDefense` (`:69456-69462`): the remix-only type-10 purge;
     `convert_purge_cell` = Convert types 9/10 (`:55671-55763`) with the
     `KillTile` default-args thorn kill.
   - `can_chlorophyte_grow` (`:69464-69515`) fixed: the remix
     +5/×15/−4/−8 arm and the above-rock halving of BOTH caps + ×1.5 of
     BOTH boxes.
   - Both `UpdateWorld` dispatchers call the gated form on ACTIVE picks
     only (the C# `nactive()` else-if; the old unconditional call on
     inactive cells drew nothing anyway but sat at the wrong site). The
     overground dispatcher's arm placement keeps the port's existing
     approximation shape (documented at the call site).
   - 6 witnesses written NOT run; pins unchanged (runtime-only), a==b
     stays.
6. **P9** — the underground moss arm. → landed (this slice): the
   `:73699-73755` branch is the overground body minus the `NextDouble`
   gate — `grow_moss_arm_underground` calls the extracted shared
   `moss_arm_body` (the overground `grow_moss_arm` keeps its gate and
   delegates to the same body, behaviour frozen). The dispatcher routes
   the moss/moss-brick kinds into the arm and falls through to the wall
   arms (the C# does not return). The `flag9` net gate parks; the
   `PlaceTile(184)` spray keeps the overground slice's bare-write
   approximation (the `:59703-59721` support gate / `frameY = Next(3)*18`
   / re-frame documented at the body). 2 witnesses written NOT run; pins
   unchanged (runtime-only), a==b stays.

**The POST-pass family is exhausted by this lane's slices** (P0 does not
exist; P1/P10/P12/P14 landed or stay by design; P2-P9 + P11 + P13 landed
above).

## Discipline

- Verbatim ports (golden rule): same structure, maths, draw order; citation
  comment per block; C# read in ≤100-line windows.
- The hardmode transform draws on the caller's stream (the C# `genRand` ==
  `Main.rand` under the SwapRandom, `:4445`); no new stream is invented.
  `World.dungeon_x` becomes owned state (the `.wld` header field already
  exists — the port wrote/skipped it as "unowned"; this lane fills it).
- **Hash pins**: every slice is outside the classic pipeline or flag-gated
  (drunk/remix/anniversary/hardmode), so `SEED42_512` and `DEFAULT_WORLD`
  stay UNCHANGED by these slices (the `start_in_hardmode` micro-seed world
  changes by design — it has no pin). `a == b` determinism asserts stay
  untouched; witnesses written NOT run (code-only lane, no cargo/tests).
- ONE slice per commit+push; `anchors.txt` row per slice.

## POST FINAL (2026-11-21, final phase — wiring/runtime/keep-one/matrix)

### PF-1 — hardUpdateWorld WIRING verify: LIVE, every tick, gate = `:71501`

The C# chain, re-read in ≤100-line windows:

- `Main.cs:17800-1805`: the in-world branch (`!gameMenu || netMode == 2`)
  wraps `UpdateWorld_WorldGenAndInvasion` in `SwapRandom("UpdateWorldGen")`.
- `Main.cs:18099-18119`: the wrapper — `netMode == 1` returns (clients never
  grow), else `WorldGen.UpdateWorld()` then `Main.UpdateInvasion()`. **No
  tick-skip counter: every 60 Hz update.** (The dispatch's ":17900-region"
  pointer lands on `UpdateWorld_NPCs` `:17898` — the hardUpdateWorld cadence
  is NOT there; the call site is `:18108`.)
- `WorldGen.cs:71495-71507`: `UpdateWorld`'s head — the
  `isGeneratingOrLoadingWorld` early return, then
  `hardModeWorldUpdates = Main.hardMode || (Main.remixWorld &&
  Main.getGoodWorld && !Main.tenthAnniversaryWorld)` (`:71501`), the
  `StopBiomeSpreadPower` flag (`:71502-71507`).
- `WorldGen.cs:71536-71543`: `GetWorldUpdateRate()`; rate 0 returns; the
  overground `3e-5·rate` picks (`:71568-71601`, ×1.5 raining) and the
  underground `1.5e-5·rate` picks (`:71619-71622`; the remix `2.5e-5` dual
  dispatch `:71605-71615` parks — seed lane).
- `WorldGen.cs:72216-72218` (overground) / `:73252-73254` (underground):
  `else if (Main.tile[i, j].nactive()) { WorldGen.hardUpdateWorld(i, j); … }`
  — the ONLY two call sites (grep: `:69600` def + these two).
- `WorldGen.cs:69600-69605`: the body's own gate —
  `!hardModeWorldUpdates || inActive()` returns. So the arms run on ACTIVE
  picks only, and the gate is **hardmode OR remix·getGood·!anniversary —
  NOT hardmode-only** (a remix·getGood pre-hardmode world still spreads).

The Rust chain (verified live, not just ported):

- `app/frame/world.rs:627-644`: `update_world_frame` → the `step_allowed`
  branch → `growth_phase(dt)` (`:643`) → `App::update_growth`
  (`app/mod.rs:1552-1554`) → `world_sim/growth.rs::update_growth` (`:61`).
- `world_sim/growth.rs:74`: `app.growth_step.tick(...)` — the
  `FixedStepper::new(1.0/60.0, 8)` (`app/mod.rs:3994`) — every fixed step
  calls `tr_core::growth::update_world` on the `UpdateWorldGen` stream
  (`:83-93`).
- `tr-core/growth.rs:926-998`: `update_world` runs the overground loop
  (`:960-977`) → `update_world_overground_tile` and the underground loop
  (`:979-996`) → `update_world_underground_tile`.
- `tr-core/growth.rs:722-730` (overground, every ACTIVE pick ahead of the
  type arms — the documented `else if` placement approximation) and
  `:1733-1741` (underground, after the alch arm, on ACTIVE picks — the C#
  `nactive()` else-if order) both call `spread::hard_update_world_gated`
  with `hard_mode_world_updates(world)` (`:1004-1009` = the `:71501`
  derivation verbatim).
- `spread.rs:362-380`: the gated form's `:69602` gate + actuated skip.

**VERDICT: WIRED.** The dispatcher call-sites are live in the UpdateWorld
tick (60 Hz fixed step), every tick, gate = `hardMode || remix·getGood·
!anniversary` (not hardmode-only). Two new dispatcher-level witnesses pin
the wiring (the ebonstone/grass box corrupts through EACH dispatcher in
hardmode and freezes pre-hardmode): `growth.rs::
w4_d103_post_the_overground_dispatcher_runs_hard_update_world` +
`…_underground_…` — written NOT run.

**Residual deviation (pre-existing, documented at the call site)**:
`world_sim/growth.rs:110-124` draws ONE extra port-only surrogate pick per
growth step (`hard_update_world_gated` on a fresh random underground cell)
— the C# has no such pick (hardUpdateWorld only runs on cells the rate
loops chose). The DW-1.4 draw-count pin (`test_dw_1_4_growth_consumes_
exactly_its_stream_prefix`) freezes it as port behaviour; removing it is a
stream-shape change this lane does not make (the runtime-only pin policy).
Recorded here as the one cadence deviation: the port runs the infection
arms on (rate-loop picks + 1 surrogate) vs the C#'s rate-loop picks.

### PF-2 — SmashAltar runtime census: hammer arm LIVE, chat WIRED (this phase), sound rides the kill table, wraith parks

The ITEM-use chain (the census asked for the `ItemCheck` hammer-altar arm):

- C# `Player.cs:45646-45704` (`ItemCheck_UseMiningTools_ActuallyUseMiningTool`):
  `Main.tileHammer[26]` (`Main.cs:7204`) routes the altar to the hammer arm
  (`:45665`); `sItem.hammer > 0` adds damage (`:45670`), `CanKillTile`
  refusal zeroes (`:45671-45674`), the altar gate `type == 26 && (hammer <
  80 || !hardMode)` zeroes + `Hurt(ByOther(4), statLife/2, -direction)`
  (`:45675-45679`), `AddDamage >= 100` → `KillTile(x, y)` (`:45681-45684`).
- C# `WorldGen.cs:49444-49449` (`KillTile`'s case 26): after the object
  dies, `!generatingWorld && !TransformingWorld` → `SmashAltar(i, j)`.
- Rust `world_sim/editing.rs::hammer_tile_hit` (`:1249-1329`): the whole
  arm verbatim — the `can_kill_tile` zero (`:1260`), the altar refusal +
  self-hurt (`:1270-1288`), the `>= 100` break (`:1290-1291`) →
  `break_tile` (`:1304`, the kill cascade) → on the dead altar
  `tr_core::ores::smash_altar(&mut app.world, &mut app.ore_tiers, altars,
  rng)` on the **UpdatePlayers** stream (`:1307-1310`, the DW-1.1 pin),
  `altars_smashed += 1` (`:1311`), `notify_progression_net(6)` =
  `AchievementsHelper.NotifyProgressionEvent(6)` BEGONE_EVIL (`:49108`).
  **The P6 seed arms ride this call** — `ores.rs::smash_altar` (`:239-362`)
  is the full `:48927-49109` body (the drunk flips, the placed-ore band,
  the remix inverted band, the anniversary strength, the OreRunner
  batches). **WIRED, not just ported.**

**Gap found + wired THIS phase — the ore-blessing chat.** The C# broadcasts
`ChatHelper.BroadcastChatMessage(NetworkText.FromKey(Lang.misc[num7].Key),
ChatColors.World)` at `:48969` (cobalt, key 12 / palladium +9 = 21),
`:49002` (mythril 13 / orichalcum 22), `:49034` (adamantite 14 / titanium
23). The port's smash seam had NO chat (`let _ = ore;`). Now
`hammer_tile_hit` broadcasts `crate::lang::misc(key)` in `COLOR_WORLD`
keyed on the PLACED ore (the drunk flips move the key with the ore — the
C# reads `SavedOreTiers` AFTER the flips, same). Witness
`w4_d103_post_the_smash_broadcasts_the_ore_blessing` — written NOT run.
(The lang rows verified on disk: `Terraria.Localization.Content.en-US.
Legacy.json` `LegacyMisc` 12/13/14/21/22/23 = the six "Your world has been
blessed with …!" lines.)

**Sound census (d77 co-lane)**: the altar-smash cue is the EXISTING
`KillTile_PlaySounds` arm `WorldGen.cs:66524-66528` — `(type == 26 || type
== 695) && frameX >= 54` → `SoundEngine.PlaySound(4, …)` (NPC_Killed_1),
else the default Dig 0. The port's `kill_tile_sound`
(`edit/kill.rs:2327`) carries the arm verbatim (witness
`s1_e_21c_kill_tile_sound_table` `:4220-4222`), and `break_tile`
(`editing.rs:5627-5638`) plays it for the hit cell BEFORE the smash seam —
the C# order (`KillTile` plays at `:63558`, SmashAltar runs in the case-26
tail). The smash itself draws NO extra sound (the C# `SmashAltar` body has
no `PlaySound`). d77's tip (`d61e6362`) carries no altar rows — its scope
is projectile/kill-tail cues; **the altar cue needs no new d77 work — it
already rides the shared kill table. VERDICT: sound LIVE, co-lane closed.**

**Wraith spawn (`:49078-49105`) — stays PARKED (NPC lane).** The C# arm:
`netMode != 1`, the journey `SpawnRateSliderPerPlayerPower` disable gate
(`:49082-49093`), `Player.FindClosest`, `Main.rand.Next(2) + 1` wraiths via
`NPC.SpawnOnPlayer(player.whoAmI, 82)` (`:49100-49104`). The port has no
`NpcKind::Wraith` (the roster's `from_u16` maps 82 → PrimeSaw — the
port-local id space), no aiStyle-22 wraith machine (the Poltergeist/Drippler
machines are type-specialised), and `SpawnOnPlayer`'s 200×30 scan
(`NPC.cs:80570-80573`'s 82 band) is unported. The frame code ALREADY waits
(`npc_findframe_b.rs::FindFrameKind::Wraith`, the `NPC.cs:74823-74884`
case-48 group, `SetDefaults` `NPC.cs:9852-9878` read: 24×44, aiStyle 22,
dmg 65, def 16, 160 HP, alpha 100, kb 0.7, value 500, NPCHit54/NPCDeath52,
the remix ×1.2 scale arm). Wiring it needs the NPC lane's kind + AI +
spawn-scan ports — out of this lane's scope (12c Part 2 "Not this lane"
row stands). The journey spawn-disable power is also unported
(`creative.rs` carries the registry, not the per-player disable state).

### PF-3 — Convert-9/10 keep-one verdict: KEEP f25's `convert_chlorophyte_cell`

Both ports read the SAME C# window — `WorldGen.cs:55665-55763` (the
`Convert` switch's case 8/9/10), entered through the single-cell overload
`:55085-55097` (the `InWorld(1)` + `!active && wall <= 0` head) and the
`Convert_ActuallyConvertTile` writer `:56312-56327`.

| arm | d103 `convert_purge_cell` (spread.rs:564, P8 slice, WIRED into hard_update_world_gated + chlorophyte_defense) | f25 `convert_chlorophyte_cell` (C5 `a6b21b72`, UNWIRED) |
| --- | --- | --- |
| type coverage | 9/10 only | **8/9/10** (the mud/jungle-grass → 211 arm too) |
| head guard | `!active && wall == 0` ✓ | same ✓ |
| tile rows 9 | 2/109/477/492→60, 0→59, 23/199/661/662→60, 25/203→1, 112/234→53, 398/399→397, 400/401→396 ✓ | identical ✓ |
| tile rows 10 | the four infection grasses + the evil pairs ✓ | identical ✓ |
| thorn kill | 24/32/201/205/352/636 `kill_tile(noItem:false)` ✓ | identical ✓ |
| `Convert_ActuallyConvertTile` gate | **MISSING** — writes unconditionally (no `active && type != newType` diff gate) | **present** (`:56314`) |
| tree cross-kill | **MISSING** | `try_kill_invalid_tree_above` (`:56317`) ✓ |
| re-frame | `frame_x/frame_y = 0` bare reset (the old approximation) | `reframe_area(…, 1, false)` superset (the cleanse/d1 precedent) ✓ |
| sized walk | none (the call sites are single-cell) | `convert_sized_chlorophyte` (the `:55067-55083` Manhattan<6 walk) ✓ |

**VERDICT: KEEP f25 C5's `convert_chlorophyte_cell` + `convert_sized_
chlorophyte`** — it is the verbatim `Convert_ActuallyConvertTile` consumer
(the diff gate + the tree kill + the reframe), covers type 8 as well, and
carries 2 witnesses. **DROP d103's `convert_purge_cell` at the merge** and
rewire the two consumers onto f25's body:
`hard_update_world_gated`'s purge pick (`spread.rs:447`, type 9) and
`chlorophyte_defense` (`spread.rs:292`, type 10). The type-8 approximation
`convert_cell(.., chlorophyte: true)` (`spread.rs:119-134`, the seeding
arms `:411`/`:432`) is the SAME gap — rewire onto f25's type 8 too.
Draw-neutral: none of the three C# arms draws RNG (the reframe's
`frame_rng` is the separate framing stream), so the rewire cannot move the
`UpdateWorldGen` stream — the P8 draw-count witnesses stay valid; the
tile-state witnesses that assert the bare `frame_x = 0` reset (none do —
they assert kinds) stay valid. The d103 branch keeps its wired body until
the merge (this lane does not rewire onto an unmerged sibling — the
keep-one decision is the merger's, per the 88 playbook convention).

### PF-4 — research/12c final status + matrix-88/86 cross-refs

**12c final status: the POST-pass family is EXHAUSTED and POST-FINAL
verified.** P0 does not exist in 1.4.5.8; P1/P10/P12/P14 landed or stay by
design; P2-P9 + P11 + P13 landed (commits `04637c12`/`23f55212`/
`339d9f6d`/`014cf293`/`1c844cbb`); PF-1/PF-2 above close the wiring/runtime
gaps; PF-3 publishes the keep-one verdict. Remaining parked arms (all
out-of-lane, named owners): the wraith spawn + `SpawnOnPlayer` scan (NPC
lane), the journey spawn-disable power (creative lane), the `SendData(7)`
world-info re-broadcast on a new tier (`:49041`, net lane — the port's
`netglue` has no world-info resend seam; the tiers round-trip through the
save payload, `saveglue.rs:108-111`), the remix dual-dispatch growth loop
(`:71605-71615`, seed lane d107).

**Matrix cross-refs (read at the pushed tips):**

- `research/88-worldgen-master-matrix.md` (g36 tip `6d46f6a5`): the 109-row
  final pipeline table (§5) + the keep-one playbook. **d103's rows**: 55
  Oasis, 58 Waterfalls, 60 Wall Variety, 61 Life Crystals, 62 Statues, 64
  Surface Chests, 66 Water Chests, 71 Cave Walls — all **KEEP d103** (§4
  single-owner table), matching this lane's landed 9/9 (the classic
  passes, tip `8be04cb0` at the matrix's read). Row 73 Floating Island
  Houses: **DROP d103** (g44 owns it) — consistent with 12c Part 1's
  "g37 overlaps — merger reconciles" note; the DROP is surgical
  (`island_houses.rs` + the islands ledger edits only, §6.2). **The
  POST-pass family (P1-P14) has NO matrix rows — 88/86 scope is the
  `VANILLA_CHAIN` pipeline only; the hardmode/growth functions are
  runtime, outside the 109 registrations.** No status conflict: this
  lane's tail slices never claimed a pipeline row.
- `research/86-wg-passes-4-reconcile.md` (d104 tip, §6 addendum DONE):
  counts **55 base + 50 landed + 3 gated + 1 UNLANDED (Final Cleanup) =
  109**. d103's 9 classic rows sit in the 50 landed. The §6.3 Larva
  adjudication (d105 owns the vanilla slot, g41's NTB larva is a seed
  tail) and §6.4's 7→1 closure do not touch d103's rows. **Statuses
  align**: 12c Part 1's landed/pending map agrees with 86 §4/§6 on every
  d103 row; the only divergence is the island-houses ownership (12c
  Part 1 says "merger reconciles", 88 §2.11 adjudicates g44 — this doc
  adopts the 88 verdict, PF-4 above).
- **Gap flag for the merger**: 88 §6.1 lists Random Gems / Moss Grass /
  Final Cleanup as unlanded; 86 §6.4 (LATER, d105 tip `96f65cd6`) closed
  Random Gems + Moss Grass — **88's gap list is stale by two rows**; the
  authoritative count is 86's (1 unlanded: Final Cleanup). d103 owns none
  of the three.

### POST FINAL witnesses (written NOT run) + verification

- `growth.rs::w4_d103_post_the_overground_dispatcher_runs_hard_update_world`
  + `…_underground_…` (PF-1, the wiring pins).
- `editing.rs::w4_d103_post_the_smash_broadcasts_the_ore_blessing` (PF-2).
- Code-only lane: **no cargo/tests/rustfmt/graphify run**. Evidence = the
  C# re-reads cited above (all ≤100-line windows) + the Rust call-chain
  reads + the sibling-tip reads (`git show origin/<branch>:<path>`).
- Pins unchanged (the chat line + the witnesses are runtime/test-only);
  `a == b` untouched.

## WG-POST RETIRE (2026-11-21, resume — the SmashAltar wraith activation + the post-gen coordination)

> Resumed after the prior session's TIMEOUT (WIP frozen 13:03). Two code
> slices landed + pushed (`ce6a76a58` the g8-kind Wraith activation +
> `SpawnOnPlayer` scan, `882be0ec1` the smash-seam wiring); this section is
> the third slice — the d103-side witnesses for the post-gen coordination
> seam + the coordination census. The PF-2 "wraith parks" verdict is
> SUPERSEDED by the activation below.

### R-1 — the SmashAltar wraith spawn: ACTIVATED (the g8-kind constants-copy)

PF-2 parked the wraith spawn (`WorldGen.cs:49078-49105`) on the NPC lane.
This resume activates it as a **g8-kind constants-copy** (the g8 branch's
`NpcKind::Wraith` + the d24 aiStyle-22 machine, copied verbatim — NOT a
re-port; the g8 merge reconciles the duplicate kind):

- `NpcKind::Wraith` lands the full `SetDefaults` row (`NPC.cs:9852-9878`:
  24×44, aiStyle 22, dmg 65, def 16, 160 HP, kb 0.7, NPCHit54/NPCDeath52,
  alpha 100, value 500, noGravity+noTileCollide; the remix ×1.2 scale arm
  `:9867-9877` parks with the g8 merge). Protocol id 360 (port-local, the
  vanilla 82 stays PrimeSaw's — injective both directions).
- `npc_ai_style22.rs` is the VERBATIM copy of the g8 branch's d24 module
  (`origin/task/W4-g8-npc-kind-wire-1` blob `2a38b1051`, code-identical —
  only the doc header + allow comment differ): the aiStyle-22 ghost machine
  `NPC.cs:24666-25234`. On THIS branch the axis' 22 arm (kind-guarded)
  routes ONLY the Wraith through `ai_022_arm` + `integrate_no_tile_collide`
  (the Wraith path draws NO RNG — the module's census witness); the other
  seven members stay UNWIRED until the g8 merge.
- `spawner.rs::spawn_on_player_tile` ports `NPC.SpawnOnPlayer`'s generic
  tail (`NPC.cs:80565-80633`): the 500×50 scan with the 316/82 200×30 band,
  the shared `spawn_area_rects`/`check_not_spawning_on_screen_at` helpers,
  the solid/house-wall veto (the King-Slime `tile[num16,num16]` quirk
  verbatim), the down-column ground walk, the safe-area/HasTileSpawnSpace
  gates + the last-attempt bypasses, the type-50 CanHit headroom probe.
- `editing.rs::hammer_tile_hit`'s smash tail runs `Main.rand.Next(2)+1` on
  the UpdatePlayers stream → `spawn_on_player_tile(82)` → `NewNPC`'s
  `Bottom=(X,Y)` placement with `SpawnBoss`'s timeLeft ×20 (937→18740) and
  `Target=plr`. The journey spawn-disable gate parks (creative lane); the
  getGood `NewNPC` Next(3) transform draw parks (seed lane, no 82 arm).
- FindFrame case 82 (`NPC.cs:74823-74884` floater group) bridges
  `npc_findframe_b::FindFrameKind::Wraith`; the NPC_82 sheet + bestiary
  credit + portrait + hit/death cues all land. 3 witnesses written NOT run.

### R-2 — d103d post-gen coordination: FinalCleanup / ShimmerCleanUp (d106/g36)

The post-gen passes `Final Cleanup` (`WorldGen.cs:22141-22490`) and its
`ShimmerCleanUp` arm (`:22472`, body `:34716-34821`) are **d106/g36-owned**
— d103 ports neither (the 12c Part 1 row 47-50 "d106 R3-R6 pending" stands;
the matrix-88 §6.4 keep-one adjudicates g36's `shimmer.rs::shimmer_clean_up`
as the ONE arm). d103's coordination role is the **shared `Convert` type-0
cleanse seam** both ports call, plus the census below.

**The C# structure (re-read in ≤80-line windows):**

- `Final Cleanup` (`:22141`): `Main.tileSolid[484]=false` (`:22143`, a
  load-order poke), `FillWallHolesInArea` over the surface band (`:22144`,
  body `:9886-10053`), the `surfaceIsDesertNormalFunction` gate (`:22146`),
  the whole-world sweep (`:22150-22378`: the sand-family fall columns
  53/112/234/224/123 with the 314-track veto + hardened conversions, the
  wall-187/216 lava fill, the 137 slope clear, the boulder 2×2 normalisation
  + altar-26 veto 484→397, the 323-in-liquid kill, the dungeon-wall liquid
  strip, the 314 liquid strip, the 332 pile extension, the surface
  partial-liquid clear with the Clouds veto, the painting wall inheritance),
  the poo-668 scatter (`:22379-22422`, count 3/6/9 by `GetWorldSize`,
  anniversary ×5, the 3000 valve), the seed `Finish*` dispatch
  (`:22423-22471`), `ShimmerCleanUp` (`:22472`), the final whole-world
  `Liquid.LiquidCheck(createMergeTilesDuringGen:true)` sweep (`:22473-22483`,
  y-outer/x-inner), the NPC/angler/extra-spawn tails (`:22484-22488`).
- `ShimmerCleanUp` (`:34716`): the `(0,0)` sentinel early-return (`:34718`),
  `ShimmerRemoveWater` (`:34722`, body `:34823-34851`), the ±120 inner-disc
  cleanse (22/204→1, 51/56/229/230/659 kill, `Conversion.Moss`→1, 184 kill;
  `:34730-34752`), the outer 90-radius ellipse (y-stretch 1.4 above / 1.2
  below, **two `NextDouble` jitter per cell** `:34753`, 162 kill + wall-0 +
  `Convert(j,i,0,3)` `:34760`), the shimmer-fill walk (`:34764-34788`,
  127/255 rows), the anniversary tail (`:34789-34809`, 373/374/375→709 +
  `Shimmerator`), the tree-under-shimmer kill (`:34811-34820`).
- `Shimmerator`/`Shimminate` (`:35118-35147`/`:35104-35116`): the bounded
  3500-tile DFS over the connected non-solid liquid pool (+ active 56),
  then every counted cell → shimmer-255 + the tree/obsidian/cut(except 28)
  kill. `maxTileCount` is 3500 at this slot (the last classic writer is
  Grass Wall's `:19593`; `AddSpikeCaves` `:24087` also sets 3500 but runs
  earlier at `:16305`).

**The sibling ports (read at the pushed tips):**

- **g36** (`origin/task/W4-g36-worldgen-missing-a`, tip `a91bf92b7`, the
  gap-fill `bf7b6bf5b`): `shimmer.rs::shimmer_clean_up` (`pub`, the ONE arm
  per matrix-88 §6.4) + `shimmerator`/`shimmer_remove_water`/
  `solid_tile_ignore_water`. **UNWIRED by design** — the merger composes it
  at the TAIL of g41's `step_seed_finish` (after `micro_surface::run`), NOT
  inside `pass_final_cleanup`. Reads g36 A1's published
  `GenVars.shimmer_position` (`pipeline.rs:2093`). 7 witnesses written NOT
  run. The `Convert(j,i,0,3)` cleanse reuses **`spread::convert_cleanse`**
  (the d103-owned helper — see R-3).
- **d106** (`origin/task/W4-d106-worldgen-passes-6`, tip `c82e70a9a`, R6
  `4612ffc57`): `cleanup.rs::final_cleanup` (the whole pass) + a LOCAL
  `shimmer_clean_up`/`shimmerator` copy. The retire-final
  (`312-d106-worldgen-6-retire-final.md` §2.2) publishes the **DROP hunks**:
  the local `shimmer_clean_up`/`shimmerator` + the final LiquidCheck sweep
  block DELETE at the graft (g36's arm + the g47/g46-union LiquidSystem
  survive); `liquids::shimmer_remove_water` STAYS (the None-guard upgrade).
  d106 R6 as landed threads ONE `pass_rng("Final Cleanup")` through
  `poo_scatter` → `shimmer_clean_up` (the stream-trap fix). Witnesses
  `r6_the_shimmer_cleanup_continues_the_scatter_stream` +
  `r6_the_merge_sweep_matches_the_y_outer_replay` written NOT run.
- **g37** (`origin/task/W4-g37-worldgen-missing-b`, tip `81fd1e0d0`):
  `final_cleanup.rs::pass_final_cleanup` (the wall-hole fill + sweep +
  scatter, `:22143-22422`); the `:22423-22488` seed/shimmer/liquid tails
  DEFERRED (the `ShimmerCleanUp` arm explicitly deferred to g36 — "reads
  `GenVars.shimmerPosition`, which g36 A1 owns; no slot on this branch").

**The coordination verdict (d103-side):**

1. **The `Convert` type-0 cleanse is the d103-owned seam.** Both g36's and
   d106's `shimmer_clean_up` route the ellipse's `Convert(j,i,0,3)`
   (`:34760`) through `spread::convert_cleanse` (the S1-D-4 Clentaminator
   helper, `spread.rs:644`). d103 does NOT port FinalCleanup/ShimmerCleanUp
   — it owns the shared cleanse the two ports call. R-3 pins it.
2. **The stream contract.** The C# `genRand` is ONE per-pass UnifiedRandom;
   the poo scatter (`:22379-22422`) and the ShimmerCleanUp ellipse jitter
   (`:34753`) draw on the SAME instance. `Streams::pass_rng(name)` returns a
   FRESH stream per call (`pipeline.rs:230`), so the merger MUST thread ONE
   `Rng` through scatter→cleanup (matrix-88 §6.4 form (a): stash
   `gv.final_cleanup_rng`). d106 R6 already threads it; g36's arm takes
   `rng: &mut Rng` (the merger passes the threaded instance). On a classic
   world with no shimmer (`shimmer_position == None`) the arm draws NOTHING
   (the sentinel returns before any draw) — the trap only bites shimmer
   worlds. **d103's `convert_cleanse` draws NO caller RNG** (the C# `Convert`
   head `:55067-55083` is RNG-free); its reframe rolls ride `world.frame_rng`
   (the recorded genRand deviation, `spread.rs:662-668`), so the cleanse
   never perturbs the threaded scatter stream. This is the load-bearing
   d103-side invariant for the compose.
3. **The `tilesIgnoreWater` window** (`:34767`/`:34810`) lifts the boulder
   family (138/484/664/711-716) + grate 546 solidity for the Shimmerator
   flood. g36 carries `solid_tile_ignore_water` (the `liquid.rs::BOULDER_SOLIDS`
   cited copy); d106's local copy documents it as a no-op stand-in (the
   boulder/grate family does not appear in the shimmer disc's fill path).
   The window is a documented no-op for the cleanse arm itself.
4. **The `kill_tile` no_item receipt.** At gen time the C# `KillTile`
   ALWAYS suppresses items (`:63546-63549`, `isGeneratingOrLoadingWorld`
   true for the whole generation). d106/g36's gen-time `kill_tile(...,true)`
   is behaviourally identical, not a deviation (research-312 §3a). d103's
   `convert_cleanse` thorn kill passes `false` (the runtime Clentaminator
   path, `spread.rs:767`) — at the gen-time compose the C# override would
   make it `true`, but the thorn kill drops no item either way (the thorn
   kinds 32/352 have no `KillTile_GetItemDrops` row), so the seam is
   draw-neutral and item-neutral. **No d103 change needed.**

### R-3 — the d103-side witnesses (written NOT run)

`spread.rs` (the `tests` module, the d103-owned `convert_cleanse` seam):

- `w4_d103_retire_the_cleanse_cell_purifies_and_kills_thorns` — the type-0
  cell ladder (`:55527-55664`): the evil grass/stone/sand/ice/hardened-sand/
  sandstone → their pure targets, the thorn 32/352 kill, the pure-grass/
  stone no-ops, the wall 69→64 / 3→349 arms. Drives `convert_cleanse_cell`
  directly (the single-cell arm).
- `w4_d103_retire_the_cleanse_size3_walk_gates_the_corners` — the size-3
  square walk (`:55067-55083`): the Manhattan `< 6` gate refuses the four
  corners (|dx|+|dy|==6), every other in-box cell purifies; a==b twin
  replay (the walk is deterministic, the reframe rolls ride
  `world.frame_rng` seeded identically by `World::new`).

These pin the d103-owned helper the g36/d106 ShimmerCleanUp ports call —
the coordination seam, not the post-gen passes themselves (d106/g36 own
those). Code-only lane: **no cargo/tests/rustfmt/graphify run**. Evidence =
the C# re-reads cited above (all ≤80-line windows) + the sibling-tip reads
(`git show origin/<branch>:<path>`) + the Rust call-chain reads. Pins
unchanged (the witnesses are test-only); `a == b` untouched.