//! Hotbar selection state + smart-select auto-tool + the quick-heal chooser —
//! the `Player.selectedItemState` machine (`Player.cs:414-568`), the
//! `SmartSelect*` family (`Player.cs:17410-17716`) and
//! `QuickHeal_GetItemToUse` (`Player.cs:5549-5596`).
//!
//! The port keeps the live slot in `App.selected` (every reader stays put);
//! this module owns the C#'s *buffered/overridden* halves: a switch request
//! buffers until the player is idle (`CanChangeSelectedItemImmediately`,
//! `Player.cs:428-438`), and only then does `Update` apply it and reset the
//! use timers (`OnSelectionChanged`, `:551-567`).

use crate::inv::{AMMO, COINS};
use crate::item::{ItemSlot, INV};
use crate::registry;

/// `Player.ClampHotbarOffset` (`Player.cs:31895-31906`): the wrap into
/// `0..=9` — the C#'s two `while` loops, `rem_euclid` for any offset.
pub(crate) fn clamp_hotbar_offset(mut offset: i32) -> usize {
    while offset > 9 {
        offset -= 10;
    }
    while offset < 0 {
        offset += 10;
    }
    offset as usize
}

/// `Player.SelectedItemState` (`Player.cs:414-568`) minus the live
/// `selected` field (the port's `App.selected` is that field). `buffered`
/// and `overridden` are the C#'s `-1`-sentinel ints.
#[derive(Clone, Copy, Debug)]
pub(crate) struct SelectionState {
    /// `hotbar` (`Player.cs:422`): the last hotbar-range selection — the
    /// wheel cycles from HERE, not from the (possibly overridden) live slot
    /// (`HandleHotbarControls`, `:31794`).
    pub hotbar: usize,
    /// `buffered` (`Player.cs:424`): the pending explicit switch, -1 = none.
    pub buffered: i32,
    /// `overridden` (`Player.cs:426`): the slot to restore after an
    /// override (smart-select / mouse item), -1 = none.
    pub overridden: i32,
}

impl SelectionState {
    pub(crate) fn new() -> Self {
        Self {
            hotbar: 0,
            buffered: -1,
            overridden: -1,
        }
    }

    /// `HasActiveOverride` (`Player.cs:444`).
    pub(crate) fn has_active_override(&self) -> bool {
        self.overridden >= 0
    }

    /// `HasBufferedChange` (`Player.cs:446`).
    pub(crate) fn has_buffered_change(&self) -> bool {
        self.buffered >= 0
    }

    /// `Select` (`Player.cs:486-518`), local-player half. Returns whether
    /// the tick cue plays (`SoundEngine.PlaySound(12)`, `:506` — suppressed
    /// when the request lands on the override/buffer slot). The remote
    /// half (`:492-495`, immediate) is the net glue's own write; the
    /// `afkCounter` resets (`:515-516`) have no port field.
    pub(crate) fn select(&mut self, selected: usize, item: usize, item_is_air: bool) -> bool {
        if item < 10 {
            self.hotbar = item;
        }
        // `:498-501`: an air slot at 10+ is refused outright.
        if item >= 10 && item_is_air {
            return false;
        }
        if selected != item {
            let play = item as i32 != self.overridden && item as i32 != self.buffered;
            self.buffered = item as i32;
            play
        } else {
            // `:510-514`: re-selecting the live slot cancels both halves.
            self.buffered = -1;
            self.overridden = -1;
            false
        }
    }

