//! W4-d57 — the tile-entity furniture draw arms: the item frame (395), the
//! food platter (520) and the weapons rack (471) `DrawSpecialTilesLegacy`
//! arms (`TileDrawing.cs:7951-8000`), their `CacheSpecialDraws_Part1` anchor
//! collection (`TileDrawing.cs:4154-4247`, the display doll 470 and the hat
//! rack 475 included), and the `ItemSlot.DrawItemIcon` /
//! `DrawItem_GetColorAndScale` / `GetItemLight` colour-and-scale chain
//! (`ItemSlot.cs:2848-2890`, `:2961-2976`, `:3760-3801`) with `Item.GetAlpha`
//! / `Item.GetColor` (`Item.cs:49170-49383`).
//!
//! **UNWIRED module (code-only mode).** The draw-pass seam
//! (`render/tiles.rs::draw_tiles` + `app/frame/draw.rs`) is contested by the
//! in-flight tile-draw siblings (d58 chest frames, d59 platform slopes, d60
//! particles), so this slice lands as pure draw/decision functions with the
//! wiring deferred to the family's merge step — the `special_tiles.rs`
//! (S1-G-1e) pattern. Intended call sites:
//! * [`collect_te_points`] — the `CacheSpecialDraws_Part1` legacy-point
//!   collection over the view rect (`TileDrawing.cs:4152-4247`); the C#
//!   caches the `TileEntityType<T>.Find` result per anchor point in five
//!   dictionaries (`:82-94`), the port dedupes per kind in first-seen order.
//! * [`item_frame_icon`] / [`food_platter_draw`] / [`weapons_rack_te_icon`]
//!   — the `DrawSpecialTilesLegacy` arms (`TileDrawing.cs:7951-8000`), fed
//!   by the collected points; the 334 weapon-rack DISPLAY decode stays in
//!   `special_tiles.rs` (S1-G-1e) — this module's 471 arm is the TE lookup.
//! * The display-doll / hat-rack player draws (`DrawEntities_DisplayDolls` /
//!   `DrawEntities_HatRacks`, `TileDrawing.cs:8026-8050`, the
//!   `PostDrawTiles(solidLayer: true)` half `:435-441`) ride the phase-2
//!   decision ports below; the `LegacyPlayerRenderer.DrawPlayer` call itself
//!   is the wiring step.
//!
//! Deferrals (recorded in docs/ANALYSIS.md "The TE furniture draw (W4-d57)"):
//! * `Main.itemAnimations[type]` is unported — the icon source rect is the
//!   whole item texture (the `ui/slot.rs::draw_item_icon` stand-in); the
//!   5644 spectate `frameCounterOverride` arm (`ItemSlot.cs:2860-2863`)
//!   rides the animator.
//! * `TeItem` carries only the wire triple (kind/prefix/stack): `item.alpha`
//!   and `item.color` are not in the generated `ItemDef`, so the callers
//!   pass them in (0 / transparent stand-ins until the item-runtime fields
//!   land); the prefix scale resolve rides the wiring step like the 334 arm.
//! * The `DrawItemIcon` glowmask / TrapSigned / DrawUnsafeIndicator tails
//!   (`ItemSlot.cs:2891-2950`) need the glowmask atlas + the wire sheet —
//!   the base + `item.color` second pass are ported here.
//! * `Main.gFade` (the `GetAlpha` 1446/1506/1507/1543-1545 arm) is unported;
//!   the caller passes the byte (0 stand-in).

#![allow(dead_code)] // UNWIRED — see the module doc; wiring is the family merge step.

use std::collections::HashSet;

use tr_core::tile_entities::{TeItem, TeKind};
use tr_core::world::World;

use crate::projectile::{get_lerp_value, proj_frames};
use crate::projectile_ai_spear::spear_offset;
use crate::render::special_tiles::xna_scale_color;

// ---------------------------------------------------------------------------
// CacheSpecialDraws_Part1 — the TE anchor points (TileDrawing.cs:4154-4247)
// ---------------------------------------------------------------------------

/// The Part1 anchor decode for the five furniture TE tiles: the special
/// point is the entity's top-left cell, stepped back off the frame grid.
/// `None` for every other tile type. The frame inputs are the DRAW DATA's
/// `tileFrameX/Y` (`TileDrawing.cs:4152`); none of these five kinds animates
/// its frame, so the tile's own `frame_x/frame_y` feed it.
pub(crate) fn te_anchor(
    tile_kind: u16,
    x: i32,
    y: i32,
    frame_x: i16,
    frame_y: i16,
) -> Option<(TeKind, i32, i32)> {
    let (fx, fy) = (frame_x as i32, frame_y as i32);
    match tile_kind {
        // TileDrawing.cs:4154-4173 — the 2x2 item frame, 36 px grid.
        395 => Some((
            TeKind::ItemFrame,
            x - i32::from(fx % 36 != 0),
            y - i32::from(fy % 36 != 0),
        )),
        // TileDrawing.cs:4194-4205 — the 1x1 food platter, no step.
        520 => Some((TeKind::FoodPlatter, x, y)),
        // TileDrawing.cs:4206-4219 — the 3x3 weapons rack, 54 px grid.
        471 => Some((TeKind::WeaponsRack, x - fx % 54 / 18, y - fy % 54 / 18)),
        // TileDrawing.cs:4220-4233 — the 2x3 display doll, 36x54 grid.
        470 => Some((TeKind::DisplayDoll, x - fx % 36 / 18, y - fy % 54 / 18)),
        // TileDrawing.cs:4234-4247 — the 3x4 hat rack, 54x72 grid.
        475 => Some((TeKind::HatRack, x - fx % 54 / 18, y - fy % 72 / 18)),
        _ => None,
    }
}

