//! The NPC `Lighting.AddLight` sources, SECOND half (W4-e7, S1-E-5 wave 4).
//!
//! `NPC.cs` carries exactly 80 `Lighting.AddLight` call sites. The first 40
//! (`:21307`–`:48641`) ride `task/W4-c20-addlight-npcs-tiles`; this module
//! ports the SECOND 40 (`:49929`–`:92412`), verbatim, draw-side, feeding the
//! SAME [`crate::entity_lights::EntityLights`] pair as the aiStyle-8 tail
//! (`npc_lights.rs`) and every other family (one list pair per engine,
//! `LightingEngine.cs:36-38`, max-merged one frame late `:208-235`).
//!
//! ## The 40 sites, in C# source order
//!
//! | # | site | arm | port |
//! |---|------|-----|------|
//! | 41 | `:49929` | Destroyer family airborne light | [`destroyer_light`] |
//! | 42 | `:50630` | Probe 139 tile-light probe | [`probe_light`] |
//! | 43 | `:52428` | The Hungry II 116 | cited skip (no kind — see below) |
//! | 44 | `:53232` | town shimmer state 25 | [`town_shimmer_light`] |
//! | 45 | `:54984` | town magic-attack aura | [`town_magic_light`] |
//! | 46 | `:55131` | town shimmer-arrival pose | [`town_arrival_light`] |
//! | 47 | `:55554` | TownSlimeRainbow 681 | [`town_slime_light`] |
//! | 48 | `:55559` | TownSlimeYellow 683/687 | [`town_slime_light`] |
//! | 49 | `:56420` | Medusa 480 charge-up | [`medusa_light`] |
//! | 50 | `:56427` | Medusa 480 charge hold | [`medusa_light`] |
//! | 51 | `:56925` | TorchZombie 590 (left) | [`torch_zombie_light`] |
//! | 52 | `:56939` | TorchZombie 590 (right) | [`torch_zombie_light`] |
//! | 53 | `:56955` | ArmedTorchZombie 591 (left) | [`armed_torch_zombie_light`] |
//! | 54 | `:56963` | ArmedTorchZombie 591 (right) | [`armed_torch_zombie_light`] |
//! | 55 | `:58404` | hell Armored Bones 277-280 | [`fighter_tail_light`] |
//! | 56 | `:58408` | MartianWalker 520 | [`fighter_tail_light`] |
//! | 57 | `:58412` | DesertGhoulCorruption 525 | [`fighter_tail_light`] |
//! | 58 | `:58416` | DesertGhoulCrimson 526 | [`fighter_tail_light`] |
//! | 59 | `:58420` | DesertGhoulHallow 527 | [`fighter_tail_light`] |
//! | 60 | `:60346` | slime 1 lantern content 75 | [`slime_lights`] |
//! | 61 | `:60760` | ore-slime content tint | cited skip (content init deferred) |
//! | 62 | `:60896` | slime herb content 314 | cited skip (content init deferred) |
//! | 63 | `:61057` | RainbowSlime 244 disco | [`slime_lights`] |
//! | 64 | `:61112` | GoldenSlime 667 | [`slime_lights`] |
//! | 65 | `:61130` | ShimmerSlime 676 torch 23 | [`slime_lights`] |
//! | 66 | `:61454` | LavaSlime 59 | [`slime_lights`] |
//! | 67 | `:62420` | HeadlessHorseman gallop | landed W-10 (`npc_ai_unicorn_b::horseman_light`) — no double-feed |
//! | 68 | `:90841` | trail mode 4 (120/137/138) | [`trail_light`] |
//! | 69 | `:91247` | OnFire! 24 | [`buff_vfx_lights`] |
//! | 70 | `:91264` | Hellfire 323 | [`buff_vfx_lights`] |
//! | 71 | `:91280` | Daybreak 189 | [`buff_vfx_lights`] |
//! | 72 | `:91295` | Betsy's Curse 203 | [`buff_vfx_lights`] |
//! | 73 | `:91480` | Frostburn 44 | [`buff_vfx_lights`] |
//! | 74 | `:91496` | Frostfire 324 | [`buff_vfx_lights`] |
//! | 75 | `:91512` | Cursed Inferno 39 | [`buff_vfx_lights`] |
//! | 76 | `:92382` | Truffle 160 cast light | cited skip (`Main.rand.Next(28,42)` per tick) |
//! | 77 | `:92395` | mushroom towns 254-261/634/635 | cited skip (same rand draw) |
//! | 78 | `:92404` | Cyborg 209 cast light | cited skip (same rand draw) |
//! | 79 | `:92407` | Cyborg 209 rocket pose | [`cast_lights`] |
//! | 80 | `:92412` | UndeadMiner 44 helmet lamp | [`cast_lights`] |
//!
//! ## RNG census — why the feed is draw-side
//!
//! Every ported arm's `AddLight` line is itself RNG-free. The rand-bearing
//! halves are skipped WHOLESALE (the `npc_lights.rs` precedent): the
//! shimmer-state dust (`:53231`/`:53233`/`:53237`), Medusa's charge dust
//! (`:56409-56417`, `:56442-56455`), the torch zombies' `Next(30)` dust
//! (`:56926`/`:56940`/`:56956`/`:56964`) and the slime dust gates
//! (`:60347`, `:60892`, `:61113`, `:61131`, `:61455`), and every
//! `UpdateNPC_BuffApplyVFX` dust arm (`:91150-91511`). Skipping an arm
//! wholesale leaves the `Main.rand` stream untouched, so the fixed-step
//! order stays exact. The three `UpdateNPC_CastLights` flicker arms
//! (160/254-261+634/635/209) draw `Main.rand.Next(28, 42)` UNCONDITIONALLY
//! every tick and cannot be reconstructed without a sim-side feed — they
//! stay cited skips (the `gold_worm_sparkle` precedent,
//! `npc_ai_style66.rs`, for the future wiring). The gold-critter sparkle
//! (`:92345-92368`) has no `AddLight`.
//!
//! ## netOffset conventions (verbatim per site)
//!
//! The C# adds `netOffset` only inside the arms that say so — Medusa
//! (`:56386`/`:56421`, `:56426`/`:56457`), the torch zombies
//! (`:56916`/`:56946`, `:56950`/`:56970`) and the LavaSlime 59
//! (`:61453`/`:61457`). The Destroyer (`AI_037` has ZERO netOffset
//! writes), the Probe's `:50630` (the arm's wraps are elsewhere), the
//! Hungry II, the town arms (`AI_007` has ZERO), the fighter tail
//! (`:58402-58421` unwrapped) and the slime arms except 59 read the RAW
//! position.
//! * `UpdateNPC_BuffApplyVFX` adds it once at the head (`:91149`).
//! * `UpdateNPC_CastLights`' gold block adds/removes it around itself
//!   only (`:92347`/`:92367`) — the 209/44 arms read the RAW position.
//! * `UpdateNPC_UpdateTrails`' mode-4 light reads the RAW position
//!   (`:90841` — only `oldPos[0]` gets the offset, `:90843`).
//!
//! ## Folds and cited skips (ANALYSIS "The NPC lights, second half")
//!
//! * The Hungry II 116 (`:52428`) has NO port kind: the roster's `Hungry`
//!   is TheHungry 115 (aiStyle 29, 30×30/240 HP/kb 1.1, `NPC.cs:10378-
//!   10392`; `world_sim/npcs.rs:6017` maps the WoF spawn's net 115 onto
//!   it) — the II is a SEPARATE 30×32/80 HP/aiStyle-2 kind
//!   (`NPC.cs:10393-10404`, `npc_ai_eye.rs`'s census: "116 … have no
//!   kinds yet"). Folding 116 onto `Hungry` would light the wrong NPC —
//!   the arm stays a cited skip until the kind lands (NO `cs_type` row:
//!   the `Hungry` fallback resolves to the bestiary's 115, which the
//!   `type == 116` gate can never match).
//! * The town shimmer state 25 is a cited skip in `npc_ai_town.rs`
//!   (`:53187-53277`); its light is ported here behind the same state
//!   shape, unreachable until that wiring. The `AddLight` line fires for
//!   EVERY state-25 tick that reaches it — the `ai[2] > 0` countdown
//!   returns first (`:53206-53214`), and below `ai[1] == 30` the
//!   `Utils.Remap` clamps to 0 (a zero-amount source still enters the
//!   C#'s per-frame list).
//! * The town magic-attack (14) / shimmer-arrival (24) arms are gated on
//!   the aiStyle-7 dispatch (`NPC.cs:20812-20816`): `ai[0]` states are
//!   per-machine, so a non-town machine's state 14 must never light.
//!   Only the five `MagicAuraColor` rows (54/108/160/20/663) carry a
//!   colour; every other type's aura is the set's `Color.White` default
//!   (`NPCID.cs:10475`), but only those five reach the attack arms
//!   (`NPCID.Sets.AttackType == 2`, `:10473`) and only the Dryad 20 the
//!   arrival pose (`HaveDryadDoStardewAnimation`, `:19184-19194`) — the
//!   `None` fold for unrowed types is unreachable, recorded here.
//! * The slime content lights 75/ore/314 (`:60346`/`:60760`/`:60896`)
//!   need `SlimeCanContainItems`' init (`npc_ai_slime.rs`'s deferred
//!   A1-5 slice — `ai[1]` never holds a content id yet). The 75 arm is
//!   ported behind its gate (unreachable until the init lands, the
//!   module's own census); the ore arm additionally needs `NPC.color`
//!   (no port field) and the herb arm its content gate — both cited.
//! * `canDisplayBuffs` (`NPC.cs:6199`) is cleared ONLY by the Golem
//!   245/246/247/248 rows inside `getGoodAdjustments` (`:18010`/`:18015`/
//!   `:18020`), which runs only under `Main.getGoodWorld` (`:17843`). The
//!   port's machines run stock (`get_good_world: false`), so the flag
//!   folds to TRUE — but it IS read at `:91145` (the BuffApplyVFX gate),
//!   kept explicit in [`can_display_buffs`] for the future FTW wiring.
//! * TrailingMode 4's nine duplicate lights per tick (`:90838-90842`
//!   loops `oldPos.Length - 1 = 9` times over the SAME raw-position
//!   tile) are kept verbatim — the list is max-merged, but the count is
//!   part of the C#'s per-frame list.

