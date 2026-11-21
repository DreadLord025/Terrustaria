//! The item tooltip's SECOND half — `Main.MouseText_DrawItemTooltip_GetLinesInfo`
//! from the material row's tail through the bestiary notes
//! (`Main.cs:20937-21342`), the gate repairs the first half's rows need
//! (`:20669`/`:20717`/`:20753-20765`/`:20816-20826`/`:20876-20887`/
//! `:20919-20935`) and the name-colour override (`:20328-20332` +
//! `GetItemNameColor` `:20557-20629`). The first half (the hover name through
//! the material row, S1-E-20) stays in `tooltip.rs`; this module owns every
//! line-kind after it plus the shared context the rows read.
//!
//! Deferred, cited (ANALYSIS "The tooltip lines' second half (W4-f28)"):
//! the loadout SharedFrom row (`:20640-20643` — `GetSharedLoadoutItem`
//! unported; the rest of the loadout head IS live, the e16 salvage per
//! research/128 §5-e16.4), the Journey research rows (`:21318-21333` — no
//! sacrifice tracker
//! wired into the session), the bestiary-notes rows (`:21334-21342` —
//! `Item.BestiaryNotes` is only written by `UIBestiaryInfoItemLine.cs:155`,
//! whose hover line is unported), the shop lines
//! (`MouseText_DrawItemTooltip_AddShopLines`, `:20448-20555` — the recorded
//! S1-P-i44 chrome follow-up), `NewCraftingUI.AddTooltipLines` (`:20363-20366`)
//! and the yoyo-logo draw arm (`:20408-20439` — the row's " " placeholder is
//! emitted; the OneDropLogo texture pass is render work).

use crate::itemdata::ItemDef;

use super::tooltip::{item_name_color, player_crit};

/// `Main.cs:20634-20635` — the prefix rows' negative/positive colours:
/// `color = (190, 120, 120)`, `color2 = (120, 190, 120)`.
pub(crate) const PREFIX_NEGATIVE: [f32; 4] = [190.0 / 255.0, 120.0 / 255.0, 120.0 / 255.0, 1.0];
pub(crate) const PREFIX_POSITIVE: [f32; 4] = [120.0 / 255.0, 190.0 / 255.0, 120.0 / 255.0, 1.0];

/// The world/session gates the second half's rows read. Every flag is the
/// C# static's port stand-in; the draw site fills them from the App.
#[derive(Clone, Copy, Debug, Default)]
pub struct TooltipWorld {
    /// `Main.expertMode` — the well-fed expert row (`Main.cs:21005`).
    pub expert_mode: bool,
    /// `NPC.downedPlantBoss` — the cursed-by-Plantera swap (`:20961`).
    pub downed_plant_boss: bool,
    /// `Player.downedDD2EventAnyDifficulty` — the crystal-gear warning
    /// (`:21000`). Derived from `DD2Event.DownedInvasionAnyDifficulty`
    /// (`DD2Event.cs:89-99`, T1 ‖ T2 ‖ T3 — `world.downed_dd2_any()`),
    /// the getter `Player.cs:24450-24453` latches into the .plr flag.
    pub downed_dd2_any: bool,
    /// `Main.tenthAnniversaryWorld` (`:20973`).
    pub tenth_anniversary: bool,
    /// `Main.getGoodWorld` (`:20978`).
    pub get_good_world: bool,
    /// `SpecialSeedFeatures.Mechdusa` (`SpecialSeedFeatures.cs:119-129`:
    /// `remixWorld && getGoodWorld`).
    pub mechdusa: bool,
    /// `Main.npcShop > 0` — the Discount Card / Greedy Ring's shopping row
    /// (`:20818`).
    pub npc_shop: bool,
    /// The local player's `Main.masterColor` (`Main.cs:1235`, the −13
    /// master-rarity name pulse, `DoUpdate_AnimateCursorColors`
    /// `:18225-18235`). Init 1f (`Main.Initialize`, `:66833`).
    pub master_color: f32,
    /// `Main.DiscoR/G/B` (`Main.cs:959-963`, `DoUpdate_AnimateDiscoRGB`
    /// `:19591-19648`) — the −12 expert name colour. Init (255, 0, 0).
    pub disco: [u8; 3],
}

impl TooltipWorld {
    /// The `Main.Initialize` stock (`:66780`, `:66833-66834`) — the value a
    /// context-free read (tests, menus) sees.
    pub const STOCK: Self = Self {
        expert_mode: false,
        downed_plant_boss: false,
        downed_dd2_any: false,
        tenth_anniversary: false,
        get_good_world: false,
        mechdusa: false,
        npc_shop: false,
        master_color: 1.0,
        disco: [255, 0, 0],
    };
}

/// The hovered INSTANCE's state beyond the kind defaults — the flags
/// `ItemSlot.MouseHover` stamps (`ItemSlot.cs:3183-3198`) and the inventory
/// favourite flag. The kind defaults ride the `ItemDef`; a prefixed hover's
/// stat deltas ride `prefix::prefixed_stats_of` through [`hover_stats`].
#[derive(Clone, Copy, Debug)]
pub struct HoverFlags {
    /// `Item.tooltipContext` (`ItemSlot.cs:3184`): the hovered slot's
    /// `ItemSlot.Draw` context — 0 grid / 1 coin / 2 ammo / 8,10 worn
    /// armour / 9,11 social vanity / 12 dye / 16-20 misc equips / 33 misc
    /// dye / 6 trash / 4 bank / 13 hotbar / 22 craft. −1 = no slot context
    /// (the e16 full HoverRef→ctx map, research/128 §4.3).
    pub context: i32,
    /// `Item.favorited` (`Main.cs:20655`).
    pub favorited: bool,
    /// `Item.social` — context 9/11's stamp (`ItemSlot.cs:3191-3193`).
    pub social: bool,
    /// `Item.wornArmor` — context 8's stamp (`ItemSlot.cs:3188-3189`).
    pub worn_armor: bool,
    /// `Item.BestiaryNotes` — the bestiary hover line's cache
    /// (`UIBestiaryInfoItemLine.cs:155`); unported (see the module doc).
    pub bestiary_notes: Option<&'static str>,
}

impl Default for HoverFlags {
    fn default() -> Self {
        Self {
            context: -1,
            favorited: false,
            social: false,
            worn_armor: false,
            bestiary_notes: None,
        }
    }
}

/// `ItemSlot.canLoadoutShareAt` (`ItemSlot.cs:395-399`): contexts 8-12
/// (the armour/vanity/accessory/dye columns) — the loadout-share arm of
/// the tooltip head (`Main.cs:20638`).
pub fn can_loadout_share_at(context: i32) -> bool {
    matches!(context, 8..=12)
}

/// `ItemID.Sets.UsesCursedByPlanteraTooltip` (`ItemID.cs:410`): the five
/// biome keys + the Dungeon Desert Key 4714.
fn uses_cursed_by_plantera_tooltip(kind: u16) -> bool {
    matches!(kind, 1533 | 1534 | 1535 | 1536 | 1537 | 4714)
}

/// `ItemID.Sets.IsWireableStatue` (`ItemID.cs:1581`): the statues the
/// Wireable row admits through the tile-105 exception (`Main.cs:20942`).
fn is_wireable_statue(kind: u16) -> bool {
    matches!(
        kind,
        441 | 449
            | 466
            | 471
            | 3651
            | 3652
            | 3653
            | 3654
            | 3655
            | 3656
            | 3657
            | 3658
            | 3659
            | 3660
            | 3661
            | 3662
            | 3708
            | 3709
            | 3710
            | 3711
            | 3712
            | 3713
            | 3714
            | 3715
            | 3716
            | 3717
            | 3718
            | 3719
            | 3720
            | 4397
            | 4360
            | 4342
            | 4466
            | 5317
            | 5318
            | 5319
            | 440
            | 443
            | 444
            | 445
            | 446
            | 452
            | 454
            | 459
            | 463
            | 464
            | 478
            | 473
            | 2672
            | 438
            | 453
            | 476
            | 477
            | 470
            | 4355
            | 4276
    )
}

/// `ItemID.Sets.PlaceTileOnAltUse` (`ItemID.cs:412`): the torches/glowsticks
/// and placeable-on-alt-use family the "Places…" row admits without
/// `consumable` (`Main.cs:20921`).
fn place_tile_on_alt_use(kind: u16) -> bool {
    matches!(
        kind,
        4367 | 4368
            | 4369
            | 4370
            | 4371
            | 4379
            | 4610
            | 4611
            | 4612
            | 4613
            | 4648
            | 4649
            | 4650
            | 4651
            | 4669
            | 4670
            | 4671
            | 4674
            | 4675
            | 4676
            | 4677
            | 4681
            | 4683
            | 4684
            | 261
            | 1338
            | 1992
            | 1994
            | 1995
            | 1996
            | 1997
            | 1998
            | 1999
            | 2000
            | 2001
            | 2002
            | 2003
            | 2004
            | 2006
            | 2007
            | 2015
            | 2016
            | 2017
            | 2018
            | 2019
            | 2121
            | 2122
            | 2123
            | 2156
            | 2157
            | 2205
            | 2673
            | 2740
            | 2889
            | 2890
            | 2891
            | 2892
            | 2893
            | 2894
            | 2895
            | 3191
            | 3192
            | 3193
            | 3194
            | 3563
            | 3564
            | 4068
            | 4069
            | 4070
            | 4274
            | 4334
            | 4335
            | 4336
            | 4337
            | 4338
            | 4339
            | 4340
            | 4359
            | 4361
            | 4362
            | 4363
            | 4373
            | 4374
            | 4375
            | 4395
            | 4418
            | 4419
            | 4464
            | 4465
            | 4480
            | 4482
            | 4831
            | 4832
            | 4833
            | 4834
            | 4835
            | 4836
            | 4837
            | 4838
            | 4839
            | 4840
            | 4841
            | 4842
            | 4843
            | 4844
            | 4845
            | 4847
            | 4849
            | 4961
            | 5132
            | 5212
            | 5300
            | 5311
            | 5312
            | 5313
            | 5350
            | 5511
    )
}

