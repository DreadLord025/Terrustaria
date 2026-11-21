//! Item tooltip (`Main.MouseText`, `Utils.DrawInvBG`). Filled in by Task 4 of the
//! Whole-UI P1 plan.

use glam::Vec2;
use tr_render::SpriteRenderer;

use super::slot;
use super::UiPulse;
use super::text::{hud_text, hud_text_width};
use crate::assets;
use crate::item::{HOTBAR, INV, ItemSlot};
use crate::registry;

/// `toolTipDistance` (`Main.cs:407`, set to 6 at `Main.cs:66563`).
const TOOL_TIP_DISTANCE: f32 = 6.0;
/// Opaque-box margin (`num4 = 18`, `Main.cs:20385`) used for the on-screen clamp.
const BOX_MARGIN: f32 = 18.0;
/// Opaque box offset (`X += 8; Y += 2`, `Main.cs:20383-20384`).
const BOX_OFFSET: Vec2 = Vec2::new(8.0, 2.0);
/// Box left/top inset from the text origin (`num8 = 14`, `num9 = 9`, `Main.cs:20400-20401`).
const BOX_INSET: Vec2 = Vec2::new(14.0, 9.0);
/// Box height padding: int `num9 + num9/2` = `9 + 4` = 13 (`Main.cs:20402`).
const BOX_HEIGHT_PAD: f32 = 13.0;
/// `Main.cs:20402`: `new Color(23, 25, 81, 255) * 0.925f`.
const TOOLTIP_BG: [f32; 4] = [
    23.0 / 255.0 * 0.925,
    25.0 / 255.0 * 0.925,
    81.0 / 255.0 * 0.925,
    0.925,
];

/// The nine destination rectangles of `Utils.DrawInvBG` (`Utils.cs:2548-2572`): 10 px
/// corners, the middle/corner cells stretched to fill, clamped to a 20x20 minimum
/// (`Utils.cs:2555-2562`).
pub fn inv_bg_rects(rect: [f32; 4]) -> [[f32; 4]; 9] {
    let [x, y, mut w, mut h] = rect;
    if w < 20.0 {
        w = 20.0;
    }
    if h < 20.0 {
        h = 20.0;
    }
    [
        [x, y, 10.0, 10.0],
        [x + 10.0, y, w - 20.0, 10.0],
        [x + w - 10.0, y, 10.0, 10.0],
        [x, y + 10.0, 10.0, h - 20.0],
        [x + 10.0, y + 10.0, w - 20.0, h - 20.0],
        [x + w - 10.0, y + 10.0, 10.0, h - 20.0],
        [x, y + h - 10.0, 10.0, 10.0],
        [x + 10.0, y + h - 10.0, w - 20.0, 10.0],
        [x + w - 10.0, y + h - 10.0, 10.0, 10.0],
    ]
}

/// One 10x10 source cell of `Inventory_Back13`, in the texture's own pixels.
fn inv_bg_srcs(w: f32, h: f32) -> [[f32; 4]; 9] {
    // `Utils.cs:2563-2571`: the corners read 10x10 from the four edges; the middle
    // cells read `(10, 10, 10, 10)` and are stretched by the destination rect.
    [
        [0.0, 0.0, 10.0, 10.0],
        [10.0, 0.0, 10.0, 10.0],
        [w - 10.0, 0.0, 10.0, 10.0],
        [0.0, 10.0, 10.0, 10.0],
        [10.0, 10.0, 10.0, 10.0],
        [w - 10.0, 10.0, 10.0, 10.0],
        [0.0, h - 10.0, 10.0, 10.0],
        [10.0, h - 10.0, 10.0, 10.0],
        [w - 10.0, h - 10.0, 10.0, 10.0],
    ]
}

/// Draws the `Inventory_Back13` 9-slice over `rect` (`Utils.DrawInvBG`). Missing the
/// sheet draws nothing.
pub fn draw_inv_bg(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    rect: [f32; 4],
    color: [f32; 4],
) {
    let Some(sheet) = lookups.ui_inv_bg13.as_ref() else {
        return;
    };
    let (aw, ah) = (lookups.atlas_size.0 as f32, lookups.atlas_size.1 as f32);
    let srcs = inv_bg_srcs(sheet.width as f32, sheet.height as f32);
    let dst = inv_bg_rects(rect);
    for i in 0..9 {
        let [dx, dy, dw, dh] = dst[i];
        if dw <= 0.0 || dh <= 0.0 {
            continue;
        }
        let [sx, sy, sw, sh] = srcs[i];
        let x0 = sheet.x as f32 + sx;
        let y0 = sheet.y as f32 + sy;
        let uv = [
            x0 / aw,
            y0 / ah,
            (x0 + sw) / aw,
            (y0 + sh) / ah,
        ];
        r.push_ui_sprite([dx, dy], [dw, dh], uv, color);
    }
}

/// `Main.MouseText_DrawItemTooltip_GetItemNameColor` (`Main.cs:20557-20620`): the
/// item-rare colour table; `rare == 0` and any unknown value are white.
pub fn item_name_color(rare: i32) -> [f32; 4] {
    fn rgb(r: f32, g: f32, b: f32) -> [f32; 4] {
        [r / 255.0, g / 255.0, b / 255.0, 1.0]
    }
    match rare {
        -11 => rgb(255.0, 175.0, 0.0),
        -1 => rgb(130.0, 130.0, 130.0),
        1 => rgb(150.0, 150.0, 255.0),
        2 => rgb(150.0, 255.0, 150.0),
        3 => rgb(255.0, 200.0, 150.0),
        4 => rgb(255.0, 150.0, 150.0),
        5 => rgb(255.0, 150.0, 255.0),
        6 => rgb(210.0, 160.0, 255.0),
        7 => rgb(150.0, 255.0, 10.0),
        8 => rgb(255.0, 255.0, 10.0),
        9 => rgb(5.0, 200.0, 255.0),
        10 => rgb(255.0, 40.0, 100.0),
        // `rare >= 11` (`Main.cs:20616-20619`). `-12`/`-13` use the dynamic
        // disco/master colours and are not modelled (no such items yet).
        11..=i32::MAX => rgb(180.0, 40.0, 255.0),
        _ => [1.0, 1.0, 1.0, 1.0],
    }
}

/// The tooltip box (`Main.MouseText_DrawItemTooltip`, `Main.cs:20196-20402`): from the
/// mouse, `+14` (`Main.cs:20196`), `+toolTipDistance` (`Main.cs:20378`), then the
/// opaque-box `(8, 2)` offset (`Main.cs:20383-20384`); clamped so the text plus the
/// 18 px margin stays on-screen (`Main.cs:20389-20395`). The returned rect is the
/// `DrawInvBG` box `(X-14, Y-9, w+28, h+13)` (`Main.cs:20402`).
pub fn tooltip_box_rect(mouse: Vec2, w: f32, h: f32, screen: Vec2) -> [f32; 4] {
    let mut x = mouse.x + 14.0 + TOOL_TIP_DISTANCE + BOX_OFFSET.x;
    let mut y = mouse.y + 14.0 + TOOL_TIP_DISTANCE + BOX_OFFSET.y;
    if x + w + BOX_MARGIN > screen.x {
        x = screen.x - w - BOX_MARGIN;
    }
    if y + h + BOX_MARGIN > screen.y {
        y = screen.y - h - BOX_MARGIN;
    }
    [
        x - BOX_INSET.x,
        y - BOX_INSET.y,
        w + BOX_INSET.x * 2.0,
        h + BOX_HEIGHT_PAD,
    ]
}

/// The item tooltip's box + text origin under the
/// `Main.SettingsEnabled_OpaqueBoxBehindTooltips` gate (`Main.cs:20381-20403`):
/// with the box **on** the `(8, 2)` offset, the 18 px clamp margin and the
/// `DrawInvBG` rect apply ([`tooltip_box_rect`]); with it **off** the text
/// sits at `mouse + 14 + toolTipDistance` with the plain `num4 = 4` clamp
/// margin (`Main.cs:20380`) and no box is drawn. Returns
/// `(box rect or None, text x, text y)`.
pub fn item_tooltip_layout(
    mouse: Vec2,
    w: f32,
    h: f32,
    screen: Vec2,
    opaque_box: bool,
) -> (Option<[f32; 4]>, f32, f32) {
    if opaque_box {
        let rect = tooltip_box_rect(mouse, w, h, screen);
        return (Some(rect), rect[0] + BOX_INSET.x, rect[1] + BOX_INSET.y);
    }
    let mut x = mouse.x + 14.0 + TOOL_TIP_DISTANCE;
    let mut y = mouse.y + 14.0 + TOOL_TIP_DISTANCE;
    // `num4 = 4` without the box (`Main.cs:20380`, `:20389-20395`).
    if x + w + 4.0 > screen.x {
        x = screen.x - w - 4.0;
    }
    if y + h + 4.0 > screen.y {
        y = screen.y - h - 4.0;
    }
    (None, x, y)
}

/// One `MouseText` line height (`DynamicSpriteFont.MeasureString(...).Y`). The built-in
/// bitmap fallback draws 8 px glyphs (`SpriteRenderer::push_text`).
fn line_height(lookups: &assets::Lookups) -> f32 {
    super::text::line_height(lookups)
}

/// `Main.MouseText_DrawItemTooltip_GetLinesInfo` (`Main.cs:20631-20875`) —
/// The hovered item's context for the tooltip rows: the prefix (the affix
/// name), the displayed stack and the local player — the player-side layers
/// the C# folds in (`Main.MouseText_DrawItemTooltip_GetLinesInfo`,
/// `Main.cs:20631+`): the weapon-damage multiplier, the crit bonuses,
/// `minionKB`, the quiver, the knockback gloves and the mana cost.
#[derive(Clone, Copy)]
pub struct TooltipContext<'a> {
    /// The item's rolled prefix (`Item.prefix`) — the affix name
    /// (`Item.AffixName`, `Item.cs:1194-1201`).
    pub prefix: u16,
    /// The displayed stack (`ItemSlot.EstimateDisplayStack`,
    /// `ItemSlot.cs:3150-3162`; the bulk-buy / bulk-craft arms belong to the
    /// shop's and the crafting panel's, so the plain stack is the port's).
    pub stack: u32,
    /// The local player.
    pub player: Option<&'a crate::player::Player>,
    /// `Main.LocalPlayer.inventory[Main.LocalPlayer.selectedItem].crit` — the
    /// held item's own crit, which the crit row subtracts (`Main.cs:20701`).
    pub held_crit: i32,
    /// The held item's kind — the Psycho Knife arm
    /// (`Main.cs:20762`: held 3106 && hovered 3106).
    pub held_kind: u16,
    /// `Main.player[myPlayer].HasItem(905)` (`Player.cs:55894-55904`, the
    /// 58-slot scan): the Coin Gun lets the coin kinds 71-74 show their
    /// damage row (`Main.cs:20669`).
    pub has_coin_gun: bool,
    /// `Main.npcShop > 0` (`Main.cs:20818`): the shop's equipable row swaps
    /// to `Lang.tip[60]` for the Discount Card 854 / Greedy Ring 3035.
    pub shop_open: bool,
    /// `NPC.downedPlantBoss` (`Main.cs:20961`): the biome keys' first
    /// tooltip line stays cursed until Plantera falls.
    pub downed_plant_boss: bool,
    /// `Main.tenthAnniversaryWorld` (`Main.cs:20973`) — the Wizard Hat 238
    /// anniversary row.
    pub anniversary: bool,
    /// `Main.getGoodWorld` (`Main.cs:20978`) — the Crispy Honey Block 1127
    /// burning row.
    pub get_good_world: bool,
    /// `SpecialSeedFeatures.Mechdusa` (`SpecialSeedFeatures.cs:119-128` =
    /// `remixWorld && getGoodWorld`) — the mech-summon tooltip swap
    /// (`Main.cs:20983-20999`).
    pub mechdusa: bool,
    /// `Main.player[myPlayer].downedDD2EventAnyDifficulty`
    /// (`Main.cs:21000`) — the DD2 poppers' Etherian-Mana row.
    pub downed_dd2_any: bool,
    /// `Main.expertMode` (`Main.cs:21005`) — the well-fed life-regen row.
    pub expert_mode: bool,
    /// `Main.masterColor` (`Main.cs:18225-18234`'s ±0.05 oscillation,
    /// `DoUpdate_AnimateCursorColors`): the −13 master name colour's green
    /// channel. The port has no animated global yet — 1.0 (the static
    /// default, `Main.cs:66833`).
    pub master_color: f32,
    /// `Main.DiscoR/G/B` (`Main.cs:20564-20567`): the −12 expert name
    /// colour; the caller passes the shared cycle (`App.disco`).
    pub disco: [u8; 3],
    /// `Item.favorited` (`Main.cs:20645-20663`): the favourite rows — the
    /// loadout-share arm's "Shared with other loadouts" or the plain
    /// tip[56]/tip[57] pair.
    pub favorited: bool,
    /// `Item.tooltipContext` (`ItemSlot.cs:3184`): the hovered slot's
    /// `ItemSlot.Draw` context — 8/10 worn armour, 9/11 social vanity,
    /// 8-12 the loadout-share contexts. −1 = no slot context (the
    /// crafting-recipe hover passes 22).
    pub context: i32,
    /// The open bank's index when `Player.chest` is −2..−5
    /// (`bank::bank_for_chest`): the favourite row's
    /// `IsBlockedFromTransferIntoChest` arm (`Main.cs:20659`).
    pub open_bank: Option<usize>,
    /// `Player.Settings.ShowLoadoutShareHint` (`Player.cs:372`, static
    /// ctor true `:412`): the share-hint row (`Main.cs:21312`).
    pub show_loadout_share_hint: bool,
}

