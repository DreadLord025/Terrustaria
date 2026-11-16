//! `Terraria.Physics.BallCollision` - the generic ball physics engine
//! (`decompiled/Terraria/Physics/BallCollision.cs:1-383`).
//!
//! The engine is self-contained geometry: it reads only the world's tile flags
//! (`nactive`, `slope`, `halfBrick`, `liquid`, `liquidType`, `tileSolid`,
//! `tileSolidTop`, `frameY`) and reports contacts to a listener. Every material
//! constant lives in the listener, so this module needs no content data.
//!
//! The listener is a per-call `&dyn` (the C# shares one static
//! `GolfHelper.Listener`); the entity rides as an opaque `E: Copy` payload that
//! the events carry through, mirroring `BallCollisionEvent.Entity`.

use glam::Vec2;

use crate::tile::{LiquidKind, Tile};
use crate::tiledata::{TILE_SOLID, TILE_SOLID_TOP};
use crate::world::World;

/// `PhysicsProperties.cs:5-11`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PhysicsProperties {
    pub gravity: f32,
    pub drag: f32,
}

/// `BallState.cs:3-7`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BallState {
    Resting,
    Moving,
    OutOfBounds,
}

/// `BallStepResult.cs:5-21`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BallStepResult {
    pub state: BallState,
}

impl BallStepResult {
    pub fn out_of_bounds() -> Self {
        Self {
            state: BallState::OutOfBounds,
        }
    }
    pub fn moving() -> Self {
        Self {
            state: BallState::Moving,
        }
    }
    pub fn resting() -> Self {
        Self {
            state: BallState::Resting,
        }
    }
}

/// The `Entity` fields `Step` reads and writes (`BallCollision.cs:26-28, 85-87`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BallBody {
    pub position: Vec2,
    pub velocity: Vec2,
    /// `entity.Size` - the ball's full width/height (a circle of `size.x / 2`).
    pub size: Vec2,
}

/// `BallPassThroughType.cs:3-9`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BallPassThroughType {
    None,
    Water,
    Honey,
    Lava,
    Tile,
}

/// `BallPassThroughEvent.cs:5-17`.
#[derive(Clone, Copy, Debug)]
pub struct BallPassThroughEvent<E> {
    pub time_scale: f32,
    pub tile: Option<Tile>,
    pub tile_x: i32,
    pub tile_y: i32,
    pub entity: E,
    pub kind: BallPassThroughType,
}

/// `BallCollisionEvent.cs:5-17`.
#[derive(Clone, Copy, Debug)]
pub struct BallCollisionEvent<E> {
    pub time_scale: f32,
    pub normal: Vec2,
    pub impact_point: Vec2,
    pub tile: Tile,
    pub tile_x: i32,
    pub tile_y: i32,
    pub entity: E,
}

/// `IBallContactListener.cs:5-9`. `position`/`velocity` are `ref` in the C#;
/// `collision` is also `ref` (`BallCollision.cs:74` re-reads `Normal` after it).
pub trait BallContactListener<E> {
    fn on_collision(
        &self,
        properties: &PhysicsProperties,
        position: &mut Vec2,
        velocity: &mut Vec2,
        collision: &mut BallCollisionEvent<E>,
    );

    #[allow(clippy::too_many_arguments)]
    fn on_pass_through(
        &self,
        properties: &PhysicsProperties,
        position: &mut Vec2,
        velocity: &mut Vec2,
        angular_velocity: &mut f32,
        pass_through: &mut BallPassThroughEvent<E>,
    );
}

/// `BallCollision.TileEdges` (`:10-22`).
mod edges {
    pub const NONE: u32 = 0;
    pub const TOP: u32 = 1;
    pub const BOTTOM: u32 = 2;
    pub const LEFT: u32 = 4;
    pub const RIGHT: u32 = 8;
    pub const TOP_LEFT_SLOPE: u32 = 0x10;
    pub const TOP_RIGHT_SLOPE: u32 = 0x20;
    pub const BOTTOM_LEFT_SLOPE: u32 = 0x40;
    pub const BOTTOM_RIGHT_SLOPE: u32 = 0x80;
}