/// `BuffID.Sets.IsWellFed` (`BuffID.cs:43`): the three fed tiers.
fn is_well_fed(buff_type: i32) -> bool {
    matches!(buff_type, 26 | 206 | 207)
}

/// The eleven yoyo kinds whose tooltip carries the OneDropLogo row
/// (`Main.cs:21016`).
fn is_yoyo_logo_kind(kind: u16) -> bool {
    matches!(
        kind,
        3262 | 3282 | 3283 | 3284 | 3285 | 3286 | 3316 | 3315 | 3317 | 3291 | 3389
    )
}

/// The twelve DD2 crystal-gear kinds the Etherian-Mana warning gates
/// (`Main.cs:21000`).
fn is_dd2_crystal_gear(kind: u16) -> bool {
    matches!(
        kind,
        3818 | 3819 | 3820 | 3824 | 3825 | 3826 | 3829 | 3830 | 3831 | 3832 | 3833 | 3834
    )
}

/// `Item.GetVisualCritChance` (`Item.cs:637-645`): the crit row's per-item
/// value — the Revolver 2269 adds the player's charging bonus, clamped
/// [0, 100] (`MathHelper.Clamp` on floats, then the int cast).
pub fn visual_crit_chance(
    kind: u16,
    crit: i32,
    player: Option<&crate::player::Player>,
) -> i32 {
    if kind == 2269 {
        if let Some(p) = player {
            if p.revolver_crit_chance_bonus > 0 {
                return (crit as f32 + p.revolver_crit_chance_bonus as f32).clamp(0.0, 100.0) as i32;
            }
        }
    }
    crit
}

/// The hovered instance's effective stats — the prefix applied over the
/// CALLER's row (`Item.Prefix`, `Item.cs:576-586`, through
/// `prefix::prefixed_stats_of` — the e16 caller seam, research/128 §3:
/// the C#'s `HoverItem` IS the hovered instance, so a witness-overridden
/// row must reach the prefixed math; the registry re-read cannot see it).
pub fn hover_stats(d: &ItemDef, prefix: u16) -> crate::prefix::PrefixedStats {
    crate::prefix::prefixed_stats_of(d, prefix)
}

/// The name row's colour (`Main.cs:20328-20332` + `GetItemNameColor`
/// `:20557-20629`): an expert item overrides `rare` to −12 (the disco
/// cycle); −13 is the master pulse `(255, (byte)(masterColor * 200f), 0)`.
/// `diff` 1/2 are the mediumcore/hardcore CHARACTER colours (`Main.mcColor`
/// `(1, 0.6, 0)` / `Main.hcColor` `(1, 0.15, 0.1)`, `Main.cs:66765-66766`) —
/// the LAST overrides, after every rarity arm (`:20620-20627`). The item
/// tooltip's own call passes 0 (`DrawInterface_33_MouseText`, `:45419`);
/// the stealthed-remote-player hover passes `player.difficulty`
/// (`DrawMouseOver`, `:42934`) — that hover is unported, so the parameter
/// stays 0 at every live call site, but the arms are the C#'s own.
pub fn tooltip_name_color(rare: i32, expert: bool, diff: u8, world: &TooltipWorld) -> [f32; 4] {
    let rare = if expert { -12 } else { rare };
    let base = match rare {
        -13 => [1.0, (world.master_color * 200.0) as u8 as f32 / 255.0, 0.0, 1.0],
        -12 => [
            world.disco[0] as f32 / 255.0,
            world.disco[1] as f32 / 255.0,
            world.disco[2] as f32 / 255.0,
            1.0,
        ],
        _ => item_name_color(rare),
    };
    // `:20620-20627` — the diff overrides win over every rarity colour.
    match diff {
        // `Main.mcColor = new Color(1f, 0.6f, 0f)` (`Main.cs:66765`).
        1 => [1.0, 0.6, 0.0, 1.0],
        // `Main.hcColor = new Color(1f, 0.15f, 0.1f)` (`:66766`).
        2 => [1.0, 0.15, 0.1, 1.0],
        _ => base,
    }
}

/// `Main.DoUpdate_AnimateCursorColors`' master-colour half
/// (`Main.cs:18225-18235`): ±0.05 per tick with the direction flip at the
/// [0, 1] bounds. Returns the new `(masterColor, masterColorDir)`.
pub fn advance_master_color(color: f32, dir: i32) -> (f32, i32) {
    let mut color = color + dir as f32 * 0.05;
    let mut dir = dir;
    if color > 1.0 {
        color = 1.0;
        dir = -1;
    }
    if color < 0.0 {
        color = 0.0;
        dir = 1;
    }
    (color, dir)
}

/// The gate repairs + the second half's rows, in the C#'s emission order.
/// The FIRST half (`tooltip.rs::item_tooltip_lines`) owns the hover name
/// through the material row; this appends everything after
/// `Main.cs:20937`, and the first half calls the gate helpers below for the
/// rows it already emits.
///
/// `lines` arrives holding the first half's rows; `old_kb` is the instance
/// knockback the first half captured at `Main.cs:20333` BEFORE the
/// glove/buff/shroomite scales (research/128 §5-e16.3); `share_hint_flag`
/// is the head's `flag2` latch (`Main.cs:20651` — the unfavourited
/// shareable slot); `yoyo_logo` returns the row index the OneDropLogo
/// draws over (`Main.cs:21019`'s `yoyoLogo`), or `None`.
#[allow(clippy::too_many_arguments)]
pub fn append_second_half(
    kind: u16,
    d: &ItemDef,
    prefix: u16,
    old_kb: f32,
    share_hint_flag: bool,
    show_loadout_share_hint: bool,
    flags: HoverFlags,
    world: &TooltipWorld,
    player: Option<&crate::player::Player>,
    lines: &mut Vec<(String, [f32; 4])>,
) -> Option<usize> {
    let white = [1.0, 1.0, 1.0, 1.0];
    // `:20942-20946` — the Wireable row: a placed trigger/mechanism tile,
    // with the tile-105 statue exception riding IsWireableStatue. NOTE the
    // C#'s single `&` — both sides always evaluate (no short-circuit); the
    // port's `&&` is value-identical here (both operands are pure).
    if let Some(tile) = d.tile {
        let wiring = tr_core::tiledata::SET_IS_A_TRIGGER.has(tile)
            || tr_core::tiledata::SET_IS_A_MECHANISM.has(tile);
        if wiring && (tile != 105 || is_wireable_statue(kind)) {
            lines.push((crate::lang::common_item_tooltip("Wireable"), white));
        }
    }
    // `:20947-20951` — the chest / piggy-bank-family Container row.
    if matches!(d.tile, Some(21) | Some(467)) {
        lines.push((crate::lang::common_item_tooltip("Container"), white));
    }
    // `:20952-20956` — the pressure-plate / weighted-plate WireTrigger row.
    if matches!(d.tile, Some(441) | Some(468)) {
        lines.push((crate::lang::common_item_tooltip("WireTrigger"), white));
    }
    // `:20957-20972` — the per-item tooltip text with the cursed-by-Plantera
    // swap on line 0 (while Plantera stands, the first ToolTip line is
    // REPLACED by tip[59]). `Item.ToolTip` is `Lang.GetTooltip(type)`
    // (`Item.RebuildTooltip`, `Item.cs:1203-1209`, `Lang.cs:193-196`).
    if let Some(text) = crate::lang::item_tooltip_text(kind) {
        for (i, line) in text.split('\n').enumerate() {
            if i == 0 && uses_cursed_by_plantera_tooltip(kind) && !world.downed_plant_boss {
                lines.push((crate::lang::tip(59), white));
            } else {
                lines.push((line.to_string(), white));
            }
        }
    }
    // `:20973-20977` — the anniversary Wizard Hat row.
    if world.tenth_anniversary && kind == 238 {
        lines.push((
            crate::lang::common_item_tooltip("WizardHatDuringAnniversary"),
            white,
        ));
    }
    // `:20978-20982` — the FTW Hellstone-block row.
    if world.get_good_world && kind == 1127 {
        lines.push((crate::lang::common_item_tooltip("BurningBlock"), white));
    }
    // `:20983-20999` — the Mechdusa summon rows: in a Mechdusa world the
    // three mech summons REPLACE their last tooltip line; outside one the
    // Ocram-summon 5334 drops its last TWO lines for the dead-world notice.
    if world.mechdusa {
        if matches!(kind, 556 | 557 | 544) {
            lines.pop();
            lines.push((
                crate::lang::common_item_tooltip("MechSummonDuringEverything"),
                white,
            ));
        }
    } else if kind == 5334 {
        lines.pop();
        lines.pop();
        lines.push((
            crate::lang::common_item_tooltip("MechdusaSummonNotDuringEverything"),
            white,
        ));
    }
    // `:21000-21004` — the crystal gear's Etherian-Mana warning.
    if is_dd2_crystal_gear(kind) && !world.downed_dd2_any {
        lines.push((crate::lang::misc(104), white));
    }
    // `:21005-21009` — the well-fed expert row.
    if d.buff_type > 0 && is_well_fed(d.buff_type) && world.expert_mode {
        lines.push((crate::lang::misc(40), white));
    }
    // `:21010-21015` — the buff duration row: under an hour of ticks the
    // seconds arm (Math.Round on the f64 quotient — banker's), else the
    // minutes arm (the INTEGER `/60` first, then /60.0 rounded).
    if d.buff_time > 0 {
        let row = if d.buff_time / 60 < 60 {
            crate::lang::common_item_tooltip_fmt(
                "SecondDuration",
                tr_core::damage::round_bankers(d.buff_time as f64 / 60.0) as i32,
            )
        } else {
            crate::lang::common_item_tooltip_fmt(
                "MinuteDuration",
                tr_core::damage::round_bankers((d.buff_time / 60) as f64 / 60.0) as i32,
            )
        };
        lines.push((row, white));
    }
    // `:21016-21021` — the yoyo logo's " " placeholder row; the draw pass
    // paints the OneDropLogo over it (`:20408-20439`, deferred).
    let mut yoyo_logo = None;
    if is_yoyo_logo_kind(kind) {
        yoyo_logo = Some(lines.len());
        lines.push((" ".to_string(), white));
    }
    // `:21022-21026` — the Expert row.
    if d.expert {
        lines.push((crate::lang::game_ui("Expert"), white));
    }
    // `:21027-21031` — the Master row: the INSTANCE rare. `shift_rare`
    // skips the clamp for rare ≤ −11 (`Item.cs:603`), so a prefix on a
    // master item shifts −13 by ±1/±2 and the row then drops — the
    // instance compare keeps that quirk.
    let instance_rare = hover_stats(d, prefix).rare;
    if instance_rare == -13 {
        lines.push((crate::lang::game_ui("Master"), white));
    }
    // `:21032-21266` — the prefix comparison rows.
    if prefix > 0 {
        append_prefix_rows(kind, d, prefix, old_kb, lines);
    }
    // `:21267-21311` — the armor set-bonus row (the wave-D table). The
    // unworn arm reads no player (`array[0].GetTooltipForSinglePiece`);
    // the worn arm queries the LocalPlayer's worn pieces.
    if !flags.worn_armor {
        if let Some(row) = crate::armor_sets::SETS
            .iter()
            .find(|r| r.head == kind as i32 || r.body == kind as i32 || r.legs == kind as i32)
        {
            // `array[0].GetTooltipForSinglePiece` — grey at alpha 190.
            lines.push((
                crate::armor_sets::tooltip_for_single_piece(row, kind as i32),
                [130.0 / 255.0, 130.0 / 255.0, 130.0 / 255.0, 190.0 / 255.0],
            ));
        }
    } else if let Some(p) = player {
        let (head, body, legs) = (
            p.equipped[0] as i32,
            p.equipped[1] as i32,
            p.equipped[2] as i32,
        );
        if let Some((row, result)) =
            crate::armor_sets::best_worn_set(kind as i32, head, body, legs)
        {
            let color = if result.complete() {
                // Color.LimeGreen = (50, 205, 50).
                [50.0 / 255.0, 205.0 / 255.0, 50.0 / 255.0, 1.0]
            } else {
                [130.0 / 255.0, 130.0 / 255.0, 130.0 / 255.0, 1.0]
            };
            lines.push((
                crate::armor_sets::tooltip_for_worn_armor(row, head, body, legs, result),
                color,
            ));
        }
    }
    // `:21312-21317` — the loadout-share hint (flag2 && the setting), in
    // `Main.OurFavoriteColor` (255, 231, 69) at alpha 190 (`Main.cs:66748`).
    // The e16 row (research/128 §5-e16.4): flag2 latches in the head when
    // an UNfavourited shareable slot (ctx 8-12) is hovered.
    if share_hint_flag && show_loadout_share_hint {
        lines.push((
            crate::lang::text("UI.ItemLoadoutShareHint"),
            [255.0 / 255.0, 231.0 / 255.0, 69.0 / 255.0, 190.0 / 255.0],
        ));
    }
    // `:21318-21333` — the Journey research rows: no session sacrifice
    // tracker (module doc).
    // `:21334-21342` — the bestiary notes: the only writer is the unported
    // bestiary hover line; the field rides HoverFlags for that port.
    if let Some(notes) = flags.bestiary_notes {
        if !notes.trim().is_empty() {
            for line in notes.split('\n') {
                lines.push((line.to_string(), white));
            }
        }
    }
    yoyo_logo
}

