//! W4-g13 — the Main.cs `Dust.NewDust*` sites' THIRD batch (sites 17-23 of
//! the 23-site census, `docs/research/64-dust-main-sites-claims.md` on
//! `task/W4-g12-dust-main-sites-2`; g11 owns 1-8, g12 owns 9-16).
//!
//! Batch ledger (C# `Dust.NewDust*` line = site id):
//! - 24756 — the Moon Lord core's tentacle-tip puff (`DrawNPCDirect`
//!   `:22513`, the type-398 arm `:24701-24764`). DEAD ARM: the enclosing
//!   `if (flag8)` reads `bool flag8 = false` (`:24703`) which the arm never
//!   sets — the four-site cluster `:24736/24748/24752/24756` never fires.
//!   g12 owns the first three; this batch ports the fourth verbatim behind
//!   the same dead flag, with the tip-position maths witnessable on its own.
//! - 27381/27384 — the Tavernkeep's ale-toss sparkle pair (`DrawNPCExtras`
//!   `:26912`, the type-550 `ai[0] == 18` arm `:27353-27397`): 30 iterations
//!   of two `NewDustDirect` type-4 gold specks when the talked-to player
//!   holds Ale (353) facing the NPC within 46 px and `localAI[3]` edges to 1.
//! - 38664 — the world-item coin/heart/mana sparkle (`DrawItem` `:38602`,
//!   the `FocusHelper.AllowWorldItemsToEmitEffects` gate `:38637-38666`).
//! - 57766 — the retro liquid-surface sparkle (`oldDrawWater` `:57553`,
//!   the lit-surface gate `:57747-57769`).
//! - 57789/57793 — the retro lava-bubble pair (`oldDrawWater`, the
//!   `FocusHelper.AllowMiscDustEffects && Dust.lavaBubbles < 200` gate
//!   `:57785-57803`).
//!
//! Sweep addition (`docs/research/202-dust-main-residual-sweep.md`): the
//! `DrawCapture` snow cull (`Main.cs:56735-56744`) — the one unclaimed
//! Main.cs dust window outside the 23 spawn sites; `capture_snow_cull`,
//! staged behind the S1-G-13 capture-render seam.
//!
//! ## RNG — one slot per C# pass (S1-A-10)
//!
//! Every site here executes on the DRAW path (`DrawNPCDirect` /
//! `DrawNPCExtras` / `DrawItem` / `oldDrawWater`), outside every
//! `Main.SwapRandom` block (`Main.cs:17732-17809` wraps only the update
//! stages) — so in the C# they all roll the BARE `Main.rand`. The port's
//! bare-stream slot is `NamedRngs::global()` (`app/rng.rs`, the documented
//! world-seed deviation): the wiring wave passes `app.rngs.global()` as
//! `rng`, exactly like the minecart dust tick's player-less arm
//! (`app/frame/world.rs`'s `dust_phase`). No new stream is created; the 17
//! named simulation streams are untouched (the draw path never advances
//! them — the hard invariant). The emitters stay `&mut UnifiedRandom`-generic
//! so the witnesses can pin a seed and mirror the draw sequence.
//!
//! Per site, in C# statement order: the explicit `Main.rand` gate/argument
//! draws FIRST (C# evaluates the `if` condition before the call), then the
//! `NewDust` internals roll inside `DustPool::new_dust` (`Dust.cs:75-256`,
//! d50's verbatim pool), then the post-spawn slot writes (which read
//! `Main.dust[6000]` when the pool refuses — the sentinel slot takes the
//! writes exactly like the C#).
//!
//! ## Contract + dependencies
//!
//! Pool: `tr_core::particles::DustPool` (d50, `origin/task/W4-d50-dust-type-table`
//! merged to main) — guards, cull, pressure bands, default block, ChildSafety
//! and the per-type tweaks all ride `new_dust`/`new_dust_direct` verbatim.
//! `Dust.lavaBubbles` (`Dust.cs:14`) is read here as `pool.lava_bubbles`;
//! its per-tick reset (`Dust.cs:427`) and increment (`:475`, every live
//! type-35 slot) now ride `DustPool::update_dust` (the wiring wave landed
//! the increment — it was the d50 phase-2 lane's remainder).
//!
//! ## Wiring (the draw-call wave, landed)
//!
//! The feeds below run from `app/frame/draw.rs::draw_region`, OUTSIDE the
//! G6-cached world pass (the rain/sand/gore dust precedent — the spawns
//! mutate the pool, so they cannot live inside the cached range), in the
//! C# relative order: the retro bg pass → the Moon Lord core arm → the
//! Tavernkeep ale arm → the item sparkle → the retro fg pass. Each rolls
//! the bare `Main.rand` slot (`rngs.global()`). The retro sites gate on
//! `LightMode::Retro | Trippy` (`!Lighting.NotRetro`, `Main.cs:57519`); the
//! port's liquid SPRITE draw stays on the `LiquidRenderer` record path
//! (`render/liquids.rs`) — the retro feed walks the same tile range and
//! runs ONLY the dust gates. `FocusHelper.Allow*` is `!DrawnThisUpdate`;
//! the mark sets AFTER `draw_region` (`Main.cs:62478`, the end-of-Draw
//! position), so it reads clear through the draw on a stepping frame.
//! CODE-ONLY MODE: no compiler invoked; the witnesses below are WRITTEN,
//! NOT RUN.
//! W4-g11 — the Main.cs `Dust.NewDust*` sites, batch 1 of 3 (the line-order
//! thirds split of `docs/research/64-dust-main-sites-claims.md`, W4-g12's
//! claim doc: g11 owns sites 1-8, lines 13088-13358; g12 sites 9-16; g13
//! sites 17-23). This batch: `Main.snowing()` (`Main.cs:13032-13116`, the
//! one type-76 site `:13088`) and the `Main.TeleportEffect` SEAMS
//! (`:13233-13464`).
//!
//! UNION-EXEC (research/116 keep-one, this round): the surviving
//! `TeleportEffect` BODY is g61's widest-11 (`teleport_dust.rs`, adapted
//! per-hunk from `a9628928`); this module's `teleport_effect` is the
//! **App-level dispatcher** — it builds the `TeleportDustEnv` from the
//! disjoint App fields + the caller's pass stream and resolves the Audio
//! façade (`(2, n)` → `play_npc_item_cue`, `(19, 0)` →
//! `play_splash_sound`), then calls the body for ALL styles (0-5/7/9/10/
//! 11/13). The g11 seven arm bodies + `nd`/`nd_full`/`nd_direct`/
//! `item_cue`/`portal_color` are RETIRED (the dispatcher supersedes
//! them); g61's env-level `npc_teleport` driver is superseded by the
//! App-level replay below (carrying `other_position`, gap 2). The six
//! research/116 wiring gaps are closed: styles 10/11/13 reachable-caller
//! arms (the Queen-Slime hook + the Shellphone/no-space path now fire
//! through the queue), `otherPosition` on both legs, style 9's pylon
//! `extraInfo2`, the stored `Main.waterStyle` (gap 4, `App.water_style`),
//! the `Player.Teleport` style-8 double cue + the player-tick camera
//! transition (style 1 → delay 10), and the double-grant dedupe (the
//! replay owns the C#-position grant write; the inline path keeps only
//! the move).
//!
//! Contract (the S1-E-1 pool, d50's `DustPool`): every draw rides the
//! caller's stream in C# statement order — the explicit `Main.rand`
//! argument draws FIRST (C# evaluates arguments before the call), then the
//! `NewDust` internals roll inside `DustPool::new_dust` (`Dust.cs:75-256`),
//! then the post-spawn writes (which read `Main.dust[6000]` when the pool
//! refuses — the sentinel slot exists and takes the writes exactly like
//! the C#). One draw = one stream step; no new streams (S1-A-10).
//!
//! RNG slots (one per C# pass): `snowing` runs at `Main.cs:17238`, BEFORE
//! the `DoUpdateInWorld` swap (`:17732`) — the bare `Main.rand`, the port's
//! `NamedRngs::global()` stand-in. **d130 adjudication (research/83
//! §-adjudication): the pre-swap claim STANDS** — `DoUpdate` (`:16828`)
//! runs `:17238` with no enclosing `SwapRandom` (`Update` `:16774` →
//! `DoUpdate` `:16788`, the first swap is `:17732` inside the LATER
//! `DoUpdateInWorld`), so the C# draws land on the bare `Main.rand` that
//! `RandomSwap.Swap` (`:3204-3215`) restores after every pass. **d130e
//! retire receipt (research/83 §-receipt): the F1 fold is APPLIED on
//! d130's branch — `099df89e2` (local tip `16a792d56`) retired the `Snow`
//! splitmix (struct + `App.snow` field + both init sites), `snowing` is
//! a free fn on `&mut UnifiedRandom` fed `app.rngs.global()` at the
//! `scene_scan` `:17238` slot, the pool draws ride `DustRng for
//! UnifiedRandom`, and F5's int loop landed with it. The earlier
//! REFUTED-AS-APPLIED flag (`560a02ea` splitmix-kept) is RETIRED.**
//! `TeleportEffect` is called from
//! `Player.Teleport` (`Player.cs:37444`/`:37489`, the `UpdatePlayers`
//! swap), `NPC.Teleport` (`NPC.cs:81044`/`:81051`, the caller's NPC pass)
//! and `Wiring.Teleport` (`Wiring.cs:3154`/`:3170`, the `UpdateWorldGen`
//! swap) — the caller passes its pass's stream key.
//!
//! Wiring status (W4-g11 wiring round — BOTH seams WIRED): the snow site —
//! `world_sim::scene_scan::update_scene` calls `snowing` at the `:17238`
//! slot (before `Sandstorm.EmitDust` `:17247`, on `NamedRngs::global()`),
//! and the `rain.rs` `Snowflake` vector stand-in is retired (keep-one).
//! **RETIRE-NOTE (research/83 keep-one, d130-canonical): this module's
//! `snowing` + `SnowingEnv` + `MAX_SCREEN_W`/`SNOW_TILE_MAX` are DEPRECATED
//! — the merger drops them for d130's `snow.rs` (wired + the real
//! `DustPool.snow_dust` counter + the type-76 AI/tails/draw). d130e
//! applied ALL the g11 folds: F2/F3/census at `85250712`, F1+F5+F7 at
//! `099df89e2` (global stream, int loop, the g11 mirrors ported), the
//! cull-kill at `16a792d56` (`snow::draw` shares the splash cull rect).
//! This branch keeps the body compiling until the drop; the four snow
//! witnesses below are ALREADY migrated on d130 (§-patch F7 applied —
//! the merger just drops these copies).**
//! The type-76 physics arm (`snowDust++`, the scale lerp — `Dust.cs:1745-1763`)
//! is d50's `dust_physics.rs` lane; until it lands the flakes ride the
//! generic `UpdateDust` move/kills, and the pool's generic draw pass is the
//! draw lane's. The `TeleportEffect` styles — `Player.Teleport`
//! (`player.rs::teleport_style_with_extra` queues the Entry/Exit pair + the
//! grants; `player_teleport` below replays Entry-at-OLD, move, Exit-at-NEW)
//! drained by the player tick (`world_sim/player.rs`, UpdatePlayers) and the
//! wiring pads (`world_sim/wiring.rs`, UpdateWorldGen); `NPC.Teleport`
//! (`npc_teleport` below) drained by the wiring pads in pool order. The
//! per-tick `Player.UpdateTeleportVisuals` decay stays the W4-e1 dust
//! lane's; the NPC per-tick fade (`NPC.UpdateNPC_TeleportVisuals`) is
//! WIRED PRE-AI in the `UpdateNPC` preamble (`world_sim/npcs.rs`,
//! `NPC.cs:90322` — before `AI()` `:90423`; the g61b seam-4 position
//! fold, research/116 §5), fed `Main.timeForVisualEffects` as the
//! `anim_time*60 % 216000` stand-in (g61b's shape).
//! `recall_teleport` runs its own recall dust, NOT `TeleportEffect` (the
//! mirror/recall cluster calls `Spawn`, not `Teleport`). The witnesses below
//! pin the ports + the wiring — the union witness set (research/116 §3:
//! g11's 7 style + wiring witnesses re-pinned to the dispatcher, g61's 9,
//! g12's 4, d54's 8). WRITTEN NOT RUN (code-only mode).

// The entry points are staged (see the wiring note above) — the witnesses
// exercise them under `cfg(test)`; the lint stays quiet until the wiring.
#![allow(dead_code)]

use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};

use crate::bow_launch::rotated_by;
use crate::npc::{Npc, NpcKind};
use crate::npc_ai_style64::rect_distance;

/// `Utils.ToRotation` (`Utils.cs:1655-1658`) — `Atan2` in double, cast to
/// float (the `npc_ai_empress.rs` precedent).
fn to_rotation(v: Vec2) -> f32 {
    (v.y as f64).atan2(v.x as f64) as f32
}

/// `Math.Sign(float)` — `-1`/`0`/`1` (the zero arm matters: the C# compares
/// it against `player.direction`, which is never 0, so an exactly-centred
/// player fails the ale gate).
fn math_sign(v: f32) -> i32 {
    if v > 0.0 {
        1
    } else if v < 0.0 {
        -1
    } else {
        0
    }
}

// ------------------------------------------------------ site 17 (24756) --

/// The tentacle-tip spawn position of the Moon Lord core's type-398 arm
/// (`Main.cs:24732-24756`): `vector32 + (0, 340).RotatedBy(vector33.ToRotation()
/// - num139 - PI/2)` where `vector32 = core.Center + (220, -60) * side`,
/// `vector33 = (hand.Center + (0, 76) - vector32) * 0.5` and
/// `num139 = Acos(|vector33| / 340) * (0 - side.X)`. `side_x` is the arm's
/// `vector31.X`: `-1` for the `ai[2] == 0` hand (`flag9`), `+1` for the
/// other. The float maths is the C#'s verbatim (the `Acos` NaN past unit
/// length is kept — no clamp); `RotatedBy` takes the float angle widened to
/// double (`Utils.cs:1675-1684`).
pub(crate) fn tentacle_tip_position(core_center: Vec2, hand_center: Vec2, side_x: f32) -> Vec2 {
    let vector31 = Vec2::new(side_x, 1.0);
    // `:24732` — the shoulder anchor.
    let vector32 = core_center + Vec2::new(220.0, -60.0) * vector31;
    // `:24733` — the half-length link towards the hand's `(0, 76)` offset.
    let vector33 = (hand_center + Vec2::new(0.0, 76.0) - vector32) * 0.5;
    // `:24738` — the acos bend, sign-flipped per side. `Math.Acos` runs in
    // double and casts to float before the multiply (the C#'s `(float)`).
    let num139 = ((vector33.length() / 340.0) as f64).acos() as f32 * (0.0 - vector31.x);
    // `:24756` — the tip: 340 px down the link's draw rotation.
    let angle = to_rotation(vector33) - num139 - std::f32::consts::PI / 2.0;
    vector32 + rotated_by(Vec2::new(0.0, 340.0), angle as f64)
}

/// Site 24756 — the fourth `noGravity` type-6 puff of the core's tentacle
/// cluster (`Main.cs:24754-24757`). `flag8` is the arm's dead latch
/// (`bool flag8 = false`, `:24703`, never set true anywhere in the arm):
/// the wiring wave passes the arm's flag verbatim, so this never fires —
/// the emitter exists so the cluster's fourth site is ported whole (the
/// g12/g13 boundary splits the cluster by line order, each batch citing
/// the shared arm). When enabled: one `NewDust(pos, 0, 0, 6)` (defaults:
/// speeds 0, alpha 0, `default(Color)`, scale 1) then `noGravity = true`.
/// The type-6 tweak (`Dust.cs:216-221`: `velocity.Y = Next(-10,6)*0.1`,
/// `velocity.X *= 0.3`, `scale *= 0.7`) rides the pool internals.
pub(crate) fn moon_lord_tentacle_tip_puff(
    flag8: bool,
    pos: Vec2,
    rng: &mut UnifiedRandom,
    pool: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    if !flag8 {
        return;
    }
    let idx = pool.new_dust(rng, ctx, world, pos, 0, 0, 6, 0.0, 0.0, 0, Color::default(), 1.0);
    pool.slot_mut(idx).no_gravity = true; // `:24756`
}

// -------------------------------------------------- sites 18/19 (27381/4) --

/// The talked-to player's view of the ale gate (`Main.cs:27368-27370`):
/// `HeldItem.type`, `direction`, the hitbox and the centre.
pub(crate) struct AlePlayerView {
    pub held_item: i32,
    pub direction: i32,
    /// `player.Hitbox` — `(x, y, width, height)` in world px.
    pub hitbox: (i32, i32, i32, i32),
    pub center: Vec2,
}

/// Sites 27381/27384 — the Tavernkeep's ale-toss sparkle
/// (`Main.cs:27353-27392`, the `n.type == 550 && n.ai[0] == 18f` arm of
/// `DrawNPCExtras`, `!beforeDraw` only — the caller owns that gate).
/// Returns the new `localAI[3]`.
///
/// The machine: `num18 = (int)ai[2]` must sit in `[0, 255)`; then
/// `flag = HeldItem.type == 353 && direction == Math.Sign(n.Center.X -
/// player.Center.X)` and `num19 = player.Hitbox.Distance(n.Center)` are
/// BOTH computed (the C#'s `&` is non-short-circuiting; neither rolls);
/// `(num19 < 46f) & flag` sets `localAI[3] = 1` and bursts only on the
/// edge (`localAI[3] != num20`, the old value); the failed gate edges
/// `1 -> 2`. The burst: `vector7 = n.Center + (spriteDirection*30, -6)`,
/// the 10x10 box centred on it (`vector7 - vector8/2`), 30 iterations of
/// - `NewDustDirect(box, 10, 10, 4, 0, 0, 50, Color(245,200,30,155), 0.7)`
///   then `noGravity = true`, `velocity *= 1f` (the C#'s no-op, kept);
/// - `NewDustDirect(same box, ..., 0.6)` then `velocity *= 2f`.
///
/// RNG: no explicit argument draws (every argument is a constant) — each
/// `NewDustDirect` spends the pool's six default draws (type 4 takes no
/// per-type tweak and is not ChildSafety-dangerous): 12 per iteration,
/// 360 per burst, on the caller's bare-`Main.rand` slot.
#[allow(clippy::too_many_arguments)]
pub(crate) fn tavernkeep_ale_sparkle(
    npc_center: Vec2,
    sprite_direction: i32,
    player_idx: i32,
    local_ai3: f32,
    player: Option<AlePlayerView>,
    rng: &mut UnifiedRandom,
    pool: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) -> f32 {
    // `:27365-27366` — the player-index window.
    let Some(player) = player.filter(|_| (0..255).contains(&player_idx)) else {
        return local_ai3;
    };
    // `:27369-27371` — both reads run before the `&` gate (no draws).
    let flag = player.held_item == 353
        && player.direction == math_sign(npc_center.x - player.center.x);
    let (hx, hy, hw, hh) = player.hitbox;
    let num19 = rect_distance(hx, hy, hw, hh, npc_center.x, npc_center.y);
    let num20 = local_ai3;
    // The C#'s `&` is non-short-circuiting, but both operands are already
    // evaluated above (neither rolls), so `&&` here is value-identical.
    if (num19 < 46.0) && flag {
        // `:27374-27386` — the latch write, then the edge-gated burst.
        let local_ai3 = 1.0;
        if local_ai3 != num20 {
            let vector7 = npc_center + Vec2::new(sprite_direction as f32 * 30.0, -6.0);
            let pos = vector7 - Vec2::new(10.0, 10.0) / 2.0;
            let color = Color::new(245, 200, 30, 155);
            for _ in 0..30 {
                // `:27381-27383`.
                let idx = pool.new_dust_direct(
                    rng, ctx, world, pos, 10, 10, 4, 0.0, 0.0, 50, color, 0.7,
                );
                let d = pool.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 1.0;
                // `:27384` — the second speck at 0.6 scale, doubled velocity.
                let idx = pool.new_dust_direct(
                    rng, ctx, world, pos, 10, 10, 4, 0.0, 0.0, 50, color, 0.6,
                );
                pool.slot_mut(idx).velocity *= 2.0;
            }
        }
        local_ai3
    } else if local_ai3 == 1.0 {
        // `:27388-27391` — the failed gate decays the latch.
        2.0
    } else {
        local_ai3
    }
}

