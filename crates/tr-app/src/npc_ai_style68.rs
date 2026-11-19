//! S1-D-3 — the aiStyle-68 waterfowl family: the `else if (this.aiStyle ==
//! 68)` arm of `NPC.AI` (`NPC.cs:34623-34879`), shared by exactly FOUR
//! vanilla types whose `SetDefaults` rows assign `this.aiStyle = 68` (the
//! census: `Select-String "aiStyle = 68;"` over `NPC.cs` — 4 hits, no
//! dynamic reassignment): Duck2 363 (`NPC.cs:13625-13636`), DuckWhite2 365
//! (`:13649-13660`), Seagull2 603 (`:16751-16762`), Grebe2 609
//! (`:16821-16832`). All four are the swimming "2" forms of the aiStyle-7
//! walkers Duck 362 / DuckWhite 364 / Seagull 602 / Grebe 608 — the landing
//! arm transforms back via `Transform(this.type - 1, 0f, 200 +
//! Main.rand.Next(200))` (`:34760`).
//!
//! **UNWIRED module (code-only mode, 2026-11-19).** None of the four types
//! has a `NpcKind` yet (`npc/data.rs::ai_style` carries no style-68 row), so
//! the arm lands as pure functions over the port's `Npc` slot + a context
//! seam; the wiring (new kinds, the `update_rng` style-axis arm, the
//! `size_override` rows from [`STYLE68_BIRDS`], the 1080 BirdDroppings
//! projectile and the full `NPC.Transform`) is the family's merge step —
//! the `npc_ai_style24.rs` (S1-D-3 aiStyle-24) precedent.
//!
//! The one `TargetClosest()` site (`NPC.cs:34641`) rides the shared
//! selection helper (`crate::npc_target::target_closest`,
//! `NPC.cs:77652-77781`); the `Player.FindClosest` site (`:34627`) and the
//! whole poop gate reuse the style-24 module's ports UNMODIFIED
//! (`find_closest_player`, `roll_only_bad_luck_extreme`, `is_this_cenx`,
//! `bird_that_can_poop` — the four types are in `NPCID.Sets.BirdThatCanPoop`,
//! `NPCID.cs:10435`).
//!
//! RNG: the arm has exactly TWO draw sites, both gated —
//! 1. the poop gate's `RollOnlyBadLuckExtreme(50)` (`:34629`, the
//!    conditional `NextFloat()` + `Next(5)` pair, `Luck.cs:53-60`), and
//! 2. the landing transform's `Main.rand.Next(200)` (`:34760`), drawn ONLY
//!    on a dry landing tick (`ai[1] >= 300` and grounded/wet) of one of the
//!    four types with `netMode != 1`.
//! Every other path (the swim, the flight, the bounces, the probe, the
//! dead-target return, the client) draws NOTHING — pinned by the mirrored
//! `UnifiedRandom` stream witnesses below.
#![allow(dead_code)] // UNWIRED — see the module doc; the wiring is the S1-D-3 merge step.

use crate::npc::Npc;
use crate::npc_ai_style24::{
    bird_that_can_poop, find_closest_player, roll_only_bad_luck_extreme, Style24Player,
};
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::tiledata::{TILE_SOLID, TILE_SOLID_TOP};
use tr_core::{UnifiedRandom, World};

/// One `SetDefaults` row of the family — the numbers the arm (and the
/// wiring's `size_override`/stats) reads.
#[derive(Clone, Copy, Debug)]
pub struct WaterfowlDefaults {
    pub ty: i32,
    pub width: i32,
    pub height: i32,
    pub life_max: i32,
    pub catch_item: i32,
    /// The aiStyle-7 walker this form lands back into (`type - 1`).
    pub walker_ty: i32,
}

/// The 4 style-68 rows. Every row: 28×22, damage 0, defense 0, lifeMax 5,
/// NPCHit1/NPCDeath1, value 0 (the `SetDefaults` default, `NPC.cs:8544`).
/// Citations are the per-type blocks.
pub const STYLE68_BIRDS: &[WaterfowlDefaults] = &[
    // NPC.cs:13625-13636 — Duck2 (catchItem 2122, the Duck's item).
    WaterfowlDefaults { ty: 363, width: 28, height: 22, life_max: 5, catch_item: 2122, walker_ty: 362 },
    // NPC.cs:13649-13660 — DuckWhite2 (catchItem 2123).
    WaterfowlDefaults { ty: 365, width: 28, height: 22, life_max: 5, catch_item: 2123, walker_ty: 364 },
    // NPC.cs:16751-16762 — Seagull2 (catchItem 4359).
    WaterfowlDefaults { ty: 603, width: 28, height: 22, life_max: 5, catch_item: 4359, walker_ty: 602 },
    // NPC.cs:16821-16832 — Grebe2 (catchItem 4374).
    WaterfowlDefaults { ty: 609, width: 28, height: 22, life_max: 5, catch_item: 4374, walker_ty: 608 },
];

/// The ambient inputs the arm reads off `Main` — the minimal seam (the
/// `Style24Ctx` shape minus the owl-only fields: no night scan, no
/// release/startle gate in this arm).
pub struct Style68Ctx<'a> {
    /// The player slots (the port has one session player); the style-24
    /// player seam carries `luck` + the pre-computed `IsThisCenx` answer
    /// for the shared poop gate.
    pub players: &'a [Style24Player],
    /// `Main.netMode` — the port runs 0 (single-player); 1 suppresses the
    /// poop gate (`:34625`), returns after the swim (`:34716-34719`) and
    /// skips the landing transform (`:34756`).
    pub net_mode: i32,
    /// `Main.EverLastingTicker` for the selection's `targetSetFrame`.
    pub ticker: u64,
}

/// What the arm asks the world sim to do — the request seam (the
/// `Style24Outcome` pattern).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Style68Outcome {
    /// `Projectile.NewProjectile(..., player.Center.X, base.Center.Y, 0f,
    /// 0f, 1080, 0, 0f)` (`NPC.cs:34632`): the BirdDroppings 1080, zero
    /// velocity/damage/knockback, at the carried (x, y).
    pub poop_shot: Option<(f32, f32)>,
    /// `this.Transform(this.type - 1, 0f, 200 + Main.rand.Next(200))`
    /// (`NPC.cs:34760`) — the dry landing's walker reveal: (newType, ai1).
    /// The core reset is applied to the slot in-place ([`transform_core`]);
    /// the full `SetDefaults` (sounds, value, the visuals) rides the wiring.
    pub transform: Option<(i32, f32)>,
    /// The arm's last `noGravity` write (`:34635` true, `:34638` false in
    /// the perch arm, `:34670` true again in the swim) — the `Npc` slot has
    /// no `noGravity` field yet.
    pub no_gravity: bool,
}

