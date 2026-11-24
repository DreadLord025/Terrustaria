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

## Final sweep — every Main.cs dust-family site vs the union-table (dust-retire phase)

Fresh `Dust\.NewDust|NewDustPerfect|NewDustDirect|CloneDust|Main\.dust\[|
Dust\.[A-Za-z]` census over `decompiled/Terraria/Main.cs` (direct file,
no junction recursion). Result: **ZERO unclaimed sites** — every
spawn/draw/pool window is owned by the union-table (research/64's 23-site
split + g13 research/68's residual census, re-verified here):

| Main.cs window | family | owner at the union |
| --- | --- | --- |
| `:13088` (snowing) | site 1 | d130 `snow.rs` canonical (83 §-receipt: F1-F7 APPLIED at `099df89e2`/`16a792d56`); g11 body DEPRECATED → merger DROP |
| `:13243/13262/13280/13290/13324/13340/13358` | sites 2-8 (styles 0/1/2/3/4/5/7) | g11 `teleport_dust.rs` (g61 widest-11 body) + the seams dispatcher — KEEP (116 §3) |
| `:13390/:13410 (CloneDust)/:13438/:13452` | sites 9-11 + the clone (styles 10/11/13) | SAME g61 body covers them (`teleport_dust.rs` arms 10/11/13, the spawned-slot clone guard) — g12's `teleport_effect_case*` staged fns DROP (116 §4) |
| `:22744/:22820` | sites 12/13 (bone-arm chains) | g12 `main_dust_b.rs` `skeletron_hand_bone_dust`/`prime_limb_bone_dust` — KEEP |
| `:24736/24748/24752` | sites 14-16 (Moon Lord core) | g12 `moon_lord_core_dust` — KEEP; the feed FUSES with g13's tentacle feed into ONE caller pass (116 §6) |
| `:24756` | site 17 (tentacle tip) | g13 `moon_lord_tentacle_tip_puff` — KEEP |
| `:27381/:27384` | sites 18/19 (Tavernkeep ale pair) | g13 `tavernkeep_ale_sparkle` — KEEP |
| `:38664` | site 20 (world-item sparkle) | g13 `world_item_sparkle` + `item_sparkle_feed` — KEEP |
| `:57766/:57789/:57793` | sites 21-23 (retro liquid/lava) | g13 `liquid_surface_sparkle`/`lava_bubble_pair` + retro feed — KEEP |
| `:6687-6688` | pool ctor | S1-E-1 `DustPool::new` (g13 research/68 census row) |
| `:18046-18070` | `UpdateWorld_Dust` + crash re-init | S1-E-3 `minecart::tick`'s single `update_dust` pass; the `:18065` re-init is the catch-arm recovery (port: no exceptions — cited skip, research/68) |
| `:18055` `Dust.UpdateDust` | physics/lights/draw | d50 `dust_physics.rs` + S1-E-5/g13 `dust_lights*` + the draw lane (94/94 lights closed, research/68 §3) |
| `:38832-39022` | DrawDust cull + tails | g67/d130 splash+cull union — d130e `16a792d56` folded `splash_cull_rect`/`dust_in_cull_rect` into `draw_splashes` AND `snow::draw` (type 76 shares the loop) |
| `:56735-56744` | DrawCapture snow cull | g13 `capture_snow_cull` (`1af79847`) — KEEP (116 §6; RNG-free, reads live 76 — d130's flakes are its input) |
| `:57785` `Dust.lavaBubbles < 200` | counter gate | g13 retro feed reads the pool counter (research/68 §4: both readers accounted) |
| `:66985` `new Dust[6001]` | pool alloc | S1-E-1 pool sizing (6000 + sentinel) |
| `:66700` `Main.snowDust = 0` | static init | ctor-zero; the LIVE reset is `Dust.cs:428` in `UpdateDust` (83 §-receipt counter-feed check) |

Dispatcher/feeds coverage: every g11 seam (`player_teleport`/`npc_teleport`
replays, the pad/statue queue, the Queen-Slime/Shellphone drains) feeds the
ONE g61 body; g13's feeds (`item_sparkle_feed`, `retro_liquid_dust_feed`,
`moon_lord_tentacle_feed`, `tavernkeep_ale_feed`) are draw-path callers on
`rngs.global()` — no g11-owned site is orphaned, no foreign site is
double-claimed. The teleport-dispatcher boundary holds: g61b's poof/
mimic/transform families never call `TeleportEffect` (116 §5 census), so
no Main-dust site hides in that lane.

Verification: code-only (no cargo/tests/rustfmt/graphify). The census
greps ran against the direct `decompiled\Terraria\Main.cs` path; the
owner columns re-read g12 `main_dust_b.rs` + g13 `main_dust.rs` module
docs at their origin tips and this branch's `teleport_dust.rs`.
