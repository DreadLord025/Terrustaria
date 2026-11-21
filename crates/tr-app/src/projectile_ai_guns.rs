//! W4-d6 — the aiStyle-60 gun streams (`AI()`'s inline
//! `else if (this.aiStyle == 60)`, `Projectile.cs:28994-29309`): the Water
//! Gun's blob (358, `:4487-4496`), the Slime Gun's (406, `:4868-4877`) and
//! the Shimmer Gun's (1042, `:9848-9857`). The three rows share the arm;
//! the per-kind switch (`:29009-29023`) picks the debuff (103 Wet 1500 /
//! 137 Slimed 180 / 353 Shimmer 100), the buff duration and the tile-walk
//! mode (lights OFF / lights ON / biome-torch conversion).
//!
//! C# sources read WHOLE (the golden rule):
//! - the AI arm `Projectile.cs:28994-29309`: the 0.015 scale decay + the
//!   ×5 velocity death kick (`:28996-29002` — `oldVelocity = velocity`
//!   BEFORE the Kill, so the death dust reads the kicked velocity), the
//!   `ai[0] > 3` arming gate, the owner-gated NPC/player box-buff scan
//!   (`:29024-29054` — the NPC loop keeps buffing EVERY intersecting NPC
//!   after the first Kill; `Kill()`'s `!active` head makes the repeats
//!   no-ops), the three tile walks (`:29055-29151`), the `ai[0] += ai[1]`
//!   accumulator + the >30 gravity (`:29153-29157`) and the per-kind dust
//!   draws (`:29158-29303`); the else arm's `ai[0]++` (`:29305-29308`).
//! - `GetCollisionParams` (`:19872-19876`): aiStyle 60 collides through a
//!   1×1 centre box — the movement stand-in probes that box.
//! - `WorldGen.TryToggleLight` (`WorldGen.cs:72970-73024`) →
//!   `tr_core::wiring::try_toggle_light` (the `Wiring.Toggle*` family,
//!   `Wiring.cs:2800-3072`); the campfire validate's mismatch kills ride
//!   the returned `ObjectDrop`s out to the pool's drain.
//! - `Player.BiomeTorchPlaceStyle` (`Player.cs:40236-40289`) →
//!   `world_sim::editing::biome_torch_place_style` (the
//!   `useWithoutTorchGodsFavor: true` call at `:29142` runs the body
//!   unconditionally). `TorchID.Sets.IsABiomeTorch` (`TorchID.cs:17`):
//!   `CreateBoolSet(false, 0, 18, 19, 20, 21, 23, 13, 7, 9, 22, 16)` —
//!   the listed styles TRUE (`SetFactory.cs:97-109`).
//! - the `Kill()` dust arms (`:74029-74147`): the 358's 50× dust 211, the
//!   406's 40× dust 4 and the 1042's 50× dust 324 — each reads
//!   `oldVelocity * 0.2f` as its velocity base.
//!
//! RNG contract (the `UpdateProjectiles` stream): every EXPLICIT
//! `Main.rand` draw stays in the C#'s order; the `Dust.NewDust` bodies and
//! their internal rolls go with the dust pool (the `projectile_ai_misc`
//! convention). Per tick while armed: the 358/406 draw 6×3 `Next(2)` alpha
//! gates + one `Next(4)` extra-dust gate; the 1042 adds a `Next(6)` colour
//! pick per dust and on the taken extra gate. Per death: the 358/406 draw
//! 50/40 × [4 `Next(2)` + `Next(-100,101)` + 2 `Next(-50,51)`]; the 1042
//! adds a `Next(6)` per dust.
//!
//! Movement: the rows keep `tileCollide = true`; `HandleMovement`'s
//! generic arm collides through the 1×1 `GetCollisionParams` box and the
//! blocked move falls to `position += velocity; Kill()` (`:19851-19852`).
//! The port's stand-in is the destination-box probe on that 1×1 box (the
//! `kill_step` shape, the swept-span deviation documented there).
//!
//! Deliberate skips (ANALYSIS): the dust bodies, `NetMessage.SendData(66)`-
//! shaped MP tails, `ApplyBuffTo`'s `trap && deadMansSweater` halving
//! (`:12393-12396` — the rows carry `trap = false`, so the branch is dead
//! for them anyway), the `NPCID.Sets.ShimmerTownTransform` gate (flag24 is
//! false for all three kinds — the branch is dead), the player-buff loop's
//! 255-slot walk folds to the single local player (index 0; the
//! `num442 != this.owner` exclusion keeps the owner unbuffed).
//!
//! CODE-ONLY MODE (W4-d6): the witness tests at the bottom are WRITTEN NOT
//! RUN; every expected value is hand-derived from the cited C# lines.

