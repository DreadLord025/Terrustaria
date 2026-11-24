//! The aiStyle-7 town-NPC family — `NPC.AI()`'s `if (this.aiStyle == 7)`
//! dispatch (`NPC.cs:20812-20815`) into `AI_007_TownEntities`
//! (`NPC.cs:52965-55561`) and its helpers (`:52765-52963`,
//! `:55563-55785`), S1-D-3 (town-NPC slice, code-only mode).
//!
//! IDENTITY: the C# dispatch is aiStyle 7 → the town-NPC family (this
//! module, `task/S1-D-3-town-ai`) and aiStyle 8 → the caster arm
//! (`npc_ai_caster.rs`, sibling S1-D-3).
//!
//! WIRING SPLIT (doc-sweep lane 36, `docs/research/121-doc-sweep-shimmer.md`):
//! the FULL aiStyle-7 migration (retire `update_guide`, the axis claim, the
//! door latch, the generic `UpdateNPC` tail) is W4-b3's pushed phase 1
//! (`task/W4-b3-npc-ai-town-wire` 96a5d88f) — this branch does NOT duplicate
//! it. What THIS branch wires is the state-25 emergence seam only:
//! [`town_shimmer_step`] (below) runs the W4-f47 emergence machine for a
//! town NPC whose `ai[0] == 25` at its slot turn in `world_sim/npcs.rs`,
//! draining [`ShimmerFx`] onto the dust pool / entity lights and crediting
//! progression 43. The `drain_shimmer_transforms` TownVariant arm
//! (`NPC.cs:91578-91591`) sets that state, so the shimmer→emergence chain is
//! LIVE end-to-end on this branch while the dry walk stays `update_guide`
//! until b3 lands. At the b3 merge the seam folds into b3's
//! `ai_007_town_family` (its `town_npc_ai` call reaches the same
//! `:53187` arm; the merge keeps ONE driver — the contract is documented
//! in ANALYSIS 121). The rest of the migration (TownOutputs drain, real
//! door ops, the live NPC list, `HomeTileUpdate` through `TownRooms` +
//! message 60) stays b3's phase 2. The contested pieces stay untouched:
//! the housing validation (`town.rs`), the chat selection (`npc_chat`),
//! and the caster aiStyle-8 arm.
//! WIRED — phase 1 (W4-b3, `task/W4-b3-npc-ai-town-wire`): the aiStyle
//! axis in `npc.rs::update_rng_dust` claims style 7 through
//! `Npc::ai_007_town_family`, which builds the solo classic [`TownEnv`]
//! (one session player, empty NPC list, the unported statics folded
//! false), the kind defaults ([`TownKindStats`]), the caller-carried
//! door latch (the `Npc`'s `town_door_close/x/y`, `NPC.cs:6511-6513`)
//! and a NO-OP [`TownDoorOps`] — every attempt fails, so the arm takes
//! the C#'s all-fail direction flip (tick-exact against a world with no
//! door port yet). The retired `update_guide` stand-in (the 240-tick
//! fixed flip, the constant 0.5 px/tick, the invented −6.2 jump) is
//! GONE. The adapter also runs the generic `UpdateNPC` tail the arm
//! never reaches (`integrate_town_collision`: depth-scaled gravity, the
//! ±0.005 deadband, `TileCollision` clip + dry/wet move). PHASE 2
//! remainder: drain [`TownOutputs`] (shots/strikes/partner writes/home
//! updates/saved states) through the sim's spawn/strike/housing seams,
//! implement [`TownDoorOps`] over `tr_core::edit::{open_door,
//! close_door}` (+ `ShiftTallGate` when it lands), thread the live NPC
//! list + `App.talk_npc`, and apply [`HomeTileUpdate`] through
//! `TownRooms` + message 60. The contested pieces stay untouched: the
//! housing validation (`town.rs`), the chat selection (`npc_chat`), and
//! the caster aiStyle-8 arm. The draw-delta census + the wiring
//! witnesses: ANALYSIS "The town-NPC aiStyle-7 wiring (W4-b3)".
//!
//! W4-g90 UPDATE: the STAT LADDER half is now LIVE — the App town-tick
//! feeder (`world_sim::npcs::town_stat_tick`, ahead of the slot loop)
//! runs [`town_stat_bonuses`] per town NPC per tick on [`TownEnvFlags`]
//! (the world flags incl. BOTH combat books + the boss progression), so
//! the +250 HP / ×0.8 cooldown / +0.25 damage / +8 defense book rows and
//! the boss ladder apply in play. The AI dispatch above stays the b3
//! lane's (its phase-1 `ai_007_town_family` builds its own env; at the
//! merge the feeder's ladder call dedupes onto b3's arm — the ladder is
//! idempotent per tick, same inputs → same writes, zero draws).
//!
//! SCOPE — the whole arm's STRUCTURE with the branches the port's town
//! kinds take (the ten housed kinds + the eleven town-like sellers,
//! `npc/data.rs::is_town_npc_like`; every one is 18×40). Cited skips
//! (land with their kinds/systems):
//! * the town CRITTERS (`NPCID.Sets.TownCritter`, `ID/NPCID.cs:10459`,
//!   49 types — none ported): the `:53278-53295` facing arm, the
//!   `CritterThatCanTurnOnPlayers` turn-on (`:53392-53414`), the
//!   `flag8/9/10` water bodies (`:53451-53453`) and every `TownCritter`
//!   timer add (`:53732`/`:53750`/`:53824`/`:54191`/`:54236`) — the
//!   branches are present behind [`town_critter`] and evaluate false;
//! * the town PETS (`IsTownPet`, `:10118` — none ported): the idle
//!   animation rolls ([`pet_idle_animation`], `:55750-55785`) and the
//!   pet gates are ported as pure fns, unreachable until the pets land;
//! * the SHIMMER state 25 (`:53187-53277`) + the shimmer teleport scan
//!   (`:55563-55641`, `ShimmerHelper.FindSpotWithoutShimmer`): the state
//!   is a cited skip (returns, draws shimmer dust — the dust task's).
//!   The RNG-free DECISION CORE (the ai[2] hold, the climb, the transparency
//!   decay, the completion variant-flip + the progression-43 gate) is ported
//!   as `crate::town_shimmer::town_shimmer_step` (W4-f30) — the wiring calls
//!   it here once the shimmer trigger lands (f47's buff-353 ramp →
//!   `GetShimmered` town-set arm); the visuals/teleport stay f47's seam;
//! * the Old Man 37 / Clothier 54 / Santa 142 (`:53158-53161` +
//!   the type arms), the Mechanic's owned-proj
//!   (`:55563-55641`, `ShimmerHelper.FindSpotWithoutShimmer`): PORTED
//!   (W4-f47 phase 2) and WIRED via [`town_shimmer_step`] (doc-sweep
//!   lane 36) — the visuals drain onto the shared pool/lights;
//! * the Old Man 37 / Clothier 54 / Santa 142 / the bound-NPC Transform
//!   rows 362/364/602/608 (`:53158-53184`), the Mechanic's owned-proj
//!   latch (`:53166-53179`) and the Angler's wet edge rescue
//!   (`:53375-53391`): the kinds are ported but those arms ride systems
//!   the port has not landed (dungeon home, owned proj
//!   search); the Angler arm IS ported ([`TownEnv`] carries the inputs).
//!   The waterfowl takeoff rows 362/364/602/608 (`:53180-53184`) ARE
//!   ported (W4-c10 sweep): the gate emits [`TownOutputs::waterfowl_takeoff`]
//!   and returns; the caller runs the `Transform(type + 1)` core;
//! * the ATTACK bodies 10/12/14 (`:54329-54994`): the trigger ladder
//!   (`:55443-55545`) enters the states and arms the timers; the shot
//!   bodies port the per-type preset tables + the aim maths + the RNG
//!   jitter as [`TownShot`] requests, but `Projectile.NewProjectile`'s
//!   spawn drain and `GetSwingStats`' melee sweep (15, `:54995-55122`)
//!   land with the projectile wiring — the sweep emits [`TownStrike`]
//!   rects instead of striking;
//! * the DEFENSE/HP progression ladder (`:52999-53157`, the boss-downed
//!   `num2`/`num3` terms): the ladder IS ported ([`town_stat_bonuses`])
//!   but its `defLifeMax`/`defDefense` reads need the kind table's
//!   defaults rows — the seam carries them in [`TownKindStats`];
//! * lighting/dust/particles/sound (`Lighting.AddLight`, `Dust.*`,
//!   `ParticleOrchestrator`, `SoundEngine`): the render/audio tasks'.
//!
//! The arm RETURNS at `:55561` — the generic `UpdateNPC` tail (gravity
//! `NPC.cs:90881`'s `NPC.gravity = 0.3f`, `position += velocity`,
//! `UpdateCollision`) is the caller's, NOT this module's.
//!
//! RNG by construction (every `Main.rand` draw on the ported paths, in
//! UpdateNPCs order — one `UnifiedRandom` threaded through, the caster
//! module's convention; `UnifiedRandom.cs:120-141` via
//! `tr_core::UnifiedRandom`):
//! 1. `AI_007_TryForcingSitting` (`:52895`): `Next(10800)` on the sit.
//! 2. The flee trigger (`:53569`): `Next(300)`; the partner's flee
//!    (`:53592`): `Next(120)` BEFORE the NPC's own (`:53600`):
//!    `Next(120)` — partner first, verbatim order.
//! 3. State 0: the wet-pet/duck flee `Next(500, 700)` (`:53644`), the
//!    go-home walk `Next(200)` (`:53688`), the drown flee `Next(300)`
//!    + the critter `Next(200, 400)` (`:53728-53733`), the idle→walk
//!    `Next(300)` (+critter `Next(200, 400)`, `:53746-53751`), the
//!    blocked flip `Next(120)` (`:53758`), the wander turn `Next(80)`
//!    (`:53782` — drawn ONLY when the 25-tile band test fails and the
//!    ±50 arms did not fire, i.e. the C#'s else-if position).
//! 4. State 1: the good-rest return `Next(200)` (`:53795`), the walk
//!    end `Next(300)` then `Next(100)` critter / `Next(900)` common
//!    (`:53820-53829`), the ledge bail `Next(50)` (`:54013`), the door
//!    gate `Next(10)` (`:54043` — drawn whenever the door tile matches,
//!    even in the rain, because the `|` is non-short-circuiting), the
//!    slime-air ledge `Next(50)` (`:54221`), the stuck flip `Next(300)`
//!    (+critter `Next(200, 400)`, `:54232-54237`).
//! 5. States 2/11 (`:54250`, `:54263`), 3/4/5/8/9/16/17/20-23
//!    (`:54292`, `:54294`), 6/7/18/19 (`:54323`, `:54325`): the cited
//!    `Next(60)` pairs.
//! 6. Attacks 10/12/14: the shot jitter `Utils.RandomVector2` = TWO
//!    `NextDouble` draws (`Utils.cs:2431-2434`), the Santa `Next(5)`
//!    ai0 (`:54504`), the Steampunker `SelectRandom` = `Next(3)`
//!    (`:54728`), the DyeTrader `Next(12)` (`:54776`), the Wizard's
//!    `SelectRandom(1,1,1,1,2,2,3)` = `Next(7)` + `NextDouble`×2 per
//!    bolt (`:54927-54931`), the Party Girl/Dryad spot retries =
//!    `NextDouble`×2 per try (`:54940-54946`/`:54956-54962`); the
//!    cooldown `Next(maxValue)` then `Next(maxValue)` again for
//!    localAI[1]/[3] (`:54519-54521`, `:54791-54793`, `:54989-54991`).
//! 7. Attack 13 (the Nurse heal): `Next(10)` + `Next(10)`
//!    (`:54821-54823`). Attack 15 (melee): `Next(maxValue4)` ×2
//!    (`:55116-55118`) only when the re-swing arm did not fire.
//! 8. State 24: no draws (`:55123-55140`).
//! 9. The social ladder (`:55197-55441`), in else-if order:
//!    `Next(300)` talk gate → on hit `Next(2)` + `Next(1,3)`|`Next(1,4)`
//!    (`:55231-55234`); `Next(1800)` gossip → `Next(2)` +
//!    `Next(1,3)`|`Next(1,4)` + `Next(4)` + `Next(3 - localAI[2])`
//!    (`:55258-55275`); `Next(1200)` party (gated on type/party),
//!    `Next(600)` bartender, `Next(1800)` small-talk → `45 * Next(1,2)`
//!    (`:55330` — `Next(1,2)` always draws 1), `Next(600)` tax-collector
//!    → `30 * Next(1,4)` (`:55336`), `Next(1200)` player-talk,
//!    `Next(num)` sit gate (num 300, dog 60 — pets only) →
//!    `Next(10800)` (`:55392`), `Next(600)` interact → `Next(90)`
//!    (`:55416`). Every gate DRAWS even when the earlier gate's body
//!    ran? NO — the else-if chain draws only the gates it reaches.
//! 10. The attack-chance ladder (`:55448-55544`): `Next(num118)` per
//!     attack type 0/1/2/3 whose `AttackType` matches (type 2/3 are
//!     separate `if`s, not else-ifs, `:55497`/`:55526`).
//! 11. The pet idle rolls (`:55752-55779`): `Next(petIdleChance)` →
//!     `Next(20, 20+num)` (num 0 for slimes: NO draw) → `Next(300)` →
//!     the per-pet `Next(200)`/`Next(100)`/`Next(200)`/`Next(240)`.
//! No other block draws: `TargetClosest` is RNG-free
//! (`NPC.cs:77652-77781`), the resting-spot/teleport/prediction scans
//! are pure tile reads, `UpdateHomeTileState` is RNG-free. The witness
//! tests below pin the stream position for each case (WRITTEN NOT RUN —
//! code-only mode).
#![allow(dead_code)] // phase 2 drains the TownOutputs seams (W4-b3 wired phase 1)

use crate::npc::{Npc, NpcKind};
use crate::npc_target::TargetCandidate;
use glam::Vec2;
use tr_core::tile::Tile;
use tr_core::tiledata as td;
use tr_core::{UnifiedRandom, World, TILE_PX};

// ---------------------------------------------------------------------
// The minimal town-kind seam
// ---------------------------------------------------------------------

/// `NPCID.Sets.IsTownPet` (`ID/NPCID.cs:10118`): 637/638/656/670/678-684.
pub fn is_town_pet(net_id: i32) -> bool {
    matches!(net_id, 637 | 638 | 656 | 670 | 678..=684)
}

/// `NPCID.Sets.IsTownSlime` (`ID/NPCID.cs:10119`): 670/678-684.
pub fn is_town_slime(net_id: i32) -> bool {
    matches!(net_id, 670 | 678..=684)
}

/// `NPCID.Sets.TownCritter` (`ID/NPCID.cs:10459`): the 49 town critters —
/// none has a ported kind; the fn exists for the arm's gates.
pub fn town_critter(net_id: i32) -> bool {
    matches!(
        net_id,
        46 | 148
            | 149
            | 230
            | 299
            | 300
            | 303
            | 337
            | 361
            | 362
            | 364
            | 366
            | 367
            | 443
            | 445
            | 447
            | 538
            | 539
            | 540
            | 583
            | 584
            | 585
            | 592
            | 593
            | 602
            | 607
            | 608
            | 610
            | 616
            | 617
            | 625
            | 626
            | 627
            | 639..=652
            | 687
            | 688
    )
}

/// `NPCID.Sets.CritterThatCanTurnOnPlayers` (`ID/NPCID.cs:10436`).
pub fn critter_turns_on_players(net_id: i32) -> bool {
    matches!(net_id, 299 | 538 | 539 | 639..=645)
}

/// `NPCID.Sets.ShimmerTownTransform` (`ID/NPCID.cs:10469`) — the
/// `IsShimmerVariant` gate (`NPC.cs:6713-6723`).
pub fn shimmer_town_transform(net_id: i32) -> bool {
    matches!(
        net_id,
        17 | 18
            | 19
            | 20
            | 22
            | 37
            | 38
            | 54
            | 107
            | 108
            | 124
            | 142
            | 160
            | 178
            | 207
            | 208
            | 209
            | 227
            | 228
            | 229
            | 353
            | 368
            | 369
            | 441
            | 453
            | 550
            | 588
            | 633
            | 663
    )
}

/// `NPCID.Sets.DangerDetectRange` (`ID/NPCID.cs:10466`), default −1.
pub fn danger_detect_range(net_id: i32) -> i32 {
    match net_id {
        38 | 107 | 18 | 369 | 453 => 300,
        17 => 320,
        19 | 178 | 368 => 900,
        22 | 54 | 108 | 160 | 663 => 700,
        124 | 228 | 227 => 800,
        229 | 209 => 1000,
        20 => 1200,
        207 | 353 => 60,
        208 => 400,
        142 | 441 => 50,
        633 => 100,
        550 | 588 => 120,
        637 | 638 | 656 | 670 | 678..=684 => 250,
        _ => -1,
    }
}

/// `NPCID.Sets.PrettySafe` (`ID/NPCID.cs:10474`), default −1: the flee
/// distance past which the danger no longer scares this NPC.
pub fn pretty_safe(net_id: i32) -> i32 {
    match net_id {
        19 | 228 | 178 | 229 | 209 => 300,
        22 | 124 | 20 | 368 | 227 => 200,
        54 | 108 | 160 => 100,
        _ => -1,
    }
}

/// `NPCID.Sets.AttackType` (`ID/NPCID.cs:10473`), default −1: 0 ranged
/// arc, 1 straight shot, 2 magic, 3 melee.
pub fn attack_type(net_id: i32) -> i32 {
    match net_id {
        38 | 17 | 107 | 124 | 18 | 369 | 453 | 208 | 142 | 633 | 550 | 588 => 0,
        19 | 22 | 228 | 178 | 229 | 209 | 368 | 227 => 1,
        54 | 108 | 160 | 20 | 663 => 2,
        207 | 441 | 353 => 3,
        _ => -1,
    }
}

/// `NPCID.Sets.AttackTime` (`ID/NPCID.cs:10471`), default −1.
pub fn attack_time(net_id: i32) -> i32 {
    match net_id {
        38 | 17 | 124 | 18 | 369 | 453 | 142 | 208 | 550 => 34,
        107 | 229 | 209 | 54 | 368 | 227 | 663 | 20 => {
            if net_id == 20 {
                600
            } else {
                60
            }
        }
        19 | 228 => 40,
        22 | 108 => 30,
        178 => 24,
        207 | 441 => 15,
        353 | 633 => 12,
        588 => 20,
        _ => -1,
    }
}

/// `NPCID.Sets.AttackAverageChance` (`ID/NPCID.cs:10472`), default 1.
pub fn attack_average_chance(net_id: i32) -> i32 {
    match net_id {
        38 | 368 | 229 | 550 => 40,
        17 | 19 | 22 | 124 | 108 | 54 | 453 | 227 => 30,
        107 | 160 | 20 => 60,
        228 | 178 | 369 | 208 | 142 => 50,
        18 => 60,
        209 => 30,
        588 => 20,
        207 | 441 | 353 | 633 | 663 => 1,
        _ => 1,
    }
}

/// `Main.tileDungeon` (`Main.cs:7992-7995`): 41/43/44/677.
pub fn tile_dungeon(kind: u16) -> bool {
    matches!(kind, 41 | 43 | 44 | 677)
}

/// `TileID.Sets.CanBeSatOnForNPCs` (`ID/TileID.cs:512`): chairs 15 and
/// beds 497.
fn can_be_sat_on(kind: u16) -> bool {
    kind == 15 || kind == 497
}

/// `TileID.Sets.IgnoredByNpcStepUp` (`ID/TileID.cs:511`): the furniture
/// the NPC StepUp's `specialChecksMode == 1` refuses to climb
/// (`Collision.cs:3694-3697`).
fn ignored_by_npc_step_up(kind: u16) -> bool {
    matches!(kind, 14 | 469 | 18 | 16 | 134)
}

/// `TileID.Sets.AvoidedByNPCs` (`ID/TileID.cs:601`).
fn avoided_by_npcs(kind: u16) -> bool {
    matches!(
        kind,
        21 | 467
            | 55
            | 85
            | 395
            | 88
            | 463
            | 334
            | 29
            | 97
            | 99
            | 356
            | 663
            | 425
            | 440
            | 209
            | 441
            | 468
            | 471
            | 491
            | 510
            | 511
            | 520
            | 573
            | 698
    )
}

/// `TileID.Sets.InteractableByNPCs` (`ID/TileID.cs:602`).
fn interactable_by_npcs(kind: u16) -> bool {
    matches!(
        kind,
        17 | 77
            | 133
            | 12
            | 665
            | 639
            | 26
            | 695
            | 35
            | 36
            | 55
            | 395
            | 471
            | 698
            | 21
            | 467
            | 29
            | 97
            | 88
            | 99
            | 463
            | 491
            | 33
            | 372
            | 174
            | 49
            | 646
            | 100
            | 173
            | 78
            | 79
            | 94
            | 96
            | 101
            | 50
            | 707
            | 103
            | 282
            | 106
            | 114
            | 125
            | 171
            | 172
            | 207
            | 215
            | 220
            | 219
            | 244
            | 228
            | 237
            | 247
            | 128
            | 269
            | 354
            | 355
            | 377
            | 287
            | 378
            | 390
            | 302
            | 405
            | 406
            | 411
            | 425
            | 209
            | 441
            | 468
            | 452
            | 454
            | 455
            | 457
            | 462
            | 470
            | 475
            | 494
            | 499
            | 505
            | 511
            | 510
            | 520
            | 543
            | 565
            | 573
            | 597
            | 598
            | 617
            | 621
            | 464
            | 642
            | 699
    )
}

/// The per-kind defaults the stat ladder reads (`defLifeMax`,
/// `defDefense`) — the kind table's rows, carried in the seam because
/// the port's `Npc` stores only the CURRENT values.
#[derive(Clone, Copy, Debug)]
pub struct TownKindStats {
    /// `NPC.defLifeMax` (`NPC.cs:6209`).
    pub def_life_max: i32,
    /// `NPC.defDefense` (`NPC.cs:6211`).
    pub def_defense: i32,
}

/// `GameDifficultyData.TownNPCDamageMultiplier`
/// (`DataStructures/GameDifficultyData.cs:88`): Journey 2 / Classic 1 /
/// Expert 1.5 / Legendary 2. `GetAttackDamage_ForTownNPC`
/// (`NPC.cs:7062-7065`) multiplies and truncates.
pub fn town_attack_damage(normal_damage: f32, difficulty_multiplier: f32) -> i32 {
    (normal_damage * difficulty_multiplier) as i32
}

// ---------------------------------------------------------------------
// The environment + outputs
// ---------------------------------------------------------------------

/// One live player slot as the town arm reads it — the
/// `npc_target::TargetCandidate` plus the fields the town arm needs
/// that the target seam does not carry.
#[derive(Clone, Copy, Debug)]
pub struct TownPlayer {
    pub candidate: TargetCandidate,
    /// `Player.talkNPC` (`NPC.cs:53319`) — the NPC index being talked to.
    pub talk_npc: i32,
    /// `Player.stinky` (buff 120) — the stink-potion flee target
    /// (`NPC.cs:53534`).
    pub stinky: bool,
    /// `Player.sitting.isSitting` (`NPC.cs:53374`).
    pub sitting: bool,
    /// `Player.CanBeTalkedTo` (`Player.cs:3888-3898`): active, alive,
    /// drawn, `stealth == 1f`.
    pub can_be_talked_to: bool,
}

/// One OTHER live NPC as the town arm reads it (danger scan, chatter,
/// the nurse heal, the sitting occupancy). The arm mutates partners
/// through [`TownPartnerWrite`] requests — the caller applies them to
/// the live list (the module never borrows the whole `Main.npc[]`).
#[derive(Clone, Copy, Debug)]
pub struct TownNpcView {
    pub net_id: i32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub vx: f32,
    pub vy: f32,
    pub ai: [f32; 4],
    pub wet: bool,
    pub friendly: bool,
    pub damage: i32,
    pub stinky: bool,
    pub chaseable: bool,
    pub dont_take_damage: bool,
    pub immortal: bool,
    pub no_tile_collide: bool,
    pub life: i32,
    pub life_max: i32,
    /// `aiStyle == 7 && townNPC` — the sitting-occupancy gates.
    pub town_ai7: bool,
    /// `NPC.CanBeTalkedTo` (`NPC.cs:6645-6655`).
    pub can_be_talked_to: bool,
    /// The door latch (`closeDoor`) — the chatter busy test
    /// (`NPC.cs:55240`).
    pub close_door: bool,
    /// Buff 165 (the Dryad's ward check reads `FindBuffIndex(165)`,
    /// `NPC.cs:55224`) — precomputed for the seam.
    pub has_buff_165: bool,
    /// `immune[Main.myPlayer] == 0` — the melee sweep gate
    /// (`NPC.cs:55072`).
    pub immune_to_me: bool,
}

impl TownNpcView {
    fn center(&self) -> (f32, f32) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }
    /// `Entity.Distance` (`Entity.cs:195-198`) — Euclidean centre.
    fn distance(&self, cx: f32, cy: f32) -> f32 {
        let (ox, oy) = self.center();
        ((ox - cx) * (ox - cx) + (oy - cy) * (oy - cy)).sqrt()
    }
    /// The `(Bottom + (0,-2)).ToTileCoordinates()` sit probe
    /// (`NPC.cs:52855`).
    fn sit_tile(&self) -> (i32, i32) {
        (
            (self.x + self.width / 2.0) as i32 / 16,
            (self.y + self.height - 2.0) as i32 / 16,
        )
    }
    /// `NPC.CanBeChasedBy` (`NPC.cs:90030-90041`) — the stock
    /// `LetProjectilesAimAtTargetDummies = false` path.
    fn can_be_chased_by(&self) -> bool {
        self.chaseable && self.life_max > 5 && !self.dont_take_damage && !self.friendly && !self.immortal
    }
}

/// The read-only per-tick inputs the C# arm takes from `Main` statics.
pub struct TownEnv<'a> {
    pub world: &'a World,
    /// `Main.dayTime` / `Main.raining` / `Main.eclipse` / `Main.slimeRain`.
    pub day_time: bool,
    pub raining: bool,
    pub eclipse: bool,
    pub slime_rain: bool,
    /// `Main.IsItStorming` (`NPC.cs:52995`).
    pub storming: bool,
    /// `Main.netMode` — the port runs the host path (0/2); 1 (client)
    /// skips every decision block, kept for the 1:1 shape.
    pub net_mode: i32,
    /// `Main.expertMode` / `Main.masterMode` (`NPC.cs:53000-53011`).
    pub expert_mode: bool,
    pub master_mode: bool,
    pub players: &'a [TownPlayer],
    /// The OTHER NPCs in `Main.npc[]` slot order, `who_am_i` excluded by
    /// the caller (the C# skips `m == base.whoAmI` itself).
    pub npcs: &'a [(usize, TownNpcView)],
    /// `Main.dungeonX/dungeonY` — the Old Man's home (`NPC.cs:53351`).
    pub dungeon_x: i32,
    pub dungeon_y: i32,
    /// `NPC.combatBookWasUsed` / `combatBookVolumeTwoWasUsed`
    /// (`NPC.cs:53018-53031`).
    pub combat_book: bool,
    pub combat_book_two: bool,
    /// `BirthdayParty.PartyIsUp` (`NPC.cs:55289`).
    pub party_up: bool,
    /// `WorldGen.Skyblock.lowTiles` (`NPC.cs:53139`) — the port has no
    /// skyblock worlds; carried for the shape.
    pub skyblock_low_tiles: bool,
    /// `Main.infectedSeed` (`NPC.cs:53132`) — the getfixedboi Dryad.
    pub infected_seed: bool,
    /// `Main.onlyShimmerOceanWorlds` (`NPC.cs:52972`).
    pub only_shimmer_ocean: bool,
    /// `Main.xMas` (`NPC.cs:53158`).
    pub x_mas: bool,
    /// `GameDifficultyData.TownNPCDamageMultiplier.Sample(Difficulty)`.
    pub town_damage_multiplier: f32,
    /// The world progression flags the stat ladder reads (`World`).
    pub downed_slime_king: bool,
    pub downed_boss1: bool,
    pub downed_deerclops: bool,
    pub downed_boss2: bool,
    pub downed_boss3: bool,
    pub downed_queen_bee: bool,
    pub hard_mode: bool,
    pub downed_queen_slime: bool,
    pub downed_mech1: bool,
    pub downed_mech2: bool,
    pub downed_mech3: bool,
    pub downed_plant: bool,
    pub downed_empress: bool,
    pub downed_fishron: bool,
    pub downed_golem: bool,
    pub downed_cultist: bool,
    /// `NPC.homeless` (`NPC.cs:6219`) — distinct from `home == None`
    /// in the C# (a homeless NPC keeps its last home tiles until
    /// UpdateHomeTileState rewrites them); the port's `Npc.home` folds
    /// the two, so the seam carries the flag for the `:53423` /
    /// `:53804` gates. The wiring latches it from the home updates.
    pub homeless: bool,
    /// `NPC.breath` (`NPC.cs:6247`) — the Nurse's heal gate
    /// (`:55422`); the stock full breath reads > 0.
    pub breath: i32,
    /// `NPC.shimmering` (`NPC.cs:92141`) — the `:53296` home-stamp gate.
    pub shimmering: bool,
    /// `Main.timeForVisualEffects` (`Main.cs:1181`, the double counter) —
    /// the state-25 emergence light's hue (`NPC.cs:53232`).
    pub time_for_visual_effects: f64,
}

/// W4-g90 — the OWNED per-tick town inputs the App town-tick feeder
/// (`world_sim::npcs::town_stat_tick`) builds: every `TownEnv`
/// field that is not a borrow and not per-NPC. The stat-ladder feeder
/// needs the world flags + the npc-side progression inputs WITHOUT the
/// borrowed player/NPC lists (the full AI wiring — the b3 lane — builds
/// those per slot); this struct owns the scalar half so the feeder can
/// hold it across the `&mut app.npcs` walk and lend a `TownEnv` per NPC.
#[derive(Clone, Debug)]
pub struct TownEnvFlags {
    pub day_time: bool,
    pub raining: bool,
    pub eclipse: bool,
    pub slime_rain: bool,
    pub storming: bool,
    pub net_mode: i32,
    pub expert_mode: bool,
    pub master_mode: bool,
    pub dungeon_x: i32,
    pub dungeon_y: i32,
    pub combat_book: bool,
    pub combat_book_two: bool,
    pub party_up: bool,
    pub skyblock_low_tiles: bool,
    pub infected_seed: bool,
    pub only_shimmer_ocean: bool,
    pub x_mas: bool,
    pub town_damage_multiplier: f32,
    pub downed_slime_king: bool,
    pub downed_boss1: bool,
    pub downed_deerclops: bool,
    pub downed_boss2: bool,
    pub downed_boss3: bool,
    pub downed_queen_bee: bool,
    pub hard_mode: bool,
    pub downed_queen_slime: bool,
    pub downed_mech1: bool,
    pub downed_mech2: bool,
    pub downed_mech3: bool,
    pub downed_plant: bool,
    pub downed_empress: bool,
    pub downed_fishron: bool,
    pub downed_golem: bool,
    pub downed_cultist: bool,
}

