# 68 — Cage critter items: place / release / light census (W4-g34-drawdata-wire-2)

Worker: code-only subagent on `task/W4-g34-drawdata-wire-2`. Read-only census +
wiring fixes; no builds/tests run. All C# cites are `decompiled/Terraria/…`;
Rust cites are `crates/…` at the branch tip.

**Update (break-drop cascade pass):** §2's task seed is now IMPLEMENTED — the
full 92-kind cage union's break-drop cascade (`Check6x3`/`Check3x2`/`Check2x2`/
`Check1x2Top`) plus the `Check3x2` furniture/echo/log family. See §2's table
and §6 below.

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
  (W4-g34 union, research/67) and exposes `jellyfish_mode: [[u8;25];3]`
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
