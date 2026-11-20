//! The aiStyle-44 family arm — the flying-fish / flying-antlion swimmers
//! (`NPC.AI()`'s `else if (this.aiStyle == 44)` block, `NPC.cs:30861-31124`).
//!
//! S1-D-3 (aiStyle-44 round, code-only mode): this module is the faithful
//! 1:1 port of the arm as a pure function over `Npc` + a small Env seam
//! (the c7fa4513 style-56 / d6bc30ed style-14 precedent).
//!
//! **WIRED by W4-c4 (2026-11-20).** The four `NpcKind` rows + their
//! `SetDefaults` tables landed with the census commit (5c4c8311); this
//! round claims the dispatch: the arm runs in the world-sim tick closure
//! (`world_sim/npcs.rs`) at the NPC's own slot turn — the separation
//! loop reads the sibling slots live (`Style44Sibling` carries the
//! VANILLA `cs_id`, not the protocol id) and the 587 materialize's dust
//! rolls ride the shared `DustPool` on the `UpdateNPCs` stream (the
//! style-56 seam). The style axis' 44 arm (`npc.rs`) owns the post-`AI()`
//! movement: the family is noGravity but NOT noTileCollide, so
//! `UpdateNPC` runs `UpdateCollision` (`NPC.cs:90490-90492`) — the
//! bat/fish flyer stand-in (`integrate_flyer_collision`) with the wet
//! re-probe (`Collision_WaterCollision`, `:93063`). `find_frame`'s four
//! kind arms ride [`style44_find_frame`]; the draw arms + sheets
//! (`STYLE44_SHEETS`) and the bestiary portraits landed with them.
//!
//! Family identity (verified from the C#, not from the roadmap): the
//! census of `aiStyle = 44` assignments in `NPC.cs` is exactly four:
//! * type 224, `NPCID.FlyingFish` (`ID/NPCID.cs:11125`, row
//!   `NPC.cs:11885-11897`): 32×18, dmg 9, def 4, 20 HP, kb 0.8,
//!   NPCHit1/NPCDeath1, value 300.
//! * type 509, `NPCID.GiantFlyingAntlion` (`ID/NPCID.cs:11695`, row
//!   `NPC.cs:14841-14853`): 40×31, dmg 34, def 12, 90 HP, kb 0.3,
//!   NPCHit32/NPCDeath35, value 195.
//! * type 581, `NPCID.FlyingAntlion` (`ID/NPCID.cs:11839`, row
//!   `NPC.cs:14868-14880`): 30×22, dmg 29, def 8, 60 HP, kb 0.5,
//!   NPCHit32/NPCDeath35, value 90.
//! * type 587, `NPCID.EyeballFlyingFish` (`ID/NPCID.cs:11851`, row
//!   `NPC.cs:16623-16637`): 32×18, dmg 35, def 18, 300 HP, kb 0,
//!   NPCHit1/NPCDeath1, value 1000, `alpha = 255`, rarity 1.
//! (`Projectile.cs`'s aiStyle 44 is the projectile axis — a different
//! family.) `FindFrame`'s `case 44` (`NPC.cs:68555`) is the TYPE switch
//! (type 44 = UndeadMiner), NOT this style family: the four kinds
//! animate in their own type cases (`:70104-70119` for 509/581,
//! `:74763-74785` for 224/587) — ported below as `style44_find_frame`.
//!
//! Per-kind branches INSIDE the arm: the 587 materialize block
//! (`:30888-30929`), the 509 / 581 / 587 parameter overrides
//! (`:30954-31034`) and the 587 daytime flee (`:31029-31033`). The 587
//! wet exemption appears twice: the retarget gate (`:30932`) and the
//! wet-lift tail (`:31112`).
//!
//! RNG by construction (S1-A-10): the arm body draws NO direct
//! `Main.rand.Next` for types 224/509/581 — the whole steering machine
//! is deterministic. Type 587's materialize block is the only stream
//! consumer, in exactly this per-tick order (all `Main.rand`):
//! 1. alpha == 255: the 15-dust burst — per dust one
//!    `Dust.NewDustDirect(position, width, height, 5)` (`:30897`),
//!    then `NextFloat()` for scale (`:30899`), then `NextFloat()` for
//!    fadeIn (`:30900`).
//! 2. alpha != 0 after the −15 step: the 2-dust trail — same three
//!    draws per dust (`:30913-30916`).
//! 3. `Next(3) == 0` gate (`:30920`): one `NewDustDirect` (`:30922`),
//!    then ONE `NextFloat()` for scale (`:30925`). The gate draw is
//!    consumed every tick of the block even when it refuses.
//! Each accepted `NewDustDirect` rolls the pool's cited order
//! (`Dust.cs:75-256`, the S1-E-1 port): pressure-band roll only when
//! the first free slot is above a `maxDustToDraw` band (`:114-148`),
//! then `Next(width-4)` (pos X, `:170`), `Next(height-4)` (pos Y,
//! `:171`), `Next(-20,21)` (vel X, `:172`), `Next(-20,21)` (vel Y,
//! `:173`), `Next(3)` (frame row, `:175`), `Next(-20,21)` (scale,
//! `:190`). Dust type 5 is BLOOD — it IS on the `DangerousDust` list
//! (`ChildSafety.cs:32`: `CreateBoolSet(true, 5, 227, ...)` flips the
//! listed ids to false, and `DangerousDust = !SafeDust[id]`,
//! `SetFactory.cs` `CreateBoolSet`), but `ChildSafety.Disabled`
//! defaults TRUE in the vanilla static ctor (`ChildSafety.cs:33`) and
//! in the port's pool, so the gate draws NOTHING at the stock
//! settings. When the App arms it (blood off), every accepted spawn
//! rolls one extra `Next(2)` — 50 % refuse to the sentinel, the accept
//! re-types to 16 with `NextFloat()` + `Next(3)` (`Dust.cs:194-213`);
//! the wiring round feeds the armed pool and the witnesses below pin
//! the stock-disabled stream. Type 5 hits NONE of the per-type tweak
//! arms (`Dust.cs:214-252` — 6 yes, 5 no). The off-screen cull
//! (`:97-101`) and the menu/pause guards
//! refuse with ZERO pool draws — but the caller's post-writes
//! (`scale`/`fadeIn` `NextFloat`s, `velocity` multiplies) still run,
//! on the 6000 sentinel slot exactly like `Main.dust[6000]`. The
//! velocity post-writes read `base.velocity` AFTER the materialize
//! block's `velocity.Y = -6f` (`:30893`) — kept in order.
//! `TargetClosest` (`NPC.cs:77652-77781`) is RNG-free. The stream
//! witnesses below replay the cited draws by hand (WRITTEN NOT RUN —
//! code-only mode).
//!
//! Faithfulness notes (the f1701eb0 defect catalogue applied):
//! * The bounce reads `oldVelocity` (the pre-move latch), flips the
//!   facing AGAINST it and pins the velocity to ±1 (`:30864-30887`) —
//!   no damp factor, no half-bounce.
//! * `noGravity = true` is written every tick (`:30863`). The port's
//!   `Npc` has no `noGravity` field (its flyers' integration is
//!   kind-driven); the write rides the Env seam as `wants_no_gravity`
//!   for the wiring round — nothing single-player-rendered changes.
//! * The retarget gate (`:30932`) is `target == 255 || (type != 587 &&
//!   player.wet) || player.dead || CanHit(1×1 centres)` — the C#
//!   `CanHit` here means VISIBLE: while the player is seen the ai[0]
//!   timer PINS at 90 and `TargetClosest` runs every tick; only when
//!   sight breaks does the 90-tick countdown run (`:30937-30941`,
//!   still calling `TargetClosest` each tick). The port reuses
//!   `npc_target::target_closest` unmodified (faceTarget = true
//!   default, ai_style 44 — no aiStyle-3 bottom-edge arm).
//! * `player.wet` has no `TargetCandidate` field; the Env carries a
//!   parallel per-slot slice (stock false until the wiring round
//!   feeds `Player.wet`).
//! * The num663/num664 latch (`:30930-30931`) is taken AFTER the 587
//!   block and BEFORE the retarget; the `netUpdate` clear
//!   (`:30942-30945`) fires when the flag is set but neither target
//!   nor facing changed this tick — kept verbatim, including the
//!   587 burst's own `netUpdate = true` (`:30894`).
//! * num671 is the CENTRE-to-centre X distance with INTEGER halves
//!   (`(float)(width / 2)`, `:30952`) — NOT `Entity.Center`'s float
//!   halves; the facing code's convention. num672's default is the
//!   player's position.Y MINUS the NPC's integer half-height
//!   (`:30953`); the 509/581 arms replace it with the player's FLOAT
//!   centre Y minus the same half-height (`:30962`/`:30995`), and 587
//!   with the player's raw position.Y (`:31028`).
//! * The X approach (`:31049-31083`) keeps the C#'s asymmetric
//!   double-add: the second `-= num665` fires only when the velocity
//!   is still on the WRONG side of ±num667 after the first step
//!   (crossing zero adds num665/2 instead), then the cap. The Y
//!   approach (`:31088-31111`) double-adds while moving toward the
//!   anchor from the wrong side, then clamps ±num668. All f32, in
//!   source order; `num667 *= 0.8f` and `num665 *= 0.7f` are f32
//!   multiplies of the f32 literals (`:31037-31038`).
//! * The separation loop (`:30964-30985` / `:30997-31018`) scans ALL
//!   `Main.maxNPCs` slots in index order (0..200), skipping the NPC
//!   itself by `whoAmI`, comparing `type` equality and the Manhattan
//!   distance `< (float)base.width` — strict `<`, position (not
//!   centre) deltas, ±0.05 per axis, `else` (ties push +).
//! * The lost-sight anchor is `Center.Y + directionY * 1000`
//!   (`:31039`, FLOAT centre) and the facing snaps to the velocity
//!   sign with the `direction == 0` fallback to +1 (`:31040-31047`).
//! * The 150-px band subtracts `num670 / 2f` from the Y anchor
//!   (`:31084-31087`) — AFTER the lost-sight override, so it applies
//!   to the wander anchor too (250/2 = 125 for 587).
//! * The wet tail (`:31112-31123`): ×0.95 only while sinking, then
//!   −0.5 always, then the −4 cap; 587 is exempt. The −4 cap is DEAD
//!   by the clamp order — the ±num668 clamps (`:31104-31111`) run
//!   first and num668 ≤ 3.5 < 4; kept verbatim.
//! * The arm writes NO ai[1..3], NO rotation for 224 (the fish tilt
//!   is FindFrame's, `:74768`/`:74773`), NO position integration —
//!   `UpdateNPC` integrates after `AI()`; the caller owns the step.
//! * Cited skips OUTSIDE the arm: the 581/509 ambient sound rolls
//!   (`NPC.cs:90123-90126`/`:90131-90134`, `Next(800)` then
//!   `Next(44,47)` — UpdateNPC's sound chain, NOT this arm's stream),
//!   the death gore/dust (`:86056-86069`: 224's gores 312/313, 587's
//!   50× type-5 dust at 2.5·hitDirection/−2.5 + gores 1104/1105) and
//!   the antlion death gores (`:86545-86600`) — the checkDead family;
//!   the platform fall-through (`Collision_DecideFallThroughPlatforms`
//!   `:92936-92939` returns true for aiStyle 44) — the port's
//!   collision layer owns that decision; the hit/death sounds
//!   (NPCHit1/32, NPCDeath1/35).
#![allow(dead_code)] // WIRED by W4-c4 (the world-sim slot-turn pre-pass + axis 44's integration tail + find_frame); the witnesses stay as the audit cross-check

