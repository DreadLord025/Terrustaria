//! W4-d13 — `AI()`'s `else if (this.aiStyle == 127)` arm
//! (`Projectile.cs:35998-36085`): the sandnado column — the Sand Elemental's
//! friendly Sandnado (656, `ProjectileID.SandnadoFriendly`,
//! `ProjectileID.cs:1866`) and the hostile Sandnado (657,
//! `ProjectileID.SandnadoHostile`, `:1868`). The `SetDefaults` rows are
//! `:7048-7060` (656: friendly, magic, the LOCAL immunity pair 8,
//! timeLeft 1200) and `:7061-7070` (657: hostile, no pair, timeLeft 1200).
//! Both: 10x10, tileCollide false, penetrate -1.
//!
//! **RNG census (by construction).** The ONLY arm-level draw is the tail
//! dust's (`:36064-36084`, one iteration): `NextFloat` (`:36068`, the lerp
//! amount), `NextFloat` (`:36069`, the X lerp), then — only when
//! `vector160.X > -1.2` — `NextFloat` (`:36081`, the velocity X), then
//! `NextFloat` (`:36083`, the velocity Y). The `Dust.NewDust` internal rolls
//! belong to the dust pool and are SKIPPED (the S1-D-4b-7 precedent); the
//! dust body is skipped (no dust layer). The tail runs only while
//! `ai[0] < num956 - 120` (`:36060`'s early return).
//!
//! Deviations (the standing pool precedents, ANALYSIS): the
//! `SoundID.Item82` cue (`:36008`) rides `wants_sound`; the
//! `Collision.ExpandVertically` probe is a local verbatim port (the
//! `tr-core/stray.rs` copy is private); the `Collision.CanHitLine` sight
//! tests ride the pool's `can_hit_line`; `player.Top` folds to the body's
//! top-centre (no mount offsets, the `mounted_top` precedent). The 656's
//! `localAI[0]++` on-hit counter is `Damage_PVE_Inner`'s type-656 arm
//! (`Projectile.cs:14181-14184`) — wired in the damage walk, NOT here.

use glam::Vec2;
use tr_core::World;
use tr_core::loot::DropRng;

use crate::player::Player;
use crate::projectile::{SANDNADO_FRIENDLY, SANDNADO_HOSTILE, Projectile, can_hit_line};

/// `MathHelper.Lerp(value1, value2, amount)` (XNA): `value1 + (value2 -
/// value1) * amount` — NOT `Utils.GetLerpValue` (the port's
/// `get_lerp_value` is the inverse ramp).
fn math_helper_lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// `Kill()` (`Projectile.cs:70644-70648`) — the sibling-module deactivation
/// (the `projectile_ai_spear` precedent; `Projectile::kill` is private).
fn kill(p: &mut Projectile) {
    p.kill_time_left = p.time_left;
    p.active = false;
    p.time_left = 0;
}

/// `WorldGen.SolidTile3(int, int)` (`WorldGen.cs:70465-70483`): the LOOSEST
/// solid test — `nactive() && tileSolid[type] && !tileSolidTop[type]`, no
/// slope/half-brick term (the `tr-core/stray.rs` copy, verbatim).
fn solid_tile3(world: &World, x: i32, y: i32) -> bool {
    if x < 1 || x >= world.width - 1 || y < 1 || y >= world.height - 1 {
        return false;
    }
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    t.active()
        && !t.inactive()
        && tr_core::tiledata::TILE_SOLID.has(t.kind)
        && !tr_core::tiledata::TILE_SOLID_TOP.has(t.kind)
}

/// `Collision.ExpandVertically` (`Collision.cs:4372-4416`): expand the
/// column up/down from `(start_x, start_y)` until a `SolidTile3` cell or a
/// bound. Returns `(topY, bottomY)` — both INCLUSIVE of the blocking cell
/// (the caller does `topY++`/`bottomY--` for the free span). The
/// `InWorld(x, y, 10)` gate (`:4376`) returns the unchanged start pair.
/// The `tr-core/stray.rs` copy is private; this local mirror keeps the
/// guards verbatim.
fn expand_vertically(
    world: &World,
    start_x: i32,
    start_y: i32,
    max_expand_up: i32,
    max_expand_down: i32,
) -> (i32, i32) {
    let mut top_y = start_y;
    let mut bottom_y = start_y;
    if start_x < 10
        || start_x >= world.width - 10
        || start_y < 10
        || start_y >= world.height - 10
    {
        return (top_y, bottom_y);
    }
    for _ in 0..max_expand_up {
        if top_y < 10 || world.tile(start_x, top_y).is_none() {
            break;
        }
        if solid_tile3(world, start_x, top_y) {
            break;
        }
        top_y -= 1;
    }
    for _ in 0..max_expand_down {
        if bottom_y >= world.height - 10 || world.tile(start_x, bottom_y).is_none() {
            break;
        }
        if solid_tile3(world, start_x, bottom_y) {
            break;
        }
        bottom_y += 1;
    }
    (top_y, bottom_y)
}