impl TownEnvFlags {
    /// The App town-tick feeder's world-flag half (the b3 phase-1
    /// `ai_007_town_family` env shape, scalar fields only): the live
    /// `World` flags, the unported statics folded false (the style31
    /// CLASSIC_SOLO convention — storming/party/skyblock/xMas/dungeon
    /// coordinates have no ported fields), the host net path and the
    /// Classic damage multiplier. `expert_mode` is the session player's
    /// buff mirror (`Main.expertMode`'s standing-in, the b3 feeder's
    /// `player.buffs.expert_mode` read).
    ///
    /// FEEDER-CANONICAL (W4-g90 retire, the b3 receipt — research/135
    /// §11): b3's phase-1 tip `96a5d88f0` builds its `TownEnv` INLINE in
    /// `npc.rs::ai_007_town_family` and folds `combat_book_two: false`
    /// ("volume two unported") — STALE on that branch: the g90 world
    /// flag (`World.combat_book_volume_two_was_used`) is the carrier.
    /// This builder reads BOTH books live; every other scalar agrees
    /// with b3's inline env field-for-field (the retire census). At the
    /// merge b3's arm builds its env through `from_world(...).env(...)`
    /// (the live lists + the per-NPC homeless seam ride `env`), and the
    /// arm's `town_stat_bonuses` call (`npc_ai_town.rs:1737`) dedupes
    /// onto the `update_npcs` feeder pre-pass — same inputs → same
    /// writes, RNG-free, idempotent (the reset-then-read base).
    pub fn from_world(world: &World, expert_mode: bool) -> Self {
        Self {
            day_time: world.day_time,
            raining: world.raining,
            eclipse: world.eclipse,
            slime_rain: world.slime_rain,
            storming: false,
            net_mode: 0,
            expert_mode,
            master_mode: false,
            dungeon_x: 0,
            dungeon_y: 0,
            combat_book: world.combat_book_was_used,
            combat_book_two: world.combat_book_volume_two_was_used,
            party_up: false,
            skyblock_low_tiles: false,
            infected_seed: world.seeds.contains(tr_core::seeds::SeedFlags::ZENITH),
            only_shimmer_ocean: false,
            x_mas: false,
            town_damage_multiplier: 1.0,
            downed_slime_king: world.downed_slime_king,
            downed_boss1: world.downed_eoc,
            downed_deerclops: world.downed_deerclops,
            downed_boss2: world.downed_eow,
            downed_boss3: world.downed_skeletron,
            downed_queen_bee: world.downed_queen_bee,
            hard_mode: world.hard_mode,
            downed_queen_slime: world.downed_queen_slime,
            downed_mech1: world.downed_mech_boss1,
            downed_mech2: world.downed_mech_boss2,
            downed_mech3: world.downed_mech_boss3,
            downed_plant: world.downed_plant_boss,
            downed_empress: false, // no ported flag yet (the b3 fold)
            downed_fishron: world.downed_fishron,
            downed_golem: world.downed_golem,
            downed_cultist: world.downed_ancient_cultist,
        }
    }

    /// Lend the borrowed [`TownEnv`] view for ONE NPC's tick: the world
    /// reference + this flag set + the per-NPC `homeless` seam (the
    /// port's `home == None` fold). The player/NPC lists are EMPTY — the
    /// stat ladder (`NPC.cs:52999-53157`) reads neither; the full AI
    /// wiring (b3) builds its own env with the live lists.
    pub fn env<'a>(
        &'a self,
        world: &'a World,
        players: &'a [TownPlayer],
        npcs: &'a [(usize, TownNpcView)],
        homeless: bool,
    ) -> TownEnv<'a> {
        TownEnv {
            world,
            day_time: self.day_time,
            raining: self.raining,
            eclipse: self.eclipse,
            slime_rain: self.slime_rain,
            storming: self.storming,
            net_mode: self.net_mode,
            expert_mode: self.expert_mode,
            master_mode: self.master_mode,
            players,
            npcs,
            dungeon_x: self.dungeon_x,
            dungeon_y: self.dungeon_y,
            combat_book: self.combat_book,
            combat_book_two: self.combat_book_two,
            party_up: self.party_up,
            skyblock_low_tiles: self.skyblock_low_tiles,
            infected_seed: self.infected_seed,
            only_shimmer_ocean: self.only_shimmer_ocean,
            x_mas: self.x_mas,
            town_damage_multiplier: self.town_damage_multiplier,
            downed_slime_king: self.downed_slime_king,
            downed_boss1: self.downed_boss1,
            downed_deerclops: self.downed_deerclops,
            downed_boss2: self.downed_boss2,
            downed_boss3: self.downed_boss3,
            downed_queen_bee: self.downed_queen_bee,
            hard_mode: self.hard_mode,
            downed_queen_slime: self.downed_queen_slime,
            downed_mech1: self.downed_mech1,
            downed_mech2: self.downed_mech2,
            downed_mech3: self.downed_mech3,
            downed_plant: self.downed_plant,
            downed_empress: self.downed_empress,
            downed_fishron: self.downed_fishron,
            downed_golem: self.downed_golem,
            downed_cultist: self.downed_cultist,
            homeless,
            breath: 200, // `NPC.breath` unported — the stock full breath
            shimmering: false,
        }
    }
}

/// `NPC.UpdateHomeTileState` (`NPC.cs:52904-52914`) — the home-tile
/// write plus the message-60 send when anything changed. The caller
/// applies it (the `TownRooms` book + the wire).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HomeTileUpdate {
    pub homeless: bool,
    pub x: i32,
    pub y: i32,
}

/// A partner-NPC mutation the arm requests (the C# writes `Main.npc[m]`
/// directly; the port drains these after the arm returns). `slot` is
/// the `Main.npc[]` index.
#[derive(Clone, Copy, Debug)]
pub enum TownPartnerWrite {
    /// The flee hand-off (`NPC.cs:53591-53596`): ai[0]=1, ai[1]=timer,
    /// ai[2]=0, localAI[3]=0, direction, netUpdate.
    Flee { slot: usize, timer: f32, direction: i32 },
    /// The talk latch (`NPC.cs:55249-55253` / `:55278-55284`): the
    /// partner's ai[0] (4 or 17), ai[1]=duration, ai[2]=my slot,
    /// direction, and for gossip localAI[2]=localAI[3]=0.
    Talk {
        slot: usize,
        state: f32,
        timer: f32,
        partner: f32,
        direction: i32,
        gossip: bool,
    },
}

/// A projectile request — `Projectile.NewProjectile` from the attack
/// bodies (`NPC.cs:54504`, `:54776`, `:54814`, `:54931`, `:54947`,
/// `:54963`, `:54970`, `:54976`). Every one carries `npcProj = true`,
/// `noDropItem = true`; `time_left_override` is the Golfer/Goblin
/// 480 (`:54507-54514`). `variation_index` is the trailing
/// `townNpcVariationIndex` argument the Mechanic/Wizard/Party Girl/
/// Princess/Dryad calls pass (`:54504`/`:54931`/`:54947`/`:54963`/
/// `:54970`); the other calls pass none (0).
#[derive(Clone, Copy, Debug)]
pub struct TownShot {
    pub kind: i32,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub damage: i32,
    pub knockback: f32,
    pub ai0: f32,
    pub ai1: f32,
    pub time_left_override: Option<i32>,
    pub variation_index: i32,
}

/// The melee sweep's hit request (attack 15, `NPC.cs:55069-55078`):
/// `StrikeNPCNoInteraction(damage, knockback, spriteDirection)` +
/// `immune[myPlayer] = ai[1] + 2` + netUpdate. The caller drains it
/// through the strike seam.
#[derive(Clone, Copy, Debug)]
pub struct TownStrike {
    pub slot: usize,
    pub damage: i32,
    pub knockback: f32,
    pub direction: i32,
    pub immune_ticks: i32,
}

/// Everything the arm produces besides the NPC's own field writes.
#[derive(Default)]
pub struct TownOutputs {
    pub home_updates: Vec<HomeTileUpdate>,
    pub partner_writes: Vec<TownPartnerWrite>,
    pub shots: Vec<TownShot>,
    pub strikes: Vec<TownStrike>,
    /// `NPC.savedGolfer` etc. (`AI_007_TownEntities_UpdateSavedStates`,
    /// `NPC.cs:52934-52963`) — the world flags the arm sets.
    pub saved_state: Option<SavedState>,
    /// `NPC.taxCollector = true` (`NPC.cs:53306`).
    pub tax_collector_present: bool,
    /// `NPC.ai[3] = 1` despawn requests (`NPC.cs:53337-53347`): the
    /// Old Man after Skeletron — `life = -1`, `active = false`.
    pub despawn: bool,
    /// The 666-tile stink buff (`TryAddingRepeatedBuff(120, 180)`,
    /// `NPC.cs:53419-53422`) — applied through `NpcBuffs` by the caller
    /// (the arm cannot borrow the buff engine mutably twice).
    pub repeated_stink_buff: bool,
    /// W4-c10 sweep (the W4-b3 co-lane seam) — the waterfowl takeoff
    /// (`NPC.cs:53180-53184`): the walker 362/364/602/608 falling fast
    /// (`velocity.Y > 4 || < -4`) or wet transforms into its swimming
    /// form (`Transform(this.type + 1)`, the default ai0..3 = 0) and
    /// the arm RETURNS. The arm cannot run `Transform` itself (the
    /// player seam + the kind table live on the caller), so the NEW
    /// vanilla type rides here (the `wants_transform` pattern); the
    /// caller drains it through `Npc::transform_to_kind`. `Transform`
    /// draws nothing and `TransformVisuals` is silent for the
    /// waterfowl pairs (the `npc_transform_visuals.rs` census).
    pub waterfowl_takeoff: Option<i32>,
//!   (`:55563-55641`, `ShimmerHelper.FindSpotWithoutShimmer`): the state
//!   is a cited skip (returns, draws shimmer dust — the dust task's);
//! * the Old Man 37 / Clothier 54 / Santa 142 (`:53158-53161` +
//!   the type arms), the Mechanic's owned-proj
    /// W4-f47 — the state-25 emergence visuals (`NPC.cs:53228-53248`),
    /// drained by the wiring onto the dust pool / entity lights (the
    /// module's RNG-free convention: the draws already rode `rng`).
    pub shimmer_fx: Vec<ShimmerFx>,
    /// W4-f47 — the state-25 completion's progression 43
    /// (`AchievementsHelper.NotifyProgressionEvent(43)`, `NPC.cs:53273`)
    /// when the flipped variation is 1.
    pub shimmer_town_progression: bool,
}

/// One state-25 visual request (`NPC.cs:53232-53247`). The positions /
/// colours / scales are computed in-arm (the draws already spent); the
/// wiring only spawns the dust / adds the light.
#[derive(Clone, Copy, Debug)]
pub enum ShimmerFx {
    /// `Lighting.AddLight(Center, hslToRgb(tve/360 % 1, 0.6, 0.65) *
    /// Remap(ai1, 30, 90, 0, 0.7))` (`:53232`) — the RGB is the byte
    /// product / 255 (the `Color.ToVector3` convention).
    Light { x: f32, y: f32, rgb: [f32; 3] },
    /// `Dust.NewDustPerfect(pos, 309, (0,-2).RotatedBy(num5*2pi*0.11),
    /// 0, default, 1.7 - |num5|*1.3)` (`:53235`).
    Dust { x: f32, y: f32, vx: f32, vy: f32, scale: f32 },
    /// `ParticleOrchestrator.RequestParticleSpawn(ShimmerBlock, ...)`
    /// (`:53242-53246`) — the particle system is unported; the request is
    /// recorded (position + movement vector) for the future seam.
    Particle { x: f32, y: f32, mx: f32, my: f32 },
}

/// `AI_007_TownEntities_UpdateSavedStates` (`NPC.cs:52934-52963`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SavedState {
    Golfer,
    TaxCollector,
    Goblin,
    Wizard,
    Mech,
    Stylist,
    Angler,
    Bartender,
}

/// The door/gate mutation seam — `WorldGen.OpenDoor` / `CloseDoor` /
/// `ShiftTallGate` (`NPC.cs:54047-54073` / `:53838-53854`). The arm
/// calls these INLINE (the C#'s attempt order and the success/failure
/// branches are tick-visible: the all-fail direction flip feeds the
/// social ladder later in the same tick), so the wiring implements
/// this trait over `tr_core::edit::{open_door, close_door}` + the
/// tall-gate port when it lands; the tests drive a recording fake.
/// The REAL sound-complete adapter is `world_sim::editing::doors::
/// AppTownDoors` (W4-g24 doc-sweep — the W4-b3 phase-2 seam): it plays
/// the pitched 8/9 cues on `UpdateNpcs` and spawns the cleanup drops.
///
/// ADOPTION STATUS (W4-g24 FINAL, re-read of b3 @ `96a5d88f`): b3 phase 2
/// has NOT landed — `npc.rs::ai_007_town_family` still passes its
/// branch-local `NoOpTownDoors` (every attempt false). Keep-one verdict:
/// `AppTownDoors` is the survivor (b3's NoOp is a placeholder, not a
/// competing door-sound impl — no cue overlap to reconcile). The adoption
/// is a one-binding swap at the b3 call site plus the caller re-shape
/// (the arm's `world`/`player` snapshot borrows fold into the adapter's
/// `&mut App`); the `&mut dyn TownDoorOps` object-safety + the one-cue-
/// per-success `UpdateNpcs` contract are pinned by
/// `doors.rs::w4_g24_the_b3_adoption_contract_is_dyn_compatible`.
pub trait TownDoorOps {
    /// `WorldGen.OpenDoor(x, y, direction)` (`WorldGen.cs:37630`).
    fn open_door(&mut self, x: i32, y: i32, direction: i32) -> bool;
    /// `WorldGen.CloseDoor(x, y)` (`WorldGen.cs:31810`).
    fn close_door(&mut self, x: i32, y: i32) -> bool;
    /// `WorldGen.ShiftTallGate(x, y, closing)` — PORTED in tr-core
    /// (W4-g24, `edit::shift_tall_gate`, `WorldGen.cs:51723-51775`); the
    /// real adapter (`AppTownDoors`) drives it with the pitched-8 cue.
    /// b3's phase-1 `NoOpTownDoors` still returns false until the town
    /// lane adopts the adapter (the cited skip: tall gates 388/389 never
    /// open/close through THAT seam, the arm takes the all-fail flip —
    /// the player interact and the zombie bash already drive the core
    /// function).
    /// `WorldGen.ShiftTallGate(x, y, closing)` — the canonical body is
    /// `tr_core::edit::shift_tall_gate` (W4-c42 `edit/shift.rs`, the
    /// research/95 §3 union-exec, landed on this lane for the net-19
    /// reader). The NPC-bash drain re-point (the real box-overlap
    /// closure, `wire_skips: None`, cues drained on UpdateNpcs) is g4's
    /// merge work — until it lands the wiring's impl returns false (the
    /// arm takes the all-fail flip).
    fn shift_tall_gate(&mut self, x: i32, y: i32, closing: bool) -> bool;
}

/// The door latch the caller keeps on the NPC (the C#'s
/// `closeDoor`/`doorX`/`doorY`, `NPC.cs:6511-6513`). W4-b3 registered
/// the fields on `Npc` (`town_door_close/x/y`); the aiStyle-7 adapter
/// copies them into this latch before the arm and back after.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TownDoorLatch {
    pub close_door: bool,
    pub door_x: i32,
    pub door_y: i32,
}

// ---------------------------------------------------------------------
// Small shared helpers
// ---------------------------------------------------------------------

fn in_world(world: &World, x: i32, y: i32) -> bool {
    x >= 0 && y >= 0 && x < world.width && y < world.height
}

/// `WorldGen.InWorld(x, y, fluff)` (`WorldGen.cs:9688-9695`).
fn in_world_fluff(world: &World, x: i32, y: i32, fluff: i32) -> bool {
    x >= fluff && x < world.width - fluff && y >= fluff && y < world.height - fluff
}

pub(crate) fn tile_at(world: &World, x: i32, y: i32) -> Option<&Tile> {
    world.tile(x, y)
}

/// `Main.tileSolid[type]` — the raw table read (the C#'s walk gates use
/// the RAW table, not `Collision`'s leaf-block exception).
pub(crate) fn raw_solid(kind: u16) -> bool {
    (kind as usize) < td::TILE_COUNT && td::TILE_SOLID.has(kind)
}

pub(crate) fn raw_solid_top(kind: u16) -> bool {
    (kind as usize) < td::TILE_COUNT && td::TILE_SOLID_TOP.has(kind)
}

/// `tile.nactive()` — active and not actuated.
pub(crate) fn nactive(t: &Tile) -> bool {
    t.active() && !t.inactive()
}

/// `tile.topSlope()` — slope 1 or 2.
pub(crate) fn top_slope(t: &Tile) -> bool {
    let s = t.slope();
    s == 1 || s == 2
}

/// `Collision.SolidTiles(startX, endX, startY, endY)`
/// (`Collision.cs:3455-3488`): out-of-bounds counts as SOLID, a null
/// tile returns false, the per-tile test is
/// `active && !inActive && tileSolid && !tileSolidTop`.
pub(crate) fn solid_tiles(world: &World, x0: i32, x1: i32, y0: i32, y1: i32) -> bool {
    if x0 < 0 || x1 >= world.width || y0 < 0 || y1 >= world.height - 40 {
        return true;
    }
    for x in x0..=x1 {
        for y in y0..=y1 {
            let Some(t) = world.tile(x, y) else {
                return false;
            };
            if nactive(t) && raw_solid(t.kind) && !raw_solid_top(t.kind) {
                return true;
            }
        }
    }
    false
}

/// `Collision.SolidTilesVersatile` (`Collision.cs:3437-3448`): swap the
/// reversed bounds, then `SolidTiles`.
pub(crate) fn solid_tiles_versatile(world: &World, x0: i32, x1: i32, y0: i32, y1: i32) -> bool {
    let (x0, x1) = if x0 > x1 { (x1, x0) } else { (x0, x1) };
    let (y0, y1) = if y0 > y1 { (y1, y0) } else { (y0, y1) };
    solid_tiles(world, x0, x1, y0, y1)
}

/// `WorldGen.SolidTile(x, y)` (`WorldGen.cs`'s `SolidTile(int, int)`):
/// the tile is active, not actuated, solid-or-solid-top.
fn worldgen_solid_tile(world: &World, x: i32, y: i32) -> bool {
    tile_at(world, x, y).is_some_and(|t| nactive(t) && (raw_solid(t.kind) || raw_solid_top(t.kind)))
}

/// `NPC.SolidOrSlopedTileOrPlatform` (`NPC.cs:52786-52793`):
/// `(tileSolid && !tileSolidTop) || Platforms`, active, not actuated.
fn solid_or_sloped_or_platform(t: &Tile) -> bool {
    if !t.active() {
        return false;
    }
    let solid = raw_solid(t.kind) && !raw_solid_top(t.kind);
    let platform = td::SET_PLATFORMS.has(t.kind);
    (solid || platform) && !t.inactive()
}

/// `Utils.PlotLine` (`Utils.cs:2238-2292`) — the Bresenham walk with
/// the `jump: false` double-plot on the minor-axis step. `plot` returns
/// false to abort (the C# `TileActionAttempt` convention).
fn plot_line(x0: i32, y0: i32, x1: i32, y1: i32, jump: bool, plot: &mut dyn FnMut(i32, i32) -> bool) -> bool {
    if x0 == x1 && y0 == y1 {
        return plot(x0, y0);
    }
    let mut x0 = x0;
    let mut y0 = y0;
    let mut x1 = x1;
    let mut y1 = y1;
    let flag = (y1 - y0).abs() > (x1 - x0).abs();
    if flag {
        std::mem::swap(&mut x0, &mut y0);
        std::mem::swap(&mut x1, &mut y1);
    }
    let num = (x1 - x0).abs();
    let num2 = (y1 - y0).abs();
    let mut num3 = num / 2;
    let mut num4 = y0;
    let num5 = if x0 < x1 { 1 } else { -1 };
    let num6 = if y0 < y1 { 1 } else { -1 };
    let mut i = x0;
    while i != x1 {
        if flag {
            if !plot(num4, i) {
                return false;
            }
        } else if !plot(i, num4) {
            return false;
        }
        num3 -= num2;
        if num3 >= 0 {
            i += num5;
            continue;
        }
        num4 += num6;
        if !jump {
            if flag {
                if !plot(num4, i) {
                    return false;
                }
            } else if !plot(i, num4) {
                return false;
            }
        }
        num3 += num;
        i += num5;
    }
    true
}

/// `DelegateMethods.SearchAvoidedByNPCs` (`DelegateMethods.cs:896-911`):
/// out-of-world/null → false (abort), inactive or not-avoided → true
/// (keep plotting), avoided → false.
fn search_avoided_by_npcs(world: &World, x: i32, y: i32) -> bool {
    if !in_world_fluff(world, x, y, 1) {
        return false;
    }
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    !t.active() || !avoided_by_npcs(t.kind)
}

/// `Utils.PlotTileLine(Top, Bottom, width, SearchAvoidedByNPCs)`
/// (`Utils.cs:2337-2350`) — the vertical body line with the half-width
/// perpendicular offset. For the town NPC's vertical Top→Bottom line
/// the unit direction is (0,1), so the perpendicular is (−1,0)·w/2:
/// the offsets are pure ±w/2 in X.
fn body_line_avoids(world: &World, x: f32, y: f32, w: f32, h: f32) -> bool {
    let half = (w / 2.0) as f64;
    let start = (x + w / 2.0, y);
    let end = (x + w / 2.0, y + h);
    let p_min = ((start.0 as f64 - half) as i32 / 16, (start.1 as f64) as i32 / 16);
    let p_max = ((start.0 as f64 + half) as i32 / 16, (start.1 as f64) as i32 / 16);
    let p_start = ((start.0 as f64) as i32 / 16, (start.1 as f64) as i32 / 16);
    let p_end = ((end.0 as f64) as i32 / 16, (end.1 as f64) as i32 / 16);
    let min_off = (p_min.0 - p_start.0, p_min.1 - p_start.1);
    let max_off = (p_max.0 - p_start.0, p_max.1 - p_start.1);
    plot_line(p_start.0, p_start.1, p_end.0, p_end.1, true, &mut |px, py| {
        plot_line(px + min_off.0, py + min_off.1, px + max_off.0, py + max_off.1, false, &mut |tx, ty| {
            search_avoided_by_npcs(world, tx, ty)
        })
    })
}

/// `Collision.CanHit` centre walk — the shared projectile seam
/// (`projectile.rs::can_hit`, `Collision.cs`'s centre-to-centre walk).
fn can_hit_centers(world: &World, ax: f32, ay: f32, bx: f32, by: f32) -> bool {
    crate::projectile::can_hit(world, ax, ay, 0.0, 0.0, bx, by, 0.0, 0.0)
}

/// `Collision.CanHitLine` — the shared seam.
fn can_hit_line_centers(world: &World, ax: f32, ay: f32, bx: f32, by: f32) -> bool {
    crate::projectile::can_hit_line(world, ax, ay, 0.0, 0.0, bx, by, 0.0, 0.0)
}

/// `Collision.DrownCollision(position, width, height, 1f, true)` — the
/// shared collision port (`collision.rs::drown_collision`).
pub(crate) fn drown_collision(world: &World, x: f32, y: f32, w: f32, h: f32, include_slopes: bool) -> bool {
    crate::collision::drown_collision(world, x, y, w as i32, h as i32, 1.0, include_slopes)
}

/// `Collision.StepDown` — the shared port.
pub(crate) fn step_down(world: &World, npc: &mut Npc) {
    let (w, h) = npc.size();
    let mut pos = [npc.x, npc.y];
    let mut vel = [npc.vx, npc.vy];
    crate::collision::step_down(world, &mut pos, &mut vel, w, h, &mut npc.step_speed, &mut npc.gfx_off_y);
    npc.x = pos[0];
    npc.y = pos[1];
    npc.vx = vel[0];
    npc.vy = vel[1];
}

/// `Collision.StepUp(..., gravDir: 1, holdsMatching: flag23,
/// specialChecksMode: 1)` (`NPC.cs:53965`). The shared
/// `collision::step_up` ports the mode-0 shape; the town call's mode 1
/// adds the `IgnoredByNpcStepUp` gate on the platform climb
/// (`Collision.cs:3694-3697`: `flag5 = !IgnoredByNpcStepUp[tile.type]`,
/// folded into the `holdsMatching` conjunct). The port wraps the shared
/// call: when the support tile is in the ignored set the holds-matching
/// climb is refused by passing `holds_matching = false` — the C#'s
/// `flag5` only gates the platform/solid-top half of `flag4`, and the
/// genuinely-solid half never consults it, so for a solid support the
/// two forms agree; for a platform/380 support in the ignored set the
/// C# refuses and so does this wrapper. Tile 380 (the planter box) is
/// not in `SET_PLATFORMS` — the shared port's holds set omits it, a
/// recorded residual (ANALYSIS "The town-NPC aiStyle-7 re-port").
pub(crate) fn town_step_up(world: &World, npc: &mut Npc, holds_matching: bool) {
    let (w, h) = npc.size();
    // The support tile the shared StepUp probes (`Collision.cs:3640-3650`).
    let num = if npc.vx < 0.0 { -1 } else if npc.vx > 0.0 { 1 } else { 0 };
    let vx = npc.x + npc.vx;
    let probe_x = ((vx + w * 0.5 + (w * 0.5 + 1.0) * num as f32) / TILE_PX) as i32;
    let probe_y = ((npc.y + h - 1.0) / TILE_PX) as i32;
    let ignored = tile_at(world, probe_x, probe_y).is_some_and(|t| ignored_by_npc_step_up(t.kind));
    let mut pos = [npc.x, npc.y];
    let mut vel = [npc.vx, npc.vy];
    crate::collision::step_up(
        world,
        &mut pos,
        &mut vel,
        w,
        h,
        &mut npc.step_speed,
        &mut npc.gfx_off_y,
        holds_matching && !ignored,
    );
    npc.x = pos[0];
    npc.y = pos[1];
    npc.vx = vel[0];
    npc.vy = vel[1];
}

/// `Utils.RandomVector2(Main.rand, min, max)` (`Utils.cs:2431-2434`):
/// TWO `NextDouble` draws, X then Y.
fn random_vector2(rng: &mut UnifiedRandom, min: f32, max: f32) -> (f32, f32) {
    let x = (max - min) * rng.next_double() as f32 + min;
    let y = (max - min) * rng.next_double() as f32 + min;
    (x, y)
}

/// `Utils.SelectRandom<T>(Main.rand, params T[])` — `Next(objs.Length)`
/// then the pick (`Utils.cs`'s SelectRandom; the port's
/// `UnifiedRandom::next_from_slice` draws the same single `Next`).
fn select_random_index(rng: &mut UnifiedRandom, len: i32) -> i32 {
    rng.next_max(len)
}

/// `Math.Sign` for the aim guards.
fn sign_f(v: f32) -> i32 {
    if v > 0.0 {
        1
    } else if v < 0.0 {
        -1
    } else {
        0
    }
}

/// `Utils.ToDirectionInt` — true → 1, false → −1.
fn to_direction_int(b: bool) -> i32 {
    if b {
        1
    } else {
        -1
    }
}

/// `Entity.DirectionTo` — the normalized vector toward the point;
/// (0,0) when coincident (`Vector2.Normalize` of zero stays zero).
fn direction_to(ax: f32, ay: f32, bx: f32, by: f32) -> (f32, f32) {
    let dx = bx - ax;
    let dy = by - ay;
    let len = (dx * dx + dy * dy).sqrt();
    if len == 0.0 {
        (0.0, 0.0)
    } else {
        (dx / len, dy / len)
    }
}

fn rect_intersects(a: (f32, f32, f32, f32), b: (f32, f32, f32, f32)) -> bool {
    a.0 < b.0 + b.2 && a.0 + a.2 > b.0 && a.1 < b.1 + b.3 && a.1 + a.3 > b.1
}

// ---------------------------------------------------------------------
// The resting-spot family (NPC.cs:52765-52902)
// ---------------------------------------------------------------------

/// `AI_007_TownEntities_IsInAGoodRestingSpot` (`NPC.cs:52765-52784`).
pub fn is_in_a_good_resting_spot(
    net_id: i32,
    ai0: f32,
    wet: bool,
    day_time: bool,
    tile_x: i32,
    tile_y: i32,
    ideal_x: i32,
    ideal_y: i32,
) -> bool {
    if !day_time && ai0 == 5.0 {
        // `:52767-52774` — the sitting NPC's ±7 window.
        return (tile_x - ideal_x).abs() <= 7 && (tile_y - ideal_y).abs() <= 7;
    }
    // `:52775-52778` — the water critters never rest wet.
    if matches!(net_id, 361 | 445 | 687) && wet {
        return false;
    }
    tile_x == ideal_x && tile_y == ideal_y
}

/// `AI_007_FindGoodRestingSpot` (`NPC.cs:52795-52862`): drop from the
/// home tile to the first support, then at night (when not already
/// sitting near it) search the ±7×(+2/−6) chair/bed window for the
/// closest seat by Manhattan distance, adjust the chair/bed rows, and
/// refuse a seat another sitting NPC occupies.
pub fn find_good_resting_spot(
    world: &World,
    npc: &Npc,
    env_npcs: &[(usize, TownNpcView)],
    my_tile_x: i32,
    my_tile_y: i32,
    day_time: bool,
) -> (i32, i32) {
    let net_id = npc.kind.net_id();
    let Some((mut floor_x, mut floor_y)) = npc.home else {
        // `homeTileX == -1 || homeTileY == -1` (`:52799-52802`) — the
        // port models homeless as `home == None`.
        return (-1, -1);
    };
    // `:52803-52806` — drop to the first solid/sloped/platform tile.
    while floor_y < world.height - 20
        && !tile_at(world, floor_x, floor_y).is_some_and(solid_or_sloped_or_platform)
    {
        floor_y += 1;
    }
    // `:52807-52810` — daytime, or already sitting within 7 tiles.
    if day_time || (npc.ai[0] == 5.0 && (my_tile_x - floor_x).abs() < 7 && (my_tile_y - floor_y).abs() < 7) {
        return (floor_x, floor_y);
    }
    // `:52814-52817` — the dog/bunny/slime/sitting NPCs never seek a seat.
    if net_id == 638 || net_id == 656 || is_town_slime(net_id) || npc.ai[0] == 5.0 {
        return (floor_x, floor_y);
    }
    // `:52818-52839` — the window scan: x ±7 stride 1, y from +2 down
    // to −6 stride 2 (the C# walks num7 DESCENDING).
    let mut best_dist = -1i32;
    let mut best = (-1i32, -1i32);
    let mut i = floor_x - 7;
    while i <= floor_x + 7 {
        let mut j = floor_y + 2;
        while j >= floor_y - 6 {
            if let Some(t) = tile_at(world, i, j) {
                // `:52828` — active, sittable, and the frameY%40 gate:
                // a non-top frame only counts in the bottom two rows.
                if t.active()
                    && can_be_sat_on(t.kind)
                    && (i32::from(t.frame_y) % 40 != 0 || j + 1 <= floor_y + 2)
                {
                    let dist = (i - floor_x).abs() + (j - floor_y).abs();
                    if best_dist == -1 || dist < best_dist {
                        best_dist = dist;
                        best = (i, j);
                    }
                }
            }
            j -= 2;
        }
        i += 1;
    }
    if best_dist == -1 {
        return (floor_x, floor_y);
    }
    // `:52844-52852` — the bed/chair row adjust.
    if let Some(t) = tile_at(world, best.0, best.1) {
        if t.kind == 497 || t.kind == 15 {
            if i32::from(t.frame_y) % 40 != 0 {
                best.1 -= 1;
            }
            best.1 += 2;
        }
    }
    // `:52853-52859` — a sitting NPC on the seat refuses it.
    for (_, other) in env_npcs {
        if other.town_ai7 && other.ai[0] == 5.0 && other.sit_tile() == best {
            return (floor_x, floor_y);
        }
    }
    best
}

