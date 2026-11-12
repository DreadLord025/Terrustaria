//! The whip weapon geometry + the tag-damage tables — the AI_165 swing machine's
//! pure half. C#: `Projectile.AI_165_Whip` (`Projectile.cs:47667-48270`),
//! `GetWhipSettings` (`:48337-48419`), `FillWhipControlPoints` (`:48278-48335`),
//! `Projectile.Colliding`'s whip arm (`:15716-15737`), the tag tables
//! (`ItemID.cs:1473-1580`, `ProjectileID.cs:193-194`) and the tag-damage math
//! (`Projectile.cs:12919-12933`, `TagDamageChanges` `GameContent/Items/TagDamageChanges.cs`).
//!
//! Pure functions only: the `tr-app` side owns the pool slot, the player and the NPCs,
//! and calls in (`crates/tr-app/src/projectile.rs`, `tag_effects.rs`).

use crate::loot::DropRng;
use glam::Vec2;

/// The 18 whip projectile kinds — `ProjectileID.Sets.IsAWhip` (`ProjectileID.cs:193`).
pub const IS_A_WHIP: [u16; 18] = [
    841, 847, 848, 849, 912, 913, 914, 915, 952, 1028, 1029, 1030, 1031, 1032, 1033, 1034, 1035,
    1104,
];

/// `ProjectileID.Sets.IsAWhip[type]` (`ProjectileID.cs:193`).
pub fn is_whip(kind: u16) -> bool {
    IS_A_WHIP.contains(&kind)
}

/// `WhipTagEffect`'s general mark duration (`GameContent/Items/WhipTagEffect.cs:17-21`).
pub const TAG_DURATION_GENERIC: i32 = 240;
/// `WhipTagEffect_Firecracker.ProcDamageMultiplier`
/// (`GameContent/Items/WhipTagEffect_Firecracker.cs`, the static ctor).
pub const PROC_FIRECRACKER_MULTIPLIER: f32 = 2.75;
/// `WhipTagEffect_SwordWhip.MegaSlash`'s flat add
/// (`GameContent/Items/WhipTagEffect_SwordWhip.cs`).
pub const PROC_SWORDWHIP_FLAT: i32 = 30;
/// `WhipTagEffect_ElectricEel.CreateLightningBlast`'s per-bolt add (`num3`/`num4`).
pub const PROC_EEL_FLAT: i32 = 50;
/// The Constellation Whip's own projectile id (the proc gate's `type == 1034` check,
/// `WhipTagEffect_Constellation.cs`).
pub const CONSTELLATION_WHIP: u16 = 1034;
/// `Smolstar` (864) — the only non-1f `SummonTagDamageMultiplier`
/// (`ProjectileID.cs:194`).
pub const SMOLSTAR: u16 = 864;

/// `Projectile.GetWhipSettings(proj, out timeToFlyOut, out segments, out
/// rangeMultiplier)` (`Projectile.cs:48337-48419`). The `timeToFlyOut` inputs are the
/// *owner's* `itemAnimationMax` and the projectile's `MaxUpdates` (`1 + extraUpdates`);
/// `ai2` is `proj.ai[2]` and `is_preview_doll` is `proj.isAPreviewDisplayDoll`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WhipSettings {
    /// `timeToFlyOut` — the swing length in AI sub-steps.
    pub time_fly_out: f32,
    /// The control-point count minus one.
    pub segments: usize,
    /// The per-type reach multiplier (0.75 .. 2.2).
    pub range_multiplier: f32,
}

/// `Projectile.GetWhipSettings` (`Projectile.cs:48337-48419`).
pub fn get_whip_settings(
    kind: u16,
    animation_max_ticks: f32,
    max_updates: f32,
    ai2: f32,
    is_preview_doll: bool,
) -> WhipSettings {
    let mut time_fly_out = animation_max_ticks * max_updates;
    if is_preview_doll {
        time_fly_out = 60.0;
    }
    if ai2 >= 1000.0 {
        time_fly_out = ai2 - 1000.0;
    }
    let mut segments = 20usize;
    let mut range_multiplier = 1.0f32;
    match kind {
        1104 => {
            segments = 36;
            range_multiplier *= 0.75;
        }
        1028 => {
            segments = 12;
            range_multiplier *= 0.75;
        }
        841 => range_multiplier *= 0.75,
        1029 => range_multiplier *= 0.95,
        1030 => range_multiplier *= 0.85,
        914 => range_multiplier *= 1.0,
        1031 => range_multiplier *= 1.1,
        952 => range_multiplier *= 0.95,
        913 => range_multiplier *= 1.4,
        912 => range_multiplier *= 1.5,
        847 => {
            segments = 20;
            range_multiplier *= 1.75;
        }
        1032 => range_multiplier *= 1.8,
        848 => {
            segments = 40;
            range_multiplier *= 1.6;
        }
        849 => {
            segments = 30;
            range_multiplier *= 2.15;
        }
        1033 => {
            segments = 45;
            range_multiplier *= 2.0;
        }
        1034 => range_multiplier *= 2.2,
        1035 => {
            segments = 30;
            range_multiplier *= 2.2;
            if ai2 >= 10.0 {
                time_fly_out = 30.0 * max_updates;
            }
        }
        915 => {
            segments = 40;
            range_multiplier *= 2.2;
        }
        _ => {}
    }
    WhipSettings {
        time_fly_out,
        segments,
        range_multiplier,
    }
}