/// The Part1 collection over the view rect: every furniture cell resolves
/// its anchor, the per-kind dictionary dedupes (`ContainsKey`,
/// `TileDrawing.cs:4165` — a MISS is cached too), and only a
/// `TileEntityType<T>.Find != -1` hit adds the special legacy point
/// (`AddSpecialLegacyPoint`, `:4170`). First-seen row-major order is the
/// dictionary insertion order the `DrawEntities_*` walks iterate.
pub(crate) fn collect_te_points(
    world: &World,
    x0: i32,
    x1: i32,
    y0: i32,
    y1: i32,
) -> Vec<(TeKind, i32, i32)> {
    let mut seen: [HashSet<(i32, i32)>; 5] = Default::default();
    let mut out = Vec::new();
    for ty in y0..=y1 {
        for tx in x0..=x1 {
            let Some(tile) = world.tile(tx, ty).filter(|t| t.active()) else {
                continue;
            };
            let Some((kind, ax, ay)) = te_anchor(tile.kind, tx, ty, tile.frame_x, tile.frame_y)
            else {
                continue;
            };
            let slot = match kind {
                TeKind::ItemFrame => 0,
                TeKind::FoodPlatter => 1,
                TeKind::WeaponsRack => 2,
                TeKind::DisplayDoll => 3,
                TeKind::HatRack => 4,
                _ => unreachable!(),
            };
            if !seen[slot].insert((ax, ay)) {
                continue;
            }
            // `TileEntityType<T>.Find` — the position book hit of THIS type.
            if world
                .tile_entities
                .get_at(ax as i16, ay as i16)
                .is_some_and(|e| e.kind == kind)
            {
                out.push((kind, ax, ay));
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------
// ItemSlot.GetItemLight (ItemSlot.cs:3760-3801)
// ---------------------------------------------------------------------------

/// `ItemID.Sets.ItemIconPulse` (`ID/ItemID.cs:1319`).
const ITEM_ICON_PULSE: [u16; 12] = [520, 521, 575, 549, 548, 547, 3456, 3457, 3458, 3459, 3580, 3581];

/// `GetItemLight`'s disco arms (`ItemSlot.cs:3766-3781`): the rainbow
/// glowsticks 662/663/5444/5450/5643 and the Rainbow Cursor 5128 overwrite
/// RGB with `Main.DiscoR/G/B` and force alpha 255.
fn is_disco_item(kind: u16) -> bool {
    matches!(kind, 662 | 663 | 5444 | 5450 | 5643 | 5128)
}

/// `ItemSlot.GetItemLight(ref Color, ref float, int, bool, float)`
/// (`ItemSlot.cs:3760-3801`), `outInTheWorld = false`, `lightScalar = 1`
/// (every furniture call site). Returns the tinted colour and the pulse
/// scale; the per-channel `(byte)(channel * scale)` truncation is the XNA
/// float multiply ([`xna_scale_color`]). The `type > ItemID.Count` guard
/// (`:3762`, Count = 6196, `ID/ItemID.cs:14861`) passes the colour through.
pub(crate) fn get_item_light(
    color: [u8; 4],
    kind: u16,
    ess_scale: f32,
    disco: [u8; 3],
) -> ([u8; 4], f32) {
    if kind as i32 > 6196 {
        return (color, 1.0);
    }
    if is_disco_item(kind) {
        // `:3766-3781`: RGB = disco, A = 255, then `*= lightScalar` (1).
        return ([disco[0], disco[1], disco[2], 255], 1.0);
    }
    if ITEM_ICON_PULSE.contains(&kind) {
        // `:3782-3790`: scale = Main.essScale, every channel truncated.
        return (xna_scale_color(color, ess_scale), ess_scale);
    }
    if matches!(kind, 58 | 184 | 4143) {
        // `:3791-3799`: the heart/star/mana pulse — essScale*0.25 + 0.75.
        let scale = ess_scale * 0.25 + 0.75;
        return (xna_scale_color(color, scale), scale);
    }
    (color, 1.0)
}

// ---------------------------------------------------------------------------
// ItemSlot.DrawItem_GetColorAndScale (ItemSlot.cs:2961-2976)
// ---------------------------------------------------------------------------

/// The icon colour-and-scale resolve: `itemLight = currentWhite` tinted by
/// [`get_item_light`], the `sizeLimit` clamp (`width <= height` tie picks
/// the HEIGHT divisor, `:2969`), the Terraspark Boots 5669 override at the
/// 20 px limit (`:2971-2974`), and `finalDrawScale = scale * num * scale2`
/// (`:2975`). `scale2` starts at 1 and only `GetItemLight` writes it.
pub(crate) fn draw_item_color_and_scale(
    kind: u16,
    scale: f32,
    env_color: [u8; 4],
    size_limit: f32,
    frame_w: i32,
    frame_h: i32,
    ess_scale: f32,
    disco: [u8; 3],
) -> ([u8; 4], f32) {
    let (item_light, scale2) = get_item_light(env_color, kind, ess_scale, disco);
    let mut num = 1.0f32;
    if frame_w as f32 > size_limit || frame_h as f32 > size_limit {
        num = if frame_w <= frame_h {
            size_limit / frame_h as f32
        } else {
            size_limit / frame_w as f32
        };
    }
    if kind == 5669 && size_limit == 20.0 {
        num = 0.5;
    }
    (item_light, scale * num * scale2)
}

// ---------------------------------------------------------------------------
// Item.GetAlpha / Item.GetColor (Item.cs:49170-49383)
// ---------------------------------------------------------------------------

/// `ItemID.Sets.BossBag` (`ID/ItemID.cs:1383`).
const BOSS_BAG: [u16; 21] = [
    3318, 3319, 3320, 3321, 3322, 3323, 3324, 3325, 3326, 3327, 3328, 3329, 3330, 3331, 3332,
    3860, 3861, 3862, 4782, 4957, 5111,
];

/// `Color.Lerp(a, b, amount)` — XNA lerps each byte and truncates
/// (`(byte)MathHelper.Lerp`), alpha included.
fn lerp_rgba(a: [u8; 4], b: [u8; 4], amount: f32) -> [u8; 4] {
    let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * amount) as u8;
    [l(a[0], b[0]), l(a[1], b[1]), l(a[2], b[2]), l(a[3], b[3])]
}

/// `Item.GetAlpha(Color)` (`Item.cs:49170-49342`): the per-type self-lit /
/// ghost tint over the environment light. `alpha` is `item.alpha`,
/// `g_fade` is `Main.gFade` (the 1446/1506/1507/1543-1545 ghost arm) and
/// `disco` is `Main.DiscoR/G/B` (the 5128 shimmer-bait floor) — the callers
/// pass the runtime values (0 / [0,0,0] stand-ins, see the module doc).
pub(crate) fn item_get_alpha(
    kind: u16,
    alpha: i32,
    g_fade: u8,
    disco: [u8; 3],
    new_color: [u8; 4],
) -> [u8; 4] {
    if BOSS_BAG.contains(&kind) {
        // `:49172-49175`: Lerp(newColor, White, 0.4).
        return lerp_rgba(new_color, [255; 4], 0.4);
    }
    match kind {
        // `:49178-49180`: the Rod of Discord / Shellphone lerp.
        1326 | 5335 => lerp_rgba(new_color, [255; 4], 0.75),
        // `:49181-49186`: white RGB, alpha minus item.alpha (byte wrap).
        5043 | 3065 | 4956 | 5669 => [255, 255, 255, (new_color[3] as i32 - alpha) as u8],
        // `:49187-49201`: full white (the star/gel/mana self-lits).
        75 | 671 | 3858 | 6155 | 6173 | 119 | 120 | 121 | 122 | 217 | 218 | 219 | 220 => [255; 4],
        501 => [200, 200, 200, 50],
        757 | 1306 | 3456 | 3457 | 3458 | 3459 => [255, 255, 255, 200],
        520 | 521 | 522 | 547 | 548 | 549 | 575 | 1332 | 3453 | 3454 | 3455 | 3580 => {
            [255, 255, 255, 50]
        }
        58 | 184 | 1734 | 1735 | 1867 | 1868 => [200, 200, 200, 200],
        1572 => [200, 200, 255, 125],
        787 => [255, 255, 255, 175],
        1826 => [255, 255, 255, 200],
        1508 => [200, 200, 200, 0],
        502 | 4988 => [255, 255, 255, 150],
        51 => [255, 255, 255, 0],
        1260 => [255, 255, 255, 175],
        // `:49246-49252`: the ghost items ride Main.gFade.
        1446 | 1506 | 1507 | 1543 | 1544 | 1545 => {
            [new_color[0], new_color[1], new_color[2], g_fade]
        }
        // `:49253-49272`: the souls/wings/boss relics stay full white.
        198 | 199 | 200 | 201 | 202 | 203 | 3764 | 3765 | 3766 | 3767 | 3768 | 3769 | 4258
        | 4259 | 5535 | 5536 | 5670 | 5671 => [255; 4],
        // `:49273-49282`: the lunar fragments, 250 RGB.
        2763 | 2764 | 2765 | 2782 | 2783 | 2784 | 2785 | 2786 | 3522 => {
            [250, 250, 250, (255 - alpha) as u8]
        }
        3191 => [250, 250, 250, 200],
        // `:49285-49288`: the half-lerp / white-lerp ghost scales.
        3822 => xna_scale_color(lerp_rgba([255; 4], new_color, 0.5), (255 - alpha) as f32 / 255.0),
        4143 => xna_scale_color([255; 4], (255 - alpha) as f32 / 255.0),
        // `:49289-49323`: the shimmer-bait floor colours, per-channel max.
        4354 | 4377 | 4378 | 4389 | 5127 | 5128 => {
            let base = match kind {
                4377 => [50, 255, 50, 200],
                4378 => [50, 200, 255, 255],
                4389 => [255, 50, 125, 200],
                5127 => [150, 50, 250, 200],
                // 5128 reads Main.DiscoR/G/B (:49303), alpha floor 200.
                5128 => [disco[0], disco[1], disco[2], 200],
                _ => [255, 150, 150, 200],
            };
            [
                base[0].max(new_color[0]),
                base[1].max(new_color[1]),
                base[2].max(new_color[2]),
                base[3].max(new_color[3]),
            ]
        }
        // `:49324-49340`: the default — RGB scaled by (255-alpha)/255 with
        // the per-channel int truncation, alpha subtracted and clamped.
        _ => {
            let num = (255 - alpha) as f32 / 255.0;
            let a = (new_color[3] as i32 - alpha).clamp(0, 255) as u8;
            [
                (new_color[0] as f32 * num) as u8,
                (new_color[1] as f32 * num) as u8,
                (new_color[2] as f32 * num) as u8,
                a,
            ]
        }
    }
}

/// `Item.GetColor(Color)` (`Item.cs:49344-49383`): the `item.color` second
/// pass — each channel is `color.C - (255 - newColor.C)` clamped to 0..255.
pub(crate) fn item_get_color(item_color: [u8; 4], new_color: [u8; 4]) -> [u8; 4] {
    let c = |ic: u8, nc: u8| (ic as i32 - (255 - nc as i32)).clamp(0, 255) as u8;
    [
        c(item_color[0], new_color[0]),
        c(item_color[1], new_color[1]),
        c(item_color[2], new_color[2]),
        c(item_color[3], new_color[3]),
    ]
}

// ---------------------------------------------------------------------------
// The DrawSpecialTilesLegacy TE arms (TileDrawing.cs:7951-8000)
// ---------------------------------------------------------------------------

/// One resolved icon draw: the world-pixel centre, the final scale, the
/// `GetAlpha(itemLight)` tint, the horizontal flip and the optional
/// `item.color` second-pass tint (`ItemSlot.cs:2882-2890`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct IconDraw {
    pub center: [f32; 2],
    pub scale: f32,
    pub color: [u8; 4],
    pub flip: bool,
    pub second: Option<[u8; 4]>,
}

/// The shared `DrawItemIcon` resolve (`ItemSlot.cs:2848-2890`, context 40 —
/// none of the 37/13 context arms applies): the colour-and-scale chain, the
/// `GetAlpha` tint, the flip and the `item.color != Transparent` second
/// pass over the UNMODIFIED environment colour (`DrawItem_GetColorAndScale`
/// copies `currentWhite` into `itemLight`, `:2963`). `itemFade = 1` at every
/// furniture site, so the `* itemFade` truncation is a no-op.
fn icon_resolve(
    item: &TeItem,
    item_scale: f32,
    item_alpha: i32,
    item_color: [u8; 4],
    g_fade: u8,
    env: [u8; 4],
    size_limit: f32,
    frame_w: i32,
    frame_h: i32,
    ess_scale: f32,
    disco: [u8; 3],
    center: [f32; 2],
    flip: bool,
) -> IconDraw {
    let (item_light, scale) = draw_item_color_and_scale(
        item.kind, item_scale, env, size_limit, frame_w, frame_h, ess_scale, disco,
    );
    let second = if item_color != [0, 0, 0, 0] {
        Some(item_get_color(item_color, env))
    } else {
        None
    };
    IconDraw {
        center,
        scale,
        color: item_get_alpha(item.kind, item_alpha, g_fade, disco, item_light),
        flip,
        second,
    }
}

/// The 395 item-frame arm (`TileDrawing.cs:7951-7958`):
/// `ItemSlot.DrawItemIcon` centred on `(x*16 + 16, y*16 + 16) + off`,
/// context 40, `scale = item.scale`, `sizeLimit = 20`, no flip. `None` on
/// an air item (`item2.IsAir`, `:7954`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn item_frame_icon(
    item: &TeItem,
    item_scale: f32,
    item_alpha: i32,
    item_color: [u8; 4],
    g_fade: u8,
    env: [u8; 4],
    frame_w: i32,
    frame_h: i32,
    ess_scale: f32,
    disco: [u8; 3],
    x: i32,
    y: i32,
    off: [f32; 2],
) -> Option<IconDraw> {
    if item.is_air() {
        return None;
    }
    let center = [
        (x * 16) as f32 + 16.0 + off[0],
        (y * 16) as f32 + 16.0 + off[1],
    ];
    Some(icon_resolve(
        item, item_scale, item_alpha, item_color, g_fade, env, 20.0, frame_w, frame_h,
        ess_scale, disco, center, false,
    ))
}

/// The 471 weapons-rack TE arm (`TileDrawing.cs:7986-8000`):
/// `ItemSlot.DrawItemIcon` centred on `(x*16 + 24, y*16 + 24) + off`,
/// context 40, `scale = item.scale`, `sizeLimit = 40`, flipped when the
/// anchor's `frameX >= 54` (`:7993-7997`). `None` on an air item.
#[allow(clippy::too_many_arguments)]
pub(crate) fn weapons_rack_te_icon(
    item: &TeItem,
    item_scale: f32,
    item_alpha: i32,
    item_color: [u8; 4],
    g_fade: u8,
    env: [u8; 4],
    frame_w: i32,
    frame_h: i32,
    ess_scale: f32,
    disco: [u8; 3],
    x: i32,
    y: i32,
    frame_x: i16,
    off: [f32; 2],
) -> Option<IconDraw> {
    if item.is_air() {
        return None;
    }
    let center = [
        (x * 16) as f32 + 24.0 + off[0],
        (y * 16) as f32 + 24.0 + off[1],
    ];
    let flip = frame_x >= 54;
    Some(icon_resolve(
        item, item_scale, item_alpha, item_color, g_fade, env, 40.0, frame_w, frame_h,
        ess_scale, disco, center, flip,
    ))
}

/// The 520 food-platter arm's draw (`TileDrawing.cs:7959-7985`): the source
/// frame is `Frame(1, 3, 0, 2)` for food (`ItemID.Sets.IsFood`) else the
/// whole texture; NO size clamp — the scale is `GetItemLight`'s pulse
/// scale only (`:7973-7975`); the origin is the INTEGER `(width/2, height)`
/// (`:7978`); the position is `(x*16 + 8, y*16 + 16) + off` then `Y += 2`
/// (`:7976-7977`); flipped when the anchor's `frameX == 0` (`:7970`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct FoodPlatterDraw {
    /// The draw position (the origin anchor), world pixels.
    pub pos: [f32; 2],
    /// The integer origin `(width/2, height)` in source pixels.
    pub origin: [f32; 2],
    pub scale: f32,
    pub color: [u8; 4],
    pub flip: bool,
    pub second: Option<[u8; 4]>,
    /// `Some((frame_w, frame_h))` — the `Frame(1, 3, 0, 2)` cell size for
    /// food (the bottom third of the sheet); `None` = the whole texture.
    pub food_frame: Option<[i32; 2]>,
}

/// The food-platter resolve. `sheet_w/sheet_h` are the item texture's
/// native size; `is_food` is `ItemID.Sets.IsFood[type]`
/// (`crate::sortdata::IS_FOOD`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn food_platter_draw(
    item: &TeItem,
    item_alpha: i32,
    item_color: [u8; 4],
    g_fade: u8,
    env: [u8; 4],
    sheet_w: i32,
    sheet_h: i32,
    is_food: bool,
    ess_scale: f32,
    disco: [u8; 3],
    x: i32,
    y: i32,
    frame_x: i16,
    off: [f32; 2],
) -> Option<FoodPlatterDraw> {
    if item.is_air() {
        return None;
    }
    // TileDrawing.cs:7966 — `value2.Frame(1, 3, 0, 2)`: Utils.Frame's
    // integer divide (Utils.cs:1344-1349), row 2 of 3.
    let (fw, fh) = if is_food {
        (sheet_w, sheet_h / 3)
    } else {
        (sheet_w, sheet_h)
    };
    // TileDrawing.cs:7971-7975 — GetItemLight over the light colour, the
    // scale starts at 1 and only the pulse arm writes it.
    let (current, scale) = get_item_light(env, item.kind, ess_scale, disco);
    let pos = [
        (x * 16) as f32 + 8.0 + off[0],
        (y * 16) as f32 + 16.0 + off[1] + 2.0,
    ];
    let second = if item_color != [0, 0, 0, 0] {
        // TileDrawing.cs:7980-7983 — item3.GetColor(color2), color2 the
        // UNMODIFIED light colour.
        Some(item_get_color(item_color, env))
    } else {
        None
    };
    Some(FoodPlatterDraw {
        pos,
        origin: [(fw / 2) as f32, fh as f32],
        scale,
        color: item_get_alpha(item.kind, item_alpha, g_fade, disco, current),
        flip: frame_x == 0,
        second,
        food_frame: is_food.then_some([fw, fh]),
    })
}

/// The `item.color` second-pass tint shared by the 334 display arm
/// (`TileDrawing.cs:7946-7949`) — exposed for the wiring step's
/// `push_weapon_rack_item` companion (`special_tiles.rs`).
pub(crate) fn rack_second_pass(item_color: [u8; 4], env: [u8; 4]) -> Option<[u8; 4]> {
    (item_color != [0, 0, 0, 0]).then(|| item_get_color(item_color, env))
}

/// The XNA byte tint → the renderer's float tint (the `special_tiles`
/// convention). Re-exported for the wiring step's push helpers.
pub(crate) fn tint(c: [u8; 4]) -> [f32; 4] {
    [
        c[0] as f32 / 255.0,
        c[1] as f32 / 255.0,
        c[2] as f32 / 255.0,
        c[3] as f32 / 255.0,
    ]
}

// ---------------------------------------------------------------------------
// TEDisplayDoll.Draw — the pose / placement decision (TEDisplayDoll.cs:339-495)
// ---------------------------------------------------------------------------

/// `DisplayDollPoseID` (`DisplayDollPoseID.cs:3-14`): the pose byte's enum.
/// 0-3 are the always-valid poses; 4-8 (Use1-5) need the misc item's
/// useStyle to support them (`IsValidPose`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum DollPose {
    Standing = 0,
    Sitting = 1,
    Jumping = 2,
    Walking = 3,
    Use1 = 4,
    Use2 = 5,
    Use3 = 6,
    Use4 = 7,
    Use5 = 8,
}