use tr_core::lighting::LightParams;
use tr_core::tile_light::hsl_to_rgb;
use tr_core::world::World;

use crate::entity_lights::{tile_x, tile_y, XAnchor};
use crate::npc::{Npc, NpcKind};

/// One `AddLight` source: tile x, tile y, RGB (the `EntityLights::add`
/// triple).
pub(crate) type Light = (i32, i32, [f32; 3]);

/// The C# `this.type` for the light arms: the hand table (`cs_id`) plus
/// the folded-se bestiary ids the light sites read — the Probe 139
/// (`NPCID.cs:10955`; the bestiary folds it onto the Destroyer head,
/// `npc_ai_style37.rs`'s `family_type` precedent), the Golem parts
/// 246/247/248 (`NPCID.cs`' GolemHead/Fists; the bestiary folds them onto
/// 245). NO Hungry row: the roster's `Hungry` is TheHungry 115 and the
/// 116 light arm is a cited skip (module docs).
fn cs_type(n: &Npc) -> i32 {
    match n.kind {
        NpcKind::Probe => 139,
        NpcKind::GolemHead => 246,
        NpcKind::GolemFistLeft => 247,
        NpcKind::GolemFistRight => 248,
        NpcKind::GolemHeadFree => 249,
        NpcKind::TheDestroyerBody => 135,
        NpcKind::TheDestroyerTail => 136,
        _ => n.kind.cs_id() as i32,
    }
}

/// `WorldGen.InWorld(x, y)` (`WorldGen.cs:9688-9695`, fluff 0).
fn in_world(world: &World, x: i32, y: i32) -> bool {
    x >= 0 && x < world.width && y >= 0 && y < world.height
}

/// `WorldGen.SolidTile(i, j)` (`WorldGen.cs:70111-70132`): active, solid,
/// not top-only, no half-brick, no slope, not inactive. Out of world the
/// C# array read throws into the catch → false (`:70128-70131`); the
/// null-tile arm (`:70115-70118`, true) is unreachable in the port
/// (`World::tile` yields a default inactive tile in bounds — the
/// `grapple.rs` precedent).
fn solid_tile(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    t.active()
        && tr_core::tiledata::TILE_SOLID.has(t.kind)
        && !tr_core::tiledata::TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
        && !t.inactive()
}

/// `Utils.GetLerpValue(from, to, t, clamped: true)` (`Utils.cs:315-343`)
/// — the clamped 0..1 ramp.
fn get_lerp_value_clamped(from: f32, to: f32, t: f32) -> f32 {
    if from < to {
        if t < from {
            return 0.0;
        }
        if t > to {
            return 1.0;
        }
    } else {
        if t < to {
            return 1.0;
        }
        if t > from {
            return 0.0;
        }
    }
    (t - from) / (to - from)
}

/// `Utils.Remap(fromValue, fromMin, fromMax, toMin, toMax)`
/// (`Utils.cs:345-348`, clamped default true) = `MathHelper.Lerp(toMin,
/// toMax, GetLerpValue(...))`.
fn remap(v: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
    let t = get_lerp_value_clamped(from_min, from_max, v);
    to_min + (to_max - to_min) * t
}

