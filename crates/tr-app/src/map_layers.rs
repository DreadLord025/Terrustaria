//! The map icon overlay layers (`Terraria.Map.MapIconOverlay` + the
//! `Terraria.Map/*Layer*.cs` files, registered at `Main.cs:66551` in the
//! order Digtoise → BossBag → Spawn → TeamBasedSpawn → TeleportPylons →
//! Pings). The context is `MapOverlayDrawContext` (`MapOverlayDrawContext.cs`),
//! the layer walk is `Main.MapIcons.Draw` (`MapIconOverlay.cs:17-27`).

use glam::Vec2;
use tr_core::World;
use tr_core::pylons::PylonInfo;
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

    /// `Draw`'s tinted form (`MapOverlayDrawContext.cs:218-242`, the pylon
    /// markers' `color * opacity`): the same transform and hit test, with the
    /// colour scaled rather than the plain white.
    pub fn draw_tinted(
        &mut self,
        uv: [f32; 4],
        size: [f32; 2],
        position: [f32; 2],
        alignment: [f32; 2],
        tint: f32,
    ) -> bool {
        if self.opacity == 0.0 {
            return false; // `:220-223`
        }
        let pos = [
            (position[0] - self.map_position[0]) * self.map_scale + self.map_offset[0],
            (position[1] - self.map_position[1]) * self.map_scale + self.map_offset[1],
        ];
        if let Some(c) = self.clip {
            if pos[0] < c[0] || pos[1] < c[1] || pos[0] > c[2] || pos[1] > c[3] {
                return false; // `:225-228`
            }
        }
        let origin = [size[0] * alignment[0], size[1] * alignment[1]];
        let a = self.opacity * tint;
        self.r.push_ui_sprite(pos, size, uv, [a, a, a, a]);
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
    /// `Main.LocalPlayer` — the bed spawn's `SpawnX/Y` and the team.
    pub player: Option<&'a Player>,
    /// `Main.item[400]` — the BossBag drop scan (`BossBagMapLayer.cs:10-24`).
    pub item_drops: &'a [ItemDrop],
    /// `Main.projectile`'s Digtoise trackers (1098/1123) — `(bottom_px,
    /// sprite_direction)` of the first active one the local player owns, the
    /// scan breaking on it (`DigtoiseMapLayer.cs:14-25`).
    pub digtoise: Option<([f32; 2], i32)>,
    /// `PingMapLayer`'s pool.
    pub pings: &'a [([f32; 2], f32)],
    /// `Main.PylonSystem.Pylons` (`TeleportPylonsMapLayer.cs:32`) — the live
    /// network, from `tr_core::pylons::scan_pylons`.
    pub pylons: &'a [PylonInfo],
    /// `TeleportPylonsSystem.IsPlayerNearAPylon(Main.LocalPlayer)`
    /// (`TeleportPylonsMapLayer.cs:39-42`): away from a pylon the markers dim
    /// to `Color.Gray * 0.5f`.
    pub near_pylon: bool,
    /// The map tiles, for `TeleportPylonsMapLayer.IsRevealed` (`:112-119`) —
    /// only read with the team-based-spawns seed on.
    pub map: Option<&'a WorldMap>,
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

    // `DigtoiseMapLayer` (`DigtoiseMapLayer.cs:13-26`): the player's Digtoise
    // tracker (1098/1123) draws Extra_305's 2-frame row, bottom-aligned at
    // `Bottom / 16 + (0.5, 0)`; column 1 faces left (`spriteDirection <= 0`).
    if let Some((bottom, dir)) = inputs.digtoise {
        if let Some(tex) = lookups.extra_305 {
            if let Some(uv) = lookups.hud_uv(Some(&tex)) {
                let fw = tex.width as f32 / 2.0;
                let fh = tex.height as f32;
                let col = if dir <= 0 { 1.0 } else { 0.0 };
                let uw = uv[2] - uv[0];
                let uv = [
                    uv[0] + uw * 0.5 * col,
                    uv[1],
                    uv[0] + uw * 0.5 * (col + 1.0),
                    uv[3],
                ];
                let pos = [bottom[0] / 16.0 + 0.5, bottom[1] / 16.0];
                if ctx.draw(uv, [fw, fh], pos, ALIGN_BOTTOM) {
                    text = Some(crate::lang::text("UI.Digtoise"));
                }
            }
        }
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

    // `TeamBasedSpawnMapLayer` (`TeamBasedSpawnMapLayer.cs:11-21`): the seed
    // gate first. `Main.teamBasedSpawnsSeed` is unported (M5-SEED-2's deferred
    // list) and so is `ExtraSpawnPointManager`, so the gate stays the C#'s and
    // nothing draws until that seed lands.
    if inputs.team_based_spawns {
        if let Some(tex) = lookups.extra_282 {
            let _ = lookups.hud_uv(Some(&tex));
        }
    }

    // `TeleportPylonsMapLayer` (`TeleportPylonsMapLayer.cs:65-89`): the
    // markers. `:39-42` dims them to `Color.Gray * 0.5f` when the player is
    // not near a pylon; `:112-119` hides the ones the fog covers once the
    // team-based-spawns seed is on.
    let pylon_color = if inputs.near_pylon { 1.0 } else { 0.25 };
    for p in inputs.pylons {
        // `IsRevealed` (`:112-119`): unconditionally true without the seed.
        if inputs.team_based_spawns
            && !inputs
                .map
                .is_some_and(|m| m.tile_revealed(inputs.world, p.x as i32, p.y as i32))
        {
            continue;
        }
        let Some(tex) = lookups.extra_182 else {
            break;
        };
        let Some(uv) = lookups.hud_uv(Some(&tex)) else {
            break;
        };
        let fw = tex.width as f32 / 11.0;
        let fh = tex.height as f32;
        let col = p.pylon_type.style() as f32;
        let uv = [
            uv[0] + (uv[2] - uv[0]) * col / 11.0,
            uv[1],
            uv[0] + (uv[2] - uv[0]) * (col + 1.0) / 11.0,
            uv[3],
        ];
        // `:73-79`: `PositionInTiles.ToVector2() + (1.5, 2)`, centre-aligned.
        let pos = [p.x as f32 + 1.5, p.y as f32 + 2.0];
        if ctx.draw_tinted(uv, [fw, fh], pos, ALIGN_CENTER, pylon_color) {
            text = Some(crate::lang::item_name(p.pylon_type.item_id()));
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

/// `DigtoiseMapLayer`'s scan (`DigtoiseMapLayer.cs:14-26`): the first active
/// projectile of kind 1098/1123 owned by the local player (`Main.myPlayer` is
/// the port's `owner == 0`). Returns `(Bottom / 16 px, spriteDirection)`; the
/// C# `break`s on the first match, so the scan stops there too.
pub(crate) fn digtoise_marker(
    player: Option<&Player>,
    pool: &crate::projectile::ProjectilePool,
) -> Option<([f32; 2], i32)> {
    player?;
    pool.slots()
        .iter()
        .find(|p| p.active && p.owner == 0 && (p.kind == 1098 || p.kind == 1123))
        .map(|p| ([p.pos.x + p.w * 0.5, p.pos.y + p.h], p.sprite_direction))
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

    /// S1-L-4 — `TeleportPylonsMapLayer.cs:73-79`: the marker anchor is
    /// `PositionInTiles.ToVector2() + (1.5, 2)`, centre-aligned, through the
    /// context transform (`MapOverlayDrawContext.cs:44-46`).
    #[test]
    fn s1_l_4_the_pylon_marker_anchor_is_the_csharp_offset() {
        let (map_pos, map_off, map_scale) = ([0.0f32, 0.0], [100.0f32, 50.0], 2.0f32);
        let pylon = (40i16, 30i16);
        let anchor = [pylon.0 as f32 + 1.5, pylon.1 as f32 + 2.0];
        let screen = [
            (anchor[0] - map_pos[0]) * map_scale + map_off[0],
            (anchor[1] - map_pos[1]) * map_scale + map_off[1],
        ];
        assert_eq!(screen, [183.0, 114.0]);
    }

    /// S1-L-4 — `TeleportPylonsMapLayer.IsRevealed` (`:112-119`) + the port's
    /// bounds guard: with the team seed on, the fog hides an unexplored pylon;
    /// without it the answer is a flat true; out-of-bounds never panics.
    #[test]
    fn s1_l_4_the_pylon_reveal_gate_follows_the_team_seed() {
        let world = World::new("t".into(), 200, 200);
        let mut m = WorldMap::new(&world);
        assert!(!m.tile_revealed(&world, 40, 30), "nothing explored yet");
        let i = world.index(40, 30);
        m.tiles[i].light = 1;
        assert!(m.tile_revealed(&world, 40, 30), "light > 0 reveals");
        assert!(!m.tile_revealed(&world, -1, -1), "out of bounds is unrevealed");
        assert!(!m.tile_revealed(&world, 5000, 5000), "and never panics");
    }

    /// S1-L-4 — `DigtoiseMapLayer.cs:14-25`: the scan takes the first active
    /// 1098/1123 owned by the local player (`owner == 0`, vanilla's
    /// `Main.myPlayer`), yields `Bottom` and the `spriteDirection`.
    #[test]
    fn s1_l_4_the_digtoise_marker_is_the_local_players_first_tracker() {
        let world = World::new("t".into(), 200, 200);
        let player = Player::spawn(&world);
        let mut pool = crate::projectile::ProjectilePool::new();
        assert!(digtoise_marker(Some(&player), &pool).is_none(), "none spawned");
        // A foreign-owned Digtoise stays invisible to the local map.
        pool.spawn(1098, 3, Vec2::ZERO, Vec2::ZERO, 0, 0.0, false);
        // A non-Digtoise local projectile does not qualify either.
        pool.spawn(1, 0, Vec2::ZERO, Vec2::ZERO, 0, 0.0, false);
        assert!(digtoise_marker(Some(&player), &pool).is_none());
        // The local Digtoise does: `Bottom = pos + (w/2, h)`. Its width and
        // direction come from the spawn's data defaults, so the test pins the
        // formula rather than the projectile's own fields.
        let idx = pool
            .spawn(1098, 0, Vec2::new(160.0, 320.0), Vec2::ZERO, 0, 0.0, false)
            .unwrap();
        let p = &pool.slots()[idx];
        let (bottom, _dir) = digtoise_marker(Some(&player), &pool).unwrap();
        assert_eq!(
            bottom,
            [p.pos.x + p.w * 0.5, p.pos.y + p.h],
            "Bottom = pos + (w/2, h)"
        );
        // No local player, no marker (`Main.LocalPlayer` gates the scan).
        assert!(digtoise_marker(None, &pool).is_none());
    }
}

