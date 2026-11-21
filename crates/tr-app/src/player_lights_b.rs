//! The Player `Lighting.AddLight` GAP batch (`S1-E-5`, W4-e7 census close).
//!
//! `player_lights.rs` landed 32 of Player.cs's 46 `AddLight` sites and listed
//! 14 as dropped ("each needs a mount/armor/stealth system the port does not
//! carry yet"). This module re-censuses those 14 against the CURRENT port and
//! ports the **10 whose dependencies have since landed** (the mount roster,
//! the armor-slot registry resolution, `melee_swing.rs`'s hitbox/placement,
//! the wing-slot table and `Item.GetPhaseColor`'s inputs). The remaining **4**
//! stay dropped (see "what stays dropped" below). Same shape as
//! `npc_lights_b.rs`: pure fns returning `(tile_x, tile_y, rgb)` triples that
//! the draw feed appends to the ONE [`crate::entity_lights::EntityLights`]
//! pair (`LightingEngine.cs:36-38`, one list pair per engine, max-merged
//! `:208-235` one frame late). No arm draws `Main.rand` — every rand-bearing
//! dust branch is skipped wholesale, so the UpdatePlayers stream is untouched.
//!
//! Ported here (10 sites):
//! * **Rudolph's three-tile trail** (`Player.Update`, `Player.cs:24491-24498`):
//!   `mount.Active && mount.Type == 0`, three lights at `num`, `num+direction`,
//!   `num+direction*2`, all (0.5, 0.2, 0.05). `num`/`j` use the **int** division
//!   inside the cast (`(int)(position.X + (float)(width / 2)) / 16`), the live
//!   `height` for the y half (mounted Rudolph is 42+20 = 62 tall).
//! * **The three armor-set lights** (`UpdateArmorLights`, `:31149-31317`): the
//!   head/body/legs slot colour triplets, the +0.5 per matching-set bonus, the
//!   `spinningpoint` anchors rotated by `fullRotation` about
//!   `fullRotationOrigin`. TWO C# overwrite quirks are kept verbatim: body case
//!   190 (`:31226-31230`) and legs case 130 (`:31257-31261`) write the HEAD
//!   triplet (`num`/`num2`/`num3`), not their own.
//! * **The Chillet dash light** (`DashMovement`, `:21344-21354`): the dash-6
//!   active tick's centre light, (0.6, 0.3, 0.1) for the Ignis pair (63/65)
//!   else (0.5, 0.1, 0.6). The 8-dust loop after it draws `Main.rand` — skipped
//!   wholesale; the `AddLight` line itself is RNG-free.
//! * **The Kazzymodus wing light** (`WingFrame`, `:29543-29549`): wings 28,
//!   airborne, `base.Bottom` (0.3, 0.1, 0.4).
//! * **The phase-blade held light** (`ItemCheck_EmitUseVisuals`, `:44913-44917`):
//!   `Item.GetPhaseColor(shoot) * 0.5f` at the swing `itemLocation + (6, -14)`.
//! * **The Chillet melee-swing light** (`ItemCheck_EmitUseVisuals` tail,
//!   `:45028-45032`): the mount 62-65 melee swing's `itemRectangle.Center`,
//!   the dash colour × 0.5 (Vector3 float math, NOT byte).
//!
//! **What stays dropped (the other 4 of the 14):** the wing-27 Shroomite light
//! (`:36026`, `0.4f * stealth` — the port's `stealth` has no producer, it is
//! only ever set in a knockback witness) and the Snapping Stone trio
//! (`:24575-24583`, gated on `accSnappingStoneLightUp` = `cooldown == 0` with
//! `accSnappingStone`/`accPyroclast`/`accArmletOfRuin` — none of those flags,
//! the 360-tick cooldown setter `Projectile.cs:14316`, nor
//! `TryMakingSnappingStoneStrike` is ported). The light pets are Projectile.AI
//! sites (projectile family); NightOwl sets `nightVision` only.
//!
//! **Deliberate folds (recorded, ANALYSIS "The Player lights gap batch"):**
//! * `vortexStealthActive` (`:31151`) has no port field — folds false, so the
//!   armor lights are never suppressed (the Vortex set's stealth toggle is
//!   unported). `isDisplayDollOrInanimate` folds false (no display dolls).
//! * `wings`/`wingsLogic` are the SAME port field (`player.rs:5156-5161`), so
//!   `WingFrame`'s `flag = wingsLogic != wings` (`:29470`) folds false and the
//!   wing-28 `!flag` gate is always open; the merman `wings = 0` clear
//!   (`:25581`) is an unported swim-visual sibling.
//! * `noMelee`/`noUseGraphic` have no `ItemDef` fields — the phase-blade gate
//!   reduces to the 18-kind list (all 18 are `noMelee = noUseGraphic = false`,
//!   verified per-case in `Item.cs`) and the Chillet swing gate reads the
//!   `melee` class flag. The swing lights read the useStyle-1 hitbox
//!   (`melee_swing.rs`); the useStyle-3 spear rect is that module's documented
//!   out-of-scope branch. The swing `itemLocation`'s `mountOffset` IS carried
//!   (`HeightOffsetHitboxCenter` = `mount.PlayerOffsetHitbox`, `:42846` →
//!   `:3253-3267`) — the Chillet swing light runs mounted; the separate
//!   `ApplyItemPositionOffsetFromMount` `+ pos` of the held-light family is a
//!   different offset (not present on these two sites). `gravDir` is 1.
//! * `GetPhaseColor`'s float `Color` ctor clamps then byte-casts
//!   (`render/pylons.rs::color_bytes` precedent, `Color.cs`); `* 0.5f` is the
//!   truncating byte `Color operator*`; the rainbow 1101/1102 arm rides
//!   `Utils.MultiLerp` (`Utils.cs:292-303`) over the 9-entry palette at
//!   `miscCounterNormalized`. The port's `misc_counter` wraps without the C#
//!   300 reset, so `% 300` recovers the C# value (both tick once per update).