/// `Tile.nactive()` (`Tile.cs:245-252`): active and not framed off.
#[inline]
fn nactive(t: &Tile) -> bool {
    t.active() && !t.inactive()
}

/// `Main.tileSolid[type]`.
#[inline]
fn tile_solid(kind: u16) -> bool {
    TILE_SOLID.has(kind)
}

/// `Main.tileSolidTop[type]`.
#[inline]
fn tile_solid_top(kind: u16) -> bool {
    TILE_SOLID_TOP.has(kind)
}

/// `Vector2.Normalize(v)` (`v / v.Length()`): a zero vector yields NaN, exactly
/// as XNA does. `glam`'s `normalize_or_zero` would silently differ.
#[inline]
fn normalize(v: Vec2) -> Vec2 {
    v / v.length()
}

/// `Vector2.Reflect(vector, normal)`.
#[inline]
fn reflect(v: Vec2, n: Vec2) -> Vec2 {
    v - 2.0 * v.dot(n) * n
}

/// `BallCollision.Step` (`:24-89`).
pub fn step<E: Copy>(
    properties: &PhysicsProperties,
    body: &mut BallBody,
    entity_angular_velocity: &mut f32,
    entity: E,
    world: &World,
    listener: &dyn BallContactListener<E>,
) -> BallStepResult {
    // The C# copies into locals and only writes the entity back at `:85-87`,
    // which is what makes the `OutOfBounds` early return leave it untouched.
    let mut position = body.position;
    let mut velocity = body.velocity;
    let size = body.size;
    let mut angular = *entity_angular_velocity;

    let num2 = size.x * 0.5;
    angular *= properties.drag;
    velocity *= properties.drag;
    let mut num3 = velocity.length();
    if num3 > 1000.0 {
        velocity = 1000.0 * normalize(velocity);
        num3 = 1000.0;
    }
    let num4 = std::cmp::max(1, (num3 / 2.0).ceil() as i32);
    let num5 = 1.0 / num4 as f32;
    velocity *= num5;
    angular *= num5;
    let num6 = properties.gravity / (num4 * num4) as f32;

    let mut flag = false;
    for _ in 0..num4 {
        velocity.y += num6;
        if let Some((kind, tile, tx, ty)) = check_for_pass_through(position + size * 0.5, world) {
            if kind == BallPassThroughType::Tile
                && tile_solid(tile.kind)
                && !tile_solid_top(tile.kind)
            {
                velocity *= 0.0;
                angular *= 0.0;
                flag = true;
            } else {
                let mut pass_through = BallPassThroughEvent {
                    time_scale: num5,
                    tile: Some(tile),
                    tile_x: tx,
                    tile_y: ty,
                    entity,
                    kind,
                };
                listener.on_pass_through(
                    properties,
                    &mut position,
                    &mut velocity,
                    &mut angular,
                    &mut pass_through,
                );
            }
        }
        position += velocity;
        if !is_ball_in_world(position, size, world) {
            return BallStepResult::out_of_bounds();
        }
        if let Some((collision_point, collision_tile, ctx, cty)) =
            get_closest_edge_to_circle(position, size, velocity, world)
        {
            let normal = normalize(position + size * 0.5 - collision_point);
            position = collision_point + normal * (num2 + 0.0001) - size * 0.5;
            let mut collision = BallCollisionEvent {
                time_scale: num5,
                normal,
                impact_point: collision_point,
                tile: collision_tile,
                tile_x: ctx,
                tile_y: cty,
                entity,
            };
            flag = true;
            velocity = reflect(velocity, collision.normal);
            listener.on_collision(properties, &mut position, &mut velocity, &mut collision);
            angular = (collision.normal.x * velocity.y - collision.normal.y * velocity.x) / num2;
        }
    }

    velocity /= num5;
    angular /= num5;
    let mut result = BallStepResult::moving();
    if flag
        && velocity.x > -0.01
        && velocity.x < 0.01
        && velocity.y <= 0.0
        && velocity.y > 0.0 - properties.gravity
    {
        result = BallStepResult::resting();
    }
    body.position = position;
    body.velocity = velocity;
    *entity_angular_velocity = angular;
    result
}