    /// `Update` (`Player.cs:520-549`) + `OverrideSelection` (`:464-484`).
    /// `can_change` is `CanChangeSelectedItemImmediately` (`:428-438`:
    /// `!UsingOrReusingItem && ItemTimeIsZero` — the caller folds
    /// `itemAnimation > 0 || reuseDelay > 0 || channel` into the first half;
    /// `pendingItemReuse` has no port field). `inv` backs the `:536`
    /// `HeldItem.IsAir` read — the C# reads the NEW selection's slot.
    /// Returns `(new_selected, continue_auto_reuse_with_new_item)` when the
    /// live slot changed — the caller applies `OnSelectionChanged`
    /// (`:551-567`).
    pub(crate) fn update(
        &mut self,
        selected: usize,
        can_change: bool,
        inv: &[ItemSlot; INV],
        override_slot: Option<usize>,
    ) -> Option<(usize, bool)> {
        if !can_change {
            return None;
        }
        let old = selected;
        let had_buffered = self.has_buffered_change();
        let mut sel = selected;
        if had_buffered {
            sel = self.buffered as usize;
        } else if self.has_active_override() {
            sel = self.overridden as usize;
        }
        self.buffered = -1;
        self.overridden = -1;
        // `:536-539`: an air hold at 10+ falls back to the hotbar slot.
        if sel >= 10 && inv.get(sel).is_some_and(|s| s.count == 0) {
            sel = self.hotbar;
        }
        // `:540-543` -> `OverrideSelection` (`:464-484`): the override
        // re-arms `overridden` with the pre-override slot (only the first
        // time), and landing back on it clears the override.
        if let Some(item) = override_slot {
            if item != sel {
                if self.overridden == -1 {
                    self.overridden = old as i32;
                }
                if item as i32 == self.overridden {
                    self.overridden = -1;
                }
                sel = item;
            }
        }
        if sel != old {
            // `:544-547`: `OnSelectionChanged(hasBufferedChange &&
            // !HasActiveOverride)` — the override state is read AFTER the
            // override arm above.
            Some((sel, had_buffered && !self.has_active_override()))
        } else {
            None
        }
    }
}

// ---------------------------------------------------------------------------
// Smart-select (`Player.controlTorch`, the LeftShift auto-tool)
// ---------------------------------------------------------------------------

/// `ItemID.Sets.Glowsticks` (`ItemID.cs:1374`) — no core set exists yet,
/// so the literal lives here (the torch/water-torch/scraper sets ride
/// `tr_core::smart_cursor`, the extractinator modes `tr_core::extract`).
const GLOWSTICKS: &[u16] = &[282, 286, 3112, 3002, 4776, 5643];

/// The C# `inventory[i]` index space over the port's split arrays: 0..49 the
/// grid, 50..53 the coins, 54..57 the ammo (`Player.cs:1576`, the port's
/// `inv::coin_slot_index`/`ammo_slot_index` numbering).
fn carry_slot<'a>(
    inv: &'a [ItemSlot; INV],
    coins: &'a [ItemSlot; COINS],
    ammo: &'a [ItemSlot; AMMO],
    i: usize,
) -> &'a ItemSlot {
    if i < INV {
        &inv[i]
    } else if i < INV + COINS {
        &coins[i - INV]
    } else {
        &ammo[i - INV - COINS]
    }
}

/// The smart-select inputs — the C# reads `Main.tile`, the mouse world
/// point, the player box and `Player.wet` off globals; the port hands them
/// in (the `world_sim/player.rs` seam).
pub(crate) struct SmartSelectCtx<'a> {
    pub world: &'a tr_core::World,
    pub inv: &'a [ItemSlot; INV],
    pub coins: &'a [ItemSlot; COINS],
    pub ammo: &'a [ItemSlot; AMMO],
    /// `Main.mouseX + Main.screenPosition` — the mouse WORLD point.
    pub mouse: (f32, f32),
    /// `base.position` + the box height (`base.width` is the fixed
    /// `player::WIDTH`).
    pub player_pos: (f32, f32),
    pub player_h: f32,
    /// `base.wet`.
    pub wet: bool,
}