use glam::Vec2;
use tr_core::TILE_PX;

use crate::player::{Player, WIDTH};

/// The `AddLight(Vector2, …)` overload's tile form (`Lighting.cs:135-143`):
/// `(int)(v / 16f)` per axis — truncate the QUOTIENT, not the pixel.
fn quot_tile(v: Vec2) -> (i32, i32) {
    ((v.x / TILE_PX) as i32, (v.y / TILE_PX) as i32)
}

/// Rudolph's three-tile trail (`Player.Update`, `Player.cs:24491-24498`).
/// `mount.Active && mount.Type == 0`; the int-division centre anchor, the
/// live mounted height, three lights stepping by `direction`.
pub(crate) fn rudolph_trail(p: &Player) -> Vec<(i32, i32, [f32; 3])> {
    if !p.mount.is_active() || p.mount.kind() != 0 {
        return Vec::new();
    }
    // `:24493-24494`: `(int)(position.X + (float)(width / 2)) / 16` — the
    // width/2 is INT division inside the cast; the y half reads the live
    // height (Rudolph grows the hitbox to 42 + heightBoost 20 = 62).
    let num = (p.x + (p.width as i32 / 2) as f32) as i32 / 16;
    let j = (p.y + (p.height as i32 / 2) as f32 - 14.0) as i32 / 16;
    let rgb = [0.5, 0.2, 0.05];
    vec![
        (num, j, rgb),
        (num + p.facing, j, rgb),
        (num + p.facing * 2, j, rgb),
    ]
}

/// `Utils.RotatedBy(spinningpoint, radians, center)` (`Utils.cs:1675-1684`):
/// the float cos/sin of the double angle, centre-relative. Both the point and
/// the centre are position-relative here (the C# `fullRotationOrigin` is the
/// hitbox centre offset, `PlayerSleepingHelper.cs:74`).
fn rotate_about(v: Vec2, radians: f32, center: Vec2) -> Vec2 {
    let num = (radians as f64).cos() as f32;
    let num2 = (radians as f64).sin() as f32;
    let d = v - center;
    center + Vec2::new(d.x * num - d.y * num2, d.x * num2 + d.y * num)
}

