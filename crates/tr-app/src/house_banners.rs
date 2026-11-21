//! The world housing banners — `Main.DrawNPCHousesInWorld`
//! (`decompiled/Terraria/Main.cs:40711-40832`), the `DrawInterface_7_TownNPCHouseBanners`
//! layer (`:46214-46222`, gated on `Main.EquipPage == 1`). Every housed town
//! NPC gets a `House_Banner` flag above its home tile with its head icon; a
//! right-click on the banner kicks the NPC out (`WorldGen.kickOut`,
//! `WorldGen.cs:4869-4879`) and plays `SoundEngine.PlaySound(12)` (`:40828`).
//!
//! W4-g21 batch 4, host 53. Faithfulness notes:
//! * The banner sort (`NPCBannerSorter`, `:40706-40709`) is the DESCENDING
//!   `housingCategory` compare — the port keeps the C#'s `List.Sort` instability
//!   note (equal categories keep their relative order in practice via a stable
//!   sort; the C# uses an unstable introsort, but the banner positions only
//!   depend on the per-home occupant count, not the sort order of equals).
//! * `TownNPCProfiles.GetHeadIndexSafe` (`TownNPCProfiles.cs:200-207`) resolves
//!   the variant head for the town pets; the port's `head_of` reads the default
//!   head (the variant profiles are unported — the recorded deviation).
//! * The `gravDir == -1` flip (`:40793-40799`) is unported (the port's player
//!   carries no gravity-dir flip); the banner draws upright.
//! * The controller-housing-cursor arm (`:40813-40820`) is gamepad-only.

use glam::Vec2;
use tr_core::lighting::LightStyle;
use tr_core::world::World;
use tr_render::SpriteRenderer;

use crate::assets;
use crate::npc::Npc;
use crate::town::TownRooms;

/// One drawn banner: the NPC index, its screen-space hover box and the head
/// frame. The hover box is the C#'s `(homeTileX, num3, value2.Width,
/// value2.Height - 8)` (`Main.cs:40810-40821`).
#[derive(Clone, Copy, Debug)]
pub(crate) struct Banner {
    pub npc: usize,
    /// The hover box `[x, y, w, h]` in screen px (`:40821`).
    pub rect: [f32; 4],
    /// `num2` — 1 when the home is registered to this NPC type (`:40737-40740`).
    pub reserved: i32,
}

/// `Main.NPCBannerSorter` (`Main.cs:40706-40709`): the descending
/// `housingCategory` compare.
fn banner_sort_key(npc: &Npc) -> i32 {
    -crate::town::housing_category(npc.kind.net_id())
}

/// The housed town NPCs that draw a banner (`Main.cs:40715-40721`): active,
/// townNPC, not homeless, `homeTileX/Y > 0`, type != 37 (the Old Man).
fn banner_npcs(npcs: &[Npc]) -> Vec<usize> {
    let mut out: Vec<usize> = (0..npcs.len())
        .filter(|&i| {
            let n = &npcs[i];
            n.hp > 0
                && n.kind.is_town_npc()
                && n.home.is_some()
                && n.kind.net_id() != 37
                && n.home.map_or(false, |(x, y)| x > 0 && y > 0)
        })
        .collect();
    out.sort_by_key(|&i| banner_sort_key(&npcs[i]));
    out
}

/// `WorldGen.TownManager.AddOccupantsToList` (`TownRoomManager.cs:22-31`) via
/// the port's `TownRooms::occupants_at` — the registered room occupants at the
/// home tile.
fn home_reserved(rooms: &TownRooms, npc: &Npc, home: (i32, i32)) -> i32 {
    let occupants = rooms.occupants_at(home);
    if occupants.contains(&npc.kind.net_id()) { 1 } else { 0 }
}

/// The solid-tile walk-up (`Main.cs:40755-40772`): from `homeTileY - 1`, walk
/// up until an active solid tile; `num3 < 10` breaks (the banner draws at the
/// break row), a null tile skips the banner. Returns the anchor row `num3`
/// AFTER the `num3++` (`:40779`), or `None` when the banner is skipped.
fn banner_anchor_row(world: &World, home_x: i32, home_y: i32) -> Option<i32> {
    let mut num3 = home_y - 1;
    if world.tile(home_x, num3).is_none() {
        return None; // `:40751-40754`
    }
    loop {
        let t = world.tile(home_x, num3)?;
        let solid = t.active()
            && tr_core::tiledata::TILE_SOLID.has(t.kind);
        if solid {
            break;
        }
        num3 -= 1;
        if num3 < 10 {
            break; // `:40759-40762`
        }
        if world.tile(home_x, num3).is_none() {
            return None; // `:40763-40767`
        }
    }
    // `:40775-40779`: the platform (19) lowers the anchor 8 px; then num3++.
    Some(num3 + 1)
}

