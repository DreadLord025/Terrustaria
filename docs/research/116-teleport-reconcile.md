# 116 — TeleportEffect reconcile: the 4-way union plan (W4-g11)

**STATUS: SEALED (§8, 2026-11-21)** — EXECUTED + FINAL + RETIRE + the
four seal receipts: this lane's teleport-union census (`9a9ac1fd`/
`98283df0`/`2e978b279`), the g61d spawn-seal mutual at `6e922fc6`
(pushed — the §7.1 push-owed flag RETIRED), the d54 bible
(research/131 §8.3 item 4, the `main_dust.rs` THREE-WAY resolve per
§6's per-fn table), and the teleport-family bible below. No further
g11 commits planned; the merger's worklist is §7.2 unchanged.

**Prior status: EXECUTED + FINAL PHASE** (union-exec commits `56afc0c0` +
`8540a8ad`, docs `9a9ac1fd`; final phase: the g61b co-lane cross-verify +
the fade PRE-AI re-slot + the statue style-14 latch + the main_dust.rs
filename union-plan — §5/§6 below). The keep-one landed: g61's widest-11 body adapted per-hunk
into `crates/tr-app/src/teleport_dust.rs` (`TeleportDustEnv` = g61's
`SpawnVisualsEnv` minus the gore pool; the canonical `ui::color::hsl_to_rgb`;
sounds at the C# position via the callback; the style-10 spawned-slot clone
guard; the pylon style 9), `main_dust::teleport_effect` survives as the
App-level dispatcher (RefCell-shared pass stream — the `CellRng` `DustRng`
adapter — the Audio façade resolves `(2,n)`/`(19,0)` mid-switch), and ALL
six wiring gaps are closed (the execution record is §4 below). The three
pool helpers took g61's `DustRng` generalisation (`particles.rs`,
behaviour-identical). `update_npc_teleport_visuals` is WIRED **PRE-AI** in
the `UpdateNPC` slot preamble (`world_sim/npcs.rs`, `NPC.cs:90322` —
before `AI()` `:90423`; the g61b seam-4 order fold, §5), on the
`UpdateNPCs` stream, the g11 `Npc` fields, fed the `anim_time*60 %
216000` `timeForVisualEffects` stand-in. g61's env-level `npc_teleport` driver is
retired (the App-level replay supersedes it). **Merger drops** (§3 retire
list, restated for the merge): g12's three `main_dust_b.rs::
teleport_effect_case10/11/13` fns + their 4 witnesses (the bone-arm/Moon-Lord
halves stay g12's), d54's `world_dust_main.rs::teleport_effect` +
`hsl_to_rgb_bytes` + `portal_color` + `next_vector2_from_rectangle` + their 7
teleport witnesses + `hsl_and_portal_colors` (the snowing half is research/83's
scope). The witness union (33 adapted + 3 gap + 2 final-phase) is WRITTEN NOT
RUN (code-only).

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
  unconditional per `a9628928`) — UNWIRED; the union drains it PRE-AI in
  the `UpdateNPC` slot preamble (`NPC.cs:90322` — before `AI()` `:90423`;
  the final-phase §5 fold of g61b's seam-4 position) on the `UpdateNPCs`
  stream.
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

## 4. Execution record (union-exec phase)

Commits on `task/W4-g11-dust-main-sites-1`:

* `56afc0c0` — union 1/3: `teleport_dust.rs` (the g61 body adapted
  per-hunk from `a9628928`; `TeleportDustEnv` drops the gore pool — no
  teleport arm spawns gores; `update_npc_teleport_visuals` re-signed to
  `&mut Npc` reading the g11 fields, g61's `(f32, i32, i32)` triple
  retired). `particles.rs`: the three helpers (`new_dust_perfect`/
  `new_dust_direct`/`clone_dust`) take g61's `<R: DustRng>` generalisation
  verbatim. `App.water_style` added (the `Main.waterStyle` static,
  `Main.cs:1105`), written at the C# position (`frame/draw.rs`'s
  `DrawWaters` block, `:57475`) — gap 4's stored-static seam.
* `8540a8ad` — union 2/3: the dispatcher + the gap closures + the wiring
  + the witness union. Per gap:
  1. **Reachable-caller arms** — the Queen-Slime hook
     (`projectile.rs:17051-17073`) now calls
     `p.teleport_style(target.x, target.y, 10)` (the `p.teleport`
     movement stand-in retired); the projectile pass tail
     (`world_sim/projectiles.rs`, after `take_spear_hold`) drains the
     queued pair through `main_dust::player_teleport` on
     `RngKey::UpdateProjectiles` (`Main.cs:17775` — the C# `Teleport`
     executes inside the projectile pass). The Shellphone/no-space
     style-11 path already queued (`player.rs:9477`/`:9586`); its drain
     (`world_sim/player.rs`) now fires 50 Perfect dust through the
     dispatcher instead of falling through. Census of the other C#
     `Teleport` callers: styles 12/14 (`Player.cs:28867`,
     `NPC.cs:53266`, `Wiring.cs:2437`/`:2465`) have no dust arm — the
     dispatcher's fall-through is faithful; style 13's frog caller
     (`NPC.cs:79880`) rides the NPC pass when its lane lands (the arm
     is live in the dispatcher today).
  2. **`otherPosition`** — both replays pass it on both legs: Entry =
     `Vec2::new(new_x, new_y)` (`Player.cs:37444`/`NPC.cs:81044`), Exit =
     the OLD position (`:37489`/`:81051`). The req structs already carry
     old/new — no field addition needed; the g61 driver's legs are
     preserved in the App-level replay.
  3. **Style-9 pylon `extraInfo2`** — `Player.last_teleport_pylon_style_used`
     added (`player.rs`), `queue_teleport_effect` latches `extraInfo`
     into it and reads it back for style 9 (`Player.cs:37437-37441`);
     the Exit leg reads the latched value via the existing `extra_info`
     hand-off shape (style 9's Exit `extraInfo2` = the same latch —
     `:37489` reads `extraInfo2` unchanged after the head).
  4. **`water_style`** — the dispatcher reads `app.water_style` (the
     stored static); the inline `calculate_water_style` recompute is
     gone.
  5. **Head/tail arms** — the style-8 double Item6 (`:37450-37453`, TWO
     case-2 pitch draws old-then-new on the pass stream) and the
     myPlayer camera transition (`:37456-37463`, style 1 →
     `lerpSpeedupDelay = 10` else 0) are IN the replay; the wiring pad's
     inline transition retired (one C#-position arm serves both
     callers). The remaining head/tail arms stay cited skips:
     `_funkytownAchievementCheckCooldown`/`environmentBuffImmunityTimer`
     (fields unported), `PressurePlateHelper` ×2 / `netOffset` /
     `SetOrRequestSpectating` / `DoUnbreakableWallScan` (unported
     systems), `mapTime`/`maxQ`/`renderNow` (map-refresh lane), style-4
     `portalPhysicsFlag`/`gravity` (portal physics lane),
     `ResetAdvancedShadows`/`UpdateSocialShadow`/Bleh fixer (the
     `prev_x/prev_y` write stands in).
  6. **Double grants** — `teleport_style_with_extra` no longer writes
     `teleport_time`/`teleport_style_id` inline; the replay owns the
     `:37490-37491` write. The queue captures the PRE-move mult, so the
     effect counts are unchanged; the drain runs the same tick.
* **Wiring** — `update_npc_teleport_visuals` at the `UpdateNPC` per-slot
  tail (`world_sim/npcs.rs`, after `black_slime_darkness_touch`, before
  the look-for-home decay — `NPC.cs:90322`'s slot), on the `UpdateNPCs`
  stream with the shared dust pool/ctx; `timeForVisualEffects` passes
  0.0 (the sim-clock stand-in, cited — style 12 has no live producer;
  at rest the fade draws nothing, existing streams untouched).
  **SUPERSEDED by the final-phase §5 fold**: the C# `:90322` runs PRE-AI
  (`AI()` is `:90423`), so the tail placement was a stream-order
  deviation on any slot with a live grant; the fade now sits in the
  slot preamble and takes the `anim_time*60 % 216000` clock.
* **Witness union (33 adapted + 3 new, ALL WRITTEN NOT RUN):** g11's 12
  (7 style-port + 5 wiring in `main_dust.rs`, re-pinned to the
  dispatcher signature — the style-4 colour oracle now the canonical
  `ui::color::hsl_to_rgb`, the style-5 rig pins the stored static, the
  boundary witness INVERTED to `w4_g11_the_no_arm_styles_stay_untouched`
  [6/8/12 silent, 9/10/11/13 fire] — plus the pad integration witness
  `wiring.rs:1005` re-pinned to the replay-owned camera arm); g61's 9 (8
  in `teleport_dust.rs` under `w4_g11u_*` — the `RecordingRng` adapted
  without the `GoreRng` impl — plus
  `w4_g11_npc_teleport_style4_two_sided_burst_and_latch` in
  `main_dust.rs`, the driver witness re-rigged to the App replay + the
  `Npc` fields); g12's 4 (`w4_g11u_g12_case10/11/13_*` in `main_dust.rs`,
  same oracles, the dispatcher rig); d54's 8 (`w4_g11u_d54_*` — 7 style
  draw-census mirrors + the colour oracle re-pinned to
  `ui::color::hsl_to_rgb`). NEW gap witnesses:
  `w4_g11_style_9_carries_the_pylon_style` (gap 3),
  `w4_g11_style_8_plays_the_double_cue` (gap 5),
  `w4_g11_the_replay_arms_the_camera_delay` (gap 5); the queue witness
  re-pins the gap-6 dedupe (no inline grant).
* **Merger drops (this branch keeps its seams; the drops are the
  merge's):** g12 `main_dust_b.rs`'s three `teleport_effect_case*` fns +
  their 4 witnesses (sites 12-16 bone-arm/Moon-Lord stay); d54
  `world_dust_main.rs`'s `teleport_effect` + `hsl_to_rgb_bytes` +
  `portal_color` + `next_vector2_from_rectangle` + the 7 teleport
  witnesses + `hsl_and_portal_colors` (the snowing half → research/83);
  g61's `npc_spawn_visuals.rs` teleport family supersedes at ITS merge
  (this branch's `teleport_dust.rs` is the surviving copy — the merger
  keeps ONE; g61's poof/mimic/transform/alt-texture halves + their 8
  witnesses stay g61's). d54's latent-10 note retires with its body.
  The snowing keep-one (research/83 §-patch) is d130's execution on ITS
  branch — F1-F7 published as exact patch-hunks there.

Execution verification: code-only (no cargo/tests/rustfmt/graphify per
the lane directive). Every adapted hunk was diffed against its source
(`git show origin/task/W4-g61-npc-spawn-dust:…` at `a9628928`,
`origin/task/W4-g12-dust-main-sites-2` at `665cd1e8`,
`origin/task/W4-d54-dust-sites-world` at `c89aea9f`) and the C# re-read
(`Player.cs:37414-37496`, `NPC.cs:81035-81058`/`:90290-90334`,
`Main.cs:57465-57489`). The witnesses are WRITTEN NOT RUN — the
merge-verify wave executes them.

## 5. Final phase — the g61b co-lane cross-verify + the two seam folds

> **Dust-retire update (§7 below):** the local g61 tip advanced AGAIN to
> `b5ff1803d` (+4 over `5852697a`: seam 6 key-summon + docs + 2 borrow-fix
> rounds). The five-seam census below stands; §7 adds the seam-6 row and
> the alive/dead probe.

g61's branch ADVANCED past the `a9628928` the union-exec adapted:
`origin/task/W4-g61-npc-spawn-dust` is still `a96289284`, but the LOCAL
`task/W4-g61-npc-spawn-dust` tip is `5852697a6` (+5 wiring-seam commits:
`1eedb9a3` seam 1 fields/callback, `b4e40449` seam 2 bug-net poof,
`31baa315` seam 3 statue/lamp poofs, `c289d64f` seam 4 NPC.Teleport call
sites + fade driver, `5852697a` seam 5 Transform tail). The brief's
"env-driver retired per 116" resolves to g61's OWN keep-one note
(`1eedb9a3`: "supersedes g11 main_dust styles 0-5/7 at merge" — the
inverse direction of §3's keep-one; both branches agree the g61 BODY
survives, and this branch's `teleport_dust.rs` is the adapted copy).

**Spawn-dust dup census (g61b's five seams vs this branch):**

| g61b seam | site | this branch | verdict |
| --- | --- | --- | --- |
| 1 `1eedb9a3` | `Npc` teleport fields + ctor zero-init; `play_sound` carries the caller's stream | g11's OWN fields (`npc.rs:1256-1263`, `teleport_style_id` naming) + the dispatcher's RefCell stream | NO DUP — same fields semantically; merger keeps ONE field set (g11's names, §2) + ONE callback shape (the dispatcher's) |
| 2 `b4e40449` | bug-net catch poof (`bug_net.rs`, `NPC.cs:79883`/`:79912`, UpdatePlayers stream) | not ported here (g61's lane) | NO DUP — g61's poof family stays g61's (§4 merger-drops note); the catch seam never calls `TeleportEffect` (the frog's style-13 `NPC.Teleport` at `:79880` is the SEPARATE arm — when g61b's lane lands it must ride `main_dust::npc_teleport` on the caller's pass, not its own driver) |
| 3 `31baa315` | statue/lamp poofs (`StatueSpawn::Poof`, `tr-core/wiring.rs` + the app drain) | not ported here | NO DUP — poof ≠ teleport; the `StatueSpawn::Teleport` variant is disjoint from `Poof` |
| 4 `c289d64f` | statue style 14 + pad style 0 through g61's `npc_teleport_wiring` (immediate, `app.wiring.rand_mut()` stream); fade PRE-AI with `tfv = anim_time*60 % 216000` | pad style 0 already rides the g11 replay queue (`wiring.rs:345`, `UpdateWorldGen` key); statue style 14 was a BARE move (no grants) — **GAP FOUND + CLOSED this phase**; the fade was at the post-AI tail with tfv 0.0 — **ORDER DEVIATION FOUND + CLOSED this phase** | DUP RISK on the pad loop if both merges land both wirings — the merger keeps the g11 QUEUE replay (canonical §2) and drops g61b's `npc_teleport_wiring` helper with its env-level driver. The statue + fade folds landed HERE (below) so g61b's seam-4 semantics are already in the surviving shape |
| 5 `5852697a` | Transform tail (`wants_transform_visuals`, gore pool through `update_rng_dust`) | not ported here | NO DUP — transform ≠ teleport; g61's lane |

**Fold A — the fade PRE-AI re-slot (`world_sim/npcs.rs`).** The C#
`UpdateNPC` order (re-read `:90274-90423`): `UpdateAltTexture` `:90313`
→ `UpdateNPC_CastLights` `:90320` → **`UpdateNPC_TeleportVisuals`
`:90322`** → `UpdateNPC_CritterSounds` `:90323` → the look-for-home
decay `:90324-90327` → … → **`AI()` `:90423`**. The union-exec placed
the fade after the AI's draws (the port's per-slot tail) — a
stream-order deviation whenever a slot holds a live grant (the fade's
`Next(100)` gate + dust internals would land AFTER the AI's draws
instead of before). The final phase moves the drain into the slot
preamble (the `n.slot = i` block, ahead of the WoF presence mirror),
gates it on `teleport_time > 0.0` (the head gate draws nothing anyway —
the guard keeps the zero-grant common case borrow-free), and feeds
`time_for_visuals = (app.anim_time as f64 * 60.0) % 216000.0` captured
once per `update_npcs` call before the field borrows (g61b `c289d64f`'s
stand-in shape, the `frame/draw.rs:374` convention). The port's
critter-sound/look-for-home emissions stay at their recorded tail
positions (their own cited deviations) — only the fade moves, because
only the fade draws RNG that the AI's draws must follow. New witness
`w4_g11_the_fade_clears_style13_pre_ai_in_one_tick` (the style-13
instant clear lands on the first tick, the cleared grant is a head-gate
no-op). WRITTEN NOT RUN.

**Fold B — the statue style-14 latch (`world_sim/wiring.rs`).**
`Wiring.cs:2437`/`:2465` call `NPC.Teleport(pos, 14)` — the g11 arm
moved the rolled NPC directly, skipping the `:81052-81053` grants. The
final phase queues an `NpcTeleportReq { style: 14, key: UpdateWorldGen }`
through the SAME post-step replay drain as the pads: style 14 has no
`TeleportEffect` arm (zero dust — the wiring stream spends only the
roster roll), the move lands at the same `(x - w/2, y - h - 1)`, and
`teleport_time = 1.0` / `teleport_style_id = 14` latch for the fade
(which draws nothing for style 14 — the `_ => {}` arm — but the shared
−0.005 tail decays the grant, the C# shape). The existing
`dw_4_4_the_teleport_statue_moves_only_the_rolled_npc` witness gains the
latch assertions + the zero-dust census. WRITTEN NOT RUN.

**Audio helper note:** g61b's `Audio::play_teleport_cue` (`c289d64f`,
the `(2,n)`/`(19,0)` façade match) is the same resolution this branch's
dispatcher closure already carries inline — the merger may hoist it
into the named helper (g61b's shape) with zero behaviour change; no
dup risk (one call path either way).

## 6. Final phase — the main_dust.rs FILENAME-collision union-plan (g11 × g13, for d54f-131)

Both branches CREATE `crates/tr-app/src/main_dust.rs` (g13's research/68
§1 confirmed the collision; its merge-mechanics notes stand). Tips
re-read: g11 this branch (the union-exec dispatcher + the deprecated
snowing), g13 `origin/task/W4-g13-dust-main-sites-3` at `441f6120f`
(the `5ea44967` sweep + the `a234107a`/`441f6120` second-chain-lights
commits — those touch `dust_lights_b.rs`/`draw.rs`, NOT `main_dust.rs`;
the file is byte-identical to `5ea44967`'s). **Union = ONE file**, the
per-fn keep table:

| fn / item | g11 side | g13 side | KEEP at merge |
| --- | --- | --- | --- |
| module doc | the sites-1-8 + dispatcher + RNG-slot + retire-note prose | the sites-17-23 + draw-path RNG + wiring prose | UNION both prose blocks (the RNG sections agree: bare `Main.rand` → `rngs.global()` for the draw path; g11's teleport pass-key section is disjoint) |
| `snowing` + `SnowingEnv` + `MAX_SCREEN_W`/`SNOW_TILE_MAX` | DEPRECATED (research/83) | — | **DROP** (d130's `snow.rs` survives; the `scene_scan` block re-points per 83 F6) |
| `teleport_effect` (App dispatcher) + `CellRng` + `EffectRect` | the surviving seam | — | **KEEP g11** |
| `player_teleport`/`npc_teleport` + `PlayerTeleportReq`/`NpcTeleportReq` | the replays (+ the statue-14 queue fold) | — | **KEEP g11** |
| `to_rotation`/`math_sign`/`tentacle_tip_position`/`moon_lord_tentacle_tip_puff` | — | sites 17 + the tip maths | **KEEP g13** |
| `AlePlayerView`/`tavernkeep_ale_sparkle` | — | sites 18/19 | **KEEP g13** |
| `world_item_sparkle`/`light_color_bytes`/`item_sparkle_feed` | — | site 20 + the feed | **KEEP g13** |
| `liquid_surface_sparkle`/`lava_bubble_pair`/`retro_surface_rect`/`retro_liquid_dust_feed` | — | sites 21-23 + the retro feed | **KEEP g13** |
| `moon_lord_tentacle_feed`/`tavernkeep_ale_feed` | — | the draw-call feeds | **KEEP g13** (the Moon-Lord feed FUSES with g12's `moon_lord_core_dust` into ONE caller pass — g13 research/68's double-feed flag; both dead under `flag8` today) |
| `capture_snow_cull` | — | the DrawCapture cull (`1af79847`) | **KEEP g13** (RNG-free; reads live type-76 — d130's flakes are its input at the union, no code change) |
| `mod main_dust;` in `main.rs` | after `map_layers` (:63) | after `render` (:260) | ONE declaration (either spot; g11's is earlier — keep it, drop g13's) |
| imports | `crate::app::{App, RngKey}` + the pool trio | `crate::bow_launch::rotated_by`, `crate::npc::{Npc, NpcKind}`, `crate::npc_ai_style64::rect_distance` + the same pool trio | UNION both sets (the pool trio dedupes; `Npc` is already g11-side via the replays) |
| `tests` helpers | `rig(seed) -> App`, `live(&App)`, `stream_of`, `default_block`, `item_pitch`, `snow_rig`, `player_rig`, `union_rig`, `mirror_block` | `rig() -> (DustPool, World, SpawnCtx)`, `live(&DustPool)`, `mirror_default`, `mirror_type6`, `mirror_type35`, `ale_player`, `cull_rig` | **RENAME g13's pair** (`rig`→`pool_rig`, `live`→`pool_live` — g13 research/68's own note); the rest are name-disjoint. g11's four snow witnesses DROP with the body (migrate per 83 F7); g13's 20 witnesses KEEP verbatim |
| `#![allow(dead_code)]` | module-level (the staged entry points) | item-level on `capture_snow_cull` only | KEEP the module-level allow until the capture seam wires; g13's item-level allow becomes redundant (harmless) |

