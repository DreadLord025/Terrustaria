//! Tile and wall editing (plan stage A4): the per-frame mining/placing loop,
//! the break/place helpers behind it and the replicated-edit applier the net
//! layer feeds.

use glam::Vec2;
use tr_core::tiledata as td;
use tr_core::{TILE_PX, UnifiedRandom, World, edit};

use crate::achievements;
use crate::app::App;
use crate::item::ItemSlot;
use crate::npc::{Npc, NpcKind};
use crate::player::Player;
use crate::projectile::Projectile;
use crate::registry;
use tr_core::loot::DropRng;

use super::items::spawn_item_drop;

mod wiring;
pub(crate) use wiring::use_wiring_tool;
mod doors;
use doors::interact_tile;
pub(crate) use doors::{bed_spawn_valid, check_bed_spawn};
mod sentry;

/// One 60 Hz tick in seconds — the fixed step the swing/item timers are paced
/// against (`Player.Update`'s tick, `frame.rs`'s `FixedStepper::new(1.0 / 60.0)`).
const ONE_TICK_SECS: f32 = 1.0 / 60.0;

// ---------------------------------------------------------------------------
// S1-P-i22 — the magic guns + tomes' use arms (`Player.ItemCheck_Shoot`,
// `Player.cs:47056-49468`). The census + verdicts live in
// `docs/research/56-magic-guns-tomes-census.md`; every arm below cites its
// C# block and keeps the `Main.rand` draw order on the `UpdatePlayers`
// stream (the use pass's `loot_rng`).
// ---------------------------------------------------------------------------

/// `Utils.Remap(fromValue, fromMin, fromMax, toMin, toMax)` (`Utils.cs:345-348`)
/// — `Lerp(toMin, toMax, GetLerpValue(fromMin, fromMax, fromValue, clamped:
/// true))`. The harp's note-speed curve (`Player.cs:47407`) uses the clamped
/// default; the descending-range branches are [`crate::projectile::get_lerp_value`]'s.
fn remap_clamped(value: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
    to_min + (to_max - to_min) * crate::projectile::get_lerp_value(from_min, from_max, value)
}

/// `Camera.SmallerScaledAxis` (`Graphics/Camera.cs:29-40`): the smaller of
/// `ScaledSize`'s axes, `ScaledSize = UnscaledSize - GameViewMatrix.Translation
/// * 2` (`Camera.cs:14`). The translation is the zoom frame's offset
/// (`SpriteViewMatrix.cs:149-153`, the `app/frame/draw.rs:735-746` formula:
/// `Round(screen/2 - (screen/2) / Round(zoom/(1/128))*(1/128))`, banker's
/// rounding). The Magical Harp's distance scale reads it (`Player.cs:47395`).
/// Takes the screen + zoom by value — the use pass calls it under the live
/// `app.player`/`app.rngs` borrows (the disjoint-field convention).
fn smaller_scaled_axis(screen: Vec2, zoom: f32) -> f32 {
    let (sw, sh) = (screen.x, screen.y);
    let q = (zoom / (1.0 / 128.0)).round_ties_even() * (1.0 / 128.0);
    let tx = (sw * 0.5 - sw * 0.5 / q).round_ties_even();
    let ty = (sh * 0.5 - sh * 0.5 / q).round_ties_even();
    (sw - tx * 2.0).min(sh - ty * 2.0)
}

/// `Player.tileTargetX/Y` (`Player.cs:25389-25410`): the cursor tile clamped
/// to `[5, maxTiles-5]` per axis (the gravDir −1 mirror has no port). The
/// Electrosphere Launcher's missile anchors on it (`Player.cs:49388-49392`).
fn tile_target(world: &World, world_pos: Vec2) -> (i32, i32) {
    let tx = ((world_pos.x / TILE_PX) as i32).clamp(5, world.width - 5);
    let ty = ((world_pos.y / TILE_PX) as i32).clamp(5, world.height - 5);
    (tx, ty)
}

/// `Utils.NextVector2Circular(halfWidth, halfHeight)` (`Utils.cs:1266-1269`):
/// `NextVector2Unit() * (halfWidth, halfHeight) * NextFloat()` — one angle
/// draw then one radial draw, in that order.
fn next_vector2_circular(rng: &mut UnifiedRandom, hw: f32, hh: f32) -> Vec2 {
    let angle = rng.next_float() * std::f32::consts::TAU;
    Vec2::from_angle(angle) * Vec2::new(hw, hh) * rng.next_float()
}

/// `Utils.NextVector2CircularEdge(halfWidth, halfHeight)` (`Utils.cs:1271-1274`):
/// `NextVector2Unit() * (halfWidth, halfHeight)` — one angle draw, no radial.
fn next_vector2_circular_edge(rng: &mut UnifiedRandom, hw: f32, hh: f32) -> Vec2 {
    let angle = rng.next_float() * std::f32::consts::TAU;
    Vec2::from_angle(angle) * Vec2::new(hw, hh)
}

/// `Player.GetSparkleGuitarTarget` (`Player.cs:46626-46643`): the chaseable
/// NPCs whose hitbox intersects the 1000×800 box centred on the player
/// (`Utils.CenteredRectangle(Center, (1000, 800))`). Returns the indices into
/// `npcs` in scan order (the C#'s `validTargets` list order — the
/// `Main.rand.Next(Count)` pick reads it). Takes the player centre + the NPC
/// slice — the use pass calls it under the live `app.player`/`app.rngs`
/// borrows (the disjoint-field convention).
fn sparkle_guitar_targets(center: Vec2, npcs: &[Npc]) -> Vec<usize> {
    let (cx, cy) = (center.x, center.y);
    let (bx0, by0, bx1, by1) = (cx - 500.0, cy - 400.0, cx + 500.0, cy + 400.0);
    let mut out = Vec::new();
    for (i, n) in npcs.iter().enumerate() {
        if !n.can_be_chased_by() {
            continue;
        }
        let (w, h) = n.size();
        // `Rectangle.Intersects`: the strict-overlap test on both axes.
        if n.x < bx1 && n.x + w > bx0 && n.y < by1 && n.y + h > by0 {
            out.push(i);
        }
    }
    out
}

/// `AmmoID.Arrow` (40) and `AmmoID.Stake` (1836) (`ID/AmmoID.cs:161`, `:174`) —
/// the two categories `AmmoID.Sets.IsArrow` covers (`:115`), the gate the
/// `magicQuiver` knockback arm reads (`Player.cs:53547`).
const AMMO_ARROW: i32 = 40;
const AMMO_STAKE: i32 = 1836;

/// `ItemID.Sets.gunProj` (`ID/ItemID.cs:1327`) — the four endgame shooters
/// (`ItemID.cs`: 3475 VortexBeater, 3540 Phantasm, 3854 DD2PhoenixBow, 3930
/// Celeb2). `ItemCheck_Shoot` passes `dontConsume: true` for each
/// (`Player.cs:47071`), so their `PickAmmo` never drains; the call's result is
/// then thrown away — the shot's stats reset to the weapon's own raw values
/// (`:47073-47078`) and the spawn fires a FIXED gun projectile instead of the
/// resolved one (`:48986-49018`): 3475→615, 3930→714, 3540→630, 3854→705.
///
/// The scaled registry carries PLACEHOLDER rows for all four (identical
/// `use_ammo: 0`, damage 15, anim 25/25, Melee, `auto_reuse: false` — the
/// 683-row stand-in class M3-ITEM-5 documented), so their vanilla data is keyed
/// here on the item id, exactly like `ammo::not_consume`'s item-keyed arms; the
/// `tools/GenItemData` fix is a separate follow-up (ANALYSIS records it, Phase
/// 5). The port never routes them through the generic `Item.shoot` path either:
/// `ItemCheck_Shoot` runs `PickAmmo` for every `useAmmo > 0` item
/// (`Player.cs:47069`).
#[derive(Clone, Copy, Debug)]
struct GunProj {
    /// The key: the vanilla item id (`ItemID.cs`).
    kind: u16,
    /// `Item.shoot`'s fixed replacement (`Player.cs:48991`, `:48998`, `:49005`,
    /// `:49015`).
    projectile: u16,
    /// `Item.useAmmo` (`Item.cs:48996`): the ammo the have-ammo gate still
    /// requires.
    use_ammo: i32,
    /// `Item.damage` — the shot's raw damage (`:47075`).
    damage: i32,
    /// `Item.shootSpeed` — the shot's raw speed (`:47077`).
    shoot_speed: f32,
    /// `Item.knockBack` — the shot's raw knockback (`:47074`).
    knock_back: f32,
    /// `Item.useTime` (`Item.cs:49028`).
    use_time: i32,
    /// `Item.useAnimation` (`Item.cs:49029`) — the swing length and the cycler
    /// cooldown's base (`Player.cs:53570`).
    use_animation: i32,
    /// `Item.autoReuse` (`Item.cs:48997`).
    auto_reuse: bool,
    /// The item's damage class (`this.ranged = true` in all four case bodies) —
    /// the stat-multiplier and crit source the shot reads.
    class: registry::DamageClass,
}

/// The four rows above, keyed on the vanilla item id — the same id-keyed
/// convention `not_consume`'s chain uses (`ammo.rs`).
const GUN_PROG: [GunProj; 4] = [
    // 3475 Vortex Beater (`Item.cs:31159-31178`).
    GunProj {
        kind: 3475,
        projectile: 615,
        use_ammo: 97,
        damage: 50,
        shoot_speed: 20.0,
        knock_back: 2.0,
        use_time: 20,
        use_animation: 20,
        auto_reuse: true,
        class: registry::DamageClass::Ranged,
    },
    // 3540 Phantasm (`Item.cs:31728-31747`).
    GunProj {
        kind: 3540,
        projectile: 630,
        use_ammo: 40,
        damage: 50,
        shoot_speed: 20.0,
        knock_back: 2.0,
        use_time: 12,
        use_animation: 12,
        auto_reuse: true,
        class: registry::DamageClass::Ranged,
    },
    // 3854 DD2 Phoenix Bow (`Item.cs:34048-34065`).
    GunProj {
        kind: 3854,
        projectile: 705,
        use_ammo: 40,
        damage: 32,
        shoot_speed: 20.0,
        knock_back: 2.0,
        use_time: 18,
        use_animation: 18,
        auto_reuse: true,
        class: registry::DamageClass::Ranged,
    },
    // 3930 Celebration Mk2 (`Item.cs:34741-34748`).
    GunProj {
        kind: 3930,
        projectile: 714,
        use_ammo: 771,
        damage: 50,
        shoot_speed: 17.0,
        knock_back: 10.0,
        use_time: 6,
        use_animation: 6,
        auto_reuse: true,
        class: registry::DamageClass::Ranged,
    },
];

/// The `gunProj` row for the held item, if it is one of the four.
fn gun_proj_row(kind: u16) -> Option<&'static GunProj> {
    GUN_PROG.iter().find(|g| g.kind == kind)
}

/// S1-P-i16 — `Player.DestroyOldestProximityMinesOverMinesCap`
/// (`Player.cs:46976-47017`, called from `ItemCheck_Shoot`'s 760 arm
/// `:49450-49453`): while the owner's live proximity mines (the twelve
/// kinds `:46986-46997`) exceed the cap, the LOWEST-timeLeft mine dies
/// (the C#'s strict `<` scan keeps the first on ties). The C# `Kill()`
/// (`:47013`) runs the aiStyle-16 explosion inline — the port latches
/// `detonated` so the pool's end-of-tick explosion pass resolves it (the
/// `ai_bomb` fuse convention).
fn destroy_oldest_proximity_mines_over_cap(app: &mut App, owner: u8, cap: usize) {
    const MINES: [u16; 12] = [135, 138, 141, 144, 778, 782, 786, 789, 792, 795, 798, 801];
    loop {
        // The C# collects the owner's mines, then kills the oldest until
        // the count fits. The port re-scans per kill (the list is the
        // pool's live slots — the same set, the kill removes one).
        let mut count = 0usize;
        let mut oldest: Option<usize> = None;
        let mut oldest_time = i32::MAX;
        for (i, p) in app.projectiles.slots().iter().enumerate() {
            // Gate fix (i16 infinite loop): a mine already latched for
            // detonation is the C#'s REMOVED list entry (`Kill()` ran,
            // `Player.cs:47012` — the slot dies on the pool's next
            // explosion pass). Counting it again re-picks the same oldest
            // forever; exclude the detonated slots so the loop terminates.
            if p.active && !p.detonated && p.owner == owner && MINES.contains(&p.kind) {
                count += 1;
                if p.time_left < oldest_time {
                    oldest_time = p.time_left;
                    oldest = Some(i);
                }
            }
        }
        if count <= cap {
            break;
        }
        match oldest {
            Some(i) => app.projectiles.detonate(i),
            None => break,
        }
    }
}

/// The shooter's effective `sItem.useAmmo` (`Item.cs:48996`): the registry's
/// value, or the id-keyed vanilla one for the four `gunProj` items whose scaled
/// rows are placeholders. This is the gate `ItemCheck_Shoot` runs `PickAmmo`
/// behind (`Player.cs:47069`).
fn shooter_use_ammo(kind: u16) -> i32 {
    gun_proj_row(kind).map_or_else(
        || registry::get(kind).map_or(0, |d| d.use_ammo),
        |g| g.use_ammo,
    )
}

/// The shooter's `Item.autoReuse` (`Item.cs:48997`) — the registry's value, or
/// the id-keyed vanilla one for the four `gunProj` placeholders.
fn shooter_auto_reuse(kind: u16) -> bool {
    gun_proj_row(kind).map_or_else(
        || registry::get(kind).is_some_and(|d| d.auto_reuse),
        |g| g.auto_reuse,
    )
}

/// The shooter's instance `useAnimation` in ticks — `sItem.useAnimation`
/// (`Item.cs:49029`, prefix-scaled `Item.cs:577`), NOT `itemAnimationMax`:
/// `ApplyItemAnimation`'s style-5 `num` bump (`Player.cs:4466-4469`) makes the
/// max one tick longer, but the cycler cooldown (`:53571`) and the Fairy
/// Queen's save window (`:53635`) read the raw instance value. The four
/// `gunProj` placeholders take their id-keyed vanilla row.
fn shooter_use_anim_ticks(kind: u16, prefix: u16) -> i32 {
    gun_proj_row(kind).map_or_else(
        || {
            crate::prefix::prefixed_stats(kind, prefix)
                .map(|s| s.use_animation)
                .or_else(|| registry::get(kind).map(|d| d.use_animation as i32))
                .unwrap_or(0)
        },
        |g| g.use_animation,
    )
}

/// The shooter's action cadence in seconds — `Player.ApplyItemTime`
/// (`Player.cs:4416-4418` → `SetItemTime(sItem.useTime)`, `:4416-4419`):
/// `Item.useTime` (`Item.cs:49028`) read off the PREFIXED instance, with the
/// same fallback the swing start uses (a kind with no `useTime` paces off its
/// whole animation — the C#'s `Item.useTime` stock is 100,
/// `Item.cs:49028`-adjacent), and the vanilla value for the four `gunProj`
/// placeholders.
fn shooter_use_time_secs(kind: u16, prefix: u16, swing_secs: f32) -> f32 {
    gun_proj_row(kind).map_or_else(
        || {
            crate::prefix::prefixed_stats(kind, prefix)
                .map(|s| s.use_time as f32 * ONE_TICK_SECS)
                .filter(|t| *t > 0.0)
                .unwrap_or(swing_secs)
        },
        |g| g.use_time as f32 * ONE_TICK_SECS,
    )
}

/// The C#'s INTEGER tick comparisons out of the port's per-frame seconds
/// timers: `round(secs * 60)` at the 60 Hz fixed step. The C# reads
/// `itemAnimation` / `itemAnimationMax` / `useTime` as integers
/// (`Player.cs:4066`, `:53635`, `:53663`) while `app.swing` / `app.item_time`
/// decay by the frame's `dt`, so every window the roll chain reads is derived
/// from these counts — never from a raw float compare against "eight ticks",
/// which would drift with the frame time.
fn ticks_of(secs: f32) -> i32 {
    (secs * 60.0).round() as i32
}

/// `SetItemAnimation(int baseFrames, float multiplier)` (`Player.cs:4453-4461`):
/// `val = (int)((float)baseFrames * multiplier)` — truncation, NOT rounding —
/// floored at 1 when `baseFrames > 0`. Seconds out of the tick count.
fn scaled_anim_secs(base_frames: i32, multiplier: f32) -> f32 {
    let mut val = (base_frames as f32 * multiplier) as i32;
    if base_frames > 0 {
        val = val.max(1);
    }
    val as f32 * ONE_TICK_SECS
}

/// `ItemID.Sets.SummonerWeaponThatScalesWithAttackSpeed` (`ItemID.cs:497`):
/// the whip family whose `ApplyItemAnimation` rides the summoner attack-speed
/// multiplier (`Player.cs:4474-4477`).
fn summoner_weapon_scales_with_attack_speed(kind: u16) -> bool {
    matches!(
        kind,
        4672 | 4679
            | 4680
            | 4678
            | 4913
            | 4912
            | 4911
            | 4914
            | 5074
            | 5473
            | 5474
            | 5475
            | 5476
            | 5477
            | 5478
            | 5479
            | 5480
            | 5688
    )
}

/// `Player.ApplyItemAnimation` (`Player.cs:4463-4497`): the swing length off
/// the PREFIXED instance's `useAnimation`, through the branch chain —
/// melee (unless `ItemID.Sets.NoMeleeSpeedBonus`, the Ale Throwing Glove
/// 3821, `ItemID.cs:340`) scales by the inverted `meleeSpeed`; the scaling
/// whips by `summonerWeaponSpeedBonus * whipUseTimeMultiplier`; the ManaV2
/// slow cast (`:4478-4483`, S1-P-i19 — `DebugOptions.ManaV2` is the SHIPPED
/// true, `DebugOptions.cs:85`) stretches a mana weapon's swing ×2.5 THEN
/// adds the `num` bump; `createTile` items by `tileSpeed`, `createWall`
/// items by `wallSpeed`; everything else keeps the raw animation plus the
/// `num` bump — `autoReuse && reuseDelay == 0 && useTime <= useAnimation &&
/// shoot > 0 && useStyle == 5` adds ONE frame (`:4466-4469`) to the
/// non-melee/summon/tile/wall branches. The tail (`:4496`) re-arms
/// `reuseDelay = sItem.reuseDelay` — the caller applies it through
/// [`arm_reuse_delay`].
fn apply_item_animation_secs(app: &App, held: ItemSlot) -> f32 {
    let Some(def) = registry::get(held.kind) else {
        return crate::item::use_animation_secs_of(held.kind, held.prefix);
    };
    let fx = crate::prefix::prefixed_stats(held.kind, held.prefix);
    // The registry rows stock `u8`; the prefixed instance and the C# `int`
    // fields are `i32` — widen the fallback so the two arms agree.
    let use_anim = fx.map_or(def.use_animation as i32, |s| s.use_animation);
    let use_time = fx.map_or(def.use_time as i32, |s| s.use_time);
    let reuse_delay = fx.map_or(def.reuse_delay, |s| s.reuse_delay);
    let shoot = def.projectile.map_or(0, |p| p as i32);
    let num = if def.auto_reuse
        && reuse_delay == 0
        && use_time <= use_anim
        && shoot > 0
        && def.use_style == 5
    {
        1
    } else {
        0
    };
    let p = app.player.as_ref();
    if def.melee && held.kind != 3821 {
        // `:4470-4473`: `SetItemAnimation(useAnimation, meleeSpeed)` — the
        // INVERTED, held-item-scaled multiplier (`:25912` + `:29021-29039`).
        let mult = p.map_or(1.0, |p| p.melee_speed(held.kind));
        scaled_anim_secs(use_anim, mult)
    } else if def.summon && summoner_weapon_scales_with_attack_speed(held.kind) {
        // `:4474-4477`: the summoner multiplier already carries the
        // `whipUseTimeMultiplier` factor (`CapAttackSpeeds` `:29025` +
        // `Player.cs:4476`).
        let mult = p.map_or(1.0, |p| p.summoner_weapon_speed_multiplier(held.kind));
        scaled_anim_secs(use_anim, mult)
    } else if def.mana > 0 && p.is_some_and(|p| p.slow_magic_use) {
        // `:4478-4483`: `DebugOptions.ManaV2 && sItem.mana > 0 &&
        // slowMagicUse` — `SetItemAnimation(useAnimation, 2.5)` (int
        // truncation, floor 1) THEN `itemAnimation += num` (the bump rides
        // AFTER the scale, not before — the i19 resync fold into the i12
        // branch chain).
        let base = scaled_anim_secs(use_anim, crate::regen::SLOW_MAGIC_MULTIPLIER);
        base + num as f32 * ONE_TICK_SECS
    } else if def.tile.is_some() {
        // `:4484-4487`: `createTile >= 0` — the tileSpeed branch.
        let mult = p.map_or(1.0, |p| p.tile_speed_multiplier());
        scaled_anim_secs(use_anim, mult)
    } else if def.wall.is_some() {
        // `:4488-4491`: `createWall >= 0` — the wallSpeed branch.
        let mult = p.map_or(1.0, |p| p.wall_speed_multiplier());
        scaled_anim_secs(use_anim, mult)
    } else {
        // `:4492-4495`: `SetItemAnimation(useAnimation + num)`.
        (use_anim + num) as f32 * ONE_TICK_SECS
    }
}

/// `ApplyItemAnimation`'s tail (`Player.cs:4496`): `reuseDelay =
/// sItem.reuseDelay` — the PREFIXED instance value (`Item.cs:579` scales it
/// with `spd`). The C# never decrements the lock per tick; `ApplyReuseDelay`
/// (`:52835-52840`) consumes it whole at the next animation-zero head,
/// converting it into a silent `itemAnimation = itemTime = reuseDelay` hold.
fn arm_reuse_delay(app: &mut App, held: ItemSlot) {
    let rd = crate::prefix::prefixed_stats(held.kind, held.prefix)
        .map(|s| s.reuse_delay)
        .or_else(|| registry::get(held.kind).map(|d| d.reuse_delay))
        .unwrap_or(0);
    app.reuse_delay = rd as f32 * ONE_TICK_SECS;
}

/// The use-start timer pair — `ItemCheck_StartActualUse` (`Player.cs:51678-51680`):
/// `attackCD = 0` then `ApplyItemAnimation`. The port's `attack_cd` carries the
/// swing length as the use-start lock (the C# gate is the animation itself;
/// `attack_cd` decays in `world_sim/npcs.rs` alongside the other per-tick
/// timers, so the unit rigs that drive `update_editing` alone see ONE use per
/// press — the same stand-in the pre-i12 gates used). The reuse lock rides the
/// same start (`:4496`).
fn start_use_timers(app: &mut App, held: ItemSlot) {
    app.swing_max = apply_item_animation_secs(app, held);
    app.swing = app.swing_max;
    start_swing_extras(app, held.kind);
    app.attack_cd = app.swing_max;
    arm_reuse_delay(app, held);
}

/// The `Player.ApplyItemAnimation` tail (`Player.cs:4449-4451`): the swing
/// carries the item's `useStyle` and the current aim angle — the per-style
/// body dispatch and the held-item draw read both.
pub(crate) fn start_swing_extras(app: &mut App, held_kind: u16) {
    app.swing_style = crate::item::use_style(held_kind);
    let (sw, sh) = app.screen_size();
    let mouse_world = Vec2::new(
        app.camera.x + (app.mouse_pos.x - sw * 0.5) / app.zoom,
        app.camera.y + (app.mouse_pos.y - sh * 0.5) / app.zoom,
    );
    let (cx, cy) = app
        .player
        .as_ref()
        .map(|p| p.center())
        .unwrap_or((app.camera.x, app.camera.y));
    app.swing_aim = (mouse_world - Vec2::new(cx, cy))
        .y
        .atan2((mouse_world - Vec2::new(cx, cy)).x);
    // `Player.ItemCheck_StartActualUse` (`Player.cs:51681-51698`) plays the
    // item's `UseSound` at `base.Center` right after `ApplyItemAnimation`
    // (`:51680`) — this seam is the port's use-start (every swing/shot/
    // consume arm calls it). Single-player: the `whoAmI == Main.myPlayer`
    // arm always plays (`:51694`); the net-sync arm (`:51690-51692`) has no
    // port. The case-2 `Main.rand` draws ride the `UpdatePlayers` stream
    // (`Player.ItemCheck` under the `Main.cs:17754` swap) and run only when
    // the attenuation gate passed — inside `play_use_sound`.
    if let Some((style_index, pitch_offset)) = crate::item::initial_use_sound(held_kind) {
        app.audio.play_use_sound(
            style_index,
            cx,
            cy,
            pitch_offset,
            app.rngs.stream(crate::app::RngKey::UpdatePlayers),
        );
    }
}

/// The gem-bolt kinds the gem-staff shoot fires (`Player.cs:49262`'s kind
/// gate): 121-126 plus the amber bolt 597.
const GEM_STAFF_BOLTS: [u16; 7] = [121, 122, 123, 124, 125, 126, 597];

/// S1-P-i21 — the 12 hardmode magic staves of the `ItemCheck_Shoot` census
/// (`ItemID.cs`: RainbowRod 495, CrystalStorm 518, CursedFlames 519, NimbusRod
/// 1244, MagnetSphere 1266, GoldenShower 1336, ShadowbeamStaff 1444,
/// InfernoFork 1445, SpectreStaff 1446, BatScepter 1801, Razorpine 1930,
/// BlizzardStaff 1931). Gate-merge note: i20's GENERIC non-ammo
/// `ItemTimeIsZero` volley gate (the C#-true `:43544-43570` cadence with the
/// coins/shootsEveryUse/channel exclusions) supersedes the i21 staff-list
/// gate — the same `fire_non_ammo_shot` chain serves both, and i21's four
/// dedicated arms live inside it. Of this list only the GoldenShower
/// (`useTime 6 < useAnimation 18`, `Item.cs:16545-16546`) and the
/// BlizzardStaff (`5 < 10`, `:20976-20977`) volley mid-swing; the other ten
/// pace `useTime == useAnimation` (one shot per swing). Kept as the census
/// record (the anchors row cites it).
#[allow(dead_code)]
const HARDMODE_MAGIC_STAVES: [u16; 12] = [
    495, 518, 519, 1244, 1266, 1336, 1444, 1445, 1446, 1801, 1930, 1931,
];
/// `Player.ItemCheck_Shoot`'s whip arm (`Player.cs:48655-48690`): spawn one
/// whip projectile with the per-throw spin seed, the Wicked Armlet's +1
/// knockback, and — for the Snake Band's `withAudioVisualFeedback = false`
/// refire (`snake_refire`) — the rotated velocity, the `ai2` timeToFlyOut
/// override and the damage/knockback cut. The audible swing bumps the
/// repeat counter (`:48684-48688`).
///
/// RNG order (the `UpdatePlayers` stream, `Main.cs:17754`): the spin seed
/// `NextFloat` (`:48657`), then — for every whip but the Dark Harvest (4680)
/// — the `Next(3)` flip roll (`:48658`); the refire adds the
/// `RotatedByRandom` pair (`:48670`, two `NextDouble`s, `Utils.cs:1697-1700`).
/// The whip never self-crits (`DefaultToWhip` sets no class flag, ANALYSIS).
#[allow(clippy::too_many_arguments)]
fn spawn_whip_shot(
    projectiles: &mut crate::projectile::ProjectilePool,
    player: Option<&mut Player>,
    loot_rng: &mut UnifiedRandom,
    held: ItemSlot,
    def: &crate::registry::ItemDef,
    held_fx: Option<crate::prefix::PrefixedStats>,
    proj: u16,
    spawn: Vec2,
    shot: Vec2,
    raw_damage: i32,
    fx_kb: f32,
    snake_refire: bool,
) {
    let mut spin = 0.6 + 0.4 * loot_rng.next_float();
    if held.kind != 4680 && loot_rng.next_range(0, 3) == 0 {
        spin *= -2.5;
    }
    let mut whip_kb = fx_kb;
    // `:48663-48666` — the Wicked Armlet adds +1 knockback.
    let wicked = player.as_ref().is_some_and(|p| p.acc_wicked_armlet);
    if wicked {
        whip_kb += 1.0;
    }
    let mut whip_vx = shot.x;
    let mut whip_vy = shot.y;
    let mut whip_ai2 = 0.0f32;
    let mut whip_damage = raw_damage;
    if snake_refire {
        // `:48670` — `RotatedByRandom(0.3141592741012573)` = π/10: two
        // `NextDouble` draws, the difference times the max radians.
        let angle = loot_rng.next_double() * 0.3141592741012573
            - loot_rng.next_double() * 0.3141592741012573;
        let rotated = Vec2::new(whip_vx, whip_vy).rotate(Vec2::from_angle(angle as f32));
        whip_vx = rotated.x;
        whip_vy = rotated.y;
        // `:48673` — `ai2 = 1000 + itemAnimationMax * 2`.
        let animation_max = held_fx.map_or(def.use_animation as f32, |s| s.use_animation as f32);
        whip_ai2 = 1000.0 + animation_max * 2.0;
        // `:48676-48681` — damage ×0.3 floored at 3, knockback ×0.25.
        whip_damage = ((whip_damage as f32) * 0.3) as i32;
        if whip_damage < 3 {
            whip_damage = 3;
        }
        whip_kb *= 0.25;
    }
    let slot = projectiles.spawn_with_data_ai(
        proj, spawn.x, spawn.y, whip_vx, whip_vy, whip_damage, whip_kb, 0.0, spin,
    );
    if let Some(slot) = slot {
        // (Resync spawn-crit conversion: the `set_crit(slot, false)` stamp
        // dropped — the whip's crit is PER-HIT via its tag effect's
        // `Next(100)` roll, `WhipTagEffect.cs:69`; no spawn draw, no stamp.)
        if snake_refire {
            projectiles.set_ai2(slot, whip_ai2);
        }
        // `ApplyStatsFromSource` (`Projectile.cs:11119-11141`): the tag row
        // off the firing item + the prefix's `bonusTagDamage`, plus
        // `AI_165_Whip`'s `timeToFlyOut` inputs (the prefixed
        // `itemAnimationMax`).
        let tag_type = if tr_core::whip::tag_effect(held.kind).is_some() {
            held.kind
        } else {
            0
        };
        let bonus_tag = held_fx.map_or(0, |s| s.tag_damage);
        let animation_max = held_fx.map_or(def.use_animation as f32, |s| s.use_animation as f32);
        projectiles.set_whip_params(slot, tag_type, bonus_tag, animation_max);
    }
    // `:48684-48688` — only the audible swing bumps the repeat counter and
    // re-arms the 120-tick reset.
    if !snake_refire {
        if let Some(p) = player {
            p.repeat_whip_swings += 1;
            p.repeat_whips_reset_cooldown = 120;
        }
    }
}

/// `Player.GetWeaponDamage` (`Player.cs:53421-53429`): `(int)(damage *
/// multiplier + 5E-06f)` — the epsilon guards the float truncation. The
/// whip fire path reads it (the generic shoot path's `raw` is the same
/// value; the epsilon is the C#'s own).
fn whip_weapon_damage(base_damage: i32, multiplier: f32) -> i32 {
    if base_damage <= 0 {
        return base_damage;
    }
    (base_damage as f32 * multiplier + 5e-6) as i32
}

use crate::projectile::GemStaffFeatures;

/// `Item.shootsEveryUse` (`Item.cs:3729/4595/5695/9297/9316/10229/15126/20230/
/// 42675`): these fire only on `ItemAnimationJustStarted` (`Player.cs:43560-43562`),
/// never the mid-swing volley. All have `useTime == useAnimation`, so the exclusion
/// is belt-and-suspenders, but it keeps the gate faithful to the C# `flag4` override.
const SHOOTS_EVERY_USE: [u16; 9] = [190, 273, 368, 674, 675, 757, 1226, 1826, 5382];

/// `Utils.Remap` (`Utils.cs`): the value's position in [from_min, from_max]
/// mapped onto [to_min, to_max] - the gem-staff spread's speed curve.
fn remap_gem(value: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
    let t = if from_max > from_min {
        ((value - from_min) / (from_max - from_min)).clamp(0.0, 1.0)
    } else {
        0.0
    };
    to_min + (to_max - to_min) * t
}

/// `Player.LimitPointToPlayerReachableArea` (`Player.cs:45447-45473`): the
/// point pulled into the 1920x1200 box centred on the player by the per-axis
/// scale. The box is `Utils.CenteredRectangle(Center, MaxWorldViewSize)`
/// (`Utils.cs:947-950` — the `(int)` corner truncation, the size stays
/// 1920x1200) clamped by `WorldUtils.ClampToWorldBorders`
/// (`WorldBuilding/WorldUtils.cs:24-31` → `Utils.Clamp(Rectangle, Rectangle)`
/// `Utils.cs:236-239`): the TOP-LEFT slides inside the 640-px-inset play area
/// (`GetWorldPlayArea` `:16-22`, `leftWorld/topWorld = 0`,
/// `rightWorld/bottomWorld = maxTiles * 16`, `WorldGen.cs:6553-6554`) while
/// the box KEEPS its size — on a world smaller than the view the clamp's
/// max < min and `Utils.Clamp<T>` (`:223-234`, max first) returns the max,
/// so the box hangs off the far edge exactly as the C#'s does (S1-P-i23).
fn limit_point_to_player_reachable_area(
    center: Vec2,
    point: Vec2,
    world: &tr_core::World,
) -> Vec2 {
    // `Utils.CenteredRectangle`: the int corner, the float half-size.
    let rx = (center.x - 1920.0 * 0.5) as i32;
    let ry = (center.y - 1200.0 * 0.5) as i32;
    let (rw, rh) = (1920i32, 1200i32);
    // `GetWorldPlayArea`: the 640-px inset on every side.
    let (world_w, world_h) = (world.width as i32 * 16, world.height as i32 * 16);
    let (bl, bt) = (640i32, 640i32);
    let (br, bb) = (world_w - 640, world_h - 640);
    // `Utils.Clamp<T>` — the max test FIRST (a max < min returns the max).
    let clamp = |v: i32, min: i32, max: i32| {
        if v > max {
            max
        } else if v < min {
            min
        } else {
            v
        }
    };
    let rx = clamp(rx, bl, br - rw);
    let ry = clamp(ry, bt, bb - rh);
    // `rectangle.Center.ToVector2()` — XNA's int centre (X + Width/2).
    let mid = Vec2::new((rx + rw / 2) as f32, (ry + rh / 2) as f32);
    let off = point - mid;
    let (hw, hh) = (rw as f32 * 0.5, rh as f32 * 0.5);
    let mut scale = 1.0f32;
    if off.x.abs() > hw {
        scale = scale.min(hw / off.x.abs());
    }
    if off.y.abs() > hh {
        scale = scale.min(hh / off.y.abs());
    }
    mid + off * scale
}

/// `Player.IsInTileInteractionRange` (`Player.cs:32025-32029`) →
/// `TileReachCheckSettings.GetTileRegion` (`TileReachCheckSettings.cs:45-54`)
/// → `Collision.InTileBounds` (`Collision.cs:3776-3783`): the rectangle
/// reach, NOT a circle. `LX = (int)(position.X / 16) - x`,
/// `HX = ceil((position.X + width) / 16) - 1 + x` (and the Y pair), where
/// `x`/`y` are `tileRangeX/Y + TB` clamped by `TileReachLimit = 20`
/// (`TileReachCheckSettings.cs:20-34`, `:81-85`). The port's player hitbox
/// is `WIDTH`×`height` at `(p.x, p.y)` (`player.rs`).
fn tile_in_reach_rect(p: &Player, tx: i32, ty: i32, range_x: i32, range_y: i32, tb: i32) -> bool {
    let x = (range_x + tb).min(20);
    let y = (range_y + tb).min(20);
    let lx = (p.x / TILE_PX) as i32 - x;
    let hx = ((p.x + crate::player::WIDTH) / TILE_PX).ceil() as i32 - 1 + x;
    let ly = (p.y / TILE_PX) as i32 - y;
    let hy = ((p.y + p.height) / TILE_PX).ceil() as i32 - 1 + y;
    tx >= lx && tx <= hx && ty >= ly && ty <= hy
}

/// The per-tick `Player.tileRangeX/Y` for the mining reach — the journey
/// doubling (`Player.cs:18815-18828`) + the Extendo Grip / Architect's
/// Gizmo Pack / Grand Design `+3/+2` (`equippedAnyTileRangeAcc`,
/// `Player.cs:13040-13044`, granters `:15267-15294`) + the held Toolbox
/// `+1/+1` (`Player.cs:14954-14958`). The `TileReachLimit` clamp rides
/// [`tile_in_reach_rect`]; the artisan bread's `TB = 4` is the placement
/// path's (`Player.cs:35671`), not the mining reach's.
fn mining_tile_range(app: &App) -> (i32, i32) {
    let Some(p) = app.player.as_ref() else {
        return (5, 3);
    };
    let mut x = p.tile_range_x() as i32;
    let mut y = p.tile_range_y() as i32;
    let range_acc = p.equipped[..10].contains(&2215)
        || p.equipped[..10].contains(&3061)
        || p.equipped[..10].contains(&5126);
    if range_acc {
        x += 3;
        y += 2;
    }
    if app.inventory[app.selected].kind == 1923 && app.inventory[app.selected].count > 0 {
        x += 1;
        y += 1;
    }
    (x, y)
}

/// The `GetPickaxeDamage` world inputs (`Player.cs:53902-54045`) from the
/// port's `World` + seed flags. `Main.getGoodWorld` = the For-the-Worthy
/// seed (`worldgen_job.rs:143`); `Main.infectedSeed` is always false in
/// 1.4.5.8 (`Main.cs:52266`/`:66541`).
fn pick_world(app: &App) -> tr_core::tool::PickWorld {
    let seeds = app.world.seeds;
    tr_core::tool::PickWorld {
        world_surface: app.world.world_surface,
        underworld_layer: app.world.height - 200,
        max_tiles_x: app.world.width,
        get_good: seeds.contains(tr_core::seeds::SeedFlags::FOR_THE_WORTHY),
        infected_seed: false,
        not_the_bees: seeds.contains(tr_core::seeds::SeedFlags::NOT_THE_BEES),
        no_traps: seeds.contains(tr_core::seeds::SeedFlags::NO_TRAPS),
        tenth_anniversary: seeds.contains(tr_core::seeds::SeedFlags::TENTH_ANNIVERSARY),
    }
}

/// `Player.PickTile` (`Player.cs:53757-53826`) + `PickTile_DetermineDamage`
/// (`:53828-53844`) + `GetPickaxeDamage` (`:53902-54045`): one pick hit on
/// the cell. The damage accumulates in the `HitTile` book; at `>= 100` the
/// tile breaks (`WorldGen.KillTile` → the port's `break_tile`), otherwise
/// the fail tick plays (`KillTile(fail: true)` — the port's per-tick
/// sound/dust is the recorded deviation, the damage still accumulates).
/// Returns whether the tile broke.
///
/// The `DoesPickTargetTransformOnKill` arm (`:53840-53843`, `:54047-54054`)
/// zeroes the damage when the hit would reach 100 on a transforming tile
/// (the grass/moss family); the `KillTile(fail: true)` conversion
/// (`WorldGen.cs:63715-63735`) then strips the grass to its soil. The port
/// applies the conversion here (the tile kind write + the framing refresh)
/// and clears the hit slot, so the next hit mines the bare soil.
fn pick_tile_hit(app: &mut App, tx: i32, ty: i32, pick_power: i32) -> bool {
    let Some(tile) = app.world.tile(tx, ty).copied() else {
        return false;
    };
    if !tile.active() {
        return false;
    }
    // `:53760-53763` — the mystic snake rope refuses the pick outright.
    if tile.kind == 504 {
        return false;
    }
    let w = pick_world(app);
    // `PickTile_DetermineDamage` (`:53830-53831`): the buffer index, then
    // the damage. The mannequin/weapon-rack redirects move the buffer slot
    // (`hitTile.UpdatePosition`, `:53990`/`:54004`/`:54010`); the port
    // re-derives the redirect target after the damage call.
    let buffer_index = app.hit_tile.hit_object(tx, ty, tr_core::hit_tile::TILE);
    let mut damage = tr_core::tool::pickaxe_damage(
        tx,
        ty,
        pick_power,
        tile.kind,
        i32::from(tile.frame_x),
        i32::from(tile.frame_y),
        &w,
        |x, y| app.world.tile(x, y).copied(),
    );
    // The redirect the damage model applied — move the buffer slot to the
    // cell the C# `UpdatePosition` landed on (the mannequin left cell, the
    // weapon-rack anchor). Only the plain redirects are tracked; the
    // racked-weapon refusal already zeroed the damage.
    let (redir_x, redir_y) = pick_redirect_target(&app.world, tx, ty, tile);
    if (redir_x, redir_y) != (tx, ty) {
        app.hit_tile.update_position(buffer_index, redir_x, redir_y);
    }
    // `:53832-53835` — a `CanKillTile` refusal zeroes the damage.
    if !edit::can_kill_tile(&app.world, tx, ty, chest_holds_items(app)) {
        damage = 0;
    }
    // `:53836-53839` — getGoodWorld doubles the damage.
    if w.get_good {
        damage *= 2;
    }
    // `:53840-53843` — the transforming-tile preview: if the hit would
    // reach 100 on a grass/moss tile, the damage zeroes (the conversion
    // fires instead of the break).
    let transforms = does_pick_target_transform(&app.world, tx, ty, tile.kind);
    // `:53840-53843` — the transforming-tile preview. The C# zeroes `damage`
    // when the hit WOULD reach 100, then `PickTile`'s `AddDamage(0)` returns
    // the standing total (<100) and falls to the `KillTile(fail: true)` arm —
    // which is where the grass/moss conversion actually lands
    // (`WorldGen.cs:63715-63735`). The port converts on the preview crossing
    // directly (the break-path total never reaches 100 once the preview zeroes
    // the damage), keeping the same observable: the cell turns to its soil on
    // the hit that would have broken it, no item drop.
    let would_break = if transforms {
        let preview = app.hit_tile.add_damage(buffer_index, damage, false);
        if preview >= 100 {
            damage = 0;
        }
        preview >= 100
    } else {
        false
    };
    let total = app.hit_tile.add_damage(buffer_index, damage, true);
    if total >= 100 || would_break {
        if transforms {
            // The conversion (`WorldGen.KillTile(fail: true)`'s grass/moss
            // arms, `WorldGen.cs:63715-63735`): the tile becomes its soil,
            // the hit slot clears, the framing refreshes. No item drop.
            convert_grass_tile(app, tx, ty, tile.kind);
            app.hit_tile
                .try_clearing_and_pruning(tx, ty, tr_core::hit_tile::TILE, |x, y| {
                    app.world.tile(x, y).is_some_and(|t| t.active())
                }, |x, y| app.world.tile(x, y).map_or(0, |t| t.wall));
            return false;
        }
        // `:53773` — `ClearMiningCacheAt` clears the hit + replacement
        // books at the cell (the port has no replacement book yet).
        app.hit_tile
            .try_clearing_and_pruning(tx, ty, tr_core::hit_tile::TILE, |x, y| {
                app.world.tile(x, y).is_some_and(|t| t.active())
            }, |x, y| app.world.tile(x, y).map_or(0, |t| t.wall));
        break_tile(app, tx, ty);
        // `:53822-53825` — the tail prune runs AFTER both branches (the
        // break tick prunes twice: ClearMiningCacheAt's own prune + this).
        if damage != 0 {
            app.hit_tile.prune(|x, y| {
                app.world.tile(x, y).is_some_and(|t| t.active())
            }, |x, y| app.world.tile(x, y).map_or(0, |t| t.wall));
        }
        return true;
    }
    // `:53815` — the fail tick (`KillTile(fail: true)`): the port's
    // per-tick sound/dust is the recorded deviation; the damage stays in
    // the book. `:53822-53825` — a non-zero damage prunes the book.
    if damage != 0 {
        app.hit_tile.prune(|x, y| {
            app.world.tile(x, y).is_some_and(|t| t.active())
        }, |x, y| app.world.tile(x, y).map_or(0, |t| t.wall));
    }
    false
}

/// The mannequin/weapon-rack redirect target `GetPickaxeDamage`'s
/// `UpdatePosition` calls land on (`Player.cs:53984-54043`) — the cell the
/// hit buffer should track. Plain tiles keep their own cell.
fn pick_redirect_target(world: &tr_core::World, tx: i32, ty: i32, tile: tr_core::tile::Tile) -> (i32, i32) {
    let fx = i32::from(tile.frame_x);
    let fy = i32::from(tile.frame_y);
    if tile.kind == 128 || tile.kind == 269 {
        if fx == 18 || fx == 54 {
            return (tx - 1, ty);
        }
        return (tx, ty);
    }
    if tile.kind == 334 {
        let mut x = tx;
        let mut y = ty;
        let mut cur_fx = fx;
        let mut cur_fy = fy;
        if cur_fy == 0 {
            y += 1;
            if let Some(t) = world.tile(x, y) {
                cur_fx = i32::from(t.frame_x);
                cur_fy = i32::from(t.frame_y);
            }
        }
        if cur_fy == 36 {
            y -= 1;
            if let Some(t) = world.tile(x, y) {
                cur_fx = i32::from(t.frame_x);
                cur_fy = i32::from(t.frame_y);
            }
        }
        let _ = cur_fy;
        if cur_fx < 5000 {
            x -= cur_fx / 18 % 3;
        }
        return (x, y);
    }
    (tx, ty)
}

/// `DoesPickTargetTransformOnKill`'s tile set (`Player.cs:54049`): the
/// grass/moss family that converts to its soil instead of breaking.
fn does_pick_target_transform(world: &tr_core::World, _x: i32, _y: i32, kind: u16) -> bool {
    let _ = world;
    matches!(kind, 2 | 477 | 492 | 23 | 60 | 70 | 109 | 199 | 662 | 661 | 633)
        || tr_core::tiledata::TILE_MOSS.has(kind)
        || tr_core::tiledata::SET_TILE_MOSS_BRICK.has(kind)
}

/// The `KillTile(fail: true)` grass/moss conversion
/// (`WorldGen.cs:63715-63735`): the grass kinds become dirt 0, the jungle
/// grass family becomes mud 59, the ash grass 633 becomes ash 57, the moss
/// becomes stone 1 and the moss brick becomes gray brick 38. The framing
/// refresh (`SquareTileFrame`) rides the port's `square_tile_frame_default`.
fn convert_grass_tile(app: &mut App, tx: i32, ty: i32, kind: u16) {
    let new_kind = if matches!(kind, 2 | 23 | 109 | 199 | 477 | 492) {
        0
    } else if kind == 633 {
        57
    } else if matches!(kind, 60 | 661 | 662 | 70) {
        59
    } else if tr_core::tiledata::TILE_MOSS.has(kind) {
        1
    } else if tr_core::tiledata::SET_TILE_MOSS_BRICK.has(kind) {
        38
    } else {
        return;
    };
    if let Some(t) = app.world.tile_mut(tx, ty) {
        t.kind = new_kind;
    }
    let _ = edit::square_tile_frame_default(&mut app.world, tx, ty);
    app.queue_map_ring(tx, ty);
    app.world_key = None;
}

/// `Player.ItemCheck_StartActualUse`'s channel start for the drills AND
/// chainsaws (`Player.cs:51677` → `StartChanneling(sItem)` `:52958-52968`):
/// a `channel` item with `pick > 0` (the drill family) or `axe > 0` (the
/// chainsaw family — S1-P-i30, `ItemID.Sets.IsChainsaw`, `ItemID.cs:495`)
/// spawns its aiStyle-20 projectile once on the press edge and latches
/// `Player.channel` (`DefaultToDrillOrChainsaw`, `Projectile.cs:10732-10742`
/// — ONE machine for both families). The projectile's aiStyle-20 machine
/// (`Projectile.cs:25481-25584`) holds the item animation via
/// `SetDummyItemTime(2)` (`:25563`) and kills itself on the channel release
/// (`:25549`). The port spawns the projectile here; the hold + release ride
/// `update_editing`'s channel block.
///
/// Returns whether a drill/chainsaw projectile is live for the local player
/// (the hold gate). The spawn is idempotent — a live tool of the held kind
/// is not re-spawned (the C# `ItemCheck_Shoot` fires once per use start, and
/// the channel holds the animation so no new use starts while held).
fn drill_channel_start(app: &mut App, held: ItemSlot, world_pos: Vec2) -> bool {
    let Some(def) = registry::get(held.kind) else {
        return false;
    };
    if !def.channel || (def.pick == 0 && def.axe == 0) {
        return false;
    }
    let Some(proj) = def.projectile else {
        return false;
    };
    // A live drill/chainsaw of this kind already holds the channel.
    let live = app.projectiles.slots().iter().any(|p| {
        p.active && p.kind == proj && p.owner == app.local_id as u8
    });
    if live {
        return true;
    }
    // The spawn mirrors the generic `Item.shoot` firing (`ItemCheck_Shoot`,
    // `Player.cs:47056+`): the aim normalized to `shootSpeed` (the melee
    // `speed /= meleeSpeed`, `:47061-47063`), the player-centre origin, the
    // class-multiplied damage and the item knockback. The drill projectile
    // is a held tool — its damage rides the same `weaponDamage` chain.
    let Some(p) = app.player.as_ref() else {
        return false;
    };
    let (pcx, pcy) = p.center();
    let dir = (world_pos - Vec2::new(pcx, pcy)).normalize_or_zero();
    if dir == Vec2::ZERO {
        return false;
    }
    let class = def.class;
    let (fx_dmg, fx_spd, fx_kb) = crate::prefix::prefixed_stats(held.kind, held.prefix)
        .map_or((def.damage, def.shoot_speed, def.knock_back), |s| {
            (s.damage, s.shoot_speed, s.knock_back)
        });
    let speed = if class == registry::DamageClass::Melee {
        fx_spd / p.melee_speed(held.kind)
    } else {
        fx_spd
    };
    let raw = fx_dmg as f32 * p.weapon_damage_multiplier(class);
    let spawn = Vec2::new(pcx, pcy) + dir * 10.0;
    let slot = app.projectiles.spawn_with_data_ai(
        proj,
        spawn.x,
        spawn.y,
        dir.x * speed,
        dir.y * speed,
        raw as i32,
        fx_kb,
        0.0,
        0.0,
    );
    // M3 sync: no throw-time crit roll - the drill's damage rides the
    // per-hit crit block in damage_pve (Projectile.cs:12883-12917,
    // S1-P-i11); the generic ItemCheck_Shoot tail (Player.cs:49437) carries
    // no roll. The spawn spends no UpdatePlayers draw.
    let _ = slot;
    // `StartChanneling` (`Player.cs:52958-52968`) latches the channel.
    if let Some(p) = app.player.as_mut() {
        p.channel = true;
        p.channel_item = held.kind;
        p.channel_target = world_pos;
    }
    // The swing start (`ApplyItemAnimation`, `Player.cs:4463-4497`) — the
    // drill's `useAnimation`; `toolTime = 1` (`:51660-51663`). The channel
    // hold (`drill_holding` below) keeps the swing alive at the dummy 2
    // ticks while the projectile lives; the `toolTime` countdown wraps to
    // `useTime - 1` so the first pick hit lands `useTime` ticks after the
    // press (`Player.cs:43582-43586`).
    app.swing_max = crate::item::use_animation_secs_of(held.kind, held.prefix);
    app.swing = app.swing_max;
    start_swing_extras(app, held.kind);
    app.tool_time = ONE_TICK_SECS;
    true
}

/// Whether a drill/chainsaw projectile is live for the local player — the
/// channel hold gate (`Projectile.cs:25563`'s `SetDummyItemTime(2)` keeps
/// the item animation alive while the tool lives; the chainsaws share the
/// aiStyle-20 machine, S1-P-i30).
fn drill_live(app: &App, held: ItemSlot) -> bool {
    let Some(def) = registry::get(held.kind) else {
        return false;
    };
    if !def.channel || (def.pick == 0 && def.axe == 0) {
        return false;
    }
    let Some(proj) = def.projectile else {
        return false;
    };
    app.projectiles
        .slots()
        .iter()
        .any(|p| p.active && p.kind == proj && p.owner == app.local_id as u8)
}

/// `Player.ItemCheck_UseMiningTools` (`Player.cs:45584-45644`) +
/// `ItemCheck_UseMiningTools_ActuallyUseMiningTool` (`:45646-45782`): the
/// pickaxe/drill mining tick. Fires at `toolTime == 0 && itemAnimation > 0
/// && controlUseItem` (`:45608`/`:45626`) with the target in the item's
/// reach (`IsTargetTileInItemRange`, `:45599`/`:46313-46316`). The pick
/// arm (`:45772-45780`) runs `PickTile(x, y, sItem.pick)` and re-arms
/// `itemTime = (int)(useTime * pickSpeed)`.
///
/// The axe tiles (`Main.tileAxe`) and hammer tiles (`Main.tileHammer`)
/// route to their own arms (`:45656-45771`) — a pick never damages them;
/// those ports are the i30/i31 tasks. The demon-altar hammer arm below
/// keeps its pre-i29 time-based pacing (the i31 task ports the hammer
/// hit-buffer model).
fn mining_tick(app: &mut App, tx: i32, ty: i32, held: ItemSlot) {
    // The wall-pass gate (`Player.cs:45650` `canHitWalls = true`): every
    // early return leaves it true — the wall pass (`:45639-45643`) runs
    // even when the tile arm did nothing (the exposed-wall case).
    app.mining_can_hit_walls = true;
    let Some(def) = registry::get(held.kind) else {
        app.mine_target = None;
        app.mine_timer = 0.0;
        return;
    };
    let tool_pick = i32::from(def.pick);
    let tool_axe = i32::from(def.axe);
    let tool_hammer = i32::from(def.hammer);
    // `:45595-45598` — a non-tool never mines.
    if tool_pick <= 0 && tool_axe <= 0 && tool_hammer <= 0 {
        app.mine_target = None;
        app.mine_timer = 0.0;
        return;
    }
    let Some(tile) = app.world.tile(tx, ty).copied() else {
        app.mine_target = None;
        app.mine_timer = 0.0;
        return;
    };
    // `:45599-45603` — the reach gate (`IsTargetTileInItemRange` with the
    // item's `tileBoost`); `noBuilding` forces it false (the port has no
    // noBuilding state — the frozen/webbed gate above already refuses).
    let (range_x, range_y) = mining_tile_range(app);
    let in_reach = app
        .player
        .as_ref()
        .is_some_and(|p| tile_in_reach_rect(p, tx, ty, range_x, range_y, def.tile_boost));
    if !in_reach {
        app.mine_target = None;
        app.mine_timer = 0.0;
        return;
    }
    // The swing start — the C# `ItemCheck_TryStartUse` →
    // `ItemCheck_StartActualUse` (`Player.cs:42733`/`:51657`) arms the
    // animation on the press edge (`ApplyItemAnimation`, `:4463-4497`); the
    // port starts the swing when none runs. The autoReuse restart rides the
    // generic swing block below (`Player.cs:42819-42822`); a non-autoReuse
    // pick waits for a release. `toolTime = 1` (`:51660-51663`) arms at the
    // start so the first pick hit lands one tick later. The drill's swing is
    // started by `drill_channel_start` and held by the channel block, so it
    // never reaches this start with `swing <= 0`.
    if app.swing <= 0.0 {
        app.swing_max = crate::item::use_animation_secs_of(held.kind, held.prefix);
        app.swing = app.swing_max;
        start_swing_extras(app, held.kind);
        app.tool_time = ONE_TICK_SECS;
    }
    // `:45608`/`:45626` — the action gate: `toolTime == 0 && itemAnimation
    // > 0 && controlUseItem`. The port's `tool_time` is seconds; the tick
    // quantisation reads it as whole ticks (the `ticks_of` convention).
    let tool_ready = ticks_of(app.tool_time) <= 0;
    let anim_running = app.swing > 0.0;
    if !tool_ready || !anim_running || !app.mine_down {
        // The dig target tracks the cursor even when the action is not
        // ready (the crack overlay reads it); the timer is the book's
        // damage, refreshed below.
        app.mine_target = Some((tx, ty));
        sync_mine_crack_state(app, tx, ty);
        return;
    }
    // `:45652-45654` — an inactive cell mines nothing (the wall arm below
    // handles the exposed wall).
    if !tile.active() {
        app.mine_target = None;
        app.mine_timer = 0.0;
        return;
    }
    let axe_tile = tr_core::tool::is_axe_tile(tile.kind);
    let hammer_tile = tr_core::tool::is_hammer_tile(tile.kind);
    // `:45656` — the tool/tile match: a pick mines the non-axe/non-hammer
    // tiles, an axe chops the axe tiles, a hammer smashes the hammer tiles.
    let pick_match = tool_pick > 0 && !axe_tile && !hammer_tile;
    let axe_match = tool_axe > 0 && axe_tile;
    let hammer_match = tool_hammer > 0 && hammer_tile;
    app.mine_target = Some((tx, ty));
    if !pick_match && !axe_match && !hammer_match {
        app.mine_target = None;
        app.mine_timer = 0.0;
        // No tool arm ran — the pound + wall passes still run (the C#
        // `ActuallyUseMiningTool` falls through to `:45781`). The
        // `tileHammer` refusal (`:45665-45667`) fires even WITHOUT a
        // matching hammer (a pick over an altar refuses the wall pass).
        let tile_hit_id = app.hit_tile.hit_object(tx, ty, tr_core::hit_tile::TILE);
        let mut can_hit_walls = !hammer_tile;
        // `:45777-45780` — the shared `sItem.pick > 0` tail runs on the
        // no-match fall-through too (a hamaxe over a plain poundable tile
        // re-arms the pickSpeed cadence before the pound).
        if tool_pick > 0 {
            let pick_speed = app.player.as_ref().map_or(1.0, |p| p.pick_speed());
            let use_time = held_use_time_ticks(held);
            app.item_time = ((use_time as f32 * pick_speed) as i32).max(0) as f32 * ONE_TICK_SECS;
        }
        try_pounding_tile_tick(app, tx, ty, tile_hit_id, &mut can_hit_walls, held);
        app.mining_can_hit_walls = can_hit_walls;
        return;
    }
    // `:45660` — the shared tile-book slot, taken BEFORE the arm split
    // (`hitTile.HitObject(x, y, 1)`); the pound arm (`:45781`) reuses it.
    let tile_hit_id = app.hit_tile.hit_object(tx, ty, tr_core::hit_tile::TILE);
    // `:45656-45659` + `:45667` — the wall-pass gate: a matching tool on its
    // tile (or ANY tileHammer tile) refuses the wall pass this tick; the
    // pound arm can flip it back (`:45986`).
    let mut can_hit_walls = !(pick_match || axe_match || hammer_tile);
    if hammer_match {
        // The hammer arm (`Player.cs:45665-45704`, S1-P-i31): the damage
        // accumulates in the shared `HitTile` book, `sItem.hammer` per hit.
        hammer_tile_hit(app, tx, ty, tile.kind, tool_hammer, tile_hit_id, held);
        if app.world.tile(tx, ty).is_some_and(|t| t.active()) {
            sync_mine_crack_state(app, tx, ty);
        } else {
            app.mine_target = None;
            app.mine_timer = 0.0;
        }
    } else if pick_match {
        // The pick arm (`:45772-45780`): `PickTile(x, y, sItem.pick)`, then
        // `itemTime = (int)(useTime * pickSpeed)`.
        let broke = pick_tile_hit(app, tx, ty, tool_pick);
        let pick_speed = app.player.as_ref().map_or(1.0, |p| p.pick_speed());
        let use_time = held_use_time_ticks(held);
        app.item_time = ((use_time as f32 * pick_speed) as i32).max(0) as f32 * ONE_TICK_SECS;
        if broke {
            app.mine_target = None;
            app.mine_timer = 0.0;
        } else {
            sync_mine_crack_state(app, tx, ty);
        }
    } else if axe_match {
        // The axe arm (`Player.cs:45706-45771`, S1-P-i30).
        let broke = axe_tile_hit(app, tx, ty, tile.kind, tool_axe, held);
        if broke {
            app.mine_target = None;
            app.mine_timer = 0.0;
        } else {
            sync_mine_crack_state(app, tx, ty);
        }
    }
    // `:45777-45780` — the shared `sItem.pick > 0` tail: the dual-power
    // hamaxes/draxes take the pickSpeed re-arm even on the hammer/axe arms
    // (the per-arm re-arms above are the pure-tool copies).
    if tool_pick > 0 && !pick_match {
        let pick_speed = app.player.as_ref().map_or(1.0, |p| p.pick_speed());
        let use_time = held_use_time_ticks(held);
        app.item_time = ((use_time as f32 * pick_speed) as i32).max(0) as f32 * ONE_TICK_SECS;
    }
    // `:45781` — the pound arm runs on EVERY mining tick (the release latch
    // gates it), after the tool arms.
    try_pounding_tile_tick(app, tx, ty, tile_hit_id, &mut can_hit_walls, held);
    // The wall pass (`:45639-45643`) — the caller in `update_editing` reads
    // the returned gate; the pound may have cleared it (`:45986`).
    app.mining_can_hit_walls = can_hit_walls;
}

/// The hammer arm's one hit — `ItemCheck_UseMiningTools_ActuallyUseMiningTool`
/// (`Player.cs:45665-45704`, S1-P-i31). `num2 += sItem.hammer` (`:45670`);
/// a `CanKillTile` refusal zeroes it (`:45671-45674`); the demon altar 26
/// additionally needs a ≥ 80 hammer in hardmode — otherwise the damage
/// zeroes AND the smasher takes `Hurt(ByOther(4), statLife / 2, -direction)`
/// (`:45675-45679`). At `>= 100` the cell breaks (`KillTile` — the altar's
/// case-26 arm runs `SmashAltar` + BEGONE_EVIL through `break_tile`'s
/// cascade, the M5-HARDFORCE-ores seam); otherwise the fail tick
/// (`KillTile(fail: true)` — the per-tick sound/dust is the recorded i29
/// deviation, the damage stays in the book). `ApplyItemTime(sItem)`
/// (`:45702`) re-arms `itemTime = useTime` (NO pickSpeed scale — the shared
/// `sItem.pick > 0` tail in `mining_tick` handles the dual-power tools).
fn hammer_tile_hit(
    app: &mut App,
    tx: i32,
    ty: i32,
    kind: u16,
    hammer_power: i32,
    tile_hit_id: usize,
    held: ItemSlot,
) {
    let mut damage = hammer_power;
    // `:45671-45674`.
    if !edit::can_kill_tile(&app.world, tx, ty, chest_holds_items(app)) {
        damage = 0;
    }
    // `:45675-45679` — the altar refusal + self-hurt. The C# `Hurt` runs on
    // EVERY refused hit (the damage zeroed, the swing still plays). The
    // `-base.direction` hitDirection maps to a knockback origin on the
    // player's FACING side (`hurt_inner`'s `from_x` convention: the pop
    // pushes AWAY from `from_x`, so `-direction` → origin at
    // `centre + direction * 16`). The hurt's dodge rolls ride the
    // `UpdatePlayers` stream (the ItemCheck swap, `Main.cs:17754`).
    if kind == 26 && (hammer_power < 80 || !app.world.hard_mode) {
        damage = 0;
        let (half_hp, facing) = app
            .player
            .as_ref()
            .map(|p| (p.hp / 2, p.facing))
            .unwrap_or((0, 1));
        if half_hp > 0 {
            let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
            if let Some(p) = app.player.as_mut() {
                let from_x = p.x + crate::player::WIDTH * 0.5 + facing as f32 * TILE_PX;
                // The C# `Hurt(ByOther(4), statLife / 2, -direction)` takes
                // the default `cooldownCounter = -1` (General) — the
                // retaliate lane (`ImmunityCooldownID.Sets.Retaliate`
                // membership, the `hurt` wrapper's true).
                let _ = p.hurt(half_hp, from_x, rng, true);
            }
        }
    }
    // `:45681` — the accumulation; `>= 100` breaks.
    let total = app.hit_tile.add_damage(tile_hit_id, damage, true);
    if total >= 100 {
        // `:45683` — `ClearMiningCacheAt(x, y, 1)` (its own Prune runs
        // here, `HitTile.cs:149-157`).
        app.hit_tile
            .try_clearing_and_pruning(tx, ty, tr_core::hit_tile::TILE, |x, y| {
                app.world.tile(x, y).is_some_and(|t| t.active())
            }, |x, y| app.world.tile(x, y).map_or(0, |t| t.wall));
        // `:45684` — `WorldGen.KillTile(x, y)`. The altar's case-26 arm
        // (`WorldGen.cs:49444-49449`) smashes AFTER the tile dies: the ore
        // seeding + BEGONE_EVIL ride the M5-HARDFORCE-ores seam here (the
        // kill cascade's `check_orb` handles the orb 31 / heart 695/696
        // drops and the EoW/BoC summons).
        let was_altar = kind == 26;
        break_tile(app, tx, ty);
        if was_altar && !app.world.tile(tx, ty).is_some_and(|t| t.active()) {
            let altars = app.world.altars_smashed;
            let ore = {
                let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
                tr_core::ores::smash_altar(&mut app.world, &mut app.ore_tiers, altars, rng)
            };
            // `WorldGen.SmashAltar`'s ore-blessing broadcast (`WorldGen.cs:
            // 48969`/`:49002`/`:49034`): `ChatHelper.BroadcastChatMessage(
            // NetworkText.FromKey(Lang.misc[num7].Key), ChatColors.World)`.
            // The key rides the PLACED ore — 12/13/14 the classic trio
            // (Cobalt/Mythril/Adamantite), +9 = 21/22/23 the alternates
            // (Palladium/Orichalcum/Titanium, `:48964-48966`/`:48997-48999`/
            // `:49029-49031`). The C# broadcasts inside SmashAltar before the
            // OreRunner batches; the port's `smash_altar` returns the placed
            // ore, so the line lands here (the chat-log order is unchanged).
            let blessing = match ore {
                107 => 12,
                221 => 21,
                108 => 13,
                222 => 22,
                111 => 14,
                223 => 23,
                _ => 12,
            };
            crate::chat::broadcast_death_text(
                &mut app.chat,
                &crate::lang::misc(blessing),
                crate::chat::COLOR_WORLD,
            );
            app.world.altars_smashed = altars + 1;
            let _ = app.notify_progression_net(6);
            app.world_key = None;
        }
    }
    // `:45698-45701` — a non-zero damage prunes the book AFTER both
    // branches (the break tick prunes TWICE: the ClearMiningCacheAt prune
    // above + this tail — the C# order, kept).
    if damage != 0 {
        app.hit_tile.prune(|x, y| {
            app.world.tile(x, y).is_some_and(|t| t.active())
        }, |x, y| app.world.tile(x, y).map_or(0, |t| t.wall));
    }
    // `:45702` — `ApplyItemTime(sItem)`: `itemTime = useTime` (the
    // dual-power pick tail in `mining_tick` overrides for the hamaxes).
    let use_time = held_use_time_ticks(held);
    app.item_time = use_time.max(0) as f32 * ONE_TICK_SECS;
}

/// The pound arm's tick wrapper — `ItemCheck_UseMiningTools_TryPoundingTile`
/// (`Player.cs:45969-46311`, S1-P-i31). The `poundRelease` latch gates it;
/// the world-state cycle lives in [`tr_core::edit::try_pound_tile`]. The
/// `hitTile.AddDamage(tileHitId, damageAmount)` (`:45993`) runs BEFORE the
/// cycle: a locked door's zero damage keeps the latch and plays the fail
/// tick's Dig 0 (`:45995-45997`); a fresh pound's 100 always clears the
/// threshold. `ApplyItemTime(sItem)` (`:45987`) fires on every accepted
/// pound (the dual-power pick tail already ran in `mining_tick` — the C#
/// order is the same: `:45777-45780` then `:45987`).
fn try_pounding_tile_tick(
    app: &mut App,
    tx: i32,
    ty: i32,
    tile_hit_id: usize,
    can_hit_walls: &mut bool,
    held: ItemSlot,
) {
    // `:45971-45974`.
    if !app.pound_release {
        return;
    }
    let hammer_power = registry::get(held.kind).map_or(0, |d| i32::from(d.hammer));
    let tile = app.world.tile(tx, ty).copied();
    // `:45975-45985` — the hammer / active / poundable gates clear the latch.
    if hammer_power <= 0
        || !tile.is_some_and(|t| t.active() && edit::is_tile_poundable(t.kind))
    {
        app.pound_release = false;
        return;
    }
    // `:45986-45987` — the wall pass is refused for this tick; the item
    // time re-arms (`ApplyItemTime(sItem)` — plain `useTime`, NO pickSpeed
    // scale; the C# pound runs AFTER the `:45777-45780` pick tail, so it
    // overrides the dual-power re-arm too).
    *can_hit_walls = false;
    let use_time = held_use_time_ticks(held);
    app.item_time = use_time.max(0) as f32 * ONE_TICK_SECS;
    // `:45988-45998` — the damage add; the locked-door zero keeps the latch.
    let damage = edit::pound_damage(&app.world, tx, ty);
    let total = app.hit_tile.add_damage(tile_hit_id, damage, true);
    if total < 100 {
        // The fail tick: `KillTile(fail, effectOnly)` dust (skipped) + the
        // Dig 0 cue (`SoundEngine.PlaySound(0, x*16, y*16)`, `:45996`). The
        // case-0 variation + pitch draws ride the `UpdatePlayers` stream
        // (the ItemCheck swap), only when audible (`play_place_sound`).
        app.audio.play_place_sound(
            (0, 1),
            tx as f32 * TILE_PX,
            ty as f32 * TILE_PX,
            app.rngs.stream(crate::app::RngKey::UpdatePlayers),
        );
        return;
    }
    // `:45999` — `ClearMiningCacheAt(x, y, 1)`.
    app.hit_tile
        .try_clearing_and_pruning(tx, ty, tr_core::hit_tile::TILE, |x, y| {
            app.world.tile(x, y).is_some_and(|t| t.active())
        }, |x, y| app.world.tile(x, y).map_or(0, |t| t.wall));
    // The cycle (`:46004-46309`). The chest closure captures the DISJOINT
    // `world_chests` field (not `app`) so it coexists with the `&mut
    // app.world` borrow — the `chest_holds_items(app)` helper borrows all
    // of `app` and would conflict.
    let chests = &app.world_chests;
    let holds = |x: i32, y: i32| {
        chests
            .iter()
            .any(|c| c.tile == (x, y) && c.slots.iter().any(|s| s.count > 0))
    };
    let outcome = edit::try_pound_tile(&mut app.world, tx, ty, holds);
    match outcome {
        edit::PoundOutcome::Pounded { lift } => {
            // The SlopeTile/PoundTile Dig 0 cue (`WorldGen.cs:80782`/
            // `:80822`) — one play per landed write, INSIDE the gate (a
            // refused write plays nothing: the GateRefused arm below).
            // The platform's last step is a DIRECT `slope(0)` write +
            // `PoundTile` (`Player.cs:46047-46048`) — ONE cue, not two.
            app.audio.play_place_sound(
                (0, 1),
                tx as f32 * TILE_PX,
                ty as f32 * TILE_PX,
                app.rngs.stream(crate::app::RngKey::UpdatePlayers),
            );
            if lift {
                // The flatten player-push (`WorldGen.cs:80787-80795` /
                // `:80826-80834`): every living player whose box intersects
                // the cell rides to its top, the drop booked in `gfxOffY`.
                // The port has one player.
                let rect_x = tx as f32 * TILE_PX;
                let rect_y = ty as f32 * TILE_PX;
                if let Some(p) = app.player.as_mut() {
                    if !p.dead
                        && p.x + crate::player::WIDTH > rect_x
                        && p.x < rect_x + TILE_PX
                        && p.y + p.height > rect_y
                        && p.y < rect_y + TILE_PX
                    {
                        p.gfx_off_y += p.y + p.height - rect_y;
                        p.y = rect_y - p.height;
                    }
                }
            }
            app.queue_map_ring(tx, ty);
            app.world_key = None;
            // `:46310` — the latch clears on the cycle arms.
            app.pound_release = false;
        }
        edit::PoundOutcome::GateRefused => {
            // Every SlopeTile/PoundTile call refused at the CanPoundTile
            // gate (`WorldGen.cs:80770`/`:80807`): NO cue, no lift — but
            // the latch still clears (`Player.cs:46310`) and the caches
            // wake (the platform's ungated direct `slope(0)` write may
            // have landed, `:46047`).
            app.queue_map_ring(tx, ty);
            app.world_key = None;
            app.pound_release = false;
        }
        edit::PoundOutcome::Framed | edit::PoundOutcome::Consumed => {
            // The frame arms carry no sound (`SendTileSquare` only); the
            // Consumed seams (470/314) change nothing in the port. The
            // frame writes replicate through `SendTileSquare` in the C# —
            // no port net kind yet (ANALYSIS); the local caches still wake.
            app.queue_map_ring(tx, ty);
            app.world_key = None;
            app.pound_release = false;
        }
        // The locked-door / refused arms keep the latch (`:45997` returns
        // before the `:46310` clear). The slope/half-brick net replication
        // (SendData 17 subtypes 7/14/23) has no port kind yet — the pound
        // does NOT `request_edit` (a kind-0 break would mis-replicate);
        // recorded deviation, ANALYSIS.
        edit::PoundOutcome::LockedDoor | edit::PoundOutcome::Refused => {}
    }
}

/// The wall pass — `ItemCheck_UseMiningTools_TryFindingWallToHammer` +
/// `ItemCheck_UseMiningTools_TryHittingWall` + `PickWall`
/// (`Player.cs:45832-45967`, S1-P-i31). Runs when the mining tick left
/// `canHitWalls` true (`:45639`): the finder picks the wall cell (the
/// cursor's own, or the mouse-quadrant 2×2 scan), the hit gate
/// (`:45910`) checks the tile/wall/poundRelease composition, and
/// `PickWall(x, y, (int)(hammer * 1.5f))` accumulates in the WALL book
/// (`hitTile.HitObject(x, y, 2)`). `itemTime = useTime / 2` (`:45913`).
fn wall_hammer_tick(app: &mut App, tx: i32, ty: i32, held: ItemSlot, mouse: (f32, f32)) {
    let hammer_power = registry::get(held.kind).map_or(0, |d| i32::from(d.hammer));
    if hammer_power <= 0 {
        return;
    }
    let (wx, wy) = edit::try_finding_wall_to_hammer(&app.world, tx, ty, mouse);
    // `:45910` — the hit gate.
    let Some(cell) = app.world.tile(wx, wy).copied() else {
        return;
    };
    if cell.wall == 0 {
        return;
    }
    let tile_blocks = cell.active()
        && wx == tx
        && wy == ty
        && (tr_core::tool::is_hammer_tile(cell.kind) || app.pound_release);
    if tile_blocks {
        return;
    }
    if !edit::can_player_smash_wall(&app.world, wx, wy) {
        return;
    }
    // `:45912` — `PickWall(wX, wY, (int)((float)sItem.hammer * 1.5f))`.
    let damage = (hammer_power as f32 * 1.5) as i32;
    let wall_id = app.hit_tile.hit_object(wx, wy, tr_core::hit_tile::WALL);
    let total = app.hit_tile.add_damage(wall_id, damage, true);
    if total >= 100 {
        // `:45947` — `hitTile.Clear(tileId)` clears the WALL slot.
        app.hit_tile.clear(wall_id);
        // `:45948` — `ClearMiningCacheAt(x, y, 2)`. The C# ignores the type
        // argument and always prunes the TILE book (`Player.cs:53848-53849`
        // passes 1): a TILE slot at the cell, if any, clears + prunes.
        app.hit_tile
            .try_clearing_and_pruning(wx, wy, tr_core::hit_tile::TILE, |x, y| {
                app.world.tile(x, y).is_some_and(|t| t.active())
            }, |x, y| app.world.tile(x, y).map_or(0, |t| t.wall));
        // `:45949` — `WorldGen.KillWall(x, y)`.
        break_wall(app, wx, wy);
    }
    // `:45963-45966` — a non-zero damage prunes AFTER both branches (the
    // fail tick's KillWall(fail) dust is the recorded deviation).
    if damage != 0 {
        app.hit_tile.prune(|x, y| {
            app.world.tile(x, y).is_some_and(|t| t.active())
        }, |x, y| app.world.tile(x, y).map_or(0, |t| t.wall));
    }
    // `:45913` — `itemTime = useTime / 2`.
    let use_time = held_use_time_ticks(held);
    app.item_time = (use_time / 2).max(0) as f32 * ONE_TICK_SECS;
}

/// The axe arm's one chop — `ItemCheck_UseMiningTools_ActuallyUseMiningTool`
/// (`Player.cs:45706-45771`, S1-P-i30). The damage line (`:45708-45712`)
/// lives in [`tr_core::tool::axe_damage`] (the f32 1.2 scale, the cactus
/// triple, the getGoodWorld 1.3); the `Main.tileNoFail` 100 head
/// (`:45661-45664`) never fires for an axe tile (no `tileAxe` id is
/// `tileNoFail`, `Main.cs:7189-7203` vs `:7208-7223`), so the base is 0.
/// The accumulation rides the SAME `hitTile` book the pick arm uses
/// (`:45660` `HitObject(x, y, 1)` runs before the arm split); at `>= 100`
/// the cell breaks (`:45725-45755`), otherwise the fail tick
/// (`KillTile(fail: true)`, `:45758` — the port's per-tick sound/dust is
/// the recorded i29 deviation, the damage stays in the book). Returns
/// whether the tile broke.
fn axe_tile_hit(app: &mut App, tx: i32, ty: i32, kind: u16, axe_power: i32, held: ItemSlot) -> bool {
    // `:45660` — the buffer slot for the cell (the shared book).
    let buffer_index = app.hit_tile.hit_object(tx, ty, tr_core::hit_tile::TILE);
    // `:45708-45712` — the damage ladder. `Main.getGoodWorld` = the
    // For-the-Worthy seed (the `pick_world` mapping).
    let get_good = app
        .world
        .seeds
        .contains(tr_core::seeds::SeedFlags::FOR_THE_WORTHY);
    let mut damage = tr_core::tool::axe_damage(kind, axe_power, 0, get_good);
    // `:45717-45720` — a `CanKillTile` refusal zeroes the chop damage.
    if !edit::can_kill_tile(&app.world, tx, ty, chest_holds_items(app)) {
        damage = 0;
    }
    // `:45721-45724` — the Don't Starve cactus chop hurts the chopper on
    // EVERY damaging hit (not only the break): `Main.DamageVar(6f, 0f -
    // this.luck)`, hitDirection 0 (no knockback), not quiet, no crit, the
    // TileContactDamage cooldown. The port's single hurt lane folds the
    // cooldown counters onto `hurt_cd` (the M4-HAZARD-1 admission,
    // `touch_damage.rs`'s precedent) and `hitDirection 0` maps to the
    // player's own centre (no directional kick). The damage draw rides the
    // `UpdatePlayers` stream (the ItemCheck swap, `Main.cs:17754`), before
    // the kill's own drop draws — the C# order.
    if app
        .world
        .seeds
        .contains(tr_core::seeds::SeedFlags::DONT_STARVE)
        && damage > 0
        && kind == 80
    {
        let luck = app.player.as_ref().map_or(0.0, |p| p.luck());
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        let dmg = tr_core::damage::damage_var(rng, 6.0, -luck);
        if let Some(p) = app.player.as_mut() {
            let (cx, _) = p.center();
            let _ = p.hurt(dmg, cx, rng, true);
        }
    }
    // `:45725` — the accumulation; `>= 100` breaks.
    let total = app.hit_tile.add_damage(buffer_index, damage, true);
    if total >= 100 {
        // `:45727-45742` — Lucy the Axe's (5095) chopped-tree chat bubble
        // (`LucyAxeMessage`) has no port chat-bubble system: the recorded
        // deviation, the chop itself is unchanged (ANALYSIS, S1-P-i30).
        // `:45743` — `ClearMiningCacheAt(x, y, 1)` clears the hit slot
        // (`Player.cs:53846-53850`; the port has no replacement book).
        app.hit_tile
            .try_clearing_and_pruning(tx, ty, tr_core::hit_tile::TILE, |x, y| {
                app.world.tile(x, y).is_some_and(|t| t.active())
            }, |x, y| app.world.tile(x, y).map_or(0, |t| t.wall));
        // `:45744` — `IsBottomOfTreeTrunkNoRoots(x, y)` read BEFORE the kill
        // (the Acorn Axe's replant gate).
        let replant_base =
            held.kind == 5295 && is_bottom_of_tree_trunk_no_roots(&app.world, tx, ty);
        // `:45745` — `WorldGen.KillTile(x, y)`: the whole-tree fall rides
        // the kill cascade (`tr_core::edit::kill_tile` → `CheckTree` /
        // `CheckPalmTree` / `CheckCactus` / the case-72 support check).
        break_tile(app, tx, ty);
        // `:45750-45753` — the Staff of Regrowth's twin, the Acorn Axe
        // (5295), replants an acorn sapling into the felled base cell.
        if replant_base {
            try_replanting_tree(app, tx, ty);
        }
        // `:45764-45767` — the tail prune runs AFTER both branches (the
        // break tick prunes twice: ClearMiningCacheAt's own prune + this).
        if damage != 0 {
            app.hit_tile.prune(|x, y| {
                app.world.tile(x, y).is_some_and(|t| t.active())
            }, |x, y| app.world.tile(x, y).map_or(0, |t| t.wall));
        }
        // `:45768` — `ApplyItemTime(sItem)`: `itemTime = useTime` verbatim
        // (`Player.cs:4416-4419` — NO `pickSpeed` scale; the pick arm's
        // `:45777-45780` re-arm is gated on `sItem.pick > 0`, which a pure
        // axe never reaches; the Drax 579 carries both powers and takes
        // that re-arm below, exactly like the C#).
        rearm_axe_item_time(app, held);
        return true;
    }
    // `:45758` — the fail tick (`KillTile(fail: true)`): the per-tick
    // sound/dust is the recorded deviation; the damage stays in the book.
    // `:45764-45767` — a non-zero damage prunes the book.
    if damage != 0 {
        app.hit_tile.prune(|x, y| {
            app.world.tile(x, y).is_some_and(|t| t.active())
        }, |x, y| app.world.tile(x, y).map_or(0, |t| t.wall));
    }
    // `:45768` — the cadence re-arm on the fail tick too.
    rearm_axe_item_time(app, held);
    false
}

/// The axe arm's `ApplyItemTime(sItem)` (`Player.cs:45768` → `:4416-4419`):
/// `itemTime = useTime` (prefix-aware — `Item.Prefix` scales the INSTANCE's
/// `useTime`), then the shared `sItem.pick > 0` tail (`:45777-45780`)
/// overrides it with `(int)(useTime * pickSpeed)` for the dual-power
/// Drax/hamaxes.
fn rearm_axe_item_time(app: &mut App, held: ItemSlot) {
    let use_time = held_use_time_ticks(held);
    let def = registry::get(held.kind);
    if def.is_some_and(|d| d.pick > 0) {
        let pick_speed = app.player.as_ref().map_or(1.0, |p| p.pick_speed());
        app.item_time = ((use_time as f32 * pick_speed) as i32).max(0) as f32 * ONE_TICK_SECS;
    } else {
        app.item_time = use_time.max(0) as f32 * ONE_TICK_SECS;
    }
}

/// The held item's `useTime` in ticks, prefix-aware (`Item.Prefix` scales
/// `useTime`, `prefix.rs:400`). The mining `itemTime` re-arm reads it
/// (`Player.cs:45779`).
fn held_use_time_ticks(held: ItemSlot) -> i32 {
    if held.prefix != 0 {
        if let Some(s) = crate::prefix::prefixed_stats(held.kind, held.prefix) {
            return s.use_time;
        }
    }
    registry::get(held.kind).map_or(0, |d| i32::from(d.use_time))
}

/// Refresh the crack overlay's `mine_timer`/`mine_hardness` from the
/// `HitTile` book's damage at the cell — the port's projection of the C#
/// `HitTile` damage onto the 0..100 crack ladder (`render/tile_cracks.rs`,
/// `Main.cs:61533-61626`). The book is the source of truth now; the timer
/// is the damage fraction and the hardness is 1.0 (the full break).
fn sync_mine_crack_state(app: &mut App, tx: i32, ty: i32) {
    let damage = app
        .hit_tile
        .try_finding(tx, ty, tr_core::hit_tile::TILE)
        .map_or(0, |id| app.hit_tile.data[id].damage);
    app.mine_hardness = 1.0;
    app.mine_timer = (damage.clamp(0, 100) as f32) / 100.0;
}

pub(crate) fn update_editing(app: &mut App, dt: f32, screen: Vec2) {
    #[cfg(test)]
    if std::env::var("AMMO_TRACE").is_ok() {
        eprintln!("[trace-top] entered, screen={}x{}", screen.x, screen.y);
    }
    // `Player.ItemCheck`'s `CCed` gate (`Player.cs:42598-42603` → `:3377-3387`:
    // frozen || webbed || stoned): the FIRST thing ItemCheck does — the early
    // return runs BEFORE the auto-reuse logic and every countdown, so a CCed
    // tick neither decays `itemTime` nor consumes the reuse lock. The CC
    // itself clears the channel and the animation (`channel = false;
    // itemAnimation = itemAnimationMax = 0`); the port's `attack_cd` start
    // lock mirrors the animation, so it clears with it.
    if app.player.as_ref().is_some_and(|p| p.cced()) {
        if let Some(p) = app.player.as_mut() {
            p.channel = false;
            p.channel_item = 0;
            p.channel_target = Vec2::ZERO;
            // S1-P-i23's cancel key + the i20 shoot-id seam clear with the
            // latch (every clear site carries the full set).
            p.channel_proj_expected = 0;
            p.channel_index_expected = 0;
            p.channel_shoot = 0;
        }
        app.swing = 0.0;
        app.swing_max = 0.0;
        app.attack_cd = 0.0;
        app.edit_target = None;
        app.mine_target = None;
        app.mine_timer = 0.0;
        return;
    }
    // `Player.ItemCheck`'s countdown order (`Player.cs:42656-42671`,
    // `:42808-42845`): the negative clamps, the `itemAnimation == 0 →
    // itemAnimationMax = 0` reset and the `ApplyReuseDelay` conversion run at
    // the TOP of the tick. The animation decrement (`:42814`) and `itemTime`
    // decrement (`:42836`) both run BEFORE the use arms (`ItemCheck_OwnerOnlyCode`
    // at `:42854` reads the post-decrement values), so the port decays both at
    // the frame head — the start gates and the shot gate below read the same
    // tick the C# gates see. The start gate (`:42733`) sits between the two
    // decrements in the C#, but a fresh start arms `itemAnimation`/`itemTime`
    // to their full values, so the head-decay of the prior (zero) state is a
    // no-op on the start tick — the alignment is identical.
    // `ItemCheck_AutoReuseLogic`'s end-of-swing early-out (`Player.cs:52886-
    // 52899`): an `autoReuse` item at `itemAnimation == 1` with a live stack
    // zeroes the animation BEFORE the countdown and `ApplyReuseDelay` — the
    // C# runs `ItemCheck_AutoReuseLogic` (`:42654`) ahead of the `itemAnimation
    // == 0` reset (`:42664`) and the reuse conversion (`:42668`). For a
    // `reuseDelay == 0` item the head decay below already folds the `1 -> 0`
    // tick into the same-frame restart; for `reuseDelay > 0` (the Clockwork's
    // 14) the early-out additionally lets the conversion run on the tick the
    // last animation tick would otherwise have consumed, matching the source's
    // `useAnimation + reuseDelay` cadence. The local-player arm (`:52894`) is
    // the one the port models; the net-sync `ApplyItemAnimation` re-arm
    // (`:52888-52891`) and `ShootsOnUseRelease` (`:52895-52898`) have no port.
    // The animation-only zeroing (not `itemTime`) is deliberate: the source
    // zeroes just `itemAnimation`, so a running volley's `itemTime` re-arm
    // survives into the reuse hold exactly like the C#.
    {
        let held = app.inventory[app.selected];
        let auto = registry::get(held.kind).is_some_and(|d| d.auto_reuse)
            || shooter_auto_reuse(held.kind);
        if auto && held.count > 0 && ticks_of(app.swing) == 1 {
            app.swing = 0.0;
        }
    }
    // `Player.itemAnimationMax = 0` when the animation ends (`:42664-42667`).
    if app.swing <= 0.0 {
        app.swing_max = 0.0;
    }
    // `ApplyReuseDelay` (`Player.cs:42668-42671` → `:52835-52840`): the
    // animation-zero tick with a pending lock converts it into a silent
    // `itemAnimation = itemTime = reuseDelay` hold — the lock itself clears.
    // The hold counts down through the head decays; while it runs the start
    // gates stay shut (`itemAnimation > 0`), and the shot gate reads
    // `itemTime > 0` so nothing fires inside the hold. `swing_max` stays 0
    // through the hold — the C# never re-arms `itemAnimationMax` here
    // (`:52837` writes only `itemAnimation`), so the volley windows
    // (`anim_max_ticks`) read the ended swing, exactly like the source.
    // The port's `attack_cd` start lock extends over the hold too: the C#
    // start gate reads `itemAnimation == 0`, which is false for every hold
    // tick — `attack_cd` carries the same shut-out for the rigs that drive
    // `update_editing` without the per-tick decay pass.
    if app.swing <= 0.0 && app.reuse_delay > 0.0 {
        app.swing = app.reuse_delay;
        app.item_time = app.reuse_delay;
        app.attack_cd = app.attack_cd.max(app.reuse_delay);
        app.reuse_delay = 0.0;
    }
    // The two per-tick decrements (`:42814` animation, `:42836` itemTime),
    // floored at 0 — the C# `itemAnimation--` / `itemTime--` inside their
    // `> 0` guards.
    // `ItemCheck_ApplyManaRegenDelay` (`Player.cs:42810-42813` → `:50587-50593`):
    // EVERY animation tick of a mana-paying item re-applies the regen delay
    // (so mana cannot regen mid-swing), except the spaceGun four. It runs
    // while `itemAnimation > 0` — which includes the `ApplyReuseDelay` hold
    // above (the hold sets `itemAnimation = reuseDelay`). The value is
    // recomputed from the CURRENT mana each tick (`ApplyManaRegenerationDelay`
    // `:19396-19416`), so it stays pinned until the swing/hold ends. The
    // start tick's delay is installed by the pay path below (the C# countdown
    // runs after the mid-tick start; the port's head decay runs before it, so
    // the pay-time apply covers that one tick — same post-pay mana, same value).
    if app.swing > 0.0 {
        let held = app.inventory[app.selected];
        if registry::get(held.kind).is_some_and(|d| d.mana > 0) {
            if let Some(p) = app.player.as_mut() {
                if !crate::armor_sets::space_gun_skips_mana(p.space_gun, held.kind) {
                    let (max_mana, mana, buff) = (p.mana_max, p.mana, p.mana_regen_buff);
                    crate::regen::apply_mana_regen_delay(&mut p.regen, max_mana, mana, buff);
                }
            }
        }
    }
    app.swing = (app.swing - dt).max(0.0);
    // (Resync keep-one: the i19 per-anim-tick mana-regen-delay block is the
    // shared `ItemCheck_ApplyManaRegenDelay` head ABOVE — both lanes carried
    // it; the gate's second copy was dropped, it re-applied the same delay
    // twice per tick.)
    app.item_time = (app.item_time - dt).max(0.0);
    // `Player.itemRotation`'s per-tick update for the aimed styles (useStyle
    // 5 reads the mouse; `ItemCheck`'s shoot block, `Player.cs:47349-47376`).
    if app.swing > 0.0 && app.swing_style == 5 {
        let (sw, sh) = app.screen_size();
        let mouse_world = Vec2::new(
            app.camera.x + (app.mouse_pos.x - sw * 0.5) / app.zoom,
            app.camera.y + (app.mouse_pos.y - sh * 0.5) / app.zoom,
        );
        let (cx, cy) = app
            .player
            .as_ref()
            .map(|p| p.center())
            .unwrap_or((app.camera.x, app.camera.y));
        let d = mouse_world - Vec2::new(cx, cy);
        app.swing_aim = d.y.atan2(d.x);
    }
    // `Player.releaseUseItem = !controlUseItem` runs every tick
    // (`Player.cs:42829`), so the press/release edges stay current even on
    // frames where the cursor is over the HUD or a panel. The *previous*
    // frame's edges are captured before the overwrite: the use gates below
    // must compare this frame's press against last frame's state (the C#
    // `controlUseItem && releaseUseItem` pair), not against the same frame's.
    let prev_place_down = app.place_prev_down;
    let prev_place_released = app.place_released;
    app.place_prev_down = app.place_down;
    // `releaseUseItem = !controlUseItem` (`Player.cs:42829`) runs at the END
    // of the tick — AFTER the use-start gate (`:42733`) read the PREVIOUS
    // frame's latch. So the gate this frame sees is `!controlUseItem` from
    // LAST frame: the press tick still carries `true` from the idle frame
    // before it, while a held frame carries `false` until
    // `ItemCheck_AutoReuseLogic`/`TryAllowingItemReuse` force it back on
    // below. `mine_prev_down` holds last frame's button, so the latch is
    // `!mine_prev_down` read BEFORE the overwrite. (The old same-frame
    // `!mine_down` write made the latch false on the press tick itself,
    // which shut every non-autoReuse weapon out of its own press.)
    let prev_mine_down = app.mine_prev_down;
    // S1-P-i37's press-edge capture: the use-start gate (`:42733`) reads
    // LAST frame's release latch, not this frame's freshly derived one.
    let prev_mine_released = app.mine_released;
    app.mine_prev_down = app.mine_down;
    app.mine_released = !prev_mine_down;
    app.place_released = !app.place_down;
    // `ItemCheck_AutoReuseLogic` (`Player.cs:52877-52903`): an `autoReuse`
    // item held with the button down forces `releaseUseItem = true`
    // (`:52885`) — the start gate's release half is satisfied every tick, so
    // the swing restarts the moment the animation ends. The `itemAnimation
    // == 1 → 0` early-out (`:52886-52899`) is the same end-of-swing the
    // decay above already produced. `TryAllowingItemReuse` (`:52905-52920`)
    // extends the forced release to the glove/set/setting arms:
    // `autoReuseGlove` covers melee (except the Flying Knife 3030) and the
    // attack-speed-scaling summon weapons; `stressBall || autoReuseAllWeapons`
    // covers every damage > 0 item that is not mid-channel.
    {
        let held = app.inventory[app.selected];
        let def = registry::get(held.kind);
        let auto = def.is_some_and(|d| d.auto_reuse) || shooter_auto_reuse(held.kind);
        if held.count > 0 && app.mine_down && auto {
            app.mine_released = true;
        }
        let (glove, stress) = app
            .player
            .as_ref()
            .map_or((false, false), |p| (p.auto_reuse_glove, p.stress_ball));
        let glove_arm = glove
            && def.is_some_and(|d| {
                (d.melee && held.kind != 3030)
                    || (d.summon && summoner_weapon_scales_with_attack_speed(held.kind))
            });
        let channel_held = app.player.as_ref().is_some_and(|p| p.channel);
        let all_arm = (stress || app.settings.auto_reuse_all)
            && def.is_some_and(|d| d.damage > 0 && (!d.channel || !channel_held));
        // The C# `TryAllowingItemReuse` arms set `releaseUseItem` with NO
        // button gate (`:52916-52919`); the start gate's own
        // `controlUseItem` half (`:42733`) is what keeps a released button
        // from starting a use. The port's gates read `mine_down` alongside
        // `mine_released`, so the force applies on any frame.
        if held.count > 0 && (glove_arm || all_arm) {
            app.mine_released = true;

        }
    }
    // `Player.ItemCheck`'s channel clear (`Player.cs:42787-42807`) runs
    // EVERY tick, ahead of the cursor/reach gates below — the C# clears
    // `channel` on `!flag5` (the button up, the buffered item switch
    // `:42800-42803`, the pending inventory actions `:42796-42798`) even
    // when the cursor is over the HUD or out of reach. The fullscreen map
    // (`:24799`) and the minimap hover (`MinimapFrame.cs:164`'s
    // `mouseInterface`) zero `controlUseItem` the same way, so they feed
    // `flag5` too. The port hoists the clear here so a release cannot leave
    // the latch armed behind an early return (S1-P-i23); the aim-refresh
    // block below only runs while the latch survives this gate.
    {
        let held = app.inventory[app.selected];
        let holding_channel_item = registry::get(held.kind).is_some_and(|d| d.channel)
            && held.count > 0;
        let use_suppressed = !app.mine_down
            || app.inventory_open
            || app.map.as_ref().is_some_and(|m| m.open)
            || app.minimap_hover.is_some();
        if let Some(p) = app.player.as_mut() {
            if p.channel
                && (use_suppressed || !holding_channel_item || held.kind != p.channel_item)
            {
                // `:42804-42807` — `if (!flag5) channel = false`; the empty
                // hand drops the held channel item the same way
                // (`HeldItem.shoot == type` no longer holds). The selection
                // switch rides `:42800-42803`'s `HasBufferedChange` arm
                // (`flag5 = false`): even a switch to ANOTHER channel item
                // drops the latch — the new item re-arms at its own use
                // start with its own cancel key.
                p.channel = false;
                p.channel_item = 0;
                p.channel_target = Vec2::ZERO;
                p.channel_proj_expected = 0;
                p.channel_index_expected = 0;
                // S1-P-i20 seam: the shoot-id field clears with the latch.
                p.channel_shoot = 0;
            }
        }
    }
    // S1-P-i29 — the per-tick `toolTime` block (`Player.cs:43576-43587`):
    // while NOT channeling, `toolTime = itemTime` (the mirror); while
    // channeling (a drill), `toolTime--` wraps to `useTime - 1`. Runs every
    // tick the item-use tick runs (the C# block is unconditional in
    // `ItemCheck_OwnerOnlyCode`); the early-return gates below stand in for
    // the C#'s `controlUseItem`/`mouseInterface` clears that skip the use.
    {
        let held = app.inventory[app.selected];
        let channeling = app.player.as_ref().is_some_and(|p| p.channel);
        if !channeling {
            app.tool_time = app.item_time;
        } else {
            app.tool_time -= dt;
            if app.tool_time < 0.0 {
                let use_time = held_use_time_ticks(held);
                app.tool_time = (use_time - 1).max(0) as f32 * ONE_TICK_SECS;
            }
        }
    }
    if let Some((x, y)) = app.dev_break.take() {
        break_tile(app, x, y);
    }
    for (x, y, kind) in std::mem::take(&mut app.dev_place) {
        let placed = edit::place_tile(&mut app.world, x, y, kind);
        if placed {
            app.queue_map_ring(x, y);
            app.liquids.wake_around(&app.world, x, y, 3);
            app.world_key = None;
        }
    }
    // `Player.cs:24798-24807`: while the map is fullscreen `controlUseItem`/
    // `controlUseTile` are cleared, so no mining, placing or attacking happens
    // behind it; the left button drags the map instead (`update_map_input`).
    // The timers still tick — the C# `ItemCheck` runs its countdowns before
    // any input gate (`:42808-42845`), so the tail decrements run on every
    // early exit too.
    if app.map.as_ref().is_some_and(|m| m.open) {
        app.edit_target = None;
        app.mine_target = None;
        app.mine_timer = 0.0;
        return;
    }
    // `MinimapFrame.Update` sets `Main.LocalPlayer.mouseInterface = true`
    // while a zoom button is hovered (`MinimapFrame.cs:164`), which zeroes
    // the use-item / use-tile inputs for the frame — so mining, placing and
    // attacking are suppressed behind the frame the same way the fullscreen
    // map suppresses them above (`handle_minimap_clicks` keeps this flag
    // current, having run earlier this frame).
    if app.minimap_hover.is_some() {
        app.edit_target = None;
        app.mine_target = None;
        app.mine_timer = 0.0;
        return;
    }
    // `Player.ItemCheck`'s `CCed` gate ran at the TOP of the pass (the C#
    // early return precedes the countdowns, `Player.cs:42598-42603`) — the
    // gate's channel-drop + itemAnimation zeroing folded into it there
    // (ONE CCed head, the resync keep-one).
    let m = app.mouse_pos;
    let hud_top = screen.y - 96.0;
    let cursor_blocked = m.x < 0.0
        || m.y < 0.0
        || m.x > screen.x
        || m.y > screen.y
        || (m.y > hud_top && m.x < 300.0)
        || (app.debug && m.x < 780.0 && m.y < 340.0);
    // S1-P-i05 — the useStyle-3 (Thrust) use-start + stab sweep
    // (`Player.cs:43644-43686`). The C#'s melee-hit block runs EVERY tick
    // while `itemAnimation > 0`, independent of the cursor — only the
    // use-START is cursor-gated (`controlUseItem` zeroed over the HUD).
    // So the sweep runs here (before the HUD return) and an in-flight
    // stab completes even with the pointer over the hotbar; the start
    // reads `cursor_blocked`.
    let thrust_took = thrust_stab_tick(app, prev_mine_released, prev_mine_down, cursor_blocked);
    if cursor_blocked {
        app.edit_target = None;
        app.mine_target = None;
        app.mine_timer = 0.0;
        return;
    }
    // S1-P-i38 — the recall / magic-mirror / ice-mirror / potion-of-return
    // `ItemCheck` arms (`Player.cs:42976-42999` / `:43107-43140` /
    // `:43141-43172`, `ItemCheck_OwnerOnlyCode`): like the boss spawners they
    // run in the item-use tick, not the cursor/reach path — the mirror fires
    // mid-swing at `itemTime == useTime/2` regardless of where the cursor
    // points. The machine sits with the boss-spawner block (after the
    // map/HUD/frozen gates, before the reach gate). The press edge is the true
    // previous-frame pair (the port overwrote `mine_prev_down` to the current
    // frame at the top). A started use consumes the tick like the boss arms;
    // a live swing's per-tick arm runs without consuming.
    if crate::recall_teleport::recall_item_check(app, prev_mine_released && !prev_mine_down) {
        return;
    }
    // The boss spawners (`Player.ItemCheck_UseBossSpawners`,
    // `Player.cs:43751-43900`) and the event items (`ItemCheck_UseEventItems`,
    // `:43921-44055`) fire MID-SWING: their gate is `ItemTimeIsZero &&
    // itemAnimation > 0` (`:43753`, `:43923` etc.) — the use START
    // (`ApplyItemAnimation`) already ran on the press tick, the summon fires
    // on the first action tick inside the animation and `ApplyItemTime`
    // re-arms the cadence. The port starts the swing here on the press edge
    // and fires the summon when `item_time` hits zero inside the running
    // swing. A REFUSED summon (day gate, living boss) keeps re-checking each
    // action tick — the C# gate never consumes the tick on a refusal — so the
    // early return only covers the "not an action tick yet" wait.
    if app.mine_down && !app.inventory_open {
        let held = app.inventory[app.selected];
        let is_summon = matches!(
            held.kind,
            43 | 361 | 602 | 1844 | 1958 | 3601 | 1315 | 70 | 560 | 1133 | 1331 | 544 | 556 | 557
        );
        if is_summon && held.count > 0 {
            if app.attack_cd <= 0.0 && app.reuse_delay <= 0.0 && app.mine_released {
                start_use_timers(app, held);
                // The summon fires on the START tick itself: the C# gate is
                // `ItemTimeIsZero && itemAnimation > 0` (`Player.cs:43753`),
                // and `StartActualUse` leaves `itemTime` at 0 while the
                // animation is full — so the first action tick IS the press
                // tick (`ItemAnimationJustStarted`, `:4066`). The head decay
                // already ran on the prior zero state, so `item_time` stays 0
                // here and the summon arms below fire this tick.
                app.item_time = 0.0;
            }
            if app.swing <= 0.0 || app.item_time > 0.0 {
                // Not an action tick yet — the summon waits; the tick is
                // consumed so no other use arm fires under it.
                return;
            }
        }
        if held.kind == 43 && held.count > 0 && use_suspicious_looking_eye(app) {
            return;
        }
        // The event items ride the same tick (`ItemCheck_UseEventItems`,
        // `Player.cs:43920-43940`): the Goblin Battle Standard 361 starts
        // the invasion through `CanStartInvasion(1, ignoreDelay: true)`.
        if held.kind == 361 && held.count > 0 && use_goblin_battle_standard(app) {
            return;
        }
        // `ItemCheck_UseBossSpawners`' Frost Legion arm (`Player.cs:43941-43956`):
        // the Snow Globe 602 — `CanStartInvasion(2, ignoreDelay: true)`, the
        // roar, then `invasionDelay = 0; StartInvasion(2)` while no invasion
        // is active.
        if held.kind == 602 && held.count > 0 && use_snow_globe(app) {
            return;
        }
        // `ItemCheck_UseEventItems`' moon arms (`Player.cs:43974-43985`,
        // `:44041-44052`): the Pumpkin Moon Medallion 1844 → `startPumpkinMoon`,
        // the Naughty Present 1958 → `startSnowMoon` — the
        // `!dayTime && !pumpkinMoon && !snowMoon && !DD2Event.Ongoing` gates,
        // the `ApplyItemTime` re-arm and `SoundEngine.PlaySound(15, ...)`. The
        // C#'s mid-swing `ItemTimeIsZero && itemAnimation > 0` precondition
        // rides the summon gate above; the `Main.NewText` row is UI — out.
        if held.kind == 1844 && held.count > 0 && use_pumpkin_moon_medallion(app) {
            return;
        }
        if held.kind == 1958 && held.count > 0 && use_naughty_present(app) {
            return;
        }
        // `ItemCheck_UseBossSpawners`' Celestial Sigil arm (`Player.cs`:
        // 44028-44034`): the 3601 use gates on `downedGolemBoss && hardMode
        // && !AnyDanger && !AnyoneNearCultists` and calls
        // `WorldGen.StartImpendingDoom(720)` — the countdown's item value
        // (`NPC.cs:6037`).
        if held.kind == 3601 && held.count > 0 && use_celestial_sigil(app) {
            return;
        }
        // `ItemCheck_UseBossSpawners`' Pirate Invasion arm
        // (`Player.cs:43957-43971`): the Pirate Map 1315 — the
        // `CanStartInvasion(3, ignoreDelay: true)` gate, the roar, then
        // `invasionDelay = 0; StartInvasion(3)` while no invasion is
        // active.
        if held.kind == 1315 && held.count > 0 && use_pirate_map(app) {
            return;
        }
        // `ItemCheck_Wiring_Gate`'s Eternia arm (`Player.cs:32425-32444`):
        // the Etherian Crystal 3828 over the Eternia Crystal Stand 466 —
        // the gates (`no ongoing event`, `no crystal`, `no moon events`,
        // the 3828 in the inventory, `WouldFailSpawningHere`'s refusal),
        // the consume, then `DD2Event.SummonCrystal` (the frame offset's
        // spawn point, `StartInvasion`).
        if held.kind == 3828 && held.count > 0 && use_etherian_crystal(app) {
            return;
        }
        // `ItemCheck_UseBossSpawners`' pre-hardmode summons
        // (`Player.cs:43751-43900`): the Worm Food 70, the Slime Crown
        // 560, the Abeemination 1133, the Bloody Spine 1331 (the eye 43
        // keeps its own fn above).
        if held.count > 0 && use_boss_summon_item(app) {
            return;
        }
    }
    let world_pos = app.camera + (m - screen * 0.5) / app.zoom;
    // S1-P-i36 — the bug net's catch arm (`Player.ItemCheck`'s
    // `Player.cs:43648-43661`): the swung-item rectangle is PLAYER-relative,
    // so the catch runs ahead of the cursor-reach gate (the C# `ItemCheck`
    // never reach-gates the melee hitbox block). The nets never mine
    // (`pick/axe/hammer` 0) and their `damage` −1 keeps them out of
    // `MeleeHitNPCs` (`:43662`), so the arm consumes the whole use.
    if crate::bug_net::update_bug_net(app) {
        app.edit_target = None;
        app.mine_target = None;
        app.mine_timer = 0.0;
        return;
    }
    // S1-P-i36 — the makeNPC release arm (`Player.cs:43614-43625`): the
    // swing starts regardless of reach; only the release itself is
    // reach-gated inside (`ItemCheck_ReleaseCritter`, `:44064`). LMB on a
    // captured-critter item releases it (the RMB cage/anchor placement
    // keeps the generic place path below).
    if crate::bug_net::update_critter_release(app, world_pos) {
        app.edit_target = None;
        app.mine_target = None;
        app.mine_timer = 0.0;
        return;
    }
    let tx = (world_pos.x / TILE_PX).floor() as i32;
    let ty = (world_pos.y / TILE_PX).floor() as i32;
    if !app.world.in_bounds(tx, ty) {
        app.edit_target = None;
        return;
    }
    // The reach gate: the borrow ends at the statement (the arms below
    // re-borrow `app.player` individually, and the shooters' swing start
    // mutates `app` after the gate).
    let player_in_reach = app.player.as_ref().is_some_and(|p| {
        let (pcx, pcy) = p.center();
        let dx = tx as f32 * TILE_PX + 8.0 - pcx;
        let dy = ty as f32 * TILE_PX + 8.0 - pcy;
        // The port's circular gate is the documented stand-in for the C#'s
        // `IsInTileInteractionRange(..., TileReachCheckSettings.Simple)`
        // rectangle (`TileReachCheckSettings.cs:22-70`, X 5 / Y 3) — and its
        // radius now tracks `Player.tileRangeX` (`Player.cs:2496`), so the
        // journey `FarPlacementRangePower`'s doubling (`:18815-18828`) widens
        // the reach exactly as it does in the vanilla. Base 5 + 1 = the
        // previous 6-tile constant, so nothing off the power moves.
        let reach = (p.tile_range_x() + 1.0) * TILE_PX;
        dx * dx + dy * dy <= reach * reach
    });
    if !player_in_reach {
        app.edit_target = None;
        app.mine_target = None;
        app.mine_timer = 0.0;
        return;
    }
    app.edit_target = Some((tx, ty));
    // `Player.cs:25446` — the smart cursor re-points the tile target
    // (`SmartCursorHelper.SmartCursorLookup`) before the mining/placement
    // arms below run; the vanilla writeback sets `Player.tileTargetX/Y`
    // alongside `Main.SmartCursorX/Y` (`:157-162`).
    let (tx, ty) = match super::smart_cursor::repoint(app, tx, ty, (world_pos.x, world_pos.y)) {
        Some((sx, sy)) => {
            app.edit_target = Some((sx, sy));
            (sx, sy)
        }
        None => (tx, ty),
    };
    #[cfg(test)]
    if std::env::var("AMMO_TRACE").is_ok() {
        eprintln!("[trace] past the reach gate");
    }
    let held = app.inventory[app.selected];
    #[cfg(test)]
    eprintln!(
        "[flow-dbg] reached: kind={} mine_down={}",
        held.kind, app.mine_down
    );
    // `Player.Update`'s early-dismount head (`Player.cs:24992-24995`) +
    // `ItemCheck_ManageRightClickFeatures` (`:31695-31702`): using an item
    // dismounts the mounts that say so — the held item being the very same
    // mount is exempt. The gate is `TryingToUseItem` (`:24319-24338`): the
    // use button held, a usable held item, and NOT (`itemAnimation == 0 &&
    // !Main.mouseLeftRelease`) — `Main.mouseLeftRelease` mirrors the CURRENT
    // button state (`app.mine_released = !mine_down`, the tick head), so on
    // the press tick the gate is closed and the dismount fires from the
    // next tick while the animation runs. `TryingToUseItem`'s can-use half
    // reads `ignoreCursed: true`, so the transformation mounts' `noItems`
    // lock never blocks the dismount itself — it runs BEFORE the use-start
    // refusal below, exactly like the C# order.
    let trying_to_use_item = app.mine_down
        && !app.inventory_open
        && held.count > 0
        && crate::item::use_style(held.kind) != 0
        && (app.swing > 0.0 || app.mine_released);
    if trying_to_use_item {
        if let Some(p) = app.player.as_mut() {
            if p.mount.is_active()
                && p.mount.dismount_on_item_use()
                && crate::mount::mount_type_of_item(held.kind) != Some(p.mount.kind())
            {
                // `:31697` — `TryEarlyDismount` (the gated path).
                p.try_early_dismount(&app.world);
                // `:31698-31701` — a mount that survived the dismount attempt
                // (CCed / no space) cancels the item use this tick.
                if p.mount.is_active() {
                    app.mine_target = None;
                    app.mine_timer = 0.0;
                    return;
                }
            }
        }
    }
    // `Player.ItemCheck_CheckCanUse_Inner`'s noItems gate (`Player.cs:52140-
    // 52148`) inside `ItemCheck_TryStartUse` (`:52023`): a Cursed (23) or
    // transformation-mount item lock refuses EVERY use start — weapons,
    // potions, saddles and mining alike — unless a dismounts-on-item-use
    // mount can dismount (`can_use_item_past_no_items`). The early-dismount
    // head above already freed the ones that could; what remains refuses.
    if app.player.as_ref().is_some_and(|p| !p.can_use_item_past_no_items(&app.world, false)) {
        app.mine_target = None;
        app.mine_timer = 0.0;
        return;
    }
    let held_weapon = held.count > 0 && registry::is_weapon(held.kind);
    // `Player.ItemCheck`'s Snake Band repeat swing (`Player.cs:42711-42731`):
    // with `accSnakeBand`, every third swing (`repeatWhipSwings % 3 == 0`)
    // fires a second whip at the animation's half-way point
    // (`itemAnimation == (int)(itemAnimationMax * 0.5f)`), via
    // `ItemCheck_Shoot(..., withAudioVisualFeedback: false)`. The port's
    // `app.swing` counts DOWN from `swing_max`, so the C# countdown value
    // `itemAnimation` maps to `ticks_of(app.swing)`. The refire picks the
    // next whip in the inventory when `maxTagEffects > 1` (`:42717-42728`);
    // the port fires the held whip (the inventory-scan variant is a
    // documented follow-up). The trigger arms `whip_snake_refire` for the
    // fire arm below and spawns the refire projectile directly (the C#
    // calls `ItemCheck_Shoot` mid-animation, not at the use start).
    app.whip_snake_refire = false;
    if let Some(p) = app.player.as_ref() {
        if p.acc_snake_band
            && held.count > 0
            && held.kind > 0
            && tr_core::whip::is_whip(registry::projectile(held.kind).unwrap_or(0))
            && p.repeat_whip_swings % 3 == 0
            && app.swing > 0.0
        {
            let anim = ticks_of(app.swing);
            let anim_max = ticks_of(app.swing_max);
            if anim > 0 && anim == (anim_max as f32 * 0.5) as i32 {
                app.whip_snake_refire = true;
            }
        }
    }
    if app.whip_snake_refire {
        // The refire is a direct `ItemCheck_Shoot(..., withAudioVisualFeedback:
        // false)` call (`Player.cs:42729`) — it spawns mid-animation without
        // the use-start timers. The aim/spawn mirror the fire path: the mouse
        // direction normalized to `shootSpeed`, the spawn at the player
        // center + 10 px along the aim. The damage is the weapon damage
        // (`GetWeaponDamage`, `:42653`), the knockback the item's.
        if let (Some(def), Some(proj)) = (registry::get(held.kind), registry::projectile(held.kind)) {
            let held_fx = crate::prefix::prefixed_stats(held.kind, held.prefix);
            let (fx_dmg, fx_spd, fx_kb) = held_fx.map_or(
                (def.damage, def.shoot_speed, def.knock_back),
                |s| (s.damage, s.shoot_speed, s.knock_back),
            );
            // The aim/spawn scalars read the player immutably; the borrow
            // ends before the spawn helper re-borrows the pool/player/rng.
            let aim = app.player.as_ref().map(|p| {
                let (pcx, pcy) = p.center();
                let (sw, sh) = app.screen_size();
                let mouse_world = Vec2::new(
                    app.camera.x + (app.mouse_pos.x - sw * 0.5) / app.zoom,
                    app.camera.y + (app.mouse_pos.y - sh * 0.5) / app.zoom,
                );
                let dir = (mouse_world - Vec2::new(pcx, pcy)).normalize_or_zero();
                let raw = whip_weapon_damage(fx_dmg, p.weapon_damage_multiplier(def.class));
                (dir, raw)
            });
            if let Some((dir, raw)) = aim {
                if dir != Vec2::ZERO {
                    let (pcx, pcy) = app.player.as_ref().unwrap().center();
                    let spawn = Vec2::new(pcx, pcy) + dir * 10.0;
                    let shot = dir * fx_spd;
                    let loot_rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
                    spawn_whip_shot(
                        &mut app.projectiles,
                        app.player.as_mut(),
                        loot_rng,
                        held,
                        def,
                        held_fx,
                        proj,
                        spawn,
                        shot,
                        raw,
                        fx_kb,
                        true,
                    );
                }
            }
        }
        app.whip_snake_refire = false;
    }
    // `Player.ItemCheck`'s use-start gate (`Player.cs:42733`): `controlUseItem
    // && releaseUseItem && itemAnimation == 0` — ONE use per press edge
    // (S1-P-i37: the old ungated `mine_down` block re-drank every tick while
    // LMB was held, draining the stack and re-arming buff 21 forever). The
    // captured previous-frame edges are the `releaseUseItem` stand-in; the
    // swing timers are the `itemAnimation == 0` half.
    let use_press_edge = app.mine_down
        && prev_mine_released
        && !prev_mine_down
        && app.attack_cd <= 0.0
        && app.reuse_delay <= 0.0;
    // `Player.ItemCheck`'s consumable arms: the food (buff 26/206/207,
    // gated by `CanUseItem`'s wellFed check `:14353`) and the drink family
    // (the healing/mana/buff potions — `potions.rs`, S1-P-i37).
    if use_press_edge && held.count > 0 && !app.inventory_open {
        let def = registry::get(held.kind);
        if def.is_some_and(|d| d.consumable && matches!(d.buff_type, 26 | 206 | 207)) {
            let Some(def) = def else { unreachable!() };
            // The sim-context player borrow is immutable; the fed flag was
            // derived by this tick's scan, so the gate reads it directly.
            let can_eat = !app.player.as_ref().is_some_and(|p| p.well_fed);
            if can_eat {
                let time = if def.buff_time > 0 {
                    def.buff_time
                } else {
                    14400 // the 4-minute vanilla fallback
                };
                if let Some(p) = app.player.as_mut() {
                    p.buffs.add_buff(def.buff_type, time);
                    p.eat_food();
                }
                app.inventory[app.selected].count =
                    app.inventory[app.selected].count.saturating_sub(1);
                if app.inventory[app.selected].count == 0 {
                    app.inventory[app.selected].kind = 0;
                }
                start_use_timers(app, held);
                app.mine_target = None;
                app.mine_timer = 0.0;
                return;
            }
        }
        // The drink family (S1-P-i37 — `potions.rs`): the healing potions
        // (`item.potion`, `Player.cs:42756-42758` + `:42867-42875`), the mana
        // potions (`healMana`, NOT potion-flagged — `Item.cs:2680-2692`) and
        // the buff potions (`buffType > 0`, `:42876-42883`). The Potion
        // Sickness gate (`ItemCheck_CheckCanUse_Inner`, `:52217-52220`), the
        // per-kind delays (`ApplyPotionDelay` `:51853-51886`), the Mana
        // Sickness grant (`:51938`) and the stack consume all live there.
        if crate::potions::use_potion_item(app, held) {
            return;
        }
        // (Resync keep-one: the gate's i19 inline mana-potion arm was
        // dropped — the i37 `use_potion_item` seam above owns EVERY drink
        // (the mana potions are `healMana > 0` drinkables, `Item.cs:2680-
        // 2692`), including the buff-94 grant + the ManaEffect text + the
        // `UndoSlowMagicUse` half (`Player.cs:51946-51948`/`:51954-51966`,
        // folded into `potions::use_potion_item` — the i37 coordination
        // note's deferred i19 fold).)
    }
    // The held item's prefixed stats (`Item.Prefix`'s application on a
    // fresh item, recomputed — see prefix.rs): the swing pacing, the mana
    // cost and the throw damage read the instance values. The throw's crit
    // is per-hit now (the projectile's class flags), not a spawn roll.
    let held_fx = crate::prefix::prefixed_stats(held.kind, held.prefix);
    let held_use_ammo = shooter_use_ammo(held.kind);
    // `Player.channel` (`Player.cs:52958-52968`): while the button is held
    // on a `channel` item the aim target rides the player for the aiStyle-9
    // spells (`AI_009_MagicMissiles`'s owner block, `Projectile.cs:43239-
    // 43249`). The C# ARMS the channel at the use start (`StartChanneling
    // (sItem)` inside `ItemCheck_StartActualUse`, `:51677`) and CLEARS it on
    // the button release (`ItemCheck` `:42804-42807` — the hoisted clear at
    // the tick head), a CC (`:42598-42603`), the tracked shot's death
    // (`TryCancelChannel` `:52975-52981`, the pool kill hook) or the dead
    // tick (`UpdateDead` `:17017`) — NOT per-tick from the button state. The
    // port keeps that latch: the arm rides the use start below; this block
    // only refreshes the aim while the channel is open (S1-P-i23).
    {
        let holding_channel_item = registry::get(held.kind).is_some_and(|d| d.channel)
            && held.count > 0;
        // S1-P-i54 — `ItemCheck` (`Player.cs:42800-42807`): a buffered
        // selection change forces `flag5 = false`, which both clears
        // `channel` and blocks the use path — so the hold cannot re-arm
        // the channel while the switch is pending (the player pass's
        // interrupt clears it; this gate keeps it cleared until the
        // switch lands). The i23 latch survives the aim refresh — only
        // the buffered-switch term drops it here (the release/CC/switch
        // clears run at the hoisted tick-head gate + the else-if below).
        let holding = holding_channel_item
            && app.mine_down
            && held_weapon
            && !app.inventory_open
            && !app.selection.has_buffered_change();
        if let Some(p) = app.player.as_mut() {
            if p.channel
                && holding
                && held.kind == p.channel_item
            {
                let (cx, cy) = p.center();
                let target = limit_point_to_player_reachable_area(
                    Vec2::new(cx, cy),
                    world_pos,
                    &app.world,
                );
                p.channel_item = held.kind;
                // `HeldItem.shoot` (`Projectile.cs:43241`): the aiStyle-9
                // spells compare the held item's PROJECTILE id against their
                // own kind — not the item id (S1-P-i20).
                p.channel_shoot = registry::projectile(held.kind).unwrap_or(0);
                p.channel_target = target;
            } else if !p.channel && holding_channel_item && !held_weapon && app.mine_down && !app.inventory_open {
                // The projectile-less channel items (the mounts'
                // `StartChanneling()` `Player.cs:52941`, the parameterless
                // overload — no cancel key; the fairy bell): the port's use
                // path only arms the weapon channel at the shot start, so
                // this family keeps its per-tick arm (the pre-i23 behaviour
                // for the `!held_weapon` family).
                p.channel = true;
                p.channel_item = held.kind;
                p.channel_proj_expected = 0;
                p.channel_index_expected = 0;
            } else if p.channel {
                // Gate-merge keep-both: HEAD's release/switch clear (i20's
                // seam — `channel_shoot` zeroes with the rest) stays for the
                // weapon family; i23's hoisted tick-head clear owns the
                // canonical release path.
                p.channel = false;
                p.channel_item = 0;
                p.channel_shoot = 0;
                p.channel_target = Vec2::ZERO;
            }
        }
    }
    // S1-P-i35 — the fishing rod's use chain (`Player.ItemCheck_TryStartUse`
    // → `ItemCheck_PullFishingBobbers` `Player.cs:52368-52399`, then
    // `ItemCheck_Shoot`'s bobber spawn `:47181-47184`/`:49437`). A live
    // bobber makes the use a REEL (the pull), never a cast; the cast fires
    // only when no bobber is out. The pulley gate (`:52055-52058`) blocks
    // the whole use while roped.
    let held_pole = registry::get(held.kind).map_or(0, |d| d.fishing_pole);
    if held_pole > 0 && app.mine_down && !app.inventory_open {
        let pulley = app.player.as_ref().is_some_and(|p| p.pulley);
        if !pulley && fishing_use(app, held, world_pos, prev_mine_released && !prev_mine_down) {
            return;
        }
    }
    if app.mine_down && held_weapon && !app.inventory_open {
        // Weapons use LMB toward the cursor instead of mining. The shuriken
        // is not `autoReuse` (`Item.cs:1869-1885`): one throw per press edge
        // (`Player.cs:42733` `controlUseItem && releaseUseItem && itemAnimation == 0`).
        // The frame-head auto-reuse block already forced `mine_released` for
        // the autoReuse/glove/setting arms, so the edge read covers both.
        let throw_ready = app.mine_released;
        // The ammo shooters take the shooter path below — the throw's
        // `Item.projectile` is the ammo's projectile for them, and the shot
        // fires at `ItemTimeIsZero` mid-swing, not on the press.
        #[cfg(test)]
        eprintln!(
            "[shooter-dbg] mine_down={} held_weapon={} cd={} reuse={} ready={throw_ready}",
            app.mine_down, held_weapon, app.attack_cd, app.reuse_delay
        );
        if held_use_ammo > 0 && app.attack_cd <= 0.0 && app.reuse_delay <= 0.0 && throw_ready {
            // `ItemCheck_TryStartUse` → `ItemCheck_CheckCanUse_Inner` →
            // `HasAmmo` (`Player.cs:52021-52023`, `:52316-52319`,
            // `:53431-53446`): with NO matching ammo stack anywhere the use
            // is REFUSED — no swing starts, no timer arms (S1-P-i15; the old
            // "dry swing" was a divergence — the C#'s `canShoot = false`
            // early-return only fires mid-swing when the last ammo drained).
            if !crate::ammo::has_ammo(&app.inventory, &app.equip, held_use_ammo) {
                return;
            }
            // The shooter's use start — the ammo exists (`HasAmmo` above);
            // the shot itself fires at `ItemTimeIsZero` (the fire block
            // below).
            // `ApplyItemAnimation` (`Player.cs:4463-4497`): the gunProj four
            // keep their id-keyed vanilla animation but still take the
            // `num` bump (`:4466-4469` + the `:4494` tail — all four are
            // autoReuse style-5 shooters with `useTime <= useAnimation`,
            // `shoot > 0` and `reuseDelay == 0`, so each swings ONE tick
            // longer than its raw `useAnimation`); everything else runs the
            // branch chain. (S1-P-i12)
            app.swing_max = gun_proj_row(held.kind).map_or_else(
                || apply_item_animation_secs(app, held),
                |g| (g.use_animation + 1) as f32 * ONE_TICK_SECS,
            );
            app.swing = app.swing_max;
            start_swing_extras(app, held.kind);
            arm_reuse_delay(app, held);
            // `ItemCheck_StartActualUse` (`Player.cs:51657-51697`) sets the
            // animation and leaves `itemTime` alone — the C# decrements it
            // (`:42836`) BEFORE the shoot gate (`ItemCheck_OwnerOnlyCode`
            // `:42854`), so the FIRST shot of the swing fires one tick after
            // the start at `itemAnimation == itemAnimationMax - 1`, which IS
            // `ItemAnimationJustStarted` (`:4066`). The port's decay sits at
            // the frame HEAD (before this start block), so arming ONE tick
            // (not zero — a zero would fire here and now, at the still-
            // undecayed `swing == max`) puts that first shot on the next
            // frame, whose head decay has already taken the swing to
            // `max - 1`: the tick value the volley windows below derive from.
            app.item_time = ONE_TICK_SECS;
            app.attack_cd = app.swing_max;
            app.shot_fired = false;
            app.world_key = None;
        }
        // M4-PROJ-8 Phase 6 — `ItemCheck_MinionAltFeatureUse`
        // (`Player.cs:49773-49780`): the summon staff's right-click picks the
        // minions' target. The port's right-button press edge stands for the
        // C#'s `altFunctionUse == 2`; the feature set gates which staves.
        let targeting_staff = registry::projectile(held.kind)
            .is_some_and(|proj| crate::projectile::MINION_TARGETING_FEATURE.contains(&proj));
        if targeting_staff
            && app.place_down
            && !app.place_prev_down
            && app.attack_cd <= 0.0
            && app.reuse_delay <= 0.0
        {
            start_use_timers(app, held);
            // `MinionNPCTargetAim` (`Player.cs:56913-56931`): the nearest
            // chaseable NPC to the mouse world point; the same target clears.
            let (sw, sh) = app.screen_size();
            let mouse_world = Vec2::new(
                app.camera.x + (app.mouse_pos.x - sw * 0.5) / app.zoom,
                app.camera.y + (app.mouse_pos.y - sh * 0.5) / app.zoom,
            );
            if let Some(p) = app.player.as_mut() {
                // `ItemCheck_MinionAltFeatureUse` passes
                // `doNotDisableIfTheTargetIsTheSame: false` (`Player.cs:49777`).
                p.minion_npc_target_aim(&app.npcs, mouse_world, false);
            }
        }
        // S1-P-i29/i30 — the drills and chainsaws (a `channel` item with
        // `pick > 0` or `axe > 0`) are mining tools, not throws: their
        // projectile spawns once via the channel start
        // (`drill_channel_start`, `ItemCheck_StartActualUse` →
        // `StartChanneling`, `Player.cs:51677`/`:52958`) and the aiStyle-20
        // machine holds it. The generic throw would re-spawn the tool every
        // swing — gate it out here (the C# `ItemCheck_Shoot` fires once per
        // use start, and the channel holds the animation so no new use
        // starts while held).
        let drill_tool =
            registry::get(held.kind).is_some_and(|d| d.channel && (d.pick > 0 || d.axe > 0));
        if app.attack_cd <= 0.0
            && app.reuse_delay <= 0.0
            && throw_ready
            && held_use_ammo == 0
            && !drill_tool
        {
            // S1-P-i26 — the DD2 sentry use gates (`ItemCheck_CheckCanUse_Inner`,
            // `Player.cs:52079-52111`): the crystal-payment lock, the
            // downed-tier/event gate, the sentry-overlap refusal and the
            // solid-tile refusal. The C# runs them in `CheckCanUse` BEFORE the
            // use starts (and BEFORE the boomerang stack limits below —
            // :52079 < :52162), so a refusal here skips the whole fire block
            // (no swing, no spawn). `downedDD2EventAnyDifficulty` (`:1916`) is
            // the per-tick latch of `DD2Event.DownedInvasionAnyDifficulty`
            // (`:24450-24453`, `DD2Event.cs:89-99`) — the port reads the
            // world's three tier flags directly (both monotonic).
            if let Some(proj) = registry::projectile(held.kind) {
                if tr_core::dd2::is_dd2_turret(proj) {
                    let (cx, cy, facing) = app
                        .player
                        .as_ref()
                        .map(|p| {
                            let (x, y) = p.center();
                            (x, y, p.facing)
                        })
                        .unwrap_or((app.camera.x, app.camera.y, 1));
                    let downed_any = app.world.dd2_downed_t1
                        || app.world.dd2_downed_t2
                        || app.world.dd2_downed_t3;
                    if !sentry::dd2_sentry_can_use(
                        &app.world,
                        &app.projectiles,
                        &app.inventory,
                        &app.equip,
                        held.kind,
                        proj,
                        world_pos,
                        Vec2::new(cx, cy),
                        facing,
                        app.world.dd2_event_ongoing,
                        downed_any,
                    ) {
                        return;
                    }
                }
            }
            // `ItemCheck_CheckCanUse_Inner`'s boomerang stack limits
            // (`Player.cs:52162-52186`, S1-P-i08): the single-flight list
            // (`:52162` — 6/19/33/52/113/320/333/383/491/867/1052/902/866/
            // 301 + the phaseblades; the ported kinds are 6/52/113) refuses
            // the use while ANY active same-kind projectile of this owner
            // exists; the Light Disc 106 (`:52172-52186`) counts and refuses
            // at `>= 6`. The 272 (`>= 10`, `:52187`) and 1000 (`>= 3`,
            // `:52202`) rows are unported kinds. The gate runs at the use
            // start, ahead of the mana pay (`ItemCheck_TryStartUse`
            // `:52023-52026`).
            if let Some(shoot) = registry::projectile(held.kind) {
                let limit = match shoot {
                    crate::projectile::LIGHT_DISC => Some(6usize),
                    crate::projectile::ENCHANTED_BOOMERANG
                    | crate::projectile::WOODEN_BOOMERANG
                    | crate::projectile::ICE_BOOMERANG => Some(1),
                    _ => None,
                };
                if let Some(limit) = limit {
                    let in_flight = app
                        .projectiles
                        .slots()
                        .iter()
                        .filter(|p| {
                            p.active && p.owner == app.local_id as u8 && p.kind == shoot
                        })
                        .count();
                    if in_flight >= limit {
                        return;
                    }
                }
            }
            // `Player.ItemCheck_ActuallyPayMana` → `CheckMana`
            // (`Player.cs:52638-52655`, `:52576-52617`): the magic items pay
            // `(int)(item.mana * manaCost)` — the multiplier the armor pieces,
            // the accessories, the casters' sets and Magic Power lower (M3-
            // EQUIP-9). Under the shipped `DebugOptions.ManaV2`
            // (`DebugOptions.cs:85`, S1-P-i19) the check NEVER refuses: enough
            // mana pays, not enough drains the bar to 0 and arms the 2.5×
            // slow cast (`slowMagicUse`, `DoRunOutOfManaEffect` `:52619-52622`).
            // CheckMana itself does NOT apply the regen delay — the swing's
            // per-anim-tick `ItemCheck_ApplyManaRegenDelay` (`:42810-42813`)
            // owns it; the start tick's install below stands in for that
            // first tick (the port's head decay runs before the mid-tick
            // use start).
            let base_mana = held_fx.map(|s| s.mana).unwrap_or(0);
            // `StartChanneling(sItem)`'s `item.channel` gate (`:52960`).
            let def_is_channel = registry::get(held.kind).is_some_and(|d| d.channel);
            // The mutable player borrow covers the whole throw (the mana
            // pay + the reads the block makes through it).
            let Some(p) = app.player.as_mut() else {
                unreachable!()
            };
            // `Player.ItemCheck_CheckCanUse` (`Player.cs:52221-52224`): the
            // Silenced debuff (35) refuses every mana-paying item — the gate
            // reads the item's BASE mana, before the manaCost scale (S1-E-7
            // wave A; the buff arm is `UpdateBuffs` `:10043`). This refusal is
            // upstream of CheckMana and survives ManaV2.
            if base_mana > 0 && p.silence {
                return;
            }
            // `Player.ItemCheck_CheckCanUse_Inner` (`Player.cs:52112-52115`): a wet
            // (non-lava) player refuses the fire spells — the Flamelash (shoot 34)
            // among the listed staves; the 85/15/1122 kinds keep the verbatim set.
            // `base.wet` is `WetCollision` (ANY liquid, `Collision.cs:1605-1645`); the
            // port splits the kinds (`player.rs:4315-4358`), so the water/honey/
            // shimmer union stands in, and `!lavaWet` is the lava flag itself.
            if (p.wet || p.honey_wet || p.shimmer_wet)
                && !p.lava_wet
                && matches!(registry::projectile(held.kind), Some(85 | 15 | 34 | 1122))
            {
                return;
            }
            // S1-C-12 wave D — `ItemCheck_PayMana_ShouldSkipManaUse`'s
            // spaceGun arm (`Player.cs:52695-52698`): the Meteor set makes
            // the four laser weapons cost no mana; the same ids skip the
            // regen delay (`ItemCheck_ApplyManaRegenDelay`, `:50587-50593`)
            // — no pay, no delay. The 3269 arm (`:52691-52694`) and the
            // turret/minion alt-fire arms (`:52699-52706`) stay out (no
            // port items/seam).
            let skip_mana = crate::armor_sets::space_gun_skips_mana(p.space_gun, held.kind);
            if base_mana > 0 && !skip_mana {
                // `CheckMana(rawAmountToPay, pay: true)` (`:52650`): the
                // entry clears `slowMagicUse` (`:52578`); the ManaV2 empty
                // arm re-arms it. `ItemCheck_PayMana_GetManaCostToPay`
                // (`:52679-52687`) is the plain `item.mana` here (the 3852
                // alt-fire ×2 has no port seam).
                crate::regen::check_mana(
                    &mut p.mana,
                    &mut p.slow_magic_use,
                    base_mana,
                    p.mana_cost,
                    true,
                );
                // The use-start tick's `ItemCheck_ApplyManaRegenDelay`
                // (`Player.cs:42808-42813` → `:50587-50593`): the animation
                // was just armed by `StartActualUse`, so the delay re-arms
                // from the POST-pay mana this same tick. The ongoing
                // animation ticks re-arm it at the `update_editing` head.
                crate::regen::apply_mana_regen_delay(
                    &mut p.regen,
                    p.mana_max,
                    p.mana,
                    p.mana_regen_buff,
                );
            } else if base_mana > 0 {
                // spaceGun skip: `ItemCheck_ActuallyPayMana`'s skipUsageCheck
                // arm clears `slowMagicUse` (`:52645-52648`) and pays nothing.
                p.slow_magic_use = false;
            }
            // `Player.StartChanneling(sItem)` (`Player.cs:52958-52968`, called
            // by `ItemCheck_StartActualUse` `:51677` AFTER the mana pay): a
            // `channel` item arms the hold and seeds the cancel key with its
            // `shoot` — the tracked-shot index resets to the C# struct's 0
            // default (`_channelShotCache = new ChannelCancelKey { ... }`).
            // The aim refresh rides the channel block above on the next tick
            // (the C# reads `Main.MouseWorld` live in the AI, the port
            // snapshots it); seed it here so the spawn tick already carries
            // the clamped cursor (S1-P-i23).
            if def_is_channel {
                let (cx, cy) = p.center();
                p.channel = true;
                p.channel_item = held.kind;
                p.channel_target = limit_point_to_player_reachable_area(
                    Vec2::new(cx, cy),
                    world_pos,
                    &app.world,
                );
                p.channel_proj_expected = registry::projectile(held.kind).unwrap_or(0);
                p.channel_index_expected = 0;
                // S1-P-i20 seam (gate-merge keep-both): the held item's shoot
                // id rides `channel_shoot` alongside i23's cancel-key field;
                // the aiStyle-9 hold gate resolves via channel_item, this
                // field stays the i20 witness seam.
                p.channel_shoot = registry::projectile(held.kind).unwrap_or(0);
            }
            // `SetItemAnimation`'s tail (`Player.cs:4496`): the reuse lock
            // rides the use start (the prefixed instance value). The direct
            // field write keeps the player borrow (`p`) alive for the throw
            // below — the same value [`arm_reuse_delay`] applies elsewhere.
            app.reuse_delay = crate::prefix::prefixed_stats(held.kind, held.prefix)
                .map(|s| s.reuse_delay)
                .or_else(|| registry::get(held.kind).map(|d| d.reuse_delay))
                .unwrap_or(0) as f32
                * ONE_TICK_SECS;
            // `Player.ItemCheck_Shoot` (`Player.cs:47056+`): the generic
            // `Item.shoot` firing — the aim vector normalized to
            // `shootSpeed` (`:47274-47292`), the spawn at the player center
            // (`RotatedRelativePoint(MountedCenter)`), the damage
            // `weaponDamage` (the class multiplier) and the item's knockback.
            let def = registry::get(held.kind);
            if let (Some(proj), Some(def)) = (registry::projectile(held.kind), def) {
                // `Item.SetDefaults`' hook epilogue (`Item.cs:48893-48898`):
                // every hook item (`Main.projHook[shoot]`, aiStyle 7) has
                // `useStyle = useTime = useAnimation = 0`, so it NEVER fires
                // through `ItemCheck_Shoot`'s LMB swing — the hook launches
                // only via `QuickGrapple` (the `controlHook` edge,
                // `Player.cs:27482-27487`). The port's generic shoot path must
                // skip the hook projectiles (the `TR_HOOK`/item fire lives in
                // `world_sim/player.rs`). The aiStyle-7 test is the
                // authoritative `Main.projHook` (`Main.cs:6620-6624`).
                // (S1-P-i39 — the resync keep-both: this gate runs FIRST,
                // the staff-buff grant below never sees a hook item.)
                if crate::grapple::is_hook_projectile(proj) {
                    return;
                }
                // S1-P-i25 — `Player.ItemCheck_ApplyPetBuffs`' summon-staff
                // grant (`Player.cs:42766` → `:53190-53281`): the use-start
                // `AddBuff(sItem.buffType, 3600)` is the minion-buff
                // PERSISTENCE mechanism — the keep-alive arm's 18000 refresh
                // pins this 3600-tick buff while the minion lives. The port
                // previously granted only the `:42880` `item.buffTime` (0 for
                // the staves — a ghost the sweep drops), so the buff never
                // persisted and AI_062's flag read let the minion despawn.
                // The `MinionTargetingFeature` alt-use gate (`:42764`) folds
                // out: the port's right-click aim is its own seam above and
                // never reaches this left-button shoot path.
                if crate::projectile::MINION_STAFF_PET_BUFF_ITEMS.contains(&held.kind)
                    && def.buff_type > 0
                {
                    p.buffs.add_buff(def.buff_type, 3600);
                }
                // M4-PROJ-8 Phase 6 — the summon use-path, in the C#'s order:
                // `Player.FreeUpPetsAndMinions` frees the budget at the use
                // tick (`Player.cs:42780`) BEFORE the spawn, and the
                // `item.buffType > 0` grant rides the same use
                // (`Player.cs:42879-42884`, the 90/27 exclusions).
                if def.buff_type > 0 && def.buff_type != 90 && def.buff_type != 27 {
                    p.buffs.add_buff(def.buff_type, def.buff_time);
                }
                // `Player.cs:42778` — the free-up gate:
                // `item.shoot >= 0 && item.shoot < ProjectileID.Count &&
                // (LightPet[item.shoot] || Main.projPet[item.shoot])`. The
                // sentry staves' shoot kinds are in NEITHER set, so the C#
                // never frees for them (their refresh rides the shoot arm's
                // own same-kind kill + `UpdateMaxTurrets`); the port
                // previously called the free-up for every shoot > 0.
                if proj > 0
                    && (crate::projectile::PROJ_PET.contains(&proj)
                        || crate::projectile::LIGHT_PET_PROJ.contains(&proj))
                {
                    app.projectiles
                        .free_up_pets_and_minions(p, proj, held.kind, app.local_id as u8);
                }
                // S1-P-i20 — the shot body lives in `fire_non_ammo_shot` (the
                // shared use-start + mid-swing volley seam — the i08
                // boomerang / i16 launcher / i55 golf / i07 dual-flail /
                // i28 javelin content transplanted INTO it at the resync).
                // It returns whether the aim was non-zero (or the boomerang
                // zero-aim fallback fired); the swing tail below rides the
                // USE START only — the C# `ApplyItemAnimation` rides
                // `ItemCheck_StartActualUse` (`:51680`), not `ItemCheck_Shoot`,
                // so a mid-swing volley shot must NOT restart the swing.
                if fire_non_ammo_shot(app, held, held_fx, world_pos, proj, def, screen) {
                    // `Player.ApplyItemAnimation` (`Player.cs:4463-4497`):
                    // the i12 branch chain — melee/summoner speed scales,
                    // the ManaV2 slow cast (×2.5 + the num bump, the i19
                    // fold), tile/wall speed, else useAnimation + num.
                    // `attack_cd` matches so the throw repeats only when
                    // the swing is over.
                    app.swing_max = apply_item_animation_secs(app, held);
                    app.swing = app.swing_max;
                    start_swing_extras(app, held.kind);
                    app.attack_cd = app.swing_max;
                    // S1-P-i20 — `ApplyItemTime` (`Player.cs:47185-47194` →
                    // `SetItemTime(useTime)`, `:4416-4419`): the FIRST shot fired on
                    // the press tick (the `ItemTimeIsZero` gate, `:43544`), and the
                    // shot re-arms the action cadence to `useTime` — the mid-swing
                    // volley gate below reads it. A `useTime < useAnimation` staff
                    // (the Aqua Scepter 8 < 16) fires its whole volley in this swing.
                    app.item_time = shooter_use_time_secs(held.kind, held.prefix, app.swing_max);
                }
            }
            // `NewProjectile`'s `TryUpdateChannel` tail (`Projectile.cs:
            // 11097-11100`): the use-path spawns track the cancel key on the
            // SAME tick they spawn (the C# calls it inside `NewProjectile`,
            // before the next `Update`). The pool queued the intents; apply
            // them to the local player here so a same-tick death matches the
            // fresh index (S1-P-i23). The projectile-pass drain handles the
            // AI-side spawns (the prism lasers, the magnet bolts). A fresh
            // player borrow — `start_swing_extras` above took the whole app.
            if let Some(p) = app.player.as_mut() {
                app.projectiles.apply_channel_tracks(p);
            }
        }


        // The vanilla's `ItemCheck_Shoot` runs inside `ItemCheck`, before the
        // mining/placing arms return (`Player.cs:43681` fires the shot from
        // the swing arm itself), so every item falls through to this pass's
        // melee/mining/placing arms — including the ammo shooters, whose
        // projectile block sits ABOVE this point. The pre-BUG-09 code
        // returned here for the non-ammo case, which made every melee weapon
        // unreachable.
    }
/// S1-P-i35 — the fishing rod's use chain (`Player.ItemCheck_TryStartUse` →
/// `ItemCheck_PullFishingBobbers` `Player.cs:52368-52399` → `ItemCheck_Shoot`
/// `:47056+`). Returns `true` when the use is consumed (a reel OR a cast), so
/// the caller skips the generic weapon/mining arms. The press edge gates the
/// cast (the pole is not `autoReuse`, `Item.cs`'s row); a live bobber makes
/// every press a reel.
fn fishing_use(app: &mut App, held: ItemSlot, world_pos: Vec2, press_edge: bool) -> bool {
    let owner = app.local_id as u8;
    // `ConsumeBait`'s bait scan (`Player.cs:52452-52470`): the ammo slots
    // 54-57 first, then the grid 0-49, matching `localAI[2]` — but the pull
    // needs the bait BEFORE it knows which bobber bites, so resolve the best
    // bait the same way `Fishing_GetBait` does (`:42254-42280`).
    let bait = {
        let Some(p) = app.player.as_ref() else { return false };
        let cond = p.fishing_conditions(
            &app.inventory,
            &app.equip.coins,
            &app.equip.ammo,
            app.selected,
            &app.world,
            app.rain.is_raining(),
            app.rain.cloud_bg(),
        );
        if cond.2 > 0 { Some((cond.3, cond.2)) } else { None }
    };
    let (acc_tackle_box, acc_fishing_line) = app
        .player
        .as_ref()
        .map(|p| (p.acc_tackle_box, p.acc_fishing_line))
        .unwrap_or((false, false));
    let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
    let (no_bobber, bait_use) = app.projectiles.pull_fishing_bobbers(
        owner,
        bait,
        acc_tackle_box,
        acc_fishing_line,
        |kind| registry::get(kind).map_or(0, |d| d.rare),
        rng,
    );
    // The bait drain + the ladybug luck (`ConsumeBait` `:52513-52524`).
    if let Some(use_bait) = bait_use {
        if use_bait.consumed {
            // Find the slot the scan would pick (ammo first, then grid).
            let mut drained = false;
            for sl in app.equip.ammo.iter_mut() {
                if sl.count > 0 && sl.kind == use_bait.bait_type {
                    sl.count -= 1;
                    if sl.count == 0 {
                        sl.kind = 0;
                    }
                    drained = true;
                    break;
                }
            }
            if !drained {
                for sl in app.inventory.iter_mut() {
                    if sl.count > 0 && sl.kind == use_bait.bait_type {
                        sl.count -= 1;
                        if sl.count == 0 {
                            sl.kind = 0;
                        }
                        break;
                    }
                }
            }
            // `:52515-52518` — the ladybug baits 4361/4362 run LadyBugKilled.
            if use_bait.bait_type == 4361 || use_bait.bait_type == 4362 {
                let center = app.player.as_ref().map(|p| p.center());
                if let (Some(p), Some(pos)) = (app.player.as_mut(), center) {
                    let add = p.on_lady_bug_killed(pos, use_bait.bait_type == 4362);
                    app.rain.add_lady_bug_rain_boost(add);
                }
            }
        }
    }
    // The pull's NPC spawns (`PullBobber` `:52403-52437`): the Duke Fishron
    // (370) at the player, the blood-moon enemy at the bobber. The unported
    // enemy kinds (586/587/618/620/621/682) skip the spawn but the 682 still
    // raises `unlockedSlimeRedSpawn` (`:52429-52432`).
    for owner_id in app.projectiles.drain_duke_spawns() {
        if owner_id == owner {
            let center = app.player.as_ref().map(|p| p.center());
            if let Some((cx, cy)) = center {
                let (w, h) = NpcKind::DukeFishron.size();
                let mut n = Npc::new(NpcKind::DukeFishron, cx - w * 0.5, cy - h, 1, 0.0);
                n.hp = n.max_hp;
                app.npcs.push(n);
            }
        }
    }
    for (npc_type, x, y) in app.projectiles.drain_fishing_enemies() {
        if npc_type == 682 {
            app.world.unlocked_slime_red_spawn = true;
        }
        // The blood-moon fish enemies have no NpcKind rows (module doc) —
        // the roll ran verbatim, the spawn is the documented skip.
        let _ = (x, y);
    }
    if !no_bobber {
        // A live bobber: the use is a reel, consumed (no cast).
        return true;
    }
    // No bobber out: the cast fires on the press edge (`ItemCheck_Shoot`).
    if !press_edge || app.attack_cd > 0.0 || app.reuse_delay > 0.0 {
        // The use still consumes (the pole swings) but no cast this tick.
        return true;
    }
    // The player reads are snapshotted before the pool/rng borrows (the
    // disjoint-field capture the shoot path uses).
    let Some((override_bobber, pcx, pcy, facing, class_crit)) = app.player.as_ref().map(|p| {
        let class = registry::get(held.kind).map_or(registry::DamageClass::None, |d| d.class);
        (p.override_fishing_bobber, p.mounted_center().0, p.mounted_center().1, p.facing, p.class_crit(class))
    }) else { return true };
    // `ItemCheck_Shoot`'s bobber override (`Player.cs:47181-47184`): the
    // glow-bobber accessory replaces the pole's own shoot.
    let mut proj = registry::projectile(held.kind).unwrap_or(0);
    if override_bobber > -1 {
        proj = override_bobber as u16;
    }
    if proj == 0 {
        return true;
    }
    let def = registry::get(held.kind);
    let fx = crate::prefix::prefixed_stats(held.kind, held.prefix);
    let speed = fx.map_or(def.map_or(0.0, |d| d.shoot_speed), |s| s.shoot_speed);
    let damage = fx.map_or(def.map_or(0, |d| d.damage), |s| s.damage);
    let kb = fx.map_or(def.map_or(0.0, |d| d.knock_back), |s| s.knock_back);
    // The spawn point + aim (`:47196`/`:47277-47307`): the mounted centre,
    // the aim normalized to shootSpeed (the degenerate aim → facing).
    let spawn = Vec2::new(pcx, pcy);
    let dir = world_pos - spawn;
    let vel = if dir == Vec2::ZERO || dir.is_nan() {
        Vec2::new(facing as f32 * speed, 0.0)
    } else {
        dir.normalize() * speed
    };
    // M3 sync: no throw-time crit roll - the per-hit model is canonical
    // (Projectile.cs:12883-12917, S1-P-i11; the generic ItemCheck_Shoot tail
    // Player.cs:49437 carries no roll). The bobber's damage is -1 so the PVE
    // walk refuses anyway; the cast spends no UpdatePlayers draw.
    let _ = class_crit;
    let slot = app.projectiles.spawn_with_data_ai_owned(
        proj, owner, spawn.x, spawn.y, vel.x, vel.y, damage, kb, 0.0, 0.0,
    );
    let _ = slot;
    // The swing (`ApplyItemAnimation` `:4449-4451`).
    app.swing_max = crate::item::use_animation_secs_of(held.kind, held.prefix);
    app.swing = app.swing_max;
    start_swing_extras(app, held.kind);
    app.attack_cd = app.swing_max;
    true
}


/// `Player.ItemCheck_Shoot` (`Player.cs:47056-49468`) — the non-ammo shot:
/// the aim normalized to `shootSpeed` (`:47277-47307`), the spawn at the player
/// centre + aim*10 (`RotatedRelativePoint(MountedCenter)`, `:47196`), the
/// per-item spawn arms (the gem staves, the Spirit Flame, the Sky Fracture, the
/// Nebula Blaze, the Bee Gun's multi-bee volley, the generic tail) and the
/// sentry/consumable tail. Runs on the use-start tick AND on every mid-swing
/// `ItemTimeIsZero` tick (the volley gate below): the C# fires the shot from the
/// `itemAnimation > 0 && ItemTimeIsZero` gate (`:43544-43570`), and
/// `ApplyItemTime` (`:47185-47194`) re-arms `itemTime = useTime` after EACH shot
/// — a `useTime < useAnimation` staff (the Aqua Scepter 8 < 16,
/// `Item.cs:3252-3268`) fires its whole volley inside one swing, not one shot per
/// swing (the i01 census item). The mana pay, the buff grant and
/// `FreeUpPetsAndMinions` ride the USE START (`ItemCheck_TryStartUse` /
/// `:42760-42780`), not the shot — they stay in the start block. Returns whether
/// the aim was non-zero (the port's `dir != ZERO` gate) so the caller keeps the
/// swing tail gated the same way. `p` is re-borrowed here (read-only) rather than
/// passed in: the chain touches only `app` fields disjoint from `app.player`
/// (projectiles/rngs/inventory/world/local_id), so the split borrow compiles.
/// `screen` rides the S1-P-i22 Magical Harp arm (the camera's smaller scaled
/// axis, `Player.cs:47395`); both call sites pass the use pass's screen.
fn fire_non_ammo_shot(
    app: &mut App,
    held: ItemSlot,
    held_fx: Option<crate::prefix::PrefixedStats>,
    world_pos: Vec2,
    proj: u16,
    def: &'static registry::ItemDef,
    screen: Vec2,
) -> bool {
    let Some(p) = app.player.as_ref() else {
        return false;
    };
    let (pcx, pcy) = p.center();
    let dir = (world_pos - Vec2::new(pcx, pcy)).normalize_or_zero();
    // S1-P-i08 — the useStyle-1 aiStyle-3 boomerang variant (the ported kinds
    // 6/52/113/106): the zero-aim case still fires in the C# (`:47291-47296`),
    // so it escapes the `dir != ZERO` gate (the facing fallback below).
    let boomerang_use = def.use_style == 1
        && crate::projectile::proj_data(proj).is_some_and(|d| d.ai_style == 3);
    if dir == Vec2::ZERO && !boomerang_use {
        return false;
    }
    let class = def.class;
    // The turret cap is snapshotted here — the whip arm below re-borrows the
    // player mutably (S1-P-i24's repeat-counter write), so the tail's sentry
    // sweep must not hold the shared read borrow across it.
    let max_turrets = p.max_turrets;
    // The i08 facing flip (`Player.cs:47202-47224`): `num = Dot(UnitX, aim)`;
    // `num > 0` → `ChangeDir(1)` else `ChangeDir(-1)` (the zero-aim case takes
    // the −1 arm too). The snapshot is read-only — the write rides the mutable
    // re-borrow at the boomerang arm below (the shared `p` borrow stays alive
    // for the whole chain).
    let boomerang_facing = if dir.x > 0.0 { 1 } else { -1 };
    let (fx_dmg, fx_spd, fx_kb) = held_fx
        .map_or((def.damage, def.shoot_speed, def.knock_back), |s| {
            (s.damage, s.shoot_speed, s.knock_back)
        });
    // S1-P-i25 — `Player.cs:47151` — `KnockBack = GetWeaponKnockback(sItem,
    // KnockBack)` runs ONCE here, after the ammo fuse / gunProj reset and
    // BEFORE every spawn arm: the summon `minionKB` add (`:53348-53351`), the
    // melee glove's ×2 (`:53352`), the Titan buff's ×1.5 (`:53356`) and the
    // shroomite-stealth ranged scale (`:53360`). Every arm below reads the
    // folded value — the gem arm's RepeatsGem `+1` (`:49303`) rides it, and
    // the whip (`summon = true`, `Item.cs`'s DefaultToWhip items) takes the
    // `minionKB` add too.
    let fx_kb = p.weapon_knockback(def, fx_kb);
    // `if (sItem.melee && !ProjectileID.Sets
    // .NoMeleeSpeedVelocityScaling[projToShoot]) speed /=
    // this.meleeSpeed` (`:47060-47063`). The exception set
    // (`ProjectileID.cs:547`: 699/707/877/878/879/985/973/1082/1083) — the
    // jousting lances 877-879 (S1-P-i06) are the ported members: their thrust
    // speed does NOT divide by meleeSpeed. Every other melee shot divides by
    // the INVERTED, held-item-scaled multiplier (`Player.cs:25912` +
    // `CapAttackSpeeds` `:29021-29039` — i12's held-kind form, the resync
    // keep-one over the gate's rng-less `melee_speed()`).
    let no_melee_speed_scaling =
        matches!(proj, 699 | 707 | 877 | 878 | 879 | 985 | 973 | 1082 | 1083);
    let fx_spd = if class == registry::DamageClass::Melee && !no_melee_speed_scaling {
        fx_spd / p.melee_speed(held.kind)
    } else {
        fx_spd
    };
    // `if (sItem.type == 2611 || sItem.type == 5526)
    // pointPosition += vector2` (`:47239-47245`): the 1-px
    // spawn shift along the aim (the Flairon's arm). The boomerang spawn is
    // the C#'s exact `pointPosition` — `RotatedRelativePoint(MountedCenter)`
    // (`:47196`) = the player center with no rotation/mount/sleep offsets;
    // the `+ dir * 10` the generic port arm carries is its own recorded
    // deviation (M4-PROJ-3), NOT the C# — the boomerang variant drops it.
    let mut spawn = if boomerang_use {
        Vec2::new(pcx, pcy)
    } else if held.kind == 2611 || held.kind == 5526 {
        Vec2::new(pcx, pcy) + dir * 10.0 + dir
    } else {
        Vec2::new(pcx, pcy) + dir * 10.0
    };
    // S1-P-i28 — `if (sItem.type == 3094 || sItem.type == 3378 ||
    // sItem.type == 3543) pointPosition.Y = position.Y + height / 3`
    // (`Player.cs:47225-47228`): the two javelins throw from the upper third
    // of the hitbox, not the centre. The integer `height / 3` truncates.
    if held.kind == 3094 || held.kind == 3378 || held.kind == 3543 {
        spawn.y = p.y + (p.height as i32 / 3) as f32;
    }
    let raw = fx_dmg as f32 * p.weapon_damage_multiplier(class);
    // The i28 consumable tail's flag reads snapshotted here — the shared
    // `p` read borrow must END before the mutable re-borrows below (the
    // boomerang facing flip, the Spider cycle write, the whip helper's
    // repeat-counter write). The four flags are per-tick equip grants;
    // nothing between here and the tail writes them.
    let (huntress90, chloro80, ammo80, ammo75) = (
        p.huntress_ammo_cost90,
        p.chloro_ammo_cost80,
        p.ammo_cost80,
        p.ammo_cost75,
    );
    // The crit does NOT roll at throw time (the resync spawn-crit conversion,
    // M1 canonical): the C# rolls it PER HIT inside `Damage_PVE_Inner`
    // (`Projectile.cs:12883-12917`) on the `UpdateProjectiles` stream; the
    // generic `ItemCheck_Shoot` tail (`Player.cs:49437`) carries NO roll. The
    // spawn carries the class flags (`proj_data`) + the `ApplyStatsFromSource`
    // stamps below (`:11128-11132`: the prefix arpen + the Revolver charge
    // bonus); the throw's `UpdatePlayers` stream spends no crit draw (the
    // Nebula Blaze pre-pass is the first draw of the generic shot). The
    // summon/minion/sentry rows carry no class flags — they never crit
    // (the i25/i26 census obligation, now structural).
    let loot_rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
    let arpen = held_fx.map_or(0, |s| s.armor_penetration);
    let revolver_bonus = if held.kind == 2269 {
        p.revolver_crit_chance_bonus
    } else {
        0
    };
    // `Player.cs:47119-47130` - the Nebula Blaze's pre-pass: a
    // 20% roll upgrades the bolt (634→635) at triple damage —
    // `Damage *= 3` on the weapon-damage int (`:42653`'s
    // `GetWeaponDamage` truncation, so the port triples the
    // truncated value) — and the miss decrements the shoot
    // speed BEFORE the aim vector is built.
    let mut proj = proj;
    let mut shot_speed = fx_spd;
    let mut blaze_damage = None;
    if held.kind == 3542 {
        if loot_rng.next_range(0, 100) < 20 {
            proj += 1;
            blaze_damage = Some((raw as i32) * 3);
        } else {
            shot_speed -= 1.0;
        }
    }
    // `Player.ItemCheck_Shoot`'s per-item spawn arms - the
    // magic staves replace the generic `Item.shoot` firing,
    // each `return`ing in the C#; the port folds them into
    // one if/else chain before the shared swing tail. The
    // base velocity is the aim normalized to `shootSpeed`.
    // S1-P-i08 — the zero-aim fallback (`:47291-47296`):
    // `num4 = direction, num5 = 0, num6 = speed` then the
    // `num4 *= num6` scale — the shot flies at the facing
    // × the shootSpeed (the NaN arm is unreachable through
    // `normalize_or_zero`).
    let shot = if dir == Vec2::ZERO && boomerang_use {
        Vec2::new(boomerang_facing as f32 * shot_speed, 0.0)
    } else {
        Vec2::new(dir.x * shot_speed, dir.y * shot_speed)
    };
    // S1-P-i08 — the useStyle-1 boomerang throw (`Player.cs:49437`'s generic
    // `NewProjectile` tail): the exact-center spawn, the aim (or the facing
    // fallback) normalized to `shootSpeed / meleeSpeed`, `weaponDamage`,
    // `GetWeaponKnockback`'s melee arms (already folded into fx_kb above),
    // ai0/ai1/ai2 = 0. No crit draw at throw time — the crit rolls per-hit
    // in `damage_pve` (`Projectile.cs:12883-12917`, S1-P-i11). The facing
    // flip (`ChangeDir` `:47202-47224`) writes through a short mutable
    // re-borrow (the shared read borrow is parked for the arm).
    if boomerang_use {
        if let Some(pl) = app.player.as_mut() {
            pl.facing = boomerang_facing;
        }
        // `NewProjectile` centers the box on the point (`Projectile.cs:
        // 10876-10877`: `position = X − width/2, Y − height/2`); the port's
        // spawn seam takes the top-left, so the half-box subtracts here —
        // the disc's CENTER lands on the player centre exactly.
        let (bw, bh) = crate::projectile::proj_data(proj)
            .map_or((0.0, 0.0), |d| (d.w, d.h));
        let slot = app.projectiles.spawn_with_data_ai(
            proj,
            spawn.x - bw * 0.5,
            spawn.y - bh * 0.5,
            shot.x,
            shot.y,
            raw as i32,
            fx_kb,
            0.0,
            0.0,
        );
        // Gate i08×i11: the throw-time crit stamp is gone — the per-hit
        // roll reads the melee class flag off `proj_data`.
        if let Some(slot) = slot {
            app.projectiles.set_armor_penetration(slot, arpen);
            app.projectiles.set_bonus_crit_chance(slot, revolver_bonus);
        }
    // S1-P-i25 — the summon-staff minion spawn arm
    // (`Player.ItemCheck_Shoot`'s per-item `SpawnMinionOnCursor`
    // arms, `Player.cs:48067-48085`/`:48436-48496`/`:48556-48559`).
    // A summon item whose shoot is a MINION (not a sentry, not a
    // whip) spawns at the CURSOR via `SpawnMinionOnCursor`
    // (`:49573-49603`) — NOT the generic player-centre flight:
    // the point is `Main.MouseWorld` clamped by
    // `LimitPointToPlayerReachableArea`, the velocity is ZERO
    // (the Optic Staff's `RotatedBy` of a zero vector stays zero),
    // the damage is the UNSCALED `sItem.damage` (`:47060`'s local
    // `damage`, not the class-multiplied `Damage` — the pool's
    // `summon_tick_block` applies `minionDamage` at `:16707`), and
    // the knockback takes `GetWeaponKnockback`'s `minionKB` bonus
    // (`:47151`/`:53348-53351`). The `ai0/ai1` seeds ride
    // `SpawnMinionOnCursor`'s item switch (`:49582-49598`).
    // Gate-merge (i25 round 2): the Clay Pot (6161) has NO
    // `SpawnMinionOnCursor` arm in the C# — it rides the generic tail
    // (`:49437`) with the SCALED damage + the `originalDamage` override
    // (`:49442-49444`), so it is excluded here even though its shoot
    // (1118) is a minion row.
    } else if def.summon
        && held.kind != 6161
        && crate::projectile::proj_data(proj).is_some_and(|d| d.minion)
    {
        // `SpawnMinionOnCursor`'s cursor point + reach clamp
        // (`:49575-49577`).
        let cursor =
            limit_point_to_player_reachable_area(Vec2::new(pcx, pcy), world_pos, &app.world);
        // The per-item kind + ai seeds. `damage` is the prefixed
        // UNSCALED item damage (`fx_dmg`), the C# local `damage`.
        let unscaled = fx_dmg;
        let owner = app.local_id as u8;
        // `NewProjectile` takes the CENTRE and subtracts the
        // half-box (`Projectile.cs:10876-10877`); the port's
        // spawn seam takes the top-left, so the cursor point is
        // re-centred per spawned kind (the inferno precedent,
        // `projectile.rs`'s blast spawn). `fx_kb` already carries
        // `GetWeaponKnockback`'s fold (`:47151`, above).
        let spawn_minion = |projectiles: &mut crate::projectile::ProjectilePool,
                            kind: u16,
                            ai0: f32,
                            ai1: f32|
         -> Option<usize> {
            let row = crate::projectile::proj_data(kind)?;
            projectiles.spawn_with_data_ai_owned(
                kind,
                owner,
                cursor.x - row.w * 0.5,
                cursor.y - row.h * 0.5,
                0.0,
                0.0,
                unscaled,
                fx_kb,
                ai0,
                ai1,
            )
        };
        if held.kind == 3531 {
            // The Stardust Dragon Staff's chain spawn
            // (`:48498-48554`) stays deferred — the head/segment
            // spawn choreography (the growing chain, the tail
            // relink) is the use-path follow-up. The chain AI
            // itself landed at the gate (`dragon_step`, the 121
            // arm — the i56 coverage pin demanded it); the 625-628
            // rows book slots and fly. The buff grant + free-up
            // above already ran.
        } else if held.kind == 1157 {
            // `:48436-48440` — the Pygmy Staff: one
            // `Main.rand.Next(191, 195)` kind pick, ai1 = 60
            // (`:49584-49585`), then `localAI[0] = 30`.
            let kind = loot_rng.next_range(191, 195) as u16;
            if let Some(slot) = spawn_minion(&mut app.projectiles, kind, 0.0, 60.0) {
                app.projectiles.set_local_ai0(slot, 30.0);
            }
        } else if held.kind == 2535 {
            // `:48468-48474` — the Optic Staff spawns BOTH twins
            // (387 + 388) at the cursor, zero velocity (the
            // `RotatedBy` of `Vector2.Zero`), ai1 = 1 (`:49589`).
            for k in [proj, proj + 1] {
                let _ = spawn_minion(&mut app.projectiles, k, 0.0, 1.0);
            }
        } else if held.kind == 2551 {
            // `:48476-48481` — the Spider Staff cycles
            // 390/391/392 via `nextCycledSpiderMinionType` (never
            // reset, `Player.cs:1280`), then increments mod 3.
            let cycle = p.next_cycled_spider_minion_type;
            let kind = (proj as i32 + cycle) as u16;
            let _ = spawn_minion(&mut app.projectiles, kind, 0.0, 0.0);
            let p_mut = app.player.as_mut().unwrap();
            p_mut.next_cycled_spider_minion_type += 1;
            p_mut.next_cycled_spider_minion_type %= 3;
        } else if held.kind == 2584 {
            // `:48483-48486` — the Pirate Staff: one
            // `Main.rand.Next(3)` offset onto 393/394/395.
            let kind = proj + loot_rng.next_range(0, 3) as u16;
            let _ = spawn_minion(&mut app.projectiles, kind, 0.0, 0.0);
        } else if held.kind == 5664 || held.kind == 6149 {
            // `:48452-48456` — the Foxsparks staves spawn then
            // set `localAI[0] = 30`.
            if let Some(slot) = spawn_minion(&mut app.projectiles, proj, 0.0, 0.0) {
                app.projectiles.set_local_ai0(slot, 30.0);
            }
        } else {
            // The plain `SpawnMinionOnCursor` staves. The ai1
            // seed is 1 for 2364/2365/2535/2621/2749/3474
            // (`:49587-49593`), the ai0 seed 45 for 6164
            // (`:49595-49596`); every other staff keeps 0/0.
            let (ai0, ai1) = match held.kind {
                2364 | 2365 | 2535 | 2621 | 2749 | 3474 => (0.0, 1.0),
                6164 => (45.0, 0.0),
                _ => (0.0, 0.0),
            };
            let _ = spawn_minion(&mut app.projectiles, proj, ai0, ai1);
        }
    } else if matches!(held.kind, 1572 | 2366 | 3571 | 3569 | 5119 | 5463) {
        // S1-P-i26 (supersedes the i25 round-2 sentry arm at the gate) вЂ”
        // `Player.cs:48189-48212`: the placeable staff sentries (Frost Hydra
        // 1572в†’308, Queen Spider 2366в†’377, Rainbow Crystal 3571в†’643, Moon
        // Lord Turret 3569в†’641, Houndius Shootius 5119в†’966, Dead Cells
        // Barnacle 5463в†’1025). `PickSentryLandingSpot` picks the cursor/tile
        // spot, the spawn drops at `(0, 15)` (the barnacle at `(0, 0)` on the
        // raw cursor Y), the per-item `ai0` (60/90 for the 1572/5119), then
        // `originalDamage = sItem.damage` (the UNSCALED item damage,
        // `:47060`/`:48209`) and `UpdateMaxTurrets`. No `Main.rand` draw.
        let (spawn_x, mut spawn_y) =
            sentry::pick_sentry_landing_spot(&app.world, held.kind, world_pos);
        let mut speed_y = 15.0f32;
        let ai0 = match held.kind {
            1572 => 60.0,
            5119 => 90.0,
            5463 => {
                spawn_y = world_pos.y;
                speed_y = 0.0;
                0.0
            }
            _ => 0.0,
        };
        // `NewProjectile` centres the box (`Projectile.cs:10876-10877`); the
        // port's spawn seam takes the top-left, so subtract half the row's size.
        let data = crate::projectile::proj_data(proj).unwrap();
        // `GetWeaponDamage` (`Player.cs:53421-53429`): the class-scaled damage
        // truncates `(float)damage * multiplier + 5E-06f` вЂ” the epsilon rides
        // the cast (the ammo path's `weapon_damage` precedent).
        let sentry_damage = (raw + 5e-6) as i32;
        let slot = app.projectiles.spawn_with_data_ai_owned(
            proj,
            app.local_id as u8,
            spawn_x - data.w * 0.5,
            spawn_y - data.h * 0.5,
            0.0,
            speed_y,
            sentry_damage,
            fx_kb,
            ai0,
            0.0,
        );
        if let Some(slot) = slot {
            // `:48209` вЂ” `originalDamage = damage` where `damage =
            // sItem.damage` (`:47060`), the prefixed item damage BEFORE the
            // class multiplier.
            app.projectiles.set_original_damage(slot, fx_dmg);
        }
    } else if tr_core::dd2::is_dd2_turret(proj) {
        // S1-P-i26 вЂ” `Player.cs:49057-49094`: the DD2 tower poppers.
        // `PayDD2CrystalsBeforeUse` consumes the 10 crystals (the use gates
        // already ran at the block head), `FindSentryRestingSpot` picks the
        // ground spot, the per-tier `ai0/ai1/localAI[0]` payload rides the
        // spawn, `originalDamage = sItem.damage`, then `UpdateMaxTurrets`.
        // No `Main.rand` draw in this arm.
        sentry::pay_dd2_crystals(&mut app.inventory, &mut app.equip, held.kind);
        let (cx, cy) = p.center();
        let facing = p.facing;
        let (wx, wy, push) = sentry::find_sentry_resting_spot(
            &app.world,
            proj,
            world_pos,
            Vec2::new(cx, cy),
            facing,
        );
        let mut ai0 = 0.0f32;
        let mut ai1 = 0.0f32;
        let mut local0 = 0.0f32;
        match held.kind {
            3824 | 3825 | 3826 => {
                ai1 = sentry::ballistra_shot_delay(p.set_squire_t3, p.ballista_panic) as f32;
            }
            3832 | 3833 | 3834 => {
                local0 =
                    sentry::explosive_trap_cooldown(p.set_huntress_t2, p.set_huntress_t3) as f32;
            }
            3818 => {
                ai0 = 1.0;
                ai1 = 80.0;
            }
            3819 => {
                ai0 = 1.0;
                ai1 = 70.0;
            }
            3820 => {
                ai0 = 1.0;
                ai1 = 60.0;
            }
            _ => {}
        }
        let data = crate::projectile::proj_data(proj).unwrap();
        // `GetWeaponDamage` (`Player.cs:53421-53429`): the epsilon rides the cast.
        let sentry_damage = (raw + 5e-6) as i32;
        let slot = app.projectiles.spawn_with_data_ai_owned(
            proj,
            app.local_id as u8,
            wx as f32 - data.w * 0.5,
            (wy - push) as f32 - data.h * 0.5,
            0.0,
            0.0,
            sentry_damage,
            fx_kb,
            ai0,
            ai1,
        );
        if let Some(slot) = slot {
            // `:49090-49091` вЂ” the unscaled item damage and the trap's
            // `localAI[0]` cooldown.
            app.projectiles.set_original_damage(slot, fx_dmg);
            app.projectiles.set_local_ai0(slot, local0);
        }
    } else if tr_core::golf_state::is_a_golf_ball(proj as i32) {
        // S1-P-i55 — `ItemCheck_Shoot`'s golf-ball arm
        // (`Player.cs:47516-47537`): within 100 px of the player the use
        // tees the ball on a nearby tee
        // (`TryPlacingAGolfBallNearANearbyTee(MouseWorld)`); failing that, a
        // far or blocked aim spawns at the hand point with the aim velocity,
        // a near clear aim spawns AT the mouse with zero velocity. The arm
        // `return`s in the C# — the port skips the generic spawn but keeps
        // the shared tail below. (Transplanted from main at the resync.)
        let to_mouse = world_pos - Vec2::new(pcx, pcy);
        let mut placed = false;
        if to_mouse.length() < 100.0 {
            // The disjoint-parts resolver — the seam's `p` read borrow is
            // parked for the call (the parts take only what they need).
            let (px, py, ph, trx, try_, equipped) = {
                let pl = app.player.as_ref().unwrap();
                (pl.x, pl.y, pl.height, pl.tile_range_x(), pl.tile_range_y(), pl.equipped)
            };
            placed = crate::projectile_ai_golf::try_place_golf_ball_near_tee_parts(
                &mut app.projectiles,
                &app.world,
                world_pos,
                world_pos,
                Vec2::new(px, py),
                Vec2::new(crate::player::WIDTH, ph),
                trx,
                try_,
                app.selected,
                &app.inventory,
                &equipped,
                app.local_id as u8,
            );
        }
        if !placed {
            let damage = raw as i32;
            let slot = if to_mouse.length() > 100.0
                || !crate::projectile::can_hit(
                    &app.world, pcx, pcy, 1.0, 1.0,
                    world_pos.x, world_pos.y, 1.0, 1.0,
                )
            {
                // `:47529` — the hand-point spawn with the aim velocity
                // (the C# takes the centre, the port's seam takes the
                // top-left).
                let row = crate::projectile::proj_data(proj);
                let (hw, hh) = row.map_or((3.5, 3.5), |d| (d.w * 0.5, d.h * 0.5));
                app.projectiles.spawn_with_data_ai_owned(
                    proj,
                    app.local_id as u8,
                    spawn.x - hw,
                    spawn.y - hh,
                    shot.x,
                    shot.y,
                    damage,
                    fx_kb,
                    0.0,
                    0.0,
                )
            } else {
                // `:47533` — the mouse-point spawn, zero velocity.
                let row = crate::projectile::proj_data(proj);
                let (hw, hh) = row.map_or((3.5, 3.5), |d| (d.w * 0.5, d.h * 0.5));
                app.projectiles.spawn_with_data_ai_owned(
                    proj,
                    app.local_id as u8,
                    world_pos.x - hw,
                    world_pos.y - hh,
                    0.0,
                    0.0,
                    damage,
                    fx_kb,
                    0.0,
                    0.0,
                )
            };
            if let Some(slot) = slot {
                // M3 sync: no throw-time crit stamp — the golf ball crits
                // per-hit in damage_pve (Projectile.cs:12883-12917).
                app.projectiles.set_armor_penetration(slot, arpen);
                app.projectiles.set_bonus_crit_chance(slot, revolver_bonus);
                // `NewProjectile`'s >10-ball cull (`Projectile.cs:10932-
                // 10955`): the use spawns are the local owner's, so the
                // `Owner == Main.myPlayer` gate holds.
                crate::projectile_ai_golf::cull_extra_golf_balls(
                    &mut app.projectiles,
                    app.local_id as u8,
                    proj,
                    damage,
                );
            }
        }
    } else if GEM_STAFF_BOLTS.contains(&proj) {
        // `Player.cs:49262-49375` - the gem-staff shoot.
        // `PackGemStaffFeatures` (`:49470-49489`): the bits
        // from the bolt kind and the matching body robe
        // (`GetEffectiveArmor(1)` - the port's `equipped[1]`,
        // the robe slot 1282/1285/4256/1284/1283/1286/1287).
        let mut features = GemStaffFeatures::from_ai(0.0);
        let robe = p.equipped[1];
        features.set_bit(GemStaffFeatures::HOMING, proj == 121 || robe == 1282);
        features.set_bit(GemStaffFeatures::FAST_THEN_SLOW, proj == 124 || robe == 1285);
        features.set_bit(GemStaffFeatures::CAN_BOUNCE, proj == 597 || robe == 4256);
        features.set_bit(GemStaffFeatures::SWIRL_TWINS, proj == 123 || robe == 1284);
        features.set_bit(GemStaffFeatures::AOE_EXPLOSION, proj == 122 || robe == 1283);
        features.set_bit(GemStaffFeatures::AR_PEN_SPREAD, proj == 125 || robe == 1286);
        features.set_bit(GemStaffFeatures::BIGGER_HITBOX, proj == 126 || robe == 1287);
        features.set_bit(GemStaffFeatures::ALL_GEMS, held.kind == 6171);
        if !features.all_gems() {
            features.set_bit(
                GemStaffFeatures::REPEATS_GEM,
                (proj == 121 && robe == 1282)
                    || (proj == 124 && robe == 1285)
                    || (proj == 597 && robe == 4256)
                    || (proj == 123 && robe == 1284)
                    || (proj == 122 && robe == 1283)
                    || (proj == 125 && robe == 1286)
                    || (proj == 126 && robe == 1287),
            );
        }
        if features.all_gems() {
            // `:49266-49273`: the gembag head forces the four.
            features.set_bit(GemStaffFeatures::HOMING, true);
            features.set_bit(GemStaffFeatures::CAN_BOUNCE, true);
            features.set_bit(GemStaffFeatures::AOE_EXPLOSION, true);
            features.set_bit(GemStaffFeatures::FAST_THEN_SLOW, true);
        }
        let features = features;
        // `:49286-49295`: the velocity multipliers.
        let (mut vx, mut vy) = (shot.x, shot.y);
        if features.fast_then_slow() {
            vx *= 5.0;
            vy *= 5.0;
        }
        if features.can_bounce() {
            vx *= 1.25;
            vy *= 1.25;
        }
        let (mut sx, mut sy) = (spawn.x, spawn.y);
        if features.ar_pen_spread() {
            // `Main.rand.NextVector2CircularEdge(12, 12)`
            // (`Utils.cs:1271-1274`) - a unit vector scaled by
            // the half-widths.
            let angle = loot_rng.next_float() * std::f32::consts::TAU;
            let off = Vec2::from_angle(angle) * 12.0;
            sx += off.x;
            sy += off.y;
        }
        // `:49296-49304`: the damage multiplier and the
        // RepeatsGem bonus.
        let mut damage = (raw * features.damage_multiplier()) as i32;
        let mut kb = fx_kb;
        if features.repeats_gem() {
            damage += 4;
            kb += 1.0;
        }
        let bits = features.ai_state();
        let spin = p.facing == -1;
        if features.all_gems2() {
            // `:49307-49349`: seven bolts, the per-index kind
            // (the loop's initial `NextFromList` pick is
            // overwritten by the switch, so it is skipped).
            for n in 1..=7i32 {
                let slot_index = if spin { 8 - n } else { n };
                let kind = match slot_index {
                    1 => 121,
                    2 => 122,
                    3 => 123,
                    4 => 126,
                    5 => 124,
                    6 => 597,
                    _ => 125,
                };
                let offset = (n - 4) as f32;
                let mut v = Vec2::new(vx, vy).rotate(Vec2::from_angle(
                    offset * (std::f32::consts::PI / 4.0) / 10.0,
                ));
                v *= remap_gem(offset.abs(), 0.0, 3.0, 1.3, 1.0);
                let slot = app.projectiles.spawn_with_data_ai(
                    kind, sx, sy, v.x, v.y, damage, kb, bits, n as f32,
                );
                if let Some(slot) = slot {
                    app.projectiles.set_armor_penetration(slot, arpen);
                    app.projectiles.set_bonus_crit_chance(slot, revolver_bonus);
                }
            }
        } else if features.all_gems() {
            // `:49350-49364`: three bolts of one random gem
            // kind (the `NextFromList` pick, `:49354`).
            let pick = loot_rng.next_range(0, 7);
            let kind = [121u16, 122, 123, 124, 597, 125, 126][pick as usize];
            for n in 1..=3i32 {
                let offset = (n - 2) as f32;
                let mut v = Vec2::new(vx, vy).rotate(Vec2::from_angle(
                    offset * (std::f32::consts::PI / 4.0) / 10.0,
                ));
                v *= remap_gem(offset.abs(), 0.0, 1.0, 1.1, 1.0);
                let slot = app.projectiles.spawn_with_data_ai(
                    kind, sx, sy, v.x, v.y, damage, kb, bits, n as f32,
                );
                if let Some(slot) = slot {
                    app.projectiles.set_armor_penetration(slot, arpen);
                    app.projectiles.set_bonus_crit_chance(slot, revolver_bonus);
                }
            }
        } else {
            // `:49365-49372`: the single bolt, the SwirlTwins
            // twin with `ai1 = 1f`.
            let slot = app.projectiles.spawn_with_data_ai(
                proj, sx, sy, vx, vy, damage, kb, bits, 0.0,
            );
            if let Some(slot) = slot {
                // The resync spawn-crit conversion: no throw-time stamp —
                // the per-hit roll reads the class flags; the
                // ApplyStatsFromSource stamps ride every spawn (:11128-11132).
                app.projectiles.set_armor_penetration(slot, arpen);
                app.projectiles.set_bonus_crit_chance(slot, revolver_bonus);
            }
            if features.swirl_twins() {
                let slot = app.projectiles.spawn_with_data_ai(
                    proj, sx, sy, vx, vy, damage, kb, bits, 1.0,
                );
                if let Some(slot) = slot {
                    app.projectiles.set_armor_penetration(slot, arpen);
                    app.projectiles.set_bonus_crit_chance(slot, revolver_bonus);
                }
            }
        }
    } else if held.kind == 3779 {
        // `Player.cs:48793-48807` - the Spirit Flame's spawn:
        // up to 10 aim retries against the line of sight, the
        // bolt at the player centre + direction * 30 with
        // zero velocity, `ai0 = -2`.
        let mut angle = loot_rng.next_float() * std::f32::consts::TAU;
        for _ in 0..10 {
            let probe = spawn + shot.rotate(Vec2::from_angle(angle)) * 100.0;
            // S1-P-i22: `:48798` — `Collision.CanHit` (the Manhattan
            // walk, NOT CanHitLine).
            if crate::projectile::can_hit(
                &app.world, spawn.x, spawn.y, 0.0, 0.0, probe.x, probe.y, 0.0, 0.0,
            ) {
                break;
            }
            angle = loot_rng.next_float() * std::f32::consts::TAU;
        }
        let speed = 0.95 + loot_rng.next_float() * 0.3;
        let v = shot.rotate(Vec2::from_angle(angle)) * speed;
        let slot = app.projectiles.spawn_with_data_ai(
            proj,
            spawn.x + v.x * 30.0,
            spawn.y + v.y * 30.0,
            0.0,
            0.0,
            raw as i32,
            fx_kb,
            -2.0,
            0.0,
        );
        if let Some(slot) = slot {
            app.projectiles.set_armor_penetration(slot, arpen);
            app.projectiles.set_bonus_crit_chance(slot, revolver_bonus);
        }
    } else if held.kind == 3787 {
        // `Player.cs:48808-48827` - the Sky Fracture's spawn:
        // a ring point 20-60 px out (up to 50 retries with an
        // 8 px line probe), the velocity lerped 25% toward the
        // cursor direction. S1-P-i22: the C# draws the angle AND a radial
        // ONCE before the loop (`:48810` + `:48813` — the
        // pre-loop ring value is overwritten by the loop's
        // first iteration, but its radial DRAW is spent), then
        // per iteration: radial draw → probe → angle re-roll.
        let mut face = loot_rng.next_float() * std::f32::consts::TAU;
        // `:48813` — the pre-loop radial draw (result discarded).
        let _pre = 20.0 + (60.0 - 20.0) * loot_rng.next_float();
        let mut ring = spawn;
        for _ in 0..50 {
            let radius = 20.0 + (60.0 - 20.0) * loot_rng.next_float();
            ring = spawn + Vec2::from_angle(face) * radius;
            // `:48817` — `SafeNormalize(Vector2.UnitX)`: the
            // zero-delta fallback is +X, not zero.
            let outward = (ring - spawn).normalize_or(Vec2::X);
            let probe = ring + outward * 8.0;
            // `:48817` — `Collision.CanHit` (NOT CanHitLine).
            if crate::projectile::can_hit(
                &app.world, spawn.x, spawn.y, 0.0, 0.0, probe.x, probe.y, 0.0, 0.0,
            ) {
                break;
            }
            face = loot_rng.next_float() * std::f32::consts::TAU;
        }
        // `:48823-48825` — `vector61 = (num4,num5).SafeNormalize
        // (UnitY) * speed`; `Lerp(v4.SafeNormalize(vector61) *
        // speed, vector61, 0.25)`. The along fallback is UnitY
        // and the toward fallback is vector61 itself.
        let along = shot.normalize_or(Vec2::Y) * fx_spd;
        let toward = (world_pos - ring).normalize_or(along) * fx_spd;
        let v = toward + (along - toward) * 0.25;
        let slot = app.projectiles.spawn_with_data_ai(
            proj, ring.x, ring.y, v.x, v.y, raw as i32, fx_kb, 0.0, 0.0,
        );
        if let Some(slot) = slot {
            app.projectiles.set_armor_penetration(slot, arpen);
            app.projectiles.set_bonus_crit_chance(slot, revolver_bonus);
        }
    } else if held.kind == 3542 {
        // `Player.cs:48778-48792` - the Nebula Blaze's spawn:
        // the aim angle within ±(π/4 · 0.7), up to 10 retries
        // against the line of sight (the angle draw precedes
        // the loop, each failed probe re-rolls), then the
        // velocity at one multiplier draw `0.95 + rand·0.3`
        // AFTER the loop.
        let mut angle = (loot_rng.next_float() - 0.5)
            * (std::f32::consts::PI / 4.0)
            * 0.7;
        for _ in 0..10 {
            let probe = spawn + shot.rotate(Vec2::from_angle(angle)) * 100.0;
            if crate::projectile::can_hit_line(
                &app.world, spawn.x, spawn.y, 0.0, 0.0, probe.x, probe.y, 0.0, 0.0,
            ) {
                break;
            }
            angle = (loot_rng.next_float() - 0.5)
                * (std::f32::consts::PI / 4.0)
                * 0.7;
        }
        let v = shot.rotate(Vec2::from_angle(angle))
            * (0.95 + loot_rng.next_float() * 0.3);
        let damage = blaze_damage.unwrap_or(raw as i32);
        let slot = app.projectiles.spawn_with_data_ai(
            proj, spawn.x, spawn.y, v.x, v.y, damage, fx_kb, 0.0, 0.0,
        );
        if let Some(slot) = slot {
            app.projectiles.set_armor_penetration(slot, arpen);
            app.projectiles.set_bonus_crit_chance(slot, revolver_bonus);
        }
    } else if held.kind == 494 {
        // S1-P-i22 — `Player.cs:47392-47423` - the Magical Harp's note:
        // the kind rolls 76 + Next(3), the velocity scales by
        // the cursor distance over the camera's smaller scaled
        // axis, and the note's ai0 is the clamped pitch.
        let note = proj + loot_rng.next_range(0, 3) as u16;
        // `:47395-47399` - num7 = cursor distance / (axis/2),
        // clamped to 1. The distance is the PRE-scale aim length
        // (`num7` at `:47290`, before the speed normalization).
        let mut dist = (world_pos - spawn).length() / (smaller_scaled_axis(screen, app.zoom) / 2.0);
        if dist > 1.0 {
            dist = 1.0;
        }
        // `:47400-47404` - the spread + the distance scale.
        // The C# computes num10/num11 here but the spawn at
        // `:47410` uses vector9 (not them) — the two spread
        // DRAWS are still spent, the values discarded.
        let mut nx = shot.x + loot_rng.next_range(-40, 41) as f32 * 0.01;
        let mut ny = shot.y + loot_rng.next_range(-40, 41) as f32 * 0.01;
        dist *= 1.75;
        nx *= dist + 0.1;
        ny *= dist + 0.1;
        let _ = (nx, ny);
        // `:47405-47409` - the target offset: MouseWorld -
        // Center + NextVector2Circular(40,40), remapped to the
        // 20-60 speed divisor.
        let (pcx, pcy) = p.center();
        let mut off = world_pos - Vec2::new(pcx, pcy);
        off += next_vector2_circular(loot_rng, 40.0, 40.0);
        let divisor = remap_clamped(off.length(), 540.0, 960.0, 20.0, 60.0);
        let v9 = off / divisor * 2.0;
        // `:47410` - ai2 = vector9.Length() / num12.
        let ai2 = v9.length() / divisor;
        // The notes' ×0.95 (`Projectile.cs:13432-13436`) is a
        // HIT-time fold — out of the use-path scope (census).
        let slot = app.projectiles.spawn_with_data_ai(
            note, spawn.x, spawn.y, v9.x, v9.y, raw as i32, fx_kb, 0.0, 1.0,
        );
        if let Some(slot) = slot {
            app.projectiles.set_armor_penetration(slot, arpen);
            app.projectiles.set_bonus_crit_chance(slot, revolver_bonus);
            app.projectiles.set_ai2(slot, ai2);
            // `:47411-47420` - num7 = clamp(num7*2-1, -1, 1)
            // overrides ai0 (the pitch the note's AI reads).
            let pitch = (dist * 2.0 - 1.0).clamp(-1.0, 1.0);
            app.projectiles.set_ai0(slot, pitch);
        }
    } else if held.kind == 4715 {
        // S1-P-i22 — `Player.cs:47994-48025` - the Sparkle Guitar's note:
        // a chaseable target (or the cursor), the arc's control
        // point in ai0/ai1, the launch velocity an upward edge
        // vector scaled by twice the control distance.
        let (pcx, pcy) = p.center();
        let center = Vec2::new(pcx, pcy);
        let targets = sparkle_guitar_targets(center, &app.npcs);
        let mut target = world_pos;
        let has_target = !targets.is_empty();
        if has_target {
            let pick = targets[loot_rng.next_range(0, targets.len() as i32) as usize];
            let n = &app.npcs[pick];
            let (ncx, ncy) = n.center();
            target = Vec2::new(ncx + n.vx * 20.0, ncy + n.vy * 20.0);
        }
        let mut ctrl = target - center;
        if !has_target {
            target += next_vector2_circular(loot_rng, 24.0, 24.0);
            if ctrl.length() > 700.0 {
                ctrl *= 700.0 / ctrl.length();
                target = center + ctrl;
            }
        }
        // `:48013-48022` - the edge vector, forced upward.
        let mut edge = next_vector2_circular_edge(loot_rng, 1.0, 1.0);
        if edge.y > 0.0 {
            edge *= -1.0;
        }
        if edge.y.abs() < 0.5 {
            edge.y = (0.0 - loot_rng.next_float()) * 0.5 - 0.5;
        }
        edge *= ctrl.length() * 2.0;
        let slot = app.projectiles.spawn_with_data_ai(
            proj, spawn.x, spawn.y, edge.x, edge.y, raw as i32, fx_kb,
            target.x, target.y,
        );
        if let Some(slot) = slot {
            app.projectiles.set_armor_penetration(slot, arpen);
            app.projectiles.set_bonus_crit_chance(slot, revolver_bonus);
        }
    } else if held.kind == 1308 {
        // S1-P-i22 — `Player.cs:48115-48134` - the Poison Staff's volley:
        // 3 fangs, 4 on a Next(3)==0 roll; each spreads by
        // 0.05·index and re-normalizes to shootSpeed. The
        // index-0 fang still spends its two draws (×0).
        let mut count = 3;
        if loot_rng.next_range(0, 3) == 0 {
            count += 1;
        }
        for i in 0..count {
            let spread = 0.05 * i as f32;
            let fx = shot.x + loot_rng.next_range(-35, 36) as f32 * spread;
            let fy = shot.y + loot_rng.next_range(-35, 36) as f32 * spread;
            let len = (fx * fx + fy * fy).sqrt();
            let scale = if len > 0.0 { fx_spd / len } else { 0.0 };
            // The fang's ×0.75 (`Projectile.cs:13414-13416`) is
            // a HIT-time fold in `Damage_PVE_Inner` (it compounds
            // per hit) — out of the use-path scope, recorded in
            // the census; the spawn carries the raw damage.
            let slot = app.projectiles.spawn_with_data_ai(
                proj, spawn.x, spawn.y, fx * scale, fy * scale,
                raw as i32, fx_kb, 0.0, 0.0,
            );
            if let Some(slot) = slot {
                // The resync spawn-crit conversion: no throw-time stamp —
                // the per-hit roll reads the class flags; the
                // ApplyStatsFromSource stamps ride every spawn (:11128-11132).
                app.projectiles.set_armor_penetration(slot, arpen);
                app.projectiles.set_bonus_crit_chance(slot, revolver_bonus);
            }
        }
    } else if held.kind == 65 {
        // S1-P-i22 — `Player.cs:47251-47266` (the projToShoot==9 block) +
        // `:49095-49108` (the type-65 arm) - the Starfury star:
        // the spawn point rides the cursor X with a random
        // lateral offset, 600 px above the mounted centre; the
        // damage ×1.5, knockback 0; ai1 is the SolidTile-walked
        // cursor Y the star falls to.
        let (_pcx, pcy) = p.center();
        let mouse_x = world_pos.x;
        let left = p.x;
        let right = p.x + crate::player::WIDTH;
        let mut side = -1.0f32;
        if mouse_x < left {
            side = 1.0;
        } else if mouse_x <= right && loot_rng.next_range(0, 2) == 0 {
            side = 1.0;
        }
        let star_x = p.x + crate::player::WIDTH * 0.5
            + loot_rng.next_range(0, 201) as f32 * side
            + (mouse_x - p.x);
        let star_spawn = Vec2::new(star_x, pcy - 600.0);
        // `:47265` — the use-time ×1.5 (the int truncation).
        // The star's hit-time ×0.75 (`Projectile.cs:14502-14504`)
        // is out of the use-path scope (census).
        let star_damage = (raw * 1.5) as i32;
        // `:49099-49105` - vec walks from MouseWorld toward the
        // spawn while its tile is solid and it sits below the
        // spawn point.
        let step = (star_spawn - world_pos).normalize_or(Vec2::new(0.0, -1.0));
        let mut vec = world_pos;
        while vec.y > star_spawn.y
            && crate::projectile::solid_tile(
                &app.world,
                (vec.x / TILE_PX) as i32,
                (vec.y / TILE_PX) as i32,
            )
        {
            vec += step * 16.0;
        }
        let slot = app.projectiles.spawn_with_data_ai(
            proj, star_spawn.x, star_spawn.y, shot.x, shot.y,
            star_damage, 0.0, 0.0, vec.y,
        );
        if let Some(slot) = slot {
            app.projectiles.set_armor_penetration(slot, arpen);
            app.projectiles.set_bonus_crit_chance(slot, revolver_bonus);
        }
    } else if held.kind == 1121 {
        // `Player.cs:48309-48332` — the Bee Gun's multi-bee
        // spawn. The count: `Next(1, 4)` base, +1 per
        // `Next(6) == 0` (two rolls), +1 more only while
        // `strongBees` and a third `Next(3) == 0` (the Hive
        // Pack). Each bee: the aim velocity jittered by
        // `Next(-35, 36) * 0.02` per axis, the kind/damage/
        // knockback off `beeType`/`beeDamage`/`beeKB`
        // (`Player.cs:12417-12444`), and `magic = true`.
        // The named-argument evaluation order is SpeedX,
        // SpeedY, Type, Damage, KnockBack — the two velocity
        // draws precede the bee-type coin flip.
        let mut count = loot_rng.next_range(1, 4);
        if loot_rng.next_range(0, 6) == 0 {
            count += 1;
        }
        if loot_rng.next_range(0, 6) == 0 {
            count += 1;
        }
        if p.strong_bees && loot_rng.next_range(0, 3) == 0 {
            count += 1;
        }
        for _ in 0..count {
            let vx = shot.x + loot_rng.next_range(-35, 36) as f32 * 0.02;
            let vy = shot.y + loot_rng.next_range(-35, 36) as f32 * 0.02;
            let (bee_kind, strong) = crate::hurt_modifiers::bee_type(p, loot_rng);
            let bee_dmg = crate::hurt_modifiers::bee_damage(strong, raw as i32, loot_rng);
            let bee_kb = crate::hurt_modifiers::bee_kb(strong, fx_kb);
            let slot = app.projectiles.spawn_with_data_ai(
                bee_kind, spawn.x, spawn.y, vx, vy, bee_dmg, bee_kb, 0.0, 0.0,
            );
            if let Some(slot) = slot {
                // The resync spawn-crit conversion: no throw-time stamp —
                // the per-hit roll reads the class flags; the
                // ApplyStatsFromSource stamps ride every spawn (:11128-11132).
                app.projectiles.set_armor_penetration(slot, arpen);
                app.projectiles.set_bonus_crit_chance(slot, revolver_bonus);
            }
        }
    } else if held.kind == 1930 {
        // S1-P-i21 — `Player.cs:47647-47662` — the Razorpine's volley:
        // `2 + Next(3)` needles, each spread by `Next(-35, 36)
        // * 0.025 * index` per axis then renormalized to
        // `shootSpeed`, spawned `1.75 * (count - index)` px
        // ahead along the BASE aim, `ai0 = Next(0, 10 *
        // (index + 1))`. Draw order per needle: spread X,
        // spread Y, then the ai0 roll (the NewProjectile
        // argument evaluates after the speed).
        let count = 2 + loot_rng.next_range(0, 3);
        for n in 0..count {
            let spread = 0.025 * n as f32;
            let mut vx = shot.x + loot_rng.next_range(-35, 36) as f32 * spread;
            let mut vy = shot.y + loot_rng.next_range(-35, 36) as f32 * spread;
            let k = fx_spd / (vx * vx + vy * vy).sqrt();
            vx *= k;
            vy *= k;
            let lead = 1.75 * (count - n) as f32;
            let ai0 = loot_rng.next_range(0, 10 * (n + 1)) as f32;
            let slot = app.projectiles.spawn_with_data_ai(
                proj, spawn.x + shot.x * lead, spawn.y + shot.y * lead,
                vx, vy, raw as i32, fx_kb, ai0, 0.0,
            );
            if let Some(slot) = slot {
                // The resync spawn-crit conversion: no throw-time stamp —
                // the per-hit roll reads the class flags; the
                // ApplyStatsFromSource stamps ride every spawn (:11128-11132).
                app.projectiles.set_armor_penetration(slot, arpen);
                app.projectiles.set_bonus_crit_chance(slot, revolver_bonus);
            }
        }
    } else if held.kind == 1931 {
        // S1-P-i21 — `Player.cs:47663-47692` — the Blizzard Staff's two
        // icicles from the sky. Per icicle: a point 600 px above
        // the mounted centre, offset toward the cursor by
        // `Next(201) * -direction`, averaged with the centre X
        // and jittered `Next(-200, 201)`, dropped `100 * index`;
        // the aim re-derived from that point, its Y flipped
        // positive and floored at 20, normalized to shootSpeed;
        // the velocity takes `Next(-40, 41) * 0.02` per axis and
        // `ai1 = Next(5)` (the icicle row). Draw order: Next(201),
        // Next(-200, 201), spread X, spread Y, ai1.
        let (cx, cy) = p.center();
        for n in 0..2 {
            let sky_x = p.x + crate::player::WIDTH * 0.5
                + loot_rng.next_range(0, 201) as f32 * -(p.facing as f32)
                + (world_pos.x - p.x);
            let sx = (sky_x + cx) / 2.0 + loot_rng.next_range(-200, 201) as f32;
            let sy = cy - 600.0 - 100.0 * n as f32;
            let mut dx = world_pos.x - sx;
            let mut dy = world_pos.y - sy;
            if dy < 0.0 {
                dy = -dy;
            }
            if dy < 20.0 {
                dy = 20.0;
            }
            let k = fx_spd / (dx * dx + dy * dy).sqrt();
            dx *= k;
            dy *= k;
            let vx = dx + loot_rng.next_range(-40, 41) as f32 * 0.02;
            let vy = dy + loot_rng.next_range(-40, 41) as f32 * 0.02;
            let ai1 = loot_rng.next_range(0, 5) as f32;
            let slot = app.projectiles.spawn_with_data_ai(
                proj, sx, sy, vx, vy, raw as i32, fx_kb, 0.0, ai1,
            );
            if let Some(slot) = slot {
                // The resync spawn-crit conversion: no throw-time stamp —
                // the per-hit roll reads the class flags; the
                // ApplyStatsFromSource stamps ride every spawn (:11128-11132).
                app.projectiles.set_armor_penetration(slot, arpen);
                app.projectiles.set_bonus_crit_chance(slot, revolver_bonus);
            }
        }
    } else if held.kind == 1244 {
        // S1-P-i21 — `Player.cs:48271-48277` — the Nimbus Rod's cloud:
        // the generic shot at the aim normalized to shootSpeed,
        // then `ai[0..1]` overwritten with the cursor world
        // point (the cloud flies to it and settles).
        let slot = app.projectiles.spawn_with_data_ai(
            proj, spawn.x, spawn.y, shot.x, shot.y, raw as i32, fx_kb,
            world_pos.x, world_pos.y,
        );
        if let Some(slot) = slot {
            app.projectiles.set_armor_penetration(slot, arpen);
            app.projectiles.set_bonus_crit_chance(slot, revolver_bonus);
        }
    } else if held.kind == 1801 {
        // S1-P-i21 — `Player.cs:48344-48354` — the Bat Scepter's bats:
        // `Next(2, 4)` bats, each at the aim plus
        // `Next(-35, 36) * 0.05` per axis (no renormalize).
        // Draw order per bat: spread X, spread Y.
        let count = loot_rng.next_range(2, 4);
        for _ in 0..count {
            let vx = shot.x + loot_rng.next_range(-35, 36) as f32 * 0.05;
            let vy = shot.y + loot_rng.next_range(-35, 36) as f32 * 0.05;
            let slot = app.projectiles.spawn_with_data_ai(
                proj, spawn.x, spawn.y, vx, vy, raw as i32, fx_kb, 0.0, 0.0,
            );
            if let Some(slot) = slot {
                // The resync spawn-crit conversion: no throw-time stamp —
                // the per-hit roll reads the class flags; the
                // ApplyStatsFromSource stamps ride every spawn (:11128-11132).
                app.projectiles.set_armor_penetration(slot, arpen);
                app.projectiles.set_bonus_crit_chance(slot, revolver_bonus);
            }
        }
    } else if tr_core::whip::is_whip(proj) {
        // S1-P-i24 — `Player.ItemCheck_Shoot`'s whip arm
        // (`Player.cs:48655-48690`): the 18 whip items fire
        // their own projectile with a per-throw spin seed.
        // The C# `return`s here, so the generic tweaks above
        // never run for a whip. The audible swing bumps the
        // Snake Band repeat counter (`:48684-48688`); the
        // mid-animation refire is a separate trigger above.
        // The damage is `GetWeaponDamage` (`Player.cs:42653`
        // → `:53421-53429`, the `+5E-06f` epsilon), the same
        // value the refire reads.
        let whip_dmg = whip_weapon_damage(fx_dmg, p.weapon_damage_multiplier(class));
        // The shared `p` read borrow ends here (NLL); the whip helper
        // re-borrows the player mutably for the repeat-counter write.
        spawn_whip_shot(
            &mut app.projectiles,
            app.player.as_mut(),
            loot_rng,
            held,
            def,
            held_fx,
            proj,
            spawn,
            shot,
            whip_dmg,
            fx_kb,
            false,
        );
    } else {
        // The generic `Item.shoot` firing, with the two
        // per-item tweaks: the Crystal Storm's +/-0.04 spread
        // (`Player.cs:47821-47827`) and the Ice Rod's tile
        // target in `ai0/ai1` (`:49383-49387`).
        let (mut vx, mut vy) = (shot.x, shot.y);
        if held.kind == 518 {
            vx += loot_rng.next_range(-40, 41) as f32 * 0.04;
            vy += loot_rng.next_range(-40, 41) as f32 * 0.04;
        }
        let (ai0, ai1) = if proj == crate::projectile::MAGICAL_ICE_BLOCK {
            (
                (world_pos.x / 16.0) as i32 as f32,
                (world_pos.y / 16.0) as i32 as f32,
            )
        } else if proj == crate::projectile::PURPLE_LASER {
            // S1-P-i22 — `Player.cs:49379-49382` - the Laser Rifle's
            // PurpleLaser rides `ai0 = Main.rand.Next(3, 6)`
            // (the extraUpdates remap the rider reads).
            (loot_rng.next_range(3, 6) as f32, 0.0)
        } else {
            (0.0, 0.0)
        };
        // `Player.cs:49426-49429` - the Leaf Blower's spawn
        // jitter rides the generic shot: the point takes
        // `NextVector2Circular(12, 12)` minus the full
        // velocity vector. `Utils.cs:1266-1269`: the unit
        // vector of one angle draw times the half-widths times
        // a second radial draw — two draws, in that order.
        let mut leaf_spawn = spawn;
        if proj == crate::projectile::LEAF {
            let angle = loot_rng.next_float() * std::f32::consts::TAU;
            let off = Vec2::from_angle(angle) * 12.0 * loot_rng.next_float();
            leaf_spawn += off - Vec2::new(vx, vy);
        }
        // `Player.cs:49409-49411` — the Aqua Scepter's spawn
        // `ai[2]` seeds the lava-bounce LCG (`Next(0, 20000)`);
        // the draw rides the generic tail's ai-payload block,
        // after the leaf jitter and before the spawn.
        let aqua_seed = if proj == crate::projectile::AQUA_SCEPTER {
            Some(loot_rng.next_range(0, 20000) as f32)
        } else {
            None
        };
        // `Player.cs:49413-49425` — the dual-flail sibling launch (S1-P-i07,
        // transplanted from main at the resync): throwing the Blue Moon 26 /
        // Sunfury 35 while the OTHER flail sits in the inventory spawns it
        // too, at the MIRRORED velocity (`0f - num4, 0f - num5`), with that
        // item's own GetWeaponDamage / GetWeaponKnockback and `ai2 = 1f`
        // (the drawLayer flag). The scan walks inventory slots 0..50 for the
        // first non-air item whose shoot is the other flail, and runs BEFORE
        // the primary spawn (`:49437`).
        if proj == crate::projectile::BLUE_MOON || proj == crate::projectile::SUNFURY {
            for sib in app.inventory.iter().take(50) {
                if sib.count == 0 {
                    continue; // `!item.IsAir`
                }
                let Some(sib_proj) = registry::projectile(sib.kind) else {
                    continue;
                };
                if sib_proj == proj
                    || (sib_proj != crate::projectile::BLUE_MOON
                        && sib_proj != crate::projectile::SUNFURY)
                {
                    continue;
                }
                let sib_def = registry::get(sib.kind);
                let sib_fx = crate::prefix::prefixed_stats(sib.kind, sib.prefix);
                let (sib_dmg, sib_kb_base, sib_class) = match (sib_def, sib_fx) {
                    (Some(d), Some(fx)) => (fx.damage, fx.knock_back, d.class),
                    (Some(d), None) => (d.damage, d.knock_back, d.class),
                    (None, _) => continue,
                };
                // `GetWeaponDamage` (`Player.cs:53421-53429`):
                // `(int)(damage * multiplier + 5E-06f)`.
                let sib_dmg = (sib_dmg as f32
                    * p.weapon_damage_multiplier(sib_class)
                    + 5.0e-6) as i32;
                // The i25 `&ItemDef` knockback form (the resync keep-one —
                // main's DamageClass-arg call re-armed).
                let sib_kb = p.weapon_knockback(sib_def.unwrap(), sib_kb_base);
                if let Some(sib_slot) = app.projectiles.spawn_with_data_ai(
                    sib_proj,
                    leaf_spawn.x,
                    leaf_spawn.y,
                    -vx,
                    -vy,
                    sib_dmg,
                    sib_kb,
                    0.0,
                    0.0,
                ) {
                    // `ai9 = 1f` → the sibling's `ai[2]`.
                    app.projectiles.set_ai2(sib_slot, 1.0);
                }
                break;
            }
        }
        let slot = app.projectiles.spawn_with_data_ai(
            proj, leaf_spawn.x, leaf_spawn.y, vx, vy, raw as i32, fx_kb, ai0, ai1,
        );
        if let Some(slot) = slot {
            app.projectiles.set_armor_penetration(slot, arpen);
            app.projectiles.set_bonus_crit_chance(slot, revolver_bonus);
            if let Some(seed) = aqua_seed {
                app.projectiles.set_ai2(slot, seed);
            }
            // S1-P-i25 round 2 — `Player.cs:49442-49444`: the
            // Clay Pot (6161) rides THIS generic tail (no
            // `SpawnMinionOnCursor` arm — player-centre spawn,
            // aim velocity, SCALED `Damage`), then overrides
            // `originalDamage = damage` (the UNSCALED item
            // damage) so the pool's `summon_tick_block`
            // (`Projectile.cs:16707`) recomputes off it.
            if held.kind == 6161 {
                app.projectiles.set_original_damage(slot, fx_dmg);
            }
            // `ApplyStatsFromSource` (`Projectile.cs:11119-11141`):
            // the whip's tag row off the firing item + the
            // prefix's `bonusTagDamage`, plus `AI_165_Whip`'s
            // `timeToFlyOut` inputs (the prefixed
            // `itemAnimationMax`).
            let tag_type = if tr_core::whip::tag_effect(held.kind).is_some() {
                held.kind
            } else {
                0
            };
            let bonus_tag = held_fx.map_or(0, |s| s.tag_damage);
            let animation_max =
                held_fx.map_or(def.use_animation as f32, |s| s.use_animation as f32);
            app.projectiles
                .set_whip_params(slot, tag_type, bonus_tag, animation_max);
        }
    }
    // `Player.cs:48210`/`:49092` — the sentry spawns run the turret
    // sweep right after (the base cap of 1 would otherwise
    // accumulate unbounded sentries; the budget-change sweep
    // in the player tick only catches cap drops). The cap was
    // snapshotted at the head (the whip arm re-borrows the
    // player mutably — S1-P-i24's repeat-counter write).
    // S1-P-i26: the `DD2Event.Ongoing` read is the world's live
    // flag (`TurretShouldPersist` gates on it, `Projectile.cs:
    // 670-690`), not the `event_ongoing()` stub — during the
    // event the O'A towers persist through the sweep.
    if crate::projectile::proj_data(proj).is_some_and(|d| d.sentry) {
        app.projectiles.update_max_turrets(
            max_turrets,
            app.local_id as u8,
            app.world.dd2_event_ongoing,
        );
    }
    // S1-P-i28 — `Player.ItemCheck`'s consumable arm (`Player.cs:43280-
    // 43326`, transplanted from main at the resync): the decrement is NOT
    // unconditional. For a `ranged` consumable the four not-consume rolls
    // run in order on the `UpdatePlayers` stream, each gated by its flag and
    // each consuming its draw even when an earlier roll already saved the
    // stack (the C# `if`s are independent, no short-circuit):
    //   huntressAmmoCost90 → Next(10)==0  (`:43285`)
    //   chloroAmmoCost80   → Next(5)==0   (`:43289`)
    //   ammoCost80         → Next(5)==0   (`:43293`)
    //   ammoCost75         → Next(4)==0   (`:43297`)
    // The three water bottles carry no `ranged` flag, so they skip the roll
    // chain and always consume.
    let mut consume = true;
    if def.ranged {
        if huntress90 && loot_rng.next_range(0, 10) == 0 {
            consume = false;
        }
        if chloro80 && loot_rng.next_range(0, 5) == 0 {
            consume = false;
        }
        if ammo80 && loot_rng.next_range(0, 5) == 0 {
            consume = false;
        }
        if ammo75 && loot_rng.next_range(0, 4) == 0 {
            consume = false;
        }
    }
    if def.consumable && consume {
        // `if (item.stack > 0) item.stack--` (`:43317-43320`).
        app.inventory[app.selected].count =
            app.inventory[app.selected].count.saturating_sub(1);
        // `if (item.stack <= 0) itemTime = itemAnimation` (`:43321-43324`):
        // the last throw locks the action cadence to the remaining animation
        // so the empty slot cannot re-fire. The seam fires on the press
        // frame; the use-start caller arms the swing right after, so the
        // latch reads the swing it just armed (the mid-swing volley reads
        // the live one).
        if app.inventory[app.selected].count == 0 && app.swing > 0.0 {
            app.item_time = app.swing;
        }
    }
    true
}

/// `Player.ItemCheck`'s tile-first order for the tools: the cursor cell is a
/// minable/chopable tile the held tool can work, and the vanilla checks the
/// tile against the tool before the NPC boxes — `ItemCheck_UseMiningTools`
/// (`Player.cs:43593`) runs ahead of the melee-hit box block
/// (`Player.cs:43644-43686`). The weapons take the NPC arm regardless
/// (`damage > 0` and `!noMelee`, `Player.cs:43648`).
fn cursor_cell_is_tile_target(app: &App, world_pos: Vec2) -> bool {
    let held = app.inventory[app.selected];
    if held.count == 0 {
        return false;
    }
    let tx = (world_pos.x / TILE_PX).floor() as i32;
    let ty = (world_pos.y / TILE_PX).floor() as i32;
    let Some(tile) = app.world.tile(tx, ty).copied() else {
        return false;
    };
    if !tile.active() {
        return false;
    }
    if tr_core::tool::is_axe_tile(tile.kind) {
        return registry::get(held.kind).map(|d| d.axe).unwrap_or(0) > 0;
    }
    // S1-P-i31 — the hammer tiles (`Main.tileHammer`) answer to the hammer
    // power (`Player.cs:45656`), never the pick.
    if tr_core::tool::is_hammer_tile(tile.kind) {
        return registry::get(held.kind).map(|d| d.hammer).unwrap_or(0) > 0;
    }
    let pick = registry::get(held.kind).map(|d| d.pick).unwrap_or(0);
    let below_surface = (ty as f64) > app.world.world_surface;
    pick > 0 && tr_core::tool::pick_can_mine(tile.kind, pick as i32, below_surface)
}

    // Melee: LMB over an NPC within four tiles hits it instead of mining.
    // `ItemCheck_MeleeHitNPCs` runs for every item with `damage > 0` and
    // `!noMelee` (`Player.cs:43648`) — the tools included, their damage
    // coming from the same registry row. The ammo shooters skip it — they
    // fire their `useAmmo` projectile instead (`ItemCheck_Shoot`'s gate,
    // `Player.cs:47069-47075`). The vanilla prioritises the TILE under the
    // cursor for the tools: an NPC hit only lands when the cell under the
    // cursor is not a targetable tile (the `ItemCheck` tile-first order).
    // S1-P-i05: the useStyle-3 sweep above owns the umbrella pair — the
    // cursor arm must not double-fire (`thrust_took`).
    if app.mine_down
        && !thrust_took
        && held_use_ammo == 0
        && app.attack_cd <= 0.0
        && app.reuse_delay <= 0.0
        && app.mine_released
        && !app.inventory_open
        && (held_weapon || !cursor_cell_is_tile_target(app, world_pos))
    {
        let ax = world_pos.x;
        let ay = world_pos.y;
        let (pcx, pcy) = app.player.as_ref().map(|p| p.center()).unwrap_or((ax, ay));
        // `:44383`'s `this.GetArmorPenetration(sItem.melee)` sum, read before
        // the `app.npcs` borrow below.
        let melee_pen = app
            .player
            .as_ref()
            .map_or(0, |p| p.get_armor_penetration(true));
        let reach2 = 4.0 * TILE_PX;
        if (ax - pcx).powi(2) + (ay - pcy).powi(2) <= reach2 * reach2 {
            // `ProcessHitAgainstNPC`'s friendly gate (`Player.cs:44272-44276`):
            // `nPC.friendly && (nPC.type != 22 || !this.killGuide) &&
            // (nPC.type != 54 || !this.killClothier) && (!nPC.isLikeATownNPC
            // || (sItem.type != 5129 && sItem.type != 3351))` → `return`.
            // So every friendly NPC (all town kinds plus the six
            // bound/rescue forms, [`crate::npc::NpcKind::is_friendly`]) is
            // refused except: the Guide under the Guide Voodoo Doll (267),
            // the Clothier under the Clothier's Voodoo Doll (1307), and any
            // town NPC under the Flymeal (5129) or the Tax Collector's Stick
            // of Doom (3351) — the two items the C# whitelists by id.
            // `dontTakeDamage` skips the shielded (the Brain's Creeper
            // shield, the crystal's spawn shield). The dolls arm their flags
            // through `ApplyEquipFunctional` (`Player.cs:14368`, the
            // accessory slots 3..10 the `UpdateEquips` loop `:12948-12954`
            // walks) and `ResetEffects` clears them every tick (`:18365-18366`
            // — the port reads the per-tick `Player.equipped` snapshot, so a
            // doll in the bag no longer counts; the pre-fix gate scanned the
            // whole inventory).
            let (kill_guide, kill_clothier) = app
                .player
                .as_ref()
                .map(|p| p.doll_flags())
                .unwrap_or((false, false));
            let hits_town_npc = matches!(held.kind, 5129 | 3351);
            let hit_idx = app.npcs.iter().position(|n| {
                n.contains(ax, ay)
                    && !n.dont_take_damage
                    && (!n.kind.is_friendly()
                        || (n.kind == NpcKind::Guide && kill_guide)
                        || (n.kind == NpcKind::Clothier && kill_clothier)
                        || (n.kind.is_town_npc_like() && hits_town_npc))
            });
            if let Some(idx) = hit_idx {
                // S1-P-i05/i10/i11 keep-one (M1 re-sync): the landed-strike
                // resolution lives in the extracted resolve_melee_strike —
                // the cursor arm and the useStyle-3 stab sweep share it. The
                // V4 i50 Lucky Coin proc was folded INTO the shared fn (it
                // sits after the hit-cue tail there, C# ApplyNPCOnHitEffects
                // order :44404). The swing-start tail below is i12's
                // start_use_timers (ApplyItemAnimation, V4-landed).
                resolve_melee_strike(app, idx, held, held_weapon, held_fx, melee_pen, pcx);
                // One swing per `ApplyItemAnimation` (`Player.cs:4463-4497`):
                // the melee branch scales by the inverted meleeSpeed, the
                // reuse lock rides the start (`:4496`).
                start_use_timers(app, held);
                app.mine_target = None;
                app.mine_timer = 0.0;
                return;
            }
        }
        // The healing-potion duplicate that used to sit here (the gated
        // `!sick` copy) was SHADOWED by the ungated press-arm above and is
        // gone with it (S1-P-i37): every melee strike already returns, so
        // the block was unreachable for NPCs in reach, and the drink family
        // now rides `potions::use_potion_item` at the use-start seam.
    }
    // `Player.cs:42768-42771` (the use-success arm) + the can-use gate
    // `:52312-52314` -> `Mount.CanMount` (`Mount.cs:6581-6604`): a saddle
    // item equips its mount (a swap while mounted goes through the same
    // `SetMount` guards, `Mount.cs:6196-6213`). Sits after the weapon/potion
    // arms so the sim-context player borrow is already dead (NLL). The start
    // gate is i12's landed form (`:42733` + the auto-reuse forced release);
    // `ItemCheck_CheckCanUse_Inner`'s noItems gate (`:52140-52148`,
    // ignoreCursed = false, S1-P-i41) rides the same probe.
    if app.mine_down
        && held.count > 0
        && !app.inventory_open
        && app.attack_cd <= 0.0
        && app.reuse_delay <= 0.0
        && app.mine_released
    {
        if let Some(m) = crate::mount::mount_type_of_item(held.kind) {
            // The can-mount check and the mount swap run under their own
            // player borrows - `start_use_timers` takes the whole app (the
            // sound seam), so the borrow must end before it.
            let can = app.player.as_ref().is_some_and(|p| {
                p.can_use_item_past_no_items(&app.world, false) && p.can_mount(&app.world, m)
            });
            if can {
                if let Some(p) = app.player.as_mut() {
                    let wet = p.wet;
                    p.set_mount(m, wet);
                }
                start_use_timers(app, held);
                app.mine_target = None;
                app.mine_timer = 0.0;
                return;
            }
        }
    }
    let held_wall = held.count > 0 && registry::place_wall(held.kind).is_some();

    // S1-P-i29 — the pickaxe/drill mining interaction, ported from
    // `Player.ItemCheck_UseMiningTools` (`Player.cs:45584-45644`) →
    // `ItemCheck_UseMiningTools_ActuallyUseMiningTool` (`:45646-45782`) →
    // `PickTile` (`:53757-53826`) → `GetPickaxeDamage` (`:53902-54045`).
    // The per-tick `toolTime` block (`:43576-43587`) mirrors `itemTime`
    // while not channeling, or counts down on a channel drill; the pick hit
    // fires at `toolTime == 0 && itemAnimation > 0 && controlUseItem`
    // (`:45608`/`:45626`) and re-arms `itemTime = (int)(useTime * pickSpeed)`
    // (`:45779`). The damage accumulates in the `HitTile` book
    // (`tr_core::hit_tile`, `HitTile.cs`) until 100 breaks the tile.
    //
    // No `frame_important` gate on this branch — vanilla's LMB mining chain
    // has none: `tileFrameImportant` is read only by the placement/replacement
    // paths (`Player.cs:40019`, `:40608`), never by the pick tick. Cactus 80
    // is `tileAxe` (`Main.cs:7210`), so a pick does not break it — the axe
    // gate covers it (the i30 axe task owns the chop pacing).
    let held_def = registry::get(held.kind);
    let is_mining_tool =
        held_def.is_some_and(|d| d.pick > 0 || d.axe > 0 || d.hammer > 0);
    // The drill channel start (`ItemCheck_StartActualUse` → `StartChanneling`,
    // `Player.cs:51677`/`:52958-52968`): a channel drill spawns its drill
    // projectile on the press edge and latches the channel. The projectile's
    // aiStyle-20 machine holds the animation (`SetDummyItemTime(2)`,
    // `Projectile.cs:25563`) and kills itself on the release (`:25549`).
    if app.mine_down && !app.inventory_open && is_mining_tool {
        let _ = drill_channel_start(app, held, world_pos);
    }
    // The drill channel hold: while the drill projectile lives, the item
    // animation HOLDS at the dummy 2 ticks (`Projectile.cs:25563`'s
    // `SetDummyItemTime(2)` — `itemAnimation = itemTime = 2`,
    // `itemAnimationMax = 3`, `Player.cs:4432-4441`). The port keeps the
    // swing alive at the same 2-tick value so the `toolTime == 0 &&
    // itemAnimation > 0` mining gate keeps firing every `useTime` ticks.
    let drill_holding = drill_live(app, held) && app.mine_down && !app.inventory_open;
    if drill_holding {
        app.swing = 2.0 * ONE_TICK_SECS;
        app.swing_max = 3.0 * ONE_TICK_SECS;
    }
    if !app.inventory_open && is_mining_tool {
        // S1-P-i31 — the hammer interaction (`ItemCheck_UseMiningTools`,
        // `Player.cs:45584-45644`). The C# order: the `poundRelease` clear
        // (`:45608-45615`, BEFORE the reach return), the mining action
        // (`:45626-45634`), the release latch (`:45635-45638`), the wall
        // pass (`:45639-45643`). The action-ready gate mirrors `:45608`:
        // `toolTime == 0 && itemAnimation > 0 && controlUseItem`.
        let action_ready =
            ticks_of(app.tool_time) <= 0 && app.swing > 0.0 && app.mine_down;
        // `:45608-45615` — a non-target cursor tile clears the pound latch
        // (runs even out of reach — the C# clear precedes the `!flag`
        // return at `:45616`).
        if action_ready {
            let target_ok = app
                .world
                .tile(tx, ty)
                .is_some_and(|t| t.active() && edit::is_a_hammer_target(t.kind));
            if !target_ok {
                app.pound_release = false;
            }
        }
        let (range_x, range_y) = mining_tile_range(app);
        let in_reach = app.player.as_ref().is_some_and(|p| {
            tile_in_reach_rect(p, tx, ty, range_x, range_y, held_def.map_or(0, |d| d.tile_boost))
        });
        if app.mine_down && in_reach {
            // The tile arm (`:45626-45634`) — `mining_tick` handles the
            // inactive-cell early return (`:45652-45654`) itself and leaves
            // `mining_can_hit_walls` true for the wall pass.
            mining_tick(app, tx, ty, held);
        } else {
            app.mine_target = None;
            app.mine_timer = 0.0;
            app.mining_can_hit_walls = true;
        }
        // `:45635-45638` — `releaseUseItem` sets the pound latch (the
        // release tick and every tick after, until a pound consumes it).
        if in_reach && app.mine_released {
            app.pound_release = true;
        }
        // `:45639-45643` — the wall pass: the finder + the hit gate +
        // `PickWall`. The gate reads the SAME `toolTime == 0` the action
        // used (the C# `toolTime` does not change inside UseMiningTools —
        // `ApplyItemTime` writes `itemTime` only).
        //
        // Gate merge (i31 ⨉ i33): i33's timer-based wall cadence (written
        // against pre-i29 main) is SUPERSEDED by the i31 `wall_hammer_tick`
        // — the faithful `HitTile` WALL-book port of the same `PickWall`
        // (`Player.cs:45942-45967`: integer `(int)(hammer * 1.5f)` damage,
        // break at >= 100, `itemTime = useTime / 2` at `:45913`), with the
        // finder (`:45832-45906`) and the full `:45910` gate composition
        // (incl. `CanPlayerSmashWall` on the FULL `Main.wallHouse` table,
        // i33's `edit::wall_house`). The i33 enclosed-unsafe refusal and
        // the drop-conversion table ride `wall_hammer_tick` → `break_wall`.
        if in_reach && action_ready && app.mining_can_hit_walls {
            wall_hammer_tick(app, tx, ty, held, (world_pos.x, world_pos.y));
        }
    } else {
        app.mine_target = None;
        app.mine_timer = 0.0;
    }

    // `Player.ItemCheck` pacing: the swing animation lasts `useAnimation`
    // (`Player.cs:4443-4451`, through `ApplyItemAnimation`'s speed branches);
    // the *action* re-fires whenever `itemTime` reaches zero while the
    // animation still runs and the button is held (`Player.cs:39743`
    // `ItemTimeIsZero && itemAnimation > 0 && controlUseItem`,
    // `ApplyItemTime = useTime`, `Player.cs:4416-4418`) — dirt places every
    // 10 ticks inside the 15-tick swing, walls every 7. An `autoReuse` kind
    // restarts the swing the moment it ends (`Player.cs:42819-42822` +
    // `ItemCheck_AutoReuseLogic`'s forced release, `:52885`); a non-autoReuse
    // kind waits for a release. The reuse lock (`reuseDelay`, `:4496` +
    // `ApplyReuseDelay` `:52835`) gates the START: while it counts down no
    // new use begins; the converted hold keeps `itemTime > 0` so no
    // mid-swing action fires inside it either.
    let timings = registry::get(held.kind);
    let swing_secs = apply_item_animation_secs(app, held);
    // `ApplyItemTime(sItem)` (`Player.cs:4416-4419`): `SetItemTime(useTime)`
    // reads the PREFIXED instance (`Item.cs:578` scales `useTime` with `spd`).
    let action_secs = crate::prefix::prefixed_stats(held.kind, held.prefix)
        .map(|s| s.use_time)
        .or_else(|| timings.map(|d| d.use_time as i32))
        .filter(|t| *t > 0)
        .map(|t| t as f32 * ONE_TICK_SECS)
        .unwrap_or(swing_secs);
    // `Player.channel`'s toolTime arm (`Player.cs:43576-43587`): while the
    // channel is open the tool cadence runs off its OWN `toolTime--` counter
    // (reset to `useTime - 1` on underflow) instead of `itemTime` — the
    // channel item's animation holds. The port's stand-in keeps the swing
    // alive while the LATCHED channel is open on a projectile-less channel
    // item (the mounts, the fairy bell); the latch drops on release/CC/kill/
    // death, so the hold ends with it (S1-P-i23 — the per-tick button
    // re-derive would resurrect a kill-cancelled channel).
    let channels = timings.is_some_and(|d| d.channel)
        && app.mine_down
        && !held_weapon
        && app.player.as_ref().is_some_and(|p| p.channel);
    if channels && app.swing <= 0.0 {
        // The hold: the animation stays at the last frame.
        app.swing = 0.2;
        app.attack_cd = app.attack_cd.max(0.2);
    }
    let swing_over = app.swing <= 0.0;
    // The forced release the frame-head auto-reuse block installed stands in
    // for `releaseUseItem` on the USE button; the placement route keeps its
    // own right-button edge (the port maps `controlUseTile` to placement).
    let reuse_ready = app.mine_released || (prev_place_released && !prev_place_down);
    let starting = swing_over && reuse_ready && app.reuse_delay <= 0.0;
    // The reuse hold (`ApplyReuseDelay`'s conversion, `:52835-52840`) runs
    // with `itemAnimationMax == 0` — the mid-swing action gate stays shut
    // inside it (the hold's `itemTime` counts down with the animation, so
    // `ItemTimeIsZero` only lands on the hold's last tick, where the C#
    // animation is also 0 and the gate is shut anyway).
    let in_reuse_hold = app.swing > 0.0 && app.swing_max <= 0.0;
    let inside_swing = app.swing > 0.0 && app.item_time <= 0.0 && !in_reuse_hold;
    // Tile interactions (`Player.Update` order: LookForTileInteractions
    // (`Player.cs:27430`) runs before ItemCheck (`:31827`)): one action per
    // right-click press — `UpdateReleaseUseTile` (`:29224`) clears
    // `releaseUseTile` right after the attempt, so the door branch fires on
    // the press edge. A handled interaction consumes the click; anything else
    // falls through to the place path unchanged. The dispatch order mirrors
    // the C#'s priority: the chest and tile routes first (they set the
    // `player.tileInteractionHappened` the chat arm checks, `Main.cs:43077`),
    // then the town-NPC chat (`Main.cs:42993-43083`), then the collapsed
    // shop arms — the vanilla's chat-panel buttons, which stay reachable
    // through the same-NPC re-click (`talkNPC == i` declines the talk).
    if app.place_down
        && !prev_place_down
        && !app.inventory_open
        && crate::chest::try_open_chest_at_cursor(app, world_pos)
    {
        return;
    }
    // The dresser top-row storage open (`Player.cs:32864-32925`, S1-P-i43):
    // the same right-click priority as the chest route, before the furniture
    // dispatch (the dresser has no `interact_at` interaction of its own).
    if app.place_down
        && !prev_place_down
        && !app.inventory_open
        && crate::chest::try_open_dresser_at_cursor(app, world_pos)
    {
        return;
    }
    if app.place_down && !prev_place_down && !app.inventory_open && interact_tile(app, tx, ty) {
        return;
    }
    // The town-NPC chat arm (`Main.cs:43069-43083`): the region-gated fresh
    // press latches `SetTalkNPC(i)` and fills `Main.npcChatText = GetChat()`;
    // the per-tick close rule (`Player.cs:27494-27503`) rides inside the call.
    // `Player.mouseInterface` (`Main.cs:41799-41813`): the open shop grid's
    // rect suppresses the world use — a press over the panel neither talks
    // nor places.
    let shop_panel_hover = app.inventory_open
        && app.shop_npc.is_some()
        && crate::ui::inventory::shop_slot_at(
            app.mouse_pos,
            app.lookups.ui_back.map_or(52.0, |b| b.width as f32),
        )
        .is_some();
    if !shop_panel_hover && crate::world_sim::npcs::talk_to_npc(app, prev_place_down).is_some() {
        return;
    }
    if app.place_down
        && !prev_place_down
        && !app.inventory_open
        && !shop_panel_hover
        && crate::world_sim::npcs::try_open_shop_at_cursor(app, world_pos)
    {
        return;
    }
    // `ItemCheck_Shoot` (`Player.cs:47056+`): the ammo shooters fire at
    // `ItemTimeIsZero` mid-swing (`Player.cs:43544`'s
    // `itemAnimation > 0 && ItemTimeIsZero` gate). `PickAmmo`
    // (`Player.cs:53458-53676`) picks the ammo, fuses the damage/speed and
    // drains it through the not-consume rolls; `canShoot = false` (no
    // matching ammo) fires nothing but the swing still played.
    // The swing decay and `item_time`'s decrement share this tick — the shot
    // fires on the tick `item_time` hits zero (`ItemTimeIsZero`,
    // `Player.cs:43544`), gated by the swing still running (`itemAnimation > 0`)
    // and a started swing — and it fires on EVERY such tick: `ApplyItemTime`
    // (`ItemCheck_Shoot`'s tail, `Player.cs:47184-47193` — `itemTime = useTime`)
    // re-arms the cadence after each shot, which is what lets a
    // `useTime < useAnimation` shooter (the Clockwork 4<12, the Fairy Queen
    // 2<30) fire its whole volley instead of one shot per swing.
    //
    // ## The tick semantics
    //
    // The C# compares `itemAnimation`, `itemAnimationMax` and `useTime` as
    // INTEGERS (`:4066`, `:53635`, `:53663`) while the port's timers are seconds
    // decaying per frame (`:45`/`:72`), so the wire derives the three counts
    // once, rounded at 60 Hz, and reads every window off those — never a raw
    // float compare against "eight ticks". `item_time` is read the same way so
    // a sub-tick float residue cannot slip a volley shot a frame late.
    let anim_max_ticks = ticks_of(app.swing_max);
    let use_time_ticks = ticks_of(shooter_use_time_secs(held.kind, held.prefix, app.swing_max));
    let remaining_anim_ticks = ticks_of(app.swing);
    let item_time_ticks = ticks_of(app.item_time);
    // `shot_fired` is now the per-SWING first-shot marker (cleared at the start
    // block): the C#'s `ItemAnimationJustStarted` is `itemAnimation ==
    // itemAnimationMax - 1` (`Player.cs:4066`), and the press-frame arming puts
    // the swing's first shot exactly there, so "this is the swing's first shot"
    // is the port's stand-in for that comparison.
    let just_started = !app.shot_fired;
    // `ApplyReuseDelay`'s converted hold (`Player.cs:52835-52840`) sets
    // `itemAnimation = itemTime = reuseDelay` but leaves `itemAnimationMax`
    // at 0 — so a running animation with a zero max IS the reuse hold, and
    // the C#'s `ItemTimeIsZero` shot gate (`:43544`) never fires inside it
    // (the hold's `itemTime` counts down in lockstep with the animation).
    let in_reuse_hold = remaining_anim_ticks > 0 && anim_max_ticks == 0;
    if held_weapon
        && held_use_ammo > 0
        && item_time_ticks <= 0
        && app.swing_max > 0.0
        && remaining_anim_ticks > 0
        && !in_reuse_hold
    {
        app.shot_fired = true;
        // `Player.ammoCyclingMode`/`ammoCyclingOffset` — per-tick granted by
        // `grant_ammo_mastery` (`Player.cs:14528-14544`), so the wire reads the
        // state the fixed step already derived.
        let mut cycling = app
            .player
            .as_ref()
            .map_or(crate::ammo::AmmoCycling::default(), |p| p.ammo_cycling);
        // `PickAmmo_IterateRange`'s Clentaminator override (`Player.cs:53706-53709`):
        // the two solution guns always take their FIRST matching stack,
        // whatever the mode and offset — forced here so `ammo::pick` stays
        // shooter-agnostic.
        if held.kind == 779 || held.kind == 5134 {
            cycling.offset = 0;
        }
        let cycling = (cycling.mode != crate::ammo::AmmoCyclingMode::None).then_some(&cycling);
        let Some(src) = crate::ammo::pick(&app.inventory, &app.equip, held_use_ammo, cycling)
        else {
            // `canShoot = false` (`Player.cs:53462-53466`): the use plays
            // out dry — no shot, no drain. `ItemCheck_Shoot` returns BEFORE
            // `ApplyItemTime` on this path (`Player.cs:47147`), so `itemTime`
            // stays zero and the dry swing keeps re-running the pick, exactly
            // like the vanilla.
            return;
        };
        let ammo_kind = crate::ammo::slot(src, &app.inventory, &app.equip).kind;
        let ammo_def = registry::get(ammo_kind);
        // The `gunProj` four (`ItemID.cs:1327`) run the same chain and then
        // discard its stat fuse (`:47073-47078`) and its projectile
        // (`:48986-49018`) — the wire keeps the resolved projectile only as the
        // roll chain's `projToShoot` key.
        let gun_proj = gun_proj_row(held.kind);
        // `Player.cs:53458-53568`: the resolution chain over the picked ammo —
        // the specific-pair table, the per-weapon overrides and the stat fuse.
        // `None` = nothing to resolve (an ammo kind the registry does not
        // carry): the shot is refused, the dry swing still played.
        let resolved = ammo_def.and_then(|ammo_def| {
            let Some(weapon_def) = registry::get(held.kind) else {
                return None;
            };
            let flags = app
                .player
                .as_ref()
                .map_or(crate::ammo::AmmoFlags::default(), |p| p.ammo_flags());
            // `speed`'s and `projToShoot`'s seeds are `sItem.shootSpeed` /
            // `sItem.shoot` (the INSTANCE's, `Player.cs:47058-47059`) — the
            // prefixed value for a prefixed weapon.
            let weapon_shoot_speed = gun_proj.map_or_else(
                || held_fx.map_or(weapon_def.shoot_speed, |s| s.shoot_speed),
                |g| g.shoot_speed,
            );
            crate::ammo::resolve_projectile(
                held.kind,
                ammo_kind,
                held_use_ammo,
                ammo_def.damage,
                ammo_def.shoot_speed,
                ammo_def.knock_back,
                registry::projectile(held.kind).map_or(0, |p| p as i32),
                weapon_shoot_speed,
                app.inventory[app.selected].kind,
                &flags,
            )
        });
        let Some(resolved) = resolved else {
            return;
        };
        let proj_to_shoot = resolved.projectile;
        // `Player.cs:53569-53577`: the cycling update, inside `PickAmmo` AFTER
        // the resolution chain and BEFORE the roll chain — so a `Random` mode's
        // `Main.rand.Next()` draw (`:53575`) precedes the roll arms' draws.
        // `!dontConsume` only: the four gunProj items never advance the cycler.
        let dont_consume = gun_proj.is_some();
        if !dont_consume && cycling.is_some() {
            if let Some(p) = app.player.as_mut() {
                // `ammoCyclingCooldown = sItem.useAnimation + 120` (`:53571`)
                // — the INSTANCE animation ticks, NOT the bumped
                // `itemAnimationMax` (the style-5 `num` tick, `:4466-4469`).
                p.ammo_cycling.cooldown = shooter_use_anim_ticks(held.kind, held.prefix) + 120;
                p.ammo_cycling.offset += 1;
                if p.ammo_cycling.mode == crate::ammo::AmmoCyclingMode::Random {
                    // `ammoCyclingOffset = Main.rand.Next()` (`:53575`) — one
                    // unbounded draw from the same stream the roll arms below
                    // read (`Player.ItemCheck` → the `UpdatePlayers` swap,
                    // `Main.cs:17754`); `UnifiedRandom::next` is already
                    // .NET's non-negative `Random.Next()`.
                    p.ammo_cycling.offset =
                        app.rngs.stream(crate::app::RngKey::UpdatePlayers).next();
                }
            }
        }
        // `Player.cs:53578-53682`: the drain behind the not-consume rolls, with
        // the windows the C# reads at this tick derived from the integer counts
        // above. `dontConsume` seeds the chain for the four gunProj items
        // (`ItemID.cs:1327`) — their own rolls still spend their draws.
        let flags = app
            .player
            .as_ref()
            .map_or(crate::ammo::AmmoFlags::default(), |p| p.ammo_flags());
        if !crate::ammo::not_consume(
            held.kind,
            held_use_ammo,
            proj_to_shoot,
            &crate::ammo::VolleyWindows {
                just_started,
                // `itemAnimation > sItem.useAnimation - 8` (`:53635`) — the
                // Fairy Queen's first-eight-ticks window reads the INSTANCE
                // animation (30), not the bumped `itemAnimationMax` (31)
                // (S1-P-i12's `shooter_use_anim_ticks` raw-anim seam).
                anim_gt_use_anim_minus_8: remaining_anim_ticks
                    > shooter_use_anim_ticks(held.kind, held.prefix) - 8,
                // `itemAnimation < itemAnimationMax - sItem.useTime` (`:53663`
                // /`:53667`) — the spray/Clentaminator first-shot window DOES
                // read the bumped max.
                anim_lt_anim_max_minus_use_time: remaining_anim_ticks
                    < anim_max_ticks - use_time_ticks,
                // `Main.remixWorld` (`:53659`) — the Snowball Cannon's remix
                // save gate, read off the world's seed flags
                // (`tr_core::seeds::SeedFlags::REMIX`, S1-P-i15).
                remix_world: app
                    .world
                    .seeds
                    .contains(tr_core::seeds::SeedFlags::REMIX),
            },
            &flags,
            dont_consume,
            // `PickAmmo` runs in `Player.ItemCheck` → `UpdatePlayers`.
            app.rngs.stream(crate::app::RngKey::UpdatePlayers),
        ) {
            let mut equip = std::mem::take(&mut app.equip);
            let mut inv = app.inventory;
            // `Player.cs:53675-53682`: the drain mutates the picked slot when
            // `item.consumable` — the gate lives inside `drain` — and the
            // shot fires EITHER WAY: the Endless Quiver / Musket Pouch are
            // picked and fired with no stack change. `drain`'s false return
            // is informational, never a refusal (S1-P-i15).
            crate::ammo::drain(src, &mut inv, &mut equip);
            app.inventory = inv;
            app.equip = equip;
        }
        // The post-PickAmmo bow conversions (`Player.cs:47157-47176`) ride
        // `bow_launch::post_pick_overrides` at the launch (S1-P-i13/i17
        // keep-one — the wire's not-consume rolls above key on the
        // PRE-conversion projectile, the C#'s order).
        // The shot's stats. `Player.cs:53541-53568` fuses the ammo's own damage
        // (through the weapon's class multiplier) and shootSpeed onto the
        // weapon's; `Player.cs:47073-47078` then discards the whole fuse for the
        // four gunProj items, whose shot carries the weapon's own raw values.
        let Some(weapon_def) = registry::get(held.kind) else {
            return;
        };
        // The class the shot's multiplier and crit read: the weapon's own, or
        // the id-keyed vanilla one for the four `gunProj` placeholders (all
        // four are `ranged` in `Item.cs`).
        let class = gun_proj.map_or(weapon_def.class, |g| g.class);
        // The weapon's own (prefixed) instance values. `shoot_speed` is NOT
        // read here: the resolver's fused `speed` already starts from it
        // (`Player.cs:47058-47059`'s seed), and the gunProj reset overrides it
        // with the id-keyed vanilla value.
        let (w_dmg, w_kb, _w_crit) = held_fx.map_or(
            (weapon_def.damage, weapon_def.knock_back, weapon_def.crit),
            |s| (s.damage, s.knock_back, s.crit),
        );
        // The stat fuse reads the player; the bow launch (S1-P-i13)
        // re-borrows it for the geometry/facing flip, so the fuse runs in
        // its own scope and the launch dispatch follows. No crit rolls here:
        // the projectile crit is per-hit in `damage_pve`
        // (`Projectile.cs:12883-12917`, S1-P-i11 — the C# `ItemCheck_Shoot`
        // never rolls a ranged crit at spawn).
        let fused = if let Some(p) = app.player.as_mut() {
            // `Player.GetWeaponDamage` (`:53427`): the weapon's own part is
            // `(int)(damage * multiplier + 5E-06f)` — the C# truncates it
            // BEFORE the fuse (S1-C-4), and the multiplier is the ranged
            // dispatch (`:53396-53398`, the ammo categories).
            let mult = p.weapon_damage_multiplier_with_ammo(class, held.kind, held_use_ammo);
            let weapon_damage = (w_dmg as f32 * mult + 5e-6) as i32;
            let mut damage = weapon_damage as f32;
            let mut speed = resolved.speed;
            let mut knock_back = w_kb;
            if let Some(gun) = gun_proj {
                // `Player.cs:47074-47078`: `KnockBack = sItem.knockBack;
                // Damage = weaponDamage; speed = sItem.shootSpeed`, and the
                // fixed projectile from `ItemCheck_Shoot`'s tail (`:48986-49018`).
                // `weaponDamage` is the truncated, class-multiplied weapon
                // damage (`Player.GetWeaponDamage`, the argument
                // `ItemCheck_Shoot` was called with). The weapon's own raw
                // values come from the id-keyed vanilla table — the scaled
                // registry rows for these four are placeholders.
                damage = weapon_damage as f32;
                speed = gun.shoot_speed;
                knock_back = gun.knock_back;
            } else {
                if resolved.damage > 0 {
                    // `Damage += (int)(num * GetWeaponDamageMultiplier(item))`
                    // (`:53552-53555`) — the AMMO-side contribution the
                    // resolver fused, truncated on its own.
                    damage += (resolved.damage as f32 * mult) as i32 as f32;
                }
                if resolved.projectile == 285 {
                    // `:53556-53559`: the Nanite-bullet arm scales the FUSED
                    // damage — ONE site, after the add (S1-P-i15 / i03 census
                    // A1: the resolver's old ammo-half scale doubled it to
                    // ×1.5625; the resolver now returns the unscaled `num`).
                    damage = (damage * 1.25) as i32 as f32;
                }
                // `KnockBack *= 1.1f` (`:53547`) runs BEFORE
                // `KnockBack += item.knockBack` (`:53568`), so it scales the
                // weapon's own value only — `AmmoShot.knock_back` is the ammo
                // side the resolver added last.
                if flags.magic_quiver
                    && (held_use_ammo == AMMO_ARROW || held_use_ammo == AMMO_STAKE)
                {
                    knock_back *= 1.1;
                }
                knock_back += resolved.knock_back;
            }
            // `Player.cs:47151` — `GetWeaponKnockback` runs AFTER the ammo fuse
            // and the gunProj reset (`:47077-47082`), on the fused value: the
            // summon minionKB add FIRST, then the melee glove's ×2 / the Titan
            // buff's ×1.5 / the shroomite-stealth ranged scale
            // (`:53348-53363`). The i25 helper is the census-corrected order
            // (the resync keep-one — main's inline kbBuff/stealth pair was the
            // subset). The gunProj four are Ranged with no stealth armed in
            // the rigs, so this is the identity there (the DW-4.2
            // raw-knockback pins hold).
            let knock_back = p.weapon_knockback(weapon_def, knock_back);
            // No crit roll at throw time (S1-P-i11, M1 re-sync onto the V5
            // gun lane): the C# `ItemCheck_Shoot` generic tail (`:49437`)
            // carries NO crit roll — `Damage_PVE_Inner` rolls the projectile
            // crit PER HIT (`Projectile.cs:12883-12917`,
            // `damage_pve`'s `projectile_hit_crit`). The Revolver 2269's
            // charge bonus is NOT folded into a throw roll either: the C#
            // stamps it onto the projectile at spawn
            // (`Projectile.ApplyStatsFromSource`, `Projectile.cs:11129-11132`:
            // `bonusCritChance += revolverCritChanceBonus`) and the per-hit
            // roll adds it (`:12886-12894`); the spawn sites below stamp
            // `bonus_crit_chance`. The per-tick charge/decay draw lives in
            // `world_sim/player.rs` (`Player.cs:42677-42703`).
            let revolver_bonus = if held.kind == 2269 {
                p.revolver_crit_chance_bonus
            } else {
                0
            };
            Some((damage, speed, knock_back, revolver_bonus))
        } else {
            None
        };
        let Some((damage, speed, knock_back, revolver_bonus)) = fused else {
            return;
        };
        if crate::bow_launch::is_arrow_shooter(held_use_ammo) {
            // S1-P-i13 — the bow launch: the post-pick `projToShoot` overrides
            // (`Player.cs:47157-47176`), the `RotatedRelativePoint(MountedCenter)`
            // spawn anchor + facing flip (`:47196-47224`), the aim normalized to
            // `speed` (`:47277-47307`) and the per-bow spawn arms (the Tsunami
            // fan, the Stormbow/Blood Rain rain, the Chlorophyte spread, the
            // Betsy fan, the Fairy Queen aimed shot, the Hellwing dual spin, the
            // generic single tail). The module owns the spawn + the audio cue.
            //
            // The two `gunProj` BOWS (the Phantasm 3540, the DD2 Phoenix Bow
            // 3854) ride the SAME launch geometry — `ItemCheck_Shoot` runs the
            // shared anchor/facing/aim (`:47196-47307`) for EVERY shooter before
            // the per-item arms, then their arms fire the fixed bolt at that
            // anchor with the aim velocity (`:48996-48999` the Phantasm → 630,
            // `:49014-49017` the Phoenix → 705). The fixed-projectile reset is
            // carried by passing `g.projectile` (not the resolved arrow): the
            // post-pick overrides are all non-matching for 3540/3854 and neither
            // has a per-bow arm, so they take the generic single tail at the
            // rotated anchor — the C#'s shape, with the pre-i13 invented
            // centre + 10 px offset and the missing facing flip corrected.
            let proj = gun_proj.map_or(resolved.projectile, |g| g.projectile);
            crate::bow_launch::fire(
                app,
                held.kind,
                proj,
                speed,
                damage as i32,
                knock_back,
                world_pos,
                anim_max_ticks,
                remaining_anim_ticks,
            );
        } else if crate::gun_launch::is_bullet_shooter(held_use_ammo) {
            // S1-P-i14 — the gun launch: the SAME shared geometry the bow half
            // uses (the `RotatedRelativePoint(MountedCenter)` anchor, the
            // facing flip, the aim normalized to `speed` — `Player.cs:47196-
            // 47307`, inherited from i13 through `bow_launch::launch_frame`),
            // then the per-gun spread/burst arms (the Minishark/Megashark/Chain
            // Gun/SDMG/Uzi/Gatligator spreads, the Shotgun/Boomstick/Tactical/
            // Quad-Barrel fans, the Clockwork's animation-window tightening,
            // the Xenopopper bubbles, the Onyx Blaster's bolt, the Vortex
            // Beater's fixed 615 + ai0 roll, the Pew-matic Horn's frame roll).
            // The per-gun UseSound plays at the USE START (`start_swing_extras`,
            // `Player.cs:51681-51698`) — the launch itself is silent.
            //
            // The bullet `gunProj` item (the Vortex Beater 3475) rides here with
            // its fixed projectile (`g.projectile`, the `:47077-47082` reset);
            // the Celebration Mk2 3930 is `useAmmo` Rocket — the rocket task's.
            let proj = gun_proj.map_or(resolved.projectile, |g| g.projectile);
            // `sItem.scale` — the prefixed instance scale the Xenopopper's
            // muzzle advance reads (`Player.cs:47605`, `Item.cs:582`).
            let scale = held_fx.map_or(1.0, |s| s.scale);
            // `ApplyStatsFromSource` (`Projectile.cs:11128-11132`): the
            // weapon prefix's armorPenetration + the Revolver's charge bonus
            // ride the shot at spawn; the crit itself is per-hit (i11).
            let arpen = held_fx.map_or(0, |s| s.armor_penetration);
            crate::gun_launch::fire(
                app,
                held.kind,
                proj,
                speed,
                damage as i32,
                knock_back,
                arpen,
                revolver_bonus,
                world_pos,
                remaining_anim_ticks,
                scale,
            );
        } else {
            // The remaining non-arrow, non-bullet shooters (the rocket
            // launchers + the Celebration Mk2 3930, the dart guns, the
            // Clentaminators, the Snowball/Nail cannons) keep the generic
            // spawn — the rocket/dart launch arms are i16's task. The pre-i13
            // centre + 10 px anchor stands for them until then.
            let Some(p) = app.player.as_ref() else {
                return;
            };
            let (pcx, pcy) = p.center();
            let dir = (world_pos - Vec2::new(pcx, pcy)).normalize_or_zero();
            if dir == Vec2::ZERO {
                return;
            }
            // S1-P-i16 — the dart guns' muzzle offsets
            // (`Player.cs:47267-47276`): the Blowgun 986 / Blowpipe 281
            // `pointPosition.X += 6 * direction; pointPosition.Y -= 6f *
            // gravDir`, the Dart Pistol 3007 `X -= 4 * direction; Y -= 2f *
            // gravDir` (the port keeps gravDir 1). The port's spawn keeps
            // its documented centre + 10·dir stand-in (ANALYSIS "Bombs");
            // the offsets ride ON TOP of it — the C# shifts the aim origin
            // too (`:47277-47278`), a sub-pixel aim difference at the
            // port's 80-px test reach.
            let muzzle = match held.kind {
                986 | 281 => Vec2::new(6.0 * p.facing as f32, -6.0),
                3007 => Vec2::new(-4.0 * p.facing as f32, -2.0),
                _ => Vec2::ZERO,
            };
            let spawn = Vec2::new(pcx, pcy) + dir * 10.0 + muzzle;
            let aim = dir;
            // No crit draw at throw time (S1-P-i11): the projectile crit
            // rolls per-hit in damage_pve (Projectile.cs:12883-12917).
            // `ApplyStatsFromSource` (`Projectile.cs:11128`): the weapon
            // prefix's `armorPenetration` rides the ammo shot (V5 landing,
            // kept); the Revolver bonus stamp is :11129-11132 (kind-gated).
            let arpen = held_fx.map_or(0, |s| s.armor_penetration);
            // S1-P-i16 — the launcher/dart per-weapon spawn arms
            // (`ItemCheck_Shoot`'s tail, `Player.cs:48991-49031` +
            // `:49388-49453`), in the C#'s order. The generic spawn below
            // keeps the resolved projectile + the fused stats; these arms
            // override the projectile / ai / velocity where the C# does.
            let mut shot_kind = gun_proj.map_or(resolved.projectile, |g| g.projectile);
            let mut ai0 = 0.0f32;
            let mut ai1 = 0.0f32;
            let vel = Vec2::new(aim.x * speed, aim.y * speed);
            // `Player.cs:47169-47172` — the Electrosphere Launcher fires the
            // FIXED 442 bolt (the PickAmmo additive arm's result discarded).
            if held.kind == 2796 {
                shot_kind = 442;
            }
            // `Player.cs:48991-48995` — the Celebration Mk2: the fixed 714
            // with `ai0 = 5 * Next(0, 20)` (the gunProj reset already set
            // the kind/stats; the ai0 draw is the arm's own).
            if held.kind == 3930 {
                let loot_rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
                ai0 = 5.0 * loot_rng.next_range(0, 20) as f32;
            }
            // `Player.cs:49388-49392` — the 442 bolt seeds its target tile
            // (`Player.tileTargetX/Y` — the port's cursor cell).
            if shot_kind == 442 {
                ai0 = (world_pos.x / TILE_PX).floor();
                ai1 = (world_pos.y / TILE_PX).floor();
            }
            // origin/main’s owned spawner (the owner plumbing of the
            // projectile-ownership task) + this task’s resolution: the pair
            // table’s or the ammo’s own projectile, the gunProj fixed id when
            // the weapon is one of the four, and the fused knockback (the
            // weapon’s own + the ammo’s + the magicQuiver scale), which
            // supersedes the pre-fuse `w_kb` read.
            // (Resync keep-one = MAIN's reconciled ammo tail: the i22
            // 2796→442 force + the 442 tile-seed already live in main's i16
            // block above — the gate's named-const copies were the same
            // arms. The crit stamp is GONE (per-hit canonical), the
            // arpen/revolver spawn stamps ride every ammo spawn.)
            if held.kind == 3546 {
                // `Player.cs:49019-49031` — the Fireworks Launcher: TWO
                // rockets per shot, each a jittered aim (`Next(-40,41) *
                // 0.05` per axis), the spawn 6 px perpendicular to the
                // jittered aim (`RotatedBy(-π/2 * direction)`), the kind
                // `167 + Next(4)`, `ai1 = 1f`. The ammo fuse's stats ride
                // both (the C# passes the same Damage/KnockBack).
                for _ in 0..2 {
                    let loot_rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
                    let jx = aim.x * speed + loot_rng.next_range(-40, 41) as f32 * 0.05;
                    let jy = aim.y * speed + loot_rng.next_range(-40, 41) as f32 * 0.05;
                    let jitter = Vec2::new(jx, jy);
                    let perp = jitter
                        .normalize_or_zero()
                        .rotate(Vec2::from_angle(
                            -std::f32::consts::FRAC_PI_2 * p.facing as f32,
                        ))
                        * 6.0;
                    let pos = spawn + perp;
                    let kind = 167 + loot_rng.next_range(0, 4) as u16;
                    let slot = app.projectiles.spawn_with_data_owned(
                        kind,
                        app.local_id as u8,
                        pos.x,
                        pos.y,
                        jx,
                        jy,
                        damage as i32,
                        knock_back,
                        0.0,
                        1.0,
                    );
                    if let Some(slot) = slot {
                        app.projectiles.set_armor_penetration(slot, arpen);
                        app.projectiles.set_bonus_crit_chance(slot, revolver_bonus);
                    }
                }
            } else {
                let slot = app.projectiles.spawn_with_data_owned(
                    shot_kind,
                    app.local_id as u8,
                    spawn.x,
                    spawn.y,
                    vel.x,
                    vel.y,
                    damage as i32,
                    knock_back,
                    ai0,
                    ai1,
                );
                if let Some(slot) = slot {
                    app.projectiles.set_armor_penetration(slot, arpen);
                    app.projectiles.set_bonus_crit_chance(slot, revolver_bonus);
                }
                // `Player.cs:49450-49453` — the Proximity Mine Launcher
                // caps its owner's mines at 20 (the oldest die).
                if held.kind == 760 {
                    destroy_oldest_proximity_mines_over_cap(app, app.local_id as u8, 20);
                }
            }
            app.audio.play_at("Item_1", pcx, pcy, 1.0);
        }
        // `ApplyItemTime` (`Player.cs:47184-47193` → `SetItemTime(useTime)`,
        // `:4416-4419`): the shot re-arms the action cadence, so the next shot
        // of the volley fires `useTime` ticks later — for as long as the swing
        // still runs (the gate above) and for every shot, not once per swing.
        // This is the whole difference between a one-shot-per-swing port and
        // the vanilla's volley.
        app.item_time = shooter_use_time_secs(held.kind, held.prefix, app.swing_max);
    }
    // S1-P-i20 — the non-ammo volley tick (`Player.cs:43544-43570`): the
    // `itemAnimation > 0 && ItemTimeIsZero` gate fires the shot on EVERY tick the
    // action timer hits zero mid-swing, and `ItemCheck_Shoot`'s `ApplyItemTime`
    // (`:47185-47194` → `SetItemTime(useTime)`, `:4416-4419`) re-arms the cadence
    // after each shot. A `useTime < useAnimation` staff (the Aqua Scepter 8 < 16)
    // fires its whole volley inside one swing — the i01 census item: the pre-fix
    // port fired once per swing. The use-start block above fired the first shot on
    // the press tick and re-armed `item_time = useTime`, so this gate catches only
    // the MID-SWING re-fires (`item_time` is still positive on the press tick). The
    // tick semantics mirror the ammo fire block: integer counts off the 60 Hz-
    // rounded timers, never a raw float compare. The mana pay rides the use start
    // (`ItemCheck_TryStartUse`), NOT each shot — a volley shot spends no extra mana.
    if held_weapon
        && held_use_ammo == 0
        && item_time_ticks <= 0
        && app.swing_max > 0.0
        && remaining_anim_ticks > 0
        // `sItem.IsACoin` → `canShoot = false` (`Player.cs:47103-47106`): coins
        // never fire, so they take no volley.
        && !registry::is_coin(held.kind)
        // `sItem.shootsEveryUse` → `flag4 = ItemAnimationJustStarted`
        // (`Player.cs:43560-43562`): these fire once per use, never mid-swing.
        && !SHOOTS_EVERY_USE.contains(&held.kind)
        // Channel items (the drills/chainsaws, the aimed spells, the Last Prism)
        // fire once per use and then the channel holds the projectile — the C#
        // keeps their `itemTime` armed via the channel/mining cadence, so they
        // never reach a mid-swing re-fire. The port skips them here.
        && !registry::get(held.kind).is_some_and(|d| d.channel)
    {
        // The per-item animation-window limits (`Player.cs:43491`/`:43502`/
        // `:43506`/`:43514`): the shot fires only while the swing is inside
        // `useTime*N` of its start (`flag = false` beyond). The Zenith / True
        // Copper Shortsword use `*3` (their `useTime = useAnimation/3`, so the
        // window spans the whole swing — all three shots fire); the Fairy Queen
        // magic item, the Book Staff and the Daybloom Staff use `*4`. The 4953
        // limit (`:43510`) is the ammo path's, handled by its own fire block.
        let window_blocked = match held.kind {
            4956 | 5669 => remaining_anim_ticks < anim_max_ticks - use_time_ticks * 3,
            4952 | 3852 | 6152 => remaining_anim_ticks < anim_max_ticks - use_time_ticks * 4,
            _ => false,
        };
        if !window_blocked {
            if let (Some(proj), Some(def)) = (registry::projectile(held.kind), registry::get(held.kind)) {
                if fire_non_ammo_shot(app, held, held_fx, world_pos, proj, def, screen) {
                    // `ApplyItemTime` (`Player.cs:47185-47194`): re-arm the cadence
                    // after EACH shot — the next volley shot fires `useTime` ticks
                    // later, for as long as the swing runs.
                    app.item_time = shooter_use_time_secs(held.kind, held.prefix, app.swing_max);
                }
            }
        }
    }
    if app.place_down && !app.inventory_open && (starting || inside_swing) {
        // The extractinator arm first (`PlaceThing`'s order, `Player.cs:39586-39597`:
        // `PlaceThing_ItemInExtractinator` runs before the tile/wall routes). It
        // consumes the tick on its own terms — `ApplyItemTime` makes the swing
        // and the action time the SAME value (`Player.cs:4406-4413`, ×0.33 on
        // tile 642), so the generic place-route timings below must not clobber
        // it; `Some(secs)` is the applied action time.
        let mut extracted_secs = None;
        if !held_wall {
            extracted_secs = crate::world_sim::extract::use_extractinator(app, tx, ty, world_pos);
        }
        let used = if extracted_secs.is_some() {
            true
        } else if held_wall {
            // `PlaceThing_Walls` (`Player.cs:39600-39634`): the replacement
            // arm, the plain place and the fill-empty-space tail — the route
            // owns the success re-read and the stack consumption.
            place_wall_route(app, tx, ty)
        } else if !held_weapon {
            // `Player.ItemCheck_UseWiringTools` (`Player.cs:43572`): the
            // wrench family, the cutter and the actuator answer before the
            // tile-placement route — they place no tile, so the two never
            // compete.
            if use_wiring_tool(app, tx, ty) {
                true
            } else {
                use_tile_item(app, tx, ty)
            }
        } else {
            false
        };
        if used {
            if let Some(secs) = extracted_secs {
                // `SetItemTime` (`Player.cs:4406-4413`): swing == item time ==
                // useTime × multiplier (the extraction's own `ApplyItemTime`).
                app.swing_max = secs;
                app.swing = secs;
                app.item_time = secs;
            } else {
                if starting {
                    // The use start: `ApplyItemAnimation` (`Player.cs:4463-4497`)
                    // — the createTile/createWall branches scale by the
                    // inverted tileSpeed/wallSpeed — and the reuse lock rides
                    // the same start (`:4496`).
                    start_use_timers(app, held);
                }
                // The placement ACTION cadence is speed-scaled too:
                // `PlaceThing_Tiles_PlaceIt` re-arms `ApplyItemTime(item,
                // tileSpeed)` (`Player.cs:40141`) and the wall route
                // `ApplyItemTime(item, wallSpeed)` (`:39623`) — the
                // multiplier variant truncates `(int)(useTime * mult)` and
                // floors at 1 (`:4421-4430`). The wiring tools place no
                // tile/wall and keep the plain `ApplyItemTime` cadence
                // (`ItemCheck_UseWiringTools` re-arms via `SetItemTime`).
                let place_mult = if held_wall {
                    app.player.as_ref().map_or(1.0, |p| p.wall_speed_multiplier())
                } else if timings.is_some_and(|d| d.tile.is_some()) {
                    app.player.as_ref().map_or(1.0, |p| p.tile_speed_multiplier())
                } else {
                    1.0
                };
                app.item_time = if place_mult == 1.0 {
                    action_secs
                } else {
                    let base = ticks_of(action_secs);
                    let mut val = (base as f32 * place_mult) as i32;
                    if base > 0 {
                        val = val.max(1);
                    }
                    val as f32 * ONE_TICK_SECS
                };
            }
        }
    }
}

pub(crate) fn break_wall(app: &mut App, tx: i32, ty: i32) {
    let Some(wall) = edit::break_wall(&mut app.world, tx, ty) else {
        return;
    };
    // `WorldGen.KillWall` plays BEFORE the dust and the drop
    // (`WorldGen.cs:60385` precedes `KillWall_DropItems` `:60400`): the
    // cue table's draws ride the `UpdatePlayers` stream (the hammer break
    // runs in `Player.PickWall`, `Player.cs:45949`, under the
    // `Main.cs:17754` swap) ahead of the drop's own rolls (S1-E-21i).
    let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
    app.audio.play_wall_break_sound(
        edit::kill_wall_sound(wall),
        tx as f32 * TILE_PX,
        ty as f32 * TILE_PX,
        rng,
    );
    // `KillWall_DropItems` (`WorldGen.cs:60443-60450`) drops the
    // `KillWall_GetItemDrops` conversion, not a reverse `createWall` lookup:
    // the unsafe dungeon walls drop their SAFE item (7→135, 94→1378, …),
    // stone 1 drops StoneBlock 26, glass 21 drops 392, and the natural
    // unsafe walls (dirt 2, mud 15, …) drop NOTHING (S1-P-i33).
    let id = edit::kill_wall_drop(wall);
    if id > 0 {
        // The wall break runs in `Player.ItemCheck` — the `UpdatePlayers`
        // stream.
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        spawn_item_drop(&mut app.items, rng, id, 1, tx, ty);
    }
    app.request_edit(tx, ty, 2, 0);
    app.world_key = None;
}

/// `Player.PlaceThing_Walls`' placement gate (`Player.cs:39602-39610`), minus
/// the reach/timing halves the caller owns: the cell must not already carry
/// the held wall (`wall == createWall` refuses, `:39607` — a DIFFERENT wall
/// passes on to the replacement route), and one of the four neighbours must
/// be active OR walled (`!active && wall <= 0` on all four refuses). The
/// cell's OWN tile never supports a wall — the C# reads only the neighbours.
/// A wall may sit behind an active tile.
pub(crate) fn can_place_wall(app: &App, tx: i32, ty: i32) -> bool {
    let Some(t) = app.world.tile(tx, ty) else {
        return false;
    };
    let held = app.inventory[app.selected];
    let Some(create_wall) = registry::place_wall(held.kind) else {
        return false;
    };
    if held.count == 0 || registry::is_weapon(held.kind) {
        return false;
    }
    // `:39607` — the same-wall refusal.
    if t.wall == create_wall {
        return false;
    }
    // `:39607` — the four-neighbour support: active() OR wall > 0. The port's
    // out-of-world read answers the C#'s boundary-tile reads as inactive and
    // wall-less (`PlaceWall`'s own 2-cell guard refuses those cells anyway).
    [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|&(dx, dy)| {
        app.world
            .tile(tx + dx, ty + dy)
            .is_some_and(|n| n.active() || n.wall != 0)
    })
}

/// `Player.PlaceThing_Walls` (`Player.cs:39600-39634`) — the wall placement
/// route: the replacement arm first when tile replacement is enabled
/// (`TileReplacementEnabled`, `:39612-39615` → `PlaceThing_TryReplacingWalls`),
/// then the plain `WorldGen.PlaceWall` (`:39620`) with the success re-read
/// (`wall == createWall`, `:39621`), then `PlaceThing_Walls_FillEmptySpace`
/// (`:39628`). Returns whether either arm consumed the action tick.
pub(crate) fn place_wall_route(app: &mut App, tx: i32, ty: i32) -> bool {
    let held = app.inventory[app.selected];
    let Some(create_wall) = registry::place_wall(held.kind) else {
        return false;
    };
    if !can_place_wall(app, tx, ty) {
        return false;
    }
    // `PlaceThing_TryReplacingWalls` (`Player.cs:39816-39829`): the
    // `PlaceThing_ValidWallForReplacement` gate (`:39831-39839`: wall > 0 and
    // not 350) + `NearFriendlyWall` + `WorldGen.ReplaceWall`. The port's
    // replacement toggle (`builderAccStatus[10]`) defaults ON like vanilla's
    // and has no UI yet (ANALYSIS, S1-P-i33).
    let cell_wall = app.world.tile(tx, ty).map_or(0, |t| t.wall);
    if cell_wall > 0
        && cell_wall != edit::WALL_UNBREAKABLE
        && edit::near_friendly_wall(&app.world, tx, ty)
    {
        if let Some(old) = edit::replace_wall(&mut app.world, tx, ty, create_wall) {
            // `ReplaceWall` plays the OLD wall's `KillWall_PlaySounds` cue
            // (`WorldGen.cs:62550`) and drops its `KillWall_GetItemDrops`
            // item (`:62551`) — both on the `UpdatePlayers` stream, sound
            // before drop, the `KillWall` order. The new wall's placement
            // does NOT play the Dig 0 (that is `PlaceWall`'s own, `:75141`).
            let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
            app.audio.play_wall_break_sound(
                edit::kill_wall_sound(old),
                tx as f32 * TILE_PX,
                ty as f32 * TILE_PX,
                rng,
            );
            let id = edit::kill_wall_drop(old);
            if id > 0 {
                let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
                spawn_item_drop(&mut app.items, rng, id, 1, tx, ty);
            }
            // The C# sends `17` action 22 (`Player.cs:39822`); the port's
            // edit record carries the post-edit wall either way.
            app.request_edit(tx, ty, 3, create_wall);
            app.world_key = None;
            // The replacement consumes the held item like a placement
            // (`item.consumable`, the `Player.cs:43280` tick).
            app.inventory[app.selected].count =
                app.inventory[app.selected].count.saturating_sub(1);
            if app.inventory[app.selected].count == 0 {
                app.inventory[app.selected].kind = 0;
            }
            return true;
        }
        // A refused replacement (350 handled above; progression walls) falls
        // through to the plain place, which the occupied cell refuses —
        // `PlaceThing_TryReplacingWalls` returns `canUse` unchanged on
        // failure (`Player.cs:39828`).
    }
    if edit::place_wall(&mut app.world, tx, ty, create_wall) {
        place_wall_item(app, tx, ty, create_wall);
        // `PlaceThing_Walls_FillEmptySpace` (`Player.cs:39636-39715`).
        fill_empty_wall_space(app, tx, ty, create_wall);
        return true;
    }
    false
}

/// `Player.PlaceThing_Walls_FillEmptySpace` (`Player.cs:39636-39715`): after a
/// successful wall placement with `stack > 1`, each of the four neighbours
/// (left, right, up, down — the `i == 0..3` order) that is empty and has ALL
/// FOUR of its own neighbours carrying `createWall` (`num3 != 4` skips) gets
/// the wall too, consuming one item per fill. The centre cell the caller just
/// placed is part of each neighbour's count, which is why the fill only ever
/// closes single-cell holes.
fn fill_empty_wall_space(app: &mut App, tx: i32, ty: i32, create_wall: u16) {
    // `:39638` — the last item never fills.
    if app.inventory[app.selected].count <= 1 {
        return;
    }
    for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
        let (num, num2) = (tx + dx, ty + dy);
        // `:39663` — only empty cells.
        if app.world.tile(num, num2).is_none_or(|t| t.wall != 0) {
            continue;
        }
        // `:39667-39692` — all four neighbours must carry the SAME wall.
        let mut num3 = 0;
        for (jx, jy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            if app
                .world
                .tile(num + jx, num2 + jy)
                .is_some_and(|t| t.wall == create_wall)
            {
                num3 += 1;
            }
        }
        if num3 != 4 {
            continue;
        }
        // `:39697-39704` — PlaceWall + the success re-read, then the stack
        // decrement (`SetDefaults(0)` at zero).
        if edit::place_wall(&mut app.world, num, num2, create_wall) {
            app.inventory[app.selected].count =
                app.inventory[app.selected].count.saturating_sub(1);
            if app.inventory[app.selected].count == 0 {
                app.inventory[app.selected].kind = 0;
            }
            // `WorldGen.PlaceWall`'s Dig 0 (`WorldGen.cs:75141`) per fill,
            // on the `UpdatePlayers` stream — same as the main placement.
            app.audio.play_place_sound(
                (0, 1),
                num as f32 * TILE_PX,
                num2 as f32 * TILE_PX,
                app.rngs.stream(crate::app::RngKey::UpdatePlayers),
            );
            app.request_edit(num, num2, 3, create_wall);
            app.world_key = None;
            if app.inventory[app.selected].count == 0 {
                return;
            }
        }
    }
}

/// `Player.PlaceThing_Walls`' post-placement tail (`Player.cs:39621-39633`):
/// the item decrement + the Dig 0 + the edit record. The core
/// `edit::place_wall` has already landed by the time this runs (the caller
/// re-reads `wall == createWall`, `:39621`); the fill-empty-space arm
/// (`:39628`) is the route's own tail, not this one's.
pub(crate) fn place_wall_item(app: &mut App, tx: i32, ty: i32, wall: u16) {
    app.inventory[app.selected].count = app.inventory[app.selected].count.saturating_sub(1);
    if app.inventory[app.selected].count == 0 {
        app.inventory[app.selected].kind = 0;
    }
    // `WorldGen.PlaceWall` plays the plain 0 (`WorldGen.cs:75141`) — the
    // case-0 variation + pitch ride the `UpdatePlayers` stream
    // (`Player.ItemCheck` under the `Main.cs:17754` swap).
    app.audio.play_place_sound(
        (0, 1),
        tx as f32 * TILE_PX,
        ty as f32 * TILE_PX,
        app.rngs.stream(crate::app::RngKey::UpdatePlayers),
    );
    app.request_edit(tx, ty, 3, wall);
    app.world_key = None;
}

/// `Chest.CanDestroyChest`'s item scan (`Chest.cs:609-623`) against the
/// app's in-memory chest store: any slot with a positive stack at the
/// anchor refuses the break. The C#'s `type > 0` half is implied — the
/// store only holds real stacks. (M3-CHEST-UI landed the store; the
/// refusal is live — S1-P-i43 retired the stale "stays empty" note.)
fn chest_holds_items(app: &App) -> impl Fn(i32, i32) -> bool + '_ {
    move |x, y| {
        app.world_chests
            .iter()
            .any(|c| c.tile == (x, y) && c.slots.iter().any(|s| s.count > 0))
    }
}

/// Map one legacy `(sound id, style)` cue to the port's pinned asset + volume
/// scale (`LegacySoundPlayer.PlaySound`, `LegacySoundPlayer.cs:360-1100`):
/// the per-case asset family and the style's volume multiplier. The port pins
/// one variation per family (research D3) and does not port the per-play pitch
/// jitter (`Main.rand.Next(-10,11)*0.01` etc. — an audio-only deviation).
/// `None` = a family the port has no cue for.
pub(crate) fn kill_cue((id, style): edit::SoundCue) -> Option<(&'static str, f32)> {
    Some(match (id, style) {
        // case 0 (`:440-452`) — `SoundDig[Next(3)]`; pinned to Dig_1.
        (0, _) => ("Dig_1", 1.0),
        // case 2 (`:516-644`) — `SoundItem[style]`; style 52's volume 0.35
        // (`:567-569`).
        (2, 27) => ("Item_27", 1.0),
        (2, 48) => ("Item_48", 1.0),
        (2, 49) => ("Item_49", 1.0),
        (2, 50) => ("Item_50", 1.0),
        (2, 52) => ("Item_52", 0.35),
        (2, 127) => ("Item_127", 1.0),
        (2, 173) => ("Item_173", 1.0),
        (2, 177) => ("Item_177", 1.0),
        // case 3 (`:646-671`) — `SoundNpcHit[style]`; styles 20-54 halve
        // the volume (`:647-650`).
        (3, 25) => ("NPC_Hit_25", 0.5),
        // case 4 (`:673-694`) — `SoundNpcKilled[style]` (styles 23-57 halve
        // the volume, `:674-677` — neither style used here is in range).
        (4, 11) => ("NPC_Killed_11", 1.0),
        (4, _) => ("NPC_Killed_1", 1.0),
        // case 6 (`:706-715`) — `SoundGrass`.
        (6, _) => ("Grass", 1.0),
        // case 13 (`:783-791`) — `SoundShatter`.
        (13, _) => ("Shatter", 1.0),
        // case 18 (`:865-869`) — `SoundCoins`.
        (18, _) => ("Coins", 1.0),
        // case 21 (`:924-935`) — `SoundTink[Next(3)]`; pinned to Tink_1.
        (21, _) => ("Tink_1", 1.0),
        _ => return None,
    })
}

pub(crate) fn break_tile(app: &mut App, tx: i32, ty: i32) {
    // The three `CanKillTile` refusals (`WorldGen.cs:62219-62326`) gate
    // every break source — the C# `KillTile` entry duplicates them through
    // `CheckTileBreakability` (the 1/2 flavours, `:63537-63545`) and
    // `Chest.DestroyChest` (:63695). Vanilla answers the pick with the fail
    // tick's sound/dust instead of the break; the port has no per-tick
    // path, so the refusal is silent.
    if !edit::can_kill_tile(&app.world, tx, ty, chest_holds_items(app)) {
        return;
    }
    // `AchievementsHelper.NotifyTileDestroyed` + the CustomInt "Pick"
    // counter (BULLDOZER) — the destroyed tile feeds both rows.
    let kind = app
        .world
        .tile(tx, ty)
        .filter(|t| t.active())
        .map(|t| t.kind);
    if let Some(kind) = kind {
        let _ = app
            .achievements
            .notify(achievements::CondKind::TileDestroyed(kind));
        let _ = app
            .achievements
            .notify(achievements::CondKind::CustomInt("Pick", 10000));
    }
    // The hit cell's frames — the C# `tileCache` of `KillTile_GetItemDrops`
    // (`WorldGen.cs:63964`): the per-cell drops read them after the tile is
    // already cleared (the planter-box style `frameY / 18`, the herb style
    // `frameX / 18`).
    let hit = app
        .world
        .tile(tx, ty)
        .filter(|t| t.active())
        .map(|t| HitCell {
            x: tx,
            y: ty,
            kind: t.kind,
            frame_x: t.frame_x,
            frame_y: t.frame_y,
        });
    app.queue_map_ring(tx, ty);
    let Some(kill) = edit::kill_tile(&mut app.world, tx, ty, false) else {
        return;
    };
    // `Chest.DestroyChest` (`Chest.cs:625-645`, called from
    // `CheckTileBreakability2_ShouldTileSurvive`, `WorldGen.cs:62506`/:62514):
    // a broken chest 21/467 (2 wide) or dresser 88 (3 wide) loses its store
    // entry, and the local player browsing it closes SILENTLY (`:640-643` —
    // `chest = -1`, no sound; the range close's 11 never fires). The anchor
    // is the object's top-left cell, the same decode `can_kill_tile` uses
    // (`kill.rs:2640-2641`). S1-P-i43: the port previously relied on
    // `tick_open`'s dead-tile close (sound 11) — the C# closes here, silent.
    if let Some(hit) = &hit {
        let wide: i32 = match hit.kind {
            21 | 467 => 2,
            88 => 3,
            _ => 0,
        };
        if wide > 0 {
            let anchor_x = hit.x - i32::from(hit.frame_x) / 18 % wide;
            let anchor_y = hit.y - i32::from(hit.frame_y) / 18;
            if let Some(idx) = crate::chest::find_chest(&app.world_chests, anchor_x, anchor_y) {
                crate::chest::remove_chest(&mut app.world_chests, idx);
                if let Some(p) = app.player.as_mut() {
                    if p.chest == idx as i32 {
                        p.chest = crate::bank::CHEST_CLOSED; // `:642` — silent
                    }
                }
            }
        }
    }
    // `WorldGen.KillTile_PlaySounds(i, j, fail, tileCache)` (`WorldGen.cs:
    // 66427-66575`) for the hit cell, played before the cascade's loot — the
    // C# calls it at the top of `KillTile` (:63558) and the tile is always a
    // real break here (`fail: false`; the port's refusal path returns above).
    if let Some(hit) = &hit {
        let sounds = edit::kill_tile_sound(hit.kind, hit.frame_x, false);
        let (px, py) = (tx as f32 * TILE_PX, ty as f32 * TILE_PX);
        for cue in [sounds.primary, sounds.extra, sounds.shatter]
            .into_iter()
            .flatten()
        {
            if let Some((name, volume)) = kill_cue(cue) {
                app.audio.play_at(name, px, py, volume);
            }
        }
    }
    // `CheckPot`'s break sound by style (:56856-56867, `pot_break_sound`),
    // at the event cell.
    for event in &kill.events {
        if let Some((name, volume)) = kill_cue((edit::pot_break_sound(event.style), 1)) {
            app.audio.play_at(
                name,
                event.x as f32 * TILE_PX,
                event.y as f32 * TILE_PX,
                volume,
            );
        }
    }
    // The breaking player's staff bonus (`GetPlayerForTile(x, y)
    // .staffOfRegrowthBonus`, `WorldGen.cs:65689` — the flag the placement
    // path sets for held 213/5295): the port reads the held slot at the
    // break moment.
    let staff_bonus = staff_of_regrowth(app.inventory[app.selected].kind);
    // `Player.FindClosest` (`WorldGen.cs:66255-66256`) picks the player
    // nearest the tree's ground cell and reads their held item's axe power —
    // the bonus-wood roll's threshold. The port has one player.
    let axe_power = registry::get(app.inventory[app.selected].kind)
        .map(|d| i32::from(d.axe))
        .unwrap_or(0);
    let ctx = drop_ctx(app);
    // `KillTile` runs from the player's tile use — `Player.ItemCheck` under
    // the `UpdatePlayers` swap (`Main.cs:17754`); the break's `Main.rand`
    // draws ride `stream(UpdatePlayers)`.
    let (drops, portals) = {
        let main_rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        break_drop_events(
            &kill,
            &app.world,
            hit,
            staff_bonus,
            axe_power,
            main_rng,
            &ctx,
        )
    };
    for (id, count, x, y) in drops {
        // The tile-break drops draw the same `UpdatePlayers` stream the
        // break rolls used (`Main.rand` = the active swap, `Main.cs:17754`).
        // No per-drop cue: the C# plays the break sound once, from
        // `KillTile_PlaySounds` (the table above), never per `Item.NewItem`.
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        spawn_item_drop(&mut app.items, rng, id, count, x, y);
    }
    // `WorldGen.CheckPalmTree`'s canopy re-roll (`:54385-54388`): the palm
    // cells whose cell above is gone take a fresh canopy frame. The roll
    // needs the `genRand` stream, so the core recorded the request and the
    // app resolves it here (one `UpdatePlayers` throw seeds the `FastRandom`
    // stand-in).
    for (px, py) in &kill.palm_frame_rolls {
        let mut gen_stream = tr_core::tile_light::FastRandom::new(
            app.rngs
                .stream(crate::app::RngKey::UpdatePlayers)
                .next() as u64,
        );
        let frame_x = (gen_stream.next_range(7, 10) * 22) as i16;
        if let Some(cell) = app.world.tile_mut(*px, *py) {
            cell.frame_x = frame_x;
        }
    }
    // `TileFrameImportant`'s `case 72` canopy re-roll (`WorldGen.cs:
    // 85855-85873`): the mushroom-tree top cell takes `genRand.Next(3)` →
    // frame (18, 0/18/36). Same seam as the palm roll above.
    for (mx, my) in &kill.mushroom_frame_rolls {
        let mut gen_stream = tr_core::tile_light::FastRandom::new(
            app.rngs
                .stream(crate::app::RngKey::UpdatePlayers)
                .next() as u64,
        );
        let frame_y = (gen_stream.next(3) * 18) as i16;
        if let Some(cell) = app.world.tile_mut(*mx, *my) {
            cell.frame_x = 18;
            cell.frame_y = frame_y;
        }
    }
    // The Coin Portal projectile of the pot loot (:57176-57182); the port
    // has no 518 family yet — `spawn_with_data` declines it and the break
    // still returns like the C# (recorded deviation, phase 5).
    for (px, py) in portals {
        let _ = app.projectiles.spawn_with_data(
            518,
            px as f32 * TILE_PX + 16.0,
            py as f32 * TILE_PX + 16.0,
            0.0,
            -12.0,
            0,
            0.0,
        );
    }
    app.request_edit(tx, ty, 0, 0);
    app.liquids.wake_around(&app.world, tx, ty, 3);
    // `PressurePlateHelper.DestroyPlate` (`WorldGen.cs:65327`, case 428 of
    // `KillTile_GetItemDrops`): a pressed weighted plate that is broken fires
    // its switch once so the device releases. The pick break runs in
    // `Player.ItemCheck` under the `UpdatePlayers` swap (`Main.cs:17754`), so
    // the click's pitch draw rides that stream (the `HitSwitch` precedent,
    // `doors.rs`). An unpressed plate is a no-op.
    if hit.as_ref().is_some_and(|h| h.kind == 428) {
        app.wiring.destroy_plate(&mut app.world, tx, ty);
        for (px, py) in app.wiring.take_switch_clicks() {
            app.audio.play_switch_click(
                px as f32,
                py as f32,
                app.rngs.stream(crate::app::RngKey::UpdatePlayers),
            );
        }
    }
    // `WorldGen.TileFrame`'s falling check (`WorldGen.cs:81380`): the tile
    // above the break loses its support and the falling kind spawns the
    // sandfall projectile (`SpawnFallingBlockProjectile`, `:86039-86090`).
    if let Some(fall) = tr_core::falling::check_falling(&app.world, tx, ty - 1) {
        if let Some(t) = app.world.tile_mut(tx, ty - 1) {
            *t = Default::default();
        }
        let _ = app.projectiles.spawn_falling(
            fall.tile_kind,
            fall.proj_kind,
            fall.damage,
            fall.x,
            fall.y,
        );
    }
    app.world_key = None;
}

/// The world/player inputs the drop resolvers read. Every C# flag with no
/// port state defaults to `false` (`Main.expertMode/dontStarveWorld` — the
/// world modes are not persisted yet; remix/getGood/tenth/vampire never reach
/// the resolvers). Built once per break from `App` ([`drop_ctx`]); the
/// `Default` zeroes serve the record-only test paths.
#[derive(Default)]
pub(crate) struct DropCtx {
    /// `Main.dayTime/moonPhase/bloodMoon/time` (`World`) and
    /// `Main.raining/cloudAlpha` (`Rain`).
    pub day_time: bool,
    pub moon_phase: u8,
    pub blood_moon: bool,
    pub time: f64,
    pub raining: bool,
    pub cloud_alpha: f32,
    /// `Main.worldSurface/rockLayer/UnderworldLayer/maxTilesY/hardMode`
    /// (`Main.cs:2790`: `UnderworldLayer = maxTilesY - 200`).
    pub world_surface: f64,
    pub rock_layer: f64,
    pub underworld_layer: i32,
    pub max_tiles_y: i32,
    pub hard_mode: bool,
    /// `Main.expertMode` / `Main.dontStarveWorld` — no port state (tests flip
    /// them; ANALYSIS entry, phase 5).
    pub expert_mode: bool,
    pub dont_starve: bool,
    /// M5-SEED-2: `Main.vampireSeed` — `InitializeSecretSeeds` sets it for the
    /// `vampirism` micro seed (`WorldGen.cs:555-558`), and
    /// `SpawnThingsFromPot`'s four arms read it (`:57477`/`:57494`/`:57518`/
    /// `:57539`). A classic world keeps it false, so the pot rolls are
    /// unchanged.
    pub vampirism: bool,
    /// The closest player (`Player.FindClosest` — the local one): the hearts
    /// gate `statLife < statLifeMax2` (:57512) and the torch-stack scan
    /// (:57482-57497, capped at the 20/30 threshold the C# loop breaks at).
    /// No player (headless shots) reads full life + 0 torches.
    pub player_hurt: bool,
    pub torches: i32,
    /// `player2.Zone*` — the scene metrics refresh at the player centre;
    /// `ZoneGlowshroom` has no port metric (false, phase-5 ANALYSIS).
    pub zone_hallow: bool,
    pub zone_crimson: bool,
    pub zone_corrupt: bool,
    pub zone_glowshroom: bool,
}

/// Builds [`DropCtx`] from the app state at the break moment.
fn drop_ctx(app: &App) -> DropCtx {
    let (player_hurt, torches) = match &app.player {
        Some(p) => (
            p.hp < p.max_hp,
            app.inventory
                .iter()
                .filter(|slot| {
                    slot.count > 0
                        && registry::place_tile(slot.kind)
                            .is_some_and(|tile| td::SET_TORCHES.has(tile))
                })
                .map(|slot| slot.count as i32)
                .sum::<i32>()
                .min(20),
        ),
        None => (false, 0),
    };
    DropCtx {
        day_time: app.world.day_time,
        moon_phase: app.world.moon_phase,
        blood_moon: app.world.blood_moon,
        time: app.world.time,
        raining: app.rain.is_raining(),
        cloud_alpha: app.rain.visual_alpha(),
        world_surface: app.world.world_surface,
        rock_layer: app.world.rock_layer,
        underworld_layer: app.world.height - 200,
        max_tiles_y: app.world.height,
        hard_mode: app.world.hard_mode,
        expert_mode: false,
        // M5-SEED-1: `Main.dontStarveWorld` (`editing.rs`'s fruit/pot
        // branches, `:1437-1474`) — the world's own flag replaces the
        // hard-coded false.
        dont_starve: app
            .world
            .seeds
            .contains(tr_core::seeds::SeedFlags::DONT_STARVE),
        // M5-SEED-2: `Main.vampireSeed` (`WorldGen.cs:555-558`'s
        // `SecretSeed.vampirism.Enabled` write) — `SpawnThingsFromPot`'s
        // `:57477`/`:57494`/`:57518`/`:57539` arms.
        vampirism: app
            .world
            .micro_seeds
            .contains(tr_core::seeds::MicroSeedFlags::VAMPIRISM),
        player_hurt,
        torches,
        zone_hallow: app.scene_metrics.zone_hallow,
        zone_crimson: app.scene_metrics.zone_crimson,
        zone_corrupt: app.scene_metrics.zone_corrupt,
        zone_glowshroom: false,
    }
}

/// The hit cell's drop-relevant state, captured before the kill clears the
/// tile (the C# `tileCache` of `KillTile_GetItemDrops`, `WorldGen.cs:63964`).
#[derive(Clone, Copy)]
struct HitCell {
    x: i32,
    y: i32,
    kind: u16,
    frame_x: i16,
    frame_y: i16,
}

/// The item drops of one `kill_tile` result: the hit cell's per-cell
/// `KillTile_DropItems` rolls (`WorldGen.cs:63860-63864`), the `Check*`
/// `Item.NewItem` records (:86508/:49424/:43494/:47121/:57972), every pot
/// event's `SpawnThingsFromPot` loot (:57011-57014), every cascade-killed
/// herb's rolls (:65674-65698) and every 2x1 pile's loot block
/// (:47130-47245). `(item, stack, x, y)` in spawn order, plus the Coin
/// Portal projectile requests. Free of `App` — the RNG streams come in as
/// parameters (`genRand` = the seeded `FastRandom` stand-in, `Main.rand` =
/// the caller's `UnifiedRandom` stage stream).
fn break_drop_events(
    kill: &edit::TileKill,
    world: &World,
    hit: Option<HitCell>,
    staff_of_regrowth: bool,
    axe_power: i32,
    main_rng: &mut UnifiedRandom,
    ctx: &DropCtx,
) -> (Vec<(u16, u32, i32, i32)>, Vec<(i32, i32)>) {
    let mut events = Vec::new();
    // The hit cell's per-cell drop (`KillTile_DropItems` for the dying cell —
    // the C# `tileCache` carries the frames; captured pre-kill in
    // [`break_tile`]).
    if let Some(hit) = &hit {
        events.extend(cell_drops(hit, staff_of_regrowth, main_rng, ctx));
    }
    // The cascade's `Check*` records: the check's own `Item.NewItem` (the
    // sunflower's 63) or the object's item through `registry::tile_drop`.
    for drop in &kill.object_drops {
        let id = drop
            .item
            .map(|item| item as u16)
            .or_else(|| registry::tile_drop(drop.tile_kind, 0));
        if let Some(id) = id {
            events.push((id, 1, drop.x, drop.y));
        }
    }
    // The killed tree cells' wood and acorns (`WorldGen.KillTile_GetTreeDrops`,
    // `WorldGen.cs:66189-66261`, and the palm arm of `KillTile_GetItemDrops`,
    // `:65232-65266`).
    for event in &kill.tree_events {
        match event {
            edit::TreeDropEvent::Palm {
                x,
                y,
                frame_x,
                item,
            } => {
                if *item != 0 {
                    events.push((*item, 1, *x, *y));
                }
                // `if (tileCache.frameX <= 132 && tileCache.frameX >= 88)
                // secondaryItem = 27;` (`:65239-65242`).
                if (88..=132).contains(frame_x) {
                    events.push((27, 1, *x, *y));
                }
            }
            edit::TreeDropEvent::Trunk {
                x,
                y,
                frame_x,
                frame_y,
                tree_type,
                ..
            } => {
                let mut gen_stream =
                    tr_core::tile_light::FastRandom::new(main_rng.next() as u64);
                for (id, count) in trunk_tree_drops(
                    *tree_type,
                    *frame_x,
                    *frame_y,
                    axe_power,
                    &mut gen_stream,
                    main_rng,
                ) {
                    events.push((id, count, *x, *y));
                }
            }
            edit::TreeDropEvent::Gem {
                x,
                y,
                frame_x,
                frame_y,
                gem,
                seed,
            } => {
                for (id, count) in
                    gem_tree_drops(*gem, *seed, *frame_x, *frame_y, main_rng)
                {
                    events.push((id, count, *x, *y));
                }
            }
            edit::TreeDropEvent::Mushroom { x, y } => {
                for (id, count) in mushroom_tree_drops(main_rng) {
                    events.push((id, count, *x, *y));
                }
            }
            edit::TreeDropEvent::Cactus { x, y } => {
                // `case 80: dropItem = 276;` (`WorldGen.cs:64717-64719`).
                events.push((276, 1, *x, *y));
            }
        }
    }
    // `SpawnThingsFromPot` per pot event: one `UpdatePlayers` throw seeds the
    // `genRand` stand-in (the owner's contract), `Main.rand` rolls consume
    // the stage stream directly, in the C# order. The Coin Portal roll returns a
    // projectile request instead of loot.
    let mut portals = Vec::new();
    for event in &kill.events {
        let cell = world.tile(event.x, event.y);
        let liquid = cell.map(|t| t.liquid).unwrap_or(0);
        let wall = cell.map(|t| t.wall).unwrap_or(0);
        let mut gen_stream =
            tr_core::tile_light::FastRandom::new(main_rng.next() as u64);
        let (loot, portal) = spawn_things_from_pot(
            ctx,
            event.style,
            event.x,
            event.y,
            liquid,
            wall,
            &mut gen_stream,
            main_rng,
        );
        if portal {
            // `Projectile.NewProjectile(…, i * 16 + 16, j * 16 + 16, 0f,
            // -12f, 518, 0, 0f, Main.myPlayer)` (:57176-57182).
            portals.push((event.x, event.y));
        }
        for (id, count) in loot {
            events.push((id, count.max(1) as u32, event.x, event.y));
        }
    }
    // The cascade-killed herb cells: the per-cell herb rolls (`Main.rand`
    // only — no `genRand` rolls, no seed throw).
    for herb in &kill.herb_events {
        for (id, count) in herb_drops(
            herb.kind,
            herb.style,
            herb.y,
            staff_of_regrowth,
            ctx,
            main_rng,
        ) {
            events.push((id, count.max(1) as u32, herb.x, herb.y));
        }
    }
    // The broken piles' loot blocks: 185's 2x1 coin/ore table on the seeded
    // stream (`genRand` counts, `WorldGen.cs:47142-47245`), 187's
    // sword-in-stone roll on the item stream (`Main.rand.Next(30)`,
    // `:49828-49839`).
    for pile in &kill.pile_events {
        if pile.kind == 187 {
            if let Some(id) = sword_pile_roll(pile.frame_x, main_rng) {
                events.push((id, 1, pile.x, pile.y));
            }
            continue;
        }
        let mut gen_stream =
            tr_core::tile_light::FastRandom::new(main_rng.next() as u64);
        for (id, count) in pile_loot(pile.frame_x, &mut gen_stream) {
            events.push((id, count.max(1) as u32, pile.x, pile.y));
        }
    }
    (events, portals)
}

/// `WorldGen.KillTile_GetTreeDrops` (`WorldGen.cs:66189-66261`) — the killed
/// trunk cell's wood and the canopy marker's acorn.
///
/// The C# rolls in this order: the acorn roll (`genRand.Next(2)`, the canopy
/// marker cell only — `frameX >= 22 && frameY >= 198`), the mushroom roll
/// (`genRand.Next(2)`), the bonus wood (`genRand.Next(35) <= axe`) and its
/// second chance (`Main.rand.Next(3) == 0`, short-circuited). The port's
/// seam: one `UpdatePlayers` throw seeds the `FastRandom` that stands in for
/// the C#'s `genRand` stream, and the `Main.rand` roll spends the stage
/// stream directly — the house convention (PORT_MAP's "the RNG seam").
fn trunk_tree_drops(
    tree_type: tr_core::tree::TreeType,
    frame_x: i16,
    frame_y: i16,
    axe_power: i32,
    gen_stream: &mut tr_core::tile_light::FastRandom,
    main_rng: &mut UnifiedRandom,
) -> Vec<(u16, u32)> {
    let mut drop_item = 0u16;
    let mut secondary_item = 0u16;
    // The canopy marker's acorn roll (`:66191-66212`).
    if frame_x >= 22 && frame_y >= 198 {
        if gen_stream.next(2) == 0 {
            drop_item = 9;
            if tr_core::tree::tree_type_drops_acorns(tree_type) {
                secondary_item = 27;
            }
        } else {
            drop_item = 9;
        }
    } else {
        drop_item = 9;
    }
    if drop_item != 9 {
        return Vec::new();
    }
    // The family's wood (`:66224-66254`).
    match tree_type {
        tr_core::tree::TreeType::Ash => drop_item = 5215,
        tr_core::tree::TreeType::Corrupt => drop_item = 619,
        tr_core::tree::TreeType::Crimson => drop_item = 911,
        tr_core::tree::TreeType::Jungle => drop_item = 620,
        tr_core::tree::TreeType::Hallowed => drop_item = 621,
        tr_core::tree::TreeType::Snow => drop_item = 2503,
        // The glowing mushroom drops the mushroom itself half the time and
        // nothing the other half (`:66241-66250`).
        tr_core::tree::TreeType::Mushroom => {
            drop_item = if gen_stream.next(2) == 0 { 183 } else { 0 };
        }
        _ => {}
    }
    // The bonus wood (`:66255-66260`): the held axe's power against a 1-in-35
    // roll, with a 1-in-3 roll as the second chance.
    let bonus = gen_stream.next(35) <= axe_power || main_rng.next_range(0, 3) == 0;
    let stack = if bonus { 2u32 } else { 1u32 };
    let mut drops = Vec::new();
    if drop_item != 0 {
        drops.push((drop_item, stack));
    }
    if secondary_item != 0 {
        drops.push((secondary_item, 1));
    }
    drops
}

/// `KillTile_GetItemDrops`'s gem-tree cases (`WorldGen.cs:65699-65746`)
/// through `SetGemTreeDrops` (`:66094-66108`), all `Main.rand`: the
/// `Next(10) == 0` gem-or-stone roll, then the canopy marker's
/// (`frameX >= 22 && frameY >= 198`) `Next(2) == 0` gemcorn roll, then — in
/// the case body AFTER `SetGemTreeDrops` returns — the stone's
/// `Next(1, 3)` stack. The seed roll runs even when the gem roll landed
/// stone (it sits after the gem/stone choice, before the stack roll).
fn gem_tree_drops(
    gem: u16,
    seed: u16,
    frame_x: i16,
    frame_y: i16,
    main_rng: &mut UnifiedRandom,
) -> Vec<(u16, u32)> {
    let drop_item = if main_rng.next_max(10) == 0 { gem } else { 3 };
    let mut secondary_item = 0u16;
    if frame_x >= 22 && frame_y >= 198 && main_rng.next_max(2) == 0 {
        secondary_item = seed;
    }
    let mut stack = 1u32;
    if drop_item == 3 {
        stack = main_rng.next_range(1, 3) as u32;
    }
    let mut drops = Vec::new();
    if drop_item != 0 {
        drops.push((drop_item, stack));
    }
    if secondary_item != 0 {
        drops.push((secondary_item, 1));
    }
    drops
}

/// `KillTile_GetItemDrops`'s `case 71: case 72:` (`WorldGen.cs:65642-65652`),
/// `Main.rand` only: `Next(40) == 0` → the Mushroom Grass Seeds 194, else
/// `Next(2) == 0` → the Glowing Mushroom 183, else nothing. The second roll
/// runs whenever the first misses (the C# `else if`).
fn mushroom_tree_drops(main_rng: &mut UnifiedRandom) -> Vec<(u16, u32)> {
    if main_rng.next_max(40) == 0 {
        vec![(194, 1)]
    } else if main_rng.next_max(2) == 0 {
        vec![(183, 1)]
    } else {
        Vec::new()
    }
}

/// `Check3x2`'s `case 187` (`WorldGen.cs:49828-49839`): only the
/// sword-in-stone styles (origin `frameX` 918..=970) drop anything —
/// `Main.rand.Next(30) == 0` mints the Terragrim 4144, otherwise the
/// Enchanted Sword 989. Every other 187 frame sits in the empty
/// `GetItemDrops` group (:65759-65815).
fn sword_pile_roll(frame_x: i32, rng: &mut UnifiedRandom) -> Option<u16> {
    if !(918..=970).contains(&frame_x) {
        return None;
    }
    Some(if rng.next_range(0, 30) == 0 {
        4144
    } else {
        989
    })
}

/// The per-cell slice of `KillTile_GetItemDrops` (`WorldGen.cs:63964-66092`)
/// for the dying cell: 78 → 222 (:64711-64713), 380 → `3215 + frameY / 18`
/// (:65211-65213), the herb stages 83/84 → the RNG rolls (:65674-65698);
/// 27/28/82 and the piles are the empty case group (:65759-65815) — their
/// loot is the cascade record / the pot event — and registered furniture
/// drops through its `Check*` record only (no per-cell double spawn). The
/// unregistered legacy kinds keep the `registry::tile_drop` fallback (dirt →
/// 2, stone → 3, the cactus arm).
fn cell_drops(
    hit: &HitCell,
    staff_of_regrowth: bool,
    main_rng: &mut UnifiedRandom,
    ctx: &DropCtx,
) -> Vec<(u16, u32, i32, i32)> {
    let (x, y) = (hit.x, hit.y);
    let rolls: Vec<(u16, i32)> = match hit.kind {
        78 => vec![(222, 1)],
        380 => vec![((3215 + i32::from(hit.frame_y) / 18) as u16, 1)],
        // S1-P-i30: the tree/cactus kinds drop through their per-cell kill
        // records (`TreeDropEvent`, resolved in `break_drop_events`) — the
        // hit cell included — so the generic per-cell arm must not double
        // them. The trunk/palm kinds already resolve to `None` through
        // `registry::tile_drop`; the gem trees 583-589, the mushroom tree 72
        // and the cactus 80 (whose `tile_drop` row feeds the touch/plate
        // lanes) are suppressed here (the records carry their drops).
        5 | 72 | 80 | 323 | 583..=589 | 596 | 616 | 634 => Vec::new(),
        83 | 84 => herb_drops(
            hit.kind,
            i32::from(hit.frame_x) / 18,
            y,
            staff_of_regrowth,
            ctx,
            main_rng,
        ),
        // S1-F-7: the gate is the C# `KillTile_GetItemDrops` empty case group
        // (`WorldGen.cs:63988-66090`), generated as `KILL_TILE_NO_CELL_DROP` —
        // NOT mere registration. The wooden slice used `def_for_tile(...).is_some()`,
        // which only matched because every registered kind happened to be in the
        // empty group; the full registry registers torches/platforms/orbs/plants
        // that DO drop per-cell, so the gate must read the actual C# group.
        _ if tr_core::tileobject::kill_tile_no_cell_drop(hit.kind) => Vec::new(),
        _ => registry::tile_drop(hit.kind, hit.frame_y)
            .into_iter()
            .map(|id| (id, 1))
            .collect(),
    };
    rolls
        .into_iter()
        .map(|(id, count)| (id, count.max(1) as u32, x, y))
        .collect()
}

/// `IsAlchemyPlantHarvestable` (`WorldGen.cs:66143-66174`): the per-style
/// bloom window — 0 Daybloom by day, 1 Moonglow by night, 3 Deathweed on a
/// blood moon (or the new moon), 4 Waterleaf in the rain (`Main.cloudAlpha`
/// included), 5 Fireblossom past 40500 ticks unless it rains while shallow.
/// The `remixWorld` branch has no port state (skipped — ANALYSIS, phase 5).
fn is_alchemy_plant_harvestable(style: i32, y: i32, ctx: &DropCtx) -> bool {
    if style == 0 && ctx.day_time {
        return true;
    }
    if style == 1 && !ctx.day_time {
        return true;
    }
    if style == 3 && !ctx.day_time && (ctx.blood_moon || ctx.moon_phase == 0) {
        return true;
    }
    if style == 4 && (ctx.raining || ctx.cloud_alpha > 0.0) {
        return true;
    }
    if style == 5 {
        let deep = y as f64 > ctx.world_surface;
        // `(!Main.raining | flag) && Main.time > 40500.0` — the C# bitwise
        // `|` on bools; both operands are side-effect free.
        if (!ctx.raining || deep) && ctx.time > 40500.0 {
            return true;
        }
    }
    false
}

/// `IsHarvestableHerbWithSeed` (`WorldGen.cs:66118-66141`): the immature 82
/// never seeds; 84 always; 83 only inside its bloom window.
fn is_harvestable_herb_with_seed(kind: u16, style: i32, y: i32, ctx: &DropCtx) -> bool {
    match kind {
        84 => true,
        83 => is_alchemy_plant_harvestable(style, y, ctx),
        _ => false,
    }
}

/// The per-cell herb rolls of `KillTile_GetItemDrops`'s `case 83: case 84:`
/// arm (`WorldGen.cs:65674-65698`): the herb `313 + num` (6 → 2358) always
/// drops (stack 1, `1..3` with the Staff of Regrowth); the seeds `307 + num`
/// (6 → 2357) drop `1..6` with the staff, else `1..4` when
/// `IsHarvestableHerbWithSeed`, else never. All rolls are `Main.rand`
/// (`Main.cs` drops carry no `genRand` rolls) — the `UpdatePlayers` stream.
fn herb_drops(
    kind: u16,
    style: i32,
    y: i32,
    staff_of_regrowth: bool,
    ctx: &DropCtx,
    main_rng: &mut UnifiedRandom,
) -> Vec<(u16, i32)> {
    // The case labels are 83/84 only — the immature 82 sits in the empty
    // case group (:65759-65815) and drops nothing.
    if kind == 82 {
        return Vec::new();
    }
    let (mut herb_id, mut seed_id) = (313 + style, 307 + style);
    if style == 6 {
        herb_id = 2358;
        seed_id = 2357;
    }
    let harvestable = is_harvestable_herb_with_seed(kind, style, y, ctx);
    let mut drops = Vec::with_capacity(2);
    if staff_of_regrowth {
        // `dropItemStack = Main.rand.Next(1, 3); secondaryItemStack =
        // Main.rand.Next(1, 6);` — the roll order is the C#'s.
        let stack = main_rng.next_range(1, 3);
        drops.push((herb_id as u16, stack));
        let seeds = main_rng.next_range(1, 6);
        drops.push((seed_id as u16, seeds));
    } else if harvestable {
        drops.push((herb_id as u16, 1));
        let seeds = main_rng.next_range(1, 4);
        drops.push((seed_id as u16, seeds));
    } else {
        drops.push((herb_id as u16, 1));
    }
    drops
}

/// `WorldGen.GetFruitForPot` (`WorldGen.cs:57018-57087`): the dontStarve
/// fruit pairs by pot style, all on the `genRand` stream. (The `InWorld`
/// head guard never binds — the pot cell is inside the cascade's 5-tile
/// margin.)
fn get_fruit_for_pot(pot_style: i32, gen_rand: &mut tr_core::tile_light::FastRandom) -> u16 {
    let pick = |gen_rand: &mut tr_core::tile_light::FastRandom, a: u16, b: u16| {
        if gen_rand.next_range(0, 2) == 0 { a } else { b }
    };
    if (4..=6).contains(&pot_style) {
        return pick(gen_rand, 4286, 4295);
    }
    if (7..=9).contains(&pot_style) || (28..=30).contains(&pot_style) {
        return pick(gen_rand, 4294, 4292);
    }
    if ((25..=27).contains(&pot_style)) || ((34..=36).contains(&pot_style)) {
        return pick(gen_rand, 4283, 4287);
    }
    if (16..=18).contains(&pot_style) {
        return pick(gen_rand, 4284, 4289);
    }
    if (22..=24).contains(&pot_style) {
        return pick(gen_rand, 4296, 4285);
    }
    if (13..=15).contains(&pot_style) {
        return pick(gen_rand, 5277, 5278);
    }
    match gen_rand.next_range(0, 5) {
        0 => 4009,
        1 => 4293,
        2 => 4282,
        3 => 4290,
        _ => 4291,
    }
}

/// The style multiplier table of `SpawnThingsFromPot`'s head
/// (`WorldGen.cs:57099-57159`): 0-3 neutral, then the per-band factors (the
/// hardmode band 28-30 only raises it in hardmode), folded by
/// `num = (num * 2 + 1) / 3` (:57159).
fn pot_style_multiplier(style: i32, hard_mode: bool) -> f32 {
    let mut num: f32 = 1.0;
    if (4..=6).contains(&style) {
        num = 1.25;
    } else if (7..=9).contains(&style) {
        num = 1.75;
    } else if (10..=12).contains(&style) {
        num = 1.9;
    } else if (13..=15).contains(&style) {
        num = 2.1;
    } else if (16..=18).contains(&style) {
        num = 1.6;
    } else if (19..=21).contains(&style) {
        num = 3.5;
    } else if (22..=24).contains(&style) {
        num = 1.6;
    } else if (25..=27).contains(&style) {
        num = 10.0;
    } else if (28..=30).contains(&style) && hard_mode {
        num = 4.0;
    } else if (31..=33).contains(&style) {
        num = 2.0;
    } else if (34..=36).contains(&style) {
        num = 1.25;
    }
    (num * 2.0 + 1.0) / 3.0
}

/// `int range = (int)(500f / ((num + 1f) / 2f))` (`WorldGen.cs:57171`) — the
/// Coin Portal's luck range (`Player.GetClosestRollLuck`, :57176).
fn pot_luck_range(num: f32) -> i32 {
    (500.0f32 / ((num + 1.0) / 2.0)) as i32
}

/// `WorldGen.SpawnThingsFromPot` (`WorldGen.cs:57089-57811`), the full roll
/// order. `gen_rand` is the event's `genRand` stand-in (`FastRandom` seeded
/// from one `UpdatePlayers` throw, the owner's contract), `main_rng` the
/// `Main.rand` stream (the caller's stage stream). `liquid`/`wall` are the pot
/// origin cell's post-kill state (KillTile keeps liquid and wall) for the
/// glowstick branch and the dungeon key. Returns the loot plus the Coin
/// Portal flag (the caller spawns the 518 projectile).
///
/// The special-world branches skip WITHOUT consuming rolls (C# `&&`
/// short-circuits): the Beenade (tenth anniversary :57167), the dontStarve
/// fruit (behind `ctx.dont_starve`), the getGoodWorld bomb, every
/// remixWorld branch and the netMode-2 wormhole. The boss-downed `×1.1`
/// flags (:57706-57750) have no port state — all false.
fn spawn_things_from_pot(
    ctx: &DropCtx,
    style: i32,
    _x: i32,
    y: i32,
    liquid: u8,
    wall: u16,
    gen_rand: &mut tr_core::tile_light::FastRandom,
    main_rng: &mut UnifiedRandom,
) -> (Vec<(u16, i32)>, bool) {
    // :57090-57095 — the depth flags (`flag` caves, `flag2` above the
    // underworld; the remix rewrites have no port state).
    let flag = (y as f64) < ctx.rock_layer;
    let flag2 = y < ctx.underworld_layer;
    let flag4 = (34..=36).contains(&style);
    // :57099-57171 — the style multiplier and the luck range.
    let range = pot_luck_range(pot_style_multiplier(style, ctx.hard_mode));
    // :57176-57182 — the Coin Portal (`Player.GetClosestRollLuck` →
    // `Luck.RollLuck(luck, range)` with the port's luck 0 = one
    // `Main.rand.Next(range)`, `Luck.cs:5-12`, `Player.cs:18236-18239`);
    // the caller spawns the projectile (`NewProjectile(…, 518, …)`).
    if main_rng.next_range(0, range) == 0 {
        return (Vec::new(), true);
    }
    // :57184-57182 — the dungeon key: `genRand.Next(35)` consumes first, the
    // dungeon-wall/depth checks judge after.
    if gen_rand.next_range(0, 35) == 0
        && tr_core::scene::is_dungeon_wall(wall)
        && y as f64 > ctx.world_surface
    {
        return (vec![(327, 1)], false);
    }
    // :57189-57192 — the dontStarve fruit (`flag3` = below surface +
    // `Main.dontStarveWorld`).
    if ctx.dont_starve && y as f64 > ctx.world_surface && gen_rand.next_range(0, 20) == 0 {
        let stack = gen_rand.next_range(1, 3);
        return (vec![(get_fruit_for_pot(style, gen_rand), stack)], false);
    }
    // :57194-57199 (getGoodWorld bomb), :57200-57241 (remix slimes/stars),
    // :57243-57247 (remix center rope) — skipped, no port state.
    // :57248-57462 — the potion table: `genRand.Next(45) == 0 || (
    // Main.rand.Next(45) == 0 && expertMode)` — the `Main.rand` roll is
    // consumed whenever the `genRand` roll misses, expert or not.
    if gen_rand.next_range(0, 45) == 0 || (main_rng.next_range(0, 45) == 0 && ctx.expert_mode) {
        return (potion_table(ctx, y, flag, flag2, gen_rand), false);
    }
    // :57464-57466 — the netMode-2 wormhole potion: skipped (single player).
    // :57469-57475 — `num11 = Main.rand.Next(7)`, expert shifts it down one
    // (no roll), and the full-torch inventory scan only flips `flag5`.
    let mut num11 = main_rng.next_range(0, 7);
    if ctx.expert_mode {
        num11 -= 1;
    }
    // :57475-57492 — the 50-slot torch-stack scan: `num13 = 20` (30 under
    // `Main.vampireSeed`, `:57477-57481`) is the loop's break threshold, so
    // the vampirism world needs 50 % more torches in the inventory before
    // `flag5` clears.
    let torch_threshold = if ctx.vampirism { 30 } else { 20 };
    let flag5 = ctx.torches < torch_threshold;
    // :57494-57496 — `if (flag5 && Main.vampireSeed) num11 = 1;`: a torchless
    // player in a vampirism world is FORCED into the torch arm below, past
    // the hearts. The roll has already been drawn, so the classic stream is
    // untouched (the C#'s guard short-circuits on a classic world).
    if flag5 && ctx.vampirism {
        num11 = 1;
    }
    // :57498-57516 — the hearts (only while hurt).
    if num11 == 0 && ctx.player_hurt {
        let mut drops = vec![(58, 1)];
        if main_rng.next_range(0, 2) == 0 {
            drops.push((58, 1));
        }
        if ctx.expert_mode {
            if main_rng.next_range(0, 2) == 0 {
                drops.push((58, 1));
            }
            if main_rng.next_range(0, 2) == 0 {
                drops.push((58, 1));
            }
        }
        return (drops, false);
    }
    // M5-SEED-2: `:57518-57523` — the vampire-seed wood. `if
    // (Main.vampireSeed && (flag2 | flag) && WorldGen.genRand.Next(6) == 0)`
    // → the pot's style wood (9 plain, 2503 for the 4-6 pots, 620 for the
    // 7-9 ones) × `Main.rand.Next(10, 31)`. `flag` is "below the rock layer"
    // and `flag2` "above the underworld layer" (`:57090-57095`), so the arm
    // fires for any cave-or-shallow cell and SKIPS (without its `genRand`
    // draw) in the deep-underworld band. The `genRand.Next(6)` sits after
    // the hearts arm, exactly as the C#.
    if ctx.vampirism && (flag2 || flag) && gen_rand.next_range(0, 6) == 0 {
        let wood = if (4..=6).contains(&style) {
            2503
        } else if (7..=9).contains(&style) {
            620
        } else {
            9
        };
        return (vec![(wood, main_rng.next_range(10, 31))], false);
    }
    // :57531-57591 — the torches (num11 == 1, or 0 while the player is below
    // the torch threshold).
    if num11 == 1 || (num11 == 0 && flag5) {
        return (vec![torch_drop(ctx, style, liquid, main_rng)], false);
    }
    // :57590-57607 — the arrows and the healing potions.
    if num11 == 2 {
        let stack = main_rng.next_range(10, 21);
        let mut arrow = 40;
        if flag && gen_rand.next_range(0, 2) == 0 {
            arrow = if !ctx.hard_mode { 42 } else { 168 };
        }
        if y > ctx.underworld_layer {
            arrow = 265;
        } else if ctx.hard_mode {
            // `Main.rand.Next(2) != 0 ? 47 : (SavedOreTiers.Silver != 168 ?
            // 278 : 4915)` — hardmode ore tiers are not ported.
            arrow = if main_rng.next_range(0, 2) != 0 {
                47
            } else {
                278
            };
        }
        return (vec![(arrow, stack)], false);
    }
    if num11 == 3 {
        let potion = if y > ctx.underworld_layer || ctx.hard_mode {
            188
        } else {
            28
        };
        let mut count = 1;
        if ctx.expert_mode && main_rng.next_range(0, 3) != 0 {
            count += 1;
        }
        return (vec![(potion, count)], false);
    }
    // :57608-57643 — the bombs (styles 34-36 anywhere above the underworld,
    // otherwise the deep cells); a miss falls through to the rope/coins.
    if num11 == 4 && (flag4 || flag2) {
        let bomb = if flag4 { 4423 } else { 166 };
        let mut count = main_rng.next_range(0, 4) + 1;
        if ctx.expert_mode {
            count += main_rng.next_range(0, 4);
        }
        return (vec![(bomb, count)], false);
    }
    // :57644-57647 — the pre-hardmode underground rope.
    if (num11 == 4 || num11 == 5) && y < ctx.underworld_layer && !ctx.hard_mode {
        return (vec![(965, main_rng.next_range(20, 41))], false);
    }
    // :57649-57810 — the coins.
    (
        coin_drops(
            ctx,
            pot_style_multiplier(style, ctx.hard_mode),
            y,
            flag,
            gen_rand,
            main_rng,
        ),
        false,
    )
}

/// The potion table of [`spawn_things_from_pot`] (`WorldGen.cs:57249-57461`):
/// one `genRand` pick per depth band, the longer-than-`else` arms (2322/
/// 2329 rolls) run for every `num >= 7` beside their own item.
fn potion_table(
    ctx: &DropCtx,
    y: i32,
    flag: bool,
    flag2: bool,
    gen_rand: &mut tr_core::tile_light::FastRandom,
) -> Vec<(u16, i32)> {
    let mut drops = Vec::with_capacity(2);
    if (y as f64) < ctx.world_surface {
        // :57250-57285 — the surface band.
        let num7 = gen_rand.next_range(0, 10);
        drops.push((
            match num7 {
                0 => 292,
                1 => 298,
                2 => 299,
                3 => 290,
                4 => 2322,
                5 => 2324,
                6 => 2325,
                _ => 2350,
            },
            1,
        ));
        if num7 >= 7 {
            let stack = gen_rand.next_range(1, 3);
            drops.push((2350, stack));
        }
    } else if flag {
        // :57286-57329 — the cave band.
        let num8 = gen_rand.next_range(0, 11);
        drops.push((
            match num8 {
                0 => 289,
                1 => 298,
                2 => 299,
                3 => 290,
                4 => 303,
                5 => 291,
                6 => 304,
                7 => 2322,
                8 => 2329,
                _ => 2350,
            },
            1,
        ));
        if num8 >= 7 {
            let stack = gen_rand.next_range(1, 3);
            drops.push((2350, stack));
        }
    } else if flag2 {
        // :57330-57397 — above the underworld.
        let num9 = gen_rand.next_range(0, 15);
        drops.push((
            match num9 {
                0 => 296,
                1 => 295,
                2 => 299,
                3 => 302,
                4 => 303,
                5 => 305,
                6 => 301,
                7 => 302,
                8 => 297,
                9 => 304,
                10 => 2322,
                11 => 2323,
                12 => 2327,
                13 => 2329,
                _ => 2350,
            },
            1,
        ));
        if num9 >= 7 {
            let stack = gen_rand.next_range(1, 3);
            drops.push((2350, stack));
        }
        if gen_rand.next_range(0, 15) == 0 {
            drops.push((4870, 1));
        }
    } else {
        // :57398-57461 — the deep band.
        let num10 = gen_rand.next_range(0, 14);
        drops.push((
            match num10 {
                0 => 296,
                1 => 295,
                2 => 293,
                3 => 288,
                4 => 294,
                5 => 297,
                6 => 304,
                7 => 305,
                8 => 301,
                9 => 302,
                10 => 288,
                11 => 300,
                12 => 2323,
                13 => 2326,
                _ => 300,
            },
            1,
        ));
        if gen_rand.next_range(0, 5) == 0 {
            drops.push((4870, 1));
        }
    }
    drops
}

/// The torch drop of [`spawn_things_from_pot`] (`WorldGen.cs:57532-57591`):
/// the base count plus the expert/vampire/biome bonuses — the first true
/// biome/style arm consumes its `Main.rand.Next(2, 7)` — then the liquid
/// swap (glowstick in water, the desert's 286).
fn torch_drop(
    ctx: &DropCtx,
    style: i32,
    liquid: u8,
    main_rng: &mut UnifiedRandom,
) -> (u16, i32) {
    let mut num14 = main_rng.next_range(2, 7);
    if ctx.expert_mode {
        num14 += main_rng.next_range(1, 7);
    }
    // M5-SEED-2: `:57539-57542` — `if (Main.vampireSeed) num14 +=
    // Main.rand.Next(2, 7);`, the fourth stack adder in the C# order (after
    // the base and the expert one, before the per-style adds).
    if ctx.vampirism {
        num14 += main_rng.next_range(2, 7);
    }
    let mut torch = 8;
    let mut glowstick = 282;
    if ctx.zone_hallow {
        num14 += main_rng.next_range(2, 7);
        torch = 4387;
    } else if (22..=24).contains(&style) || ctx.zone_crimson {
        num14 += main_rng.next_range(2, 7);
        torch = 4386;
    } else if (16..=18).contains(&style) || ctx.zone_corrupt {
        num14 += main_rng.next_range(2, 7);
        torch = 4385;
    } else if (7..=9).contains(&style) {
        num14 = ((num14 + main_rng.next_range(2, 7)) as f32 * 1.5) as i32;
        torch = 4388;
    } else if (4..=6).contains(&style) {
        torch = 974;
        glowstick = 286;
    } else if (34..=36).contains(&style) {
        num14 += main_rng.next_range(2, 7);
        torch = 4383;
    } else if ctx.zone_glowshroom {
        num14 += main_rng.next_range(2, 7);
        torch = 5293;
    }
    if liquid > 0 {
        (glowstick, num14)
    } else {
        (torch, num14)
    }
}

/// The coin tail of [`spawn_things_from_pot`] (`WorldGen.cs:57649-57810`):
/// the `genRand` base, the depth/roll/expert/style multipliers and the
/// denomination split. The twelve boss-downed `×1.1` flags have no port
/// state (all false).
fn coin_drops(
    ctx: &DropCtx,
    num: f32,
    y: i32,
    flag: bool,
    gen_rand: &mut tr_core::tile_light::FastRandom,
    main_rng: &mut UnifiedRandom,
) -> Vec<(u16, i32)> {
    let mut num17: f32 = 200.0 + gen_rand.next_range(-100, 101) as f32;
    if (y as f64) < ctx.world_surface {
        num17 *= 0.5;
    } else if flag {
        num17 *= 0.75;
    } else if y > ctx.max_tiles_y - 250 {
        num17 *= 1.25;
    }
    num17 *= 1.0 + main_rng.next_range(-20, 21) as f32 * 0.01;
    if main_rng.next_range(0, 4) == 0 {
        num17 *= 1.0 + main_rng.next_range(5, 11) as f32 * 0.01;
    }
    if main_rng.next_range(0, 8) == 0 {
        num17 *= 1.0 + main_rng.next_range(10, 21) as f32 * 0.01;
    }
    if main_rng.next_range(0, 12) == 0 {
        num17 *= 1.0 + main_rng.next_range(20, 41) as f32 * 0.01;
    }
    if main_rng.next_range(0, 16) == 0 {
        num17 *= 1.0 + main_rng.next_range(40, 81) as f32 * 0.01;
    }
    if main_rng.next_range(0, 20) == 0 {
        num17 *= 1.0 + main_rng.next_range(50, 101) as f32 * 0.01;
    }
    if ctx.expert_mode {
        num17 *= 2.5;
        if main_rng.next_range(0, 2) == 0 {
            num17 *= 1.25;
        }
        if main_rng.next_range(0, 3) == 0 {
            num17 *= 1.5;
        }
        if main_rng.next_range(0, 4) == 0 {
            num17 *= 1.75;
        }
    }
    num17 *= num;
    split_coins(num17, main_rng)
}

/// The denomination split that closes `SpawnThingsFromPot`
/// (`WorldGen.cs:57752-57810`): platinum/gold/silver/copper with the `> 50`
/// shrinks — the gate roll is consumed only above 50 (C# `&&`), the copper's
/// second divisor is `Next(4) + 1`.
fn split_coins(mut num17: f32, main_rng: &mut UnifiedRandom) -> Vec<(u16, i32)> {
    let mut drops = Vec::new();
    while (num17 as i32) > 0 {
        // The >50 shrinks: `if (num > 50 && Main.rand.Next(2) == 0)` — the
        // gate roll is consumed only above 50 (C# `&&`).
        if num17 > 1_000_000.0 {
            let mut n = (num17 / 1_000_000.0) as i32;
            if n > 50 && main_rng.next_range(0, 2) == 0 {
                n /= main_rng.next_range(0, 3) + 1;
            }
            if main_rng.next_range(0, 2) == 0 {
                n /= main_rng.next_range(0, 3) + 1;
            }
            num17 -= (1_000_000 * n) as f32;
            drops.push((74, n));
        } else if num17 > 10_000.0 {
            let mut n = (num17 / 10_000.0) as i32;
            if n > 50 && main_rng.next_range(0, 2) == 0 {
                n /= main_rng.next_range(0, 3) + 1;
            }
            if main_rng.next_range(0, 2) == 0 {
                n /= main_rng.next_range(0, 3) + 1;
            }
            num17 -= (10_000 * n) as f32;
            drops.push((73, n));
        } else if num17 > 100.0 {
            let mut n = (num17 / 100.0) as i32;
            if n > 50 && main_rng.next_range(0, 2) == 0 {
                n /= main_rng.next_range(0, 3) + 1;
            }
            if main_rng.next_range(0, 2) == 0 {
                n /= main_rng.next_range(0, 3) + 1;
            }
            num17 -= (100 * n) as f32;
            drops.push((72, n));
        } else {
            let mut n = num17 as i32;
            if n > 50 && main_rng.next_range(0, 2) == 0 {
                n /= main_rng.next_range(0, 3) + 1;
            }
            if main_rng.next_range(0, 2) == 0 {
                n /= main_rng.next_range(0, 4) + 1;
            }
            if n < 1 {
                n = 1;
            }
            num17 -= n as f32;
            drops.push((71, n));
        }
    }
    drops
}

/// The 2x1 pile loot table of `Check2x1` (`WorldGen.cs:47130-47245`): the
/// coin/ore sub-styles keyed on the origin `frameX`, counts on the `genRand`
/// stream. The `if` ranges are disjoint, so at most one block runs.
fn pile_loot(frame_x: i32, gen_rand: &mut tr_core::tile_light::FastRandom) -> Vec<(u16, i32)> {
    let mut drops = Vec::new();
    if (576..=610).contains(&frame_x) {
        // The copper pile: one guaranteed pair plus the six conditional
        // pairs (:47142-47176).
        drops.push((71, gen_rand.next_range(10, 100)));
        drops.push((71, gen_rand.next_range(20, 100)));
        if gen_rand.next_range(0, 3) != 0 {
            drops.push((71, gen_rand.next_range(10, 100)));
        }
        if gen_rand.next_range(0, 3) != 0 {
            drops.push((71, gen_rand.next_range(20, 100)));
        }
        if gen_rand.next_range(0, 2) == 0 {
            drops.push((71, gen_rand.next_range(10, 100)));
        }
        if gen_rand.next_range(0, 2) == 0 {
            drops.push((71, gen_rand.next_range(20, 100)));
        }
        if gen_rand.next_range(0, 3) == 0 {
            drops.push((71, gen_rand.next_range(10, 100)));
        }
        if gen_rand.next_range(0, 3) == 0 {
            drops.push((71, gen_rand.next_range(20, 100)));
        }
        if gen_rand.next_range(0, 4) == 0 {
            drops.push((71, gen_rand.next_range(10, 100)));
        }
        if gen_rand.next_range(0, 4) == 0 {
            drops.push((71, gen_rand.next_range(20, 100)));
        }
        if gen_rand.next_range(0, 5) == 0 {
            drops.push((71, gen_rand.next_range(10, 100)));
        }
        if gen_rand.next_range(0, 5) == 0 {
            drops.push((71, gen_rand.next_range(20, 100)));
        }
    } else if (612..=646).contains(&frame_x) {
        drops.push((72, gen_rand.next_range(10, 100)));
        if gen_rand.next_range(0, 2) == 0 {
            drops.push((72, gen_rand.next_range(20, 100)));
        }
        if gen_rand.next_range(0, 3) == 0 {
            drops.push((72, gen_rand.next_range(30, 100)));
        }
    } else if (648..=682).contains(&frame_x) {
        drops.push((73, gen_rand.next_range(1, 6)));
        if gen_rand.next_range(0, 2) == 0 {
            drops.push((73, gen_rand.next_range(2, 6)));
        }
        if gen_rand.next_range(0, 3) == 0 {
            drops.push((73, gen_rand.next_range(3, 6)));
        }
    } else if (684..=718).contains(&frame_x) {
        drops.push((181, gen_rand.next_range(1, 4)));
    } else if (720..=754).contains(&frame_x) {
        drops.push((180, gen_rand.next_range(1, 4)));
    } else if (756..=790).contains(&frame_x) {
        drops.push((177, gen_rand.next_range(1, 4)));
    } else if (792..=826).contains(&frame_x) {
        drops.push((179, gen_rand.next_range(1, 4)));
    } else if (828..=862).contains(&frame_x) {
        drops.push((178, gen_rand.next_range(1, 4)));
    } else if (864..=898).contains(&frame_x) {
        drops.push((182, gen_rand.next_range(1, 4)));
    }
    drops
}

/// `Player.Spawn_ForceClearArea` (`Player.cs:37914-37935`): the invalid spawn
/// area's 3×3 strip above the floor — non-top solids die through
/// `WorldGen.KillTile` (sounds + cascade drops, `noItem = false`), and every
/// liquid cell zeroes its liquid (the lava flag with it, `:37928`) then runs
/// `SquareTileFrame`. The drain runs this AFTER the teleport landed
/// (`Shellphone_Spawn` queued it before the position write; the C# clears
/// first — same tick, no probe reads the area in between).
pub(crate) fn spawn_force_clear_area(app: &mut App, floor_x: i32, floor_y: i32) {
    for i in floor_x - 1..floor_x + 2 {
        for j in floor_y - 3..floor_y {
            // `WorldGen.InWorld(i, j)` — the default fluff 0 (`:9688`).
            if i < 0 || j < 0 || i >= app.world.width || j >= app.world.height {
                continue;
            }
            // `:37922-37925` — `nactive() && tileSolid && !tileSolidTop`.
            let kill = app.world.tile(i, j).is_some_and(|t| {
                t.active()
                    && !t.inactive()
                    && td::TILE_SOLID.has(t.kind)
                    && !td::TILE_SOLID_TOP.has(t.kind)
            });
            if kill {
                break_tile(app, i, j);
            }
            // `:37926-37931` — the liquid wipe; `lava(false)` clears ONLY the
            // lava bit (the honey/shimmer flags stay, like the C#).
            if app.world.tile(i, j).is_some_and(|t| t.liquid > 0) {
                if let Some(t) = app.world.tile_mut(i, j) {
                    t.liquid = 0;
                    t.flags &= !tr_core::tile::FLAG_LIQUID_LAVA;
                }
                for drop in tr_core::edit::square_tile_frame_default(&mut app.world, i, j) {
                    let id = drop
                        .item
                        .map(|item| item as u16)
                        .or_else(|| registry::tile_drop(drop.tile_kind, 0));
                    if let Some(id) = id {
                        spawn_item_drop(
                            &mut app.items,
                            app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                            id,
                            1,
                            drop.x,
                            drop.y,
                        );
                    }
                }
                app.request_edit(i, j, 0, 0);
                app.liquids.wake_around(&app.world, i, j, 3);
                app.world_key = None;
            }
        }
    }
}

pub(crate) fn can_place(app: &App, tx: i32, ty: i32) -> bool {
    let Some(t) = app.world.tile(tx, ty) else {
        return false;
    };
    if t.active()
        || item_gate_blocks(
            app.inventory[app.selected].kind,
            app.inventory[app.selected].count,
        )
    {
        return false;
    }
    // `Collision.EmptyTile` (`Collision.cs:1359-1381`) — consulted by
    // `WorldGen.PlaceTile` only for SOLID placements (`WorldGen.cs:59373`:
    // `forced || EmptyTile(i, j) || !tileSolid[num] || …`); a non-solid tile
    // bypasses the overlap entirely (a torch stands inside the player). The
    // cell refuses when any live player OR NPC hitbox intersects it. The
    // player arm keeps the port's 20x42 box; the NPC arm reads each NPC's own
    // `size()` box (`NPC.base.width/height`, the `SetDefaults_ForNetId` scale).
    // The C# `!dead && !ghost` player filters fold to the port's live-player
    // check (no hardcore ghosts / PVP deaths — `player.rs::ghost_like`).
    let held_solid = registry::place_tile(app.inventory[app.selected].kind)
        .is_some_and(|k| td::TILE_SOLID.has(k));
    if held_solid {
        let (x0, y0) = (tx as f32 * TILE_PX, ty as f32 * TILE_PX);
        // Gate merge (i32 ⨉ i41): the player box reads the LIVE
        // `p.width`/`p.height` (the i41 mount PlayerSize threading — a Rat
        // rider's 14×14 body no longer blocks a 20×42 ghost box), and the
        // NPC overlap arm (i32, `Collision.cs:1366-1379`) stays.
        if let Some(p) = &app.player {
            if x0 < p.x + p.width
                && x0 + TILE_PX > p.x
                && y0 < p.y + p.height
                && y0 + TILE_PX > p.y
            {
                return false;
            }
        }
        for n in &app.npcs {
            let (nw, nh) = n.size();
            if x0 < n.x + nw && x0 + TILE_PX > n.x && y0 < n.y + nh && y0 + TILE_PX > n.y {
                return false;
            }
        }
    }
    // S1-P-i42 — the torch family's own support arm
    // (`PlaceThing_Tiles_BlockPlacementForAssortedThings`,
    // `Player.cs:41106-41198`): the TARGET cell's own wall holds the torch
    // (`:41108-41111`) — the four-neighbour scan below never reads it, so a
    // wall-only cell (no adjacent block/wall) refused every wall torch.
    // Without a wall, the support must be `SolidTileNoAttach` below/left/
    // right (`:41186-41197` — solid, NOT noAttach, flat, not actuated; the
    // platform exception rides the below arm only) or a tree-trunk triple /
    // beam on a side. The generic neighbour scan stays for every other kind.
    // (Gate merge i32 ⨉ i42: the torch arm returns early — the i32 default
    // adjacency arm + platform fallback below apply to the non-torch kinds.)
    if registry::place_tile(app.inventory[app.selected].kind) == Some(4) {
        if t.wall > 0 {
            return true;
        }
        let below = app.world.tile(tx, ty + 1);
        let left = app.world.tile(tx - 1, ty);
        let right = app.world.tile(tx + 1, ty);
        // `:41147-41169` — the support types read RAW, then fold to −1 when
        // the cell is not `nactive()` (the target's own halfBrick folds the
        // below type too, `:41148-41151`). The sloped/half-brick supports
        // were already flattened by `flatten_torch_supports` in the mutable
        // route (`Player.cs:41114-41146` → `WorldGen.SlopeTile`), so the type
        // checks below see flat cells exactly like the C#.
        let kind_of = |t: Option<&tr_core::tile::Tile>| -> i32 {
            t.filter(|n| tr_core::tileobject::nactive(n))
                .map_or(-1, |n| n.kind as i32)
        };
        let mut num3 = kind_of(below);
        if t.half_brick() {
            num3 = -1; // `:41148-41151`
        }
        let num4 = kind_of(left);
        let num5 = kind_of(right);
        let tree = kind_of(app.world.tile(tx - 1, ty - 1));
        let tree2 = kind_of(app.world.tile(tx + 1, ty - 1));
        let tree3 = kind_of(app.world.tile(tx - 1, ty + 1));
        let tree4 = kind_of(app.world.tile(tx + 1, ty + 1));
        // `:41186-41188` — below: `tileSolid[num3] && (!tileNoAttach[num3]
        // || Platforms[num3])`.
        if num3 >= 0
            && td::TILE_SOLID.has(num3 as u16)
            && (!tr_core::torch::no_attach_pub(num3 as u16)
                || td::SET_PLATFORMS.has(num3 as u16))
        {
            return true;
        }
        // `:41190-41193` — left: solid-non-noAttach / tree triple / beam.
        if (num4 >= 0 && td::TILE_SOLID.has(num4 as u16) && !tr_core::torch::no_attach_pub(num4 as u16))
            || (tr_core::torch::is_tree_type_pub(num4)
                && tr_core::torch::is_tree_type_pub(tree)
                && tr_core::torch::is_tree_type_pub(tree3))
            || (num4 >= 0 && td::SET_IS_BEAM.has(num4 as u16))
        {
            return true;
        }
        // `:41194-41197` — right: the mirror.
        if (num5 >= 0 && td::TILE_SOLID.has(num5 as u16) && !tr_core::torch::no_attach_pub(num5 as u16))
            || (tr_core::torch::is_tree_type_pub(num5)
                && tr_core::torch::is_tree_type_pub(tree2)
                && tr_core::torch::is_tree_type_pub(tree4))
            || (num5 >= 0 && td::SET_IS_BEAM.has(num5 as u16))
        {
            return true;
        }
        return false;
    }
    // The default adjacency arm of `BlockPlacementForAssortedThings`
    // (`Player.cs:41247-41261`): a cardinal neighbour supports the placement
    // when it is active and (`tileSolid` OR `TileID.Sets.IsBeam` OR `tileRope`
    // OR the minecart track 314), or it carries any wall; failing all four
    // neighbours, the target cell's OWN wall > 0 still accepts (:41257-41260).
    let neighbour_supports = |n: &tr_core::tile::Tile| {
        n.wall != 0
            || (n.active()
                && (td::TILE_SOLID.has(n.kind)
                    || td::SET_IS_BEAM.has(n.kind)
                    || td::TILE_ROPE.has(n.kind)
                    || n.kind == 314))
    };
    if [(0, -1), (0, 1), (-1, 0), (1, 0)]
        .iter()
        .any(|&(dx, dy)| app.world.tile(tx + dx, ty + dy).is_some_and(neighbour_supports))
    {
        return true;
    }
    if t.wall > 0 {
        return true;
    }
    // The platform fallback (`Player.cs:41332-41345`): when the default arm
    // refused a platform-placing item, ANY active tile in the 3x3 around the
    // target accepts (the diagonals included — the platform hangs off a
    // corner). The C# scans `k = x-1..=x+1, l = y-1..=y+1`; the target itself
    // is inactive (checked above), so it contributes nothing.
    let held_kind = registry::place_tile(app.inventory[app.selected].kind);
    if held_kind.is_some_and(|k| td::SET_PLATFORMS.has(k)) {
        for dy in -1..=1 {
            for dx in -1..=1 {
                if app.world.tile(tx + dx, ty + dy).is_some_and(|n| n.active()) {
                    return true;
                }
            }
        }
    }
    false
}

/// The item gate shared by both placement routes (the head of `can_place` and
/// of `use_tile_item`): an empty slot, a weapon or the legacy cactus kind
/// (80, `ItemID.Cactus`) never places. Free of `App` so the decision is
/// unit-testable without the app harness.
fn item_gate_blocks(kind: u16, count: u32) -> bool {
    count == 0 || registry::is_weapon(kind) || kind == 80
}

/// `Player.PlaceThing_Tiles_IsBlockedByLava` (`Player.cs:41489-41504`): a
/// target holding lava refuses a solid `createTile` outright (`tileSolid`),
/// else it refuses when `TileObjectData.CheckLiquidPlacement` says no
/// (`:41498`). A dry target never blocks. Free of `App` for unit witnesses.
fn is_blocked_by_lava(create_tile: u16, place_style: i32, target: &tr_core::tile::Tile) -> bool {
    if target.liquid > 0 && target.liquid_kind() == tr_core::tile::LiquidKind::Lava {
        if td::TILE_SOLID.has(create_tile) {
            return true;
        }
        return !tr_core::tileobject::check_liquid_placement(create_tile, place_style, target);
    }
    false
}

/// The routing decision of `use_tile_item`: the legacy neighbour/overlap
/// pre-checks (`can_place`) gate only kinds that do NOT ride the
/// `TileObject.CanPlace` route — those are judged by the anchors inside the
/// dispatcher (vanilla furniture can sit on platforms or over the player, and
/// `BlockPlacementIfOverPlayers`, `Player.cs:41427`, covers none of the wooden
/// set). S1-F-7: the gate is the C# `CustomPlace` flag with the 82/227/4
/// exclusions (`routes_through_tile_object`), not mere registration — the full
/// registry registers the natural objects (pots/piles/plants/platforms/torches)
/// which keep the legacy pre-checks exactly as vanilla.
///
/// The alch seeds (82/83/84) are the exception: `BlockPlacementForAssortedThings`
/// forces `canPlace = true` for them (`Player.cs:41294-41296`,
/// `if (Main.tileAlch[createTile]) canPlace = true;`) BEFORE the cut rule, so
/// the neighbour pre-check never gates a seed — the assorted block's cut rule
/// and `PlaceAlch`'s soil/liquid rules decide instead.
fn legacy_precheck_applies(kind: u16, style: i32) -> bool {
    !tr_core::tileobject::routes_through_tile_object(kind, style)
        && !tr_core::tileobject::tile_alch(kind)
}

/// What `TileObject.CanPlace` receives in vanilla (`Player.cs:39772`):
/// `Item.placeStyle` of the held item and `Player.direction`, the facing
/// defaulting to right (+1) with no player — the same default the replicated
/// `apply_edit` route sends (Decision #7).
fn style_and_facing(held_kind: u16, facing: Option<i32>) -> (i32, i32) {
    (registry::place_style(held_kind), facing.unwrap_or(1))
}

/// The world-write half of `place_tile`: the dispatcher write
/// (`edit::place_item_tile`) followed by the item consumption vanilla ties to
/// a successful `Place` (`Player.cs:40118-40126`) — a refused placement
/// keeps the item. Free of `App` so the consumption condition is
/// unit-testable without the app harness.
fn place_item_and_consume(
    world: &mut World,
    held: &mut ItemSlot,
    tx: i32,
    ty: i32,
    kind: u16,
    style: i32,
    dir: i32,
) -> bool {
    if !edit::place_item_tile(world, tx, ty, kind, style, dir) {
        return false;
    }
    held.count = held.count.saturating_sub(1);
    true
}

/// The tile-placement route of `Player.PlaceThing_Tiles` (`Player.cs:39721`):
/// the shared item gates, then `tr_core::edit::place_item_tile` dispatches
/// registry kinds through `TileObject.CanPlace`/`Place`
/// (`PlaceThing_Tiles_TryPlacing`, `Player.cs:39771-39778`) and everything else
/// through the legacy `WorldGen.PlaceTile` path with the app's
/// neighbour/overlap pre-checks (`can_place`).
fn use_tile_item(app: &mut App, tx: i32, ty: i32) -> bool {
    let held = app.inventory[app.selected];
    if item_gate_blocks(held.kind, held.count) {
        return false;
    }
    let Some(kind) = registry::place_tile(held.kind) else {
        return false;
    };
    // `Item.placeStyle` — the `CustomPlace(type, placeStyle)` gate reads it
    // (`Player.cs:39771`); the style-resolved sub-tile flag decides the route.
    let style = registry::place_style(held.kind);
    // `PlaceThing_Tiles`' lava gate (`Player.cs:39743`): an inactive target in
    // lava refuses a solid tile outright, and a non-solid tile whose
    // `TileObjectData.CheckLiquidPlacement` refuses (`IsBlockedByLava`,
    // `:41489-41504`) — unless the target is a replaceable block or the item
    // is one of the grass/moss overrides (:39743's OR chain). The gate reads
    // the target BEFORE the assorted cut rule, exactly as the C# evaluates it
    // ahead of `TryPlacing`; a lava cell never carries a cuttable plant, so
    // the ordering is observation-equivalent.
    if let Some(target) = app.world.tile(tx, ty).copied() {
        let grass_override = matches!(kind, 199 | 23 | 662 | 661 | 2 | 109 | 60 | 70 | 633)
            || td::TILE_MOSS.has(kind);
        if !target.active()
            && !grass_override
            && !tr_core::tileobject::is_replaceable_block(target.kind)
            && is_blocked_by_lava(kind, style, &target)
        {
            return false;
        }
    }
    // The plant rules of `PlaceThing_Tiles_BlockPlacementForAssortedThings`
    // (`Player.cs:41036-41360`) run on the non-`TileObject` placement route
    // before the can-place chain — seeds (82) are forced onto that route by
    // `Player.cs:39771-39772` even though the registry has the def. The two
    // C# halves split by the held item: the Staff of Regrowth sets
    // `staffOfRegrowthBonus` (:41038-41041) and takes the kill+replant block
    // (:41262-41293, `canPlace` false unless the target was 0/1/38); every
    // other item takes the `flag2` tileCut rule (:41298-41331) — the kill
    // plants nothing, so the clicked seed then places itself through the
    // normal route below and is consumed.
    if assorted_placement_route(kind, style) {
        if staff_of_regrowth(held.kind) {
            if !adjust_plants(app, tx, ty) {
                return false;
            }
        } else if !cut_plants_for_placement(app, tx, ty, kind) {
            return false;
        }
        // S1-P-i42 — the torch family's support flatten
        // (`PlaceThing_Tiles_BlockPlacementForAssortedThings`,
        // `Player.cs:41114-41146`): a sloped/half-brick support below/left/
        // right is pounded flat (`WorldGen.SlopeTile`) BEFORE the support
        // type check, so the torch can stand on it. Runs only when no
        // `SolidTileNoAttach` support already holds (the C#'s outer gate).
        if kind == 4 {
            flatten_torch_supports(app, tx, ty);
        }
    }
    if legacy_precheck_applies(kind, style) && !can_place(app, tx, ty) {
        return false;
    }
    place_tile(app, tx, ty, kind)
}

/// The torch support flatten of
/// `PlaceThing_Tiles_BlockPlacementForAssortedThings`
/// (`Player.cs:41114-41146`): when no `SolidTileNoAttach` support holds the
/// cell below/left/right, the first sloped/half-brick non-platform support
/// in the C#'s priority order (below → left → right) is pounded flat
/// (`WorldGen.SlopeTile`, `:41120`/`:41131`/`:41140`). The Dig sound the
/// flatten plays (`WorldGen.cs:80782`) rides the `UpdatePlayers` stream
/// (placement runs under the `Main.cs:17754` swap); the `KillTile(fail,
/// effectOnly)` dust (`:80780`) is the documented visual skip.
fn flatten_torch_supports(app: &mut App, tx: i32, ty: i32) {
    // `:41114` — the outer gate: any flat attachable support already holds.
    if tr_core::edit::solid_tile_no_attach(&app.world, tx, ty + 1)
        || tr_core::edit::solid_tile_no_attach(&app.world, tx - 1, ty)
        || tr_core::edit::solid_tile_no_attach(&app.world, tx + 1, ty)
    {
        return;
    }
    // The C#'s if/else-if chain (`:41116`/`:41127`/`:41138`): the FIRST
    // sloped/half-brick support in below → left → right order takes the
    // branch; it is pounded flat only when not a platform (the platform
    // check sits INSIDE the below/left arms, `:41118`/`:41129`, and in the
    // right arm's condition, `:41138`). The outer gate already proved all
    // three are not-attach, so the repeated `!SolidTileNoAttach` conjuncts
    // inside are redundant. A platform support consumes its branch without
    // flattening (no fall-through to the next arm).
    for (sx, sy) in [(tx, ty + 1), (tx - 1, ty), (tx + 1, ty)] {
        let Some(s) = app.world.tile(sx, sy).copied() else {
            continue;
        };
        if s.half_brick() || s.slope() != 0 {
            // The chest closure captures the DISJOINT `world_chests` field
            // (the try_pounding_tile_tick precedent) — `chest_holds_items`
            // borrows all of `app` and would conflict with `&mut app.world`.
            let chests = &app.world_chests;
            let holds = |x: i32, y: i32| {
                chests
                    .iter()
                    .any(|c| c.tile == (x, y) && c.slots.iter().any(|s| s.count > 0))
            };
            if !td::SET_PLATFORMS.has(s.kind)
                && tr_core::edit::slope_tile(&mut app.world, sx, sy, 0, false, holds)
            {
                app.audio.play_place_sound(
                    (0, 1),
                    sx as f32 * TILE_PX,
                    sy as f32 * TILE_PX,
                    app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                );
                app.world_key = None;
            }
            return;
        }
    }
}

/// The C# placement route the plant rules ride (`Player.cs:39771-39772`):
/// `TileObjectData.CustomPlace(tileToCreate, placeStyle) && tileToCreate != 82
/// && tileToCreate != 227 && tileToCreate != 4` sends 82/227/4 to the legacy
/// route even when registered. S1-F-7: the predicate is the negation of the
/// shared registry-route gate (`routes_through_tile_object`) — the natural
/// objects (pots 28, piles 185/186/187, plants, platforms 19) are registered
/// but `CustomPlace`-false, so they ride this legacy route too.
fn assorted_placement_route(kind: u16, style: i32) -> bool {
    !tr_core::tileobject::routes_through_tile_object(kind, style)
}

/// The held-item gate of `staffOfRegrowthBonus` (`Player.cs:41038-41041`, the
/// placement route; `:44649-44651`, the swing cut) — items 213/5295.
fn staff_of_regrowth(kind: u16) -> bool {
    matches!(kind, 213 | 5295)
}

/// The Staff-of-Regrowth half of
/// `Player.PlaceThing_Tiles_BlockPlacementForAssortedThings`
/// (`Player.cs:41038-41041`, `:41262-41293`): with the staff held, clicking
/// over a plant destroys it — saplings 3/73 and the blooming herb 84 always,
/// the mature 83 only when `IsAlchemyPlantHarvestable` — and the blooming
/// herb auto-replants (`try_replanting_herbs`). The harvest's drops ride the
/// same `break_tile` kill path as every other break source. Returns the C#
/// `canPlace` at the method's return: the staff's own branch sets it true
/// only over 0/1/38 (:41045-41048; an inactive cell reads as type 0), so no
/// grass is placed into a cell the staff just harvested.
fn adjust_plants(app: &mut App, tx: i32, ty: i32) -> bool {
    // :41045-41048 — `if (flag) { if (type == 0 || 1 || 38) canPlace = true; }`
    let can_place = app
        .world
        .tile(tx, ty)
        .map_or(true, |t| !t.active() || matches!(t.kind, 0 | 1 | 38));
    if let Some(t) = app.world.tile(tx, ty).copied() {
        if t.active() {
            match t.kind {
                3 | 73 | 84 => {
                    let herb_style = i32::from(t.frame_x) / 18;
                    let replant = t.kind == 84;
                    break_tile(app, tx, ty);
                    // `if (!Main.tile[...].active()) { if (type == 84)
                    // TryReplantingHerbs(herbStyle); }` (:41270-41278).
                    if replant && app.world.tile(tx, ty).is_some_and(|t| !t.active()) {
                        try_replanting_herbs(app, tx, ty, herb_style);
                    }
                }
                83 => {
                    // :41283-41292 — the kill is unconditional once the bloom
                    // window holds, and the replant follows without a check.
                    let herb_style = i32::from(t.frame_x) / 18;
                    if is_alchemy_plant_harvestable(herb_style, ty, &drop_ctx(app)) {
                        break_tile(app, tx, ty);
                        try_replanting_herbs(app, tx, ty, herb_style);
                    }
                }
                _ => {}
            }
        }
    }
    can_place
}

/// The `flag2` half of
/// `Player.PlaceThing_Tiles_BlockPlacementForAssortedThings`
/// (`Player.cs:41298-41331`) — the rule every non-staff placement item runs
/// on the legacy route: an active target that is `tileCut`,
/// `BreakableWhenPlacing` or `IsADripTile` and not the held item's own tile
/// is killed when it does not stand on 78/380/579 (`num9`), or — potted —
/// only for the rule's named plants (saplings, vines, drips, 165,
/// harvestable alch herbs) with an alch-placing item (`flag8`, the seeds).
/// No replant here: the killed cell takes the normal placement route below,
/// so the clicked seed plants itself (`PlaceAlch`) and is consumed. Returns
/// the C# `canPlace`: `false` refuses the placement and the plant stands.
fn cut_plants_for_placement(app: &mut App, tx: i32, ty: i32, create_tile: u16) -> bool {
    let Some(t) = app.world.tile(tx, ty).copied() else {
        return true;
    };
    // :41303 — the block only judges active cuttable targets.
    if !t.active() {
        return true;
    }
    if !(tr_core::tileobject::tile_cut(t.kind)
        || tr_core::tileobject::breakable_when_placing(t.kind)
        || tr_core::tileobject::is_drip_tile(t.kind))
    {
        return true;
    }
    // :41327-41330 — the held item's own tile refuses instead of cutting.
    if t.kind == create_tile {
        return false;
    }
    // :41307 — the support types below, read raw (no `active()` in the C#).
    let on_pot = app
        .world
        .tile(tx, ty + 1)
        .is_some_and(|s| matches!(s.kind, 78 | 380 | 579));
    // :41308-41313 — `(flag3 | flag4 | flag5 | flag6 | flag7) & flag8`.
    let ctx = drop_ctx(app);
    let named_plant = matches!(t.kind, 3 | 73)
        || tr_core::tileobject::is_vine(t.kind)
        || tr_core::tileobject::is_drip_tile(t.kind)
        || t.kind == 165
        || (tr_core::tileobject::tile_alch(t.kind)
            && is_harvestable_herb_with_seed(t.kind, i32::from(t.frame_x) / 18, ty, &ctx));
    let alch_item = tr_core::tileobject::tile_alch(create_tile);
    // `num9 || ((flag3|flag4|flag5|flag6|flag7) & flag8)` (:41314-41321).
    if !on_pot || (named_plant && alch_item) {
        break_tile(app, tx, ty);
        true
    } else {
        false // :41322-41325 — the potted plant stands; the placement refuses.
    }
}

/// `Player.TryReplantingHerbs` (`Player.cs:45824-45830`): the support below
/// must be active and 78/380/579 (579 = RockGolemHead — the decompile's
/// text, kept 1:1; `PlaceAlch`'s soil sets refuse it, so the gate alone
/// never plants on 579), then a direct `WorldGen.PlaceTile(x, y, 82, …,
/// herbStyle)` — no item consumed, no sound.
fn try_replanting_herbs(app: &mut App, tx: i32, ty: i32, herb_style: i32) {
    let supported = app
        .world
        .tile(tx, ty + 1)
        .is_some_and(|s| s.active() && matches!(s.kind, 78 | 380 | 579));
    if !supported {
        return;
    }
    if edit::place_item_tile(&mut app.world, tx, ty, 82, herb_style, 1) {
        app.request_edit(tx, ty, 1, 82);
        app.world_key = None;
    }
}

/// `Player.IsBottomOfTreeTrunkNoRoots` (`Player.cs:45784-45802`): the cell is
/// an active trunk (`TileID.Sets.IsATreeTrunk` or the palm 323) and NOT a
/// root-row cell (frameY 132-176 with frameX 22/44 — the palm exempt). The
/// Acorn Axe's replant gate reads it before the kill.
fn is_bottom_of_tree_trunk_no_roots(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    if !t.active() {
        return false;
    }
    if !td::SET_IS_A_TREE_TRUNK.has(t.kind) && t.kind != 323 {
        return false;
    }
    if t.kind != 323
        && (132..=176).contains(&t.frame_y)
        && (t.frame_x == 22 || t.frame_x == 44)
    {
        return false;
    }
    true
}

/// `Player.TryReplantingTree` (`Player.cs:45804-45822`): the Acorn Axe's
/// felled-base replant — `TileObject.CanPlace(tileTargetX, tileTargetY, 20,
/// 0, direction)` then `TileObject.Place` + `SquareTileFrame`. No item is
/// consumed (the axe is a tool, not the acorn); the placement rolls ride the
/// world's placement stream (the `place_item_tile` seam, S1-F-7b).
fn try_replanting_tree(app: &mut App, tx: i32, ty: i32) {
    let Some(def) = tr_core::tileobject::def_for_tile(20) else {
        return;
    };
    let dir = app.player.as_ref().map_or(1, |p| p.facing);
    let mut rand = std::mem::take(&mut app.world.place_rng);
    let instance = tr_core::tileobject::can_place(
        &app.world,
        tx,
        ty,
        def,
        0,
        dir,
        None,
        &mut rand,
        app.world.tile_object_random_cache,
    );
    app.world.place_rng = rand;
    let Some(instance) = instance else {
        return;
    };
    let placed = tr_core::tileobject::place(&mut app.world, &instance);
    // `WorldGen.SquareTileFrame(tileTargetX, tileTargetY)` (:45813) runs
    // unconditionally after the Place attempt (the C# calls it before the
    // `if (flag)` net/placement-hook tail).
    tr_core::framing::reframe_area(&mut app.world, tx, ty, 2, true);
    if placed {
        app.request_edit(tx, ty, 1, 20);
        app.world_key = None;
    }
}

/// `WorldGen.PlaceTile`'s placement-sound switch (`WorldGen.cs:60263-60282`)
/// is `tr_core::edit::place_tile_sound` (the `SoundCue` pair); the play
/// itself — the case-0 `Next(3)` variation + pitch jitter included — rides
/// [`crate::audio::Audio::play_place_sound`] (the faithful legacy cases,
/// S1-E-21 wave B).
pub(crate) fn place_tile(app: &mut App, tx: i32, ty: i32, kind: u16) -> bool {
    // `Item.placeStyle` + `Player.direction` are what `TileObject.CanPlace`
    // receives in vanilla (`Player.cs:39772`); the dispatcher resolves style
    // alternates and click points from them.
    let (mut style, dir) = style_and_facing(
        app.inventory[app.selected].kind,
        app.player.as_ref().map(|p| p.facing),
    );
    // S1-P-i42 — `PlaceThing_Tiles_PlaceIt`'s biome-torch conversion
    // (`Player.cs:40129-40132`): with the Torch God's favour unlocked
    // (`UsingBiomeTorches`) and a PLAIN torch (style 0), the placed style
    // becomes the biome's (`BiomeTorchPlaceStyle`, `:40236-40289`). The
    // hold-light family already reads the same unlock (`player_lights.rs`);
    // this is the placement half. The ZoneShimmer / ZoneUndergroundDesert
    // arms read fields the port's `Zones` does not carry — skipped in place,
    // the remaining arms (incl. temple + remix) keep their C# order (the
    // player_lights `biome_torch_hold_style` precedent).
    if kind == 4 && style == 0 {
        if let Some(p) = app.player.as_ref() {
            if p.unlocked_biome_torches {
                style = biome_torch_place_style(p, &app.world);
            }
        }
    }
    if !place_item_and_consume(
        &mut app.world,
        &mut app.inventory[app.selected],
        tx,
        ty,
        kind,
        style,
        dir,
    ) {
        return false;
    }
    // `Player.cs:40135-40140`: on a successful placement, an acorn's sapling
    // (`tileToCreate == 20 | 590 | 595 | 615`) bumps CONSERVATIONIST's `Do`
    // counter (`AchievementsHelper.cs:401-404`).
    if matches!(kind, 20 | 590 | 595 | 615) {
        let _ = app.achievements.notify_custom_int("Do", 1);
    }
    app.liquids.wake_around(&app.world, tx, ty, 3);
    // The placement cue: `tr_core::edit::place_tile_sound` is the
    // `WorldGen.PlaceTile` tail switch (`WorldGen.cs:60263-60282` — 127 →
    // Item30, 314 → Item52, 330-333 → Coins, default Dig); the
    // `TileObject`-routed kinds play the plain 0
    // (`Player.PlaceThing_Tiles_PlaceIt`, `Player.cs:40124`) and the special
    // kinds never route there (no `CustomPlace` def). The case-0
    // `Main.rand.Next(3)` variation + `Next(-10, 11) * 0.01` pitch
    // (`LegacySoundPlayer.cs:442-450`) ride the `UpdatePlayers` stream —
    // placement runs from `Player.ItemCheck` under the `Main.cs:17754` swap
    // (the `break_tile` precedent) — and only when audible.
    app.audio.play_place_sound(
        edit::place_tile_sound(kind),
        tx as f32 * TILE_PX,
        ty as f32 * TILE_PX,
        app.rngs.stream(crate::app::RngKey::UpdatePlayers),
    );
    app.request_edit(tx, ty, 1, kind);
    app.world_key = None;
    true
}

/// `Player.BiomeTorchPlaceStyle` (`Player.cs:40236-40289`): the biome torch
/// style for a plain torch placed with the Torch God's favour. The arm order
/// is the C#'s verbatim; `ZoneShimmer` (`:40240`) and `ZoneUndergroundDesert`
/// (`:40280`) read fields the port's `Zones` does not carry and are skipped
/// in place (the remaining arms keep their order, the
/// `player_lights::biome_torch_hold_style` precedent). `ZoneLihzhardTemple`
/// (`:40248`) rides `Zones.temple`; the remix desert arm (`:40284-40287`)
/// reads the world's REMIX seed flag. The underworld arm reads
/// `Main.UnderworldLayer * 16` = `(height - 200) * 16` (`Main.cs:2790`).
fn biome_torch_place_style(p: &crate::player::Player, world: &World) -> i32 {
    let z = p.zones;
    // `:40244-40247` — the dungeon arm comes before the underworld one.
    if z.dungeon {
        return 13;
    }
    // `:40248-40251` — the temple (Lihzhard) arm.
    if z.temple {
        return 21;
    }
    // `:40252-40255` — below the underworld layer.
    let underworld_px = (world.height - 200) as f32 * TILE_PX;
    if p.y > underworld_px {
        return 7;
    }
    if z.glowshroom {
        return 22; // `:40256-40259`
    }
    if z.hallow {
        return 20; // `:40260-40263`
    }
    if z.corrupt {
        return 18; // `:40264-40267`
    }
    if z.crimson {
        return 19; // `:40268-40271`
    }
    if z.snow {
        return 9; // `:40272-40275`
    }
    if z.jungle {
        return 21; // `:40276-40279`
    }
    // `:40280-40283` — the surface-desert half (the underground-desert half
    // reads an unported `Zones` field — skipped).
    if z.desert && (p.y as f64) < world.world_surface * TILE_PX as f64 {
        return 16;
    }
    // `:40284-40287` — `ZoneDesert && Main.remixWorld`: the remix seed's
    // inverted world grows its desert below the surface line, so the surface
    // gate above never fires there (the player_lights hold-style precedent).
    if z.desert && world.seeds.contains(tr_core::seeds::SeedFlags::REMIX) {
        return 16;
    }
    0
}

/// Applies a replicated edit. `kind`: 0 break tile, 1 place tile, 2 break wall,
/// 3 place wall. Returns the wall id a kind-2 break removed — the C# replay
/// runs the full `KillWall` (`MessageBuffer.cs:1177`), whose sound arm plays
/// on the receiver too; the caller plays the cue on the bare `Main.rand`
/// stream (the message handler runs outside every swap, `Main.cs:16895`'s
/// `NpcStrike` precedent — S1-E-21i).
pub(crate) fn apply_edit(world: &mut World, x: i32, y: i32, kind: u8, tile: u16) -> Option<u16> {
    match kind {
        0 => {
            // Vanilla suppresses item drops on the replication side
            // (`Main.netMode != 1` gate, `WorldGen.cs:63860`); the cascade
            // itself must run so every client converges on the same world.
            // The `KillTile` entry refuses the same three `CanKillTile`
            // cases (`CheckTileBreakability`, `:63537-63545`); no chest
            // store is reachable from a bare world, so that case passes
            // (the store is empty until M3-CHEST-UI anyway).
            if tr_core::edit::can_kill_tile(world, x, y, |_, _| false) {
                edit::kill_tile(world, x, y, true);
            }
            None
        }
        1 => {
            // Replicated placements take the same route as the local player.
            // Vanilla replicates object placements with a dedicated message
            // (`NetMessage.SendObjectPlacement`) carrying style/alternate/
            // random/direction; the generic edit message cannot, so the
            // registry route converges with style 0 / facing right.
            edit::place_item_tile(world, x, y, tile, 0, 1);
            None
        }
        2 => edit::break_wall(world, x, y),
        3 => {
            edit::place_wall(world, x, y, tile);
            None
        }
        _ => None,
    }
}

/// `ProcessHitAgainstNPC`'s landed-strike resolution
/// (`Player.cs:44291-44419`) — the class multiplier, the crit +
/// `DamageVar` rolls, the Flymeal/cane arms, the penetration sum, the
/// `StrikeNPC` call and its combat-text/net/sound tails. Extracted
/// VERBATIM from the cursor melee arm (S1-P-i05) so the useStyle-3 stab
/// sweep lands through the SAME resolution — the draw order and the
/// `UpdatePlayers` stream draws are byte-identical to the pre-extract
/// inline code. `idx` is the already-gated NPC (friendly/dontTakeDamage
/// refusals ran in the caller); `pcx` the player-centre X (the knockback
/// direction). Returns nothing — the callers own the swing-start arms.
fn resolve_melee_strike(
    app: &mut App,
    idx: usize,
    held: crate::item::ItemSlot,
    held_weapon: bool,
    held_fx: Option<crate::prefix::PrefixedStats>,
    melee_pen: i32,
    pcx: f32,
) {
                let n = &app.npcs[idx];
                // The Flymeal's TERRARIST (42) fires before the strike in the
                // C# (`Player.cs:44377-44387` vs `:44404`), but the notify
                // needs `&mut app` — it lands just ahead of the strike below,
                // still before `StrikeNPC`.
                let mut flymeal_town_hit = false;
                // `Player.ItemCheck_MeleeHitNPCs`: the class multiplier, the
                // crit roll (`meleeCrit` + the item's crit), `DamageVar`, then
                // `CalculateDamageNPCsTake` with the NPC's defense
                // (`NPC.cs:81226`; armor pen joins when the item data carries
                // it — M3-TOOL).
                let class = if held_weapon {
                    registry::get(held.kind)
                        .map(|d| d.class)
                        .unwrap_or_default()
                } else {
                    registry::DamageClass::Melee
                };
                let item_crit = held_fx.map(|s| s.crit).unwrap_or(0);
                // `ProcessHitAgainstNPC`'s `weaponDamage` (`Player.cs:44241`):
                // the class multiplier over the item's damage. Tools are
                // weapons in vanilla for `ItemCheck` purposes — the pickaxe
                // row carries `damage: 5, knock_back: 2.0` — so the tool arm
                // reads the same registry values (the pre-BUG-09 fixed 15
                // was the placeholder).
                let raw = held_fx
                    .map_or(registry::damage(held.kind), |s| s.damage)
                    .max(0) as f32
                    * app
                        .player
                        .as_ref()
                        .map(|p| p.weapon_damage_multiplier(class))
                        .unwrap_or(1.0);
                // `if (nPC.isLikeATownNPC && sItem.type == 3351) dmg = 1;`
                // (`Player.cs:44300-44304`): the Tax Collector's cane is a
                // one-damage slap on the town (it showers coins on mobs,
                // `:44362-44371` — the item drop rides the M3 item system).
                let raw = if n.kind.is_town_npc_like() && held.kind == 3351 {
                    1.0
                } else {
                    raw
                };
                // `Main.DamageVar(dmg, this.luck)` — the player-dealt roll
                // passes the raw value (`ProcessHitAgainstNPC`,
                // `Player.cs:44342`; damage taken negates it, `:31387`).
                let (luck, class_crit) = app
                    .player
                    .as_ref()
                    .map(|p| (p.luck(), p.class_crit(class)))
                    .unwrap_or((0.0, 0));
                // `Player.ItemCheck`'s melee hit runs under the
                // `UpdatePlayers` swap (`Main.cs:17754`).
                let loot_rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
                let crit = tr_core::damage::crit_roll(loot_rng, class_crit + item_crit);
                let mut dmg = tr_core::damage::damage_var(loot_rng, raw, luck);
                // `if (sItem.type == 5129 && nPC.isLikeATownNPC)`
                // (`Player.cs:44344-44353`): the Flymeal takes full armour
                // penetration (`armorPenetrationPercent = 1f`), doubles the
                // rolled damage and doubles it again on the Nurse (type 18).
                let mut pen_percent = 0.0f32;
                if held.kind == 5129 && n.kind.is_town_npc_like() {
                    pen_percent = 1.0;
                    dmg *= 2;
                    if n.kind == NpcKind::Nurse {
                        dmg *= 2;
                    }
                    // `:44377-44387` — the Flymeal's second block: a hit on a
                    // `townNPC` fires TERRARIST (42). The C# splits it from the
                    // `:44345` multiplier arm but gates on the same item and
                    // both run before `StrikeNPC` (`:44404`). Both predicates
                    // collapse to `is_town_npc_like()` here: it *is* the
                    // vanilla `townNPC` flag, and `isLikeATownNPC` only adds
                    // type 453, which the port does not ship.
                    flymeal_town_hit = true;
                }
                // The struck NPC's Copy reads — captured before the
                // mutable buff borrow below ends the `n` borrow (NLL).
                let n_defense = n.defense();
                let (nx, ny) = (n.x, n.y);
                let (bw, bh) = n.size();
                // The kind's hit cue (`NPC.HitSound`) — read before the app
                // borrows below.
                let hit_cue = n.kind.hit_sound();
                // `this.StatusToNPC(sItem.type, npcIndex)`
                // (`Player.cs:44391`, the block `:6502-6617`) — the melee
                // on-hit debuff infliction runs AFTER the crit/DamageVar
                // rolls above and BEFORE the nano-flask boost / the
                // armour-pen add / `StrikeNPC` (`:44396-44403`). The
                // buff-duration `Next` rolls ride the same
                // `UpdatePlayers` stream in the C#'s block order (the
                // `melee_on_hit.rs` module doc); an immune NPC still
                // consumes them (the C# evaluates the duration argument
                // before `NPC.AddBuff`'s guard, `NPC.cs:89928`).
                let enchant_flags = app.player.as_ref().map(|p| p.melee_on_hit_flags());
                if let Some(flags) = enchant_flags {
                    let status_rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
                    crate::melee_on_hit::status_to_npc_flags(
                        flags,
                        held.kind,
                        &mut app.npcs[idx].buffs,
                        status_rng,
                    );
                    // `if (this.meleeEnchant == 6) num2 = (int)((float)
                    // num2 * Player.GetNanoFlaskDamageBoost())`
                    // (`Player.cs:44396-44399`, `:44423-44426`) — the
                    // Flask of Nanites' ×1.05 on the rolled damage,
                    // between StatusToNPC and the armour-pen add.
                    if flags.melee_enchant == 6 {
                        dmg = (dmg as f32 * crate::melee_on_hit::nano_flask_damage_boost()) as i32;
                    }
                }
                // `:44383`: `num2 += nPC.checkArmorPenetration(
                // this.GetArmorPenetration(sItem.melee), armorPenetrationPercent)`
                // — the player's own penetration sum (the Shark Tooth
                // Necklace 3212, the Honey Comb 4007, the Sharpened buff 159's
                // melee-only +12) joins the Flymeal's full percent.
                dmg += tr_core::damage::check_armor_penetration(
                    melee_pen,
                    pen_percent,
                    n_defense,
                    false,
                    false,
                    false,
                );
                let dmg = tr_core::damage::strike_npc_damage(dmg, n_defense, crit);
                // `NPC.StrikeNPC` plays `this.HitSound` at `base.position`
                // (`NPC.cs:81546`) so the hit is placed at the NPC corner —
                // the position/size/cue reads were captured above, before
                // the StatusToNPC buff borrow.
                // `ItemCheck_MeleeHitNPCs`' knockback (`Player.cs:43664-43679`):
                // the item's `knockBack` times the glove/buff multipliers (the
                // port's 1f — no ported accessory grants them).
                let kb = held_fx.map_or(registry::get(held.kind).map_or(0.0, |d| d.knock_back), |s| s.knock_back);
                // `NPC.StrikeNPC` (`NPC.cs:81189-81563`): the strike lands on
                // the carrier's life pool when the hit NPC is a linked
                // segment (`NPC.realLife`, `NPC.cs:81374-81378`). The owner is
                // this player's slot — `StrikeNPC_Inner`'s interaction gate
                // key (`NPC.cs:81266-81272`).
                let me = app.local_id as u8;
                if flymeal_town_hit {
                    let _ = app.notify_progression_net(42);
                }
                // `Main.expertMode` (`NPC.cs:81419`) — the knockback gate's
                // ×15 reads the player's expert flag (the port's world-mode
                // stand-in, `BuffSlots.expert_mode`).
                let expert = app.player.as_ref().is_some_and(|p| p.buffs.expert_mode);
                let landed = crate::npc::Npc::strike_melee(
                    &mut app.npcs, idx, dmg, pcx, kb, crit, Some(me), expert,
                );
                // S1-P-i05: `ProcessHitAgainstNPC`'s tail
                // (`Player.cs:44414/:44419`) — the landed hit re-arms the
                // per-NPC melee lock to the CURRENT `itemAnimation` ticks
                // and `ApplyAttackCooldown` (`:44428-44431`) sets
                // `attackCD = max(1, (int)(itemAnimationMax * 0.33))`. The
                // cursor arm's caller overwrites `attack_cd` with the full
                // swing right after (the established start-arm stand-in);
                // the stab sweep keeps this C# value.
                if landed {
                    let anim_ticks = ticks_of(app.swing);
                    let uid = app.npcs[idx].uid;
                    if let Some(p) = app.player.as_mut() {
                        p.melee_npc_hit_cooldown.insert(uid, anim_ticks);
                    }
                    let max_ticks = ticks_of(app.swing_max);
                    // Keep-one (gate i04/i05): the shared ApplyAttackCooldown
                    // helper — identical double-truncation semantics.
                    app.attack_cd =
                        crate::melee_swing::MeleeHitCooldowns::attack_cooldown_frames(max_ticks)
                            as f32
                            / 60.0;
                }
                // `NPC.StrikeNPC_Inner`'s `HitEffect` tail (`NPC.cs:81488`):
                // the Destroyer body's probe gate (`:89266-89285`) — the
                // roll rides the melee strike's stream (the `UpdatePlayers`
                // swap the damage/crit rolls above already ride).
                if landed {
                    let probe_rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
                    if let Some(anchor) = crate::npc_ai_style37::probe_launch_on_hit(
                        &mut app.npcs[idx], probe_rng, false,
                    ) {
                        app.npcs[idx].wants_probe = Some(anchor);
                    }
                }
                if landed {
                    // `NPC.cs:81250-81264`: a landed strike on a `lifeMax > 1`
                    // NPC spawns the combat text with the post-defence number
                    // (`:81239`) and the crit flag. The local player's strike
                    // takes the hostile pair (`:81258`) — never the friendly
                    // (`:81254`) or network (`:81261`) arms.
                    let text = dmg.to_string();
                    let font = if crit {
                        app.lookups.combat_crit_font.as_ref()
                    } else {
                        app.lookups.combat_text_font.as_ref()
                    };
                    // `FontAssets.CombatText[crit ? 1 : 0].MeasureString`
                    // (`CombatText.cs:76`) — the placement size.
                    let size = match font {
                        Some(f) => [crate::ui::text::font_width(f, &text, 1.0), f.line_spacing],
                        None => [text.chars().count() as f32 * 8.0, 10.0],
                    };
                    let location = crate::combat_text::Rect {
                        x: nx,
                        y: ny,
                        width: bw,
                        height: bh,
                    };
                    let color = crate::combat_text::damage_text_color(crit, false, false);
                    // The port's player carries no flipped-gravity state, so
                    // `Main.player[Main.myPlayer].gravDir` is the normal 1f
                    // (`CombatText.cs:93`).
                    let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
                    crate::combat_text::spawn(
                        &mut app.combat_texts,
                        location,
                        color,
                        text,
                        crit,
                        false,
                        size,
                        1.0,
                        rng,
                    );
                }
                // A joining client's strike goes to the host as `28`
                // (`NPC.StrikeNPC`'s `fromNet: false` send, `NPC.cs:81218`).
                app.send_npc_strike(idx, dmg, kb, crit, pcx >= nx);
                // `NPC.StrikeNPC` plays `this.HitSound` at `base.position`
                // (`NPC.cs:81544-81553`) — the kind's own hit cue through
                // the faithful case-3 path (S1-E-21 wave C): the volume
                // arms + the `Main.rand.Next(-10, 11) * 0.01` pitch draw
                // ride the `UpdatePlayers` stream (`Main.cs:17754`), only
                // after the attenuation gate passed. The C#'s `null`
                // guard: the kinds with no row play nothing.
                if let Some(cue) = hit_cue {
                    let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
                    app.audio.play_npc_hit_sound(cue, nx, ny, rng);
                }
                // S1-P-i50 — the Lucky Coin family's melee-hit coin proc
                // (`ApplyNPCOnHitEffects`' tail, `Player.cs:44537-44549`,
                // called at `:44404` AFTER `StrikeNPC` — so after the
                // combat text and the hit cue above): `npc.value > 0 &&
                // hasLuckyCoin && Main.rand.Next(5) == 0` → type 71,
                // upgraded to 72 on `Next(10) == 0` and to 73 on
                // `Next(100) == 0` (independent rolls, gold wins), then
                // `RequestNewItem(stack: Next(1, 11), velocity:
                // (Next(10, 31) * 0.2 * direction, Next(-20, 1) * 0.2),
                // center: npc.Center, prefix: 0)`. The proc runs on a
                // landed hit regardless of the kill (`:44404` precedes the
                // `DidNPCDie` check `:44415`); the draws ride the
                // `UpdatePlayers` swap (`Main.cs:17754`). The explicit
                // velocity skips `Item.NewItem`'s spread draws
                // (`Item.cs:49738-49740`) and `prefix: 0` draws nothing —
                // six draws total when the Next(5) gate passes, one when
                // it refuses. (V4-landed block, M1 re-sync: folded into the
                // shared resolve_melee_strike so the stab sweep lands it too.)
                let lucky = app.player.as_ref().is_some_and(|p| p.has_lucky_coin);
                if landed && lucky && app.npcs[idx].value() > 0.0 {
                    let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
                    if rng.next_max(5) == 0 {
                        let mut kind = 71u16;
                        if rng.next_max(10) == 0 {
                            kind = 72;
                        }
                        if rng.next_max(100) == 0 {
                            kind = 73;
                        }
                        let stack = rng.next_range(1, 11) as u32;
                        let dir = app.player.as_ref().map_or(1, |p| p.facing);
                        let vx = rng.next_range(10, 31) as f32 * 0.2 * dir as f32;
                        let vy = rng.next_range(-20, 1) as f32 * 0.2;
                        let (ncx, ncy) = app.npcs[idx].center();
                        // `WorldItem.Center = center` (`Item.cs:49736`) —
                        // the 14x14 box centred on the NPC.
                        let mut coin = crate::item::ItemDrop::new(
                            kind,
                            stack,
                            0,
                            ncx - crate::item::SIZE * 0.5,
                            ncy - crate::item::SIZE * 0.5,
                            vx,
                            vy,
                            0.0,
                        );
                        // `NewItemOwnership.ReserveForLocalPlayer` →
                        // `enemyGrabDelayTime = DefaultGrabDelay = 100`
                        // (`Item.cs:49757-49760`, `WorldItem.cs:110`): the
                        // monsters cannot suck it for 100 ticks; the port's
                        // revenge arm has no enemy-delay field. The player
                        // grab gate reads `grab_delay_time` (S1-P-i47/i51):
                        // the enemy-delay is a separate C# clock, so the
                        // drop stays player-grabbable immediately.
                        coin.age = 0.0;
                        app.items.push(coin);
                    }
                }
}

/// S1-P-i05 — `ItemCheck`'s useStyle-3 (Thrust) use-start + stab sweep
/// (`Player.cs:43644-43686` with the `ItemCheck_GetMeleeHitbox` arm
/// `:45164-45189`). The umbrella pair (946/4707) are the only style-3
/// items in 1.4.5.8 (`Item.cs:12320-12332`/`:38683-38695`); they are
/// `noMelee: false` melee weapons with no projectile, so the port's
/// cursor arm never fired them (its `held_weapon` = `projectile.is_some()`
/// gate is the shooter definition) — this seam gives them the C#'s
/// behaviour: the press starts the use (`ItemCheck_TryStartUse` →
/// `ItemCheck_StartActualUse` `:51657-51698`: `attackCD = 0`,
/// `ResetMeleeHitCooldowns`, `ApplyItemAnimation`), then EVERY tick of
/// the animation the stab rect (`thrust_melee_hitbox` off the live
/// `thrust_item_placement`) scans the NPC pool — the wind-up third is
/// `dontAttack` (`:45168-45172`), the extension/retract rect hits every
/// NPC it intersects once per swing (`meleeNPCHitCooldown`,
/// `:44246`/`:44414`), under `attackCD <= 0` (`ApplyAttackCooldown`
/// `:44428-44431` after each landed hit). The per-tick
/// `UpdateMeleeHitCooldowns` decrement (`:43664`) runs once per damage
/// tick before the scan. The friendly/dontTakeDamage gates ride the
/// same predicates as the cursor arm (`ProcessHitAgainstNPC`
/// `:44265-44277`); the `CanHit` wall check (`:44287`) and the
/// `nPC.immune[whoAmI] == 0` gate ride the port's established
/// `strike_melee` `hurt_cd` stand-in. Returns true when the held item
/// took the thrust path (the caller skips the cursor melee arm).
fn thrust_stab_tick(
    app: &mut App,
    prev_mine_released: bool,
    prev_mine_down: bool,
    cursor_blocked: bool,
) -> bool {
    let held = app.inventory[app.selected];
    let Some(def) = registry::get(held.kind) else {
        return false;
    };
    if def.use_style != 3 || held.count == 0 || def.damage <= 0 {
        return false;
    }
    // The use-start: the press edge (or the held button on autoReuse)
    // with the timers clear (`ItemCheck_TryStartUse`'s gates,
    // `Player.cs:52042+`; the port's cursor arm's timer pair + the
    // `controlUseItem && releaseUseItem` edge, `:42733`). The umbrellas
    // are `autoReuse: false` (`Item.cs:12320+`) — one stab per press.
    // The cursor gate (`controlUseItem` zeroed over the HUD,
    // `Player.cs:24798-24807`) blocks the START only.
    let reuse_ready = def.auto_reuse || (prev_mine_released && !prev_mine_down);
    if app.mine_down
        && reuse_ready
        && !cursor_blocked
        && !app.inventory_open
        && app.swing <= 0.0
        && app.attack_cd <= 0.0
        && app.reuse_delay <= 0.0
    {
        app.swing_max = crate::item::use_animation_secs_of(held.kind, held.prefix);
        app.swing = app.swing_max;
        start_swing_extras(app, held.kind);
        // `ItemCheck_StartActualUse` (`Player.cs:51678-51680`):
        // `attackCD = 0; ResetMeleeHitCooldowns(); ApplyItemAnimation`.
        app.attack_cd = 0.0;
        if let Some(p) = app.player.as_mut() {
            p.melee_npc_hit_cooldown.clear();
        }
    }
    if app.swing <= 0.0 {
        return true;
    }
    let Some(p) = app.player.as_ref() else {
        return true;
    };
    let remaining = app.swing / app.swing_max.max(f32::EPSILON);
    let scale = crate::player::adjusted_item_scale(
        crate::prefix::prefixed_stats(held.kind, held.prefix)
            .map_or(def.scale, |s| s.scale),
    );
    // The held frame: the atlas icon quad is the `GetDrawHitbox`
    // stand-in the renderer uses (`Item.GetDrawHitbox` returns the
    // texture frame; the umbrellas' 44x44 stock frame `Item.cs:12321`).
    let (frame_w, frame_h) = app
        .lookups
        .item_icon(held.kind)
        .map_or((def.width as f32, def.height as f32), |(_, w, h)| (w, h));
    let (loc, _rot) = crate::player::thrust_item_placement(
        p.x,
        p.y,
        crate::player::WIDTH,
        p.facing as f32,
        remaining,
        frame_w,
        scale,
        0.0, // mountOffset — HeightOffsetHitboxCenter stands at 0
        i32::from(held.kind),
    );
    let Some(hitbox) = crate::player::thrust_melee_hitbox(
        loc,
        frame_w,
        frame_h,
        scale,
        p.facing,
        remaining,
        i32::from(held.kind),
    ) else {
        // `dontAttack` — the wind-up third never attacks (`:45170`), and
        // the `:43651` gate skips `UpdateMeleeHitCooldowns` too.
        return true;
    };
    let (rx, ry, rw, rh) = hitbox;
    let (pcx, _) = p.center();
    let melee_pen = p.get_armor_penetration(true);
    let (kill_guide, kill_clothier) = p.doll_flags();
    let attack_ready = app.attack_cd <= 0.0;
    let hits_town_npc = matches!(held.kind, 5129 | 3351);
    let held_fx = crate::prefix::prefixed_stats(held.kind, held.prefix);
    // `Player.cs:43664`: `UpdateMeleeHitCooldowns` — one decrement per
    // non-dontAttack damage tick, BEFORE the NPC scan (`:43681`).
    if let Some(p) = app.player.as_mut() {
        for cd in p.melee_npc_hit_cooldown.values_mut() {
            *cd -= 1;
        }
    }
    let cooldowns = app
        .player
        .as_ref()
        .map(|p| p.melee_npc_hit_cooldown.clone())
        .unwrap_or_default();
    // `ItemCheck_MeleeHitNPCs` (`Player.cs:44243-44252`): the slot-order
    // scan; the gates are `active && immune[whoAmI] == 0 &&
    // CanHitNPCWithMeleeHit && attackCD <= 0`. The `immune` gate rides
    // `strike_melee`'s `hurt_cd` stand-in (the established melee path).
    for idx in 0..app.npcs.len() {
        let n = &app.npcs[idx];
        if n.hp <= 0 || n.dont_take_damage {
            continue;
        }
        if n.kind.is_friendly()
            && !(n.kind == NpcKind::Guide && kill_guide)
            && !(n.kind == NpcKind::Clothier && kill_clothier)
            && !(n.kind.is_town_npc_like() && hits_town_npc)
        {
            continue;
        }
        if !attack_ready || cooldowns.get(&n.uid).is_some_and(|&cd| cd > 0) {
            continue;
        }
        // `ProcessHitAgainstNPC`'s rect intersect (`:44278-44279`).
        let (nw, nh) = n.size();
        let (nx, ny) = (n.x as i32, n.y as i32);
        let intersects = rx < nx + nw as i32
            && rx + rw > nx
            && ry < ny + nh as i32
            && ry + rh > ny;
        if !intersects {
            continue;
        }
        resolve_melee_strike(app, idx, held, true, held_fx, melee_pen, pcx);
        // The landed hit re-armed `attack_cd` (ApplyAttackCooldown) —
        // the C# scan keeps walking the slots under the new cooldown
        // (`:44246` reads the live field); with attackCD >= 1 no further
        // slot can hit this tick, so the walk ends here.
        break;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    /// S1-E-21 wave B: the placement-sound switch (`WorldGen.cs:60263-60282`)
    /// — the per-type cue selection the play path
    /// (`Audio::play_place_sound`) resolves; the RNG contract of the play
    /// is pinned in `audio.rs::s1_e_21b_*`.
    #[test]
    fn s1_e_21b_the_placement_cue_follows_the_tile_type() {
        // 127 (Actuator) → SoundID.Item30 (`SoundID.cs:1413`).
        assert_eq!(edit::place_tile_sound(127), (2, 30));
        // 314 (minecart track) → SoundID.Item52 (`SoundID.cs:1435`).
        assert_eq!(edit::place_tile_sound(314), (2, 52));
        // 330-333 (coin piles) → the legacy id 18 (Coins, `:60277`).
        for kind in 330..=333 {
            assert_eq!(edit::place_tile_sound(kind), (18, 1));
        }
        // The default → the legacy id 0 (Dig, `:60280`).
        assert_eq!(edit::place_tile_sound(0), (0, 1)); // dirt
        assert_eq!(edit::place_tile_sound(4), (0, 1)); // torch
        assert_eq!(edit::place_tile_sound(21), (0, 1)); // chest
        assert_eq!(edit::place_tile_sound(128), (0, 1)); // not 127
        assert_eq!(edit::place_tile_sound(329), (0, 1)); // not a coin pile
        assert_eq!(edit::place_tile_sound(334), (0, 1));
        // The special kinds are not `CustomPlace` registrations — the
        // `TileObject` route (the plain 0, `Player.cs:40124`) never claims
        // them, so the tail switch's per-type cues hold for both routes.
        for kind in [127u16, 314, 330, 331, 332, 333] {
            assert!(!tr_core::tileobject::routes_through_tile_object(kind, 0));
        }
    }

    /// BUG-16: the killed trunk cell's drop — the wood of the tree's family
    /// and the canopy marker's acorn (`WorldGen.KillTile_GetTreeDrops`,
    /// `WorldGen.cs:66189-66261`).
    #[test]
    fn bug_16_the_trunk_drop_is_the_family_wood_and_the_acorn() {
        let mut rng = tr_core::UnifiedRandom::new(0);
        let mut gen_stream = tr_core::tile_light::FastRandom::new(1);
        // A plain trunk cell: the wood of the tree's family, no acorn (the
        // acorn roll is the canopy marker's only — `frameX >= 22 && frameY >= 198`).
        let drops = trunk_tree_drops(
            tr_core::tree::TreeType::Forest,
            0,
            0,
            0,
            &mut gen_stream,
            &mut rng,
        );
        assert_eq!(drops[0].0, 9, "the forest tree drops Wood");
        assert!(
            !drops.iter().any(|(id, _)| *id == 27),
            "a plain trunk cell never drops an acorn"
        );
        // The jungle family drops Rich Mahogany (`GetTreeType`'s switch).
        let mut rng = tr_core::UnifiedRandom::new(0);
        let mut gen_stream = tr_core::tile_light::FastRandom::new(3);
        let drops = trunk_tree_drops(
            tr_core::tree::TreeType::Jungle,
            0,
            0,
            0,
            &mut gen_stream,
            &mut rng,
        );
        assert_eq!(drops[0].0, 620, "the jungle tree drops Rich Mahogany");
        // The canopy marker rolls the acorn (half the time, for every family
        // but the mushroom and the jungle).
        let mut acorns = 0;
        for seed in 0..24u64 {
            let mut rng = tr_core::UnifiedRandom::new(0);
            let mut gen_stream = tr_core::tile_light::FastRandom::new(seed);
            let drops = trunk_tree_drops(
                tr_core::tree::TreeType::Forest,
                22,
                198,
                0,
                &mut gen_stream,
                &mut rng,
            );
            assert_eq!(drops[0].0, 9, "the canopy cell drops wood too");
            if drops.iter().any(|(id, _)| *id == 27) {
                acorns += 1;
            }
        }
        assert!(acorns > 0, "the canopy marker rolls acorns");
        // The jungle never does (`TreeTypeDropsAcorns`, `WorldGen.cs:63112`).
        let mut jungle_acorns = 0;
        for seed in 0..24u64 {
            let mut rng = tr_core::UnifiedRandom::new(0);
            let mut gen_stream = tr_core::tile_light::FastRandom::new(seed);
            let drops = trunk_tree_drops(
                tr_core::tree::TreeType::Jungle,
                22,
                198,
                0,
                &mut gen_stream,
                &mut rng,
            );
            if drops.iter().any(|(id, _)| *id == 27) {
                jungle_acorns += 1;
            }
        }
        assert_eq!(jungle_acorns, 0, "the jungle tree never drops acorns");
    }

    /// S1-P-i30: the gem-tree drop order (`SetGemTreeDrops`,
    /// `WorldGen.cs:66094-66108` + the case bodies `:65699-65746`): the
    /// `Next(10)` gem-or-stone roll, the canopy marker's `Next(2)` gemcorn
    /// roll, then the stone's `Next(1, 3)` stack — all `Main.rand`, in that
    /// order. A non-marker stone cell spends exactly TWO draws (the gem roll
    /// and the stack roll); the seed roll never runs off the marker.
    #[test]
    fn s1_p_i30_the_gem_drop_rolls_in_the_csharp_order() {
        // Scan for a seed whose first Next(10) is NOT the gem (9 in 10 do).
        let mut stone_seed = 0i32;
        while tr_core::UnifiedRandom::new(stone_seed).next_max(10) == 0 {
            stone_seed += 1;
        }
        // The real resolver on that stream: stone 3, then the stack draw.
        let mut rng = tr_core::UnifiedRandom::new(stone_seed);
        let drops = gem_tree_drops(180, 4851, 0, 0, &mut rng);
        assert_eq!(drops.len(), 1, "a plain gem cell drops only the stone");
        assert_eq!(drops[0].0, 3, "the non-gem roll drops Stone");
        assert!((1..=2).contains(&drops[0].1), "the stone stack is Next(1,3)");
        // The stream spent exactly the gem roll + the stack roll (no seed
        // roll off the marker): a fresh clone advanced by those two draws
        // matches the resolver's stream state.
        let mut mirror = tr_core::UnifiedRandom::new(stone_seed);
        let _ = mirror.next_max(10);
        let _ = mirror.next_range(1, 3);
        assert_eq!(rng, mirror, "two draws, in the C# order");
        // The canopy marker spends the seed roll BETWEEN the gem and stack
        // rolls (`:66104` sits inside `SetGemTreeDrops`, before the case
        // body's stack roll).
        let mut rng = tr_core::UnifiedRandom::new(stone_seed);
        let drops = gem_tree_drops(180, 4851, 22, 198, &mut rng);
        let mut mirror = tr_core::UnifiedRandom::new(stone_seed);
        let gem_roll = mirror.next_max(10);
        let seed_roll = mirror.next_max(2);
        let stack = if gem_roll != 0 { mirror.next_range(1, 3) } else { 1 };
        assert_eq!(rng, mirror, "the marker spends gem, seed, then stack");
        assert_eq!(drops[0].0, if gem_roll == 0 { 180 } else { 3 });
        assert_eq!(drops[0].1, stack as u32);
        if seed_roll == 0 {
            assert!(drops.iter().any(|(id, _)| *id == 4851), "the gemcorn");
        } else {
            assert!(!drops.iter().any(|(id, _)| *id == 4851));
        }
    }

    /// S1-P-i30: the mushroom-tree drop (`case 71/72`,
    /// `WorldGen.cs:65642-65652`): `Next(40) == 0` → the seeds 194, else
    /// `Next(2) == 0` → the mushroom 183, else nothing. The second roll runs
    /// only when the first misses (the `else if`).
    #[test]
    fn s1_p_i30_the_mushroom_drop_is_the_two_stage_roll() {
        // Scan seeds for each outcome and mirror the draw count.
        let mut saw_seeds = false;
        let mut saw_shroom = false;
        let mut saw_none = false;
        for seed in 0..200i32 {
            let mut rng = tr_core::UnifiedRandom::new(seed);
            let drops = mushroom_tree_drops(&mut rng);
            let mut mirror = tr_core::UnifiedRandom::new(seed);
            let r40 = mirror.next_max(40);
            if r40 == 0 {
                assert_eq!(drops, vec![(194, 1)], "the 1-in-40 seeds");
                saw_seeds = true;
            } else {
                let r2 = mirror.next_max(2);
                if r2 == 0 {
                    assert_eq!(drops, vec![(183, 1)], "the 1-in-2 mushroom");
                    saw_shroom = true;
                } else {
                    assert!(drops.is_empty(), "nothing");
                    saw_none = true;
                }
            }
            assert_eq!(rng, mirror, "the draw count matches the C# else-if");
        }
        assert!(saw_seeds && saw_shroom && saw_none, "all three arms reachable");
    }

    /// DW-3.2 — the melee-hit rig: the `mining_app` shape with a blue slime
    /// parked under the screen-centre cursor and no tile there, so the
    /// `mine_down` arm takes the NPC branch (`ItemCheck_MeleeHitNPCs` →
    /// `ProcessHitAgainstNPC`). A floor row at y 300 gives the spawner a
    /// surface; the cursor tile (30, 260) stays air so nothing is mined.

    /// BUG-09: the melee arms run for weapons and tools alike — the inverted
    /// early return is gone and the damage reads the held row (`ItemCheck`'s
    /// `GetWeaponDamage` × the class multiplier → `DamageVar` → the NPC's
    /// defence; the tools included: the pickaxe's own `damage: 5`).
    #[test]
    fn bug_09_the_melee_hit_reads_the_item_damage() {
        // A wooden sword (damage 7, knockback 5.0) over the slime.
        let mut a = melee_app();
        a.inventory[a.selected].kind = 24;
        a.inventory[a.selected].count = 1;
        let hp_0 = a.npcs[0].hp;
        mine_frames(&mut a, 1);
        assert!(a.npcs[0].hp < hp_0, "the sword hit lands (defence-reduced > 0)");
        // The amount is the sword's OWN row through the ±15 % `DamageVar`
        // band and the slime's defence (`CalculateDamageNPCsTake`), x2 on a
        // crit — the rig's `UpdatePlayers` stream starts pristine at the
        // world seed, so the draws are fixed.
        // The pre-BUG-09 placeholder never reached this arm (the sword
        // returned early); the flat 15 discriminator is the pick below.
        let sword = hp_0 - a.npcs[0].hp;
        // (The literal is a stream re-pin: S1-A-11 P4 moved the pass onto the
        // `UpdatePlayers` stream, so DamageVar's draw — and the landed value —
        // changed; the value stays inside the ±15 % band + defence.)
        assert_eq!(sword, 7, "the sword's 7 through DamageVar − defence = {sword}");
        // The Iron Pickaxe (damage 5, knockback 2.0) with the cursor on the
        // NPC over air: the vanilla's `!noMelee && damage > 0` gate runs the
        // melee arm for the tools too — over an idle cursor cell the tool
        // hits (`ItemCheck`'s tile-first order only diverts the hit when the
        // cursor sits on a tile the tool can work).
        let mut b = melee_app();
        b.inventory[b.selected].kind = 1;
        let hp_b = b.npcs[0].hp;
        mine_frames(&mut b, 1);
        assert!(b.npcs[0].hp < hp_b, "the tool hits the NPC over air");
        // The pick's own 5 (±15 %) after the defence — NOT the pre-BUG-09
        // flat 15 (that row read 14 here: 15 − 2/2).
        let pick = hp_b - b.npcs[0].hp;
        // (Same stream re-pin — see the sword's note above.)
        assert_eq!(pick, 5, "the pick's 5 through DamageVar − defence = {pick}");
        // The same pick over a tile under the cursor mines instead — the
        // tile arm runs (the pick's power clears the stone under it).
        let mut c = melee_app();
        c.inventory[c.selected].kind = 1;
        let ci = c.world.index(30, 260); // the cursor cell (the camera anchor)
        c.world.tiles[ci] = Tile { kind: 1, ..Default::default() };
        c.world.tiles[ci].flags |= FLAG_ACTIVE;
        let hp_c = c.npcs[0].hp;
        mine_frames(&mut c, 1);
        assert_eq!(c.npcs[0].hp, hp_c, "the tile target runs, not the NPC hit");
    }


    /// M3-EQUIP-20 — the melee penetration row raises the hit
    /// (`Player.cs:44383` `num2 += nPC.checkArmorPenetration(...)`).
    #[test]
    fn m3_equip_20_the_penetration_raises_the_melee_hit() {
        // The same wooden sword as BUG-09's rig: 7 through DamageVar, the
        // slime's defense 2 halves to 1 -> 6.
        let mut a = melee_app();
        a.inventory[a.selected].kind = 24;
        a.inventory[a.selected].count = 1;
        let hp_0 = a.npcs[0].hp;
        mine_frames(&mut a, 1);
        let plain = hp_0 - a.npcs[0].hp;
        // The stream re-pin (S1-A-11 P4) — see BUG-09's note.
        assert_eq!(plain, 7, "the baseline hit");
        // The Shark Tooth Necklace (3212) adds a flat 5 of penetration;
        // `checkArmorPenetration` caps it at defense / 2 = 1, so the hit
        // gains exactly 1.
        let mut b = melee_app();
        b.inventory[b.selected].kind = 24;
        b.inventory[b.selected].count = 1;
        // The derived field is what the hit reads (`Player::update` fills it
        // from the worn slots — covered by
        // `player::tests::the_penetration_rows_add_up`); this rig drives only
        // the editing frame, so the value is set here directly.
        if let Some(p) = b.player.as_mut() {
            p.armor_penetration = 5;
        }
        let hp_b = b.npcs[0].hp;
        mine_frames(&mut b, 2);
        let poked = hp_b - b.npcs[0].hp;
        assert_eq!(
            poked,
            plain + 1,
            "the necklace's 1 point of penetration (equipped={:?}, pen={})",
            b.player.as_ref().map(|p| p.equipped[3]),
            b.player.as_ref().map_or(0, |p| p.get_armor_penetration(true))
        );
    }

    /// S1-P-i10 — the Bee Keeper's Confused lands unconditionally
    /// (`Player.StatusToNPC` `Player.cs:6613-6615`, called at `:44391`
    /// BEFORE the strike): 31 for a fixed 120, no roll. The slime is
    /// only immune to 20 (`NPCID.cs:6079-6084`), so 31 sticks.
    #[test]
    fn s1_p_i10_the_bee_keeper_confuses_on_the_melee_hit() {
        let mut a = melee_app();
        a.inventory[a.selected].kind = 1123; // Bee Keeper (no projectile row)
        a.inventory[a.selected].count = 1;
        mine_frames(&mut a, 1);
        let n = &a.npcs[0];
        let slot = n.buffs.find_buff_index(31).expect("Confused lands");
        assert_eq!(n.buffs.buff_time[slot], 120, "the fixed 120 (:6614)");
    }

    /// S1-P-i10 — the Magma Stone's OnFire3 ALWAYS lands (the 360/240/120
    /// ladder, `Player.cs:6552-6566`) and its 1-2 draws shift the
    /// `UpdatePlayers` stream vs the same hit without the flag (the
    /// DW-3.2 stream-divergence pattern). The flag is the derived field
    /// the hit reads — set directly like `m3_equip_20`'s penetration
    /// (the grant arms are pinned in `player.rs`/`accessory_combat.rs`).
    #[test]
    fn s1_p_i10_the_magma_stone_burns_and_draws_on_the_hit_stream() {
        let mut plain = melee_app();
        plain.inventory[plain.selected].kind = 24;
        plain.inventory[plain.selected].count = 1;
        mine_frames(&mut plain, 1);
        assert!(plain.npcs[0].buffs.find_buff_index(323).is_none());
        let stream_plain = plain.rngs.stream(crate::app::RngKey::UpdatePlayers).clone();

        let mut a = melee_app();
        a.inventory[a.selected].kind = 24;
        a.inventory[a.selected].count = 1;
        a.player.as_mut().unwrap().magma_stone = true;
        mine_frames(&mut a, 1);
        let n = &a.npcs[0];
        let slot = n.buffs.find_buff_index(323).expect("OnFire3 always lands");
        assert!(
            [120, 240, 360].contains(&n.buffs.buff_time[slot]),
            "the ladder's three fixed times (:6556/:6560/:6564)"
        );
        assert_ne!(
            *a.rngs.stream(crate::app::RngKey::UpdatePlayers),
            stream_plain,
            "the magma ladder drew 1-2 extra Next rolls mid-hit"
        );
    }

    /// S1-P-i10 — the Flask of Venom adds Venom 70 (60 * Next(5, 10))
    /// AND PotentAcid 395 (fixed 60) on the melee hit
    /// (`Player.cs:6506-6510`).
    #[test]
    fn s1_p_i10_the_venom_flask_adds_both_debuffs_on_the_melee_hit() {
        let mut a = melee_app();
        a.inventory[a.selected].kind = 24;
        a.inventory[a.selected].count = 1;
        a.player.as_mut().unwrap().melee_enchant = 1;
        mine_frames(&mut a, 1);
        let n = &a.npcs[0];
        let v = n.buffs.find_buff_index(70).expect("Venom lands");
        let t = n.buffs.buff_time[v];
        assert!((300..600).contains(&t) && t % 60 == 0, "60 * Next(5, 10) = {t}");
        let p = n.buffs.find_buff_index(395).expect("PotentAcid lands");
        assert_eq!(n.buffs.buff_time[p], 60, "the fixed 60 (:6509)");
    }

    /// S1-P-i10 — the Poison flask's draw is consumed even though the
    /// slime is immune to Poisoned 20: the buff never lands but the
    /// stream still diverges from the plain hit (the C# evaluates
    /// `60 * Main.rand.Next(5, 10)` before `NPC.AddBuff`'s guard,
    /// `NPC.cs:89928-89931`).
    #[test]
    fn s1_p_i10_the_poison_flask_draws_even_on_the_immune_slime() {
        let mut plain = melee_app();
        plain.inventory[plain.selected].kind = 24;
        plain.inventory[plain.selected].count = 1;
        mine_frames(&mut plain, 1);
        let stream_plain = plain.rngs.stream(crate::app::RngKey::UpdatePlayers).clone();

        let mut a = melee_app();
        a.inventory[a.selected].kind = 24;
        a.inventory[a.selected].count = 1;
        a.player.as_mut().unwrap().melee_enchant = 8;
        mine_frames(&mut a, 1);
        assert!(
            a.npcs[0].buffs.find_buff_index(20).is_none(),
            "the slime's immunity rejects the buff"
        );
        assert_ne!(
            *a.rngs.stream(crate::app::RngKey::UpdatePlayers),
            stream_plain,
            "the duration draw happened before the immunity guard"
        );
    }

    /// S1-P-i10 — the Flymeal's Stinky 120 (fixed 300, `:6571-6574`)
    /// lands on the town NPC it is allowed to slap (the `:44274`
    /// friendly-gate exception, BUG-10's rig).
    #[test]
    fn s1_p_i10_the_flymeal_stinks_up_the_guide() {
        let mut a = melee_app();
        let guide = crate::npc::Npc::spawn_guide(&a.world, 30).expect("the Guide spawns");
        a.npcs.clear();
        a.npcs.push(guide);
        a.npcs[0].x = 30.0 * TILE_PX;
        a.npcs[0].y = 260.0 * TILE_PX;
        a.inventory[a.selected].kind = 5129;
        a.inventory[a.selected].count = 1;
        mine_frames(&mut a, 1);
        let n = &a.npcs[0];
        let slot = n.buffs.find_buff_index(120).expect("Stinky lands");
        assert_eq!(n.buffs.buff_time[slot], 300, "the fixed 300 (:6573)");
    }

    /// BUG-10: melee on a town NPC is refused (`ProcessHitAgainstNPC`'s
    /// friendly gate, `Player.cs:44272-44276`) — the Guide Voodoo Doll (267)
    /// and the Clothier's Voodoo Doll (1307) are the melee exceptions, and
    /// they only arm while WORN (`ApplyEquipFunctional` over the accessory
    /// slots 3..10, `Player.cs:14368`/`:12948-12954`).
    #[test]
    fn bug_10_melee_refuses_the_town_npcs() {
        // A Guide standing on the floor at the cursor.
        let mut a = melee_app();
        let guide = crate::npc::Npc::spawn_guide(&a.world, 30).expect("the Guide spawns");
        a.npcs.clear();
        a.npcs.push(guide);
        a.inventory[a.selected].kind = 24; // the wooden sword
        let hp = a.npcs[0].hp;
        let hp_max = a.npcs[0].max_hp;
        mine_frames(&mut a, 1);
        assert_eq!(a.npcs[0].hp, hp, "the town NPC is not hurt");
        assert_eq!(a.npcs[0].max_hp, hp_max);
        // The doll in the BAG arms nothing: `ResetEffects` clears the flag
        // every tick and only `ApplyEquipFunctional` re-arms it, so the
        // pre-fix inventory scan was over-permissive.
        let mut bag = melee_app();
        let guide = crate::npc::Npc::spawn_guide(&bag.world, 30).expect("the Guide spawns");
        bag.npcs.clear();
        bag.npcs.push(guide);
        bag.inventory[bag.selected].kind = 24; // the sword does the hit
        bag.inventory[0].kind = 267; // the voodoo doll in the bag
        bag.inventory[0].count = 1;
        bag.npcs[0].x = 30.0 * TILE_PX;
        bag.npcs[0].y = 260.0 * TILE_PX;
        let hp = bag.npcs[0].hp;
        mine_frames(&mut bag, 1);
        assert_eq!(bag.npcs[0].hp, hp, "a bagged doll does not arm the flag");
        // The same doll WORN in an accessory slot arms it.
        let mut b = melee_app();
        let guide = crate::npc::Npc::spawn_guide(&b.world, 30).expect("the Guide spawns");
        b.npcs.clear();
        b.npcs.push(guide);
        b.inventory[b.selected].kind = 24; // the sword does the hit
        // `Player.cs:15124` — the worn doll arms `killGuide`. The rig drives
        // `update_editing` directly (no `update_player` tick), so the worn
        // accessory is written into the per-tick `Player.equipped` snapshot
        // the real loop fills from `App.equip.armor` (`world_sim/player.rs`).
        b.player.as_mut().unwrap().equipped[3] = 267;
        b.equip.armor[3].kind = 267;
        b.equip.armor[3].count = 1;
        // The Guide spawns at the surface; the rig's cursor sits at
        // (30, 260) — park the Guide into the cursor's box the way the
        // slime rig does.
        b.npcs[0].x = 30.0 * TILE_PX;
        b.npcs[0].y = 260.0 * TILE_PX;
        let hp = b.npcs[0].hp;
        mine_frames(&mut b, 1);
        assert!(b.npcs[0].hp < hp, "the doll-armed hit lands on the Guide");
    }

    /// BUG-10, the `friendly` half: the C# refuses EVERY friendly NPC, not
    /// just the ten housing kinds — the eleven seller town NPCs
    /// (`is_town_npc_like`, `NPC.cs:6856`) and the six bound/rescue forms
    /// (`NPC.cs:10257`/`:10493`/`:13502`/`:13796`/`:16652`).
    #[test]
    fn bug_10_melee_refuses_the_sellers_and_the_bound_forms() {
        for kind in [
            NpcKind::Angler,
            NpcKind::Stylist,
            NpcKind::TaxCollector,
            NpcKind::Golfer,
            NpcKind::Zoologist,
            NpcKind::Princess,
            NpcKind::DyeTrader,
            NpcKind::PartyGirl,
            NpcKind::Wizard,
            NpcKind::Mechanic,
            NpcKind::TravellingMerchant,
            NpcKind::BoundGoblin,
            NpcKind::BoundWizard,
            NpcKind::BoundMechanic,
            NpcKind::WebbedStylist,
            NpcKind::SleepingAngler,
            NpcKind::SleepingGolfer,
        ] {
            assert!(kind.is_friendly(), "{kind:?} is friendly in the C#");
            let mut a = melee_app();
            let mut npc = crate::npc::Npc::spawn_town(&a.world, 30, kind)
                .unwrap_or_else(|| panic!("{kind:?} spawns"));
            a.npcs.clear();
            npc.x = 30.0 * TILE_PX;
            npc.y = 260.0 * TILE_PX;
            a.npcs.push(npc);
            a.inventory[a.selected].kind = 24; // the wooden sword
            let hp = a.npcs[0].hp;
            mine_frames(&mut a, 1);
            assert_eq!(a.npcs[0].hp, hp, "{kind:?} is not hurt by melee");
        }
    }

    /// BUG-10, the two item exceptions: the Flymeal (5129) and the Tax
    /// Collector's Stick of Doom (3351) are the only melee items the C#
    /// lets through the friendly gate on a town NPC
    /// (`Player.cs:44274`), and the cane's hit is forced to one damage
    /// (`:44300-44304`) while the Flymeal doubles (x4 on the Nurse).
    #[test]
    fn bug_10_flymeal_and_stick_of_doom_hit_the_town() {
        // The Flymeal on a Guide: `isLikeATownNPC`-gated, so it lands.
        let mut a = melee_app();
        let guide = crate::npc::Npc::spawn_guide(&a.world, 30).expect("the Guide spawns");
        a.npcs.clear();
        a.npcs.push(guide);
        a.npcs[0].x = 30.0 * TILE_PX;
        a.npcs[0].y = 260.0 * TILE_PX;
        a.inventory[a.selected].kind = 5129;
        a.inventory[a.selected].count = 1;
        let hp = a.npcs[0].hp;
        mine_frames(&mut a, 1);
        assert!(a.npcs[0].hp < hp, "the Flymeal gets through the friendly gate");
        // `Player.cs:44377-44387` — the Flymeal's `townNPC` branch fires
        // TERRARIST (42).
        assert!(
            progression(&a, 42),
            "the Flymeal credits TERRARIST on a town NPC"
        );
        // The doubled 15 plus the 100 % armour penetration clears the
        // Guide's defence 30 by a wide margin (the sword's ~0 would not).
        assert!(
            hp - a.npcs[0].hp > 20,
            "the Flymeal doubles the roll ({} dmg)",
            hp - a.npcs[0].hp
        );
        // The Stick of Doom lands for exactly one: `dmg = 1` before
        // DamageVar and the defence floor (`CalculateDamageNPCsTake`'s
        // max(1, 1 - 15) = 1).
        let mut b = melee_app();
        let guide = crate::npc::Npc::spawn_guide(&b.world, 30).expect("the Guide spawns");
        b.npcs.clear();
        b.npcs.push(guide);
        b.npcs[0].x = 30.0 * TILE_PX;
        b.npcs[0].y = 260.0 * TILE_PX;
        b.inventory[b.selected].kind = 3351;
        b.inventory[b.selected].count = 1;
        let hp = b.npcs[0].hp;
        mine_frames(&mut b, 1);
        assert_eq!(hp - b.npcs[0].hp, 1, "the cane slaps for one");
        // A hostile is untouched by the friendly gate: the slime still takes
        // the plain item damage (the sword's 6 on the rig's defence).
        let mut c = melee_app();
        let hp = c.npcs[0].hp;
        c.inventory[c.selected].kind = 24;
        mine_frames(&mut c, 1);
        assert!(c.npcs[0].hp < hp, "the hostile is still hittable");
        // S1-E-18d — the `townNPC` predicate: a plain slime never credits
        // TERRARIST, and the Stick of Doom is a different item (`:44345` is
        // gated on 5129, `:44377` too).
        assert!(!progression(&c, 42), "a hostile credits nothing");
        assert!(!progression(&b, 42), "the Stick of Doom is not the Flymeal");
    }

    /// Whether the `PROGRESSION_EVENT_<id>` condition (`AchievementCondition
    /// .Name`, the save key) is completed on any row.
    fn progression(app: &App, id: u32) -> bool {
        let key = format!("PROGRESSION_EVENT_{id}");
        app.achievements
            .list
            .iter()
            .flat_map(|a| a.conditions.iter())
            .any(|c| c.name == key && c.completed)
    }

    fn melee_app() -> App {
        let mut w = World::new("t".into(), 64, 800);
        for x in 0..64 {
            let i = w.index(x, 300);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        w.compute_column_surface();
        let mut app = mining_app(w, 30, 260);
        let mut npc =
            crate::npc::Npc::spawn_slime_variant(&app.world, 30, crate::npc::SlimeVariant::Blue)
                .expect("the floor row spawns the slime");
        npc.x = 30.0 * TILE_PX;
        npc.y = 260.0 * TILE_PX;
        app.npcs.push(npc);
        app
    }

    /// S1-P-i05 — the useStyle-3 (Thrust) stab-sweep rig: the `melee_app`
    /// shape with the Umbrella (946 — the only style-3 melee weapon pair
    /// in 1.4.5.8, `Item.cs:12320-12332`) held and the slime parked
    /// inside the stab's reach. The player stands at x = 33 tiles
    /// (`mining_app`'s tx+3), facing right; the stab rect at mid-stab
    /// spans roughly [player.x + 5, player.x + 56] (the 44-wide frame,
    /// the 1.4 growth, the -6 nudge), so the slime at 34.5 tiles
    /// (player.x + 24) sits inside it. WRITTEN NOT RUN (code-only mode).
    fn thrust_app() -> App {
        let mut app = melee_app();
        app.inventory[app.selected] = crate::item::ItemSlot {
            kind: 946,
            name: "Umbrella",
            count: 1,
            prefix: 0,
        };
        // The slime inside the stab reach: the player's top-left is
        // (528, 4160) (33*16, 260*16); the first stab tick (remaining
        // 14/22) lands the rect at (526, 4166, 51, 40) — park the
        // 24x18 slime at (546, 4160), its box [546,570]x[4160,4178]
        // intersects.
        app.npcs[0].x = 546.0;
        app.npcs[0].y = app.player.as_ref().unwrap().y;
        // The stable identity (`NPC.whoAmI`) — `update_npcs` assigns it
        // in the live loop; the rig pins it so the cooldown map keys.
        app.npcs[0].uid = 1;
        app
    }

    /// The press starts the use (`ItemCheck_StartActualUse`,
    /// `Player.cs:51657-51698`): the swing arms at the umbrella's 22-tick
    /// `useAnimation`, `attackCD` clears and the melee cooldowns reset.
    /// The wind-up third (ticks 22..15, remaining > 0.666) is `dontAttack`
    /// (`:45168-45172`) — no damage before the stab arm.
    #[test]
    fn s1_p_i05_the_press_starts_the_stab_and_the_windup_never_hits() {
        let mut app = thrust_app();
        let hp0 = app.npcs[0].hp;
        // Tick 1: the press edge (mine_released true, mine_prev_down
        // false — the rig's fresh state) starts the use.
        app.mine_released = true;
        app.mine_prev_down = false;
        mine_frames(&mut app, 1);
        assert!(app.swing > 0.0, "the stab plays");
        assert_eq!(app.swing_style, 3, "the thrust style rides the swing");
        assert_eq!(app.npcs[0].hp, hp0, "the wind-up third never attacks");
        // The wind-up runs while remaining > 0.666: 22 ticks * 0.666 =
        // 14.65 — ticks 22 down to 15 stay wind-up (8 ticks).
        mine_frames(&mut app, 7);
        assert_eq!(app.npcs[0].hp, hp0, "still the wind-up at tick 8");
        // Tick 9: remaining = 14/22 = 0.636 < 0.666 — the stab arm hits.
        mine_frames(&mut app, 1);
        assert!(app.npcs[0].hp < hp0, "the stab arm lands the hit");
        // The umbrella's 10 through DamageVar ± defence — the same
        // resolution the cursor arm uses (the shared helper).
        let dealt = hp0 - app.npcs[0].hp;
        assert!(dealt > 0, "the umbrella's own damage row");
    }

    /// The one-hit-per-swing lock (`meleeNPCHitCooldown`,
    /// `Player.cs:44246`/`:44414`): the landed hit re-arms the cooldown to
    /// the current `itemAnimation` and `ApplyAttackCooldown`
    /// (`:44428-44431`) sets `attackCD = max(1, (int)(max * 0.33))` — the
    /// rest of the stab ticks cannot re-hit the same NPC.
    #[test]
    fn s1_p_i05_the_stab_hits_once_per_swing() {
        let mut app = thrust_app();
        app.mine_released = true;
        app.mine_prev_down = false;
        // Run the whole 22-tick animation with the button held.
        mine_frames(&mut app, 22);
        let hp_after = app.npcs[0].hp;
        assert!(hp_after < app.npcs[0].max_hp, "the stab landed");
        // The hit lands at tick 9 (itemAnimation 14 — the first
        // non-wind-up tick); the 13 later ticks each decrement the
        // cooldown once (the non-dontAttack ticks): 14 - 13 = 1.
        let dealt = app.npcs[0].max_hp - hp_after;
        // One umbrella hit: 10 ± 15 % through the slime's defence 2
        // (half-rounded), x2 on a crit — the single-hit ceiling is 22;
        // two hits would reach past it on any non-minimal pair.
        assert!(dealt <= 24, "one hit's band, not two: {dealt}");
        let uid = app.npcs[0].uid;
        let cd = app
            .player
            .as_ref()
            .unwrap()
            .melee_npc_hit_cooldown
            .get(&uid)
            .copied()
            .unwrap_or(0);
        assert_eq!(cd, 1, "the cooldown counts down with the animation");
        // A fresh press after the swing re-arms (ResetMeleeHitCooldowns,
        // `:51679`). M1 re-sync onto i12's landed end-of-tick latch
        // (`releaseUseItem = !controlUseItem`, `Player.cs:42829`, the V4
        // gate fix `mine_released = !prev_mine_down`): after a HELD swing
        // the latch needs TWO released frames to propagate — the first
        // released frame still writes `!prev_mine_down = false`, the
        // second writes `true`, and only the next press reads it. The rig
        // never decays attack_cd (that pass lives in world_sim/npcs.rs) —
        // clear it like the established `repress` helper does.
        app.npcs[0].hp = app.npcs[0].max_hp; // park the life for the count
        app.npcs[0].hurt_cd = 0.0;
        app.attack_cd = 0.0;
        app.mine_down = false;
        mine_frames(&mut app, 2);
        app.mine_down = true;
        let hp_before2 = app.npcs[0].hp;
        mine_frames(&mut app, 23);
        assert!(app.npcs[0].hp < hp_before2, "the second stab lands too");
    }

    /// The direction lock through the sim: the umbrella's `useTurn` is
    /// false (`Item.cs:12320+` carries no useTurn), so the facing holds
    /// through the stab even with the opposite run key held
    /// (`Player.cs:19837`). The `frame_inputs` refresh rides
    /// `world_sim::player::update_player` — the real wiring path; the
    /// lock itself is pinned in `player.rs::s1_p_i05_thrust_tests`.
    #[test]
    fn s1_p_i05_the_stab_locks_the_facing_through_the_sim() {
        let mut app = thrust_app();
        app.mine_released = true;
        app.mine_prev_down = false;
        mine_frames(&mut app, 1);
        assert!(app.swing > 0.0);
        // The player tick refreshes the frame inputs off the live swing.
        crate::world_sim::player::update_player(&mut app, 1.0 / 60.0);
        let p = app.player.as_ref().unwrap();
        assert!(p.frame_inputs.swing_active);
        assert!(!p.frame_inputs.use_turn, "the umbrella does not turn");
        assert_eq!(p.facing, 1);
    }

    /// RNG purity (the clone-mirror contract): the wind-up tick
    /// (`dontAttack`, `Player.cs:45168-45172`) and a non-intersecting stab
    /// tick draw NOTHING — the C#'s `ItemCheck_GetMeleeHitbox` + the
    /// `ItemCheck_MeleeHitNPCs` scan are pure until a hit lands
    /// (`ProcessHitAgainstNPC`'s rolls, `:44298+`). The witness clones the
    /// `UpdatePlayers` `UnifiedRandom` before the seam and asserts
    /// equality after (the established clone pattern — the seam's own
    /// draws, if any, would desync the clone). The use-START is excluded:
    /// its `UseSound` draws belong to `start_swing_extras`, not the sweep
    /// (`Player.cs:51681-51698`). WRITTEN NOT RUN (code-only mode).
    #[test]
    fn s1_p_i05_the_windup_and_miss_ticks_draw_nothing() {
        let mut app = thrust_app();
        // Arm the swing WITHOUT the use-start (mine_down false skips the
        // start block): 22 ticks, remaining 1.0 — the wind-up.
        app.swing_max = 22.0 / 60.0;
        app.swing = app.swing_max;
        app.swing_style = 3;
        app.mine_down = false;
        let before = app
            .rngs
            .stream(crate::app::RngKey::UpdatePlayers)
            .clone();
        assert!(thrust_stab_tick(&mut app, false, false, false));
        let after = app
            .rngs
            .stream(crate::app::RngKey::UpdatePlayers)
            .clone();
        assert_eq!(before, after, "the wind-up tick is RNG-pure");
        // A mid-stab tick whose rect misses every NPC: the slime parked
        // far right, out of the stab's reach (remaining 0.5 — the stab
        // arm, the rect near the player at x 528).
        app.npcs[0].x = 2000.0;
        app.swing = 11.0 / 60.0;
        let before = app
            .rngs
            .stream(crate::app::RngKey::UpdatePlayers)
            .clone();
        assert!(thrust_stab_tick(&mut app, false, false, false));
        let after = app
            .rngs
            .stream(crate::app::RngKey::UpdatePlayers)
            .clone();
        assert_eq!(before, after, "the miss tick is RNG-pure");
        // The miss tick still ran the cooldown decrement (`:43664`) —
        // the sweep took the stab arm, it did not bail at the wind-up.
        assert!(app.npcs[0].hp > 0, "nothing was struck");
    }

    #[test]
    fn test_DW_3_2_melee_hit_passes_the_player_luck_to_damage_var() {
        // DW-3.2 — `Main.DamageVar(dmg, this.luck)` with the RAW value for
        // player-dealt damage (`ProcessHitAgainstNPC`, `Player.cs:44342`;
        // damage taken negates it, `:31387` — that contact site is not wired
        // in the port yet). At luck 1.0 the reroll branch always fires
        // (2 extra draws), so the item stream diverges from luck 0 while the
        // hit lands either way.
        let mut a = melee_app();
        let hp_0 = a.npcs[0].hp;
        mine_frames(&mut a, 1);
        assert!(a.npcs[0].hp < hp_0, "the melee hit landed");
        let state_0 = a.rngs.stream(crate::app::RngKey::UpdatePlayers).clone();

        let mut b = melee_app();
        b.player.as_mut().unwrap().luck_state.luck = 1.0;
        mine_frames(&mut b, 1);
        assert!(b.npcs[0].hp < hp_0, "the lucky hit landed too");
        assert_ne!(
            *b.rngs.stream(crate::app::RngKey::UpdatePlayers),
            state_0,
            "luck 1.0 rerolled DamageVar"
        );
    }

    /// S1-P-i50 — the Lucky Coin melee proc (`ApplyNPCOnHitEffects`' tail,
    /// `Player.cs:44537-44549`): a landed hit on a `value > 0` NPC with
    /// `hasLuckyCoin` draws `Next(5)`; on 0 it draws the denomination
    /// upgrades (`Next(10)`, `Next(100)`), the stack (`Next(1, 11)`) and the
    /// velocity pair (`Next(10, 31)`, `Next(-20, 1)`), then spawns the coin
    /// at the NPC's centre. Differential witness: the ring and no-ring rigs
    /// share every pre-proc draw (crit/DamageVar/probe/combat-text/hit-cue),
    /// so the ring's stream is the no-ring's advanced by exactly the proc's
    /// draws — one when the Next(5) gate refuses, six when it passes. Both
    /// branches are exercised across a seed sweep; the spawn shape is
    /// pinned whenever the gate fires.
    #[test]
    fn s1_p_i50_the_lucky_coin_proc_draws_and_spawns_on_a_landed_hit() {
        let mut saw_fire = false;
        let mut saw_refuse = false;
        for seed in 0..40i32 {
            // The no-ring baseline: the hit lands, no coin spawns.
            let mut base = melee_app();
            seed_stream(&mut base, seed);
            let hp0 = base.npcs[0].hp;
            mine_frames(&mut base, 1);
            assert!(base.npcs[0].hp < hp0, "the baseline hit lands (seed {seed})");
            assert!(base.items.is_empty(), "no ring, no coin");
            let base_state = base.rngs.stream(crate::app::RngKey::UpdatePlayers).clone();

            // The ring rig, same seed: identical pre-proc draws, then the proc.
            let mut ring = melee_app();
            ring.player.as_mut().unwrap().has_lucky_coin = true;
            seed_stream(&mut ring, seed);
            mine_frames(&mut ring, 1);
            assert!(ring.npcs[0].hp < hp0, "the ring hit lands too");

            // Replay the two possible proc tails on the baseline state.
            let mut refused = base_state.clone();
            let _ = refused.next_max(5);
            let mut passed = base_state.clone();
            let gate = passed.next_max(5);
            let mut kind = 71u16;
            let mut stack = 0u32;
            if gate == 0 {
                if passed.next_max(10) == 0 {
                    kind = 72;
                }
                if passed.next_max(100) == 0 {
                    kind = 73;
                }
                stack = passed.next_range(1, 11) as u32;
                let _ = passed.next_range(10, 31);
                let _ = passed.next_range(-20, 1);
            }
            let ring_state = ring.rngs.stream(crate::app::RngKey::UpdatePlayers).clone();
            assert!(
                ring_state == refused || ring_state == passed,
                "seed {seed}: the proc draws exactly 1 (refused) or 6 (passed) past the baseline"
            );
            if gate == 0 {
                saw_fire = true;
                assert_eq!(ring.items.len(), 1, "the proc spawned one coin");
                let coin = &ring.items[0];
                assert_eq!(coin.kind, kind, "the denomination rolls");
                assert_eq!(coin.count, stack, "Next(1, 11) stack");
                assert_eq!(coin.prefix, 0, "prefix: 0");
                // Centre on the NPC (`WorldItem.Center = center`).
                let (ncx, ncy) = ring.npcs[0].center();
                assert!((coin.center().x - ncx).abs() < 1e-3);
                assert!((coin.center().y - ncy).abs() < 1e-3);
            } else {
                saw_refuse = true;
                assert!(ring.items.is_empty(), "the refused gate spawns nothing");
            }
        }
        assert!(saw_fire, "some seed fires the Next(5)==0 gate");
        assert!(saw_refuse, "some seed refuses it");
    }

    /// 40x40 world with a stone floor at y = 20 — the core `edit.rs` fixture
    /// shape, deep enough inside the world for `CanPlace`'s 5-tile edge margin
    /// (`TileObject.cs:215`).
    pub(super) fn object_world() -> World {
        let mut w = World::new("t".into(), 40, 40);
        w.frame_important = vec![false; td::TILE_COUNT];
        for x in 0..40 {
            let i = w.index(x, 20);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    pub(super) fn slot(kind: u16, count: u32) -> ItemSlot {
        ItemSlot {
            kind,
            name: "",
            count,
            prefix: 0,
        }
    }
    /// The App-free [`break_drop_events`] harness for the record tests: a
    /// `Default` ctx (the record paths never read it) and a fixed stream.
    fn drop_events(
        kill: &edit::TileKill,
        hit: Option<HitCell>,
    ) -> (Vec<(u16, u32, i32, i32)>, Vec<(i32, i32)>) {
        let mut main = tr_core::UnifiedRandom::new(0);
        break_drop_events(
            kill,
            &object_world(),
            hit,
            false,
            0,
            &mut main,
            &DropCtx::default(),
        )
    }

    // (a) the empty-slot/weapon gate shared by `use_tile_item` and `can_place`.
    #[test]
    fn item_gate_blocks_empty_slot_weapon_and_cactus() {
        // An empty slot never places, whatever the kind.
        assert!(item_gate_blocks(2, 0));
        // A weapon (shuriken, `ItemID.Shuriken` 42) never places.
        assert!(item_gate_blocks(42, 99));
        // The legacy cactus kind never places (`ItemID.Cactus` 80).
        assert!(item_gate_blocks(80, 99));
        // A stackable block item passes (dirt, `ItemID.DirtBlock` 2).
        assert!(!item_gate_blocks(2, 50));
        // The work bench item passes (`ItemID.WorkBench` 36).
        assert!(!item_gate_blocks(36, 1));
    }

    // (b) the registry-bypass condition of `use_tile_item`.
    #[test]
    fn registry_kinds_bypass_the_legacy_prechecks() {
        // The wooden set rides the `TileObject.CanPlace` route (`CustomPlace`
        // true, `Player.cs:39772`): the legacy neighbour/overlap pre-checks
        // must not gate it — the verdict belongs to the dispatcher.
        for tile in [10, 11, 14, 15, 18, 21] {
            assert!(!legacy_precheck_applies(tile, 0), "tile {tile} registry route");
        }
        // Unregistered kinds keep the legacy pre-checks. Torch 4 is registered
        // now (S1-F-7) but explicitly excluded from the TileObject route
        // (`Player.cs:39772`), so it keeps the legacy pre-checks too.
        for tile in [0, 1, 4] {
            assert!(legacy_precheck_applies(tile, 0), "tile {tile} legacy");
        }
        // S1-F-7: the natural objects split by their `CustomPlace` flag — pots
        // 28, small piles 185, platforms 19 and saplings 3 are registered but
        // `CustomPlace`-false (legacy pre-checks); dye plants 227 are excluded
        // by kind. The LARGE piles 186/187 are `CustomPlace`-true and ride the
        // TileObject route (the `Check3x2` cascade judges them).
        for tile in [28, 185, 19, 3, 227] {
            assert!(tr_core::tileobject::def_for_tile(tile).is_some(), "tile {tile} registered");
            assert!(legacy_precheck_applies(tile, 0), "tile {tile} legacy (CustomPlace false / excluded)");
        }
        for tile in [186, 187] {
            assert!(!legacy_precheck_applies(tile, 0), "tile {tile} registry route (CustomPlace true)");
        }
        // The bypass keys on the *tile* kind the held item maps to: the work
        // bench item (36) places tile 18 and bypasses; dirt (2) places tile 0
        // and does not.
        assert_eq!(registry::place_tile(36), Some(18));
        assert!(!legacy_precheck_applies(registry::place_tile(36).unwrap(), 0));
        assert_eq!(registry::place_tile(2), Some(0));
        assert!(legacy_precheck_applies(registry::place_tile(2).unwrap(), 0));
    }

    // (c) the style/facing resolution `place_tile` feeds the dispatcher.
    #[test]
    fn style_and_facing_resolve_the_can_place_inputs() {
        // `Item.placeStyle` (0 for the whole wooden set, `Item.cs:173`) and
        // `Player.direction`; no player defaults to right (+1) — the same
        // default the replicated `apply_edit` route sends (Decision #7).
        assert_eq!(style_and_facing(36, Some(1)), (0, 1));
        assert_eq!(style_and_facing(36, Some(-1)), (0, -1));
        assert_eq!(style_and_facing(36, None), (0, 1));
        // An unregistered item id resolves like the registry accessor: style 0.
        assert_eq!(style_and_facing(9999, None), (0, 1));
    }

    // (d) item consumption is tied to a successful place (`Player.cs:40118-40126`).
    #[test]
    fn successful_place_consumes_one_item() {
        let mut w = object_world();
        let mut held = slot(36, 10);
        assert!(place_item_and_consume(&mut w, &mut held, 10, 19, 18, 0, 1));
        assert_eq!(held.count, 9);
        // The object is in the world: the 2x1 bench fills both cells.
        for x in [10, 11] {
            let t = w.tile(x, 19).unwrap();
            assert!(t.active() && t.kind == 18, "bench cell ({x}, 19)");
        }
    }

    #[test]
    fn refused_place_keeps_the_item_and_the_world() {
        let mut w = object_world();
        // A stone block occupies the bench's second cell: partial overlap.
        let i = w.index(11, 19);
        w.tiles[i] = Tile {
            kind: 1,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        let mut held = slot(36, 10);
        let before = w.revision;
        assert!(!place_item_and_consume(&mut w, &mut held, 10, 19, 18, 0, 1));
        assert_eq!(held.count, 10, "a refused placement keeps the item");
        assert_eq!(w.revision, before, "a refused placement writes nothing");
        assert!(!w.tile(10, 19).unwrap().active(), "origin cell untouched");
    }

    #[test]
    fn style_and_facing_flow_through_to_the_dispatcher_frames() {
        // The wrapper passes `style`/`dir` straight to `place_item_tile`:
        // placeStyle 1 shifts the bench one style column (36 px full width,
        // `TileObjectData.cs:2818-2825`), facing right picks the chair's
        // `PlaceRight` alternate (`TileObject.cs:209`).
        let mut w = object_world();
        let mut held = slot(36, 1);
        assert!(place_item_and_consume(&mut w, &mut held, 10, 19, 18, 1, 1));
        assert_eq!(w.tile(10, 19).unwrap().frame_x, 36);
        assert_eq!(w.tile(11, 19).unwrap().frame_x, 54);

        let mut w2 = object_world();
        assert!(place_item_and_consume(&mut w2, &mut held, 10, 19, 15, 0, 1));
        assert_eq!(w2.tile(10, 18).unwrap().frame_x, 18);
        let mut w3 = object_world();
        assert!(place_item_and_consume(
            &mut w3, &mut held, 10, 19, 15, 0, -1
        ));
        assert_eq!(w3.tile(10, 18).unwrap().frame_x, 0);
    }

    // ---- Phase 3 (DW-3.2): the break drop events ---------------------------

    #[test]
    fn test_DW_3_2_break_drop_events_legacy_tile_drops_at_the_hit_cell() {
        // Stone (tile 1) has no registry def: the per-cell drop table
        // (`KillTile_GetItemDrops`) maps it to the stone item at the hit
        // cell (`WorldGen.cs:63860-63864`) — the hit state carries the
        // dying cell's position and frames (the C# `tileCache`).
        let kill = edit::TileKill {
            kind: 1,
            ..Default::default()
        };
        let hit = Some(HitCell {
            x: 7,
            y: 12,
            kind: 1,
            frame_x: 0,
            frame_y: 0,
        });
        let (drops, portals) = drop_events(&kill, hit);
        assert!(portals.is_empty());
        assert_eq!(drops, vec![(3, 1, 7, 12)]);
    }

    #[test]
    fn test_DW_3_2_break_drop_events_object_cascades_resolve_items() {
        // Registry kinds drop nothing per-cell (`KillTile_GetItemDrops`'s
        // empty case group, `WorldGen.cs:65748-66091`): the item comes from
        // the cascade record only, mapped through `tile_drop`'s style-0
        // slice. The bench record carries the C# drop position
        // (origin x, trigger y); the open-door record maps to the
        // closed-door item (`GetDoorItem(0)` = 25).
        let kill = edit::TileKill {
            kind: 18,
            object_drops: vec![edit::ObjectDrop {
                tile_kind: 18,
                x: 10,
                y: 19,
                ..Default::default()
            }],
            ..Default::default()
        };
        let hit = Some(HitCell {
            x: 11,
            y: 19,
            kind: 18,
            frame_x: 0,
            frame_y: 0,
        });
        let (drops, portals) = drop_events(&kill, hit);
        assert!(portals.is_empty());
        assert_eq!(drops, vec![(36, 1, 10, 19)]);

        let kill = edit::TileKill {
            kind: 1,
            object_drops: vec![edit::ObjectDrop {
                tile_kind: 11,
                x: 9,
                y: 18,
                ..Default::default()
            }],
            ..Default::default()
        };
        let hit = Some(HitCell {
            x: 9,
            y: 17,
            kind: 1,
            frame_x: 0,
            frame_y: 0,
        });
        assert_eq!(
            drop_events(&kill, hit).0,
            vec![(3, 1, 9, 17), (25, 1, 9, 18)],
            "legacy hit drop plus the chained open-door object drop"
        );
    }

    #[test]
    fn s1_e_21c_kill_cue_maps_the_legacy_families() {
        // The five families the tile-break table can emit, pinned to the
        // port's asset + the case's volume scale (`LegacySoundPlayer.cs`).
        assert_eq!(kill_cue((0, 1)), Some(("Dig_1", 1.0)));
        assert_eq!(kill_cue((6, 1)), Some(("Grass", 1.0)));
        assert_eq!(kill_cue((13, 1)), Some(("Shatter", 1.0)));
        assert_eq!(kill_cue((18, 1)), Some(("Coins", 1.0)));
        assert_eq!(kill_cue((21, 1)), Some(("Tink_1", 1.0)));
        // Item case: the per-style asset, the style-52 attenuation (:567-569).
        assert_eq!(kill_cue((2, 27)), Some(("Item_27", 1.0)));
        assert_eq!(kill_cue((2, 177)), Some(("Item_177", 1.0)));
        assert_eq!(kill_cue((2, 52)), Some(("Item_52", 0.35)));
        // NPC hit styles 20-54 halve the volume (:647-650).
        assert_eq!(kill_cue((3, 25)), Some(("NPC_Hit_25", 0.5)));
        // The killed family: the mummy's style 11 and the plain style 1.
        assert_eq!(kill_cue((4, 11)), Some(("NPC_Killed_11", 1.0)));
        assert_eq!(kill_cue((4, 1)), Some(("NPC_Killed_1", 1.0)));
        // An unlisted family stays silent rather than guessing a cue.
        assert_eq!(kill_cue((99, 1)), None);
    }

    #[test]
    fn s1_e_21c_a_break_plays_the_tile_cue_once() {
        // A plain dirt tile (kind 0? no — 1 = stone) breaks with the Tink
        // cue (`WorldGen.cs:66500-66504`) and nothing per drop.
        let mut w = World::new("t".into(), 40, 40);
        for x in 0..40 {
            let i = w.index(x, 20);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        w.compute_column_surface();
        w.compute_chunks();
        let mut app = mining_app(w, 20, 20);
        let before = app.audio.played_count();
        break_tile(&mut app, 20, 20);
        assert_eq!(
            app.audio.played_count() - before,
            1,
            "one break cue, no per-drop sound"
        );
    }

    // ---- Phase 1 (M1.4-TO-5): the ungated LMB mining branch ----------------

    use crate::app::flow::front_flow_tests::flow_app;

    pub(super) const MINE_SCREEN: Vec2 = Vec2::new(1280.0, 720.0);

    #[test]
    fn s1_p_i41_the_saddle_use_equips_the_mount_on_the_press_edge() {
        // `Player.ItemCheck`'s mount arm (`Player.cs:42768-42771`) behind
        // the use-start gate (`:42733`): the press edge equips the saddle's
        // mount + plays the use animation. The Blessed Apple (3260) → 10.
        let mut app = mining_app(World::new("t".into(), 64, 800), 30, 260);
        for x in 0..64 {
            let i = app.world.index(x, 300);
            app.world.tiles[i] = Tile { kind: 1, ..Default::default() };
            app.world.tiles[i].flags |= FLAG_ACTIVE;
        }
        // Park the player on the ground row 300.
        let p = app.player.as_mut().unwrap();
        p.y = 300.0 * TILE_PX - p.height;
        p.vy = 0.0;
        // The reach gate runs before the use arms — point the camera (and
        // the screen-centre cursor) at the player's body.
        app.camera = Vec2::new(p.x + 10.0, p.y + 20.0);
        app.inventory[app.selected] = ItemSlot {
            kind: 3260,
            name: "Blessed Apple",
            count: 1,
            prefix: 0,
        };
        app.mine_down = true;
        app.mine_released = true; // the button was up last frame
        app.mine_prev_down = false;
        let (sw, sh) = app.screen_size();
        let screen = Vec2::new(sw, sh);
        update_editing(&mut app, 1.0 / 60.0, screen);
        let p = app.player.as_ref().unwrap();
        assert!(p.mount.is_active() && p.mount.kind() == 10, "the saddle equips");
        assert!(p.buffs.find_buff_index(162).is_some(), "the mount buff");
        assert!(app.swing > 0.0, "the use animation runs");
        // A held press (no fresh edge) does not re-equip.
        app.mine_down = true;
        update_editing(&mut app, 1.0 / 60.0, screen);
        assert!(app.player.as_ref().unwrap().mount.is_active());
    }

    #[test]
    fn s1_p_i41_using_an_item_dismounts_the_pixie_and_cursed_refuses() {
        // `TryEarlyDismount` (`Mount.cs:6626-6632`) from the use head
        // (`Player.cs:24992`/`:31695-31702`): a dismounts-on-item-use mount
        // drops when a DIFFERENT item is used; a surviving mount cancels
        // the use. The Cursed (23) lock refuses the use outright
        // (`ItemCheck_CheckCanUse_Inner` `:52140-52148`, no mount exemption).
        let mut app = mining_app(World::new("t".into(), 64, 800), 30, 260);
        for x in 0..64 {
            let i = app.world.index(x, 300);
            app.world.tiles[i] = Tile { kind: 1, ..Default::default() };
            app.world.tiles[i].flags |= FLAG_ACTIVE;
        }
        let p = app.player.as_mut().unwrap();
        p.y = 300.0 * TILE_PX - p.height;
        p.vy = 0.0;
        app.camera = Vec2::new(p.x + 10.0, p.y + 20.0);
        assert!(p.set_mount(61, false)); // the Pixie, dismountsOnItemUse
        p.no_items = true; // the per-tick lock (Mount.cs:5144)
        // Hold the Iron Pickaxe (a non-mount item) mid-animation.
        app.inventory[app.selected] = ItemSlot {
            kind: 1,
            name: "Iron Pickaxe",
            count: 1,
            prefix: 0,
        };
        app.mine_down = true;
        app.swing = 0.2; // itemAnimation > 0 — the TryingToUseItem gate passes
        let (sw, sh) = app.screen_size();
        let screen = Vec2::new(sw, sh);
        update_editing(&mut app, 1.0 / 60.0, screen);
        let p = app.player.as_ref().unwrap();
        assert!(!p.mount.is_active(), "the use dismounted the Pixie");
        // `Mount.cs:6152` restores noItems only for 55/56 — the Pixie's lock
        // survives the dismount until the next tick's ResetEffects clear.
        assert!(p.no_items, "61 keeps the lock until ResetEffects");
        // Now Cursed: a fresh mount + the debuff refuse every use start.
        let p = app.player.as_mut().unwrap();
        assert!(p.set_mount(10, false)); // the Unicorn does NOT dismount on use
        p.no_items = true;
        p.cursed = true;
        app.swing = 0.0;
        app.mine_released = true;
        app.mine_prev_down = false;
        update_editing(&mut app, 1.0 / 60.0, screen);
        let p = app.player.as_ref().unwrap();
        assert!(p.mount.is_active(), "the Cursed lock keeps the mount");
        assert!(app.swing <= 0.0, "the use never started");
        assert!(app.mine_target.is_none());
    }

    #[test]
    fn s1_p_i41_spawn_force_clear_kills_solids_and_zeroes_liquid() {
        // `Player.Spawn_ForceClearArea` (`Player.cs:37914-37935`): the 3×3
        // strip above the floor — non-top solids die (their item drops
        // through the kill cascade), platforms (solidTop) stay, and every
        // liquid cell zeroes its liquid + lava flag.
        let mut w = World::new("clear".into(), 64, 128);
        for x in 0..64 {
            let i = w.index(x, 60);
            w.tiles[i] = Tile { kind: 1, ..Default::default() };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        // The floor cell (30, 60); the strip is x 29..=31, y 57..=59.
        let dirt = w.index(29, 59);
        w.tiles[dirt] = Tile { kind: 1, ..Default::default() }; // stone
        w.tiles[dirt].flags |= FLAG_ACTIVE;
        let plat = w.index(30, 58);
        w.tiles[plat] = Tile { kind: 19, ..Default::default() }; // platform
        w.tiles[plat].flags |= FLAG_ACTIVE;
        let lava = w.index(31, 57);
        w.tiles[lava].liquid = 255;
        w.tiles[lava].set_liquid_kind(tr_core::tile::LiquidKind::Lava);
        let mut app = flow_app(None, vec![], vec![]);
        app.world = w;
        app.liquids = tr_core::liquid::LiquidSystem::new(&app.world);
        spawn_force_clear_area(&mut app, 30, 60);
        assert!(!app.world.tile(29, 59).unwrap().active(), "the stone died");
        assert!(app.world.tile(30, 58).unwrap().active(), "the platform stays");
        assert_eq!(app.world.tile(31, 57).unwrap().liquid, 0, "liquid zeroed");
        assert_eq!(
            app.world.tile(31, 57).unwrap().liquid_kind(),
            tr_core::tile::LiquidKind::Water,
            "the lava flag cleared"
        );
        assert!(
            app.items.iter().any(|d| d.kind == 3),
            "the killed stone dropped its Stone Block item (noItem = false)"
        );
        // The floor row itself is NOT in the strip (`floorY - 3 .. floorY`).
        assert!(app.world.tile(30, 60).unwrap().active(), "the floor survives");
    }

    /// A headless mining rig: `world` staged by the caller, the player parked
    /// three tiles right of the target, the camera pinned so the screen-centre
    /// cursor points at `(tx, ty)` — everything the mining branch of
    /// `update_editing` reads (the HUD/early-return gates, the reach circle,
    /// the held slot). The held Iron Pickaxe clears the tool-tier gates from
    /// main (pick 40, `tr_core::tool`); the axe test swaps in the Iron Axe;
    /// the pick carries no projectile so the weapon branch falls through, and
    /// `flow_app`'s NPC list is empty so the melee branch does too.
    /// The shooter rig: the Wooden Bow held, the arrows in the ammo slots
    /// (and optionally the grid), the cursor to the right of the player.
    fn held_use_ammo_dbg(app: &App) -> i32 {
        crate::registry::get(app.inventory[app.selected].kind).map_or(0, |d| d.use_ammo)
    }

    fn shooter_app() -> App {
        let mut w = World::new("t".into(), 64, 800);
        for x in 0..64 {
            let i = w.index(x, 300);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        w.compute_column_surface();
        let mut app = mining_app(w, 30, 260);
        // The Cobalt Repeater (auto_reuse, useAmmo 40, shoot 15): the press
        // fires without the release edge (`Player.cs`'s autoReuse gate).
        app.inventory[app.selected] = crate::item::ItemSlot {
            kind: 435,
            name: "Cobalt Repeater",
            count: 1,
            prefix: 0,
        };
        // The rig simulates a button ALREADY held: the previous frame ended
        // with it down (`mine_prev_down`), so the release latch
        // (`releaseUseItem = !controlUseItem`, `Player.cs:42829` — the port
        // reads the previous frame's button) starts false, exactly like a
        // held button in the vanilla. The autoReuse forced release
        // (`ItemCheck_AutoReuseLogic`, `:52885`) re-arms it each tick.
        app.mine_down = true;
        app.mine_prev_down = true;
        app
    }

    #[test]
    fn s1_p_i07_the_dual_flail_launches_the_inventory_sibling() {
        // `Player.cs:49413-49425`: throwing the Blue Moon (item 163, shoot
        // 26) while the Sunfury (item 220, shoot 35) sits in the inventory
        // spawns the Sunfury too, at the MIRRORED velocity, with `ai[2] = 1`
        // (the `ai9` drawLayer flag) and its own GetWeaponDamage /
        // GetWeaponKnockback. The sibling spawn runs BEFORE the primary
        // (`:49437`). The old port never launched the sibling.
        let mut app = shooter_app();
        // Hold the Blue Moon; park the Sunfury in a grid slot.
        app.inventory[app.selected] = crate::item::ItemSlot {
            kind: 163,
            name: "Blue Moon",
            count: 1,
            prefix: 0,
        };
        app.inventory[20] = crate::item::ItemSlot {
            kind: 220,
            name: "Sunfury",
            count: 1,
            prefix: 0,
        };
        // Gate fix: the Blue Moon is `autoReuse: false` — the throw needs the
        // press edge (`Player.cs:42733` `controlUseItem && releaseUseItem`).
        // The rig's fresh state (`mine_released: false`) never arms it; the
        // whip/staff rigs set the edge explicitly. Same here.
        app.mine_released = true;
        app.mine_prev_down = false;
        let (sw, sh) = app.screen_size();
        // Aim right of the player so the throw velocity is +x.
        app.mouse_pos = Vec2::new(sw * 0.5 + 80.0, sh * 0.5);
        let screen = Vec2::new(sw, sh);
        update_editing(&mut app, 1.0 / 60.0, screen);
        let moon = app
            .projectiles
            .slots()
            .iter()
            .find(|p| p.active && p.kind == crate::projectile::BLUE_MOON);
        let sunfury = app
            .projectiles
            .slots()
            .iter()
            .find(|p| p.active && p.kind == crate::projectile::SUNFURY);
        assert!(moon.is_some(), "the held Blue Moon launches");
        let sunfury = sunfury.expect("the inventory Sunfury launches as the sibling");
        let moon = moon.unwrap();
        assert!(
            (sunfury.vel.x + moon.vel.x).abs() < 1e-3 && (sunfury.vel.y + moon.vel.y).abs() < 1e-3,
            "the sibling rides the mirrored velocity (moon {:?}, sunfury {:?})",
            moon.vel,
            sunfury.vel
        );
        assert_eq!(sunfury.ai[2], 1.0, "the sibling carries ai2 = 1 (the drawLayer flag)");
        // The Sunfury's base damage 32 (Item.cs:4133) at the default ×1
        // melee multiplier; the Blue Moon's is 27 — the sibling uses its OWN.
        assert_eq!(sunfury.damage, 32, "the sibling uses its own GetWeaponDamage");
    }

    #[test]
    fn the_bow_pulls_ammo_from_the_ammo_slots_first() {
        // `AmmoSlotOrder_Default` (`Player.cs:57141`): the ammo slots 54-57
        // precede the grid — the bow fires the ammo-slot stack, not the
        // grid's.
        let mut app = shooter_app();
        app.equip.ammo[0] = crate::item::ItemSlot {
            kind: 40,
            name: "Wooden Arrow",
            count: 7,
            prefix: 0,
        };
        app.inventory[20] = crate::item::ItemSlot {
            kind: 40,
            name: "Wooden Arrow",
            count: 99,
            prefix: 0,
        };
        let mut inv = app.inventory;
        let mut equip = std::mem::take(&mut app.equip);
        let pick = crate::ammo::pick(&inv, &equip, 40, None).unwrap();
        assert_eq!(pick, crate::ammo::AmmoSource::Ammo(0));
        crate::ammo::drain(pick, &mut inv, &mut equip);
        app.inventory = inv;
        app.equip = equip;
        assert_eq!(app.equip.ammo[0].count, 6, "the ammo slot drained");
        assert_eq!(app.inventory[20].count, 99, "the grid untouched");
    }

    #[test]
    fn the_bow_without_ammo_refuses_the_use() {
        // `ItemCheck_TryStartUse` → `HasAmmo` (`Player.cs:52021-52023`,
        // `:52316-52319`, `:53431-53446`): no ammo anywhere → the use is
        // refused, NO swing starts (S1-P-i15 fixed the old "dry swing"
        // divergence — `canShoot = false` (`:53462-53466`) only declines a
        // shot MID-swing, it never starts one).
        let mut app = shooter_app();
        let before = app.inventory[app.selected].count;
        // The cursor sits 80 px right of the screen centre (within the
        // 6-tile reach) — the reach gate is not what refuses the use.
        let (sw, sh) = app.screen_size();
        app.mouse_pos = Vec2::new(sw * 0.5 + 80.0, sh * 0.5);
        let screen = Vec2::new(sw, sh);
        update_editing(&mut app, 1.0 / 60.0, screen);
        assert_eq!(app.swing, 0.0, "no swing — HasAmmo refused the use");
        // The later ticks keep refusing: nothing fires, nothing drains.
        for _ in 0..40 {
            update_editing(&mut app, 1.0 / 60.0, screen);
        }
        assert!(app.projectiles.slots().iter().all(|p| !p.active));
        assert_eq!(app.inventory[app.selected].count, before);
    }

    #[test]
    fn the_bow_with_arrows_fires_and_drains() {
        // `ItemCheck_Shoot` (`Player.cs:47056+`): the projectile = the ammo's
        // `Item.shoot` (the Wooden Arrow's 1), the damage = the weapon's
        // + the ammo's through the class multiplier, the speed = the
        // weapon's + the ammo's `shootSpeed`.
        let mut app = shooter_app();
        app.equip.ammo[0] = crate::item::ItemSlot {
            kind: 40,
            name: "Wooden Arrow",
            count: 7,
            prefix: 0,
        };
        let (sw, sh) = app.screen_size();
        // The cursor 80 px right of the screen centre: within the reach, the
        // aim direction +x.
        app.mouse_pos = Vec2::new(sw * 0.5 + 80.0, sh * 0.5);
        let screen = Vec2::new(sw, sh);
        // The swing start + the fire tick.
        update_editing(&mut app, 1.0 / 60.0, screen);
        for _ in 0..40 {
            update_editing(&mut app, 1.0 / 60.0, screen);
        }
        let fired: Vec<_> = app
            .projectiles
            .slots()
            .iter()
            .filter(|p| p.active)
            .collect();
        assert_eq!(fired.len(), 1, "one arrow");
        assert_eq!(
            fired[0].kind, 1,
            "the ammo's Item.shoot — the WoodenArrowFriendly"
        );
        assert!(fired[0].vel.x > 0.0, "flies at the cursor");
        assert_eq!(app.equip.ammo[0].count, 6, "the arrow drained");
    }

    // ---- S1-P-i42: the light-item throw / flare-gun wire -------------------

    #[test]
    fn the_glowstick_throws_as_a_consumable_ai14_shot() {
        // `Player.ItemCheck_Shoot`'s generic arm (`Player.cs:49437`): the
        // glowstick (282, useStyle 1, shoot 50, shootSpeed 6, consumable,
        // NO useAmmo) throws its own projectile at the aim normalized to 6,
        // and the stack drains one. The row is aiStyle 14 (`Projectile.cs:
        // 1326-1336`), so the dispatch routes it to the glowstick arm.
        let mut app = shooter_rig(282, "Glowstick", &[]);
        app.inventory[app.selected].count = 5;
        // Gate V3: the glowstick is NOT autoReuse — the throw fires on the
        // press edge (`Player.cs:42733` controlUseItem && releaseUseItem).
        // Arm the released state so the first held frame reads as a press.
        app.mine_released = true;
        app.mine_prev_down = false;
        let (sw, sh) = app.screen_size();
        let screen = Vec2::new(sw, sh);
        update_editing(&mut app, 1.0 / 60.0, screen);
        for _ in 0..40 {
            update_editing(&mut app, 1.0 / 60.0, screen);
        }
        let fired: Vec<_> = app
            .projectiles
            .slots()
            .iter()
            .filter(|p| p.active && p.kind == 50)
            .collect();
        assert!(!fired.is_empty(), "the glowstick spawned");
        assert!(fired[0].vel.x > 0.0, "flies at the cursor");
        // The speed is the aim normalized to shootSpeed 6 (no melee scale —
        // the glowstick is DamageClass.None).
        assert!(
            (fired[0].vel.length() - 6.0).abs() < 0.5,
            "speed ~6, got {}",
            fired[0].vel.length()
        );
        assert_eq!(app.inventory[app.selected].count, 4, "one glowstick consumed");
    }

    #[test]
    fn the_flare_gun_fires_the_ammo_flare_and_drains_it() {
        // `ItemCheck_Shoot` + `PickAmmo` (`Player.cs:47056+`, `:53458+`): the
        // Flare Gun (930, useAmmo 931, shoot 163) fires the ammo's own
        // projectile (the Flare 931's shoot 163), the speed = the weapon's
        // 6 + the ammo's 6 = 12, and the flare stack drains one.
        let mut app = shooter_rig(930, "Flare Gun", &[(0, 931, 7)]);
        // Gate V3: the Flare Gun is NOT autoReuse — arm the press edge.
        app.mine_released = true;
        app.mine_prev_down = false;
        let (sw, sh) = app.screen_size();
        let screen = Vec2::new(sw, sh);
        update_editing(&mut app, 1.0 / 60.0, screen);
        for _ in 0..40 {
            update_editing(&mut app, 1.0 / 60.0, screen);
        }
        let fired: Vec<_> = app
            .projectiles
            .slots()
            .iter()
            .filter(|p| p.active && p.kind == 163)
            .collect();
        assert!(!fired.is_empty(), "the flare spawned");
        assert!(fired[0].vel.x > 0.0, "flies at the cursor");
        // The fused speed 12 (weapon 6 + ammo 6, `Player.cs:53542`).
        assert!(
            (fired[0].vel.length() - 12.0).abs() < 1.0,
            "speed ~12, got {}",
            fired[0].vel.length()
        );
        assert_eq!(app.equip.ammo[0].count, 6, "the flare drained");
    }

    #[test]
    fn the_flare_gun_without_ammo_plays_a_dry_swing() {
        // `canShoot = false` (`Player.cs:53462-53466`): no flare ammo — the
        // use plays out but nothing spawns.
        let mut app = shooter_rig(930, "Flare Gun", &[]);
        let (sw, sh) = app.screen_size();
        let screen = Vec2::new(sw, sh);
        update_editing(&mut app, 1.0 / 60.0, screen);
        for _ in 0..40 {
            update_editing(&mut app, 1.0 / 60.0, screen);
        }
        assert!(
            app.projectiles.slots().iter().all(|p| !p.active),
            "no flare without ammo"
        );
    }

    // ---- Phase 4 (M3-ITEM-6): the integrated shooter wire -----------------

    /// A shooter rig: the world/reach/cursor layout of `shooter_app` with an
    /// explicit HELD kind and the ammo stacks the caller names (the first tuple
    /// goes in the ammo slot, the second-to-last in the grid rows). The cursor
    /// sits 80 px right of the screen centre — within the 6-tile reach, the aim
    /// direction +x.
    fn shooter_rig(kind: u16, name: &'static str, ammo: &[(usize, u16, u32)]) -> App {
        let mut app = shooter_app();
        app.inventory[app.selected] = crate::item::ItemSlot {
            kind,
            name,
            count: 1,
            prefix: 0,
        };
        for &(i, ammo_kind, count) in ammo {
            // The ammo-slot index and the grid rows share one numbering here:
            // a slot < 8 lands in `equip.ammo`, anything else in the grid.
            let slot = crate::item::ItemSlot {
                kind: ammo_kind,
                name: crate::registry::name(ammo_kind),
                count,
                prefix: 0,
            };
            if i < 8 {
                app.equip.ammo[i] = slot;
            } else {
                app.inventory[i] = slot;
            }
        }
        let (sw, sh) = app.screen_size();
        app.mouse_pos = Vec2::new(sw * 0.5 + 80.0, sh * 0.5);
        // S1-P-i16 — level the aim: the rig's player centre sits 13 px below
        // the camera line (`mining_app`'s cell + 8 vs the centre's + 21), so
        // the generic aim would tilt −0.16 rad. The launcher/dart witnesses
        // read exact velocity/position components — the cursor line rides
        // the centre, `dir` is exactly +x (the staff rig keeps its own).
        app.camera.y = app.player.as_ref().unwrap().center().1;
        app
    }

    /// The live projectile count — the rig never kills them, so it only grows.
    fn shots(app: &App) -> usize {
        app.projectiles.slots().iter().filter(|p| p.active).count()
    }

    /// One frame of the shot wire at the exact 60 Hz step, recording
    /// `(remaining animation ticks, live shots)` AFTER the frame whenever a shot
    /// fired in it — the tick value the roll chain's windows read
    /// (`Player.cs:4066`, `:53635`, `:53663`), which the DW-4.1 edge assertions
    /// check instead of eyeballing.
    fn step(app: &mut App, screen: Vec2, log: &mut Vec<(i32, usize)>) {
        let before = shots(app);
        update_editing(app, ONE_TICK_SECS, screen);
        let after = shots(app);
        if after > before {
            log.push(((app.swing * 60.0).round() as i32, after));
        }
    }

    /// Hold the button for `n` frames (the press IS the first frame).
    fn hold(app: &mut App, screen: Vec2, n: usize, log: &mut Vec<(i32, usize)>) {
        for _ in 0..n {
            step(app, screen, log);
        }
    }

    /// The elapsed ticks of a log's shots — the C#'s `itemAnimation` values at
    /// the corresponding ticks (the press-frame arming puts the first shot at
    /// `max - 1`).
    fn shot_ticks(log: &[(i32, usize)]) -> Vec<i32> {
        log.iter().map(|&(tick, _)| tick).collect()
    }

    /// The fixed step's per-tick ammo-mastery pass, which the `update_editing`
    /// rig does not drive: `update_player`'s grant + cycler tick
    /// (`world_sim/player.rs`, the C#'s `UpdateBuffs` → `UpdateEquips` →
    /// `Update_AmmoCycler` → `ItemCheck` order, `Player.cs:25566`/`:25633`/
    /// `:24566`/`:31827`). Tests that arm the flags or a cycling mode call it
    /// before each press.
    fn mastery_tick(app: &mut App) {
        if let Some(p) = app.player.as_mut() {
            let armor = app.equip.armor;
            let misc = app.equip.misc_equips;
            crate::world_sim::player::grant_ammo_mastery(p, &armor, &misc);
            p.ammo_cycling.tick();
        }
    }

    /// A fresh press: the discrete-use start gate reads `attack_cd` (the
    /// port's `itemAnimation == 0` stand-in, census U6), so a test that wants
    /// a second swing clears the start lock, the animation and the reuse lock
    /// and lets the held button repeat (`Player.cs:42819-42822`'s autoReuse
    /// restart). The autoReuse forced release re-arms `mine_released` each
    /// tick, so no button edge is needed for an autoReuse kind.
    fn repress(app: &mut App) {
        app.swing = 0.0;
        app.swing_max = 0.0;
        app.attack_cd = 0.0;
        app.reuse_delay = 0.0;
    }

    /// S1-P-i17 — the post-PickAmmo bow conversions (`Player.cs:47157-
    /// 47168`): the Molten Fury burns ONLY the wooden arrow, the Marrow
    /// and the Ice Bow convert EVERY ammo unconditionally.
    #[test]
    fn s1_p_i17_the_bow_conversions_fire_after_the_pick() {
        // The Molten Fury (120) + the Wooden Arrow (40) → the FlameArrow 2.
        // Gate fix: the Molten Fury is `autoReuse: false` (`Item.cs:120` sets
        // none) — the shot needs the press edge (`Player.cs:42733`).
        let mut app = shooter_rig(120, "Molten Fury", &[(0, 40, 99)]);
        app.mine_released = true;
        app.mine_prev_down = false;
        let screen = app.screen_size().into();
        let mut log = Vec::new();
        hold(&mut app, screen, 30, &mut log);
        let fired: Vec<_> = app.projectiles.slots().iter().filter(|p| p.active).collect();
        assert!(!fired.is_empty(), "the Fury fired");
        assert!(fired.iter().all(|p| p.kind == 2), "the wooden arrow burns");
        // The Molten Fury + the Flaming Arrow (41) → stays 2 (the ammo's
        // own shoot - the conversion gates on projToShoot == 1).
        let mut app = shooter_rig(120, "Molten Fury", &[(0, 41, 99)]);
        app.mine_released = true;
        app.mine_prev_down = false;
        let mut log = Vec::new();
        hold(&mut app, screen, 30, &mut log);
        let fired: Vec<_> = app.projectiles.slots().iter().filter(|p| p.active).collect();
        assert!(fired.iter().all(|p| p.kind == 2), "the Flaming Arrow's own 2");
        // The Marrow (682) + the Wooden Arrow → the Bone Arrow 117
        // (UNCONDITIONAL — `:47161-47164` has no projToShoot gate).
        let mut app = shooter_rig(682, "Marrow", &[(0, 40, 99)]);
        let mut log = Vec::new();
        hold(&mut app, screen, 30, &mut log);
        let fired: Vec<_> = app.projectiles.slots().iter().filter(|p| p.active).collect();
        assert!(!fired.is_empty());
        assert!(fired.iter().all(|p| p.kind == 117), "the Marrow fletches bone");
        // The Ice Bow (725) + the Wooden Arrow → the Frost Arrow 120.
        let mut app = shooter_rig(725, "Ice Bow", &[(0, 40, 99)]);
        let mut log = Vec::new();
        hold(&mut app, screen, 30, &mut log);
        let fired: Vec<_> = app.projectiles.slots().iter().filter(|p| p.active).collect();
        assert!(!fired.is_empty());
        assert!(fired.iter().all(|p| p.kind == 120), "the Ice Bow frosts");
    }

    /// S1-P-i17 — the Daedalus Stormbow's arrow-type volley branch
    /// (`Player.cs:47424-47437`): the heavy arrows (91/4/5/41) roll to
    /// DROP one (2-3 per volley), every other kind rolls to ADD one
    /// (3-4). ONE ammo drains per volley (the pick runs once). The count
    /// roll is seed-dependent, so the witness observes SEVERAL volleys
    /// and asserts every one lands in its band.
    #[test]
    fn s1_p_i17_the_stormbow_volley_is_arrow_type_gated() {
        // The wooden arrow: every volley is 3 or 4, never 2.
        let mut app = shooter_rig(3029, "Daedalus Stormbow", &[(0, 40, 99)]);
        let screen = app.screen_size().into();
        let mut log = Vec::new();
        let mut volleys = Vec::new();
        for _ in 0..6 {
            let before = shots(&app);
            hold(&mut app, screen, 25, &mut log);
            let n = shots(&app);
            if n > before {
                volleys.push(n - before);
            }
            repress(&mut app);
        }
        assert!(volleys.len() >= 3, "several volleys fired");
        for v in &volleys {
            assert!((3..=4).contains(v), "the wooden volley is 3-4, got {v}");
        }
        assert_eq!(
            app.equip.ammo[0].count as usize,
            99 - volleys.len(),
            "one ammo per volley"
        );
        assert!(
            app.projectiles.slots().iter().filter(|p| p.active).all(|p| p.kind == 1),
            "the volley carries the ammo's kind"
        );
        // The volley rains from ABOVE: every spawn Y sits above the
        // player, every velocity Y points down (the aim-Y fold).
        let py = app.player.as_ref().unwrap().y;
        for p in app.projectiles.slots().iter().filter(|p| p.active) {
            assert!(p.pos.y < py, "the arrow spawns above the player");
            assert!(p.vel.y > 0.0, "the arrow rains down");
        }
        // The Holy Arrow (516 → 91): every volley is 2 or 3, never 4.
        let mut app = shooter_rig(3029, "Daedalus Stormbow", &[(0, 516, 99)]);
        let mut log = Vec::new();
        let mut volleys = Vec::new();
        for _ in 0..6 {
            let before = shots(&app);
            hold(&mut app, screen, 25, &mut log);
            let n = shots(&app);
            if n > before {
                volleys.push(n - before);
            }
            repress(&mut app);
        }
        assert!(volleys.len() >= 3, "several volleys fired");
        for v in &volleys {
            assert!((2..=3).contains(v), "the Holy volley is 2-3, got {v}");
        }
        assert!(
            app.projectiles.slots().iter().filter(|p| p.active).all(|p| p.kind == 91),
            "the Holy Arrow's own 91"
        );
    }

    /// S1-P-i17 — the Blood Rain Bow (`Player.cs:47466-47499`): 1-3
    /// arrows per volley (`Next(1,3)` + the 1/3 bonus), all the 819
    /// Blood Arrow, raining from above.
    #[test]
    fn s1_p_i17_the_blood_rain_bow_volley() {
        let mut app = shooter_rig(4381, "Blood Rain Bow", &[(0, 40, 99)]);
        let screen = app.screen_size().into();
        let mut log = Vec::new();
        let mut volleys = Vec::new();
        for _ in 0..6 {
            let before = shots(&app);
            hold(&mut app, screen, 25, &mut log);
            let n = shots(&app);
            if n > before {
                volleys.push(n - before);
            }
            repress(&mut app);
        }
        assert!(volleys.len() >= 3, "several volleys fired");
        for v in &volleys {
            assert!((1..=3).contains(v), "the Blood volley is 1-3, got {v}");
        }
        assert!(
            app.projectiles.slots().iter().filter(|p| p.active).all(|p| p.kind == 819),
            "the bow's unconditional 819 (`Player.cs:53507-53510`)"
        );
        assert_eq!(
            app.equip.ammo[0].count as usize,
            99 - volleys.len(),
            "one ammo per volley"
        );
    }

    #[test]
    fn test_DW_4_1_the_clockwork_swing_drains_one_ammo_of_three_shots() {
        // `Player.cs:53631` + `:4066`: the Clockwork's `useTime 4 < useAnimation
        // 12` fires 3 shots per swing at the animation ticks 11/7/3 — the first
        // is "just started" (`ItemAnimationJustStarted`) and consumes, the
        // other two save, so ONE ammo drains per swing. The port's per-shot
        // `ApplyItemTime` re-arm (`Player.cs:47184-47193`) is what makes the
        // other two shots happen at all.
        let mut app = shooter_rig(434, "Clockwork Assault Rifle", &[(0, 97, 99)]);
        let screen = app.screen_size().into();
        let mut log = Vec::new();
        hold(&mut app, screen, 20, &mut log);
        // The three shot ticks — the vanilla's 11/7/3, asserted rather than
        // eyeballed (the phase's core risk is exactly this alignment).
        assert_eq!(
            shot_ticks(&log),
            vec![11, 7, 3],
            "the Clockwork's shot ticks"
        );
        assert_eq!(log.len(), 3, "3 shots per swing");
        assert_eq!(shots(&app), 3, "3 musket balls spawned");
        assert_eq!(
            app.projectiles
                .slots()
                .iter()
                .find(|p| p.active)
                .unwrap()
                .kind,
            14,
            "the Musket Ball — the ammo's own Item.shoot"
        );
        assert_eq!(app.equip.ammo[0].count, 98, "exactly one ammo drained");
    }

    #[test]
    fn test_DW_4_1_the_fairy_queen_drains_eleven_of_fifteen_shots() {
        // `Player.cs:53635`: `itemAnimation > sItem.useAnimation - 8` — with the
        // Fairy Queen's anim 30 / useTime 2 the window is the ticks 30/28/26/24
        // (the first eight), so FOUR of the 15 shots save and ELEVEN drain.
        // The style-5 `num` bump (`ApplyItemAnimation`, `:4466-4469`) makes
        // `itemAnimationMax` 31, so the volley runs at 30/28/…/2 — the window
        // itself still reads the INSTANCE `useAnimation` 30 (boundary 22).
        let mut app = shooter_rig(4953, "Fairy Queen Ranged Item", &[(0, 40, 99)]);
        let screen = app.screen_size().into();
        let mut log = Vec::new();
        hold(&mut app, screen, 34, &mut log);
        assert_eq!(
            shot_ticks(&log),
            (2..=30).rev().step_by(2).collect::<Vec<_>>(),
            "15 shots at 30/28/…/2 — the vanilla's `itemAnimation` values"
        );
        assert_eq!(app.equip.ammo[0].count, 99 - 11, "eleven ammo drained");
        // The four saved shots sit exactly inside the window, the eleven
        // drained ones outside it — the window's boundary is 30 - 8 = 22.
        let inside: Vec<_> = shot_ticks(&log).into_iter().filter(|&t| t > 22).collect();
        assert_eq!(inside, vec![30, 28, 26, 24], "the free shots");
        assert_eq!(
            app.projectiles
                .slots()
                .iter()
                .find(|p| p.active)
                .map(|p| p.kind),
            Some(932),
            "the Fairy Queen's shot (`:53503-53506`, the wooden-arrow default)"
        );
    }

    #[test]
    fn test_DW_4_1_the_fairy_queen_window_edges_are_tick_exact() {
        // The window's two edges, asserted at the gate rather than inferred: a
        // whole swing's worth of shots, each one stamped with the tick the wire
        // derived it from, must partition into "saved" (inside the window) and
        // "drained" (outside) at exactly `remaining > 22`. One tick either way
        // and the drained count changes — the phase's determinism risk.
        let mut app = shooter_rig(4953, "Fairy Queen Ranged Item", &[(0, 40, 99)]);
        let screen = app.screen_size().into();
        let mut log = Vec::new();
        hold(&mut app, screen, 34, &mut log);
        // The vanilla's per-shot save pattern, re-derived from the C# order:
        // the 4953 arm has no `Main.rand` (`Player.cs:53635`), so the four
        // window shots save with certainty and the rest drain.
        assert_eq!(log.len(), 15, "the volley's shot count");
        assert_eq!(log.iter().filter(|&&(t, _)| t > 22).count(), 4);
        assert_eq!(log.iter().filter(|&&(t, _)| t <= 22).count(), 11);
        assert_eq!(99 - app.equip.ammo[0].count, 11);
    }

    /// S1-C-4 — `Player.GetWeaponDamage` (`:53427`) truncates the weapon's own
    /// part `(int)(damage * multiplier + 5E-06f)` BEFORE the ammo fuse adds its
    /// separately-truncated `(int)(num * multiplier)` (`:53552-53555`), and the
    /// multiplier is the ranged dispatch (`:53396-53398`): a bullet weapon with
    /// a fractional `rangedDamage` shows both truncations in the fused shot.
    #[test]
    fn s1_c_4_the_weapon_part_truncates_before_the_ammo_fuse() {
        let mut app = shooter_rig(434, "Clockwork Assault Rifle", &[(0, 97, 99)]);
        // The fractional multiplier the accessories would build (the arming
        // sites' stand-in — the dispatch test owns the arms).
        app.player.as_mut().unwrap().ranged_damage = 1.4;
        let screen = app.screen_size().into();
        let mut log = Vec::new();
        hold(&mut app, screen, 12, &mut log);
        let fired: Vec<_> = app
            .projectiles
            .slots()
            .iter()
            .filter(|p| p.active)
            .collect();
        assert!(fired.len() >= 1, "the volley fired ({})", fired.len());
        let wd = registry::get(434).unwrap().damage as f32;
        let ad = registry::get(97).unwrap().damage as f32;
        let weapon_part = (wd * 1.4 + 5e-6) as i32;
        let ammo_part = (ad * 1.4) as i32;
        assert_eq!(
            fired[0].damage,
            weapon_part + ammo_part,
            "the two truncations land before the add ({wd}*1.4 = {weapon_part}, {ad}*1.4 = {ammo_part})"
        );
    }

    #[test]
    fn test_DW_4_2_the_gun_prog_four_fire_their_fixed_projectile_and_drain_nothing() {
        // `ItemID.cs:1327` + `Player.cs:47071-47078` / `:48986-49018`: the four
        // endgame shooters require their ammo, spend no roll draws on the drain
        // (`dontConsume` seeds the chain true — their own rollers still spend
        // theirs), fire a FIXED gun projectile and carry the weapon's own RAW
        // damage/speed/knockback, the ammo fuse discarded. Their registry rows
        // are placeholders (`use_ammo: 0`), so the wire keys their vanilla data
        // on the item id.
        let cases: [(u16, &str, i32, u16, i32, f32, f32); 4] = [
            (3475, "Vortex Beater", 97, 615, 50, 20.0, 2.0),
            (3540, "Phantasm", 40, 630, 50, 20.0, 2.0),
            (3854, "DD2 Phoenix Bow", 40, 705, 32, 20.0, 2.0),
            (3930, "Celeb2", 771, 714, 50, 17.0, 10.0),
        ];
        for (kind, name, use_ammo, projectile, damage, speed, knock_back) in cases {
            let mut app = shooter_rig(kind, name, &[(0, use_ammo as u16, 99)]);
            let screen = app.screen_size().into();
            let mut log = Vec::new();
            hold(&mut app, screen, 30, &mut log);
            let fired: Vec<_> = app
                .projectiles
                .slots()
                .iter()
                .filter(|p| p.active)
                .collect();
            assert_eq!(fired.len(), 1, "{name}: one shot per swing");
            assert_eq!(
                fired[0].kind, projectile,
                "{name}: the fixed gun projectile"
            );
            assert_eq!(
                fired[0].damage, damage,
                "{name}: the weapon's own raw damage (the fuse discarded)"
            );
            assert!(
                fired[0].vel.x > 0.0 && (fired[0].vel.length() - speed).abs() < 0.001,
                "{name}: the weapon's own raw shootSpeed ({speed}) at the cursor's \
                 direction — got {:?}",
                fired[0].vel
            );
            assert_eq!(
                fired[0].knock_back, knock_back,
                "{name}: the weapon's own raw knockback"
            );
            assert_eq!(
                app.equip.ammo[0].count, 99,
                "{name}: `dontConsume` — no ammo drained"
            );
        }
    }

    #[test]
    fn test_DW_4_2_the_gun_prog_four_stay_dry_without_ammo() {
        // `HasAmmo` (`Player.cs:53431-53446`, `:52316-52319`) refuses the use
        // start without ammo — the gunProj reset never runs (S1-P-i15: the
        // old "the swing plays dry" wording pinned the pre-HasAmmo shape).
        // No shot, no drain, no panic either way.
        for (kind, name) in [
            (3475u16, "Vortex Beater"),
            (3540, "Phantasm"),
            (3854, "DD2 Phoenix Bow"),
            (3930, "Celeb2"),
        ] {
            let mut app = shooter_rig(kind, name, &[]);
            let screen = app.screen_size().into();
            let mut log = Vec::new();
            hold(&mut app, screen, 30, &mut log);
            assert!(log.is_empty(), "{name}: dry — nothing fired");
            assert_eq!(shots(&app), 0, "{name}: nothing spawned");
        }
    }

    /// S1-P-i13 retry — the two `gunProj` BOWS take the bow launch geometry.
    /// `ItemCheck_Shoot` runs the shared `RotatedRelativePoint(MountedCenter)`
    /// anchor (`Player.cs:47196`) + the `ChangeDir` aim-dot flip
    /// (`:47202-47224`) + the aim normalized from the ANCHOR (`:47277-47307`)
    /// for EVERY shooter before the per-item arms; the Phantasm 3540's arm
    /// fires the fixed 630 at that anchor with `num4/num5` (`:48996-48999`),
    /// the DD2 Phoenix Bow 3854's the fixed 705 (`:49014-49017`). The
    /// pre-i13 wire spawned them at the invented `center + dir*10` with no
    /// facing flip (the i01 census U24/U25) — the retry routes them through
    /// `bow_launch::fire`'s generic single tail instead. For the rig's
    /// stationary unmounted player the rotated anchor == `center()`
    /// (`fullRotation = 0`, `gfxOffY = 0`, `mount.PlayerOffset = 0`), so the
    /// shot sits AT the centre — not +10 px along the aim — and the facing
    /// flips toward the cursor.
    #[test]
    fn s1_p_i13_the_gun_prog_bows_fire_from_the_rotated_anchor_with_the_facing_flip() {
        for (kind, name, bolt) in [
            (3540u16, "Phantasm", 630u16),
            (3854, "DD2 Phoenix Bow", 705),
        ] {
            // Cursor LEFT of the player: the aim dot is negative, so
            // `ChangeDir(-1)` (`Player.cs:47216-47223`). −40 px keeps the
            // cursor tile inside the rig's 6-tile reach gate (the player
            // centre sits 50 px right of the camera).
            let mut app = shooter_rig(kind, name, &[(0, 40, 99)]);
            app.player.as_mut().unwrap().facing = 1;
            let (sw, sh) = app.screen_size();
            app.mouse_pos = Vec2::new(sw * 0.5 - 40.0, sh * 0.5);
            let screen = Vec2::new(sw, sh);
            let mut log = Vec::new();
            hold(&mut app, screen, 30, &mut log);
            let fired: Vec<_> = app
                .projectiles
                .slots()
                .iter()
                .filter(|p| p.active)
                .collect();
            assert_eq!(fired.len(), 1, "{name}: one fixed bolt per swing");
            assert_eq!(fired[0].kind, bolt, "{name}: the fixed gun projectile");
            assert_eq!(
                app.player.as_ref().unwrap().facing,
                -1,
                "{name}: the launch flips the facing toward the cursor (`:47214-47224`)"
            );
            // The anchor: `RotatedRelativePoint(MountedCenter)` folds to the
            // body centre for the rig's player — the invented +10 px offset
            // is gone. The pre-shot centre is captured from the rig (the
            // player never moves in the shooter wire).
            let (cx, cy) = app.player.as_ref().unwrap().center();
            assert!(
                (fired[0].pos.x - cx).abs() < 1e-3 && (fired[0].pos.y - cy).abs() < 1e-3,
                "{name}: the bolt spawns at the rotated anchor, not centre+10 ({:?} vs ({cx},{cy}))",
                fired[0].pos
            );
            assert!(fired[0].vel.x < 0.0, "{name}: the aim flies left");
            assert!(
                (fired[0].vel.length() - 20.0).abs() < 1e-3,
                "{name}: the gunProj reset's raw shootSpeed (`:47077-47082`)"
            );
            assert_eq!(
                app.equip.ammo[0].count, 99,
                "{name}: `dontConsume` — the fixed reset drains nothing"
            );
        }
    }

    #[test]
    fn s1_p_i15_the_endless_quiver_fires_without_draining() {
        // `Player.cs:53675`: `if (!flag && item.consumable)` — the Endless
        // Quiver 3103 (`consumable: false`, `Item.cs:28700-28711`, `ammo:
        // Arrow`) is picked and fired but its stack never moves. The wire's
        // old `if !drained { return; }` refused the shot outright (S1-P-i15
        // fixed it): the shot must SPAWN and the stack must stay 1.
        let mut app = shooter_rig(435, "Cobalt Repeater", &[(0, 3103, 1)]);
        let screen = app.screen_size().into();
        let mut log = Vec::new();
        hold(&mut app, screen, 26, &mut log);
        assert_eq!(log.len(), 1, "the Endless Quiver fired");
        assert_eq!(shots(&app), 1, "the wooden arrow spawned (proj 1)");
        assert_eq!(
            app.equip.ammo[0].count, 1,
            "`item.consumable` false — the stack never drains"
        );
        assert_eq!(app.equip.ammo[0].kind, 3103, "no TurnToAir");
        // The Endless Musket Pouch 3104 (`ammo: Bullet`) is the gun twin —
        // the Clockwork 434's FIRST shot always attempts the drain (its
        // window arm saves only the later volley shots, `Player.cs:53631`),
        // so the non-consumable gate is exercised deterministically.
        let mut app = shooter_rig(434, "Clockwork Assault Rifle", &[(0, 3104, 1)]);
        let screen = app.screen_size().into();
        let mut log = Vec::new();
        hold(&mut app, screen, 14, &mut log);
        assert_eq!(log.len(), 3, "the whole 3-shot volley fired");
        assert_eq!(app.equip.ammo[0].count, 1, "the pouch never drains");
    }

    #[test]
    fn s1_p_i15_has_ammo_refuses_the_use_start_without_a_stack() {
        // `Player.HasAmmo` (`Player.cs:53431-53446`) inside
        // `ItemCheck_CheckCanUse_Inner` (`:52316-52319`): no matching stack
        // anywhere → the use is refused, no swing arms. A grid-only wrong
        // ammo kind (bullets for a bow) is the same refusal as an empty
        // inventory — the scan is over ALL three arrays.
        let mut app = shooter_rig(435, "Cobalt Repeater", &[(20, 97, 99)]);
        let rng_before = app.rngs.stream(crate::app::RngKey::UpdatePlayers).clone();
        let screen = app.screen_size().into();
        update_editing(&mut app, ONE_TICK_SECS, screen);
        assert_eq!(app.swing, 0.0, "the wrong ammo kind refuses the use");
        let mut log = Vec::new();
        hold(&mut app, screen, 30, &mut log);
        assert!(log.is_empty(), "nothing fired");
        assert_eq!(
            *app.rngs.stream(crate::app::RngKey::UpdatePlayers),
            rng_before,
            "no draw spent on a refused use"
        );
        // Add a matching arrow stack: the SAME rig now starts and fires.
        app.equip.ammo[0] = crate::item::ItemSlot {
            kind: 40,
            name: "Wooden Arrow",
            count: 5,
            prefix: 0,
        };
        repress(&mut app);
        let mut log = Vec::new();
        hold(&mut app, screen, 26, &mut log);
        assert_eq!(log.len(), 1, "with ammo the use starts and fires");
    }

    #[test]
    fn s1_p_i15_the_nanite_bullet_scales_the_fused_damage_once() {
        // `Player.cs:53552-53559`: `Damage += (int)(num * mult)` THEN
        // `Damage = (int)(Damage * 1.25f)` — ONE scale on the FUSED value.
        // The old port scaled the ammo half in the resolver AND the fused
        // value at the wire (×1.5625 on the ammo half — i03 census A1).
        // Proj 285 has no `proj_data` row yet (M4-PROJ), so the spawn
        // declines and the fused damage is read off the resolver + the
        // wire's formula directly: the Musket 31 + the Nano Bullet 10 = 41,
        // `(int)(41 * 1.25)` = 51 — NOT 31 + 12 = 43 (the double-scale).
        let mut app = shooter_rig(96, "Musket", &[(0, 1350, 9)]);
        let screen = app.screen_size().into();
        let mut log = Vec::new();
        // The Musket is not autoReuse: one idle frame arms the press edge.
        app.mine_down = false;
        hold(&mut app, screen, 1, &mut log);
        app.mine_down = true;
        hold(&mut app, screen, 34, &mut log);
        assert_eq!(app.equip.ammo[0].count, 8, "the Nano Bullet drained");
        let resolved = crate::ammo::resolve_projectile(
            96,
            1350,
            97,
            10,
            4.6,
            3.6,
            10,
            9.0,
            96,
            &crate::ammo::AmmoFlags::default(),
        )
        .expect("the pair resolves");
        assert_eq!(resolved.projectile, 285);
        assert_eq!(resolved.damage, 10, "the resolver returns the unscaled num");
        let fused = 31 + resolved.damage;
        assert_eq!((fused as f32 * 1.25) as i32, 51, "one fused ×1.25");
    }

    #[test]
    fn s1_p_i15_the_remix_snowball_cannon_saves_half_its_ammo() {
        // `Player.cs:53659-53661`: on a REMIX world the Snowball Cannon 1319
        // saves 1/2 its snowballs; on a plain world it drains every shot. The
        // gate is `World.seeds` (`SeedFlags::REMIX`), threaded through
        // `VolleyWindows::remix_world`. Both rigs share one seed, so the
        // remix run's drain count must equal the mirrored `Next(2) == 0`
        // stream's non-saves — the gate's observable, seed-robust.
        let run = |remix: bool| {
            let mut app = shooter_rig(1319, "Snowball Cannon", &[(0, 949, 999)]);
            if remix {
                app.world.seeds |= tr_core::seeds::SeedFlags::REMIX;
            }
            *app.rngs.stream(crate::app::RngKey::UpdatePlayers) =
                tr_core::UnifiedRandom::new(0x5190_0519);
            let screen = app.screen_size().into();
            let mut log = Vec::new();
            for _ in 0..60 {
                hold(&mut app, screen, 21, &mut log);
                repress(&mut app);
            }
            assert_eq!(log.len(), 60, "60 swings fired (remix {remix})");
            999 - app.equip.ammo[0].count
        };
        let plain = run(false);
        let remix = run(true);
        assert_eq!(plain, 60, "a plain world drains every snowball");
        // The mirror: per shot the remix arm's `Next(2)` (save on 0) is the
        // shooter's ONLY `UpdatePlayers` draw — the crit rolls per-hit on
        // `UpdateProjectiles` (S1-P-i11's spawn→hit move; the gate dropped
        // the wire's old throw-time `crit_roll` from this mirror).
        let mut mirror = tr_core::UnifiedRandom::new(0x5190_0519);
        let drains = (0..60)
            .filter(|_| {
                let saved = mirror.next_max(2) == 0;
                !saved
            })
            .count();
        assert_eq!(
            remix as usize, drains,
            "the remix drain count is the mirrored Next(2) stream's"
        );
        assert!(remix < 60, "the remix arm saved at least once");
    }

    #[test]
    fn test_DW_4_3_the_ammo_slots_mode_picks_alternately_behind_the_offset() {
        // `Player.cs:53685-53725` + `:53569-53577`: with a cycling mode granted
        // the pick rotates behind the offset, and every successful pick without
        // `dontConsume` advances it. Two arrow stacks in the ammo slots: the
        // picks alternate 0/1/0/1 as the offset grows and wraps modulo the
        // count — the drain shows which stack was picked.
        let mut app = shooter_rig(435, "Cobalt Repeater", &[(0, 40, 12), (1, 40, 12)]);
        app.equip.misc_equips[0] = crate::item::ItemSlot {
            kind: 6168, // the Ammo Cycler → AmmoSlots (`Player.cs:14528-14544`)
            name: "Ammo Cycler",
            count: 1,
            prefix: 0,
        };
        let screen = app.screen_size().into();
        let mut log = Vec::new();
        for _ in 0..4 {
            mastery_tick(&mut app);
            hold(&mut app, screen, 26, &mut log);
            repress(&mut app);
        }
        assert_eq!(log.len(), 4, "one shot per swing (useTime == useAnimation)");
        assert_eq!(
            (app.equip.ammo[0].count, app.equip.ammo[1].count),
            (12 - 2, 12 - 2),
            "the picks alternated: two drains each"
        );
    }

    #[test]
    fn test_DW_4_3_the_cycling_cooldown_expires_and_resets_the_offset() {
        // `Player.cs:53569-53575` + `Update_AmmoCycler` (`:28419-28426`): each
        // successful non-dontConsume pick arms `cooldown = useAnimation + 120`
        // and advances the offset; the per-tick pass decrements the cooldown and
        // resets the offset at zero — so an idle cycler returns to the first
        // stack instead of freezing mid-rotation.
        let mut app = shooter_rig(435, "Cobalt Repeater", &[(0, 40, 12), (1, 40, 12)]);
        app.equip.misc_equips[0] = crate::item::ItemSlot {
            kind: 6168,
            name: "Ammo Cycler",
            count: 1,
            prefix: 0,
        };
        let screen = app.screen_size().into();
        let mut log = Vec::new();
        mastery_tick(&mut app);
        hold(&mut app, screen, 26, &mut log);
        // The Cobalt Repeater's `useAnimation` 23 (`itemdata.rs` — the vanilla's
        // `Item.cs:6488-6505`), so the cycler's cooldown is 23 + 120 ticks.
        {
            let p = app.player.as_ref().unwrap();
            assert_eq!(p.ammo_cycling.cooldown, 143, "useAnimation + 120");
            assert_eq!(p.ammo_cycling.offset, 1, "the pick advanced it");
        }
        // Still counting: the offset holds while the cooldown runs out.
        for _ in 0..142 {
            app.player.as_mut().unwrap().ammo_cycling.tick();
        }
        {
            let p = app.player.as_ref().unwrap();
            assert_eq!((p.ammo_cycling.cooldown, p.ammo_cycling.offset), (1, 1));
        }
        // The expiring tick resets the offset (`Player.cs:28419-28426`).
        app.player.as_mut().unwrap().ammo_cycling.tick();
        {
            let p = app.player.as_ref().unwrap();
            assert_eq!((p.ammo_cycling.cooldown, p.ammo_cycling.offset), (0, 0));
        }
        // The reset pick therefore lands back on the FIRST stack: without the
        // reset the offset 1 would have carried the pick to the second stack
        // (`11 / 11`), so `10 / 12` — the first stack twice — is the proof.
        mastery_tick(&mut app);
        repress(&mut app);
        hold(&mut app, screen, 26, &mut log);
        assert_eq!(
            (app.equip.ammo[0].count, app.equip.ammo[1].count),
            (12 - 2, 12 - 0),
            "after the reset the first stack drains again"
        );
    }

    #[test]
    fn test_DW_4_3_the_clentaminator_always_picks_the_first_stack() {
        // `PickAmmo_IterateRange`'s Clentaminator override (`Player.cs:53706-53709`):
        // 779/5134 force `num = 0` regardless of the mode and the offset, so
        // their sprays always come out of the first matching stack. The offset
        // here is armed to a value that WOULD rotate a normal weapon to the
        // second stack (`5 % 2 == 1`).
        // The Clentaminator's own volley is anim 30 / useTime 5 (`Item.cs:10501-10516`
        // → `itemdata.rs`): the style-5 `num` bump makes the max 31, so 6 shots
        // at the ticks 30/25/20/15/10/5, of which the first consumes —
        // `:53667`'s `itemAnimation < itemAnimationMax - useTime` window is
        // FALSE on the first shot (30 < 26 is false) and true for every later
        // one — so exactly ONE solution drains per swing. The
        // picked stack, not the drain count, is what this case pins: the
        // resolved solution projectile 145 has no `proj_data` row yet (M4-PROJ's
        // AI), so the spray's shots are refused at the spawn and the drain is
        // the only observable.
        for (kind, name, accessory) in [
            (779u16, "Clentaminator", 6168u16),
            (779, "Clentaminator", 6193), // the Random cycler
            (5134, "Clentaminator2", 6168),
        ] {
            let mut app = shooter_rig(kind, name, &[(0, 780, 9), (1, 780, 9)]);
            app.equip.misc_equips[0] = crate::item::ItemSlot {
                kind: accessory,
                name: "Ammo Cycler",
                count: 1,
                prefix: 0,
            };
            let screen = app.screen_size().into();
            let mut log = Vec::new();
            for _ in 0..3 {
                // The offset re-armed before every swing: a value that WOULD rotate
                // a normal weapon to the second stack (`5 % 2 == 1`).
                mastery_tick(&mut app);
                app.player.as_mut().unwrap().ammo_cycling.offset = 5;
                repress(&mut app);
                hold(&mut app, screen, 34, &mut log);
            }
            // The second stack is NEVER picked — the override forces index 0. Only
            // the first stack's count varies, because the Clentaminator2 (5134)
            // saves its own first shots with a 1/3 roll (`Player.cs:53591`, its own
            // arm) while 779 drains on every swing's first shot.
            assert_eq!(
                app.equip.ammo[1].count, 9,
                "{name} (cycler {accessory}): the second stack is never picked"
            );
            assert!(
                app.equip.ammo[0].count < 9,
                "{name} (cycler {accessory}): at least one drain came out of the \
             first stack, offset 5 notwithstanding"
            );
        }
    }

    #[test]
    fn test_DW_4_3_the_cycling_pick_reads_the_fixed_step_grant() {
        // The grant is per-tick (`Player.ResetEffects` `:18479` /
        // `UpdateEquips` `:14528-14544`): the pick rotates only once the tick's
        // scan has granted the mode, which is why the wire reads the mode off
        // the Player instead of caching it. The offset is armed by hand here so
        // the two reads differ observably with ONE pick each.
        let mut app = shooter_rig(435, "Cobalt Repeater", &[(0, 40, 12), (1, 40, 12)]);
        app.equip.misc_equips[0] = crate::item::ItemSlot {
            kind: 6168,
            name: "Ammo Cycler",
            count: 1,
            prefix: 0,
        };
        let screen = app.screen_size().into();
        // No `mastery_tick` and the offset armed to 1: the accessory is equipped
        // but the tick's scan has not run, so the mode is the `None` default and
        // the pick ignores the offset — the first stack.
        app.player.as_mut().unwrap().ammo_cycling.offset = 1;
        let mut log = Vec::new();
        hold(&mut app, screen, 26, &mut log);
        assert_eq!(
            (app.equip.ammo[0].count, app.equip.ammo[1].count),
            (11, 12),
            "no grant yet → the first matching stack, offset ignored"
        );
        // The tick's scan grants the mode and the SAME armed offset now rotates
        // the pick to the second stack.
        mastery_tick(&mut app);
        repress(&mut app);
        hold(&mut app, screen, 26, &mut log);
        assert_eq!(
            (app.equip.ammo[0].count, app.equip.ammo[1].count),
            (11, 11),
            "the granted mode rotates the pick behind the offset"
        );
    }

    #[test]
    fn test_DW_4_3_a_random_cycling_pick_spends_one_extra_draw() {
        // The determinism invariant, pinned on the stream: the `Random` mode's
        // `ammoCyclingOffset = Main.rand.Next()` (`Player.cs:53577`) is ONE extra
        // draw on the `UpdatePlayers` stream (the C# updates the cycler inside
        // `PickAmmo`). The Cobalt Repeater carries no roll arm of its own and the
        // ammo has no flags. The crit does NOT roll at spawn — it is per-hit in
        // `Damage_PVE_Inner` on the `UpdateProjectiles` stream
        // (`Projectile.cs:12883-12917`, S1-P-i11), so the plain shot spends ZERO
        // `UpdatePlayers` draws and the cycler's shot spends exactly its one.
        // Asserted against a helper that advances the same stream
        // (`draws_after` re-pinned from the splitmix rounds to stream draws,
        // DW-4.4, same count contract), so a later refactor cannot silently
        // add or drop a draw.
        let draws_after = |seed: i32, n: usize| {
            let mut rng = tr_core::UnifiedRandom::new(seed);
            for _ in 0..n {
                rng.next();
            }
            rng
        };
        let stream = |app: &mut App| app.rngs.stream(crate::app::RngKey::UpdatePlayers).clone();
        // The same shot without the cycler: no spawn-time draw at all.
        let mut plain = shooter_rig(435, "Cobalt Repeater", &[(0, 40, 12)]);
        let seed = 0x5EED_CA5Eu64 as i32;
        *plain.rngs.stream(crate::app::RngKey::UpdatePlayers) = tr_core::UnifiedRandom::new(seed);
        let screen = plain.screen_size().into();
        let mut log = Vec::new();
        hold(&mut plain, screen, 26, &mut log);
        assert_eq!(log.len(), 1, "one shot");
        assert_eq!(
            stream(&mut plain),
            draws_after(seed, 0),
            "the plain shot spends no UpdatePlayers draw (crit is per-hit)"
        );
        // With the Random cycler: the cycler's draw is the only one.
        let mut random = shooter_rig(435, "Cobalt Repeater", &[(0, 40, 12)]);
        random.equip.misc_equips[0] = crate::item::ItemSlot {
            kind: 6193, // the Ammo Cycler (Random) (`Player.cs:14538`)
            name: "Ammo Cycler",
            count: 1,
            prefix: 0,
        };
        *random.rngs.stream(crate::app::RngKey::UpdatePlayers) =
            tr_core::UnifiedRandom::new(seed);
        let screen = random.screen_size().into();
        mastery_tick(&mut random);
        let mut log = Vec::new();
        hold(&mut random, screen, 26, &mut log);
        assert_eq!(log.len(), 1, "one shot");
        assert_eq!(
            stream(&mut random),
            draws_after(seed, 1),
            "the Random cycler adds exactly one draw (the only spawn-time draw)"
        );
        // The offset is that first draw (`UnifiedRandom::next` is already
        // .NET's non-negative `Random.Next()`).
        let mut first_draw = tr_core::UnifiedRandom::new(seed);
        let expected_offset = first_draw.next();
        assert_eq!(
            random.player.as_ref().unwrap().ammo_cycling.offset,
            expected_offset,
            "the offset is the cycler's first draw"
        );
    }

    // ---- the dirty test-plan rows -----------------------------------------

    #[test]
    fn test_dirty_a_volley_swing_without_ammo_stays_dry_and_spends_nothing() {
        // A volley shooter with no ammo: `HasAmmo` (`Player.cs:53431-53446`,
        // called `:52316-52319` inside `ItemCheck_CheckCanUse_Inner`) refuses
        // the use START — no swing, no pick, no resolve, no roll, no crit, no
        // drain (S1-P-i15: the old "the swing still played out" pinned a
        // divergence). The RNG stream's immobility is the assertion (the
        // determinism invariant).
        let mut app = shooter_rig(434, "Clockwork Assault Rifle", &[]);
        let rng_before = app.rngs.stream(crate::app::RngKey::UpdatePlayers).clone();
        let screen = app.screen_size().into();
        let mut log = Vec::new();
        update_editing(&mut app, ONE_TICK_SECS, screen);
        assert_eq!(app.swing, 0.0, "the use is refused — no swing starts");
        hold(&mut app, screen, 30, &mut log);
        assert!(log.is_empty(), "nothing fired");
        assert_eq!(shots(&app), 0);
        assert_eq!(
            *app.rngs.stream(crate::app::RngKey::UpdatePlayers),
            rng_before,
            "the stream is untouched"
        );
    }

    #[test]
    fn test_dirty_a_random_cycling_pick_with_no_stacks_is_refused() {
        // `PickAmmo_IterateRange`'s `num2 == 0 → null` (`Player.cs:53702-53705`):
        // a cycler armed to Random with no matching stack anywhere refuses the
        // shot — no panic, no phantom pick, no drain, and NOT a zero-division
        // crash at `offset % count`.
        let mut app = shooter_rig(435, "Cobalt Repeater", &[]);
        app.equip.misc_equips[0] = crate::item::ItemSlot {
            kind: 6193, // the Ammo Cycler (Random) (`Player.cs:14538`)
            name: "Ammo Cycler",
            count: 1,
            prefix: 0,
        };
        mastery_tick(&mut app);
        let rng_before = app.rngs.stream(crate::app::RngKey::UpdatePlayers).clone();
        let screen = app.screen_size().into();
        let mut log = Vec::new();
        hold(&mut app, screen, 30, &mut log);
        assert!(log.is_empty(), "no shot");
        assert_eq!(
            *app.rngs.stream(crate::app::RngKey::UpdatePlayers),
            rng_before,
            "and no draw spent"
        );
    }

    #[test]
    fn test_dirty_a_resolved_kind_without_a_proj_data_row_refuses_the_shot() {
        // The `proj_data` trap `projectile.rs` documents: a kind with no row
        // makes `spawn_with_data` decline silently. The original rig (the
        // Cobalt Repeater + the Frostburn Arrow 988 → 172) LEFT this test
        // with S1-P-i13/i17 - the 172 now rows and the shot fires. The i16-era
        // Venom Bullet 283 trap ALSO left with S1-P-i18 (the 283 now rows).
        // Gate fix (M1 keep-both of the V1 + V5 re-points):
        //
        // Half 1 — the now-rowed 172 spawns AND drains (the i13 row landed):
        let mut app = shooter_rig(435, "Cobalt Repeater", &[(0, 988, 9)]);
        let screen = app.screen_size().into();
        let mut log = Vec::new();
        hold(&mut app, screen, 26, &mut log);
        assert!(shots(&app) >= 1, "the 172 row landed (i13) — the shot spawns");
        assert_eq!(app.equip.ammo[0].count, 8, "the picked arrow still drained");
        //
        // Half 2 — a genuinely row-less resolved kind declines the spawn while
        // the ammo drains: the Silver Bullet (278 → projectile 981,
        // `Projectile.cs:981`'s SetDefaults arm exists in C# but the port has
        // no 981 row yet). The Clockwork Assault Rifle (434, useAmmo 97) fires
        // it: the wire must refuse the shot without panicking, while the ammo
        // it already picked still drains (the drain runs before the spawn,
        // `Player.cs:53675`).
        let mut app2 = shooter_rig(434, "Clockwork Assault Rifle", &[(0, 278, 9)]);
        let mut log2 = Vec::new();
        hold(&mut app2, screen, 26, &mut log2);
        assert_eq!(shots(&app2), 0, "the spawn declined the unknown kind 981");
        assert_eq!(app2.equip.ammo[0].count, 8, "the picked bullet still drained");
    }

    #[test]
    fn test_dirty_the_acorn_slingshot_drains_past_the_ammo_box() {
        // `Player.cs:53599` (the Ammo Box's 1/5 save) vs `:53671` (the Acorn
        // Slingshot's `flag = false`, the chain's LAST arm): with the Ammo Box
        // buff on every swing drains exactly one acorn, no matter how many times
        // the box's roll would have saved. The wire's `ammo_flags` read is what
        // makes the box's roll reachable at all.
        // The Acorn Slingshot's resolved projectile 1088 (its own `Item.shoot`,
        // the Acorn carrying none) has a row (`Projectile.cs:10145-10149`), so
        // the ten shots fire AND the ten acorns drain — the spawn is not the
        // observable any more, the drain past the box's saves is.
        let mut app = shooter_rig(5629, "Acorn Slingshot", &[(0, 27, 12)]);
        app.player.as_mut().unwrap().buffs.add_buff(93, 3600);
        let screen = app.screen_size().into();
        let mut log = Vec::new();
        for _ in 0..10 {
            mastery_tick(&mut app);
            hold(&mut app, screen, 26, &mut log);
            repress(&mut app);
        }
        assert_eq!(shots(&app), 10, "the acorn spawns — its row exists");
        assert_eq!(
            app.equip.ammo[0].count, 2,
            "ten acorns drained — the box never saves for the Acorn Slingshot"
        );
    }

    // ---- S1-P-i16: the rocket launchers & dart guns ------------------------

    /// The Rocket Launcher + Rocket I resolve through the pair table to the
    /// 134 rocket (`AmmoID.cs:28`), the fused damage = the weapon's 55 + the
    /// rocket's 40 (both × the specialist multiplier 1.0 at the rig's
    /// defaults), the speed = the launcher's 5 + the rocket's 0, and the
    /// rocket KEEPS its damage in flight (`Projectile.cs:47175`'s
    /// keep-damage list — the pre-port `ai_bomb` zeroed it).
    #[test]
    fn s1_p_i16_the_rocket_launcher_fires_the_134_rocket_with_fused_stats() {
        let mut app = shooter_rig(759, "Rocket Launcher", &[(0, 771, 9)]);
        let screen = app.screen_size().into();
        let mut log = Vec::new();
        hold(&mut app, screen, 32, &mut log);
        assert_eq!(log.len(), 1, "one rocket per swing (useTime == useAnimation 30)");
        let fired: Vec<_> = app.projectiles.slots().iter().filter(|p| p.active).collect();
        assert_eq!(fired.len(), 1);
        assert_eq!(fired[0].kind, 134, "the pair table's 759 + 771 cell");
        assert_eq!(fired[0].damage, 95, "55 + 40, both × 1.0");
        assert!(
            (fired[0].vel.length() - 5.0).abs() < 0.001,
            "the launcher's shootSpeed 5 + the rocket's 0"
        );
        assert_eq!(app.equip.ammo[0].count, 8, "the rocket drained");
        // The flight keeps the damage: one pool tick must not zero it.
        let mut rng = tr_core::tile_light::FastRandom::new(1);
        app.projectiles.update(
            &mut app.world,
            &mut [],
            None,
            None,
            None,
            crate::projectile::YoyoInputs::default(),
            crate::projectile::FlailInputs::default(),
            &mut rng,
            0,
            false,
        );
        let rocket = app.projectiles.slots().iter().find(|p| p.active).unwrap();
        assert_eq!(rocket.damage, 95, ":47175 — the keep-damage list");
        // The ×1.1 speed-up below 15 (`:47364-47367`): 5 → 5.5.
        assert!(
            (rocket.vel.length() - 5.5).abs() < 0.01,
            "the straight-flight rocket accelerates"
        );
    }

    /// The Grenade Launcher's 133 grenade arcs (`Projectile.cs:47496-47506`):
    /// no gravity before ai[0] > 15, then +0.2/tick. The Proximity Mine
    /// Launcher's 135 mine settles (`:47483-47495`): +0.2 gravity × 0.97 drag
    /// from the first tick.
    #[test]
    fn s1_p_i16_the_grenade_arcs_and_the_mine_settles() {
        // The grenade: 15 straight ticks, then the arc.
        let mut app = shooter_rig(758, "Grenade Launcher", &[(0, 771, 9)]);
        let screen = app.screen_size().into();
        let mut log = Vec::new();
        hold(&mut app, screen, 22, &mut log);
        let fired: Vec<_> = app.projectiles.slots().iter().filter(|p| p.active).collect();
        assert_eq!(fired[0].kind, 133, "the pair table's 758 + 771 cell");
        let slot = app
            .projectiles
            .slots()
            .iter()
            .position(|p| p.active)
            .unwrap();
        let mut rng = tr_core::tile_light::FastRandom::new(1);
        for _ in 0..15 {
            app.projectiles.update(
                &mut app.world, &mut [], None, None, None,
                crate::projectile::YoyoInputs::default(),
                crate::projectile::FlailInputs::default(),
                &mut rng, 0, false,
            );
        }
        assert_eq!(
            app.projectiles.slots()[slot].vel.y, 0.0,
            ":47498 — no gravity before ai[0] > 15"
        );
        app.projectiles.update(
            &mut app.world, &mut [], None, None, None,
            crate::projectile::YoyoInputs::default(),
            crate::projectile::FlailInputs::default(),
            &mut rng, 0, false,
        );
        assert!(
            (app.projectiles.slots()[slot].vel.y - 0.2).abs() < 1e-5,
            ":47504 — the 16th tick adds 0.2"
        );
        // The mine: the settle runs from the first tick.
        let mut app2 = shooter_rig(760, "Proximity Mine Launcher", &[(0, 771, 9)]);
        let mut log2 = Vec::new();
        hold(&mut app2, screen, 52, &mut log2);
        let mine = app2.projectiles.slots().iter().find(|p| p.active).unwrap();
        assert_eq!(mine.kind, 135, "the pair table's 760 + 771 cell");
        let slot2 = app2.projectiles.slots().iter().position(|p| p.active).unwrap();
        let vy0 = mine.vel.y;
        let mut rng2 = tr_core::tile_light::FastRandom::new(1);
        app2.projectiles.update(
            &mut app2.world, &mut [], None, None, None,
            crate::projectile::YoyoInputs::default(),
            crate::projectile::FlailInputs::default(),
            &mut rng2, 0, false,
        );
        let vy1 = app2.projectiles.slots()[slot2].vel.y;
        assert!(
            (vy1 - (vy0 + 0.2) * 0.97).abs() < 1e-4,
            ":47485-47486 — the +0.2 gravity then the 0.97 drag"
        );
    }

    /// The Snowman Cannon's 338 rocket homes (`Projectile.cs:47270-47309`):
    /// the acquire opens at ai[0] > 30, the 1/12 lerp toward the nearest
    /// chaseable NPC at speed 16.
    #[test]
    fn s1_p_i16_the_snowman_rocket_homes_after_30_ticks() {
        let mut app = shooter_rig(1946, "Snowman Cannon", &[(0, 771, 9)]);
        let screen = app.screen_size().into();
        let mut log = Vec::new();
        hold(&mut app, screen, 17, &mut log);
        let fired: Vec<_> = app.projectiles.slots().iter().filter(|p| p.active).collect();
        assert_eq!(fired[0].kind, 338, "the pair table's 1946 + 771 cell");
        assert!(
            (fired[0].vel.length() - 15.0).abs() < 0.001,
            "the Snowman's shootSpeed 15"
        );
    }

    /// The Dart Pistol + Poison Dart: the 267 dart spawns (`Item.shoot` arm
    /// `Player.cs:53491-53494`), the muzzle offset shifts the spawn −4·dir X
    /// / −2 Y (`:47272-47276`), the fused damage = 28 + 10.
    #[test]
    fn s1_p_i16_the_dart_pistol_fires_the_267_dart_from_its_offset_muzzle() {
        let mut app = shooter_rig(3007, "Dart Pistol", &[(0, 1310, 9)]);
        let screen = app.screen_size().into();
        let mut log = Vec::new();
        hold(&mut app, screen, 24, &mut log);
        assert_eq!(log.len(), 1);
        let fired: Vec<_> = app.projectiles.slots().iter().filter(|p| p.active).collect();
        assert_eq!(fired.len(), 1);
        assert_eq!(fired[0].kind, 267, "the Poison Dart's Item.shoot");
        assert_eq!(fired[0].damage, 38, "28 + 10");
        assert!(
            (fired[0].vel.length() - 15.0).abs() < 0.001,
            "the pistol's 13 + the dart's shootSpeed 2"
        );
        // The muzzle offset: the spawn sits 4 px LEFT of the centre + 10·dir
        // line and 2 px UP (the rig aims +x, facing +1).
        let (pcx, pcy) = app.player.as_ref().unwrap().center();
        assert!(
            (fired[0].pos.x - (pcx + 10.0 - 4.0)).abs() < 0.01,
            ":47274 — the −4·direction X offset"
        );
        assert!(
            (fired[0].pos.y - (pcy - 2.0)).abs() < 0.01,
            ":47275 — the −2·gravDir Y offset"
        );
        assert_eq!(app.equip.ammo[0].count, 8, "the dart drained");
    }

    /// The Dart Rifle + Crystal Dart: the 477 dart spawns with penetrate 5 /
    /// extraUpdates 1 (`Projectile.cs:5518-5528`), the fused damage 52 + 14.
    #[test]
    fn s1_p_i16_the_dart_rifle_fires_the_477_crystal_dart() {
        let mut app = shooter_rig(3008, "Dart Rifle", &[(0, 3009, 9)]);
        let screen = app.screen_size().into();
        let mut log = Vec::new();
        hold(&mut app, screen, 40, &mut log);
        assert_eq!(log.len(), 1);
        let fired: Vec<_> = app.projectiles.slots().iter().filter(|p| p.active).collect();
        assert_eq!(fired[0].kind, 477, "the Crystal Dart's Item.shoot");
        assert_eq!(fired[0].damage, 66, "52 + 14");
        assert_eq!(fired[0].penetrate, 5, ":5525");
        assert_eq!(fired[0].extra_updates, 1, ":5526");
        assert!(
            (fired[0].vel.length() - 15.5).abs() < 0.001,
            "the rifle's 14.5 + the dart's 1"
        );
    }

    /// The Electrosphere Launcher fires the FIXED 442 bolt
    /// (`Player.cs:47169-47172`) with the cursor tile as its ai[0]/ai[1]
    /// target (`:49388-49392`), NOT the PickAmmo additive arm's result.
    #[test]
    fn s1_p_i16_the_electrosphere_fires_the_442_bolt_at_the_cursor_tile() {
        let mut app = shooter_rig(2796, "Electrosphere Launcher", &[(0, 771, 9)]);
        // Gate fix: the Electrosphere Launcher is `autoReuse: false`
        // (`Item.cs:2796` sets none) — the shot needs the press edge
        // (`Player.cs:42733`). The rig's fresh state never arms it.
        app.mine_released = true;
        app.mine_prev_down = false;
        let screen = app.screen_size().into();
        let mut log = Vec::new();
        hold(&mut app, screen, 14, &mut log);
        assert_eq!(log.len(), 1);
        let fired: Vec<_> = app.projectiles.slots().iter().filter(|p| p.active).collect();
        assert_eq!(fired[0].kind, 442, ":47171 — the fixed bolt");
        // The cursor sits 80 px right of the screen centre; the target tile
        // is the cursor cell (`Player.tileTargetX/Y`).
        let (sw, sh) = app.screen_size();
        let world_pos = app.camera + (app.mouse_pos - Vec2::new(sw * 0.5, sh * 0.5)) / app.zoom;
        assert_eq!(fired[0].ai[0], (world_pos.x / TILE_PX).floor(), ":49390");
        assert_eq!(fired[0].ai[1], (world_pos.y / TILE_PX).floor(), ":49391");
        assert_eq!(fired[0].time_left, 600, ":5134");
    }

    /// The Fireworks Launcher fires TWO rockets per shot
    /// (`Player.cs:49019-49031`), each `167 + Next(4)` with ai1 = 1.
    #[test]
    fn s1_p_i16_the_fireworks_launcher_fires_two_rockets() {
        let mut app = shooter_rig(3546, "Fireworks Launcher", &[(0, 771, 9)]);
        let screen = app.screen_size().into();
        let mut log = Vec::new();
        hold(&mut app, screen, 32, &mut log);
        assert_eq!(log.len(), 1, "one USE per swing");
        let fired: Vec<_> = app.projectiles.slots().iter().filter(|p| p.active).collect();
        assert_eq!(fired.len(), 2, ":49021 — the two-rocket loop");
        for p in fired {
            assert!((167..=170).contains(&p.kind), "167 + Next(4)");
            assert_eq!(p.ai[1], 1.0, ":49028 — ai1: 1f");
            assert_eq!(p.time_left, 45, ":2565");
        }
        assert_eq!(app.equip.ammo[0].count, 8, "ONE rocket drained per use");
    }

    /// The Celebration Mk2 fires the FIXED 714 with `ai0 = 5 * Next(0, 20)`
    /// (`Player.cs:48991-48994`), the gunProj reset's own stats (50 damage /
    /// 17 speed / 10 kb, `Item.cs:34741-34748`), NOT the ammo fuse's.
    #[test]
    fn s1_p_i16_the_celebration_fires_the_714_with_the_ai0_draw() {
        let mut app = shooter_rig(3930, "Celebration Mk2", &[(0, 771, 9)]);
        let screen = app.screen_size().into();
        let mut log = Vec::new();
        hold(&mut app, screen, 8, &mut log);
        assert_eq!(log.len(), 1, "one shot (useTime == useAnimation 6)");
        let fired: Vec<_> = app.projectiles.slots().iter().filter(|p| p.active).collect();
        assert_eq!(fired.len(), 1);
        assert_eq!(fired[0].kind, 714, ":48993 — the fixed 714");
        assert_eq!(fired[0].damage, 50, "the gunProj reset's own damage");
        assert!(
            (fired[0].vel.length() - 17.0).abs() < 0.001,
            "the gunProj reset's shootSpeed 17"
        );
        // `ai0 = 5 * Next(0, 20)` — a multiple of 5 in [0, 95].
        let ai0 = fired[0].ai[0];
        assert!(ai0 >= 0.0 && ai0 <= 95.0, ":48993 — the ai0 range, got {ai0}");
        assert_eq!(ai0 % 5.0, 0.0, ":48993 — 5 × an integer");
    }

    /// The Proximity Mine Launcher caps its owner's mines at 20
    /// (`Player.cs:49450-49453` → `:46976-47017`): the 21st shot kills the
    /// oldest mine (the lowest timeLeft).
    #[test]
    fn s1_p_i16_the_proximity_mine_launcher_caps_at_20() {
        let mut app = shooter_rig(760, "Proximity Mine Launcher", &[(0, 771, 99)]);
        let screen = app.screen_size().into();
        // Seed 20 live mines directly (the pool's spawn seam), all the 135
        // kind, ASCENDING timeLeft so the oldest (the lowest) is slot 0 —
        // the C#'s strict `<` scan (`:47008`) keeps the first on ties.
        for i in 0..20 {
            let s = app
                .projectiles
                .spawn_with_data_owned(
                    135, 0, 200.0 + i as f32 * 20.0, 200.0, 0.0, 0.0, 80, 4.0, 0.0, 0.0,
                )
                .unwrap();
            app.projectiles.slots_mut()[s].time_left = 1000 + i as i32;
        }
        let mut log = Vec::new();
        // 51 frames: the useTime == useAnimation 50 gun fires once (the
        // press frame), the second would need frame 52.
        hold(&mut app, screen, 51, &mut log);
        assert_eq!(log.len(), 1, "the 21st mine fired");
        // The cap killed the oldest (slot 0, timeLeft 1000) — it detonates
        // (the `detonate` latch; the pool's next update resolves the blast).
        assert!(
            app.projectiles.slots()[0].detonated,
            "the oldest mine is detonating"
        );
        let live_mines = app
            .projectiles
            .slots()
            .iter()
            .filter(|p| p.active && p.kind == 135 && !p.detonated)
            .count();
        assert_eq!(live_mines, 20, "the cap holds at 20");
    }

    pub(super) fn mining_app(world: World, tx: i32, ty: i32) -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = world;
        let mut player = Player::spawn(&app.world);
        player.x = (tx + 3) as f32 * TILE_PX;
        player.y = ty as f32 * TILE_PX;
        app.player = Some(player);
        app.camera = Vec2::new(tx as f32 * TILE_PX + 8.0, ty as f32 * TILE_PX + 8.0);
        app.mouse_pos = Vec2::new(640.0, 360.0); // screen centre == the camera
        app.inventory[app.selected].kind = 1; // the Iron Pickaxe (pick 40)
        app.inventory[app.selected].count = 1;
        app.mine_down = true;
        app
    }

    fn mine_frames(app: &mut App, n: usize) {
        for _ in 0..n {
            update_editing(app, 1.0 / 60.0, MINE_SCREEN);
        }
    }

    #[test]
    fn the_far_placement_power_widens_the_edit_reach() {
        // `FarPlacementRangePower` (`CreativePowers.cs:730-767`) doubles
        // `Player.tileRangeX/Y` (`Player.cs:18815-18828`); the reach gate in
        // `update_editing` tracks `tile_range_x` (its documented stand-in for
        // `TileReachCheckSettings.Simple`).
        let mut w = World::new("t".into(), 64, 800);
        for x in 0..64 {
            let i = w.index(x, 260);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        w.compute_column_surface();
        let mut app = mining_app(w, 30, 260);
        // The target 13 tiles left of the player: outside the base 6-tile
        // reach, inside the doubled 19.
        app.player.as_mut().unwrap().x = 17.0 * TILE_PX;
        update_editing(&mut app, 1.0 / 60.0, MINE_SCREEN);
        assert!(app.edit_target.is_none(), "out of the base reach");
        assert!(app.mine_target.is_none(), "and nothing is mined");
        app.player.as_mut().unwrap().far_placement_range = true;
        update_editing(&mut app, 1.0 / 60.0, MINE_SCREEN);
        assert_eq!(
            app.edit_target,
            Some((30, 260)),
            "the power widens the reach to the cursor's tile"
        );
    }

    // ---- S1-P-i29: the pickaxe/drill mining interaction --------------------
    //
    // The C# chain: `ItemCheck_UseMiningTools` (`Player.cs:45584`) →
    // `ItemCheck_UseMiningTools_ActuallyUseMiningTool` (`:45646`) → `PickTile`
    // (`:53757`) → `GetPickaxeDamage` (`:53902`). The damage accumulates in
    // the `HitTile` book (`HitTile.cs`) until 100 breaks the tile; the cadence
    // is `toolTime`/`itemTime` (`:43576-43587`, `:45779`). These witnesses pin
    // the accumulation, the cadence, the reach rectangle, the grass
    // conversion and the drill channel. WRITTEN, NOT RUN (code-only mode).

    /// A single-tile mining world: the target cell at (tx, ty), the player
    /// three tiles to its right (the `mining_app` shape), the Iron Pickaxe
    /// held. The world is 64 wide so the dungeon x-band / ore draws have room.
    fn pick_world_at(kind: u16, tx: i32, ty: i32) -> App {
        let mut w = World::new("pick".into(), 64, 800);
        let i = w.index(tx, ty);
        w.tiles[i] = Tile { kind, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.compute_column_surface();
        mining_app(w, tx, ty)
    }

    /// The `HitTile` damage at a cell (0 when no slot tracks it).
    fn hit_damage(app: &App, tx: i32, ty: i32) -> i32 {
        app.hit_tile
            .try_finding(tx, ty, tr_core::hit_tile::TILE)
            .map_or(0, |id| app.hit_tile.data[id].damage)
    }

    /// S1-P-i29 — the pick damage accumulates in the `HitTile` book and the
    /// tile breaks at 100 (`Player.PickTile` `Player.cs:53769`). The Iron
    /// Pickaxe (pick 40) on stone (kind 1, plain power): 40 + 40 + 40 = 120,
    /// so the third hit breaks it. The first two hits leave the tile standing
    /// with the damage in the book.
    #[test]
    fn s1_p_i29_pick_damage_accumulates_to_a_break() {
        let mut app = pick_world_at(1, 30, 260);
        // The cadence: useTime 13 ticks between hits (`Player.cs:45779`,
        // pickSpeed 1). The first hit lands a tick or two after the press
        // (the `toolTime` arm), the next 13 ticks later. 40 frames covers
        // three hits comfortably.
        mine_frames(&mut app, 2);
        // After the first hit the tile stands and the book carries 40.
        assert!(app.world.tile(30, 260).unwrap().active(), "stone stands after hit 1");
        assert_eq!(hit_damage(&app, 30, 260), 40, "the first hit's 40");
        mine_frames(&mut app, 13);
        assert!(app.world.tile(30, 260).unwrap().active(), "stone stands after hit 2");
        assert_eq!(hit_damage(&app, 30, 260), 80, "two hits accumulate to 80");
        mine_frames(&mut app, 13);
        assert!(!app.world.tile(30, 260).unwrap().active(), "the third hit breaks stone");
        // The break clears the hit slot (`ClearMiningCacheAt`, `:53773`).
        assert_eq!(hit_damage(&app, 30, 260), 0, "the break clears the book");
        // The stone item dropped (`break_tile`'s per-cell drop).
        assert!(app.items.iter().any(|it| it.kind == 3), "the stone item dropped");
    }

    /// S1-P-i29 — the mining cadence is `itemTime = (int)(useTime * pickSpeed)`
    /// (`Player.cs:45779`), NOT the old time-per-tile hardness. A faster
    /// pickSpeed (the Mining buff 104, `:10136`, −0.25) shortens the interval;
    /// the damage per hit is unchanged (the pick power, not the speed).
    #[test]
    fn s1_p_i29_the_mining_cadence_reads_use_time_and_pick_speed() {
        // The Iron Pickaxe: useTime 13, pick 40 on stone (plain power). With
        // pickSpeed 1 the hits are 13 ticks apart (first hit tick 2, then 15);
        // with the Mining buff (pickSpeed 0.75) they are (int)(13*0.75) = 9
        // ticks apart (first hit tick 2, then 11). Gate V3: measure the damage
        // in a window BEFORE the third hit breaks the stone (the break clears
        // the book, so a post-break read would show 0, not the cadence).
        let mut plain = pick_world_at(1, 30, 260);
        mine_frames(&mut plain, 14); // hits at tick 2 only (next at 15)
        let plain_dmg = hit_damage(&plain, 30, 260);
        let mut fast = pick_world_at(1, 30, 260);
        // The Mining buff's −0.25 pickSpeed fold (the C# UpdateBuffs arm
        // `:10134-10137`, the port's positive-bonus convention), set directly
        // so the player stays parked on the target.
        fast.player.as_mut().unwrap().pick_speed_bonus += 0.25;
        mine_frames(&mut fast, 14); // hits at tick 2 AND 11 (next at 20)
        let fast_dmg = hit_damage(&fast, 30, 260);
        // The faster pickSpeed lands MORE hits in the same window (each hit
        // is the same 40 damage). The plain cadence lands fewer.
        assert!(
            fast_dmg > plain_dmg,
            "the Mining buff's faster cadence lands more damage: fast {fast_dmg} vs plain {plain_dmg}"
        );
        // Both are multiples of the pick's 40 (the damage per hit is the
        // power, not the speed), and the stone still stands (no break yet).
        assert_eq!(plain_dmg % 40, 0, "plain damage is a multiple of 40");
        assert_eq!(fast_dmg % 40, 0, "fast damage is a multiple of 40");
        assert!(plain.world.tile(30, 260).unwrap().active(), "plain stone stands");
        assert!(fast.world.tile(30, 260).unwrap().active(), "fast stone stands");
    }

    /// S1-P-i29 — the reach is the C# rectangle (`TileReachCheckSettings.
    /// GetTileRegion`, `TileReachCheckSettings.cs:45-54`), not a circle: the
    /// X reach is `tileRangeX + tileBoost` (clamped 20), the Y reach
    /// `tileRangeY + tileBoost`. A target straight up beyond the Y reach is
    /// refused even though a circle of the X radius would reach it.
    #[test]
    fn s1_p_i29_the_reach_is_a_rectangle_not_a_circle() {
        // The player at (33, 260); the target 5 tiles ABOVE at (33, 255).
        // The base Y reach is 3 (+ the pick's tileBoost 0), so 5 up is out;
        // the X reach is 5, so a circle of radius 5 would reach it.
        let mut w = World::new("reach".into(), 64, 800);
        let i = w.index(33, 255);
        w.tiles[i] = Tile { kind: 1, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.compute_column_surface();
        let mut app = mining_app(w, 33, 255);
        // mining_app puts the player at (tx+3, ty) = (36, 255); move it to
        // (33, 260) so the target is 5 above and 0 across.
        app.player.as_mut().unwrap().x = 33.0 * TILE_PX;
        app.player.as_mut().unwrap().y = 260.0 * TILE_PX;
        app.camera = Vec2::new(33.0 * TILE_PX + 8.0, 255.0 * TILE_PX + 8.0);
        mine_frames(&mut app, 40);
        assert!(
            app.world.tile(33, 255).unwrap().active(),
            "5 tiles up is beyond the Y reach 3 — the rectangle refuses"
        );
        // The same target 5 tiles to the SIDE is within the X reach 5.
        let mut w2 = World::new("reach2".into(), 64, 800);
        let i2 = w2.index(38, 260);
        w2.tiles[i2] = Tile { kind: 1, ..Default::default() };
        w2.tiles[i2].flags |= FLAG_ACTIVE;
        w2.compute_column_surface();
        let mut app2 = mining_app(w2, 38, 260);
        app2.player.as_mut().unwrap().x = 33.0 * TILE_PX;
        app2.player.as_mut().unwrap().y = 260.0 * TILE_PX;
        app2.camera = Vec2::new(38.0 * TILE_PX + 8.0, 260.0 * TILE_PX + 8.0);
        mine_frames(&mut app2, 40);
        assert!(
            !app2.world.tile(38, 260).unwrap().active(),
            "5 tiles across is within the X reach 5 — the rectangle reaches"
        );
    }

    /// S1-P-i29 — the `tileBoost` widens the reach (`IsTargetTileInItemRange`,
    /// `Player.cs:46313-46316`). The Chlorophyte Pickaxe (1230, tileBoost 1)
    /// reaches one tile farther than the Iron Pickaxe (tileBoost 0).
    #[test]
    fn s1_p_i29_the_tile_boost_widens_the_reach() {
        // Gate V3: `tile_in_reach_rect` is the C# `GetTileRegion` rectangle
        // (`TileReachCheckSettings.cs:45-54`): HX = ceil((X+width)/16)−1+reach.
        // The player at x=33 tiles (width 20px) spans columns 33..34, so the
        // Iron Pickaxe's X reach 5 reaches column 39 (34+5); tileBoost 1
        // widens it to 40. Probe column 40: refused at tileBoost 0, reached
        // at tileBoost 1 (`IsTargetTileInItemRange`, `Player.cs:46313-46316`).
        let mut w = World::new("boost".into(), 64, 800);
        let i = w.index(40, 260);
        w.tiles[i] = Tile { kind: 1, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.compute_column_surface();
        let mut app = mining_app(w, 40, 260);
        app.player.as_mut().unwrap().x = 33.0 * TILE_PX;
        app.player.as_mut().unwrap().y = 260.0 * TILE_PX;
        app.camera = Vec2::new(40.0 * TILE_PX + 8.0, 260.0 * TILE_PX + 8.0);
        let (rx, ry) = mining_tile_range(&app);
        let iron_tb = registry::get(1).map_or(0, |d| d.tile_boost);
        assert!(
            !app.player.as_ref().is_some_and(|p| tile_in_reach_rect(p, 40, 260, rx, ry, iron_tb)),
            "tileBoost 0 refuses the cell past its X reach"
        );
        let chlo_tb = registry::get(1230).map_or(0, |d| d.tile_boost);
        assert_eq!(chlo_tb, iron_tb + 1, "the Chlorophyte Pickaxe adds one tile of boost");
        assert!(
            app.player.as_ref().is_some_and(|p| tile_in_reach_rect(p, 40, 260, rx, ry, chlo_tb)),
            "tileBoost 1 reaches the widened cell"
        );
    }

    /// S1-P-i29 — the grass conversion (`DoesPickTargetTransformOnKill`,
    /// `Player.cs:54047-54054` + `KillTile(fail: true)` `WorldGen.cs:63715-
    /// 63718`): a grass tile (kind 2) at 100 damage converts to dirt (kind 0)
    /// instead of breaking — no item drop, the tile stays active as dirt.
    #[test]
    fn s1_p_i29_grass_converts_to_dirt_instead_of_breaking() {
        let mut app = pick_world_at(2, 30, 260);
        // Dirt/grass takes the double pick-power add (`:53972`): the Iron
        // Pickaxe's 40 → 80 per hit. Two hits reach 160 >= 100 → the
        // conversion fires on the second.
        mine_frames(&mut app, 30);
        let t = app.world.tile(30, 260).unwrap();
        assert!(t.active(), "the grass converts, it does not break");
        assert_eq!(t.kind, 0, "grass 2 → dirt 0");
        // No dirt item dropped (the conversion is not a break).
        assert!(app.items.is_empty(), "the conversion drops nothing");
    }

    /// S1-P-i29 — the drill channel: a drill (the Cobalt Drill 385, channel
    /// true, pick 110, useTime 7) spawns its drill projectile on the press
    /// and mines continuously while held; the release kills the projectile.
    #[test]
    fn s1_p_i29_the_drill_channels_and_mines_continuously() {
        let mut app = pick_world_at(1, 30, 260);
        app.inventory[app.selected].kind = 385; // the Cobalt Drill
        // The press: the drill projectile spawns (kind 59, the Cobalt Drill's
        // shoot) and the channel latches.
        mine_frames(&mut app, 1);
        assert!(
            app.projectiles.slots().iter().any(|p| p.active && p.kind == 59),
            "the drill projectile spawned"
        );
        assert!(app.player.as_ref().unwrap().channel, "the channel latched");
        // The drill mines continuously: the Cobalt Drill's pick 110 on stone
        // (plain power) breaks it in one hit (110 >= 100). The channel holds
        // the animation so the hit lands within the useTime cadence.
        mine_frames(&mut app, 20);
        assert!(!app.world.tile(30, 260).unwrap().active(), "the drill broke the stone");
        // The release: the channel clears; the projectile's own Kill rides
        // the aiStyle-20 machine on the next POOL tick (the i30 chainsaw
        // precedent — `update_editing` alone never ticks the pool).
        app.mine_down = false;
        mine_frames(&mut app, 2);
        assert!(!app.player.as_ref().unwrap().channel, "the release clears the channel");
        crate::world_sim::projectiles::update_projectiles(&mut app, ONE_TICK_SECS);
        assert!(
            !app.projectiles.slots().iter().any(|p| p.active && p.kind == 59),
            "the release kills the drill projectile"
        );
    }

    /// S1-P-i29 — the `pickSpeed` floor (`Player.cs:25972-25975`): the buff
    /// discounts cannot push pickSpeed below 0.3. The Mining buff (−0.25)
    /// plus the Ancient Chisel (−0.25) plus WellFed (−0.15) sums to −0.65,
    /// but pickSpeed clamps at 0.3.
    #[test]
    fn s1_p_i29_the_pick_speed_floor_is_0_3() {
        let mut app = pick_world_at(1, 30, 260);
        // Gate V3: the buff→stat fold runs in `update_player` (the C#
        // `UpdateBuffs`), not in `add_buff` — drive one player tick so the
        // Mining/WellFed bonuses land before the accessor reads them.
        {
            let p = app.player.as_mut().unwrap();
            p.buffs.add_buff(104, 3600); // Mining −0.25
            p.buffs.add_buff(26, 3600); // WellFed −0.05 (the tier 26 row)
        }
        crate::world_sim::player::update_player(&mut app, ONE_TICK_SECS);
        let p = app.player.as_mut().unwrap();
        p.pick_speed_bonus += 0.5; // the chisel-style extra
        // The raw 1 − bonus would be below 0.3; the accessor clamps.
        assert!(p.pick_speed() >= 0.3, "pickSpeed clamps at 0.3");
        assert!((p.pick_speed() - 0.3).abs() < 1e-6, "the floor is exactly 0.3");
    }

    /// S1-P-i29 — a pick cannot mine the axe tiles (`Main.tileAxe`,
    /// `Player.cs:45656`): cactus 80 stands under the Iron Pickaxe. The axe
    /// arm (the i30 task) owns the chop.
    #[test]
    fn s1_p_i29_a_pick_refuses_the_axe_tiles() {
        let mut app = pick_world_at(80, 30, 260);
        mine_frames(&mut app, 60);
        assert!(app.world.tile(30, 260).unwrap().active(), "a pick never chops cactus");
        assert_eq!(hit_damage(&app, 30, 260), 0, "no pick damage on the axe tile");
    }

    /// S1-P-i29 — the instant-break rows (`Main.tileNoFail`, `Player.cs:53905-
    /// 53908`): a torch (kind 4) breaks on the first pick hit regardless of
    /// the pick power.
    #[test]
    fn s1_p_i29_the_no_fail_tiles_break_on_the_first_hit() {
        let mut app = pick_world_at(4, 30, 260);
        mine_frames(&mut app, 3);
        assert!(!app.world.tile(30, 260).unwrap().active(), "the torch breaks on hit 1");
        assert!(app.items.iter().any(|it| it.kind == 8), "the torch item dropped");
    }

    /// S1-P-i36 — the bug net's catch arm end-to-end: the swung-item
    /// rectangle intersects the goldfish's box, `CatchNPC` converts it to
    /// the Goldfish item 261 (`NPC.cs:9472`) at the player's centre, and
    /// the NPC leaves the pool. The catch runs even with the cursor FAR
    /// outside the tile reach (the arm is player-relative,
    /// `Player.cs:43654`).
    #[test]
    fn s1_p_i36_the_bug_net_catches_a_goldfish_into_item_261() {
        let mut app = mining_app(World::new("t".into(), 64, 800), 30, 260);
        // The net replaces the pickaxe; the cursor parks far away (the
        // mining_app camera points 3 tiles left of the player — inside
        // reach, so move the mouse to the screen corner instead: the
        // catch arm must not care).
        app.inventory[app.selected] = crate::item::ItemSlot {
            kind: 1991,
            name: "Bug Net",
            count: 1,
            prefix: 0,
        };
        app.mouse_pos = Vec2::new(1270.0, 10.0);
        let p = app.player.as_ref().unwrap();
        let (px, py) = (p.x, p.y);
        // The goldfish's 20×18 box (`NPC.cs:9463-9464`) centred on the
        // player's swing arc, just right of the player.
        let fish = Npc::new(NpcKind::Goldfish, px + 14.0, py + 4.0, 1, 0.0);
        app.npcs.push(fish);
        assert!(app.items.is_empty());
        update_editing(&mut app, 1.0 / 60.0, MINE_SCREEN);
        // The swing started (25 ticks, `Item.cs:21305-21315`) and the
        // catch fired on the first animation tick.
        assert!(app.swing > 0.0, "the net swings");
        assert!(app.npcs.is_empty(), "the goldfish is caught");
        assert_eq!(app.items.len(), 1, "one converted item");
        assert_eq!(app.items[0].kind, 261, "the Goldfish item (NPC.cs:9472)");
        assert_eq!(app.items[0].count, 1);
        // The net is not consumed and never mines.
        assert_eq!(app.inventory[app.selected].count, 1);
        assert!(app.mine_target.is_none());
    }

    /// S1-P-i36 — the makeNPC release arm end-to-end: LMB with the
    /// Goldfish item 261 releases NPC 55 at the cursor point, drains the
    /// stack, and the released critter carries `releaseOwner` (never
    /// friendly — `NPC.cs:90538-90541`).
    #[test]
    fn s1_p_i36_the_goldfish_item_releases_a_owned_critter() {
        let mut app = mining_app(World::new("t".into(), 64, 800), 30, 260);
        app.inventory[app.selected] = crate::item::ItemSlot {
            kind: 261,
            name: "Goldfish",
            count: 2,
            prefix: 0,
        };
        assert!(app.npcs.is_empty());
        update_editing(&mut app, 1.0 / 60.0, MINE_SCREEN);
        assert_eq!(app.npcs.len(), 1, "the critter releases");
        // The first `UpdateNPC` runs the catchable-immunity pass
        // (`NPC.cs:90538-90541`): releaseOwner != 255 zeroes the window.
        crate::bug_net::tick_catchable_immunity(&mut app.npcs[0]);
        let n = &app.npcs[0];
        assert_eq!(n.kind, NpcKind::Goldfish);
        assert_eq!(n.release_owner, app.local_id as u8, "NPC.cs:80162");
        assert_eq!(n.catchable_immunity, 0, "the released window is zeroed");
        assert!(!n.friendly, "NPC.cs:90543-90551");
        // `NewNPC`'s timeLeft = (int)(750 * 1.25) (`NPC.cs:80772`).
        assert_eq!(n.despawn_timer, 937);
        // The stack drained once (`Player.cs:43317-43320`).
        assert_eq!(app.inventory[app.selected].count, 1);
        // The release point: the cursor world point, truncated
        // (`Player.cs:44066-44071`); Bottom = (X, Y) (`NPC.cs:80770`).
        let (w, h) = n.size();
        assert_eq!(n.x + w * 0.5, (app.camera.x) as i32 as f32);
        assert_eq!(n.y + h, (app.camera.y) as i32 as f32);
    }

    /// S1-E-18: the demon altar takes the hammer arm of
    /// `ItemCheck_UseMiningTools_ActuallyUseMiningTool` (`Player.cs:45665-45704`).
    /// A ≥ 80-power hammer in hardmode builds to a kill, running `SmashAltar`
    /// (`WorldGen.cs:48920+`) and BEGONE_EVIL (`:49108`); a pre-hardmode world
    /// refuses (`:45675-45679`).
    #[test]
    fn the_altar_hammer_smash_seeds_ore_and_fires_begone_evil() {
        // Width 420 (the production minimum): the ore bands draw
        // `Next(100, maxTilesX - 100)` (`:48945`), which needs maxTilesX > 200
        // — the C#'s own `Next` assertion rejects the old 64-wide fixture.
        let mut w = World::new("t".into(), 420, 800);
        let i = w.index(30, 260);
        w.tiles[i] = Tile {
            kind: 26,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.compute_column_surface();
        let mut app = mining_app(w, 30, 260);
        app.world.hard_mode = true;
        app.inventory[app.selected].kind = 367; // the Pwnhammer (hammer 80)
        mine_frames(&mut app, 30);
        // The smash removes the altar tile (`WorldGen.KillTile`'s case 26) and
        // advances `altarCount` exactly once — a second swing finds no tile.
        assert_eq!(app.world.altars_smashed, 1, "the altar smashes exactly once");
        assert!(
            !app.world.tile(30, 260).is_some_and(|t| t.active()),
            "the altar tile dies with the smash"
        );
        assert!(
            matches!(app.ore_tiers.cobalt, 107 | 221),
            "the first smash seeds Cobalt (SmashAltar's tier 0, 50/50 to 221): {}",
            app.ore_tiers.cobalt
        );
        assert!(
            app.achievements
                .list
                .iter()
                .any(|a| a.name == "BEGONE_EVIL" && a.completed),
            "progression 6 rode the hammer's smash"
        );

        // The pre-hardmode refusal (`Player.cs:45675-45679`): the same hammer
        // never lands a smash and never clears the altar.
        let mut w2 = World::new("t".into(), 64, 800);
        let i2 = w2.index(30, 260);
        w2.tiles[i2] = Tile {
            kind: 26,
            ..Default::default()
        };
        w2.tiles[i2].flags |= FLAG_ACTIVE;
        w2.compute_column_surface();
        let mut app2 = mining_app(w2, 30, 260);
        app2.inventory[app.selected].kind = 367;
        mine_frames(&mut app2, 60);
        assert_eq!(app2.world.altars_smashed, 0, "pre-hardmode refuses the altar");
        assert!(app2.world.tile(30, 260).is_some_and(|t| t.active()));
        assert!(!app2
            .achievements
            .list
            .iter()
            .any(|a| a.name == "BEGONE_EVIL" && a.completed));
    }

    /// DW-1.1: the smash's ore draws ride the `UpdatePlayers` stream — the
    /// hammer hit's stream — and never touch the growth pass's
    /// `UpdateWorldGen` stream (the two were one `growth_rng` field before
    /// the S1-A-11 split).
    #[test]
    fn test_dw_1_1_altar_smash_draws_the_update_players_stream() {
        let mut w = World::new("t".into(), 420, 800);
        let i = w.index(30, 260);
        w.tiles[i] = Tile {
            kind: 26,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.compute_column_surface();
        let mut app = mining_app(w, 30, 260);
        app.world.hard_mode = true;
        app.inventory[app.selected].kind = 367; // the Pwnhammer (hammer 80)
        let pristine = tr_core::UnifiedRandom::new(app.world.seed as i32);
        mine_frames(&mut app, 30);
        // The smash landed (the same rig as the behavioural pin above) and
        // the `UpdatePlayers` stream left its pristine seed state.
        assert_eq!(app.world.altars_smashed, 1, "the smash ran");
        assert_ne!(
            *app.rngs.stream(crate::app::RngKey::UpdatePlayers),
            pristine,
            "the smash drew the UpdatePlayers stream"
        );
        // The growth pass's stream is untouched by the editing pass.
        assert_eq!(
            *app.rngs.stream(crate::app::RngKey::UpdateWorldGen),
            pristine,
            "the altar smash never draws UpdateWorldGen"
        );
    }

    /// W4-d103 POST FINAL item 2: the smash broadcasts the ore-blessing line
    /// (`WorldGen.cs:48969`/`:49002`/`:49034` — `Lang.misc[12..14]` classic,
    /// `[21..23]` the alternates) in the World color, exactly once per smash,
    /// keyed on the PLACED ore (the drunk flips move the key with the ore).
    #[test]
    fn w4_d103_post_the_smash_broadcasts_the_ore_blessing() {
        let mut w = World::new("t".into(), 420, 800);
        let i = w.index(30, 260);
        w.tiles[i] = Tile {
            kind: 26,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.compute_column_surface();
        let mut app = mining_app(w, 30, 260);
        app.world.hard_mode = true;
        app.inventory[app.selected].kind = 367; // the Pwnhammer (hammer 80)
        let chat_before = app.chat.messages.len();
        mine_frames(&mut app, 30);
        assert_eq!(app.world.altars_smashed, 1, "the smash ran");
        assert_eq!(
            app.chat.messages.len(),
            chat_before + 1,
            "exactly one blessing line per smash"
        );
        let line = app.chat.messages.last().unwrap();
        // The first smash is tier 0: Cobalt 107 (misc 12) or Palladium 221
        // (misc 21) — the 50/50 roll decides, the key rides the placed ore.
        let key = match app.ore_tiers.cobalt {
            107 => 12,
            221 => 21,
            other => panic!("tier 0 placed {other}, not 107/221"),
        };
        assert_eq!(line.text, crate::lang::misc(key));
        assert_eq!(line.color, crate::chat::COLOR_WORLD);
    }

    // ---- S1-P-i31: the hammer interaction --------------------------------
    //
    // The C# chain: `ItemCheck_UseMiningTools` (`Player.cs:45584`) → the
    // hammer arm (`:45665-45704`), `TryPoundingTile` (`:45969-46311`),
    // `TryFindingWallToHammer`/`TryHittingWall`/`PickWall`
    // (`:45832-45967`). The pound latch (`poundRelease`, `:1502`) sets on
    // the release (`:45635-45638`) and is consumed by the next press's
    // mining tick. WRITTEN, NOT RUN (code-only mode).

    /// A hammer-held rig over one tile: the Copper Hammer (3505, hammer 35,
    /// useTime 23, autoReuse) unless overridden.
    fn hammer_world_at(kind: u16, tx: i32, ty: i32) -> App {
        let mut app = pick_world_at(kind, tx, ty);
        app.inventory[app.selected].kind = 3505; // the Copper Hammer
        app
    }

    /// One release-press cycle: release TWO frames (the i12 end-of-tick
    /// latch, `Player.cs:42829` — after a held swing the FIRST release frame
    /// still writes false from `prev_mine_down = true`), press `press`
    /// frames (the first action tick lands ONE pound), then release and run
    /// `hold` frames to drain the swing. (M3 sync: the Copper Hammer is
    /// `autoReuse` (`Item.cs:31417` → `SetDefaults1(7)` `:1396`), so the
    /// landed i12 `AutoReuseLogic` forces `releaseUseItem` every HELD tick
    /// (`:52885`) and a held hammer keeps pounding at the useTime cadence —
    /// the drain phase must be RELEASED to land exactly one pound per
    /// cycle, the vanilla quick-click shape.)
    fn pound_cycle(app: &mut App, hold: usize, press: usize) {
        // Drain the previous swing RELEASED first (>= 2 frames for the
        // latch), then the fresh press lands one pound on its first
        // action tick.
        app.mine_down = false;
        mine_frames(app, hold.max(2));
        app.mine_down = true;
        mine_frames(app, press);
    }

    /// S1-P-i31 — the pound latch gates `TryPoundingTile` (`:45971`).
    /// (M3 sync re-derivation: the Copper Hammer is `autoReuse`
    /// (`Item.cs:31417` → `SetDefaults1(7)` `:1396`), so the landed i12
    /// `AutoReuseLogic` forces `releaseUseItem` every held tick (`:52885`)
    /// and the `:45635-45638` arm re-arms `poundRelease` continuously — a
    /// HELD hammer pounds at the useTime cadence (vanilla autoswing). A
    /// quick release+press lands exactly ONE pound per press.)
    #[test]
    fn s1_p_i31_the_pound_needs_a_release_then_a_press() {
        let mut app = hammer_world_at(1, 30, 260);
        // The release arms the latch (two frames — the i12 end-of-tick
        // latch convention, `Player.cs:42829`).
        app.mine_down = false;
        mine_frames(&mut app, 2);
        assert!(app.pound_release, "releaseUseItem sets poundRelease");
        // The press: the first ready tick pounds (the half brick).
        app.mine_down = true;
        mine_frames(&mut app, 3);
        let t = app.world.tile(30, 260).unwrap();
        assert!(t.half_brick(), "the press pounds the flat stone to a half brick");
        // Released drain: no second pound (the mining action is gated on
        // `controlUseItem`, `:45608`/`:45626`).
        app.mine_down = false;
        mine_frames(&mut app, 60);
        assert!(app.world.tile(30, 260).unwrap().half_brick(), "one pound per press");
        assert_eq!(app.world.tile(30, 260).unwrap().slope(), 0);
        // The next press lands the second pound (half → slope 1).
        app.mine_down = true;
        mine_frames(&mut app, 3);
        let t = app.world.tile(30, 260).unwrap();
        assert!(!t.half_brick() && t.slope() == 1, "the second press cycles to slope 1");
        // The HELD autoReuse hammer keeps pounding (`:52885` forces the
        // release latch every tick → `:45635` re-arms it): 120 held frames
        // advance the cycle past slope 1.
        mine_frames(&mut app, 120);
        let t = app.world.tile(30, 260).unwrap();
        assert!(t.slope() > 1 || t.half_brick(), "the held autoReuse hammer cycles on");
    }

    /// S1-P-i31 — the full cycle over repeated presses: flat → half →
    /// slope 1 → 2 → 3 → 4 → flat (`Player.cs:46239-46301` +
    /// `WorldGen.PoundTile`/`SlopeTile`). Each cycle is one release+press.
    #[test]
    fn s1_p_i31_the_stone_cycles_half_then_four_slopes() {
        let mut app = hammer_world_at(1, 30, 260);
        let steps: [(bool, u8); 7] = [
            (true, 0),
            (false, 1),
            (false, 2),
            (false, 3),
            (false, 4),
            (false, 0),
            (true, 0),
        ];
        for (half, slope) in steps {
            // Release (two frames — the i12 end-of-tick latch, :42829) +
            // a 3-frame press lands ONE pound; the drain is RELEASED so
            // the autoReuse hold does not cycle on (M3 sync).
            app.mine_down = false;
            mine_frames(&mut app, 2);
            app.mine_down = true;
            mine_frames(&mut app, 3);
            let t = app.world.tile(30, 260).unwrap();
            assert_eq!(t.half_brick(), half, "half at slope {slope}");
            assert_eq!(t.slope(), slope);
            // Let the swing finish so the next press re-arms cleanly.
            app.mine_down = false;
            mine_frames(&mut app, 40);
        }
    }

    /// S1-P-i31 — the platform cycle (`Player.cs:46008-46054`): the
    /// isolated platform slopes 1 → 2, then the CanPoundTile arm writes
    /// slope 0 + the half brick, then the half brick clears.
    #[test]
    fn s1_p_i31_the_platform_slopes_then_half_bricks() {
        let mut app = hammer_world_at(19, 30, 260);
        app.mine_down = false;
        mine_frames(&mut app, 2);
        app.mine_down = true;
        mine_frames(&mut app, 3);
        assert_eq!(app.world.tile(30, 260).unwrap().slope(), 1);
        pound_cycle(&mut app, 40, 3);
        assert_eq!(app.world.tile(30, 260).unwrap().slope(), 2);
        pound_cycle(&mut app, 40, 3);
        let t = app.world.tile(30, 260).unwrap();
        assert!(t.half_brick() && t.slope() == 0, "the platform's third pound half-bricks");
        pound_cycle(&mut app, 40, 3);
        assert!(!app.world.tile(30, 260).unwrap().half_brick(), "the fourth clears it");
    }

    /// S1-P-i31 — the hammer arm on the shadow orb 31 (`Main.tileHammer`,
    /// `Player.cs:45665-45704`): the Copper Hammer's 35 accumulates
    /// 35 + 35 + 35 = 105 ≥ 100 → the orb breaks on the third hit; a pick
    /// never damages it (`:45656`).
    #[test]
    fn s1_p_i31_the_hammer_breaks_the_orb_the_pick_never_does() {
        let mut app = hammer_world_at(31, 30, 260);
        mine_frames(&mut app, 2);
        assert!(app.world.tile(30, 260).unwrap().active(), "the orb stands after hit 1");
        assert_eq!(hit_damage(&app, 30, 260), 35);
        mine_frames(&mut app, 23);
        assert_eq!(hit_damage(&app, 30, 260), 70, "two hits accumulate");
        mine_frames(&mut app, 23);
        assert!(!app.world.tile(30, 260).unwrap().active(), "the third hit breaks the orb");
        // The pick refuses the hammer tile.
        let mut pick = pick_world_at(31, 30, 260);
        mine_frames(&mut pick, 60);
        assert!(pick.world.tile(30, 260).unwrap().active(), "a pick never breaks the orb");
        assert_eq!(hit_damage(&pick, 30, 260), 0);
    }

    /// S1-P-i31 — the altar refusal hurts the smasher (`Player.cs:45675-
    /// 45679`): a pre-hardmode Pwnhammer hit zeroes the damage AND lands
    /// `Hurt(statLife / 2)` — the player's hp drops by half (no defence in
    /// the fixture), the altar stands.
    #[test]
    fn s1_p_i31_the_refused_altar_hit_halves_the_smashers_life() {
        let mut w = World::new("t".into(), 64, 800);
        let i = w.index(30, 260);
        w.tiles[i] = Tile { kind: 26, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.compute_column_surface();
        let mut app = mining_app(w, 30, 260);
        app.inventory[app.selected].kind = 367; // the Pwnhammer
        let hp0 = app.player.as_ref().unwrap().hp;
        mine_frames(&mut app, 3);
        assert!(app.world.tile(30, 260).unwrap().active(), "pre-hardmode refuses");
        let hp1 = app.player.as_ref().unwrap().hp;
        assert!(hp1 < hp0, "the refused hit hurts: {hp0} -> {hp1}");
        // The hurt is statLife / 2 through the defence-free lane (the
        // fixture player wears nothing): hp0 - hp0/2, floored at the
        // endurance/1 minimum the hurt pipeline applies.
        assert_eq!(hp1, hp0 - hp0 / 2, "the hurt is exactly half the life");
        assert_eq!(app.world.altars_smashed, 0);
    }

    /// S1-P-i31 — the wall pass (`Player.cs:45832-45967`): the hammer on an
    /// exposed HOUSE wall accumulates `(int)(35 * 1.5) = 52` per hit in the
    /// WALL book; two hits (104 ≥ 100) break it and drop the wall item.
    /// The cadence is `useTime / 2` (`:45913`).
    #[test]
    fn s1_p_i31_the_hammer_breaks_an_exposed_house_wall() {
        let mut w = World::new("wall".into(), 64, 800);
        let i = w.index(30, 260);
        w.tiles[i].wall = 24; // a house wall (Main.wallHouse)
        w.compute_column_surface();
        let mut app = mining_app(w, 30, 260);
        app.inventory[app.selected].kind = 3505; // the Copper Hammer
        mine_frames(&mut app, 2);
        assert_eq!(app.world.tile(30, 260).unwrap().wall, 24, "the wall stands after hit 1");
        let wall_dmg = app
            .hit_tile
            .try_finding(30, 260, tr_core::hit_tile::WALL)
            .map_or(0, |id| app.hit_tile.data[id].damage);
        assert_eq!(wall_dmg, 52, "the first wall hit's (int)(35 * 1.5)");
        // useTime 23 / 2 = 11 ticks between wall hits.
        mine_frames(&mut app, 12);
        assert_eq!(app.world.tile(30, 260).unwrap().wall, 0, "the second hit breaks the wall");
        assert!(
            app.items.iter().any(|it| it.kind == registry::by_wall(24).unwrap()),
            "the wall item dropped"
        );
    }

    /// S1-P-i31 — `CanPlayerSmashWall` (`Player.cs:45917-45940`): an
    /// INTERIOR unsafe wall (every 3×3 neighbour the same unsafe wall)
    /// refuses the hammer; the same wall at the patch edge breaks.
    #[test]
    fn s1_p_i31_the_interior_unsafe_wall_refuses_the_hammer() {
        let mut w = World::new("wall".into(), 64, 800);
        for x in 28..33 {
            for y in 258..263 {
                let i = w.index(x, y);
                w.tiles[i].wall = 2; // the unsafe dirt wall
            }
        }
        w.compute_column_surface();
        // The interior cell (30, 260): every neighbour is wall 2.
        let mut app = mining_app(w, 30, 260);
        app.inventory[app.selected].kind = 3505;
        mine_frames(&mut app, 60);
        assert_eq!(app.world.tile(30, 260).unwrap().wall, 2, "the interior unsafe wall stands");
        // The edge cell (28, 260): its left neighbour is wall 0.
        let mut w2 = World::new("wall".into(), 64, 800);
        for x in 28..33 {
            for y in 258..263 {
                let i = w2.index(x, y);
                w2.tiles[i].wall = 2;
            }
        }
        w2.compute_column_surface();
        let mut app2 = mining_app(w2, 28, 260);
        app2.inventory[app2.selected].kind = 3505;
        mine_frames(&mut app2, 60);
        assert_eq!(app2.world.tile(28, 260).unwrap().wall, 0, "the edge unsafe wall breaks");
    }

    /// S1-P-i31 — the locked golden door refuses the pound and KEEPS the
    /// latch (`Player.cs:45989-45998`): the stone under a locked door never
    /// half-bricks, and the latch survives for the next tick.
    #[test]
    fn s1_p_i31_the_locked_door_refuses_the_pound() {
        let mut w = World::new("t".into(), 64, 800);
        let i = w.index(30, 260);
        w.tiles[i] = Tile { kind: 1, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
        // The locked door above (the golden frames, WorldGen.cs:69191-69197).
        let j = w.index(30, 259);
        w.tiles[j] = Tile { kind: 10, frame_x: 0, frame_y: 594, ..Default::default() };
        w.tiles[j].flags |= FLAG_ACTIVE;
        w.compute_column_surface();
        let mut app = mining_app(w, 30, 260);
        app.inventory[app.selected].kind = 3505;
        app.mine_down = false;
        mine_frames(&mut app, 1);
        app.mine_down = true;
        mine_frames(&mut app, 10);
        let t = app.world.tile(30, 260).unwrap();
        assert!(!t.half_brick() && t.slope() == 0, "the locked door refuses the pound");
        assert!(app.pound_release, "the refused pound keeps the latch");
    }

    /// S1-P-i31 — the flatten player-push (`WorldGen.cs:80787-80795`): a
    /// player standing INSIDE the pounded cell rides to its top when the
    /// half brick clears (the un-half pound), the drop booked in gfxOffY.
    #[test]
    fn s1_p_i31_the_flatten_lifts_the_player_out_of_the_cell() {
        let mut app = hammer_world_at(1, 30, 260);
        // Park the player inside the target cell (the rig's player sits 3
        // tiles right; move it onto the cell, feet 8 px into it).
        let p = app.player.as_mut().unwrap();
        p.x = 30.0 * TILE_PX + 2.0;
        p.y = 260.0 * TILE_PX + 8.0 - p.height;
        // First cycle: flat → half (no lift — the half brick SETS). Two
        // release frames (the i12 end-of-tick latch, :42829).
        app.mine_down = false;
        mine_frames(&mut app, 2);
        app.mine_down = true;
        mine_frames(&mut app, 3);
        assert!(app.world.tile(30, 260).unwrap().half_brick());
        let y_after_half = app.player.as_ref().unwrap().y;
        // Second cycle: half → slope 1 (the cycle arm, no flatten lift).
        pound_cycle(&mut app, 40, 3);
        assert_eq!(app.world.tile(30, 260).unwrap().slope(), 1);
        assert_eq!(app.player.as_ref().unwrap().y, y_after_half, "the slope arm does not lift");
        // Run the cycle to slope 4, then the flatten pound (slope 4 → 0)
        // lifts the player to the cell top.
        pound_cycle(&mut app, 40, 3); // 2
        pound_cycle(&mut app, 40, 3); // 3
        pound_cycle(&mut app, 40, 3); // 4
        let gfx0 = app.player.as_ref().unwrap().gfx_off_y;
        pound_cycle(&mut app, 40, 3); // 0 — the flatten
        assert_eq!(app.world.tile(30, 260).unwrap().slope(), 0);
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.y, 260.0 * TILE_PX - p.height, "the player rides to the cell top");
        assert!(p.gfx_off_y > gfx0, "the drop books into gfxOffY");
    }

    /// The whip use path (M4-PROJ-7): one press = one swing, the spawn carries
    /// the tag row, the whip never self-crits, and the swing lasts the item's
    /// `useAnimation`.
    fn whip_app(kind: u16) -> App {
        let mut w = World::new("t".into(), 64, 800);
        for x in 0..64 {
            let i = w.index(x, 300);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        w.compute_column_surface();
        let mut app = mining_app(w, 30, 260);
        app.inventory[app.selected] = crate::item::ItemSlot {
            kind,
            name: "whip",
            count: 1,
            prefix: 0,
        };
        // The press edge is the test's: `mine_down` flips to true on the
        // frame under test (`Player.cs:42733`'s release-then-press gate).
        app.mine_down = false;
        app.mine_released = true;
        app
    }

    #[test]
    fn the_whip_press_swings_once_and_spawns_the_tagged_slot() {
        // The BlandWhip (4672 → 841): `autoReuse = false`, `useAnimation` 30,
        // `DefaultToWhip`'s slice, `crit` never rolled for the whip itself.
        let mut app = whip_app(4672);
        // One idle frame sets the edge state (`mine_prev_down` = false,
        // `mine_released` = true); the press on the next frame fires.
        mine_frames(&mut app, 1);
        app.mine_down = true;
        mine_frames(&mut app, 1);
        let spawned: Vec<_> = app
            .projectiles
            .slots()
            .iter()
            .filter(|p| p.active)
            .collect();
        assert_eq!(spawned.len(), 1, "one whip projectile");
        let p = spawned[0];
        assert_eq!(p.kind, 841);
        assert_eq!(p.tag_effect_type, 4672, "the tag row rides the slot");
        assert!(!p.crit, "the whip never self-crits");
        assert_eq!(p.local_hit_cooldown, -1, "the one-hit-per-NPC sentinel");
        // `timeToFlyOut = itemAnimationMax(30) * MaxUpdates(2)`.
        assert_eq!(
            p.whip_settings.map(|s| s.time_fly_out),
            Some(60.0),
            "the swing lives the useAnimation × MaxUpdates"
        );
        // The swing timer matches the item's `useAnimation` (30 ticks = 0.5 s).
        assert!((app.swing_max - 0.5).abs() < 1e-6, "the 30-tick swing");
        // The reuse lock: a held press does not throw a second whip.
        app.mine_released = false;
        mine_frames(&mut app, 60);
        let still = app.projectiles.slots().iter().filter(|p| p.active).count();
        assert!(still <= 1, "one swing per press edge (the past whip ended)");
    }

    #[test]
    fn the_prefixed_minion_tag_damage_reaches_the_slot() {
        // The summon prefixes 85-97 add `Item.bonusTagDamage`
        // (`TryGetPrefixStatMultipliersForItem`, `Item.cs:900-950` — the
        // Ruthless 85 / Mythical-adjacent rows carry tagdmg 3/5); the fire
        // path copies it onto the slot (`ApplyStatsFromSource`'s `:11127`).
        // The prefix must be a SUMMON-table id: 71 (Angry) is an accessory
        // damage row with tagdmg 0 — the old fixture's `want` was 0 and the
        // assertion passed vacuously.
        let mut app = whip_app(4678);
        app.inventory[app.selected].prefix = 88; // Focused: dmg ×1.1, tagdmg +3
        mine_frames(&mut app, 1);
        app.mine_down = true;
        mine_frames(&mut app, 1);
        let p = app
            .projectiles
            .slots()
            .iter()
            .find(|p| p.active)
            .expect("the SwordWhip");
        let want = crate::prefix::prefixed_stats(4678, 88).unwrap().tag_damage;
        assert_eq!(want, 3, "the Focused row carries +3 (`Item.cs:916-919`)");
        assert_eq!(p.bonus_tag_damage, want);
    }

    #[test]
    fn the_whip_spin_seed_is_nonzero_and_seed_deterministic() {
        // `Player.cs:48657-48661`: `ai1 = 0.6 + 0.4 * NextFloat()`, then the
        // `×−2.5` flip on `Next(3) == 0` (every whip but the Dark Harvest).
        // The old generic path spawned ai1 = 0 (a straight whip); the spin
        // seed winds the collision spiral. Two same-seed runs agree.
        let run = |seed: i32| {
            let mut app = whip_app(4672);
            seed_stream(&mut app, seed);
            mine_frames(&mut app, 1);
            app.mine_down = true;
            mine_frames(&mut app, 1);
            app.projectiles
                .slots()
                .iter()
                .find(|p| p.active)
                .map(|p| p.ai[1])
                .unwrap()
        };
        let a = run(0x5EED);
        let b = run(0x5EED);
        assert_eq!(a, b, "the spin seed is seed-stable");
        assert!(a != 0.0, "the spin seed is nonzero (the spiral winds)");
        // The seed is either the positive arm [0.6, 1.0] or the flipped
        // negative arm [−2.5, −1.5].
        assert!(
            (0.6..=1.0).contains(&a) || (-2.5..=-1.5).contains(&a),
            "spin {a} outside the C# arms"
        );
    }

    #[test]
    fn the_dark_harvest_never_flips_its_spin() {
        // `Player.cs:48658`: the `Next(3)` flip is skipped for the Dark
        // Harvest (4680) — its spin stays in the positive [0.6, 1.0] arm
        // across seeds (the flip roll is not even drawn).
        for seed in 0..24i32 {
            let mut app = whip_app(4680);
            seed_stream(&mut app, seed);
            mine_frames(&mut app, 1);
            app.mine_down = true;
            mine_frames(&mut app, 1);
            let spin = app
                .projectiles
                .slots()
                .iter()
                .find(|p| p.active)
                .map(|p| p.ai[1])
                .unwrap();
            assert!(
                (0.6..=1.0).contains(&spin),
                "seed {seed}: Dark Harvest spin {spin} must stay positive"
            );
        }
    }

    #[test]
    fn the_whip_throw_bumps_the_repeat_counter_and_cooldown() {
        // `Player.cs:48684-48688`: the audible swing increments
        // `repeatWhipSwings` (from −1 → 0) and re-arms the 120-tick reset.
        let mut app = whip_app(4672);
        mine_frames(&mut app, 1);
        app.mine_down = true;
        mine_frames(&mut app, 1);
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.repeat_whip_swings, 0, "the first throw is swing #0");
        assert_eq!(p.repeat_whips_reset_cooldown, 120, "the reset re-arms");
    }

    #[test]
    fn the_wicked_armlet_adds_one_knockback_to_the_whip() {
        // `Player.cs:48663-48666`: `accWickedArmlet` adds +1 knockback per
        // whip throw. The BlandWhip's base knockback is 1.0 → 2.0.
        let mut app = whip_app(4672);
        app.player.as_mut().unwrap().acc_wicked_armlet = true;
        mine_frames(&mut app, 1);
        app.mine_down = true;
        mine_frames(&mut app, 1);
        let p = app
            .projectiles
            .slots()
            .iter()
            .find(|p| p.active)
            .expect("the whip");
        assert!((p.knock_back - 2.0).abs() < 1e-5, "kb {} = base 1 + 1", p.knock_back);
    }

    pub(super) fn set_tile(w: &mut World, x: i32, y: i32, kind: u16) {
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
    }

    /// S1-P-i30: a tree column at x = 30 over a ground row at y = 261, the
    /// base cell (30, 260) the mining cursor targets. The canopy marker sits
    /// on top; every other cell is a plain trunk frame.
    fn axe_tree_world(kind: u16, ground: u16, height: i32) -> World {
        let mut w = World::new("t".into(), 64, 800);
        w.frame_important = vec![false; td::TILE_COUNT];
        w.frame_important[kind as usize] = true;
        for x in 0..64 {
            set_tile(&mut w, x, 261, ground);
        }
        for dy in 0..height {
            let i = w.index(30, 260 - dy);
            let (fx, fy) = if dy == height - 1 { (22, 198) } else { (0, 0) };
            w.tiles[i] = Tile {
                kind,
                frame_x: fx,
                frame_y: fy,
                flags: FLAG_ACTIVE,
                ..Default::default()
            };
        }
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    /// S1-P-i30: the axe arm chops the gem tree (583) — the `Main.tileAxe`
    /// family was missing 583-589, so the pick gate refused them. With the
    /// Iron Axe (axe 9 → 10 damage/hit, useTime 19) ten hits reach 100 and
    /// the whole trunk collapses; every cell drops through the gem record
    /// (`WorldGen.cs:65713-65719`). The 200-frame window covers the
    /// ~173-frame break.
    #[test]
    fn s1_p_i30_the_axe_chops_the_gem_tree() {
        // Stone ground (the gem ground test, `:30304-30315`).
        let mut app = mining_app(axe_tree_world(583, 1, 4), 30, 260);
        app.inventory[app.selected].kind = 10; // the Iron Axe, axe 9
        // A pick alone never chops an axe tile (`Player.cs:45656`).
        let mut pick = mining_app(axe_tree_world(583, 1, 4), 30, 260);
        pick.inventory[pick.selected].kind = 1; // the Iron Pickaxe
        mine_frames(&mut pick, 60);
        assert!(
            pick.world.tile(30, 260).unwrap().active(),
            "the pick cannot chop the gem tree"
        );
        mine_frames(&mut app, 200);
        for dy in 0..4 {
            assert!(
                !app.world.tile(30, 260 - dy).unwrap().active(),
                "the gem cell at dy {dy} survived the axe"
            );
        }
        // One gem/stone drop per cell (4); the canopy marker MAY add a
        // gemcorn (4851) on its `Next(2)` roll, so the count is 4 or 5.
        assert!(
            app.items.len() == 4 || app.items.len() == 5,
            "one gem-record drop per cell (+ the canopy gemcorn), got {}",
            app.items.len()
        );
        // Every drop is the gem (180), the stone (3) or the gemcorn (4851) —
        // never the wood 9.
        for item in app.items.iter() {
            assert!(
                item.kind == 180 || item.kind == 3 || item.kind == 4851,
                "the gem cell drops gem/stone/gemcorn, got {}",
                item.kind
            );
        }
    }

    /// S1-P-i30: the axe arm chops the cactus (80) and the whole plant
    /// collapses, one Cactus 276 per segment (`WorldGen.CheckCactus`,
    /// `:54116-54158`; the drop `case 80`, `:64717-64719`). The cactus takes
    /// the TRIPLE axe damage (`Player.cs:45708`: `(int)(axe*3 * 1.2f)` = 32
    /// for the Iron Axe), so four hits reach 128 — the 19-tick cadence puts
    /// the break at frame ~60.
    #[test]
    fn s1_p_i30_the_axe_chops_the_cactus_whole() {
        let mut w = World::new("t".into(), 64, 800);
        w.frame_important = vec![false; td::TILE_COUNT];
        for x in 0..64 {
            set_tile(&mut w, x, 261, 53); // the sand
        }
        for dy in 0..4 {
            set_tile(&mut w, 30, 260 - dy, 80);
        }
        w.compute_column_surface();
        w.compute_chunks();
        let mut app = mining_app(w, 30, 260);
        app.inventory[app.selected].kind = 10; // the Iron Axe
        mine_frames(&mut app, 60);
        for dy in 0..4 {
            assert!(
                !app.world.tile(30, 260 - dy).unwrap().active(),
                "the cactus segment at dy {dy} survived"
            );
        }
        assert_eq!(app.items.len(), 4, "one cactus drop per segment");
        assert!(app.items.iter().all(|i| i.kind == 276), "the Cactus item");
    }

    /// S1-P-i30: the Don't Starve cactus chop hurts the chopper
    /// (`Player.cs:45721-45724`): `DamageVar(6, -luck)` on the chop tick,
    /// only when the world carries the seed. A classic world chops free.
    #[test]
    fn s1_p_i30_the_dont_starve_cactus_chop_hurts() {
        let cactus_world = || {
            let mut w = World::new("t".into(), 64, 800);
            w.frame_important = vec![false; td::TILE_COUNT];
            for x in 0..64 {
                set_tile(&mut w, x, 261, 53);
            }
            set_tile(&mut w, 30, 260, 80);
            w.compute_column_surface();
            w.compute_chunks();
            w
        };
        // The classic world: no hurt. The cactus takes the triple axe
        // damage (32/hit), four hits reach 128 — the break lands ~frame 59;
        // the 80-frame window covers it.
        let mut classic = mining_app(cactus_world(), 30, 260);
        classic.inventory[classic.selected].kind = 10;
        let hp_classic = classic.player.as_ref().unwrap().hp;
        mine_frames(&mut classic, 80);
        assert!(
            !classic.world.tile(30, 260).unwrap().active(),
            "the cactus fell"
        );
        assert_eq!(
            classic.player.as_ref().unwrap().hp,
            hp_classic,
            "the classic chop is free"
        );
        // The Don't Starve world: the chop hurts.
        let mut ds = mining_app(cactus_world(), 30, 260);
        ds.inventory[ds.selected].kind = 10;
        ds.world.seeds.insert(tr_core::seeds::SeedFlags::DONT_STARVE);
        let hp_ds = ds.player.as_ref().unwrap().hp;
        mine_frames(&mut ds, 80);
        assert!(!ds.world.tile(30, 260).unwrap().active(), "the cactus fell");
        assert!(
            ds.player.as_ref().unwrap().hp < hp_ds,
            "the Don't Starve chop hurts the chopper"
        );
    }

    /// S1-P-i30: the Acorn Axe (5295) replants a sapling into the felled base
    /// cell (`Player.cs:45744-45753` → `TryReplantingTree` `:45804-45822`);
    /// a plain axe leaves the cell empty.
    #[test]
    fn s1_p_i30_the_acorn_axe_replants_the_felled_base() {
        // A forest tree over dirt (the sapling's anchor tiles, D_20). The
        // Acorn Axe (axe 25 → 30 damage/hit, useTime 12) breaks the base on
        // the 4th hit (~frame 39); the 200-frame window covers it.
        let mut app = mining_app(axe_tree_world(5, 2, 4), 30, 260);
        app.inventory[app.selected].kind = 5295; // the Acorn Axe
        mine_frames(&mut app, 200);
        // The trunk above the sapling fell: the sapling (tile 20, 1x2 with
        // origin (0,1)) occupies the base cell (30,260) AND the cell above
        // it (30,259), so the first free trunk cell is (30,258).
        assert!(
            !app.world.tile(30, 258).unwrap().active(),
            "the trunk above the sapling fell"
        );
        // The base cell now holds the replanted sapling (tile 20, 1x2 with
        // origin (0,1) — the anchor row is the cursor cell).
        let base = app.world.tile(30, 260).unwrap();
        assert!(base.active() && base.kind == 20, "the sapling replanted");
        assert!(
            app.world.tile(30, 259).unwrap().active()
                && app.world.tile(30, 259).unwrap().kind == 20,
            "the sapling's top cell"
        );
        // The plain Iron Axe fells the same tree but replants nothing. The
        // Iron Axe (axe 9 → 10 damage/hit, useTime 19) needs ten hits to
        // reach 100 — the 200-frame window covers the ~173-frame break.
        let mut plain = mining_app(axe_tree_world(5, 2, 4), 30, 260);
        plain.inventory[plain.selected].kind = 10;
        mine_frames(&mut plain, 200);
        assert!(
            !plain.world.tile(30, 259).unwrap().active(),
            "the trunk fell"
        );
        assert!(
            !plain.world.tile(30, 260).unwrap().active(),
            "the plain axe leaves the base empty"
        );
    }

    /// S1-P-i30: the axe hit-buffer model (`Player.cs:45706-45771`): the
    /// chop damage accumulates in the shared `HitTile` book and the tile
    /// breaks at 100 — NOT the old time-per-tile hardness. The Iron Axe
    /// (axe 9) deals `(int)(9f * 1.2f) = 10` per hit (`:45708`), so the
    /// forest trunk (kind 5) stands at 9 hits (90) and falls on the 10th.
    /// The cadence is `ApplyItemTime(useTime)` = 19 ticks (`:45768`), so
    /// the hits land at frames 2, 21, 40, … — frame 173 is the 10th.
    #[test]
    fn s1_p_i30_the_axe_damage_accumulates_in_the_hit_book() {
        let mut app = mining_app(axe_tree_world(5, 2, 4), 30, 260);
        app.inventory[app.selected].kind = 10; // the Iron Axe, axe 9
        // After the first hit the trunk stands and the book carries 10.
        mine_frames(&mut app, 2);
        assert!(app.world.tile(30, 260).unwrap().active(), "the trunk stands after hit 1");
        assert_eq!(hit_damage(&app, 30, 260), 10, "the first chop's 10");
        // Nine hits accumulate to 90 — still standing. Hits 2..9 land at
        // frames 21, 40, …, 154 (the 19-tick cadence); frame 154 is hit 9.
        mine_frames(&mut app, 152); // frames 3..154
        assert!(app.world.tile(30, 260).unwrap().active(), "the trunk stands at 90");
        assert_eq!(hit_damage(&app, 30, 260), 90, "nine chops accumulate to 90");
        // The 10th hit (frame 173) reaches 100 and fells the whole tree.
        mine_frames(&mut app, 30);
        assert!(!app.world.tile(30, 260).unwrap().active(), "the 10th chop fells the base");
        assert_eq!(hit_damage(&app, 30, 260), 0, "the break clears the book");
        // The wood dropped (the forest family's Wood 9).
        assert!(app.items.iter().any(|it| it.kind == 9), "the wood dropped");
    }

    /// S1-P-i30: the chainsaw channel — the Cobalt Chainsaw (383, channel
    /// true, axe 14, useTime 7 after the `IsChainsaw` epilogue) spawns its
    /// aiStyle-20 projectile (57) on the press and chops continuously while
    /// held; the release clears the channel and the projectile's machine
    /// kills itself (`Projectile.cs:25549`). The chop damage is
    /// `(int)(14f * 1.2f) = 16` per hit (`Player.cs:45708`), so the forest
    /// trunk falls on the 7th hit (112 ≥ 100) — the 7-tick channel cadence
    /// (`Player.cs:43582-43586`) lands it far faster than the Iron Axe.
    #[test]
    fn s1_p_i30_the_chainsaw_channels_and_chops_continuously() {
        let mut app = mining_app(axe_tree_world(5, 2, 4), 30, 260);
        app.inventory[app.selected].kind = 383; // the Cobalt Chainsaw
        // The press: the chainsaw projectile spawns (kind 57) and the
        // channel latches.
        mine_frames(&mut app, 1);
        assert!(
            app.projectiles.slots().iter().any(|p| p.active && p.kind == 57),
            "the chainsaw projectile spawned"
        );
        assert!(app.player.as_ref().unwrap().channel, "the channel latched");
        // The chainsaw chops continuously: 16 damage/hit, 7-tick cadence →
        // the 7th hit (112) fells the trunk well inside 60 frames.
        mine_frames(&mut app, 60);
        assert!(!app.world.tile(30, 260).unwrap().active(), "the chainsaw felled the trunk");
        assert!(app.items.iter().any(|it| it.kind == 9), "the wood dropped");
        // The release: the channel clears (the projectile's own Kill rides
        // the aiStyle-20 machine on the next pool tick — asserted below
        // through `update_projectiles`).
        app.mine_down = false;
        mine_frames(&mut app, 1);
        assert!(!app.player.as_ref().unwrap().channel, "the release clears the channel");
        crate::world_sim::projectiles::update_projectiles(&mut app, ONE_TICK_SECS);
        assert!(
            !app.projectiles.slots().iter().any(|p| p.active && p.kind == 57),
            "the release kills the chainsaw projectile"
        );
    }

    /// S1-P-i30: the lunar chainsaws ride the `ITEM_CORRECTIONS` seam (the
    /// GenItemData placeholder band left them zero rows): the Vortex
    /// Chainsaw (2773) carries axe 27, channel true, useTime 4 (the
    /// `IsChainsaw` epilogue's `(int)(7 * 0.6)`, `Item.cs:48899-48912`) and
    /// shoot 427 — so it chops `(int)(27f * 1.2f) = 32` per hit and fells
    /// the forest trunk on the 4th hit (128 ≥ 100).
    #[test]
    fn s1_p_i30_the_lunar_chainsaw_row_chops() {
        let def = registry::get(2773).expect("the corrected Vortex Chainsaw row");
        assert_eq!(def.axe, 27, "the corrected axe power");
        assert!(def.channel, "the chainsaw channels");
        assert_eq!(def.use_time, 4, "the IsChainsaw epilogue's useTime");
        assert_eq!(def.projectile, Some(427), "the Vortex Chainsaw's shoot");
        let mut app = mining_app(axe_tree_world(5, 2, 4), 30, 260);
        app.inventory[app.selected].kind = 2773;
        mine_frames(&mut app, 1);
        assert!(
            app.projectiles.slots().iter().any(|p| p.active && p.kind == 427),
            "the Vortex Chainsaw projectile spawned"
        );
        mine_frames(&mut app, 40);
        assert!(!app.world.tile(30, 260).unwrap().active(), "the lunar chainsaw felled the trunk");
    }

    /// DW-1.1: with `frame_important` set — the exact condition the removed
    /// gate read — the closed door now runs the mining branch to `break_tile`:
    /// the whole object cascades away and the door item drops, the same result
    /// the other break sources already produce (vanilla's pick tick has no
    /// `frame_important` gate, `Player.cs:45656` → `PickTile :53757`).
    #[test]
    fn test_DW_1_1_frame_important_door_breaks_through_the_mining_branch() {
        let mut w = object_world();
        w.frame_important[10] = true;
        // A doorway: a stone ceiling above the 3-tall door (both `SolidTile`
        // anchors `CheckDoorClosed` verifies), the floor comes with the fixture.
        set_tile(&mut w, 10, 16, 1);
        assert!(edit::place_item_tile(&mut w, 10, 17, 10, 0, 1));
        for y in 17..=19 {
            let t = w.tile(10, y).unwrap();
            assert!(t.active() && t.kind == 10, "door cell (10, {y}) placed");
        }
        let mut app = mining_app(w, 10, 18);
        mine_frames(&mut app, 40); // hardness 0.25 s × 35/40 ≈ 13 frames
        for y in 17..=19 {
            assert!(
                !app.world.tile(10, y).unwrap().active(),
                "door cell (10, {y}) cascaded away"
            );
        }
        assert_eq!(app.items.len(), 1, "one object drop from the cascade");
        assert_eq!(app.items[0].kind, registry::tile_drop(10, 0).unwrap());
        assert_eq!(app.items[0].kind, 25, "GetDoorItem(0) — the wooden door");
    }

    /// DW-1.1: the chest likewise — `frame_important[21]` no longer gates the
    /// branch; breaking one cell destroys the whole object and drops its item.
    #[test]
    fn test_DW_1_1_frame_important_chest_breaks_through_the_mining_branch() {
        let mut w = object_world();
        w.frame_important[21] = true;
        assert!(edit::place_item_tile(&mut w, 10, 19, 21, 0, 1));
        for x in [10, 11] {
            for y in [18, 19] {
                let t = w.tile(x, y).unwrap();
                assert!(t.active() && t.kind == 21, "chest cell ({x}, {y}) placed");
            }
        }
        let mut app = mining_app(w, 10, 18);
        mine_frames(&mut app, 40);
        for x in [10, 11] {
            for y in [18, 19] {
                assert!(
                    !app.world.tile(x, y).unwrap().active(),
                    "chest cell ({x}, {y}) cascaded away"
                );
            }
        }
        assert_eq!(app.items.len(), 1);
        assert_eq!(app.items[0].kind, registry::tile_drop(21, 0).unwrap());
        assert_eq!(app.items[0].kind, 48, "the chest item");
    }

    /// M1.4-TO-6: the cell under a chest refuses the mining break
    /// (`CanKillTile`'s container-family arm, `WorldGen.cs:62259-62270`) —
    /// the support and the object stay, no drops.
    #[test]
    fn the_support_under_a_chest_refuses_the_mining_break() {
        let mut w = object_world();
        assert!(edit::place_item_tile(&mut w, 10, 19, 21, 0, 1));
        let mut app = mining_app(w, 10, 20); // the support cell
        mine_frames(&mut app, 40);
        assert!(
            app.world.tile(10, 20).unwrap().active(),
            "the support refuses"
        );
        for x in [10, 11] {
            for y in [18, 19] {
                assert!(
                    app.world.tile(x, y).unwrap().active(),
                    "chest cell ({x}, {y}) stands"
                );
            }
        }
        assert!(app.items.is_empty());
    }

    /// M1.4-TO-6: a chest holding items refuses its own break
    /// (`:62311-62323` through `CanDestroyChest`); emptying the store
    /// restores the DW-1.1 behaviour.
    #[test]
    fn a_stocked_chest_refuses_the_mining_break_until_emptied() {
        let mut w = object_world();
        assert!(edit::place_item_tile(&mut w, 10, 19, 21, 0, 1));
        let mut app = mining_app(w, 10, 18);
        app.world_chests.push(crate::craft::WorldChest {
            tile: (10, 18),
            slots: vec![crate::item::ItemSlot {
                kind: 9,
                name: crate::registry::name(9),
                count: 6,
                prefix: 0,
            }],
            name: String::new(),
            max_items: crate::chest::DEFAULT_MAX_ITEMS as usize,
            ..Default::default()
        });
        mine_frames(&mut app, 40);
        for x in [10, 11] {
            for y in [18, 19] {
                assert!(
                    app.world.tile(x, y).unwrap().active(),
                    "stocked chest cell ({x}, {y}) refuses"
                );
            }
        }
        assert!(app.items.is_empty());
        app.world_chests.clear();
        mine_frames(&mut app, 40);
        for x in [10, 11] {
            for y in [18, 19] {
                assert!(
                    !app.world.tile(x, y).unwrap().active(),
                    "the emptied chest breaks as in DW-1.1"
                );
            }
        }
        assert_eq!(app.items.len(), 1);
        assert_eq!(app.items[0].kind, 48, "the chest item");
    }

    /// S1-P-i43 — `Chest.DestroyChest`'s silent close-on-destroy
    /// (`Chest.cs:640-643`, called from `CheckTileBreakability2_ShouldTileSurvive`,
    /// `WorldGen.cs:62506`): breaking the open chest's tile removes the store
    /// entry and closes the player index WITHOUT sound 11 (the range close's
    /// menu-close never fires — the C# writes `chest = -1` directly).
    #[test]
    fn s1_p_i43_breaking_the_open_chest_closes_it_silently() {
        let mut w = object_world();
        assert!(edit::place_item_tile(&mut w, 10, 19, 21, 0, 1));
        let mut app = mining_app(w, 10, 18);
        let idx = crate::chest::create_chest(&mut app.world_chests, 10, 18);
        {
            let p = app.player.as_mut().unwrap();
            p.chest = idx as i32;
            p.chest_x = 10;
            p.chest_y = 18;
        }
        break_tile(&mut app, 10, 18);
        // The store entry is tombstoned and the open index closed silently.
        assert_eq!(
            crate::chest::find_chest(&app.world_chests, 10, 18),
            None,
            "DestroyChest removed the entry (:639)"
        );
        assert_eq!(
            app.player.as_ref().unwrap().chest, -1,
            "the open index closed (:642)"
        );
        // The chest cells cascaded away (the break ran).
        for x in [10, 11] {
            for y in [18, 19] {
                assert!(!app.world.tile(x, y).unwrap().active(), "chest cell ({x}, {y}) broke");
            }
        }
    }

    /// M1.4-TO-6: the locked golden door refuses `break_tile`
    /// (`:62288-62296` through `IsLockedDoor`); the top cell's frames are
    /// forced to the lock row — the break is silent and the object stays.
    #[test]
    fn the_locked_door_refuses_the_break() {
        let mut w = object_world();
        // The doorway's SolidTile anchors, as in DW-1.1.
        set_tile(&mut w, 10, 16, 1);
        assert!(edit::place_item_tile(&mut w, 10, 17, 10, 0, 1));
        w.tile_mut(10, 17).unwrap().frame_y = 594;
        let mut app = mining_app(w, 10, 17);
        break_tile(&mut app, 10, 17);
        for y in 17..=19 {
            assert!(
                app.world.tile(10, y).unwrap().active(),
                "the locked door refuses ({y})"
            );
        }
        assert!(app.items.is_empty());
    }

    /// DW-1.2: the removed cactus carve-out is behaviour-neutral and the
    /// merged tool tiers restore vanilla's axe rule — kind 80 is `tileAxe`
    /// (`Main.cs:7210`), so the rig holds the Iron Axe: the cactus breaks,
    /// drops per cell and runs no object cascade (`check_object`'s
    /// `_ => None`). S1-P-i30: the chop now rides the `HitTile` book — the
    /// cactus takes the TRIPLE axe damage (`Player.cs:45708`:
    /// `(int)(9*3 * 1.2f) = 32`), so four hits reach 128; the 19-tick
    /// cadence puts the break at frame ~59 (the 80-frame window covers it).
    #[test]
    fn test_DW_1_2_cactus_80_still_breaks_with_the_per_cell_drop() {
        let mut w = object_world();
        w.frame_important[80] = true;
        set_tile(&mut w, 10, 19, 80);
        let mut app = mining_app(w, 10, 19);
        app.inventory[app.selected].kind = 10; // the Iron Axe (axe 9)
        mine_frames(&mut app, 80);
        assert!(!app.world.tile(10, 19).unwrap().active(), "cactus broke");
        assert!(
            app.world.tile(10, 20).unwrap().active(),
            "the floor below survives — no cascade for kind 80"
        );
        assert_eq!(app.items.len(), 1, "the per-cell drop only");
        assert_eq!(app.items[0].kind, registry::tile_drop(80, 0).unwrap());
        assert_eq!(app.items[0].kind, 276, "the cactus item");
    }

    /// DW-1.2: a plain non-frameImportant tile keeps its pre-change behaviour —
    /// the stone floor breaks after its tool-scaled hardness (0.55 s at the
    /// baseline, ×35/40 for the held Iron Pickaxe) and drops the stone item
    /// at the hit cell.
    #[test]
    fn test_DW_1_2_non_frame_important_tile_still_breaks_as_before() {
        let w = object_world(); // frame_important[1] = false
        let mut app = mining_app(w, 20, 20);
        mine_frames(&mut app, 60); // stone hardness 0.55 s × 35/40 ≈ 29 frames
        assert!(!app.world.tile(20, 20).unwrap().active(), "stone broke");
        assert_eq!(app.items.len(), 1);
        assert_eq!(app.items[0].kind, 3, "the stone item, per-cell drop");
    }

    /// DW-1.5: releasing the button before the hardness threshold leaves the
    /// tile intact and resets the mining state — the unchanged `else` semantics.
    #[test]
    fn test_DW_1_5_releasing_mine_down_resets_the_mine_state() {
        let w = object_world();
        let mut app = mining_app(w, 20, 20);
        mine_frames(&mut app, 10); // 9/60 s into the pick-scaled stone hardness
        assert_eq!(app.mine_target, Some((20, 20)));
        assert!(app.mine_timer > 0.0 && app.mine_timer < 0.55);
        app.mine_down = false;
        mine_frames(&mut app, 1);
        assert!(app.world.tile(20, 20).unwrap().active(), "tile intact");
        assert_eq!(app.mine_target, None);
        assert_eq!(app.mine_timer, 0.0);
    }

    /// DW-1.5: opening the inventory mid-accumulation interrupts the same way.
    #[test]
    fn test_DW_1_5_opening_the_inventory_resets_the_mine_state() {
        let w = object_world();
        let mut app = mining_app(w, 20, 20);
        mine_frames(&mut app, 10);
        assert_eq!(app.mine_target, Some((20, 20)));
        app.inventory_open = true;
        mine_frames(&mut app, 1);
        assert!(app.world.tile(20, 20).unwrap().active(), "tile intact");
        assert_eq!(app.mine_target, None);
        assert_eq!(app.mine_timer, 0.0);
    }

    /// A frame_important kind without a registry entry (torch 4) breaks as a
    /// single cell with its `by_tile` drop — the plan's stated edge case for
    /// the ungated branch (no `Check*` routine exists for kind 4).
    #[test]
    fn torch_frame_important_kind_breaks_single_cell_with_its_item() {
        let mut w = object_world();
        w.frame_important[4] = true;
        set_tile(&mut w, 10, 19, 4);
        let mut app = mining_app(w, 10, 19);
        mine_frames(&mut app, 40);
        assert!(!app.world.tile(10, 19).unwrap().active(), "torch broke");
        assert_eq!(app.items.len(), 1);
        assert_eq!(app.items[0].kind, registry::tile_drop(4, 0).unwrap());
        assert_eq!(app.items[0].kind, 8, "the torch item");
    }

    // ---- M1.4-TO-4 phase 1 (DW-1.1): the seed route through the app --------

    /// `object_world` with the floor repainted to grass — a style-0 soil of
    /// `PlaceAlch` (`WorldGen.cs:45395-45397`).
    fn grass_floor_world() -> World {
        let mut w = object_world();
        for x in 0..40 {
            set_tile(&mut w, x, 20, 2);
        }
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    #[test]
    fn test_DW_1_1_seeds_plant_through_the_app_route() {
        // The routing: tile 82 is registered but excluded from the TileObject
        // route (`Player.cs:39772`, `tileToCreate != 82`); the legacy neighbour
        // pre-check does NOT gate the seed either — `BlockPlacementForAssortedThings`
        // forces `canPlace = true` for alch kinds (`Player.cs:41294-41296`), so
        // the assorted cut rule and `PlaceAlch`'s soil/liquid rules decide.
        assert!(!legacy_precheck_applies(82, 0));
        // The registry resolves the seed's tile and placeStyle.
        assert_eq!(registry::place_tile(307), Some(82));
        assert_eq!(registry::place_style(307), 0);
        assert_eq!(registry::place_style(312), 5, "Fireblossom Seeds");
        // The full item route: grass soil, Daybloom Seeds held — one item
        // consumed, the herb written with the style's frames.
        let mut app = flow_app(None, vec![], vec![]);
        app.world = grass_floor_world();
        app.inventory[app.selected] = slot(307, 5);
        assert!(use_tile_item(&mut app, 10, 19));
        assert_eq!(app.inventory[app.selected].count, 4, "one seed consumed");
        let t = *app.world.tile(10, 19).unwrap();
        assert!(t.active() && t.kind == 82, "the herb is in the world");
        assert_eq!((t.frame_x, t.frame_y), (0, 0), "frameX = 18*0");
        // Refused soil keeps the item (stone is not in style 0's set).
        let mut app2 = flow_app(None, vec![], vec![]);
        app2.world = object_world();
        app2.inventory[app2.selected] = slot(307, 5);
        assert!(!use_tile_item(&mut app2, 10, 19));
        assert_eq!(app2.inventory[app2.selected].count, 5, "the seed is kept");
        assert!(
            !app2.world.tile(10, 19).unwrap().active(),
            "nothing written"
        );
    }

    #[test]
    fn test_DW_1_3_planter_box_items_place_through_the_app_route() {
        // The planter boxes keep the legacy pre-checks (no registry entry)
        // and land in the case-380 write with `frameY = 18 * placeStyle`.
        assert!(legacy_precheck_applies(380, 0));
        let mut app = flow_app(None, vec![], vec![]);
        app.world = object_world();
        app.inventory[app.selected] = slot(3215, 3); // Day Bloom Planter Box
        assert!(use_tile_item(&mut app, 10, 19));
        assert_eq!(app.inventory[app.selected].count, 2);
        let t = *app.world.tile(10, 19).unwrap();
        assert!(t.active() && t.kind == 380);
        assert_eq!(t.frame_y, 0, "placeStyle 0");
        // Fire Blossom Planter Box (placeStyle 7) → frameY 126.
        let mut app2 = flow_app(None, vec![], vec![]);
        app2.world = object_world();
        app2.inventory[app2.selected] = slot(3222, 1);
        assert!(use_tile_item(&mut app2, 12, 19));
        assert_eq!(app2.world.tile(12, 19).unwrap().frame_y, 126);
    }

    // ---- Phase 3 (M1.4-TO-4): drops + RNG streams ---------------------------

    /// A staged break rig: the world swapped in, a local player present (the
    /// `FindClosest` stand-in), the `UpdatePlayers` stream pinned for
    /// determinism.
    fn break_app(world: World, seed: u64) -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = world;
        app.player = Some(Player::spawn(&app.world));
        *app.rngs.stream(crate::app::RngKey::UpdatePlayers) =
            tr_core::UnifiedRandom::new(seed as i32);
        app
    }

    /// A synthetic herb cell (the core fixture construction): kind 82/83/84,
    /// `frameX = 18 * style`, frameY 0 (`Main.cs:7685-7687` frame-important).
    fn set_herb(w: &mut World, x: i32, y: i32, kind: u16, style: i32) {
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind,
            frame_x: (18 * style) as i16,
            frame_y: 0,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        if w.frame_important.len() <= kind as usize {
            w.frame_important.resize(td::TILE_COUNT, false);
        }
        for k in [82u16, 83, 84] {
            w.frame_important[k as usize] = true;
        }
    }

    /// A synthetic 2x2 loot pot of `style` (frameY = 36 * style, the origin
    /// at `(x, y)` — `CheckPot`'s frame reconstruction).
    fn set_pot(w: &mut World, x: i32, y: i32, style: i32) {
        for (dx, dy) in [(0i32, 0i32), (1, 0), (0, 1), (1, 1)] {
            let i = w.index(x + dx, y + dy);
            w.tiles[i] = Tile {
                kind: 28,
                frame_x: (dx * 18) as i16,
                frame_y: (36 * style + dy * 18) as i16,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        if w.frame_important.len() <= 28 {
            w.frame_important.resize(td::TILE_COUNT, false);
        }
        w.frame_important[28] = true;
    }

    /// A synthetic 3x2 tile-187 pile: `frameX = 18*dx + 54*style`, `frameY =
    /// 18*dy` (`Main.cs:7415-7417` frame-important). Style 17 is the
    /// sword-in-stone family (`WorldGen.cs:49828-49839`'s 918..=970 gate).
    fn set_large_pile_187(w: &mut World, x: i32, y: i32, style: i32) {
        for dy in 0..2i32 {
            for dx in 0..3i32 {
                let i = w.index(x + dx, y + dy);
                w.tiles[i] = Tile {
                    kind: 187,
                    frame_x: (dx * 18 + style * 54) as i16,
                    frame_y: (dy * 18) as i16,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        if w.frame_important.len() <= 187 {
            w.frame_important.resize(td::TILE_COUNT, false);
        }
        w.frame_important[187] = true;
    }

    /// The resolver harness: `gen_seed` seeds the `genRand` stream, `seed`
    /// the `Main.rand` stream; depth bands per `WorldGen`'s scales in
    /// [`pot_ctx`] (surface 200, rock 400, underworld 800).
    fn run_pot(
        ctx: &DropCtx,
        style: i32,
        y: i32,
        gen_seed: u64,
        seed: u64,
        liquid: u8,
        wall: u16,
    ) -> (Vec<(u16, i32)>, bool) {
        let mut main = tr_core::UnifiedRandom::new(seed as i32);
        let mut gen_stream = tr_core::tile_light::FastRandom::new(gen_seed);
        spawn_things_from_pot(ctx, style, 10, y, liquid, wall, &mut gen_stream, &mut main)
    }

    /// A `DropCtx` with the vanilla depth bands and no special flags.
    fn pot_ctx() -> DropCtx {
        DropCtx {
            world_surface: 200.0,
            rock_layer: 400.0,
            underworld_layer: 800,
            max_tiles_y: 1000,
            ..Default::default()
        }
    }

    /// The first `gen_seed` whose stream misses both the dungeon key (35)
    /// and the potion arm (45) — the clean "falls through to num11" stream.
    fn gen_seed_clean() -> u64 {
        for seed in 0u64..10_000 {
            let mut g = tr_core::tile_light::FastRandom::new(seed);
            if g.next_range(0, 35) != 0 && g.next_range(0, 45) != 0 {
                return seed;
            }
        }
        panic!("no clean gen_seed seed");
    }

    /// The first `Main.rand` seed whose replay of the portal roll
    /// (`Next(range)`), the potion arm's `Next(45)` and `Next(7)` lands
    /// `num11` on `want` — the C# draw order with `expertMode`'s shift.
    fn main_seed_for_num11(want: i32, range: i32, expert: bool) -> u64 {
        for seed in 0u64..1_000_000 {
            let mut main = tr_core::UnifiedRandom::new(seed as i32);
            if main.next_range(0, range) == 0 {
                continue; // the Coin Portal
            }
            if main.next_range(0, 45) == 0 {
                continue; // the potion arm's main roll
            }
            let mut num11 = main.next_range(0, 7);
            if expert {
                num11 -= 1;
            }
            if num11 == want {
                return seed;
            }
        }
        panic!("no seed for num11 == {want}");
    }

    fn herb_ctx(day: bool, moon: u8, blood: bool, rain: bool, alpha: f32, time: f64) -> DropCtx {
        DropCtx {
            day_time: day,
            moon_phase: moon,
            blood_moon: blood,
            raining: rain,
            cloud_alpha: alpha,
            time,
            world_surface: 200.0,
            ..Default::default()
        }
    }

    fn herb_rolls(ctx: &DropCtx, kind: u16, style: i32, staff: bool) -> Vec<(u16, i32)> {
        let mut main = tr_core::UnifiedRandom::new(7u64 as i32);
        herb_drops(kind, style, 250, staff, ctx, &mut main)
    }

    /// The coin value of a split, in copper.
    fn coin_value(loot: &[(u16, i32)]) -> i64 {
        loot.iter()
            .map(|(id, count)| match id {
                71 => *count as i64,
                72 => *count as i64 * 100,
                73 => *count as i64 * 10_000,
                _ => *count as i64 * 1_000_000,
            })
            .sum()
    }

    // DW-3.2: the herb harvest matrix (`KillTile_GetItemDrops` :65674-65698).
    #[test]
    fn test_DW_3_2_herb_drop_matrix() {
        let day = herb_ctx(true, 0, false, false, 0.0, 0.0);
        // Blooming 84: the herb always, seeds `Next(1, 4)` (1-3).
        let drops = herb_rolls(&day, 84, 0, false);
        assert_eq!(drops[0], (313, 1), "Daybloom herb, stack 1");
        assert_eq!(drops[1].0, 307);
        assert!(
            (1..=3).contains(&drops[1].1),
            "seeds 1-3, got {}",
            drops[1].1
        );
        // The staff swaps the roll set: herbs `Next(1, 3)`, seeds `Next(1, 6)`.
        let drops = herb_rolls(&day, 84, 0, true);
        assert!((1..=2).contains(&drops[0].1), "staff herbs 1-2");
        assert!((1..=5).contains(&drops[1].1), "staff seeds 1-5");
        // Mature 83 outside the bloom window: the herb alone, no seeds.
        let night = herb_ctx(false, 5, false, false, 0.0, 0.0);
        assert_eq!(herb_rolls(&night, 83, 0, false), vec![(313, 1)]);
        // Mature 83 inside the window (Daybloom by day): herb + seeds.
        assert_eq!(herb_rolls(&day, 83, 0, false).len(), 2);
        // The immature 82 drops nothing at all — the case arm is 83/84 only
        // and 82 sits in the empty case group (:65759-65815).
        assert_eq!(herb_rolls(&day, 82, 0, false), Vec::new());
        // The Shiverthorn pair (style 6 → 2358/2357).
        let drops = herb_rolls(&day, 84, 6, false);
        assert_eq!(drops[0].0, 2358);
        assert_eq!(drops[1].0, 2357);
    }

    // DW-3.2: `IsAlchemyPlantHarvestable` per style (`WorldGen.cs:66143-66174`).
    #[test]
    fn test_DW_3_2_is_alchemy_plant_harvestable_matrix() {
        let day = herb_ctx(true, 0, false, false, 0.0, 0.0);
        // The new moon (phase 0) — Deathweed's other window.
        let night = herb_ctx(false, 0, false, false, 0.0, 0.0);
        // 0 Daybloom by day, 1 Moonglow by night.
        assert!(is_alchemy_plant_harvestable(0, 100, &day));
        assert!(!is_alchemy_plant_harvestable(0, 100, &night));
        assert!(is_alchemy_plant_harvestable(1, 100, &night));
        assert!(!is_alchemy_plant_harvestable(1, 100, &day));
        // 3 Deathweed: night AND (blood moon OR moonPhase 0).
        assert!(is_alchemy_plant_harvestable(3, 100, &night));
        let blood = herb_ctx(false, 4, true, false, 0.0, 0.0);
        assert!(is_alchemy_plant_harvestable(3, 100, &blood));
        let wrong_moon = herb_ctx(false, 5, false, false, 0.0, 0.0);
        assert!(!is_alchemy_plant_harvestable(3, 100, &wrong_moon));
        // 4 Waterleaf: rain or cloudAlpha.
        assert!(is_alchemy_plant_harvestable(
            4,
            100,
            &herb_ctx(true, 0, false, true, 0.0, 0.0)
        ));
        assert!(is_alchemy_plant_harvestable(
            4,
            100,
            &herb_ctx(true, 0, false, false, 0.5, 0.0)
        ));
        assert!(!is_alchemy_plant_harvestable(4, 100, &day));
        // 5 Fireblossom: time > 40500 and not (raining while shallow).
        let late = herb_ctx(true, 0, false, false, 0.0, 40500.5);
        assert!(is_alchemy_plant_harvestable(5, 250, &late), "deep and late");
        let early = herb_ctx(true, 0, false, false, 0.0, 40499.0);
        assert!(!is_alchemy_plant_harvestable(5, 250, &early));
        // `(!Main.raining | flag)`: not raining passes at any depth, rain
        // needs the deep flag.
        assert!(
            is_alchemy_plant_harvestable(5, 250, &herb_ctx(true, 0, false, true, 0.9, 40500.5)),
            "raining but deep passes"
        );
        assert!(
            !is_alchemy_plant_harvestable(5, 100, &herb_ctx(true, 0, false, true, 0.9, 40500.5)),
            "raining and above the surface fails"
        );
        // Styles 2 and 6 have no window.
        assert!(!is_alchemy_plant_harvestable(2, 100, &day));
        assert!(!is_alchemy_plant_harvestable(6, 100, &day));
    }

    // DW-3.2: the observable break path — mining a blooming herb drops the
    // herb and its seeds, an immature one nothing.
    #[test]
    fn test_DW_3_2_mining_blooming_herb_drops_herb_and_seeds() {
        let mut w = object_world();
        set_herb(&mut w, 10, 19, 84, 3); // Deathweed blooming
        let mut app = break_app(w, 0xABCD);
        break_tile(&mut app, 10, 19);
        let kinds: Vec<u16> = app.items.iter().map(|it| it.kind).collect();
        assert!(kinds.contains(&316), "Deathweed herb 316 in {kinds:?}");
        assert!(kinds.contains(&310), "Deathweed seeds 310 in {kinds:?}");

        // The immature 82 drops nothing.
        let mut w = object_world();
        set_herb(&mut w, 10, 19, 82, 3);
        let mut app = break_app(w, 0xABCD);
        break_tile(&mut app, 10, 19);
        assert!(app.items.is_empty(), "82 never drops");
    }

    // DW-3.2: the cascade channel — breaking the soil under a mature herb
    // drops through `TileKill.herb_events` (the app resolves the rolls).
    #[test]
    fn test_DW_3_2_support_break_herb_drops_via_the_event_channel() {
        let mut w = object_world();
        set_herb(&mut w, 10, 19, 83, 0); // Daybloom, mature
        let mut app = break_app(w, 0xABCD);
        break_tile(&mut app, 10, 20); // the soil dies, the herb follows
        let kinds: Vec<u16> = app.items.iter().map(|it| it.kind).collect();
        assert!(kinds.contains(&3), "the stone drop of the hit cell");
        assert!(kinds.contains(&313), "the herb via the event channel");
    }

    // DW-3.3: the pot loot flows through the whole break path.
    #[test]
    fn test_DW_3_3_breaking_pot_spawns_loot() {
        let mut w = object_world();
        set_pot(&mut w, 10, 18, 0);
        let mut app = break_app(w, 0x5EED);
        break_tile(&mut app, 10, 18);
        for (dx, dy) in [(0i32, 0i32), (1, 0), (0, 1), (1, 1)] {
            assert!(
                !app.world.tile(10 + dx, 18 + dy).unwrap().active(),
                "pot cell gone"
            );
        }
        assert!(!app.items.is_empty(), "the loot spawns at the event cell");
        for it in &app.items {
            assert!(
                !matches!(
                    it.kind,
                    51 | 166..=174 | 197..=204 | 315 | 316 | 698..=703 | 1122..=1124
                ),
                "gore shard {} must not spawn as an item",
                it.kind
            );
        }
    }

    // DW-3.3 (merge fix): `Check3x2`'s `case 187` (`WorldGen.cs:49828-49839`)
    // — the sword-in-stone styles (origin frameX 918..=970) drop exactly one
    // sword: `Main.rand.Next(30) == 0` mints the Terragrim 4144, else the
    // Enchanted Sword 989. Regular 187 frames drop nothing (the empty
    // `GetItemDrops` group :65813-65815).
    #[test]
    fn test_DW_3_3_sword_shrine_pile_drops_one_sword() {
        let mut w = object_world();
        set_large_pile_187(&mut w, 10, 18, 17);
        let mut app = break_app(w, 0x5EED);
        break_tile(&mut app, 10, 18);
        for dy in 0..2i32 {
            for dx in 0..3i32 {
                assert!(
                    !app.world.tile(10 + dx, 18 + dy).unwrap().active(),
                    "pile cell ({},{}) gone",
                    10 + dx,
                    18 + dy
                );
            }
        }
        assert_eq!(app.items.len(), 1, "exactly one sword drop");
        assert!(
            matches!(app.items[0].kind, 989 | 4144),
            "the Enchanted Sword or the 1/30 Terragrim, got {}",
            app.items[0].kind
        );
    }

    #[test]
    fn test_DW_3_3_regular_187_pile_drops_nothing() {
        let mut w = object_world();
        set_large_pile_187(&mut w, 10, 18, 0);
        let mut app = break_app(w, 0x5EED);
        break_tile(&mut app, 10, 18);
        assert!(app.items.is_empty(), "no drop outside the sword frames");
    }

    // DW-3.3: the branch order on fixed seeds — each branch isolated by
    // replaying the C# draw order and asserting the resolver's output
    // matches an independent replay of the same draws.
    #[test]
    fn test_DW_3_3_pot_branch_order_on_fixed_seeds() {
        let ctx = pot_ctx();
        let gen_seed = gen_seed_clean();
        let surface_y = 100; // above world_surface 200

        // Coin Portal: the first main draw == 0 → no items, the portal flag.
        let mut portal_seed = 0u64;
        for seed in 0u64..100_000 {
            let mut main = tr_core::UnifiedRandom::new(seed as i32);
            if main.next_range(0, 500) == 0 {
                portal_seed = seed;
                break;
            }
        }
        let (loot, portal) = run_pot(&ctx, 0, surface_y, gen_seed, portal_seed, 0, 0);
        assert!(portal, "the portal roll fires on the found seed");
        assert!(loot.is_empty(), "the portal returns without loot");

        // Dungeon key: `genRand.Next(35) == 0` with a dungeon wall below the
        // surface → exactly item 327, before any potion roll.
        let mut key_gen = 0u64;
        for seed in 0u64..100_000 {
            let mut g = tr_core::tile_light::FastRandom::new(seed);
            if g.next_range(0, 35) == 0 {
                key_gen = seed;
                break;
            }
        }
        let mut key_main = 0u64;
        for seed in 0u64..100_000 {
            let mut main = tr_core::UnifiedRandom::new(seed as i32);
            if main.next_range(0, 500) != 0 {
                key_main = seed;
                break;
            }
        }
        let (loot, portal) = run_pot(&ctx, 0, 250, key_gen, key_main, 0, 7);
        assert!(!portal);
        assert_eq!(loot, vec![(327, 1)], "the golden key");
        // Without the dungeon wall the branch misses and the flow continues.
        let (loot, _) = run_pot(&ctx, 0, 250, key_gen, key_main, 0, 1);
        assert_ne!(loot, vec![(327, 1)]);

        // Potions: `genRand.Next(45) == 0` → the surface band's table.
        let mut potion_gen = 0u64;
        for seed in 1u64..100_000 {
            let mut g = tr_core::tile_light::FastRandom::new(seed);
            if g.next_range(0, 35) != 0 && g.next_range(0, 45) == 0 {
                potion_gen = seed;
                break;
            }
        }
        let (loot, _) = run_pot(&ctx, 0, surface_y, potion_gen, key_main, 0, 0);
        // Replay the surface table: `genRand.Next(10)` after the 45-roll.
        let mut g = tr_core::tile_light::FastRandom::new(potion_gen);
        let _ = g.next_range(0, 35);
        let _ = g.next_range(0, 45);
        let num7 = g.next_range(0, 10);
        let expected_item = match num7 {
            0 => 292,
            1 => 298,
            2 => 299,
            3 => 290,
            4 => 2322,
            5 => 2324,
            6 => 2325,
            _ => 2350,
        };
        assert_eq!(loot[0].0, expected_item, "surface potion table num7={num7}");
        if num7 >= 7 {
            assert_eq!(loot.len(), 2, "the >=7 arm adds the 2350 stack");
            assert_eq!(loot[1].0, 2350);
            assert!((1..=2).contains(&loot[1].1));
        }

        // Torches (num11 == 1): the base 8, count `Next(2, 7)`; in liquid it
        // swaps to the glowstick 282.
        let torch_seed = main_seed_for_num11(1, 500, false);
        let (loot, _) = run_pot(&ctx, 0, surface_y, gen_seed, torch_seed, 0, 0);
        assert_eq!(loot[0].0, 8, "the plain torch");
        assert!((2..=6).contains(&loot[0].1), "count 2-6, got {}", loot[0].1);
        let (loot, _) = run_pot(&ctx, 0, surface_y, gen_seed, torch_seed, 128, 0);
        assert_eq!(loot[0].0, 282, "glowstick in liquid");

        // Arrows (num11 == 2): 40 × `Next(10, 21)` above the surface.
        let arrow_seed = main_seed_for_num11(2, 500, false);
        let mut main = tr_core::UnifiedRandom::new(arrow_seed as i32);
        let _ = main.next_range(0, 500);
        let _ = main.next_range(0, 45);
        let _ = main.next_range(0, 7);
        let stack = main.next_range(10, 21);
        let (loot, _) = run_pot(&ctx, 0, surface_y, gen_seed, arrow_seed, 0, 0);
        assert_eq!(loot, vec![(40, stack)], "wooden arrows above ground");

        // Healing potions (num11 == 3): one lesser healing 28.
        let heal_seed = main_seed_for_num11(3, 500, false);
        let (loot, _) = run_pot(&ctx, 0, surface_y, gen_seed, heal_seed, 0, 0);
        assert_eq!(loot, vec![(28, 1)]);

        // Bombs (num11 == 4 above the underworld): 166 × `Next(4) + 1`.
        let bomb_seed = main_seed_for_num11(4, 500, false);
        let mut main = tr_core::UnifiedRandom::new(bomb_seed as i32);
        let _ = main.next_range(0, 500);
        let _ = main.next_range(0, 45);
        let _ = main.next_range(0, 7);
        let bombs = main.next_range(0, 4) + 1;
        let (loot, _) = run_pot(&ctx, 0, surface_y, gen_seed, bomb_seed, 0, 0);
        assert_eq!(loot, vec![(166, bombs)]);

        // Rope (num11 == 5 above the underworld, pre-hardmode): 965 ×
        // `Next(20, 41)`.
        let rope_seed = main_seed_for_num11(5, 500, false);
        let mut main = tr_core::UnifiedRandom::new(rope_seed as i32);
        let _ = main.next_range(0, 500);
        let _ = main.next_range(0, 45);
        let _ = main.next_range(0, 7);
        let rope = main.next_range(20, 41);
        let (loot, _) = run_pot(&ctx, 0, surface_y, gen_seed, rope_seed, 0, 0);
        assert_eq!(loot, vec![(965, rope)]);

        // Hearts (num11 == 0 while hurt): 58s, the second behind `Next(2)`.
        let mut hurt = pot_ctx();
        hurt.player_hurt = true;
        let heart_seed = main_seed_for_num11(0, 500, false);
        let mut main = tr_core::UnifiedRandom::new(heart_seed as i32);
        let _ = main.next_range(0, 500);
        let _ = main.next_range(0, 45);
        let _ = main.next_range(0, 7);
        let second = main.next_range(0, 2) == 0;
        let (loot, _) = run_pot(&hurt, 0, surface_y, gen_seed, heart_seed, 0, 0);
        assert!(loot.iter().all(|(id, _)| *id == 58), "hearts only");
        assert_eq!(loot.len(), if second { 2 } else { 1 });

        // Coins (num11 == 6): every drop is a coin and the total equals the
        // replayed `num17` — the C# split subtracts exactly, so the pushed
        // stacks sum to the pot's value.
        let coin_seed = main_seed_for_num11(6, 500, false);
        let (loot, _) = run_pot(&ctx, 0, 900, gen_seed, coin_seed, 0, 0); // deep band
        assert!(!loot.is_empty(), "the coin branch always drops");
        assert!(loot.iter().all(|(id, _)| (71..=74).contains(id)));
        // Replay: gen `Next(35)`/`Next(45)` misses, the coin base; main
        // portal/potion/num11 draws, then the depth (deep ×1.25), the ±20 %
        // roll and the five bonus gates.
        let mut g = tr_core::tile_light::FastRandom::new(gen_seed);
        let _ = g.next_range(0, 35);
        let _ = g.next_range(0, 45);
        let mut num17 = 200.0f32 + g.next_range(-100, 101) as f32;
        let mut main = tr_core::UnifiedRandom::new(coin_seed as i32);
        let _ = main.next_range(0, 500);
        let _ = main.next_range(0, 45);
        let _ = main.next_range(0, 7);
        num17 *= 1.25; // y 900 > maxTilesY 1000 - 250
        num17 *= 1.0 + main.next_range(-20, 21) as f32 * 0.01;
        if main.next_range(0, 4) == 0 {
            num17 *= 1.0 + main.next_range(5, 11) as f32 * 0.01;
        }
        if main.next_range(0, 8) == 0 {
            num17 *= 1.0 + main.next_range(10, 21) as f32 * 0.01;
        }
        if main.next_range(0, 12) == 0 {
            num17 *= 1.0 + main.next_range(20, 41) as f32 * 0.01;
        }
        if main.next_range(0, 16) == 0 {
            num17 *= 1.0 + main.next_range(40, 81) as f32 * 0.01;
        }
        if main.next_range(0, 20) == 0 {
            num17 *= 1.0 + main.next_range(50, 101) as f32 * 0.01;
        }
        num17 *= pot_style_multiplier(0, false);
        assert_eq!(
            coin_value(&loot),
            num17 as i64,
            "coin total == replayed num17"
        );
    }

    // M5-SEED-2: `Main.vampireSeed` (`WorldGen.cs:555-558`) arms
    // `SpawnThingsFromPot` four times — the torch threshold 20 → 30
    // (`:57477-57481`), the `flag5` forced-torch reassignment (`:57494-57497`),
    // the extra wood drop (`:57518-57523`) and the extra `Next(2, 7)` stack
    // (`:57539-57542`). Every arm is behind the flag, so a classic world's
    // pot rolls are byte-identical.
    #[test]
    fn test_M5_SEED_2_the_vampirism_arms_change_the_pot_drops() {
        let classic = pot_ctx();
        let mut vamp = pot_ctx();
        vamp.vampirism = true;
        let surface_y = 100; // above world_surface 200 → `flag` (below rock)
        // A gen stream that misses the dungeon key (`Next(35)`), the potion
        // arm (`Next(45)`) AND the vampirism wood arm's `Next(6)`, so the
        // tests below reach the torch/coin arms.
        let gen_seed = {
            let mut found = None;
            for seed in 0u64..100_000 {
                let mut g = tr_core::tile_light::FastRandom::new(seed);
                if g.next_range(0, 35) != 0
                    && g.next_range(0, 45) != 0
                    && g.next_range(0, 6) != 0
                {
                    found = Some(seed);
                    break;
                }
            }
            found.expect("a clean gen seed")
        };

        // (1) The forced-torch arm (`:57494-57497`): a torchless hurt player
        // whose `num11` rolled 0 (the hearts branch) is reassigned to 1 and
        // walks into the torch arm instead — the hearts never drop.
        let mut hurt = pot_ctx();
        hurt.player_hurt = true;
        let mut hurt_vamp = pot_ctx();
        hurt_vamp.player_hurt = true;
        hurt_vamp.vampirism = true;
        let heart_seed = main_seed_for_num11(0, 500, false);
        let (loot, _) = run_pot(&hurt, 0, surface_y, gen_seed, heart_seed, 0, 0);
        assert!(loot.iter().all(|(id, _)| *id == 58), "classic: hearts");
        let (loot, _) = run_pot(&hurt_vamp, 0, surface_y, gen_seed, heart_seed, 0, 0);
        assert_eq!(loot.len(), 1, "vampirism: the single torch row");
        assert_eq!(loot[0].0, 8, "the forced torch branch");
        // The stack is the base `Next(2, 7)` PLUS the vampirism adder — it
        // replayed out of the same stream.
        let mut main = tr_core::UnifiedRandom::new(heart_seed as i32);
        let _ = main.next_range(0, 500);
        let _ = main.next_range(0, 45);
        let _ = main.next_range(0, 7);
        assert_eq!(loot[0].1, main.next_range(2, 7) + main.next_range(2, 7));

        // (2) The 20 → 30 threshold (`:57477-57481`): 25 torches clears the
        // classic scan (flag5 = false → `num11 == 0` falls through to the
        // coins) but not the vampirism one (flag5 = true → the torch arm).
        let mut stocked = pot_ctx();
        stocked.torches = 25;
        let mut stocked_vamp = pot_ctx();
        stocked_vamp.torches = 25;
        stocked_vamp.vampirism = true;
        let (loot, _) = run_pot(&stocked, 0, 900, gen_seed, heart_seed, 0, 0);
        assert!(loot.iter().all(|(id, _)| (71..=74).contains(id)), "coins");
        let (loot, _) = run_pot(&stocked_vamp, 0, 900, gen_seed, heart_seed, 0, 0);
        assert_eq!(loot[0].0, 8, "25 torches is below the vampirism 30");

        // (3) The extra wood arm (`:57518-57523`): `genRand.Next(6) == 0`
        // (after the 35/45 draws) with `flag2 | flag` → wood × `Next(10, 31)`.
        // The deep-underworld band (y > underworld_layer) has neither flag,
        // so the arm skips and its `genRand` draw is never spent.
        let mut wood_gen = 0u64;
        for seed in 1u64..100_000 {
            let mut g = tr_core::tile_light::FastRandom::new(seed);
            if g.next_range(0, 35) != 0 && g.next_range(0, 45) != 0 && g.next_range(0, 6) == 0 {
                wood_gen = seed;
                break;
            }
        }
        let (loot, _) = run_pot(&vamp, 0, surface_y, wood_gen, heart_seed, 0, 0);
        assert_eq!(loot.len(), 1, "the single wood row");
        assert_eq!(loot[0].0, 9, "the plain pot's wood");
        assert!(
            (10..=30).contains(&loot[0].1),
            "wood 10-30, got {}",
            loot[0].1
        );
        // The deep band: no flag, no wood — with the torch scan satisfied the
        // `num11 == 0` cell falls through to the coins instead. (A stocked
        // player must clear the vampirism 30, else the forced-torch arm
        // above fires first.)
        let mut deep_vamp = pot_ctx();
        deep_vamp.vampirism = true;
        deep_vamp.torches = 30;
        let (loot, _) = run_pot(&deep_vamp, 0, 900, wood_gen, heart_seed, 0, 0);
        assert!(
            loot.iter().all(|(id, _)| (71..=74).contains(id)),
            "no wood below the underworld layer"
        );
        // And the same seed in a classic world: the wood arm never runs, so
        // the `num11 == 0` cell with a torchless player walks into the
        // ordinary torch arm instead (`(num11 == 0) & flag5`, `:57590`).
        let (loot, _) = run_pot(&classic, 0, surface_y, wood_gen, heart_seed, 0, 0);
        assert_eq!(loot.len(), 1);
        assert_eq!(loot[0].0, 8, "classic: the ordinary torch row");
    }

    // DW-3.3: the style multiplier drives the luck range and the coin scale.
    #[test]
    fn test_DW_3_3_style_multiplier_range_and_coins() {
        let close = |a: f32, b: f32| (a - b).abs() < 1e-6;
        // `num = (num * 2 + 1) / 3` after the band table.
        assert!(close(pot_style_multiplier(0, false), 1.0));
        assert!(close(
            pot_style_multiplier(4, false),
            (1.25 * 2.0 + 1.0) / 3.0
        ));
        assert!(close(pot_style_multiplier(25, false), 7.0), "10 → 21/3");
        assert!(
            close(pot_style_multiplier(28, false), 1.0),
            "hardmode-gated"
        );
        assert!(close(pot_style_multiplier(28, true), 3.0), "4 → 9/3");
        assert_eq!(pot_luck_range(1.0), 500);
        assert_eq!(pot_luck_range(7.0), 125);
        // The ×10 style 25 multiplies the same deep pot ×7 — the total
        // equals the replayed `num17`.
        let ctx = pot_ctx();
        let gen_seed = gen_seed_clean();
        let range = pot_luck_range(pot_style_multiplier(25, false));
        let seed = main_seed_for_num11(6, range, false);
        let (rich, _) = run_pot(&ctx, 25, 900, gen_seed, seed, 0, 0);
        let mut g = tr_core::tile_light::FastRandom::new(gen_seed);
        let _ = g.next_range(0, 35);
        let _ = g.next_range(0, 45);
        let mut num17 = 200.0f32 + g.next_range(-100, 101) as f32;
        let mut main = tr_core::UnifiedRandom::new(seed as i32);
        let _ = main.next_range(0, range);
        let _ = main.next_range(0, 45);
        let _ = main.next_range(0, 7);
        num17 *= 1.25; // deep
        num17 *= 1.0 + main.next_range(-20, 21) as f32 * 0.01;
        if main.next_range(0, 4) == 0 {
            num17 *= 1.0 + main.next_range(5, 11) as f32 * 0.01;
        }
        if main.next_range(0, 8) == 0 {
            num17 *= 1.0 + main.next_range(10, 21) as f32 * 0.01;
        }
        if main.next_range(0, 12) == 0 {
            num17 *= 1.0 + main.next_range(20, 41) as f32 * 0.01;
        }
        if main.next_range(0, 16) == 0 {
            num17 *= 1.0 + main.next_range(40, 81) as f32 * 0.01;
        }
        if main.next_range(0, 20) == 0 {
            num17 *= 1.0 + main.next_range(50, 101) as f32 * 0.01;
        }
        num17 *= pot_style_multiplier(25, false);
        assert_eq!(coin_value(&rich), num17 as i64, "style 25 scales by ×7");
    }

    // DW-3.5: expert mode — the num11 shift turns the torch arm into hearts,
    // the coins pay ×2.5 plus three extra rolls, torches gain `Next(1, 7)`.
    #[test]
    fn test_DW_3_5_expert_shifts_and_coin_multipliers() {
        let ctx = pot_ctx();
        let mut expert = pot_ctx();
        expert.expert_mode = true;
        let gen_seed = gen_seed_clean();
        // A seed that lands num11 == 1 non-expert lands 0 under expert —
        // torches become hearts while hurt.
        expert.player_hurt = true;
        let seed = main_seed_for_num11(1, 500, false);
        let (loot, _) = run_pot(&expert, 0, 100, gen_seed, seed, 0, 0);
        assert!(
            loot.iter().all(|(id, _)| *id == 58),
            "expert shifts num11 1 → 0: torches became hearts"
        );
        // The coin multiplier: the same coin-branch seed pays more.
        let seed = main_seed_for_num11(6, 500, false);
        let (classic, _) = run_pot(&ctx, 0, 900, gen_seed, seed, 0, 0);
        let (exp, _) = run_pot(&expert, 0, 900, gen_seed, seed, 0, 0);
        assert!(
            coin_value(&exp) > coin_value(&classic),
            "expert pays more: {} vs {}",
            coin_value(&exp),
            coin_value(&classic)
        );
    }

    #[test]
    fn test_DW_3_5_expert_torch_bonus() {
        let ctx = pot_ctx();
        let mut expert = pot_ctx();
        expert.expert_mode = true;
        let gen_seed = gen_seed_clean();
        let seed = main_seed_for_num11(1, 500, false);
        // Replay both streams: classic `Next(2, 7)`, expert adds `Next(1, 7)`.
        let mut main = tr_core::UnifiedRandom::new(seed as i32);
        let _ = main.next_range(0, 500);
        let _ = main.next_range(0, 45);
        let _ = main.next_range(0, 7);
        let base = main.next_range(2, 7);
        let bonus = main.next_range(1, 7);
        let (loot, _) = run_pot(&expert, 0, 100, gen_seed, seed, 0, 0);
        assert_eq!(loot, vec![(8, base + bonus)], "expert torch bonus");
        let (loot, _) = run_pot(&ctx, 0, 100, gen_seed, seed, 0, 0);
        assert_eq!(loot, vec![(8, base)]);
    }

    // DW-3.5: the >50 coin shrink — a 51-copper pot splits when the gate
    // roll hits and stays whole otherwise; value is always conserved.
    #[test]
    fn test_DW_3_5_coin_split_shrinks_above_50() {
        let mut shrunk = 0u64;
        for seed in 0u64..100_000 {
            let mut main = tr_core::UnifiedRandom::new(seed as i32);
            // The copper arm at 51: the gate `Next(2)` fires only above 50.
            if main.next_range(0, 2) == 0 {
                shrunk = seed;
                break;
            }
        }
        let mut main = tr_core::UnifiedRandom::new(shrunk as i32);
        let drops = split_coins(51.0, &mut main);
        assert_eq!(coin_value(&drops) as f32, 51.0, "value conserved");
        assert!(
            drops.len() > 1 || drops[0].1 < 51,
            "the >50 gate divided the stack"
        );
        // A seed whose gate misses keeps the 51 whole (`Next(2) != 0`).
        let mut whole = 0u64;
        for seed in 0u64..100_000 {
            let mut main = tr_core::UnifiedRandom::new(seed as i32);
            if main.next_range(0, 2) != 0 {
                whole = seed;
                break;
            }
        }
        let mut main = tr_core::UnifiedRandom::new(whole as i32);
        assert_eq!(split_coins(51.0, &mut main), vec![(71, 51)]);
        // Sweep: conservation holds across magnitudes and denominations.
        for cents in [
            1.0f32,
            50.0,
            51.0,
            99.0,
            100.0,
            5_050.0,
            123_456.0,
            2_500_000.0,
        ] {
            let mut main = tr_core::UnifiedRandom::new(42u64 as i32);
            let drops = split_coins(cents, &mut main);
            assert_eq!(coin_value(&drops), cents as i64, "split of {cents}");
            assert!(drops.iter().all(|(_, count)| *count >= 1));
        }
    }

    // DW-3.4: two runs with the same seed and the same action sequence give
    // identical drop sets.
    #[test]
    fn test_DW_3_4_two_runs_same_seed_identical_drops() {
        let run = || {
            let mut w = object_world();
            set_pot(&mut w, 10, 18, 0);
            set_herb(&mut w, 14, 19, 84, 2);
            set_herb(&mut w, 15, 19, 83, 0);
            let mut app = break_app(w, 0xF00D);
            break_tile(&mut app, 10, 18); // the pot
            break_tile(&mut app, 14, 19); // the blooming herb (hit cell)
            break_tile(&mut app, 15, 20); // the mature herb via its soil
            let mut drops: Vec<(u16, u32)> =
                app.items.iter().map(|it| (it.kind, it.count)).collect();
            drops.sort_unstable();
            drops
        };
        assert_eq!(
            run(),
            run(),
            "identical seeds and actions → identical drops"
        );
    }

    // DW-3.6 (amended): the seed route is the `flag2` tileCut rule
    // (`Player.cs:41302-41340`), not the staff's replant block — a
    // non-potted cuttable plant dies to any placement item regardless of
    // bloom (`num9`), and the clicked seed then plants ITSELF through the
    // normal placement route: consumed, its own style, stage 0.
    #[test]
    fn test_DW_3_6_seed_on_non_potted_herb_kills_and_plants_the_seed() {
        let mut w = object_world();
        // Grass soil (the Daybloom set) with an out-of-bloom mature 83.
        let i = w.index(10, 19);
        w.tiles[i] = Tile {
            kind: 2,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        set_herb(&mut w, 10, 18, 83, 0);
        let mut app = break_app(w, 0xBEA5);
        app.world.day_time = false; // night: `IsAlchemyPlantHarvestable(0)` false
        app.inventory[app.selected] = slot(307, 5); // Daybloom seeds
        // The cut rule kills (no pot below), then the seed plants itself.
        assert!(use_tile_item(&mut app, 10, 18));
        let t = app.world.tile(10, 18).unwrap();
        assert!(t.active() && t.kind == 82, "the seed planted itself");
        assert_eq!(t.frame_x, 0, "the seed's own style, stage 0");
        // The seed was consumed by its own placement.
        assert_eq!(app.inventory[app.selected].count, 4);
        // The kill surfaced the herb drop; the out-of-bloom 83 drops no seeds.
        let kinds: Vec<u16> = app.items.iter().map(|it| it.kind).collect();
        assert!(kinds.contains(&313), "the herb drop in {kinds:?}");
        assert!(!kinds.contains(&307), "no seeds out of bloom");
    }

    // DW-3.6 (amended): a potted herb in bloom — the kill needs
    // `flag7` (harvestable) & `flag8` (the seeds) on top of the pot support
    // (`Player.cs:41307-41316`); the seed then plants itself into the pot
    // and is consumed.
    #[test]
    fn test_DW_3_6_seed_on_potted_blooming_herb_kills_and_plants_the_seed() {
        let mut w = object_world();
        let i = w.index(10, 19);
        w.tiles[i] = Tile {
            kind: 78,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.frame_important[78] = true;
        set_herb(&mut w, 10, 18, 84, 0);
        let mut app = break_app(w, 0xBEA5);
        app.inventory[app.selected] = slot(307, 5);
        assert!(use_tile_item(&mut app, 10, 18));
        let t = app.world.tile(10, 18).unwrap();
        assert!(
            t.active() && t.kind == 82 && t.frame_x == 0,
            "the seed's own plant"
        );
        assert_eq!(app.inventory[app.selected].count, 4, "the seed consumed");
        // The harvest drops ride the same kill path (84 always harvestable).
        let kinds: Vec<u16> = app.items.iter().map(|it| it.kind).collect();
        assert!(
            kinds.contains(&313) && kinds.contains(&307),
            "herb + seed drops in {kinds:?}"
        );
    }

    // DW-3.6 (amended): a potted out-of-bloom 83 — `num9` false and `flag7`
    // false, so the C# kill rule fails and `canPlace = false`
    // (`Player.cs:41322-41325`): the plant stands and the seed is kept.
    #[test]
    fn test_DW_3_6_seed_on_potted_out_of_bloom_herb_is_refused() {
        let mut w = object_world();
        let i = w.index(10, 19);
        w.tiles[i] = Tile {
            kind: 78,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.frame_important[78] = true;
        set_herb(&mut w, 10, 18, 83, 0);
        let mut app = break_app(w, 0xBEA5);
        app.world.day_time = false; // night: not harvestable
        app.inventory[app.selected] = slot(307, 5);
        assert!(!use_tile_item(&mut app, 10, 18));
        let t = app.world.tile(10, 18).unwrap();
        assert!(t.active() && t.kind == 83, "the mature herb stands");
        assert_eq!(app.inventory[app.selected].count, 5, "the seed kept");
        assert!(app.items.is_empty(), "no harvest");
    }

    // DW-3.6 (amended): the Staff-of-Regrowth keeps the kill+replant block
    // (`Player.cs:41262-41293` — `TryReplantingHerbs`'s only C# call sites)
    // and `canPlace` stays false, so no grass is placed into the harvested
    // cell; the staff rolls 1-2 herbs + 1-5 seeds (`WorldGen.cs:65689-65693`).
    #[test]
    fn test_DW_3_6_staff_click_harvests_replants_and_refuses_placement() {
        let mut w = object_world();
        let i = w.index(10, 19);
        w.tiles[i] = Tile {
            kind: 78,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.frame_important[78] = true;
        set_herb(&mut w, 10, 18, 84, 0);
        let mut app = break_app(w, 0xBEA5);
        app.inventory[app.selected] = slot(213, 1); // Staff of Regrowth
        assert!(
            !use_tile_item(&mut app, 10, 18),
            "no placement after the kill"
        );
        let t = app.world.tile(10, 18).unwrap();
        assert!(t.active() && t.kind == 82 && t.frame_x == 0, "replanted");
        let kinds: Vec<u16> = app.items.iter().map(|it| it.kind).collect();
        assert!(
            kinds.contains(&313) && kinds.contains(&307),
            "staff rolls in {kinds:?}"
        );
        // The staff item is not consumed.
        assert_eq!(app.inventory[app.selected].count, 1);
    }

    // DW-3.6: the Staff-of-Regrowth replant support matrix 78/380/579
    // (`Player.cs:45826`, called from the `flag` block :41279/:41290).

    #[test]
    fn test_store_1_the_void_lens_routes_pickups_into_bank4() {
        // `Player.cs:39395`'s pickup arm: holding 4131 (the Void Lens)
        // enables the vault (`IsVoidVaultEnabled`) — but the vault is
        // `GetItem`'s LAST resort (`:39395-39398`, S1-P-i47): the inventory
        // passes run first, and only their overflow fills bank4. The grid
        // is full of stone here, so the wood has nowhere else to go.
        let mut app = mining_app(object_world(), 14, 260);
        app.inventory[20] = crate::item::ItemSlot {
            kind: 4131,
            name: "Void Lens",
            count: 1,
            prefix: 0,
        };
        for (i, s) in app.inventory.iter_mut().enumerate() {
            if i != 20 {
                *s = slot(3, registry::max_stack(3));
            }
        }
        // The per-tick refresh rides update_player; drive it directly here.
        app.player.as_mut().unwrap().void_vault_enabled = true;
        seed_stream(&mut app, 0);
        let mut rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        crate::world_sim::items::spawn_item_drop(&mut app.items, rng, 9, 5, 14, 260);
        // The spawn pop (`Item.NewItem`'s vy) then the fall. The window covers
        // the `WorldItem.cs:363-364` air gravity of 0.1/tick: the arc back
        // down to the grab rectangle is longer than the old 0.3 tuning assumed.
        for _ in 0..200 {
            crate::world_sim::items::update_items(&mut app, 1.0 / 60.0);
        }
        let p = app.player.as_ref().unwrap();
        assert!(
            p.banks[3].iter().any(|sl| sl.kind == 9 && sl.count == 5),
            "the wood stack landed in bank4"
        );
        assert!(
            !app.inventory.iter().any(|sl| sl.kind == 9),
            "the inventory stayed out of it"
        );
        // The reverse arm (S1-P-i47): with grid room, the pickup lands in
        // the INVENTORY and the vault stays untouched — the old port
        // routed the vault first.
        let mut app = mining_app(object_world(), 14, 260);
        app.inventory[20] = crate::item::ItemSlot {
            kind: 4131,
            name: "Void Lens",
            count: 1,
            prefix: 0,
        };
        app.player.as_mut().unwrap().void_vault_enabled = true;
        seed_stream(&mut app, 0);
        let mut rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        crate::world_sim::items::spawn_item_drop(&mut app.items, rng, 9, 5, 14, 260);
        for _ in 0..200 {
            crate::world_sim::items::update_items(&mut app, 1.0 / 60.0);
        }
        let p = app.player.as_ref().unwrap();
        assert!(
            p.banks[3].iter().all(|sl| sl.count == 0),
            "the vault is the last resort — the grid took it"
        );
        assert_eq!(
            app.inventory.iter().filter(|sl| sl.kind == 9).map(|sl| sl.count).sum::<u32>(),
            5,
            "the wood landed in the grid"
        );
    }

    #[test]
    fn test_DW_3_6_replant_support_78_380_579() {
        let replant_world = |support: u16| {
            let mut w = object_world();
            let i = w.index(10, 19);
            w.tiles[i] = Tile {
                kind: support,
                frame_x: 0,
                frame_y: 0,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
            w.frame_important[support as usize] = true;
            set_herb(&mut w, 10, 18, 84, 4);
            w
        };
        // 78 and 380 replant the harvested herb (the style survives).
        for support in [78u16, 380] {
            let mut app = break_app(replant_world(support), 1);
            assert!(!adjust_plants(&mut app, 10, 18), "no placement canPlace");
            let t = app.world.tile(10, 18).unwrap();
            assert!(
                t.active() && t.kind == 82 && t.frame_x == 72,
                "support {support} replants style 4"
            );
        }
        // 579 passes the gate but `PlaceAlch`'s soil sets refuse it — the
        // decompile's text kept 1:1 (the plan Notes).
        let mut app = break_app(replant_world(579), 1);
        assert!(!adjust_plants(&mut app, 10, 18));
        assert!(
            !app.world.tile(10, 18).unwrap().active(),
            "579 cannot host the replant"
        );
        // Plain dirt is no support at all (the gate fails).
        let mut w = object_world();
        set_herb(&mut w, 10, 18, 84, 0);
        let mut app = break_app(w, 1);
        assert!(!adjust_plants(&mut app, 10, 18));
        assert!(!app.world.tile(10, 18).unwrap().active());
    }

    // Beyond the DW list: `Player.cs:41327-41330` — the held item's own tile
    // refuses instead of cutting (seeds on the immature 82).
    #[test]
    fn seed_on_its_own_immature_tile_is_refused() {
        let mut w = object_world();
        set_herb(&mut w, 10, 18, 82, 0);
        let mut app = break_app(w, 1);
        app.inventory[app.selected] = slot(307, 5);
        assert!(!use_tile_item(&mut app, 10, 18));
        let t = app.world.tile(10, 18).unwrap();
        assert!(t.active() && t.kind == 82, "the young plant stands");
        assert_eq!(app.inventory[app.selected].count, 5, "the seed kept");
    }

    // Beyond the DW list: the `drop_ctx` torch scan mirrors the C# inventory
    // loop — torch items by their placed tile, capped at the 20 threshold.
    #[test]
    fn drop_ctx_counts_torch_stacks_to_the_threshold() {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = object_world();
        // The scan is the closest player's inventory — the rig needs one.
        app.player = Some(Player::spawn(&app.world));
        app.inventory[0] = slot(8, 30); // the torch item (places tile 4)
        app.inventory[1] = slot(8, 30);
        let ctx = drop_ctx(&app);
        assert_eq!(ctx.torches, 20, "60 torches cap at the 20 threshold");
        app.inventory[1] = slot(2, 30); // dirt is no torch
        assert_eq!(drop_ctx(&app).torches, 20, "one 30-stack still caps");
        app.inventory[0] = slot(2, 30);
        assert_eq!(drop_ctx(&app).torches, 0, "no torches → flag5's true side");
    }

    // Beyond the DW list: mining a planter box drops its own style row.
    #[test]
    fn planter_box_break_drops_the_frame_row_item() {
        let mut w = object_world();
        let i = w.index(10, 19);
        w.tiles[i] = Tile {
            kind: 380,
            frame_x: 0,
            frame_y: 54, // style 3 → item 3218
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.frame_important[380] = true;
        let mut app = break_app(w, 1);
        break_tile(&mut app, 10, 19);
        assert_eq!(app.items.len(), 1);
        assert_eq!(app.items[0].kind, 3218, "3215 + frameY / 18");
    }

    // Beyond the DW list: the 2x1 copper pile (frameX 576) loots copper
    // coins through the pile event channel.
    #[test]
    fn copper_pile_break_spawns_copper_coins() {
        let mut w = object_world();
        let i = w.index(10, 19);
        w.tiles[i] = Tile {
            kind: 185,
            frame_x: 576,
            frame_y: 18,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        let i = w.index(11, 19);
        w.tiles[i] = Tile {
            kind: 185,
            frame_x: 594,
            frame_y: 18,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.frame_important[185] = true;
        let mut app = break_app(w, 0xC01A);
        break_tile(&mut app, 10, 20); // the support dies, the pile follows
        assert!(!app.items.is_empty(), "the pile loot spawned");
        // The hit cell (stone floor) drops its own item; every pile loot
        // entry is copper with the `Next(10, 100)` counts.
        assert!(
            app.items
                .iter()
                .filter(|it| it.kind != 3)
                .all(|it| it.kind == 71 && (10..=99).contains(&(it.count as i32))),
            "the copper pile drops copper only"
        );
        assert!(app.items.iter().any(|it| it.kind == 71));
    }

    // Beyond the DW list: the dontStarve fruit arm — `GetFruitForPot`
    // (:57018-57087) behind its `flag3` gate, the pair on the genRand stream.
    #[test]
    fn fruit_branch_dont_starve_picks_the_style_pair() {
        let mut ctx = pot_ctx();
        ctx.dont_starve = true;
        // The first gen seed hitting `Next(20) == 0` after the 35-miss.
        let mut fruit_gen = 0u64;
        for seed in 0u64..100_000 {
            let mut g = tr_core::tile_light::FastRandom::new(seed);
            if g.next_range(0, 35) != 0 && g.next_range(0, 20) == 0 {
                fruit_gen = seed;
                break;
            }
        }
        let mut key_main = 0u64;
        for cand in 0u64..100_000 {
            let mut main = tr_core::UnifiedRandom::new(cand as i32);
            if main.next_range(0, 500) != 0 {
                key_main = cand;
                break;
            }
        }
        // Style 4 → the 4286/4295 pair.
        let (loot, portal) = run_pot(&ctx, 4, 250, fruit_gen, key_main, 0, 0);
        assert!(!portal);
        assert_eq!(loot.len(), 1, "one fruit stack");
        assert!(
            matches!(loot[0].0, 4286 | 4295),
            "style 4 pair, got {}",
            loot[0].0
        );
        assert!((1..=2).contains(&loot[0].1), "stack Next(1, 3)");
        // Without dontStarveWorld the branch is unreachable (no roll).
        let ctx2 = pot_ctx();
        let (loot, _) = run_pot(&ctx2, 4, 250, fruit_gen, key_main, 0, 0);
        assert!(!loot.iter().any(|(id, _)| matches!(id, 4286 | 4295)));
    }

    // Beyond the DW list: the `num9` arm — any placement item cuts a
    // non-potted cuttable plant (`Player.cs:41307-41316`); the seed's own
    // placement then refuses on the sapling's stone floor (not a Daybloom
    // soil), the seed kept.
    #[test]
    fn seed_click_cuts_a_sapling_and_its_own_planting_refuses_on_stone() {
        let mut w = object_world();
        let i = w.index(10, 19);
        w.tiles[i] = Tile {
            kind: 3,
            frame_x: 0,
            frame_y: 0,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        let mut app = break_app(w, 1);
        app.inventory[app.selected] = slot(307, 5);
        assert!(
            !use_tile_item(&mut app, 10, 19),
            "the seed's placement refuses"
        );
        assert!(
            !app.world.tile(10, 19).unwrap().active(),
            "the sapling was cut"
        );
        assert!(
            !app.world.tile(10, 18).unwrap().active(),
            "nothing planted above"
        );
        assert_eq!(app.inventory[app.selected].count, 5, "the seed kept");
    }

    // Beyond the DW list: the Staff of Regrowth seeds a mature 83 even
    // outside its bloom window (the staff branch precedes the harvest test,
    // `WorldGen.cs:65689-65693`).
    #[test]
    fn staff_seeds_the_mature_herb_outside_the_window() {
        let night = herb_ctx(false, 5, false, false, 0.0, 0.0);
        let drops = herb_rolls(&night, 83, 0, true);
        assert_eq!(drops.len(), 2, "herb + seeds despite the closed window");
        assert_eq!(drops[0].0, 313);
        assert_eq!(drops[1].0, 307);
    }

    /// An app with a staff held and the cursor 80 px right of the centre
    /// (the +x aim, inside the reach). The button is freshly pressed: the
    /// prior frame's `releaseUseItem` is armed (`mine_released = true`, the
    /// C# `controlUseItem && releaseUseItem` use-start gate, `Player.cs:42733`)
    /// so the non-`autoReuse` staves (Vilethorn/Demon Scythe/Magic Missile/
    /// Flamelash/Nimbus) fire on this press — without it the press edge is
    /// missing and the use-start refuses.
    fn staff_app(item: u16, name: &'static str) -> App {
        let mut app = shooter_app();
        app.inventory[app.selected] = crate::item::ItemSlot {
            kind: item,
            name,
            count: 1,
            prefix: 0,
        };
        let (sw, sh) = app.screen_size();
        app.mouse_pos = Vec2::new(sw * 0.5 + 80.0, sh * 0.5);
        app.mine_released = true;
        app.mine_prev_down = false;
        app
    }

    /// One fire tick of the held staff.
    fn fire_once(app: &mut App) {
        let (sw, sh) = app.screen_size();
        update_editing(app, 1.0 / 60.0, Vec2::new(sw, sh));
    }

    #[test]
    fn test_dw_5_1_the_ruby_staff_fires_one_bolt_with_the_shoot_speed() {
        // `Player.cs:49365-49372`'s single-bolt arm: the plain staff fires
        // its own bolt at the aim normalized to `shootSpeed`, the damage
        // through the magic class multiplier.
        let mut app = staff_app(743, "Ruby Staff");
        fire_once(&mut app);
        let bolts: Vec<_> = app
            .projectiles
            .slots()
            .iter()
            .filter(|p| p.active)
            .collect();
        assert_eq!(bolts.len(), 1, "one ruby bolt");
        assert_eq!(bolts[0].kind, 125, "the staff's Item.shoot");
        // The ruby bolt's own feature pack carries ArPenSpread, so the
        // damage rides its 0.5 multiplier (`Projectile.cs:177-186`).
        assert!(
            (bolts[0].vel.length() - 9.0).abs() < 0.05,
            "shootSpeed 9 toward the cursor (|v|={})",
            bolts[0].vel.length()
        );
        assert_eq!(bolts[0].damage, 9, "18 through the ArPenSpread 0.5");
    }

    #[test]
    fn test_dw_5_1_the_sapphire_staff_fires_the_swirl_twins_pair() {
        // `:49365-49372`: the SwirlTwins twin rides `ai1 = 1f`.
        let mut app = staff_app(741, "Sapphire Staff");
        fire_once(&mut app);
        let bolts: Vec<_> = app
            .projectiles
            .slots()
            .iter()
            .filter(|p| p.active)
            .collect();
        assert_eq!(bolts.len(), 2, "the bolt and its twin");
        assert!(
            bolts.iter().any(|p| p.ai[1] == 1.0),
            "the twin rides ai1 = 1f (`:49371`)"
        );
    }

    #[test]
    fn test_dw_5_1_the_bejeweled_staff_fires_the_all_gems_trio() {
        // `:49350-49364`: the Bejeweled Staff (6171, the AllGems gate) fires
        // three bolts of one random gem kind, each carrying the forced
        // Homing/CanBounce/AoeExplosion/FastThenSlow bits (`:49266-49273`)
        // and the ×5/×1.25 velocity multipliers (`:49286-49295`).
        let mut app = staff_app(6171, "Bejeweled Staff");
        fire_once(&mut app);
        let bolts: Vec<_> = app
            .projectiles
            .slots()
            .iter()
            .filter(|p| p.active)
            .collect();
        assert_eq!(bolts.len(), 3, "the AllGems trio");
        let kind = bolts[0].kind;
        assert!([121u16, 122, 123, 124, 597, 125, 126].contains(&kind), "one gem kind");
        assert!(bolts.iter().all(|p| p.kind == kind), "all three of one kind");
        for bolt in &bolts {
            let features = crate::projectile::GemStaffFeatures::from_ai(bolt.ai[0]);
            assert!(features.all_gems(), "the AllGems bit rides the spawn");
            assert!(features.homing() && features.can_bounce(), "the forced four");
            assert!(features.aoe_explosion() && features.fast_then_slow(), "the forced four");
            // 6 (shootSpeed) × 5 × 1.25 = 37.5, and the middle bolt rides
            // the AllGems spread's `toMin2 = 1.1` (`:49361`).
            let want = if bolt.ai[1] == 2.0 { 37.5 * 1.1 } else { 37.5 };
            assert!(
                (bolt.vel.length() - want).abs() < 0.05,
                "the velocity multipliers (|v|={}, want {want})",
                bolt.vel.length()
            );
            // AllGems short-circuits the multipliers to 1f
            // (`Projectile.cs:180-183`), so the raw 27 rides straight.
            assert_eq!(bolt.damage, 27, "AllGems multiplier = 1");
        }
    }

    #[test]
    fn test_dw_5_1_the_gem_spawn_is_seed_deterministic() {
        // The shoot rolls ride the app's item stream: two apps built the
        // same way (same world seed) produce byte-equal spawns.
        let seeds_match = |staff: u16| {
            let mut a = staff_app(staff, "Ruby Staff");
            let mut b = staff_app(staff, "Ruby Staff");
            assert_eq!(
                *a.rngs.stream(crate::app::RngKey::UpdatePlayers),
                *b.rngs.stream(crate::app::RngKey::UpdatePlayers),
                "the same seed"
            );
            fire_once(&mut a);
            fire_once(&mut b);
            a.projectiles
                .slots()
                .iter()
                .filter(|p| p.active)
                .zip(b.projectiles.slots().iter().filter(|p| p.active))
                .all(|(x, y)| x.kind == y.kind && x.pos == y.pos && x.vel == y.vel && x.ai == y.ai)
        };
        assert!(seeds_match(741), "the SwirlTwins pair is seed-stable");
        assert!(seeds_match(6171), "the AllGems random kind is seed-stable");
    }

    #[test]
    fn test_dw_5_2_the_channel_holds_the_aim_and_the_release_clears_it() {
        // `Player.cs:52958-52968` + `:52975-52981`: a `channel` item held
        // with the button down keeps the clamped aim on the player; the
        // release clears it. The channel ARMS at the use start
        // (`StartChanneling(sItem)` `:51677`), so the fixture drives a real
        // press edge (`:42733`'s `controlUseItem && releaseUseItem` pair) —
        // the pre-i23 per-tick re-arm is gone (S1-P-i23).
        let mut app = staff_app(113, "Magic Missile");
        let (sw, sh) = app.screen_size();
        let screen = Vec2::new(sw, sh);
        app.mine_down = false;
        update_editing(&mut app, 1.0 / 60.0, screen);
        app.mine_down = true;
        update_editing(&mut app, 1.0 / 60.0, screen);
        let p = app.player.as_ref().unwrap();
        assert!(p.channel, "the held channel keeps the aim");
        assert_eq!(p.channel_item, 113, "the held item's kind");
        assert!(
            p.channel_target.x > 0.0 && p.channel_target.x != app.camera.x,
            "the aim rides the clamped cursor (target {})",
            p.channel_target
        );
        // The release.
        app.mine_down = false;
        update_editing(&mut app, 1.0 / 60.0, screen);
        let p = app.player.as_ref().unwrap();
        assert!(!p.channel, "the release clears the channel");
        assert_eq!(p.channel_item, 0, "no held item");
    }

    // ---- S1-P-i23: the channel-latch witnesses (written, NOT run) ---------

    #[test]
    fn s1_p_i23_the_use_start_seeds_the_cancel_key() {
        // `StartChanneling(sItem)` (`Player.cs:52958-52968`, called by
        // `ItemCheck_StartActualUse` `:51677` after the mana pay): the key
        // carries the item's SHOOT (16 for the Magic Missile 113) and the
        // index resets to the struct's 0 default; the spawn's tracking
        // tail (`Projectile.cs:11097-11100`) then lands the slot index on
        // the SAME tick.
        let mut app = staff_app(113, "Magic Missile");
        let (sw, sh) = app.screen_size();
        let screen = Vec2::new(sw, sh);
        app.mine_down = false;
        update_editing(&mut app, 1.0 / 60.0, screen);
        app.mine_down = true;
        update_editing(&mut app, 1.0 / 60.0, screen);
        let p = app.player.as_ref().unwrap();
        assert!(p.channel, "the use start armed the latch");
        assert_eq!(p.channel_proj_expected, 16, "the item's shoot, not its id");
        let shot = app
            .projectiles
            .slots()
            .iter()
            .position(|s| s.active && s.kind == 16)
            .expect("the press fired the missile");
        assert_eq!(
            p.channel_index_expected, shot as u16,
            "the use-path tail tracked the spawn on its own tick"
        );
    }

    #[test]
    fn s1_p_i23_the_item_switch_drops_the_latch_even_between_channel_items() {
        // `Player.cs:42800-42807`: `selectedItemState.HasBufferedChange`
        // clears `flag5`, so `channel = false` — a hotbar switch to ANOTHER
        // channel item (Flamelash 218) drops the Magic Missile's latch; the
        // new item re-arms only at its own use start (its own press edge).
        let mut app = staff_app(113, "Magic Missile");
        let (sw, sh) = app.screen_size();
        let screen = Vec2::new(sw, sh);
        app.mine_down = false;
        update_editing(&mut app, 1.0 / 60.0, screen);
        app.mine_down = true;
        update_editing(&mut app, 1.0 / 60.0, screen);
        assert!(app.player.as_ref().unwrap().channel);
        // The switch: the button stays down, the selection changes.
        app.inventory[app.selected] = crate::item::ItemSlot {
            kind: 218,
            name: "Flamelash",
            count: 1,
            prefix: 0,
        };
        update_editing(&mut app, 1.0 / 60.0, screen);
        let p = app.player.as_ref().unwrap();
        assert!(!p.channel, "the buffered switch drops the latch");
        assert_eq!(p.channel_proj_expected, 0, "the key cleared with it");
        assert_eq!(p.channel_index_expected, 0);
    }

    #[test]
    fn s1_p_i23_the_release_clears_even_when_the_cursor_is_out_of_reach() {
        // `Player.cs:42804-42807` runs every tick, ahead of the cursor/reach
        // gates — the C# `ItemCheck` clears the channel on the button
        // release regardless of where the cursor sits. The port hoists the
        // clear to the tick head so a release cannot leave the latch armed
        // behind the reach early-return (S1-P-i23). Arm in reach, then move
        // the player far from the cursor AND release on the same tick.
        let mut app = staff_app(113, "Magic Missile");
        let (sw, sh) = app.screen_size();
        let screen = Vec2::new(sw, sh);
        app.mine_down = false;
        update_editing(&mut app, 1.0 / 60.0, screen);
        app.mine_down = true;
        update_editing(&mut app, 1.0 / 60.0, screen);
        assert!(app.player.as_ref().unwrap().channel, "armed in reach");
        // The player walks far away; the cursor stays put (now out of
        // reach), and the button releases.
        app.player.as_mut().unwrap().x += 4000.0;
        app.mine_down = false;
        update_editing(&mut app, 1.0 / 60.0, screen);
        let p = app.player.as_ref().unwrap();
        assert!(
            !p.channel,
            "the hoisted clear drops the latch behind the reach gate"
        );
        assert_eq!(p.channel_proj_expected, 0);
    }

    #[test]
    fn s1_p_i23_the_cc_head_drops_the_channel_and_the_key() {
        // `ItemCheck`'s CCed head (`Player.cs:42598-42603`): `channel =
        // false` before the early return — frozen, webbed OR stoned
        // (`CCed` `:3377-3387`). The key clears with the latch.
        for cc in ["frozen", "webbed", "stoned"] {
            let mut app = staff_app(113, "Magic Missile");
            let (sw, sh) = app.screen_size();
            let screen = Vec2::new(sw, sh);
            app.mine_down = false;
            update_editing(&mut app, 1.0 / 60.0, screen);
            app.mine_down = true;
            update_editing(&mut app, 1.0 / 60.0, screen);
            assert!(app.player.as_ref().unwrap().channel);
            {
                let p = app.player.as_mut().unwrap();
                match cc {
                    "frozen" => p.frozen = true,
                    "webbed" => p.webbed = true,
                    _ => {
                        p.buffs.add_buff(156, 600);
                    }
                }
            }
            update_editing(&mut app, 1.0 / 60.0, screen);
            let p = app.player.as_ref().unwrap();
            assert!(!p.channel, "the {cc} CC drops the channel");
            assert_eq!(p.channel_item, 0);
            assert_eq!(p.channel_proj_expected, 0);
            assert_eq!(p.channel_index_expected, 0);
        }
    }

    #[test]
    fn s1_p_i23_the_dead_tick_drops_the_channel() {
        // `UpdateDead` (`Player.cs:17017`): the dead tick clears the
        // channel — the player-side clear the pool's owner-death kill
        // (`Projectile.cs:67824`) pairs with.
        let w = World::new("t".into(), 64, 800);
        let mut p = crate::player::Player::spawn(&w);
        p.channel = true;
        p.channel_item = 113;
        p.channel_proj_expected = 16;
        p.channel_index_expected = 7;
        p.channel_target = Vec2::new(500.0, 400.0);
        p.update_dead(false, false, false, false);
        assert!(!p.channel, "the dead tick clears the latch");
        assert_eq!(p.channel_item, 0);
        assert_eq!(p.channel_target, Vec2::ZERO);
        assert_eq!(p.channel_proj_expected, 0);
        assert_eq!(p.channel_index_expected, 0);
    }

    #[test]
    fn s1_p_i23_the_clamp_keeps_the_box_size_on_a_small_world() {
        // `LimitPointToPlayerReachableArea` (`Player.cs:45447-45473`): the
        // 1920x1200 box KEEPS its size through `Utils.Clamp(Rectangle,
        // Rectangle)` (`Utils.cs:236-239`) — on a world smaller than the
        // view the clamp's max < min and `Utils.Clamp<T>` (`:223-234`, max
        // FIRST) returns the max, so the box hangs off the far edge. The
        // 64x64-tile world (1024x1024 px): the play area is (640, 640,
        // 384−1920, 384−1200) = max (−1536, −816) < min (640, 640), so BOTH
        // corners clamp to the max — the box centre lands at
        // (−1536 + 960, −816 + 600) = (−576, −216).
        let w = World::new("t".into(), 64, 64);
        let center = Vec2::new(512.0, 512.0);
        // A point far outside the box: the per-axis scale pulls it onto
        // the box edge around the (−576, −216) centre.
        let point = Vec2::new(512.0 + 5000.0, 512.0);
        let out = limit_point_to_player_reachable_area(center, point, &w);
        // off = (6088, 728); |off.x| > 960 → 960/6088, |off.y| > 600 →
        // 600/728 — the x scale wins (the smaller). out = mid + off·scale.
        let scale = (960.0f32 / 6088.0).min(600.0 / 728.0);
        let want = Vec2::new(-576.0 + 6088.0 * scale, -216.0 + 728.0 * scale);
        assert!(
            (out - want).length() < 1e-3,
            "the max-first clamp hangs the box off the far edge (out={out:?})"
        );
        assert!((out.x - 384.0).abs() < 1e-3, "the box's right edge: −576 + 960");
    }

    #[test]
    fn s1_p_i23_the_clamp_truncates_the_corner_and_centres_on_ints() {
        // `Utils.CenteredRectangle` (`Utils.cs:947-950`): the corner is
        // `(int)(center − size/2)` — truncation toward zero — and the
        // centre is XNA's int `X + Width/2`. On a large world the clamp is
        // a no-op: centre (2000.7, 1500.2) → corner (1040, 900) → box
        // centre (2000, 1500), NOT the float centre.
        let w = World::new("t".into(), 512, 512);
        let center = Vec2::new(2000.7, 1500.2);
        // A point inside the box passes through unchanged.
        let point = Vec2::new(2100.0, 1600.0);
        let out = limit_point_to_player_reachable_area(center, point, &w);
        assert_eq!(out, point, "an inside point is untouched");
        // A point past the right edge lands exactly on it: the int centre
        // 2000 + the 960 half-width.
        let far = Vec2::new(9000.0, 1500.0);
        let out = limit_point_to_player_reachable_area(center, far, &w);
        assert!(
            (out.x - 2960.0).abs() < 1e-3 && (out.y - 1500.0).abs() < 1e-3,
            "the int centre drives the edge (out={out:?})"
        );
    }

    #[test]
    fn s1_p_i23_the_clamp_insets_the_play_area_by_640() {
        // `GetWorldPlayArea` (`WorldBuilding/WorldUtils.cs:16-22`): the
        // 640-px inset on every side. On a 200x200-tile world (3200x3200
        // px) the play area is (640, 640)..(2560, 2560); a player near the
        // left edge has the box's top-left clamped to 640 — the box centre
        // sits at 640 + 960 = 1600, not at the player's x.
        let w = World::new("t".into(), 200, 200);
        let center = Vec2::new(700.0, 1600.0);
        // The corner would be 700 − 960 = −260 < 640 → clamped to 640.
        let point = Vec2::new(700.0 - 5000.0, 1600.0);
        let out = limit_point_to_player_reachable_area(center, point, &w);
        // mid = (1600, 1600); off = (−5900, 0) → scale = 960/5900.
        let want_x = 1600.0 - 5900.0 * (960.0 / 5900.0);
        assert!(
            (out.x - want_x).abs() < 1e-3 && (out.y - 1600.0).abs() < 1e-3,
            "the 640 inset slides the box (out={out:?})"
        );
        assert!((out.x - 640.0).abs() < 1e-3, "the box's left edge at the inset");
    }

    #[test]
    fn test_dw_5_3_the_low_mana_use_is_blocked() {
        // S1-P-i19 — under the shipped `DebugOptions.ManaV2`
        // (`DebugOptions.cs:85`, never toggled false) `CheckMana` NEVER
        // refuses (`Player.cs:52610-52614`): a staff without the mana drains
        // the bar to 0, arms the 2.5× slow cast and STILL fires. The old
        // V1 "no mana, no shot" expectation was the pre-1.4.5 machine the
        // source no longer ships.
        let mut app = staff_app(127, "Space Gun");
        app.player.as_mut().unwrap().mana = 0;
        fire_once(&mut app);
        let fired = app.projectiles.slots().iter().filter(|p| p.active).count();
        assert_eq!(fired, 1, "ManaV2: the empty-bar cast still fires");
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.mana, 0, "the bar stays drained");
        assert!(p.slow_magic_use, "DoRunOutOfManaEffect armed the slow cast");
        // The slow cast stretches the swing ×2.5 (`ApplyItemAnimation`
        // `:4478-4483`): the Space Gun's 17-tick animation truncates to 42
        // ((int)(17 × 2.5), `SetItemAnimation`'s int truncation) THEN takes
        // the `num` +1 bump (`:4481-4482` — the autoReuse style-5 shooter
        // gate `:4466-4469`), so the swing is 43 ticks (the i12 resync
        // re-pin: the gate's pre-i12 42 missed the bump).
        assert!(
            (app.swing_max - 43.0 / 60.0).abs() < 1e-4,
            "the 2.5× slow-cast animation + the num bump, got {}",
            app.swing_max
        );
        // The regen delay re-armed from the empty bar: the long curve
        // (1×240+45)×0.7 = 199 is capped to 4 by the shipped ManaV2 cap
        // (`Player.cs:19403-19410`) — the empty bar does NOT keep 199.
        assert_eq!(p.regen.mana_regen_delay, 4.0);
    }

    #[test]
    fn s1_p_i19_the_paid_cast_keeps_the_normal_pacing_and_four_tick_delay() {
        // A staff WITH mana pays `(int)(mana × manaCost)` and keeps its
        // normal animation; the ManaV2 delay pins at 4 while mana remains
        // (`Player.cs:19403-19415`).
        let mut app = staff_app(743, "Ruby Staff"); // mana 9, useAnimation 28
        fire_once(&mut app);
        let fired = app.projectiles.slots().iter().filter(|p| p.active).count();
        assert_eq!(fired, 1);
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.mana, 20 - 9, "the plain pay at manaCost 1");
        assert!(!p.slow_magic_use, "enough mana — no slow cast");
        // The i12 num bump (`:4466-4469` + `:4494`): the Ruby Staff is an
        // autoReuse style-5 shooter with useTime == useAnimation and
        // reuseDelay 0, so the else branch swings useAnimation + 1 = 29
        // (the gate's pre-i12 28 missed the bump — the resync re-pin).
        assert!(
            (app.swing_max - 29.0 / 60.0).abs() < 1e-4,
            "the normal 28+1-tick animation"
        );
        assert_eq!(p.regen.mana_regen_delay, 4.0, "the V2 pin while mana remains");
    }

    #[test]
    fn s1_p_i19_the_animation_ticks_re_arm_the_regen_delay() {
        // `Player.cs:42808-42813`: EVERY animation tick of a mana item
        // re-arms the delay (`ItemCheck_ApplyManaRegenDelay`), not once per
        // cast — the regen engine's decrement never wins mid-swing.
        let mut app = staff_app(743, "Ruby Staff");
        fire_once(&mut app);
        assert_eq!(app.player.as_ref().unwrap().regen.mana_regen_delay, 4.0);
        // The next editing tick decays the swing (still > 0) and re-arms
        // the delay back to 4 — without the per-tick site it would sit at 3.
        fire_once(&mut app);
        assert!(app.swing > 0.0, "the 28-tick swing still runs");
        assert_eq!(
            app.player.as_ref().unwrap().regen.mana_regen_delay,
            4.0,
            "the animation head re-arms the delay every tick"
        );
    }

    #[test]
    fn s1_p_i19_the_mana_cost_modifiers_scale_the_pay() {
        // `CheckMana` pays `(int)(item.mana × manaCost)` (`:52579`): the
        // Nature's Gift's −0.06 (`:15110-15113`) turns the Ruby Staff's 9
        // into (int)(9 × 0.94) = 8. The fold runs on the player directly
        // (the equip grant pass), keeping the pay deterministic.
        let mut app = staff_app(743, "Ruby Staff");
        {
            let p = app.player.as_mut().unwrap();
            p.equipped[3] = 223; // Nature's Gift, accessory slot 0
            p.update(&app.world, crate::player::PlayerInput::default());
            assert!(
                (p.mana_cost - 0.94).abs() < 1e-4,
                "the accessory row folded, got {}",
                p.mana_cost
            );
        }
        fire_once(&mut app);
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.mana, 20 - 8, "the scaled pay: (int)(9 × 0.94) = 8");
    }

    #[test]
    fn s1_p_i19_the_mana_potion_grants_sickness_and_restores() {
        // `ApplyLifeAndOrMana` (`Player.cs:51888-51952`): the drink restores
        // `healMana` capped at the max and grants Mana Sickness (94) for
        // 300 ticks — the ONLY grant site; weapons never grant it.
        let mut app = staff_app(110, "Lesser Mana Potion"); // healMana 50
        app.player.as_mut().unwrap().mana = 0;
        fire_once(&mut app);
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.mana, 20, "50 restored, capped at the 20 max");
        let idx = p.buffs.find_buff_index(94).expect("Mana Sickness granted");
        assert_eq!(p.buffs.buff_time[idx], 300, "manaSickTime (:57101)");
        assert_eq!(app.inventory[app.selected].count, 0, "the potion consumed");
        // A weapon cast grants NO sickness (`CheckMana` has no buff grant).
        let mut app = staff_app(743, "Ruby Staff");
        fire_once(&mut app);
        assert!(
            app.player.as_ref().unwrap().buffs.find_buff_index(94).is_none(),
            "weapons never grant Mana Sickness"
        );
    }

    #[test]
    fn s1_p_i19_the_sickness_cuts_magic_damage_by_the_decaying_fraction() {
        // `UpdateBuffs`' 94 arm (`Player.cs:10318-10326`): reduction =
        // 0.25 × buffTime/300 read AFTER the tick-head decrement
        // (`update_buffs_tick` runs first, `:24535`); the recalc multiplies
        // magicDamage by 1 − reduction (`:25900-25902`).
        let w = World::new("t".into(), 64, 800);
        let mut p = crate::player::Player::spawn(&w);
        p.buffs.add_buff(94, 301); // the head decrement leaves 300
        p.update(&w, crate::player::PlayerInput::default());
        assert!(p.mana_sick);
        assert!((p.mana_sick_reduction - 0.25).abs() < 1e-4);
        assert!(
            (p.magic_damage - 0.75).abs() < 1e-4,
            "the full 25% cut at fresh sickness, got {}",
            p.magic_damage
        );
        // Half-expired: 150/300 → 12.5% cut.
        let idx = p.buffs.find_buff_index(94).unwrap();
        p.buffs.buff_time[idx] = 151; // the decrement leaves 150
        p.update(&w, crate::player::PlayerInput::default());
        assert!((p.mana_sick_reduction - 0.125).abs() < 1e-4);
        assert!((p.magic_damage - 0.875).abs() < 1e-4);
        // Expired: the flag clears in ResetEffects, the damage returns to 1.
        p.buffs.del_buff(idx);
        p.update(&w, crate::player::PlayerInput::default());
        assert!(!p.mana_sick);
        assert!((p.magic_damage - 1.0).abs() < 1e-4);
    }

    #[test]
    fn s1_e_7a_the_silenced_debuff_refuses_mana_items() {
        // `Player.cs:52221-52224`: `sItem.mana > 0 && silence` blocks the use
        // outright — no bolt, no mana pay (the buff arm is `:10043`).
        let mut app = staff_app(127, "Space Gun");
        app.player.as_mut().unwrap().buffs.add_buff(35, 600);
        app.player.as_mut().unwrap().silence = true;
        fire_once(&mut app);
        let fired = app.projectiles.slots().iter().filter(|p| p.active).count();
        assert_eq!(fired, 0, "silenced, no shot");
        assert_eq!(app.player.as_ref().unwrap().mana, 20, "no mana paid");
        // Without the debuff the same staff fires (the base gate passes).
        let mut ok = staff_app(127, "Space Gun");
        fire_once(&mut ok);
        let fired = ok.projectiles.slots().iter().filter(|p| p.active).count();
        assert_eq!(fired, 1);
    }

    #[test]
    fn test_dw_5_3_the_amber_staff_fires_its_bolt_with_the_corrected_row() {
        // The placeholder-band correction (registry's `ITEM_CORRECTIONS`)
        // rides the whole shoot path: the Amber Staff fires the amber bolt
        // 597 at damage 21 and pays its mana 9.
        let mut app = staff_app(3377, "Amber Staff");
        fire_once(&mut app);
        let bolts: Vec<_> = app
            .projectiles
            .slots()
            .iter()
            .filter(|p| p.active)
            .collect();
        assert_eq!(bolts.len(), 1, "one amber bolt");
        assert_eq!(bolts[0].kind, 597, "the corrected shoot id");
        // The amber bolt's own feature pack carries CanBounce (the 0.8
        // multiplier, `Projectile.cs:177-186`).
        assert_eq!(bolts[0].damage, 16, "21 through the CanBounce 0.8");
        assert_eq!(
            app.player.as_ref().unwrap().mana,
            20 - 9,
            "the corrected mana cost"
        );
    }

    // ---- S1-P-i12: the useTime/useAnimation countdown machinery ------------

    /// `SetItemAnimation(baseFrames, multiplier)` (`Player.cs:4453-4461`):
    /// `(int)(baseFrames * multiplier)` truncates toward zero (NOT banker's
    /// rounding) and floors at 1 when `baseFrames > 0`. Mirror clone,
    /// written not run.
    #[test]
    fn i12_set_item_animation_truncates_and_floors_at_one() {
        // 20 × (1/1.12) = 17.857 → 17 (truncation, not 18).
        assert_eq!(ticks_of(scaled_anim_secs(20, 1.0 / 1.12)), 17);
        // 15 × 0.8 = 12 exact.
        assert_eq!(ticks_of(scaled_anim_secs(15, 0.8)), 12);
        // A sub-1 product floors at 1 (the `baseFrames > 0` guard).
        assert_eq!(ticks_of(scaled_anim_secs(3, 0.1)), 1);
        // A zero base stays zero (no floor).
        assert_eq!(ticks_of(scaled_anim_secs(0, 0.5)), 0);
    }

    /// `ApplyItemAnimation`'s branch chain (`Player.cs:4463-4497`): the melee
    /// branch scales by the inverted meleeSpeed (except the Ale Throwing
    /// Glove 3821, `NoMeleeSpeedBonus`), the createTile/createWall branches
    /// by the inverted tile/wallSpeed, and the style-5 autoReuse shooters
    /// take the `num` +1 tick. Mirror clone, written not run.
    #[test]
    fn i12_apply_item_animation_routes_the_speed_branches() {
        let mut app = shooter_app();
        let slot = |kind: u16| crate::item::ItemSlot { kind, name: "", count: 1, prefix: 0 };
        // The Iron Pickaxe (1): melee, useAnimation 20. No speed grants →
        // multiplier 1 → 20 ticks.
        assert_eq!(ticks_of(apply_item_animation_secs(&app, slot(1))), 20);
        // With the Feral Claws' +0.12: 20 × (1/1.12) = 17.857 → 17.
        app.player.as_mut().unwrap().melee_speed_bonus = 0.12;
        assert_eq!(ticks_of(apply_item_animation_secs(&app, slot(1))), 17);
        // The Ale Throwing Glove (3821) is `NoMeleeSpeedBonus` (`ItemID.cs:340`)
        // and the port's row is ranged (`melee: false`), so it never enters the
        // melee branch — it keeps its raw animation (32) despite the speed grant.
        let ale = registry::get(3821).map(|d| d.use_animation as i32).unwrap_or(0);
        if ale > 0 {
            assert_eq!(ticks_of(apply_item_animation_secs(&app, slot(3821))), ale);
        }
        // The Dirt Block (2): createTile, useAnimation 15. The Builder buff's
        // +0.25 → tileSpeed 1/1.25 = 0.8 → 15 × 0.8 = 12.
        app.player.as_mut().unwrap().tile_speed_bonus = 0.25;
        assert_eq!(ticks_of(apply_item_animation_secs(&app, slot(2))), 12);
        // The Cobalt Repeater (435): a style-5 autoReuse shooter with
        // useTime == useAnimation 23, reuseDelay 0, shoot > 0 → the `num`
        // bump makes it 24 (`:4466-4469` + `:4494`).
        assert_eq!(ticks_of(apply_item_animation_secs(&app, slot(435))), 24);
        // The gunProj four take the same +1 on their id-keyed animation:
        // the Vortex Beater (3475) is 20 → 21.
        let vb = gun_proj_row(3475).unwrap();
        assert_eq!(vb.use_animation + 1, 21);
    }

    /// `ApplyReuseDelay` (`Player.cs:42668-42671` → `:52835-52840`): the
    /// animation-zero tick with a pending lock converts it into a silent
    /// `itemAnimation = itemTime = reuseDelay` hold — the lock itself clears
    /// and `itemAnimationMax` stays 0 (the C# writes only `itemAnimation`).
    /// The port's `attack_cd` start lock extends over the hold. Mirror clone
    /// of the head block, driven on an idle app so no use arm interferes.
    /// Written not run.
    #[test]
    fn i12_apply_reuse_delay_converts_the_lock_into_a_silent_hold() {
        let mut app = shooter_app();
        // An idle hand: no held item, no button — the head countdowns run
        // but no use arm fires.
        app.inventory[app.selected] = crate::item::ItemSlot {
            kind: 0,
            name: "",
            count: 0,
            prefix: 0,
        };
        app.mine_down = false;
        app.mine_prev_down = false;
        app.swing = 0.0;
        app.swing_max = 0.0;
        app.item_time = 0.0;
        app.attack_cd = 0.0;
        // The pending lock: the Clockwork's 14 (`Item.cs:6475`).
        app.reuse_delay = 14.0 * ONE_TICK_SECS;
        let screen = app.screen_size().into();
        update_editing(&mut app, ONE_TICK_SECS, screen);
        // The conversion armed BOTH timers at the lock, then the head decay
        // took one tick off each: 14 - 1 = 13.
        assert_eq!(ticks_of(app.swing), 13, "the hold runs as the animation");
        assert_eq!(ticks_of(app.item_time), 13, "itemTime rides the hold");
        assert_eq!(app.reuse_delay, 0.0, "the lock itself cleared");
        assert_eq!(app.swing_max, 0.0, "itemAnimationMax stays 0 through the hold");
        assert!(app.attack_cd > 0.0, "the start lock covers the hold");
        // The hold counts down to zero without re-arming the lock.
        for _ in 0..13 {
            update_editing(&mut app, ONE_TICK_SECS, screen);
        }
        assert_eq!(ticks_of(app.swing), 0, "the hold ended");
        assert_eq!(app.reuse_delay, 0.0, "no second conversion");
    }

    /// `ItemCheck_ApplyManaRegenDelay` (`Player.cs:42810-42813` →
    /// `:50587-50593`): EVERY animation tick of a mana-paying item re-applies
    /// the regen delay from the CURRENT mana, so it stays pinned until the
    /// swing ends — not applied once at pay. The spaceGun four skip it
    /// (`:50589`). The head block reads the HELD kind's `mana` and the
    /// running `swing`, so the witness drives it with a mana item held but
    /// the button released and a long `attack_cd` (no re-fire arm runs).
    /// Mirror clone, written not run.
    #[test]
    fn i12_the_mana_regen_delay_reapplies_every_animation_tick() {
        let mut app = staff_app(743, "Ruby Staff"); // mana 9, not spaceGun
        app.mine_down = false; // no use arm; the running swing just decays
        app.mine_prev_down = false;
        // A running swing with the action timer already spent; the long
        // start lock keeps every re-fire gate shut.
        app.swing = 20.0 * ONE_TICK_SECS;
        app.swing_max = 20.0 * ONE_TICK_SECS;
        app.item_time = 0.0;
        app.attack_cd = 60.0 * ONE_TICK_SECS;
        let p = app.player.as_mut().unwrap();
        p.mana = 10;
        p.mana_max = 20;
        p.regen.mana_regen_delay = 0.0;
        let screen = app.screen_size().into();
        update_editing(&mut app, ONE_TICK_SECS, screen);
        // `ApplyManaRegenerationDelay` (`:19398`): (1 - 10/20)*60*4 + 45 = 165,
        // ×0.7 = 115.5 → 115 — THEN the shipped ManaV2 cap (`:19403-19415`,
        // the V2 fix round): >4 clamps to 4, and mana > 0 pins 4. The head
        // re-applied it on this anim tick (the i19 cap-4 re-pin — the
        // pre-V2 115 expectation was the legacy curve).
        let delay = app.player.as_ref().unwrap().regen.mana_regen_delay;
        assert_eq!(delay, 4.0, "the per-tick re-apply pinned the delay (ManaV2 cap)");
        // Drain it by hand, then one more anim tick re-pins it (the swing is
        // still running) — the once-at-pay port would have left it at 0.
        app.player.as_mut().unwrap().regen.mana_regen_delay = 0.0;
        update_editing(&mut app, ONE_TICK_SECS, screen);
        let delay = app.player.as_ref().unwrap().regen.mana_regen_delay;
        assert_eq!(delay, 4.0, "the next anim tick re-applied it again");
        // The spaceGun four skip the delay (`:50589`): with the Meteor set's
        // flag on, the Space Gun's swing never re-applies.
        let mut sg = staff_app(127, "Space Gun");
        sg.mine_down = false;
        sg.mine_prev_down = false;
        sg.swing = 10.0 * ONE_TICK_SECS;
        sg.swing_max = 10.0 * ONE_TICK_SECS;
        sg.attack_cd = 60.0 * ONE_TICK_SECS;
        let p = sg.player.as_mut().unwrap();
        p.space_gun = true;
        p.mana = 10;
        p.mana_max = 20;
        p.regen.mana_regen_delay = 0.0;
        update_editing(&mut sg, ONE_TICK_SECS, screen);
        assert_eq!(
            sg.player.as_ref().unwrap().regen.mana_regen_delay,
            0.0,
            "the spaceGun skip leaves the delay alone"
        );
    }

    // ---- M4-PROJ-6b phase 5: the shoot-path specials -----------------------

    /// The shoot path's cursor world point — the same mapping the fire block
    /// reads (`camera + (mouse - screen/2) / zoom`, `update_editing`'s
    /// `world_pos`).
    fn staff_aim(app: &App) -> Vec2 {
        let (sw, sh) = app.screen_size();
        app.camera + (app.mouse_pos - Vec2::new(sw * 0.5, sh * 0.5)) / app.zoom
    }

    /// The stream state after `n` draws from `seed` (the re-pinned
    /// `draws_after`: splitmix rounds out, `UnifiedRandom` draws in — the
    /// draw-count contract is unchanged).
    fn draws_after(seed: i32, n: usize) -> tr_core::UnifiedRandom {
        let mut rng = tr_core::UnifiedRandom::new(seed);
        for _ in 0..n {
            rng.next();
        }
        rng
    }

    /// Pins the editing pass's `UpdatePlayers` stream to `seed`.
    fn seed_stream(app: &mut App, seed: i32) {
        *app.rngs.stream(crate::app::RngKey::UpdatePlayers) = tr_core::UnifiedRandom::new(seed);
    }

    /// The editing pass's `UpdatePlayers` stream state (the registry's
    /// lazy-create needs the mutable app).
    fn stream_of(app: &mut App) -> tr_core::UnifiedRandom {
        app.rngs
            .stream(crate::app::RngKey::UpdatePlayers)
            .clone()
    }

    #[test]
    fn test_dw_5_1_the_leaf_blower_jitter_is_seed_deterministic() {
        // `Player.cs:49426-49429`: the Leaf Blower's spawn point takes
        // `NextVector2Circular(12, 12)` minus the full velocity vector —
        // the same seed twice gives byte-equal spawns, and the displacement
        // is the velocity's pull (3..27 px back along the aim) inside the
        // 12 px circular jitter.
        let run = || {
            let mut app = staff_app(1178, "Leaf Blower");
            seed_stream(&mut app, 0x1EAF);
            fire_once(&mut app);
            app.projectiles
                .slots()
                .iter()
                .filter(|p| p.active)
                .map(|p| (p.kind, p.pos, p.vel, p.ai))
                .collect::<Vec<_>>()
        };
        let a = run();
        let b = run();
        assert_eq!(a, b, "the jittered spawn is seed-stable");
        assert_eq!(a.len(), 1, "one leaf puff");
        assert_eq!(a[0].0, crate::projectile::LEAF, "the Leaf Blower's shoot");
        // The aim and the un-jittered spawn point, from the same mapping the
        // fire block reads; the fired app's player and cursor did not move.
        let mut app = staff_app(1178, "Leaf Blower");
        seed_stream(&mut app, 0x1EAF);
        fire_once(&mut app);
        let bolt = app.projectiles.slots().iter().find(|p| p.active).unwrap();
        let aim = staff_aim(&app);
        let (cx, cy) = app.player.as_ref().unwrap().center();
        let dir = (aim - Vec2::new(cx, cy)).normalize_or_zero();
        assert_eq!(bolt.vel, dir * 15.0, "the velocity keeps the full shootSpeed");
        let offset = bolt.pos - (Vec2::new(cx, cy) + dir * 10.0);
        let along = offset.dot(dir);
        let perp = (offset - dir * along).length();
        assert!(
            (-27.0 - 1e-3..=-3.0 + 1e-3).contains(&along),
            "the velocity vector is subtracted (along={along})"
        );
        assert!(
            perp <= 12.0 + 1e-3,
            "the circular jitter stays inside 12 (perp={perp})"
        );
    }

    #[test]
    fn test_dw_5_1_the_leaf_blower_jitter_spends_two_circular_draws() {
        // `Utils.NextVector2Circular` (`Utils.cs:1266-1269`): one angle draw
        // (`NextVector2Unit`'s NextFloat) then one radial draw — the whole
        // shot spends exactly the pair (the crit is per-hit now, no spawn
        // draw).
        let mut app = staff_app(1178, "Leaf Blower");
        let seed = 0x1EAF_B07E as i32;
        seed_stream(&mut app, seed);
        fire_once(&mut app);
        assert_eq!(
            app.projectiles.slots().iter().filter(|p| p.active).count(),
            1,
            "one shot"
        );
        assert_eq!(
            stream_of(&mut app),
            draws_after(seed, 2),
            "the circular pair only (crit is per-hit)"
        );
    }

    // ---- S1-P-i21: the hardmode staves' shoot arms (WRITTEN, NOT RUN) -------
    // Code-only mode: no cargo/rustc/tests invoked. Each expected value was
    // verified by re-reading the cited C# and hand arithmetic; a follow-up
    // build must run `cargo test -p tr-app` before merging.

    /// `Player.cs:47647-47662` — the Razorpine fires `2 + Next(3)` needles
    /// (2-4), each renormalized to shootSpeed 12, spawned ahead along the base
    /// aim. The first needle (index 0) has zero spread, so its velocity is the
    /// base aim exactly and its ai0 is `Next(0, 10)` (0-9).
    #[test]
    fn s1_p_i21_the_razorpine_fires_a_renormalized_volley() {
        let mut app = staff_app(1930, "Razorpine");
        seed_stream(&mut app, 0x51A2);
        fire_once(&mut app);
        let needles: Vec<_> = app.projectiles.slots().iter().filter(|p| p.active).collect();
        assert!((2..=4).contains(&needles.len()), "2-4 needles, got {}", needles.len());
        let aim = staff_aim(&app);
        let (cx, cy) = app.player.as_ref().unwrap().center();
        let dir = (aim - Vec2::new(cx, cy)).normalize_or_zero();
        for n in &needles {
            assert_eq!(n.kind, 336, "the Razorpine's shoot");
            assert!((n.vel.length() - 12.0).abs() < 0.05, "renormalized to 12");
            assert!(n.ai[0] >= 0.0 && n.ai[0] < 40.0, "ai0 = Next(0, 10*(i+1))");
        }
        // The first needle: zero spread → the base aim, ai0 in 0..9.
        assert!((needles[0].vel - dir * 12.0).length() < 1e-3, "the first needle keeps the base aim");
        assert!(needles[0].ai[0] < 10.0, "the first needle's ai0 = Next(0, 10)");
        // The spawn lead: the first needle sits `1.75 * count` px ahead along
        // the SCALED aim (`num4`/`num5` = aim * shootSpeed, `Player.cs:47659`),
        // NOT the unit direction — the lead is `shot * 1.75 * count`.
        let spawn = Vec2::new(cx, cy) + dir * 10.0;
        let lead = needles[0].pos - spawn;
        assert!(
            (lead - dir * 12.0 * (1.75 * needles.len() as f32)).length() < 1e-2,
            "the lead along the scaled aim"
        );
    }

    /// `Player.cs:47647-47662` — the Razorpine's draw chain: the count roll
    /// `Next(3)`, then per needle the spread X, spread Y and the ai0 roll —
    /// `1 + 3*count` draws total (the resync spawn-crit conversion dropped
    /// the throw-time crit).
    #[test]
    fn s1_p_i21_the_razorpine_draw_chain_is_seed_deterministic() {
        let seed = 0x51A2_0001;
        // Replay the count to know the draw total.
        let mut state = tr_core::UnifiedRandom::new(seed);
        let count = 2 + state.next_range(0, 3);
        let mut app = staff_app(1930, "Razorpine");
        seed_stream(&mut app, seed);
        fire_once(&mut app);
        assert_eq!(
            stream_of(&mut app),
            draws_after(seed, (1 + 3 * count) as usize),
            "count + 3 draws per needle"
        );
    }

    /// `Player.cs:47663-47692` — the Blizzard Staff fires two icicles from the
    /// sky (600 px above the mounted centre), each with `ai1 = Next(5)` (the
    /// icicle row 0-4) and a downward velocity (the aim Y is flipped positive).
    #[test]
    fn s1_p_i21_the_blizzard_fires_two_sky_icicles() {
        let mut app = staff_app(1931, "Blizzard Staff");
        seed_stream(&mut app, 0xB112);
        fire_once(&mut app);
        let icicles: Vec<_> = app.projectiles.slots().iter().filter(|p| p.active).collect();
        assert_eq!(icicles.len(), 2, "two icicles");
        let (cx, cy) = app.player.as_ref().unwrap().center();
        for (i, n) in icicles.iter().enumerate() {
            assert_eq!(n.kind, 337, "the Blizzard's shoot");
            assert!(n.ai[1] >= 0.0 && n.ai[1] < 5.0, "ai1 = Next(5)");
            assert!(n.vel.y > 0.0, "the icicle falls");
            // The spawn Y is 600 + 100*i above the centre.
            assert!((n.pos.y - (cy - 600.0 - 100.0 * i as f32)).abs() < 1e-2, "the sky height");
            let _ = cx;
        }
    }

    /// `Player.cs:47663-47692` — the Blizzard's draw chain: per icicle
    /// Next(201), Next(-200, 201), spread X, spread Y, ai1 — 10 draws (the
    /// resync spawn-crit conversion dropped the throw-time crit).
    #[test]
    fn s1_p_i21_the_blizzard_draw_chain_is_seed_deterministic() {
        let seed = 0xB112_0002u32 as i32;
        let mut app = staff_app(1931, "Blizzard Staff");
        seed_stream(&mut app, seed);
        fire_once(&mut app);
        assert_eq!(stream_of(&mut app), draws_after(seed, 10), "5 draws per icicle");
    }

    /// `Player.cs:48271-48277` — the Nimbus Rod spawns one cloud at the aim
    /// normalized to shootSpeed 16, with `ai[0..1]` = the cursor world point.
    /// No RNG beyond the crit (one draw).
    #[test]
    fn s1_p_i21_the_nimbus_cloud_carries_the_cursor() {
        let mut app = staff_app(1244, "Nimbus Rod");
        seed_stream(&mut app, 0xC10D);
        fire_once(&mut app);
        let clouds: Vec<_> = app.projectiles.slots().iter().filter(|p| p.active).collect();
        assert_eq!(clouds.len(), 1, "one cloud");
        let c = clouds[0];
        assert_eq!(c.kind, 237, "the Nimbus Rod's shoot");
        let aim = staff_aim(&app);
        assert!((c.ai[0] - aim.x).abs() < 1e-3 && (c.ai[1] - aim.y).abs() < 1e-3,
            "ai[0..1] = the cursor world point");
        assert!((c.vel.length() - 16.0).abs() < 0.05, "shootSpeed 16");
        assert_eq!(stream_of(&mut app), draws_after(0xC10D, 0), "no spawn-time draw (crit is per-hit)");
    }

    /// `Player.cs:48344-48354` — the Bat Scepter fires `Next(2, 4)` bats (2-3),
    /// each at the aim plus `Next(-35, 36) * 0.05` per axis (no renormalize).
    #[test]
    fn s1_p_i21_the_bat_scepter_fires_a_spread_flock() {
        let mut app = staff_app(1801, "Bat Scepter");
        seed_stream(&mut app, 0xBA75);
        fire_once(&mut app);
        let bats: Vec<_> = app.projectiles.slots().iter().filter(|p| p.active).collect();
        assert!((2..=3).contains(&bats.len()), "2-3 bats, got {}", bats.len());
        let aim = staff_aim(&app);
        let (cx, cy) = app.player.as_ref().unwrap().center();
        let dir = (aim - Vec2::new(cx, cy)).normalize_or_zero();
        let base = dir * 10.0; // shootSpeed 10
        for b in &bats {
            assert_eq!(b.kind, 316, "the Bat Scepter's shoot");
            // The spread is at most 35 * 0.05 = 1.75 per axis.
            assert!((b.vel.x - base.x).abs() <= 1.75 + 1e-3, "the X spread");
            assert!((b.vel.y - base.y).abs() <= 1.75 + 1e-3, "the Y spread");
        }
    }

    /// `Player.cs:48344-48354` — the Bat Scepter's draw chain: the count roll
    /// `Next(2, 4)`, then per bat the spread X and spread Y — `1 + 2*count`
    /// draws total (the resync spawn-crit conversion dropped the crit).
    #[test]
    fn s1_p_i21_the_bat_scepter_draw_chain_is_seed_deterministic() {
        let seed = 0xBA75_0003u32 as i32;
        let mut state = tr_core::UnifiedRandom::new(seed);
        let count = state.next_range(2, 4);
        let mut app = staff_app(1801, "Bat Scepter");
        seed_stream(&mut app, seed);
        fire_once(&mut app);
        assert_eq!(
            stream_of(&mut app),
            draws_after(seed, (1 + 2 * count) as usize),
            "count + 2 draws per bat"
        );
    }

    // ---- S1-P-i21 (the i01 non-ammo volley tick): the ItemTimeIsZero re-fire ----
    // WRITTEN, NOT RUN (code-only mode). The C# fires `ItemCheck_Shoot` at every
    // `itemAnimation > 0 && ItemTimeIsZero` tick (`Player.cs:43544` → `:43568`),
    // `ApplyItemTime` re-arming `itemTime = useTime` after each shot (`:47193`).
    // A `useTime < useAnimation` staff therefore volleys inside ONE swing; the
    // pre-fix non-ammo path fired once per swing and never armed `item_time`.

    /// `Item.cs:16545-16546` (useAnimation 18, useTime 6) + `Player.cs:43544`/
    /// `:47193` — the Golden Shower fires THREE streams per swing (the shot at
    /// the swing start, then the `ItemTimeIsZero` re-fires 6 and 12 ticks later),
    /// not one. The mana pays ONCE per swing (`ItemCheck_TryStartUse`,
    /// `Player.cs:52026`), so the whole volley costs the single 7-mana pay.
    #[test]
    fn s1_p_i21_the_golden_shower_volleys_three_streams_per_swing() {
        let mut app = staff_app(1336, "Golden Shower");
        app.player.as_mut().unwrap().mana = 100;
        seed_stream(&mut app, 0x601D);
        let (sw, sh) = app.screen_size();
        let screen = Vec2::new(sw, sh);
        // 18 ticks = one full swing. The streams spawn at ticks 1, 7, 13.
        for _ in 0..18 {
            update_editing(&mut app, ONE_TICK_SECS, screen);
        }
        let streams: Vec<_> = app
            .projectiles
            .slots()
            .iter()
            .filter(|p| p.active && p.kind == 280)
            .collect();
        assert_eq!(streams.len(), 3, "three streams per swing (useTime 6 < useAnimation 18)");
        // The single mana pay: 100 − 7 = 93 (the volley re-fires pay nothing).
        assert_eq!(app.player.as_ref().unwrap().mana, 93, "one 7-mana pay per swing");
    }

    /// `Item.cs:20976-20977` (useAnimation 10, useTime 5) + `Player.cs:47663-
    /// 47692` — the Blizzard Staff fires TWO icicle pairs per swing (four
    /// icicles): the pair at the swing start and the `ItemTimeIsZero` re-fire 5
    /// ticks later. Each pair re-derives its sky point and re-rolls `ai1`.
    #[test]
    fn s1_p_i21_the_blizzard_volleys_two_icicle_pairs_per_swing() {
        let mut app = staff_app(1931, "Blizzard Staff");
        app.player.as_mut().unwrap().mana = 100;
        seed_stream(&mut app, 0xB112);
        let (sw, sh) = app.screen_size();
        let screen = Vec2::new(sw, sh);
        // 10 ticks = one full swing. The pairs spawn at ticks 1 and 6.
        for _ in 0..10 {
            update_editing(&mut app, ONE_TICK_SECS, screen);
        }
        let icicles: Vec<_> = app
            .projectiles
            .slots()
            .iter()
            .filter(|p| p.active && p.kind == 337)
            .collect();
        assert_eq!(icicles.len(), 4, "two pairs per swing (useTime 5 < useAnimation 10)");
        // The single mana pay: 100 − 9 = 91.
        assert_eq!(app.player.as_ref().unwrap().mana, 91, "one 9-mana pay per swing");
    }

    /// `Item.cs:7578-7579` (useAnimation 7 == useTime 7) — the Crystal Storm
    /// fires ONE shard per swing: the `ItemTimeIsZero` re-arm keeps the cadence
    /// at `useTime`, so the next shot lands only when the swing restarts. The
    /// volley fix must not turn the `useTime == useAnimation` staves into
    /// multi-shot weapons.
    #[test]
    fn s1_p_i21_the_crystal_storm_stays_one_shard_per_swing() {
        let mut app = staff_app(518, "Crystal Storm");
        app.player.as_mut().unwrap().mana = 100;
        seed_stream(&mut app, 0xC57);
        let (sw, sh) = app.screen_size();
        let screen = Vec2::new(sw, sh);
        // 7 ticks = one full swing; only the swing-start shot fires.
        for _ in 0..7 {
            update_editing(&mut app, ONE_TICK_SECS, screen);
        }
        let shards: Vec<_> = app
            .projectiles
            .slots()
            .iter()
            .filter(|p| p.active && p.kind == 94)
            .collect();
        assert_eq!(shards.len(), 1, "one shard per swing (useTime == useAnimation)");
    }

    /// `Player.cs:47647-47662` + `:43544`/`:47193` — the Razorpine's volley
    /// re-fire rolls a FRESH count and spreads on the `UpdatePlayers` stream
    /// (the C#'s one `ItemCheck_Shoot` chain runs per shot). The draw chain per
    /// shot is count + 3·count (the resync spawn-crit conversion dropped the
    /// crit); two shots in one swing spend the sum. (Razorpine useTime 8 ==
    /// useAnimation 8 → the num bump makes the swing 9, so one volley per
    /// swing; this pins the per-shot draw order the re-fire inherits.)
    #[test]
    fn s1_p_i21_the_razorpine_volley_draw_order_is_per_shot() {
        let seed = 0x51A2_0004;
        let mut state = tr_core::UnifiedRandom::new(seed);
        let count = 2 + state.next_range(0, 3);
        let mut app = staff_app(1930, "Razorpine");
        app.player.as_mut().unwrap().mana = 100;
        seed_stream(&mut app, seed);
        fire_once(&mut app);
        assert_eq!(
            stream_of(&mut app),
            draws_after(seed, (1 + 3 * count) as usize),
            "count + 3 draws per needle (the re-fire re-rolls the same chain)"
        );
    }

    #[test]
    fn test_dw_5_1_the_nebula_blaze_roll_chain_is_seed_deterministic() {
        // `Player.cs:47119-47130` + `:48778-48792`: the pre-pass roll precedes
        // the spawn-angle rolls on one stream — the same seed twice gives
        // byte-equal spawns.
        let run = || {
            let mut app = staff_app(3542, "Nebula Blaze");
            seed_stream(&mut app, 0xB1A2E);
            fire_once(&mut app);
            app.projectiles
                .slots()
                .iter()
                .filter(|p| p.active)
                .map(|p| (p.kind, p.pos, p.vel, p.damage))
                .collect::<Vec<_>>()
        };
        let a = run();
        let b = run();
        assert_eq!(a, b, "the roll chain is seed-stable");
        assert_eq!(a.len(), 1, "one bolt");
    }

    #[test]
    fn test_dw_5_1_the_nebula_blaze_pre_pass_lands_on_both_sides_across_seeds() {
        // The 20% gate splits the spawn (`Player.cs:47119-47130`): the miss
        // keeps 634 at the plain weapon damage with the decremented speed
        // (`else speed--`), the hit upgrades to 635 at the int ×3 with the
        // full speed; the pre-pass is the stream's FIRST draw (the crit is
        // per-hit now, no spawn draw), the multiplier the third, and the
        // angle stays inside ±(π/4 · 0.7) (`:48778-48792`).
        let mut saw_miss = false;
        let mut saw_hit = false;
        for seed in 0..40u64 {
            let s = (seed.wrapping_mul(0x9E37_79B9_7F4A_7C15)) as i32;
            // The replay: draw 1 is the pre-pass's `Main.rand.Next(100)` —
            // the kind must follow it.
            let mut state = tr_core::UnifiedRandom::new(s);
            let hit = state.next_range(0, 100) < 20;
            let mut app = staff_app(3542, "Nebula Blaze");
            seed_stream(&mut app, s);
            fire_once(&mut app);
            let bolts: Vec<_> = app
                .projectiles
                .slots()
                .iter()
                .filter(|p| p.active)
                .collect();
            assert_eq!(bolts.len(), 1, "seed {seed}: one bolt");
            let bolt = bolts[0];
            assert_eq!(
                bolt.kind,
                if hit { 635 } else { 634 },
                "seed {seed}: the kind follows the pre-pass roll"
            );
            if hit {
                saw_hit = true;
                assert_eq!(bolt.damage, 390, "130 through the int ×3");
            } else {
                saw_miss = true;
                assert_eq!(bolt.damage, 130, "the plain weapon damage");
            }
            // The multiplier is the stream's third draw (0.95 + rand·0.3)
            // over the pre-pass's speed decision; the angle draw (second) is
            // consumed between them.
            let mut state = tr_core::UnifiedRandom::new(s);
            let _ = state.next_range(0, 100);
            let _angle = state.next_float();
            let mult = 0.95 + state.next_float() * 0.3;
            let want = if hit { 6.0 } else { 5.0 } * mult;
            assert!(
                (bolt.vel.length() - want).abs() < 1e-3,
                "seed {seed}: |v|={} against the replayed {want}",
                bolt.vel.length()
            );
            // The aim angle gate: the rotation stays inside ±(π/4 · 0.7).
            let aim = staff_aim(&app);
            let (cx, cy) = app.player.as_ref().unwrap().center();
            let dir = (aim - Vec2::new(cx, cy)).normalize_or_zero();
            let cross = dir.x * bolt.vel.y - dir.y * bolt.vel.x;
            let dot = dir.x * bolt.vel.x + dir.y * bolt.vel.y;
            let dev = cross.atan2(dot);
            assert!(
                dev.abs() <= std::f32::consts::PI / 4.0 * 0.7 + 1e-3,
                "seed {seed}: the aim angle gate (dev={dev})"
            );
            assert_eq!(
                stream_of(&mut app),
                draws_after(s, 3),
                "seed {seed}: exactly pre-pass, angle, multiplier"
            );
        }
        assert!(
            saw_miss && saw_hit,
            "both sides of the 20% gate across seeds (miss={saw_miss} hit={saw_hit})"
        );
    }

    #[test]
    fn test_dw_5_1_the_last_prism_fires_once_per_press_and_the_channel_holds() {
        // The Last Prism has NO shoot special — the C#'s generic spawn
        // (`:49435`) fires once per press edge (`ItemCheck_AutoReuseLogic`
        // `:52877` forces the re-fire only for autoReuse items; 3541 rides
        // reuseDelay 5 + channel), and the Phase-4 holder keep-alive reads
        // the channel the shoot block feeds (`Projectile.cs:66617`).
        let mut app = staff_app(3541, "Last Prism");
        let seed = 0x915E;
        seed_stream(&mut app, seed);
        let (sw, sh) = app.screen_size();
        let screen = Vec2::new(sw, sh);
        // The press edge: one released tick, then the press (`:42733`'s
        // `controlUseItem && releaseUseItem` pair).
        app.mine_down = false;
        update_editing(&mut app, 1.0 / 60.0, screen);
        app.mine_down = true;
        update_editing(&mut app, 1.0 / 60.0, screen);
        let holders: Vec<_> = app
            .projectiles
            .slots()
            .iter()
            .filter(|p| p.active)
            .collect();
        assert_eq!(holders.len(), 1, "one holder per press");
        assert_eq!(
            holders[0].kind,
            crate::projectile::LAST_PRISM,
            "the generic spawn fires 633"
        );
        let p = app.player.as_ref().unwrap();
        assert!(p.channel, "the held channel feeds the holder");
        assert_eq!(p.channel_item, 3541, "the holder's keep-alive item");
        // Holding the button does not re-fire (non-autoReuse) and the
        // channel stays armed for the holder's keep-alive.
        for _ in 0..30 {
            update_editing(&mut app, 1.0 / 60.0, screen);
        }
        assert_eq!(
            app.projectiles.slots().iter().filter(|p| p.active).count(),
            1,
            "the hold does not re-fire"
        );
        assert!(
            app.player.as_ref().unwrap().channel,
            "the hold keeps the channel alive"
        );
        // And no special-case rolls: the whole run spends NO draw on the
        // stream (the crit is per-hit now, on UpdateProjectiles).
        assert_eq!(
            stream_of(&mut app),
            draws_after(seed, 0),
            "no spawn-time draw (crit is per-hit)"
        );
    }

    // ---- S1-P-i20: the early magic staves (ManageShoot census) -------------
    // Tests-as-code: WRITTEN NOT RUN (the task's code-only mode). Each pins
    // the C# `ItemCheck_Shoot` arm it ports, with the `Player.cs` citation.

    /// The Vilethorn (item 64) fires its base segment 7 through the generic
    /// spawn — `shootSpeed 32`, the magic class damage, no special arm
    /// (`Player.cs:49437`'s generic tail; the aiStyle-4 chain is the
    /// projectile's own, pinned in `projectile_ai_tests`).
    #[test]
    fn s1_p_i20_the_vilethorn_fires_its_base_segment() {
        let mut app = staff_app(64, "Vilethorn");
        fire_once(&mut app);
        let bolts: Vec<_> = app
            .projectiles
            .slots()
            .iter()
            .filter(|p| p.active)
            .collect();
        assert_eq!(bolts.len(), 1, "one base segment");
        assert_eq!(bolts[0].kind, crate::projectile::VILE_BASE, "Item.shoot 7");
        assert!(
            (bolts[0].vel.length() - 32.0).abs() < 0.05,
            "shootSpeed 32 (|v|={})",
            bolts[0].vel.length()
        );
        assert_eq!(bolts[0].damage, 10, "the magic damage 10");
        // The generic tail spends NO draw (the resync spawn-crit conversion:
        // the crit is per-hit on UpdateProjectiles, the Vilethorn's generic
        // spawn has no other roll).
        let seed = 0x57A2;
        let mut app = staff_app(64, "Vilethorn");
        seed_stream(&mut app, seed);
        fire_once(&mut app);
        assert_eq!(stream_of(&mut app), draws_after(seed, 0), "no spawn-time draw");
    }

    /// The Band of Starpower (item 111) is an ACCESSORY (`Item.cs:2692-2699`:
    /// `accessory = true`, no shoot/damage/mana) — it is not a weapon, so the
    /// use path never fires it (`registry::is_weapon` is false: no projectile).
    #[test]
    fn s1_p_i20_the_band_of_starpower_is_not_a_weapon() {
        assert!(!crate::registry::is_weapon(111), "no Item.shoot");
        assert_eq!(crate::registry::projectile(111), None, "no projectile");
        let mut app = staff_app(111, "Band of Starpower");
        fire_once(&mut app);
        assert_eq!(
            app.projectiles.slots().iter().filter(|p| p.active).count(),
            0,
            "an accessory fires nothing"
        );
    }

    /// The Aqua Scepter (item 157) fires proj 22 with the lava-bounce LCG seed
    /// in `ai[2]` (`Player.cs:49409-49411`'s `Next(0, 20000)`), `shootSpeed
    /// 12.5`, the magic damage 27. The seed draw rides the generic tail after
    /// the crit.
    #[test]
    fn s1_p_i20_the_aqua_scepter_fires_with_the_lava_seed() {
        let mut app = staff_app(157, "Aqua Scepter");
        fire_once(&mut app);
        let bolts: Vec<_> = app
            .projectiles
            .slots()
            .iter()
            .filter(|p| p.active)
            .collect();
        assert_eq!(bolts.len(), 1, "one stream");
        assert_eq!(bolts[0].kind, crate::projectile::AQUA_SCEPTER, "Item.shoot 22");
        assert!(
            (bolts[0].vel.length() - 12.5).abs() < 0.05,
            "shootSpeed 12.5 (|v|={})",
            bolts[0].vel.length()
        );
        assert_eq!(bolts[0].damage, 27, "the magic damage 27");
        assert!(
            (0.0..20000.0).contains(&bolts[0].ai[2]),
            "the LCG seed is a Next(0, 20000) draw ({})",
            bolts[0].ai[2]
        );
        // The seed is the stream's FIRST draw (the resync spawn-crit
        // conversion dropped the throw-time crit), and the same seed twice
        // gives the same ai[2].
        let seed = 0xA9A5i32;
        let run = || {
            let mut app = staff_app(157, "Aqua Scepter");
            seed_stream(&mut app, seed);
            fire_once(&mut app);
            let b = app.projectiles.slots().iter().find(|p| p.active).unwrap();
            (b.ai[2], stream_of(&mut app))
        };
        let (a_seed, a_stream) = run();
        let (b_seed, b_stream) = run();
        assert_eq!(a_seed, b_seed, "the seed is seed-stable");
        assert_eq!(a_stream, draws_after(seed, 1), "the LCG seed only (crit is per-hit)");
        assert_eq!(a_stream, b_stream, "the stream is seed-stable");
    }

    /// The Bee Gun (item 1121) fires 1-5 bees (`Player.cs:48309-48332`):
    /// `Next(1, 4)` base (1-3), +1 per `Next(6) == 0` (two rolls), +1 more only
    /// while `strongBees` and a third `Next(3) == 0`. Each bee is kind 181 (or
    /// 566 with the Hive Pack), the aim jittered by `Next(-35, 36) * 0.02`, the
    /// damage `beeDamage` (base + `Next(2)`), `magic = true`.
    #[test]
    fn s1_p_i20_the_bee_gun_fires_a_multi_bee_volley() {
        // Without the Hive Pack: 1-5 bees, all kind 181.
        let mut app = staff_app(1121, "Bee Gun");
        fire_once(&mut app);
        let bees: Vec<_> = app
            .projectiles
            .slots()
            .iter()
            .filter(|p| p.active)
            .collect();
        assert!(
            (1..=5).contains(&bees.len()),
            "Next(1,4) + two Next(6) bonuses, no strongBees ({} bees)",
            bees.len()
        );
        for b in &bees {
            assert_eq!(b.kind, crate::projectile::BEE, "the plain bee 181");
            // The damage is the base 9 + Next(2) (0 or 1).
            assert!(
                (9..=10).contains(&b.damage),
                "beeDamage: 9 + Next(2) ({})",
                b.damage
            );
            // The velocity is the aim (shootSpeed 8) jittered by ±0.7.
            let aim = staff_aim(&app);
            let (cx, cy) = app.player.as_ref().unwrap().center();
            let dir = (aim - Vec2::new(cx, cy)).normalize_or_zero();
            let base = dir * 8.0;
            assert!(
                (b.vel.x - base.x).abs() <= 0.7 + 1e-3
                    && (b.vel.y - base.y).abs() <= 0.7 + 1e-3,
                "the aim jitter is Next(-35,36)*0.02 (v={:?})",
                b.vel
            );
        }
    }

    /// The Bee Gun's Hive Pack arm (`strongBees`): the third `Next(3) == 0`
    /// bonus and `beeType`'s `Next(2) == 0` coin flip (`Player.cs:12417-12426`).
    /// A full RNG replay predicts the count and every bee's kind/damage/
    /// knockback; the actual spawns must match draw-for-draw. The draw order
    /// (`Player.cs:48311-48328`): crit, count base `Next(1,4)`, two `Next(6)`
    /// bonuses, the `strongBees` `Next(3)` bonus, then per bee the two
    /// `Next(-35,36)` velocity jitters, `beeType`'s `Next(2)`, `beeDamage`'s
    /// `Next(1,4)` (strong) / `Next(2)` (plain). `beeKB` draws nothing.
    #[test]
    fn s1_p_i20_the_bee_gun_hive_pack_rolls_the_giant_bee() {
        let mut saw_giant = false;
        let mut saw_third_bonus = false;
        for seed in 0..60u64 {
            let s = (seed.wrapping_mul(0x9E37_79B9_7F4A_7C15)) as i32;
            // The replay: predict the count and each bee off the same stream
            // (the resync spawn-crit conversion dropped the throw-time crit).
            let mut rng = tr_core::UnifiedRandom::new(s);
            let mut count = rng.next_range(1, 4);
            if rng.next_range(0, 6) == 0 {
                count += 1;
            }
            if rng.next_range(0, 6) == 0 {
                count += 1;
            }
            if rng.next_range(0, 3) == 0 {
                count += 1;
                saw_third_bonus = true;
            }
            let mut predicted: Vec<(u16, i32, f32)> = Vec::new();
            for _ in 0..count {
                let _vx = rng.next_range(-35, 36);
                let _vy = rng.next_range(-35, 36);
                // beeType: strongBees && Next(2) == 0 → giant 566.
                let strong = rng.next_range(0, 2) == 0;
                let kind = if strong { crate::projectile::GIANT_BEE } else { crate::projectile::BEE };
                // beeDamage: strong → 9 + Next(1,4); plain → 9 + Next(2).
                let dmg = if strong { 9 + rng.next_range(1, 4) } else { 9 + rng.next_range(0, 2) };
                // beeKB: strong → 0.5 + 0.25*1.1; plain → 0.25.
                let kb = if strong { 0.5 + 0.25 * 1.1 } else { 0.25 };
                predicted.push((kind, dmg, kb));
            }
            // The actual fire.
            let mut app = staff_app(1121, "Bee Gun");
            app.player.as_mut().unwrap().strong_bees = true;
            seed_stream(&mut app, s);
            fire_once(&mut app);
            let bees: Vec<_> = app
                .projectiles
                .slots()
                .iter()
                .filter(|p| p.active)
                .collect();
            assert_eq!(bees.len(), predicted.len(), "seed {seed}: the count matches the replay");
            for (b, &(kind, dmg, kb)) in bees.iter().zip(predicted.iter()) {
                assert_eq!(b.kind, kind, "seed {seed}: beeType's coin flip");
                assert_eq!(b.damage, dmg, "seed {seed}: beeDamage");
                assert!(
                    (b.knock_back - kb).abs() < 1e-3,
                    "seed {seed}: beeKB ({} vs {kb})",
                    b.knock_back
                );
                if kind == crate::projectile::GIANT_BEE {
                    saw_giant = true;
                }
            }
            // The whole volley's stream matches the replay's draw count.
            assert_eq!(stream_of(&mut app), rng, "seed {seed}: the stream is draw-exact");
        }
        assert!(saw_giant, "the Hive Pack rolls giant bees across seeds");
        assert!(saw_third_bonus, "the strongBees Next(3) bonus fires across seeds");
    }

    /// The Bee Gun's volley is seed-deterministic: the same seed twice gives
    /// byte-equal spawns (the count rolls, the per-bee velocity jitter, the
    /// beeType coin flip and the beeDamage draw all ride one stream).
    #[test]
    fn s1_p_i20_the_bee_gun_volley_is_seed_deterministic() {
        let run = || {
            let mut app = staff_app(1121, "Bee Gun");
            app.player.as_mut().unwrap().strong_bees = true;
            seed_stream(&mut app, 0xBEE5);
            fire_once(&mut app);
            app.projectiles
                .slots()
                .iter()
                .filter(|p| p.active)
                .map(|p| (p.kind, p.pos, p.vel, p.damage, p.knock_back))
                .collect::<Vec<_>>()
        };
        assert_eq!(run(), run(), "the volley is seed-stable");
    }

    /// The Magic Missile (item 113) channel holds the aim in the REAL use path:
    /// `channel_shoot` carries the held item's shoot id (16), so the aiStyle-9
    /// arm's `HeldItem.shoot == this.type` gate (`Projectile.cs:43241`) passes
    /// and the spell keeps the clamped cursor. (S1-P-i20: the pre-fix port
    /// compared the item id 113 against the proj id 16, so the hold failed.)
    #[test]
    fn s1_p_i20_the_magic_missile_channel_holds_in_the_use_path() {
        let mut app = staff_app(113, "Magic Missile");
        fire_once(&mut app);
        let p = app.player.as_ref().unwrap();
        assert!(p.channel, "the channel item arms the channel");
        assert_eq!(p.channel_item, 113, "the held item's kind");
        assert_eq!(p.channel_shoot, 16, "the held item's shoot id (proj 16)");
        // The fired spell carries the aim in ai[0..1] (the generic spawn's
        // velocity is the aim; the channel hold keeps it on the cursor).
        let spell = app
            .projectiles
            .slots()
            .iter()
            .find(|p| p.active && p.kind == crate::projectile::MAGIC_MISSILE)
            .expect("the Magic Missile fired");
        assert_eq!(spell.kind, 16, "Item.shoot 16");
        // The release clears both channel fields.
        app.mine_down = false;
        fire_once(&mut app);
        let p = app.player.as_ref().unwrap();
        assert!(!p.channel, "the release clears the channel");
        assert_eq!(p.channel_shoot, 0, "no shoot id is being channelled");
    }

    /// The Flamelash (item 218) channel carries its shoot id 34 the same way.
    #[test]
    fn s1_p_i20_the_flamelash_channel_carries_its_shoot_id() {
        let mut app = staff_app(218, "Flamelash");
        fire_once(&mut app);
        let p = app.player.as_ref().unwrap();
        assert!(p.channel, "the channel item arms the channel");
        assert_eq!(p.channel_item, 218, "the held item's kind");
        assert_eq!(p.channel_shoot, 34, "the held item's shoot id (proj 34)");
        let spell = app
            .projectiles
            .slots()
            .iter()
            .find(|p| p.active && p.kind == crate::projectile::FLAMELASH)
            .expect("the Flamelash fired");
        assert_eq!(spell.kind, 34, "Item.shoot 34");
    }

    /// The Water Bolt (item 165), Demon Scythe (item 272) and Space Gun
    /// (item 127) fire their own projectiles through the generic spawn —
    /// `shootSpeed` 4.5 / 0.2 / 10, the magic damage 19 / 35 / 20.
    #[test]
    fn s1_p_i20_the_water_bolt_demon_scythe_space_gun_fire_their_shoot() {
        for (item, name, proj, speed, damage) in [
            (165u16, "Water Bolt", crate::projectile::WATER_BOLT, 4.5f32, 19i32),
            (272, "Demon Scythe", crate::projectile::DEMON_SCYTHE, 0.2, 35),
            (127, "Space Gun", crate::projectile::GREEN_LASER, 10.0, 20),
        ] {
            let mut app = staff_app(item, name);
            fire_once(&mut app);
            let bolts: Vec<_> = app
                .projectiles
                .slots()
                .iter()
                .filter(|p| p.active)
                .collect();
            assert_eq!(bolts.len(), 1, "{name}: one shot");
            assert_eq!(bolts[0].kind, proj, "{name}: Item.shoot {proj}");
            assert!(
                (bolts[0].vel.length() - speed).abs() < 0.05,
                "{name}: shootSpeed {speed} (|v|={})",
                bolts[0].vel.length()
            );
            assert_eq!(bolts[0].damage, damage, "{name}: the magic damage {damage}");
        }
    }

    /// The Space Gun's Meteor-set mana skip (`Player.cs:52695-52698`): with
    /// `spaceGun` armed the shot costs no mana and fires even at zero mana.
    #[test]
    fn s1_p_i20_the_space_gun_meteor_set_skips_mana() {
        let mut app = staff_app(127, "Space Gun");
        app.player.as_mut().unwrap().space_gun = true;
        app.player.as_mut().unwrap().mana = 0;
        fire_once(&mut app);
        assert_eq!(
            app.projectiles.slots().iter().filter(|p| p.active).count(),
            1,
            "the Meteor set fires the Space Gun at zero mana"
        );
        assert_eq!(app.player.as_ref().unwrap().mana, 0, "no mana paid");
    }

    // ---- S1-P-i20: the ItemTimeIsZero volley tick + the wet gate -----------
    // Tests-as-code: WRITTEN NOT RUN (the task's code-only mode).

    /// The Aqua Scepter (item 157, `useTime 8 < useAnimation 16`,
    /// `Item.cs:3252-3268`) fires its whole volley inside ONE swing — the i01
    /// census item. The C# `itemAnimation > 0 && ItemTimeIsZero` gate
    /// (`Player.cs:43544`) fires the first shot on the press tick, then
    /// `ApplyItemTime` (`:47185-47194`) re-arms `itemTime = useTime = 8`, so
    /// the second shot lands 8 ticks later while the swing still runs. The
    /// i12 num bump (`:4466-4469`) stretches the swing to 17 ticks, so the
    /// THIRD re-fire (tick 17, swing 1) still lands inside it — the resync
    /// re-pin: the gate's pre-i12 16-tick swing held only 2 shots. Mana is
    /// paid ONCE per swing (`ItemCheck_TryStartUse`, `:52021`), not per shot.
    #[test]
    fn s1_p_i20_the_aqua_scepter_fires_a_two_shot_volley_per_swing() {
        let mut app = staff_app(157, "Aqua Scepter");
        let (sw, sh) = app.screen_size();
        let screen = Vec2::new(sw, sh);
        let mut log = Vec::new();
        // Hold past the 17-tick swing: the volley is 3 shots (press + 8 + 16).
        hold(&mut app, screen, 20, &mut log);
        assert_eq!(log.len(), 3, "three shots per swing (the volley + the num bump)");
        // The press-frame arming puts the first shot at swing == max (17); the
        // re-arms fire 8 ticks apart at swing 9 and 1.
        assert_eq!(shot_ticks(&log), vec![17, 9, 1], "the volley's shot ticks");
        assert_eq!(shots(&app), 3, "three water streams spawned");
        assert!(
            app.projectiles
                .slots()
                .iter()
                .filter(|p| p.active)
                .all(|p| p.kind == crate::projectile::AQUA_SCEPTER),
            "all are Item.shoot 22"
        );
        // One mana pay per swing (7), not per shot.
        assert_eq!(app.player.as_ref().unwrap().mana, 20 - 7, "one mana pay");
    }

    /// Each volley shot re-rolls the Aqua Scepter's lava-bounce LCG seed
    /// (`Next(0,20000)`, `Player.cs:49409-49411`) — the generic tail's per-shot
    /// draw. The resync spawn-crit conversion dropped the throw-time crit, so
    /// three shots spend exactly three draws, and the `ai[2]` seeds differ
    /// (the stream advanced between them).
    #[test]
    fn s1_p_i20_the_aqua_volley_rolls_crit_and_seed_per_shot() {
        let seed = 0xA9A5i32;
        let mut app = staff_app(157, "Aqua Scepter");
        seed_stream(&mut app, seed);
        let (sw, sh) = app.screen_size();
        let screen = Vec2::new(sw, sh);
        let mut log = Vec::new();
        hold(&mut app, screen, 20, &mut log);
        assert_eq!(log.len(), 3, "the volley fired");
        // The replay: seed, seed, seed — the per-shot generic-tail order
        // (no crit draw; the crit is per-hit on UpdateProjectiles).
        let mut rng = tr_core::UnifiedRandom::new(seed);
        let seed1 = rng.next_range(0, 20000);
        let seed2 = rng.next_range(0, 20000);
        let seed3 = rng.next_range(0, 20000);
        let bolts: Vec<_> = app
            .projectiles
            .slots()
            .iter()
            .filter(|p| p.active)
            .collect();
        assert_eq!(bolts.len(), 3, "three streams");
        assert_eq!(bolts[0].ai[2], seed1 as f32, "the first shot's LCG seed");
        assert_eq!(bolts[1].ai[2], seed2 as f32, "the second shot's LCG seed");
        assert_eq!(bolts[2].ai[2], seed3 as f32, "the third shot's LCG seed");
        assert_ne!(seed1, seed2, "the stream advanced between the shots");
        assert_eq!(stream_of(&mut app), rng, "the stream is draw-exact");
    }

    /// The Flamelash (item 218, shoot 34) refuses the use while wet (non-lava):
    /// `ItemCheck_CheckCanUse_Inner` (`Player.cs:52112-52115`) blocks the fire
    /// spells `{85, 15, 34, 1122}`. No shot, no mana pay, no swing. `base.wet`
    /// is ANY liquid (`WetCollision`, `Collision.cs:1605-1645`); the port's
    /// water/honey/shimmer union stands in (`player.rs:4315-4358`).
    #[test]
    fn s1_p_i20_the_wet_flamelash_refuses_the_use() {
        for wet_kind in ["water", "honey", "shimmer"] {
            let mut app = staff_app(218, "Flamelash");
            let p = app.player.as_mut().unwrap();
            match wet_kind {
                "water" => p.wet = true,
                "honey" => p.honey_wet = true,
                _ => p.shimmer_wet = true,
            }
            let mana_before = p.mana;
            fire_once(&mut app);
            assert_eq!(
                app.projectiles.slots().iter().filter(|p| p.active).count(),
                0,
                "{wet_kind}: the wet Flamelash fires nothing"
            );
            assert_eq!(
                app.player.as_ref().unwrap().mana,
                mana_before,
                "{wet_kind}: no mana paid on the refused use"
            );
            assert!(app.swing <= 0.0, "{wet_kind}: no swing started");
        }
    }

    /// The lava exclusion (`!base.lavaWet`, `Player.cs:52112`): a player in lava
    /// is NOT refused — the Flamelash fires normally.
    #[test]
    fn s1_p_i20_the_lava_wet_flamelash_still_fires() {
        let mut app = staff_app(218, "Flamelash");
        let p = app.player.as_mut().unwrap();
        p.lava_wet = true;
        p.mana = 30; // the Flamelash costs 21; the stock 20 would refuse the pay
        fire_once(&mut app);
        assert_eq!(
            app.projectiles.slots().iter().filter(|p| p.active).count(),
            1,
            "the lava-wet Flamelash fires (the !lavaWet exclusion)"
        );
    }

    /// The dry Flamelash fires and channels its shoot id 34 (the wet gate is
    /// vacuous when dry) — the control for the wet-refusal witnesses.
    #[test]
    fn s1_p_i20_the_dry_flamelash_fires_and_channels() {
        let mut app = staff_app(218, "Flamelash");
        app.player.as_mut().unwrap().mana = 30; // the Flamelash costs 21
        fire_once(&mut app);
        assert_eq!(
            app.projectiles.slots().iter().filter(|p| p.active).count(),
            1,
            "the dry Flamelash fires"
        );
        let p = app.player.as_ref().unwrap();
        assert!(p.channel, "the channel item arms the channel");
        assert_eq!(p.channel_shoot, 34, "the held item's shoot id");
    }

    // --- S1-P-i22: the magic guns + tomes' use arms ------------------------
    // Witnesses WRITTEN, NOT RUN (code-only mode). Every expected value is
    // hand-derived from the cited C# lines; the RNG-order argument replays
    // the draws on a clone of the seeded stream.

    /// The Laser Rifle's PurpleLaser carries `ai0 = Next(3, 6)`
    /// (`Player.cs:49379-49382`) — the stream spends that one draw (the
    /// resync spawn-crit conversion dropped the throw-time crit).
    #[test]
    fn s1_p_i22_the_laser_rifle_laser_carries_the_ai0_roll() {
        let mut app = staff_app(514, "Laser Rifle");
        let seed = 0x1A5E;
        seed_stream(&mut app, seed);
        fire_once(&mut app);
        let bolts: Vec<_> = app.projectiles.slots().iter().filter(|p| p.active).collect();
        assert_eq!(bolts.len(), 1, "one laser");
        assert_eq!(bolts[0].kind, crate::projectile::PURPLE_LASER);
        // The replay: draw 1 the ai0 roll (no crit draw).
        let mut state = tr_core::UnifiedRandom::new(seed);
        let ai0 = state.next_range(3, 6);
        assert_eq!(bolts[0].ai[0], ai0 as f32, "ai0 = Next(3,6)");
        assert!((3.0..6.0).contains(&bolts[0].ai[0]), "the roll range");
        assert_eq!(stream_of(&mut app), draws_after(seed, 1), "the ai0 draw only");
    }

    /// The Poison Staff fires 3 fangs, 4 on a `Next(3) == 0` roll
    /// (`Player.cs:48115-48134`); the index-0 fang still spends its two draws.
    #[test]
    fn s1_p_i22_the_poison_staff_volley_count_and_draws() {
        let mut saw3 = false;
        let mut saw4 = false;
        for seed in 0..40u64 {
            let s = (seed.wrapping_mul(0x9E37_79B9_7F4A_7C15)) as i32;
            let mut state = tr_core::UnifiedRandom::new(s);
            let extra = state.next_range(0, 3) == 0;
            let count = if extra { 4 } else { 3 };
            let mut app = staff_app(1308, "Poison Staff");
            // The staff costs 22 mana — above the rig's stock 20.
            app.player.as_mut().unwrap().mana = 100;
            seed_stream(&mut app, s);
            fire_once(&mut app);
            let fangs: Vec<_> = app.projectiles.slots().iter().filter(|p| p.active).collect();
            assert_eq!(fangs.len(), count, "seed {seed}: the volley count");
            assert!(fangs.iter().all(|p| p.kind == crate::projectile::POISON_FANG));
            // Each fang re-normalizes to shootSpeed 13.5.
            for f in &fangs {
                assert!(
                    (f.vel.length() - 13.5).abs() < 1e-3,
                    "seed {seed}: |v| = {}",
                    f.vel.length()
                );
            }
            // The draws: the count roll + 2·count spread draws (the resync
            // spawn-crit conversion dropped the throw-time crit).
            assert_eq!(
                stream_of(&mut app),
                draws_after(s, 1 + 2 * count),
                "seed {seed}: the draw count"
            );
            if extra { saw4 = true; } else { saw3 = true; }
        }
        assert!(saw3 && saw4, "both volley sizes across seeds");
    }

    /// The Magical Harp rolls the note kind `76 + Next(3)` and spends exactly
    /// Next(3) + two Next(-40,41) + the circular pair (`Player.cs:47392-47423`;
    /// the resync spawn-crit conversion dropped the throw-time crit).
    #[test]
    fn s1_p_i22_the_magical_harp_note_kind_and_draws() {
        let mut app = staff_app(494, "Magical Harp");
        let seed = 0x4A7Bu32 as i32;
        seed_stream(&mut app, seed);
        fire_once(&mut app);
        let notes: Vec<_> = app.projectiles.slots().iter().filter(|p| p.active).collect();
        assert_eq!(notes.len(), 1, "one note");
        let mut state = tr_core::UnifiedRandom::new(seed);
        let kind = 76 + state.next_range(0, 3);
        assert_eq!(notes[0].kind, kind as u16, "the note kind roll");
        assert!([76u16, 77, 78].contains(&notes[0].kind));
        // The note's ai1 is 1 (`:47410`), ai0 the clamped pitch (`:47420`).
        assert_eq!(notes[0].ai[1], 1.0, "ai1 = 1");
        assert!((-1.0..=1.0).contains(&notes[0].ai[0]), "the pitch clamp");
        // Next(3) + Next(-40,41)×2 + circular(angle,radial) = 5 draws.
        assert_eq!(stream_of(&mut app), draws_after(seed, 5), "the draw count");
    }

    /// The Sky Fracture's ring spends the pre-loop angle + radial draws, then
    /// per-iteration radial → probe → angle (`Player.cs:48808-48827`). In open
    /// air the first probe passes, so the run is angle + pre-radial +
    /// loop-radial = 3 draws (the resync spawn-crit conversion dropped the
    /// throw-time crit).
    #[test]
    fn s1_p_i22_the_sky_fracture_ring_draw_order() {
        let mut app = staff_app(3787, "Sky Fracture");
        let seed = 0x5CF7;
        seed_stream(&mut app, seed);
        fire_once(&mut app);
        let bolts: Vec<_> = app.projectiles.slots().iter().filter(|p| p.active).collect();
        assert_eq!(bolts.len(), 1, "one fracture");
        assert_eq!(bolts[0].kind, crate::projectile::SKY_FRACTURE);
        // The replay: angle, pre-loop radial (discarded), loop radial.
        let mut state = tr_core::UnifiedRandom::new(seed);
        let face = state.next_float() * std::f32::consts::TAU;
        let _pre = state.next_float();
        let radius = 20.0 + 40.0 * state.next_float();
        let (cx, cy) = app.player.as_ref().unwrap().center();
        let aim = staff_aim(&app);
        let dir = (aim - Vec2::new(cx, cy)).normalize_or_zero();
        let spawn = Vec2::new(cx, cy) + dir * 10.0;
        let ring = spawn + Vec2::from_angle(face) * radius;
        assert!(
            (bolts[0].pos - ring).length() < 1e-3,
            "the ring point follows the draw order (pos {:?}, ring {:?})",
            bolts[0].pos,
            ring
        );
        assert_eq!(stream_of(&mut app), draws_after(seed, 3), "the three draws");
    }

    /// The Starfury star spawns 600 px above the mounted centre with damage
    /// ×1.5 and knockback 0 (`Player.cs:47251-47266` + `:49095-49108`). The
    /// cursor sits right of the player (outside [Left, Right]), so the side
    /// is −1 and NO Next(2) is spent — only the Next(201) offset. The sword
    /// is non-autoReuse, so the use needs a press edge (the Last Prism
    /// witness's released-then-press shape).
    #[test]
    fn s1_p_i22_the_starfury_star_spawn_and_damage() {
        let mut app = staff_app(65, "Starfury");
        let seed = 0x57A7;
        seed_stream(&mut app, seed);
        let (sw, sh) = app.screen_size();
        let screen = Vec2::new(sw, sh);
        // The press edge: one released tick, then the press.
        app.mine_down = false;
        update_editing(&mut app, 1.0 / 60.0, screen);
        app.mine_down = true;
        update_editing(&mut app, 1.0 / 60.0, screen);
        let stars: Vec<_> = app.projectiles.slots().iter().filter(|p| p.active).collect();
        assert_eq!(stars.len(), 1, "one star");
        assert_eq!(stars[0].kind, crate::projectile::STARFURY);
        let (cx, cy) = app.player.as_ref().unwrap().center();
        assert!((stars[0].pos.y - (cy - 600.0)).abs() < 1e-3, "600 px above");
        // Damage 25 × 1.5 = 37 (the melee class multiplier is 1).
        assert_eq!(stars[0].damage, 37, "damage ×1.5");
        assert_eq!(stars[0].knock_back, 0.0, "knockback 0");
        // Next(201) only — the cursor is outside [Left, Right], no Next(2);
        // the resync spawn-crit conversion dropped the throw-time crit.
        assert_eq!(stream_of(&mut app), draws_after(seed, 1), "the Next(201) offset only");
    }

    /// The Sparkle Guitar with no chaseable target spends crit + the circular
    /// pair (24) + the edge angle + (maybe) the Y-fix draw
    /// (`Player.cs:47994-48025`); the note's ai0/ai1 carry the control point.
    #[test]
    fn s1_p_i22_the_sparkle_guitar_no_target_arc() {
        let mut app = staff_app(4715, "Sparkle Guitar");
        let seed = 0x6A7A;
        seed_stream(&mut app, seed);
        fire_once(&mut app);
        let notes: Vec<_> = app.projectiles.slots().iter().filter(|p| p.active).collect();
        assert_eq!(notes.len(), 1, "one note");
        assert_eq!(notes[0].kind, crate::projectile::SPARKLE_GUITAR);
        // The launch velocity points upward (the edge vector is forced up).
        assert!(notes[0].vel.y < 0.0, "the edge vector is upward");
        // The control point (ai0/ai1) is near the cursor (no target).
        let aim = staff_aim(&app);
        let ctrl = Vec2::new(notes[0].ai[0], notes[0].ai[1]);
        assert!((ctrl - aim).length() <= 24.0 + 1e-3, "the control point jitter");
    }

    /// S1-P-i25 round 2 — `WorldGen.SolidTile2` (`WorldGen.cs:70492-70509`):
    /// an active solid flat block is solid; a half-brick, an actuated-off tile
    /// and a sloped block are not. Gate-merge: the i26 `sentry` module's
    /// `solid_tile2` is the canonical port — the out-of-bounds cell reads
    /// FALSE (the C# try/catch arm; the port's dense array has no in-world
    /// null, so `None` is always the indexed-out case). The i25 witness's
    /// `null reads true` expectation conflated the two C# arms — corrected.
    #[test]
    fn i25_sentry_solid_tile2_matches_the_source_predicate() {
        let mut w = World::new("t".into(), 64, 800);
        let put = |w: &mut World, x: i32, y: i32, t: Tile| {
            let i = w.index(x, y);
            w.tiles[i] = t;
        };
        // A flat full stone block: solid.
        let mut flat = Tile { kind: 1, ..Default::default() };
        flat.flags |= FLAG_ACTIVE;
        put(&mut w, 10, 10, flat);
        assert!(sentry::solid_tile2(&w, 10, 10), "the flat block");
        // A half-brick: NOT solid (the `!halfBrick()` term).
        let mut half = flat;
        half.flags |= tr_core::tile::FLAG_HALF_BRICK;
        put(&mut w, 11, 10, half);
        assert!(!sentry::solid_tile2(&w, 11, 10), "the half-brick");
        // Actuated off: NOT solid (the `!inActive()` term).
        let mut off = flat;
        off.flags |= tr_core::tile::FLAG_INACTIVE;
        put(&mut w, 12, 10, off);
        assert!(!sentry::solid_tile2(&w, 12, 10), "the actuated-off block");
        // A sloped block: NOT solid (the `slope() == 0` term).
        let mut sloped = flat;
        sloped.set_slope(1);
        put(&mut w, 13, 10, sloped);
        assert!(!sentry::solid_tile2(&w, 13, 10), "the sloped block");
        // Inactive (no FLAG_ACTIVE): NOT solid.
        put(&mut w, 14, 10, Tile { kind: 1, ..Default::default() });
        assert!(!sentry::solid_tile2(&w, 14, 10), "the inactive cell");
        // Out of bounds: the try/catch arm reads FALSE (`WorldGen.cs:70503-70507`).
        assert!(!sentry::solid_tile2(&w, -1, 10), "the indexed-out cell");
    }

    /// S1-P-i25 round 2 — `PickSentryLandingSpot` (`Player.cs:49525-49543`):
    /// the scan walks the cursor column (and its two neighbours) down to the
    /// first solid row, steps back one, and returns `row * 16 - 24` at the
    /// cursor's exact X. The 3571/3569/5463 staves skip the scan.
    #[test]
    fn i25_pick_sentry_landing_spot_scans_to_the_floor() {
        let mut w = World::new("t".into(), 64, 800);
        // A floor at row 300 across the three scan columns.
        for x in 29..=31 {
            let i = w.index(x, 300);
            w.tiles[i] = Tile { kind: 1, ..Default::default() };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        // The cursor high above the floor: the scan drops to row 299.
        let (sx, sy) = sentry::pick_sentry_landing_spot(&w, 1572, Vec2::new(480.0, 4000.0));
        assert_eq!(sx, 480.0, "the cursor's exact X");
        assert_eq!(sy, 299.0 * 16.0 - 24.0, "one row above the floor");
        // A side-column solid stops the scan too (the three-column test):
        // only column 31 solid at row 250 → the scan stops at 250, back to 249.
        let i = w.index(31, 250);
        w.tiles[i] = Tile { kind: 1, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
        let (_, sy2) = sentry::pick_sentry_landing_spot(&w, 2366, Vec2::new(480.0, 3000.0));
        assert_eq!(sy2, 249.0 * 16.0 - 24.0, "the side column stops the scan");
        // The skip-scan trio lands at the cursor row (no floor walk).
        for item in [3571u16, 3569, 5463] {
            let (_, sy3) = sentry::pick_sentry_landing_spot(&w, item, Vec2::new(480.0, 4000.0));
            let row = (4000.0f32 / 16.0) as i32;
            assert_eq!(sy3, row as f32 * 16.0 - 24.0, "item {item} skips the scan");
        }
    }

    // ---- S1-P-i32: the tile-validity / adjacency census fixes --------------
    //
    // Witnesses for the three WRONGs the census found in the legacy
    // placement pre-check (`can_place`) and the missing lava gate. Written,
    // not run (code-only mode). Each pins the C# line it restores.

    /// `Player.cs:41253` — the default adjacency arm accepts a neighbour that
    /// is `tileSolid` OR `IsBeam` OR `tileRope` OR the minecart track 314.
    /// The pre-fix port only read `tileSolid`, so a block could not be placed
    /// against a rope / beam / track. Each support kind now accepts; a
    /// non-solid, non-support neighbour (a torch 4) still refuses.
    #[test]
    fn s1_p_i32_adjacency_accepts_beam_rope_and_track_neighbours() {
        // A rig with NO floor under the target and NO walls: the only support
        // is the single neighbour we stage, so the verdict isolates the
        // adjacency predicate. Target (10, 10); the neighbour sits at (11,10).
        for (kind, label) in [
            (124u16, "beam (IsBeam)"),
            (213, "rope (tileRope)"),
            (314, "minecart track"),
            (1, "solid stone (the control)"),
        ] {
            let mut app = flow_app(None, vec![], vec![]);
            app.world = World::new("adj".into(), 40, 40);
            app.world.frame_important = vec![false; td::TILE_COUNT];
            app.player = None; // no overlap gate
            app.inventory[app.selected] = slot(2, 5); // dirt places tile 0 (solid)
            set_tile(&mut app.world, 11, 10, kind);
            assert!(can_place(&app, 10, 10), "{label} must support the block");
        }
        // A torch (4) is none of solid/beam/rope/track: it refuses.
        let mut app = flow_app(None, vec![], vec![]);
        app.world = World::new("adj".into(), 40, 40);
        app.world.frame_important = vec![false; td::TILE_COUNT];
        app.player = None;
        app.inventory[app.selected] = slot(2, 5);
        set_tile(&mut app.world, 11, 10, 4);
        assert!(!can_place(&app, 10, 10), "a torch is no adjacency support");
    }

    /// `Player.cs:41257-41260` — when all four cardinal neighbours fail, the
    /// target cell's OWN wall > 0 still accepts the placement. The pre-fix
    /// port never read the target's wall.
    #[test]
    fn s1_p_i32_the_targets_own_wall_supports_placement() {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = World::new("wall".into(), 40, 40);
        app.world.frame_important = vec![false; td::TILE_COUNT];
        app.player = None;
        app.inventory[app.selected] = slot(2, 5);
        // No neighbours at all; only the target's own background wall.
        let i = app.world.index(10, 10);
        app.world.tiles[i].wall = 2;
        assert!(can_place(&app, 10, 10), "the cell's own wall accepts");
        // Without the wall and without neighbours: refused.
        app.world.tiles[i].wall = 0;
        assert!(!can_place(&app, 10, 10), "no support at all refuses");
    }

    /// `Collision.EmptyTile` (`Collision.cs:1373-1379`) — an NPC hitbox over
    /// the cell refuses a SOLID placement. The pre-fix port only checked the
    /// player. The blue slime's 24x18 box (`npc/data.rs:2500`, scale 1.0)
    /// covers the target cell.
    #[test]
    fn s1_p_i32_an_npc_over_the_cell_refuses_a_solid_block() {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = object_world();
        app.player = None;
        app.inventory[app.selected] = slot(2, 5); // solid dirt
        // Floor below supports the target (10, 19); the slime sits on it.
        let npc = Npc::new(NpcKind::Slime, 10.0 * TILE_PX, 19.0 * TILE_PX, 1, 0.0);
        app.npcs.push(npc);
        assert!(!can_place(&app, 10, 19), "the slime's box refuses the block");
        // Move the slime clear: the same cell now places.
        app.npcs[0].x = 30.0 * TILE_PX;
        assert!(can_place(&app, 10, 19), "a clear cell accepts");
    }

    /// `WorldGen.cs:59373` — `EmptyTile` is consulted only for SOLID
    /// placements (`forced || EmptyTile || !tileSolid[num] || …`); a
    /// non-solid tile bypasses the overlap. A torch (tile 4, non-solid)
    /// placed into the player's own cell is NOT refused by the overlap gate
    /// (the pre-fix port refused every kind over the player).
    #[test]
    fn s1_p_i32_a_non_solid_tile_bypasses_the_player_overlap() {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = object_world();
        // Park the player so its 20x42 box covers the target (10, 19).
        let mut p = Player::spawn(&app.world);
        p.x = 10.0 * TILE_PX;
        p.y = 18.0 * TILE_PX;
        app.player = Some(p);
        // Torch item 8 places tile 4 (non-solid): the overlap gate stands down.
        app.inventory[app.selected] = slot(8, 5);
        assert!(can_place(&app, 10, 19), "a torch is not blocked by the player box");
        // The same cell with a SOLID block item IS refused.
        app.inventory[app.selected] = slot(2, 5);
        assert!(!can_place(&app, 10, 19), "a solid block is blocked by the player box");
    }

    /// `Player.cs:41489-41504` (`IsBlockedByLava`) + `:39743` — a solid tile
    /// refuses an inactive lava cell outright; a non-solid tile answers
    /// `CheckLiquidPlacement`. The pre-fix port had no lava gate at all.
    #[test]
    fn s1_p_i32_lava_refuses_solid_blocks_and_water_death_torches() {
        // The pure predicate: a solid createTile in lava always blocks.
        let mut lava = tr_core::tile::Tile::default();
        lava.liquid = 200;
        lava.set_liquid_kind(tr_core::tile::LiquidKind::Lava);
        assert!(is_blocked_by_lava(0, 0, &lava), "dirt (solid) in lava blocks");
        assert!(is_blocked_by_lava(1, 0, &lava), "stone (solid) in lava blocks");
        // A dry cell never blocks.
        let dry = tr_core::tile::Tile::default();
        assert!(!is_blocked_by_lava(0, 0, &dry), "a dry cell never blocks");
        // Torch (4) is non-solid and `tileWaterDeath`/`lavaDeath`: the global
        // LiquidPlace fallback refuses it in lava (style 0, no sub-tile).
        assert!(is_blocked_by_lava(4, 0, &lava), "a plain torch refuses lava");
        // The full item route: a dirt stack over a lava cell refuses and keeps
        // the item; the cell stays empty.
        let mut app = flow_app(None, vec![], vec![]);
        app.world = object_world();
        app.player = None;
        app.inventory[app.selected] = slot(2, 5);
        let i = app.world.index(10, 19);
        app.world.tiles[i].liquid = 200;
        app.world.tiles[i].set_liquid_kind(tr_core::tile::LiquidKind::Lava);
        assert!(!use_tile_item(&mut app, 10, 19), "lava refuses the dirt");
        assert_eq!(app.inventory[app.selected].count, 5, "the item is kept");
        assert!(!app.world.tile(10, 19).unwrap().active(), "nothing written");
    }

    /// `Player.cs:39753-39760` (`PlaceThing_IsReplaceableBlock`) — the
    /// replaceable-block predicate the lava gate's OR chain reads: a tileCut
    /// target is replaceable except 484/711, the drip band 373-375 + 461/709
    /// always, else `BreakableWhenPlacing`.
    #[test]
    fn s1_p_i32_replaceable_block_matches_the_csharp_predicate() {
        use tr_core::tileobject::is_replaceable_block;
        assert!(is_replaceable_block(3), "grass is tileCut");
        assert!(is_replaceable_block(82), "a herb is tileCut");
        assert!(!is_replaceable_block(484), "484 is the named tileCut exemption");
        assert!(!is_replaceable_block(711), "711 is the named tileCut exemption");
        for k in [373u16, 374, 375, 461, 709] {
            assert!(is_replaceable_block(k), "drip {k} is always replaceable");
        }
        assert!(is_replaceable_block(186), "a large pile is BreakableWhenPlacing");
        assert!(!is_replaceable_block(1), "stone is not replaceable");
    }

    /// `Player.cs:41332-41345` — the platform fallback: when the default
    /// adjacency arm refuses, ANY active tile in the 3x3 (diagonals included)
    /// accepts a platform-placing item. A torch neighbour (4) is no default
    /// support, but its diagonal presence still carries the platform. The
    /// pre-fix port had no fallback, so the platform refused.
    #[test]
    fn s1_p_i32_platform_fallback_accepts_any_3x3_active_tile() {
        // Wood Platform item 94 places tile 19 (a SET_PLATFORMS member).
        assert_eq!(registry::place_tile(94), Some(19));
        let mut app = flow_app(None, vec![], vec![]);
        app.world = World::new("plat".into(), 40, 40);
        app.world.frame_important = vec![false; td::TILE_COUNT];
        app.player = None;
        app.inventory[app.selected] = slot(94, 5);
        // A torch at the DIAGONAL (11, 11): no cardinal support, no wall.
        set_tile(&mut app.world, 11, 11, 4);
        assert!(can_place(&app, 10, 10), "the diagonal torch carries the platform");
        // The same diagonal with a BLOCK item (dirt, no fallback) refuses.
        app.inventory[app.selected] = slot(2, 5);
        assert!(!can_place(&app, 10, 10), "a block gets no platform fallback");
        // Clear the torch: even the platform now refuses (no 3x3 support).
        let i = app.world.index(11, 11);
        app.world.tiles[i] = tr_core::tile::Tile::default();
        app.inventory[app.selected] = slot(94, 5);
        assert!(!can_place(&app, 10, 10), "no support at all refuses the platform");
    }
}

/// `Player.ItemCheck_UseBossSpawners`'s eye arm (`Player.cs:43759-43771`):
/// the night gate, the roar (`SoundEngine.PlaySound(15, ...)`, `SoundID.Roar`
/// = 15), then `NPC.SpawnOnPlayer(onWhichPlayer, 4)`. The
/// `SummonItemCheck` (`Player.cs:42162-42176`) refuses while an EoC is
/// already alive; `ApplyItemTime`'s re-fire guard rides the swing timers
/// (the food/potion arms' shape).
/// `SummonItemCheck` (`Player.cs:42162-42179`): the item's own boss
/// living anywhere refuses the use — the pre-hardmode summon pairs.
fn summon_item_check(app: &App, item: u16) -> bool {
    let boss = match item {
        43 => NpcKind::EyeOfCthulhu,
        70 => NpcKind::EaterOfWorldsHead,
        560 => NpcKind::KingSlime,
        1133 => NpcKind::QueenBee,
        1331 => NpcKind::BrainOfCthulhu,
        // 544 refuses while EITHER twin lives (`:42168-42169`'s 125/126
        // pair); 556 rides the Destroyer 134, 557 the Prime 127.
        544 => NpcKind::Retinazer,
        556 => NpcKind::TheDestroyer,
        557 => NpcKind::SkeletronPrime,
        _ => return true,
    };
    !app.npcs
        .iter()
        .any(|n| n.hp > 0 && (n.kind == boss || (item == 544 && n.kind == NpcKind::Spazmatism)))
}

/// The pre-hardmode boss spawners (`ItemCheck_UseBossSpawners`,
/// `Player.cs:43751-43900`): the Worm Food 70 (`ZoneCorrupt` -> the EoW
/// head 13), the Slime Crown 560 (ungated -> the King 50), the
/// Abeemination 1133 (ungated -> the Queen Bee 222; the vanilla's
/// Item173 sound rides the roar — the port's bank holds the one), the
/// Bloody Spine 1331 (`ZoneCrimson` -> the Brain 266). The mech trio
/// 544/556/557 and the later
/// 4988/5120/5334 = the follow-ups. `NPC.SpawnOnPlayer` puts the boss at
/// the player; the EoW rides the worm engine (the chain assembles in the
/// worm pass); the Brain arrives alone (the Creepers' orbit = the
/// follow-up).
pub(crate) fn use_boss_summon_item(app: &mut App) -> bool {
    let held = app.inventory[app.selected];
    let item = held.kind;
    if !matches!(item, 70 | 560 | 1133 | 1331 | 544 | 556 | 557) {
        return false;
    }
    if !summon_item_check(app, item) {
        return false;
    }
    // The biome gates (`:43775`'s ZoneCorrupt arm, `:43889`'s ZoneCrimson;
    // 560/1133 run ungated). The mech trio swaps the biome for the clock:
    // `!Main.IsItDay()` (`:43838`/`:43849`/`:43860`); the 1.4.5 item
    // variant's DisabledBossSummonVariant folds out (no variant system).
    let zone_ok = match item {
        70 => app.scene_metrics.zone_corrupt,
        1331 => app.scene_metrics.zone_crimson,
        544 | 556 | 557 => !app.world.day_time,
        _ => true,
    };
    if !zone_ok {
        return false;
    }
    let (px, py) = app
        .player
        .as_ref()
        .map(|p| p.center())
        .unwrap_or((0.0, 0.0));
    let xt = ((px / TILE_PX) as i32).clamp(2, app.world.width - 3);
    let yt = ((py / TILE_PX) as i32).clamp(2, app.world.height - 3);
    let boss = match item {
        70 => {
            let mut h = Npc::spawn_worm_head(&app.world, xt, yt, NpcKind::EaterOfWorldsHead);
            h.hp = h.max_hp;
            h.worm_left = 65;
            Some(h)
        }
        560 => {
            // SpawnOnPlayer's at-the-player semantics — the surface scan of
            // spawn_king_slime refuses the deep/indoor spawns the vanilla
            // allows.
            let mut k = Npc::new(NpcKind::KingSlime, px, py, 1, 0.0);
            k.hp = k.max_hp;
            Some(k)
        }
        1133 => {
            let mut b = Npc::new(NpcKind::QueenBee, px, py, 1, 0.0);
            b.hp = b.max_hp;
            Some(b)
        }
        // The mech trio (`:43838-43876`): the Eye spawns BOTH eyes
        // (SpawnOnPlayer(125) + SpawnOnPlayer(126)); the Worm rides the
        // 80-budget chain; the Skull lands as the Prime. The port pushes
        // the pair through the one returned boss + the direct list push.
        544 => {
            let mut a = Npc::new(NpcKind::Retinazer, px, py, 1, 0.0);
            a.hp = a.max_hp;
            let mut b = Npc::new(NpcKind::Spazmatism, px, py, 1, 0.0);
            b.hp = b.max_hp;
            app.npcs.push(a);
            Some(b)
        }
        556 => {
            let mut h = Npc::spawn_worm_head(&app.world, xt, yt, NpcKind::TheDestroyer);
            h.hp = h.max_hp;
            h.worm_left = 80;
            Some(h)
        }
        557 => {
            let mut b = Npc::new(NpcKind::SkeletronPrime, px, py, 1, 0.0);
            b.hp = b.max_hp;
            Some(b)
        }
        _ => {
            let mut b = Npc::new(NpcKind::BrainOfCthulhu, px, py, 1, 0.0);
            b.hp = b.max_hp;
            Some(b)
        }
    };
    let Some(boss) = boss else {
        return false;
    };
    app.audio.play("Roar_0");
    app.inventory[app.selected].count = app.inventory[app.selected].count.saturating_sub(1);
    if app.inventory[app.selected].count == 0 {
        app.inventory[app.selected].kind = 0;
    }
    app.npcs.push(boss);
    // `ApplyItemTime(sItem)` (`Player.cs:43759` etc.): the summon re-arms the
    // ACTION cadence only — the swing keeps running. The PREFIXED instance
    // value (`Item.cs:578`), read off the pre-consume slot.
    app.item_time = crate::prefix::prefixed_stats(item, held.prefix)
        .map(|s| s.use_time)
        .or_else(|| registry::get(item).map(|d| d.use_time as i32))
        .filter(|t| *t > 0)
        .map(|t| t as f32 * ONE_TICK_SECS)
        .unwrap_or(app.swing_max);
    app.mine_target = None;
    app.mine_timer = 0.0;
    true
}

/// The Goblin Battle Standard (`Player.ItemCheck_UseEventItems`,
/// `Player.cs:43920-43940`): the use through `CanStartInvasion(1,
/// ignoreDelay: true)` — the refusal leaves the stack intact — then
/// `StartInvasion()` sizes `80 + 40 x the qualifying players` and the
/// stack pays one (`Item.consumable`); the roar rides the eye's sound.
pub(crate) fn use_goblin_battle_standard(app: &mut App) -> bool {
    let Some(player) = app.player.as_ref() else {
        return false;
    };
    let life_max = player.max_hp;
    if !crate::world_sim::npcs::can_start_invasion(&app.world, life_max) {
        return false;
    }
    app.world.invasion_delay = 0;
    crate::world_sim::npcs::start_invasion_in(
        app,
        1,
        1,
        Some(crate::app::RngKey::UpdatePlayers),
    );
    app.audio.play("Roar_0");
    app.inventory[app.selected].count = app.inventory[app.selected].count.saturating_sub(1);
    if app.inventory[app.selected].count == 0 {
        app.inventory[app.selected].kind = 0;
    }
    // `ApplyItemTime(sItem)` (`Player.cs:43925`): the ACTION cadence re-arms,
    // the swing keeps running (`ItemTimeIsZero && itemAnimation > 0` gate).
    app.item_time = crate::item::use_time_secs(361);
    app.mine_target = None;
    app.mine_timer = 0.0;
    true
}

/// `ItemCheck_UseBossSpawners`' Frost Legion arm (`Player.cs:43941-43956`):
/// the Snow Globe (602) — the `CanStartInvasion(2, ignoreDelay: true)` gate
/// (`invasionType == 0` + a 200+ max-hp player; the delay folds out), the
/// roar (`SoundID.Roar` 15), then `invasionDelay = 0; StartInvasion(2)` —
/// the march state rides the World (the morning-roll resolve's shape).
/// The Celestial Sigil (`Player.cs:44028-44034`'s `ItemCheck_UseBossSpawners`
/// arm): the item 3601 gates on `downedGolemBoss && hardMode && !AnyDanger()
/// && !AnyoneNearCultists()` and calls `WorldGen.StartImpendingDoom(720)` —
/// the countdown's item value (`NPC.cs:6037`).
pub(crate) fn use_celestial_sigil(app: &mut App) -> bool {
    if app.world.moon_lord_countdown > 0 || app.world.lunar_apocalypse_is_up {
        return false;
    }
    if !app.world.downed_golem || !app.world.hard_mode {
        return false;
    }
    // `!AnyDanger()`: no invasion and no active event (`NPC.cs:80243`).
    if app.world.invasion_type != 0 || app.world.dd2_event_ongoing {
        return false;
    }
    // `!AnyoneNearCultists()` (`NPC.cs:80290-80313`).
    if crate::world_sim::lunar::anyone_near_cultists(app) {
        return false;
    }
    app.world.moon_lord_countdown = crate::world_sim::lunar::SIGIL_COUNTDOWN;
    app.world.max_moon_lord_countdown = crate::world_sim::lunar::SIGIL_COUNTDOWN;
    crate::chat::broadcast_death_text(
        &mut app.chat,
        "The air is growing colder...",
        crate::chat::COLOR_COMMAND,
    );
    app.inventory[app.selected].count = app.inventory[app.selected].count.saturating_sub(1);
    if app.inventory[app.selected].count == 0 {
        app.inventory[app.selected].kind = 0;
    }
    // `ApplyItemTime(sItem)` (`Player.cs:44031`): the action cadence re-arms.
    app.item_time = crate::item::use_time_secs(3601);
    app.mine_target = None;
    true
}

pub fn use_snow_globe(app: &mut App) -> bool {
    // `Player.ItemCheck_UseEventItems`' Snow Globe arm (`Player.cs:43940-43951`):
    // `CanStartInvasion(2, ignoreDelay: true)` then `invasionDelay = 0` +
    // `StartInvasion(2)` — the shared body derives the size and the side roll
    // (`Main.cs:64435-64486`) on the `UpdatePlayers` stream.
    let life_max = app.player.as_ref().map(|p| p.max_hp).unwrap_or(0);
    if !crate::world_sim::npcs::can_start_invasion(&app.world, life_max) {
        return false;
    }
    app.audio.play("Roar_0");
    app.world.invasion_delay = 0;
    crate::world_sim::npcs::start_invasion_in(
        app,
        1,
        2,
        Some(crate::app::RngKey::UpdatePlayers),
    );
    app.inventory[app.selected].count = app.inventory[app.selected].count.saturating_sub(1);
    if app.inventory[app.selected].count == 0 {
        app.inventory[app.selected].kind = 0;
    }
    // `ApplyItemTime(sItem)` (`Player.cs:43942`): the action cadence re-arms.
    app.item_time = crate::item::use_time_secs(602);
    app.mine_target = None;
    app.mine_timer = 0.0;
    true
}

/// The shared moon-item use: `Player.ApplyItemTime`'s re-arm
/// (`Player.cs:43976`/`:44043` — `SetItemTime(useTime)`, `:4416-4418`) plus
/// `SoundEngine.PlaySound(15, ...)` — the `SoundID.Roar` (15) both C# arms
/// play. The C# gate is `ItemTimeIsZero && itemAnimation > 0`: the ACTION
/// re-arms inside the running swing, the animation is untouched.
fn apply_moon_item(app: &mut App, kind: u16) {
    app.audio.play("Roar_0");
    app.inventory[app.selected].count = app.inventory[app.selected].count.saturating_sub(1);
    if app.inventory[app.selected].count == 0 {
        app.inventory[app.selected].kind = 0;
    }
    app.item_time = crate::item::use_time_secs(kind);
    app.mine_target = None;
    app.mine_timer = 0.0;
}

/// The Pumpkin Moon Medallion 1844 (`ItemCheck_UseEventItems`,
/// `Player.cs:43974-43985`): `!dayTime && !pumpkinMoon && !snowMoon &&
/// !DD2Event.Ongoing` — the refusal is free; otherwise `Main.startPumpkinMoon`.
/// The `Main.NewText` row is UI — out.
pub(crate) fn use_pumpkin_moon_medallion(app: &mut App) -> bool {
    if app.world.day_time
        || app.world.pumpkin_moon
        || app.world.snow_moon
        || app.world.dd2_event_ongoing
    {
        return false;
    }
    apply_moon_item(app, 1844);
    app.world.start_pumpkin_moon();
    true
}

/// The Naughty Present 1958 (`Player.cs:44041-44052`): the snow twin — same
/// gates, `Main.startSnowMoon`.
pub(crate) fn use_naughty_present(app: &mut App) -> bool {
    if app.world.day_time
        || app.world.pumpkin_moon
        || app.world.snow_moon
        || app.world.dd2_event_ongoing
    {
        return false;
    }
    apply_moon_item(app, 1958);
    app.world.start_snow_moon();
    true
}

pub(crate) fn use_suspicious_looking_eye(app: &mut App) -> bool {
    let eye_alive = app
        .npcs
        .iter()
        .any(|n| n.kind == NpcKind::EyeOfCthulhu && n.hp > 0);
    if app.world.day_time || eye_alive {
        return false;
    }
    let Some(player) = app.player.as_ref() else {
        return false;
    };
    let (pcx, pcy) = player.center();
    let Some(eye) = Npc::spawn_eye_of_cthulhu(&app.world, pcx, pcy) else {
        return false;
    };
    app.audio.play("Roar_0");
    app.inventory[app.selected].count = app.inventory[app.selected].count.saturating_sub(1);
    if app.inventory[app.selected].count == 0 {
        app.inventory[app.selected].kind = 0;
    }
    // `ApplyItemTime(sItem)` (`Player.cs:43774`): the action cadence re-arms
    // inside the running swing (`ItemTimeIsZero && itemAnimation > 0`).
    app.item_time = crate::item::use_time_secs(43);
    app.mine_target = None;
    app.mine_timer = 0.0;
    app.npcs.push(eye);
    true

}

#[cfg(test)]
mod loot_summon_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;

    #[test]
    fn the_boss_summon_items_gate_and_consume() {
        // `ItemCheck_UseBossSpawners` (`Player.cs:43751-43900`): the
        // ungated crown/abee, the corruption/crimson gates, the
        // SummonItemCheck refusal, the stack pay.
        let mut app = flow_app(None, vec![], vec![]);
        let mut player = crate::player::Player::spawn(&app.world);
        player.x = 64.0;
        player.y = 64.0;
        app.player = Some(player);
        // The Worm Food outside the corruption refuses free.
        app.inventory[app.selected] = ItemSlot {
            kind: 70,
            name: "Worm Food".into(),
            count: 1,
            prefix: 0,
        };
        assert!(!use_boss_summon_item(&mut app), "the corruption gate holds");
        assert_eq!(app.inventory[app.selected].count, 1, "the refusal is free");
        app.scene_metrics.zone_corrupt = true;
        assert!(use_boss_summon_item(&mut app));
        assert!(
            app.npcs
                .iter()
                .any(|n| n.kind == NpcKind::EaterOfWorldsHead && n.hp > 0)
        );
        assert_eq!(app.inventory[app.selected].count, 0, "the use paid one");
        // The Bloody Spine under the corruption refuses (the crimson gate).
        app.inventory[app.selected] = ItemSlot {
            kind: 1331,
            name: "Bloody Spine".into(),
            count: 1,
            prefix: 0,
        };
        assert!(!use_boss_summon_item(&mut app), "the crimson gate holds");
        app.scene_metrics.zone_crimson = true;
        assert!(use_boss_summon_item(&mut app));
        assert!(
            app.npcs
                .iter()
                .any(|n| n.kind == NpcKind::BrainOfCthulhu && n.hp > 0)
        );
        // The re-summon refuses while the boss lives (`SummonItemCheck`).
        app.inventory[app.selected] = ItemSlot {
            kind: 1331,
            name: "Bloody Spine".into(),
            count: 1,
            prefix: 0,
        };
        assert!(!use_boss_summon_item(&mut app), "the living brain refuses");
        assert_eq!(app.inventory[app.selected].count, 1);
        // The ungated Slime Crown: the first use summons the king.
        app.inventory[app.selected] = ItemSlot {
            kind: 560,
            name: "Slime Crown".into(),
            count: 1,
            prefix: 0,
        };
        assert!(use_boss_summon_item(&mut app));
        assert!(
            app.npcs
                .iter()
                .any(|n| n.kind == NpcKind::KingSlime && n.hp > 0)
        );
    }

    #[test]
    fn the_bloody_spine_brain_spawns_swarm_and_shields() {
        // `AI_054`'s init + the shield: the Bloody Spine's brain gets the
        // 20-Creeper swarm at the first state pass (`localAI[0]` 0 -> 1),
        // the shield holds while a Creeper lives (`dontTakeDamage`), and
        // the last Creeper's death frees the brain.
        let mut app = flow_app(None, vec![], vec![]);
        let mut player = crate::player::Player::spawn(&app.world);
        player.x = 64.0;
        player.y = 64.0;
        app.player = Some(player);
        app.scene_metrics.zone_crimson = true;
        app.inventory[app.selected] = ItemSlot {
            kind: 1331,
            name: "Bloody Spine".into(),
            count: 1,
            prefix: 0,
        };
        assert!(use_boss_summon_item(&mut app));
        crate::world_sim::npcs::update_brain_state(&mut app);
        let creepers = app
            .npcs
            .iter()
            .filter(|n| n.kind == NpcKind::Creeper && n.hp > 0)
            .count();
        assert_eq!(creepers, 20, "the init swarm");
        let brain = app
            .npcs
            .iter()
            .find(|n| n.kind == NpcKind::BrainOfCthulhu)
            .expect("the brain");
        assert!(brain.dont_take_damage, "the Creeper shield holds");
        assert!(app.world.brain_present);
        // The shielded brain refuses the hit (`StrikeNPC`'s no-op).
        let mut brain_mut = app
            .npcs
            .iter_mut()
            .find(|n| n.kind == NpcKind::BrainOfCthulhu)
            .unwrap();
        let hp_before = brain_mut.hp;
        assert!(
            !brain_mut.hurt(50, brain_mut.x - 10.0, 5.0, false, false),
            "the shield refuses"
        );
        assert_eq!(brain_mut.hp, hp_before);
        drop(brain_mut);
        // The last Creeper's death frees the brain.
        for n in app.npcs.iter_mut() {
            if n.kind == NpcKind::Creeper {
                n.hp = 0;
            }
        }
        crate::world_sim::npcs::update_brain_state(&mut app);
        let mut brain_mut = app
            .npcs
            .iter_mut()
            .find(|n| n.kind == NpcKind::BrainOfCthulhu)
            .unwrap();
        assert!(!brain_mut.dont_take_damage, "the shield falls");
        assert!(brain_mut.hurt(50, brain_mut.x - 10.0, 5.0, false, false));
        assert!(brain_mut.hp < hp_before);
    }

    #[test]
    fn the_mech_summons_night_gated_pair_and_refusals() {
        // `ItemCheck_UseBossSpawners`' mech arms (`Player.cs:43838-43876`):
        // day refuses free; night summons — the Eye spawns BOTH eyes, the
        // Worm the 80-budget chain, the Skull the Prime; the living-boss
        // refusal rides `SummonItemCheck`'s 125/126 pair.
        let mut app = flow_app(None, vec![], vec![]);
        let mut player = crate::player::Player::spawn(&app.world);
        player.x = 64.0;
        player.y = 64.0;
        app.player = Some(player);
        app.world.day_time = true;
        app.inventory[app.selected] = ItemSlot {
            kind: 544,
            name: "Mechanical Eye".into(),
            count: 1,
            prefix: 0,
        };
        assert!(!use_boss_summon_item(&mut app), "day refuses");
        assert_eq!(app.inventory[app.selected].count, 1, "the refusal is free");
        app.world.day_time = false;
        assert!(use_boss_summon_item(&mut app));
        assert!(
            app.npcs
                .iter()
                .any(|n| n.kind == NpcKind::Retinazer && n.hp > 0)
        );
        assert!(
            app.npcs
                .iter()
                .any(|n| n.kind == NpcKind::Spazmatism && n.hp > 0)
        );
        assert_eq!(app.inventory[app.selected].count, 0, "the use paid one");
        // Either twin living refuses the re-summon (`:42168-42169`).
        app.inventory[app.selected] = ItemSlot {
            kind: 544,
            name: "Mechanical Eye".into(),
            count: 1,
            prefix: 0,
        };
        assert!(!use_boss_summon_item(&mut app), "the living twins refuse");
        assert_eq!(app.inventory[app.selected].count, 1);
        // The Mechanical Worm rides the chain engine.
        app.inventory[app.selected] = ItemSlot {
            kind: 556,
            name: "Mechanical Worm".into(),
            count: 1,
            prefix: 0,
        };
        assert!(use_boss_summon_item(&mut app));
        let head = app
            .npcs
            .iter()
            .find(|n| n.kind == NpcKind::TheDestroyer)
            .expect("the destroyer head");
        assert_eq!(head.worm_left, 80, "the mech worm budget");
        // The Mechanical Skull lands the Prime.
        app.inventory[app.selected] = ItemSlot {
            kind: 557,
            name: "Mechanical Skull".into(),
            count: 1,
            prefix: 0,
        };
        assert!(use_boss_summon_item(&mut app));
        assert!(
            app.npcs
                .iter()
                .any(|n| n.kind == NpcKind::SkeletronPrime && n.hp > 0)
        );
    }
    #[test]
    fn the_battle_standard_gates_and_sizing() {
        // `Main.CanStartInvasion` (`Main.cs:64414-64432`): the 200-life
        // gate; `StartInvasion(1)` (`:64435-64486`): 80 + 40 x qualifying.
        use crate::world_sim::npcs::{can_start_invasion, start_invasion};
        let mut w = tr_core::World::new("t".into(), 64, 64);
        let mut rng = tr_core::UnifiedRandom::new(1);
        assert!(!can_start_invasion(&w, 100), "the base life waits");
        assert!(can_start_invasion(&w, 200));
        assert!(start_invasion(&mut w, 1, 1, &mut rng));
        assert!(w.invasion_goblins);
        assert_eq!(w.invasion_size, 120, "80 + 40 x one qualifying player");
        assert!(!start_invasion(&mut w, 1, 1, &mut rng));
        assert_eq!(w.invasion_size, 120, "the active invasion blocks");
    }

    #[test]
    fn the_battle_standard_starts_the_invasion_and_consumes() {
        // `ItemCheck_UseEventItems` (`Player.cs:43920-43940`): the use
        // starts the invasion + pays the stack; the refusal keeps it.
        let mut app = flow_app(None, vec![], vec![]);
        let mut player = crate::player::Player::spawn(&app.world);
        player.max_hp = 200;
        player.hp = player.max_hp;
        app.player = Some(player);
        app.inventory[app.selected] = ItemSlot {
            kind: 361,
            name: "Goblin Battle Standard".into(),
            count: 2,
            prefix: 0,
        };
        assert!(use_goblin_battle_standard(&mut app));
        assert!(app.world.invasion_goblins);
        assert_eq!(app.world.invasion_size, 120);
        assert_eq!(app.inventory[app.selected].count, 1, "the use paid one");
        assert!(
            !use_goblin_battle_standard(&mut app),
            "the active invasion refuses"
        );
        assert_eq!(app.inventory[app.selected].count, 1, "the refusal is free");
    }

    #[test]
    fn the_suspicious_eye_summons_only_at_night_and_once() {
        // `ItemCheck_UseBossSpawners`' gates: day → refused; night → the
        // roar + the spawn + the consumption; alive → refused
        // (`SummonItemCheck`).
        let mut app = flow_app(None, vec![], vec![]);
        let mut player = crate::player::Player::spawn(&app.world);
        player.x = 64.0;
        player.y = 64.0;
        app.player = Some(player);
        app.world.day_time = true;
        app.inventory[app.selected] = ItemSlot {
            kind: 43,
            name: "Suspicious Looking Eye".into(),
            count: 1,
            prefix: 0,
        };
        assert!(!use_suspicious_looking_eye(&mut app), "day refuses");
        app.world.day_time = false;
        assert!(use_suspicious_looking_eye(&mut app));
        assert_eq!(app.inventory[app.selected].count, 0, "the use consumed");
        assert!(
            app.npcs
                .iter()
                .any(|n| n.kind == NpcKind::EyeOfCthulhu && n.hp > 0)
        );
        app.inventory[app.selected] = ItemSlot {
            kind: 43,
            name: "Suspicious Looking Eye".into(),
            count: 1,
            prefix: 0,
        };
        assert!(
            !use_suspicious_looking_eye(&mut app),
            "SummonItemCheck refuses while alive"
        );
        assert_eq!(app.inventory[app.selected].count, 1);
    }

    #[test]
    fn the_eoc_kill_sets_the_downed_flag() {
        // `NPC.downedBoss1 = true` rides the death pass; the loot resolves
        // against the full database (the executor skips the unregistered
        // items).
        let mut app = flow_app(None, vec![], vec![]);
        let mut player = crate::player::Player::spawn(&app.world);
        player.x = 64.0;
        player.y = 64.0;
        app.player = Some(player);
        app.world.day_time = false;
        let mut eye = crate::npc::Npc::spawn_eye_of_cthulhu(&app.world, 64.0, 64.0).unwrap();
        eye.hp = 0;
        app.npcs.push(eye);
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        assert!(app.world.downed_eoc);
        assert!(!app.npcs.iter().any(|n| n.kind == NpcKind::EyeOfCthulhu));
    }
}

/// `ItemCheck_UseBossSpawners`' Pirate Invasion arm (`Player.cs:43957-43984`):
/// the Pirate Map (1315) — the `CanStartInvasion(3, ignoreDelay: true)` gate,
/// the roar (`SoundID.Roar` 15), then `invasionDelay = 0; StartInvasion(3)`,
/// whose body sizes the pirates (`80+40n + 40+20n`) and rolls the side.
pub fn use_pirate_map(app: &mut App) -> bool {
    // `Player.cs:43957-43984`: `CanStartInvasion(3, ignoreDelay: true)` then
    // `invasionDelay = 0` + `StartInvasion(3)` — the shared body sizes the
    // army (`80+40n + 40+20n`) and rolls the side on `UpdatePlayers`.
    let life_max = app.player.as_ref().map(|p| p.max_hp).unwrap_or(0);
    if !crate::world_sim::npcs::can_start_invasion(&app.world, life_max) {
        return false;
    }
    app.audio.play("Roar_0");
    app.world.invasion_delay = 0;
    crate::world_sim::npcs::start_invasion_in(
        app,
        1,
        3,
        Some(crate::app::RngKey::UpdatePlayers),
    );
    app.inventory[app.selected].count = app.inventory[app.selected].count.saturating_sub(1);
    if app.inventory[app.selected].count == 0 {
        app.inventory[app.selected].kind = 0;
    }
    // `ApplyItemTime(sItem)` (`Player.cs:43959`): the action cadence re-arms.
    app.item_time = crate::item::use_time_secs(1315);
    app.mine_target = None;
    app.mine_timer = 0.0;
    true
}
/// `Player.cs:32425-32444`'s Etherian arm: the Etherian Crystal (3828)
/// placed over the Eternia Crystal Stand (466) starts the Old One's Army.
/// The gates verbatim: no ongoing event, no crystal alive, no pumpkin/snow
/// moon, the 3828 in the inventory, the stand's frame (`frameX/18*16`,
/// `frameY/18*16` — the 3x3 stand anchor) gives the spawn point (+40, +64),
/// then `ConsumeItem` and `DD2Event.SummonCrystal` (`StartInvasion` + the
/// crystal NPC at the point + the starter-crystal drops).
///
/// Deviations: `DropStarterCrystals`' per-player drops ride the item-drop pass;
/// `WouldFailSpawningHere` (`DD2Event.cs:603-613`) landed with B-6c (S1-E-12d) —
/// the 60-tile each-side arena check + the bartender warning, the walker in
/// `tr_core::stray`.
pub fn use_etherian_crystal(app: &mut App) -> bool {
    if app.world.invasion_type != 0 || app.world.snow_moon || app.world.pumpkin_moon {
        return false;
    }
    let crystal_alive = app
        .npcs
        .iter()
        .any(|n| n.hp > 0 && n.kind == NpcKind::DD2EterniaCrystal);
    if crystal_alive {
        return false;
    }
    // The stand tile under the cursor.
    let m = app.mouse_pos;
    let (sw, sh) = app.screen_size();
    let screen = Vec2::new(sw, sh);
    let world_pos = app.camera + (m - screen * 0.5) / app.zoom;
    let tx = (world_pos.x / TILE_PX).floor() as i32;
    let ty = (world_pos.y / TILE_PX).floor() as i32;
    let tile = app.world.tile(tx, ty);
    let is_stand = tile.is_some_and(|t| t.active() && t.kind == 466);
    if !is_stand {
        return false;
    }
    let (fx, fy) = tile
        .map(|t| (t.frame_x as i32, t.frame_y as i32))
        .unwrap_or((0, 0));
    let (sx, sy) = (tx * 16 - fx / 18 * 16 + 40, ty * 16 - fy / 18 * 16 + 64);
    // B-6c (S1-E-12d): `WouldFailSpawningHere(myX, num28)`
    // (`Player.cs:32432`, `DD2Event.cs:603-613`): the arena walker must
    // find >= 60 walkable tiles on EACH side of the clicked stand column
    // (`CheckArenaScore(Point(x,y).ToWorldCoordinates())` — the default
    // (8,8) centre, `Utils.cs:1762-1765`), else the use fails with the
    // bartender warning (`FailureMessage`, `:614-627` — yellow 255,255,0,
    // `DungeonDefenders2.BartenderWarning`) and NOTHING consumes. The
    // C#'s `num28` is the stand's bottom row (frameY 0/18/36 → +3/+2/+1,
    // `Player.cs:32404-32417`); the port's single-tile stand fixture
    // carries frameY 0, so +3. The walker draws no RNG.
    let stand_y = ty + if fy == 0 { 3 } else if fy == 18 { 2 } else { 1 };
    let (left_end, right_end) = tr_core::stray::check_arena_score(
        &app.world,
        (tx as f32 * 16.0 + 8.0, stand_y as f32 * 16.0 + 8.0),
    );
    if right_end.0 - tx < 60 || tx - left_end.0 < 60 {
        app.chat.new_text(
            crate::lang::text("DungeonDefenders2.BartenderWarning"),
            [255, 255, 0],
        );
        return false;
    }
    // The consume.
    app.inventory[app.selected].count = app.inventory[app.selected].count.saturating_sub(1);
    if app.inventory[app.selected].count == 0 {
        app.inventory[app.selected].kind = 0;
    }
    // `StartInvasion` (`DD2Event.cs:253-291`): the counters and the boss
    // latches start clean (`:268-275`), `LostThisRun`/`WonThisRun` clear
    // (`:271-272`), the 300-tick spawn hold arms (`:288`) and
    // `WipeEntities` runs (`:289`) — all BEFORE the crystal's `NewNPC`
    // (`SummonCrystalDirect`, `:596-597`), so the wipe cannot eat it.
    app.world.dd2_event_ongoing = true;
    app.world.dd2_wave = 1;
    app.world.dd2_wave_kills = 0.0;
    app.world.dd2_lost = false;
    // `:272`: `WonThisRun = false`; `:257-259`: the crystal-drop budget
    // zeroes; `:270`: `_spawnedBetsyT3 = false` (S1-E-12c).
    app.world.dd2_won = false;
    app.world.dd2_crystal_drops = tr_core::dd2_victory::CrystalDropState::default();
    app.world.dd2_betsy_spawned = false;
    app.world.dd2_latches = tr_core::dd2::DownedLatches::default();
    app.world.dd2_spawn_hold = tr_core::dd2::SPAWN_HOLD_START;
    crate::world_sim::npcs::wipe_dd2_entities(app);
    // The crystal + the event start.
    let mut crystal = Npc::new(
        NpcKind::DD2EterniaCrystal,
        sx as f32 - 20.0,
        sy as f32 - 20.0,
        1,
        0.0,
    );
    crystal.hp = crystal.max_hp;
    app.npcs.push(crystal);
    // The C# arm (`Player.cs:32438-32445`) is a tile interaction: consume +
    // `SummonCrystal`, NO `ApplyItemTime` — the swing timers stay untouched.
    true
}

/// The full use-loop integration — a Slime Staff in hand fires through
/// `update_editing`: the buff grant, the free-up, the spawn, then the pool's
/// next tick books the slot, and the player tick's arm keeps the buff alive.
#[cfg(test)]
mod use_loop_tests {
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::world_sim::editing::limit_point_to_player_reachable_area;
    use glam::Vec2;

    fn app_with_staff(kind: u16) -> crate::app::App {
        // A real staged world (the 8×8 `flow_app` default is too small — the
        // reach gate and the `LimitPointToPlayerReachableArea` clamp misbehave
        // on it): 64 wide, a solid floor at row 300, the player + camera near
        // column 30. The cursor sits at the screen centre the tests pass
        // (800×600 → mouse 400,300), so the use-path cursor == the camera and
        // the reach clamp is the identity (the cursor is beside the player).
        let mut w = tr_core::World::new("t".into(), 64, 800);
        for x in 0..64 {
            let i = w.index(x, 300);
            w.tiles[i] = tr_core::tile::Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
        w.compute_column_surface();
        let mut app = flow_app(None, vec![], vec![]);
        app.world = w;
        // Single-player: the local player's id is 0 (the recount's owner).
        app.local_id = 0;
        let mut player = crate::player::Player::spawn(&app.world);
        player.x = 33.0 * 16.0;
        player.y = 297.0 * 16.0;
        player.max_minions = 5;
        player.max_turrets = 5;
        app.player = Some(player);
        app.camera = Vec2::new(30.0 * 16.0 + 8.0, 297.0 * 16.0 + 8.0);
        // The cursor at the (800×600) screen centre the tests pass.
        app.mouse_pos = Vec2::new(400.0, 300.0);
        app.inventory[app.selected] = crate::item::ItemSlot {
            kind,
            name: crate::registry::name(kind),
            count: 1,
            prefix: 0,
        };
        // Mute the listener so the use-start sound draws nothing on
        // `UpdatePlayers` (the RNG witnesses isolate the crit/kind-pick draws;
        // `play_sound` returns None before the pitch draw at volume 0).
        app.audio.set_listener(crate::sound::Listener {
            camera_center: (app.camera.x, app.camera.y),
            sound_volume: 0.0,
            ambient_volume: 0.0,
            quiet_ambient_sounds: false,
            game_menu: false,
            generating_or_loading: false,
            server: false,
        });
        app
    }

    #[test]
    fn the_slime_staff_use_grants_the_buff_and_spawns() {
        let mut app = app_with_staff(1309); // the Slime Staff — shoot 266, buff 64
        app.mine_down = true;
        app.mine_prev_down = false;
        // One editing tick: the use fires (the swing starts, the free-up runs
        // and the projectile spawns), and the verbatim `item.buffType > 0`
        // grant lands (`Player.cs:42879`).
        crate::world_sim::editing::update_editing(&mut app, 1.0 / 60.0, Vec2::new(800.0, 600.0));
        assert_eq!(
            app.projectiles
                .slots()
                .iter()
                .filter(|s| s.active && s.kind == 266)
                .count(),
            1,
            "the Slime minion spawned"
        );
        // The staff's row carries `buffTime = 0` and the C#'s arm guard skips
        // 0-time slots, so the grant alone is a ghost the sweep removes
        // (verbatim, ANALYSIS); the keep-alive loop is asserted with a live
        // buff: the arm refreshes it while the minion lives.
        {
            let p = app.player.as_mut().unwrap();
            p.buffs.add_buff(64, 600);
        }
        crate::world_sim::player::update_player(&mut app, 1.0 / 60.0);
        let p = app.player.as_ref().unwrap();
        let b = p.buffs.find_buff_index(64).expect("buff 64 held");
        assert_eq!(p.buffs.buff_time[b], 18000, "the arm's refresh");
        assert!(p.slime, "the arm's flag");
        // The pool's next tick books the minion against the budget.
        app.mine_down = false;
        let mut npcs = Vec::new();
        let mut rng = tr_core::tile_light::FastRandom::new(7);
        let local = app.local_id as u8;
        let host = !app.is_client();
        app.projectiles.update(
            &mut app.world,
            &mut npcs,
            None,
            None,
            app.player.as_mut(),
            crate::projectile::YoyoInputs::default(),
            crate::projectile::FlailInputs::default(),
            &mut rng,
            local,
            host,
        );
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.num_minions, 1, "the slot booked");
        assert_eq!(p.slots_minions, 1.0);
    }

    /// S1-P-i25 — the summon spawn arm books the UNSCALED item damage as
    /// `originalDamage` (`Player.cs:49601`), not the class-multiplied shot: the
    /// pool's `summon_tick_block` applies `minionDamage` later (`:16707`). The
    /// velocity is ZERO (`SpawnMinionOnCursor`, `:49600`) — the minion is born
    /// at the cursor, not flying from the player.
    #[test]
    fn i25_the_minion_spawns_at_the_cursor_with_unscaled_damage() {
        let mut app = app_with_staff(2364); // Hornet Staff — shoot 373, damage 12
        app.player.as_mut().unwrap().minion_damage = 3.0; // a big multiplier
        app.mine_down = true;
        app.mine_prev_down = false;
        crate::world_sim::editing::update_editing(&mut app, 1.0 / 60.0, Vec2::new(800.0, 600.0));
        let slot = app
            .projectiles
            .slots()
            .iter()
            .position(|s| s.active && s.kind == 373)
            .expect("the Hornet spawned");
        let p = &app.projectiles.slots()[slot];
        assert_eq!(p.original_damage, 12, "the UNSCALED item damage, not 12*3");
        assert_eq!(p.damage, 12, "the spawn damage is unscaled too");
        assert_eq!((p.vel.x, p.vel.y), (0.0, 0.0), "zero spawn velocity");
        assert_eq!(p.ai[1], 1.0, "the Hornet Staff's ai1 seed (`:49587-49593`)");
        assert!(p.minion, "the minion identification row");
        assert_eq!(p.minion_slots, 1.0, "a full slot");
    }

    /// S1-P-i25 — the Spider Staff cycles 390/391/392 across uses via
    /// `nextCycledSpiderMinionType` (`Player.cs:48478-48480`), never reset.
    #[test]
    fn i25_the_spider_staff_cycles_its_three_kinds() {
        let mut app = app_with_staff(2551); // Spider Staff — shoot 390
        let mut seen = Vec::new();
        for _ in 0..4 {
            app.mine_down = true;
            app.mine_prev_down = false;
            crate::world_sim::editing::update_editing(&mut app, 1.0 / 60.0, Vec2::new(800.0, 600.0));
            // Record the newest spider kind, then clear the pool + swing so the
            // next press is a fresh use (the autoReuse repeat needs a release).
            let kind = app
                .projectiles
                .slots()
                .iter()
                .filter(|s| s.active && matches!(s.kind, 390 | 391 | 392))
                .map(|s| s.kind)
                .last()
                .expect("a spider spawned");
            seen.push(kind);
            for s in app.projectiles.slots_mut() {
                s.active = false;
            }
            app.swing = 0.0;
            app.attack_cd = 0.0;
            app.reuse_delay = 0.0;
            app.mine_down = false;
        }
        assert_eq!(seen, vec![390, 391, 392, 390], "the mod-3 cycle");
        assert_eq!(app.player.as_ref().unwrap().next_cycled_spider_minion_type, 1);
    }

    /// S1-P-i25 — the Optic Staff spawns BOTH twins (387 + 388) in one use
    /// (`Player.cs:48468-48474`), each a half slot, ai1 = 1.
    #[test]
    fn i25_the_optic_staff_spawns_both_twins() {
        let mut app = app_with_staff(2535); // Optic Staff — shoot 387
        app.mine_down = true;
        app.mine_prev_down = false;
        crate::world_sim::editing::update_editing(&mut app, 1.0 / 60.0, Vec2::new(800.0, 600.0));
        let kinds: Vec<u16> = app
            .projectiles
            .slots()
            .iter()
            .filter(|s| s.active && s.minion)
            .map(|s| s.kind)
            .collect();
        assert_eq!(kinds, vec![387, 388], "both twins, slot order");
        for s in app.projectiles.slots().iter().filter(|s| s.active && s.minion) {
            assert_eq!(s.minion_slots, 0.5, "the twins' half slot");
            assert_eq!(s.ai[1], 1.0, "the Optic ai1 seed");
            assert_eq!((s.vel.x, s.vel.y), (0.0, 0.0), "the RotatedBy-zero velocity");
        }
    }

    /// S1-P-i25 — the Pygmy Staff picks one of 191-194 via `Main.rand.Next(191,
    /// 195)` (`Player.cs:48438`), seeds ai1 = 60 (`:49584-49585`) and
    /// `localAI[0] = 30` (`:48439`). The pick rides the UpdatePlayers stream.
    #[test]
    fn i25_the_pygmy_staff_picks_a_kind_and_seeds_the_ai() {
        let mut app = app_with_staff(1157); // Pygmy Staff — shoot 191
        app.mine_down = true;
        app.mine_prev_down = false;
        crate::world_sim::editing::update_editing(&mut app, 1.0 / 60.0, Vec2::new(800.0, 600.0));
        let slot = app
            .projectiles
            .slots()
            .iter()
            .position(|s| s.active && s.minion)
            .expect("a pygmy spawned");
        let p = &app.projectiles.slots()[slot];
        assert!(matches!(p.kind, 191 | 192 | 193 | 194), "the Next(191,195) pick");
        assert_eq!(p.ai[1], 60.0, "the Pygmy ai1 seed");
        assert_eq!(p.local_ai[0], 30.0, "the localAI[0] = 30 write");
        assert_eq!(p.minion_slots, 1.0, "a full slot");
    }

    /// S1-P-i25 — the use-start `ItemCheck_ApplyPetBuffs` grant is 3600 ticks
    /// (`Player.cs:53190-53281`), the minion-buff PERSISTENCE mechanism: the
    /// keep-alive arm's 18000 refresh pins it while the minion lives. The
    /// `:42880` `item.buffTime` grant is 0 for the staves (a ghost).
    #[test]
    fn i25_the_minion_buff_grant_is_3600_and_persists() {
        let mut app = app_with_staff(2364); // Hornet Staff — buff 125
        app.mine_down = true;
        app.mine_prev_down = false;
        crate::world_sim::editing::update_editing(&mut app, 1.0 / 60.0, Vec2::new(800.0, 600.0));
        let p = app.player.as_ref().unwrap();
        let b = p.buffs.find_buff_index(125).expect("buff 125 granted");
        assert_eq!(p.buffs.buff_time[b], 3600, "the ApplyPetBuffs 3600 grant");
        // The keep-alive arm refreshes to 18000 while the Hornet lives.
        crate::world_sim::player::update_player(&mut app, 1.0 / 60.0);
        let p = app.player.as_ref().unwrap();
        let b = p.buffs.find_buff_index(125).expect("buff 125 kept");
        assert_eq!(p.buffs.buff_time[b], 18000, "the keep-alive refresh");
        assert!(p.hornet_minion, "the arm's flag");
    }

    /// S1-P-i25 round 2 — the cursor minion spawn subtracts the half-box:
    /// `NewProjectile` takes the CENTRE (`Projectile.cs:10876-10877`), the
    /// port's seam the top-left, so the Hornet's 24x26 box centres on the
    /// CLAMPED cursor. The fixture's 64-wide world is narrower than the
    /// reachable rect's 640-px insets, so `LimitPointToPlayerReachableArea`
    /// (`Player.cs:45447-45473`) pulls the rect's max BELOW its min and the
    /// max-first `Utils.Clamp` returns the max: rx = 1024−640−1920 = −1536,
    /// mid.x = −1536+960 = −576, and the off.x = 488−(−576) = 1064 > 960
    /// half-width scales to 960/1064 — the clamped cursor X is 384 (col 24),
    /// Y stays 4760 (the tall world's clamp is the identity). The module is
    /// C#-true on the small world; the witness applies the same clamp.
    #[test]
    fn i25_the_minion_cursor_spawn_is_centred_on_the_cursor() {
        let mut app = app_with_staff(2364); // Hornet Staff — shoot 373 (24x26)
        app.mine_down = true;
        app.mine_prev_down = false;
        crate::world_sim::editing::update_editing(&mut app, 1.0 / 60.0, Vec2::new(800.0, 600.0));
        // The cursor uses the SAME screen the use path was given (800×600).
        let (sw, sh) = (800.0f32, 600.0f32);
        let raw = app.camera + (app.mouse_pos - Vec2::new(sw * 0.5, sh * 0.5)) / app.zoom;
        let (pcx, pcy) = app.player.as_ref().unwrap().center();
        let cursor = limit_point_to_player_reachable_area(Vec2::new(pcx, pcy), raw, &app.world);
        let slot = app
            .projectiles
            .slots()
            .iter()
            .position(|s| s.active && s.kind == 373)
            .expect("the Hornet spawned");
        let p = &app.projectiles.slots()[slot];
        assert_eq!(p.pos.x, cursor.x - 12.0, "centre X minus width/2");
        assert_eq!(p.pos.y, cursor.y - 13.0, "centre Y minus height/2");
    }

    /// S1-P-i25 round 2 — the Optic twins centre on the CLAMPED cursor too
    /// (each 40x20, `Projectile.cs`'s RETANIMINI/SPAZMAMINI slice); the same
    /// small-world reach clamp as the Hornet witness above.
    #[test]
    fn i25_the_optic_twins_centre_on_the_cursor() {
        let mut app = app_with_staff(2535); // Optic Staff — shoot 387
        app.mine_down = true;
        app.mine_prev_down = false;
        crate::world_sim::editing::update_editing(&mut app, 1.0 / 60.0, Vec2::new(800.0, 600.0));
        // The cursor uses the SAME screen the use path was given (800×600).
        let (sw, sh) = (800.0f32, 600.0f32);
        let raw = app.camera + (app.mouse_pos - Vec2::new(sw * 0.5, sh * 0.5)) / app.zoom;
        let (pcx, pcy) = app.player.as_ref().unwrap().center();
        let cursor = limit_point_to_player_reachable_area(Vec2::new(pcx, pcy), raw, &app.world);
        for s in app.projectiles.slots().iter().filter(|s| s.active && s.minion) {
            assert_eq!(s.pos.x, cursor.x - 20.0, "the twin's centre X minus 20");
            assert_eq!(s.pos.y, cursor.y - 10.0, "the twin's centre Y minus 10");
        }
    }

    /// S1-P-i25 round 2 — the Clay Pot (6161) has NO `SpawnMinionOnCursor` arm:
    /// it rides the generic tail (`Player.cs:49437`) — player-centre spawn, aim
    /// velocity at `shootSpeed`, SCALED damage — then overrides
    /// `originalDamage = damage` (UNSCALED, `:49442-49444`).
    #[test]
    fn i25_the_clay_pot_rides_the_generic_tail_with_the_unscaled_override() {
        let mut app = app_with_staff(6161); // Clay Pot — shoot 1118, damage 5, speed 10
        app.player.as_mut().unwrap().minion_damage = 3.0;
        app.mine_down = true;
        app.mine_prev_down = false;
        crate::world_sim::editing::update_editing(&mut app, 1.0 / 60.0, Vec2::new(800.0, 600.0));
        let slot = app
            .projectiles
            .slots()
            .iter()
            .position(|s| s.active && s.kind == 1118)
            .expect("the Clay Pot minion spawned");
        let p = &app.projectiles.slots()[slot];
        assert_eq!(p.damage, 15, "the SCALED spawn damage (5 * 3)");
        assert_eq!(p.original_damage, 5, "the UNSCALED override (`:49444`)");
        // The generic aim velocity — NOT the cursor arm's zero. The cursor sits
        // at the player centre, so the aim folds to the facing (+x).
        assert!(p.vel.length() > 0.0, "the generic aim velocity, not zero");
        assert!((p.vel.length() - 10.0).abs() < 1e-3, "shootSpeed 10");
    }

    /// S1-P-i25 round 2 — the summon shot spends NO crit draw: the C# rolls
    /// crit at HIT only for melee/ranged/magic (`Projectile.cs:12890-12901`),
    /// never at spawn for a summon. The plain minion staff's whole use leaves
    /// the `UpdatePlayers` stream untouched.
    #[test]
    fn i25_the_summon_shot_spends_no_crit_draw() {
        let mut app = app_with_staff(2364); // Hornet Staff — no kind pick
        let seed = 0x51A7;
        *app.rngs.stream(crate::app::RngKey::UpdatePlayers) = tr_core::UnifiedRandom::new(seed);
        app.mine_down = true;
        app.mine_prev_down = false;
        crate::world_sim::editing::update_editing(&mut app, 1.0 / 60.0, Vec2::new(800.0, 600.0));
        assert!(
            app.projectiles.slots().iter().any(|s| s.active && s.kind == 373),
            "the Hornet spawned"
        );
        let mut want = tr_core::UnifiedRandom::new(seed);
        // Zero draws: the stream is byte-equal to its fresh state.
        assert_eq!(
            app.rngs.stream(crate::app::RngKey::UpdatePlayers).clone(),
            want,
            "the summon shot rolls no crit (no draw)"
        );
        // The Pygmy's kind pick is the ONE draw the summon arm spends.
        let mut pygmy = app_with_staff(1157);
        *pygmy.rngs.stream(crate::app::RngKey::UpdatePlayers) = tr_core::UnifiedRandom::new(seed);
        pygmy.mine_down = true;
        pygmy.mine_prev_down = false;
        crate::world_sim::editing::update_editing(&mut pygmy, 1.0 / 60.0, Vec2::new(800.0, 600.0));
        want.next_range(191, 195);
        assert_eq!(
            pygmy.rngs.stream(crate::app::RngKey::UpdatePlayers).clone(),
            want,
            "the Pygmy spends exactly the kind pick, no crit"
        );
    }

    /// S1-P-i25 round 2 — the summon staff's knockback takes `GetWeaponKnockback`'s
    /// `minionKB` add (`Player.cs:47151`/`:53348-53351`): the Hornet's registry base
    /// plus the accessory's `minionKB`.
    #[test]
    fn i25_the_minion_knockback_takes_the_minion_kb_add() {
        let mut app = app_with_staff(2364); // Hornet Staff
        app.player.as_mut().unwrap().minion_kb = 2.0;
        app.mine_down = true;
        app.mine_prev_down = false;
        crate::world_sim::editing::update_editing(&mut app, 1.0 / 60.0, Vec2::new(800.0, 600.0));
        let p = app
            .projectiles
            .slots()
            .iter()
            .find(|s| s.active && s.kind == 373)
            .expect("the Hornet spawned");
        let base = crate::registry::get(2364).unwrap().knock_back;
        assert_eq!(p.knock_back, base + 2.0, "the minionKB add rides the spawn");
    }

    /// A staged world for the sentry landing scan: 64 wide, a solid floor row
    /// at `floor_y`, the player + camera + cursor near column 30 (the reach
    /// gate passes). The cursor's world point is returned for the assertions.
    fn sentry_app(item: u16, floor_y: i32, cursor_dy: f32) -> (crate::app::App, Vec2) {
        let mut w = tr_core::World::new("t".into(), 64, 800);
        for x in 0..64 {
            let i = w.index(x, floor_y);
            w.tiles[i] = tr_core::tile::Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
        w.compute_column_surface();
        let mut app = flow_app(None, vec![], vec![]);
        app.world = w;
        app.local_id = 0;
        let mut player = crate::player::Player::spawn(&app.world);
        player.x = 33.0 * 16.0;
        player.y = (floor_y - 3) as f32 * 16.0;
        player.max_turrets = 5;
        player.max_turrets_old = 5;
        app.player = Some(player);
        app.camera = Vec2::new(30.0 * 16.0 + 8.0, (floor_y - 3) as f32 * 16.0 + 8.0);
        let (sw, sh) = app.screen_size();
        app.mouse_pos = Vec2::new(sw * 0.5, sh * 0.5 + cursor_dy);
        app.inventory[app.selected] = crate::item::ItemSlot {
            kind: item,
            name: crate::registry::name(item),
            count: 1,
            prefix: 0,
        };
        let cursor = app.camera + (app.mouse_pos - Vec2::new(sw * 0.5, sh * 0.5)) / app.zoom;
        (app, cursor)
    }

    /// One fresh press of the held sentry staff. The press edge arms the
    /// prior-frame release (`mine_released = true`, `mine_prev_down = false`)
    /// — the sentry staves are NOT autoReuse, so the fire gate reads
    /// `controlUseItem && releaseUseItem` (`Player.cs:42733`); the
    /// `sentry_use_tests` fixture's latch, transplanted.
    fn sentry_fire(app: &mut crate::app::App) {
        app.mine_down = true;
        app.mine_prev_down = false;
        app.mine_released = true;
        let (sw, sh) = app.screen_size();
        crate::world_sim::editing::update_editing(app, 1.0 / 60.0, Vec2::new(sw, sh));
    }

    /// S1-P-i25 round 2 — the Frost Hydra (1572 → 308) lands one row above the
    /// first solid row of the cursor column (`PickSentryLandingSpot`,
    /// `Player.cs:49525-49543`), drops at `vy = 15`, seeds `ai0 = 60`
    /// (`:48197-48198`), carries the SCALED damage + the UNSCALED
    /// `originalDamage` (`:48208-48209`), and centres its 80x74 box.
    #[test]
    fn i25_the_frost_hydra_lands_above_the_floor_with_the_ai0_seed() {
        let (mut app, cursor) = sentry_app(1572, 300, -48.0); // cursor row 294 (inside the 6-tile reach), floor 300
        app.player.as_mut().unwrap().minion_damage = 2.0;
        sentry_fire(&mut app);
        let slot = app
            .projectiles
            .slots()
            .iter()
            .position(|s| s.active && s.kind == 308)
            .expect("the Frost Hydra spawned");
        let p = &app.projectiles.slots()[slot];
        // The scan stops at the floor row 300, steps back to 299: spawnY =
        // 299*16 - 24 = 4760; the half-box (74/2 = 37) gives top-left 4723.
        assert_eq!(p.pos.y, 299.0 * 16.0 - 24.0 - 37.0, "one row above the floor, centred");
        assert_eq!(p.pos.x, cursor.x - 40.0, "the cursor X, centred (80/2)");
        assert_eq!((p.vel.x, p.vel.y), (0.0, 15.0), "the vy = 15 drop");
        assert_eq!(p.ai[0], 60.0, "the Frost Hydra ai0 seed (`:48198`)");
        assert_eq!(p.damage, 200, "the SCALED damage (100 * 2)");
        assert_eq!(p.original_damage, 100, "the UNSCALED originalDamage (`:48209`)");
        assert!(p.sentry, "the sentry identification");
    }

    /// S1-P-i25 round 2 — the Houndius Shootius (5119 → 966) seeds `ai0 = 90`
    /// (`:48200-48201`) and lands above the floor like the Hydra.
    #[test]
    fn i25_the_houndius_seeds_ai0_90() {
        let (mut app, _cursor) = sentry_app(5119, 300, -48.0);
        sentry_fire(&mut app);
        let p = app
            .projectiles
            .slots()
            .iter()
            .find(|s| s.active && s.kind == 966)
            .expect("the Houndius spawned");
        assert_eq!(p.ai[0], 90.0, "the Houndius ai0 seed (`:48201`)");
        assert_eq!(p.vel.y, 15.0, "the vy = 15 drop");
        assert_eq!(p.pos.y, 299.0 * 16.0 - 24.0 - 30.0, "above the floor, centred (60/2)");
    }

    /// S1-P-i25 round 2 — the Rainbow Crystal (3571 → 643) SKIPS the landing
    /// scan (`:49527`'s `num`): it lands at the cursor ROW even with solid
    /// floor below, `ai0 = 0`, `vy = 15`.
    #[test]
    fn i25_the_rainbow_crystal_skips_the_scan_to_the_cursor_row() {
        let (mut app, cursor) = sentry_app(3571, 300, -48.0); // cursor row 294 (inside the reach)
        sentry_fire(&mut app);
        let p = app
            .projectiles
            .slots()
            .iter()
            .find(|s| s.active && s.kind == 643)
            .expect("the Rainbow Crystal spawned");
        let cursor_row = (cursor.y / 16.0) as i32;
        assert_eq!(
            p.pos.y,
            cursor_row as f32 * 16.0 - 24.0 - 16.0,
            "the cursor row (no scan), centred (32/2)"
        );
        assert_eq!(p.ai[0], 0.0, "no ai0 seed for the crystal");
        assert_eq!(p.vel.y, 15.0, "the vy = 15 drop");
    }

    /// S1-P-i25 round 2 — the Dead Cells Barnacle (5463 → 1025) lands at the
    /// cursor's exact world Y with `vy = 0` (`:48203-48206`), `ai0 = 0`.
    #[test]
    fn i25_the_barnacle_lands_at_the_cursor_row_with_zero_velocity() {
        let (mut app, cursor) = sentry_app(5463, 300, -48.0);
        sentry_fire(&mut app);
        let p = app
            .projectiles
            .slots()
            .iter()
            .find(|s| s.active && s.kind == 1025)
            .expect("the Barnacle spawned");
        assert_eq!(p.pos.y, cursor.y - 19.0, "the cursor Y, centred (38/2)");
        assert_eq!(p.pos.x, cursor.x - 21.0, "the cursor X, centred (42/2)");
        assert_eq!((p.vel.x, p.vel.y), (0.0, 0.0), "the vy = 0 barnacle");
        assert_eq!(p.ai[0], 0.0, "no ai0 seed for the barnacle");
    }

    /// S1-P-i25 round 2 — the sentry spawn runs `UpdateMaxTurrets` right after
    /// (`Player.cs:48210`): over `max_turrets` the lowest-`timeLeft` wipable
    /// sentry dies. Two fresh Hydras at the cap of 1 leave only the newest.
    #[test]
    fn i25_the_sentry_spawn_runs_the_turret_cap_sweep() {
        let (mut app, _cursor) = sentry_app(1572, 300, -48.0);
        app.player.as_mut().unwrap().max_turrets = 1;
        // An older Hydra with a lower timeLeft — the sweep's victim.
        let old = app
            .projectiles
            .spawn_with_data_ai_owned(308, 0, 100.0, 100.0, 0.0, 0.0, 100, 7.5, 60.0, 0.0)
            .unwrap();
        app.projectiles.slots_mut()[old].time_left = 100;
        sentry_fire(&mut app);
        assert!(!app.projectiles.slots()[old].active, "the oldest (lowest timeLeft) evicted");
        let live = app
            .projectiles
            .slots()
            .iter()
            .filter(|s| s.active && s.kind == 308)
            .count();
        assert_eq!(live, 1, "the cap of 1 holds after the spawn");
    }
}

/// S1-P-i26 — the sentry-summon USE path's integration witnesses: the two
/// `ItemCheck_Shoot` sentry arms (`Player.cs:48189-48212` / `:49057-49094`)
/// fire through `update_editing` at the cursor/tile spot with the C# payload,
/// and the `UpdatePlayers` stream's draw count stays 1:1 (the sentry arms
/// make no crit draw). WRITTEN NOT RUN (code-only mode).
#[cfg(test)]
mod sentry_use_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

    /// A 64×800 world with a stone floor at row 300, the player standing just
    /// above it at column 30, the cursor on the player (inside the reach).
    /// The listener is muted so the use-sound's pitch draw never fires — the
    /// `UpdatePlayers` stream then carries only the crit roll for a non-sentry
    /// summon and nothing for a sentry (the RNG witness's mirrored-stream
    /// baseline; the use-start sound draws the same on both either way).
    fn sentry_app(kind: u16) -> App {
        let mut w = World::new("t".into(), 64, 800);
        for x in 0..64 {
            let i = w.index(x, 300);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        w.compute_column_surface();
        let mut app = flow_app(None, vec![], vec![]);
        app.world = w;
        app.local_id = 0;
        let mut p = crate::player::Player::spawn(&app.world);
        p.x = 30.0 * TILE_PX;
        p.y = 300.0 * TILE_PX - crate::player::HEIGHT;
        p.facing = 1;
        p.max_turrets = 5;
        app.player = Some(p);
        app.camera = Vec2::new(30.0 * TILE_PX + 8.0, 300.0 * TILE_PX);
        let (sw, sh) = app.screen_size();
        app.mouse_pos = Vec2::new(sw * 0.5, sh * 0.5);
        app.inventory[app.selected] = ItemSlot {
            kind,
            name: crate::registry::name(kind),
            count: 1,
            prefix: 0,
        };
        app.mine_down = true;
        app.mine_prev_down = false;
        // The press edge: the sentry staves are NOT autoReuse, so the fire
        // gate reads `prev_mine_released && !prev_mine_down`
        // (`Player.cs:42733` `controlUseItem && releaseUseItem`). The previous
        // frame the button was released (`mine_released = true`,
        // `mine_prev_down = false`); this frame it is pressed. Without the
        // released latch the fire block never runs for a non-autoReuse staff.
        app.mine_released = true;
        // Mute the listener: `play_sound` returns None before the pitch draw
        // (`sound.rs:82-84`), so the use start draws nothing on UpdatePlayers.
        app.audio.set_listener(crate::sound::Listener {
            camera_center: (app.camera.x, app.camera.y),
            sound_volume: 0.0,
            ambient_volume: 0.0,
            quiet_ambient_sounds: false,
            game_menu: false,
            generating_or_loading: false,
            server: false,
        });
        app
    }

    fn fire(app: &mut App) {
        let (sw, sh) = app.screen_size();
        update_editing(app, 1.0 / 60.0, Vec2::new(sw, sh));
    }

    /// The Frost Hydra staff (1572 → 308) drops the sentry at the landing spot
    /// (`PickSentryLandingSpot`: one row above the floor, `spawnY = 4760`),
    /// velocity `(0, 15)`, `ai0 = 60`, and the UNSCALED item damage rides
    /// `originalDamage` (`:48209`).
    #[test]
    fn the_frost_hydra_staff_drops_the_sentry_at_the_landing_spot() {
        let mut app = sentry_app(1572);
        fire(&mut app);
        let s = app
            .projectiles
            .slots()
            .iter()
            .find(|p| p.active && p.kind == 308)
            .expect("the Frost Hydra spawned");
        // Centre at (cursor X, 4760); the box is 80×74 → top-left (448, 4723).
        assert!((s.pos.x + s.w * 0.5 - 488.0).abs() < 1e-3, "the cursor X centre");
        assert!((s.pos.y + s.h * 0.5 - 4760.0).abs() < 1e-3, "one row above the floor");
        assert_eq!(s.vel, Vec2::new(0.0, 15.0), "the (0, 15) drop");
        assert_eq!(s.ai[0], 60.0, "the 1572 ai0");
        assert_eq!(s.original_damage, 100, "the unscaled item damage");
        assert!(s.sentry, "the sentry flag");
    }

    /// The Houndius Shootius (5119 → 966) carries `ai0 = 90`; the Dead Cells
    /// Barnacle (5463 → 1025) spawns at the raw cursor Y with zero velocity.
    #[test]
    fn the_houndius_and_barnacle_carry_their_spawn_payloads() {
        let mut app = sentry_app(5119);
        fire(&mut app);
        let s = app
            .projectiles
            .slots()
            .iter()
            .find(|p| p.active && p.kind == 966)
            .expect("the Houndius spawned");
        assert_eq!(s.ai[0], 90.0, "the 5119 ai0");
        assert_eq!(s.vel, Vec2::new(0.0, 15.0));

        let mut app = sentry_app(5463);
        fire(&mut app);
        let s = app
            .projectiles
            .slots()
            .iter()
            .find(|p| p.active && p.kind == 1025)
            .expect("the barnacle spawned");
        assert_eq!(s.vel, Vec2::ZERO, "the barnacle's (0, 0)");
        // The barnacle's spawnY is the raw cursor Y (the camera Y = 4800).
        assert!((s.pos.y + s.h * 0.5 - 4800.0).abs() < 1e-3, "the raw cursor Y");
    }

    /// The DD2 Flameburst T1 popper (3818 → 663) pays 10 crystals, spawns at
    /// the resting spot with `ai0 = 1, ai1 = 80`, and the unscaled damage.
    #[test]
    fn the_flameburst_popper_pays_and_seeds_the_reload() {
        let mut app = sentry_app(3818);
        app.world.dd2_downed_t1 = true;
        app.inventory[1] = ItemSlot {
            kind: 3822,
            name: "DD2 Energy Crystal",
            count: 10,
            prefix: 0,
        };
        fire(&mut app);
        let s = app
            .projectiles
            .slots()
            .iter()
            .find(|p| p.active && p.kind == 663)
            .expect("the Flameburst tower spawned");
        assert_eq!(s.ai[0], 1.0, "the 3818 ai0");
        assert_eq!(s.ai[1], 80.0, "the 3818 ai1 reload");
        assert_eq!(s.original_damage, 17, "the unscaled item damage");
        assert_eq!(app.inventory[1].count, 0, "the 10 crystals paid");
        assert_eq!(app.inventory[1].kind, 0, "the drained cell turns to air");
    }

    /// The DD2 Explosive Trap T1 popper (3832 → 691) seeds `localAI[0]` with
    /// the stock 90-tick cooldown (`GetExplosiveTrapCooldown`).
    #[test]
    fn the_explosive_trap_popper_seeds_the_scan_cooldown() {
        let mut app = sentry_app(3832);
        app.world.dd2_downed_t1 = true;
        app.inventory[1] = ItemSlot {
            kind: 3822,
            name: "DD2 Energy Crystal",
            count: 10,
            prefix: 0,
        };
        fire(&mut app);
        let s = app
            .projectiles
            .slots()
            .iter()
            .find(|p| p.active && p.kind == 691)
            .expect("the trap spawned");
        assert_eq!(s.local_ai[0], 90.0, "the stock cooldown");
        assert_eq!(s.original_damage, 24);
    }

    /// The RNG witness: the sentry use draws NO crit on the `UpdatePlayers`
    /// stream — the C# `ItemCheck_Shoot` makes NO spawn-time crit draw for ANY
    /// summon (`Projectile.Damage`'s crit block `Projectile.cs:12890-12901`
    /// gates on the melee/ranged/magic flags no summon row sets; S1-P-i25
    /// removed the port's throw-time roll for the whole `def.summon` family).
    /// The listener is muted so the use-start sound draws nothing either: BOTH
    /// the minion (Slime Staff) and the sentry (Frost Hydra) leave the stream
    /// at its fresh state — the census is 1:1 (zero draws), not a one-draw
    /// delta. The witness pins both streams to the untouched seed.
    #[test]
    fn the_sentry_use_draws_no_crit_on_the_updateplayers_stream() {
        // The control: the Slime Staff (a non-sentry summon) — no crit draw.
        let mut control = sentry_app(1309);
        *control.rngs.stream(crate::app::RngKey::UpdatePlayers) = tr_core::UnifiedRandom::new(0x5E17);
        fire(&mut control);
        let control_after = control
            .rngs
            .stream(crate::app::RngKey::UpdatePlayers)
            .clone();
        // The sentry: the Frost Hydra staff — no crit draw.
        let mut sentry = sentry_app(1572);
        *sentry.rngs.stream(crate::app::RngKey::UpdatePlayers) = tr_core::UnifiedRandom::new(0x5E17);
        fire(&mut sentry);
        let sentry_after = sentry
            .rngs
            .stream(crate::app::RngKey::UpdatePlayers)
            .clone();
        // Both summons skip the crit draw (and the muted sound draws nothing):
        // the two streams are identical AND both equal the untouched seed.
        assert_eq!(control_after, sentry_after, "both summons skip the crit draw");
        assert_eq!(
            sentry_after,
            tr_core::UnifiedRandom::new(0x5E17),
            "the sentry use draws nothing on UpdatePlayers (census 1:1)"
        );
        assert_eq!(
            control_after,
            tr_core::UnifiedRandom::new(0x5E17),
            "the minion use draws nothing on UpdatePlayers either"
        );
    }

    /// The DD2 use gate refuses before any tier is downed and outside the
    /// event (`:52083-52086`): no swing, no spawn, no crystal payment. A
    /// downed tier opens it (the resting spot's tile-above is clear air).
    #[test]
    fn the_dd2_popper_refuses_without_a_downed_tier() {
        let mut app = sentry_app(3818);
        app.inventory[1] = ItemSlot {
            kind: 3822,
            name: "DD2 Energy Crystal",
            count: 10,
            prefix: 0,
        };
        fire(&mut app);
        assert!(
            app.projectiles.slots().iter().all(|p| !p.active),
            "no tower spawns before a tier is downed"
        );
        assert_eq!(app.inventory[1].count, 10, "no crystals paid");
        // A downed tier opens the gate. The staff is not autoReuse — the C#
        // needs the release edge for a second use (`Player.cs:42733`), so the
        // witness releases and re-presses between the two fires.
        app.world.dd2_downed_t1 = true;
        app.mine_down = false;
        fire(&mut app);
        app.mine_down = true;
        fire(&mut app);
        assert!(
            app.projectiles
                .slots()
                .iter()
                .any(|p| p.active && p.kind == 663),
            "the tower spawns once a tier is downed"
        );
        assert_eq!(app.inventory[1].count, 0, "the 10 crystals paid");
    }

    /// The Ballista popper (3824 → 677) seeds `ai1` with the
    /// `GetBallistraShotDelay` ladder (`Projectile.cs:68653-68669`): 160 stock,
    /// 100 with setSquireT3, 60 with ballistaPanic, 30 with both.
    #[test]
    fn the_ballista_popper_seeds_the_shot_delay_ladder() {
        let cases: [(bool, bool, f32); 4] = [
            (false, false, 160.0),
            (true, false, 100.0),
            (false, true, 60.0),
            (true, true, 30.0),
        ];
        for (squire_t3, panic, want) in cases {
            let mut app = sentry_app(3824);
            app.world.dd2_downed_t1 = true;
            app.inventory[1] = ItemSlot {
                kind: 3822,
                name: "DD2 Energy Crystal",
                count: 10,
                prefix: 0,
            };
            {
                let p = app.player.as_mut().unwrap();
                p.set_squire_t3 = squire_t3;
                p.ballista_panic = panic;
            }
            fire(&mut app);
            let s = app
                .projectiles
                .slots()
                .iter()
                .find(|p| p.active && p.kind == 677)
                .expect("the Ballista tower spawned");
            assert_eq!(s.ai[0], 0.0, "the Ballista carries no ai0");
            assert_eq!(
                s.ai[1], want,
                "squireT3={squire_t3} panic={panic} → {want}"
            );
        }
    }

    /// The sentry knockback takes `GetWeaponKnockback`'s summon arm
    /// (`Player.cs:53346-53369`): `KnockBack += minionKB` (`:53348-53351`),
    /// then the `kbBuff` ×1.5 (`:53356-53359`). The Frost Hydra staff's base
    /// knockback is 7.5.
    #[test]
    fn the_sentry_spawn_carries_the_summon_knockback() {
        let mut app = sentry_app(1572);
        {
            let p = app.player.as_mut().unwrap();
            p.minion_kb = 1.5;
            p.kb_buff = true;
        }
        fire(&mut app);
        let s = app
            .projectiles
            .slots()
            .iter()
            .find(|p| p.active && p.kind == 308)
            .expect("the Frost Hydra spawned");
        // (7.5 + 1.5) × 1.5 = 13.5.
        assert!(
            (s.knock_back - 13.5).abs() < 1e-4,
            "the summon knockback ladder: {}",
            s.knock_back
        );
    }

    /// The spawn sweep runs right after the sentry spawn (`Player.cs:48210`):
    /// at `max_turrets = 1` a second Frost Hydra use evicts the older, lower-
    /// `timeLeft` sentry (`UpdateMaxTurrets` `:53076-53088` — the lowest life
    /// dies while the wipable count exceeds the budget). The aged slot is
    /// seeded through the pool seam directly (the use path's own first spawn
    /// would carry the fresh 36000 and tie the scan's first-listed keep).
    #[test]
    fn the_spawn_sweep_evicts_the_lowest_life_sentry_over_cap() {
        let mut app = sentry_app(1572);
        app.player.as_mut().unwrap().max_turrets = 1;
        // An aged Hydra already on the field (timeLeft 35000 < the fresh
        // spawn's 36000) — the pool seam the AI arms' own spawns use.
        let aged = app
            .projectiles
            .spawn_with_data_ai(308, 600.0, 4000.0, 0.0, 0.0, 100, 7.5, 0.0, 0.0)
            .expect("the aged Hydra");
        app.projectiles.set_time_left(aged, 35000);
        fire(&mut app);
        let live: Vec<usize> = app
            .projectiles
            .slots()
            .iter()
            .enumerate()
            .filter(|(_, p)| p.active && p.kind == 308)
            .map(|(i, _)| i)
            .collect();
        assert_eq!(live.len(), 1, "the cap sweep keeps one");
        assert_ne!(live[0], aged, "the aged lower-life Hydra was evicted");
    }
}

/// S1-F-13 Phase 1: the medallion / Naughty-Present triggers
/// (`Player.cs:43974-43985`, `:44041-44052`).
#[cfg(test)]
mod moon_item_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;

    /// A held stack of the given kind in the selected slot, over an app whose
    /// world is at night with no events running.
    fn night_app(kind: u16) -> App {
        let mut app = flow_app(None, vec![], vec![]);
        let mut player = crate::player::Player::spawn(&app.world);
        player.x = 64.0;
        player.y = 64.0;
        app.player = Some(player);
        app.inventory[app.selected] = ItemSlot {
            kind,
            name: "test".into(),
            count: 1,
            prefix: 0,
        };
        app.world.day_time = false;
        app
    }

    /// DW-1.1: the medallion at night starts the Pumpkin Moon — the flags,
    /// the resets, `waveNumber = 1`, the stack payment and the swing.
    #[test]
    fn the_medallion_at_night_starts_the_pumpkin_moon() {
        let mut app = night_app(1844);
        assert!(use_pumpkin_moon_medallion(&mut app));
        assert!(app.world.pumpkin_moon && !app.world.snow_moon);
        assert_eq!(
            (
                app.world.wave_number,
                app.world.wave_kills,
                app.world.total_invasion_points
            ),
            (1, 0.0, 0.0),
            "Main.cs:10979-10996's single-player resets"
        );
        assert_eq!(app.inventory[app.selected].count, 0, "the use paid one");
        // `ApplyItemTime` (`Player.cs:43976` → `SetItemTime(useTime)`,
        // `:4416-4419`) re-arms the ACTION cadence — `itemTime`, not the
        // animation: the C# gate is `ItemTimeIsZero && itemAnimation > 0`,
        // the swing itself is untouched. The medallion's useTime is 45
        // (`Item.cs`'s 1844 row).
        assert!(
            (app.item_time - 45.0 / 60.0).abs() < 1e-6,
            "ApplyItemTime armed the action cadence (itemTime = {})",
            app.item_time
        );
    }

    /// DW-1.1: the Naughty Present starts the Snow Moon twin — and the
    /// mutual clears work in both directions.
    #[test]
    fn the_naughty_present_at_night_starts_the_snow_moon() {
        let mut app = night_app(1958);
        assert!(use_naughty_present(&mut app));
        assert!(app.world.snow_moon && !app.world.pumpkin_moon);
        assert_eq!(
            (
                app.world.wave_number,
                app.world.wave_kills,
                app.world.total_invasion_points
            ),
            (1, 0.0, 0.0)
        );
        // The mutual clear lives in `start_*` itself (covered on `World`):
        // the C# trigger's `!pumpkinMoon && !snowMoon` gate means the medallion
        // can never be used over the twin — a running snow moon refuses it
        // and its state stays untouched.
        let mut app = night_app(1844);
        app.world.start_snow_moon();
        app.world.wave_number = 6;
        assert!(!use_pumpkin_moon_medallion(&mut app));
        assert!(app.world.snow_moon && !app.world.pumpkin_moon);
        assert_eq!(app.world.wave_number, 6, "the refusal leaves the twin alone");
        assert_eq!(app.inventory[app.selected].count, 1, "and it is free");
        // And the present over a running pumpkin moon refuses the same way.
        let mut app = night_app(1958);
        app.world.start_pumpkin_moon();
        app.world.wave_number = 6;
        assert!(!use_naughty_present(&mut app));
        assert!(app.world.pumpkin_moon && !app.world.snow_moon);
        assert_eq!(app.world.wave_number, 6);
        assert_eq!(app.inventory[app.selected].count, 1);
    }

    /// DW-1.1: by day, with either moon event running, or under the Old One's
    /// Army the triggers refuse — and the refusal is free (`Player.cs:43974`'s
    /// `!dayTime && !pumpkinMoon && !snowMoon && !DD2Event.Ongoing`).
    #[test]
    fn the_moon_items_refuse_by_day_or_with_an_event_running() {
        let mut app = night_app(1844);
        app.world.day_time = true;
        assert!(!use_pumpkin_moon_medallion(&mut app));
        assert!(!app.world.pumpkin_moon && !app.world.snow_moon);
        assert_eq!(app.inventory[app.selected].count, 1, "the refusal is free");
        assert!(!use_naughty_present(&mut app));
        assert_eq!(app.inventory[app.selected].count, 1, "the refusal is free");
        // A running pumpkin event refuses the present and the twin's arm...
        let mut app = night_app(1958);
        app.world.start_pumpkin_moon();
        assert!(!use_naughty_present(&mut app));
        assert_eq!(app.inventory[app.selected].count, 1);
        // ...and a running snow event refuses the medallion.
        let mut app = night_app(1844);
        app.world.start_snow_moon();
        assert!(!use_pumpkin_moon_medallion(&mut app));
        assert_eq!(app.inventory[app.selected].count, 1);
        // The Old One's Army refuses both.
        let mut app = night_app(1844);
        app.world.dd2_event_ongoing = true;
        assert!(!use_pumpkin_moon_medallion(&mut app));
        let mut app = night_app(1958);
        app.world.dd2_event_ongoing = true;
        assert!(!use_naughty_present(&mut app));
    }
}
/// The item-mechanics slice: the three player-facing flows the port keeps
/// (KillTile → drop, pickup → stack to `maxStack`, item → tile placement) plus
/// the dropped-item physics pinned to `WorldItem.MoveInWorld`'s constants.
#[cfg(test)]
mod item_mechanics {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::item::{EMPTY_SLOT, ItemDrop, INV};
    use tr_core::tile::Tile;
    use tr_core::tile::FLAG_ACTIVE;
    // S1-P-i28 — the thrown-consumable witnesses below reuse the mining rig
    // from the sibling `tests` mod (the player parked three tiles right of
    // the target, the camera pinned so the screen-centre cursor points at it).
    // Both helpers are `pub(super)` there; without these imports the calls do
    // not resolve (they are private to `tests`, not visible through `super::*`).
    use super::tests::{MINE_SCREEN, mining_app};

    fn slot(kind: u16, count: u32) -> ItemSlot {
        ItemSlot {
            kind,
            name: registry::name(kind),
            count,
            prefix: 0,
        }
    }

    /// A flat world with one solid stone row at `y = 20` — the floor the
    /// placement and the drop physics need.
    fn flat_world() -> World {
        let mut w = World::new("items".into(), 40, 40);
        for x in 0..40 {
            let i = w.index(x, 20);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    fn flat_app() -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = flat_world();
        let mut player = Player::spawn(&app.world);
        player.x = 10.0 * TILE_PX;
        player.y = 18.0 * TILE_PX;
        app.player = Some(player);
        app.inventory = std::array::from_fn(|_| EMPTY_SLOT);
        app
    }

    fn set(app: &mut App, x: i32, y: i32, kind: u16) {
        let i = app.world.index(x, y);
        app.world.tiles[i] = Tile {
            kind,
            ..Default::default()
        };
        app.world.tiles[i].flags |= FLAG_ACTIVE;
    }

    /// t1 — `WorldGen.KillTile_GetItemDrops`'s dirt arm
    /// (`WorldGen.cs:64057-64062`: `case 0/case 2/… → dropItem = 2`) and the
    /// `KillTile_DropItems` spawn (`WorldGen.cs:63947-63958`): breaking a dirt
    /// tile leaves one Dirt Block drop at the tile (`x*16, y*16, 16, 16`).
    #[test]
    fn breaking_dirt_drops_dirt_item() {
        // Pins: `ItemID.DirtBlock` = 2, `TileID.Dirt` = 0.
        assert_eq!(registry::tile_drop(0, 0), Some(2), "dirt drops dirt");
        let mut app = flat_app();
        set(&mut app, 10, 19, 0);
        break_tile(&mut app, 10, 19);
        assert!(!app.world.tile(10, 19).unwrap().active(), "the tile broke");
        assert_eq!(app.items.len(), 1, "exactly one drop");
        assert_eq!(app.items[0].kind, 2, "the Dirt Block item");
        assert_eq!(app.items[0].count, 1, "one tile, one block");
    }

    /// t2 — `Player.GetItem_FillIntoOccupiedSlot`
    /// (`Player.cs:39493-39518`): a partial fill stops at `maxStack`; the
    /// leftover rides the empty-slot passes (`:39365-39394`) and only a
    /// grid with NO room keeps it in the world (`PickupItem`,
    /// `Player.cs:35103`: `itemToPickUp.ReplaceWith(item)`).
    #[test]
    fn pickup_merges_to_maxstack() {
        // `Item.CommonMaxStack = 9999` (`Item.cs:49966`) is what `SetDefaults`
        // stamps on every row (`Item.cs:49017`), so stone and dirt both cap
        // there.
        const MAX: u32 = 9999;
        assert_eq!(registry::max_stack(3), MAX, "Item.CommonMaxStack");
        // Case 1: the merge fills slot 0 to the cap, the remainder 7 lands
        // in the next empty HOTBAR slot (the usable-kind arm `:39365`).
        let mut app = flat_app();
        app.inventory[0] = slot(3, MAX - 3);
        let pc = app.player.as_ref().unwrap().center();
        let mut drop = ItemDrop::new(3, 10, 0, pc.0 - 4.0, pc.1 - 4.0, 0.0, 0.0, 0.0);
        drop.age = 1.0;
        app.items.push(drop);
        crate::world_sim::items::update_items(&mut app, ONE_TICK_SECS);
        assert_eq!(app.inventory[0].count, MAX, "the slot fills to maxStack");
        assert_eq!(app.inventory[1].count, 7, "the overflow opens slot 1");
        assert_eq!(app.inventory[1].kind, 3);
        assert!(app.items.is_empty(), "the drop was fully consumed");
        // Case 2: a grid with no room at all — the surplus stays on the
        // ground (`ReplaceWith`), never an over-full slot.
        let mut app = flat_app();
        for s in app.inventory.iter_mut() {
            *s = slot(2, registry::max_stack(2));
        }
        app.inventory[0] = slot(3, MAX - 3);
        let pc = app.player.as_ref().unwrap().center();
        let mut drop = ItemDrop::new(3, 10, 0, pc.0 - 4.0, pc.1 - 4.0, 0.0, 0.0, 0.0);
        drop.age = 1.0;
        app.items.push(drop);
        crate::world_sim::items::update_items(&mut app, ONE_TICK_SECS);
        assert_eq!(app.inventory[0].count, MAX, "the slot fills to maxStack");
        assert_eq!(app.items.len(), 1, "the remainder is still on the ground");
        assert_eq!(app.items[0].count, 7, "10 - (maxStack - 9996) = 7");
        assert_eq!(
            app.inventory.iter().filter(|s| s.kind == 3).count(),
            1,
            "no second stone stack while the grid is full"
        );
    }

    /// t3 — `Player.PlaceThing` (`Player.cs:39580`) through the item route:
    /// the torch item (8) resolves `createTile = 4` (`Item.cs:33745-33754`'s
    /// `placeStyle`/`createTile` pair) and `place_item_and_consume` spends one
    /// from the held stack.
    #[test]
    fn placing_torch_consumes_item_and_spawns_tile() {
        // Pins: `ItemID.Torch` = 8, `TileID.Torch` = 4.
        assert_eq!(registry::place_tile(8), Some(4), "the torch places tile 4");
        let mut app = flat_app();
        app.inventory[app.selected] = slot(8, 5);
        // Away from the player's own box — `can_place` refuses the cells the
        // player's 20x42 body covers (`Player.cs:41427`'s
        // `BlockPlacementIfOverPlayers` stand-in).
        assert!(use_tile_item(&mut app, 15, 19), "the placement succeeds");
        let t = *app.world.tile(15, 19).unwrap();
        assert!(t.active() && t.kind == 4, "a torch stands on the floor");
        assert_eq!(app.inventory[app.selected].count, 4, "one torch consumed");
        assert_eq!(app.inventory[app.selected].kind, 8, "the stack stays torch");
    }

    /// t4 — `WorldItem.UpdateItem`'s air constants (`WorldItem.cs:363-364`:
    /// `gravity = 0.1f`, `maxFallSpeed = 7f`), the horizontal friction of
    /// `MoveInWorld` (`WorldItem.cs:854-860`: `velocity.X *= 0.95f`, then the
    /// `< 0.1` snap) and the flush stop of `Collision.TileCollision`
    /// (`Collision.cs:2446-2451` — the blocked axis keeps only the residual, so
    /// the drop does not bounce).
    #[test]
    fn dropped_item_lies_on_ground_with_csharp_physics() {
        let mut app = flat_app();
        let floor_top = 20.0 * TILE_PX;
        // In the air: one step adds exactly `gravity` and applies the friction.
        let mut drop = ItemDrop::new(2, 1, 0, 100.0, 100.0, 2.0, 0.0, 0.0);
        drop.update(&app.world, ONE_TICK_SECS);
        assert!(
            (drop.vel.y - 0.1).abs() < 1e-6,
            "gravity is 0.1, got {}",
            drop.vel.y
        );
        assert!(
            (drop.vel.x - 1.9).abs() < 1e-6,
            "friction is 0.95, got {}",
            drop.vel.x
        );
        // The fall speed caps at 7.
        let mut fast = ItemDrop::new(2, 1, 0, 100.0, 100.0, 0.0, 40.0, 0.0);
        fast.update(&app.world, ONE_TICK_SECS);
        assert!((fast.vel.y - 7.0).abs() < 1e-6, "maxFall is 7, got {}", fast.vel.y);
        // Resting on the floor: the drop lies still, it never bounces.
        let mut rest = ItemDrop::new(2, 1, 0, 100.0, floor_top - 14.0, 0.0, 0.0, 0.0);
        for _ in 0..30 {
            rest.update(&app.world, ONE_TICK_SECS);
        }
        assert_eq!(rest.vel.y, 0.0, "no restitution off the ground");
        assert_eq!(rest.vel.x, 0.0, "friction settles the slide");
        assert!(
            (rest.pos.y - (floor_top - 14.0)).abs() < 0.5,
            "the drop lies flush on the floor, at {}",
            rest.pos.y
        );
    }

    // ---- S1-P-i28: the thrown-consumable throwing arm --------------------
    // WRITTEN, NOT RUN (code-only mode). The fixtures reuse `mining_app` (the
    // player parked three tiles right of the target, the cursor at the screen
    // centre). The thrown weapon's `useAmmo == 0`, so it takes the generic
    // shoot path (the press-frame fire), not the ammo-shooter path.

    /// A thrown-consumable rig: the held stack set to `kind`/`count`, the
    /// cursor 80 px right of the player (a clear rightward aim).
    fn thrown_app(kind: u16, count: u32) -> App {
        let w = World::new("t".into(), 64, 800);
        let mut app = mining_app(w, 30, 260);
        app.inventory[app.selected] = crate::item::ItemSlot {
            kind,
            name: "thrown",
            count,
            prefix: 0,
        };
        let (sw, sh) = app.screen_size();
        app.mouse_pos = Vec2::new(sw * 0.5 + 80.0, sh * 0.5);
        app.mine_down = true;
        // The press edge (`Player.cs:42733`'s `controlUseItem && releaseUseItem`):
        // the non-autoReuse thrown kinds (the shuriken, the waters, the
        // grenades, …) only fire on a FRESH press — the previous frame must
        // read released-and-not-down, or `throw_ready` refuses the first
        // frame's swing. `mining_app` leaves both latches false (a held
        // button), so arm the release the vanilla starts from.
        app.mine_prev_down = false;
        app.mine_released = true;
        app
    }

    // ---- S1-P-i37: the drink press-edge regression witnesses ---------------
    // WRITTEN NOT RUN (code-only swarm). The P1 census bug: the old heal arm
    // computed `sick` but never read it and ran on every `mine_down` tick —
    // holding LMB drained the whole stack and re-armed buff 21 forever. The
    // fix gates the consumable arms on the `Player.cs:42733` press edge
    // (`controlUseItem && releaseUseItem && itemAnimation == 0`) and routes
    // the drink through `potions::use_potion_item` (the `:52217` sickness
    // gate reads `potionDelay`, not the buff slot).

    pub(super) const DRINK_SCREEN: Vec2 = Vec2::new(1280.0, 720.0);

    /// A drink rig on the flow app's 8x8 air world: the player parked at the
    /// world centre, the camera pinned on the cursor cell (in bounds, in
    /// reach, no tile under it — the melee/mining arms fall through), the
    /// SILENT listener so the use-sound draws nothing.
    pub(super) fn potion_press_app(kind: u16, count: u32) -> App {
        let mut app = flow_app(None, vec![], vec![]);
        let mut p = Player::spawn(&app.world);
        p.x = 4.0 * TILE_PX;
        p.y = 4.0 * TILE_PX;
        p.hp = 50;
        app.player = Some(p);
        app.camera = Vec2::new(4.0 * TILE_PX + 8.0, 4.0 * TILE_PX + 8.0);
        app.mouse_pos = Vec2::new(640.0, 360.0);
        app.audio.set_listener(crate::sound::Listener {
            camera_center: (app.camera.x, app.camera.y),
            sound_volume: 0.0,
            ambient_volume: 1.0,
            quiet_ambient_sounds: false,
            game_menu: false,
            generating_or_loading: false,
            server: false,
        });
        app.inventory[app.selected] = ItemSlot {
            kind,
            name: registry::name(kind),
            count,
            prefix: 0,
        };
        // The released state: one idle frame arms the press edge.
        app.mine_down = false;
        app.mine_prev_down = false;
        app.mine_released = true;
        app
    }

    /// The shuriken throw spawns the aiStyle-2 shuriken (3) at shootSpeed 9
    /// and consumes one from the stack (`Player.cs:47056+`, `:43317`).
    #[test]
    fn s1_p_i28_shuriken_throws_and_consumes() {
        let mut app = thrown_app(42, 5);
        let screen = MINE_SCREEN;
        update_editing(&mut app, 1.0 / 60.0, screen);
        let shot = app
            .projectiles
            .slots()
            .iter()
            .find(|p| p.active)
            .expect("the shuriken spawns");
        assert_eq!(shot.kind, 3, "ItemID.Shuriken's shoot is 3");
        // The aim is rightward; the speed is the item's shootSpeed 9.
        assert!(shot.vel.x > 0.0, "the throw flies toward the cursor");
        assert!((shot.vel.length() - 9.0).abs() < 1e-3, "shootSpeed 9");
        assert_eq!(app.inventory[app.selected].count, 4, "one consumed");
    }

    /// The bone javelin throws from the upper third of the hitbox
    /// (`Player.cs:47225-47228`): spawn.Y = position.Y + height/3.
    #[test]
    fn s1_p_i28_bone_javelin_spawn_height() {
        let mut app = thrown_app(3378, 5);
        let screen = MINE_SCREEN;
        let py = app.player.as_ref().unwrap().y;
        let ph = app.player.as_ref().unwrap().height;
        update_editing(&mut app, 1.0 / 60.0, screen);
        let shot = app
            .projectiles
            .slots()
            .iter()
            .find(|p| p.active)
            .expect("the bone javelin spawns");
        assert_eq!(shot.kind, 598, "ItemID.BoneJavelin's shoot is 598");
        // The spawn Y is the upper third (`Player.cs:47227`): the port's
        // generic spawn point is the centre + dir*10, but the javelin arm
        // overrides `spawn.y = position.Y + height/3`, and
        // `spawn_with_data_ai` takes that point as the projectile's TOP-LEFT
        // (`pos`, not a centre). So `shot.pos.y` is exactly `p.y + height/3`.
        let expected_y = py + (ph as i32 / 3) as f32;
        assert!(
            (shot.pos.y - expected_y).abs() < 1e-3,
            "the javelin throws from the upper third, got {} want {}",
            shot.pos.y,
            expected_y
        );
    }

    /// The last throw locks the action cadence (`Player.cs:43321-43324`):
    /// `itemTime = itemAnimation` when the stack empties.
    #[test]
    fn s1_p_i28_last_throw_locks_cadence() {
        let mut app = thrown_app(42, 1);
        let screen = MINE_SCREEN;
        update_editing(&mut app, 1.0 / 60.0, screen);
        assert_eq!(app.inventory[app.selected].count, 0, "the stack empties");
        // itemTime is latched to the swing (the remaining animation).
        assert!(
            (app.item_time - app.swing).abs() < 1e-6,
            "itemTime = itemAnimation on the empty stack"
        );
    }

    /// The water bottles carry no `ranged` flag, so they skip the not-consume
    /// roll chain and always consume (`Item.cs:6285-6318`, `Player.cs:43283`).
    #[test]
    fn s1_p_i28_holy_water_always_consumes() {
        let mut app = thrown_app(422, 3);
        // Arm every not-consume flag — the water bottle must ignore them all.
        if let Some(p) = app.player.as_mut() {
            p.huntress_ammo_cost90 = true;
            p.chloro_ammo_cost80 = true;
            p.ammo_cost80 = true;
            p.ammo_cost75 = true;
        }
        let screen = MINE_SCREEN;
        update_editing(&mut app, 1.0 / 60.0, screen);
        let shot = app
            .projectiles
            .slots()
            .iter()
            .find(|p| p.active)
            .expect("the holy water spawns");
        assert_eq!(shot.kind, 69, "ItemID.HolyWater's shoot is 69");
        assert_eq!(app.inventory[app.selected].count, 2, "the water always consumes");
    }
}

#[cfg(test)]
mod s1_p_i08_throw_tests {
    //! S1-P-i08 (`task/S1-P-i08-boomerang`, code-only mode — tests WRITTEN
    //! NOT RUN): the useStyle-1 boomerang throw arm. The stack limits
    //! (`Player.cs:52162-52186`: one in flight for 6/52/113, six for the
    //! Light Disc 106), the exact-center spawn (`RotatedRelativePoint
    //! (MountedCenter)` `:47196` — the generic arm's `+ dir * 10` is the
    //! port's own deviation, dropped for the variant), the `ChangeDir` aim
    //! flip (`:47202-47224`), the zero-aim facing fallback (`:47291-47296`)
    //! and `GetWeaponKnockback`'s melee arms (`:53352-53359`).

    use super::*;
    // M1 re-sync: the i37 drink witnesses landed in this module by the merge
    // splice; their rig lives in the sibling item_mechanics module.
    use super::item_mechanics::{DRINK_SCREEN, potion_press_app};
    use crate::projectile::{
        ENCHANTED_BOOMERANG, ICE_BOOMERANG, LIGHT_DISC, WOODEN_BOOMERANG,
    };
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    /// The shooter layout (`tests::shooter_app`'s shape): a 64×800 world
    /// with a floor at row 300, the player at tile (33, 260), the camera
    /// on (488, 4168) and the mine button held.
    fn rig_world() -> World {
        let mut w = World::new("t".into(), 64, 800);
        for x in 0..64 {
            let i = w.index(x, 300);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        w.compute_column_surface();
        w
    }

    /// The boomerang rig: the given boomerang held and the cursor 80 px
    /// right of the player centre (the +x aim — the camera sits 13 px
    /// above the centre in this rig, so the cursor's Y rides the centre,
    /// not the screen middle).
    fn boomerang_app(item: u16, name: &'static str) -> App {
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        app.world = rig_world();
        let mut player = crate::player::Player::spawn(&app.world);
        player.x = 33.0 * TILE_PX;
        player.y = 260.0 * TILE_PX;
        app.player = Some(player);
        app.camera = Vec2::new(30.0 * TILE_PX + 8.0, 260.0 * TILE_PX + 8.0);
        app.inventory[app.selected] = ItemSlot {
            kind: item,
            name,
            count: 1,
            prefix: 0,
        };
        app.mine_down = true;
        // The press edge: the non-autoReuse boomerangs fire on it
        // (`Player.cs:42733`'s `controlUseItem && releaseUseItem`).
        app.mine_prev_down = false;
        app.mine_released = true;
        let (sw, sh) = app.screen_size();
        let (cx, cy) = app.player.as_ref().unwrap().center();
        let off = (Vec2::new(cx, cy) - app.camera) * app.zoom;
        app.mouse_pos = Vec2::new(sw * 0.5 + off.x + 80.0, sh * 0.5 + off.y);
        app
    }

    /// One fire tick of the held boomerang (`tests::fire_once`'s shape).
    fn fire_once(app: &mut App) {
        let (sw, sh) = app.screen_size();
        update_editing(app, 1.0 / 60.0, Vec2::new(sw, sh));
    }

    /// Pins the editing pass's `UpdatePlayers` stream to `seed`
    /// (`tests::seed_stream`'s shape).
    fn seed_stream(app: &mut App, seed: i32) {
        *app.rngs.stream(crate::app::RngKey::UpdatePlayers) = tr_core::UnifiedRandom::new(seed);
    }

    /// The stream state after `n` draws from `seed` (`tests::draws_after`).
    fn draws_after(seed: i32, n: usize) -> tr_core::UnifiedRandom {
        let mut rng = tr_core::UnifiedRandom::new(seed);
        for _ in 0..n {
            rng.next();
        }
        rng
    }

    /// The editing pass's `UpdatePlayers` stream state.
    fn stream_of(app: &mut App) -> tr_core::UnifiedRandom {
        app.rngs
            .stream(crate::app::RngKey::UpdatePlayers)
            .clone()
    }

    fn active_boomerangs(app: &App) -> Vec<usize> {
        app.projectiles
            .slots()
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                p.active
                    && matches!(
                        p.kind,
                        ENCHANTED_BOOMERANG | WOODEN_BOOMERANG | ICE_BOOMERANG | LIGHT_DISC
                    )
            })
            .map(|(i, _)| i)
            .collect()
    }

    #[test]
    fn s1_p_i08_the_throw_spawns_at_the_exact_center() {
        // `Player.cs:47196` + `:49437`: `pointPosition = RotatedRelativePoint
        // (MountedCenter)` — the player center with no rotation/mount
        // offsets; `NewProjectile` centers the box on it (`:10876-10877`).
        // The old generic arm spawned at center + dir * 10.
        let mut app = boomerang_app(284, "Wooden Boomerang");
        fire_once(&mut app);
        let slots = active_boomerangs(&app);
        assert_eq!(slots.len(), 1, "one wooden boomerang");
        let p = &app.projectiles.slots()[slots[0]];
        let (cx, cy) = app.player.as_ref().unwrap().center();
        // The box's CENTER lands on the player centre (`NewProjectile`'s
        // `position = X − width/2` — the slot's pos is the top-left).
        assert_eq!(
            p.pos + Vec2::new(p.w * 0.5, p.h * 0.5),
            Vec2::new(cx, cy),
            "the exact-center spawn"
        );
        assert_eq!(p.kind, WOODEN_BOOMERANG);
        assert_eq!(p.vel, Vec2::new(6.5, 0.0), "shootSpeed 6.5 along +x");
        assert_eq!(p.damage, 10, "weaponDamage at the ×1 class multiplier");
        assert_eq!(p.knock_back, 5.0, "the item's knockback");
        assert_eq!(p.ai, [0.0, 0.0, 0.0, 0.0], "the generic ai0/ai1/ai2 = 0");
        assert_eq!(
            app.inventory[app.selected].count, 1,
            "the boomerang is not consumed"
        );
    }

    #[test]
    fn s1_p_i08_the_throw_flips_the_facing_to_the_aim() {
        // `Player.cs:47213-47224`: `Dot(UnitX, aim) > 0` → ChangeDir(1),
        // else ChangeDir(-1). The player starts facing right; a left aim
        // flips to −1, a right aim keeps +1.
        let mut app = boomerang_app(284, "Wooden Boomerang");
        let (sw, sh) = app.screen_size();
        let (cx, cy) = app.player.as_ref().unwrap().center();
        let off = (Vec2::new(cx, cy) - app.camera) * app.zoom;
        app.mouse_pos = Vec2::new(sw * 0.5 + off.x - 80.0, sh * 0.5 + off.y);
        fire_once(&mut app);
        assert_eq!(app.player.as_ref().unwrap().facing, -1, "the left aim");
        let slots = active_boomerangs(&app);
        assert_eq!(slots.len(), 1);
        assert_eq!(
            app.projectiles.slots()[slots[0]].vel,
            Vec2::new(-6.5, 0.0),
            "the shot flies left"
        );
        // The right aim on a fresh app keeps +1.
        let mut app = boomerang_app(284, "Wooden Boomerang");
        fire_once(&mut app);
        assert_eq!(app.player.as_ref().unwrap().facing, 1, "the right aim");
    }

    #[test]
    fn s1_p_i08_the_zero_aim_falls_back_to_the_facing() {
        // `Player.cs:47291-47296`: the zero mouse delta sets `num4 =
        // direction, num5 = 0, num6 = speed` — the shot still fires, at the
        // facing × shootSpeed. The old `dir != ZERO` gate skipped the throw
        // entirely.
        let mut app = boomerang_app(284, "Wooden Boomerang");
        let (sw, sh) = app.screen_size();
        let (cx, cy) = app.player.as_ref().unwrap().center();
        let off = (Vec2::new(cx, cy) - app.camera) * app.zoom;
        // The cursor exactly on the player centre: the zero mouse delta
        // (`normalize_or_zero` folds the exact-zero aim to ZERO).
        app.mouse_pos = Vec2::new(sw * 0.5 + off.x, sh * 0.5 + off.y);
        app.player.as_mut().unwrap().facing = -1;
        fire_once(&mut app);
        let slots = active_boomerangs(&app);
        assert_eq!(slots.len(), 1, "the zero aim still throws");
        assert_eq!(
            app.projectiles.slots()[slots[0]].vel,
            Vec2::new(-6.5, 0.0),
            "the facing fallback"
        );
        // The zero aim's ChangeDir takes the −1 arm too (`num > 0` is false
        // at the zero dot) — the facing stays −1.
        assert_eq!(app.player.as_ref().unwrap().facing, -1);
    }

    #[test]
    fn s1_p_i08_one_boomerang_in_flight_blocks_the_rethrow() {
        // `Player.cs:52162-52171`: the single-flight list refuses the use
        // while ANY active same-kind projectile of this owner exists —
        // across all three ported kinds (6/52/113 each gate on their own).
        // The rig never decays `attack_cd` (that pass lives outside
        // `update_editing`), so every attempt re-arms the press edge and
        // clears the two locks (`tests::repress`'s convention).
        for (item, name, kind) in [
            (55u16, "Enchanted Boomerang", ENCHANTED_BOOMERANG),
            (284, "Wooden Boomerang", WOODEN_BOOMERANG),
            (670, "Ice Boomerang", ICE_BOOMERANG),
        ] {
            let mut app = boomerang_app(item, name);
            for _ in 0..30 {
                app.attack_cd = 0.0;
                app.reuse_delay = 0.0;
                app.mine_prev_down = false;
                app.mine_released = true;
                fire_once(&mut app);
            }
            let live = active_boomerangs(&app);
            assert_eq!(live.len(), 1, "{name}: exactly one in flight");
            assert_eq!(app.projectiles.slots()[live[0]].kind, kind);
            // Kill it: the next press edge throws again.
            app.projectiles.kill(live[0]);
            app.attack_cd = 0.0;
            app.reuse_delay = 0.0;
            app.mine_prev_down = false;
            app.mine_released = true;
            fire_once(&mut app);
            assert_eq!(active_boomerangs(&app).len(), 1, "{name}: rethrown");
        }
    }

    #[test]
    fn s1_p_i08_the_light_disc_stacks_to_six() {
        // `Player.cs:52172-52186`: the 106 counts its active same-owner
        // shots and refuses at `>= 6`. The disc is autoReuse — the held
        // button rethrows at the use cadence (useTime 14).
        let mut app = boomerang_app(561, "Light Disc");
        // The rig's `update_editing` never ticks the projectile pool, so
        // every thrown disc stays active at its spawn point — the cap is a
        // pure spawn-count gate here. The cursor stays at the rig's +80 px
        // (inside the reach gate, which returns before the throw block).
        // The autoReuse cadence: the rig never decays `attack_cd` inside
        // `update_editing`, so the loop re-arms it every 14 ticks (the
        // disc's useTime/useAnimation) — the C# held-button cadence.
        for tick in 0..84 {
            if tick % 14 == 0 {
                app.attack_cd = 0.0;
                app.reuse_delay = 0.0;
            }
            fire_once(&mut app);
        }
        let live = active_boomerangs(&app);
        assert_eq!(live.len(), 6, "the six-disc cap");
        assert!(
            live.iter()
                .all(|&i| app.projectiles.slots()[i].kind == LIGHT_DISC),
            "all six are discs"
        );
        // The next cadence attempt refuses while all six live.
        app.attack_cd = 0.0;
        app.reuse_delay = 0.0;
        fire_once(&mut app);
        assert_eq!(active_boomerangs(&app).len(), 6, "the cap holds");
        // Free one slot: the next cadence use fills it back to six.
        app.projectiles.kill(live[0]);
        app.attack_cd = 0.0;
        app.reuse_delay = 0.0;
        fire_once(&mut app);
        assert_eq!(active_boomerangs(&app).len(), 6, "the freed slot refills");
    }

    #[test]
    fn s1_p_i08_the_knockback_glove_and_buff_scale_the_throw() {
        // `GetWeaponKnockback` (`Player.cs:53352-53359`): the melee kbGlove
        // doubles, the kbBuff ×1.5 — both multiply the item's knockback.
        let mut app = boomerang_app(284, "Wooden Boomerang");
        app.player.as_mut().unwrap().kb_glove = true;
        fire_once(&mut app);
        let slots = active_boomerangs(&app);
        assert_eq!(app.projectiles.slots()[slots[0]].knock_back, 10.0, "5 × 2");
        let mut app = boomerang_app(284, "Wooden Boomerang");
        {
            let p = app.player.as_mut().unwrap();
            p.kb_glove = true;
            p.kb_buff = true;
        }
        fire_once(&mut app);
        let slots = active_boomerangs(&app);
        assert_eq!(
            app.projectiles.slots()[slots[0]].knock_back, 15.0,
            "5 × 2 × 1.5"
        );
    }

    #[test]
    fn s1_p_i08_the_throw_spends_no_updateplayers_draw() {
        // RNG census (gate reconciliation i08×i11): the boomerang throw
        // spends NO `UpdatePlayers` draw. The C# rolls the projectile crit
        // PER HIT in `Damage_PVE_Inner` (`Projectile.cs:12883-12917`) on the
        // `UpdateProjectiles` stream — `ItemCheck_Shoot`'s generic tail
        // (`Player.cs:49437`) carries no crit roll (the only throw-time
        // crit in the shoot path is the Zenith's melee arm, `:48859`).
        // S1-P-i11 moved the port's spawn-time roll to the hit; the i08
        // witness's "one crit draw" pinned the pre-i11 deviation. Two
        // identical seeds give byte-equal spawns; the stream is untouched.
        //
        // The Item126 use sound's case-2 pitch draws only AFTER the
        // attenuation gate passed. The listener is pinned FAR from the
        // player (the use plays at the player centre) so the gate refuses
        // and the pitch draw never fires — the witness is robust to the
        // rig's camera default (the retry census's observation 2: headless
        // does NOT mute the listener, the DISTANCE refuses). The
        // GetRandomPitch NextFloat rides the audio-only `sound_style_random`,
        // never this stream.
        let far_listener = |app: &mut App| {
            app.audio.set_listener(crate::sound::Listener {
                camera_center: (100000.0, 100000.0),
                sound_volume: 1.0,
                ambient_volume: 1.0,
                quiet_ambient_sounds: false,
                game_menu: false,
                generating_or_loading: false,
                server: false,
            });
        };
        let run = || {
            let mut app = boomerang_app(284, "Wooden Boomerang");
            far_listener(&mut app);
            seed_stream(&mut app, 0xB00B);
            fire_once(&mut app);
            app.projectiles
                .slots()
                .iter()
                .filter(|p| p.active)
                .map(|p| (p.kind, p.pos, p.vel, p.ai, p.crit))
                .collect::<Vec<_>>()
        };
        assert_eq!(run(), run(), "the throw is seed-stable");
        let mut app = boomerang_app(284, "Wooden Boomerang");
        far_listener(&mut app);
        seed_stream(&mut app, 0xB00B);
        fire_once(&mut app);
        let after = stream_of(&mut app);
        assert_eq!(after, draws_after(0xB00B, 0), "the throw spends no draw (crit is per-hit)");
    }

    #[test]
    fn holding_lmb_drinks_once_not_every_tick() {
        let mut app = potion_press_app(188, 30);
        update_editing(&mut app, 1.0 / 60.0, DRINK_SCREEN); // idle: edge armed
        app.mine_down = true;
        for _ in 0..120 {
            update_editing(&mut app, 1.0 / 60.0, DRINK_SCREEN);
        }
        let p = app.player.as_ref().unwrap();
        assert_eq!(app.inventory[app.selected].count, 29, "ONE drink per press");
        assert_eq!(p.hp, 100);
        assert!(p.potion_delay > 3500, "the 3600 clock barely decayed");
        let i = p.buffs.find_buff_index(21).expect("buff 21 granted once");
        assert!(p.buffs.buff_time[i] > 3500, "no per-tick re-arm");
    }

    #[test]
    fn a_second_press_while_sick_drinks_nothing() {
        let mut app = potion_press_app(188, 5);
        update_editing(&mut app, 1.0 / 60.0, DRINK_SCREEN);
        app.mine_down = true;
        update_editing(&mut app, 1.0 / 60.0, DRINK_SCREEN); // the first drink
        assert_eq!(app.inventory[app.selected].count, 4);
        // Release + re-press while the sickness clock runs: the `:52217`
        // gate refuses — no heal, no consume, no buff refresh. The swing
        // locks are cleared (their decay lives in the npcs pass, outside
        // `update_editing`) so the press edge REACHES the sickness gate —
        // the refusal under test is the gate's, not the animation's.
        app.mine_down = false;
        update_editing(&mut app, 1.0 / 60.0, DRINK_SCREEN);
        app.attack_cd = 0.0;
        app.reuse_delay = 0.0;
        let before = app.player.as_ref().unwrap().buffs.buff_time
            [app.player.as_ref().unwrap().buffs.find_buff_index(21).unwrap()];
        app.player.as_mut().unwrap().hp = 50;
        app.mine_down = true;
        update_editing(&mut app, 1.0 / 60.0, DRINK_SCREEN);
        assert_eq!(app.inventory[app.selected].count, 4, "the stack untouched");
        assert_eq!(app.player.as_ref().unwrap().hp, 50, "no heal while sick");
        let after = app.player.as_ref().unwrap().buffs.buff_time
            [app.player.as_ref().unwrap().buffs.find_buff_index(21).unwrap()];
        // `update_editing` does not tick the buff clocks (that is the player
        // pass) — the pin is the absence of a RE-ARM: the timer is unchanged.
        assert_eq!(after, before, "no buff-21 refresh on the refused press");
    }

    #[test]
    fn a_buff_potion_press_grants_its_buff_through_sickness() {
        // Ironskin 292 (buffType 5, buffTime 28800): NOT potion-flagged, so
        // the sickness clock does not gate it (`Item.cs:4858-4871`).
        let mut app = potion_press_app(292, 3);
        app.player.as_mut().unwrap().potion_delay = 3000;
        update_editing(&mut app, 1.0 / 60.0, DRINK_SCREEN);
        app.mine_down = true;
        update_editing(&mut app, 1.0 / 60.0, DRINK_SCREEN);
        let p = app.player.as_ref().unwrap();
        let i = p.buffs.find_buff_index(5).expect("Ironskin on the press");
        assert_eq!(p.buffs.buff_time[i], 28800);
        assert_eq!(app.inventory[app.selected].count, 2);
    }

    #[test]
    fn a_mana_potion_press_restores_and_grants_94() {
        // The Lesser Mana Potion 110 had NO use path before i37 (the old
        // arm gated on `potion && heal_life > 0`).
        let mut app = potion_press_app(110, 3);
        let p = app.player.as_mut().unwrap();
        p.mana = 0;
        p.mana_max = 200;
        update_editing(&mut app, 1.0 / 60.0, DRINK_SCREEN);
        app.mine_down = true;
        update_editing(&mut app, 1.0 / 60.0, DRINK_SCREEN);
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.mana, 50);
        assert!(p.buffs.find_buff_index(94).is_some(), "ManaSickness 94");
        assert_eq!(app.inventory[app.selected].count, 2);
    }
}

/// S1-P-i33 — the wall placement/removal census fixes: the adjacency gate,
/// the unsafe→safe replacement, the fill-empty-space tail, the hammer smash
/// gate + damage cadence, and the drop conversions. Code-only mode: the
/// witnesses are WRITTEN NOT RUN (no compiler invoked on the branch).
#[cfg(test)]
mod wall_placement_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    const SCREEN: Vec2 = Vec2::new(1280.0, 720.0);

    /// A wall-edit rig: the player parked three tiles right of `(tx, ty)`,
    /// the camera pinned so the screen-centre cursor points at the target,
    /// and the audio listener at the camera so the placement/break cues are
    /// audible (their RNG draws ride the `UpdatePlayers` stream).
    fn wall_app(world: World, tx: i32, ty: i32) -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = world;
        let mut player = Player::spawn(&app.world);
        player.x = (tx + 3) as f32 * TILE_PX;
        player.y = ty as f32 * TILE_PX;
        app.player = Some(player);
        app.camera = Vec2::new(tx as f32 * TILE_PX + 8.0, ty as f32 * TILE_PX + 8.0);
        app.mouse_pos = Vec2::new(640.0, 360.0);
        app.audio.set_listener(crate::sound::Listener {
            camera_center: (app.camera.x, app.camera.y),
            sound_volume: 1.0,
            ambient_volume: 1.0,
            quiet_ambient_sounds: false,
            game_menu: false,
            generating_or_loading: false,
            server: false,
        });
        app
    }

    fn empty_world() -> World {
        World::new("walls".into(), 40, 40)
    }

    fn set_wall(w: &mut World, x: i32, y: i32, wall: u16) {
        let i = w.index(x, y);
        w.tiles[i].wall = wall;
    }

    fn hold_wall(app: &mut App, kind: u16, count: u32) {
        app.inventory[app.selected] = crate::item::ItemSlot {
            kind,
            name: registry::name(kind),
            count,
            prefix: 0,
        };
    }

    /// `PlaceThing_Walls`' adjacency gate (`Player.cs:39607`): a neighbour
    /// that is ACTIVE (even wall-less) supports the placement; the cell's own
    /// tile never does. The old port required a neighbouring WALL or the
    /// cell's own solid tile — both wrong.
    #[test]
    fn s1_p_i33_an_active_wallless_neighbour_supports_the_wall() {
        let mut w = empty_world();
        // A stone block to the LEFT of the target, no wall anywhere.
        let i = w.index(9, 10);
        w.tiles[i] = Tile { kind: 1, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
        let mut app = wall_app(w, 10, 10);
        hold_wall(&mut app, 93, 5); // Wood Wall (createWall 4)
        assert!(can_place_wall(&app, 10, 10), "the active neighbour supports");
        // The cell's OWN tile does not support: a lone block with no
        // neighbours refuses.
        let mut w2 = empty_world();
        let j = w2.index(10, 10);
        w2.tiles[j] = Tile { kind: 1, ..Default::default() };
        w2.tiles[j].flags |= FLAG_ACTIVE;
        let mut app2 = wall_app(w2, 10, 10);
        hold_wall(&mut app2, 93, 5);
        assert!(!can_place_wall(&app2, 10, 10), "the own tile never supports");
    }

    /// The same-wall refusal (`Player.cs:39607`): a cell already carrying the
    /// held wall refuses; a DIFFERENT wall passes on to the replacement route.
    #[test]
    fn s1_p_i33_same_wall_refuses_but_a_different_wall_passes() {
        let mut w = empty_world();
        set_wall(&mut w, 9, 10, 4); // a wood-wall neighbour for support
        set_wall(&mut w, 10, 10, 4);
        let mut app = wall_app(w, 10, 10);
        hold_wall(&mut app, 93, 5); // Wood Wall → 4
        assert!(!can_place_wall(&app, 10, 10), "same wall refuses");
        // A different existing wall passes the gate (the replacement arm).
        set_wall(&mut app.world, 10, 10, 2); // unsafe dirt
        assert!(can_place_wall(&app, 10, 10), "a different wall passes");
    }

    /// `PlaceThing_TryReplacingWalls` (`Player.cs:39816-39829`) +
    /// `WorldGen.ReplaceWall` (`WorldGen.cs:62530-62556`): placing a safe wall
    /// onto an unsafe wall converts it, drops the OLD wall's item, and plays
    /// the OLD wall's break cue — not the placement Dig.
    #[test]
    fn s1_p_i33_unsafe_wall_converts_to_safe_and_drops_the_old_item() {
        let mut w = empty_world();
        set_wall(&mut w, 9, 10, 4); // support neighbour
        set_wall(&mut w, 10, 10, 2); // unsafe dirt wall
        let mut app = wall_app(w, 10, 10);
        hold_wall(&mut app, 93, 5); // Wood Wall → 4
        app.place_down = true;
        app.place_prev_down = false;
        update_editing(&mut app, 1.0 / 60.0, SCREEN);
        assert_eq!(app.world.tile(10, 10).unwrap().wall, 4, "converted to wood");
        // The unsafe dirt wall (2) drops NOTHING (`kill_wall_drop(2) == 0`).
        assert!(app.items.is_empty(), "unsafe dirt drops no item");
        assert_eq!(app.inventory[app.selected].count, 4, "one item consumed");
    }

    /// The replacement drops the OLD wall's converted item: a dungeon unsafe
    /// wall (7) replaced by its safe item's wall drops the safe item 135.
    #[test]
    fn s1_p_i33_replacing_a_dungeon_wall_drops_the_safe_item() {
        let mut w = empty_world();
        set_wall(&mut w, 9, 10, 4);
        set_wall(&mut w, 10, 10, 7); // blue dungeon unsafe
        w.downed_skeletron = true; // the progression gate opens
        let mut app = wall_app(w, 10, 10);
        hold_wall(&mut app, 135, 5); // Blue Brick Wall → 17
        app.place_down = true;
        app.place_prev_down = false;
        update_editing(&mut app, 1.0 / 60.0, SCREEN);
        assert_eq!(app.world.tile(10, 10).unwrap().wall, 17);
        assert!(
            app.items.iter().any(|d| d.kind == 135),
            "the old dungeon wall drops its safe item 135"
        );
    }

    /// `PlaceThing_Walls_FillEmptySpace` (`Player.cs:39636-39715`): the fill
    /// only closes a neighbour hole whose four sides ALL carry the same wall.
    /// A plus-shape around the target leaves every plus cell with an open
    /// outer side, so only the main placement lands (one item).
    #[test]
    fn s1_p_i33_fill_empty_space_needs_a_full_enclosure() {
        let mut w = empty_world();
        // A plus of wood walls around the hole (10,10).
        for (x, y) in [(9, 10), (11, 10), (10, 9), (10, 11)] {
            set_wall(&mut w, x, y, 4);
        }
        let mut app = wall_app(w, 10, 10);
        hold_wall(&mut app, 93, 5);
        app.place_down = true;
        app.place_prev_down = false;
        update_editing(&mut app, 1.0 / 60.0, SCREEN);
        // The hole (10,10) is filled by the main placement.
        assert_eq!(app.world.tile(10, 10).unwrap().wall, 4);
        // Each plus cell still has an open outer side (e.g. (9,10)'s left
        // (8,10) is empty), so the fill closes nothing more.
        assert_eq!(app.inventory[app.selected].count, 4, "only the hole filled");
    }

    /// The fill-empty-space tail fires when the placed cell completes a
    /// neighbour's four-wall enclosure: a 2x2 block missing one corner, the
    /// placement fills the corner AND the fill closes the remaining hole.
    #[test]
    fn s1_p_i33_fill_empty_space_consumes_a_second_item() {
        let mut w = empty_world();
        // Enclose (11,10) on three sides; (10,10) is the fourth (the target).
        set_wall(&mut w, 12, 10, 4);
        set_wall(&mut w, 11, 9, 4);
        set_wall(&mut w, 11, 11, 4);
        // Gate V3 fixture: the target (10,10) needs a supporting neighbour
        // (`PlaceThing_Walls`' four-neighbour gate, `Player.cs:39607`) — a
        // wall below it that is NOT one of (11,10)'s four sides, so the fill
        // still sees exactly the three-wall enclosure + the new (10,10).
        set_wall(&mut w, 10, 11, 4);
        let mut app = wall_app(w, 10, 10);
        hold_wall(&mut app, 93, 5);
        app.place_down = true;
        app.place_prev_down = false;
        update_editing(&mut app, 1.0 / 60.0, SCREEN);
        // Main placement fills (10,10); the fill then sees (11,10) enclosed
        // on all four sides (10,10 / 12,10 / 11,9 / 11,11) and fills it too.
        assert_eq!(app.world.tile(10, 10).unwrap().wall, 4);
        assert_eq!(app.world.tile(11, 10).unwrap().wall, 4, "the fill closed the hole");
        assert_eq!(app.inventory[app.selected].count, 3, "two items consumed");
    }

    /// The last item never fills (`Player.cs:39638`): with `stack == 1` the
    /// main placement lands but the fill-empty-space tail returns early.
    #[test]
    fn s1_p_i33_the_last_item_never_fills() {
        let mut w = empty_world();
        set_wall(&mut w, 12, 10, 4);
        set_wall(&mut w, 11, 9, 4);
        set_wall(&mut w, 11, 11, 4);
        // Gate V3 fixture: the target's supporting neighbour (see the
        // second-item test) — below the target, off (11,10)'s four sides.
        set_wall(&mut w, 10, 11, 4);
        let mut app = wall_app(w, 10, 10);
        hold_wall(&mut app, 93, 1); // the last item
        app.place_down = true;
        app.place_prev_down = false;
        update_editing(&mut app, 1.0 / 60.0, SCREEN);
        assert_eq!(app.world.tile(10, 10).unwrap().wall, 4);
        assert_eq!(app.world.tile(11, 10).unwrap().wall, 0, "no fill on the last item");
        assert_eq!(app.inventory[app.selected].count, 0);
    }

    /// `Player.CanPlayerSmashWall` (`Player.cs:45917-45940`) gates the hammer:
    /// a fully enclosed unsafe wall refuses the break; opening a neighbour
    /// lets it through. The unbreakable 350 always refuses.
    #[test]
    fn s1_p_i33_the_hammer_refuses_an_enclosed_unsafe_wall() {
        let mut w = empty_world();
        // Gate V3 fixture: a 5x5 of unsafe dirt wall (the i31 finder's
        // banker's-quadrant 2x2 scan walks to the patch EDGE on a 3x3 and
        // smashes that edge cell instead — `TryFindingWallToHammer`,
        // `Player.cs:45862-45905`). On a 5x5 the cursor (10,10) is interior:
        // every scanned cell's 3x3 stays inside the patch, so the finder
        // keeps the cursor and `CanPlayerSmashWall` refuses the enclosed cell.
        for x in 8..=12 {
            for y in 8..=12 {
                set_wall(&mut w, x, y, 2);
            }
        }
        let mut app = wall_app(w, 10, 10);
        hold_wall(&mut app, 3505, 1); // Copper Hammer (hammer 35)
        app.mine_down = true;
        for _ in 0..120 {
            update_editing(&mut app, 1.0 / 60.0, SCREEN);
        }
        assert_eq!(app.world.tile(10, 10).unwrap().wall, 2, "enclosed unsafe refuses");
        // Open a neighbour of the cursor → the hammer breaks it.
        set_wall(&mut app.world, 9, 9, 0);
        for _ in 0..120 {
            update_editing(&mut app, 1.0 / 60.0, SCREEN);
        }
        assert_eq!(app.world.tile(10, 10).unwrap().wall, 0, "the open edge lets it break");
    }

    /// The hammer's wall cadence is the `PickWall` damage model
    /// (`Player.cs:45912`/`:45945`): `ceil(100 / (hammer * 1.5))` hits at
    /// `useTime / 2` ticks — uniform for every wall (no wall-1 special case).
    #[test]
    fn s1_p_i33_the_hammer_cadence_is_the_damage_model() {
        // Copper Hammer: hammer 35 → 52 per hit → 2 hits; useTime 23 → 11
        // ticks between hits. Gate V3: the i31 chain arms `toolTime = 1` on
        // the press (`ItemCheck_StartActualUse`, `Player.cs:51660-51663`), so
        // the FIRST wall hit lands on tick 2 (not tick 11) and the second one
        // `useTime / 2 = 11` ticks later, on tick 13. A house wall (wood 4)
        // breaks on the second hit (104 >= 100).
        let mut w = empty_world();
        set_wall(&mut w, 10, 10, 4);
        let mut app = wall_app(w, 10, 10);
        hold_wall(&mut app, 3505, 1);
        app.mine_down = true;
        // 12 ticks: the first hit (52) landed, the second (tick 13) has not.
        for _ in 0..12 {
            update_editing(&mut app, 1.0 / 60.0, SCREEN);
        }
        assert_eq!(app.world.tile(10, 10).unwrap().wall, 4, "12 ticks is short");
        update_editing(&mut app, 1.0 / 60.0, SCREEN);
        assert_eq!(app.world.tile(10, 10).unwrap().wall, 0, "the 13th tick breaks");
    }

    /// `break_wall`'s drop is the `KillWall_GetItemDrops` conversion, not a
    /// reverse `createWall` lookup: stone wall 1 drops StoneBlock 26, glass 21
    /// drops 392, and the natural unsafe dirt wall 2 drops nothing.
    #[test]
    fn s1_p_i33_the_break_drop_is_the_conversion_table() {
        for (wall, expect) in [(1u16, Some(26u16)), (21, Some(392)), (2, None)] {
            let mut w = empty_world();
            set_wall(&mut w, 10, 10, wall);
            let mut app = wall_app(w, 10, 10);
            break_wall(&mut app, 10, 10);
            match expect {
                Some(id) => assert!(
                    app.items.iter().any(|d| d.kind == id),
                    "wall {wall} drops {id}"
                ),
                None => assert!(app.items.is_empty(), "wall {wall} drops nothing"),
            }
        }
    }
}