/// `BallCollision.CheckForPassThrough` (`:91-123`).
fn check_for_pass_through(
    center: Vec2,
    world: &World,
) -> Option<(BallPassThroughType, Tile, i32, i32)> {
    // `center.ToTileCoordinates()` is `(int)vec.X >> 4` (`Utils.cs`): truncate
    // toward zero, then an arithmetic shift - not a floor divide.
    let tx = (center.x as i32) >> 4;
    let ty = (center.y as i32) >> 4;
    let tile = *world.tile(tx, ty)?;
    if nactive(&tile) {
        if is_position_inside_tile(center, tx, ty, &tile) {
            return Some((BallPassThroughType::Tile, tile, tx, ty));
        }
        return None;
    }
    if tile.liquid > 0 {
        let surface = (ty + 1) as f32 * 16.0 - tile.liquid as f32 / 255.0 * 16.0;
        // `tile.liquidType()` 1 = lava, 2 = honey, default = water
        // (`Tile.cs:216-237`) - shimmer (3) therefore reports as water.
        let kind = match tile.liquid_kind() {
            LiquidKind::Lava => BallPassThroughType::Lava,
            LiquidKind::Honey => BallPassThroughType::Honey,
            _ => BallPassThroughType::Water,
        };
        if surface < center.y {
            return Some((kind, tile, tx, ty));
        }
        return None;
    }
    None
}

/// `BallCollision.IsPositionInsideTile` (`:125-141`).
fn is_position_inside_tile(position: Vec2, tx: i32, ty: i32, tile: &Tile) -> bool {
    if tile.slope() == 0 && !tile.half_brick() {
        return true;
    }
    let v = position / 16.0 - Vec2::new(tx as f32, ty as f32);
    match tile.slope() {
        0 => v.y > 0.5,
        1 => v.y > v.x,
        2 => v.y > 1.0 - v.x,
        3 => v.y < 1.0 - v.x,
        4 => v.y < v.x,
        _ => false,
    }
}

/// `BallCollision.IsBallInWorld` (`:143-150`).
fn is_ball_in_world(position: Vec2, size: Vec2, world: &World) -> bool {
    if position.x > 32.0
        && position.y > 32.0
        && position.x + size.x < world.width as f32 * 16.0 - 32.0
    {
        return position.y + size.y < world.height as f32 * 16.0 - 32.0;
    }
    false
}

/// `BallCollision.GetClosestEdgeToCircle` (`:152-180`).
fn get_closest_edge_to_circle(
    position: Vec2,
    size: Vec2,
    velocity: Vec2,
    world: &World,
) -> Option<(Vec2, Tile, i32, i32)> {
    let bounds = get_tile_bounds(position, size);
    let center = position + size * 0.5;
    let mut e = edges::NONE;
    e |= if !(velocity.y < 0.0) {
        edges::TOP
    } else {
        edges::BOTTOM
    };
    e |= if !(velocity.x < 0.0) {
        edges::LEFT
    } else {
        edges::RIGHT
    };
    e |= if !(velocity.y > velocity.x) {
        edges::TOP_RIGHT_SLOPE
    } else {
        edges::BOTTOM_LEFT_SLOPE
    };
    e |= if !(velocity.y > 0.0 - velocity.x) {
        edges::TOP_LEFT_SLOPE
    } else {
        edges::BOTTOM_RIGHT_SLOPE
    };

    let mut best = f32::MAX;
    let mut hit: Option<(Vec2, Tile, i32, i32)> = None;
    for i in bounds.0..bounds.1 {
        for j in bounds.2..bounds.3 {
            if let Some((closest, dist)) = get_collision_point_for_tile(e, i, j, center, world) {
                if dist < best && !(velocity.dot(center - closest) > 0.0) {
                    best = dist;
                    // The C# stores `Main.tile[i, j]`; the tile was already
                    // proven non-null by the per-tile test.
                    hit = world.tile(i, j).map(|t| (closest, *t, i, j));
                }
            }
        }
    }
    let num2 = size.x / 2.0;
    if best < num2 * num2 {
        hit
    } else {
        None
    }
}

