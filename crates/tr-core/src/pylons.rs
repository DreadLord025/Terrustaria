//! The pylon network — `GameContent/TeleportPylonsSystem.cs`,
//! `TeleportPylonInfo.cs`/`TeleportPylonType.cs` and the
//! `TELogicSensor`-style shell `TETeleportationPylon.cs`.
//!
//! Source map (line numbers into `decompiled/Terraria/`):
//! - the type enum `TeleportPylonType.cs:5-17` (11 kinds, the discriminant =
//!   the tile style, `TETeleportationPylon.cs:175-178`)
//! - the style → item map (`:116-131`): 0 → 4876 (the default surface
//!   purity), 1 → 4875, 2-8 → 4916-4921 + 4951, 9/10 → 5652/5653
//! - the network scan (`TeleportPylonsSystem.cs:62-84`): the TE book walk
//!   (`TETeleportationPylon` + `TryGetPylonType`)
//! - the teleport gate ladder (`HandleTeleportRequest` `:93-186`): the
//!   player-near check, the NPC-home count (2, the Victory pylon 0), the
//!   Lihzahrd wall guard, the biome gate per type, the other-usable-pylon
//!   stage ladder (1/2/0)
//! - `DoesPositionHaveEnoughNPCs` (`:203-219`): the town NPC homes inside
//!   `ZoneScanSize` and < 100 px from the pylon centre
//! - `DoesPylonAcceptTeleportation` (`:240-306`): the per-type biome gates
//!   off `SceneMetrics.EnoughTilesFor*`
//!
//! Deviations (ANALYSIS "Pylon network (M7-PYLON-1)"): the remix branches
//! are skipped (the port generates no remix worlds); the NPC-home scan runs
//! over the caller-provided home list (the town NPC field `homeTileX/Y`
//! arrives with M4-TOWN); the map-click UI and the net module are P4/MP
//! scope; the teleport itself is the port's positional move (the effects =
//! M2-DEATH polish).

use crate::scene::SceneMetrics;
use crate::world::World;

/// `TeleportPylonType` (`TeleportPylonType.cs:5-17`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PylonType {
    SurfacePurity,
    Jungle,
    Hallow,
    Underground,
    Beach,
    Desert,
    Snow,
    GlowingMushroom,
    Victory,
    Underworld,
    Shimmer,
}

impl PylonType {
    pub fn from_id(id: u8) -> Option<Self> {
        Some(match id {
            0 => PylonType::SurfacePurity,
            1 => PylonType::Jungle,
            2 => PylonType::Hallow,
            3 => PylonType::Underground,
            4 => PylonType::Beach,
            5 => PylonType::Desert,
            6 => PylonType::Snow,
            7 => PylonType::GlowingMushroom,
            8 => PylonType::Victory,
            9 => PylonType::Underworld,
            10 => PylonType::Shimmer,
            _ => return None,
        })
    }

    /// `GetPylonTypeFromPylonTileStyle` (`TETeleportationPylon.cs:175-178`):
    /// the tile style IS the type (`frameX / 54`).
    pub fn from_tile_style(style: i32) -> Option<Self> {
        Self::from_id(style as u8)
    }

    /// `GetPylonItemTypeFromTileStyle` (`:116-131`): the placed item.
    pub fn item_id(self) -> u16 {
        match self {
            PylonType::SurfacePurity => 4876,
            PylonType::Jungle => 4875,
            PylonType::Hallow => 4916,
            PylonType::Underground => 4917,
            PylonType::Beach => 4918,
            PylonType::Desert => 4919,
            PylonType::Snow => 4920,
            PylonType::GlowingMushroom => 4921,
            PylonType::Victory => 4951,
            PylonType::Underworld => 5652,
            PylonType::Shimmer => 5653,
        }
    }
}

/// `TeleportPylonInfo.cs`: the network entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PylonInfo {
    pub x: i16,
    pub y: i16,
    pub pylon_type: PylonType,
}

/// `HowManyNPCsDoesPylonNeed` (`:309-316`): 2 homes, except the Victory
/// pylon's 0.
pub fn npcs_needed(pylon_type: PylonType) -> i32 {
    if pylon_type != PylonType::Victory {
        return 2;
    }
    0
}

