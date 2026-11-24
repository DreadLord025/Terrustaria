//! The player luck value — `Player.RecalculateLuck` (`Player.cs:29137-29164`),
//! the three `Main.dayRate` decay timers (`Player.cs:18103-18110` coinLuck,
//! `:18836-18854` ladyBugLuckTimeLeft, `:29166-29182` brokenMirrorBadLuckTime),
//! the coin-tier / ladybug-window helpers (`:18115-18153`,
//! `:18156-18167`, `NPC.cs:81566-81652`) and the torch-luck scan
//! (`Player.cs:17731-18088`).
//!
//! Everything the port cannot drive yet is a *neutral-default input* in
//! [`LuckState`]: `usedGalaxyPearl` (no player-file writer). The W4-e26
//! census wired the rest: the lantern-night flag copies
//! `world_events::lanterns_up` per tick, the garden-gnome flag copies the
//! SceneMetrics tile-567 marker, the equipment bonus accumulates the
//! clover/horseshoe/lucky-coin tail (`Player.cs:13050-13069`), and the
//! ladybug kill/nearby/town-death setters ride the fishing-bait, NPC and
//! death passes. The formula applies them in the original order — the
//! M2-BUFF-2 pattern.
//!
//! The value is **never clamped** — the −0.7/1.0 caps are UI-only
//! (`NormalizedLuck`, `Player.cs:4107-4122`); only `torchLuck` floors at 0
//! (`:18062-18065`). The multiplayer sync (`luckNeedsSync`, packet 134,
//! `NetMessage.cs:1557-1568`) is net-only and not ported.

use tr_core::scene::{ocean_depths, Zones};
use tr_core::tile::LiquidKind;
use tr_core::world::World;

/// `TorchID.Count` (`TorchID.cs:185`) — the size of `Player.nearbyTorch`.
pub const TORCH_ID_COUNT: usize = 24;

/// `NPC.ladyBugGoodLuckTime` (`NPC.cs:95298`).
pub const LADYBUG_GOOD_LUCK_TIME: i32 = 43_200;
/// `NPC.ladyBugBadLuckTime` (`NPC.cs:95299`) — negative on purpose.
pub const LADYBUG_BAD_LUCK_TIME: i32 = -10_800;
/// `NPC.ladyBugRainTime` (`NPC.cs:95300`) — what a kill adds to
/// `Main.ladyBugRainBoost`; `rain.rs` owns that field, so the kill setter
/// returns this add for its future writer.
pub const LADYBUG_RAIN_TIME: i32 = 1_800;
/// `NPC.maximumAmountOfTimesLadyBugRainCanStack` (`NPC.cs:95301`).
pub const LADYBUG_RAIN_CAP: i32 = 10 * LADYBUG_RAIN_TIME;

/// Every term of `Player.RecalculateLuck` (`Player.cs:29137-29164`) plus its
/// decay timers (`:18103-18110`, `:18836-18854`, `:29166-29182`).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LuckState {
    /// `Player.luck` (`Player.cs:3086`) — rebuilt from scratch every tick,
    /// never clamped.
    pub luck: f32,
    /// `Player.coinLuck` (`Player.cs:3092`) — copper-coin equivalents
    /// (`WorldItem.cs:1692-1716` shimmer absorption; no writer in the port yet).
    pub coin_luck: f32,
    /// `Player.ladyBugLuckTimeLeft` (`Player.cs:3084`).
    pub lady_bug_luck_time_left: i32,
    /// `Player.torchLuck` (`Player.cs:3062`) — the Phase-2 scan fills it;
    /// floors at 0 (`:18062-18065`).
    pub torch_luck: f32,
    /// `Player.luckPotion` (`Player.cs:1134`) — the buff-257 tier by
    /// *remaining* time (`:10285-10301`); cleared by ResetEffects (`:18814`).
    pub luck_potion: u8,
    /// `Player.kiteLuckLevel` (`Player.cs:3094`) — the buff-383 tier
    /// (`buffTime / 10800` clamp 0..3, `:9920-9924`). The C# has **no** reset:
    /// the field is assigned only inside the buff-383 branch, so it stays
    /// stale after the buff expires.
    pub kite_luck_level: u8,
    /// `Player.stinky` (`Player.cs:1146`) — buff 120 (`:10276-10280`);
    /// cleared by ResetEffects (`:18732`).
    pub stinky: bool,
    /// `Player.brokenMirrorBadLuck` (`Player.cs:2078`) — the −0.25 term
    /// while the timer runs (`:29166-29182`).
    pub broken_mirror_bad_luck: bool,
    /// `Player.brokenMirrorBadLuckTime` (`Player.cs:2078`) — set to 25200 by
    /// the PrettyMirror projectile (`Projectile.cs:70921`); no writer in the
    /// port yet, the decay still runs.
    pub broken_mirror_bad_luck_time: i32,
    /// `Player.equipmentBasedLuckBonus` (`Player.cs:2774`) — ResetEffects
    /// zeroes it (`:18812`), `UpdateEquips` accumulates the clover/horseshoe/
    /// lucky-coin adds in the C# order (`:13050-13069`, W4-e26).
    pub equipment_based_luck_bonus: f32,
    /// `Player.usedGalaxyPearl` (`Player.cs:858`) — the +0.03 term
    /// (`RecalculateLuck` `:29142-29145`). W4-a6: the persistence owner is
    /// `Player.used_galaxy_pearl` (the 5340 use path, the `.plr` slot
    /// `:54784`/`:55258`); the player's `update_luck` mirrors it here
    /// before every recalculation (the g90 lane-7 consumer split).
    /// `Player.usedGalaxyPearl` (`Player.cs:858`) — the C# persists it to the
    /// player file (`:54784/:55258`). W4-g88 v10 fold: the persistence owner
    /// is `Player.used_galaxy_pearl` (the sidecar unlock line's token 6,
    /// g90b `e76ee133`'s shape); `update_luck` mirrors it here before every
    /// recalculation (`:29142-29145`). The 5340 use arm is a6's
    /// (`powerups::use_shimmer_permanent` sets this field directly — the
    /// merger reconciles per research/138).
    pub used_galaxy_pearl: bool,
    /// `LanternNight.LanternsUp` (`LanternNight.cs:17-27`) — copied per
    /// tick from the S1-F-14 scheduler (`world_sim/player.rs`, W4-e26).
    pub lantern_night: bool,
    /// `Player.HasGardenGnomeNearby` (`Player.cs:2076`) — copied from the
    /// SceneMetrics tile-567 count (`SceneMetrics.cs:591`,
    /// `Player.cs:16726-16730`; W4-e26).
    pub has_garden_gnome_nearby: bool,
    // — the torch-luck scan state (`Player.cs:3054-3064`) —
    /// `Player.luckyTorchCounter` (`:3058`): the window's row cursor — one
    /// row per tick, 81 rows per recalculation.
    pub lucky_torch_counter: i32,
    /// `Player.nearbyTorch[TorchID.Count]` (`:3054`): the torch styles seen
    /// in the current window (cleared on every consume).
    pub nearby_torch: [bool; TORCH_ID_COUNT],
    /// `Player.nearbyTorches` (`:3054`): the unsnuffed-torch count — the
    /// Torch God trigger's input (the event rides `torch_god.rs`, W4-f33).
    pub nearby_torches: i32,
    /// `Player.dryCoralTorch` (`:3060`): a coral torch seen out of water.
    pub dry_coral_torch: bool,
    /// `Player._nextTorchLuckCheckCenter` (`:3064`): where the next 81-row
    /// window starts (reset to the player center on every consume).
    pub next_torch_luck_check_center: (f32, f32),
}

impl LuckState {
    /// `Player.GetLadyBugLuck` (`Player.cs:18156-18167`): the timer over its
    /// nominal window — `t/43200` while positive, `(0 − t)/−10800` while
    /// negative (the negative constant keeps the term negative).
    pub fn get_lady_bug_luck(&self) -> f32 {
        if self.lady_bug_luck_time_left > 0 {
            self.lady_bug_luck_time_left as f32 / LADYBUG_GOOD_LUCK_TIME as f32
        } else if self.lady_bug_luck_time_left < 0 {
            (0.0 - self.lady_bug_luck_time_left as f32) / LADYBUG_BAD_LUCK_TIME as f32
        } else {
            0.0
        }
    }

    /// `Player.CalculateCoinLuck` (`Player.cs:18115-18153`), verbatim including
    /// the duplicated unreachable `> 24900 → 0.175` branch (`:18125-18132`) —
    /// kept 1:1, the second test can never fire.
    pub fn calculate_coin_luck(&self) -> f32 {
        if self.coin_luck == 0.0 {
            return 0.0;
        }
        if self.coin_luck > 249_000.0 {
            return 0.2;
        }
        if self.coin_luck > 24_900.0 {
            return 0.175;
        }
        if self.coin_luck > 24_900.0 {
            // `Player.cs:18129-18132`: duplicated, unreachable — kept verbatim.
            return 0.175;
        }
        if self.coin_luck > 2_490.0 {
            return 0.15;
        }
        if self.coin_luck > 249.0 {
            return 0.125;
        }
        if (self.coin_luck as f64) > 24.9 {
            return 0.1;
        }
        if (self.coin_luck as f64) > 2.49 {
            return 0.075;
        }
        if (self.coin_luck as f64) > 0.249 {
            return 0.05;
        }
        0.025
    }

    /// `Player.UpdateCoinLuck` (`Player.cs:18103-18110`): only a positive
    /// balance decays, ×`0.9999^dayRate` per tick (the C# does
    /// `(float)Math.Pow(0.9999, Main.dayRate)` — f64 pow, then f32), floored
    /// to 0 below 0.25 (a `(double)` compare).
    pub fn update_coin_decay(&mut self, day_rate: i32) {
        if self.coin_luck <= 0.0 {
            return;
        }
        self.coin_luck *= (0.9999_f64).powf(day_rate as f64) as f32;
        if (self.coin_luck as f64) < 0.25 {
            self.coin_luck = 0.0;
        }
    }

    /// `Player.UpdateLadyBugLuckTime` (`Player.cs:18836-18854`): the good
    /// timer drains down to 0, the bad timer drains up to 0 — never sign-flips.
    pub fn update_lady_bug_decay(&mut self, day_rate: i32) {
        if self.lady_bug_luck_time_left > 0 {
            self.lady_bug_luck_time_left -= day_rate;
            if self.lady_bug_luck_time_left < 0 {
                self.lady_bug_luck_time_left = 0;
            }
        } else if self.lady_bug_luck_time_left < 0 {
            self.lady_bug_luck_time_left += day_rate;
            if self.lady_bug_luck_time_left > 0 {
                self.lady_bug_luck_time_left = 0;
            }
        }
    }