Witness census at the union: g11's 26 surviving `main_dust.rs` witnesses
(30 minus the 4 snow) + the pad/statue wiring witnesses + g13's 20 +
`teleport_dust.rs`'s 8 — all WRITTEN NOT RUN, the merge-verify wave
executes. d54's `world_dust_main.rs` keeps its OWN filename (no
collision with the union file); its teleport + snowing halves drop per
§3/83, the world-site bodies stay d54's.

Final-phase verification: code-only (no cargo/tests/rustfmt/graphify).
The fade order re-read `NPC.cs:90274-90423` directly; the statue call
sites re-read `Wiring.cs:2437`/`:2465`; the g13 file diffed
`git diff 5ea44967 441f6120 -- crates/tr-app/src/main_dust.rs` (empty);
the g61b seams read as full diffs (`git show 1eedb9a3/b4e40449/
31baa315/c289d64f/5852697a`). The two new/extended witnesses are
WRITTEN NOT RUN.

## 7. Dust-retire phase — the g61b probe (alive) + the ONE-paragraph merger instruction

### 7.1 g61b worktree probe (read-only) — ALIVE, local-ahead, push owed

`TR-W4-g61` probed at 2026-11-21 09:05 local:

* **Tip `b5ff1803d`** (committed 08:32:20), **+9 over origin `a96289284`**
  (ancestry verified: `git merge-base --is-ancestor a96289284 b5ff1803d`
  exit 0). The nine: `1eedb9a3` seam 1, `b4e40449` seam 2, `31baa315`
  seam 3, `c289d64f` seam 4, `5852697a` seam 5, `8f545007` seam 6
  (key-summon), `7bed9a1c` wiring docs, `4402e4be` + `b5ff1803` the two
  NLL borrow-fix rounds.
