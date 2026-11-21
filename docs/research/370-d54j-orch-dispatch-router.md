# 370 — W4-d54j orchestrator dispatch router: route table + contracts

Status: LANDING doc (code-only round, 2026-11-21, branch
`task/W4-d54-dust-sites-world`). **Number 370 per registry-150 §9** —
chain: 355 PROVISIONAL self-pick (per the dispatch directive: 150
foreign-dirty at sweep time → skip reading it, use 355 provisional,
note owed) → the registry owner's section-9 wave landed mid-flight
(`880b9ecbe`..`534e9ad4d`) seating 355→d11 (`355-w4-d11-retire-final`,
DONE `7820a9466`) and assigning **d54j → 370** ("rename owed 355→370
when writer lands"). This landing IS the writer — the rename is
EXECUTED here (355-d54j-orch-dispatch-router → 370-d54j-orch-dispatch-
router); the registry row's DONE receipt is the owner's to append.

## What landed

`crates/tr-app/src/orch_dispatch.rs` — the
`ParticleOrchestrator.SpawnParticlesDirect` port
(`ParticleOrchestrator.cs:162-416`) for the d54-owned first-40 lane
(research/67), shaped after e4's `orchestrator_dust.rs::spawn_orchestra`
(`:775` match). `mod orch_dispatch;` registered in main.rs. The phase-1
skeleton (8 arms, 2 witnesses) landed `e4671580c`; this round completes
the remaining 16 routing destinations + the four line wrappers. **7
witnesses WRITTEN NOT RUN** (code-only mode — no
cargo/tests/rustfmt/graphify).

**Merger shape:** this module IS the `_ => {}` arm of e4's router. e4
keeps its 15 kind bytes (1/3/5/7/8/9/12/13/14/15/25/37/55/72/77); d54
owns the 24 arms below; every other enum byte is an unported `Spawn_*`
body (no draws — the documented lane-boundary drop). At the merge the
two compose into ONE `SpawnParticlesDirect` port — never run both for
one kind (double-draw risk, research/131 §8.3 step 7). The kind sets are
DISJOINT (verified byte-for-byte against
`ParticleOrchestraType.cs:3-88` declaration order).

## Route table (24 arms, kind byte = enum declaration index)

| Kind | Name (`ParticleOrchestraType.cs`) | C# dispatch | Body (d54 module) |
| ---- | --------------------------------- | ----------- | ----------------- |
| 81 | ArmletOfRuinRecharge | `:411-413` | `orch_dust::armlet_of_ruin_recharge` (`:418-435`) — InvokedPlayer |
| 80 | PyroclasticStoneRecharge | `:408-410` | `orch_dust::pyroclastic_stone_recharge` (`:437-452`) — InvokedPlayer |
| 79 | SnappingStoneRecharge | `:405-407` | `orch_dust::snapping_stone_recharge` (`:454-470`) — InvokedPlayer |
| 69 | SnappingStone | `:384-386` | `orch_dust::snapping_stone` (`:593-646`) — movement.x |
| 74 | SnappingStoneMelee | `:390-392` | `orch_dust::snapping_stone_melee` (`:648-731`) — position |
| 75 | SnappingStoneSummon | `:393-395` | `orch_dust::snapping_stone_summon` (`:733-813`) — position |
| 68 | PaladinsHammerShockwave | `:381-383` | `orch_dust::paladins_hammer_shockwave` (`:815-879`) — gen, disco |
| 63 | PaladinsShieldHit | `:369-371` | `orch_dust_lightning::paladins_shield_hit` (`:902-972`) — TWO player indices from truncated position, `Utils.IndexInRange` (`Utils.cs:2441-2448`), SP index 0 |
| 61 | StormLightning | `:360-362` | `orch_dust_lightning::storm_lightning_dust` (`:1052-1161`) — the caller pre-resolves EndPosition into `settings.position` (the `:1038` write-back seam) + white PackedValue (`:1029-1033`) + weapon = invoker != 255 (`:1034`); gen, disco |
| 47 | RainbowBoulder1 | `:321-323` | `orch_dust_boulder::rainbow_boulder1` (`:1502-1556`) |
| 49 | RainbowBoulder3 | `:327-329` | `orch_dust_boulder::rainbow_boulder3` (`:1585-1646`) |
| 50 | RainbowBoulder4 | `:330-332` | `orch_dust_boulder::rainbow_boulder4` (`:1431-1500`), pet=false |
| 60 | RainbowBoulderPetBounce | `:357-359` | same body, **pet=true** (the C# passes `pet: true`) |
| 44 | MoonLordWhipHit | `:312-314` | `orch_dust_boulder::moon_lord_whip` (`:1648-1703`) |
| 42 | DeadCellsBarrelExplosion | `:306-308` | `orch_dust_boulder::dead_cells_barrel_explosion` (`:1705-1779`) |
| 41 | DeadCellsFlint | `:303-305` | `orch_dust_boulder::dead_cells_flint` (`:1781-1889`) — gen, disco |
| 38 | ShadowOrbExplosion | `:294-296` | `orch_dust_boulder::shadow_orb_explosion` (`:2028-2118`) |
| 39 | UFOLaser | `:297-299` | router-local `ufo_laser` (`:1950-1957`) — 2×type-160 per step, `Next(2)` colour gates: LimeGreen (50,205,50) / CornflowerBlue (100,149,237) |
| 56 | MagnetSphereBolt | `:345-347` | router-local (`:1959-1968`) — 4×type-160 per step, offsets 0/.25/.5/.75, no colour draws |
| 57 | HeatRay | `:348-350` | router-local (`:1970-1979`) — the Magnet shape, type 162 |
| 58 | ShadowbeamHostile | `:351-353` | router-local (`:1981-1994`) — 3×type-173, velocityScalar **0.03334** |
| 59 | ShadowbeamFriendly | `:354-356` | same body, hostile=false — velocityScalar **0.25** |
| 35 | DeadCellsDownDashExplosion | `:285-287` | `orch_dust_town::dead_cells_down_dash_explosion` (`:2120-2213`) — info, gen, disco |
| 32 | ShimmerTownNPC | `:276-278` | `orch_dust_town::shimmer_town_npc` (`:2254-2315`) |
| 29 | Digestion | `:270-272` | `orch_dust_town::digestion` (`:2328-2347`) — movement.x |
| 27 | LoadoutChange | `:264-266` | `orch_dust_town::load_out_change` (`:2370-2399`) — player.active gate, Hitbox ints, **gravDir folds to 1** (the port has no gravity flip, player.rs doc) |
| 36 | DeadCellsBarnacleShotFiring | `:288-290` | `orch_dust_town::barnacle_shot_firing` (`:2521-2549`) |
| 26 | TownSlimeTransform | `:261-263` | `Spawn_TownSlimeTransform` (`:2401-2415`) info switch: **0→Nerdy** (`:2569`), **1→Copper** (`:2578`), **2→Elder** (`:2551`); other info draws NOTHING (no default arm) |

(28 kind bytes = the d54j-catalog's **24 routed arms**: the recharge
trio shares ONE arm, 50/60 share the boulder4 body (pet flag), 58/59
share the shadowbeam body (hostile flag) — 26 literal match arms in the
port, 24 routing destinations in the catalog's counting.)

The four line wrappers route through the batch-7 helpers
`spawn_in_line` (`SpawnHelper_SpawnInLine` `:1996-2014`: step = max(4,
|SafeNormalize(Δ)·info/1000|), walk |Δ|) → `single_line_dust`
(`SpawnHelper_SpawnSingleLineDust` `:2016-2026`, site 28: 1×1 NewDust,
position snap-back, scale Next(70,110)·0.013, velocity ×0.2, colour
write only when non-default).

## Contracts

- **Router drains the caller-pass stream** (S1-A-10): `rng` =
  `Main.rand` of the invoking pass, `gen` = `WorldGen.genRand`'s
  runtime stand-in (`World::frame_rng`), `disco` = the
  `Main.DiscoR/G/B` snapshot for the tile-sweep delegation. The router
  itself draws NOTHING — every draw belongs to the routed body.
- **Argument draws inline at the queue site** (the e4 S1-E-21c
  convention): settings-argument draws run at the C# call site when
  the request queues; the bodies defer to the pass-tail drain calling
  this router on the SAME stream. Wiring the queue
  (`netmodules.rs:447`'s Particles drop / e4's `pending_orchestra`
  shape) is the wiring wave — this module is LATENT (`#![allow(dead_code)]`,
  the e4 precedent).
- **StormLightning renderer seam**: the C#'s `:1038`
  `settings.PositionInWorld = stormLightningParticle.EndPosition`
  write-back means the CALLER pre-resolves the bolt end into
  `settings.position` before the drain (the router signature stays the
  directive's 9-parameter shape — no extra ctx struct); the
  camera-punch NextFloat (`:1041`) + `Main.NewLightning` thunder draws
  (`Main.cs:58712-58726`) are consumed by the caller in order.
- **Flint empty-world early return**: kind 41 with no tile hit draws
  NOTHING (`flag` false → return before the 30-dust tail, `:1874`);
  the arm-count witness pins it.
- **Keep-ones owed at the merger**: `D54Settings` retires into e4's
  `OrchestraSettings`; `d54_kind` folds into e4's `orchestra_kind`;
  the two routers compose into ONE dispatch (e4 arm order first or
  C# switch order — either, the arms are disjoint so order is
  draw-inert).

## Witnesses (7, WRITTEN NOT RUN)

1. `w4_d54j_arm_count_pin` — the 27 drawing kind bytes advance the
   stream on a live player / 16px line; kind 41 (Flint) pins the
   empty-world early return (zero draws); e4's 15 bytes + 29 unported
   strays all drop with zero draws.
2. `w4_d54j_router_armlet_draw_order_replay` — kind 81 drains exactly
   the ten type-15 blocks (50-iteration probe parity).
3. `w4_d54j_line_wrappers_draw_the_step_ledger` — 16px line, info 0 →
   4 steps; UFO 2 gates/step, Magnet/HeatRay/Shadowbeam none; per-kind
   live-slot census 8/16/16/12/12.
4. `w4_d54j_town_slime_transform_switches_on_info` — 0→Nerdy 60
   type-4 blocks (no colour draw), 1→Copper 40 / 2→Elder 30 type-43
   (no clones), unknown info zero draws.
5. `w4_d54j_loadout_gates_on_the_active_player` — missing/inactive
   player zero draws; live player drains the 40+5 perfect ledger.
6. `w4_d54j_shield_hit_resolves_the_position_indices` — (0,0) fires
   both arms (8 blocks); out-of-range zero draws.
7. `w4_d54j_storm_lightning_routes_the_resolved_position` — the
   caller-resolved position routes into the body; twin-stream + gen +
   slot-census parity against the port's own `storm_lightning_dust`
   (the delegation is the oracle).

## Also this round

research/131 §8.2 corrections applied in place + an addendum:
named machines 87+36 → **79+36**; Kill() remainder 234 → **CLOSED**
(g65 `f6e07bb90`/`c356cb99c`, 406 = 399+7, 316-kind FINAL drain, 0
unclaimed); e4 UNWIRED → **WIRED** (wiring batches 5-8 through SEAL
`c38af30f3`; the mount half still drops to f6; the reforge tail keeps
prefix.rs count-only as its ONE path).

Code-only mode: no cargo/tests/rustfmt/graphify invoked; witnesses
WRITTEN NOT RUN, owed a compile window in the merge-verify wave.