/// `AI_007_TryForcingSitting` (`NPC.cs:52864-52902`): when the home
/// floor's tile-above is a chair/bed (not the forbidden chair frames
/// 1080-1098) and no NPC sits on the NPC's own seat probe, sit:
/// ai[0]=5, ai[1]=900+Next(10800), direction from the frame, Bottom
/// snapped to the seat, velocity zero, localAI[3]=0.
/// Returns whether the sit happened (the caller's netUpdate rides it).
pub fn try_forcing_sitting(
    world: &World,
    npc: &mut Npc,
    env_npcs: &[(usize, TownNpcView)],
    home_floor_x: i32,
    home_floor_y: i32,
    rng: &mut UnifiedRandom,
) -> bool {
    if !in_world(world, home_floor_x, home_floor_y) {
        return false;
    }
    let net_id = npc.kind.net_id();
    let Some(tile) = tile_at(world, home_floor_x, home_floor_y - 1) else {
        return false;
    };
    // `:52871-52879` — the kind gates + the chair/bed type + the frame
    // exclusion (chair frames 1080..=1098 are the "someone sits here"
    // variants the NPC may not force).
    let mut flag = net_id != 638 && net_id != 656 && !is_town_slime(net_id) && npc.ai[0] != 5.0;
    if flag {
        flag &= tile.active() && (tile.kind == 15 || tile.kind == 497);
    }
    if flag {
        flag &= tile.kind != 15 || !(1080..=1098).contains(&i32::from(tile.frame_y));
    }
    let (w, h) = npc.size();
    let my_seat = ((npc.x + w / 2.0) as i32 / 16, (npc.y + h - 2.0) as i32 / 16);
    if flag {
        for (_, other) in env_npcs {
            if other.town_ai7 && other.ai[0] == 5.0 && other.sit_tile() == my_seat {
                flag = false;
                break;
            }
        }
    }
    if !flag {
        return false;
    }
    // `:52894-52900`.
    npc.ai[0] = 5.0;
    npc.ai[1] = 900.0 + rng.next_max(10800) as f32;
    npc.facing = if tile.frame_x != 0 { 1 } else { -1 };
    // `base.Bottom = (homeFloorX*16 + 8 + 2*direction, homeFloorY*16)`.
    npc.x = home_floor_x as f32 * 16.0 + 8.0 + 2.0 * npc.facing as f32 - w / 2.0;
    npc.y = home_floor_y as f32 * 16.0 - h;
    npc.vx = 0.0;
    npc.vy = 0.0;
    npc.local_ai[3] = 0.0;
    npc.net_update = true;
    true
}

/// `AI_007_TownEntities_TeleportToHome` (`NPC.cs:55643-55671`): try the
/// home column, then −1, then +1; the first whose 3×3 body space above
/// the floor is clear takes the NPC (velocity zero, position snapped,
/// then TryForcingSitting). All three blocked → homeless +
/// QuickFindHome (the housing seam's, requested via the return).
/// Returns true when a spot was taken; false asks the caller to run
/// `quick_find_home` (the `town.rs` seam — NOT re-run here).
pub fn teleport_to_home(
    world: &World,
    npc: &mut Npc,
    env_npcs: &[(usize, TownNpcView)],
    home_floor_x: i32,
    home_floor_y: i32,
    rng: &mut UnifiedRandom,
) -> bool {
    let net_id = npc.kind.net_id();
    let (w, h) = npc.size();
    for i in 0..3 {
        // `:55648-55653` — the switch: i=1 → −1, i=0 → 0, else +1.
        let num = home_floor_x + match i {
            1 => -1,
            0 => 0,
            _ => 1,
        };
        if net_id == 37 || !solid_tiles(world, num - 1, num + 1, home_floor_y - 3, home_floor_y - 1) {
            npc.vx = 0.0;
            npc.vy = 0.0;
            npc.x = num as f32 * 16.0 + 8.0 - w / 2.0;
            npc.y = (home_floor_y as f32 * 16.0 - h) - 0.1;
            npc.net_update = true;
            try_forcing_sitting(world, npc, env_npcs, home_floor_x, home_floor_y, rng);
            return true;
        }
    }
    false
}

/// `AI_007_TownEntities_GetWalkPrediction` (`NPC.cs:55673-55743`):
/// keepwalking (the avoid-line + the stationary-friendly crowd + the
/// drowning override) and avoidFalling (the home-band/critter
/// exemption, then the 6-tile column probe: liquid depth, lava, the
/// first solid, the ceiling-of-height drown test).
pub fn get_walk_prediction(
    world: &World,
    npc: &Npc,
    env_npcs: &[(usize, TownNpcView)],
    my_tile_x: i32,
    home_floor_x: i32,
    can_breath_under_water: bool,
    currently_drowning: bool,
    tile_x: i32,
    tile_y: i32,
) -> (bool, bool) {
    let net_id = npc.kind.net_id();
    let (w, h) = npc.size();
    let mut keepwalking = false;
    let mut avoid_falling = true;
    let in_home_band = my_tile_x >= home_floor_x - 35 && my_tile_x <= home_floor_x + 35;
    // `:55678-55695` — the isLikeATownNPC && ai[1] < 30 crowd test.
    if npc.kind.is_town_npc_like() && npc.ai[1] < 30.0 {
        keepwalking = !body_line_avoids(world, npc.x, npc.y, w, h);
        if !keepwalking {
            // The hitbox inflated ±20 in X; a stationary friendly NPC
            // intersecting it keeps the walk going.
            let hitbox = (npc.x - 20.0, npc.y, w + 40.0, h);
            for (_, other) in env_npcs {
                if other.friendly && other.vx == 0.0 && rect_intersects(hitbox, (other.x, other.y, other.width, other.height)) {
                    keepwalking = true;
                    break;
                }
            }
        }
    }
    // `:55696-55699`.
    if !keepwalking && currently_drowning {
        keepwalking = true;
    }
    // `:55700-55703` — critters and home-bound walks may fall.
    if avoid_falling && (town_critter(net_id) || (!in_home_band && npc.facing == sign_f((home_floor_x - my_tile_x) as f32))) {
        avoid_falling = false;
    }
    if !avoid_falling {
        return (keepwalking, false);
    }
    // `:55708-55733` — the column probe, j = −1..=4.
    let mut lava = false;
    let mut liquid_count = 0i32;
    let mut p = (0i32, 0i32);
    for j in -1..=4 {
        let Some(t) = tile_at(world, tile_x, tile_y + j) else {
            continue; // Framing.GetTileSafely clamps; a null reads empty
        };
        if t.liquid > 0 {
            liquid_count += 1;
            if t.flags & tr_core::tile::FLAG_LIQUID_LAVA != 0 {
                lava = true;
                break;
            }
        }
        if nactive(t) && raw_solid(t.kind) {
            if liquid_count > 0 {
                p = (tile_x, tile_y + j);
            }
            avoid_falling = false;
            break;
        }
    }
    avoid_falling |= lava;
    // `:55735-55738` — the liquid column at least the NPC's height.
    if (liquid_count as f64) >= (h / 16.0).ceil() as f64 {
        avoid_falling = true;
    }
    // `:55739-55742` — the drown probe at the liquid surface.
    if !avoid_falling && p.0 != 0 && p.1 != 0 {
        avoid_falling = crate::collision::drown_collision(
            world,
            p.0 as f32 * 16.0 + 8.0 - w / 2.0,
            p.1 as f32 * 16.0 - h,
            w as i32,
            h as i32,
            1.0,
            false,
        );
    }
    (keepwalking, avoid_falling)
}

/// `AI_007_TownEntities_CheckIfWillDrown` (`NPC.cs:55745-55748`) — the
/// identity wrapper the C# keeps (a future hook); ported verbatim.
pub fn check_if_will_drown(currently_drowning: bool) -> bool {
    currently_drowning
}

/// `AI_007_AttemptToPlayIdleAnimationsForPets` (`NPC.cs:55750-55785`):
/// the stationary pet's idle roll. Pure except the RNG draws; returns
/// whether the idle started.
pub fn pet_idle_animation(npc: &mut Npc, pet_idle_chance: i32, net_mode: i32, rng: &mut UnifiedRandom) -> bool {
    let net_id = npc.kind.net_id();
    if npc.vx != 0.0 || net_mode == 1 || rng.next_max(pet_idle_chance) != 0 {
        return false;
    }
    // `:55754-55762` — the animation count: dog 2, slimes 0, else 3.
    let mut num = 3;
    if net_id == 638 {
        num = 2;
    }
    if is_town_slime(net_id) {
        num = 0;
    }
    // `:55763` — slimes take state 20 with NO draw.
    npc.ai[0] = if num == 0 { 20.0 } else { rng.next_range(20, 20 + num) as f32 };
    npc.ai[1] = 200.0 + rng.next_max(300) as f32;
    if npc.ai[0] == 20.0 && net_id == 637 {
        npc.ai[1] = 500.0 + rng.next_max(200) as f32;
    }
    if npc.ai[0] == 21.0 && net_id == 638 {
        npc.ai[1] = 100.0 + rng.next_max(100) as f32;
    }
    if npc.ai[0] == 22.0 && net_id == 656 {
        npc.ai[1] = 200.0 + rng.next_max(200) as f32;
    }
    if npc.ai[0] == 20.0 && is_town_slime(net_id) {
        npc.ai[1] = 180.0 + rng.next_max(240) as f32;
    }
    npc.ai[2] = 0.0;
    npc.local_ai[3] = 0.0;
    npc.net_update = true;
    true
}

// ---------------------------------------------------------------------
// The stat ladder (NPC.cs:52999-53157)
// ---------------------------------------------------------------------

/// The `isLikeATownNPC` stat block (`NPC.cs:53014-53157`): the lifeMax
/// reset + the combat-book/boss-downed progression (num2 damage scale,
/// num3 cooldown scale, defense adds), the tipsy multipliers, the
/// skyblock fold, the Steampunker's /3. Returns `(num2, num3)` and
/// writes lifeMax/life/defense/defDefense through the NPC + the seam's
/// defaults. `dryad_ward` is the buff-165 latch (`NPC.cs:92075`).
#[allow(clippy::too_many_arguments)]
pub fn town_stat_bonuses(
    npc: &mut Npc,
    stats: TownKindStats,
    env: &TownEnv<'_>,
    dryad_ward: bool,
    tipsy: bool,
) -> (f32, f32) {
    let net_id = npc.kind.net_id();
    let mut num2 = 1.0f32;
    let mut num3 = 2.0f32;
    // `:53000-53011` — the difficulty defense (the ward adds).
    let ward_add = if env.master_mode {
        20
    } else if env.expert_mode {
        15
    } else {
        10
    };
    let mut defense = stats.def_defense + if dryad_ward { ward_add } else { 0 };
    if !npc.kind.is_town_npc_like() {
        // The block is isLikeATownNPC-gated; the non-town aiStyle-7
        // users (none ported) skip it. The defense write above still
        // ran in the C# (`:53000-53011` precedes the gate).
        npc.defense_override = Some(defense);
        return (num2, num3);
    }
    // `:53016-53035` — the lifeMax reset + the two combat books.
    let flag3 = npc.hp == npc.max_hp;
    npc.max_hp = stats.def_life_max;
    if env.combat_book {
        npc.max_hp += 250;
        num3 *= 0.8;
        num2 += 0.25;
        defense += 8;
    }
    if env.combat_book_two {
        npc.max_hp += 250;
        num3 *= 0.8;
        num2 += 0.25;
        defense += 8;
    }
    // `:53032-53035` — the `&` (non-short-circuit) full-heal on growth.
    if (npc.max_hp > stats.def_life_max) & flag3 {
        npc.hp = npc.max_hp;
    }
    // `:53036-53131` — the boss ladder, in C# order.
    let mut boss = |downed: bool, d2: f32, def: i32| {
        if downed {
            num3 *= 0.985;
            num2 += d2;
            defense += def;
        }
    };
    boss(env.downed_slime_king, 0.05, 2);
    boss(env.downed_boss1, 0.05, 2);
    boss(env.downed_deerclops, 0.1, 3);
    boss(env.downed_boss2, 0.1, 3);
    boss(env.downed_boss3, 0.1, 3);
    boss(env.downed_queen_bee, 0.1, 3);
    boss(env.hard_mode, 0.4, 12);
    boss(env.downed_queen_slime, 0.15, 6);
    boss(env.downed_mech1, 0.15, 6);
    boss(env.downed_mech2, 0.15, 6);
    boss(env.downed_mech3, 0.15, 6);
    boss(env.downed_plant, 0.15, 8);
    boss(env.downed_empress, 0.15, 8);
    boss(env.downed_fishron, 0.15, 8);
    boss(env.downed_golem, 0.15, 8);
    boss(env.downed_cultist, 0.15, 20);
    // `:53132-53138` — the infected-seed Dryad: flag2 (the flee-TOWARD
    // marker, returned via the caller's env read), full heal, immortal,
    // defDefense = 99999. The defDefense write is a PERSISTENT defaults
    // change the read-only seam cannot carry — the wiring applies it to
    // the kind table; this tick's defense keeps the ladder value (the
    // C# wrote defense at :53002 BEFORE this block, verbatim).
    if env.infected_seed && net_id == 20 {
        npc.hp = npc.max_hp;
        npc.immortal = true;
    }
    // `:53139-53146` — the skyblock fold.
    if env.net_mode != 1 && env.skyblock_low_tiles {
        if num3 > 1.0 {
            num3 = (num3 + 1.0) / 2.0;
        }
        num2 += 0.5;
    }
    // `:53147-53150` — the Steampunker's cooldown /3.
    if net_id == 209 {
        num3 /= 3.0;
    }
    // `:53151-53156` — tipsy: defense ×1.1 truncated, num2 ×1.1,
    // num3 ×0.9.
    if tipsy {
        defense = (defense as f64 * 1.1) as i32;
        num2 *= 1.1;
        num3 *= 0.9;
    }
    npc.defense_override = Some(defense);
    (num2, num3)
}

// ---------------------------------------------------------------------
// The main arm
// ---------------------------------------------------------------------

/// `AI_007_TownEntities` (`NPC.cs:52965-55561`) — the whole town-NPC
/// tick. `door` is the caller-carried door latch (the C#'s
/// closeDoor/doorX/doorY fields), `stats` the kind defaults, `who_am_i`
/// the NPC's `Main.npc[]` slot. The generic UpdateNPC tail (gravity,
/// integration, collision) is the caller's — this fn never integrates.
#[allow(clippy::too_many_arguments)]
/// `AI_007_TownEntities_Shimmer_TeleportToLandingSpot` +
/// `ScanForBestSpotToLandOn` (`NPC.cs:55563-55641`): the four expanding
/// scans (top 1..29 step 2, home 1..29, top 30-or-0..59 allowSolidTop,
/// home 30..59), the first spot wins; a ≥560-px jump arms `ai[2] = 30`.
/// `ShimmerHelper.FindSpotWithoutShimmer` is the tr-core port; the
/// `IsSpotShimmerFree` probe (`ShimmerHelper.cs:67-82`) rides the closure.
fn shimmer_teleport_to_landing_spot(npc: &mut Npc, world: &World, homeless: bool) {
    let (w, h) = npc.size();
    let top_x = (npc.x + w / 2.0) as i32 / 16;
    let top_y = npc.y as i32 / 16;
    let home = npc.home;
    // `flag = homeless && (homeTileX == -1 || homeTileY == -1)` — the
    // port folds the pair into `home: Option` (a homeless NPC keeps no
    // home), so `homeless` alone is the flag.
    let flag0 = homeless && home.is_none();
    let spot_free = |px: f32, py: f32, allow_top: bool| -> bool {
        shimmer_spot_free(world, px, py, w as i32, h as i32, allow_top)
    };
    let mut result: Option<(f32, f32)> = None;
    // Scan 1 (`:55590-55598`): the top, expand 1..29 step 2.
    let mut i = 1;
    while i < 30 {
        if let Some(p) = tr_core::shimmer::find_spot_without_shimmer(
            top_x, top_y, i, flag0, w, h, &|px, py| spot_free(px, py, flag0),
        ) {
            result = Some(p);
            break;
        }
        i += 2;
    }
    // Scan 2 (`:55599-55610`): the home, same window.
    if result.is_none() {
        if let Some((hx, hy)) = home {
            let mut j = 1;
            while j < 30 {
                if let Some(p) = tr_core::shimmer::find_spot_without_shimmer(
                    hx, hy, j, flag0, w, h, &|px, py| spot_free(px, py, flag0),
                ) {
                    result = Some(p);
                    break;
                }
                j += 2;
            }
        }
    }
    // Scan 3 (`:55611-55625`): the top, expand (flag0 ? 30 : 0)..59,
    // allowSolidTop forced true.
    if result.is_none() {
        let start = if flag0 { 30 } else { 0 };
        let mut k = start;
        while k < 60 {
            if let Some(p) = tr_core::shimmer::find_spot_without_shimmer(
                top_x, top_y, k, true, w, h, &|px, py| spot_free(px, py, true),
            ) {
                result = Some(p);
                break;
            }
            k += 2;
        }
    }
    // Scan 4 (`:55626-55639`): the home, 30..59, allowSolidTop true.
    if result.is_none() {
        if let Some((hx, hy)) = home {
            let mut l = 30;
            while l < 60 {
                if let Some(p) = tr_core::shimmer::find_spot_without_shimmer(
                    hx, hy, l, true, w, h, &|px, py| spot_free(px, py, true),
                ) {
                    result = Some(p);
                    break;
                }
                l += 2;
            }
        }
    }
    if let Some((px, py)) = result {
        let (ox, oy) = (npc.x, npc.y);
        npc.x = px;
        npc.y = py;
        let (mx, my) = (px - ox, py - oy);
        if (mx * mx + my * my).sqrt() >= 560.0 {
            npc.ai[2] = 30.0; // the ShimmerTownNPCSend particle is the seam
        }
        npc.net_update = true;
    }
}

/// `ShimmerHelper.IsSpotShimmerFree` (`ShimmerHelper.cs:67-82`): the body
/// box must be solid-free, the 100-px column below it must HIT solid
/// (allowSolidTop widens the support test), and the body+100 box must not
/// touch shimmer liquid. `Collision.SolidCollision` (`:2700-2778`).
fn shimmer_spot_free(world: &World, x: f32, y: f32, w: i32, h: i32, allow_top: bool) -> bool {
    if solid_collision_box(world, x, y, w, h, false) {
        return false;
    }
    if !solid_collision_box(world, x, y + h as f32, w, 100, allow_top) {
        return false;
    }
    let (_, _, shimmer) = crate::npc_ai_slime::wet_collision(world, x, y, w, h + 100);
    !shimmer
}

/// `Collision.SolidCollision(Position, Width, Height, acceptTopSurfaces)`
/// (`Collision.cs:2735-2778`): active, not actuated; solid-not-top, OR
/// (acceptTop) solid-top frameY==0 / platform proper-top-frame. The
/// half-brick 8-px lift, strict overlap.
fn solid_collision_box(world: &World, x: f32, y: f32, w: i32, h: i32, accept_top: bool) -> bool {
    let x0 = ((x / 16.0) as i32 - 1).clamp(0, world.width - 1);
    let x1 = (((x + w as f32) / 16.0) as i32 + 2).clamp(0, world.width - 1);
    let y0 = ((y / 16.0) as i32 - 1).clamp(0, world.height - 40);
    let y1 = (((y + h as f32) / 16.0) as i32 + 2).clamp(0, world.height - 40);
    for i in x0..x1 {
        for j in y0..y1 {
            let Some(t) = world.tile(i, j) else { continue };
            if !nactive(t) {
                continue;
            }
            let mut flag = raw_solid(t.kind) && !raw_solid_top(t.kind);
            if accept_top {
                flag = if td::SET_PLATFORMS.has(t.kind) {
                    flag || platform_proper_top_frame(t.frame_x)
                } else {
                    flag || (raw_solid_top(t.kind) && t.frame_y == 0)
                };
            }
            if !flag {
                continue;
            }
            let tx = (i * 16) as f32;
            let mut ty = (j * 16) as f32;
            let mut th = 16;
            if t.half_brick() {
                ty += 8.0;
                th -= 8;
            }
            if x + w as f32 > tx && x < tx + 16.0 && y + h as f32 > ty && y < ty + th as f32 {
                return true;
            }
        }
    }
    false
}

/// `WorldGen.PlatformProperTopFrame` (`WorldGen.cs:70153-70165`):
/// `frameX / 18` in 0..7, 12..16 or 25..26.
fn platform_proper_top_frame(frame_x: i16) -> bool {
    let num = frame_x as i32 / 18;
    ((0..=7).contains(&num) || (12..=16).contains(&num)) || (25..=26).contains(&num)
}

/// `AI_007_TownEntities`' state 25 (`NPC.cs:53187-53277`, W4-f47): the
/// shimmer emergence. ai[1]==0 zeroes vx + the landing-spot teleport;
/// ai[2] is the teleport hold; ai[1] climbs, at ≥30 the transparency
/// decays 1/60 per tick UNLESS wet (the WetCollision probe), velocity
/// (0, -4*t); the hue light + the 309 dust + the 1/15 particle triple;
/// at ai[1] ≥ 75 && t ≤ 0 the variant flips, the ai/localAI zero, the
/// (0,-4) kick, Teleport(pos, 12) and progression 43 when variation 1.
fn town_shimmer_state25(
    npc: &mut Npc,
    env: &TownEnv<'_>,
    rng: &mut UnifiedRandom,
    out: &mut TownOutputs,
) {
    let world = env.world;
    let (w, h) = npc.size();
    npc.dont_take_damage = true; // :53189
    if npc.ai[1] == 0.0 {
        npc.vx = 0.0; // :53192
    }
    // :53194-53197 — the wet flags clear (the port's NPC carries
    // shimmer/honey; `wet` is the swimmer cache, cleared here too).
    npc.shimmer_wet = false;
    npc.honey_wet = false;
    npc.wet = false;
    // :53198-53201 — the client-only early return (netMode 1) is out.
    if npc.ai[1] == 0.0 && npc.ai[2] < 1.0 {
        shimmer_teleport_to_landing_spot(npc, world, env.homeless); // :53204
    }
    if npc.ai[2] > 0.0 {
        npc.ai[2] -= 1.0; // :53208
        if npc.ai[2] <= 0.0 {
            npc.ai[1] = 1.0; // :53211
        }
        return; // :53213
    }
    npc.ai[1] += 1.0; // :53215
    if npc.ai[1] >= 30.0 {
        // :53218-53225 — the transparency decay gates on a FRESH
        // WetCollision probe (not the cached wet flag).
        let (wet, _, _) = crate::npc_ai_slime::wet_collision(world, npc.x, npc.y, w as i32, h as i32);
        if !wet {
            npc.shimmer_transparency = (npc.shimmer_transparency - 1.0 / 60.0).clamp(0.0, 1.0);
        } else {
            npc.ai[1] = 30.0;
        }
        npc.vx = 0.0;
        npc.vy = -4.0 * npc.shimmer_transparency; // :53226
    }
    // :53228-53236 — the hitbox (Y+20, Height-20), the direction draw,
    // the hue light and the conditional dust. The draws ride `rng` in
    // the C# order: NextFloatDirection, then the NextFloat gate, then
    // (on a hit) the rectangle pair + the circular pair.
    let hb = (npc.x, npc.y + 20.0, w, h - 20.0);
    let num5 = rng.next_float_direction(); // :53231
    let (cx, cy) = (npc.x + w / 2.0, npc.y + h / 2.0);
    let hue = (env.time_for_visual_effects as f32 / 360.0) % 1.0;
    let rgb = crate::ui::color::hsl_to_rgb(hue as f64, 0.6, 0.65, 255);
    let light_scale = shimmer_remap(npc.ai[1], 30.0, 90.0, 0.0, 0.7);
    out.shimmer_fx.push(ShimmerFx::Light {
        x: cx,
        y: cy,
        rgb: [
            rgb[0] as f32 / 255.0 * light_scale,
            rgb[1] as f32 / 255.0 * light_scale,
            rgb[2] as f32 / 255.0 * light_scale,
        ],
    });
    // :53233 — `NextFloat() > Remap(ai1, 30, 60, 1, 0.5)`.
    let gate = shimmer_remap(npc.ai[1], 30.0, 60.0, 1.0, 0.5);
    if rng.next_float() > gate {
        // NextVector2FromRectangle(hitbox) + NextVector2Circular(8, 0)
        // + (0, 4) — Utils.cs:1256-1268 (two + two NextFloat draws).
        let rx = hb.0 + rng.next_float() * hb.2;
        let ry = hb.1 + rng.next_float() * hb.3;
        let ang = std::f32::consts::TAU * rng.next_float();
        let rad = rng.next_float();
        let (ux, uy) = (ang.cos(), ang.sin());
        let px = rx + ux * 8.0 * rad;
        let py = ry + uy * 0.0 * rad + 4.0;
        // (0,-2).RotatedBy(num5 * 2pi * 0.11) — Utils.cs:1675-1684.
        let rot = (num5 * std::f32::consts::TAU * 0.11) as f64;
        let (rc, rs) = (rot.cos() as f32, rot.sin() as f32);
        let (vx, vy) = (0.0 * rc - (-2.0) * rs, 0.0 * rs + (-2.0) * rc);
        out.shimmer_fx.push(ShimmerFx::Dust {
            x: px,
            y: py,
            vx,
            vy,
            scale: 1.7 - num5.abs() * 1.3,
        });
    }
    // :53237-53248 — ai1 > 60 && Next(15) == 0 → three ShimmerBlock
    // particles (the orchestrator is unported; the requests record).
    if npc.ai[1] > 60.0 && rng.next_max(15) == 0 {
        for _ in 0..3 {
            let px = npc.x + rng.next_float() * w;
            let py = npc.y + rng.next_float() * h;
            let (dx, dy) = direction_to(cx, cy, px, py);
            let sign = (rng.next_max(2) * 2 - 1) as f32;
            let rot = (std::f32::consts::PI * 9.0 / 20.0 * sign) as f64;
            let (rc, rs) = (rot.cos() as f32, rot.sin() as f32);
            let mag = rng.next_float();
            out.shimmer_fx.push(ShimmerFx::Particle {
                x: px,
                y: py,
                mx: (dx * rc - dy * rs) * mag,
                my: (dx * rs + dy * rc) * mag,
            });
        }
    }
    // :53249-53250 — TargetClosest + GetTargetData (the target centre).
    let players: Vec<TargetCandidate> = env.players.iter().map(|p| p.candidate).collect();
    crate::npc_target::target_closest(npc, world, &players, true, 7, 0);
    let target_center = if npc.target >= 0 && (npc.target as usize) < env.players.len() {
        let p = &env.players[npc.target as usize].candidate;
        (p.x + p.width as f32 / 2.0, p.y + p.height as f32 / 2.0)
    } else {
        (cx, cy)
    };
    // :53251-53275 — the completion: ai1 ≥ 75 && t ≤ 0 (netMode != 1).
    if npc.ai[1] >= 75.0 && npc.shimmer_transparency <= 0.0 {
        npc.ai = [0.0; 4];
        // :53257 — the Math.Sign call's result is DISCARDED (the C#
        // computes it and drops it; no direction write).
        let _ = (target_center.0 - cx).signum();
        npc.vx = 0.0;
        npc.vy = -4.0; // :53258
        npc.local_ai = [0.0; 4];
        npc.net_update = true;
        npc.variation_index = if npc.variation_index != 1 { 1 } else { 0 }; // :53264
        // :53266 — Teleport(position, 12): the position is UNCHANGED
        // (the C# teleports to where it already stands — the effect +
        // teleportTime latch are the visual seam, unported).
        if npc.variation_index == 1 {
            out.shimmer_town_progression = true; // :53273
        }
    }
}

/// `Utils.Remap` (`Utils.cs:345-348`) — the clamped lerp.
fn shimmer_remap(v: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
    let t = ((v - from_min) / (from_max - from_min)).clamp(0.0, 1.0);
    to_min + (to_max - to_min) * t
}

/// The state-25 emergence SEAM (doc-sweep lane 36, W4-f47): the world-sim
/// slot loop calls this for a living town NPC whose `ai[0] == 25` — the
/// state `drain_shimmer_transforms`' TownVariant arm sets
/// (`NPC.cs:91578-91591`). It builds the solo classic [`TownEnv`] (the
/// b3 adapter's fold: one session player, empty NPC list, the unported
/// statics false), runs [`town_shimmer_state25`] on the caller's
/// `UpdateNpcs` stream, and drains the visuals: the hue light onto the
/// shared `EntityLights` (the `(int)(v/16)` truncation, the
/// `npc_lights.rs` convention), the 309 dust via `NewDustPerfect` (the
/// draws already rode `rng` in-arm — the perfect spawn's own 6 draws ride
/// the same stream here, the C# `:53235` order), the particle requests
/// stay recorded (the orchestrator is unported). Progression 43 credits
/// through the caller's returned flag. Returns `true` when the state ran
/// (the caller then skips the dry walk for this tick — the C# arm RETURNS
/// at `:53277`).
///
/// MERGE CONTRACT (b3): when `task/W4-b3-npc-ai-town-wire` lands, its
/// `ai_007_town_family` adapter reaches the same `:53187` arm through
/// `town_npc_ai`; the merger keeps ONE driver — either this seam folds
/// into the adapter's `TownOutputs` drain or the adapter's state-25 call
/// is dropped in favour of this pre-pass. The draws are identical either
/// way (the same in-arm order on the same stream); the b3 adapter's
/// `shimmering: false` fold must become the NPC's live buff-353 flag.
pub fn town_shimmer_step(
    npc: &mut Npc,
    world: &World,
    player: &crate::player::Player,
    time_for_visual_effects: f64,
    rng: &mut UnifiedRandom,
    dust: &mut tr_core::particles::DustPool,
    dust_ctx: &tr_core::particles::SpawnCtx,
    lights: &mut crate::entity_lights::EntityLights,
) -> bool {
    if npc.ai[0] != 25.0 || npc.hp <= 0 || !npc.kind.is_town_npc_like() {
        return false;
    }
    let players = [TownPlayer {
        candidate: crate::npc_target::session_candidate(player),
        // `Player.talkNPC` lives on the App — the solo seam reads nobody
        // talking (the b3 phase-1 fold).
        talk_npc: -1,
        stinky: player.buffs.buff_type.contains(&120),
        sitting: player.sitting,
        can_be_talked_to: !player.dead && player.stealth == 0.0,
    }];
    let env = TownEnv {
        world,
        day_time: world.day_time,
        raining: world.raining,
        eclipse: world.eclipse,
        slime_rain: world.slime_rain,
        storming: false, // the wind/storm scan unported (the b3 fold)
        net_mode: 0,
        expert_mode: player.buffs.expert_mode,
        master_mode: false,
        players: &players,
        npcs: &[],
        dungeon_x: 0,
        dungeon_y: 0,
        combat_book: world.combat_book_was_used,
        combat_book_two: false,
        party_up: false,
        skyblock_low_tiles: false,
        infected_seed: world.seeds.contains(tr_core::seeds::SeedFlags::ZENITH),
        only_shimmer_ocean: false,
        x_mas: false,
        town_damage_multiplier: 1.0,
        downed_slime_king: world.downed_slime_king,
        downed_boss1: world.downed_eoc,
        downed_deerclops: world.downed_deerclops,
        downed_boss2: world.downed_eow,
        downed_boss3: world.downed_skeletron,
        downed_queen_bee: world.downed_queen_bee,
        hard_mode: world.hard_mode,
        downed_queen_slime: world.downed_queen_slime,
        downed_mech1: world.downed_mech_boss1,
        downed_mech2: world.downed_mech_boss2,
        downed_mech3: world.downed_mech_boss3,
        downed_plant: world.downed_plant_boss,
        downed_empress: false,
        downed_fishron: world.downed_fishron,
        downed_golem: world.downed_golem,
        downed_cultist: world.downed_ancient_cultist,
        homeless: npc.home.is_none(),
        breath: 200,
        // `NPC.shimmering` — the live buff-353 flag (the f47 NPC wiring).
        shimmering: npc.shimmering,
        time_for_visual_effects,
    };
    let mut out = TownOutputs::default();
    town_shimmer_state25(npc, &env, rng, &mut out);
    // The visual drain (the draws already spent in-arm; the perfect
    // spawn's own NewDust draws ride the same stream at the C# `:53235`
    // position — after the gate draws, before the particle triple).
    // ORDER NOTE: the C# spawns each dust INLINE between the light and
    // the particle roll; the port drains the whole list after the arm,
    // so the dusts' NewDust draws land AFTER the particle draws of the
    // same tick. The in-arm gate/direction draws keep their exact order;
    // only the allocator's own draws shift within the tick (documented,
    // ANALYSIS 121 — the same pool-pass convention as the item splash).
    for fx in out.shimmer_fx {
        match fx {
            ShimmerFx::Light { x, y, rgb } => {
                lights.add((x / 16.0) as i32, (y / 16.0) as i32, rgb);
            }
            ShimmerFx::Dust { x, y, vx, vy, scale } => {
                dust.new_dust_perfect(
                    rng,
                    dust_ctx,
                    world,
                    Vec2::new(x, y),
                    309,
                    Some(Vec2::new(vx, vy)),
                    0,
                    tr_core::particles::Color::default(),
                    scale,
                );
            }
            ShimmerFx::Particle { .. } => {
                // The orchestrator is unported — the request was recorded
                // (the draws spent); nothing to spawn.
            }
        }
    }
    out.shimmer_town_progression
}

