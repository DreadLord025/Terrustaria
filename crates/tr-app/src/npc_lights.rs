//! The NPC `Lighting.AddLight` sources (`S1-E-5`).
//!
//! C# `Lighting.AddLight` funnels every overload into
//! `Lighting.AddLight(int i, int j, float r, float g, float b)`
//! (`Lighting.cs:157-163`), which appends to the active engine's per-frame
//! list behind `!Main.gamePaused && Main.netMode != 2`
//! (`LightingEngine.cs:51-54`). The engine then max-merges that list onto the
//! working light map (`LightingEngine.cs:208-235`), one frame late. There is
//! ONE list pair per engine — [`crate::entity_lights::EntityLights`] — and
//! this family feeds the same pair as the item family
//! (`entity_lights.rs::item_light`); the tile-anchor vocabulary is shared
//! from there too.
//!
//! Ported here: the aiStyle-8 light tail at the end of `NPC.AI`
//! (`NPC.cs:21280-21373`) — the type chain after `base.position +=
//! netOffset`. Of the ported aiStyle-8 kinds:
//!
//! * 29/45 (Goblin Sorcerer/Tim) and 32 (Dark Caster) take **dust-only**
//!   arms (`:21280-21300`) — no `AddLight`, so they stay dark here;
//! * 533 (Desert Djinn) lights `base.Top` with (0.6, 0.6, 0.3)
//!   (`:21358-21361`);
//! * the `else` arm (`:21362-21373`) lights `base.Center` with
//!   `(0.6, 0.6*0.65, 0.6*0.4)` — the Fire Imp (24) is the port's only
//!   remaining aiStyle-8 kind.
//!
//! Arms that draw `Main.rand` are **not** ported — the 29/45/32 dust arms
//! and the `else` arm's `Next(6)` dust half belong to the NPC-visual dust
//! pass, which needs the dust system (S1-E-2/S1-E-3, other branches).
//! Skipping an arm wholesale leaves the `Main.rand` stream untouched, so
//! the fixed-step order stays exact. The aiStyle-23 arms
//! (`NPC.cs:25239-25250`, types 83 = CursedHammer / 179 = CrimsonAxe / the
//! fallback) wait for those kinds — neither exists in the port's roster
//! (`NPCID.cs:10843`/`:11035`; PrimeVice 130 rides aiStyle 34 and
//! DD2KoboldFlyerT3 575 aiStyle 108, so they never reach that chain).
//!
//! S1-D-3 W-1d adds the WoF Hungry's aiStyle-29 tail (`NPC.cs:26403`,
//! the (0.3, 0.2, 0.1) centre light) — the same RNG-free draw-side feed.

use glam::Vec2;

use crate::entity_lights::{tile_x, tile_y, XAnchor};
use crate::npc::NpcKind;