/// `DoesPositionHaveEnoughNPCs` (`:203-219`): the town-NPC homes inside the
/// `ZoneScanSize` rectangle centred on the pylon, each home within 100
/// **tiles** of its NPC's current centre (`Vector2.Distance(homeTile,
/// Center / 16) < 100f` — the home-and-NPC proximity, verbatim). `homes`
/// are the `((homeTileX, homeTileY), (centerX / 16, centerY / 16))` pairs
/// of the `isLikeATownNPC && !homeless` NPCs — the app supplies them
/// (M4-TOWN owns the fields).
pub fn does_position_have_enough_npcs(
    homes: &[((i32, i32), (f32, f32))],
    center: (i16, i16),
    needed: i32,
    world: &World,
) -> bool {
    if needed <= 0 {
        return true;
    }
    let (zw, zh) = crate::scene::ZONE_SCAN_SIZE;
    let (cx, cy) = (center.0 as i32, center.1 as i32);
    let (x0, x1) = (cx - zw / 2, cx + zw / 2);
    let (y0, y1) = (cy - zh / 2, cy + zh / 2);
    let mut count = needed;
    for &((hx, hy), (ncx, ncy)) in homes {
        if hx < x0 || hx > x1 || hy < y0 || hy > y1 {
            continue;
        }
        let dx = hx as f32 - ncx;
        let dy = hy as f32 - ncy;
        if (dx * dx + dy * dy).sqrt() < 100.0 {
            count -= 1;
            if count == 0 {
                return true;
            }
        }
    }
    false
}

/// `DoesPylonAcceptTeleportation` (`:240-306`), the non-remix branch: the
/// per-type biome gate. The inputs are the world's boundaries and the scene
/// metrics scanned at the pylon (`SceneMetricsScanSettings` with the pylon
/// as the centre, `:138-142`).
#[allow(clippy::too_many_arguments)]
pub fn does_pylon_accept_teleportation(
    metrics: &SceneMetrics,
    pylon_type: PylonType,
    x: i32,
    y: i32,
    world: &World,
) -> bool {
    match pylon_type {
        // `:241-262`: the surface purity — above the surface, away from the
        // ocean bands, and no other biome claiming the spot.
        PylonType::SurfacePurity => {
            let surface_ok = (y as f64) <= world.world_surface;
            if !surface_ok || x >= world.width - 380 || x <= 380 {
                return false;
            }
            !(metrics.enough_tiles_for_jungle()
                || metrics.enough_tiles_for_snow()
                || metrics.enough_tiles_for_desert()
                || metrics.enough_tiles_for_glowing_mushroom()
                || metrics.enough_tiles_for_hallow()
                || metrics.enough_tiles_for_crimson()
                || metrics.enough_tiles_for_corruption())
        }
        PylonType::Jungle => metrics.enough_tiles_for_jungle(),
        // : the hallow gate.
        PylonType::Hallow => metrics.enough_tiles_for_hallow(),
        PylonType::Snow => metrics.enough_tiles_for_snow(),
        PylonType::Desert => metrics.enough_tiles_for_desert(),
        // `:280-290`: the ocean bands — above the surface, inside the
        // 380-tile edges.
        PylonType::Beach => {
            let height_ok = (y as f64) <= world.world_surface
                && (y as f64) > world.world_surface * 0.349_999_994_039_535_5;
            let edge_ok = x >= world.width - 380 || x <= 380;
            edge_ok && height_ok
        }
        PylonType::GlowingMushroom => metrics.enough_tiles_for_glowing_mushroom(),
        // `:293-294`: below the surface.
        PylonType::Underground => (y as f64) >= world.world_surface,
        PylonType::Victory => true,
        // `:297-298`: the underworld depth (`Main.UnderworldLayer =
        // maxTilesY - 200`, `Main.cs:2790`).
        PylonType::Underworld => y >= world.height - 200,
        PylonType::Shimmer => metrics.enough_tiles_for_shimmer(),
    }
}

/// `IsPlayerNearAPylon` (`:191-193`): the tile 597 in the pylon interaction
/// range — the port checks the 3×4 pylon body around the player's tile.
pub fn is_player_near_a_pylon(world: &World, player_tile: (i32, i32)) -> bool {
    let (px, py) = player_tile;
    for dy in -4..=4i32 {
        for dx in -5..=5i32 {
            if let Some(t) = world.tile(px + dx, py + dy) {
                if t.active() && t.kind == 597 {
                    return true;
                }
            }
        }
    }
    false
}