use glam::Vec2;
use tr_core::World;
use tr_core::loot::DropRng;

use crate::npc::Npc;
use crate::projectile::{Projectile, SHIMMER_GUN_STREAM, SLIME_GUN, WATER_GUN};

/// `TorchID.Sets.IsABiomeTorch` (`TorchID.cs:17`):
/// `CreateBoolSet(false, 0, 18, 19, 20, 21, 23, 13, 7, 9, 22, 16)` — the
/// listed styles TRUE (`SetFactory.cs:97-109`), every other style false.
fn is_a_biome_torch(style: i32) -> bool {
    matches!(style, 0 | 7 | 9 | 13 | 16 | 18 | 19 | 20 | 21 | 22 | 23)
}

/// `Rectangle.Intersects` (`XNA`: strict `<` on all four edges) over the
/// int-truncated boxes the C# builds (`:29026`/`:29030`).
fn rects_intersect(ax: f32, ay: f32, aw: f32, ah: f32, bx: f32, by: f32, bw: f32, bh: f32) -> bool {
    let (ax, ay, aw, ah) = (ax as i32, ay as i32, aw as i32, ah as i32);
    let (bx, by, bw, bh) = (bx as i32, by as i32, bw as i32, bh as i32);
    bx < ax + aw && ax < bx + bw && by < ay + ah && ay < by + bh
}