// ------------------------------------------------------ site 20 (38664) --

/// Site 38664 — the world-item coin/heart/mana sparkle (`DrawItem`,
/// `Main.cs:38637-38666`). `allow_effects` is
/// `FocusHelper.AllowWorldItemsToEmitEffects` (`FocusHelper.cs:31` —
/// `!WorldUpdateStepper.DrawnThisUpdate`, the App's
/// `stepper.drawn_this_update` inverted at wiring); `is_coin` is
/// `item.IsACoin` (`registry::is_coin`); `light_r` is the light colour's R
/// byte at the item's centre tile (`Lighting.GetColor`, `:38619`).
/// Returns whether a dust spawned.
///
/// Gate order (`:38637`): the focus flag, the coin/58/109 kind test and
/// `color.R > 60` short-circuit BEFORE the single `Main.rand.Next(500)`
/// draw; the draw compares
/// `Next(500) - (|velocity.X| + |velocity.Y|) * 10 < color.R / 50` — the
/// right side is INTEGER division (`byte / int`). On a hit: type 43,
/// `Color.White`, alpha 254, scale 0.5 — coins override to
/// `default(Color)` (all-zero), alpha 0, scale 1 and the per-coin type
/// switch 71/72/73/74 -> 244/245/246/247 (`:38649-38663`). The spawn box
/// is the item's own `(position, width, height)`; post-spawn
/// `velocity *= 0f` (`:38665`). Types 43/244-247 take no pool tweak:
/// 1 gate draw + 6 default draws on a hit, 1 on a gate miss, 0 when the
/// pre-gates fail.
#[allow(clippy::too_many_arguments)]
pub(crate) fn world_item_sparkle(
    allow_effects: bool,
    item_type: i32,
    is_coin: bool,
    position: Vec2,
    width: i32,
    height: i32,
    velocity: Vec2,
    light_r: u8,
    rng: &mut UnifiedRandom,
    pool: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) -> bool {
    // `:38637` — the short-circuit chain, then the gate draw.
    if !(allow_effects && (is_coin || item_type == 58 || item_type == 109) && light_r > 60) {
        return false;
    }
    let speed = velocity.x.abs() + velocity.y.abs();
    if !(rng.next_max(500) as f32 - speed * 10.0 < (light_r as i32 / 50) as f32) {
        return false;
    }
    // `:38639-38648` — the defaults, then the coin override.
    let mut ty = 43;
    let mut new_color = Color::WHITE;
    let mut alpha = 254;
    let mut scale = 0.5;
    if is_coin {
        new_color = Color::default();
        alpha = 0;
        scale = 1.0;
    }
    // `:38649-38663` — the coin type switch.
    match item_type {
        71 => ty = 244,
        72 => ty = 245,
        73 => ty = 246,
        74 => ty = 247,
        _ => {}
    }
    // `:38664-38665`.
    let idx = pool.new_dust(
        rng, ctx, world, position, width, height, ty, 0.0, 0.0, alpha, new_color, scale,
    );
    pool.slot_mut(idx).velocity *= 0.0;
    true
}

// ------------------------------------------------------ site 21 (57766) --

/// Site 57766 — the retro liquid-surface sparkle (`oldDrawWater`,
/// `Main.cs:57747-57769`). `surface_y` is the arm's `vector2.Y` (the
/// waterline the rect machinery settled on), `rect_y` the draw rect's Y
/// (`value.Y`), `color` the tile's light colour bytes and `honey` the
/// cell's `Tile.honey()` flag. Returns whether a dust spawned.
///
/// Gate order: `(R > 20 || B > 20 || G > 20) && value.Y < 4` first (no
/// draws); then `num19 = max(R, G, B) / 30` (INTEGER divide) and the
/// single `Main.rand.Next(20000) < num19` draw — drawn even when `num19`
/// is 0 (the C# draws then compares). On a hit: `newColor` white
/// `(255,255,255)` — alpha 255, the 3-arg XNA ctor — or `(255,255,50)`
/// for honey; `NewDust((j*16, vector2.Y - 2), 16, 8, 43, 0, 0, 254,
/// newColor, 0.75)` then `velocity *= 0f`. Type 43 takes no pool tweak:
/// 1 gate draw + 6 default draws on a hit, 1 on a miss, 0 when the
/// pre-gates fail.
#[allow(clippy::too_many_arguments)]
pub(crate) fn liquid_surface_sparkle(
    j: i32,
    surface_y: f32,
    rect_y: i32,
    color: (u8, u8, u8),
    honey: bool,
    rng: &mut UnifiedRandom,
    pool: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) -> bool {
    let (r, g, b) = color;
    // `:57747` — the light/rect pre-gate.
    if !((r > 20 || b > 20 || g > 20) && rect_y < 4) {
        return false;
    }
    // `:57749-57758` — the max channel, integer-divided by 30.
    let mut num19 = r as i32;
    if g as i32 > num19 {
        num19 = g as i32;
    }
    if b as i32 > num19 {
        num19 = b as i32;
    }
    num19 /= 30;
    // `:57759` — the gate draw (always, once the pre-gate passed).
    if !(rng.next_max(20000) < num19) {
        return false;
    }
    // `:57761-57765` — the honey tint.
    let new_color = if honey {
        Color::new(255, 255, 50, 255)
    } else {
        Color::new(255, 255, 255, 255)
    };
    // `:57766-57767`.
    let idx = pool.new_dust(
        rng,
        ctx,
        world,
        Vec2::new(j as f32 * 16.0, surface_y - 2.0),
        16,
        8,
        43,
        0.0,
        0.0,
        254,
        new_color,
        0.75,
    );
    pool.slot_mut(idx).velocity *= 0.0;
    true
}

// -------------------------------------------------- sites 22/23 (57789/93) --

/// Sites 57789/57793 — the retro lava-bubble pair (`oldDrawWater`,
/// `Main.cs:57785-57803`). The caller owns the enclosing `tile.lava()`
/// arm (and its `num16` tint multipliers, which are draw-side, not dust);
/// `allow_misc` is `FocusHelper.AllowMiscDustEffects` (`FocusHelper.cs:39`,
/// the same `!DrawnThisUpdate` flag), `num11` the arm's surface-depth
/// float `(256 - liquid) / 32f`, `rect_y` the draw rect's Y (`value.Y`),
/// `liquid` the cell's liquid byte.
///
/// Gate order: `AllowMiscDustEffects && Dust.lavaBubbles < 200` first (no
/// draws — the budget reads the pool's `lava_bubbles` static, maintained
/// by the d50 phase-2 physics chain). Then TWO independent gates:
/// - `:57787-57790`: `liquid > 200 && Next(700) == 0` — the `Next(700)`
///   draw is short-circuited away when the liquid is shallow; on a hit
///   `NewDust((j*16, i*16), 16, 16, 35)` with ALL defaults (speeds 0,
///   alpha 0, `default(Color)`, scale 1).
/// - `:57791-57802`: `value.Y == 0 && Next(350) == 0` — drawn regardless
///   of the first gate's outcome; on a hit `NewDust((j*16, i*16 + num11*2
///   - 8), 16, 8, 35, 0, 0, 50, default, 1.5)` then the post chain:
///   `velocity *= 0.8`, `velocity.X *= 2`, `velocity.Y -= Next(1,7)*0.1`,
///   `if Next(10) == 0 { velocity.Y *= Next(2,5) }`, `noGravity = true`.
///
/// The type-35 pool tweak (`Dust.cs:243-247`: `velocity *= 0.1`,
/// `velocity.Y = -0.5`) rides the internals BEFORE the post chain. Draw
/// census: site 22 = 1 gate + 6 default; site 23 = 1 gate + 6 default +
/// `Next(1,7)` + `Next(10)` + the conditional `Next(2,5)` (9 or 10).
#[allow(clippy::too_many_arguments)]
pub(crate) fn lava_bubble_pair(
    j: i32,
    i: i32,
    num11: f32,
    rect_y: i32,
    liquid: u8,
    allow_misc: bool,
    rng: &mut UnifiedRandom,
    pool: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    // `:57785` — the focus + budget gate.
    if !(allow_misc && pool.lava_bubbles < 200) {
        return;
    }
    // `:57787-57790` — the deep-liquid bubble.
    if liquid > 200 && rng.next_max(700) == 0 {
        pool.new_dust(
            rng,
            ctx,
            world,
            Vec2::new(j as f32 * 16.0, i as f32 * 16.0),
            16,
            16,
            35,
            0.0,
            0.0,
            0,
            Color::default(),
            1.0,
        );
    }
    // `:57791-57802` — the surface bubble + its post-roll chain.
    if rect_y == 0 && rng.next_max(350) == 0 {
        let idx = pool.new_dust(
            rng,
            ctx,
            world,
            Vec2::new(j as f32 * 16.0, i as f32 * 16.0 + num11 * 2.0 - 8.0),
            16,
            8,
            35,
            0.0,
            0.0,
            50,
            Color::default(),
            1.5,
        );
        let d = pool.slot_mut(idx);
        d.velocity *= 0.8; // `:57794`
        d.velocity.x *= 2.0; // `:57795`
        d.velocity.y -= rng.next_range(1, 7) as f32 * 0.1; // `:57796`
        if rng.next_max(10) == 0 {
            // `:57797-57800` — the 1-in-10 kick, `Next(2,5)` integer draws.
            d.velocity.y *= rng.next_range(2, 5) as f32;
        }
        d.no_gravity = true; // `:57801`
    }
}

// =====================================================================
// Wiring seams (the draw-call wave). Each feed runs on the DRAW path and
// rolls the bare `Main.rand` slot (`rngs.global()`); the pure emitters
// above stay `&mut UnifiedRandom`-generic so the witnesses pin a seed.
// =====================================================================

/// `Lighting.GetColor(x, y)` (`Lighting.cs:216-242`) as XNA bytes: the raw
/// engine colour times `GlobalBrightness * 255`, truncated then clamped to
/// 255 per channel (the C# clamps AFTER the multiply, so a >1 channel lands
/// at 255, not a wrap). The light map stores the RAW engine colour (the
/// `shade` brightness multiply is a draw-side step), so the brightness is
/// applied here — `style.brightness` is `GlobalBrightness` (1.2, or 1.0
/// under the Darkness debuff, `Lighting.cs:108-116`).
pub(crate) fn light_color_bytes(light: &tr_core::lighting::LightMap, brightness: f32, x: i32, y: i32) -> (u8, u8, u8) {
    let c = light.get(x, y);
    let num = brightness * 255.0;
    let byte = |v: f32| ((v * num) as i32).min(255).max(0) as u8;
    (byte(c[0]), byte(c[1]), byte(c[2]))
}

/// Site 20's draw-call feed — `Main.DrawItems` (`Main.cs:38790-38798`) walks
/// the 400 item slots and calls `DrawItem` per live drop; the sparkle gate
/// (`:38637-38666`) is [`world_item_sparkle`]. The port's `items` vec is the
/// live-drop list in slot order (the dead are culled), so the walk is the
/// same order. `allow_effects` is `FocusHelper.AllowWorldItemsToEmitEffects`
/// (`!DrawnThisUpdate`, `FocusHelper.cs:31`). The light colour is
/// `Lighting.GetColor(item.Center.ToTileCoordinates())` (`:38619`).
///
/// The spawn box is the C#'s `item.width/height` = 16 (`WorldItem.cs:138`
/// sets `base.width = base.height = 16`); the port's physics box is
/// `item::SIZE` (14), so the dust box uses the C# 16, not the physics size —
/// the default block's `Next(width-4)` draw count depends on it.
pub(crate) fn item_sparkle_feed(
    items: &[crate::item::ItemDrop],
    allow_effects: bool,
    brightness: f32,
    light: &tr_core::lighting::LightMap,
    rng: &mut UnifiedRandom,
    pool: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    for it in items {
        // `:38604` — `!item.active || item.IsAir`: the port's list holds only
        // live, non-air drops (kind 0 / count 0 never enter it).
        // `:38619` — `Lighting.GetColor(item.Center.ToTileCoordinates())`:
        // the C# `item.Center` is `position + width/2` with the 16-px box
        // (`WorldItem.cs:138`), i.e. `pos + 8`, NOT the port's 14-px physics
        // centre (`pos + 7`) — the light tile can differ at a cell boundary.
        let (cx, cy) = (it.pos.x + 8.0, it.pos.y + 8.0);
        let (r, _, _) = light_color_bytes(
            light,
            brightness,
            (cx / tr_core::TILE_PX) as i32,
            (cy / tr_core::TILE_PX) as i32,
        );
        world_item_sparkle(
            allow_effects,
            it.kind as i32,
            crate::registry::is_coin(it.kind),
            it.pos,
            16, // `WorldItem.cs:138` — the C# item box, not the 14-px physics box
            16,
            it.vel,
            r,
            rng,
            pool,
            ctx,
            world,
        );
    }
}

/// Site 17's draw-call feed — the type-398 Moon Lord core arm of
/// `DrawNPCDirect` (`Main.cs:24701-24764`). The arm loops the two sides
/// (`num136` 0/1), finds the matching 397 hand (`ai[2] == num136 &&
/// ai[3] == core.whoAmI`, `:24724`), and — behind the DEAD `flag8` latch
/// (`bool flag8 = false`, `:24703`, never set) — spawns the tentacle-tip
/// puff (`:24756`). The port's hand→core link is `carrier_uid` (the C#'s
/// `ai[3] = whoAmI`, `NPC.cs:41227`); the side is `ai[2]` (0/1). The whole
/// cluster is dead in the C#, so this feed spawns nothing and draws nothing
/// — it exists so the fourth site is wired whole (the g12/g13 boundary
/// splits the cluster by line order). `flag8` is passed verbatim as `false`.
pub(crate) fn moon_lord_tentacle_feed(
    npcs: &[Npc],
    rng: &mut UnifiedRandom,
    pool: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    for core in npcs {
        if core.kind != NpcKind::MoonLordCore || core.hp <= 0 {
            continue;
        }
        let (ccx, ccy) = core.center();
        let core_center = Vec2::new(ccx, ccy);
        // `:24717` — the two-side loop. `flag9 = num136 == 0`; the side
        // vector's X is -1 for side 0, +1 for side 1 (`:24720`).
        for side in 0..2 {
            let side_x = if side == 0 { -1.0 } else { 1.0 };
            // `:24722-24729` — the matching hand: type 397, `ai[2] == side`,
            // linked to this core (`ai[3] == whoAmI` → `carrier_uid`).
            let hand = npcs.iter().find(|n| {
                n.kind == NpcKind::MoonLordHand
                    && n.hp > 0
                    && n.ai[2] == side as f32
                    && n.carrier_uid == core.uid
            });
            let Some(hand) = hand else { continue };
            let (hcx, hcy) = hand.center();
            let tip = tentacle_tip_position(core_center, Vec2::new(hcx, hcy), side_x);
            // `:24754-24757` — the dead `flag8` gate, verbatim.
            moon_lord_tentacle_tip_puff(false, tip, rng, pool, ctx, world);
        }
    }
}

/// Sites 18/19's draw-call feed — the type-550 Tavernkeep `ai[0] == 18` arm
/// of `DrawNPCExtras` (`Main.cs:27353-27397`, `!beforeDraw` only). The C#
/// runs it once per drawn NPC in the over-tiles pass; the port walks the
/// live NPC list in slot order. `localAI[3]` is the edge latch the emitter
/// reads and writes — it lives on the NPC (`npc.local_ai[3]`), so the feed
/// takes `&mut [Npc]`. The talked-to player is `Main.player[(int)ai[2]]`
/// (`:27365-27368`); the port has one session player (slot 0), so a non-zero
/// index finds no player and the emitter leaves the latch untouched.
pub(crate) fn tavernkeep_ale_feed(
    npcs: &mut [Npc],
    player: Option<&crate::player::Player>,
    rng: &mut UnifiedRandom,
    pool: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    for n in npcs.iter_mut() {
        // `:27353` — the kind + state gate; `:27355-27358` — `!beforeDraw`
        // (this feed IS the after-draw pass, so the gate is satisfied).
        if n.kind != NpcKind::DD2Bartender || n.hp <= 0 || n.ai[0] != 18.0 {
            continue;
        }
        let (ncx, ncy) = n.center();
        let npc_center = Vec2::new(ncx, ncy);
        let player_idx = n.ai[2] as i32;
        // `:27368` — `Main.player[num18]`. The port's single player is slot 0.
        let view = player.filter(|_| player_idx == 0).map(|p| {
            let (pcx, pcy) = p.center();
            AlePlayerView {
                held_item: p.held_item_kind as i32,
                direction: p.facing,
                hitbox: (p.x as i32, p.y as i32, p.width as i32, p.height as i32),
                center: Vec2::new(pcx, pcy),
            }
        });
        let new_latch = tavernkeep_ale_sparkle(
            npc_center,
            n.sprite_dir,
            player_idx,
            n.local_ai[3],
            view,
            rng,
            pool,
            ctx,
            world,
        );
        n.local_ai[3] = new_latch;
    }
}

/// The retro liquid-surface rect machinery (`oldDrawWater`,
/// `Main.cs:57668-57746`): the waterline `vector2.Y` and the draw rect's
/// `value.Y` the two dust gates read (`value.Y < 4` for the surface sparkle,
/// `value.Y == 0` for the lava surface bubble). `num11 = (256 - liquid)/32f`
/// is the cell's surface depth; `above`/`below` are the neighbour liquid
/// bytes, `above_solid` the `nactive && tileSolid && !tileSolidTop` test on
/// the cell above. The C# draws the rect too, but the port's liquid draw is
/// the `LiquidRenderer` record path — only the dust-gate inputs are needed
/// here, so this returns `(surface_y, rect_y)` without the sprite.
///
/// The branch order is the C#'s verbatim (`:57671-57746`): the shallow-below
/// arm (with its four sub-cases), the deep-above arm, the full-cell-above-solid
/// arm, then the five-cell scan below. `bg` only matters in the shallow-below
/// `< 200` sub-case (the background pass skips it, `:57686-57689`).
fn retro_surface_rect(
    j: i32,
    i: i32,
    liquid: u8,
    above: u8,
    below: u8,
    above_solid: bool,
    below_solid: bool,
    world: &World,
    bg: bool,
) -> Option<(f32, i32)> {
    let num11 = (256.0 - liquid as f32) / 32.0;
    // `:57668-57669` — the default waterline + rect.
    let mut surface_y = i as f32 * 16.0 + (num11 as i32) as f32 * 2.0;
    let mut rect_y = 0;
    // `:57671` — the below cell is not a full solid liquid block.
    if below < 245 && !below_solid {
        let num17 = (256.0 - below as f32) / 32.0;
        // `:57674-57682` — the opacity clamp (draw-side, not a dust input).
        if below < 200 {
            // `:57686-57689` — the background pass skips the shallow-below cell.
            if bg {
                return None;
            }
            if above > 0 {
                // `:57690-57694` — the C# writes `above > 0 && above > 0`
                // (a duplicated read); the `else if (above > 0)` at `:57695`
                // is therefore DEAD. Kept as the single live condition.
                rect_y = 4;
            } else if below > 0 {
                // `:57701-57705`.
                surface_y =
                    i as f32 * 16.0 + (num11 as i32) as f32 * 2.0 + (num17 as i32) as f32 * 2.0;
                rect_y = 4;
            } else {
                // `:57706-57710` — the isolated droplet.
                let num18 = num11 / 2.0;
                surface_y = i as f32 * 16.0
                    + (num18 as i32) as f32 * 2.0
                    + (num17 as i32) as f32 * 2.0;
                rect_y = 4;
            }
        } else {
            // `:57712-57716` — the below cell is deep (>= 200).
            rect_y = 4;
        }
    } else if above > 32 {
        // `:57718-57721`.
        rect_y = 4;
    } else if num11 < 1.0 && above_solid {
        // `:57722-57726` — a full cell under a solid ceiling.
        surface_y = i as f32 * 16.0;
        rect_y = 4;
    } else {
        // `:57727-57746` — the five-cell scan below for a shallow pocket.
        let mut flag2 = true;
        for k in (i + 1)..(i + 6) {
            let t = world.tile(j, k).copied().unwrap_or_default();
            let k_solid = t.active()
                && !t.inactive()
                && tr_core::tiledata::TILE_SOLID.has(t.kind)
                && !tr_core::tiledata::TILE_SOLID_TOP.has(t.kind);
            if k_solid {
                break;
            }
            if t.liquid < 200 {
                flag2 = false;
                break;
            }
        }
        if !flag2 {
            // `:57737-57741`.
            rect_y = 4;
        } else if above > 0 {
            // `:57742-57745`.
            rect_y = 2;
        }
    }
    Some((surface_y, rect_y))
}

