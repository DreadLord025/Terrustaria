//! The broadsword (useStyle 1) melee swing interaction — the hitbox half of
//! `Player.ItemCheck` (`Player.cs:43644-43686`), the arc half of
//! `ItemCheck_ApplyUseStyle` (`:50686-50864`) and the per-NPC hit cooldowns
//! (`:44212-44239`). S1-P-i04 landed the module UNWIRED (the
//! `npc_ai_town.rs` precedent); W4-a1 phase 1 wired the hitbox half into
//! `world_sim/editing.rs::melee_swing_tick` — the useStyle-1 swing-rect
//! scan that replaced the cursor-point stand-in (the cooldown/`attack_cd`
//! pacing pass is W4-a1 phase 2).
//!
//! The render-side arc math (`item_rotation`/`item_location`,
//! `render/player_layers.rs`) was ported by S1-G-17/18; this module re-ports
//! the same `ItemCheck_ApplyUseStyle` arms with the RAW `itemAnimation` tick
//! counters the C# compares against (the draw's normalised fraction drifts
//! off the C# `0.333`/`0.666` double thresholds on the boundary ticks) plus
//! the pieces S1-G-17 left out: the Bladed Glove claw arm, the per-item
//! nudges, the `UsesBetterMeleeItemLocation` offset and the gravDir mirror.
//!
//! RNG: nothing in this module draws. The C# hit path's draws
//! (`Main.rand.Next(1, 101)` crit `:44298`, `Main.DamageVar` `:44342`) stay
//! with the caller in C# order — the witness tests mirror them on
//! `UnifiedRandom` clones.
#![allow(dead_code)] // the arc/particle halves still await their render/pacing seams (W4-a1 phase 2)

/// A C# `Rectangle` — the melee `itemRectangle` is integer arithmetic
/// throughout (`(int)` truncation toward zero, `Player.cs:45102-45159`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MeleeRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl MeleeRect {
    /// `Rectangle.Intersects` (XNA): strict inequalities — touching edges do
    /// NOT intersect.
    pub(crate) fn intersects(&self, other: &MeleeRect) -> bool {
        other.x < self.x + self.width
            && self.x < other.x + other.width
            && other.y < self.y + self.height
            && self.y < other.y + other.height
    }

    /// `Rectangle.TopLeft/TopRight/BottomLeft/BottomRight` as f32 corners —
    /// `Utils.LineRectangleDistance` (`Utils.cs:829-832`) reads the exclusive
    /// right/bottom edges.
    fn corners(&self) -> [[f32; 2]; 4] {
        let (x, y) = (self.x as f32, self.y as f32);
        let (r, b) = (x + self.width as f32, y + self.height as f32);
        [[x, y], [r, y], [x, b], [r, b]]
    }
}

/// `ItemID.Sets.UsesBetterMeleeItemLocation` (`ItemID.cs:80`, `:348`):
/// `Factory.CreateBoolSet(false, 426)` — the Breaker Blade alone.
pub(crate) fn uses_better_melee_item_location(item_type: i32) -> bool {
    item_type == 426
}

/// `Item.claw` — allocated empty (`Item.cs:49976`), the single writer is
/// `Main.cs:6793-6795`: `case 1827: Item.claw[item.type] = true` (the Bladed
/// Glove).
pub(crate) fn is_claw(item_type: i32) -> bool {
    item_type == 1827
}

/// `ItemID.Sets.NoMeleeSpeedBonus` (`ItemID.cs:64`, `:340`):
/// `Factory.CreateBoolSet(false, 3821)` — the Terragrim alone.
pub(crate) fn no_melee_speed_bonus(item_type: i32) -> bool {
    item_type == 3821
}

/// `ItemID.Sets.BonusMeleeSpeedMultiplier` (`ItemID.cs:256`, `:1350`):
/// `CreateFloatSet(1f, 1827f, 0.5f, 3013f, 0.25f, 3106f, 0.33f, 757f,
/// 0.75f)` — the Bladed Glove, the Fetid Baghnakhs, the Psycho Knife and
/// the Beam Sword scale only a fraction of the player's melee-speed bonus
/// (`Player.cs:25912`).
pub(crate) fn bonus_melee_speed_multiplier(item_type: i32) -> f32 {
    match item_type {
        1827 => 0.5,
        3013 => 0.25,
        3106 => 0.33,
        757 => 0.75,
        _ => 1.0,
    }
}

/// `Player.TurnAttackSpeedToUseTimeMultiplier` (`Player.cs:29028-29039`),
/// the `CapAttackSpeeds` fold (`:29021-29026`): the accumulated attack-speed
/// bonus (1 + sum) caps at 3 then INVERTS into the use-time multiplier —
/// faster attacks mean a SHORTER animation.
pub(crate) fn attack_speed_to_use_time_multiplier(speed: f32) -> f32 {
    let mut speed = speed;
    if speed > 3.0 {
        speed = 3.0;
    }
    if speed != 0.0 {
        speed = 1.0 / speed;
    }
    speed
}

/// `Player.ApplyItemAnimation`'s melee arm (`Player.cs:4463-4497` →
/// `SetItemAnimation(baseFrames, multiplier)`, `:4453-4461`): a melee item
/// outside `NoMeleeSpeedBonus` swings `useAnimation * meleeSpeed` ticks,
/// `(int)` truncated and floored at 1 when the base is positive. The
/// per-item bonus fold (`:25912`: `meleeSpeed = 1 + (meleeSpeed - 1) *
/// BonusMeleeSpeedMultiplier[type]`) applies BEFORE the cap/invert — the
/// caller passes the accumulated 1+bonus and the held type. Returns the
/// animation tick count (`itemAnimation = itemAnimationMax = val`,
/// `:4449-4450`).
pub(crate) fn melee_animation_frames(
    item_type: i32,
    use_animation: i32,
    melee_speed_accumulated: f32,
) -> i32 {
    let speed = 1.0
        + (melee_speed_accumulated - 1.0) * bonus_melee_speed_multiplier(item_type);
    let multiplier = attack_speed_to_use_time_multiplier(speed);
    let mut val = (use_animation as f32 * multiplier) as i32;
    if use_animation > 0 {
        val = val.max(1);
    }
    val
}

/// `Player.GetAdjustedItemScale` (`Player.cs:42566-42574`) +
/// `ApplyMeleeScale` (`:42576-42582`): `item.scale`, times 1.1 only when the
/// item is melee AND `meleeScaleGlove` (the port has no glove flag owner —
/// `accessory_combat.rs` documents it unported — so the caller passes the
/// flag through).
pub(crate) fn adjusted_item_scale(scale: f32, melee: bool, melee_scale_glove: bool) -> f32 {
    let mut scale = scale;
    if melee && melee_scale_glove {
        scale *= 1.1;
    }
    scale
}

/// The useStyle-1 swing rotation (`Player.cs:50858`, also `:50691`/`:50710`):
/// `itemRotation = (itemAnimation / itemAnimationMax - 0.5) * (-direction) *
/// 3.5 - direction * 0.3`. A 3.5 rad sweep: -2.05*dir at the swing start
/// (`itemAnimation == itemAnimationMax`), +1.45*dir at the end.
pub(crate) fn swing_rotation(item_animation: i32, item_animation_max: i32, direction: i32) -> f32 {
    let dir = direction as f32;
    (item_animation as f32 / item_animation_max as f32 - 0.5) * -dir * 3.5 - dir * 0.3
}