* **Status: tracked-clean**, only two untracked scratch files
  (`spawn_census_raw.txt` 2026-11-20 21:09, `spawn_regions_dump.txt`
  21:13 — census dumps, junk-class per the lifecycle rules).
* **ALIVE**: `world_sim/wiring.rs` mtime 08:30:34, `chest.rs` 08:25:30,
  `bug_net.rs` 08:22:55 — writes within ~35 min of the probe, matching
  the borrow-fix commit times. NOT dead-dirty; no rescue needed beyond
  the ordinary **push of the 9 local commits** (the g61 session owns
  that push; this lane must NOT push another agent's branch).

**Seam-6 dup census (the new commit vs this branch):**
`8f545007` adds `Player.last_chest` + `chest::chest_change_events`
(`Player.cs:28760-28775` at `:25384`, close edge) wired at the
player-tick tail, and `chest::big_mimic_summon_check`
(`NPC.cs:77517-77605`). **NO DUP with the teleport dispatcher** — the
key-summon is a `NewNPC` + `BigMimicSpawnSmoke` spawn family (g61's
`npc_spawn_visuals.rs` lane), never `TeleportEffect`; the player-tick
tail region hosts BOTH the g11 Shellphone-11 drain and g61b's
`chest_change_events` — they SHARE the UpdatePlayers stream, so the
merger keeps both at their cited slots (`chest_change_events` at the
`Player.Update` `:25384` position; the replay drain at §2's canonical
tail — the replay-deferral stream note of §2 already covers the
relative order, no new deviation). The
borrow-fix rounds (`4402e4be`/`b5ff1803`) touch g61-side call shapes
only (`catch_npc` anchor copy, pad gate-probe copy) — the pad-loop
merge note from §5 seam 4 stands: keep the g11 QUEUE replay, drop
g61b's `npc_teleport_wiring` helper (the borrow fix dies with it).

### 7.2 ONE-paragraph merger instruction (dust-master 131 cross-ref)

**For the d54f-131 dust-master merge (research/131 §2 Main lane + §3
order):** resolve `main_dust.rs` to ONE file per §6's keep table — keep
g11's teleport SEAMS dispatcher (`teleport_effect` + `CellRng` +
`player_teleport`/`npc_teleport` replays + the statue-14 queue fold)
calling the ONE g61 widest-11 body in `teleport_dust.rs` (adapted from
`a9628928`; styles 0-13 incl. pylon 9, the six §4 gaps closed), keep
g13's six site-17-23 bodies + feeds + `capture_snow_cull` verbatim
(its `rig`/`live` helpers renamed `pool_rig`/`pool_live`), keep g12's
`main_dust_b.rs` bone-arm pair + `moon_lord_core_dust` (its three
`teleport_effect_case*` fns + 4 witnesses DROP — the g61 body covers
sites 9-11), fuse g12's Moon-Lord feed with g13's tentacle feed into ONE
caller pass, dedupe the `mod main_dust;` declaration to g11's spot; DROP
g11's deprecated `snowing`/`SnowingEnv`/consts + the four snow witnesses
+ the `scene_scan` SnowingEnv block for d130's `snow.rs` — **read d130's
LOCAL tip `16a792d56`, not origin `560a02ea`**: d130e applied F1+F5+F7 at
`099df89e2` (global `UnifiedRandom` stream, splitmix retired, int loop,
g11 mirrors ported) + the cull-kill at `16a792d56`, so NO patch-hunks or
witness migration are owed (research/83 §-receipt) — and flag both
weather/spawn rescue pushes: d130's 2 fold commits and g61's 9 wiring
commits (`b5ff1803d` tip, ALIVE) are local-only ahead of their origins;
keep g61's `npc_spawn_visuals.rs` whole (poof/mimic/transform/key-summon
— disjoint from the dispatcher, §5/§7.1 censuses), keep ONE `Npc`
teleport field set (g11's names) + ONE sound-callback shape (the
dispatcher's; g61b's `play_teleport_cue` may be hoisted as the named
helper), keep the g11 pad QUEUE replay and DROP g61b's
`npc_teleport_wiring` env-driver helper (its borrow fixes die with it),
keep d54's `world_dust_main.rs` filename but DROP its teleport + snowing
halves; the union witness census is g11's 26 surviving `main_dust.rs`
witnesses + the pad/statue wiring witnesses + g13's 20 +
`teleport_dust.rs`'s 8 + d130's re-pinned snow witnesses — ALL WRITTEN
NOT RUN, the merge-verify wave executes the catalog command.