/// Sites 21-23's draw-call feed — the retro `oldDrawWater` dust sites
/// (`Main.cs:57747-57803`), reached only when `!Lighting.NotRetro` (Retro /
/// Trippy, `Main.cs:57519-57522`). The port's liquid draw is the
/// `LiquidRenderer` record path, so this feed walks the same screen tile
/// range the C# does and runs ONLY the dust gates (the sprite draw stays on
/// the record path). `drew_lava` is the C#'s `Main.drewLava` latch
/// (`:57472` reset per `DrawWaters`, `:57938` set at the pass end): the lava
/// cells draw once per frame, on the first pass that reaches them.
///
/// The walk order is the C#'s row-major (`i` outer, `j` inner, `:57593-57595`);
/// the RNG draws ride the bare `Main.rand` slot in that order. `bg` selects
/// the background pass (the brightness gate is bypassed, `:57601`, and the
/// shallow-below cells skip, `:57686`). `allow_misc` is
/// `FocusHelper.AllowMiscDustEffects` (`!DrawnThisUpdate`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn retro_liquid_dust_feed(
    world: &World,
    light: &tr_core::lighting::LightMap,
    brightness: f32,
    camera: Vec2,
    screen: Vec2,
    zoom: f32,
    bg: bool,
    allow_misc: bool,
    drew_lava: &mut bool,
    rng: &mut UnifiedRandom,
    pool: &mut DustPool,
    ctx: &SpawnCtx,
) {
    // `:57573-57592` — the screen tile range. The port renders the unpadded
    // viewport (offScreenRange cancels out of the draw positions, the bg.rs
    // precedent), so the range is the view's world-px extent / 16 with the
    // C#'s -1/+2 (x) and -1/+5 (y) margins, clamped to the 5-tile world edge.
    let half = screen * 0.5 / zoom;
    let top_left = camera - half;
    let mut num7 = (top_left.x / 16.0 - 1.0) as i32;
    let mut num8 = ((top_left.x + screen.x / zoom) / 16.0) as i32 + 2;
    let mut num9 = (top_left.y / 16.0 - 1.0) as i32;
    let mut num10 = ((top_left.y + screen.y / zoom) / 16.0) as i32 + 5;
    num7 = num7.max(5);
    num8 = num8.min(world.width - 5);
    num9 = num9.max(5);
    num10 = num10.min(world.height - 5);
    // `:57593-57595` — row-major, the C#'s `i` outer / `j` inner.
    for i in num9..(num10 + 4) {
        for j in (num7 - 2)..(num8 + 2) {
            let Some(t) = world.tile(j, i).copied() else { continue };
            // `:57601` — the liquid / solid / brightness pre-gate.
            let nactive_solid = t.active()
                && !t.inactive()
                && tr_core::tiledata::TILE_SOLID.has(t.kind)
                && !tr_core::tiledata::TILE_SOLID_TOP.has(t.kind);
            let lit = {
                let c = light.get(j, i);
                brightness * (c[0] + c[1] + c[2]) / 3.0 > 0.0
            };
            if t.liquid == 0 || nactive_solid || !(lit || bg) {
                continue;
            }
            let color = light_color_bytes(light, brightness, j, i);
            let num11 = (256.0 - t.liquid as f32) / 32.0;
            let kind = t.liquid_kind();
            // `:57609-57653` — the lava/honey/shimmer style + the drewLava gate.
            // The lava arm (`:57611`) and the style gate (`:57650`) both skip
            // lava/honey once lava drew; the union is `(lava || honey) && drew`.
            let is_lava = kind == tr_core::tile::LiquidKind::Lava;
            let is_honey = kind == tr_core::tile::LiquidKind::Honey;
            if (is_lava || is_honey) && *drew_lava {
                continue;
            }
            // The neighbour facts for the rect machinery.
            let above_t = world.tile(j, i - 1).copied().unwrap_or_default();
            let below_t = world.tile(j, i + 1).copied().unwrap_or_default();
            let solid_of = |t: tr_core::tile::Tile| {
                t.active()
                    && !t.inactive()
                    && tr_core::tiledata::TILE_SOLID.has(t.kind)
                    && !tr_core::tiledata::TILE_SOLID_TOP.has(t.kind)
            };
            let Some((surface_y, rect_y)) = retro_surface_rect(
                j,
                i,
                t.liquid,
                above_t.liquid,
                below_t.liquid,
                solid_of(above_t),
                solid_of(below_t),
                world,
                bg,
            ) else {
                continue;
            };
            // `:57747-57769` — site 21, the surface sparkle (every liquid kind).
            liquid_surface_sparkle(j, surface_y, rect_y, color, is_honey, rng, pool, ctx, world);
            // `:57778-57803` — sites 22/23, the lava-bubble pair (lava only).
            if is_lava {
                lava_bubble_pair(j, i, num11, rect_y, t.liquid, allow_misc, rng, pool, ctx, world);
            }
        }
    }
    // `:57938` — the pass-end latch: lava drew (or the pass ran), so a later
    // pass this frame skips the lava cells.
    *drew_lava = true;
}

