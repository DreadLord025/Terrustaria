//! The Player `Lighting.AddLight` sources (`S1-E-5`, player family).
//!
//! C# `Lighting.AddLight` funnels every overload into
//! `Lighting.AddLight(int i, int j, float r, float g, float b)`
//! (`Lighting.cs:157-163`), which appends to the active engine's per-frame
//! list behind `!Main.gamePaused && Main.netMode != 2`
//! (`LightingEngine.cs:51-54`). The engine then max-merges that list onto the
//! working light map (`LightingEngine.cs:208-235`), one frame late. There is
//! ONE list pair per engine — [`crate::entity_lights::EntityLights`] — and
//! this family feeds the same pair as the item family
//! (`entity_lights.rs::item_light`) and the NPC family (`npc_lights.rs`).
//!
//! Ported here — **32 of Player.cs's 46 `AddLight` call sites**, every
//! RNG-free one (the remaining 14 are listed under "what stays dropped"):
//!
//! * **Shine (buff 11)** — `UpdateBuffs`' arm (`Player.cs:9993-9996`):
//!   `(int)(position.X + (float)(width / 2)) / 16` — the **int** division
//!   inside the cast, so width 20 folds to exactly 10. The LIVE feed is
//!   S1-E-7a's `entity_lights::shine_light`; this family's arm is a
//!   documented skip (no double-feed), witnessed against the live feed.
//! * **Inferno (116)** `:10215`, **Electrified (144)** `:10316`, **Ice
//!   Barrier (62)** `:10466` — the other three buff-arm lights. Inferno's
//!   and Electrified's RGBs also ride `buff_effects.rs`'s accumulator
//!   (S1-E-7b); that accumulator has no consumer yet, so this module's
//!   `buff_time > 0` scan is the live wiring — when the accumulator's scan
//!   lands, these two arms move to it (the RGBs already agree).
//! * **The Jellyfish Necklace family** — `UpdateEquips`' accessory walk
//!   (`Player.cs:15224-15262`, reached by the slot-3..10
//!   `ApplyEquipFunctional` loop `:12946-12952`): 1303 / 1860 light
//!   (0.225, 0.05, 0.15) dry and (1.8, 0.4, 1.2) wet; 1861 the arctic pair.
//!   `(int)Center.X / 16` truncates **before** dividing
//!   ([`XAnchor::CenterTruncated`]).
//! * **`ItemCheck_EmitHeldItemLight`** (`Player.cs:49810-50194`) — the
//!   held-light family: the biome torch (the `num3` → `num2` torch-id
//!   chain `:49824-49893`, `TorchID.TorchColor` at full amount), the
//!   candles 105/713, the Water/Peace/Shadow candles 148/3117/5322, the
//!   glowsticks 282/286/3112/3002/4776/5643 and the Fairy Queen's magic
//!   item 4952 (holdStyle 6 — its `itemLocation` comes from the
//!   `:50563-50573` arm, not the generic holdStyle-1 anchor). `num` is
//!   the C#'s local `float num = 1f` (`:49812`, never rewritten), so
//!   every `x * num` product is the literal.
//!
//! **What stays dropped, exactly** (see ANALYSIS "Entity lights (S1-E-5)"):
//!
//! * Every dust branch in `ItemCheck_EmitHeldItemLight` draws `Main.rand`
//!   behind `stealth >= 1f` (`:49902`/`:49922`/`:49950`/…); the port skips
//!   the branches wholesale (the dust system is S1-E-2/S1-E-3, other
//!   branches), so no `Main.rand` draw is consumed or skipped mid-stream —
//!   the UpdatePlayers stream order stays exact. The `AddLight` lines
//!   themselves are RNG-free and land.
//! * **4776's `Utils.FloodFillTile` spread** (`:50135`) is not ported —
//!   only the `AddLight(pos2, rgb)` line (`:50136`) lands. FloodFill
//!   writes the light map directly, not the entity list.
//! * **5643's Rainbow Glowstick** reads `Main.DiscoR/G/B` (`:50140-50142`);
//!   the port's `LightParams.disco` carries them.
//! * **The light pets** (Fairy Bell / Wisp in a Bottle / Magic Lantern /
//!   Suspicious Looking Tentacle / …) emit their lights from
//!   `Projectile.AI`, not Player.cs — none of Player.cs's 46 sites is a
//!   pet light; they belong to the projectile family (another worker).
//!   **NightOwl (12)** sets `nightVision` only (`:9997-10000`) — no
//!   `AddLight`, nothing to port.
//! * The remaining **14 dropped sites** (each needs a mount/armor/stealth
//!   system the port does not carry yet): the mount lights (`:21354`,
//!   `:24495-24497` Rudolph's three-tile trail, `:45032`), the armor
//!   lights (`UpdateArmorLights`, the three Vortex-stealth chains
//!   `:31279`/`:31297`/`:31315`), the wing-28/27 lights (`:29547`,
//!   `:36026`), the Snapping Stone trio (`:24575-24583`, gated on
//!   `accSnappingStoneLightUp`), and the phase-blade held light
//!   (`:44916`, needs `Item.GetPhaseColor`). The gemspark `essScale`
//!   rows are WorldItem's (already documented).
//! * `happyFunTorchTime` (the Torch God event, W4-f33) now has a port field
//!   (`p.torch_god.happy_fun_torch_time`): the held-torch light gate
//!   (`Player.cs:49817`) reads it — during the event the held torch emits
//!   no light. `Player.UsingBiomeTorches`
//!   (`Player.cs:3950-3959`) is `unlockedBiomeTorches &&
//!   builderAccStatus[11] == 0`; the port reads
//!   `p.using_biome_torches()` — both halves wired, the builder toggle
//!   flips on the `DrawTorchBiomeSwapIcon` click (W4-g21 batch 5 host 73,
//!   `Main.cs:42585`; the old "toggle folds to 0" deviation retired).
//!   builderAccStatus[11] == 0`; the port reads the property
//!   (`p.using_biome_torches()`, W4-f33 — the 5043 use arms both halves).
//!   The builder-toggle CLICK is unported (`Main.cs:42532-42585`), so the
//!   slot stays at the ctor's 0 (on) in play — documented deviation.

use glam::Vec2;
use tr_core::lighting::LightParams;
use tr_core::scene::Zones;
use tr_core::TILE_PX;

use crate::entity_lights::{tile_x, tile_y, XAnchor};
use crate::player::{Player, WIDTH};

/// The `AddLight` gate of `Player.ItemCheck` (`Player.cs:42847`):
/// `!JustDroppedAnItem && IsAllowedToHoldItems`. `JustDroppedAnItem` is
/// ported now (S1-P-i51: the drop sets it, `Player.cs:5213`, the animation
/// expiry clears it, `:43338-43339` — `p.just_dropped_an_item`);
/// `IsAllowedToHoldItems` is only cleared by the unported webbed/stoned
/// visuals and folds to true — recorded here so the fold is discoverable.
const HELD_LIGHT_ALLOWED: bool = true;

