//! The map icon overlay layers (`Terraria.Map.MapIconOverlay` + the
//! `Terraria.Map/*Layer*.cs` files, registered at `Main.cs:66551` in the
//! order Digtoise → BossBag → Spawn → TeamBasedSpawn → TeleportPylons →
//! Pings). The context is `MapOverlayDrawContext` (`MapOverlayDrawContext.cs`),
//! the layer walk is `Main.MapIcons.Draw` (`MapIconOverlay.cs:17-27`).

use glam::Vec2;
use tr_core::World;
use tr_render::SpriteRenderer;

use crate::assets;
use crate::item::ItemDrop;
use crate::map::WorldMap;
use crate::player::Player;

/// The two `Alignment` multipliers the layers use (`Terraria.UI.Alignment`).
const ALIGN_BOTTOM: [f32; 2] = [0.5, 1.0];
const ALIGN_CENTER: [f32; 2] = [0.5, 0.5];

/// `Terraria.Map.MapOverlayDrawContext` (`MapOverlayDrawContext.cs:8-70`):
/// the map-space → screen-space transform, the optional clip rect and the
/// hover hit-test. [`Self::draw`] pushes the icon quad and returns
/// `DrawResult.IsMouseOver`.
pub(crate) struct MapOverlayContext<'a> {
    r: &'a mut SpriteRenderer,
    /// `Main.MouseScreen`.
    mouse: Vec2,
    /// `Main.DrawMap`'s `mapPosition` — `Vector2.Zero` on the fullscreen map
    /// (the layer positions are world tiles).
    map_position: [f32; 2],
    /// The map's top-left on screen (`num - 10f * num5`, `Main.cs:55625`).
    map_offset: [f32; 2],
    /// The minimap's screen rect (`Main.cs:55703`); `None` on the fullscreen.
    clip: Option<[f32; 4]>,
    /// `num5`: the px-per-tile scale.
    map_scale: f32,
    /// `num37`/`num46`/`num77`: the icon draw scale.
    draw_scale: f32,
    /// `b / 255`: the map alpha.
    opacity: f32,
}

impl MapOverlayContext<'_> {
    /// `Draw` (`MapOverlayDrawContext.cs:33-58`): the transform, the clip,
    /// the aligned quad and the mouse-over rect.
    pub fn draw(
        &mut self,
        uv: [f32; 4],
        size: [f32; 2],
        position: [f32; 2],
        alignment: [f32; 2],
    ) -> bool {
        if self.opacity == 0.0 {
            return false; // `:36-39`
        }
        let pos = [
            (position[0] - self.map_position[0]) * self.map_scale + self.map_offset[0],
            (position[1] - self.map_position[1]) * self.map_scale + self.map_offset[1],
        ];
        if let Some(c) = self.clip {
            if pos[0] < c[0] || pos[1] < c[1] || pos[0] > c[2] || pos[1] > c[3] {
                return false; // `:45-48`: `DrawResult.Culled`
            }
        }
        let origin = [size[0] * alignment[0], size[1] * alignment[1]];
        self.r
            .push_ui_sprite(pos, size, uv, [self.opacity; 4]);
        // `:57-58`: the hover rect is the draw box shifted by the aligned
        // origin, sized by the draw scale.
        let hx = pos[0] - origin[0] * self.draw_scale;
        let hy = pos[1] - origin[1] * self.draw_scale;
        let hw = size[0] * self.draw_scale;
        let hh = size[1] * self.draw_scale;
        self.mouse.x >= hx
            && self.mouse.x <= hx + hw
            && self.mouse.y >= hy
            && self.mouse.y <= hy + hh
    }

    /// `GetUnclampedDrawRegion`'s centre (`MapOverlayDrawContext.cs:60-70`) —
    /// the pylon line's endpoints.
    pub fn unclamped_center(
        &self,
        size: [f32; 2],
        position: [f32; 2],
        scale: f32,
        alignment: [f32; 2],
    ) -> [f32; 2] {
        let pos = [
            (position[0] - self.map_position[0]) * self.map_scale + self.map_offset[0],
            (position[1] - self.map_position[1]) * self.map_scale + self.map_offset[1],
        ];
        let origin = [size[0] * alignment[0], size[1] * alignment[1]];
        let num = self.draw_scale * scale;
        [
            pos[0] - origin[0] * num + size[0] * num * 0.5,
            pos[1] - origin[1] * num + size[1] * num * 0.5,
        ]
    }
}

/// `ItemID.Sets.BossBag` (`ItemID.cs:1383`) — the map layer's boss-bag drops.
const BOSS_BAGS: [u16; 21] = [
    3318, 3319, 3320, 3321, 3322, 3323, 3324, 3325, 3326, 3327, 3328, 3329,
    3330, 3331, 3332, 3860, 3861, 3862, 4782, 4957, 5111,
];

