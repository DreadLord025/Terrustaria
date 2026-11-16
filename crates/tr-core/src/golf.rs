//! `Terraria.GameContent.Golf.GolfHelper` - the shot model, the club table and
//! the contact listener (`decompiled/Terraria/GameContent/Golf/GolfHelper.cs:1-474`).
//!
//! **What is here**: the parts that are pure arithmetic or a hard-coded table -
//! `FindVectorOnOval`, `CalculateShotStrength`, `GetClubProperties`,
//! `IsPlayerHoldingClub`, `IsGolfShotValid`/`ValidateShot`, `HitGolfBall`'s
//! velocity half, and the `ContactListener`'s bounce / bumper / hole-capture /
//! pass-through logic.
//!
//! **What is not**: anything needing the per-tile `TileMaterial` dampening
//! table (`GolfPhysics.{Direct,Side,Club,PassThrough}ImpactDampening` and
//! `ImpactDampeningResistanceEfficiency`). That data is an *embedded resource*
//! of `Terraria.dll` (`TileMaterials.cs:22-24` deserialises
//! `Terraria.GameContent.Metadata.MaterialData.Materials.json`), and this repo
//! has neither the DLL nor the JSON (`assets/Content/` ships only Fonts,
//! Images, Sounds). It reaches the port through [`GolfMaterials`] instead of a
//! hard-coded guess - see `docs/ANALYSIS.md` "Golf (S1-E-9)".

use glam::Vec2;

use crate::ball::{
    BallCollisionEvent, BallContactListener, BallPassThroughEvent, PhysicsProperties,
};
use crate::tile::Tile;

/// `GolfHelper.cs:471` - `new PhysicsProperties(0.3f, 0.99f)`.
pub const PHYSICS: PhysicsProperties = PhysicsProperties {
    gravity: 0.3,
    drag: 0.99,
};

/// `GolfHelper.PointsNeededForLevel1/2/3` (`:255-259`) - the Golfer shop tiers.
pub const POINTS_NEEDED_FOR_LEVEL1: i32 = 500;
pub const POINTS_NEEDED_FOR_LEVEL2: i32 = 1000;
pub const POINTS_NEEDED_FOR_LEVEL3: i32 = 2000;

/// `TileID.GolfHole` (`TileID.cs:1581-1613`).
pub const TILE_GOLF_HOLE: u16 = 476;
/// The two bumper tiles `GolfHelper.cs:41-42` pushes the ball away from.
pub const TILE_BUMPER_A: u16 = 421;
pub const TILE_BUMPER_B: u16 = 422;

/// The five per-material dampening floats (`TileGolfPhysics.cs:5-20`).
///
/// Defaults are `1.0` for every factor, which is the identity for all four
/// uses (`velocity * 1.0`, `lerp(v * 1.0, v, e) = v`), so an unknown material
/// leaves the ball's motion untouched rather than guessing a friction value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GolfPhysics {
    pub direct_impact_dampening: f32,
    pub side_impact_dampening: f32,
    pub club_impact_dampening: f32,
    pub pass_through_dampening: f32,
    pub impact_dampening_resistance_efficiency: f32,
}

impl Default for GolfPhysics {
    fn default() -> Self {
        Self {
            direct_impact_dampening: 1.0,
            side_impact_dampening: 1.0,
            club_impact_dampening: 1.0,
            pass_through_dampening: 1.0,
            impact_dampening_resistance_efficiency: 1.0,
        }
    }
}

impl GolfPhysics {
    /// Every factor the same - the `"Default"` material of `TileMaterials.cs:25`.
    pub const fn uniform(v: f32) -> Self {
        Self {
            direct_impact_dampening: v,
            side_impact_dampening: v,
            club_impact_dampening: v,
            pass_through_dampening: v,
            impact_dampening_resistance_efficiency: v,
        }
    }
}

/// `TileMaterials.GetByTileId` (`TileMaterials.cs:44-47`).
pub trait GolfMaterials {
    fn get(&self, tile_kind: u16) -> GolfPhysics;
}