use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};

/// The family's vanilla ids (the census above). `kind` on the port's
/// `Npc` is a `NpcKind` enum with no rows for these yet, so the arm
/// takes the C# type as an input; the wiring round maps kind → id.
pub const FLYING_FISH: u16 = 224;
pub const GIANT_FLYING_ANTLION: u16 = 509;
pub const FLYING_ANTLION: u16 = 581;
pub const EYEBALL_FLYING_FISH: u16 = 587;

/// One `Main.npc[i]` slot as the separation loop sees it
/// (`NPC.cs:30964-30985`): active, type, position. The slice is
/// slot-ordered (index = whoAmI); `self_index` is this NPC's slot.
#[derive(Clone, Copy, Debug)]
pub struct Style44Sibling {
    pub active: bool,
    pub npc_type: u16,
    pub x: f32,
    pub y: f32,
}

/// The per-tick environment the arm reads through `Main`'s statics.
/// The caller (the future style-axis arm) fills it from the App's
/// session player slice, the live NPC roster, the shared dust pool
/// (S1-E-1) and the pool's `Main.rand` analogue stream — the same
/// seam shape as the style-56 round's `Style56Env` (c7fa4513).
pub struct Style44Env<'a> {
    /// `this.type` — one of the four census ids above.
    pub npc_type: u16,
    /// `base.whoAmI` — this NPC's slot in `siblings`.
    pub self_index: usize,
    /// `Main.npc[0..Main.maxNPCs]` in slot order.
    pub siblings: &'a [Style44Sibling],
    /// The read-only world the target scan's and the gate's `CanHit`
    /// walks read.
    pub world: &'a World,
    /// The `Main.player` slots as the target scan sees them (the
    /// port's session player is slot 0).
    pub players: &'a [TargetCandidate],
    /// `Main.player[j].wet` per slot (`:30932`) — no
    /// `TargetCandidate` field; stock false until the wiring feeds
    /// `Player.wet`. Indexed like `players`.
    pub player_wet: &'a [bool],
    /// `Main.EverLastingTicker` for the `targetSetFrame` stamp
    /// (`NPC.cs:77780`) — the same stand-in as `update_zombie`.
    pub ticker: u64,
    /// `Main.dust` — the shared 6001-slot pool (`Dust.cs:75-256`).
    pub dust: &'a mut DustPool,
    /// The `Main` statics `NewDust` reads (`Dust.cs:77-98`).
    pub dust_ctx: &'a SpawnCtx,
    /// The pool's draw stream (`Main.rand`'s per-consumer analogue,
    /// the S1-E-1 convention).
    pub rng: &'a mut UnifiedRandom,
    /// `NPC.noGravity = true` (`:30863`): the port's `Npc` has no
    /// noGravity field; the wiring round consumes this latch (the
    /// four kinds are gravity-free flyers, like the port's bats).
    pub wants_no_gravity: bool,
}

