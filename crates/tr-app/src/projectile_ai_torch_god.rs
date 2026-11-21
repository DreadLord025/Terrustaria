//! The Torch God's torch projectile (949, aiStyle 184) — W4-f33.
//!
//! `AI_184_BadTorchLuck` (`Projectile.cs:42787-42822`): the owner-gated
//! `SelfHurtPlayers` (`:15025-15062`), the first-tick `SoundID.Item8` latch,
//! the `ai[1]` distance-budget drain (tileCollide arms when it runs out)
//! and the 4-dust trail census. The `Kill()` census is the 20-dust burst +
//! `SoundID.Item10` (`:70976-70995`).
//!
//! `SetDefaults` (`Projectile.cs:8948-8955`): 4×4, aiStyle 184,
//! `penetrate = -1`, `tileCollide = false` — the row lives in
//! `projectile/data.rs`. The shot NEVER dies on a player hit (the hurt
//! rides the AI's own `SelfHurtPlayers`, not the pool's hostile lane —
//! `hostile` stays false so `Damage_EVP` refuses, `:14775`, and
//! `friendly` stays false so `Damage_PVE_Inner` refuses, `:12611`).
//!
//! RNG: every draw rides the `UpdateProjectiles` stream (`Main.cs:17775` —
//! the AI runs under that swap). The dust visuals are the documented skip
//! (the pool has no dust layer); the CENSUS keeps every `Main.rand` draw:
//! `Dust.NewDust`'s six (`Dust.cs:170-175`, `:190`, the `npc_ai_empress`
//! precedent) + the torch-family seventh `Next(-10, 6)` (`Dust.cs:214-219`
//! — every torch dust id EXCEPT 156/234/66 is in that list) + the AI's
//! per-dust `Next(3)` noGravity roll (`:42811` / `:70984`). The pool-
//! pressure skip draws (`Dust.cs:114-148`) and the off-screen early-out
//! (`:98`) are the documented assumption (no skip draws), and
//! `ChildSafety.Disabled = true` by default (`ChildSafety.cs:33`) so the
//! dangerous-dust re-type arm (`Dust.cs:194-213`) never runs.

use glam::Vec2;
use tr_core::loot::DropRng;

use crate::player::Player;
use crate::projectile::Projectile;

/// `TorchID.Dust` (`TorchID.cs:179-184`) — the per-style dust ids, verbatim.
pub const TORCH_DUST: [i32; 24] = [
    6, 59, 60, 61, 62, 63, 64, 65, 75, 135, 158, 169, 156, 234, 66, 242, 293, 294, 295, 296,
    297, 298, 307, 310,
];

/// The `Dust.cs:214` torch/fire family list restricted to the ids
/// `TORCH_DUST` can produce: every entry except 156 (desert), 234 (bone)
/// and 66 (rainbow) takes the extra `Next(-10, 6)` velocity-Y draw
/// (`Dust.cs:216`) — the X×0.3 / scale×0.7 writes are visual-only.
fn torch_family_extra_draw(dust_id: i32) -> bool {
    !matches!(dust_id, 156 | 234 | 66)
}

/// `Dust.NewDust(position, 4, 4, id, 0, 0, 100)`'s RNG consumption
/// (`Dust.cs:170-175`, `:190` + the `:214-219` family arm): the 4×4 box
/// clamps to 5 (`:155-162`), so the position draws are `Next(1)` — they
/// always roll 0 but still advance the stream. Six base draws + the
/// family's seventh, in the C#'s order.
fn new_dust_census(rng: &mut dyn DropRng, dust_id: i32) {
    rng.next_range(0, 1); // :170 — Next(max(4,5)-4)
    rng.next_range(0, 1); // :171
    rng.next_range(-20, 21); // :172
    rng.next_range(-20, 21); // :173
    rng.next_range(0, 3); // :175 — frame.Y
    rng.next_range(-20, 21); // :190 — scale
    if torch_family_extra_draw(dust_id) {
        rng.next_range(-10, 6); // :216
    }
}

