//! Plantera's Hook machine — the C#'s `else if (this.aiStyle == 52)` inline
//! arm (`NPC.cs:32076-32240`; the family has no `AI_052_*` method). S1-D-3
//! (aiStyle-52), **code-only mode**: the machine is ported here as pure
//! functions over the NPC pool; the sim wiring is deferred — the family's
//! only kind, `NPCID.PlanterasHook` (263, the single `this.aiStyle = 52;`
//! row in `SetDefaults`, `NPC.cs:12402`), has no `NpcKind` yet, and the
//! port's Plantera v1 (`npc.rs::update_plantera`) spawns no hooks. The
//! machine activates unchanged when the kind lands. See docs/ANALYSIS.md
//! "Plantera's Hook machine (aiStyle-52)".
//!
//! The C# family is ONE type — 263 (`SetDefaults`, `NPC.cs:12396-12409`:
//! 40×40, noGravity/noTileCollide, dmg 60, def 24, lifeMax 4000,
//! `dontTakeDamage = true`). The machine is type-parameterised anyway
//! (`HookCtx::npc_type` + the caller's `type_of` kind→id closure, the
//! Destroyer precedent): the anchor-search's same-type scan (`NPC.cs:32129`)
//! compares against it.
//!
//! Determinism — every `Main.rand` draw of the arm, in source order, all on
//! the caller's [`UnifiedRandom`] (the exact `Main.rand` stream; NPC AI runs
//! under the `UpdateNpcs` swap):
//!   1. `Next(60, 300)` (`:32131`) — once PER matching pool slot inside the
//!      `!flag59 && localAI[0] <= 0 && ai[0] != 0` gate (`:32125`): every
//!      other active same-type NPC with non-zero velocity re-arms the timer
//!      (the last match's draw wins). Zero draws when the gate fails.
//!   2. `Next(300, 600)` (`:32137`) — on entering the anchor search
//!      (`localAI[0] <= 0` after step 1).
//!   3. Per attempt (the `while (!flag60 && num778 <= 1000)` loop, `:32140`,
//!      1..=1001 attempts): `Next(-num781, num781 + 1)` twice (`:32156`,
//!      `:32157` — the tile jitter, ALWAYS drawn, even when the wall-target
//!      arm below overwrites the aim), then `Next(6)` (`:32158`) ONLY when
//!      the boss is below half life — the draw rides the left operand of
//!      the `&&`, so at full life it is skipped, below half it is drawn
//!      even when `!= 0` (the gate-fail draw).
//!   4. Nothing else: the speed ladder, the snap/normalise and the rotation
//!      draw no RNG; `TargetClosest` (`:32160`) scans without `Main.rand`.
//! The witness tests replay the stream draw-for-draw against a clone.
//!
//! Pool indices: the C# reads the `NPC.plantBoss` static slot and walks the
//! 200-slot `Main.npc` array (`Main.maxNPCs = 200`, `InitData.cs:9`). The
//! port's pool compacts every frame, so the caller passes the pool slice +
//! this NPC's index; a `plant_boss` slot outside the slice reads as "no
//! boss" (the C# clears the static to −1 when the boss slot goes inactive,
//! `NPC.cs:8393`/`:95175` — the orphan strike, `:32080-32085`).

use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::{tiledata as td, UnifiedRandom, World};

/// `NPCID.PlanterasHook` (`NPCID.cs`) — the family's only type.
pub const PLANTERAS_HOOK_TYPE: i32 = 263;

/// The per-NPC inputs the machine reads that `Npc` does not carry (or that
/// live on `Main`/`NPC` statics in the C#). The wiring passes the port's
/// stock values (single-player, classic, no getGoodWorld).
#[derive(Clone, Debug)]
pub struct HookCtx {
    /// `this.type` — 263 (the only `aiStyle = 52` row, `NPC.cs:12402`).
    pub npc_type: i32,
    /// `base.whoAmI` — this NPC's index in the pool slice (`:32129`'s
    /// self-skip).
    pub who_am_i: i32,
    /// `NPC.plantBoss` (`NPC.cs` static) — the boss's pool index; < 0 (or
    /// an absent/inactive slot) = the orphan strike (`:32080-32085`).
    pub plant_boss: i32,
    /// `Main.netMode` — the anchor search runs when `!= 1` (`:32106`), the
    /// client anchor latch when `== 1` (`:32095`); the port is
    /// single-player → 0.
    pub net_mode: i32,
    /// `Main.expertMode` (`:32198-32205`) — the +1 / +1 speed bumps.
    pub expert_mode: bool,
    /// `Main.getGoodWorld` (`:32220-32233`) — the thorn latch
    /// (`localAI[3]`); no port flag → false.
    pub get_good_world: bool,
    /// `Main.player[Main.npc[NPC.plantBoss].target].ZoneJungle`
    /// (`:32090`) — the boss-target player's jungle zone. The port's
    /// `Player` has no `zone_jungle` field yet (the scene metrics carry the
    /// render-side flag), so the wiring passes it per tick.
    pub boss_target_zone_jungle: bool,
}

impl HookCtx {
    /// The port's stock context for a hook at pool index `who_am_i` with
    /// the boss at `plant_boss` (single-player, classic, no getGoodWorld,
    /// jungle target).
    pub fn stock(who_am_i: i32, plant_boss: i32) -> Self {
        Self {
            npc_type: PLANTERAS_HOOK_TYPE,
            who_am_i,
            plant_boss,
            net_mode: 0,
            expert_mode: false,
            get_good_world: false,
            boss_target_zone_jungle: true,
        }
    }
}

