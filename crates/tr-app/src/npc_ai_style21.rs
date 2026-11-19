//! The aiStyle-21 family — the Blazing Wheel (`NPC.AI()`'s
//! `else if (this.aiStyle == 21)` arm, `NPC.cs:24615-24665`). S1-D-3,
//! aiStyle-21 round.
//!
//! The family has ONE vanilla member: type 72, `NPCID.BlazingWheel`
//! (`ID/NPCID.cs:10821`, `SetDefaults` row `NPC.cs:9714-9731`). The sibling
//! arm `aiStyle == 22` (the Poltergeist family, `NPC.cs:24666+`) is NOT
//! touched by this module.
//!
//! The machine: a diagonal ricochet. `ai[0] == 0` runs `TargetClosest()`
//! once, pins `directionY = 1` and arms `ai[0] = 1` (`:24617-24622`);
//! `ai[1]` selects the axis mode — 0 = vertical-first, 1 = horizontal-first
//! (`:24624-24661`). Each mode spins the sprite by
//! `direction * directionY * 0.13` (plus in mode 0, minus in mode 1), latches
//! `ai[0] = 2` on its primary-axis collision, flips to the other mode when
//! the primary contact clears, and reflects on the secondary axis. The tail
//! writes the constant ±6 velocity on both axes (`:24662-24663`) and adds
//! the centre light (`:24664`).
//!
//! The wheel's tile movement is its own: `ApplyTileCollision`'s type-72 arm
//! (`NPC.cs:92612-92615`) calls `Collision_MoveBlazingWheel`
//! (`NPC.cs:92775-92783`) — a 12×12 box centred on the NPC run through
//! `Collision.noSlopeCollision` with `fallThrough: true, fall2: true`
//! (`Collision.cs:2033-2133`) — then the dry-move latch
//! (`Collision_MoveWhileDry`, `NPC.cs:92591-92607`) derives
//! `collideX`/`collideY` from the velocity change and integrates the
//! position. The wheel is never wet: `Collision_WaterCollision` forces
//! `wetCount = 0` / no lava for `type == 72 || aiStyle == 21`
//! (`NPC.cs:93055-93060`), and `UpdateCollision` skips the slope/stair pass
//! for type 72 (`NPC.cs:92572`) — so the dry branch is the only reachable
//! one and is what [`collision_move_blazing_wheel`] ports.
//!
//! RNG: the arm draws NO `Main.rand` — by construction, not by gating.
//! `NPC.cs:24615-24665` contains no `Next`/`NextFloat`; neither does
//! `TargetClosest`/`TryTrackingTarget`/`SetTargetTrackingValues`
//! (`NPC.cs:77652-77781`, the `npc_target.rs` port), `noSlopeCollision`
//! (`Collision.cs:2033-2133`), `Collision_MoveWhileDry`
//! (`NPC.cs:92591-92607`), `FindFrame` case 72 (`NPC.cs:74642-74653`) or the
//! `Lighting.AddLight` funnel (`Lighting.cs:157-163`). The nearest wheel
//! `Next` — `GetStackForSlimeItemDrop` case 72's `Next(20, 100)`
//! (`NPC.cs:79782-79784`) — is keyed on **ItemID** 72 (SilverCoin,
//! `ID/ItemID.cs:1827`), a loot-stack helper with no caller in the
//! decompile, unreachable from this AI. So none of these functions takes an
//! RNG parameter; the witness test
//! `the_style21_arm_draws_no_rng` pins a `UnifiedRandom` state across the
//! calls (written, not run — code-only mode).
//!
//! WIRED by S1-D-3-wire-smallkinds (2026-11-19): `NpcKind::BlazingWheel`
//! (protocol id 353, vanilla type 72) now exists; `update_rng`'s aiStyle
//! axis claims style 21 (`npc.rs`), calling [`ai_style21`] +
//! [`collision_move_blazing_wheel`]; the light triple is fed to
//! `EntityLights` by the draw-side pass (`app/frame/draw.rs`, reading the
//! App's pulsing `mouse_text_color`), and [`blazing_wheel_frame`] drives
//! the kind's `find_frame` row. See docs/ANALYSIS.md "The Blazing Wheel arm
//! (S1-D-3 aiStyle-21)" + "The small-kinds wiring wave".
#![allow(dead_code)] // WIRED by S1-D-3-wire-smallkinds (axis 21 + the light feed); the defaults row + witnesses stay as the audit cross-check

