# 65 — Main.cs dust sites, batch 1 (W4-g11): ledger + witnesses

Status: LANDED (code-only, 2026-11-21, `task/W4-g11-dust-main-sites-1`).
The lane split is W4-g12's claim doc
(`64-dust-main-sites-claims.md`, commit 73a85c01) — probed in-flight before
any edit; g11 owns sites 1-8 (lines 13088-13358), g12 sites 9-16, g13
sites 17-23. This doc records g11's half only; the shared census lives in
64.

## Sites (all in `crates/tr-app/src/main_dust.rs`)

| # | C# line | Family | Arm | Port |
| - | ------- | ------ | --- | ---- |
| 1 | Main.cs:13088 | NewDust | `snowing()` :13032-13116 | `snowing(env, rng, ctx, world, pool)` |
| 2 | :13243 | NewDust | `TeleportEffect` style 0 | `teleport_effect(.., 0, ..)` |
| 3 | :13262 | NewDust | style 1 | idem |
| 4 | :13280 | NewDust | style 2 | idem |
| 5 | :13290 | NewDust | style 3 | idem |
| 6 | :13324 | NewDust | style 4 | idem |
| 7 | :13340 | NewDustDirect | style 5 | idem |
| 8 | :13358 | NewDustDirect | style 7 | idem |

Boundary: 13358 is g11's (style 7 whole); 13390 (style 10) is g12's.
Styles 6/8/12 have no C# arm; style 9 spawns through
`TeleportPylonsSystem.SpawnInWorldDust` (not a Main.cs NewDust site — the
pylon lane's). The witness `w4_g11_the_g12_styles_stay_untouched` pins the
boundary: styles 6/8/9/10/11/12/13 draw nothing and spend no stream step.

## Contract

- Pool: d50's `tr_core::particles::DustPool::new_dust` /
  `new_dust_direct` (`Dust.cs:75-256` verbatim) — pressure bands, default
  block, ChildSafety, per-type tweaks all ride the pool. The sentinel
  (6000) takes the post-spawn writes exactly like the C#
  (`w4_g11_a_refused_spawn_still_writes_the_sentinel`).
- RNG, one slot per C# pass: `snowing` runs at `Main.cs:17238`, BEFORE
  the `DoUpdateInWorld` swap (`:17732`) — the bare `Main.rand`, the port's
  `NamedRngs::global()`. `TeleportEffect` takes the caller's pass key
  (`Player.Teleport` → UpdatePlayers, `NPC.Teleport` → the NPC pass,
  `Wiring.Teleport` → UpdateWorldGen). The Item6/Item8 cues and the
  style-5 Splash ride the same stream (the C# draws them mid-pass); the
  audio-only `GetRandomPitch` NextFloat stays on the audio stream (the
  `play_npc_item_cue` façade).
- Argument order: explicit `Main.rand` argument draws FIRST, then the
  pool internals, then the post-writes (the d51/e2 convention).
- `Main.snowDust` (Main.cs:621) is a Main static the pool does not carry
  (d50's cited skip) — the budget reads the live type-76 population, the
  `SandStormCount` stand-in pattern (sand.rs:115).
- `PortalHelper.GetPortalColor` (PortalHelper.cs:373-392): the SP arm
  only (`netMode == 0`); the entry orange pins byte-exact (255,184,0),
  the exit blue rides the ported `tile_light::hsl_to_rgb`.
- Dropped with citations: `snowing`'s per-iteration try/catch (no
  exceptions), `TeleportEffect`'s unread `side` param, `otherPosition`
  (feeds only g12's style 10), the MP portal hue ladder, `Color.GhostWhite`
  spelled (248,248,255).

## Wiring status (staged — the W4-e2 batch-1 precedent)

The entry points are staged (`#![allow(dead_code)]`), witnesses pin them:
- Site 1's live stand-in is `rain.rs`'s `Snowflake` vector (`spawn_snow`);
  it keeps running until the dust-76 physics/draw wiring round (d50's
  `dust_physics.rs` carries the 76 arm; the pool's generic draw pass is
  the draw lane's). The `SnowingEnv` fields map 1:1 onto the App state
  the wiring caller gathers (scene_metrics, rain.visual_alpha,
  wind.speed_current, gfx_quality.quality, the Deerclops scan).
- Sites 2-8's callers (`Player.Teleport`/`NPC.Teleport`/`Wiring.Teleport`)
  have their effect halves as the port's documented render seams
  (`player.rs::teleport_style` — "the camera/effect/map halves are render
  seams"; `recall_teleport` runs its own recall dust, NOT
  `TeleportEffect`). The wiring follows with those seams; `EffectRect`
  is the entity `getRect()`.

## Witnesses (11, WRITTEN NOT RUN — code-only)

`main_dust.rs::tests`: styles 0/1 scale ladder after the cue; style 2
soundless GhostWhite; style 3 full shrink ladder (5×Next(3) + 2 gates +
fadeIn re-roll); style 4 portal pair colours; styles 5/7 cue AFTER the
column (+24 box, dustWater/35 tweaks); the g12 boundary; the sentinel
contract; snowing's four gates; the dry-arm draw-sequence mirror (incl.
the cull/sentinel branch); the storm-arm mirror with Deerclops (4-seed
sweep, local budget counter); the live-76 budget break. Every mirror
ends with a full-stream equality (`UnifiedRandom` clone compare — the
d51 `stream_of` pattern), so any extra/missing/reordered draw fails.
