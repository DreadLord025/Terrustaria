//! The aiStyle-39 family arm — the tortoises / giant shellies / Solar
//! Sroller (`NPC.AI()`'s `else if (this.aiStyle == 39)` block,
//! `NPC.cs:29145-29665`).
//!
//! S1-D-3 (aiStyle-39 round, code-only mode): this module is the faithful
//! 1:1 port of the arm as a pure function over `Npc` + a small context
//! seam. WIRED (S1-D-4 W-10): `npc.rs::update_solar_sroller` drives it
//! over the invented v1; W4-g9 moved that SAME wrapper onto the
//! aiStyle-39 axis arm (the kind match's sroller arm is an
//! `unreachable!` marker now). The sibling D-3 rounds land the same way:
//! module + tests first, dispatch claim second — the `npc_ai_style28.rs`
//! pattern, commit `040df4c5`.
//!
//! The five kinds on this style (`SetDefaults`, `NPC.cs`):
//! * `NPCID.GiantTortoise` 153 (`:10930-10943`): 46×32, dmg 80, def 30,
//!   lifeMax 470, kb 0.3, `noGravity = false`.
//! * `NPCID.IceTortoise` 154 (`:10945-10959`): 46×32, dmg 55, def 28,
//!   lifeMax 400, kb 0.3, `noGravity = false`, coldDamage.
//! * `NPCID.GiantShelly`/`GiantShelly2` 496/497 (`:14678-14689`): 28×22,
//!   dmg 16, def 12, lifeMax 50, kb 0.75 — the `type >= 496 && <= 497`
//!   branches below (the 0.5 walk cap, the +2/+4 aggro, the 0.5-tick
//!   counters, the 1.5× lunge damage, the ×0.75 lunge speed, the state-0
//!   early return).
//! * `NPCID.SolarSroller` 417 (`:15355-15367`): 46×52, dmg 80, def 34,
//!   lifeMax 700, kb 0 — the ONLY kind that takes the ball states 6/5
//!   (`:29412-29419` collapses 52 → 32, `:29636-29641` restores).
//! The port has only `NpcKind::SolarSroller` today; the type branches ride
//! [`Style39Type`] until the wiring round adds the tortoise/shelly kinds
//! (the caller sets `size_override` for the unported sizes).
//!
//! RNG by construction: the arm DOES draw `Main.rand`, so the fn takes
//! `rng: &mut UnifiedRandom` — and W4-e2 wires the six dust sites
//! (29214/29427/29540/29555/29558/29572) through the S1-E-1 `DustPool`
//! (`new_dust`) on that SAME stream, exactly like the C#'s `Main.dust`.
//! The four gore bodies (`:29564`) stay W4-d55's lane: their ARGUMENT
//! draws are consumed here in order (the twins' spin hand-off split).
//! Every draw is consumed in C# order, cited at its site:
//! * `:29212` `Next(10) == 0` — the 154 head dust gate; on 0 the
//!   `NewDust` body draws (type 67 hits no per-type tweak, `Dust.cs:214-252`).
//! * `:29418` `Next(2, 5)` — the 417 ball revolutions (2..=4).
//! * `:29424` `Next(3) < 2` — the 154 lunge dust gate + the same 67 body.
//! * `:29537` `Next(3) < 2` — the 417 ball dust gate; the type-6 body
//!   ADDS the `Next(-10, 6)` vy re-roll (`Dust.cs:215-219` — 6 is in the
//!   tweak set) after the 7 default-block draws.
//! * `:29547-29581` the `ai[3] > 0` FX block. On `ai[3] == 1` FIRST the
//!   32 dust PAIRS (each: a scale-2.5 spawn then a scale-1.5 spawn, both
//!   type 6 → 8 draws each, `:29555-29559`), THEN the 4 gores — each
//!   `Next(100)` ×2 (the spawn offsets), `Next(61, 64)` (the type), the
//!   `NewGore` body's `Next(10, 31)` + `Next(-20, 21)` (`Gore.cs:1327-1328`;
//!   61-63 are safe/sticky/drip-free types — no further draws), then
//!   `Next(-10, 11)` ×2 (the velocity nudges) — 7 draws ×4. Then, EVERY
//!   `ai[3] > 0` tick, the 5 type-31 dust spawns (7 draws each, no tweak)
//!   each followed by its `NextFloat()` velocity scale (`:29573` — the
//!   float draw runs even when `NewDust` refuses the spawn).
//! The `netOffset` round trips around the spawns are kept VERBATIM (not
//! folded into the spawn argument) so the float rounding of the restored
//! position matches; zero in single-player. `TargetClosest`
//! (`NPC.cs:77652-77781`) is RNG-free. The stream witnesses below replay
//! the cited draws by hand (WRITTEN NOT RUN — code-only mode).
//!
//! Faithfulness notes vs the live `update_solar_sroller` (the wiring
//! round replaces it with this arm):
//! * The wind-up (`ai[0] == 1`) counts `ai[1]++` for 417 (`:29403`) — the
//!   live arm adds 0.5 (that is the SHELLY rate, `:29399`).
//! * The ball speed is the AIMED launch (`:29582-29611`, 16/10 by CanHit,
//!   the `|dx| * 0.2` arc offset gated on `directionY > 0`) — the live arm
//!   invents `5 + ai[2] * 0.6` per tick.
//! * The ball exit is `ai[2] == 0 || ai[1] >= 1200` (`:29622`) — the live
//!   arm invents `ai[1] >= 60 * ai[2]`. The `ai[2]` revolution decrement
//!   is NOT in the AI arm: it lives in `Collision_MoveSolarSroller`
//!   (`NPC.cs:92736-92757`, the bounce pass — `ai[2]--`, `ai[3] = 1`, the
//!   ×0.9 `oldVelocity` reflection + direction flip while `ai[2] > 0`),
//!   which the wiring round ports with the collision seam.
//! * The ball entry/unroll adjust `position.Y` around the height change
//!   (`:29414-29416`, `:29638-29640`) — the live arm never changes height.
//! * The damage/defense/knockback writes (`:29262-29271`, `:29437-29438`,
//!   `:29534-29536`) ride the difficulty helpers (`:7031-7034`,
//!   `:7047-7050`) — the live arm writes only a hardcoded defense 68.
//! * The ball rotation is `Clamp(vx / 10 * direction, ±π/10)` (`:29632`)
//!   — the live arm uses `vx * 0.05`.
//! * The head's step-up probe (`:29161-29202`, the six-tile column, the
//!   halfbrick ±8, the ≤16.1 lift, stepSpeed 0.75/1.5), the `justHit`
//!   reset (`:29203-29208`, `type != 417`), the cliff probe
//!   (`:29282-29314`), the wet gates and the CanHit launch gates are all
//!   absent from the live arm.
//!
//! Caller-owned (outside `AI()`, documented like style28's integration
//! note): the position integration + gravity (`NPC.cs:90459-90472`), the
//! `gfxOffY` ease-down (`:90396-90419`), the sroller bounce pass
//! (`:92736-92757`), and `FindFrame`'s style-39 tail (`:77292-77368`:
//! `type != 417` zeroes the rotation, `frameCounter += |vx| * 1.1`, the
//! 6-px frame bands). The `noGravity = false` write (`:29496`) is a
//! redundant reset — all five kinds are `noGravity = false` by default
//! and no code in the style arms it true — so it is a cited no-op here
//! (the port's gravity pipeline always applies gravity to these kinds).
//! The `position ± netOffset` pairs around the dust spawns
//! (`:29211/:29218`, `:29426/:29430`, `:29539/:29544`, `:29549/:29580`)
//! run verbatim — the spawn reads the offset position, then the position
//! is restored by subtraction (zero net effect in single-player).
//!
//! Stand-ins (the port's `Npc` has no such fields, same convention as
//! style28's context): `gfxOffY`/`stepSpeed`/`knockBackResist` ride
//! [`Style39Context`] (the C# fields `NPC.cs:6095/:6097` + the
//! `knockBackResist` write sites); `justHit` rides `ctx.just_hit` (the
//! wiring maps `hurt_cd > 0` — a documented widening: the C# flag lasts
//! one tick, cleared at `NPC.cs:90516`, the port's cooldown ~11);
//! `this.type` rides `ctx.kind`; `defDamage`/`defDefense` ride the
//! [`Style39Type`] table (the `SetDefaults` rows above — the C# copies
//! damage/defense into them at the end of `SetDefaults`).
#![allow(dead_code)] // wiring deferred (code-only mode) — the dispatch claim is the follow-up round

use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::tile::Tile;
use tr_core::tiledata as td;
use tr_core::{UnifiedRandom, World};

/// The `this.type` axis of the arm. The port has only
/// `NpcKind::SolarSroller` today; the wiring round maps the kind (or the
/// future tortoise/shelly kinds) onto this enum.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style39Type {
    /// `NPCID.GiantTortoise` 153 (`NPC.cs:10930-10943`).
    GiantTortoise,
    /// `NPCID.IceTortoise` 154 (`NPC.cs:10945-10959`).
    IceTortoise,
    /// `NPCID.GiantShelly`/`GiantShelly2` 496/497 (`NPC.cs:14678-14689`)
    /// — the arm's `type >= 496 && type <= 497` branches treat the pair
    /// identically, so one variant covers both.
    GiantShelly,
    /// `NPCID.SolarSroller` 417 (`NPC.cs:15355-15367`).
    SolarSroller,
}