/// The prefix comparison rows (`Main.cs:21032-21266`): each stat delta
/// against the fresh `SetDefaults` comparison item, `Math.Round` (banker's)
/// on the f64 percentage, the `+` prefix only on the positive arm, the
/// green/red colour pair — the MANA row's colour flips (`> 0` is red,
/// `:21093`). The knockback row compares `oldKB` — the instance knockback
/// captured at `:20333` BEFORE the draw pass's glove/buff/shroomite scaling
/// — against the defaults. The fixed rows for prefixes 62-80 close the
/// block (`:21152-21265`). `old_kb` arrives captured by the FIRST half at
/// `:20333` — the caller-seam discipline (research/128 §5-e16.3): the
/// prefix application runs over the caller's row, so a witness-overridden
/// knockback reaches BOTH the delta row and the word tier.
fn append_prefix_rows(
    kind: u16,
    d: &ItemDef,
    prefix: u16,
    old_kb: f32,
    lines: &mut Vec<(String, [f32; 4])>,
) {
    let stats = hover_stats(d, prefix);
    // `Main.tooltipPrefixComparisonItem` — a fresh SetDefaults item of the
    // same type (`:21034-21039`); the port's registry row IS that item.
    let round_pct = |delta: f64, base: f64| -> f64 {
        tr_core::damage::round_bankers(delta / base * 100.0) as f64
    };
    // The C# prints the DOUBLE (`"+" + num7`): whole values print without a
    // decimal point (`15`, not `15.0`); .NET's shortest-round-trip shape
    // matches Rust's `{}` for the f64s these ratios produce.
    let sign = |v: f64| -> String {
        if v > 0.0 { format!("+{v}") } else { format!("{v}") }
    };
    // `:21040-21053` — damage %.
    if d.damage != stats.damage {
        let num7 = round_pct((stats.damage - d.damage) as f64, d.damage as f64);
        lines.push((
            format!("{}{}", sign(num7), crate::lang::tip(39)),
            if num7 < 0.0 { PREFIX_NEGATIVE } else { PREFIX_POSITIVE },
        ));
    }
    // `:21054-21067` — speed % (the delta × −1: faster is positive).
    if d.use_animation as i32 != stats.use_animation {
        let num8 = round_pct(
            (stats.use_animation - d.use_animation as i32) as f64,
            d.use_animation as f64,
        ) * -1.0;
        lines.push((
            format!("{}{}", sign(num8), crate::lang::tip(40)),
            if num8 < 0.0 { PREFIX_NEGATIVE } else { PREFIX_POSITIVE },
        ));
    }
    // `:21068-21081` — crit points (no rounding, the float delta prints).
    if d.crit != stats.crit {
        let num9 = (stats.crit - d.crit) as f64;
        lines.push((
            format!("{}{}", sign(num9), crate::lang::tip(41)),
            if num9 < 0.0 { PREFIX_NEGATIVE } else { PREFIX_POSITIVE },
        ));
    }
    // `:21082-21095` — mana cost % (the colour INVERTS: costlier is red).
    if d.mana != stats.mana {
        let num10 = round_pct((stats.mana - d.mana) as f64, d.mana as f64);
        lines.push((
            format!("{}{}", sign(num10), crate::lang::tip(42)),
            if num10 > 0.0 { PREFIX_NEGATIVE } else { PREFIX_POSITIVE },
        ));
    }
    // `:21096-21109` — size %.
    if d.scale != stats.scale {
        let num11 = round_pct((stats.scale - d.scale) as f64, d.scale as f64);
        lines.push((
            format!("{}{}", sign(num11), crate::lang::tip(43)),
            if num11 < 0.0 { PREFIX_NEGATIVE } else { PREFIX_POSITIVE },
        ));
    }
    // `:21110-21123` — velocity %.
    if d.shoot_speed != stats.shoot_speed {
        let num12 = round_pct(
            (stats.shoot_speed - d.shoot_speed) as f64,
            d.shoot_speed as f64,
        );
        lines.push((
            format!("{}{}", sign(num12), crate::lang::tip(44)),
            if num12 < 0.0 { PREFIX_NEGATIVE } else { PREFIX_POSITIVE },
        ));
    }
    // `:21124-21137` — knockback %: `oldKB` is the instance knockback
    // CAPTURED at `:20333` BEFORE `MouseText_DrawItemTooltip`'s glove/buff/
    // shroomite scaling (`:20334-20350` mutates hoverItem.knockBack in place,
    // but the captured float rides the ref parameter) — i.e. the prefixed
    // instance's own knockback, unscaled. The first half passes it in.
    if d.knock_back != old_kb {
        let num13 = round_pct((old_kb - d.knock_back) as f64, d.knock_back as f64);
        lines.push((
            format!("{}{}", sign(num13), crate::lang::tip(45)),
            if num13 < 0.0 { PREFIX_NEGATIVE } else { PREFIX_POSITIVE },
        ));
    }
    // `:21138-21144` — armor penetration points, the `{0:+0;-#}` format.
    if stats.armor_penetration != 0 {
        lines.push((
            crate::lang::common_item_tooltip_fmt("PrefixArmorPenetration", stats.armor_penetration),
            if stats.armor_penetration < 0 { PREFIX_NEGATIVE } else { PREFIX_POSITIVE },
        ));
    }
    // `:21145-21151` — summon tag damage points, the same format.
    if stats.tag_damage != 0 {
        lines.push((
            crate::lang::common_item_tooltip_fmt("PrefixTagDamage", stats.tag_damage),
            if stats.tag_damage < 0 { PREFIX_NEGATIVE } else { PREFIX_POSITIVE },
        ));
    }
    // `:21152-21265` — the accessory prefixes' fixed rows. Every arm is its
    // own `if` in the C# (a prefix matches at most one anyway).
    let fixed: Option<(String, usize)> = match prefix {
        62 => Some(("+1".into(), 25)),
        63 => Some(("+2".into(), 25)),
        64 => Some(("+3".into(), 25)),
        65 => Some(("+4".into(), 25)),
        // `:21178` — the literal carries the trailing space: "+20 " + tip[31].
        66 => Some(("+20 ".into(), 31)),
        67 => Some(("+2".into(), 5)),
        68 => Some(("+4".into(), 5)),
        69 => Some(("+1".into(), 39)),
        70 => Some(("+2".into(), 39)),
        71 => Some(("+3".into(), 39)),
        72 => Some(("+4".into(), 39)),
        73 => Some(("+1".into(), 46)),
        74 => Some(("+2".into(), 46)),
        75 => Some(("+3".into(), 46)),
        76 => Some(("+4".into(), 46)),
        77 => Some(("+1".into(), 47)),
        78 => Some(("+2".into(), 47)),
        79 => Some(("+3".into(), 47)),
        80 => Some(("+4".into(), 47)),
        _ => None,
    };
    if let Some((amount, tip)) = fixed {
        lines.push((format!("{amount}{}", crate::lang::tip(tip)), PREFIX_POSITIVE));
    }
}