/// `Player.BiomeTorchHoldStyle` (`Player.cs:40291-40346`): the plain torch
/// (8) becomes the biome torch while `UsingBiomeTorches`
/// (`:3950-3959` — `unlockedBiomeTorches && builderAccStatus[11] == 0`;
/// the port reads `p.using_biome_torches()`, the builder toggle wired by
/// the `DrawTorchBiomeSwapIcon` click, W4-g21 batch 5 host 73). The arm
/// order is the C#'s verbatim;
/// `ZoneShimmer` (`:40297`) and `ZoneUndergroundDesert` (`:40337`) read
/// fields the port's `Zones` does not carry and are skipped in place (the
/// remaining arms keep their order). `ZoneLihzhardTemple` (`:40305`) rides
/// the caller passes `p.using_biome_torches()`, W4-f33 — the builder
/// toggle click is unported so the slot stays 0/on, ANALYSIS deviation).
/// The arm order is the C#'s verbatim — ALL twelve arms now live (W4-f33:
/// `ZoneShimmer` `:40297` and `ZoneUndergroundDesert` `:40337` were skipped
/// while the `Zones` snapshot dropped the flags; the snapshot carries them
/// now, `scene.rs::zones`). `ZoneLihzhardTemple` (`:40305`) rides
/// `Zones.temple` and the remix-world desert arm (`:40341`) reads the
/// world's REMIX seed flag (S1-P-i42: both restored — the pre-fix port
/// dropped them, so a temple/remix torch held the wrong colour).
pub(crate) fn biome_torch_hold_style(
    kind: u16,
    using_biome_torches: bool,
    zones: Zones,
    pos_y: f32,
    world: &tr_core::world::World,
) -> u16 {
    // `:40293` — `!UsingBiomeTorches || itemType != 8` returns unchanged.
    if !using_biome_torches || kind != 8 {
        return kind;
    }
    // `:40297-40300` — the shimmer arm comes FIRST (the Aether torch 5353).
    if zones.shimmer {
        return 5353;
    }
    // `:40301-40304` — the dungeon arm comes BEFORE the temple/underworld.
    if zones.dungeon {
        return 3004;
    }
    // `:40305-40308` — `ZoneLihzhardTemple` → the Jungle torch 4388.
    if zones.temple {
        return 4388;
    }
    // `:40309` — below `Main.UnderworldLayer * 16` (`Main.cs:2790`:
    // `maxTilesY - 200`).
    let underworld_px = (world.height - 200) as f32 * TILE_PX;
    if pos_y > underworld_px {
        return 433;
    }
    if zones.glowshroom {
        return 5293; // `:40313-40316`
    }
    if zones.hallow {
        return 4387; // `:40317-40320`
    }
    if zones.corrupt {
        return 4385; // `:40321-40324`
    }
    if zones.crimson {
        return 4386; // `:40325-40328`
    }
    if zones.snow {
        return 974; // `:40329-40332`
    }
    if zones.jungle {
        return 4388; // `:40333-40336`
    }
    // `:40337-40340` — `(ZoneDesert && position.Y < worldSurface * 16.0) ||
    // ZoneUndergroundDesert` (W4-f33: the underground half restored — the
    // `Zones` snapshot carries `underground_desert` now).
    if (zones.desert && pos_y < world.world_surface as f32 * TILE_PX)
        || zones.underground_desert
    {
        return 4383;
    }
    // `:40341-40344` — `ZoneDesert && Main.remixWorld`: the remix seed's
    // inverted world grows its desert below the surface line, so the
    // surface gate above never fires there.
    if zones.desert && world.seeds.contains(tr_core::seeds::SeedFlags::REMIX) {
        return 4383;
    }
    kind
}

/// `Player.BiomeCampfireHoldStyle` (`Player.cs:40403-40458`): the campfire
/// 966's biome conversion under `UsingBiomeTorches` — the cursor icon pass
/// (`DrawInterface_40_InteractItemIcon`, `Main.cs:45053`) reads it. The arm
/// order is the C#'s verbatim; `ZoneShimmer` (`:40409`) and
/// `ZoneUndergroundDesert` (`:40449`) read fields the port's `Zones` does
/// not carry and are skipped in place (the `biome_torch_hold_style`
/// precedent), the remaining arms keep their order.
pub(crate) fn biome_campfire_hold_style(
    kind: u16,
    using_biome_torches: bool,
    zones: Zones,
    pos_y: f32,
    world: &tr_core::world::World,
) -> u16 {
    // `:40405` — `!UsingBiomeTorches || itemType != 966` returns unchanged.
    if !using_biome_torches || kind != 966 {
        return kind;
    }
    // `:40413-40416` — the glowshroom arm comes BEFORE the temple/dungeon
    // (the torch chain's order differs here).
    if zones.glowshroom {
        return 5299;
    }
    // `:40417-40420` — `ZoneLihzhardTemple` → the Jungle campfire 4694.
    if zones.temple {
        return 4694;
    }
    // `:40421-40424` — the dungeon arm.
    if zones.dungeon {
        return 3724;
    }
    // `:40425-40428` — below `Main.UnderworldLayer * 16` (`Main.cs:2790`:
    // `maxTilesY - 200`).
    let underworld_px = (world.height - 200) as f32 * TILE_PX;
    if pos_y > underworld_px {
        return 3047;
    }
    if zones.hallow {
        return 4693; // `:40429-40432`
    }
    if zones.corrupt {
        return 4691; // `:40433-40436`
    }
    if zones.crimson {
        return 4692; // `:40437-40440`
    }
    if zones.snow {
        return 3048; // `:40441-40444`
    }
    if zones.jungle {
        return 4694; // `:40445-40448`
    }
    // `:40449-40452` — the surface-desert half reads the zone flag and the
    // world-surface line like the C#; `ZoneUndergroundDesert` is unported
    // (the `Zones` struct carries no such flag — documented skip).
    if zones.desert && pos_y < world.world_surface as f32 * TILE_PX {
        return 4689;
    }
    // `:40453-40456` — `ZoneDesert && Main.remixWorld`.
    if zones.desert && world.seeds.contains(tr_core::seeds::SeedFlags::REMIX) {
        return 4689;
    }
    kind
}

/// The `num3` → `num2` torch-id chain (`Player.cs:49824-49893`): the
/// biome-torch hold style maps to its `TorchID` row. The final
/// `num3 >= 427` arm covers 427-432 (the six gem torches) as
/// `num3 - 426`; the explicit arms above it shadow the ids they name.
fn torch_id_for_hold_style(style: u16) -> i32 {
    let id = match style {
        523 => 8,
        974 => 9,
        1245 => 10,
        1333 => 11,
        2274 => 12,
        3004 => 13,
        3045 => 14,
        3114 => 15,
        4383 => 16,
        4384 => 17,
        4385 => 18,
        4386 => 19,
        4387 => 20,
        4388 => 21,
        5293 => 22,
        5353 => 23,
        n if n >= 427 => n as i32 - 426,
        _ => 0,
    };
    // `:49892` — `MathHelper.Clamp(num2, 0, TorchID.Count - 1)`.
    id.clamp(0, crate::luck::TORCH_ID_COUNT as i32 - 1)
}