/// `NPC.AI()`'s aiStyle-44 arm (`NPC.cs:30861-31124`), one tick.
pub fn style44_step(npc: &mut Npc, env: &mut Style44Env<'_>) {
    let npc_type = env.npc_type;
    // `:30863` — noGravity = true, every tick.
    env.wants_no_gravity = true;
    // `:30864-30875` — collideX: flip against oldVelocity.X, pin ±1.
    if npc.collide_x {
        if npc.old_vx > 0.0 {
            npc.facing = -1;
        } else {
            npc.facing = 1;
        }
        npc.vx = npc.facing as f32;
    }
    // `:30876-30887` — collideY: flip against oldVelocity.Y, pin ±1.
    if npc.collide_y {
        if npc.old_vy > 0.0 {
            npc.direction_y = -1;
        } else {
            npc.direction_y = 1;
        }
        npc.vy = npc.direction_y as f32;
    }
    // `:30888-30929` — the 587 materialize: the netOffset round trip,
    // the alpha-255 launch burst, the −15 fade, the trail and the
    // 1-in-3 sparkle. The dust post-writes read the CURRENT velocity
    // (the −6 launch included), in the C#'s statement order.
    if npc_type == EYEBALL_FLYING_FISH {
        // `:30890` — position += netOffset (zero in single-player;
        // the round trip is kept verbatim for the float restore).
        npc.x += npc.net_offset.x;
        npc.y += npc.net_offset.y;
        let (w, h) = npc.size();
        if npc.alpha == 255 {
            // `:30893-30894` — the launch kick + the wire flag.
            npc.vy = -6.0;
            npc.net_update = true;
            // `:30895-30902` — 15 dusts, type 5.
            for _ in 0..15 {
                let i = env.dust.new_dust_direct(
                    env.rng,
                    env.dust_ctx,
                    env.world,
                    Vec2::new(npc.x, npc.y),
                    w as i32,
                    h as i32,
                    5,
                    0.0,
                    0.0,
                    0,
                    Color::default(),
                    1.0,
                );
                let d = env.dust.slot_mut(i);
                d.velocity *= 0.5; // `:30898`
                d.scale = 1.0 + env.rng.next_float() * 0.5; // `:30899`
                d.fade_in = 1.5 + env.rng.next_float() * 0.5; // `:30900`
                d.velocity += Vec2::new(npc.vx, npc.vy) * 0.5; // `:30901`
            }
        }
        // `:30904-30908` — alpha -= 15, floored at 0 (the C# int).
        npc.alpha = npc.alpha.saturating_sub(15);
        if npc.alpha != 0 {
            // `:30911-30918` — the 2-dust trail.
            for _ in 0..2 {
                let i = env.dust.new_dust_direct(
                    env.rng,
                    env.dust_ctx,
                    env.world,
                    Vec2::new(npc.x, npc.y),
                    w as i32,
                    h as i32,
                    5,
                    0.0,
                    0.0,
                    0,
                    Color::default(),
                    1.0,
                );
                let d = env.dust.slot_mut(i);
                d.velocity *= 1.0; // `:30914` — the C#'s no-op, kept
                d.scale = 1.0 + env.rng.next_float() * 0.5; // `:30915`
                d.fade_in = 1.5 + env.rng.next_float() * 0.5; // `:30916`
                d.velocity += Vec2::new(npc.vx, npc.vy) * 0.3; // `:30917`
            }
        }
        // `:30920-30927` — the 1-in-3 sparkle; the gate draw runs
        // every tick of the block.
        if env.rng.next_max(3) == 0 {
            let i = env.dust.new_dust_direct(
                env.rng,
                env.dust_ctx,
                env.world,
                Vec2::new(npc.x, npc.y),
                w as i32,
                h as i32,
                5,
                0.0,
                0.0,
                0,
                Color::default(),
                1.0,
            );
            let d = env.dust.slot_mut(i);
            d.velocity *= 0.0; // `:30923`
            d.alpha = 120; // `:30924`
            d.scale = 0.7 + env.rng.next_float() * 0.5; // `:30925`
            d.velocity += Vec2::new(npc.vx, npc.vy) * 0.3; // `:30926`
        }
        // `:30928` — position -= netOffset.
        npc.x -= npc.net_offset.x;
        npc.y -= npc.net_offset.y;
    }
    // `:30930-30931` — the change latch, AFTER the 587 block.
    let num663 = npc.target;
    let num664 = npc.facing;
    // `:30932-30941` — the retarget gate. `Collision.CanHit(Center, 1,
    // 1, player.Center, 1, 1)` — the 1×1 centre sight walk; the C#
    // pins ai[0] = 90 while the player is visible (or wet/dead/none),
    // counts down only while sight is broken, and calls TargetClosest
    // on BOTH paths.
    // The C# indexes `Main.player[this.target]` unconditionally — slot
    // 255 is the always-present EMPTY player (inactive, dead, position
    // 0), so the gate's `dead` leg fires and the anchors below read the
    // zero box. The port's slice is short (one session player); the
    // last-slot clamp stands in for the empty slot — the wiring round
    // feeds a zeroed dead stand-in at the tail for exactness. With the
    // gate's `target == 255` leg short-circuiting first, the read is
    // observationally the C#'s for every reachable target.
    let slot = (npc.target.max(0) as usize).min(env.players.len() - 1);
    let p = &env.players[slot];
    let (cx, cy) = npc.center();
    let pcx = p.x + p.width as f32 / 2.0;
    let pcy = p.y + p.height as f32 / 2.0;
    let visible = crate::projectile::can_hit(env.world, cx, cy, 1.0, 1.0, pcx, pcy, 1.0, 1.0);
    let wet = env.player_wet[slot];
    if npc.target == 255 || (npc_type != EYEBALL_FLYING_FISH && wet) || p.dead || visible {
        npc.ai[0] = 90.0;
        target_closest(npc, env.world, env.players, true, 44, env.ticker);
    } else if npc.ai[0] > 0.0 {
        npc.ai[0] -= 1.0;
        target_closest(npc, env.world, env.players, true, 44, env.ticker);
    }
    // `:30942-30945` — the netUpdate clear: set but unchanged.
    if npc.net_update && num663 == npc.target && num664 == npc.facing {
        npc.net_update = false;
    }
    // `:30946-30953` — the default parameters and the anchors. num671
    // uses INTEGER half-widths (the facing convention), num672 the
    // player's raw Y minus the NPC's integer half-height.
    let mut num665 = 0.05f32; // X accel
    let mut num666 = 0.01f32; // Y accel
    let mut num667 = 3.0f32; // X cap
    let mut num668 = 1.0f32; // Y cap
    let mut num669 = 30.0f32; // X dead band
    let mut num670 = 100.0f32; // Y hover band
    let (w, h) = npc.size();
    // The same empty-slot stand-in as the gate above: `target` here is
    // post-`TargetClosest` (0..254, or the `:77731-77734` fallback 0),
    // but the clamp keeps the read total for a caller-supplied 255.
    let p = &env.players[(npc.target.max(0) as usize).min(env.players.len() - 1)];
    let num671 = (npc.x + (w as i32 / 2) as f32 - (p.x + (p.width / 2) as f32)).abs();
    let mut num672 = p.y - (h as i32 / 2) as f32;
    // `:30954-31034` — the per-kind overrides. The separation loops
    // are identical bodies with different constants above them; the
    // scan order, the strict `<` and the tie-push-+ are the C#'s.
    if npc_type == GIANT_FLYING_ANTLION || npc_type == FLYING_ANTLION {
        if npc_type == GIANT_FLYING_ANTLION {
            // `:30956-30961`
            num665 = 0.08;
            num666 = 0.03;
            num667 = 4.5;
            num668 = 2.0;
            num669 = 40.0;
            num670 = 150.0;
        } else {
            // `:30989-30994`
            num665 = 0.06;
            num666 = 0.02;
            num667 = 4.0;
            num668 = 2.0;
            num669 = 40.0;
            num670 = 150.0;
        }
        // `:30962`/`:30995` — the player's FLOAT centre Y.
        num672 = p.y + p.height as f32 / 2.0 - (h as i32 / 2) as f32;
        // `:30963`/`:30996` — the velocity tilt.
        npc.rotation = npc.vx * 0.1;
        // `:30964-30985`/`:30997-31018` — the swarm separation.
        for (j, s) in env.siblings.iter().enumerate() {
            if j != env.self_index
                && s.active
                && s.npc_type == npc_type
                && (npc.x - s.x).abs() + (npc.y - s.y).abs() < w
            {
                if npc.x < s.x {
                    npc.vx -= 0.05;
                } else {
                    npc.vx += 0.05;
                }
                if npc.y < s.y {
                    npc.vy -= 0.05;
                } else {
                    npc.vy += 0.05;
                }
            }
        }
    } else if npc_type == EYEBALL_FLYING_FISH {
        // `:31022-31028`
        num665 = 0.13;
        num666 = 0.09;
        num667 = 6.5;
        num668 = 3.5;
        num669 = 0.0;
        num670 = 250.0;
        num672 = p.y;
        // `:31029-31033` — daytime: flee upward (anchor 0) and turn.
        if env.world.day_time {
            num672 = 0.0;
            npc.facing *= -1;
        }
    }
    // `:31035-31048` — lost sight: the slow wander. The anchor rides
    // directionY 1000 px away; the facing snaps to the velocity sign.
    if npc.ai[0] <= 0.0 {
        num667 *= 0.8;
        num665 *= 0.7;
        num672 = npc.center().1 + (npc.direction_y * 1000) as f32;
        if npc.vx < 0.0 {
            npc.facing = -1;
        } else if npc.vx > 0.0 || npc.facing == 0 {
            npc.facing = 1;
        }
    }
    // `:31049-31083` — the X approach, outside the dead band. The
    // double-add fires only while still on the wrong side of the cap
    // after the first step; crossing zero adds half instead.
    if num671 > num669 {
        if npc.facing == -1 && npc.vx > 0.0 - num667 {
            npc.vx -= num665;
            if npc.vx > num667 {
                npc.vx -= num665;
            } else if npc.vx > 0.0 {
                npc.vx -= num665 / 2.0;
            }
            if npc.vx < 0.0 - num667 {
                npc.vx = 0.0 - num667;
            }
        } else if npc.facing == 1 && npc.vx < num667 {
            npc.vx += num665;
            if npc.vx < 0.0 - num667 {
                npc.vx += num665;
            } else if npc.vx < 0.0 {
                npc.vx += num665 / 2.0;
            }
            if npc.vx > num667 {
                npc.vx = num667;
            }
        }
    }
    // `:31084-31087` — far out: hover half a band ABOVE the anchor.
    if num671 > num670 {
        num672 -= num670 / 2.0;
    }
    // `:31088-31111` — the Y approach + the ±num668 clamp.
    if npc.y < num672 {
        npc.vy += num666;
        if npc.vy < 0.0 {
            npc.vy += num666;
        }
    } else {
        npc.vy -= num666;
        if npc.vy > 0.0 {
            npc.vy -= num666;
        }
    }
    if npc.vy < 0.0 - num668 {
        npc.vy = 0.0 - num668;
    }
    if npc.vy > num668 {
        npc.vy = num668;
    }
    // `:31112-31123` — the wet lift (587 exempt): damp the sink,
    // lift 0.5, cap the rise at −4.
    if npc_type != EYEBALL_FLYING_FISH && npc.wet {
        if npc.vy > 0.0 {
            npc.vy *= 0.95;
        }
        npc.vy -= 0.5;
        if npc.vy < -4.0 {
            npc.vy = -4.0;
        }
    }
}