/// The per-third width-offset `num` cascades of the generic useStyle-1 arm
/// (`Player.cs:50712-50853`). `third` is 0 = swing END
/// (`itemAnimation < max * 0.333`), 1 = middle (`< max * 0.666`), 2 = start.
/// The C# threshold comparisons run in DOUBLE (`(double)itemAnimation <
/// (double)itemAnimationMax * 0.333`), so the boundary ticks (e.g. anim 6.66
/// of max 20) fall where the f64 product puts them, not an f32 rounding.
pub(crate) fn swing_third(item_animation: i32, item_animation_max: i32) -> u8 {
    let anim = item_animation as f64;
    let max = item_animation_max as f64;
    if anim < max * 0.333 {
        0
    } else if anim < max * 0.666 {
        1
    } else {
        2
    }
}

/// The generic arm's per-third width cascade (`Player.cs:50717-50741`,
/// `:50748-50772`, `:50800-50828`): `third` 0 = swing END
/// (`10 / >32:14 / >=52:24 / >=64:28 / >=92:38`, hammers +8, Keybrand +12),
/// 1 = middle (`10 / >32:18 / ...`, +4/+6), 2 = start (`6 / >32:14 /
/// >=48:18 / ...`, +4/+8). Shared with the draw's `item_location`
/// (`render/player_layers.rs`) so the two never drift.
pub(crate) fn swing_width_num(third: u8, frame_w: f32, item_type: i32) -> f32 {
    let hammer = matches!(item_type, 2330 | 2320 | 2341);
    let keybrand = item_type == 671;
    match third {
        0 => {
            let mut num = 10.0f32;
            if frame_w > 32.0 {
                num = 14.0;
            }
            if frame_w >= 52.0 {
                num = 24.0;
            }
            if frame_w >= 64.0 {
                num = 28.0;
            }
            if frame_w >= 92.0 {
                num = 38.0;
            }
            if hammer {
                num += 8.0;
            }
            if keybrand {
                num += 12.0;
            }
            num
        }
        1 => {
            let mut num = 10.0f32;
            if frame_w > 32.0 {
                num = 18.0;
            }
            if frame_w >= 52.0 {
                num = 24.0;
            }
            if frame_w >= 64.0 {
                num = 28.0;
            }
            if frame_w >= 92.0 {
                num = 38.0;
            }
            if hammer {
                num += 4.0;
            }
            if keybrand {
                num += 6.0;
            }
            num
        }
        _ => {
            let mut num = 6.0f32;
            if frame_w > 32.0 {
                num = 14.0;
            }
            if frame_w >= 48.0 {
                num = 18.0;
            }
            if frame_w >= 52.0 {
                num = 24.0;
            }
            if frame_w >= 64.0 {
                num = 28.0;
            }
            if frame_w >= 92.0 {
                num = 38.0;
            }
            if hammer {
                num += 4.0;
            }
            if keybrand {
                num += 8.0;
            }
            num
        }
    }
}

/// The generic arm's per-third height cascade (`Player.cs:50774-50794` for
/// the middle, `:50830-50850` for the start; the end third's Y is the fixed
/// `position.Y + 24`, `:50743`). Middle: `10 / >32:8 / >52:12 / >64:14`,
/// hammers +4, Keybrand +10. Start: `10 / >32:10 / >52:12 / >64:14`,
/// hammers +4, Keybrand +8.
pub(crate) fn swing_height_num(third: u8, frame_h: f32, item_type: i32) -> f32 {
    let hammer = matches!(item_type, 2330 | 2320 | 2341);
    let keybrand = item_type == 671;
    let mut num = 10.0f32;
    if third == 1 && frame_h > 32.0 {
        num = 8.0;
    }
    if frame_h > 52.0 {
        num = 12.0;
    }
    if frame_h > 64.0 {
        num = 14.0;
    }
    if hammer {
        num += 4.0;
    }
    if keybrand {
        num += if third == 1 { 10.0 } else { 8.0 };
    }
    num
}

/// The generic arm's `itemLocation` + `itemRotation`
/// (`Player.cs:50712-50864`), the claw arm (`:50693-50711`) and the gravDir
/// mirror (`:50860-50864`). `pos` is `base.position`, `player_w`/`player_h`
/// the player hitbox (20x42), `frame_w`/`frame_h` the held item's
/// `Item.GetDrawHitbox` frame (`Item.cs:49595-49632` — the texture frame, NOT
/// `Item.width/height`), `mount_offset` the mount's Y offset (0 unmounted).
/// Returns `(itemLocation, itemRotation)`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn swing_item_placement(
    item_type: i32,
    item_animation: i32,
    item_animation_max: i32,
    direction: i32,
    grav_dir: f32,
    pos: [f32; 2],
    player_w: f32,
    player_h: f32,
    frame_w: f32,
    frame_h: f32,
    mount_offset: f32,
) -> ([f32; 2], f32) {
    let dir = direction as f32;
    let third = swing_third(item_animation, item_animation_max);
    let mut loc;
    if is_claw(item_type) {
        // `:50695-50709`: the claw's three hand positions.
        loc = match third {
            0 => [
                pos[0] + player_w * 0.5 + (frame_w * 0.5 - 10.0) * dir,
                pos[1] + 26.0 + mount_offset,
            ],
            1 => [
                pos[0] + player_w * 0.5 + (frame_w * 0.5 - 8.0) * dir,
                pos[1] + 24.0 + mount_offset,
            ],
            _ => [
                pos[0] + player_w * 0.5 - (frame_w * 0.5 - 6.0) * dir,
                pos[1] + 20.0 + mount_offset,
            ],
        };
    } else {
        // The `zero` nudge of `UsesBetterMeleeItemLocation` (`:50744`,
        // `:50796`, `:50852`), applied at `:50854-50857` scaled by
        // `Directions` = `(direction, gravDir)` (`Player.cs:3849`).
        let zero;
        match third {
            0 => {
                // `:50715-50745`: the swing END.
                let num = swing_width_num(0, frame_w, item_type);
                loc = [
                    pos[0] + player_w * 0.5 + (frame_w * 0.5 - num) * dir,
                    pos[1] + 24.0 + mount_offset,
                ];
                zero = [-4.0f32, 1.0f32];
            }
            1 => {
                // `:50746-50797`: the middle third.
                let num2 = swing_width_num(1, frame_w, item_type);
                let x = pos[0] + player_w * 0.5 + (frame_w * 0.5 - num2) * dir;
                let num_y = swing_height_num(1, frame_h, item_type);
                loc = [x, pos[1] + num_y + mount_offset];
                zero = [-6.0f32, -4.0f32];
            }
            _ => {
                // `:50798-50853`: the swing START — X SUBTRACTS the offset.
                let num3 = swing_width_num(2, frame_w, item_type);
                let x = pos[0] + player_w * 0.5 - (frame_w * 0.5 - num3) * dir;
                let num_y = swing_height_num(2, frame_h, item_type);
                loc = [x, pos[1] + num_y + mount_offset];
                zero = [4.0f32, -2.0f32];
            }
        }
        if uses_better_melee_item_location(item_type) {
            // `:50854-50857`: `itemLocation += zero * this.Directions`.
            loc[0] += zero[0] * dir;
            loc[1] += zero[1] * grav_dir;
        }
    }
    // `:50858` (also the claw's `:50710`): the arc rotation.
    let mut rotation = swing_rotation(item_animation, item_animation_max, direction);
    if grav_dir == -1.0 {
        // `:50860-50864`: the upside-down mirror.
        rotation = 0.0 - rotation;
        loc[1] = pos[1] + player_h + (pos[1] - loc[1]);
    }
    (loc, rotation)
}