/// `AI()`'s aiStyle-60 arm (`Projectile.cs:28994-29309`). Returns the
/// campfire-validate mismatch kills the slime walk produced (the pool
/// drains them into item spawns). `old_velocity` is NOT read here — the
/// death kick writes `velocity` itself and the death hook distinguishes
/// the scale death by `scale <= 0` (see [`gun_stream_death_draws`]).
pub(crate) fn gun_stream(
    p: &mut Projectile,
    world: &mut World,
    npcs: &mut [Npc],
    player: Option<&mut crate::player::Player>,
    local_owner: u8,
    rng: &mut dyn DropRng,
) -> Vec<tr_core::edit::ObjectDrop> {
    let mut drops = Vec::new();
    // `:28996-29002` — the scale decay; at <= 0 the ×5 kick, the
    // oldVelocity latch (the death hook reads the kicked velocity through
    // the scale marker) and the Kill. NO return — the armed block below
    // still runs on the dead slot.
    p.scale -= 0.015;
    if p.scale <= 0.0 {
        p.vel *= 5.0;
        p.active = false;
        p.time_left = 0;
    }
    // `:29003-29308` — the arming gate.
    if p.ai[0] > 3.0 {
        // `:29005-29023` — the per-kind constants.
        let (buff, duration, players_too) = match p.kind {
            WATER_GUN => (103, 1500, true),
            SLIME_GUN => (137, 180, true),
            SHIMMER_GUN_STREAM => (353, 100, false),
            _ => (103, 1500, true),
        };
        // `:29024-29152` — the owner-gated contact + tile block.
        if p.owner == local_owner {
            // `:29027-29043` — the NPC scan: EVERY intersecting NPC gets
            // the buff (the Kill repeats are no-ops through the !active
            // head). `active` folds to hp > 0 (the port convention).
            for n in npcs.iter_mut() {
                if n.hp > 0
                    && !n.dont_take_damage
                    && n.max_hp > 1
                    && rects_intersect(p.pos.x, p.pos.y, p.w, p.h, n.x, n.y, n.size().0, n.size().1)
                {
                    // flag24 is false for all three kinds — the
                    // ShimmerTownTransform gate (`:29033`) is dead.
                    n.buffs.add_buff(buff, duration);
                    p.active = false;
                    p.time_left = 0;
                }
            }
            // `:29044-29054` — the player scan (the 358/406 only): the
            // single local player is index 0; `num442 != this.owner`
            // keeps the owner unbuffed. `ApplyBuffTo`'s trap halving is
            // dead (trap = false). The `&mut` reborrow is scoped so the
            // Shimmer tile walk below can read the player immutably.
            if players_too {
                if let Some(pl) = player.as_deref_mut() {
                    if p.owner != 0
                        && !pl.dead
                        && rects_intersect(
                            p.pos.x, p.pos.y, p.w, p.h,
                            pl.x, pl.y, crate::player::WIDTH, pl.height,
                        )
                    {
                        pl.buffs.add_buff(buff, duration);
                        p.active = false;
                        p.time_left = 0;
                    }
                }
            }
            // `:29055-29151` — the tile walks. The three share the box
            // decode: `(int)Left.X / 16 ..= (int)Right.X / 16`,
            // `(int)(Top.Y / 16) ..= (int)(Bottom.Y / 16)`; the liquid
            // kill compares `Bottom.Y > y*16 + (256 - liquid)/32*2` (the
            // int arithmetic verbatim).
            let bottom = p.pos.y + p.h;
            let x0 = p.pos.x as i32 / 16;
            let x1 = (p.pos.x + p.w) as i32 / 16;
            let y0 = (p.pos.y / 16.0) as i32;
            let y1 = (bottom / 16.0) as i32;
            for tx in x0..=x1 {
                for ty in y0..=y1 {
                    // `WorldGen.InWorld(x, y, 10)` (`WorldGen.cs:9688`).
                    if tx < 10 || tx >= world.width - 10 || ty < 10 || ty >= world.height - 10 {
                        continue;
                    }
                    // Copy the cell facts out before any mutable call
                    // (the walk writes tiles / lights below).
                    let Some(t) = world.tile(tx, ty) else { continue };
                    let (liquid, active, kind, frame_y) = (t.liquid, t.active(), t.kind, t.frame_y);
                    if liquid > 0 {
                        if bottom > (ty * 16 + (256 - liquid as i32) / 32 * 2) as f32 {
                            p.active = false;
                            p.time_left = 0;
                        }
                    } else if active {
                        match buff {
                            // `:29078-29081` — the Water Gun turns lights OFF.
                            103 => {
                                let _ = tr_core::wiring::try_toggle_light(
                                    world, tx, ty, Some(false), false, &mut drops,
                                );
                            }
                            // `:29108-29111` — the Slime Gun turns them ON.
                            137 => {
                                let _ = tr_core::wiring::try_toggle_light(
                                    world, tx, ty, Some(true), false, &mut drops,
                                );
                            }
                            // `:29138-29148` — the Shimmer Gun converts
                            // biome torches to the owner's biome style.
                            353 => {
                                if kind == 4 && is_a_biome_torch(frame_y as i32 / 22) {
                                    let style = player
                                        .as_deref()
                                        .map(|pl| {
                                            crate::world_sim::editing::biome_torch_place_style(pl, world)
                                        })
                                        .unwrap_or(0);
                                    if style != frame_y as i32 / 22 {
                                        if let Some(mt) = world.tile_mut(tx, ty) {
                                            mt.frame_y = (style * 22) as i16;
                                        }
                                        // NetMessage.SendTileSquare — MP-only.
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        // `:29153-29157` — the accumulator and the >30 gravity.
        p.ai[0] += p.ai[1];
        if p.ai[0] > 30.0 {
            p.vel.y += 0.1;
        }
        // `:29158-29303` — the per-kind dust draws (the bodies skip).
        match p.kind {
            WATER_GUN | SLIME_GUN => {
                // 6 dusts × 3 `Next(2)` alpha gates, then the `Next(4)`
                // extra-dust gate (its body draws nothing explicit).
                for _ in 0..6 {
                    for _ in 0..3 {
                        let _ = rng.next_range(0, 2);
                    }
                }
                let _ = rng.next_range(0, 4);
            }
            SHIMMER_GUN_STREAM => {
                // 6 dusts × [3 `Next(2)` + the `Next(6)` colour], then the
                // `Next(4)` gate (+ one `Next(6)` when taken).
                for _ in 0..6 {
                    for _ in 0..3 {
                        let _ = rng.next_range(0, 2);
                    }
                    let _ = rng.next_range(0, 6);
                }
                if rng.next_range(0, 4) == 0 {
                    let _ = rng.next_range(0, 6);
                }
            }
            _ => {}
        }
    } else {
        // `:29305-29308` — the pre-arm counter.
        p.ai[0] += 1.0;
    }
    // HandleMovement (`:17015`): the 1×1 centre-box probe
    // (`GetCollisionParams`, `:19872-19876`), the blocked move's
    // `position += velocity; Kill()` (`:19851-19852`). Runs on the dead
    // slot too (the `!active` return sits after, `:17027`).
    let cx = p.pos.x + p.w * 0.5 - 0.5;
    let cy = p.pos.y + p.h * 0.5 - 0.5;
    let hit_x = crate::player::collides_rect(world, cx + p.vel.x, cy, 1.0, 1.0);
    let hit_y = crate::player::collides_rect(world, cx, cy + p.vel.y, 1.0, 1.0);
    p.pos += p.vel;
    if hit_x || hit_y {
        p.active = false;
        p.time_left = 0;
    }
    drops
}

/// The `Kill()` dust census (`Projectile.cs:74029-74147`) — the 358's 50×
/// dust 211 (`:74029-74062`), the 406's 40× dust 4 (`:74063-74098`) and the
/// 1042's 50× dust 324 (`:74099-74147`). Each arm first writes
/// `velocity = oldVelocity * 0.2f`; on the SCALE death the C# latched
/// `oldVelocity = velocity` AFTER the ×5 kick (`:28999-29000`), so the
/// kicked velocity is the base — the `scale <= 0` marker distinguishes it
/// from every other death path (only this arm decays the scale).
/// `old_velocity` is the pool's sub-step-start capture (`:16766`).
pub(crate) fn gun_stream_death_draws(p: &mut Projectile, old_velocity: Vec2, rng: &mut dyn DropRng) {
    let (count, shimmer) = match p.kind {
        WATER_GUN => (50, false),
        SLIME_GUN => (40, false),
        SHIMMER_GUN_STREAM => (50, true),
        _ => return,
    };
    let base = if p.scale <= 0.0 { p.vel } else { old_velocity };
    p.vel = base * 0.2;
    for _ in 0..count {
        // The 3 alpha `Next(2)` gates + the scale/noGravity `Next(2)` pick.
        for _ in 0..4 {
            let _ = rng.next_range(0, 2);
        }
        // The velocity jitter: `Next(-100, 101)` then 2× `Next(-50, 51)`.
        let _ = rng.next_range(-100, 101);
        let _ = rng.next_range(-50, 51);
        let _ = rng.next_range(-50, 51);
        if shimmer {
            // The 1042's `Next(6)` colour pick (`:74131`).
            let _ = rng.next_range(0, 6);
        }
    }
}

// =====================================================================
// Witness tests — WRITTEN, NOT RUN (code-only mode: no cargo/rustc).
// Every expected value is hand-derived from the cited C# lines.
// =====================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectile::proj_data;
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    /// A draw-recording stream (the `projectile_ai_misc` CensusRng shape).
    struct CensusRng {
        ranges: Vec<(i32, i32)>,
    }
    impl DropRng for CensusRng {
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            self.ranges.push((min, max));
            min
        }
        fn next_float(&mut self) -> f32 {
            0.0
        }
    }

    fn world() -> World {
        World::new("t".into(), 300, 200)
    }

    fn proj(kind: u16) -> Projectile {
        let d = proj_data(kind).expect("the rowed kind");
        let mut p = Projectile::default();
        p.active = true;
        p.kind = kind;
        p.owner = 0;
        p.pos = Vec2::new(100.0 * 16.0, 50.0 * 16.0);
        p.w = d.w;
        p.h = d.h;
        p.tile_collide = d.tile_collide;
        p.alpha = d.alpha;
        p.scale = d.scale;
        p.time_left = d.time_left;
        p.penetrate = d.penetrate;
        p.extra_updates = d.extra_updates;
        p
    }

    fn put(w: &mut World, x: i32, y: i32, kind: u16, frame_x: i16, frame_y: i16) {
        let i = w.index(x, y);
        w.tiles[i] = Tile { kind, frame_x, frame_y, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
    }

    #[test]
    fn w4_d6_the_gun_rows_are_verbatim() {
        // `:4487-4496` / `:4868-4877` / `:9848-9857`.
        let d = proj_data(WATER_GUN).unwrap();
        assert_eq!((d.w, d.h), (18.0, 18.0));
        assert_eq!(d.ai_style, 60);
        assert_eq!(d.alpha, 255);
        assert_eq!(d.penetrate, -1);
        assert_eq!(d.extra_updates, 2);
        assert!(d.ignore_water);
        assert!(d.tile_collide);
        let d = proj_data(SLIME_GUN).unwrap();
        assert_eq!((d.w, d.h), (14.0, 14.0));
        assert_eq!(d.ai_style, 60);
        let d = proj_data(SHIMMER_GUN_STREAM).unwrap();
        assert_eq!((d.w, d.h), (18.0, 18.0));
        assert_eq!(d.ai_style, 60);
        assert_eq!(d.extra_updates, 2);
    }

    #[test]
    fn w4_d6_the_scale_decays_and_the_death_kicks_the_velocity_x5() {
        // `:28996-29002`: 1.0 − 0.015 per sub-step; at <= 0 the ×5 kick
        // and the kill. 67 sub-steps land 1.0 − 67·0.015 = −0.005.
        let mut w = world();
        let mut rng = CensusRng { ranges: Vec::new() };
        let mut p = proj(WATER_GUN);
        p.vel = Vec2::new(2.0, 0.0);
        for _ in 0..66 {
            let _ = gun_stream(&mut p, &mut w, &mut [], None, 0, &mut rng);
        }
        assert!(p.active, "66 decays stay positive (1.0 − 0.99)");
        let _ = gun_stream(&mut p, &mut w, &mut [], None, 0, &mut rng);
        assert!(!p.active, "the 67th decay crosses zero");
        assert!(p.scale <= 0.0);
        assert_eq!(p.vel.x, 10.0, "the ×5 death kick");
        // The pre-arm counter ran 4 ticks then the accumulator took over.
        assert_eq!(p.ai[0], 4.0, "ai[0]++ while <= 3, then += ai[1] = 0");
    }

    #[test]
    fn w4_d6_the_arm_gate_counts_to_4_then_accumulates_ai1_and_gravitates_past_30() {
        // `:29003`/`:29305-29308`: ai[0]++ until > 3; then ai[0] += ai[1]
        // and the >30 gravity (`:29153-29157`).
        let mut w = world();
        let mut rng = CensusRng { ranges: Vec::new() };
        let mut p = proj(SLIME_GUN);
        p.ai[1] = 10.0;
        for _ in 0..4 {
            let _ = gun_stream(&mut p, &mut w, &mut [], None, 0, &mut rng);
        }
        assert_eq!(p.ai[0], 4.0);
        assert!(rng.ranges.is_empty(), "no draws before the arm gate");
        let _ = gun_stream(&mut p, &mut w, &mut [], None, 0, &mut rng);
        assert_eq!(p.ai[0], 14.0, "the first += ai[1]");
        assert_eq!(p.vel.y, 0.0, "no gravity at 14");
        let _ = gun_stream(&mut p, &mut w, &mut [], None, 0, &mut rng);
        let _ = gun_stream(&mut p, &mut w, &mut [], None, 0, &mut rng);
        assert_eq!(p.ai[0], 34.0);
        assert!((p.vel.y - 0.1).abs() < 1e-6, "the >30 gravity");
        // The armed 406 tick draws 18 Next(2) + one Next(4).
        assert_eq!(rng.ranges.len(), 3 * 19, "three armed ticks");
        assert_eq!(rng.ranges[18], (0, 4), "the extra-dust gate closes the tick");
    }

    #[test]
    fn w4_d6_the_water_stream_buffs_every_intersecting_npc_and_turns_lights_off() {
        // `:29027-29043`: the NPC loop buffs EVERY intersecting NPC (the
        // Kill repeats no-op). `:29078-29081`: the walk forces lights OFF
        // (the torch's ±66 frameX flip, `Wiring.cs:2903-2918`).
        let mut w = world();
        let mut rng = CensusRng { ranges: Vec::new() };
        let mut p = proj(WATER_GUN);
        p.ai[0] = 4.0;
        let mut n1 = Npc::new(crate::npc::NpcKind::Zombie, p.pos.x, p.pos.y, 1, 0.0);
        n1.max_hp = 40;
        n1.hp = 40;
        let mut n2 = Npc::new(crate::npc::NpcKind::Zombie, p.pos.x + 2.0, p.pos.y, 1, 0.0);
        n2.max_hp = 40;
        n2.hp = 40;
        // A LIT torch under the blob's centre (frameX 66 = OFF, 0 = ON).
        let tx = (p.pos.x + p.w * 0.5) as i32 / 16;
        let ty = (p.pos.y + p.h * 0.5) as i32 / 16;
        put(&mut w, tx, ty, 4, 0, 0);
        let mut npcs = vec![n1, n2];
        let _ = gun_stream(&mut p, &mut w, &mut npcs, None, 0, &mut rng);
        assert!(!p.active, "the contact kills");
        assert!(npcs[0].buffs.has_buff(103), "Wet on the first");
        assert!(npcs[1].buffs.has_buff(103), "Wet on the second too (no break)");
        assert_eq!(w.tile(tx, ty).unwrap().frame_x, 66, "the lit torch goes OFF");
        // A second pass on an already-OFF torch leaves it (the forced
        // state matches — `Wiring.cs:2906`).
        let mut p = proj(WATER_GUN);
        p.ai[0] = 4.0;
        let _ = gun_stream(&mut p, &mut w, &mut npcs, None, 0, &mut rng);
        assert_eq!(w.tile(tx, ty).unwrap().frame_x, 66, "no double flip");
    }

    #[test]
    fn w4_d6_the_slime_stream_turns_lights_on_and_the_liquid_kills() {
        // `:29108-29111`: forced ON. `:29100-29107`: a liquid cell deeper
        // than the surface line kills (`Bottom.Y > y*16 + (256-l)/32*2`).
        let mut w = world();
        let mut rng = CensusRng { ranges: Vec::new() };
        let mut p = proj(SLIME_GUN);
        p.ai[0] = 4.0;
        let tx = (p.pos.x + p.w * 0.5) as i32 / 16;
        let ty = (p.pos.y + p.h * 0.5) as i32 / 16;
        put(&mut w, tx, ty, 4, 66, 0); // an OFF torch
        let _ = gun_stream(&mut p, &mut w, &mut [], None, 0, &mut rng);
        assert_eq!(w.tile(tx, ty).unwrap().frame_x, 0, "the OFF torch goes ON");
        // The liquid kill: full water (255) in the centre cell — the
        // surface line sits at ty*16 + 0; the blob's bottom is below it.
        let mut p = proj(SLIME_GUN);
        p.ai[0] = 4.0;
        let i = w.index(tx, ty);
        w.tiles[i] = Tile { liquid: 255, ..Default::default() };
        let _ = gun_stream(&mut p, &mut w, &mut [], None, 0, &mut rng);
        assert!(!p.active, "the submerged blob dies");
    }

    #[test]
    fn w4_d6_the_shimmer_stream_converts_biome_torches_and_draws_the_colour_picks() {
        // `:29138-29148`: a biome torch's frameY becomes the owner's
        // biome style × 22. `:29265-29279`: the per-dust Next(6) colour.
        let mut w = World::new("t".into(), 300, 1200);
        let mut rng = CensusRng { ranges: Vec::new() };
        let mut p = proj(SHIMMER_GUN_STREAM);
        p.ai[0] = 4.0;
        let tx = (p.pos.x + p.w * 0.5) as i32 / 16;
        let ty = (p.pos.y + p.h * 0.5) as i32 / 16;
        put(&mut w, tx, ty, 4, 0, 0); // the plain torch (style 0, in the set)
        let mut player = crate::player::Player::spawn(&w);
        player.y = 100.0; // above the underworld line ((1200-200)*16)
        player.zones.snow = true; // the snow arm → style 9 (`Player.cs:40272`)
        let _ = gun_stream(&mut p, &mut w, &mut [], Some(&mut player), 0, &mut rng);
        assert_eq!(w.tile(tx, ty).unwrap().frame_y, 9 * 22, "the snow torch style");
        // The armed 1042 tick: 6 × [3 Next(2) + Next(6)] + the Next(4)
        // gate; the CensusRng returns min (0) so the gate TAKES → one
        // more Next(6).
        assert_eq!(rng.ranges.len(), 6 * 4 + 2);
        assert_eq!(rng.ranges[3], (0, 6), "the first colour pick");
        assert_eq!(rng.ranges[24], (0, 4), "the extra gate");
        assert_eq!(rng.ranges[25], (0, 6), "the taken gate's colour");
    }

    #[test]
    fn w4_d6_the_death_dust_census_counts_and_orders() {
        // `:74029-74147`: the 358 draws 50 × 7, the 406 40 × 7, the 1042
        // 50 × 8 (the colour pick). The velocity base is oldVelocity·0.2
        // — or the ×5-kicked velocity on the scale death.
        let mut rng = CensusRng { ranges: Vec::new() };
        let mut p = proj(WATER_GUN);
        gun_stream_death_draws(&mut p, Vec2::new(4.0, 0.0), &mut rng);
        assert_eq!(rng.ranges.len(), 50 * 7);
        assert_eq!(rng.ranges[4], (-100, 101), "the jitter after the 4 gates");
        assert_eq!(p.vel.x, 0.8, "oldVelocity * 0.2");
        // The scale death reads the KICKED velocity (`:28999-29000`).
        let mut rng = CensusRng { ranges: Vec::new() };
        let mut p = proj(SLIME_GUN);
        p.scale = -0.005;
        p.vel = Vec2::new(10.0, 0.0);
        gun_stream_death_draws(&mut p, Vec2::new(2.0, 0.0), &mut rng);
        assert_eq!(rng.ranges.len(), 40 * 7);
        assert_eq!(p.vel.x, 2.0, "the kicked 10 × 0.2, not the old 2 × 0.2");
        let mut rng = CensusRng { ranges: Vec::new() };
        let mut p = proj(SHIMMER_GUN_STREAM);
        gun_stream_death_draws(&mut p, Vec2::ZERO, &mut rng);
        assert_eq!(rng.ranges.len(), 50 * 8);
        assert_eq!(rng.ranges[7], (0, 6), "the colour pick closes each dust");
    }

    #[test]
    fn w4_d6_the_1x1_collision_probe_kills_on_contact() {
        // `GetCollisionParams` (`:19872-19876`): the 1×1 centre box. A
        // solid tile the centre crosses kills; the 18×18 box's edges
        // alone do not.
        let mut w = world();
        let mut rng = CensusRng { ranges: Vec::new() };
        let mut p = proj(WATER_GUN);
        p.vel = Vec2::new(2.0, 0.0);
        // The centre sits at pos + 9 − 0.5; one step of +2 keeps it in
        // open air — no kill.
        let _ = gun_stream(&mut p, &mut w, &mut [], None, 0, &mut rng);
        assert!(p.active);
        // A solid exactly where the centre lands next.
        let cx = (p.pos.x + p.w * 0.5 - 0.5 + p.vel.x) as i32 / 16;
        let cy = (p.pos.y + p.h * 0.5 - 0.5) as i32 / 16;
        put(&mut w, cx, cy, 1, 0, 0);
        let _ = gun_stream(&mut p, &mut w, &mut [], None, 0, &mut rng);
        assert!(!p.active, "the 1×1 probe hits");
    }
}