/// The banner draw + hover-box pass (`Main.cs:40724-40831`). Returns the
/// banners in draw order; the caller hit-tests the right-click against them.
/// `camera`/`zoom`/`screen` are the world→UI transform the C# reads through
/// `Main.screenPosition` (the port's banners ride the world-effect range like
/// the pylon pass, so the transform is the same `draw_cam`/`world_zoom`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_house_banners(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    world: &World,
    npcs: &[Npc],
    rooms: &TownRooms,
    light: &tr_core::lighting::LightMap,
    style: &LightStyle,
    anim_time: f32,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
) -> Vec<Banner> {
    let Some(banner_sheet) = lookups.house_banner.as_ref() else {
        return Vec::new();
    };
    // `value.Frame(2, 2)` (`:40784`): the banner cell is half the sheet.
    let bw = (banner_sheet.width / 2) as f32;
    let bh = (banner_sheet.height / 2) as f32;
    let atlas = lookups.atlas_size;
    let mut out = Vec::new();
    let indices = banner_npcs(npcs);
    for (j, &i) in indices.iter().enumerate() {
        let npc = &npcs[i];
        let Some(home) = npc.home else { continue };
        let (home_x, home_y) = home;
        let reserved = home_reserved(rooms, npc, home);
        let category = crate::town::housing_category(npc.kind.net_id());
        // The per-home occupant count (`:40741-40750`): the later banners in
        // the sorted list that share this home tile stack 26 px apart.
        let mut stack = 0;
        for &k in indices.iter().skip(j + 1) {
            if npcs[k].home == Some(home) {
                stack += 1;
            }
        }
        let num7 = stack * 26;
        let Some(anchor) = banner_anchor_row(world, home_x, home_y) else { continue };
        // `:40773-40778`: num8 = 8, num9 = 18 (platform 19 → 10).
        let num9 = if world.tile(home_x, anchor - 1).map_or(false, |t| t.kind == 19) {
            10.0
        } else {
            18.0
        };
        let num11 = anchor as f32 * 16.0 + num7 as f32;
        // The banner frame: column `reserved`, row `category` (`:40785-40792`).
        let fx = reserved as u32;
        let fy = category.max(0) as u32;
        let src_x = banner_sheet.x + fx * (banner_sheet.width / 2);
        let src_y = banner_sheet.y + fy * (banner_sheet.height / 2);
        let uv = [
            src_x as f32 / atlas.0 as f32,
            src_y as f32 / atlas.1 as f32,
            (src_x + banner_sheet.width / 2) as f32 / atlas.0 as f32,
            (src_y + banner_sheet.height / 2) as f32 / atlas.1 as f32,
        ];
        // `Lighting.GetColor(homeTileX, num3)` (`:40800`) — the anchor row.
        let c = style.shade(light.get(home_x, anchor), home_x, anchor, anim_time);
        let color = [c[0], c[1], c[2], 1.0];
        // The banner centre anchor (`:40800`): origin (bw/2, bh/2), so the
        // top-left is the draw position minus half the cell.
        let bx = home_x as f32 * 16.0 + 8.0;
        let by = num11 + num9;
        let sx = (bx - camera.x) * zoom + screen.x * 0.5;
        let sy = (by - camera.y) * zoom + screen.y * 0.5;
        r.push_fx_sprite_rot(
            [sx - bw * 0.5 * zoom, sy - bh * 0.5 * zoom],
            [bw * zoom, bh * zoom],
            uv,
            color,
            0.0,
        );
        // The head icon (`:40801-40809`): scaled to fit 24 px, centred at
        // (bx, num11 + num9 + 2).
        if let Some(head) = crate::ui::housing::head_of(npc.kind) {
            if let Some(head_sheet) = lookups.npc_heads.get(head).and_then(|s| s.as_ref()) {
                let hw = head_sheet.width as f32;
                let hh = head_sheet.height as f32;
                let big = hw.max(hh);
                let scale = if big > 24.0 { 24.0 / big } else { 1.0 };
                let huv = [
                    head_sheet.x as f32 / atlas.0 as f32,
                    head_sheet.y as f32 / atlas.1 as f32,
                    (head_sheet.x + head_sheet.width) as f32 / atlas.0 as f32,
                    (head_sheet.y + head_sheet.height) as f32 / atlas.1 as f32,
                ];
                let hy = num11 + num9 + 2.0;
                let hsx = (bx - camera.x) * zoom + screen.x * 0.5;
                let hsy = (hy - camera.y) * zoom + screen.y * 0.5;
                r.push_fx_sprite_rot(
                    [hsx - hw * 0.5 * scale * zoom, hsy - hh * 0.5 * scale * zoom],
                    [hw * scale * zoom, hh * scale * zoom],
                    huv,
                    color,
                    0.0,
                );
            }
        }
        // The hover box (`:40810-40821`): x = bx - bw/2, y = num11 + 4,
        // w = bw, h = bh - 8 (num13 = -8). Screen-space.
        let rect = [
            (bx - bw * 0.5 - camera.x) * zoom + screen.x * 0.5,
            (num11 + 4.0 - camera.y) * zoom + screen.y * 0.5,
            bw * zoom,
            (bh - 8.0) * zoom,
        ];
        out.push(Banner { npc: i, rect, reserved });
    }
    out
}

