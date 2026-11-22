//! The entity `Lighting.AddLight` sources (`S1-E-5`).
//!
//! C# `Lighting.AddLight` funnels every overload into
//! `Lighting.AddLight(int i, int j, float r, float g, float b)`
//! (`Lighting.cs:157-163`), which appends to the active engine's per-frame list
//! behind `!Main.gamePaused && Main.netMode != 2` (`LightingEngine.cs:51-54`).
//! The engine then max-merges that list onto the working light map
//! (`LightingEngine.cs:208-235`), one frame late. [`EntityLights`] is that pair
//! of lists.
//!
//! Ported here: `WorldItem.UpdateItem_VisualEffects`'s RNG-free `AddLight`
//! arms (`WorldItem.cs:1295-1648`). The `Main.rand`-drawing flicker arms and
//! the `Main.essScale` fragment arms (`:1206-1293`) live in
//! [`crate::item_lights`] and feed the same list from the fixed step (their
//! draws ride the `UpdateItems` stream); the dust-gated arms (116, 501 and
//! the `% 12` halves of 75/`BossBag`) landed in [`crate::item_dust`] (W4-g15)
//! — their light halves stay here / in `item_lights`, the dust halves there.
//!
//! The NPC family (`npc_lights.rs::npc_light`, `NPC.AI`'s aiStyle-8 light
//! tail) and the Player family (`player_lights.rs::player_lights`, the
//! buff/accessory/held-item arms) feed the **same** [`EntityLights`] pair —
//! there is one list pair per engine in the C#, not one per source family.
//! All feeds run in the frame pass (`app/frame/draw.rs`) and share the
//! tile-anchor helpers below.

use glam::Vec2;
use tr_core::lighting::{DynamicLight, LightParams};
use tr_core::TILE_PX;

use crate::itemdata::ItemDef;

/// `ItemID.Sets.Torches` (`ItemID.cs:1371`).
pub(crate) const TORCHES: &[u16] = &[
    8, 427, 3004, 523, 433, 429, 974, 1333, 1245, 3114, 430, 3045, 428, 2274, 431, 432, 4383, 4384,
    4385, 4386, 4387, 4388, 5293, 5353,
];

/// `ItemID.Sets.WaterTorches` (`ItemID.cs:1372`).
pub(crate) const WATER_TORCHES: &[u16] = &[523, 1333, 4384];

/// `ItemID.Sets.BossBag` (`ItemID.cs:1383`).
pub(crate) const BOSS_BAG: &[u16] = &[
    3318, 3319, 3320, 3321, 3322, 3323, 3324, 3325, 3326, 3327, 3328, 3329, 3330, 3331, 3332, 3860,
    3861, 3862, 4782, 4957, 5111,
];

/// `Item.createTile` for a torch (`TileID.Torches = 4`).
const TILE_TORCHES: u16 = 4;

/// The per-frame `AddLight` list pair (`LightingEngine.cs:36-38`,
/// `_perFrameLights` / `_oldPerFrameLights`).
///
/// Vanilla applies the **previous** frame's list because `LightTiles` runs
/// before the update that refills the current one; [`Self::end_frame`] is that
/// swap (`LightingEngine.cs:228-233`).
#[derive(Default)]
pub(crate) struct EntityLights {
    current: Vec<DynamicLight>,
    previous: Vec<DynamicLight>,
}

impl EntityLights {
    /// `ILightingEngine.AddLight` (`LightingEngine.cs:51-54`): append one
    /// source. The gate (`Lighting.cs:159`) is the caller's.
    pub(crate) fn add(&mut self, x: i32, y: i32, rgb: [f32; 3]) {
        self.current.push(DynamicLight { x, y, rgb });
    }

    /// The list the light pass consumes: the previous frame's sources
    /// (`LightingEngine.cs:210-214`).
    pub(crate) fn visible(&self) -> &[DynamicLight] {
        &self.previous
    }

    /// `Utils.Swap` + `Clear` + the stamp (`LightingEngine.cs:228-233`).
    pub(crate) fn end_frame(&mut self) {
        std::mem::swap(&mut self.current, &mut self.previous);
        self.current.clear();
    }

