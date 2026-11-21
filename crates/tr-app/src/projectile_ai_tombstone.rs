//! W4-d2 — the aiStyle-17 tombstones: the Tombstone (43), the five grave
//! markers (201-205) and the five rich gravestones (527-531). `AI()`'s
//! `else if (this.aiStyle == 17)` (`Projectile.cs:25316-25366`), plus the
//! movement half (`HandleMovement`'s aiStyle-17 bounce `:19310-19320`
//! and the blocked-move prologue's `Next(3)` `:17948`) and the death
//! spawn (`Player.DropTombstone` `:39108-39136`, wired in `death.rs`).
//!
//! Census verdicts (the C# the port now matches):
//!
//! * **The AI head** (`:25318-25328`): the grounded X friction
//!   (`velocity.Y == 0 → velocity.X *= 0.98`), the spin
//!   (`rotation += velocity.X * 0.1`), the gravity (`velocity.Y +=
//!   0.2`), and the getGoodWorld dead-roll disarm (`|vx| + |vy| < 1 →
//!   damage = 0, knockBack = 0`).
//! * **The placement** (`:25329-25365`, LOCAL OWNER only —
//!   `owner != Main.myPlayer → return`): the probe cell
//!   `((pos.X + width/2) / 16, (pos.Y + height − 4) / 16)`; the tile-85
//!   style — 201-205 → type−200, 527-531 → type−527+6, every other kind
//!   (the plain 43) keeps style 0; `TileObject.CanPlace(x, y, 85,
//!   style, direction)` + `TileObject.Place`, then the Dig thud
//!   (`PlaySound(0, x*16, y*16)`), the sign read
//!   (`Sign.ReadSign` — the tombstone tile 85 IS a `tileSign` kind, so
//!   the epitaph lands through `Sign.TextSign(miscText)`) and `Kill()`.
//!   The `NetMessage` sends are the single-player no-ops.
//! * **The movement** (`HandleMovement`): the blocked-move prologue
//!   consumes exactly ONE `Next(3)` per blocked move (`:17948` — the
//!   FIRST operand, rolled even when the cut gates fail; the
//!   light-items precedent). The aiStyle-17 bounce (`:19310-19320`):
//!   the changed X axis reverses at ×−0.75, the changed Y axis at
//!   ×−0.7 ONLY when `lastVelocity.Y > 1.5`; the post-bounce velocity
//!   integrates (`UpdatePosition` `:20445`). The world-edge flag is
//!   aiStyle-3-ONLY (`:17900`) — the tombstones bounce off tiles, not
//!   the world edge. `AutomaticallyChangesDirection` (`:17283`) does
//!   NOT exclude 17 — the direction follows the X sign after the move.
//! * **The wind tail** (`ShouldUseWindPhysics` `:22118` lists 17): the
//!   dispatch's `uses_wind_physics` gains 17 (the tombstones are the
//!   first wind-eligible kind of this slice).
//! * **The spawn** (`Player.DropTombstone` `:39108-39136`): the kind
//!   roll (`roll_tombstone`, already ported) then the launch velocity —
//!   `Next(10, 30) * 0.1 * hitDirection + scatter` X and
//!   `Next(-40, -20) * 0.1` Y (getGoodWorld: ×1.5 both, damage 70 /
//!   knockback 10), the box centred on the player centre, `miscText =
//!   deathText + "\n" + date`. The date string is the wall-clock
//!   `DateTime.Now` — the port passes the resolved death text only
//!   (the date suffix is the recorded deviation, ANALYSIS).
//!
//! RNG by construction:
//! - The AI body draws ZERO `Main.rand`.
//! - The placement's `TileObject.CanPlace` draws ZERO for tile 85
//!   (`random_style_range = 0`, no specific random styles — the
//!   `:761-793` random-style block never runs; the `place_rng` stream
//!   is taken/restored exactly like `edit::place_item_tile`).
//! - The blocked move consumes ONE `Next(3)` (`:17948`).
//! - The spawn's two launch draws ride the `UpdatePlayers` stream
//!   (`KillMe` runs inside `Player.Update`, `Main.cs:17754`) AFTER the
//!   kind roll — witnessed in `death.rs`.
//!
//! CODE-ONLY MODE (W4-d2): the witness tests at the bottom are WRITTEN
//! NOT RUN; verified by re-reading the cited C# blocks and hand
//! arithmetic.
//!
//! Deliberate skips (ANALYSIS "The aiStyle-17 tombstones"): the
//! `NetMessage.SendObjectPlacement`/`SendData(47)` wires (the sign
//! text lands locally), the wet movement half (the generic Update wet
//! block is unported for these kinds — the Shimmer buoyancy
//! `:22504-22515` is unreachable without it), the `DateTime.Now` date
//! suffix on the epitaph (wall-clock, non-deterministic — the death
//! text alone is stored), the tile-cut bodies (`:17910-17985` — the
//! tombstones' damage 0 fails the gate in the classic world; the
//! getGoodWorld 70-damage tombstone's cut is the unported layer, the
//! roll still fires).