/// The world-side gates the tooltip rows read (`Main.cs:20961-21031`) —
/// the `draw_hud` caller derives them from the `World` once per frame.
#[derive(Clone, Copy, Debug)]
pub struct TooltipWorld {
    /// `NPC.downedPlantBoss` (`Main.cs:20961`).
    pub downed_plant_boss: bool,
    /// `Main.tenthAnniversaryWorld` (`Main.cs:20973`).
    pub anniversary: bool,
    /// `Main.getGoodWorld` (`Main.cs:20978`).
    pub get_good_world: bool,
    /// `SpecialSeedFeatures.Mechdusa` (`SpecialSeedFeatures.cs:119-128`).
    pub mechdusa: bool,
    /// `Main.player[myPlayer].downedDD2EventAnyDifficulty`
    /// (`Main.cs:21000`) — the port derives it from the three tier
    /// latches (`world.rs:450-452`, the `sentry.rs:341` precedent).
    pub downed_dd2_any: bool,
    /// `Main.expertMode` (`Main.cs:21005`) — the player's difficulty flag
    /// (`buffs.expert_mode`, the `world_sim/items.rs:1556` precedent).
    pub expert_mode: bool,
    /// `Main.DiscoR/G/B` — the shared cycle (`App.disco`).
    pub disco: [u8; 3],
}

impl Default for TooltipWorld {
    fn default() -> Self {
        Self {
            downed_plant_boss: false,
            anniversary: false,
            get_good_world: false,
            mechdusa: false,
            downed_dd2_any: false,
            expert_mode: false,
            disco: [255, 0, 0],
        }
    }
}

impl Default for TooltipContext<'_> {
    fn default() -> Self {
        Self {
            prefix: 0,
            stack: 1,
            player: None,
            held_crit: 0,
            held_kind: 0,
            has_coin_gun: false,
            shop_open: false,
            downed_plant_boss: false,
            anniversary: false,
            get_good_world: false,
            mechdusa: false,
            downed_dd2_any: false,
            expert_mode: false,
            master_color: 1.0,
            disco: [255, 0, 0],
            favorited: false,
            context: -1,
            open_bank: None,
            show_loadout_share_hint: true,
        }
    }
}

/// `Item.GetHoverName(item, stack)` (`Item.cs:438-446`): the affix name
/// (`Item.AffixName`, `:1194-1201`) plus the stack suffix.
fn hover_name(kind: u16, prefix: u16, stack: u32) -> String {
    let mut name = crate::lang::prefixed_item_name(kind, prefix);
    if stack > 1 {
        name = format!("{name} ({stack})");
    }
    name
}

/// `ItemID.Sets.ToolTipDamageMultiplier` (`ItemID.cs:492`): 1 by default, 2
/// for the nine ids the vanilla doubles.
fn tooltip_damage_multiplier(kind: u16) -> f32 {
    match kind {
        162 | 163 | 220 | 389 | 1259 | 4272 | 5011 | 5012 | 5526 => 2.0,
        _ => 1.0,
    }
}

/// The player's crit for the hovered item's class (`Player.meleeCrit` /
/// `rangedCrit` / `magicCrit`, `Main.cs:20701-20717`).
fn player_crit(p: &crate::player::Player, class: registry::DamageClass) -> i32 {
    match class {
        registry::DamageClass::Melee => p.melee_crit,
        registry::DamageClass::Ranged => p.ranged_crit,
        registry::DamageClass::Magic => p.magic_crit,
        _ => 0,
    }
}