/// The `FillWhipControlPoints` inputs (`Projectile.cs:48278-48335`) — everything the
/// curve reads off the projectile and its owner. `held_use_animation` is the held item's
/// base `useAnimation` (`ContentSamples.ItemsByType[heldItem.type].useAnimation`, ×2 in
/// the C# line); `whip_range_multiplier` is `Player.whipRangeMultiplier` (1f default).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WhipCurveInputs {
    pub kind: u16,
    /// `proj.ai[0]` — the sub-step counter.
    pub ai0: f32,
    /// `FillWhipControlPoints`' `frameOffset` (0 for the live frame, −1 for the previous).
    pub frame_offset: i32,
    /// `proj.ai[1]` — the per-whip spin seed (forced to 1f for collision points).
    pub ai1: f32,
    /// `proj.spriteDirection` (±1).
    pub sprite_direction: i32,
    /// `proj.rotation` (the whip points are rotated by this about the arm).
    pub rotation: f32,
    /// `owner.GetArmPosition()`.
    pub arm_position: Vec2,
    pub held_use_animation: f32,
    pub whip_range_multiplier: f32,
    /// `proj.velocity.Length()` — the item's `shootSpeed` for the vanilla whips.
    pub velocity_len: f32,
    /// `getActualCollisionPoints` — forces the spin seed to 1f.
    pub get_actual_collision_points: bool,
}

/// `Projectile.FillWhipControlPoints` (`Projectile.cs:48278-48335`): the `segments + 1`
/// control points of the whip's curve, in the C# append order (the arm position first).
pub fn fill_whip_control_points(i: &WhipCurveInputs, settings: &WhipSettings) -> Vec<Vec2> {
    // `num = (proj.ai[0] + frameOffset) / timeToFlyOut`; the CobWhip's remap.
    let mut num = (i.ai0 + i.frame_offset as f32) / settings.time_fly_out;
    if i.kind == 1028 {
        num = num * 0.65 + 0.3;
    }
    let num2 = 0.5f32;
    let num3 = 1.0 + num2;
    let num4 = std::f32::consts::PI * 10.0 * (1.0 - num * num3) * (i.sprite_direction as f32)
        / settings.segments as f32;
    let mut num5 = num * num3;
    let mut num6 = 0.0f32;
    if num5 > 1.0 {
        num6 = (num5 - 1.0) / num2;
        num5 = 1.0 + (0.0 - 1.0) * num6;
    }
    let num7 = i.held_use_animation * 2.0 * num * i.whip_range_multiplier;
    let num8 = i.velocity_len * num7 * num5 * settings.range_multiplier / settings.segments as f32;
    let num9 = 1.0f32;
    let arm_position = i.arm_position;
    let mut vector = arm_position;
    let mut num10 = -std::f32::consts::PI / 2.0;
    let mut vector2 = vector;
    let mut num11 =
        0.0 + std::f32::consts::PI / 2.0 + std::f32::consts::PI / 2.0 * i.sprite_direction as f32;
    let mut vector3 = vector;
    let mut num12 = 0.0 + std::f32::consts::PI / 2.0;
    let mut control_points = Vec::with_capacity(settings.segments + 1);
    control_points.push(arm_position);
    let num13 = if i.get_actual_collision_points {
        1.0
    } else {
        i.ai1
    };
    for s in 0..settings.segments {
        let num14 = num4 * (s as f32 / settings.segments as f32) * num9 * num13;
        let vector4 = vector + Vec2::from_angle(num10) * num8;
        let vector5 = vector3 + Vec2::from_angle(num12) * (num8 * 2.0);
        let vector6 = vector2 + Vec2::from_angle(num11) * (num8 * 2.0);
        let num15 = 1.0 - num5;
        let num16 = 1.0 - num15 * num15;
        let lerp_outer = vector5.lerp(vector4, num16 * 0.9 + 0.1);
        let lerp_inner = vector6.lerp(lerp_outer, num16 * 0.7 + 0.3);
        let spinningpoint = arm_position + (lerp_inner - arm_position) * Vec2::new(1.0, num3);
        control_points.push(rotate_by(
            spinningpoint,
            i.rotation + 4.712389 * (num6 * num6) * i.sprite_direction as f32,
            arm_position,
        ));
        num10 += num14;
        num12 += num14;
        num11 += num14;
        vector = vector4;
        vector3 = vector5;
        vector2 = vector6;
    }
    control_points
}

/// `Vector2.RotatedBy(point, angle, center)` — rotate `point` about `center`.
pub fn rotate_by(point: Vec2, angle: f32, center: Vec2) -> Vec2 {
    let (sin, cos) = angle.sin_cos();
    let d = point - center;
    center + Vec2::new(d.x * cos - d.y * sin, d.x * sin + d.y * cos)
}