    /// The `lights` half of the light-map cache key (`LightKey::lights`): every
    /// cell and colour of the list the pass will read. Two frames with the same
    /// key produce the same map.
    pub(crate) fn key(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for l in &self.previous {
            for v in [l.x as u32, l.y as u32, l.rgb[0].to_bits(), l.rgb[1].to_bits(), l.rgb[2].to_bits()] {
                for b in v.to_le_bytes() {
                    h = (h ^ b as u64).wrapping_mul(0x0100_0000_01b3);
                }
            }
        }
        h
    }
}

/// Where the C# reads the light's tile x from. The forms are **not**
/// interchangeable for negative coordinates (`(int)(v/16)` truncates the
/// quotient, `(int)v / 16` truncates first). Shared with the NPC family
/// (`npc_lights.rs`) — one anchor vocabulary for every `AddLight` source.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum XAnchor {
    /// `(int)((position.X + width / 2f) / 16f)` — the common arm.
    Center,
    /// `(int)((position.X + width) / 16f)` — the right edge.
    Right,
    /// `(int)Center.X / 16` — truncate before dividing.
    CenterTruncated,
    /// `base.Top` (`Entity.cs:86-91`): `(position.X + width / 2f,
    /// position.Y)`, then the `Lighting.AddLight(Vector2, ...)` overload's
    /// `(int)(v / 16f)` per axis (`Lighting.cs:140-143`) — centre x, top y,
    /// truncate the quotient.
    Top,
}

#[inline]
pub(crate) fn tile_x(pos: Vec2, size: f32, anchor: XAnchor) -> i32 {
    match anchor {
        XAnchor::Center | XAnchor::Top => ((pos.x + size / 2.0) / TILE_PX) as i32,
        XAnchor::Right => ((pos.x + size) / TILE_PX) as i32,
        XAnchor::CenterTruncated => (pos.x + size / 2.0) as i32 / 16,
    }
}

#[inline]
pub(crate) fn tile_y(pos: Vec2, size: f32, anchor: XAnchor) -> i32 {
    match anchor {
        XAnchor::CenterTruncated => (pos.y + size / 2.0) as i32 / 16,
        XAnchor::Top => (pos.y / TILE_PX) as i32,
        _ => ((pos.y + size / 2.0) / TILE_PX) as i32,
    }
}

/// `Player.UpdateBuffs`' Shine arm (`Player.cs:9993-9996`, S1-E-7 wave A):
/// `Lighting.AddLight((int)(position.X + width / 2) / 16, (int)(position.Y +
/// height / 2) / 16, 0.8f, 0.95f, 1f)` while buff 11 is held. The C# truncates
/// the centre coordinate to int BEFORE the /16 (`XAnchor::CenterTruncated`);
/// `Player.Center` is `position + size/2` per axis.
pub(crate) fn shine_light(p: &crate::player::Player) -> Option<(i32, i32, [f32; 3])> {
    if p.buffs.find_buff_index(11).is_none() {
        return None;
    }
    let (cx, cy) = p.center();
    Some((cx as i32 / 16, cy as i32 / 16, [0.8, 0.95, 1.0]))
}