/// The three armor-set lights (`UpdateArmorLights`, `Player.cs:31149-31317`).
/// `head`/`body`/`legs` are the functional armor slots' registry ids
/// (`:25621-25623` assign them from `GetEffectiveArmor(0/1/2)` BEFORE this
/// runs at `:25684`; the vanity override is `PlayerFrame`'s, later). The two
/// overwrite quirks (body 190 / legs 130 write the HEAD triplet) are kept.
pub(crate) fn armor_lights(p: &Player) -> Vec<(i32, i32, [f32; 3])> {
    // `:31151-31154`: `if (vortexStealthActive) return;` — folds false.
    let slot = |i: usize, f: fn(&crate::registry::ItemDef) -> i16| {
        crate::registry::get(p.equipped[i]).map_or(-1, f)
    };
    let head = slot(0, |d| d.head_slot);
    let body = slot(1, |d| d.body_slot);
    let legs = slot(2, |d| d.leg_slot);
    // The HEAD triplet (`:31155-31205`).
    let (mut num, mut num2, mut num3) = match head {
        11 => (0.92, 0.8, 0.65),
        216 => (0.7, 0.95, 0.82),
        285 => (1.0, 0.88, 0.73),
        169 => (0.0, 0.36, 0.4),
        170 => (0.4, 0.16, 0.36),
        171 => (0.5, 0.25, 0.05),
        189 => (0.9, 0.9, 0.7),
        178 => (0.1, 0.2, 0.3),
        211 => (0.2, 0.4, 0.8),
        _ => (0.0, 0.0, 0.0),
    };
    // The BODY triplet (`:31206-31236`) — case 190 writes the HEAD triplet.
    let (mut num4, mut num5, mut num6) = (0.0f32, 0.0f32, 0.0f32);
    match body {
        175 => (num4, num5, num6) = (0.0, 0.36, 0.4),
        176 => (num4, num5, num6) = (0.4, 0.16, 0.36),
        177 => (num4, num5, num6) = (0.5, 0.25, 0.05),
        190 => (num, num2, num3) = (0.9, 0.9, 0.7), // `:31226-31230` quirk
        205 => (num4, num5, num6) = (0.2, 0.4, 0.8),
        _ => {}
    }
    // The LEGS triplet (`:31237-31262`) — case 130 writes the HEAD triplet.
    let (mut num7, mut num8, mut num9) = (0.0f32, 0.0f32, 0.0f32);
    match legs {
        110 => (num7, num8, num9) = (0.0, 0.36, 0.4),
        111 => (num7, num8, num9) = (0.4, 0.16, 0.36),
        112 => (num7, num8, num9) = (0.5, 0.25, 0.05),
        130 => (num, num2, num3) = (0.9, 0.9, 0.7), // `:31257-31261` quirk
        _ => {}
    }
    let mut out = Vec::new();
    let pos = Vec2::new(p.x, p.y);
    let origin = Vec2::new(p.full_rotation_origin.0, p.full_rotation_origin.1);
    let rot = p.full_rotation;
    let w2 = (p.width as i32 / 2) as f32;
    let h2 = (p.height as i32 / 2) as f32;
    let dir = p.facing as f32;
    // A light's tile: `(int)(position + spinningpoint) / 16` (`:31279` etc.),
    // the point rotated about the origin only when `fullRotation != 0`.
    let emit = |out: &mut Vec<(i32, i32, [f32; 3])>, sp: Vec2, rgb: [f32; 3]| {
        let sp = if rot != 0.0 { rotate_about(sp, rot, origin) } else { sp };
        let x = (pos.x + sp.x) as i32 / 16;
        let y = (pos.y + sp.y) as i32 / 16;
        out.push((x, y, rgb));
    };
    // HEAD light (`:31263-31280`): +0.5 per triplet match with body / legs.
    if num != 0.0 || num2 != 0.0 || num3 != 0.0 {
        let mut num10 = 1.0f32;
        if num == num4 && num2 == num5 && num3 == num6 {
            num10 += 0.5;
        }
        if num == num7 && num2 == num8 && num3 == num9 {
            num10 += 0.5;
        }
        let sp = Vec2::new(w2 + 8.0 * dir, 2.0);
        emit(&mut out, sp, [num * num10, num2 * num10, num3 * num10]);
    }
    // BODY light (`:31281-31298`).
    if num4 != 0.0 || num5 != 0.0 || num6 != 0.0 {
        let mut num11 = 1.0f32;
        if num4 == num && num5 == num2 && num6 == num3 {
            num11 += 0.5;
        }
        if num4 == num7 && num5 == num8 && num6 == num9 {
            num11 += 0.5;
        }
        let sp = Vec2::new(w2 + 8.0, h2);
        emit(&mut out, sp, [num4 * num11, num5 * num11, num6 * num11]);
    }
    // LEGS light (`:31299-31316`): the y anchor is FLOAT height * 0.75.
    if num7 != 0.0 || num8 != 0.0 || num9 != 0.0 {
        let mut num12 = 1.0f32;
        if num7 == num4 && num8 == num5 && num9 == num6 {
            num12 += 0.5;
        }
        if num7 == num && num8 == num2 && num9 == num3 {
            num12 += 0.5;
        }
        let sp = Vec2::new(w2 + 8.0 * dir, p.height * 0.75);
        emit(&mut out, sp, [num7 * num12, num8 * num12, num9 * num12]);
    }
    out
}

/// The Chillet dash light (`DashMovement`, `Player.cs:21344-21354`): the
/// dash-6 active tick (`dashDelay < 0`), not the direction-flip end branch.
/// `base.Center` via the quotient-truncating overload; the Ignis pair (63/65)
/// burns warm, the rest purple. The 8-dust loop after it is skipped (RNG).
pub(crate) fn chillet_dash_light(p: &Player) -> Option<(i32, i32, [f32; 3])> {
    // `:21346`: `Math.Sign(velocity.X) != direction || dashType != 6` ends the
    // dash (no light); the else arm lights. The port's dash_movement uses the
    // same `vx.signum() as i32 != facing` convention (`player.rs:8745`).
    if p.dash != 6 || p.dash_delay >= 0 || p.dash_type != 6 {
        return None;
    }
    if p.vx.signum() as i32 != p.facing {
        return None;
    }
    let kind = p.mount.kind();
    let rgb = if kind == 63 || kind == 65 {
        [0.6, 0.3, 0.1]
    } else {
        [0.5, 0.1, 0.6]
    };
    let (cx, cy) = p.center();
    let (x, y) = quot_tile(Vec2::new(cx, cy));
    Some((x, y, rgb))
}