/// `Projectile.SelfHurtPlayers` (`Projectile.cs:15025-15062`) for the 949:
/// the rect-rect `Colliding(base.Hitbox, localPlayer.Hitbox)` gate, the
/// `DamageVar(damage, 0f - luck)` roll and the non-PVP `Hurt`
/// (`flag = !hostile && type != 949` is FALSE for the 949, `:15044` — so
/// `playerIndex = -1`, `pvp = false`). The shot does NOT die (no kill in
/// this method; `penetrate = -1`). `FixedSelfDamageForPlayers[949]` is
/// null (`ProjectileID.cs:342-422` — rockets + 1117 only), `trap` is false
/// (the deadMansSweater halving + trapDebuffSource never fire), and
/// `StatusPlayer` has no 949 arm (`:12108+`). `ownerHitCheck` is false
/// (the stock default). Returns whether a hit landed (the C# `Hurt > 0`
/// gate — the port's `hurt_with_source` bool).
fn self_hurt_players(p: &mut Projectile, player: &mut Player, rng: &mut dyn DropRng) -> bool {
    // `:15028` — active (the player exists), !dead, !immune, the box overlap.
    if player.dead || player.immune {
        return false;
    }
    let (pcx, _pcy) = player.center();
    let proj_cx = p.pos.x + p.w * 0.5;
    // XNA `Rectangle.Intersects` — the int-cast rects, strict `<` on all
    // four edges. The player box is the live hitbox (the mount framework's
    // `p.height`, the S1-P-i41 convention).
    let overlaps = (p.pos.x as i32) < (player.x + player.width) as i32
        && (player.x as i32) < (p.pos.x + p.w) as i32
        && (p.pos.y as i32) < (player.y + player.height) as i32
        && (player.y as i32) < (p.pos.y + p.h) as i32;
    if !overlaps {
        return false;
    }
    // `:15032` — `direction = !(player.Center.X < proj.Center.X) ? 1 : -1`.
    let hit_direction = if pcx < proj_cx { -1 } else { 1 };
    p.direction = hit_direction;
    // `:15033-15039` — no fixed-self-damage row for 949 → the shot's damage;
    // `DamageVar(value, 0f - localPlayer.luck)`.
    let dmg = tr_core::damage::damage_var(rng, p.damage as f32, 0.0 - player.luck());
    // `:15050` — `Hurt(dodgeable: IsDamageDodgeable(), ByProjectile(-1,
    // whoAmI), num2, direction, pvp: false)`. The 949 is not in the
    // 9999-damage undodgeable set (`:15064-15072`) → dodgeable.
    // `hurt_with_source` derives the knockback sign from `from_x`
    // (`player.center >= from_x → +1`); feeding `player.center −
    // hit_direction` reproduces the C#'s explicit `hitDirection` exactly
    // (including the tie: C# `!(pcx < proj_cx)` → +1 when equal).
    player.hurt_with_source(dmg, pcx - hit_direction as f32, rng, true, p.kind)
}

/// `AI_184_BadTorchLuck` (`Projectile.cs:42787-42822`) — one AI sub-step.
/// `local_owner` is `Main.myPlayer`; the hurt gate reads
/// `Main.LocalPlayer.unlockedBiomeTorches` (`:42789`): once the favor is
/// used the rain stops hurting. The movement integrate is the pool's
/// `kill_step` (the dispatch calls it after this arm — `HandleMovement`'s
/// default: free flight while `tileCollide` is false, kill-on-tile once
/// the budget arms it).
pub fn ai_184_bad_torch_luck(
    p: &mut Projectile,
    player: Option<&mut Player>,
    local_owner: u8,
    rng: &mut dyn DropRng,
) {
    // `:42789-42792` — the owner gate + the favor unlock gate. The C#
    // reads `Main.LocalPlayer` (always present); the port's `None` skips
    // the hurt only — the latch/flight/dust below run for every owner.
    if p.owner == local_owner {
        if let Some(pl) = player {
            if !pl.unlocked_biome_torches {
                self_hurt_players(p, pl, rng);
            }
        }
    }
    // `:42793-42797` — the first-tick Item8 latch. The C# plays inline;
    // the port queues the cue (the S1-E-21h convention — the pool drains
    // it at the pass tail on the same stream).
    if p.local_ai[0] == 0.0 {
        p.local_ai[0] = 1.0;
        p.wants_sound = Some((2, 8, p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5, 0.0));
    }
    // `:42798-42805` — the distance budget: while ai[1] > 0 it drains by
    // the speed (can overshoot negative); once <= 0 the tile collide arms.
    // `velocity.Length()` is XNA's form: the f32 sum, the f64 sqrt cast
    // back (the `npc_ai_style64::vector_distance` precedent).
    if p.ai[1] > 0.0 {
        let speed = ((p.vel.x * p.vel.x + p.vel.y * p.vel.y) as f64).sqrt() as f32;
        p.ai[1] -= speed;
    } else {
        p.tile_collide = true;
    }
    // `:42806-42821` — the 4-dust trail: each dust spawns at
    // `position + velocity/4*i` (visual), rolls the NewDust census + the
    // `Next(3) != 0` noGravity roll; the velocity ×0.3 and the disco
    // colour (id 66) draw nothing.
    let dust_id = p.ai[0] as i32;
    for _ in 0..4 {
        new_dust_census(rng, dust_id);
        let _no_gravity = rng.next_range(0, 3) != 0; // :42811
    }
}