/// The held item's `itemLocation` at **rest** for the lights' arms —
/// `ItemCheck_ApplyHoldStyle` (`Player.cs:50240-50585`). Every lit kind
/// except 4952 has `holdStyle = 1` (Item.cs SetDefaults), so the generic
/// arm (`:50400-50414`) places them — the same anchor the draw layer uses
/// (`render/player_layers.rs::hold_item_placement`). The glowstick
/// nudges (`:50403-50412`) are the C#'s: 282/286/3112/4776/5643 shift
/// `-2·dir, +4`, the Spelunker Glowstick 3002 `-4·dir, +2`.
/// **4952 has `holdStyle = 6`** (Item.cs case 4952): its location comes
/// from the `:50563-50573` arm — `Center.X + 8·dir`,
/// `position.Y + 40 − 2 + mountOffset`, plus the headgear-row offset
/// (`Main.OffsetsPlayerHeadgear[bodyFrame.Y / 56]`, X = 0 always).
/// `frame_w` is the held item's atlas frame width (the draw's
/// `item_size[0]`); `mountOffset` folds to 0 (no mount offsets ported).
fn rest_item_location(p: &Player, kind: u16, frame_w: f32) -> Vec2 {
    let dir = p.facing as f32;
    if kind == 4952 {
        // `:50568-50570` — holdStyle 6.
        let headgear = crate::player::offsets_player_headgear_y(p.body_frame as usize);
        return Vec2::new(
            p.x + WIDTH * 0.5 + 8.0 * dir,
            p.y + 40.0 - 2.0 + headgear,
        );
    }
    let mut x = p.x + WIDTH * 0.5 + (frame_w * 0.5 + 2.0) * dir;
    let mut y = p.y + 24.0;
    match kind {
        282 | 286 | 3112 | 4776 | 5643 => {
            x -= dir * 2.0; // `:50405`
            y += 4.0; // `:50406`
        }
        3002 => {
            x -= 4.0 * dir; // `:50410`
            y += 2.0; // `:50411`
        }
        _ => {}
    }
    Vec2::new(x, y)
}

/// The `AddLight(Vector2, …)` overload's tile form (`Lighting.cs:140-143`):
/// `(int)(v / 16f)` per axis — truncate the quotient, not the pixel.
fn light_tile(v: Vec2) -> (i32, i32) {
    ((v.x / TILE_PX) as i32, (v.y / TILE_PX) as i32)
}

/// The buff-arm lights of `Player.UpdateBuffs` (`Player.cs:9993-10466`),
/// in slot order like the C#'s `for j` scan. Returns the (tile x, tile y,
/// rgb) triples the arms add this tick.
///
/// * Shine 11 (`:9995`): `(int)(position.X + (float)(width / 2)) / 16` —
///   SKIPPED here: S1-E-7a's `entity_lights::shine_light` is the live
///   feed (same anchor form); emitting twice would double-feed.
/// * Inferno 116 (`:10215`) / Ice Barrier 62 (`:10466`):
///   `(int)(Center.X / 16f)` — truncate the quotient.
/// * Electrified 144 (`:10316`): `(int)Center.X / 16` — truncate first.
/// * Ice Barrier's half-life gate (`:10464`): `statLife <= statLifeMax2 *
///   0.5`; the port's single `max_hp` stands in for both caps (the
///   `player.rs:2376` fold).
pub(crate) fn player_buff_lights(p: &Player) -> Vec<(i32, i32, [f32; 3])> {
    let mut out = Vec::new();
    let pos = Vec2::new(p.x, p.y);
    for j in 0..p.buffs.buff_type.len() {
        let bt = p.buffs.buff_type[j];
        if bt == 0 || p.buffs.buff_time[j] <= 0 {
            continue;
        }
        match bt {
            // `:9993-9996` — Shine. The LIVE feed is S1-E-7a's
            // `entity_lights::shine_light` (draw.rs feeds it before this
            // family); emitting it here too would double-feed the same
            // `EntityLights` pair. Skipped in the feed path only — the
            // module's own witness tests still pin the C# int-division
            // anchor form (`width / 2` = 20 → 10, height 42 → 21).
            11 => {}
            // `:10212-10215` — Inferno: the flag arm's light half (the
            // RGB matches `buff_effects.rs::DebuffEffects.inferno_light`).
            116 => {
                let x = tile_x(pos, WIDTH, XAnchor::Center);
                let y = tile_y(pos, p.height, XAnchor::Center);
                out.push((x, y, [0.65, 0.4, 0.1]));
            }
            // `:10313-10316` — Electrified.
            144 => {
                let x = tile_x(pos, WIDTH, XAnchor::CenterTruncated);
                let y = tile_y(pos, p.height, XAnchor::CenterTruncated);
                out.push((x, y, [0.3, 0.8, 1.1]));
            }
            // `:10462-10466` — Ice Barrier, only at half life or below.
            62 => {
                if p.hp as f64 <= p.max_hp as f64 * 0.5 {
                    let x = tile_x(pos, WIDTH, XAnchor::Center);
                    let y = tile_y(pos, p.height, XAnchor::Center);
                    out.push((x, y, [0.1, 0.2, 0.45]));
                }
            }
            _ => {}
        }
    }
    out
}

/// The Jellyfish Necklace family's accessory lights
/// (`Player.UpdateEquips`, `Player.cs:15224-15262`): one light per worn
/// kind, `(int)Center.X / 16` (truncate first), dry/wet colour pairs.
/// The C# reaches `ApplyEquipFunctional` through the **slot-3..10**
/// accessory loop (`:12946-12952`, `IsItemSlotUnlockedAndUsable` folds
/// true like `doll_flags`); the port's `equipped` array is the same
/// `Player.armor` slot order (`world_sim/player.rs:541-543`), so the walk
/// reads `equipped[3..10]` — armor slots 0..2 and the vanity rows never
/// light.
pub(crate) fn player_accessory_lights(p: &Player) -> Vec<(i32, i32, [f32; 3])> {
    let mut out = Vec::new();
    let pos = Vec2::new(p.x, p.y);
    let x = tile_x(pos, WIDTH, XAnchor::CenterTruncated);
    let y = tile_y(pos, p.height, XAnchor::CenterTruncated);
    for &kind in &p.equipped[3..10] {
        let rgb = match kind {
            // `:15224-15234` — the Jellyfish Necklace.
            1303 => {
                if p.wet {
                    [1.8, 0.4, 1.2]
                } else {
                    [0.225, 0.05, 0.15]
                }
            }
            // `:15235-15247` — the Jellyfish Diving Gear (same colours).
            1860 => {
                if p.wet {
                    [1.8, 0.4, 1.2]
                } else {
                    [0.225, 0.05, 0.15]
                }
            }
            // `:15248-15262` — the Arctic Diving Gear (the blue pair).
            1861 => {
                if p.wet {
                    [0.4, 1.2, 1.8]
                } else {
                    [0.05, 0.15, 0.225]
                }
            }
            _ => continue,
        };
        out.push((x, y, rgb));
    }
    out
}

