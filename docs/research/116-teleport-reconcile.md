# 116 — TeleportEffect reconcile: the 4-way union plan (W4-g11)

W4-g11 reconcile phase, code-only. `Main.TeleportEffect`
(`Main.cs:13233-13464`) was ported by FOUR lanes; this doc is the per-style
comparison + the wiring-pattern record + the keep-one union plan. Every claim
re-derived from the C# (the whole switch re-read, plus `Player.Teleport`
`Player.cs:37414-37496`, `NPC.Teleport` `NPC.cs:81035-81058`,
`NPC.UpdateNPC_TeleportVisuals` `NPC.cs:92194-92241`,
`TeleportPylonsSystem.SpawnInWorldDust` `TeleportPylonsSystem.cs:327-393`,
`Main.hslToRgb` `Main.cs:47890-47942`, `PortalHelper.GetPortalColor`
`PortalHelper.cs:373-392`, `Dust.dustWater` `Dust.cs:402-418`). No code
changes to any lane in this commit — the union is the merger's worklist.

Arms compared:

| arm | commit | body | status |
| --- | --- | --- | --- |
| g11 (this branch) | `36602419` (+ `13a1c48e`) | `main_dust.rs::teleport_effect` styles 0/1/2/3/4/5/7 + `player_teleport`/`npc_teleport` replays | **WIRED** (player tick + wiring pads) |
| g61 | `a9628928` (bodies `6345f7b7`/`8d23e298`) | `npc_spawn_visuals.rs::teleport_effect` — the widest **11** arms (0/1/2/3/4/5/7/9/10/11/13) + `npc_teleport` driver + `update_npc_teleport_visuals` + `make_transition_effects_for_texture_changes` | UNWIRED (generic `SpawnVisualsEnv`, `#![allow(dead_code)]`) |
| d54 batch-4 | `48343a4c` | `world_dust_main.rs::teleport_effect` — **10** styles (0/1/2/3/4/5/7/10/11/13, no 9) | LATENT (sounds deferred to the caller) |
| g12 batch-2 | `665cd1e8` | `main_dust_b.rs::teleport_effect_case10/11/13` — **3** styles (10/11/13) | STAGED (the e2/d50 pattern) |

## 1. Per-style table

Draw sequences were walked arm-by-arm against the C# statement order
(argument draws → allocator internals → post-writes, sentinel included).
**All four arms are draw-equivalent on every style they share** — the
differences are shape/coverage/plumbing, not stream order.