pub fn town_npc_ai(
    npc: &mut Npc,
    env: &TownEnv<'_>,
    stats: TownKindStats,
    door: &mut TownDoorLatch,
    doors: &mut dyn TownDoorOps,
    who_am_i: usize,
    rng: &mut UnifiedRandom,
    out: &mut TownOutputs,
) {
    let world = env.world;
    let net_id = npc.kind.net_id();
    let (w, h) = npc.size();
    // The C# `townNPC` flag: every is_town_npc_like kind writes
    // `townNPC = true` in SetDefaults (`NPC.cs:8983+`); the port's
    // narrower is_town_npc() set is the HOUSING set, not this flag.
    let town_npc_flag = npc.kind.is_town_npc_like();
    // The C# `stinky` latch: buff 120 (`NPC.cs:92061-92064`).
    let stinky = npc.buffs.find_buff_index(120).is_some();
    let cx = npc.x + w / 2.0;
    let cy = npc.y + h / 2.0;
    // The C# re-reads `base.Center` at every use site; the teleport
    // home (`:53448`) moves the NPC mid-tick, so the danger scan and
    // everything after it read the POST-teleport centre. The port
    // shadows cx/cy there (the pre-teleport reads above keep the
    // entry values, verbatim).

    // `:52967` — ShimmeredTownNPCs[type] = IsShimmerVariant. The
    // bitmap lives in town.rs (the save seam); the caller applies it
    // from variation_index + shimmer_town_transform. Recorded here as
    // a read-only note: no draw, no state the arm reads back.

    // `:52968-52976` — the Andrew/Slimefors easter eggs. The C# writes
    // `defDefense` (the BASE the stat ladder below reads), so the port
    // adjusts the seam's defaults copy BEFORE the ladder runs — a
    // defense_override here would be clobbered by it.
    let mut stats = stats;
    if net_id == 441 && npc.given_name == "Andrew" {
        stats.def_defense = 200;
    }
    if env.only_shimmer_ocean && net_id == 680 && npc.given_name == "Slimefors" {
        npc.hp = npc.max_hp;
        stats.def_defense = 99999;
    }

    // `:52977-52981` — the sit-gate chance: 300, pets 0.
    let mut num = 300;
    if net_id == 638 || net_id == 656 || is_town_slime(net_id) {
        num = 0;
    }

    // `:52982-52998` — the "go home" weather flag.
    let mut flag = env.raining;
    if !env.day_time {
        flag = true;
    }
    if env.eclipse {
        flag = true;
    }
    if env.slime_rain {
        flag = true;
    }
    if env.storming && (npc.y / 16.0) < world.world_surface as f32 {
        flag = true;
    }

    // `:52999-53157` — the stat ladder.
    let dryad_ward = npc.buffs.find_buff_index(165).is_some();
    let tipsy = npc.buffs.find_buff_index(25).is_some();
    let (num2, num3) = town_stat_bonuses(npc, stats, env, dryad_ward, tipsy);
    let flag2 = env.infected_seed && net_id == 20; // the flee-TOWARD marker

    // `:53158-53161` — Santa outside xMas dies (the strike seam).
    if net_id == 142 && env.net_mode != 1 && !env.x_mas {
        out.strikes.push(TownStrike {
            slot: who_am_i,
            damage: 9999,
            knockback: 0.0,
            direction: 0,
            immune_ticks: 0,
        });
    }
    // `:53162-53165` — the firefly/lightning-bug localAI[0] roll.
    if (net_id == 148 || net_id == 149) && npc.local_ai[0] == 0.0 {
        npc.local_ai[0] = rng.next_range(1, 5) as f32;
    }
    // `:53166-53179` — the Mechanic's owned-projectile latch: W4-d11
    // DOC-SWEEP ported the seam — the pool's
    // `lazyNPCOwnedProjectileSearchArray` cache (the 582's `:34050`
    // write) feeds `mechanic_wrench_latched`, and the world-sim NPC pass
    // re-derives `local_ai[0]` at the Mechanic's slot turn
    // (`world_sim/npcs.rs`, the pre-AI head block). The `:55214-55217`
    // gate below reads the live value.
    // `:53180-53184` — the waterfowl takeoff (W4-c10 sweep, the W4-b3
    // co-lane seam; b3's phase 1 left this a cited skip): the walker
    // 362/364/602/608 falling fast or wet transforms into the swimming
    // form and the arm RETURNS. The `Transform(type + 1)` core needs
    // the caller's player seam + kind table, so the new type rides
    // `out.waterfowl_takeoff` (the `wants_transform` pattern) and the
    // caller runs `Npc::transform_to_kind`. RNG-free: Transform draws
    // nothing; TransformVisuals is silent for the waterfowl pairs
    // (`npc_transform_visuals.rs`). The C# reads `base.wet` latched by
    // the previous tick's UpdateCollision — the port's pre-AI `wet`
    // (the b3 adapter's `is_wet` pass) is the same value.
    if matches!(net_id, 362 | 364 | 602 | 608)
        && env.net_mode != 1
        && (npc.vy > 4.0 || npc.vy < -4.0 || npc.wet)
    {
        out.waterfowl_takeoff = Some(net_id + 1);
        return;
    }

    // `:53185` — UpdateSavedStates.
    out.saved_state = match net_id {
        588 => Some(SavedState::Golfer),
        441 => Some(SavedState::TaxCollector),
        107 => Some(SavedState::Goblin),
        108 => Some(SavedState::Wizard),
        124 => Some(SavedState::Mech),
        353 => Some(SavedState::Stylist),
        369 => Some(SavedState::Angler),
        550 => Some(SavedState::Bartender),
        _ => None,
    };
    // `:53186` — dontTakeDamage = false every tick.
    npc.dont_take_damage = false;

    // `:53187-53277` — the shimmer state 25 (W4-f47): the emergence
    // machine. The visuals ride the `shimmer_fx` seam (the draws spend
    // `rng` in-arm, the wiring spawns the dust/lights); the C# returns
    // from the whole arm here.
    if npc.ai[0] == 25.0 {
        town_shimmer_state25(npc, env, rng, out);
        return;
    }

    // `:53278-53299` — the town-critter facing + the homeless home
    // stamp. No ported kind is a TownCritter; the else arm runs.
    if town_critter(net_id) && npc.target == 255 {
        // The critter TargetClosest + facing: unported kinds, cited.
    } else if npc.home.is_none() && npc.vy == 0.0 && !env.shimmering {
        // `:53296-53298` — the homeless home stamp.
        out.home_updates.push(HomeTileUpdate {
            homeless: true,
            x: cx as i32 / 16,
            y: (npc.y + h + 4.0) as i32 / 16,
        });
    }

    // `:53300-53303` — the tile probes + the resting spot.
    let mut flag5 = false; // a player is talking to this NPC
    let num6 = (npc.x + w / 2.0) as i32 / 16;
    let num7 = (npc.y + h + 1.0) as i32 / 16;
    let (floor_x, floor_y) = find_good_resting_spot(world, npc, env.npcs, num6, num7, env.day_time);
    // `:53304-53307`.
    if net_id == 441 {
        out.tax_collector_present = true;
    }
    // `:53308-53312`.
    npc.direction_y = -1;
    if npc.facing == 0 {
        npc.facing = 1;
    }

    // `:53313-53336` — the talking latch: any player whose talkNPC is
    // this NPC forces state 0 / ai[1]=300 / localAI[3]=100 and faces
    // the player (unless already in a talk/attack state).
    let flag6 = npc.ai[0] == 24.0;
    let mut num8 = 0i32;
    if !flag6 {
        for p in env.players {
            if !p.candidate.active || p.talk_npc != who_am_i as i32 {
                continue;
            }
            flag5 = true;
            if npc.ai[0] != 10.0 && npc.ai[0] != 12.0 && npc.ai[0] != 14.0 && npc.ai[0] != 15.0 {
                if npc.ai[0] != 0.0 {
                    npc.net_update = true;
                }
                npc.ai[0] = 0.0;
                npc.ai[1] = 300.0;
                npc.local_ai[3] = 100.0;
                let p_center_x = p.candidate.x + p.candidate.width as f32 / 2.0;
                num8 = if !(p_center_x < npc.x + w / 2.0) { 1 } else { -1 };
            }
        }
    }

    // `:53337-53348` — ai[3] == 1: the despawn request.
    if npc.ai[3] == 1.0 {
        npc.hp = -1;
        out.despawn = true;
        npc.net_update = true;
        return;
    }
    // `:53349-53357` — the Old Man: dungeon home, gone after Skeletron.
    if net_id == 37 && env.net_mode != 1 {
        out.home_updates.push(HomeTileUpdate {
            homeless: false,
            x: env.dungeon_x,
            y: env.dungeon_y,
        });
        if env.downed_boss3 {
            npc.ai[3] = 1.0;
            npc.net_update = true;
        }
    }
    // `:53358-53374` — the Travelling Merchant: homeless, night departure.
    if net_id == 368 {
        if !env.day_time {
            out.home_updates.push(HomeTileUpdate {
                homeless: true,
                x: (cx / 16.0) as i32,
                y: (npc.y + h + 2.0) as i32 / 16,
            });
            if !flag5 && npc.ai[0] == 0.0 {
                npc.ai[0] = 1.0;
                npc.ai[1] = 200.0;
            }
            flag = false;
        }
    }
    // `:53375-53391` — the Angler's wet edge rescue.
    if net_id == 369 && npc.home.is_none() && npc.wet {
        if cx / 16.0 < 380.0 || cx / 16.0 > (world.width - 380) as f32 {
            out.home_updates.push(HomeTileUpdate {
                homeless: true,
                x: world.spawn_x,
                y: world.spawn_y,
            });
            npc.ai[0] = 1.0;
            npc.ai[1] = 200.0;
        }
        if npc.x / 16.0 < 300.0 {
            npc.facing = 1;
        } else if npc.x / 16.0 > (world.width - 300) as f32 {
            npc.facing = -1;
        }
    }
    // `:53392-53414` — CritterThatCanTurnOnPlayers: unported kinds.

    // `:53415-53418` — WorldGen.IsTileLoaded: the port's worlds are
    // fully loaded; vacuous.
    // `:53419-53422` — the 666 tile's stink buff.
    if tile_at(world, num6, num7).is_some_and(|t| t.kind == 666) && env.net_mode != 1 {
        out.repeated_stink_buff = true;
    }

    // `:53423-53450` — the night/rain teleport home when no player is
    // near either the NPC or the resting spot. The C# re-reads
    // `this.position` for the rect AFTER the earlier arms may have
    // moved it; the port re-reads npc.x/y here too (the teleport
    // itself is the only mover inside this block).
    let dungeon_tile = tile_at(world, num6, num7).is_some_and(|t| tile_dungeon(t.kind));
    if !env.homeless
        && env.net_mode != 1
        && town_npc_flag
        && (flag || (net_id == 37 && dungeon_tile))
        && !is_in_a_good_resting_spot(net_id, npc.ai[0], npc.wet, env.day_time, num6, num7, floor_x, floor_y)
    {
        // The safe-area rect: sWidth/sHeight 1920×1200, safeRange 62/39
        // (`NPC.cs:935-936`, the spawner's S_WIDTH/S_HEIGHT seam).
        let s_width = crate::spawner::S_WIDTH;
        let s_height = crate::spawner::S_HEIGHT;
        let safe_x = (s_width / 16 * 52) / 100;
        let safe_y = (s_height / 16 * 52) / 100;
        let mut flag7 = true;
        for k in 0..2 {
            if !flag7 {
                break;
            }
            // The C# `new Rectangle((int)(...), ...)` truncates each
            // origin to int (`:53432-53435`); the player rect truncates
            // its position too (`:53439`).
            let rect = if k == 0 {
                (
                    (npc.x + w / 2.0 - s_width as f32 / 2.0 - safe_x as f32) as i32 as f32,
                    (npc.y + h / 2.0 - s_height as f32 / 2.0 - safe_y as f32) as i32 as f32,
                    (s_width + safe_x * 2) as f32,
                    (s_height + safe_y * 2) as f32,
                )
            } else {
                (
                    (floor_x as f32 * 16.0 + 8.0 - s_width as f32 / 2.0 - safe_x as f32) as i32 as f32,
                    (floor_y as f32 * 16.0 + 8.0 - s_height as f32 / 2.0 - safe_y as f32) as i32 as f32,
                    (s_width + safe_x * 2) as f32,
                    (s_height + safe_y * 2) as f32,
                )
            };
            for p in env.players {
                if p.candidate.active
                    && rect_intersects(rect, (p.candidate.x as i32 as f32, p.candidate.y as i32 as f32, p.candidate.width as f32, p.candidate.height as f32))
                {
                    flag7 = false;
                    break;
                }
            }
        }
        if flag7 {
            if !teleport_to_home(world, npc, env.npcs, floor_x, floor_y, rng) {
                // `:55668-55669` — homeless + QuickFindHome: the
                // housing seam's scan, requested through the home
                // update (the caller runs town.rs::quick_find_home).
                out.home_updates.push(HomeTileUpdate { homeless: true, x: -1, y: -1 });
            }
        }
    }
    // The post-teleport centre re-read (see the entry note).
    let cx = npc.x + w / 2.0;
    let cy = npc.y + h / 2.0;

    // `:53451-53469` — the kind-group flags + the danger ranges.
    let flag8 = matches!(net_id, 300 | 447 | 610); // the duck family
    let flag9 = matches!(net_id, 616 | 617 | 625); // the water striders
    let flag10 = matches!(net_id, 361 | 445 | 687); // the sea critters
    let flag11 = is_town_slime(net_id);
    let flag12 = flag9 | flag10; // canBreathUnderWater
    let flag13 = flag9 | flag10;
    let flag14 = flag11;
    let flag15 = flag11;
    let mut num10 = 200.0f32;
    let ddr = danger_detect_range(net_id);
    if ddr != -1 {
        num10 = ddr as f32;
    }
    let mut num11 = num10;
    if flag5 && attack_type(net_id) == 3 && num11 < 250.0 {
        num11 = 250.0;
    }

    // `:53470-53621` — the danger scan.
    let mut flag16 = false; // any danger in the wide range
    let mut flag17 = false; // any danger in the attack range
    let mut flag18 = false; // a stinky danger
    let flag19 = false; // the C# never sets it (dead)
    let mut num12 = -1.0f32; // nearest danger LEFT (negative dx)
    let mut num13 = -1.0f32; // nearest danger RIGHT (positive dx)
    let mut num14 = 0i32; // the flee/attack direction
    let mut num15 = -1i32; // the chaseable LEFT target slot
    let mut num16 = -1i32; // the chaseable RIGHT target slot
    if !flag9 && env.net_mode != 1 {
        for (m, other) in env.npcs {
            // `:53483` — the skip chain.
            if critter_turns_on_players(other.net_id)
                || other.net_id == 690
                || ((other.friendly || other.damage <= 0) && !other.stinky)
                || other.distance(cx, cy) >= num11
                || (net_id == 453 && is_skeleton(other.net_id))
                || (!other.no_tile_collide && !can_hit_centers(world, cx, cy, other.center().0, other.center().1))
            {
                continue;
            }
            flag16 = true;
            if other.distance(cx, cy) >= num10 {
                continue;
            }
            let chaseable = other.can_be_chased_by();
            flag17 = true;
            if other.stinky {
                flag18 = true;
            }
            let num17 = other.center().0 - cx;
            if net_id == 614 {
                // The penguin's inverted pairing (`:53499-53512`).
                if num17 < 0.0 && (num12 == -1.0 || num17 > num12) {
                    num13 = num17;
                    num16 = *m as i32;
                }
                if num17 > 0.0 && (num13 == -1.0 || num17 < num13) {
                    num12 = num17;
                    num15 = *m as i32;
                }
                continue;
            }
            if num17 < 0.0 && (num12 == -1.0 || num17 > num12) {
                num12 = num17;
                if chaseable {
                    num15 = *m as i32;
                }
            }
            if num17 > 0.0 && (num13 == -1.0 || num17 < num13) {
                num13 = num17;
                if chaseable {
                    num16 = *m as i32;
                }
            }
        }
        // `:53530-53551` — the stinky-player fallback.
        if !flag17 && !flag5 {
            for (n, p) in env.players.iter().enumerate() {
                if p.candidate.active
                    && !p.candidate.dead
                    && p.stinky
                    && player_distance(p, cx, cy) < num10
                    && (p.stinky || !matches!(net_id, 633 | 37 | 441 | 228 | 107 | 453))
                {
                    flag17 = true;
                    flag18 = true;
                    let p_cx = p.candidate.x + p.candidate.width as f32 / 2.0;
                    let num18 = p_cx - cx;
                    if num18 < 0.0 && (num12 == -1.0 || num18 > num12) {
                        num12 = num18;
                        num15 = n as i32;
                    }
                    if num18 > 0.0 && (num13 == -1.0 || num18 < num13) {
                        num13 = num18;
                        num16 = n as i32;
                    }
                }
            }
        }
        // `:53552-53620` — the flee decision.
        if flag17 && !flag5 {
            num14 = if num12 == -1.0 {
                1
            } else if num13 != -1.0 {
                to_direction_int(num13 < 0.0 - num12)
            } else {
                -1
            };
            let mut num19 = 0.0f32;
            if num12 != -1.0 {
                num19 = 0.0 - num12;
            }
            if num19 == 0.0 || (num13 < num19 && num13 > 0.0) {
                num19 = num13;
            }
            if npc.ai[0] == 8.0 {
                // `:53564-53574` — the cornered state re-flees when the
                // danger crosses to the faced side.
                if npc.facing == -num14 {
                    npc.ai[0] = 1.0;
                    npc.ai[1] = 300.0 + rng.next_max(300) as f32;
                    npc.ai[2] = 0.0;
                    npc.local_ai[3] = 0.0;
                    npc.net_update = true;
                }
            } else if npc.ai[0] != 10.0 && npc.ai[0] != 12.0 && npc.ai[0] != 13.0 && npc.ai[0] != 14.0 && npc.ai[0] != 15.0 {
                let ps = pretty_safe(net_id);
                if ps != -1 && (ps as f32) < num19 {
                    flag17 = false;
                } else if npc.ai[0] != 1.0 {
                    // `:53583` — the walk prediction before the flee.
                    let tile_x = ((npc.x + w / 2.0 + 15.0 * npc.facing as f32) / 16.0) as i32;
                    let tile_y = ((npc.y + h - 16.0) / 16.0) as i32;
                    let (_, avoid_falling) = get_walk_prediction(
                        world, npc, env.npcs, num6, floor_x, flag12, npc.wet && !flag12, tile_x, tile_y,
                    );
                    if !avoid_falling {
                        // `:53586-53598` — the talk partner flees too.
                        if matches!(npc.ai[0], 3.0 | 4.0 | 16.0 | 17.0) {
                            let partner_slot = npc.ai[2] as usize;
                            if let Some((_, partner)) = env.npcs.iter().find(|(s, _)| *s == partner_slot) {
                                let _ = partner; // active check below via the view
                                out.partner_writes.push(TownPartnerWrite::Flee {
                                    slot: partner_slot,
                                    timer: 120.0 + rng.next_max(120) as f32,
                                    direction: -num14,
                                });
                            }
                        }
                        npc.ai[0] = 1.0;
                        npc.ai[1] = 120.0 + rng.next_max(120) as f32;
                        npc.ai[2] = 0.0;
                        npc.local_ai[3] = 0.0;
                        npc.facing = if flag2 { num14 } else { -num14 };
                        npc.net_update = true;
                    }
                } else if !flag2 && npc.ai[0] == 1.0 && npc.facing != -num14 {
                    npc.facing = -num14;
                    npc.net_update = true;
                }
            }
        }
    }

    // -----------------------------------------------------------------
    // The state machine
    // -----------------------------------------------------------------
    if npc.ai[0] == 0.0 {
        town_state_idle(npc, env, door, who_am_i, rng, out, IdleCtx {
            flag, flag5, flag8, flag10, flag11, flag12, flag16, flag17, num6, num7, floor_x, floor_y, num8, stinky,
        });
    } else if npc.ai[0] == 1.0 {
        town_state_walk(npc, env, door, doors, rng, out, WalkCtx {
            flag, flag8, flag9, flag10, flag12, flag14, flag15, flag17, num6, num7, floor_x, floor_y, town_npc_flag,
        });
    } else if npc.ai[0] == 2.0 || npc.ai[0] == 11.0 {
        // `:54245-54266` — the small-talk / tax-collector pause.
        if env.net_mode != 1 {
            npc.local_ai[3] -= 1.0;
            if rng.next_max(60) == 0 && npc.local_ai[3] == 0.0 {
                npc.local_ai[3] = 60.0;
                npc.facing *= -1;
                npc.net_update = true;
            }
        }
        npc.ai[1] -= 1.0;
        npc.vx *= 0.8;
        if npc.ai[1] <= 0.0 {
            npc.local_ai[3] = 40.0;
            npc.ai[0] = 0.0;
            npc.ai[1] = 60.0 + rng.next_max(60) as f32;
            npc.net_update = true;
        }
    } else if matches!(npc.ai[0], 3.0 | 4.0 | 5.0 | 8.0 | 9.0 | 16.0 | 17.0 | 20.0 | 21.0 | 22.0 | 23.0) {
        // `:54267-54297` — the talk/sit/cornered/interact/idle-anim pause.
        npc.vx *= 0.8;
        npc.ai[1] -= 1.0;
        if npc.ai[0] == 8.0 && npc.ai[1] < 60.0 && flag17 {
            npc.ai[1] = 180.0;
            npc.net_update = true;
        }
        if npc.ai[0] == 5.0 {
            // `:54276-54288` — the sitting check: the seat tile must
            // still be a chair/bed, else the sit ends. The
            // sittingManager.AddNPC registration is the caller's seam.
            let seat = ((npc.x + w / 2.0) as i32 / 16, (npc.y + h - 2.0) as i32 / 16);
            match tile_at(world, seat.0, seat.1) {
                Some(t) if t.kind == 15 || t.kind == 497 => {}
                _ => npc.ai[1] = 0.0,
            }
        }
        if npc.ai[1] <= 0.0 {
            npc.ai[0] = 0.0;
            npc.ai[1] = 60.0 + rng.next_max(60) as f32;
            npc.ai[2] = 0.0;
            npc.local_ai[3] = 30.0 + rng.next_max(60) as f32;
            npc.net_update = true;
        }
    } else if matches!(npc.ai[0], 6.0 | 7.0 | 18.0 | 19.0) {
        // `:54298-54328` — the player-facing talk states.
        if npc.ai[0] == 18.0 && !(1.0..=2.0).contains(&npc.local_ai[3]) {
            npc.local_ai[3] = 2.0;
        }
        npc.vx *= 0.8;
        npc.ai[1] -= 1.0;
        let num37 = npc.ai[2] as i32;
        let valid = (0..=255).contains(&num37)
            && env.players.get(num37 as usize).is_some_and(|p| {
                p.can_be_talked_to
                    && player_distance(p, cx, cy) <= 200.0
                    && can_hit_line_centers(world, npc.x + w / 2.0, npc.y, p.candidate.x + p.candidate.width as f32 / 2.0, p.candidate.y)
            });
        if !valid {
            npc.ai[1] = 0.0;
        }
        if npc.ai[1] > 0.0 {
            if let Some(p) = env.players.get(num37 as usize) {
                let p_cx = p.candidate.x + p.candidate.width as f32 / 2.0;
                let num38 = if cx < p_cx { 1 } else { -1 };
                if num38 != npc.facing {
                    npc.net_update = true;
                }
                npc.facing = num38;
            }
        } else {
            npc.ai[0] = 0.0;
            npc.ai[1] = 60.0 + rng.next_max(60) as f32;
            npc.ai[2] = 0.0;
            npc.local_ai[3] = 30.0 + rng.next_max(60) as f32;
            npc.net_update = true;
        }
    } else if npc.ai[0] == 10.0 {
        town_attack_ranged_arc(npc, env, rng, out, AttackCtx { num2, num14, num15, num16, flag17, who_am_i });
    } else if npc.ai[0] == 12.0 {
        town_attack_straight(npc, env, rng, out, AttackCtx { num2, num14, num15, num16, flag17, who_am_i });
    } else if npc.ai[0] == 13.0 {
        // `:54797-54826` — the Nurse's heal bolt (projectile 584).
        npc.vx *= 0.8;
        if attack_time(net_id) as f32 == npc.ai[1] {
            // frameCounter = 0 — the frame seam's, no draw.
        }
        npc.ai[1] -= 1.0;
        npc.local_ai[3] += 1.0;
        if npc.local_ai[3] == 1.0 && env.net_mode != 1 {
            let target_slot = npc.ai[2] as usize;
            if let Some((_, target)) = env.npcs.iter().find(|(s, _)| *s == target_slot) {
                let (tcx, tcy) = target.center();
                let mut vec3 = direction_to(cx, cy, tcx, tcy - 20.0);
                if vec3.0.is_nan() || vec3.1.is_nan() || sign_f(vec3.0) == -npc.sprite_dir {
                    vec3 = (npc.sprite_dir as f32, -1.0);
                }
                vec3 = (vec3.0 * 8.0, vec3.1 * 8.0);
                out.shots.push(TownShot {
                    kind: 584,
                    x: cx + (npc.sprite_dir * 16) as f32,
                    y: cy - 2.0,
                    vx: vec3.0,
                    vy: vec3.1,
                    damage: 0,
                    knockback: 0.0,
                    ai0: npc.ai[2],
                    ai1: 0.0,
                    time_left_override: None,
                    variation_index: 0,
                });
            }
        }
        if npc.ai[1] <= 0.0 {
            npc.ai[0] = 0.0;
            npc.ai[1] = 10.0 + rng.next_max(10) as f32;
            npc.ai[2] = 0.0;
            npc.local_ai[3] = 5.0 + rng.next_max(10) as f32;
            npc.net_update = true;
        }
    } else if npc.ai[0] == 14.0 {
        town_attack_magic(npc, env, rng, out, AttackCtx { num2, num14, num15, num16, flag17, who_am_i });
    } else if npc.ai[0] == 15.0 {
        town_attack_melee(npc, env, rng, out, AttackCtx { num2, num14, num15, num16, flag17, who_am_i });
    } else if npc.ai[0] == 24.0 {
        // `:55123-55140` — the shimmer-arrival pose: face right, decay.
        npc.vx *= 0.8;
        npc.ai[1] -= 1.0;
        npc.local_ai[3] += 1.0;
        npc.facing = 1;
        npc.sprite_dir = 1;
        if npc.ai[1] <= 0.0 {
            npc.ai[0] = 0.0;
            npc.ai[1] = 480.0;
            npc.ai[2] = 0.0;
            npc.local_ai[1] = 480.0;
            npc.net_update = true;
        }
    }

    // `:55141-55196` — the slime/sea-critter water tails (no ported
    // kind is a slime/strider; the branches evaluate false).
    if flag14 && npc.wet {
        let num86 = (cx / 16.0) as i32;
        if npc.collide_x || (num86 < 5 && npc.facing == -1) || (num86 > world.width - 5 && npc.facing == 1) {
            npc.facing *= -1;
            npc.vx *= -0.25;
            npc.net_update = true;
        }
        npc.vy *= 0.9;
        npc.vy -= 0.5;
        if npc.vy < -15.0 {
            npc.vy = -15.0;
        }
    }
    if flag13 && npc.wet {
        if flag10 {
            npc.ai[1] = 50.0;
        }
        let num88 = (cx / 16.0) as i32;
        if npc.collide_x || (num88 < 5 && npc.facing == -1) || (num88 > world.width - 5 && npc.facing == 1) {
            npc.facing *= -1;
            npc.vx *= -0.25;
            npc.net_update = true;
        }
        if let Some(water_line) = crate::collision::get_water_line(world, (cx / 16.0) as i32, (cy / 16.0) as i32) {
            let num90 = cy + 1.0;
            if cy > water_line {
                npc.vy -= 0.8;
                if npc.vy < -4.0 {
                    npc.vy = -4.0;
                }
                if num90 + npc.vy < water_line {
                    npc.vy = water_line - num90;
                }
            } else {
                npc.vy = npc.vy.min(water_line - num90);
            }
        } else {
            npc.vy -= 0.2;
        }
    }

    // `:55197-55545` — the social + attack-trigger ladder. The C#
    // re-reads base.Center at every use site; the state machine above
    // may have moved the NPC (StepUp/StepDown, the sit snap), so the
    // ladder reads the POST-state centre.
    let cx = npc.x + w / 2.0;
    let cy = npc.y + h / 2.0;
    if env.net_mode != 1 && npc.kind.is_town_npc_like() && (!flag5 | flag17) {
        let mut num3 = num3;
        let mut num14 = num14;
        if flag5 {
            num3 *= 0.8;
            num14 = if num12 == -1.0 {
                1
            } else if num13 != -1.0 {
                to_direction_int(num13 < 0.0 - num12)
            } else {
                -1
            };
        }
        let flag31 = npc.ai[0] < 2.0 && !flag17 && !npc.wet;
        let mut flag32 = (npc.ai[0] < 2.0 || npc.ai[0] == 8.0) && (flag17 | flag19) && !flag18;
        if npc.local_ai[1] > 0.0 {
            npc.local_ai[1] -= 1.0;
        }
        if npc.local_ai[1] > 0.0 {
            flag32 = false;
        }
        if flag32 && net_id == 124 {
            // `:55214-55217` — the Mechanic's sentry-active gate:
            // localAI[0] is the owned-proj latch (W4-d11 DOC-SWEEP: the
            // world-sim pass re-derives it from the pool's cache) — the
            // C# suppresses the attack while the wrench lives.
            if npc.local_ai[0] == 1.0 {
                flag32 = false;
            }
        }
        if flag32 && net_id == 20 {
            // `:55218-55230` — the Dryad only attacks when a town NPC
            // within 1200 px lacks the ward (buff 165).
            flag32 = false;
            for (_, other) in env.npcs {
                if other.town_ai7 && other.distance(cx, cy) <= 1200.0 && !other.has_buff_165 {
                    flag32 = true;
                    break;
                }
            }
        }
        // The CanTalk gate (`NPC.cs:6633-6643`): isLikeATownNPC &&
        // aiStyle 7 && vy == 0 && !IsTownPet.
        let can_talk = npc.kind.is_town_npc_like() && npc.vy == 0.0 && !is_town_pet(net_id);
        town_social_ladder(npc, env, rng, out, SocialCtx {
            can_talk, flag31, flag17, num12, num13, num14, num3, num, who_am_i,
        });
        // `:55422-55442` — the Nurse's heal scan.
        if env.net_mode != 1 && npc.ai[0] < 2.0 && npc.vy == 0.0 && net_id == 18 && env.breath > 0 {
            let mut num116 = -1i32;
            let mut worst = 0i32;
            for (m, other) in env.npcs {
                // `:55428` — CanHitLine takes the FULL hitboxes
                // (`base.position, width, height` vs the target's), not
                // the centre points; the shared seam walks centre to
                // centre, so the port passes the boxes through the
                // projectile seam's box form.
                if other.town_ai7
                    && other.life != other.life_max
                    && (num116 == -1 || other.life_max - other.life > worst)
                    && crate::projectile::can_hit_line(
                        world, npc.x, npc.y, w, h, other.x, other.y, other.width, other.height,
                    )
                    && other.distance(cx, cy) < 500.0
                {
                    num116 = *m as i32;
                    worst = other.life_max - other.life;
                }
            }
            if num116 != -1 {
                npc.ai[0] = 13.0;
                npc.ai[1] = 34.0;
                npc.ai[2] = num116 as f32;
                npc.local_ai[3] = 0.0;
                let target_x = env.npcs.iter().find(|(s, _)| *s == num116 as usize).map(|(_, v)| v.x).unwrap_or(npc.x);
                npc.facing = if npc.x < target_x { 1 } else { -1 };
                npc.net_update = true;
            }
        }
        // `:55443-55545` — the attack-chance ladder.
        let mut num118 = (attack_average_chance(net_id) as f32 * num3) as i32;
        if num118 < 1 {
            num118 = 1;
        }
        let at = attack_type(net_id);
        let ac = attack_average_chance(net_id);
        // Type 0 (`:55448-55472`).
        if flag32 && npc.vy == 0.0 && at == 0 && ac > 0 && rng.next_max(num118) == 0 {
            let num119 = attack_time(net_id);
            let mut num120 = if num14 == 1 { num16 } else { num15 };
            let num121 = if num14 == 1 { num15 } else { num16 };
            if num120 != -1 && !target_can_hit(env, cx, cy, num120) {
                num120 = if num121 == -1 || !target_can_hit(env, cx, cy, num121) { -1 } else { num121 };
            }
            let mut flag37 = num120 != -1;
            if flag37 && net_id == 633 {
                flag37 = target_distance(env, cx, cy, num120) <= 50.0;
            }
            if flag37 {
                npc.local_ai[2] = npc.ai[0];
                npc.ai[0] = 10.0;
                npc.ai[1] = num119 as f32;
                npc.ai[2] = 0.0;
                npc.local_ai[3] = 0.0;
                npc.facing = if npc.x < target_x_of(env, num120) { 1 } else { -1 };
                npc.net_update = true;
            }
        } else if flag32 && npc.vy == 0.0 && at == 1 && ac > 0 && rng.next_max(num118) == 0 {
            // Type 1 (`:55473-55496`).
            let num122 = attack_time(net_id);
            let mut num123 = if num14 == 1 { num16 } else { num15 };
            let num124 = if num14 == 1 { num15 } else { num16 };
            if num123 != -1 && !target_can_hit_line(env, cx, cy, num123) {
                num123 = if num124 == -1 || !target_can_hit_line(env, cx, cy, num124) { -1 } else { num124 };
            }
            if num123 != -1 {
                let (tcx, tcy) = target_center(env, num123);
                let vector8 = direction_to(cx, cy, tcx, tcy);
                if (-0.5..=0.5).contains(&vector8.1) {
                    npc.local_ai[2] = npc.ai[0];
                    npc.ai[0] = 12.0;
                    npc.ai[1] = num122 as f32;
                    npc.ai[2] = vector8.1;
                    npc.local_ai[3] = 0.0;
                    npc.facing = if npc.x < target_x_of(env, num123) { 1 } else { -1 };
                    npc.net_update = true;
                }
            }
        }
        // Type 2 (`:55497-55525`) — a SEPARATE if, not an else-if.
        if flag32 && npc.vy == 0.0 && at == 2 && ac > 0 && rng.next_max(num118) == 0 {
            let num125 = attack_time(net_id);
            let mut num126 = if num14 == 1 { num16 } else { num15 };
            let num127 = if num14 == 1 { num15 } else { num16 };
            if num126 != -1 && !target_can_hit_line(env, cx, cy, num126) {
                num126 = if num127 == -1 || !target_can_hit_line(env, cx, cy, num127) { -1 } else { num127 };
            }
            if num126 != -1 {
                npc.local_ai[2] = npc.ai[0];
                npc.ai[0] = 14.0;
                npc.ai[1] = num125 as f32;
                npc.ai[2] = 0.0;
                npc.local_ai[3] = 0.0;
                npc.facing = if npc.x < target_x_of(env, num126) { 1 } else { -1 };
                npc.net_update = true;
            } else if net_id == 20 {
                // The Dryad fires even with no visible target.
                npc.local_ai[2] = npc.ai[0];
                npc.ai[0] = 14.0;
                npc.ai[1] = num125 as f32;
                npc.ai[2] = 0.0;
                npc.local_ai[3] = 0.0;
                npc.net_update = true;
            }
        }
        // Type 3 (`:55526-55545`).
        if flag32 && npc.vy == 0.0 && at == 3 && ac > 0 && rng.next_max(num118) == 0 {
            let num128 = attack_time(net_id);
            let mut num129 = if num14 == 1 { num16 } else { num15 };
            let num130 = if num14 == 1 { num15 } else { num16 };
            if num129 != -1 && !target_can_hit(env, cx, cy, num129) {
                num129 = if num130 == -1 || !target_can_hit(env, cx, cy, num130) { -1 } else { num130 };
            }
            if num129 != -1 {
                npc.local_ai[2] = npc.ai[0];
                npc.ai[0] = 15.0;
                npc.ai[1] = num128 as f32;
                npc.ai[2] = 0.0;
                npc.local_ai[3] = 0.0;
                npc.facing = if npc.x < target_x_of(env, num129) { 1 } else { -1 };
                npc.net_update = true;
            }
        }
    }
    // `:55547-55560` — the rainbow/copper-slime lights: the light
    // task's, no state.
}