/// `BallCollision.GetCollisionPointForTile` (`:182-229`).
fn get_collision_point_for_tile(
    mut edges_to_test: u32,
    x: i32,
    y: i32,
    center: Vec2,
    world: &World,
) -> Option<(Vec2, f32)> {
    let tile = *world.tile(x, y)?;
    if !nactive(&tile) || (!tile_solid(tile.kind) && !tile_solid_top(tile.kind)) {
        return None;
    }
    if !tile_solid(tile.kind) && tile_solid_top(tile.kind) && tile.frame_y != 0 {
        return None;
    }
    if tile_solid_top(tile.kind) {
        edges_to_test &= edges::TOP | edges::BOTTOM_LEFT_SLOPE | edges::BOTTOM_RIGHT_SLOPE;
    }
    let tile_position = Vec2::new(x as f32 * 16.0, y as f32 * 16.0);

    let mut found = false;
    let mut closest = Vec2::ZERO;
    let mut distance_squared = 0.0f32;

    if let Some(edge) = get_slope_edge(&mut edges_to_test, &tile, tile_position) {
        closest = closest_point_on_line_segment(center, edge);
        distance_squared = (closest - center).length_squared();
        found = true;
    }
    if let Some(edge) = get_top_or_bottom_edge(edges_to_test, x, y, tile_position, &tile, world) {
        let p = closest_point_on_line_segment(center, edge);
        let d = (p - center).length_squared();
        if !found || d < distance_squared {
            distance_squared = d;
            closest = p;
        }
        found = true;
    }
    if let Some(edge) = get_left_or_right_edge(edges_to_test, x, y, tile_position, &tile, world) {
        let p = closest_point_on_line_segment(center, edge);
        let d = (p - center).length_squared();
        if !found || d < distance_squared {
            distance_squared = d;
            closest = p;
        }
        found = true;
    }
    if found {
        Some((closest, distance_squared))
    } else {
        None
    }
}

/// `BallCollision.GetSlopeEdge` (`:231-276`). `edges_to_test` is `ref` in the C#.
fn get_slope_edge(
    edges_to_test: &mut u32,
    tile: &Tile,
    tile_position: Vec2,
) -> Option<(Vec2, Vec2)> {
    match tile.slope() {
        0 => None,
        1 => {
            *edges_to_test &= edges::BOTTOM | edges::LEFT | edges::BOTTOM_LEFT_SLOPE;
            if (*edges_to_test & edges::BOTTOM_LEFT_SLOPE) == 0 {
                return None;
            }
            Some((tile_position, tile_position + Vec2::new(16.0, 16.0)))
        }
        2 => {
            *edges_to_test &= edges::BOTTOM | edges::RIGHT | edges::BOTTOM_RIGHT_SLOPE;
            if (*edges_to_test & edges::BOTTOM_RIGHT_SLOPE) == 0 {
                return None;
            }
            Some((
                tile_position + Vec2::new(0.0, 16.0),
                tile_position + Vec2::new(16.0, 0.0),
            ))
        }
        3 => {
            *edges_to_test &= edges::TOP | edges::LEFT | edges::TOP_LEFT_SLOPE;
            if (*edges_to_test & edges::TOP_LEFT_SLOPE) == 0 {
                return None;
            }
            Some((
                tile_position + Vec2::new(0.0, 16.0),
                tile_position + Vec2::new(16.0, 0.0),
            ))
        }
        4 => {
            *edges_to_test &= edges::TOP | edges::RIGHT | edges::TOP_RIGHT_SLOPE;
            if (*edges_to_test & edges::TOP_RIGHT_SLOPE) == 0 {
                return None;
            }
            Some((tile_position, tile_position + Vec2::new(16.0, 16.0)))
        }
        _ => None,
    }
}

/// `BallCollision.GetTopOrBottomEdge` (`:278-307`).
fn get_top_or_bottom_edge(
    edges_to_test: u32,
    x: i32,
    y: i32,
    mut tile_position: Vec2,
    tile: &Tile,
    world: &World,
) -> Option<(Vec2, Vec2)> {
    if (edges_to_test & edges::BOTTOM) != 0 {
        let below = world.tile(x, y + 1).copied();
        if let Some(b) = below {
            if is_neighbor_solid(Some(&b)) && b.slope() != 1 && b.slope() != 2 && !b.half_brick() {
                return None;
            }
        }
        return Some((
            Vec2::new(tile_position.x, tile_position.y + 16.0),
            Vec2::new(tile_position.x + 16.0, tile_position.y + 16.0),
        ));
    }
    if (edges_to_test & edges::TOP) != 0 {
        let above = world.tile(x, y - 1).copied();
        if !tile.half_brick() {
            if let Some(a) = above {
                if is_neighbor_solid(Some(&a)) && a.slope() != 3 && a.slope() != 4 {
                    return None;
                }
            }
        }
        if tile.half_brick() {
            tile_position.y += 8.0;
        }
        return Some((
            Vec2::new(tile_position.x, tile_position.y),
            Vec2::new(tile_position.x + 16.0, tile_position.y),
        ));
    }
    None
}