/// `NPC.FindFrame`'s type cases for the family (`NPC.cs:70104-70119`
/// for 509/581, `:74763-74785` for 224/587). `frame` is the ROW index
/// (the port's `Npc::frame` convention — the C# stores `frame.Y = row
/// * frameHeight`), `frame_counter` the C# `frameCounter`, `frame_count`
/// `Main.npcFrameCount[type]` (224/587: 4, 509/581: 9 — the
/// `Main.cs:67082` table). The 224/587 arm also writes the velocity
/// tilt rotation (`:74768`/`:74773`); the 509/581 arm leaves the AI's
/// `velocity.X * 0.1` tilt alone.
pub fn style44_find_frame(
    npc_type: u16,
    vx: f32,
    vy: f32,
    facing: i32,
    frame_count: u32,
    frame: &mut u32,
    frame_counter: &mut f32,
    sprite_dir: &mut i32,
    rotation: &mut f32,
) {
    match npc_type {
        GIANT_FLYING_ANTLION | FLYING_ANTLION => {
            // `:70106-70109` — face the motion only when they agree.
            if (vx > 0.0 && facing > 0) || (vx < 0.0 && facing < 0) {
                *sprite_dir = facing;
            }
            // `:70110-70118` — ++frameCounter, advance every 3 ticks,
            // wrap at npcFrameCount.
            *frame_counter += 1.0;
            if *frame_counter >= 3.0 {
                *frame_counter = 0.0;
                *frame += 1;
                if *frame >= frame_count {
                    *frame = 0;
                }
            }
        }
        FLYING_FISH | EYEBALL_FLYING_FISH => {
            // `:74765-74774` — the sprite flips with the velocity sign
            // and the tilt is ±velocity.Y * 0.2f. vx == 0 writes
            // neither (the C#'s two independent ifs).
            if vx > 0.0 {
                *sprite_dir = 1;
                *rotation = vy * 0.2;
            }
            if vx < 0.0 {
                *sprite_dir = -1;
                *rotation = (0.0 - vy) * 0.2;
            }
            // `:74775-74784` — advance every 6 ticks, wrap at row 4
            // (the literal `num * 4`, both kinds' npcFrameCount).
            *frame_counter += 1.0;
            if *frame_counter >= 6.0 {
                *frame += 1;
                *frame_counter = 0.0;
            }
            if *frame >= 4 {
                *frame = 0;
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::particles::DUST_SENTINEL;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

    fn cand(x: f32, y: f32) -> TargetCandidate {
        TargetCandidate {
            active: true,
            dead: false,
            ghost: false,
            x,
            y,
            width: 20,
            height: 42,
            aggro: 0,
            item_animation: 0,
            npc_type_no_aggro: false,
            tank_pet: None,
        }
    }

    /// A family stand-in: the port has no kind rows yet (the wiring
    /// round adds them with the census rows); the box rides
    /// `size_override` so the arm reads the C#'s exact width/height.
    /// Facing 1, the SetDefaults target 255.
    fn flyer(x: f32, y: f32, w: f32, h: f32) -> Npc {
        let mut n = Npc::new(NpcKind::Creeper, x, y, 1, 0.0);
        n.size_override = Some((w, h));
        n
    }

    fn air() -> World {
        World::new("t".into(), 200, 120)
    }

    /// On-screen ctx: the test boxes sit inside the 400-px cull
    /// margin; maxDustToDraw 6000 puts slot 0 in the low band (no
    /// gate draw).
    fn ctx() -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(1920.0, 1080.0),
            max_dust_to_draw: 6000,
        }
    }

    /// A minimal env: one visible player at (400, 489) (centre 410,
    /// 510), no siblings, dry, night.
    struct Bench {
        world: World,
        pool: DustPool,
        ctx: SpawnCtx,
        rng: UnifiedRandom,
        players: [TargetCandidate; 1],
        wet: [bool; 1],
        siblings: Vec<Style44Sibling>,
    }

    impl Bench {
        fn new() -> Self {
            Self {
                world: air(),
                pool: DustPool::new(),
                ctx: ctx(),
                rng: UnifiedRandom::new(12345),
                players: [cand(400.0, 489.0)],
                wet: [false],
                siblings: Vec::new(),
            }
        }

        fn env(&mut self, npc_type: u16) -> Style44Env<'_> {
            Style44Env {
                npc_type,
                self_index: 0,
                siblings: &self.siblings,
                world: &self.world,
                players: &self.players,
                player_wet: &self.wet,
                ticker: 7,
                dust: &mut self.pool,
                dust_ctx: &self.ctx,
                rng: &mut self.rng,
                wants_no_gravity: false,
            }
        }
    }

    /// The census + the parameter table (`NPC.cs:30946-30951`,
    /// `:30956-30961`, `:30989-30994`, `:31022-31027`): the four ids
    /// and the six constants per kind, verified by driving one tick
    /// from rest with the player far right (num671 > band) and
    /// reading the first-step accelerations.
    #[test]
    fn s1_d_3_s44_the_parameter_table_matches_the_census() {
        assert_eq!(
            (FLYING_FISH, GIANT_FLYING_ANTLION, FLYING_ANTLION, EYEBALL_FLYING_FISH),
            (224, 509, 581, 587)
        );
        // (box, X accel first step, Y cap): the player sits far right
        // and BELOW, so both approaches run their plain first step.
        let rows: [(u16, f32, f32, f32, f32); 4] = [
            (FLYING_FISH, 32.0, 18.0, 0.05, 1.0),
            (GIANT_FLYING_ANTLION, 40.0, 31.0, 0.08, 2.0),
            (FLYING_ANTLION, 30.0, 22.0, 0.06, 2.0),
            (EYEBALL_FLYING_FISH, 32.0, 18.0, 0.13, 3.5),
        ];
        for (t, w, h, accel, _ycap) in rows {
            let mut b = Bench::new();
            b.world.day_time = false;
            let mut n = flyer(100.0, 100.0, w, h);
            if t == EYEBALL_FLYING_FISH {
                n.alpha = 0; // skip the materialize draws here
            }
            let mut e = b.env(t);
            style44_step(&mut n, &mut e);
            assert_eq!(n.vx, accel, "type {t}: the first X step is num665");
            assert!(e.wants_no_gravity, "type {t}: noGravity latch");
        }
        // The Y caps: start far ABOVE the anchor moving down fast.
        for (t, w, h, ycap) in [
            (FLYING_FISH, 32.0f32, 18.0f32, 1.0f32),
            (GIANT_FLYING_ANTLION, 40.0, 31.0, 2.0),
            (FLYING_ANTLION, 30.0, 22.0, 2.0),
            (EYEBALL_FLYING_FISH, 32.0, 18.0, 3.5),
        ] {
            let mut b = Bench::new();
            b.players = [cand(100.0, 900.0)]; // anchor far below
            let mut n = flyer(100.0, 100.0, w, h);
            n.vy = 50.0;
            if t == EYEBALL_FLYING_FISH {
                n.alpha = 0;
            }
            style44_step(&mut n, &mut b.env(t));
            assert_eq!(n.vy, ycap, "type {t}: the Y clamp is num668");
        }
    }

    /// `NPC.cs:30864-30887` — the bounces read oldVelocity, flip the
    /// facing against it and PIN the velocity to ±1 (no damp). The
    /// geometry keeps the later passes from masking the pin: the
    /// player sits inside the 30-px X dead band on the bounce's side
    /// (no X write) and exactly ON the Y anchor (the else branch's
    /// −0.01 double-add is the only Y write, `:31096-31102`).
    #[test]
    fn s1_d_3_s44_the_bounces_pin_the_velocity_against_old_velocity() {
        let mut b = Bench::new();
        // Player (90, 109): centre X 100 vs the NPC's 116 -> num671 16
        // (dead band, and TargetClosest faces LEFT, agreeing with the
        // bounce); anchor = 109 - 9 = 100 == the NPC's y; the player's
        // rect centre Y 130 > 109 -> directionY +1, agreeing too.
        b.players = [cand(90.0, 109.0)];
        let mut n = flyer(100.0, 100.0, 32.0, 18.0);
        n.collide_x = true;
        n.collide_y = true;
        n.old_vx = 2.5; // was moving right -> face left
        n.old_vy = -3.0; // was moving up -> face down
        n.vx = 2.5;
        n.vy = -3.0;
        style44_step(&mut n, &mut b.env(FLYING_FISH));
        assert_eq!(n.facing, -1);
        assert_eq!(n.vx, -1.0, "collideX pins to direction, not a bounce damp");
        assert_eq!(n.direction_y, 1);
        // The pin is 1; the same tick's Y approach subtracts 0.01 twice
        // (y == anchor -> else, still > 0 -> double-add).
        assert_eq!(n.vy, (1.0 - 0.01) - 0.01);
        // The zero oldVelocity edge: `> 0f` is false -> face +1. The
        // player mirrors to the RIGHT (120, 109): centre 130, num671
        // 14, facing +1, same anchor.
        let mut b2 = Bench::new();
        b2.players = [cand(120.0, 109.0)];
        let mut n2 = flyer(100.0, 100.0, 32.0, 18.0);
        n2.collide_x = true;
        n2.collide_y = true;
        n2.facing = -1;
        n2.direction_y = -1;
        style44_step(&mut n2, &mut b2.env(FLYING_FISH));
        assert_eq!(n2.facing, 1);
        assert_eq!(n2.vx, 1.0);
        assert_eq!(n2.direction_y, 1);
        assert_eq!(n2.vy, (1.0 - 0.01) - 0.01);
    }

    /// `NPC.cs:30932-30941` — visible pins ai[0] = 90; a wall breaks
    /// sight and the countdown runs (TargetClosest still fires); at
    /// zero the wander branch takes over.
    #[test]
    fn s1_d_3_s44_the_sight_gate_pins_ninety_then_counts_down() {
        let mut b = Bench::new();
        let mut n = flyer(100.0, 100.0, 32.0, 18.0);
        n.target = 0;
        n.ai[0] = 5.0;
        style44_step(&mut n, &mut b.env(FLYING_FISH));
        assert_eq!(n.ai[0], 90.0, "open air: the gate pins 90 (:30934)");
        // A solid column between the centres breaks the 1×1 walk.
        // NPC centre (116, 109) -> tile (7, 6); player centre (410,
        // 510) -> tile (25, 31). The walk steps Y first (|dx| 18 <
        // |dy| 25): a full column at x 7..25 row-band blocks it.
        for y in 0..80 {
            let i = b.world.index(16, y);
            b.world.tiles[i] = Tile { kind: 1, ..Default::default() };
            b.world.tiles[i].flags |= FLAG_ACTIVE;
        }
        let mut n2 = flyer(100.0, 100.0, 32.0, 18.0);
        n2.target = 0;
        n2.ai[0] = 5.0;
        style44_step(&mut n2, &mut b.env(FLYING_FISH));
        assert_eq!(n2.ai[0], 4.0, "sight broken: the countdown (:30939)");
        // At zero it stays zero (the `> 0f` gate) and the wander
        // anchor rides directionY: vy accelerates toward +1000.
        let mut n3 = flyer(100.0, 100.0, 32.0, 18.0);
        n3.target = 0;
        n3.ai[0] = 0.0;
        n3.direction_y = 1;
        style44_step(&mut n3, &mut b.env(FLYING_FISH));
        assert_eq!(n3.ai[0], 0.0);
        assert!(n3.vy > 0.0, "the wander anchor is 1000 px below");
    }

    /// `NPC.cs:31035-31048` — the lost-sight wander scales the caps
    /// (×0.8 / ×0.7) and snaps the facing to the velocity sign with
    /// the direction==0 fallback.
    #[test]
    fn s1_d_3_s44_the_wander_scales_and_snaps_the_facing() {
        let mut b = Bench::new();
        // Block sight so ai[0] stays 0 (the wall from the gate test).
        for y in 0..80 {
            let i = b.world.index(16, y);
            b.world.tiles[i] = Tile { kind: 1, ..Default::default() };
            b.world.tiles[i].flags |= FLAG_ACTIVE;
        }
        let mut n = flyer(100.0, 100.0, 32.0, 18.0);
        n.target = 0;
        n.ai[0] = 0.0;
        n.vx = -1.0;
        n.facing = 1;
        style44_step(&mut n, &mut b.env(FLYING_FISH));
        assert_eq!(n.facing, -1, "vx < 0 snaps the facing (:31040-31043)");
        // The X approach runs at the scaled accel while the target is
        // far: num671 = |116 - 410| = 294 > 30. Facing -1 after the
        // snap, vx -1 > -2.4: first step -0.035, still < 0 so no
        // double-add; the witness pins the ×0.7 literal.
        assert_eq!(n.vx, -1.0 - 0.05f32 * 0.7);
        // vx == 0 with facing 0 -> +1 (the `|| direction == 0` arm).
        let mut n2 = flyer(100.0, 100.0, 32.0, 18.0);
        n2.target = 0;
        n2.ai[0] = 0.0;
        n2.facing = 0;
        style44_step(&mut n2, &mut b.env(FLYING_FISH));
        assert_eq!(n2.facing, 1);
    }

    /// `NPC.cs:31049-31083` — the asymmetric double-add: crossing
    /// zero adds accel/2, staying on the wrong side of the CAP adds
    /// the full accel again, then the cap pins.
    #[test]
    fn s1_d_3_s44_the_x_approach_double_add_is_asymmetric() {
        let mut b = Bench::new();
        // Facing +1, player far right (num671 = 294 > 30), vx just
        // below zero: first step crosses to +0.03 (0.05 - 0.02), the
        // `vx < 0` re-test is false -> NO half-add on the far side.
        let mut n = flyer(100.0, 100.0, 32.0, 18.0);
        n.target = 0;
        n.vx = -0.02;
        style44_step(&mut n, &mut b.env(FLYING_FISH));
        assert_eq!(n.vx, -0.02 + 0.05);
        // vx positive but small: first step stays > 0 -> half-add.
        let mut n2 = flyer(100.0, 100.0, 32.0, 18.0);
        n2.target = 0;
        n2.vx = -0.08;
        style44_step(&mut n2, &mut b.env(FLYING_FISH));
        assert_eq!(n2.vx, -0.08 + 0.05 + 0.05 / 2.0, "crossing zero: +accel/2");
        // The cap: vx at 2.98 + 0.05 = 3.03 > 3 -> pinned 3.
        let mut n3 = flyer(100.0, 100.0, 32.0, 18.0);
        n3.target = 0;
        n3.vx = 2.98;
        style44_step(&mut n3, &mut b.env(FLYING_FISH));
        assert_eq!(n3.vx, 3.0);
        // Inside the dead band (num671 <= 30): no X write at all.
        let mut b2 = Bench::new();
        b2.players = [cand(120.0, 489.0)]; // centres 116 vs 130: 14 < 30
        let mut n4 = flyer(100.0, 100.0, 32.0, 18.0);
        n4.target = 0;
        n4.vx = 0.7;
        style44_step(&mut n4, &mut b2.env(FLYING_FISH));
        assert_eq!(n4.vx, 0.7, "the dead band holds the velocity");
    }

    /// `NPC.cs:31084-31111` — the far band lifts the anchor by
    /// num670/2 and the Y approach double-adds toward it.
    #[test]
    fn s1_d_3_s44_the_far_band_lifts_the_anchor_half_a_band() {
        let mut b = Bench::new();
        // num671 = 294 > 100: anchor = player.y - 9 - 50 = 430.
        // NPC y 100 < 430, vy 0: += 0.01, not < 0 -> single add.
        let mut n = flyer(100.0, 100.0, 32.0, 18.0);
        n.target = 0;
        style44_step(&mut n, &mut b.env(FLYING_FISH));
        assert_eq!(n.vy, 0.01);
        // Rising toward a BELOW anchor: the wrong-side double-add fires
        // only while STILL rising after the first step (`vy < 0f`).
        // From -0.02 the first +0.01 crosses to -0.01 — the f32 sum is
        // EXACTLY -0.01 (Sterbenz: the operands are within a factor 2,
        // so the subtraction is exact), the re-test holds, the second
        // +0.01 lands on 0.
        let mut n2 = flyer(100.0, 100.0, 32.0, 18.0);
        n2.target = 0;
        n2.vy = -0.02;
        style44_step(&mut n2, &mut b.env(FLYING_FISH));
        assert_eq!(n2.vy, 0.0, "still rising after step 1: double add");
        // Crossing zero: single add only (-0.005 + 0.01 > 0).
        let mut n2b = flyer(100.0, 100.0, 32.0, 18.0);
        n2b.target = 0;
        n2b.vy = -0.005;
        style44_step(&mut n2b, &mut b.env(FLYING_FISH));
        assert_eq!(n2b.vy, -0.005 + 0.01, "crossed zero: no double add");
        // The anchor arithmetic witness: at y == anchor exactly the
        // else branch runs (vy -= accel). anchor 430 with the lift.
        let mut n3 = flyer(100.0, 430.0, 32.0, 18.0);
        n3.target = 0;
        n3.vy = 0.0;
        style44_step(&mut n3, &mut b.env(FLYING_FISH));
        assert_eq!(n3.vy, -0.01, "y == anchor takes the else (:31096)");
    }

    /// `NPC.cs:30964-30985` — the antlion separation: same-type
    /// active siblings inside the Manhattan width push ±0.05 per
    /// axis; the self slot, other types and inactive slots are
    /// skipped; ties push +.
    #[test]
    fn s1_d_3_s44_the_antlions_separate_from_their_swarm() {
        let mut b = Bench::new();
        b.siblings = vec![
            Style44Sibling { active: true, npc_type: GIANT_FLYING_ANTLION, x: 100.0, y: 100.0 }, // self slot 0
            Style44Sibling { active: true, npc_type: GIANT_FLYING_ANTLION, x: 110.0, y: 105.0 }, // near, right+below
            Style44Sibling { active: true, npc_type: FLYING_ANTLION, x: 101.0, y: 101.0 },       // other type
            Style44Sibling { active: false, npc_type: GIANT_FLYING_ANTLION, x: 101.0, y: 101.0 },// inactive
            Style44Sibling { active: true, npc_type: GIANT_FLYING_ANTLION, x: 200.0, y: 100.0 }, // |dx| 100 >= 40
        ];
        let mut n = flyer(100.0, 100.0, 40.0, 31.0);
        n.target = 0;
        let vx0 = n.vx;
        let vy0 = n.vy;
        style44_step(&mut n, &mut b.env(GIANT_FLYING_ANTLION));
        // Only sibling 1 pushes: npc left of it -> vx -= 0.05, above
        // it -> vy -= 0.05, BEFORE the steering adds its own steps.
        // The X approach then adds num665 = 0.08 (facing +1, far
        // target). The Y approach adds 0.03 TWICE: the anchor is far
        // below (495 - 75 = 420), the first add leaves vy at -0.02,
        // still < 0 -> the wrong-side double-add (`:31091-31094`).
        assert_eq!(n.vx, vx0 - 0.05 + 0.08);
        assert_eq!(n.vy, vy0 - 0.05 + 0.03 + 0.03);
        // `:30963` runs BEFORE the loop: the tilt reads the entry vx.
        assert_eq!(n.rotation, vx0 * 0.1);
    }

    /// `NPC.cs:30963`/`:30996` — the rotation is written BEFORE the
    /// separation loop, so it reads the pre-separation velocity.
    #[test]
    fn s1_d_3_s44_the_antlion_tilt_reads_the_pre_separation_velocity() {
        let mut b = Bench::new();
        b.siblings = vec![
            Style44Sibling { active: true, npc_type: FLYING_ANTLION, x: 100.0, y: 100.0 },
            Style44Sibling { active: true, npc_type: FLYING_ANTLION, x: 110.0, y: 100.0 },
        ];
        let mut n = flyer(100.0, 100.0, 30.0, 22.0);
        n.target = 0;
        n.vx = 1.0;
        style44_step(&mut n, &mut b.env(FLYING_ANTLION));
        assert_eq!(n.rotation, 0.1, "1.0 * 0.1 taken before the -0.05 push");
        assert_eq!(n.vx, 1.0 - 0.05 + 0.06, "the push then the approach");
    }

    /// `NPC.cs:31020-31033` — 587: the raw-player-Y anchor, the zero
    /// dead band, and the daytime flee (anchor 0 + facing flip).
    #[test]
    fn s1_d_3_s44_the_eyeball_fish_flees_at_dawn() {
        let mut b = Bench::new();
        b.world.day_time = true;
        let mut n = flyer(100.0, 100.0, 32.0, 18.0);
        n.target = 0;
        n.alpha = 0;
        n.facing = 1;
        style44_step(&mut n, &mut b.env(EYEBALL_FLYING_FISH));
        assert_eq!(n.facing, -1, "daytime flips the facing (:31032)");
        // Anchor 0 - 125 (the far-band lift) = -125: the fish rises.
        assert!(n.vy < 0.0, "the dawn anchor is above");
        // Night: the anchor is the player's raw Y (489), no flip.
        let mut b2 = Bench::new();
        b2.world.day_time = false;
        let mut n2 = flyer(100.0, 100.0, 32.0, 18.0);
        n2.target = 0;
        n2.alpha = 0;
        n2.facing = 1;
        style44_step(&mut n2, &mut b2.env(EYEBALL_FLYING_FISH));
        assert_eq!(n2.facing, 1);
        assert!(n2.vy > 0.0, "night: chase the player's raw Y");
    }

    /// `NPC.cs:31112-31123` — the wet tail: damp the sink ×0.95,
    /// lift 0.5, cap −4; 587 is exempt. NOTE the C# clamp order: the
    /// ±num668 clamps (`:31104-31111`) run BEFORE the tail, so the
    /// −4 cap is dead for the family (num668 ≤ 3.5 < 4 and the lift
    /// only lowers vy from ≥ −num668) — kept verbatim, witnessed by
    /// the fast riser landing on −num668 − 0.5, NOT −4.
    #[test]
    fn s1_d_3_s44_the_wet_lift_damps_the_sink_and_caps_the_rise() {
        let mut b = Bench::new();
        let mut n = flyer(100.0, 480.0, 32.0, 18.0); // near the anchor: small vy steps
        n.target = 0;
        n.wet = true;
        n.vy = 1.0;
        style44_step(&mut n, &mut b.env(FLYING_FISH));
        // num671 = |116 - 410| = 294 > num670 100: anchor = 489 - 9 - 50
        // = 430 < y 480 -> the else branch: 1 - 0.01 = 0.99, still > 0
        // -> double-add -> 0.98; then ×0.95, then −0.5.
        assert_eq!(n.vy, ((1.0 - 0.01) - 0.01) * 0.95 - 0.5);
        // A fast riser: the steering clamps −10 to −num668 = −1 FIRST
        // (`:31104`), then the tail lifts to −1.5 — the −4 cap
        // (`:31119`) never fires (dead by the clamp order).
        let mut n2 = flyer(100.0, 480.0, 32.0, 18.0);
        n2.target = 0;
        n2.wet = true;
        n2.vy = -10.0;
        style44_step(&mut n2, &mut b.env(FLYING_FISH));
        assert_eq!(n2.vy, -1.0 - 0.5);
        // 587 exempt: the same setup keeps the plain steering value.
        let mut n3 = flyer(100.0, 480.0, 32.0, 18.0);
        n3.target = 0;
        n3.wet = true;
        n3.alpha = 0;
        n3.vy = 1.0;
        style44_step(&mut n3, &mut b.env(EYEBALL_FLYING_FISH));
        // 587's anchor: raw player Y 489 minus the far-band 125
        // (num671 294 > num670 250) = 364 < 480 -> the else branch:
        // 1 - 0.09 = 0.91, still > 0 -> double-add -> 0.82. No wet tail.
        assert_eq!(n3.vy, (1.0 - 0.09) - 0.09, "no wet tail for 587 (:31112)");
    }

    /// `NPC.cs:30891-30908` — the materialize: the alpha-255 tick
    /// launches at −6, arms netUpdate, fades by 15; the fade floors
    /// at 0 and the trail stops.
    #[test]
    fn s1_d_3_s44_the_eyeball_fish_materializes_over_seventeen_ticks() {
        let mut b = Bench::new();
        let mut n = flyer(100.0, 100.0, 32.0, 18.0);
        n.target = 0;
        n.alpha = 255;
        style44_step(&mut n, &mut b.env(EYEBALL_FLYING_FISH));
        assert_eq!(n.alpha, 240);
        // The launch kick is overwritten by the steering tail in the
        // same tick (the C#'s order) — the witness is the burst's
        // velocity post-write: dust vel += velocity * 0.5 with the
        // −6 launch still live at :30901.
        let d = b.pool.slot(0);
        assert!(d.active);
        // 15 burst + 2 trail dusts occupied slots 0..16 (the 1-in-3
        // gate may add one more; the count is >= 17).
        let active = (0..DUST_SENTINEL).filter(|&i| b.pool.slot(i).active).count();
        assert!(active >= 17, "15 burst + 2 trail, got {active}");
        // Fade to zero: 255 / 15 = 17 ticks; alpha 15 -> 0, no trail.
        let mut n2 = flyer(100.0, 100.0, 32.0, 18.0);
        n2.target = 0;
        n2.alpha = 15;
        let mut b2 = Bench::new();
        style44_step(&mut n2, &mut b2.env(EYEBALL_FLYING_FISH));
        assert_eq!(n2.alpha, 0);
        let active2 = (0..DUST_SENTINEL).filter(|&i| b2.pool.slot(i).active).count();
        assert!(active2 <= 1, "alpha 0: only the 1-in-3 sparkle can spawn");
    }

    /// The RNG-order witness (S1-A-10, WRITTEN NOT RUN): the 587
    /// materialize tick consumes exactly the cited draws. A twin
    /// stream replayed by hand — 15×(6 pool + 2 float) + 2×(6 pool +
    /// 2 float) + 1 gate + (spawn ? 6 pool + 1 float : 0) — must land
    /// on the same next value as the arm's stream.
    #[test]
    fn s1_d_3_s44_the_materialize_stream_order_matches_the_cited_draws() {
        let mut b = Bench::new();
        let mut n = flyer(100.0, 100.0, 32.0, 18.0);
        n.target = 0;
        n.alpha = 255;
        style44_step(&mut n, &mut b.env(EYEBALL_FLYING_FISH));
        // Replay: low-band slots (no pressure draw), 32×18 box ->
        // Next(28), Next(14), Next(-20,21) ×2, Next(3), Next(-20,21).
        let mut twin = UnifiedRandom::new(12345);
        let mut spawned = 0usize;
        for k in 0..17 {
            // All 17 spawns are on-screen and low-band: accepted.
            twin.next_max(28);
            twin.next_max(14);
            twin.next_range(-20, 21);
            twin.next_range(-20, 21);
            twin.next_max(3);
            twin.next_range(-20, 21);
            twin.next_float(); // scale
            twin.next_float(); // fadeIn
            spawned += 1;
            let _ = k;
        }
        // The 1-in-3 gate.
        let gate = twin.next_max(3);
        if gate == 0 {
            twin.next_max(28);
            twin.next_max(14);
            twin.next_range(-20, 21);
            twin.next_range(-20, 21);
            twin.next_max(3);
            twin.next_range(-20, 21);
            twin.next_float(); // scale only — no fadeIn (:30925)
            spawned += 1;
        }
        assert_eq!(spawned, (0..DUST_SENTINEL).filter(|&i| b.pool.slot(i).active).count());
        assert_eq!(twin.next(), b.rng.next(), "the streams stay in lockstep");
        assert_eq!(twin.next(), b.rng.next());
    }

    /// `NPC.cs:30942-30945` — the netUpdate clear: the burst arms the
    /// flag, but an unchanged target/facing clears it in the same
    /// tick; a real change keeps it.
    #[test]
    fn s1_d_3_s44_the_net_update_clear_needs_an_unchanged_pick() {
        let mut b = Bench::new();
        let mut n = flyer(100.0, 100.0, 32.0, 18.0);
        n.alpha = 255;
        n.target = 255; // the pick WILL change -> flag survives
        style44_step(&mut n, &mut b.env(EYEBALL_FLYING_FISH));
        assert!(n.net_update, "255 -> 0 is a change (:30942 keeps it)");
        // Steady state: target 0, facing already settled, old_*
        // latched — the burst arms the flag, the clear removes it.
        let mut n2 = flyer(100.0, 100.0, 32.0, 18.0);
        n2.alpha = 255;
        n2.target = 0;
        n2.old_target = 0;
        n2.old_direction = n2.facing;
        n2.old_direction_y = n2.direction_y;
        style44_step(&mut n2, &mut b.env(EYEBALL_FLYING_FISH));
        assert!(!n2.net_update, "unchanged pick: the clear fires");
    }

    /// `NPC.cs:70104-70119` — the antlion flap: 3-tick stride over 9
    /// rows, sprite faces the motion only when they agree.
    #[test]
    fn s1_d_3_s44_the_antlion_frame_cycles_every_three_ticks() {
        let mut frame = 0u32;
        let mut fc = 0.0f32;
        let mut sd = -1i32;
        let mut rot = 0.0f32;
        for _ in 0..2 {
            style44_find_frame(FLYING_ANTLION, 1.0, 0.0, 1, 9, &mut frame, &mut fc, &mut sd, &mut rot);
        }
        assert_eq!(frame, 0, "two ticks: not yet 3");
        style44_find_frame(FLYING_ANTLION, 1.0, 0.0, 1, 9, &mut frame, &mut fc, &mut sd, &mut rot);
        assert_eq!(frame, 1);
        assert_eq!(fc, 0.0);
        assert_eq!(sd, 1, "vx and facing agree");
        // Disagreement leaves the sprite alone.
        style44_find_frame(FLYING_ANTLION, -1.0, 0.0, 1, 9, &mut frame, &mut fc, &mut sd, &mut rot);
        assert_eq!(sd, 1);
        // The wrap at row 9.
        frame = 8;
        fc = 2.0;
        style44_find_frame(GIANT_FLYING_ANTLION, 1.0, 0.0, 1, 9, &mut frame, &mut fc, &mut sd, &mut rot);
        assert_eq!(frame, 0);
    }

    /// `NPC.cs:74763-74785` — the fish flap: 6-tick stride over 4
    /// rows, the ±velocity.Y·0.2 tilt, vx == 0 writes neither.
    #[test]
    fn s1_d_3_s44_the_fish_frame_tilts_with_the_vertical_velocity() {
        let mut frame = 0u32;
        let mut fc = 0.0f32;
        let mut sd = -1i32;
        let mut rot = 0.0f32;
        style44_find_frame(FLYING_FISH, 2.0, 1.5, 1, 4, &mut frame, &mut fc, &mut sd, &mut rot);
        assert_eq!(sd, 1);
        assert_eq!(rot, 1.5 * 0.2);
        style44_find_frame(EYEBALL_FLYING_FISH, -2.0, 1.5, -1, 4, &mut frame, &mut fc, &mut sd, &mut rot);
        assert_eq!(sd, -1);
        assert_eq!(rot, -1.5 * 0.2);
        // vx == 0: no sprite/rotation write.
        style44_find_frame(FLYING_FISH, 0.0, 9.0, 1, 4, &mut frame, &mut fc, &mut sd, &mut rot);
        assert_eq!(sd, -1);
        assert_eq!(rot, -1.5 * 0.2);
        // The 6-tick stride + the row-4 wrap.
        for _ in 0..3 {
            style44_find_frame(FLYING_FISH, 1.0, 0.0, 1, 4, &mut frame, &mut fc, &mut sd, &mut rot);
        }
        assert_eq!(frame, 1, "6 ticks total -> row 1");
        frame = 3;
        fc = 5.0;
        style44_find_frame(FLYING_FISH, 1.0, 0.0, 1, 4, &mut frame, &mut fc, &mut sd, &mut rot);
        assert_eq!(frame, 0, "row 4 wraps to 0 (:74781)");
    }
}