impl Style39Type {
    /// The C# `this.type` (the `== 154` / `== 417` / `>= 496 && <= 497`
    /// comparisons below read it through the helpers).
    pub fn net_id(self) -> i32 {
        match self {
            Self::GiantTortoise => 153,
            Self::IceTortoise => 154,
            // 496 stands for the pair — no arm distinguishes 496 from 497.
            Self::GiantShelly => 496,
            Self::SolarSroller => 417,
        }
    }

    /// `this.defDamage` — the `SetDefaults` damage row (the C# copies
    /// `damage` into `defDamage` at the end of `SetDefaults`).
    pub fn def_damage(self) -> i32 {
        match self {
            Self::GiantTortoise => 80,  // :10936
            Self::IceTortoise => 55,    // :10951
            Self::GiantShelly => 16,    // :14682
            Self::SolarSroller => 80,   // :15360
        }
    }

    /// `this.defDefense` — the `SetDefaults` defense row.
    pub fn def_defense(self) -> i32 {
        match self {
            Self::GiantTortoise => 30,  // :10937
            Self::IceTortoise => 28,    // :10952
            Self::GiantShelly => 12,    // :14683
            Self::SolarSroller => 34,   // :15361
        }
    }

    /// The `type >= 496 && type <= 497` branch axis.
    fn is_shelly(self) -> bool {
        matches!(self, Self::GiantShelly)
    }
}

/// The arm's inputs + the three C# NPC fields the port's `Npc` does not
/// carry (the stand-in note in the module doc). The caller fills the
/// inputs each tick and reads the outputs back.
#[derive(Clone, Copy, Debug)]
pub struct Style39Context {
    /// `this.type` — the per-kind branch axis.
    pub kind: Style39Type,
    /// `NPC.difficulty` (`NPC.cs:6535`, default 1 = Classic;
    /// `GameDifficultyLevel.cs:17-21`).
    pub difficulty: f32,
    /// `this.justHit` (`NPC.cs:29203/:29278`) — the wiring maps the
    /// port's `hurt_cd > 0` (the documented one-tick widening).
    pub just_hit: bool,
    /// `Main.EverLastingTicker` for the `targetSetFrame` stamp (the same
    /// stand-in as style28/`update_zombie`).
    pub ticker: u64,
    /// `NPC.gfxOffY` (`NPC.cs:6095`) — IN/OUT: the step-up probe adds the
    /// lift (`:29189`); the caller owns the ease-down (`:90396-90419`).
    pub gfx_off_y: f32,
    /// `NPC.stepSpeed` (`NPC.cs:6097`) — OUT: 0.75/1.5 (`:29193/:29197`).
    pub step_speed: f32,
    /// `NPC.knockBackResist` — OUT: the state-0 difficulty-scaled write
    /// (`:29266/:29270`) and the ball's 0 (`:29536`).
    pub knockback_resist: f32,
}

/// `NPC.GetAttackDamage_LerpBetweenFinalValues` (`NPC.cs:7047-7050`):
/// `(int)Utils.Remap(difficulty, Classic, Expert, normal, expert)` —
/// `Remap` (`Utils.cs:345-348`) is `MathHelper.Lerp(toMin, toMax,
/// GetLerpValue(fromMin, fromMax, value, clamped: true))`, and
/// `GetLerpValue` (`Utils.cs:315-343`) clamps then divides;
/// `MathHelper.Lerp` is `value1 + (value2 - value1) * amount`. Classic =
/// 1, Expert = 2 (`GameDifficultyLevel.cs:18-19`).
fn attack_damage_lerp(difficulty: f32, normal: f32, expert: f32) -> i32 {
    let t = if difficulty < 1.0 {
        0.0
    } else if difficulty > 2.0 {
        1.0
    } else {
        (difficulty - 1.0) / (2.0 - 1.0)
    };
    (normal + (expert - normal) * t) as i32
}

/// `GameDifficultyData.KnockbackToEnemiesMultiplier.Sample(difficulty)`
/// (`GameDifficultyData.cs:86`: the keys Classic(1) → 1, Master(3) → 0.8;
/// `LinearCurve.Sample` `:35-54`: the bracket scan, the zero-span guard
/// returns `key.output`, else `(value - key.input) * num2 / num +
/// key.output` — the operation order kept verbatim).
fn knockback_multiplier(difficulty: f32) -> f32 {
    if difficulty <= 1.0 {
        1.0 // the first-key zero-span arm (:50-52)
    } else if difficulty >= 3.0 {
        0.8 // past the last key both cursors sit on it → zero span
    } else {
        let num = 3.0f32 - 1.0f32;
        let num2 = 0.8f32 - 1.0f32;
        (difficulty - 1.0) * num2 / num + 1.0
    }
}

// The tile predicates the step-up/cliff probes read. The C# reads the
// RAW `Main.tileSolid`/`Main.tileSolidTop` tables here (no `inActive()`
// conjunct — unlike `Collision.StepUp`), so these mirror the tables, not
// `collision.rs::solid` (which adds the actuated gate). `tileSolid[192]`
// is false at runtime (`WorldGen.cs:15715`, the living-tree leaves).

/// `Main.tileSolid[type]` (with the `WorldGen.cs:15715` leaf override).
fn tile_solid(kind: u16) -> bool {
    (kind as usize) < td::TILE_COUNT && td::TILE_SOLID.has(kind) && kind != 192
}

/// `Main.tileSolidTop[type]`.
fn tile_solid_top(kind: u16) -> bool {
    td::TILE_SOLID_TOP.has(kind)
}

/// `Tile.nactive()` (`Tile.cs:240-247`: the active bit set, the inactive
/// bit clear).
fn nactive(t: &Tile) -> bool {
    t.active() && !t.inactive()
}

/// `Tile.topSlope()` (`Tile.cs:290-297`: slope 1 or 2).
fn top_slope(t: &Tile) -> bool {
    let s = t.slope();
    s == 1 || s == 2
}

/// `Framing.GetTileSafely(i, j)` (`Framing.cs:426-439`): out-of-world
/// returns a fresh (inactive) tile — the port's `Tile::default()`.
fn tile_safely(world: &World, x: i32, y: i32) -> Tile {
    world.tile(x, y).copied().unwrap_or_default()
}

/// The head's inline step-up probe (`NPC.cs:29151-29202`) — NOT
/// `Collision.StepUp` (that one uses stepSpeed 1/2, `NPC.cs:59938/:59942`;
/// this probe writes 0.75/1.5). The six-tile column ahead of the next
/// position, the halfbrick ±8 surface offsets, the ≤16.1 lift.
fn step_up_probe(npc: &mut Npc, world: &World, ctx: &mut Style39Context) {
    let (w, h) = npc.size();
    let wi = w as i32;
    // `:29151-29160` — flag44 is the constant true; num560 the vx sign.
    let flag44 = true;
    let mut num560 = 0i32;
    if npc.vx < 0.0 {
        num560 = -1;
    }
    if npc.vx > 0.0 {
        num560 = 1;
    }
    // `:29161-29164` — the probe position; `width / 2` is INTEGER division.
    let probe_x = npc.x + npc.vx;
    let probe_y = npc.y;
    let num561 =
        ((probe_x + (wi / 2) as f32 + ((wi / 2 + 1) * num560) as f32) / 16.0) as i32;
    let num562 = ((probe_y + h - 1.0) / 16.0) as i32;
    // `:29165` — the probe tile must straddle the hitbox horizontally.
    if !((num561 * 16) as f32 >= probe_x + w || ((num561 * 16 + 16) as f32) <= probe_x) {
        // `:29167-29172` — the six tiles (t1 the top, t6 the foot row,
        // t2 the behind-column tile at t3's row).
        let t1 = tile_safely(world, num561, num562 - 4);
        let t2 = tile_safely(world, num561 - num560, num562 - 3);
        let t3 = tile_safely(world, num561, num562 - 3);
        let t4 = tile_safely(world, num561, num562 - 2);
        let t5 = tile_safely(world, num561, num562 - 1);
        let t6 = tile_safely(world, num561, num562);
        // `:29173` — the support predicate, verbatim conjunct order.
        let support = (nactive(&t6)
            && !top_slope(&t6)
            && !top_slope(&t5)
            && ((tile_solid(t6.kind) && !tile_solid_top(t6.kind))
                || (flag44
                    && tile_solid_top(t6.kind)
                    && (!tile_solid(t5.kind) || !nactive(&t5))
                    && t6.kind != 16
                    && t6.kind != 18
                    && t6.kind != 134)))
            || (t5.half_brick() && nactive(&t5));
        let t5_clear = !nactive(&t5)
            || !tile_solid(t5.kind)
            || tile_solid_top(t5.kind)
            || (t5.half_brick()
                && (!nactive(&t1) || !tile_solid(t1.kind) || tile_solid_top(t1.kind)));
        let t4_clear = !nactive(&t4) || !tile_solid(t4.kind) || tile_solid_top(t4.kind);
        let t3_clear = !nactive(&t3) || !tile_solid(t3.kind) || tile_solid_top(t3.kind);
        let t2_clear = !nactive(&t2) || !tile_solid(t2.kind) || tile_solid_top(t2.kind);
        if support && t5_clear && t4_clear && t3_clear && t2_clear {
            // `:29175-29183` — the surface height with the halfbrick offsets.
            let mut num563 = (num562 * 16) as f32;
            if t6.half_brick() {
                num563 += 8.0;
            }
            if t5.half_brick() {
                num563 -= 8.0;
            }
            // `:29184-29199` — the lift, only when ≤16.1 px.
            if num563 < probe_y + h {
                let num564 = probe_y + h - num563;
                if (num564 as f64) <= 16.1 {
                    // `:29189` — gfxOffY reads the ORIGINAL position.Y.
                    ctx.gfx_off_y += npc.y + h - num563;
                    npc.y = num563 - h;
                    ctx.step_speed = if num564 < 9.0 { 0.75 } else { 1.5 };
                }
            }
        }
    }
}