/// The item's tooltip rows — `Main.MouseText_DrawItemTooltip_GetLinesInfo`
/// (`Main.cs:20631-21010`): line 0 is the hover name in its rare colour
/// (`:20358-20360`), then the stat/tool rows each behind its own value gate,
/// through `Lang.tip[i]` (`LegacyTooltip.{i}`, `Lang.cs:497`) and the
/// `CommonItemTooltip.*` language rows.
pub fn item_tooltip_lines(
    kind: u16,
    d: &crate::itemdata::ItemDef,
    ctx: TooltipContext<'_>,
) -> Vec<(String, [f32; 4])> {
    // The C# reads the PREFIXED hovered instance (`Item.Prefix` already
    // applied, `Item.cs:576-586`): every stat below rides the prefixed
    // stats, and the name colour the prefixed rare (`Main.cs:20328`).
    let ps = crate::prefix::prefixed_stats_of(d, ctx.prefix);
    // `Main.cs:20333` — `oldKB` is captured BEFORE the caller's
    // glove/buff/shroomite scales mutate `hoverItem.knockBack`
    // (`:20333-20350`): the prefix comparison (`:21124`) reads the
    // PREFIX-only knockback, while the word tier reads the scaled one.
    let old_kb = ps.knock_back;
    // `:20636-20663` — the favourite / loadout-share head. `flag` latches
    // the loadout-share arm (it suppresses the equipable row, `:20816`);
    // `flag2` latches the unfavourited shareable slot (the share-hint row
    // at the tail, `:21312`). The port has no loadout system, so
    // `GetSharedLoadoutItem` never answers (`ItemSlot.cs:3171-3174`'s
    // shared-item arm is unreachable) — the favourited arm prints
    // `UI.ItemLoadoutShared` directly.
    let mut loadout_share_flag = false;
    let mut share_hint_flag = false;
    let mut lines: Vec<(String, [f32; 4])> = vec![(
        hover_name(kind, ctx.prefix, ctx.stack),
        // `:20328-20332` — the expert items force rare −12 (the disco
        // colour); `:20560-20567` — −13 reads masterColor, −12 the shared
        // disco cycle. The diff 1/2 mediumcore/hardcore overrides
        // (`:20620-20627`) stay unported (no difficulty flag).
        if super::tooltip_sets::expert_only(kind) {
            super::color::tint([ctx.disco[0], ctx.disco[1], ctx.disco[2], 255])
        } else if ps.rare == -13 {
            // `(byte)(Main.masterColor * 200f)` (`:20562`) — masterColor
            // stays in [0, 1], so the truncation cannot wrap.
            [1.0, (ctx.master_color * 200.0) as u8 as f32 / 255.0, 0.0, 1.0]
        } else {
            item_name_color(ps.rare)
        },
    )];
    if ctx.context >= 0 && super::tooltip_sets::can_loadout_share_at(ctx.context) {
        if ctx.favorited {
            lines.push((
                crate::lang::text("UI.ItemLoadoutShared"),
                [1.0, 1.0, 1.0, 1.0],
            ));
        } else {
            share_hint_flag = true;
        }
        loadout_share_flag = true;
    } else if ctx.favorited {
        lines.push((crate::lang::tip(56), [1.0, 1.0, 1.0, 1.0]));
        lines.push((crate::lang::tip(57), [1.0, 1.0, 1.0, 1.0]));
        // `:20659-20662` — the favorited bank-bound items refuse their own
        // container (`ChestUI.IsBlockedFromTransferIntoChest`).
        if super::tooltip_sets::blocked_from_transfer_into_chest(kind, ctx.open_bank) {
            lines.push((
                crate::lang::text("UI.ItemCannotBePlacedInsideItself"),
                [1.0, 1.0, 1.0, 1.0],
            ));
        }
    }
    // `:20664-20668` — the social-slot row: contexts 9/11 set `social`
    // (`ItemSlot.cs:3191-3194`); the vanity kinds and the info accessories
    // / music boxes (hasVanityEffects) keep their function.
    let social = matches!(ctx.context, 9 | 11);
    if social && !d.vanity && !super::tooltip_sets::has_vanity_effects(kind) {
        lines.push((crate::lang::tip(61), [1.0, 1.0, 1.0, 1.0]));
    }
    // `:20669` — the damage gate: `damage > 0 && (!notAmmo || useStyle != 0)
    // && (type < 71 || type > 74 || HasItem(905))`. The coins 71-74 are
    // notAmmo placeables — their damage row only shows while the Coin Gun
    // is carried.
    if ps.damage > 0
        && (!crate::item::not_ammo(kind) || d.use_style != 0)
        && (!(71..=74).contains(&kind) || ctx.has_coin_gun)
    {
        // Damage (`:20672-20700`): the item's own damage × the per-item
        // tooltip multiplier, then the rapid-attack ramp
        // (`ItemID.Sets.RapidAttackBonusDamage`, `:20673-20676` — the
        // read-only `ApplyRapidAttackBonus`), then the player's
        // weapon-damage multiplier.
        let mut base = (ps.damage as f32 * tooltip_damage_multiplier(kind)) as i32;
        if let Some(p) = ctx.player {
            base = p.apply_rapid_attack_bonus(base, kind);
        }
        let multiplier = ctx.player.map_or(1.0, |p| {
            p.weapon_damage_multiplier_with_ammo(d.class, kind, d.use_ammo)
        });
        let shown = (multiplier * base as f32 + 5e-6) as i32;
        // The class suffix (melee tip[2], ranged tip[3], magic tip[4],
        // summon tip[53], else tip[55]).
        let suffix = match d.class {
            registry::DamageClass::Melee => 2,
            registry::DamageClass::Ranged => 3,
            registry::DamageClass::Magic => 4,
            registry::DamageClass::Summon => 53,
            _ => 55,
        };
        lines.push((
            format!("{}{}", shown, crate::lang::tip(suffix)),
            [1.0, 1.0, 1.0, 1.0],
        ));
        // Crit (`:20701-20717`) — the melee/ranged/magic classes only: the
        // player's crit, minus the held item's own, plus the hovered item's
        // `GetVisualCritChance` (`Item.cs:637-645`: the Revolver 2269 adds
        // the charged bonus, clamped to [0, 100]).
        if matches!(
            d.class,
            registry::DamageClass::Melee | registry::DamageClass::Ranged | registry::DamageClass::Magic
        ) {
            let mut visual_crit = ps.crit;
            if kind == 2269 {
                if let Some(p) = ctx.player {
                    if p.revolver_crit_chance_bonus > 0 {
                        visual_crit = (visual_crit + p.revolver_crit_chance_bonus).clamp(0, 100);
                    }
                }
            }
            let crit = ctx
                .player
                .map_or(visual_crit, |p| player_crit(p, d.class) - ctx.held_crit + visual_crit);
            lines.push((
                format!("{}{}", crit, crate::lang::tip(5)),
                [1.0, 1.0, 1.0, 1.0],
            ));
        }
        // Use time (`:20717-20752`) — the `useAnimation` tiers behind
        // `useStyle != 0 && (!summon || IsAWhip[shoot])`: the summon staffs
        // show no speed row, the whips do (`ProjectileID.cs:193`).
        let whip = d
            .projectile
            .is_some_and(|p| tr_core::whip::is_whip(p));
        if d.use_style != 0 && (!d.summon || whip) {
            let tier = if ps.use_animation <= 8 {
                6
            } else if ps.use_animation <= 20 {
                7
            } else if ps.use_animation <= 25 {
                8
            } else if ps.use_animation <= 30 {
                9
            } else if ps.use_animation <= 35 {
                10
            } else if ps.use_animation <= 45 {
                11
            } else if ps.use_animation <= 55 {
                12
            } else {
                13
            };
            lines.push((crate::lang::tip(tier), [1.0, 1.0, 1.0, 1.0]));
        }
        // Knockback (`:20333-20345` + `:20749-20798`): the gloves and the
        // buff scale the item's own (prefixed) knockback, the shroomite
        // armour scales the ranged one, then the summon bonus and the
        // quiver's ×1.1.
        let mut kb = ps.knock_back;
        if let Some(p) = ctx.player {
            let mut num = 1.0f32;
            if d.class == registry::DamageClass::Melee && p.kb_glove {
                num += 1.0;
            }
            if p.kb_buff {
                num += 0.5;
            }
            if num != 1.0 {
                kb *= num;
            }
            if d.class == registry::DamageClass::Ranged && p.shroomite_stealth {
                kb *= 1.0 + (1.0 - p.stealth) * 0.5;
            }
        }
        if d.class == registry::DamageClass::Summon {
            kb += ctx.player.map_or(0.0, |p| p.minion_kb);
        }
        if ctx
            .player
            .map_or(false, |p| p.magic_quiver && d.use_ammo == crate::ammo::AMMO_ARROW)
            || d.use_ammo == crate::ammo::AMMO_STAKE
        {
            // `:20760` — `(int)(num6 * 1.1f)`: the C# TRUNCATES here (the
            // gloves/shroomite scales above stay float).
            kb = (kb * 1.1) as i32 as f32;
        }
        // `:20762-20765` — the Psycho Knife (3106) held AND hovered: the
        // stealth backstab bonus scales the shown knockback.
        if ctx.held_kind == 3106 && kind == 3106 {
            if let Some(p) = ctx.player {
                kb += kb * (1.0 - p.stealth);
            }
        }
        // The nine word tiers (`:20766-20798`); the first compare runs in
        // f64 (`(double)num6 <= 1.5`, `:20770`).
        let kb = if kb == 0.0 {
            14
        } else if kb as f64 <= 1.5 {
            15
        } else if kb <= 3.0 {
            16
        } else if kb <= 4.0 {
            17
        } else if kb <= 6.0 {
            18
        } else if kb <= 7.0 {
            19
        } else if kb <= 9.0 {
            20
        } else if kb <= 11.0 {
            21
        } else {
            22
        };
        lines.push((crate::lang::tip(kb), [1.0, 1.0, 1.0, 1.0]));
    }
    // `:20804-20815` — the fishing rows (`GameUI.PrecentFishingPower` —
    // the vanilla's own typo'd key — / `BaitRequired` / `BaitPower`).
    if d.fishing_pole > 0 {
        lines.push((
            crate::lang::game_ui_fmt("PrecentFishingPower", d.fishing_pole),
            [1.0, 1.0, 1.0, 1.0],
        ));
        lines.push((
            crate::lang::game_ui("BaitRequired"),
            [1.0, 1.0, 1.0, 1.0],
        ));
    }
    if d.bait > 0 {
        lines.push((
            crate::lang::game_ui_fmt("BaitPower", d.bait as i32),
            [1.0, 1.0, 1.0, 1.0],
        ));
    }
    // `:20816-20830` — the equipable row. The C#'s gate also admits the
    // hook projectiles (`Main.projHook[item.shoot]`), the mounts
    // (`item.mountType != -1`) and the light / vanity pets. `flag` is the
    // loadout-share arm's latch (`:20653`) — a shared slot never prints
    // the equipable row.
    let hook = d
        .projectile
        .map_or(false, |p| crate::projectile_id::is_hook_projectile(p as i32));
    if !loadout_share_flag
        && (d.head_slot > 0
            || d.body_slot > 0
            || d.leg_slot > 0
            || d.accessory
            || hook
            || d.mount_type != -1
            || (d.buff_type > 0
                && (crate::buffs::is_light_pet(d.buff_type)
                    || crate::buffs::is_vanity_pet(d.buff_type))))
    {
        // `:20818-20825` — the Discount Card 854 / Greedy Ring 3035
        // cannot be equipped or removed while shopping: tip[60].
        if (kind == 854 || kind == 3035) && ctx.shop_open {
            lines.push((crate::lang::tip(60), [1.0, 1.0, 1.0, 1.0]));
        } else {
            lines.push((crate::lang::tip(23), [1.0, 1.0, 1.0, 1.0]));
        }
    }
    // `:20828-20832` — the tileWand row: tip[52] ("Consumes ") + the
    // consumed material's name (`Item.cs`'s six tileWand writes).
    if let Some(wand) = super::tooltip_sets::tile_wand(kind) {
        lines.push((
            format!("{}{}", crate::lang::tip(52), crate::lang::item_name(wand)),
            [1.0, 1.0, 1.0, 1.0],
        ));
    }
    // `:20828-20841` — the quest / vanity rows.
    if d.quest_item {
        lines.push((crate::lang::inter(65), [1.0, 1.0, 1.0, 1.0]));
    }
    if d.vanity {
        lines.push((crate::lang::tip(24), [1.0, 1.0, 1.0, 1.0]));
    }
    // The tool rows (`:20842-20864`) — each behind its own value gate.
    if d.defense > 0 {
        lines.push((
            format!("{}{}", d.defense, crate::lang::tip(25)),
            [1.0, 1.0, 1.0, 1.0],
        ));
    }
    if d.pick > 0 {
        lines.push((
            format!("{}{}", d.pick, crate::lang::tip(26)),
            [1.0, 1.0, 1.0, 1.0],
        ));
    }
    if d.axe > 0 {
        lines.push((
            format!("{}{}", d.axe * 5, crate::lang::tip(27)),
            [1.0, 1.0, 1.0, 1.0],
        ));
    }
    if d.hammer > 0 {
        lines.push((
            format!("{}{}", d.hammer, crate::lang::tip(28)),
            [1.0, 1.0, 1.0, 1.0],
        ));
    }
    // `:20865-20875` — the range bonus (`tip[54]`, the `+N` when positive:
    // `:20868` prefixes the sign only for the positive arm).
    if d.tile_boost != 0 {
        let row = if d.tile_boost > 0 {
            format!("+{}{}", d.tile_boost, crate::lang::tip(54))
        } else {
            format!("{}{}", d.tile_boost, crate::lang::tip(54))
        };
        lines.push((row, [1.0, 1.0, 1.0, 1.0]));
    }
    // `:20876-20893` — the life / mana restore rows. The Strange Brew 3001
    // prints the RANGE row (`RestoresLifeRange`, healLife..120, `:20878-20882`).
    if d.heal_life > 0 {
        let row = if kind == 3001 {
            crate::lang::text_args(
                "CommonItemTooltip.RestoresLifeRange",
                &[d.heal_life.to_string(), "120".to_string()],
            )
        } else {
            crate::lang::common_item_tooltip_fmt("RestoresLife", d.heal_life)
        };
        lines.push((row, [1.0, 1.0, 1.0, 1.0]));
    }
    if d.heal_mana > 0 {
        lines.push((
            crate::lang::common_item_tooltip_fmt("RestoresMana", d.heal_mana),
            [1.0, 1.0, 1.0, 1.0],
        ));
    }
    // `:20891-20904` — the buff / healing / mount row.
    if d.buff_type > 0
        && !d.summon
        && !crate::buffs::is_light_pet(d.buff_type)
        && !crate::buffs::is_vanity_pet(d.buff_type)
    {
        lines.push((
            crate::lang::common_item_tooltip("Buffs"),
            [1.0, 1.0, 1.0, 1.0],
        ));
    } else if d.heal_life > 0 {
        lines.push((
            crate::lang::common_item_tooltip("Healing"),
            [1.0, 1.0, 1.0, 1.0],
        ));
    } else if d.heal_mana > 0 {
        lines.push((
            crate::lang::common_item_tooltip("ManaHealing"),
            [1.0, 1.0, 1.0, 1.0],
        ));
    } else if d.mount_type > 0 && !crate::mount::is_cart(d.mount_type) {
        lines.push((
            crate::lang::common_item_tooltip("Mount"),
            [1.0, 1.0, 1.0, 1.0],
        ));
    }
    // `:20914-20918` — the mana cost row (the prefixed mana).
    if ps.mana > 0
        && ((kind != 127 && kind != 4347 && kind != 4348 && kind != 514)
            || !ctx.player.map_or(false, |p| p.space_gun))
    {
        let cost = (ps.mana as f32 * ctx.player.map_or(1.0, |p| p.mana_cost)) as i32;
        lines.push((
            crate::lang::common_item_tooltip_fmt("UsesMana", cost),
            [1.0, 1.0, 1.0, 1.0],
        ));
    }
    // `:20919-20936` — the place / ammo / consumable rows. The place gate is
    // `createWall > 0 || createTile > -1 || type == 849`; inside it the row
    // needs `PlaceTileOnAltUse[type] || consumable` (`ItemID.cs:412`). The
    // ammo row refuses notAmmo; the consumable row refuses the
    // chlorophyte-extractinator kinds (86/1329/2337-2339).
    let places = d.wall.map_or(false, |w| w > 0) || d.tile.is_some() || kind == 849;
    if places {
        if super::tooltip_sets::place_tile_on_alt_use(kind) || d.consumable {
            lines.push((crate::lang::tip(33), [1.0, 1.0, 1.0, 1.0]));
        }
    } else if d.ammo > 0 && !crate::item::not_ammo(kind) {
        lines.push((crate::lang::tip(34), [1.0, 1.0, 1.0, 1.0]));
    } else if d.consumable && !super::tooltip_sets::chlorophyte_extractinator_consumable(kind) {
        lines.push((crate::lang::tip(35), [1.0, 1.0, 1.0, 1.0]));
    }
    // `:20937-20941` — the material row.
    if d.material {
        lines.push((crate::lang::tip(36), [1.0, 1.0, 1.0, 1.0]));
    }
    // `:20942-20956` — the wiring rows: Wireable (the trigger/mechanism tile
    // sets with the statue carve-out), Container (tile 21/467) and
    // WireTrigger (tile 441/468).
    if super::tooltip_sets::wireable(kind, d.tile) {
        lines.push((
            crate::lang::common_item_tooltip("Wireable"),
            [1.0, 1.0, 1.0, 1.0],
        ));
    }
    if matches!(d.tile, Some(21) | Some(467)) {
        lines.push((
            crate::lang::common_item_tooltip("Container"),
            [1.0, 1.0, 1.0, 1.0],
        ));
    }
    if matches!(d.tile, Some(441) | Some(468)) {
        lines.push((
            crate::lang::common_item_tooltip("WireTrigger"),
            [1.0, 1.0, 1.0, 1.0],
        ));
    }
    // `:20957-20972` — the per-item tooltip text (`Lang.GetTooltip`,
    // `Lang.cs:193-196`): the `ItemTooltip.<ItemID name>` language rows,
    // one line each. The biome keys' FIRST line becomes `Lang.tip[59]`
    // ("It has been cursed by a powerful Jungle creature") until Plantera
    // falls (`ItemID.Sets.UsesCursedByPlanteraTooltip`, `ItemID.cs:410`).
    if let Some(text) = crate::lang::item_tooltip_text(kind) {
        for (i, line) in text.split('\n').enumerate() {
            let row = if i == 0
                && super::tooltip_sets::uses_cursed_by_plantera_tooltip(kind)
                && !ctx.downed_plant_boss
            {
                crate::lang::tip(59)
            } else {
                line.to_string()
            };
            lines.push((row, [1.0, 1.0, 1.0, 1.0]));
        }
    }
    // `:20973-20977` — the anniversary Wizard Hat 238 row.
    if ctx.anniversary && kind == 238 {
        lines.push((
            crate::lang::common_item_tooltip("WizardHatDuringAnniversary"),
            [1.0, 1.0, 1.0, 1.0],
        ));
    }
    // `:20978-20982` — the for-the-worthy Crispy Honey Block 1127 row.
    if ctx.get_good_world && kind == 1127 {
        lines.push((
            crate::lang::common_item_tooltip("BurningBlock"),
            [1.0, 1.0, 1.0, 1.0],
        ));
    }
    // `:20983-20999` — the Mechdusa swap. Under Mechdusa the three mech
    // summons' LAST line becomes "Part of a set"; outside it Ocram's Razor
    // 5334 drops its last line and REPLACES the previous one (the double
    // `numLines--` walk) with "It has no effect in this world".
    if ctx.mechdusa {
        if matches!(kind, 556 | 557 | 544) {
            if let Some(last) = lines.last_mut() {
                last.0 = crate::lang::common_item_tooltip("MechSummonDuringEverything");
            }
        }
    } else if kind == 5334 {
        lines.pop();
        if let Some(last) = lines.last_mut() {
            last.0 = crate::lang::common_item_tooltip("MechdusaSummonNotDuringEverything");
        }
    }
    // `:21000-21004` — the DD2 poppers need Etherian Mana until the
    // Eternia Crystal has been defended (`Lang.misc[104]`).
    if super::tooltip_sets::dd2_needs_etherian_mana(kind) && !ctx.downed_dd2_any {
        lines.push((crate::lang::misc(104), [1.0, 1.0, 1.0, 1.0]));
    }
    // `:21005-21009` — the well-fed items' expert life-regen row
    // (`BuffID.Sets.IsWellFed`, `Lang.misc[40]`).
    if d.buff_type > 0 && super::tooltip_sets::is_well_fed(d.buff_type) && ctx.expert_mode {
        lines.push((crate::lang::misc(40), [1.0, 1.0, 1.0, 1.0]));
    }
    // `:21010-21015` — the buff duration row: under 60 seconds the
    // SecondDuration key with `Math.Round(buffTime / 60.0)` (banker's),
    // else MinuteDuration with `Math.Round((buffTime / 60) / 60.0)` — the
    // C# divides the INT quotient again.
    if d.buff_time > 0 {
        let row = if d.buff_time / 60 < 60 {
            let secs = tr_core::damage::round_bankers(d.buff_time as f64 / 60.0);
            crate::lang::common_item_tooltip_fmt("SecondDuration", secs as i32)
        } else {
            let mins = tr_core::damage::round_bankers((d.buff_time / 60) as f64 / 60.0);
            crate::lang::common_item_tooltip_fmt("MinuteDuration", mins as i32)
        };
        lines.push((row, [1.0, 1.0, 1.0, 1.0]));
    }
    // `:21016-21021` — the yoyo kinds append a blank line whose index the
    // draw pass swaps for the OneDropLogo texture (`:20410-20438`). The
    // texture is unported; the blank line keeps the row geometry.
    if super::tooltip_sets::yoyo_logo(kind) {
        lines.push((" ".to_string(), [1.0, 1.0, 1.0, 1.0]));
    }
    // `:21022-21031` — the expert / master rarity rows.
    if super::tooltip_sets::expert_only(kind) {
        lines.push((crate::lang::game_ui("Expert"), [1.0, 1.0, 1.0, 1.0]));
    }
    if ps.rare == -13 {
        lines.push((crate::lang::game_ui("Master"), [1.0, 1.0, 1.0, 1.0]));
    }
    // `:21032-21265` — the prefix comparison rows (the red/green deltas
    // against a fresh SetDefaults copy) + the accessory fixed rows.
    append_prefix_lines(d, &ps, old_kb, ctx.prefix, &mut lines);
    // `:21267-21311` — the armor-set bonus row. `wornArmor` is context 8
    // (`ItemSlot.cs:3188-3189`): the worn piece queries the local player's
    // equipped head/body/legs and picks the best set (`:21280-21306`);
    // every other context prints the single-piece grey row.
    if let Some((row, result)) = crate::armor_sets::best_worn_set(
        kind as i32,
        ctx.player.map_or(0, |p| p.equipped[0] as i32),
        ctx.player.map_or(0, |p| p.equipped[1] as i32),
        ctx.player.map_or(0, |p| p.equipped[2] as i32),
    ) {
        if ctx.context != 8 {
            lines.push((
                crate::armor_sets::tooltip_for_single_piece(row, kind as i32),
                // `new Color(130, 130, 130, 190)` (`:21275`).
                super::color::tint([130, 130, 130, 190]),
            ));
        } else {
            let color = if result.complete() {
                // `Color.LimeGreen` (`:21307`) = (50, 205, 50).
                super::color::tint([50, 205, 50, 255])
            } else {
                super::color::tint([130, 130, 130, 255])
            };
            lines.push((
                crate::armor_sets::tooltip_for_worn_armor(
                    row,
                    ctx.player.map_or(0, |p| p.equipped[0] as i32),
                    ctx.player.map_or(0, |p| p.equipped[1] as i32),
                    ctx.player.map_or(0, |p| p.equipped[2] as i32),
                    result,
                ),
                color,
            ));
        }
    }
    // `:21312-21317` — the loadout share hint (flag2 && the setting), in
    // `Main.OurFavoriteColor` (255, 231, 69) at alpha 190 (`Main.cs:66748`).
    if share_hint_flag && ctx.show_loadout_share_hint {
        lines.push((
            crate::lang::text("UI.ItemLoadoutShareHint"),
            super::color::tint([255, 231, 69, 190]),
        ));
    }
    // `:21318-21333` — the journey creative-sacrifice rows: CONSTANTS-OFF
    // (the port has no journey difficulty, `Main.LocalPlayer.difficulty
    // == 3` never holds — the `player.rs:4494` precedent).
    // `:21334-21342` — `Item.BestiaryNotes`: only the bestiary's item
    // cache ever writes the field (`UIBestiaryInfoItemLine.cs:155`); the
    // port's bestiary drop rows never build an item tooltip — cited drop.
    lines
}

