//! Item tooltip (`Main.MouseText`, `Utils.DrawInvBG`). Filled in by Task 4 of the
//! Whole-UI P1 plan.

use glam::Vec2;
use tr_render::SpriteRenderer;

use super::slot;
use super::UiPulse;
use super::text::{hud_text, hud_text_width};
use super::tooltip_lines2::{self, HoverFlags, TooltipWorld};
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
    /// W4-f28 — the hovered instance's `ItemSlot.MouseHover` stamps +
    /// favourite flag (`ItemSlot.cs:3183-3198`, `Item.favorited`).
    pub flags: HoverFlags,
    /// W4-f28 — the world/session gates (expert mode, the downed latches,
    /// the seed flags, npcShop, the master/disco colour animators).
    pub world: TooltipWorld,
    /// `Main.player[myPlayer].inventory[selectedItem].type` — the coin-gun
    /// damage gate's `HasItem(905)` scan (`Main.cs:20669`) and the Psycho
    /// Knife's self-hold arm (`:20762`). `None` = no inventory visible (the
    /// menu/craft hovers keep the C#'s no-player shape).
    pub inventory: Option<&'a [ItemSlot]>,
    /// The held item's KIND (the inventory scan's `selectedItem` row) —
    /// the Psycho Knife 3106 self-hold compare (`Main.cs:20762`).
    pub held_kind: u16,
    /// `Main.LocalPlayer.chest` (`Player.cs:2876`) — the favourite rows'
    /// bank-blocked notice (`Main.cs:20659`). −1 closed.
    pub open_chest: i32,
}