// ---------------------------------------------------------------------
// State 0 — the idle (NPC.cs:53622-53789)
// ---------------------------------------------------------------------

struct IdleCtx {
    flag: bool,
    flag5: bool,
    flag8: bool,
    flag10: bool,
    flag11: bool,
    flag12: bool,
    flag16: bool,
    flag17: bool,
    num6: i32,
    num7: i32,
    floor_x: i32,
    floor_y: i32,
    num8: i32,
    stinky: bool,
}

#[allow(clippy::too_many_arguments)]
fn town_state_idle(
    npc: &mut Npc,
    env: &TownEnv<'_>,
    _door: &mut TownDoorLatch,
    _who_am_i: usize,
    rng: &mut UnifiedRandom,
    _out: &mut TownOutputs,
    c: IdleCtx,
) {
    let world = env.world;
    let net_id = npc.kind.net_id();
    let (w, h) = npc.size();
    // `:53624-53627` — face the talking player when no danger.
    if c.flag5 && c.num8 != 0 && !c.flag17 && !c.flag16 {
        npc.facing = c.num8;
    }
    // `:53628-53631` — the stinky NPC flees at once.
    if c.stinky {
        npc.ai[0] = 1.0;
    }
    // `:53632-53635`.
    if npc.local_ai[3] > 0.0 {
        npc.local_ai[3] -= 1.0;
    }
    // `:53636-53640` — the pet idle chance base.
    let mut num20 = 120;
    if net_id == 638 {
        num20 = 60;
    }
    // `:53641-53648` — the wet sea-critter/slime flee.
    if (c.flag10 | c.flag11) && npc.wet {
        npc.ai[0] = 1.0;
        npc.ai[1] = 200.0 + rng.next_range(500, 700) as f32;
        npc.ai[2] = 0.0;
        npc.local_ai[3] = 0.0;
        npc.net_update = true;
    } else if c.flag && !c.flag5 && !town_critter(net_id) {
        // `:53649-53694` — the go-home arm.
        if env.net_mode != 1 {
            if c.num6 == c.floor_x && c.num7 == c.floor_y {
                if npc.vx != 0.0 {
                    npc.net_update = true;
                }
                if npc.vx > 0.1 {
                    npc.vx -= 0.1;
                } else if npc.vx < -0.1 {
                    npc.vx += 0.1;
                } else {
                    npc.vx = 0.0;
                    try_forcing_sitting(world, npc, env.npcs, c.floor_x, c.floor_y, rng);
                }
                if is_town_pet(net_id) {
                    pet_idle_animation(npc, num20 * 4, env.net_mode, rng);
                }
            } else {
                npc.facing = if c.num6 > c.floor_x { -1 } else { 1 };
                npc.ai[0] = 1.0;
                npc.ai[1] = 200.0 + rng.next_max(200) as f32;
                npc.ai[2] = 0.0;
                npc.local_ai[3] = 0.0;
                npc.net_update = true;
            }
        }
    } else {
        // `:53695-53763` — the daytime idle.
        if c.flag8 {
            npc.vx *= 0.5;
        }
        if npc.vx > 0.1 {
            npc.vx -= 0.1;
        } else if npc.vx < -0.1 {
            npc.vx += 0.1;
        } else {
            npc.vx = 0.0;
        }
        if env.net_mode != 1 {
            if !c.flag5 && is_town_pet(net_id) && (100.0..=150.0).contains(&npc.ai[1]) {
                pet_idle_animation(npc, num20, env.net_mode, rng);
            }
            if npc.ai[1] > 0.0 {
                npc.ai[1] -= 1.0;
            }
            let mut flag21 = true;
            let tile_x = ((npc.x + w / 2.0 + 15.0 * npc.facing as f32) / 16.0) as i32;
            let tile_y = ((npc.y + h - 16.0) / 16.0) as i32;
            let (_, avoid_falling2) = get_walk_prediction(
                world, npc, env.npcs, c.num6, c.floor_x, c.flag12, npc.wet && !c.flag12, tile_x, tile_y,
            );
            // `:53725-53736` — the drown flee.
            if npc.wet
                && !c.flag12
                && check_if_will_drown(drown_collision(world, npc.x, npc.y, w, h, true))
            {
                npc.ai[0] = 1.0;
                npc.ai[1] = 200.0 + rng.next_max(300) as f32;
                npc.ai[2] = 0.0;
                if town_critter(net_id) {
                    npc.ai[1] += rng.next_range(200, 400) as f32;
                }
                npc.local_ai[3] = 0.0;
                npc.net_update = true;
            }
            if avoid_falling2 {
                flag21 = false;
            }
            if npc.ai[1] <= 0.0 {
                if flag21 && !avoid_falling2 {
                    npc.ai[0] = 1.0;
                    npc.ai[1] = 200.0 + rng.next_max(300) as f32;
                    npc.ai[2] = 0.0;
                    if town_critter(net_id) {
                        npc.ai[1] += rng.next_range(200, 400) as f32;
                    }
                    npc.local_ai[3] = 0.0;
                    npc.net_update = true;
                } else {
                    npc.facing *= -1;
                    npc.ai[1] = 60.0 + rng.next_max(120) as f32;
                    npc.net_update = true;
                }
            }
        }
    }
    // `:53764-53788` — the wander bounds: the ±50 hard turn outside the
    // ±25 band, else the 1/80 random turn.
    if env.net_mode != 1
        && (!c.flag || is_in_a_good_resting_spot(net_id, npc.ai[0], npc.wet, env.day_time, c.num6, c.num7, c.floor_x, c.floor_y))
    {
        if c.num6 < c.floor_x - 25 || c.num6 > c.floor_x + 25 {
            if npc.local_ai[3] == 0.0 {
                if c.num6 < c.floor_x - 50 && npc.facing == -1 {
                    npc.facing = 1;
                    npc.net_update = true;
                } else if c.num6 > c.floor_x + 50 && npc.facing == 1 {
                    npc.facing = -1;
                    npc.net_update = true;
                }
            }
        } else if rng.next_max(80) == 0 && npc.local_ai[3] == 0.0 {
            npc.local_ai[3] = 200.0;
            npc.facing *= -1;
            npc.net_update = true;
        }
    }
}

// ---------------------------------------------------------------------
// State 1 — the walk (NPC.cs:53790-54244)
// ---------------------------------------------------------------------

struct WalkCtx {
    flag: bool,
    flag8: bool,
    flag9: bool,
    flag10: bool,
    flag12: bool,
    flag14: bool,
    flag15: bool,
    flag17: bool,
    num6: i32,
    num7: i32,
    floor_x: i32,
    floor_y: i32,
    town_npc_flag: bool,
}

#[allow(clippy::too_many_arguments)]
fn town_state_walk(
    npc: &mut Npc,
    env: &TownEnv<'_>,
    door: &mut TownDoorLatch,
    doors: &mut dyn TownDoorOps,
    rng: &mut UnifiedRandom,
    _out: &mut TownOutputs,
    c: WalkCtx,
) {
    let world = env.world;
    let net_id = npc.kind.net_id();
    let (w, h) = npc.size();
    // `:53792-53798` — the good-rest return home.
    if env.net_mode != 1
        && c.flag
        && is_in_a_good_resting_spot(net_id, npc.ai[0], npc.wet, env.day_time, c.num6, c.num7, c.floor_x, c.floor_y)
        && !town_critter(net_id)
    {
        npc.ai[0] = 0.0;
        npc.ai[1] = 200.0 + rng.next_max(200) as f32;
        npc.local_ai[3] = 60.0;
        npc.net_update = true;
        return;
    }
    // `:53801` — the drown flag.
    let flag22 = !c.flag12 && drown_collision(world, npc.x, npc.y, w, h, true);
    if !flag22 {
        // `:53804-53814` — the far-from-home timer burn.
        let dungeon_tile = tile_at(world, c.num6, c.num7).is_some_and(|t| tile_dungeon(t.kind));
        if env.net_mode != 1 && !env.homeless && !dungeon_tile && (c.num6 < c.floor_x - 35 || c.num6 > c.floor_x + 35) {
            if npc.x < (c.floor_x * 16) as f32 && npc.facing == -1 {
                npc.ai[1] -= 5.0;
            } else if npc.x > (c.floor_x * 16) as f32 && npc.facing == 1 {
                npc.ai[1] -= 5.0;
            }
        }
        npc.ai[1] -= 1.0;
    }
    // `:53817-53832` — the walk end.
    if npc.ai[1] <= 0.0 {
        npc.ai[0] = 0.0;
        npc.ai[1] = 300.0 + rng.next_max(300) as f32;
        npc.ai[2] = 0.0;
        if town_critter(net_id) {
            npc.ai[1] -= rng.next_max(100) as f32;
        } else {
            npc.ai[1] += rng.next_max(900) as f32;
        }
        npc.local_ai[3] = 60.0;
        npc.net_update = true;
    }
    // `:53833-53864` — the door-close pass. The C# clears closeDoor
    // only when CloseDoor/ShiftTallGate SUCCEEDS, then the ±4 far-arm
    // clears it unconditionally; a non-door tile clears it at once.
    if door.close_door
        && ((npc.x + w / 2.0) / 16.0 > (door.door_x + 2) as f32 || (npc.x + w / 2.0) / 16.0 < (door.door_x - 2) as f32)
    {
        let tile_kind = tile_at(world, door.door_x, door.door_y).map(|t| t.kind);
        let far = (npc.x + w / 2.0) / 16.0 > (door.door_x + 4) as f32
            || (npc.x + w / 2.0) / 16.0 < (door.door_x - 4) as f32
            || (npc.y + h / 2.0) / 16.0 > (door.door_y + 4) as f32
            || (npc.y + h / 2.0) / 16.0 < (door.door_y - 4) as f32;
        match tile_kind {
            Some(11) => {
                if doors.close_door(door.door_x, door.door_y) {
                    door.close_door = false;
                }
                if far {
                    door.close_door = false;
                }
            }
            Some(389) => {
                if doors.shift_tall_gate(door.door_x, door.door_y, true) {
                    door.close_door = false;
                }
                if far {
                    door.close_door = false;
                }
            }
            _ => door.close_door = false,
        }
    }
    // `:53865-53927` — the speed/accel table.
    let mut num21 = 1.0f32;
    let mut num22 = 0.07f32;
    if net_id == 614 && c.flag17 {
        num21 = 1.5;
        num22 = 0.1;
    } else if matches!(net_id, 299 | 539 | 538 | 639..=645) {
        num21 = 1.5;
    } else if c.flag9 {
        if npc.wet {
            num22 = 1.0;
            num21 = 2.0;
        } else {
            num22 = 0.07;
            num21 = 0.5;
        }
    }
    if net_id == 625 {
        if npc.wet {
            num22 = 1.0;
            num21 = 2.5;
        } else {
            num22 = 0.07;
            num21 = 0.2;
        }
    }
    if c.flag8 {
        num21 = 2.0;
        num22 = 1.0;
    }
    if npc.friendly && (c.flag17 | flag22) {
        num21 = 1.5 + (1.0 - npc.hp as f32 / npc.max_hp as f32) * 0.9;
        num22 = 0.1;
    }
    if c.flag14 && npc.wet {
        num21 = 2.0;
        num22 = 0.2;
    }
    if c.flag10 && npc.wet {
        if npc.vx.abs() < 0.05 && npc.vy.abs() < 0.05 {
            npc.vx += num21 * 10.0 * npc.facing as f32;
        } else {
            npc.vx *= 0.9;
        }
    } else if npc.vx < 0.0 - num21 || npc.vx > num21 {
        if npc.vy == 0.0 {
            npc.vx *= 0.8;
            npc.vy *= 0.8;
        }
    } else if npc.vx < num21 && npc.facing == 1 {
        npc.vx += num22;
        if npc.vx > num21 {
            npc.vx = num21;
        }
    } else if npc.vx > 0.0 - num21 && npc.facing == -1 {
        npc.vx -= num22;
        // `:53946-53949` — the C# quirk: the clamp tests `> num21`
        // (POSITIVE) even on the negative branch, so it never fires.
        if npc.vx > num21 {
            npc.vx = num21;
        }
    }
    // `:53951-53966` — the step helpers. flag23: the home tile's row
    // minus 32 px is NOT above the NPC (the C# `homeTileY * 16 - 32 >
    // position.Y` → false).
    let home_tile_y = npc.home.map(|(_, y)| y).unwrap_or(-1);
    let flag23 = !((home_tile_y * 16 - 32) as f32 > npc.y);
    if !flag23 && npc.vy == 0.0 {
        step_down(world, npc);
    }
    if npc.vy >= 0.0 {
        town_step_up(world, npc, flag23);
    }
    // `:53967-54196` — the grounded walk logic.
    if npc.vy == 0.0 {
        let num23 = ((npc.x + w / 2.0 + 15.0 * npc.facing as f32) / 16.0) as i32;
        let num24 = ((npc.y + h - 16.0) / 16.0) as i32;
        let num25 = 180.0f32;
        let (mut keepwalking3, mut avoid_falling3) = get_walk_prediction(
            world, npc, env.npcs, c.num6, c.floor_x, c.flag12, flag22, num23, num24,
        );
        let mut flag24 = false;
        let mut flag25 = false;
        // `:53975-53992` — the drown jump.
        if npc.wet && !c.flag12 && npc.kind.is_town_npc_like() && {
            flag25 = check_if_will_drown(flag22);
            flag25
        } && npc.local_ai[3] <= 0.0
        {
            avoid_falling3 = true;
            npc.local_ai[3] = num25;
            let mut num26 = 0;
            let mut num27 = 0;
            while num27 <= 10 && tile_at(world, num23 - npc.facing, num24 - num27).is_some_and(|t| t.liquid != 0) {
                num26 += 1;
                num27 += 1;
            }
            let mut num28 = ((num26 * 16 + 16) as f32 * 2.0 * 0.3).sqrt();
            if num28 > 26.0 {
                num28 = 26.0;
            }
            npc.vy = 0.0 - num28;
            npc.local_ai[3] = npc.x;
            flag24 = true;
        }
        // `:53993-53917` — the ledge bail: ≤2 solid supports below.
        if avoid_falling3 && !flag24 {
            let num29 = ((npc.x + w / 2.0) / 16.0) as i32;
            let mut num30 = 0;
            for num31 in -1..=1 {
                if tile_at(world, num29 + num31, num24 + 1).is_some_and(|t| nactive(t) && raw_solid(t.kind)) {
                    num30 += 1;
                }
            }
            if num30 <= 2 {
                if npc.vx != 0.0 {
                    npc.net_update = true;
                }
                keepwalking3 = false;
                avoid_falling3 = false;
                npc.ai[0] = 0.0;
                npc.ai[1] = 50.0 + rng.next_max(50) as f32;
                npc.ai[2] = 0.0;
                npc.local_ai[3] = 40.0;
            }
        }
        // `:54018-54023` — the stuck-at-the-jump-x flip.
        if npc.x == npc.local_ai[3] && !flag24 {
            npc.facing *= -1;
            npc.net_update = true;
            npc.local_ai[3] = num25;
        }
        // `:54024-54038` — the drown cooldown decay.
        if flag22 && !flag24 {
            if npc.local_ai[3] > num25 {
                npc.local_ai[3] = num25;
            }
            if npc.local_ai[3] > 0.0 {
                npc.local_ai[3] -= 1.0;
            }
        } else {
            npc.local_ai[3] = -1.0;
        }
        // `:54039-54080` — the door open.
        let tile3 = tile_at(world, num23, num24);
        let tile4 = tile_at(world, num23, num24 - 1);
        let tile5 = tile_at(world, num23, num24 - 2);
        let flag26 = h / 16.0 < 3.0;
        let door_tile = tile5.is_some_and(|t| nactive(t) && (t.kind == 10 || t.kind == 388));
        if c.town_npc_flag && door_tile && (rng.next_max(10) == 0 || c.flag) {
            if env.net_mode != 1 {
                // `:54047-54078` — the three open attempts in C# order:
                // own direction, reverse, tall gate; the first success
                // arms the latch + ai[1] += 80, the all-fail flips.
                let opened = doors.open_door(num23, num24 - 2, npc.facing)
                    || doors.open_door(num23, num24 - 2, -npc.facing)
                    || doors.shift_tall_gate(num23, num24 - 2, false);
                if opened {
                    door.close_door = true;
                    door.door_x = num23;
                    door.door_y = num24 - 2;
                    npc.net_update = true;
                    npc.ai[1] += 80.0;
                } else {
                    npc.facing *= -1;
                    npc.net_update = true;
                }
            }
        } else {
            // `:54083-54186` — the obstacle/jump/turn logic.
            if (npc.vx < 0.0 && npc.facing == -1) || (npc.vx > 0.0 && npc.facing == 1) {
                let mut flag27 = false; // turn around
                let mut flag28 = false; // cornered (state 8)
                let t5_solid = tile5.is_some_and(|t| nactive(t) && raw_solid(t.kind) && !raw_solid_top(t.kind));
                let t4_solid = tile4.is_some_and(|t| nactive(t) && raw_solid(t.kind) && !raw_solid_top(t.kind));
                if t5_solid && (!flag26 || t4_solid) {
                    // The 3-tile wall: jump −6 when the head space clears.
                    if !solid_tiles_versatile(world, num23 - npc.facing * 2, num23 - npc.facing, num24 - 5, num24 - 1)
                        && !solid_tiles(world, num23, num23, num24 - 5, num24 - 3)
                    {
                        npc.vy = -6.0;
                        npc.net_update = true;
                    } else if c.flag8 {
                        if worldgen_solid_tile(world, (npc.x + w / 2.0) as i32 / 16 + npc.facing, (npc.y + h / 2.0) as i32 / 16) {
                            npc.facing *= -1;
                            npc.vx *= 0.0;
                            npc.net_update = true;
                        }
                    } else if c.flag17 {
                        flag28 = true;
                        flag27 = true;
                    } else if !flag25 {
                        flag27 = true;
                    }
                } else if t4_solid {
                    // The 2-tile wall: jump −5.
                    if !solid_tiles_versatile(world, num23 - npc.facing * 2, num23 - npc.facing, num24 - 4, num24 - 1)
                        && !solid_tiles(world, num23, num23, num24 - 4, num24 - 2)
                    {
                        npc.vy = -5.0;
                        npc.net_update = true;
                    } else if c.flag17 {
                        flag28 = true;
                        flag27 = true;
                    } else {
                        flag27 = true;
                    }
                } else if npc.y + h - (num24 * 16) as f32 > 20.0
                    && tile3.is_some_and(|t| nactive(t) && raw_solid(t.kind) && !top_slope(t))
                {
                    // The 1-tile ledge: jump −4.4.
                    if !solid_tiles_versatile(world, num23 - npc.facing * 2, num23, num24 - 3, num24 - 1) {
                        npc.vy = -4.4;
                        npc.net_update = true;
                    } else if c.flag17 {
                        flag28 = true;
                        flag27 = true;
                    } else {
                        flag27 = true;
                    }
                } else if avoid_falling3 {
                    if !flag25 {
                        flag27 = true;
                    }
                    if c.flag17 {
                        flag28 = true;
                    }
                } else if c.flag15
                    && !solid_tiles_versatile(world, num23 - npc.facing * 2, num23 - npc.facing, num24 - 2, num24 - 1)
                {
                    npc.vy = -5.0;
                    npc.net_update = true;
                }
                if flag28 {
                    keepwalking3 = false;
                    npc.vx = 0.0;
                    npc.ai[0] = 8.0;
                    npc.ai[1] = 240.0;
                    npc.net_update = true;
                }
                if flag27 {
                    npc.facing *= -1;
                    npc.vx *= -1.0;
                    npc.net_update = true;
                }
                if keepwalking3 {
                    npc.ai[1] = 90.0;
                    npc.net_update = true;
                }
                if npc.vy < 0.0 {
                    npc.local_ai[3] = npc.x;
                }
            }
            // `:54187-54194` — the wet/critter jump boost.
            if npc.vy < 0.0 && npc.wet {
                npc.vy *= 1.2;
            }
            if npc.vy < 0.0 && town_critter(net_id) && !c.flag8 {
                npc.vy *= 1.2;
            }
        }
    } else if c.flag15 && !npc.wet {
        // `:54197-54225` — the airborne slime's landing scan.
        let num32 = ((npc.x + w / 2.0) / 16.0) as i32;
        let num33 = ((npc.y + h - 16.0) / 16.0) as i32;
        let mut num34 = 0;
        for num35 in -1..=1 {
            for num36 in 1..=6 {
                if tile_at(world, num32 + num35, num33 + num36)
                    .is_some_and(|t| t.liquid > 0 || (nactive(t) && raw_solid(t.kind)))
                {
                    num34 += 1;
                }
            }
        }
        if num34 <= 2 {
            if npc.vx != 0.0 {
                npc.net_update = true;
            }
            npc.vx *= 0.2;
            npc.ai[0] = 0.0;
            npc.ai[1] = 50.0 + rng.next_max(50) as f32;
            npc.ai[2] = 0.0;
            npc.local_ai[3] = 40.0;
        }
    }
    // `:54226-54242` — the stuck flip: position unchanged and velocity
    // unchanged-plus-gravity for 30 ticks. The C#'s `this.localAI[2]++
    // >= 30f` is a POST-increment: the comparison reads the OLD value,
    // the counter advances every tick the first four conjuncts hold
    // (the `&&` chain short-circuits before it when they don't).
    let stuck_shape = npc.old_x == npc.x
        && npc.old_y == npc.y
        && npc.old_vx == npc.vx
        && npc.old_vy == npc.vy + NPC_GRAVITY
        && !c.flag17;
    let stuck_count = npc.local_ai[2];
    if stuck_shape {
        npc.local_ai[2] += 1.0;
    }
    if stuck_shape && stuck_count >= 30.0 {
        npc.local_ai[2] = 0.0;
        if env.net_mode != 1 {
            npc.ai[0] = 1.0;
            npc.ai[1] = 200.0 + rng.next_max(300) as f32;
            npc.ai[2] = 0.0;
            if town_critter(net_id) {
                npc.ai[1] += rng.next_range(200, 400) as f32;
            }
            npc.facing *= -1;
            npc.vx *= -1.0;
            npc.net_update = true;
        }
    }
}

/// `NPC.gravity`'s stock value (`NPC.cs:90881`) — the stuck test's
/// `velocity + (0, NPC.gravity)` probe. The generic UpdateNPC tail sets
/// it per tick; the town arm reads the stock 0.3.
const NPC_GRAVITY: f32 = 0.3;

// ---------------------------------------------------------------------
// The social ladder (NPC.cs:55197-55421)
// ---------------------------------------------------------------------

struct SocialCtx {
    can_talk: bool,
    flag31: bool,
    flag17: bool,
    num12: f32,
    num13: f32,
    num14: i32,
    num3: f32,
    num: i32,
    who_am_i: usize,
}

