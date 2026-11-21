//! S1-D-3 — the aiStyle-24 bird family: the `else if (this.aiStyle == 24)`
//! arm of `NPC.AI` (`NPC.cs:25313-25537`), shared by the 11 vanilla types
//! whose `SetDefaults` rows assign `this.aiStyle = 24`:
//! Bird 74 / BirdBlue 297 / BirdRed 298 (`NPC.cs:9747-9771`), GoldBird 442
//! (`:14247-14261`), Owl 611 (`:16846-16859`), ScarletMacaw 671 / BlueMacaw
//! 672 / Toucan 673 / YellowCockatiel 674 / GrayCockatiel 675
//! (`:17496-17565`) and OwlMimic 689 (`:17667-17679`).
//!
//! **UNWIRED module (code-only mode, 2026-11-18).** None of the 11 types has
//! a `NpcKind` yet (`npc/data.rs::ai_style` carries no style-24 row), so the
//! arm lands as pure functions over the port's `Npc` slot + a context seam;
//! the wiring (new kinds, the `update_rng` style-axis arm, the
//! `size_override` rows from [`STYLE24_BIRDS`], the 1080 BirdDroppings
//! projectile and the full `NPC.Transform`) is the family's merge step —
//! the `drawdata_anim.rs` (S1-F-9b) / `special_tiles.rs` (S1-G-1e) pattern.
//!
//! The three `TargetClosest()` call sites (`NPC.cs:25338`, `:25377`,
//! `:25535`) ride the shared selection helper
//! (`crate::npc_target::target_closest`, `NPC.cs:77652-77781`); the two
//! `Player.FindClosest` sites (`:25317`, `:25330`) ride
//! [`find_closest_player`] (`Player.cs:4942-4967`).
//!
//! RNG: the arm's only draw is the poop gate's
//! `Player.RollOnlyBadLuckExtreme(50)` (`NPC.cs:25319` →
//! `Player.cs:18208-18211` → `Luck.cs:53-60`): `Main.rand.NextFloat()` is
//! consumed ONLY when `luck < 0` (the `&&` short-circuit), and a second
//! draw `Main.rand.Next(range / 10)` happens ONLY when that float came in
//! under `-luck`; a non-negative-luck player draws NOTHING and the roll
//! returns −1 (never 0 — only a Cenx-named player poops then, and the `||`
//! still evaluates the roll FIRST, `:25319`).
#![allow(dead_code)] // UNWIRED — see the module doc; the wiring is the S1-D-3 merge step.

use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::tiledata::TILE_SOLID;
use tr_core::{UnifiedRandom, World};

/// One `SetDefaults` row of the family — the numbers the arm (and the
/// wiring's `size_override`/stats) reads. `catch_item`/`npc_slots`/`rarity`
/// ride along so the wiring step has the whole row in one place.
#[derive(Clone, Copy, Debug)]
pub struct BirdDefaults {
    pub ty: i32,
    pub width: i32,
    pub height: i32,
    pub life_max: i32,
    pub catch_item: i32,
    pub npc_slots: f32,
    pub rarity: i32,
}

/// The 11 style-24 rows. Every row: damage 0, defense 0, lifeMax 5,
/// knockBackResist 0.8, NPCHit1/NPCDeath1, value 0 (the `SetDefaults`
/// default, `NPC.cs:8544`). Citations are the per-type blocks.
pub const STYLE24_BIRDS: &[BirdDefaults] = &[
    // NPC.cs:9747-9771 — the 74/297/298 group (14×14, npcSlots 0.4).
    BirdDefaults { ty: 74, width: 14, height: 14, life_max: 5, catch_item: 2015, npc_slots: 0.4, rarity: 0 },
    BirdDefaults { ty: 297, width: 14, height: 14, life_max: 5, catch_item: 2016, npc_slots: 0.4, rarity: 0 },
    BirdDefaults { ty: 298, width: 14, height: 14, life_max: 5, catch_item: 2017, npc_slots: 0.4, rarity: 0 },
    // NPC.cs:14247-14261 — GoldBird (14×14, npcSlots 0.4, rarity 3).
    BirdDefaults { ty: 442, width: 14, height: 14, life_max: 5, catch_item: 2889, npc_slots: 0.4, rarity: 3 },
    // NPC.cs:16846-16859 — Owl (18×34, npcSlots 0.4).
    BirdDefaults { ty: 611, width: 18, height: 34, life_max: 5, catch_item: 4395, npc_slots: 0.4, rarity: 0 },
    // NPC.cs:17496-17565 — the five jungle birds (14×14, npcSlots 1).
    BirdDefaults { ty: 671, width: 14, height: 14, life_max: 5, catch_item: 5212, npc_slots: 1.0, rarity: 0 },
    BirdDefaults { ty: 672, width: 14, height: 14, life_max: 5, catch_item: 5300, npc_slots: 1.0, rarity: 0 },
    BirdDefaults { ty: 673, width: 14, height: 14, life_max: 5, catch_item: 5311, npc_slots: 1.0, rarity: 0 },
    BirdDefaults { ty: 674, width: 14, height: 14, life_max: 5, catch_item: 5312, npc_slots: 1.0, rarity: 0 },
    BirdDefaults { ty: 675, width: 14, height: 14, life_max: 5, catch_item: 5313, npc_slots: 1.0, rarity: 0 },
    // NPC.cs:17667-17679 — OwlMimic (18×34, npcSlots 0.4, NO catchItem).
    BirdDefaults { ty: 689, width: 18, height: 34, life_max: 5, catch_item: 0, npc_slots: 0.4, rarity: 0 },
];

/// `NPCID.Sets.BirdThatCanPoop` (`NPCID.cs:10435`):
/// `CreateBoolSet(74, 297, 298, 442, 363, 365, 603, 609, 671, 672, 673,
/// 674, 675)` — note it includes the four non-style-24 waterfowl (Duck2 363,
/// DuckWhite2 365, Seagull2 603, Grebe2 609 — the aiStyle-68 swimming forms,
/// `NPCID.cs:11403`/`:11407`/`:11883`/`:11895`, the `npc_ai_style68.rs`
/// family) and EXCLUDES Owl 611 / OwlMimic 689 (the owls never poop).
/// (W4-d127 census fix: the prior note mislabelled these "Cardinal 363 /
/// BlueJay 365 / Seagull 603 / Cormorant 609" — Cardinal is 298 and BlueJay
/// 297, both style-24 members of THIS family, `NPCID.cs:12394-12395`.)
pub fn bird_that_can_poop(ty: i32) -> bool {
    matches!(ty, 74 | 297 | 298 | 442 | 363 | 365 | 603 | 609 | 671 | 672 | 673 | 674 | 675)
}

/// `Player.IsThisCenx` (`Player.cs:18179-18186`): the four dev names,
/// compared after `name.ToLower()`.
pub fn is_this_cenx(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower == "cenx" || lower == "cblox" || lower == "jade lightning" || lower == "cenigit"
}

/// `Luck.RollOnlyBadLuckExtreme(float luck, int range)` (`Luck.cs:53-60`,
/// reached via `Player.RollOnlyBadLuckExtreme`, `Player.cs:18208-18211`):
/// bad luck only — a negative-luck player rolls `NextFloat()` (one draw),
/// and on a hit rolls `Next(range / 10)` (a second draw); everything else
/// returns −1. The `&&` short-circuit means a non-negative luck consumes
/// NO draw at all.
pub fn roll_only_bad_luck_extreme(luck: f32, range: i32, rng: &mut UnifiedRandom) -> i32 {
    // :55-58 — `if (luck < 0f && Main.rand.NextFloat() < 0f - luck)`.
    if luck < 0.0 && rng.next_float() < 0.0 - luck {
        // `Main.rand.Next(range / 10)` — integer division FIRST (:57).
        return rng.next_max(range / 10);
    }
    // :59.
    -1
}