/// The side effects the machine cannot apply itself (the port has no
/// strike / thorn seams inside a pure AI fn). Everything the `Npc` struct
/// carries (velocity, `ai`, `local_ai`, `rotation`, `target`, `net_update`)
/// is written in place, in C# order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HookEvents {
    /// `this.StrikeNPCNoInteraction(9999, 0f, 0)` + `netUpdate = true`
    /// (`NPC.cs:32082-32083`) — the orphaned hook dies through the caller's
    /// strike path (the port's `strike_npc` convention); the machine
    /// returns immediately after, nothing else runs.
    pub orphan_strike: bool,
    /// `WorldGen.SpawnPlanteraThorns(base.Center)` (`NPC.cs:32223`,
    /// `WorldGen.cs:76113`) — the ftw thorn burst when the hook reaches its
    /// anchor while `localAI[3] == 1`. The tile edits ride the caller.
    pub spawn_thorns: bool,
}

/// `WorldGen.InWorld(x, y, fluff = 0)` (`WorldGen.cs:9688-9695`):
/// `x >= 0 && x < maxTilesX && y >= 0 && y < maxTilesY`.
fn in_world(world: &World, x: i32, y: i32) -> bool {
    x >= 0 && x < world.width && y >= 0 && y < world.height
}

/// `WorldGen.SolidTile(i, j)` (`WorldGen.cs:70111-70132`): an active,
/// solid, non-solidTop, non-halfBrick, slopeless, non-actuated tile. The
/// C# `try/catch` turns an out-of-array index into `false`; the `null`
/// tile's `true` (`:70115-70118`) is unreachable behind the arm's
/// `InWorld` gate (`:32171`). Same shape as `tr_core::edit::solid_tile`
/// (`edit.rs:501-511`, crate-private — kept local like the precedent's
/// tile probes).
fn solid_tile(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    t.active()
        && td::TILE_SOLID.has(t.kind)
        && !td::TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
        && !t.inactive()
}

/// `Main.player[Main.npc[NPC.plantBoss].target]` — the boss's target slot.
/// The C# indexes the 255-slot array (an absent slot reads as an inactive
/// player at the origin); the port's slice holds the session player(s), an
/// absent slot reads as the C# inactive-slot zeros.
fn boss_target<'a>(
    boss: &Npc,
    players: &'a [TargetCandidate],
) -> Option<&'a TargetCandidate> {
    usize::try_from(boss.target).ok().and_then(|t| players.get(t))
}