/// The capture-mode snow cull (`Main.DrawCapture`, `Main.cs:56735-56744`):
/// unless the capture area sits STRICTLY inside the current view with a
/// 16-px margin on every side, every live type-76 flake dies before the
/// chunk renders (the snow would otherwise smear across the capture). The
/// gate is the C#'s four-negation OR verbatim — it fires when ANY side
/// fails the strict containment, i.e. also for an area wholly OFF-screen;
/// only a strictly-inside area keeps its snow. `area` is the capture
/// `settings.Area` tile rect `(x, y, w, h)` (the `* 16` products are INTEGER
/// before the float cast, `:56735`); `view_pos`/`view_size` are the saved
/// `Main.screenPosition`/`screenWidth`/`screenHeight` (`vector`/`num2`/
/// `num3`, `:56694-56696`) — the PRE-capture view, not the area's camera.
/// The walk spans `Main.dust.Length` = 6001 slots INCLUDING the sentinel
/// (`:56737`, the pool's full slice — the sentinel is inactive, so the
/// extra slot is a no-op, kept for the verbatim range). No RNG draw, no
/// other type touched. Returns the cull count (witness convenience; the
/// C# is void). STAGED: `DrawCapture`'s GPU half is the S1-G-13
/// render-target seam (`CaptureRenderer` stub, ANALYSIS "The render-target
/// family"); the wiring calls this at `:56735`'s position when that lands.
#[allow(dead_code)]
pub(crate) fn capture_snow_cull(
    area: (i32, i32, i32, i32),
    view_pos: Vec2,
    view_size: Vec2,
    pool: &mut DustPool,
) -> usize {
    let (ax, ay, aw, ah) = area;
    let left = (ax * 16) as f32;
    let top = (ay * 16) as f32;
    let right = ((ax + aw) * 16) as f32;
    let bottom = ((ay + ah) * 16) as f32;
    // `:56735` — the containment gate, the C#'s negation shape kept.
    if !(!(left > view_pos.x - 16.0)
        || !(top > view_pos.y - 16.0)
        || !(right < view_pos.x + view_size.x + 16.0)
        || !(bottom < view_pos.y + view_size.y + 16.0))
    {
        return 0;
    }
    // `:56737-56743` — the full-array walk, active type-76 only.
    let mut culled = 0;
    for d in pool.slots_mut() {
        if d.active && d.dust_type == 76 {
            d.active = false;
            culled += 1;
        }
    }
    culled
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::particles::DUST_SENTINEL;

    /// The witness rig: an empty 64x64 world, the full 6000 budget (no
    /// pressure-band draws below slot 3000), a 1920x1080 screen at the
    /// origin (the cull never fires for the on-screen fixtures).
    fn rig() -> (DustPool, World, SpawnCtx) {
        let pool = DustPool::new();
        let world = World::new("g13".into(), 64, 64);
        let ctx = SpawnCtx {
            screen_size: Vec2::new(1920.0, 1080.0),
            max_dust_to_draw: 6000,
            ..SpawnCtx::default()
        };
        (pool, world, ctx)
    }

    fn live(pool: &DustPool) -> Vec<&tr_core::particles::Dust> {
        pool.slots()[..DUST_SENTINEL].iter().filter(|d| d.active).collect()
    }

    /// The pool's six default draws (`Dust.cs:153-193`) mirrored on an
    /// independent stream: returns the expected `(position, velocity,
    /// scale, frame_y)` for a spawn at `pos` with box `w x h`, speeds
    /// `(sx, sy)` and the caller's `scale` (the 0-box clamp to 5 and the
    /// per-100 frame-band shift included).
    #[allow(clippy::too_many_arguments)]
    fn mirror_default(
        m: &mut UnifiedRandom,
        ty: i32,
        pos: Vec2,
        w: i32,
        h: i32,
        sx: f32,
        sy: f32,
        scale: f32,
    ) -> (Vec2, Vec2, f32, i32) {
        let w = w.max(5);
        let h = h.max(5);
        let p = Vec2::new(
            pos.x + m.next_max(w - 4) as f32 + 4.0,
            pos.y + m.next_max(h - 4) as f32 + 4.0,
        );
        let v = Vec2::new(
            m.next_range(-20, 21) as f32 * 0.1 + sx,
            m.next_range(-20, 21) as f32 * 0.1 + sy,
        );
        let mut fy = 10 * m.next_max(3);
        let mut band = ty;
        while band >= 100 {
            band -= 100;
            fy += 30;
        }
        let s = (1.0 + m.next_range(-20, 21) as f32 * 0.01) * scale;
        (p, v, s, fy)
    }

    /// The type-6 tweak mirror (`Dust.cs:216-221`).
    fn mirror_type6(m: &mut UnifiedRandom, v: Vec2, s: f32) -> (Vec2, f32) {
        let vy = m.next_range(-10, 6) as f32 * 0.1;
        (Vec2::new(v.x * 0.3, vy), s * 0.7)
    }

    /// The type-35 tweak mirror (`Dust.cs:243-247`) — no draws.
    fn mirror_type35(v: Vec2) -> Vec2 {
        Vec2::new(v.x * 0.1, -0.5)
    }

    // ------------------------------------------------ site 17 witnesses --

    /// `:24732-24756` — the tip maths: a hand directly below the shoulder
    /// anchor makes `vector33` vertical, `ToRotation = PI/2`, so the tip
    /// rotation collapses to `-num139` and the 340-px arm swings by the
    /// acos bend. Both sides mirror about the core.
    #[test]
    fn w4_g13_the_tentacle_tip_maths_matches_the_arm_geometry() {
        let core = Vec2::new(1000.0, 500.0);
        // Right side (`side_x = +1`): shoulder (1220, 440); hand centre
        // chosen so `hand + (0,76) - shoulder = (0, 400)` -> vector33
        // (0, 200), length 200.
        let hand = Vec2::new(1220.0, 440.0 + 400.0 - 76.0);
        let tip = tentacle_tip_position(core, hand, 1.0);
        let num139 = ((200.0f32 / 340.0) as f64).acos() as f32 * -1.0;
        let angle = std::f32::consts::FRAC_PI_2 - num139 - std::f32::consts::PI / 2.0;
        let expect = Vec2::new(1220.0, 440.0) + rotated_by(Vec2::new(0.0, 340.0), angle as f64);
        assert_eq!(tip, expect);
        // The link's half-scale: |tip - shoulder| is exactly 340.
        assert!(((tip - Vec2::new(1220.0, 440.0)).length() - 340.0).abs() < 1e-3);
        // Left side mirrors the shoulder anchor.
        let hand_l = Vec2::new(780.0, 440.0 + 400.0 - 76.0);
        let tip_l = tentacle_tip_position(core, hand_l, -1.0);
        assert!((tip_l.x - (2000.0 - tip.x)).abs() < 1e-3, "mirrored about the core");
        assert!((tip_l.y - tip.y).abs() < 1e-3);
    }

    /// The dead latch: `flag8 = false` (`:24703`) — zero spawns, zero
    /// draws, the stream untouched.
    #[test]
    fn w4_g13_the_dead_flag8_arm_spends_nothing() {
        let (mut pool, world, ctx) = rig();
        let mut rng = UnifiedRandom::new(0x5EED);
        moon_lord_tentacle_tip_puff(false, Vec2::new(300.0, 300.0), &mut rng, &mut pool, &ctx, &world);
        assert!(live(&pool).is_empty());
        assert_eq!(rng, UnifiedRandom::new(0x5EED));
    }

    /// The enabled emitter (the verbatim port the dead flag gates): one
    /// type-6 puff, 6 default draws + the type-6 tweak draw, `noGravity`,
    /// the 0-box clamp to 5.
    #[test]
    fn w4_g13_the_tentacle_puff_mirrors_seven_draws() {
        let (mut pool, world, ctx) = rig();
        let pos = Vec2::new(300.0, 300.0);
        let mut rng = UnifiedRandom::new(0x5EED);
        moon_lord_tentacle_tip_puff(true, pos, &mut rng, &mut pool, &ctx, &world);
        let slots = live(&pool);
        assert_eq!(slots.len(), 1);
        let d = slots[0];
        assert_eq!(d.dust_type, 6);
        assert!(d.no_gravity);
        assert_eq!(d.alpha, 0);
        assert_eq!(d.color, Color::default());
        let mut m = UnifiedRandom::new(0x5EED);
        let (p, v, s, fy) = mirror_default(&mut m, 6, pos, 0, 0, 0.0, 0.0, 1.0);
        let (v, s) = mirror_type6(&mut m, v, s);
        assert_eq!(d.position, p);
        assert_eq!(d.velocity, v);
        assert_eq!(d.scale, s);
        assert_eq!(d.frame_y, fy);
        assert_eq!(d.frame_x, 60);
        assert_eq!(rng, m, "the mirror walks the stream");
    }

    // -------------------------------------------- sites 18/19 witnesses --

    fn ale_player(center: Vec2) -> AlePlayerView {
        AlePlayerView {
            held_item: 353,
            direction: 1,
            hitbox: (center.x as i32 - 10, center.y as i32 - 20, 20, 40),
            center,
        }
    }

    /// The full burst: 60 type-4 specks, 360 draws, the 0.7/0.6 scale
    /// alternation, the doubled second velocity, `noGravity` on the first
    /// only, the gold tint, the latch write to 1.
    #[test]
    fn w4_g13_the_ale_burst_replays_sixty_specks_in_order() {
        let (mut pool, world, ctx) = rig();
        let npc = Vec2::new(300.0, 300.0);
        // The player stands 30 px to the NPC's left: direction 1 faces
        // the NPC (`Sign(300 - 270) = 1`), the hitbox distance is < 46.
        let player = ale_player(Vec2::new(270.0, 300.0));
        let mut rng = UnifiedRandom::new(0x5EED);
        let latch = tavernkeep_ale_sparkle(
            npc, 1, 0, 0.0, Some(player), &mut rng, &mut pool, &ctx, &world,
        );
        assert_eq!(latch, 1.0);
        let slots = live(&pool);
        assert_eq!(slots.len(), 60);
        let pos = npc + Vec2::new(30.0, -6.0) - Vec2::new(5.0, 5.0);
        let color = Color::new(245, 200, 30, 155);
        let mut m = UnifiedRandom::new(0x5EED);
        for (i, d) in slots.iter().enumerate() {
            assert_eq!(d.dust_type, 4);
            assert_eq!(d.alpha, 50);
            assert_eq!(d.color, color);
            let first = i % 2 == 0;
            let scale = if first { 0.7 } else { 0.6 };
            let (p, v, s, fy) = mirror_default(&mut m, 4, pos, 10, 10, 0.0, 0.0, scale);
            assert_eq!(d.position, p);
            assert_eq!(d.scale, s);
            assert_eq!(d.frame_y, fy);
            assert_eq!(d.velocity, if first { v } else { v * 2.0 });
            assert_eq!(d.no_gravity, first, "only the 0.7 speck is weightless");
        }
        assert_eq!(rng, m, "the mirror walks the stream");
    }

    /// The edge latch: a re-entry with `localAI[3] == 1` writes 1 again
    /// but does NOT burst (`localAI[3] != num20` fails) — zero draws.
    #[test]
    fn w4_g13_the_ale_latch_is_edge_triggered() {
        let (mut pool, world, ctx) = rig();
        let npc = Vec2::new(300.0, 300.0);
        let player = ale_player(Vec2::new(270.0, 300.0));
        let mut rng = UnifiedRandom::new(0x5EED);
        let latch = tavernkeep_ale_sparkle(
            npc, 1, 0, 1.0, Some(player), &mut rng, &mut pool, &ctx, &world,
        );
        assert_eq!(latch, 1.0);
        assert!(live(&pool).is_empty());
        assert_eq!(rng, UnifiedRandom::new(0x5EED));
    }

    /// The failed gates: no Ale / wrong facing / out of range each decay
    /// `1 -> 2` (or leave the other values) with zero draws; the
    /// player-index window refuses outside `[0, 255)`.
    #[test]
    fn w4_g13_the_ale_gates_decay_the_latch_without_draws() {
        let npc = Vec2::new(300.0, 300.0);
        // No Ale.
        let mut p = ale_player(Vec2::new(270.0, 300.0));
        p.held_item = 0;
        // Wrong facing (the player is to the RIGHT, direction still 1).
        let mut p2 = ale_player(Vec2::new(330.0, 300.0));
        p2.direction = 1;
        // Out of range (>= 46 px).
        let p3 = ale_player(Vec2::new(200.0, 300.0));
        for player in [p, p2, p3] {
            let (mut pool, world, ctx) = rig();
            let mut rng = UnifiedRandom::new(0x5EED);
            let latch = tavernkeep_ale_sparkle(
                npc, 1, 0, 1.0, Some(player), &mut rng, &mut pool, &ctx, &world,
            );
            assert_eq!(latch, 2.0, "the failed gate decays 1 -> 2");
            assert!(live(&pool).is_empty());
            assert_eq!(rng, UnifiedRandom::new(0x5EED));
        }
        // The index window: -1 and 255 leave the latch untouched.
        for idx in [-1, 255] {
            let (mut pool, world, ctx) = rig();
            let mut rng = UnifiedRandom::new(0x5EED);
            let latch = tavernkeep_ale_sparkle(
                npc, 1, idx, 0.0, None, &mut rng, &mut pool, &ctx, &world,
            );
            assert_eq!(latch, 0.0);
            assert!(live(&pool).is_empty());
            assert_eq!(rng, UnifiedRandom::new(0x5EED));
        }
    }

    // ------------------------------------------------ site 20 witnesses --

    /// The gate census across seeds: the `Next(500)` draw rides the bare
    /// stream exactly when the pre-gates pass; a hit spawns one dust with
    /// the coin/heart defaults and a zeroed velocity; the mirror walks
    /// every draw in order. Seed 100011 is a pinned hitting seed
    /// (`Next(500) = 2 < 24` — derived offline from the decompiled
    /// `UnifiedRandom.cs`, the GOLDEN_42 precedent); the other four are
    /// pinned misses.
    #[test]
    fn w4_g13_the_item_sparkle_gate_and_spawn_mirror_the_stream() {
        let pos = Vec2::new(300.0, 300.0);
        let vel = Vec2::new(1.5, -0.5);
        for seed in [100011i32, 1, 7, 42, 0x5EED] {
            for (ty, coin) in [(71i32, true), (58, false), (109, false)] {
                let (mut pool, world, ctx) = rig();
                let mut rng = UnifiedRandom::new(seed);
                let mut m = UnifiedRandom::new(seed);
                // The mirror predicts the gate: light R 200 -> R/50 = 4,
                // speed sum 2.0 -> the draw must land below 24.
                let gate = m.next_max(500);
                let hit = (gate as f32) - 2.0 * 10.0 < (200i32 / 50) as f32;
                let fired = world_item_sparkle(
                    true, ty, coin, pos, 20, 20, vel, 200, &mut rng, &mut pool, &ctx, &world,
                );
                assert_eq!(fired, hit, "seed {seed} type {ty}");
                let slots = live(&pool);
                assert_eq!(slots.len(), usize::from(hit));
                if hit {
                    let d = slots[0];
                    let expect_ty = match ty {
                        71 => 244,
                        _ => 43,
                    };
                    assert_eq!(d.dust_type, expect_ty);
                    if coin {
                        assert_eq!(d.alpha, 0);
                        assert_eq!(d.color, Color::default());
                    } else {
                        assert_eq!(d.alpha, 254);
                        assert_eq!(d.color, Color::WHITE);
                    }
                    let scale = if coin { 1.0 } else { 0.5 };
                    let (p, _, s, fy) = mirror_default(&mut m, expect_ty, pos, 20, 20, 0.0, 0.0, scale);
                    assert_eq!(d.position, p);
                    assert_eq!(d.velocity, Vec2::ZERO, "velocity *= 0f");
                    assert_eq!(d.scale, s);
                    assert_eq!(d.frame_y, fy);
                }
                assert_eq!(rng, m, "seed {seed} type {ty}: the mirror walks the stream");
            }
        }
    }

    /// The pre-gates short-circuit BEFORE the draw: a dark tile
    /// (`R <= 60`), a non-coin/heart/mana kind and the focus flag each
    /// spend zero draws.
    #[test]
    fn w4_g13_the_item_sparkle_pre_gates_draw_nothing() {
        let pos = Vec2::new(300.0, 300.0);
        for (allow, ty, coin, r) in [
            (false, 71i32, true, 200u8),
            (true, 1, false, 200),
            (true, 58, false, 60),
        ] {
            let (mut pool, world, ctx) = rig();
            let mut rng = UnifiedRandom::new(0x5EED);
            let fired = world_item_sparkle(
                allow, ty, coin, pos, 20, 20, Vec2::ZERO, r, &mut rng, &mut pool, &ctx, &world,
            );
            assert!(!fired);
            assert!(live(&pool).is_empty());
            assert_eq!(rng, UnifiedRandom::new(0x5EED));
        }
    }

    /// The coin type switch: 72/73/74 map to 245/246/247 (the 71 row is
    /// covered by the gate witness). Seed 100011 is the pinned hitting
    /// seed (`Next(500) = 2 < 5` at zero velocity / R 255, derived
    /// offline from the decompiled `UnifiedRandom.cs`); the field mirror
    /// then pins the row.
    #[test]
    fn w4_g13_the_coin_rows_map_to_their_sparkle_types() {
        let pos = Vec2::new(300.0, 300.0);
        for (ty, expect) in [(72i32, 245i32), (73, 246), (74, 247)] {
            let seed = 100011;
            let (mut pool, world, ctx) = rig();
            let mut rng = UnifiedRandom::new(seed);
            let fired = world_item_sparkle(
                true, ty, true, pos, 20, 20, Vec2::ZERO, 255, &mut rng, &mut pool, &ctx, &world,
            );
            assert!(fired);
            let slots = live(&pool);
            assert_eq!(slots.len(), 1);
            assert_eq!(slots[0].dust_type, expect);
            assert_eq!(slots[0].alpha, 0);
            assert_eq!(slots[0].color, Color::default());
            let mut m = UnifiedRandom::new(seed);
            let _ = m.next_max(500);
            let (p, _, s, fy) = mirror_default(&mut m, expect, pos, 20, 20, 0.0, 0.0, 1.0);
            assert_eq!(slots[0].position, p);
            assert_eq!(slots[0].scale, s);
            assert_eq!(slots[0].frame_y, fy);
            assert_eq!(rng, m);
        }
    }

    // ------------------------------------------------ site 21 witnesses --

    /// The surface-sparkle census across seeds: the pre-gate draws
    /// nothing, the `Next(20000)` draw always rides once past it (even
    /// when `max/30 == 0`), a hit spawns the white/honey 43 at
    /// `(j*16, surface_y - 2)` with a zeroed velocity. Seed 100702 is a
    /// pinned hitting seed (`Next(20000) = 4 < 6`, derived offline from
    /// the decompiled `UnifiedRandom.cs`); the other four are misses.
    #[test]
    fn w4_g13_the_surface_sparkle_gate_and_spawn_mirror_the_stream() {
        for seed in [100702i32, 1, 7, 42, 0x5EED] {
            for honey in [false, true] {
                let (mut pool, world, ctx) = rig();
                let mut rng = UnifiedRandom::new(seed);
                let mut m = UnifiedRandom::new(seed);
                // Light (200, 10, 10): max 200 -> num19 = 6.
                let gate = m.next_max(20000);
                let hit = gate < 6;
                let fired = liquid_surface_sparkle(
                    20, 320.0, 0, (200, 10, 10), honey, &mut rng, &mut pool, &ctx, &world,
                );
                assert_eq!(fired, hit, "seed {seed} honey {honey}");
                let slots = live(&pool);
                assert_eq!(slots.len(), usize::from(hit));
                if hit {
                    let d = slots[0];
                    assert_eq!(d.dust_type, 43);
                    assert_eq!(d.alpha, 254);
                    let expect_color = if honey {
                        Color::new(255, 255, 50, 255)
                    } else {
                        Color::new(255, 255, 255, 255)
                    };
                    assert_eq!(d.color, expect_color);
                    let (p, _, s, fy) =
                        mirror_default(&mut m, 43, Vec2::new(320.0, 318.0), 16, 8, 0.0, 0.0, 0.75);
                    assert_eq!(d.position, p);
                    assert_eq!(d.velocity, Vec2::ZERO);
                    assert_eq!(d.scale, s);
                    assert_eq!(d.frame_y, fy);
                }
                assert_eq!(rng, m, "seed {seed}: the mirror walks the stream");
            }
        }
    }

    /// The pre-gates: a dark surface (`max <= 20`) and a deep rect
    /// (`value.Y >= 4`) draw nothing; a dim-but-passing light
    /// (`max 21..29 -> num19 == 0`) still spends the `Next(20000)` draw.
    #[test]
    fn w4_g13_the_surface_sparkle_pre_gates_and_zero_budget_draw() {
        for (color, rect_y, expect_draw) in [
            ((20u8, 20u8, 20u8), 0i32, false),
            ((200, 10, 10), 4, false),
            ((21, 10, 10), 0, true),
        ] {
            let (mut pool, world, ctx) = rig();
            let mut rng = UnifiedRandom::new(0x5EED);
            let fired = liquid_surface_sparkle(
                20, 320.0, rect_y, color, false, &mut rng, &mut pool, &ctx, &world,
            );
            assert!(!fired, "num19 == 0 or a failed pre-gate never hits");
            assert!(live(&pool).is_empty());
            let mut m = UnifiedRandom::new(0x5EED);
            if expect_draw {
                let _ = m.next_max(20000);
            }
            assert_eq!(rng, m, "the draw census");
        }
    }

    // -------------------------------------------- sites 22/23 witnesses --

    /// The lava pair's full draw census across pinned seeds: the two
    /// gates are independent (`Next(700)` only when `liquid > 200`,
    /// `Next(350)` only when `rect_y == 0`), the surface bubble's post
    /// chain spends `Next(1,7)` + `Next(10)` + the conditional
    /// `Next(2,5)`, and the type-35 tweak rides the internals before the
    /// post writes. The seeds are derived offline from the decompiled
    /// `UnifiedRandom.cs` (the GOLDEN_42 precedent), in the EMITTER's
    /// draw order: 100323 deep-only, 100042 surface-only, 219697 double,
    /// 108460 surface-with-kick (`Next(10) == 0`, `Next(2,5) = 2`),
    /// 1/42 both-miss.
    #[test]
    fn w4_g13_the_lava_pair_mirrors_the_gate_and_post_chain() {
        for seed in [100323i32, 100042, 219697, 108460, 1, 42] {
            let (mut pool, world, ctx) = rig();
            let mut rng = UnifiedRandom::new(seed);
            // The probe predicts both gates in the EMITTER's draw order:
            // Next(700), the deep spawn's six default draws (only on a
            // hit), then Next(350).
            let mut probe = UnifiedRandom::new(seed);
            let hit_deep = probe.next_max(700) == 0;
            if hit_deep {
                for _ in 0..6 {
                    probe.next();
                }
            }
            let hit_surface = probe.next_max(350) == 0;
            lava_bubble_pair(
                20, 20, 3.5, 0, 255, true, &mut rng, &mut pool, &ctx, &world,
            );
            let slots = live(&pool);
            assert_eq!(slots.len(), usize::from(hit_deep) + usize::from(hit_surface));
            // The mirror walks the same order: gate, deep block, gate,
            // surface block + post chain.
            let mut m = UnifiedRandom::new(seed);
            let _ = m.next_max(700);
            let mut at = 0;
            if hit_deep {
                let d = slots[at];
                at += 1;
                assert_eq!(d.dust_type, 35);
                assert_eq!(d.alpha, 0);
                assert!(!d.no_gravity, "the deep bubble keeps gravity");
                let (p, v, s, fy) =
                    mirror_default(&mut m, 35, Vec2::new(320.0, 320.0), 16, 16, 0.0, 0.0, 1.0);
                let v = mirror_type35(v);
                assert_eq!(d.position, p);
                assert_eq!(d.velocity, v);
                assert_eq!(d.scale, s);
                assert_eq!(d.frame_y, fy);
            }
            let _ = m.next_max(350);
            if hit_surface {
                let d = slots[at];
                assert_eq!(d.dust_type, 35);
                assert_eq!(d.alpha, 50);
                assert!(d.no_gravity);
                // num11 3.5 -> y = 320 + 7 - 8 = 319.
                let (p, v, s, fy) =
                    mirror_default(&mut m, 35, Vec2::new(320.0, 319.0), 16, 8, 0.0, 0.0, 1.5);
                let mut v = mirror_type35(v);
                v *= 0.8;
                v.x *= 2.0;
                v.y -= m.next_range(1, 7) as f32 * 0.1;
                if m.next_max(10) == 0 {
                    v.y *= m.next_range(2, 5) as f32;
                }
                assert_eq!(d.position, p);
                assert_eq!(d.velocity, v);
                assert_eq!(d.scale, s);
                assert_eq!(d.frame_y, fy);
            }
            assert_eq!(rng, m, "seed {seed}: the mirror walks the stream");
        }
    }

    /// A forced double hit (both gates land): seed 219697, derived
    /// offline from the decompiled `UnifiedRandom.cs` in the emitter's
    /// draw order (`Next(700) = 0`, six deep default draws, `Next(350) =
    /// 0`). Pins the two-slot order: the deep bubble first, the surface
    /// bubble second.
    #[test]
    fn w4_g13_the_lava_pair_orders_deep_before_surface() {
        let seed = 219697;
        let (mut pool, world, ctx) = rig();
        let mut rng = UnifiedRandom::new(seed);
        lava_bubble_pair(20, 20, 3.5, 0, 255, true, &mut rng, &mut pool, &ctx, &world);
        let slots = live(&pool);
        assert_eq!(slots.len(), 2);
        // The deep bubble's box is (320, 320) 16x16: the default block's
        // Next(12)+4 offset puts y in [324, 336).
        assert!(slots[0].position.y >= 324.0 && slots[0].position.y < 336.0);
        assert!(!slots[0].no_gravity);
        assert!(slots[1].no_gravity);
        assert_eq!(slots[1].alpha, 50);
    }

    /// The budget + focus gates draw nothing; the shallow-liquid
    /// short-circuit skips `Next(700)` but still spends `Next(350)` when
    /// the rect is at the surface; a non-zero rect Y skips both.
    #[test]
    fn w4_g13_the_lava_gates_census_the_skipped_draws() {
        // Budget full.
        let (mut pool, world, ctx) = rig();
        pool.lava_bubbles = 200;
        let mut rng = UnifiedRandom::new(0x5EED);
        lava_bubble_pair(20, 20, 3.5, 0, 255, true, &mut rng, &mut pool, &ctx, &world);
        assert!(live(&pool).is_empty());
        assert_eq!(rng, UnifiedRandom::new(0x5EED));
        // Focus off.
        let (mut pool, world, ctx) = rig();
        let mut rng = UnifiedRandom::new(0x5EED);
        lava_bubble_pair(20, 20, 3.5, 0, 255, false, &mut rng, &mut pool, &ctx, &world);
        assert!(live(&pool).is_empty());
        assert_eq!(rng, UnifiedRandom::new(0x5EED));
        // Shallow liquid (<= 200): Next(700) skipped, Next(350) still
        // drawn. Seed 100323's FIRST draw lands Next(350) == 0 (derived
        // offline), so the surface spawn + post chain are exercised.
        let (mut pool, world, ctx) = rig();
        let mut rng = UnifiedRandom::new(100323);
        let mut m = UnifiedRandom::new(100323);
        let hit = m.next_max(350) == 0;
        assert!(hit, "the pinned shallow seed hits the surface gate");
        lava_bubble_pair(20, 20, 3.5, 0, 200, true, &mut rng, &mut pool, &ctx, &world);
        assert_eq!(live(&pool).len(), usize::from(hit));
        if hit {
            // The surface spawn's post chain rides the mirror.
            let (_, v, _, _) = mirror_default(&mut m, 35, Vec2::new(320.0, 319.0), 16, 8, 0.0, 0.0, 1.5);
            let mut v = mirror_type35(v);
            v *= 0.8;
            v.x *= 2.0;
            v.y -= m.next_range(1, 7) as f32 * 0.1;
            if m.next_max(10) == 0 {
                v.y *= m.next_range(2, 5) as f32;
            }
            assert_eq!(live(&pool)[0].velocity, v);
        }
        assert_eq!(rng, m);
        // Non-surface rect: both gates skipped, zero draws.
        let (mut pool, world, ctx) = rig();
        let mut rng = UnifiedRandom::new(0x5EED);
        lava_bubble_pair(20, 20, 3.5, 4, 255, true, &mut rng, &mut pool, &ctx, &world);
        assert!(live(&pool).is_empty());
        assert_eq!(rng, UnifiedRandom::new(0x5EED));
    }

    // ------------------------------------------------ wiring witnesses --

    /// The item-sparkle feed walks the drop list in slot order and rolls the
    /// bare stream exactly as the per-drop `world_item_sparkle` calls do —
    /// the feed's stream state equals a hand-walked mirror over the same
    /// drops (the stream-parity contract: the feed adds no draws of its own).
    #[test]
    fn w4_g13_the_item_feed_walks_the_drops_in_order_on_the_bare_stream() {
        let (mut pool, world, ctx) = rig();
        // A lit world so the light gate passes: compute a light map over the
        // drops' tiles (the surface sky seeds them).
        let light = tr_core::lighting::LightMap::compute(
            &world, 0, 0, 63, 63, &tr_core::lighting::LightParams::default(),
        );
        let brightness = tr_core::lighting::GLOBAL_BRIGHTNESS;
        // Two coins + a heart, all at lit surface tiles.
        let drops = vec![
            crate::item::ItemDrop::new(71, 1, 0, 100.0, 100.0, 0.0, 0.0, 0.0),
            crate::item::ItemDrop::new(58, 1, 0, 200.0, 100.0, 0.0, 0.0, 0.0),
            crate::item::ItemDrop::new(73, 1, 0, 300.0, 100.0, 0.0, 0.0, 0.0),
        ];
        let mut rng = UnifiedRandom::new(0x5EED);
        item_sparkle_feed(&drops, true, brightness, &light, &mut rng, &mut pool, &ctx, &world);
        // The mirror: the same per-drop calls, same order, same stream.
        let (mut pool2, _, _) = rig();
        let mut m = UnifiedRandom::new(0x5EED);
        for it in &drops {
            let (cx, cy) = (it.pos.x + 8.0, it.pos.y + 8.0);
            let (r, _, _) = light_color_bytes(
                &light, brightness,
                (cx / tr_core::TILE_PX) as i32, (cy / tr_core::TILE_PX) as i32,
            );
            world_item_sparkle(
                true, it.kind as i32, crate::registry::is_coin(it.kind),
                it.pos, 16, 16, it.vel, r, &mut m, &mut pool2, &ctx, &world,
            );
        }
        assert_eq!(rng, m, "the feed adds no draws beyond the per-drop calls");
        assert_eq!(live(&pool).len(), live(&pool2).len());
    }

    /// The Moon Lord feed is a DEAD arm: the flag8 latch is false, so even a
    /// living core with both linked hands spawns nothing and draws nothing.
    #[test]
    fn w4_g13_the_moon_lord_feed_is_dead_and_draws_nothing() {
        let (mut pool, world, ctx) = rig();
        let mut core = Npc::new(NpcKind::MoonLordCore, 1000.0, 500.0, 1, 0.0);
        core.hp = core.max_hp;
        core.uid = 7;
        let mut hand0 = Npc::new(NpcKind::MoonLordHand, 800.0, 400.0, 1, 0.0);
        // The parts' `lifeMax` is 0 (they die with the core, `NPC.cs`); give
        // the fixture a survivable pool so the feed's `hp > 0` hand-find runs.
        hand0.hp = 1000;
        hand0.ai[2] = 0.0;
        hand0.carrier_uid = 7;
        let mut hand1 = Npc::new(NpcKind::MoonLordHand, 1200.0, 400.0, 1, 0.0);
        hand1.hp = 1000;
        hand1.ai[2] = 1.0;
        hand1.carrier_uid = 7;
        let npcs = vec![core, hand0, hand1];
        let mut rng = UnifiedRandom::new(0x5EED);
        moon_lord_tentacle_feed(&npcs, &mut rng, &mut pool, &ctx, &world);
        assert!(live(&pool).is_empty(), "the dead flag8 arm never spawns");
        assert_eq!(rng, UnifiedRandom::new(0x5EED), "zero draws");
    }

    /// The ale feed writes the latch onto the NPC and bursts only on the
    /// 0→1 edge; a re-entry with the latch already 1 draws nothing. The
    /// stream parity: the feed's draws equal the emitter's own burst census.
    #[test]
    fn w4_g13_the_ale_feed_latches_the_npc_and_bursts_on_the_edge() {
        let (mut pool, world, ctx) = rig();
        let mut n = Npc::new(NpcKind::DD2Bartender, 300.0, 300.0, 1, 0.0);
        n.hp = n.max_hp;
        n.ai[0] = 18.0;
        n.ai[2] = 0.0; // talks to player 0
        n.sprite_dir = 1;
        let mut p = crate::player::Player::spawn(&world);
        p.x = 270.0;
        p.y = 300.0;
        p.held_item_kind = 353; // Ale
        p.facing = 1; // faces the NPC to the right
        let mut npcs = vec![n];
        let mut rng = UnifiedRandom::new(0x5EED);
        tavernkeep_ale_feed(&mut npcs, Some(&p), &mut rng, &mut pool, &ctx, &world);
        assert_eq!(npcs[0].local_ai[3], 1.0, "the latch edges to 1");
        assert_eq!(live(&pool).len(), 60, "the 30x2 burst");
        // Re-entry: latch already 1, no burst, no draws.
        let before = rng.clone();
        let count = live(&pool).len();
        tavernkeep_ale_feed(&mut npcs, Some(&p), &mut rng, &mut pool, &ctx, &world);
        assert_eq!(npcs[0].local_ai[3], 1.0);
        assert_eq!(live(&pool).len(), count, "no second burst on the held latch");
        assert_eq!(rng, before, "the held-latch re-entry draws nothing");
    }

    /// The retro feed's rect machinery: a full liquid cell with a full cell
    /// below and nothing above lands `rect_y == 0` (the surface), so the
    /// lava surface-bubble gate (`value.Y == 0`) is reachable; a cell with a
    /// shallow pocket below lands `rect_y == 4` (the sparkle gate's `< 4`
    /// fails, the bubble gate's `== 0` fails). Pins the two gate inputs.
    #[test]
    fn w4_g13_the_retro_rect_machinery_feeds_the_two_gates() {
        let mut world = World::new("g13".into(), 64, 64);
        // A full cell (liquid 255) in a full column: below >= 245 → the
        // else-if chain; above == 0, num11 == 0.03 < 1, above not solid →
        // the five-cell scan (k = 21..25); EVERY scanned cell must stay
        // >= 200 (an empty cell's liquid 0 < 200 flips flag2), so fill all
        // five; flag2 stays true, above == 0 → rect_y stays 0.
        for k in 21..26 {
            world.tile_mut(20, k).unwrap().liquid = 255;
        }
        let (sy, ry) = retro_surface_rect(20, 20, 255, 0, 255, false, false, &world, false).unwrap();
        assert_eq!(ry, 0, "the full-column surface keeps rect_y 0");
        assert!((sy - (20.0 * 16.0 + 0.0)).abs() < 1e-3, "num11 truncates to 0");
        // A cell with a shallow pocket below (liquid 100 < 200): the
        // shallow-below arm takes it before the scan; above == 0, below > 0
        // → the `:57701` waterline, rect_y 4.
        world.tile_mut(20, 21).unwrap().liquid = 100;
        let (_, ry2) = retro_surface_rect(20, 20, 255, 0, 100, false, false, &world, false).unwrap();
        assert_eq!(ry2, 4, "the shallow pocket below lifts rect_y to 4");
        // The background pass skips the shallow-below cell entirely.
        assert!(retro_surface_rect(20, 20, 255, 0, 100, false, false, &world, true).is_none());
    }

    // ---------------------------------------- capture snow-cull witnesses --

    /// Seed three live type-76 flakes + one type-35 + one dead 76 directly
    /// (the cull is RNG-free; no spawn draws needed).
    fn cull_rig() -> DustPool {
        let mut pool = DustPool::new();
        for (i, (ty, active)) in [(76, true), (76, true), (76, true), (35, true), (76, false)]
            .into_iter()
            .enumerate()
        {
            let d = pool.slot_mut(i);
            d.dust_type = ty;
            d.active = active;
        }
        pool
    }

    /// `Main.cs:56735` — an area STRICTLY inside the view (16-px margin on
    /// every side) keeps its snow: the gate is false, nothing dies.
    #[test]
    fn w4_g13_a_strictly_inside_capture_area_keeps_its_snow() {
        let mut pool = cull_rig();
        // View (0,0)-(1920,1080); area tiles (2,2)-(118,65) → px
        // (32,32)-(1888,1040): every side clears the ±16 margin strictly.
        let n = capture_snow_cull((2, 2, 116, 63), Vec2::ZERO, Vec2::new(1920.0, 1080.0), &mut pool);
        assert_eq!(n, 0, "the containment gate holds");
        assert_eq!(live(&pool).len(), 4, "all three flakes + the 35 survive");
    }

    /// Any side failing the strict containment fires the cull: exactly the
    /// live type-76 slots die, the 35 and the dead 76 are untouched. The
    /// four boundary touches (== on each side) all fail the strict compare.
    #[test]
    fn w4_g13_the_cull_kills_only_live_type76_on_any_boundary_failure() {
        // Left edge touches: area.x*16 == view.x - 16 → `>` fails → cull.
        let mut pool = cull_rig();
        let n = capture_snow_cull((-1, 2, 10, 10), Vec2::ZERO, Vec2::new(1920.0, 1080.0), &mut pool);
        assert_eq!(n, 3, "the three live flakes die");
        assert!(pool.slot(3).active, "the type-35 survives");
        assert!(!pool.slot(4).active, "the dead 76 stays dead");
        assert_eq!(live(&pool).len(), 1);
        // Right edge touches: (x+w)*16 == view.x + width + 16 → `<` fails.
        let mut pool = cull_rig();
        // (111+10)*16 = 1936 == 1920+16 → boundary → cull.
        let n = capture_snow_cull((111, 2, 10, 10), Vec2::ZERO, Vec2::new(1920.0, 1080.0), &mut pool);
        assert_eq!(n, 3, "the right-boundary touch culls too");
        // A wholly OFF-screen area also culls (the OR fires on every side).
        let mut pool = cull_rig();
        let n = capture_snow_cull((500, 500, 10, 10), Vec2::ZERO, Vec2::new(1920.0, 1080.0), &mut pool);
        assert_eq!(n, 3, "off-screen areas cull (the gate is containment, not overlap)");
    }
}
use crate::app::{App, RngKey};