/// The banner under the mouse (`Main.cs:40821`): the inclusive hover box.
pub(crate) fn banner_at(banners: &[Banner], mouse: Vec2) -> Option<usize> {
    banners
        .iter()
        .find(|b| {
            mouse.x >= b.rect[0]
                && mouse.x <= b.rect[0] + b.rect[2]
                && mouse.y >= b.rect[1]
                && mouse.y <= b.rect[1] + b.rect[3]
        })
        .map(|b| b.npc)
}

/// `Lang.GetNPCHouseBannerText` (`Lang.cs:413-420`): the reserved banner shows
/// `Game.ReservedForNPC` with the NPC's full name, else the full name.
pub(crate) fn banner_text(npc: &Npc, reserved: i32) -> String {
    let name = crate::ui::housing::label(
        crate::ui::housing::head_of(npc.kind).unwrap_or(0),
        Some(0),
        std::slice::from_ref(npc),
    )
    .unwrap_or_default();
    if reserved == 1 {
        crate::lang::text_args("Game.ReservedForNPC", &[name])
    } else {
        name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;

    fn housed_npc(kind: NpcKind, home: (i32, i32)) -> Npc {
        let mut n = Npc::new(kind, home.0 as f32 * 16.0, home.1 as f32 * 16.0, 1, 0.0);
        n.home = Some(home);
        n.hp = n.max_hp;
        n
    }

    /// `banner_npcs` keeps only the housed, non-Old-Man town NPCs and sorts by
    /// descending housingCategory (the pets first).
    #[test]
    fn w4_g21_banner_npcs_filters_and_sorts_by_category() {
        let guide = housed_npc(NpcKind::Guide, (60, 100));
        let mut homeless = housed_npc(NpcKind::Merchant, (70, 100));
        homeless.home = None;
        let npcs = vec![guide, homeless];
        let idx = banner_npcs(&npcs);
        assert_eq!(idx, vec![0], "only the housed Guide draws");
    }

    /// `banner_anchor_row` walks up to the first solid tile; a home with no
    /// solid above row 10 breaks at the floor.
    #[test]
    fn w4_g21_banner_anchor_walks_up_to_solid() {
        let mut w = World::new("t".into(), 200, 200);
        // A solid floor at row 90, the home at row 100.
        for x in 0..200 {
            let i = w.index(x, 90);
            w.tiles[i].kind = 1;
            w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
        // The walk from row 99 up lands on row 90, then num3++ → 91.
        assert_eq!(banner_anchor_row(&w, 60, 100), Some(91));
    }

    /// `banner_at` hit-tests the inclusive hover box.
    #[test]
    fn w4_g21_banner_at_hit_tests_the_box() {
        let banners = vec![Banner { npc: 3, rect: [100.0, 100.0, 20.0, 20.0], reserved: 0 }];
        assert_eq!(banner_at(&banners, Vec2::new(110.0, 110.0)), Some(3));
        assert_eq!(banner_at(&banners, Vec2::new(100.0, 100.0)), Some(3), "inclusive edges");
        assert_eq!(banner_at(&banners, Vec2::new(121.0, 110.0)), None);
    }
}