/// The teleport outcome (`HandleTeleportRequest`'s key ladder, `:93-186`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TeleportOutcome {
    /// The gates passed — the player moves to the pylon.
    Teleported,
    /// `Net.CannotTeleportToPylonBecausePlayerIsNotNearAPylon`.
    NotNear,
    /// `...BecauseNotEnoughNPCs` at the destination.
    NotEnoughNPCs,
    /// `...BecauseAccessingLihzahrdTempleEarly`.
    Temple,
    /// `...BecauseNotMeetingBiomeRequirements`.
    Biome,
    /// The stage ladder (`:166-176`): 1 = no NPCs at the current pylon,
    /// 2 = no biome, else not-near.
    NoUsablePylon(u8),
}

/// `HandleTeleportRequest` (`:93-186`) minus the net/chat sides: the gate
/// ladder. `homes` feed the NPC-home checks; `metrics` must be pre-scanned
/// at the DESTINATION pylon; `usable_metrics` is the callback scanning each
/// candidate pylon (the port's SceneMetrics is expensive, so the scan stays
/// the caller's).
pub fn handle_teleport_request<F>(
    world: &World,
    pylons: &[PylonInfo],
    info: PylonInfo,
    player_near_pylon: bool,
    player_tile: (i32, i32),
    homes: &[((i32, i32), (f32, f32))],
    downed_plant_boss: bool,
    metrics: &SceneMetrics,
    mut scan: F,
) -> TeleportOutcome
where
    F: FnMut((i16, i16)) -> SceneMetrics,
{
    // The `flag` chain (`:96-186`): the first failed gate wins the message.
    // `:97-104`: the player stands near a pylon.
    if !player_near_pylon {
        return TeleportOutcome::NotNear;
    }
    // `:106-114`: the destination's NPC homes.
    let needed = npcs_needed(info.pylon_type);
    if !does_position_have_enough_npcs(homes, (info.x, info.y), needed, world) {
        return TeleportOutcome::NotEnoughNPCs;
    }
    // `:116-127`: the temple wall guard — the pre-Golem Lihzahrd wall.
    if !downed_plant_boss
        && info.y as f64 > world.world_surface
        && world
            .tile(info.x as i32, info.y as i32)
            .is_some_and(|t| t.wall == 87)
    {
        return TeleportOutcome::Temple;
    }
    // `:128-144`: the destination's biome gate.
    if !does_pylon_accept_teleportation(metrics, info.pylon_type, info.x as i32, info.y as i32, world) {
        return TeleportOutcome::Biome;
    }
    // `:145-176`: at least one OTHER pylon reachable from here must pass its
    // own gates; the failure stage picks the error key.
    let mut stage = 0u8;
    for p in pylons {
        if !pylon_in_interaction_range(player_tile, (p.x as i32, p.y as i32)) {
            continue;
        }
        if stage < 1 {
            stage = 1;
        }
        if does_position_have_enough_npcs(
            homes,
            (p.x, p.y),
            npcs_needed(p.pylon_type),
            world,
        ) {
            let m = scan((p.x, p.y));
            if does_pylon_accept_teleportation(&m, p.pylon_type, p.x as i32, p.y as i32, world) {
                return TeleportOutcome::Teleported;
            }
        }
    }
    TeleportOutcome::NoUsablePylon(stage)
}

/// `InTileEntityInteractionRange` + `TileReachCheckSettings.Pylons`
/// (`Player.cs:32015-32025`, `TileReachCheckSettings.cs:86-90`): the 60-tile
/// override range over the 3×4 pylon body.
pub fn pylon_in_interaction_range(player_tile: (i32, i32), pylon: (i32, i32)) -> bool {
    let reach = 60;
    (player_tile.0 - pylon.0).abs() <= reach && (player_tile.1 - pylon.1).abs() <= reach
}