Verification: code-only. Probe = `git -C TR-W4-g61 status --porcelain`
+ `git log` + direct-path file mtimes (no junction recursion); seam 6
read as `git show 8f545007` (player.rs/world_sim/player.rs hunks quoted
above); ancestry + ahead-counts from `git log origin..local --oneline`.

## 8. SEAL (2026-11-21) — the four seal receipts + the teleport-family bible

**This report is SEALED as research/116.** The lane's complete hand-off
to the merge wave is §7.2's ONE-paragraph instruction; the receipts
below are the seal-phase re-reads, every one from a pushed tip or a
direct-path `git show` (no junction recursion, no checkouts of other
lanes). Code-only throughout: every witness cited is WRITTEN NOT RUN.

### 8.1 Receipt 1 — the teleport-union git-log census (this branch)

`git log` census of the union chain at HEAD `2e978b279` (origin
`task/W4-g11-dust-main-sites-1` == HEAD, ancestry exit 0; `9a9ac1fd0`
and `98283df08` both ancestors of HEAD, exit 0):

| commit | phase | what it seals |
| --- | --- | --- |
| `56afc0c01` | union 1/3 | `teleport_dust.rs` — g61's widest-11 body adapted per-hunk; the `DustRng` pool generalisation; `App.water_style` (gap 4); 8 witnesses |
| `8540a8ad5` | union 2/3 | the App dispatcher + the **6 gap closures**; `update_npc_teleport_visuals` wired; the g12 boundary witness INVERTS; 4 gap + 4 g12 + 8 d54 union witnesses |
| `9a9ac1fd0` | union 3/3 | **EXECUTED** docs (this report §4); research/83 F1-F7 exact patch-hunks published; INDEX/ANALYSIS/PORT_MAP/anchors |
| `cc8c767b0` | final code | the seam-4 folds — fade PRE-AI (`NPC.cs:90322` before AI `:90423`) + the statue style-14 queue latch; `snowing` DEPRECATED; 2 witnesses |
| `98283df08` | final docs | **FINAL** §5/§6 — the g61b co-lane census (no spawn-dust dup; pad-loop keep-g11-queue/drop-g61b-helper), the `main_dust.rs` g11×g13 ONE-file per-fn keep table, the :17238 pre-swap scope re-walk (the g11 global-stream claim STANDS) |
| `c2e445a38`…`2e978b279` | dust-retire 1-4/4 | the d130e F1-applied receipt (research/83 §-receipt), the Main.cs FINAL sweep (research/65 — ZERO unclaimed sites), the g61b probe + seam-6 NO-DUP + §7.2's merger instruction |