/// An integer rect (min corner + size), matching XNA `Rectangle`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }
    pub fn left(&self) -> i32 {
        self.x
    }
    pub fn right(&self) -> i32 {
        self.x + self.w
    }
    pub fn top(&self) -> i32 {
        self.y
    }
    pub fn bottom(&self) -> i32 {
        self.y + self.h
    }
    pub fn centered(cx: i32, cy: i32, w: i32, h: i32) -> Self {
        Self::new(cx - w / 2, cy - h / 2, w, h)
    }
    /// XNA `Rectangle.Inflate(horizontalAmount, verticalAmount)`: pushes the
    /// edges out by the given amounts (a negative amount shrinks them) —
    /// `X -= h`, `Width += 2h`, `Y -= v`, `Height += 2v`. Matches the C#
    /// `r.Inflate(-1, -1)` reading used at `Terraria/Chest.cs:31119`.
    pub fn inflate(&mut self, horizontal_amount: i32, vertical_amount: i32) {
        self.x -= horizontal_amount;
        self.y -= vertical_amount;
        self.w += horizontal_amount * 2;
        self.h += vertical_amount * 2;
    }
    /// XNA `Rectangle.Union`.
    pub fn union(self, other: Rect) -> Rect {
        let x = self.left().min(other.left());
        let y = self.top().min(other.top());
        Rect::new(
            x,
            y,
            self.right().max(other.right()) - x,
            self.bottom().max(other.bottom()) - y,
        )
    }
    /// XNA `Rectangle.Intersects` (strict: touching edges do not intersect).
    pub fn intersects(self, other: Rect) -> bool {
        other.left() < self.right()
            && self.left() < other.right()
            && other.top() < self.bottom()
            && self.top() < other.bottom()
    }
}

/// `Collision.CanHitLine`-free sweep: `Projectile.Colliding`'s whip arm
/// (`Projectile.cs:15716-15737`). `now` is the live frame's point list and `prev` the
/// previous frame's (the same list order); each per-point rect is `box` centred on the
/// `ToPoint()`-truncated point, and the two frames' rects are **unioned** before the
/// NPC test — a swept-span test, not an OR.
pub fn sweep_hits(now: &[Vec2], prev: &[Vec2], box_w: i32, box_h: i32, npc: Rect) -> bool {
    debug_assert_eq!(now.len(), prev.len());
    for s in 0..now.len() {
        let a = Rect::centered(now[s].x as i32, now[s].y as i32, box_w, box_h);
        let b = Rect::centered(prev[s].x as i32, prev[s].y as i32, box_w, box_h);
        if a.union(b).intersects(npc) {
            return true;
        }
    }
    false
}

/// The vanilla tag effect classes (`GameContent/Items/WhipTagEffect_*.cs`) — only the
/// arms whose math ports now carry a distinct id; the rest are the plain
/// `WhipTagEffect` body.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TagEffectClass {
    /// The plain `WhipTagEffect` (`TagDamage` + the optional crit/buff rows).
    Plain,
    /// `WhipTagEffect_Starcrash`.
    Starcrash,
    /// `WhipTagEffect_SwordWhip` (the +30 flat proc).
    SwordWhip,
    /// `WhipTagEffect_ViolentDisplayOfFlower`.
    ViolentDisplayOfFlower,
    /// `WhipTagEffect_DarkHarvest`.
    DarkHarvest,
    /// `WhipTagEffect_ElectricEel` (the +50 + 50·n flat proc).
    ElectricEel,
    /// `WhipTagEffect_Constellation` (TagDuration 480; the immediate proc enable).
    Constellation,
    /// `WhipTagEffect_Kaleidoscope`.
    Kaleidoscope,
    /// `WhipTagEffect_Possession`.
    Possession,
    /// `WhipTagEffect_Firecracker` (the ×2.75 proc multiplier, no `TagDamage`).
    Firecracker,
}

/// One row of `ItemID.Sets.UniqueTagEffects` (`ItemID.cs:1473-1580`). The C# rows carry
/// only the non-default initializers; the defaults are `TagDamage = 0`, `CritChance = 0`,
/// `PlayerBuffId = 0` and the general 240 duration (`WhipTagEffect.cs:17-21`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TagEffect {
    pub item_id: u16,
    pub class: TagEffectClass,
    pub tag_damage: i32,
    pub crit_chance: i32,
    pub tag_duration: i32,
    pub player_buff_id: i32,
    pub player_buff_time: i32,
    pub player_buff_applied_manually: bool,
}