/// The Kazzymodus wing light (`WingFrame`, `Player.cs:29543-29549`): wings 28,
/// `ShouldDrawWingsThatAreAlwaysAnimated` (`:30737-30748`), `!flag`
/// (`wingsLogic != wings`, folds false in the port) and airborne. `base.Bottom`
/// (0.3, 0.1, 0.4). The whole block sits under `!CCed` (`:26799-26820`).
pub(crate) fn kazzymodus_wing_light(p: &Player) -> Option<(i32, i32, [f32; 3])> {
    if p.cced() || p.wings_logic != 28 || !p.has_wings {
        return None;
    }
    // `ShouldDrawWingsThatAreAlwaysAnimated`: vy != 0 && grappling[0] <= -1 &&
    // (!wet || !ShouldFloatInWater) && mount.CanUseWings.
    if p.vy == 0.0 || p.grappling_active || (p.wet && p.should_float_in_water()) {
        return None;
    }
    if !p.mount.can_use_wings() {
        return None;
    }
    let (bx, by) = p.bottom();
    let (x, y) = quot_tile(Vec2::new(bx, by));
    Some((x, y, [0.3, 0.1, 0.4]))
}

/// `Item.GetPhaseColorDirect` (`Item.cs:49111-49168`): the per-colour float
/// triplet (the `num *= …` products off the 1f base).
fn phase_direct(shoot: u16) -> [f32; 3] {
    match shoot {
        1059 | 1065 => [0.1, 0.5, 1.2],
        1060 | 1066 => [1.0, 0.2, 0.1],
        1061 | 1067 => [0.1, 1.0, 0.2],
        1062 | 1068 => [0.8, 0.1, 1.0],
        1063 | 1069 => [0.8, 0.9, 1.0],
        1064 | 1070 => [0.8, 0.8, 0.0],
        1071 | 1072 => [0.9, 0.5, 0.0],
        1075 | 1076 => [1.05, 0.1, 0.4],
        _ => [1.0, 1.0, 1.0],
    }
}

/// `new Color(float, float, float)` (`Color.cs`): clamp then `(int)(v * 255)`
/// per channel — the `render/pylons.rs::color_bytes` precedent.
fn color_bytes(v: [f32; 3]) -> [u8; 3] {
    [
        (v[0].clamp(0.0, 1.0) * 255.0) as u8,
        (v[1].clamp(0.0, 1.0) * 255.0) as u8,
        (v[2].clamp(0.0, 1.0) * 255.0) as u8,
    ]
}

/// `Color.Lerp` (`MonoGame Color.cs`): float lerp then the truncating byte
/// cast per channel (`tr_core::dust_table::lerp_color` precedent).
fn lerp_bytes(a: [u8; 3], b: [u8; 3], t: f32) -> [u8; 3] {
    let ch = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t) as u8;
    [ch(a[0], b[0]), ch(a[1], b[1]), ch(a[2], b[2])]
}

/// `Utils.MultiLerp(percent, colors)` (`Utils.cs:292-303`): the 1/(n-1) band
/// walk, then `Color.Lerp` of the bracketing pair.
fn multi_lerp_color(percent: f32, colors: &[[u8; 3]]) -> [u8; 3] {
    let num = 1.0f32 / (colors.len() as f32 - 1.0);
    let mut num2 = num;
    let mut num3 = 0usize;
    while percent / num2 > 1.0 && num3 < colors.len() - 2 {
        num2 += num;
        num3 += 1;
    }
    let t = (percent - num * num3 as f32) / num;
    lerp_bytes(colors[num3], colors[num3 + 1], t)
}