/// The knockback DISPLAY row's chain (`Main.cs:20333-20350` +
/// `:20753-20765`): the gloves' melee ×2 / the buff's +0.5 additive scale,
/// the shroomite ranged stealth scale (both applied to `hoverItem.knockBack`
/// in `MouseText_DrawItemTooltip` before GetLinesInfo), then the summon
/// minionKB add, the quiver/stake ×1.1 INT truncation and the Psycho Knife
/// 3106 self-hold stealth arm — in the C#'s order. `held_kind` is
/// `Main.player[myPlayer].inventory[selectedItem].type` (`:20762`).
pub fn display_knockback(
    kind: u16,
    d: &ItemDef,
    instance_kb: f32,
    player: Option<&crate::player::Player>,
    held_kind: u16,
) -> f32 {
    let mut kb = instance_kb;
    if let Some(p) = player {
        // `:20334-20346` — the glove/buff multiplier.
        let mut num = 1.0f32;
        if d.melee && p.kb_glove {
            num += 1.0;
        }
        if p.kb_buff {
            num += 0.5;
        }
        if num != 1.0 {
            kb *= num;
        }
        // `:20347-20350` — the shroomite ranged stealth scale.
        if d.ranged && p.shroomite_stealth {
            kb *= 1.0 + (1.0 - p.stealth) * 0.5;
        }
        // `:20754-20757` — the summon minionKB add.
        if d.summon {
            kb += p.minion_kb;
        }
        // `:20758-20761` — the quiver/stake ×1.1, INT-truncated.
        if (p.magic_quiver && d.use_ammo == crate::ammo::AMMO_ARROW)
            || d.use_ammo == crate::ammo::AMMO_STAKE
        {
            kb = (kb * 1.1) as i32 as f32;
        }
        // `:20762-20765` — the Psycho Knife holding itself.
        if held_kind == 3106 && kind == 3106 {
            kb += kb * (1.0 - p.stealth);
        }
    } else if d.use_ammo == crate::ammo::AMMO_STAKE {
        // `:20758`'s second disjunct reads no player.
        kb = (kb * 1.1) as i32 as f32;
    }
    kb
}

/// The first half's gate repairs, shared so `tooltip.rs` and the witnesses
/// read one source.

/// The damage row's gate (`Main.cs:20669`): `damage > 0` (the INSTANCE's
/// prefixed damage), the notAmmo/useStyle exclusion and the coin-gun arm —
/// coins 71-74 only show damage while the player carries the Coin Gun 905
/// (`Player.HasItem`, `Player.cs:55894-55904`: the 58-slot scan — the grid
/// plus the coin/ammo rows; the draw site computes `has_coin_gun` over
/// grid + `equip.coins` + `equip.ammo`, research/128 §4.4 — the old
/// `.take(58)` over the 50-slot grid slice never reached rows 50-57).
pub fn damage_row_gate(kind: u16, d: &ItemDef, instance_damage: i32, has_coin_gun: bool) -> bool {
    instance_damage > 0
        && (!d.not_ammo || d.use_style != 0)
        && (!(71..=74).contains(&kind) || has_coin_gun)
}

/// The use-time row's gate (`Main.cs:20717`): `useStyle != 0` and — for a
/// summon item — the shot must be a whip (`ProjectileID.Sets.IsAWhip`).
pub fn use_time_row_gate(d: &ItemDef) -> bool {
    d.use_style != 0
        && (!d.summon
            || d.projectile
                .is_some_and(|p| tr_core::whip::is_whip(p)))
}

/// The equipable row's shopping arm (`Main.cs:20818-20825`): the Discount
/// Card 854 / Greedy Ring 3035 print tip[60] while a shop is open.
pub fn equipable_row(kind: u16, npc_shop: bool) -> String {
    if matches!(kind, 854 | 3035) && npc_shop {
        crate::lang::tip(60)
    } else {
        crate::lang::tip(23)
    }
}

/// The life-restore row (`Main.cs:20876-20887`): the Strange Brew 3001
/// prints the RANGE form (its own healLife..120).
pub fn restores_life_row(kind: u16, heal_life: i32) -> String {
    if kind == 3001 {
        tr_core::localization::format_net(
            &crate::lang::common_item_tooltip("RestoresLifeRange"),
            &[heal_life.to_string(), "120".to_string()],
        )
    } else {
        crate::lang::common_item_tooltip_fmt("RestoresLife", heal_life)
    }
}

/// The place/ammo/consumable ladder's gates (`Main.cs:20919-20936`):
/// the place gate is `createWall > 0 || createTile > -1 || type == 849`
/// (the e16 `wall > 0` check, research/128 §2 row 16 — `wall.is_some()`
/// would admit a `Some(0)` the C# refuses); inside it the row needs
/// `PlaceTileOnAltUse[type] || consumable`. The ammo row excludes
/// `notAmmo`, the consumable row excludes the extractinator-only family.
fn places(d: &ItemDef, kind: u16) -> bool {
    d.wall.map_or(false, |w| w > 0) || d.tile.is_some() || kind == 849
}

pub fn place_row_active(d: &ItemDef, kind: u16) -> bool {
    places(d, kind) && (place_tile_on_alt_use(kind) || d.consumable)
}

pub fn ammo_row_active(d: &ItemDef, kind: u16) -> bool {
    !places(d, kind) && d.ammo > 0 && !d.not_ammo
}

pub fn consumable_row_active(d: &ItemDef, kind: u16) -> bool {
    !places(d, kind)
        && !(d.ammo > 0 && !d.not_ammo)
        && d.consumable
        && !d.extractinator_consumable
}

/// The social row (`Main.cs:20664-20668`): a context-9/11 hover whose item
/// is neither vanity nor an info accessory prints tip[61].
pub fn social_row(flags: HoverFlags, d: &ItemDef) -> Option<String> {
    (flags.social && !d.vanity && !d.has_vanity_effects).then(|| crate::lang::tip(61))
}

/// The favourite rows (`Main.cs:20655-20663`): tip[56] + tip[57], and the
/// bank-blocked notice when the open container is the item's own bank
/// (`ChestUI.IsBlockedFromTransferIntoChest`, `ChestUI.cs:1291-1302` — the
/// Money Trough 3213 in bank −2, the Void Lens 4131 / Closed Void Bag 5325
/// in bank4 −5).
pub fn favorite_rows(kind: u16, favorited: bool, open_chest: i32) -> Vec<String> {
    if !favorited {
        return Vec::new();
    }
    let mut rows = vec![crate::lang::tip(56), crate::lang::tip(57)];
    let blocked = (kind == 3213 && open_chest == -2)
        || (matches!(kind, 4131 | 5325) && open_chest == -5);
    if open_chest != -1 && blocked {
        rows.push(crate::lang::text("UI.ItemCannotBePlacedInsideItself"));
    }
    rows
}

/// The tile-wand row (`Main.cs:20828-20832`): tip[52] + the consumed item's
/// name (`Lang.GetItemNameValue`).
pub fn tile_wand_row(tile_wand: i32) -> Option<String> {
    (tile_wand > 0).then(|| {
        format!(
            "{}{}",
            crate::lang::tip(52),
            crate::lang::item_name(tile_wand as u16)
        )
    })
}

/// The crit row's value (`Main.cs:20701-20717`): the player's class crit,
/// minus the held item's own, plus the hovered item's VISUAL crit.
pub fn crit_row_value(
    kind: u16,
    d: &ItemDef,
    player: Option<&crate::player::Player>,
    held_crit: i32,
) -> i32 {
    let visual = visual_crit_chance(kind, d.crit, player);
    player.map_or(visual, |p| player_crit(p, d.class) - held_crit + visual)
}

/// The damage row's value (`Main.cs:20671-20677`): the per-item tooltip
/// multiplier, the rapid-attack bonus (the tooltip read passes
/// `increaseBonus: false` — `min(rapidAttackBonus, 0.5)` behind the
/// `RapidAttackBonusDamage` set guard), then the player's weapon-damage
/// multiplier with the `5E-06f` epsilon before the int truncation.
pub fn damage_row_value(
    kind: u16,
    base_damage: i32,
    player: Option<&crate::player::Player>,
    d: &ItemDef,
) -> i32 {
    let mut num2 = (base_damage as f32 * super::tooltip::tooltip_damage_multiplier(kind)) as i32;
    if let Some(p) = player {
        // `:20673-20676` — the set guard, then the read-only scale.
        if matches!(kind, 2608 | 3349 | 672) {
            num2 = (num2 as f32 * (1.0 + p.rapid_attack_scale())) as i32;
        }
        let multiplier = p.weapon_damage_multiplier_with_ammo(d.class, kind, d.use_ammo);
        return (multiplier * num2 as f32 + 5e-6) as i32;
    }
    num2
}