/// `ItemID.Sets.UniqueTagEffects` (`ItemID.cs:1473-1580`), verbatim, in the C# order.
/// `TagDamage` is the `WhipTagEffect` initializer — the summon bonus per tagged minion
/// hit, NOT the item's damage.
pub const TAG_EFFECTS: [TagEffect; 18] = [
    TagEffect {
        item_id: 5688,
        class: TagEffectClass::Plain,
        tag_damage: 3,
        crit_chance: 0,
        tag_duration: TAG_DURATION_GENERIC,
        player_buff_id: 0,
        player_buff_time: 0,
        player_buff_applied_manually: false,
    },
    TagEffect {
        item_id: 5473,
        class: TagEffectClass::Plain,
        tag_damage: 3,
        crit_chance: 0,
        tag_duration: TAG_DURATION_GENERIC,
        player_buff_id: 365,
        player_buff_time: 180,
        player_buff_applied_manually: true,
    },
    TagEffect {
        item_id: 4672,
        class: TagEffectClass::Plain,
        tag_damage: 4,
        crit_chance: 0,
        tag_duration: TAG_DURATION_GENERIC,
        player_buff_id: 0,
        player_buff_time: 0,
        player_buff_applied_manually: false,
    },
    TagEffect {
        item_id: 5474,
        class: TagEffectClass::Plain,
        tag_damage: 5,
        crit_chance: 0,
        tag_duration: TAG_DURATION_GENERIC,
        player_buff_id: 0,
        player_buff_time: 0,
        player_buff_applied_manually: false,
    },
    TagEffect {
        item_id: 5475,
        class: TagEffectClass::Plain,
        tag_damage: 5,
        crit_chance: 0,
        tag_duration: TAG_DURATION_GENERIC,
        player_buff_id: 0,
        player_buff_time: 0,
        player_buff_applied_manually: false,
    },
    TagEffect {
        item_id: 5476,
        class: TagEffectClass::Starcrash,
        tag_damage: 2,
        crit_chance: 0,
        tag_duration: TAG_DURATION_GENERIC,
        player_buff_id: 0,
        player_buff_time: 0,
        player_buff_applied_manually: false,
    },
    TagEffect {
        item_id: 4913,
        class: TagEffectClass::Plain,
        tag_damage: 6,
        crit_chance: 0,
        tag_duration: TAG_DURATION_GENERIC,
        player_buff_id: 314,
        player_buff_time: 180,
        player_buff_applied_manually: false,
    },
    TagEffect {
        item_id: 5074,
        class: TagEffectClass::Plain,
        tag_damage: 7,
        crit_chance: 0,
        tag_duration: TAG_DURATION_GENERIC,
        player_buff_id: 0,
        player_buff_time: 0,
        player_buff_applied_manually: false,
    },
    TagEffect {
        item_id: 4911,
        class: TagEffectClass::Plain,
        tag_damage: 6,
        crit_chance: 0,
        tag_duration: TAG_DURATION_GENERIC,
        player_buff_id: 312,
        player_buff_time: 180,
        player_buff_applied_manually: true,
    },
    TagEffect {
        item_id: 4678,
        class: TagEffectClass::SwordWhip,
        tag_damage: 9,
        crit_chance: 0,
        tag_duration: TAG_DURATION_GENERIC,
        player_buff_id: 308,
        player_buff_time: 180,
        player_buff_applied_manually: false,
    },
    TagEffect {
        item_id: 5477,
        class: TagEffectClass::ViolentDisplayOfFlower,
        tag_damage: 9,
        crit_chance: 0,
        tag_duration: TAG_DURATION_GENERIC,
        player_buff_id: 0,
        player_buff_time: 0,
        player_buff_applied_manually: false,
    },
    TagEffect {
        item_id: 4680,
        class: TagEffectClass::DarkHarvest,
        tag_damage: 15,
        crit_chance: 0,
        tag_duration: TAG_DURATION_GENERIC,
        player_buff_id: 311,
        player_buff_time: 180,
        player_buff_applied_manually: false,
    },
    TagEffect {
        item_id: 4679,
        class: TagEffectClass::Plain,
        tag_damage: 8,
        crit_chance: 10,
        tag_duration: TAG_DURATION_GENERIC,
        player_buff_id: 0,
        player_buff_time: 0,
        player_buff_applied_manually: false,
    },
    TagEffect {
        item_id: 5478,
        class: TagEffectClass::ElectricEel,
        tag_damage: 12,
        crit_chance: 5,
        tag_duration: TAG_DURATION_GENERIC,
        player_buff_id: 0,
        player_buff_time: 0,
        player_buff_applied_manually: false,
    },
    TagEffect {
        item_id: 5479,
        class: TagEffectClass::Constellation,
        tag_damage: 15,
        crit_chance: 15,
        tag_duration: 480,
        player_buff_id: 0,
        player_buff_time: 0,
        player_buff_applied_manually: false,
    },
    TagEffect {
        item_id: 4914,
        class: TagEffectClass::Kaleidoscope,
        tag_damage: 20,
        crit_chance: 10,
        tag_duration: TAG_DURATION_GENERIC,
        player_buff_id: 0,
        player_buff_time: 0,
        player_buff_applied_manually: false,
    },
    TagEffect {
        item_id: 5480,
        class: TagEffectClass::Possession,
        tag_damage: 25,
        crit_chance: 10,
        tag_duration: TAG_DURATION_GENERIC,
        player_buff_id: 0,
        player_buff_time: 0,
        player_buff_applied_manually: false,
    },
    TagEffect {
        item_id: 4912,
        class: TagEffectClass::Firecracker,
        tag_damage: 0,
        crit_chance: 0,
        tag_duration: TAG_DURATION_GENERIC,
        player_buff_id: 0,
        player_buff_time: 0,
        player_buff_applied_manually: false,
    },
];

/// `ItemID.Sets.UniqueTagEffects[item.type]` — the null check is the caller's.
pub fn tag_effect(item: u16) -> Option<&'static TagEffect> {
    TAG_EFFECTS.iter().find(|t| t.item_id == item)
}

