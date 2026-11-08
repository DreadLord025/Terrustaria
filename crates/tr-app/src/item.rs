//! World item drops and the inventory slot record, the first pass of `Terraria.Item`.
//!
//! Blocks broken by the player spawn a small physics entity instead of teleporting into
//! the inventory. Drops fall with gravity, bounce off solid tiles, and are magnetised to
//! the player once they are close and old enough, then added to the first matching or
//! empty hotbar slot. Rendering uses the tile's plain 16x16 frame (we do not extract the
//! separate `Item_*` sprites yet) tinted by the light map.

use glam::Vec2;
use tr_core::lighting::{LightMap, LightStyle};
use tr_core::{World, TILE_PX};

use crate::assets;
use crate::registry;

// --- Inventory slots (`Main.inventory` entries) -----------------------------

pub(crate) const HOTBAR: usize = 10;
/// `Player.inventory` length: hotbar plus the four storage rows of the open grid.
pub(crate) const INV: usize = 50;
/// `Player.SetItemAnimation` fallback for kinds outside [`registry::ITEMS`]:
/// 15 ticks — the `SetDefaults1` block default (`Item.cs:1405-1415`; the raw
/// `ResetStats` default is 100, `Item.cs:49020-49022`, which no live kind
/// reaches).
pub(crate) const DEFAULT_USE_ANIMATION_TICKS: u16 = 15;

/// The held item's swing duration in seconds — `useAnimation` ticks at the
/// 60 Hz simulation rate (`Player.ApplyItemAnimation`, `Player.cs:4449-4451`:
/// `itemAnimation = itemAnimationMax = useAnimation`, one tick per frame).
pub(crate) fn use_animation_secs(kind: u16) -> f32 {
    let ticks = registry::get(kind)
        .map(|d| d.use_animation as u16)
        .filter(|t| *t > 0)
        .unwrap_or(DEFAULT_USE_ANIMATION_TICKS);
    ticks as f32 / 60.0
}

/// The prefix-aware variant: `Item.Prefix` scales `useAnimation` (`spd`),
/// and the swing paces off the prefixed value (`Player.cs:4449-4451` reads
/// the instance). Falls back to the raw kind when the prefix is absent.
pub(crate) fn use_animation_secs_of(kind: u16, prefix: u16) -> f32 {
    if prefix == 0 {
        return use_animation_secs(kind);
    }
    crate::prefix::prefixed_stats(kind, prefix)
        .map(|s| s.use_animation)
        .filter(|t| *t > 0)
        .map(|t| t as f32 / 60.0)
        .unwrap_or_else(|| use_animation_secs(kind))
}

/// `Item.useStyle` of the kind (`Item.SetDefaults`; the
/// `ItemUseStyleID.cs` constants — 1 Swing, 2 EatFood, 3 Thrust, 4 HoldUp,
/// 5 Shoot, 6 DrinkLong, 7 DrinkOld, 8 GolfPlay, 9 DrinkLiquid, 10 Hidden,
/// 11 MowTheLawn, 12 Guitar, 13 Rapier, 14 RaiseLamp, 15 HoldOrb, 16
/// PlaySound), 0 when unregistered.
pub(crate) fn use_style(kind: u16) -> u8 {
    registry::get(kind).map_or(0, |d| d.use_style)
}