/// The network scan (`UpdatePylonsListAndBroadcastChanges`, `:62-84`): the
/// pylon entities off the TE book, the type from the anchor tile's frame.
pub fn scan_pylons(world: &World) -> Vec<PylonInfo> {
    let mut out = Vec::new();
    for e in world.tile_entities.entities() {
        if e.kind != crate::tile_entities::TeKind::TeleportationPylon {
            continue;
        }
        let Some(t) = world.tile(e.x as i32, e.y as i32) else {
            continue;
        };
        if !t.active() || t.kind != 597 {
            continue;
        }
        // `GetPylonStyleFromTile` (`:113-115`): `frameX / 54`.
        let Some(pylon_type) = PylonType::from_tile_style(t.frame_x as i32 / 54) else {
            continue;
        };
        out.push(PylonInfo {
            x: e.x,
            y: e.y,
            pylon_type,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::{FLAG_ACTIVE, Tile};

    fn world() -> World {
        World::new("t".into(), 800, 400)
    }

    fn put_pylon(world: &mut World, x: i32, y: i32, style: i32) {
        let i = world.index(x, y);
        world.tiles[i] = Tile {
            kind: 597,
            frame_x: (style * 54) as i16,
            ..Default::default()
        };
        world.tiles[i].flags |= FLAG_ACTIVE;
    }

    #[test]
    fn the_type_map_and_items_are_verbatim() {
        // `TeleportPylonType.cs:5-17` + `TETeleportationPylon.cs:116-131`:
        // the style IS the type; the items 4876/4875/4916-4921/4951/
        // 5652-5653.
        assert_eq!(PylonType::from_tile_style(0), Some(PylonType::SurfacePurity));
        assert_eq!(PylonType::from_tile_style(7), Some(PylonType::GlowingMushroom));
        assert_eq!(PylonType::from_tile_style(11), None);
        assert_eq!(PylonType::SurfacePurity.item_id(), 4876);
        assert_eq!(PylonType::Jungle.item_id(), 4875);
        assert_eq!(PylonType::GlowingMushroom.item_id(), 4921);
        assert_eq!(PylonType::Shimmer.item_id(), 5653);
    }

    #[test]
    fn the_npc_home_check_uses_the_zone_and_100px() {
        // `:203-219`: the homes inside `ZoneScanSize` and < 100 px from the
        // pylon.
        let w = world();
        let center = (400i16, 100i16);
        // Two homes within 100 tiles of their NPCs.
        let homes = [
            ((399, 99), (399.5, 99.5)),
            ((402, 103), (402.5, 103.5)),
        ];
        assert!(does_position_have_enough_npcs(&homes, center, 2, &w));
        // An NPC 150 tiles from its home counts out.
        let homes = [
            ((399, 99), (399.5, 99.5)),
            ((402, 103), (552.5, 103.5)),
        ];
        assert!(!does_position_have_enough_npcs(&homes, center, 2, &w));
        // A home outside the ZoneScanSize rect counts out.
        let homes = [
            ((399, 99), (399.5, 99.5)),
            ((490, 100), (490.5, 100.5)),
        ];
        assert!(!does_position_have_enough_npcs(&homes, center, 2, &w));
        // The Victory pylon needs none.
        assert!(does_position_have_enough_npcs(&[], center, 0, &w));
    }

    #[test]
    fn the_biome_gates_match_the_types() {
        let mut metrics = SceneMetrics::new();
        let w = world();
        // The pure surface: the purity pylon works above the surface away
        // from the edges.
        metrics.snow_tile_count = 0;
        assert!(does_pylon_accept_teleportation(
            &metrics, PylonType::SurfacePurity, 400, 100, &w
        ));
        // The snow gate: 1500 snow tiles.
        metrics.snow_tile_count = 1500;
        assert!(does_pylon_accept_teleportation(&metrics, PylonType::Snow, 400, 100, &w));
        assert!(
            !does_pylon_accept_teleportation(&metrics, PylonType::SurfacePurity, 400, 100, &w),
            "a biome at the spot claims the purity pylon"
        );
        // The jungle gate: 140 tiles.
        metrics.snow_tile_count = 0;
        metrics.jungle_tile_count = 140;
        assert!(does_pylon_accept_teleportation(&metrics, PylonType::Jungle, 400, 100, &w));
        assert!(!does_pylon_accept_teleportation(&metrics, PylonType::Snow, 400, 100, &w));
        // The beach edges: inside the 380-tile bands above the surface.
        metrics.jungle_tile_count = 0;
        assert!(does_pylon_accept_teleportation(&metrics, PylonType::Beach, 300, 100, &w));
        assert!(!does_pylon_accept_teleportation(&metrics, PylonType::Beach, 400, 100, &w));
        // The underworld depth.
        assert!(does_pylon_accept_teleportation(
            &metrics, PylonType::Underworld, 400, 250, &w
        ));
        assert!(!does_pylon_accept_teleportation(
            &metrics, PylonType::Underworld, 400, 100, &w
        ));
    }

    #[test]
    fn the_teleport_ladder_walks_the_gates() {
        let mut w = world();
        // Two pylons: the snow one far away, the purity one near the player.
        put_pylon(&mut w, 400, 100, 6); // snow
        put_pylon(&mut w, 500, 100, 0); // purity
        w.tile_entities
            .place_unchecked(crate::tile_entities::TeKind::TeleportationPylon, 400, 100);
        w.tile_entities
            .place_unchecked(crate::tile_entities::TeKind::TeleportationPylon, 500, 100);
        let pylons = scan_pylons(&w);
        assert_eq!(pylons.len(), 2);
        // The destination snow pylon without snow tiles → Biome (the NPC
        // gate passes on two at-home townsfolk).
        let metrics = SceneMetrics::new();
        let homes = [
            ((400, 102), (400.5, 102.5)),
            ((401, 102), (401.5, 102.5)),
        ];
        let outcome = handle_teleport_request(
            &w,
            &pylons,
            pylons[0],
            true,
            (500, 100),
            &homes,
            false,
            &metrics,
            |_| SceneMetrics::new(),
        );
        assert_eq!(outcome, TeleportOutcome::Biome);
        // The empty home list → NotEnoughNPCs first (the C# gate order).
        let outcome = handle_teleport_request(
            &w,
            &pylons,
            pylons[0],
            true,
            (500, 100),
            &[],
            false,
            &metrics,
            |_| SceneMetrics::new(),
        );
        assert_eq!(outcome, TeleportOutcome::NotEnoughNPCs);
        // Not near a pylon at all → NotNear (the first gate wins before
        // the NPC check — the C#'s  chain).
        let outcome = handle_teleport_request(
            &w,
            &pylons,
            pylons[0],
            false,
            (500, 100),
            &[],
            false,
            &metrics,
            |_| SceneMetrics::new(),
        );
        assert_eq!(outcome, TeleportOutcome::NotNear);
        _ = &homes;
        // The temple wall below the surface → Temple.
        let mut w2 = world();
        put_pylon(&mut w2, 400, 300, 0);
        let i = w2.index(400, 300);
        w2.tiles[i].wall = 87;
        w2.tile_entities
            .place_unchecked(crate::tile_entities::TeKind::TeleportationPylon, 400, 300);
        let pylons2 = scan_pylons(&w2);
        assert_eq!(pylons2.len(), 1);
        let homes = [
            ((400, 302), (400.5, 302.5)),
            ((401, 302), (401.5, 302.5)),
        ];
        let outcome = handle_teleport_request(
            &w2,
            &pylons2,
            pylons2[0],
            true,
            (500, 100),
            &homes,
            false,
            &metrics,
            |_| SceneMetrics::new(),
        );
        assert_eq!(outcome, TeleportOutcome::Temple);
    }

    #[test]
    fn the_network_scan_reads_the_styles() {
        let mut w = world();
        put_pylon(&mut w, 400, 100, 1); // jungle
        put_pylon(&mut w, 500, 100, 8); // victory
        w.tile_entities
            .place_unchecked(crate::tile_entities::TeKind::TeleportationPylon, 400, 100);
        w.tile_entities
            .place_unchecked(crate::tile_entities::TeKind::TeleportationPylon, 500, 100);
        let pylons = scan_pylons(&w);
        assert_eq!(pylons.len(), 2);
        assert_eq!(pylons[0].pylon_type, PylonType::Jungle);
        assert_eq!(pylons[1].pylon_type, PylonType::Victory);
        assert_eq!((pylons[0].x, pylons[0].y), (400, 100));
    }
}