fn town_social_ladder(
    npc: &mut Npc,
    env: &TownEnv<'_>,
    rng: &mut UnifiedRandom,
    out: &mut TownOutputs,
    c: SocialCtx,
) {
    let world = env.world;
    let net_id = npc.kind.net_id();
    let (w, h) = npc.size();
    let cx = npc.x + w / 2.0;
    let cy = npc.y + h / 2.0;
    // The else-if chain, in C# order. Each gate draws ONLY when reached.
    if c.can_talk && c.flag31 && npc.ai[0] == 0.0 && npc.vy == 0.0 && rng.next_max(300) == 0 {
        // `:55231-55257` — the NPC-to-NPC talk (states 3/4).
        let mut num92 = 420.0f32;
        num92 *= if rng.next_max(2) != 0 {
            rng.next_range(1, 3) as f32
        } else {
            rng.next_range(1, 4) as f32
        };
        for (m, other) in env.npcs {
            let busy = (other.ai[0] == 1.0 && other.close_door)
                || (other.ai[0] == 1.0 && other.ai[1] > 200.0)
                || other.ai[0] > 1.0
                || other.wet;
            if other.can_be_talked_to
                && !busy
                && other.distance(cx, cy) < 100.0
                && other.distance(cx, cy) > 20.0
                && can_hit_centers(world, cx, cy, other.center().0, other.center().1)
            {
                let num96 = to_direction_int(npc.x < other.x);
                npc.ai[0] = 3.0;
                npc.ai[1] = num92;
                npc.ai[2] = *m as f32;
                npc.facing = num96;
                npc.net_update = true;
                out.partner_writes.push(TownPartnerWrite::Talk {
                    slot: *m,
                    state: 4.0,
                    timer: num92,
                    partner: c.who_am_i as f32,
                    direction: -num96,
                    gossip: false,
                });
                break;
            }
        }
    } else if c.can_talk && c.flag31 && npc.ai[0] == 0.0 && npc.vy == 0.0 && rng.next_max(1800) == 0 {
        // `:55258-55288` — the gossip (states 16/17), no pets.
        let mut num97 = 420.0f32;
        num97 *= if rng.next_max(2) != 0 {
            rng.next_range(1, 3) as f32
        } else {
            rng.next_range(1, 4) as f32
        };
        for (m, other) in env.npcs {
            let busy = (other.ai[0] == 1.0 && other.close_door)
                || (other.ai[0] == 1.0 && other.ai[1] > 200.0)
                || other.ai[0] > 1.0
                || other.wet;
            if other.can_be_talked_to
                && !is_town_pet(other.net_id)
                && !busy
                && other.distance(cx, cy) < 100.0
                && other.distance(cx, cy) > 20.0
                && can_hit_centers(world, cx, cy, other.center().0, other.center().1)
            {
                let num101 = to_direction_int(npc.x < other.x);
                npc.ai[0] = 16.0;
                npc.ai[1] = num97;
                npc.ai[2] = *m as f32;
                npc.local_ai[2] = rng.next_max(4) as f32;
                npc.local_ai[3] = rng.next_max(3 - npc.local_ai[2] as i32) as f32;
                npc.facing = num101;
                npc.net_update = true;
                out.partner_writes.push(TownPartnerWrite::Talk {
                    slot: *m,
                    state: 17.0,
                    timer: num97,
                    partner: c.who_am_i as f32,
                    direction: -num101,
                    gossip: true,
                });
                break;
            }
        }
    } else if !is_town_pet(net_id)
        && c.flag31
        && npc.ai[0] == 0.0
        && npc.vy == 0.0
        && rng.next_max(1200) == 0
        && (net_id == 208 || (env.party_up && attack_type(net_id) == attack_type(208)))
    {
        // `:55289-55307` — the Party Girl's party talk (state 6). W4-f41
        // closeout census (research/70 §8h): there is NO C# party-gathering
        // behavior — town NPCs do NOT cluster near the party-giver; the
        // full `BirthdayParty` reference census (13 files) finds only this
        // talk gate + the facing state machine (`:54298-54328`, ported
        // below at the 6/7/18/19 arm), the hats (c39/d55), the chat text
        // (d120), the slime inside-items (b1), the PartyBunny critter spawn
        // (spawner lane), the emote probe (UI lane), the alt-texture
        // (NPC-texture lane), the bunny-statue swap (wiring lane) and the
        // NerdySlime spawn gate (town-spawn lane). This gate + state machine
        // are the S1-D-3 port; the aiStyle-7 axis WIRING is W4-b3's
        // (96a5d88f0: the axis arm npc.rs:4722 + the ai_007_town_family
        // adapter :10121) and the live `party_up` feed at the TownEnv build
        // is b3 phase 2's declared follow-up (its npc.rs:10160 still
        // `party_up: false` — the solo adapter takes (&World, &Player), no
        // App, so the feed rides the phase-2 live-list/talk threading) —
        // co-lane, not ported here. The f41 party-center-FALSE census
        // (research/70 §8h/§9b) confirms this gate + the facing machine are
        // the ONLY party-position-adjacent town behavior the C# has.
        for (n, p) in env.players.iter().enumerate() {
            if p.candidate.active
                && !p.candidate.dead
                && player_distance(p, cx, cy) < 150.0
                && can_hit_line_centers(world, npc.x + w / 2.0, npc.y, p.candidate.x + p.candidate.width as f32 / 2.0, p.candidate.y)
            {
                let num105 = to_direction_int(npc.x < p.candidate.x);
                npc.ai[0] = 6.0;
                npc.ai[1] = 300.0;
                npc.ai[2] = n as f32;
                npc.facing = num105;
                npc.net_update = true;
                break;
            }
        }
    } else if c.flag31 && npc.ai[0] == 0.0 && npc.vy == 0.0 && rng.next_max(600) == 0 && net_id == 550 {
        // `:55308-55326` — the Bartender's talk (state 18).
        for (n, p) in env.players.iter().enumerate() {
            if p.candidate.active
                && !p.candidate.dead
                && player_distance(p, cx, cy) < 150.0
                && can_hit_line_centers(world, npc.x + w / 2.0, npc.y, p.candidate.x + p.candidate.width as f32 / 2.0, p.candidate.y)
            {
                let num109 = to_direction_int(npc.x < p.candidate.x);
                npc.ai[0] = 18.0;
                npc.ai[1] = 300.0;
                npc.ai[2] = n as f32;
                npc.facing = num109;
                npc.net_update = true;
                break;
            }
        }
    } else if !is_town_pet(net_id) && c.flag31 && npc.ai[0] == 0.0 && npc.vy == 0.0 && rng.next_max(1800) == 0 {
        // `:55327-55332` — the small talk (state 2). Next(1,2) == 1.
        npc.ai[0] = 2.0;
        npc.ai[1] = 45.0 * rng.next_range(1, 2) as f32;
        npc.net_update = true;
    } else if c.flag31 && npc.ai[0] == 0.0 && npc.vy == 0.0 && rng.next_max(600) == 0 && net_id == 229 {
        // `:55333-55338` — the Tax Collector's grumble (state 11).
        npc.ai[0] = 11.0;
        npc.ai[1] = 30.0 * rng.next_range(1, 4) as f32;
        npc.net_update = true;
    } else if c.flag31 && npc.ai[0] == 0.0 && npc.vy == 0.0 && rng.next_max(1200) == 0 {
        // `:55339-55357` — the player-facing talk (state 7).
        for (n, p) in env.players.iter().enumerate() {
            if p.can_be_talked_to
                && player_distance(p, cx, cy) < 150.0
                && can_hit_line_centers(world, npc.x + w / 2.0, npc.y, p.candidate.x + p.candidate.width as f32 / 2.0, p.candidate.y)
            {
                let num113 = to_direction_int(npc.x < p.candidate.x);
                npc.ai[0] = 7.0;
                npc.ai[1] = 220.0;
                npc.ai[2] = n as f32;
                npc.facing = num113;
                npc.net_update = true;
                break;
            }
        }
    } else if c.flag31 && npc.ai[0] == 1.0 && npc.vy == 0.0 && c.num > 0 && rng.next_max(c.num) == 0 {
        // `:55358-55400` — the walk-up sit (state 5).
        let point = ((npc.x + w / 2.0) as i32 / 16, (npc.y + h - 2.0) as i32 / 16);
        let mut flag35 = in_world_fluff(world, point.0, point.1, 1);
        if flag35 {
            for (_, other) in env.npcs {
                if other.town_ai7 && other.ai[0] == 5.0 && other.sit_tile() == point {
                    flag35 = false;
                    break;
                }
            }
            if flag35 {
                for p in env.players {
                    if p.candidate.active && p.sitting {
                        let p_seat = (
                            (p.candidate.x + p.candidate.width as f32 / 2.0) as i32 / 16,
                            (p.candidate.y + p.candidate.height as f32 / 2.0) as i32 / 16,
                        );
                        if p_seat == point {
                            flag35 = false;
                            break;
                        }
                    }
                }
            }
        }
        if flag35 {
            if let Some(t) = tile_at(world, point.0, point.1) {
                flag35 = t.kind == 15 || t.kind == 497;
                if flag35 && t.kind == 15 && (1080..=1098).contains(&i32::from(t.frame_y)) {
                    flag35 = false;
                }
                if flag35 {
                    npc.ai[0] = 5.0;
                    npc.ai[1] = 900.0 + rng.next_max(10800) as f32;
                    npc.facing = if t.frame_x != 0 { 1 } else { -1 };
                    // `:55394` — Bottom = (x*16+8+2*dir, y*16+16).
                    npc.x = point.0 as f32 * 16.0 + 8.0 + 2.0 * npc.facing as f32 - w / 2.0;
                    npc.y = point.1 as f32 * 16.0 + 16.0 - h;
                    npc.vx = 0.0;
                    npc.vy = 0.0;
                    npc.local_ai[3] = 0.0;
                    npc.net_update = true;
                }
            }
        }
    } else if c.flag31 && npc.ai[0] == 1.0 && npc.vy == 0.0 && rng.next_max(600) == 0 && body_line_avoids(world, npc.x, npc.y, w, h) {
        // `:55401-55421` — the interact pause (state 9).
        let point2 = ((cx + npc.facing as f32 * 10.0) as i32 / 16, (cy as i32) / 16);
        let mut flag36 = in_world_fluff(world, point2.0, point2.1, 1);
        if flag36 {
            if let Some(t) = tile_at(world, point2.0, point2.1) {
                if !nactive(t) || !interactable_by_npcs(t.kind) {
                    flag36 = false;
                }
            } else {
                flag36 = false;
            }
        }
        if flag36 {
            npc.ai[0] = 9.0;
            npc.ai[1] = 40.0 + rng.next_max(90) as f32;
            npc.vx = 0.0;
            npc.vy = 0.0;
            npc.local_ai[3] = 0.0;
            npc.net_update = true;
        }
    }
    let _ = (c.flag17, c.num12, c.num13, c.num14, c.num3);
}

// ---------------------------------------------------------------------
// The attack bodies (NPC.cs:54329-55122)
// ---------------------------------------------------------------------

struct AttackCtx {
    num2: f32,
    num14: i32,
    num15: i32,
    num16: i32,
    flag17: bool,
    who_am_i: usize,
}

/// The shared aim guard (`:54497-54500` etc.): NaN or wrong-side →
/// `(spriteDirection, -1)` for the arc, `(spriteDirection, 0)` for the
/// straight/magic.
fn aim_guard(vec: (f32, f32), sprite_dir: i32, fallback_y: f32) -> (f32, f32) {
    if vec.0.is_nan() || vec.1.is_nan() || sign_f(vec.0) != sprite_dir {
        (sprite_dir as f32, fallback_y)
    } else {
        vec
    }
}

/// The target lookups — the danger scan's slots index `Main.npc[]`.
/// The stinky-player fallback (`:53539-53548`) stores PLAYER slots into
/// the same num15/num16, and the attack bodies then read
/// `Main.npc[num15]` — a vanilla quirk (the player index is read as an
/// NPC index); the port keeps it verbatim: the lookup is always the
/// NPC list, a player-slot index simply finds whatever NPC sits there
/// (or nothing).
fn target_view<'a>(env: &'a TownEnv<'_>, slot: i32) -> Option<&'a TownNpcView> {
    env.npcs.iter().find(|(s, _)| *s == slot as usize).map(|(_, v)| v)
}

fn target_center(env: &TownEnv<'_>, slot: i32) -> (f32, f32) {
    target_view(env, slot).map(|v| v.center()).unwrap_or((0.0, 0.0))
}

fn target_x_of(env: &TownEnv<'_>, slot: i32) -> f32 {
    target_view(env, slot).map(|v| v.x).unwrap_or(0.0)
}

fn target_distance(env: &TownEnv<'_>, cx: f32, cy: f32, slot: i32) -> f32 {
    target_view(env, slot).map(|v| v.distance(cx, cy)).unwrap_or(f32::MAX)
}

fn target_can_hit(env: &TownEnv<'_>, cx: f32, cy: f32, slot: i32) -> bool {
    let (tx, ty) = target_center(env, slot);
    can_hit_centers(env.world, cx, cy, tx, ty)
}

fn target_can_hit_line(env: &TownEnv<'_>, cx: f32, cy: f32, slot: i32) -> bool {
    let (tx, ty) = target_center(env, slot);
    can_hit_line_centers(env.world, cx, cy, tx, ty)
}

fn player_distance(p: &TownPlayer, cx: f32, cy: f32) -> f32 {
    let px = p.candidate.x + p.candidate.width as f32 / 2.0;
    let py = p.candidate.y + p.candidate.height as f32 / 2.0;
    ((px - cx) * (px - cx) + (py - cy) * (py - cy)).sqrt()
}

/// `NPCID.Sets.Skeletons` (`ID/NPCID.cs:10478`) — the Skeleton
/// Merchant's ignore list (`NPC.cs:53483`'s `type == 453` gate).
fn is_skeleton(net_id: i32) -> bool {
    matches!(
        net_id,
        21 | 44
            | 77
            | 110
            | 167
            | 197
            | 201..=203
            | 291..=293
            | 322..=324
            | 449..=452
            | 481
            | 635
    )
}

/// Attack 10 — the ranged arc (`NPC.cs:54329-54524`).
fn town_attack_ranged_arc(npc: &mut Npc, env: &TownEnv<'_>, rng: &mut UnifiedRandom, out: &mut TownOutputs, c: AttackCtx) {
    let net_id = npc.kind.net_id();
    let (w, h) = npc.size();
    let cx = npc.x + w / 2.0;
    let cy = npc.y + h / 2.0;
    // `:54346-54481` — the per-type preset table.
    let (mut proj, mut speed, mut dmg, mut fire_at, mut cooldown, mut cd_rand, mut lead, mut kb) =
        (0i32, 0.0f32, 0i32, 0i32, 0i32, 0i32, 0.0f32, 0.0f32);
    match net_id {
        38 => (proj, speed, dmg, fire_at, cooldown, cd_rand, lead, kb) = (30, 6.0, 20, 10, 180, 120, 16.0, 7.0),
        633 => {
            (proj, speed, dmg, fire_at, cooldown, cd_rand, lead, kb) = (880, 24.0, 15, 1, 15, 10, 0.0, 7.0);
            // ShouldBestiaryGirlBeLycantrope: the full-moon night gate
            // — the port's env carries no moon phase; the classic
            // (non-lycanthrope) row is the default, recorded deviation.
        }
        550 => (proj, speed, dmg, fire_at, cooldown, cd_rand, lead, kb) = (669, 6.0, 24, 10, 120, 60, 16.0, 9.0),
        588 => (proj, speed, dmg, fire_at, cooldown, cd_rand, lead, kb) = (721, 8.0, 15, 5, 20, 10, 16.0, 9.0),
        208 => (proj, speed, dmg, fire_at, cooldown, cd_rand, lead, kb) = (588, 6.0, 30, 10, 60, 120, 16.0, 6.0),
        17 => (proj, speed, dmg, fire_at, cooldown, cd_rand, lead, kb) = (48, 9.0, 12, 10, 60, 60, 16.0, 1.5),
        369 => (proj, speed, dmg, fire_at, cooldown, cd_rand, lead, kb) = (520, 12.0, 10, 10, 0, 1, 16.0, 3.0),
        453 => (proj, speed, dmg, fire_at, cooldown, cd_rand, lead, kb) = (21, 14.0, 14, 10, 0, 1, 16.0, 3.0),
        107 => (proj, speed, dmg, fire_at, cooldown, cd_rand, lead, kb) = (24, 5.0, 15, 10, 60, 60, 16.0, 1.0),
        124 => (proj, speed, dmg, fire_at, cooldown, cd_rand, lead, kb) = (582, 10.0, 11, 1, 30, 30, 0.0, 3.5),
        18 => (proj, speed, dmg, fire_at, cooldown, cd_rand, lead, kb) = (583, 8.0, 8, 1, 15, 10, 10.0, 2.0),
        142 => (proj, speed, dmg, fire_at, cooldown, cd_rand, lead, kb) = (589, 7.0, 22, 1, 10, 1, 10.0, 2.0),
        _ => {}
    }
    let dmg = town_attack_damage(dmg as f32 * c.num2, env.town_damage_multiplier);
    if attack_time(net_id) as f32 == npc.ai[1] {
        npc.local_ai[3] = 0.0; // frameCounter = 0 rides the frame seam
    }
    npc.vx *= 0.8;
    npc.ai[1] -= 1.0;
    npc.local_ai[3] += 1.0;
    if npc.local_ai[3] == fire_at as f32 && env.net_mode != 1 {
        let num45 = danger_detect_range(net_id) as f32;
        let mut vec = (0.0f32, -1.0f32);
        if c.num14 == 1 && npc.sprite_dir == 1 && c.num16 != -1 {
            let (tx, ty) = target_center(env, c.num16);
            let dist = target_distance(env, cx, cy, c.num16);
            let clamp = (dist / num45).clamp(0.0, 1.0);
            vec = direction_to(cx, cy, tx, ty - lead * clamp);
        }
        if c.num14 == -1 && npc.sprite_dir == -1 && c.num15 != -1 {
            let (tx, ty) = target_center(env, c.num15);
            let dist = target_distance(env, cx, cy, c.num15);
            let clamp = (dist / num45).clamp(0.0, 1.0);
            vec = direction_to(cx, cy, tx, ty - lead * clamp);
        }
        vec = aim_guard(vec, npc.sprite_dir, -1.0);
        vec = (vec.0 * speed, vec.1 * speed);
        // num46 = 0 — the jitter range is zero, but RandomVector2 still
        // DRAWS its two NextDouble calls (`:54502`).
        let (jx, jy) = random_vector2(rng, 0.0, 0.0);
        vec = (vec.0 + jx, vec.1 + jy);
        let ai0 = if net_id == 142 { rng.next_max(5) as f32 } else { 0.0 };
        let ai1 = if net_id == 124 { c.who_am_i as f32 } else { 0.0 };
        out.shots.push(TownShot {
            kind: proj,
            x: cx + (npc.sprite_dir * 16) as f32,
            y: cy - 2.0,
            vx: vec.0,
            vy: vec.1,
            damage: dmg,
            knockback: kb,
            ai0,
            ai1,
            time_left_override: if net_id == 588 || net_id == 107 { Some(480) } else { None },
            // `:54504` — only the Mechanic's call passes the variation.
            variation_index: if net_id == 124 { npc.variation_index } else { 0 },
        });
    }
    if npc.ai[1] <= 0.0 {
        npc.ai[0] = if npc.local_ai[2] == 8.0 && c.flag17 { 8.0 } else { 0.0 };
        npc.ai[1] = cooldown as f32 + rng.next_max(cd_rand) as f32;
        npc.ai[2] = 0.0;
        let half = cooldown / 2 + rng.next_max(cd_rand);
        npc.local_ai[1] = half as f32;
        npc.local_ai[3] = half as f32;
        npc.net_update = true;
    }
}

/// Attack 12 — the straight shot (`NPC.cs:54525-54796`).
fn town_attack_straight(npc: &mut Npc, env: &TownEnv<'_>, rng: &mut UnifiedRandom, out: &mut TownOutputs, c: AttackCtx) {
    let net_id = npc.kind.net_id();
    let (w, h) = npc.size();
    let cx = npc.x + w / 2.0;
    let cy = npc.y + h / 2.0;
    let (mut proj, mut speed, mut dmg, mut cooldown, mut cd_rand, mut kb, mut fire_at, mut lead, mut jitter) =
        (0i32, 0.0f32, 0i32, 0i32, 0i32, 0.0f32, 0i32, 0i32, 0.0f32);
    let mut volley = false; // flag29 — the multi-shot ladder
    match net_id {
        19 => {
            (proj, speed, dmg, cooldown, cd_rand, kb, fire_at, lead, jitter) = (14, 13.0, 24, 14, 4, 3.0, 1, 0, 0.5);
            if env.hard_mode {
                dmg = 15;
                if npc.local_ai[3] > fire_at as f32 {
                    fire_at = 10;
                    volley = true;
                }
                if npc.local_ai[3] > fire_at as f32 {
                    fire_at = 20;
                    volley = true;
                }
                if npc.local_ai[3] > fire_at as f32 {
                    fire_at = 30;
                    volley = true;
                }
            }
        }
        227 => {
            (proj, speed, dmg, cooldown, cd_rand, kb, fire_at, lead, jitter) = (587, 10.0, 8, 10, 1, 1.75, 1, 0, 0.5);
            if npc.local_ai[3] > fire_at as f32 {
                fire_at = 12;
                volley = true;
            }
            if npc.local_ai[3] > fire_at as f32 {
                fire_at = 24;
                volley = true;
            }
            if env.hard_mode {
                dmg += 2;
            }
        }
        368 => {
            (proj, speed, dmg, cooldown, cd_rand, kb, fire_at, lead, jitter) = (14, 13.0, 24, 12, 5, 2.0, 1, 0, 0.2);
            if env.hard_mode {
                dmg = 30;
                proj = 357;
            }
        }
        22 => {
            speed = 10.0;
            dmg = 12;
            fire_at = 1;
            if env.hard_mode {
                (proj, cooldown, cd_rand) = (2, 15, 10);
                dmg += 6;
            } else {
                (proj, cooldown, cd_rand) = (1, 30, 20);
            }
            kb = 2.75;
            lead = 4;
            jitter = 0.7;
        }
        228 => (proj, speed, dmg, cooldown, cd_rand, kb, fire_at, lead, jitter) = (267, 14.0, 20, 10, 1, 3.0, 1, 6, 0.4),
        178 => {
            (proj, speed, dmg, cooldown, cd_rand, kb, fire_at, lead, jitter) =
                (242, 13.0, if env.hard_mode { 15 } else { 11 }, 10, 1, 2.0, 1, 0, 0.3);
            if npc.local_ai[3] > fire_at as f32 {
                fire_at = 8;
                volley = true;
            }
            if npc.local_ai[3] > fire_at as f32 {
                fire_at = 16;
                volley = true;
            }
        }
        229 => {
            (proj, speed, dmg, cooldown, cd_rand, kb, fire_at, lead, jitter) = (14, 14.0, 24, 10, 1, 2.0, 1, 0, 0.7);
            for step in [16, 24, 32, 40, 48] {
                if npc.local_ai[3] > fire_at as f32 {
                    fire_at = step;
                    volley = true;
                }
            }
            // `:54717-54724` — the close-range coin burst.
            let num55 = if c.num14 == 1 && npc.sprite_dir == 1 {
                c.num16
            } else if c.num14 == -1 && npc.sprite_dir == -1 {
                c.num15
            } else {
                -1
            };
            if npc.local_ai[3] == 0.0 && num55 != -1 && target_distance(env, cx, cy, num55) < pretty_safe(net_id) as f32 {
                jitter = 0.1;
                proj = 162;
                dmg = 50;
                kb = 10.0;
                speed = 24.0;
            }
        }
        209 => {
            // `:54728` — SelectRandom(134, 133, 135): one Next(3) draw.
            let pick = select_random_index(rng, 3);
            proj = [134, 133, 135][pick as usize];
            fire_at = 1;
            match proj {
                135 => (speed, dmg, cooldown, cd_rand, kb, jitter) = (12.0, 30, 30, 10, 7.0, 0.2),
                133 => (speed, dmg, cooldown, cd_rand, kb, jitter) = (10.0, 25, 10, 1, 6.0, 0.2),
                134 => (speed, dmg, cooldown, cd_rand, kb, jitter) = (13.0, 20, 20, 10, 4.0, 0.1),
                _ => {}
            }
        }
        _ => {}
    }
    let dmg = town_attack_damage(dmg as f32 * c.num2, env.town_damage_multiplier);
    if attack_time(net_id) as f32 == npc.ai[1] {
        npc.local_ai[3] = 0.0;
    }
    npc.vx *= 0.8;
    npc.ai[1] -= 1.0;
    npc.local_ai[3] += 1.0;
    let num55 = if c.num14 == 1 && npc.sprite_dir == 1 {
        c.num16
    } else if c.num14 == -1 && npc.sprite_dir == -1 {
        c.num15
    } else {
        -1
    };
    if npc.local_ai[3] == fire_at as f32 && env.net_mode != 1 {
        let mut vec2 = (0.0f32, 0.0f32);
        if num55 != -1 {
            let (tx, ty) = target_center(env, num55);
            vec2 = direction_to(cx, cy, tx, ty - lead as f32);
        }
        vec2 = aim_guard(vec2, npc.sprite_dir, 0.0);
        vec2 = (vec2.0 * speed, vec2.1 * speed);
        let (jx, jy) = random_vector2(rng, -jitter, jitter);
        vec2 = (vec2.0 + jx, vec2.1 + jy);
        // The DyeTrader's ai1 = Next(12)/6 (`:54776`).
        let ai1 = if net_id == 227 { rng.next_max(12) as f32 / 6.0 } else { 0.0 };
        out.shots.push(TownShot {
            kind: proj,
            x: cx + (npc.sprite_dir * 16) as f32,
            y: cy - 2.0,
            vx: vec2.0,
            vy: vec2.1,
            damage: dmg,
            knockback: kb,
            ai0: 0.0,
            ai1,
            time_left_override: None,
            variation_index: 0,
        });
    }
    // `:54780-54787` — the volley's aim-Y latch into ai[2].
    if npc.local_ai[3] == fire_at as f32 && volley && num55 != -1 {
        let (tx, ty) = target_center(env, num55);
        let vector2 = direction_to(cx, cy, tx, ty);
        if (-0.5..=0.5).contains(&vector2.1) {
            npc.ai[2] = vector2.1;
        }
    }
    if npc.ai[1] <= 0.0 {
        npc.ai[0] = if npc.local_ai[2] == 8.0 && c.flag17 { 8.0 } else { 0.0 };
        npc.ai[1] = cooldown as f32 + rng.next_max(cd_rand) as f32;
        npc.ai[2] = 0.0;
        let half = cooldown / 2 + rng.next_max(cd_rand);
        npc.local_ai[1] = half as f32;
        npc.local_ai[3] = half as f32;
        npc.net_update = true;
    }
}

/// Attack 14 — the magic shot (`NPC.cs:54827-54994`).
fn town_attack_magic(npc: &mut Npc, env: &TownEnv<'_>, rng: &mut UnifiedRandom, out: &mut TownOutputs, c: AttackCtx) {
    let net_id = npc.kind.net_id();
    let (w, h) = npc.size();
    let cx = npc.x + w / 2.0;
    let cy = npc.y + h / 2.0;
    let (mut proj, mut speed, mut dmg, mut fire_at, mut cooldown, mut cd_rand, mut kb, mut lead, mut jitter) =
        (0i32, 0.0f32, 0i32, 0i32, 0i32, 0i32, 0.0f32, 0.0f32, 0.0f32);
    match net_id {
        54 => (proj, speed, dmg, fire_at, cooldown, cd_rand, kb, jitter) = (585, 10.0, 16, 30, 20, 15, 2.0, 1.0),
        108 => (proj, speed, dmg, fire_at, cooldown, cd_rand, kb, lead) = (15, 6.0, 18, 15, 15, 5, 3.0, 20.0),
        160 => {
            (proj, dmg, fire_at, cooldown, cd_rand, kb) = (590, 40, 15, 10, 1, 3.0);
            // `:54884-54886` — the fire_at ladder rounds UP to the next
            // 15 above localAI[3].
            while npc.local_ai[3] > fire_at as f32 {
                fire_at += 15;
            }
        }
        663 => {
            (proj, dmg, fire_at, cooldown, cd_rand, kb) = (950, if env.hard_mode { 20 } else { 15 }, 15, 0, 0, 3.0);
            while npc.local_ai[3] > fire_at as f32 {
                fire_at += 10;
            }
        }
        20 => (proj, fire_at, cooldown, cd_rand, kb) = (586, 24, 10, 1, 3.0),
        _ => {}
    }
    let dmg = town_attack_damage(dmg as f32 * c.num2, env.town_damage_multiplier);
    if attack_time(net_id) as f32 == npc.ai[1] {
        npc.local_ai[3] = 0.0;
    }
    npc.vx *= 0.8;
    npc.ai[1] -= 1.0;
    npc.local_ai[3] += 1.0;
    let num67 = if c.num14 == 1 && npc.sprite_dir == 1 {
        c.num16
    } else if c.num14 == -1 && npc.sprite_dir == -1 {
        c.num15
    } else {
        -1
    };
    if npc.local_ai[3] == fire_at as f32 && env.net_mode != 1 {
        let num64 = danger_detect_range(net_id) as f32;
        let mut vec4 = (0.0f32, 0.0f32);
        if num67 != -1 {
            let (tx, ty) = target_center(env, num67);
            let dist = target_distance(env, cx, cy, num67);
            let clamp = (dist / num64).clamp(0.0, 1.0);
            vec4 = direction_to(cx, cy, tx, ty - lead * clamp);
        }
        vec4 = aim_guard(vec4, npc.sprite_dir, 0.0);
        vec4 = (vec4.0 * speed, vec4.1 * speed);
        let (jx, jy) = random_vector2(rng, -jitter, jitter);
        vec4 = (vec4.0 + jx, vec4.1 + jy);
        let spawn_x = cx + (npc.sprite_dir * 16) as f32;
        let spawn_y = cy - 2.0;
        if net_id == 108 {
            // `:54925-54935` — the Wizard's bolt volley:
            // SelectRandom(1,1,1,1,2,2,3) = Next(7), then per bolt a
            // ±3.4 RandomVector2.
            let count = [1, 1, 1, 1, 2, 2, 3][select_random_index(rng, 7) as usize];
            for _ in 0..count {
                let (jx2, jy2) = random_vector2(rng, -3.4, 3.4);
                out.shots.push(TownShot {
                    kind: proj,
                    x: spawn_x,
                    y: spawn_y,
                    vx: vec4.0 + jx2,
                    vy: vec4.1 + jy2,
                    damage: dmg,
                    knockback: kb,
                    ai0: 0.0,
                    ai1: 0.0,
                    time_left_override: None,
                    variation_index: npc.variation_index,
                });
            }
        } else if net_id == 160 || net_id == 663 {
            // `:54936-54967` — the Party Girl / Princess spot spawn:
            // up to 10 / 5 retries of a random point near the target,
            // each retry drawing RandomVector2(0,1)×2. The C# reads
            // Main.npc[num67] unconditionally (an inactive slot reads
            // as a zero box); the port's live list may lack the slot,
            // so a missing view skips the spawn (no draws — the C#
            // would spawn at the inactive NPC's stale position, a
            // recorded seam residual).
            if let Some(target) = target_view(env, num67) {
                let (mut tries, scale) = if net_id == 160 { (10, 5.0f32) } else { (5, 1.0f32) };
                let base = if net_id == 160 {
                    (target.x - target.width * 2.0, target.y - target.height * 2.0)
                } else {
                    (target.x, target.y)
                };
                let mut spot = {
                    let (rx, ry) = random_vector2(rng, 0.0, 1.0);
                    (base.0 + target.width * rx * scale, base.1 + target.height * ry * scale)
                };
                while tries > 0 && worldgen_solid_tile(env.world, spot.0 as i32 / 16, spot.1 as i32 / 16) {
                    tries -= 1;
                    let (rx, ry) = random_vector2(rng, 0.0, 1.0);
                    spot = (base.0 + target.width * rx * scale, base.1 + target.height * ry * scale);
                }
                out.shots.push(TownShot {
                    kind: proj,
                    x: spot.0,
                    y: spot.1,
                    vx: 0.0,
                    vy: 0.0,
                    damage: dmg,
                    knockback: kb,
                    ai0: 0.0,
                    ai1: 0.0,
                    time_left_override: None,
                    variation_index: npc.variation_index,
                });
            }
        } else {
            // The Dryad (ai1 = whoAmI) and the Clothier (plain).
            out.shots.push(TownShot {
                kind: proj,
                x: spawn_x,
                y: spawn_y,
                vx: vec4.0,
                vy: vec4.1,
                damage: dmg,
                knockback: kb,
                ai0: 0.0,
                ai1: if net_id == 20 { c.who_am_i as f32 } else { 0.0 },
                time_left_override: None,
                variation_index: if net_id == 20 { npc.variation_index } else { 0 },
            });
        }
    }
    // `:54981-54985` — the magic aura light: the light task's.
    if npc.ai[1] <= 0.0 {
        npc.ai[0] = if npc.local_ai[2] == 8.0 && c.flag17 { 8.0 } else { 0.0 };
        npc.ai[1] = cooldown as f32 + rng.next_max(cd_rand) as f32;
        npc.ai[2] = 0.0;
        let half = cooldown / 2 + rng.next_max(cd_rand);
        npc.local_ai[1] = half as f32;
        npc.local_ai[3] = half as f32;
        npc.net_update = true;
    }
}