/// `Main.maxScreenW` under the HiDef profile (`Main.cs:3709` — the port
/// renders HiDef; `sand.rs`'s 1920 stand-in is its own recorded deviation).
const MAX_SCREEN_W: f32 = 4096.0;

/// `SceneMetrics.SnowTileMax` (`SceneMetrics.cs:932`).
const SNOW_TILE_MAX: f32 = 6000.0;

/// The `Main` statics `Main.snowing` reads (`Main.cs:13032-13116`), gathered
/// by the (future) wiring caller. `scaled_position`/`scaled_size` are
/// `Main.Camera.ScaledPosition`/`ScaledSize` — the world-space view rect
/// (the `scene_scan.rs` convention: `camera - screen*0.5/zoom`,
/// `screen/zoom`).
pub(crate) struct SnowingEnv {
    /// `Main.remixWorld` (`:13034`) — the REMIX seed flag.
    pub remix_world: bool,
    /// `Main.gamePaused` (`:13040`).
    pub game_paused: bool,
    /// `Main.SceneMetrics.SnowTileCount` (`:13040`/`:13044`).
    pub snow_tile_count: i32,
    /// `Main.SceneMetrics.SurfaceAtmospherics` (`:13040`).
    pub surface_atmospherics: bool,
    /// `Main.cloudAlpha` (`:13044-13046` passim).
    pub cloud_alpha: f32,
    /// `Main.Camera.ScaledSize` (`:13038`).
    pub scaled_size: Vec2,
    /// `Main.Camera.ScaledPosition` (`:13039`).
    pub scaled_position: Vec2,
    /// `Main.gfxQuality` (`:13057`).
    pub gfx_quality: f32,
    /// `NPC.IsADeerclopsNearScreen()` (`:13047`, `NPC.cs:44534-44537` —
    /// `npcsFoundForCheckActive[668]`, any live Deerclops).
    pub deerclops_near: bool,
    /// `Main.player[Main.myPlayer].velocity.Y` (`:13067-13070`).
    pub player_vy: f32,
    /// `Main.raining` (`:13092`).
    pub raining: bool,
    /// `Main.windSpeedCurrent` (`:13094`/`:13099`).
    pub wind_speed_current: f32,
}

/// `Main.snowing()` (`Main.cs:13032-13116`) — the ambient snowfall's dust
/// spawn, verbatim. Runs on the bare `Main.rand` (the caller passes
/// `NamedRngs::global()` — `:17238` sits before every `SwapRandom`).
///
/// **DEPRECATED (research/83 keep-one): the merger drops this body for
/// d130's `snow.rs::snowing` — see the module-doc RETIRE-NOTE. F2/F3
/// landed on d130 at `85250712`; F1+F5+F7 APPLIED at `099df89e2` (the
/// global `UnifiedRandom` stream — the splitmix retired, the g11 §-
/// adjudication confirmed and folded; §-receipt in research/83).**
///
/// `Main.snowDust` (`Main.cs:621`) is a Main static the pool does not carry
/// (d50's note: the 76 arm's `snowDust++` is cited and skipped); the budget
/// reads the live type-76 population instead — the `SandStormCount`
/// stand-in pattern (`sand.rs:115`, `Sandstorm.cs:166`). The per-iteration
/// `try/catch` (`:13055`/`:13112-13114`) has no port equivalent (no
/// exceptions); dropped with this note.
pub(crate) fn snowing(
    env: &SnowingEnv,
    rng: &mut UnifiedRandom,
    ctx: &SpawnCtx,
    world: &World,
    pool: &mut DustPool,
) {
    // `:13034-13037` — the remix world has no snowfall.
    if env.remix_world {
        return;
    }
    let scaled_size = env.scaled_size;
    let scaled_position = env.scaled_position;
    // `:13040-13043`.
    if env.game_paused || env.snow_tile_count <= 0 || !env.surface_atmospherics {
        return;
    }
    // `:13044` — `Math.Pow` is the f64 pow (the float args promote).
    let mut num = ((env.snow_tile_count as f32 / SNOW_TILE_MAX) as f64)
        .powf((4.0 - env.cloud_alpha * 3.0) as f64) as f32;
    // `:13045` — the inner `(int)` truncates BEFORE the storm multiplier.
    let mut num2 = ((500.0 * (scaled_size.x / MAX_SCREEN_W)) as i32 as f32
        * (1.0 + 2.0 * env.cloud_alpha)) as i32;
    // `:13046`.
    let num3 = 1.0 + 50.0 * env.cloud_alpha;
    // `:13047-13052` — the Deerclops throttle.
    let flag = env.deerclops_near;
    if flag {
        num /= 20.0;
        num2 /= 3;
    }
    let mut i = 0;
    // `:13053` — `(float)i < num3`.
    while (i as f32) < num3 {
        i += 1;
        // `:13057-13060` — the budget gate BREAKS the loop (not continue).
        let snow_dust = crate::minecart::live_type(pool, 76) as f32;
        if !(snow_dust < num2 as f32 * (env.gfx_quality / 2.0 + 0.5) + num2 as f32 * 0.1) {
            break;
        }
        // `:13061-13064`.
        if !(rng.next_float() < num) {
            continue;
        }
        // `:13065-13066`.
        let mut num4 = rng.next_max(scaled_size.x as i32 + 1500) - 750;
        let mut num5 = scaled_position.y as i32 - rng.next_max(50);
        // `:13067-13070` — the falling player drags the spawn band up.
        if env.player_vy > 0.0 {
            num5 -= env.player_vy as i32;
        }
        // `:13071-13078` — the edge pushes; the second `Next(5)` rolls only
        // when the first refused (the C# `else if`).
        if rng.next_max(5) == 0 {
            num4 = rng.next_max(500) - 500;
        } else if rng.next_max(5) == 0 {
            num4 = rng.next_max(500) + scaled_size.x as i32;
        }
        // `:13079-13082` — off-screen x spreads y over the view; the
        // `(double)` products truncate (the f32→f64 promotion is exact).
        if num4 < 0 || num4 as f32 > scaled_size.x {
            num5 += rng.next_max(((scaled_size.y as f64) * 0.8) as i32)
                + ((scaled_size.y as f64) * 0.1) as i32;
        }
        // `:13083-13086` — the world-px offset + the open-air tile gate.
        num4 += scaled_position.x as i32;
        let num6 = num4 / 16;
        let num7 = num5 / 16;
        let open = world
            .tile(num6, num7)
            .is_some_and(|t| !(t.active() && !t.inactive()) && t.wall == 0);
        if !open {
            continue;
        }
        // `:13088` — site 1: the type-76 flake, 10×10 box, defaults.
        let num8 = pool.new_dust(
            rng,
            ctx,
            world,
            Vec2::new(num4 as f32, num5 as f32),
            10,
            10,
            76,
            0.0,
            0.0,
            0,
            Color::default(),
            1.0,
        );
        let d = pool.slot_mut(num8);
        // `:13089-13091`.
        d.scale += env.cloud_alpha * 0.2;
        d.velocity.y = 3.0 + rng.next_max(30) as f32 * 0.1;
        d.velocity.y *= d.scale;
        if !env.raining {
            // `:13094-13095` — the dry wind arm.
            d.velocity.x = env.wind_speed_current + rng.next_range(-10, 10) as f32 * 0.1;
            d.velocity.x += env.wind_speed_current * 15.0;
        } else {
            // `:13099-13100` — the storm arm: `Math.Sqrt(Math.Abs(w))`
            // promotes to f64; `Math.Sign` is the three-way sign (Rust's
            // `signum` maps +0 to +1 — the sqrt(0) product is 0 either way,
            // but the explicit match keeps the C# shape).
            let sign = if env.wind_speed_current > 0.0 {
                1.0
            } else if env.wind_speed_current < 0.0 {
                -1.0
            } else {
                0.0
            };
            d.velocity.x = (env.wind_speed_current.abs() as f64).sqrt() as f32
                * sign
                * (env.cloud_alpha + 0.5)
                * 10.0
                + rng.next_float() * 0.2
                - 0.1;
            d.velocity.y *= 0.5;
        }
        // `:13102-13108` — the shared tail (the SECOND `scale +=` is the
        // C#'s own double bump).
        d.velocity.y *= 1.0 + 0.3 * env.cloud_alpha;
        d.scale += env.cloud_alpha * 0.2;
        if flag {
            d.scale -= 0.5;
        }
        d.velocity *= 1.0 + env.cloud_alpha * 0.5;
    }
}

/// The `Microsoft.Xna.Framework.Rectangle` `TeleportEffect` takes — the
/// entity's `getRect()` (world px, integer box).
#[derive(Clone, Copy)]
pub(crate) struct EffectRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl EffectRect {
    /// `Rectangle.TopLeft()` (`:13324`/`:13336`/`:13354`).
    fn top_left(self) -> Vec2 {
        Vec2::new(self.x as f32, self.y as f32)
    }

    /// `Rectangle.Center` — XNA's integer halves (`X + Width/2`).
    fn center(self) -> (i32, i32) {
        (self.x + self.width / 2, self.y + self.height / 2)
    }
}

/// `Main.TeleportEffect` (`Main.cs:13233-13464`) — the App-level
/// dispatcher (research/116 union-exec): builds the `TeleportDustEnv`
/// from the disjoint App fields + the caller's pass stream, resolves the
/// Audio façade at the C# call position, and calls the surviving g61
/// body (`teleport_dust::teleport_effect`) for ALL styles. `side` is
/// unread in the whole C# body — dropped; `other_position` feeds style
/// 10 (Entry leg = newPos, Exit leg = the OLD position).
///
/// `key` is the calling pass's `SwapRandom` slot (one RNG slot per C#
/// pass): `Player.Teleport` → `UpdatePlayers` (`Main.cs:17754`),
/// `NPC.Teleport` → the NPC pass's stream, `Wiring.Teleport` →
/// `UpdateWorldGen`. The sounds ride the same stream (the C# draws them
/// from `Main.rand` mid-pass): `(2, n)` → `play_npc_item_cue(n, …)` (the
/// case-2 `Next(-6,7)` pitch), `(19, 0)` → `play_splash_sound(0, …)` (the
/// case-19 `Next(-10,11)` pitch) — the same draws the witnesses pin.
pub(crate) fn teleport_effect(
    app: &mut App,
    rect: EffectRect,
    style: i32,
    extra_info: i32,
    dust_count_mult: f32,
    other_position: Vec2,
    key: RngKey,
) {
    let ctx = app.spawn_ctx();
    // Gap 4 (research/116): the STORED `Main.waterStyle` static
    // (`Main.cs:1105`, `DrawWaters` `:57475`) — not an inline recompute.
    let water_style = app.water_style;
    // The sequential-borrow split (the `recall_teleport` precedent): the
    // sound closure and the env both need the pass stream while `audio`
    // and `dust` are borrowed — the stream is taken OUT by value (the
    // `UnifiedRandom` clone) into a `RefCell` both sides share through
    // short `borrow_mut` scopes (the body never holds a draw borrow
    // across a `play_sound` call — every cue sits between complete dust
    // statements, exactly the C# order), and the advanced state is
    // written back after the body. No other consumer touches the pass
    // stream mid-call, so this is observation-identical to the C#'s one
    // `Main.rand`.
    let rng_cell = std::cell::RefCell::new(app.rngs.stream(key).clone());
    let audio = &mut app.audio;
    let mut play_sound = |id: i32, sound_style: i32, x: f32, y: f32| {
        let mut rng = rng_cell.borrow_mut();
        match id {
            2 => {
                audio.play_npc_item_cue(sound_style, x, y, &mut rng);
            }
            19 => {
                audio.play_splash_sound(sound_style, x, y, &mut rng);
            }
            _ => {}
        }
    };
    let mut cell_rng = CellRng(&rng_cell);
    let mut env = crate::teleport_dust::TeleportDustEnv {
        world: &app.world,
        dust: &mut app.dust,
        dust_ctx: &ctx,
        rng: &mut cell_rng,
        play_sound: &mut play_sound,
    };
    crate::teleport_dust::teleport_effect(
        &mut env,
        (rect.x, rect.y, rect.width, rect.height),
        style,
        extra_info,
        dust_count_mult,
        other_position,
        water_style,
    );
    *app.rngs.stream(key) = rng_cell.into_inner();
}

/// The dispatcher's shared-stream adapter: a `DustRng` over the
/// `RefCell<UnifiedRandom>` the sound closure also draws from (each
/// trait method takes one short `borrow_mut` — the body's draws and the
/// cues interleave at the C# statement order without overlapping
/// borrows).
struct CellRng<'a>(&'a std::cell::RefCell<UnifiedRandom>);

impl tr_core::particles::DustRng for CellRng<'_> {
    fn next_max(&mut self, max: i32) -> i32 {
        self.0.borrow_mut().next_max(max)
    }
    fn next_range(&mut self, min: i32, max: i32) -> i32 {
        self.0.borrow_mut().next_range(min, max)
    }
    fn next_float(&mut self) -> f32 {
        self.0.borrow_mut().next_float()
    }
}

/// W4-g11 wiring — the `Player.Teleport` request (`Player.cs:37414-37496`):
/// the movement core queues it (`Player::teleport_style_with_extra`), the
/// drain replays the C# order — the Entry `TeleportEffect` at the OLD box
/// (`:37444`), the move (`:37446`), the Exit at the NEW box (`:37489`), the
/// `teleportTime`/`teleportStyle` grants (`:37490-37491`). `key` is the
/// CALLER's pass stream: `UpdatePlayers` for the player-tick callers
/// (`Main.cs:17754`), `UpdateWorldGen` for the wiring pads (`Wiring.cs:3154`
/// under `WorldGen.UpdateWorld`), `UpdateProjectiles` for the Queen-Slime
/// hook (`Projectile.cs:52775` under the projectile pass).
pub(crate) struct PlayerTeleportReq {
    pub old_x: f32,
    pub old_y: f32,
    pub new_x: f32,
    pub new_y: f32,
    pub style: i32,
    /// The Entry effect's `extraInfo2` (the OLD portal colour, `:37433-37436`).
    pub extra_info2: i32,
    /// The Exit effect's `extraInfo2` (the NEW portal colour, `:37475-37478`).
    pub extra_info: i32,
    /// `:37442` — `Clamp(1 - teleportTime*0.99, 0.01, 1)`.
    pub mult: f32,
    pub key: RngKey,
}