/// `WorldGen.SolidTile(int, int)` (`WorldGen.cs:70111-70132`): active, in
/// `Main.tileSolid`, NOT in `Main.tileSolidTop`, not half-brick, slope 0,
/// not actuated-inactive; a null tile (outside the array) is `true` — the
/// port's `World::tile` returns `None` there, matching. (The same shape as
/// `drawdata_cases_a.rs::solid_tile` / `grapple.rs::solid_tile`; kept local
/// so those files stay byte-identical this round — the shared-helper
/// extraction is the wiring step's, the `npc_ai_style16.rs` precedent.)
fn solid_tile(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    t.active()
        && TILE_SOLID.has(t.kind)
        && !TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
        && !t.inactive()
}

/// `Collision.CanHit(Entity, Entity)` (`Collision.cs:371-374`) over the
/// port's shared walk (`crate::projectile::can_hit`, `Collision.cs:381-479`).
fn can_hit_boxes(world: &World, ax: f32, ay: f32, aw: f32, ah: f32, bx: f32, by: f32, bw: f32, bh: f32) -> bool {
    crate::projectile::can_hit(world, ax, ay, aw, ah, bx, by, bw, bh)
}

/// The shared selection with this arm's fixed arguments —
/// `this.TargetClosest()` defaults `faceTarget = true`
/// (`NPC.cs:77652`), `aiStyle` 68 (not the fighter 3, so the centre rule
/// applies, `:77761`).
fn select_target(npc: &mut Npc, world: &World, cands: &[TargetCandidate], ticker: u64) {
    if !cands.is_empty() {
        target_closest(npc, world, cands, true, 68, ticker);
    }
}

/// The core of `NPC.Transform(type - 1, 0f, ai1)` (`NPC.cs:81084-81145`)
/// that the rest of the sim observes: the ±height reposition around
/// `SetDefaults` (`:81092-81100` — old height 22 down, new height 26 up →
/// net −4), the `ai[]`/`localAI[]` zeroing (`SetDefaults`, `:8564-8571`),
/// the `ClearTarget` 255 sentinel (`:8539`), `netUpdate = true` (`:8530`),
/// the collide-flag reset (`:8527-8528`), the `TargetClosest()` (`:81101`,
/// now on the NEW type's aiStyle 7 — neither 68 nor 7 is the fighter 3, so
/// the selection is identical), the life rescale `life * lifeMax /
/// oldLifeMax` floored at 1 (`:81106-81110`, integer division; both rows
/// are lifeMax 5 so the life passes through), and THEN the ai overrides
/// (`:81111-81114`: ai[0] = 0, ai[1] = the drawn 200..399). The walker
/// rows: 22×26, aiStyle 7, lifeMax 5 (`NPC.cs:13613-13624`/`:13637-13648`/
/// `:16739-16750`/`:16809-16820`). The value/knockback/sound/visual halves
/// (`TransformVisuals`, `GiveTownUniqueDataToNPCsThatNeedIt`) are the
/// wiring's full-`Transform` job — recorded in ANALYSIS.
fn transform_core(npc: &mut Npc, world: &World, cands: &[TargetCandidate], ticker: u64, ai1: f32) {
    let (_, old_h) = npc.size();
    let old_life = npc.hp;
    let old_life_max = npc.max_hp;
    // :81094 — `position.Y += height` (the OLD height).
    npc.y += old_h;
    // :81096 SetDefaults(newType) — the observable subset.
    npc.size_override = Some((22.0, 26.0)); // the four walker rows
    npc.ai = [0.0; 4]; // :8564-8567
    npc.local_ai = [0.0; 4]; // :8568-8571
    npc.target = 255; // ClearTarget, :8539
    npc.old_target = npc.target; // :8540
    npc.target_rect = (0, 0, 0, 0); // :8541
    npc.net_update = true; // :8530
    npc.collide_x = false; // :8527-8528
    npc.collide_y = false;
    npc.max_hp = 5; // the walker rows' lifeMax
    // :81099 — `position.Y -= height` (the NEW height).
    npc.y -= 26.0;
    // :81101 — TargetClosest on the new type (aiStyle 7).
    if !cands.is_empty() {
        target_closest(npc, world, cands, true, 7, ticker);
    }
    // :81106-81110 — the integer rescale, floored at 1.
    npc.hp = (old_life * npc.max_hp / old_life_max).max(1);
    // :81111-81114 — the ai overrides AFTER the reset (ai0 = 0f default).
    npc.ai[0] = 0.0;
    npc.ai[1] = ai1;
    // The type flip itself rides `Style68Outcome::transform` (the port's
    // kinds carry no waterfowl/walker rows yet).
}