/// The 949's `Kill()` census (`Projectile.cs:70976-70995`): the Item10
/// cue at the center, then 20 dusts — each the NewDust census + the
/// `Next(3)` noGravity roll (the velocity ×0.8 and the disco colour draw
/// nothing). The cue rides the pool's sound queue (the pass-tail drain,
/// the S1-E-21c batch convention — the C# plays it BEFORE the dust draws;
/// the port's deferred pitch draw is the recorded sound-wave deviation).
pub fn kill_census(
    p: &Projectile,
    rng: &mut dyn DropRng,
    sounds: &mut Vec<(u16, i32, f32, f32, f32)>,
) {
    sounds.push((2, 10, p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5, 0.0));
    let dust_id = p.ai[0] as i32;
    for _ in 0..20 {
        new_dust_census(rng, dust_id);
        let _no_gravity = rng.next_range(0, 3) != 0; // :70984
    }
}

/// The aim vector the event's launch computes (`Player.cs:17895-17899`):
/// spawn at the torch cell's top-center, `vector2 = Center − spawn`,
/// `num9 = vector2.Length()` (XNA: the f32 sum, the f64 sqrt cast back),
/// `vector2.Normalize()` (the reciprocal-multiply form — a zero vector
/// yields NaN, NO guard, the C# has none; the `npc_ai_style64` precedent),
/// then × 8. `ai[1]` carries the pre-normalize length.
pub fn launch_aim(spawn: Vec2, player_center: Vec2) -> (Vec2, f32) {
    let (x, y) = (player_center.x - spawn.x, player_center.y - spawn.y);
    let distance = ((x * x + y * y) as f64).sqrt() as f32;
    let num = 1.0f32 / distance;
    (Vec2::new(x * num * 8.0, y * num * 8.0), distance)
}

