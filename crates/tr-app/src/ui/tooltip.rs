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
    /// `Main.player[myPlayer].HasItem(905)` (`Player.cs:55894-55904`) — the
    /// coin-gun damage gate's 58-slot scan (`Main.cs:20669`). The draw site
    /// computes it over grid + `equip.coins` + `equip.ammo` (`inv::INV_SLOTS`
    /// = 58); a `inventory` slice alone is the 50-slot grid and would miss
    /// the coin/ammo rows 50-57 (research/128 §4.4).
    pub has_coin_gun: bool,
    /// The hovered player's carry grid (`&inventory[..]`, the 50-slot
    /// grid). Kept on the context for the shop/craft hover seams
    /// (research/128 §4.3); the coin-gun scan rides `has_coin_gun` above —
    /// this slice alone cannot see the coin/ammo rows 50-57.
    pub inventory: Option<&'a [ItemSlot]>,
    /// The held item's KIND (the inventory scan's `selectedItem` row) —
    /// the Psycho Knife 3106 self-hold compare (`Main.cs:20762`).
    pub held_kind: u16,
    /// `Main.LocalPlayer.chest` (`Player.cs:2876`) — the favourite rows'
    /// bank-blocked notice (`Main.cs:20659`). −1 closed.
    pub open_chest: i32,
    /// `Player.Settings.ShowLoadoutShareHint` (`Player.cs:372`, static
    /// ctor true `:412`): the share-hint row's gate (`Main.cs:21312`).
    pub show_loadout_share_hint: bool,
    /// W4-f28 — `NewCraftingUI._missingRequirementsTooltipText`
    /// (`NewCraftingUI.cs:526`): the hovered UNAVAILABLE recipe's reason,
    /// appended after the shop lines while the crafting panel is visible
    /// (`Main.cs:20363-20366`). The port's grid lists craftable rows only,
    /// so the draw sites leave this `None` until the persistent recipe-list
    /// port lands (docs/ANALYSIS.md).
    pub crafting_missing_reason: Option<String>,
    /// W4-f28 — `MouseTextCache.diff` (`Main.cs:20172`): the mediumcore (1)
    /// / hardcore (2) character-colour override on the name row
    /// (`GetItemNameColor`, `:20620-20627`). The item tooltip's own call
    /// passes 0 (`DrawInterface_33_MouseText`, `:45419`); the stealthed-
    /// remote-player hover passes `player.difficulty` (`:42934`, unported).
    pub diff: u8,
    /// W4-f28 — `Main.LocalPlayer.difficulty` (`Main.cs:21318`): 3 is the
    /// Journey character, the research rows' gate. The c24 port's
    /// `App.active_player_difficulty` feeds it; 0 (classic) until then.
    pub difficulty: u8,
    /// W4-f28 — the session sacrifice tracker (`Main.LocalPlayerCreativeTracker.
    /// ItemSacrifices`, `Main.cs:21318`): the c24 App field feeds it;
    /// `None` until that port lands (the research rows stay off).
    pub research: Option<&'a crate::research::ResearchTracker>,
    /// W4-f28 — the shop price row's hover inputs
    /// (`Main.MouseText_DrawItemTooltip_AddShopLines`, `Main.cs:20448-20555`):
    /// `Some` while a shop is open (`Main.npcShop > 0`) — the row appends
    /// after `GetLinesInfo` on EVERY hover, the shop grid's context-15
    /// stamp (`buy = true`) included. `None` = no shop, no row.
    pub shop_lines: Option<crate::shop::ShopLineContext>,
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
            has_coin_gun: false,
            inventory: None,
            held_kind: 0,
            open_chest: -1,
            show_loadout_share_hint: true,
            crafting_missing_reason: None,
            diff: 0,
            difficulty: 0,
            research: None,
            shop_lines: None,
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
    item_tooltip_rows(kind, d, ctx).0
}