/// The aiStyle-8 light tail (`NPC.cs:21280-21373`): the `AddLight` source
/// this NPC contributes this frame, or `None`.
///
/// `pos` is the NPC's top-left **including `netOffset`** (the C# adds it at
/// `:21279` before the chain and removes it at `:21374`); `width`/`height`
/// are the hitbox sides. No arm draws RNG, so the feed's placement cannot
/// move any stream.
pub(crate) fn npc_light(
    kind: NpcKind,
    pos: Vec2,
    width: f32,
    height: f32,
) -> Option<(i32, i32, [f32; 3])> {
    let (anchor, rgb) = match kind {
        // `:21280-21300` — the sorcerer/Tim (29/45) and Dark Caster (32)
        // arms spawn dust only; no light.
        NpcKind::GoblinSorcerer | NpcKind::Tim | NpcKind::DarkCaster => return None,
        // `:21358-21361` — `Lighting.AddLight(base.Top, 0.6f, 0.6f, 0.3f)`.
        // `base.Top` is `(position.X + width/2f, position.Y)`
        // (`Entity.cs:86-91`) and the Vector2 overload truncates the
        // quotient per axis (`Lighting.cs:140-143`).
        NpcKind::DesertDjinn => (XAnchor::Top, [0.6, 0.6, 0.3]),
        // `:21362-21373` — the else arm: `num118 = 0.6f`,
        // `AddLight(base.Center, num118, num118 * 0.65f, num118 * 0.4f)`.
        // The `Main.rand.Next(6)` dust half is skipped (module docs). The
        // Fire Imp is the port's only other aiStyle-8 kind; the remaining
        // vanilla members (693/172/281-286) are not ported.
        NpcKind::FireImp => (XAnchor::Center, [0.6, 0.6 * 0.65, 0.6 * 0.4]),
        // S1-D-3 W-1d — the Hungry's aiStyle-29 light tail
        // (`NPC.cs:26403`): `Lighting.AddLight((int)(position.X +
        // (float)(width / 2)) / 16, (int)(position.Y + (float)(height /
        // 2)) / 16, 0.3f, 0.2f, 0.1f)`. The halves are INTEGER
        // (`width / 2` on the int hitbox), the centre is truncated to
        // int BEFORE the /16 — `XAnchor::CenterTruncated` (the Shine
        // precedent). For the Hungry's even 30×30 box the integer and
        // float halves agree; the anchor keeps the C# form for any
        // future odd-sized consumer.
        NpcKind::Hungry => (XAnchor::CenterTruncated, [0.3, 0.2, 0.1]),
        // W4-d33 wiring — the fairy critters' glow light
        // (`NPC.cs:48641`: `AddLight(Center, value3.ToVector3() * 0.7)`
        // — the per-type PRIMARY colour, `:48618-48630`: HotPink /
        // LimeGreen / RoyalBlue, ToVector3 = /255). RNG-free (the Lerp
        // NextFloat belongs to the dust half, which the sim's AI arm
        // rolls on the UpdateNPCs stream — the light feed reads only
        // the constant primary).
        NpcKind::FairyCritterPink => (
            XAnchor::Center,
            [255.0 / 255.0 * 0.7, 105.0 / 255.0 * 0.7, 180.0 / 255.0 * 0.7],
        ),
        NpcKind::FairyCritterGreen => (
            XAnchor::Center,
            [50.0 / 255.0 * 0.7, 205.0 / 255.0 * 0.7, 50.0 / 255.0 * 0.7],
        ),
        NpcKind::FairyCritterBlue => (
            XAnchor::Center,
            [65.0 / 255.0 * 0.7, 105.0 / 255.0 * 0.7, 225.0 / 255.0 * 0.7],
        ),
        _ => return None,
    };
    Some((
        tile_x(pos, width, anchor),
        tile_y(pos, height, anchor),
        rgb,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desert_djinn_emits_light_at_top_tile() {
        // `base.Top` = (position.X + width/2, position.Y): the Djinn's 28x62
        // box (`NPC.cs:14976-14977`) at (160, 176) has Top = (174, 176);
        // (int)(174/16) = 10, (int)(176/16) = 11. Colour (0.6, 0.6, 0.3).
        let (x, y, rgb) = npc_light(NpcKind::DesertDjinn, Vec2::new(160.0, 176.0), 28.0, 62.0)
            .expect("DesertDjinn emits light");
        assert_eq!((x, y), (10, 11));
        assert_eq!(rgb, [0.6, 0.6, 0.3]);
    }

    #[test]
    fn desert_djinn_top_uses_the_center_x_not_the_left_edge() {
        // The left-edge form would give tile 9 for x = 158 (158/16 = 9.875)
        // while Top's centre x = 158 + 14 = 172 lands on tile 10.
        let (x, _, _) = npc_light(NpcKind::DesertDjinn, Vec2::new(158.0, 176.0), 28.0, 62.0)
            .expect("DesertDjinn emits light");
        assert_eq!(x, 10);
    }

    #[test]
    fn desert_djinn_negative_position_truncates_the_quotient() {
        // `(int)(v / 16f)` truncates toward zero: Top = (-20 + 14, -20) =
        // (-6, -20); -6/16 = -0.375 -> 0, -20/16 = -1.25 -> -1.
        let (x, y, _) = npc_light(NpcKind::DesertDjinn, Vec2::new(-20.0, -20.0), 28.0, 62.0)
            .expect("DesertDjinn emits light");
        assert_eq!((x, y), (0, -1));
    }

    #[test]
    fn fire_imp_emits_the_else_arm_center_light() {
        // `:21364-21365` — the else arm's (0.6, 0.39, 0.24) at
        // `base.Center`. The imp's 18x40 box at (160, 176) has center
        // (169, 196) -> tile (10, 12).
        let (x, y, rgb) = npc_light(NpcKind::FireImp, Vec2::new(160.0, 176.0), 18.0, 40.0)
            .expect("FireImp emits the else arm's light");
        assert_eq!((x, y), (10, 12));
        assert_eq!(rgb, [0.6, 0.6 * 0.65, 0.6 * 0.4]);
    }

    /// S1-D-3 W-1d — the Hungry's aiStyle-29 tail (`NPC.cs:26403`):
    /// `AddLight((int)(position.X + width/2) / 16, (int)(position.Y +
    /// height/2) / 16, 0.3, 0.2, 0.1)` — the CenterTruncated anchor
    /// (the C#'s cast order; for the even 30×30 box the integer halves
    /// and the truncation commute with the /16, so the placement pins
    /// the colour + the centre tile).
    #[test]
    fn the_hungry_emits_the_style29_tail_light() {
        let (x, y, rgb) = npc_light(NpcKind::Hungry, Vec2::new(160.0, 176.0), 30.0, 30.0)
            .expect("the Hungry lights its centre");
        // Centre (175, 191) → tiles (10, 11).
        assert_eq!((x, y), (10, 11));
        assert_eq!(rgb, [0.3, 0.2, 0.1]);
        // A fractional centre: 240.9 + 15 = 255.9 → (int)255 → tile 15.
        let (x2, _, _) = npc_light(NpcKind::Hungry, Vec2::new(240.9, 0.0), 30.0, 30.0)
            .expect("the Hungry lights");
        assert_eq!(x2, 255 / 16, "(int)centre then /16");
    }

    #[test]
    fn the_dust_only_casters_stay_dark() {
        // `:21280-21300` — 29/45/32 spawn dust, never light.
        for kind in [NpcKind::GoblinSorcerer, NpcKind::Tim, NpcKind::DarkCaster] {
            assert!(npc_light(kind, Vec2::ZERO, 18.0, 40.0).is_none());
        }
    }

    #[test]
    fn other_npcs_return_none() {
        assert!(npc_light(NpcKind::Slime, Vec2::ZERO, 24.0, 18.0).is_none());
        assert!(npc_light(NpcKind::Zombie, Vec2::ZERO, 18.0, 40.0).is_none());
        assert!(npc_light(NpcKind::Guide, Vec2::ZERO, 18.0, 40.0).is_none());
        // The aiStyle-23 chain's kinds (83 CursedHammer / 179 CrimsonAxe)
        // are not in the roster; PrimeVice/DD2KoboldFlyerT3 ride other AI
        // styles and never reach it.
        assert!(npc_light(NpcKind::PrimeVice, Vec2::ZERO, 52.0, 52.0).is_none());
        assert!(npc_light(NpcKind::DD2KoboldFlyerT3, Vec2::ZERO, 38.0, 38.0).is_none());
    }
}