/// W4-f33 witnesses — WRITTEN NOT RUN (code-only mode). The 949 row, the
/// dust censuses' exact draw counts, the budget drain, the SelfHurtPlayers
/// gates and the favor-unlock immunity — pinned to the C# lines above.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectile::{proj_data, ProjectilePool, TORCH_GOD};
    use tr_core::UnifiedRandom;

    fn shot(dust_id: i32, distance: f32) -> Projectile {
        Projectile {
            active: true,
            kind: TORCH_GOD,
            owner: 0,
            pos: Vec2::new(105.0, 110.0),
            vel: Vec2::new(8.0, 0.0),
            damage: 40,
            w: 4.0,
            h: 4.0,
            tile_collide: false,
            friendly: false,
            penetrate: -1,
            ai: [dust_id as f32, distance, 0.0, 0.0],
            ..Default::default()
        }
    }

    #[test]
    fn w4_f33_the_949_row_matches_set_defaults() {
        // `Projectile.cs:8948-8955` — 4×4, aiStyle 184, penetrate −1,
        // tileCollide false; the stock timeLeft 3600 / extraUpdates 0.
        let d = proj_data(TORCH_GOD).expect("the 949 row");
        assert_eq!((d.w, d.h), (4.0, 4.0));
        assert_eq!(d.ai_style, 184);
        assert_eq!(d.penetrate, -1);
        assert!(!d.tile_collide);
        assert_eq!(d.time_left, 3600);
        assert_eq!(d.extra_updates, 0);
        assert!(!d.trap, "the stock trap false — no sweater halving");
        // The pool spawn carries the row (the drain's spawn path).
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data_ai_owned(TORCH_GOD, 0, 100.0, 100.0, 8.0, 0.0, 40, 1.0, 6.0, 200.0)
            .expect("the spawn");
        let p = &pool.slots()[slot];
        assert_eq!((p.w, p.h), (4.0, 4.0));
        assert_eq!(p.ai, [6.0, 200.0, 0.0, 0.0]);
        assert_eq!(p.penetrate, -1);
        assert!(!p.tile_collide);
    }

    #[test]
    fn w4_f33_the_torch_dust_table_is_verbatim() {
        // `TorchID.cs:179-184`.
        assert_eq!(
            TORCH_DUST,
            [6, 59, 60, 61, 62, 63, 64, 65, 75, 135, 158, 169, 156, 234, 66, 242, 293, 294,
             295, 296, 297, 298, 307, 310]
        );
        // The `Dust.cs:214` family split: 156/234/66 draw six, the rest seven.
        assert!(!torch_family_extra_draw(156));
        assert!(!torch_family_extra_draw(234));
        assert!(!torch_family_extra_draw(66));
        for id in [6, 59, 75, 135, 158, 169, 242, 293, 307, 310] {
            assert!(torch_family_extra_draw(id), "dust {id} is in the :214 list");
        }
    }

    #[test]
    fn w4_f33_the_ai_budget_drains_by_the_speed_then_arms_tile_collide() {
        // `Projectile.cs:42798-42805` — ai[1] -= velocity.Length() while
        // positive; <= 0 → tileCollide = true (the overshoot stays negative).
        let mut p = shot(6, 20.0);
        let mut rng = UnifiedRandom::new(1);
        ai_184_bad_torch_luck(&mut p, None, 0, &mut rng);
        assert_eq!(p.ai[1], 12.0, "20 − 8");
        assert!(!p.tile_collide);
        ai_184_bad_torch_luck(&mut p, None, 0, &mut rng);
        assert_eq!(p.ai[1], 4.0);
        ai_184_bad_torch_luck(&mut p, None, 0, &mut rng);
        assert_eq!(p.ai[1], -4.0, "the overshoot is kept");
        // The `else` arm fires on the NEXT tick (the C# if/else — the
        // drain tick itself does not arm the collide).
        assert!(!p.tile_collide);
        ai_184_bad_torch_luck(&mut p, None, 0, &mut rng);
        assert!(p.tile_collide, "the budget ran out");
        // Once armed it stays armed; the budget no longer drains.
        ai_184_bad_torch_luck(&mut p, None, 0, &mut rng);
        assert_eq!(p.ai[1], -4.0);
        assert!(p.tile_collide);
    }

    #[test]
    fn w4_f33_the_first_tick_latch_queues_item8_once() {
        // `Projectile.cs:42793-42797` — localAI[0] 0→1, SoundID.Item8
        // (sound_id 2, style 8 — `soundid.rs:160`) at base.Center.
        let mut p = shot(6, 100.0);
        let mut rng = UnifiedRandom::new(1);
        ai_184_bad_torch_luck(&mut p, None, 0, &mut rng);
        assert_eq!(p.local_ai[0], 1.0);
        assert_eq!(p.wants_sound, Some((2, 8, 107.0, 112.0, 0.0)));
        p.wants_sound = None;
        ai_184_bad_torch_luck(&mut p, None, 0, &mut rng);
        assert_eq!(p.wants_sound, None, "the latch holds");
    }

    #[test]
    fn w4_f33_the_ai_dust_census_draws_exactly_32_or_28_per_tick() {
        // 4 dusts × (NewDust's 6 + the family 7th + the Next(3) noGravity)
        // = 32 for a family id; 4 × (6 + 1) = 28 for the rainbow 66.
        for (dust_id, per_dust) in [(6, 8), (66, 7), (156, 7), (234, 7), (310, 8)] {
            let mut p = shot(dust_id, 1000.0);
            let mut rng = UnifiedRandom::new(99);
            let mut expect = UnifiedRandom::new(99);
            ai_184_bad_torch_luck(&mut p, None, 0, &mut rng);
            for _ in 0..4 {
                new_dust_census(&mut expect, dust_id);
                expect.next_range(0, 3);
            }
            assert_eq!(rng, expect, "dust {dust_id}: {per_dust} draws × 4");
        }
    }

    #[test]
    fn w4_f33_the_kill_census_draws_160_and_queues_item10() {
        // `Projectile.cs:70976-70995` — Item10 (2, style 10) + 20 dusts ×
        // (6 + family 7th + Next(3)) = 160 for a family id.
        let p = shot(6, 0.0);
        let mut rng = UnifiedRandom::new(7);
        let mut expect = UnifiedRandom::new(7);
        let mut sounds = Vec::new();
        kill_census(&p, &mut rng, &mut sounds);
        assert_eq!(sounds, vec![(2, 10, 107.0, 112.0, 0.0)]);
        for _ in 0..20 {
            new_dust_census(&mut expect, 6);
            expect.next_range(0, 3);
        }
        assert_eq!(rng, expect, "20 × 8 draws");
        // The rainbow id: 20 × 7 = 140.
        let p = shot(66, 0.0);
        let mut rng = UnifiedRandom::new(7);
        let mut expect = UnifiedRandom::new(7);
        kill_census(&p, &mut rng, &mut Vec::new());
        for _ in 0..20 {
            new_dust_census(&mut expect, 66);
            expect.next_range(0, 3);
        }
        assert_eq!(rng, expect);
    }

    /// A player whose box overlaps the shot at (105,110): x 100..120,
    /// y 100..142 (the stock 20×42 hitbox).
    fn overlapping_player() -> (tr_core::World, Player) {
        let w = tr_core::World::new("t".into(), 64, 64);
        let mut pl = Player::spawn(&w);
        pl.x = 100.0;
        pl.y = 100.0;
        pl.hp = 100;
        pl.stat_defense = 0;
        (w, pl)
    }

    #[test]
    fn w4_f33_self_hurt_lands_damage_var_and_never_kills_the_shot() {
        // `Projectile.cs:15025-15062` — the overlap → DamageVar(40, −luck)
        // → Hurt; the shot survives (penetrate −1, no kill in the method).
        let (_w, mut pl) = overlapping_player();
        let mut p = shot(6, 100.0);
        let mut rng = UnifiedRandom::new(31);
        let mut expect = UnifiedRandom::new(31);
        let dmg = tr_core::damage::damage_var(&mut expect, 40.0, 0.0);
        ai_184_bad_torch_luck(&mut p, Some(&mut pl), 0, &mut rng);
        assert_eq!(pl.hp, 100 - dmg, "the DamageVar hit landed");
        assert!(p.active, "the shot never dies on the hit");
        assert_eq!(p.penetrate, -1);
        // The stream: the DamageVar draw, then the 4-dust census.
        for _ in 0..4 {
            new_dust_census(&mut expect, 6);
            expect.next_range(0, 3);
        }
        assert_eq!(rng, expect);
        // The i-frames gate the repeat (`:15028` immune).
        assert!(pl.immune);
        let hp = pl.hp;
        ai_184_bad_torch_luck(&mut p, Some(&mut pl), 0, &mut rng);
        assert_eq!(pl.hp, hp, "the immune gate holds");
    }

    #[test]
    fn w4_f33_the_favor_unlock_stops_the_hurt() {
        // `Projectile.cs:42789` — `!Main.LocalPlayer.unlockedBiomeTorches`:
        // once the favor is used the rain flies but never hurts.
        let (_w, mut pl) = overlapping_player();
        pl.unlocked_biome_torches = true;
        let mut p = shot(6, 100.0);
        let mut rng = UnifiedRandom::new(31);
        let mut expect = UnifiedRandom::new(31);
        ai_184_bad_torch_luck(&mut p, Some(&mut pl), 0, &mut rng);
        assert_eq!(pl.hp, 100, "no hurt after the unlock");
        // Only the dust census draws — no DamageVar.
        for _ in 0..4 {
            new_dust_census(&mut expect, 6);
            expect.next_range(0, 3);
        }
        assert_eq!(rng, expect);
    }

    #[test]
    fn w4_f33_the_hurt_gates_dead_immune_owner_and_overlap() {
        let (_w, mut pl) = overlapping_player();
        // Dead → no hurt (`:15028`).
        pl.dead = true;
        let mut p = shot(6, 100.0);
        let mut rng = UnifiedRandom::new(1);
        ai_184_bad_torch_luck(&mut p, Some(&mut pl), 0, &mut rng);
        assert_eq!(pl.hp, 100);
        // Foreign owner → no hurt (the `owner == Main.myPlayer` gate).
        pl.dead = false;
        let mut p = shot(6, 100.0);
        p.owner = 1;
        ai_184_bad_torch_luck(&mut p, Some(&mut pl), 0, &mut rng);
        assert_eq!(pl.hp, 100);
        // No overlap → no hurt (the shot far away).
        let mut p = shot(6, 100.0);
        p.pos = Vec2::new(500.0, 500.0);
        ai_184_bad_torch_luck(&mut p, Some(&mut pl), 0, &mut rng);
        assert_eq!(pl.hp, 100);
    }

    /// A min-returning census rng (the d19 `CensusRng` shape): every
    /// `Next(min, max)` rolls `min` — the belt's `Next(10) == 0` dodge
    /// fires deterministically; the ranges are recorded for the order pin.
    struct MinRng {
        ranges: Vec<(i32, i32)>,
    }

    impl DropRng for MinRng {
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            self.ranges.push((min, max));
            min
        }
        fn next_float(&mut self) -> f32 {
            0.0
        }
    }

    /// The d19b dodge contract (research/93 addendum §2 hunk C) on the
    /// f33 arm: `SelfHurtPlayers` `:15050` passes `IsDamageDodgeable()` —
    /// the 949 is outside the exactly-9999 undodgeable set (`:15064-15072`),
    /// so `hurt_with_source(…, dodgeable: true, 949)` rides the shared
    /// `hurt_inner` dodge lane (`Player.cs:38214-38241` — sash → belt →
    /// brain → shadow). A belt-armed owner rolls the 1/10 BEFORE any
    /// damage lands; the dodge swallows the hit (no HP loss, the 120/80
    /// i-frames arm). WRITTEN NOT RUN (code-only).
    #[test]
    fn w4_f33_the_self_hurt_is_dodgeable() {
        let (_w, mut pl) = overlapping_player();
        pl.black_belt = true;
        let mut p = shot(6, 100.0);
        let mut rng = MinRng { ranges: Vec::new() };
        ai_184_bad_torch_luck(&mut p, Some(&mut pl), 0, &mut rng);
        assert_eq!(pl.hp, 100, "the belt dodged the fireball");
        assert!(pl.immune, "NinjaDodge's SetImmuneTimeForAllTypes");
        // The stream order: the DamageVar variance roll, then the belt's
        // 1/10, then the 4-dust census (the dodge returns before defense).
        assert_eq!(rng.ranges[0], (-15, 16), "DamageVar (`Main.cs:66120`)");
        assert_eq!(rng.ranges[1], (0, 10), "the belt's 1/10 (`:38222`)");
        assert!(p.active, "the shot never dies on the hit");
        // Without the belt the same overlap DOES hurt: the min-roll
        // DamageVar(40) = 40 × 0.85 = 34, defense 0 → 34 damage.
        let (_w2, mut pl2) = overlapping_player();
        let mut q = shot(6, 100.0);
        let mut rng2 = MinRng { ranges: Vec::new() };
        ai_184_bad_torch_luck(&mut q, Some(&mut pl2), 0, &mut rng2);
        assert_eq!(pl2.hp, 100 - 34, "the un-dodged fireball hurts");
        assert!(!rng2.ranges.contains(&(0, 10)), "no belt, no dodge roll");
    }

    #[test]
    fn w4_f33_the_launch_aim_is_normalized_to_8_with_the_raw_distance() {
        // `Player.cs:17895-17899`.
        let spawn = Vec2::new(1768.0, 6400.0);
        let center = Vec2::new(1760.0, 6400.0);
        let (aim, dist) = launch_aim(spawn, center);
        assert_eq!(dist, 8.0);
        assert_eq!(aim, Vec2::new(-8.0, 0.0));
        // A diagonal: (3,4) away → distance 5, aim (4.8, 6.4).
        let (aim, dist) = launch_aim(Vec2::ZERO, Vec2::new(3.0, 4.0));
        assert_eq!(dist, 5.0);
        assert!((aim.x - 4.8).abs() < 1e-5 && (aim.y - 6.4).abs() < 1e-5);
        // The zero-distance degenerate: XNA Normalize is 0 × ∞ = NaN —
        // NO guard (the C# has none; the out-of-world guard kills the NaN
        // slot next tick, `Projectile.cs:16690`).
        let (aim, dist) = launch_aim(spawn, spawn);
        assert_eq!(dist, 0.0);
        assert!(aim.x.is_nan() && aim.y.is_nan());
    }
}