/// `Player.ItemCheck_GetMeleeHitbox` (`Player.cs:45099-45190`), the
/// useStyle-1 path verbatim. The base rect is the held item's draw frame at
/// the truncated `itemLocation` (`:45102/:45126` — the dedServ 32x32 fallback
/// at `:45102` is overwritten by the client branch at `:45126`; the port is
/// the client), shrunk for the four DST melee frames (`:45107-45125`), scaled
/// by `GetAdjustedItemScale` (`:45128-45130`), flipped to the facing/gravity
/// side (`:45131-45138`), then grown per swing third (`:45139-45161`):
///
/// * END third (`anim < max*0.333`): X pulled back `(int)(w*1.4 - w)` when
///   facing left, `w = (int)(w*1.4)`, `y += (int)(h*0.5*gravDir)`,
///   `h = (int)(h*1.1)`.
/// * START third (`!(anim < max*0.666)`): X pulled back `(int)(w*1.2)` when
///   facing right, `w *= 2`, `y -= (int)((h*1.4 - h)*gravDir)`,
///   `h = (int)(h*1.4)`.
/// * MIDDLE third: the base rect unchanged.
///
/// The C# grows the rect in DOUBLE then truncates once — every `(int)` here
/// is that truncation, in the C#'s statement order (the END third's Y shift
/// reads the PRE-growth height, the START third's reads it too).
/// `dontAttack` is only ever set by the useStyle-3 (spear) branch
/// (`:45168-45171`) — for useStyle 1 it is always false, so this fn returns
/// the rect alone.
#[allow(clippy::too_many_arguments)]
pub(crate) fn melee_hitbox(
    item_type: i32,
    item_animation: i32,
    item_animation_max: i32,
    direction: i32,
    grav_dir: f32,
    item_location: [f32; 2],
    frame_w: i32,
    frame_h: i32,
    adjusted_scale: f32,
) -> MeleeRect {
    // `:45105-45125`: the frame shrinks (the DST crossover weapons).
    let (num, num2) = match item_type {
        5094 | 5095 => (frame_w - 10, frame_h - 10),
        5096 => (frame_w - 12, frame_h - 12),
        5097 => (frame_w - 8, frame_h - 8),
        _ => (frame_w, frame_h),
    };
    // `:45126-45130`: the truncated location + the scaled size.
    let mut rect = MeleeRect {
        x: item_location[0] as i32,
        y: item_location[1] as i32,
        width: (num as f32 * adjusted_scale) as i32,
        height: (num2 as f32 * adjusted_scale) as i32,
    };
    // `:45131-45138`: the side flips.
    if direction == -1 {
        rect.x -= rect.width;
    }
    if grav_dir == 1.0 {
        rect.y -= rect.height;
    }
    // `:45139-45161`: the useStyle-1 thirds.
    match swing_third(item_animation, item_animation_max) {
        0 => {
            if direction == -1 {
                rect.x -= (rect.width as f64 * 1.4 - rect.width as f64) as i32;
            }
            rect.width = (rect.width as f64 * 1.4) as i32;
            rect.y += (rect.height as f64 * 0.5 * grav_dir as f64) as i32;
            rect.height = (rect.height as f64 * 1.1) as i32;
        }
        2 => {
            if direction == 1 {
                rect.x -= (rect.width as f64 * 1.2) as i32;
            }
            rect.width *= 2;
            rect.y -= ((rect.height as f64 * 1.4 - rect.height as f64) * grav_dir as f64) as i32;
            rect.height = (rect.height as f64 * 1.4) as i32;
        }
        _ => {}
    }
    rect
}

/// `Utils.Vector2.Between` (`Utils.cs:1718-1725`): the inclusive box test.
fn between(p: [f32; 2], min: [f32; 2], max: [f32; 2]) -> bool {
    p[0] >= min[0] && p[0] <= max[0] && p[1] >= min[1] && p[1] <= max[1]
}

/// `Utils.ClosestPointOnLine` (`Utils.cs:1860-1875`): the clamped segment
/// projection (degenerate segments project onto A via the NaN comparisons —
/// both `< 0` and `> 1` are false on NaN, so `A + vector * NaN` returns NaN;
/// the C# has the same hole, kept verbatim).
fn closest_point_on_line(p: [f32; 2], a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    let v = [p[0] - a[0], p[1] - a[1]];
    let seg = [b[0] - a[0], b[1] - a[1]];
    let len_sq = seg[0] * seg[0] + seg[1] * seg[1];
    let t = (v[0] * seg[0] + v[1] * seg[1]) / len_sq;
    if t < 0.0 {
        return a;
    }
    if t > 1.0 {
        return b;
    }
    [a[0] + seg[0] * t, a[1] + seg[1] * t]
}

/// `Utils.LineRectangleDistance` (`Utils.cs:827-838`): 0 when either endpoint
/// is inside the corner box, else the min over the four corners of each
/// corner's distance to its closest point on the segment.
pub(crate) fn line_rectangle_distance(rect: &MeleeRect, start: [f32; 2], end: [f32; 2]) -> f32 {
    let c = rect.corners();
    // `lineStart.Between(vector, vector4)` — TopLeft..BottomRight.
    if between(start, c[0], c[3]) || between(end, c[0], c[3]) {
        return 0.0;
    }
    let dist = |p: [f32; 2]| {
        let q = closest_point_on_line(p, start, end);
        let (dx, dy) = (p[0] - q[0], p[1] - q[1]);
        (dx * dx + dy * dy).sqrt()
    };
    dist(c[0]).min(dist(c[1])).min(dist(c[2])).min(dist(c[3]))
}

