//! The aiStyle-40 arm — the wall-crawler family (`NPC.AI()`'s
//! `else if (this.aiStyle == 40)` inline block, `NPC.cs:29666-29898`).
//!
//! W4-d26 (census 31-40, code-only mode): this module is the faithful
//! 1:1 port of the arm as a pure function over `Npc` + a small context
//! seam. It is NOT wired into the style axis yet — the five kinds are
//! unported in `NpcKind` (the style39 tortoise/shelly precedent: module
//! + witnesses first, the kind rows + dispatch claim in the wiring
//! round). Every other file is byte-identical except the one `mod` line
//! in `main.rs` and the anchors row.
//!
//! The five kinds on this style (`SetDefaults`, `NPC.cs` — all 36×36,
//! `noGravity = true`, `noTileCollide` stays FALSE: the collideX/Y
//! bounce below reads the tile pass):
//! * `NPCID.WallCreeperWall` 165 (`:11073-11087`): dmg 30, def 10,
//!   lifeMax 80, kb 0.25, value 100, npcSlots 0.5 → walk 164.
//! * `NPCID.JungleCreeperWall` 237 (`:12063-12077`): dmg 100, def 28,
//!   lifeMax 400, kb 0.25, value 1000, npcSlots 0.75 → walk 236; the
//!   3/0.12 chase pair.
//! * `NPCID.BlackRecluseWall` 238 (`:11044-11058`): dmg 100, def 40,
//!   lifeMax 350, kb 0.25, value 500, npcSlots 0.5 → walk 163.
//! * `NPCID.BloodCrawlerWall` 240 (`:12091-12104`): dmg 30, def 8,
//!   lifeMax 60, kb 0.5, value 130 → walk 239.
//! * `NPCID.DesertScorpionWall` 531 (`:14946-14959`): dmg 66, def 24,
//!   lifeMax 320, kb 0.5, value 600 → walk 530; the 4/0.16 chase pair
//!   and the +π/2 rotation offset (`:29812-29815`).
//!
//! The machine:
//! * `:29668-29671` — re-target when `target < 0 || == 255 || dead`,
//! * `:29672-29683` — the chase pair: 2/0.08 default, 237 → 3/0.12,
//!   531 → 4/0.16,
//! * `:29684-29692` — BOTH centres snapped to the 8-px grid
//!   (`(int)(v / 8f) * 8` — truncation, not floor),
//! * `:29693-29697` — confused flips the aim ×−2,
//! * `:29698-29710` — normalize to the chase speed; a zero delta keeps
//!   the raw velocity as the aim,
//! * `:29711-29715` — a dead player overrides the aim:
//!   `(direction * speed/2, -speed/2)`,
//! * `:29716` — `spriteDirection = -1` every tick,
//! * `:29717-29775` — `!CanHit` wander: `ai[0]++`, the ±0.023 vy/vx
//!   nudges by the `ai[0]` sign bands (>0 / ±100), the 200 → −200 wrap,
//!   the aim ×0.007 blend, `rotation = atan2(vy, vx)`, the four ×0.9
//!   >|1.5| damps and the four ±3 caps,
//! * `:29776-29811` — the CanHit chase: the per-axis accel with the
//!   DOUBLE step when the velocity crosses zero toward the aim,
//!   `rotation = atan2(aimY, aimX)`,
//! * `:29812-29815` — 531 adds π/2 to the rotation,
//! * `:29816-29842` — the collideX/Y bounces: `oldVelocity * -0.5`
//!   with the direction-gated ±2 X floors and the ±1.5 → ±2 Y floors,
//! * `:29843-29846` — a velocity sign flip vs `oldVelocity` stamps
//!   `netUpdate` unless `justHit`,
//! * `:29847-29868` — the expert WebSpit (472): the gate is
//!   `!confused && netMode != 1 && expertMode && target >= 0 &&
//!   !dead && (type == 163 || 238 || 236 || 237) && CanHit(centre, 1,
//!   1, playerCentre, 1, 1)` — the type list mixes the WALK ids
//!   163/236 with the wall ids 238/237 verbatim (only 237/238 can
//!   reach it inside this arm); `localAI[0]++`, the justHit
//!   `Next(20, 60)` subtraction floored at 0, then the
//!   `localAI[0] > Next(180, 900)` threshold → reset + the
//!   centre-anchored normalized ×8 spit at damage 18
//!   (`GetAttackDamage_ForProjectiles(18f, 18f)`, `:7052-7055` — the
//!   Remap of equal endpoints is the constant 18, a cited fold),
//! * `:29869-29892` — the walk transform: the switch maps
//!   165→164 / 237→236 / 238→163 / 240→239 / 531→530; on
//!   `localAI[1] == 0 && !NPCCanStickToWalls(walkId) &&
//!   TryMoveToTransformableSpaceInto(walkId)` the position moves to
//!   the adjusted top-left and `Transform(walkId, 0, 0, 0, 0,
//!   withReposition: false)` runs, then `localAI[1] = 12`,
//! * `:29894-29897` — the wall types decrement `localAI[1]` floored
//!   at 0 — AFTER a transform `this.type` is the WALK id, so the
//!   transform tick skips the decrement (the port's `transformed`
//!   flag mirrors the type swap).
//!
//! RNG census (the block DOES draw `Main.rand`, so the fn takes the
//! caller's stream): exactly TWO draw sites, both inside the expert
//! spit gate, in this order per qualifying tick:
//! 1. `:29854` `Next(20, 60)` — ONLY when `justHit` (the subtraction),
//! 2. `:29860` `Next(180, 900)` — EVERY gated tick (the threshold).
//! Classic difficulty draws NOTHING. `TargetClosest`
//! (`NPC.cs:77652-77781`), `Collision.CanHit` (`:381-479`),
//! `NPCCanStickToWalls` (`:55787-55833`), the size probes
//! (`Collision.cs:2455-2545`) and `Transform` (`:81084-81145` — the
//! walk `SetDefaults` rows are constants; `TransformVisuals`
//! `:80963+` has no branch for these five pairs) are all RNG-free.
//! The witnesses replay the cited draws by hand (WRITTEN NOT RUN —
//! code-only mode).
//!
//! Caller-owned (outside `AI()`): the tile-collision integration that
//! sets `collideX`/`collideY`/`oldVelocity` (`NPC.cs:90459-90498` —
//! noGravity true, noTileCollide false, fall-through-platforms TRUE
//! for aiStyle 40, `Collision_DecideFallThroughPlatforms`
//! `:92932-92935`), the spit's `NewProjectile` (the out request carries
//! the C# CENTRE coords; the drain's top-left convention applies) and
//! the full `Transform` — the out request carries the adjusted
//! top-left; the caller must run the `withReposition: false` variant
//! (NO bottom-edge Y shift — `transform_to_kind`'s default repositions,
//! so the wiring needs the false form), then re-stamp
//! `local_ai[1] = 12` (the C# order: SetDefaults zeroes localAI, THEN
//! `:29891` writes 12).
//!
//! Stand-ins (the style39 convention): `this.type` rides
//! [`Style40Type`] (the kinds are unported); `justHit` rides
//! `ctx.just_hit` (the wiring maps `hurt_cd > 0` — the documented
//! widening); `Main.expertMode`/`Main.netMode` ride the ctx (the
//! style30/32 wiring precedent); `EverLastingTicker` is the ticker-0
//! stand-in. `confused` reads the buff engine's Confused (31) directly
//! (`NPC.cs:92029-92032` — the `update_frost_legion` precedent).
#![allow(dead_code)] // wiring deferred (code-only mode) — the kind rows + dispatch claim are the follow-up round

