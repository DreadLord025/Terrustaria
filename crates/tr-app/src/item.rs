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

/// `Player.ItemCheck_StartActualUse`'s sound gate (`Player.cs:51681-51682`):
/// `sItem.UseSound != null && !ItemID.Sets.SkipsInitialUseSound[sItem.type]`
/// — the set is `{2350, 4870}` (`ItemID.cs:408`, the Recall Potion / Potion
/// of Return: their cue plays at the teleport, not the use start). Returns
/// the kind's `SOUND_STYLES` index + `useSoundPitch`
/// (`Player.cs:51696`'s `PlaySound(sItem.UseSound, base.Center,
/// sItem.useSoundPitch)`), `None` when the use start is silent.
pub(crate) fn initial_use_sound(kind: u16) -> Option<(u16, f32)> {
    if matches!(kind, 2350 | 4870) {
        return None;
    }
    let d = registry::get(kind)?;
    Some((d.use_sound?, d.use_sound_pitch))
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

/// `Item.CommonMaxStack` (`Item.cs:49966`) �?" the default stack limit applied
/// by `ResetStats`/`SetDefaults` before id-specific overrides.
pub(crate) const COMMON_MAX_STACK: u32 = 9999;
/// Compatibility alias; prefer [`COMMON_MAX_STACK`].
pub(crate) const MAX_STACK: u32 = COMMON_MAX_STACK;

pub(crate) const EMPTY_SLOT: ItemSlot = ItemSlot {
    kind: 0,
    name: "",
    count: 0,
    prefix: 0,
};

impl ItemSlot {
    /// `Item.Clone` (`Item.cs:49837-49840`, S1-F-10): a memberwise copy of the
    /// item instance snapshot.
    pub(crate) fn clone_item(self) -> Self {
        self
    }

    /// `Item.Refresh` (`Item.cs:49945-49961`, S1-F-10, simplified): re-apply the
    /// kind's default name and clamp the stack to the current `maxStack`. The
    /// port has no variant system, so `onlyIfVariantChanged` is ignored and the
    /// snapshot is refreshed unconditionally when the slot is not air.
    pub(crate) fn refresh(&mut self, _only_if_variant_changed: bool) {
        if self.kind == 0 || self.count == 0 {
            *self = EMPTY_SLOT;
            return;
        }
        self.name = registry::name(self.kind);
        let max = registry::max_stack(self.kind);
        if self.count > max {
            self.count = max;
        }
    }
}

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
/// `Player.defaultItemGrabRange = 42` (`Player.cs:57121`, read by
/// `GetItemGrabRange`, `Player.cs:35132`) — the px radius the magnet pulls in.
const MAGNET_RADIUS: f32 = 42.0;
/// `Item.lifeGrabRange = 250` (`Item.cs:49964`): `GetItemGrabRange` adds it
/// when `lifeMagnet` is armed and the kind is a heart (`Player.cs:35145-35148`).
pub const LIFE_GRAB_RANGE: f32 = 250.0;
/// `Item.manaGrabRange = 300` (`Item.cs:49963`): the star family's bonus under
/// `manaMagnet` (`Player.cs:35137-35140`).
pub const MANA_GRAB_RANGE: f32 = 300.0;

/// `Player.PickupItem`'s heart family (`Player.cs:35059`): the Heart 58, the
/// Candy Apple 1734 (Halloween) and the Candy Cane 1867 (Christmas) — the
/// `Item.NewItem` seasonal remap's products (`Item.cs:49668-49714`).
pub const HEART_KINDS: [u16; 3] = [58, 1734, 1867];
/// `Player.PickupItem`'s star family (`Player.cs:35065`): the Mana Star 184,
/// the Soul Cake 1735 and the Sugar Plum 1868.
pub const STAR_KINDS: [u16; 3] = [184, 1735, 1868];
/// `ItemID.Sets.IsAPickup`'s heart/star subset (`ItemID.cs:493` — the full
/// set also holds the nebula trio 3453-3455 and the Mana Cloak star 4143,
/// unported families): `Player.ItemSpace` returns `CanTakeItem: true`
/// unconditionally for them (`Player.cs:39155-39158`) — an inventory-full
/// player still grabs them, and a FULL-life/full-mana player consumes them
/// too (the heal/restore caps; only the star's PULL is gated at full mana,
/// `Player.cs:34947`).
pub fn is_heart_or_star(kind: u16) -> bool {
    HEART_KINDS.contains(&kind) || STAR_KINDS.contains(&kind)
}

/// The heart/star pull inputs — the C#'s `Player` fields `GrabItems` reads
/// (`Player.cs:34943-34952`). The generic drop keeps the port's radial
/// stand-in pull (the ITEMS-2 deviation); the heart/star families run the
/// verbatim magnet speeds.
#[derive(Clone, Copy, Debug, Default)]
pub struct PickupContext {
    /// `Player.lifeMagnet` — buff 105 (`Player.cs:10140`).
    pub life_magnet: bool,
    /// `Player.manaMagnet` — accessories 2219/2220/2221/4000.
    pub mana_magnet: bool,
    /// `HeldItem.magic` (`Player.cs:34947`): a magic weapon in hand pulls
    /// stars even at full mana.
    pub held_item_magic: bool,
    /// `statMana != statManaMax2` (`Player.cs:34947`).
    pub mana_not_full: bool,
}

/// `Player.GetItemGrabRange` (`Player.cs:35130-35166`), the heart/star rows:
/// the 42 base + the 250 `lifeGrabRange` under `lifeMagnet` for hearts, the
/// 300 `manaGrabRange` under `manaMagnet` for stars. The coin/treasure/
/// nebula/journey rows are unported families (the standing ITEMS-2 note).
pub fn grab_range(kind: u16, ctx: &PickupContext) -> f32 {
    let mut num = MAGNET_RADIUS;
    if ctx.mana_magnet && STAR_KINDS.contains(&kind) {
        num += MANA_GRAB_RANGE; // `:35137-35140`
    }
    if ctx.life_magnet && HEART_KINDS.contains(&kind) {
        num += LIFE_GRAB_RANGE; // `:35145-35148`
    }
    num
}

/// `Player.PullItem_Pickup` (`Player.cs:35031-35041`): the unit direction
/// from the drop's box centre to the player's centre scaled to `speed`,
/// blended into the velocity as `(vel * (acc - 1) + pull) / acc`. The C#
/// centre is `position + (width / 2, height / 2)` — the INTEGER halving of
/// the heart/star's 12x12 box (`Item.cs` SetDefaults) is +6; the port's
/// 14x14 drop box centre rides [`ItemDrop::center`] (+7, the standing
/// drop-box deviation).
fn pull_item_pickup(vel: Vec2, drop_center: Vec2, player_center: Vec2, speed: f32, acc: f32) -> Vec2 {
    let mut d = player_center - drop_center;
    let num3 = speed / (d.x * d.x + d.y * d.y).sqrt();
    d *= num3;
    (vel * (acc - 1.0) + d) / acc
}
/// `WorldItem.UpdateItem`'s air constants (`WorldItem.cs:363-364`):
/// `float gravity = 0.1f; float maxFallSpeed = 7f;` — the wet/shimmer/honey
/// arms override them (`:380-396`), the air case is this pair.
const GRAVITY: f32 = 0.1;
const MAX_FALL: f32 = 7.0;
/// `WorldItem.MoveInWorld` (`WorldItem.cs:854-860`): the horizontal friction
/// `base.velocity.X *= 0.95f` and the snap to rest below `0.1`.
const FRICTION_X: f32 = 0.95;
const FRICTION_EPSILON: f32 = 0.1;
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
        self.box_in_liquid(world, |t| tr_core::shimmer::tile_is_shimmer(t))
    }

    /// `Collision.WetCollision(position, width, height)` (`WorldItem.cs:870`):
    /// the drop's box overlaps any non-empty liquid cell. This is `base.wet`,
    /// the gate on most `UpdateItem_VisualEffects` light arms.
    pub fn box_is_wet(&self, world: &World) -> bool {
        self.box_in_liquid(world, |t| t.liquid > 0)
    }

    fn box_in_liquid(&self, world: &World, hit: impl Fn(&tr_core::tile::Tile) -> bool) -> bool {
        let x0 = (self.pos.x / tr_core::TILE_PX) as i32;
        let x1 = ((self.pos.x + SIZE) / tr_core::TILE_PX) as i32;
        let y0 = (self.pos.y / tr_core::TILE_PX) as i32;
        let y1 = ((self.pos.y + SIZE) / tr_core::TILE_PX) as i32;
        for ty in y0..=y1 {
            for tx in x0..=x1 {
                if let Some(t) = world.tile(tx, ty) {
                    if hit(t) {
                        return true;
                    }
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
    /// Returns true when the drop was collected by `player`. The legacy
    /// entry point: no magnet flags and no held weapon (the pre-i52
    /// behaviour for the generic radial pull; the star pull gate reads
    /// `mana_not_full: true` — the caller with a live player passes the
    /// real context through [`Self::update_with_pickup`]).
    pub fn update(&mut self, world: &World, player_center: Option<Vec2>, dt: f32) -> bool {
        self.update_with_pickup(
            world,
            player_center,
            &PickupContext { mana_not_full: true, ..PickupContext::default() },
            dt,
        )
    }

    /// [`Self::update`] with the `GrabItems` player context (S1-P-i52): the
    /// heart/star families run the C#'s grab-range bonuses and magnet pulls
    /// (`Player.cs:34943-34952`, `:35130-35166`); every other kind keeps the
    /// port's radial stand-in (the ITEMS-2 deviation).
    pub fn update_with_pickup(
        &mut self,
        world: &World,
        player_center: Option<Vec2>,
        ctx: &PickupContext,
        dt: f32,
    ) -> bool {
        self.age += dt;
        self.shimmer_wet = self.box_in_shimmer(world);
        if let Some(pc) = player_center {
            let d = pc - self.center();
            let dist = d.length();
            if self.age > PICKUP_DELAY && is_heart_or_star(self.kind) {
                // The direct `Hitbox.Intersects` pickup (`Player.cs:34917-34922`)
                // fires regardless of the pull gates — a full-mana player
                // standing ON a star still consumes it (`ItemSpace`'s
                // `IsAPickup` short-circuit, `:39155-39158`). The port's
                // 10 px centre-distance stand-in is the ITEMS-2 convention.
                if dist < 10.0 {
                    return true;
                }
                let star = STAR_KINDS.contains(&self.kind);
                // `:34947` — `else if (!flag2 || this.HeldItem.magic ||
                // this.statMana != this.statManaMax2)`: a star with no
                // `manaMagnet`, no magic weapon in hand and full mana is
                // NOT pulled at all (the whole pull chain is skipped).
                let pull_gated =
                    star && !ctx.mana_magnet && !ctx.held_item_magic && !ctx.mana_not_full;
                if dist < grab_range(self.kind, ctx) && !pull_gated {
                    if star && ctx.mana_magnet {
                        // `:34943-34946` — `PullItem_Pickup(worldItem, 12f, 5)`.
                        self.vel = pull_item_pickup(self.vel, self.center(), pc, 12.0, 5.0);
                    } else if !star && ctx.life_magnet {
                        // `:34949-34952` — `PullItem_Pickup(worldItem, 15f, 5)`.
                        self.vel = pull_item_pickup(self.vel, self.center(), pc, 15.0, 5.0);
                    } else {
                        // The unmagnetised heart/star falls to the port's
                        // radial stand-in for `PullItem_Common(0.75f)`
                        // (`:34971`, the ITEMS-2 deviation).
                        let speed = if dist < 10.0 { 8.0 } else { 4.0 };
                        self.vel = d.normalize_or_zero() * speed;
                    }
                    self.pos += self.vel;
                    return false;
                }
                // Out of range or pull-gated: the drop keeps its physics.
            } else if self.age > PICKUP_DELAY && dist < MAGNET_RADIUS {
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
        // `WorldItem.cs:363-364` — the air gravity and the fall cap.
        self.vel.y = (self.vel.y + GRAVITY).min(MAX_FALL);
        // `WorldItem.cs:854-860` — the friction and the `0.1` rest snap, both
        // applied BEFORE the collision pass (`MoveInWorld`'s order).
        self.vel.x *= FRICTION_X;
        if self.vel.x.abs() < FRICTION_EPSILON {
            self.vel.x = 0.0;
        }
        // `WorldItem.cs:864`: the belt contacts bend the velocity before the
        // AABB pass. The C# stores the `bool` in `item.onConveyor`, which only
        // suppresses the draw-side velocity spin (`Main.cs:38613`) - the port
        // draws drops upright, so the flag has no consumer and is dropped.
        let mut bp = [self.pos.x, self.pos.y];
        let mut bv = [self.vel.x, self.vel.y];
        crate::collision::apply_conveyor_belt_movement_to_velocity(
            world,
            &mut bp,
            &mut bv,
            SIZE as i32,
            SIZE as i32,
        );
        self.pos = Vec2::new(bp[0], bp[1]);
        self.vel = Vec2::new(bv[0], bv[1]);
        let nx = self.pos.x + self.vel.x;
        if crate::player::collides_rect(world, nx, self.pos.y, SIZE, SIZE) {
            // `Collision.TileCollision` (`Collision.cs:2415-2426`) leaves only
            // the residual that fit on the blocked axis (`result.X = …`), so a
            // drop stops dead against a wall instead of bouncing back.
            self.vel.x = 0.0;
        } else {
            self.pos.x = nx;
        }
        let ny = self.pos.y + self.vel.y;
        if crate::player::collides_rect(world, self.pos.x, ny, SIZE, SIZE) {
            if self.vel.y > 0.0 {
                let ty = ((ny + SIZE) / TILE_PX).floor() as i32;
                self.pos.y = ty as f32 * TILE_PX - SIZE;
            } else {
                let ty = (ny / TILE_PX).floor() as i32;
                self.pos.y = (ty + 1) as f32 * TILE_PX;
            }
            // `Collision.cs:2446-2451`: `result.Y` collapses to the flush
            // residual (`+ 0.01f` for `gravDir == 1`), i.e. no restitution —
            // the drop lies where it landed.
            self.vel.y = 0.0;
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

    /// S1-E-21 wave A — the per-site cue selection: `Item.UseSound` rows
    /// (`Item.cs`'s `SetDefaults` cases + the `DefaultTo*` helpers) resolve
    /// through `SOUND_STYLES` to the legacy `(SoundId 2, Style)` pair, and
    /// the `SkipsInitialUseSound` gate keeps 2350/4870 silent at the use
    /// start (`Player.cs:51681-51682`, `ItemID.cs:408`).
    #[test]
    fn s1_e_21a_the_item_use_cue_selection_follows_the_vanilla_rows() {
        let style_of = |kind: u16| {
            let (idx, _) = initial_use_sound(kind).expect("the kind plays at the use start");
            let s = &crate::soundid::SOUND_STYLES[idx as usize];
            (s.sound_id, s.style, s.name)
        };
        // The Copper Pickaxe 1 (`Item.cs:1401`) and the Iron Broadsword 4
        // (`:1437`) → SoundID.Item1 (`SoundID.cs:1384`).
        assert_eq!(style_of(1), (2, 1, "Item1"));
        assert_eq!(style_of(4), (2, 1, "Item1"));
        // The Mushroom 5 (`Item.cs:1444`) → Item2 (`:1385`).
        assert_eq!(style_of(5), (2, 2, "Item2"));
        // The Lesser Healing Potion 28 — `DefaultToHealingPotion`'s Item3
        // (`Item.cs:48572`, `SoundID.cs:1386`).
        assert_eq!(style_of(28), (2, 3, "Item3"));
        // The Wooden Bow 39 — `DefaultToBow`'s Item5 (`Item.cs:48427`,
        // `SoundID.cs:1388`).
        assert_eq!(style_of(39), (2, 5, "Item5"));
        // The Storm Tiger Staff 4607 — its own case row's Item44
        // (`Item.cs:38142`, `SoundID.cs:1427`); the `DefaultToStaff` helper's
        // Item43 (`Item.cs:48021`) rides the 6171 correction below.
        assert_eq!(style_of(4607), (2, 44, "Item44"));
        // The whips — `DefaultToWhip`'s Item152 (`Item.cs:47819`,
        // `SoundID.cs:1536`).
        assert_eq!(style_of(4672), (2, 152, "Item152"));
        // Abigail's Flower 5114 → the AbigailSummon trackable
        // (`SoundID.cs:1699`).
        assert_eq!(style_of(5114).2, "AbigailSummon");
        assert_eq!(crate::soundid::SOUND_STYLES[440].sound_id, 42);
        // The hand-written 6171 correction rides the staff sound too
        // (`registry.rs`'s ITEM_CORRECTIONS, `Item.cs:47551`).
        assert_eq!(style_of(6171), (2, 43, "Item43"));
        // The Paladin's Hammer 1513 keeps its -0.5 pitch offset
        // (`Item.cs:18499`).
        let (_, pitch) = initial_use_sound(1513).unwrap();
        assert_eq!(pitch, -0.5);
        // The gate: 2350 (Recall Potion) / 4870 (Potion of Return) skip the
        // initial sound (`ItemID.cs:408`); the null rows play nothing —
        // the Compass 393 (`DefaultToVoiceOverrideAccessory`,
        // `Item.cs:47788`) and the golf clubs (`:48524`).
        assert_eq!(initial_use_sound(2350), None);
        assert_eq!(initial_use_sound(4870), None);
        assert_eq!(initial_use_sound(393), None);
        assert_eq!(initial_use_sound(4039), None);
        // The Dirt Block 2 has no UseSound row (`Item.cs:1406-1416`).
        assert_eq!(initial_use_sound(2), None);
    }

    #[test]
    fn s1_e_21a_every_use_sound_style_resolves_to_a_registered_cue() {
        // Every distinct `use_sound` index the generated table names must
        // land on a cue the bank registers: the id-2 family → `Item_<style>`
        // in SFX_CUES (plus 18/19, the style-1 remap targets), the id-42
        // trackables → the pinned first variation in ITEM_CUSTOM_CUES.
        for d in crate::itemdata::ITEMS.iter() {
            let Some(idx) = d.use_sound else { continue };
            let s = &crate::soundid::SOUND_STYLES[idx as usize];
            if s.sound_id == 42 {
                let base = s.trackable.unwrap();
                let cue = if s.variations == 1 {
                    base.to_string()
                } else {
                    format!("{base}_0")
                };
                assert!(
                    crate::audio::ITEM_CUSTOM_CUES.contains(&cue.as_str()),
                    "item {}'s trackable {cue} is unregistered",
                    d.id
                );
            } else {
                assert_eq!(s.sound_id, 2, "item {}'s style id", d.id);
                let cue = format!("Item_{}", s.style);
                assert!(
                    crate::audio::sfx_cue_registered(&cue),
                    "item {}'s cue {cue} is unregistered",
                    d.id
                );
            }
        }
    }

    #[test]
    fn s1_f_10_item_slot_clone_and_refresh_snapshot() {
        // `Item.Clone`/`Item.Refresh` (`Item.cs:49837-49840`, `:49945-49961`,
        // S1-F-10): clone copies the instance snapshot; refresh re-applies the
        // kind's defaults and clamps to `maxStack` (CommonMaxStack 9999 for
        // unregistered ids).
        let slot = ItemSlot {
            kind: 2, // Dirt Block, max stack 9999
            name: "Dirt Block",
            count: 10,
            prefix: 0,
        };
        assert_eq!(slot.clone_item(), slot);
        let mut over = ItemSlot {
            kind: 6196, // unregistered -> CommonMaxStack 9999
            name: "Wrong",
            count: 12_000,
            prefix: 0,
        };
        over.refresh(false);
        assert_eq!(over.kind, 6196);
        assert_eq!(over.name, "Item");
        assert_eq!(over.count, 9999);
        let mut air = EMPTY_SLOT;
        air.refresh(false);
        assert_eq!(air, EMPTY_SLOT);
    }

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

    // ---- S1-P-i52 — the heart/star grab families (written, NOT run) ----

    /// `Player.GetItemGrabRange` (`Player.cs:35130-35166`): the 42 base
    /// (`:57121`), +250 `Item.lifeGrabRange` (`Item.cs:49964`) for the heart
    /// trio under `lifeMagnet` (`:35145-35148`), +300 `Item.manaGrabRange`
    /// (`:49963`) for the star trio under `manaMagnet` (`:35137-35140`).
    /// The candy variants ride the SAME rows — the C# tests the type lists,
    /// not the base ids.
    #[test]
    fn s1_p_i52_the_grab_range_rows_add_the_magnet_bonuses() {
        let plain = PickupContext::default();
        for k in HEART_KINDS {
            assert_eq!(grab_range(k, &plain), 42.0, "heart {k} plain");
        }
        for k in STAR_KINDS {
            assert_eq!(grab_range(k, &plain), 42.0, "star {k} plain");
        }
        let life = PickupContext { life_magnet: true, ..Default::default() };
        for k in HEART_KINDS {
            assert_eq!(grab_range(k, &life), 292.0, "heart {k} + lifeMagnet");
        }
        for k in STAR_KINDS {
            assert_eq!(grab_range(k, &life), 42.0, "lifeMagnet ignores stars");
        }
        let mana = PickupContext { mana_magnet: true, ..Default::default() };
        for k in STAR_KINDS {
            assert_eq!(grab_range(k, &mana), 342.0, "star {k} + manaMagnet");
        }
        for k in HEART_KINDS {
            assert_eq!(grab_range(k, &mana), 42.0, "manaMagnet ignores hearts");
        }
        // A generic kind never earns either bonus (`:35137`/`:35145`'s
        // type gates).
        let both = PickupContext { life_magnet: true, mana_magnet: true, ..Default::default() };
        assert_eq!(grab_range(12, &both), 42.0);
    }

    /// `Player.PullItem_Pickup` (`Player.cs:35031-35041`): a resting drop
    /// 100 px to the RIGHT of the player, heart + lifeMagnet (speed 15,
    /// acc 5): the pull vector is (-15, 0), the blend `(0*4 + (-15))/5` =
    /// -3 — the first magnet tick moves the heart 3 px, not the radial
    /// stand-in's 4. The star + manaMagnet arm (12/5) blends to -2.4.
    #[test]
    fn s1_p_i52_the_magnet_pull_blends_at_the_csharp_speed_and_acc() {
        let w = ground();
        let pc = Vec2::new(200.0, 200.0);
        // centre at (300, 200) — pos = centre - 7 (the 14x14 box).
        let mut heart = ItemDrop::new(58, 1, 0, 293.0, 193.0, 0.0, 0.0, 0.0);
        heart.age = 1.0; // past PICKUP_DELAY
        let ctx = PickupContext { life_magnet: true, ..Default::default() };
        assert!(!heart.update_with_pickup(&w, Some(pc), &ctx, 1.0 / 60.0));
        // f32: 15/100 is inexact, so the blend lands within an ulp of -3
        // (the C# runs the same f32 arithmetic).
        assert!((heart.vel.x - -3.0).abs() < 1e-4, "15/5 blend from rest: {}", heart.vel.x);
        assert_eq!(heart.vel.y, 0.0);
        assert!((heart.pos.x - 290.0).abs() < 1e-3, "the blend moves the drop: {}", heart.pos.x);
        assert_eq!(heart.pos.y, 193.0);
        // The second tick blends again: (−3*4 + −15)/5 = −5.4.
        assert!(!heart.update_with_pickup(&w, Some(pc), &ctx, 1.0 / 60.0));
        assert!((heart.vel.x - -5.4).abs() < 1e-4, "vel {}", heart.vel.x);
        let mut star = ItemDrop::new(184, 1, 0, 293.0, 193.0, 0.0, 0.0, 0.0);
        star.age = 1.0;
        let ctx = PickupContext { mana_magnet: true, mana_not_full: true, ..Default::default() };
        assert!(!star.update_with_pickup(&w, Some(pc), &ctx, 1.0 / 60.0));
        assert!((star.vel.x - -2.4).abs() < 1e-4, "12/5 blend: {}", star.vel.x);
    }

    /// `Player.GrabItems` `:34947` — `else if (!flag2 || this.HeldItem.magic
    /// || this.statMana != this.statManaMax2)`: a star at FULL mana with no
    /// manaMagnet and no magic weapon in hand is not pulled at all (it keeps
    /// falling under gravity); a magic weapon in hand pulls it (the radial
    /// stand-in), and so does a non-full mana pool.
    #[test]
    fn s1_p_i52_the_full_mana_star_pull_gate_follows_the_held_class() {
        let w = ground();
        let pc = Vec2::new(200.0, 200.0);
        let mk = || {
            let mut s = ItemDrop::new(184, 1, 0, 223.0, 193.0, 0.0, 0.0, 0.0);
            s.age = 1.0;
            s
        };
        // Full mana, no magnet, no magic held: outside the pull, gravity runs.
        let full = PickupContext { mana_not_full: false, ..Default::default() };
        let mut s = mk();
        assert!(!s.update_with_pickup(&w, Some(pc), &full, 1.0 / 60.0));
        assert!(s.pos.x == 223.0, "no horizontal pull: {}", s.pos.x);
        assert!(s.vel.y > 0.0, "gravity applied instead: {}", s.vel.y);
        // Magic weapon in hand: pulled even at full mana.
        let magic = PickupContext { mana_not_full: false, held_item_magic: true, ..Default::default() };
        let mut s = mk();
        s.update_with_pickup(&w, Some(pc), &magic, 1.0 / 60.0);
        assert!(s.vel.x < 0.0, "the magic-held pull: {}", s.vel);
        // Not full: pulled.
        let hungry = PickupContext { mana_not_full: true, ..Default::default() };
        let mut s = mk();
        s.update_with_pickup(&w, Some(pc), &hungry, 1.0 / 60.0);
        assert!(s.vel.x < 0.0, "the non-full pull: {}", s.vel);
        // manaMagnet overrides the gate entirely (`:34943`'s first arm).
        let magnet = PickupContext { mana_not_full: false, mana_magnet: true, ..Default::default() };
        let mut s = mk();
        s.update_with_pickup(&w, Some(pc), &magnet, 1.0 / 60.0);
        assert!(s.vel.x < 0.0, "the magnet pull at full mana: {}", s.vel);
    }

    /// The DIRECT pickup is never refused at full stats: `ItemSpace`'s
    /// `IsAPickup` short-circuit (`Player.cs:39155-39158`, `ItemID.cs:493`)
    /// returns `CanTakeItem: true` before any slot walk, and `GrabItems`'
    /// `Hitbox.Intersects` arm (`:34917-34922`) fires without the `:34947`
    /// pull gate — a full-mana player standing ON a star consumes it (the
    /// restore caps at the max). The heart never had any gate.
    #[test]
    fn s1_p_i52_the_touch_pickup_fires_at_full_stats() {
        let w = ground();
        let pc = Vec2::new(200.0, 200.0);
        let full = PickupContext { mana_not_full: false, ..Default::default() };
        for k in [58u16, 1734, 1867, 184, 1735, 1868] {
            let mut d = ItemDrop::new(k, 1, 0, pc.x - 7.0, pc.y - 7.0, 0.0, 0.0, 0.0);
            d.age = 1.0;
            assert!(
                d.update_with_pickup(&w, Some(pc), &full, 1.0 / 60.0),
                "kind {k} collected on touch at full stats"
            );
        }
    }

    /// The heart's 292 px reach under Heartreach: a heart 250 px away (out
    /// of the plain 42 reach) is pulled; at 300 px (past 292) it is not.
    #[test]
    fn s1_p_i52_the_heartreach_potion_extends_the_heart_reach() {
        let w = ground();
        let pc = Vec2::new(400.0, 200.0);
        let ctx = PickupContext { life_magnet: true, ..Default::default() };
        let mut near = ItemDrop::new(1734, 1, 0, 400.0 + 250.0 - 7.0, 193.0, 0.0, 0.0, 0.0);
        near.age = 1.0;
        near.update_with_pickup(&w, Some(pc), &ctx, 1.0 / 60.0);
        assert!(near.vel.x < 0.0, "250 px is inside the 292 reach");
        let mut far = ItemDrop::new(1734, 1, 0, 400.0 + 300.0 - 7.0, 193.0, 0.0, 0.0, 0.0);
        far.age = 1.0;
        far.update_with_pickup(&w, Some(pc), &ctx, 1.0 / 60.0);
        assert_eq!(far.vel.x, 0.0, "300 px is outside the 292 reach");
    }
}