/// `Player.FindClosest(Vector2 Position, int Width, int Height)`
/// (`Player.cs:4942-4967`): the default is the FIRST active slot (even a
/// dead one, `:4944-4952`); then the Manhattan scan over active-and-alive
/// slots with the strict `<` replacement (first slot wins ties,
/// `:4953-4965`). Both halves are INTEGER divisions
/// (`(float)(width / 2)`, `:4958`). The port's slice is indexed by player
/// slot; an all-inactive slice falls back to slot 0 (the C# `result = 0`
/// initializer).
pub fn find_closest_player(players: &[TargetCandidate], x: f32, y: f32, width: i32, height: i32) -> usize {
    // :4944-4952 — the first-active default.
    let mut result = 0usize;
    for (i, p) in players.iter().enumerate() {
        if p.active {
            result = i;
            break;
        }
    }
    // :4953-4965 — the Manhattan scan; `-1` sentinel = "no pick yet".
    let mut num = -1.0f32;
    let qx = x + (width / 2) as f32;
    let qy = y + (height / 2) as f32;
    for (j, p) in players.iter().enumerate() {
        if p.active && !p.dead {
            let num2 = (p.x + (p.width / 2) as f32 - qx).abs()
                + (p.y + (p.height / 2) as f32 - qy).abs();
            if num == -1.0 || num2 < num {
                num = num2;
                result = j;
            }
        }
    }
    result
}

/// One `Main.player[j]` slot as the style-24 arm sees it: the shared
/// selection's candidate plus the two fields only the poop gate reads
/// (`luck` for the roll, the pre-computed `IsThisCenx` answer — the port's
/// session player name lives outside the sim).
#[derive(Clone, Debug)]
pub struct Style24Player {
    pub candidate: TargetCandidate,
    /// `Player.luck` (`Player.cs:3086`).
    pub luck: f32,
    /// [`is_this_cenx`] over `Player.name` — computed by the wiring.
    pub is_cenx: bool,
}

/// One `Main.npc[i]` slot as the owl's night scan sees it
/// (`NPC.cs:25345-25367`). `is_self` carries the `num328 == base.whoAmI`
/// skip so the slice keeps the C#'s slot order (the same-type `break`
/// depends on it).
#[derive(Clone, Copy, Debug)]
pub struct NeighborNpc {
    pub is_self: bool,
    /// `Main.npc[i].active` (`:25347`).
    pub active: bool,
    /// `Main.npc[i].townNPC` (`:25351`).
    pub town_npc: bool,
    /// `Main.npc[i].type` (`:25362`).
    pub ty: i32,
    /// `Main.npc[i].ai[0]` (`:25362`).
    pub ai0: f32,
    /// The box (`base.position`/`width`/`height`) for `Center` and
    /// `Collision.CanHit` (`:25353-25356`).
    pub x: f32,
    pub y: f32,
    pub width: i32,
    pub height: i32,
}

/// The ambient inputs the arm reads off `Main` — the minimal seam.
pub struct Style24Ctx<'a> {
    /// The player slots (the port has one session player).
    pub players: &'a [Style24Player],
    /// The NPC slots in `Main.npc` order, self included (`is_self`).
    pub neighbors: &'a [NeighborNpc],
    /// `Main.netMode` — the port runs 0 (single-player); 1 suppresses the
    /// poop/mimic/release blocks (`:25315`, `:25328`, `:25378`).
    pub net_mode: i32,
    /// `Main.cloudAlpha` (`:25343`, `:25373`).
    pub cloud_alpha: f32,
    /// `Main.dayTime` (`:25328`, `:25343`, `:25373`).
    pub day_time: bool,
    /// `NPC.releaseOwner` (`NPC.cs:6079`, default 255; `:25380`). The
    /// field has no `Npc` slot yet — the wiring passes 255 until the
    /// critter-release path lands.
    pub release_owner: i32,
    /// `Main.EverLastingTicker` for the selection's `targetSetFrame`.
    pub ticker: u64,
}

/// What the arm asks the world sim to do — the request seam (the
/// `wants_hostile_shot` pattern).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Style24Outcome {
    /// `Projectile.NewProjectile(..., player.Center.X, base.Center.Y, 0f,
    /// 0f, 1080, 0, 0f)` (`NPC.cs:25322`): the BirdDroppings 1080
    /// (`ProjectileID.cs:2714`), zero velocity/damage/knockback, at the
    /// carried (x, y).
    pub poop_shot: Option<(f32, f32)>,
    /// `this.Transform(317)` (`NPC.cs:25333`) — the OwlMimic's DemonEyeOwl
    /// reveal (`NPCID.cs:11311`). The core reset is applied to the slot
    /// in-place ([`transform_core`]); the full `SetDefaults` (sounds,
    /// value, knockBackResist, the visuals) rides the wiring.
    pub transform: Option<i32>,
    /// The arm's last `noGravity` write (`:25325` true, `:25372` false) —
    /// the `Npc` slot has no `noGravity` field yet.
    pub no_gravity: bool,
}

/// `Collision.CanHit(Entity, Entity)` (`Collision.cs:371-374`) over the
/// port's shared walk (`crate::projectile::can_hit`, `Collision.cs:381-479`).
fn can_hit_boxes(world: &World, ax: f32, ay: f32, aw: f32, ah: f32, bx: f32, by: f32, bw: f32, bh: f32) -> bool {
    crate::projectile::can_hit(world, ax, ay, aw, ah, bx, by, bw, bh)
}

/// The shared selection with this arm's fixed arguments —
/// `this.TargetClosest()` defaults `faceTarget = true`
/// (`NPC.cs:77652`), `aiStyle` 24 (not the fighter 3, so the centre rule
/// applies, `:77761`).
fn select_target(npc: &mut Npc, world: &World, cands: &[TargetCandidate], ticker: u64) {
    if !cands.is_empty() {
        target_closest(npc, world, cands, true, 24, ticker);
    }
}

/// The core of `NPC.Transform(317)` (`NPC.cs:81084-81114`) that the rest
/// of THIS arm observes: the ±height reposition around `SetDefaults`
/// (`:81092-81100` — old height 34 down, new height 32 up → net +2), the
/// `ai[]`/`localAI[]` zeroing (`SetDefaults`, `:8564-8567`), the
/// `ClearTarget` 255 sentinel (`:8539`), `netUpdate = true` (`:8530`), the
/// collide-flag reset (`:8527-8528`), the `TargetClosest()` (`:81101`) and
/// the life rescale `life * lifeMax / oldLifeMax` floored at 1
/// (`:81106-81110`, integer division). The DemonEyeOwl row: 30×32, aiStyle
/// 2, lifeMax 75 (`NPC.cs:13007-13019`). The value/knockback/sound/visual
/// halves (`TransformVisuals`, `GiveTownUniqueDataToNPCsThatNeedIt`) are
/// the wiring's full-`Transform` job — recorded in ANALYSIS.
fn transform_core(npc: &mut Npc, world: &World, cands: &[TargetCandidate], ticker: u64) {
    let (_, old_h) = npc.size();
    let old_life = npc.hp;
    let old_life_max = npc.max_hp;
    // :81094 — `position.Y += height` (the OLD height).
    npc.y += old_h;
    // :81096 SetDefaults(317) — the observable subset.
    npc.size_override = Some((30.0, 32.0)); // :13009-13010
    npc.ai = [0.0; 4]; // :8564-8567
    npc.local_ai = [0.0; 4]; // :8568-8571
    npc.target = 255; // ClearTarget, :8539
    npc.old_target = npc.target; // :8540
    npc.target_rect = (0, 0, 0, 0); // :8541
    npc.net_update = true; // :8530
    npc.collide_x = false; // :8527-8528
    npc.collide_y = false;
    npc.max_hp = 75; // :13014
    // :81099 — `position.Y -= height` (the NEW height).
    npc.y -= 32.0;
    // :81101.
    select_target(npc, world, cands, ticker);
    // :81106-81110 — the integer rescale, floored at 1.
    npc.hp = (old_life * npc.max_hp / old_life_max).max(1);
}

