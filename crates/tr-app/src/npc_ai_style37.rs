//! The Destroyer's machine — `NPC.AI_037_Destroyer` (`NPC.cs:49756-50242`,
//! dispatched by `else if (this.aiStyle == 37)` at `NPC.cs:28991-28994`).
//! S1-D-3 (aiStyle-37) machine + **S1-D-3-wire-destroyer wiring**: the sim
//! runs the family through `world_sim/npcs.rs::update_destroyers` (the
//! `destroyer_family_step` driver below) — the aiStyle-6 worm pass skips the
//! family (`is_worm()` no longer lists it) and `npc.rs`'s kind arm is a no-op
//! with the machine claimed here. See docs/ANALYSIS.md "The Destroyer
//! machine (aiStyle-37)".
//!
//! The C# family is three types — the head (134), the body (135) and the tail
//! (136), all aiStyle 37 (`SetDefaults`, `NPC.cs:10659-10719`: 38×38, scale
//! 1.25, noGravity/noTileCollide, alpha 255, lifeMax 80000; the head dmg 70 /
//! def 0 / value 120000 / boss, the body dmg 55 / def 30, the tail dmg 40 /
//! def 35). The port ships all three (`NpcKind::TheDestroyer/Body/Tail`);
//! the machine stays **type-parameterised** (`DestroyerCtx::npc_type`).
//!
//! Determinism: the machine's only `Main.rand` draws are the body's laser
//! cadence — `Next(4)` (`NPC.cs:49858`) and `Next(1400, 26000)` (`:49859`)
//! **every tick** (the threshold comparison draws even when it fails), then
//! four `Next(-20, 21)` draws (`:49866`, `:49867`, `:49871`, `:49872`) only
//! inside the `Collision.CanHit` gate, in that order. All take the caller's
//! [`UnifiedRandom`] (the exact `Main.rand` stream) in source order. The
//! witness tests replay the stream draw-for-draw against a clone.
//!
//! Pool indices: the C# reads `Main.npc[(int)this.ai[1]]` (the previous
//! segment) and walks the Mech Queen chain over the 200-slot `Main.npc`
//! array (`Main.maxNPCs = 200`, `InitData.cs:9`; `Main.npc.Length = 201`,
//! `Main.cs:66989`). The port's `App::npcs` compacts every frame, so the
//! caller passes the pool slice + this NPC's index; an `ai[1]` slot outside
//! the slice reads as "missing link" (the C#'s `try/catch` at `:49999-50007`
//! swallows the same failure and keeps the pre-branch aim).

use crate::npc::{Npc, NpcKind};
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::tiledata as td;
use tr_core::{UnifiedRandom, World};

/// The family membership (`NPC.cs:49765`'s `type >= 134 && type <= 136`).
pub fn is_destroyer_family(kind: NpcKind) -> bool {
    matches!(
        kind,
        NpcKind::TheDestroyer | NpcKind::TheDestroyerBody | NpcKind::TheDestroyerTail
    )
}

/// The vanilla type of a family kind (`SetDefaults`, `NPC.cs:10659-10719`).
pub fn family_npc_type(kind: NpcKind) -> i32 {
    match kind {
        NpcKind::TheDestroyer => 134,
        NpcKind::TheDestroyerBody => 135,
        NpcKind::TheDestroyerTail => 136,
        _ => unreachable!("the driver gates on is_destroyer_family"),
    }
}

/// `Main.maxNPCs` (`InitData.cs:9`) — the Mech Queen chain walk's bound
/// (`NPC.cs:49763`).
const MAX_NPCS: i32 = 200;
/// `Main.npc.Length` (`Main.cs:66989`: `new NPC[Main.maxNPCs + 1]`) — the
/// follower branch's bound (`NPC.cs:49996`).
const NPC_ARRAY_LEN: i32 = MAX_NPCS + 1;

/// The per-NPC inputs the machine reads that `Npc` does not carry yet (or
/// that live on `Main` in the C#). The wiring passes the port's stock values
/// (single-player, classic, no Mech Queen, no getGoodWorld) — the same
/// convention as the other difficulty-gated machines (`npc.rs`).
#[derive(Clone, Debug)]
pub struct DestroyerCtx {
    /// `this.type` — 134 (head) / 135 (body) / 136 (tail). The port's only
    /// family kind is `NpcKind::TheDestroyer` → 134.
    pub npc_type: i32,
    /// `base.whoAmI` — this NPC's index in the pool slice.
    pub who_am_i: i32,
    /// `Main.netMode` — the spawn/laser block runs when `!= 1`
    /// (`NPC.cs:49831`); the port is single-player → 0.
    pub net_mode: i32,
    /// `NPC.soundDelay` — no `Npc` field yet; the caller owns the counter
    /// (the C# decrements it in `UpdateNPC`, `NPC.cs:90381-90383`, before
    /// the AI runs).
    pub sound_delay: i32,
    /// `NPC.justHit` (`NPC.cs:50225`) — the wiring's stand-in is
    /// `hurt_cd > 0.0` (the port's recoil convention, `npc.rs`).
    pub just_hit: bool,
    /// `NPC.IsMechQueenUp` (`NPC.cs:6868`) — no Mech Queen in the port
    /// (S1-D-3j) → false; the chain-walk and the ride stay constants-off.
    pub mech_queen_up: bool,
    /// `NPC.mechQueen` — the Queen's pool index (unused while the flag is
    /// false).
    pub mech_queen: i32,
    /// `Main.getGoodWorld` — no port flag → false (the 1.2× accel and the
    /// 100-segment count stay off).
    pub get_good_world: bool,
    /// `NPC.difficulty` (`GameDifficultyLevel.cs:17-21`: Journey 0.5,
    /// Classic 1, Expert 2, Master 3, Legendary 4) — the port runs classic.
    pub difficulty: f32,
}

impl DestroyerCtx {
    /// The port's stock context for a head-kind Destroyer at pool index
    /// `who_am_i` (single-player, classic, no flags).
    pub fn head(who_am_i: i32) -> Self {
        Self {
            npc_type: 134,
            who_am_i,
            net_mode: 0,
            sound_delay: 0,
            just_hit: false,
            mech_queen_up: false,
            mech_queen: 0,
            get_good_world: false,
            difficulty: 1.0,
        }
    }
}

/// The body's laser shot intent (`NPC.cs:49865-49879`): projectile type 100
/// (`num11`), the aim already offset by `velocity * 5`, `timeLeft = 300`
/// (`:49878`). The caller resolves `Projectile.NewProjectile` +
/// `GetSpawnSource_ForProjectile` (the sim's projectile pass, the
/// `wants_sphere` convention).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LaserShot {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    /// `GetAttackDamage_ForProjectiles(22f, 18f)` (`NPC.cs:49873`).
    pub damage: i32,
    pub proj_type: i32,
    pub time_left: i32,
}

/// The side effects the machine cannot apply itself (the port has no dust /
/// lighting / sound / NPC-slot-allocation seams inside `Npc::update`). The
/// caller resolves them; everything the `Npc` struct carries (position,
/// velocity, `ai`, `localAI`, `alpha`, `rotation`, `target`, `netUpdate`) is
/// written in place, in C# order.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DestroyerEvents {
    /// `this.realLife` (`NPC.cs:49786`, `:49836`) — the port's carrier
    /// identity is `worm_id`/`seg_index` (`Npc::carrier_index`), so the
    /// index write rides the caller. `None` = untouched this tick.
    pub real_life: Option<i32>,
    /// The fade-in dust burst: 2× `Dust.NewDust(position, w, h, 182, 0, 0,
    /// 100, default, 2f)` with `noGravity`/`noLight` (`NPC.cs:49799-49804`)
    /// — 0 or 2.
    pub fade_dust: u32,
    /// The orphaned-segment death (`NPC.cs:49824-49829`): `life = 0` is
    /// written in place; `HitEffect()` + `checkDead()` ride the caller.
    pub wants_death: bool,
    /// The head's first-tick chain spawn (`NPC.cs:49833-49855`):
    /// `GetDestroyerSegmentsCount()` links — `j == count` is the tail (136),
    /// the rest bodies (135), each spawned at the parent's bottom-centre
    /// (`(int)(pos.X + width/2), (int)(pos.Y + height)`, `:49847`) with
    /// `ai[3] = realLife = head.whoAmI`, `ai[1] = parent`,
    /// `parent.ai[0] = child` (`:49848-49851`) and a `NetMessage.SendData(23)`
    /// per link (`:49852`). The slot allocation is the sim's job (the
    /// `wants_limbs` convention); `ai[3]` is written in place.
    pub spawn_segments: Option<i32>,
    /// The body's laser (`NPC.cs:49877-49878`).
    pub laser: Option<LaserShot>,
    /// `Lighting.AddLight((int)(centre.X/16), (int)(centre.Y/16),
    /// 0.3f, 0.1f, 0.05f)` (`NPC.cs:49929`) — the tile coords; the colour is
    /// the fixed literal.
    pub add_light: Option<(i32, i32)>,
    /// The daytime escape past the rock layer (`NPC.cs:49968-49977`):
    /// every NPC with `aiStyle == 37` goes `active = false`. The port's
    /// pool has no `active` field — the caller retains.
    pub despawn_family: bool,
    /// `SoundEngine.PlaySound(15, (int)pos.X, (int)pos.Y)` (`NPC.cs:50093`)
    /// — the burrow roar; `ctx.sound_delay` carries the cooldown.
    pub roar: bool,
}

/// `NPC.GetDestroyerSegmentsCount` (`NPC.cs:50945-50952`): 100 on
/// getGoodWorld, else 80.
pub fn destroyer_segments_count(get_good_world: bool) -> i32 {
    if get_good_world {
        return 100;
    }
    80
}