/// `ProjectileID.Sets.SummonTagDamageMultiplier[type]` (`ProjectileID.cs:194`): the
/// 1f default with the single Smolstar (864) row at 0.669f.
pub fn summon_tag_damage_multiplier(proj_kind: u16) -> f32 {
    if proj_kind == SMOLSTAR { 0.669 } else { 1.0 }
}

/// `TagDamageChanges` (`GameContent/Items/TagDamageChanges.cs`) — the per-hit damage
/// accumulator every active tag effect modifies. `None` carries the 1f multiplier.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TagDamageChanges {
    pub add_projectile_tag_damage: bool,
    pub added_base_damage: i32,
    pub added_flat_damage: i32,
    pub total_damage_multiplier: f32,
    pub crit: Option<bool>,
    pub highest_added_base_damage: i32,
}

impl Default for TagDamageChanges {
    fn default() -> Self {
        Self::none()
    }
}

impl TagDamageChanges {
    /// `TagDamageChanges.None` — the 1f-multiplier default.
    pub fn none() -> Self {
        Self {
            add_projectile_tag_damage: false,
            added_base_damage: 0,
            added_flat_damage: 0,
            total_damage_multiplier: 1.0,
            crit: None,
            highest_added_base_damage: 0,
        }
    }
}

/// `WhipTagEffect.ModifyTaggedHit` (`GameContent/Items/WhipTagEffect.cs:61-73`) — the
/// base "tagged hit" arm: the flat `TagDamage` add with the highest tracking and the
/// crit roll. The roll runs even at `CritChance == 0` (the C# `Main.rand.Next(100)`
/// always fires — the stream must match).
pub fn modify_tagged_hit(e: &TagEffect, changes: &mut TagDamageChanges, rng: &mut dyn DropRng) {
    changes.add_projectile_tag_damage = true;
    changes.added_base_damage += e.tag_damage;
    if changes.highest_added_base_damage < e.tag_damage {
        changes.highest_added_base_damage = e.tag_damage;
    }
    if rng.next_range(1, 101) < e.crit_chance {
        changes.crit = Some(true);
    }
}

/// `WhipTagEffect_*.ModifyProcHit` (`GameContent/Items/`) — the proc hit's pure arms:
/// the SwordWhip's +30 flat, the Firecracker's ×2.75 multiplier and the ElectricEel's
/// `50 + 50·n` (n = the nearby-NPC count the caller counted, the chain damage itself is
/// a documented follow-up). The remaining classes' proc bodies only spawn projectiles
/// or particles.
pub fn modify_proc_hit(e: &TagEffect, changes: &mut TagDamageChanges, nearby_count: i32) {
    match e.class {
        TagEffectClass::SwordWhip => changes.added_flat_damage += PROC_SWORDWHIP_FLAT,
        TagEffectClass::Firecracker => {
            changes.total_damage_multiplier *= PROC_FIRECRACKER_MULTIPLIER
        }
        TagEffectClass::ElectricEel => {
            changes.added_flat_damage += PROC_EEL_FLAT + PROC_EEL_FLAT * nearby_count
        }
        _ => {}
    }
}

/// `UniqueTagEffect.OnProcHit`'s return (`GameContent/Items/`): whether the proc hit
/// consumes the proc window (`TagEffectState.ClearProcOnNPC`, `TagEffectState.cs:114-117`).
/// The plain whips, the Kaleidoscope and the DarkHarvest leave the base `false`; the
/// Constellation consumes only for its own projectile type.
pub fn proc_hit_consumes(e: &TagEffect, proj_kind: u16) -> bool {
    match e.class {
        TagEffectClass::Starcrash
        | TagEffectClass::SwordWhip
        | TagEffectClass::ViolentDisplayOfFlower
        | TagEffectClass::ElectricEel
        | TagEffectClass::Possession
        | TagEffectClass::Firecracker => true,
        TagEffectClass::Constellation => proj_kind == CONSTELLATION_WHIP,
        TagEffectClass::Plain | TagEffectClass::DarkHarvest | TagEffectClass::Kaleidoscope => false,
    }
}