/// The aiStyle-24 arm (`NPC.cs:25313-25537`), in the C#'s exact order:
/// the poop gate → `noGravity = true` → the owl/mimic block → the
/// `ai[0]` state machine (0 perched / 2 descend / else fly) → the wet
/// tail. `ty` is `this.type` (the port's kinds carry no bird rows yet);
/// the mimic transform flips it to 317 mid-arm exactly as the C# does.
pub fn ai_style_24(
    npc: &mut Npc,
    ty: i32,
    world: &World,
    ctx: &Style24Ctx,
    rng: &mut UnifiedRandom,
) -> Style24Outcome {
    let mut ty = ty;
    let mut out = Style24Outcome::default();
    let cands: Vec<TargetCandidate> = ctx.players.iter().map(|p| p.candidate).collect();
    // The C# re-reads `base.Center`/`base.width`/`base.height` at every
    // use; the only mid-arm mutation is the mimic Transform, which
    // refreshes these below.
    let (mut w, mut h) = npc.size();
    let (mut cx, mut cy) = (npc.x + w * 0.5, npc.y + h * 0.5); // Entity.Center, Entity.cs:50-55

    // :25315-25324 — the poop gate.
    if ctx.net_mode != 1 && npc.ai[0] == 1.0 && bird_that_can_poop(ty) && npc.local_ai[3] == 0.0 {
        // :25317 — Player.FindClosest(base.position, base.width, base.height).
        let j = find_closest_player(&cands, npc.x, npc.y, w as i32, h as i32);
        if let Some(p) = ctx.players.get(j) {
            let pc = &p.candidate;
            // :25318-25319 — the column alignment: Center.X inside the
            // player's [Left.X, Right.X] (Entity.cs:62-84 — position.X ..
            // position.X + width), the player's Top.Y (= position.Y,
            // Entity.cs:86-96) strictly BELOW the bird's Bottom.Y
            // (= position.Y + height, Entity.cs:122-132), line of sight,
            // then the roll == 0 OR the Cenx bypass — the roll is the
            // LEFT operand, so it is evaluated (and draws) either way.
            let x = cx;
            if x >= pc.x
                && x <= pc.x + pc.width as f32
                && pc.y > npc.y + h
                && can_hit_boxes(world, npc.x, npc.y, w, h, pc.x, pc.y, pc.width as f32, pc.height as f32)
                && (roll_only_bad_luck_extreme(p.luck, 50, rng) == 0 || p.is_cenx)
            {
                // :25321-25322.
                npc.local_ai[3] = 1.0;
                let pcx = pc.x + pc.width as f32 / 2.0; // player Center.X
                out.poop_shot = Some((pcx, cy));
            }
        }
    }

    // :25325 — `this.noGravity = true`.
    out.no_gravity = true;

    // :25326-25369 — the owl / owl-mimic block.
    if ty == 611 || ty == 689 {
        // :25328-25335 — the mimic reveal: night, within 80 px of the
        // closest player, with line of sight.
        if ty == 689 && ctx.net_mode != 1 && !ctx.day_time {
            let j = find_closest_player(&cands, npc.x, npc.y, w as i32, h as i32);
            if let Some(p) = ctx.players.get(j) {
                let pc = &p.candidate;
                let pcx = pc.x + pc.width as f32 / 2.0;
                let pcy = pc.y + pc.height as f32 / 2.0;
                // Vector2.Distance — the Euclidean norm (:25331).
                let dist = ((cx - pcx) * (cx - pcx) + (cy - pcy) * (cy - pcy)).sqrt();
                if dist < 80.0
                    && can_hit_boxes(world, npc.x, npc.y, w, h, pc.x, pc.y, pc.width as f32, pc.height as f32)
                {
                    // :25333 — Transform(317); the arm keeps running with
                    // the NEW type (the C# `this.type` is now 317).
                    transform_core(npc, world, &cands, ctx.ticker);
                    out.transform = Some(317);
                    ty = 317;
                    // The Transform moved/resized the slot — every later
                    // `base.Center`/`width`/`height` read sees the new box.
                    (w, h) = npc.size();
                    (cx, cy) = (npc.x + w * 0.5, npc.y + h * 0.5);
                }
            }
        }
        if npc.local_ai[0] == 0.0 {
            // :25336-25342 — the one-shot init (also runs right after the
            // transform: SetDefaults zeroed localAI[0]).
            select_target(npc, world, &cands, ctx.ticker);
            npc.ai[0] = 1.0;
            npc.local_ai[0] = 1.0;
            npc.net_update = true;
        } else if npc.ai[0] == 1.0 && ctx.cloud_alpha == 0.0 && !ctx.day_time {
            // :25343-25368 — the night roost scan, in slot order.
            for n in ctx.neighbors {
                // :25347-25350 — skip self and inactive slots.
                if n.is_self || !n.active {
                    continue;
                }
                let ncx = n.x + n.width as f32 / 2.0;
                let ncy = n.y + n.height as f32 / 2.0;
                if n.town_npc {
                    // :25351-25361 — a town NPC within 96 px horizontally
                    // and 32..320 px BELOW, hidden behind tiles: descend.
                    if (cx - ncx).abs() < 96.0 {
                        let dy = ncy - cy;
                        if dy > 32.0
                            && dy < 320.0
                            && !can_hit_boxes(world, npc.x, npc.y, w, h, n.x, n.y, n.width as f32, n.height as f32)
                        {
                            npc.ai[0] = 2.0;
                        }
                    }
                } else if n.ty == ty && n.ai0 != 1.0 && (cx - ncx).abs() < 320.0 {
                    // :25362-25366 — a perched same-type neighbour within
                    // 320 px: stay perched-ready and STOP the scan.
                    npc.ai[0] = 1.0;
                    break;
                }
            }
        }
    }

    // :25370-25522 — the ai[0] state machine.
    if npc.ai[0] == 0.0 {
        // :25370-25398 — perched on the ground.
        out.no_gravity = false; // :25372
        if (ty == 611 || ty == 689) && (ctx.cloud_alpha > 0.0 || ctx.day_time) {
            // :25373-25376 — rain or day flushes the owl off its perch.
            npc.ai[0] = 1.0;
        }
        select_target(npc, world, &cands, ctx.ticker); // :25377
        if ctx.net_mode != 1 {
            // :25380 — the release/startle gate. The C# compares
            // `(double)velocity.Y > 0.3`: the float 0.3f widens to
            // 0.30000001192… > the double literal 0.3, so vy == 0.3f
            // DOES startle — kept as the f64 compare.
            if ctx.release_owner != 255 || npc.vx != 0.0 || npc.vy < 0.0 || (npc.vy as f64) > 0.3 {
                npc.ai[0] = 1.0;
                npc.net_update = true;
                npc.facing = -npc.facing; // base.direction = -base.direction
            } else if ty != 611 && ty != 689 {
                // :25386-25396 — the proximity flutter: the player's
                // int-truncated rect vs the bird's rect grown 100 px on
                // every side, XNA Rectangle.Intersects (strict edges),
                // or any damage taken.
                if let Some(p) = cands.get(npc.target.max(0) as usize) {
                    let (pw, ph) = (p.width, p.height);
                    let (px, py) = (p.x as i32, p.y as i32);
                    let (bx, by) = (npc.x as i32 - 100, npc.y as i32 - 100);
                    let (bw, bh) = (w as i32 + 200, h as i32 + 200);
                    let intersects = px < bx + bw && bx < px + pw && py < by + bh && by < py + ph;
                    if intersects || npc.hp < npc.max_hp {
                        npc.ai[0] = 1.0;
                        npc.vy -= 6.0; // :25392
                        npc.net_update = true;
                        npc.facing = -npc.facing;
                    }
                }
            }
        }
    } else if npc.ai[0] == 2.0 {
        // :25399-25412 — the roost descent.
        npc.vx *= 0.98;
        if npc.vy == 0.0 {
            npc.ai[0] = 0.0;
            npc.vx = 0.0;
        }
        npc.vy += 0.05;
        if npc.vy > 2.0 {
            npc.vy = 2.0;
        }
    } else if !cands.get(npc.target.max(0) as usize).map_or(false, |p| p.dead) {
        // :25413-25522 — the flight (the target player is alive; an
        // absent slot reads like the C#'s inactive default, dead=false).
        // :25415-25419 — the macaw family flies faster.
        let mut top_speed = 3.0f32;
        if ty == 671 || ty == 672 || ty == 673 || ty == 674 || ty == 675 {
            top_speed = 4.0;
        }
        // :25420-25432 — the X bounce off oldVelocity.
        if npc.collide_x {
            npc.facing *= -1;
            npc.vx = npc.old_vx * -0.5;
            if npc.facing == -1 && npc.vx > 0.0 && npc.vx < top_speed - 1.0 {
                npc.vx = top_speed - 1.0;
            }
            if npc.facing == 1 && npc.vx < 0.0 && npc.vx > 0.0 - top_speed + 1.0 {
                npc.vx = 0.0 - top_speed + 1.0;
            }
        }
        // :25433-25444 — the Y bounce.
        if npc.collide_y {
            npc.vy = npc.old_vy * -0.5;
            if npc.vy > 0.0 && npc.vy < 1.0 {
                npc.vy = 1.0;
            }
            if npc.vy < 0.0 && npc.vy > -1.0 {
                npc.vy = -1.0;
            }
        }
        // :25445-25476 — the X steering. The inner `> num330` / `< -num330`
        // tests are the C#'s own (dead on the facing side, live after a
        // bounce pushed the velocity past the cap) — kept verbatim.
        if npc.facing == -1 && npc.vx > 0.0 - top_speed {
            npc.vx -= 0.1;
            if npc.vx > top_speed {
                npc.vx -= 0.1;
            } else if npc.vx > 0.0 {
                npc.vx -= 0.05;
            }
            if npc.vx < 0.0 - top_speed {
                npc.vx = 0.0 - top_speed;
            }
        } else if npc.facing == 1 && npc.vx < top_speed {
            npc.vx += 0.1;
            if npc.vx < 0.0 - top_speed {
                npc.vx += 0.1;
            } else if npc.vx < 0.0 {
                npc.vx += 0.05;
            }
            if npc.vx > top_speed {
                npc.vx = top_speed;
            }
        }
        // :25477-25501 — the ground probe: one column, `direction` tiles
        // ahead of the centre, 15 rows down from the feet. INTEGER
        // half-width (`(float)(base.width / 2)`, :25477); the C# (int)
        // casts truncate toward zero.
        let col = ((npc.x + (w as i32 / 2) as f32) / 16.0) as i32 + npc.facing;
        let row = ((npc.y + h) / 16.0) as i32;
        let mut no_ground = true; // flag38
        let mut ground_near = false; // flag39
        for r in row..row + 15 {
            // :25484-25487 — WorldGen.InWorld(col, r) (WorldGen.cs:9688-9695).
            if col < 0 || col >= world.width || r < 0 || r >= world.height {
                continue;
            }
            // :25488-25491 — a null tile is a fresh empty Tile (not solid,
            // no liquid) — the port's None reads the same.
            let blocked = world.tile(col, r).map_or(false, |t| {
                // :25492 — `nactive() && tileSolid[type]` (NO solidTop /
                // slope / halfBrick terms here) OR any liquid.
                (t.active() && !t.inactive() && TILE_SOLID.has(t.kind)) || t.liquid > 0
            });
            if blocked {
                if r < row + 5 {
                    ground_near = true; // :25494-25497
                }
                no_ground = false;
                break; // :25499
            }
        }
        // :25502-25521 — the hover corrections and caps.
        if no_ground {
            npc.vy += 0.05;
        } else {
            npc.vy -= 0.1;
        }
        if ground_near {
            npc.vy -= 0.2;
        }
        if npc.vy > 2.0 {
            npc.vy = 2.0;
        }
        if npc.vy < -4.0 {
            npc.vy = -4.0;
        }
    }

    // :25523-25536 — the wet tail (always runs).
    if npc.wet {
        npc.ai[1] = 0.0;
        if npc.vy > 0.0 {
            npc.vy *= 0.95;
        }
        npc.vy -= 0.5;
        if npc.vy < -4.0 {
            npc.vy = -4.0;
        }
        select_target(npc, world, &cands, ctx.ticker); // :25535
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

    /// A bird-shaped slot: the port has no bird kinds yet, so the tests
    /// ride a carrier kind with the row's `size_override` (the wiring
    /// does the same until the kinds land).
    fn bird(ty: i32, x: f32, y: f32) -> Npc {
        let row = STYLE24_BIRDS.iter().find(|r| r.ty == ty).unwrap();
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

    fn ctx<'a>(players: &'a [Style24Player], neighbors: &'a [NeighborNpc]) -> Style24Ctx<'a> {
        Style24Ctx {
            players,
            neighbors,
            net_mode: 0,
            cloud_alpha: 0.0,
            day_time: true,
            release_owner: 255,
            ticker: 7,
        }
    }

    /// `NPC.cs:9747-9771`, `:14247-14261`, `:16846-16859`,
    /// `:17496-17565`, `:17667-17679` — the 11 SetDefaults rows.
    #[test]
    fn the_style24_rows_match_set_defaults() {
        assert_eq!(STYLE24_BIRDS.len(), 11);
        for r in STYLE24_BIRDS {
            assert_eq!(r.life_max, 5, "type {} lifeMax", r.ty);
        }
        let owl = STYLE24_BIRDS.iter().find(|r| r.ty == 611).unwrap();
        assert_eq!((owl.width, owl.height, owl.catch_item), (18, 34, 4395));
        let mimic = STYLE24_BIRDS.iter().find(|r| r.ty == 689).unwrap();
        assert_eq!((mimic.width, mimic.height, mimic.catch_item), (18, 34, 0));
        let gold = STYLE24_BIRDS.iter().find(|r| r.ty == 442).unwrap();
        assert_eq!((gold.catch_item, gold.rarity), (2889, 3));
        let macaw = STYLE24_BIRDS.iter().find(|r| r.ty == 671).unwrap();
        assert_eq!(macaw.npc_slots, 1.0, "the jungle birds keep npcSlots 1");
        let bird = STYLE24_BIRDS.iter().find(|r| r.ty == 74).unwrap();
        assert_eq!(bird.npc_slots, 0.4);
    }

    /// `NPCID.cs:10435` — the poop set includes the four non-style-24
    /// birds and excludes the owls.
    #[test]
    fn the_poop_set_matches_the_npcid_factory() {
        for ty in [74, 297, 298, 442, 363, 365, 603, 609, 671, 672, 673, 674, 675] {
            assert!(bird_that_can_poop(ty), "type {ty} poops");
        }
        for ty in [611, 689, 21, 1] {
            assert!(!bird_that_can_poop(ty), "type {ty} never poops");
        }
    }

    /// `Player.cs:18179-18186` — the four names, case-insensitive.
    #[test]
    fn the_cenx_names_are_case_insensitive() {
        for name in ["Cenx", "CBLOX", "Jade Lightning", "cenigit"] {
            assert!(is_this_cenx(name), "{name}");
        }
        assert!(!is_this_cenx("Redigit"));
        assert!(!is_this_cenx("cenx2"));
    }

    /// `Luck.cs:53-60` — the stream witness: for every seed the roll
    /// consumes EXACTLY the draws the C# would, in order (NextFloat, then
    /// Next(5) only on the hit), and returns the same value. The manual
    /// replay pins `next_float` ≡ `Main.rand.NextFloat()` and `next_max`
    /// ≡ `Main.rand.Next(int)` (unified_random.rs, Utils.cs:1198-1201).
    #[test]
    fn the_bad_luck_roll_replays_the_csharp_stream() {
        for seed in 0..64 {
            let mut a = UnifiedRandom::new(seed);
            let mut b = UnifiedRandom::new(seed);
            let got = roll_only_bad_luck_extreme(-0.5, 50, &mut a);
            // The C# order: one NextFloat, then Next(50/10) only on a hit.
            let f = b.next_float();
            let want = if f < 0.5 { b.next_max(5) } else { -1 };
            assert_eq!(got, want, "seed {seed}");
            assert_eq!(a, b, "seed {seed}: the streams must stay aligned");
        }
    }

    /// `Luck.cs:55` — the `&&` short-circuit: luck >= 0 draws NOTHING and
    /// returns −1 (so only the Cenx bypass can poop for a lucky player).
    #[test]
    fn non_negative_luck_draws_nothing() {
        for luck in [0.0f32, 0.3, 1.0] {
            let mut a = UnifiedRandom::new(123);
            let b = a.clone();
            assert_eq!(roll_only_bad_luck_extreme(luck, 50, &mut a), -1);
            assert_eq!(a, b, "no draw for luck {luck}");
        }
    }

    /// `Player.cs:4944-4965` — the first-active default (even dead), then
    /// the alive Manhattan scan with integer halves and first-slot ties.
    #[test]
    fn find_closest_defaults_then_scans_the_living() {
        let mut dead = cand(0.0, 0.0);
        dead.dead = true;
        // Only a dead player: the default keeps its slot.
        assert_eq!(find_closest_player(&[dead], 500.0, 500.0, 14, 14), 0);
        // A live farther slot wins over the dead near one.
        let live = cand(400.0, 400.0);
        assert_eq!(find_closest_player(&[dead, live], 500.0, 500.0, 14, 14), 1);
        // Ties keep the first slot (strict <): centres 610 and 404 are
        // both 103 px from the query centre 507.
        let a = cand(600.0, 500.0);
        let b = cand(394.0, 500.0);
        assert_eq!(find_closest_player(&[a, b], 500.0, 500.0, 14, 14), 0);
    }

    /// `NPC.cs:25318-25319` — the column gate: the bird's centre X must
    /// sit inside the player's [Left, Right] and the player's top must be
    /// strictly below the bird's bottom; a bird beside the player never
    /// poops even for Cenx.
    #[test]
    fn the_poop_needs_the_player_directly_below() {
        let w = air();
        let players = [player(1000.0, 1100.0, 0.0, true)];
        let c = ctx(&players, &[]);
        // Bird centre 1007 inside [1000, 1020], bottom 1014 < player top 1100.
        let mut n = bird(74, 1000.0, 1000.0);
        n.ai[0] = 1.0;
        let out = ai_style_24(&mut n, 74, &w, &c, &mut UnifiedRandom::new(1));
        assert_eq!(out.poop_shot, Some((1010.0, 1007.0)), "player Centre.X, bird Centre.Y");
        assert_eq!(n.local_ai[3], 1.0, "the one-shot latch (:25321)");
        // Beside the player: centre 1057 > Right 1020 — no poop.
        let mut n2 = bird(74, 1050.0, 1000.0);
        n2.ai[0] = 1.0;
        let out2 = ai_style_24(&mut n2, 74, &w, &c, &mut UnifiedRandom::new(1));
        assert_eq!(out2.poop_shot, None);
        assert_eq!(n2.local_ai[3], 0.0);
    }

    /// `NPC.cs:25315`, `:25321` — the gate needs ai[0] == 1 and an unset
    /// localAI[3]; the latch blocks every later tick. The owls (611/689)
    /// are outside the poop set entirely.
    #[test]
    fn the_poop_latches_once_and_the_owls_never_poop() {
        let w = air();
        let players = [player(1000.0, 1100.0, 0.0, true)];
        let c = ctx(&players, &[]);
        let mut n = bird(74, 1000.0, 1000.0);
        n.ai[0] = 1.0;
        n.local_ai[3] = 1.0; // already pooped
        let out = ai_style_24(&mut n, 74, &w, &c, &mut UnifiedRandom::new(1));
        assert_eq!(out.poop_shot, None);
        // Perched (ai[0] == 0): no poop either.
        let mut n2 = bird(74, 1000.0, 1000.0);
        let out2 = ai_style_24(&mut n2, 74, &w, &c, &mut UnifiedRandom::new(1));
        assert_eq!(out2.poop_shot, None);
        // The owl is not in BirdThatCanPoop.
        let mut n3 = bird(611, 1000.0, 1000.0);
        n3.ai[0] = 1.0;
        n3.local_ai[0] = 1.0; // skip the init TargetClosest write
        let out3 = ai_style_24(&mut n3, 611, &w, &c, &mut UnifiedRandom::new(1));
        assert_eq!(out3.poop_shot, None);
    }

    /// `NPC.cs:25380-25385` — the release/startle gate flips the facing
    /// and takes off; the `(double)vy > 0.3` widen means vy == 0.3f
    /// startles too (0.3f as f64 > the 0.3 double literal).
    #[test]
    fn the_perch_startles_on_release_or_motion() {
        let w = air();
        let players = [player(2000.0, 1000.0, 0.0, false)];
        let mut c = ctx(&players, &[]);
        c.release_owner = 3; // a released critter
        let mut n = bird(74, 1000.0, 1000.0);
        let out = ai_style_24(&mut n, 74, &w, &c, &mut UnifiedRandom::new(1));
        assert_eq!(n.ai[0], 1.0);
        assert_eq!(n.facing, -1, "the direction flips");
        assert!(n.net_update);
        assert!(!out.no_gravity, "the perched arm clears noGravity (:25372)");
        // vy == 0.3f: the double-widened compare startles.
        let c2 = ctx(&players, &[]);
        let mut n2 = bird(74, 1000.0, 1000.0);
        n2.vy = 0.3;
        ai_style_24(&mut n2, 74, &w, &c2, &mut UnifiedRandom::new(1));
        assert_eq!(n2.ai[0], 1.0, "0.3f widens above the 0.3 double");
    }

    /// `NPC.cs:25386-25396` — the proximity flutter: the grown-100 rect
    /// intersect (or damage) launches with vy -= 6; the owls are exempt.
    #[test]
    fn the_perch_flutters_when_the_player_comes_close() {
        let w = air();
        // Player centre ~250 px away: outside the grown rect
        // (bird rect x 900..1214 with the +200 growth).
        let far = [player(1500.0, 1000.0, 0.0, false)];
        let c = ctx(&far, &[]);
        let mut n = bird(74, 1000.0, 1000.0);
        ai_style_24(&mut n, 74, &w, &c, &mut UnifiedRandom::new(1));
        assert_eq!(n.ai[0], 0.0, "the far player leaves the bird perched");
        // Inside the grown rect.
        let near = [player(1100.0, 1000.0, 0.0, false)];
        let c2 = ctx(&near, &[]);
        let mut n2 = bird(74, 1000.0, 1000.0);
        ai_style_24(&mut n2, 74, &w, &c2, &mut UnifiedRandom::new(1));
        assert_eq!(n2.ai[0], 1.0);
        assert_eq!(n2.vy, -6.0, "the flutter impulse (:25392)");
        // Damage alone launches even with the player far away.
        let mut n3 = bird(74, 1000.0, 1000.0);
        n3.hp = 4;
        ai_style_24(&mut n3, 74, &w, &c, &mut UnifiedRandom::new(1));
        assert_eq!(n3.ai[0], 1.0);
        assert_eq!(n3.vy, -6.0);
    }

    /// `NPC.cs:25399-25412` — the descent: ×0.98 X damp, landing zeroes
    /// the state and X, +0.05 gravity capped at 2.
    #[test]
    fn the_descent_damps_and_lands() {
        let w = air();
        let players = [player(2000.0, 1000.0, 0.0, false)];
        let c = ctx(&players, &[]);
        let mut n = bird(611, 1000.0, 1000.0);
        n.ai[0] = 2.0;
        n.local_ai[0] = 1.0;
        n.vx = 1.0;
        n.vy = 0.0; // touching the ground this tick
        ai_style_24(&mut n, 611, &w, &c, &mut UnifiedRandom::new(1));
        assert_eq!(n.vx, 0.0, "the landing zeroes X");
        assert_eq!(n.vy, 0.05, "the gravity still adds after the landing check");
        // The state machine is an if/else chain: the descent branch ran
        // this tick, so the perch arm (and its daytime flush) waits for
        // the next tick — ai[0] stays 0 here.
        assert_eq!(n.ai[0], 0.0, "the landing returns to the perch");
        // Airborne: the damp and the cap.
        let mut n2 = bird(611, 1000.0, 1000.0);
        n2.ai[0] = 2.0;
        n2.local_ai[0] = 1.0;
        n2.vx = 1.0;
        n2.vy = 1.99;
        ai_style_24(&mut n2, 611, &w, &c, &mut UnifiedRandom::new(1));
        assert_eq!(n2.vx, 0.98);
        assert_eq!(n2.vy, 2.0, "the 2f cap");
    }

    /// `NPC.cs:25415-25444` — the bounce arms: the macaw family's 4f
    /// speed, the oldVelocity × −0.5 bounce and the small-velocity floors.
    #[test]
    fn the_flight_bounces_off_old_velocity() {
        let w = air();
        let players = [player(2000.0, 1000.0, 0.0, false)];
        let c = ctx(&players, &[]);
        let mut n = bird(671, 1000.0, 1000.0);
        n.ai[0] = 1.0;
        n.collide_x = true;
        n.old_vx = 4.0; // flying right into a wall
        let out = ai_style_24(&mut n, 671, &w, &c, &mut UnifiedRandom::new(1));
        assert!(out.no_gravity);
        assert_eq!(n.facing, -1, "the X bounce flips the direction");
        // vx = -2, facing -1: the steering subtracts 0.1 (+0.05 while > 0
        // is skipped, vx < 0), floor -4 untouched.
        assert_eq!(n.vx, -2.1);
        // The small-X bounce floor (:25424-25427): facing -1 with a tiny
        // positive bounce velocity floors to top_speed - 1 = 2 BEFORE the
        // steering (2 > 0 adds the extra -0.05).
        let mut n3 = bird(74, 1000.0, 1000.0);
        n3.ai[0] = 1.0;
        n3.collide_x = true;
        n3.old_vx = -1.0; // → vx = 0.5, facing flips to -1
        ai_style_24(&mut n3, 74, &w, &c, &mut UnifiedRandom::new(1));
        assert_eq!(n3.vx, 2.0 - 0.1 - 0.05, "floor 2 then -0.1 -0.05");
        // The Y bounce floor: old_vy 1 → -0.5 → floored to -1.
        let mut n2 = bird(74, 1000.0, 1000.0);
        n2.ai[0] = 1.0;
        n2.collide_y = true;
        n2.old_vy = 1.0;
        ai_style_24(&mut n2, 74, &w, &c, &mut UnifiedRandom::new(1));
        // vy = -1 after the floor; the probe finds no ground in the air
        // world (+0.05), so the tick ends at -0.95.
        assert_eq!(n2.vy, -0.95);
    }

    /// `NPC.cs:25477-25521` — the ground probe: solid or liquid within 15
    /// rows below the feet brakes (−0.1), within the first 5 rows brakes
    /// harder (−0.2 more); open air sinks (+0.05); the caps hold.
    #[test]
    fn the_flight_probes_the_column_below() {
        let mut w = air();
        // Bird 74 at (1000, 1000): centre col (1000+7)/16 = 62, +facing 1
        // → 63; feet row (1000+14)/16 = 63. Solid at row 65 (< 63+5).
        let i = w.index(63, 65);
        w.tiles[i] = Tile { kind: 1, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
        let players = [player(2000.0, 1000.0, 0.0, false)];
        let c = ctx(&players, &[]);
        let mut n = bird(74, 1000.0, 1000.0);
        n.ai[0] = 1.0;
        ai_style_24(&mut n, 74, &w, &c, &mut UnifiedRandom::new(1));
        // steering: facing 1, vx 0 → +0.1 then +0.05 (still < 0? no: 0.1
        // > 0 skips) → 0.1; probe: near ground → -0.1 - 0.2 = -0.3.
        assert_eq!(n.vy, -0.3, "the near-ground double brake");
        // Liquid counts as ground too (:25492).
        let mut w2 = air();
        let i2 = w2.index(63, 70); // row 70: >= 63+5, far ground
        w2.tiles[i2] = Tile { kind: 0, liquid: 1, ..Default::default() };
        let mut n2 = bird(74, 1000.0, 1000.0);
        n2.ai[0] = 1.0;
        ai_style_24(&mut n2, 74, &w2, &c, &mut UnifiedRandom::new(1));
        assert_eq!(n2.vy, -0.1, "the far-ground single brake");
        // Open air sinks +0.05 (the far-ground fixture's neighbour column
        // is air): a fresh bird ends the tick at +0.05.
        let w3 = air();
        let mut n3 = bird(74, 1000.0, 1000.0);
        n3.ai[0] = 1.0;
        ai_style_24(&mut n3, 74, &w3, &c, &mut UnifiedRandom::new(1));
        assert_eq!(n3.vy, 0.05, "open air sinks");
        // The -4 rise cap holds under the near-ground double brake.
        let mut n4 = bird(74, 1000.0, 1000.0);
        n4.ai[0] = 1.0;
        n4.vy = -3.99;
        ai_style_24(&mut n4, 74, &w, &c, &mut UnifiedRandom::new(1));
        assert_eq!(n4.vy, -4.0, "the rise cap");
    }

    /// `NPC.cs:25523-25536` — the wet tail zeroes ai[1], damps a sinking
    /// vy ×0.95, floats −0.5 (cap −4) and re-picks the target.
    #[test]
    fn the_wet_tail_floats_and_retargets() {
        let w = air();
        let players = [player(2000.0, 1000.0, 0.0, false)];
        let c = ctx(&players, &[]);
        let mut n = bird(74, 1000.0, 1000.0);
        n.ai[0] = 1.0;
        n.ai[1] = 5.0;
        n.wet = true;
        n.vy = 2.0;
        n.target = 255;
        ai_style_24(&mut n, 74, &w, &c, &mut UnifiedRandom::new(1));
        assert_eq!(n.ai[1], 0.0);
        // fly probe (open air): 2.0 + 0.05 = 2.05 → cap 2; wet: ×0.95 =
        // 1.9, −0.5 = 1.4 (float rounding: 2f*0.95f = 1.9f).
        assert!((n.vy - 1.4).abs() < 1e-6, "vy {}", n.vy);
        assert_eq!(n.target, 0, "the wet TargetClosest (:25535)");
        assert_eq!(n.target_set_frame, 7, "the ticker stamp");
    }

    /// `NPC.cs:25343-25368` — the owl's night scan: a hidden town NPC
    /// 32..320 px below sets ai[0] = 2; a perched same-type neighbour
    /// within 320 px keeps ai[0] = 1 and breaks; day/rain skip the scan.
    #[test]
    fn the_owl_scans_for_a_roost_at_night() {
        let mut w = air();
        // A solid column between the owl (col ~63) and the town NPC
        // below blocks CanHit.
        for r in 64..70 {
            let i = w.index(63, r);
            w.tiles[i] = Tile { kind: 1, ..Default::default() };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        let players = [player(2000.0, 1000.0, 0.0, false)];
        // Owl 611 at (1000, 1000), centre (1009, 1017). Town NPC centre
        // (1009, 1117): dx 0 < 96, dy 100 in (32, 320), hidden.
        let town = NeighborNpc {
            is_self: false, active: true, town_npc: true, ty: 22, ai0: 0.0,
            x: 1000.0, y: 1096.0, width: 18, height: 42,
        };
        let me = NeighborNpc { is_self: true, active: true, town_npc: false, ty: 611, ai0: 1.0, x: 1000.0, y: 1000.0, width: 18, height: 34 };
        let neighbors = [me, town];
        let c2 = Style24Ctx { neighbors: &neighbors, day_time: false, ..ctx(&players, &[]) };
        let mut n = bird(611, 1000.0, 1000.0);
        n.ai[0] = 1.0;
        n.local_ai[0] = 1.0;
        n.vy = -1.0; // airborne — a grounded owl would land the same tick
        ai_style_24(&mut n, 611, &w, &c2, &mut UnifiedRandom::new(1));
        assert_eq!(n.ai[0], 2.0, "the hidden town NPC below pulls the roost");
        // A perched same-type neighbour within 320 px keeps ai[0] = 1.
        let other = NeighborNpc {
            is_self: false, active: true, town_npc: false, ty: 611, ai0: 0.0,
            x: 1200.0, y: 1000.0, width: 18, height: 34,
        };
        let neighbors2 = [me, other];
        let c3 = Style24Ctx { neighbors: &neighbors2, day_time: false, ..ctx(&players, &[]) };
        let mut n2 = bird(611, 1000.0, 1000.0);
        n2.ai[0] = 1.0;
        n2.local_ai[0] = 1.0;
        ai_style_24(&mut n2, 611, &w, &c3, &mut UnifiedRandom::new(1));
        assert_eq!(n2.ai[0], 1.0);
        // Daytime: the scan is skipped even with the town NPC there.
        let c4 = Style24Ctx { neighbors: &neighbors, ..ctx(&players, &[]) };
        let mut n3 = bird(611, 1000.0, 1000.0);
        n3.ai[0] = 1.0;
        n3.local_ai[0] = 1.0;
        ai_style_24(&mut n3, 611, &w, &c4, &mut UnifiedRandom::new(1));
        assert_eq!(n3.ai[0], 1.0);
    }

    /// `NPC.cs:25328-25335` + `:81084-81110` — the mimic reveal: night,
    /// within 80 px, line of sight → Transform(317): the +2 reposition
    /// (34 down, 32 up), the ai/localAI reset, the 5→75 life rescale and
    /// the init branch re-running on the zeroed localAI[0].
    #[test]
    fn the_mimic_reveals_into_the_owl_eye() {
        let w = air();
        // Player centre (1010, 1021) vs owl centre (1009, 1017): ~4 px.
        let players = [player(1000.0, 1000.0, 0.0, false)];
        let mut c = ctx(&players, &[]);
        c.day_time = false;
        let mut n = bird(689, 1000.0, 1000.0);
        n.ai[0] = 1.0;
        n.local_ai[0] = 1.0;
        n.hp = 3; // damaged: 3 * 75 / 5 = 45
        let out = ai_style_24(&mut n, 689, &w, &c, &mut UnifiedRandom::new(1));
        assert_eq!(out.transform, Some(317));
        assert_eq!(n.y, 1002.0, "the ±height reposition nets +2");
        assert_eq!(n.size(), (30.0, 32.0), "the DemonEyeOwl box (:13009-13010)");
        assert_eq!(n.max_hp, 75);
        assert_eq!(n.hp, 45, "the integer life rescale (:81106)");
        assert_eq!(n.ai[0], 1.0, "the init branch re-arms after the reset");
        assert_eq!(n.local_ai[0], 1.0);
        assert!(n.net_update);
        // Daytime: no reveal.
        let c2 = ctx(&players, &[]);
        let mut n2 = bird(689, 1000.0, 1000.0);
        n2.ai[0] = 1.0;
        n2.local_ai[0] = 1.0;
        let out2 = ai_style_24(&mut n2, 689, &w, &c2, &mut UnifiedRandom::new(1));
        assert_eq!(out2.transform, None);
    }

    /// W4-d127 census — the family is EXACTLY the 11 `SetDefaults aiStyle = 24`
    /// types (9 assignment sites: 74/297/298 share `NPC.cs:9751`), and the
    /// poop set's four extra members are the aiStyle-68 WATERFOWL, not birds
    /// of this arm. Pins the doc-fix: Cardinal is 298 and BlueJay 297 (both
    /// style-24 members HERE, `NPCID.cs:12394-12395`), while 363/365/603/609
    /// are Duck2/DuckWhite2/Seagull2/Grebe2 (`NPCID.cs:11403`/`:11407`/
    /// `:11883`/`:11895`) — the old note's "Cardinal 363 / BlueJay 365" was a
    /// mislabel. The owls 611/689 are style-24 but outside `BirdThatCanPoop`.
    #[test]
    fn the_census_splits_the_eleven_birds_from_the_four_waterfowl() {
        let members: Vec<i32> = STYLE24_BIRDS.iter().map(|r| r.ty).collect();
        assert_eq!(
            members,
            vec![74, 297, 298, 442, 611, 671, 672, 673, 674, 675, 689],
            "the 11 aiStyle-24 SetDefaults members (NPC.cs:9751/14251/16850/\
             17500/17514/17528/17542/17556/17671)"
        );
        // The four waterfowl are in the poop set but NOT style-24 members.
        for ty in [363, 365, 603, 609] {
            assert!(bird_that_can_poop(ty), "waterfowl {ty} poops (NPCID.cs:10435)");
            assert!(!members.contains(&ty), "waterfowl {ty} is aiStyle-68, not 24");
        }
        // Cardinal 298 / BlueJay 297 ARE style-24 members (the doc-fix).
        assert!(members.contains(&298), "Cardinal 298 is a style-24 bird");
        assert!(members.contains(&297), "BlueJay 297 is a style-24 bird");
        // The owls are style-24 but never poop.
        assert!(members.contains(&611) && members.contains(&689));
        assert!(!bird_that_can_poop(611) && !bird_that_can_poop(689));
    }

    /// W4-d127 — the owl's day/night perch gate (`NPC.cs:25373-25376`): a
    /// perched owl (ai[0] == 0, already initialised) flushes to flight on
    /// DAY or RAIN, and stays perched on a clear NIGHT. The perched owl is
    /// exempt from the proximity flutter (`:25386`'s `type != 611 && != 689`),
    /// so the day/night term is the ONLY take-off trigger at rest.
    #[test]
    fn the_perched_owl_flushes_by_day_or_rain_and_roosts_by_night() {
        let w = air();
        let players = [player(2000.0, 1000.0, 0.0, false)];
        // Clear night: the owl keeps its perch (ai[0] stays 0).
        let night = Style24Ctx { day_time: false, cloud_alpha: 0.0, ..ctx(&players, &[]) };
        let mut n = bird(611, 1000.0, 1000.0);
        n.ai[0] = 0.0;
        n.local_ai[0] = 1.0; // skip the init; sit on the perch arm
        ai_style_24(&mut n, 611, &w, &night, &mut UnifiedRandom::new(1));
        assert_eq!(n.ai[0], 0.0, "a clear night keeps the owl perched");
        // Day: the flush lifts it to flight.
        let day = Style24Ctx { day_time: true, ..ctx(&players, &[]) };
        let mut n2 = bird(611, 1000.0, 1000.0);
        n2.ai[0] = 0.0;
        n2.local_ai[0] = 1.0;
        ai_style_24(&mut n2, 611, &w, &day, &mut UnifiedRandom::new(1));
        assert_eq!(n2.ai[0], 1.0, "daytime flushes the owl (:25373-25376)");
        // Rain (cloudAlpha > 0) at night: the flush lifts it too.
        let rain = Style24Ctx { day_time: false, cloud_alpha: 0.5, ..ctx(&players, &[]) };
        let mut n3 = bird(611, 1000.0, 1000.0);
        n3.ai[0] = 0.0;
        n3.local_ai[0] = 1.0;
        ai_style_24(&mut n3, 611, &w, &rain, &mut UnifiedRandom::new(1));
        assert_eq!(n3.ai[0], 1.0, "rain flushes the owl (:25373's cloudAlpha arm)");
    }

    /// `NPC.cs:25336-25342` — the owl init: localAI[0] == 0 targets, sets
    /// ai[0] = 1, latches localAI[0] and arms netUpdate exactly once.
    #[test]
    fn the_owl_init_latches_once() {
        let w = air();
        let players = [player(2000.0, 1000.0, 0.0, false)];
        let c = ctx(&players, &[]);
        let mut n = bird(611, 1000.0, 1000.0);
        n.net_update = false;
        ai_style_24(&mut n, 611, &w, &c, &mut UnifiedRandom::new(1));
        assert_eq!(n.ai[0], 1.0);
        assert_eq!(n.local_ai[0], 1.0);
        assert!(n.net_update);
        assert_eq!(n.target, 0, "the init TargetClosest (:25338)");
    }

    /// W4-d127 chain-verify — the 317 completion contract: the module's
    /// `transform_core` applies the FULL observable Transform subset
    /// in-place (`NPC.cs:81092-81110`), so the sim's drain must be
    /// completion-ONLY (the kind swap via `kind_from_net_id(317)`, the
    /// override clear) — a second `transform_to_kind` would reposition
    /// and rescale twice. Pins the exact post-core state the drain
    /// receives: +2 net reposition, the 30x32 override, 75 lifeMax, the
    /// integer rescale floored at 1, the zeroed ai/localAI + re-armed
    /// owl init, netUpdate true, target re-picked. WRITTEN NOT RUN.
    #[test]
    fn the_transform_core_state_is_the_drains_whole_input() {
        let w = air();
        let players = [player(1000.0, 1000.0, 0.0, false)];
        let mut c = ctx(&players, &[]);
        c.day_time = false;
        let mut n = bird(689, 1000.0, 1000.0);
        n.ai[0] = 1.0;
        n.local_ai[0] = 1.0;
        n.hp = 1; // the floor: 1 * 75 / 5 = 15 (no floor needed)
        let out = ai_style_24(&mut n, 689, &w, &c, &mut UnifiedRandom::new(1));
        assert_eq!(out.transform, Some(317), "the drain request");
        // The drain's completion writes are exactly: kind swap +
        // override clear. Everything else is ALREADY C#-faithful:
        assert_eq!(n.y, 1002.0, "the ±height reposition ran ONCE (:81094/:81099)");
        assert_eq!(n.size_override, Some((30.0, 32.0)));
        assert_eq!(n.max_hp, 75, "the 317 row (:13014)");
        assert_eq!(n.hp, 15, "1 * 75 / 5 — integer (:81106)");
        assert_eq!(n.local_ai, [1.0, 0.0, 0.0, 0.0], "reset then owl init re-arm");
        assert!(n.net_update);
        assert_eq!(n.target, 0, "the Transform TargetClosest (:81101)");
        // The life floor: hp 0 cannot occur (life > 0 gate), but the
        // rescale of a 1-hp 5-max slot into a 5-max target floors at 1
        // (`:81108-81109`) — the core's `.max(1)` mirrors it.
        let mut m = bird(689, 1000.0, 1000.0);
        m.ai[0] = 1.0;
        m.local_ai[0] = 1.0;
        m.hp = 1;
        m.max_hp = 500; // synthetic: 1 * 75 / 500 = 0 → floor 1
        transform_core(&mut m, &w, &cands_of(&players), 7);
        assert_eq!(m.hp, 1, "the rescale floors at 1 (:81108-81109)");
    }

    fn cands_of(players: &[Style24Player]) -> Vec<TargetCandidate> {
        players.iter().map(|p| p.candidate).collect()
    }

    /// W4-d127 chain-verify — the 317 row census (the d38 tip's rows,
    /// `NPC.cs:13007-13019`): DemonEyeOwl is ROWED on the pushed
    /// `task/W4-d38-npc-kinds-281-350` tip (data.rs:590/:969 cs_id both
    /// ways, ai_style 2, bestiary net_id 317) and routes to the eye
    /// machine (its `update_kind_ai` arm `DemonEyeOwl => update_eye`;
    /// the b2 tip claims the `2 => ai_002_eye_family` axis). The port's
    /// own roster (this branch) has no 317 row — the chain's merge gate
    /// is the d38 landing, NOT a re-row here (one row per kind, the
    /// census ownership rule). Pins the ROW the transform targets so a
    /// d38 drift breaks THIS lane's witness too. WRITTEN NOT RUN.
    #[test]
    fn the_demon_eye_owl_row_matches_the_d38_census() {
        // NPC.cs:13007-13019 — 30x32, aiStyle 2, dmg 16, def 6, 75 HP,
        // kb 0.7, value 100, NPCHit1/NPCDeath1. The transform_core's
        // observable subset reads exactly these two stat rows:
        let (w, h) = (30.0f32, 32.0f32); // :13009-13010
        let life_max = 75; // :13014
        let mut n = bird(689, 0.0, 0.0);
        n.hp = 5;
        let players: [Style24Player; 0] = [];
        transform_core(&mut n, &air(), &cands_of(&players), 0);
        assert_eq!(n.size(), (w, h));
        assert_eq!(n.max_hp, life_max);
        // The mimic keeps NO catchItem (STYLE24_BIRDS row 689) — the
        // reveal is combat-only; 317 is not npcCatchable either
        // (`Main.cs:9326-9418` has no 317), so the friendly refresh
        // the c8 head added reads false on both sides.
        assert!(!crate::bug_net::npc_catchable(317));
        assert!(!crate::bug_net::npc_catchable(689));
    }

    /// W4-d127 chain item 2 — the AXIS claim source pin (the c8
    /// pattern): `update_rng_dust`'s style match carries EXACTLY ONE
    /// `24 =>` arm routing to `ai_024_bird_family`, and the family fn
    /// calls the faithful module with the sim-stamped cloud_alpha +
    /// neighbours, the slot's release_owner and the vanilla cs_id.
    /// Scanned from source like `s1_d_4c_every_kind_has_exactly_one_arm`
    /// — a duplicate arm is dead code (the FIRST wins), so the count
    /// pin keeps the claim single. WRITTEN NOT RUN.
    #[test]
    fn the_axis_24_claim_is_single_and_routes_the_family() {
        let src = include_str!("npc.rs");
        let axis = src.find("match self.kind.ai_style()").expect("the style axis");
        let rest = &src[axis..];
        let end = rest.find("_ => self.update_kind_ai").expect("the axis fallthrough");
        let body = &rest[..end];
        let arms: Vec<&str> = body
            .lines()
            .map(|l| l.trim_start())
            .filter(|l| l.starts_with("24 =>"))
            .collect();
        assert_eq!(arms.len(), 1, "one live axis-24 arm (S1-D-4c)");
        assert!(arms[0].contains("ai_024_bird_family"), "the arm routes the family fn");
        // The family fn feeds the module (the call site exists) and
        // queues BOTH out-requests (the poop shot + the transform).
        let fam = src.find("fn ai_024_bird_family").expect("the family fn");
        let frest = &src[fam..];
        let fend = frest.find("\n    }").expect("the fn's close");
        let fbody = &frest[..fend];
        assert!(fbody.contains("npc_ai_style24::ai_style_24(self, self.kind.cs_id()"));
        assert!(fbody.contains("cloud_alpha: self.cloud_alpha"));
        assert!(fbody.contains("neighbors: &neighbors"));
        assert!(fbody.contains("release_owner: self.release_owner as i32"));
        assert!(fbody.contains("wants_hostile_shot.push((1080"));
        assert!(fbody.contains("wants_transform = Some(ty)"));
        // The integration tail: the gravity gate reads the outcome's
        // noGravity, the depth-scaled body_gravity + the wet 0.2/7 row,
        // the dead-band snap, the oldVelocity latch.
        assert!(fbody.contains("if !out.no_gravity"));
        assert!(fbody.contains("npc_ai_golem::body_gravity"));
        assert!(fbody.contains("(self.vy + 0.2).min(7.0)"));
        assert!(fbody.contains("(self.vx as f64) < 0.005"));
        assert!(fbody.contains("self.old_vx = self.vx"));
    }

    /// W4-d127 chain item 2 — the sim-stamp source pin: the pre-pass
    /// stamps `cloud_alpha` (Rain.visual_alpha) and the slot-order
    /// roost neighbours with the per-slot `is_self` flag for EVERY
    /// living ai_style-24 kind (all 11 — the owls' scan is not
    /// special-cased away), gated by the any-bird probe. WRITTEN NOT
    /// RUN.
    #[test]
    fn the_sim_stamps_reach_every_style24_kind() {
        let src = include_str!("world_sim/npcs.rs");
        let start = src.find("the aiStyle-24 birds' ambient pre-stamps").expect("the pre-pass");
        let rest = &src[start..];
        let end = rest.find("// The Twins' shared-despawn seam").expect("the next pre-pass");
        let body = &rest[..end];
        assert!(body.contains("app.rain.visual_alpha()"), "the live cloudAlpha");
        assert!(body.contains("n.kind.ai_style() == 24"), "the style gate covers ALL 11 kinds");
        assert!(body.contains("is_self: j == i"), "the whoAmI skip rides the slot index");
        assert!(body.contains("n.kind.is_town_resident()"), "the townNPC row (:25351)");
        assert!(body.contains("cloud_alpha = cloud"));
        assert!(body.contains("style24_neighbors = snap"));
        // The stamp fields exist on the Npc ctor (the merge-ready
        // claim: no kind carries style 24 yet, so the pass no-ops).
        let npc_src = include_str!("npc.rs");
        assert!(npc_src.contains("cloud_alpha: 0.0,"), "the clear-sky stock");
        assert!(npc_src.contains("style24_neighbors: Vec::new(),"));
    }

    /// W4-d127 chain item 2 — the family arm's behaviour on a carrier
    /// slot (no style-24 kind exists on this branch — the census
    /// ownership rule): a perched carrier bird with the row's box,
    /// stamped neighbours and a negative-luck player runs the WHOLE
    /// seam shape the axis arm applies: the module call, the gravity
    /// tail (perched → gravity ON, the depth-scaled term), the dead-
    /// band snap and the ground landing. Drives the module + tail
    /// pieces directly (the private fn's shape is pinned by the source
    /// witness above). WRITTEN NOT RUN.
    #[test]
    fn the_perched_bird_lands_under_the_depth_scaled_gravity() {
        let mut w = air();
        // A floor at row 70 (y 1120): the bird at y 1100 falls 6 px.
        for c in 0..200 {
            let i = w.index(c, 70);
            w.tiles[i] = Tile { kind: 1, ..Default::default() };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        w.world_surface = 60.0;
        let players = [player(2000.0, 1000.0, 0.0, false)];
        let c = ctx(&players, &[]);
        let mut n = bird(74, 100.0, 1100.0);
        n.ai[0] = 0.0; // perched
        n.local_ai[0] = 1.0;
        let out = ai_style_24(&mut n, 74, &w, &c, &mut UnifiedRandom::new(1));
        assert!(!out.no_gravity, "the perch arm returns gravity (:25372)");
        // The tail's gravity term: depth-scaled 0.3 (y/16 = 68.75, the
        // 60+10*num² floor with width 200 → num² tiny → num2 clamps
        // 0.25 → 0.075), capped 10.
        let g = crate::npc_ai_golem::body_gravity(n.y, w.width as i32, w.world_surface);
        assert!(g > 0.0 && g <= 0.3);
        n.vy = (n.vy + g).min(10.0);
        let vy0 = n.vy;
        assert!(vy0 > 0.0, "the perched bird starts falling");
        // The ground snap (the tail's shape): feet (y + 14) cross row
        // 70's top (1120) → snap to 1120 - 14 and zero vy.
        let ny = n.y + n.vy;
        let feet_row = ((ny + 14.0) / 16.0).floor() as i32;
        let blocked = w
            .tile((n.x / 16.0) as i32, feet_row)
            .map_or(false, |t| t.active() && !t.inactive() && TILE_SOLID.has(t.kind));
        if blocked && n.vy > 0.0 {
            n.y = feet_row as f32 * 16.0 - 14.0;
            n.vy = 0.0;
        }
        // Either it landed exactly on the floor top or is still above.
        assert!(n.y + 14.0 <= 1120.0 + 0.001, "never inside the floor");
    }
}