use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::{UnifiedRandom, World};

/// The `this.type` axis of the arm — the five wall forms. The walk
/// forms (164/236/163/239/530) are aiStyle 3 and never run this arm.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style40Type {
    /// `NPCID.WallCreeperWall` 165 (`NPC.cs:11073-11087`).
    WallCreeperWall,
    /// `NPCID.JungleCreeperWall` 237 (`NPC.cs:12063-12077`).
    JungleCreeperWall,
    /// `NPCID.BlackRecluseWall` 238 (`NPC.cs:11044-11058`).
    BlackRecluseWall,
    /// `NPCID.BloodCrawlerWall` 240 (`NPC.cs:12091-12104`).
    BloodCrawlerWall,
    /// `NPCID.DesertScorpionWall` 531 (`NPC.cs:14946-14959`).
    DesertScorpionWall,
}

impl Style40Type {
    /// The C# `this.type` (the `== 237` / `== 531` / spit-gate
    /// comparisons read it).
    pub fn net_id(self) -> i32 {
        match self {
            Self::WallCreeperWall => 165,
            Self::JungleCreeperWall => 237,
            Self::BlackRecluseWall => 238,
            Self::BloodCrawlerWall => 240,
            Self::DesertScorpionWall => 531,
        }
    }

    /// The `:29869-29887` switch — the walk form each wall drops into.
    pub fn walk_id(self) -> i32 {
        match self {
            Self::WallCreeperWall => 164,
            Self::JungleCreeperWall => 236,
            Self::BlackRecluseWall => 163,
            Self::BloodCrawlerWall => 239,
            Self::DesertScorpionWall => 530,
        }
    }

    /// The walk form's `SetDefaults` lifeMax — the caller's
    /// `Transform` life rescale (`:81106`) needs it (the port's kind
    /// table has no walk rows yet).
    pub fn walk_life_max(self) -> i32 {
        match self {
            Self::WallCreeperWall => 80,   // :11065
            Self::JungleCreeperWall => 400, // :12056
            Self::BlackRecluseWall => 350, // :11036
            Self::BloodCrawlerWall => 60,  // :12085
            Self::DesertScorpionWall => 320, // :14940
        }
    }
}

/// The walk forms' shared `SetDefaults` hitbox — all five are 50×20
/// (`NPC.cs:11061-11062`/`:12051-12052`/`:11032-11033`/
/// `:12080-12081`/`:14935-14936`); the size probes read it.
const WALK_SIZE: (f32, f32) = (50.0, 20.0);

/// The env seam — the style32/39 convention.
#[derive(Clone, Copy, Debug)]
pub struct Style40Context {
    /// `this.type` — the wall form.
    pub kind: Style40Type,
    /// `Main.expertMode` (`:29849`) — the wiring reads the player's
    /// difficulty flag (the style30 precedent).
    pub expert: bool,
    /// `Main.netMode` (`:29847`/`:29849`) — 0 single-player.
    pub net_mode: i32,
    /// `this.justHit` (`:29843`/`:29852`) — the wiring maps
    /// `hurt_cd > 0` (the style39 documented widening).
    pub just_hit: bool,
    /// `Main.EverLastingTicker` for `TargetClosest`'s stamp — the
    /// ticker-0 stand-in (the `update_zombie` convention).
    pub ticker: u64,
}

