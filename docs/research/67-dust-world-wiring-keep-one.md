# 67 — World-lane dust WIRING (W4-f6): keep-one + seam census

Status: WIRING DELIVERED (W4-f6, the world lane's wiring phase, branch
`task/W4-f6-dust-world-3`). The 40 staged emitters (research/65, commits
`885a6bfa`/`d6f555b8`/`9dec91b0`/`680ac9d8`) are now called from the live
machines at their C# positions. Code-only: WRITTEN NOT RUN, the new
wiring witnesses are owed a compile window before merge.

## The e4 co-lane keep-one (mount 1-19 ⊂ f6's 27)

`origin/task/W4-e4-dust-sites-world-2` (research/66) staged a SECOND
independent port of the same Mount.cs sites: their "mount 1-19"
(`0b9ad269` batch 3 + `9eb72737` batch 4) covers `Mount.cs:3113-5071` —
the drill beam pair, the unicorn/wolf/witch dash rings + bodies, the
basilisk charge pair, the golf-cart smoke, the four minecart effects, the
witch-broom trinket pair and the drill-mount trail. **Every one of those
19 sites is inside f6's 27-site Mount census** (research/65): f6 ports
3113-6493 (the full movement + effects + spawn/dismount set).

**Keep-one decision: f6's `mount_dust.rs` is canonical for ALL Mount.cs
dust.** Rationale:

- f6's set is the superset (27 ⊃ 19) and is the one WIRED to the live
  `MountState` machine (`frame_extras`, `queue_mount_effect_dust`,
  `queue_spawn_dust`, the `pending_mount_dust` drain). e4's emitters are
  staged-only (no live caller in their branch).