/// `WhipTagEffect_Constellation.OnTaggedHit` —
/// `owner.TagEffectStack.TryEnableProcOnNPC(5479, npcHit)`: the Constellation's tag
/// arms the proc window on the very hit that applied it.
pub fn constellation_enables_proc_on_tag(e: &TagEffect) -> bool {
    e.class == TagEffectClass::Constellation
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile_light::FastRandom;

    fn rng() -> FastRandom {
        FastRandom::new(42)
    }

    /// `DefaultToWhip()`'s projectile slice (`Projectile.cs:10704-10718`): 18×18,
    /// aiStyle 165, penetrate −1, no tile collide, `extraUpdates` 1 (2 for 847/849),
    /// `usesLocalNPCImmunity`, `localNPCHitCooldown = -1`.
    fn whip_max_updates(kind: u16) -> f32 {
        if kind == 847 || kind == 849 { 3.0 } else { 2.0 }
    }

    #[test]
    fn the_is_a_whip_set_is_verbatim() {
        assert_eq!(IS_A_WHIP.len(), 18);
        for kind in IS_A_WHIP {
            assert!(is_whip(kind), "{kind} must be a whip");
        }
        // A handful of non-whips, incl. the neighbours the whip arms share with melee.
        assert!(!is_whip(864));
        assert!(!is_whip(3));
        assert!(!is_whip(213));
        assert!(!is_whip(0));
    }

    #[test]
    fn the_whip_settings_table_is_verbatim() {
        // Default animation (30) × MaxUpdates, no ai2, no preview doll.
        let s = |kind: u16| get_whip_settings(kind, 30.0, whip_max_updates(kind), 0.0, false);
        assert_eq!(s(841).segments, 20);
        assert_eq!(s(841).range_multiplier, 0.75);
        assert_eq!(s(1104).segments, 36);
        assert_eq!(s(1104).range_multiplier, 0.75);
        assert_eq!(s(1028).segments, 12);
        assert_eq!(s(1029).range_multiplier, 0.95);
        assert_eq!(s(1030).range_multiplier, 0.85);
        assert_eq!(s(1031).range_multiplier, 1.1);
        assert_eq!(s(952).range_multiplier, 0.95);
        assert_eq!(s(913).range_multiplier, 1.4);
        assert_eq!(s(912).range_multiplier, 1.5);
        assert_eq!(s(847).range_multiplier, 1.75);
        assert_eq!(s(1032).range_multiplier, 1.8);
        assert_eq!(s(848).segments, 40);
        assert_eq!(s(848).range_multiplier, 1.6);
        assert_eq!(s(849).segments, 30);
        assert_eq!(s(849).range_multiplier, 2.15);
        assert_eq!(s(1033).segments, 45);
        assert_eq!(s(1034).range_multiplier, 2.2);
        assert_eq!(s(1035).segments, 30);
        assert_eq!(s(1035).range_multiplier, 2.2);
        assert_eq!(s(915).segments, 40);
        assert_eq!(s(915).range_multiplier, 2.2);
        // `timeToFlyOut = itemAnimationMax * MaxUpdates` (`:48339`).
        assert_eq!(s(841).time_fly_out, 60.0);
        assert_eq!(s(847).time_fly_out, 90.0);
        assert_eq!(s(849).time_fly_out, 90.0);
    }

    #[test]
    fn the_time_fly_out_overrides_match() {
        // `isAPreviewDisplayDoll` → 60 (`:48340-48343`).
        assert_eq!(
            get_whip_settings(841, 30.0, 2.0, 0.0, true).time_fly_out,
            60.0
        );
        // `ai[2] >= 1000` → `ai[2] - 1000` (`:48344-48347`).
        assert_eq!(
            get_whip_settings(841, 30.0, 2.0, 1035.0, false).time_fly_out,
            35.0
        );
        // `case 1035` with `ai[2] >= 10` → `30 * MaxUpdates` (`:48410-48417`).
        assert_eq!(
            get_whip_settings(1035, 30.0, 2.0, 11.0, false).time_fly_out,
            60.0
        );
        // Below 10 the override does not fire.
        assert_eq!(
            get_whip_settings(1035, 30.0, 2.0, 9.0, false).time_fly_out,
            60.0
        );
    }

    fn curve_inputs(kind: u16, ai0: f32, frame_offset: i32) -> WhipCurveInputs {
        WhipCurveInputs {
            kind,
            ai0,
            frame_offset,
            ai1: 0.0,
            sprite_direction: 1,
            rotation: 0.0,
            arm_position: Vec2::new(100.0, 100.0),
            held_use_animation: 30.0,
            whip_range_multiplier: 1.0,
            velocity_len: 4.0,
            get_actual_collision_points: false,
        }
    }

    #[test]
    fn the_curve_has_segments_plus_one_points_and_full_reach() {
        let settings = get_whip_settings(841, 30.0, 2.0, 0.0, false);
        // `num = ai0 / timeFlyOut` at the tip tick (60) → progress 1.0, `num5 = 1.5`
        // clamped by the retract (num6 = 1) → num5 = 0 at the extreme; use a mid tick
        // so the extension is inside its envelope.
        let pts = fill_whip_control_points(&curve_inputs(841, 30.0, 0), &settings);
        assert_eq!(pts.len(), settings.segments + 1);
        // The first point is the arm position (`:48311`).
        assert_eq!(pts[0], Vec2::new(100.0, 100.0));
        // Mid-extension: progress 0.5 → num5 = 0.75 (< 1, no retract) →
        // reach = 4 · (30·2·0.5·1) · 0.75 · 0.75 / 20 = 1.35 px per segment step.
        let reach = 4.0 * 60.0 * 0.5 * 0.75 * 0.75 / 20.0;
        let d = (pts[1] - pts[0]).length();
        assert!(
            (d - reach).abs() < reach,
            "first segment step {d} should be on the reach scale {reach}"
        );
    }

    #[test]
    fn the_curve_retracts_past_progress_one() {
        let settings = get_whip_settings(841, 30.0, 2.0, 0.0, false);
        // Progress past 2/3 engages the retract: the points come back toward the arm.
        let mid = fill_whip_control_points(&curve_inputs(841, 40.0, 0), &settings);
        let late = fill_whip_control_points(&curve_inputs(841, 59.0, 0), &settings);
        let tip_mid = (mid[settings.segments] - mid[0]).length();
        let tip_late = (late[settings.segments] - late[0]).length();
        assert!(
            tip_late < tip_mid,
            "the tip must retract ({tip_mid} -> {tip_late})"
        );
    }

    #[test]
    fn the_cob_whip_remaps_the_progress() {
        let settings = get_whip_settings(1028, 30.0, 2.0, 0.0, false);
        // `Remap(num, 0, 1, 0.3, 0.95)`: ai0 0 → progress 0.3, ai0 = timeFlyOut → 0.95.
        // The reach scale `num7·num5` is 8.1 px at 0.3 (0.45 extension) and 8.55 at
        // 0.95 (the retract leaves 0.15 of the 57 px arm) — pin the golden tip vectors.
        let a = fill_whip_control_points(&curve_inputs(1028, 0.0, 0), &settings);
        let b = fill_whip_control_points(&curve_inputs(1028, settings.time_fly_out, 0), &settings);
        let tip_a = a[settings.segments];
        let tip_b = b[settings.segments];
        // The golden tip vectors: ai0=0 (progress remapped to 0.3) and the full swing
        // (0.95) — the reach arm `num7·num5` is 8.1 px vs 8.55 px (the retract leaves
        // 0.15 of the 57 px arm), so the two stay on the same scale.
        let golden_a = Vec2::new(89.70895, 94.73491);
        let golden_b = Vec2::new(129.75433, 89.2903);
        assert!((tip_a - golden_a).length() < 0.05, "ai0=0 tip {:?}", tip_a);
        assert!(
            (tip_b - golden_b).length() < 0.05,
            "full-swing tip {:?}",
            tip_b
        );
    }

    #[test]
    fn the_sweep_spans_the_gap_between_frames() {
        let settings = get_whip_settings(841, 30.0, 2.0, 0.0, false);
        let inputs_now = curve_inputs(841, 30.0, 0);
        let mut inputs_prev = inputs_now;
        inputs_prev.frame_offset = -1;
        let now = fill_whip_control_points(&inputs_now, &settings);
        let prev = fill_whip_control_points(&inputs_prev, &settings);
        let box_ = 18;
        // An NPC box that only the *span* between the two frames' rects covers: put it
        // in the gap past the current tip, inside the previous frame's union span.
        let tip_now = now[settings.segments];
        let tip_prev = prev[settings.segments];
        let gap = Rect::centered(
            ((tip_now.x + tip_prev.x) * 0.5) as i32,
            ((tip_now.y + tip_prev.y) * 0.5) as i32,
            4,
            4,
        );
        assert!(
            sweep_hits(&now, &prev, box_, box_, gap),
            "the swept span must hit an NPC between the frames"
        );
        // An NPC far away never hits.
        let far = Rect::new(9000, 9000, 18, 18);
        assert!(!sweep_hits(&now, &prev, box_, box_, far));
    }

    #[test]
    fn the_tag_table_is_verbatim() {
        let td = |item: u16| tag_effect(item).unwrap();
        assert_eq!(td(5688).tag_damage, 3);
        assert_eq!(td(5473).tag_damage, 3);
        assert_eq!(td(4672).tag_damage, 4);
        assert_eq!(td(5474).tag_damage, 5);
        assert_eq!(td(5475).tag_damage, 5);
        assert_eq!(td(5476).tag_damage, 2);
        assert_eq!(td(4913).tag_damage, 6);
        assert_eq!(td(5074).tag_damage, 7);
        assert_eq!(td(4911).tag_damage, 6);
        assert_eq!(td(4678).tag_damage, 9);
        assert_eq!(td(5477).tag_damage, 9);
        assert_eq!(td(4680).tag_damage, 15);
        assert_eq!(td(4679).tag_damage, 8);
        assert_eq!(td(5478).tag_damage, 12);
        assert_eq!(td(5479).tag_damage, 15);
        assert_eq!(td(4914).tag_damage, 20);
        assert_eq!(td(5480).tag_damage, 25);
        // The Firecracker sets no TagDamage — its value is the proc multiplier.
        assert_eq!(td(4912).tag_damage, 0);
        assert_eq!(tag_effect(4), None);
        // The durations: the general 240 and the Constellation's 480.
        assert_eq!(td(4672).tag_duration, 240);
        assert_eq!(td(5479).tag_duration, 480);
        // The crit rows.
        assert_eq!(td(4679).crit_chance, 10);
        assert_eq!(td(5478).crit_chance, 5);
        assert_eq!(td(5479).crit_chance, 15);
        assert_eq!(td(4914).crit_chance, 10);
        assert_eq!(td(5480).crit_chance, 10);
        // The player buff rows (the two "manual" rows grant in their own on-hit code).
        assert_eq!(td(5473).player_buff_id, 365);
        assert!(td(5473).player_buff_applied_manually);
        assert_eq!(td(4913).player_buff_id, 314);
        assert!(!td(4913).player_buff_applied_manually);
        assert_eq!(td(4911).player_buff_id, 312);
        assert_eq!(td(4678).player_buff_id, 308);
        assert_eq!(td(4680).player_buff_id, 311);
        assert_eq!(td(5478).class, TagEffectClass::ElectricEel);
        assert_eq!(td(4678).class, TagEffectClass::SwordWhip);
        assert_eq!(td(4912).class, TagEffectClass::Firecracker);
    }

    #[test]
    fn the_summon_tag_damage_multiplier_is_verbatim() {
        for kind in IS_A_WHIP {
            assert_eq!(summon_tag_damage_multiplier(kind), 1.0);
        }
        assert_eq!(summon_tag_damage_multiplier(864), 0.669);
        assert_eq!(summon_tag_damage_multiplier(1), 1.0);
    }

    /// A `DropRng` that returns a fixed `next(1,101)` roll — the crit arm has no other
    /// consumer, so the deterministic roll drives the decision directly.
    struct FixedRoll(i32);
    impl DropRng for FixedRoll {
        fn next_range(&mut self, _min: i32, _max: i32) -> i32 {
            self.0
        }
        fn next_float(&mut self) -> f32 {
            0.0
        }
    }

    #[test]
    fn the_tagged_hit_arm_accumulates_and_rolls_crit() {
        let mut changes = TagDamageChanges::none();
        let e = tag_effect(4914).unwrap(); // Rainbow: TagDamage 20, crit 10.
        // Roll 4 < 10 → crit set; roll 95 ≥ 10 → left unset.
        modify_tagged_hit(e, &mut changes, &mut FixedRoll(4));
        assert!(changes.add_projectile_tag_damage);
        assert_eq!(changes.added_base_damage, 20);
        assert_eq!(changes.highest_added_base_damage, 20);
        assert_eq!(changes.crit, Some(true));
        let mut no_crit = TagDamageChanges::none();
        modify_tagged_hit(e, &mut no_crit, &mut FixedRoll(95));
        assert_eq!(no_crit.crit, None);
        assert_eq!(no_crit.added_base_damage, 20);
    }

    #[test]
    fn the_tag_damage_math_reproduces_the_c_sharp_line() {
        // The `Projectile.cs:12924` line, verbatim shape: the inner float arm truncates
        // to int, then joins `num10 + added_flat`, then × the total multiplier.
        fn c_sharp_line(base: i32, changes: &TagDamageChanges, bonus_tag: i32, proj: u16) -> i32 {
            let inner = (changes.highest_added_base_damage as f32
                + (changes.added_base_damage - changes.highest_added_base_damage) as f32 * 1.0
                + bonus_tag as f32)
                * summon_tag_damage_multiplier(proj);
            ((base + inner as i32 + changes.added_flat_damage) as f32
                * changes.total_damage_multiplier) as i32
        }
        let mut r = rng();
        // SwordWhip tag (9) + its proc (+30 flat) on a 1f-multiplier weapon.
        let mut changes = TagDamageChanges::none();
        let e = tag_effect(4678).unwrap();
        modify_tagged_hit(e, &mut changes, &mut r);
        modify_proc_hit(e, &mut changes, 0);
        assert_eq!(c_sharp_line(100, &changes, 0, 1), 139);
        // Firecracker's ×2.75 multiplier arm (TagDamage 0, no flat).
        let mut fc = TagDamageChanges::none();
        let fce = tag_effect(4912).unwrap();
        modify_tagged_hit(fce, &mut fc, &mut r);
        modify_proc_hit(fce, &mut fc, 0);
        assert_eq!(fc.total_damage_multiplier, 2.75);
        assert_eq!(fc.added_flat_damage, 0);
        assert_eq!(fc.added_base_damage, 0);
        assert_eq!(c_sharp_line(100, &fc, 0, 1), 275);
        // ElectricEel's flat arm with 2 nearby NPCs: 50 + 50·2.
        let mut ee = TagDamageChanges::none();
        let ee_e = tag_effect(5478).unwrap();
        modify_proc_hit(ee_e, &mut ee, 2);
        assert_eq!(ee.added_flat_damage, 150);
        // The Smolstar row: inner 9 · 0.669 = 6.021 → 6 (truncated) → 100+6+30 = 136.
        assert_eq!(c_sharp_line(100, &changes, 0, 864), 136);
    }

    #[test]
    fn the_proc_consumes_gate_is_verbatim() {
        let consumes = |item: u16, proj: u16| proc_hit_consumes(tag_effect(item).unwrap(), proj);
        assert!(consumes(4678, 1)); // SwordWhip
        assert!(consumes(4912, 1)); // Firecracker
        assert!(consumes(5478, 1)); // ElectricEel
        assert!(consumes(5477, 1)); // Flower
        assert!(consumes(5476, 1)); // Starcrash
        assert!(consumes(5480, 1)); // Possession
        assert!(consumes(5479, 1034)); // Constellation on its own whip
        assert!(!consumes(5479, 1)); // Constellation on another shot
        assert!(!consumes(4672, 1)); // the plain whips keep the window
        assert!(!consumes(4680, 1)); // DarkHarvest
        assert!(!consumes(4914, 1)); // Kaleidoscope
    }

    #[test]
    fn the_constellation_tag_arms_the_proc_window() {
        assert!(constellation_enables_proc_on_tag(tag_effect(5479).unwrap()));
        assert!(!constellation_enables_proc_on_tag(
            tag_effect(4672).unwrap()
        ));
    }
}