/// A table that answers `GolfPhysics::default()` for every tile - the state of
/// the port until the retail `Materials.json` is available.
#[derive(Clone, Copy, Debug, Default)]
pub struct DefaultMaterials;

impl GolfMaterials for DefaultMaterials {
    fn get(&self, _tile_kind: u16) -> GolfPhysics {
        GolfPhysics::default()
    }
}

/// `GolfHelper.ClubProperties` (`:14-21`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClubProperties {
    pub minimum_strength: Vec2,
    pub maximum_strength: Vec2,
    pub rough_land_resistance: f32,
}

/// `GolfHelper.ShotStrength` (`:23-30`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShotStrength {
    pub absolute_strength: f32,
    pub relative_strength: f32,
    pub rough_land_resistance: f32,
}

/// `GolfHelper.FindVectorOnOval` (`:272-279`).
pub fn find_vector_on_oval(vector: Vec2, radius: Vec2) -> Vec2 {
    if radius.x.abs() < 0.0001 || radius.y.abs() < 0.0001 {
        return Vec2::ZERO;
    }
    // `Vector2.Normalize` is `v / v.Length()`; a zero input yields NaN as in XNA.
    let v = vector / radius;
    v / v.length() * radius
}

/// `GolfHelper.CalculateShotStrength` (`:281-289`).
///
/// The C# opens with a discarded `Vector2.Normalize(shotVector)` - a no-op
/// (`Vector2` is a struct, so the result is thrown away), kept as a comment
/// rather than a line because it cannot change behaviour.
pub fn calculate_shot_strength(shot_vector: Vec2, club: ClubProperties) -> ShotStrength {
    let value = shot_vector.length();
    let max = find_vector_on_oval(shot_vector, club.maximum_strength).length();
    let min = find_vector_on_oval(shot_vector, club.minimum_strength).length();
    let num3 = value.clamp(min, max);
    ShotStrength {
        absolute_strength: num3 * 32.0,
        relative_strength: ((num3 - min) / (max - min)).max(0.001),
        rough_land_resistance: club.rough_land_resistance,
    }
}

/// `GolfHelper.GetClubProperties` (`:331-354`) - the 12-entry club table.
pub fn get_club_properties(item_id: i32) -> ClubProperties {
    let v = Vec2::new(0.25, 0.25);
    let (min, max, rough) = match item_id {
        4039 | 4587 | 4591 | 4595 => (v, Vec2::ONE, 0.0),
        4092 | 4588 | 4592 | 4596 => (Vec2::ZERO, v, 0.0),
        4093 | 4589 | 4593 | 4597 => (v, Vec2::new(0.65, 1.5), 1.0),
        4094 | 4590 | 4594 | 4598 => (v, Vec2::new(1.5, 0.65), 0.0),
        _ => {
            return ClubProperties {
                minimum_strength: Vec2::ZERO,
                maximum_strength: Vec2::ZERO,
                rough_land_resistance: 0.0,
            }
        }
    };
    ClubProperties {
        minimum_strength: min,
        maximum_strength: max,
        rough_land_resistance: rough,
    }
}

/// `GolfHelper.IsPlayerHoldingClub` (`:291-303`).
pub fn is_player_holding_club(held_item_type: i32) -> bool {
    held_item_type == 4039
        || (4092..=4094).contains(&held_item_type)
        || (4587..=4598).contains(&held_item_type)
}

/// `GolfHelper.IsGolfShotValid` (`:392-404`).
///
/// `player_bottom` is `Player.Bottom`; `direction` is `Player.direction`.
pub fn is_golf_shot_valid(ball_center: Vec2, player_bottom: Vec2, direction: i32) -> bool {
    let mut v = ball_center - player_bottom;
    if direction == -1 {
        v.x *= -1.0;
    }
    v.x >= -16.0 && v.x <= 32.0 && v.y <= 16.0 && v.y >= -16.0
}

