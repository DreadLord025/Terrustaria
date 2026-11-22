//! The reforge menu's draw block (`Main.DrawInventory`'s inline arm,
//! `Main.cs:41581-41679`, W4-g57): the (50, 270) context-5 input slot, the
//! `Lang.inter[46]` label + the coin breakdown, the (120, 310) hammer button
//! at `Main.reforgeScale` (`TextureAssets.Reforge[hover ? 1 : 0]`) and the
//! hovered button's `Lang.inter[19]` name. The hover-scale animation itself
//! runs in `App::update_reforge_menu` (`Main.cs:41559-41572`); this module
//! only draws. Visual only — no RNG.
//!
//! Documented skips (ANALYSIS, W4-g57): `ItemSlot.DrawSavings` (`:41637`,
//! the piggy-bank/savings row) has no port surface yet; the button's
//! `UILinkPointNavigator.SetPosition` (`:41648`) is gamepad-only.

use glam::Vec2;
use tr_render::SpriteRenderer;

use super::{inventory, slot, text};
use crate::assets;
use crate::item::ItemSlot;

/// The block's literal anchors (`Main.cs:41581-41582`, `:41639-41640`) —
/// neither rides `Main.mH`.
const SLOT_X: f32 = 50.0;
const SLOT_Y: f32 = 270.0;
const BUTTON_CX: f32 = 120.0; // num59 = num51 + 70
const BUTTON_CY: f32 = 310.0; // num60 = num52 + 40
/// `Main.inventoryScale = 0.85` restored at `Main.cs:41526` before the block.
const SCALE: f32 = 0.85;
/// `Main.inventoryBack`'s port stand-in (the `ui::inventory` convention).
const BACK_TINT: [f32; 4] = [220.0 / 255.0; 4];

/// `Colors.CoinPlatinum/Gold/Silver/Copper` (`Colors.cs:108-111`).
const COIN_COLORS: [[u8; 4]; 4] = [
    [220, 220, 198, 255],
    [224, 201, 92, 255],
    [181, 192, 193, 255],
    [246, 138, 96, 255],
];

/// The coin breakdown (`Main.cs:41593-41636`): the floor-at-1 cost split
/// into platinum/gold/silver/copper counts (`:41598-41620`), each rendered
/// `[c/hex:count Lang.inter[15+i]] ` (`:41621-41636`). The hex is
/// `Colors.AlphaDarken(Coin*).Hex3()` — the coin colour ×
/// `mouseTextColor/255` (`Colors.cs:84-87`), XNA byte truncation, lowercase
/// (`Utils.cs:2000-2003`'s `.ToLower()`).
pub(crate) fn coin_breakdown(cost: u64, mouse_text_color: u8) -> String {
    let mut num58 = (cost as i64).max(1); // `:41598-41601`
    let mut counts = [0i64; 4];
    for (i, unit) in [1_000_000i64, 10_000, 100, 1].iter().enumerate() {
        if num58 >= *unit {
            counts[i] = num58 / unit;
            num58 -= counts[i] * unit;
        }
    }
    let f = mouse_text_color as f32 / 255.0;
    let mut out = String::new();
    for i in 0..4 {
        if counts[i] > 0 {
            let c = super::color::mul(COIN_COLORS[i], f); // AlphaDarken
            out.push_str(&format!(
                "[c/{:02x}{:02x}{:02x}:{} {}] ",
                c[0],
                c[1],
                c[2],
                counts[i],
                crate::lang::inter(15 + i),
            ));
        }
    }
    out
}

/// The hovered button's name (`Main.cs:41651`: `Main.hoverItemName =
/// Lang.inter[19].Value`) — `Some` only while the button draws (the slot
/// holds an item) and the strict 30×30 hit box holds. The caller prints it
/// through the pending-mouse-text layer.
pub(crate) fn button_hover_name(slot: &ItemSlot, mouse: Vec2) -> Option<String> {
    (slot.kind > 0 && inventory::reforge_button_hit(mouse))
        .then(|| crate::lang::inter(19))
}

/// `ChatManager.DrawColorCodedStringWithShadow` (`ChatManager.cs:333-353`)
/// over the breakdown's `[c/HEX:text] ` snippets: four black-shadow copies
/// at ±1.5 px then the fill, each snippet in its own colour, the gaps in
/// the base colour — the `ui::chest` shadow convention with the snippet
/// split the chat log does not need yet.
fn draw_coded(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    x: f32,
    y: f32,
    base: [f32; 4],
    s: &str,
) {
    let mut cx = x;
    let mut rest = s;
    while !rest.is_empty() {
        let (chunk, color, next) = if let Some(tail) = rest.strip_prefix("[c/") {
            match tail.split_once(':') {
                Some((hex, after)) if hex.len() == 6 => {
                    let end = after.find(']').unwrap_or(after.len());
                    let rgb = |i: usize| {
                        u8::from_str_radix(&hex[i..i + 2], 16).unwrap_or(255) as f32 / 255.0
                    };
                    (
                        &after[..end],
                        [rgb(0), rgb(2), rgb(4), base[3]],
                        &after[(end + 1).min(after.len())..],
                    )
                }
                _ => (rest, base, ""),
            }
        } else {
            let end = rest.find("[c/").unwrap_or(rest.len());
            (&rest[..end], base, &rest[end..])
        };
        if !chunk.is_empty() {
            let shadow = [0.0, 0.0, 0.0, color[3]];
            for (dx, dy) in [(-1.0f32, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0)] {
                text::hud_text(r, lookups, cx + dx * 1.5, y + dy * 1.5, 1.0, shadow, chunk);
            }
            text::hud_text(r, lookups, cx, y, 1.0, color, chunk);
            cx += text::hud_text_width(r, lookups, chunk, 1.0);
        }
        rest = next;
    }
}

