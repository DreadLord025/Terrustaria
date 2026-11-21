//! `Main.DrawInterface_40_InteractItemIcon` (`Main.cs:45038-45115`), the
//! cursor item-icon pass (W4-g79 phase 5): the held tool's icon beside the
//! pointer while a tile-interaction gate arms `Player.cursorItemIconEnabled`
//! (the paint-tool gates, `world_sim/editing/paint.rs`), plus the paint
//! preview riding it — `Main.TryGetAmmo`'s brush/roller arm
//! (`Main.cs:45117-45136`) through `item::paint_tool_preview` (the
//! `FindPaintOrCoating` pick at the (22,22)/(28,28) offsets, ×0.8 scale).
//!
//! Recorded deviations (ANALYSIS "Tile paint use paths (W4-g79)"):
//! * `Main.HoveringOverAnNPC` / `Player.mouseInterface` (`:45039`) have no
//!   ported fields — the suppression is unported; the pass draws whenever
//!   the enable flag rides.
//! * `cursorItemIconID`/`Text`/`Push`/`Reversed` (`Player.cs:1852-1858`)
//!   have no ported setters yet (the tile-interaction icon arms are
//!   unported), so the drawn kind is always the held/biome-converted kind,
//!   the push is 0, the text arm (`:45102-45105`) and the rarity-colour
//!   second draw (`:45106-45110`, `item.color`) never fire.
//! * The `_itemIconCacheTime` linger (`ItemIconCacheUpdate`/`Verification`,
//!   `Main.cs:46515-46540`) has no ported writer — the gate reduces to the
//!   `flag` half (`:45061-45066`).
//! * `Item.GetAlpha`'s spectre gFade arm (`Item.cs:49246-49252`) needs
//!   `Main.gFade` (unported); every tint here is `GetAlpha`'s default arm
//!   at alpha 0 — white (`Item.cs:49324-49339`). `ItemSlot.GetItemLight`'s
//!   disco/pulse arms (`ItemSlot.cs:3760-3800`) need the essScale/disco
//!   feeds the HUD pass does not carry — white stands, like the slot draws.
//! * `Item.GetDrawHitbox`'s animation-frame arms (`Item.cs:49595-49632`)
//!   are unported — the icon rect is the item's own sheet, the
//!   `slot::draw_item_icon` precedent.

use tr_render::SpriteRenderer;

use crate::assets;
use crate::inv;
use crate::item::{INV, ItemSlot};
use crate::player::Player;

use super::slot;

/// The pass's drawn kind (`Main.cs:45043-45060`): the held item's type,
/// biome-converted while `UsingBiomeTorches` — the torch 8 through
/// `BiomeTorchHoldStyle` (`:45050`), the campfire 966 through
/// `BiomeCampfireHoldStyle` (`:45053`). The `cursorItemIconID != 0`
/// override (`:45057-45060`) has no ported setter (module note).
pub(crate) fn interact_icon_kind(
    held_kind: u16,
    player: &Player,
    world: &tr_core::world::World,
) -> u16 {
    if !player.using_biome_torches() {
        return held_kind;
    }
    match held_kind {
        8 => crate::player_lights::biome_torch_hold_style(
            held_kind,
            true,
            player.zones,
            player.y,
            world,
        ),
        966 => crate::player_lights::biome_campfire_hold_style(
            held_kind,
            true,
            player.zones,
            player.y,
            world,
        ),
        _ => held_kind,
    }
}

/// The `flag` gate (`Main.cs:45061`): `cursorItemIconEnabled && num != 0 &&
/// Main.MouseDisplayItem.IsAir`. `MouseDisplayItem` falls back to
/// `Main.mouseItem` while `FakeCursorItem` is air (`Main.cs:2955-2965`) —
/// the fake cursor is unported, so the mouse item is the whole read.
pub(crate) fn interact_icon_gate(enabled: bool, kind: u16, mouse_item: &ItemSlot) -> bool {
    let mouse_air = mouse_item.kind == 0 || mouse_item.count == 0;
    enabled && kind != 0 && mouse_air
}