/// `GolfHelper.ValidateShot` (`:406-432`) - the legality box plus the shot-angle
/// clamp to `[-1.5207964181900024, 0]`. Returns `(valid, shot_vector)` because
/// the C# writes `shotVector` through a `ref` before deciding.
pub fn validate_shot(
    ball_center: Vec2,
    player_bottom: Vec2,
    direction: i32,
    mut shot_vector: Vec2,
) -> (bool, Vec2) {
    let mut v = ball_center - player_bottom;
    if direction == -1 {
        v.x *= -1.0;
        shot_vector.x *= -1.0;
    }
    let rot = shot_vector.y.atan2(shot_vector.x);
    if rot > 0.0 {
        shot_vector = shot_vector.length() * Vec2::new(0.0f32.cos(), 0.0f32.sin());
    } else if rot < -1.5207964 {
        const A: f32 = -1.5207964181900024;
        shot_vector = shot_vector.length() * Vec2::new(A.cos(), A.sin());
    }
    if direction == -1 {
        shot_vector.x *= -1.0;
    }
    let valid = v.x >= -16.0 && v.x <= 32.0 && v.y <= 16.0 && v.y >= -16.0;
    (valid, shot_vector)
}

/// `GolfHelper.HitGolfBall` (`:434-457`) - the velocity half. The
/// `timeLeft = 18000` / `ai[1]++` / `localAI[1] = 1` / `RecordSwing` writes are
/// projectile-side and land with the 721/722 AIs.
pub fn hit_golf_ball_velocity(
    velocity: Vec2,
    rough_land_resistance: f32,
    tile_below: Option<Tile>,
    materials: &dyn GolfMaterials,
) -> Vec2 {
    let Some(tile) = tile_below else {
        return velocity;
    };
    if !tile.active() {
        return velocity;
    }
    let gp = materials.get(tile.kind);
    let dampened = velocity * gp.club_impact_dampening;
    let t = gp.impact_dampening_resistance_efficiency * rough_land_resistance;
    dampened.lerp(velocity, t)
}

/// `GolfHelper.ContactListener` (`:32-253`). The effect calls
/// (`PutBallInCup`'s scoring/net/wiring, the dust bursts, the sounds, the chat
/// text) are the caller's - this struct only produces the physics decisions and
/// reports a [`GolfContact`] for each one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GolfContact {
    /// `:57-69` - the ball dropped into the cup at this tile.
    InCup { tile_x: i32, tile_y: i32 },
    /// `:71-74` - `BreaksFromToyBreaker` + `TileID.Sets.BreaksToys`.
    BreaksToy,
}

/// The listener's state: it holds the material table and collects contacts.
///
/// `IBallContactListener`'s methods are non-`mut` in the C# (the static
/// `GolfHelper.Listener` is a single shared instance), so the contact log is
/// behind a `RefCell` rather than taking `&mut self`.
pub struct GolfContactListener<'a> {
    pub materials: &'a dyn GolfMaterials,
    contacts: core::cell::RefCell<Vec<GolfContact>>,
    wants_bounce_dust: core::cell::Cell<bool>,
}

impl<'a> GolfContactListener<'a> {
    pub fn new(materials: &'a dyn GolfMaterials) -> Self {
        Self {
            materials,
            contacts: core::cell::RefCell::new(Vec::new()),
            wants_bounce_dust: core::cell::Cell::new(false),
        }
    }

    /// The contacts raised since the last [`Self::take_contacts`].
    pub fn take_contacts(&self) -> Vec<GolfContact> {
        core::mem::take(&mut *self.contacts.borrow_mut())
    }

    /// Whether `:75-81`'s bounce-dust condition held on the last collision.
    pub fn wants_bounce_dust(&self) -> bool {
        self.wants_bounce_dust.get()
    }
}