impl SmartSelectCtx<'_> {
    /// `IsInTileInteractionRange(tX, tY, TileReachCheckSettings.Simple, TB)`
    /// (`Player.cs:32025-32029`): the Simple reach rect grown by the tool's
    /// `tileBoost` (`tr_core::smart_cursor::reachable_rect` is the same
    /// `GetTileRegion` port), then `Collision.InTileBounds`.
    fn in_range(&self, tx: i32, ty: i32, tile_boost: i32) -> bool {
        let (lx, ly, hx, hy) = tr_core::smart_cursor::reachable_rect(
            self.player_pos,
            crate::player::WIDTH as i32,
            self.player_h as i32,
            tile_boost,
        );
        tx >= lx && tx <= hx && ty >= ly && ty <= hy
    }

    /// `SmartSelect_GetAvailableToolRanges` (`Player.cs:17681-17716`): the
    /// FIRST item of each tool family pins that family's range at its
    /// `tileBoost`; -10 (no tool) shrinks the rect into nothing.
    fn tool_ranges(&self) -> (i32, i32, i32, i32, i32, i32) {
        let (mut pick, mut axe, mut hammer, mut cannon, mut extract, mut scraper) =
            (-10, -10, -10, -10, -10, -10);
        for slot in self.inv.iter() {
            let Some(d) = registry::get(slot.kind) else {
                continue;
            };
            if d.pick > 0 && pick == -10 {
                pick = d.tile_boost;
            }
            if d.axe > 0 && axe == -10 {
                axe = d.tile_boost;
            }
            if d.hammer > 0 && hammer == -10 {
                hammer = d.tile_boost;
            }
            if tr_core::smart_cursor::TORCH_ITEMS.contains(&slot.kind) && cannon == -10 {
                cannon = d.tile_boost;
            }
            if tr_core::smart_cursor::PAINT_SCRAPER_ITEMS.contains(&slot.kind) && scraper == -10 {
                scraper = d.tile_boost;
            }
            if tr_core::extract::extractinator_mode(slot.kind) != -1 && extract == -10 {
                extract = d.tile_boost;
            }
        }
        (pick, axe, hammer, cannon, extract, scraper)
    }

    /// `SmartSelect_GetToolStrategy` (`Player.cs:17623-17679`): the context
    /// strategy + the wet-tile flag. The C#'s `try/catch` swallows
    /// out-of-world reads; the port's `World::tile` answers `None` (the
    /// same fall-through to the torch strategy).
    fn tool_strategy(&self, tx: i32, ty: i32) -> (i32, bool) {
        let mut strategy = 0;
        let mut found = false;
        let wet_tile = self
            .world
            .tile(tx, ty)
            .is_some_and(|t| t.liquid > 0);
        let (pick, axe, hammer, cannon, extract, scraper) = self.tool_ranges();
        if let Some(t) = self.world.tile(tx, ty) {
            if t.active() {
                let kind = t.kind;
                // `:17635-17639`: the Extractinator (219) / Chlorophyte
                // Extractinator (642). The `ItemTrader` (shimmer) half of
                // strategy 7 (`:17595-17600`) has no port — the
                // extractinator-item arm below is the whole pick.
                if (kind == 219 || kind == 642) && self.in_range(tx, ty, extract) {
                    strategy = 7;
                    found = true;
                } else if kind == 209
                    && t.frame_x / 72 < 3
                    && self.in_range(tx, ty, cannon)
                {
                    // `:17640-17644`: the Snowball Cannon's first 3 frames.
                    strategy = 6;
                    found = true;
                } else if tr_core::tileobject::tile_hammer(kind)
                    && self.in_range(tx, ty, hammer)
                {
                    strategy = 1;
                    found = true;
                } else if tr_core::tileobject::tile_axe(kind) && self.in_range(tx, ty, axe) {
                    strategy = 2;
                    found = true;
                } else if kind == 184 && self.in_range(tx, ty, scraper) {
                    // `:17655-17659`: the Alchemy Table takes the
                    // paint-scraper strategy (the C#'s own odd arm).
                    strategy = 8;
                    found = true;
                } else if self.in_range(tx, ty, pick) {
                    strategy = 3;
                    found = true;
                }
            } else if wet_tile && self.wet {
                strategy = 4;
                found = true;
            }
        }
        // `:17675-17678`: no tool matched but the player is wet → the
        // underwater light strategy regardless of the tile.
        if !found && self.wet {
            strategy = 4;
        }
        (strategy, wet_tile)
    }

    /// `SmartSelect_PickToolForStrategy` (`Player.cs:17472-17621`): the
    /// first matching slot in 0..49 wins; the torch arms keep scanning
    /// (a later torch beats an earlier glowstick, `:17480-17490`).
    fn pick_tool(&self, tx: i32, ty: i32, strategy: i32, wet_tile: bool, selected: usize) -> usize {
        let mut result = selected;
        let torch = |k: u16| tr_core::smart_cursor::TORCH_ITEMS.contains(&k);
        let glow = |k: u16| GLOWSTICKS.contains(&k);
        let water_torch = |k: u16| tr_core::smart_cursor::WATER_TORCH_ITEMS.contains(&k);
        // The flare gun's ammo probe (`:17528-17535`, `:17563-17570`):
        // any carried slot 57..0 whose `ammo` matches the flare's `useAmmo`.
        let has_flare_ammo = |use_ammo: i32| {
            (0..INV + COINS + AMMO).rev().any(|j| {
                let s = carry_slot(self.inv, self.coins, self.ammo, j);
                s.count > 0 && registry::get(s.kind).is_some_and(|d| d.ammo == use_ammo)
            })
        };
        let flare_ammo = registry::get(930).map_or(0, |d| d.use_ammo);
        for i in 0..INV {
            let slot = &self.inv[i];
            let kind = slot.kind;
            match strategy {
                0 => {
                    if torch(kind) {
                        result = i;
                        break;
                    }
                    if glow(kind) {
                        result = i;
                    }
                    continue;
                }
                1 => {
                    if registry::get(kind).is_some_and(|d| d.hammer > 0) {
                        result = i;
                        break;
                    }
                    continue;
                }
                2 => {
                    if registry::get(kind).is_some_and(|d| d.axe > 0) {
                        result = i;
                        break;
                    }
                    continue;
                }
                3 => {
                    if registry::get(kind).is_some_and(|d| d.pick > 0) {
                        result = i;
                        break;
                    }
                    continue;
                }
                4 => {
                    // `:17512-17550`: dry torches only upgrade a non-torch
                    // hold; wet tiles take glowsticks / flares / water
                    // torches.
                    if !glow(kind) && kind != 930 && torch(kind) && !water_torch(kind) {
                        if !torch(self.inv[selected].kind) {
                            result = i;
                        }
                        continue;
                    }
                    if glow(kind) && wet_tile {
                        result = i;
                    } else if kind == 930 && wet_tile {
                        if !has_flare_ammo(flare_ammo) {
                            continue;
                        }
                        result = i;
                    } else {
                        if !water_torch(kind) {
                            continue;
                        }
                        result = i;
                    }
                    break;
                }
                5 => {
                    // `:17551-17585`: the far-cursor light strategy — the
                    // torch upgrade arm, then the flare (ammo-gated), then
                    // glowsticks.
                    if torch(kind) {
                        if !torch(self.inv[selected].kind) {
                            result = i;
                        }
                        continue;
                    }
                    if kind == 930 {
                        if !has_flare_ammo(flare_ammo) {
                            continue;
                        }
                        result = i;
                    } else {
                        if !glow(kind) {
                            continue;
                        }
                        result = i;
                    }
                    break;
                }
                6 => {
                    if torch(kind) {
                        result = i;
                        break;
                    }
                    continue;
                }
                7 => {
                    // `Player.cs:17593-17607` (W4-f47): the ItemTrader arm
                    // first — on the Chlorophyte Extractinator (642) a slot
                    // the trade table accepts wins; else the ExtractinatorMode
                    // arm (`:17601-17605`).
                    let trader_tile = self
                        .world
                        .tile(tx, ty)
                        .is_some_and(|t| t.active() && t.kind == 642);
                    if trader_tile
                        && slot.count > 0
                        && tr_core::extract::chlorophyte_trade(kind, slot.count).is_some()
                    {
                        result = i;
                        break;
                    }
                    if tr_core::extract::extractinator_mode(kind) >= 0 {
                        result = i;
                        break;
                    }
                    continue;
                }
                8 => {
                    if tr_core::smart_cursor::PAINT_SCRAPER_ITEMS.contains(&kind) {
                        result = i;
                        break;
                    }
                    continue;
                }
                _ => continue,
            }
        }
        result
    }
}