/// `WorldItem.UpdateItem_VisualEffects` (`WorldItem.cs:1204-1649`): the
/// `AddLight` source this drop contributes this tick, or `None`.
///
/// `pos` is the drop's top-left and `size` its box side (the port's items are
/// uniformly `SIZE`×`SIZE`, `item.rs:151`; C# reads `width`/`height`, equal for
/// a dropped item). `wet` is `base.wet` (`WorldItem.cs:881`).
pub(crate) fn item_light(
    def: &ItemDef,
    pos: Vec2,
    size: f32,
    wet: bool,
    p: &LightParams,
) -> Option<(i32, i32, [f32; 3])> {
    let kind = def.id;
    let mut x = XAnchor::Center;
    let mut y = XAnchor::Center;
    let rgb: [f32; 3];

    // The arm order is `WorldItem.cs:1295-1648`, verbatim; the rand/essScale
    // head (`:1206-1293`) is `crate::item_lights::rand_light`, fed from the
    // fixed step so its draws ride the `UpdateItems` stream.
    if kind == 3822 {
        // `:1306` — `AddLight(base.Center, 0.1f, 0.3f, 0.1f)`.
        rgb = [0.1, 0.3, 0.1];
    } else if kind == 1970 {
        rgb = [0.75, 0.0, 0.75]; // `:1310`
    } else if kind == 1972 {
        rgb = [0.0, 0.0, 0.75]; // `:1314`
    } else if kind == 1971 {
        rgb = [0.75, 0.75, 0.0]; // `:1318`
    } else if kind == 1973 {
        rgb = [0.0, 0.75, 0.0]; // `:1322`
    } else if kind == 1974 {
        rgb = [0.75, 0.0, 0.0]; // `:1326`
    } else if kind == 1975 {
        rgb = [0.75, 0.75, 0.75]; // `:1330`
    } else if kind == 1976 {
        rgb = [0.75, 0.375, 0.0]; // `:1334`
    } else if kind == 2679 {
        rgb = [0.6, 0.0, 0.6]; // `:1338`
    } else if kind == 2687 {
        rgb = [0.0, 0.0, 0.6]; // `:1342`
    } else if kind == 2689 {
        rgb = [0.6, 0.6, 0.0]; // `:1346`
    } else if kind == 2683 {
        rgb = [0.0, 0.6, 0.0]; // `:1350`
    } else if kind == 2685 {
        rgb = [0.6, 0.0, 0.0]; // `:1354`
    } else if kind == 2681 {
        rgb = [0.6, 0.6, 0.6]; // `:1358`
    } else if kind == 2677 {
        rgb = [0.6, 0.375, 0.0]; // `:1362`
    } else if kind == 105 {
        // `:1364-1370` — the plain torch, only while dry.
        if wet {
            return None;
        }
        rgb = [1.0, 0.95, 0.8];
    } else if kind == 2701 {
        rgb = [0.7, 0.65, 0.55]; // `:1373`
    } else if def.tile == Some(TILE_TORCHES) {
        // `:1375-1382` — `(!wet && Torches[type]) || WaterTorches[type]`,
        // coloured by `Lighting.AddLight(base.Center, placeStyle)`
        // (`Lighting.cs:151-155`): `TorchID.TorchColor(placeStyle)` at full
        // amount, positioned by `(int)Center.X / 16` — truncate, then divide.
        let torch = TORCHES.contains(&kind);
        let water = WATER_TORCHES.contains(&kind);
        if !((!wet && torch) || water) {
            return None;
        }
        x = XAnchor::CenterTruncated;
        y = XAnchor::CenterTruncated;
        rgb = tr_core::tile_light::torch_color(def.place_style, p);
    } else if kind == 3114 {
        if wet {
            return None; // `:1383-1389`
        }
        rgb = [1.0, 0.0, 1.0];
    } else if kind == 1245 {
        if wet {
            return None; // `:1390-1396`
        }
        rgb = [1.0, 0.5, 0.0];
    } else if kind == 433 {
        if wet {
            return None; // `:1397-1403`
        }
        // `0.5*demonTorch + 1*(1-demonTorch)`, `0.3`,
        // `1*demonTorch + 0.5*(1-demonTorch)`.
        let d = p.demon_torch;
        rgb = [0.5 * d + (1.0 - d), 0.3, d + 0.5 * (1.0 - d)];
    } else if kind == 523 {
        rgb = [0.85, 1.2, 0.7]; // `:1406`
    } else if kind == 974 {
        if wet {
            return None; // `:1408-1414`
        }
        rgb = [0.75, 0.85, 1.4];
    } else if kind == 1333 {
        rgb = [1.25, 1.25, 0.7]; // `:1417`
    } else if kind == 4383 {
        if wet {
            return None; // `:1419-1425`
        }
        rgb = [1.4, 0.85, 0.55];
    } else if kind == 5293 {
        if wet {
            return None; // `:1426-1432`
        }
        rgb = [0.25, 0.65, 1.0];
    } else if kind == 5353 {
        if wet {
            return None; // `:1433-1439`
        }
        rgb = [0.81, 0.72, 1.0];
    } else if kind == 4384 {
        rgb = [0.25, 1.3, 0.8]; // `:1442`
    } else if kind == 3045 {
        // `:1446` — the disco torch.
        rgb = [
            p.disco[0] as f32 / 255.0,
            p.disco[1] as f32 / 255.0,
            p.disco[2] as f32 / 255.0,
        ];
    } else if kind == 3004 {
        rgb = [0.95, 0.65, 1.3]; // `:1450`
    } else if kind == 2274 {
        if wet {
            return None; // `:1452-1461`
        }
        rgb = [0.75, 1.3499999, 1.5];
    } else if (427..=432).contains(&kind) {
        if wet {
            return None; // `:1462-1508`
        }
        // `num16 = type - 426`, then the six fixed rows.
        rgb = match kind - 426 {
            1 => [0.1, 0.2, 1.1],
            2 => [1.0, 0.1, 0.1],
            3 => [0.0, 1.0, 0.1],
            4 => [0.9, 0.0, 0.9],
            5 => [1.3, 1.3, 1.3],
            6 => [0.9, 0.9, 0.0],
            _ => return None,
        };
    } else if [2777, 2778, 2779, 2780, 2781, 2760, 2761, 2762, 3524].contains(&kind) {
        rgb = [0.4, 0.16, 0.36]; // `:1511`
    } else if [2772, 2773, 2774, 2775, 2776, 2757, 2758, 2759, 3523].contains(&kind) {
        rgb = [0.0, 0.36, 0.4]; // `:1515`
    } else if [2782, 2783, 2784, 2785, 2786, 2763, 2764, 2765, 3522].contains(&kind) {
        rgb = [0.5, 0.25, 0.05]; // `:1519`
    } else if [3462, 3463, 3464, 3465, 3466, 3381, 3382, 3383, 3525].contains(&kind) {
        rgb = [0.3, 0.3, 0.2]; // `:1523`
    } else if kind == 41 {
        if wet {
            return None; // `:1525-1531`
        }
        x = XAnchor::Right;
        rgb = [1.0, 0.75, 0.55];
    } else if kind == 988 {
        if wet {
            return None; // `:1532-1538`
        }
        x = XAnchor::Right;
        rgb = [0.35, 0.65, 1.0];
    } else if kind == 1326 {
        x = XAnchor::CenterTruncated; // `:1541` — `(int)Center.X / 16`
        y = XAnchor::CenterTruncated;
        rgb = [1.0, 0.1, 0.8];
    } else if kind == 5335 {
        x = XAnchor::CenterTruncated; // `:1545`
        y = XAnchor::CenterTruncated;
        rgb = [0.85, 0.1, 0.8];
    } else if (5140..=5146).contains(&kind) {
        // `:1547-1591` — the seven biome-torch multipliers on a white base.
        let (mut r, mut g, mut b) = (1.0f32, 1.0f32, 1.0f32);
        match kind {
            5140 => {
                r *= 0.9;
                g *= 0.8;
                b *= 0.1;
            }
            5141 => {
                r *= 0.25;
                g *= 0.1;
                b *= 0.0;
            }
            5142 => {
                r *= 0.0;
                g *= 0.25;
                b *= 0.0;
            }
            5143 => {
                r *= 0.0;
                g *= 0.16;
                b *= 0.34;
            }
            5144 => {
                r *= 0.3;
                g *= 0.0;
                b *= 0.17;
            }
            5145 => {
                r *= 0.3;
                g *= 0.0;
                b *= 0.35;
            }
            _ => {
                r *= p.disco[0] as f32 / 255.0;
                g *= p.disco[1] as f32 / 255.0;
                b *= p.disco[2] as f32 / 255.0;
            }
        }
        x = XAnchor::Right;
        rgb = [r, g, b];
    } else if kind == 282 {
        x = XAnchor::Right;
        rgb = [0.7, 1.0, 0.8]; // `:1594`
    } else if kind == 286 {
        x = XAnchor::Right;
        rgb = [0.7, 0.8, 1.0]; // `:1598`
    } else if kind == 3112 {
        x = XAnchor::Right;
        rgb = [1.0, 0.6, 0.85]; // `:1602`
    } else if kind == 4776 {
        x = XAnchor::Right;
        rgb = [0.7, 0.0, 1.0]; // `:1606`
    } else if kind == 3002 {
        x = XAnchor::Right;
        rgb = [1.05, 0.95, 0.55]; // `:1610`
    } else if kind == 5643 {
        x = XAnchor::Right;
        rgb = [
            p.disco[0] as f32 / 255.0, // `:1614`
            p.disco[1] as f32 / 255.0,
            p.disco[2] as f32 / 255.0,
        ];
    } else if kind == 331 {
        x = XAnchor::Right;
        rgb = [0.55, 0.75, 0.6]; // `:1618`
    } else if kind == 183 {
        x = XAnchor::Right;
        rgb = [0.15, 0.45, 0.9]; // `:1622`
    } else if kind == 75 {
        // `:1626` — the light half; the `timeSinceItemSpawned % 12` dust half
        // rides `item_dust::fallen_star_sparkle` (W4-g15).
        x = XAnchor::Right;
        rgb = [0.8, 0.7, 0.1];
    } else if BOSS_BAG.contains(&kind) {
        // `:1638` — the light half; the dust half rides
        // `item_dust::boss_bag_sparkle` (W4-g15).
        x = XAnchor::Right;
        rgb = [0.4, 0.4, 0.4];
    } else {
        return None;
    }

    Some((tile_x(pos, size, x), tile_y(pos, size, y), rgb))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The real generated row for `id` — the test must run against the shipped
    /// `createTile`/`placeStyle` values, not a hand-built stand-in.
    fn def(id: u16) -> ItemDef {
        *crate::registry::get(id).expect("id is in the item table")
    }

    fn params() -> LightParams {
        let mut p = LightParams::default();
        p.demon_torch = 1.0;
        p.disco = [255, 0, 0];
        p
    }

    #[test]
    fn s1_e_7a_shine_lights_the_players_center_tile() {
        // `Player.cs:9993-9996`: the truncating `/16` anchor — centre
        // (170.9, 661.0) → (170/16, 661/16) = (10, 41), NOT (10, 41) via the
        // quotient form (170.9/16 = 10.68 → 10 the same, but 661/16 = 41.3 →
        // 41 vs (int)661/16 = 41 — the negative/edge case differs; the
        // truncation is pinned by construction).
        let w = tr_core::World::new("t".into(), 64, 800);
        let mut p = crate::player::Player::spawn(&w);
        assert!(shine_light(&p).is_none(), "no buff, no light");
        p.buffs.add_buff(11, 600);
        p.x = 160.9;
        p.y = 640.0;
        let (x, y, rgb) = shine_light(&p).expect("Shine emits");
        let (cx, cy) = p.center();
        assert_eq!((x, y), (cx as i32 / 16, cy as i32 / 16));
        assert_eq!(rgb, [0.8, 0.95, 1.0]);
    }

    #[test]
    fn a_dry_torch_item_lights_its_center_tile() {
        // Id 1970 at world (160, 176) with the 14 px box: center x = 167,
        // 167 / 16 = 10.4 -> tile 10; center y = 183, / 16 = 11.4 -> tile 11.
        let (x, y, rgb) = item_light(&def(1970), Vec2::new(160.0, 176.0), 14.0, false, &params())
            .expect("a dry 1970 emits light");
        assert_eq!((x, y), (10, 11));
        assert_eq!(rgb, [0.75, 0.0, 0.75]);
    }

    #[test]
    fn a_wet_torch_item_goes_dark_while_a_water_torch_stays_lit() {
        let p = params();
        // 105 is gated on `!base.wet` (`WorldItem.cs:1366`).
        assert!(item_light(&def(105), Vec2::ZERO, 14.0, true, &p).is_none());

        // 523 (Cursed Torch) also has a `type == 523` arm at `:1404`, but it is
        // **unreachable**: `createTile == 4` matches first at `:1375`, so the
        // item takes the torch-colour path. `WaterTorches` (`ItemID.cs:1372`)
        // skips the wet gate there, so a wet 523 still lights.
        let water = def(523);
        assert_eq!(water.tile, Some(TILE_TORCHES));
        let lit = item_light(&water, Vec2::ZERO, 14.0, true, &p).expect("water torch stays lit");
        // `placeStyle` 8 -> `TORCH_LIGHT[8]` = `[1.0, 1.6, 0.5]`.
        assert_eq!(lit.2, [1.0, 1.6, 0.5]);
    }

    #[test]
    fn a_plain_torch_in_water_goes_dark() {
        // 8 is a `Torches` row but not a `WaterTorches` one, so `:1378`'s
        // `(!wet && Torches) || WaterTorches` fails under water.
        assert_eq!(def(8).tile, Some(TILE_TORCHES));
        assert!(item_light(&def(8), Vec2::ZERO, 14.0, true, &params()).is_none());
    }

    #[test]
    fn the_demon_torch_ramp_blends_both_ends() {
        // 433 has its own `type == 433` arm at `:1397-1403`, but it is
        // **shadowed** by the `createTile == 4` arm at `:1375` — the item lights
        // through `TorchID.TorchColor(placeStyle 7)`. The two formulas are the
        // same expression (`TorchID.cs:165`'s demon row), so the ramp shows
        // either way; this pins the value.
        let mut p = params();
        p.demon_torch = 1.0;
        assert_eq!(
            item_light(&def(433), Vec2::ZERO, 14.0, false, &p).unwrap().2,
            [0.5, 0.3, 1.0]
        );
        p.demon_torch = 0.0;
        assert_eq!(
            item_light(&def(433), Vec2::ZERO, 14.0, false, &p).unwrap().2,
            [1.0, 0.3, 0.5]
        );
    }

    #[test]
    fn a_placeable_torch_takes_the_torch_arm_not_its_own_id_arm() {
        // Every placeable torch (`createTile == 4`) is caught by `:1375` before
        // its own `type == N` arm, so the `427..=432` colours at `:1470-1504`
        // never run for them. 427 (Blue Torch) pins the difference: its own arm
        // would give `[0.1, 0.2, 1.1]`, the torch arm gives
        // `TORCH_LIGHT[placeStyle 1]` = `[0.0, 0.1, 1.3]`.
        let blue = def(427);
        assert_eq!(blue.tile, Some(TILE_TORCHES));
        let got = item_light(&blue, Vec2::ZERO, 14.0, false, &params()).unwrap().2;
        assert_eq!(got, tr_core::tile_light::TORCH_LIGHT[1]);
        assert_ne!(got, [0.1, 0.2, 1.1], "not the shadowed 427..=432 arm");
    }

    #[test]
    fn the_torch_colour_table_scales_by_place_style() {
        // `WorldItem.cs:1380` passes the item's `placeStyle` to
        // `TorchID.TorchColor`; style 0 is the plain 1/0.95/0.8 torch.
        let mut torch = def(8);
        torch.tile = Some(TILE_TORCHES);
        let got = item_light(&torch, Vec2::ZERO, 14.0, false, &params()).unwrap().2;
        assert_eq!(got, [1.0, 0.95, 0.8]);
        // A non-torch `createTile == 4` item is dark: 784 has `tile: Some(4)`
        // but is absent from `ItemID.Sets.Torches` (`ItemID.cs:1371`).
        assert!(item_light(&def(784), Vec2::ZERO, 14.0, false, &params()).is_none());
    }

    #[test]
    fn the_right_edge_anchor_truncates_before_dividing() {
        // `:1541` uses `(int)Center.X / 16`, which differs from
        // `(int)(Center.X / 16)` for negative pixel positions.
        let (x, _, _) = item_light(&def(1326), Vec2::new(-20.0, -20.0), 14.0, false, &params())
            .expect("1326 lights");
        // center = -13.0; `(int)-13 / 16` = -13 / 16 = 0 in Rust (truncating).
        assert_eq!(x, 0);
        // The `(int)(v / 16)` form would floor to -1 instead.
        let (x2, _, _) = item_light(&def(1970), Vec2::new(-20.0, -20.0), 14.0, false, &params())
            .expect("1970 lights");
        assert_eq!(x2, 0);
    }

    #[test]
    fn the_per_frame_list_is_one_frame_late() {
        let mut lights = EntityLights::default();
        lights.add(1, 2, [0.5, 0.5, 0.5]);
        assert!(lights.visible().is_empty(), "the first frame has no sources");
        lights.end_frame();
        assert_eq!(lights.visible().len(), 1);
        assert_eq!(lights.visible()[0].x, 1);
        lights.add(3, 4, [0.1, 0.2, 0.3]);
        assert_eq!(lights.visible().len(), 1, "the new source waits a frame");
        let before = lights.key();
        lights.end_frame();
        // `end_frame` swaps then clears `current` (`LightingEngine.cs:228-233`),
        // so `previous` now holds exactly the one source added during the frame
        // just ended — the earlier source is gone, not accumulated.
        assert_eq!(lights.visible().len(), 1);
        assert_eq!(lights.visible()[0].x, 3);
        assert_ne!(lights.key(), before, "the key follows the visible list");
    }
}