impl<E: Copy> BallContactListener<E> for GolfContactListener<'_> {
    /// `GolfHelper.cs:34-82`.
    fn on_collision(
        &self,
        _properties: &PhysicsProperties,
        _position: &mut Vec2,
        velocity: &mut Vec2,
        collision: &mut BallCollisionEvent<E>,
    ) {
        let gp = self.materials.get(collision.tile.kind);
        // `:37` - the side/direct blend along the contact normal.
        *velocity = *velocity * gp.side_impact_dampening
            + collision.normal
                * velocity.dot(collision.normal)
                * (gp.direct_impact_dampening - gp.side_impact_dampening);

        match collision.tile.kind {
            // `:41-56` - the bumpers push along the tangent.
            TILE_BUMPER_A | TILE_BUMPER_B => {
                let num2 = 2.5 * collision.time_scale;
                let mut tangent = Vec2::new(-collision.normal.y, collision.normal.x);
                if collision.tile.kind == TILE_BUMPER_B {
                    tangent = -tangent;
                }
                let num3 = velocity.dot(tangent);
                if num3 < num2 {
                    *velocity += tangent * (num2 - num3).clamp(0.0, num2 * 0.5);
                }
            }
            // `:57-69` - the hole captures when the normal points up and the
            // per-sub-step speed is <= 100.
            TILE_GOLF_HOLE => {
                let num = velocity.length() / collision.time_scale;
                if !(collision.normal.y > -0.01) && !(num > 100.0) {
                    *velocity *= 0.0;
                    self.contacts.borrow_mut().push(GolfContact::InCup {
                        tile_x: collision.tile_x,
                        tile_y: collision.tile_y,
                    });
                }
            }
            _ => {}
        }

        // `:75-81` - the small upward bounce's dust.
        self.wants_bounce_dust
            .set(velocity.y < -0.3 && velocity.y > -2.0 && velocity.length() > 1.0);
    }

    /// `GolfHelper.cs:125-147`.
    fn on_pass_through(
        &self,
        _properties: &PhysicsProperties,
        _position: &mut Vec2,
        velocity: &mut Vec2,
        angular_velocity: &mut f32,
        pass_through: &mut BallPassThroughEvent<E>,
    ) {
        use crate::ball::BallPassThroughType;
        match pass_through.kind {
            BallPassThroughType::Water => {
                *velocity *= 0.91;
                *angular_velocity *= 0.91;
            }
            BallPassThroughType::Honey => {
                *velocity *= 0.8;
                *angular_velocity *= 0.8;
            }
            BallPassThroughType::Tile => {
                let Some(tile) = pass_through.tile else {
                    return;
                };
                let d = self.materials.get(tile.kind).pass_through_dampening;
                *velocity *= d;
                *angular_velocity *= d;
            }
            // `:144-145` - lava is an explicit no-op.
            BallPassThroughType::Lava | BallPassThroughType::None => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ball() -> ClubProperties {
        get_club_properties(4039)
    }

    #[test]
    fn the_club_table_covers_all_twelve_clubs() {
        // `:331-354` - four rows repeated across three id blocks. The iron
        // (4039) is the 0.25/1.0 driver, the putter (4092) is 0/0.25, the
        // wedge (4093) is the only rough-land club, the driver (4094) is wide.
        assert_eq!(ball().maximum_strength, Vec2::ONE);
        assert_eq!(ball().rough_land_resistance, 0.0);

        let putter = get_club_properties(4092);
        assert_eq!(putter.minimum_strength, Vec2::ZERO);
        assert_eq!(putter.maximum_strength, Vec2::new(0.25, 0.25));

        let wedge = get_club_properties(4093);
        assert_eq!(wedge.maximum_strength, Vec2::new(0.65, 1.5));
        assert_eq!(
            wedge.rough_land_resistance, 1.0,
            "only the wedge resists rough land"
        );

        // Every id block repeats the same four rows.
        for (a, b) in [(4039, 4587), (4092, 4588), (4093, 4589), (4094, 4590)] {
            assert_eq!(get_club_properties(a), get_club_properties(b));
        }
        // An unknown id is `default`, i.e. all-zero.
        let none = get_club_properties(1);
        assert_eq!(none.maximum_strength, Vec2::ZERO);
        assert_eq!(none.minimum_strength, Vec2::ZERO);
    }

    #[test]
    fn holding_a_club_follows_the_csharp_id_ranges() {
        assert!(is_player_holding_club(4039));
        assert!(is_player_holding_club(4092) && is_player_holding_club(4094));
        assert!(!is_player_holding_club(4091) && !is_player_holding_club(4095));
        assert!(is_player_holding_club(4587) && is_player_holding_club(4598));
        assert!(!is_player_holding_club(4586) && !is_player_holding_club(4599));
    }

    #[test]
    fn the_oval_projection_returns_a_point_on_the_oval() {
        // `:272-279` - `normalize(v / r) * r`, so the result satisfies
        // `(x/rx)^2 + (y/ry)^2 == 1`.
        let r = Vec2::new(1.5, 0.65);
        let p = find_vector_on_oval(Vec2::new(3.0, 1.0), r);
        let check = (p.x / r.x).powi(2) + (p.y / r.y).powi(2);
        assert!((check - 1.0).abs() < 1e-5, "on-oval check = {check}");
        // A degenerate radius is zero, not NaN (`:274-277`).
        assert_eq!(
            find_vector_on_oval(Vec2::new(1.0, 1.0), Vec2::ZERO),
            Vec2::ZERO
        );
        assert_eq!(
            find_vector_on_oval(Vec2::new(1.0, 1.0), Vec2::new(0.0, 1.0)),
            Vec2::ZERO
        );
    }

    #[test]
    fn shot_strength_is_clamped_between_the_club_ovals() {
        // `:281-289`: `absolute = clamp(len, min, max) * 32`,
        // `relative = max((abs/32 - min) / (max - min), 0.001)`.
        let club = ball();
        // A short aim (0.05) is below the min oval, so it clamps up to min.
        let short = calculate_shot_strength(Vec2::new(0.05, 0.0), club);
        let min = find_vector_on_oval(Vec2::new(0.05, 0.0), club.minimum_strength).length();
        assert!((short.absolute_strength - min * 32.0).abs() < 1e-4);
        assert!(
            (short.relative_strength - 0.001).abs() < 1e-6,
            "the floor is 0.001, got {}",
            short.relative_strength
        );

        // A huge aim clamps to the max oval.
        let long = calculate_shot_strength(Vec2::new(9.0, 0.0), club);
        let max = find_vector_on_oval(Vec2::new(9.0, 0.0), club.maximum_strength).length();
        assert!((long.absolute_strength - max * 32.0).abs() < 1e-4);
        assert!(
            (long.relative_strength - 1.0).abs() < 1e-6,
            "at max, relative = 1"
        );
    }

    #[test]
    fn the_legality_box_is_the_csharp_rectangle() {
        // `:392-404` - x in [-16, 32], y in [-16, 16], measured from
        // `Player.Bottom`, mirrored for a left-facing player.
        let bottom = Vec2::new(100.0, 100.0);
        assert!(is_golf_shot_valid(Vec2::new(100.0, 100.0), bottom, 1));
        assert!(
            is_golf_shot_valid(Vec2::new(116.0, 100.0), bottom, 1),
            "x = +16"
        );
        assert!(
            is_golf_shot_valid(Vec2::new(132.0, 100.0), bottom, 1),
            "x = +32"
        );
        assert!(
            !is_golf_shot_valid(Vec2::new(133.0, 100.0), bottom, 1),
            "x = +33"
        );
        assert!(
            is_golf_shot_valid(Vec2::new(84.0, 100.0), bottom, 1),
            "x = -16"
        );
        assert!(
            !is_golf_shot_valid(Vec2::new(83.0, 100.0), bottom, 1),
            "x = -17"
        );
        assert!(
            is_golf_shot_valid(Vec2::new(100.0, 116.0), bottom, 1),
            "y = +16"
        );
        assert!(
            !is_golf_shot_valid(Vec2::new(100.0, 117.0), bottom, 1),
            "y = +17"
        );
        // Facing left mirrors x: the ball at +20 is now out of the box.
        assert!(!is_golf_shot_valid(Vec2::new(120.0, 100.0), bottom, -1));
        assert!(is_golf_shot_valid(Vec2::new(80.0, 100.0), bottom, -1));
    }

    #[test]
    fn the_shot_angle_clamps_to_the_csharp_bounds() {
        // `:414-422` - an upward shot (rotation > 0) is flattened to 0 rad; a
        // shot steeper than -1.5208 rad is clamped to exactly that angle.
        let bottom = Vec2::new(0.0, 0.0);
        let (valid, v) = validate_shot(Vec2::new(0.0, 0.0), bottom, 1, Vec2::new(1.0, 1.0));
        assert!(valid);
        assert!(v.y.abs() < 1e-6, "flattened to horizontal, got {v:?}");
        assert!((v.x - 2.0f32.sqrt()).abs() < 1e-5, "length preserved");

        let steep = Vec2::new(-1.0, -10.0);
        let (_, clamped) = validate_shot(Vec2::new(0.0, 0.0), bottom, 1, steep);
        let angle = clamped.y.atan2(clamped.x);
        assert!((angle - (-1.5207964)).abs() < 1e-5, "angle = {angle}");
        assert!(
            (clamped.length() - steep.length()).abs() < 1e-4,
            "length preserved"
        );
    }

    #[test]
    fn a_bumper_pushes_the_ball_along_its_tangent() {
        // `:41-56` - `num2 = 2.5 * timeScale`; when the tangential speed is
        // below it, the ball gains `clamp(num2 - num3, 0, num2/2)`.
        struct Probe<'a>(GolfContactListener<'a>);
        let mats = DefaultMaterials;
        let mut vel = Vec2::new(0.0, 0.0);
        let mut pos = Vec2::ZERO;
        let mut collision = BallCollisionEvent {
            time_scale: 1.0,
            normal: Vec2::new(0.0, -1.0),
            impact_point: Vec2::ZERO,
            tile: Tile {
                kind: TILE_BUMPER_A,
                ..Default::default()
            },
            tile_x: 0,
            tile_y: 0,
            entity: (),
        };
        let listener = Probe(GolfContactListener::new(&mats));
        listener
            .0
            .on_collision(&PHYSICS, &mut pos, &mut vel, &mut collision);
        // tangent = (-normal.y, normal.x) = (1, 0); num2 = 2.5, num3 = 0,
        // so the push is clamp(2.5, 0, 1.25) = 1.25 along +x.
        assert!((vel.x - 1.25).abs() < 1e-6, "vx = {}", vel.x);
        assert!(vel.y.abs() < 1e-6);
    }

    #[test]
    fn the_hole_captures_only_when_the_normal_points_up_and_the_ball_is_slow() {
        // `:57-69` - `!(normal.Y > -0.01) && !(speed/timeScale > 100)`.
        let mats = DefaultMaterials;
        let mut pos = Vec2::ZERO;

        // Normal up (-1), slow: captured.
        let listener = GolfContactListener::new(&mats);
        let mut vel = Vec2::new(1.0, 0.0);
        let mut c = BallCollisionEvent {
            time_scale: 1.0,
            normal: Vec2::new(0.0, -1.0),
            impact_point: Vec2::ZERO,
            tile: Tile {
                kind: TILE_GOLF_HOLE,
                ..Default::default()
            },
            tile_x: 12,
            tile_y: 34,
            entity: (),
        };
        listener.on_collision(&PHYSICS, &mut pos, &mut vel, &mut c);
        assert_eq!(vel, Vec2::ZERO, "the cup stops the ball");
        assert_eq!(
            listener.take_contacts(),
            vec![GolfContact::InCup {
                tile_x: 12,
                tile_y: 34
            }]
        );

        // Too fast: not captured.
        let listener = GolfContactListener::new(&mats);
        let mut fast = Vec2::new(200.0, 0.0);
        listener.on_collision(&PHYSICS, &mut pos, &mut fast, &mut c);
        assert!(
            listener.take_contacts().is_empty(),
            "101+ px/step overshoots the cup"
        );

        // Normal pointing sideways: not captured (the ball rolls over the rim).
        let listener = GolfContactListener::new(&mats);
        let mut side = Vec2::new(1.0, 0.0);
        c.normal = Vec2::new(1.0, 0.0);
        listener.on_collision(&PHYSICS, &mut pos, &mut side, &mut c);
        assert!(listener.take_contacts().is_empty());
    }

    #[test]
    fn liquids_dampen_by_the_csharp_factors_and_lava_does_not() {
        // `:125-147` - water 0.91, honey 0.8, lava an explicit no-op.
        use crate::ball::BallPassThroughType;
        let mats = DefaultMaterials;
        let listener = GolfContactListener::new(&mats);
        let mut pos = Vec2::ZERO;
        let mut av = 1.0f32;

        let mut water = BallPassThroughEvent {
            time_scale: 1.0,
            tile: None,
            tile_x: 0,
            tile_y: 0,
            entity: (),
            kind: BallPassThroughType::Water,
        };
        let mut v = Vec2::new(100.0, 100.0);
        listener.on_pass_through(&PHYSICS, &mut pos, &mut v, &mut av, &mut water);
        assert!(
            (v.x - 91.0).abs() < 1e-4 && (v.y - 91.0).abs() < 1e-4,
            "{v:?}"
        );
        assert!((av - 0.91).abs() < 1e-6, "av = {av}");

        av = 1.0;
        let mut honey = BallPassThroughEvent {
            time_scale: 1.0,
            tile: None,
            tile_x: 0,
            tile_y: 0,
            entity: (),
            kind: BallPassThroughType::Honey,
        };
        let mut v = Vec2::new(100.0, 100.0);
        listener.on_pass_through(&PHYSICS, &mut pos, &mut v, &mut av, &mut honey);
        assert!((v.x - 80.0).abs() < 1e-4, "{v:?}");
        assert!((av - 0.8).abs() < 1e-6);

        av = 1.0;
        let mut lava = BallPassThroughEvent {
            time_scale: 1.0,
            tile: None,
            tile_x: 0,
            tile_y: 0,
            entity: (),
            kind: BallPassThroughType::Lava,
        };
        let mut v = Vec2::new(100.0, 100.0);
        listener.on_pass_through(&PHYSICS, &mut pos, &mut v, &mut av, &mut lava);
        assert_eq!(v, Vec2::new(100.0, 100.0), "lava is a no-op");
        assert_eq!(av, 1.0);
    }

    #[test]
    fn the_club_dampening_blends_by_the_rough_land_resistance() {
        // `:443` - `lerp(v * clubImpact, v, efficiency * rough)`. With the
        // identity material the lerp cannot change anything, so the test uses
        // a half-dampening table to prove both ends of the blend.
        struct Half;
        impl GolfMaterials for Half {
            fn get(&self, _k: u16) -> GolfPhysics {
                GolfPhysics {
                    club_impact_dampening: 0.5,
                    impact_dampening_resistance_efficiency: 1.0,
                    ..GolfPhysics::default()
                }
            }
        }
        let tile = Tile {
            kind: 1,
            flags: crate::tile::FLAG_ACTIVE,
            ..Default::default()
        };
        let v = Vec2::new(100.0, 0.0);

        // rough = 0: fully dampened.
        let hard = hit_golf_ball_velocity(v, 0.0, Some(tile), &Half);
        assert!((hard.x - 50.0).abs() < 1e-5, "{hard:?}");
        // rough = 1: fully restored.
        let rough = hit_golf_ball_velocity(v, 1.0, Some(tile), &Half);
        assert!((rough.x - 100.0).abs() < 1e-5, "{rough:?}");
        // No tile below: untouched.
        assert_eq!(hit_golf_ball_velocity(v, 1.0, None, &Half), v);
        // An inactive tile is skipped (`tile.active()` gate, `:440`).
        let inert = Tile {
            kind: 1,
            ..Default::default()
        };
        assert_eq!(hit_golf_ball_velocity(v, 0.0, Some(inert), &Half), v);
    }
}