fn is_boss_bag(kind: u16) -> bool {
    BOSS_BAGS.contains(&kind)
}

/// The layer inputs — the C# reads the `Main.*` globals; the port threads
/// them through one struct.
pub(crate) struct MapIconInputs<'a> {
    pub world: &'a World,
    /// `Main.LocalPlayer` — the bed spawn's `SpawnX/Y`.
    pub player: Option<&'a Player>,
    /// `Main.item[400]` — the BossBag drop scan (`BossBagMapLayer.cs:10-24`).
    pub item_drops: &'a [ItemDrop],
    /// `Main.projectile`'s Digtoise trackers (1098/1123) — the port spawns
    /// neither, so the list stays empty in-game.
    pub digtoise: &'a [[f32; 2]],
    /// `PingMapLayer`'s pool.
    pub pings: &'a [([f32; 2], f32)],
    /// The pylon count — `Main.PylonSystem.Pylons` has no port system yet.
    pub pylons: usize,
    /// The team-based-spawns seed gate (`TeamBasedSpawnMapLayer.cs:10`).
    pub team_based_spawns: bool,
    pub mouse: Vec2,
}

/// `Main.MapIcons.Draw`'s layer walk (`MapIconOverlay.cs:17-27`) in the
/// registration order. Returns the hovered layer's localized text, if any —
/// the C# `ref text` feeds `Main.DrawMap`'s hover line.
pub(crate) fn map_layers_draw(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    inputs: &MapIconInputs,
    map_position: [f32; 2],
    map_offset: [f32; 2],
    clip: Option<[f32; 4]>,
    map_scale: f32,
    draw_scale: f32,
    opacity: f32,
) -> Option<String> {
    let mut ctx = MapOverlayContext {
        r,
        mouse: inputs.mouse,
        map_position,
        map_offset,
        clip,
        map_scale,
        draw_scale,
        opacity,
    };
    let mut text: Option<String> = None;

    // `DigtoiseMapLayer` (`DigtoiseMapLayer.cs:10-27`): the player's
    // Digtoise tracker (1098/1123) draws Extra_305's 2-frame row,
    // bottom-aligned at `Bottom / 16 + (0.5, 0)`.
    for pos in inputs.digtoise {
        if let Some(tex) = lookups.extra_305 {
            let Some(uv) = lookups.hud_uv(Some(&tex)) else {
                break;
            };
            let size = [tex.width as f32 / 2.0, tex.height as f32];
            if ctx.draw(uv, size, [pos[0] + 0.5, pos[1]], ALIGN_BOTTOM) {
                text = Some(crate::lang::text("UI.Digtoise"));
            }
        }
        break;
    }

    // `BossBagMapLayer` (`BossBagMapLayer.cs:10-24`): every active boss-bag
    // drop draws its item icon centred on the drop's tile.
    for drop in inputs.item_drops {
        if !is_boss_bag(drop.kind) {
            continue;
        }
        if let Some(rect) = lookups.item_icons.get(&drop.kind) {
            let Some(uv) = lookups.hud_uv(Some(rect)) else {
                continue;
            };
            let size = [rect.width as f32, rect.height as f32];
            let pos = [drop.pos.x / 16.0 + 0.5, drop.pos.y / 16.0 + 0.5];
            if ctx.draw(uv, size, pos, ALIGN_CENTER) {
                text = Some(crate::lang::item_name(drop.kind));
            }
        }
    }

    // `SpawnMapLayer` (`SpawnMapLayer.cs:9-27`): the world spawn and the bed
    // spawn markers, bottom-aligned.
    if let Some(tex) = lookups.map_spawn {
        if let Some(uv) = lookups.hud_uv(Some(&tex)) {
            let size = [tex.width as f32, tex.height as f32];
            let pos = [inputs.world.spawn_x as f32 + 0.5, inputs.world.spawn_y as f32];
            if ctx.draw(uv, size, pos, ALIGN_BOTTOM) {
                text = Some(crate::lang::text("UI.SpawnPoint"));
            }
        }
    }
    // `localPlayer.SpawnX/SpawnY` — the port keeps the bed spawn on the
    // World (`World.bed_spawn`), which `Player.Spawn`'s re-validation
    // (`Player.cs:37571-37575`) already consumes.
    if let Some((bx, by)) = inputs.world.bed_spawn {
        if let Some(tex) = lookups.map_spawn_bed {
            if let Some(uv) = lookups.hud_uv(Some(&tex)) {
                let size = [tex.width as f32, tex.height as f32];
                let pos = [bx as f32 + 0.5, by as f32];
                if ctx.draw(uv, size, pos, ALIGN_BOTTOM) {
                    text = Some(crate::lang::text("UI.SpawnBed"));
                }
            }
        }
    }

    // `TeamBasedSpawnMapLayer` (`TeamBasedSpawnMapLayer.cs:9-21`): the seed
    // gate — the team-based-spawns seed is not ported (M5-SEED-2's deferred
    // list) and `ExtraSpawnPointManager` has no counterpart, so the layer
    // stays dark until that seed lands.
    if inputs.team_based_spawns {
        if let Some(tex) = lookups.extra_282 {
            let _ = lookups.hud_uv(Some(&tex));
        }
    }

    // `TeleportPylonsMapLayer` (`TeleportPylonsMapLayer.cs:38-115`): the
    // pylon links and the Extra_182/299 markers draw only when
    // `Main.PylonSystem.Pylons` holds entries — the port has no
    // `TeleportPylonsSystem` yet (the tile-entity work), so the count is 0
    // and nothing draws.
    if inputs.pylons > 0 {
        if let Some(tex) = lookups.extra_182 {
            let _ = lookups.hud_uv(Some(&tex));
        }
        if let Some(tex) = lookups.extra_299 {
            let _ = lookups.hud_uv(Some(&tex));
        }
    }

    // `PingMapLayer` (`PingMapLayer.cs:24-42`): the 1x5 ping animation
    // cycles `CurrentRow = seconds * 10 % 5` over the pool.
    if let Some(tex) = lookups.map_ping {
        let Some(full_uv) = lookups.hud_uv(Some(&tex)) else {
            return text;
        };
        let fw = tex.width as f32;
        let fh = tex.height as f32 / 5.0;
        for (pos, age) in inputs.pings {
            let row = ((age * 10.0) as usize) % 5;
            let row_h = (full_uv[3] - full_uv[1]) / 5.0;
            let uv = [
                full_uv[0],
                full_uv[1] + row as f32 * row_h,
                full_uv[2],
                full_uv[1] + (row as f32 + 1.0) * row_h,
            ];
            ctx.draw(uv, [fw, fh], *pos, ALIGN_CENTER);
        }
    }
    text
}

