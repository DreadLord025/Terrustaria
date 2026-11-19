//! The Destroyer's machine — `NPC.AI_037_Destroyer` (`NPC.cs:49756-50242`,
//! dispatched by `else if (this.aiStyle == 37)` at `NPC.cs:28991-28994`).
//! S1-D-3 (aiStyle-37), **code-only mode**: the machine is ported here as
//! pure functions over the NPC pool; the sim wiring is deferred (the port's
//! Destroyer currently rides the aiStyle-6 worm pass — `npc.rs`'s
//! `NpcKind::TheDestroyer => {}` arm + `world_sim/npcs.rs::update_worms_in`'s
//! 80-budget chain). See docs/ANALYSIS.md "The Destroyer machine (aiStyle-37)".
//!
//! The C# family is three types — the head (134), the body (135) and the tail
//! (136), all aiStyle 37 (`SetDefaults`, `NPC.cs:10659-10719`: 38×38, scale
//! 1.25, noGravity/noTileCollide, alpha 255, lifeMax 80000; the head dmg 70 /
//! def 0 / value 120000 / boss, the body dmg 55 / def 30, the tail dmg 40 /
//! def 35). The port has only `NpcKind::TheDestroyer` (the head), so the
//! machine is **type-parameterised** (`DestroyerCtx::npc_type`): the body/tail
//! branches activate unchanged when their kinds land.
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
use tr_core::tiledata as td;
use tr_core::{UnifiedRandom, World};

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
/// (`NPC.cs:49765`: `type >= 134 && type <= 136`). The port's only family
/// kind is the head; the body/tail kinds join when they land.
fn family_type(n: &Npc) -> Option<i32> {
    match n.kind {
        NpcKind::TheDestroyer => Some(134),
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
                ev.fade_dust = 2;
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
        let ev = destroyer_step(&mut pool, 0, &mut ctx, &w, &players, &mut rng, 0);
        assert_eq!(pool[0].alpha, 213, "255 - 42 (:49806)");
        assert_eq!(ev.fade_dust, 2);
        // The body behind an opaque parent keeps its alpha.
        let mut pool2 = [head(500.0, 500.0), head(500.0, 548.0)];
        pool2[1].ai[1] = 0.0; // parent slot 0, alpha 255
        let mut ctx2 = DestroyerCtx::head(1);
        ctx2.npc_type = 135;
        let ev2 = destroyer_step(&mut pool2, 1, &mut ctx2, &w, &players, &mut rng, 0);
        assert_eq!(pool2[1].alpha, 255, "the parent is not half-visible yet");
        assert_eq!(ev2.fade_dust, 0);
        // Once the parent drops below 128 the body fades too.
        pool2[0].alpha = 127;
        destroyer_step(&mut pool2, 1, &mut ctx2, &w, &players, &mut rng, 0);
        assert_eq!(pool2[1].alpha, 213);
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
        let ev = destroyer_step(&mut pool, 0, &mut ctx, &w, &players, &mut rng, 0);
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
        let ev = destroyer_step(&mut pool, 0, &mut ctx, &w, &players, &mut rng, 0);
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
        let ev = destroyer_step(&mut pool, 1, &mut ctx, &w, &players, &mut rng, 0);
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
        destroyer_step(&mut pool, 0, &mut ctx, &w, &players, &mut rng, 0);
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
        let ev = destroyer_step(&mut pool, 0, &mut ctx, &w, &players, &mut rng, 0);
        assert!(pool[0].vy >= 1.0);
        assert!(!ev.despawn_family);
        // Below the rock layer: the despawn flag.
        let mut pool2 = [head(500.0, 1000.0)];
        let mut ctx2 = DestroyerCtx::head(0);
        let ev2 = destroyer_step(&mut pool2, 0, &mut ctx2, &w, &players, &mut rng, 0);
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
        destroyer_step(&mut pool, 0, &mut ctx, &w, &players, &mut rng, 0);
        assert_eq!(pool[0].local_ai[1], 0.0, "in tiles (:49956)");
        // Shallow liquid (64) does not count; deep (65) does.
        let mut w2 = air();
        let j = w2.index(32, 33);
        w2.tiles[j].liquid = 64;
        let mut pool2 = [head(500.0, 500.0)];
        let mut ctx2 = DestroyerCtx::head(0);
        destroyer_step(&mut pool2, 0, &mut ctx2, &w2, &players, &mut rng, 0);
        assert_eq!(pool2[0].local_ai[1], 1.0, "liquid 64 is not > 64 (:49912)");
        w2.tiles[j].liquid = 65;
        let mut pool3 = [head(500.0, 500.0)];
        let mut ctx3 = DestroyerCtx::head(0);
        destroyer_step(&mut pool3, 0, &mut ctx3, &w2, &players, &mut rng, 0);
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
        let ev = destroyer_step(&mut pool, 0, &mut ctx, &w, &players, &mut rng, 0);
        assert_eq!(ev.spawn_segments, Some(80));
        assert_eq!(pool[0].ai[3], 3.0);
        assert_eq!(ev.real_life, Some(3));
        pool[0].ai[0] = 4.0; // the first child's slot, as the caller writes
        let ev2 = destroyer_step(&mut pool, 0, &mut ctx, &w, &players, &mut rng, 0);
        assert_eq!(ev2.spawn_segments, None);
    }
}