/// The block itself (`Main.cs:41581-41679`). `reforge_scale` is
/// `Main.reforgeScale`, `mouse_reforge` the resolved hover latch
/// (`Main.cs:41641`'s `num61` — the port resolves it in
/// `App::update_reforge_menu` from the same hit box).
pub(crate) fn draw(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    slot_item: &ItemSlot,
    cost: u64,
    reforge_scale: f32,
    mouse_reforge: bool,
    mouse_text_color: u8,
) {
    // `:41583` — `text = Lang.inter[46].Value + ": "` ("Reforge: ").
    let mut label = format!("{}: ", crate::lang::inter(46));
    if slot_item.kind > 0 {
        // `:41638` — the coin breakdown right of the label, white base.
        let breakdown = coin_breakdown(cost, mouse_text_color);
        let lx = SLOT_X + 50.0 + text::hud_text_width(r, lookups, &label, 1.0);
        draw_coded(r, lookups, lx, SLOT_Y, [1.0; 4], &breakdown);
        // `:41642-41647` — the hammer button: `[1]` while hovered, centred
        // on (120, 310) (origin `Size()/2`) at `Main.reforgeScale`.
        let sheet = lookups.ui_reforge[usize::from(mouse_reforge)];
        if let Some(sheet) = sheet {
            let (w, h) = (sheet.width as f32, sheet.height as f32);
            super::push_whole(
                r,
                lookups,
                &sheet,
                [
                    BUTTON_CX - w * 0.5 * reforge_scale,
                    BUTTON_CY - h * 0.5 * reforge_scale,
                    w * reforge_scale,
                    h * reforge_scale,
                ],
                [1.0; 4],
            );
        }
    } else {
        label = crate::lang::inter(20); // `:41670` — "place an item here"
    }
    // `:41672` — the label at (100, 270) in `Color(mouseTextColor ×4)`.
    let t = mouse_text_color as f32 / 255.0;
    draw_coded(r, lookups, SLOT_X + 50.0, SLOT_Y, [t, t, t, t], &label);
    // `:41679` — the context-5 slot (`Inventory_Back4`, `ItemSlot.cs:2222`).
    slot::draw_slot(
        r, lookups, slot_item, SLOT_X, SLOT_Y, 5, 0, false, SCALE, BACK_TINT,
        super::UiPulse { text: mouse_text_color as f32 / 255.0, ..Default::default() },
        None, -1, true, false,
    );
}

// --- witnesses (W4-g57) — WRITTEN NOT RUN (code-only swarm, no compiler) ---
#[cfg(test)]
mod tests {
    use super::*;

    /// `Main.cs:41598-41636`: 0 floors to 1 copper; the split walks
    /// platinum → gold → silver → copper; each row carries its coin colour
    /// (AlphaDarken at 255 = the raw colour) and the `Lang.inter[15+i]` row.
    #[test]
    fn the_coin_breakdown_matches_the_csharp_split() {
        let s = coin_breakdown(0, 255);
        assert!(s.starts_with("[c/f68a60:1 "), "0 floors to 1 copper: {s}");
        let s = coin_breakdown(1_234_567, 255);
        assert!(s.contains("[c/dcdcc6:1 "), "1 platinum: {s}");
        assert!(s.contains("[c/e0c95c:23 "), "23 gold: {s}");
        assert!(s.contains("[c/b5c0c1:45 "), "45 silver: {s}");
        assert!(s.contains("[c/f68a60:67 "), "67 copper: {s}");
        // The row order is platinum, gold, silver, copper (`:41621-41636`).
        let p = s.find("dcdcc6").unwrap();
        let g = s.find("e0c95c").unwrap();
        let si = s.find("b5c0c1").unwrap();
        let c = s.find("f68a60").unwrap();
        assert!(p < g && g < si && si < c);
        // A zero row prints nothing (`:41621`'s `> 0` gates).
        let s = coin_breakdown(100, 255);
        assert!(!s.contains("dcdcc6") && !s.contains("e0c95c"));
        assert!(s.contains("[c/b5c0c1:1 "));
    }

    /// `Colors.AlphaDarken` (`Colors.cs:84-87`): ×190/255 truncates —
    /// copper (246,138,96) → (183,102,71).
    #[test]
    fn the_breakdown_darkens_by_the_mouse_text_clock() {
        let s = coin_breakdown(1, 190);
        assert!(s.starts_with("[c/b76647:1 "), "AlphaDarken at 190: {s}");
    }

    /// `Main.cs:41651`: the hover name is `Lang.inter[19]`, only with an
    /// item in the slot and the pointer strictly inside (105,135)×(295,325).
    #[test]
    fn the_hover_name_gates_on_the_item_and_the_strict_box() {
        let held = ItemSlot { kind: 1, name: "x", count: 1, prefix: 0 };
        assert!(button_hover_name(&held, Vec2::new(120.0, 310.0)).is_some());
        assert!(button_hover_name(&held, Vec2::new(105.0, 310.0)).is_none());
        assert!(button_hover_name(&held, Vec2::new(500.0, 500.0)).is_none());
        assert!(button_hover_name(&crate::item::EMPTY_SLOT, Vec2::new(120.0, 310.0)).is_none());
    }
}