/// The main icon's centre (`Main.cs:45089-45092`): `(mouseX + num3,
/// mouseY + num3) + GetDrawHitbox(num).Size() * cursorScale * 0.5`, where
/// `num3 = 10 + cursorItemIconPush` — the push has no ported setter, 0.
pub(crate) fn icon_center(
    mouse: [f32; 2],
    icon_w: f32,
    icon_h: f32,
    cursor_scale: f32,
) -> (f32, f32) {
    (
        mouse[0] + 10.0 + icon_w * cursor_scale * 0.5,
        mouse[1] + 10.0 + icon_h * cursor_scale * 0.5,
    )
}

/// The paint preview's top-left + size (`Main.cs:45095-45099`): the raw
/// `spriteBatch.Draw` at `(mouseX + 10, mouseY + 10) + ammoOffset` with
/// `scale = cursorScale * ammoScale` (0.8, `TryGetAmmo`'s out-default
/// `:45121`) and the DEFAULT origin — the position is the top-left, not a
/// centre (the main icon's `DrawItemIcon` centres; this arm does not).
pub(crate) fn preview_rect(
    mouse: [f32; 2],
    offset: [f32; 2],
    icon_w: f32,
    icon_h: f32,
    cursor_scale: f32,
) -> ([f32; 2], [f32; 2]) {
    let scale = cursor_scale * 0.8;
    (
        [mouse[0] + 10.0 + offset[0], mouse[1] + 10.0 + offset[1]],
        [icon_w * scale, icon_h * scale],
    )
}