/// The `Player.Teleport` replay (W4-g11 wiring, union-exec per
/// research/116): the Entry effect at the OLD box (`otherPosition` =
/// newPos, `:37444`), the movement core, the style-8 double Item6 cue
/// (`:37450-37453` — TWO pitch draws on the pass stream, old then new),
/// the myPlayer camera transition (`:37456-37463` — style 1 →
/// `lerpSpeedupDelay = 10`, else 0; the wiring pads no longer start it
/// inline, this replay owns it for BOTH callers), the Exit effect at the
/// NEW box (`otherPosition` = the OLD position, `:37489`), then the
/// grants (`:37490-37491` — the C#-position write; the inline path's
/// duplicate grant is retired, gap 6). The map-refresh/black-fade halves
/// (`mapTime`/`maxQ`/`renderNow`, `:37464-37473`) stay unported (cited,
/// research/116 gap 5); the style-4 portalPhysicsFlag/gravity writes
/// (`:37479-37480`) are the portal physics lane's.
pub(crate) fn player_teleport(app: &mut App, req: PlayerTeleportReq) {
    let Some(p) = app.player.as_ref() else {
        return;
    };
    let (w, h) = (p.width as i32, p.height as i32);
    let (pw, ph) = (p.width, p.height);
    let old_rect = EffectRect { x: req.old_x as i32, y: req.old_y as i32, width: w, height: h };
    // `:37444` — the Entry effect BEFORE the position write; style 10's
    // `otherPosition` is newPos on this leg (gap 2).
    teleport_effect(
        app,
        old_rect,
        req.style,
        req.extra_info2,
        req.mult,
        Vec2::new(req.new_x, req.new_y),
        req.key,
    );
    // `:37446`/`:37455`/`:37488` — the move (no re-queue).
    if let Some(p) = app.player.as_mut() {
        p.teleport_move_core(req.new_x, req.new_y, req.style);
    }
    // `:37450-37453` — the style-8 double cue: Item6 at the OLD position
    // then at newPos, two case-2 pitch draws on the pass stream (gap 5).
    if req.style == 8 {
        let rng = app.rngs.stream(req.key);
        app.audio.play_npc_item_cue(6, req.old_x, req.old_y, rng);
        let rng = app.rngs.stream(req.key);
        app.audio.play_npc_item_cue(6, req.new_x, req.new_y, rng);
    }
    // `:37456-37463` — the myPlayer camera transition (SP: always
    // myPlayer): style 1 arms the 10-tick lerp speedup delay, else 0.
    // The OLD centre feeds the tracking-arm compensation (the port's
    // `start_camera_transition_for_player_teleport` distance gate).
    {
        let old_center = Vec2::new(req.old_x + pw * 0.5, req.old_y + ph * 0.5);
        let delay = if req.style == 1 { 10 } else { 0 };
        let mut state = std::mem::take(&mut app.camera_state);
        crate::world_sim::camera::start_camera_transition_for_player_teleport(
            &mut state,
            old_center,
            0.1,
            delay,
        );
        app.camera_state = state;
    }
    let new_rect = EffectRect { x: req.new_x as i32, y: req.new_y as i32, width: w, height: h };
    // `:37489` — the Exit effect AFTER the move; style 10's
    // `otherPosition` is the OLD position on this leg (gap 2).
    teleport_effect(
        app,
        new_rect,
        req.style,
        req.extra_info,
        req.mult,
        Vec2::new(req.old_x, req.old_y),
        req.key,
    );
    // `:37490-37491` — the grants (the single C#-position write, gap 6).
    if let Some(p) = app.player.as_mut() {
        p.teleport_time = 1.0;
        p.teleport_style_id = req.style;
    }
}

/// W4-g11 wiring — the `NPC.Teleport` request (`NPC.cs:81035-81058`): the
/// wiring-pad drain queues it, the replay runs the Entry effect at the OLD
/// box (`:81044`), the position write (`:81045`), the Exit at the NEW box
/// (`:81051`), the grants (`:81052-81053`). `index` is the NPC's slot in
/// `App::npcs`; `key` the caller's pass stream (`UpdateWorldGen` for the
/// pads, `Wiring.cs:3170`).
pub(crate) struct NpcTeleportReq {
    pub index: usize,
    pub new_x: f32,
    pub new_y: f32,
    pub style: i32,
    pub extra_info: i32,
    pub key: RngKey,
}