/// The probe launch gate — `NPC.HitEffect`'s type-135 arm
/// (`NPC.cs:89266-89285`), called from `StrikeNPC_Inner` (`:81488`) after
/// the knockback: a LIVING body (`life > 0`, host only) that has not
/// launched yet (`ai[2] == 0`) rolls `Main.rand.Next(25)` (50 with the Mech
/// Queen); a 0 sets `ai[2] = 1`, spawns the Probe (139) at the body's
/// bottom-centre (`(int)(pos.X + width/2), (int)(pos.Y + height)`,
/// `:89278`) and flags `netUpdate`.
///
/// RNG: the `Next(maxValue)` draw happens ONLY when `ai[2] == 0` — the C#'s
/// `&&` short-circuits (`:89275`), so a launched body never draws again.
/// The roll rides the STRIKE's stream (the C# `HitEffect` runs inside
/// `StrikeNPC`, under the projectile/melee update's `Main.rand` swap), not
/// the NPC pass's. The caller resolves the returned bottom-centre into a
/// `NpcKind::Probe` spawn (the port's slot allocation is the sim's job).
pub fn probe_launch_on_hit(
    n: &mut Npc,
    rng: &mut dyn tr_core::loot::DropRng,
    mech_queen_up: bool,
) -> Option<(f32, f32)> {
    // `:89268` — type 135, alive, host (`Main.netMode != 1`; the port is
    // single-player).
    if n.kind != NpcKind::TheDestroyerBody || n.hp <= 0 {
        return None;
    }
    let max_value = if mech_queen_up { 50 } else { 25 };
    // `:89275` — the `ai[2] == 0` gate BEFORE the draw (short-circuit).
    // `next_range(0, max)` is the same single Sample()*max draw as the
    // concrete `next_max` (UnifiedRandom.cs:126-130 vs :140) — the strike
    // paths hand a `dyn DropRng` (the projectile pass's stream seam).
    if n.ai[2] == 0.0 && rng.next_range(0, max_value) == 0 {
        n.ai[2] = 1.0;
        n.net_update = true;
        let (w, h) = n.size();
        // `:89278` — the NewNPC bottom-centre anchor (the integer casts).
        return Some(((n.x + w * 0.5) as i32 as f32, (n.y + h) as i32 as f32));
    }
    None
}

/// `NPC.GetAttackDamage_ForProjectiles` (`NPC.cs:7052-7055`):
/// `(int)Utils.Remap(difficulty, Classic(1), Expert(2), normal, expert)` —
/// `Remap` is `Lerp(toMin, toMax, GetLerpValue(from, to, t, clamped: true))`
/// (`Utils.cs:345-348`, `:315-343`); the clamp keeps Master at the expert
/// value.
pub fn attack_damage_for_projectiles(difficulty: f32, normal: f32, expert: f32) -> i32 {
    // `GetLerpValue(1, 2, difficulty, clamped: true)` (`Utils.cs:317-342`).
    let t = if difficulty < 1.0 {
        0.0
    } else if difficulty > 2.0 {
        1.0
    } else {
        (difficulty - 1.0) / (2.0 - 1.0)
    };
    // `MathHelper.Lerp` — `toMin + (toMax - toMin) * t`.
    let remapped = normal + (expert - normal) * t;
    remapped as i32
}

/// `Utils.RotatedBy(this Vector2, double radians, Vector2 center)`
/// (`Utils.cs:1675-1684`) — the float cos/sin of the double angle, the
/// centre-relative spin. Used by the Mech Queen ride (`NPC.cs:50237`).
pub fn rotated_by(px: f32, py: f32, radians: f64, cx: f32, cy: f32) -> (f32, f32) {
    let num = radians.cos() as f32;
    let num2 = radians.sin() as f32;
    let vx = px - cx;
    let vy = py - cy;
    (cx + vx * num - vy * num2, cy + vx * num2 + vy * num)
}

/// `NPC.GetMechQueenCenter` (`NPC.cs:50954-50961`): the Skeletron Mech
/// (type 127) centre rides 14 px up; every other type is `base.Center`.
pub fn mech_queen_center(queen: &Npc, queen_type: i32) -> (f32, f32) {
    let (cx, cy) = queen.center();
    if queen_type == 127 {
        return (cx, cy - 14.0);
    }
    (cx, cy)
}

/// The family type of a pool NPC for the Mech Queen chain walk
/// (`NPC.cs:49765`: `type >= 134 && type <= 136`).
fn family_type(n: &Npc) -> Option<i32> {
    match n.kind {
        NpcKind::TheDestroyer => Some(134),
        NpcKind::TheDestroyerBody => Some(135),
        NpcKind::TheDestroyerTail => Some(136),
        _ => None,
    }
}

/// The `Main.player[this.target]` read after a selection: the C# indexes
/// the 255-slot array (an absent slot is an inactive, non-dead player at
/// the origin); the port's slice holds the session player(s) and
/// `TargetClosest` itself falls back to slot 0 (`NPC.cs:77731-77734`).
fn target_player<'a>(players: &'a [TargetCandidate], target: i32) -> &'a TargetCandidate {
    players
        .get(target as usize)
        .unwrap_or(&players[0])
}

/// XNA `Rectangle.Intersects` — the strict four-edge overlap
/// (`a.Left < b.Right && b.Left < a.Right && a.Top < b.Bottom &&
/// b.Top < a.Bottom`), used by the head's dive gate (`NPC.cs:49941`).
fn rects_intersect(a: (i32, i32, i32, i32), b: (i32, i32, i32, i32)) -> bool {
    a.0 < b.0 + b.2 && b.0 < a.0 + a.2 && a.1 < b.1 + b.3 && b.1 < a.1 + a.3
}

/// The tile-overlap scan (`NPC.cs:49884-49924`): the NPC's box expanded by
/// one tile left/up and two right/down, clamped to the world; a tile counts
/// when `(nactive() && (tileSolid[type] || (tileSolidTop[type] &&
/// frameY == 0))) || liquid > 64` (`:49912` — note: NO `inActive()` test
/// here, unlike `Collision`'s solid gates) and its 16×16 box strictly
/// overlaps the NPC's (`:49916`).
fn overlaps_solid_or_liquid(world: &World, n: &Npc) -> bool {
    let (w, h) = n.size();
    // `:49884-49887` — the C# `(int)` cast truncates toward zero (kept
    // explicit: `.floor()` would differ for off-world negative positions).
    let mut x0 = (n.x / 16.0).trunc() as i32 - 1;
    let mut x1 = ((n.x + w) / 16.0).trunc() as i32 + 2;
    let mut y0 = (n.y / 16.0).trunc() as i32 - 1;
    let mut y1 = ((n.y + h) / 16.0).trunc() as i32 + 2;
    // `:49888-49903` — the world clamps.
    if x0 < 0 {
        x0 = 0;
    }
    if x1 > world.width {
        x1 = world.width;
    }
    if y0 < 0 {
        y0 = 0;
    }
    if y1 > world.height {
        y1 = world.height;
    }
    // `:49908-49923` — the column-major scan, first overlap wins.
    for k in x0..x1 {
        for l in y0..y1 {
            let Some(t) = world.tile(k, l) else {
                continue; // `Main.tile[k, l] != null`
            };
            // `WorldGen.cs:15715`: `tileSolid[192] = false` after the
            // Living Trees pass (the port's table convention, collision.rs).
            let solid = t.active()
                && ((td::TILE_SOLID.has(t.kind) && t.kind != 192)
                    || (td::TILE_SOLID_TOP.has(t.kind) && t.frame_y == 0));
            if solid || t.liquid > 64 {
                let tx = (k * 16) as f32;
                let ty = (l * 16) as f32;
                if n.x + w > tx && n.x < tx + 16.0 && n.y + h > ty && n.y < ty + 16.0 {
                    return true;
                }
            }
        }
    }
    false
}