/// `Player.GetPointOnSwungItemPath` (`Player.cs:44742-44752`): a point along
/// the swung blade. `num = sqrt(w² + h²)`, `num2 = (direction == 1) * PI/2`
/// (+ `PI/2 * direction` when gravDir is -1), the outward direction is
/// `itemRotation.ToRotationVector2().RotatedBy(3.926991f + num2)` and the
/// location `RotatedRelativePoint(itemLocation + outward * num * t * scale)`.
/// The port has no `fullRotation` outside sleep and no sit offset in the sim
/// (`RotatedRelativePoint`, `:4225-4251`, is the identity plus `gfxOffY` at
/// rest), so the caller passes `gfx_off_y` and the identity holds.
#[allow(clippy::too_many_arguments)]
pub(crate) fn point_on_swung_item_path(
    sprite_w: f32,
    sprite_h: f32,
    normalized_point_on_path: f32,
    item_scale: f32,
    item_rotation: f32,
    item_location: [f32; 2],
    direction: i32,
    grav_dir: f32,
    gfx_off_y: f32,
) -> ([f32; 2], [f32; 2]) {
    let num = (sprite_w * sprite_w + sprite_h * sprite_h).sqrt();
    let mut num2 = if direction == 1 {
        std::f32::consts::FRAC_PI_2
    } else {
        0.0
    };
    if grav_dir == -1.0 {
        num2 += std::f32::consts::FRAC_PI_2 * direction as f32;
    }
    // `ToRotationVector2` = (cos, sin); `RotatedBy` the XNA matrix.
    let ang = item_rotation + 3.926991 + num2;
    let outward = [ang.cos(), ang.sin()];
    let mut loc = [
        item_location[0] + outward[0] * num * normalized_point_on_path * item_scale,
        item_location[1] + outward[1] * num * normalized_point_on_path * item_scale,
    ];
    // `RotatedRelativePoint`'s `addGfxOffY` (`:4235-4238`) — identity rotation
    // at rest (`fullRotation == 0`, no mount offset, no sit/sleep offset).
    loc[1] += gfx_off_y;
    (loc, outward)
}

/// `Player.meleeNPCHitCooldown` (`Player.cs:44212-44239`) — the per-NPC i-frame
/// mirror the melee hit gate reads (`CanHitNPCWithMeleeHit`, `:44231-44234`:
/// `meleeNPCHitCooldown[npcIndex] <= 0`). `Main.maxNPCs = 200`
/// (`InitData.cs:9`).
#[derive(Clone, Debug)]
pub(crate) struct MeleeHitCooldowns {
    slots: [i32; 200],
}

impl Default for MeleeHitCooldowns {
    fn default() -> Self {
        Self { slots: [0; 200] }
    }
}

impl MeleeHitCooldowns {
    /// `ResetMeleeHitCooldowns` (`Player.cs:44212-44218`), called by
    /// `ItemCheck_StartActualUse` (`:51679`) — every new swing clears the
    /// array (the myPlayer gate is the local player, always true here).
    pub(crate) fn reset(&mut self) {
        self.slots = [0; 200];
    }

    /// `UpdateMeleeHitCooldowns` (`Player.cs:44220-44229`): every slot
    /// decrements unconditionally (negative slots keep counting down), once
    /// per `ItemCheck` melee block entry (`:43664`) — NOT once per hit.
    pub(crate) fn update(&mut self) {
        for slot in self.slots.iter_mut() {
            *slot -= 1;
        }
    }

    /// `CanHitNPCWithMeleeHit` (`Player.cs:44231-44234`).
    pub(crate) fn can_hit(&self, npc_index: usize) -> bool {
        self.slots[npc_index] <= 0
    }

    /// `SetMeleeHitCooldown` (`Player.cs:44236-44239`) — the hit site passes
    /// the CURRENT `itemAnimation` (`:44414`), so the NPC is immune for the
    /// rest of this swing.
    pub(crate) fn set(&mut self, npc_index: usize, time_in_frames: i32) {
        self.slots[npc_index] = time_in_frames;
    }

    /// `Player.ApplyAttackCooldown()` (`Player.cs:44428-44431`):
    /// `attackCD = Math.Max(1, (int)(itemAnimationMax * 0.33))` — the double
    /// product truncated once.
    pub(crate) fn attack_cooldown_frames(item_animation_max: i32) -> i32 {
        1.max((item_animation_max as f64 * 0.33) as i32)
    }
}