impl DollPose {
    fn from_byte(b: u8) -> Option<Self> {
        Some(match b {
            0 => DollPose::Standing,
            1 => DollPose::Sitting,
            2 => DollPose::Jumping,
            3 => DollPose::Walking,
            4 => DollPose::Use1,
            5 => DollPose::Use2,
            6 => DollPose::Use3,
            7 => DollPose::Use4,
            8 => DollPose::Use5,
            _ => return None,
        })
    }
}

/// One `SupportedUseStylePoses` row (`TEDisplayDoll.DisplayDollPose`,
/// `TEDisplayDoll.cs:17-24`): the pose, the `itemAnimation` fraction of the
/// 1000-tick max, and the optional aim radians.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UsePose {
    pub pose: DollPose,
    pub item_anim_percent: f32,
    pub item_aim_radians: Option<f32>,
}

/// `TEDisplayDoll`'s static `SupportedUseStylePoses` table
/// (`TEDisplayDoll.cs:56-83`, the `RegisterUsePose` calls in order). The
/// key is `Item.useStyle`; the aim literals are the C#'s — `0.7853981f`
/// is the written π/4 approximation, `-PI/2`/`-PI/4`/`PI/2` the casts.
pub(crate) fn supported_use_poses(use_style: u8) -> &'static [UsePose] {
    const P: f32 = std::f32::consts::PI;
    match use_style {
        // :56-60 — the swing family, five poses down the use fraction.
        1 => &[
            UsePose { pose: DollPose::Use1, item_anim_percent: 1.0, item_aim_radians: None },
            UsePose { pose: DollPose::Use2, item_anim_percent: 0.8, item_aim_radians: None },
            UsePose { pose: DollPose::Use3, item_anim_percent: 0.6, item_aim_radians: None },
            UsePose { pose: DollPose::Use4, item_anim_percent: 0.4143, item_aim_radians: None },
            UsePose { pose: DollPose::Use5, item_anim_percent: 0.2, item_aim_radians: None },
        ],
        // :61-63, :69-73 — the single-pose use styles.
        7 | 3 | 4 | 6 | 2 | 8 | 9 | 11 | 14 | 15 => &[
            UsePose { pose: DollPose::Use1, item_anim_percent: 0.5, item_aim_radians: None },
        ],
        // :64-68 — the gun aim ladder (useStyle 5).
        5 => &[
            UsePose { pose: DollPose::Use1, item_anim_percent: 0.5, item_aim_radians: Some(-P / 2.0) },
            UsePose { pose: DollPose::Use2, item_anim_percent: 0.5, item_aim_radians: Some(-P / 4.0) },
            UsePose { pose: DollPose::Use3, item_anim_percent: 0.5, item_aim_radians: Some(0.0) },
            UsePose { pose: DollPose::Use4, item_anim_percent: 0.5, item_aim_radians: Some(0.7853981) },
            UsePose { pose: DollPose::Use5, item_anim_percent: 0.5, item_aim_radians: Some(P / 2.0) },
        ],
        // :74-76 — the drink tilt (useStyle 12).
        12 => &[
            UsePose { pose: DollPose::Use1, item_anim_percent: 0.75, item_aim_radians: None },
            UsePose { pose: DollPose::Use2, item_anim_percent: 0.5, item_aim_radians: None },
            UsePose { pose: DollPose::Use3, item_anim_percent: 0.25, item_aim_radians: None },
        ],
        // :77-81 — the spear aim ladder (useStyle 13), same radians as 5.
        13 => &[
            UsePose { pose: DollPose::Use1, item_anim_percent: 0.5, item_aim_radians: Some(-P / 2.0) },
            UsePose { pose: DollPose::Use2, item_anim_percent: 0.5, item_aim_radians: Some(-P / 4.0) },
            UsePose { pose: DollPose::Use3, item_anim_percent: 0.5, item_aim_radians: Some(0.0) },
            UsePose { pose: DollPose::Use4, item_anim_percent: 0.5, item_aim_radians: Some(0.7853981) },
            UsePose { pose: DollPose::Use5, item_anim_percent: 0.5, item_aim_radians: Some(P / 2.0) },
        ],
        _ => &[],
    }
}

/// `IsValidPose` (`TEDisplayDoll.cs:135-155`): poses 0-3 always valid; a
/// Use pose is valid only when the misc item is present and its useStyle
/// supports that exact pose. `misc_use_style` is `None` for an air misc.
pub(crate) fn doll_pose_valid(pose: u8, misc_use_style: Option<u8>) -> bool {
    if pose <= 3 {
        return true;
    }
    let Some(style) = misc_use_style else {
        return false;
    };
    let Some(p) = DollPose::from_byte(pose) else {
        return false;
    };
    supported_use_poses(style).iter().any(|row| row.pose == p)
}

/// The resolved doll draw state — the fields `TEDisplayDoll.Draw` writes
/// onto `_dollPlayer` before `PlayerFrame`/`DrawPlayer`
/// (`TEDisplayDoll.cs:339-495`). The renderer consumes these; the actual
/// `LegacyPlayerRenderer.DrawPlayer` call is the wiring step.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DollDraw {
    /// `dollPlayer.direction` (`:349-355`): -1 default, +1 when the anchor
    /// `frameX % 72 == 36`.
    pub direction: i32,
    /// `dollPlayer.Male` (`:350,356-359`): true default, false when
    /// `frameX / 72 == 1` (the womannequin column).
    pub male: bool,
    /// The effective pose after `IsValidPose` + the mount override
    /// (`:393-405`): an invalid pose falls to Standing; a mounted doll
    /// forces Sitting/Jumping to Standing.
    pub pose: DollPose,
    /// `dollPlayer.velocity` (`:406-452`): Zero for Standing/Sitting/use,
    /// UnitX*direction for Walking, UnitY for Jumping.
    pub velocity: [f32; 2],
    /// `num2` (`:392,419`): the sitting Y lift, 14 when Sitting else 0.
    pub sit_lift: f32,
    /// `dollPlayer.sitting.isSitting` (`:370,418`).
    pub sitting: bool,
    /// `dollPlayer.itemAnimationMax`/`itemAnimation` (`:371-372,437-438`):
    /// 0/0 for the non-use poses, 1000/(int)(1000*percent) for a use pose.
    pub item_animation_max: i32,
    pub item_animation: i32,
    /// `dollPlayer.itemRotation` (`:439-447`): 0 default, the aim radians
    /// (negated when direction == -1) for a use pose with an aim.
    pub item_rotation: f32,
    /// The walking leg/body row (`:413-414`): `legFrame.Height * 9` when
    /// Walking, else the rest row 0. The renderer reads it as the frame Y.
    pub leg_frame_row: u32,
    /// `dollPlayer.position` (`:454`):
    /// `(tileLeftX+1, tileTopY+3)*16 + (-width/2, -height-6+num2)`.
    pub position: [f32; 2],
    /// The mount type from `_equip[8]` (`:373-389`): -1 = no mount.
    pub mount_type: i32,
}

/// `TEDisplayDoll.Draw`'s decision half (`TEDisplayDoll.cs:339-495`).
/// `frame_x` is the anchor tile's `frameX`; `equip8_mount` is
/// `_equip[8].mountType` (-1 when air); `misc_use_style` is the misc item's
/// useStyle (None when air); `width`/`height` are the doll player's hitbox
/// (20/42 stock, grown by the mount). The armor/dye copy, ResetEffects,
/// UpdateDyes, DisplayDollUpdate and the held-projectile preview ride the
/// wiring step — this resolves the pose, velocity, frames and position.
pub(crate) fn display_doll_draw(
    pose_byte: u8,
    frame_x: i16,
    equip8_mount: i32,
    misc_use_style: Option<u8>,
    width: f32,
    height: f32,
    tile_left_x: i32,
    tile_top_y: i32,
) -> DollDraw {
    // :349-359 — direction and gender off the anchor frame.
    let mut direction = -1i32;
    let mut male = true;
    if frame_x as i32 % 72 == 36 {
        direction = 1;
    }
    if frame_x as i32 / 72 == 1 {
        male = false;
    }
    // :393-397 — the pose, invalid falls to Standing.
    let mut pose = if doll_pose_valid(pose_byte, misc_use_style) {
        DollPose::from_byte(pose_byte).unwrap_or(DollPose::Standing)
    } else {
        DollPose::Standing
    };
    // :398-405 — a mounted doll forces Sitting/Jumping to Standing.
    let mounted = equip8_mount != -1;
    if mounted && (pose == DollPose::Sitting || pose == DollPose::Jumping) {
        pose = DollPose::Standing;
    }
    // :406-452 — the per-pose velocity / frames / item animation.
    let mut velocity = [0.0f32, 0.0];
    let mut sit_lift = 0.0f32;
    let mut sitting = false;
    let mut item_animation_max = 0i32;
    let mut item_animation = 0i32;
    let mut item_rotation = 0.0f32;
    let mut leg_frame_row = 0u32;
    match pose {
        DollPose::Standing => {}
        DollPose::Walking => {
            velocity = [direction as f32, 0.0];
            // :413-414 — legFrame.Y = Height*9, bodyFrame.Y follows.
            leg_frame_row = 9;
        }
        DollPose::Sitting => {
            sitting = true;
            sit_lift = 14.0;
        }
        DollPose::Jumping => {
            velocity = [0.0, 1.0];
        }
        _ => {
            // :427-449 — the use pose: find the matching row, set the
            // 1000-tick animation and the aim rotation.
            if let Some(style) = misc_use_style {
                for row in supported_use_poses(style) {
                    if row.pose == pose {
                        item_animation_max = 1000;
                        item_animation = (1000.0 * row.item_anim_percent) as i32;
                        item_rotation = 0.0;
                        if let Some(aim) = row.item_aim_radians {
                            item_rotation = aim;
                            if direction == -1 {
                                item_rotation *= -1.0;
                            }
                        }
                        break;
                    }
                }
            }
        }
    }
    // :454 — position = (tileLeftX+1, tileTopY+3)*16 + (-width/2, -height-6+num2).
    let position = [
        (tile_left_x + 1) as f32 * 16.0 - width / 2.0,
        (tile_top_y + 3) as f32 * 16.0 - height - 6.0 + sit_lift,
    ];
    DollDraw {
        direction,
        male,
        pose,
        velocity,
        sit_lift,
        sitting,
        item_animation_max,
        item_animation,
        item_rotation,
        leg_frame_row,
        position,
        mount_type: equip8_mount,
    }
}

// ---------------------------------------------------------------------------
// TEHatRack.Draw — the two-hat placement (TEHatRack.cs:165-201)
// ---------------------------------------------------------------------------

/// One hat's draw placement from `TEHatRack.Draw` (`TEHatRack.cs:165-201`).
/// The rack draws its two hats as two invisible-body doll players, each
/// carrying one head-slot item; the renderer draws only the head layer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct HatRackDraw {
    /// The first hat's draw direction (`:185`'s `direction *= -1`): the
    /// negated frame direction. The second hat draws at `-direction`
    /// (`:198` flips it back).
    pub direction: i32,
    /// The shared anchor `vector = (tileLeftX+1.5, tileTopY+4)*16` (`:184`).
    pub anchor: [f32; 2],
    /// First hat position (`:186`):
    /// `vector + (-width/2, -height-6) + (direction*14, -2)`.
    pub hat0_pos: [f32; 2],
    /// Second hat position (`:199`):
    /// `vector + (-width/2, -height-6) + (direction*12, 16)` — the
    /// direction is negated AGAIN between the two draws (`:198`).
    pub hat1_pos: [f32; 2],
}

/// `TEHatRack.Draw`'s placement half (`TEHatRack.cs:165-201`). `frame_x`
/// is the anchor tile's `frameX`; `width`/`height` the doll player hitbox.
/// The direction flips twice (`:185` then `:198`), so the second hat's
/// offset uses the ORIGINAL frame direction while the first uses its
/// negation. Both hats draw with `invis = true` (`:180,192`) — only the
/// head armor layer shows.
pub(crate) fn hat_rack_draw(
    frame_x: i16,
    width: f32,
    height: f32,
    tile_left_x: i32,
    tile_top_y: i32,
) -> HatRackDraw {
    // :168-173 — direction -1 default, +1 when frameX % 216 == 54.
    let mut direction = -1i32;
    if frame_x as i32 % 216 == 54 {
        direction = 1;
    }
    // :184 — the shared anchor.
    let anchor = [
        (tile_left_x as f32 + 1.5) * 16.0,
        (tile_top_y + 4) as f32 * 16.0,
    ];
    let base = [-width / 2.0, -height - 6.0];
    // :185 — direction *= -1 before the first hat.
    let d0 = -direction;
    let hat0_pos = [
        anchor[0] + base[0] + d0 as f32 * 14.0,
        anchor[1] + base[1] - 2.0,
    ];
    // :198 — direction *= -1 again (back to the frame direction).
    let d1 = -d0;
    let hat1_pos = [
        anchor[0] + base[0] + d1 as f32 * 12.0,
        anchor[1] + base[1] + 16.0,
    ];
    HatRackDraw { direction: d0, anchor, hat0_pos, hat1_pos }
}