    /// `Player.UpdateBrokenMirrorLuck` (`Player.cs:29166-29182`): while the
    /// timer runs it drains (clamped at 0) and drives the flag; without a
    /// timer the flag clears. The diff→sync tail is net-only.
    pub fn update_broken_mirror_decay(&mut self, day_rate: i32) {
        if self.broken_mirror_bad_luck_time > 0 {
            self.broken_mirror_bad_luck = true;
            self.broken_mirror_bad_luck_time -= day_rate;
            if self.broken_mirror_bad_luck_time < 0 {
                self.broken_mirror_bad_luck_time = 0;
            }
        } else {
            self.broken_mirror_bad_luck = false;
        }
    }

    /// `Player.RecalculateLuck` (`Player.cs:29137-29164`), verbatim: rebuilt
    /// from scratch every tick, never clamped.
    pub fn recalculate_luck(&mut self) {
        self.luck = self.get_lady_bug_luck() * 0.2 + self.torch_luck * 0.2;
        self.luck += self.luck_potion as f32 * 0.1;
        self.luck += self.kite_luck_level as f32 * 0.1 / 3.0;
        if self.used_galaxy_pearl {
            self.luck += 0.03;
        }
        if self.lantern_night {
            self.luck += 0.3;
        }
        if self.has_garden_gnome_nearby {
            self.luck += 0.2;
        }
        if self.stinky {
            self.luck -= 0.25;
        }
        self.luck += self.equipment_based_luck_bonus;
        self.luck += self.calculate_coin_luck();
        if self.broken_mirror_bad_luck {
            self.luck -= 0.25;
        }
    }
}

/// `Player.TryRecalculatingTorchLuck` (`Player.cs:17731-17795`): the scan
/// processes **one tile row per player tick** across an 81-row window (X ±40,
/// clamped 10..maxTilesX−10) starting at
/// [`LuckState::next_torch_luck_check_center`]; when the cursor reaches the
/// window's bottom row the counters are consumed and `torch_luck` is
/// recalculated. Above the world surface or dead → consume immediately
/// (`:17750-17755`) — the field starts at (0,0), so the very first tick
/// consumes and anchors the window, exactly like the C# default. A `null`
/// tile returns without walking on (`:17761-17763`).
///
/// The no-event entry point (the tests + any caller without the player
/// context): delegates to [`torch_scan_step_event`] with no event — the
/// `happyFunTorchTime` / `torchGodCooldown` head (`:17734-17741`) and the
/// trigger tail (`:18066-18086`) are the W4-f33 seam, live only through
/// the event-carrying entry.
pub fn torch_scan_step(
    luck: &mut LuckState,
    world: &World,
    center: (f32, f32),
    dead: bool,
    held_torch_style: Option<u8>,
    zones: Zones,
) {
    let mut rng = tr_core::UnifiedRandom::default();
    let _ = torch_scan_step_impl(luck, world, center, dead, held_torch_style, zones, None, &mut rng);
}

/// The event-carrying scan entry (W4-f33) — `Player.Update`'s call site
/// (`player.rs::update_luck`): the C# head (`:17733-17742`) runs FIRST
/// (the event consumes the tick instead of the scan), the trigger tail
/// (`:18066-18086`) runs inside every consume BEFORE `nearbyTorches = 0`.
/// Returns the tick's deferred Torch God writes (the `pending_*`
/// convention — all RNG already consumed on `rng`, the UpdatePlayers
/// stream).
pub fn torch_scan_step_event(
    luck: &mut LuckState,
    world: &World,
    center: (f32, f32),
    dead: bool,
    held_torch_style: Option<u8>,
    zones: Zones,
    event: &mut crate::torch_god::TorchGodEventCtx<'_>,
    rng: &mut dyn tr_core::loot::DropRng,
) -> crate::torch_god::TorchGodPending {
    torch_scan_step_impl(
        luck,
        world,
        center,
        dead,
        held_torch_style,
        zones,
        Some(event),
        rng,
    )
}

fn torch_scan_step_impl(
    luck: &mut LuckState,
    world: &World,
    center: (f32, f32),
    dead: bool,
    held_torch_style: Option<u8>,
    zones: Zones,
    mut event: Option<&mut crate::torch_god::TorchGodEventCtx<'_>>,
    rng: &mut dyn tr_core::loot::DropRng,
) -> crate::torch_god::TorchGodPending {
    // The event head (`Player.cs:17733-17742`): while happyFunTorchTime
    // runs, the cursor zeroes and TorchAttack replaces the whole scan;
    // otherwise the cooldown decrements once per call.
    if let Some(ctx) = event.as_deref_mut() {
        if let Some(pending) =
            crate::torch_god::try_recalculating_torch_luck_head(ctx, luck, world, center, rng)
        {
            return pending;
        }
    }
    if (luck.next_torch_luck_check_center.1 as f64) < world.world_surface * 16.0 || dead {
        torch_consume_impl(luck, center, held_torch_style, zones, world, event);
        return Default::default();
    }
    // `int num = 1; int num2 = 40;` — one row per call, a ±40-tile window.
    let window = 40_i32;
    let top = luck.next_torch_luck_check_center.1 as i32 / 16 - window;
    let bottom = luck.next_torch_luck_check_center.1 as i32 / 16 + window;
    // `Utils.Clamp` (`Utils.cs:223-234`) tests the max first, so a world
    // narrower than the 10-tile margin folds onto the max instead of
    // panicking on a reversed range (the same fold recorded for the
    // underground `Collision.GetTilesIn` clamp).
    let max_x = world.width - 10;
    let x_lo = if luck.next_torch_luck_check_center.0 as i32 / 16 - window > max_x {
        max_x
    } else {
        (luck.next_torch_luck_check_center.0 as i32 / 16 - window).max(10)
    };
    let x_hi = if luck.next_torch_luck_check_center.0 as i32 / 16 + window > max_x {
        max_x
    } else {
        (luck.next_torch_luck_check_center.0 as i32 / 16 + window).max(10)
    };
    // `for (int i = 0; i < num; i++)` with `num == 1`: exactly one row.
    let row = top + luck.lucky_torch_counter;
    if row < 10 || row > world.height - 10 {
        luck.lucky_torch_counter += 1;
        return Default::default();
    }
    for x in x_lo..=x_hi {
        let Some(tile) = world.tile(x, row) else {
            // `if (tile == null) { return; }` — the whole scan stops.
            return Default::default();
        };
        if !tile.active() || tile.kind != 4 || tile.frame_x < 0 || tile.frame_y < 0 {
            continue;
        }
        if tile.frame_x < 66 {
            luck.nearby_torches += 1;
        }
        let style = tile.frame_y / 22;
        if (style as usize) < TORCH_ID_COUNT {
            luck.nearby_torch[style as usize] = true;
            // The coral gate: a torch sitting in water (liquidType 0) is not
            // "dry" (`:17772-17775`).
            if style == 17 && (tile.liquid == 0 || tile.liquid_kind() != LiquidKind::Water) {
                luck.dry_coral_torch = true;
            }
        }
    }
    if row >= bottom {
        torch_consume_impl(luck, center, held_torch_style, zones, world, event);
        return Default::default();
    }
    luck.lucky_torch_counter += 1;
    Default::default()
}

/// `Player.UpdateTorchLuck_ConsumeCountersAndCalculate`
/// (`Player.cs:17912-18088`): zeroes the cursor, resets `torchLuck`, anchors
/// the next window at the current center, feeds the held torch's own style
/// in, then runs the 12-verdict biome table and the five ladders. The Torch
/// God trigger tail (`:18066-18086`) runs through the W4-f33 event seam
/// (when the context is present) AFTER the clamp, BEFORE the
/// `nearbyTorches = 0` reset — the C#'s exact order.
fn torch_consume_and_calculate(
    luck: &mut LuckState,
    center: (f32, f32),
    held_torch_style: Option<u8>,
    zones: Zones,
    world: &World,
) {
    torch_consume_impl(luck, center, held_torch_style, zones, world, None);
}