impl Default for TooltipContext<'_> {
    fn default() -> Self {
        Self {
            prefix: 0,
            stack: 1,
            player: None,
            held_crit: 0,
            flags: HoverFlags::default(),
            world: TooltipWorld::STOCK,
            inventory: None,
            held_kind: 0,
            open_chest: -1,
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
pub(super) fn tooltip_damage_multiplier(kind: u16) -> f32 {
    match kind {
        162 | 163 | 220 | 389 | 1259 | 4272 | 5011 | 5012 | 5526 => 2.0,
        _ => 1.0,
    }
}

/// The player's crit for the hovered item's class (`Player.meleeCrit` /
/// `rangedCrit` / `magicCrit`, `Main.cs:20701-20717`).
pub(super) fn player_crit(p: &crate::player::Player, class: registry::DamageClass) -> i32 {
    match class {
        registry::DamageClass::Melee => p.melee_crit,
        registry::DamageClass::Ranged => p.ranged_crit,
        registry::DamageClass::Magic => p.magic_crit,
        _ => 0,
    }
}

/// The item's tooltip rows — `Main.MouseText_DrawItemTooltip_GetLinesInfo`
/// (`Main.cs:20631-21342`): line 0 is the hover name in its rare colour
/// (`:20358-20360`), then the stat/tool rows each behind its own value gate,
/// through `Lang.tip[i]` (`LegacyTooltip.{i}`, `Lang.cs:497`) and the
/// `CommonItemTooltip.*` language rows. The second half (the Wireable row
/// through the armour set-bonus row) is [`tooltip_lines2::append_second_half`];
/// the favourite/social/tile-wand rows and the W4-f28 gate repairs read the
/// shared helpers there.
pub fn item_tooltip_lines(
    kind: u16,
    d: &crate::itemdata::ItemDef,
    ctx: TooltipContext<'_>,
) -> Vec<(String, [f32; 4])> {
    // The hovered instance's stats: the prefix applied over the defaults
    // (`Item.Prefix`, `Item.cs:576-586`) — the C#'s HoverItem IS the prefixed
    // instance; the unregistered kinds fall back to the bare def.
    let stats = tooltip_lines2::hover_stats(kind, ctx.prefix);
    let damage = stats.map_or(d.damage, |s| s.damage);
    let crit = stats.map_or(d.crit, |s| s.crit);
    let use_animation = stats.map_or(d.use_animation as i32, |s| s.use_animation);
    let mana = stats.map_or(d.mana, |s| s.mana);
    let knock_back = stats.map_or(d.knock_back, |s| s.knock_back);
    let rare = stats.map_or(d.rare, |s| s.rare);
    let mut lines: Vec<(String, [f32; 4])> = vec![(
        hover_name(kind, ctx.prefix, ctx.stack),
        tooltip_lines2::tooltip_name_color(rare, d.expert, &ctx.world),
    )];
    // `:20655-20663` — the favourite rows (the loadout arm is unported, so
    // the plain-favourite branch is the live one).
    for row in tooltip_lines2::favorite_rows(kind, ctx.flags.favorited, ctx.open_chest) {
        lines.push((row, [1.0, 1.0, 1.0, 1.0]));
    }
    // `:20664-20668` — the social-slot row.
    if let Some(row) = tooltip_lines2::social_row(ctx.flags, d) {
        lines.push((row, [1.0, 1.0, 1.0, 1.0]));
    }
    // `:20669` — the damage gate: the notAmmo/useStyle exclusion and the
    // coin-gun `HasItem(905)` arm.
    if tooltip_lines2::damage_row_gate(kind, d, damage, ctx.inventory) {
        // Damage (`:20672-20700`): the item's own damage × the per-item
        // tooltip multiplier × the rapid-attack scale, then the player's
        // weapon-damage multiplier (`GetWeaponDamageMultiplier`).
        let shown = tooltip_lines2::damage_row_value(kind, damage, ctx.player, d);
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
        // VISUAL crit (`Item.GetVisualCritChance` — the Revolver's charge).
        if matches!(
            d.class,
            registry::DamageClass::Melee | registry::DamageClass::Ranged | registry::DamageClass::Magic
        ) {
            let mut crit_def = *d;
            crit_def.crit = crit;
            let shown_crit =
                tooltip_lines2::crit_row_value(kind, &crit_def, ctx.player, ctx.held_crit);
            lines.push((
                format!("{}{}", shown_crit, crate::lang::tip(5)),
                [1.0, 1.0, 1.0, 1.0],
            ));
        }
        // Use time (`:20717-20747`) — the `useAnimation` tiers behind
        // `useStyle != 0` and the summon item's whip gate.
        if tooltip_lines2::use_time_row_gate(d) {
            let tier = if use_animation <= 8 {
                6
            } else if use_animation <= 20 {
                7
            } else if use_animation <= 25 {
                8
            } else if use_animation <= 30 {
                9
            } else if use_animation <= 35 {
                10
            } else if use_animation <= 45 {
                11
            } else if use_animation <= 55 {
                12
            } else {
                13
            };
            lines.push((crate::lang::tip(tier), [1.0, 1.0, 1.0, 1.0]));
        }
        // Knockback (`:20333-20350` + `:20753-20765`): the gloves/buff scale,
        // the shroomite stealth scale, the summon minionKB add, the
        // quiver/stake ×1.1 INT truncation and the Psycho Knife self-hold —
        // the same chain the prefix row's `oldKB` compares.
        let kb = tooltip_lines2::display_knockback(kind, d, knock_back, ctx.player, ctx.held_kind);
        // The nine word tiers (`:20766-20801`). The 1.5 compare is a DOUBLE
        // compare in the C# (`(double)num6 <= 1.5`).
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
    // `:20805-20812` — the fishing rows (`GameUI.PrecentFishingPower` /
    // `BaitRequired` / `GameUI.BaitPower`).
    if d.fishing_pole > 0 {
        lines.push((
            crate::lang::game_ui_fmt("RecentFishingPower", d.fishing_pole),
            [1.0, 1.0, 1.0, 1.0],
        ));
        lines.push((
            crate::lang::game_ui_fmt("BaitRequired", 0),
            [1.0, 1.0, 1.0, 1.0],
        ));
    }
    if d.bait > 0 {
        lines.push((
            crate::lang::game_ui_fmt("BaitPower", d.bait as i32),
            [1.0, 1.0, 1.0, 1.0],
        ));
    }
    // `:20816-20827` — the equipable row. The C#'s gate also admits the
    // hook projectiles (`Main.projHook[item.shoot]`), the mounts
    // (`item.mountType != -1`) and the light / vanity pets; the shopping
    // arm swaps tip[23] for tip[60] on the Discount Card / Greedy Ring.
    let hook = d
        .projectile
        .map_or(false, |p| crate::projectile_id::is_hook_projectile(p as i32));
    if d.head_slot > 0
        || d.body_slot > 0
        || d.leg_slot > 0
        || d.accessory
        || hook
        || d.mount_type != -1
        || (d.buff_type > 0
            && (crate::buffs::is_light_pet(d.buff_type)
                || crate::buffs::is_vanity_pet(d.buff_type)))
    {
        lines.push((
            tooltip_lines2::equipable_row(kind, ctx.world.npc_shop),
            [1.0, 1.0, 1.0, 1.0],
        ));
    }
    // `:20828-20832` — the tile-wand row: tip[52] + the consumed item's name.
    if let Some(row) = tooltip_lines2::tile_wand_row(d.tile_wand) {
        lines.push((row, [1.0, 1.0, 1.0, 1.0]));
    }
    // `:20833-20842` — the quest / vanity rows.
    if d.quest_item {
        lines.push((crate::lang::inter(65), [1.0, 1.0, 1.0, 1.0]));
    }
    if d.vanity {
        lines.push((crate::lang::tip(24), [1.0, 1.0, 1.0, 1.0]));
    }
    // The tool rows (`:20843-20862`) — each behind its own value gate.
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
    // `:20863-20875` — the range bonus (`tip[54]`, the `+N` when positive:
    // `:20868` prefixes the sign only for the positive arm).
    if d.tile_boost != 0 {
        let row = if d.tile_boost > 0 {
            format!("+{}{}", d.tile_boost, crate::lang::tip(54))
        } else {
            format!("{}{}", d.tile_boost, crate::lang::tip(54))
        };
        lines.push((row, [1.0, 1.0, 1.0, 1.0]));
    }
    // `:20876-20893` — the life / mana restore rows (the Strange Brew
    // 3001 prints the RANGE form).
    if d.heal_life > 0 {
        lines.push((
            tooltip_lines2::restores_life_row(kind, d.heal_life),
            [1.0, 1.0, 1.0, 1.0],
        ));
    }
    if d.heal_mana > 0 {
        lines.push((
            crate::lang::common_item_tooltip_fmt("RestoresMana", d.heal_mana),
            [1.0, 1.0, 1.0, 1.0],
        ));
    }
    // `:20894-20913` — the buff / healing / mount row.
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
    // `:20914-20918` — the mana cost row.
    if mana > 0
        && ((kind != 127 && kind != 4347 && kind != 4348 && kind != 514)
            || !ctx.player.map_or(false, |p| p.space_gun))
    {
        let cost = (mana as f32 * ctx.player.map_or(1.0, |p| p.mana_cost)) as i32;
        lines.push((
            crate::lang::common_item_tooltip_fmt("UsesMana", cost),
            [1.0, 1.0, 1.0, 1.0],
        ));
    }
    // `:20919-20936` — the place / ammo / consumable ladder: the placeable
    // arm admits `PlaceTileOnAltUse`, the ammo row excludes `notAmmo`, the
    // consumable row excludes the extractinator-only family.
    if tooltip_lines2::place_row_active(d, kind) {
        lines.push((crate::lang::tip(33), [1.0, 1.0, 1.0, 1.0]));
    } else if tooltip_lines2::ammo_row_active(d, kind) {
        lines.push((crate::lang::tip(34), [1.0, 1.0, 1.0, 1.0]));
    } else if tooltip_lines2::consumable_row_active(d, kind) {
        lines.push((crate::lang::tip(35), [1.0, 1.0, 1.0, 1.0]));
    }
    // `:20937-20941` — the material row.
    if d.material {
        lines.push((crate::lang::tip(36), [1.0, 1.0, 1.0, 1.0]));
    }
    // `:20942-21342` — the second half: the Wireable/Container/WireTrigger
    // rows, the per-item text (with the cursed-by-Plantera swap), the seed
    // rows, the DD2/well-fed/duration rows, the yoyo placeholder, the
    // Expert/Master rows, the prefix comparison rows and the set-bonus row.
    tooltip_lines2::append_second_half(
        kind,
        d,
        ctx.prefix,
        ctx.flags,
        &ctx.world,
        ctx.player,
        &mut lines,
    );
    lines
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
    // with the raw `mouseTextColor` as its alpha. The byte multiply TRUNCATES
    // per channel (`(byte)(c * num2)`) — the per-line alpha (the set-bonus
    // row's 190, `:21275`) does not survive; every line draws at the tint.
    let tint = pulse.text;
    let mut line_y = 0.0;
    for (text, color) in &lines {
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
}