/// `Item.GetPhaseColor` (`Item.cs:49053-49109`): the 9-entry palette
/// (`_phaseColors`, `:49057-49068`), the shoot→index switch, and the rainbow
/// 1101/1102 `MultiLerp` at `miscCounterNormalized`. Returns bytes.
fn phase_color(shoot: u16, misc_counter: u32) -> [u8; 3] {
    let palette: [[u8; 3]; 9] = [
        color_bytes(phase_direct(1059)),
        color_bytes(phase_direct(1060)),
        color_bytes(phase_direct(1061)),
        color_bytes(phase_direct(1062)),
        color_bytes(phase_direct(1063)),
        color_bytes(phase_direct(1064)),
        color_bytes(phase_direct(1071)),
        color_bytes(phase_direct(1075)),
        color_bytes(phase_direct(1059)),
    ];
    match shoot {
        1059 | 1065 => palette[0],
        1060 | 1066 => palette[1],
        1061 | 1067 => palette[2],
        1062 | 1068 => palette[3],
        1063 | 1069 => palette[4],
        1064 | 1070 => palette[5],
        1071 | 1072 => palette[6],
        1075 | 1076 => palette[7],
        1101 | 1102 => {
            // `miscCounterNormalized` = miscCounter / 300f (`:3389`); the port
            // counter wraps without the 300 reset, so `% 300` recovers it.
            let percent = (misc_counter % 300) as f32 / 300.0;
            multi_lerp_color(percent, &palette)
        }
        _ => [255, 255, 255],
    }
}

/// The 18 phase-blade / phasesaber kinds (`Player.cs:44913`): 5670/5671,
/// 5535/5536, 4258/4259, 198-203, 3764-3769. All are `noUseGraphic = false`
/// (verified per-case in `Item.cs`), so the gate reduces to this list.
fn is_phase_blade(kind: u16) -> bool {
    matches!(kind, 5670 | 5671 | 5535 | 5536 | 4258 | 4259)
        || (198..=203).contains(&kind)
        || (3764..=3769).contains(&kind)
}

/// The swing `itemLocation` the two use-visual lights read.
/// `melee_swing.rs::swing_item_placement` over the raw tick counters (the
/// draw's `swing_anim * 60` convention, `render/entities.rs:1664`). The C#
/// `ItemCheck_ApplyUseStyle`'s `mountOffset` is `HeightOffsetHitboxCenter`
/// (`Player.cs:42846` → `:3253-3267`), i.e. `mount.PlayerOffsetHitbox` while
/// mounted — the port's `player_offset_hitbox()`; 0 unmounted.
fn swing_placement(p: &Player, kind: u16, frame_w: f32, frame_h: f32) -> Option<Vec2> {
    let anim = (p.frame_inputs.swing_anim * 60.0) as i32;
    let max = (p.frame_inputs.swing_anim_max * 60.0).max(1.0) as i32;
    let mount_offset = p.mount.player_offset_hitbox() as f32;
    let (loc, _) = crate::melee_swing::swing_item_placement(
        kind as i32,
        anim,
        max,
        p.facing,
        1.0,
        [p.x, p.y],
        WIDTH,
        p.height,
        frame_w,
        frame_h,
        mount_offset,
    );
    Some(Vec2::new(loc[0], loc[1]))
}

/// The phase-blade held light (`ItemCheck_EmitUseVisuals`,
/// `Player.cs:44913-44917`): `GetPhaseColor(shoot) * 0.5f` at
/// `(int)((itemLocation.X + 6 + velocity.X) / 16f)`,
/// `(int)((itemLocation.Y - 14) / 16f)`. The outer `ItemCheck` gate
/// (`:43648-43653`) is `itemAnimation > 0 && !dontAttack` (useStyle 1 → false).
pub(crate) fn phase_blade_light(
    p: &Player,
    kind: u16,
    frame_w: f32,
    frame_h: f32,
) -> Option<(i32, i32, [f32; 3])> {
    if !p.frame_inputs.swing_active || !is_phase_blade(kind) {
        return None;
    }
    let shoot = crate::registry::get(kind).and_then(|d| d.projectile)?;
    let loc = swing_placement(p, kind, frame_w, frame_h)?;
    // `Color * 0.5f` — the truncating byte operator*, then `(float)(int)R / 255`.
    let c = phase_color(shoot, p.misc_counter);
    let half = |b: u8| (b as f32 * 0.5) as u8;
    let rgb = [
        half(c[0]) as f32 / 255.0,
        half(c[1]) as f32 / 255.0,
        half(c[2]) as f32 / 255.0,
    ];
    let (x, y) = quot_tile(Vec2::new(loc.x + 6.0 + p.vx, loc.y - 14.0));
    Some((x, y, rgb))
}