fn torch_consume_impl(
    luck: &mut LuckState,
    center: (f32, f32),
    held_torch_style: Option<u8>,
    zones: Zones,
    world: &World,
    event: Option<&mut crate::torch_god::TorchGodEventCtx<'_>>,
) {
    luck.lucky_torch_counter = 0;
    luck.torch_luck = 0.0;
    luck.next_torch_luck_check_center = center;
    // The held torch counts as nearby (`:17917-17920`).
    if let Some(style) = held_torch_style {
        if (style as usize) < TORCH_ID_COUNT {
            luck.nearby_torch[style as usize] = true;
        }
    }
    // `num` = the wrong-biome penalty accumulator, `num2` = the correct-biome
    // bonus (`:17921-17922`). The gate `!ZoneDungeon && !ZoneLihzhardTemple`
    // (`:17923`) runs verbatim since W4-e27: `Zones.temple` rides
    // `SceneMetrics.zone_lihzhard_temple` (wall 87, `SceneMetrics.cs:694`).
    let mut num = 0.0_f32;
    let mut num2 = 0.0_f32;
    if !zones.dungeon && !zones.temple {
        if luck.nearby_torch[9] {
            // Ice torch (`TorchID.Ice`).
            if zones.snow { num2 += 1.0; } else { num += 1.0; }
        }
        if luck.nearby_torch[16] {
            // Desert torch.
            if zones.desert { num2 += 1.0; } else { num += 1.0; }
        }
        if luck.nearby_torch[17] {
            // Coral torch: in the ocean depths it is correct, dry elsewhere
            // it penalizes (`:17938-17946`).
            if ocean_depths(
                world.world_surface,
                world.rock_layer,
                center.0 as i32 / 16,
                world.width,
                center.1 as i32 / 16,
            ) {
                num2 += 1.0;
            } else if luck.dry_coral_torch {
                num += 1.0;
            }
        }
        if luck.nearby_torch[21] {
            // Jungle torch.
            if zones.jungle { num2 += 1.0; } else { num += 1.0; }
        }
        if luck.nearby_torch[22] {
            // Mushroom torch.
            if zones.glowshroom { num2 += 1.0; } else { num += 1.0; }
        }
        if luck.nearby_torch[18] {
            // Corrupt torch.
            if zones.corrupt { num2 += 1.0; } else { num += 1.0; }
        }
        if luck.nearby_torch[8] && zones.corrupt {
            // Cursed torch: never penalized, bonus only in the corruption.
            num2 += 1.0;
        }
        if luck.nearby_torch[19] {
            // Crimson torch.
            if zones.crimson { num2 += 1.0; } else { num += 1.0; }
        }
        if luck.nearby_torch[11] && zones.crimson {
            // Ichor torch: never penalized, bonus only in the crimson.
            num2 += 1.0;
        }
        if luck.nearby_torch[20] {
            // Hallowed torch.
            if zones.hallow { num2 += 1.0; } else { num += 1.0; }
        }
        if luck.nearby_torch[13]
            && !zones.snow
            && !zones.desert
            && !zones.corrupt
            && !zones.crimson
            && !zones.hallow
            && !zones.jungle
        {
            // Bone torch in a fully neutral biome.
            num2 += 0.5;
        }
        if luck.nearby_torch[0] {
            // The plain torch penalizes snow hardest.
            if zones.snow {
                num += 2.0;
            } else if zones.desert || zones.corrupt || zones.crimson || zones.hallow {
                num += 0.5;
            }
        }
    }
    if num2 >= 1.0 {
        luck.torch_luck += 1.0;
    } else if num2 > 0.0 {
        luck.torch_luck += 0.5;
    }
    if num >= 2.0 {
        luck.torch_luck += -1.5;
    } else if num >= 1.0 {
        luck.torch_luck += -1.0;
    } else if num > 0.0 {
        luck.torch_luck += -0.5;
    }
    luck.dry_coral_torch = false;
    for flag in &mut luck.nearby_torch {
        *flag = false;
    }
    // `torchLuck` is the ONLY clamped component in the system (`:18062-18065`).
    if luck.torch_luck < 0.0 {
        luck.torch_luck = 0.0;
    }
    // The Torch God trigger tail (`:18066-18086`, W4-f33) — AFTER the
    // clamp, BEFORE the `nearbyTorches = 0` reset, the C#'s exact order.
    if let Some(ctx) = event {
        crate::torch_god::torch_god_trigger_check(ctx, luck, world.world_surface);
    }
    luck.nearby_torches = 0;
}

/// The player-side part of `NPC.LadyBugKilled` (`NPC.cs:81566-81598`): the
/// bad-luck window — radius 400 px, linear weight `1 − d/400` (full under
/// 100 px), the negative value ×2 for a gold ladybug, keeping the **most
/// negative** timer. Returns the raw `ladyBugRainTime` add the kill contributes
/// to `Main.ladyBugRainBoost` (clamped at [`LADYBUG_RAIN_CAP`] by `rain.rs`,
/// which owns that field — the boost add happens before the dead bail, exactly
/// like the source).
pub fn on_lady_bug_killed(
    luck: &mut LuckState,
    position: (f32, f32),
    player_center: (f32, f32),
    player_dead: bool,
    gold: bool,
) -> i32 {
    // `Main.ladyBugRainBoost += NPC.ladyBugRainTime; if > cap → cap`
    // (`NPC.cs:81568-81572`) — the cap check rides the rain writer.
    let rain_add = LADYBUG_RAIN_TIME;
    if player_dead {
        return rain_add;
    }
    let radius = 400.0_f32;
    let mut len =
        ((position.0 - player_center.0).powi(2) + (position.1 - player_center.1).powi(2)).sqrt();
    if len < radius {
        if len < 100.0 {
            // `if (vector.Length() < 100f) { vector = default; }` — full weight.
            len = 0.0;
        }
        let weight = 1.0 - (len as f64) / (radius as f64);
        let mut time = (LADYBUG_BAD_LUCK_TIME as f64 * weight) as i32;
        if gold {
            time *= 2;
        }
        if time < luck.lady_bug_luck_time_left {
            luck.lady_bug_luck_time_left = time;
        }
    }
    rain_add
}

/// The player-side part of `NPC.LadyBugLuck` (`NPC.cs:81600-81626`): the good
/// window for a nearby (released, wild) ladybug — radius 800 px, weight
/// `(1 − d/800)^6` (full under 30 px), the positive value ×2 for gold, keeping
/// the **largest** timer, and never overwriting existing bad luck. The
/// `releaseOwner != 255` wild-only gate is NPC-side and stays with the M4
/// critter row.
pub fn on_lady_bug_nearby(
    luck: &mut LuckState,
    position: (f32, f32),
    player_center: (f32, f32),
    gold: bool,
) {
    if luck.lady_bug_luck_time_left < 0 {
        return;
    }
    let radius = 800.0_f32;
    let mut len =
        ((position.0 - player_center.0).powi(2) + (position.1 - player_center.1).powi(2)).sqrt();
    if len < radius {
        if len < 30.0 {
            len = 0.0;
        }
        let weight = (1.0 - (len as f64) / (radius as f64)).powf(6.0);
        let mut time = (LADYBUG_GOOD_LUCK_TIME as f64 * weight) as i32;
        if gold {
            time *= 2;
        }
        if time > luck.lady_bug_luck_time_left {
            luck.lady_bug_luck_time_left = time;
        }
    }
}

/// `NPC.NPCLuck` (`NPC.cs:81628-81652`): a town-NPC death moves the timer in
/// thirds — the Guide's death is worth `+43200/3`, any other town NPC's
/// `−10800/3` (Clothier 54 exempt), each keeping the extreme value, and the
/// good half never overwrites bad luck. The `type != 37 && type != 453`
/// caller gate (Old Man / Skeleton Merchant) stays with the M6 town row.
pub fn on_town_npc_death(luck: &mut LuckState, npc_type: i32, player_dead: bool) {
    if player_dead {
        return;
    }
    if npc_type == 22 {
        // The Guide: only good luck gains.
        if luck.lady_bug_luck_time_left < 0 {
            return;
        }
        let time = LADYBUG_GOOD_LUCK_TIME / 3;
        if time > luck.lady_bug_luck_time_left {
            luck.lady_bug_luck_time_left = time;
        }
    } else if npc_type != 54 {
        let time = LADYBUG_BAD_LUCK_TIME / 3;
        if time < luck.lady_bug_luck_time_left {
            luck.lady_bug_luck_time_left = time;
        }
    }
}

/// `Player.TryToPoop` (`Player.cs:18896-18940`, W4-f32): the toilet-sitting
/// digestion roll. `wellFed` gates the whole fn; `Next(600)` (200 tipsy)
/// gates the tick; the fed-tier ladder 207→3 / 206→2 / 26→1 picks the
/// stack multiplier; `DelBuff` removes the tier; `Utils.Clamp(buffTime /
/// 3600 * mult, mult, 999)` sizes the stack; `RollOnlyBadLuckExtreme(10)`
/// (`Luck.cs:53-60` — bad luck only, `Next(10/10)` = `Next(1)` = always 0
/// on a gate hit) boosts it ×1.2. Returns the Poop Block (5395) stack to
/// spawn at the player's `MountedCenter` (`RequestNewItem`,
/// `GrabDelayForAllPlayers`), or `None` when no roll fires. The caller
/// owns the sitting gate (`UpdateLifeRegen`'s toilet arm, `:19237-19240`)
/// and the buff-slot mutation: `fed_slot` is the `FindBuffIndex` result
/// the caller resolved (the tier ladder order 207/206/26), `buff_time`
/// that slot's remaining ticks. RNG slots: `Next(num)` FIRST, then the
/// luck roll — the `DelBuff`/clamp run between them in the C# but draw
/// nothing.
pub fn try_to_poop_stack(
    rng: &mut dyn tr_core::loot::DropRng,
    luck: f32,
    well_fed: bool,
    tipsy: bool,
    fed_tier: Option<(i32, i32)>, // (buff id 207/206/26, buffTime)
) -> Option<i32> {
    if !well_fed {
        return None; // `:18898-18901`
    }
    let num = if tipsy { 600 / 3 } else { 600 }; // `:18902-18906`
    if rng.next_range(0, num) != 0 {
        return None; // `:18907-18910`
    }
    // `:18911-18922`: the tier ladder — 207 → 3, else 206 → 2, else 26 → 1.
    let (mult, buff_time) = match fed_tier {
        Some((207, t)) => (3, t),
        Some((206, t)) => (2, t),
        Some((26, t)) => (1, t),
        _ => return None, // `num3 == -1` — no fed buff, no drop
    };
    // `:18925-18927`: DelBuff (the caller's slot clear), then
    // `Utils.Clamp(num4 / 3600 * num2, num2, 999)` — integer division
    // first, the max-first clamp fold (`Utils.cs`, the e27 pin).
    let v = buff_time / 3600 * mult;
    let mut stack = if v > 999 { 999 } else if v < mult { mult } else { v };
    // `:18928-18931`: RollOnlyBadLuckExtreme(10) — at luck >= 0 the roll
    // returns -1 with NO draw; negative luck draws the NextFloat gate,
    // and a hit rolls Next(1) = 0 → the ×1.2 boost always lands.
    if tr_core::loot::roll_only_bad_luck_extreme(rng, luck, 10) == 0 {
        stack = (stack as f64 * 1.2) as i32;
    }
    Some(stack) // `:18938` — RequestNewItem(5395, stack)
}

/// `Player.PlayGuitarChord` (`Player.cs:46671-46703`, W4-f32): the
/// Carbon/Ivy/Sparkle Guitar's chord cue by cursor distance. The luck
/// roll runs FIRST (`RollOnlyBadLuckExtreme(10)`, `:46675`): at luck >= 0
/// no draw; a negative-luck gate hit rolls `Next(1)` = 0 and then the
/// pitch offset `Next(-50, 51) * 0.01` (`:46677`). The cue ladder
/// (`:46679-46702`): range > 5/6 → Item49, > 4/6 → Item48, > 3/6 →
/// Item47, > 2/6 → Item51, > 1/6 → Item52, else Item50. Returns
/// `(cue, pitch_offset)`; the caller plays it at the player's Centre.
pub fn guitar_chord_cue(
    rng: &mut dyn tr_core::loot::DropRng,
    luck: f32,
    range: f32,
) -> (&'static str, f32) {
    let num = 1.0f32 / 6.0;
    let mut pitch_offset = 0.0f32;
    if tr_core::loot::roll_only_bad_luck_extreme(rng, luck, 10) == 0 {
        pitch_offset = rng.next_range(-50, 51) as f32 * 0.01;
    }
    let cue = if range > num * 5.0 {
        "Item49"
    } else if range > num * 4.0 {
        "Item48"
    } else if range > num * 3.0 {
        "Item47"
    } else if range > num * 2.0 {
        "Item51"
    } else if range > num {
        "Item52"
    } else {
        "Item50"
    };
    (cue, pitch_offset)
}