/// `Item.holdStyle` of the kind (`Item.SetDefaults`), 0 when unregistered.
pub(crate) fn hold_style(kind: u16) -> u8 {
    registry::get(kind).map_or(0, |d| d.hold_style)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ItemSlot {
    pub(crate) kind: u16,
    pub(crate) name: &'static str,
    pub(crate) count: u32,
    /// The rolled prefix (`Item.prefix`, `Item.cs:504`'s field); 0 = none.
    /// Two slots stack only when both match (`Item.CanStack`,
    /// `Item.cs:49865-49871`: same type AND same prefix).
    pub(crate) prefix: u16,
}

/// `Item.MAX_STACK` for the blocks and items we model.
pub(crate) const MAX_STACK: u32 = 999;

pub(crate) const EMPTY_SLOT: ItemSlot = ItemSlot {
    kind: 0,
    name: "",
    count: 0,
    prefix: 0,
};

/// `Player.PickupItem`: stack into the hotbar first, then an empty hotbar slot,
/// then the storage rows. The stacking rule is `Item.CanStack`
/// (`Item.cs:49865-49871`): same kind AND same prefix — a prefixed drop never
/// merges into an unprefixed stack.
pub(crate) fn slot_for_pickup(
    inventory: &mut [ItemSlot; INV],
    kind: u16,
    prefix: u16,
) -> Option<usize> {
    for range in [0..HOTBAR, HOTBAR..INV] {
        if let Some(i) = range.clone().find(|&i| {
            inventory[i].count > 0 && inventory[i].kind == kind && inventory[i].prefix == prefix
        }) {
            return Some(i);
        }
    }
    for range in [0..HOTBAR, HOTBAR..INV] {
        if let Some(i) = range.clone().find(|&i| {
            inventory[i].count == 0
                || (inventory[i].kind == kind
                    && inventory[i].prefix == prefix
                    && inventory[i].count < registry::max_stack(kind))
        }) {
            if inventory[i].count == 0 {
                inventory[i] = ItemSlot {
                    kind,
                    name: registry::name(kind),
                    count: 0,
                    prefix,
                };
            }
            return Some(i);
        }
    }
    None
}

pub const SIZE: f32 = 14.0;
const MAGNET_RADIUS: f32 = 42.0;
const GRAVITY: f32 = 0.3;
const MAX_FALL: f32 = 6.0;
/// Seconds before a drop can be collected (vanilla `Item.instanced` pickup delay).
const PICKUP_DELAY: f32 = 0.35;

#[derive(Clone, Debug)]
pub struct ItemDrop {
    pub kind: u16,
    pub count: u32,
    /// `WorldItem.shimmerTime` (`WorldItem.cs:34`) — the 0..1 ramp while the
    /// drop rests in shimmer liquid; the transform fires above 0.9.
    pub shimmer_time: f32,
    /// `WorldItem.shimmered` (`WorldItem.cs:32`) — the item has transformed at
    /// least once (it floats in the liquid afterwards).
    pub shimmered: bool,
    /// The ramp crossed 0.9 during the last [`Self::update`] — the transform
    /// fires that same tick (`WorldItem.cs:531-534` sets `shimmerTime = 0.9f`
    /// and calls `GetShimmered` inside `UpdateShimmer`).
    pub shimmer_pending: bool,
    /// `Entity.shimmerWet` (`Entity.cs:28`) — the drop's box overlaps shimmer
    /// liquid this tick.
    pub shimmer_wet: bool,
    /// The natural roll's prefix (`Item.NewItem`'s `prefix` argument,
    /// `Item.cs:49730`); 0 = none.
    pub prefix: u16,
    /// Top-left of the 14x14 item box, world pixels.
    pub pos: Vec2,
    pub vel: Vec2,
    pub age: f32,
    /// Bob phase so items on the ground do not pulse in lockstep.
    pub phase: f32,
}

impl ItemDrop {
    pub fn new(
        kind: u16,
        count: u32,
        prefix: u16,
        x: f32,
        y: f32,
        vx: f32,
        vy: f32,
        phase: f32,
    ) -> Self {
        Self {
            kind,
            count,
            shimmer_time: 0.0,
            shimmered: false,
            shimmer_pending: false,
            shimmer_wet: false,
            prefix,
            pos: Vec2::new(x, y),
            vel: Vec2::new(vx, vy),
            age: 0.0,
            phase,
        }
    }

    pub fn center(&self) -> Vec2 {
        self.pos + Vec2::splat(SIZE * 0.5)
    }

    /// `WorldItem.CanShimmerAtPosition` (`WorldItem.cs:756-768`): the tile
    /// above the drop's center (the C# reads `Center.X / 16` and
    /// `position.Y / 16 - 1`) is shimmer liquid.
    pub fn over_shimmer(&self, world: &World) -> bool {
        let tx = (self.center().x / tr_core::TILE_PX) as i32;
        let ty = (self.pos.y / tr_core::TILE_PX) as i32 - 1;
        world.tile(tx, ty)
            .is_some_and(|t| tr_core::shimmer::tile_is_shimmer(t))
    }

    /// The drop's 14x14 box overlapping shimmer liquid this tick — the port's
    /// stand-in for the C#'s `Collision.shimmer` flag that
    /// `WorldItem.Update`'s wet half sets (`WorldItem.cs:868-877`).
    fn box_in_shimmer(&self, world: &World) -> bool {
        let x0 = (self.pos.x / tr_core::TILE_PX) as i32;
        let x1 = ((self.pos.x + SIZE) / tr_core::TILE_PX) as i32;
        let y0 = (self.pos.y / tr_core::TILE_PX) as i32;
        let y1 = ((self.pos.y + SIZE) / tr_core::TILE_PX) as i32;
        for ty in y0..=y1 {
            for tx in x0..=x1 {
                if world
                    .tile(tx, ty)
                    .is_some_and(|t| tr_core::shimmer::tile_is_shimmer(t))
                {
                    return true;
                }
            }
        }
        false
    }

    /// Whether the last [`Self::update`]'s `UpdateShimmer` armed the transform
    /// (`WorldItem.cs:528-535` — the C# caps the ramp at 0.9 and fires for the
    /// local player's drops; the port is SP-only, so the fire is unconditional
    /// here and the client re-own branch is out).
    pub fn shimmer_transform_due(&self) -> bool {
        self.shimmer_pending && !self.shimmered
    }

    /// `WorldItem.UpdateShimmer` (`WorldItem.cs:455-546`): the shimmered
    /// physics (no gravity, the 0.05 rise over shimmer), the ramp ±0.01 and the
    /// 0.9 trigger check. The dust/light arms are render-layer notes.
    fn update_shimmer(&mut self, world: &World, dt: f32) {
        if self.shimmered {
            // `:457-508` — gravity is zero for the transformed item; the rise
            // applies over shimmer liquid (or already shimmerWet), floor -4.
            let over = self.shimmer_wet || self.over_shimmer(world);
            if over && self.vel.y > -4.0 {
                self.vel.y -= 0.05;
            } else if !over {
                self.vel.y *= 0.9;
            }
        }
        if self.shimmer_wet && !self.shimmered && self.over_shimmer(world) {
            self.shimmer_time = (self.shimmer_time + 0.01).min(1.0);
            // `:518-536` — the crossing is checked inside the same update, so
            // the value is capped at 0.9 and the transform fires this tick.
            self.shimmer_pending = self.shimmer_time > 0.9;
        } else if self.shimmer_time > 0.0 && !self.shimmered {
            self.shimmer_pending = false;
            self.shimmer_time = (self.shimmer_time - 0.01).max(0.0);
        }
    }

    /// One fixed 60 Hz step of gravity, collision and the pickup magnet.
    /// Returns true when the drop was collected by `player`.
    pub fn update(&mut self, world: &World, player_center: Option<Vec2>, dt: f32) -> bool {
        self.age += dt;
        self.shimmer_wet = self.box_in_shimmer(world);
        if let Some(pc) = player_center {
            let d = pc - self.center();
            let dist = d.length();
            if self.age > PICKUP_DELAY && dist < MAGNET_RADIUS {
                // Pull toward the player; skip the ground physics while collecting.
                let speed = if dist < 10.0 { 8.0 } else { 4.0 };
                self.vel = d.normalize_or_zero() * speed;
                self.pos += self.vel;
                return dist < 10.0;
            }
        }
        self.update_shimmer(world, dt);
        if self.shimmered {
            // `WorldItem.cs:457-508` — the transformed item does not fall.
            self.pos += self.vel;
            self.shimmer_time = 1.0;
            return false;
        }
        self.vel.y = (self.vel.y + GRAVITY).min(MAX_FALL);
        let nx = self.pos.x + self.vel.x;
        if crate::player::collides_rect(world, nx, self.pos.y, SIZE, SIZE) {
            self.vel.x = -self.vel.x * 0.35;
        } else {
            self.pos.x = nx;
        }
        let ny = self.pos.y + self.vel.y;
        if crate::player::collides_rect(world, self.pos.x, ny, SIZE, SIZE) {
            if self.vel.y > 0.0 {
                let ty = ((ny + SIZE) / TILE_PX).floor() as i32;
                self.pos.y = ty as f32 * TILE_PX - SIZE;
                if self.vel.y > 2.0 {
                    self.vel.y = -self.vel.y * 0.3;
                } else {
                    self.vel.y = 0.0;
                    self.vel.x *= 0.6;
                }
            } else {
                let ty = (ny / TILE_PX).floor() as i32;
                self.pos.y = (ty + 1) as f32 * TILE_PX;
                self.vel.y = 0.0;
            }
        } else {
            self.pos.y = ny;
        }
        false
    }

    /// Draws the drop in world space with a small ground bob. Returns the quad count.
    pub fn draw(
        &self,
        r: &mut tr_render::SpriteRenderer,
        lookups: &assets::Lookups,
        light: &LightMap,
        style: &LightStyle,
        anim_time: f32,
    ) -> u32 {
        let uv = if let Some((uv, _, _)) = lookups.item_icon(self.kind) {
            Some(uv)
        } else if let Some(coin) = registry::coin_index(self.kind) {
            lookups.coin_uv(coin)
        } else if registry::projectile(self.kind).is_some() {
            lookups.shuriken_uv()
        } else if let Some(wall) = registry::place_wall(self.kind) {
            lookups.wall_icon_uv(wall)
        } else if let Some(tile) = registry::place_tile(self.kind) {
            lookups.plain_tex(tile).map(|t| t.uv)
        } else {
            None
        };
        let Some(uv) = uv else {
            return 0;
        };
        let tx = (self.pos.x / TILE_PX) as i32;
        let ty = (self.pos.y / TILE_PX) as i32;
        let l = light.get(tx, ty);
        let c = style.shade(l, tx, ty, anim_time);
        let bob = (anim_time * 3.0 + self.phase).sin() * 1.5;
        r.push_sprite(
            [self.pos.x + 1.0, self.pos.y + 1.0 + bob],
            [SIZE, SIZE],
            uv,
            [c[0], c[1], c[2], 1.0],
        );
        1
    }
}

/// Hash of the item list for the G6 world-range key; includes the animation step so
/// the ground bob updates.
pub fn render_key(items: &[ItemDrop], anim_step: u32) -> u32 {
    let mut h = items.len() as u32 ^ (anim_step << 24);
    for it in items {
        h = h.wrapping_mul(31).wrapping_add(it.kind as u32);
        h = h.wrapping_mul(31).wrapping_add(it.pos.x.to_bits());
        h = h.wrapping_mul(31).wrapping_add(it.pos.y.to_bits());
        h = h.wrapping_mul(31).wrapping_add(it.count);
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

    #[test]
    fn pickup_prefers_stacks_then_empty_slots() {
        let mut inv: [ItemSlot; INV] =
            std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        inv[0] = ItemSlot {
            kind: 3,
            name: "Stone Block",
            count: 4,
            prefix: 0,
        };
        assert_eq!(slot_for_pickup(&mut inv, 3, 0), Some(0));
        assert_eq!(slot_for_pickup(&mut inv, 12, 0), Some(1));
        assert_eq!(inv[1].kind, 12);
        assert_eq!(inv[1].name, "Copper Ore");
        assert_eq!(inv[1].count, 0);
        for slot in &mut inv {
            slot.count = 1;
        }
        assert_eq!(
            slot_for_pickup(&mut inv, 53, 0),
            None,
            "full hotbar must refuse"
        );
    }

    fn ground() -> World {
        let mut w = World::new("items".into(), 32, 32);
        for x in 0..32 {
            for y in 20..32 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    #[test]
    fn registry_maps_items_to_tiles_walls_and_coins() {
        // `Terraria.ID.ItemID` ids: Dirt Block 2, Stone Wall 26, Shuriken 42.
        assert_eq!(registry::place_wall(30), Some(16)); // WallID.Dirt = 16 (WallID.cs:137)
        assert_eq!(registry::name(30), "Dirt Wall");
        assert_eq!(registry::name(3), "Stone Block");
        assert_eq!(registry::by_wall(16), Some(30)); // the placed (safe) wall
        assert_eq!(registry::by_wall(2), Some(5546)); // DefaultToPlaceableWall(2), Item.cs case 5546
        assert_eq!(registry::projectile(42), Some(crate::projectile::SHURIKEN));
        assert!(registry::is_weapon(42));
        assert_eq!(registry::by_tile(0), Some(2));
        assert_eq!(registry::coin_index(73), Some(2));
        assert_eq!(registry::coin_value(73), 10_000);
        assert_eq!(registry::name(73), "Gold Coin");
        assert_eq!(registry::migrate_kind(0), 2);
        assert_eq!(registry::migrate_kind(10_002), 30);
        assert_eq!(registry::migrate_kind(20_001), 42);
        assert_eq!(registry::migrate_kind(30_002), 73);
    }

    #[test]
    fn drops_fall_and_rest_on_the_ground() {
        let w = ground();
        let mut it = ItemDrop::new(2, 1, 0, 10.0 * TILE_PX, 10.0 * TILE_PX, 0.0, 0.0, 0.0);
        for _ in 0..240 {
            assert!(!it.update(&w, None, 1.0 / 60.0));
        }
        let feet = it.pos.y + SIZE;
        assert!(
            (feet - 20.0 * TILE_PX).abs() < 2.0,
            "rested at {feet} instead of the floor"
        );
    }

    #[test]
    fn drops_bounce_off_walls() {
        let mut w = ground();
        for y in 14..20 {
            let i = w.index(14, y);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        let mut it = ItemDrop::new(2, 1, 0, 10.0 * TILE_PX, 19.0 * TILE_PX, 6.0, 0.0, 0.0);
        for _ in 0..120 {
            it.update(&w, None, 1.0 / 60.0);
        }
        assert!(it.pos.x + SIZE <= 14.0 * TILE_PX, "x {}", it.pos.x);
    }

    #[test]
    fn magnet_collects_a_drop_next_to_the_player() {
        let w = ground();
        let player = Vec2::new(16.0 * TILE_PX, 20.0 * TILE_PX - 21.0);
        let mut it = ItemDrop::new(12, 3, 0, 16.0 * TILE_PX + 8.0, 19.0 * TILE_PX, 0.0, 0.0, 0.0);
        let mut collected = false;
        for _ in 0..240 {
            if it.update(&w, Some(player), 1.0 / 60.0) {
                collected = true;
                break;
            }
        }
        assert!(collected, "drop was never collected (pos {:?})", it.pos);
    }
}