/// `NPC.AI_037_Destroyer` (`NPC.cs:49756-50242`) — one tick for the NPC at
/// pool index `me`. `players` is the target-scan slice (the
/// `npc_target::target_closest` convention); `ticker` is
/// `Main.EverLastingTicker` for the `targetSetFrame` stamp (0 until the
/// caller migration, the S1-D-3p convention).
pub fn destroyer_step(
    npcs: &mut [Npc],
    me: usize,
    ctx: &mut DestroyerCtx,
    world: &World,
    players: &[TargetCandidate],
    rng: &mut UnifiedRandom,
    ticker: u64,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
) -> DestroyerEvents {
    let mut ev = DestroyerEvents::default();
    let npc_type = ctx.npc_type;
    // `:49758-49759` — `num` (the Mech Queen chain depth) and `num2` (10).
    let mut num = 0i32;
    let num2 = 10i32;
    // `:49760-49783` — the Mech Queen chain walk (non-head types only):
    // follow `ai[1]` up the chain while the links are active family
    // segments; the head ends the count, ten links without a head reset it.
    if ctx.mech_queen_up && npc_type != 134 {
        let mut num3 = npcs[me].ai[1] as i32;
        while num3 > 0 && num3 < MAX_NPCS {
            let link = usize::try_from(num3)
                .ok()
                .and_then(|i| npcs.get(i))
                .and_then(|n| family_type(n).map(|t| (t, n.ai[1])));
            match link {
                Some((t, next)) => {
                    num += 1;
                    if t == 134 {
                        break;
                    }
                    if num >= num2 {
                        num = 0;
                        break;
                    }
                    num3 = next as i32;
                }
                None => {
                    num = 0;
                    break;
                }
            }
        }
    }
    // `:49784-49787` — `ai[3] > 0` mirrors into `realLife`.
    if npcs[me].ai[3] > 0.0 {
        ev.real_life = Some(npcs[me].ai[3] as i32);
    }
    // `:49788-49791` — the invalid/dead target re-pick. The `== 255`
    // sentinel short-circuits before the player read (the C#'s `||` order).
    {
        let t = npcs[me].target;
        let invalid = t < 0
            || t == 255
            || players.get(t as usize).map_or(false, |p| p.dead);
        if invalid {
            target_closest(&mut npcs[me], world, players, true, 37, ticker);
        }
    }
    // `:49792-49812` — the family fade-in. `:49794`'s `velocity.Length()`
    // is a discarded read. The head always fades; a link fades only while
    // its parent (`ai[1]`) is already half-visible (`alpha < 128`).
    if (134..=136).contains(&npc_type) {
        let parent_alpha = if npc_type == 134 {
            0 // the head's own arm of the `||` — the parent read is skipped
        } else {
            usize::try_from(npcs[me].ai[1] as i32)
                .ok()
                .and_then(|i| npcs.get(i))
                .map_or(255, |n| n.alpha as i32)
        };
        if npc_type == 134 || parent_alpha < 128 {
            if npcs[me].alpha != 0 {
                // `:49799-49804` — 2× dust 182, noGravity/noLight, scale 2.
                // W4-f4 (site 49801): the fade burst now rolls the live
                // pool on the caller's UpdateNPCs stream — ZERO explicit
                // rand draws (type 182 hits no per-type tweak). The
                // position is the pre-integration box (the family driver
                // integrates AFTER this step, the C#'s AI-before-move
                // order). `ev.fade_dust` stays the count for the witnesses.
                ev.fade_dust = 2;
                let (fw, fh) = npcs[me].size();
                for _ in 0..2 {
                    let idx = dust.new_dust(
                        rng, dust_ctx, world,
                        glam::Vec2::new(npcs[me].x, npcs[me].y),
                        fw as i32, fh as i32, 182, 0.0, 0.0, 100,
                        Color::default(), 2.0,
                    );
                    let dd = dust.slot_mut(idx);
                    dd.no_gravity = true;
                    dd.no_light = true;
                }
            }
            // `:49806-49810` — `alpha -= 42`, floored at 0 (the port's u8
            // saturates identically for a non-negative start).
            npcs[me].alpha = npcs[me].alpha.saturating_sub(42);
        }
    }
    // `:49813-49830` — an orphaned link dies: `ai[1] <= 0` or the parent's
    // `life <= 0` → `life = 0` + `HitEffect()` + `checkDead()`.
    if npc_type > 134 {
        let ai1 = npcs[me].ai[1];
        let orphan = ai1 <= 0.0
            || usize::try_from(ai1 as i32)
                .ok()
                .and_then(|i| npcs.get(i))
                .map_or(true, |n| n.hp <= 0);
        if orphan {
            npcs[me].hp = 0;
            ev.wants_death = true;
        }
    }
    // `:49831-49883` — the host-only block (`Main.netMode != 1`).
    if ctx.net_mode != 1 {
        // `:49833-49855` — the head's first tick spawns the chain.
        if npcs[me].ai[0] == 0.0 && npc_type == 134 {
            npcs[me].ai[3] = ctx.who_am_i as f32;
            ev.real_life = Some(ctx.who_am_i);
            let count = destroyer_segments_count(ctx.get_good_world);
            // The link plan (`:49840-49854`): j in 0..=count, type 135
            // (136 at j == count), each at the parent's bottom-centre with
            // ai[3]/realLife = head, ai[1] = parent, parent.ai[0] = child.
            // The slot writes need NewNPC's allocation — the caller's pass.
            ev.spawn_segments = Some(count);
        }
        // `:49856-49882` — the body's laser cadence.
        if npc_type == 135 {
            // `:49858` — `localAI[0] += Main.rand.Next(4)` EVERY tick.
            npcs[me].local_ai[0] += rng.next_max(4) as f32;
            // `:49859` — the threshold draw happens every tick too (the
            // comparison's right side is evaluated regardless).
            let threshold = rng.next_range(1400, 26000) as f32;
            if npcs[me].local_ai[0] >= threshold {
                npcs[me].local_ai[0] = 0.0;
                // `:49862` — the re-aim before the sight gate.
                target_closest(&mut npcs[me], world, players, true, 37, ticker);
                let p = *target_player(players, npcs[me].target);
                let (w, h) = npcs[me].size();
                // `:49863` — `Collision.CanHit(position, w, h, player...)`.
                if crate::projectile::can_hit(
                    world,
                    npcs[me].x,
                    npcs[me].y,
                    w,
                    h,
                    p.x,
                    p.y,
                    p.width as f32,
                    p.height as f32,
                ) {
                    // `:49865` — the muzzle: X float half, Y INTEGER half.
                    let mut mx = npcs[me].x + w * 0.5;
                    let mut my = npcs[me].y + (h as i32 / 2) as f32;
                    // `:49866-49867` — the player centre (float halves)
                    // minus the muzzle, plus the two aim jitters.
                    let mut num8 = p.x + p.width as f32 * 0.5 - mx
                        + rng.next_range(-20, 21) as f32;
                    let mut num9 = p.y + p.height as f32 * 0.5 - my
                        + rng.next_range(-20, 21) as f32;
                    // `:49868-49870` — normalise to speed 8.
                    let num10 = 8.0 / (num8 * num8 + num9 * num9).sqrt();
                    num8 *= num10;
                    num9 *= num10;
                    // `:49871-49872` — the two spread jitters (×0.05).
                    num8 += rng.next_range(-20, 21) as f32 * 0.05;
                    num9 += rng.next_range(-20, 21) as f32 * 0.05;
                    // `:49873-49878` — damage 22/18, type 100, muzzle
                    // offset ×5, timeLeft 300.
                    let damage =
                        attack_damage_for_projectiles(ctx.difficulty, 22.0, 18.0);
                    mx += num8 * 5.0;
                    my += num9 * 5.0;
                    ev.laser = Some(LaserShot {
                        x: mx,
                        y: my,
                        vx: num8,
                        vy: num9,
                        damage,
                        proj_type: 100,
                        time_left: 300,
                    });
                    npcs[me].net_update = true;
                }
            }
        }
    }
    // `:49884-49924` — the tile-overlap probe (`flag2`).
    let mut flag2 = overlaps_solid_or_liquid(world, &npcs[me]);
    // `:49925-49957` — the air/light/dive-gate block.
    if !flag2 {
        // `:49927-49930` — the light rides every family type except a
        // body in its `ai[2] == 1` state.
        if npc_type != 135 || npcs[me].ai[2] != 1.0 {
            let (w, h) = npcs[me].size();
            ev.add_light = Some((
                ((npcs[me].x + (w as i32 / 2) as f32) / 16.0) as i32,
                ((npcs[me].y + (h as i32 / 2) as f32) / 16.0) as i32,
            ));
        }
        npcs[me].local_ai[1] = 1.0;
        // `:49932-49952` — the head dives back down when it has risen
        // above the target and no active player sits within 1000 px.
        if npc_type == 134 {
            let (w, h) = npcs[me].size();
            let rect = (npcs[me].x as i32, npcs[me].y as i32, w as i32, h as i32);
            let p = *target_player(players, npcs[me].target);
            if npcs[me].y > p.y {
                let mut far = true;
                for q in players.iter() {
                    if q.active
                        && rects_intersect(
                            rect,
                            (q.x as i32 - 1000, q.y as i32 - 1000, 2000, 2000),
                        )
                    {
                        far = false;
                        break;
                    }
                }
                if far {
                    flag2 = true;
                }
            }
        }
    } else {
        npcs[me].local_ai[1] = 0.0;
    }
    // `:49958-49978` — the daytime escape: sink, double-sink below the
    // surface, despawn the whole family below the rock layer.
    let mut num18 = 16.0f32;
    let p = *target_player(players, npcs[me].target);
    // `Main.IsItDay()` (`Main.cs:62825-62832`) — `dayTime` (no remixWorld
    // in the port).
    if world.day_time || p.dead {
        flag2 = false;
        npcs[me].vy += 1.0;
        if (npcs[me].y as f64) > world.world_surface * 16.0 {
            npcs[me].vy += 1.0;
            num18 = 32.0;
        }
        if (npcs[me].y as f64) > world.rock_layer * 16.0 {
            ev.despawn_family = true;
        }
    }
    // `:49979-49985` — the accel pair, ×1.2 on getGoodWorld.
    let mut num19 = 0.1f32;
    let mut num20 = 0.15f32;
    if ctx.get_good_world {
        num19 *= 1.2;
        num20 *= 1.2;
    }
    // `:49986-49995` — the snapped aim: both centres floored to the 16-px
    // grid (the player's halves are INTEGER divisions, the NPC's centre
    // uses float halves before the snap).
    let (w, h) = npcs[me].size();
    let mut vector3 = (npcs[me].x + w * 0.5, npcs[me].y + h * 0.5);
    let mut num21 = p.x + (p.width / 2) as f32;
    let mut num22 = p.y + (p.height / 2) as f32;
    num21 = (num21 / 16.0).trunc() * 16.0;
    num22 = (num22 / 16.0).trunc() * 16.0;
    vector3.0 = (vector3.0 / 16.0).trunc() * 16.0;
    vector3.1 = (vector3.1 / 16.0).trunc() * 16.0;
    num21 -= vector3.0;
    num22 -= vector3.1;
    let mut num23 = (num21 * num21 + num22 * num22).sqrt();
    // `:49996-50034` — the follower branch: a link chases its parent.
    if npcs[me].ai[1] > 0.0 && npcs[me].ai[1] < NPC_ARRAY_LEN as f32 {
        let mut num24 = (44.0 * npcs[me].scale) as i32;
        // `:50001-50003` — re-aim at the parent's centre (INTEGER halves);
        // the `try/catch` (`:49999-50007`) keeps the snapped player aim
        // when the slot is gone.
        vector3 = (npcs[me].x + w * 0.5, npcs[me].y + h * 0.5);
        let parent = usize::try_from(npcs[me].ai[1] as i32)
            .ok()
            .and_then(|i| npcs.get(i))
            .map(|n| {
                let (pw, ph) = n.size();
                (n.x, n.y, pw, ph)
            });
        if let Some((px, py, pw, ph)) = parent {
            num21 = px + (pw as i32 / 2) as f32 - vector3.0;
            num22 = py + (ph as i32 / 2) as f32 - vector3.1;
        }
        // `:50008-50033` — the C# runs the spacing snap even when the
        // parent read threw (the catch keeps the snapped player aim).
        {
            // `:50008-50020` — the Mech Queen compression: each chain
            // depth shrinks the spacing 10%, floored at 0.
            if num > 0 {
                let mut num25 = num24 as f32 - num24 as f32 * ((num as f32 - 1.0) * 0.1);
                if num25 < 0.0 {
                    num25 = 0.0;
                }
                if num25 > num24 as f32 {
                    num25 = num24 as f32;
                }
                // The C# re-reads the parent here too; a missing slot
                // throws out of the whole branch (the try ends at :50007,
                // so this read is UNGUARDED in the source — the port keeps
                // the aim when the slot is gone, the same net effect as
                // the head's own despawn on a broken chain).
                if let Some((_, py, _, ph)) = parent {
                    num22 = py + (ph as i32 / 2) as f32 + num25 - vector3.1;
                }
            }
            // `:50021-50033` — the spacing snap: rotate toward the parent,
            // then teleport to exactly `num24` px away (velocity zeroed).
            npcs[me].rotation = num22.atan2(num21) + 1.57;
            num23 = (num21 * num21 + num22 * num22).sqrt();
            if num > 0 {
                // `:50025` — INTEGER division first: `num24 / num2 * num`.
                num24 = num24 / num2 * num;
            }
            num23 = (num23 - num24 as f32) / num23;
            num21 *= num23;
            num22 *= num23;
            npcs[me].vx = 0.0;
            npcs[me].vy = 0.0;
            npcs[me].x += num21;
            npcs[me].y += num22;
            // `:50033` — the final rotation re-reads the parent's centre
            // against the PRE-move centre (the decompiler's `x:`/`y:`
            // named args are the standard `Atan2(y, x)` order).
            if let Some((px, py, pw, ph)) = parent {
                npcs[me].rotation = (py + (ph as i32 / 2) as f32 - vector3.1)
                    .atan2(px + (pw as i32 / 2) as f32 - vector3.0)
                    + 1.57;
            }
        }
    } else {
        // `:50037-50205` — the head's free movement.
        if !flag2 {
            // `:50039-50077` — the air approach: re-pick, gravity 0.15
            // toward the cap, then the three-way horizontal steer.
            target_closest(&mut npcs[me], world, players, true, 37, ticker);
            npcs[me].vy += 0.15;
            if npcs[me].vy > num18 {
                npcs[me].vy = num18;
            }
            if ((npcs[me].vx.abs() + npcs[me].vy.abs()) as f64) < num18 as f64 * 0.4 {
                if npcs[me].vx < 0.0 {
                    npcs[me].vx -= num19 * 1.1;
                } else {
                    npcs[me].vx += num19 * 1.1;
                }
            } else if npcs[me].vy == num18 {
                if npcs[me].vx < num21 {
                    npcs[me].vx += num19;
                } else if npcs[me].vx > num21 {
                    npcs[me].vx -= num19;
                }
            } else if npcs[me].vy > 4.0 {
                if npcs[me].vx < 0.0 {
                    npcs[me].vx += num19 * 0.9;
                } else {
                    npcs[me].vx -= num19 * 0.9;
                }
            }
        } else {
            // `:50081-50094` — the burrow roar on the sound cooldown.
            if ctx.sound_delay == 0 {
                let mut num26 = num23 / 40.0;
                if num26 < 10.0 {
                    num26 = 10.0;
                }
                if num26 > 20.0 {
                    num26 = 20.0;
                }
                ctx.sound_delay = num26 as i32;
                ev.roar = true;
            }
            // `:50095-50100` — re-normalise the snapped aim to the cap;
            // `num27`/`num28` latch the UNSCALED magnitudes.
            num23 = (num21 * num21 + num22 * num22).sqrt();
            let num27 = num21.abs();
            let num28 = num22.abs();
            let num29 = num18 / num23;
            num21 *= num29;
            num22 *= num29;
            // `:50101-50119` — both axes aligned: the fast 0.15 approach.
            let vx = npcs[me].vx;
            let vy = npcs[me].vy;
            if ((vx > 0.0 && num21 > 0.0) || (vx < 0.0 && num21 < 0.0))
                && ((vy > 0.0 && num22 > 0.0) || (vy < 0.0 && num22 < 0.0))
            {
                if npcs[me].vx < num21 {
                    npcs[me].vx += num20;
                } else if npcs[me].vx > num21 {
                    npcs[me].vx -= num20;
                }
                if npcs[me].vy < num22 {
                    npcs[me].vy += num20;
                } else if npcs[me].vy > num22 {
                    npcs[me].vy -= num20;
                }
            }
            // `:50120-50160` — any axis aligned: the 0.1 approach plus the
            // two perpendicular turn boosts.
            let vx = npcs[me].vx;
            let vy = npcs[me].vy;
            if (vx > 0.0 && num21 > 0.0)
                || (vx < 0.0 && num21 < 0.0)
                || (vy > 0.0 && num22 > 0.0)
                || (vy < 0.0 && num22 < 0.0)
            {
                if npcs[me].vx < num21 {
                    npcs[me].vx += num19;
                } else if npcs[me].vx > num21 {
                    npcs[me].vx -= num19;
                }
                if npcs[me].vy < num22 {
                    npcs[me].vy += num19;
                } else if npcs[me].vy > num22 {
                    npcs[me].vy -= num19;
                }
                if (num22.abs() as f64) < num18 as f64 * 0.2
                    && ((npcs[me].vx > 0.0 && num21 < 0.0)
                        || (npcs[me].vx < 0.0 && num21 > 0.0))
                {
                    if npcs[me].vy > 0.0 {
                        npcs[me].vy += num19 * 2.0;
                    } else {
                        npcs[me].vy -= num19 * 2.0;
                    }
                }
                if (num21.abs() as f64) < num18 as f64 * 0.2
                    && ((npcs[me].vy > 0.0 && num22 < 0.0)
                        || (npcs[me].vy < 0.0 && num22 > 0.0))
                {
                    if npcs[me].vx > 0.0 {
                        npcs[me].vx += num19 * 2.0;
                    } else {
                        npcs[me].vx -= num19 * 2.0;
                    }
                }
            } else if num27 > num28 {
                // `:50161-50182` — the horizontal-dominant turn.
                if npcs[me].vx < num21 {
                    npcs[me].vx += num19 * 1.1;
                } else if npcs[me].vx > num21 {
                    npcs[me].vx -= num19 * 1.1;
                }
                if ((npcs[me].vx.abs() + npcs[me].vy.abs()) as f64)
                    < num18 as f64 * 0.5
                {
                    if npcs[me].vy > 0.0 {
                        npcs[me].vy += num19;
                    } else {
                        npcs[me].vy -= num19;
                    }
                }
            } else {
                // `:50183-50204` — the vertical-dominant turn.
                if npcs[me].vy < num22 {
                    npcs[me].vy += num19 * 1.1;
                } else if npcs[me].vy > num22 {
                    npcs[me].vy -= num19 * 1.1;
                }
                if ((npcs[me].vx.abs() + npcs[me].vy.abs()) as f64)
                    < num18 as f64 * 0.5
                {
                    if npcs[me].vx > 0.0 {
                        npcs[me].vx += num19;
                    } else {
                        npcs[me].vx -= num19;
                    }
                }
            }
        }
        // `:50206` — the velocity tilt (the 1.57f literal, NOT π/2).
        npcs[me].rotation = npcs[me].vy.atan2(npcs[me].vx) + 1.57;
        // `:50207-50229` — the head's net-sync flags: the burrow state
        // flip and the velocity sign reversals (unless justHit).
        if npc_type == 134 {
            if flag2 {
                if npcs[me].local_ai[0] != 1.0 {
                    npcs[me].net_update = true;
                }
                npcs[me].local_ai[0] = 1.0;
            } else {
                if npcs[me].local_ai[0] != 0.0 {
                    npcs[me].net_update = true;
                }
                npcs[me].local_ai[0] = 0.0;
            }
            if ((npcs[me].vx > 0.0 && npcs[me].old_vx < 0.0)
                || (npcs[me].vx < 0.0 && npcs[me].old_vx > 0.0)
                || (npcs[me].vy > 0.0 && npcs[me].old_vy < 0.0)
                || (npcs[me].vy < 0.0 && npcs[me].old_vy > 0.0))
                && !ctx.just_hit
            {
                npcs[me].net_update = true;
            }
        }
    }
    // `:50231-50241` — the Mech Queen ride: the head hangs 100 px below
    // the Queen's centre, spun by her horizontal velocity.
    if ctx.mech_queen_up && npc_type == 134 {
        let queen = usize::try_from(ctx.mech_queen)
            .ok()
            .and_then(|i| npcs.get(i));
        if let Some(queen) = queen {
            // The Queen is the Skeletron Mech (127) in the C# fight.
            let (qcx, qcy) = mech_queen_center(queen, 127);
            let qvx = queen.vx;
            let qvy = queen.vy;
            let num30 = qvx * 0.025;
            let (rx, ry) = rotated_by(qcx, qcy + 100.0, num30 as f64, qcx, qcy);
            let (w, h) = npcs[me].size();
            npcs[me].x = rx - w / 2.0 + qvx;
            npcs[me].y = ry - h / 2.0 + qvy;
            npcs[me].vx = 0.0;
            npcs[me].vy = 0.0;
            npcs[me].rotation = num30 * 0.75 + std::f32::consts::PI;
        }
    }
    ev
}