/// `Main.cs:20634-20635` — the prefix-delta colours: `color` (190,120,120)
/// for the worse arm, `color2` (120,190,120) for the better.
const PREFIX_BAD: [f32; 4] = [190.0 / 255.0, 120.0 / 255.0, 120.0 / 255.0, 1.0];
const PREFIX_GOOD: [f32; 4] = [120.0 / 255.0, 190.0 / 255.0, 120.0 / 255.0, 1.0];

/// One percent-delta row (`Main.cs:21040-21137`): `Math.Round((new - base)
/// / base * 100)` (banker's), the `+` prefix only on the positive arm, the
/// colour by the delta's sign — EXCEPT the mana row, whose colour test is
/// `num10 > 0` (a mana-cost INCREASE is the worse arm, `:21093`).
fn push_percent_delta(
    lines: &mut Vec<(String, [f32; 4])>,
    delta: f64,
    tip: usize,
    invert_colour: bool,
) {
    let n = tr_core::damage::round_bankers(delta);
    let text = if n > 0 {
        format!("+{n}{}", crate::lang::tip(tip))
    } else {
        format!("{n}{}", crate::lang::tip(tip))
    };
    let bad = if invert_colour { n > 0 } else { n < 0 };
    lines.push((text, if bad { PREFIX_BAD } else { PREFIX_GOOD }));
}

/// `Main.cs:21032-21265` — the prefix block. `item2` is the fresh
/// `SetDefaults` copy (`Main.tooltipPrefixComparisonItem` caches one per
/// type; the port recomputes — same values). The percent rows compare the
/// PREFIXED stats against the base; the knockback row compares `oldKB`
/// (the prefixed knockback captured BEFORE the caller's glove/buff/
/// shroomite scales, `:20333`). The accessory prefixes 62-80 grant player
/// benefits, not item stats, so they print fixed rows instead
/// (`:21152-21265`).
fn append_prefix_lines(
    d: &crate::itemdata::ItemDef,
    ps: &crate::prefix::PrefixedStats,
    old_kb: f32,
    prefix: u16,
    lines: &mut Vec<(String, [f32; 4])>,
) {
    if prefix == 0 {
        return;
    }
    let f = |a: i32, b: i32| (a as f32 - b as f32) as f64 / b as f64 * 100.0;
    if ps.damage != d.damage {
        push_percent_delta(lines, f(ps.damage, d.damage), 39, false);
    }
    if ps.use_animation != d.use_animation as i32 {
        // `:21056` — the speed delta is NEGATED (faster = positive). The
        // C# prints the rounded double through string.Concat; whole
        // values render without a decimal point.
        let n = -tr_core::damage::round_bankers(f(ps.use_animation, d.use_animation as i32));
        let text = if n > 0 {
            format!("+{n}{}", crate::lang::tip(40))
        } else {
            format!("{n}{}", crate::lang::tip(40))
        };
        lines.push((text, if n < 0 { PREFIX_BAD } else { PREFIX_GOOD }));
    }
    if ps.crit != d.crit {
        // `:21070` — the crit delta is the raw difference, NOT a percent.
        // The C# prints the float through string.Concat (whole values
        // render without a decimal point).
        let n = ps.crit - d.crit;
        let text = if n > 0 {
            format!("+{n}{}", crate::lang::tip(41))
        } else {
            format!("{n}{}", crate::lang::tip(41))
        };
        lines.push((text, if n < 0 { PREFIX_BAD } else { PREFIX_GOOD }));
    }
    if ps.mana != d.mana {
        push_percent_delta(lines, f(ps.mana, d.mana), 42, true);
    }
    if ps.scale != d.scale {
        let delta = (ps.scale - d.scale) as f64 / d.scale as f64 * 100.0;
        push_percent_delta(lines, delta, 43, false);
    }
    if ps.shoot_speed != d.shoot_speed {
        let delta = (ps.shoot_speed - d.shoot_speed) as f64 / d.shoot_speed as f64 * 100.0;
        push_percent_delta(lines, delta, 44, false);
    }
    if old_kb != d.knock_back {
        let delta = (old_kb - d.knock_back) as f64 / d.knock_back as f64 * 100.0;
        push_percent_delta(lines, delta, 45, false);
    }
    // `:21138-21151` — the armor-penetration / tag-damage rows through the
    // `{0:+0;-#}` format keys. The port's `format_net` drops format
    // specifiers (research/33 D5), so the sign is applied here.
    if ps.armor_penetration != 0 {
        lines.push((
            signed_fmt("CommonItemTooltip.PrefixArmorPenetration", ps.armor_penetration),
            if ps.armor_penetration < 0 { PREFIX_BAD } else { PREFIX_GOOD },
        ));
    }
    if ps.tag_damage != 0 {
        lines.push((
            signed_fmt("CommonItemTooltip.PrefixTagDamage", ps.tag_damage),
            if ps.tag_damage < 0 { PREFIX_BAD } else { PREFIX_GOOD },
        ));
    }
    // `:21152-21265` — the accessory prefixes' fixed rows (all green).
    let fixed: Option<(i32, usize)> = match prefix {
        62 => Some((1, 25)),
        63 => Some((2, 25)),
        64 => Some((3, 25)),
        65 => Some((4, 25)),
        66 => Some((20, 31)),
        67 => Some((2, 5)),
        68 => Some((4, 5)),
        69 => Some((1, 39)),
        70 => Some((2, 39)),
        71 => Some((3, 39)),
        72 => Some((4, 39)),
        73 => Some((1, 46)),
        74 => Some((2, 46)),
        75 => Some((3, 46)),
        76 => Some((4, 46)),
        77 => Some((1, 47)),
        78 => Some((2, 47)),
        79 => Some((3, 47)),
        80 => Some((4, 47)),
        _ => None,
    };
    if let Some((n, tip)) = fixed {
        // `:21178` — the Warding mana row is "+20 " + tip[31] (the space
        // is in the literal, unlike every other fixed row).
        let text = if prefix == 66 {
            format!("+{n} {}", crate::lang::tip(tip))
        } else {
            format!("+{n}{}", crate::lang::tip(tip))
        };
        lines.push((text, PREFIX_GOOD));
    }
}

/// The `{0:+0;-#}` rows (`CommonItemTooltip.PrefixArmorPenetration` /
/// `PrefixTagDamage`): the C# format prints `+N` / `-N` (zero as `0`).
/// `format_net` drops the specifier (research/33 D5), so the signed
/// argument is built here and substituted plain.
fn signed_fmt(key: &str, n: i32) -> String {
    let arg = if n > 0 { format!("+{n}") } else { n.to_string() };
    crate::lang::text_args(key, &[arg])
}

/// `Main.MouseText_DrawItemTooltip` (`Main.cs:20322-20446`): the generic tooltip box
/// plus the item name in its rare colour, with placeholder zeroed stat lines for
/// weapons (spec §9.3; the real `Item.ToolTip` lines are a later data port).
/// `opaque_box` is `Main.SettingsEnabled_OpaqueBoxBehindTooltips`
/// (`Main.cs:20324`): with it off the lines draw without the `DrawInvBG` box
/// (`Main.cs:20398-20403`).
#[allow(clippy::too_many_arguments)]
pub fn draw_item_tooltip(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    mouse: Vec2,
    hover: Option<&ItemSlot>,
    screen: Vec2,
    pulse: UiPulse,
    ctx: TooltipContext<'_>,
    opaque_box: bool,
) {
    let Some(item) = hover else {
        return;
    };
    // Empty slots show nothing (`Main.HoverItem.type > 0` guard, `Main.cs:20213`).
    if item.count == 0 {
        return;
    }
    let Some(d) = registry::get(item.kind) else {
        return;
    };
    // `ItemSlot.EstimateDisplayStack` (`ItemSlot.cs:3150-3162`): the plain
    // stack — the bulk-buy / bulk-craft arms belong to the shop's and the
    // crafting panel's hovers.
    let ctx = TooltipContext {
        stack: item.count,
        ..ctx
    };
    let lines = item_tooltip_lines(item.kind, d, ctx);

    // `Main.cs:20367-20377`: the widest line and the summed line heights, `num3 = 0`.
    let lh = line_height(lookups);
    let mut w = 0.0f32;
    let mut h = 0.0f32;
    for (text, _) in &lines {
        w = w.max(hud_text_width(r, lookups, text, 1.0));
        h += lh;
    }
    let (rect, tx, ty) = item_tooltip_layout(mouse, w, h, screen, opaque_box);
    // `Main.cs:20398-20403`: the box draws only while the setting is on.
    if let Some(rect) = rect {
        draw_inv_bg(r, lookups, rect, TOOLTIP_BG);
    }
    // `Main.cs:20406-20407`: each line colour is multiplied by `mouseTextColor/255`
    // with the raw `mouseTextColor` as its alpha.
    let tint = pulse.text;
    let mut line_y = 0.0;
    for (text, color) in &lines {
        // `:20406` — `(byte)(channel * mouseTextColor/255f)` per channel
        // (truncated), alpha the RAW `mouseTextColor`. The byte truncation
        // matters for the sub-white row colours (the grey set-bonus row,
        // the red/green prefix deltas).
        let line = [
            (color[0] * 255.0 * tint) as u8 as f32 / 255.0,
            (color[1] * 255.0 * tint) as u8 as f32 / 255.0,
            (color[2] * 255.0 * tint) as u8 as f32 / 255.0,
            tint,
        ];
        // `ChatManager.DrawColorCodedStringWithShadow` (`ChatManager.cs:333-344`):
        // four black copies at `shadowDirection * spread` (`spread = 2`) under the text.
        for dir in [(-2.0f32, 0.0f32), (2.0, 0.0), (0.0, -2.0), (0.0, 2.0)] {
            hud_text(
                r,
                lookups,
                tx + dir.0,
                ty + line_y + dir.1,
                1.0,
                [0.0, 0.0, 0.0, tint],
                text,
            );
        }
        hud_text(r, lookups, tx, ty + line_y, 1.0, line, text);
        line_y += lh;
    }
}

/// `Main.MouseTextHackZoom(text)` → `Main.MouseText` (`Main.cs:20123-20131`,
/// `:20138-20210`) for a single uncoloured line: the bordered box at the pointer
/// plus the white text. This is the shape the PVP/team hover uses
/// (`Main.cs:42684`/`:42736` — `MouseTextHackZoom(Language.GetTextValue("UI.*"))`);
/// the item tooltip above is the many-line variant of the same pass.
pub fn draw_mouse_text(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    mouse: Vec2,
    text: &str,
    screen: Vec2,
    pulse: UiPulse,
) {
    draw_mouse_text_block(
        r,
        lookups,
        mouse,
        &[(text.to_string(), [1.0, 1.0, 1.0, 1.0])],
        screen,
        pulse,
    );
}