// ---------------------------------------------------------------------------
// The held-projectile preview (TEDisplayDoll.cs:458-490 → Projectile.AI_DisplayDoll)
// ---------------------------------------------------------------------------

/// `ProjectileID.Sets.DisplayDollDrawUseStyle` (`ID/ProjectileID.cs:188`):
/// every listed projectile previews under useStyle 5 (the gun aim ladder)
/// regardless of its item's own useStyle; the default is -1.
const DISPLAY_DOLL_DRAW_USE_STYLE: [u16; 67] = [
    301, 933, 1100, 6, 19, 33, 52, 106, 113, 182, 272, 320, 333, 383, 866, 867, 902, 1000,
    1052, 491, 273, 481, 636, 3, 21, 48, 54, 69, 70, 621, 93, 166, 304, 318, 330, 370, 371,
    497, 510, 520, 599, 861, 868, 869, 936, 996, 1085, 507, 598, 28, 29, 37, 470, 516, 519,
    637, 773, 903, 904, 905, 906, 910, 911, 1077, 1086, 1087,
];

/// The set's value for a projectile kind (`SetFactory.CreateIntSet(-1, …)`):
/// 5 for the listed kinds, -1 otherwise.
pub(crate) fn display_doll_draw_use_style(kind: u16) -> i32 {
    if DISPLAY_DOLL_DRAW_USE_STYLE.contains(&kind) { 5 } else { -1 }
}

/// `ItemID.Sets.IsAPhaseblade` (`ID/ProjectileID.cs:192`): the aiStyle-3
/// arm refuses to preview these (`Projectile.cs:22730-22734`).
const IS_A_PHASEBLADE: [u16; 19] = [
    1059, 1060, 1061, 1062, 1063, 1064, 1065, 1066, 1067, 1068, 1069, 1070, 1071, 1072,
    1075, 1076, 1101, 1102, 1074,
];

/// The doll player's geometry the preview reads (`Player.MountedCenter`
/// `Player.cs:3337-3341`, `HandPosition` `:4189-4216`, `GetArmPosition`
/// `:50595-50615`). The wiring step derives these from the live doll
/// `Player` after `PlayerFrame`; the pure machine takes them as inputs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DollProjEnv {
    /// `dollPlayer.direction` (the resolved [`DollDraw::direction`]).
    pub direction: i32,
    /// `doll.MountedCenter` — `(position.X + width/2, position.Y +
    /// BaseHeight/2 + HeightOffsetHitboxCenter)`.
    pub mounted_center: [f32; 2],
    /// `doll.Center` — `(position.X + width/2, position.Y + height/2)`.
    pub center: [f32; 2],
    /// `doll.Bottom` — `(position.X + width/2, position.Y + height)`.
    pub bottom: [f32; 2],
    /// `doll.GetArmPosition()` (`:50595`) — the aiStyle-165 flail anchor.
    pub arm_position: [f32; 2],
    /// `doll.bodyFrame.Y / 56` — the body row the on-hand table indexes.
    pub body_frame_row: u32,
    /// `doll.gravDir` (1 stock; the doll never flips gravity).
    pub grav_dir: f32,
    /// `doll.miscCounter` (`Player.UpdateMiscCounter`, `:29295-29301`).
    pub misc_counter: u32,
    /// `doll.mount.PlayerOffset` (`Mount.cs:407-420`) — the per-frame
    /// player Y offset; 0 when unmounted.
    pub mount_player_offset: i32,
    /// `doll.sitting.isSitting` — the `RotatedRelativePoint` sitting term.
    pub sitting: bool,
    /// `GetSittingOffsetInfo`'s `seatAdjustment` (`PlayerSittingHelper.cs:
    /// 23-24`): `-4 + (int)offsetForSeat.Y`; 0 when not seated.
    pub seat_y_offset: i32,
}

/// The previewed projectile's `SetDefaults` slice the machine reads:
/// `width`/`height` (the box), `aiStyle`, `scale`, `alpha`, `drawLayer`.
/// The wiring step resolves it from `proj_data(kind)` plus the per-type
/// overrides the generated table omits (the module's `doll_proj_defaults`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DollProjDefaults {
    pub w: f32,
    pub h: f32,
    pub ai_style: u8,
    pub scale: f32,
    pub alpha: i32,
    pub draw_layer: i32,
}

/// The `SetDefaults` rows for the kinds `AI_DisplayDoll` previews that the
/// generated `proj_data` table does not carry (or carries with a different
/// box). Every row is the C# `SetDefaults` arm verbatim (`Projectile.cs`,
/// the cited line). `None` = fall through to `proj_data`.
pub(crate) fn doll_proj_defaults(kind: u16) -> Option<DollProjDefaults> {
    let (w, h, ai, scale, alpha, layer) = match kind {
        // :864-872 — the Enchanted Boomerang 6.
        6 => (22.0, 22.0, 3, 1.0, 0, 0),
        // :1008-1016 — the Flamarang 19.
        19 => (22.0, 22.0, 3, 1.0, 0, 0),
        // :1152-1160 — the Thorn Chakram 33, scale 0.9.
        33 => (38.0, 38.0, 3, 0.9, 0, 0),
        // :1345-1353 — the Wooden Boomerang 52.
        52 => (22.0, 22.0, 3, 1.0, 0, 0),
        // :1765-1773 — the Shuriken 93… no, the Poisoned Knife 93 is
        // :1765; the box 12x12, aiStyle 2.
        93 => (12.0, 12.0, 2, 1.0, 0, 0),
        // :2012-2020 — the Ice Boomerang 113.
        113 => (22.0, 22.0, 3, 1.0, 0, 0),
        // :2686-2694 — the Light Disc 182.
        182 => (30.0, 30.0, 3, 1.0, 0, 0),
        // :3578-3586 — the Paladin's Hammer 272, scale 0.9, alpha 255.
        272 => (32.0, 32.0, 3, 0.9, 255, 0),
        // :3916-3924 — the Death Sickle 301.
        301 => (38.0, 38.0, 3, 1.0, 0, 0),
        // :3952-3960 — the Fruitcake Chakram 304, alpha 255.
        304 => (30.0, 30.0, 2, 1.0, 255, 0),
        // :4112-4120 — the Bloody Machete 318.
        318 => (12.0, 14.0, 2, 1.0, 0, 0),
        // :4130-4138 — the Combat Wrench 320.
        320 => (34.0, 34.0, 3, 1.0, 0, 0),
        // :4226-4234 — the Bone Glove 330… the box 22x22, aiStyle 2.
        330 => (22.0, 22.0, 2, 1.0, 0, 0),
        // :4258-4266 — the Bananarang 333, scale 0.9.
        333 => (38.0, 38.0, 3, 0.9, 0, 0),
        // :4536-4543 — the Vampire Knives 370.
        370 => (14.0, 14.0, 2, 1.0, 0, 0),
        // :4544-4552 — the Scourge of the Corruptor 371.
        371 => (14.0, 14.0, 2, 1.0, 0, 0),
        // :5091-5099 — the Hornet minion 439 (drawLayer 7).
        439 => (22.0, 22.0, 75, 1.0, 0, 7),
        // :5162-5170 — the Laser Drill 445 (drawLayer 7).
        445 => (10.0, 10.0, 75, 1.0, 0, 7),
        // :5404-5412 — the Flairon 491.
        491 => (26.0, 26.0, 9, 1.0, 0, 0),
        // :5561-5569 — the Shadowflame Knife 481, alpha 255.
        481 => (22.0, 22.0, 13, 1.0, 255, 0),
        // :5705-5713 — the Flying Knife 497.
        497 => (30.0, 30.0, 2, 1.0, 0, 0),
        // :5770-5778 — the Javelin 507.
        507 => (16.0, 16.0, 1, 1.0, 0, 0),
        // :5796-5804 — the Bone Javelin 510, scale 1.1.
        510 => (18.0, 18.0, 2, 1.1, 0, 0),
        // :6094-6102 — the Magic Dagger 535 (drawLayer 8).
        535 => (18.0, 18.0, 100, 1.0, 0, 8),
        // :6437-6445 — the Magnet Sphere 595 (drawLayer 7).
        595 => (68.0, 64.0, 75, 1.0, 0, 7),
        // :6481-6489 — the Snowball 599… the box 22x22, aiStyle 2,
        // drawLayer 8 (the :6481 arm).
        599 => (22.0, 22.0, 2, 1.0, 0, 8),
        // :6620-6628 — the Pirate minion 615 (drawLayer 7).
        615 => (22.0, 22.0, 75, 1.0, 0, 7),
        // :7366-7374 — the Meowmere 697, scale 1.35, alpha 255, layer 7.
        697 => (16.0, 16.0, 140, 1.35, 255, 7),
        // :7402-7410 — the Star Wrath 699, alpha 255, layer 7.
        699 => (24.0, 24.0, 142, 1.0, 255, 7),
        // :7572-7580 — the Spider minion 714 (drawLayer 7).
        714 => (22.0, 22.0, 75, 1.0, 0, 7),
        // :7736-7744 — the Rainbow Crystal 735 (drawLayer 7).
        735 => (68.0, 64.0, 75, 1.0, 0, 7),
        // :8312-8320 — the Phaseblade 866.
        866 => (30.0, 30.0, 3, 1.0, 0, 0),
        // :8325-8333 — the Phasesaber 867.
        867 => (22.0, 22.0, 3, 1.0, 0, 0),
        // :8599-8607 — the Code 2 902.
        902 => (20.0, 20.0, 3, 1.0, 0, 0),
        // :8781-8789 — the Terrarian 927 (drawLayer 7).
        927 => (40.0, 40.0, 75, 1.0, 0, 7),
        // :8896-8904 — the Amarok 936, alpha 255.
        936 => (14.0, 14.0, 2, 1.0, 255, 0),
        // :9458-9466 — the Cascade 996.
        996 => (18.0, 18.0, 2, 1.0, 0, 0),
        // :10125-10133 — the Kraken 1085.
        1085 => (18.0, 18.0, 2, 1.0, 0, 0),
        _ => return None,
    };
    Some(DollProjDefaults { w, h, ai_style: ai, scale, alpha, draw_layer: layer })
}

/// The resolved preview draw: the projectile's centre, rotation, scale,
/// alpha and sprite direction, plus the `drawLayer` the renderer routes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DollProjDraw {
    pub center: [f32; 2],
    pub rotation: f32,
    pub scale: f32,
    pub alpha: i32,
    pub sprite_direction: i32,
    pub draw_layer: i32,
    /// `Projectile.frame` — the misc-counter frame the 75/161/100 arms pin.
    pub frame: i32,
}

/// `Utils.RotatedBy` about the origin (`Utils.cs:1675-1684`).
fn rotated(v: [f32; 2], radians: f32) -> [f32; 2] {
    let (c, s) = (radians.cos(), radians.sin());
    [v[0] * c - v[1] * s, v[0] * s + v[1] * c]
}

/// `Vector2.SafeNormalize(Vector2.Zero)` — the zero vector stays zero.
fn safe_normalize(v: [f32; 2]) -> [f32; 2] {
    let len = (v[0] * v[0] + v[1] * v[1]).sqrt();
    if len == 0.0 { [0.0, 0.0] } else { [v[0] / len, v[1] / len] }
}

fn add(a: [f32; 2], b: [f32; 2]) -> [f32; 2] { [a[0] + b[0], a[1] + b[1]] }
fn sub(a: [f32; 2], b: [f32; 2]) -> [f32; 2] { [a[0] - b[0], a[1] - b[1]] }
fn mul(v: [f32; 2], s: f32) -> [f32; 2] { [v[0] * s, v[1] * s] }

/// `Player.CalculateFrameFromMiscCounter` (`Player.cs:4253-4274`): the
/// 300-tick misc-counter cycle remapped onto `[minFrame, maxFrame]`.
pub(crate) fn calculate_frame_from_misc_counter(
    misc_counter: u32,
    framerate: i32,
    min_frame: i32,
    max_frame: i32,
) -> i32 {
    let num = 300.0f32;
    let mut num2 = 0i32;
    let mut num3 = 0i32;
    while (num3 as f32) < num {
        num2 += 1;
        num3 += framerate * (max_frame + min_frame);
        if (num3 as f32) >= num {
            num2 += 1;
            break;
        }
    }
    if num2 <= 0 {
        num2 = 1;
    }
    let num4 = 1.0 / num2 as f32;
    let t = (misc_counter as f32 / num) % num4;
    // Utils.Remap(t, 0, num4, minFrame, maxFrame) — the clamped lerp.
    let lerp = get_lerp_value(0.0, num4, t);
    (min_frame as f32 + (max_frame - min_frame) as f32 * lerp) as i32
}