/// `NPC.AI()`'s aiStyle-39 arm (`NPC.cs:29145-29665`), one tick.
/// `players` is the target-scan slice (at least one slot, like style28).
/// The caller owns the position integration, gravity, the gfxOffY
/// ease-down and (for 417) the `Collision_MoveSolarSroller` bounce pass
/// (`NPC.cs:92736-92757`) — the module-doc notes cite each.
pub fn style39_step(
    npc: &mut Npc,
    world: &World,
    ctx: &mut Style39Context,
    players: &[TargetCandidate],
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
) {
    let ty = ctx.kind;
    // `:29147-29150` — retarget when untargeted / the target died / the
    // facing is zero. The C# indexes `Main.player[this.target]` (the 256
    // slots); a slot absent from the port's slice stands for the inactive
    // 255 slot (dead) — the same convention `npc_target` uses.
    let target_dead = players
        .get(npc.target as usize)
        .map_or(true, |p| p.dead);
    if npc.target < 0 || target_dead || npc.facing == 0 {
        target_closest(npc, world, players, true, 39, ctx.ticker);
    }
    // `:29151-29202` — the step-up probe.
    step_up_probe(npc, world, ctx);
    // `:29203-29208` — a hit resets the machine (417 is exempt).
    if ctx.just_hit && ty != Style39Type::SolarSroller {
        npc.ai[0] = 0.0;
        npc.ai[1] = 0.0;
        target_closest(npc, world, players, true, 39, ctx.ticker);
    }
    // `:29209-29219` — the 154 frost dust. The gate draw IS consumed
    // (`Main.rand.Next(10) == 0`); W4-e2 (site 29214) the spawn rides the
    // live pool: type 67, speeds (vx·0.5, vy·0.5), alpha 90, scale 1.5;
    // post-writes noGravity + velocity ×0.2. The ±netOffset pair cancels
    // exactly (single-player, the module doc).
    if ty == Style39Type::IceTortoise && rng.next_max(10) == 0 {
        let (w, h) = npc.size();
        npc.x += npc.net_offset.x;
        npc.y += npc.net_offset.y;
        let idx = dust.new_dust(
            rng, dust_ctx, world, Vec2::new(npc.x, npc.y), w as i32, h as i32,
            67, npc.vx * 0.5, npc.vy * 0.5, 90, Color::default(), 1.5,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true; // `:29215`
        d.velocity *= 0.2; // `:29216`
        npc.x -= npc.net_offset.x;
        npc.y -= npc.net_offset.y;
    }
    if npc.ai[0] == 0.0 {
        // `:29222-29230` — the facing follows the velocity sign; the
        // sprite mirrors it.
        if npc.vx < 0.0 {
            npc.facing = -1;
        } else if npc.vx > 0.0 {
            npc.facing = 1;
        }
        npc.sprite_dir = npc.facing;
        let (w, h) = npc.size();
        // `:29231-29235` — the distance reads the player's FLOAT-half
        // centre X but the RAW position.Y (not the centre).
        let p = &players[npc.target as usize];
        let cx = npc.x + w * 0.5;
        let cy = npc.y + h * 0.5;
        let num566 = p.x + p.width as f32 * 0.5 - cx;
        let num567 = p.y - cy;
        let num568 = (num566 * num566 + num567 * num567).sqrt();
        let flag45 = crate::projectile::can_hit(
            world, npc.x, npc.y, w, h, p.x, p.y, p.width as f32, p.height as f32,
        );
        // `:29236-29261` — the aggro counter. The shelly pair adds 2/4,
        // the rest 4/10; only the non-shelly arm has the wet shortcut.
        // The `&` in `(num568 > 200f) & flag45` is the non-short-circuit
        // bitwise and — both sides always evaluate (no side effects
        // here, so `&&` is observationally identical).
        if ty.is_shelly() {
            if num568 > 200.0 && flag45 {
                npc.ai[1] += 2.0;
            }
            if num568 > 600.0 && (flag45 || npc.y + h > p.y - 200.0) {
                npc.ai[1] += 4.0;
            }
        } else {
            if num568 > 200.0 && flag45 {
                npc.ai[1] += 4.0;
            }
            if num568 > 600.0 && (flag45 || npc.y + h > p.y - 200.0) {
                npc.ai[1] += 10.0;
            }
            if npc.wet {
                npc.ai[1] = 1000.0;
            }
        }
        // `:29262-29271` — the stat resets + the difficulty-scaled kb.
        npc.defense_override = Some(ty.def_defense());
        npc.damage_override = Some(ty.def_damage());
        ctx.knockback_resist = if ty.is_shelly() { 0.75 } else { 0.3 }
            * knockback_multiplier(ctx.difficulty);
        // `:29272-29277` — the wind-up trigger.
        npc.ai[1] += 1.0;
        if npc.ai[1] >= 400.0 {
            npc.ai[1] = 0.0;
            npc.ai[0] = 1.0;
        }
        // `:29278-29281` — a blocked move flips the facing (not on a hit).
        if !ctx.just_hit && npc.vx != npc.old_vx {
            npc.facing *= -1;
        }
        // `:29282-29314` — the cliff probe: grounded and the player above
        // the feet → scan the 4×5 tile window 3 ahead; no solid → flip.
        if npc.vy == 0.0 && p.y < npc.y + h {
            // `:29288/:29293` — the centre uses the FLOAT half-width in
            // f64 (`(double)position.X + (double)width * 0.5`).
            let centre_tile = ((npc.x as f64 + w as f64 * 0.5) / 16.0) as i32;
            let (num569, num570) = if npc.facing > 0 {
                (centre_tile, centre_tile + 3)
            } else {
                (centre_tile - 3, centre_tile)
            };
            let num571 = ((npc.y + h + 2.0) / 16.0) as i32 - 1;
            let num572 = num571 + 4;
            let mut flag46 = false;
            for tx in num569..=num570 {
                for ty2 in num571..=num572 {
                    // `:29303` — null tiles (out of world) are skipped;
                    // the test is nactive + tileSolid (no solidTop arm).
                    if let Some(t) = world.tile(tx, ty2) {
                        if nactive(t) && tile_solid(t.kind) {
                            flag46 = true;
                        }
                    }
                }
            }
            if !flag46 {
                npc.facing *= -1;
                npc.vx = 0.1 * npc.facing as f32;
            }
        }
        // `:29315-29342` — the shelly walk: cap 0.5, accel 0.07, the
        // grounded ×0.8 drag on BOTH components, then RETURN (the shelly
        // state-0 arm ends here — no shared tail).
        if ty.is_shelly() {
            let num575 = 0.5f32;
            if npc.vx < 0.0 - num575 || npc.vx > num575 {
                if npc.vy == 0.0 {
                    npc.vx *= 0.8;
                    npc.vy *= 0.8;
                }
            } else if npc.vx < num575 && npc.facing == 1 {
                npc.vx += 0.07;
                if npc.vx > num575 {
                    npc.vx = num575;
                }
            } else if npc.vx > 0.0 - num575 && npc.facing == -1 {
                npc.vx -= 0.07;
                if npc.vx < 0.0 - num575 {
                    npc.vx = 0.0 - num575;
                }
            }
            return;
        }
        // `:29343-29392` — the tortoise/sroller walk: cap 1 within 400 px
        // of the player, 1.5 beyond; same accel/drag shape.
        let num576 = 1.0f32;
        if num568 < 400.0 {
            if npc.vx < 0.0 - num576 || npc.vx > num576 {
                if npc.vy == 0.0 {
                    npc.vx *= 0.8;
                    npc.vy *= 0.8;
                }
            } else if npc.vx < num576 && npc.facing == 1 {
                npc.vx += 0.07;
                if npc.vx > num576 {
                    npc.vx = num576;
                }
            } else if npc.vx > 0.0 - num576 && npc.facing == -1 {
                npc.vx -= 0.07;
                if npc.vx < 0.0 - num576 {
                    npc.vx = 0.0 - num576;
                }
            }
        } else if npc.vx < -1.5 || npc.vx > 1.5 {
            if npc.vy == 0.0 {
                npc.vx *= 0.8;
                npc.vy *= 0.8;
            }
        } else if npc.vx < 1.5 && npc.facing == 1 {
            npc.vx += 0.07;
            if npc.vx > 1.5 {
                npc.vx = 1.5;
            }
        } else if npc.vx > -1.5 && npc.facing == -1 {
            npc.vx -= 0.07;
            if npc.vx < -1.5 {
                npc.vx = -1.5;
            }
        }
    } else if npc.ai[0] == 1.0 {
        // `:29396` — the wind-up halves the horizontal speed each tick.
        npc.vx *= 0.5;
        // `:29397-29404` — the shelly counts half-ticks, the rest whole.
        if ty.is_shelly() {
            npc.ai[1] += 0.5;
        } else {
            npc.ai[1] += 1.0;
        }
        // `:29405-29420` — at 30 the lunge arms; 417 collapses instead.
        if npc.ai[1] >= 30.0 {
            npc.net_update = true;
            target_closest(npc, world, players, true, 39, ctx.ticker);
            npc.ai[1] = 0.0;
            npc.ai[2] = 0.0;
            npc.ai[0] = 3.0;
            if ty == Style39Type::SolarSroller {
                // `:29414-29418` — keep the feet planted across the
                // 52 → 32 collapse, then roll the revolution count.
                let (w, h) = npc.size();
                npc.y += h;
                npc.size_override = Some((w, 32.0));
                npc.y -= 32.0;
                npc.ai[0] = 6.0;
                npc.ai[2] = rng.next_range(2, 5) as f32;
            }
        }
    }
    else if npc.ai[0] == 3.0 {
        // `:29424-29431` — the 154 lunge dust gate. W4-e2 (site 29427):
        // the same 67 body as the head drip — alpha 90, scale 1.5,
        // noGravity + velocity ×0.2; the ±netOffset pair kept verbatim.
        if ty == Style39Type::IceTortoise && rng.next_max(3) < 2 {
            let (w, h) = npc.size();
            npc.x += npc.net_offset.x;
            npc.y += npc.net_offset.y;
            let idx = dust.new_dust(
                rng, dust_ctx, world, Vec2::new(npc.x, npc.y), w as i32, h as i32,
                67, npc.vx * 0.5, npc.vy * 0.5, 90, Color::default(), 1.5,
            );
            let d = dust.slot_mut(idx);
            d.no_gravity = true; // `:29428`
            d.velocity *= 0.2; // `:29429`
            npc.x -= npc.net_offset.x;
            npc.y -= npc.net_offset.y;
        }
        // `:29432-29438` — the lunge stats: damage lerps 2× → 2×·0.9
        // (the shelly 1.5×), defense doubles.
        let mut num578 = 2.0f32;
        if ty.is_shelly() {
            num578 = 1.5;
        }
        let normal = ty.def_damage() as f32 * num578;
        npc.damage_override =
            Some(attack_damage_lerp(ctx.difficulty, normal, normal * 0.9));
        npc.defense_override = Some(ty.def_defense() * 2);
        // `:29439-29477` — the launch tick. NOTE the counter reaches 1
        // here (the wind-up zeroed it), then the arm ++s AGAIN (:29446),
        // so the first flight tick sees ai[1] == 2.
        npc.ai[1] += 1.0;
        if npc.ai[1] == 1.0 {
            npc.net_update = true;
            target_closest(npc, world, players, true, 39, ctx.ticker);
            // `:29444-29445` — the spin seed (0.3) applied once here...
            npc.ai[2] += 0.3;
            npc.rotation += npc.ai[2] * npc.facing as f32;
            npc.ai[1] += 1.0;
            let (w, h) = npc.size();
            let p = &players[npc.target as usize];
            let flag47 = crate::projectile::can_hit(
                world, npc.x, npc.y, w, h, p.x, p.y, p.width as f32, p.height as f32,
            );
            // `:29448-29456` — the speed: 10 (6 without sight), ×0.75 shelly.
            let mut num579 = if flag47 { 10.0 } else { 6.0 };
            if ty.is_shelly() {
                num579 *= 0.75;
            }
            // `:29457-29465` — the aim: the player's FLOAT-half centre X,
            // the raw position.Y, minus the |dx|·0.2 arc offset UNLESS
            // directionY > 0 (the player below the centre).
            let cx = npc.x + w * 0.5;
            let cy = npc.y + h * 0.5;
            let mut num580 = p.x + p.width as f32 * 0.5 - cx;
            let mut num581 = num580.abs() * 0.2;
            if npc.direction_y > 0 {
                num581 = 0.0;
            }
            let mut num582 = p.y - cy - num581;
            let mut num583 = (num580 * num580 + num582 * num582).sqrt();
            npc.net_update = true;
            // `:29467-29469` — normalise to the launch speed.
            num583 = num579 / num583;
            num580 *= num583;
            num582 *= num583;
            // `:29470-29476` — no sight → the flat −10 lob; ai[3] latches
            // the launch vx (the flight's horizontal lock).
            if !flag47 {
                num582 = -10.0;
            }
            npc.vx = num580;
            npc.vy = num582;
            npc.ai[3] = npc.vx;
        } else {
            // `:29480-29488` — over the player (x-overlap, above the
            // player's bottom): damp vx ×0.8, release the lock, ease the
            // rise (+0.2 while vy < 0).
            let (w, _h) = npc.size();
            let p = &players[npc.target as usize];
            if npc.x + w > p.x && npc.x < p.x + p.width as f32 && npc.y < p.y + p.height as f32 {
                npc.vx *= 0.8;
                npc.ai[3] = 0.0;
                if npc.vy < 0.0 {
                    npc.vy += 0.2;
                }
            }
            // `:29489-29493` — the lock holds vx and fights gravity −0.22.
            if npc.ai[3] != 0.0 {
                npc.vx = npc.ai[3];
                npc.vy -= 0.22;
            }
            // `:29494-29499` — 90 ticks → the settle state. The
            // `noGravity = false` write is the cited no-op (module doc).
            if npc.ai[1] >= 90.0 {
                npc.ai[1] = 0.0;
                npc.ai[0] = 4.0;
            }
        }
        // `:29501-29505` — the wet rise + the per-tick spin (runs on the
        // launch tick TOO — the spin lands twice there, verbatim).
        if npc.wet && npc.direction_y < 0 {
            npc.vy -= 0.3;
        }
        npc.rotation += npc.ai[2] * npc.facing as f32;
    } else if npc.ai[0] == 4.0 {
        // `:29509-29513` — the settle: wet rise, ×0.96 vx drag.
        if npc.wet && npc.direction_y < 0 {
            npc.vy -= 0.3;
        }
        npc.vx *= 0.96;
        // `:29514-29522` — the spin decays 0.01/tick; once spent, a
        // grounded (vy >= 0) tick zeroes the rotation.
        if npc.ai[2] > 0.0 {
            npc.ai[2] -= 0.01;
            npc.rotation += npc.ai[2] * npc.facing as f32;
        } else if npc.vy >= 0.0 {
            npc.rotation = 0.0;
        }
        // `:29523-29530` — spent AND (grounded OR wet) → the rest state.
        if npc.ai[2] <= 0.0 && (npc.vy == 0.0 || npc.wet) {
            npc.net_update = true;
            npc.rotation = 0.0;
            npc.ai[2] = 0.0;
            npc.ai[1] = 0.0;
            npc.ai[0] = 5.0;
        }
    } else if npc.ai[0] == 6.0 {
        // `:29534-29536` — the ball stats: damage lerps 1.8× → 1.4×,
        // defense doubles, kb 0.
        npc.damage_override = Some(attack_damage_lerp(
            ctx.difficulty,
            ty.def_damage() as f32 * 1.8,
            ty.def_damage() as f32 * 1.4,
        ));
        npc.defense_override = Some(ty.def_defense() * 2);
        ctx.knockback_resist = 0.0;
        // `:29537-29545` — the ball dust gate. W4-e2 (site 29540): the
        // type-6 body at Center − (30, 30), box 60×60, speeds (vx·0.5,
        // vy·0.5), alpha 90, scale 1.5; post-writes noGravity, velocity
        // ×0.2, fadeIn 1. The type-6 pool tweak ADDS the Next(-10, 6) vy
        // re-roll inside new_dust (Dust.cs:215-219).
        if rng.next_max(3) < 2 {
            npc.x += npc.net_offset.x;
            npc.y += npc.net_offset.y;
            let (cx, cy) = npc.center();
            let idx = dust.new_dust(
                rng, dust_ctx, world, Vec2::new(cx - 30.0, cy - 30.0), 60, 60,
                6, npc.vx * 0.5, npc.vy * 0.5, 90, Color::default(), 1.5,
            );
            let d = dust.slot_mut(idx);
            d.no_gravity = true; // `:29541`
            d.velocity *= 0.2; // `:29542`
            d.fade_in = 1.0; // `:29543`
            npc.x -= npc.net_offset.x;
            npc.y -= npc.net_offset.y;
        }
        npc.ai[1] += 1.0;
        // `:29547-29581` — the bounce FX layer: ai[3] is armed to 1 by
        // the collision bounce pass (`Collision_MoveSolarSroller`,
        // :92744 — the wiring round ports it) and counts to 10 here.
        if npc.ai[3] > 0.0 {
            // `:29549` — the netOffset add wraps the whole FX block.
            npc.x += npc.net_offset.x;
            npc.y += npc.net_offset.y;
            if npc.ai[3] == 1.0 {
                // W4-e2 (sites 29555/29558) — `:29552-29561`: the 32 dust
                // PAIRS at Center − (50, 50), box 100×100, type 6, speeds
                // zero, alpha 100: first scale 2.5 (noGravity, velocity
                // ×3), then scale 1.5 (velocity ×2, noGravity). The
                // type-6 pool tweak rolls inside each new_dust.
                let (cx, cy) = npc.center();
                let vector68 = Vec2::new(cx - 50.0, cy - 50.0);
                for _ in 0..32 {
                    let idx = dust.new_dust(
                        rng, dust_ctx, world, vector68, 100, 100, 6,
                        0.0, 0.0, 100, Color::default(), 2.5,
                    );
                    let d = dust.slot_mut(idx);
                    d.no_gravity = true; // `:29556`
                    d.velocity *= 3.0; // `:29557`
                    let idx = dust.new_dust(
                        rng, dust_ctx, world, vector68, 100, 100, 6,
                        0.0, 0.0, 100, Color::default(), 1.5,
                    );
                    let d = dust.slot_mut(idx);
                    d.velocity *= 2.0; // `:29559`
                    d.no_gravity = true; // `:29560`
                }
                // `:29562-29568` — the 4 gores: the ARGUMENT draws are
                // arm-level and consumed here in C# order — per gore:
                // Next(100) ×2 (the offset), Next(61, 64) (the type),
                // then Next(-10, 11) ×2 (the velocity nudges). The
                // NewGore-internal Next(10,31)/Next(-20,21) pair
                // (Gore.cs:1327-1328) is pool-side, documented above.
                for _ in 0..4 {
                    let _ox = (50 * rng.next_max(100)) as f32 / 100.0;
                    let _oy = (50 * rng.next_max(100)) as f32 / 100.0;
                    let _gore_type = rng.next_range(61, 64);
                    let _nvx = rng.next_range(-10, 11) as f32 * 0.05;
                    let _nvy = rng.next_range(-10, 11) as f32 * 0.05;
                }
            }
            // W4-e2 (site 29572) — `:29570-29574`: the 5 trailing type-31
            // dusts at position, box (width, height), speeds zero, alpha
            // 100, scale 1.5; the post-write velocity × NextFloat runs
            // even when NewDust refuses (the C# reads Main.dust[num590]
            // either way — the sentinel takes the write).
            let (w, h) = npc.size();
            for _ in 0..5 {
                let idx = dust.new_dust(
                    rng, dust_ctx, world, Vec2::new(npc.x, npc.y),
                    w as i32, h as i32, 31, 0.0, 0.0, 100, Color::default(), 1.5,
                );
                let scale = rng.next_float(); // `:29573`
                dust.slot_mut(idx).velocity *= scale;
            }
            // `:29575-29579` — the FX layer wraps at 10.
            npc.ai[3] += 1.0;
            if npc.ai[3] >= 10.0 {
                npc.ai[3] = 0.0;
            }
            // `:29580` — the netOffset restore closes the FX block.
            npc.x -= npc.net_offset.x;
            npc.y -= npc.net_offset.y;
        }
        // `:29582-29611` — the launch tick: same aim shape as the lunge,
        // speed 16 (10 without sight), no-sight lob −12, NO ai[3] latch.
        if npc.ai[1] == 1.0 {
            npc.net_update = true;
            target_closest(npc, world, players, true, 39, ctx.ticker);
            let (w, h) = npc.size();
            let p = &players[npc.target as usize];
            let flag48 = crate::projectile::can_hit(
                world, npc.x, npc.y, w, h, p.x, p.y, p.width as f32, p.height as f32,
            );
            let num591 = if flag48 { 16.0 } else { 10.0 };
            let cx = npc.x + w * 0.5;
            let cy = npc.y + h * 0.5;
            let mut num592 = p.x + p.width as f32 * 0.5 - cx;
            let mut num593 = num592.abs() * 0.2;
            if npc.direction_y > 0 {
                num593 = 0.0;
            }
            let mut num594 = p.y - cy - num593;
            let mut num595 = (num592 * num592 + num594 * num594).sqrt();
            npc.net_update = true;
            num595 = num591 / num595;
            num592 *= num595;
            num594 *= num595;
            if !flag48 {
                num594 = -12.0;
            }
            npc.vx = num592;
            npc.vy = num594;
        } else {
            // `:29614-29621` — over the player: ×0.9 vx, +0.2 rise ease
            // (NO ai[3] release here — the ball keeps no vx lock).
            let (w, _h) = npc.size();
            let p = &players[npc.target as usize];
            if npc.x + w > p.x && npc.x < p.x + p.width as f32 && npc.y < p.y + p.height as f32 {
                npc.vx *= 0.9;
                if npc.vy < 0.0 {
                    npc.vy += 0.2;
                }
            }
            // `:29622-29626` — the exit: revolutions spent (the bounce
            // pass decrements ai[2], :92743) or the 1200-tick timeout.
            if npc.ai[2] == 0.0 || npc.ai[1] >= 1200.0 {
                npc.ai[1] = 0.0;
                npc.ai[0] = 5.0;
            }
        }
        // `:29628-29632` — the wet rise + the roll spin: the vx-derived
        // angle clamped to ±π/10, per tick.
        if npc.wet && npc.direction_y < 0 {
            npc.vy -= 0.3;
        }
        let spin = (npc.vx / 10.0 * npc.facing as f32)
            .clamp(-std::f32::consts::PI / 10.0, std::f32::consts::PI / 10.0);
        npc.rotation += spin;
    } else if npc.ai[0] == 5.0 {
        // `:29636-29641` — 417 unrolls: keep the feet planted across the
        // 32 → 52 restore.
        if ty == Style39Type::SolarSroller {
            let (w, h) = npc.size();
            npc.y += h;
            npc.size_override = Some((w, 52.0));
            npc.y -= 52.0;
        }
        // `:29642-29651` — the rest: zero spin/vx, the half-tick counter
        // for the shelly pair.
        npc.rotation = 0.0;
        npc.vx = 0.0;
        if ty.is_shelly() {
            npc.ai[1] += 0.5;
        } else {
            npc.ai[1] += 1.0;
        }
        // `:29652-29658` — 30 ticks → back to the walk.
        if npc.ai[1] >= 30.0 {
            target_closest(npc, world, players, true, 39, ctx.ticker);
            npc.net_update = true;
            npc.ai[1] = 0.0;
            npc.ai[0] = 0.0;
        }
        // `:29659-29663` — wet overrides the exit: straight back to the
        // lunge (runs AFTER the 30-tick arm, so it wins the same tick).
        if npc.wet {
            npc.ai[0] = 3.0;
            npc.ai[1] = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::tile::{Tile, FLAG_ACTIVE, FLAG_HALF_BRICK};

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
        World::new("t".into(), 200, 200)
    }

    /// W4-e2: the permissive dust ctx for the witnesses — the whole world
    /// on-screen, the full 6000 budget (the `npc::permissive_dust_ctx`
    /// shape, local so the pure module keeps no npc.rs test dependency).
    fn permissive_ctx(w: &World) -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(w.width as f32 * 16.0, w.height as f32 * 16.0),
            max_dust_to_draw: 6000,
        }
    }

    fn ctx(kind: Style39Type) -> Style39Context {
        Style39Context {
            kind,
            difficulty: 1.0, // Classic
            just_hit: false,
            ticker: 0,
            gfx_off_y: 0.0,
            step_speed: 1.0,
            knockback_resist: 0.0,
        }
    }

    /// A sroller-shaped NPC with a live target slot 0 and a right facing
    /// (the head retarget gate stays shut so the state machine is
    /// isolated). The unported kinds ride `size_override` + `ctx.kind`.
    fn npc39(x: f32, y: f32, size: (f32, f32)) -> Npc {
        let mut n = Npc::new(NpcKind::SolarSroller, x, y, 1, 0.0);
        n.size_override = Some(size);
        n.target = 0;
        n
    }

    fn put(w: &mut World, x: i32, y: i32, extra: u8) {
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind: 1,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE | extra;
    }

    /// `NPC.cs:29396-29419` — the wind-up: 417 counts WHOLE ticks (the
    /// live arm's 0.5 is the shelly rate), halves vx, and at 30 collapses
    /// 52 → 32 with the feet planted, rolling `Next(2, 5)` revolutions —
    /// exactly one RNG draw, in C# order.
    #[test]
    fn the_windup_collapses_the_sroller_and_rolls_the_revolutions() {
        let w = air();
        let mut c = ctx(Style39Type::SolarSroller);
        let mut n = npc39(1087.0, 984.0, (46.0, 52.0));
        n.ai[0] = 1.0;
        n.ai[1] = 29.0;
        n.vx = 2.0;
        let mut rng = UnifiedRandom::new(12345);
        let mut probe = rng.clone();
        style39_step(&mut n, &w, &mut c, &[cand(5000.0, 2000.0)], &mut rng, &mut DustPool::new(), &permissive_ctx(&w));
        assert_eq!(n.vx, 1.0, "the wind-up halves vx (:29396)");
        assert_eq!(n.ai[0], 6.0, "the ball state (:29417)");
        assert_eq!(n.size_override, Some((46.0, 32.0)), "the collapse");
        assert_eq!(n.y, 984.0 + 52.0 - 32.0, "the feet stay planted");
        assert!(n.ai[2] >= 2.0 && n.ai[2] <= 4.0, "Next(2, 5)");
        let want = probe.next_range(2, 5) as f32;
        assert_eq!(n.ai[2], want, "the revolution draw (:29418)");
        assert_eq!(rng, probe, "exactly one draw consumed");
        assert!(n.net_update, "the transition syncs (:29407)");
        // The shelly pair counts half-ticks: 29 → 29.5 does NOT arm.
        let mut c2 = ctx(Style39Type::GiantShelly);
        let mut n2 = npc39(1087.0, 984.0, (28.0, 22.0));
        n2.ai[0] = 1.0;
        n2.ai[1] = 29.0;
        style39_step(&mut n2, &w, &mut c2, &[cand(5000.0, 2000.0)], &mut rng, &mut DustPool::new(), &permissive_ctx(&w));
        assert_eq!(n2.ai[1], 29.5, "the shelly 0.5 rate (:29399)");
        assert_eq!(n2.ai[0], 1.0, "still winding up");
    }

    /// `NPC.cs:29582-29611` + `:29632` — the ball launch: speed 16 with
    /// sight, the aim at the player's raw Y (the arc offset is gated off
    /// by `directionY > 0`), the 1.8× damage / doubled defense / kb 0,
    /// and the ±π/10-clamped vx spin.
    #[test]
    fn the_ball_launch_aims_at_sixteen_and_spins_clamped() {
        let w = air();
        let mut c = ctx(Style39Type::SolarSroller);
        // Ball centre (1110, 1000); player (1400, 1400) → dx 300, dy 400
        // (the player's centre Y 1421 is BELOW → directionY stays 1, so
        // the |dx|·0.2 offset is zeroed, :29595-29598). dist 500.
        let mut n = npc39(1087.0, 984.0, (46.0, 32.0));
        n.ai[0] = 6.0;
        n.ai[2] = 3.0;
        n.direction_y = 1;
        let mut rng = UnifiedRandom::new(7);
        style39_step(&mut n, &w, &mut c, &[cand(1400.0, 1400.0)], &mut rng, &mut DustPool::new(), &permissive_ctx(&w));
        assert!((n.vx - 9.6).abs() < 1e-4, "300·16/500 ({})", n.vx);
        assert!((n.vy - 12.8).abs() < 1e-4, "400·16/500 ({})", n.vy);
        assert_eq!(n.damage_override, Some(144), "80·1.8 classic (:29534)");
        assert_eq!(n.defense_override, Some(68), "34·2 (:29535)");
        assert_eq!(c.knockback_resist, 0.0, "the ball kb (:29536)");
        // vx/10 = 0.96 > π/10 → the clamp binds (:29632).
        let want = std::f32::consts::PI / 10.0;
        assert!((n.rotation - want).abs() < 1e-5, "{}", n.rotation);
        assert_eq!(n.ai[1], 1.0, "the launch tick");
    }

    /// `NPC.cs:29622-29626` — the ball exit is `ai[2] == 0` (the bounce
    /// pass spends the revolutions, `:92743`) OR the 1200-tick timeout —
    /// NOT the live arm's `60 * ai[2]` ticks.
    #[test]
    fn the_ball_exit_is_revolutions_or_the_1200_timeout() {
        let w = air();
        let mut c = ctx(Style39Type::SolarSroller);
        let p = [cand(5000.0, 2000.0)];
        let mut rng = UnifiedRandom::new(1);
        // Revolutions spent → exit.
        let mut n = npc39(1087.0, 984.0, (46.0, 32.0));
        n.ai[0] = 6.0;
        n.ai[1] = 5.0;
        n.ai[2] = 0.0;
        style39_step(&mut n, &w, &mut c, &p, &mut rng, &mut DustPool::new(), &permissive_ctx(&w));
        assert_eq!(n.ai[0], 5.0, "the unroll (:29625)");
        assert_eq!(n.ai[1], 0.0);
        // The timeout: 1199 → 1200 exits even with revolutions left.
        let mut n2 = npc39(1087.0, 984.0, (46.0, 32.0));
        n2.ai[0] = 6.0;
        n2.ai[1] = 1199.0;
        n2.ai[2] = 3.0;
        style39_step(&mut n2, &w, &mut c, &p, &mut rng, &mut DustPool::new(), &permissive_ctx(&w));
        assert_eq!(n2.ai[0], 5.0, "the 1200 timeout (:29622)");
        // Neither → the roll continues.
        let mut n3 = npc39(1087.0, 984.0, (46.0, 32.0));
        n3.ai[0] = 6.0;
        n3.ai[1] = 10.0;
        n3.ai[2] = 3.0;
        style39_step(&mut n3, &w, &mut c, &p, &mut rng, &mut DustPool::new(), &permissive_ctx(&w));
        assert_eq!(n3.ai[0], 6.0);
        assert_eq!(n3.ai[1], 11.0);
    }

    /// `NPC.cs:29636-29663` — the unroll restores 52 with the feet
    /// planted; the 30-tick rest exits to the walk, and the wet override
    /// wins the SAME tick (it runs after the exit arm).
    #[test]
    fn the_unroll_restores_the_height_and_wet_overrides_the_exit() {
        let w = air();
        let mut c = ctx(Style39Type::SolarSroller);
        let p = [cand(5000.0, 2000.0)];
        let mut rng = UnifiedRandom::new(2);
        let mut n = npc39(1087.0, 1000.0, (46.0, 32.0));
        n.ai[0] = 5.0;
        n.ai[1] = 29.0;
        n.rotation = 1.0;
        n.vx = 3.0;
        style39_step(&mut n, &w, &mut c, &p, &mut rng, &mut DustPool::new(), &permissive_ctx(&w));
        assert_eq!(n.size_override, Some((46.0, 52.0)), "the restore");
        assert_eq!(n.y, 1000.0 + 32.0 - 52.0, "the feet stay planted");
        assert_eq!(n.rotation, 0.0, "(:29642)");
        assert_eq!(n.vx, 0.0, "(:29643)");
        assert_eq!(n.ai[0], 0.0, "the 30-tick exit (:29657)");
        assert_eq!(n.ai[1], 0.0);
        // Wet: the exit to 0 is overridden to the lunge (:29659-29663).
        let mut n2 = npc39(1087.0, 1000.0, (46.0, 32.0));
        n2.ai[0] = 5.0;
        n2.ai[1] = 29.0;
        n2.wet = true;
        style39_step(&mut n2, &w, &mut c, &p, &mut rng, &mut DustPool::new(), &permissive_ctx(&w));
        assert_eq!(n2.ai[0], 3.0, "wet → straight to the lunge");
        assert_eq!(n2.ai[1], 0.0);
    }

    /// `NPC.cs:29439-29477` + `:29505` — the lunge launch: speed 10, the
    /// 2× damage / doubled defense, the ai[3] vx latch, the ++ TWICE (the
    /// first flight tick sees 2), and the 0.3 spin seed applied on BOTH
    /// the launch line and the shared tail.
    #[test]
    fn the_lunge_launch_locks_vx_and_spins_twice() {
        let w = air();
        let mut c = ctx(Style39Type::GiantTortoise);
        // Centre (1110, 1000), player (1400, 1400) below → directionY 1
        // after the launch TargetClosest → no arc offset. dist 500.
        let mut n = npc39(1087.0, 984.0, (46.0, 32.0));
        n.ai[0] = 3.0;
        let mut rng = UnifiedRandom::new(3);
        style39_step(&mut n, &w, &mut c, &[cand(1400.0, 1400.0)], &mut rng, &mut DustPool::new(), &permissive_ctx(&w));
        assert!((n.vx - 6.0).abs() < 1e-4, "300·10/500 ({})", n.vx);
        assert!((n.vy - 8.0).abs() < 1e-4, "400·10/500 ({})", n.vy);
        assert_eq!(n.ai[3], n.vx, "the vx latch (:29476)");
        assert_eq!(n.ai[1], 2.0, "the double ++ (:29439/:29446)");
        assert_eq!(n.ai[2], 0.3, "the spin seed (:29444)");
        assert!((n.rotation - 0.6).abs() < 1e-5, "0.3 twice (:29445/:29505)");
        assert_eq!(n.damage_override, Some(160), "80·2 classic (:29437)");
        assert_eq!(n.defense_override, Some(60), "30·2 (:29438)");
        // The flight tick: the latch holds vx and fights gravity −0.22.
        let vy_before = n.vy;
        style39_step(&mut n, &w, &mut c, &[cand(1400.0, 1400.0)], &mut rng, &mut DustPool::new(), &permissive_ctx(&w));
        assert_eq!(n.vx, n.ai[3], "the lock (:29491)");
        assert!((n.vy - (vy_before - 0.22)).abs() < 1e-5, "(:29492)");
    }

    /// `NPC.cs:29480-29493` — over the player the lunge damps vx ×0.8,
    /// releases the ai[3] lock and eases the rise; with the lock released
    /// the −0.22 pull does NOT run.
    #[test]
    fn the_lunge_releases_over_the_player() {
        let w = air();
        let mut c = ctx(Style39Type::GiantTortoise);
        let mut n = npc39(1087.0, 984.0, (46.0, 32.0));
        n.ai[0] = 3.0;
        n.ai[1] = 5.0;
        n.ai[3] = 6.0;
        n.vx = 6.0;
        n.vy = -2.0;
        let mut rng = UnifiedRandom::new(4);
        // The player overlaps x (1100..1120 vs 1087..1133) and the npc
        // is above the player's bottom (984 < 1042).
        style39_step(&mut n, &w, &mut c, &[cand(1100.0, 1000.0)], &mut rng, &mut DustPool::new(), &permissive_ctx(&w));
        assert!((n.vx - 4.8).abs() < 1e-5, "×0.8 (:29482)");
        assert_eq!(n.ai[3], 0.0, "the release (:29483)");
        assert!((n.vy - (-1.8)).abs() < 1e-5, "+0.2 (:29486)");
    }

    /// `NPC.cs:29315-29342` — the shelly walk: the 0.5 cap with the 0.07
    /// accel and the clamp, the grounded ×0.8 drag on BOTH components,
    /// and the +2/+4 aggro with the 0.75 kb.
    #[test]
    fn the_shelly_walk_caps_at_half() {
        let w = air();
        let mut c = ctx(Style39Type::GiantShelly);
        let p = [cand(5000.0, 2000.0)]; // far + below → no cliff probe
        let mut rng = UnifiedRandom::new(5);
        let mut n = npc39(1087.0, 984.0, (28.0, 22.0));
        n.vx = 0.49;
        n.old_vx = 0.49; // no blocked-move flip
        style39_step(&mut n, &w, &mut c, &p, &mut rng, &mut DustPool::new(), &permissive_ctx(&w));
        assert_eq!(n.vx, 0.5, "0.49 + 0.07 clamps (:29327-29331)");
        assert_eq!(n.ai[1], 7.0, "+2 +4 +1 (:29240/:29244/:29272)");
        assert_eq!(c.knockback_resist, 0.75, "the shelly kb (:29266)");
        assert_eq!(n.damage_override, Some(16), "the defDamage reset");
        assert_eq!(n.defense_override, Some(12));
        // Over the cap + grounded: the ×0.8 drag hits both components.
        let mut n2 = npc39(1087.0, 984.0, (28.0, 22.0));
        n2.vx = 0.6;
        n2.vy = 0.0;
        n2.old_vx = 0.6;
        style39_step(&mut n2, &w, &mut c, &p, &mut rng, &mut DustPool::new(), &permissive_ctx(&w));
        assert!((n2.vx - 0.48).abs() < 1e-5, "×0.8 (:29322)");
    }

    /// `NPC.cs:29249-29277` — the tortoise/sroller aggro: +4/+10 far with
    /// sight, only the +1 near, and the wet shortcut to 1000 arms the
    /// wind-up on the same tick.
    #[test]
    fn the_aggro_counter_and_the_wet_shortcut() {
        let w = air();
        let mut c = ctx(Style39Type::GiantTortoise);
        let mut rng = UnifiedRandom::new(6);
        // Near (< 200): only the ++ lands.
        let mut n = npc39(1087.0, 984.0, (46.0, 32.0));
        n.vy = 0.5; // skip the cliff probe
        style39_step(&mut n, &w, &mut c, &[cand(1120.0, 990.0)], &mut rng, &mut DustPool::new(), &permissive_ctx(&w));
        assert_eq!(n.ai[1], 1.0, "near → no aggro adds");
        assert_eq!(n.ai[0], 0.0);
        // Far + wet: 1000 → ++ → ≥400 → the wind-up.
        let mut n2 = npc39(1087.0, 984.0, (46.0, 32.0));
        n2.vy = 0.5;
        n2.wet = true;
        style39_step(&mut n2, &w, &mut c, &[cand(5000.0, 2000.0)], &mut rng, &mut DustPool::new(), &permissive_ctx(&w));
        assert_eq!(n2.ai[0], 1.0, "the wet shortcut (:29259/:29273)");
        assert_eq!(n2.ai[1], 0.0);
        // Far + dry: +4 +10 +1 = 15, still walking.
        let mut n3 = npc39(1087.0, 984.0, (46.0, 32.0));
        n3.vy = 0.5;
        style39_step(&mut n3, &w, &mut c, &[cand(5000.0, 2000.0)], &mut rng, &mut DustPool::new(), &permissive_ctx(&w));
        assert_eq!(n3.ai[1], 15.0);
        assert_eq!(n3.ai[0], 0.0);
    }

    /// `NPC.cs:29203-29208` + `:29278-29281` — the justHit reset is
    /// EXEMPT for 417; the blocked-move facing flip needs `!justHit`.
    #[test]
    fn the_justhit_reset_is_exempt_for_the_sroller() {
        let w = air();
        let mut rng = UnifiedRandom::new(8);
        let p = [cand(1120.0, 990.0)];
        // Tortoise: hit mid-lunge → state 0, then the walk runs the same
        // tick (near player → ai[1] = 1).
        let mut c = ctx(Style39Type::GiantTortoise);
        c.just_hit = true;
        let mut n = npc39(1087.0, 984.0, (46.0, 32.0));
        n.ai[0] = 3.0;
        n.ai[1] = 50.0;
        n.vy = 0.5;
        style39_step(&mut n, &w, &mut c, &p, &mut rng, &mut DustPool::new(), &permissive_ctx(&w));
        assert_eq!(n.ai[0], 0.0, "the reset (:29205)");
        assert_eq!(n.ai[1], 1.0, "the walk ran after the reset");
        // Sroller: the same hit does NOT reset the ball.
        let mut c2 = ctx(Style39Type::SolarSroller);
        c2.just_hit = true;
        let mut n2 = npc39(1087.0, 984.0, (46.0, 32.0));
        n2.ai[0] = 6.0;
        n2.ai[1] = 5.0;
        n2.ai[2] = 3.0;
        style39_step(&mut n2, &w, &mut c2, &[cand(5000.0, 2000.0)], &mut rng, &mut DustPool::new(), &permissive_ctx(&w));
        assert_eq!(n2.ai[0], 6.0, "417 is exempt (:29203)");
        assert_eq!(n2.ai[1], 6.0);
        // The blocked-move flip: vx != oldVelocity.X, no hit → flip.
        let mut c3 = ctx(Style39Type::GiantTortoise);
        let mut n3 = npc39(1087.0, 984.0, (46.0, 32.0));
        n3.vx = 1.0;
        n3.old_vx = 0.5;
        n3.vy = 0.5;
        style39_step(&mut n3, &w, &mut c3, &p, &mut rng, &mut DustPool::new(), &permissive_ctx(&w));
        assert_eq!(n3.facing, -1, "the flip (:29280)");
    }

    /// `NPC.cs:29161-29202` — the step-up probe: a one-tile ledge ahead
    /// lifts the hitbox (≤16.1), gfxOffY absorbs the lift, stepSpeed is
    /// 1.5 for a ≥9-px lift and 0.75 for the halfbrick's 8-px one.
    #[test]
    fn the_step_up_probe_lifts_onto_the_ledge() {
        let mut w = air();
        // The ledge tile (66, 59): its top is y 944, the npc's feet 960.
        put(&mut w, 66, 59, 0);
        let mut c = ctx(Style39Type::GiantTortoise);
        let mut n = npc39(1010.0, 928.0, (46.0, 32.0));
        n.ai[0] = 2.0; // no state arm — isolate the probe
        n.vx = 1.0;
        let mut rng = UnifiedRandom::new(9);
        style39_step(&mut n, &w, &mut c, &[cand(1120.0, 990.0)], &mut rng, &mut DustPool::new(), &permissive_ctx(&w));
        assert_eq!(n.y, 912.0, "the lift onto 944 (:29190)");
        assert_eq!(c.gfx_off_y, 16.0, "gfxOffY absorbs it (:29189)");
        assert_eq!(c.step_speed, 1.5, "16 ≥ 9 (:29197)");
        // Halfbrick ledge: the surface is 952 → an 8-px lift, 0.75.
        let mut w2 = air();
        put(&mut w2, 66, 59, FLAG_HALF_BRICK);
        let mut c2 = ctx(Style39Type::GiantTortoise);
        let mut n2 = npc39(1010.0, 928.0, (46.0, 32.0));
        n2.ai[0] = 2.0;
        n2.vx = 1.0;
        style39_step(&mut n2, &w2, &mut c2, &[cand(1120.0, 990.0)], &mut rng, &mut DustPool::new(), &permissive_ctx(&w2));
        assert_eq!(n2.y, 920.0, "the halfbrick +8 (:29178)");
        assert_eq!(c2.gfx_off_y, 8.0);
        assert_eq!(c2.step_speed, 0.75, "8 < 9 (:29193)");
    }

    /// `NPC.cs:29282-29314` — the cliff probe: grounded with the player
    /// above the feet, no solid in the 4×5 window ahead → flip + the
    /// 0.1·direction nudge (the walk's accel then applies to it).
    #[test]
    fn the_cliff_probe_flips_at_the_edge() {
        let mut w = air();
        // Ground only BEHIND the window (the window is tiles 63..66).
        for x in 58..=62 {
            put(&mut w, x, 60, 0);
        }
        let mut c = ctx(Style39Type::GiantTortoise);
        let mut n = npc39(1000.0, 928.0, (46.0, 32.0));
        n.vx = 0.5;
        n.old_vx = 0.5; // no blocked-move flip before the probe
        let mut rng = UnifiedRandom::new(10);
        // The player above the feet (900 < 960) arms the probe.
        style39_step(&mut n, &w, &mut c, &[cand(1010.0, 900.0)], &mut rng, &mut DustPool::new(), &permissive_ctx(&w));
        assert_eq!(n.facing, -1, "the cliff flip (:29311)");
        // −0.1 nudge, then the walk accel −0.07 (facing −1, |vx| < 1).
        assert!((n.vx - (-0.17)).abs() < 1e-5, "(:29312 + :29363)");
    }

    /// `NPC.cs:29537` + `:29547-29581` — the ball FX layer's draws, in C#
    /// order, W4-e2 with the REAL pool internals: the dust gate Next(3),
    /// on a pass the type-6 body (7 default draws + the Next(-10,6) vy
    /// re-roll), then on ai[3] == 1 the 32 type-6 pairs, the 4 gores'
    /// Next(100)×2 / Next(61,64) / Next(-10,11)×2 each (the NewGore body
    /// is d55's), then the 5 type-31 spawns each + its NextFloat scale;
    /// ai[3] counts to 10 and wraps. The mirror replays every cited draw
    /// through the same pool calls and compares full stream state.
    #[test]
    fn the_ball_fx_layer_consumes_the_cited_draws() {
        let w = air();
        let dc = permissive_ctx(&w);
        let mut c = ctx(Style39Type::SolarSroller);
        let p = [cand(5000.0, 2000.0)];
        let mut n = npc39(1087.0, 984.0, (46.0, 32.0));
        n.ai[0] = 6.0;
        n.ai[1] = 5.0; // a flight tick, not the launch
        n.ai[2] = 3.0;
        n.ai[3] = 1.0;
        let mut rng = UnifiedRandom::new(12345);
        let mut probe = rng.clone();
        let mut pool = DustPool::new();
        let mut mpool = DustPool::new();
        style39_step(&mut n, &w, &mut c, &p, &mut rng, &mut pool, &dc);
        // The mirror: the gate, then (on a pass) the type-6 body at the
        // same anchor the arm read (Center − 30 after the netOffset add,
        // zero here).
        let gate = probe.next_max(3); // the dust gate (:29537)
        let (cx, cy) = (1087.0 + 23.0, 984.0 + 16.0);
        if gate < 2 {
            mpool.new_dust(
                &mut probe, &dc, &w, Vec2::new(cx - 30.0, cy - 30.0), 60, 60,
                6, n.vx * 0.5, n.vy * 0.5, 90, Color::default(), 1.5,
            );
        }
        // The FX block (ai[3] was 1): the 32 pairs at Center − 50.
        for _ in 0..32 {
            mpool.new_dust(
                &mut probe, &dc, &w, Vec2::new(cx - 50.0, cy - 50.0), 100, 100,
                6, 0.0, 0.0, 100, Color::default(), 2.5,
            );
            mpool.new_dust(
                &mut probe, &dc, &w, Vec2::new(cx - 50.0, cy - 50.0), 100, 100,
                6, 0.0, 0.0, 100, Color::default(), 1.5,
            );
        }
        for _ in 0..4 {
            probe.next_max(100); // the gore offset X (:29564)
            probe.next_max(100); // the gore offset Y
            probe.next_range(61, 64); // the gore type
            probe.next_range(-10, 11); // the velocity nudge X (:29566)
            probe.next_range(-10, 11); // the velocity nudge Y (:29567)
        }
        for _ in 0..5 {
            mpool.new_dust(
                &mut probe, &dc, &w, Vec2::new(1087.0, 984.0), 46, 32,
                31, 0.0, 0.0, 100, Color::default(), 1.5,
            );
            probe.next_float(); // the dust velocity scale (:29573)
        }
        assert_eq!(rng, probe, "the full draw order incl. pool internals");
        assert_eq!(n.ai[3], 2.0, "the FX counter (:29575)");
        // ai[3] == 9: no gore burst, just the gate (+ body) and the 5
        // type-31 spawns + floats, wrap to 0.
        let mut n2 = npc39(1087.0, 984.0, (46.0, 32.0));
        n2.ai[0] = 6.0;
        n2.ai[1] = 5.0;
        n2.ai[2] = 3.0;
        n2.ai[3] = 9.0;
        let mut rng2 = UnifiedRandom::new(999);
        let mut probe2 = rng2.clone();
        let mut pool2 = DustPool::new();
        let mut mpool2 = DustPool::new();
        style39_step(&mut n2, &w, &mut c, &p, &mut rng2, &mut pool2, &dc);
        let gate2 = probe2.next_max(3);
        if gate2 < 2 {
            mpool2.new_dust(
                &mut probe2, &dc, &w, Vec2::new(cx - 30.0, cy - 30.0), 60, 60,
                6, n2.vx * 0.5, n2.vy * 0.5, 90, Color::default(), 1.5,
            );
        }
        for _ in 0..5 {
            mpool2.new_dust(
                &mut probe2, &dc, &w, Vec2::new(1087.0, 984.0), 46, 32,
                31, 0.0, 0.0, 100, Color::default(), 1.5,
            );
            probe2.next_float();
        }
        assert_eq!(rng2, probe2, "the gate + 5 spawns");
        assert_eq!(n2.ai[3], 0.0, "the wrap at 10 (:29576-29579)");
    }

    /// `NPC.cs:29212` + `:29424` — the IceTortoise dust gates: Next(10) in
    /// the head, Next(3) in the lunge; W4-e2 (sites 29214/29427) a passed
    /// gate rolls the type-67 body through the pool (7 default draws, no
    /// per-type tweak). The mirror replays gate + body and compares state.
    #[test]
    fn the_ice_tortoise_gates_consume_their_draws() {
        let w = air();
        let dc = permissive_ctx(&w);
        let mut c = ctx(Style39Type::IceTortoise);
        // State 0: the head gate (+ body on a pass). The head dust reads
        // the PRE-walk velocity/position (:29214 precedes the state arms),
        // so the mirror snapshots them before the step.
        let mut n = npc39(1087.0, 984.0, (46.0, 32.0));
        n.vy = 0.5;
        let (hx, hy, hvx, hvy) = (n.x, n.y, n.vx, n.vy);
        let mut rng = UnifiedRandom::new(42);
        let mut probe = rng.clone();
        let mut pool = DustPool::new();
        let mut mpool = DustPool::new();
        style39_step(&mut n, &w, &mut c, &[cand(1120.0, 990.0)], &mut rng, &mut pool, &dc);
        let g = probe.next_max(10);
        if g == 0 {
            mpool.new_dust(
                &mut probe, &dc, &w, Vec2::new(hx, hy), 46, 32,
                67, hvx * 0.5, hvy * 0.5, 90, Color::default(), 1.5,
            );
        }
        assert_eq!(rng, probe, "the head Next(10) + body (:29212/:29214)");
        // State 3 flight: the HEAD gate Next(10) runs first (it is not
        // state-gated, :29209-29219), then the lunge gate Next(3).
        let mut n2 = npc39(1087.0, 984.0, (46.0, 32.0));
        n2.ai[0] = 3.0;
        n2.ai[1] = 5.0;
        n2.vx = 1.0;
        n2.vy = 1.0;
        let mut rng2 = UnifiedRandom::new(43);
        let mut probe2 = rng2.clone();
        let mut pool2 = DustPool::new();
        let mut mpool2 = DustPool::new();
        style39_step(&mut n2, &w, &mut c, &[cand(5000.0, 2000.0)], &mut rng2, &mut pool2, &dc);
        let g1 = probe2.next_max(10);
        if g1 == 0 {
            mpool2.new_dust(
                &mut probe2, &dc, &w, Vec2::new(1087.0, 984.0), 46, 32,
                67, 0.5, 0.5, 90, Color::default(), 1.5,
            );
        }
        let g2 = probe2.next_max(3);
        if g2 < 2 {
            mpool2.new_dust(
                &mut probe2, &dc, &w, Vec2::new(1087.0, 984.0), 46, 32,
                67, 0.5, 0.5, 90, Color::default(), 1.5,
            );
        }
        assert_eq!(rng2, probe2, "the head + lunge gates (:29212/:29424)");
    }
}