/// `Player.ItemCheck_EmitHeldItemLight` (`Player.cs:49810-50194`): the
/// held item's light contributions this tick, in the C#'s arm order.
///
/// `kind` is the selected item (0/empty = no light), `frame_w` its atlas
/// frame width (the rest anchor's `heldItemFrame.Width`). The dust
/// halves are skipped wholesale (module docs) — no arm here draws RNG.
/// `RotatedRelativePoint` (`Player.cs:4225-4251`) rides the player's own
/// ported method, so the sleeping/grav/mount folds match the draw.
pub(crate) fn held_item_lights(
    p: &Player,
    kind: u16,
    frame_w: f32,
    zones: Zones,
    world: &tr_core::world::World,
    params: &LightParams,
) -> Vec<(i32, i32, [f32; 3])> {
    let mut out = Vec::new();
    if kind == 0 || !HELD_LIGHT_ALLOWED || p.just_dropped_an_item || p.pulley {
        // Every arm below gates on `!this.pulley` (`:49817` etc.); the
        // one exception (4952) gates on it too (`:50174`).
        return out;
    }
    let dir = p.facing as f32;
    let loc = rest_item_location(p, kind, frame_w);
    // `pos` — `ApplyItemPositionOffsetFromMount` (`:49816`): zero while
    // unmounted; the port's mount offset table is unported (folds to 0).
    let pos = Vec2::ZERO;
    let mut emit = |v: Vec2, rgb: [f32; 3]| {
        let (x, y) = light_tile(p.rotated_relative_point(v, 0));
        out.push((x, y, rgb));
    };

    // `:49817-49940` — the torch family: `(Torches[type] && !wet) ||
    // WaterTorches[type]`, `!pulley && !happyFunTorchTime` (W4-f33: the
    // event flag now exists — during the Torch God event the held torch
    // emits no light, the favor burn).
    let torch = crate::entity_lights::TORCHES.contains(&kind);
    let water_torch = crate::entity_lights::WATER_TORCHES.contains(&kind);
    if ((torch && !p.wet) || water_torch) && !p.torch_god.happy_fun_torch_time {
        let style = biome_torch_hold_style(kind, p.using_biome_torches(), zones, p.y, world);
        let id = torch_id_for_hold_style(style);
        let rgb = tr_core::tile_light::torch_color(id, params);
        // `:49918` / `:49938` — the light rides `velocity` (the dust
        // anchor does not); ±12 off the item location, -14 up.
        let off = if dir < 0.0 { -12.0 } else { 12.0 };
        emit(
            Vec2::new(loc.x + off + p.vx, loc.y - 14.0 + p.vy) + pos,
            rgb,
        );
    }

    // `:49941-50069` — the candle else-if chain.
    if (kind == 105 || kind == 713) && !p.wet {
        // `:49961` / `:49976` — the plain candles: -16/+6, (1, .95, .8).
        let off = if dir < 0.0 { -16.0 } else { 6.0 };
        emit(
            Vec2::new(loc.x + off + p.vx, loc.y - 14.0) + pos,
            [1.0, 0.95, 0.8],
        );
    } else if kind == 148 {
        // `:49999` / `:50014` — the Water Candle: (0, .5, 1).
        let off = if dir < 0.0 { -16.0 } else { 6.0 };
        emit(
            Vec2::new(loc.x + off + p.vx, loc.y - 14.0) + pos,
            [0.0, 0.5, 1.0],
        );
    } else if kind == 3117 && !p.wet {
        // `:50017-50054` — the Peace Candle nudges the location itself
        // (`:50019` `itemLocation.X -= direction * 4`); the light rides
        // `velocity.X` (`:50038` / `:50053`) but NOT the mount `+ pos`.
        let loc = Vec2::new(loc.x - dir * 4.0, loc.y);
        let off = if dir < 0.0 { -16.0 } else { 6.0 };
        emit(
            Vec2::new(loc.x + off + p.vx, loc.y - 14.0),
            [0.9, 0.1, 0.75],
        );
    } else if kind == 5322 && !p.wet {
        // `:50056-50068` — the Shadow Candle: (.2, .3, .32); rides
        // `velocity.X`, no mount `+ pos` (`:50063` / `:50067`).
        let off = if dir < 0.0 { -16.0 } else { 6.0 };
        emit(
            Vec2::new(loc.x + off + p.vx, loc.y - 14.0),
            [0.2, 0.3, 0.32],
        );
    }

    // `:50070-50122` — the glowstick `if`s (not else-if: each is its own
    // statement, but the kinds are disjoint so only one fires).
    if kind == 282 {
        // `:50074` / `:50078` — the Glowstick: (.7, 1, .8).
        let off = if dir < 0.0 { -16.0 } else { 6.0 };
        emit(
            Vec2::new(loc.x + off + p.vx, loc.y - 14.0) + pos,
            [0.7, 1.0, 0.8],
        );
    }
    if kind == 3002 {
        // `:50088` / `:50092` — the Spelunker Glowstick: (1.05, .95, .55).
        // The `spelunkerTimer` half (`:50094-50099`) is the ore-spot
        // helper, not a light — unported, no RNG.
        let off = if dir < 0.0 { -16.0 } else { 6.0 };
        emit(
            Vec2::new(loc.x + off + p.vx, loc.y - 14.0) + pos,
            [1.05, 0.95, 0.55],
        );
    }
    if kind == 286 {
        // `:50105` / `:50109` — the Sticky Glowstick: (.7, .8, 1).
        let off = if dir < 0.0 { -16.0 } else { 6.0 };
        emit(
            Vec2::new(loc.x + off + p.vx, loc.y - 14.0) + pos,
            [0.7, 0.8, 1.0],
        );
    }
    if kind == 3112 {
        // `:50116` / `:50120` — the Bouncy Glowstick: (1, .6, .85).
        let off = if dir < 0.0 { -16.0 } else { 6.0 };
        emit(
            Vec2::new(loc.x + off + p.vx, loc.y - 14.0) + pos,
            [1.0, 0.6, 0.85],
        );
    }
    if kind == 4776 {
        // `:50123-50136` — the Fairy Glowstick: (.9, .35, 1). The
        // `FloodFillTile` spread (`:50135`) is dropped (module docs);
        // the `AddLight` line lands at the same rotated point.
        let off = if dir < 0.0 { -16.0 } else { 6.0 };
        emit(
            Vec2::new(loc.x + off + p.vx, loc.y - 14.0) + pos,
            [0.9, 0.35, 1.0],
        );
    }
    if kind == 5643 {
        // `:50138-50150` — the Rainbow Glowstick: `Main.DiscoR/G/B / 255`.
        let off = if dir < 0.0 { -16.0 } else { 6.0 };
        emit(
            Vec2::new(loc.x + off + p.vx, loc.y - 14.0) + pos,
            [
                params.disco[0] as f32 / 255.0,
                params.disco[1] as f32 / 255.0,
                params.disco[2] as f32 / 255.0,
            ],
        );
    }
    // `:50152-50173` — 3542 (the Shellphone's beacon) is dust-only
    // (four `Dust.NewDust` draws, no `AddLight`) — skipped wholesale.
    if kind == 4952 && !p.petting.is_petting {
        // `:50174-50181` — the Fairy Queen's magic item: the light rides
        // `itemLocation + (8·dir, -10·gravDir)` (gravDir = 1 in the port)
        // on top of the holdStyle-6 location (`rest_item_location`),
        // colour (1, .7, .8) × 1.3. W4-g10 FINAL: the `petting.isPetting`
        // gate is LIVE (`:50174` refuses while petting); the `Next(40)`
        // dust half is skipped.
        let v = loc + Vec2::new(8.0 * dir, -10.0) + pos;
        emit(v, [1.0 * 1.3, 0.7 * 1.3, 0.8 * 1.3]);
        // colour (1, .7, .8) × 1.3. W4-d21 fold 2: the `petting.isPetting`
        // gate is live (the pulley gate already rides `p.pulley` upstream);
        // the `Next(40)` dust half is skipped.
        if !p.petting.is_petting {
            let v = loc + Vec2::new(8.0 * dir, -10.0) + pos;
            emit(v, [1.0 * 1.3, 0.7 * 1.3, 0.8 * 1.3]);
        }
    }
    out
}