/// The Chillet melee-swing light (`ItemCheck_EmitUseVisuals` tail,
/// `Player.cs:45028-45032`): mount 62-65, a melee swing, the dash colour × 0.5
/// (Vector3 FLOAT math) at `itemRectangle.Center`. The rect is the useStyle-1
/// `melee_swing.rs::melee_hitbox`; `Center` is the int half-sum.
pub(crate) fn chillet_swing_light(
    p: &Player,
    kind: u16,
    frame_w: f32,
    frame_h: f32,
) -> Option<(i32, i32, [f32; 3])> {
    let kind_mount = p.mount.kind();
    if !p.mount.is_active() || !(62..=65).contains(&kind_mount) {
        return None;
    }
    if !p.frame_inputs.swing_active {
        return None;
    }
    let def = crate::registry::get(kind)?;
    // `sItem.melee && !noMelee && !noUseGraphic` — the port reads the melee
    // class flag (noMelee/noUseGraphic have no ItemDef fields, module docs).
    if !def.melee || crate::item::use_style(kind) != 1 {
        return None;
    }
    let loc = swing_placement(p, kind, frame_w, frame_h)?;
    let scale = crate::player::adjusted_item_scale(def.scale);
    let rect = crate::melee_swing::melee_hitbox(
        kind as i32,
        (p.frame_inputs.swing_anim * 60.0) as i32,
        (p.frame_inputs.swing_anim_max * 60.0).max(1.0) as i32,
        p.facing,
        1.0,
        [loc.x, loc.y],
        frame_w as i32,
        frame_h as i32,
        scale,
    );
    // `Rectangle.Center` = (X + Width/2, Y + Height/2), int halves.
    let center = Vec2::new(
        (rect.x + rect.width / 2) as f32,
        (rect.y + rect.height / 2) as f32,
    );
    // Vector3 * 0.5f — float, NOT the byte Color operator*.
    let base = if kind_mount == 63 || kind_mount == 65 {
        [0.6f32, 0.3, 0.1]
    } else {
        [0.5, 0.1, 0.6]
    };
    let rgb = [base[0] * 0.5, base[1] * 0.5, base[2] * 0.5];
    let (x, y) = quot_tile(center);
    Some((x, y, rgb))
}