/// The Probe's shot intent (`NPC.cs:50609-50625`): projectile type 84 at
/// the SNAPPED centre (`vector`, the 8-px grid), velocity = the snapped aim
/// (`num4`/`num5` — NOT re-normalised in the classic fight), damage
/// `GetAttackDamage_ForProjectiles(25, 22)` (`:50618`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProbeShot {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub damage: i32,
    pub proj_type: i32,
}

/// The Probe's machine — `AI_005_EaterOfSouls`'s type-139 path
/// (`NPC.cs:50244-50961`, the arm dispatched by aiStyle 5). The classic
/// solo port (no Mech Queen, no zenithWorld, single-player host):
///
/// * `:50250-50252` — the target re-pick runs EVERY tick (the C#'s
///   `target <= 255` middle term is always true — the port's
///   `target_closest`, aiStyle 5).
/// * `:50294-50295` — `num = 6`, `num2 = 0.05` (the default row; 139's own
///   `num = 3` branch is zenithWorld-only, `:50414-50417`).
/// * `:50418-50444` — the 8-px snapped aim at speed `num`.
/// * `:50445-50479` — the `num8` wobble: 139 IS in the `num8` set, so while
///   the raw distance > 100 the `ai[0]` counter drives the ±0.023 drifts
///   (the same family block the Eater of Souls rides).
/// * `:50489-50545` — the `ai[3] != 0` Mech-Queen ride is constants-off;
///   the else arm clears `dontTakeDamage` (`:50546-50549`) and runs the
///   plain approach (`flag5` false for 139 — no double-add, `:50447`).
/// * `:50593-50641` — the laser cadence: `localAI[0]++` every tick
///   (`+2` more while riding, `justHit` resets), fires at ≥ 120 when the
///   target exists, the firing-distance rect holds and `CanHit` passes;
///   then the tile-light probe and the sprite-aim rotation
///   (`atan2(num5, num4)`, +3.14 when the aim points left — `:50633-50641`).
/// * `:50656-50707` — the collide-bounce block: 139 is on the family list
///   (`num14 = 0.7`) but `noTileCollide` (`SetDefaults :10733`) means the
///   port integrates straight through (the `hover_step` convention); the
///   trail dust excludes 139 (`:50695`), so NO rand draws ride it.
/// * `:50813-50831` — the far-chase boost: when the snapped distance > 600
///   and the velocity aligns with the aim, `vx *= 1.05` below 12; else
///   `vx *= 0.9`.
///
/// RNG: the type-139 path draws NOTHING from `Main.rand` (the wobble is a
/// counter, the dust arms exclude 139) — the function takes no rng, so the
/// `UpdateNPCs` stream is untouched BY CONSTRUCTION.
pub fn probe_step(
    n: &mut Npc,
    world: &World,
    players: &[TargetCandidate],
    just_hit: bool,
    ticker: u64,
) -> Option<ProbeShot> {
    // `:50250-50252` — `target < 0 || target <= 255 || dead`: the middle
    // term is ALWAYS true (the 255-slot array), so AI_005 re-picks the
    // target EVERY tick — unlike AI_037's `== 255` sentinel gate.
    target_closest(n, world, players, true, 5, ticker);
    let p = *target_player(players, n.target);
    // `:50294-50295` — the default speed pair.
    let num = 6.0f32;
    let num2 = 0.05f32;
    let (w, h) = n.size();
    // `:50418-50426` — the 8-px snap on both centres (the player's halves
    // are INTEGER divisions, `:50419-50420`).
    let mut vector = (n.x + w * 0.5, n.y + h * 0.5);
    let mut num4 = p.x + (p.width / 2) as f32;
    let mut num5 = p.y + (p.height / 2) as f32;
    num4 = (num4 / 8.0).trunc() * 8.0;
    num5 = (num5 / 8.0).trunc() * 8.0;
    vector.0 = (vector.0 / 8.0).trunc() * 8.0;
    vector.1 = (vector.1 / 8.0).trunc() * 8.0;
    num4 -= vector.0;
    num5 -= vector.1;
    let mut num6 = (num4 * num4 + num5 * num5).sqrt();
    let num7 = num6;
    // `:50429-50432` — `flag3` (the > 600 px chase flag).
    let flag3 = num6 > 600.0;
    // `:50433-50444` — the normalised aim; a zero distance keeps velocity.
    if num6 == 0.0 {
        num4 = n.vx;
        num5 = n.vy;
    } else {
        num6 = num / num6;
        num4 *= num6;
        num5 *= num6;
    }
    // `:50445-50473` — the `num8` wobble (139 is in the set, `:50445`;
    // `flag4` false). The `|` operands are plain bools — no short-circuit
    // difference. The inner gate is the RAW distance > 100.
    if num7 > 100.0 {
        n.ai[0] += 1.0;
        if n.ai[0] > 0.0 {
            n.vy += 0.023;
        } else {
            n.vy -= 0.023;
        }
        if n.ai[0] < -100.0 || n.ai[0] > 100.0 {
            n.vx += 0.023;
        } else {
            n.vx -= 0.023;
        }
        if n.ai[0] > 200.0 {
            n.ai[0] = -200.0;
        }
    }
    // `:50480-50484` — the dead-target flee vector.
    if p.dead {
        num4 = n.facing as f32 * num / 2.0;
        num5 = (0.0 - num) / 2.0;
    }
    // `:50489-50549` — the Mech Queen ride is constants-off; the else arm
    // clears the shield (`:50546-50548`) and approaches (`flag5` false).
    n.dont_take_damage = false;
    if n.vx < num4 {
        n.vx += num2;
    } else if n.vx > num4 {
        n.vx -= num2;
    }
    if n.vy < num5 {
        n.vy += num2;
    } else if n.vy > num5 {
        n.vy -= num2;
    }
    // `:50593-50641` — the laser cadence + the sprite-aim rotation.
    let mut shot = None;
    n.local_ai[0] += 1.0;
    // `:50596-50598` — the +2 ride bonus is Mech-Queen-only (off).
    // `:50599-50602` — `justHit` resets the counter (the port's `hurt_cd`
    // stand-in rides the caller's flag).
    if just_hit {
        n.local_ai[0] = 0.0;
    }
    // `:50603-50607` — the 120-tick threshold (360 with the Queen, off).
    if n.local_ai[0] >= 120.0 {
        n.local_ai[0] = 0.0;
        // `:50611` — `(targetData.Type != None) & flag2` then `CanHit`.
        // A dead player is still a Player target (the C# gate does not
        // read `dead` — the flee vector only bends the aim).
        // The firing-distance rect (`AI_GlobalFiringDistanceCheck`,
        // `NPC.cs:55835-55842`): the target centre's 1920×1200 view rect
        // deflated 50, containing the probe's centre.
        let pcx = p.x + p.width as f32 * 0.5;
        let pcy = p.y + p.height as f32 * 0.5;
        let in_range = (vector.0 - pcx).abs() <= 1920.0 / 2.0 - 50.0
            && (vector.1 - pcy).abs() <= 1200.0 / 2.0 - 50.0;
        if in_range
            && crate::projectile::can_hit(
                world, n.x, n.y, w, h, p.x, p.y, p.width as f32, p.height as f32,
            )
        {
            // `:50618-50625` — damage 25/22, type 84, the SNAPPED centre
            // and the snapped aim as the velocity (the Queen's
            // re-normalised 8-speed form is off).
            shot = Some(ProbeShot {
                x: vector.0,
                y: vector.1,
                vx: num4,
                vy: num5,
                damage: attack_damage_for_projectiles(1.0, 25.0, 22.0),
                proj_type: 84,
            });
        }
    }
    // `:50627-50632` — the tile-light probe (no port lighting seam inside
    // the AI; the fixed 0.3/0.1/0.05 colour rides the sim's light pass
    // convention — SKIPPED like the other machines' AddLight lines).
    // `:50633-50641` — the sprite-aim rotation (the type-23 shape).
    if num4 > 0.0 {
        n.sprite_dir = 1;
        n.rotation = num5.atan2(num4);
    }
    if num4 < 0.0 {
        n.sprite_dir = -1;
        n.rotation = num5.atan2(num4) + 3.14;
    }
    // `:50656-50707` — the bounce block: `noTileCollide` (`:10733`) means
    // the direct integration (the collideX/Y flags never arm); the dust
    // arm excludes 139 (`:50695`) — no draws.
    n.x += n.vx;
    n.y += n.vy;
    // `:50813-50831` — the far-chase boost (the non-short-circuit `&` is
    // two plain bools).
    if flag3 {
        if (n.vx > 0.0 && num4 > 0.0) || (n.vx < 0.0 && num4 < 0.0) {
            // `num25 = 12` (5 with the Queen, off).
            if n.vx.abs() < 12.0 {
                n.vx *= 1.05;
            }
        } else {
            n.vx *= 0.9;
        }
    }
    // The off-world guard (the framework shape, `worm_head_step`).
    if n.y > world.height as f32 * 16.0 + 200.0 {
        n.hp = 0;
    }
    shot
}