#[cfg(test)]
mod w4_f28_tests {
    use super::*;
    use crate::ui::tooltip::{item_tooltip_lines, TooltipContext};

    /// The language rows the witnesses read; the guard serialises the
    /// register→read sequence against the culture-switching tests (the
    /// `s1_e_20_*` pattern in `tooltip.rs`).
    fn register_lang() {
        crate::lang::test_register_source(
            "Terraria.Localization.Content.en-US.Items.json",
            r#"{
                "ItemName": { "Wood": "Wood", "JungleKey": "Jungle Key",
                              "MechanicalWorm": "Mechanical Worm",
                              "MechdusaSummon": "Suspicious Looking Skull",
                              "Code1": "Code 1", "Terrarian": "Terrarian",
                              "Apple": "Apple", "ShadowScale": "Shadow Scale",
                              "Chest": "Chest", "FishStatue": "Fish Statue",
                              "ArmorStatue": "Armor Statue",
                              "LivingWoodWand": "Living Wood Wand",
                              "StrangeBrew": "Strange Brew",
                              "SlimeStaff": "Slime Staff", "FireWhip": "Fire Whip",
                              "FalconBlade": "Falcon Blade",
                              "IronBroadsword": "Iron Broadsword",
                              "GravityGlobe": "Gravity Globe",
                              "DiscountCard": "Discount Card",
                              "Revolver": "Revolver", "Gel": "Gel",
                              "CopperCoin": "Copper Coin",
                              "WizardHat": "Wizard Hat",
                              "CrispyHoneyBlock": "Crispy Honey Block",
                              "MiningHelmet": "Mining Helmet",
                              "MeteorHelmet": "Meteor Helmet",
                              "ShadowGreaves": "Shadow Greaves",
                              "DepthMeter": "Depth Meter" },
                "ItemTooltip": {
                    "JungleKey": "Unlocks a Jungle Chest in the dungeon",
                    "MechanicalWorm": "Summons The Destroyer",
                    "MechdusaSummon": "Summons ???\n'You really shouldn't'",
                    "Apple": "{$CommonItemTooltip.MinorStats}\n'An apple a day keeps Doctor Bones away!'",
                    "SlimeStaff": "Summons a baby slime to fight for you",
                    "FireWhip": "{$CommonItemTooltip.Whips}\nStrike enemies with blazing energy",
                    "LivingWoodWand": "Places living wood",
                    "StrangeBrew": "Side Effects May Include:\n-Unpredictable Healing",
                    "FalconBlade": "Rapid attacks deal more damage",
                    "GravityGlobe": "Allows the holder to reverse gravity\nPress Up to change gravity",
                    "DiscountCard": "Lowers shop prices by 20%",
                    "Revolver": "Well-timed shots reduce cooldown and increase critical strike chance",
                    "Gel": "'Both tasty and flammable'",
                    "WizardHat": "5% increased magic damage",
                    "DepthMeter": "Displays depth"
                }
            }"#,
        );
        crate::lang::test_register_source(
            "Terraria.Localization.Content.en-US.Game.json",
            r#"{
                "CommonItemTooltip": {
                    "Wireable": "Wireable",
                    "Container": "Collects nearby dropped items when signalled",
                    "WireTrigger": "{InputTrigger_InteractWithTile} when placed to signal",
                    "RestoresLife": "Restores {0} life",
                    "RestoresLifeRange": "Restores from {0} to {1} life",
                    "SecondDuration": "{0} second duration",
                    "MinuteDuration": "{0} minute duration",
                    "PrefixArmorPenetration": "{0:+0;-#} armor penetration",
                    "PrefixTagDamage": "{0:+0;-#} summon tag damage",
                    "WizardHatDuringAnniversary": "Increases your max number of minions by 1",
                    "BurningBlock": "Burns you when touched without special protection",
                    "MechSummonDuringEverything": "'Part of a set'",
                    "MechdusaSummonNotDuringEverything": "'It has no effect in this world'",
                    "MinorStats": "Minor improvements to all stats"
                },
                "GameUI": { "Expert": "Expert", "Master": "Master",
                            "PrecentFishingPower": "{0}% fishing power",
                            "BaitRequired": "Requires bait to catch fish" },
                "UI": { "ItemCannotBePlacedInsideItself": "Placing into itself will be blocked",
                        "ItemLoadoutShared": "Shared with other loadouts",
                        "ItemLoadoutShareHint": "{InputTriggerUI_FavoriteItem} to share across loadouts",
                        "ItemSetBonusGeneral": "Set Bonus: {Description}",
                        "ItemSetBonusEquipped": "Set Bonus ({Numerator}/{Denominator})\n{Description}",
                        "ItemSetBonusDecidedByHead": "Changes with head piece" }
            }"#,
        );
    }

    fn def(kind: u16) -> &'static ItemDef {
        crate::registry::get(kind).expect("the kind's row")
    }

    fn texts(rows: &[(String, [f32; 4])]) -> Vec<&str> {
        rows.iter().map(|(t, _)| t.as_str()).collect()
    }

    /// W4-f28 — the Wireable row's statue exception (`Main.cs:20942`): the
    /// Fish Statue 444 is in `IsWireableStatue` (wireable), the Armor Statue
    /// 360 is not (the tile-105 exception blocks it); the Chest 48 prints
    /// Wireable AND Container (tile 21 is a mechanism).
    #[test]
    fn w4_f28_the_wireable_rows_follow_the_statue_exception() {
        let _g = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        register_lang();
        let rows = item_tooltip_lines(444, def(444), TooltipContext::default());
        assert!(texts(&rows).contains(&"Wireable"), "{:?}", texts(&rows));
        let rows = item_tooltip_lines(360, def(360), TooltipContext::default());
        assert!(!texts(&rows).contains(&"Wireable"), "the armor statue is not wireable");
        let rows = item_tooltip_lines(48, def(48), TooltipContext::default());
        assert!(texts(&rows).contains(&"Wireable"));
        assert!(texts(&rows).contains(&"Collects nearby dropped items when signalled"));
    }

    /// W4-f28 — the cursed-by-Plantera swap (`Main.cs:20961`): the Jungle
    /// Key's first tooltip line is tip[59] while Plantera stands, the plain
    /// text once she is downed.
    #[test]
    fn w4_f28_the_jungle_key_swaps_its_first_line_until_plantera_falls() {
        let _g = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        register_lang();
        let cursed = crate::lang::tip(59);
        let rows = item_tooltip_lines(1533, def(1533), TooltipContext::default());
        assert!(texts(&rows).contains(&cursed.as_str()), "{:?}", texts(&rows));
        assert!(!texts(&rows).contains(&"Unlocks a Jungle Chest in the dungeon"));
        let ctx = TooltipContext {
            world: TooltipWorld { downed_plant_boss: true, ..TooltipWorld::STOCK },
            ..Default::default()
        };
        let rows = item_tooltip_lines(1533, def(1533), ctx);
        assert!(!texts(&rows).contains(&cursed.as_str()));
        assert!(texts(&rows).contains(&"Unlocks a Jungle Chest in the dungeon"));
    }

    /// W4-f28 — the Mechdusa summon rows (`Main.cs:20983-20999`): in a
    /// Mechdusa world the Mechanical Worm's last line becomes 'Part of a
    /// set'; outside one the Suspicious Looking Skull drops its last TWO
    /// lines for the dead-world notice.
    #[test]
    fn w4_f28_the_mechdusa_summon_rows_replace_the_tail() {
        let _g = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        register_lang();
        let ctx = TooltipContext {
            world: TooltipWorld { mechdusa: true, ..TooltipWorld::STOCK },
            ..Default::default()
        };
        let rows = item_tooltip_lines(556, def(556), ctx);
        let t = texts(&rows);
        assert!(t.contains(&"'Part of a set'"), "{t:?}");
        assert!(!t.contains(&"Summons The Destroyer"), "the last line was REPLACED");
        // Outside Mechdusa: the skull's two tooltip lines drop, the notice
        // takes their place (the numLines-- twice arm).
        let rows = item_tooltip_lines(5334, def(5334), TooltipContext::default());
        let t = texts(&rows);
        assert!(t.contains(&"'It has no effect in this world'"), "{t:?}");
        assert!(!t.contains(&"Summons ???"));
        assert!(!t.contains(&"'You really shouldn't'"));
    }

    /// W4-f28 — the anniversary/FTW rows (`Main.cs:20973-20982`) and the
    /// DD2 crystal-gear warning (`:21000-21004`).
    #[test]
    fn w4_f28_the_seed_and_dd2_rows_gate_on_the_world_flags() {
        let _g = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        register_lang();
        let ctx = TooltipContext {
            world: TooltipWorld { tenth_anniversary: true, ..TooltipWorld::STOCK },
            ..Default::default()
        };
        let rows = item_tooltip_lines(238, def(238), ctx);
        assert!(texts(&rows).contains(&"Increases your max number of minions by 1"));
        let rows = item_tooltip_lines(238, def(238), TooltipContext::default());
        assert!(!texts(&rows).contains(&"Increases your max number of minions by 1"));
        let ctx = TooltipContext {
            world: TooltipWorld { get_good_world: true, ..TooltipWorld::STOCK },
            ..Default::default()
        };
        let rows = item_tooltip_lines(1127, def(1127), ctx);
        assert!(texts(&rows).contains(&"Burns you when touched without special protection"));
        // The FTW row stays off in a plain world.
        let rows = item_tooltip_lines(1127, def(1127), TooltipContext::default());
        assert!(!texts(&rows).contains(&"Burns you when touched without special protection"));
        // The DD2 popper 3818 warns until the crystal has been defended.
        let warning = crate::lang::misc(104);
        let rows = item_tooltip_lines(3818, def(3818), TooltipContext::default());
        assert!(texts(&rows).contains(&warning.as_str()), "{:?}", texts(&rows));
        let ctx = TooltipContext {
            world: TooltipWorld { downed_dd2_any: true, ..TooltipWorld::STOCK },
            ..Default::default()
        };
        let rows = item_tooltip_lines(3818, def(3818), ctx);
        assert!(!texts(&rows).contains(&warning.as_str()));
    }

    /// W4-f28 — the well-fed expert row (`Main.cs:21005-21009`) and the
    /// buff-duration row (`:21010-21015`): the Apple's 18000 ticks print
    /// the MINUTE arm (18000/60 = 300 s ≥ 60 → round(300/60) = 5); a
    /// 90-tick buff prints round(1.5) = 2 seconds (banker's).
    #[test]
    fn w4_f28_the_well_fed_and_duration_rows() {
        let _g = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        register_lang();
        let fed = crate::lang::misc(40);
        let rows = item_tooltip_lines(4009, def(4009), TooltipContext::default());
        assert!(!texts(&rows).contains(&fed.as_str()), "classic: no expert row");
        assert!(texts(&rows).contains(&"5 minute duration"), "{:?}", texts(&rows));
        let ctx = TooltipContext {
            world: TooltipWorld { expert_mode: true, ..TooltipWorld::STOCK },
            ..Default::default()
        };
        let rows = item_tooltip_lines(4009, def(4009), ctx);
        assert!(texts(&rows).contains(&fed.as_str()), "expert: the well-fed row");
        // The seconds arm: 90 ticks → Math.Round(1.5) = 2 (banker's).
        let mut apple = *def(4009);
        apple.buff_time = 90;
        let rows = item_tooltip_lines(4009, &apple, TooltipContext::default());
        assert!(texts(&rows).contains(&"2 second duration"), "{:?}", texts(&rows));
        // 30 ticks → Math.Round(0.5) = 0.
        apple.buff_time = 30;
        let rows = item_tooltip_lines(4009, &apple, TooltipContext::default());
        assert!(texts(&rows).contains(&"0 second duration"));
    }

    /// W4-f28 — the yoyo placeholder + Expert/Master rows
    /// (`Main.cs:21016-21031`): the Code 1 carries the " " row at the
    /// returned logo index; the Gravity Globe (expert) prints Expert and
    /// its name colour is the disco cycle, not the rare-8 yellow.
    #[test]
    fn w4_f28_the_yoyo_placeholder_and_the_expert_row() {
        let _g = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        register_lang();
        let rows = item_tooltip_lines(3262, def(3262), TooltipContext::default());
        let idx = rows.iter().position(|(t, _)| t == " ").expect("the logo row");
        // The logo row sits AFTER the stat rows, BEFORE any Expert row.
        assert!(idx > 1);
        let mut lines = Vec::new();
        let logo = append_second_half(
            3262, def(3262), 0, def(3262).knock_back, false, true,
            HoverFlags::default(), &TooltipWorld::STOCK, None, &mut lines,
        );
        assert_eq!(logo, Some(lines.len() - 1));
        let rows = item_tooltip_lines(1131, def(1131), TooltipContext::default());
        assert!(texts(&rows).contains(&"Expert"), "{:?}", texts(&rows));
        // The expert name colour: the disco stock (255, 0, 0).
        assert_eq!(rows[0].1, [1.0, 0.0, 0.0, 1.0]);
        // The master pulse: (255, (byte)(masterColor * 200), 0).
        let c = tooltip_name_color(-13, false, 0, &TooltipWorld { master_color: 0.5, ..TooltipWorld::STOCK });
        assert_eq!(c, [1.0, 100.0 / 255.0, 0.0, 1.0]);
        // The animator: 1f + 0.05 clamps to 1 and flips the direction.
        assert_eq!(advance_master_color(1.0, 1), (1.0, -1));
        assert_eq!(advance_master_color(0.0, -1), (0.0, 1));
    }

    /// W4-f28 — the favourite + social + tile-wand rows
    /// (`Main.cs:20655-20668`, `:20828-20832`) and the bank-blocked notice
    /// (`ChestUI.cs:1291-1302`).
    #[test]
    fn w4_f28_the_favourite_social_and_wand_rows() {
        let _g = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        register_lang();
        let ctx = TooltipContext {
            flags: HoverFlags { favorited: true, ..Default::default() },
            ..Default::default()
        };
        let rows = item_tooltip_lines(4, def(4), ctx);
        let t = texts(&rows);
        assert!(t.contains(&crate::lang::tip(56).as_str()), "{t:?}");
        assert!(t.contains(&crate::lang::tip(57).as_str()));
        // The Money Trough favourited with its own bank open: the notice.
        let ctx = TooltipContext {
            flags: HoverFlags { favorited: true, ..Default::default() },
            open_chest: -2,
            ..Default::default()
        };
        let rows = item_tooltip_lines(3213, def(3213), ctx);
        assert!(texts(&rows).contains(&"Placing into itself will be blocked"));
        // The Void Lens needs bank4 (−5), not the piggy bank.
        let rows = item_tooltip_lines(4131, def(4131), ctx);
        assert!(!texts(&rows).contains(&"Placing into itself will be blocked"));
        // The social row: a plain helmet in the vanity column prints
        // tip[61]; the Depth Meter (hasVanityEffects) does not.
        let ctx = TooltipContext {
            flags: HoverFlags { social: true, ..Default::default() },
            ..Default::default()
        };
        let rows = item_tooltip_lines(88, def(88), ctx);
        assert!(texts(&rows).contains(&crate::lang::tip(61).as_str()));
        let rows = item_tooltip_lines(18, def(18), ctx);
        assert!(!texts(&rows).contains(&crate::lang::tip(61).as_str()));
        // The Living Wood Wand consumes Wood: tip[52] + the name.
        let rows = item_tooltip_lines(832, def(832), TooltipContext::default());
        let wand = format!("{}{}", crate::lang::tip(52), "Wood");
        assert!(texts(&rows).contains(&wand.as_str()), "{:?}", texts(&rows));
    }

    /// W4-f28 — the place/ammo/consumable ladder's repairs
    /// (`Main.cs:20919-20936`): the Goldfish 261 is PlaceTileOnAltUse (the
    /// place row without consumable — it IS consumable too, but the set is
    /// the gate that matters for the non-consumable members); the Acorn 27
    /// is notAmmo (no ammo row despite ammo 27); the Shadow Scale 86 is
    /// extractinator-only (no plain Consumable row).
    #[test]
    fn w4_f28_the_place_ammo_consumable_ladder_repairs() {
        let _g = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        register_lang();
        let place = crate::lang::tip(33);
        let ammo = crate::lang::tip(34);
        let consumable = crate::lang::tip(35);
        // The Acorn: placeable + notAmmo → the place row, never the ammo row.
        let rows = item_tooltip_lines(27, def(27), TooltipContext::default());
        let t = texts(&rows);
        assert!(t.contains(&place.as_str()), "{t:?}");
        assert!(!t.contains(&ammo.as_str()));
        // The Gel: ammo 23, not placeable → the ammo row, not Consumable.
        let rows = item_tooltip_lines(23, def(23), TooltipContext::default());
        let t = texts(&rows);
        assert!(t.contains(&ammo.as_str()), "{t:?}");
        assert!(!t.contains(&consumable.as_str()));
        // The Shadow Scale: extractinator-only → no Consumable row.
        let rows = item_tooltip_lines(86, def(86), TooltipContext::default());
        assert!(!texts(&rows).contains(&consumable.as_str()), "{:?}", texts(&rows));
        // The Apple: plain consumable → the Consumable row.
        let rows = item_tooltip_lines(4009, def(4009), TooltipContext::default());
        assert!(texts(&rows).contains(&consumable.as_str()));
        // A PlaceTileOnAltUse member that is NOT consumable still places:
        // the gate is the set OR consumable.
        let mut fish = *def(261);
        fish.consumable = false;
        assert!(place_row_active(&fish, 261));
    }

    /// W4-f28 — the damage gate's coin-gun arm (`Main.cs:20669`): the
    /// Copper Coin's damage row appears only while the player carries the
    /// Coin Gun 905 (the 58-slot `HasItem` scan rides `ctx.has_coin_gun`);
    /// the Strange Brew's life row prints the RANGE form
    /// (`:20878-20882`).
    #[test]
    fn w4_f28_the_coin_gun_gate_and_the_range_restore() {
        let _g = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        register_lang();
        let damage = crate::lang::tip(3);
        let rows = item_tooltip_lines(71, def(71), TooltipContext::default());
        assert!(!texts(&rows).iter().any(|t| t.ends_with(damage.as_str())), "no coin gun: no damage row");
        let ctx = TooltipContext { has_coin_gun: true, ..Default::default() };
        let rows = item_tooltip_lines(71, def(71), ctx);
        assert!(texts(&rows).contains(&format!("25{damage}").as_str()), "{:?}", texts(&rows));
        // The Strange Brew: RestoresLifeRange(70, 120).
        let rows = item_tooltip_lines(3001, def(3001), TooltipContext::default());
        assert!(texts(&rows).contains(&"Restores from 70 to 120 life"), "{:?}", texts(&rows));
    }

    /// W4-f28 — the use-time row's whip gate (`Main.cs:20717`): the Slime
    /// Staff (summon, projectile 266 — no whip) prints NO use-time row;
    /// the Fire Whip 4912 (whip 913) does.
    #[test]
    fn w4_f28_the_summon_use_time_row_needs_a_whip() {
        let _g = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        register_lang();
        let tier = crate::lang::tip(9); // useAnimation 28/30 → the <=30 tier
        let rows = item_tooltip_lines(1309, def(1309), TooltipContext::default());
        assert!(!texts(&rows).contains(&tier.as_str()), "{:?}", texts(&rows));
        let rows = item_tooltip_lines(4912, def(4912), TooltipContext::default());
        assert!(texts(&rows).contains(&tier.as_str()), "{:?}", texts(&rows));
    }

    /// W4-f28 — the prefix comparison rows (`Main.cs:21032-21266`): the
    /// Legendary Iron Broadsword's damage/speed/crit/size/knockback deltas
    /// against the fresh defaults, the mana row's INVERTED colour, the
    /// accessory fixed rows and the armor-penetration format.
    #[test]
    fn w4_f28_the_prefix_comparison_rows() {
        let _g = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        register_lang();
        // Legendary 81 on the Iron Broadsword 4: dmg ×1.15 (12→14), spd
        // ×0.9 (20→18), kb ×1.15 (5.5→6.325), size ×1.1, crit +5.
        let ctx = TooltipContext { prefix: 81, ..Default::default() };
        let rows = item_tooltip_lines(4, def(4), ctx);
        let t = texts(&rows);
        // round((14-12)/12*100) = round(16.67) = 17.
        assert!(t.contains(&format!("+17{}", crate::lang::tip(39)).as_str()), "{t:?}");
        // round((18-20)/20*100)*-1 = +10.
        assert!(t.contains(&format!("+10{}", crate::lang::tip(40)).as_str()), "{t:?}");
        assert!(t.contains(&format!("+5{}", crate::lang::tip(41)).as_str()), "{t:?}");
        // round((1.1-1)/1*100) = 10 (the f32 1.1 promotes to 1.10000002…).
        assert!(t.contains(&format!("+10{}", crate::lang::tip(43)).as_str()), "{t:?}");
        // round((6.325-5.5)/5.5*100) = round(15) = 15.
        assert!(t.contains(&format!("+15{}", crate::lang::tip(45)).as_str()), "{t:?}");
        // The colours: the damage row green, a negative delta red.
        let dmg_row = rows.iter().find(|(t, _)| t.starts_with("+17")).unwrap();
        assert_eq!(dmg_row.1, PREFIX_POSITIVE);
        // The Hard 62 accessory prefix: the fixed +1 defense row.
        let ctx = TooltipContext { prefix: 62, ..Default::default() };
        let rows = item_tooltip_lines(18, def(18), ctx);
        let fixed = format!("+1{}", crate::lang::tip(25));
        assert!(texts(&rows).contains(&fixed.as_str()), "{:?}", texts(&rows));
        // The Arcane 66 accessory prefix: "+20 " + tip[31] (the literal
        // carries the trailing space, `:21178`).
        let ctx = TooltipContext { prefix: 66, ..Default::default() };
        let rows = item_tooltip_lines(18, def(18), ctx);
        let fixed = format!("+20 {}", crate::lang::tip(31));
        assert!(texts(&rows).contains(&fixed.as_str()), "{:?}", texts(&rows));
        // The Fabled 85 accessory prefix: +10 armor penetration, the
        // {0:+0;-#} format; the +3 summon tag damage row.
        let ctx = TooltipContext { prefix: 85, ..Default::default() };
        let rows = item_tooltip_lines(18, def(18), ctx);
        assert!(texts(&rows).contains(&"+10 armor penetration"), "{:?}", texts(&rows));
        assert!(texts(&rows).contains(&"+3 summon tag damage"), "{:?}", texts(&rows));
        // The Master row (`:21027`): the master trophy 4924 (rare −13)
        // prints it; the Guarding 63 prefix (×1.1 value ≥ 1.05) shifts the
        // instance rare −13 → −12 (the shift_rare clamp skips ≤ −11) and
        // the row drops. (The ×1.05 tier stays put — `1.05f` is
        // 1.04999995 < 1.05, the `shift_rare` doc's quirk.)
        let rows = item_tooltip_lines(4924, def(4924), TooltipContext::default());
        assert!(texts(&rows).contains(&"Master"), "{:?}", texts(&rows));
        let ctx = TooltipContext { prefix: 63, ..Default::default() };
        let rows = item_tooltip_lines(4924, def(4924), ctx);
        assert!(!texts(&rows).contains(&"Master"), "the prefix shifted rare off −13");
    }

    /// W4-f28 — the rapid-attack damage scale (`Main.cs:20673-20676`,
    /// `Player.cs:18251-18273`): the Falcon Blade's damage row reads
    /// `min(bonus, 0.5)`; the charge caps at 0.62 and decays 0.005/tick.
    #[test]
    fn w4_f28_the_rapid_attack_scale_caps_and_decays() {
        let _g = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        register_lang();
        let world = tr_core::World::new("t".into(), 10, 10);
        let mut p = crate::player::Player::spawn(&world);
        p.rapid_attack_bonus = 0.7; // above the 0.5 read cap
        let ctx = TooltipContext { player: Some(&p), ..Default::default() };
        let rows = item_tooltip_lines(2608, def(2608), ctx);
        // 25 × 1.5 = 37.5 → 37.
        let damage = crate::lang::tip(2);
        assert!(texts(&rows).contains(&format!("37{damage}").as_str()), "{:?}", texts(&rows));
        // The use-site charge: +0.12 capped at 0.62; the damage read uses
        // the PRE-charge bonus (min(0.55, 0.5) = 0.5 → 25 × 1.5 = 37).
        p.rapid_attack_bonus = 0.55;
        let out = p.apply_rapid_attack_bonus(25, 2608, true);
        assert_eq!(out, 37);
        assert!((p.rapid_attack_bonus - 0.62).abs() < 1e-6, "the 0.62 cap");
        // A non-member passes through.
        assert_eq!(p.apply_rapid_attack_bonus(12, 4, true), 12);
    }

    /// W4-f28 — the Revolver's visual crit (`Item.cs:637-645`): the crit
    /// row adds the charging bonus clamped [0, 100]; the set-bonus row
    /// (`Main.cs:21267-21311`) prints the grey single-piece text unworn
    /// and the green complete text worn.
    #[test]
    fn w4_f28_the_visual_crit_and_the_set_bonus_row() {
        let _g = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        register_lang();
        let world = tr_core::World::new("t".into(), 10, 10);
        let mut p = crate::player::Player::spawn(&world);
        p.revolver_crit_chance_bonus = 12;
        p.ranged_crit = 9;
        let ctx = TooltipContext { player: Some(&p), held_crit: 4, ..Default::default() };
        let rows = item_tooltip_lines(2269, def(2269), ctx);
        // 9 − 4 + clamp(0 + 12) = 17.
        let crit = crate::lang::tip(5);
        assert!(texts(&rows).contains(&format!("17{crit}").as_str()), "{:?}", texts(&rows));
        // The clamp: bonus 200 on crit 0 → 100.
        p.revolver_crit_chance_bonus = 200;
        assert_eq!(visual_crit_chance(2269, 0, Some(&p)), 100);
        // The Shadow Greaves 100 unworn: the grey single-piece row.
        let rows = item_tooltip_lines(100, def(100), TooltipContext::default());
        let set_row = rows.iter().find(|(t, _)| t.starts_with("Set Bonus:")).expect("the set row");
        assert_eq!(set_row.1, [130.0 / 255.0, 130.0 / 255.0, 130.0 / 255.0, 190.0 / 255.0]);
        // Worn with the full set: the green (1/1… 3/3) equipped row.
        p.equipped[0] = 102;
        p.equipped[1] = 101;
        p.equipped[2] = 100;
        let ctx = TooltipContext {
            player: Some(&p),
            flags: HoverFlags { worn_armor: true, ..Default::default() },
            ..Default::default()
        };
        let rows = item_tooltip_lines(100, def(100), ctx);
        let set_row = rows.iter().find(|(t, _)| t.starts_with("Set Bonus (3/3)")).expect("the worn row");
        assert_eq!(set_row.1, [50.0 / 255.0, 205.0 / 255.0, 50.0 / 255.0, 1.0]);
    }

    /// W4-f28 — the equipable row's shopping arm (`Main.cs:20818-20825`):
    /// the Discount Card prints tip[60] while a shop is open, tip[23]
    /// otherwise.
    #[test]
    fn w4_f28_the_discount_card_shopping_row() {
        let _g = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        register_lang();
        let rows = item_tooltip_lines(854, def(854), TooltipContext::default());
        assert!(texts(&rows).contains(&crate::lang::tip(23).as_str()));
        let ctx = TooltipContext {
            world: TooltipWorld { npc_shop: true, ..TooltipWorld::STOCK },
            ..Default::default()
        };
        let rows = item_tooltip_lines(854, def(854), ctx);
        let t = texts(&rows);
        assert!(t.contains(&crate::lang::tip(60).as_str()), "{t:?}");
        assert!(!t.contains(&crate::lang::tip(23).as_str()));
    }

    /// W4-f28 — the knockback display chain (`Main.cs:20333-20350` +
    /// `:20753-20765`): the quiver ×1.1 INT truncation, the Psycho Knife
    /// self-hold stealth arm and the gloves' melee scale — and the prefix
    /// row's oldKB is the UNSCALED instance knockback.
    #[test]
    fn w4_f28_the_knockback_chain_truncates_and_scales() {
        let _g = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        register_lang();
        let world = tr_core::World::new("t".into(), 10, 10);
        let mut p = crate::player::Player::spawn(&world);
        // The stake ×1.1 truncation: 4.25 × 1.1 = 4.675 → 4 (int).
        let mut scimitar = *def(3349);
        scimitar.use_ammo = crate::ammo::AMMO_STAKE;
        let kb = display_knockback(3349, &scimitar, 4.25, Some(&p), 0);
        assert_eq!(kb, 4.0);
        // The Psycho Knife holding itself at stealth 0.5: kb × 1.5.
        p.stealth = 0.5;
        let knife = def(3106);
        let kb = display_knockback(3106, knife, knife.knock_back, Some(&p), 3106);
        assert_eq!(kb, knife.knock_back * 1.5);
        // Not holding it: no stealth arm.
        let kb = display_knockback(3106, knife, knife.knock_back, Some(&p), 4);
        assert_eq!(kb, knife.knock_back);
        // The gloves: melee kb ×2.
        p.kb_glove = true;
        let kb = display_knockback(4, def(4), 5.5, Some(&p), 0);
        assert_eq!(kb, 11.0);
    }

    /// W4-f28 salvage (research/128 §5-e16.1) — the fishing rows
    /// (`Main.cs:20804-20815`): the power row reads the vanilla's OWN typo'd
    /// key `GameUI.PrecentFishingPower` (`:20806`); the base-era
    /// `RecentFishingPower` spelling resolved nothing. The Wood Fishing Pole
    /// 2289 (power 5) prints "5% fishing power" then the argument-less
    /// BaitRequired row (`:20808`).
    #[test]
    fn w4_f28_the_fishing_rows_read_the_vanilla_typo_key() {
        let _g = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        register_lang();
        let rows = item_tooltip_lines(2289, def(2289), TooltipContext::default());
        let t = texts(&rows);
        assert!(t.contains(&"5% fishing power"), "{t:?}");
        assert!(t.contains(&"Requires bait to catch fish"), "{t:?}");
        // The wrong key must not be read anywhere on this path.
        assert!(!t.contains(&"RecentFishingPower"));
    }

    /// W4-f28 salvage (research/128 §3, e16
    /// `w4_e16_the_quiver_knockback_truncates_before_the_word_tier`):
    /// the stat application runs over the CALLER's row — a witness-
    /// overridden knockback 9.5 with the quiver's ×1.1 truncates to
    /// (int)10.45 = 10 → the <=11 word tip[21]. The registry re-read
    /// (`prefixed_stats(kind, …)`) could not see the override.
    #[test]
    fn w4_f28_the_caller_row_seam_lets_an_overridden_knockback_reach_the_word_tier() {
        let _g = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        register_lang();
        let mut bow = *def(1);
        bow.knock_back = 9.5;
        bow.use_ammo = crate::ammo::AMMO_ARROW;
        let world = tr_core::World::new("t".into(), 10, 10);
        let mut p = crate::player::Player::spawn(&world);
        p.magic_quiver = true;
        let ctx = TooltipContext { player: Some(&p), ..Default::default() };
        let rows = item_tooltip_lines(1, &bow, ctx);
        let t = texts(&rows);
        assert!(t.contains(&crate::lang::tip(21).as_str()), "{t:?}");
        assert!(!t.contains(&crate::lang::tip(22).as_str()));
        // The seam itself: the overridden row reaches prefixed_stats_of.
        let ps = hover_stats(&bow, 0);
        assert_eq!(ps.knock_back, 9.5);
    }

    /// W4-f28 salvage (research/128 §5-e16.3, e16
    /// `w4_e16_the_knockback_delta_reads_the_prefix_only_oldkb`): oldKB is
    /// captured BEFORE the glove scale (`Main.cs:20333`) — the Legendary
    /// Iron Broadsword's knockback DELTA row stays prefix-only (+15%) even
    /// with the glove, while the WORD tier reads the glove-scaled
    /// 5.5 × 1.15 × 2 = 12.65 → the insane word tip[22].
    #[test]
    fn w4_f28_the_knockback_delta_reads_the_prefix_only_oldkb() {
        let _g = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        register_lang();
        let world = tr_core::World::new("t".into(), 10, 10);
        let mut p = crate::player::Player::spawn(&world);
        p.kb_glove = true;
        let ctx = TooltipContext { prefix: 81, player: Some(&p), ..Default::default() };
        let rows = item_tooltip_lines(4, def(4), ctx);
        let t = texts(&rows);
        // oldKB = 5.5 * 1.15 = 6.325; (6.325 - 5.5) / 5.5 * 100 = 15.
        let kb_delta = format!("+15{}", crate::lang::tip(45));
        assert!(t.contains(&kb_delta.as_str()), "{t:?}");
        let glove_leak = format!("+130{}", crate::lang::tip(45));
        assert!(!t.contains(&glove_leak.as_str()), "the glove scale must not leak: {t:?}");
        // The word tier reads the glove-scaled 12.65 → tip[22].
        assert!(t.contains(&crate::lang::tip(22).as_str()), "{t:?}");
    }

    /// W4-f28 salvage (research/128 §5-e16.4, e16
    /// `w4_e16_the_loadout_share_arm_suppresses_the_equipable_row`): a
    /// shareable context (8-12, `ItemSlot.cs:395-399`) latches `flag` — the
    /// favourited Meteor Helmet prints UI.ItemLoadoutShared and the
    /// equipable row drops (`Main.cs:20816`'s `!flag`); the unfavourited
    /// piece latches flag2 (the share hint at the tail, `:21312-21317`,
    /// OurFavoriteColor (255,231,69) at alpha 190, `Main.cs:66748`).
    #[test]
    fn w4_f28_the_loadout_share_arm_suppresses_the_equipable_row() {
        let _g = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        register_lang();
        let flags = HoverFlags { context: 8, favorited: true, worn_armor: true, ..Default::default() };
        let ctx = TooltipContext { flags, ..Default::default() };
        let rows = item_tooltip_lines(123, def(123), ctx);
        let t = texts(&rows);
        assert!(t.contains(&"Shared with other loadouts"), "{t:?}");
        assert!(!t.contains(&crate::lang::tip(23).as_str()), "the equipable row drops");
        assert!(!t.contains(&crate::lang::tip(56).as_str()), "no plain favourite pair");
        // The unfavourited shareable slot: the hint row in OurFavoriteColor.
        let flags = HoverFlags { context: 8, worn_armor: true, ..Default::default() };
        let ctx = TooltipContext { flags, ..Default::default() };
        let rows = item_tooltip_lines(123, def(123), ctx);
        let hint = rows
            .iter()
            .find(|(t, _)| t == "{InputTriggerUI_FavoriteItem} to share across loadouts")
            .expect("the share hint");
        assert_eq!(hint.1, [1.0, 231.0 / 255.0, 69.0 / 255.0, 190.0 / 255.0]);
        // The setting off drops the hint (Player.cs:372/:412).
        let ctx = TooltipContext { flags, show_loadout_share_hint: false, ..Default::default() };
        let rows = item_tooltip_lines(123, def(123), ctx);
        assert!(!texts(&rows).iter().any(|t| t.contains("share across loadouts")));
        // canLoadoutShareAt: exactly 8..=12.
        for c in 8..=12 {
            assert!(can_loadout_share_at(c));
        }
        assert!(!can_loadout_share_at(7));
        assert!(!can_loadout_share_at(13));
        assert!(!can_loadout_share_at(-1));
    }

    /// W4-f28 — the diff 1/2 name colours (`GetItemNameColor`,
    /// `Main.cs:20620-20627`): the mediumcore `mcColor` (1, 0.6, 0) and the
    /// hardcore `hcColor` (1, 0.15, 0.1) (`Main.cs:66765-66766`) override
    /// EVERY rarity colour, the expert disco included. The item tooltip's
    /// own call passes 0; the stealthed-remote-player hover passes the
    /// character's difficulty (`:42934`). WRITTEN NOT RUN (code-only mode).
    #[test]
    fn w4_f28_the_diff_colours_override_every_rarity() {
        let _g = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        register_lang();
        // diff 0 keeps the rarity colour (the white rare-0 name).
        let c = tooltip_name_color(0, false, 0, &TooltipWorld::STOCK);
        assert_eq!(c, [1.0, 1.0, 1.0, 1.0]);
        // diff 1: the mediumcore orange, over the rare-8 yellow AND the
        // expert disco (the override runs after every rarity arm).
        let c = tooltip_name_color(8, false, 1, &TooltipWorld::STOCK);
        assert_eq!(c, [1.0, 0.6, 0.0, 1.0]);
        let c = tooltip_name_color(0, true, 1, &TooltipWorld::STOCK);
        assert_eq!(c, [1.0, 0.6, 0.0, 1.0]);
        // diff 2: the hardcore red, over the master pulse too.
        let c = tooltip_name_color(-13, false, 2, &TooltipWorld { master_color: 0.5, ..TooltipWorld::STOCK });
        assert_eq!(c, [1.0, 0.15, 0.1, 1.0]);
        // The tooltip rows carry it through the context.
        let ctx = TooltipContext { diff: 2, ..Default::default() };
        let rows = item_tooltip_lines(4, def(4), ctx);
        assert_eq!(rows[0].1, [1.0, 0.15, 0.1, 1.0]);
        // Any other diff value keeps the rarity colour (the C# has only
        // the two arms).
        let c = tooltip_name_color(0, false, 3, &TooltipWorld::STOCK);
        assert_eq!(c, [1.0, 1.0, 1.0, 1.0]);
    }
}