/// Site 41 — `NPC.AI_037_Destroyer`'s airborne light (`NPC.cs:49925-49930`):
/// while the family segment overlaps NO solid/liquid tile (`flag2`, the
/// `:49884-49924` scan), every type except a body in its `ai[2] == 1`
/// state lights the quotient-truncated centre (0.3, 0.1, 0.05). RAW
/// position — `AI_037` never adds `netOffset`. The draw-side scan below
/// is the verbatim twin of `npc_ai_style37.rs::overlaps_solid_or_liquid`
/// (kept here so the feed does not reach into the sim module's private
/// fn).
pub(crate) fn destroyer_light(n: &Npc, world: &World) -> Option<Light> {
    let t = cs_type(n);
    if !(134..=136).contains(&t) {
        return None;
    }
    if destroyer_overlaps_solid_or_liquid(world, n) {
        return None; // `:49925` — `if (!flag2)`
    }
    if t == 135 && n.ai[2] == 1.0 {
        return None; // `:49927`
    }
    let (w, h) = n.size();
    Some((
        tile_x_pos(n.x, w),
        tile_y_pos(n.y, h),
        [0.3, 0.1, 0.05],
    ))
}

/// The `:49884-49924` tile-overlap scan (`flag2`), verbatim (the
/// `npc_ai_style37.rs::overlaps_solid_or_liquid` twin).
fn destroyer_overlaps_solid_or_liquid(world: &World, n: &Npc) -> bool {
    let (w, h) = n.size();
    let mut x0 = (n.x / 16.0).trunc() as i32 - 1;
    let mut x1 = ((n.x + w) / 16.0).trunc() as i32 + 2;
    let mut y0 = (n.y / 16.0).trunc() as i32 - 1;
    let mut y1 = ((n.y + h) / 16.0).trunc() as i32 + 2;
    if x0 < 0 {
        x0 = 0;
    }
    if x1 > world.width {
        x1 = world.width;
    }
    if y0 < 0 {
        y0 = 0;
    }
    if y1 > world.height {
        y1 = world.height;
    }
    for k in x0..x1 {
        for l in y0..y1 {
            let Some(t) = world.tile(k, l) else {
                continue;
            };
            // `:49912` — NO `inActive()` test here (unlike Collision's
            // solid gates); the Living Trees 192 exclusion is the port's
            // table convention (`WorldGen.cs:15715`, collision.rs).
            let solid = t.active()
                && ((tr_core::tiledata::TILE_SOLID.has(t.kind) && t.kind != 192)
                    || (tr_core::tiledata::TILE_SOLID_TOP.has(t.kind) && t.frame_y == 0));
            if solid || t.liquid > 64 {
                let tx = (k * 16) as f32;
                let ty = (l * 16) as f32;
                if n.x + w > tx && n.x < tx + 16.0 && n.y + h > ty && n.y < ty + 16.0 {
                    return true;
                }
            }
        }
    }
    false
}

/// Site 42 — the Probe 139's tile-light probe (`NPC.cs:50624-50631`,
/// inside `AI_005_EaterOfSouls`' type-139 arm): the SNAPPED centre
/// (`(int)position.X + width / 2`, INTEGER half — `:50624-50627`) must be
/// in-world and non-solid; the light itself uses the QUOTIENT-truncated
/// float centre (`:50630`). RAW position (the arm's netOffset wraps are
/// at `:50689`/`:51007`, elsewhere).
pub(crate) fn probe_light(n: &Npc, world: &World) -> Option<Light> {
    if cs_type(n) != 139 {
        return None;
    }
    let (w, h) = n.size();
    let mut num12 = n.x as i32 + w as i32 / 2;
    let mut num13 = n.y as i32 + h as i32 / 2;
    num12 /= 16;
    num13 /= 16;
    if !(in_world(world, num12, num13) && !solid_tile(world, num12, num13)) {
        return None;
    }
    Some((tile_x_pos(n.x, w), tile_y_pos(n.y, h), [0.3, 0.1, 0.05]))
}

/// Site 44 — the town shimmer state 25's hue-cycling light
/// (`NPC.cs:53187-53232`): every state-25 tick that reaches the line
/// lights `hslToRgb(timeForVisualEffects / 360 % 1, 0.6, 0.65) *
/// Remap(ai[1], 30, 90, 0, 0.7)` — the `ai[2] > 0` countdown returns
/// first (`:53206-53214`), and below `ai[1] == 30` the Remap clamps to a
/// ZERO amount (the source still enters the C#'s list). The `Main.rand`
/// draws around it (`:53231`/`:53233`/`:53237`) are the wholesale skip.
/// The state itself is a cited skip in `npc_ai_town.rs` — this arm is
/// unreachable until that wiring lands (the 75-lantern precedent). RAW
/// position (`AI_007` has no netOffset).
pub(crate) fn town_shimmer_light(n: &Npc, p: &LightParams) -> Option<Light> {
    if n.kind.ai_style() != 7 || n.ai[0] != 25.0 || n.ai[2] > 0.0 {
        return None;
    }
    let hue = (p.time_for_visuals / 360.0) % 1.0;
    let rgb = hsl_to_rgb(hue, 0.6, 0.65);
    let amount = remap(n.ai[1], 30.0, 90.0, 0.0, 0.7);
    let (w, h) = n.size();
    Some((
        tile_x(n_pos(n), w, XAnchor::Center),
        tile_y(n_pos(n), h, XAnchor::Center),
        [rgb[0] * amount, rgb[1] * amount, rgb[2] * amount],
    ))
}

/// `NPC.GetMagicAuraColor` (`NPC.cs:52916-52932`) + the
/// `NPCID.Sets.MagicAuraColor` rows (`NPCID.cs:10475`): the per-type
/// colour bytes /255 (XNA `Color.ToVector3`), the variation-1 swaps for
/// the Truffle (BGR), the Dryad (90, 20, 210) and the Princess
/// (255, 0, 77). The set's default is `Color.White`; the princess's base
/// is `Main.hslToRgb(0.92, 1, 0.78)` (byte-quantised, alpha 127 dropped
/// by ToVector3). Only the five rowed types reach the aura arms (module
/// docs) — the unrowed default stays `None`, unreachable.
fn magic_aura_color(t: i32, variation: i32) -> Option<[f32; 3]> {
    let base: [u8; 3] = match t {
        54 => [100, 4, 227],
        108 => [255, 80, 60],
        160 => [40, 80, 255],
        20 => [40, 255, 80],
        663 => {
            let c = hsl_to_rgb(0.92, 1.0, 0.78);
            [(c[0] * 255.0) as u8, (c[1] * 255.0) as u8, (c[2] * 255.0) as u8]
        }
        _ => return None,
    };
    let rgb = if variation == 1 {
        match t {
            160 => [base[2], base[1], base[0]],
            20 => [90, 20, 210],
            663 => [255, 0, 77],
            _ => base,
        }
    } else {
        base
    };
    Some([
        rgb[0] as f32 / 255.0,
        rgb[1] as f32 / 255.0,
        rgb[2] as f32 / 255.0,
    ])
}

/// Site 45 — the town magic attack (state 14)'s aura light
/// (`NPC.cs:54981-54985`): `GetMagicAuraColor().ToVector3() * num65`
/// where `num65 = 1f` (`:54838`, never rewritten in the arm). The
/// aiStyle-7 gate is the `AI()` dispatch (`:20812-20816`). RAW position.
pub(crate) fn town_magic_light(n: &Npc) -> Option<Light> {
    if n.kind.ai_style() != 7 || n.ai[0] != 14.0 {
        return None;
    }
    let rgb = magic_aura_color(cs_type(n), n.variation_index)?;
    let (w, h) = n.size();
    Some((
        tile_x(n_pos(n), w, XAnchor::Center),
        tile_y(n_pos(n), h, XAnchor::Center),
        rgb,
    ))
}