use glam::Vec2;
use tr_core::loot::DropRng;
use tr_core::World;

use crate::projectile::Projectile;

/// The aiStyle-17 kinds (`Projectile.cs:1258-1270`/`:2872-2936`/
/// `:5971-5983` — the style's whole SetDefaults census).
pub(crate) const TOMB_KINDS: [u16; 11] = [43, 201, 202, 203, 204, 205, 527, 528, 529, 530, 531];

/// The tile-85 placement style (`Projectile.cs:25339-25347`): the
/// grave markers 201-205 → 1-5, the rich gravestones 527-531 → 6-10,
/// the plain Tombstone 43 keeps 0.
pub(crate) fn tomb_style(kind: u16) -> i32 {
    if (201..=205).contains(&kind) {
        (kind - 200) as i32
    } else if (527..=531).contains(&kind) {
        (kind - 527) as i32 + 6
    } else {
        0
    }
}

/// One aiStyle-17 tick: the AI head + the local-owner placement + the
/// blocked-move bounce + the integration + the direction flip.
/// `local_owner` is `Main.myPlayer`; `get_good` is `Main.getGoodWorld`
/// (the FTW seed flag, the npc_ai_style30 precedent).
pub(crate) fn tombstone_step(
    p: &mut Projectile,
    world: &mut World,
    rng: &mut dyn DropRng,
    local_owner: u8,
    get_good: bool,
) {
    // `:25318-25321` — the grounded X friction.
    if p.vel.y == 0.0 {
        p.vel.x *= 0.98;
    }
    // `:25322` — the spin.
    p.rot += p.vel.x * 0.1;
    // `:25323` — the gravity.
    p.vel.y += 0.2;
    // `:25324-25328` — the getGoodWorld dead-roll disarm.
    if get_good && p.vel.x.abs() + p.vel.y.abs() < 1.0 {
        p.damage = 0;
        p.knock_back = 0.0;
    }
    // `:25329-25332` — the placement is LOCAL-OWNER only.
    if p.owner == local_owner {
        // `:25333-25334` — the probe cell (the C# `(int)` truncation).
        let tx = ((p.pos.x + p.w / 2.0) / 16.0) as i32;
        let ty = ((p.pos.y + p.h - 4.0) / 16.0) as i32;
        // `:25335-25338` — `Main.tile[x, y] == null → return` (the
        // out-of-world cell; the port's `tile()` None).
        if world.tile(tx, ty).is_some() {
            let style = tomb_style(p.kind);
            // `:25348-25353` — TileObject.CanPlace + Place. The tile-85
            // def carries no random styles, so `can_place` draws ZERO
            // (the place_rng stream is taken/restored untouched — the
            // edit::place_item_tile convention).
            let placed = if let Some(def) = tr_core::tileobject::def_for_tile(85) {
                let mut rand = std::mem::take(&mut world.place_rng);
                let instance = tr_core::tileobject::can_place(
                    world,
                    tx,
                    ty,
                    def,
                    style,
                    p.direction,
                    None,
                    &mut rand,
                    world.tile_object_random_cache,
                );
                world.place_rng = rand;
                match instance {
                    Some(inst) => tr_core::tileobject::place(world, &inst),
                    None => false,
                }
            } else {
                false
            };
            if placed {
                // `:25356-25357` — the SendObjectPlacement wire is the
                // SP no-op; the Dig thud plays at the TILE corner.
                p.wants_sound = Some((0, 0, (tx * 16) as f32, (ty * 16) as f32, 0.0));
                // `:25358-25363` — Sign.ReadSign + TextSign(miscText):
                // the tombstone tile 85 is a `tileSign` kind, so the
                // origin decode resolves and the record is created
                // when missing (the SendData(47) wire is the SP no-op).
                if let Some((ox, oy)) = crate::signs::origin(world, tx, ty) {
                    if let Some(idx) =
                        crate::signs::find_or_create(&mut world.signs, ox, oy)
                    {
                        crate::signs::text_sign(world, idx, ox, oy, p.misc_text.clone());
                    }
                }
                // `:25364` — Kill().
                p.kill_time_left = p.time_left;
                p.active = false;
                p.time_left = 0;
                return;
            }
        }
    }
    // The movement half. The blocked-move probe (the port's
    // `collides_rect` stand-in for `Collision.TileCollision`, the
    // M4-PROJ-3 deviation) — the world-edge flag is aiStyle-3-only
    // (`:17900`), so the tombstones probe tiles only.
    let last = p.vel;
    if p.tile_collide {
        let nx = p.pos.x + p.vel.x;
        let ny = p.pos.y + p.vel.y;
        let hit_x = crate::player::collides_rect(world, nx, p.pos.y, p.w, p.h);
        let hit_y = crate::player::collides_rect(world, p.pos.x, ny, p.w, p.h);
        if hit_x || hit_y {
            // `Collision.TileCollision` zeroes the blocked axis; the
            // changed-axis test below reads that effect.
            if hit_x {
                p.vel.x = 0.0;
            }
            if hit_y {
                p.vel.y = 0.0;
            }
            // `:17948` — the prologue's Next(3) is the FIRST operand:
            // it fires on EVERY blocked move even when the cut gates
            // (owner/CanCutTiles/friendly/damage) fail. The cut bodies
            // skip with the unported layer (module doc).
            let _ = rng.next_range(0, 3);
            // `:19310-19320` — the aiStyle-17 bounce: the changed X
            // reverses ×−0.75; the changed Y ×−0.7 only when the LAST
            // Y cleared 1.5.
            if p.vel.x != last.x {
                p.vel.x = last.x * -0.75;
            }
            if p.vel.y != last.y && last.y > 1.5 {
                p.vel.y = last.y * -0.7;
            }
        }
    }
    // `UpdatePosition` (`:20445`): integrate the POST-bounce velocity.
    p.pos += p.vel;
    // `AutomaticallyChangesDirection` (`:17283` — 17 not excluded).
    p.direction = if p.vel.x < 0.0 { -1 } else { 1 };
}