/// `Main.MouseTextInner`'s plain-text arm (`Main.cs:20218-20319`) for several
/// lines: the box measured over every line (the C# measures `cursorText` and
/// the buff tooltip separately, `:218-221`) and one four-way-shadowed line per
/// entry, stacked by `line_height`. The buff hover (`:43237-43251`) and the
/// banner list (`MouseText_DrawBuffTooltip`, `:21345-21489`) ride this.
pub fn draw_mouse_text_block(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    mouse: Vec2,
    lines: &[(String, [f32; 4])],
    screen: Vec2,
    pulse: UiPulse,
) {
    if lines.is_empty() {
        return;
    }
    let lh = line_height(lookups);
    let mut w = 0.0f32;
    for (text, _) in lines {
        w = w.max(hud_text_width(r, lookups, text, 1.0));
    }
    let h = lh * lines.len() as f32;
    let rect = tooltip_box_rect(mouse, w, h, screen);
    draw_inv_bg(r, lookups, rect, TOOLTIP_BG);
    // The same `(14, 9)` inset and `mouseTextColor` tint the tooltip uses
    // (`Main.cs:20400-20407`, `:20319`).
    let tx = rect[0] + BOX_INSET.x;
    let ty = rect[1] + BOX_INSET.y;
    let tint = pulse.text;
    let mut line_y = 0.0;
    for (text, color) in lines {
        let line = [color[0] * tint, color[1] * tint, color[2] * tint, tint];
        // `ChatManager.DrawColorCodedStringWithShadow` (`ChatManager.cs:333-344`)
        // at the `spread = 2` the interface text uses.
        for dir in [(-2.0f32, 0.0f32), (2.0, 0.0), (0.0, -2.0), (0.0, 2.0)] {
            hud_text(
                r,
                lookups,
                tx + dir.0,
                ty + line_y + dir.1,
                1.0,
                [0.0, 0.0, 0.0, tint],
                text,
            );
        }
        hud_text(r, lookups, tx, ty + line_y, 1.0, line, text);
        line_y += lh;
    }
}