/// The frame feed's whole player family: buff lights + accessory lights +
/// the held-item lights, in that order. The C# runs the buff scan inside
/// `UpdateBuffs` (`:25566`), the accessory walk inside `UpdateEquips`
/// (`:25633`) and the held light inside `ItemCheck` — all before the draw;
/// the port collects them after the step like the item/NPC feeds. No arm
/// draws RNG, so the placement cannot move any stream.
pub(crate) fn player_lights(
    p: &Player,
    held_kind: u16,
    held_frame_w: f32,
    zones: Zones,
    world: &tr_core::world::World,
    params: &LightParams,
) -> Vec<(i32, i32, [f32; 3])> {
    let mut out = player_buff_lights(p);
    out.extend(player_accessory_lights(p));
    out.extend(held_item_lights(p, held_kind, held_frame_w, zones, world, params));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::world::World;

    fn player() -> Player {
        Player::spawn(&World::new("t".into(), 64, 800))
    }

    fn params() -> LightParams {
        LightParams {
            sky: [1.0; 3],
            hell_glow: 0.55,
            mouse_text_color: 255,
            demon_torch: 0.0,
            disco: [0, 0, 0],
            global_time: 0.0,
            time_for_visuals: 0.0,
            time: 0.0,
            day_time: true,
            raining: false,
            shimmer_darken: 0.0,
            shimmer_alpha: 0.0,
            sundial_cooldown: 0,
            moondial_cooldown: 0,
            cage_mode: [[0u8; tr_core::tile_light::CAGE_FRAMES]; 3],
            rng_seed: 0,
            draw_invisible_walls: false,
        }
    }

    fn zones() -> Zones {
        Zones {
            snow: false,
            desert: false,
            corrupt: false,
            crimson: false,
            hallow: false,
            jungle: false,
            glowshroom: false,
            dungeon: false,
            temple: false,
            shimmer: false,
            underground_desert: false,
        }
    }

    #[test]
    fn shine_is_skipped_here_and_agrees_with_the_live_feed() {
        // `:9995` — `(int)(position.X + (float)(width / 2)) / 16`: the
        // int division makes width 20 fold to 10. At (160, 176):
        // x = (160+10)/16 = 10, y = (176+21)/16 = 12. The LIVE feed is
        // S1-E-7a's `entity_lights::shine_light` — this family skips
        // buff 11 so the pair never double-feeds; pin both the skip and
        // the agreement with the live feed's anchor.
        let mut p = player();
        p.x = 160.0;
        p.y = 176.0;
        p.buffs.buff_type[0] = 11;
        p.buffs.buff_time[0] = 600;
        assert!(player_buff_lights(&p).is_empty(), "no double-feed");
        assert_eq!(
            crate::entity_lights::shine_light(&p),
            Some((10, 12, [0.8, 0.95, 1.0]))
        );
    }

    #[test]
    fn shine_negative_position_truncates_then_divides() {
        // The int form truncates the pixel sum BEFORE dividing: at
        // x = -30: (-30+10) = -20, -20/16 = -1. The live S1-E-7a feed
        // owns the emission; pin its negative tile (this family skips).
        let mut p = player();
        p.x = -30.0;
        p.y = -30.0;
        p.buffs.buff_type[0] = 11;
        p.buffs.buff_time[0] = 600;
        assert!(player_buff_lights(&p).is_empty(), "no double-feed");
        let (x, y, _) = crate::entity_lights::shine_light(&p).expect("Shine emits");
        // x: (-30 + 10) = -20 -> -20/16 = -1; y: (-30 + 21) = -9 -> 0.
        assert_eq!(x, -1);
        assert_eq!(y, 0);
    }

    #[test]
    fn inferno_and_electrified_use_their_own_anchor_forms() {
        // Inferno `:10215` truncates the quotient; Electrified `:10316`
        // truncates the pixel first. At a fractional centre they differ.
        let mut p = player();
        p.x = 150.5; // centre x = 160.5
        p.y = 176.0; // centre y = 197
        p.buffs.buff_type[0] = 116;
        p.buffs.buff_time[0] = 600;
        p.buffs.buff_type[1] = 144;
        p.buffs.buff_time[1] = 600;
        let lights = player_buff_lights(&p);
        assert_eq!(lights.len(), 2);
        // Both land on tile 10 here (160.5/16 = 10.03 -> 10; 160/16 = 10).
        assert_eq!(lights[0], (10, 12, [0.65, 0.4, 0.1]));
        assert_eq!(lights[1], (10, 12, [0.3, 0.8, 1.1]));
        // Both forms truncate toward zero, so they agree on every
        // integer-plus-fraction input ((int)(x/16) == (int)x/16 for all
        // x — the fractional part can never cross a 16-px boundary).
        // The port keeps each arm's own C# form anyway; pin the tiles.
        p.x = 5.9; // centre x = 15.9
        let lights = player_buff_lights(&p);
        // Inferno: (int)(15.9/16) = 0; Electrified: (int)15.9/16 = 15/16 = 0.
        assert_eq!(lights[0].0, 0);
        assert_eq!(lights[1].0, 0);
    }

    #[test]
    fn ice_barrier_only_lights_at_half_life() {
        let mut p = player();
        p.x = 160.0;
        p.y = 176.0;
        p.buffs.buff_type[0] = 62;
        p.buffs.buff_time[0] = 600;
        p.max_hp = 200;
        p.hp = 150; // above half — the C# would DelBuff; no light.
        assert!(player_buff_lights(&p).is_empty());
        p.hp = 100; // exactly half — `<=` gates it on.
        let lights = player_buff_lights(&p);
        assert_eq!(lights, vec![(10, 12, [0.1, 0.2, 0.45])]);
    }

    #[test]
    fn night_owl_emits_no_light() {
        // `:9997-10000` — NightOwl sets nightVision only.
        let mut p = player();
        p.buffs.buff_type[0] = 12;
        p.buffs.buff_time[0] = 600;
        assert!(player_buff_lights(&p).is_empty());
    }

    #[test]
    fn jellyfish_necklace_dry_and_wet_colours() {
        let mut p = player();
        p.x = 160.0;
        p.y = 176.0;
        p.equipped[3] = 1303;
        p.wet = false;
        let lights = player_accessory_lights(&p);
        // Centre (170, 197): (int)170/16 = 10, (int)197/16 = 12.
        assert_eq!(lights, vec![(10, 12, [0.225, 0.05, 0.15])]);
        p.wet = true;
        let lights = player_accessory_lights(&p);
        assert_eq!(lights, vec![(10, 12, [1.8, 0.4, 1.2])]);
        p.equipped[3] = 1861; // the arctic pair.
        let lights = player_accessory_lights(&p);
        assert_eq!(lights, vec![(10, 12, [0.4, 1.2, 1.8])]);
        p.wet = false;
        let lights = player_accessory_lights(&p);
        assert_eq!(lights, vec![(10, 12, [0.05, 0.15, 0.225])]);
    }

    #[test]
    fn only_the_accessory_slots_3_to_9_light() {
        // The C# reaches the arms through the slot-3..10 loop
        // (`:12946-12952`) — a necklace kind in an armor slot (0..2) or a
        // later row never lights.
        let mut p = player();
        p.x = 160.0;
        p.y = 176.0;
        p.equipped[0] = 1303;
        p.equipped[10] = 1303;
        assert!(player_accessory_lights(&p).is_empty());
        p.equipped[9] = 1303;
        assert_eq!(player_accessory_lights(&p).len(), 1);
    }

    #[test]
    fn held_torch_lights_the_biome_colour_at_the_hand() {
        // A plain torch (8) in no biome zone: torch id 0, the plain
        // torch colour. Facing right, frame 20 wide: loc.x = x + 10 +
        // (10+2) = x + 22; the light rides +12 + vx, -14 + vy.
        let mut p = player();
        p.x = 160.0;
        p.y = 176.0;
        p.facing = 1;
        p.vx = 0.0;
        p.vy = 0.0;
        let w = World::new("t".into(), 64, 800);
        let lights = held_item_lights(&p, 8, 20.0, zones(), &w, &params());
        assert_eq!(lights.len(), 1);
        // loc = (182, 200); light point = (194, 186) -> tile (12, 11).
        assert_eq!((lights[0].0, lights[0].1), (12, 11));
        assert_eq!(lights[0].2, tr_core::tile_light::torch_color(0, &params()));
    }

    #[test]
    fn a_wet_plain_torch_goes_dark_but_the_cursed_torch_stays_lit() {
        let mut p = player();
        p.wet = true;
        let w = World::new("t".into(), 64, 800);
        assert!(held_item_lights(&p, 8, 20.0, zones(), &w, &params()).is_empty());
        // 523 is a WaterTorches row — it lights under water.
        let lights = held_item_lights(&p, 523, 20.0, zones(), &w, &params());
        assert_eq!(lights.len(), 1);
        assert_eq!(lights[0].2, tr_core::tile_light::torch_color(8, &params()));
    }

    #[test]
    fn the_biome_torch_substitution_follows_the_zone_order() {
        let mut p = player();
        p.x = 160.0;
        p.y = 176.0;
        p.unlocked_biome_torches = true;
        let w = World::new("t".into(), 64, 800);
        let mut z = zones();
        z.snow = true;
        let lights = held_item_lights(&p, 8, 20.0, z, &w, &params());
        // Snow → 974 → torch id 9 (the Ice torch).
        assert_eq!(lights[0].2, tr_core::tile_light::torch_color(9, &params()));
        // Hallow shadows snow (the C# order: hallow before snow).
        z.hallow = true;
        let lights = held_item_lights(&p, 8, 20.0, z, &w, &params());
        assert_eq!(lights[0].2, tr_core::tile_light::torch_color(20, &params()));
        // Below the underworld line the Demon torch wins over everything
        // except the dungeon (`:40301` comes before `:40309`).
        p.y = (w.height - 100) as f32 * TILE_PX;
        let lights = held_item_lights(&p, 8, 20.0, z, &w, &params());
        assert_eq!(lights[0].2, tr_core::tile_light::torch_color(7, &params()));
        // The dungeon arm (`:40301-40304`) shadows even the underworld.
        z.dungeon = true;
        let lights = held_item_lights(&p, 8, 20.0, z, &w, &params());
        assert_eq!(lights[0].2, tr_core::tile_light::torch_color(13, &params()));
    }

    /// W4-f33 — the RESTORED hold arms: shimmer (`:40297-40300`, the Aether
    /// torch 5353 → id 23) shadows everything; the underground-desert half
    /// of `:40337` converts below the surface line without the remix seed.
    /// WRITTEN NOT RUN (code-only).
    #[test]
    fn w4_f33_the_shimmer_and_underground_desert_hold_arms() {
        let mut p = player();
        p.x = 160.0;
        p.y = 176.0;
        p.unlocked_biome_torches = true;
        let w = World::new("t".into(), 64, 800);
        // Shimmer FIRST: it shadows the dungeon + snow arms.
        let mut z = zones();
        z.shimmer = true;
        z.dungeon = true;
        z.snow = true;
        let lights = held_item_lights(&p, 8, 20.0, z, &w, &params());
        assert_eq!(lights[0].2, tr_core::tile_light::torch_color(23, &params()));
        // Underground desert: below the surface line, no remix, no snow —
        // the `|| ZoneUndergroundDesert` half fires (`:40337`).
        let mut z = zones();
        z.underground_desert = true;
        p.y = (w.world_surface as f32 + 10.0) * TILE_PX;
        let lights = held_item_lights(&p, 8, 20.0, z, &w, &params());
        assert_eq!(lights[0].2, tr_core::tile_light::torch_color(16, &params()));
        // The jungle arm (`:40333-40336`) → 4388 → id 21.
        let mut z = zones();
        z.jungle = true;
        p.y = 176.0;
        let lights = held_item_lights(&p, 8, 20.0, z, &w, &params());
        assert_eq!(lights[0].2, tr_core::tile_light::torch_color(21, &params()));
        // The temple arm (`:40305-40308`) shares the jungle torch 4388.
        let mut z = zones();
        z.temple = true;
        let lights = held_item_lights(&p, 8, 20.0, z, &w, &params());
        assert_eq!(lights[0].2, tr_core::tile_light::torch_color(21, &params()));
        // The glowshroom arm (`:40313-40316`) → 5293 → id 22.
        let mut z = zones();
        z.glowshroom = true;
        let lights = held_item_lights(&p, 8, 20.0, z, &w, &params());
        assert_eq!(lights[0].2, tr_core::tile_light::torch_color(22, &params()));
        // The corrupt/crimson pair (`:40321-40328`) → 4385/4386 → 18/19.
        let mut z = zones();
        z.corrupt = true;
        let lights = held_item_lights(&p, 8, 20.0, z, &w, &params());
        assert_eq!(lights[0].2, tr_core::tile_light::torch_color(18, &params()));
        let mut z = zones();
        z.crimson = true;
        let lights = held_item_lights(&p, 8, 20.0, z, &w, &params());
        assert_eq!(lights[0].2, tr_core::tile_light::torch_color(19, &params()));
    }

    #[test]
    fn the_biome_torch_needs_the_torch_god_unlock() {
        // `UsingBiomeTorches` (`:3950-3959`) reads `unlockedBiomeTorches`
        // first — without the unlock the plain torch stays plain even in
        // a biome zone (`:40293` returns the item type unchanged).
        let mut p = player();
        p.x = 160.0;
        p.y = 176.0;
        p.unlocked_biome_torches = false;
        let w = World::new("t".into(), 64, 800);
        let mut z = zones();
        z.snow = true;
        let lights = held_item_lights(&p, 8, 20.0, z, &w, &params());
        assert_eq!(lights[0].2, tr_core::tile_light::torch_color(0, &params()));
    }

    /// W4-f33 — the builder-toggle half of `UsingBiomeTorches`
    /// (`Player.cs:3958`): the unlock alone is not enough —
    /// `builderAccStatus[11] == 1` (the toggle OFF) keeps the plain torch
    /// plain even with the favor used. The 5043 use sets the slot to 0
    /// (`:45232` → the setter `:3960-3963`); the toggle click
    /// (`Main.cs:42532-42585`) is unported, so this pins the property's
    /// gate shape for when it lands. WRITTEN NOT RUN (code-only).
    #[test]
    fn w4_f33_the_builder_toggle_gates_the_hold_conversion() {
        let mut p = player();
        p.x = 160.0;
        p.y = 176.0;
        p.unlocked_biome_torches = true;
        let w = World::new("t".into(), 64, 800);
        let mut z = zones();
        z.snow = true;
        // The ctor slot 0 (on): the snow conversion fires.
        assert!(p.using_biome_torches());
        let lights = held_item_lights(&p, 8, 20.0, z, &w, &params());
        assert_eq!(lights[0].2, tr_core::tile_light::torch_color(9, &params()));
        // Toggle OFF (`builderAccStatus[11] = 1`): the property reads
        // false (`:3958`) and the torch stays plain (`:40293`).
        p.builder_acc_status[11] = 1;
        assert!(!p.using_biome_torches());
        let lights = held_item_lights(&p, 8, 20.0, z, &w, &params());
        assert_eq!(lights[0].2, tr_core::tile_light::torch_color(0, &params()));
        // The setter pair (`:3960-3963`): true → 0, false → 1.
        p.set_using_biome_torches(true);
        assert_eq!(p.builder_acc_status[11], 0);
        p.set_using_biome_torches(false);
        assert_eq!(p.builder_acc_status[11], 1);
        // The unlock gates the property regardless of the slot.
        p.builder_acc_status[11] = 0;
        p.unlocked_biome_torches = false;
        assert!(!p.using_biome_torches());
    }

    #[test]
    fn w4_f33_the_torch_god_event_kills_the_held_torch_light() {
        // `Player.cs:49817` — `!happyFunTorchTime`: during the Torch God
        // event the held torch (plain OR water) emits no light. The candle
        // family is unaffected (the gate is torch-only).
        let mut p = player();
        p.x = 160.0;
        p.y = 176.0;
        let w = World::new("t".into(), 64, 800);
        // Baseline: the plain torch lights.
        assert_eq!(held_item_lights(&p, 8, 20.0, zones(), &w, &params()).len(), 1);
        // The event runs → the torch goes dark.
        p.torch_god.happy_fun_torch_time = true;
        assert!(held_item_lights(&p, 8, 20.0, zones(), &w, &params()).is_empty());
        // The water torch (523) too — the gate covers the whole family.
        assert!(held_item_lights(&p, 523, 20.0, zones(), &w, &params()).is_empty());
        // The candle (105) keeps its light — the gate is torch-only.
        assert_eq!(held_item_lights(&p, 105, 20.0, zones(), &w, &params()).len(), 1);
        // The event ends → the torch lights again.
        p.torch_god.happy_fun_torch_time = false;
        assert_eq!(held_item_lights(&p, 8, 20.0, zones(), &w, &params()).len(), 1);
    /// W4-g79 — `BiomeCampfireHoldStyle` (`Player.cs:40403-40458`): the
    /// campfire 966's zone ladder, the glowshroom-before-temple order (the
    /// torch chain differs), the underworld line and the unlock gate.
    #[test]
    fn w4_g79_the_biome_campfire_substitution_follows_the_zone_order() {
        let w = World::new("t".into(), 64, 800);
        let mut z = zones();
        // No unlock: the plain campfire stays plain even in a biome.
        assert_eq!(biome_campfire_hold_style(966, false, z, 176.0, &w), 966);
        // A non-campfire kind never converts.
        assert_eq!(biome_campfire_hold_style(8, true, z, 176.0, &w), 8);
        // Snow → 3048 (the Snow campfire).
        z.snow = true;
        assert_eq!(biome_campfire_hold_style(966, true, z, 176.0, &w), 3048);
        // Hallow shadows snow (`:40429` before `:40441`).
        z.hallow = true;
        assert_eq!(biome_campfire_hold_style(966, true, z, 176.0, &w), 4693);
        // The glowshroom arm (`:40413`) shadows even the hallow.
        z.glowshroom = true;
        assert_eq!(biome_campfire_hold_style(966, true, z, 176.0, &w), 5299);
        // Below the underworld line the Demon campfire 3047 wins over the
        // hallow/snow but not the glowshroom/temple/dungeon above it.
        let deep_y = (w.height - 100) as f32 * TILE_PX;
        z.glowshroom = false;
        assert_eq!(biome_campfire_hold_style(966, true, z, deep_y, &w), 3047);
        // The dungeon arm (`:40421`) shadows the underworld.
        z.dungeon = true;
        assert_eq!(biome_campfire_hold_style(966, true, z, deep_y, &w), 3724);
        // The temple arm (`:40417`) shadows the dungeon.
        z.temple = true;
        assert_eq!(biome_campfire_hold_style(966, true, z, deep_y, &w), 4694);
    }

    #[test]
    fn the_glowstick_family_colours_and_nudges() {
        let mut p = player();
        p.x = 160.0;
        p.y = 176.0;
        p.facing = 1;
        let w = World::new("t".into(), 64, 800);
        // 282: loc = (x+22-2, y+24+4) = (180, 204); light +6, -14 →
        // (186, 190) → tile (11, 11), colour (.7, 1, .8).
        let lights = held_item_lights(&p, 282, 20.0, zones(), &w, &params());
        assert_eq!(lights, vec![(11, 11, [0.7, 1.0, 0.8])]);
        // 3002: loc = (x+22-4, y+24+2) = (178, 202); light (184, 188) →
        // tile (11, 11), colour (1.05, .95, .55).
        let lights = held_item_lights(&p, 3002, 20.0, zones(), &w, &params());
        assert_eq!(lights, vec![(11, 11, [1.05, 0.95, 0.55])]);
        // 286 / 3112 share the 282 nudge.
        let lights = held_item_lights(&p, 286, 20.0, zones(), &w, &params());
        assert_eq!(lights[0].2, [0.7, 0.8, 1.0]);
        let lights = held_item_lights(&p, 3112, 20.0, zones(), &w, &params());
        assert_eq!(lights[0].2, [1.0, 0.6, 0.85]);
    }

    #[test]
    fn the_candles_gate_on_wet_and_the_peace_candle_nudges_the_location() {
        let mut p = player();
        p.x = 160.0;
        p.y = 176.0;
        p.facing = 1;
        let w = World::new("t".into(), 64, 800);
        // 105 dry: loc (182, 200), light (188, 186) → (11, 11).
        let lights = held_item_lights(&p, 105, 20.0, zones(), &w, &params());
        assert_eq!(lights, vec![(11, 11, [1.0, 0.95, 0.8])]);
        p.wet = true;
        assert!(held_item_lights(&p, 105, 20.0, zones(), &w, &params()).is_empty());
        // 148 (Water Candle) lights even wet.
        let lights = held_item_lights(&p, 148, 20.0, zones(), &w, &params());
        assert_eq!(lights[0].2, [0.0, 0.5, 1.0]);
        p.wet = false;
        // 3117 (Peace Candle): loc.x -= 4 → (178, 200), light (184, 186)
        // → (11, 11), colour (.9, .1, .75).
        let lights = held_item_lights(&p, 3117, 20.0, zones(), &w, &params());
        assert_eq!(lights, vec![(11, 11, [0.9, 0.1, 0.75])]);
        // 5322 (Shadow Candle): (.2, .3, .32).
        let lights = held_item_lights(&p, 5322, 20.0, zones(), &w, &params());
        assert_eq!(lights[0].2, [0.2, 0.3, 0.32]);
    }

    #[test]
    fn the_rainbow_glowstick_reads_the_disco_colour() {
        let mut p = player();
        p.x = 160.0;
        p.y = 176.0;
        let w = World::new("t".into(), 64, 800);
        let mut pr = params();
        pr.disco = [255, 128, 0];
        let lights = held_item_lights(&p, 5643, 20.0, zones(), &w, &pr);
        assert_eq!(lights[0].2, [1.0, 128.0 / 255.0, 0.0]);
    }

    #[test]
    fn the_fairy_queen_item_lights_at_the_gravity_offset() {
        // `:50178-50181` — loc + (8·dir, -10), colour ×1.3. The location
        // is the holdStyle-6 arm (`:50568-50570`): Center.X + 8·dir,
        // y + 40 − 2 + headgear (row 0 → +2).
        let mut p = player();
        p.x = 160.0;
        p.y = 176.0;
        p.facing = 1;
        let w = World::new("t".into(), 64, 800);
        let lights = held_item_lights(&p, 4952, 20.0, zones(), &w, &params());
        assert_eq!(lights.len(), 1);
        // loc = (170 + 8, 176 + 38 + 2) = (178, 216); + (8, -10) =
        // (186, 206) → tile (11, 12).
        assert_eq!((lights[0].0, lights[0].1), (11, 12));
        assert_eq!(lights[0].2, [1.3, 0.7 * 1.3, 0.8 * 1.3]);
    }

    #[test]
    fn facing_left_mirrors_the_light_offset() {
        let mut p = player();
        p.x = 160.0;
        p.y = 176.0;
        p.facing = -1;
        let w = World::new("t".into(), 64, 800);
        // Torch: loc.x = 160 + 10 - 12 = 158; light -12 → 146, y 186.
        let lights = held_item_lights(&p, 8, 20.0, zones(), &w, &params());
        assert_eq!((lights[0].0, lights[0].1), (9, 11));
        // Glowstick 282: loc.x = 158 + 2 = 160; light -16 → 144 → tile 9.
        let lights = held_item_lights(&p, 282, 20.0, zones(), &w, &params());
        assert_eq!(lights[0].0, 9);
    }

    #[test]
    fn the_pulley_and_empty_hand_emit_nothing() {
        let mut p = player();
        let w = World::new("t".into(), 64, 800);
        assert!(held_item_lights(&p, 0, 20.0, zones(), &w, &params()).is_empty());
        p.pulley = true;
        assert!(held_item_lights(&p, 8, 20.0, zones(), &w, &params()).is_empty());
        assert!(held_item_lights(&p, 4952, 20.0, zones(), &w, &params()).is_empty());
    }

    /// W4-d21 fold 2 — the petting gate on the 4952 light (`:50174`):
    /// while petting, the Fairy Queen item's light refuses; the other
    /// held lights keep burning (their gates read pulley only).
    #[test]
    fn w4_d21_the_petting_gate_refuses_the_fairy_queen_light() {
        let mut p = player();
        let w = World::new("t".into(), 64, 800);
        assert!(!held_item_lights(&p, 4952, 20.0, zones(), &w, &params()).is_empty());
        p.petting.is_petting = true;
        assert!(
            held_item_lights(&p, 4952, 20.0, zones(), &w, &params()).is_empty(),
            ":50174 — the petting refusal"
        );
        assert!(
            !held_item_lights(&p, 8, 20.0, zones(), &w, &params()).is_empty(),
            "the torch light has no petting gate"
        );
    }

    #[test]
    fn the_torch_id_chain_matches_the_csharp_arms() {
        // Every explicit arm of `:49824-49891` plus the `>= 427` tail.
        assert_eq!(torch_id_for_hold_style(523), 8);
        assert_eq!(torch_id_for_hold_style(974), 9);
        assert_eq!(torch_id_for_hold_style(1245), 10);
        assert_eq!(torch_id_for_hold_style(1333), 11);
        assert_eq!(torch_id_for_hold_style(2274), 12);
        assert_eq!(torch_id_for_hold_style(3004), 13);
        assert_eq!(torch_id_for_hold_style(3045), 14);
        assert_eq!(torch_id_for_hold_style(3114), 15);
        assert_eq!(torch_id_for_hold_style(4383), 16);
        assert_eq!(torch_id_for_hold_style(4384), 17);
        assert_eq!(torch_id_for_hold_style(4385), 18);
        assert_eq!(torch_id_for_hold_style(4386), 19);
        assert_eq!(torch_id_for_hold_style(4387), 20);
        assert_eq!(torch_id_for_hold_style(4388), 21);
        assert_eq!(torch_id_for_hold_style(5293), 22);
        assert_eq!(torch_id_for_hold_style(5353), 23);
        assert_eq!(torch_id_for_hold_style(427), 1);
        assert_eq!(torch_id_for_hold_style(432), 6);
        assert_eq!(torch_id_for_hold_style(8), 0);
        // The clamp (`:49892`): an out-of-range tail id clamps to 23.
        assert_eq!(torch_id_for_hold_style(4389), 23);
    }
}