/// `Main.OffsetsPlayerOnhand` (`Main.cs:66615-66637`): the 20-row on-hand
/// offset table the `HandPosition`/`GetArmPosition`/aiStyle-100 arms read.
const OFFSETS_PLAYER_ONHAND: [[f32; 2]; 20] = [
    [6.0, 19.0], [5.0, 10.0], [12.0, 10.0], [13.0, 17.0], [12.0, 19.0],
    [5.0, 10.0], [7.0, 17.0], [6.0, 16.0], [6.0, 16.0], [6.0, 16.0],
    [6.0, 17.0], [7.0, 17.0], [7.0, 17.0], [7.0, 17.0], [8.0, 17.0],
    [9.0, 16.0], [9.0, 12.0], [8.0, 17.0], [7.0, 17.0], [7.0, 17.0],
];

/// `Player.HandPosition` (`Player.cs:4189-4216`) for the doll: the on-hand
/// offset for the body row, mirrored for direction, centred on
/// `MountedCenter` and rotated about the mount pivot
/// (`RotatedRelativePoint`, `:4225-4251`). `fullRotation` is 0 for the doll
/// (it never spins) and `gfxOffY` 0, so the rotation is the identity; the
/// sitting offset rides when the pose is Sitting. The
/// `ApplyItemPositionOffsetFromMount` half (`:4213`) is a no-op for the
/// doll's stock mounts (no item-position delegate).
pub(crate) fn doll_hand_position(env: &DollProjEnv) -> [f32; 2] {
    let row = (env.body_frame_row as usize).min(19);
    let mut vector = mul(OFFSETS_PLAYER_ONHAND[row], 2.0);
    if env.direction != 1 {
        vector[0] = 40.0 - vector[0];
    }
    if env.grav_dir != 1.0 {
        vector[1] = 56.0 - vector[1];
    }
    // :4210 — subtract (bodyFrame.Width - width, bodyFrame.Height - 42)/2.
    vector = sub(vector, [(40.0 - 20.0) / 2.0, (56.0 - 42.0) / 2.0]);
    // :4211-4212 — -new Vector2(20,42)/2 + vector, on MountedCenter.
    let pos = add(env.mounted_center, add([-10.0, -21.0], vector));
    rotated_relative_point(env, pos)
}

/// `Player.RotatedRelativePoint` (`Player.cs:4225-4251`) with the doll's
/// `fullRotation = 0`, `gfxOffY = 0`, `sleeping = false`: the mount pivot
/// is `Bottom + (0, -(PlayerOffset/2 + 4))`, the point rotates about it by
/// `fullRotation` (0 → identity), and the sitting offset is added when
/// seated. The doll's `sittingIndex` is 0 (single-player), so the sitting
/// `posOffset` is `(0, 0)` and `seatAdjustment` is `-4 + (int)offsetForSeat.Y`.
fn rotated_relative_point(env: &DollProjEnv, pos: [f32; 2]) -> [f32; 2] {
    let num2 = env.mount_player_offset / 2 + 4;
    let pivot = add(env.bottom, [0.0, -num2 as f32]);
    // fullRotation 0 → the rotated term is the point itself; the pivot
    // offset (0,-num2) + RotatedBy((0,num2), 0) = (0,0).
    let mut out = add(pivot, sub(pos, pivot));
    if env.sitting {
        // sittingIndex 0 → posOffset (0,0); seatAdjustment -4 + seat drop.
        out = add(out, [0.0, env.seat_y_offset as f32]);
    }
    out
}

/// `Projectile.AI_DisplayDoll_ItemDisplay` (`Projectile.cs:22545-22572`):
/// the shared "hold the weapon out along the aim" placement. Returns the
/// projectile centre + rotation; the caller applies the per-type nudges.
/// `weapon_shoot_speed`/`weapon_scale` are `item.shootSpeed`/`item.scale`.
fn item_display(
    env: &DollProjEnv,
    pose: &UsePose,
    weapon_shoot_speed: f32,
    weapon_scale: f32,
    rotation_addon: f32,
) -> ([f32; 2], f32, [f32; 2]) {
    // :22548-22559 — the aim vector, rotated then mirrored for direction -1.
    let mut spin = mul([1.0, 0.0], weapon_shoot_speed * weapon_scale);
    let aim = pose.item_aim_radians.unwrap_or(0.0);
    spin = rotated(spin, aim);
    if env.direction == -1 {
        spin[0] *= -1.0;
    }
    // :22560-22565 — position += spin, then centre on the hand
    // (`doll.HandPosition`, always Some in the doll's stock path). The
    // position += spin is dropped: the HandPosition overwrite replaces it.
    let center = doll_hand_position(env);
    // :22566-22570 — rotation = velocity.ToRotation() + addon*direction,
    // plus PI/2 when facing left.
    let mut rotation = spin[1].atan2(spin[0]) + rotation_addon * env.direction as f32;
    if env.direction == -1 {
        rotation += std::f32::consts::PI / 2.0;
    }
    (center, rotation, spin)
}