/// `BallCollision.GetLeftOrRightEdge` (`:309-344`).
fn get_left_or_right_edge(
    edges_to_test: u32,
    x: i32,
    y: i32,
    tile_position: Vec2,
    tile: &Tile,
    world: &World,
) -> Option<(Vec2, Vec2)> {
    if (edges_to_test & edges::LEFT) != 0 {
        let left = world.tile(x - 1, y).copied();
        if let Some(l) = left {
            if is_neighbor_solid(Some(&l))
                && l.slope() != 1
                && l.slope() != 3
                && (!l.half_brick() || tile.half_brick())
            {
                return None;
            }
        }
        let mut start = Vec2::new(tile_position.x, tile_position.y);
        if tile.half_brick() {
            start.y += 8.0;
        }
        return Some((start, Vec2::new(tile_position.x, tile_position.y + 16.0)));
    }
    if (edges_to_test & edges::RIGHT) != 0 {
        let right = world.tile(x + 1, y).copied();
        if let Some(r) = right {
            if is_neighbor_solid(Some(&r))
                && r.slope() != 2
                && r.slope() != 4
                && (!r.half_brick() || tile.half_brick())
            {
                return None;
            }
        }
        let mut start = Vec2::new(tile_position.x + 16.0, tile_position.y);
        if tile.half_brick() {
            start.y += 8.0;
        }
        return Some((
            start,
            Vec2::new(tile_position.x + 16.0, tile_position.y + 16.0),
        ));
    }
    None
}

/// `BallCollision.GetTileBounds` (`:346-351`): `(left, right, top, bottom)`,
/// floor-based - unlike `ToTileCoordinates`, which truncates.
fn get_tile_bounds(position: Vec2, size: Vec2) -> (i32, i32, i32, i32) {
    let left = (position.x / 16.0).floor() as i32;
    let top = (position.y / 16.0).floor() as i32;
    let right = ((position.x + size.x) / 16.0).floor() as i32 - left + 1;
    let bottom = ((position.y + size.y) / 16.0).floor() as i32 - top + 1;
    (left, left + right, top, top + bottom)
}

/// `BallCollision.IsNeighborSolid` (`:353-360`).
fn is_neighbor_solid(tile: Option<&Tile>) -> bool {
    match tile {
        Some(t) => nactive(t) && tile_solid(t.kind) && !tile_solid_top(t.kind),
        None => false,
    }
}