- f6's emitters carry the post-spawn writes + the draw-mirror witnesses
  pinned to the same `DustPool` contract; merging both would double-draw
  every Mount site (two `NewDust` calls per C# site = a determinism break).

**Merger contract:** when e4 lands, DROP its `mount_dust.rs` Mount
emitters (the 19) in favour of f6's; KEEP e4's
`orchestrator_dust.rs` (ParticleOrchestrator 41-61, `fc2f64a2`/`e0d6d870`)
— that file is e4's exclusive lane (f6 never ports orchestrator sites).
The e4 claim doc's "mount 5071 is e4's, 5099+ stays f6's" boundary is
SUPERSEDED: f6 owns the whole Mount file (its 27-site census predates the
wiring and was pushed first, `960fc28d`). e4 wires orch 41-61 only.

The shared `GetFairyQueenWeaponsColorFull` / `NextVector2Square/Circular`
/ `DirectionTo` / `witch_broom_trinket_rotation` helpers: f6's
`mount_dust.rs` copies are canonical (e4's `witch_broom_trinket_rotation`
is the same RNG-free port; keep f6's, drop e4's duplicate).

## Seam census (where each staged emitter now fires)

| Emitter (research/65) | Live seam | RNG pass |
| --------------------- | --------- | -------- |
| Mount spawn/dismount (6382/6391/6404/6413/6493) | `player.rs::queue_spawn_dust` ← `set_mount` (:6300) / `dismount` (:6169) / `start_death` (KillMe :38907) | UpdatePlayers |
| Mount failed-dismount (6339) | `player.rs::quick_mount` ladder (:6058) | UpdatePlayers |
| Mount movement (3878/3896/3939/3970/4000/4023/4103/4123) | `mount.rs::frame_extras` ← `player.rs::mount_update_frame` (the 5 PlayerFrame/TryLanding sites) | UpdatePlayers |
| Mount drill beam (3113/3170) | STAGED — `UseDrill`/DrillMountData is M2-MOUNT-4 (no live caller) | UpdatePlayers (when wired) |
| Mount effects (4703/4719/4750/4782/4824/4849/5061/5071/5099/5149) | `player.rs::queue_mount_effect_dust` ← `update_mount` (UpdateEffects :4804) | UpdatePlayers |
| Mount golf-cart smoke (4626) | DEAD — `EmitGolfCartSmoke` has zero callers (verified) | n/a |
| Chest unlock (420) | `chest.rs::unlock_dust` ← both `unlock_chest` sites (:33178/:33204) | **UpdatePlayers** (re-pin: ItemCheck→TileInteractionsUse, NOT bare global) |
| Utils.PoofOfSmoke (1089) | `bug_net.rs::catch_npc` StatuePoof/MysticFrog (:79912) | UpdatePlayers |
| Pylon teleport (390) | `render/pylons.rs::draw_pylons_trophies_void` pylon gate (:8669-8673) | private `tile_draw_rand` (gates) + bare global (spawn) |
| Void-lens (48/63) | `render/pylons.rs` void-lens Update (:8717-8720) | bare global (`Main.rand`) |
| Lava bubbles (560/564) | `render/tiles.rs::draw_tiles` ← `liquid_draw::build_with_bubbles` (:552-577) | private `liquid_rand` (`LiquidRenderer._random`) |
| PopupText fancy flash (461/470/479/488) | STAGED — g57's PopupText module is on `task/W4-g57-ui-reforge-anim`, NOT in this branch; the reforge contexts are RNG-zero (`PrepareEffects` ItemCraft-only), so the flash arm has no live trigger here | UpdateFloatingText (when g57 lands) |
| Rain splash (122) | SKIPPED — d130 (`task/W4-d130-weather-ambience`) owns the rain machine | bare global (draw) |
| Nebula level-up (3697) | STAGED — MessageBuffer case 102 is MP-only net dispatch; f12's buff-scan (`task/W4-f12-buff-frame-wiring`) owns the nebula tier state, the net-102 handler is unported in SP | net dispatch pass |

## New private streams (the clock-seed → world-seed deviation)

`App.tile_draw_rand` (`TileDrawing._rand`, `TileDrawing.cs:104`/:191) and
`App.liquid_rand` (`LiquidRenderer._random`, `LiquidRenderer.cs:132`) are
the two PRIVATE draw-side streams the C# seeds from `Environment.TickCount`.
The port seeds both from the world seed (the `mine_crack_rng` determinism
convention, ANALYSIS "The tile-crack overlay"). Neither is a `SwapRandom`
key. The pylon dust GATES draw `tile_draw_rand`; the pylon SPAWN + the
void-lens draw the bare global (`NamedRngs::global()`); the lava bubbles
draw `liquid_rand`.

### The g68 `tile_draw_rand` same-name collision — KEEP ONE FIELD

`origin/task/W4-g68-weather-wind-leaves` (tip `d247a1c9`, wiring-1
`ea9e784f`) independently added `App.tile_draw_rand` for the
`EmitLiquidDrops` per-draw gates (`foliage::emit_liquid_drops_area`,
`TileDrawing.cs:7609` `_rand.Next(num*2)`). **The C# census says these are
the SAME stream, not two:**

- `TileDrawing._rand` is ONE private field (`TileDrawing.cs:104`, seeded
  once `:191`) on the ONE `Main.TilesRenderer` instance (`Main.cs:2301`,
  constructed once `Main.cs:6541`). The drip gate (:7609, inside
  `Draw(solidLayer:false)`'s tile switch :693) and the pylon crystal gates
  (:8669-8670, inside `PostDrawTiles(false)` → `DrawTeleportationPylons`
  :424) both read `this._rand` on that instance — renderer-lifetime,
  shared, no second field anywhere in the class.
- `Mount.cs` has ZERO private RNG fields (grepped `private.*Random|new
  UnifiedRandom|new FastRandom|_rand\b` — no hits): every Mount dust roll
  rides `Main.rand` (the UpdatePlayers pass). The "two-fields rename"
  alternative does not apply — there is no Mount draw stream to separate.
- The distinct `LiquidRenderer._random` (f6's `App.liquid_rand`) stays a
  SEPARATE field: different C# class, different instance, different seed
  site (`LiquidRenderer.cs:132`). That two-field split is already correct.

**Keep-one decision: ONE `App.tile_draw_rand`, BOTH consumers** (g68's
drip gates + f6's pylon gates). Both branches declare the identical field
(`pub(crate) tile_draw_rand: tr_core::UnifiedRandom`, seeded
`UnifiedRandom::new(world_seed as i32)`), so the declaration conflict
resolves by keeping ONE declaration with the union of the doc comments —
NO rename, NO second field. Splitting into two streams would be a
determinism break (the C# interleaves both consumers' draws on one
stream).

**Merge-order pin (the interleaving):** C# runs `Draw(solidLayer:false)`
(the per-tile drip gates, row-major) BEFORE `PostDrawTiles(false)` (the
per-visible-pylon gates). The port's two seams already sit in that order
in `app/frame/draw.rs`: g68's `emit_liquid_drops_area` inserts right after
the `tile_cracks` overlay (:1221 at f6 tip), f6's
`draw_pylons_trophies_void` follows (:1243). The merger keeps drip-first,
pylon-second, both drawing `&mut self.tile_draw_rand` in the same frame;
neither call may move inside the G6-cached range (both are per-DRAW
cadence, `WorldFrameKey` untouched). The d60 tile-pass-attach contract
(g68's rescan stand-in) does not change this order — the attached cells
replay the same row-major sequence.

## New player state (the wiring's fields)

- `Player.basilisk_charge` (`Player.cs:842`) — PLAYER state, survives the
  dismount (folded player-side before `frame_extras`).
- `Player.time_for_visuals` / `wind_for_visuals` / `wind_in_place` — the
  Pixie tail's even-tick gate + the Witch Broom trinket sway inputs, fed
  per-tick by `world_sim/player.rs` (the `frame_inputs` convention).
- `MountState.dash_latch` — the 44/45 `_mountSpecificData` bool, reset on
  dismount (`Mount.cs:6151`/`:6212`).
- `Player.pending_mount_dust` — the request channel (the
  `pending_sticky_dust` convention), drained on UpdatePlayers.

## Witnesses (WRITTEN NOT RUN, code-only)

The wiring adds drain-parity witnesses (the request channel changes no
draws vs the direct emitter): `w4_f6_drain_spawn_matches_the_direct_wolf_emitter`,
`w4_f6_drain_failed_dismount_matches_the_direct_emitter`,
`w4_f6_drain_basilisk_matches_the_direct_pair`,
`w4_f6_drain_pixie_tail_matches_the_direct_emitter`,
`w4_f6_pylon_dust_box_centers_the_crystal_rect`,
`w4_f6_bubble_cells_match_build_and_collect_lava`. The 32 original
draw-mirror witnesses (research/65) still stand.