/// The expert WebSpit request (`:29863-29866`) — the C# CENTRE coords
/// and the normalized ×8 velocity; damage is the constant-18 Remap
/// fold (`GetAttackDamage_ForProjectiles(18f, 18f)`, `:7052-7055`).
#[derive(Clone, Copy, Debug)]
pub struct SpitRequest {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub damage: i32,
}

/// The walk-transform request (`:29888-29892`) — the adjusted
/// top-left the C# position moves to BEFORE `Transform(walkId, 0, 0,
/// 0, 0, withReposition: false)`. The caller runs the full Transform
/// (life rescale via [`Style40Type::walk_life_max`], TargetClosest,
/// the ai/localAI zero loops) and THEN re-stamps `local_ai[1] = 12`
/// (the `:29891` post-SetDefaults order).
#[derive(Clone, Copy, Debug)]
pub struct TransformRequest {
    pub walk_net_id: i32,
    pub x: f32,
    pub y: f32,
}

/// The arm's out-requests (the style43 `queen_bee_ai` out convention).
#[derive(Clone, Copy, Debug, Default)]
pub struct Style40Out {
    pub spit: Option<SpitRequest>,
    pub transform: Option<TransformRequest>,
}

/// `NPC.NPCCanStickToWalls(Vector2 centerPos)` (`NPC.cs:55812-55833`):
/// the 3×3 tile scan around the centre's tile — a tile counts when
/// `(!active() || !tileSolid[type] || Platforms[type]) && wall > 0`;
/// MORE than 4 counting tiles sticks.
fn can_stick_to_walls_at(world: &World, cx: f32, cy: f32) -> bool {
    let tx = cx as i32 / 16;
    let ty = cy as i32 / 16;
    let mut count = 0;
    for i in (tx - 1)..=(tx + 1) {
        for j in (ty - 1)..=(ty + 1) {
            // `Framing.GetTileSafely` — out-of-world reads a fresh
            // (inactive, wall 0) tile.
            let t = world.tile(i, j).copied().unwrap_or_default();
            let open = !t.active()
                || !tr_core::tiledata::TILE_SOLID.has(t.kind)
                || tr_core::tiledata::SET_PLATFORMS.has(t.kind);
            if open && t.wall > 0 {
                count += 1;
                if count > 4 {
                    return true;
                }
            }
        }
    }
    false
}

/// `NPC.TryGetAdjustedHitboxFromTransformation(targetType)`
/// (`NPC.cs:81060-81071`) for the walk forms: the bottom-centre resize
/// probe (`Collision.TryChangingSizeFromBottomCenter`,
/// `Collision.cs:2486-2545` — the port's faithful seam). All five walk
/// ids exist in `ContentSamples.NpcsByNetId`, so only the collision
/// half can refuse.
fn adjusted_walk_hitbox(npc: &Npc, world: &World) -> Option<(f32, f32)> {
    let (w, h) = npc.size();
    crate::collision::try_changing_size_from_bottom_center(
        world,
        [npc.x, npc.y],
        [w, h],
        [WALK_SIZE.0, WALK_SIZE.1],
    )
    .map(|p| (p[0], p[1]))
}

/// `NPC.NPCCanStickToWalls(int transformID)` (`NPC.cs:55787-55810`):
/// probe the ADJUSTED hitbox's centre when the resize fits, else the
/// current centre. The temporary position/size swap is folded into
/// the centre arithmetic (the adjusted box's centre — no observable
/// state escapes the C# helper).
fn can_stick_to_walls(npc: &Npc, world: &World) -> bool {
    if let Some((ax, ay)) = adjusted_walk_hitbox(npc, world) {
        return can_stick_to_walls_at(
            world,
            ax + WALK_SIZE.0 * 0.5,
            ay + WALK_SIZE.1 * 0.5,
        );
    }
    let (w, h) = npc.size();
    can_stick_to_walls_at(world, npc.x + w * 0.5, npc.y + h * 0.5)
}