/// The aiStyle-52 arm (`NPC.cs:32076-32240`) — one hook's tick. `npcs[me]`
/// is the hook; `ctx.plant_boss` indexes the boss in the same slice.
/// `type_of` resolves a pool NPC's vanilla `NPC.type` (the `:32129`
/// same-type scan reads `Main.npc[num777].type`): the wiring passes the
/// kind→id table; no ported kind resolves to 263 yet, so the sibling
/// re-arm stays dormant until the hook kind lands (the witness tests pass
/// a stand-in mapping). Returns the seam effects; the hook's fields are
/// written in place.
pub fn planteras_hook_step(
    npcs: &mut [Npc],
    me: usize,
    ctx: &HookCtx,
    world: &World,
    players: &[TargetCandidate],
    rng: &mut UnifiedRandom,
    ticker: u64,
    type_of: impl Fn(&Npc) -> i32,
) -> HookEvents {
    let mut ev = HookEvents::default();
    // `:32078-32079` — flag58 (the enrage retreat) / flag59 (target dead).
    let mut flag58 = false;
    let mut flag59 = false;
    // `:32080-32085` — the orphan strike. The port reads an absent/inactive
    // boss slot as the cleared static (−1): `NPC.plantBoss` is reset when
    // the boss slot deactivates (`NPC.cs:8393`/`:95175`).
    let boss_slot = usize::try_from(ctx.plant_boss).ok().filter(|&i| i < npcs.len());
    let Some(bi) = boss_slot.filter(|&i| npcs[i].hp > 0) else {
        ev.orphan_strike = true;
        npcs[me].net_update = true;
        return ev;
    };
    // `:32086-32094` — the boss-target player's dead / zone / depth gates.
    // The absent-slot read is the C# inactive-slot zeros (dead false,
    // ZoneJungle false, position (0,0) — above any positive worldSurface).
    let (p_dead, p_x, p_y, p_w, p_h, p_zone_jungle) = match boss_target(&npcs[bi], players) {
        Some(p) => (p.dead, p.x, p.y, p.width, p.height, ctx.boss_target_zone_jungle),
        None => (false, 0.0f32, 0.0f32, 20i32, 42i32, false),
    };
    if p_dead {
        flag59 = true;
    }
    // `:32090` — the C#'s `(double)position.Y < Main.worldSurface * 16.0`
    // is a double comparison; the underworld bound is the f32 cast of
    // `Main.UnderworldLayer * 16` (int product) with a strict `>`.
    // `Main.UnderworldLayer = maxTilesY - 200` (`Main.cs:2790`).
    let underworld_layer = world.height - 200;
    if (ctx.plant_boss != -1 && !p_zone_jungle)
        || (p_y as f64) < world.world_surface * 16.0
        || p_y > (underworld_layer * 16) as f32
        || flag59
    {
        npcs[me].local_ai[0] -= 4.0;
        flag58 = true;
    }
    // `:32095-32105` — the client's anchor latch: BOTH reads are
    // `base.Center.X / 16f` in the decompile (ai[1] latches X too — the
    // vanilla quirk, kept 1:1, NOT a Y transcription).
    if ctx.net_mode == 1 {
        let (cx, _) = npcs[me].center();
        if npcs[me].ai[0] == 0.0 {
            npcs[me].ai[0] = (cx / 16.0) as i32 as f32;
        }
        if npcs[me].ai[1] == 0.0 {
            npcs[me].ai[1] = (cx / 16.0) as i32 as f32;
        }
    }
    // `:32106-32184` — the server/single-player anchor search.
    if ctx.net_mode != 1 {
        // `:32108-32112` — an unanchored hook resets the timer, then ticks.
        if npcs[me].ai[0] == 0.0 || npcs[me].ai[1] == 0.0 {
            npcs[me].local_ai[0] = 0.0;
        }
        npcs[me].local_ai[0] -= 1.0;
        // `:32113-32120` — the boss-life accelerations: `life < lifeMax / 2`
        // and `/ 4` are C# INTEGER divisions (30000 → 15000/7500; an odd
        // lifeMax floors), strict `<`.
        if npcs[bi].hp < npcs[bi].max_hp / 2 {
            npcs[me].local_ai[0] -= 2.0;
        }
        if npcs[bi].hp < npcs[bi].max_hp / 4 {
            npcs[me].local_ai[0] -= 2.0;
        }
        // `:32121-32124` — the enrage retreat drains 6 more per tick.
        if flag58 {
            npcs[me].local_ai[0] -= 6.0;
        }
        // `:32125-32134` — a sibling hook already moving re-arms the timer:
        // ONE `Next(60, 300)` PER matching slot (the loop keeps going — the
        // last match's draw wins), self skipped, same type, either velocity
        // component non-zero.
        if !flag59 && npcs[me].local_ai[0] <= 0.0 && npcs[me].ai[0] != 0.0 {
            // The draw lands after the scan (the borrow checker forbids the
            // in-loop write); EVERY matching slot still spends its own
            // Next(60, 300) and the LAST match's value wins - the C#'s
            // stream order and final state are unchanged.
            let mut rearm: Option<f32> = None;
            for (j, other) in npcs.iter().enumerate() {
                if j as i32 != ctx.who_am_i
                    && other.hp > 0
                    && type_of(other) == ctx.npc_type
                    && (other.vx != 0.0 || other.vy != 0.0)
                {
                    rearm = Some(rng.next_range(60, 300) as f32);
                }
            }
            if let Some(v) = rearm {
                npcs[me].local_ai[0] = v;
            }
        }
        // `:32135-32183` — the anchor search.
        if npcs[me].local_ai[0] <= 0.0 {
            npcs[me].local_ai[0] = rng.next_range(300, 600) as f32;
            let mut flag60 = false;
            let mut num778 = 0;
            // `:32140` — `num778 <= 1000` admits attempt 1001.
            while !flag60 && num778 <= 1000 {
                num778 += 1;
                // `:32143-32148` — the player centre; an unanchored hook
                // (`ai[0] == 0`) aims at the player+boss midpoint.
                let p_cx = p_x + p_w as f32 / 2.0;
                let p_cy = p_y + p_h as f32 / 2.0;
                let mut num779 = (p_cx / 16.0) as i32;
                let mut num780 = (p_cy / 16.0) as i32;
                if npcs[me].ai[0] == 0.0 {
                    let (bcx, bcy) = npcs[bi].center();
                    num779 = ((p_cx + bcx) / 32.0) as i32;
                    num780 = ((p_cy + bcy) / 32.0) as i32;
                }
                // `:32150-32154` — the dead-target retreat anchors below
                // the boss. NOTE the decompile's cast placement:
                // `(int)position.X / 16` (cast BEFORE the int divide) but
                // `(int)(position.Y + 400f) / 16` (cast AFTER the add).
                if flag59 {
                    num779 = npcs[bi].x as i32 / 16;
                    num780 = (npcs[bi].y + 400.0) as i32 / 16;
                }
                // `:32155` — the search radius grows 20 → 120 tiles
                // (`(int)(100f * (num778 / 1000f))` — the f32 quotient).
                let num781 = 20 + (100.0 * (num778 as f32 / 1000.0)) as i32;
                // `:32156-32157` — the jitter draws, ALWAYS taken (even
                // when the wall-target arm below overwrites the aim).
                let mut num782 = num779 + rng.next_range(-num781, num781 + 1);
                let mut num783 = num780 + rng.next_range(-num781, num781 + 1);
                // `:32158-32168` — below half life, `Next(6)` is drawn
                // EVERY attempt (the left `&&` operand); on a 0 the hook
                // re-targets and a walled player centre overrides the aim.
                if npcs[bi].hp < npcs[bi].max_hp / 2 && rng.next_max(6) == 0 {
                    target_closest(&mut npcs[me], world, players, true, 52, ticker);
                    let t = usize::try_from(npcs[me].target)
                        .ok()
                        .and_then(|t| players.get(t));
                    if let Some(tp) = t {
                        let num784 = ((tp.x + tp.width as f32 / 2.0) / 16.0) as i32;
                        let num785 = ((tp.y + tp.height as f32 / 2.0) / 16.0) as i32;
                        if world
                            .tile(num784, num785)
                            .is_some_and(|t| t.wall > 0)
                        {
                            num782 = num784;
                            num783 = num785;
                        }
                    }
                }
                // `:32169-32181` — the anchor test inside the try/catch:
                // a solid tile, or a wall once the search is past attempt
                // 500 OR the boss is below half life.
                if in_world(world, num782, num783)
                    && (solid_tile(world, num782, num783)
                        || (world
                            .tile(num782, num783)
                            .is_some_and(|t| t.wall > 0)
                            && (num778 > 500
                                || npcs[bi].hp < npcs[bi].max_hp / 2)))
                {
                    flag60 = true;
                    npcs[me].ai[0] = num782 as f32;
                    npcs[me].ai[1] = num783 as f32;
                    npcs[me].net_update = true;
                }
            }
        }
    }
    // `:32185-32188` — no positive anchor, no movement (the early return).
    if !(npcs[me].ai[0] > 0.0) || !(npcs[me].ai[1] > 0.0) {
        return ev;
    }
    // `:32189-32213` — the speed ladder: 6 / 8 / 10 by boss life (the same
    // int `/ 2`, `/ 4` thresholds), +1 expert, +1 more expert-below-half,
    // ×2 enrage retreat, ×2 dead target (both multiply, ×4 together).
    let mut num786 = 6.0f32;
    if npcs[bi].hp < npcs[bi].max_hp / 2 {
        num786 = 8.0;
    }
    if npcs[bi].hp < npcs[bi].max_hp / 4 {
        num786 = 10.0;
    }
    if ctx.expert_mode {
        num786 += 1.0;
    }
    if ctx.expert_mode && npcs[bi].hp < npcs[bi].max_hp / 2 {
        num786 += 1.0;
    }
    if flag58 {
        num786 *= 2.0;
    }
    if flag59 {
        num786 *= 2.0;
    }
    // `:32214-32237` — the anchor pull: the anchor tile's centre
    // (`ai * 16 - 8`) minus the hook's centre; inside `12 + speed` the hook
    // SNAPS (velocity = the raw delta, no normalise), else it flies at
    // exactly `speed` (`delta * (speed / dist)`).
    let (cx, cy) = npcs[me].center();
    let num787 = npcs[me].ai[0] * 16.0 - 8.0 - cx;
    let num788 = npcs[me].ai[1] * 16.0 - 8.0 - cy;
    let mut num789 = (num787 * num787 + num788 * num788).sqrt();
    if num789 < 12.0 + num786 {
        // `:32220-32224` — the ftw thorn burst on arrival.
        if ctx.net_mode != 1 && ctx.get_good_world && npcs[me].local_ai[3] == 1.0 {
            npcs[me].local_ai[3] = 0.0;
            ev.spawn_thorns = true;
        }
        npcs[me].vx = num787;
        npcs[me].vy = num788;
    } else {
        // `:32230-32233` — the ftw thorn latch while still travelling.
        if ctx.net_mode != 1 && ctx.get_good_world {
            npcs[me].local_ai[3] = 1.0;
        }
        num789 = num786 / num789;
        npcs[me].vx = num787 * num789;
        npcs[me].vy = num788 * num789;
    }
    // `:32238-32239` - the rotation faces the BOSS (not the anchor):
    // `Atan2(x: bossCentre.X - centre.X, y: bossCentre.Y - centre.Y) -
    // 1.57f` - Math.Atan2(y, x) with NAMED args means y = dy, x = dx,
    // i.e. atan2(dy, dx) (the plantera-audit G1 precedent); the double
    // seam widens, atan2 in f64, casts back; 1.57f is the literal.
    let (bcx, bcy) = npcs[bi].center();
    npcs[me].rotation = ((bcy - cy) as f64).atan2((bcx - cx) as f64) as f32 - 1.57;
    ev
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

    fn air() -> World {
        // height 400 keeps UnderworldLayer = height - 200 = 200 positive
        // (the 120-tall draft made it -80 -> bound -1280, so EVERY player
        // tripped the underworld retreat gate).
        let mut w = World::new("t".into(), 200, 400);
        w.world_surface = 30.0;
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

    /// The boss stand-in: the port's Plantera kind (86×86 box, hp/max_hp
    /// pinned to the C# 30000 row — the machine reads only life/lifeMax,
    /// centre, position and target).
    fn boss(x: f32, y: f32) -> Npc {
        let mut n = Npc::new(NpcKind::Plantera, x, y, 1, 0.0);
        n.hp = 30000;
        n.max_hp = 30000;
        n.target = 0;
        n
    }

    /// The hook stand-in: the kind is unported, so the pool slot rides a
    /// benign kind — the machine reads only the fields it writes (the
    /// same-type scan rides `ctx.npc_type`).
    fn hook(x: f32, y: f32) -> Npc {
        Npc::new(NpcKind::BurningSphere, x, y, 1, 0.0)
    }

    /// The test wiring's kind→type mapping: the BurningSphere stand-in
    /// resolves to 263 (the hook), the Plantera boss to 262 — the real
    /// wiring passes the port's kind→id table when the hook kind lands.
    fn step(
        npcs: &mut [Npc],
        me: usize,
        ctx: &HookCtx,
        w: &World,
        players: &[TargetCandidate],
        rng: &mut UnifiedRandom,
    ) -> HookEvents {
        planteras_hook_step(npcs, me, ctx, w, players, rng, 0, |n| match n.kind {
            NpcKind::BurningSphere => 263,
            NpcKind::Plantera => 262,
            _ => 0,
        })
    }

    /// A world with one solid dirt tile at (x, y).
    fn solid_at(w: &mut World, x: i32, y: i32) {
        let i = w.index(x, y);
        w.tiles[i] = Tile { kind: 1, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
    }

    /// `NPC.cs:32080-32085` — no boss: the orphan strike intent, the net
    /// flag, and NOTHING else (no timer tick, no RNG, no movement).
    #[test]
    fn s1_d_3_s52_the_orphan_strike_draws_nothing_and_returns() {
        let w = air();
        let players = [cand(1000.0, 800.0)];
        let mut pool = [hook(500.0, 500.0)];
        let mut ctx = HookCtx::stock(0, -1);
        ctx.plant_boss = -1;
        let mut rng = UnifiedRandom::new(11);
        let witness = rng.clone();
        let ev = step(&mut pool, 0, &ctx, &w, &players, &mut rng);
        assert!(ev.orphan_strike);
        assert!(pool[0].net_update);
        assert_eq!(rng, witness, "the orphan path consumes zero draws");
        assert_eq!(pool[0].local_ai[0], 0.0, "no timer tick before the return");
        assert_eq!((pool[0].vx, pool[0].vy), (0.0, 0.0));
    }

    /// `NPC.cs:32108-32120` — the timer ladder: unanchored resets to 0 then
    /// −1; below half −3; below quarter −5; the retreat gate adds −4 up
    /// front and −6 inside (−11 total). Boss lifeMax 30000 → the int
    /// thresholds 15000/7500 are STRICT `<`.
    #[test]
    fn s1_d_3_s52_the_timer_ladder_follows_the_boss_life_and_retreat() {
        let w = air();
        let players = [cand(1000.0, 800.0)]; // jungle, below surface 480
        let mut rng = UnifiedRandom::new(1);
        // Full life, anchored, timer 100 -> 99. The player must sit BELOW
        // the surface line (world_surface 30 * 16 = 480): y 800 is under it.
        let mut pool = [boss(900.0, 700.0), hook(500.0, 500.0)];
        pool[1].ai = [10.0, 10.0, 0.0, 0.0];
        pool[1].local_ai[0] = 100.0;
        let ctx = HookCtx::stock(1, 0);
        let deep = [cand(1000.0, 800.0)];
        step(&mut pool, 1, &ctx, &w, &deep, &mut rng);
        assert_eq!(pool[1].local_ai[0], 99.0);
        // Below half (14999 < 15000): −1 −2 = −3.
        pool[0].hp = 14999;
        pool[1].local_ai[0] = 100.0;
        step(&mut pool, 1, &ctx, &w, &players, &mut rng);
        assert_eq!(pool[1].local_ai[0], 97.0);
        // Exactly half (15000) is NOT below half (strict <).
        pool[0].hp = 15000;
        pool[1].local_ai[0] = 100.0;
        step(&mut pool, 1, &ctx, &w, &players, &mut rng);
        assert_eq!(pool[1].local_ai[0], 99.0);
        // Below quarter (7499 < 7500): −1 −2 −2 = −5.
        pool[0].hp = 7499;
        pool[1].local_ai[0] = 100.0;
        step(&mut pool, 1, &ctx, &w, &players, &mut rng);
        assert_eq!(pool[1].local_ai[0], 95.0);
        // The retreat (player above the surface): −4 −1 −6 = −11 at full
        // life (`:32092`, `:32112`, `:32123`).
        pool[0].hp = 30000;
        let high = [cand(1000.0, 100.0)];
        pool[1].local_ai[0] = 100.0;
        step(&mut pool, 1, &ctx, &w, &high, &mut rng);
        assert_eq!(pool[1].local_ai[0], 89.0);
    }

    /// `NPC.cs:32125-32134` — the sibling re-arm draws ONE `Next(60, 300)`
    /// PER moving same-type slot (the last draw wins), and NOTHING when no
    /// sibling moves or the gate fails (witness: stream-clone equality).
    #[test]
    fn s1_d_3_s52_the_sibling_rearm_draws_once_per_moving_hook() {
        let w = air();
        let players = [cand(1000.0, 800.0)];
        // Two moving siblings → two draws; the timer holds the second.
        let mut pool = [boss(900.0, 700.0), hook(500.0, 500.0), hook(600.0, 500.0), hook(700.0, 500.0)];
        pool[1].ai = [10.0, 10.0, 0.0, 0.0];
        pool[1].local_ai[0] = 1.0; // ticks to 0 → the re-arm gate opens
        pool[2].vx = 1.0;
        pool[3].vy = -1.0;
        let ctx = HookCtx::stock(1, 0);
        // Pick a seed whose first two Next(60,300) draws differ (the
        // last-draw-wins pin needs them distinct).
        let mut seed = 21;
        for s in 0..1000 {
            let mut probe = UnifiedRandom::new(s);
            if probe.next_range(60, 300) != probe.next_range(60, 300) {
                seed = s;
                break;
            }
        }
        let mut rng = UnifiedRandom::new(seed);
        let mut witness = rng.clone();
        let d1 = witness.next_range(60, 300);
        let d2 = witness.next_range(60, 300);
        let ev = step(&mut pool, 1, &ctx, &w, &players, &mut rng);
        assert_eq!(rng, witness, "exactly two Next(60,300) draws, in slot order");
        assert_eq!(pool[1].local_ai[0], d2 as f32, "the LAST match's draw wins");
        assert_ne!(d1, d2);
        assert!(!ev.orphan_strike);
        // A stationary sibling draws nothing (the velocity gate): the
        // re-arm is skipped and the step falls straight into the search —
        // the FIRST draw is Next(300,600), never Next(60,300). Pin it by
        // making attempt 1 anchor (a solid player-centre tile + a seed
        // whose first jitter is (0,0)): the stream then holds exactly the
        // three search draws.
        let mut w2 = air();
        solid_at(&mut w2, 63, 51);
        let mut found = None;
        for seed in 0..1000 {
            let mut probe = UnifiedRandom::new(seed);
            let _ = probe.next_range(300, 600);
            if probe.next_range(-20, 21) == 0 && probe.next_range(-20, 21) == 0 {
                found = Some(seed);
                break;
            }
        }
        let seed = found.expect("a (0,0) first-jitter seed");
        let mut pool2 = [boss(900.0, 700.0), hook(500.0, 500.0), hook(600.0, 500.0)];
        pool2[1].ai = [10.0, 10.0, 0.0, 0.0];
        pool2[1].local_ai[0] = 1.0;
        let mut rng2 = UnifiedRandom::new(seed);
        let mut witness2 = rng2.clone();
        let d_search = witness2.next_range(300, 600);
        let _ = witness2.next_range(-20, 21);
        let _ = witness2.next_range(-20, 21);
        step(&mut pool2, 1, &ctx, &w2, &players, &mut rng2);
        assert_eq!(rng2, witness2, "no re-arm draw — straight to the search");
        assert_eq!(pool2[1].local_ai[0], d_search as f32);
    }

    /// `NPC.cs:32135-32183` — a full-life search attempt consumes exactly
    /// `Next(300, 600)` + two `Next(-20, 21)` jitters (radius 20 on attempt
    /// 1, `20 + (int)(100f * 1/1000f)`); NO `Next(6)` at full life. The
    /// anchor is the player centre tile when it is solid.
    #[test]
    fn s1_d_3_s52_the_full_life_search_draws_three_and_skips_next6() {
        let mut w = air();
        // Player centre (1010, 821) → tile (63, 51); make it solid.
        solid_at(&mut w, 63, 51);
        let players = [cand(1000.0, 800.0)];
        let mut pool = [boss(900.0, 700.0), hook(500.0, 500.0)];
        pool[1].ai = [10.0, 10.0, 0.0, 0.0];
        pool[1].local_ai[0] = 1.0;
        let ctx = HookCtx::stock(1, 0);
        // Find a seed whose first attempt's jitter lands on (63, 51):
        // scan seeds offline-style inside the test (deterministic).
        let mut found = None;
        for seed in 0..1000 {
            let mut probe = UnifiedRandom::new(seed);
            let _ = probe.next_range(300, 600);
            let jx = probe.next_range(-20, 21);
            let jy = probe.next_range(-20, 21);
            if 63 + jx == 63 && 51 + jy == 51 {
                found = Some(seed);
                break;
            }
        }
        let seed = found.expect("a seed with a (0,0) first jitter exists");
        let mut rng = UnifiedRandom::new(seed);
        let mut witness = rng.clone();
        let d_timer = witness.next_range(300, 600);
        let jx = witness.next_range(-20, 21);
        let jy = witness.next_range(-20, 21);
        assert_eq!((jx, jy), (0, 0));
        step(&mut pool, 1, &ctx, &w, &players, &mut rng);
        assert_eq!(rng, witness, "Next(300,600) + two jitters, NO Next(6) at full life");
        assert_eq!(pool[1].local_ai[0], d_timer as f32);
        assert_eq!(pool[1].ai[0], 63.0, "the anchor is the solid player-centre tile");
        assert_eq!(pool[1].ai[1], 51.0);
        assert!(pool[1].net_update);
    }

    /// `NPC.cs:32158` — below half life the `Next(6)` rides EVERY attempt
    /// (the left `&&` operand), even when it fails: a successful first
    /// attempt consumes timer + 2 jitters + Next(6), in that order.
    #[test]
    fn s1_d_3_s52_below_half_life_every_attempt_draws_next6() {
        let mut w = air();
        solid_at(&mut w, 63, 51);
        let players = [cand(1000.0, 800.0)];
        let mut pool = [boss(900.0, 700.0), hook(500.0, 500.0)];
        pool[0].hp = 14999; // below the int 30000/2 threshold
        pool[1].ai = [10.0, 10.0, 0.0, 0.0];
        pool[1].local_ai[0] = 1.0;
        let ctx = HookCtx::stock(1, 0);
        let mut found = None;
        for seed in 0..1000 {
            let mut probe = UnifiedRandom::new(seed);
            let _ = probe.next_range(300, 600);
            let jx = probe.next_range(-20, 21);
            let jy = probe.next_range(-20, 21);
            let d6 = probe.next_max(6);
            if jx == 0 && jy == 0 && d6 != 0 {
                found = Some(seed);
                break;
            }
        }
        let seed = found.expect("a seed with a (0,0) jitter and a failing Next(6)");
        let mut rng = UnifiedRandom::new(seed);
        let mut witness = rng.clone();
        let _ = witness.next_range(300, 600);
        let _ = witness.next_range(-20, 21);
        let _ = witness.next_range(-20, 21);
        let d6 = witness.next_max(6);
        assert_ne!(d6, 0, "the gate FAILS but the draw is still consumed");
        step(&mut pool, 1, &ctx, &w, &players, &mut rng);
        assert_eq!(rng, witness, "the failing Next(6) is drawn (gate-fail draw)");
        assert_eq!((pool[1].ai[0], pool[1].ai[1]), (63.0, 51.0));
    }

    /// `NPC.cs:32189-32213` — the speed ladder: 6 base, 8 below half, 10
    /// below quarter, +1 expert, +1 expert-below-half, ×2 retreat, ×2 dead
    /// (×4 together). Read through the far-flight normalise (speed =
    /// |velocity|).
    #[test]
    fn s1_d_3_s52_the_speed_ladder_matches_the_csharp() {
        let mut w = air();
        let players = [cand(1000.0, 800.0)];
        let mut rng = UnifiedRandom::new(5);
        let mut speed_of = |pool: &mut [Npc], ctx: &HookCtx, players: &[TargetCandidate]| {
            step(pool, 1, ctx, &w, players, &mut rng);
            (pool[1].vx * pool[1].vx + pool[1].vy * pool[1].vy).sqrt()
        };
        let setup = |hp: i32| {
            let mut pool = [boss(900.0, 700.0), hook(500.0, 500.0)];
            pool[0].hp = hp;
            pool[1].ai = [60.0, 55.0, 0.0, 0.0]; // far anchor, timer high
            pool[1].local_ai[0] = 500.0;
            pool
        };
        let ctx = HookCtx::stock(1, 0);
        let mut expert = ctx.clone();
        expert.expert_mode = true;
        assert!((speed_of(&mut setup(30000), &ctx, &players) - 6.0).abs() < 1e-4);
        assert!((speed_of(&mut setup(14999), &ctx, &players) - 8.0).abs() < 1e-4);
        assert!((speed_of(&mut setup(7499), &ctx, &players) - 10.0).abs() < 1e-4);
        assert!((speed_of(&mut setup(30000), &expert, &players) - 7.0).abs() < 1e-4);
        assert!((speed_of(&mut setup(14999), &expert, &players) - 10.0).abs() < 1e-4);
        // Retreat (player above the surface): ×2 → 12 at full life.
        let high = [cand(1000.0, 100.0)];
        assert!((speed_of(&mut setup(30000), &ctx, &high) - 12.0).abs() < 1e-4);
        // Dead target: ×2 → 12; dead + retreat → ×4 = 24.
        let mut dead = [cand(1000.0, 800.0)];
        dead[0].dead = true;
        assert!((speed_of(&mut setup(30000), &ctx, &dead) - 12.0).abs() < 1e-4);
        let mut dead_high = [cand(1000.0, 100.0)];
        dead_high[0].dead = true;
        assert!((speed_of(&mut setup(30000), &ctx, &dead_high) - 24.0).abs() < 1e-4);
    }

    /// `NPC.cs:32218-32237` — inside `12 + speed` the hook SNAPS (velocity
    /// = the raw anchor-centre delta, NOT normalised); outside it flies at
    /// exactly `speed` along the delta. The anchor centre is `ai*16 - 8`.
    #[test]
    fn s1_d_3_s52_the_snap_band_is_strict_and_the_flight_is_normalised() {
        let w = air();
        let players = [cand(1000.0, 800.0)];
        let mut rng = UnifiedRandom::new(9);
        let ctx = HookCtx::stock(1, 0);
        // Hook centre (508, 508) (the 16-box stand-in); anchor (33, 32)
        // centre (520, 504) — delta (12, -4), dist ~12.65 < 12 + 6 = 18 →
        // the snap keeps the RAW delta.
        let mut pool = [boss(900.0, 700.0), hook(500.0, 500.0)];
        pool[1].ai = [33.0, 32.0, 0.0, 0.0];
        pool[1].local_ai[0] = 500.0;
        step(&mut pool, 1, &ctx, &w, &players, &mut rng);
        let (cx, cy) = pool[1].center();
        assert_eq!(pool[1].vx, 33.0 * 16.0 - 8.0 - cx);
        assert_eq!(pool[1].vy, 32.0 * 16.0 - 8.0 - cy);
        assert_eq!(pool[1].vx, 12.0, "the snap keeps the raw delta");
        assert_eq!(pool[1].vy, -4.0);
        // dist exactly 18 is NOT < 18 (strict) → the normalised flight at
        // 6: anchor (33, 32) centre (520, 504), hook centre (520, 522).
        let mut pool2 = [boss(900.0, 700.0), hook(500.0, 500.0)];
        pool2[1].ai = [33.0, 32.0, 0.0, 0.0];
        let (_, h) = pool2[1].size();
        pool2[1].y = 522.0 - h * 0.5;
        pool2[1].local_ai[0] = 500.0;
        step(&mut pool2, 1, &ctx, &w, &players, &mut rng);
        let dist = (pool2[1].vx * pool2[1].vx + pool2[1].vy * pool2[1].vy).sqrt();
        assert!((dist - 6.0).abs() < 1e-4, "dist {dist}: exactly 18 normalises to speed 6");
        assert!(pool2[1].vy < 0.0);
    }

    /// `NPC.cs:32150-32154` — the dead-target retreat anchors below the
    /// boss with the decompile's cast placement: X is `(int)position.X / 16`
    /// (cast BEFORE the int divide), Y is `(int)(position.Y + 400f) / 16`
    /// (the +400 INSIDE the cast, then the int divide). The observable
    /// pin: the +400 offset rides Y only, and the divide is integer.
    #[test]
    fn s1_d_3_s52_the_dead_retreat_keeps_the_cast_placement() {
        let mut w = air();
        // Boss position (905.9, 700.7): X = (int)905.9 / 16 = 905/16 = 56;
        // Y = (int)(700.7 + 400) / 16 = 1100/16 = 68 (a missing +400 would
        // give 43; a float divide would give 68.75 → 68 only after the
        // cast — the int divide is pinned by the exact 1100/16 = 68).
        let mut pool = [boss(905.9, 700.7), hook(500.0, 500.0)];
        pool[1].ai = [10.0, 10.0, 0.0, 0.0];
        pool[1].local_ai[0] = 1.0;
        let mut dead = [cand(1000.0, 800.0)];
        dead[0].dead = true;
        // Anchor the retreat tile solid so attempt 1 with a (0,0) jitter
        // succeeds: X = (int)905.9/16 = 56, Y = (int)(700.7+400)/16 = 68.
        solid_at(&mut w, 56, 68);
        let ctx = HookCtx::stock(1, 0);
        let mut found = None;
        for seed in 0..4000 {
            let mut probe = UnifiedRandom::new(seed);
            let _ = probe.next_range(300, 600);
            if probe.next_range(-20, 21) == 0 && probe.next_range(-20, 21) == 0 {
                found = Some(seed);
                break;
            }
        }
        let seed = found.expect("a (0,0) first-jitter seed");
        let mut rng = UnifiedRandom::new(seed);
        step(&mut pool, 1, &ctx, &w, &dead, &mut rng);
        assert_eq!(pool[1].ai[0], 56.0, "(int)position.X / 16 (:32152)");
        assert_eq!(pool[1].ai[1], 68.0, "(int)(position.Y + 400f) / 16 (:32153)");
    }

    /// `NPC.cs:32185-32188` — a non-positive anchor returns before the
    /// speed ladder: no velocity, no rotation write.
    #[test]
    fn s1_d_3_s52_a_zero_anchor_returns_before_the_movement() {
        let w = air();
        let players = [cand(1000.0, 800.0)];
        let mut rng = UnifiedRandom::new(4);
        let ctx = HookCtx::stock(1, 0);
        let mut pool = [boss(900.0, 700.0), hook(500.0, 500.0)];
        pool[1].ai = [0.0, 10.0, 0.0, 0.0];
        pool[1].local_ai[0] = 500.0; // no search this tick
        let ev = step(&mut pool, 1, &ctx, &w, &players, &mut rng);
        assert!(!ev.orphan_strike);
        assert_eq!((pool[1].vx, pool[1].vy), (0.0, 0.0));
        assert_eq!(pool[1].rotation, 0.0, "the rotation write is past the return");
    }

    /// `NPC.cs:32238-32239` — the rotation faces the BOSS centre with the
    /// named-arg Atan2 (dx first) minus the 1.57f literal.
    #[test]
    fn s1_d_3_s52_the_rotation_faces_the_boss_with_the_literal() {
        let w = air();
        let players = [cand(1000.0, 800.0)];
        let mut rng = UnifiedRandom::new(6);
        let ctx = HookCtx::stock(1, 0);
        let mut pool = [boss(900.0, 700.0), hook(500.0, 500.0)];
        pool[1].ai = [60.0, 55.0, 0.0, 0.0];
        pool[1].local_ai[0] = 500.0;
        step(&mut pool, 1, &ctx, &w, &players, &mut rng);
        let (cx, cy) = pool[1].center();
        let (bcx, bcy) = pool[0].center();
        let expect = ((bcy - cy) as f64).atan2((bcx - cx) as f64) as f32 - 1.57;
        assert_eq!(pool[1].rotation, expect);
        assert_ne!(expect, ((bcy - cy) as f64).atan2((bcx - cx) as f64) as f32 - std::f32::consts::FRAC_PI_2,
            "1.57f is the literal, not π/2");
    }

    /// `NPC.cs:32220-32233` — the ftw thorn latch: travelling sets
    /// `localAI[3] = 1`; arriving with the latch fires the thorn intent
    /// once and clears it; without getGoodWorld nothing latches.
    #[test]
    fn s1_d_3_s52_the_ftw_thorn_latch_fires_once_on_arrival() {
        let w = air();
        let players = [cand(1000.0, 800.0)];
        let mut rng = UnifiedRandom::new(8);
        let mut ctx = HookCtx::stock(1, 0);
        ctx.get_good_world = true;
        // Far anchor → travelling: the latch sets, no thorns.
        let mut pool = [boss(900.0, 700.0), hook(500.0, 500.0)];
        pool[1].ai = [60.0, 55.0, 0.0, 0.0];
        pool[1].local_ai[0] = 500.0;
        let ev = step(&mut pool, 1, &ctx, &w, &players, &mut rng);
        assert!(!ev.spawn_thorns);
        assert_eq!(pool[1].local_ai[3], 1.0);
        // Near anchor (the snap band) → the thorn burst, the latch clears.
        pool[1].ai = [33.0, 32.0, 0.0, 0.0];
        let ev2 = step(&mut pool, 1, &ctx, &w, &players, &mut rng);
        assert!(ev2.spawn_thorns);
        assert_eq!(pool[1].local_ai[3], 0.0);
        // A second arrival does not re-fire.
        let ev3 = step(&mut pool, 1, &ctx, &w, &players, &mut rng);
        assert!(!ev3.spawn_thorns);
    }

    /// `NPC.cs:32095-32105` — the client latch copies `Center.X / 16f`
    /// into BOTH ai slots (the decompile's X/X quirk, kept 1:1) and runs
    /// no search (netMode 1 skips `:32106-32184`).
    #[test]
    fn s1_d_3_s52_the_client_latch_copies_x_into_both_slots() {
        let w = air();
        let players = [cand(1000.0, 800.0)];
        let mut rng = UnifiedRandom::new(2);
        let witness = rng.clone();
        let mut ctx = HookCtx::stock(1, 0);
        ctx.net_mode = 1;
        let mut pool = [boss(900.0, 700.0), hook(500.0, 500.0)];
        let (cx, _) = pool[1].center();
        step(&mut pool, 1, &ctx, &w, &players, &mut rng);
        let expect = (cx / 16.0) as i32 as f32;
        assert_eq!(pool[1].ai[0], expect);
        assert_eq!(pool[1].ai[1], expect, "ai[1] latches X too (:32103)");
        assert_eq!(rng, witness, "the client path draws nothing");
    }

    /// `NPC.cs:32171` — the wall anchor: at full life a wall-only tile is
    /// rejected until attempt 501 (`num778 > 500`); below half life it is
    /// accepted from attempt 1. The radius at attempt 1 is 20, so a wall
    /// at the player centre tile is reachable with a (0,0) jitter.
    #[test]
    fn s1_d_3_s52_the_wall_anchor_gates_on_attempt_500_or_half_life() {
        let mut w = air();
        let i = w.index(63, 51);
        w.tiles[i] = Tile { wall: 4, ..Default::default() }; // wall, no tile
        let players = [cand(1000.0, 800.0)];
        let ctx = HookCtx::stock(1, 0);
        // Below half life: the wall anchors on the first (0,0)-jitter attempt.
        let mut found = None;
        for seed in 0..2000 {
            let mut probe = UnifiedRandom::new(seed);
            let _ = probe.next_range(300, 600);
            if probe.next_range(-20, 21) == 0 && probe.next_range(-20, 21) == 0 {
                found = Some(seed);
                break;
            }
        }
        let seed = found.expect("a (0,0) first jitter seed");
        let mut pool = [boss(900.0, 700.0), hook(500.0, 500.0)];
        pool[0].hp = 14999;
        pool[1].ai = [10.0, 10.0, 0.0, 0.0];
        pool[1].local_ai[0] = 1.0;
        let mut rng = UnifiedRandom::new(seed);
        step(&mut pool, 1, &ctx, &w, &players, &mut rng);
        assert_eq!((pool[1].ai[0], pool[1].ai[1]), (63.0, 51.0),
            "below half life the wall anchors immediately");
        // Full life: the same seed's attempt 1 rejects the wall; the search
        // keeps drawing (two jitters per attempt, no Next(6) at full life).
        let mut pool2 = [boss(900.0, 700.0), hook(500.0, 500.0)];
        pool2[1].ai = [10.0, 10.0, 0.0, 0.0];
        pool2[1].local_ai[0] = 1.0;
        let mut rng2 = UnifiedRandom::new(seed);
        let mut witness2 = rng2.clone();
        let _ = witness2.next_range(300, 600);
        let _ = witness2.next_range(-20, 21);
        let _ = witness2.next_range(-20, 21);
        step(&mut pool2, 1, &ctx, &w, &players, &mut rng2);
        assert_ne!(rng2, witness2, "attempt 1 rejected the wall — more draws followed");
    }
}