impl Projectile {
    /// `AI()`'s aiStyle-127 arm (`Projectile.cs:35998-36085`). `player` is
    /// the owner link (the 656's sight test reads `Main.player[owner]`);
    /// `local_owner` gates that test (`owner == Main.myPlayer`). The
    /// movement is the dispatch's free integrate (`tileCollide = false`,
    /// `UpdatePosition` `:20445`); the arm sets the box + Centre itself.
    pub(crate) fn ai_sandnado(
        &mut self,
        world: &World,
        player: Option<&Player>,
        rng: &mut dyn DropRng,
        local_owner: u8,
    ) {
        // `:36000-36004` — the life cap: 900, the hostile 657's 300.
        let num956 = if self.kind == SANDNADO_HOSTILE { 300.0f32 } else { 900.0 };
        // `:36005-36009` — the one-shot Item82 cue at the Centre.
        if self.sound_delay == 0 {
            self.sound_delay = -1;
            let (cx, cy) = self.center();
            self.wants_sound = Some((2, 82, cx, cy, 0.0));
        }
        // `:36010-36014` — the age tick + the cap kill.
        self.ai[0] += 1.0;
        if self.ai[0] >= num956 {
            kill(self);
        }
        // `:36015-36024` — the friendly 656's 30-hit fade: damage 0 and the
        // age jumps into the last-120 window (the modulo keeps the phase).
        if self.kind == SANDNADO_FRIENDLY && self.local_ai[0] >= 30.0 {
            self.damage = 0;
            if self.ai[0] < num956 - 120.0 {
                let num957 = self.ai[0] % 60.0;
                self.ai[0] = num956 - 120.0 + num957;
                self.net_update = true;
            }
        }
        // `:36025-36038` — the column box from the vertical expand.
        let num958 = 15.0f32;
        let num959 = 15.0f32;
        let center = Vec2::new(self.pos.x + self.w * 0.5, self.pos.y + self.h * 0.5);
        let point8_x = center.x as i32 / 16;
        let point8_y = center.y as i32 / 16;
        let (mut top_y, mut bottom_y) =
            expand_vertically(world, point8_x, point8_y, num958 as i32, num959 as i32);
        top_y += 1;
        bottom_y -= 1;
        let value11 = Vec2::new(point8_x as f32, top_y as f32) * 16.0 + Vec2::splat(8.0);
        let value12 = Vec2::new(point8_x as f32, bottom_y as f32) * 16.0 + Vec2::splat(8.0);
        let vector158 = value11.lerp(value12, 0.5);
        let mut vector159 = Vec2::new(0.0, value12.y - value11.y);
        vector159.x = vector159.y * 0.2;
        self.w = (vector159.x * 0.65) as i32 as f32;
        self.h = vector159.y as i32 as f32;
        self.pos = vector158 - Vec2::new(self.w * 0.5, self.h * 0.5);
        // `:36039-36059` — the friendly 656's owner sight test: if NO point
        // along the column sees the owner's Centre or Top, jump the age into
        // the last-120 window.
        if self.kind == SANDNADO_FRIENDLY && self.owner == local_owner {
            if let Some(pl) = player {
                let (pcx, pcy) = pl.center();
                let center20 = Vec2::new(pcx, pcy);
                // `player.Top` — the body's top-centre (no mount offsets).
                let top = Vec2::new(pl.x + pl.width * 0.5, pl.y);
                let mut flag71 = false;
                // `:36044` — the f32 accumulation loop (0 → 1 by 0.05).
                let mut num960 = 0.0f32;
                while num960 < 1.0 {
                    let position = value11.lerp(value12, num960);
                    if can_hit_line(world, position.x, position.y, 0.0, 0.0, center20.x, center20.y, 0.0, 0.0)
                        || can_hit_line(world, position.x, position.y, 0.0, 0.0, top.x, top.y, 0.0, 0.0)
                    {
                        flag71 = true;
                        break;
                    }
                    num960 += 0.05;
                }
                if !flag71 && self.ai[0] < num956 - 120.0 {
                    let num961 = self.ai[0] % 60.0;
                    self.ai[0] = num956 - 120.0 + num961;
                    self.net_update = true;
                }
            }
        }
        // `:36060-36063` — the tail dust only runs in the active window.
        if !(self.ai[0] < num956 - 120.0) {
            return;
        }
        // `:36064-36084` — the single tail dust (the module doc's census).
        for _ in 0..1 {
            let value13 = -0.5f32;
            let value14 = 0.9f32;
            let amount3 = rng.next_float(); // `:36068`
            let mut vector160 = Vec2::new(
                math_helper_lerp(0.1, 1.0, rng.next_float()), // `:36069`
                math_helper_lerp(value13, value14, amount3),
            );
            vector160.x *= math_helper_lerp(2.2, 0.6, amount3);
            vector160.x *= -1.0;
            let vector161 = Vec2::new(6.0, 10.0);
            let _vector162 = vector158 + vector159 * vector160 * 0.5 + vector161;
            // The dust body is skipped; the conditional velocity draws stay.
            if vector160.x > -1.2 {
                let _ = rng.next_float(); // `:36081` velocity X
            }
            let _ = rng.next_float(); // `:36083` velocity Y
        }
    }
}