/// One tick of the whole family — the sim driver's core (the pieces the
/// tests drive directly). Runs AFTER the per-NPC step (the `AI_006` pass's
/// position, `world_sim/npcs.rs::update_destroyers`):
///
/// 1. the head's first tick allocates the chain (the C#'s `NewNPC` loop,
///    `NPC.cs:49833-49855` — j 0..=count, 135s then the 136 at j == count,
///    each at the parent's bottom-centre; `NewNPC` draws no rand);
/// 2. the `realLife` pool pays parked damage and mirrors the head's life
///    onto the links (`NPC.cs:81374-81378`, the worm pass's precedent);
/// 3. every family slot runs `destroyer_step` in pool order (the C#'s
///    `UpdateNPC` slot order — the head's slot is below its children's,
///    `GetAvailableNPCSlot` searches upward from the head's `Start`).
///
/// Slot mapping: the C#'s `ai[1]`/`ai[3]` slot links cannot survive the
/// port's compacting pool, so the driver runs the machine over a SCRATCH
/// pool of the family alone with a dummy slot 0 — the C#'s
/// `CannotSpawnInSlot0` (`NPCID.cs:10452` lists 134/135/136) guarantees no
/// family member ever sits at slot 0, so `ai[1] = parent_scratch_index`
/// keeps the machine's `ai[1] > 0` gate verbatim. The stable carrier
/// identity stays `worm_id`/`seg_index` (`Npc::carrier_index`).
///
/// RNG: only `destroyer_step`'s body cadence draws (2 per body per tick,
/// +4 on a firing tick), in pool order on the caller's `UpdateNPCs`
/// stream. The chain spawn and the pool mirror draw NOTHING.
///
/// Returns `(pool_index, events)` per family slot, in tick order.
pub fn destroyer_family_step(
    npcs: &mut Vec<Npc>,
    world: &World,
    players: &[TargetCandidate],
    rng: &mut UnifiedRandom,
    next_worm_id: &mut u32,
    ticker: u64,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
) -> Vec<(usize, DestroyerEvents)> {
    // 1. The chain spawn (the head's first tick, `ai[0] == 0`).
    let head_idx = npcs
        .iter()
        .position(|n| n.kind == NpcKind::TheDestroyer && n.hp > 0 && n.ai[0] == 0.0);
    if let Some(head) = head_idx {
        if npcs[head].worm_id == 0 {
            npcs[head].worm_id = *next_worm_id;
            *next_worm_id += 1;
        }
        let count = destroyer_segments_count(false);
        // `:49836-49837` — `ai[3] = realLife = whoAmI` (the carrier stamp;
        // the port's pool index stands in for the slot).
        npcs[head].ai[3] = head as f32;
        let (hw, hh) = npcs[head].size();
        // `:49847` — the bottom-centre anchor (the integer casts).
        let spawn_x = (npcs[head].x + hw * 0.5) as i32 as f32;
        let spawn_y = (npcs[head].y + hh) as i32 as f32;
        let wid = npcs[head].worm_id;
        for j in 0..=count {
            // `:49842-49846` — 136 at j == count, else 135.
            let kind = if j == count {
                NpcKind::TheDestroyerTail
            } else {
                NpcKind::TheDestroyerBody
            };
            let (cw, ch) = kind.size();
            // `NewNPC`'s `Bottom = (X, Y)` (`NPC.cs:80770`): the box's
            // bottom-centre lands on the anchor.
            let mut link = Npc::new(kind, spawn_x - cw * 0.5, spawn_y - ch, 1, 0.0);
            link.hp = link.max_hp;
            link.worm_id = wid;
            link.seg_index = (j + 1) as u32;
            link.ai[3] = head as f32;
            npcs.push(link);
        }
        // `:49851` — `parent.ai[0] = child` closes the first-tick gate
        // (any non-zero value; the port stores the first child's index+1).
        npcs[head].ai[0] = (head + 1) as f32;
    }
    // 2. The realLife pool (`NPC.cs:81374-81378`): parked damage pays the
    // carrier, then every link mirrors the head's life/lifeMax.
    let mut carrier_of = std::collections::HashMap::new();
    for (i, n) in npcs.iter().enumerate() {
        if is_destroyer_family(n.kind) && n.seg_index == 0 && n.worm_id != 0 {
            carrier_of.insert(n.worm_id, i);
        }
    }
    for i in 0..npcs.len() {
        if !is_destroyer_family(npcs[i].kind) {
            continue;
        }
        let dmg = npcs[i].pending_pool_damage;
        if dmg != 0 {
            npcs[i].pending_pool_damage = 0;
            if let Some(&c) = carrier_of.get(&npcs[i].worm_id) {
                npcs[c].hp -= dmg;
            }
        }
    }
    for i in 0..npcs.len() {
        if !is_destroyer_family(npcs[i].kind) || npcs[i].seg_index == 0 {
            continue;
        }
        if let Some(&c) = carrier_of.get(&npcs[i].worm_id) {
            npcs[i].hp = npcs[c].hp;
            npcs[i].max_hp = npcs[c].max_hp;
        }
    }
    // 3. The machine pass over the scratch pool (dummy slot 0).
    let family: Vec<usize> = npcs
        .iter()
        .enumerate()
        .filter(|(_, n)| is_destroyer_family(n.kind) && n.hp > 0)
        .map(|(i, _)| i)
        .collect();
    if family.is_empty() {
        return Vec::new();
    }
    let mut scratch: Vec<Npc> = Vec::with_capacity(family.len() + 1);
    scratch.push(npcs[family[0]].clone()); // the dummy slot 0 (never read)
    let mut slot_of = std::collections::HashMap::new();
    for &i in &family {
        slot_of.insert(i, scratch.len());
        scratch.push(npcs[i].clone());
    }
    // The parent link: seg k's parent is seg k-1 of the same chain (the
    // C#'s ai[1] write, `:49850`); the head keeps ai[1] = 0.
    let mut parent_slot = std::collections::HashMap::new();
    for &i in &family {
        let key = (npcs[i].worm_id, npcs[i].seg_index);
        parent_slot.insert(key, slot_of[&i]);
    }
    let mut out = Vec::with_capacity(family.len());
    for &i in &family {
        let me = slot_of[&i];
        // `NPC.cs:92538` — `oldVelocity = velocity` right before the
        // position integration (the port's no-collision machines stamp it
        // at the AI tail, the poltergeist/style10 precedent).
        scratch[me].old_vx = scratch[me].vx;
        scratch[me].old_vy = scratch[me].vy;
        let seg = scratch[me].seg_index;
        scratch[me].ai[1] = if seg == 0 {
            0.0
        } else {
            parent_slot
                .get(&(scratch[me].worm_id, seg - 1))
                .map(|&s| s as f32)
                .unwrap_or(0.0)
        };
        let mut ctx = DestroyerCtx::head(me as i32);
        ctx.npc_type = family_npc_type(scratch[me].kind);
        ctx.just_hit = scratch[me].hurt_cd > 0.0;
        // `NPC.cs:90381-90383` — `soundDelay--` once per update, BEFORE
        // the AI runs (the roar's cooldown decay).
        scratch[me].sound_delay = (scratch[me].sound_delay - 1).max(0);
        ctx.sound_delay = scratch[me].sound_delay;
        let ev = destroyer_step(&mut scratch, me, &mut ctx, world, players, rng, ticker, dust, dust_ctx);
        scratch[me].sound_delay = ctx.sound_delay;
        // `noTileCollide` (`NPC.cs:10670-10671`): the direct integration
        // (`NPC.cs:90495-90498`, the `integrate_no_tile_collide` shape).
        // The follower branch zeroed the velocity, so its snap position
        // rides unchanged; the head integrates its steer.
        scratch[me].x += scratch[me].vx;
        scratch[me].y += scratch[me].vy;
        // Copy the machine's writes back (ai[1] holds the scratch link —
        // the driver re-stamps it every tick; the stable identity is
        // worm_id/seg_index).
        let mut updated = scratch[me].clone();
        updated.ai[1] = npcs[i].ai[1];
        npcs[i] = updated;
        out.push((i, ev));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

    fn air() -> World {
        let mut w = World::new("t".into(), 200, 120);
        w.day_time = false;
        w
    }

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

    fn head(x: f32, y: f32) -> Npc {
        // The port's Destroyer row: 48×48 (the C#'s 38×38 at scale 1.25),
        // scale 1.0 in the port (the size rides the box) — the spacing
        // tests set scale explicitly where the C# reads it.
        let mut n = Npc::new(NpcKind::TheDestroyer, x, y, 1, 0.0);
        n.alpha = 255; // the SetDefaults fade-in start (`NPC.cs:10678`)
        n.boss = true;
        n
    }

    /// `NPC.cs:50945-50952` — 80 segments, 100 on getGoodWorld.
    #[test]
    fn s1_d_3_s37_the_segment_count_matches_the_csharp() {
        assert_eq!(destroyer_segments_count(false), 80);
        assert_eq!(destroyer_segments_count(true), 100);
    }

    /// `NPC.cs:7052-7055` + `Utils.cs:315-348` — the laser damage remaps
    /// 22 → 18 across Classic → Expert and clamps past Expert.
    #[test]
    fn s1_d_3_s37_the_laser_damage_remaps_and_clamps() {
        assert_eq!(attack_damage_for_projectiles(1.0, 22.0, 18.0), 22);
        assert_eq!(attack_damage_for_projectiles(2.0, 22.0, 18.0), 18);
        assert_eq!(attack_damage_for_projectiles(3.0, 22.0, 18.0), 18);
        assert_eq!(attack_damage_for_projectiles(0.5, 22.0, 18.0), 22);
        assert_eq!(attack_damage_for_projectiles(1.5, 22.0, 18.0), 20);
    }

    /// `NPC.cs:49792-49811` — the head fades 42/tick from 255 with a
    /// 2-dust burst while visible, floored at 0; a body whose parent is
    /// still opaque (`alpha >= 128`) does not fade.
    #[test]
    fn s1_d_3_s37_the_fade_in_steps_42_and_gates_on_the_parent() {
        let w = air();
        let players = [cand(1000.0, 1000.0)];
        let mut rng = UnifiedRandom::new(7);
        let mut pool = [head(500.0, 500.0)];
        let mut ctx = DestroyerCtx::head(0);
        let ev = destroyer_step(&mut pool, 0, &mut ctx, &w, &players, &mut rng, 0, &mut DustPool::new(), &crate::npc::permissive_dust_ctx(&w));
        assert_eq!(pool[0].alpha, 213, "255 - 42 (:49806)");
        assert_eq!(ev.fade_dust, 2);
        // The body behind an opaque parent keeps its alpha.
        let mut pool2 = [head(500.0, 500.0), head(500.0, 548.0)];
        pool2[1].ai[1] = 0.0; // parent slot 0, alpha 255
        let mut ctx2 = DestroyerCtx::head(1);
        ctx2.npc_type = 135;
        let ev2 = destroyer_step(&mut pool2, 1, &mut ctx2, &w, &players, &mut rng, 0, &mut DustPool::new(), &crate::npc::permissive_dust_ctx(&w));
        assert_eq!(pool2[1].alpha, 255, "the parent is not half-visible yet");
        assert_eq!(ev2.fade_dust, 0);
        // Once the parent drops below 128 the body fades too.
        pool2[0].alpha = 127;
        destroyer_step(&mut pool2, 1, &mut ctx2, &w, &players, &mut rng, 0, &mut DustPool::new(), &crate::npc::permissive_dust_ctx(&w));
        assert_eq!(pool2[1].alpha, 213);
    }

    /// W4-f4 — site 49801 (`NPC.cs:49799-49804`): the fade burst rolls the
    /// live pool on the caller's stream. A fading body (parent alpha 100 <
    /// 128) draws 2× new_dust internals (type 182, no per-type tweak) THEN
    /// the cadence Next(4)+Next(1400,26000) — the fade block (`:49792`)
    /// precedes the type-135 laser arm (`:49856`). The mirror replays
    /// exactly that; the dust records match the mirrored pool draw-for-
    /// draw, with the noGravity/noLight posts. The opaque-parent gate
    /// spawns NOTHING (the cadence draws alone). WRITTEN NOT RUN.
    #[test]
    fn w4_f4_the_destroyer_fade_dust_mirrors_site_49801() {
        let w = air();
        let players = [cand(1000.0, 1000.0)];
        let mut pool = [head(500.0, 500.0), head(500.0, 548.0)];
        pool[0].alpha = 100; // the parent is half-visible
        pool[1].ai[1] = 0.0; // parent slot 0
        let mut ctx = DestroyerCtx::head(1);
        ctx.npc_type = 135;
        let dctx = crate::npc::permissive_dust_ctx(&w);
        let (bw, bh) = pool[1].size();
        let mut rng = UnifiedRandom::new(23);
        let mut witness = rng.clone();
        let mut mirror = DustPool::new();
        for _ in 0..2 {
            let idx = mirror.new_dust(
                &mut witness, &dctx, &w,
                glam::Vec2::new(pool[1].x, pool[1].y),
                bw as i32, bh as i32, 182, 0.0, 0.0, 100,
                Color::default(), 2.0,
            );
            let dd = mirror.slot_mut(idx);
            dd.no_gravity = true;
            dd.no_light = true;
        }
        let _ = witness.next_max(4);
        let _ = witness.next_range(1400, 26000);
        let mut dust = DustPool::new();
        let ev = destroyer_step(&mut pool, 1, &mut ctx, &w, &players, &mut rng, 0, &mut dust, &dctx);
        assert_eq!(rng, witness, "2 fade bursts then the cadence, in source order");
        assert_eq!(ev.fade_dust, 2);
        assert_eq!(pool[1].alpha, 213, "the fade still steps 42");
        assert_eq!(pool[0].alpha, 100, "the parent is untouched");
        let active: Vec<usize> = (0..6000).filter(|&i| dust.slot(i).active).collect();
        assert_eq!(active.len(), 2, "the 2-dust burst");
        for (j, &i) in active.iter().enumerate() {
            let d = dust.slot(i);
            let m = mirror.slot(j);
            assert_eq!(d.dust_type, 182);
            assert!(d.no_gravity && d.no_light);
            assert_eq!(d.position, m.position);
            assert_eq!(d.velocity, m.velocity);
            assert_eq!(d.scale, m.scale);
            assert_eq!(d.alpha, 100);
            assert_eq!(d.color, Color::default());
        }
        // The opaque-parent gate: no fade, no dust — the cadence alone.
        let mut pool2 = [head(500.0, 500.0), head(500.0, 548.0)];
        pool2[1].ai[1] = 0.0; // parent slot 0, alpha 255
        let mut ctx2 = DestroyerCtx::head(1);
        ctx2.npc_type = 135;
        let mut rng2 = UnifiedRandom::new(23);
        let mut witness2 = rng2.clone();
        let _ = witness2.next_max(4);
        let _ = witness2.next_range(1400, 26000);
        let mut dust2 = DustPool::new();
        let ev2 = destroyer_step(&mut pool2, 1, &mut ctx2, &w, &players, &mut rng2, 0, &mut dust2, &dctx);
        assert_eq!(rng2, witness2, "the gated fade draws nothing");
        assert_eq!(ev2.fade_dust, 0);
        assert!((0..6000).all(|i| !dust2.slot(i).active), "no dust behind an opaque parent");
    }

    /// `NPC.cs:49858-49859` — the cadence draws `Next(4)` then
    /// `Next(1400, 26000)` EVERY tick, in that order; a non-firing tick
    /// consumes exactly those two draws (witness: the stream state equals
    /// a clone advanced by the same two calls).
    #[test]
    fn s1_d_3_s37_the_laser_cadence_draws_two_per_tick_in_order() {
        let w = air();
        let players = [cand(1000.0, 1000.0)];
        let mut pool = [head(500.0, 500.0)];
        pool[0].ai[1] = 0.0;
        let mut ctx = DestroyerCtx::head(0);
        ctx.npc_type = 135;
        let mut rng = UnifiedRandom::new(42);
        let mut witness = rng.clone();
        let d1 = witness.next_max(4);
        let d2 = witness.next_range(1400, 26000);
        let ev = destroyer_step(&mut pool, 0, &mut ctx, &w, &players, &mut rng, 0, &mut DustPool::new(), &crate::npc::permissive_dust_ctx(&w));
        assert_eq!(rng, witness, "exactly Next(4) + Next(1400,26000), in order");
        assert_eq!(pool[0].local_ai[0], d1 as f32);
        assert!(ev.laser.is_none(), "the accumulator starts far below the threshold");
        let _ = d2;
    }

    /// `NPC.cs:49859-49878` — a firing tick consumes the two cadence draws
    /// plus FOUR `Next(-20, 21)` draws (aim X, aim Y, spread X, spread Y —
    /// `:49866`, `:49867`, `:49871`, `:49872`), and the shot is type 100 /
    /// damage 22 / timeLeft 300 at speed ~8 offset ×5 from the muzzle.
    #[test]
    fn s1_d_3_s37_the_firing_tick_draws_four_jitters_and_spawns_type_100() {
        let w = air();
        let players = [cand(1000.0, 500.0)];
        let mut pool = [head(500.0, 500.0)];
        pool[0].local_ai[0] = 30000.0; // above any threshold draw
        let mut ctx = DestroyerCtx::head(0);
        ctx.npc_type = 135;
        let mut rng = UnifiedRandom::new(42);
        let mut witness = rng.clone();
        let _ = witness.next_max(4);
        let _ = witness.next_range(1400, 26000);
        for _ in 0..4 {
            let _ = witness.next_range(-20, 21);
        }
        let ev = destroyer_step(&mut pool, 0, &mut ctx, &w, &players, &mut rng, 0, &mut DustPool::new(), &crate::npc::permissive_dust_ctx(&w));
        assert_eq!(rng, witness, "2 cadence + 4 jitter draws, in source order");
        let shot = ev.laser.expect("the sight line is clear");
        assert_eq!(shot.proj_type, 100);
        assert_eq!(shot.damage, 22);
        assert_eq!(shot.time_left, 300);
        assert_eq!(pool[0].local_ai[0], 0.0, "the accumulator resets (:49861)");
        assert!(pool[0].net_update);
        // Speed 8 ± the 1-px spread, direction +X toward the player.
        let speed = (shot.vx * shot.vx + shot.vy * shot.vy).sqrt();
        assert!((speed - 8.0).abs() < 1.5, "speed {speed}");
        assert!(shot.vx > 0.0);
    }

    /// `NPC.cs:49813-49829` — a body whose parent slot is dead dies
    /// (`life = 0` + the HitEffect/checkDead intent).
    #[test]
    fn s1_d_3_s37_an_orphaned_link_dies() {
        let w = air();
        let players = [cand(1000.0, 1000.0)];
        let mut rng = UnifiedRandom::new(1);
        let mut pool = [head(500.0, 500.0), head(500.0, 548.0)];
        pool[0].hp = 0;
        pool[1].ai[1] = 0.0;
        let mut ctx = DestroyerCtx::head(1);
        ctx.npc_type = 135;
        let ev = destroyer_step(&mut pool, 1, &mut ctx, &w, &players, &mut rng, 0, &mut DustPool::new(), &crate::npc::permissive_dust_ctx(&w));
        assert!(ev.wants_death);
        assert_eq!(pool[1].hp, 0);
    }

    /// `NPC.cs:49996-50033` — the follower snaps to exactly `(int)(44 *
    /// scale)` px from the parent's centre with zero velocity, and the
    /// rotation is `Atan2(dy, dx) + 1.57f`.
    #[test]
    fn s1_d_3_s37_the_follower_snaps_to_the_44_scale_spacing() {
        let w = air();
        let players = [cand(5000.0, 5000.0)];
        let mut rng = UnifiedRandom::new(3);
        // The link at slot 0, its parent (the head) at slot 1 — the C#'s
        // `ai[1] > 0` gate needs a non-zero parent slot.
        let mut pool = [head(1200.0, 1000.0), head(1000.0, 1000.0)];
        pool[0].ai[1] = 1.0;
        pool[0].scale = 1.25; // the SetDefaults row (`NPC.cs:10696`)
        pool[0].alpha = 0; // skip the fade noise
        let mut ctx = DestroyerCtx::head(0);
        ctx.npc_type = 135;
        destroyer_step(&mut pool, 0, &mut ctx, &w, &players, &mut rng, 0, &mut DustPool::new(), &crate::npc::permissive_dust_ctx(&w));
        // num24 = (int)(44 * 1.25) = 55; the parent centre is (1024, 1024)
        // (the port's 48-box integer halves), the link sits 55 px short of
        // it along the +X axis from (1224, 1024).
        let (pw, ph) = pool[1].size();
        let pcx = pool[1].x + (pw as i32 / 2) as f32;
        let pcy = pool[1].y + (ph as i32 / 2) as f32;
        let (lw, lh) = pool[0].size();
        let lcx = pool[0].x + lw * 0.5;
        let lcy = pool[0].y + lh * 0.5;
        let dist = ((lcx - pcx) * (lcx - pcx) + (lcy - pcy) * (lcy - pcy)).sqrt();
        assert!((dist - 55.0).abs() < 0.01, "spacing {dist}");
        assert_eq!(pool[0].vx, 0.0);
        assert_eq!(pool[0].vy, 0.0);
        // The link is to the RIGHT of the parent: atan2(0, -55) = π.
        assert!((pool[0].rotation - (std::f32::consts::PI + 1.57)).abs() < 0.001);
    }

    /// `NPC.cs:49958-49978` — the daytime escape adds 1 to vy (2 below the
    /// surface, cap 32) and flags the family despawn below the rock layer.
    #[test]
    fn s1_d_3_s37_the_daytime_escape_sinks_and_despawns() {
        let mut w = air();
        w.day_time = true;
        w.world_surface = 30.0; // px 480
        w.rock_layer = 60.0; // px 960
        let players = [cand(1000.0, 100.0)];
        let mut rng = UnifiedRandom::new(5);
        // Above the surface: one sink tick.
        let mut pool = [head(500.0, 100.0)];
        let mut ctx = DestroyerCtx::head(0);
        let ev = destroyer_step(&mut pool, 0, &mut ctx, &w, &players, &mut rng, 0, &mut DustPool::new(), &crate::npc::permissive_dust_ctx(&w));
        assert!(pool[0].vy >= 1.0);
        assert!(!ev.despawn_family);
        // Below the rock layer: the despawn flag.
        let mut pool2 = [head(500.0, 1000.0)];
        let mut ctx2 = DestroyerCtx::head(0);
        let ev2 = destroyer_step(&mut pool2, 0, &mut ctx2, &w, &players, &mut rng, 0, &mut DustPool::new(), &crate::npc::permissive_dust_ctx(&w));
        assert!(ev2.despawn_family);
        assert!(pool2[0].vy >= 2.0, "the double sink below the surface");
    }

    /// `NPC.cs:49884-49924` — the tile probe counts solid tiles and
    /// liquid > 64, but not liquid <= 64; `localAI[1]` mirrors the result
    /// (`:49931`/`:49956`).
    #[test]
    fn s1_d_3_s37_the_tile_probe_reads_solids_and_deep_liquid() {
        let mut w = air();
        let players = [cand(5000.0, 5000.0)];
        let mut rng = UnifiedRandom::new(9);
        // A solid tile under the head's box (500..548 x, 500..548 y →
        // tiles 31..34): tile (32, 33) sits inside.
        let i = w.index(32, 33);
        w.tiles[i] = Tile { kind: 1, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
        let mut pool = [head(500.0, 500.0)];
        let mut ctx = DestroyerCtx::head(0);
        destroyer_step(&mut pool, 0, &mut ctx, &w, &players, &mut rng, 0, &mut DustPool::new(), &crate::npc::permissive_dust_ctx(&w));
        assert_eq!(pool[0].local_ai[1], 0.0, "in tiles (:49956)");
        // Shallow liquid (64) does not count; deep (65) does.
        let mut w2 = air();
        let j = w2.index(32, 33);
        w2.tiles[j].liquid = 64;
        let mut pool2 = [head(500.0, 500.0)];
        let mut ctx2 = DestroyerCtx::head(0);
        destroyer_step(&mut pool2, 0, &mut ctx2, &w2, &players, &mut rng, 0, &mut DustPool::new(), &crate::npc::permissive_dust_ctx(&w));
        assert_eq!(pool2[0].local_ai[1], 1.0, "liquid 64 is not > 64 (:49912)");
        w2.tiles[j].liquid = 65;
        let mut pool3 = [head(500.0, 500.0)];
        let mut ctx3 = DestroyerCtx::head(0);
        destroyer_step(&mut pool3, 0, &mut ctx3, &w2, &players, &mut rng, 0, &mut DustPool::new(), &crate::npc::permissive_dust_ctx(&w));
        assert_eq!(pool3[0].local_ai[1], 0.0);
    }

    /// `NPC.cs:50237` + `Utils.cs:1675-1684` — RotatedBy spins the point
    /// about the centre: +90° maps (0, 100) below the origin to (-100, 0).
    #[test]
    fn s1_d_3_s37_rotated_by_spins_about_the_centre() {
        let (x, y) = rotated_by(0.0, 100.0, std::f64::consts::FRAC_PI_2, 0.0, 0.0);
        assert!((x + 100.0).abs() < 0.001, "x {x}");
        assert!(y.abs() < 0.001, "y {y}");
        // The Mech Queen centre offset (`NPC.cs:50956-50959`): type 127
        // rides 14 px up.
        let q = head(100.0, 100.0);
        let (cx, cy) = mech_queen_center(&q, 127);
        let (bx, by) = q.center();
        assert_eq!((cx, cy), (bx, by - 14.0));
        assert_eq!(mech_queen_center(&q, 134), (bx, by));
    }

    /// `NPC.cs:49833-49855` — the head's first tick (ai[0] == 0) stamps
    /// ai[3] = whoAmI and requests the 80-link chain; a later tick
    /// (ai[0] != 0) does not re-spawn.
    #[test]
    fn s1_d_3_s37_the_head_spawns_its_chain_once() {
        let w = air();
        let players = [cand(1000.0, 1000.0)];
        let mut rng = UnifiedRandom::new(11);
        let mut pool = [head(500.0, 500.0)];
        let mut ctx = DestroyerCtx::head(3);
        let ev = destroyer_step(&mut pool, 0, &mut ctx, &w, &players, &mut rng, 0, &mut DustPool::new(), &crate::npc::permissive_dust_ctx(&w));
        assert_eq!(ev.spawn_segments, Some(80));
        assert_eq!(pool[0].ai[3], 3.0);
        assert_eq!(ev.real_life, Some(3));
        pool[0].ai[0] = 4.0; // the first child's slot, as the caller writes
        let ev2 = destroyer_step(&mut pool, 0, &mut ctx, &w, &players, &mut rng, 0, &mut DustPool::new(), &crate::npc::permissive_dust_ctx(&w));
        assert_eq!(ev2.spawn_segments, None);
    }

    // ---- S1-D-3-wire-destroyer: the wiring witnesses (WRITTEN NOT RUN,
    // code-only mode — no compiler invoked on this branch). ----

    /// `NPC.cs:89266-89285` — the probe gate draws Next(25) ONLY while
    /// `ai[2] == 0` (the `&&` short-circuit); a launched body never draws
    /// again. Witness: the stream state equals a clone advanced by exactly
    /// the gated rolls.
    #[test]
    fn s1_d_3_wire_the_probe_gate_draws_only_before_launch() {
        let mut body = head(500.0, 500.0);
        body.kind = NpcKind::TheDestroyerBody;
        let mut rng = UnifiedRandom::new(7);
        let mut witness = rng.clone();
        // Roll until the launch fires (the 1/25 gate); every roll is one
        // Next(25) on both streams.
        let mut anchor = None;
        for _ in 0..1000 {
            let d = witness.next_max(25);
            anchor = probe_launch_on_hit(&mut body, &mut rng, false);
            assert_eq!(rng, witness, "one Next(25) per gated roll");
            if anchor.is_some() {
                assert_eq!(d, 0, "the launch fires on the 0 roll");
                break;
            }
        }
        let (ax, ay) = anchor.expect("a 0 rolls within 1000 tries");
        assert_eq!(body.ai[2], 1.0);
        assert!(body.net_update);
        // The bottom-centre anchor (`:89278`, the integer casts).
        assert_eq!(ax, (500.0 + 24.0) as i32 as f32);
        assert_eq!(ay, (500.0 + 48.0) as i32 as f32);
        // After the launch: NO draws (the ai[2] gate short-circuits).
        let before = rng.clone();
        assert!(probe_launch_on_hit(&mut body, &mut rng, false).is_none());
        assert_eq!(rng, before, "the launched body never draws");
        // The head and the tail never roll (`:89268`'s type gate).
        let mut h = head(500.0, 500.0);
        let before = rng.clone();
        assert!(probe_launch_on_hit(&mut h, &mut rng, false).is_none());
        assert_eq!(rng, before);
    }

    /// `NPC.cs:50593-50616` — the probe's cadence: localAI[0]++ per tick,
    /// justHit resets, the fire at >= 120 needs the sight line; the shot
    /// is type 84 / damage 25 at the SNAPPED centre with the snapped aim
    /// velocity. The machine draws NOTHING (witness: no rng parameter).
    #[test]
    fn s1_d_3_wire_the_probe_fires_at_120_ticks_on_the_snapped_aim() {
        let w = air();
        let players = [cand(1000.0, 500.0)];
        let mut n = Npc::new(NpcKind::Probe, 500.0, 500.0, 1, 0.0);
        n.hp = n.max_hp;
        // 119 quiet ticks (the counter climbs; the probe drifts toward the
        // player — the sight line stays clear, no fire before 120).
        for _ in 0..119 {
            assert!(probe_step(&mut n, &w, &players, false, 0).is_none());
        }
        assert_eq!(n.local_ai[0], 119.0);
        // The 120th tick fires at the CURRENT snapped centre/aim.
        let (cx, cy) = (n.x + 15.0, n.y + 15.0);
        let sx = (cx / 8.0).trunc() * 8.0;
        let sy = (cy / 8.0).trunc() * 8.0;
        let shot = probe_step(&mut n, &w, &players, false, 0)
            .expect("the 120th tick fires");
        assert_eq!(n.local_ai[0], 0.0, "the fire resets the counter");
        assert_eq!(shot.proj_type, 84);
        assert_eq!(shot.damage, 25);
        assert_eq!((shot.x, shot.y), (sx, sy), "the snapped centre");
        // The aim: the player centre (1010, 521) snaps to (1008, 520);
        // the difference normalised to speed 6 (the num=6 row).
        let sp = (shot.vx * shot.vx + shot.vy * shot.vy).sqrt();
        assert!((sp - 6.0).abs() < 0.01, "the num=6 aim ({sp})");
        assert!(shot.vx > 0.0);
        // justHit resets the counter mid-climb (`:50599-50602`).
        n.local_ai[0] = 50.0;
        probe_step(&mut n, &w, &players, true, 0);
        assert_eq!(n.local_ai[0], 0.0);
    }

    /// `NPC.cs:50813-50831` — the far-chase boost: past 600 px the aligned
    /// velocity grows 1.05 below 12; the misaligned decays 0.9. (The
    /// wobble's ±0.023 and the approach's ±0.05 run FIRST — the witness
    /// arithmetic carries them.)
    #[test]
    fn s1_d_3_wire_the_probe_boosts_when_far_and_aligned() {
        let w = air();
        // The player 1000 px right: the snapped distance > 600.
        let players = [cand(1500.0, 500.0)];
        let mut n = Npc::new(NpcKind::Probe, 500.0, 500.0, 1, 0.0);
        n.hp = n.max_hp;
        n.vx = 5.0; // aligned (+X toward the aim), below 12
        probe_step(&mut n, &w, &players, false, 0);
        // ai[0] 0→1: the wobble's vx −0.023 (ai[0] ≤ 100), then the
        // approach +0.05 (5.027 < num4 ≈ 6), then ×1.05.
        assert!((n.vx - (5.0 - 0.023 + 0.05) * 1.05).abs() < 0.001, "vx {}", n.vx);
        // Misaligned: the approach +0.05 first (−5 → −4.973), then ×0.9.
        let mut m = Npc::new(NpcKind::Probe, 500.0, 500.0, 1, 0.0);
        m.hp = m.max_hp;
        m.vx = -5.0;
        probe_step(&mut m, &w, &players, false, 0);
        assert!((m.vx - (-5.0 - 0.023 + 0.05) * 0.9).abs() < 0.001, "vx {}", m.vx);
    }

    /// The driver: the chain spawn (82 slots, the body/tail kinds), the
    /// realLife mirror, the once-only gate, and the RNG order — the spawn
    /// draws nothing, the bodies draw 2/tick each in pool order.
    #[test]
    fn s1_d_3_wire_the_driver_spawns_the_chain_and_mirrors_the_pool() {
        let w = air();
        let players = [cand(1000.0, 1000.0)];
        let mut h = head(500.0, 500.0);
        h.slot = 0;
        let mut npcs = vec![h];
        let mut rng = UnifiedRandom::new(5);
        let mut next_id = 1u32;
        let mut dust = DustPool::new();
        let dctx = crate::npc::permissive_dust_ctx(&w);
        // Tick 1: the spawn draws NOTHING (the C# NewNPC has no rand).
        let events = destroyer_family_step(
            &mut npcs, &w, &players, &mut rng, &mut next_id, 0,
            &mut dust, &dctx,
        );
        assert_eq!(npcs.len(), 82, "head + 80 bodies + tail");
        assert_eq!(npcs[81].kind, NpcKind::TheDestroyerTail);
        // W4-f4: the fade-in dust (site 49801) rolls the live pool. Every
        // segment that fades reports fade_dust == 2 and spawns exactly 2
        // type-182 dusts (noGravity/noLight, scale 2). The head always
        // fades; a link fades only while its parent is half-visible. The
        // total active dust is 2× the fading-segment count.
        let fading: usize = events.iter().filter(|(_, ev)| ev.fade_dust == 2).count();
        assert!(fading >= 1, "the head always fades");
        let active: Vec<usize> = (0..6000).filter(|&i| dust.slot(i).active).collect();
        assert_eq!(active.len(), fading * 2, "2 dusts per fading segment");
        for &i in &active {
            assert_eq!(dust.slot(i).dust_type, 182);
            assert!(dust.slot(i).no_gravity && dust.slot(i).no_light);
            // new_dust rolls scale = (1 + Next(-20,21)*0.01) * 2.0 —
            // the 2f argument times the jitter band [0.8, 1.2].
            let s = dust.slot(i).scale;
            assert!((1.6..=2.4).contains(&s), "the 2f scale band ({s})");
            assert_eq!(dust.slot(i).alpha, 100, "the 100 alpha argument");
            assert_eq!(dust.slot(i).color, Color::default(), "default(Color)");
        }
        // The 80 bodies each drew Next(4)+Next(1400,26000) (the cadence);
        // the fade adds pool-internal draws. The stream advanced past the
        // cadence-only baseline (the dust rolls are real).
        let mut cadence_only = UnifiedRandom::new(5);
        for _ in 0..80 {
            let _ = cadence_only.next_max(4);
            let _ = cadence_only.next_range(1400, 26000);
        }
        assert_ne!(rng, cadence_only, "the fade dust advanced the stream beyond the cadence");
        assert_eq!(events.len(), 82);
        // The pool mirror: a strike on a body lands on the head's pool.
        npcs[40].pending_pool_damage = 100;
        let hp_before = npcs[0].hp;
        destroyer_family_step(&mut npcs, &w, &players, &mut rng, &mut next_id, 0, &mut dust, &dctx);
        assert_eq!(npcs[0].hp, hp_before - 100);
        assert_eq!(npcs[40].hp, npcs[0].hp, "the mirror (:81377-81378)");
        // The once-only gate: no re-spawn on the later ticks.
        assert_eq!(npcs.len(), 82);
    }
}