/// Index of the slot drawn under `mouse`. Open inventory: the 10x5 grid at
/// `inventoryScale = 0.85` (`Main.DrawInventory`). Closed: the hotbar, whose slots are
/// 52x52 scaled by `hotbarScale` and strided by 4 px (`GUIHotbarDrawInner`,
/// `Main.cs:43613-43639`). Empty slots are not hoverable (`Main.cs:20213`).
pub fn hovered_slot(
    mouse: Vec2,
    inventory_open: bool,
    _selected: usize,
    hotbar_scales: &[f32; HOTBAR],
    inventory: &[ItemSlot; INV],
) -> Option<usize> {
    let hit = |x: f32, y: f32, s: f32| {
        mouse.x >= x && mouse.x <= x + s && mouse.y >= y && mouse.y <= y + s
    };
    if inventory_open {
        let inv_scale = 0.85f32;
        let slot = 52.0 * inv_scale;
        for j in 0..5usize {
            for i in 0..10usize {
                let idx = i + j * 10;
                let x = 20.0 + i as f32 * 56.0 * inv_scale;
                let y = 20.0 + j as f32 * 56.0 * inv_scale;
                if hit(x, y, slot) {
                    return (inventory[idx].count > 0).then_some(idx);
                }
            }
        }
    } else {
        // `GUIHotbarDrawInner` (`Main.cs:43614-43639`): the closed hotbar shares
        // the draw pass's per-slot geometry — `slot::closed_hotbar_metrics` gives
        // the row `y`, the truncated x advance and the untruncated hover size the
        // C# hit test compares against (`Main.mouseX <= num + Width * scale`).
        let mut x = 20.0;
        for i in 0..HOTBAR {
            let scale = hotbar_scales[i].clamp(0.75, 1.0);
            let (y, advance, size) = slot::closed_hotbar_metrics(scale);
            if hit(x, y, size) {
                return (inventory[i].count > 0).then_some(i);
            }
            x += advance;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    /// M7-UI-6: the tooltip row set — the equipable row (`tip[23]`), the
    /// tool range bonus (`tip[54]`, sign-prefixed only when positive,
    /// `Main.cs:20865-20875`) and the bait row (`GameUI.BaitPower`,
    /// `:20805-20816`) each appear only behind their own gate.
    #[test]
    fn m7_ui_6_the_tooltip_rows_follow_their_gates() {
        let copper_pickaxe = crate::registry::get(1).expect("the iron pickaxe row");
        let rows = item_tooltip_lines(1, copper_pickaxe, TooltipContext::default());
        assert!(rows.len() > 1, "the name plus the tool rows");
        // The pickaxe is not equipable: no `tip[23]`.
        let equip = crate::lang::tip(23);
        assert!(!rows.iter().any(|(t, _)| *t == equip), "no equipable row");
        // Its `pick` row is `Lang.tip[26]`-suffixed.
        assert!(
            rows.iter().any(|(t, _)| t.ends_with(&crate::lang::tip(26))),
            "the pick-power row"
        );
        // An accessory with a range bonus: both rows appear.
        let row = crate::itemdata::ItemDef {
            tile_boost: 3,
            accessory: true,
            ..*copper_pickaxe
        };
        let rows = item_tooltip_lines(1, &row, TooltipContext::default());
        assert!(rows.iter().any(|(t, _)| *t == equip), "the equipable row");
        let boost = crate::lang::tip(54);
        assert!(
            rows.iter().any(|(t, _)| t == &format!("+3{boost}")),
            "the positive range row carries the +"
        );
        // A negative boost drops the sign prefix (`:20872`).
        let row = crate::itemdata::ItemDef {
            tile_boost: -2,
            ..*copper_pickaxe
        };
        let rows = item_tooltip_lines(1, &row, TooltipContext::default());
        assert!(rows.iter().any(|(t, _)| t == &format!("-2{boost}")));
    }

    use super::*;

    #[test]
    fn inv_bg_min_size_is_twenty() {
        let r = inv_bg_rects([100.0, 100.0, 5.0, 8.0]);
        assert_eq!(r[0], [100.0, 100.0, 10.0, 10.0]);
        // and the nine pieces tile the clamped 20x20 rect: r[8] == [110.0, 110.0, 10.0, 10.0]
        assert_eq!(r[8], [110.0, 110.0, 10.0, 10.0]);
    }

    #[test]
    fn item_name_color_zero_is_white_neg_one_is_grey() {
        assert_eq!(item_name_color(0), [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(item_name_color(-1), [130.0 / 255.0, 130.0 / 255.0, 130.0 / 255.0, 1.0]);
    }

    #[test]
    fn item_name_color_covers_high_and_negative_rarities() {
        // `rare >= 11` is the purple expert/master colour (`Main.cs:20616`).
        assert_eq!(
            item_name_color(11),
            [180.0 / 255.0, 40.0 / 255.0, 1.0, 1.0]
        );
        assert_eq!(item_name_color(99), item_name_color(11));
        // `rare == -11` is the gold quest colour (`Main.cs:20568`).
        assert_eq!(
            item_name_color(-11),
            [1.0, 175.0 / 255.0, 0.0, 1.0]
        );
    }

    #[test]
    fn tooltip_clamps_to_the_screen() {
        let r = tooltip_box_rect(Vec2::new(1590.0, 880.0), 200.0, 40.0, Vec2::new(1600.0, 900.0));
        assert!(r[0] + r[2] <= 1600.0 && r[1] + r[3] <= 900.0);
    }

    /// `Main.SettingsEnabled_OpaqueBoxBehindTooltips` (`Main.cs:20381-20403`):
    /// with the box on, the text origin is the box's `(14, 9)` inset and the
    /// `(8, 2)` offset applies; with it off there is no box and the text sits
    /// at `mouse + 14 + toolTipDistance` with the 4 px clamp margin.
    #[test]
    fn the_opaque_box_setting_gates_the_tooltip_box() {
        let mouse = Vec2::new(100.0, 100.0);
        let screen = Vec2::new(1600.0, 900.0);
        let (rect, tx, ty) = item_tooltip_layout(mouse, 50.0, 20.0, screen, true);
        let rect = rect.expect("the box draws while the setting is on");
        assert_eq!(tx, rect[0] + BOX_INSET.x);
        assert_eq!(ty, rect[1] + BOX_INSET.y);
        // mouse + 14 + 6 + the (8, 2) box offset.
        assert_eq!(tx, 100.0 + 14.0 + 6.0 + 8.0);
        assert_eq!(ty, 100.0 + 14.0 + 6.0 + 2.0);
        let (rect, tx, ty) = item_tooltip_layout(mouse, 50.0, 20.0, screen, false);
        assert!(rect.is_none(), "no box while the setting is off");
        assert_eq!(tx, 100.0 + 14.0 + 6.0);
        assert_eq!(ty, 100.0 + 14.0 + 6.0);
        // The off-arm clamps with the 4 px margin (`num4 = 4`, `:20380`).
        let (_, tx, _) = item_tooltip_layout(Vec2::new(1590.0, 100.0), 50.0, 20.0, screen, false);
        assert_eq!(tx, 1600.0 - 50.0 - 4.0);
    }

    #[test]
    fn hovered_slot_none_when_not_over_a_cell() {
        let inv: [ItemSlot; INV] =
        std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        assert_eq!(hovered_slot(Vec2::new(2.0, 2.0), false, 0, &[1.0; HOTBAR], &inv), None);
    }

    /// The stat lines `item_tooltip_lines` builds for an item kind, in the
    /// C#'s order (`Main.cs:20631-20800`).
    fn stat_lines(kind: u16) -> Vec<String> {
        let Some(d) = registry::get(kind) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        if d.damage > 0 {
            let suffix = match d.class {
                registry::DamageClass::Melee => 2,
                registry::DamageClass::Ranged => 3,
                registry::DamageClass::Magic => 4,
                registry::DamageClass::Summon => 53,
                _ => 55,
            };
            out.push(format!("{}{}", d.damage, crate::lang::tip(suffix)));
            if matches!(
                d.class,
                registry::DamageClass::Melee
                    | registry::DamageClass::Ranged
                    | registry::DamageClass::Magic
            ) {
                out.push(format!("{}{}", d.crit, crate::lang::tip(5)));
            }
            if d.use_style != 0 {
                let tier = if d.use_animation <= 8 {
                    6
                } else if d.use_animation <= 20 {
                    7
                } else if d.use_animation <= 25 {
                    8
                } else if d.use_animation <= 30 {
                    9
                } else if d.use_animation <= 35 {
                    10
                } else if d.use_animation <= 45 {
                    11
                } else if d.use_animation <= 55 {
                    12
                } else {
                    13
                };
                out.push(crate::lang::tip(tier));
            }
            let kb = if d.knock_back == 0.0 {
            14
        } else if d.knock_back <= 1.5 {
            15
        } else if d.knock_back <= 3.0 {
            16
        } else if d.knock_back <= 4.0 {
            17
        } else if d.knock_back <= 6.0 {
            18
        } else if d.knock_back <= 7.0 {
            19
        } else if d.knock_back <= 9.0 {
            20
        } else if d.knock_back <= 11.0 {
            21
        } else {
            22
        };
        out.push(crate::lang::tip(kb));
        }
        if d.defense > 0 {
            out.push(format!("{}{}", d.defense, crate::lang::tip(25)));
        }
        if d.pick > 0 {
            out.push(format!("{}{}", d.pick, crate::lang::tip(26)));
        }
        if d.axe > 0 {
            out.push(format!("{}{}", d.axe * 5, crate::lang::tip(27)));
        }
        if d.hammer > 0 {
            out.push(format!("{}{}", d.hammer, crate::lang::tip(28)));
        }
        out
    }

    #[test]
    fn a_copper_shortsword_shows_damage_crit_use_time_and_a_knockback_word() {
        // The Copper Shortsword (item 1907? — any registered sword) — the
        // first row carries the damage and its class suffix through
        // `LegacyTooltip.2` (melee), the crit row `LegacyTooltip.5`, then the
        // use-time tier and the knockback word.
        // The Iron Shortsword (6): melee, damage 8, crit 0, useAnimation 12,
        // knockBack 4 (`itemdata.rs`).
        let lines = stat_lines(6);
        assert_eq!(lines.len(), 4, "damage + crit + use time + knockback: {lines:?}");
        assert_eq!(lines[0], format!("8{}", crate::lang::tip(2))); // melee suffix
        assert_eq!(lines[1], format!("0{}", crate::lang::tip(5))); // crit
        assert_eq!(lines[2], crate::lang::tip(7)); // useAnimation 12 -> the <=20 tier
        assert_eq!(lines[3], crate::lang::tip(17)); // knockBack 4 -> the <=4 word (`Main.cs:20769-20772`)
    }

    #[test]
    fn a_tool_shows_its_pick_axe_hammer_rows() {
        // A pickaxe (pick > 0, axe 0) shows the pick row only.
        // The Iron Pickaxe (1): pick 43, axe 0 — the pick row shows, the axe
        // row does not (`itemdata.rs`).
        let lines = stat_lines(1);
        assert!(
            lines.iter().any(|l| l.contains(&crate::lang::tip(26))),
            "{lines:?}"
        );
        assert!(!lines.iter().any(|l| l.contains(&crate::lang::tip(27))));
    }

    #[test]
    fn a_plain_block_has_no_stat_rows() {
        assert!(stat_lines(2).is_empty());
    }

    // --- S1-E-20: the tooltip's missing rows ---------------------------------

    /// The language rows the tooltip reads: the item names, the prefix names
    /// and the per-item tooltip texts.
    fn register_tooltip_language() {
        crate::lang::test_register_source(
            "Terraria.Localization.Content.en-US.Items.json",
            r#"{
                "ItemName": { "IronPickaxe": "Iron Pickaxe" },
                "Prefix": { "Legendary": "Legendary" },
                "CombineFormat": { "Prefix": "{PrefixName} {ItemName}" },
                "ItemTooltip": { "IronPickaxe": "Can dig hard rock" }
            }"#,
        );
    }

    #[test]
    fn s1_e_20_the_hover_name_carries_the_affix_and_the_stack() {
        // The reads below hit the process-global language manager; hold the
        // guard for the whole test so a sibling's culture reload cannot land
        // between the registration and the assertions.
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        // `Item.GetHoverName(item, stack)` (`Item.cs:438-446`): the affix name
        // (`Item.AffixName`, `:1194-1201`) plus the stack suffix. The guard
        // above already serialises the register→read sequence against the
        // culture-switching tests (the S1-E-17 flow tests init the language).
        register_tooltip_language();
        let pickaxe = crate::registry::get(1).expect("the iron pickaxe row");
        let rows = item_tooltip_lines(1, pickaxe, TooltipContext::default());
        assert_eq!(rows[0].0, "Iron Pickaxe", "the plain name");
        // The prefix: the language's prefix name + the item name through the
        // `CombineFormat.Prefix` format (`Lang.cs:225-239`).
        let rows = item_tooltip_lines(
            1,
            pickaxe,
            TooltipContext {
                prefix: 81,
                ..Default::default()
            },
        );
        assert_eq!(rows[0].0, "Legendary Iron Pickaxe", "the affix name");
        // The stack suffix.
        let rows = item_tooltip_lines(
            1,
            pickaxe,
            TooltipContext {
                stack: 7,
                ..Default::default()
            },
        );
        assert_eq!(rows[0].0, "Iron Pickaxe (7)", "the stack suffix");
    }

    #[test]
    fn s1_e_20_the_per_item_tooltip_text_becomes_rows() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        // `Lang.GetTooltip(item.type)` (`Lang.cs:193-196`) + the row loop
        // (`Main.cs:20958-20972`): the `ItemTooltip.<ItemID name>` language
        // rows, one line each.
        register_tooltip_language();
        let pickaxe = crate::registry::get(1).expect("the iron pickaxe row");
        let rows = item_tooltip_lines(1, pickaxe, TooltipContext::default());
        assert!(
            rows.iter().any(|(t, _)| t == "Can dig hard rock"),
            "the per-item tooltip row: {:?}",
            rows.iter().map(|(t, _)| t).collect::<Vec<_>>()
        );
    }

    #[test]
    fn s1_e_20_the_equipable_gate_admits_the_mounts_and_the_hooks() {
        // `Main.cs:20816-20830`: the equipable row's gate also admits the
        // hook projectiles (`Main.projHook[item.shoot]`), the mounts
        // (`item.mountType != -1`) and the light / vanity pets.
        let equip = crate::lang::tip(23);
        let pickaxe = crate::registry::get(1).expect("the iron pickaxe row");
        let rows = item_tooltip_lines(1, pickaxe, TooltipContext::default());
        assert!(!rows.iter().any(|(t, _)| *t == equip), "the pickaxe is not equipable");
        // A mount (the mount type 1 = the Copper Horse).
        let mount = crate::itemdata::ItemDef {
            mount_type: 1,
            ..*pickaxe
        };
        let rows = item_tooltip_lines(1, &mount, TooltipContext::default());
        assert!(rows.iter().any(|(t, _)| *t == equip), "the mount row");
        // A grappling hook (the projectile 13 is an aiStyle-7 hook).
        let hook = crate::itemdata::ItemDef {
            projectile: Some(13),
            ..*pickaxe
        };
        let rows = item_tooltip_lines(1, &hook, TooltipContext::default());
        assert!(rows.iter().any(|(t, _)| *t == equip), "the hook row");
        // A light pet (the buff 19 is a light pet).
        let pet = crate::itemdata::ItemDef {
            buff_type: 19,
            ..*pickaxe
        };
        let rows = item_tooltip_lines(1, &pet, TooltipContext::default());
        assert!(rows.iter().any(|(t, _)| *t == equip), "the light-pet row");
    }

    #[test]
    fn s1_e_20_the_player_layers_scale_the_damage_and_the_crit() {
        // `Main.cs:20672-20717`: the damage row carries the player's
        // weapon-damage multiplier and the crit row the player's crit.
        let pickaxe = crate::registry::get(1).expect("the iron pickaxe row");
        let world = tr_core::World::new("t".into(), 10, 10);
        let mut player = crate::player::Player::spawn(&world);
        player.melee_damage = 2.0;
        player.melee_crit = 20;
        let rows = item_tooltip_lines(
            1,
            pickaxe,
            TooltipContext {
                player: Some(&player),
                held_crit: 4,
                ..Default::default()
            },
        );
        // The pickaxe's own damage is 5 — doubled by the player's multiplier.
        let damage = crate::lang::tip(2);
        assert!(
            rows.iter().any(|(t, _)| t == &format!("10{damage}")),
            "the scaled damage row: {:?}",
            rows.iter().map(|(t, _)| t).collect::<Vec<_>>()
        );
        // The crit: the player's 20, minus the held item's 4, plus the
        // pickaxe's own 0.
        let crit = crate::lang::tip(5);
        assert!(
            rows.iter().any(|(t, _)| t == &format!("16{crit}")),
            "the crit row: {:?}",
            rows.iter().map(|(t, _)| t).collect::<Vec<_>>()
        );
    }

    #[test]
    fn s1_e_20_the_heal_and_mana_rows_appear() {
        // `Main.cs:20876-20910`: the life / mana restore rows and the buff
        // row.
        let pickaxe = crate::registry::get(1).expect("the iron pickaxe row");
        let potion = crate::itemdata::ItemDef {
            heal_life: 50,
            ..*pickaxe
        };
        let rows = item_tooltip_lines(1, &potion, TooltipContext::default());
        assert!(
            rows.iter().any(|(t, _)| t.contains("50")),
            "the life restore row: {:?}",
            rows.iter().map(|(t, _)| t).collect::<Vec<_>>()
        );
        let mana = crate::itemdata::ItemDef {
            heal_mana: 20,
            ..*pickaxe
        };
        let rows = item_tooltip_lines(1, &mana, TooltipContext::default());
        assert!(
            rows.iter().any(|(t, _)| t.contains("20")),
            "the mana restore row: {:?}",
            rows.iter().map(|(t, _)| t).collect::<Vec<_>>()
        );
    }

    // --- W4-e16 family 1: the stat-gate corrections ---------------------------

    /// `Main.cs:20669`: the coins 71-74 (notAmmo, useStyle 1 — the place
    /// arm admits them) show their damage row ONLY while the Coin Gun 905
    /// is carried; the acorn 27 (notAmmo, useStyle 1) keeps its gate.
    #[test]
    fn w4_e16_the_coin_damage_row_needs_the_coin_gun() {
        let coin = crate::registry::get(71).expect("the copper coin row");
        let rows = item_tooltip_lines(71, coin, TooltipContext::default());
        // The coin is RANGED class (`itemdata.rs`) — the tip[3] suffix.
        let dmg = crate::lang::tip(3);
        assert!(
            !rows.iter().any(|(t, _)| t.ends_with(&dmg)),
            "no damage row without the Coin Gun: {rows:?}"
        );
        let rows = item_tooltip_lines(
            71,
            coin,
            TooltipContext { has_coin_gun: true, ..Default::default() },
        );
        assert!(
            rows.iter().any(|(t, _)| t == &format!("25{dmg}")),
            "the coin's 25 damage with the Coin Gun: {rows:?}"
        );
    }

    /// `Main.cs:20717`: the summon staffs show no use-time row; the whips
    /// (IsAWhip[shoot]) do. The Pygmy Staff 1157 (summon, shoot 191 — not a
    /// whip) vs a whip-shaped summon row.
    #[test]
    fn w4_e16_the_use_time_row_skips_the_summon_staffs_but_not_the_whips() {
        let staff = crate::registry::get(1157).expect("the pygmy staff row");
        let rows = item_tooltip_lines(1157, staff, TooltipContext::default());
        let tier = crate::lang::tip(9); // useAnimation 28 -> the <=30 tier
        assert!(!rows.iter().any(|(t, _)| *t == tier), "no speed row: {rows:?}");
        let whip = crate::itemdata::ItemDef { projectile: Some(841), ..*staff };
        let rows = item_tooltip_lines(1157, &whip, TooltipContext::default());
        assert!(rows.iter().any(|(t, _)| *t == tier), "the whip keeps it: {rows:?}");
    }

    /// `Main.cs:20758-20760`: the quiver's ×1.1 TRUNCATES — kb 6.5 → 7.15 →
    /// (int) 7 → the <=7 "Strong knockback" word, not the <=9 tier the
    /// untruncated 7.15 would still pick; kb 8.5 → 9.35 → 9 stays <=9, but
    /// kb 9.5 → 10.45 → 10 crosses into the <=11 word.
    #[test]
    fn w4_e16_the_quiver_knockback_truncates_before_the_word_tier() {
        let bow = crate::registry::get(1).expect("a base row");
        let row = crate::itemdata::ItemDef {
            knock_back: 9.5,
            use_ammo: crate::ammo::AMMO_ARROW,
            ..*bow
        };
        let world = tr_core::World::new("t".into(), 10, 10);
        let mut player = crate::player::Player::spawn(&world);
        player.magic_quiver = true;
        let rows = item_tooltip_lines(
            1,
            &row,
            TooltipContext { player: Some(&player), ..Default::default() },
        );
        // (int)(9.5 * 1.1f) = (int)10.45 = 10 -> tip[21] (<=11), not tip[22].
        assert!(rows.iter().any(|(t, _)| *t == crate::lang::tip(21)), "{rows:?}");
        assert!(!rows.iter().any(|(t, _)| *t == crate::lang::tip(22)));
    }

    /// `Main.cs:20762-20765`: the Psycho Knife 3106 held AND hovered adds
    /// `kb * (1 - stealth)` — at full stealth (0) the shown kb doubles.
    #[test]
    fn w4_e16_the_psycho_knife_stealth_arm_doubles_the_shown_knockback() {
        let base = crate::registry::get(1).expect("a base row");
        let knife = crate::itemdata::ItemDef { knock_back: 4.0, ..*base };
        let world = tr_core::World::new("t".into(), 10, 10);
        let mut player = crate::player::Player::spawn(&world);
        player.stealth = 0.0;
        let ctx = TooltipContext {
            player: Some(&player),
            held_kind: 3106,
            ..Default::default()
        };
        let rows = item_tooltip_lines(3106, &knife, ctx);
        // 4 + 4 * (1 - 0) = 8 -> the <=9 word tip[20], not the <=4 tip[17].
        assert!(rows.iter().any(|(t, _)| *t == crate::lang::tip(20)), "{rows:?}");
        // A different held kind leaves the row at the base word.
        let rows = item_tooltip_lines(
            3106,
            &knife,
            TooltipContext { player: Some(&player), ..Default::default() },
        );
        assert!(rows.iter().any(|(t, _)| *t == crate::lang::tip(17)), "{rows:?}");
    }

    /// `Main.cs:20673-20676` + `Player.cs:18251-18273`: the rapid-attack
    /// kinds scale the shown damage by (1 + min(bonus, 0.5)); other kinds
    /// pass through.
    #[test]
    fn w4_e16_the_rapid_attack_bonus_scales_only_the_three_kinds() {
        let world = tr_core::World::new("t".into(), 10, 10);
        let mut player = crate::player::Player::spawn(&world);
        player.rapid_attack_bonus = 0.7; // clamps to 0.5
        assert_eq!(player.apply_rapid_attack_bonus(25, 2608), 37); // (int)(25*1.5)
        assert_eq!(player.apply_rapid_attack_bonus(25, 1), 25, "not in the set");
        let blade = crate::registry::get(2608).expect("the falcon blade row");
        let rows = item_tooltip_lines(
            2608,
            blade,
            TooltipContext { player: Some(&player), ..Default::default() },
        );
        let dmg = crate::lang::tip(2);
        assert!(rows.iter().any(|(t, _)| t == &format!("37{dmg}")), "{rows:?}");
    }

    /// `Item.cs:637-645`: the Revolver 2269's crit row carries the charged
    /// bonus clamped to [0, 100]; other kinds read the plain crit.
    #[test]
    fn w4_e16_the_revolver_visual_crit_clamps_the_charge() {
        let revolver = crate::registry::get(2269).expect("the revolver row");
        let world = tr_core::World::new("t".into(), 10, 10);
        let mut player = crate::player::Player::spawn(&world);
        player.ranged_crit = 4;
        player.revolver_crit_chance_bonus = 120;
        let rows = item_tooltip_lines(
            2269,
            revolver,
            TooltipContext { player: Some(&player), ..Default::default() },
        );
        // clamp(0 + 120, 0, 100) = 100; the row is 4 - 0 + 100.
        let crit = crate::lang::tip(5);
        assert!(rows.iter().any(|(t, _)| t == &format!("104{crit}")), "{rows:?}");
    }

    /// `Main.cs:20818-20825`: the Discount Card 854's equipable row swaps to
    /// tip[60] while a shop is open.
    #[test]
    fn w4_e16_the_shop_swaps_the_discount_card_equipable_row() {
        let mirror = crate::registry::get(854).expect("the discount card row");
        let rows = item_tooltip_lines(854, mirror, TooltipContext::default());
        assert!(rows.iter().any(|(t, _)| *t == crate::lang::tip(23)));
        let rows = item_tooltip_lines(
            854,
            mirror,
            TooltipContext { shop_open: true, ..Default::default() },
        );
        assert!(rows.iter().any(|(t, _)| *t == crate::lang::tip(60)), "{rows:?}");
        assert!(!rows.iter().any(|(t, _)| *t == crate::lang::tip(23)));
    }

    /// `Main.cs:20828-20832`: the tileWand row is tip[52] + the material's
    /// name — the Living Wood Wand 832 consumes Wood 9.
    #[test]
    fn w4_e16_the_tile_wand_row_names_the_consumed_material() {
        let wand = crate::registry::get(832).expect("the living wood wand row");
        let rows = item_tooltip_lines(832, wand, TooltipContext::default());
        let want = format!("{}{}", crate::lang::tip(52), crate::lang::item_name(9));
        assert!(rows.iter().any(|(t, _)| *t == want), "{rows:?}");
    }

    /// `Main.cs:20878-20882`: the Strange Brew 3001 prints the range row
    /// (healLife..120), not the plain RestoresLife.
    #[test]
    fn w4_e16_the_strange_brew_prints_the_life_range() {
        let brew = crate::registry::get(3001).expect("the strange brew row");
        let rows = item_tooltip_lines(3001, brew, TooltipContext::default());
        let want = crate::lang::text_args(
            "CommonItemTooltip.RestoresLifeRange",
            &[brew.heal_life.to_string(), "120".to_string()],
        );
        assert!(rows.iter().any(|(t, _)| *t == want), "{rows:?}");
        let plain = crate::lang::common_item_tooltip_fmt("RestoresLife", brew.heal_life);
        assert!(!rows.iter().any(|(t, _)| *t == plain));
    }

    /// `Main.cs:20919-20936`: the extractinator kinds (86, the Shadow
    /// Scale) drop the Consumable row; the alt-use kinds (4367,
    /// non-consumable) still show "Can be placed"; the notAmmo acorn 27
    /// drops the Ammo row.
    #[test]
    fn w4_e16_the_place_ammo_consumable_gates() {
        let sirens = crate::registry::get(86).expect("the shadow scale row");
        let rows = item_tooltip_lines(86, sirens, TooltipContext::default());
        assert!(
            !rows.iter().any(|(t, _)| *t == crate::lang::tip(35)),
            "the extractinator kind drops Consumable: {rows:?}"
        );
        let kite = crate::registry::get(4367).expect("the blue kite row");
        let rows = item_tooltip_lines(4367, kite, TooltipContext::default());
        assert!(
            rows.iter().any(|(t, _)| *t == crate::lang::tip(33)),
            "the alt-use kind keeps Can be placed: {rows:?}"
        );
        // The ale 353: ammo > 0 but notAmmo and no tile — the Ammo row
        // drops, the Consumable row stays (`Main.cs:20927-20935`).
        let ale = crate::registry::get(353).expect("the ale row");
        let rows = item_tooltip_lines(353, ale, TooltipContext::default());
        assert!(
            !rows.iter().any(|(t, _)| *t == crate::lang::tip(34)),
            "the notAmmo ale drops Ammo: {rows:?}"
        );
        assert!(rows.iter().any(|(t, _)| *t == crate::lang::tip(35)));
    }

    /// `Main.cs:20942-20956`: the chest 48 shows Wireable + Container; the
    /// Goblin Statue 441 (tile 105, in the statue set) shows Wireable; the
    /// non-set statue kind drops it.
    #[test]
    fn w4_e16_the_wiring_rows_follow_the_tile_sets() {
        let chest = crate::registry::get(48).expect("the chest row");
        let rows = item_tooltip_lines(48, chest, TooltipContext::default());
        assert!(rows.iter().any(|(t, _)| *t == crate::lang::common_item_tooltip("Wireable")));
        assert!(rows.iter().any(|(t, _)| *t == crate::lang::common_item_tooltip("Container")));
        let statue = crate::registry::get(441).expect("the goblin statue row");
        let rows = item_tooltip_lines(441, statue, TooltipContext::default());
        assert!(rows.iter().any(|(t, _)| *t == crate::lang::common_item_tooltip("Wireable")));
    }

    // --- W4-e16 family 2: the ToolTip tail + the world rows -------------------

    /// The language rows the tail reads (the shipped values,
    /// `Terraria.Localization.Content.en-US.Items.json` / `.Legacy.json`).
    fn register_tail_language() {
        crate::lang::test_register_source(
            "Terraria.Localization.Content.en-US.Items.json",
            r#"{
                "ItemTooltip": {
                    "JungleKey": "Unlocks a Jungle Chest in the dungeon",
                    "MechanicalWorm": "Summons The Destroyer",
                    "MechdusaSummon": "Summons ???\n'You really shouldn't'"
                },
                "CommonItemTooltip": {
                    "MechSummonDuringEverything": "'Part of a set'",
                    "MechdusaSummonNotDuringEverything": "'It has no effect in this world'",
                    "WizardHatDuringAnniversary": "Increases your max number of minions by 1",
                    "BurningBlock": "Burns you when touched without special protection",
                    "SecondDuration": "{0} second duration",
                    "MinuteDuration": "{0} minute duration"
                }
            }"#,
        );
    }

    /// `Main.cs:20961-20965`: the Jungle Key 1533's first tooltip line is
    /// tip[59] until Plantera falls, then the real text.
    #[test]
    fn w4_e16_the_biome_keys_stay_cursed_until_plantera() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        register_tail_language();
        let key = crate::registry::get(1533).expect("the jungle key row");
        let rows = item_tooltip_lines(1533, key, TooltipContext::default());
        assert!(rows.iter().any(|(t, _)| *t == crate::lang::tip(59)), "{rows:?}");
        assert!(
            !rows.iter().any(|(t, _)| t == "Unlocks a Jungle Chest in the dungeon"),
            "the real text stays hidden: {rows:?}"
        );
        let rows = item_tooltip_lines(
            1533,
            key,
            TooltipContext { downed_plant_boss: true, ..Default::default() },
        );
        assert!(
            rows.iter().any(|(t, _)| t == "Unlocks a Jungle Chest in the dungeon"),
            "{rows:?}"
        );
        assert!(!rows.iter().any(|(t, _)| *t == crate::lang::tip(59)));
    }

    /// `Main.cs:20973-20982`: the anniversary Wizard Hat 238 and the
    /// for-the-worthy Crispy Honey Block 1127 rows.
    #[test]
    fn w4_e16_the_seed_rows_gate_on_their_world_flags() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        register_tail_language();
        let hat = crate::registry::get(238).expect("the wizard hat row");
        let rows = item_tooltip_lines(238, hat, TooltipContext::default());
        let anniversary_row = crate::lang::common_item_tooltip("WizardHatDuringAnniversary");
        assert!(!rows.iter().any(|(t, _)| *t == anniversary_row));
        let rows = item_tooltip_lines(
            238,
            hat,
            TooltipContext { anniversary: true, ..Default::default() },
        );
        assert!(rows.iter().any(|(t, _)| *t == anniversary_row), "{rows:?}");
        let honey = crate::registry::get(1127).expect("the crispy honey row");
        let burning = crate::lang::common_item_tooltip("BurningBlock");
        let rows = item_tooltip_lines(1127, honey, TooltipContext::default());
        assert!(!rows.iter().any(|(t, _)| *t == burning));
        let rows = item_tooltip_lines(
            1127,
            honey,
            TooltipContext { get_good_world: true, ..Default::default() },
        );
        assert!(rows.iter().any(|(t, _)| *t == burning), "{rows:?}");
    }

    /// `Main.cs:20983-20999`: under Mechdusa the Mechanical Worm 556's last
    /// line becomes "Part of a set"; outside it Ocram's Razor 5334 drops
    /// its last line and replaces the previous one (the double `numLines--`).
    #[test]
    fn w4_e16_the_mechdusa_swap_rewrites_the_last_lines() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        register_tail_language();
        let worm = crate::registry::get(556).expect("the mechanical worm row");
        let rows = item_tooltip_lines(
            556,
            worm,
            TooltipContext { mechdusa: true, ..Default::default() },
        );
        assert_eq!(
            rows.last().unwrap().0, "'Part of a set'",
            "the last line is rewritten, not appended: {rows:?}"
        );
        let razor = crate::registry::get(5334).expect("the ocram's razor row");
        let rows = item_tooltip_lines(5334, razor, TooltipContext::default());
        assert_eq!(
            rows.last().unwrap().0, "'It has no effect in this world'",
            "{rows:?}"
        );
        assert!(
            !rows.iter().any(|(t, _)| t == "'You really shouldn't'"),
            "the dropped last line stays gone: {rows:?}"
        );
        assert!(
            !rows.iter().any(|(t, _)| t == "Summons ???"),
            "the previous line was REPLACED (numLines-- twice): {rows:?}"
        );
        // Under Mechdusa the razor keeps its own text (the else-if arm).
        let rows = item_tooltip_lines(
            5334,
            razor,
            TooltipContext { mechdusa: true, ..Default::default() },
        );
        assert!(rows.iter().any(|(t, _)| t == "'You really shouldn't'"), "{rows:?}");
    }

    /// `Main.cs:21000-21009`: the DD2 popper 3818 prints misc[104] until
    /// the crystal is defended; the Apple 4009 (WellFed 26) prints misc[40]
    /// only in expert mode.
    #[test]
    fn w4_e16_the_dd2_and_well_fed_rows() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        register_tail_language();
        let popper = crate::registry::get(3818).expect("the dd2 popper row");
        let rows = item_tooltip_lines(3818, popper, TooltipContext::default());
        assert!(rows.iter().any(|(t, _)| *t == crate::lang::misc(104)), "{rows:?}");
        let rows = item_tooltip_lines(
            3818,
            popper,
            TooltipContext { downed_dd2_any: true, ..Default::default() },
        );
        assert!(!rows.iter().any(|(t, _)| *t == crate::lang::misc(104)));
        let apple = crate::registry::get(4009).expect("the apple row");
        let rows = item_tooltip_lines(4009, apple, TooltipContext::default());
        assert!(!rows.iter().any(|(t, _)| *t == crate::lang::misc(40)));
        let rows = item_tooltip_lines(
            4009,
            apple,
            TooltipContext { expert_mode: true, ..Default::default() },
        );
        assert!(rows.iter().any(|(t, _)| *t == crate::lang::misc(40)), "{rows:?}");
    }

    /// `Main.cs:21010-21015`: the Apple's 18000 ticks → 300 s → 5 minutes;
    /// the seconds arm rounds BANKER'S (2730 → 45.5 → 46, 2670 → 44.5 → 44).
    #[test]
    fn w4_e16_the_duration_row_rounds_bankers() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        register_tail_language();
        let apple = crate::registry::get(4009).expect("the apple row");
        let rows = item_tooltip_lines(4009, apple, TooltipContext::default());
        assert!(rows.iter().any(|(t, _)| t == "5 minute duration"), "{rows:?}");
        let base = crate::registry::get(1).expect("a base row");
        let odd_half = crate::itemdata::ItemDef { buff_time: 2730, ..*base };
        let rows = item_tooltip_lines(1, &odd_half, TooltipContext::default());
        assert!(rows.iter().any(|(t, _)| t == "46 second duration"), "{rows:?}");
        let even_half = crate::itemdata::ItemDef { buff_time: 2670, ..*base };
        let rows = item_tooltip_lines(1, &even_half, TooltipContext::default());
        assert!(rows.iter().any(|(t, _)| t == "44 second duration"), "{rows:?}");
    }

    /// `Main.cs:21016-21031`: the Code 1 yoyo appends the blank logo line;
    /// the Hive Backpack 3333 prints GameUI.Expert and takes the −12 disco
    /// name colour; the Pirate Ship Mount 4792 (rare −13) prints
    /// GameUI.Master with the masterColor green channel.
    #[test]
    fn w4_e16_the_yoyo_expert_and_master_rows() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        register_tail_language();
        let yoyo = crate::registry::get(3262).expect("the code 1 row");
        let rows = item_tooltip_lines(3262, yoyo, TooltipContext::default());
        assert!(rows.iter().any(|(t, _)| t == " "), "the blank logo line: {rows:?}");
        let backpack = crate::registry::get(3333).expect("the hive backpack row");
        let ctx = TooltipContext { disco: [0, 255, 0], ..Default::default() };
        let rows = item_tooltip_lines(3333, backpack, ctx);
        assert!(rows.iter().any(|(t, _)| *t == crate::lang::game_ui("Expert")), "{rows:?}");
        assert_eq!(rows[0].1, [0.0, 1.0, 0.0, 1.0], "the disco name colour");
        let mount = crate::registry::get(4792).expect("the pirate ship mount row");
        let rows = item_tooltip_lines(4792, mount, TooltipContext::default());
        assert!(rows.iter().any(|(t, _)| *t == crate::lang::game_ui("Master")), "{rows:?}");
        // masterColor 1.0 -> (byte)(1 * 200) = 200.
        assert_eq!(rows[0].1, [1.0, 200.0 / 255.0, 0.0, 1.0]);
    }

    // --- W4-e16 family 3: the prefix comparison rows --------------------------

    /// `Main.cs:21032-21137`: the Legendary 81 Iron Broadsword 4 prints the
    /// percent deltas against the base — damage 12→14 (+17%), speed 20→18
    /// (+10%, negated), crit +5 (raw), knockback 5.5→6.325 (+15%), size
    /// 1→1.1 (+10%) — all green; the name colour rides the prefixed rare.
    #[test]
    fn w4_e16_the_legendary_prefix_prints_the_percent_deltas() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        register_tail_language();
        let sword = crate::registry::get(4).expect("the iron broadsword row");
        let rows = item_tooltip_lines(
            4,
            sword,
            TooltipContext { prefix: 81, ..Default::default() },
        );
        let texts: Vec<&str> = rows.iter().map(|(t, _)| t.as_str()).collect();
        assert!(texts.contains(&format!("+17{}", crate::lang::tip(39)).as_str()), "{texts:?}");
        assert!(texts.contains(&format!("+10{}", crate::lang::tip(40)).as_str()), "{texts:?}");
        assert!(texts.contains(&format!("+5{}", crate::lang::tip(41)).as_str()), "{texts:?}");
        assert!(texts.contains(&format!("+15{}", crate::lang::tip(45)).as_str()), "{texts:?}");
        assert!(texts.contains(&format!("+10{}", crate::lang::tip(43)).as_str()), "{texts:?}");
        // The damage ROW itself reads the prefixed 14, not the base 12.
        assert!(texts.contains(&format!("14{}", crate::lang::tip(2)).as_str()), "{texts:?}");
        // Every delta row is green (120,190,120).
        for (t, c) in &rows {
            if t.ends_with(&crate::lang::tip(39)) || t.ends_with(&crate::lang::tip(45)) {
                assert_eq!(*c, PREFIX_GOOD, "{t}");
            }
        }
    }

    /// `Main.cs:21040-21051`: the Broken 10 prefix (dmg 0.85) prints the
    /// negative damage delta in RED (190,120,120); the mana row's colour
    /// test is INVERTED (`:21093` — a cost increase is the worse arm).
    #[test]
    fn w4_e16_the_negative_deltas_are_red_and_the_mana_colour_inverts() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        register_tail_language();
        let sword = crate::registry::get(4).expect("the iron broadsword row");
        // Broken 10: dmg 0.85 -> round(12 * 0.85) = 10 -> (10-12)/12 = -17%.
        let rows = item_tooltip_lines(
            4,
            sword,
            TooltipContext { prefix: 10, ..Default::default() },
        );
        let bad = rows
            .iter()
            .find(|(t, _)| *t == format!("-17{}", crate::lang::tip(39)))
            .expect("the negative damage delta");
        assert_eq!(bad.1, PREFIX_BAD);
        // The Adept 26 prefix on the Space Gun 127 (mana 6 -> round(6*0.85)
        // = 5): the mana delta is -17% and GREEN (a decrease is good).
        let gun = crate::registry::get(127).expect("the space gun row");
        let rows = item_tooltip_lines(
            127,
            gun,
            TooltipContext { prefix: 26, ..Default::default() },
        );
        let mana = rows
            .iter()
            .find(|(t, _)| t.ends_with(&crate::lang::tip(42)))
            .expect("the mana delta");
        assert_eq!(mana.1, PREFIX_GOOD, "{mana:?}");
        // The Mystical 35 prefix (mcst 1.2): mana 6 -> round(7.2) = 7,
        // +17% — RED through the inverted test.
        let rows = item_tooltip_lines(
            127,
            gun,
            TooltipContext { prefix: 35, ..Default::default() },
        );
        let mana = rows
            .iter()
            .find(|(t, _)| *t == format!("+17{}", crate::lang::tip(42)))
            .expect("the positive mana delta");
        assert_eq!(mana.1, PREFIX_BAD);
    }

    /// `Main.cs:21138-21151`: the Ruthless 85 Sanguine Staff 4269 prints
    /// the armor-penetration (+10) and tag-damage (+3) rows through the
    /// `{0:+0;-#}` keys (the sign applied port-side, research/33 D5).
    #[test]
    fn w4_e16_the_whip_prefix_prints_the_signed_pen_and_tag_rows() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        crate::lang::test_register_source(
            "Terraria.Localization.Content.en-US.Items.json",
            r#"{
                "CommonItemTooltip": {
                    "PrefixArmorPenetration": "{0:+0;-#} armor penetration",
                    "PrefixTagDamage": "{0:+0;-#} summon tag damage"
                }
            }"#,
        );
        let staff = crate::registry::get(4269).expect("the sanguine staff row");
        let rows = item_tooltip_lines(
            4269,
            staff,
            TooltipContext { prefix: 85, ..Default::default() },
        );
        assert!(
            rows.iter().any(|(t, c)| t == "+10 armor penetration" && *c == PREFIX_GOOD),
            "{rows:?}"
        );
        assert!(
            rows.iter().any(|(t, c)| t == "+3 summon tag damage" && *c == PREFIX_GOOD),
            "{rows:?}"
        );
    }

    /// `Main.cs:21152-21265`: the accessory prefixes print fixed green
    /// rows — Hard 62 "+1 defense", Warding 66 "+20 mana" (the literal
    /// space), Quick 2 67 "+2% critical strike chance".
    #[test]
    fn w4_e16_the_accessory_prefixes_print_the_fixed_rows() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        register_tail_language();
        let watch = crate::registry::get(15).expect("the copper watch row");
        for (prefix, want) in [
            (62u16, format!("+1{}", crate::lang::tip(25))),
            (66, format!("+20 {}", crate::lang::tip(31))),
            (67, format!("+2{}", crate::lang::tip(5))),
            (80, format!("+4{}", crate::lang::tip(47))),
        ] {
            let rows = item_tooltip_lines(
                15,
                watch,
                TooltipContext { prefix, ..Default::default() },
            );
            let row = rows.iter().find(|(t, _)| *t == want).expect(&want);
            assert_eq!(row.1, PREFIX_GOOD, "prefix {prefix}: {rows:?}");
        }
    }

    /// `Main.cs:20333` + `:21124`: oldKB is captured BEFORE the glove
    /// scale — the knockback DELTA row stays prefix-only (+15%) even with
    /// the glove, while the WORD tier reads the glove-scaled knockback
    /// (5.5 × 1.15 × 2 = 12.65 → the insane word tip[22]).
    #[test]
    fn w4_e16_the_knockback_delta_reads_the_prefix_only_oldkb() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        register_tail_language();
        let sword = crate::registry::get(4).expect("the iron broadsword row");
        let world = tr_core::World::new("t".into(), 10, 10);
        let mut player = crate::player::Player::spawn(&world);
        player.kb_glove = true;
        let rows = item_tooltip_lines(
            4,
            sword,
            TooltipContext { prefix: 81, player: Some(&player), ..Default::default() },
        );
        // oldKB = 5.5 * 1.15 = 6.325; (6.325 - 5.5) / 5.5 * 100 = 15.
        assert!(
            rows.iter().any(|(t, _)| *t == format!("+15{}", crate::lang::tip(45))),
            "{rows:?}"
        );
        assert!(
            !rows.iter().any(|(t, _)| *t == format!("+130{}", crate::lang::tip(45))),
            "the glove scale must not leak into the delta: {rows:?}"
        );
        // The word tier reads the glove-scaled 12.65 → tip[22].
        assert!(rows.iter().any(|(t, _)| *t == crate::lang::tip(22)), "{rows:?}");
    }

    // --- W4-e16 family 4: favourite/social/loadout + armor-set + hint ---------

    /// `Main.cs:20655-20663`: the favorited grid item prints tip[56]/[57];
    /// the favorited Money Trough 3213 with the piggy bank open adds the
    /// "Placing into itself will be blocked" row.
    #[test]
    fn w4_e16_the_favorite_rows_and_the_bank_block() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        register_tail_language();
        let sword = crate::registry::get(4).expect("the iron broadsword row");
        let rows = item_tooltip_lines(
            4,
            sword,
            TooltipContext { favorited: true, ..Default::default() },
        );
        assert!(rows.iter().any(|(t, _)| *t == crate::lang::tip(56)), "{rows:?}");
        assert!(rows.iter().any(|(t, _)| *t == crate::lang::tip(57)));
        let trough = crate::registry::get(3213).expect("the money trough row");
        let rows = item_tooltip_lines(
            3213,
            trough,
            TooltipContext { favorited: true, open_bank: Some(0), ..Default::default() },
        );
        assert!(
            rows.iter().any(|(t, _)| *t == crate::lang::text("UI.ItemCannotBePlacedInsideItself")),
            "{rows:?}"
        );
        // The same item over a world chest (no bank) keeps the plain pair.
        let rows = item_tooltip_lines(
            3213,
            trough,
            TooltipContext { favorited: true, ..Default::default() },
        );
        assert!(
            !rows.iter().any(|(t, _)| *t == crate::lang::text("UI.ItemCannotBePlacedInsideItself"))
        );
    }

    /// `Main.cs:20638-20654` + `:20816`: a shareable context (8-12) latches
    /// `flag` — the favorited piece prints UI.ItemLoadoutShared and the
    /// equipable row drops; the unfavourited piece latches flag2 (the
    /// share hint at the tail).
    #[test]
    fn w4_e16_the_loadout_share_arm_suppresses_the_equipable_row() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        register_tail_language();
        let helmet = crate::registry::get(123).expect("the meteor helmet row");
        let rows = item_tooltip_lines(
            123,
            helmet,
            TooltipContext { context: 8, favorited: true, ..Default::default() },
        );
        assert!(
            rows.iter().any(|(t, _)| *t == crate::lang::text("UI.ItemLoadoutShared")),
            "{rows:?}"
        );
        assert!(!rows.iter().any(|(t, _)| *t == crate::lang::tip(23)));
        assert!(!rows.iter().any(|(t, _)| *t == crate::lang::tip(56)));
        // The unfavourited shareable slot: the hint row, OurFavoriteColor
        // at alpha 190 (`Main.cs:21315`, `Main.cs:66748`).
        let rows = item_tooltip_lines(
            123,
            helmet,
            TooltipContext { context: 8, ..Default::default() },
        );
        let hint = rows
            .iter()
            .find(|(t, _)| *t == crate::lang::text("UI.ItemLoadoutShareHint"))
            .expect("the share hint");
        assert_eq!(hint.1, super::color::tint([255, 231, 69, 190]));
        // The setting off drops the hint.
        let rows = item_tooltip_lines(
            123,
            helmet,
            TooltipContext { context: 8, show_loadout_share_hint: false, ..Default::default() },
        );
        assert!(
            !rows.iter().any(|(t, _)| *t == crate::lang::text("UI.ItemLoadoutShareHint"))
        );
    }

    /// `Main.cs:20664-20668`: the social vanity slot (context 9/11) prints
    /// tip[61] — except the vanity kinds and the hasVanityEffects kinds
    /// (the Compass 393 keeps its function).
    #[test]
    fn w4_e16_the_social_row_skips_the_vanity_and_info_kinds() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        register_tail_language();
        let helmet = crate::registry::get(123).expect("the meteor helmet row");
        let rows = item_tooltip_lines(
            123,
            helmet,
            TooltipContext { context: 9, ..Default::default() },
        );
        assert!(rows.iter().any(|(t, _)| *t == crate::lang::tip(61)), "{rows:?}");
        let compass = crate::registry::get(393).expect("the compass row");
        let rows = item_tooltip_lines(
            393,
            compass,
            TooltipContext { context: 9, ..Default::default() },
        );
        assert!(
            !rows.iter().any(|(t, _)| *t == crate::lang::tip(61)),
            "the info accessory keeps its function: {rows:?}"
        );
        let box_row = crate::registry::get(562).expect("the music box row");
        let rows = item_tooltip_lines(
            562,
            box_row,
            TooltipContext { context: 9, ..Default::default() },
        );
        assert!(!rows.iter().any(|(t, _)| *t == crate::lang::tip(61)));
    }

    /// `Main.cs:21267-21311`: the Meteor Helmet 123 in the grid prints the
    /// grey single-piece set row; worn (context 8) with the full set
    /// equipped it prints the LimeGreen found/needed row.
    #[test]
    fn w4_e16_the_armor_set_row_grey_unworn_green_complete() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        register_tail_language();
        let helmet = crate::registry::get(123).expect("the meteor helmet row");
        let world = tr_core::World::new("t".into(), 10, 10);
        let mut player = crate::player::Player::spawn(&world);
        player.equipped[0] = 123;
        player.equipped[1] = 124;
        player.equipped[2] = 125;
        // Unworn (grid context 0): the grey (130,130,130,190) single piece.
        let rows = item_tooltip_lines(
            123,
            helmet,
            TooltipContext { player: Some(&player), ..Default::default() },
        );
        let set_row = rows
            .iter()
            .find(|(_, c)| *c == super::color::tint([130, 130, 130, 190]))
            .expect("the grey single-piece row");
        assert!(!set_row.0.is_empty());
        // Worn (context 8) + complete: LimeGreen (50,205,50).
        let rows = item_tooltip_lines(
            123,
            helmet,
            TooltipContext { player: Some(&player), context: 8, ..Default::default() },
        );
        assert!(
            rows.iter().any(|(_, c)| *c == super::color::tint([50, 205, 50, 255])),
            "{rows:?}"
        );
        // Worn but incomplete: the grey (130,130,130,255) found/needed row.
        player.equipped[2] = 0;
        let rows = item_tooltip_lines(
            123,
            helmet,
            TooltipContext { player: Some(&player), context: 8, ..Default::default() },
        );
        assert!(
            rows.iter().any(|(_, c)| *c == super::color::tint([130, 130, 130, 255])),
            "{rows:?}"
        );
    }
}
