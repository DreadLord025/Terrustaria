# 185 — DrawData wiring lane 1 (W4-g33): the critter-cage animator + draw wire

> **RENUMBERED 65 → 185 (batch-H merge, registry research/150 §3 round-1:
> f17's `65-kinds-coverage-1.md` keeps 65, g33's fork → 185; slug kept).**
> **Batch-H merge note:** the g33 `drawdata_cages.rs` animator + its
> live-stream warm-up + draw wiring DROPPED at this merge per research/197
> §3/§6 — the g34 union (`critter_cage_anim.rs`, the 92-id `cage_draw_data`
> wiring, the fresh-clone warm-up, the 40 per-machine witnesses ported +
> the all-92 t0 parity pin) is the keep-one. This doc survives as the
> lane-1 scope record + the documented limitations register (the 620
> legacy lid pass unwired, the negative-probe clamp, the fused-layer
> flag-clear note — all still true of the merged result).
>
> Wave 4 · 2026-11-21 · branch `task/W4-g33-drawdata-wire-1` @ base a725acb0 ·
> code-only (no builds/tests run; witnesses WRITTEN NOT RUN).

## Scope taken (FIRST half of the d61-d64 unwired list)

The d61-d64 census (371/371 arms EXIST, 0 MISSING) left these wiring gaps:

1. **Critter cages — 92 ids frozen at row 0** (d63 §summary item 1, d64
   batches 1-3): the `drawdata_furniture.rs` arms are complete but
   `furniture_post_pass`/`cage_draw_data` had no live caller and the
   `Main.*CageFrame` tables never moved (`AnimateTiles_CritterCages`
   deferred at `drawdata_anim.rs:455`). **THIS LANE.**
2. TILE-DRAW-2 glow bind (`_glow` at `tiles.rs`, ~30 glow records incl.
   634/100/33/637, the martian/meteor furniture rows, 412's two-rect
   cross-fade `TileDrawing.cs:1345-1356`, the platform glow side-fills
   `:1367-1401`). **g34 (second half).**
3. Chest rewrite `CacheSpecialDraws_Part2` (`chest_draw_rewrite` /
   `special_tiles::chest_frame_adjust`, d57/d58 seam) + the 21-columns-48/49
   glow rect. **g34.**
4. Deviations: torch `solid_above` probe fidelity (d61 batch 5),
   637 painted-sheet glow bind, 564 pre-clear glow rect y (d64 R15),
   temp-animation registry 349/593/594/441/468 (d63 §summary item 2),
   83 paint-sheet override. **g34.**

## What landed

* `crates/tr-core/src/drawdata_cages.rs` — `Main.AnimateTiles_CritterCages`
  (`Main.cs:13511-16596`) verbatim: all 34 table loops in source order +
  `UpdateOwlCageFrame` (`:16617-16666`) + `UpdateDragonflyJarFrame`
  (`:16668-16727`). The `Main.critterCage` gate (`:13513`), the
  `Initialize_TileAndNPCData2` 3600-call warm-up (`:6802-6807`) as
  `CageAnimator::warm_up`, `frame_of` (the `cage_draw_data` lookup),
  `anim_key` (G6) and `is_cage_kind` (the G6 gate set). Keep-one folds:
  squirrel≡squirrelOrange, mallard≡duck, bird≡blueBird≡redBird,
  snail/snail2 (two roll-range pairs) — the C# bodies are byte-identical.
  Verbatim quirks preserved: penguin 10/11 band `> 3` (`:15166`), frog
  18-23 outside-gate wrap (`:15711`), macaw `> 5`/`> 6` gates, ladybug
  unconditional base `Next(10,12)` (`:15990`), butterfly unconditional
  base `Next(3,16)` (`:16411`), jellyfish separate-ifs mode chain,
  pufferfish post-step inflate pick + 183-tick cycle, owl `||`
  short-circuit `Next(80)`, seagull/rat counter-mod glide ladders.
* **Draw wire** (`render/tiles.rs`): `cage_draw_data` replaces the static
  cage rows at the C# FIRST-switch position — after `draw_data`, BEFORE
  `apply_world_corrections`/`cases_b`/`cases_a` (the glow switch reads the
  animated `addFrY`, e.g. firefly jars 568-570 / butterfly 580). The
  proximity flag is cleared at the pass head (`TileDrawing.cs:474`) and
  set per cage cell inside the drawdata resolution (`:5672` etc. — C#
  sets it in `GetTileDrawData`, before the visibility gates).
* **Tick wire** (`app/frame/world.rs::animate_tiles_phase`): the cage
  animator runs at the `AnimateTiles` TAIL (`Main.cs:19303`), same 60 Hz
  step, same `Main.rand` stand-in, right after the tile rows.
* **State** (`app/mod.rs`): `App::cage_anim`, warmed up once at
  construction on the global stream; NOT reset on world entry (C# process
  state — `Main.cs:8393-8394` resets only `tileFrame`).
* **G6**: `WorldFrameKey::tile_anim` FNV-mixes `CageAnimator::anim_key()`
  when a cage kind is active in the view rect (`is_cage_kind`); cage-free
  and animator-free views keep the pre-wiring cache behaviour bit-for-bit.
* 3 `anchors.txt` rows; 40 module witnesses + 2 wiring witnesses
  (source-order pin, t0 static-parity over all 92 cage ids) — all
  WRITTEN NOT RUN (code-only).

## Known limitations (documented, for the verify wave)

* The 620 special-pass butterfly draw (`DrawSpecialTilesLegacy`,
  `special_tiles.rs::draw_butterfly_cage`) stays UNWIRED (the legacy
  special-point pass is unported); the 620 cage-table arm IS wired, so
  its `critterCage` flag contribution rides the table arm.
* Negative cage-probe slots (the `% cageFrames` sign quirk) clamp to
  slot 0 in `frame_of` instead of throwing (unreachable in-world).
* The warm-up rides the world-seeded global stand-in (the documented
  `Main.rand` determinism deviation, `rng.rs`), not `DateTime.Now.Ticks`.
* The port's single combined tile pass clears `critterCage` once per
  frame; C# clears it at the non-solid `Draw` head only — net-identical
  because the solid pass never clears it and both passes set it per cage.