/// The aiStyle-68 arm (`NPC.cs:34623-34879`), in the C#'s exact order:
/// the poop gate → `noGravity = true` → the `ai[0]` state machine
/// (0 swimming-perch / else flight with the 300-tick landing). `ty` is
/// `this.type` (the port's kinds carry no waterfowl rows yet).
pub fn ai_style_68(
    npc: &mut Npc,
    ty: i32,
    world: &World,
    ctx: &Style68Ctx,
    rng: &mut UnifiedRandom,
) -> Style68Outcome {
    let mut out = Style68Outcome::default();
    let cands: Vec<TargetCandidate> = ctx.players.iter().map(|p| p.candidate).collect();
    let (w, h) = npc.size();
    let (wi, hi) = (w as i32, h as i32);
    let (cx, cy) = (npc.x + w * 0.5, npc.y + h * 0.5); // Entity.Center, Entity.cs:50-55

    // :34625-34634 — the poop gate (the style-24 gate verbatim, same set).
    if ctx.net_mode != 1 && npc.ai[0] == 1.0 && bird_that_can_poop(ty) && npc.local_ai[3] == 0.0 {
        // :34627 — Player.FindClosest(base.position, base.width, base.height).
        let j = find_closest_player(&cands, npc.x, npc.y, wi, hi);
        if let Some(p) = ctx.players.get(j) {
            let pc = &p.candidate;
            // :34628-34629 — Center.X inside the player's [Left.X, Right.X]
            // (Entity.cs:62-84), the player's Top.Y (= position.Y) strictly
            // BELOW the bird's Bottom.Y (= position.Y + height), line of
            // sight, then the roll == 0 OR the Cenx bypass — the roll is
            // the LEFT operand, so it is evaluated (and draws) either way.
            let x3 = cx;
            if x3 >= pc.x
                && x3 <= pc.x + pc.width as f32
                && pc.y > npc.y + h
                && can_hit_boxes(world, npc.x, npc.y, w, h, pc.x, pc.y, pc.width as f32, pc.height as f32)
                && (roll_only_bad_luck_extreme(p.luck, 50, rng) == 0 || p.is_cenx)
            {
                // :34631-34632.
                npc.local_ai[3] = 1.0;
                let pcx = pc.x + pc.width as f32 / 2.0; // player Center.X
                out.poop_shot = Some((pcx, cy));
            }
        }
    }

    // :34635 — `this.noGravity = true`.
    out.no_gravity = true;

    if npc.ai[0] == 0.0 {
        // :34636-34735 — the swimming perch.
        out.no_gravity = false; // :34638
        // :34639-34645 — keep the OLD facing when there WAS a target:
        // num999 = direction, num1000 = the OLD target; TargetClosest may
        // re-face, but the old direction wins when num1000 >= 0 && the old
        // direction was non-zero.
        let old_dir = npc.facing;
        let old_target = npc.target;
        select_target(npc, world, &cands, ctx.ticker); // :34641
        if old_target >= 0 && old_dir != 0 {
            npc.facing = old_dir; // :34644
        }
        // :34646-34715 — the wet swim. The gate column is the INTEGER
        // division of the truncated probe X (`(int)(Center.X + (width/2 +
        // 8) * direction) / 16`, :34646 — cast FIRST, then int /16), the
        // row is `(int)(Center.Y / 16f)`; InWorld fluff 5
        // (WorldGen.cs:9688-9695).
        let probe_col = (cx + ((wi / 2 + 8) * npc.facing) as f32) as i32 / 16;
        let probe_row = (cy / 16.0) as i32;
        let in_world = probe_col >= 5
            && probe_col < world.width - 5
            && probe_row >= 5
            && probe_row < world.height - 5;
        if npc.wet && in_world {
            // :34648 — the 19/20 blend toward 2·direction.
            npc.vx = (npc.vx * 19.0 + 2.0 * npc.facing as f32) / 20.0;
            // :34649-34652 — the three probe rows: centre, top, feet.
            let j3 = (npc.y / 16.0) as i32;
            let feet_row = ((npc.y + h) / 16.0) as i32;
            // :34661 — the obstacle/bank flip: the 5-tile world margin
            // (the STRICT `> maxTilesX - 5` kept, unlike InWorld's `>=`),
            // any of the three SolidTile probes, or a dry feet cell.
            let feet_liquid = world.tile(probe_col, feet_row).map_or(0, |t| t.liquid);
            if probe_col < 5
                || probe_col > world.width - 5
                || solid_tile(world, probe_col, probe_row)
                || solid_tile(world, probe_col, j3)
                || solid_tile(world, probe_col, feet_row)
                || feet_liquid == 0
            {
                npc.facing *= -1; // :34663
            }
            npc.sprite_dir = npc.facing; // :34665
            // :34666-34669 — a sinking velocity halves.
            if npc.vy > 0.0 {
                npc.vy *= 0.5;
            }
            out.no_gravity = true; // :34670
            // :34671-34698 — the water line: the CENTRE column/row now
            // (`(int)(Center.X / 16f)`), the default line is the feet; the
            // first liquid-bearing row of (row−1, row, row+1) sets the
            // surface at that row's bottom minus `liquid / 16` (INTEGER
            // division). NOT `Collision.GetWaterLine` — this inline form
            // has no `y−2` dry gate and no InWorld guard, kept verbatim.
            let col2 = (cx / 16.0) as i32;
            let row2 = (cy / 16.0) as i32;
            let liq = |r: i32| -> i32 { world.tile(col2, r).map_or(0, |t| t.liquid as i32) };
            let mut line = npc.y + h; // :34673
            if liq(row2 - 1) > 0 {
                // :34688.
                line = (row2 * 16) as f32 - (liq(row2 - 1) / 16) as f32;
            } else if liq(row2) > 0 {
                // :34692.
                line = ((row2 + 1) * 16) as f32 - (liq(row2) / 16) as f32;
            } else if liq(row2 + 1) > 0 {
                // :34696.
                line = ((row2 + 2) * 16) as f32 - (liq(row2 + 1) / 16) as f32;
            }
            line -= 6.0; // :34698
            // :34699-34714 — the hover: below the line rise −0.1 (cap −8)
            // with the one-step-ahead snap; at/above it pin to the line.
            if cy > line {
                npc.vy -= 0.1;
                if npc.vy < -8.0 {
                    npc.vy = -8.0;
                }
                if cy + npc.vy < line {
                    npc.vy = line - cy;
                }
            } else {
                npc.vy = line - cy;
            }
        }
        // :34716-34719 — the client stops here.
        if ctx.net_mode == 1 {
            return out;
        }
        // :34720-34726 — dry: take off at once (no impulse).
        if !npc.wet {
            npc.ai[0] = 1.0;
            npc.net_update = true;
            npc.facing = -npc.facing;
            return out;
        }
        // :34727-34734 — the proximity flutter: the player's int-truncated
        // rect vs the bird's rect grown 100 px on every side, XNA
        // Rectangle.Intersects (strict edges), or any damage taken.
        if let Some(p) = cands.get(npc.target.max(0) as usize) {
            let (pw, ph) = (p.width, p.height);
            let (px, py) = (p.x as i32, p.y as i32);
            let (bx, by) = (npc.x as i32 - 100, npc.y as i32 - 100);
            let (bw, bh) = (wi + 200, hi + 200);
            let intersects = px < bx + bw && bx < px + pw && py < by + bh && by < py + ph;
            if intersects || npc.hp < npc.max_hp {
                npc.ai[0] = 1.0;
                npc.vy -= 6.0; // :34731
                npc.net_update = true;
                npc.facing = -npc.facing;
            }
        }
    } else {
        // :34736-34878 — the flight.
        // :34738-34741 — a dead target freezes the machine (an absent slot
        // reads like the C#'s inactive default, dead=false).
        if cands.get(npc.target.max(0) as usize).map_or(false, |p| p.dead) {
            return out;
        }
        // :34742-34747 — the 300-tick flight timer.
        npc.ai[1] += 1.0;
        let flag71 = npc.ai[1] >= 300.0;
        if flag71 {
            // :34748-34775 — the landing attempt.
            if npc.vy == 0.0 || npc.collide_y || npc.wet {
                // :34752-34755 — touched down: zero everything.
                npc.vx = 0.0;
                npc.vy = 0.0;
                npc.ai[0] = 0.0;
                npc.ai[1] = 0.0;
                if ctx.net_mode != 1 {
                    // :34758-34761 — the dry landing transforms back into
                    // the walker; the `200 + Main.rand.Next(200)` argument
                    // is evaluated (ONE draw) only inside this gate.
                    if (ty == 363 || ty == 365 || ty == 603 || ty == 609) && !npc.wet {
                        let ai1 = 200.0 + rng.next_max(200) as f32;
                        transform_core(npc, world, &cands, ctx.ticker, ai1);
                        out.transform = Some((ty - 1, ai1));
                    }
                    npc.net_update = true; // :34762
                }
            } else {
                // :34765-34773 — still airborne: damp and sink, cap 2.
                npc.vx *= 0.98;
                npc.vy += 0.1;
                if npc.vy > 2.0 {
                    npc.vy = 2.0;
                }
            }
            return out; // :34774
        }
        // :34776-34788 — the X bounce off oldVelocity with the ±2 floors.
        if npc.collide_x {
            npc.facing *= -1;
            npc.vx = npc.old_vx * -0.5;
            if npc.facing == -1 && npc.vx > 0.0 && npc.vx < 2.0 {
                npc.vx = 2.0;
            }
            if npc.facing == 1 && npc.vx < 0.0 && npc.vx > -2.0 {
                npc.vx = -2.0;
            }
        }
        // :34789-34800 — the Y bounce with the ±1 floors.
        if npc.collide_y {
            npc.vy = npc.old_vy * -0.5;
            if npc.vy > 0.0 && npc.vy < 1.0 {
                npc.vy = 1.0;
            }
            if npc.vy < 0.0 && npc.vy > -1.0 {
                npc.vy = -1.0;
            }
        }
        // :34801-34832 — the X steering (the 3f cap). The inner `> 3` /
        // `< -3` tests are the C#'s own (dead on the facing side, live
        // after a bounce pushed the velocity past the cap) — kept verbatim.
        if npc.facing == -1 && npc.vx > -3.0 {
            npc.vx -= 0.1;
            if npc.vx > 3.0 {
                npc.vx -= 0.1;
            } else if npc.vx > 0.0 {
                npc.vx -= 0.05;
            }
            if npc.vx < -3.0 {
                npc.vx = -3.0;
            }
        } else if npc.facing == 1 && npc.vx < 3.0 {
            npc.vx += 0.1;
            if npc.vx < -3.0 {
                npc.vx += 0.1;
            } else if npc.vx < 0.0 {
                npc.vx += 0.05;
            }
            if npc.vx > 3.0 {
                npc.vx = 3.0;
            }
        }
        // :34833-34857 — the ground probe: one column, `direction` tiles
        // ahead of the centre, 15 rows down from the feet. INTEGER
        // half-width (`(float)(base.width / 2)`, :34833); the C# (int)
        // casts truncate toward zero. InWorld fluff 2 (:34840).
        let col = ((npc.x + (wi / 2) as f32) / 16.0) as i32 + npc.facing;
        let row = ((npc.y + h) / 16.0) as i32;
        let mut no_ground = true; // flag72
        let mut ground_near = false; // flag73
        for r in row..row + 15 {
            // :34840-34843 — WorldGen.InWorld(col, r, 2).
            if col < 2 || col >= world.width - 2 || r < 2 || r >= world.height - 2 {
                continue;
            }
            // :34844-34847 — a null tile is a fresh empty Tile (not solid,
            // no liquid) — the port's None reads the same.
            let blocked = world.tile(col, r).map_or(false, |t| {
                // :34848 — `nactive() && tileSolid[type]` (NO solidTop /
                // slope / halfBrick terms here) OR any liquid.
                (t.active() && !t.inactive() && TILE_SOLID.has(t.kind)) || t.liquid > 0
            });
            if blocked {
                if r < row + 5 {
                    ground_near = true; // :34850-34853
                }
                no_ground = false;
                break; // :34855
            }
        }
        // :34858-34877 — the hover corrections and caps (the waterfowl
        // sink +0.1 and cap +3/−4 — NOT the style-24 bird's +0.05/+2).
        if no_ground {
            npc.vy += 0.1;
        } else {
            npc.vy -= 0.1;
        }
        if ground_near {
            npc.vy -= 0.2;
        }
        if npc.vy > 3.0 {
            npc.vy = 3.0;
        }
        if npc.vy < -4.0 {
            npc.vy = -4.0;
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use crate::npc_ai_style24::is_this_cenx;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

    /// A waterfowl-shaped slot: the port has no waterfowl kinds yet, so the
    /// tests ride a carrier kind with the row's `size_override` (the wiring
    /// does the same until the kinds land).
    fn bird(ty: i32, x: f32, y: f32) -> Npc {
        let row = STYLE68_BIRDS.iter().find(|r| r.ty == ty).unwrap();
        let mut n = Npc::new(NpcKind::Zombie, x, y, 1, 0.0);
        n.size_override = Some((row.width as f32, row.height as f32));
        n.hp = row.life_max;
        n.max_hp = row.life_max;
        n
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

    fn player(x: f32, y: f32, luck: f32, is_cenx: bool) -> Style24Player {
        Style24Player { candidate: cand(x, y), luck, is_cenx }
    }

    fn air() -> World {
        World::new("t".into(), 200, 120)
    }

    /// An air world with a liquid floor at row 100 — the flight probe
    /// only scans 15 rows below the feet (rows 63..78), so the floor
    /// never reads as ground. NOTE the bird's own feet row 63 stays DRY
    /// here: a swim fixture that must NOT take the dry-feet flip
    /// (`NPC.cs:34661`) calls `wet_feet` below (the flutter/full-cycle
    /// witnesses deliberately ride the dry-feet line = feet path).
    fn pool() -> World {
        let mut w = air();
        for x in 0..200 {
            let i = w.index(x, 100);
            w.tiles[i] = Tile { kind: 0, liquid: 255, ..Default::default() };
        }
        w
    }

    /// Liquid at the bird's own cells: a bird at (1000, 1000) has centre
    /// row 63 / feet row 63 (`(int)((Y + height) / 16f)`, `:34652`) and
    /// probe columns 62 (facing -1) / 64 (facing +1) (`:34649`), so rows
    /// 62..64 × cols 61..65 carry liquid 255 — the swim's dry-feet flip
    /// (`:34661`) stays quiet and the water line reads row 62's surface
    /// (`:34686-34688`: 63*16 − 255/16 = 993, −6 → 987).
    fn wet_feet(w: &mut World) {
        for r in 62..=64 {
            for col in 61..=65 {
                let i = w.index(col, r);
                w.tiles[i] = Tile { kind: 0, liquid: 255, ..Default::default() };
            }
        }
    }

    fn ctx(players: &[Style24Player]) -> Style68Ctx<'_> {
        Style68Ctx { players, net_mode: 0, ticker: 7 }
    }

    /// `NPC.cs:13625-13636`, `:13649-13660`, `:16751-16762`,
    /// `:16821-16832` — the 4 SetDefaults rows (the census).
    #[test]
    fn the_style68_rows_match_set_defaults() {
        assert_eq!(STYLE68_BIRDS.len(), 4, "the census: exactly four aiStyle = 68 rows");
        for r in STYLE68_BIRDS {
            assert_eq!((r.width, r.height, r.life_max), (28, 22, 5), "type {}", r.ty);
            assert_eq!(r.walker_ty, r.ty - 1, "the landing transforms to type - 1");
        }
        let items: Vec<i32> = STYLE68_BIRDS.iter().map(|r| r.catch_item).collect();
        assert_eq!(items, vec![2122, 2123, 4359, 4374]);
        // All four are in BirdThatCanPoop (NPCID.cs:10435); the walkers
        // (362/364/602/608) are NOT.
        for r in STYLE68_BIRDS {
            assert!(bird_that_can_poop(r.ty));
            assert!(!bird_that_can_poop(r.walker_ty));
        }
    }

    /// `NPC.cs:34625-34633` — the poop gate: the column alignment, the
    /// latch, and the Cenx bypass (a non-negative-luck player draws
    /// NOTHING — the shared `Luck.cs:55` short-circuit).
    #[test]
    fn the_poop_needs_the_player_directly_below_and_latches() {
        let w = air();
        let players = [player(1000.0, 1100.0, 0.0, true)];
        let c = ctx(&players);
        // Bird centre 1014 inside [1000, 1020], bottom 1022 < player top 1100.
        let mut n = bird(363, 1000.0, 1000.0);
        n.ai[0] = 1.0;
        let mut rng = UnifiedRandom::new(1);
        let before = rng.clone();
        let out = ai_style_68(&mut n, 363, &w, &c, &mut rng);
        assert_eq!(out.poop_shot, Some((1010.0, 1011.0)), "player Centre.X, bird Centre.Y");
        assert_eq!(n.local_ai[3], 1.0, "the one-shot latch (:34631)");
        assert_eq!(rng, before, "the Cenx bypass with luck 0 draws nothing");
        // The latch blocks the next tick.
        let out2 = ai_style_68(&mut n, 363, &w, &c, &mut rng);
        assert_eq!(out2.poop_shot, None);
        // Beside the player: centre 1064 > Right 1020 — no poop.
        let mut n2 = bird(363, 1050.0, 1000.0);
        n2.ai[0] = 1.0;
        let out3 = ai_style_68(&mut n2, 363, &w, &c, &mut UnifiedRandom::new(1));
        assert_eq!(out3.poop_shot, None);
        assert_eq!(n2.local_ai[3], 0.0);
    }

    /// `NPC.cs:34629` → `Luck.cs:53-60` — the RNG stream witness: a
    /// negative-luck player under the flying bird consumes EXACTLY the
    /// draws the C# would (NextFloat, then Next(5) only on the hit), in
    /// order, whether or not the poop fires.
    #[test]
    fn the_poop_roll_replays_the_csharp_stream() {
        let w = air();
        for seed in 0..64 {
            let players = [player(1000.0, 1100.0, -0.5, false)];
            let c = ctx(&players);
            let mut n = bird(603, 1000.0, 1000.0);
            n.ai[0] = 1.0;
            let mut a = UnifiedRandom::new(seed);
            let mut b = UnifiedRandom::new(seed);
            let out = ai_style_68(&mut n, 603, &w, &c, &mut a);
            // The C# order: one NextFloat, then Next(50/10) only on a hit.
            let f = b.next_float();
            let roll = if f < 0.5 { b.next_max(5) } else { -1 };
            assert_eq!(out.poop_shot.is_some(), roll == 0, "seed {seed}");
            assert_eq!(a, b, "seed {seed}: the streams must stay aligned");
        }
    }

    /// `NPC.cs:34639-34645` — the perch restores the OLD facing after
    /// TargetClosest re-faces. NOTE the C# gate `num1000 >= 0` reads the
    /// OLD target: `ClearTarget()` sets 255 (`:42924-42927`), and 255 >= 0
    /// is TRUE in the C# int — the restore fires for the sentinel too.
    /// The only skip is the field initializer's -1 (`NPC.cs:6457`), which
    /// SetDefaults overwrites before any AI tick — unreachable live.
    #[test]
    fn the_perch_keeps_its_old_facing_when_it_had_a_target() {
        let mut w = pool();
        // Wet feet in BOTH probe columns (62 for facing -1, 64 for +1):
        // the dry-feet flip (:34661) would otherwise undo the restore.
        wet_feet(&mut w);
        // The player is to the RIGHT: a fresh TargetClosest faces +1.
        let players = [player(2000.0, 1000.0, 0.0, false)];
        let c = ctx(&players);
        let mut n = bird(363, 1000.0, 1000.0);
        n.facing = -1;
        n.old_direction = -1; // the UpdateNPC pre-AI latch (:90393-90395)
        n.target = 0; // there WAS a target
        n.old_target = 0;
        n.wet = true; // stay perched (the dry arm would take off)
        ai_style_68(&mut n, 363, &w, &c, &mut UnifiedRandom::new(1));
        assert_eq!(n.facing, -1, "the old direction wins (:34644)");
        // The 255 ClearTarget sentinel ALSO restores: 255 >= 0 passes the
        // C# gate (`:34642`), so TargetClosest's +1 is overwritten by the
        // old -1 again. (The earlier witness expected +1 here — wrong:
        // 255 is not a negative sentinel in the C# int compare.)
        let mut n2 = bird(363, 1000.0, 1000.0);
        n2.facing = -1;
        n2.old_direction = -1;
        n2.target = 255;
        n2.wet = true;
        ai_style_68(&mut n2, 363, &w, &c, &mut UnifiedRandom::new(1));
        assert_eq!(n2.facing, -1, "255 >= 0: the restore fires (:34642-34644)");
        // The true skip: the field-initializer -1 (`NPC.cs:6457`) — the
        // only value that fails `num1000 >= 0`. TargetClosest's facing
        // stands.
        let mut n3 = bird(363, 1000.0, 1000.0);
        n3.facing = -1;
        n3.old_direction = -1;
        n3.target = -1;
        n3.wet = true;
        ai_style_68(&mut n3, 363, &w, &c, &mut UnifiedRandom::new(1));
        assert_eq!(n3.facing, 1, "-1 < 0: TargetClosest's facing stands");
    }

    /// `NPC.cs:34720-34726` — a dry perch takes off at once: ai[0] = 1,
    /// netUpdate, the facing flip, NO vy impulse (unlike the flutter).
    #[test]
    fn the_dry_perch_takes_off_without_an_impulse() {
        let w = air();
        let players = [player(2000.0, 1000.0, 0.0, false)];
        let c = ctx(&players);
        let mut n = bird(365, 1000.0, 1000.0);
        n.target = 0;
        let out = ai_style_68(&mut n, 365, &w, &c, &mut UnifiedRandom::new(1));
        assert_eq!(n.ai[0], 1.0);
        assert!(n.net_update);
        assert_eq!(n.facing, -1);
        assert_eq!(n.vy, 0.0, "no impulse on the dry takeoff");
        assert!(!out.no_gravity, "the perch arm clears noGravity (:34638)");
    }

    /// `NPC.cs:34727-34734` — the wet proximity flutter: the grown-100
    /// rect intersect (or damage) launches with vy -= 6 and the flip.
    #[test]
    fn the_wet_perch_flutters_when_the_player_comes_close() {
        let w = pool();
        let far = [player(1500.0, 1000.0, 0.0, false)];
        let c = ctx(&far);
        let mut n = bird(363, 1000.0, 1000.0);
        n.wet = true;
        n.target = 0;
        ai_style_68(&mut n, 363, &w, &c, &mut UnifiedRandom::new(1));
        assert_eq!(n.ai[0], 0.0, "the far player leaves the bird swimming");
        // Inside the grown rect (bird rect x 900..1228 with the +200).
        let near = [player(1100.0, 1000.0, 0.0, false)];
        let c2 = ctx(&near);
        let mut n2 = bird(363, 1000.0, 1000.0);
        n2.wet = true;
        n2.target = 0;
        ai_style_68(&mut n2, 363, &w, &c2, &mut UnifiedRandom::new(1));
        assert_eq!(n2.ai[0], 1.0);
        // The swim ran first (wet, in world): the water line defaults to
        // the feet (no liquid in the air world) → line = 1022 − 6 = 1016,
        // centre 1011 ≤ line → vy = 5; then the flutter −6 → −1.
        assert_eq!(n2.vy, -1.0, "the swim pin then the −6 flutter (:34731)");
        // Damage alone launches even with the player far away.
        let mut n3 = bird(363, 1000.0, 1000.0);
        n3.wet = true;
        n3.target = 0;
        n3.hp = 4;
        ai_style_68(&mut n3, 363, &w, &c, &mut UnifiedRandom::new(1));
        assert_eq!(n3.ai[0], 1.0);
    }

    /// `NPC.cs:34646-34714` — the wet swim: the 19/20 X blend, the sinking
    /// halve, spriteDirection = direction, and the water-line hover with
    /// the integer `liquid / 16` shortfall and the −6 offset.
    #[test]
    fn the_wet_swim_blends_and_hovers_on_the_water_line() {
        let mut w = air();
        // Bird 363 at (1000, 1000): centre (1014, 1011), centre col 63,
        // centre row 63; probe col (1014 + 22)/16 = 64, feet row 63.
        // Water in the centre column rows 63/64 (the bird's own cells).
        for r in [63, 64] {
            let i = w.index(63, r);
            w.tiles[i] = Tile { kind: 0, liquid: 255, ..Default::default() };
        }
        // The probe column needs liquid at the feet row too (else flip).
        let i = w.index(64, 63);
        w.tiles[i] = Tile { kind: 0, liquid: 255, ..Default::default() };
        let players = [player(2000.0, 1000.0, 0.0, false)];
        let c = ctx(&players);
        let mut n = bird(363, 1000.0, 1000.0);
        n.wet = true;
        n.target = 0;
        n.facing = 1;
        n.vx = 1.0;
        n.vy = 2.0; // sinking
        let out = ai_style_68(&mut n, 363, &w, &c, &mut UnifiedRandom::new(1));
        assert!(out.no_gravity, "the swim re-arms noGravity (:34670)");
        assert_eq!(n.vx, (1.0 * 19.0 + 2.0) / 20.0, "the 19/20 blend (:34648)");
        assert_eq!(n.sprite_dir, 1, "spriteDirection = direction (:34665)");
        // vy halved to 1; the line: row 62 dry, row 63 liquid 255 →
        // (63+1)*16 − 255/16 = 1024 − 15 = 1009, −6 → 1003. Centre 1011 >
        // 1003 → vy 1 − 0.1 = 0.9; 1011 + 0.9 = 1011.9 ≥ 1003, no snap.
        assert_eq!(n.vy, 0.9, "the halve then the −0.1 rise");
        assert_eq!(n.facing, 1, "no obstacle: the facing stands");
    }

    /// `NPC.cs:34661-34664` — the obstacle flip: a solid probe cell or a
    /// dry feet cell reverses the direction (and spriteDirection follows).
    #[test]
    fn the_swim_flips_at_a_bank_or_a_dry_feet_cell() {
        let mut w = air();
        // Liquid everywhere around the bird EXCEPT the probe feet cell.
        for r in 62..66 {
            for col in 62..66 {
                let i = w.index(col, r);
                w.tiles[i] = Tile { kind: 0, liquid: 255, ..Default::default() };
            }
        }
        let players = [player(2000.0, 1000.0, 0.0, false)];
        let c = ctx(&players);
        let mut n = bird(363, 1000.0, 1000.0);
        n.wet = true;
        n.target = 0;
        n.facing = 1;
        ai_style_68(&mut n, 363, &w, &c, &mut UnifiedRandom::new(1));
        assert_eq!(n.facing, 1, "the wet probe column keeps the facing");
        // Dry the feet cell of the probe column (64, 63).
        let i = w.index(64, 63);
        w.tiles[i].liquid = 0;
        let mut n2 = bird(363, 1000.0, 1000.0);
        n2.wet = true;
        n2.target = 0;
        n2.facing = 1;
        ai_style_68(&mut n2, 363, &w, &c, &mut UnifiedRandom::new(1));
        assert_eq!(n2.facing, -1, "the dry feet cell flips (:34661-34663)");
        assert_eq!(n2.sprite_dir, -1);
        // A solid tile in the probe column flips too.
        let i2 = w.index(64, 63);
        w.tiles[i2] = Tile { kind: 1, liquid: 255, ..Default::default() };
        w.tiles[i2].flags |= FLAG_ACTIVE;
        let mut n3 = bird(363, 1000.0, 1000.0);
        n3.wet = true;
        n3.target = 0;
        n3.facing = 1;
        ai_style_68(&mut n3, 363, &w, &c, &mut UnifiedRandom::new(1));
        assert_eq!(n3.facing, -1, "SolidTile flips");
    }

    /// `NPC.cs:34738-34741` — a dead target freezes the flight: no timer
    /// increment, no steering, noGravity stays true.
    #[test]
    fn the_dead_target_freezes_the_flight() {
        let w = air();
        let mut dead = player(2000.0, 1000.0, 0.0, false);
        dead.candidate.dead = true;
        let players = [dead];
        let c = ctx(&players);
        let mut n = bird(609, 1000.0, 1000.0);
        n.ai[0] = 1.0;
        n.ai[1] = 42.0;
        n.target = 0;
        n.local_ai[3] = 1.0; // skip the poop gate
        let out = ai_style_68(&mut n, 609, &w, &c, &mut UnifiedRandom::new(1));
        assert_eq!(n.ai[1], 42.0, "the timer never increments");
        assert_eq!(n.vx, 0.0);
        assert!(out.no_gravity);
    }

    /// `NPC.cs:34743-34774` + `:81084-81114` — the dry landing at tick 300:
    /// the zeros, the Transform(type − 1) with the ONE Next(200) draw, the
    /// −4 reposition (22 down, 26 up), the 22×26 walker box, ai[1] = the
    /// drawn 200..399 AFTER the reset, and the stream witness.
    #[test]
    fn the_dry_landing_transforms_back_into_the_walker() {
        let w = air();
        let players = [player(2000.0, 1000.0, 0.0, false)];
        let c = ctx(&players);
        for seed in 0..32 {
            let mut n = bird(363, 1000.0, 1000.0);
            n.ai[0] = 1.0;
            n.ai[1] = 299.0; // increments to 300 this tick
            n.target = 0;
            n.local_ai[3] = 1.0;
            n.vy = 0.0; // grounded
            n.vx = 2.0;
            n.hp = 3; // damaged: 3 * 5 / 5 = 3 passes the rescale
            let mut a = UnifiedRandom::new(seed);
            let mut b = UnifiedRandom::new(seed);
            let out = ai_style_68(&mut n, 363, &w, &c, &mut a);
            let want_ai1 = 200.0 + b.next_max(200) as f32;
            assert_eq!(out.transform, Some((362, want_ai1)), "seed {seed}");
            assert_eq!(a, b, "seed {seed}: exactly ONE Next(200) draw");
            assert_eq!(n.y, 996.0, "the ±height reposition nets −4");
            assert_eq!(n.size(), (22.0, 26.0), "the walker box");
            assert_eq!(n.ai[0], 0.0);
            assert_eq!(n.ai[1], want_ai1, "ai[1] = the drawn idle (:81112)");
            assert_eq!(n.local_ai, [0.0; 4], "SetDefaults zeroed localAI");
            assert_eq!(n.vx, 0.0);
            assert_eq!(n.vy, 0.0);
            assert_eq!(n.hp, 3);
            assert!(n.net_update);
            assert_eq!(n.target, 0, "the Transform's TargetClosest (:81101)");
            assert_eq!(n.target_set_frame, 7, "the ticker stamp");
        }
    }

    /// `NPC.cs:34750-34763` — a WET landing zeroes the state but never
    /// transforms (the `!base.wet` gate) and draws NOTHING; the client
    /// (netMode 1) skips the transform + netUpdate too.
    #[test]
    fn the_wet_landing_and_the_client_draw_nothing() {
        let w = air();
        let players = [player(2000.0, 1000.0, 0.0, false)];
        let c = ctx(&players);
        let mut n = bird(603, 1000.0, 1000.0);
        n.ai[0] = 1.0;
        n.ai[1] = 299.0;
        n.target = 0;
        n.local_ai[3] = 1.0;
        n.wet = true; // the wet landing path
        let mut rng = UnifiedRandom::new(5);
        let before = rng.clone();
        let out = ai_style_68(&mut n, 603, &w, &c, &mut rng);
        assert_eq!(out.transform, None);
        assert_eq!(n.ai[0], 0.0);
        assert_eq!(n.ai[1], 0.0);
        assert_eq!(rng, before, "no draw on the wet landing");
        // The client: the landing zeros run, the transform/netUpdate skip.
        let c2 = Style68Ctx { net_mode: 1, ..ctx(&players) };
        let mut n2 = bird(603, 1000.0, 1000.0);
        n2.ai[0] = 1.0;
        n2.ai[1] = 299.0;
        n2.target = 0;
        n2.local_ai[3] = 1.0;
        n2.vy = 0.0;
        n2.net_update = false;
        let out2 = ai_style_68(&mut n2, 603, &w, &c2, &mut rng);
        assert_eq!(out2.transform, None);
        assert!(!n2.net_update, "the client skips the netUpdate (:34756)");
        assert_eq!(rng, before, "the client draws nothing");
    }

    /// `NPC.cs:34765-34773` — the airborne 300-tick descent: ×0.98 X damp,
    /// +0.1 sink capped at 2, no landing until touchdown.
    #[test]
    fn the_airborne_landing_attempt_damps_and_sinks() {
        let w = air();
        let players = [player(2000.0, 1000.0, 0.0, false)];
        let c = ctx(&players);
        let mut n = bird(365, 1000.0, 1000.0);
        n.ai[0] = 1.0;
        n.ai[1] = 299.0;
        n.target = 0;
        n.local_ai[3] = 1.0;
        n.vx = 1.0;
        n.vy = 1.99; // airborne, not collideY, not wet
        let mut rng = UnifiedRandom::new(9);
        let before = rng.clone();
        let out = ai_style_68(&mut n, 365, &w, &c, &mut rng);
        assert_eq!(out.transform, None);
        assert_eq!(n.ai[0], 1.0, "still flying");
        assert_eq!(n.vx, 0.98);
        assert_eq!(n.vy, 2.0, "the 2f descent cap");
        assert_eq!(rng, before, "the descent draws nothing");
    }

    /// `NPC.cs:34776-34800` — the bounce arms: oldVelocity × −0.5 with the
    /// ±2 X floors and the ±1 Y floors.
    #[test]
    fn the_flight_bounces_off_old_velocity() {
        let w = air();
        let players = [player(2000.0, 1000.0, 0.0, false)];
        let c = ctx(&players);
        let mut n = bird(363, 1000.0, 1000.0);
        n.ai[0] = 1.0;
        n.target = 0;
        n.local_ai[3] = 1.0;
        n.collide_x = true;
        n.old_vx = 4.0; // flying right into a wall
        ai_style_68(&mut n, 363, &w, &c, &mut UnifiedRandom::new(1));
        assert_eq!(n.facing, -1, "the X bounce flips the direction");
        // vx = −2, facing −1: steering −0.1 (the > 0 extra skipped).
        assert_eq!(n.vx, -2.1);
        // The small-X floor: old_vx −1 → vx 0.5, facing flips to −1 →
        // floor 2 BEFORE the steering (2 > 0 adds the extra −0.05).
        let mut n2 = bird(363, 1000.0, 1000.0);
        n2.ai[0] = 1.0;
        n2.target = 0;
        n2.local_ai[3] = 1.0;
        n2.collide_x = true;
        n2.old_vx = -1.0;
        ai_style_68(&mut n2, 363, &w, &c, &mut UnifiedRandom::new(1));
        assert_eq!(n2.vx, 2.0 - 0.1 - 0.05, "floor 2 then −0.1 −0.05");
        // The Y bounce floor: old_vy 1 → −0.5 → floored to −1; the probe
        // finds no ground in the air world (+0.1) → −0.9.
        let mut n3 = bird(363, 1000.0, 1000.0);
        n3.ai[0] = 1.0;
        n3.target = 0;
        n3.local_ai[3] = 1.0;
        n3.collide_y = true;
        n3.old_vy = 1.0;
        ai_style_68(&mut n3, 363, &w, &c, &mut UnifiedRandom::new(1));
        assert_eq!(n3.vy, -0.9);
    }

    /// `NPC.cs:34833-34877` — the ground probe: solid or liquid within 15
    /// rows below the feet brakes (−0.1), within the first 5 rows brakes
    /// harder (−0.2 more); open air sinks +0.1; the +3/−4 caps hold.
    #[test]
    fn the_flight_probes_the_column_below() {
        let mut w = air();
        // Bird 363 at (1000, 1000): centre col (1000+14)/16 = 63, +facing
        // 1 → 64; feet row (1000+22)/16 = 63. Solid at row 65 (< 63+5).
        let i = w.index(64, 65);
        w.tiles[i] = Tile { kind: 1, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
        let players = [player(2000.0, 1000.0, 0.0, false)];
        let c = ctx(&players);
        let mut n = bird(363, 1000.0, 1000.0);
        n.ai[0] = 1.0;
        n.target = 0;
        n.local_ai[3] = 1.0;
        ai_style_68(&mut n, 363, &w, &c, &mut UnifiedRandom::new(1));
        // steering: facing 1, vx 0 → +0.1 (the < 0 extra skipped); probe:
        // near ground → −0.1 − 0.2 = −0.3.
        assert_eq!(n.vx, 0.1);
        assert_eq!(n.vy, -0.3, "the near-ground double brake");
        // Liquid counts as ground too (:34848); far ground (row 70 ≥ 68).
        let mut w2 = air();
        let i2 = w2.index(64, 70);
        w2.tiles[i2] = Tile { kind: 0, liquid: 1, ..Default::default() };
        let mut n2 = bird(363, 1000.0, 1000.0);
        n2.ai[0] = 1.0;
        n2.target = 0;
        n2.local_ai[3] = 1.0;
        ai_style_68(&mut n2, 363, &w2, &c, &mut UnifiedRandom::new(1));
        assert_eq!(n2.vy, -0.1, "the far-ground single brake");
        // Open air sinks +0.1 (NOT the style-24 bird's +0.05).
        let w3 = air();
        let mut n3 = bird(363, 1000.0, 1000.0);
        n3.ai[0] = 1.0;
        n3.target = 0;
        n3.local_ai[3] = 1.0;
        ai_style_68(&mut n3, 363, &w3, &c, &mut UnifiedRandom::new(1));
        assert_eq!(n3.vy, 0.1, "open air sinks +0.1");
        // The caps: +3 sink, −4 rise.
        let mut n4 = bird(363, 1000.0, 1000.0);
        n4.ai[0] = 1.0;
        n4.target = 0;
        n4.local_ai[3] = 1.0;
        n4.vy = 2.99;
        ai_style_68(&mut n4, 363, &w3, &c, &mut UnifiedRandom::new(1));
        assert_eq!(n4.vy, 3.0, "the sink cap");
        let mut n5 = bird(363, 1000.0, 1000.0);
        n5.ai[0] = 1.0;
        n5.target = 0;
        n5.local_ai[3] = 1.0;
        n5.vy = -3.99;
        ai_style_68(&mut n5, 363, &w, &c, &mut UnifiedRandom::new(1));
        assert_eq!(n5.vy, -4.0, "the rise cap");
    }

    /// `NPC.cs:34716-34719` — the client's perch returns after the swim:
    /// no takeoff, no flutter, and the poop gate is suppressed (:34625).
    #[test]
    fn the_client_perch_returns_after_the_swim() {
        let w = pool();
        let players = [player(1000.0, 1100.0, 0.0, true)];
        let c = Style68Ctx { net_mode: 1, ..ctx(&players) };
        let mut n = bird(363, 1000.0, 1000.0);
        n.target = 0;
        n.old_target = 0; // the pre-AI latch (:90393-90395) — an unchanged
        n.net_update = false; // target arms NO flag; the takeoff is what's skipped
        // Wet: the single-player arm would flutter; the client returns
        // after the swim with no state change.
        n.wet = true;
        // Steady-state facing: the player centre 1010 sits LEFT of the
        // bird centre 1014, so SetTargetTrackingValues faces -1
        // (`:77751-77754`). Mirror the UpdateNPC pre-AI latch with that
        // settled facing (`:90393-90395`) — then target/facing/directionY
        // are all unchanged and the shared change flag (`:77776-77779`)
        // arms NOTHING; the takeoff netUpdate is the only flag left for
        // the client to (not) raise.
        n.facing = -1;
        n.old_direction = -1;
        let out = ai_style_68(&mut n, 363, &w, &c, &mut UnifiedRandom::new(1));
        assert_eq!(n.ai[0], 0.0, "no flutter on the client");
        assert!(!n.net_update, "the client skips the takeoff netUpdate");
        assert_eq!(out.poop_shot, None);
        // Flying with the poop alignment: the gate's netMode check skips.
        let mut n2 = bird(363, 1000.0, 1000.0);
        n2.ai[0] = 1.0;
        n2.target = 0;
        let out2 = ai_style_68(&mut n2, 363, &w, &c, &mut UnifiedRandom::new(1));
        assert_eq!(out2.poop_shot, None);
        assert_eq!(n2.local_ai[3], 0.0);
    }

    /// The RNG-by-construction witness: a ten-tick replay of a full
    /// swim → flutter → flight → dry-landing cycle against a mirrored
    /// oracle stream that draws ONLY at the two cited sites — the states
    /// must stay aligned tick by tick.
    #[test]
    fn the_full_cycle_replays_the_cited_draws_only() {
        let w = pool();
        // Cenx-named player directly below: the poop fires with luck 0
        // (no draw), then the landing draws its one Next(200). The pure
        // fn never integrates position, so the column stays aligned.
        let players = [player(1000.0, 1100.0, 0.0, is_this_cenx("Cenx"))];
        let c = ctx(&players);
        let mut n = bird(609, 1000.0, 1000.0);
        n.ai[0] = 1.0;
        n.ai[1] = 295.0; // lands within the window
        n.target = 0;
        n.vx = 1.0; // airborne: the landing attempt descends until touchdown
        n.vy = 1.0;
        let mut a = UnifiedRandom::new(77);
        let mut b = UnifiedRandom::new(77);
        let mut pooped = false;
        let mut landed = false;
        for tick in 0..10 {
            // The poop gate needs ai[0] == 1 (the landing tick zeroes it):
            // ai[1] starts 295, so tick 4 is the 300-tick landing — touch
            // down exactly there (the probe would re-add +0.1 otherwise).
            if tick == 4 {
                n.vy = 0.0;
            }
            let out = ai_style_68(&mut n, 609, &w, &c, &mut a);
            if out.poop_shot.is_some() {
                // luck 0 + Cenx: the roll short-circuits, NO draw.
                pooped = true;
            }
            if let Some((new_ty, ai1)) = out.transform {
                assert_eq!(new_ty, 608);
                assert_eq!(ai1, 200.0 + b.next_max(200) as f32);
                landed = true;
            }
            assert_eq!(a, b, "the streams stay aligned every tick");
            if landed {
                break;
            }
        }
        assert!(pooped, "the Cenx poop fired");
        assert!(landed, "the dry landing transformed");
    }
}