// ---------------------------------------------------------------------------
// Witness tests — WRITTEN NOT RUN (code-only swarm mode; no cargo invoked).
// Every expected value is derived by hand from the cited C# lines. The RNG
// witness mirrors the C# hit-path draws on `UnifiedRandom` CLONES: the module
// itself draws nothing, so a clone replay of `Next(1,101)` (crit,
// `Player.cs:44298`) then `DamageVar`'s draws proves the caller-side order is
// the only order — the hitbox/cooldown math never touches the stream.
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    /// A 24x28 Iron Broadsword frame at position (100, 200), player 20x42,
    /// facing right, normal gravity, no mount.
    fn placement(
        item_type: i32,
        anim: i32,
        max: i32,
        dir: i32,
        frame_w: f32,
        frame_h: f32,
    ) -> ([f32; 2], f32) {
        swing_item_placement(
            item_type,
            anim,
            max,
            dir,
            1.0,
            [100.0, 200.0],
            20.0,
            42.0,
            frame_w,
            frame_h,
            0.0,
        )
    }

    #[test]
    fn the_third_boundaries_compare_in_double() {
        // `Player.cs:50715/:50746`: `(double)itemAnimation <
        // (double)itemAnimationMax * 0.333`. Max 20: 20*0.333 = 6.66,
        // 20*0.666 = 13.32 — anim 6 is the END third, 7 the middle, 13 the
        // middle, 14 the start.
        assert_eq!(swing_third(6, 20), 0);
        assert_eq!(swing_third(7, 20), 1);
        assert_eq!(swing_third(13, 20), 1);
        assert_eq!(swing_third(14, 20), 2);
        // Max 35 (the Breaker Blade): 35*0.333 = 11.655, 35*0.666 = 23.31.
        assert_eq!(swing_third(11, 35), 0);
        assert_eq!(swing_third(12, 35), 1);
        assert_eq!(swing_third(23, 35), 1);
        assert_eq!(swing_third(24, 35), 2);
        // Max 8 (the Bladed Glove): 8*0.333 = 2.664, 8*0.666 = 5.328.
        assert_eq!(swing_third(2, 8), 0);
        assert_eq!(swing_third(3, 8), 1);
        assert_eq!(swing_third(5, 8), 1);
        assert_eq!(swing_third(6, 8), 2);
    }

    #[test]
    fn the_rotation_sweeps_3_5_radians() {
        // `Player.cs:50858`: (anim/max - 0.5) * -dir * 3.5 - dir * 0.3.
        // Start (20/20, dir 1): 0.5 * -3.5 - 0.3 = -2.05.
        let r = swing_rotation(20, 20, 1);
        assert!((r - -2.05).abs() < 1e-6, "{r}");
        // End (0/20, dir 1): -0.5 * -3.5 - 0.3 = 1.45.
        let r = swing_rotation(0, 20, 1);
        assert!((r - 1.45).abs() < 1e-6, "{r}");
        // Mid (10/20, dir 1): -0.3.
        let r = swing_rotation(10, 20, 1);
        assert!((r - -0.3).abs() < 1e-6, "{r}");
        // Facing left mirrors: start +2.05, end -1.45.
        assert!((swing_rotation(20, 20, -1) - 2.05).abs() < 1e-6);
        assert!((swing_rotation(0, 20, -1) - -1.45).abs() < 1e-6);
    }

    #[test]
    fn the_generic_thirds_place_the_item_per_cascade() {
        // Iron Broadsword 24x28 (`itemdata.rs` id 4 — the draw frame here is
        // the fixture's 24x28), position (100, 200), width 20, dir 1.
        // END third (`Player.cs:50715-50744`): num 10 (W <= 32), X = 100 + 10
        // + (12 - 10) = 112, Y = 224.
        let (loc, _) = placement(4, 2, 20, 1, 24.0, 28.0);
        assert!((loc[0] - 112.0).abs() < 1e-6, "{loc:?}");
        assert!((loc[1] - 224.0).abs() < 1e-6, "{loc:?}");
        // MIDDLE (`:50746-50796`): num2 10, X = 112; height 28 <= 32 → 10,
        // Y = 210.
        let (loc, _) = placement(4, 10, 20, 1, 24.0, 28.0);
        assert!((loc[0] - 112.0).abs() < 1e-6, "{loc:?}");
        assert!((loc[1] - 210.0).abs() < 1e-6, "{loc:?}");
        // START (`:50798-50852`): num3 6, X = 100 + 10 - (12 - 6) = 104,
        // Y = 210.
        let (loc, _) = placement(4, 18, 20, 1, 24.0, 28.0);
        assert!((loc[0] - 104.0).abs() < 1e-6, "{loc:?}");
        assert!((loc[1] - 210.0).abs() < 1e-6, "{loc:?}");
        // Facing left mirrors X around position.X + width*0.5 = 110:
        // end 108, start 116.
        let (loc, _) = placement(4, 2, 20, -1, 24.0, 28.0);
        assert!((loc[0] - 108.0).abs() < 1e-6, "{loc:?}");
        let (loc, _) = placement(4, 18, 20, -1, 24.0, 28.0);
        assert!((loc[0] - 116.0).abs() < 1e-6, "{loc:?}");
    }

    #[test]
    fn the_width_cascades_step_at_the_csharp_thresholds() {
        // END third (`Player.cs:50717-50733`): 32 → 10, 33 → 14, 52 → 24,
        // 64 → 28, 92 → 38. X = 110 + (W/2 - num).
        for (w, num) in [(32.0, 10.0), (33.0, 14.0), (52.0, 24.0), (64.0, 28.0), (92.0, 38.0)] {
            let (loc, _) = placement(4, 2, 20, 1, w, 28.0);
            let expect = 110.0 + (w * 0.5 - num);
            assert!((loc[0] - expect).abs() < 1e-6, "W {w}: {loc:?}");
        }
        // MIDDLE (`:50748-50764`): 33 → 18 (not 14).
        let (loc, _) = placement(4, 10, 20, 1, 33.0, 28.0);
        assert!((loc[0] - (110.0 + 16.5 - 18.0)).abs() < 1e-6, "{loc:?}");
        // START (`:50800-50820`): the extra >=48 step — 48 → 18. X = 110 -
        // (W/2 - num).
        let (loc, _) = placement(4, 18, 20, 1, 48.0, 28.0);
        assert!((loc[0] - (110.0 - (24.0 - 18.0))).abs() < 1e-6, "{loc:?}");
        // START 33 → 14 (no >=48 yet).
        let (loc, _) = placement(4, 18, 20, 1, 33.0, 28.0);
        assert!((loc[0] - (110.0 - (16.5 - 14.0))).abs() < 1e-6, "{loc:?}");
        // The height cascades: MIDDLE (`:50774-50786`) H 33 → 8, 53 → 12,
        // 65 → 14; Y = 200 + num.
        for (h, num) in [(32.0, 10.0), (33.0, 8.0), (53.0, 12.0), (65.0, 14.0)] {
            let (loc, _) = placement(4, 10, 20, 1, 24.0, h);
            assert!((loc[1] - (200.0 + num)).abs() < 1e-6, "H {h}: {loc:?}");
        }
    }

    #[test]
    fn the_per_item_nudges_join_the_cascades() {
        // The fishing hammers (`Player.cs:50734-50737/:50765-50768/:50821-
        // 50824`): +8 end / +4 middle / +4 start on the width num; the
        // Keybrand (671, `:50738-50741` etc.): +12 / +6 / +8.
        // Reaver Shark 2341, 24x28: end num 10+8=18 → X = 110 + (12-18) = 104.
        let (loc, _) = placement(2341, 2, 20, 1, 24.0, 28.0);
        assert!((loc[0] - 104.0).abs() < 1e-6, "{loc:?}");
        // Keybrand 671, 40x40 frame: end num 14+12=26 → X = 110 + (20-26) =
        // 104; middle num 18+6=24 → X = 106, height num 8+10=18 → Y = 218;
        // start num 14+8=22 → X = 110 - (20-22) = 112, height 10+8=18 →
        // Y = 218.
        let (loc, _) = placement(671, 2, 20, 1, 40.0, 40.0);
        assert!((loc[0] - 104.0).abs() < 1e-6, "{loc:?}");
        let (loc, _) = placement(671, 10, 20, 1, 40.0, 40.0);
        assert!((loc[0] - 106.0).abs() < 1e-6, "{loc:?}");
        assert!((loc[1] - 218.0).abs() < 1e-6, "{loc:?}");
        let (loc, _) = placement(671, 18, 20, 1, 40.0, 40.0);
        assert!((loc[0] - 112.0).abs() < 1e-6, "{loc:?}");
        assert!((loc[1] - 218.0).abs() < 1e-6, "{loc:?}");
    }

    #[test]
    fn the_breaker_blade_rides_the_better_location_nudge() {
        // `ItemID.cs:348`: only 426 is in `UsesBetterMeleeItemLocation`.
        // `Player.cs:50854-50857`: `itemLocation += zero * Directions`.
        // 60x70 frame: END num 24 (>=52, not >=64), X = 110 + (30-24) = 116,
        // Y = 224; zero (-4, 1) → (112, 225).
        let (loc, _) = placement(426, 3, 35, 1, 60.0, 70.0);
        assert!((loc[0] - 112.0).abs() < 1e-6, "{loc:?}");
        assert!((loc[1] - 225.0).abs() < 1e-6, "{loc:?}");
        // MIDDLE: num2 24, X = 116; height 70 > 64 → 14, Y = 214; zero
        // (-6, -4) → (110, 210).
        let (loc, _) = placement(426, 17, 35, 1, 60.0, 70.0);
        assert!((loc[0] - 110.0).abs() < 1e-6, "{loc:?}");
        assert!((loc[1] - 210.0).abs() < 1e-6, "{loc:?}");
        // START: num3 24, X = 110 - (30-24) = 104; height 14, Y = 214; zero
        // (4, -2) → (108, 212).
        let (loc, _) = placement(426, 30, 35, 1, 60.0, 70.0);
        assert!((loc[0] - 108.0).abs() < 1e-6, "{loc:?}");
        assert!((loc[1] - 212.0).abs() < 1e-6, "{loc:?}");
        // Facing left: the nudge's X flips with Directions.X — END X = 110 -
        // 6 = 104, nudge -4 * -1 = +4 → 108; Y unchanged 225.
        let (loc, _) = placement(426, 3, 35, -1, 60.0, 70.0);
        assert!((loc[0] - 108.0).abs() < 1e-6, "{loc:?}");
        assert!((loc[1] - 225.0).abs() < 1e-6, "{loc:?}");
        // A non-set item (the Keybrand) takes NO nudge — its end X stays 104
        // (the previous test's value).
        assert!(!uses_better_melee_item_location(671));
    }

    #[test]
    fn the_bladed_glove_takes_the_claw_arm() {
        // `Main.cs:6793-6795`: 1827 alone is `Item.claw`.
        // `Player.cs:50695-50710`: END X = 110 + (12-10) = 112, Y = 226;
        // MIDDLE X = 110 + (12-8) = 114, Y = 224; START X = 110 - (12-6) =
        // 104, Y = 220. The rotation is the same arc (`:50710`).
        assert!(is_claw(1827));
        let (loc, rot) = placement(1827, 2, 8, 1, 24.0, 28.0);
        assert!((loc[0] - 112.0).abs() < 1e-6, "{loc:?}");
        assert!((loc[1] - 226.0).abs() < 1e-6, "{loc:?}");
        assert!((rot - swing_rotation(2, 8, 1)).abs() < 1e-6);
        let (loc, _) = placement(1827, 4, 8, 1, 24.0, 28.0);
        assert!((loc[0] - 114.0).abs() < 1e-6, "{loc:?}");
        assert!((loc[1] - 224.0).abs() < 1e-6, "{loc:?}");
        let (loc, _) = placement(1827, 7, 8, 1, 24.0, 28.0);
        assert!((loc[0] - 104.0).abs() < 1e-6, "{loc:?}");
        assert!((loc[1] - 220.0).abs() < 1e-6, "{loc:?}");
        // The claw arm ignores the BetterMeleeItemLocation nudge — only the
        // generic else branch applies it (`:50854` sits inside the else).
        assert!(!uses_better_melee_item_location(1827));
    }

    #[test]
    fn the_gravity_mirror_flips_both_outputs() {
        // `Player.cs:50860-50864`: rotation negated, Y mirrored around
        // position.Y + height: Y' = 200 + 42 + (200 - Y).
        let (loc, rot) = swing_item_placement(
            4, 18, 20, 1, -1.0, [100.0, 200.0], 20.0, 42.0, 24.0, 28.0, 0.0,
        );
        // The start third placed Y 210 → mirrored 232; X unchanged 104.
        assert!((loc[0] - 104.0).abs() < 1e-6, "{loc:?}");
        assert!((loc[1] - 232.0).abs() < 1e-6, "{loc:?}");
        assert!((rot - -swing_rotation(18, 20, 1)).abs() < 1e-6, "{rot}");
        // The Breaker Blade's nudge Y flips with Directions.Y BEFORE the
        // mirror: start Y 214 + (-2 * -1) = 216 → mirrored 226.
        let (loc, _) = swing_item_placement(
            426, 30, 35, 1, -1.0, [100.0, 200.0], 20.0, 42.0, 60.0, 70.0, 0.0,
        );
        assert!((loc[1] - 226.0).abs() < 1e-6, "{loc:?}");
    }

    #[test]
    fn the_mount_offset_rides_every_y_row() {
        // `mountOffset` joins the Y assignments (`Player.cs:50743/:50795/
        // :50851`); the mirror at `:50863` has no mount term (verbatim).
        let (loc, _) = swing_item_placement(
            4, 2, 20, 1, 1.0, [100.0, 200.0], 20.0, 42.0, 24.0, 28.0, 6.0,
        );
        assert!((loc[1] - 230.0).abs() < 1e-6, "{loc:?}");
    }

    #[test]
    fn the_hitbox_base_rect_truncates_and_flips() {
        // `Player.cs:45126-45138`: location truncated, size scaled, X flipped
        // by width when facing left, Y flipped by height when gravDir 1.
        // MIDDLE third (no growth): 24x28 at (112.7, 210.2), scale 1 →
        // (112, 210-28, 24, 28) = (112, 182, 24, 28) facing right.
        let r = melee_hitbox(4, 10, 20, 1, 1.0, [112.7, 210.2], 24, 28, 1.0);
        assert_eq!(r, MeleeRect { x: 112, y: 182, width: 24, height: 28 });
        // Facing left: X = 112 - 24 = 88.
        let r = melee_hitbox(4, 10, 20, -1, 1.0, [112.7, 210.2], 24, 28, 1.0);
        assert_eq!(r.x, 88);
        // gravDir -1: no Y flip (`:45135` only fires at 1).
        let r = melee_hitbox(4, 10, 20, 1, -1.0, [112.7, 210.2], 24, 28, 1.0);
        assert_eq!(r.y, 210);
        // The scale truncates the product: 24 * 1.15 = 27.6 → 27
        // (the Reaver Shark's scale, `:45129`).
        let r = melee_hitbox(2341, 10, 22, 1, 1.0, [112.0, 210.0], 24, 28, 1.15);
        assert_eq!((r.width, r.height), (27, 32));
    }

    #[test]
    fn the_dst_frames_shrink_before_the_scale() {
        // `Player.cs:45107-45125`: 5094/5095 -10, 5096 -12, 5097 -8 — applied
        // to the frame BEFORE the scale multiply (`:45129`). Tentacle Spike
        // 5094: frame 24x28 → 14x18, scale 1.2 → (16.8, 21.6) → (16, 21).
        let r = melee_hitbox(5094, 10, 21, 1, 1.0, [112.0, 210.0], 24, 28, 1.2);
        assert_eq!((r.width, r.height), (16, 21));
        // Ham Bat 5096: 12x16 * 1.2 = (14.4, 19.2) → (14, 19).
        let r = melee_hitbox(5096, 10, 20, 1, 1.0, [112.0, 210.0], 24, 28, 1.2);
        assert_eq!((r.width, r.height), (14, 19));
        // Bat Bat 5097: 16x20 * 1.15 = (18.4, 23.0) → (18, 23).
        let r = melee_hitbox(5097, 20, 40, 1, 1.0, [112.0, 210.0], 24, 28, 1.15);
        assert_eq!((r.width, r.height), (18, 23));
    }

    #[test]
    fn the_end_third_grows_the_hitbox_down_and_wide() {
        // `Player.cs:45141-45149`. Base 24x28 at loc (112, 224) facing right:
        // rect (112, 196, 24, 28). No X pull (dir 1); w = (int)(24*1.4) = 33;
        // y += (int)(28*0.5*1) = 14 → 210; h = (int)(28*1.1) = 30.
        let r = melee_hitbox(4, 2, 20, 1, 1.0, [112.0, 224.0], 24, 28, 1.0);
        assert_eq!(r, MeleeRect { x: 112, y: 210, width: 33, height: 30 });
        // Facing left: base X = 112-24 = 88; pull (int)(24*1.4-24) = 9 →
        // X 79; same growth.
        let r = melee_hitbox(4, 2, 20, -1, 1.0, [112.0, 224.0], 24, 28, 1.0);
        assert_eq!(r, MeleeRect { x: 79, y: 210, width: 33, height: 30 });
        // The Y shift reads the PRE-growth height (28, not 30) — and scales
        // with gravDir: -1 → y -= 14.
        let r = melee_hitbox(4, 2, 20, 1, -1.0, [112.0, 224.0], 24, 28, 1.0);
        assert_eq!(r.y, 224 - 14);
        assert_eq!(r.height, 30);
    }

    #[test]
    fn the_start_third_doubles_the_width_and_lifts_the_box() {
        // `Player.cs:45151-45159`. Base 24x28 at loc (104, 210) facing right:
        // rect (104, 182, 24, 28). X -= (int)(24*1.2) = 28 → 76; w *= 2 → 48;
        // y -= (int)((28*1.4-28)*1) = (int)11.2 = 11 → 171; h = (int)(28*1.4)
        // = 39.
        let r = melee_hitbox(4, 18, 20, 1, 1.0, [104.0, 210.0], 24, 28, 1.0);
        assert_eq!(r, MeleeRect { x: 76, y: 171, width: 48, height: 39 });
        // Facing left: NO X pull (the `direction == 1` gate), base X = 80.
        let r = melee_hitbox(4, 18, 20, -1, 1.0, [104.0, 210.0], 24, 28, 1.0);
        assert_eq!(r.x, 80);
        assert_eq!(r.width, 48);
        // gravDir -1: the Y term flips sign → y += 11.
        let r = melee_hitbox(4, 18, 20, 1, -1.0, [104.0, 210.0], 24, 28, 1.0);
        assert_eq!(r.y, 210 + 11);
    }

    #[test]
    fn the_middle_third_leaves_the_base_rect() {
        // `Player.cs:45139-45161`: neither branch fires between the
        // thresholds — the rect is the base.
        let r = melee_hitbox(4, 7, 20, 1, 1.0, [112.0, 210.0], 24, 28, 1.0);
        assert_eq!(r, MeleeRect { x: 112, y: 182, width: 24, height: 28 });
        let r = melee_hitbox(4, 13, 20, 1, 1.0, [112.0, 210.0], 24, 28, 1.0);
        assert_eq!(r, MeleeRect { x: 112, y: 182, width: 24, height: 28 });
    }

    #[test]
    fn the_full_swing_hitbox_follows_the_placement() {
        // The composition the ItemCheck runs (`Player.cs:42846` →
        // `:43380` → `:43650`): placement first, hitbox from its output.
        // Iron Broadsword, 20-tick swing, dir 1: the START rect reaches
        // BACK over the player (X 76 < player X 100), the END rect reaches
        // FORWARD (X 112..145).
        let (loc, _) = placement(4, 18, 20, 1, 24.0, 28.0);
        let start = melee_hitbox(4, 18, 20, 1, 1.0, loc, 24, 28, 1.0);
        assert!(start.x < 100, "the wind-up covers the body: {start:?}");
        let (loc, _) = placement(4, 2, 20, 1, 24.0, 28.0);
        let end = melee_hitbox(4, 2, 20, 1, 1.0, loc, 24, 28, 1.0);
        assert!(end.x + end.width > 130, "the follow-through reaches: {end:?}");
        // A slime standing at (140, 190) 24x24 is inside the END box
        // (112..145 x 210..240)? Y 190+24 = 214 > 210 — intersects.
        let slime = MeleeRect { x: 140, y: 190, width: 24, height: 24 };
        assert!(end.intersects(&slime));
        // ...and outside the START box (76..124): X 140 >= 124 — no.
        assert!(!start.intersects(&slime));
    }

    #[test]
    fn intersects_is_strict_on_touching_edges() {
        // XNA `Rectangle.Intersects`: `<` on all four sides.
        let a = MeleeRect { x: 0, y: 0, width: 10, height: 10 };
        let touch = MeleeRect { x: 10, y: 0, width: 10, height: 10 };
        assert!(!a.intersects(&touch));
        let overlap = MeleeRect { x: 9, y: 9, width: 10, height: 10 };
        assert!(a.intersects(&overlap));
    }

    #[test]
    fn the_cooldowns_tick_reset_and_gate() {
        // `Player.cs:44212-44239` + the hit site `:44414`.
        let mut cd = MeleeHitCooldowns::default();
        assert!(cd.can_hit(3));
        cd.set(3, 12); // SetMeleeHitCooldown(3, itemAnimation)
        assert!(!cd.can_hit(3));
        assert!(cd.can_hit(4));
        // UpdateMeleeHitCooldowns decrements EVERY slot (`:44226`), including
        // the untouched ones (they go negative — `can_hit` stays true).
        for _ in 0..12 {
            cd.update();
        }
        assert!(cd.can_hit(3), "12 ticks drain the 12-frame immunity");
        cd.set(3, 5);
        cd.update();
        assert!(!cd.can_hit(3));
        // ResetMeleeHitCooldowns (`:44216`, called at StartActualUse
        // `:51679`) clears the array mid-swing.
        cd.reset();
        assert!(cd.can_hit(3));
    }

    #[test]
    fn the_attack_cooldown_is_a_third_of_the_animation() {
        // `Player.cs:44430`: Math.Max(1, (int)(itemAnimationMax * 0.33)).
        assert_eq!(MeleeHitCooldowns::attack_cooldown_frames(20), 6);
        assert_eq!(MeleeHitCooldowns::attack_cooldown_frames(35), 11);
        assert_eq!(MeleeHitCooldowns::attack_cooldown_frames(8), 2);
        // The floor: max 3 → (int)0.99 = 0 → 1.
        assert_eq!(MeleeHitCooldowns::attack_cooldown_frames(3), 1);
        assert_eq!(MeleeHitCooldowns::attack_cooldown_frames(0), 1);
    }

    #[test]
    fn the_adjusted_scale_only_gloves_melee_items() {
        // `Player.cs:42566-42582`.
        assert!((adjusted_item_scale(1.15, true, false) - 1.15).abs() < 1e-6);
        assert!((adjusted_item_scale(1.0, true, true) - 1.1).abs() < 1e-6);
        assert!((adjusted_item_scale(1.0, false, true) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn the_melee_speed_shortens_the_animation() {
        // `Player.cs:29028-29039` (cap 3, invert) + `:4453-4461` (the
        // truncated product, floored at 1) + `:4470-4473` (the melee arm).
        // No bonus: 20 ticks stay 20.
        assert_eq!(melee_animation_frames(4, 20, 1.0), 20);
        // +20% melee speed: 1/1.2 = 0.8333, 20 * 0.8333 = 16.67 → 16.
        assert_eq!(melee_animation_frames(4, 20, 1.2), 16);
        // The cap: accumulated 4.0 → speed caps at 3 → 1/3, 20/3 = 6.67 → 6.
        assert_eq!(melee_animation_frames(4, 20, 4.0), 6);
        // The floor: a 1-tick base at any speed stays >= 1.
        assert_eq!(melee_animation_frames(4, 1, 3.0), 1);
        // The Terragrim (3821) is NoMeleeSpeedBonus (`ItemID.cs:340`) — the
        // CALLER skips the melee arm for it (`:4470`); the set predicate:
        assert!(no_melee_speed_bonus(3821));
        assert!(!no_melee_speed_bonus(4));
        // The Bladed Glove's half-bonus fold (`:25912`, `ItemID.cs:1350`):
        // accumulated 1.2 → 1 + 0.2*0.5 = 1.1 → 1/1.1, 8 * 0.909 = 7.27 → 7.
        assert_eq!(melee_animation_frames(1827, 8, 1.2), 7);
        // The Fetid Baghnakhs' quarter fold: 1 + 0.2*0.25 = 1.05 → 8/1.05 =
        // 7.62 → 7.
        assert_eq!(melee_animation_frames(3013, 8, 1.2), 7);
        // The Psycho Knife's 0.33 fold: 1 + 0.2*0.33 = 1.066 → the f32
        // product 20 * (1/1.066) lands a hair under 18.7617 → 18.
        assert_eq!(melee_animation_frames(3106, 20, 1.2), 18);
        // The zero-speed hole is verbatim (`:29034`): speed 0 stays 0 →
        // the product truncates to 0 → the floor lifts it to 1.
        assert_eq!(melee_animation_frames(4, 20, 0.0), 1);
    }

    #[test]
    fn the_swung_path_points_ride_the_arc() {
        // `Player.cs:44742-44752`, the Fiery Greatsword's hit probe
        // (`:44282-44283`): 70x70 sprite, num = sqrt(9800) ≈ 98.99495.
        // At t = 0 the location is itemLocation (+gfxOffY); at t = 0.9 it
        // sits 0.9*num*scale along the outward direction.
        let rot = swing_rotation(10, 20, 1); // -0.3 mid-swing
        let (p0, out0) =
            point_on_swung_item_path(70.0, 70.0, 0.0, 1.0, rot, [112.0, 210.0], 1, 1.0, 0.0);
        assert!((p0[0] - 112.0).abs() < 1e-4 && (p0[1] - 210.0).abs() < 1e-4);
        let (p1, out1) =
            point_on_swung_item_path(70.0, 70.0, 0.9, 1.0, rot, [112.0, 210.0], 1, 1.0, 0.0);
        // outward = (rot + 3.926991 + PI/2).ToRotationVector2().
        let ang = rot + 3.926991 + std::f32::consts::FRAC_PI_2;
        assert!((out1[0] - ang.cos()).abs() < 1e-6);
        assert!((out0[0] - out1[0]).abs() < 1e-6);
        let num = (70.0f32 * 70.0 + 70.0 * 70.0).sqrt();
        assert!((p1[0] - (112.0 + out1[0] * num * 0.9)).abs() < 1e-3);
        assert!((p1[1] - (210.0 + out1[1] * num * 0.9)).abs() < 1e-3);
        // Facing left drops the PI/2 term (`:44745` — (direction == 1) only).
        let (_, outl) =
            point_on_swung_item_path(70.0, 70.0, 0.9, 1.0, rot, [112.0, 210.0], -1, 1.0, 0.0);
        let angl = rot + 3.926991;
        assert!((outl[0] - angl.cos()).abs() < 1e-6);
        // gfxOffY joins the location, not the direction (`:44751` →
        // RotatedRelativePoint `:4237`).
        let (pg, _) =
            point_on_swung_item_path(70.0, 70.0, 0.0, 1.0, rot, [112.0, 210.0], 1, 1.0, 3.0);
        assert!((pg[1] - 213.0).abs() < 1e-4);
    }

    #[test]
    fn the_line_rectangle_distance_matches_the_utils_geometry() {
        // `Utils.cs:827-838`. A 10x10 box at (0,0); a horizontal segment
        // y = 20 from x -10 to 20 passes 10 below the bottom corners →
        // distance 10 (the bottom corners project inside the segment).
        let r = MeleeRect { x: 0, y: 0, width: 10, height: 10 };
        let d = line_rectangle_distance(&r, [-10.0, 20.0], [20.0, 20.0]);
        assert!((d - 10.0).abs() < 1e-4, "{d}");
        // An endpoint inside the corner box → 0 (`:833-836`).
        let d = line_rectangle_distance(&r, [5.0, 5.0], [50.0, 50.0]);
        assert_eq!(d, 0.0);
        // A segment fully right of the box: the nearest corners are the
        // right pair at (10, 0)/(10, 10); the segment x = 30, y -10..20 →
        // distance 20.
        let d = line_rectangle_distance(&r, [30.0, -10.0], [30.0, 20.0]);
        assert!((d - 20.0).abs() < 1e-4, "{d}");
        // The Fiery Greatsword gate (`Player.cs:44284`): distance <= 16
        // counts as a hit even without rect intersection.
        assert!(line_rectangle_distance(&r, [26.0, -10.0], [26.0, 20.0]) <= 16.0);
        assert!(line_rectangle_distance(&r, [27.0, -10.0], [27.0, 20.0]) > 16.0);
    }

    /// The RNG-order witness: the module draws NOTHING, so the C# hit path's
    /// stream (`Player.cs:44298` crit `Next(1, 101)`, then `:44342`
    /// `Main.DamageVar`) is replayed on two CLONES of one seeded
    /// `UnifiedRandom` — one through the port's `crit_roll`/`damage_var`
    /// (the editing.rs call order), one raw. Equal end states prove the
    /// hitbox/cooldown math between the draws consumes no randomness and
    /// the caller order is preserved.
    #[test]
    fn the_hit_path_draws_ride_the_caller_stream_in_csharp_order() {
        use tr_core::loot::DropRng;
        let base = tr_core::UnifiedRandom::new(12345);
        // Lane A: the port's helpers in the editing.rs order (crit first,
        // DamageVar second — `world_sim/editing.rs:1232-1233`).
        let mut a = base.clone();
        let crit_a = tr_core::damage::crit_roll(&mut a, 4);
        let dmg_a = tr_core::damage::damage_var(&mut a, 12.0, 0.0);
        // Lane B: the raw C# draws. Crit: `Main.rand.Next(1, 101) <= crit`
        // (`Player.cs:44298`). Then `Main.DamageVar(dmg, 0f)`
        // (`Main.cs:66118-66134`): ONE `Next(-15, 16)` percent draw (luck 0
        // takes neither reroll arm), `dmg * (1 + pct * 0.01)` rounded
        // banker's (`(int)Math.Round`).
        let mut b = base.clone();
        let roll_b = b.next_range(1, 101);
        let crit_b = roll_b <= 4;
        let pct_b = b.next_range(-15, 16);
        let num_b = 12.0f32 * (1.0 + pct_b as f32 * 0.01);
        let dmg_b = tr_core::damage::round_bankers(num_b as f64) as i32;
        assert_eq!(crit_a, crit_b);
        assert_eq!(dmg_a, dmg_b);
        assert_eq!(a, b, "the two lanes drained the clone identically");
        // The untouched original proves the clones did not touch the live
        // stream.
        assert_eq!(base, tr_core::UnifiedRandom::new(12345));
        // And the swing math itself is stream-free: computing a full swing
        // between the draws changes nothing.
        let mut c = base.clone();
        let _ = tr_core::damage::crit_roll(&mut c, 4);
        for anim in (0..20).rev() {
            let (loc, _) = placement(4, anim, 20, 1, 24.0, 28.0);
            let _ = melee_hitbox(4, anim, 20, 1, 1.0, loc, 24, 28, 1.0);
        }
        let mut cd = MeleeHitCooldowns::default();
        cd.update();
        cd.set(0, 19);
        let _ = tr_core::damage::damage_var(&mut c, 12.0, 0.0);
        assert_eq!(c, a, "the swing math consumed no draws");
    }
}