/// The whole gap batch, in the C#'s intra-`Update` family order (Rudolph trail
/// `:24491` → armor `:25684` → wing `:26820` → the dash + two use-visual lights
/// inside `ItemCheckWrapped` `:28285`). The `EntityLights` max-merge is
/// commutative and no arm draws RNG, so the order only documents the C# layout.
pub(crate) fn player_lights_b(
    p: &Player,
    held_kind: u16,
    frame_w: f32,
    frame_h: f32,
) -> Vec<(i32, i32, [f32; 3])> {
    let mut out = rudolph_trail(p);
    out.extend(armor_lights(p));
    if let Some(l) = chillet_dash_light(p) {
        out.push(l);
    }
    if let Some(l) = kazzymodus_wing_light(p) {
        out.push(l);
    }
    if let Some(l) = phase_blade_light(p, held_kind, frame_w, frame_h) {
        out.push(l);
    }
    if let Some(l) = chillet_swing_light(p, held_kind, frame_w, frame_h) {
        out.push(l);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::world::World;

    fn player() -> Player {
        Player::spawn(&World::new("t".into(), 64, 800))
    }

    #[test]
    fn rudolph_trail_steps_three_tiles_by_direction() {
        // `:24493-24497`: at (160, 176), mounted height 62: num = (160+10)/16
        // = 10, j = (176+31-14)/16 = 193/16 = 12. Facing right → 10, 11, 12.
        let mut p = player();
        p.x = 160.0;
        p.y = 176.0;
        p.facing = 1;
        assert!(p.set_mount(0, false), "Rudolph equips");
        // set_mount grows the hitbox to 42 + heightBoost 20 = 62, bottom-
        // anchored (y shifts -20); re-pin so num/j read the intended centre.
        p.x = 160.0;
        p.y = 176.0;
        let lights = rudolph_trail(&p);
        assert_eq!(
            lights,
            vec![
                (10, 12, [0.5, 0.2, 0.05]),
                (11, 12, [0.5, 0.2, 0.05]),
                (12, 12, [0.5, 0.2, 0.05]),
            ]
        );
        p.facing = -1;
        let lights = rudolph_trail(&p);
        assert_eq!(lights[1].0, 9);
        assert_eq!(lights[2].0, 8);
    }

    #[test]
    fn rudolph_trail_needs_the_reindeer_mount() {
        let mut p = player();
        p.x = 160.0;
        p.y = 176.0;
        assert!(rudolph_trail(&p).is_empty(), "unmounted");
        assert!(p.set_mount(2, false), "the slime mount equips");
        assert!(rudolph_trail(&p).is_empty(), "not Rudolph");
    }

    #[test]
    fn armor_lights_three_slots_and_the_match_bonus() {
        // The Spectre set: head 169 (0, .36, .4), body 175 (0, .36, .4),
        // legs 110 (0, .36, .4) — all three triplets equal → each light ×2.
        // Mining Helmet 88 → head_slot 11 is a different colour; use the
        // registry to resolve real kinds. Find kinds carrying the slots.
        let mut p = player();
        p.x = 160.0;
        p.y = 176.0;
        p.facing = 1;
        // Resolve kinds by slot id from the registry (the draw's convention).
        let find = |slot: i16, f: fn(&crate::registry::ItemDef) -> i16| {
            (1u16..7000).find(|&k| crate::registry::get(k).is_some_and(|d| f(d) == slot))
        };
        let head_kind = find(169, |d| d.head_slot).expect("a head 169 kind");
        let body_kind = find(175, |d| d.body_slot).expect("a body 175 kind");
        let legs_kind = find(110, |d| d.leg_slot).expect("a legs 110 kind");
        p.equipped[0] = head_kind;
        p.equipped[1] = body_kind;
        p.equipped[2] = legs_kind;
        let lights = armor_lights(&p);
        assert_eq!(lights.len(), 3, "head + body + legs");
        // All triplets (0, .36, .4) match → ×2 → (0, .72, .8).
        for l in &lights {
            assert_eq!(l.2, [0.0, 0.72, 0.8]);
        }
        // head anchor (w2 + 8*dir, 2) = (18, 2) → tile ((160+18)/16, (176+2)/16)
        // = (11, 11). body (w2+8, h2) = (18, 21) → (11, 12). legs (18, 31.5) →
        // (11, 12).
        assert_eq!((lights[0].0, lights[0].1), (11, 11));
        assert_eq!((lights[1].0, lights[1].1), (11, 12));
        assert_eq!((lights[2].0, lights[2].1), (11, 12));
    }

    #[test]
    fn armor_body_190_overwrites_the_head_triplet() {
        // `:31226-31230`: body 190 writes num/num2/num3 = (.9, .9, .7), NOT the
        // body triplet. With no head/legs, only the HEAD light fires at ×1.
        let mut p = player();
        p.x = 160.0;
        p.y = 176.0;
        let find = |slot: i16, f: fn(&crate::registry::ItemDef) -> i16| {
            (1u16..7000).find(|&k| crate::registry::get(k).is_some_and(|d| f(d) == slot))
        };
        p.equipped[1] = find(190, |d| d.body_slot).expect("a body 190 kind");
        let lights = armor_lights(&p);
        assert_eq!(lights.len(), 1, "only the head triplet is lit");
        assert_eq!(lights[0].2, [0.9, 0.9, 0.7]);
    }

    #[test]
    fn armor_legs_130_overwrites_the_head_triplet() {
        // `:31257-31261`: legs 130 writes the HEAD triplet too.
        let mut p = player();
        p.x = 160.0;
        p.y = 176.0;
        let find = |slot: i16, f: fn(&crate::registry::ItemDef) -> i16| {
            (1u16..7000).find(|&k| crate::registry::get(k).is_some_and(|d| f(d) == slot))
        };
        p.equipped[2] = find(130, |d| d.leg_slot).expect("a legs 130 kind");
        let lights = armor_lights(&p);
        assert_eq!(lights.len(), 1);
        assert_eq!(lights[0].2, [0.9, 0.9, 0.7]);
    }

    #[test]
    fn chillet_dash_colour_splits_on_the_ignis_pair() {
        let mut p = player();
        p.facing = 1;
        p.vx = 16.0;
        p.dash = 6;
        p.dash_type = 6;
        p.dash_delay = -1;
        assert!(p.set_mount(62, false), "the Chillet equips");
        // set_mount grows the hitbox bottom-anchored (shifting y); re-pin the
        // position so the centre is (170, 197) → quotient tile (10, 12).
        p.x = 160.0;
        p.y = 176.0;
        assert_eq!(chillet_dash_light(&p), Some((10, 12, [0.5, 0.1, 0.6])));
        assert!(p.set_mount(63, false), "the Ignis Chillet equips");
        p.x = 160.0;
        p.y = 176.0;
        assert_eq!(chillet_dash_light(&p), Some((10, 12, [0.6, 0.3, 0.1])));
    }

    #[test]
    fn chillet_dash_gates_on_the_active_non_flip_tick() {
        let mut p = player();
        p.x = 160.0;
        p.y = 176.0;
        p.facing = 1;
        p.vx = 16.0;
        p.dash = 6;
        p.dash_type = 6;
        p.dash_delay = -1;
        assert!(p.set_mount(62, false));
        assert!(chillet_dash_light(&p).is_some());
        p.dash_delay = 0; // ready, not dashing
        assert!(chillet_dash_light(&p).is_none());
        p.dash_delay = -1;
        p.vx = -16.0; // direction flip → the end branch, no light
        assert!(chillet_dash_light(&p).is_none());
        p.vx = 16.0;
        p.dash = 2; // not a mount dash
        assert!(chillet_dash_light(&p).is_none());
    }

    #[test]
    fn kazzymodus_wing_lights_the_bottom_when_airborne() {
        let mut p = player();
        p.x = 160.0;
        p.y = 176.0;
        p.wings_logic = 28;
        p.has_wings = true;
        p.vy = 2.0;
        // Bottom (170, 218) → quotient tile (10, 13).
        assert_eq!(kazzymodus_wing_light(&p), Some((10, 13, [0.3, 0.1, 0.4])));
        p.vy = 0.0;
        assert!(kazzymodus_wing_light(&p).is_none(), "grounded");
        p.vy = 2.0;
        p.grappling_active = true;
        assert!(kazzymodus_wing_light(&p).is_none(), "grappling");
        p.grappling_active = false;
        p.frozen = true;
        assert!(kazzymodus_wing_light(&p).is_none(), "CCed");
    }

    #[test]
    fn phase_color_byte_math_pins_the_blue_blade() {
        // Blue Phaseblade 198 → shoot 1059 → (0.1, 0.5, 1.2) → bytes
        // (25, 127, 255) (1.2 clamps to 255). * 0.5 → (12, 63, 127). / 255.
        let c = phase_color(1059, 0);
        assert_eq!(c, [25, 127, 255]);
        let half = |b: u8| (b as f32 * 0.5) as u8;
        assert_eq!([half(c[0]), half(c[1]), half(c[2])], [12, 63, 127]);
    }

    #[test]
    fn phase_color_rainbow_walks_the_palette() {
        // At miscCounter 0 the MultiLerp sits at palette[0] (the blue base).
        assert_eq!(phase_color(1101, 0), phase_color(1059, 0));
        // At miscCounter 150 (percent 0.5) the band walk lands mid-palette;
        // pin it against a direct MultiLerp of the same palette.
        let palette: [[u8; 3]; 9] = [
            color_bytes(phase_direct(1059)),
            color_bytes(phase_direct(1060)),
            color_bytes(phase_direct(1061)),
            color_bytes(phase_direct(1062)),
            color_bytes(phase_direct(1063)),
            color_bytes(phase_direct(1064)),
            color_bytes(phase_direct(1071)),
            color_bytes(phase_direct(1075)),
            color_bytes(phase_direct(1059)),
        ];
        assert_eq!(phase_color(1101, 150), multi_lerp_color(0.5, &palette));
        // The 300-wrap fold: counter 300 ≡ counter 0.
        assert_eq!(phase_color(1101, 300), phase_color(1101, 0));
    }

    #[test]
    fn phase_blade_light_gates_on_the_swing_and_kind() {
        let mut p = player();
        p.x = 160.0;
        p.y = 176.0;
        p.facing = 1;
        p.frame_inputs.swing_active = true;
        p.frame_inputs.swing_anim = 10.0 / 60.0;
        p.frame_inputs.swing_anim_max = 20.0 / 60.0;
        // A non-phaseblade kind never lights.
        assert!(phase_blade_light(&p, 8, 20.0, 20.0).is_none());
        // The Blue Phaseblade lights while swinging.
        assert!(phase_blade_light(&p, 198, 40.0, 40.0).is_some());
        p.frame_inputs.swing_active = false;
        assert!(phase_blade_light(&p, 198, 40.0, 40.0).is_none());
    }

    #[test]
    fn chillet_swing_light_halves_the_dash_colour_float() {
        let mut p = player();
        p.x = 160.0;
        p.y = 176.0;
        p.facing = 1;
        p.frame_inputs.swing_active = true;
        p.frame_inputs.swing_anim = 10.0 / 60.0;
        p.frame_inputs.swing_anim_max = 20.0 / 60.0;
        assert!(p.set_mount(62, false));
        // A melee useStyle-1 kind (the Copper Broadsword 4 is melee swing).
        let l = chillet_swing_light(&p, 4, 32.0, 32.0).expect("the swing lights");
        // Vector3 × 0.5 float: (0.5, 0.1, 0.6) → (0.25, 0.05, 0.3).
        assert_eq!(l.2, [0.25, 0.05, 0.3]);
        assert!(p.set_mount(63, false));
        let l = chillet_swing_light(&p, 4, 32.0, 32.0).expect("the Ignis swing");
        assert_eq!(l.2, [0.3, 0.15, 0.05]);
    }

    #[test]
    fn chillet_swing_light_needs_the_mount_and_melee() {
        let mut p = player();
        p.x = 160.0;
        p.y = 176.0;
        p.frame_inputs.swing_active = true;
        p.frame_inputs.swing_anim = 10.0 / 60.0;
        p.frame_inputs.swing_anim_max = 20.0 / 60.0;
        // Unmounted → no light.
        assert!(chillet_swing_light(&p, 4, 32.0, 32.0).is_none());
        assert!(p.set_mount(62, false));
        // A non-melee kind (the torch 8) → no light.
        assert!(chillet_swing_light(&p, 8, 20.0, 20.0).is_none());
    }

    #[test]
    fn the_batch_is_empty_for_a_plain_idle_player() {
        let p = player();
        assert!(player_lights_b(&p, 0, 0.0, 0.0).is_empty());
    }
}
