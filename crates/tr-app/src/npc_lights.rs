//! The NPC `Lighting.AddLight` sources (`S1-E-5`).
//!
//! C# `Lighting.AddLight` funnels every overload into
//! `Lighting.AddLight(int i, int j, float r, float g, float b)`
//! (`Lighting.cs:157-163`), which appends to the active engine's per-frame list
//! behind `!Main.gamePaused && Main.netMode != 2` (`LightingEngine.cs:51-54`).
//! The engine then max-merges that list onto the working light map
//! (`LightingEngine.cs:208-235`), one frame late. [`EntityLights`] is that pair
//! of lists.
//!
//! Ported here: NPC `AI` methods' `AddLight` arms (`NPC.cs` various lines).
//! Arms that draw `Main.rand` are **not** ported — they belong to the
//! NPC-visual dust pass, which needs the dust system (S1-E-2/S1-E-3, other
//! branches). Skipping an arm wholesale leaves the `Main.rand` stream untouched,
//! so the fixed-step order stays exact.

use glam::Vec2;
use tr_core::lighting::LightParams;
use tr_core::TILE_PX;

use crate::npc::NpcKind;

/// Where the C# reads the light's tile x from. The three forms are **not**
/// interchangeable for negative coordinates (`(int)(v/16)` truncates the
/// quotient, `(int)v / 16` truncates first).
#[derive(Clone, Copy, PartialEq)]
enum XAnchor {
    /// `(int)((position.X + width / 2f) / 16f)` — the common arm.
    Center,
    /// `(int)((position.X + width) / 16f)` — the right edge.
    Right,
    /// `(int)Center.X / 16` — truncate before dividing.
    CenterTruncated,
    /// `(int)(position.X / 16f)` — the top-left (no size added). Used by
    /// `Lighting.AddLight(Vector2, ...)` overloads that take a bare position
    /// (e.g. `base.Top`).
    Top,
}

#[inline]
fn tile_x(pos: Vec2, size: f32, anchor: XAnchor) -> i32 {
    match anchor {
        XAnchor::Center => ((pos.x + size / 2.0) / TILE_PX) as i32,
        XAnchor::Right => ((pos.x + size) / TILE_PX) as i32,
        XAnchor::CenterTruncated => (pos.x + size / 2.0) as i32 / 16,
        XAnchor::Top => (pos.x / 16.0) as i32,
    }
}

#[inline]
fn tile_y(pos: Vec2, size: f32, anchor: XAnchor) -> i32 {
    match anchor {
        XAnchor::CenterTruncated => (pos.y + size / 2.0) as i32 / 16,
        XAnchor::Top => (pos.y / 16.0) as i32,
        _ => ((pos.y + size / 2.0) / TILE_PX) as i32,
    }
}