#[cfg(test)]
mod w4_d13_style127_tests {
    use super::*;
    use crate::projectile::proj_data;

    fn world() -> World {
        World::new("t".into(), 64, 64)
    }

    fn nado(kind: u16, pos: Vec2) -> Projectile {
        let mut p = Projectile::default();
        p.active = true;
        p.kind = kind;
        p.owner = 0;
        p.w = 10.0;
        p.h = 10.0;
        p.pos = pos;
        p.tile_collide = false;
        p
    }

    struct CensusRng {
        ranges: Vec<(i32, i32)>,
        floats: usize,
    }

    impl DropRng for CensusRng {
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            self.ranges.push((min, max));
            min
        }
        fn next_float(&mut self) -> f32 {
            self.floats += 1;
            0.0
        }
    }

    /// The two rows' `SetDefaults` slice (`Projectile.cs:7048-7070`).
    #[test]
    fn w4_d13_the_two_rows_carry_the_csharp_set_defaults() {
        for kind in [SANDNADO_FRIENDLY, SANDNADO_HOSTILE] {
            let d = proj_data(kind).unwrap_or_else(|| panic!("kind {kind} rowed"));
            assert_eq!((d.w, d.h), (10.0, 10.0), "kind {kind}");
            assert_eq!(d.ai_style, 127, "kind {kind}");
            assert_eq!(d.penetrate, -1, "kind {kind}");
            assert_eq!(d.time_left, 1200, "kind {kind}");
            assert!(!d.tile_collide, "kind {kind}");
        }
        let f = proj_data(SANDNADO_FRIENDLY).unwrap();
        assert!(f.uses_local_immunity, "the 656's local pair");
        assert_eq!(f.local_hit_cooldown, 8, "`:7058`");
        assert!(f.magic, "`:7054`");
        let h = proj_data(SANDNADO_HOSTILE).unwrap();
        assert!(!h.uses_local_immunity, "the 657 has no pair");
    }

    /// `:36005-36014` — the one-shot Item82 cue and the age tick; the cap
    /// kill at the life boundary.
    #[test]
    fn w4_d13_the_cue_latches_and_the_age_ticks() {
        let w = world();
        let mut p = nado(SANDNADO_HOSTILE, Vec2::new(300.0, 300.0));
        let mut rng = CensusRng { ranges: Vec::new(), floats: 0 };
        p.ai_sandnado(&w, None, &mut rng, 0);
        assert_eq!(p.sound_delay, -1, "the latch");
        assert_eq!(p.wants_sound.map(|r| (r.0, r.1)), Some((2, 82)));
        assert_eq!(p.ai[0], 1.0, "the age tick");
        // The cap kill: ai[0] at 299 → 300 >= 300.
        p.ai[0] = 299.0;
        p.sound_delay = -1;
        p.ai_sandnado(&w, None, &mut rng, 0);
        assert!(!p.active, "the 657's 300 cap");
    }

    /// `:36015-36024` — the friendly 656's 30-hit fade: damage 0 and the
    /// age jumps into the last-120 window with the modulo phase.
    #[test]
    fn w4_d13_the_656_fade_jumps_the_age() {
        let w = world();
        let mut p = nado(SANDNADO_FRIENDLY, Vec2::new(300.0, 300.0));
        p.damage = 40;
        p.local_ai[0] = 30.0;
        p.ai[0] = 99.0; // ++ → 100; 100 % 60 = 40; 900-120+40 = 820
        p.sound_delay = -1;
        let mut rng = CensusRng { ranges: Vec::new(), floats: 0 };
        p.ai_sandnado(&w, None, &mut rng, 0);
        assert_eq!(p.damage, 0, "the fade zeroes damage");
        assert_eq!(p.ai[0], 820.0, "the modulo age jump");
        assert!(p.net_update);
        // 820 >= 780 (900-120): the tail dust does NOT run.
        assert_eq!(rng.floats, 0, "no tail draws past the window");
    }

    /// `:36064-36084` — the tail dust census in the active window: the two
    /// lerp NextFloats, then the conditional velocity X (amount3 = 0 →
    /// vector160.X = -0.22 > -1.2, so it draws) and the velocity Y.
    #[test]
    fn w4_d13_the_tail_dust_draws_four_floats() {
        let w = world();
        let mut p = nado(SANDNADO_HOSTILE, Vec2::new(300.0, 300.0));
        p.ai[0] = 0.0; // ++ → 1 < 180 (300-120): the tail runs
        p.sound_delay = -1;
        let mut rng = CensusRng { ranges: Vec::new(), floats: 0 };
        p.ai_sandnado(&w, None, &mut rng, 0);
        // amount3 = 0 → X = lerp(0.1,1,0)=0.1 × lerp(2.2,0.6,0)=2.2 × -1
        // = -0.22 > -1.2 → the velocity-X draw fires. 4 floats total.
        assert_eq!(rng.floats, 4, "the two lerps + velocity X + velocity Y");
        assert!(rng.ranges.is_empty(), "no range draws on this path");
    }
}