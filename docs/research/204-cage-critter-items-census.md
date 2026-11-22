# 204 — Cage critter items: place / release / light census (W4-g34-drawdata-wire-2)

> **RENUMBERED 68 → 204 (batch-H merge, registry research/150 §3 round-1:
> f37's `68-mood-reconcile.md` keeps 68, this doc → 204; slug kept, all
> refs repointed).**

> **STATUS: SEALED (2026-11-21).** The place/release/light census (§0-§5),
> the 92-kind break-drop cascade (§6), the seal verification + the Chillet
> port + the centre-gap FIX-ONCE (§7) and the cage-family bible (§8) are
> complete on this branch. Witnesses WRITTEN NOT RUN (code-only).

Worker: code-only subagent on `task/W4-g34-drawdata-wire-2`. Read-only census +
wiring fixes; no builds/tests run. All C# cites are `decompiled/Terraria/…`;
Rust cites are `crates/…` at the branch tip.

**Update (break-drop cascade pass):** §2's task seed is now IMPLEMENTED — the
full 92-kind cage union's break-drop cascade (`Check6x3`/`Check3x2`/`Check2x2`/
`Check1x2Top`) plus the `Check3x2` furniture/echo/log family. See §2's table
and §6 below.

**Update (SEAL round, 2026-11-21):** §7 records the seal verification — the
92-kind union landed at `cf85a1cf`, the g34f cage-retire DROP list verified
executed, the d105 Chillet-egg task seed PORTED (§7.3), and the g78
ObjectDrop 32×32 centre-drop gap FIXED ONCE at the spawn seam (§7.4). The
cage-family bible is §8.

## 0. The brief's pointers corrected against the source