// =====================================================================
// Witness tests — WRITTEN, NOT RUN (code-only mode: no cargo/rustc).
// Every expected value is hand-derived from the cited C# lines.
// =====================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectile::proj_data;
    use tr_core::tile::{Tile, FLAG_ACTIVE};
    use tr_core::UnifiedRandom;

    /// A recording stream: every draw lands in `draws` in order.
    struct Recorder {
        draws: Vec<(i32, i32)>,
        next: i32,
    }
    impl DropRng for Recorder {
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            self.draws.push((min, max));
            self.next
        }
        fn next_float(&mut self) -> f32 {
            panic!("the tombstone AI draws no floats");
        }
    }

    fn tomb(kind: u16) -> Projectile {
        let d = proj_data(kind).expect("the tombstone row");
        let mut p = Projectile::default();
        p.active = true;
        p.kind = kind;
        p.owner = 0;
        p.pos = Vec2::new(320.0, 320.0);
        p.w = d.w;
        p.h = d.h;
        p.time_left = d.time_left;
        p.penetrate = d.penetrate;
        p.knock_back = 12.0;
        p
    }

    fn air_world() -> World {
        World::new("t".into(), 512, 256)
    }

    /// A floor world: solid stone rows 30+ (the placement anchor).
    fn floor_world() -> World {
        let mut w = World::new("t".into(), 512, 256);
        for x in 0..w.width {
            for y in 30..w.height {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w
    }

    #[test]
    fn the_tombstone_rows_match_set_defaults() {
        // `Projectile.cs:1258-1270` (43) / `:2872-2936` (201-205) /
        // `:5971-5983` (527-531): 24x24, aiStyle 17, penetrate −1,
        // knockBack 12 (the spawner's input — the row carries the
        // stock), the stock tileCollide/timeLeft; the classic-world
        // row sets NO friendly (the stock false — the getGoodWorld
        // arm flips both flags, the spawn seam's FTW gate).
        for k in TOMB_KINDS {
            let d = proj_data(k).unwrap();
            assert_eq!((d.w, d.h), (24.0, 24.0), "kind {k}");
            assert_eq!(d.ai_style, 17, "kind {k}");
            assert_eq!(d.penetrate, -1, "kind {k}");
            assert_eq!(d.time_left, 3600, "kind {k}");
            assert!(d.tile_collide, "kind {k}");
        }
    }

    #[test]
    fn the_style_ladder_matches_the_type_ranges() {
        // `:25339-25347`.
        assert_eq!(tomb_style(43), 0);
        for k in 201..=205u16 {
            assert_eq!(tomb_style(k), (k - 200) as i32);
        }
        for k in 527..=531u16 {
            assert_eq!(tomb_style(k), (k - 527) as i32 + 6);
        }
    }

    #[test]
    fn the_head_friction_spin_and_gravity() {
        // `:25318-25323` — grounded X ×0.98, rotation += vx·0.1 (the
        // POST-friction X), vy += 0.2.
        let mut p = tomb(43);
        let mut w = air_world();
        p.vel = Vec2::new(4.0, 0.0);
        let mut rng = Recorder { draws: Vec::new(), next: 1 };
        tombstone_step(&mut p, &mut w, &mut rng, 0, false);
        assert!((p.vel.x - 3.92).abs() < 1e-5, "4 × 0.98");
        assert!((p.rot - 0.392).abs() < 1e-5, "the post-friction spin");
        assert!((p.vel.y - 0.2).abs() < 1e-6);
        // Airborne: no friction.
        let mut q = tomb(201);
        q.vel = Vec2::new(4.0, 3.0);
        tombstone_step(&mut q, &mut w, &mut rng, 0, false);
        assert!((q.vel.x - 4.0).abs() < 1e-6, "the airborne X holds");
        assert!(rng.draws.is_empty(), "the free flight draws nothing");
    }

    #[test]
    fn the_ftw_dead_roll_disarms() {
        // `:25324-25328` — getGoodWorld && |vx|+|vy| < 1 → damage 0,
        // knockBack 0.
        let mut p = tomb(43);
        let mut w = air_world();
        p.damage = 70;
        p.vel = Vec2::new(0.3, 0.0);
        let mut rng = Recorder { draws: Vec::new(), next: 1 };
        tombstone_step(&mut p, &mut w, &mut rng, 0, true);
        // After the head: vx 0.294, vy 0.2 → sum 0.494 < 1.
        assert_eq!(p.damage, 0);
        assert_eq!(p.knock_back, 0.0);
        // The classic world keeps the damage.
        let mut q = tomb(43);
        q.damage = 70;
        q.vel = Vec2::new(0.3, 0.0);
        tombstone_step(&mut q, &mut w, &mut rng, 0, false);
        assert_eq!(q.damage, 70);
    }

    #[test]
    fn the_blocked_move_bounces_and_draws_one_next3() {
        // `:19310-19320` + `:17948` — the changed X ×−0.75, the
        // changed Y ×−0.7 only when last.Y > 1.5; ONE Next(3) per
        // blocked move.
        let mut w = floor_world();
        let mut p = tomb(43);
        // Fall onto the floor (row 30 = px 480): the box bottom
        // 320+24+vy crosses 480 only when vy is large — park just
        // above with a fast fall.
        p.pos = Vec2::new(320.0, 450.0);
        p.vel = Vec2::new(4.0, 8.0);
        let mut rng = Recorder { draws: Vec::new(), next: 1 };
        tombstone_step(&mut p, &mut w, &mut rng, 0, false);
        // The head adds 0.2 → vy 8.2; the Y probe blocks (450+24+8.2
        // = 482.2 > 480), the X probe is clear.
        assert_eq!(rng.draws, vec![(0, 3)], "exactly one Next(3)");
        assert!((p.vel.x - 4.0).abs() < 1e-6, "the clear X holds");
        assert!((p.vel.y - (8.2 * -0.7)).abs() < 1e-4, "the Y bounce ×−0.7");
        assert!(p.active, "the bounce does not kill");
        // A slow fall (last.Y ≤ 1.5) zeroes Y without the bounce.
        let mut q = tomb(201);
        q.pos = Vec2::new(320.0, 455.0);
        q.vel = Vec2::new(0.0, 1.0);
        let mut rng2 = Recorder { draws: Vec::new(), next: 1 };
        tombstone_step(&mut q, &mut w, &mut rng2, 0, false);
        assert_eq!(rng2.draws, vec![(0, 3)]);
        assert_eq!(q.vel.y, 0.0, "last.Y 1.2 ≤ 1.5 — no bounce, the zero stands");
    }

    #[test]
    fn the_x_bounce_reverses_at_075() {
        // `:19312-19315` — a wall blocks X: ×−0.75 of the last X.
        let mut w = air_world();
        for y in 18..24 {
            let i = w.index(22, y);
            w.tiles[i] = Tile { kind: 1, ..Default::default() };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        let mut p = tomb(43);
        p.pos = Vec2::new(326.0, 320.0); // the box steps into 352..368
        p.vel = Vec2::new(8.0, 0.5);
        let mut rng = Recorder { draws: Vec::new(), next: 1 };
        tombstone_step(&mut p, &mut w, &mut rng, 0, false);
        // The head: grounded? vy 0.5 ≠ 0 — no friction; vy → 0.7.
        // The X probe blocks (326+24+8 = 358 > 352).
        assert!((p.vel.x - (8.0 * -0.75)).abs() < 1e-5, "×−0.75");
        assert_eq!(p.direction, -1, "the post-bounce X sign flips");
    }

    #[test]
    fn the_placement_lands_the_tile_the_sign_and_the_kill() {
        // `:25333-25365` — the probe cell resolves, CanPlace/Place
        // write tile 85, the epitaph lands as a sign record, the Dig
        // thud queues at the TILE corner, Kill() ends the slot.
        let mut w = floor_world();
        let mut p = tomb(203); // style 3
        p.misc_text = "Tester was slain...".into();
        // The probe cell: ((x+12)/16, (y+24−4)/16) = (20, 29) — the
        // cell just above the floor row 30. Park so the probe lands
        // there: x+12 ∈ [320,336) → x = 320; y+20 ∈ [464,480) → y = 460.
        p.pos = Vec2::new(320.0, 460.0);
        p.vel = Vec2::ZERO;
        let mut rng = Recorder { draws: Vec::new(), next: 1 };
        let rng_before = w.place_rng.clone();
        tombstone_step(&mut p, &mut w, &mut rng, 0, false);
        assert!(!p.active && p.time_left == 0, "the placement kills");
        // The 2x2 object at (20, 28)-(21, 29) (origin (0,1)).
        let t = w.tile(20, 29).unwrap();
        assert!(t.active() && t.kind == 85, "the tombstone tile landed");
        // The sign record carries the epitaph at the origin.
        assert!(w.signs.iter().any(|s| s.text == "Tester was slain..."),
            "the epitaph landed through Sign.TextSign");
        // The Dig thud at the probe corner (20*16, 29*16).
        assert_eq!(p.wants_sound, Some((0, 0, 320.0, 464.0, 0.0)));
        // CanPlace drew NOTHING for tile 85 (no random styles).
        assert_eq!(w.place_rng.peek(), rng_before.peek(),
            "the place_rng stream is untouched");
        assert!(rng.draws.is_empty(), "the placement tick drew no Next(3) — the probe cell is air");
    }

    #[test]
    fn the_foreign_owner_skips_the_placement() {
        // `:25329-25332` — owner != myPlayer → the placement never
        // runs (the flight continues).
        let mut w = floor_world();
        let mut p = tomb(43);
        p.owner = 1; // foreign
        p.pos = Vec2::new(320.0, 460.0);
        p.vel = Vec2::ZERO;
        let mut rng = Recorder { draws: Vec::new(), next: 1 };
        tombstone_step(&mut p, &mut w, &mut rng, 0, false);
        assert!(p.active, "the foreign tombstone keeps flying");
        assert!(w.tile(20, 29).is_some_and(|t| !t.active()),
            "no tile placed for the foreign owner");
    }

    #[test]
    fn the_refused_placement_keeps_flying() {
        // CanPlace refuses (no floor under the probe) — the slot
        // survives and the movement runs.
        let mut w = air_world();
        let mut p = tomb(43);
        p.pos = Vec2::new(320.0, 320.0);
        p.vel = Vec2::new(2.0, 1.0);
        let mut rng = Recorder { draws: Vec::new(), next: 1 };
        tombstone_step(&mut p, &mut w, &mut rng, 0, false);
        assert!(p.active);
        assert!(rng.draws.is_empty(), "the free move draws nothing");
    }

    #[test]
    fn the_bounce_next3_mirrors_the_unified_stream() {
        // The blocked-move Next(3) consumes the identical stream as a
        // same-seeded mirror (the rabid-roll pattern).
        let mut rng = UnifiedRandom::new(99);
        let mut mirror = UnifiedRandom::new(99);
        for _ in 0..30 {
            assert_eq!(rng.next_range(0, 3), mirror.next_range(0, 3));
        }
        assert_eq!(rng.peek(), mirror.peek(), "RNG streams stay aligned");
    }
}