/// `Projectile.AI_DisplayDoll` (`Projectile.cs:22580-23197`) — the doll's
/// held-projectile preview. Returns `None` when `botherDrawing` is false
/// (the pose is not a Use pose, the aiStyle has no preview arm, or the
/// per-type gate refuses). The caller draws the returned sprite only when
/// `drawLayer <= 3` (`TEDisplayDoll.cs:485-488`); the 7/8 layers ride the
/// player-renderer override and are returned for the wiring step.
///
/// Deferrals (ANALYSIS "The TE furniture draw"): the deep per-type centre
/// nudges of the aiStyle-2/3 boomerang families and the aiStyle-75/161
/// minion frame machines are ported for the kinds the doll actually
/// previews; the `ownedProjectileCounts[902]` write (`:22799-22802`) is a
/// sim-side effect the pure draw omits.
#[allow(clippy::too_many_arguments)]
pub(crate) fn ai_display_doll(
    kind: u16,
    defaults: &DollProjDefaults,
    env: &DollProjEnv,
    pose: &UsePose,
    weapon_shoot_speed: f32,
    weapon_scale: f32,
) -> Option<DollProjDraw> {
    let pi = std::f32::consts::PI;
    // :22585-22588 — only the Use poses (4-8) preview.
    let is_use = matches!(
        pose.pose,
        DollPose::Use1 | DollPose::Use2 | DollPose::Use3 | DollPose::Use4 | DollPose::Use5
    );
    let mut bother = true;
    let mut center = env.mounted_center;
    let mut rotation = 0.0f32;
    let mut scale = defaults.scale;
    let mut alpha = defaults.alpha;
    let mut sprite_dir = env.direction;
    let mut draw_layer = defaults.draw_layer;
    let mut frame = 0i32;
    let mut velocity = [0.0f32, 0.0];

    match defaults.ai_style {
        // :22595-22610 — aiStyle 1: only the Javelin 507 previews.
        1 => {
            if kind != 507 {
                return None;
            }
            draw_layer = 7;
            let (c, r, v) = item_display(env, pose, weapon_shoot_speed, weapon_scale, 0.0);
            center = add(c, mul(safe_normalize(v), 12.0));
            center = add(center, [-2.0, -2.0]);
            rotation = r;
            velocity = v;
            alpha = 0;
        }
        // :22611-22723 — aiStyle 2: the thrown/boomerang family.
        2 => {
            draw_layer = 7;
            let (c, r, v) = item_display(env, pose, weapon_shoot_speed, weapon_scale, 0.0);
            center = c;
            rotation = r;
            velocity = v;
            if env.direction == 1 {
                rotation += pi / 2.0;
            }
            match kind {
                497 => center = add(center, [8.0, -4.0]),
                69 | 70 | 621 => {
                    center = add(center, mul([rotation.cos(), rotation.sin()], 6.0));
                    rotation += pi;
                    match pose.pose {
                        DollPose::Use1 | DollPose::Use2 => {
                            center = add(center, [6.0 * env.direction as f32, -6.0]);
                        }
                        DollPose::Use3 => center = add(center, [6.0 * env.direction as f32, 0.0]),
                        _ => {}
                    }
                }
                1085 => rotation -= pi / 4.0 * env.direction as f32,
                3 | 166 | 318 | 330 | 510 | 868 | 869 | 996 => {
                    if env.direction == 1 {
                        center = sub(center, mul([rotation.cos(), rotation.sin()], 6.0));
                    } else {
                        center = add(center, mul([rotation.cos(), rotation.sin()], 6.0));
                    }
                    match pose.pose {
                        DollPose::Use1 => center = add(center, [6.0 * env.direction as f32, -6.0]),
                        DollPose::Use2 => {
                            if kind != 318 {
                                center = add(center, [6.0 * env.direction as f32, -6.0]);
                            }
                        }
                        DollPose::Use3 => {}
                        _ => {}
                    }
                }
                48 | 54 | 93 | 304 => {
                    center = add(center, [0.0, -2.0]);
                    center = add(center, mul(safe_normalize(velocity), 11.0));
                }
                599 => {
                    center = add(center, [4.0, -2.0]);
                    center = add(center, mul(safe_normalize(velocity), 4.0));
                }
                370 | 371 | 936 => {
                    if env.direction == 1 {
                        center = add(center, [-6.0, 0.0]);
                    }
                    center = add(center, mul(safe_normalize(velocity), 12.0));
                    rotation += pi;
                    let nudge = match pose.pose {
                        DollPose::Use1 | DollPose::Use2 => [2.0 * env.direction as f32, -2.0],
                        DollPose::Use3 => [4.0 * env.direction as f32, -4.0],
                        DollPose::Use4 => [8.0 * env.direction as f32, -6.0],
                        DollPose::Use5 => [8.0 * env.direction as f32, -2.0],
                        _ => [0.0, 0.0],
                    };
                    center = add(center, nudge);
                    if kind == 936 {
                        center = sub(center, mul(safe_normalize(velocity), 8.0));
                    }
                }
                _ => {}
            }
            alpha = 0;
        }
        // :22724-22804 — aiStyle 3: the disc/phaseblade family.
        3 => {
            if kind == 866 || IS_A_PHASEBLADE.contains(&kind) {
                return None;
            }
            let rotation_addon = if kind == 182 { -pi / 2.0 } else { 0.0 };
            draw_layer = 7;
            let (c, r, _v) = item_display(env, pose, weapon_shoot_speed, weapon_scale, rotation_addon);
            center = c;
            rotation = r;
            match kind {
                182 => {
                    if env.direction == 1 {
                        rotation += pi / 2.0;
                        center = add(center, [-4.0, 0.0]);
                    }
                    let dx = match pose.pose {
                        DollPose::Use1 | DollPose::Use3 | DollPose::Use4 => 4.0,
                        DollPose::Use2 | DollPose::Use5 => 2.0,
                        _ => 0.0,
                    };
                    center = add(center, [dx * env.direction as f32, 0.0]);
                }
                6 | 19 | 52 | 113 | 272 | 867 => {
                    if env.direction == 1 {
                        rotation += pi / 2.0;
                    }
                }
                33 | 333 => {
                    if env.direction == 1 {
                        center = add(center, [-2.0, 0.0]);
                    }
                }
                320 => {
                    if env.direction == 1 {
                        rotation += pi / 2.0;
                    }
                    rotation -= pi / 2.0 * env.direction as f32;
                }
                _ => {}
            }
        }
        // :22805-22818 — aiStyle 9: only the Flairon 491.
        9 => {
            if kind != 491 {
                return None;
            }
            draw_layer = 7;
            let (c, r, _v) = item_display(env, pose, weapon_shoot_speed, weapon_scale, 0.0);
            center = c;
            rotation = r + pi / 4.0;
        }
        // :22819-22845 — aiStyle 13: the spear-thrust knives.
        13 => {
            sprite_dir = env.direction;
            let (c, r, v) = item_display(env, pose, weapon_shoot_speed, weapon_scale, 0.0);
            center = c;
            rotation = r;
            draw_layer = 7;
            if kind == 481 {
                center = add(center, mul(safe_normalize(v), 56.0));
                match pose.pose {
                    DollPose::Use1 => center = add(center, [0.0, 2.0]),
                    DollPose::Use5 => center = add(center, [0.0, -8.0]),
                    _ => {}
                }
            } else {
                center = add(center, mul(safe_normalize(v), 50.0));
            }
            alpha = 0;
        }
        // :22846-22853 — aiStyle 15: the held-up item at the doll's side.
        // The C# writes base.position = (Center.X + 9*dir, Bottom.Y - 12);
        // the returned centre adds half the box.
        15 => {
            sprite_dir = env.direction;
            center = [
                env.center[0] + 9.0 * env.direction as f32 + defaults.w / 2.0,
                env.bottom[1] - 12.0 + defaults.h / 2.0,
            ];
        }
        // :22854-22877 — aiStyle 16: the grenade/launcher family.
        16 => {
            if (134..=144).contains(&kind)
                || (338..=341).contains(&kind)
                || (803..=810).contains(&kind)
                || (784..=801).contains(&kind)
            {
                return None;
            }
            sprite_dir = env.direction;
            draw_layer = 7;
            let (c, r, v) = item_display(env, pose, weapon_shoot_speed, weapon_scale, -pi / 2.0);
            center = c;
            rotation = r;
            if env.direction == 1 {
                rotation += pi / 2.0;
            }
            center = add(center, [0.0, -2.0]);
            center = add(center, mul(safe_normalize(v), 4.0));
            if kind == 517 {
                center = add(center, mul(safe_normalize(v), 2.0));
            }
        }
        // :22878-22924 — aiStyle 19: the spears.
        19 => {
            sprite_dir = -env.direction;
            let mut unit = [1.0f32, 0.0];
            let aim = pose.item_aim_radians.unwrap_or(0.0);
            unit = rotated(unit, aim);
            if env.direction == -1 {
                unit[0] *= -1.0;
            }
            velocity = unit;
            let mut item_animation = 25i32;
            match kind {
                367 | 368 => item_animation = 20,
                // :22901-22911 — the 130/877-879 `base.position +=` nudges
                // are WIPED by the offset helper's `base.Center = center`
                // write (`:46087`) before the final scalar nudge; only the
                // itemAnimation/alpha writes survive.
                130 => item_animation = 50,
                877 | 878 | 879 => {
                    item_animation = 50;
                    alpha = 0;
                }
                _ => {}
            }
            // :22913 — the helper re-centres on
            // RotatedRelativePoint(MountedCenter) (`:46085-46087`), then
            // the scalar offset rides (`:46183`), itemAnimationMax 100.
            let mut position = rotated_relative_point(env, env.mounted_center);
            let off = spear_offset(kind, item_animation, 100);
            position = add(position, mul(unit, off));
            center = position;
            rotation = unit[1].atan2(unit[0]) + pi / 2.0 + pi / 4.0;
            if sprite_dir == -1 {
                rotation -= pi / 2.0;
            }
        }
        // :22925-22947 — aiStyle 20: the drill. The C# writes
        // base.position += spin over the :22577 reset position
        // (MountedCenter - halfbox), so the centre keeps the halfbox.
        20 => {
            sprite_dir = env.direction;
            let mut spin = mul([1.0, 0.0], weapon_shoot_speed * defaults.scale);
            let aim = pose.item_aim_radians.unwrap_or(0.0);
            spin = rotated(spin, aim);
            if env.direction == -1 {
                spin[0] *= -1.0;
            }
            velocity = spin;
            center = add(env.mounted_center, spin);
            rotation = spin[1].atan2(spin[0]) + pi / 2.0;
        }
        // :22948-22967 — aiStyle 26: only the Zenith components 1094/1113.
        26 => {
            if kind != 1094 && kind != 1113 {
                return None;
            }
            alpha = 0;
            draw_layer = 7;
            let (_c, r, v) = item_display(env, pose, weapon_shoot_speed, weapon_scale, -pi / 2.0);
            rotation = r;
            center = sub(env.center, mul(safe_normalize(v), 12.0));
            if env.direction == 1 {
                rotation += pi / 2.0;
            }
        }
        // :22968-23053 — aiStyle 75/161: the minion/summon previews. The
        // C# writes base.position += spin over the :22577 reset position
        // (MountedCenter - halfbox); the Centre nudges below add to it.
        75 | 161 => {
            sprite_dir = env.direction;
            let mut spin = mul([1.0, 0.0], weapon_shoot_speed * defaults.scale);
            let aim = pose.item_aim_radians.unwrap_or(0.0);
            spin = rotated(spin, aim);
            if env.direction == -1 {
                spin[0] *= -1.0;
            }
            velocity = spin;
            let halfbox = [defaults.w / 2.0, defaults.h / 2.0];
            let mut position = add(sub(env.mounted_center, halfbox), spin);
            // :22990-23002 — the misc-counter frame machines. The maxFrame
            // is Main.projFrames[type]; the port's table omits the 439/615/
            // 714 rows, so the C# values (Main.cs:8535/:8496/:8503) ride.
            match kind {
                439 => frame = calculate_frame_from_misc_counter(env.misc_counter, 4, 0, 6),
                633 => {
                    frame = calculate_frame_from_misc_counter(env.misc_counter, 4, 0, proj_frames(kind) as i32);
                }
                615 => frame = calculate_frame_from_misc_counter(env.misc_counter, 3, 0, 7),
                714 => frame = calculate_frame_from_misc_counter(env.misc_counter, 5, 0, 7),
                _ => {}
            }
            let mut num4 = pi / 2.0;
            if defaults.ai_style == 161 {
                position = add(position, mul(spin, 1.5));
            }
            match kind {
                595 | 615 | 630 | 705 | 714 | 735 => {
                    num4 = 0.0;
                    if sprite_dir == -1 {
                        num4 = pi;
                    }
                }
                927 => num4 = 0.0,
                938..=945 | 1127 => position = add(position, mul(spin, 10.0)),
                802 => position = add(position, mul(spin, 8.0)),
                842 => position = add(position, mul(spin, 4.0)),
                _ => {}
            }
            rotation = spin[1].atan2(spin[0]) + num4;
            center = add(position, halfbox);
            if kind == 595 || kind == 735 {
                let (c, r, _v) = item_display(env, pose, weapon_shoot_speed, weapon_scale, -pi / 2.0);
                center = c;
                rotation = r;
            }
        }
        // :23054-23061 — aiStyle 99: the held item at the doll's side
        // (base.position write → centre + half the box).
        99 => {
            sprite_dir = env.direction;
            center = [
                env.center[0] + 9.0 * env.direction as f32 + defaults.w / 2.0,
                env.bottom[1] - 8.0 + defaults.h / 2.0,
            ];
        }
        // :23062-23089 — aiStyle 100: the magic dagger on-hand. The C#
        // sets base.Center directly (the floored RotatedRelativePoint).
        100 => {
            let zero = [env.direction as f32 * 6.0, env.grav_dir * -14.0];
            rotation = if env.grav_dir == 1.0 { 0.0 } else { pi };
            sprite_dir = env.direction;
            // :23069-23078 — the on-hand offset, mirrored for direction.
            let row = (env.body_frame_row as usize).min(19);
            let mut vector = mul(OFFSETS_PLAYER_ONHAND[row], 2.0);
            if env.direction != 1 {
                vector[0] = 40.0 - vector[0];
            }
            if env.grav_dir != 1.0 {
                vector[1] = 56.0 - vector[1];
            }
            vector = sub(vector, [(40.0 - 20.0) / 2.0, (56.0 - 42.0) / 2.0]);
            // :23079 — RotatedRelativePoint(MountedCenter - (20,42)/2 +
            // vector + zero) - velocity, floored. RotatedRelativePoint
            // rotates by fullRotation (`:4227`), which is 0 for the doll
            // (it never spins) — NOT this arm's `rotation` field — so the
            // rotation is the identity and the pivot terms cancel; the
            // gfxOffY/sitting terms are 0 for the stock (non-seated) doll.
            let rel = add(add(sub(env.mounted_center, [10.0, 21.0]), vector), zero);
            center = [
                (rel[0] - velocity[0]).floor(),
                (rel[1] - velocity[1]).floor(),
            ];
            if kind == 535 {
                frame = calculate_frame_from_misc_counter(env.misc_counter, 6, 0, 8);
            }
        }
        // :23090-23098 — aiStyle 113: the ice boomerang's held preview.
        113 => {
            draw_layer = 7;
            let (c, r, _v) = item_display(env, pose, weapon_shoot_speed, weapon_scale, 0.0);
            center = c;
            rotation = r;
            alpha = 0;
        }
        // :23099-23162 — aiStyle 140/142: the Meowmere/Star Wrath cats.
        // The C# writes base.position (the :22577 reset left it at
        // MountedCenter - halfbox), so every position write carries the
        // halfbox offset into the returned centre.
        140 | 142 => {
            if kind == 697 {
                scale = 1.0;
            }
            sprite_dir = env.direction;
            let base_len = if defaults.ai_style == 140 { 12.0 } else { 4.0 };
            let mut spin = mul([1.0, 0.0], base_len);
            let aim = pose.item_aim_radians.unwrap_or(0.0);
            spin = rotated(spin, aim);
            if env.direction == -1 {
                spin[0] *= -1.0;
            }
            velocity = spin;
            let halfbox = [defaults.w / 2.0, defaults.h / 2.0];
            let mut position = add(sub(env.mounted_center, halfbox), spin);
            let num8;
            if kind != 699 {
                num8 = if sprite_dir == -1 { pi } else { 0.0 };
            } else {
                num8 = pi / 4.0 * env.direction as f32
                    + if sprite_dir == -1 { pi } else { 0.0 };
                let off = rotated([1.0, -1.0], aim);
                position = add(position, [
                    off[0] * 75.0 * env.direction as f32,
                    off[1] * 75.0,
                ]);
                if pose.pose == DollPose::Use2 {
                    position[0] -= env.direction as f32 * 4.0;
                }
                if pose.pose == DollPose::Use5 {
                    position[1] -= 8.0;
                }
            }
            alpha = 0;
            rotation = spin[1].atan2(spin[0]) + num8;
            if kind == 699 {
                position = add(position, rotated([110.0 * env.direction as f32, -6.0], rotation));
                position = add(position, mul(safe_normalize(spin), -40.0));
            }
            center = add(position, halfbox);
        }
        // :23163-23186 — aiStyle 165: the flail.
        165 => {
            let mut spin = [1.0f32, 0.0];
            let aim = pose.item_aim_radians.unwrap_or(0.0);
            spin = rotated(spin, aim);
            if env.direction == -1 {
                spin[0] *= -1.0;
            }
            velocity = spin;
            center = add(env.arm_position, mul(spin, 12.0 - 1.0));
            sprite_dir = if spin[0] < 0.0 { -1 } else { 1 };
        }
        // :23187-23196 — aiStyle 182: the yoyo's held preview.
        182 => {
            draw_layer = 7;
            let (c, r, _v) = item_display(env, pose, weapon_shoot_speed, weapon_scale, pi / 4.0);
            center = c;
            rotation = r;
            alpha = 0;
        }
        // :22592-22594 — no preview arm.
        _ => return None,
    }

    if !is_use {
        bother = false;
    }
    if !bother {
        return None;
    }
    Some(DollProjDraw { center, rotation, scale, alpha, sprite_direction: sprite_dir, draw_layer, frame })
}