The task brief named "tile 345", "cage items 2440+", "cage release right-click"
and "Main AddLight tile-345 style arm". **None of these match the 1.4.5.8
decompilation.** The golden rule (port the C# first, never invent) forces the
following reconciliation:

| Brief name | Real location |
| --- | --- |
| "tile 345" | `TileID.Honeyfall = 345` (`TileID.cs:1319`) — a liquid-fall block, NOT a cage. The cage tiles are 275+ (`TileID.cs:1179-2049`); the 92-id draw union is `drawdata_furniture::is_cage_kind`. |
| "cage items 2440+" | 2440/2441 are the Green/Pink **Jellyfish Jar** (`ItemID.cs:6563-6565`), tiles 317/318. The big-cage items start at **2162** (Bunny Cage, `ItemID.cs:6007`); the small-cage/bottle rows run 1993/2005/2174/2190/2207/2741/3070+/3254+/4364+/4848+/4963/5351 etc. |
| "cage release right-click" | **Does not exist.** `Player.TileInteractionsUse` (`Player.cs:32118-33586`) has ZERO cage arms (census below). Critter release is **LMB item-use** via `ItemCheck_ReleaseCritter` (`:44057-44074`) — already ported. Recovering a critter from a *placed* cage is the **break-drop cascade** (`Check6x3`/`Check3x2`), not a right-click. |
| "Main AddLight tile-345 style arm" | Firefly/bottle light is a **`TileLightScanner` constant** (`TileLightScanner.cs:917-934`), NOT `Main.AddLight`. `Main.AddLight`'s only tile-adjacent arms are the gem-tree foliage (`TileDrawing.cs:8127/8172/8220`) and the hook chains (`Main.cs:29016-29028`). |

## 1. Place census — ALREADY WIRED (nothing missing)

* Every cage/bottle item row in `itemdata.rs` carries `use_style: 1`,
  `tile: Some(<cage>)`, `place_style: 0` (census: all 58 sampled rows pass;
  zero exceptions). E.g. 2162→275, 2440→317, 1993→270, 4848→581, 5351→660.
* Every cage tile kind has a generated `TileObjectDef`
  (`tileobject/registry.rs`; census: all 92 cage ids + 270/271 present, zero
  missing). `Style6x3`/`StyleSmallCage`/the 1x2 bottle defs carry
  `uses_custom_can_place: true` (`TileObjectData.cs:2667-2782`/`:3673-3781`).
* Route: `world_sim/editing.rs::use_tile_item` → `edit::place_item_tile` →
  `routes_through_tile_object` (true for cages) → `tileobject::can_place`/
  `place` (`Player.cs:39771-39778`). The legacy `WorldGen.PlaceTile` cage arms
  (`Place6x3` `WorldGen.cs:59884-59887`, `Place3x2` `:59888-59891`) are the
  fallback the registry route supersedes — vanilla places cages through the
  registry too. **No place arm is missing.**

## 2. Release census — LMB item-use ported; break-drop cascade is the gap

* **LMB release** (`makeNPC > 0` block `Player.cs:43614-43625` →
  `ItemCheck_ReleaseCritter` `:44057-44074` → `NPC.ReleaseNPC`
  `NPC.cs:80106-80167` + `CanReleaseNPCs` `:80070-80104`): ported whole in
  `bug_net.rs` (`release_plan`/`release_position`/`can_release_npcs`/
  `release_npc`), wired ahead of the reach gate in `editing.rs:2116`
  (S1-P-i36). Cage items have `make_npc: 0` (they place a tile, they do not
  release on use) — the release arm correctly never fires for them.
* **Right-click on a placed cage**: census of `TileInteractionsUse`'s every
  `Main.tile[myX,myY].type == …` arm (`Player.cs:32118-33586`) finds **no cage
  id**. `TileInteractionsMouseOver` (`:33587+`) likewise. There is no
  right-click release in vanilla; inventing one would violate the golden rule.
* **The real "get the critter back" path** is breaking the cage:
  `WorldGen.KillTile` → `SquareTileFrame` → `TileFrameImportant` →
  `Check6x3` (`WorldGen.cs:50242-50463`, dispatch `:86019-86026` for 275-281)
  and `Check3x2` (`:49187-49849`, dispatch `:85976-85989` for the small cages)
  drop the cage item (`Item.NewItem(…, 2162)` etc., `:50291-50454` /
  `:49460-49524`). All cage kinds ARE in `KILL_TILE_NO_CELL_DROP` (census: every
  cage id true), so the per-cell drop is correctly suppressed; the missing
  piece WAS the object-level `Check*` record that returns the cage item.
  **NOW IMPLEMENTED (W4-g34 break-drop cascade, this branch).** The census
  below was re-verified against the source by interpreting the
  `TileFrameImportant` nested-switch dispatch (the ILSpy goto-fallthrough
  structure) statement-by-statement:

  | Check routine | Kinds | Drop |
  | --- | --- | --- |
  | `Check6x3` (`:50242`) | **41 big cages** (275-281, 296/297, 309, 358/359, 413/414, 542, 550/551, 553/554, 558/559, 599-612, 632, 640, 643-645, 710) | the cage item (`CAGE_6X3_DROP`, `:50291-50454`) — **the critter stays INSIDE the dropped cage item; there is NO release arm in Check6x3** |
  | `Check3x2` (`:49187`) small cages | **23** (285/286, 298/299, 310, 339, 361-364, 391-394, 532/533, 538, 544, 555/556, 582, 619, 629) | the cage item (`:49460-49524`; 361-364→3073+type-361, 391-394→3254+type-391) |
  | `Check2x2` (`:48397`) water cages | **28** (fish bowls 282/505/543/598, fairy jars 568-570, butterflies 288-295/360/580/620, dragonflies 521-527, jellyfish 316-318) | the jar/bowl item (`:48570-48751`; all equal the placing item) |
  | `Check1x2Top` (`:46706`) bottles | 270/271/581/660 (+ soul bottles 572, jar 698, lantern 42) | the bottle item (`:46787-46811`) |

  41 + 23 + 28 = **92** — the full cage union (`drawdata_furniture::is_cage_kind`).
  The `Check3x2` furniture family (17/26/77/86/89/114/133/215/217/218/237/244/
  377/405/469/486/695), the echo piles (647/648/651), the fallen log (488/704),
  the oasis echo (705), the terragrim-shrine echo (706) and the sandcastle (552)
  were ported in the same pass (the brief's "port ALL missing arms"). The
  support judges were ported verbatim (the cage/platform group `:49234`, the
  altar boulder-refusal `:49270`, the fallen-log grass family `:49252`, the
  Check6x3 table-top arm `:50271`, the Check1x2Top anchor `:46760`). The
  fallen-log wood stack (`genRand.Next(10,21)`, `:49553`) rides a
  `PileLootEvent` resolved app-side. **Deviations recorded:** the
  `MysticLogFairiesEvent.FallenLogDestroyed` fairy-spawn event (`:49851`) is
  unported; the Check2x2 boulder/ceiling/chandelier arms (95/126/132/138/484/
  664/665/711-716) are separate lanes. Witnesses WRITTEN NOT RUN (code-only).

## 3. Light census — constants ported; ONE animated feed was unwired (FIXED)

* **Firefly/bottle constants**: `TileLightScanner.cs:917-934` — 270 firefly
  `[0.73,1,0.41]`, 271 lightning bug `[0.45,0.95,1]`, 581 lavafly
  `[1,0.75,0.5]`, 660 shimmerfly `TorchColor(23)`. All ported in
  `tile_light/scanner.rs:923-925` + `:102` (660 via `torch_color`). These are
  `GetTileLight` seeds, NOT `Main.AddLight`. Already wired through
  `apply_tile_light`.
* **Other cage light**: 286/619 glowing snail `[0.1,0.2,0.7]`
  (`TileLightScanner.cs:896-901`), 582/598 magma snail/lava bowl
  `[0.7,0.2,0.1]` (`:911-916`), 620 empress butterfly HSL (`:902-910`),
  568-570 fairy jars (`:544-558`), 580 hell butterfly (`:559-563`), 391
  enchanted nightcrawler (`:564-568`) — all ported (`scanner.rs:801-805`,
  `:906-926`). 360 gold butterfly has NO light arm (census: `TILE_LIGHTED`
  bit false, no `TileLightScanner` case) — correctly dark.
* **THE GAP (fixed)**: the jellyfish cages 316/317/318 have an **animated**
  light — `TileLightScanner.cs:714-768` reads `Main.jellyfishCageMode[type-316,
  idx]` and brightens while mode == 2. The scanner arm was ported
  (`scanner.rs::cage_lights` `:179-200`, reads `p.cage_mode`), but the app feed
  `app/frame/draw.rs:390` **hardcoded `cage_mode` to all-zeros** with a stale
  "`Main.AnimateTiles` is not ported" comment. The cage animator IS now wired
  (W4-g34 union, research/197) and exposes `jellyfish_mode: [[u8;25];3]`
  (`critter_cage_anim.rs:111`). **Fix:** feed `self.cage_anim.jellyfish_mode`
  into `LightParams.cage_mode` so the pulse tracks the draw frames. The
  `LightKey` already carries `cage_mode` (`lighting/params.rs:244/283`), so the
  cache invalidates correctly when the mode changes.

## 4. Changes made (this wire)

1. `crates/tr-app/src/app/frame/draw.rs` — `cage_mode` feed now reads
   `self.cage_anim.jellyfish_mode` (was all-zeros); comment corrected.
2. `crates/tr-core/src/tile_light/scanner.rs` — 2 witness tests WRITTEN NOT
   RUN: `jellyfish_cage_light_follows_the_mode_byte` (dim/bright per mode,
   all three bowl kinds, modes 1/3 stay dim) and
   `jellyfish_cage_mode_slot_is_origin_derived` (the origin-derived slot index,
   `TileLightScanner.cs:718-720`).

## 5. Verification

Code-only mode: no cargo/tests/rustfmt invoked. The wiring is a field-to-field
copy of matching types (`[[u8; CAGE_FRAMES]; 3]` both sides, `CAGE_FRAMES = 25`
in `tile_light/palette.rs:11` and `drawdata_furniture.rs:32`). The two witnesses
follow the existing `scanner.rs` test harness (`lit`/`close`/`tile`).

## 6. Break-drop cascade pass (W4-g34, this branch)

The §2 task seed is implemented across six commits (one per object-family):

1. `check_6x3` + `CAGE_6X3_DROP` — the 41 big cages (`WorldGen.cs:50242-50463`).
   Breaking any cell sweeps all 18 and drops the cage item at the trigger cell;
   the critter stays inside (no release arm). Support = `SolidTileAllowBottomSlope`
   OR active solid-top `frameY==0` (table top, `:50271`); tail ring `:50456`.
2. `check_3x2` small cages — the 23 kinds (`:49460-49524`); the `:49234`
   cage/platform support group; drop = the placing item (`item: None` → `by_tile`).
3. `check_3x2` furniture — 17 kinds (`:49432-49706`); fixed drops via `by_tile`,
   style-dependent drops via `check_3x2_furniture_drop` (benches/forge/campfire/
   Tables2/altar-replica); altar 26 sweeps but drops nothing (SmashAltar app-wired);
   the `:49270` boulder-refusal support arm.
4. `check_3x2` echo/log — 647/648/651/488/704/705/706/552 (`:49552-49678`);
   `check_3x2_echo_drop` style-keyed items; fallen-log wood stack rides a
   `PileLootEvent` (app rolls `genRand.Next(10,21)`); the `:49252` grass-family
   support arm (infected-seed via `MicroSeedFlags::WORLD_IS_INFECTED`); sandcastle
   552 sweeps but drops nothing.
5. `check_1x2_top` (was `check_lantern`) — 42/270/271/572/581/660/698
   (`:46706-46813`); the `:46760` anchor judge (flag2 platform/380, flag3 rope);
   bottle drops fixed, soul bottles `4695+style`, lantern via `by_tile`; drop
   position corrected to (trigger x, origin y).
6. `check_2x2` (was `check_candelabra_sink`) — 100/172 + the 28 water cages
   (`:48397-48799` default branch); all water-cage drops equal the placing item.

All dispatch arms verified by interpreting the `TileFrameImportant` nested-switch
(the ILSpy goto-fallthrough) statement-by-statement. `KILL_TILE_NO_CELL_DROP`
verified == the C# `KillTile_GetItemDrops` empty group (342 kinds). The drop-arms
compose at the shared `square_tile_frame` cascade position (the d54
`kill_tile_make_tile_dust` co-lane wires dust at the same `KillTile` site — no
overlap: this lane records `ObjectDrop`/`PileLootEvent`, the dust lane records
particles). 19 witness tests WRITTEN NOT RUN across the six families.

**Deviations:** `MysticLogFairiesEvent.FallenLogDestroyed` (`:49851`, the fairy-
spawn event) unported; the Check2x2 boulder/ceiling/chandelier arms (95/126/132/
138/484/664/665/711-716) and the Check3x3/Check4x2/Check3x4 extra kinds are
separate lanes (the brief scoped Check6x3/Check3x2 + the cage union).

## 7. SEAL round (2026-11-21, this branch)

### 7.1 The 92-kind union SEAL — verified landed

The break-drop cascade landed across `9c46ad94a` (Check6x3) → `69aa9a22d`
(small cages) → `88856797f` (furniture) → `b55043e87` (echo/log) → `3582392f4`
(bottles) → `9aa1ca722` (water cages) → `c0e5c63ab` (docs) → `cf85a1cf9`
(witness fixes + the Check2x2 tail ring). Re-verified at the seal tip:

* `check_6x3` — 41 kinds (`CAGE_6X3_DROP`, `:50291-50454`); the critter stays
  INSIDE the dropped cage item (census: NO release arm in Check6x3 — release
  is the LMB `ItemCheck_ReleaseCritter`, already ported in `bug_net.rs`).
* `check_3x2` — 53 kinds (5 tables + 23 small cages + 17 furniture + 8
  echo/log/sandcastle).
* `check_2x2` — 100/172 + the 28 water cages (+ 751/752 since §7.3).
* `check_1x2_top` — 7 kinds (42/270/271/572/581/660/698).
* The `TileFrameImportant` ILSpy goto-fallthrough dispatch INTERPRETED
  statement-by-statement (the brace-depth trace): the 275-281 case + the
  `:85278` guard fall-through (Check6x3), the `:85547` empty-break group →
  `goto case 14` (Check3x2), the `:85653`/`:85784` groups + the 316-318 range
  guard `:85662` (Check2x2), the `:85752-85759` group (Check1x2Top).
* 41 + 23 + 28 = 92 = `drawdata_furniture::is_cage_kind` — the full union.

### 7.2 The g34f cage-retire — verified EXECUTED

The research/197 §3 DROP list (the cage-animator union's retire half) was
re-censused at the seal tip; every g34-owned DROP item is landed:

| DROP item | State at the seal tip |
| --- | --- |
| `drawdata_cages.rs` (f14/g33 variants) | ABSENT from this branch (glob empty) — the kept module is `critter_cage_anim.rs` |
| g34 `enter_loaded_world` cage reset | GONE — `app/mod.rs:2399-2407` keeps `tile_anim.reset()` only, the PROCESS-state comment cites the static ctor |
| lane-2-only draw gate | WIDENED — `render/tiles.rs:750` reads `is_cage_kind` (92) |
| lane-2-only G6 gate | WIDENED — `app/frame/draw.rs:1033` reads `is_cage_kind` |
| post-slice-A cage position | MOVED — the rewrite rides the FIRST-switch tail before `apply_cases_a` (`render/tiles.rs:735-762`) |
| g33 live-stream warm-up | NOT on this branch — `app/mod.rs:4191` warms on a fresh `UnifiedRandom::new(world_seed)` clone |
| `is_lane2_cage_kind` | KEPT deliberately (the census-split documentation predicate + its split witness, anchors row 900) — NOT a retire target |

The f14/g33-side DROPs (their modules, dispatcher slot, key field) belong to
those branches' merges — the merger drops them wholesale per research/197 §6;
nothing on THIS branch owes them.

### 7.3 The d105 Chillet-egg task seed — PORTED

The d105 RETIRE receipt (research/129 R2, commit `926bfc377`) pinned the
egg-tile-break drop UNCLAIMED to this lane: `Check2x2` case 752 → 6142
(`WorldGen.cs:48576-48578`) + the sibling case 751 → 5667 (`:48573-48575`).
Ported here:

* **Dispatch** (`check_object`): `751 | 752 => check_2x2`. The reach was
  interpreted, not guessed: the inner switch lists 751/752 as a PLAIN `break`
  (`:85689-85691`) inside the `type < 316 || type > 318` guard's switch; the
  break exits that switch, falls past the guard's closing brace to
  `goto case 132` (`:85927`) → `Check2x2(i, j, type)` (`:85945`). The same
  route the 316-318 range guard takes. (`Check2x2Style`'s `goto case 254`
  tail at `:85950` is NOT reached — the case-132 group's `break` at `:85946`
  exits the enclosing switch first.)
* **Geometry**: both eggs are `Style2x2` (`TileObjectData.cs:4015-4017`; the
  751 carries DrawFrameOffsets for the sleeping Digtoise sprite, the 752 is
  the plain copy) — the existing `check_2x2` origin/frame/floor judge covers
  them unchanged (full height 36, solid-or-table floor, no half-bricks).
* **Drop**: `item = Some(5667)` / `Some(6142)` in the record. Neither kind
  has a placing item (`itemdata` 5667/6142 `tile: None` — the Digtoise is a
  summon item, the egg a bag), so the explicit check drop is the ONLY drop:
  751/752 are NOT in `KILL_TILE_NO_CELL_DROP` (the C# empty group ends at
  733), but the per-cell `by_tile` fallback finds no placing item — no
  double spawn.
* **Witnesses** (WRITTEN NOT RUN): `w4_g34_chillet_egg_752_break_drops_the_egg_item`,
  `w4_g34_digtoise_751_break_drops_the_sibling_item`,
  `w4_g34_egg_2x2_no_item_support_break_sweeps_without_the_drop`.
* The bag-open half stays f35's (`misc_bags::open_chillet_egg`, per the d105
  receipt) — this lane owns the TILE-break drop only.

### 7.4 The g78 ObjectDrop 32×32 centre-drop gap — FIXED ONCE

The g78 wiring contract (research/77 §3.2, commit `82bc71ea8`) censused the
PRE-EXISTING gap shared by every `ObjectDrop` consumer: the C# `Item.NewItem`
rect enters ONLY through the centre conversion (`Item.cs:49649-49651` —
`center = (X + Width/2, Y + Height/2)`), while the port spawned at the tile
TOP-LEFT (~15 px left/up of the C# centre for the 32×32 furniture box). Fixed
ONCE at the seam, not per-check:

* **Core**: `ObjectDrop::spawn_center_px` (`edit/kill.rs`) — the rect census
  per kind: 32×32 (+16/+16) for the furniture/cage/object checks, 16×16
  (+8/+8) for the doors (`DropDoorItem` `:69414`) + the on-table 1×1 per-cell
  drop (`KillTile_DropItems` `:63955` — 13/33/49/78), 48×32 (+24/+16) for the
  oasis echo 705 (`:49454`) + the worm cages 391-394 (`:49481`).
* **The orb un-pre-centred**: `check_orb` carried `x: ox+1, y: oy+1` (a
  pre-centred tile) — the C# spawns at the rect top-left `(num, num2)`
  (`:54229` etc.); the record now carries `(ox, oy)` like every other check
  and the seam centres it. Net spawn position UNCHANGED for the orb.
* **Consumers** (all five): `break_drop_events` (the event stream now carries
  PIXEL positions — object records centre, every other record keeps the
  tile top-left pixel the port has always spawned, so the per-cell/tree/pot/
  herb/pile spawns stay byte-identical and the RNG order is untouched), the
  `break_tile` spawn loop (`spawn_item_drop_px`), `doors::interaction_events`
  (+ its spawn loop), the two liquid-wipe drains (`editing.rs` +
  `recall_teleport.rs`), and the actuation-drop drain (`world_sim/wiring.rs`).
* **Deliberate scope**: the NON-object lanes (per-cell/tree/pot/herb/pile)
  keep their standing top-left spawn — their own rect-centre conversion is a
  separate lane (the g78 contract scoped the ObjectDrop seam only).
* **Witnesses** (WRITTEN NOT RUN): `w4_g34_object_drop_spawn_center_matches_the_csharp_rects`
  (the 9-case rect census) + the re-pinned `test_DW_3_2_*` / `test_DW_4_1_*`
  coordinate assertions (pixel centres).

## 8. The cage-family bible (merger assist)

One table per cage subsystem — the C# anchor, the port site, the owner, the
ruling. The merger keeps THIS branch's cage wiring wholesale (research/197
§6); the f14/g33 duplicates drop at their merges.

| Subsystem | C# anchor | Port (this branch) | Owner / ruling |
| --- | --- | --- | --- |
| Place (all 92 + bottles) | `TileObjectData` Style6x3/StyleSmallCage/1x2 defs (`:2667-2782`/`:3673-3781`), `Player.cs:39771-39778` | `tileobject::can_place`/`place` via `routes_through_tile_object` (§1) | LANDED pre-wave; no cage place arm missing |
| LMB critter release | `ItemCheck_ReleaseCritter` `Player.cs:44057-44074` + `NPC.ReleaseNPC` `:80106-80167` | `bug_net.rs` (`release_plan`/`release_npc`), wired `editing.rs:2116` (S1-P-i36) | LANDED; cage items `make_npc: 0` — never fires for them |
| Right-click release | NONE in vanilla (`TileInteractionsUse` census, §0/§2) | none — correctly absent | NO arm to port; inventing one violates the golden rule |
| Break-drop: 41 big cages | `Check6x3` `WorldGen.cs:50242-50463` | `check_6x3` + `CAGE_6X3_DROP` | THIS branch (W4-g34); critter stays inside |
| Break-drop: 23 small cages | `Check3x2` `:49460-49524` + the `:49234` support group | `check_3x2` small-cage arm | THIS branch |
| Break-drop: 28 water cages | `Check2x2` default `:48397-48799` | `check_2x2` | THIS branch |
| Break-drop: bottles/lantern | `Check1x2Top` `:46706-46813` | `check_1x2_top` | THIS branch |
| Break-drop: furniture/echo/log | `Check3x2` `:49432-49706` | `check_3x2` furniture/echo arms | THIS branch; fairy-spawn event unported (deviation) |
| Break-drop: Palworld eggs 751/752 | `Check2x2` `:48573-48578` | `check_2x2` egg arm (§7.3) | THIS branch (the d105 seed) |
| Drop spawn geometry | `Item.NewItem` centre `Item.cs:49649-49651` | `ObjectDrop::spawn_center_px` (§7.4) | THIS branch (the g78 FIX-ONCE); the g78 monolith lane composes — its `check_monolith` records ride the same seam |
| Animator (34 tables) | `AnimateTiles_CritterCages` `Main.cs:13511-16596` + owl/dragonfly helpers | `critter_cage_anim.rs` (whole port) | THIS branch keeps; f14/g33 `drawdata_cages.rs` DROP at their merges (research/197) |
| Draw rewrite (92 ids) | FIRST `GetTileDrawData` switch tail `TileDrawing.cs:5629-5907` | `cage_draw_data` at the slice-B/A composite position (`render/tiles.rs:750`) | THIS branch; f14 order folded in |
| Warm-up | `Initialize_TileAndNPCData2` `Main.cs:6802-6807` | `App::new`/`flow_app` fresh world-seed clone (`app/mod.rs:4191`) | THIS branch (f14 semantics); g33 live-stream form DROP |
| Lifetime | static ctor `Main.cs:66502`/`:66901-66972` | PROCESS state — no world-entry reset | THIS branch; the old reset DROPped |
| Light: constants | `TileLightScanner.cs:896-934` | `tile_light/scanner.rs` | LANDED pre-wave |
| Light: jellyfish pulse | `TileLightScanner.cs:714-768` reads `jellyfishCageMode` | `scanner.rs::cage_lights` fed `cage_anim.jellyfish_mode` (`app/frame/draw.rs:393`) | THIS branch (the feed fix) |
| Egg bag-open | `Player.TryOpenContainer` (the 58-kind dispatch) | `misc_bags::open_chillet_egg` | f35's (landed `0fd244e79`/`00a5b27db`) — NOT this lane |

**Co-lane boundaries:** the g78 monolith `check_monolith` (research/77 §3.2)
adds a NEW `Check2xX` sibling + dispatch arm — hunk-disjoint from this lane's
`check_2x2` egg arm; its records ride the §7.4 seam unchanged (the 32×32
default). The d54 dust lane composes at the same `KillTile` site (particles
vs records, §6). The g78 FIX-ONCE means the monolith lane must NOT re-centre
per-check — the seam owns it.
