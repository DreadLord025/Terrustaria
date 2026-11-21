# 68 — Cage critter items: place / release / light census (W4-g34-drawdata-wire-2)

Worker: code-only subagent on `task/W4-g34-drawdata-wire-2` (tip `5bd63950`).
Read-only census + ONE wiring fix; no builds/tests run. All C# cites are
`decompiled/Terraria/…`; Rust cites are `crates/…` at the branch tip.

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
  `:49460-49524`). The port's kill cascade (`edit/kill.rs::check_object`) has
  **no cage arms** — `check_3x2` covers only 14/87/88/186/187, and there is no
  `check_6x3`. All cage kinds ARE in `KILL_TILE_NO_CELL_DROP` (census: every
  cage id true), so the per-cell drop is correctly suppressed; the missing
  piece is the object-level `Check*` record that returns the cage item.
  **This is a real, sizeable gap — but it is the break-drop lane, not the
  "right-click release" the brief asked for, and porting `Check6x3`/the
  `Check3x2` cage arms is a multi-object cascade task outside this wire's
  scope.** Recorded here as a task seed; NOT implemented (no invention).

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
follow the existing `scanner.rs` test harness (`lit`/`close`/`tile`). The
break-drop cascade gap (§2) is recorded, not implemented.