/// The pass itself — the layer-40 draw (`Main.cs:45038-45115`), called
/// from `draw_hud` after the mouse-item layer (38) like the C#'s layer
/// order (`:44899-44913`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    inventory: &[ItemSlot; INV],
    equip: &inv::EquipSlots,
    selected: usize,
    player: Option<&Player>,
    world: &tr_core::world::World,
    mouse_item: &ItemSlot,
    mouse: [f32; 2],
    cursor_scale: f32,
) {
    let Some(p) = player else {
        return;
    };
    let held = inventory[selected];
    let kind = interact_icon_kind(held.kind, p, world);
    if !interact_icon_gate(p.cursor_item_icon_enabled, kind, mouse_item) {
        return;
    }
    // `:45072-45081` — the tint chain reduces to white (module note: the
    // gFade/GetItemLight arms are unported, the icon-ID override never
    // fires). `:45087-45092` — the main icon at cursorScale, sizeLimit
    // 9999 (no clamp).
    if kind > 0 {
        if let Some((_, iw, ih)) = slot::item_icon_rect(lookups, kind) {
            let center = icon_center(mouse, iw, ih, cursor_scale);
            let icon = ItemSlot { kind, ..held };
            slot::draw_item_icon(r, lookups, &icon, center, cursor_scale, [1.0; 4], 9999.0);
        }
        // `:45093-45100` — the ammo preview, only while the drawn kind IS
        // the held kind (the icon-ID override would skip it): the paint
        // tools' `FindPaintOrCoating` pick at the verbatim offsets.
        if kind == held.kind {
            if let Some((ammo, offset)) = crate::item::paint_tool_preview(held.kind, inventory, equip)
            {
                if let Some((uv, iw, ih)) = slot::item_icon_rect(lookups, ammo.kind) {
                    let (pos, size) = preview_rect(mouse, offset, iw, ih, cursor_scale);
                    // `ammoItem.GetAlpha(ammoColor)` — white at alpha 0
                    // (`Item.cs:49324-49339`, the previewed paint takes the
                    // default arm).
                    r.push_ui_sprite(pos, size, uv, [1.0; 4]);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::EMPTY_SLOT;

    fn player() -> Player {
        Player::spawn(&tr_core::world::World::new("t".into(), 64, 800))
    }

    /// The gate (`Main.cs:45061`): all three conjuncts — the enable flag,
    /// a non-zero kind, an air mouse item.
    #[test]
    fn w4_g79_the_icon_gate_needs_flag_kind_and_an_air_cursor() {
        let air = EMPTY_SLOT;
        let carried = ItemSlot { kind: 9, count: 3, ..EMPTY_SLOT };
        assert!(interact_icon_gate(true, 1071, &air));
        assert!(!interact_icon_gate(false, 1071, &air), "no flag");
        assert!(!interact_icon_gate(true, 0, &air), "air held item");
        assert!(!interact_icon_gate(true, 1071, &carried), "carrying");
        // A zero-count stack is air too (`Item.IsAir`).
        let empty_stack = ItemSlot { kind: 9, count: 0, ..EMPTY_SLOT };
        assert!(interact_icon_gate(true, 1071, &empty_stack));
    }

    /// The biome conversion (`Main.cs:45045-45056`): the torch/campfire
    /// kinds convert under `UsingBiomeTorches`, everything else passes.
    #[test]
    fn w4_g79_the_icon_kind_converts_only_the_torch_and_campfire() {
        let w = tr_core::world::World::new("t".into(), 64, 800);
        let mut p = player();
        p.y = 176.0;
        // No unlock: both stay plain.
        assert_eq!(interact_icon_kind(8, &p, &w), 8);
        assert_eq!(interact_icon_kind(966, &p, &w), 966);
        p.unlocked_biome_torches = true;
        p.zones.snow = true;
        assert_eq!(interact_icon_kind(8, &p, &w), 974, "the Ice torch");
        assert_eq!(interact_icon_kind(966, &p, &w), 3048, "the Snow campfire");
        // A paint tool never converts.
        assert_eq!(interact_icon_kind(1071, &p, &w), 1071);
        // The builder toggle off restores the plain kinds.
        p.builder_acc_status[11] = 1;
        assert_eq!(interact_icon_kind(8, &p, &w), 8);
        assert_eq!(interact_icon_kind(966, &p, &w), 966);
    }

    /// The main icon's centre (`Main.cs:45089-45092`): mouse + 10 + half
    /// the scaled hitbox; the preview's top-left (`:45099`): mouse + 10 +
    /// the verbatim offset, ×0.8 scale, NO centring.
    #[test]
    fn w4_g79_the_icon_centres_and_the_preview_tops_left_at_08() {
        let mouse = [100.0, 200.0];
        // A 24x24 brush icon at cursorScale 1: centre (122, 222).
        assert_eq!(icon_center(mouse, 24.0, 24.0, 1.0), (122.0, 222.0));
        // The brush preview offset (22, 22): top-left (132, 232), the
        // 24x24 paint icon at 0.8 → 19.2 px.
        let (pos, size) = preview_rect(mouse, [22.0, 22.0], 24.0, 24.0, 1.0);
        assert_eq!(pos, [132.0, 232.0]);
        assert!((size[0] - 19.2).abs() < 1e-5 && (size[1] - 19.2).abs() < 1e-5);
        // The roller offset (28, 28) at cursorScale 1.1: scale 0.88.
        let (pos, size) = preview_rect(mouse, [28.0, 28.0], 24.0, 24.0, 1.1);
        assert_eq!(pos, [138.0, 238.0]);
        assert!((size[0] - 24.0 * 0.88).abs() < 1e-5);
    }

    /// The end-to-end pick: the brush + a stocked Red Paint draws the
    /// preview record (`TryGetAmmo`'s arm through `paint_tool_preview`),
    /// the scraper never does (`Main.cs:45124` lists only the four
    /// brush/roller ids).
    #[test]
    fn w4_g79_the_preview_pick_rides_the_brush_and_roller_only() {
        let mut inv = [EMPTY_SLOT; INV];
        let equip = inv::EquipSlots::default();
        inv[0] = ItemSlot { kind: 1073, count: 5, ..EMPTY_SLOT }; // Red Paint
        let brush = crate::item::paint_tool_preview(1071, &inv, &equip);
        assert_eq!(brush.map(|(s, o)| (s.kind, o)), Some((1073, [22.0, 22.0])));
        let roller = crate::item::paint_tool_preview(1544, &inv, &equip);
        assert_eq!(roller.map(|(s, o)| (s.kind, o)), Some((1073, [28.0, 28.0])));
        assert!(crate::item::paint_tool_preview(1100, &inv, &equip).is_none());
        // No paint stocked: no preview.
        let empty = [EMPTY_SLOT; INV];
        assert!(crate::item::paint_tool_preview(1071, &empty, &equip).is_none());
    }
}