/// The `NPC.Teleport` replay (W4-g11 wiring). The MP `SendData(65)`
/// (`:81054-81057`) is single-player dead.
pub(crate) fn npc_teleport(app: &mut App, req: NpcTeleportReq) {
    let Some(n) = app.npcs.get(req.index) else {
        return;
    };
    let (w, h) = n.size();
    let (w, h) = (w as i32, h as i32);
    // `:81037-81041` — the Entry reads the OLD portal colour.
    let extra_info2 = if req.style == 4 { n.last_portal_color_index } else { 0 };
    // `:81042` — `teleportTime > 0 ? 0.3 : 1`.
    let mult = if n.teleport_time > 0.0 { 0.3 } else { 1.0 };
    let old_rect = EffectRect { x: n.x as i32, y: n.y as i32, width: w, height: h };
    let old_pos = Vec2::new(n.x, n.y);
    // `:81044` — the Entry effect BEFORE the position write; style 10's
    // `otherPosition` is newPos on this leg (gap 2, `NPC.cs:81044`).
    teleport_effect(
        app,
        old_rect,
        req.style,
        extra_info2,
        mult,
        Vec2::new(req.new_x, req.new_y),
        req.key,
    );
    // `:81045-81050` — the move + the portal hand-off.
    let Some(n) = app.npcs.get_mut(req.index) else {
        return;
    };
    n.x = req.new_x;
    n.y = req.new_y;
    let exit_extra = if req.style == 4 {
        n.last_portal_color_index = req.extra_info;
        n.last_portal_color_index
    } else {
        extra_info2
    };
    let new_rect = EffectRect { x: req.new_x as i32, y: req.new_y as i32, width: w, height: h };
    // `:81051` — the Exit effect AFTER the move; style 10's
    // `otherPosition` is the OLD position on this leg (gap 2).
    teleport_effect(app, new_rect, req.style, exit_extra, mult, old_pos, req.key);
    // `:81052-81053` — the grants.
    if let Some(n) = app.npcs.get_mut(req.index) {
        n.teleport_time = 1.0;
        n.teleport_style_id = req.style;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::sound::Listener;
    use tr_core::particles::DUST_SENTINEL;

    const KEY: RngKey = RngKey::UpdatePlayers;

    /// The teleport rig (the d51 `rig` pattern): an empty world, the full
    /// 6000 dust budget (no pressure-band draws below slot 3000), the
    /// camera parked on the effect box, the pass stream pinned to `seed`,
    /// and the audio listener ON the box centre so the attenuation gate
    /// passes deterministically (the cue's one pitch draw then rides the
    /// pass stream, exactly like the C#).
    fn rig(seed: i32) -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = World::new("dust".into(), 64, 128);
        app.gfx_quality.max_dust_to_draw = 6000;
        app.camera = Vec2::new(320.0, 640.0);
        app.audio.set_listener(Listener {
            camera_center: (310.0, 621.0),
            sound_volume: 1.0,
            ambient_volume: 1.0,
            quiet_ambient_sounds: false,
            game_menu: false,
            generating_or_loading: false,
            server: false,
        });
        *app.rngs.stream(KEY) = UnifiedRandom::new(seed);
        app
    }

    fn rect() -> EffectRect {
        EffectRect { x: 300, y: 600, width: 20, height: 42 }
    }

    fn live(app: &App) -> Vec<&tr_core::particles::Dust> {
        app.dust.slots().iter().filter(|d| d.active).collect()
    }

    fn stream_of(app: &mut App) -> UnifiedRandom {
        app.rngs.stream(KEY).clone()
    }

    /// The six `NewDust` default-block draws (`Dust.cs:153-193`) for a box
    /// clamped to ≥5 — none of this batch's types (159/164/58/180/263/33/
    /// 35/76) rolls a per-type tweak draw beyond the shared block (33/35
    /// tweak without draws).
    fn default_block(rng: &mut UnifiedRandom, pos: Vec2, w: i32, h: i32) -> (Vec2, Vec2, f32) {
        let w = w.max(5);
        let h = h.max(5);
        let px = pos.x + rng.next_max(w - 4) as f32 + 4.0;
        let py = pos.y + rng.next_max(h - 4) as f32 + 4.0;
        let vx = rng.next_range(-20, 21) as f32 * 0.1;
        let vy = rng.next_range(-20, 21) as f32 * 0.1;
        let _ = rng.next_max(3); // frame row
        let scale = 1.0 + rng.next_range(-20, 21) as f32 * 0.01;
        (Vec2::new(px, py), Vec2::new(vx, vy), scale)
    }

    /// The Item6/Item8 cue's one case-2 pitch draw (`sound.rs::
    /// item_use_case2`'s default arm — styles 6/8 hit no remap/volume arm).
    fn item_pitch(rng: &mut UnifiedRandom) {
        let _ = rng.next_range(-6, 7);
    }

    /// Sites 2/3 — styles 0/1: the Item6/Item8 cue FIRST (one pitch draw),
    /// then `Width*Height/5 * mult` puffs, each the six default draws + the
    /// `Next(20,70)` scale roll; j<10 adds 0.25, j<5 adds another 0.25.
    #[test]
    fn w4_g11_styles_0_and_1_roll_the_scale_ladder_after_the_cue() {
        for (style, ty) in [(0, 159), (1, 164)] {
            let mut app = rig(0x5EED);
            // 20*42/5 = 168; ×0.05 → 8.4 → 8 puffs (j<5 and j<10 both fire
            // on the first five).
            teleport_effect(&mut app, rect(), style, 0, 0.05, Vec2::ZERO, KEY);
            let slots = live(&app);
            assert_eq!(slots.len(), 8);
            let mut rng = UnifiedRandom::new(0x5EED);
            item_pitch(&mut rng); // the cue precedes the loop (`:13239`)
            for (j, d) in slots.iter().enumerate() {
                assert_eq!(d.dust_type, ty);
                let (p, v, _) = default_block(&mut rng, rect().top_left(), 20, 42);
                assert_eq!(d.position, p);
                assert_eq!(d.velocity, v);
                let mut scale = rng.next_range(20, 70) as f32 * 0.01;
                if j < 10 {
                    scale += 0.25;
                }
                if j < 5 {
                    scale += 0.25;
                }
                assert_eq!(d.scale, scale, "the roll OVERWRITES the spawn scale");
            }
            assert_eq!(stream_of(&mut app), rng);
        }
    }

    /// Site 4 — style 2: NO sound, 50×mult GhostWhite type-58 puffs
    /// (alpha 150, scale 1.2), each velocity ×0.5.
    #[test]
    fn w4_g11_style_2_is_the_soundless_ghostwhite_burst() {
        let mut app = rig(7);
        teleport_effect(&mut app, rect(), 2, 0, 0.1, Vec2::ZERO, KEY); // 5 puffs
        let slots = live(&app);
        assert_eq!(slots.len(), 5);
        let mut rng = UnifiedRandom::new(7);
        for d in &slots {
            assert_eq!(d.dust_type, 58);
            assert_eq!(d.alpha, 150);
            assert_eq!(d.color, Color::new(248, 248, 255, 255));
            let (p, v, s) = default_block(&mut rng, rect().top_left(), 20, 42);
            assert_eq!(d.position, p);
            assert_eq!(d.velocity, v * 0.5);
            assert_eq!(d.scale, s * 1.2);
        }
        // No cue: exactly 5×6 draws, nothing else.
        assert_eq!(stream_of(&mut app), rng);
    }

    /// Site 5 — style 3: the Item6 cue, then 50×mult type-180 noGravity
    /// puffs with the five-roll ×0.75 ladder, the two ×2/×1.2 gates, the
    /// `Next(2)` fadeIn/scale re-roll and the final ×0.8.
    #[test]
    fn w4_g11_style_3_mirrors_the_full_shrink_ladder() {
        let mut app = rig(0xC0FFEE);
        teleport_effect(&mut app, rect(), 3, 0, 0.1, Vec2::ZERO, KEY); // 5 puffs
        let slots = live(&app);
        assert_eq!(slots.len(), 5);
        let mut rng = UnifiedRandom::new(0xC0FFEE);
        item_pitch(&mut rng);
        for d in &slots {
            assert_eq!(d.dust_type, 180);
            assert!(d.no_gravity);
            let (p, mut v, mut s) = default_block(&mut rng, rect().top_left(), 20, 42);
            assert_eq!(d.position, p);
            for _ in 0..5 {
                if rng.next_max(3) == 0 {
                    v *= 0.75;
                }
            }
            if rng.next_max(3) == 0 {
                v *= 2.0;
                s *= 1.2;
            }
            if rng.next_max(3) == 0 {
                v *= 2.0;
                s *= 1.2;
            }
            let mut fade = 0.0;
            if rng.next_max(2) == 0 {
                fade = rng.next_range(75, 100) as f32 * 0.01;
                s = rng.next_range(25, 75) as f32 * 0.01;
            }
            s *= 0.8;
            assert_eq!(d.velocity, v);
            assert_eq!(d.scale, s);
            assert_eq!(d.fade_in, fade);
        }
        assert_eq!(stream_of(&mut app), rng);
    }

    /// Site 6 — style 4: the portal colour by `extraInfo % 2` (union-exec:
    /// the canonical `ui::color::hsl_to_rgb` — the C#-exact f32→f64 widen +
    /// banker's rounding; the entry orange is pinned byte-exact), `A = 255`
    /// overwrites the helper's 66, and the fixed 1.2/0.4/noLight/noGravity
    /// shape carries NO post-spawn draws.
    #[test]
    fn w4_g11_style_4_paints_the_portal_pair() {
        for extra in [0, 1] {
            let mut app = rig(11);
            teleport_effect(&mut app, rect(), 4, extra, 0.05, Vec2::ZERO, KEY); // 8 puffs
            let slots = live(&app);
            assert_eq!(slots.len(), 8);
            let expected = if extra % 2 != 0 {
                let c = crate::ui::color::hsl_to_rgb(0.52, 1.0, 0.6, 255);
                Color::new(c[0], c[1], c[2], 255)
            } else {
                // hsl(0.12, 1, 0.5): num2 = 1, t = 0 — r = hue2rgb(0.4533)
                // = 1 → 255, g = hue2rgb(0.12) = 0.72 → 184, b =
                // hue2rgb(-0.2133 → 0.7867) = 0 → 0 (banker's-rounded).
                Color::new(255, 184, 0, 255)
            };
            let mut rng = UnifiedRandom::new(11);
            item_pitch(&mut rng);
            for d in &slots {
                assert_eq!(d.dust_type, 263);
                assert_eq!(d.color, expected);
                assert!(d.no_light && d.no_gravity);
                assert_eq!(d.scale, 1.2);
                assert_eq!(d.fade_in, 0.4);
                let (p, v, _) = default_block(&mut rng, rect().top_left(), 20, 42);
                assert_eq!(d.position, p);
                assert_eq!(d.velocity, v);
            }
            assert_eq!(stream_of(&mut app), rng);
        }
    }

    /// Sites 7/8 — styles 5/7: the +24-height column (`dustWater()` /
    /// type 35), the `vy = vy*0 - 3.5` / `vx *= 1.5` shape, alpha 130,
    /// scale 0.8, fadeIn 1.2 — and the cue AFTER the loop (the style-0
    /// Splash's one `Next(-10,11)` for 5, the Item8 pitch for 7).
    #[test]
    fn w4_g11_styles_5_and_7_play_the_cue_after_the_column() {
        for style in [5, 7] {
            let mut app = rig(23);
            // Union-exec (gap 4): the dispatcher reads the STORED
            // `Main.waterStyle` — the rig pins the static, no recompute.
            app.water_style = 0;
            let mult = if style == 5 { 0.05 } else { 0.1 }; // 5 puffs each
            teleport_effect(&mut app, rect(), style, 0, mult, Vec2::ZERO, KEY);
            let slots = live(&app);
            assert_eq!(slots.len(), 5);
            let ty = if style == 5 {
                tr_core::particles::dust_water(app.water_style)
            } else {
                35
            };
            let mut rng = UnifiedRandom::new(23);
            for d in &slots {
                assert_eq!(d.dust_type, ty);
                let (p, mut v, _) = default_block(&mut rng, rect().top_left(), 20, 66);
                // The 33/35 per-type tweaks (`Dust.cs:214-252`, no draws).
                if ty == 33 {
                    v *= 0.5;
                    v.y += 1.0;
                } else if ty == 35 {
                    v *= 0.1;
                    v.y = -0.5;
                }
                assert_eq!(d.position, p);
                v.y *= 0.0;
                v.y -= 3.5;
                v.x *= 1.5;
                assert_eq!(d.velocity, v);
                assert_eq!(d.scale, 0.8);
                assert_eq!(d.alpha, 130);
                assert!(d.no_gravity);
                assert_eq!(d.fade_in, 1.2);
            }
            // The AFTER-loop cue: one pitch draw on the same stream.
            if style == 5 {
                let _ = rng.next_range(-10, 11); // splash_case19 style 0
            } else {
                item_pitch(&mut rng); // Item8
            }
            assert_eq!(stream_of(&mut app), rng);
        }
    }

    /// The batch boundary, INVERTED at the union (research/116 §3): only
    /// styles 6/8/12 have no C# arm and must draw NOTHING; the g12/g61
    /// styles 9/10/11/13 now FIRE through the dispatcher (the Queen-Slime
    /// hook's 10 and the Shellphone/no-space 11 are reachable callers).
    #[test]
    fn w4_g11_the_no_arm_styles_stay_untouched() {
        for style in [6, 8, 12] {
            let mut app = rig(3);
            teleport_effect(&mut app, rect(), style, 0, 1.0, Vec2::ZERO, KEY);
            assert!(live(&app).is_empty(), "style {style} drew dust");
            assert_eq!(stream_of(&mut app), UnifiedRandom::new(3));
        }
        // The union arms fire: 9 (pylon), 10 (shimmer), 11 (perfect
        // burst), 13 (frog sparkle) all draw dust + advance the stream.
        for style in [9, 10, 11, 13] {
            let mut app = rig(3);
            teleport_effect(&mut app, rect(), style, 0, 1.0, Vec2::new(500.0, 300.0), KEY);
            assert!(!live(&app).is_empty(), "style {style} drew no dust");
            assert_ne!(stream_of(&mut app), UnifiedRandom::new(3));
        }
    }

    /// The sentinel contract: a paused spawn ctx refuses every `NewDust`
    /// (no internals draws), but the post-spawn writes still land on
    /// `Main.dust[6000]` and the explicit scale rolls still ride the
    /// stream (the C# writes the sentinel blindly).
    #[test]
    fn w4_g11_a_refused_spawn_still_writes_the_sentinel() {
        let mut app = rig(5);
        app.game_paused = true;
        teleport_effect(&mut app, rect(), 0, 0, 0.05, Vec2::ZERO, KEY); // 8 puffs
        assert!(live(&app).is_empty());
        let mut rng = UnifiedRandom::new(5);
        item_pitch(&mut rng);
        let mut last_scale = 0.0;
        for j in 0..8 {
            let mut scale = rng.next_range(20, 70) as f32 * 0.01;
            if j < 10 {
                scale += 0.25;
            }
            if j < 5 {
                scale += 0.25;
            }
            last_scale = scale;
        }
        assert_eq!(app.dust.slot(DUST_SENTINEL).scale, last_scale);
        assert_eq!(stream_of(&mut app), rng);
    }

    // --- snowing (site 1) -------------------------------------------------

    /// The snowing rig: a big open world, the view parked far inside it so
    /// every rolled spawn lands on an open tile, the global stream pinned.
    fn snow_rig(seed: i32) -> (App, SnowingEnv) {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = World::new("snow".into(), 512, 512);
        app.gfx_quality.max_dust_to_draw = 6000;
        app.camera = Vec2::new(4096.0 + 640.0, 4096.0 + 360.0);
        *app.rngs.global() = UnifiedRandom::new(seed);
        let env = SnowingEnv {
            remix_world: false,
            game_paused: false,
            snow_tile_count: 6000,
            surface_atmospherics: true,
            cloud_alpha: 0.0,
            scaled_size: Vec2::new(1280.0, 720.0),
            scaled_position: Vec2::new(4096.0, 4096.0),
            gfx_quality: 0.0,
            deerclops_near: false,
            player_vy: 5.0,
            raining: false,
            wind_speed_current: 0.3,
        };
        (app, env)
    }

    /// The four early-return gates draw NOTHING (`:13034-13043`).
    #[test]
    fn w4_g11_snowing_gates_before_any_draw() {
        for gate in 0..4 {
            let (mut app, mut env) = snow_rig(1);
            match gate {
                0 => env.remix_world = true,
                1 => env.game_paused = true,
                2 => env.snow_tile_count = 0,
                _ => env.surface_atmospherics = false,
            }
            let ctx = app.spawn_ctx();
            snowing(&env, app.rngs.global(), &ctx, &app.world, &mut app.dust);
            assert!(live(&app).is_empty(), "gate {gate} drew dust");
            assert_eq!(app.rngs.global().clone(), UnifiedRandom::new(1));
        }
    }

    /// The `NewDust` off-screen cull (`Dust.cs:97-101`) for the snow rigs:
    /// screen_position (4096,4096), screen 1280×720, dCount 0 → margin 400,
    /// rect (3696,3696,2080,1520); XNA `Intersects` is strict `<`.
    fn snow_cull_passes(x: i32, y: i32) -> bool {
        3696 < x + 10 && x < 3696 + 2080 && 3696 < y + 10 && y < 3696 + 1520
    }

    /// Site 1, the dry-wind arm: cloudAlpha 0 → ONE iteration, the full
    /// snow count → the `NextFloat` gate always passes; the mirror walks
    /// every draw in C# order (the two `Next(5)` edge gates roll in `else
    /// if` order, the y-spread only off-screen) and pins the flake's
    /// position/velocity/scale — including the double `cloudAlpha` scale
    /// bump (0 here) and the `windSpeedCurrent * 15` dry tail. A culled
    /// spawn draws NO internals but the post-writes still roll onto the
    /// sentinel (`Dust.cs:79` — the C# writes `Main.dust[6000]` blindly).
    #[test]
    fn w4_g11_snowing_mirrors_the_dry_arm_draw_sequence() {
        let (mut app, env) = snow_rig(0x5EED);
        let ctx = app.spawn_ctx();
        snowing(&env, app.rngs.global(), &ctx, &app.world, &mut app.dust);
        let mut rng = UnifiedRandom::new(0x5EED);
        let _ = rng.next_float(); // the spawn gate (`:13061`)
        let mut num4 = rng.next_max(1280 + 1500) - 750;
        let mut num5 = 4096 - rng.next_max(50);
        num5 -= 5; // player_vy > 0 (`:13067-13070`)
        if rng.next_max(5) == 0 {
            num4 = rng.next_max(500) - 500;
        } else if rng.next_max(5) == 0 {
            num4 = rng.next_max(500) + 1280;
        }
        if num4 < 0 || num4 as f32 > 1280.0 {
            num5 += rng.next_max((720.0f64 * 0.8) as i32) + (720.0f64 * 0.1) as i32;
        }
        num4 += 4096;
        let passes = snow_cull_passes(num4, num5);
        let mut v = Vec2::ZERO;
        let mut s = 0.0;
        if passes {
            let (p, vel, scale) =
                default_block(&mut rng, Vec2::new(num4 as f32, num5 as f32), 10, 10);
            let d = live(&app)[0];
            assert_eq!(d.dust_type, 76);
            assert_eq!(d.position, p);
            v = vel;
            s = scale;
        } else {
            assert!(live(&app).is_empty(), "the cull refuses the spawn");
        }
        // `:13089-13091` — scale += 0 (cloudAlpha 0), the vy re-roll.
        v.y = 3.0 + rng.next_max(30) as f32 * 0.1;
        v.y *= s;
        // `:13094-13095` — the dry wind arm.
        v.x = 0.3 + rng.next_range(-10, 10) as f32 * 0.1;
        v.x += 0.3 * 15.0;
        // `:13102-13108` — the shared tail at cloudAlpha 0.
        v.y *= 1.0;
        if passes {
            let d = live(&app)[0];
            assert_eq!(d.velocity, v);
            assert_eq!(d.scale, s);
        } else {
            // The sentinel took the writes; its scale is the pool default
            // (0), so only the scale-independent x write is pinned.
            assert_eq!(app.dust.slot(DUST_SENTINEL).velocity.x, v.x);
        }
        assert_eq!(app.rngs.global().clone(), rng);
    }

    /// Site 1, the storm arm (`Main.raining`): the sqrt/sign wind formula
    /// (negative wind → negative carry), the vy ×0.5, the cloudAlpha
    /// double scale bump + the ×(1+0.3a)/×(1+0.5a) tails, and the
    /// Deerclops −0.5 scale cut. cloudAlpha 0.5 → num3 = 26 iterations;
    /// the mirror walks them all with a LOCAL budget counter (the pool's
    /// live count grows as the port spawns — the mirror must read it at
    /// the same moment). The deerclops gate is 0.05/iteration, so the
    /// seed sweep guarantees at least one spawn.
    #[test]
    fn w4_g11_snowing_mirrors_the_storm_arm_with_deerclops() {
        let mut any_spawn = false;
        for seed in [99i32, 7, 0x5EED, 12345] {
            let (mut app, mut env) = snow_rig(seed);
            env.cloud_alpha = 0.5;
            env.raining = true;
            env.wind_speed_current = -0.4;
            env.deerclops_near = true;
            let ctx = app.spawn_ctx();
            snowing(&env, app.rngs.global(), &ctx, &app.world, &mut app.dust);
            let mut rng = UnifiedRandom::new(seed);
            // num = (6000/6000)^(4-1.5) / 20 = 0.05; num2 = 156*2/3 = 104.
            let num = 1.0f32 / 20.0;
            let num2 = (((500.0 * (1280.0 / 4096.0)) as i32 as f32) * 2.0) as i32 / 3;
            let mut spawned = 0usize;
            let mut i = 0;
            while (i as f32) < 1.0 + 50.0 * 0.5 {
                i += 1;
                if !(spawned as f32 < num2 as f32 * 0.5 + num2 as f32 * 0.1) {
                    break;
                }
                if !(rng.next_float() < num) {
                    continue;
                }
                let mut num4 = rng.next_max(1280 + 1500) - 750;
                let mut num5 = 4096 - rng.next_max(50) - 5;
                if rng.next_max(5) == 0 {
                    num4 = rng.next_max(500) - 500;
                } else if rng.next_max(5) == 0 {
                    num4 = rng.next_max(500) + 1280;
                }
                if num4 < 0 || num4 as f32 > 1280.0 {
                    num5 += rng.next_max((720.0f64 * 0.8) as i32) + (720.0f64 * 0.1) as i32;
                }
                num4 += 4096;
                let mut v = Vec2::ZERO;
                let mut s = 0.0;
                let mut p = Vec2::ZERO;
                if snow_cull_passes(num4, num5) {
                    let (pos, vel, scale) =
                        default_block(&mut rng, Vec2::new(num4 as f32, num5 as f32), 10, 10);
                    p = pos;
                    v = vel;
                    s = scale;
                    let d = app.dust.slot(spawned);
                    assert!(d.active);
                    assert_eq!(d.dust_type, 76);
                    assert_eq!(d.position, p);
                    spawned += 1;
                }
                s += 0.5 * 0.2;
                v.y = 3.0 + rng.next_max(30) as f32 * 0.1;
                v.y *= s;
                v.x = (0.4f64.sqrt() as f32) * -1.0 * (0.5 + 0.5) * 10.0
                    + rng.next_float() * 0.2
                    - 0.1;
                v.y *= 0.5;
                v.y *= 1.0 + 0.3 * 0.5;
                s += 0.5 * 0.2;
                s -= 0.5; // the Deerclops cut
                v *= 1.0 + 0.5 * 0.5;
                if snow_cull_passes(num4, num5) {
                    let d = app.dust.slot(spawned - 1);
                    assert_eq!(d.velocity, v);
                    assert_eq!(d.scale, s);
                }
            }
            assert_eq!(live(&app).len(), spawned);
            assert_eq!(app.rngs.global().clone(), rng);
            any_spawn |= spawned > 0;
        }
        assert!(any_spawn, "the seed sweep must pass the 0.05 gate");
    }

    /// The budget gate BREAKS before any draw: with the Deerclops throttle
    /// at cloudAlpha 0, num2 = 156/3 = 52 → cap 52×0.5 + 52×0.1 = 31.2; a
    /// pool already over it spends nothing (the `Main.snowDust` stand-in
    /// reads the live 76 population, the `SandStormCount` pattern).
    #[test]
    fn w4_g11_snowing_breaks_on_the_live_76_budget() {
        let (mut app, mut env) = snow_rig(1);
        env.deerclops_near = true; // num2 = 156/3 = 52 → cap 31.2
        for slot in app.dust.slots_mut()[..40].iter_mut() {
            slot.active = true;
            slot.dust_type = 76;
        }
        let ctx = app.spawn_ctx();
        snowing(&env, app.rngs.global(), &ctx, &app.world, &mut app.dust);
        assert_eq!(crate::minecart::live_type(&app.dust, 76), 40);
        assert_eq!(app.rngs.global().clone(), UnifiedRandom::new(1));
    }

    // --- W4-g11 wiring: the Teleport replays -------------------------------

    /// The player rig: the teleport rig + a player parked on the OLD box
    /// (300, 600), the listener on its centre so both effect cues pass the
    /// attenuation gate.
    fn player_rig(seed: i32) -> App {
        let mut app = rig(seed);
        let mut p = crate::player::Player::spawn(&app.world);
        p.x = 300.0;
        p.y = 600.0;
        app.player = Some(p);
        app
    }

    /// The `Player.Teleport` replay (`Player.cs:37414-37496`): the Entry
    /// effect at the OLD box BEFORE the move, the Exit at the NEW box AFTER,
    /// both on the caller's stream in C# order — style 1's two Item8 cues +
    /// two 8-puff bursts (mult 0.05), the full stream mirror walks them.
    #[test]
    fn w4_g11_player_teleport_replays_entry_then_exit() {
        let mut app = player_rig(0x5EED);
        player_teleport(
            &mut app,
            PlayerTeleportReq {
                old_x: 300.0,
                old_y: 600.0,
                new_x: 500.0,
                new_y: 700.0,
                style: 1,
                extra_info2: 0,
                extra_info: 0,
                mult: 0.05,
                key: KEY,
            },
        );
        // The move + the grants (`:37446`/`:37490-37491`).
        let p = app.player.as_ref().unwrap();
        assert_eq!((p.x, p.y), (500.0, 700.0));
        assert_eq!(p.teleport_time, 1.0);
        assert_eq!(p.teleport_style_id, 1);
        // 8 puffs at the OLD box + 8 at the NEW (`20*42/5*0.05 = 8.4 → 8`).
        let slots = live(&app);
        assert_eq!(slots.len(), 16);
        let mut rng = UnifiedRandom::new(0x5EED);
        for (burst, origin) in [(0usize, Vec2::new(300.0, 600.0)), (8, Vec2::new(500.0, 700.0))] {
            item_pitch(&mut rng); // the Item8 cue precedes each burst
            for j in 0..8 {
                let d = slots[burst + j];
                assert_eq!(d.dust_type, 164);
                let (pos, v, _) = default_block(&mut rng, origin, 20, 42);
                assert_eq!(d.position, pos);
                assert_eq!(d.velocity, v);
                let mut scale = rng.next_range(20, 70) as f32 * 0.01;
                if j < 10 {
                    scale += 0.25;
                }
                if j < 5 {
                    scale += 0.25;
                }
                assert_eq!(d.scale, scale);
            }
        }
        assert_eq!(stream_of(&mut app), rng);
    }

    /// The queue contract: `teleport_style` captures the OLD box + the
    /// `Clamp(1 - teleportTime*0.99, 0.01, 1)` mult at the call moment and
    /// moves the player; the take drains once (the `pending_force_clear`
    /// convention). Union-exec (gap 6): the grants moved to the replay —
    /// the inline path leaves `teleportTime`/`teleportStyle` untouched
    /// until `player_teleport` writes them at the C# position.
    #[test]
    fn w4_g11_teleport_style_queues_the_effect_pair() {
        let mut app = player_rig(3);
        let p = app.player.as_mut().unwrap();
        p.teleport_time = 0.5; // mult = 1 - 0.495 = 0.505
        p.teleport_style(500.0, 700.0, 1);
        assert_eq!((p.x, p.y), (500.0, 700.0), "the move ran inline");
        assert_eq!(p.teleport_time, 0.5, "the grant is the replay's (gap 6)");
        assert_eq!(p.teleport_style_id, 0);
        let req = p.take_teleport_effect().expect("the pair queued");
        assert_eq!((req.0, req.1, req.2, req.3), (300.0, 600.0, 500.0, 700.0));
        assert_eq!((req.4, req.5, req.6), (1, 0, 0));
        assert!((req.7 - 0.505).abs() < 1e-6, "mult = 1 - 0.5*0.99: {}", req.7);
        assert!(p.take_teleport_effect().is_none(), "drains once");
        // A second teleport AFTER the replay's grant clamps the mult at
        // 0.01 (teleportTime 1 — the replay owns the write).
        p.teleport_time = 1.0;
        p.teleport_style(600.0, 800.0, 2);
        let req = p.take_teleport_effect().unwrap();
        assert_eq!(req.7, 0.01);
    }

    /// Union-exec gap 3: style 9 latches `lastTeleportPylonStyleUsed` and
    /// the Entry effect's `extraInfo2` reads it back (`Player.cs:37437-
    /// 37441`); the pylon tile style reaches the dust colour table.
    #[test]
    fn w4_g11_style_9_carries_the_pylon_style() {
        let mut app = player_rig(3);
        let p = app.player.as_mut().unwrap();
        p.teleport_style_with_extra(500.0, 700.0, 9, 6);
        assert_eq!(p.last_teleport_pylon_style_used, 6);
        let req = p.take_teleport_effect().unwrap();
        assert_eq!(req.4, 9);
        assert_eq!(req.5, 6, "extraInfo2 = the latched pylon style");
    }

    /// Union-exec gap 5: the style-8 head arm plays Item6 TWICE (old then
    /// new, `Player.cs:37450-37453`) — two case-2 pitch draws on the pass
    /// stream, no dust (style 8 has no TeleportEffect arm).
    #[test]
    fn w4_g11_style_8_plays_the_double_cue() {
        let mut app = player_rig(0x5EED);
        player_teleport(
            &mut app,
            PlayerTeleportReq {
                old_x: 300.0,
                old_y: 600.0,
                new_x: 320.0,
                new_y: 610.0,
                style: 8,
                extra_info2: 0,
                extra_info: 0,
                mult: 1.0,
                key: KEY,
            },
        );
        assert!(live(&app).is_empty(), "style 8 has no dust arm");
        let mut rng = UnifiedRandom::new(0x5EED);
        item_pitch(&mut rng); // Item6 at the OLD position
        item_pitch(&mut rng); // Item6 at newPos
        assert_eq!(stream_of(&mut app), rng);
    }

    /// Union-exec gap 5: the replay owns the myPlayer camera transition
    /// for BOTH callers — style 1 arms `lerpSpeedupDelay = 10`
    /// (`Player.cs:37459-37462`), every other style 0.
    #[test]
    fn w4_g11_the_replay_arms_the_camera_delay() {
        for (style, delay) in [(0, 0), (1, 10)] {
            let mut app = player_rig(3);
            player_teleport(
                &mut app,
                PlayerTeleportReq {
                    old_x: 300.0,
                    old_y: 600.0,
                    new_x: 500.0,
                    new_y: 700.0,
                    style,
                    extra_info2: 0,
                    extra_info: 0,
                    mult: 0.05,
                    key: KEY,
                },
            );
            assert_eq!(app.camera_state.camera_lerp, 0.1);
            assert_eq!(app.camera_state.camera_lerp_time_toggle, delay);
        }
    }

    /// g61 `w4_g61_npc_teleport_two_sided_burst_and_latch` (union-adapted
    /// to the App-level replay + the g11 `Npc` fields): `NPC.cs:81035-
    /// 81058` — Entry at the old rect (OLD portal colour), Exit at the
    /// new (NEW colour), the pre-move mult, the style-4 index latch
    /// between the calls, teleportTime/Style written last.
    #[test]
    fn w4_g11_npc_teleport_style4_two_sided_burst_and_latch() {
        let mut app = rig(47);
        let mut n = crate::npc::Npc::new(crate::npc::NpcKind::Creeper, 100.0, 100.0, 1, 0.0);
        n.size_override = Some((10.0, 10.0)); // 100/5 = 20 per side
        n.hp = n.max_hp;
        n.teleport_time = 0.5; // mid-teleport: mult 0.3
        n.last_portal_color_index = 2; // the OLD index
        app.npcs.push(n);
        // The listener on the mid-point so both cue gates pass.
        app.audio.set_listener(Listener {
            camera_center: (200.0, 150.0),
            sound_volume: 1.0,
            ambient_volume: 1.0,
            quiet_ambient_sounds: false,
            game_menu: false,
            generating_or_loading: false,
            server: false,
        });
        npc_teleport(
            &mut app,
            NpcTeleportReq { index: 0, new_x: 300.0, new_y: 200.0, style: 4, extra_info: 5, key: KEY },
        );
        assert_eq!((app.npcs[0].x, app.npcs[0].y), (300.0, 200.0));
        assert_eq!(app.npcs[0].teleport_time, 1.0);
        assert_eq!(app.npcs[0].teleport_style_id, 4);
        assert_eq!(app.npcs[0].last_portal_color_index, 5, "the NEW index latched");
        // 20×0.3 = 6 per side → 12 dust; Entry = OLD colour (index 2 →
        // portal 0), Exit = NEW (index 5 → portal 1).
        let slots = live(&app);
        assert_eq!(slots.len(), 12);
        let old_c = crate::teleport_dust::portal_color(2);
        let new_c = crate::teleport_dust::portal_color(5);
        for d in &slots[..6] {
            assert_eq!(d.color.r, old_c.r);
            assert!(d.position.x < 200.0);
        }
        for d in &slots[6..] {
            assert_eq!(d.color.r, new_c.r);
            assert!(d.position.x >= 300.0);
        }
    }

    /// The `NPC.Teleport` replay (`NPC.cs:81035-81058`): the Entry at the OLD
    /// box, the move, the Exit at the NEW box, the grants — style 0's two
    /// Item6 cues + two bursts. The NPC box is its `size()` (the Nurse 20×42
    /// stand-in is the rig's; the mirror reads the live size).
    #[test]
    fn w4_g11_npc_teleport_replays_entry_then_exit() {
        let mut app = rig(0xC0FFEE);
        let mut n = crate::npc::Npc::new(crate::npc::NpcKind::Nurse, 300.0, 600.0, 1, 0.0);
        n.hp = n.max_hp;
        app.npcs.push(n);
        let (w, h) = app.npcs[0].size();
        let (wi, hi) = (w as i32, h as i32);
        npc_teleport(
            &mut app,
            NpcTeleportReq {
                index: 0,
                new_x: 500.0,
                new_y: 700.0,
                style: 0,
                extra_info: 0,
                key: KEY,
            },
        );
        assert_eq!((app.npcs[0].x, app.npcs[0].y), (500.0, 700.0));
        assert_eq!(app.npcs[0].teleport_time, 1.0);
        assert_eq!(app.npcs[0].teleport_style_id, 0);
        // mult 1 (teleportTime was 0): `w*h/5` puffs per burst.
        let per_burst = (wi * hi / 5) as usize;
        let slots = live(&app);
        assert_eq!(slots.len(), per_burst * 2);
        let mut rng = UnifiedRandom::new(0xC0FFEE);
        for (burst, origin) in
            [(0usize, Vec2::new(300.0, 600.0)), (per_burst, Vec2::new(500.0, 700.0))]
        {
            item_pitch(&mut rng); // the Item6 cue precedes each burst
            for j in 0..per_burst {
                let d = slots[burst + j];
                assert_eq!(d.dust_type, 159);
                let (pos, v, _) = default_block(&mut rng, origin, wi, hi);
                assert_eq!(d.position, pos);
                assert_eq!(d.velocity, v);
                let mut scale = rng.next_range(20, 70) as f32 * 0.01;
                if j < 10 {
                    scale += 0.25;
                }
                if j < 5 {
                    scale += 0.25;
                }
                assert_eq!(d.scale, scale);
            }
        }
        assert_eq!(stream_of(&mut app), rng);
        // A re-teleport inside the window reads mult 0.3 (`:81042`).
        let before = app.rngs.stream(KEY).clone();
        npc_teleport(
            &mut app,
            NpcTeleportReq { index: 0, new_x: 500.0, new_y: 700.0, style: 0, extra_info: 0, key: KEY },
        );
        let expected = ((wi * hi / 5) as f32 * 0.3) as usize;
        assert_eq!(live(&app).len() - per_burst * 2, expected * 2);
        assert_ne!(app.rngs.stream(KEY).clone(), before);
    }

    /// The caller-key contract: the wiring pads replay on `UpdateWorldGen`,
    /// the player tick on `UpdatePlayers` — the OTHER streams stay untouched
    /// (one RNG slot per C# pass, S1-A-10).
    #[test]
    fn w4_g11_the_replay_rides_only_the_callers_stream() {
        let mut app = player_rig(9);
        // The global baseline BEFORE any read (the rig's spawn_ctx does not
        // draw; the replay must not touch the bare stream either).
        let global_before = app.rngs.global().clone();
        let untouched: Vec<_> = crate::app::RngKey::ALL
            .iter()
            .copied()
            .filter(|k| *k != crate::app::RngKey::UpdateWorldGen)
            .map(|k| (k, app.rngs.stream(k).clone()))
            .collect();
        player_teleport(
            &mut app,
            PlayerTeleportReq {
                old_x: 300.0,
                old_y: 600.0,
                new_x: 500.0,
                new_y: 700.0,
                style: 1,
                extra_info2: 0,
                extra_info: 0,
                mult: 0.05,
                key: crate::app::RngKey::UpdateWorldGen,
            },
        );
        assert!(!live(&app).is_empty(), "the replay drew on UpdateWorldGen");
        for (k, before) in untouched {
            assert_eq!(app.rngs.stream(k).clone(), before, "{k:?} moved");
        }
        assert_eq!(app.rngs.global().clone(), global_before);
    }

    // --- the union witnesses (research/116 §3: g12's 4 + d54's 8, adapted
    // to the App dispatcher; the draw sequences are identical per §1) ---

    /// The six internals on a mirror stream, value-consuming (the g12
    /// `mirror_internals` / d54 `block` shape — the <5 clamp makes a
    /// 0/2-box roll `Next(1)`).
    fn mirror_block(rng: &mut UnifiedRandom, w: i32, h: i32) {
        let _ = rng.next_max(w.max(5) - 4);
        let _ = rng.next_max(h.max(5) - 4);
        let _ = rng.next_range(-20, 21);
        let _ = rng.next_range(-20, 21);
        let _ = rng.next_max(3);
        let _ = rng.next_range(-20, 21);
    }

    /// The permissive union rig: the whole 64×128 world on-screen (no
    /// cull), the full budget, the listener on the effect box centre so
    /// the cue gates pass — the g12 `ctx()`/`audio_at` convention.
    fn union_rig(seed: i32) -> App {
        let mut app = rig(seed);
        app.camera = Vec2::new(512.0, 1024.0);
        app.audio.set_listener(Listener {
            camera_center: (310.0, 621.0),
            sound_volume: 1.0,
            ambient_volume: 1.0,
            quiet_ambient_sounds: false,
            game_menu: false,
            generating_or_loading: false,
            server: false,
        });
        app
    }

    /// g12 `w4_g12_case10_mirrors_the_sparkle_lane_and_clone`
    /// (union-adapted): the case-10 draw sequence mirrors exactly — per
    /// dust the three argument floats, the internals, the dead
    /// `Next(20,70)`, the gated pull float (n < num7*0.8), and the
    /// clone's six internals; the final stream states agree and the
    /// records pin the local-overwrite + white-0.65×-clone shape.
    #[test]
    fn w4_g11u_g12_case10_mirrors_the_sparkle_lane_and_clone() {
        let mut app = union_rig(7);
        let other = Vec2::new(900.0, 300.0);
        teleport_effect(&mut app, rect(), 10, 0, 1.0, other, KEY);
        // The mirror: 60 iterations + the trailing case-2 pitch draw.
        let mut mirror = UnifiedRandom::new(7);
        let mut locals = Vec::new();
        let num7 = 60;
        for n in 0..num7 {
            let fi = 0.4 + mirror.next_float();
            let sc = 0.4 + mirror.next_float();
            let _hue = mirror.next_float();
            mirror_block(&mut mirror, 50, 72); // inflated 20+30 × 42+30
            let _dead = mirror.next_range(20, 70);
            if (n as f32) < (num7 as f32) * 0.8 {
                let _pull = mirror.next_float();
            }
            mirror_block(&mut mirror, 0, 0); // CloneDust's NewDust
            locals.push((fi, sc));
        }
        let _pitch = mirror.next_range(-6, 7); // the Item8 case-2 draw
        assert_eq!(stream_of(&mut app), mirror, "the case-10 draw sequence is exact");
        let slots = live(&app);
        assert_eq!(slots.len(), 120, "60 originals + 60 clones");
        for (k, &(fi, sc)) in locals.iter().enumerate() {
            let o = slots[2 * k];
            let cl = slots[2 * k + 1];
            assert_eq!(o.dust_type, 267);
            assert_eq!(o.alpha, 127);
            assert!(o.no_gravity && o.no_light);
            assert_eq!(o.fade_in, fi, "the local fadeIn overwrites");
            assert_eq!(o.scale, sc, "the dead Next(20,70) ladder is overwritten");
            assert_eq!(cl.scale, sc * 0.65);
            assert_eq!(cl.fade_in, fi * 0.65);
            assert_eq!(cl.color, Color::new(255, 255, 255, 255));
            assert_eq!(cl.position, o.position, "CloneDust copies the position");
        }
    }

    /// g12 `w4_g12_case10_mult_truncates_and_refusal_burns_arguments`
    /// (union-adapted): the count truncates `(int)(60f * mult)` and the
    /// menu-guard refusal still burns the argument draws — the sentinel
    /// takes the post writes and NO clone fires (idx == 6000).
    #[test]
    fn w4_g11u_g12_case10_mult_truncates_and_refusal_burns_arguments() {
        let mut app = union_rig(9);
        app.game_paused = true; // the spawn ctx refuses every slot
        teleport_effect(&mut app, rect(), 10, 0, 0.3, Vec2::ZERO, KEY);
        assert!(live(&app).is_empty(), "the guard refuses every slot");
        // 18 iterations × (3 argument floats + Next(20,70) + the gated
        // pull float for n < 14.4) — no internals, no clone draws.
        let mut mirror = UnifiedRandom::new(9);
        for n in 0..18 {
            let _ = mirror.next_float();
            let _ = mirror.next_float();
            let _ = mirror.next_float();
            let _ = mirror.next_range(20, 70);
            if (n as f32) < 18.0 * 0.8 {
                let _ = mirror.next_float();
            }
        }
        let _pitch = mirror.next_range(-6, 7);
        assert_eq!(stream_of(&mut app), mirror, "refused spawns still burn the argument draws");
        // The sentinel took the last local writes (the C#'s Main.dust[6000]).
        let s = app.dust.slot(DUST_SENTINEL);
        assert!(s.no_gravity && s.no_light);
    }

    /// g12 `w4_g12_case11_mirrors_the_perfect_burst` (union-adapted):
    /// the case-11 sequence — Next(4) colour pick, the position pair,
    /// the Perfect internals, the scale float — ×50, no sound; the
    /// colours/scale/fadeIn/velocity-doubling pin.
    #[test]
    fn w4_g11u_g12_case11_mirrors_the_perfect_burst() {
        let mut app = union_rig(13);
        teleport_effect(&mut app, rect(), 11, 0, 1.0, Vec2::ZERO, KEY);
        let mut mirror = UnifiedRandom::new(13);
        let mut picks = Vec::new();
        for _ in 0..50 {
            let pick = mirror.next_max(4);
            let _px = mirror.next_float();
            let _py = mirror.next_float();
            mirror_block(&mut mirror, 0, 0);
            let sc = 0.8 + mirror.next_float() * 0.6;
            picks.push((pick, sc));
        }
        assert_eq!(stream_of(&mut app), mirror, "the case-11 draw sequence is exact");
        let slots = live(&app);
        assert_eq!(slots.len(), 50);
        for (k, &(pick, sc)) in picks.iter().enumerate() {
            let d = slots[k];
            assert_eq!(d.dust_type, 267);
            assert!(d.no_gravity);
            assert_eq!(d.fade_in, 0.5);
            assert_eq!(d.scale, sc);
            let want = match pick {
                0 | 1 => Color::new(100, 255, 100, 255),
                2 => Color::new(255, 255, 0, 255),
                _ => Color::WHITE,
            };
            assert_eq!(d.color, want);
            assert!(d.position.x >= 300.0 && d.position.x < 320.0);
            assert!(d.position.y >= 600.0 && d.position.y < 642.0);
        }
    }

    /// g12 `w4_g12_case13_sound_first_then_the_smoke_ladder`
    /// (union-adapted): the case-13 Item8 pitch draw precedes the 21
    /// smoke dusts; each dust draws the position pair + the Next(1,3)
    /// scale argument BEFORE the internals; the velocity ladder pins
    /// (×1.75, extra ×1.5 on i%3==0) and the two flags.
    #[test]
    fn w4_g11u_g12_case13_sound_first_then_the_smoke_ladder() {
        let mut app = union_rig(17);
        teleport_effect(&mut app, rect(), 13, 0, 1.0, Vec2::ZERO, KEY);
        let mut mirror = UnifiedRandom::new(17);
        let _pitch = mirror.next_range(-6, 7); // the Item8 cue FIRST
        let mut scales = Vec::new();
        for _ in 0..21 {
            let _px = mirror.next_float();
            let _py = mirror.next_float();
            let sc = mirror.next_range(1, 3);
            mirror_block(&mut mirror, 2, 2);
            scales.push(sc);
        }
        assert_eq!(stream_of(&mut app), mirror, "the case-13 draw sequence is exact");
        let slots = live(&app);
        assert_eq!(slots.len(), 21);
        let ctx = app.spawn_ctx();
        for (i, &sc) in scales.iter().enumerate() {
            let d = slots[i];
            assert_eq!(d.dust_type, 27);
            assert_eq!(d.alpha, 100);
            assert!(d.no_gravity && d.no_light_emittance);
            // Re-derive via a mirror pool (the rolled velocity jitter is
            // in [-2,2]; the ladder is ×1.75, ×1.5 more on i%3==0).
            let mut mp = DustPool::new();
            let mut mr = UnifiedRandom::new(17);
            let _ = mr.next_range(-6, 7);
            for _ in 0..i {
                let _ = mr.next_float();
                let _ = mr.next_float();
                let _ = mr.next_range(1, 3);
                mirror_block(&mut mr, 2, 2);
            }
            let px = 300.0 + mr.next_float() * 20.0;
            let py = 600.0 + mr.next_float() * 42.0;
            let sarg = mr.next_range(1, 3) as f32;
            let mi = mp.new_dust(&mut mr, &ctx, &app.world, Vec2::new(px, py), 2, 2, 27, 0.0, 0.0, 100, Color::default(), sarg);
            let mut want = mp.slot(mi).velocity * 1.75;
            if i % 3 == 0 {
                want *= 1.5;
            }
            assert_eq!(d.velocity, want);
            assert_eq!(d.scale, mp.slot(mi).scale, "the Next(1,3) scale argument");
            assert_eq!(sc, sarg, "the two mirrors agree on the scale roll");
        }
    }

    /// d54 `w4_d54_teleport_style0_scale_ladder` (union-adapted):
    /// `Main.cs:13240-13253` — style 0's full-count scale ladder at
    /// mult 1 (80 puffs on a 20×20 rect).
    #[test]
    fn w4_g11u_d54_style0_scale_ladder() {
        let mut app = union_rig(54);
        let r = EffectRect { x: 300, y: 600, width: 20, height: 20 };
        teleport_effect(&mut app, r, 0, 0, 1.0, Vec2::ZERO, KEY);
        let mut probe = UnifiedRandom::new(54);
        item_pitch(&mut probe); // the union plays the Item6 cue first
        for j in 0..80 {
            let _ = default_block(&mut probe, r.top_left(), 20, 20);
            let roll = probe.next_range(20, 70) as f32 * 0.01;
            let mut want = roll;
            if j < 10 {
                want += 0.25;
            }
            if j < 5 {
                want += 0.25;
            }
            let d = app.dust.slot(j as usize);
            assert_eq!(d.dust_type, 159);
            assert!((d.scale - want).abs() < 1e-6, "j {j}");
        }
        assert_eq!(stream_of(&mut app), probe);
    }

    /// d54 `w4_d54_teleport_style3_draws_match_the_oracle` (union-adapted):
    /// the full post census at mult 1 — 5 Next(3) damps, 2 Next(3) doubles,
    /// the Next(2) overwrite arm (2 more rolls on a pass), the final ×0.8.
    #[test]
    fn w4_g11u_d54_style3_draws_match_the_oracle() {
        for seed in [54, 7] {
            let mut app = union_rig(seed);
            teleport_effect(&mut app, rect(), 3, 0, 1.0, Vec2::ZERO, KEY);
            let mut probe = UnifiedRandom::new(seed);
            item_pitch(&mut probe);
            for _ in 0..50 {
                let _ = default_block(&mut probe, rect().top_left(), 20, 42);
                for _ in 0..5 {
                    probe.next_max(3);
                }
                probe.next_max(3);
                probe.next_max(3);
                if probe.next_max(2) == 0 {
                    probe.next_range(75, 100);
                    probe.next_range(25, 75);
                }
            }
            assert_eq!(stream_of(&mut app), probe, "style3 seed {seed}");
        }
    }

    /// d54 `w4_d54_teleport_styles5_7_writes_and_draws` (union-adapted):
    /// the dustWater/35 Direct puffs at mult 1 (box h+24), the overwrite
    /// chain leaves vy=-3.5, vx = spawn-vx ×1.5 (after the per-type
    /// tweak), scale 0.8, alpha 130, fadeIn 1.2, noGravity; the cue AFTER.
    #[test]
    fn w4_g11u_d54_styles5_7_writes_and_draws() {
        for (style, ty, count) in [(5, tr_core::particles::dust_water(0), 100), (7, 35, 50)] {
            let mut app = union_rig(54);
            app.water_style = 0;
            teleport_effect(&mut app, rect(), style, 0, 1.0, Vec2::ZERO, KEY);
            let mut probe = UnifiedRandom::new(54);
            for _ in 0..count {
                let _ = default_block(&mut probe, rect().top_left(), 20, 66);
            }
            // The AFTER-loop cue on the same stream.
            if style == 5 {
                let _ = probe.next_range(-10, 11);
            } else {
                item_pitch(&mut probe);
            }
            assert_eq!(stream_of(&mut app), probe, "style {style}");
            // Replay the first spawn's vx: the default block's roll,
            // then the type tweak (33: ×0.5; 35: ×0.1), then ×1.5.
            let mut rp = UnifiedRandom::new(54);
            rp.next_max(16);
            rp.next_max(62);
            let vx = rp.next_range(-20, 21) as f32 * 0.1;
            let tweak = if ty == 35 { 0.1 } else { 0.5 };
            let d = app.dust.slot(0);
            assert_eq!(d.dust_type, ty);
            assert!((d.velocity.y - -3.5).abs() < 1e-6);
            assert!((d.velocity.x - vx * tweak * 1.5).abs() < 1e-6);
            assert!((d.scale - 0.8).abs() < 1e-6);
            assert_eq!(d.alpha, 130);
            assert!((d.fade_in - 1.2).abs() < 1e-6);
            assert!(d.no_gravity);
        }
    }

    /// d54 `w4_d54_teleport_style10_draws_match_the_oracle` (union-adapted):
    /// style 10's per-iteration census at mult 1 — the two local
    /// NextFloats, the hue NextFloat, the Direct block, the Next(20,70)
    /// scale roll, the 80% push NextFloat, then the CloneDust block.
    #[test]
    fn w4_g11u_d54_style10_draws_match_the_oracle() {
        let mut app = union_rig(54);
        teleport_effect(&mut app, rect(), 10, 0, 1.0, Vec2::new(500.0, 700.0), KEY);
        let mut probe = UnifiedRandom::new(54);
        let count = 60;
        for n in 0..count {
            probe.next_float(); // fadeIn local
            probe.next_float(); // scale local
            probe.next_float(); // hue
            let _ = default_block(&mut probe, rect().top_left(), 50, 72); // inflated
            probe.next_range(20, 70);
            if (n as f32) < count as f32 * 0.8 {
                probe.next_float();
            }
            let _ = default_block(&mut probe, Vec2::ZERO, 0, 0); // CloneDust
        }
        item_pitch(&mut probe); // the Item8 AFTER the loop
        assert_eq!(stream_of(&mut app), probe);
    }

    /// d54 `w4_d54_teleport_style11_draws_match_the_oracle` (union-adapted):
    /// the Next(4) colour pick FIRST, then the two position NextFloats,
    /// then the Perfect block, then the scale NextFloat — all four
    /// colour arms seen across the 50 dust.
    #[test]
    fn w4_g11u_d54_style11_draws_match_the_oracle() {
        let mut app = union_rig(54);
        teleport_effect(&mut app, rect(), 11, 0, 1.0, Vec2::ZERO, KEY);
        let mut probe = UnifiedRandom::new(54);
        let mut saw = [false; 4];
        for _ in 0..50 {
            let pick = probe.next_max(4);
            saw[pick as usize] = true;
            probe.next_float();
            probe.next_float();
            let _ = default_block(&mut probe, Vec2::ZERO, 0, 0); // Perfect
            probe.next_float(); // scale
        }
        assert_eq!(stream_of(&mut app), probe);
        assert!(saw.iter().all(|&s| s), "all four colour arms");
        let d = app.dust.slot(0);
        assert_eq!(d.dust_type, 267);
        assert!(d.no_gravity);
        assert!((d.fade_in - 0.5).abs() < 1e-6);
    }

    /// d54 `w4_d54_teleport_style13_draws_match_the_oracle` (union-adapted):
    /// the two position NextFloats, the Next(1,3) scale argument, the
    /// 2×2→5-clamped block; velocity ×1.75 (×1.5 more on i%3==0),
    /// noLightEmittance + noGravity; the Item8 cue FIRST.
    #[test]
    fn w4_g11u_d54_style13_draws_match_the_oracle() {
        let mut app = union_rig(54);
        teleport_effect(&mut app, rect(), 13, 0, 1.0, Vec2::ZERO, KEY);
        let mut probe = UnifiedRandom::new(54);
        item_pitch(&mut probe);
        for _ in 0..21 {
            probe.next_float();
            probe.next_float();
            probe.next_range(1, 3);
            let _ = default_block(&mut probe, Vec2::ZERO, 2, 2);
        }
        assert_eq!(stream_of(&mut app), probe);
        let d = app.dust.slot(0);
        assert_eq!(d.dust_type, 27);
        assert!(d.no_light_emittance && d.no_gravity);
        assert_eq!(d.alpha, 100);
    }

    /// d54 `w4_d54_hsl_and_portal_colors` (union-adapted): the colour
    /// oracle re-pins to the canonical `ui::color::hsl_to_rgb` (the C#-
    /// exact f32→f64 widen + banker's rounding, `Main.cs:47890-47942`).
    #[test]
    fn w4_g11u_d54_hsl_and_portal_colors() {
        let hsl = crate::ui::color::hsl_to_rgb;
        // Pure red / green at the primary hues (blue's 1/3-f64 hue lands
        // a hair off the wrap — the C# float-widen shape, pinned via the
        // helper itself, not a byte literal).
        assert_eq!(hsl(0.0, 1.0, 0.5, 255), [255, 0, 0, 255]);
        assert_eq!(hsl(1.0 / 3.0, 1.0, 0.5, 255), [0, 255, 0, 255]);
        // The saturation-0 grey arm.
        assert_eq!(hsl(0.3, 0.0, 0.5, 255), [128, 128, 128, 255]);
        // Portal 0 (orange-ish) vs portal 1 (cyan-ish), alpha 66.
        let p0 = crate::teleport_dust::portal_color(0);
        let p1 = crate::teleport_dust::portal_color(1);
        assert_eq!(p0.a, 66);
        assert_eq!(p1.a, 66);
        let w0 = hsl(0.12, 1.0, 0.5, 66);
        let w1 = hsl(0.52, 1.0, 0.6, 66);
        assert_eq!((p0.r, p0.g, p0.b), (w0[0], w0[1], w0[2]));
        assert_eq!((p1.r, p1.g, p1.b), (w1[0], w1[1], w1[2]));
        // The player index does not change the single-player colour.
        assert_eq!(crate::teleport_dust::portal_color(2), p0);
        assert_eq!(crate::teleport_dust::portal_color(3), p1);
    }
}