/// Attack 15 — the melee swing (`NPC.cs:54995-55122`). The sweep emits
/// [`TownStrike`] requests; `GetSwingStats`/`TweakSwingStats`
/// (`NPC.cs:66485-66609`) build the item rect.
fn town_attack_melee(npc: &mut Npc, env: &TownEnv<'_>, rng: &mut UnifiedRandom, out: &mut TownOutputs, c: AttackCtx) {
    let net_id = npc.kind.net_id();
    let (w, h) = npc.size();
    let cx = npc.x + w / 2.0;
    let cy = npc.y + h / 2.0;
    let (mut dmg, mut cooldown, mut cd_rand, mut speed_kb, mut item_w, mut item_h) = (0i32, 0i32, 0i32, 0.0f32, 0i32, 0i32);
    match net_id {
        207 => (dmg, item_w, item_h, cooldown, cd_rand, speed_kb) = (11, 32, 32, 12, 6, 4.25),
        441 => {
            (dmg, item_w, item_h, cooldown, cd_rand, speed_kb) = (9, 28, 28, 9, 3, 3.5);
            if npc.given_name == "Andrew" {
                dmg *= 2;
                speed_kb *= 2.0;
            }
        }
        353 => (dmg, item_w, item_h, cooldown, cd_rand, speed_kb) = (10, 32, 32, 15, 8, 5.0),
        _ if is_town_pet(net_id) => (dmg, item_w, item_h, cooldown, cd_rand, speed_kb) = (10, 32, 32, 15, 8, 3.0),
        _ => {}
    }
    let dmg = town_attack_damage(dmg as f32 * c.num2, env.town_damage_multiplier);
    if attack_time(net_id) as f32 == npc.ai[1] {
        npc.local_ai[3] = 0.0;
    }
    npc.vx *= 0.8;
    npc.ai[1] -= 1.0;
    if env.net_mode != 1 {
        // `:55060-55067` — the swing rect.
        let swing_max = attack_time(net_id) * 2;
        let swing_current = npc.ai[1] as i32;
        let (sx, sy) = get_swing_stats(cx, npc.y, swing_max, swing_current, npc.sprite_dir, item_w, item_h);
        let mut rect = (sx as i32 as f32, sy as i32 as f32, item_w as f32, item_h as f32);
        if npc.sprite_dir == -1 {
            rect.0 -= item_w as f32;
        }
        rect.1 -= item_h as f32;
        tweak_swing_stats(&mut rect, swing_max, swing_current, npc.sprite_dir);
        // `:55069-55078` — the sweep.
        for (m, other) in env.npcs {
            if other.immune_to_me
                && !other.dont_take_damage
                && !other.friendly
                && other.damage > 0
                && rect_intersects(rect, (other.x, other.y, other.width, other.height))
                && (other.no_tile_collide
                    || crate::projectile::can_hit(
                        env.world, npc.x, npc.y, w, h, other.x, other.y, other.width, other.height,
                    ))
            {
                out.strikes.push(TownStrike {
                    slot: *m,
                    damage: dmg,
                    knockback: speed_kb,
                    direction: npc.sprite_dir,
                    immune_ticks: npc.ai[1] as i32 + 2,
                });
            }
        }
    }
    if npc.ai[1] <= 0.0 {
        // `:55080-55121` — the re-swing arm.
        let mut flag30 = false;
        if c.flag17 {
            let probe_x = cx + -c.num14 as f32 * 32.0;
            if !can_hit_centers(env.world, cx, cy, probe_x, cy) || npc.local_ai[2] == 8.0 {
                flag30 = true;
            }
            if flag30 {
                let num83 = attack_time(net_id);
                let mut num84 = if c.num14 == 1 { c.num16 } else { c.num15 };
                let num85 = if c.num14 == 1 { c.num15 } else { c.num16 };
                if num84 != -1 && !target_can_hit(env, cx, cy, num84) {
                    num84 = if num85 == -1 || !target_can_hit(env, cx, cy, num85) { -1 } else { num85 };
                }
                if num84 != -1 {
                    npc.ai[0] = 15.0;
                    npc.ai[1] = num83 as f32;
                    npc.ai[2] = 0.0;
                    npc.local_ai[3] = 0.0;
                    npc.facing = if npc.x < target_x_of(env, num84) { 1 } else { -1 };
                    npc.net_update = true;
                } else {
                    flag30 = false;
                }
            }
        }
        if !flag30 {
            npc.ai[0] = if npc.local_ai[2] == 8.0 && c.flag17 { 8.0 } else { 0.0 };
            npc.ai[1] = cooldown as f32 + rng.next_max(cd_rand) as f32;
            npc.ai[2] = 0.0;
            let half = cooldown / 2 + rng.next_max(cd_rand);
            npc.local_ai[1] = half as f32;
            npc.local_ai[3] = half as f32;
            npc.net_update = true;
        }
    }
}

/// `NPC.GetSwingStats` (`NPC.cs:66485-66585`) — the swing's item
/// position through the three 1/3 phases. Returns the position only
/// (the rotation tuple element is the draw seam's).
fn get_swing_stats(center_x: f32, pos_y: f32, swing_max: i32, swing_current: i32, aim_dir: i32, item_w: i32, item_h: i32) -> (f32, f32) {
    let cur = swing_current as f64;
    let max = swing_max as f64;
    if cur < max * 0.333 {
        let mut num = 10.0f32;
        if item_w > 32 {
            num = 14.0;
        }
        if item_w >= 52 {
            num = 24.0;
        }
        if item_w >= 64 {
            num = 28.0;
        }
        if item_w >= 92 {
            num = 38.0;
        }
        (center_x + (item_w as f32 * 0.5 - num) * aim_dir as f32, pos_y + 24.0)
    } else if cur < max * 0.666 {
        let mut num2 = 10.0f32;
        if item_w > 32 {
            num2 = 18.0;
        }
        if item_w >= 52 {
            num2 = 24.0;
        }
        if item_w >= 64 {
            num2 = 28.0;
        }
        if item_w >= 92 {
            num2 = 38.0;
        }
        let x = center_x + (item_w as f32 * 0.5 - num2) * aim_dir as f32;
        let mut num2 = 10.0f32;
        if item_h > 32 {
            num2 = 8.0;
        }
        if item_h > 52 {
            num2 = 12.0;
        }
        if item_h > 64 {
            num2 = 14.0;
        }
        (x, pos_y + num2)
    } else {
        let mut num3 = 6.0f32;
        if item_w > 32 {
            num3 = 14.0;
        }
        if item_w >= 48 {
            num3 = 18.0;
        }
        if item_w >= 52 {
            num3 = 24.0;
        }
        if item_w >= 64 {
            num3 = 28.0;
        }
        if item_w >= 92 {
            num3 = 38.0;
        }
        let x = center_x - (item_w as f32 * 0.5 - num3) * aim_dir as f32;
        let mut num3 = 10.0f32;
        if item_h > 32 {
            num3 = 10.0;
        }
        if item_h > 52 {
            num3 = 12.0;
        }
        if item_h > 64 {
            num3 = 14.0;
        }
        (x, pos_y + num3)
    }
}

/// `NPC.TweakSwingStats` (`NPC.cs:66587-66609`) — the phase-1/3 rect
/// inflation (the middle phase leaves the rect alone).
fn tweak_swing_stats(rect: &mut (f32, f32, f32, f32), swing_max: i32, swing_current: i32, aim_dir: i32) {
    let cur = swing_current as f64;
    let max = swing_max as f64;
    if cur < max * 0.333 {
        if aim_dir == -1 {
            rect.0 -= (rect.2 as f64 * 1.4 - rect.2 as f64) as i32 as f32;
        }
        rect.2 = (rect.2 as f64 * 1.4) as i32 as f32;
        rect.1 += (rect.3 as f64 * 0.5) as i32 as f32;
        rect.3 = (rect.3 as f64 * 1.1) as i32 as f32;
    } else if !(cur < max * 0.666) {
        if aim_dir == 1 {
            rect.0 -= (rect.2 as f64 * 1.2) as i32 as f32;
        }
        rect.2 *= 2.0;
        rect.1 -= (rect.3 as f64 * 1.4 - rect.3 as f64) as i32 as f32;
        rect.3 = (rect.3 as f64 * 1.4) as i32 as f32;
    }
}

