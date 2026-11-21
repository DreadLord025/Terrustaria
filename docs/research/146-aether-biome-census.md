# 146 — The AETHER biome census (W4-e19 final, SHIMMER item 3)

The C# "Aether" biome is `SceneMetrics.ZoneShimmer` — there is no separate
`ZoneAether` field (a full `SceneMetrics.cs` scan: the only aether-named
members are `ShimmerTileCount`/`ShimmerMonolithState`/`ZoneShimmer`/
`UndergroundForShimmering`). The music track is `MusicID.Aether = 91`
(`ID/MusicID.cs:200`).

## The census (C# → Rust, per seam)

| Seam | C# | Rust state | Action |
| --- | --- | --- | --- |
| Zone metric | `SceneMetrics.cs:594` (`ShimmerTileCount = _liquidCounts[3]`), `:252` (`>= ShimmerTileThreshold`), `:921` (300), `:704-705` (depth window + `!ZoneDungeon`) | `tr-core/scene.rs:795-800` + `scene_scan.rs:177-179` — verbatim | NONE (already faithful) |
| Music gate | `Main.cs:12666-12669` (`ZoneShimmer → 91`), after eclipse `:12644`, before town `:12670` | `audio.rs:2708-2712` + `frame/world.rs:194` — verbatim | witness added (`w4_e19_the_aether_zone_picks_track_91_in_ladder_order`) |
| TOW music 72 | `Main.cs:11951-11953` (the drunk-world ladder) | unported — the swap gate `Main.cs:11055-11074` (`drunkWorld && !remixWorld && !getGoodWorld`) has no port | DEFERRED (the swap lane's; the normal ladder always runs) |
| Bg skip | `Main.cs:52800-52805` (`shimmerAlpha == 1` → return; `> 0` → `ugBackTransition = 0`) | `render/bg.rs::draw_background` fed 0.0 | **PORTED** (the unclaimed small): the live `scene_state.shimmer_alpha` threaded through `draw_sky`; return at 1 |
| ugBackTransition zero | `Main.cs:52830-52833` | the port pins `undergroundBackground` 0 (no cross-fade) | no-op here; **g72 merge note**: its live `UgBackState` must zero the transition on the `> 0` arm (its `ug_back_step` already takes `shimmer_alpha`) |
| Natural star dim/skip | `Main.cs:62086-62089` (skip at 1), `:63107-63110` (`color *= 1 - num`, the `artificial: false` arm) | `render/sky.rs` star loop drew unconditionally | **PORTED** (the unclaimed small, g69d-star co-lane): skip at 1, dim by `1 - alpha` |
| Star channel lerp | `Main.cs:63090-63102` (`i % 3` picks G/B/R, each `* (1f - num * 0f)` = ×1) | — | NO-OP in the decompile (`num * 0f`); not ported, cited |
| Artificial star pass | `Main.cs:62199-62206` (`DrawStarsInBackground(artificial: true)` over the black quad, `color *= num`) | unported | **g69d-star's claim** — needs the star fall/respawn machine |
| Underground star update | `Main.cs:17256-17274` (`Star.UpdateStars()` + `Next(numStars)` + `Next(90) == 0` fall/respawn while `shimmerAlpha > 0` below the surface) | the port's `make_stars` field is static (no per-tick update) | **g69d-star's claim** — the RNG draws would shift the Update stream; needs the star machine first |
| Monolith ramp terms | `SceneState.cs:285-289` (state 1 ON / state 2 OFF-wins / the 5347 shader / player shimmering) | `scene.rs::update_shimmer` read only zone + player | **PORTED** (the unclaimed small): state 1 + state 2 terms; the shader term reads false (ApplyShader unported, M2's accessory seam) |
| Monolith scan | `SceneMetrics.cs:547-549` (tile 658 `frameY / 54`) | `scene.rs:682` — verbatim | NONE |
| liquidAlpha[13] blue boost | `TileLightScanner.cs:3160-3166` | `tile_light/mod.rs:153-155` fed 0 | **DEAD CODE census**: no C# writer raises index 13 (DrawWaters `:57476-57489` skips style 14 via `IsLiquidStyleWater` `:57456-57463`; the capture loop `:56701-56707` writes 0..=10). The 0 feed is FAITHFUL; the old ANALYSIS "biome does not exist yet" note retired |
| Black backdrop quad | `Main.cs:62184-62187` (`Black * shimmerAlpha` full-screen after the world) | the port's DrawBlack is folded into the tile dark-cell merge (`tiles.rs:769-798`) | DEFERRED to the g72/g69d merge lane — needs a `screen_fx` seam; the aether's signature black bg is the artificial-star pass's backdrop |
| Shimmer colour shine | `Main.cs:21637-21643`/`:21734-21742` (`shimmerShine` (1.2, 1, 1.6) tile/wall tint) | unported | g76-shimmer-draw's claim (the liquid draw lane) |
| Sky colour dim | `Main.cs:63047-63050` (`colorOfTheSkies *= 1 - shimmerAlpha`) | `frame.rs:339` already multiplies `(1 - shimmer_alpha)` | NONE (already faithful) |
| Ambience mute | `Main.cs:11538`+ (the shimmer mutes the ambience tracks) | `audio.rs:4510-4527` — verbatim | NONE |

## Witnesses (WRITTEN NOT RUN — code-only)

1. `tr-core/scene.rs::w4_e19_the_shimmer_monolith_states_gate_the_ramp` —
   state 1 ramps the alpha without the zone; state 2 suppresses even the
   zone; state 0 keeps the zone trigger.
2. `tr-app/audio.rs::w4_e19_the_aether_zone_picks_track_91_in_ladder_order` —
   91 beats the sandstorm/underworld/jungle tail; the eclipse (27) and the
   boss scan (93) outrank it.

The bg-skip + star-dim arms are draw-path seams (no headless witness rig for
the sprite passes); their gates are the two `== 1.0` / `1.0 - alpha`
expressions cited above, re-verified against the C# windows.

## Co-lane merge notes

- **g72-bg-layers-3**: its `draw_background` already takes `shimmer_alpha`
  and returns at 1 (the same arm ported here) — keep-one at merge, either
  side is faithful; its `ug_back_step(ug_back, picked, shimmer_alpha)` must
  keep the `> 0` transition-zero arm (`Main.cs:52830-52833`).
- **g69d-star** (the star lane): the natural-star dim/skip ported here is
  the `artificial: false` half; the artificial pass + the underground
  update arm (the `Next(numStars)`/`Next(90)` draws) stay its claim.
- **g76-shimmer-draw**: the `shimmerShine` tile/wall tint + the liquid
  draw's aether arm stay its claim.