| style | C# | g11 (wired) | g61 (widest) | d54 (latent) | g12 (staged) | notes |
| --- | --- | --- | --- | --- | --- | --- |
| 0 | `:13237-13255` Item6 first, `(W*H/5)*mult` type-159, `Next(20,70)*0.01` scale ASSIGN + j<10/j<5 bumps | ✓ | ✓ | ✓ (sound deferred) | — | identical ladders; integer `W*H/5` truncation everywhere |
| 1 | `:13256-13274` Item8 first, type 164, same shape | ✓ | ✓ | ✓ (sound deferred) | — | identical |
| 2 | `:13275-13283` NO sound, 50×mult type-58 GhostWhite α150 scale 1.2, vel ×0.5 | ✓ | ✓ | ✓ | — | identical |
| 3 | `:13284-13317` Item6 first, 50×mult type-180 noGravity, 5×`Next(3)` damp + 2×`Next(3)` double + `Next(2)` fadeIn/scale overwrite + ×0.8 tail | ✓ | ✓ (the two double arms as `for _ in 0..2` — same draws) | ✓ (sound deferred) | — | identical draw census (8 rolls + the gated pair per dust) |
| 4 | `:13318-13333` Item8 first, `(W*H/5)*mult` type-263, `GetPortalColor(extraInfo)` + noLight/noGravity/1.2/0.4, **per-dust `color.A = 255`** (`:13330`) | ✓ (colour hoisted, `d.color.a = 255` per dust — faithful) | ✓ (colour + A255 hoisted once — observation-equal, `GetPortalColor` draws nothing) | ✓ (per-dust recompute — faithful shape) | — | `portal_color` impls differ: g11 `tile_light::hsl_to_rgb` + `(c*255).round()` (double-round recovers the bytes; the palette's f64-inner arithmetic is observation-equal at these constants), g61 `ui::color::hsl_to_rgb` (**the C#-exact f32→f64 widen + `round_ties_even` + i64 byte wrap — canonical**), d54 own f64 `hsl_to_rgb_bytes` (observation-equal at s=1, l∈{0.5,0.6}, not the C# float-widen shape) |
| 5 | `:13334-13351` NO leading sound, 100×mult `dustWater()` Direct h+24 column, the 7-write chain, **`PlaySound(19, Center, 0)` AFTER** (`:13349`) | ✓ — but `water_style` recomputed inline (`biome::calculate_water_style(world, camera)`); the C# reads the **`Main.waterStyle` static** | ✓ (`water_style` param — canonical; 5\|7 merged arm, same draws) | ✓ (`water_style` param; sound deferred) | — | g11's inline recompute is a deviation (a mid-pass biome change would flip the dust type); union takes the stored/param value |
| 7 | `:13352-13369` type 35, 50×mult, same column, Item8 AFTER | ✓ | ✓ | ✓ (sound deferred) | — | identical |
| 9 | `:13370-13380` Inflate(15,15) FIRST, 100×mult `SpawnInWorldDust(extraInfo, rect)`, Item6 after at the INFLATED centre | — (cited: "the pylon lane's") | ✓ **only arm** — `pylon_spawn_in_world_dust` (`TeleportPylonsSystem.cs:327-393`: the 11-entry colour table, type 43 α254 scale 0.5, float-ctor clamp incl. the Snow/Mushroom 1.2 blue, vel ×0.1 then Y−0.2) | — | — | g61's pylon port is the union's style-9 body |
| 10 | `:13381-13418` Inflate, 60×mult, fadeIn/scale locals FIRST, hue `NextFloat` in the `newColor` argument, Direct 267 α127, the DEAD `Next(20,70)` ladder (draws kept), the n<0.8·count velocity pull + `NextFloat`, flags + local overwrites, `dustIndex != 6000` CloneDust (×0.65 pair + white), Item8 AFTER | — (`_ => {}` fall-through) | ✓ (clone guard reads the SPAWNED slot's `dust_index` — the C# shape) | ✓ (same slot-read guard) | ✓ (guard reads the returned `idx != DUST_SENTINEL` — observation-equal: a refusal returns 6000 and the sentinel slot's `dust_index` is 6000) | all three draw-equivalent; g61/g12 use the canonical `ui::color::hsl_to_rgb`, d54 its f64 helper; **`otherPosition` required** (Entry leg = newPos, Exit leg = old pos) |
| 11 | `:13419-13446` NO sound, 50 (NO mult), `Next(4)` colour pick FIRST, position pair, Perfect 267, noGravity/colour/vel×2/scale 0.8+`NextFloat`×0.6/fadeIn 0.5 | — | ✓ | ✓ (keeps the unreachable `_ => Green` default — dead, harmless) | ✓ | identical; the initial `Color.Green` is always overwritten |
| 13 | `:13447-13462` Item8 FIRST (un-inflated centre), 21 (NO mult), position pair, `Next(1,3)` scale ARGUMENT, 2×2 type-27 α100, vel×1.75 + i%3 ×1.5, noLightEmittance/noGravity | — | ✓ | ✓ (sound deferred) | ✓ | identical |
| 6/8/12 | no switch arm | fall-through ✓ | fall-through ✓ | fall-through ✓ | n/a | style 12 exists only in the NPC per-tick decay (below) |

Sounds: g11 plays through the `Audio` façade on the caller's pass stream
(`item_cue` → `play_npc_item_cue`, style 5 → `play_splash_sound(0, …)` — the
case-19 `Next(-10,11)` pitch); g61 through the `SpawnVisualsEnv::play_sound`
callback AT the C# call position (the wiring round resolves the same façade);
d54 defers them to the caller entirely (a latent-body convention — the union
must NOT keep the deferral, the cue draws ride mid-switch); g12 uses the
façade directly. All agree on position (before/after the loop) and integer
`Rectangle.Center` halves.

## 2. The wiring pattern — g11's Entry-OLD → Exit-NEW queue is canonical

g11 is the only wired arm; its seam shape is the union's wiring contract:

* **`Player.Teleport`** (`Player.cs:37414-37496`):
  `Player::teleport_style_with_extra` (`player.rs:9508`) queues the effect
  pair (`pending_teleport_effect`: old/new boxes, style, `extra_info2` =
  OLD portal colour for style 4, `extra_info`, `mult = Clamp(1 -
  teleportTime*0.99, 0.01, 1)` read PRE-move) + runs the movement core
  inline + the grants; the player tick (`world_sim/player.rs:1564`) takes
  the queue into a `PlayerTeleportReq` on `RngKey::UpdatePlayers`
  (`Main.cs:17754`), drained post-step beside `force_clear` by
  `main_dust::player_teleport` — **Entry at the OLD box (`:37444`), move,
  Exit at the NEW box (`:37489`), grants (`:37490-37491`)**. The same-tick
  deferral is observation-identical (nothing probes the dust between).
* **Wiring pads** (`Wiring.cs:3154`/`:3170`): `queue_teleport_effect`
  (queue ONLY — the move defers to the replay so the Entry reads the true
  OLD box), the replay vec drained on `RngKey::UpdateWorldGen`; the camera
  transition stays inline (`Player.cs:37463`, style-0 delay 0). NPC hits
  queue `NpcTeleportReq` in pool order (the C# `k` walk).
* **`NPC.Teleport`** (`NPC.cs:81035-81058`): `main_dust::npc_teleport`
  replays Entry-at-OLD (`extra_info2` = OLD `last_portal_color_index` for
  style 4, `mult = teleportTime > 0 ? 0.3 : 1` PRE-move), move, the style-4
  latch BEFORE the Exit, Exit-at-NEW, grants. The `Npc` teleport fields
  (`npc.rs:1256-1263`: `teleport_time`/`teleport_style_id`/
  `last_portal_color_index`) are g11's — they replace g61's
  `(f32, i32, i32)` triple parameter.
* **Per-tick decay**: `Player.UpdateTeleportVisuals` (`Player.cs:16613-
  16670`) stays the W4-e1 dust lane's; `NPC.UpdateNPC_TeleportVisuals`
  (`NPC.cs:92194-92241`, styles 0/4/12/13 + the shared −0.005 tail) is
  ported by g61 (`update_npc_teleport_visuals`, the style-12 witness gate
  unconditional per `a9628928`) — UNWIRED; the union drains it at the
  `UpdateNPC` tail (`NPC.cs:90322`) on the `UpdateNPCs` stream.
* The recall/mirror cluster (`recall_teleport.rs`) calls `Player.Spawn`,
  NOT `Teleport` — outside this reconcile (g11's module doc note stands).

### Wiring gaps found in the g11 arm (the union's fix list)

1. **Styles 10/11/13 fall through** g11's match (`_ => {}`): the REACHABLE
   callers emit no dust today — the Queen-Slime hook (`projectile.rs:17064`,
   `Projectile.cs:52775` → `Teleport(vector, 10)`, still the movement
   stand-in `p.teleport`) and the Shellphone/no-space style-11 path
   (`player.rs:9576` `shellphone_spawn`). The union routes them into the
   surviving body (g61's arms / g12's staged cases).
2. **`otherPosition` is not carried**: `PlayerTeleportReq`/`NpcTeleportReq`
   and the queue tuple have no `other_position` — style 10 needs it (Entry
   leg = `newPos` `Player.cs:37444`/`NPC.cs:81044`, Exit leg = the OLD
   position `:37489`/`:81051`). g61's driver passes it correctly on both
   legs; extend the reqs + queue (the old box is already captured).
3. **Style 9's `extraInfo2`** (`Player.cs:37437-37441`:
   `lastTeleportPylonStyleUsed = extraInfo`, then `extraInfo2` = it) is
   missing from `queue_teleport_effect` (only style 4 handled). Add the
   field + arm with the pylon lane.
4. **Style 5's `water_style`** inline recompute → the stored
   `Main.waterStyle` analogue (param), per §1.
5. **`Player.Teleport` head/tail arms not in the g11 replay** (each cited,
   none draws dust RNG except the style-8 pair):
   `_funkytownAchievementCheckCooldown = 100` / `environmentBuffImmunityTimer
   = 4` (`:37418-37419`, fields unported); the style-8 double Item6 at
   old/new (`:37450-37453`, TWO cue draws on the pass stream);
   `PressurePlateHelper.UpdatePlayerPosition` ×2 (`:37445`/`:37482`),
   `netOffset`/`SetOrRequestSpectating`/`DoUnbreakableWallScan`
   (`:37447-37449`); the myPlayer camera/map arm (`:37456-37474` — the
   wiring pad starts the transition inline, but the PLAYER-TICK callers
   (style 1 → delay 10, `:37459-37462`) do not; `mapTime`/`maxQ`/`renderNow`
   unported); style-4 `portalPhysicsFlag = true` / `gravity = 0`
   (`:37479-37480`, cited skip — the portal physics lane's);
   `ResetAdvancedShadows` + 3× `UpdateSocialShadow` + `oldPosition` Bleh
   fixer (`:37483-37488` — g11's `prev_x/prev_y` write stands in).
6. The grants are written twice on the inline path (queue-time + replay) —
   idempotent, documented in `13a1c48e`; the union keeps the replay write
   as the C#-position one.

## 3. The union plan (keep-one)

**Body: KEEP g61's `npc_spawn_visuals.rs::teleport_effect`** — the widest
(11 arms incl. the pylon style 9), the canonical `ui::color::hsl_to_rgb`,
sounds at the C# position via the callback, the faithful style-10 slot-read
clone guard. **Wiring: KEEP g11's seams** — the Entry-OLD→Exit-NEW queue,
the `PlayerTeleportReq`/`NpcTeleportReq` replays, the `RngKey` per-pass
streams, the `Npc` teleport fields, the pad drains.

Concretely, at merge:

* g11's `main_dust::teleport_effect(app, rect, style, extra, mult, key)`
  survives as the **App-level dispatcher**: it builds a `SpawnVisualsEnv`
  from the disjoint App fields (world/dust/ctx/gore + `app.rngs.stream(key)`)
  and a `play_sound` closure resolving the Audio façade (`(2, n)` →
  `play_npc_item_cue(n, …)`, `(19, 0)` → `play_splash_sound(0, …)` — the
  same pitch draws g11's witnesses pin), then calls g61's body for ALL
  styles. `water_style` becomes a dispatcher input (the stored static).
* **Retire:** g11's seven arm bodies + `nd`/`nd_full`/`nd_direct`/
  `item_cue`/`portal_color` (the dispatcher no longer needs them); d54's
  ten-style `teleport_effect` + `hsl_to_rgb_bytes` + `portal_color` +
  `next_vector2_from_rectangle` (the snowing half is `83`'s scope); g12's
  three `teleport_effect_case*` fns (the bone-arm/Moon-Lord halves of
  `main_dust_b.rs` are OUTSIDE this reconcile — they stay g12's); g61's
  env-level `npc_teleport` driver (g11's App-level replay supersedes it,
  carrying `other_position` per gap 2).
* **Extend:** the reqs/queue with `other_position` (gap 2) + the style-9
  pylon field (gap 3); the player-tick camera transition (gap 5); the
  head/tail arms land with their own lanes, cited above.
* **Wire:** `update_npc_teleport_visuals` at the `UpdateNPC` tail
  (`NPC.cs:90322`, `UpdateNPCs` stream, the g11 `Npc` fields); the
  Queen-Slime hook re-points at `teleport_style(…, 10)` (the queue path)
  once the dispatcher covers style 10.
* **Witness union (all WRITTEN NOT RUN, code-only):** g11's 7 style-port
  witnesses + 4 wiring witnesses in `main_dust.rs` and the pad integration
  witness (`wiring.rs:1020 w4_g11_the_pads_teleport_with_the_effect_pair`)
  — the seam contract, they keep pinning the dispatcher; g61's 9 teleport
  witnesses of its 18
  (`style0/3/4/9/10/11`, `npc_teleport_two_sided_burst_and_latch`, the two
  `teleport_visuals_*` decay witnesses — the draw sequences are identical
  per §1, so they transfer with rig adaptation; the poof/mimic/transform/
  alt-texture 8 stay with their own seams), g12's 4 teleport witnesses
  (`case10_mirrors…`, `case10_mult_truncates…`, `case11_mirrors…`,
  `case13_sound_first…` — same oracles; its bone-arm/Moon-Lord 3 stay
  g12's), d54's 7 teleport + `hsl_and_portal_colors` (the colour oracle
  re-pins to `ui::color::hsl_to_rgb`; its 2 snowing witnesses go to `83`).
  The `w4_g11_the_g12_styles_stay_untouched` witness INVERTS at the union
  (styles 10/11/13 now fire through the dispatcher).

## Verification

Code-only mode per the task brief: no cargo/tests/graphify. The per-style
rows were derived by reading the full C# switch + drivers cited above and
diffing the four port bodies (`git show 36602419`/`a9628928`/`48343a4c`/
`665cd1e8`) arm-by-arm; the wiring rows by reading this branch's
`player.rs:9490-9567`, `world_sim/player.rs:1556-1576`,
`world_sim/wiring.rs:303-371`, `main_dust.rs:505-605`,
`projectile.rs:17049-17066`. The union plan is a merger worklist, WRITTEN
NOT APPLIED; no other lane's code was touched.