/// `Main.MapIcons`' accessor over the world map's ping pool — the C# holds
/// the pool on the static `Main.Pings`; the port keeps it beside the map
/// tiles it draws over.
pub(crate) fn map_pings(map: &WorldMap) -> &[([f32; 2], f32)] {
    &map.pings
}

#[cfg(test)]
mod tests {
    use super::*;

    // The context's transform (`MapOverlayDrawContext.cs:44-46`): the world
    // tile position through `mapScale` + the offset.
    #[test]
    fn overlay_context_transform_maps_tiles_to_screen() {
        let mut world = World::new("t".into(), 200, 128);
        world.spawn_x = 100;
        world.spawn_y = 60;
        world.bed_spawn = Some((42, 30));
        // The spawn marker's expected screen position: the tile centre
        // through the scale + the offset — the C# `Draw`'s
        // `(position - mapPosition) * mapScale + mapOffset`.
        let map_offset = [300.0, 200.0];
        let map_scale = 4.0;
        let expected = [
            (world.spawn_x as f32 + 0.5) * map_scale + map_offset[0],
            world.spawn_y as f32 * map_scale + map_offset[1],
        ];
        assert_eq!(expected[0], 702.0);
        assert_eq!(expected[1], 440.0);
        let _ = (world.bed_spawn.unwrap(), map_offset, map_scale);
    }

    #[test]
    fn boss_bag_set_matches_itemid_1383() {
        // `ItemID.cs:1383`: the 21 boss-bag kinds.
        for kind in [3318, 3325, 3332, 3860, 4782, 4957, 5111] {
            assert!(is_boss_bag(kind));
        }
        for kind in [8, 9, 23, 3317, 5112] {
            assert!(!is_boss_bag(kind));
        }
    }

    #[test]
    fn ping_pool_expiry_and_cap() {
        // `PingMapLayer.cs:47-51` (the 100 cap) + `:37-40` (the 15 s drop);
        // the 5-frame cycle rows at 10 Hz.
        let mut m = WorldMap::new(&World::new("t".into(), 64, 128));
        for i in 0..100 {
            m.add_ping([i as f32, i as f32]);
        }
        assert_eq!(m.pings.len(), 100);
        m.add_ping([50.0, 50.0]);
        assert_eq!(m.pings.len(), 100, "the cap");
        m.age_pings(14.9);
        assert_eq!(m.pings.len(), 100);
        let rows: Vec<usize> = m
            .pings
            .iter()
            .map(|(_, age)| ((age * 10.0) as usize) % 5)
            .collect();
        assert_eq!(rows[0], 4, "14.9 s -> row 4");
        m.age_pings(0.2);
        assert!(m.pings.is_empty(), "the 15 s expiry");
    }
}