/// Site 46 — the town shimmer-arrival pose (state 24)'s aura light
/// (`NPC.cs:55123-55131`): `GetMagicAuraColor().ToVector3()` at full
/// amount, every tick of the pose. The aiStyle-7 gate as site 45. RAW
/// position.
pub(crate) fn town_arrival_light(n: &Npc) -> Option<Light> {
    if n.kind.ai_style() != 7 || n.ai[0] != 24.0 {
        return None;
    }
    let rgb = magic_aura_color(cs_type(n), n.variation_index)?;
    let (w, h) = n.size();
    Some((
        tile_x(n_pos(n), w, XAnchor::Center),
        tile_y(n_pos(n), h, XAnchor::Center),
        rgb,
    ))
}

/// Sites 47/48 — the town slimes' tail lights (`NPC.cs:55547-55560`,
/// `AI_007_TownEntities`' unconditional tail): 681 (TownSlimeRainbow)
/// lights `TorchID.TorchColor(23) * 0.35` (the shimmer torch, animated
/// by `mouseTextColor`); 683/687 (TownSlimeYellow + its bound form)
/// pulse `(0.25, 0.25, 0.1) * WrappedLerp(0.75, 1, timeForVisualEffects
/// % 120 / 120)`. The type gates are the C#'s own (all three types are
/// aiStyle 7, `NPC.cs:17594-17600`/`:17640-17644` — the tail only runs
/// inside `AI_007`). RAW position. No port kind yet (the town-slime
/// roster wave) — the arms are ported whole, unreachable until it lands.
pub(crate) fn town_slime_light(n: &Npc, p: &LightParams) -> Option<Light> {
    let t = cs_type(n);
    let rgb = if t == 681 {
        let c = tr_core::tile_light::torch_color(23, p);
        [c[0] * 0.35, c[1] * 0.35, c[2] * 0.35]
    } else if t == 683 || t == 687 {
        // `Utils.WrappedLerp` (`Utils.cs:305-313`): `num = percent * 2;
        // if (num > 1) num = 2 - num; Lerp(v1, v2, num)`.
        let mut num = ((p.time_for_visuals % 120.0) / 120.0) * 2.0;
        if num > 1.0 {
            num = 2.0 - num;
        }
        let num132 = 0.75 + (1.0 - 0.75) * num;
        [0.25 * num132, 0.25 * num132, 0.1 * num132]
    } else {
        return None;
    };
    let (w, h) = n.size();
    Some((
        tile_x(n_pos(n), w, XAnchor::Center),
        tile_y(n_pos(n), h, XAnchor::Center),
        rgb,
    ))
}

/// Sites 49/50 — Medusa 480's charge lights (`NPC.cs:56356-56457`,
/// `AI_003_Fighters`' type-480 arm): while `ai[2] < 0` (the wind-up
/// countdown), BOTH the deep wind-up (`ai[2] < -num22`, `:56384-56422`)
/// and the hold (`-num22 <= ai[2] < 0`, `:56424-56457`) light
/// (0.9, 0.75, 0.1) at the centre — `num22` is 180, or 240 below a third
/// life (`:56360-56369`; both arms share the colour so the boundary is
/// not re-derived here, the two `ai[2] < 0` arms cover the span). The
/// C# adds `netOffset` around both arms (`:56386`/`:56421`,
/// `:56426`/`:56457`) — the feed reads the offset position. The
/// charge-up dust (`:56405-56418`, three `Main.rand` draws) is the
/// wholesale skip; the `num25 == 1` sound (`:56401-56404`) is the audio
/// task's. No port kind yet — ported whole, unreachable until it lands.
pub(crate) fn medusa_light(n: &Npc) -> Option<Light> {
    if cs_type(n) != 480 || n.ai[2] >= 0.0 {
        return None;
    }
    let (w, h) = n.size();
    let pos = n_pos(n) + n.net_offset;
    Some((
        tile_x(pos, w, XAnchor::Center),
        tile_y(pos, h, XAnchor::Center),
        [0.9, 0.75, 0.1],
    ))
}

/// Sites 51/52 — TorchZombie 590's torch (`NPC.cs:56914-56947`): the
/// torch tile sits at `(int)(position.Y + 6) / 16` rows, column
/// `(int)(Center.X - 22) / 16` facing left / `(int)(Center.X + 14) / 16`
/// facing right; the light fires only when the column is in-world and
/// BOTH it and its neighbour are liquid-free. The neighbour reads
/// `Framing.GetTileSafely` (`Framing.cs:426-438`): out of world it yields
/// a FRESH `Tile` (liquid 0) — the check PASSES there. (1, 0.95, 0.8).
/// The C# adds `netOffset` (`:56916`/`:56946`). The `Next(30)` dust
/// (`:56926`/`:56940`) is the wholesale skip. `spriteDirection` is the
/// port's `sprite_dir`. No port kind yet — ported whole.
pub(crate) fn torch_zombie_light(n: &Npc, world: &World) -> Option<Light> {
    if cs_type(n) != 590 {
        return None;
    }
    let (w, _h) = n.size();
    let pos = n_pos(n) + n.net_offset;
    let cx = pos.x + w / 2.0;
    let num52 = (pos.y + 6.0) as i32 / 16;
    let col = if n.sprite_dir < 0 {
        (cx - 22.0) as i32 / 16
    } else {
        (cx + 14.0) as i32 / 16
    };
    let neighbour = if n.sprite_dir < 0 { col + 1 } else { col - 1 };
    // `GetTileSafely`'s fresh-tile fallback: liquid 0 out of world.
    let liquid_free = |x: i32| world.tile(x, num52).map_or(true, |t| t.liquid == 0);
    if !(in_world(world, col, num52) && liquid_free(neighbour) && liquid_free(col)) {
        return None;
    }
    Some((col, num52, [1.0, 0.95, 0.8]))
}

/// Sites 53/54 — ArmedTorchZombie 591's torch (`NPC.cs:56948-56970`):
/// while DRY, the light sits at the Vector2 point `(Center.X - 36,
/// position.Y + 24)` facing left / `(Center.X + 28, position.Y + 24)`
/// facing right — the Vector2 overload truncates the quotient per axis
/// (`Lighting.cs:140-143`). (1, 0.95, 0.8). netOffset added
/// (`:56950`/`:56970`); the `ai[2] == 0 && Next(30)` dust is the
/// wholesale skip. No port kind yet — ported whole.
pub(crate) fn armed_torch_zombie_light(n: &Npc) -> Option<Light> {
    if cs_type(n) != 591 || n.wet {
        return None;
    }
    let (w, _h) = n.size();
    let pos = n_pos(n) + n.net_offset;
    let cx = pos.x + w / 2.0;
    let (px, py) = if n.sprite_dir < 0 {
        (cx - 36.0, pos.y + 24.0)
    } else {
        (cx + 28.0, pos.y + 24.0)
    };
    Some(((px / 16.0) as i32, (py / 16.0) as i32, [1.0, 0.95, 0.8]))
}