/// `BallCollision.ClosestPointOnLineSegment` (`:362-377`).
fn closest_point_on_line_segment(point: Vec2, line: (Vec2, Vec2)) -> Vec2 {
    let value = point - line.0;
    let vector = line.1 - line.0;
    let num = vector.length_squared();
    let num2 = value.dot(vector) / num;
    if num2 < 0.0 {
        return line.0;
    }
    if num2 > 1.0 {
        return line.1;
    }
    line.0 + vector * num2
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::FLAG_ACTIVE;

    /// A listener that does nothing - the engine's own geometry only.
    struct NopListener;
    impl BallContactListener<()> for NopListener {
        fn on_collision(
            &self,
            _p: &PhysicsProperties,
            _pos: &mut Vec2,
            _vel: &mut Vec2,
            _c: &mut BallCollisionEvent<()>,
        ) {
        }
        fn on_pass_through(
            &self,
            _p: &PhysicsProperties,
            _pos: &mut Vec2,
            _vel: &mut Vec2,
            _av: &mut f32,
            _pt: &mut BallPassThroughEvent<()>,
        ) {
        }
    }

    fn world(w: i32, h: i32) -> World {
        World::new("test".into(), w, h)
    }

    fn solid(w: &mut World, x: i32, y: i32, kind: u16) {
        let t = w.tile_mut(x, y).expect("in bounds");
        t.kind = kind;
        t.flags |= FLAG_ACTIVE;
    }

    const STONE: u16 = 1;

    #[test]
    fn gravity_and_drag_follow_the_csharp_order() {
        // `BallCollision.cs:31-47`: drag the angular velocity, drag the
        // velocity, then add `gravity / num4^2` per sub-step.
        let w = world(200, 200);
        let props = PhysicsProperties {
            gravity: 0.3,
            drag: 0.99,
        };
        let mut body = BallBody {
            position: Vec2::new(1600.0, 1600.0),
            velocity: Vec2::new(10.0, 0.0),
            size: Vec2::new(16.0, 16.0),
        };
        let mut av = 0.0f32;
        let r = step(&props, &mut body, &mut av, (), &w, &NopListener);
        // speed 10 -> num4 = ceil(10/2) = 5, num5 = 0.2, gravity term
        // 0.3 / 25 = 0.012 added 5 times = 0.06, then / 0.2 = 0.3.
        assert_eq!(r.state, BallState::Moving);
        assert!(
            (body.velocity.y - 0.3).abs() < 1e-6,
            "vy = {}",
            body.velocity.y
        );
        // x: 10 * 0.99 = 9.9, * 0.2 = 1.98 per sub-step, 5 times = 9.9,
        // then / 0.2 = 49.5 -> the sub-step scaling cancels out exactly.
        assert!(
            (body.velocity.x - 9.9).abs() < 1e-5,
            "vx = {}",
            body.velocity.x
        );
    }

    #[test]
    fn the_speed_clamp_runs_after_drag_and_resets_the_speed() {
        // `:32-38` - the drag is applied *first*, so a 5000 px/frame shot is
        // dragged to 4950 and then clamped straight to 1000; the drag does not
        // survive. The sub-step split and re-merge is float-lossy, hence the
        // tolerance rather than an exact 1000.
        let w = world(4000, 4000);
        let props = PhysicsProperties {
            gravity: 0.3,
            drag: 0.99,
        };
        let mut body = BallBody {
            position: Vec2::new(2000.0, 2000.0),
            velocity: Vec2::new(5000.0, 0.0),
            size: Vec2::new(16.0, 16.0),
        };
        let mut av = 0.0f32;
        step(&props, &mut body, &mut av, (), &w, &NopListener);
        assert!(
            (body.velocity.x - 1000.0).abs() < 1e-2,
            "vx = {}",
            body.velocity.x
        );

        // A speed below the clamp keeps the drag.
        let mut slow = BallBody {
            position: Vec2::new(2000.0, 2000.0),
            velocity: Vec2::new(100.0, 0.0),
            size: Vec2::new(16.0, 16.0),
        };
        step(&props, &mut slow, &mut av, (), &w, &NopListener);
        assert!(
            (slow.velocity.x - 99.0).abs() < 1e-4,
            "vx = {}",
            slow.velocity.x
        );
    }

    #[test]
    fn a_ball_out_of_the_world_is_not_moved() {
        // `:63-66` returns before `:85-87` writes the entity back, so the body
        // keeps its pre-step state.
        let w = world(100, 100);
        let props = PhysicsProperties {
            gravity: 0.3,
            drag: 0.99,
        };
        let before = BallBody {
            position: Vec2::new(16.0, 1600.0),
            velocity: Vec2::new(0.0, 1.0),
            size: Vec2::new(16.0, 16.0),
        };
        let mut body = before;
        let mut av = 0.0f32;
        let r = step(&props, &mut body, &mut av, (), &w, &NopListener);
        assert_eq!(r.state, BallState::OutOfBounds);
        assert_eq!(body.position, before.position);
        assert_eq!(body.velocity, before.velocity);
    }

    #[test]
    fn a_ball_bounces_off_a_flat_floor_and_settles() {
        // A solid row at y = 100; the ball falls onto it and reflects.
        let mut w = world(64, 200);
        for x in 0..64 {
            solid(&mut w, x, 100, STONE);
        }
        let props = PhysicsProperties {
            gravity: 0.3,
            drag: 0.99,
        };
        let mut body = BallBody {
            position: Vec2::new(500.0, 1600.0 - 16.0),
            velocity: Vec2::new(0.0, 6.0),
            size: Vec2::new(16.0, 16.0),
        };
        let mut av = 0.0f32;
        let r = step(&props, &mut body, &mut av, (), &w, &NopListener);
        assert_eq!(r.state, BallState::Moving);
        // The reflection flips Y, so the ball is moving up afterwards.
        assert!(body.velocity.y < 0.0, "vy = {}", body.velocity.y);
        // And it was pushed out of the tile: its bottom is above y = 1616.
        assert!(
            body.position.y + 16.0 <= 1600.0 + 1e-3,
            "y = {}",
            body.position.y
        );
    }

    #[test]
    fn a_ball_resting_on_the_floor_reports_resting() {
        // `:81-84` - after a contact, |vx| < 0.01 and -gravity < vy <= 0.
        // A ball dropped from exactly rest bounces to vy = -gravity, which is
        // *not* resting (the bound is strict), so the fixture starts with a
        // small upward velocity: -0.099 + 0.3 = 0.201 down, reflect -> -0.201.
        let mut w = world(64, 200);
        for x in 0..64 {
            solid(&mut w, x, 100, STONE);
        }
        let props = PhysicsProperties {
            gravity: 0.3,
            drag: 0.99,
        };
        let mut body = BallBody {
            position: Vec2::new(500.0, 1584.0),
            velocity: Vec2::new(0.0, -0.1),
            size: Vec2::new(16.0, 16.0),
        };
        let mut av = 0.0f32;
        let r = step(&props, &mut body, &mut av, (), &w, &NopListener);
        assert_eq!(r.state, BallState::Resting);
    }

    #[test]
    fn the_tile_bounds_floor_but_tile_coordinates_truncate() {
        // `:348-350` uses `Math.Floor`; `ToTileCoordinates` uses `(int)x >> 4`,
        // i.e. truncate toward zero *then* shift. For x = -1 those agree (both
        // -1), so the discriminating case is x in [-15, -1): floor gives -1
        // while the truncating shift gives 0.
        assert_eq!(
            get_tile_bounds(Vec2::new(-1.0, -1.0), Vec2::new(16.0, 16.0)),
            (-1, 1, -1, 1)
        );
        assert_eq!((-1.0f32 / 16.0).floor() as i32, -1);
        assert_eq!(((-1.0f32) as i32) >> 4, -1);
        assert_eq!(((-0.5f32 / 16.0).floor()) as i32, -1);
        assert_eq!(((-0.5f32) as i32) >> 4, 0);
    }

    #[test]
    fn a_water_pass_through_is_reported_to_the_listener() {
        // `:105-121` - the surface is `(y+1)*16 - liquid/255*16`; the event
        // fires only when that surface is above the ball's centre.
        use std::cell::RefCell;
        struct Recorder(RefCell<Vec<BallPassThroughType>>);
        impl BallContactListener<()> for Recorder {
            fn on_collision(
                &self,
                _p: &PhysicsProperties,
                _pos: &mut Vec2,
                _vel: &mut Vec2,
                _c: &mut BallCollisionEvent<()>,
            ) {
            }
            fn on_pass_through(
                &self,
                _p: &PhysicsProperties,
                _pos: &mut Vec2,
                _vel: &mut Vec2,
                _av: &mut f32,
                pt: &mut BallPassThroughEvent<()>,
            ) {
                self.0.borrow_mut().push(pt.kind);
            }
        }
        let mut w = world(64, 200);
        let t = w.tile_mut(30, 100).unwrap();
        t.liquid = 255;
        let rec = Recorder(RefCell::new(Vec::new()));
        let props = PhysicsProperties {
            gravity: 0.3,
            drag: 0.99,
        };
        let mut body = BallBody {
            position: Vec2::new(480.0, 1600.0),
            velocity: Vec2::new(0.0, 0.0),
            size: Vec2::new(16.0, 16.0),
        };
        let mut av = 0.0f32;
        step(&props, &mut body, &mut av, (), &w, &rec);
        assert_eq!(*rec.0.borrow(), vec![BallPassThroughType::Water]);
    }
}