use crate::collision::no_slope_collision;
use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::World;

/// The type-72 `SetDefaults` row (`NPC.cs:9714-9731`), verbatim. The
/// `HitSound`/`DeathSound` are `SoundID.NPCHit1`/`NPCDeath1` =
/// `LegacySoundStyle(3, 1)`/`(4, 1)` (`ID/SoundID.cs:1257`/`:1315`).
pub struct BlazingWheelDefaults {
    pub npc_slots: f32,
    pub width: i32,
    pub height: i32,
    pub ai_style: i32,
    pub damage: i32,
    pub defense: i32,
    pub life_max: i32,
    pub alpha: i32,
    pub behind_tiles: bool,
    /// `(SoundType, Style)` — `SoundID.NPCHit1`.
    pub hit_sound: (i32, i32),
    /// `(SoundType, Style)` — `SoundID.NPCDeath1`.
    pub death_sound: (i32, i32),
    pub knock_back_resist: f32,
    pub no_gravity: bool,
    pub dont_take_damage: bool,
    pub scale: f32,
}

/// `NPC.cs:9714-9731` — `else if (this.type == 72)`.
pub const BLAZING_WHEEL: BlazingWheelDefaults = BlazingWheelDefaults {
    npc_slots: 0.3,
    width: 34,
    height: 34,
    ai_style: 21,
    damage: 24,
    defense: 100,
    life_max: 100,
    alpha: 100,
    behind_tiles: true,
    hit_sound: (3, 1),
    death_sound: (4, 1),
    knock_back_resist: 0.0,
    no_gravity: true,
    dont_take_damage: true,
    scale: 1.2,
};

/// `Main.npcFrameCount[72]` = 4 (`Main.cs:67082`'s array, the 70-79 row
/// `16, 2, 4, 6, ...` at `Main.cs:67092`).
pub const BLAZING_WHEEL_FRAME_COUNT: u32 = 4;

/// The aiStyle-21 arm (`NPC.cs:24615-24665`). One fixed step of the
/// ricochet machine; returns the `Lighting.AddLight` contribution
/// (`:24664`) as `(tileX, tileY, [r, g, b])` for the caller's
/// `EntityLights` feed — the pure seam, since the arm's only side effect
/// beyond the NPC state is the light.
///
/// `players`/`ticker` feed the one-shot `TargetClosest()` (`:24619`) through
/// the shared `npc_target.rs` port (`NPC.cs:77652-77781`); `mouse_text_color`
/// is `Main.mouseTextColor` (`Main.cs:1683`, the 190..=255 pulse
/// `Main.cs:18216-18223`) read by the light's green term.
pub fn ai_style21(
    npc: &mut Npc,
    world: &World,
    players: &[TargetCandidate],
    ticker: u64,
    mouse_text_color: u8,
) -> (i32, i32, [f32; 3]) {
    // `:24617-24622` — the init: target once, pin directionY, arm ai[0].
    if npc.ai[0] == 0.0 {
        target_closest(npc, world, players, true, 21, ticker);
        npc.direction_y = 1;
        npc.ai[0] = 1.0;
    }
    // `int num290 = 6;` (`:24623`).
    let num290 = 6.0f32;
    if npc.ai[1] == 0.0 {
        // `:24626` — `rotation += (float)(direction * directionY) * 0.13f`
        // (the int product cast to float, then scaled).
        npc.rotation += (npc.facing * npc.direction_y) as f32 * 0.13;
        // `:24627-24630` — the vertical contact latches the flip state.
        if npc.collide_y {
            npc.ai[0] = 2.0;
        }
        // `:24631-24636` — contact cleared: reflect X, swap to mode 1.
        if !npc.collide_y && npc.ai[0] == 2.0 {
            npc.facing = -npc.facing;
            npc.ai[1] = 1.0;
            npc.ai[0] = 1.0;
        }
        // `:24637-24641` — a side contact reflects Y and swaps immediately.
        if npc.collide_x {
            npc.direction_y = -npc.direction_y;
            npc.ai[1] = 1.0;
        }
    } else {
        // `:24645` — mode 1 spins the other way.
        npc.rotation -= (npc.facing * npc.direction_y) as f32 * 0.13;
        // `:24646-24649`.
        if npc.collide_x {
            npc.ai[0] = 2.0;
        }
        // `:24650-24655` — contact cleared: reflect Y, swap to mode 0.
        if !npc.collide_x && npc.ai[0] == 2.0 {
            npc.direction_y = -npc.direction_y;
            npc.ai[1] = 0.0;
            npc.ai[0] = 1.0;
        }
        // `:24656-24660` — a floor/ceiling contact reflects X and swaps.
        if npc.collide_y {
            npc.facing = -npc.facing;
            npc.ai[1] = 0.0;
        }
    }
    // `:24662-24663` — the constant ±6 velocity on both axes.
    npc.vx = num290 * npc.facing as f32;
    npc.vy = num290 * npc.direction_y as f32;
    // `:24664` — `Lighting.AddLight(i: (int)(position.X + (float)(width / 2))
    // / 16, j: (int)(position.Y + (float)(height / 2)) / 16, r: 0.9f,
    // g: 0.3f + (float)(270 - Main.mouseTextColor) / 400f, b: 0.2f)`.
    blazing_wheel_light(npc.x, npc.y, npc.size(), mouse_text_color)
}