/// Sites 55-59 — `AI_003_Fighters`' unconditional type tail
/// (`NPC.cs:58402-58421`): the hell Armored Bones 277-280 light
/// (0.2, 0.1, 0) at the TRUNCATED centre (`(int)Center.X / 16` —
/// truncate before dividing); the MartianWalker 520 lights
/// `Top + (0, 20)` with (0.3, 0.3, 0.7); the Desert Ghouls 525/526/527
/// light `Top + (0, 15)` with their (0.7, 1, 0.2)×0.5 / (1, 1, 0.5)×0.4
/// / (0.6, 0.3, 1)×0.4 colours. `Top` = `(position.X + width/2,
/// position.Y)` (`Entity.cs:86-91`), the Vector2 overload truncates the
/// quotient per axis. RAW position (the tail is unwrapped — the
/// fighter's netOffset wraps are elsewhere).
pub(crate) fn fighter_tail_light(n: &Npc) -> Option<Light> {
    let t = cs_type(n);
    let (w, h) = n.size();
    let pos = n_pos(n);
    if (277..=280).contains(&t) {
        return Some((
            tile_x(pos, w, XAnchor::CenterTruncated),
            tile_y(pos, h, XAnchor::CenterTruncated),
            [0.2, 0.1, 0.0],
        ));
    }
    let (dy, rgb) = match t {
        520 => (20.0, [0.3, 0.3, 0.7]),
        525 => (15.0, [0.7 * 0.5, 1.0 * 0.5, 0.2 * 0.5]),
        526 => (15.0, [1.0 * 0.4, 1.0 * 0.4, 0.5 * 0.4]),
        527 => (15.0, [0.6 * 0.4, 0.3 * 0.4, 1.0 * 0.4]),
        _ => return None,
    };
    let top_x = pos.x + w / 2.0;
    let top_y = pos.y + dy;
    Some(((top_x / 16.0) as i32, (top_y / 16.0) as i32, rgb))
}

/// Sites 60/63/64/65/66 — `AI_001_Slimes`' light arms, in C# order:
/// * `:60343-60346` — type 1 with the lantern content `ai[1] == 75`:
///   (0.8, 0.7, 0.1)×0.3 at the quotient-truncated centre. Unreachable
///   until the content init lands (`npc_ai_slime.rs`'s A1-5 census).
/// * `:61053-61057` — RainbowSlime 244: `Main.DiscoR/G/B / 255` at the
///   quotient-truncated centre (the named-argument overload's `i`/`j`
///   are the float-centre form).
/// * `:61109-61112` — GoldenSlime 667: the fixed (204, 181, 72) bytes
///   /255 ×1.1 at the quotient-truncated centre.
/// * `:61128-61130` — ShimmerSlime 676: `AddLight(Center, 23)` — the
///   torch-id overload (`Lighting.cs:151-155`) truncates the centre
///   BEFORE dividing (`(int)position.X / 16`), full shimmer-torch colour.
/// * `:61451-61454` — LavaSlime 59: (1, 0.3, 0.1) at the
///   quotient-truncated centre, netOffset ADDED (`:61453`/`:61457` —
///   the only slime light arm that wraps).
/// The ore-slime (`:60760`, needs `NPC.color`) and herb (`:60896`)
/// content arms are the cited skips (module docs). Every dust gate is
/// the wholesale skip. The `ai[0] == -999` freeze return (`:61046-61052`)
/// only ever hits the type-1 balloon slimes (`:47908`) whose sole light
/// arm sits BEFORE it — the 244/667/676/59 arms below are unaffected.
pub(crate) fn slime_lights(n: &Npc, p: &LightParams) -> Vec<Light> {
    let mut out = Vec::new();
    let t = cs_type(n);
    let (w, h) = n.size();
    let raw = n_pos(n);
    if t == 1 && n.ai[1] == 75.0 {
        let num = 0.3;
        out.push((
            tile_x(raw, w, XAnchor::Center),
            tile_y(raw, h, XAnchor::Center),
            [0.8 * num, 0.7 * num, 0.1 * num],
        ));
    }
    if t == 244 {
        out.push((
            tile_x(raw, w, XAnchor::Center),
            tile_y(raw, h, XAnchor::Center),
            [
                p.disco[0] as f32 / 255.0,
                p.disco[1] as f32 / 255.0,
                p.disco[2] as f32 / 255.0,
            ],
        ));
    }
    if t == 667 {
        // `new Color(204, 181, 72, 255)` — `(float)(int)color.R / 255f * 1.1f`.
        out.push((
            tile_x(raw, w, XAnchor::Center),
            tile_y(raw, h, XAnchor::Center),
            [
                204.0 / 255.0 * 1.1,
                181.0 / 255.0 * 1.1,
                72.0 / 255.0 * 1.1,
            ],
        ));
    }
    if t == 676 {
        out.push((
            tile_x(raw, w, XAnchor::CenterTruncated),
            tile_y(raw, h, XAnchor::CenterTruncated),
            tr_core::tile_light::torch_color(23, p),
        ));
    }
    if t == 59 {
        let pos = raw + n.net_offset;
        out.push((
            tile_x(pos, w, XAnchor::Center),
            tile_y(pos, h, XAnchor::Center),
            [1.0, 0.3, 0.1],
        ));
    }
    out
}

/// Site 67 — the HeadlessHorseman's gallop light (`NPC.cs:62420`) is
/// ALREADY landed: `npc_ai_unicorn_b::horseman_light`, fed by the draw
/// pass (`app/frame/draw.rs`). NO double-feed here — the site is recorded
/// for the census only.

/// `NPCID.Sets.TrailingMode` (`NPCID.cs:10430`, `CreateIntSet(-1, ...)`):
/// mode 4 is exactly {120 ChaosElemental, 137 IlluminantBat, 138
/// IlluminantSlime}. Every other ported kind defaults -1 (no trail).
fn trailing_mode(t: i32) -> i32 {
    match t {
        120 | 137 | 138 => 4,
        _ => -1,
    }
}

/// Site 68 — `UpdateNPC_UpdateTrails`' mode-4 light (`NPC.cs:90836-90844`):
/// the shift loop adds (0.3, 0, 0.2) at the RAW-position tile
/// (`(int)position.X / 16`, `(int)position.Y / 16` — truncate before
/// dividing) NINE times per tick (`oldPos.Length - 1`, `NPC.cs:6119`:
/// `new Vector2[10]`). The duplicates are verbatim — the engine
/// max-merges, but the list count is the C#'s. Only the IlluminantSlime
/// 138 has a port kind (the 120/137 rows wait on their roster wave).
pub(crate) fn trail_light(n: &Npc) -> Vec<Light> {
    if trailing_mode(cs_type(n)) != 4 {
        return Vec::new();
    }
    let x = n.x as i32 / 16;
    let y = n.y as i32 / 16;
    vec![(x, y, [0.3, 0.0, 0.2]); 9]
}