/// The `SOUND_STYLES` index of an `Item<N>` cue (the ctor-order table:
/// `Item1` sits at 126 — the Copper Pickaxe's `use_sound: Some(126)` —
/// so `ItemN` = 125 + N). The guitar chord's `play_use_sound` seam reads
/// it.
pub fn item_sound_style_index(cue: &str) -> Option<u16> {
    let n: u16 = cue.strip_prefix("Item")?.parse().ok()?;
    Some(125 + n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::FLAG_ACTIVE;

    // — DW-2.1/2.2/2.3: the torch-luck scan —

    /// A small test world: 64×800 → surface row 200 (3200 px), rock layer
    /// 360, ocean level `(200+360)/2 + 40 = 320` (`ocean_depths`).
    fn torch_world() -> World {
        World::new("t".into(), 64, 800)
    }

    /// Place an active tile with explicit frames (torch kind 4: style =
    /// `frameY / 22`, unsnuffed when `frameX < 66`).
    fn place(world: &mut World, x: i32, y: i32, kind: u16, frame_x: i16, frame_y: i16) {
        if let Some(t) = world.tile_mut(x, y) {
            t.kind = kind;
            t.frame_x = frame_x;
            t.frame_y = frame_y;
            t.flags |= FLAG_ACTIVE;
        }
    }

    fn snow_zones() -> Zones {
        Zones {
            snow: true,
            ..Default::default()
        }
    }

    #[test]
    fn torch_verdict_matrix_matches_player_cs_17924() {
        // DW-2.1 — each style's verdict fed straight through
        // `torch_consume_and_calculate` (styles pre-set as if walked). The
        // wrong-biome ladders exist in the C# but the `torchLuck < 0 → 0`
        // clamp (`:18062-18065`) hides any purely-negative outcome, so the
        // observable values are 1.0 / 0.5 / 0.0.
        let w = torch_world();
        let forest = (640.0, 6400.0);

        let mut s = LuckState::default();
        torch_consume_and_calculate(&mut s, forest, None, Zones::default(), &w);
        assert_eq!(s.torch_luck, 0.0, "no torches → no verdict");

        let mut s = LuckState::default();
        s.nearby_torch[9] = true;
        torch_consume_and_calculate(&mut s, forest, None, snow_zones(), &w);
        assert_eq!(s.torch_luck, 1.0, "ice in snow");
        let mut s = LuckState::default();
        s.nearby_torch[9] = true;
        torch_consume_and_calculate(&mut s, forest, None, Zones::default(), &w);
        assert_eq!(s.torch_luck, 0.0, "ice wrong biome → −1 → clamped");

        let mut s = LuckState::default();
        s.nearby_torch[16] = true;
        let d = Zones { desert: true, ..Default::default() };
        torch_consume_and_calculate(&mut s, forest, None, d, &w);
        assert_eq!(s.torch_luck, 1.0, "desert in desert");

        // Coral: correct in the ocean depths (center tile 20,300 — x 20 < 380,
        // y 300 ≤ ocean level 320), a dry coral torch penalizes elsewhere.
        let mut s = LuckState::default();
        s.nearby_torch[17] = true;
        torch_consume_and_calculate(&mut s, (320.0, 4800.0), None, Zones::default(), &w);
        assert_eq!(s.torch_luck, 1.0, "coral in the ocean depths");
        let mut s = LuckState::default();
        s.nearby_torch[17] = true;
        s.dry_coral_torch = true;
        torch_consume_and_calculate(&mut s, forest, None, Zones::default(), &w);
        assert_eq!(s.torch_luck, 0.0, "dry coral → −1 → clamped");

        let mut s = LuckState::default();
        s.nearby_torch[21] = true;
        let j = Zones { jungle: true, ..Default::default() };
        torch_consume_and_calculate(&mut s, forest, None, j, &w);
        assert_eq!(s.torch_luck, 1.0, "jungle in jungle");

        let mut s = LuckState::default();
        s.nearby_torch[22] = true;
        let m = Zones { glowshroom: true, ..Default::default() };
        torch_consume_and_calculate(&mut s, forest, None, m, &w);
        assert_eq!(s.torch_luck, 1.0, "mushroom in glowshroom");

        let mut s = LuckState::default();
        s.nearby_torch[18] = true;
        let c = Zones { corrupt: true, ..Default::default() };
        torch_consume_and_calculate(&mut s, forest, None, c, &w);
        assert_eq!(s.torch_luck, 1.0, "corrupt in corruption");

        // Cursed/ichor never penalize — neutral ground is a plain 0, not a
        // clamped negative.
        let mut s = LuckState::default();
        s.nearby_torch[8] = true;
        torch_consume_and_calculate(&mut s, forest, None, c, &w);
        assert_eq!(s.torch_luck, 1.0, "cursed in corruption");
        let mut s = LuckState::default();
        s.nearby_torch[8] = true;
        torch_consume_and_calculate(&mut s, forest, None, Zones::default(), &w);
        assert_eq!(s.torch_luck, 0.0, "cursed never penalized");

        let mut s = LuckState::default();
        s.nearby_torch[19] = true;
        let cr = Zones { crimson: true, ..Default::default() };
        torch_consume_and_calculate(&mut s, forest, None, cr, &w);
        assert_eq!(s.torch_luck, 1.0, "crimson in crimson");
        let mut s = LuckState::default();
        s.nearby_torch[11] = true;
        torch_consume_and_calculate(&mut s, forest, None, cr, &w);
        assert_eq!(s.torch_luck, 1.0, "ichor in crimson");
        let mut s = LuckState::default();
        s.nearby_torch[11] = true;
        torch_consume_and_calculate(&mut s, forest, None, Zones::default(), &w);
        assert_eq!(s.torch_luck, 0.0, "ichor never penalized");

        let mut s = LuckState::default();
        s.nearby_torch[20] = true;
        let h = Zones { hallow: true, ..Default::default() };
        torch_consume_and_calculate(&mut s, forest, None, h, &w);
        assert_eq!(s.torch_luck, 1.0, "hallowed in hallow");

        let mut s = LuckState::default();
        s.nearby_torch[13] = true;
        torch_consume_and_calculate(&mut s, forest, None, Zones::default(), &w);
        assert_eq!(s.torch_luck, 0.5, "bone in a neutral biome");
        let mut s = LuckState::default();
        s.nearby_torch[13] = true;
        torch_consume_and_calculate(&mut s, forest, None, snow_zones(), &w);
        assert_eq!(s.torch_luck, 0.0, "bone never outside neutral");

        // The plain torch: neutral 0, and its penalties hide behind the clamp.
        let mut s = LuckState::default();
        s.nearby_torch[0] = true;
        torch_consume_and_calculate(&mut s, forest, None, Zones::default(), &w);
        assert_eq!(s.torch_luck, 0.0, "plain torch in a neutral biome");
        let mut s = LuckState::default();
        s.nearby_torch[0] = true;
        torch_consume_and_calculate(&mut s, forest, None, snow_zones(), &w);
        assert_eq!(s.torch_luck, 0.0, "plain in snow → +2 num → clamped");

        // Stacks: desert-in-desert (+1) with a plain-in-desert (−0.5) shows
        // the −0.5 ladder; ice-in-snow with a plain-in-snow (+2 → −1.5)
        // clamps back to 0.
        let mut s = LuckState::default();
        s.nearby_torch[16] = true;
        s.nearby_torch[0] = true;
        torch_consume_and_calculate(&mut s, forest, None, d, &w);
        assert_eq!(s.torch_luck, 0.5, "1 − 0.5");
        let mut s = LuckState::default();
        s.nearby_torch[9] = true;
        s.nearby_torch[0] = true;
        torch_consume_and_calculate(&mut s, forest, None, snow_zones(), &w);
        assert_eq!(s.torch_luck, 0.0, "1 − 1.5 → clamped");

        // The dungeon skip zeroes the whole table.
        let mut s = LuckState::default();
        s.nearby_torch[9] = true;
        let dz = Zones { snow: true, dungeon: true, ..Default::default() };
        torch_consume_and_calculate(&mut s, forest, None, dz, &w);
        assert_eq!(s.torch_luck, 0.0, "dungeon skip");

        // The held torch feeds its own style in (`:17917-17920`).
        let mut s = LuckState::default();
        torch_consume_and_calculate(&mut s, forest, Some(9), snow_zones(), &w);
        assert_eq!(s.torch_luck, 1.0, "held ice torch in snow");
    }

    #[test]
    fn torch_scan_walks_one_row_per_tick_and_consumes() {
        // DW-2.2 — the window rows 360..440 walk one per tick; the ice torch
        // at tile (20, 370) is picked up on step 12 and consumed on step 82.
        let mut w = torch_world();
        place(&mut w, 20, 370, 4, 0, 9 * 22);
        let center = (32.0 * 16.0, 400.0 * 16.0);
        let zones = snow_zones();
        let mut s = LuckState::default();
        // Step 1: the default (0,0) check center is above the surface → the
        // window anchors at the player.
        torch_scan_step(&mut s, &w, center, false, None, zones);
        assert_eq!(s.next_torch_luck_check_center, center);
        assert_eq!(s.lucky_torch_counter, 0);
        // Steps 2..11 → rows 360..369: not the torch's row yet.
        for _ in 0..10 {
            torch_scan_step(&mut s, &w, center, false, None, zones);
        }
        assert!(!s.nearby_torch[9]);
        assert_eq!(s.lucky_torch_counter, 10);
        torch_scan_step(&mut s, &w, center, false, None, zones); // step 12 → row 370
        assert!(s.nearby_torch[9], "the ice torch's row was walked");
        assert_eq!(s.torch_luck, 0.0, "not consumed yet");
        // Steps 13..81 → rows 371..439, then step 82 hits row 440 = bottom.
        for _ in 0..69 {
            torch_scan_step(&mut s, &w, center, false, None, zones);
        }
        torch_scan_step(&mut s, &w, center, false, None, zones); // step 82 → consume
        assert_eq!(s.torch_luck, 1.0, "ice in snow → +1");
        assert_eq!(s.lucky_torch_counter, 0, "consumed");
        assert!(!s.nearby_torch[9], "cleared on consume");
    }

    #[test]
    fn torch_scan_consumes_above_surface_and_while_dead() {
        // DW-2.2 — `:17750-17755`: the (0,0)-anchored / dead player consumes
        // immediately and never walks a window.
        let mut w = torch_world();
        place(&mut w, 20, 370, 4, 0, 9 * 22);
        let zones = snow_zones();
        let above = (32.0 * 16.0, 100.0 * 16.0);
        let mut s = LuckState::default();
        for _ in 0..200 {
            torch_scan_step(&mut s, &w, above, false, None, zones);
        }
        assert_eq!(s.lucky_torch_counter, 0);
        assert!(!s.nearby_torch[9]);
        assert_eq!(s.torch_luck, 0.0, "above the surface, no verdict");

        let mut s = LuckState::default();
        s.next_torch_luck_check_center = (640.0, 6400.0);
        torch_scan_step(&mut s, &w, (640.0, 6400.0), true, None, zones);
        assert_eq!(s.lucky_torch_counter, 0, "dead consumes immediately");
    }

    #[test]
    fn torch_scan_window_clamps_the_x_range() {
        // DW-2.2 — tile 32's ±40 window clamps to 10..54 on a 64-wide world:
        // a torch at x 58 is inside the unclamped window but never walked.
        let mut w = torch_world();
        place(&mut w, 58, 370, 4, 0, 9 * 22);
        let center = (32.0 * 16.0, 400.0 * 16.0);
        let zones = snow_zones();
        let mut s = LuckState::default();
        for _ in 0..82 {
            torch_scan_step(&mut s, &w, center, false, None, zones);
        }
        assert_eq!(s.torch_luck, 0.0, "the clamped-out torch never counts");

        // Positive control: a bone torch inside the window → +0.5 (bone only
        // bonuses in a fully neutral biome, so plain default zones).
        place(&mut w, 30, 370, 4, 0, 13 * 22);
        let mut s = LuckState::default();
        for _ in 0..82 {
            torch_scan_step(&mut s, &w, center, false, None, Zones::default());
        }
        assert_eq!(s.torch_luck, 0.5, "the in-window bone torch counts");
    }

    #[test]
    fn torch_luck_floors_at_zero_and_feeds_the_luck_term() {
        // DW-2.3 — the only clamp in the system (`:18062-18065`); the luck
        // term `torchLuck · 0.2` then reflects the floored value.
        let mut s = LuckState::default();
        s.nearby_torch[9] = true;
        s.nearby_torch[16] = true; // ice wrong + desert wrong → num 2 → −1.5
        torch_consume_and_calculate(&mut s, (640.0, 6400.0), None, Zones::default(), &torch_world());
        assert_eq!(s.torch_luck, 0.0);
        s.recalculate_luck();
        assert_eq!(s.luck, 0.0, "no negative torch term leaks");
    }

    // — DW-1.1: RecalculateLuck per-term, coin tiers, duplicated branch —

    #[test]
    fn recalculate_terms_match_their_csharp_weights() {
        // Each term verified solo at its `Player.cs:29139-29163` weight.
        let mut s = LuckState::default();
        s.recalculate_luck();
        assert_eq!(s.luck, 0.0, "all-neutral state stays 0");

        s = LuckState::default();
        s.lady_bug_luck_time_left = 43_200;
        s.recalculate_luck();
        assert_eq!(s.luck, 0.2, "ladybug · 0.2");

        s = LuckState::default();
        s.torch_luck = 1.0;
        s.recalculate_luck();
        assert_eq!(s.luck, 0.2, "torchLuck · 0.2");

        s = LuckState::default();
        s.luck_potion = 3;
        s.recalculate_luck();
        assert_eq!(s.luck, 0.3, "luckPotion · 0.1");

        s = LuckState::default();
        s.kite_luck_level = 3;
        s.recalculate_luck();
        assert_eq!(s.luck, 0.1, "kite · 0.1/3 · 3");

        s = LuckState::default();
        s.used_galaxy_pearl = true;
        s.recalculate_luck();
        assert_eq!(s.luck, 0.03, "galaxy pearl");

        s = LuckState::default();
        s.lantern_night = true;
        s.recalculate_luck();
        assert_eq!(s.luck, 0.3, "lantern night");

        s = LuckState::default();
        s.has_garden_gnome_nearby = true;
        s.recalculate_luck();
        assert_eq!(s.luck, 0.2, "garden gnome");

        s = LuckState::default();
        s.stinky = true;
        s.recalculate_luck();
        assert_eq!(s.luck, -0.25, "stinky − 0.25");

        s = LuckState::default();
        s.equipment_based_luck_bonus = 0.13;
        s.recalculate_luck();
        assert_eq!(s.luck, 0.13, "equipment bonus raw");

        s = LuckState::default();
        s.coin_luck = 249_001.0;
        s.recalculate_luck();
        assert_eq!(s.luck, 0.2, "coin tier top");

        s = LuckState::default();
        s.broken_mirror_bad_luck = true;
        s.recalculate_luck();
        assert_eq!(s.luck, -0.25, "broken mirror − 0.25");

        // Stacked terms add in the C# order — f32 accumulation drifts one
        // ulp off the decimal value, exactly like the C# `float` adds.
        s = LuckState::default();
        s.lady_bug_luck_time_left = 21_600; // 0.5 → 0.1
        s.luck_potion = 2; // 0.2
        s.stinky = true; // −0.25
        s.recalculate_luck();
        assert!((s.luck - 0.05).abs() < 1e-6, "0.1 + 0.2 − 0.25: {}", s.luck);
    }

    #[test]
    fn lady_bug_luck_term_uses_the_signed_windows() {
        let mut s = LuckState::default();
        s.lady_bug_luck_time_left = 21_600;
        assert_eq!(s.get_lady_bug_luck(), 0.5);
        s.lady_bug_luck_time_left = -5_400;
        assert_eq!(s.get_lady_bug_luck(), -0.5);
        s.lady_bug_luck_time_left = 0;
        assert_eq!(s.get_lady_bug_luck(), 0.0);
        // A gold ladybug kill can push the timer past the nominal window —
        // the term keeps scaling (`Player.cs:18156-18167` has no clamp).
        s.lady_bug_luck_time_left = -21_600;
        assert_eq!(s.get_lady_bug_luck(), -2.0);
    }

    #[test]
    fn coin_luck_tier_matrix_matches_player_cs_18115() {
        // Boundaries from `CalculateCoinLuck` (`Player.cs:18115-18153`); the
        // duplicated `> 24900 → 0.175` branch stays unreachable (DW-1.1).
        let rows: &[(f32, f32)] = &[
            (0.0, 0.0),
            (1_000_000.0, 0.2),
            (249_001.0, 0.2),
            (249_000.0, 0.175),
            (24_901.0, 0.175),
            (24_900.0, 0.15),
            (2_491.0, 0.15),
            (2_490.0, 0.125),
            (250.0, 0.125),
            (249.0, 0.1),
            (25.0, 0.1),
            (2.5, 0.075),
            // 2.49f32 widens to 2.49000000953…, which IS `> 2.49` as f64 —
            // the C# `(double)this.coinLuck > 2.49` compares the widened
            // float, so the C# too returns 0.075 here (`:18137-18140`).
            (2.49, 0.075),
            (2.489, 0.05),
            (0.25, 0.05),
            (0.249, 0.025),
            (0.001, 0.025),
        ];
        for &(coin, want) in rows {
            let mut s = LuckState::default();
            s.coin_luck = coin;
            assert_eq!(s.calculate_coin_luck(), want, "coinLuck {coin}");
        }
    }

    // — DW-1.2: the three decay timers —

    #[test]
    fn coin_decay_is_the_dayrate_power_with_the_0_25_floor() {
        // `UpdateCoinLuck` (`Player.cs:18103-18110`): one tick at dayRate 1
        // multiplies by 0.9999 exactly; the floor zeroes below 0.25.
        let mut s = LuckState::default();
        s.coin_luck = 1_000_000.0;
        s.update_coin_decay(1);
        assert!((s.coin_luck - 999_900.0).abs() < 0.001, "{}", s.coin_luck);

        s.coin_luck = 0.5;
        s.update_coin_decay(1);
        assert!((s.coin_luck - 0.49995).abs() < 1e-6);

        // Exactly 0.25 decays under the floor; just above it survives the
        // tick (the `< 0.25` compare runs on the widened f64).
        s.coin_luck = 0.25;
        s.update_coin_decay(1);
        assert_eq!(s.coin_luck, 0.0, "0.25 · 0.9999 < 0.25 → floored");

        s.coin_luck = 0.2501;
        s.update_coin_decay(1);
        assert!(
            (s.coin_luck - 0.25007498).abs() < 1e-9,
            "no floor above 0.25: {}",
            s.coin_luck
        );

        s.coin_luck = 0.0;
        s.update_coin_decay(1);
        assert_eq!(s.coin_luck, 0.0, "nothing to decay");

        s.coin_luck = -10.0;
        s.update_coin_decay(1);
        assert_eq!(s.coin_luck, -10.0, "the `!(coinLuck <= 0)` gate skips");

        // dayRate 60 (fast-forward) is 60 multiplications in one tick.
        let mut f = LuckState::default();
        f.coin_luck = 1_000_000.0;
        f.update_coin_decay(60);
        let expected = 1_000_000.0_f32 * (0.9999_f64).powf(60.0) as f32;
        assert!((f.coin_luck - expected).abs() < 0.01);
    }

    #[test]
    fn lady_bug_decay_drains_toward_zero_without_sign_flip() {
        let mut s = LuckState::default();
        s.lady_bug_luck_time_left = 3;
        s.update_lady_bug_decay(1);
        assert_eq!(s.lady_bug_luck_time_left, 2);
        s.update_lady_bug_decay(5);
        assert_eq!(s.lady_bug_luck_time_left, 0, "clamped at 0, not negative");

        s.lady_bug_luck_time_left = -3;
        s.update_lady_bug_decay(1);
        assert_eq!(s.lady_bug_luck_time_left, -2);
        s.update_lady_bug_decay(5);
        assert_eq!(s.lady_bug_luck_time_left, 0, "clamped at 0, not positive");

        // The 60× fast-forward dayRate walks the same drain in one tick.
        s.lady_bug_luck_time_left = 43_200 + 27;
        s.update_lady_bug_decay(60);
        assert_eq!(s.lady_bug_luck_time_left, 43_200 + 27 - 60);
    }

    #[test]
    fn broken_mirror_decay_drives_the_flag() {
        let mut s = LuckState::default();
        s.broken_mirror_bad_luck_time = 25_200;
        s.broken_mirror_bad_luck = false;
        s.update_broken_mirror_decay(1);
        assert!(s.broken_mirror_bad_luck);
        assert_eq!(s.broken_mirror_bad_luck_time, 25_199);

        s.broken_mirror_bad_luck_time = 1;
        s.update_broken_mirror_decay(2);
        assert_eq!(s.broken_mirror_bad_luck_time, 0);
        assert!(s.broken_mirror_bad_luck, "still set on the drain tick");

        s.update_broken_mirror_decay(1);
        assert!(!s.broken_mirror_bad_luck, "timer spent → flag clears");

        s.update_broken_mirror_decay(1);
        assert!(!s.broken_mirror_bad_luck);
        assert_eq!(s.broken_mirror_bad_luck_time, 0, "never goes negative");
    }

    // — DW-1.1 (writers, callable with no live NPC): ladybug + town setters —

    #[test]
    fn lady_bug_kill_sets_the_most_negative_timer_and_returns_the_rain_add() {
        // `NPC.LadyBugKilled` (`NPC.cs:81566-81598`): linear weight over 400 px,
        // full under 100 px, gold ×2, keep most negative; the rain add comes
        // back for the future `rain.rs` writer.
        let mut s = LuckState::default();
        let add = on_lady_bug_killed(&mut s, (200.0, 0.0), (0.0, 0.0), false, false);
        assert_eq!(add, LADYBUG_RAIN_TIME);
        assert_eq!(s.lady_bug_luck_time_left, -10_800 / 2, "d = 200 → w 0.5");

        // Inside 100 px the weight saturates at 1 (strict `< 100f`).
        let mut s = LuckState::default();
        on_lady_bug_killed(&mut s, (50.0, 0.0), (0.0, 0.0), false, false);
        assert_eq!(s.lady_bug_luck_time_left, -10_800);

        // Gold doubles the negative value.
        let mut s = LuckState::default();
        on_lady_bug_killed(&mut s, (50.0, 0.0), (0.0, 0.0), false, true);
        assert_eq!(s.lady_bug_luck_time_left, -21_600);

        // Exactly 100 px is NOT saturated (`< 100f` is strict): w = 0.75.
        let mut s = LuckState::default();
        on_lady_bug_killed(&mut s, (100.0, 0.0), (0.0, 0.0), false, false);
        assert_eq!(s.lady_bug_luck_time_left, -10_800 * 3 / 4);

        // Out of range → no timer change.
        let mut s = LuckState::default();
        on_lady_bug_killed(&mut s, (401.0, 0.0), (0.0, 0.0), false, false);
        assert_eq!(s.lady_bug_luck_time_left, 0);

        // Dead player: the timer never moves, but the rain add still returns
        // (the C# adds to `Main.ladyBugRainBoost` before the bail).
        let mut s = LuckState::default();
        let add = on_lady_bug_killed(&mut s, (50.0, 0.0), (0.0, 0.0), true, false);
        assert_eq!(add, LADYBUG_RAIN_TIME);
        assert_eq!(s.lady_bug_luck_time_left, 0);

        // The keep-most-negative rule: a worse kill wins.
        let mut s = LuckState::default();
        s.lady_bug_luck_time_left = -5_000;
        on_lady_bug_killed(&mut s, (50.0, 0.0), (0.0, 0.0), false, false);
        assert_eq!(s.lady_bug_luck_time_left, -10_800);
        on_lady_bug_killed(&mut s, (380.0, 0.0), (0.0, 0.0), false, false);
        assert_eq!(s.lady_bug_luck_time_left, -10_800, "weaker kill loses");
    }

    #[test]
    fn lady_bug_nearby_sets_the_largest_good_timer_pow6_weight() {
        // `NPC.LadyBugLuck` (`NPC.cs:81600-81626`): (1 − d/800)^6 over 800 px,
        // full under 30 px, gold ×2, keep largest, never overwrites bad luck.
        let mut s = LuckState::default();
        on_lady_bug_nearby(&mut s, (20.0, 0.0), (0.0, 0.0), false);
        assert_eq!(s.lady_bug_luck_time_left, 43_200, "under 30 px saturates");

        let mut s = LuckState::default();
        on_lady_bug_nearby(&mut s, (400.0, 0.0), (0.0, 0.0), false);
        assert_eq!(s.lady_bug_luck_time_left, (43_200.0 * 0.5_f64.powf(6.0)) as i32);

        // Exactly 30 px is NOT saturated (`< 30f` is strict): (1 − 30/800)^6.
        let mut s = LuckState::default();
        on_lady_bug_nearby(&mut s, (30.0, 0.0), (0.0, 0.0), false);
        let want = ((1.0_f64 - 30.0_f64 / 800.0).powf(6.0) * 43_200.0) as i32;
        assert_eq!(s.lady_bug_luck_time_left, want, "30 px boundary");

        // Gold doubles.
        let mut s = LuckState::default();
        on_lady_bug_nearby(&mut s, (20.0, 0.0), (0.0, 0.0), true);
        assert_eq!(s.lady_bug_luck_time_left, 86_400);

        // Out of range → untouched.
        let mut s = LuckState::default();
        on_lady_bug_nearby(&mut s, (801.0, 0.0), (0.0, 0.0), false);
        assert_eq!(s.lady_bug_luck_time_left, 0);

        // Bad luck is never overwritten by a nearby good ladybug.
        let mut s = LuckState::default();
        s.lady_bug_luck_time_left = -10_800;
        on_lady_bug_nearby(&mut s, (30.0, 0.0), (0.0, 0.0), true);
        assert_eq!(s.lady_bug_luck_time_left, -10_800);
    }

    #[test]
    fn town_npc_death_moves_the_timer_in_thirds() {
        // `NPC.NPCLuck` (`NPC.cs:81628-81652`): Guide +14400, other town NPCs
        // −3600, Clothier (54) exempt; extremes win; dead player bails.
        let mut s = LuckState::default();
        on_town_npc_death(&mut s, 22, false);
        assert_eq!(s.lady_bug_luck_time_left, 14_400, "Guide death");

        let mut s = LuckState::default();
        on_town_npc_death(&mut s, 1, false);
        assert_eq!(s.lady_bug_luck_time_left, -3_600, "any other town NPC");

        let mut s = LuckState::default();
        on_town_npc_death(&mut s, 54, false);
        assert_eq!(s.lady_bug_luck_time_left, 0, "Clothier exempt");

        // Good half never overwrites bad luck.
        let mut s = LuckState::default();
        s.lady_bug_luck_time_left = -3_600;
        on_town_npc_death(&mut s, 22, false);
        assert_eq!(s.lady_bug_luck_time_left, -3_600);

        // Extremes win.
        let mut s = LuckState::default();
        s.lady_bug_luck_time_left = 20_000;
        on_town_npc_death(&mut s, 22, false);
        assert_eq!(s.lady_bug_luck_time_left, 20_000);

        let mut s = LuckState::default();
        s.lady_bug_luck_time_left = -20_000;
        on_town_npc_death(&mut s, 1, false);
        assert_eq!(s.lady_bug_luck_time_left, -20_000);

        let mut s = LuckState::default();
        s.lady_bug_luck_time_left = -20_000;
        on_town_npc_death(&mut s, 22, false);
        assert_eq!(s.lady_bug_luck_time_left, -20_000, "good half skips");

        let mut s = LuckState::default();
        s.lady_bug_luck_time_left = -20_000;
        on_town_npc_death(&mut s, 1, true);
        assert_eq!(s.lady_bug_luck_time_left, -20_000, "dead player bails");
    }

    // — W4-e27: the biome-matching lane's verbatim witnesses —

    #[test]
    fn e27_temple_zone_skips_the_whole_verdict_table() {
        // `Player.cs:17923`: `!ZoneDungeon && !ZoneLihzhardTemple` — the
        // temple flag (wall 87, `SceneMetrics.cs:694`) zeroes the table
        // exactly like the dungeon flag, even in a matching biome.
        let w = torch_world();
        let forest = (640.0, 6400.0);
        let mut s = LuckState::default();
        s.nearby_torch[9] = true; // ice in snow would be +1
        let tz = Zones { snow: true, temple: true, ..Default::default() };
        torch_consume_and_calculate(&mut s, forest, None, tz, &w);
        assert_eq!(s.torch_luck, 0.0, "temple skip");
        // Both flags together still skip; either alone is enough.
        let mut s = LuckState::default();
        s.nearby_torch[21] = true;
        let both = Zones { jungle: true, dungeon: true, temple: true, ..Default::default() };
        torch_consume_and_calculate(&mut s, forest, None, both, &w);
        assert_eq!(s.torch_luck, 0.0, "dungeon+temple skip");
        // The held torch still feeds its flag in on a skipped consume
        // (`:17917-17920` runs BEFORE the gate) — but the table never reads
        // it, and the flag clears at the tail (`:18058-18061`).
        let mut s = LuckState::default();
        torch_consume_and_calculate(&mut s, forest, Some(9), tz, &w);
        assert_eq!(s.torch_luck, 0.0);
        assert!(!s.nearby_torch[9], "cleared even on a skipped consume");
    }

    #[test]
    fn e27_wrong_torch_penalty_ladders_are_the_csharp_steps() {
        // `Player.cs:18037-18056`: bonus `num2 ≥ 1 → +1`, `> 0 → +0.5`
        // (a SINGLE +1 no matter how many correct torches); penalty
        // `num ≥ 2 → −1.5`, `≥ 1 → −1`, `> 0 → −0.5`. The `< 0 → 0` clamp
        // (`:18062-18065`) hides purely-negative outcomes, so the ladders
        // are witnessed through mixed correct+wrong stacks.
        let w = torch_world();
        let forest = (640.0, 6400.0);
        // num2 = 2 (jungle + mushroom both correct) still bonuses only +1;
        // with one wrong torch (num = 1 → −1) the result is 0, NOT 0.5.
        let mut s = LuckState::default();
        s.nearby_torch[21] = true;
        s.nearby_torch[22] = true;
        s.nearby_torch[9] = true; // ice, not snow → num 1
        let jm = Zones { jungle: true, glowshroom: true, ..Default::default() };
        torch_consume_and_calculate(&mut s, forest, None, jm, &w);
        assert_eq!(s.torch_luck, 0.0, "+1 (capped) − 1 = 0");
        // num = 0.5 (plain torch in the corruption) → the −0.5 step:
        // jungle-in-jungle +1 minus 0.5 = 0.5.
        let mut s = LuckState::default();
        s.nearby_torch[21] = true;
        s.nearby_torch[0] = true;
        let jc = Zones { jungle: true, corrupt: true, ..Default::default() };
        torch_consume_and_calculate(&mut s, forest, None, jc, &w);
        assert_eq!(s.torch_luck, 0.5, "+1 − 0.5");
        // num = 2 (ice + desert both wrong) → −1.5: +1 − 1.5 → clamped 0.
        let mut s = LuckState::default();
        s.nearby_torch[21] = true;
        s.nearby_torch[9] = true;
        s.nearby_torch[16] = true;
        let j = Zones { jungle: true, ..Default::default() };
        torch_consume_and_calculate(&mut s, forest, None, j, &w);
        assert_eq!(s.torch_luck, 0.0, "+1 − 1.5 → clamped");
        // Bone in the corruption: the neutrality test (`:18021`) includes
        // ZoneCorrupt, so bone gives NOTHING; the plain torch's −0.5
        // (`:18031`) then clamps to 0.
        let mut s = LuckState::default();
        s.nearby_torch[13] = true;
        s.nearby_torch[0] = true;
        let c = Zones { corrupt: true, ..Default::default() };
        torch_consume_and_calculate(&mut s, forest, None, c, &w);
        assert_eq!(s.torch_luck, 0.0, "bone is not neutral in the corruption");
        // The plain torch's snow penalty is 2, not 0.5 (`:18027-18030`):
        // ice-in-snow +1 with plain-in-snow num 2 → −1.5 → clamped 0.
        let mut s = LuckState::default();
        s.nearby_torch[9] = true;
        s.nearby_torch[0] = true;
        torch_consume_and_calculate(&mut s, forest, None, snow_zones(), &w);
        assert_eq!(s.torch_luck, 0.0, "snow plain penalty is the 2-step");
    }

    #[test]
    fn e27_shimmer_and_color_torches_have_no_verdict() {
        // The luck table reads only the 12 arms of `:17925-18035`; the
        // shimmer torch (23, `TorchID.cs:131`) IS in `Sets.IsABiomeTorch`
        // (`TorchID.cs:17`) but has NO luck verdict — nor do the colour
        // torches 1-6/10/12/14/15 or the demon torch 7.
        let w = torch_world();
        let forest = (640.0, 6400.0);
        for style in [1usize, 2, 3, 4, 5, 6, 7, 10, 12, 14, 15, 23] {
            let mut s = LuckState::default();
            s.nearby_torch[style] = true;
            torch_consume_and_calculate(&mut s, forest, None, snow_zones(), &w);
            assert_eq!(s.torch_luck, 0.0, "style {style} has no verdict");
        }
    }

    #[test]
    fn e27_scan_counts_only_unsnuffed_torches_and_resets_on_consume() {
        // `Player.cs:17774-17786`: `frameX < 66` bumps `nearbyTorches`
        // BEFORE the `TorchID.Count` gate; the style flag sets for any
        // in-range style, snuffed or not. `:18087` zeroes the count on
        // every consume (the Torch God trigger tail is f33's lane).
        let mut w = torch_world();
        place(&mut w, 20, 370, 4, 0, 9 * 22); // unsnuffed ice
        place(&mut w, 21, 370, 4, 22, 9 * 22); // unsnuffed ice (frameX 22)
        place(&mut w, 22, 370, 4, 44, 16 * 22); // unsnuffed desert
        place(&mut w, 23, 370, 4, 66, 9 * 22); // SNUFFED ice (frameX 66)
        place(&mut w, 24, 370, 4, 0, 24 * 22); // style 24 ≥ Count
        let center = (32.0 * 16.0, 400.0 * 16.0);
        let mut s = LuckState::default();
        for _ in 0..12 {
            torch_scan_step(&mut s, &w, center, false, None, Zones::default());
        }
        assert_eq!(s.nearby_torches, 4, "3 styled + the style-24 torch; snuffed excluded");
        assert!(s.nearby_torch[9], "the snuffed torch still sets its style flag");
        assert!(s.nearby_torch[16]);
        assert_eq!(s.nearby_torch.iter().filter(|f| **f).count(), 2, "style 24 never flags");
        for _ in 0..70 {
            torch_scan_step(&mut s, &w, center, false, None, Zones::default());
        }
        assert_eq!(s.nearby_torches, 0, "consume zeroes the count (`:18087`)");
        // 101 unsnuffed torches (the Torch God trigger threshold, `:18066`)
        // change NOTHING in the luck factor: the count is not a verdict
        // input, and the event itself is f33's separate lane. The ±40
        // window is 81 columns wide, so the 101 ride two rows.
        let mut w = World::new("t".into(), 200, 800);
        for x in 60..=140 {
            place(&mut w, x, 370, 4, 0, 0); // 81 plain torches
        }
        for x in 60..80 {
            place(&mut w, x, 371, 4, 0, 0); // +20 → 101
        }
        let center = (100.0 * 16.0, 400.0 * 16.0);
        let mut s = LuckState::default();
        for _ in 0..13 {
            torch_scan_step(&mut s, &w, center, false, None, Zones::default());
        }
        assert_eq!(s.nearby_torches, 101, "both rows walked, count > 100");
        for _ in 0..69 {
            torch_scan_step(&mut s, &w, center, false, None, Zones::default());
        }
        assert_eq!(s.torch_luck, 0.0, "plain torches in a neutral biome: no verdict");
        assert_eq!(s.nearby_torches, 0, "the >100 count still resets, no event here");
    }

    #[test]
    fn e27_dry_coral_gate_is_liquid_zero_or_non_water() {
        // `Player.cs:17782-17785`: `num5 == 17 && (tile.liquid == 0 ||
        // tile.liquidType() != 0)` — water (type 0) with liquid > 0 is the
        // ONLY wet case; lava/honey/shimmer (`Tile.cs:235-238` types 1/2/3)
        // all read dry. The flag clears on consume (`:18057`).
        let center = (32.0 * 16.0, 400.0 * 16.0);
        for (kind, want_dry) in [
            (None, true), // liquid == 0
            (Some(LiquidKind::Water), false),
            (Some(LiquidKind::Lava), true),
            (Some(LiquidKind::Honey), true),
            (Some(LiquidKind::Shimmer), true),
        ] {
            let mut w = torch_world();
            place(&mut w, 20, 370, 4, 0, 17 * 22);
            if let Some(k) = kind {
                if let Some(t) = w.tile_mut(20, 370) {
                    t.liquid = 255;
                    t.set_liquid_kind(k);
                }
            }
            let mut s = LuckState::default();
            for _ in 0..12 {
                torch_scan_step(&mut s, &w, center, false, None, Zones::default());
            }
            assert_eq!(s.dry_coral_torch, want_dry, "liquid kind {kind:?}");
            for _ in 0..70 {
                torch_scan_step(&mut s, &w, center, false, None, Zones::default());
            }
            assert!(!s.dry_coral_torch, "cleared on consume");
        }
    }

    #[test]
    fn e27_coral_verdict_uses_the_ocean_depths_boundaries() {
        // `WorldGen.oceanDepths` (`WorldGen.cs:10963-10974`): `y >
        // oceanLevel` → false (STRICT >), else `x < 380 || x >
        // maxTilesX − 380` (`beachDistance` 380, `:86914`; `oceanLevel =
        // (worldSurface + rockLayer)/2 + 40`, `:4447`).
        let mut w = World::new("t".into(), 800, 800);
        w.world_surface = 200.0;
        w.rock_layer = 360.0; // ocean level 320
        let consume = |x_tile: i32, y_tile: i32| {
            let mut s = LuckState::default();
            s.nearby_torch[17] = true;
            torch_consume_and_calculate(
                &mut s,
                (x_tile as f32 * 16.0, y_tile as f32 * 16.0),
                None,
                Zones::default(),
                &w,
            );
            s.torch_luck
        };
        assert_eq!(consume(20, 320), 1.0, "y == oceanLevel is still ocean (strict >)");
        assert_eq!(consume(20, 321), 0.0, "one row below the ocean level");
        assert_eq!(consume(379, 300), 1.0, "x < 380 beach band");
        assert_eq!(consume(380, 300), 0.0, "x == 380 is out of the band");
        assert_eq!(consume(420, 300), 0.0, "x == maxTilesX − 380 is out");
        assert_eq!(consume(421, 300), 1.0, "x > maxTilesX − 380 beach band");
    }

    #[test]
    fn e27_window_geometry_is_verbatim_40_40_one_row_81_ticks() {
        // `Player.cs:17749-17758`: `num = 1` row per tick, `num2 = 40`
        // half-window → 81 rows, consume on the bottom row INCLUSIVE.
        let mut w = torch_world();
        place(&mut w, 20, 359, 4, 0, 13 * 22); // one row ABOVE the window
        place(&mut w, 21, 441, 4, 0, 13 * 22); // one row BELOW the window
        place(&mut w, 22, 440, 4, 0, 13 * 22); // the bottom row itself
        let center = (32.0 * 16.0, 400.0 * 16.0);
        let mut s = LuckState::default();
        for _ in 0..82 {
            torch_scan_step(&mut s, &w, center, false, None, Zones::default());
        }
        assert_eq!(s.torch_luck, 0.5, "only the bottom-row bone torch counted");
        assert_eq!(s.lucky_torch_counter, 0);
        // The consume re-anchors at the CURRENT center (`:17916`), so a
        // moving player drags the next window with them.
        let moved = (33.0 * 16.0, 401.0 * 16.0);
        torch_scan_step(&mut s, &w, moved, false, None, Zones::default());
        // (the first post-consume tick walks row top = 401−40 = 361)
        assert_eq!(s.next_torch_luck_check_center, center, "anchor set at consume");
        for _ in 0..80 {
            torch_scan_step(&mut s, &w, moved, false, None, Zones::default());
        }
        assert_eq!(s.next_torch_luck_check_center, moved, "re-anchored at the new center");
    }

    #[test]
    fn e27_narrow_world_clamp_folds_max_first_and_out_of_range_rows_skip() {
        // `Utils.Clamp` (`Utils.cs:223-234`) tests the MAX first: on a
        // 15-wide world (max_x = 5) the ±40 window folds to 10..=5 — an
        // EMPTY range, no panic, no torch ever walked.
        let mut w = World::new("t".into(), 15, 800);
        place(&mut w, 10, 370, 4, 0, 13 * 22);
        let center = (7.0 * 16.0, 400.0 * 16.0);
        let mut s = LuckState::default();
        for _ in 0..200 {
            torch_scan_step(&mut s, &w, center, false, None, Zones::default());
        }
        assert_eq!(s.torch_luck, 0.0, "the folded range never walks a tile");
        // `Player.cs:17759-17762`: rows outside 10..maxTilesY−10 `continue`
        // — the counter still bumps but the bottom-row consume check is
        // SKIPPED, so a window whose bottom lies past the bound never
        // consumes (the verbatim quirk; the port's early return matches).
        let mut w = torch_world(); // height 800 → bound 790
        place(&mut w, 20, 790, 4, 0, 13 * 22); // the last walkable row
        place(&mut w, 21, 791, 4, 0, 9 * 22); // past the bound
        let center = (32.0 * 16.0, 780.0 * 16.0); // top 740, bottom 820
        let mut s = LuckState::default();
        for _ in 0..100 {
            torch_scan_step(&mut s, &w, center, false, None, Zones::default());
        }
        assert!(s.nearby_torch[13], "row 790 walked (bound is strict >)");
        assert!(!s.nearby_torch[9], "row 791 skipped");
        assert_eq!(s.lucky_torch_counter, 99, "no consume: bottom 820 > bound 790");
        assert_eq!(s.torch_luck, 0.0, "the verdict never runs");
    }

    #[test]
    fn e27_held_torch_style_out_of_range_is_ignored() {
        // `Player.cs:17917-17920`: `placeStyle < TorchID.Count` gates the
        // held-torch feed; the sim's `held_torch_style` builder applies the
        // same gate (`world_sim/player.rs`), and the consume re-checks it.
        let w = torch_world();
        let mut s = LuckState::default();
        torch_consume_and_calculate(&mut s, (640.0, 6400.0), Some(24), snow_zones(), &w);
        assert_eq!(s.torch_luck, 0.0, "style 24 never feeds the table");
        assert!(s.nearby_torch.iter().all(|f| !*f));
    }

    #[test]
    fn luck_stays_unclamped_at_extremes() {
        // The −0.7/1.0 caps are UI-only (`Player.cs:4107-4122`); the value
        // itself passes raw (DW-1.1 edge).
        let mut s = LuckState::default();
        s.lady_bug_luck_time_left = -21_600; // −2.0 → −0.4
        s.coin_luck = 1_000_000.0; // +0.2
        s.recalculate_luck();
        assert!((s.luck - (-0.2)).abs() < 1e-6, "{}", s.luck);

        s = LuckState::default();
        s.lady_bug_luck_time_left = 86_400; // gold ladybug good → +0.4
        s.torch_luck = 1.0; // +0.2
        s.luck_potion = 3; // +0.3
        s.lantern_night = true; // +0.3
        s.coin_luck = 1_000_000.0; // +0.2
        s.recalculate_luck();
        assert!((s.luck - 1.4).abs() < 1e-6, "no upper clamp: {}", s.luck);
    }

    // ---- W4-f32: TryToPoop + PlayGuitarChord ----

    #[test]
    fn f32_try_to_poop_gates_and_stack_clamp() {
        // `Player.cs:18896-18940`: wellFed off → no draw; the Next(600)
        // gate misses → no luck draw; a hit with tier 207 and buffTime
        // 7200 → clamp(7200/3600*3, 3, 999) = 6.
        let mut rng = tr_core::UnifiedRandom::new(1);
        assert_eq!(try_to_poop_stack(&mut rng, 0.0, false, false, Some((207, 7200))), None);
        assert_eq!(rng, tr_core::UnifiedRandom::new(1), "wellFed off: no draw");
        // Find a seed whose Next(600) hits 0.
        let mut hit = None;
        for seed in 0..5000i32 {
            let mut probe = tr_core::UnifiedRandom::new(seed);
            if probe.next_max(600) == 0 {
                hit = Some(seed);
                break;
            }
        }
        let seed = hit.expect("a Next(600) hit");
        let mut a = tr_core::UnifiedRandom::new(seed);
        let stack = try_to_poop_stack(&mut a, 0.0, true, false, Some((207, 7200)));
        assert_eq!(stack, Some(6), "7200/3600*3 = 6, no luck draw at 0");
        let mut b = tr_core::UnifiedRandom::new(seed);
        let _ = b.next_max(600);
        assert_eq!(a, b, "luck 0 consumes only the Next(600)");
        // Tipsy thirds the gate (`:18903-18906`): Next(200).
        let mut hit200 = None;
        for seed in 0..5000i32 {
            let mut probe = tr_core::UnifiedRandom::new(seed);
            if probe.next_max(200) == 0 {
                hit200 = Some(seed);
                break;
            }
        }
        let seed200 = hit200.expect("a Next(200) hit");
        let mut c = tr_core::UnifiedRandom::new(seed200);
        // Tier 26, buffTime 3600 → clamp(1*1, 1, 999) = 1.
        assert_eq!(
            try_to_poop_stack(&mut c, 0.0, true, true, Some((26, 3600))),
            Some(1)
        );
        // The clamp floor: buffTime 60 → 60/3600*3 = 0 → clamped to 3.
        let mut d = tr_core::UnifiedRandom::new(seed);
        assert_eq!(
            try_to_poop_stack(&mut d, 0.0, true, false, Some((207, 60))),
            Some(3)
        );
        // The clamp cap: buffTime 3_600_000 → 3000 → 999.
        let mut e = tr_core::UnifiedRandom::new(seed);
        assert_eq!(
            try_to_poop_stack(&mut e, 0.0, true, false, Some((207, 3_600_000))),
            Some(999)
        );
        // No fed tier: the Next(600) still draws, nothing drops.
        let mut f = tr_core::UnifiedRandom::new(seed);
        assert_eq!(try_to_poop_stack(&mut f, 0.0, true, false, None), None);
    }

    #[test]
    fn f32_try_to_poop_bad_luck_boosts_the_stack() {
        // `:18928-18931`: RollOnlyBadLuckExtreme(10) — luck -1.0 always
        // hits the gate, Next(10/10) = Next(1) = 0 → the ×1.2 boost
        // (f64 truncation). 6 → (6*1.2) = 7.
        let mut hit = None;
        for seed in 0..5000i32 {
            let mut probe = tr_core::UnifiedRandom::new(seed);
            if probe.next_max(600) == 0 {
                hit = Some(seed);
                break;
            }
        }
        let seed = hit.expect("a Next(600) hit");
        let mut a = tr_core::UnifiedRandom::new(seed);
        let stack = try_to_poop_stack(&mut a, -1.0, true, false, Some((207, 7200)));
        assert_eq!(stack, Some(7), "6 * 1.2 = 7.2 → 7");
        // The draw shape: Next(600), NextFloat gate, Next(1).
        let mut b = tr_core::UnifiedRandom::new(seed);
        let _ = b.next_max(600);
        let _gate = b.next_float();
        let _one = b.next_max(1);
        assert_eq!(a, b);
        // Positive luck: NO draw, no boost (the -1 return).
        let mut c = tr_core::UnifiedRandom::new(seed);
        assert_eq!(
            try_to_poop_stack(&mut c, 0.9, true, false, Some((207, 7200))),
            Some(6)
        );
        let mut d = tr_core::UnifiedRandom::new(seed);
        let _ = d.next_max(600);
        assert_eq!(c, d, "positive luck draws nothing extra");
    }

    #[test]
    fn f32_guitar_chord_ladder_and_luck_pitch() {
        // `Player.cs:46671-46703`: the six-range cue ladder, the luck roll
        // FIRST (no draw at luck >= 0), the pitch Next(-50,51)*0.01 on a
        // bad-luck gate hit.
        let mut rng = tr_core::UnifiedRandom::new(1);
        assert_eq!(guitar_chord_cue(&mut rng, 0.0, 1.0).0, "Item49");
        assert_eq!(guitar_chord_cue(&mut rng, 0.0, 0.8).0, "Item48");
        assert_eq!(guitar_chord_cue(&mut rng, 0.0, 0.6).0, "Item47");
        assert_eq!(guitar_chord_cue(&mut rng, 0.0, 0.4).0, "Item51");
        assert_eq!(guitar_chord_cue(&mut rng, 0.0, 0.2).0, "Item52");
        assert_eq!(guitar_chord_cue(&mut rng, 0.0, 0.1).0, "Item50");
        assert_eq!(rng, tr_core::UnifiedRandom::new(1), "luck 0: no draws");
        // The boundary values are strict `>`: range == 5/6 falls to Item48.
        assert_eq!(guitar_chord_cue(&mut rng, 0.0, 5.0 / 6.0).0, "Item48");
        // Bad luck: gate + Next(1) + the pitch draw.
        let mut a = tr_core::UnifiedRandom::new(5);
        let (cue, pitch) = guitar_chord_cue(&mut a, -1.0, 1.0);
        assert_eq!(cue, "Item49");
        let mut b = tr_core::UnifiedRandom::new(5);
        let _gate = b.next_float();
        let _one = b.next_max(1);
        let want_pitch = b.next_range(-50, 51) as f32 * 0.01;
        assert_eq!(pitch, want_pitch);
        assert_eq!(a, b);
        assert!((-0.5..=0.5).contains(&pitch));
    }

    #[test]
    fn f32_item_sound_style_index_matches_the_table() {
        // The ctor-order table: Item1 at 126 (the Copper Pickaxe's
        // use_sound), Item47 at 172 — the chord cues resolve.
        assert_eq!(item_sound_style_index("Item1"), Some(126));
        assert_eq!(item_sound_style_index("Item47"), Some(172));
        assert_eq!(item_sound_style_index("Item49"), Some(174));
        assert_eq!(item_sound_style_index("Item52"), Some(177));
        assert_eq!(item_sound_style_index("NPCHit1"), None);
        for (cue, idx) in [("Item47", 172u16), ("Item48", 173), ("Item49", 174), ("Item50", 175), ("Item51", 176), ("Item52", 177)] {
            assert_eq!(crate::soundid::SOUND_STYLES[idx as usize].name, cue);
        }
    }
}