/// [`item_tooltip_lines`] plus the yoyo-logo row index
/// (`Main.MouseText_DrawItemTooltip_GetLinesInfo`'s `ref yoyoLogo`,
/// `Main.cs:20631`/`:21019`): the draw pass paints the OneDropLogo over
/// that row instead of its " " placeholder text (`:20408-20439`).
pub fn item_tooltip_rows(
    kind: u16,
    d: &crate::itemdata::ItemDef,
    ctx: TooltipContext<'_>,
) -> (Vec<(String, [f32; 4])>, Option<usize>) {
    // The hovered instance's stats: the prefix applied over the CALLER's
    // row (`Item.Prefix`, `Item.cs:576-586`, through `prefixed_stats_of` —
    // the e16 caller seam, research/128 §3): the C#'s HoverItem IS the
    // prefixed instance, so a witness-overridden def reaches the math.
    let stats = tooltip_lines2::hover_stats(d, ctx.prefix);
    let damage = stats.damage;
    let crit = stats.crit;
    let use_animation = stats.use_animation;
    let mana = stats.mana;
    // `Main.cs:20333` — `oldKB` captured BEFORE the glove/buff/shroomite
    // scales mutate `hoverItem.knockBack` (`:20334-20350`): the prefix
    // comparison (`:21124`) reads the PREFIX-only knockback while the word
    // tier reads the scaled one (research/128 §5-e16.3).
    let old_kb = stats.knock_back;
    let knock_back = stats.knock_back;
    let rare = stats.rare;
    let mut lines: Vec<(String, [f32; 4])> = vec![(
        hover_name(kind, ctx.prefix, ctx.stack),
        tooltip_lines2::tooltip_name_color(rare, d.expert, ctx.diff, &ctx.world),
    )];
    // `:20636-20663` — the favourite / loadout-share head (the e16 arm,
    // research/128 §5-e16.4). `flag` latches the loadout-share arm (it
    // suppresses the equipable row, `:20816`); `flag2` latches the
    // unfavourited shareable slot (the share-hint row at the tail,
    // `:21312`). `GetSharedLoadoutItem` (`ItemSlot.cs:2839-2846` →
    // `GetEffectiveArmor`/`GetEffectiveDye`) rides the g55 loadout port —
    // `ctx.flags.shared_from_loadout` carries its `sharedFromLoadout`
    // output (the W4-f28 contract, docs/ANALYSIS.md); until that port
    // lands the field is `None` and the favourited arm prints
    // `UI.ItemLoadoutShared` directly.
    let mut loadout_share_flag = false;
    let mut share_hint_flag = false;
    if ctx.flags.context >= 0 && tooltip_lines2::can_loadout_share_at(ctx.flags.context) {
        if let Some(from) = ctx.flags.shared_from_loadout {
            // `:20641-20644` — the SharedFrom arm: the 1-based loadout index.
            lines.push((
                crate::lang::text_args(
                    "UI.ItemLoadoutSharedFrom",
                    &[(from + 1).to_string()],
                ),
                [1.0, 1.0, 1.0, 1.0],
            ));
        } else if ctx.flags.favorited {
            lines.push((
                crate::lang::text("UI.ItemLoadoutShared"),
                [1.0, 1.0, 1.0, 1.0],
            ));
        } else {
            share_hint_flag = true;
        }
        loadout_share_flag = true;
    } else {
        // `:20655-20663` — the plain favourite rows.
        for row in tooltip_lines2::favorite_rows(kind, ctx.flags.favorited, ctx.open_chest) {
            lines.push((row, [1.0, 1.0, 1.0, 1.0]));
        }
    }
    // `:20664-20668` — the social-slot row.
    if let Some(row) = tooltip_lines2::social_row(ctx.flags, d) {
        lines.push((row, [1.0, 1.0, 1.0, 1.0]));
    }
    // `:20669` — the damage gate: the notAmmo/useStyle exclusion and the
    // coin-gun `HasItem(905)` arm (the 58-slot scan rides ctx.has_coin_gun).
    if tooltip_lines2::damage_row_gate(kind, d, damage, ctx.has_coin_gun) {
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
    // `:20804-20815` — the fishing rows. `PrecentFishingPower` is the
    // vanilla's OWN typo'd key (`:20806`, the shipped en-US row is
    // "{0}% fishing power") — the base-era `RecentFishingPower` spelling
    // resolved nothing and the row rendered blank (research/128 §5-e16.1).
    // `BaitRequired` takes NO argument (`:20808`).
    if d.fishing_pole > 0 {
        lines.push((
            crate::lang::game_ui_fmt("PrecentFishingPower", d.fishing_pole),
            [1.0, 1.0, 1.0, 1.0],
        ));
        lines.push((crate::lang::game_ui("BaitRequired"), [1.0, 1.0, 1.0, 1.0]));
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
    // `!flag` (`:20816`) — a loadout-shared slot never prints the row.
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
    let yoyo_logo = tooltip_lines2::append_second_half(
        kind,
        d,
        ctx.prefix,
        old_kb,
        share_hint_flag,
        ctx.show_loadout_share_hint,
        ctx.flags,
        &ctx.world,
        ctx.player,
        ctx.difficulty,
        ctx.research,
        &mut lines,
    );
    // `Main.cs:20361-20362` — `AddShopLines` runs right after GetLinesInfo:
    // the price row on every hover while a shop is open (the context-15
    // shop-grid stamp prints the BUY row, every other hover the SELL row).
    // The hover's CURRENT value is the prefixed instance's (`Item.Prefix`'s
    // ×value², `Item.cs:614-615`) — the caller's context carries the
    // shopCustomPrice override and the session inputs.
    if let Some(mut shop_ctx) = ctx.shop_lines {
        shop_ctx.instance_value = stats.value;
        if let Some(row) =
            crate::shop::add_shop_lines(kind, ctx.stack, ctx.world.npc_shop, shop_ctx)
        {
            lines.push((row.text, row.color));
        }
    }
    // `Main.cs:20363-20366` — THEN `NewCraftingUI.AddTooltipLines`: the
    // crafting row is the LAST row.
    crate::craft::add_crafting_tooltip_lines(
        ctx.crafting_missing_reason.as_deref(),
        kind == 0,
        &mut lines,
    );
    (lines, yoyo_logo)
}

/// `Main.cs:20634-20635` — the prefix-delta colours: `color` (190,120,120)
/// for the worse arm, `color2` (120,190,120) for the better.
const PREFIX_BAD: [f32; 4] = [190.0 / 255.0, 120.0 / 255.0, 120.0 / 255.0, 1.0];
const PREFIX_GOOD: [f32; 4] = [120.0 / 255.0, 190.0 / 255.0, 120.0 / 255.0, 1.0];

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
    let (lines, yoyo_logo) = item_tooltip_rows(item.kind, d, ctx);

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
    for (i, (text, color)) in lines.iter().enumerate() {
        // `Main.cs:20408-20439` — the yoyo-logo row draws the OneDropLogo
        // texture INSTEAD of its " " placeholder text: four black copies
        // at the ±1 px offsets (m 0-3), then the tinted copy (m 4,
        // `(num10, num10, num10, num10)` with `num10 = mouseTextColor`).
        // The row's HEIGHT still advances by the placeholder's measured
        // line (`:20444` runs for every row, the logo included).
        if Some(i) == yoyo_logo {
            if let Some(sheet) = lookups.one_drop_logo.as_ref() {
                if let Some(uv) = lookups.hud_uv(Some(sheet)) {
                    let size = [sheet.width as f32, sheet.height as f32];
                    for (m, (dx, dy)) in [
                        (-1.0f32, 0.0f32),
                        (1.0, 0.0),
                        (0.0, -1.0),
                        (0.0, 1.0),
                        (0.0, 0.0),
                    ]
                    .into_iter()
                    .enumerate()
                    {
                        let c = if m == 4 { [tint; 4] } else { [0.0, 0.0, 0.0, 1.0] };
                        r.push_ui_sprite([tx + dx, ty + line_y + dy], size, uv, c);
                    }
                }
            }
            line_y += lh;
            continue;
        }
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

/// The sign hover tooltip (`Main.DrawMouseOver`'s sign arm,
/// `Main.cs:42940-42986`): the hovered sign's text preview. The gate
/// (`:42940`): no earlier mouse text, `signHover != -1`, the record live,
/// `!mouseInterface` and the text not all-whitespace. The text wraps at
/// 460 px / 10 lines (`Utils.WordwrapString`, `:42942` — the same wrap the
/// dialog's `TextDisplayCache` uses, [`super::npc_chat::panel_lines`]);
/// the box width is the widest line capped at 460 (`:42946-42958`). The
/// origin is `mouse + 16` (`:42960`), `+ (8, 2)` with the opaque-box
/// setting (`:42961-42964`), clamped to `screenHeight − 30·lines` /
/// `screenWidth − width` (`:42965-42972`). The text colour is
/// `Color(mouseTextColor ×4)` (`:42973`); with the opaque box it lerps to
/// White (`:42976` — `Lerp(c, White, 1)` IS White) over a
/// `DrawInvBG(23, 25, 81)·0.925·0.85` box at `(x−10, y−5, w+20,
/// 30·lines+7)` (`:42977-42979`). Each line draws through
/// `DrawBorderStringFourWay` at `y + l·30` (`:42981-42984`). W4-f22.
pub fn draw_sign_hover_tooltip(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    mouse: Vec2,
    text: &str,
    screen: Vec2,
    pulse: UiPulse,
    opaque_box: bool,
) {
    if text.trim().is_empty() {
        return; // `!string.IsNullOrWhiteSpace` (`:42940`)
    }
    let lines = super::npc_chat::panel_lines(lookups, text);
    if lines.is_empty() {
        return;
    }
    let line_amount = lines.len() as f32;
    let mut w = 0.0f32;
    for line in &lines {
        w = w.max(hud_text_width(r, lookups, line, 1.0));
    }
    if w > 460.0 {
        w = 460.0; // `:42955-42958`
    }
    let mut pos = mouse + Vec2::splat(16.0); // `:42960`
    if opaque_box {
        pos += Vec2::new(8.0, 2.0); // `:42963`
    }
    if pos.y > screen.y - 30.0 * line_amount {
        pos.y = screen.y - 30.0 * line_amount; // `:42965-42968`
    }
    if pos.x > screen.x - w {
        pos.x = screen.x - w; // `:42969-42972`
    }
    let m = pulse.text;
    let mut color = [m; 4]; // `:42973`
    if opaque_box {
        color = [1.0; 4]; // `:42976` — `Color.Lerp(color, White, 1f)`
        // `:42977-42979`: `num3 = 10`, `num4 = 5`, the box
        // `(x−10, y−5, w+20, 30·lines + 5 + 2)`.
        let rect = [
            pos.x - 10.0,
            pos.y - 5.0,
            w + 20.0,
            30.0 * line_amount + 5.0 + 2.0,
        ];
        let bg = [
            23.0 / 255.0 * 0.925 * 0.85,
            25.0 / 255.0 * 0.925 * 0.85,
            81.0 / 255.0 * 0.925 * 0.85,
            0.925 * 0.85,
        ];
        draw_inv_bg(r, lookups, rect, bg);
    }
    for (l, line) in lines.iter().enumerate() {
        // `Utils.DrawBorderStringFourWay` (`Utils.cs:2464-2496`): the four
        // ±2 px side passes in Black, then the colour.
        super::text::draw_border_string(
            r,
            lookups,
            super::text::BorderFont::Ui,
            pos.x,
            pos.y + l as f32 * 30.0,
            1.0,
            &[[-2.0, 0.0], [2.0, 0.0], [0.0, -2.0], [0.0, 2.0]],
            color,
            line,
        );
    }
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

    /// W4-c24c — the Journey sacrifice row (`Main.cs:21318-21333`): the
    /// whitelisted context + difficulty 3 + a catalog-capped partial kind
    /// appends "N more needed" in the Journey colour; the gates refuse it.
    #[test]
    fn w4_c24c_the_journey_sacrifice_row_follows_its_gates() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        crate::lang::test_register_source(
            "Terraria.Localization.Content.en-US.Items.json",
            r#"{
                "ItemName": { "Torch": "Torch" },
                "CommonItemTooltip": { "CreativeSacrificeNeeded": "Research {0} more to unlock duplication" }
            }"#,
        );
        let torch = crate::registry::get(8).expect("the torch row");
        let mut t = crate::research::ResearchTracker::default();
        t.register_sacrifice(8, 90); // Torch cap 100 — 10 still needed
        let journey = TooltipContext {
            flags: tooltip_lines2::HoverFlags { context: 0, ..Default::default() },
            difficulty: 3,
            research: Some(&t),
            ..Default::default()
        };
        let rows = item_tooltip_lines(8, torch, journey);
        let last = rows.last().expect("the name plus the sacrifice row");
        assert_eq!(last.0, "Research 10 more to unlock duplication");
        let c = crate::ui::uistate::CREATIVE_MODE_COLOR;
        assert_eq!(
            last.1,
            [c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0, 1.0],
            "Colors.JourneyMode = Lerp(HotPink, White, 0.1)"
        );
        // A non-Journey difficulty refuses the row (`:21318`'s == 3).
        let rows = item_tooltip_lines(
            8,
            torch,
            TooltipContext { difficulty: 0, ..journey },
        );
        assert!(!rows.iter().any(|(text, _)| text.contains("more to unlock")));
        // A non-whitelisted context refuses (context 8 = armour, `:21318`).
        let rows = item_tooltip_lines(
            8,
            torch,
            TooltipContext { flags: tooltip_lines2::HoverFlags { context: 8, ..Default::default() }, ..journey },
        );
        assert!(!rows.iter().any(|(text, _)| text.contains("more to unlock")));
        // The duplication grid's partial cell (34) IS whitelisted.
        let rows = item_tooltip_lines(
            8,
            torch,
            TooltipContext { flags: tooltip_lines2::HoverFlags { context: 34, ..Default::default() }, ..journey },
        );
        assert!(rows.iter().any(|(text, _)| text.contains("more to unlock")));
        // A fully researched kind prints nothing (`have < cap` gate, `:21321`).
        // The context is rebuilt — the old one's `&t` borrow ends here.
        t.register_sacrifice(8, 10);
        let full = TooltipContext {
            flags: tooltip_lines2::HoverFlags { context: 0, ..Default::default() },
            difficulty: 3,
            research: Some(&t),
            ..Default::default()
        };
        let rows = item_tooltip_lines(8, torch, full);
        assert!(!rows.iter().any(|(text, _)| text.contains("more to unlock")));
        // A capless kind prints nothing (`TryGetSacrificeNumbers` false).
        let rows = item_tooltip_lines(6196, torch, full);
        assert!(!rows.iter().any(|(text, _)| text.contains("more to unlock")));
        // No tracker: no row.
        let rows = item_tooltip_lines(
            8,
            torch,
            TooltipContext { research: None, ..full },
        );
        assert!(!rows.iter().any(|(text, _)| text.contains("more to unlock")));
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
        assert_eq!(player.apply_rapid_attack_bonus(25, 2608, false), 37); // (int)(25*1.5)
        assert_eq!(player.apply_rapid_attack_bonus(25, 1, false), 25, "not in the set");
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
            TooltipContext { world: tooltip_lines2::TooltipWorld { npc_shop: true, ..Default::default() }, ..Default::default() },
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
            TooltipContext { world: tooltip_lines2::TooltipWorld { downed_plant_boss: true, ..Default::default() }, ..Default::default() },
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
            TooltipContext { world: tooltip_lines2::TooltipWorld { tenth_anniversary: true, ..Default::default() }, ..Default::default() },
        );
        assert!(rows.iter().any(|(t, _)| *t == anniversary_row), "{rows:?}");
        let honey = crate::registry::get(1127).expect("the crispy honey row");
        let burning = crate::lang::common_item_tooltip("BurningBlock");
        let rows = item_tooltip_lines(1127, honey, TooltipContext::default());
        assert!(!rows.iter().any(|(t, _)| *t == burning));
        let rows = item_tooltip_lines(
            1127,
            honey,
            TooltipContext { world: tooltip_lines2::TooltipWorld { get_good_world: true, ..Default::default() }, ..Default::default() },
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
            TooltipContext { world: tooltip_lines2::TooltipWorld { mechdusa: true, ..Default::default() }, ..Default::default() },
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
            TooltipContext { world: tooltip_lines2::TooltipWorld { mechdusa: true, ..Default::default() }, ..Default::default() },
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
            TooltipContext { world: tooltip_lines2::TooltipWorld { downed_dd2_any: true, ..Default::default() }, ..Default::default() },
        );
        assert!(!rows.iter().any(|(t, _)| *t == crate::lang::misc(104)));
        let apple = crate::registry::get(4009).expect("the apple row");
        let rows = item_tooltip_lines(4009, apple, TooltipContext::default());
        assert!(!rows.iter().any(|(t, _)| *t == crate::lang::misc(40)));
        let rows = item_tooltip_lines(
            4009,
            apple,
            TooltipContext { world: tooltip_lines2::TooltipWorld { expert_mode: true, ..Default::default() }, ..Default::default() },
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
        let ctx = TooltipContext { world: tooltip_lines2::TooltipWorld { disco: [0, 255, 0], ..Default::default() }, ..Default::default() };
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
            TooltipContext { flags: tooltip_lines2::HoverFlags { favorited: true, ..Default::default() }, ..Default::default() },
        );
        assert!(rows.iter().any(|(t, _)| *t == crate::lang::tip(56)), "{rows:?}");
        assert!(rows.iter().any(|(t, _)| *t == crate::lang::tip(57)));
        let trough = crate::registry::get(3213).expect("the money trough row");
        let rows = item_tooltip_lines(
            3213,
            trough,
            TooltipContext { flags: tooltip_lines2::HoverFlags { favorited: true, ..Default::default() }, open_chest: -2, ..Default::default() },
        );
        assert!(
            rows.iter().any(|(t, _)| *t == crate::lang::text("UI.ItemCannotBePlacedInsideItself")),
            "{rows:?}"
        );
        // The same item over a world chest (no bank) keeps the plain pair.
        let rows = item_tooltip_lines(
            3213,
            trough,
            TooltipContext { flags: tooltip_lines2::HoverFlags { favorited: true, ..Default::default() }, ..Default::default() },
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
            TooltipContext { flags: tooltip_lines2::HoverFlags { context: 8, favorited: true, worn_armor: true, ..Default::default() }, ..Default::default() },
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
            TooltipContext { flags: tooltip_lines2::HoverFlags { context: 8, worn_armor: true, ..Default::default() }, ..Default::default() },
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
            TooltipContext { flags: tooltip_lines2::HoverFlags { context: 8, worn_armor: true, ..Default::default() }, show_loadout_share_hint: false, ..Default::default() },
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
            TooltipContext { flags: tooltip_lines2::HoverFlags { context: 9, social: true, ..Default::default() }, ..Default::default() },
        );
        assert!(rows.iter().any(|(t, _)| *t == crate::lang::tip(61)), "{rows:?}");
        let compass = crate::registry::get(393).expect("the compass row");
        let rows = item_tooltip_lines(
            393,
            compass,
            TooltipContext { flags: tooltip_lines2::HoverFlags { context: 9, social: true, ..Default::default() }, ..Default::default() },
        );
        assert!(
            !rows.iter().any(|(t, _)| *t == crate::lang::tip(61)),
            "the info accessory keeps its function: {rows:?}"
        );
        let box_row = crate::registry::get(562).expect("the music box row");
        let rows = item_tooltip_lines(
            562,
            box_row,
            TooltipContext { flags: tooltip_lines2::HoverFlags { context: 9, social: true, ..Default::default() }, ..Default::default() },
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
            TooltipContext { player: Some(&player), flags: tooltip_lines2::HoverFlags { context: 8, worn_armor: true, ..Default::default() }, ..Default::default() },
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
            TooltipContext { player: Some(&player), flags: tooltip_lines2::HoverFlags { context: 8, worn_armor: true, ..Default::default() }, ..Default::default() },
        );
        assert!(
            rows.iter().any(|(_, c)| *c == super::color::tint([130, 130, 130, 255])),
            "{rows:?}"
        );
    }
}