The six gaps re-verified IN CODE at HEAD (direct-path greps):
gap 1 — the Queen-Slime hook queues `teleport_style(...,10)`
(`projectile.rs:17069`), the Shellphone/no-space legs queue style 11
(`player.rs:9483`/`:9600`/`:19684`); gap 2 — `other_position` on the req
(`main_dust.rs:330`, fed `:375`); gap 3 — `extra_info2` on the req
(`:416`/`:450`/`:526-552`, the style-9 pylon latch witness
`w4_g11_style_9_carries_the_pylon_style` `:1177`); gap 4 — the stored
`app.water_style` read at the dispatcher (`:336`, the `DrawWaters`
write); gap 5 — the style-8 double cue + the replay-owned camera
transition (the pad's inline arm retired); gap 6 — the single
C#-position grant write (`:495`, `:37490-37491`). The keep-one seam
census: `main_dust.rs` 30 `#[test]` (26 surviving after the 4-snow
drop) + `teleport_dust.rs` 8 — all WRITTEN NOT RUN. **Seam-6
key-summon: NO-DUP** (§7.1 stands — `Player.last_chest` +
`chest_change_events` at the `Player.cs:25384` slot +
`big_mimic_summon_check` `NPC.cs:77517-77605` are a NewNPC/
BigMimicSpawnSmoke spawn family, never `TeleportEffect`; the shared
UpdatePlayers tail keeps both at their cited slots).

### 8.2 Receipt 2 — the g61d spawn-seal mutual (`6e922fc6`, pushed)

`origin/task/W4-g61-npc-spawn-dust` == `6e922fc60` (re-read after
`git fetch origin --prune`; ancestry exit 0; `b5ff1803d` is an
ancestor of the seal, exit 0). **The §7.1 push-owed flag is RETIRED** —
the +9 wiring commits (`1eedb9a33`…`b5ff1803d`) are pushed; the seal
adds +4 (`e6fcb6432` KEEP-ONE-STALE fix, `df15960ac` crimson LIVE +
roster census, `fd406ecba` FINAL, `6e922fc60` SEAL + the 237→238
renumber). The g61d seal's receipt (1) names THIS lane's tip
`2e978b279` — the keep-one is MUTUAL, g11-canonical both ends.

The 7 DEPRECATED banners re-read in-place at `6e922fc60` (direct-path
`git show` windows) — all point g11-canonical, the correct direction:

| site | banner target |
| --- | --- |
| `npc_spawn_visuals.rs:470` | `teleport_effect` — FOLD-SOURCE, g11 `teleport_dust.rs` survives (`56afc0c0`) |
| `npc_spawn_visuals.rs:860` | `portal_color` — FOLD-SOURCE, g11's copy survives (same `ui::color::hsl_to_rgb`) |
| `npc_spawn_visuals.rs:882` | `pylon_spawn_in_world_dust` — FOLD-SOURCE, g11's style-9 body survives |
| `npc_spawn_visuals.rs:938` | the env-level `npc_teleport` driver — RETIRED, g11's App-level replay supersedes (gap 2 `other_position`, the g11 `Npc` fields) |
| `npc_spawn_visuals.rs:1011` | the `update_npc_teleport_visuals` triple — FOLD-SOURCE, g11's `&mut Npc` re-sign survives (`56afc0c0` + `cc8c767b`, PRE-AI slot kept) |
| `world_sim/wiring.rs:557` | `npc_teleport_wiring` — the §5 DUP RISK: merger keeps the g11 QUEUE replay, drops the immediate helper (its borrow fixes die with it) |
| `world_sim/npcs.rs:4408` | the copy-in/out fade drain — DEPRECATED SHAPE, g11's `&mut Npc` fade at the SAME pre-AI slot survives |

**Stays g61** (the spawn-family bible, g61 research/238 §4):
`poof_of_smoke` (`npc_spawn_visuals.rs:184`, `Utils.cs:1078-1093`),
`big_mimic_spawn_smoke` (`:236`), `transform_visuals` (`:289` — the
c10 census-dup contract keeps g61's FULL pool-writing body), the
alt-texture transitions, and `Audio::play_teleport_cue`
(`audio.rs:1572`) — the named cue helper survives; the g11 dispatcher
closure may hoist it (the `npc_spawn_visuals.rs:81` note), ONE
sound-callback shape at the merge. None of these ever calls
`TeleportEffect` — the §5/§7.1 NO-DUP censuses hold.

### 8.3 Receipt 3 — the d54 bible: `main_dust.rs` THREE-WAY, per §6

d54 tip `38bfd1d22` (registry-150 published), bible at research/131
§8 CONSOLIDATED FINAL (`ad0d95d4b`). Its §8.3 merger dust-checklist
item 4 reads: **"main_dust.rs THREE-WAY resolve (g11 `98283df08` ×
g13 `4444e1011` × d54) — ONE file per research/116 §6's per-fn
table"** — KEEP g11 `teleport_effect` dispatcher + `CellRng`/
`EffectRect` + `player_teleport`/`npc_teleport` + the Req types.
The §6 per-fn keep table (lines above) is the authority; the §8.1
fold table marks the g11 teleport union + the seam-4 folds
**VERIFIED-EXECUTED** and the g11×g13 filename union **OWED
(merger) — plan FINAL**; d54 keeps its own `world_dust_main.rs`
filename, its teleport + snowing halves DROP (§3/§7.2). The g13 tip
in the bible (`4444e1011`) is past this report's §6 read
(`441f6120f`) — the later commits are the Main-COMPLETE sweep; the
§6 byte-identity check covers `main_dust.rs` itself. No g11 edit
owed: the bible cites this report, not a copy.

### 8.4 Receipt 4 — research/116 SEALED + the teleport-family bible

The teleport-family declaration (this lane's closing bible):

* **ONE body:** `teleport_dust.rs::teleport_effect` (g61 `a9628928`
  widest-11 adapted — `Main.cs:13233-13464`, styles 0-5/7/9/10/11/13,
  `TeleportDustEnv`, canonical `hsl_to_rgb`, sounds at the C#
  position, the style-10 clone guard, the pylon 9
  `TeleportPylonsSystem.cs:327-393`) + `update_npc_teleport_visuals`
  (`NPC.cs:92194-92241`, g11 `Npc` fields, PRE-AI `:90322`).
* **ONE seam set:** `main_dust.rs` — the App dispatcher +
  `player_teleport`/`npc_teleport` Entry-OLD→Exit-NEW replays +
  `PlayerTeleportReq`/`NpcTeleportReq` (the statue-14 queue fold),
  drained on the player tick / wiring pads / projectile-pass tail;
  ONE `Npc` teleport field set (g11 `teleport_style_id` naming), ONE
  sound-callback shape (`play_teleport_cue` stays g61's named helper).
* **Fold-sources DROP at the merge:** the g61 teleport family (the 7
  banners, §8.2), g12's `teleport_effect_case10/11/13` + 4 witnesses,
  d54's teleport half + 7 witnesses + `hsl_and_portal_colors`; g11's
  deprecated `snowing` half drops to d130 (research/83 — d130e
  applied F1+F5+F7 locally at `16a792d56`, no hunks owed).
* **Stays disjoint:** the g61 spawn family (§8.2), g13's sites 17-23
  + `capture_snow_cull` (§6), g12's bone-arm/Moon-Lord halves, d54's
  world-site bodies.
* **Witness census:** g11's 26 surviving `main_dust.rs` + the
  pad/statue wiring witnesses + `teleport_dust.rs`'s 8 + g13's 20 +
  d130's re-pinned snow witnesses — **ALL WRITTEN NOT RUN**; the
  merge-verify wave executes the catalog command. Every witness has
  never been compiled — expect first-compile mechanical fixes (the
  code-only convention, g61 research/238 §6 item 5).

Seal verification: code-only (no cargo/tests/rustfmt/graphify). Tips
re-read after `git fetch origin --prune`: origin g11 == `2e978b279`
(ancestry exit 0), origin g61 == `6e922fc60` (ancestry exit 0,
`b5ff1803d` ancestor exit 0, +4 over it); the union-chain census from
`git log`; the 7 banners read as ≤10-line direct-path `git show`
windows at `6e922fc60`; the gap markers grepped at HEAD
(`main_dust.rs`/`player.rs`/`projectile.rs`); the d54 bible read at
`38bfd1d22` (`git show`, §8.1/§8.3 windows); the g61 bible read at
`6e922fc60` (research/238 §1/§4/§6). No further g11 commits are
planned on this lane — §7.2 is the merger's complete instruction.