/// `SmartSelectLookup` (`Player.cs:17434-17460`) + the
/// `SmartSelectLookup_GetTargetTile` mouse-tile cast (`:17462-17470`, the
/// `(int)` truncation; reverse gravity has no port field). `control_torch`
/// is the caller's `Player.controlTorch` gate (`:17436-17441` — the false
/// arm also resets `_lastSmartCursorToolStrategy`, a gamepad-only field
/// the port does not carry). Returns the override slot.
pub(crate) fn smart_select_lookup(
    ctx: &SmartSelectCtx<'_>,
    control_torch: bool,
    selected: usize,
) -> Option<usize> {
    if !control_torch {
        return None;
    }
    let tx = (ctx.mouse.0 / 16.0) as i32;
    let ty = (ctx.mouse.1 / 16.0) as i32;
    let (mut strategy, wet_tile) = ctx.tool_strategy(tx, ty);
    // `:17448-17456`: the light strategies flip to the far-cursor (5)
    // when the mouse is over 200 px from the player centre (the Y leg
    // weighted ×1.3).
    if strategy == 0 || strategy == 4 {
        let cx = ctx.player_pos.0 + crate::player::WIDTH * 0.5;
        let cy = ctx.player_pos.1 + ctx.player_h * 0.5;
        let dx = (ctx.mouse.0 - cx).abs();
        let dy = (ctx.mouse.1 - cy).abs() * 1.3;
        if (dx * dx + dy * dy).sqrt() > 200.0 {
            strategy = 5;
        }
    }
    Some(ctx.pick_tool(tx, ty, strategy, wet_tile, selected))
}