// ---------------------------------------------------------------------
// Witness tests — WRITTEN NOT RUN (code-only mode, no compiler invoked)
// ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_world_helpers::ground;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

    fn guide(x: f32, y: f32, home: Option<(i32, i32)>) -> Npc {
        let mut n = Npc::new(NpcKind::Guide, x, y, 1, 0.0);
        n.home = home;
        n
    }

    fn env<'a>(world: &'a World, players: &'a [TownPlayer], npcs: &'a [(usize, TownNpcView)]) -> TownEnv<'a> {
        TownEnv {
            world,
            day_time: true,
            raining: false,
            eclipse: false,
            slime_rain: false,
            storming: false,
            net_mode: 0,
            players,
            npcs,
            dungeon_x: 100,
            dungeon_y: 100,
            combat_book: false,
            combat_book_two: false,
            party_up: false,
            skyblock_low_tiles: false,
            infected_seed: false,
            only_shimmer_ocean: false,
            x_mas: false,
            town_damage_multiplier: 1.0,
            downed_slime_king: false,
            downed_boss1: false,
            downed_deerclops: false,
            downed_boss2: false,
            downed_boss3: false,
            downed_queen_bee: false,
            hard_mode: false,
            downed_queen_slime: false,
            downed_mech1: false,
            downed_mech2: false,
            downed_mech3: false,
            downed_plant: false,
            downed_empress: false,
            downed_fishron: false,
            downed_golem: false,
            downed_cultist: false,
            expert_mode: false,
            master_mode: false,
            homeless: false,
            breath: 200,
            shimmering: false,
            time_for_visual_effects: 0.0,
        }
    }

    fn stats() -> TownKindStats {
        TownKindStats { def_life_max: 250, def_defense: 15 }
    }

    /// A live, non-talking player slot at (x, y).
    fn player(x: f32, y: f32) -> TownPlayer {
        TownPlayer {
            candidate: TargetCandidate {
                active: true,
                dead: false,
                ghost: false,
                x,
                y,
                width: 20,
                height: 42,
                aggro: 0,
                item_animation: 0,
                npc_type_no_aggro: false,
                tank_pet: None,
            },
            talk_npc: -1,
            stinky: false,
            sitting: false,
            can_be_talked_to: true,
        }
    }

    /// The recording door fake: every attempt succeeds on demand.
    struct FakeDoors {
        attempts: Vec<(i32, i32, i32, bool)>, // (x, y, dir, tall)
        succeed: bool,
    }
    impl TownDoorOps for FakeDoors {
        fn open_door(&mut self, x: i32, y: i32, direction: i32) -> bool {
            self.attempts.push((x, y, direction, false));
            self.succeed
        }
        fn close_door(&mut self, x: i32, y: i32) -> bool {
            self.attempts.push((x, y, 0, false));
            self.succeed
        }
        fn shift_tall_gate(&mut self, x: i32, y: i32, _closing: bool) -> bool {
            self.attempts.push((x, y, 0, true));
            self.succeed
        }
    }

    /// `NPC.cs:52765-52784` — the resting-spot test: exact tile by day,
    /// the ±7 window only for a sitting NPC at night, the wet sea
    /// critters never rest.
    #[test]
    fn s1_d_3_town_the_good_resting_spot_gates() {
        assert!(is_in_a_good_resting_spot(22, 0.0, false, true, 60, 100, 60, 100));
        assert!(!is_in_a_good_resting_spot(22, 0.0, false, true, 61, 100, 60, 100));
        // Night + sitting: the ±7 window.
        assert!(is_in_a_good_resting_spot(22, 5.0, false, false, 67, 107, 60, 100));
        assert!(!is_in_a_good_resting_spot(22, 5.0, false, false, 68, 100, 60, 100));
        // Night + walking: exact only.
        assert!(!is_in_a_good_resting_spot(22, 1.0, false, false, 61, 100, 60, 100));
        // The wet sea critter (361) never rests wet.
        assert!(!is_in_a_good_resting_spot(361, 0.0, true, true, 60, 100, 60, 100));
    }

    /// `NPC.cs:52803-52806` — the floor drop: the home tile in mid-air
    /// falls to the first solid row (the ground() helper's row 120).
    #[test]
    fn s1_d_3_town_the_resting_spot_drops_to_the_floor() {
        let w = ground();
        let n = guide(60.0 * 16.0, 118.0 * 16.0, Some((60, 100)));
        let (fx, fy) = find_good_resting_spot(&w, &n, &[], 60, 118, true);
        assert_eq!((fx, fy), (60, 120), "the drop lands on the first solid row");
        // Homeless: the (-1,-1) sentinel.
        let homeless = guide(60.0 * 16.0, 118.0 * 16.0, None);
        assert_eq!(find_good_resting_spot(&w, &homeless, &[], 60, 118, true), (-1, -1));
    }

    /// `NPC.cs:52895` — the sit draws exactly ONE Next(10800); the
    /// stream position after the sit equals a single next_max(10800).
    #[test]
    fn s1_d_3_town_the_sit_draws_one_next_10800() {
        let mut w = ground();
        // A chair (15) at (60, 119), frame 0 — the tile above the floor.
        let i = w.index(60, 119);
        w.tiles[i] = Tile { kind: 15, frame_x: 0, frame_y: 0, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
        let mut n = guide(60.0 * 16.0, 118.0 * 16.0, Some((60, 120)));
        let mut rng = UnifiedRandom::new(42);
        assert!(try_forcing_sitting(&w, &mut n, &[], 60, 120, &mut rng));
        assert_eq!(n.ai[0], 5.0);
        assert_eq!(n.facing, -1, "frameX == 0 faces left");
        // The witness: a fresh stream's single draw matches ai[1]-900.
        let mut witness = UnifiedRandom::new(42);
        assert_eq!(n.ai[1], 900.0 + witness.next_max(10800) as f32);
        // The Bottom snap: x = 60*16+8+2*(-1) - 9 = 957, y = 120*16-40.
        assert_eq!(n.x, 60.0 * 16.0 + 8.0 - 2.0 - 9.0);
        assert_eq!(n.y, 120.0 * 16.0 - 40.0);
        assert_eq!(n.vx, 0.0);
        assert_eq!(n.vy, 0.0);
    }

    /// `NPC.cs:52878` — the forbidden chair frames 1080-1098 refuse the
    /// forced sit (and draw NOTHING).
    #[test]
    fn s1_d_3_town_the_forbidden_chair_frames_refuse() {
        let mut w = ground();
        let i = w.index(60, 119);
        w.tiles[i] = Tile { kind: 15, frame_x: 0, frame_y: 1080, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
        let mut n = guide(60.0 * 16.0, 118.0 * 16.0, Some((60, 120)));
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        assert!(!try_forcing_sitting(&w, &mut n, &[], 60, 120, &mut rng));
        // Zero draws: the streams stay aligned.
        assert_eq!(rng.next_max(1000), witness.next_max(1000));
    }

    /// `NPC.cs:55643-55671` — the teleport tries 0, −1, +1 in order and
    /// snaps the position 0.1 px above the floor; all-blocked returns
    /// false (the QuickFindHome request).
    #[test]
    fn s1_d_3_town_the_teleport_tries_three_columns() {
        let mut w = ground();
        // Block the home column's body space (59..61, 117..119).
        for x in 59..=61 {
            for y in 117..=119 {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        let mut n = guide(10.0 * 16.0, 10.0 * 16.0, Some((60, 120)));
        let mut rng = UnifiedRandom::new(1);
        // The −1 column (59) is blocked too; +1 (61) blocked — all fail.
        assert!(!teleport_to_home(&w, &mut n, &[], 60, 120, &mut rng));
        // Clear the +1 column's body: the third try takes it.
        for x in 60..=62 {
            for y in 117..=119 {
                let i = w.index(x, y);
                w.tiles[i] = Tile::default();
            }
        }
        // Re-block only the home and −1 columns' OWN tiles (58/59):
        // the +1 try probes (60..62), which must stay clear.
        for x in 58..=59 {
            for y in 117..=119 {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        let mut n2 = guide(10.0 * 16.0, 10.0 * 16.0, Some((60, 120)));
        assert!(teleport_to_home(&w, &mut n2, &[], 60, 120, &mut rng));
        assert_eq!(n2.x, 61.0 * 16.0 + 8.0 - 9.0, "the +1 column");
        assert_eq!(n2.y, 120.0 * 16.0 - 40.0 - 0.1);
    }

    /// `NPC.cs:55673-55743` — the walk prediction: a solid floor under
    /// the probe clears avoidFalling; a 6-tile empty column keeps it;
    /// the home-band exemption clears it for the home-bound walk.
    #[test]
    fn s1_d_3_town_the_walk_prediction_probes_the_column() {
        let w = ground();
        let n = guide(60.0 * 16.0, 118.0 * 16.0 - 40.0 + 40.0, Some((60, 120)));
        // Probe at (61, 119): the column hits the row-120 ground at j=1.
        let (_, avoid) = get_walk_prediction(&w, &n, &[], 60, 60, false, false, 61, 119);
        assert!(!avoid, "the solid floor clears the fall guard");
        // A column over the void (x 61, y 10): six empty tiles.
        let (_, avoid2) = get_walk_prediction(&w, &n, &[], 60, 60, false, false, 61, 10);
        assert!(avoid2, "the empty column keeps the fall guard");
        // The home-band exemption: outside ±35 and facing home.
        let mut far = guide(10.0 * 16.0, 118.0 * 16.0, Some((60, 120)));
        far.facing = 1; // home is to the right
        let (_, avoid3) = get_walk_prediction(&w, &far, &[], 10, 60, false, false, 11, 10);
        assert!(!avoid3, "the home-bound walk may fall");
    }

    /// `NPC.cs:53782` — the idle wander turn draws Next(80) inside the
    /// ±25 band with localAI[3] == 0. The full-tick witness: the idle
    /// arm's Next(80), then the social ladder's gates in else-if order
    /// (Next(300) talk, Next(1800) gossip, Next(1200) party — the Guide
    /// is not 208 and no party is up, so the gate's BODY is skipped but
    /// the DRAW happens — Next(600) bartender, Next(1800) small-talk).
    /// The attack-chance ladder draws NOTHING: the Guide's AttackType
    /// is 1, so the type-0 else-if chain short-circuits before its
    /// Next(num118), and the type-2/3 `if`s fail on AttackType.
    #[test]
    fn s1_d_3_town_the_idle_wander_draws_next_80_in_band() {
        let w = ground();
        let players = vec![player(60.0 * 16.0, 120.0 * 16.0 - 42.0)];
        let npcs: Vec<(usize, TownNpcView)> = Vec::new();
        let e = env(&w, &players, &npcs);
        let mut n = guide(60.0 * 16.0, 120.0 * 16.0 - 40.0, Some((60, 120)));
        n.ai[1] = 500.0; // the idle timer never expires this tick
        n.local_ai[3] = 0.0;
        let mut door = TownDoorLatch::default();
        let mut doors = FakeDoors { attempts: Vec::new(), succeed: false };
        let mut out = TownOutputs::default();
        // A seed where every reached gate draws NON-zero (the bodies
        // would add draws); the Next(80) wander result is free. The
        // Guide's idle tick reaches EIGHT social gates: talk 300,
        // gossip 1800, party 1200, bartender 600, small-talk 1800,
        // tax 600, player-talk 1200 (the sit/interact gates need
        // ai[0] == 1 and are skipped).
        let mut seed = 0;
        loop {
            let mut r = UnifiedRandom::new(seed);
            let _ = r.next_max(80);
            if r.next_max(300) != 0
                && r.next_max(1800) != 0
                && r.next_max(1200) != 0
                && r.next_max(600) != 0
                && r.next_max(1800) != 0
                && r.next_max(600) != 0
                && r.next_max(1200) != 0
            {
                break;
            }
            seed += 1;
        }
        let mut rng = UnifiedRandom::new(seed);
        let mut witness = UnifiedRandom::new(seed);
        town_npc_ai(&mut n, &e, stats(), &mut door, &mut doors, 0, &mut rng, &mut out);
        // The full draw sequence of this tick, in order.
        let _ = witness.next_max(80); // the wander gate
        let _ = witness.next_max(300); // the talk gate
        let _ = witness.next_max(1800); // the gossip gate
        let _ = witness.next_max(1200); // the party gate (drawn, body skipped)
        let _ = witness.next_max(600); // the bartender gate
        let _ = witness.next_max(1800); // the small-talk gate
        let _ = witness.next_max(600); // the tax gate (drawn, type gate fails)
        let _ = witness.next_max(1200); // the player-talk gate
        assert_eq!(rng.next_max(1_000_000), witness.next_max(1_000_000), "the full-tick stream matches");
    }

    /// `NPC.cs:53688` — the night go-home walk draws Next(200) and
    /// faces the home; `:53795` the walk's good-rest return draws
    /// Next(200) and sets localAI[3] = 60. The player stands ON the
    /// resting spot so the teleport-home block (`:53423`) refuses.
    #[test]
    fn s1_d_3_town_the_night_schedule_walks_home_then_rests() {
        let w = ground();
        let players = vec![player(60.0 * 16.0, 120.0 * 16.0 - 42.0)];
        let npcs: Vec<(usize, TownNpcView)> = Vec::new();
        let mut e = env(&w, &players, &npcs);
        e.day_time = false; // night → flag = true
        // Away from home: the idle arm starts the walk.
        let mut n = guide(40.0 * 16.0, 120.0 * 16.0 - 40.0, Some((60, 120)));
        n.local_ai[3] = 0.0;
        let mut door = TownDoorLatch::default();
        let mut doors = FakeDoors { attempts: Vec::new(), succeed: false };
        let mut out = TownOutputs::default();
        let mut rng = UnifiedRandom::new(5);
        town_npc_ai(&mut n, &e, stats(), &mut door, &mut doors, 0, &mut rng, &mut out);
        assert_eq!(n.ai[0], 1.0, "the night idle starts the go-home walk");
        assert_eq!(n.facing, 1, "home is to the right");
        let mut witness = UnifiedRandom::new(5);
        assert_eq!(n.ai[1], 200.0 + witness.next_max(200) as f32);
        // At home: the walk arm returns to idle with localAI[3] = 60.
        // wet = true silences the social ladder (flag31 needs !wet), so
        // the tick's ONLY draw is the good-rest Next(200).
        let mut n2 = guide(60.0 * 16.0, 120.0 * 16.0 - 40.0, Some((60, 120)));
        n2.ai[0] = 1.0;
        n2.ai[1] = 100.0;
        n2.wet = true;
        let mut rng2 = UnifiedRandom::new(6);
        town_npc_ai(&mut n2, &e, stats(), &mut door, &mut doors, 0, &mut rng2, &mut out);
        assert_eq!(n2.ai[0], 0.0);
        assert_eq!(n2.local_ai[3], 60.0);
        let mut witness2 = UnifiedRandom::new(6);
        assert_eq!(n2.ai[1], 200.0 + witness2.next_max(200) as f32);
    }

    /// W4-c10 sweep (the W4-b3 co-lane seam) — `NPC.cs:53180-53184`:
    /// the waterfowl walker falling fast (`vy > 4 || vy < -4`) or wet
    /// emits the `Transform(type + 1)` request and the arm RETURNS
    /// (zero draws, no state write — the caller runs the Transform
    /// core). A dry slow walker keeps walking; the client (netMode 1)
    /// never transforms; the swimmers 363/365/603/609 are not in the
    /// gate's type set.
    #[test]
    fn w4_c10_the_waterfowl_takeoff_gate_emits_the_transform() {
        let w = ground();
        let players = vec![player(60.0 * 16.0, 120.0 * 16.0 - 42.0)];
        let npcs: Vec<(usize, TownNpcView)> = Vec::new();
        let e = env(&w, &players, &npcs);
        let walkers = [
            (NpcKind::Duck, 362),
            (NpcKind::DuckWhite, 364),
            (NpcKind::Seagull, 602),
            (NpcKind::Grebe, 608),
        ];
        for (kind, ty) in walkers {
            // The wet takeoff: the request fires, the stream untouched.
            let mut n = Npc::new(kind, 40.0 * 16.0, 120.0 * 16.0 - 26.0, 1, 0.0);
            n.wet = true;
            let mut door = TownDoorLatch::default();
            let mut doors = FakeDoors { attempts: Vec::new(), succeed: false };
            let mut out = TownOutputs::default();
            let mut rng = UnifiedRandom::new(11);
            let before = rng.clone();
            town_npc_ai(&mut n, &e, stats(), &mut door, &mut doors, 0, &mut rng, &mut out);
            assert_eq!(out.waterfowl_takeoff, Some(ty + 1), "{kind:?} wet takeoff");
            assert_eq!(rng, before, "{kind:?}: the gate draws nothing");
            assert_eq!(n.ai, [0.0; 4], "{kind:?}: the arm returned pre-state");
            // The fast fall (vy > 4) and the fast rise (vy < -4).
            for vy in [4.5f32, -4.5] {
                let mut n2 = Npc::new(kind, 40.0 * 16.0, 120.0 * 16.0 - 26.0, 1, 0.0);
                n2.vy = vy;
                let mut out2 = TownOutputs::default();
                let mut rng2 = UnifiedRandom::new(12);
                let before2 = rng2.clone();
                town_npc_ai(&mut n2, &e, stats(), &mut door, &mut doors, 0, &mut rng2, &mut out2);
                assert_eq!(out2.waterfowl_takeoff, Some(ty + 1), "{kind:?} vy {vy}");
                assert_eq!(rng2, before2);
            }
            // The dry slow walker: NO request (the walk arm runs).
            let mut n3 = Npc::new(kind, 40.0 * 16.0, 120.0 * 16.0 - 26.0, 1, 0.0);
            n3.vy = 1.0;
            let mut out3 = TownOutputs::default();
            town_npc_ai(&mut n3, &e, stats(), &mut door, &mut doors, 0, &mut UnifiedRandom::new(13), &mut out3);
            assert_eq!(out3.waterfowl_takeoff, None, "{kind:?} dry slow keeps walking");
            // The client: the netMode gate refuses even when wet.
            let mut e_client = env(&w, &players, &npcs);
            e_client.net_mode = 1;
            let mut n4 = Npc::new(kind, 40.0 * 16.0, 120.0 * 16.0 - 26.0, 1, 0.0);
            n4.wet = true;
            let mut out4 = TownOutputs::default();
            town_npc_ai(&mut n4, &e_client, stats(), &mut door, &mut doors, 0, &mut UnifiedRandom::new(14), &mut out4);
            assert_eq!(out4.waterfowl_takeoff, None, "{kind:?} the client never transforms");
        }
        // The strict thresholds: vy exactly ±4 does NOT fire (the C#'s
        // `> 4f || < -4f`), the wet flag alone does.
        let mut n5 = Npc::new(NpcKind::Duck, 40.0 * 16.0, 120.0 * 16.0 - 26.0, 1, 0.0);
        n5.vy = 4.0;
        let mut door = TownDoorLatch::default();
        let mut doors = FakeDoors { attempts: Vec::new(), succeed: false };
        let mut out5 = TownOutputs::default();
        town_npc_ai(&mut n5, &e, stats(), &mut door, &mut doors, 0, &mut UnifiedRandom::new(15), &mut out5);
        assert_eq!(out5.waterfowl_takeoff, None, "vy == 4 is not > 4");
    }

    /// `NPC.cs:53946-53949` — the negative-branch clamp quirk: the C#
    /// tests `velocity.X > num21` (positive) after subtracting, so the
    /// left walk overshoots −num21 by one accel step and is NOT clamped.
    #[test]
    fn s1_d_3_town_the_left_walk_keeps_the_clamp_quirk() {
        let w = ground();
        let players = vec![player(60.0 * 16.0, 120.0 * 16.0 - 42.0)];
        let npcs: Vec<(usize, TownNpcView)> = Vec::new();
        let e = env(&w, &players, &npcs);
        let mut n = guide(60.0 * 16.0, 120.0 * 16.0 - 40.0, Some((60, 120)));
        n.ai[0] = 1.0;
        n.ai[1] = 500.0;
        n.facing = -1;
        n.vx = -0.99; // one accel step from the −1 cap
        let mut door = TownDoorLatch::default();
        let mut doors = FakeDoors { attempts: Vec::new(), succeed: false };
        let mut out = TownOutputs::default();
        let mut rng = UnifiedRandom::new(3);
        town_npc_ai(&mut n, &e, stats(), &mut door, &mut doors, 0, &mut rng, &mut out);
        // −0.99 − 0.07 = −1.06, NOT clamped back to −1 (the quirk).
        assert!(n.vx < -1.0, "the left accel overshoots the cap: {}", n.vx);
    }

    /// `NPC.cs:54043-54078` — the door gate draws Next(10) whenever the
    /// door tile matches (the `|` is non-short-circuiting: the draw
    /// happens even in the rain). The three open attempts run in C#
    /// order (own direction, reverse, tall gate); the first success
    /// arms the latch + ai[1] += 80, the all-fail flips the direction.
    #[test]
    fn s1_d_3_town_the_door_open_attempts_and_the_next_10() {
        let mut w = ground();
        // A closed door (10) at (61, 117..119): the probe tile5 row.
        for y in 117..=119 {
            let i = w.index(61, y);
            w.tiles[i] = Tile { kind: 10, frame_x: 0, frame_y: ((y - 117) * 18) as i16, ..Default::default() };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        let players = vec![player(60.0 * 16.0, 120.0 * 16.0 - 42.0)];
        let npcs: Vec<(usize, TownNpcView)> = Vec::new();
        let e = env(&w, &players, &npcs);
        // The Guide walks right at the door: num23 = (x+9+15)/16 = 61.
        let mut n = guide(61.0 * 16.0 - 9.0 - 15.0 + 1.0, 120.0 * 16.0 - 40.0, Some((60, 120)));
        n.ai[0] = 1.0;
        n.ai[1] = 500.0;
        n.facing = 1;
        n.vx = 0.5;
        let mut door = TownDoorLatch::default();
        let mut doors = FakeDoors { attempts: Vec::new(), succeed: false };
        let mut out = TownOutputs::default();
        // A seed whose Next(10) == 0 (the door gate fires in daytime).
        let mut seed = 0;
        loop {
            let mut r = UnifiedRandom::new(seed);
            if r.next_max(10) == 0 {
                break;
            }
            seed += 1;
        }
        let mut rng = UnifiedRandom::new(seed);
        town_npc_ai(&mut n, &e, stats(), &mut door, &mut doors, 0, &mut rng, &mut out);
        assert_eq!(doors.attempts.len(), 3, "the three open attempts in order");
        assert_eq!(doors.attempts[0], (61, 117, 1, false), "own direction first");
        assert_eq!(doors.attempts[1], (61, 117, -1, false), "reverse second");
        assert_eq!(doors.attempts[2], (61, 117, 0, true), "tall gate third");
        // All failed: the direction flips, the latch stays clear.
        assert_eq!(n.facing, -1, "the all-fail flip");
        assert!(!door.close_door);
        // The success path: the latch arms and ai[1] += 80.
        let mut n2 = guide(61.0 * 16.0 - 9.0 - 15.0 + 1.0, 120.0 * 16.0 - 40.0, Some((60, 120)));
        n2.ai[0] = 1.0;
        n2.ai[1] = 500.0;
        n2.facing = 1;
        n2.vx = 0.5;
        let mut door2 = TownDoorLatch::default();
        let mut doors2 = FakeDoors { attempts: Vec::new(), succeed: true };
        let mut rng2 = UnifiedRandom::new(seed);
        town_npc_ai(&mut n2, &e, stats(), &mut door2, &mut doors2, 0, &mut rng2, &mut out);
        assert_eq!(doors2.attempts.len(), 1, "the first success stops the chain");
        assert!(door2.close_door);
        assert_eq!((door2.door_x, door2.door_y), (61, 117));
        assert_eq!(n2.ai[1], 579.0, "500 − 1 (the walk decrement) + 80");
        assert_eq!(n2.facing, 1, "no flip on success");
    }

    /// `NPC.cs:55231-55256` — the talk latch: the Next(300) gate, the
    /// Next(2) + Next(1,3)|Next(1,4) duration, the partner's state 4
    /// write with the mirrored direction.
    #[test]
    fn s1_d_3_town_the_talk_latch_draws_and_writes_the_partner() {
        let w = ground();
        let players = vec![player(60.0 * 16.0, 120.0 * 16.0 - 42.0)];
        // A talkable Merchant 40 px to the right, idle, grounded.
        let partner = TownNpcView {
            net_id: 17,
            x: 60.0 * 16.0 + 40.0,
            y: 120.0 * 16.0 - 40.0,
            width: 18.0,
            height: 40.0,
            vx: 0.0,
            vy: 0.0,
            ai: [0.0, 0.0, 0.0, 0.0],
            wet: false,
            friendly: true,
            damage: 0,
            stinky: false,
            chaseable: true,
            dont_take_damage: false,
            immortal: false,
            no_tile_collide: false,
            life: 250,
            life_max: 250,
            town_ai7: true,
            can_be_talked_to: true,
            close_door: false,
            has_buff_165: false,
            immune_to_me: true,
        };
        let npcs = vec![(1usize, partner)];
        let e = env(&w, &players, &npcs);
        let mut n = guide(60.0 * 16.0, 120.0 * 16.0 - 40.0, Some((60, 120)));
        n.ai[1] = 500.0;
        let mut door = TownDoorLatch::default();
        let mut doors = FakeDoors { attempts: Vec::new(), succeed: false };
        let mut out = TownOutputs::default();
        // Find a seed whose Next(80) != 0 (the idle wander gate must
        // not add a draw) and whose Next(300) == 0 (the talk gate fires).
        let mut seed = 0;
        loop {
            let mut r = UnifiedRandom::new(seed);
            if r.next_max(80) != 0 && r.next_max(300) == 0 {
                break;
            }
            seed += 1;
        }
        let mut rng = UnifiedRandom::new(seed);
        town_npc_ai(&mut n, &e, stats(), &mut door, &mut doors, 0, &mut rng, &mut out);
        assert_eq!(n.ai[0], 3.0, "the talk state");
        assert_eq!(n.ai[2], 1.0, "the partner slot");
        assert_eq!(n.facing, 1, "the partner is to the right");
        assert_eq!(out.partner_writes.len(), 1);
        match out.partner_writes[0] {
            TownPartnerWrite::Talk { slot, state, direction, gossip, .. } => {
                assert_eq!(slot, 1);
                assert_eq!(state, 4.0);
                assert_eq!(direction, -1);
                assert!(!gossip);
            }
            _ => panic!("expected the talk write"),
        }
        // The duration witness: Next(80) (idle wander), Next(300) (the
        // talk gate), Next(2) then the branch draw.
        let mut witness = UnifiedRandom::new(seed);
        let _ = witness.next_max(80);
        let _ = witness.next_max(300);
        let dur = 420.0 * if witness.next_max(2) != 0 {
            witness.next_range(1, 3) as f32
        } else {
            witness.next_range(1, 4) as f32
        };
        assert_eq!(n.ai[1], dur);
    }

    /// `NPC.cs:53014-53157` — the stat ladder: the combat book adds
    /// 250 lifeMax / +8 defense / ×0.8 cooldown / +0.25 damage, the
    /// boss ladder's 0.985 multiplications compound in C# order, and
    /// tipsy truncates the defense ×1.1.
    #[test]
    fn s1_d_3_town_the_stat_ladder_compounds_in_order() {
        let w = ground();
        let players: Vec<TownPlayer> = Vec::new();
        let npcs: Vec<(usize, TownNpcView)> = Vec::new();
        let mut e = env(&w, &players, &npcs);
        e.combat_book = true;
        e.downed_slime_king = true;
        e.hard_mode = true;
        let mut n = guide(0.0, 0.0, None);
        let (num2, num3) = town_stat_bonuses(&mut n, stats(), &e, false, false);
        // num2 = 1 + 0.25 (book) + 0.05 (slime king) + 0.4 (hardmode).
        assert!((num2 - 1.7).abs() < 1e-5, "num2 {num2}");
        // num3 = 2 * 0.8 * 0.985 * 0.985.
        let expect3 = 2.0f32 * 0.8 * 0.985 * 0.985;
        assert!((num3 - expect3).abs() < 1e-5, "num3 {num3}");
        assert_eq!(n.max_hp, 500, "the book's +250");
        // defense = 15 + 8 (book) + 2 (slime king) + 12 (hardmode) = 37.
        assert_eq!(n.defense_override, Some(37));
        // Tipsy: 37 * 1.1 = 40.7 → 40.
        let mut n2 = guide(0.0, 0.0, None);
        let (_, _) = town_stat_bonuses(&mut n2, stats(), &e, false, true);
        assert_eq!(n2.defense_override, Some(40));
    }

    /// W4-g89e — `NPC.cs:53025-53031`: the vol-two book's block is the SAME
    /// +250 lifeMax / ×0.8 cooldown / +0.25 damage / +8 defense as vol one,
    /// applied AFTER it (both books: 2× each). The `:53032-53035` full-heal
    /// on growth (`&` non-short-circuit) fires when the NPC was at full
    /// life before the reset — the 5336 use arm's world flag
    /// (`Player.cs:45342`) feeds `env.combat_book_two`.
    #[test]
    fn w4_g89e_the_vol_two_book_stacks_the_same_bonus_and_heals_on_growth() {
        let w = ground();
        let players: Vec<TownPlayer> = Vec::new();
        let npcs: Vec<(usize, TownNpcView)> = Vec::new();
        let mut e = env(&w, &players, &npcs);
        e.combat_book_two = true;
        let mut n = guide(0.0, 0.0, None);
        n.hp = n.max_hp; // at full life — flag3 (`:53016`)
        let (num2, num3) = town_stat_bonuses(&mut n, stats(), &e, false, false);
        assert!((num2 - 1.25).abs() < 1e-5, "num2 = 1 + 0.25 (`:53029`)");
        assert!((num3 - 2.0f32 * 0.8).abs() < 1e-5, "num3 = 2 * 0.8 (`:53028`)");
        assert_eq!(n.max_hp, 500, "the book's +250 (`:53027`)");
        assert_eq!(n.defense_override, Some(23), "15 + 8 (`:53030`)");
        assert_eq!(n.hp, 500, "the growth full-heal (`:53032-53035`)");
        // Both books: the blocks stack in C# order (vol one then vol two).
        e.combat_book = true;
        let mut n2 = guide(0.0, 0.0, None);
        let (num2b, num3b) = town_stat_bonuses(&mut n2, stats(), &e, false, false);
        assert!((num2b - 1.5).abs() < 1e-5, "1 + 0.25 + 0.25");
        let expect3 = 2.0f32 * 0.8 * 0.8;
        assert!((num3b - expect3).abs() < 1e-5, "2 * 0.8 * 0.8");
        assert_eq!(n2.max_hp, 750, "250 + 250 + 250");
        assert_eq!(n2.defense_override, Some(31), "15 + 8 + 8");
        // A damaged NPC keeps its life on growth (flag3 false).
        let mut n3 = guide(0.0, 0.0, None);
        n3.hp = 10;
        let _ = town_stat_bonuses(&mut n3, stats(), &e, false, false);
        assert_eq!(n3.hp, 10, "no heal when not at full life");
    }

    /// W4-g90 lane 7 — Volume Two's row (`NPC.cs:53025-53031`) is the SAME
    /// +250/+8/×0.8/+0.25 as book one's, applied independently: both books
    /// stack to +500 lifeMax / +16 defense / ×0.64 cooldown / +0.5 damage.
    #[test]
    fn w4_g90_both_combat_books_stack_their_rows() {
        let w = ground();
        let players: Vec<TownPlayer> = Vec::new();
        let npcs: Vec<(usize, TownNpcView)> = Vec::new();
        let mut e = env(&w, &players, &npcs);
        e.combat_book = true;
        e.combat_book_two = true;
        let mut n = guide(0.0, 0.0, None);
        let (num2, num3) = town_stat_bonuses(&mut n, stats(), &e, false, false);
        assert!((num2 - 1.5).abs() < 1e-5, "1 + 0.25 + 0.25, num2 {num2}");
        assert!((num3 - 2.0f32 * 0.8 * 0.8).abs() < 1e-5, "num3 {num3}");
        assert_eq!(n.max_hp, 750, "250 + 250 + 250");
        assert_eq!(n.defense_override, Some(31), "15 + 8 + 8");
        // Book two ALONE (book one unused): the same single row.
        e.combat_book = false;
        let mut n2 = guide(0.0, 0.0, None);
        let (num2b, _) = town_stat_bonuses(&mut n2, stats(), &e, false, false);
        assert!((num2b - 1.25).abs() < 1e-5);
        assert_eq!(n2.max_hp, 500);
        assert_eq!(n2.defense_override, Some(23));
    }

    /// `NPC.cs:55750-55785` — the pet idle: the slime takes state 20
    /// with NO Next(20,20+num) draw, then Next(300) + Next(240).
    #[test]
    fn s1_d_3_town_the_pet_idle_slime_skips_the_state_draw() {
        // A town-slime-shaped NPC: the port has no TownSlime kind, so
        // the witness drives the pure fn with a Guide carrying the
        // slime net id through a view — the fn reads npc.kind.net_id(),
        // so instead we verify the DRAW COUNT on the common pet path
        // (num = 3): Next(chance) + Next(20,23) + Next(300).
        let mut n = guide(0.0, 0.0, None);
        n.vx = 0.0;
        let mut rng = UnifiedRandom::new(1);
        // Force the gate: find a seed with Next(120) == 0.
        let mut seed = 0;
        loop {
            let mut r = UnifiedRandom::new(seed);
            if r.next_max(120) == 0 {
                break;
            }
            seed += 1;
        }
        let mut rng = UnifiedRandom::new(seed);
        // The Guide is not a pet, but the fn itself has no pet gate —
        // the caller gates it; drive it directly (num = 3 path).
        assert!(pet_idle_animation(&mut n, 120, 0, &mut rng));
        assert!((20.0..23.0).contains(&n.ai[0]));
        let mut witness = UnifiedRandom::new(seed);
        let _ = witness.next_max(120);
        let state = witness.next_range(20, 23);
        assert_eq!(n.ai[0], state as f32);
        assert_eq!(n.ai[1], 200.0 + witness.next_max(300) as f32);
    }

    /// `NPC.cs:54504`/`:54519-54521` — the attack-10 cooldown draws
    /// Next(maxValue) TWICE (ai[1] then localAI[1]/[3]); the shot
    /// jitter draws two NextDouble even at range 0.
    #[test]
    fn s1_d_3_town_the_attack_cooldown_draws_twice() {
        let w = ground();
        let players = vec![player(60.0 * 16.0, 120.0 * 16.0 - 42.0)];
        let npcs: Vec<(usize, TownNpcView)> = Vec::new();
        let e = env(&w, &players, &npcs);
        // The Merchant (17): attack 10, cooldown 60 + Next(60).
        let mut n = Npc::new(NpcKind::Merchant, 60.0 * 16.0, 120.0 * 16.0 - 40.0, 1, 0.0);
        n.home = Some((60, 120));
        n.ai[0] = 10.0;
        n.ai[1] = 1.0; // expires this tick (after the decrement)
        n.local_ai[3] = 0.0; // not the fire tick (10)
        let mut door = TownDoorLatch::default();
        let mut doors = FakeDoors { attempts: Vec::new(), succeed: false };
        let mut out = TownOutputs::default();
        // A seed whose Next(60) cooldown draws are both non-zero (the
        // localAI[1] decrement then suppresses flag32, so the attack
        // ladder draws nothing) and whose SEVEN social gates all draw
        // non-zero (the player-talk gate at Next(1200) would otherwise
        // fire and overwrite ai[1]).
        let mut seed = 0;
        loop {
            let mut r = UnifiedRandom::new(seed);
            let a = r.next_max(60);
            let b = r.next_max(60);
            if a != 0
                && b != 0
                && r.next_max(300) != 0
                && r.next_max(1800) != 0
                && r.next_max(1200) != 0
                && r.next_max(600) != 0
                && r.next_max(1800) != 0
                && r.next_max(600) != 0
                && r.next_max(1200) != 0
            {
                break;
            }
            seed += 1;
        }
        let mut rng = UnifiedRandom::new(seed);
        town_npc_ai(&mut n, &e, stats(), &mut door, &mut doors, 0, &mut rng, &mut out);
        assert_eq!(n.ai[0], 0.0);
        let mut witness = UnifiedRandom::new(seed);
        let t1 = 60.0 + witness.next_max(60) as f32;
        let half = 30 + witness.next_max(60);
        assert_eq!(n.ai[1], t1);
        assert_eq!(n.local_ai[1], half as f32 - 1.0, "the social ladder's decrement");
        assert_eq!(n.local_ai[3], half as f32);
    }

    /// The citations the golden rule demands.
    #[test]
    fn s1_d_3_town_the_citations_are_present() {
        let src = include_str!("npc_ai_town.rs");
        assert!(src.contains("NPC.cs:52965-55561"));
        assert!(src.contains("NPC.cs:52795-52862"));
        assert!(src.contains("NPC.cs:55673-55743"));
        assert!(src.contains("NPC.cs:55643-55671"));
        assert!(src.contains("NPC.cs:52864-52902"));
        assert!(src.contains("NPC.cs:55750-55785"));
        assert!(src.contains("ID/NPCID.cs:10466"));
        assert!(src.contains("Collision.cs:3694-3697"));
    }

    // -----------------------------------------------------------------
    // W4-f47 — the state-25 shimmer emergence witnesses (WRITTEN NOT RUN)
    // -----------------------------------------------------------------

    /// `NPC.cs:53187-53213` — the ai[2] hold: the countdown decrements,
    /// at zero ai[1] arms 1, and the arm RETURNS (no ai[1] climb, no
    /// visuals). The wet flags clear every state-25 tick.
    #[test]
    fn w4_f47_state25_the_teleport_hold_counts_down_and_returns() {
        let w = ground();
        let players = vec![player(60.0 * 16.0, 120.0 * 16.0 - 42.0)];
        let npcs: Vec<(usize, TownNpcView)> = Vec::new();
        let e = env(&w, &players, &npcs);
        let mut n = guide(60.0 * 16.0, 120.0 * 16.0 - 40.0, Some((60, 120)));
        n.ai = [25.0, 0.0, 2.0, 0.0];
        n.wet = true;
        n.shimmer_wet = true;
        let mut rng = UnifiedRandom::new(7);
        let mut out = TownOutputs::default();
        town_shimmer_state25(&mut n, &e, &mut rng, &mut out);
        assert_eq!(n.ai[2], 1.0);
        assert_eq!(n.ai[1], 0.0, "the hold returns before the climb");
        assert!(!n.wet && !n.shimmer_wet, "the wet flags clear");
        assert!(n.dont_take_damage);
        assert!(out.shimmer_fx.is_empty(), "no visuals during the hold");
        // The hold's last tick arms ai[1] = 1.
        town_shimmer_state25(&mut n, &e, &mut rng, &mut out);
        assert_eq!(n.ai[2], 0.0);
        assert_eq!(n.ai[1], 1.0);
        // The stream never moved (the hold draws nothing).
        assert_eq!(rng, UnifiedRandom::new(7));
    }

    /// `NPC.cs:53215-53226` — the climb: below 30 no decay; at 30+ the
    /// dry transparency decays 1/60 and the velocity is (0, -4*t); a
    /// WET probe pins ai[1] at 30 (no decay).
    #[test]
    fn w4_f47_state25_the_climb_decays_dry_and_pins_wet() {
        let mut w = ground();
        // A shimmer pool at the Guide's feet (x 59..62, y 118..119).
        for x in 59..63 {
            for y in 118..120 {
                let i = w.index(x, y);
                w.tiles[i].liquid = 255;
                w.tiles[i].set_liquid_kind(tr_core::tile::LiquidKind::Shimmer);
            }
        }
        let players = vec![player(60.0 * 16.0, 120.0 * 16.0 - 42.0)];
        let npcs: Vec<(usize, TownNpcView)> = Vec::new();
        let e = env(&w, &players, &npcs);
        // Dry: stand ABOVE the pool (y 100 tiles), ai1 29 → 30.
        let mut n = guide(60.0 * 16.0, 100.0 * 16.0, Some((60, 120)));
        n.ai = [25.0, 29.0, 0.0, 0.0];
        n.shimmer_transparency = 0.89;
        let mut rng = UnifiedRandom::new(3);
        let mut out = TownOutputs::default();
        town_shimmer_state25(&mut n, &e, &mut rng, &mut out);
        assert_eq!(n.ai[1], 30.0);
        assert!((n.shimmer_transparency - (0.89 - 1.0 / 60.0)).abs() < 1e-6);
        assert_eq!(n.vx, 0.0);
        assert!((n.vy - (-4.0 * n.shimmer_transparency)).abs() < 1e-5);
        // Wet: stand IN the pool — ai1 pins at 30, no decay.
        let mut n2 = guide(60.0 * 16.0, 118.0 * 16.0, Some((60, 120)));
        n2.ai = [25.0, 40.0, 0.0, 0.0];
        n2.shimmer_transparency = 0.5;
        let mut out2 = TownOutputs::default();
        town_shimmer_state25(&mut n2, &e, &mut rng, &mut out2);
        assert_eq!(n2.ai[1], 30.0, "the wet probe pins the climb");
        assert_eq!(n2.shimmer_transparency, 0.5, "no decay while wet");
    }

    /// `NPC.cs:53251-53275` — the completion: ai1 ≥ 75 && t ≤ 0 zeroes
    /// the ai/localAI, kicks (0,-4), flips the variation and credits
    /// progression 43 when the flip lands on 1.
    #[test]
    fn w4_f47_state25_the_completion_flips_the_variant() {
        let w = ground();
        let players = vec![player(60.0 * 16.0, 120.0 * 16.0 - 42.0)];
        let npcs: Vec<(usize, TownNpcView)> = Vec::new();
        let e = env(&w, &players, &npcs);
        let mut n = guide(60.0 * 16.0, 100.0 * 16.0, Some((60, 120)));
        n.ai = [25.0, 74.0, 0.0, 0.0];
        n.shimmer_transparency = 1.0 / 60.0; // one dry decay → ≤ 0
        n.variation_index = 0;
        n.local_ai = [1.0, 2.0, 3.0, 4.0];
        let mut rng = UnifiedRandom::new(11);
        let mut out = TownOutputs::default();
        town_shimmer_state25(&mut n, &e, &mut rng, &mut out);
        assert_eq!(n.ai, [0.0; 4], "the completion zeroes the ai");
        assert_eq!(n.local_ai, [0.0; 4]);
        assert_eq!(n.vx, 0.0);
        assert_eq!(n.vy, -4.0);
        assert_eq!(n.variation_index, 1);
        assert!(out.shimmer_town_progression, "variation 1 credits 43");
        assert!(n.net_update);
        // The reverse flip: variation 1 → 0, no progression.
        let mut n2 = guide(60.0 * 16.0, 100.0 * 16.0, Some((60, 120)));
        n2.ai = [25.0, 80.0, 0.0, 0.0];
        n2.shimmer_transparency = 0.0;
        n2.variation_index = 1;
        let mut out2 = TownOutputs::default();
        town_shimmer_state25(&mut n2, &e, &mut rng, &mut out2);
        assert_eq!(n2.variation_index, 0);
        assert!(!out2.shimmer_town_progression);
    // ---- W4-g90: the App town-tick feeder (WRITTEN NOT RUN, code-only) ----

    /// The feeder's flag set carries BOTH combat books off the live
    /// World flags (`NPC.cs:53018-53031` — book one AND volume two each
    /// add +250 lifeMax / ×0.8 num3 / +0.25 num2 / +8 defense), plus the
    /// boss ladder's world reads. This is the piece the b3 phase-1 env
    /// folded false (`combat_book_two: false, // volume two unported`)
    /// that the g90 world flag now feeds.
    #[test]
    fn w4_g90_the_env_flags_feed_both_books_and_the_boss_ladder() {
        let mut w = ground();
        w.combat_book_was_used = true;
        w.combat_book_volume_two_was_used = true;
        w.downed_slime_king = true;
        w.hard_mode = true;
        let flags = TownEnvFlags::from_world(&w, false);
        assert!(flags.combat_book, "World.combat_book_was_used");
        assert!(flags.combat_book_two, "World.combat_book_volume_two_was_used");
        assert!(flags.downed_slime_king);
        assert!(flags.hard_mode);
        let e = flags.env(&w, &[], &[], false);
        let mut n = guide(0.0, 0.0, None);
        let (num2, num3) = town_stat_bonuses(&mut n, stats(), &e, false, false);
        // num2 = 1 + 0.25 + 0.25 (both books) + 0.05 (SK) + 0.4 (hard).
        assert!((num2 - 1.95).abs() < 1e-5, "num2 {num2}");
        // num3 = 2 × 0.8 × 0.8 × 0.985 (SK) × 0.985 (hard).
        let expect3 = 2.0f32 * 0.8 * 0.8 * 0.985 * 0.985;
        assert!((num3 - expect3).abs() < 1e-5, "num3 {num3}");
        assert_eq!(n.max_hp, 750, "250 + 250 + 250");
        // defense = 15 + 8 + 8 + 2 (SK) + 12 (hard) = 45.
        assert_eq!(n.defense_override, Some(45));
        assert_eq!(n.hp, 750, "the full-heal-on-growth (`:53032-53035`)");
    }

    /// The feeder loop's exact shape (the reset-then-read base + the
    /// per-NPC env lend) is IDEMPOTENT: two consecutive passes write the
    /// same state — a stale `defense_override` never compounds into the
    /// next tick's base (the `npc.rs` skeletron reset convention).
    #[test]
    fn w4_g90_the_feeder_pass_is_idempotent_across_ticks() {
        let mut w = ground();
        w.combat_book_was_used = true;
        w.downed_eoc = true;
        let flags = TownEnvFlags::from_world(&w, false);
        let mut n = guide(0.0, 0.0, Some((10, 10)));
        let mut pass = |n: &mut Npc| {
            n.defense_override = None;
            let stats = TownKindStats {
                def_life_max: n.kind.max_hp(),
                def_defense: n.defense(),
            };
            let env = flags.env(&w, &[], &[], n.home.is_none());
            let dryad = n.buffs.find_buff_index(165).is_some();
            let tipsy = n.buffs.find_buff_index(25).is_some();
            town_stat_bonuses(n, stats, &env, dryad, tipsy)
        };
        let first = pass(&mut n);
        let (hp1, def1) = (n.hp, n.defense_override);
        let second = pass(&mut n);
        assert_eq!(first.0, second.0);
        assert_eq!(first.1, second.1);
        assert_eq!(n.hp, hp1);
        assert_eq!(n.defense_override, def1);
        assert_eq!(n.defense_override, Some(15 + 8 + 2), "book + EoC");
        // The tipsy buff feeds through the SAME seam (`NPC.cs:53151`).
        n.buffs.add_buff(25, 600);
        let (d2, d3) = pass(&mut n);
        assert_eq!(n.defense_override, Some(((15 + 8 + 2) as f64 * 1.1) as i32));
        assert!((d2 - first.0 * 1.1).abs() < 1e-5);
        assert!((d3 - first.1 * 0.9).abs() < 1e-5);
    }

    /// W4-g90 RETIRE — the b3-dedup receipt witness 1/2 (WRITTEN NOT
    /// RUN): the feeder's `from_world` env and b3's phase-1 INLINE env
    /// (`npc.rs::ai_007_town_family`, tip `96a5d88f0`) agree on every
    /// scalar EXCEPT `combat_book_two` — b3 folds it false ("volume two
    /// unported", stale: the g90 world flag is the carrier). The census
    /// pins the field-for-field claim; witness 2/2 pins the ladder delta.
    #[test]
    fn w4_g90_the_b3_inline_env_matches_from_world_except_book_two() {
        let mut w = ground();
        w.combat_book_was_used = true;
        w.combat_book_volume_two_was_used = true;
        w.downed_eoc = true;
        let b3 = TownEnv {
            world: &w,
            day_time: w.day_time,
            raining: w.raining,
            eclipse: w.eclipse,
            slime_rain: w.slime_rain,
            storming: false,
            net_mode: 0,
            expert_mode: false,
            master_mode: false,
            players: &[],
            npcs: &[],
            dungeon_x: 0,
            dungeon_y: 0,
            combat_book: w.combat_book_was_used,
            combat_book_two: false, // the b3 fold — the ONE divergence
            party_up: false,
            skyblock_low_tiles: false,
            infected_seed: w.seeds.contains(tr_core::seeds::SeedFlags::ZENITH),
            only_shimmer_ocean: false,
            x_mas: false,
            town_damage_multiplier: 1.0,
            downed_slime_king: w.downed_slime_king,
            downed_boss1: w.downed_eoc,
            downed_deerclops: w.downed_deerclops,
            downed_boss2: w.downed_eow,
            downed_boss3: w.downed_skeletron,
            downed_queen_bee: w.downed_queen_bee,
            hard_mode: w.hard_mode,
            downed_queen_slime: w.downed_queen_slime,
            downed_mech1: w.downed_mech_boss1,
            downed_mech2: w.downed_mech_boss2,
            downed_mech3: w.downed_mech_boss3,
            downed_plant: w.downed_plant_boss,
            downed_empress: false,
            downed_fishron: w.downed_fishron,
            downed_golem: w.downed_golem,
            downed_cultist: w.downed_ancient_cultist,
            homeless: false,
            breath: 200,
            shimmering: false,
        };
        let feeder = TownEnvFlags::from_world(&w, false).env(&w, &[], &[], false);
        // The scalar census: every field EXCEPT combat_book_two agrees.
        assert_eq!(feeder.day_time, b3.day_time);
        assert_eq!(feeder.raining, b3.raining);
        assert_eq!(feeder.eclipse, b3.eclipse);
        assert_eq!(feeder.slime_rain, b3.slime_rain);
        assert_eq!(feeder.storming, b3.storming);
        assert_eq!(feeder.net_mode, b3.net_mode);
        assert_eq!(feeder.expert_mode, b3.expert_mode);
        assert_eq!(feeder.master_mode, b3.master_mode);
        assert_eq!(feeder.dungeon_x, b3.dungeon_x);
        assert_eq!(feeder.dungeon_y, b3.dungeon_y);
        assert_eq!(feeder.combat_book, b3.combat_book);
        assert!(feeder.combat_book_two && !b3.combat_book_two, "the ONE fold");
        assert_eq!(feeder.party_up, b3.party_up);
        assert_eq!(feeder.skyblock_low_tiles, b3.skyblock_low_tiles);
        assert_eq!(feeder.infected_seed, b3.infected_seed);
        assert_eq!(feeder.only_shimmer_ocean, b3.only_shimmer_ocean);
        assert_eq!(feeder.x_mas, b3.x_mas);
        assert_eq!(feeder.town_damage_multiplier, b3.town_damage_multiplier);
        assert_eq!(feeder.downed_slime_king, b3.downed_slime_king);
        assert_eq!(feeder.downed_boss1, b3.downed_boss1);
        assert_eq!(feeder.downed_deerclops, b3.downed_deerclops);
        assert_eq!(feeder.downed_boss2, b3.downed_boss2);
        assert_eq!(feeder.downed_boss3, b3.downed_boss3);
        assert_eq!(feeder.downed_queen_bee, b3.downed_queen_bee);
        assert_eq!(feeder.hard_mode, b3.hard_mode);
        assert_eq!(feeder.downed_queen_slime, b3.downed_queen_slime);
        assert_eq!(feeder.downed_mech1, b3.downed_mech1);
        assert_eq!(feeder.downed_mech2, b3.downed_mech2);
        assert_eq!(feeder.downed_mech3, b3.downed_mech3);
        assert_eq!(feeder.downed_plant, b3.downed_plant);
        assert_eq!(feeder.downed_empress, b3.downed_empress);
        assert_eq!(feeder.downed_fishron, b3.downed_fishron);
        assert_eq!(feeder.downed_golem, b3.downed_golem);
        assert_eq!(feeder.downed_cultist, b3.downed_cultist);
        assert_eq!(feeder.homeless, b3.homeless);
        assert_eq!(feeder.breath, b3.breath);
        assert_eq!(feeder.shimmering, b3.shimmering);
    }

    /// W4-g90 RETIRE — the b3-dedup receipt witness 2/2 (WRITTEN NOT
    /// RUN): with both books up the feeder's ladder writes 750 HP /
    /// def 31 while b3's book-two-false fold writes 500 HP / def 23 —
    /// the delta is exactly one ladder row (+250 HP / +8 def / +0.25
    /// num2 / ×0.8 num3). The merge dedup (b3's arm env through
    /// `from_world(...).env(...)`, its `:1737` ladder call onto the
    /// feeder pre-pass) restores it mechanically: same inputs → same
    /// writes, zero draws (the idempotence witness above).
    #[test]
    fn w4_g90_the_b3_book_two_fold_drops_exactly_one_ladder_row() {
        let mut w = ground();
        w.combat_book_was_used = true;
        w.combat_book_volume_two_was_used = true;
        w.downed_eoc = true;
        let feeder = TownEnvFlags::from_world(&w, false).env(&w, &[], &[], false);
        let mut b3 = TownEnvFlags::from_world(&w, false).env(&w, &[], &[], false);
        b3.combat_book_two = false; // the b3 phase-1 fold
        let mut n_feed = guide(0.0, 0.0, None);
        let mut n_b3 = guide(0.0, 0.0, None);
        let (f2, f3) = town_stat_bonuses(&mut n_feed, stats(), &feeder, false, false);
        let (b2, b3v) = town_stat_bonuses(&mut n_b3, stats(), &b3, false, false);
        assert_eq!(n_feed.max_hp, 750, "the feeder feeds BOTH books");
        assert_eq!(n_b3.max_hp, 500, "b3's fold drops book two's +250");
        assert_eq!(n_feed.hp, 750, "the full-heal-on-growth both ways");
        assert_eq!(n_b3.hp, 500);
        assert_eq!(n_feed.defense_override, Some(15 + 8 + 8 + 2));
        assert_eq!(n_b3.defense_override, Some(15 + 8 + 2));
        assert!((f2 - b2 - 0.25).abs() < 1e-5, "one +0.25 num2 row");
        assert!((f3 - b3v * 0.8).abs() < 1e-5, "one ×0.8 num3 row");
    }
}