/// NPC `AI` methods' `AddLight` source this NPC contributes this tick, or `None`.
///
/// `pos` is the NPC's top-left and `size` its hitbox width/height.
/// `rng` is the per-NPC RNG stream (passed for parity; current arms don't use it).
///
/// C# source: `NPC.AI` method arms calling `Lighting.AddLight`.
pub(crate) fn npc_light(
    kind: NpcKind,
    pos: Vec2,
    width: f32,
    height: f32,
    _rng: &mut tr_core::UnifiedRandom,
) -> Option<(i32, i32, [f32; 3])> {
    let mut x_anchor = XAnchor::Center;
    let mut y_anchor = XAnchor::Center;
    let rgb: [f32; 3];

    // The arm order follows NPC.cs AI method order.
    // NPC.cs:21358-21361 — `else if (this.type == 533) { Lighting.AddLight(base.Top, 0.6f, 0.6f, 0.3f); }`
    // `Lighting.AddLight(Vector2, float, float, float)` -> `(int)(Top.X / 16f), (int)(Top.Y / 16f)`
    // (`Lighting.cs:140-142`), so the top-left is truncated after the divide.
    if kind == NpcKind::DesertDjinn {
        x_anchor = XAnchor::Top;
        y_anchor = XAnchor::Top;
        rgb = [0.6, 0.6, 0.3];
    }
    // NPC.cs:25235-25250 — aiStyle 23 NPCs (PrimeVice, DD2KoboldFlyerT3, and fallback)
    // All three use center-tile calculation:
    // `(int)((base.position.X + (float)(base.width / 2)) / 16f)` -> XAnchor::Center
    // `(int)((base.position.Y + (float)(base.height / 2)) / 16f)` -> XAnchor::Center
    else if kind == NpcKind::PrimeVice {
        // C# reference: NPC.cs:25239-25241 — `if (this.type == 83) { Lighting.AddLight(..., 0.2f, 0.05f, 0.3f); }`
        x_anchor = XAnchor::Center;
        y_anchor = XAnchor::Center;
        rgb = [0.2, 0.05, 0.3];
    } else if kind == NpcKind::DD2KoboldFlyerT3 {
        // C# reference: NPC.cs:25243-25245 — `else if (this.type == 179) { Lighting.AddLight(..., 0.3f, 0.15f, 0.05f); }`
        x_anchor = XAnchor::Center;
        y_anchor = XAnchor::Center;
        rgb = [0.3, 0.15, 0.05];
    } else {
        // The C# `else` arm (NPC.cs:25247-25250, `0.05f, 0.2f, 0.3f`) only fires for
        // other NPCs that share aiStyle 23. No other such NpcKind exists yet, so they
        // fall through to the shared `None` until their AI style is ported.
        return None;
    }

    Some((tile_x(pos, width, x_anchor), tile_y(pos, height, y_anchor), rgb))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::UnifiedRandom;

    #[test]
    fn desert_djinn_emits_light_at_top_tile() {
        // DesertDjinn at world position (160.0, 176.0) with size 24x40.
        // Top = (160, 176). Tile coords: 160/16=10, 176/16=11.
        // Colour: (0.6, 0.6, 0.3)
        let mut rng = UnifiedRandom::new(0);
        let (x, y, rgb) = npc_light(
            NpcKind::DesertDjinn,
            Vec2::new(160.0, 176.0),
            24.0,
            40.0,
            &mut rng,
        )
        .expect("DesertDjinn emits light");
        assert_eq!((x, y), (10, 11));
        assert_eq!(rgb, [0.6, 0.6, 0.3]);
    }

    #[test]
    fn desert_djinn_negative_position_truncates_before_divide() {
        // NPC.cs uses (int)(Top.X / 16) which for negative truncates toward zero.
        // Rust's `as i32 / 16` also truncates toward zero.
        // Top = (-20.0, -20.0). (-20 as i32) / 16 = -1.25 -> 0 in truncating division.
        let mut rng = UnifiedRandom::new(0);
        let (x, y, _) = npc_light(
            NpcKind::DesertDjinn,
            Vec2::new(-20.0, -20.0),
            24.0,
            40.0,
            &mut rng,
        )
        .expect("DesertDjinn emits light");
        // (int)(-20 / 16) = (int)(-1.25) = -1 in C# (truncates toward zero)
        // Wait - in C#, (int)(-20f / 16f) = -1 (truncates toward zero)
        // In Rust: (-20.0 as i32) / 16 = -1 (truncates toward zero)
        assert_eq!(x, -1);
        assert_eq!(y, -1);
    }

    #[test]
    fn prime_vice_emits_light_at_center_tile() {
        // NPC.cs:25239-25241 — PrimeVice (NPCID 83) uses the center-tile calculation
        // Lighting.AddLight((int)((position.X + width/2) / 16), (int)((position.Y + height/2) / 16), 0.2f, 0.05f, 0.3f)
        // Position (160, 160), size 40x40 -> center = (180, 180) -> tile (11, 11).
        // Colour: (0.2, 0.05, 0.3)
        let mut rng = UnifiedRandom::new(0);
        let (x, y, rgb) = npc_light(
            NpcKind::PrimeVice,
            Vec2::new(160.0, 160.0),
            40.0,
            40.0,
            &mut rng,
        )
        .expect("PrimeVice emits light");
        assert_eq!((x, y), (11, 11));
        assert_eq!(rgb, [0.2, 0.05, 0.3]);
    }

    #[test]
    fn prime_vice_negative_center_truncates_after_divide() {
        // Center anchor: (int)((pos + size/2) / 16) truncates the *quotient*.
        // Position (-20, -20), size 40x40 -> center = (0, 0) -> tile (0, 0).
        // This differs from CenterTruncated, which would truncate the sum first.
        let mut rng = UnifiedRandom::new(0);
        let (x, y, _) = npc_light(
            NpcKind::PrimeVice,
            Vec2::new(-20.0, -20.0),
            40.0,
            40.0,
            &mut rng,
        )
        .expect("PrimeVice emits light");
        assert_eq!((x, y), (0, 0));
    }

    #[test]
    fn dd2_kobold_flyer_t3_emits_light_at_center_tile() {
        // NPC.cs:25243-25245 — DD2KoboldFlyerT3 (NPCID 179) uses the center-tile
        // calculation with Lighting.AddLight(..., 0.3f, 0.15f, 0.05f).
        // Position (160, 160), size 32x32 -> center = (176, 176) -> tile (11, 11).
        // Colour: (0.3, 0.15, 0.05)
        let mut rng = UnifiedRandom::new(0);
        let (x, y, rgb) = npc_light(
            NpcKind::DD2KoboldFlyerT3,
            Vec2::new(160.0, 160.0),
            32.0,
            32.0,
            &mut rng,
        )
        .expect("DD2KoboldFlyerT3 emits light");
        assert_eq!((x, y), (11, 11));
        assert_eq!(rgb, [0.3, 0.15, 0.05]);
    }

    #[test]
    fn other_npcs_return_none() {
        let mut rng = UnifiedRandom::new(0);
        assert!(npc_light(NpcKind::Slime, Vec2::ZERO, 24.0, 18.0, &mut rng).is_none());
        assert!(npc_light(NpcKind::Zombie, Vec2::ZERO, 18.0, 40.0, &mut rng).is_none());
        // Non-aiStyle-23 kinds stay dark.
        assert!(npc_light(NpcKind::Guide, Vec2::ZERO, 18.0, 40.0, &mut rng).is_none());
    }
}