/// `NPC.canDisplayBuffs` (`NPC.cs:6199`, default true; reset true at
/// `:8491`): the ONLY clears are the Golem 245/246/247/248 rows inside
/// `getGoodAdjustments` (`:18010`/`:18015`/`:18020`), which `SetDefaults`
/// runs only under `Main.getGoodWorld` (`:17843-17846`). The port's NPC
/// machines run the stock config (`get_good_world: false` — the
/// `npc_ai_golem.rs`/`npc_ai_style37.rs` convention), so the flag folds
/// to TRUE for every kind here. The `:91145` gate below is kept explicit
/// so the FTW wiring has its seam (the flag IS read there — the draft's
/// "no other reader" claim was wrong).
fn can_display_buffs(_n: &Npc) -> bool {
    true
}

/// Sites 69-75 — `UpdateNPC_BuffApplyVFX`'s seven fire lights
/// (`NPC.cs:91233-91513`), in C# order: OnFire! 24 / Hellfire 323 /
/// Daybreak 189 light (1, 0.3, 0.1); Betsy's Curse 203 lights
/// (0.6, 0.1, 0.9); Frostburn 44 / Frostfire 324 light (0.1, 0.6, 1);
/// Cursed Inferno 39 lights (1, 0.3, 0.1). All at
/// `((int)(position.X / 16f), (int)(position.Y / 16f + 1f))` — the +1
/// rides INSIDE the truncation. The C# adds `netOffset` once at the
/// method head (`:91149`) and the `canDisplayBuffs` gate (`:91145`)
/// returns early (the fold above). The buff flags are derived from the
/// live buff slots (`npc_buffs.rs`'s `set_flags_tick` map,
/// `NPC.cs:92009-92132`); every dust arm is the wholesale skip.
pub(crate) fn buff_vfx_lights(n: &Npc) -> Vec<Light> {
    let mut out = Vec::new();
    if !can_display_buffs(n) {
        return out; // `:91145-91148`
    }
    let pos = n_pos(n) + n.net_offset;
    let x = (pos.x / 16.0) as i32;
    let y = (pos.y / 16.0 + 1.0) as i32;
    let b = &n.buffs;
    if b.has_buff(24) {
        out.push((x, y, [1.0, 0.3, 0.1])); // `:91247`
    }
    if b.has_buff(323) {
        out.push((x, y, [1.0, 0.3, 0.1])); // `:91264`
    }
    if b.has_buff(189) {
        out.push((x, y, [1.0, 0.3, 0.1])); // `:91280`
    }
    if b.has_buff(203) {
        out.push((x, y, [0.6, 0.1, 0.9])); // `:91295`
    }
    if b.has_buff(44) {
        out.push((x, y, [0.1, 0.6, 1.0])); // `:91480`
    }
    if b.has_buff(324) {
        out.push((x, y, [0.1, 0.6, 1.0])); // `:91496`
    }
    if b.has_buff(39) {
        out.push((x, y, [1.0, 0.3, 0.1])); // `:91512`
    }
    out
}

/// Sites 79/80 — `UpdateNPC_CastLights`' RNG-free arms
/// (`NPC.cs:92397-92413`): the Cyborg 209's rocket-pose light
/// (`ai[0] == 12`, `base.Top`, (0.3, 0.1, 0.1) — the Top overload
/// truncates the quotient per axis) and the UndeadMiner 44's helmet
/// lamp at `((int)(position.X + width/2) / 16, (int)(position.Y + 4) /
/// 16)` — TRUNCATED centre x, +4 y — (0.9, 0.75, 0.5). RAW position
/// (the method's only netOffset wrap is the gold-critter block's,
/// `:92347`/`:92367`). The 160/254-261/634/635/209 flicker arms draw
/// `Main.rand.Next(28, 42)` every tick — cited skips (module docs).
pub(crate) fn cast_lights(n: &Npc) -> Vec<Light> {
    let mut out = Vec::new();
    let t = cs_type(n);
    let (w, h) = n.size();
    let pos = n_pos(n);
    if t == 209 && n.ai[0] == 12.0 {
        out.push((
            tile_x(pos, w, XAnchor::Top),
            tile_y(pos, h, XAnchor::Top),
            [0.3, 0.1, 0.1],
        ));
    }
    if t == 44 {
        out.push((
            (pos.x + w / 2.0) as i32 / 16,
            (pos.y + 4.0) as i32 / 16,
            [0.9, 0.75, 0.5],
        ));
    }
    out
}

/// The whole second-half feed for one NPC, in `UpdateNPC`'s call order
/// (`NPC.cs:90320` CastLights → `:90378` BuffApplyVFX → `:90423` AI →
/// `:90512` UpdateTrails). The AI arms are mutually exclusive by
/// aiStyle/type in the C#; the port evaluates each gate independently
/// (the same set fires). No arm draws RNG — the feed is draw-side safe.
pub(crate) fn npc_lights_b(n: &Npc, world: &World, p: &LightParams) -> Vec<Light> {
    let mut out = Vec::new();
    out.extend(cast_lights(n));
    out.extend(buff_vfx_lights(n));
    if let Some(l) = destroyer_light(n, world) {
        out.push(l);
    }
    if let Some(l) = probe_light(n, world) {
        out.push(l);
    }
    if let Some(l) = town_shimmer_light(n, p) {
        out.push(l);
    }
    if let Some(l) = town_magic_light(n) {
        out.push(l);
    }
    if let Some(l) = town_arrival_light(n) {
        out.push(l);
    }
    if let Some(l) = town_slime_light(n, p) {
        out.push(l);
    }
    if let Some(l) = medusa_light(n) {
        out.push(l);
    }
    if let Some(l) = torch_zombie_light(n, world) {
        out.push(l);
    }
    if let Some(l) = armed_torch_zombie_light(n) {
        out.push(l);
    }
    if let Some(l) = fighter_tail_light(n) {
        out.push(l);
    }
    out.extend(slime_lights(n, p));
    out.extend(trail_light(n));
    out
}

/// The raw top-left as a `Vec2` for the shared anchor helpers.
fn n_pos(n: &Npc) -> glam::Vec2 {
    glam::Vec2::new(n.x, n.y)
}

/// The quotient-truncated centre tile x (`(int)((position.X + width/2)
/// / 16f)` — the common AI form).
fn tile_x_pos(x: f32, w: f32) -> i32 {
    ((x + w / 2.0) / 16.0) as i32
}