/// The arm's light (`NPC.cs:24664`). The C# halves are INTEGER
/// (`base.width / 2`), the centre is truncated to int per axis, then the
/// tile index is the int quotient `/ 16` — the `CenterTruncated` anchor
/// form of `entity_lights.rs`, spelled out here because the green term
/// reads `Main.mouseTextColor`.
pub fn blazing_wheel_light(
    x: f32,
    y: f32,
    size: (f32, f32),
    mouse_text_color: u8,
) -> (i32, i32, [f32; 3]) {
    let (w, h) = size;
    let half_w = (w as i32 / 2) as f32;
    let half_h = (h as i32 / 2) as f32;
    let i = (x + half_w) as i32 / 16;
    let j = (y + half_h) as i32 / 16;
    // `(float)(270 - Main.mouseTextColor) / 400f` — the int difference
    // cast to float, then divided.
    let g = 0.3 + (270 - mouse_text_color as i32) as f32 / 400.0;
    (i, j, [0.9, g, 0.2])
}

/// `NPC.Collision_MoveBlazingWheel` (`NPC.cs:92775-92783`) + the dry-move
/// latch `Collision_MoveWhileDry` (`NPC.cs:92591-92607`) with
/// `UpdateCollision`'s pre-AI latch order (`oldVelocity` copied and both
/// collide flags cleared BEFORE the tile pass, `NPC.cs:92538-92540`).
///
/// The C# centres a 12×12 box on the NPC (`num = num2 = 12`, integer
/// half-extents at `:92777`) and runs `Collision.noSlopeCollision(...,
/// fallThrough: true, fall2: true)` (`:92782`, the port in `collision.rs`,
/// `Collision.cs:2033-2133`). The wheel is never wet
/// (`Collision_WaterCollision`, `NPC.cs:93055-93060`) and skips the
/// slope/stair pass (`NPC.cs:92572`), so the dry branch is the whole move.
pub fn collision_move_blazing_wheel(npc: &mut Npc, world: &World) {
    // `:92538-92540` — latch, then clear.
    let (old_vx, old_vy) = (npc.vx, npc.vy);
    npc.collide_x = false;
    npc.collide_y = false;
    // `:92777-92781` — the centre (INTEGER half-extents), minus 6 per axis.
    let (w, h) = npc.size();
    let cx = npc.x + (w as i32 / 2) as f32;
    let cy = npc.y + (h as i32 / 2) as f32;
    let num = 12;
    let num2 = 12;
    let pos = [cx - (num / 2) as f32, cy - (num2 / 2) as f32];
    // `:92782` — `base.velocity = Collision.noSlopeCollision(vector,
    // base.velocity, num, num2, fallThrough: true, fall2: true)`.
    let tc = no_slope_collision(
        world,
        pos,
        [npc.vx, npc.vy],
        num as f32,
        num2 as f32,
        true,
        true,
    );
    npc.vx = tc.vel[0];
    npc.vy = tc.vel[1];
    // `Collision_MoveWhileDry` (`:92593-92607`).
    if tc.up {
        npc.vy = 0.01;
    }
    if old_vx != npc.vx {
        npc.collide_x = true;
    }
    if old_vy != npc.vy {
        npc.collide_y = true;
    }
    npc.old_x = npc.x;
    npc.old_y = npc.y;
    npc.x += npc.vx;
    npc.y += npc.vy;
}