/// `NPC.AI()`'s aiStyle-40 arm (`NPC.cs:29666-29898`), one tick.
/// `players` is the target-scan slice (the session player is slot 0;
/// the axis arm stands an unacquired 255 target in as 0 first — the
/// limb family's mapping). The two RNG draws ride `rng` in C# order
/// (the census above); the spit/transform ride the out-requests.
pub fn style40_step(
    npc: &mut Npc,
    world: &World,
    players: &[TargetCandidate],
    ctx: &Style40Context,
    rng: &mut UnifiedRandom,
) -> Style40Out {
    let mut out = Style40Out::default();
    let ty = ctx.kind.net_id();
    // `:29668-29671` — the re-target gate.
    if npc.target < 0 || npc.target == 255 || players[npc.target as usize].dead {
        target_closest(npc, world, players, true, 40, ctx.ticker);
    }
    // `:29672-29683` — the chase pair.
    let mut num596 = 2.0f32;
    let mut num597 = 0.08f32;
    if ty == 237 {
        num596 = 3.0;
        num597 = 0.12;
    }
    if ty == 531 {
        num596 = 4.0;
        num597 = 0.16;
    }
    let p = &players[npc.target as usize];
    let (w, h) = npc.size();
    let pcx = p.x + (p.width / 2) as f32;
    let pcy = p.y + (p.height / 2) as f32;
    let cx = npc.x + w * 0.5;
    let cy = npc.y + h * 0.5;
    // `:29684-29692` — the 8-px grid snap of BOTH centres (the C#
    // `(int)` truncation, then the integer ×8).
    let snap = |v: f32| ((v / 8.0) as i32 * 8) as f32;
    let mut num598 = snap(pcx) - snap(cx);
    let mut num599 = snap(pcy) - snap(cy);
    // `:29693-29697` — the confused flip (the buff engine's 31).
    let confused = npc.buffs.find_buff_index(31).is_some();
    if confused {
        num598 *= -2.0;
        num599 *= -2.0;
    }
    // `:29698-29710` — normalize to the chase speed; the zero delta
    // keeps the raw velocity.
    let num600 = (num598 * num598 + num599 * num599).sqrt();
    if num600 == 0.0 {
        num598 = npc.vx;
        num599 = npc.vy;
    } else {
        let k = num596 / num600;
        num598 *= k;
        num599 *= k;
    }
    // `:29711-29715` — the dead-player drift.
    if p.dead {
        num598 = npc.facing as f32 * num596 / 2.0;
        num599 = (0.0 - num596) / 2.0;
    }
    // `:29716` — the constant sprite flip.
    npc.sprite_dir = -1;
    // `:29717` — the line-of-sight gate (the port's faithful CanHit).
    let los = crate::projectile::can_hit(
        world, npc.x, npc.y, w, h, p.x, p.y, p.width as f32, p.height as f32,
    );
    if !los {
        // `:29719-29739` — the wander nudges + the 200 → −200 wrap.
        npc.ai[0] += 1.0;
        if npc.ai[0] > 0.0 {
            npc.vy += 0.023;
        } else {
            npc.vy -= 0.023;
        }
        if npc.ai[0] < -100.0 || npc.ai[0] > 100.0 {
            npc.vx += 0.023;
        } else {
            npc.vx -= 0.023;
        }
        if npc.ai[0] > 200.0 {
            npc.ai[0] = -200.0;
        }
        // `:29740-29742` — the aim blend + the velocity rotation.
        npc.vx += num598 * 0.007;
        npc.vy += num599 * 0.007;
        npc.rotation = npc.vy.atan2(npc.vx);
        // `:29743-29774` — the four >|1.5| ×0.9 damps (the C#'s double
        // compares) then the four ±3 caps, in source order.
        if (npc.vx as f64) > 1.5 {
            npc.vx *= 0.9;
        }
        if (npc.vx as f64) < -1.5 {
            npc.vx *= 0.9;
        }
        if (npc.vy as f64) > 1.5 {
            npc.vy *= 0.9;
        }
        if (npc.vy as f64) < -1.5 {
            npc.vy *= 0.9;
        }
        if npc.vx > 3.0 {
            npc.vx = 3.0;
        }
        if npc.vx < -3.0 {
            npc.vx = -3.0;
        }
        if npc.vy > 3.0 {
            npc.vy = 3.0;
        }
        if npc.vy < -3.0 {
            npc.vy = -3.0;
        }
    } else {
        // `:29778-29809` — the per-axis accel with the double step on
        // the zero crossing toward the aim.
        if npc.vx < num598 {
            npc.vx += num597;
            if npc.vx < 0.0 && num598 > 0.0 {
                npc.vx += num597;
            }
        } else if npc.vx > num598 {
            npc.vx -= num597;
            if npc.vx > 0.0 && num598 < 0.0 {
                npc.vx -= num597;
            }
        }
        if npc.vy < num599 {
            npc.vy += num597;
            if npc.vy < 0.0 && num599 > 0.0 {
                npc.vy += num597;
            }
        } else if npc.vy > num599 {
            npc.vy -= num597;
            if npc.vy > 0.0 && num599 < 0.0 {
                npc.vy -= num597;
            }
        }
        // `:29810` — the AIM rotation (not the velocity's).
        npc.rotation = num599.atan2(num598);
    }
    // `:29812-29815` — the scorpion's quarter-turn offset.
    if ty == 531 {
        npc.rotation += std::f32::consts::PI / 2.0;
    }
    // `:29816-29842` — the collideX/Y bounces off the PREVIOUS tick's
    // velocity (the integration stamps collide_x/y + old_vx/vy).
    let num602 = 0.5f32;
    if npc.collide_x {
        npc.net_update = true;
        npc.vx = npc.old_vx * (0.0 - num602);
        if npc.facing == -1 && npc.vx > 0.0 && npc.vx < 2.0 {
            npc.vx = 2.0;
        }
        if npc.facing == 1 && npc.vx < 0.0 && npc.vx > -2.0 {
            npc.vx = -2.0;
        }
    }
    if npc.collide_y {
        npc.net_update = true;
        npc.vy = npc.old_vy * (0.0 - num602);
        if npc.vy > 0.0 && (npc.vy as f64) < 1.5 {
            npc.vy = 2.0;
        }
        if npc.vy < 0.0 && (npc.vy as f64) > -1.5 {
            npc.vy = -2.0;
        }
    }
    // `:29843-29846` — the sign-flip netUpdate (the four-way OR).
    if ((npc.vx > 0.0 && npc.old_vx < 0.0)
        || (npc.vx < 0.0 && npc.old_vx > 0.0)
        || (npc.vy > 0.0 && npc.old_vy < 0.0)
        || (npc.vy < 0.0 && npc.old_vy > 0.0))
        && !ctx.just_hit
    {
        npc.net_update = true;
    }
    // `:29847-29893` — the single-player block: the spit + the walk
    // transform.
    let mut transformed = false;
    if ctx.net_mode != 1 {
        // `:29849` — the spit gate (the type list keeps the WALK ids
        // 163/236 verbatim; only 237/238 reach it here).
        let spit_los = crate::projectile::can_hit(world, cx, cy, 1.0, 1.0, pcx, pcy, 1.0, 1.0);
        if !confused
            && ctx.net_mode != 1
            && ctx.expert
            && npc.target >= 0
            && !p.dead
            && (ty == 163 || ty == 238 || ty == 236 || ty == 237)
            && spit_los
        {
            npc.local_ai[0] += 1.0;
            // `:29852-29859` — draw 1: the justHit subtraction.
            if ctx.just_hit {
                npc.local_ai[0] -= rng.next_range(20, 60) as f32;
                if npc.local_ai[0] < 0.0 {
                    npc.local_ai[0] = 0.0;
                }
            }
            // `:29860-29867` — draw 2: the threshold, EVERY gated tick.
            if npc.local_ai[0] > rng.next_range(180, 900) as f32 {
                npc.local_ai[0] = 0.0;
                let mut dx = pcx - cx;
                let mut dy = pcy - cy;
                // `Vector2.Normalize()` then ×8 — the zero-length
                // guard is the port's house fold (the frost legion's
                // `inv` precedent; the C# would NaN).
                let len = (dx * dx + dy * dy).sqrt();
                if len > 0.0 {
                    let k = 8.0 / len;
                    dx *= k;
                    dy *= k;
                } else {
                    dx = 0.0;
                    dy = 0.0;
                }
                out.spit = Some(SpitRequest { x: cx, y: cy, vx: dx, vy: dy, damage: 18 });
            }
        }
        // `:29869-29887` — the switch (every Style40Type maps).
        let num604 = ctx.kind.walk_id();
        // `:29888-29892` — the transform attempt.
        if npc.local_ai[1] == 0.0 && !can_stick_to_walls(npc, world) {
            if let Some((ax, ay)) = adjusted_walk_hitbox(npc, world) {
                // `TryMoveToTransformableSpaceInto` (`:81073-81082`)
                // moves the position; `Transform(..., withReposition:
                // false)` keeps it. The full defaults swap is the
                // caller's (the out request); the module writes the
                // position + the post-Transform latch value.
                npc.x = ax;
                npc.y = ay;
                out.transform = Some(TransformRequest { walk_net_id: num604, x: ax, y: ay });
                npc.local_ai[1] = 12.0;
                transformed = true;
            }
        }
    }
    // `:29894-29897` — the latch decrement, wall types only: after a
    // transform `this.type` is the WALK id and the C# skips it.
    if !transformed {
        npc.local_ai[1] = (npc.local_ai[1] - 1.0).max(0.0);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;

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

    fn air() -> World {
        World::new("t".into(), 400, 400)
    }

    fn ctx(kind: Style40Type) -> Style40Context {
        Style40Context { kind, expert: false, net_mode: 0, just_hit: false, ticker: 0 }
    }

    /// A wall crawler at (2000, 1000), 36×36 (the unported size rides
    /// `size_override` — the style39 convention), target acquired.
    fn crawler() -> Npc {
        let mut n = Npc::new(NpcKind::Creeper, 2000.0, 1000.0, 1, 0.0);
        n.size_override = Some((36.0, 36.0));
        n.hp = n.max_hp;
        n.target = 0;
        n
    }

    /// `:29672-29683` + `:29776-29811` — the per-type chase pairs: the
    /// CanHit accel steps 0.08 / 0.12 / 0.16 toward the snapped aim.
    #[test]
    fn the_chase_pairs_accel_per_type() {
        let w = air();
        // Player far right, same row: the aim is (+speed, ~0).
        let p = [cand(2600.0, 1000.0)];
        let mut rng = UnifiedRandom::new(1);
        for (kind, accel) in [
            (Style40Type::WallCreeperWall, 0.08f32),
            (Style40Type::JungleCreeperWall, 0.12),
            (Style40Type::DesertScorpionWall, 0.16),
        ] {
            let mut n = crawler();
            style40_step(&mut n, &w, &p, &ctx(kind), &mut rng);
            assert_eq!(n.vx, accel, "{kind:?} first-tick accel");
            assert_eq!(n.sprite_dir, -1, "the constant sprite flip (:29716)");
        }
        assert_eq!(rng, UnifiedRandom::new(1), "classic draws nothing");
    }

    /// `:29684-29692` — both centres snap to the 8-px grid BEFORE the
    /// delta: a 7-px offset snaps away to zero (the aim keeps the raw
    /// velocity), an 8-px offset survives.
    #[test]
    fn the_aim_snaps_both_centres_to_the_eight_px_grid() {
        let w = air();
        let mut rng = UnifiedRandom::new(1);
        // The crawler's centre is (2018, 1018) — snapped (2016, 1016).
        // A player centre 7 px right of the snapped centre lands in
        // the SAME cell: zero delta → the raw velocity is the aim.
        let mut n = crawler();
        n.vx = 1.25;
        n.vy = -0.5;
        // pcx = 2016 + 7 = 2023 → x = 2013; pcy = 1016 → y = 995.
        let p = [cand(2013.0, 995.0)];
        style40_step(&mut n, &w, &p, &ctx(Style40Type::WallCreeperWall), &mut rng);
        // The zero-delta aim (1.25, -0.5): vx 1.25 > aim 1.25? No —
        // equal, no accel; vy -0.5 == aim, no accel.
        assert_eq!(n.vx, 1.25);
        assert_eq!(n.vy, -0.5);
        assert_eq!(n.rotation, (-0.5f32).atan2(1.25), "the aim rotation");
    }

    /// `:29693-29697` — confused flips the aim ×−2: the crawler
    /// accelerates AWAY from the player at the doubled magnitude.
    #[test]
    fn the_confused_aim_reverses_and_doubles() {
        let w = air();
        let p = [cand(2600.0, 1000.0)];
        let mut n = crawler();
        n.buffs.add_buff(31, 600);
        let mut rng = UnifiedRandom::new(1);
        style40_step(&mut n, &w, &p, &ctx(Style40Type::WallCreeperWall), &mut rng);
        // The aim is negative → vx 0 > aim decelerates by 0.08 (no
        // double step: vx stays > 0 after one step? 0 - 0.08 = -0.08
        // < 0 — the double-step gate needs vx > 0 AFTER, so once).
        assert_eq!(n.vx, -0.08);
        // The rotation points away (π, the negative-X aim).
        assert!((n.rotation.abs() - std::f32::consts::PI).abs() < 0.2, "{}", n.rotation);
    }

    /// `:29711-29715` — a dead player overrides the aim with
    /// `(direction * speed/2, -speed/2)`.
    #[test]
    fn the_dead_player_drifts_at_half_speed() {
        let w = air();
        let mut p = cand(2600.0, 1000.0);
        p.dead = true;
        let mut n = crawler();
        n.facing = -1;
        let mut rng = UnifiedRandom::new(1);
        style40_step(&mut n, &w, &[p], &ctx(Style40Type::JungleCreeperWall), &mut rng);
        // speed 3: the aim (-1.5, -1.5); vx 0 > -1.5 → -0.12, vy same.
        assert_eq!(n.vx, -0.12);
        assert_eq!(n.vy, -0.12);
        assert_eq!(n.rotation, (-1.5f32).atan2(-1.5));
    }

    /// `:29717-29775` — the !CanHit wander: a solid column between the
    /// pair breaks the line; the ai[0] bands nudge ±0.023, the aim
    /// blends ×0.007, the rotation reads the VELOCITY.
    #[test]
    fn the_blocked_wander_nudges_and_blends() {
        let mut w = World::new("t".into(), 200, 200);
        for y in 0..200 {
            let i = w.index(100, y);
            w.tiles[i] = tr_core::tile::Tile { kind: 1, ..Default::default() };
            w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
        let mut n = crawler();
        n.x = 90.0 * 16.0;
        n.y = 100.0 * 16.0;
        let p = [cand(110.0 * 16.0, 100.0 * 16.0)];
        let mut rng = UnifiedRandom::new(1);
        let vx0 = n.vx;
        let vy0 = n.vy;
        style40_step(&mut n, &w, &p, &ctx(Style40Type::WallCreeperWall), &mut rng);
        assert_eq!(n.ai[0], 1.0, "the wander counter");
        // ai[0] 1 > 0 → vy += 0.023; |ai[0]| <= 100 → vx -= 0.023;
        // then the aim (positive X) blends ×0.007.
        let aim_x = 2.0f32; // the snapped delta normalizes to speed 2
        assert!((n.vx - (vx0 - 0.023 + aim_x * 0.007)).abs() < 1e-4, "{}", n.vx);
        assert!((n.vy - (vy0 + 0.023)).abs() < 1e-4, "{}", n.vy);
        assert_eq!(n.rotation, n.vy.atan2(n.vx), "the velocity rotation");
    }

    /// `:29728-29739` + `:29743-29774` — the 200 → −200 wrap and the
    /// damp/cap ladder.
    #[test]
    fn the_wander_wraps_at_200_and_caps_at_three() {
        let mut w = World::new("t".into(), 200, 200);
        for y in 0..200 {
            let i = w.index(100, y);
            w.tiles[i] = tr_core::tile::Tile { kind: 1, ..Default::default() };
            w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
        let p = [cand(110.0 * 16.0, 100.0 * 16.0)];
        let mut n = crawler();
        n.x = 90.0 * 16.0;
        n.y = 100.0 * 16.0;
        n.ai[0] = 200.0;
        n.vx = 10.0;
        n.vy = -10.0;
        let mut rng = UnifiedRandom::new(1);
        style40_step(&mut n, &w, &p, &ctx(Style40Type::WallCreeperWall), &mut rng);
        assert_eq!(n.ai[0], -200.0, "the wrap (:29736-29739)");
        // vx: 10 + 0.023 (ai[0] 201 > 100) + aim blend, ×0.9 (>1.5),
        // then the 3 cap. vy: -10 + 0.023 + blend, ×0.9, the −3 cap.
        assert_eq!(n.vx, 3.0);
        assert_eq!(n.vy, -3.0);
    }

    /// `:29812-29815` — only 531 adds π/2 to the rotation.
    #[test]
    fn the_scorpion_adds_a_quarter_turn() {
        let w = air();
        let p = [cand(2600.0, 1000.0)];
        let mut rng = UnifiedRandom::new(1);
        let mut n = crawler();
        style40_step(&mut n, &w, &p, &ctx(Style40Type::DesertScorpionWall), &mut rng);
        let base = 0.0f32.atan2(2.0); // the aim is straight right
        assert!((n.rotation - (base + std::f32::consts::PI / 2.0)).abs() < 0.2);
        let mut n2 = crawler();
        style40_step(&mut n2, &w, &p, &ctx(Style40Type::WallCreeperWall), &mut rng);
        assert!(n2.rotation.abs() < 0.2, "no offset for the others");
    }

    /// `:29816-29842` — the collideX/Y bounces: −0.5 × oldVelocity
    /// with the direction-gated floors.
    #[test]
    fn the_bounces_reflect_old_velocity_with_floors() {
        let w = air();
        let p = [cand(2600.0, 1000.0)];
        let mut rng = UnifiedRandom::new(1);
        let mut n = crawler();
        n.collide_x = true;
        n.collide_y = true;
        n.old_vx = -1.0; // → +0.5, facing +1: no floor (needs facing -1)
        n.old_vy = -1.0; // → +0.5 < 1.5 → the 2 floor
        n.facing = 1;
        style40_step(&mut n, &w, &p, &ctx(Style40Type::WallCreeperWall), &mut rng);
        // The chase accel ran first (vx 0 → 0.08), then the bounce
        // overwrote it with the reflection.
        assert_eq!(n.vx, 0.5);
        assert_eq!(n.vy, 2.0, "the 1.5 → 2 floor (:29834-29837)");
        assert!(n.net_update);
        // facing -1 with a small positive reflection floors at +2.
        let mut n2 = crawler();
        n2.collide_x = true;
        n2.old_vx = -1.0;
        n2.facing = -1;
        style40_step(&mut n2, &w, &p, &ctx(Style40Type::WallCreeperWall), &mut rng);
        assert_eq!(n2.vx, 2.0, "the direction-gated X floor (:29821-29824)");
    }

    /// `:29843-29846` — the sign-flip netUpdate, gated on !justHit.
    #[test]
    fn the_sign_flip_stamps_net_update_unless_just_hit() {
        let w = air();
        let p = [cand(2600.0, 1000.0)];
        let mut rng = UnifiedRandom::new(1);
        let mut n = crawler();
        n.old_vx = -1.0; // the chase makes vx positive → the flip
        style40_step(&mut n, &w, &p, &ctx(Style40Type::WallCreeperWall), &mut rng);
        assert!(n.net_update);
        let mut n2 = crawler();
        n2.old_vx = -1.0;
        let mut c = ctx(Style40Type::WallCreeperWall);
        c.just_hit = true;
        style40_step(&mut n2, &w, &p, &c, &mut rng);
        assert!(!n2.net_update, "justHit suppresses the stamp");
    }

    /// RNG census: classic expert-off ticks draw NOTHING; an expert
    /// 237/238 tick draws exactly the ONE threshold `Next(180, 900)`;
    /// justHit adds the `Next(20, 60)` subtraction FIRST (the C#
    /// order). The 165/240/531 types never enter the gate.
    #[test]
    fn the_expert_spit_census_replays_the_two_draws() {
        let w = air();
        let p = [cand(2600.0, 1000.0)];
        // Expert, no justHit: one draw.
        let mut n = crawler();
        let mut c = ctx(Style40Type::JungleCreeperWall);
        c.expert = true;
        let mut rng = UnifiedRandom::new(9);
        let mut witness = UnifiedRandom::new(9);
        let _threshold = witness.next_range(180, 900);
        style40_step(&mut n, &w, &p, &c, &mut rng);
        assert_eq!(rng, witness, "exactly the threshold draw (:29860)");
        assert_eq!(n.local_ai[0], 1.0, "the counter advanced");
        // Expert + justHit: the subtraction FIRST, then the threshold.
        let mut n2 = crawler();
        n2.local_ai[0] = 500.0;
        c.just_hit = true;
        let mut rng2 = UnifiedRandom::new(9);
        let mut witness2 = UnifiedRandom::new(9);
        let sub = witness2.next_range(20, 60);
        let th = witness2.next_range(180, 900);
        style40_step(&mut n2, &w, &p, &c, &mut rng2);
        assert_eq!(rng2, witness2, "the subtraction precedes the threshold");
        let after_sub = 501.0 - sub as f32;
        let want = if after_sub > th as f32 { 0.0 } else { after_sub };
        assert_eq!(n2.local_ai[0], want);
        // The non-spit types draw nothing even in expert.
        let mut n3 = crawler();
        let mut c3 = ctx(Style40Type::WallCreeperWall);
        c3.expert = true;
        let mut rng3 = UnifiedRandom::new(9);
        style40_step(&mut n3, &w, &p, &c3, &mut rng3);
        assert_eq!(rng3, UnifiedRandom::new(9), "165 is outside the gate");
    }

    /// `:29852-29859` — the justHit subtraction floors at 0.
    #[test]
    fn the_just_hit_subtraction_floors_at_zero() {
        let w = air();
        let p = [cand(2600.0, 1000.0)];
        let mut n = crawler();
        n.local_ai[0] = 0.0;
        let mut c = ctx(Style40Type::BlackRecluseWall);
        c.expert = true;
        c.just_hit = true;
        let mut rng = UnifiedRandom::new(3);
        style40_step(&mut n, &w, &p, &c, &mut rng);
        // 0 + 1 - Next(20,60) < 0 → clamped 0 (never the threshold).
        assert_eq!(n.local_ai[0], 0.0);
    }

    /// `:29860-29867` — a counter above ANY threshold fires: the
    /// request carries the centre, the normalized ×8 velocity and the
    /// constant-18 damage; the counter resets.
    #[test]
    fn the_spit_fires_at_the_centre_normalized_times_eight() {
        let w = air();
        let p = [cand(2600.0, 1000.0)];
        let mut n = crawler();
        n.local_ai[0] = 10000.0; // > any Next(180, 900)
        let mut c = ctx(Style40Type::JungleCreeperWall);
        c.expert = true;
        let mut rng = UnifiedRandom::new(5);
        let out = style40_step(&mut n, &w, &p, &c, &mut rng);
        let spit = out.spit.expect("the threshold is always crossed");
        assert_eq!(n.local_ai[0], 0.0, "the reset (:29862)");
        assert_eq!((spit.x, spit.y), (2018.0, 1018.0), "the C# centre");
        assert_eq!(spit.damage, 18, "the equal-endpoint Remap fold");
        let sp = (spit.vx * spit.vx + spit.vy * spit.vy).sqrt();
        assert!((sp - 8.0).abs() < 1e-3, "the ×8 normalize ({sp})");
        assert!(spit.vx > 0.0, "toward the player");
    }

    /// `:29888-29892` — open space transforms: the adjusted top-left
    /// (the 36×36 → 50×20 bottom-centre resize: y +16, x −7), the
    /// walk id, the 12 latch; the transform tick skips the decrement.
    #[test]
    fn the_open_space_transform_moves_and_latches() {
        let w = air();
        let p = [cand(2600.0, 1000.0)];
        let mut n = crawler();
        let mut rng = UnifiedRandom::new(1);
        let out = style40_step(&mut n, &w, &p, &ctx(Style40Type::WallCreeperWall), &mut rng);
        let t = out.transform.expect("no walls → the drop");
        assert_eq!(t.walk_net_id, 164);
        // dh = 20 − 36 = −16 → y += 16; dw = 50 − 36 = 14 → the even
        // split x −= 7 (the empty world has full room both ways).
        assert_eq!((t.x, t.y), (2000.0 - 7.0, 1000.0 + 16.0));
        assert_eq!((n.x, n.y), (t.x, t.y), "TryMove ran before Transform");
        assert_eq!(n.local_ai[1], 12.0, "the latch, no same-tick decrement");
        assert_eq!(rng, UnifiedRandom::new(1), "the transform is RNG-free");
    }

    /// `:55812-55833` — five wall-backed open tiles around the
    /// ADJUSTED centre stick the crawler to the wall: no transform,
    /// and the latch decrements toward 0.
    #[test]
    fn five_wall_tiles_stick_and_block_the_transform() {
        let mut w = air();
        // The adjusted centre is (2018, 1026) → tile (126, 64); back
        // five of its 3×3 with walls (inactive tiles count).
        for (i, j) in [(125, 63), (126, 63), (127, 63), (125, 64), (126, 64)] {
            let idx = w.index(i, j);
            w.tiles[idx].wall = 4;
        }
        let p = [cand(2600.0, 1000.0)];
        let mut n = crawler();
        n.local_ai[1] = 3.0;
        let mut rng = UnifiedRandom::new(1);
        let out = style40_step(&mut n, &w, &p, &ctx(Style40Type::WallCreeperWall), &mut rng);
        assert!(out.transform.is_none(), ">4 wall tiles stick (:55825)");
        assert_eq!(n.local_ai[1], 2.0, "the decrement (:29896)");
        // Four tiles do NOT stick.
        let idx = w.index(126, 64);
        w.tiles[idx].wall = 0;
        let mut n2 = crawler();
        let out2 = style40_step(&mut n2, &w, &p, &ctx(Style40Type::WallCreeperWall), &mut rng);
        assert!(out2.transform.is_some(), "4 is not > 4");
    }

    /// `:29894-29897` — the latch floors at 0 on non-transform ticks.
    #[test]
    fn the_latch_decrement_floors_at_zero() {
        let mut w = air();
        for (i, j) in [(125, 63), (126, 63), (127, 63), (125, 64), (126, 64)] {
            let idx = w.index(i, j);
            w.tiles[idx].wall = 4;
        }
        let p = [cand(2600.0, 1000.0)];
        let mut n = crawler();
        n.local_ai[1] = 0.0;
        let mut rng = UnifiedRandom::new(1);
        style40_step(&mut n, &w, &p, &ctx(Style40Type::WallCreeperWall), &mut rng);
        assert_eq!(n.local_ai[1], 0.0, "max(0, -1)");
    }

    /// `:29668-29671` — the unacquired 255 target re-targets (the
    /// gate short-circuits before the slice index).
    #[test]
    fn the_unacquired_target_retargets() {
        let w = air();
        let p = [cand(2600.0, 1000.0)];
        let mut n = crawler();
        n.target = 255;
        n.facing = -1;
        let mut rng = UnifiedRandom::new(1);
        style40_step(&mut n, &w, &p, &ctx(Style40Type::WallCreeperWall), &mut rng);
        assert_eq!(n.target, 0);
        assert_eq!(n.facing, 1, "TargetClosest faces the player");
    }
}