/// The quotient-truncated centre tile y.
fn tile_y_pos(y: f32, h: f32) -> i32 {
    ((y + h / 2.0) / 16.0) as i32
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

    fn air() -> World {
        World::new("t".into(), 64, 64)
    }

    fn solid(w: &mut World, x: i32, y: i32) {
        let i = w.index(x, y);
        w.tiles[i] = Tile { kind: 1, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
    }

    #[test]
    fn destroyer_head_lights_while_airborne_and_goes_dark_over_ground() {
        // `:49925-49929` — the head (134, 48×48) at (160, 160) in an air
        // world: centre (184, 184) → tile (11, 11), (0.3, 0.1, 0.05).
        let mut n = Npc::new(NpcKind::TheDestroyer, 160.0, 160.0, 1, 0.0);
        let w = air();
        let l = destroyer_light(&n, &w).expect("airborne head lights");
        assert_eq!(l, (11, 11, [0.3, 0.1, 0.05]));
        // A solid tile INSIDE the box flips flag2 → dark (tile (11,11)
        // spans 176..192, overlapping the 160..208 box).
        let mut w2 = air();
        solid(&mut w2, 11, 11);
        assert!(destroyer_light(&n, &w2).is_none());
        // netOffset is NOT added by AI_037 — the light stays put.
        n.net_offset = glam::Vec2::new(32.0, 0.0);
        assert_eq!(destroyer_light(&n, &w).unwrap().0, 11);
    }

    #[test]
    fn destroyer_body_ai2_one_stays_dark_but_the_tail_does_not() {
        // `:49927` — only the BODY (135) skips in its ai[2] == 1 state.
        let mut body = Npc::new(NpcKind::TheDestroyerBody, 160.0, 160.0, 1, 0.0);
        let w = air();
        body.ai[2] = 1.0;
        assert!(destroyer_light(&body, &w).is_none());
        body.ai[2] = 0.0;
        assert!(destroyer_light(&body, &w).is_some());
        let mut tail = Npc::new(NpcKind::TheDestroyerTail, 160.0, 160.0, 1, 0.0);
        tail.ai[2] = 1.0;
        assert!(destroyer_light(&tail, &w).is_some());
    }

    #[test]
    fn probe_lights_only_when_the_snapped_centre_is_open() {
        // `:50624-50630` — the Probe's 30×30 box at (160, 160). BOTH the
        // gate and the light truncate a centre computed from the raw
        // position: the gate `(int)160 + 30/2 = 175` then `/16 = 10`, the
        // light `(int)((160 + 15) / 16) = (int)10.9375 = 10` — the SAME
        // tile (there is no float-vs-int split for positive positions).
        // Open gate tile → light at (10, 10); solid gate tile → dark.
        let n = Npc::new(NpcKind::Probe, 160.0, 160.0, 1, 0.0);
        let mut w = air();
        assert_eq!(probe_light(&n, &w).unwrap(), (10, 10, [0.3, 0.1, 0.05]));
        solid(&mut w, 10, 10);
        assert!(probe_light(&n, &w).is_none());
    }

    #[test]
    fn the_hungry_115_never_lights_the_116_arm() {
        // The roster's Hungry is TheHungry 115 — the `type == 116` arm
        // (`:52425-52428`) is a cited skip; cs_type resolves to 115.
        let n = Npc::new(NpcKind::Hungry, 160.0, 160.0, 1, 0.0);
        assert_eq!(cs_type(&n), 115);
    }

    #[test]
    fn town_shimmer_emits_zero_amount_below_30_and_ramps_after() {
        // `:53232` fires for EVERY reached state-25 tick: ai[1] = 10 →
        // Remap clamps to 0 (the source still enters the list); the
        // ai[2] > 0 countdown returns first (`:53206-53214`).
        let mut n = Npc::new(NpcKind::Guide, 160.0, 160.0, 1, 0.0);
        let mut p = LightParams::default();
        p.time_for_visuals = 180.0; // hue 0.5
        n.ai[0] = 25.0;
        n.ai[1] = 10.0;
        let l = town_shimmer_light(&n, &p).expect("state 25 always emits");
        assert_eq!(l.2, [0.0, 0.0, 0.0]);
        n.ai[1] = 60.0; // Remap(60, 30, 90, 0, 0.7) = 0.35
        let l = town_shimmer_light(&n, &p).unwrap();
        let rgb = hsl_to_rgb(0.5, 0.6, 0.65);
        assert_eq!(l.2, [rgb[0] * 0.35, rgb[1] * 0.35, rgb[2] * 0.35]);
        n.ai[2] = 5.0; // the countdown return
        assert!(town_shimmer_light(&n, &p).is_none());
    }

    #[test]
    fn town_magic_and_arrival_aura_are_town_only() {
        // The Dryad (20) base aura (40, 255, 80)/255; variation 1 swaps
        // to (90, 20, 210)/255 (`:52923-52926`).
        let mut n = Npc::new(NpcKind::Dryad, 160.0, 160.0, 1, 0.0);
        n.ai[0] = 14.0;
        let l = town_magic_light(&n).expect("attack aura");
        assert_eq!(l.2, [40.0 / 255.0, 1.0, 80.0 / 255.0]);
        n.variation_index = 1;
        assert_eq!(
            town_magic_light(&n).unwrap().2,
            [90.0 / 255.0, 20.0 / 255.0, 210.0 / 255.0]
        );
        n.ai[0] = 24.0;
        assert!(town_arrival_light(&n).is_some());
        // A non-town machine's state 14 never lights (the aiStyle-7
        // dispatch gate, `:20812-20816`).
        let mut bat = Npc::new(NpcKind::CaveBat, 160.0, 160.0, 1, 0.0);
        bat.ai[0] = 14.0;
        assert!(town_magic_light(&bat).is_none());
    }

    #[test]
    fn medusa_charge_lights_the_offset_position() {
        // No Medusa kind yet — the arm is ported whole behind the type
        // gate; the netOffset convention is pinned through the helper.
        let mut n = Npc::new(NpcKind::Zombie, 160.0, 160.0, 1, 0.0);
        n.ai[2] = -5.0;
        assert!(medusa_light(&n).is_none(), "no 480 kind yet");
        // The anchor math the arm uses: offset centre, quotient form.
        let pos = n_pos(&n) + glam::Vec2::new(16.0, 0.0);
        assert_eq!(tile_x(pos, 18.0, XAnchor::Center), 11);
    }

    #[test]
    fn torch_zombie_neighbour_passes_out_of_world() {
        // `:56921-56923` — the neighbour reads GetTileSafely, whose
        // out-of-world fresh Tile has liquid 0: the check PASSES there
        // (the draft's `is_some_and` wrongly failed it). No 590 kind yet,
        // so the gate is pinned through the same expression the arm uses.
        let w = air();
        let liquid_free = |x: i32, y: i32| w.tile(x, y).map_or(true, |t| t.liquid == 0);
        assert!(liquid_free(-1, 5), "fresh tile out of world is dry");
        assert!(liquid_free(64, 5), "the far edge too");
        // In-world liquid blocks it.
        let mut w2 = air();
        let i = w2.index(10, 5);
        w2.tiles[i].liquid = 100;
        let lf2 = |x: i32, y: i32| w2.tile(x, y).map_or(true, |t| t.liquid == 0);
        assert!(!lf2(10, 5));
    }

    #[test]
    fn fighter_tail_martian_walker_lights_top_plus_20() {
        // `:58408` — the Walker's 34×110 box at (160, 176): Top =
        // (160+17, 176) + (0, 20) = (177, 196) → (11, 12),
        // (0.3, 0.3, 0.7). RAW position — netOffset must not move it.
        let mut n = Npc::new(NpcKind::MartianWalker, 160.0, 176.0, 1, 0.0);
        let l = fighter_tail_light(&n).expect("walker tail lights");
        assert_eq!(l, (11, 12, [0.3, 0.3, 0.7]));
        n.net_offset = glam::Vec2::new(16.0, 0.0);
        assert_eq!(fighter_tail_light(&n).unwrap(), l);
    }

    #[test]
    fn fighter_tail_other_types_stay_dark() {
        // The 277-280 / 525-527 arms have no port kinds yet; a plain
        // zombie never matches the tail.
        let n = Npc::new(NpcKind::Zombie, 160.0, 176.0, 1, 0.0);
        assert!(fighter_tail_light(&n).is_none());
    }

    #[test]
    fn slime_lantern_content_lights_behind_its_gate() {
        // `:60343-60346` — type 1 + ai[1] == 75: (0.8, 0.7, 0.1)×0.3 at
        // the quotient centre. The 24×18 blue slime at (160, 160):
        // centre (172, 169) → (10, 10).
        let mut n = Npc::new(NpcKind::Slime, 160.0, 160.0, 1, 0.0);
        let p = LightParams::default();
        assert!(slime_lights(&n, &p).is_empty(), "no content yet");
        n.ai[1] = 75.0;
        let ls = slime_lights(&n, &p);
        assert_eq!(ls.len(), 1);
        assert_eq!(ls[0], (10, 10, [0.8 * 0.3, 0.7 * 0.3, 0.1 * 0.3]));
    }

    #[test]
    fn lava_slime_lights_the_offset_centre() {
        // `:61451-61454` — the only slime arm that wraps netOffset. The
        // LavaSlime's 24×18 box (`NPC.cs:9518-9519`) at (160, 160):
        // centre (172, 169) → (10, 10); with offset (16, 0) the centre
        // (188, 169) → (11, 10). (1, 0.3, 0.1).
        let mut n = Npc::new(NpcKind::LavaSlime, 160.0, 160.0, 1, 0.0);
        let p = LightParams::default();
        let ls = slime_lights(&n, &p);
        assert_eq!(ls.len(), 1);
        assert_eq!(ls[0], (10, 10, [1.0, 0.3, 0.1]));
        n.net_offset = glam::Vec2::new(16.0, 0.0);
        let moved = slime_lights(&n, &p);
        assert_eq!(moved[0], (11, 10, [1.0, 0.3, 0.1]));
    }

    #[test]
    fn the_disco_gold_shimmer_slime_arms_wait_on_their_kinds() {
        // 244/667/676 have no port kinds yet — the arms are ported whole
        // behind their type gates; a blue slime never matches them.
        let n = Npc::new(NpcKind::Slime, 160.0, 160.0, 1, 0.0);
        let p = LightParams::default();
        assert!(slime_lights(&n, &p).is_empty());
        // The shimmer torch-id overload truncates BEFORE dividing
        // (`Lighting.cs:151-155`) — pinned through the shared anchor.
        let pos = glam::Vec2::new(175.9, 175.9);
        assert_eq!(tile_x(pos, 0.0, XAnchor::CenterTruncated), 10);
        assert_eq!(tile_x(pos, 0.0, XAnchor::Center), 10);
        let neg = glam::Vec2::new(-1.0, -1.0);
        assert_eq!(tile_x(neg, 0.0, XAnchor::CenterTruncated), 0);
        assert_eq!(tile_x(neg, 0.0, XAnchor::Center), 0);
    }

    #[test]
    fn illuminant_slime_trail_emits_nine_duplicates() {
        // `:90838-90842` — mode 4 (138) loops oldPos.Length-1 = 9 times
        // over the SAME raw tile. The 24×18 box at (160.9, 176.9):
        // (int)160.9/16 = 10, (int)176.9/16 = 11.
        let n = Npc::new(NpcKind::IlluminantSlime, 160.9, 176.9, 1, 0.0);
        let ls = trail_light(&n);
        assert_eq!(ls.len(), 9);
        assert!(ls.iter().all(|&l| l == (10, 11, [0.3, 0.0, 0.2])));
        // A non-trail kind emits nothing.
        let z = Npc::new(NpcKind::Zombie, 160.0, 176.0, 1, 0.0);
        assert!(trail_light(&z).is_empty());
    }

    #[test]
    fn buff_vfx_lights_the_offset_tile_plus_one_row() {
        // `:91247` — OnFire! 24 at ((int)(x/16), (int)(y/16 + 1)). The
        // zombie at (160, 160) + offset (0, 16): x = 10, y = (176/16+1)
        // = 12. The +1 rides INSIDE the truncation.
        let mut n = Npc::new(NpcKind::Zombie, 160.0, 160.0, 1, 0.0);
        n.net_offset = glam::Vec2::new(0.0, 16.0);
        assert!(buff_vfx_lights(&n).is_empty(), "no buff, no light");
        n.buffs.add_buff(24, 600);
        let ls = buff_vfx_lights(&n);
        assert_eq!(ls, vec![(10, 12, [1.0, 0.3, 0.1])]);
        // Betsy's Curse 203 is the purple one.
        let mut m = Npc::new(NpcKind::Zombie, 160.0, 160.0, 1, 0.0);
        m.buffs.add_buff(203, 600);
        assert_eq!(buff_vfx_lights(&m)[0].2, [0.6, 0.1, 0.9]);
    }

    #[test]
    fn undead_miner_helmet_lamp_truncates_the_centre() {
        // `:92412` — ((int)(x + width/2)/16, (int)(y + 4)/16). The 18×40
        // miner at (160, 160): (169/16, 164/16) = (10, 10),
        // (0.9, 0.75, 0.5). RAW position.
        let mut n = Npc::new(NpcKind::UndeadMiner, 160.0, 160.0, 1, 0.0);
        let ls = cast_lights(&n);
        assert_eq!(ls, vec![(10, 10, [0.9, 0.75, 0.5])]);
        n.net_offset = glam::Vec2::new(16.0, 0.0);
        assert_eq!(cast_lights(&n), ls, "the 44 arm reads RAW position");
    }

    #[test]
    fn the_feed_runs_cast_then_buff_then_ai_then_trail() {
        // `UpdateNPC`'s call order (:90320/:90378/:90423/:90512). An
        // IlluminantSlime on fire: the buff light precedes the trail's
        // nine duplicates; no AI arm matches 138.
        let mut n = Npc::new(NpcKind::IlluminantSlime, 160.0, 160.0, 1, 0.0);
        n.buffs.add_buff(24, 600);
        let w = air();
        let p = LightParams::default();
        let ls = npc_lights_b(&n, &w, &p);
        assert_eq!(ls.len(), 10);
        assert_eq!(ls[0], (10, 11, [1.0, 0.3, 0.1]), "buff first");
        assert_eq!(ls[1], (10, 10, [0.3, 0.0, 0.2]), "trail after");
    }
}