// ---------------------------------------------------------------------------
// Quick-heal chooser — RETIRED (gate V4 keep-one reconciliation): the
// canonical `QuickHeal_GetItemToUse`/`TryChoosingItem` port lives in
// `potions.rs` (S1-P-i37, `quick_heal_get_item_to_use`/`quick_heal_try_choosing`
// over `QuickSlot`); the i54 inline copy + its `HealPick` index space are
// gone. The chooser witnesses live in `potions.rs::tests`
// (`quick_heal_chooser_*`) and the H-edge end-to-end witnesses in
// `world_sim/player.rs` (`s1_p_i54_quick_heal_*`).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::EMPTY_SLOT;

    fn slot(kind: u16, count: u32) -> ItemSlot {
        ItemSlot {
            kind,
            name: "",
            count,
            prefix: 0,
        }
    }

    // ---------------- ClampHotbarOffset ----------------

    #[test]
    fn s1_p_i54_clamp_hotbar_offset_wraps_both_ends() {
        // `Player.cs:31895-31906` — the two while loops.
        assert_eq!(clamp_hotbar_offset(0), 0);
        assert_eq!(clamp_hotbar_offset(9), 9);
        assert_eq!(clamp_hotbar_offset(10), 0);
        assert_eq!(clamp_hotbar_offset(-1), 9);
        assert_eq!(clamp_hotbar_offset(-11), 9);
        assert_eq!(clamp_hotbar_offset(23), 3);
    }

    // ---------------- Select / Update ----------------

    #[test]
    fn s1_p_i54_select_buffers_and_plays_the_tick_once() {
        // `Player.cs:486-518`: the request buffers (no live change), the
        // 12 cue plays; a repeat request onto the same buffer is silent.
        let mut s = SelectionState::new();
        assert!(s.select(0, 3, false), "the change request chimes");
        assert_eq!(s.buffered, 3);
        assert_eq!(s.hotbar, 3);
        assert!(!s.select(0, 3, false), "the buffered repeat is silent");
    }

    #[test]
    fn s1_p_i54_reselecting_the_live_slot_cancels_both_halves() {
        // `Player.cs:510-514`.
        let mut s = SelectionState::new();
        s.select(0, 3, false);
        s.overridden = 5;
        assert!(!s.select(0, 0, false));
        assert_eq!(s.buffered, -1);
        assert_eq!(s.overridden, -1);
    }

    #[test]
    fn s1_p_i54_an_air_slot_at_ten_plus_is_refused() {
        // `Player.cs:498-501` — the hotbar range never reads the flag.
        let mut s = SelectionState::new();
        assert!(!s.select(0, 12, true));
        assert_eq!(s.buffered, -1);
        assert!(s.select(0, 12, false));
        assert_eq!(s.buffered, 12);
    }

    #[test]
    fn s1_p_i54_update_defers_while_using_or_reusing() {
        // `Player.cs:522` — `CanChangeSelectedItemImmediately` false keeps
        // the buffer armed and the live slot untouched.
        let inv = [EMPTY_SLOT; INV];
        let mut s = SelectionState::new();
        s.select(0, 4, false);
        assert_eq!(s.update(0, false, &inv, None), None);
        assert_eq!(s.buffered, 4, "the request stays armed");
        // Idle: the buffer applies, `continueAutoReuseWithNewItem` true
        // (`:546` — buffered && !override).
        assert_eq!(s.update(0, true, &inv, None), Some((4, true)));
        assert_eq!(s.buffered, -1);
    }

    #[test]
    fn s1_p_i54_the_override_restores_and_rearms() {
        // `Player.cs:464-484` + `:530-533`: an override stores the old
        // slot; releasing it (no override this tick) restores; landing the
        // override back on the stored slot clears it.
        let inv = [EMPTY_SLOT; INV];
        let mut s = SelectionState::new();
        // smart-select picks slot 7 while slot 2 is live.
        assert_eq!(s.update(2, true, &inv, Some(7)), Some((7, false)));
        assert_eq!(s.overridden, 2);
        // Next idle tick, no override: restore slot 2.
        assert_eq!(s.update(7, true, &inv, None), Some((2, false)));
        assert_eq!(s.overridden, -1);
        // Override again, then the player explicitly selects the stored
        // slot: the override clears without a restore bounce.
        assert_eq!(s.update(2, true, &inv, Some(7)), Some((7, false)));
        assert_eq!(s.overridden, 2);
        assert_eq!(s.update(7, true, &inv, Some(2)), Some((2, false)));
        assert_eq!(s.overridden, -1);
    }

    #[test]
    fn s1_p_i54_a_buffered_change_beats_the_override_restore() {
        // `Player.cs:526-533`: buffered first, override only as the else.
        let inv = [EMPTY_SLOT; INV];
        let mut s = SelectionState::new();
        s.overridden = 2;
        s.select(7, 5, false);
        assert_eq!(s.update(7, true, &inv, None), Some((5, true)));
    }

    #[test]
    fn s1_p_i54_an_air_hold_at_ten_plus_falls_back_to_the_hotbar() {
        // `Player.cs:536-539`: the buffered slot 12 is air at apply time —
        // the selection falls back to the hotbar latch, not the old slot.
        let mut inv = [EMPTY_SLOT; INV];
        inv[12] = slot(9, 5);
        let mut s = SelectionState::new();
        s.select(0, 12, false);
        // The stack vanished between the request and the apply.
        inv[12] = EMPTY_SLOT;
        s.hotbar = 3;
        assert_eq!(s.update(0, true, &inv, None), Some((3, true)));
    }

    // ---------------- smart-select ----------------

    fn ctx<'a>(
        world: &'a tr_core::World,
        inv: &'a [ItemSlot; INV],
        coins: &'a [ItemSlot; COINS],
        ammo: &'a [ItemSlot; AMMO],
        mouse: (f32, f32),
        wet: bool,
    ) -> SmartSelectCtx<'a> {
        SmartSelectCtx {
            world,
            inv,
            coins,
            ammo,
            mouse,
            // The player box sits at tile (10, 10) — the mouse tiles below
            // are inside the Simple reach (X 5 / Y 3 around the box).
            player_pos: (160.0, 160.0),
            player_h: 42.0,
            wet,
        }
    }

    fn empty_carry() -> ([ItemSlot; COINS], [ItemSlot; AMMO]) {
        ([EMPTY_SLOT; COINS], [EMPTY_SLOT; AMMO])
    }

    #[test]
    fn s1_p_i54_smart_select_needs_the_torch_key() {
        // `Player.cs:17436-17441`.
        let w = tr_core::World::new("ss".into(), 40, 40);
        let inv = [EMPTY_SLOT; INV];
        let (coins, ammo) = empty_carry();
        let c = ctx(&w, &inv, &coins, &ammo, (176.0, 176.0), false);
        assert_eq!(smart_select_lookup(&c, false, 0), None);
    }

    #[test]
    fn s1_p_i54_smart_select_picks_the_pickaxe_for_a_solid_tile() {
        // `Player.cs:17660-17664` (strategy 3) + `:17505-17511`.
        let mut w = tr_core::World::new("ss".into(), 40, 40);
        let i = w.index(11, 10);
        w.tiles[i] = tr_core::tile::Tile {
            kind: 1, // stone
            ..Default::default()
        };
        w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        let mut inv = [EMPTY_SLOT; INV];
        inv[2] = slot(8, 10); // a torch first — the pickaxe still wins
        inv[5] = slot(1, 1); // the Iron Pickaxe
        let (coins, ammo) = empty_carry();
        let c = ctx(&w, &inv, &coins, &ammo, (11.0 * 16.0 + 8.0, 10.0 * 16.0 + 8.0), false);
        assert_eq!(smart_select_lookup(&c, true, 0), Some(5));
    }

    #[test]
    fn s1_p_i54_smart_select_picks_the_hammer_for_an_altar() {
        // `Player.cs:17645-17649` (tileHammer, strategy 1) + `:17491-17497`.
        let mut w = tr_core::World::new("ss".into(), 40, 40);
        let i = w.index(11, 10);
        w.tiles[i] = tr_core::tile::Tile {
            kind: 26, // the altar — TILE_HAMMER
            ..Default::default()
        };
        w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        let mut inv = [EMPTY_SLOT; INV];
        inv[1] = slot(1, 1); // the pickaxe comes first but cannot pound
        inv[4] = slot(7, 1); // the Iron Hammer
        let (coins, ammo) = empty_carry();
        let c = ctx(&w, &inv, &coins, &ammo, (11.0 * 16.0 + 8.0, 10.0 * 16.0 + 8.0), false);
        assert_eq!(smart_select_lookup(&c, true, 0), Some(4));
    }

    #[test]
    fn s1_p_i54_smart_select_picks_the_axe_for_a_tree() {
        // `Player.cs:17650-17654` (tileAxe, strategy 2) + `:17498-17504`.
        let mut w = tr_core::World::new("ss".into(), 40, 40);
        let i = w.index(11, 10);
        w.tiles[i] = tr_core::tile::Tile {
            kind: 5, // the tree — tileAxe (`Main.cs:7208-7223`)
            ..Default::default()
        };
        w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        let mut inv = [EMPTY_SLOT; INV];
        inv[3] = slot(10, 1); // the Iron Axe (axe 9)
        let (coins, ammo) = empty_carry();
        let c = ctx(&w, &inv, &coins, &ammo, (11.0 * 16.0 + 8.0, 10.0 * 16.0 + 8.0), false);
        assert_eq!(smart_select_lookup(&c, true, 0), Some(3));
    }

    #[test]
    fn s1_p_i54_smart_select_empty_tile_takes_the_torch() {
        // `Player.cs:17625` (strategy 0 default) + `:17480-17490`: the
        // first torch wins over an earlier glowstick.
        let w = tr_core::World::new("ss".into(), 40, 40);
        let mut inv = [EMPTY_SLOT; INV];
        inv[1] = slot(282, 5); // the glowstick
        inv[6] = slot(8, 10); // the torch
        let (coins, ammo) = empty_carry();
        // The mouse stays within 200 px of the centre (168, 181).
        let c = ctx(&w, &inv, &coins, &ammo, (176.0, 176.0), false);
        assert_eq!(smart_select_lookup(&c, true, 0), Some(6));
    }

    #[test]
    fn s1_p_i54_smart_select_far_cursor_flips_to_the_glowstick() {
        // `Player.cs:17448-17456`: over 200 px (Y ×1.3) the light strategy
        // becomes 5 — `:17577-17584` prefers the glowstick over the torch.
        let w = tr_core::World::new("ss".into(), 400, 400);
        let mut inv = [EMPTY_SLOT; INV];
        inv[1] = slot(8, 10); // the torch
        inv[2] = slot(282, 5); // the glowstick
        let (coins, ammo) = empty_carry();
        // 40 tiles right of the player: dx 640 > 200.
        let c = ctx(&w, &inv, &coins, &ammo, (160.0 + 640.0, 176.0), false);
        assert_eq!(smart_select_lookup(&c, true, 0), Some(2));
    }

    #[test]
    fn s1_p_i54_smart_select_wet_player_wet_tile_takes_the_water_torch() {
        // `Player.cs:17666-17670` (strategy 4) + `:17542-17549`: a wet
        // tile with a wet player takes the water torch (523), not the dry
        // torch that precedes it.
        let mut w = tr_core::World::new("ss".into(), 40, 40);
        let i = w.index(11, 10);
        w.tiles[i].liquid = 255; // no active tile — the wet arm
        let mut inv = [EMPTY_SLOT; INV];
        inv[1] = slot(8, 10); // the dry torch
        inv[2] = slot(523, 5); // the Water Torch
        let (coins, ammo) = empty_carry();
        let c = ctx(&w, &inv, &coins, &ammo, (11.0 * 16.0 + 8.0, 10.0 * 16.0 + 8.0), true);
        assert_eq!(smart_select_lookup(&c, true, 0), Some(2));
    }

    #[test]
    fn s1_p_i54_smart_select_no_tool_in_range_falls_to_the_torch() {
        // `Player.cs:17683-17688`: no pickaxe → pickRange -10 shrinks the
        // reach rect into nothing, the solid tile matches no strategy and
        // the dry player keeps strategy 0.
        let mut w = tr_core::World::new("ss".into(), 40, 40);
        let i = w.index(11, 10);
        w.tiles[i] = tr_core::tile::Tile {
            kind: 1,
            ..Default::default()
        };
        w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        let mut inv = [EMPTY_SLOT; INV];
        inv[4] = slot(8, 10);
        let (coins, ammo) = empty_carry();
        let c = ctx(&w, &inv, &coins, &ammo, (11.0 * 16.0 + 8.0, 10.0 * 16.0 + 8.0), false);
        assert_eq!(smart_select_lookup(&c, true, 0), Some(4));
    }

    // ---------------- quick-heal chooser ----------------
    // RETIRED (gate V4 keep-one): the i54 chooser tests re-derived on the
    // canonical potions.rs seam live in `potions.rs::tests`
    // (`quick_heal_chooser_*` — the smallest-overheal, the 227 +30 grace,
    // the coin/ammo/vault scan and the non-potion skips all pinned there).

    // ---------------- W4-f47: the chlorophyte trader smart-select arm ----

    /// `Player.cs:17593-17600` — on the Chlorophyte Extractinator (642) a
    /// trade-table item (Copper Ore 12) wins strategy 7 even though it is
    /// not an ExtractinatorMode input.
    #[test]
    fn w4_f47_smart_select_trader_arm_picks_the_ore_on_642() {
        let mut w = tr_core::World::new("ss".into(), 40, 40);
        let i = w.index(11, 10);
        w.tiles[i] = tr_core::tile::Tile { kind: 642, ..Default::default() };
        w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        let mut inv = [EMPTY_SLOT; INV];
        inv[3] = slot(12, 5); // Copper Ore — trader-only, mode -1
        let (coins, ammo) = empty_carry();
        let c = ctx(&w, &inv, &coins, &ammo, (11.0 * 16.0 + 8.0, 10.0 * 16.0 + 8.0), false);
        assert_eq!(smart_select_lookup(&c, true, 0), Some(3));
    }

    /// `Player.cs:17601-17605` — on the plain Extractinator (219) the
    /// trader arm does NOT fire (the table is 642-only); Copper Ore has no
    /// ExtractinatorMode, so no slot matches strategy 7.
    #[test]
    fn w4_f47_smart_select_trader_arm_is_642_only() {
        let mut w = tr_core::World::new("ss".into(), 40, 40);
        let i = w.index(11, 10);
        w.tiles[i] = tr_core::tile::Tile { kind: 219, ..Default::default() };
        w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        let mut inv = [EMPTY_SLOT; INV];
        inv[3] = slot(12, 5); // Copper Ore — no mode, no trader on 219
        let (coins, ammo) = empty_carry();
        let c = ctx(&w, &inv, &coins, &ammo, (11.0 * 16.0 + 8.0, 10.0 * 16.0 + 8.0), false);
        // No extractinator input → the strategy falls through (no pick).
        assert_eq!(smart_select_lookup(&c, true, 0), None);
        // Silt (424, mode 0) still picks on 219.
        inv[3] = slot(424, 5);
        let c2 = ctx(&w, &inv, &coins, &ammo, (11.0 * 16.0 + 8.0, 10.0 * 16.0 + 8.0), false);
        assert_eq!(smart_select_lookup(&c2, true, 0), Some(3));
    }
}