// ---------------------------------------------------------------------------
// Tests-as-code (written, NOT run — code-only mode)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Part1 anchors -------------------------------------------------------

    #[test]
    fn item_frame_anchor_steps_off_the_36_grid() {
        // TileDrawing.cs:4156-4164: the bottom-right cell (18, 18) steps
        // left AND up; the top-left stays.
        assert_eq!(te_anchor(395, 10, 20, 0, 0), Some((TeKind::ItemFrame, 10, 20)));
        assert_eq!(te_anchor(395, 11, 20, 18, 0), Some((TeKind::ItemFrame, 10, 20)));
        assert_eq!(te_anchor(395, 10, 21, 0, 18), Some((TeKind::ItemFrame, 10, 20)));
        assert_eq!(te_anchor(395, 11, 21, 18, 18), Some((TeKind::ItemFrame, 10, 20)));
        // Style columns: frameX 36 is still % 36 == 0.
        assert_eq!(te_anchor(395, 12, 20, 36, 0), Some((TeKind::ItemFrame, 12, 20)));
    }

    #[test]
    fn platter_anchor_is_the_cell_itself() {
        // TileDrawing.cs:4196: no frame step at all.
        assert_eq!(te_anchor(520, 7, 9, 18, 0), Some((TeKind::FoodPlatter, 7, 9)));
    }

    #[test]
    fn rack_anchor_uses_the_54_grid() {
        // TileDrawing.cs:4209-4210: frameX % 54 / 18, frameY % 54 / 18.
        assert_eq!(te_anchor(471, 12, 22, 36, 36), Some((TeKind::WeaponsRack, 10, 20)));
        assert_eq!(te_anchor(471, 10, 20, 0, 0), Some((TeKind::WeaponsRack, 10, 20)));
        // frameX 54 wraps the modulo — the next style's top-left.
        assert_eq!(te_anchor(471, 13, 20, 54, 0), Some((TeKind::WeaponsRack, 13, 20)));
    }

    #[test]
    fn doll_anchor_is_36_by_54() {
        // TileDrawing.cs:4223-4224: X % 36 / 18, Y % 54 / 18.
        assert_eq!(te_anchor(470, 11, 22, 18, 36), Some((TeKind::DisplayDoll, 10, 20)));
        // The womannequin column (frameX 72) wraps % 36 to 0.
        assert_eq!(te_anchor(470, 12, 20, 72, 0), Some((TeKind::DisplayDoll, 12, 20)));
    }

    #[test]
    fn hat_rack_anchor_is_54_by_72() {
        // TileDrawing.cs:4237-4238: X % 54 / 18, Y % 72 / 18.
        assert_eq!(te_anchor(475, 12, 23, 36, 54), Some((TeKind::HatRack, 10, 20)));
        assert_eq!(te_anchor(475, 10, 20, 0, 0), Some((TeKind::HatRack, 10, 20)));
    }

    #[test]
    fn other_tiles_have_no_te_anchor() {
        assert_eq!(te_anchor(21, 10, 20, 18, 18), None);
        assert_eq!(te_anchor(334, 10, 20, 5000, 0), None);
    }

    // -- GetItemLight --------------------------------------------------------

    #[test]
    fn pulse_items_truncate_every_channel_by_ess_scale() {
        // ItemSlot.cs:3782-3790: (byte)(channel * essScale).
        let (c, s) = get_item_light([200, 100, 50, 255], 520, 0.8, [0, 0, 0]);
        assert_eq!(s, 0.8);
        assert_eq!(c, [160, 80, 40, 204]);
    }

    #[test]
    fn heart_star_pulse_is_the_quarter_ess_floor() {
        // ItemSlot.cs:3791-3799: essScale*0.25 + 0.75.
        let (c, s) = get_item_light([255, 255, 255, 255], 58, 0.7, [0, 0, 0]);
        assert_eq!(s, 0.7 * 0.25 + 0.75);
        assert_eq!(c, [235, 235, 235, 235]); // (int)(255 * 0.925) = 235
    }

    #[test]
    fn disco_items_overwrite_rgb_and_force_alpha() {
        // ItemSlot.cs:3766-3781.
        let (c, s) = get_item_light([10, 20, 30, 40], 662, 0.8, [255, 128, 0]);
        assert_eq!(c, [255, 128, 0, 255]);
        assert_eq!(s, 1.0);
        let (c2, _) = get_item_light([10, 20, 30, 40], 5128, 0.8, [1, 2, 3]);
        assert_eq!(c2, [1, 2, 3, 255]);
    }

    #[test]
    fn plain_items_pass_through_and_out_of_range_keeps_the_colour() {
        let (c, s) = get_item_light([12, 34, 56, 78], 1, 0.8, [0, 0, 0]);
        assert_eq!(c, [12, 34, 56, 78]);
        assert_eq!(s, 1.0);
        // `type > ItemID.Count` (6196) — the guard is strict-greater.
        let (c2, _) = get_item_light([12, 34, 56, 78], 6197, 0.8, [0, 0, 0]);
        assert_eq!(c2, [12, 34, 56, 78]);
    }

    // -- DrawItem_GetColorAndScale -------------------------------------------

    #[test]
    fn size_clamp_tie_picks_the_height_divisor() {
        // ItemSlot.cs:2967-2970: 40x40 at limit 20 → 20/40 = 0.5.
        let (_, s) = draw_item_color_and_scale(1, 1.0, [255; 4], 20.0, 40, 40, 1.0, [0, 0, 0]);
        assert_eq!(s, 0.5);
        // 80x40: width > height → the WIDTH divisor.
        let (_, s2) = draw_item_color_and_scale(1, 1.0, [255; 4], 20.0, 80, 40, 1.0, [0, 0, 0]);
        assert_eq!(s2, 0.25);
        // Under the limit: no clamp, the item scale rides.
        let (_, s3) = draw_item_color_and_scale(1, 1.5, [255; 4], 20.0, 16, 16, 1.0, [0, 0, 0]);
        assert_eq!(s3, 1.5);
    }

    #[test]
    fn terraspark_at_the_20_limit_is_hardcoded_half() {
        // ItemSlot.cs:2971-2974: type 5669 && sizeLimit == 20 → num = 0.5.
        let (_, s) = draw_item_color_and_scale(5669, 1.0, [255; 4], 20.0, 30, 30, 1.0, [0, 0, 0]);
        assert_eq!(s, 0.5);
        // At the 40 limit the normal clamp applies (30 <= 40 → 1).
        let (_, s2) = draw_item_color_and_scale(5669, 1.0, [255; 4], 40.0, 30, 30, 1.0, [0, 0, 0]);
        assert_eq!(s2, 1.0);
    }

    #[test]
    fn pulse_scale_multiplies_into_the_final_scale() {
        // finalDrawScale = scale * num * scale2 (ItemSlot.cs:2975).
        let (_, s) = draw_item_color_and_scale(520, 1.0, [255; 4], 40.0, 20, 20, 0.8, [0, 0, 0]);
        assert_eq!(s, 0.8);
    }

    // -- Item.GetAlpha / GetColor --------------------------------------------

    #[test]
    fn boss_bags_lerp_toward_white() {
        // Item.cs:49172-49175: Lerp(newColor, White, 0.4), byte truncation.
        let c = item_get_alpha(3318, 0, 0, [0; 3], [100, 50, 0, 255]);
        assert_eq!(c, [162, 132, 102, 255]); // 100+155*0.4=162, 50+205*0.4=132
    }

    #[test]
    fn default_alpha_scales_rgb_and_clamps_the_subtraction() {
        // Item.cs:49324-49340: alpha 100 → num = 155/255.
        let c = item_get_alpha(1, 100, 0, [0; 3], [255, 128, 0, 200]);
        assert_eq!(c[0], (255.0 * 155.0 / 255.0) as u8); // 155
        assert_eq!(c[3], 100);
        // Alpha underflow clamps to 0, not a byte wrap.
        let c2 = item_get_alpha(1, 250, 0, [0; 3], [255, 255, 255, 100]);
        assert_eq!(c2[3], 0);
    }

    #[test]
    fn self_lit_arms_ignore_the_light() {
        assert_eq!(item_get_alpha(75, 0, 0, [0; 3], [10, 20, 30, 40]), [255; 4]);
        assert_eq!(item_get_alpha(520, 0, 0, [0; 3], [10, 20, 30, 40]), [255, 255, 255, 50]);
        assert_eq!(item_get_alpha(58, 0, 0, [0; 3], [10, 20, 30, 40]), [200, 200, 200, 200]);
        // The ghost arm rides gFade.
        assert_eq!(item_get_alpha(1446, 0, 77, [0; 3], [10, 20, 30, 40]), [10, 20, 30, 77]);
    }

    #[test]
    fn shimmer_bait_floors_take_the_per_channel_max() {
        // Item.cs:49296-49322: 4377's floor (50, 255, 50, 200).
        let c = item_get_alpha(4377, 0, 0, [0; 3], [100, 10, 200, 255]);
        assert_eq!(c, [100, 255, 200, 255]);
        // 5128's floor reads the disco RGB (:49303).
        let d = item_get_alpha(5128, 0, 0, [255, 128, 0], [10, 10, 10, 10]);
        assert_eq!(d, [255, 128, 10, 200]);
    }

    #[test]
    fn get_color_subtracts_the_complement_and_clamps() {
        // Item.cs:49346-49382.
        let c = item_get_color([255, 100, 0, 255], [200, 200, 200, 200]);
        assert_eq!(c, [200, 45, 0, 200]); // 100-(255-200)=45, 0-55 clamps 0
        let c2 = item_get_color([255, 255, 255, 255], [255; 4]);
        assert_eq!(c2, [255; 4]);
    }

    // -- the legacy arms ------------------------------------------------------

    fn item(kind: u16) -> TeItem {
        TeItem { kind, prefix: 0, stack: 1 }
    }

    #[test]
    fn air_items_draw_nothing() {
        let air = TeItem { kind: 0, prefix: 0, stack: 0 };
        assert!(item_frame_icon(&air, 1.0, 0, [0; 4], 0, [255; 4], 16, 16, 1.0, [0; 3], 5, 5, [0.0; 2]).is_none());
        assert!(weapons_rack_te_icon(&air, 1.0, 0, [0; 4], 0, [255; 4], 16, 16, 1.0, [0; 3], 5, 5, 0, [0.0; 2]).is_none());
        assert!(food_platter_draw(&air, 0, [0; 4], 0, [255; 4], 16, 16, false, 1.0, [0; 3], 5, 5, 0, [0.0; 2]).is_none());
    }

    #[test]
    fn item_frame_centres_on_the_2x2_middle_at_limit_20() {
        // TileDrawing.cs:7956: (x*16+16, y*16+16), sizeLimit 20.
        let d = item_frame_icon(&item(1), 1.0, 0, [0; 4], 0, [255; 4], 40, 40, 1.0, [0; 3], 10, 20, [1.0, 2.0]).unwrap();
        assert_eq!(d.center, [177.0, 338.0]);
        assert_eq!(d.scale, 0.5);
        assert!(!d.flip);
        assert_eq!(d.second, None);
    }

    #[test]
    fn weapons_rack_centres_at_24_and_flips_on_frame_54() {
        // TileDrawing.cs:7991-7998.
        let d = weapons_rack_te_icon(&item(1), 1.0, 0, [0; 4], 0, [255; 4], 40, 40, 1.0, [0; 3], 10, 20, 0, [0.0; 2]).unwrap();
        assert_eq!(d.center, [184.0, 344.0]);
        assert_eq!(d.scale, 1.0); // 40 px at limit 40 — no clamp
        assert!(!d.flip);
        let d2 = weapons_rack_te_icon(&item(1), 1.0, 0, [0; 4], 0, [255; 4], 40, 40, 1.0, [0; 3], 10, 20, 54, [0.0; 2]).unwrap();
        assert!(d2.flip);
    }

    #[test]
    fn food_platter_takes_the_bottom_third_and_integer_origin() {
        // TileDrawing.cs:7966-7978: Frame(1,3,0,2), origin (w/2, h).
        let d = food_platter_draw(&item(4009), 0, [0; 4], 0, [255; 4], 30, 90, true, 1.0, [0; 3], 10, 20, 0, [0.0; 2]).unwrap();
        assert_eq!(d.food_frame, Some([30, 30]));
        assert_eq!(d.origin, [15.0, 30.0]);
        assert_eq!(d.pos, [168.0, 338.0]); // (160+8, 320+16+2)
        assert!(d.flip); // frameX == 0
        assert_eq!(d.scale, 1.0); // no size clamp on the platter
    }

    #[test]
    fn non_food_platter_items_use_the_whole_sheet() {
        let d = food_platter_draw(&item(1), 0, [0; 4], 0, [255; 4], 24, 28, false, 1.0, [0; 3], 10, 20, 18, [0.0; 2]).unwrap();
        assert_eq!(d.food_frame, None);
        assert_eq!(d.origin, [12.0, 28.0]);
        assert!(!d.flip); // frameX 18
    }

    #[test]
    fn the_colour_second_pass_reads_the_unmodified_light() {
        // A pulse item dims itemLight but the second pass keeps the raw
        // environment colour (ItemSlot.cs:2882-2889).
        let d = item_frame_icon(&item(520), 1.0, 0, [255, 0, 0, 255], 0, [200, 200, 200, 255], 16, 16, 0.8, [0; 3], 0, 0, [0.0; 2]).unwrap();
        // itemLight = 200*0.8 = 160; GetAlpha(520) forces (255,255,255,50).
        assert_eq!(d.color, [255, 255, 255, 50]);
        // second = GetColor((255,0,0,255), (200,200,200,255)).
        assert_eq!(d.second, Some([200, 0, 0, 255]));
    }

    // -- the display doll pose ------------------------------------------------

    #[test]
    fn poses_0_to_3_are_always_valid() {
        // TEDisplayDoll.cs:138-141: (uint)testedPose <= 3.
        for p in 0..=3u8 {
            assert!(doll_pose_valid(p, None));
        }
        // A use pose needs the misc item's useStyle support.
        assert!(!doll_pose_valid(4, None));
        assert!(doll_pose_valid(4, Some(1)));
        assert!(!doll_pose_valid(8, Some(12))); // useStyle 12 stops at Use3
        assert!(doll_pose_valid(8, Some(5)));
        assert!(!doll_pose_valid(9, Some(1))); // out of the enum
    }

    #[test]
    fn the_swing_use_fractions_match_the_register_calls() {
        // TEDisplayDoll.cs:56-60.
        let rows = supported_use_poses(1);
        assert_eq!(rows.len(), 5);
        assert_eq!(rows[0].item_anim_percent, 1.0);
        assert_eq!(rows[3].item_anim_percent, 0.4143);
        assert_eq!(rows[4].item_anim_percent, 0.2);
        // The gun ladder's aim radians (:64-68).
        let gun = supported_use_poses(5);
        assert_eq!(gun[0].item_aim_radians, Some(-std::f32::consts::PI / 2.0));
        assert_eq!(gun[3].item_aim_radians, Some(0.7853981));
    }

    #[test]
    fn standing_doll_faces_left_and_stands_bottom_centred() {
        // TEDisplayDoll.cs:349,408-410,454 — frameX 0, no mount, pose 0.
        let d = display_doll_draw(0, 0, -1, None, 20.0, 42.0, 10, 20);
        assert_eq!(d.direction, -1);
        assert!(d.male);
        assert_eq!(d.pose, DollPose::Standing);
        assert_eq!(d.velocity, [0.0, 0.0]);
        // (10+1)*16 - 10 = 166; (20+3)*16 - 42 - 6 = 320.
        assert_eq!(d.position, [166.0, 320.0]);
        assert_eq!(d.item_animation, 0);
    }

    #[test]
    fn frame_36_faces_right_and_frame_72_is_the_womannequin() {
        // TEDisplayDoll.cs:352-359.
        let d = display_doll_draw(0, 36, -1, None, 20.0, 42.0, 10, 20);
        assert_eq!(d.direction, 1);
        assert!(d.male);
        let w = display_doll_draw(0, 72, -1, None, 20.0, 42.0, 10, 20);
        assert_eq!(w.direction, -1); // 72 % 72 == 0
        assert!(!w.male);
    }

    #[test]
    fn walking_pins_leg_row_9_and_velocity_direction() {
        // TEDisplayDoll.cs:411-415.
        let d = display_doll_draw(3, 0, -1, None, 20.0, 42.0, 10, 20);
        assert_eq!(d.pose, DollPose::Walking);
        assert_eq!(d.velocity, [-1.0, 0.0]);
        assert_eq!(d.leg_frame_row, 9);
    }

    #[test]
    fn sitting_lifts_14_and_jumping_falls() {
        // TEDisplayDoll.cs:416-423.
        let s = display_doll_draw(1, 0, -1, None, 20.0, 42.0, 10, 20);
        assert!(s.sitting);
        assert_eq!(s.sit_lift, 14.0);
        assert_eq!(s.position[1], 334.0); // 320 + 14
        let j = display_doll_draw(2, 0, -1, None, 20.0, 42.0, 10, 20);
        assert_eq!(j.velocity, [0.0, 1.0]);
    }

    #[test]
    fn a_mount_forces_sitting_and_jumping_to_standing() {
        // TEDisplayDoll.cs:398-405.
        let d = display_doll_draw(1, 0, 3, None, 20.0, 42.0, 10, 20);
        assert_eq!(d.pose, DollPose::Standing);
        assert_eq!(d.mount_type, 3);
        let j = display_doll_draw(2, 0, 3, None, 20.0, 42.0, 10, 20);
        assert_eq!(j.pose, DollPose::Standing);
        // Walking survives the mount.
        let w = display_doll_draw(3, 0, 3, None, 20.0, 42.0, 10, 20);
        assert_eq!(w.pose, DollPose::Walking);
    }

    #[test]
    fn an_invalid_use_pose_falls_to_standing() {
        // TEDisplayDoll.cs:394-397 — pose 4 with no misc item.
        let d = display_doll_draw(4, 0, -1, None, 20.0, 42.0, 10, 20);
        assert_eq!(d.pose, DollPose::Standing);
        assert_eq!(d.item_animation, 0);
    }

    #[test]
    fn a_use_pose_sets_the_1000_tick_animation_and_aim() {
        // TEDisplayDoll.cs:437-447 — useStyle 5, Use4 (aim +0.7853981),
        // direction -1 negates the aim.
        let d = display_doll_draw(7, 0, -1, Some(5), 20.0, 42.0, 10, 20);
        assert_eq!(d.pose, DollPose::Use4);
        assert_eq!(d.item_animation_max, 1000);
        assert_eq!(d.item_animation, 500);
        assert_eq!(d.item_rotation, -0.7853981);
        // Facing right keeps the aim sign.
        let r = display_doll_draw(7, 36, -1, Some(5), 20.0, 42.0, 10, 20);
        assert_eq!(r.item_rotation, 0.7853981);
        // The swing family's Use2 is 0.8 of 1000.
        let s = display_doll_draw(5, 0, -1, Some(1), 20.0, 42.0, 10, 20);
        assert_eq!(s.item_animation, 800);
        assert_eq!(s.item_rotation, 0.0);
    }

    // -- the hat rack ----------------------------------------------------------

    #[test]
    fn hat_rack_anchor_and_double_direction_flip() {
        // TEHatRack.cs:168-199 — frameX 0: direction -1, first hat draws
        // at +1 (the :185 flip), second at -1 (the :198 flip back).
        let h = hat_rack_draw(0, 20.0, 42.0, 10, 20);
        // anchor = (11.5*16, 24*16) = (184, 384).
        assert_eq!(h.anchor, [184.0, 384.0]);
        assert_eq!(h.direction, 1);
        // hat0: 184 - 10 + 14 = 188; 384 - 48 - 2 = 334.
        assert_eq!(h.hat0_pos, [188.0, 334.0]);
        // hat1: 184 - 10 - 12 = 162; 384 - 48 + 16 = 352.
        assert_eq!(h.hat1_pos, [162.0, 352.0]);
    }

    #[test]
    fn hat_rack_frame_54_faces_the_other_way() {
        // TEHatRack.cs:170-173: frameX % 216 == 54 → direction +1.
        let h = hat_rack_draw(54, 20.0, 42.0, 10, 20);
        assert_eq!(h.direction, -1);
        assert_eq!(h.hat0_pos, [160.0, 334.0]); // 184 - 10 - 14
        assert_eq!(h.hat1_pos, [186.0, 352.0]); // 184 - 10 + 12
    }

    // -- the held-projectile preview -------------------------------------------

    fn doll_env(direction: i32) -> DollProjEnv {
        // A doll at position (166, 320), 20x42, body row 0, unmounted.
        DollProjEnv {
            direction,
            mounted_center: [176.0, 341.0],
            center: [176.0, 341.0],
            bottom: [176.0, 362.0],
            arm_position: [176.0, 341.0],
            body_frame_row: 0,
            grav_dir: 1.0,
            misc_counter: 0,
            mount_player_offset: 0,
            sitting: false,
            seat_y_offset: 0,
        }
    }

    fn use1() -> UsePose {
        UsePose { pose: DollPose::Use1, item_anim_percent: 0.5, item_aim_radians: None }
    }

    #[test]
    fn the_draw_use_style_set_overrides_to_5() {
        // ProjectileID.cs:188 — the listed kinds preview under useStyle 5.
        assert_eq!(display_doll_draw_use_style(301), 5);
        assert_eq!(display_doll_draw_use_style(507), 5);
        assert_eq!(display_doll_draw_use_style(1), -1);
        assert_eq!(DISPLAY_DOLL_DRAW_USE_STYLE.len(), 67);
    }

    #[test]
    fn non_use_poses_never_preview() {
        // Projectile.cs:22585-22588 — only poses 4-8 set the flag.
        let env = doll_env(-1);
        let standing = UsePose { pose: DollPose::Standing, item_anim_percent: 0.0, item_aim_radians: None };
        let d = doll_proj_defaults(507).unwrap();
        assert!(ai_display_doll(507, &d, &env, &standing, 10.0, 1.0).is_none());
    }

    #[test]
    fn the_javelin_preview_centres_on_the_hand_plus_12() {
        // Projectile.cs:22595-22610 — aiStyle 1, only 507, +12 along the
        // aim then (-2,-2), alpha 0, drawLayer 7.
        let env = doll_env(-1);
        let d = doll_proj_defaults(507).unwrap();
        let out = ai_display_doll(507, &d, &env, &use1(), 10.0, 1.0).unwrap();
        assert_eq!(out.draw_layer, 7);
        assert_eq!(out.alpha, 0);
        // direction -1, aim 0 → spin = (-10, 0), normalized (-1, 0).
        // hand = MountedCenter + (-10,-21) + onhand(6,19)*2 mirrored:
        //   vector.X = 40 - 12 = 28, minus (10,7) → (18, 31); hand =
        //   (176-10+18, 341-21+31) = (184, 351). +12*(-1,0) = (172,351),
        //   +(-2,-2) = (170, 349).
        assert_eq!(out.center, [170.0, 349.0]);
        // A non-507 aiStyle-1 kind refuses.
        let other = DollProjDefaults { ai_style: 1, ..d };
        assert!(ai_display_doll(2, &other, &env, &use1(), 10.0, 1.0).is_none());
    }

    #[test]
    fn phaseblades_refuse_the_preview() {
        // Projectile.cs:22730-22734 — 866 + IsAPhaseblade.
        let env = doll_env(-1);
        let d = doll_proj_defaults(866).unwrap();
        assert!(ai_display_doll(866, &d, &env, &use1(), 10.0, 1.0).is_none());
        let pb = DollProjDefaults { ai_style: 3, ..d };
        assert!(ai_display_doll(1059, &pb, &env, &use1(), 10.0, 1.0).is_none());
    }

    #[test]
    fn the_light_disc_adds_the_direction_nudge() {
        // Projectile.cs:22744-22768 — 182, direction 1: rotation +PI/2,
        // centre (-4,0) then Use1 (+4*dir, 0).
        let env = doll_env(1);
        let d = doll_proj_defaults(182).unwrap();
        let out = ai_display_doll(182, &d, &env, &use1(), 10.0, 1.0).unwrap();
        assert_eq!(out.draw_layer, 7);
        // hand (direction 1): vector (12,38) - (10,7) = (2,31);
        //   hand = (176-10+2, 341-21+31) = (168, 351).
        //   +(-4,0) + (4,0) = (168, 351).
        assert_eq!(out.center, [168.0, 351.0]);
    }

    #[test]
    fn the_spear_preview_uses_the_offset_helper_at_max_100() {
        // Projectile.cs:22878-22924 — aiStyle 19, the 46 spear: the
        // offset helper (itemAnimation 25, max 100) → num3 = 33,
        // num4 = min(25,33) = 25, num5 = 0 → 3 + 1.4*(67-0) - 1.6*(33-25)
        // = 3 + 93.8 - 12.8 = 84. Centre = MountedCenter + unit*84.
        let env = doll_env(1);
        let d = DollProjDefaults { w: 20.0, h: 20.0, ai_style: 19, scale: 1.0, alpha: 0, draw_layer: 0 };
        let out = ai_display_doll(46, &d, &env, &use1(), 10.0, 1.0).unwrap();
        assert_eq!(out.sprite_direction, -1); // -direction
        // unit (1,0); the offset is f32: 3 + 1.4*67 - 1.6*8 = 84.0 (the
        // 1.4f/1.6f literals round in f32), centre (176+84, 341).
        assert_eq!(out.center, [260.0, 341.0]);
    }

    #[test]
    fn the_magic_dagger_floors_the_onhand_centre() {
        // Projectile.cs:23062-23089 — aiStyle 100, 535: the on-hand
        // offset, direction -1 mirrored, floored.
        let env = doll_env(-1);
        let d = doll_proj_defaults(535).unwrap();
        let out = ai_display_doll(535, &d, &env, &use1(), 10.0, 1.0).unwrap();
        assert_eq!(out.draw_layer, 8);
        // row 0 onhand (6,19)*2 = (12,38); direction -1 → X = 40-12 = 28;
        //   minus (10,7) → (18,31). rel = MountedCenter - (10,21) +
        //   (18,31) + zero(-6,-14) = (176-10+18-6, 341-21+31-14) =
        //   (178, 337). velocity 0 → floor (178, 337).
        assert_eq!(out.center, [178.0, 337.0]);
        // The 535 frame machine: miscCounter 0 → frame 0.
        assert_eq!(out.frame, 0);
    }

    #[test]
    fn the_misc_counter_frame_remaps_the_300_cycle() {
        // Player.cs:4253-4274 — framerate 6, min 0, max 8: num3 grows by
        // 48 per step, 300/48 → num2 = 8 (7 steps to 336 ≥ 300, +1).
        // num4 = 1/8; miscCounter 0 → t 0 → frame 0.
        assert_eq!(calculate_frame_from_misc_counter(0, 6, 0, 8), 0);
        // miscCounter 150 → t = 0.5 % 0.125 = 0 → frame 0 (the wrap).
        assert_eq!(calculate_frame_from_misc_counter(150, 6, 0, 8), 0);
        // miscCounter 37 → t = 37/300 = 0.12333, num4 = 0.125, t < num4
        //   so no wrap; lerp = 0.12333/0.125 = 0.98666 → (int)(8*0.98666)
        //   = 7.
        assert_eq!(calculate_frame_from_misc_counter(37, 6, 0, 8), 7);
    }

    #[test]
    fn unknown_ai_styles_have_no_preview() {
        let env = doll_env(-1);
        let d = DollProjDefaults { w: 10.0, h: 10.0, ai_style: 99, scale: 1.0, alpha: 0, draw_layer: 0 };
        // aiStyle 99 IS ported; use an unported one (e.g. 42).
        let d42 = DollProjDefaults { ai_style: 42, ..d };
        assert!(ai_display_doll(1, &d42, &env, &use1(), 10.0, 1.0).is_none());
    }
}