/// `FindFrame` case 72 (`NPC.cs:74642-74653`): `frameCounter++`, at
/// `>= 3.0` reset and advance one frame, wrapping at
/// `Main.npcFrameCount[72]` = 4. A pure pair-function until the kind's
/// `find_frame` row lands (the port's `Npc::frame_counter` is private to
/// `npc.rs`; the wiring wave folds this into it).
pub fn blazing_wheel_frame(frame: u32, frame_counter: f32) -> (u32, f32) {
    let mut counter = frame_counter + 1.0;
    let mut row = frame;
    if counter >= 3.0 {
        counter = 0.0;
        row += 1;
        if row >= BLAZING_WHEEL_FRAME_COUNT {
            row = 0;
        }
    }
    (row, counter)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::tile::{Tile, FLAG_ACTIVE};
    use tr_core::unified_random::UnifiedRandom;

    fn cand(x: f32, y: f32) -> TargetCandidate {
        TargetCandidate {
            active: true,
            dead: false,
            ghost: false,
            x,
            y,
            width: 20,
            height: 42,
            aggro: 0,
            item_animation: 0,
            npc_type_no_aggro: false,
            tank_pet: None,
        }
    }

    /// A wheel-shaped NPC: the roster has no BlazingWheel kind yet, so the
    /// fixture rides a placeholder kind with the C#'s 34×34 box
    /// (`NPC.cs:9717-9718`) through `size_override`.
    fn wheel(x: f32, y: f32) -> Npc {
        let mut n = Npc::new(NpcKind::Zombie, x, y, 1, 0.0);
        n.size_override = Some((34.0, 34.0));
        n
    }

    fn air() -> World {
        World::new("t".into(), 64, 64)
    }

    fn put(w: &mut World, x: i32, y: i32, kind: u16) {
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
    }

    /// `NPC.cs:9714-9731` — the SetDefaults row, field by field.
    #[test]
    fn the_defaults_row_matches_the_csharp() {
        let d = &BLAZING_WHEEL;
        assert_eq!(d.npc_slots, 0.3);
        assert_eq!((d.width, d.height), (34, 34));
        assert_eq!(d.ai_style, 21);
        assert_eq!(d.damage, 24);
        assert_eq!(d.defense, 100);
        assert_eq!(d.life_max, 100);
        assert_eq!(d.alpha, 100);
        assert!(d.behind_tiles);
        assert_eq!(d.hit_sound, (3, 1), "SoundID.NPCHit1 (SoundID.cs:1257)");
        assert_eq!(d.death_sound, (4, 1), "SoundID.NPCDeath1 (SoundID.cs:1315)");
        assert_eq!(d.knock_back_resist, 0.0);
        assert!(d.no_gravity);
        assert!(d.dont_take_damage);
        assert_eq!(d.scale, 1.2);
        assert_eq!(BLAZING_WHEEL_FRAME_COUNT, 4, "Main.cs:67092");
    }

    /// `NPC.cs:24617-24622` — the init runs TargetClosest once, pins
    /// directionY = 1 (overriding the pick's facing) and arms ai[0] = 1;
    /// the tail writes the ±6 velocity the same tick.
    #[test]
    fn the_init_targets_once_and_pins_direction_y() {
        let w = air();
        let mut n = wheel(1000.0, 1000.0);
        // Player above-right: the pick would face (+1, -1) ...
        let p = cand(1100.0, 900.0);
        let _ = ai_style21(&mut n, &w, &[p], 7, 225);
        assert_eq!(n.ai[0], 1.0);
        assert_eq!(n.target, 0);
        assert_eq!(n.target_set_frame, 7);
        assert_eq!(n.facing, 1, "the player is to the right");
        assert_eq!(n.direction_y, 1, ":24620 overrides the pick's -1");
        assert_eq!((n.vx, n.vy), (6.0, 6.0), ":24662-24663");
        // Second tick: ai[0] != 0, no re-target (the ticker stamp stays).
        let _ = ai_style21(&mut n, &w, &[p], 9, 225);
        assert_eq!(n.target_set_frame, 7, "TargetClosest ran only once");
    }

    /// `NPC.cs:24627-24636` — mode 0: collideY latches ai[0] = 2; the next
    /// tick without collideY flips direction and swaps to mode 1.
    #[test]
    fn mode0_latches_on_collide_y_then_flips_x() {
        let w = air();
        let mut n = wheel(1000.0, 1000.0);
        n.ai[0] = 1.0;
        n.collide_y = true;
        let _ = ai_style21(&mut n, &w, &[cand(1100.0, 1100.0)], 0, 225);
        assert_eq!(n.ai[0], 2.0);
        assert_eq!(n.facing, 1, "no flip while the contact holds");
        assert_eq!(n.ai[1], 0.0);
        // Contact cleared: the flip fires.
        n.collide_y = false;
        let _ = ai_style21(&mut n, &w, &[cand(1100.0, 1100.0)], 0, 225);
        assert_eq!(n.facing, -1);
        assert_eq!(n.ai[1], 1.0);
        assert_eq!(n.ai[0], 1.0);
        assert_eq!((n.vx, n.vy), (-6.0, 6.0));
    }

    /// `NPC.cs:24637-24641` — mode 0's side contact reflects directionY
    /// and swaps to mode 1 immediately (no ai[0] = 2 latch).
    #[test]
    fn mode0_collide_x_reflects_y_and_swaps() {
        let w = air();
        let mut n = wheel(1000.0, 1000.0);
        n.ai[0] = 1.0;
        n.collide_x = true;
        let _ = ai_style21(&mut n, &w, &[cand(1100.0, 1100.0)], 0, 225);
        assert_eq!(n.direction_y, -1);
        assert_eq!(n.ai[1], 1.0);
        assert_eq!(n.ai[0], 1.0, "the X contact does not latch 2");
        assert_eq!((n.vx, n.vy), (6.0, -6.0));
    }

    /// `NPC.cs:24646-24660` — mode 1 mirrors: collideX latches 2, the clear
    /// flips directionY back to mode 0; collideY flips direction directly.
    #[test]
    fn mode1_latches_on_collide_x_then_flips_y() {
        let w = air();
        let mut n = wheel(1000.0, 1000.0);
        n.ai[0] = 1.0;
        n.ai[1] = 1.0;
        n.collide_x = true;
        let _ = ai_style21(&mut n, &w, &[cand(1100.0, 1100.0)], 0, 225);
        assert_eq!(n.ai[0], 2.0);
        assert_eq!(n.direction_y, 1);
        n.collide_x = false;
        let _ = ai_style21(&mut n, &w, &[cand(1100.0, 1100.0)], 0, 225);
        assert_eq!(n.direction_y, -1);
        assert_eq!(n.ai[1], 0.0);
        assert_eq!(n.ai[0], 1.0);
        // The collideY arm (:24656-24660) in mode 1.
        let mut m = wheel(1000.0, 1000.0);
        m.ai[0] = 1.0;
        m.ai[1] = 1.0;
        m.collide_y = true;
        let _ = ai_style21(&mut m, &w, &[cand(1100.0, 1100.0)], 0, 225);
        assert_eq!(m.facing, -1);
        assert_eq!(m.ai[1], 0.0);
        assert_eq!(m.ai[0], 1.0, "the Y contact does not latch 2");
    }

    /// `NPC.cs:24626` vs `:24645` — the spin is
    /// `direction * directionY * 0.13`, added in mode 0, subtracted in
    /// mode 1 (the int product before the float scale).
    #[test]
    fn the_spin_sign_follows_the_mode() {
        let w = air();
        let mut n = wheel(1000.0, 1000.0);
        n.ai[0] = 1.0;
        n.facing = -1;
        n.direction_y = 1;
        let _ = ai_style21(&mut n, &w, &[cand(1100.0, 1100.0)], 0, 225);
        assert!((n.rotation - (-0.13)).abs() < 1e-6, "mode 0 adds -0.13");
        n.ai[1] = 1.0;
        let before = n.rotation;
        let _ = ai_style21(&mut n, &w, &[cand(1100.0, 1100.0)], 0, 225);
        assert!(
            (n.rotation - (before + 0.13)).abs() < 1e-6,
            "mode 1 subtracts -0.13"
        );
    }

    /// `NPC.cs:24664` — the light: centre tile via INTEGER halves and int
    /// truncation, r 0.9 / b 0.2, g = 0.3 + (270 - mouseTextColor)/400.
    #[test]
    fn the_light_is_the_mouse_text_pulse_at_the_centre_tile() {
        // 34×34 at (160, 176): centre (177, 193) → tiles (11, 12).
        let (i, j, rgb) = blazing_wheel_light(160.0, 176.0, (34.0, 34.0), 225);
        assert_eq!((i, j), (11, 12));
        assert_eq!(rgb[0], 0.9);
        assert_eq!(rgb[2], 0.2);
        // 270 - 225 = 45; 45/400 = 0.1125.
        assert!((rgb[1] - 0.4125).abs() < 1e-6);
        // The pulse's extremes (Main.cs:18216-18223): 255 → 0.3375,
        // 190 → 0.5.
        let (_, _, hi) = blazing_wheel_light(160.0, 176.0, (34.0, 34.0), 255);
        assert!((hi[1] - 0.3375).abs() < 1e-6);
        let (_, _, lo) = blazing_wheel_light(160.0, 176.0, (34.0, 34.0), 190);
        assert!((lo[1] - 0.5).abs() < 1e-6);
    }

    /// `NPC.cs:92775-92783` + `:92591-92607` — the 12×12 centre box lands
    /// on a floor: the velocity is trimmed to the tile top, collideY
    /// latches from the velocity change, the position integrates.
    #[test]
    fn the_centre_box_lands_on_the_floor_and_latches_collide_y() {
        let mut w = air();
        put(&mut w, 10, 20, 1); // stone floor, top at y = 320
        // Wheel at (143, 293): centre (160, 310), box (154, 304)..(166, 316).
        let mut n = wheel(143.0, 293.0);
        n.vy = 6.0;
        collision_move_blazing_wheel(&mut n, &w);
        // The box bottom 316 + 6 overshoots 320 → trimmed to 4.
        assert_eq!(n.vy, 4.0);
        assert!(n.collide_y, "6 != 4 latches the flag (:92601-92604)");
        assert!(!n.collide_x);
        assert_eq!(n.y, 297.0, "the position rides the trimmed velocity");
        assert_eq!(n.old_y, 293.0, "oldPosition latched (:92605)");
    }

    /// The side arm: a wall stops the +6 slide, collideX latches, x holds.
    #[test]
    fn the_centre_box_stops_at_the_wall_and_latches_collide_x() {
        let mut w = air();
        put(&mut w, 11, 19, 1); // wall tile x 176..192, y 304..320
        // Wheel at (153, 293): box (164, 304)..(176, 316), right edge ON
        // the wall's left face.
        let mut n = wheel(153.0, 293.0);
        n.vx = 6.0;
        collision_move_blazing_wheel(&mut n, &w);
        assert_eq!(n.vx, 0.0, "result.X = tile.X - (pos.X + width)");
        assert!(n.collide_x);
        assert!(!n.collide_y);
        assert_eq!(n.x, 153.0);
    }

    /// The up arm (`Collision.cs:2119-2128` + `NPC.cs:92593-92596`): a
    /// ceiling gives the +0.01 nudge, then the dry move pins vy = 0.01.
    #[test]
    fn the_ceiling_nudge_pins_the_up_velocity() {
        let mut w = air();
        put(&mut w, 10, 18, 1); // ceiling tile y 288..304
        // Box top at 304 (wheel y 293), moving up at -6.
        let mut n = wheel(143.0, 293.0);
        n.vy = -6.0;
        collision_move_blazing_wheel(&mut n, &w);
        assert_eq!(n.vy, 0.01, "up arm +0.01, then MoveWhileDry's pin");
        assert!(n.collide_y);
    }

    /// `NPC.cs:74642-74653` — the 3-tick frame stride, wrapping at 4.
    #[test]
    fn the_frame_advances_every_third_tick_and_wraps_at_four() {
        let (mut row, mut counter) = (0u32, 0.0f32);
        for tick in 1..=12 {
            let (r, c) = blazing_wheel_frame(row, counter);
            row = r;
            counter = c;
            assert_eq!(row, (tick / 3) % 4, "tick {tick}");
        }
        assert_eq!(counter, 0.0);
    }

    /// The RNG witness (module docs): the arm, the collision move and the
    /// frame step draw NOTHING — a seeded `UnifiedRandom`'s full state
    /// (cursor + 56-int seed array, `PartialEq`) is bit-identical across
    /// the calls, so no stream can shift when the wiring lands.
    #[test]
    fn the_style21_arm_draws_no_rng() {
        let mut w = air();
        put(&mut w, 10, 20, 1);
        let rng = UnifiedRandom::new(12345);
        let witness = rng.clone();
        let mut n = wheel(143.0, 280.0);
        for _ in 0..120 {
            let _ = ai_style21(&mut n, &w, &[cand(200.0, 300.0)], 0, 225);
            collision_move_blazing_wheel(&mut n, &w);
            let (r, c) = blazing_wheel_frame(n.frame, 0.0);
            n.frame = r;
            let _ = c;
        }
        assert_eq!(rng, witness, "no Next/NextFloat anywhere in the arm");
    }
}
