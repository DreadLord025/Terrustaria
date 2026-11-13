//! Player physics and collision, modelled on `Player.cs`.
//!
//! Core movement: gravity, acceleration/friction, jump, 1-tile auto-step, axis-separated
//! AABB collision against solid tiles, simple water buoyancy and ground-slope support
//! (slopes 1/2) and half bricks/ceiling slopes during movement. Survival follows
//! `Player.cs`: a 200-tick breath meter (1 unit every 7
//! ticks underwater, 2 damage once empty), fall damage over 25 tiles, lava damage and
//! out-of-combat health regeneration. Half bricks and ceiling slopes still count as full
//! cells.

use crate::registry;
use tr_core::tiledata as td;
use tr_core::{TILE_PX, World};

use crate::buffs::{BuffSlots, MAX_BUFFS};
use crate::collision;
use crate::npc::Npc;
use crate::dye;
use crate::item::{INV, ItemSlot};
use crate::luck::LuckState;
use crate::minecart;
use crate::projectile::{BABY_BIRD, PROJ_COUNT, ProjectilePool};
use crate::regen::{DotDeath, RegenInputs, RegenState, update_life_regen, update_mana_regen};
use glam::Vec2;

pub const WIDTH: f32 = 20.0;
pub const HEIGHT: f32 = 42.0;

const GRAVITY: f32 = 0.4;
const MAX_FALL: f32 = 10.0;
/// `Player.jumpSpeed` / `Player.jumpHeight`: holding the jump key keeps the
/// upward velocity at `jumpSpeed` for `jumpHeight` ticks, so tap and hold jumps
/// reach different heights.
const JUMP_SPEED: f32 = 5.01;

/// The base minecart's mount data (`Mount.SetAsMinecart` via the `mounts[6]`
/// registration, `Mount.cs:918-931`): `runSpeed = dashSpeed = 13`,
/// `acceleration = 0.04`, `jumpHeight = 15`, `jumpSpeed = 5.15` — the C#
/// `JumpHeight`/`JumpSpeed` cases (`Mount.cs:2810-2858`) list no cart, so the
/// stats are fixed (no |vx| scaling).
const CART_RUN_SPEED: f32 = 13.0;
const CART_DASH_SPEED: f32 = 13.0;
const CART_ACCELERATION: f32 = 0.04;
const CART_JUMP_HEIGHT: i32 = 15;
const CART_JUMP_SPEED: f32 = 5.15;
/// `mountData.buff` for the minecart (`Mount.cs:920`) — `AddBuff(buff, 3600)`
/// on mount (`Mount.cs:6226`), cleared on dismount (`:6151`).
const CART_MOUNT_BUFF: i32 = 118;
/// The mounted body height: `42 + heightBoost` (`SetMount`, `Mount.cs:6270`;
/// `heightBoost = 10`, `Mount.cs:923`). The width stays 20.
const CART_MOUNT_HEIGHT: f32 = 52.0;
/// `Player.breathMax`; `breath` is in ticks.
pub const BREATH_MAX: i32 = 200;
/// `Player.breathCDMax`: ticks between breath decrements.
const BREATH_CD: i32 = 7;

/// The extra-jump options (`Player.hasJumpOption_*` / `canJumpAgain_*` /
/// `isPerformingJump_*`, `Player.cs:2664-2716`). The mount jumps
/// (Unicorn/Santank/WallOfFleshGoat/Basilisk) arm from the mounts
/// (`Mount.cs:4884-4968`), the bottles from the accessories
/// (`Player.cs:14787-15003`) — both out until M3-EQUIP-3/M2-MOUNT-1, so
/// `TR_JUMPS=` pins them.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ExtraJump {
    Cloud,
    Sandstorm,
    Blizzard,
    Fart,
    Sail,
    Unicorn,
    Santank,
    WallOfFleshGoat,
    Basilisk,
}

impl ExtraJump {
    /// The scan index into the flag arrays.
    pub(crate) fn index(self) -> usize {
        match self {
            ExtraJump::Cloud => 0,
            ExtraJump::Sandstorm => 1,
            ExtraJump::Blizzard => 2,
            ExtraJump::Fart => 3,
            ExtraJump::Sail => 4,
            ExtraJump::Unicorn => 5,
            ExtraJump::Santank => 6,
            ExtraJump::WallOfFleshGoat => 7,
            ExtraJump::Basilisk => 8,
        }
    }

    /// The consumption priority (`Player.cs:20698-20748`): the mount jumps
    /// first, then Sandstorm, Blizzard, Fart, Sail, Cloud as the
    /// unconditional fall-through.
    pub(crate) const PRIORITY: [ExtraJump; 9] = [
        ExtraJump::Basilisk,
        ExtraJump::WallOfFleshGoat,
        ExtraJump::Santank,
        ExtraJump::Unicorn,
        ExtraJump::Sandstorm,
        ExtraJump::Blizzard,
        ExtraJump::Fart,
        ExtraJump::Sail,
        ExtraJump::Cloud,
    ];

    /// The hold-boost counter the jump grants, in `jumpHeight` multiples
    /// (`Player.cs:20787-21006`).
    pub(crate) fn hold_multiplier(self) -> f32 {
        match self {
            ExtraJump::Sandstorm => 3.0,
            ExtraJump::Blizzard => 1.5,
            ExtraJump::Fart => 2.0,
            ExtraJump::Sail => 1.25,
            ExtraJump::Unicorn => 2.0,
            ExtraJump::Santank => 2.0,
            ExtraJump::WallOfFleshGoat => 2.0,
            ExtraJump::Basilisk => 0.75,
            ExtraJump::Cloud => 0.75,
        }
    }

    /// The `TR_JUMPS=` name.
    pub(crate) fn parse_name(name: &str) -> Option<Self> {
        Some(match name.trim().to_ascii_lowercase().as_str() {
            "cloud" => ExtraJump::Cloud,
            "sandstorm" => ExtraJump::Sandstorm,
            "blizzard" => ExtraJump::Blizzard,
            "fart" => ExtraJump::Fart,
            "sail" | "tsunami" => ExtraJump::Sail,
            "unicorn" => ExtraJump::Unicorn,
            "santank" => ExtraJump::Santank,
            "goat" | "walloffleshgoat" => ExtraJump::WallOfFleshGoat,
            "basilisk" => ExtraJump::Basilisk,
            _ => return None,
        })
    }
}

#[derive(Default, Clone, Copy)]
pub struct PlayerInput {
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub down: bool,
    /// `Player.controlHook` (the quick-grapple key).
    pub hook: bool,
}

/// The DoT-derived half of [`RegenInputs`] for one tick — what the held buffs
/// say about the drain (`Player.cs:18949-19385`). `update_survival` copies the
/// fields into the engine's inputs; the engine (regen.rs) owns the lanes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DotDerivation {
    /// The summed DoT delta (honey mitigation included) → `RegenInputs::dot_regen`.
    regen: i32,
    /// Any DoT (or bleeding) held → `RegenInputs::dot_resets_time`.
    resets_time: bool,
    /// `burned || suffocating || (tongued && expert)` (`Player.cs:19313`).
    fast_lane: bool,
    /// The fast lane's kill reason (7 → 12 → 8, `Player.cs:19321-19331`).
    death_fast: DotDeath,
    /// The generic lane's kill reason (22 → 9 → 10 → 23 → 8, `:19362-19385`).
    death_generic: DotDeath,
}

/// The `WetCollision` movement speed per liquid (`Player.cs:28168-28171`):
/// water 0.5, lava 0.5, honey 0.25, shimmer 0.375, dry 1.0.
pub fn movement_speed_for(wet: bool, lava_wet: bool, honey_wet: bool, shimmer_wet: bool) -> f32 {
    if honey_wet {
        0.25
    } else if shimmer_wet {
        0.375
    } else if wet || lava_wet {
        0.5
    } else {
        1.0
    }
}

/// The wet-family flags the statics rebase reads (`Player.cs:24380-24437`).
/// The not-yet-ported writers default `false` — the C#'s pre-condition
/// state: `shimmering` (the absorbed state, the shimmer-emergence work),
/// `merman` (the buff table, S1-E-7), `trident` (`Player.cs:12896`'s use
/// arm), `control_up` (no input channel yet), `down_dash`
/// (`isPerformingJump_DownDash` — only the Dead Cells down-dash sets it,
/// `Player.cs:20697-20722`), `vortex_debuff` (the Moon Lord debuff) and
/// `portal_physics` (no portals).
#[derive(Clone, Copy, Debug, Default)]
pub struct WetStaticsFlags {
    pub wet: bool,
    pub honey_wet: bool,
    pub lava_wet: bool,
    pub shimmer_wet: bool,
    pub shimmering: bool,
    pub merman: bool,
    pub trident: bool,
    pub control_up: bool,
    pub down_dash: bool,
    pub vortex_debuff: bool,
    pub portal_physics: bool,
}

/// `Player.cs:24380-24437` — the per-tick statics rebase that runs right
/// after the movement defaults (`:24364-24369` resets `maxFallSpeed = 10f`,
/// `gravity = Player.defaultGravity`, `jumpHeight = 15`, `jumpSpeed =
/// 5.01f`): the wet/shimmer arms assign the tick's `gravity`/`maxFallSpeed`
/// and the jump statics (`Player.jumpHeight`/`jumpSpeed` are statics in the
/// C#, instance fields here) BEFORE any integration, then `vortexDebuff`
/// zeroes the gravity and every tick adds the `+0.01f` fall-cap creep. The
/// plain integration arm (`Player.cs:27352-27359`) consumes the rebased
/// values at FULL strength — the `gravity / 3` / `maxFallSpeed / 3` sites
/// (`:27008`, `:27339`) are the mount/wings flight integrators, not a wet
/// rule. Returns `(gravity, max_fall_speed, jump_height, jump_speed)`.
pub fn wet_gravity_and_cap(
    gravity: f32,
    max_fall_speed: f32,
    jump_height: i32,
    jump_speed: f32,
    flags: WetStaticsFlags,
) -> (f32, f32, i32, f32) {
    let WetStaticsFlags {
        wet,
        honey_wet,
        lava_wet,
        shimmer_wet,
        shimmering,
        merman,
        trident,
        control_up,
        down_dash,
        vortex_debuff,
        portal_physics,
    } = flags;
    let mut gravity = gravity;
    let mut max_fall_speed = max_fall_speed;
    let mut jump_height = jump_height;
    let mut jump_speed = jump_speed;
    if portal_physics {
        max_fall_speed = 35.0; // `:24380-24382`
    }
    if !shimmering && wet && down_dash {
        gravity *= 0.85; // `:24384-24387`
        max_fall_speed *= 0.85;
    } else if shimmer_wet || shimmering {
        if shimmering {
            gravity *= 0.9; // `:24389-24394`
            max_fall_speed *= 0.9;
        } else {
            gravity = 0.15;
            jump_height = 23;
            jump_speed = 5.51;
        }
    } else if wet {
        if honey_wet {
            gravity = 0.1; // `:24399-24402`
            max_fall_speed = 3.0;
        } else if merman {
            gravity = 0.3; // `:24404-24407`
            max_fall_speed = 7.0;
        } else if trident && !lava_wet {
            gravity = 0.25; // `:24409-24417`
            max_fall_speed = 6.0;
            jump_height = 25;
            jump_speed = 5.51;
            if control_up {
                gravity = 0.1;
                max_fall_speed = 2.0;
            }
        } else {
            gravity = 0.2; // `:24418-24424`
            max_fall_speed = 5.0;
            jump_height = 30;
            jump_speed = 6.01;
        }
    }
    if vortex_debuff {
        gravity = 0.0; // `:24433-24436`
    }
    max_fall_speed += 0.01; // `:24437`
    (gravity, max_fall_speed, jump_height, jump_speed)
}

/// `Main.OffsetsPlayerHeadgear[i].Y` (`Main.cs:66638-66660`): the per-body-row
/// headgear lift the draw and the Moon Lord leg rows read. Rows 7-9 and 14-16
/// are the `0` entries.
pub fn offsets_player_headgear_y(row: usize) -> f32 {
    match row.min(19) {
        7..=9 | 14..=16 => 0.0,
        _ => 2.0,
    }
}

/// The held-item inputs of `PlayerFrame()` (`Player.cs:36144-36147`). The C#
/// reads `this.inventory[this.selectedItem]` straight out of the player; the
/// port keeps the inventory in the app, so the app hands the same fields in
/// before each fixed step (`world_sim/player.rs`, beside `swing_aim`).
#[derive(Clone, Copy, Debug, Default)]
pub struct FrameInputs {
    /// `inventory[selectedItem].useStyle` — drives the item-use body chain.
    pub use_style: u8,
    /// `flag6` (`:36146`): `useStyle != 10 && != 14 && != 15`. Those three
    /// styles (the fishing rod family and the like) skip the item-use rows.
    pub use_style_blocks_frames: bool,
    /// `inventory[selectedItem].type` — the useStyle 5 torch carve-out and the
    /// `type == 0` empty-hand share of the useStyle 1 branch.
    pub item_type: i32,
    /// `inventory[selectedItem].holdStyle` — the idle hold rows.
    pub hold_style: i32,
    /// `flag5` = `CanVisuallyHoldItem(HeldItem)` (`:36143`).
    pub can_visually_hold: bool,
    /// `itemAnimation > 0`.
    pub swing_active: bool,
    /// `itemAnimation / itemAnimationMax` (1 at the start, 0 at the end).
    pub swing_remaining: f32,
    /// `itemRotation * direction` — the useStyle 5 aim.
    pub aim_relative: f32,
    /// `!wet || !item.noWet` — the two held-item arms that the water kills.
    pub hold_style_dry: bool,
    /// `!happyFunTorchTime || createTile != 4` — the torch hold override.
    pub torch_hold_ok: bool,
    /// The grapple body arm's lean — the hooks' centroid minus the body centre
    /// (`Player.cs:36502-36520`).
    pub grapple_dx: f32,
    pub grapple_dy: f32,
}

/// `ApplyEquipFunctional`'s `buffImmune[...] = true` rows (`Player.cs:
/// 15300-15570`): the Ankh Shield's eleven debuffs (1612/1613), its parts
/// (885-893 one each, 901-904 two each), the Chill/Frozen pair on 1921 and the
/// 156/22 rows on 3781/5354. The `ResetEffects` head clears the table every
/// tick (`Player.Update`'s `:25563` loop, the port's `reset_buff_immune`), so
/// these re-apply inside the equipment pass.
pub const BUFF_IMMUNITY_ROWS: &[(u16, &[i32])] = &[
    (885, &[30]),
    (886, &[36]),
    (887, &[20]),
    (888, &[22]),
    (889, &[32]),
    (890, &[35]),
    (891, &[23]),
    (892, &[33]),
    (893, &[31]),
    (901, &[33, 36]),
    (902, &[30, 20]),
    (903, &[32, 31]),
    (904, &[35, 23]),
    (1612, &[33, 36, 30, 20, 32, 31, 35, 23, 22, 156]),
    (1613, &[46, 33, 36, 30, 20, 32, 31, 35, 23, 22, 156]),
    (1921, &[46, 47]),
    (3781, &[156]),
    (5354, &[22, 156]),
];

#[derive(Clone, Debug)]
pub struct Player {
    // The class combat stats (`Player.cs:2430-2437`): every class starts at
    // crit 4 and a 1.0 damage multiplier; the equipment modifies them
    // (M3-EQUIP).
    pub melee_crit: i32,
    pub ranged_crit: i32,
    pub magic_crit: i32,
    pub melee_damage: f32,
    pub ranged_damage: f32,
    pub magic_damage: f32,
    /// `Player.minionDamage` (`Player.cs:2437`, default 1.0): the summon
    /// class multiplier — the minion prefixes 69-72 add to it through
    /// `weapon_damage_multiplier`'s Summon arm.
    pub minion_damage: f32,
    // The ranged effective-damage components (`Player.cs:2442-2450` +
    // `arrowDamageAdditiveStack`, reset `:18338`/`:18396-18399`, read by
    // `GetWeaponDamageMultiplier`'s ranged arms through the computed
    // properties `:3982-3986`). S1-C-4: the arms land with the multiplier;
    // the quiver stack + the Archery buff arm them now, the Shroomite
    // helmets ride the armor walk, and the `rangedMultDamage` consumers
    // (the C#'s separate multiplier class) keep the reset 1.0.
    pub ranged_mult_damage: f32,
    pub arrow_damage: f32,
    pub arrow_damage_additive_stack: f32,
    pub bullet_damage: f32,
    pub rocket_damage: f32,
    /// The per-player summon budget (M4-PROJ-8): `Player.maxMinions = 1`
    /// (`Player.cs:1260`) — raised by the buffs 110/150 and the summon armor
    /// rows, reset to 1 by `ResetEffects` (`:18448`).
    pub max_minions: i32,
    /// `Player.numMinions` (`Player.cs:1262`) — accumulated by the pool's
    /// minion block (`Projectile.cs:16736`, Phase 3), zeroed at `:28279` and
    /// on death (`:17009`); the buff-216 arm reads it (`:10706`).
    pub num_minions: i32,
    /// `Player.slotsMinions` (`Player.cs:1264`, float) — the 0.5-slot
    /// spiders' fractional budget, accumulated beside `numMinions`.
    pub slots_minions: f32,
    /// `Player.maxTurrets = 1` (`Player.cs:2848`) — the sentry budget, raised
    /// by buff 348 and reset by `ResetEffects` (`:18449`).
    pub max_turrets: i32,
    /// `Player.maxTurretsOld` (`Player.cs:2850`) — the last observed
    /// `max_turrets`; a change drives the `UpdateMaxTurrets` sweep
    /// (`:25709-25712`, Phase 5).
    pub max_turrets_old: i32,
    /// `Player.MinionAttackTargetNPC = -1` (`Player.cs:3008`) — the
    /// right-click aim target; written by the use path (Phase 6).
    pub minion_attack_target_npc: i32,
    /// `Player.ownedProjectileCounts[ProjectileID.Count]` (`Player.cs:2992`)
    /// — recounted every tick from the live pool (`:25565`); the summon
    /// buff-arms and the keep-alive checks read exactly this.
    pub owned_projectile_counts: [i32; PROJ_COUNT],
    /// `Player.highestStormTigerGemOriginalDamage` (`Player.cs:1304`) — the
    /// recount's type-831 maximum (`:12194-12215`).
    pub highest_storm_tiger_gem_original_damage: i32,
    /// `Player.highestAbigailCounterOriginalDamage` (`Player.cs:1314`) — the
    /// recount's type-970 maximum; AI_062's 963 scaling reads it.
    pub highest_abigail_counter_original_damage: i32,
    /// The 29 summon keep-alive flags `ResetEffects` clears
    /// (`Player.cs:18620-18646`+`:18741`) and `UpdateBuffs`' arms re-affirm
    /// (`:10483-10972`); AI_062 reads them to pin `timeLeft = 2` (Phase 4).
    /// Fields verbatim: `Player.cs:1268-1300` (order here groups the arm
    /// table's pairs).
    pub pygmy: bool,
    pub raven: bool,
    pub slime: bool,
    pub hornet_minion: bool,
    pub imp_minion: bool,
    pub spider_minion: bool,
    pub twins_minion: bool,
    pub pirate_minion: bool,
    pub vampire_frog: bool,
    pub sharknado_minion: bool,
    pub ufo_minion: bool,
    pub stardust_minion: bool,
    pub bats_of_light: bool,
    pub baby_bird: bool,
    pub dead_cells_mushroom_boi_minion: bool,
    pub palworld_cattiva_minion: bool,
    pub palworld_foxsparks_minion: bool,
    pub palworld_trusty_cattiva_minion: bool,
    pub palworld_trusty_foxsparks_minion: bool,
    pub clay_pot_minion: bool,
    pub forbidden_minion: bool,
    pub flinx_minion: bool,
    pub abigail_minion: bool,
    pub storm_tiger: bool,
    pub smolstar: bool,
    pub empress_blade: bool,
    pub stardust_guardian: bool,
    pub stardust_dragon: bool,
    /// `Player.DeadlySphereMinion` (`Player.cs:1288`) — buff 161's flag, the
    /// chain's last arm (`:10957-10972`).
    pub deadly_sphere_minion: bool,
    /// `Player.dead`: the player is dead and waiting for the respawn timer
    /// (`Player.cs:38908`).
    pub dead: bool,
    /// `Player.numberOfDeathsPVE` (`Player.cs:834`, `:38844`): the PVE death
    /// counter `/death` reads (`DeathCommand`).
    pub deaths_pve: u32,
    /// `Player.numberOfDeathsPVP` (`Player.cs:836`, `:38840`): the PVP death
    /// counter `/pvpdeath` reads (`PVPDeathCommand`).
    pub deaths_pvp: u32,
    /// `Player.hostile` (`Player.cs:2570`): the PVP toggle the inventory's
    /// `DrawPVPIcons` reads and toggles (`Main.cs:42664`, `:42692`). Gates the
    /// PVP damage rules (`damage::*`, `Player.Damage_PVP`).
    pub hostile: bool,
    /// `Player.team` (`Player.cs:1430`): 0 = none, 1..=5 = Red/Green/Blue/
    /// Yellow/Pink (`PlayerTeamID`). The `DrawPVPIcons` row index.
    pub team: i32,
    /// `Player.hideVisibleAccessory[10]` (`Player.cs:1708`): per-accessory
    /// "hide the visuals" flags, toggled by the ticks beside the accessory
    /// column (`Main.cs:41426-41460`) and read by the dye/visual update
    /// (`Player.cs:9610`, `:14378`). Indices 0..2 are the armour slots and keep
    /// their ticks switched off, exactly like the vanilla's `num38 > 2` gate.
    pub hide_visible_accessory: [bool; 10],
    /// `Player.deadTime`: the ticks spent dead (`Player.cs:16923`); gates the
    /// respawn-skip (90-tick lockout, `Player.cs:57111`).
    pub dead_time: i32,
    /// `Player.anglerQuestsFinished` (`Player.cs:1910`): the count
    /// the Angler's reward table reads (`GetAnglerReward_MainReward`'s
    /// milestone switch, `Player.cs:56323`).
    pub angler_quests_finished: i32,
    /// `Player.golferScoreAccumulated` (`Player.cs:1912`): the golf score
    /// the Golfer's shop and chatter tier (`Chest.cs:3158`, `NPC.cs:94792`).
    pub golfer_score: i32,
    /// `Player.respawnTimer`: the ticks left until the respawn.
    pub respawn_timer: i32,
    /// `Player.TagEffectStack` (`Player.cs`'s field): the whip tags this
    /// player applied (`tag_effects.rs`, M4-PROJ-7).
    pub tag_effects: crate::tag_effects::TagEffectStack,
    /// The jump-input edge state for the dead-skip roll.
    prev_jump_input: bool,
    /// The `UpdateBuffs`-derived debuff flags (`ResetEffects` clears,
    /// `Player.cs:18640-18720`; the per-slot derivation `:10039-10061` and
    /// the Weak/Webbed/Darkness rows). Movement reads `slow`/`chilled` as
    /// the `strongestMoveSpeedDebuff` tiers (`:25945-25953`), `frozen`/
    /// `webbed` lock the controls and the item use (`:3381`), `blind` drops
    /// `Lighting.GlobalBrightness` 1.2 → 1 (`Lighting.cs:112`).
    /// The `Player.SceneMetrics` aura snapshot — the app refreshes it per
    /// tick from the scene scan; the buff grants and the regen inputs read
    /// it (`M2-BUFF-5`).
    pub auras: tr_core::scene::Auras,
    /// `Player.manaRegenBonus` — the star-in-bottle's +10 per slot
    /// (`Player.cs:9933`), zeroed by `ResetEffects` (`:18655`).
    pub mana_regen_bonus: i32,
    /// `Player.wellFed` — any of the fed tiers 26/206/207 held
    /// (`Player.cs:11933-11967`); gates the re-eat (`:14353`) and the
    /// expert regen halving.
    pub well_fed: bool,
    /// The WellFed mining speed bonus — the `pickSpeed -=` negative
    /// (`:11945`: the tier's -0.05/-0.1/-0.15); the mining hardness
    /// multiplies by (1 - bonus).
    pub pick_speed_bonus: f32,
    /// The WellFed melee speed add (`meleeSpeed +=`, `:11937`) — rides the
    /// swing pacing until the per-item meleeSpeed lands (M3-ITEM).
    pub melee_speed_bonus: f32,
    /// `Player.statDefense` — the armor pieces' defense + the set bonus
    /// (M3-EQUIP-2); the buff arms' `statDefense` adds join as their
    /// systems land.
    pub stat_defense: i32,
    pub slow: bool,
    pub chilled: bool,
    pub frozen: bool,
    pub webbed: bool,
    pub blind: bool,
    /// The per-slot Weak count — `meleeDamage -= 0.051f` and
    /// `moveSpeed -= 0.1f` per slot (`Player.cs:11873-11878`); the
    /// `statDefense -= 4` half has no player-defense consumer yet
    /// (M3-EQUIP-3) and rides `weak_stacks` until it lands.
    pub weak_stacks: i32,
    /// Top-left of the 20x42 hitbox, world pixels.
    pub x: f32,
    pub y: f32,
    /// Position at the previous fixed 60 Hz step, used to interpolate rendering
    /// between simulation ticks on high-refresh displays.
    pub prev_x: f32,
    pub prev_y: f32,
    pub vx: f32,
    pub vy: f32,
    pub on_ground: bool,
    /// 1 = right, -1 = left.
    pub facing: i32,
    /// The equipped item kinds — `Player.armor[0..20]`'s kinds snapshot the
    /// update's grant pass reads (`UpdateEquips`, `Player.cs:12892`). Filled
    /// from `App.equip` per tick.
    pub equipped: [u16; 20],
    /// `Player.dye[0..10]`'s kinds — `UpdateDyes`' input through
    /// `GetEffectiveDye` (`Player.cs:5742-5764`; the port has no loadouts, so
    /// the slot itself is the source). Filled from `App.equip` per tick next
    /// to `equipped`.
    pub dye_slots: [u16; 10],
    /// `Player.miscDyes[0..5]`'s kinds (`Player.cs:1494`; pet/light/cart/
    /// mount/hook). Filled from `App.equip` per tick.
    pub misc_dye_slots: [u16; 5],
    /// `UpdateDyes`' `c*` cache (`Player.cs:9587-9757`) — the dye pass each
    /// draw layer applies (the `DrawData.shader` source, 0 = plain). The port
    /// fills the fields with draw consumers: the armor trio reads its dye
    /// slots directly (`cHead/cBody/cLegs = GetEffectiveDye(0..2).dye`,
    /// `:9593-9595`) and the misc five mirror `:9600-9604` (`cMount` feeds
    /// the mount layers; the pets/minions have no draw path yet). The ~27
    /// accessory effect-slot fields of `UpdateItemDye` wait for the accessory
    /// visuals — deviation in `docs/ANALYSIS.md`.
    pub c_head: u32,
    pub c_body: u32,
    pub c_legs: u32,
    pub c_pet: u32,
    pub c_light: u32,
    pub c_minecart: u32,
    pub c_mount: u32,
    pub c_grapple: u32,
    /// The equipped items' rolled prefixes, slot-parallel to `equipped` —
    /// `GrantPrefixBenefits` (`Player.cs:13953-14044`) reads them in the
    /// accessory grant pass below.
    pub equipped_prefix: [u16; 20],
    /// The four banks (`Player.bank/bank2/bank3/bank4`,
    /// `Player.cs:1580-1591` — `Chest.CreateBank(-2..-5)`, 40 slots each;
    /// player-bounded, always unlocked per `Chest.cs:178-189`).
    pub banks: [crate::bank::BankStore; 4],
    /// `Player.chest` (`Player.cs:2876`): the single open-container index —
    /// -1 closed, -2..-5 the banks (`bank.rs`'s `bank_for_chest`), >= 0 the
    /// `App.world_chests` store index. Every open/close/tick route writes
    /// this one field (`chest.rs`'s shared path — the M3-CHEST-INT unify).
    pub chest: i32,
    /// `Player.chestX`/`chestY` (`Player.cs:2878`/:2880): the open
    /// container's tile origin — set for the banks too (`OpenChest`
    /// `:33524-33525`), the per-tick range close's probe point.
    pub chest_x: i32,
    pub chest_y: i32,
    /// `Player.IsVoidVaultEnabled` (`Player.cs:24550-24553` —
    /// `HasItem(4131)`): refreshed per tick beside the other scans.
    pub void_vault_enabled: bool,
    /// The `TR_JUMPS`/`TR_BOOTS`/`TR_WINGS` pins: the debug stand-ins for the
    /// granting equipment, re-applied by the accessory pass every tick (the
    /// vanilla `ResetEffects` clears the flags and the grants re-set them).
    pub pinned_jumps: [bool; 9],
    pub pinned_balloon: bool,
    pub pinned_rocket_boots: i32,
    /// The `TR_MOUNT=<id|name>` pin: the debug stand-in for the saddle item
    /// (the real items equip through `Item.mountType`), re-applied after the
    /// per-tick flag clear while no mount is active (the `pinned_*` pattern,
    /// docs/ANALYSIS.md "Item / equip deviations").
    pub pinned_mount: Option<i32>,
    /// `PlayerSittingHelper.isSitting` (`PlayerSittingHelper.cs:18`) — the
    /// chair-sit state; the player's bottom is snapped to the seat.
    pub sitting: bool,
    /// `PlayerSleepingHelper.isSleeping` (`PlayerSleepingHelper.cs:13`) — the
    /// bed-sleep state; the bottom is snapped to the bed anchor and the pose
    /// rotates through `full_rotation`.
    pub sleeping: bool,
    /// `PlayerSleepingHelper.timeSleeping` (`:17`): the fully-asleep counter
    /// — `>= 120` (`TimeToFullyFallAsleep`, `:11`) arms the clock's ×5
    /// (`World::day_rate`).
    pub sleep_time: u32,
    /// `PlayerSleepingHelper.visualOffsetOfBedBase`'s key: the bed style
    /// (`frameY / 36`, `GetSleepingTargetInfo` `:222`) refreshed per tick —
    /// the renderer derives the offset from the `SetOffsetbyBed` table.
    pub sleeping_style: u32,
    /// `PlayerSittingHelper.offsetForSeat` (`:14`): the per-style seat drop
    /// the decoder returned, refreshed by `update_sitting` (`:84`).
    offset_for_seat: (f32, f32),
    /// `PlayerSittingHelper.GetSittingOffsetInfo`'s `posOffset`
    /// (`PlayerSittingHelper.cs:18-32`, `sittingIndex` 0 in SP): the seat
    /// offset mirrored to the facing — the draw pass reads it.
    pub seat_pos_offset: (f32, f32),
    /// `GetSittingOffsetInfo`'s `seatAdjustment` (`:23-24`): `-4 +
    /// (int)offsetForSeat.Y` — the torso drop while seated.
    pub seat_y_offset: i32,
    /// `Player.noKnockback` (`Player.cs:1950`) - the Cobalt Shield's grant
    /// (`Player.cs:15025-15028`); the Hurt gate reads it.
    pub no_knockback: bool,
    /// M5-SEED-2: `Main.vampireSeed` — the `vampirism` micro seed's
    /// `InitializeSecretSeeds` write (`WorldGen.cs:555-558`). `UpdateLifeRegen`'s
    /// OnFire row reads it (`Player.cs:18983-18992`: −100 instead of −8, and the
    /// `drippingSlime` double is 100 instead of 8). Captured from the world at
    /// spawn (`Player::spawn`) — the flag is immutable post-generation, exactly
    /// like the C#'s world-level static.
    vampirism: bool,
    /// `Player.itemRotation`'s source for useStyle 5 — the aim angle the App
    /// updates per tick (`swing_aim`), mirrored onto the player for the draw.
    pub swing_aim: f32,
    /// The accessory-granted `Player.lifeRegen` base (`Player.cs:13122`'s
    /// `lifeRegen += armorPiece.lifeRegen` - the Band's `Item.lifeRegen = 2`).
    accessory_life_regen: i32,
    /// `Player.legFrame.Y / 56`: the current leg animation row (0..=19), driven by
    /// `PlayerFrame()`.
    pub leg_frame: u32,
    /// `Player.bodyFrame.Y / 56`: the current body animation row (0..=19). Mirrors the
    /// legs while walking; the app overrides it with the item-use rows during a swing.
    pub body_frame: u32,
    /// `Player.legFrameCounter`: accumulates `|velocity.X| * 1.3` while walking and
    /// advances a leg row every `8.0`.
    leg_frame_counter: f32,
    /// `Player.bodyFrameCounter` (`Player.cs:36594`): only the `legs == 140`
    /// walk branch accumulates it (`|velocity.X| * 0.5`); every other body arm
    /// either zeroes it or leaves it alone.
    body_frame_counter: f32,
    /// `Player.swimTime` (`Player.cs:35756-35762`): the merman/flipper swim
    /// window; `PlayerFrame` decrements it and clears it out of water.
    pub swim_time: i32,
    /// `Player.miscCounter`: ticked once per update in the original; the
    /// sandstorm leg pose turns the player every 4 ticks on it
    /// (`Player.cs:36302`).
    pub misc_counter: u32,
    /// `Player.pulley` / `Player.pulleyDir` (`Player.cs:1070`): the rope
    /// pose — body rows 1 (dir 2) and 2 (dir 1).
    pub pulley: bool,
    pub pulley_dir: u8,
    /// `Player.shieldRaised` (`Player.cs:36481`): body row 10.
    pub shield_raised: bool,
    /// `Player.sliding` (`Player.cs`): the slope/ice slide pose (body row 3 in
    /// the air). No port source sets it yet, so the arm is coded but
    /// unreachable.
    pub sliding: bool,
    /// `Player.sandStorm` (`Player.cs:36298`): the Sandstorm-in-a-Bottle dash
    /// pose (leg row 0, body row 6, a 4-tick turn cadence).
    pub sand_storm: bool,
    /// `Player.carpetFrame` (`Player.cs:1094`, init `-1`): the Magic Carpet
    /// glide pose (leg row 0, body row 6).
    pub carpet_frame: i32,
    /// `Player.legs` (`Player.cs:35766`): the effective legs armour texture id
    /// (`Item.legSlot`). The Moon Lord carve-out (`140`) and the sitting
    /// carve-out (`217`) read it; `0` = the default legs sheet.
    pub legs_armor: i32,
    /// The held-item half of `PlayerFrame()` — the C# reads
    /// `inventory[selectedItem]` directly, the port feeds the summary in from
    /// the app before the tick (`world_sim/player.rs`).
    pub frame_inputs: FrameInputs,
    /// `Player.wet` — the head/body in WATER (`Player.cs:27728`).
    pub wet: bool,
    /// `Player.lavaWet` — in LAVA.
    pub lava_wet: bool,
    /// `Player.honeyWet` — in HONEY.
    pub honey_wet: bool,
    /// `Player.shimmerWet` — in SHIMMER.
    pub shimmer_wet: bool,
    /// The `WetCollision` movement-speed scale for this tick's position delta
    /// (`Player.cs:28168-28171`): water/lava 0.5, honey 0.25, shimmer 0.375,
    /// dry 1.0.
    pub liquid_step: f32,
    pub in_water: bool,
    /// `Player.controlLeft` / `Player.controlRight`: this tick's held run keys
    /// (PlayerInput). The electrified drain tests them (Player.cs:19080-19082).
    pub control_left: bool,
    pub control_right: bool,
    /// `Player.controlJump` / `controlDown`: this tick's held jump/down keys.
    pub control_jump: bool,
    pub control_down: bool,
    /// Remaining ticks of the held-jump boost (`Player.jump`).
    jump: i32,
    /// Jump was held on the previous fixed step, for press-edge detection.
    jump_held: bool,
    pub spawn_x: f32,
    pub spawn_y: f32,
    pub hp: i32,
    pub max_hp: i32,
    /// `Player.statMana` / `Player.statManaMax` (`Player.cs:1930-1932`). A new
    /// character starts at `20/20` (`UICharacterCreation.cs:1721`; the default max
    /// is also 20 in `Player.Initialize`, `Player.cs:56075`).
    /// `Player.armorPenetration` / `meleeArmorPenetration` (`Player.cs:1918-1920`):
    /// the flat penetration the hit adds to `NPC.checkArmorPenetration`
    /// (`:44383`). `meleeArmorPenetration` only counts for melee hits.
    pub armor_penetration: i32,
    pub melee_armor_penetration: i32,
    /// `Player.lavaTime` / `lavaMax` (`Player.cs:1528-1530`): the lava
    /// immunity budget — the Lava Charm family adds 420 ticks to the cap
    /// (`:15168-15183`), and while the player stands in lava the timer drains
    /// one tick per frame before the 80-damage hit lands (`:27681-27722`).
    pub lava_time: i32,
    pub lava_max: i32,
    /// `Player.thorns` / `cactusThorns` / `turtleThorns` (`Player.cs:2800-2802`):
    /// the contact reflection — `thorns` is the fraction of the NPC's damage
    /// thrown back (the Thorns potion sets 1, the Dryad's Ward adds a half, the
    /// Turtle set pins 2 through `turtle_thorns`), and `cactus_thorns` is the
    /// Cactus set's flat 15.
    pub thorns: f32,
    pub cactus_thorns: bool,
    pub turtle_thorns: bool,
    /// `Player.skyStoneEffects` (`Player.cs:1120`): the Celestial Stone/Shell
    /// and the day/night Sun/Moon Stones set it, and the one block below adds
    /// the whole stat bundle (`:12988-13003`).
    pub sky_stone_effects: bool,
    /// `Player.endurance` (`Player.cs:1138`): the fraction of incoming damage
    /// the player shrugs off — `Hurt` applies `(int)((1 - endurance) * damage)`
    /// (`:38304`) after the defense reduction. `ResetEffects` zeroes it
    /// (`:18497`); the Worm Scarf, the Turtle/Chlorophyte/Solar sets and the
    /// Endurance/Ice-Barrier buffs add per tick.
    pub endurance: f32,
    pub mana: i32,
    /// `Player.manaCost` (`Player.cs:1884`): the magic items' cost
    /// multiplier — `ResetEffects` sets 1f (`:18334`) and the armor pieces,
    /// the accessory rows, the casters' set bonuses and `BuffID.MagicPower`
    /// subtract from it. `CheckMana` pays `(int)(item.mana * manaCost)`
    /// (`:5881-5884`).
    pub mana_cost: f32,
    pub mana_max: i32,
    /// The 44 buff slots and `buffImmune` — `Player.buffType`/`buffTime`/`buffImmune`
    /// (`Player.cs:1510-1514`), driven by the `Player.AddBuff` family (`buffs.rs`).
    pub buffs: BuffSlots,
    pub hurt_cd: f32,
    /// The dodge-family flags (`Player.blackBelt`/`mysticSashDodge`/
    /// `brainOfConfusionItem`/`shadowDodge`/`longInvince`, `Player.cs:1178-
    /// 1184,2402`): armed per tick in the fold from items 963/984
    /// (`:15060`/`:15064`), 6189 (`:14633`), 3223 (`:15087` — the C# carries
    /// the `Item` reference, the port's bool is the armed-flag equivalent) and
    /// buff 59 (`:10075-10077`); `long_invince` is consume-only here, its
    /// arming sources (6187 et al.) ride C-7. `ResetEffects` clears them every
    /// frame (`:18440/:18617/:18661-18662`/`:18719`) and the `Hurt` roll block
    /// (`:38214-38241`) consumes them.
    pub black_belt: bool,
    pub mystic_sash_dodge: bool,
    pub brain_of_confusion: bool,
    pub shadow_dodge: bool,
    pub long_invince: bool,
    /// `Player.shadowDodgeTimer` (`Player.cs:1184`): the Hallowed set-bonus
    /// cooldown — set 1800 by a shadow dodge (`:38043`), decayed one per tick
    /// in `Player.Update` (`:25461-25464`).
    pub shadow_dodge_timer: i32,
    /// `Player.brainOfConfusionDodgeAnimationCounter` (`Player.cs:1158`): the
    /// 300-tick animation window a brain dodge opens (`:38049`); decayed in
    /// `ResetEffects` (`:18794-18797`), zeroed on death (`UpdateDead`,
    /// `:16932`). No render consumer yet — carried for state parity (the
    /// S1-A-9 dead-slot convention).
    pub brain_of_confusion_dodge_animation_counter: i32,
    /// `Player.creativeGodMode` (`Player.cs:1896`): refreshed every tick from
    /// the journey `GodmodePower` (`:25493-25497`; `ResetEffects` clears it at
    /// `:18789`). `Hurt` (`:38203`), `KillMe` (`:38809`) and
    /// `Update_NPCCollision` (`:31321`) all return early while it is set, and
    /// `Main.UpdateWorld_Players` refills the three pools
    /// (`Main.cs:17869-17873`).
    pub creative_god_mode: bool,
    /// `Player.immune` (`Player.cs:2636`-ish): the spawn i-frames flag —
    /// true while `immuneTime` counts down (`UpdateImmunity`,
    /// `Player.cs:18856-18869`).
    pub immune: bool,
    /// `Player.immuneTime`: 180 on the death revive, 60 on the other spawn
    /// contexts (`Player.cs:37661-37668`).
    pub immune_time: i32,
    /// `Player.immuneNoBlink` (`:37669-37671`): true while the i-frames run
    /// and the player is not hostile (the PVP flag — always false here).
    pub immune_no_blink: bool,
    /// `Player.tileRangeX/Y`'s journey half (`Player.cs:18815-18828`): the
    /// `FarPlacementRangePower` toggle that doubles the default 5/3 range.
    pub far_placement_range: bool,
    /// Remaining breath in ticks (`Player.breath`).
    pub breath: i32,
    breath_cd: i32,
    /// The life/mana regen accumulators (`Player.lifeRegen`/`lifeRegenTime`/
    /// `lifeRegenCount`/`manaRegen*`, `Player.cs:1930-1946`) — `regen.rs`.
    pub regen: RegenState,
    /// The pending DoT death reason: the drain engine's `KillMe` attribution
    /// (`Player.cs:19319-19334,19361-19384`) rides with the player until the
    /// death pipeline (world_sim) resolves the epitaph text and clears it.
    pub dot_death: Option<DotDeath>,
    /// The luck value + its decay timers (`Player.luck`/`coinLuck`/
    /// `ladyBugLuckTimeLeft`/`torchLuck`/`luckPotion`/`kiteLuckLevel`/…,
    /// `Player.cs:3084-3096` etc.) — `luck.rs`, rebuilt every tick by
    /// `RecalculateLuck` (`Player.cs:29137-29164`).
    pub luck_state: LuckState,
    /// `Player.yoyoHitCounter` (`Player.cs:1220`: `= -1`): the lifetime
    /// yoyo-hit counter behind `EverySomeYoyoHits` (`Player.cs:4308-4316`)
    /// — the M4-PROJ-4b on-hit effects (the damage doublings and the
    /// Cascade/Chik/Kraken/Eye-of-Cthulhu spawns). Vanilla never resets it
    /// (no `ResetEffects` arm, no death/throw reset), so the port doesn't
    /// either — the -1 start makes the first hit skip every effect.
    pub yoyo_hit_counter: i32,
    /// The `Player.ZoneSnow`-family flags the torch-luck table reads
    /// (`Player.cs:17924-18036`); the app copies them from `Main.SceneMetrics`
    /// next to the aura snapshot (`UpdateBiomes`, `Player.cs:16726-16729`).
    pub zones: tr_core::scene::Zones,
    /// The held torch's place style — `inventory[selectedItem].createTile == 4`
    /// (`:17917-17920`), refreshed per tick from `app.inventory[app.selected]`;
    /// `None` when nothing torch-like is held.
    pub held_torch_style: Option<u8>,
    /// `Player.fallStart` (`Player.cs:2882`): the tile row the fall started on,
    /// int-truncated like the original's `(int)(position.Y / 16f)`.
    fall_start: i32,
    /// `Player.extraFall` (`Player.cs:2726`): tiles added to the 25-tile safe
    /// fall. Nothing grants it yet (frog-gear sources = M3-EQUIP-3), but the
    /// reset-each-tick shape is the vanilla one.
    extra_fall: i32,
    /// `Player.noFallDmg` (`Player.cs:2766`): the horseshoe/balloon family
    /// (M3-EQUIP-3). Always false until the accessory system lands.
    no_fall_dmg: bool,
    /// `Player.hasWings` (`Player.cs:1336`): any wing equipped (M3-EQUIP-3).
    pub has_wings: bool,
    /// `Player.grappling[0] != -1` (the per-tick rebuilt grapple state): the
    /// hook is attached and `GrappleMovement` drives the velocity.
    pub grappling_active: bool,
    /// `Player.GoingDownWithGrapple` (`Player.cs:808`): the pull points
    /// downward — the platforms fall through.
    pub going_down_with_grapple: bool,
    /// `Player.MountFishronSpecialCounter` (`Player.cs:3004`): the Cute
    /// Fishron's 60/420-tick "special" window (`Mount.cs:5001-5008`), one
    /// tick of decay per update (`Player.cs:25625-25628`). While positive it
    /// forces [`Self::mount_fishron_special`] true.
    mount_fishron_special_counter: f32,
    /// `Player.wingTime` / `wingTimeMax` (`Player.cs:1330,1338`): the flight
    /// budget in ticks and its refill cap.
    pub wing_time: f32,
    pub wing_time_max: i32,
    /// `Player.wingsLogic` (`Player.cs:1334`): the gameplay wing id
    /// (`TR_WINGS=<ArmorIDs.Wing>` pins it; the granting items land with
    /// M3-EQUIP-3).
    pub wings_logic: i32,
    /// `Player.rocketTime` / `rocketTimeMax` / `rocketDelay` / `rocketDelay2`
    /// (`Player.cs:2744-2748`): the rocket-boots fuel and its thrust cadence.
    pub rocket_time: i32,
    rocket_time_max: i32,
    pub rocket_delay: i32,
    rocket_delay2: i32,
    /// `Player.rocketBoots` (`Player.cs:2756`): the boots tier
    /// (`TR_BOOTS=<1..5>` pins it).
    pub rocket_boots: i32,
    /// `Player.canRocket` / `rocketRelease` (`Player.cs:2762,2752`): the
    /// midair arming and the release-jump latch the trigger reads.
    can_rocket: bool,
    rocket_release: bool,
    /// `Player.releaseJump` (`Player.cs`): jump was released since the last
    /// press — the wing/rocket refills on the ground gate on it.
    release_jump: bool,
    /// `Player.tryKeepingHoveringDown/Up` (`Player.cs:1818-1820`): the
    /// press-down hover latches.
    try_keeping_hovering_down: bool,
    try_keeping_hovering_up: bool,
    down_prev: bool,
    /// `Player.hasJumpOption_*` / `canJumpAgain_*` / `isPerformingJump_*`
    /// (`Player.cs:2664-2716`), indexed by [`ExtraJump::index`]. The has
    /// flags re-derive per tick from the equipment; the can flags re-arm on
    /// the ground (`RefreshDoubleJumps`, `Player.cs:22869-22908`).
    pub has_jump_options: [bool; 9],
    pub can_jump_again: [bool; 9],
    performing_jump: Option<ExtraJump>,
    /// `Player.jumpBoost` (`Player.cs:2764`): the balloon family's stat
    /// floor (`UpdateJumpHeight`, `Player.cs:19589-19593`).
    pub jump_boost: bool,
    /// `Player.jumpHeight` / `Player.jumpSpeed` (`Player.cs:24366-24367`):
    /// the per-tick statics the jump branches read.
    jump_height: i32,
    jump_speed: f32,
    /// `Player.mount` (`Player.cs:2126`): the mount instance state — the data
    /// table and the state core live in the `mount` module.
    pub mount: crate::mount::MountState,
    /// `Player.autoJump`: the vanilla auto-jump accessibility setting; the
    /// port has no settings surface, so this carries only the mount's
    /// per-tick head-set (`Mount.UpdateEffects`, `Mount.cs:4797`).
    auto_jump: bool,
    /// `Player.justJumped` (`Player.cs:2722`): set on a grounded jump trigger
    /// (`:20678`), cleared in `ResetEffects` (`:18786`); the wing/rocket
    /// refills read it for the auto-jump (constant-jump) mounts.
    just_jumped: bool,
    /// The live hitbox height — `Player.height`: `HEIGHT` (42) normally,
    /// `42 + heightBoost` while mounted (`SetMount` `Mount.cs:6264-6277`).
    pub height: f32,
    /// `Player.lastStoned` (`Player.cs`): the petrify-transition detector.
    last_stoned: bool,
    /// `Player.dashType` (`Player.cs:1044`): the equipped dash capability.
    /// The granters (Tabi 977, EoCShield 3097, Master Ninja Gear 984) land
    /// with M3-EQUIP-3, the Solar set and the Chillet mounts with their own
    /// tasks — `TR_DASH=<1|2|3|6>` pins it for now.
    pub dash_type: i32,
    /// `Player.dash` (`:1046`): the committed dash type of the lifecycle.
    pub dash: i32,
    /// `Player.dashTime` (`:1048`): the signed 15-tick double-tap window.
    dash_time: i32,
    /// `Player.dashDelay` (`:1052`): 0 ready, -1 dashing, >0 cooldown.
    pub dash_delay: i32,
    /// `Player.eocDash` / `eocHit` (`:1054,1056`): the Shield of Cthulhu
    /// post-hit grace (the struck-NPC exclusion + the drain on cooldown).
    pub eoc_dash: i32,
    eoc_hit: i32,
    /// The previous-tick direction keys — the press edges the double-tap
    /// reads (`Player.cs:27618-27635` recomputes the release flags at the
    /// end of the tick).
    right_prev: bool,
    left_prev: bool,
    /// `Player.maxRunSpeed` / `runAcceleration` / `runSlowdown` /
    /// `accRunSpeed` (`Player.cs:24364-24371`): the per-tick horizontal
    /// defaults the `HorizontalMovement` branches and the floor modifiers ride.
    max_run_speed: f32,
    run_acceleration: f32,
    run_slowdown: f32,
    acc_run_speed: f32,
    /// `Player.moveSpeed` / `strongestMoveSpeedDebuff` (`Player.cs:2464,2462`):
    /// 1.0 until accessories/debuffs land (M3-EQUIP-3, M2-BUFF-4).
    pub(crate) move_speed: f32,
    strongest_move_speed_debuff: f32,
    /// `Player.iceSkate` / `desertBoots` (`Player.cs:1088,2256`): Ice Skates /
    /// Dunerider Boots (M3-EQUIP-3 — no accessory system yet).
    ice_skate: bool,
    desert_boots: bool,
    /// The `FloorVisuals` flags (`Player.cs:23708-23743`), sampled after the
    /// tick's movement — one tick stale relative to the movement block,
    /// exactly like the original's call order (`:26500` vs `:28271`).
    sticky: bool,
    slippy: bool,
    slippy2: bool,
    powerrun: bool,
    running_on_sand: bool,
    /// The liquid displacement fraction `Player.WetCollision` integrates free
    /// axes with (`Player.cs:23436-23451`): water/lava 0.5, honey 0.25
    /// (`Player.cs:28168-28195`), dry 1.0.
    liquid_move_speed: f32,
    pub in_lava: bool,
    /// `Player.gfxOffY`: visual offset that hides the instant lift of `StepUp` /
    /// `StepDown`; eases back to zero so steps and slopes look smooth.
    pub gfx_off_y: f32,
    prev_gfx_off_y: f32,
    /// `Player.stepSpeed`: how fast `gfxOffY` eases out.
    step_speed: f32,
    /// `Player.stairFall`: slopes may be fallen through this tick.
    stair_fall: bool,
    /// `Player.mount.Active && Mount.Minecart` — the base-cart ride state
    /// (`MountID.Sets.Cart` contains 6, `MountID.cs:43`). The full Mount
    /// framework is M2-MOUNT-1; `TR_CART` pins this state for tests and
    /// screenshots.
    pub cart_mounted: bool,
    /// `Player.cartFlip` (`Player.cs:1786`): the bouncy-bumper sprite flip,
    /// toggled on each bounce while a direction key is held
    /// (`Player.cs:28123-28130`).
    pub cart_flip: bool,
    /// `Player.onTrack` (`Player.cs:2811`): this tick's `Minecart.TrackCollision`
    /// result bit 0 — the body rides a rail. Couples the collision heights
    /// (pitfall §10.9).
    pub on_track: bool,
    /// `Player.onWrongGround` (`Player.cs:2455`): a mounted cart off the rails —
    /// set by the on-track check (`Player.cs:26665-26674`), preserved across the
    /// tick for carts (`:24371-24374`).
    pub on_wrong_ground: bool,
    /// `Player.trackBoost` (`Player.cs:1780`): the pending booster push, applied
    /// pre-input next tick and clamped (`Player.cs:19818-19833`).
    pub track_boost: f32,
    /// `Player.lastBoost` (`Player.cs:1782`): the booster tile a boost is pending
    /// from — tile coords, `(0, 0)` = none (`Minecart.cs:832-847`, `:894-897`).
    pub last_boost: (i32, i32),
    /// `Player.cartRampTime` (`Player.cs:1802`): the gravity-freeze ticks a ramp
    /// launch grants (`Player.cs:28148-28150`), consumed by the gravity gate
    /// (`:27352-27359`).
    pub cart_ramp_time: i32,
    /// `Player.fullRotation` / `fullRotationOrigin` (`Player.cs:1838-1840`): the
    /// rail-slope tilt from `Minecart.TrackRotation` (`Player.cs:28114-28115`).
    pub full_rotation: f32,
    pub full_rotation_origin: (f32, f32),
    /// `Player.fallStart2` (`Player.cs:2884`): the fall origin the rail-landing
    /// gate reads (`Minecart.cs:809-813`), captured pre-movement when
    /// `velocity.Y <= 0` (`Player.cs:25295-25297`).
    pub fall_start2: i32,
    /// The `Player.cs:28237-28241` switch trip handed to the caller (the C#
    /// `HitTrackSwitch` needs `&mut World`; the tick takes `&World`).
    pending_switch_trip: Option<(i32, i32)>,
    /// The wheel-spark rolls' named stream (the per-consumer convention — C#
    /// rolls the shared `Main.rand` at `Player.cs:19873-19901` and inside
    /// `DelegateMethods.Minecart.Sparks`). Seeded from the world seed.
    pub spark_rng: u64,
    /// The tick's queued spark particles, drained by `world_sim` into the
    /// App's `minecart::Sparks` pool (the `Main.dust` stand-in).
    pending_sparks: Vec<crate::minecart::SparkP>,
    /// The tick's queued minecart cues (Item53/55/56), drained into
    /// `App::audio::play_at` (`DelegateMethods.cs:248-256`, `Player.cs:19870`).
    pending_cart_sounds: Vec<crate::minecart::CartCue>,
    /// `Player.channel` (`Player.cs:52958-52968`): a `channel` item is held
    /// and aiming — set by `StartChanneling`, cleared by `TryCancelChannel`
    /// (`:52975-52981`) and on the release. The aiStyle-9 magic spells (Magic
    /// Missile / Flamelash / Rainbow Rod) read it while aiming
    /// (`Projectile.cs`'s `AI_009_MagicMissiles` owner block).
    /// Written by the item-use path (M4-PROJ-6 Phase 5), read by the arm
    /// (Phase 4); default off.
    pub channel: bool,
    /// The shooting item's kind while `channel` — the C# block only holds the
    /// aim while `HeldItem.shoot == type` (`AI_009_MagicMissiles`'s owner
    /// block, `Projectile.cs:43239-43249`); 0 = no item.
    pub channel_item: u16,
    /// The aim target the channel feeds the spell: `MouseWorld` clamped by
    /// `LimitPointToPlayerReachableArea` (`Projectile.cs:43241-43249`'s
    /// `ai[0..1] = LimitPointToPlayerReachableArea(MouseWorld)`), in world
    /// pixels. `Vec2::ZERO` while not channelling.
    pub channel_target: Vec2,
    // ------------------------------- the ammo mastery layer (M3-ITEM-6)
    // `Player.magicQuiver` (`Player.cs:1968`): the quivers 4002/4006/6181/1321
    // (`:14465`, `:14483`, `:14596`, `:15333`) — the 1/5 roll arm (`:53595`) and
    // the ×1.1 speed fuse (`:53543`). Cleared and re-derived per tick by the
    // grant seam in `world_sim/player.rs` (`ResetEffects` `:18605`).
    pub magic_quiver: bool,
    /// `Player.hasMoltenQuiver` (`Player.cs:1974`): 4002/6181 (`:14467`,
    /// `:14596`) — converts the wooden arrow to the FireArrow and adds 2 ammo
    /// damage (`:53537-53541`).
    pub has_molten_quiver: bool,
    /// `Player.accSharpBarb` (`Player.cs:1224`): 6160 Sharp Barb (`:14501`) —
    /// +1 ammo damage on arrows (`:53548`).
    pub acc_sharp_barb: bool,
    /// `Player.ammoBox` (`Player.cs:1982`): buff 93, the Ammo Box (`:10080`) —
    /// the 1/5 roll arm on ANY ammo (`:53599`).
    pub ammo_box: bool,
    /// `Player.ammoPotion` (`Player.cs:1984`): buff 112, the Ammo Reservation
    /// (`:10195`) — the 1/5 roll arm on ANY ammo (`:53603`).
    pub ammo_potion: bool,
    /// `Player.huntressAmmoCost90` (`Player.cs:1960`): 3804/4983 (`:13179`,
    /// `:13634`) — the 1/10 roll arm (`:53643`).
    pub huntress_ammo_cost90: bool,
    /// `Player.chloroAmmoCost80` (`Player.cs:1958`): 1002 (`:13649`) — the 1/5
    /// roll arm (`:53647`).
    pub chloro_ammo_cost80: bool,
    /// `Player.ammoCost80` (`Player.cs:1962`): 1549/3878 (`:13715`, `:13222`) —
    /// the 1/5 roll arm (`:53651`).
    pub ammo_cost80: bool,
    /// `Player.ammoCost75` (`Player.cs:1964`): 2758 (`:13908`) — the 1/4 roll
    /// arm (`:53655`).
    pub ammo_cost75: bool,
    /// `Player.archery` (`Player.cs:2286`): buff 16, the Archery Potion
    /// (`:10018`) — the ×1.2 speed fuse capped at 20 (`:53560`).
    pub archery: bool,
    // `Player.ammoCyclingMode`/`ammoCyclingOffset`/`ammoCyclingCooldown`
    // (`Player.cs:1244`, `:1248`, `:1246`): the cycling state.
    pub ammo_cycling: crate::ammo::AmmoCycling,
    // `Player.ammoCyclingMode`/`ammoCyclingOffset`/`ammoCyclingCooldown`
    // (`Player.cs:1244`, `:1248`, `:1246`): the cycling state — the mode is
    // granted per tick by the accessories 6168/6169/6193/6194 (`:14528-14544`),
    // the offset/cooldown are written inside a successful pick (`:53569-53577`)
}

/// One summon keep-alive arm of `Player.UpdateBuffs`' chain
/// (`Player.cs:10483-10947`): the buff id, the projectile kinds whose live
/// presence re-affirms the buff, and the `ResetEffects` flag the arm sets
/// (`Player.cs:18620-18646`+`:18741` clear, the arm sets true).
struct SummonBuffArm {
    buff: i32,
    kinds: &'static [u16],
    flag: fn(&mut Player) -> &mut bool,
}

fn flag_pygmy(p: &mut Player) -> &mut bool {
    &mut p.pygmy
}
fn flag_raven(p: &mut Player) -> &mut bool {
    &mut p.raven
}
fn flag_slime(p: &mut Player) -> &mut bool {
    &mut p.slime
}
fn flag_hornet_minion(p: &mut Player) -> &mut bool {
    &mut p.hornet_minion
}
fn flag_imp_minion(p: &mut Player) -> &mut bool {
    &mut p.imp_minion
}
fn flag_spider_minion(p: &mut Player) -> &mut bool {
    &mut p.spider_minion
}
fn flag_twins_minion(p: &mut Player) -> &mut bool {
    &mut p.twins_minion
}
fn flag_pirate_minion(p: &mut Player) -> &mut bool {
    &mut p.pirate_minion
}
fn flag_vampire_frog(p: &mut Player) -> &mut bool {
    &mut p.vampire_frog
}
fn flag_sharknado_minion(p: &mut Player) -> &mut bool {
    &mut p.sharknado_minion
}
fn flag_ufo_minion(p: &mut Player) -> &mut bool {
    &mut p.ufo_minion
}
fn flag_stardust_minion(p: &mut Player) -> &mut bool {
    &mut p.stardust_minion
}
fn flag_bats_of_light(p: &mut Player) -> &mut bool {
    &mut p.bats_of_light
}
fn flag_baby_bird(p: &mut Player) -> &mut bool {
    &mut p.baby_bird
}
fn flag_dead_cells_mushroom_boi_minion(p: &mut Player) -> &mut bool {
    &mut p.dead_cells_mushroom_boi_minion
}
fn flag_palworld_cattiva_minion(p: &mut Player) -> &mut bool {
    &mut p.palworld_cattiva_minion
}
fn flag_palworld_foxsparks_minion(p: &mut Player) -> &mut bool {
    &mut p.palworld_foxsparks_minion
}
fn flag_palworld_trusty_cattiva_minion(p: &mut Player) -> &mut bool {
    &mut p.palworld_trusty_cattiva_minion
}
fn flag_palworld_trusty_foxsparks_minion(p: &mut Player) -> &mut bool {
    &mut p.palworld_trusty_foxsparks_minion
}
fn flag_clay_pot_minion(p: &mut Player) -> &mut bool {
    &mut p.clay_pot_minion
}
fn flag_forbidden_minion(p: &mut Player) -> &mut bool {
    &mut p.forbidden_minion
}
fn flag_flinx_minion(p: &mut Player) -> &mut bool {
    &mut p.flinx_minion
}
fn flag_abigail_minion(p: &mut Player) -> &mut bool {
    &mut p.abigail_minion
}
fn flag_storm_tiger(p: &mut Player) -> &mut bool {
    &mut p.storm_tiger
}
fn flag_smolstar(p: &mut Player) -> &mut bool {
    &mut p.smolstar
}
fn flag_empress_blade(p: &mut Player) -> &mut bool {
    &mut p.empress_blade
}
fn flag_stardust_guardian(p: &mut Player) -> &mut bool {
    &mut p.stardust_guardian
}
fn flag_stardust_dragon(p: &mut Player) -> &mut bool {
    &mut p.stardust_dragon
}
fn flag_deadly_sphere_minion(p: &mut Player) -> &mut bool {
    &mut p.deadly_sphere_minion
}

/// The summon keep-alive arms in the C# chain's verbatim order
/// (`Player.cs:10483-10972`). Buff 216 carries its own respawn arm and buffs
/// 335/263 their status updaters — the driver special-cases those ids.
const SUMMON_BUFF_ARMS: [SummonBuffArm; 29] = [
    SummonBuffArm {
        buff: 49,
        kinds: &[191, 192, 193, 194],
        flag: flag_pygmy,
    }, // :10483
    SummonBuffArm {
        buff: 83,
        kinds: &[317],
        flag: flag_raven,
    }, // :10505
    SummonBuffArm {
        buff: 64,
        kinds: &[266],
        flag: flag_slime,
    }, // :10521
    SummonBuffArm {
        buff: 125,
        kinds: &[373],
        flag: flag_hornet_minion,
    }, // :10537
    SummonBuffArm {
        buff: 126,
        kinds: &[375],
        flag: flag_imp_minion,
    }, // :10553
    SummonBuffArm {
        buff: 133,
        kinds: &[390, 391, 392],
        flag: flag_spider_minion,
    }, // :10569
    SummonBuffArm {
        buff: 134,
        kinds: &[387, 388],
        flag: flag_twins_minion,
    }, // :10585
    SummonBuffArm {
        buff: 135,
        kinds: &[393, 394, 395],
        flag: flag_pirate_minion,
    }, // :10601
    SummonBuffArm {
        buff: 214,
        kinds: &[758],
        flag: flag_vampire_frog,
    }, // :10617
    SummonBuffArm {
        buff: 139,
        kinds: &[407],
        flag: flag_sharknado_minion,
    }, // :10633
    SummonBuffArm {
        buff: 140,
        kinds: &[423],
        flag: flag_ufo_minion,
    }, // :10649
    SummonBuffArm {
        buff: 182,
        kinds: &[613],
        flag: flag_stardust_minion,
    }, // :10665
    SummonBuffArm {
        buff: 213,
        kinds: &[755],
        flag: flag_bats_of_light,
    }, // :10681
    SummonBuffArm {
        buff: 216,
        kinds: &[759],
        flag: flag_baby_bird,
    }, // :10697 (+respawn)
    SummonBuffArm {
        buff: 355,
        kinds: &[1022],
        flag: flag_dead_cells_mushroom_boi_minion,
    }, // :10729
    SummonBuffArm {
        buff: 385,
        kinds: &[1093],
        flag: flag_palworld_cattiva_minion,
    }, // :10745
    SummonBuffArm {
        buff: 393,
        kinds: &[1118],
        flag: flag_clay_pot_minion,
    }, // :10761
    SummonBuffArm {
        buff: 394,
        kinds: &[1119],
        flag: flag_forbidden_minion,
    }, // :10777
    SummonBuffArm {
        buff: 386,
        kinds: &[1094],
        flag: flag_palworld_foxsparks_minion,
    }, // :10793
    SummonBuffArm {
        buff: 389,
        kinds: &[1112],
        flag: flag_palworld_trusty_cattiva_minion,
    }, // :10809
    SummonBuffArm {
        buff: 390,
        kinds: &[1113],
        flag: flag_palworld_trusty_foxsparks_minion,
    }, // :10825
    SummonBuffArm {
        buff: 325,
        kinds: &[951],
        flag: flag_flinx_minion,
    }, // :10841
    SummonBuffArm {
        buff: 335,
        kinds: &[970],
        flag: flag_abigail_minion,
    }, // :10857 (+status)
    SummonBuffArm {
        buff: 263,
        kinds: &[831],
        flag: flag_storm_tiger,
    }, // :10877 (+status)
    SummonBuffArm {
        buff: 271,
        kinds: &[864],
        flag: flag_smolstar,
    }, // :10897
    SummonBuffArm {
        buff: 322,
        kinds: &[946],
        flag: flag_empress_blade,
    }, // :10912
    SummonBuffArm {
        buff: 187,
        kinds: &[623],
        flag: flag_stardust_guardian,
    }, // :10928
    SummonBuffArm {
        buff: 188,
        kinds: &[625],
        flag: flag_stardust_dragon,
    }, // :10944
    SummonBuffArm {
        buff: 161,
        kinds: &[533],
        flag: flag_deadly_sphere_minion,
    }, // :10957
];

impl Player {
    /// `Player.EverySomeYoyoHits` (`Player.cs:4308-4316`): the counter's
    /// modulo against `amount` (C#'s truncated remainder, so the -1 start
    /// skips the very first hit) plus the optional bump. The one caller
    /// pair is `Projectile`'s on-hit arms: the damage doublings peek
    /// (`countUp = false`) before the strike, the spawns increment after
    /// it — both on the same hit, in that order.
    pub fn every_some_yoyo_hits(&mut self, amount: i32, count_up: bool) -> bool {
        let hit = self.yoyo_hit_counter % amount == 0;
        if count_up {
            self.yoyo_hit_counter += 1;
        }
        hit
    }

    pub fn spawn(world: &World) -> Self {
        // `FindSpawn` (`Player.cs:54592-54609`): the bed spawn — the port's
        // `World::bed_spawn` — overrides the world spawn when it validates
        // through the same `CheckSpawn` the revive path runs
        // (`Player.cs:37571-37575`). The load-time fallback is silent (the
        // failure chat needs the monitor, which the join path has not built
        // yet); the revive prints like the C#.
        let (sx, sy) = match world.bed_spawn {
            Some((bx, by)) if crate::world_sim::editing::bed_spawn_valid(world, bx, by).is_ok() => {
                // `Spawn_SetPosition` (`Player.cs:37987-37991`): the feet at
                // the spawn cell's top.
                (
                    bx as f32 * TILE_PX + 8.0 - WIDTH * 0.5,
                    by as f32 * TILE_PX - HEIGHT,
                )
            }
            _ => (
                world.spawn_x as f32 * TILE_PX + 8.0 - WIDTH * 0.5,
                world.spawn_y as f32 * TILE_PX,
            ),
        };
        Self {
            dead: false,
            deaths_pve: 0,
            angler_quests_finished: 0,
            golfer_score: 0,
            deaths_pvp: 0,
            hostile: false,
            team: 0,
            hide_visible_accessory: [false; 10],
            dead_time: 0,
            respawn_timer: 0,
            immune: false,
            immune_time: 0,
            immune_no_blink: false,
            minion_attack_target_npc: -1,
            tag_effects: crate::tag_effects::TagEffectStack::default(),
            prev_jump_input: false,
            auras: tr_core::scene::Auras::default(),
            mana_regen_bonus: 0,
            stat_defense: 0,
            well_fed: false,
            pick_speed_bonus: 0.0,
            melee_speed_bonus: 0.0,
            slow: false,
            chilled: false,
            frozen: false,
            webbed: false,
            blind: false,
            weak_stacks: 0,
            melee_crit: 4,
            ranged_crit: 4,
            magic_crit: 4,
            melee_damage: 1.0,
            ranged_damage: 1.0,
            magic_damage: 1.0,
            minion_damage: 1.0,
            ranged_mult_damage: 1.0,
            arrow_damage: 1.0,
            arrow_damage_additive_stack: 0.0,
            bullet_damage: 1.0,
            rocket_damage: 1.0,
            max_minions: 1,
            num_minions: 0,
            slots_minions: 0.0,
            max_turrets: 1,
            max_turrets_old: 1,
            owned_projectile_counts: [0; PROJ_COUNT],
            highest_storm_tiger_gem_original_damage: 0,
            highest_abigail_counter_original_damage: 0,
            pygmy: false,
            raven: false,
            slime: false,
            hornet_minion: false,
            imp_minion: false,
            spider_minion: false,
            twins_minion: false,
            pirate_minion: false,
            vampire_frog: false,
            sharknado_minion: false,
            ufo_minion: false,
            stardust_minion: false,
            bats_of_light: false,
            baby_bird: false,
            dead_cells_mushroom_boi_minion: false,
            palworld_cattiva_minion: false,
            palworld_foxsparks_minion: false,
            palworld_trusty_cattiva_minion: false,
            palworld_trusty_foxsparks_minion: false,
            clay_pot_minion: false,
            forbidden_minion: false,
            flinx_minion: false,
            abigail_minion: false,
            storm_tiger: false,
            smolstar: false,
            empress_blade: false,
            stardust_guardian: false,
            stardust_dragon: false,
            deadly_sphere_minion: false,
            x: sx,
            y: sy,
            prev_x: sx,
            prev_y: sy,
            vx: 0.0,
            vy: 0.0,
            on_ground: false,
            facing: 1,
            leg_frame: 0,
            body_frame: 0,
            leg_frame_counter: 0.0,
            body_frame_counter: 0.0,
            swim_time: 0,
            misc_counter: 0,
            pulley: false,
            pulley_dir: 0,
            shield_raised: false,
            sliding: false,
            sand_storm: false,
            carpet_frame: -1,
            legs_armor: 0,
            frame_inputs: FrameInputs::default(),
            wet: false,
            lava_wet: false,
            honey_wet: false,
            shimmer_wet: false,
            liquid_step: 1.0,
            in_water: false,
            sitting: false,
            sleeping: false,
            sleep_time: 0,
            sleeping_style: 0,
            offset_for_seat: (0.0, 0.0),
            seat_pos_offset: (0.0, 0.0),
            seat_y_offset: 0,
            no_knockback: false,
            // M5-SEED-2: `Main.vampireSeed` (`WorldGen.cs:555-558`) — the
            // OnFire regen row's source (`Player.cs:18983-18992`).
            vampirism: world
                .micro_seeds
                .contains(tr_core::seeds::MicroSeedFlags::VAMPIRISM),
            accessory_life_regen: 0,
            swing_aim: 0.0,
            equipped: [0; 20],
            dye_slots: [0; 10],
            misc_dye_slots: [0; 5],
            c_head: 0,
            c_body: 0,
            c_legs: 0,
            c_pet: 0,
            c_light: 0,
            c_minecart: 0,
            c_mount: 0,
            c_grapple: 0,
            equipped_prefix: [0; 20],
            banks: std::array::from_fn(|_| std::array::from_fn(|_| crate::item::EMPTY_SLOT)),
            chest: crate::bank::CHEST_CLOSED,
            chest_x: 0,
            chest_y: 0,
            void_vault_enabled: false,
            pinned_jumps: [false; 9],
            pinned_balloon: false,
            pinned_rocket_boots: 0,
            pinned_mount: None,
            control_left: false,
            control_right: false,
            control_jump: false,
            control_down: false,
            jump: 0,
            jump_held: false,
            spawn_x: sx,
            spawn_y: sy,
            hp: 100,
            max_hp: 100,
            armor_penetration: 0,
            melee_armor_penetration: 0,
            lava_time: 0,
            lava_max: 0,
            thorns: 0.0,
            cactus_thorns: false,
            turtle_thorns: false,
            sky_stone_effects: false,
            endurance: 0.0,
            mana: 20,
            mana_cost: 1.0,
            mana_max: 20,
            buffs: BuffSlots::default(),
            hurt_cd: 0.0,
            black_belt: false,
            mystic_sash_dodge: false,
            brain_of_confusion: false,
            shadow_dodge: false,
            long_invince: false,
            shadow_dodge_timer: 0,
            brain_of_confusion_dodge_animation_counter: 0,
            breath: BREATH_MAX,
            breath_cd: 0,
            creative_god_mode: false,
            far_placement_range: false,
            regen: RegenState::default(),
            dot_death: None,
            luck_state: LuckState::default(),
            zones: tr_core::scene::Zones::default(),
            held_torch_style: None,
            fall_start: (sy / TILE_PX) as i32,
            extra_fall: 0,
            no_fall_dmg: false,
            has_wings: false,
            grappling_active: false,
            going_down_with_grapple: false,
            mount_fishron_special_counter: 0.0,
            wing_time: 0.0,
            wing_time_max: 0,
            wings_logic: 0,
            rocket_time: 0,
            rocket_time_max: 7,
            rocket_delay: 0,
            rocket_delay2: 0,
            rocket_boots: 0,
            can_rocket: false,
            rocket_release: false,
            release_jump: false,
            try_keeping_hovering_down: false,
            try_keeping_hovering_up: false,
            down_prev: false,
            has_jump_options: [false; 9],
            can_jump_again: [false; 9],
            performing_jump: None,
            jump_boost: false,
            jump_height: 15,
            jump_speed: 5.01,
            mount: crate::mount::MountState::default(),
            auto_jump: false,
            just_jumped: false,
            height: HEIGHT,
            last_stoned: false,
            dash_type: 0,
            dash: 0,
            dash_time: 0,
            dash_delay: 0,
            eoc_dash: 0,
            eoc_hit: -1,
            right_prev: false,
            left_prev: false,
            max_run_speed: 3.0,
            run_acceleration: 0.08,
            run_slowdown: 0.2,
            acc_run_speed: 3.0,
            move_speed: 1.0,
            strongest_move_speed_debuff: 1.0,
            ice_skate: false,
            desert_boots: false,
            sticky: false,
            slippy: false,
            slippy2: false,
            powerrun: false,
            running_on_sand: false,
            liquid_move_speed: 1.0,
            in_lava: false,
            gfx_off_y: 0.0,
            prev_gfx_off_y: 0.0,
            step_speed: 1.0,
            stair_fall: false,
            cart_mounted: false,
            cart_flip: false,
            on_track: false,
            on_wrong_ground: false,
            track_boost: 0.0,
            last_boost: (0, 0),
            cart_ramp_time: 0,
            full_rotation: 0.0,
            full_rotation_origin: (0.0, 0.0),
            fall_start2: (sy / TILE_PX) as i32,
            pending_switch_trip: None,
            spark_rng: world.seed ^ 0x0CA4_75B1_A5E5_DD1D,
            pending_sparks: Vec::new(),
            pending_cart_sounds: Vec::new(),
            // `Player.channel` is off until a `channel` item is held
            // (`Player.cs:52958-52968`).
            channel: false,
            channel_item: 0,
            channel_target: Vec2::ZERO,
            magic_quiver: false,
            has_molten_quiver: false,
            acc_sharp_barb: false,
            ammo_box: false,
            ammo_potion: false,
            huntress_ammo_cost90: false,
            chloro_ammo_cost80: false,
            ammo_cost80: false,
            ammo_cost75: false,
            archery: false,
            ammo_cycling: crate::ammo::AmmoCycling::default(),
            yoyo_hit_counter: -1,
        }
    }

    /// `Collision.SolidCollision` (`Collision.cs:2700-2734`): any active
    /// solid non-solidTop tile overlapping the box — the `-1/+2` sample
    /// expansion, the half-brick top cut and the C#'s `maxTilesY - 40`
    /// bottom clamp included.
    pub(crate) fn solid_collision(world: &World, x: f32, y: f32, w: f32, h: f32) -> bool {
        let value = (x / 16.0) as i32 - 1;
        let value2 = ((x + w) / 16.0) as i32 + 2;
        let value3 = (y / 16.0) as i32 - 1;
        let value4 = ((y + h) / 16.0) as i32 + 2;
        let num = value.clamp(0, world.width as i32 - 1);
        let value2 = value2.clamp(0, world.width as i32 - 1);
        let value3 = value3.clamp(0, world.height as i32 - 40);
        let value4 = value4.clamp(0, world.height as i32 - 40);
        for i in num..value2 {
            for j in value3..value4 {
                let Some(t) = world.tile(i, j) else {
                    continue;
                };
                if !t.active()
                    || !tr_core::tiledata::TILE_SOLID.has(t.kind)
                    || tr_core::tiledata::TILE_SOLID_TOP.has(t.kind)
                {
                    continue;
                }
                let vx = i as f32 * 16.0;
                let mut vy = j as f32 * 16.0;
                let mut vh = 16.0f32;
                if t.half_brick() {
                    vy += 8.0;
                    vh -= 8.0;
                }
                if x + w > vx && x < vx + 16.0 && y + h > vy && y < vy + vh {
                    return true;
                }
            }
        }
        false
    }

    /// `Player.CanSnapToPosition` (`Player.cs:33401-33417`): the snap box
    /// (`position + offset`, the live 20x42) must be free of solid tiles.
    /// The mount-size override branch is unreachable for the port's mounts.
    pub fn can_snap_to(&self, world: &World, offset_x: f32, offset_y: f32) -> bool {
        !Player::solid_collision(
            world,
            self.x + offset_x,
            self.y + offset_y,
            WIDTH,
            self.height,
        )
    }

    /// `PlayerSittingHelper.SitDown` (`PlayerSittingHelper.cs:104-142`): the
    /// snap box is validated (`CanSnapToPosition`, `:110`), a repeat click
    /// at the same seat stands up (`:119-123`), then the bottom snaps to
    /// the seat, the facing flips to the seat's, the velocity zeroes and
    /// the seat offset stores (`:130-137`). The grapple/mount teardown
    /// stands in for `RemoveAllGrapplingHooks`/`mount.TryDismount`.
    pub fn sit_down(
        &mut self,
        world: &World,
        seat_x: f32,
        seat_y: f32,
        direction: i32,
        seat_offset: (f32, f32),
    ) {
        let bottom_x = self.x + WIDTH as f32 * 0.5;
        let bottom_y = self.y + self.height;
        if !self.can_snap_to(world, seat_x - bottom_x, seat_y - bottom_y) {
            return;
        }
        if self.sitting && (bottom_x - seat_x).abs() < 0.01 && (bottom_y - seat_y).abs() < 0.01 {
            self.sit_up();
            return;
        }
        self.grappling_active = false;
        self.x = seat_x - WIDTH as f32 * 0.5;
        self.y = seat_y - self.height;
        self.facing = direction;
        self.sitting = true;
        self.offset_for_seat = seat_offset;
        // `GetSittingOffsetInfo` (`:22-25`) with `sittingIndex` 0 (SP):
        // `posOffset = offsetForSeat * (direction, gravDir)`,
        // `seatAdjustment = -4 + (int)offsetForSeat.Y`.
        self.seat_pos_offset = (seat_offset.0 * direction as f32, seat_offset.1);
        self.seat_y_offset = -4 + seat_offset.1 as i32;
        self.vx = 0.0;
        self.vy = 0.0;
    }

    /// `PlayerSittingHelper.SitUp` (`:89-102`).
    pub fn sit_up(&mut self) {
        self.sitting = false;
        self.offset_for_seat = (0.0, 0.0);
        self.seat_pos_offset = (0.0, 0.0);
        self.seat_y_offset = 0;
    }

    /// `PlayerSleepingHelper.StartSleeping` (`PlayerSleepingHelper.cs:155-190`):
    /// the snap box is validated (`:158`), a repeat click at the same anchor
    /// wakes up (`:167-171`), the vanity/teardown strip runs (`:172-178`),
    /// the bottom snaps to the bed anchor, the facing flips to the bed's and
    /// the pose rotation arms (`:184` → `SetIsSleepingAndAdjustPlayerRotation`,
    /// `:66-83`: `fullRotation = PI/2 * (-direction)`, origin `Size / 2`).
    pub fn sleep_down(
        &mut self,
        world: &World,
        anchor_x: f32,
        anchor_y: f32,
        direction: i32,
        bed_style: u32,
    ) {
        let bottom_x = self.x + WIDTH as f32 * 0.5;
        let bottom_y = self.y + self.height;
        if !self.can_snap_to(world, anchor_x - bottom_x, anchor_y - bottom_y) {
            return;
        }
        if self.sleeping && (bottom_x - anchor_x).abs() < 0.01 && (bottom_y - anchor_y).abs() < 0.01
        {
            self.stop_sleeping();
            return;
        }
        // `player.StopVanityActions()` (`:172`): SitUp + StopSleeping (+ the
        // petting stop — not ported).
        self.sit_up();
        self.stop_sleeping();
        self.grappling_active = false;
        // `RemoveAllFishingBobbers` — no bobbers in the port yet;
        // `mount.TryDismount` (`:175-178`).
        if self.cart_mounted || self.mount.is_active() {
            let _ = self.dismount(world, false);
        }
        self.x = anchor_x - WIDTH as f32 * 0.5;
        self.y = anchor_y - self.height;
        self.facing = direction;
        self.sleeping = true;
        self.sleeping_style = bed_style;
        self.vx = 0.0;
        self.vy = 0.0;
        self.full_rotation = std::f32::consts::FRAC_PI_2 * -(direction as f32);
        self.full_rotation_origin = (WIDTH / 2.0, HEIGHT / 2.0);
    }

    /// `PlayerSleepingHelper.StopSleeping` (`:140-153`) +
    /// `SetIsSleepingAndAdjustPlayerRotation(false)` (`:76-81`).
    pub fn stop_sleeping(&mut self) {
        if self.sleeping {
            self.sleeping = false;
            self.full_rotation = 0.0;
            self.full_rotation_origin = (0.0, 0.0);
            self.sleep_time = 0;
        }
    }

    /// `PlayerSleepingHelper.UpdateState` (`PlayerSleepingHelper.cs:85-138`).
    /// `item_in_use` is the caller's `player.itemAnimation > 0`; the
    /// `danger_near` gate rides the M2-DEATH-2 stand-in (false — the port
    /// has no `DangerThatPreventsOtherDangers` members), and the eclipse
    /// arm (`:55-58`) waits for the hardmode state.
    pub fn update_sleeping(&mut self, world: &World, item_in_use: bool, danger_near: bool) {
        if !self.sleeping {
            self.sleep_time = 0;
            return;
        }
        self.sleep_time += 1;
        // `DoesPlayerHaveReasonToActUpInBed` (`:45-64`): the danger/blood
        // moon/item checks only reset the fully-asleep counter.
        if danger_near || (world.blood_moon && !world.day_time) || item_in_use {
            self.sleep_time = 0;
        }
        let bx = self.x + WIDTH as f32 * 0.5;
        let by = self.y + self.height - 2.0;
        let tx = (bx / 16.0).floor() as i32;
        let ty = (by / 16.0).floor() as i32;
        // `:97-102`: the bed tile re-decodes; a gone bed stops the sleep.
        let Some((_, _, target_direction, bed_style)) =
            tr_core::edit::sleeping_target_info(world, tx, ty)
        else {
            self.stop_sleeping();
            return;
        };
        // `:103-106`: the same exit mask as sitting (`controlUp` rides the
        // port's merged jump binding — W/ArrowUp feed `control_jump`).
        if self.control_left
            || self.control_right
            || self.control_jump
            || self.control_down
            || self.cart_mounted
            || self.mount.is_active()
            || target_direction != self.facing
        {
            self.stop_sleeping();
            return;
        }
        // `:103-106`: the same exit mask as sitting (`controlUp` rides the
        // port's merged jump binding — W/ArrowUp feed `control_jump`).
        if self.control_left
            || self.control_right
            || self.control_jump
            || self.control_down
            || self.cart_mounted
            || self.mount.is_active()
            || target_direction != self.facing
        {
            self.stop_sleeping();
            return;
        }
        // `:107-128`: a damaging/fishing/`ForcesBreaksSleeping` held item
        // breaks the sleep — the item stats land with M3-ITEM; the swing
        // itself already reset the counter above.
        // `:129-132`: the 2-players-per-tile stack (`Main.sleepingManager`)
        // is the multiplayer registry — SP no-op. `:133-137`: the offset
        // refresh — the port stores the style; the renderer derives the
        // table value.
        self.sleeping_style = bed_style;
    }

    /// The body height the sim feeds `base.height` with — 52 while the cart is
    /// mounted (`SetMount`, `Mount.cs:6270`: `height = 42 + heightBoost`), the
    /// live hitbox height otherwise (the mount framework's equip mutates it for
    /// its own mounts; unmounted 42). The width stays 20 (`:6271`).
    pub fn body_height(&self) -> f32 {
        if self.cart_mounted {
            CART_MOUNT_HEIGHT
        } else {
            self.height
        }
    }

    /// `Mount.SetMount(MountID.Minecart)` (`Mount.cs:6196-6305`) — the mount
    /// entry this phase exposes to the `TR_CART` pin (the item/QuickMount/hook
    /// paths are M2-MOUNT-3). A same-type re-mount is a no-op (`:6198`); a
    /// mount change resets the ride state (`:6205-6211`, pitfall §10.13).
    pub fn mount_cart(&mut self) {
        if self.cart_mounted {
            return;
        }
        self.cart_flip = false;
        self.last_boost = (0, 0);
        self.full_rotation = 0.0;
        self.full_rotation_origin = (0.0, 0.0);
        self.cart_mounted = true;
        // M2-MOUNT-3: through the mount framework — `SetMount(6)` grants the
        // row's buff 118 for 3600 (`:6226`) and applies the heightBoost 10
        // bottom-center growth 42 -> 52 (`:6266-6272`). The same-id guard
        // (`:6198`) keeps a re-mount a no-op, so the ride state stays.
        self.set_mount(crate::mount::MOUNT_MINECART, self.wet);
    }

    /// `Mount.CanDismountWithResult` (`Mount.cs:6611-6624`): the cart is not in
    /// `DontDismountWhenCCed` (`MountID.cs:47` — CCed never blocks it), so only
    /// the unmount-size room check can refuse. For the base cart the check is
    /// the 52→42 bottom-anchored shrink, which probes no tiles and always fits
    /// (`Collision.cs:2505-2510`).
    fn can_dismount(&self, world: &World) -> bool {
        collision::try_changing_size_from_bottom_center(
            world,
            [self.x, self.y],
            [WIDTH, self.body_height()],
            [WIDTH, HEIGHT],
        )
        .is_some()
    }

    /// `Mount.Dismount` (`Mount.cs:6142-6194`) for the cart ride: clears the
    /// ride state, the mount buff, and resizes 20×52 → 20×42 bottom-center.
    /// Both resize branches (`:6182-6193`) anchor the bottom for a shrink, so
    /// the port lifts the top by the freed heightBoost. The dismount dust is
    /// Phase 3. (Named apart from the mount-framework `dismount` — the cart
    /// ride sits outside `MountState` until M2-MOUNT-3 bridges them.)
    fn cart_dismount(&mut self) {
        if !self.cart_mounted {
            return; // `:6146-6149` — the `!_active` guard
        }
        self.cart_mounted = false;
        // M2-MOUNT-3: the framework's `Dismount` clears the mount buff
        // (`:6151`); the caller has already passed `CanDismountWithResult`,
        // so the space probe inside would also succeed.
        if let Some(buff) = self.mount.dismount() {
            self.clear_buff_type(buff);
        }
        self.cart_flip = false; // `:6157-6160`
        self.last_boost = (0, 0);
        self.full_rotation = 0.0; // `:6161-6162`
        self.full_rotation_origin = (0.0, 0.0);
        self.on_track = false;
        // The box restore: `mount_cart` now grows through the framework's
        // `grow_hitbox` (`self.height = 52`), so the drop returns it to 42
        // bottom-anchored (`:6182-6185`).
        self.y += CART_MOUNT_HEIGHT - HEIGHT;
        self.height = HEIGHT;
    }

    /// `Mount.TryDismount` (`Mount.cs:6137-6140` → `TryDismountWithResult`
    /// `:6127-6135`) for the cart ride: dismount only when the space check
    /// succeeds. (Named apart from the mount-framework `try_dismount`.)
    fn try_cart_dismount(&mut self, world: &World) {
        if self.can_dismount(world) {
            self.cart_dismount();
        }
    }

    /// Takes the pending pressure-track trip (the `Player.cs:28237-28241`
    /// `HitTrackSwitch` handoff — the caller owns `&mut World`).
    pub fn take_switch_trip(&mut self) -> Option<(i32, i32)> {
        self.pending_switch_trip.take()
    }

    /// Takes the tick's queued spark particles (the `Main.dust` handoff —
    /// the caller owns the pool).
    pub(crate) fn take_cart_sparks(&mut self) -> Vec<crate::minecart::SparkP> {
        std::mem::take(&mut self.pending_sparks)
    }

    /// Takes the tick's queued minecart cues (the `SoundEngine.PlaySound`
    /// handoff — the caller owns the audio device).
    pub(crate) fn take_cart_sounds(&mut self) -> Vec<crate::minecart::CartCue> {
        std::mem::take(&mut self.pending_cart_sounds)
    }

    /// `PlayerSittingHelper.UpdateSitting` (`PlayerSittingHelper.cs:50-87`):
    /// the seat tile re-decodes through `GetSittingTargetInfo` (`:56-61`),
    /// any control input / the mount / a facing mismatch stands up
    /// (`:62-65`), and the seat offset refreshes (`:84-85`). The 2-per-tile
    /// seat stack (`Main.sittingManager`, `:66-69`) and the Skeletron-hat
    /// bench easter egg (`:74-83`) are the multiplayer/NPC sides — SP no-op /
    /// not ported. (`controlUp` rides the port's merged jump binding —
    /// W/ArrowUp feed `control_jump`.)
    pub fn update_sitting(&mut self, world: &World) {
        if !self.sitting {
            return;
        }
        // `TryGetSittingBlock` (`:33-48`): the tile at the bottom - 2 px
        // re-decodes as a sit-able seat.
        let bx = self.x + WIDTH as f32 * 0.5;
        let by = self.y + self.height - 2.0;
        let tx = (bx / 16.0).floor() as i32;
        let ty = (by / 16.0).floor() as i32;
        let Some((_, _, target_direction, seat_down_offset)) =
            tr_core::edit::sitting_target_info(world, tx, ty, self.facing)
        else {
            self.sit_up();
            return;
        };
        if self.control_left
            || self.control_right
            || self.control_jump
            || self.control_down
            || self.cart_mounted
            || self.mount.is_active()
            || target_direction != self.facing
        {
            self.sit_up();
            return;
        }
        self.offset_for_seat = seat_down_offset;
        self.seat_pos_offset = (seat_down_offset.0 * self.facing as f32, seat_down_offset.1);
        self.seat_y_offset = -4 + seat_down_offset.1 as i32;
    }

    /// The test/inspection reads for the private speed fields.
    pub(crate) fn acc_run_speed_value(&self) -> f32 {
        self.acc_run_speed
    }

    pub(crate) fn max_run_speed_value(&self) -> f32 {
        self.max_run_speed
    }

    pub(crate) fn accessory_life_regen_value(&self) -> i32 {
        self.accessory_life_regen
    }

    /// Used by the `TR_XY` debug hook and future teleport effects.
    pub fn teleport(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
        self.prev_x = x;
        self.prev_y = y;
        self.vx = 0.0;
        self.vy = 0.0;
        self.on_ground = false;
        self.jump = 0;
        self.fall_start = (y / TILE_PX) as i32;
        self.gfx_off_y = 0.0;
        self.prev_gfx_off_y = 0.0;
    }

    pub fn respawn(&mut self) {
        self.x = self.spawn_x;
        self.y = self.spawn_y;
        self.prev_x = self.x;
        self.prev_y = self.y;
        self.vx = 0.0;
        self.vy = 0.0;
        self.on_ground = false;
        self.jump = 0;
        self.jump_held = false;
        self.leg_frame = 0;
        self.body_frame = 0;
        self.leg_frame_counter = 0.0;
        self.body_frame_counter = 0.0;
        self.swim_time = 0;
        self.pulley = false;
        self.pulley_dir = 0;
        self.shield_raised = false;
        self.sliding = false;
        self.sand_storm = false;
        self.carpet_frame = -1;
        // `Player.Spawn`'s revive life (`Player.cs:37598-37608`): the pool
        // is `max(statLifeMax, statLifeMax2)` — the port's single `max_hp`
        // is the effective max — and the life is `max(pool / 2, 100)`. A
        // fresh 100-life character revives FULL; the `spawnMax` Journey arm
        // (`:37609-37611`) has no port counterpart.
        self.hp = (self.max_hp / 2).max(100).min(self.max_hp.max(100));
        // `:37614-37618`: the i-frames — `immune = true`, the context arm
        // below lands 180 (the port's respawn is always the death revive;
        // the 60 arm rides the other `PlayerSpawnContext`s).
        self.immune = true;
        self.immune_time = 180;
        self.immune_no_blink = true;
        self.hurt_cd = 0.0;
        // `UpdateDead`'s brain-counter zero (`Player.cs:16932`): the death tail
        // clears the animation window with the rest of the per-life state.
        self.brain_of_confusion_dodge_animation_counter = 0;
        self.breath = BREATH_MAX;
        self.breath_cd = 0;
        self.regen = RegenState::default();
        self.dot_death = None;
        self.fall_start = (self.y / TILE_PX) as i32;
        self.wet = false;
        self.lava_wet = false;
        self.honey_wet = false;
        self.shimmer_wet = false;
        self.in_lava = false;
        // `Player.UpdateDead` (`Player.cs:16929`): death zeroes the special.
        self.mount_fishron_special_counter = 0.0;
        self.gfx_off_y = 0.0;
        self.prev_gfx_off_y = 0.0;
        // `Player.KillMe` dismounts the mount (`Player.cs:38907`); the respawn
        // resets the ride state with it (the position is rebuilt from spawn
        // below, so no bottom-center resize is needed here).
        self.cart_mounted = false;
        self.cart_flip = false;
        self.on_track = false;
        self.on_wrong_ground = false;
        self.track_boost = 0.0;
        self.last_boost = (0, 0);
        self.cart_ramp_time = 0;
        self.full_rotation = 0.0;
        self.full_rotation_origin = (0.0, 0.0);
        self.fall_start2 = (self.y / TILE_PX) as i32;
        self.pending_switch_trip = None;
        self.pending_sparks.clear();
        self.pending_cart_sounds.clear();
        self.buffs.clear_buff(CART_MOUNT_BUFF);
        // M2-MOUNT-3: the cart row lives in `MountState` now, so the revive
        // strip clears it like every other mount (`Player.cs:38908`) — the
        // framework's dismount returns the box with it, and the spawn
        // position is rebuilt below anyway.
        if let Some(buff) = self.mount.dismount() {
            self.clear_buff_type(buff);
        }
        self.height = HEIGHT;
        // `Player.Spawn`'s `StopVanityActions` (`Player.cs:37556`): the
        // revive clears the sit/sleep state.
        self.sit_up();
        self.stop_sleeping();
    }

    /// `Player.KillMe`'s respawn setup (`Player.cs:38908-38910`): the dead
    /// flag, `deadTime` reset and `respawnTimer = GetRespawnTime(pvp)`.
    pub fn start_death(&mut self, world: &World, boss_near: bool, expert: bool) {
        // `Player.KillMe`'s head (`Player.cs:38809-38812`): the godmode power
        // (and an already-dead player) refuses the death entirely.
        if self.creative_god_mode || self.dead {
            return;
        }
        // `Player.KillMe`'s counter arm (`Player.cs:38840-38844`): the PVP
        // flag splits the two counters; the port has no PVP, so every
        // death is PVE.
        self.deaths_pve = self.deaths_pve.saturating_add(1);
        self.dead = true;
        self.dead_time = 0;
        self.respawn_timer = Player::get_respawn_time(boss_near, expert);
        // `Player.KillMe` -> `mount.Dismount` (`Player.cs:38907`).
        let _ = self.dismount(world, true);
    }

    /// `Player.GetRespawnTime` (`Player.cs:39060-39082`): 600 base, +600 when
    /// a boss hinders the respawn, x1.5 in expert. The FTW-multiplier x2 and
    /// the master-mode path stay with M5-SEED-1 / the master difficulty.
    pub fn get_respawn_time(boss_near: bool, expert: bool) -> i32 {
        let mut time = 600;
        if boss_near {
            time += 600;
        }
        if expert {
            time = (time as f64 * 1.5) as i32;
        }
        time
    }

    /// `NPC.AnyBossHindersRespawnTime` shape (`NPC.cs`): a boss within the
    /// 4000 px Manhattan range of the death spot. The King Slime is the
    /// ported boss.
    pub fn boss_hinders(&self, npcs: &[crate::npc::Npc]) -> bool {
        let (cx, cy) = self.center();
        npcs.iter().any(|n| {
            n.kind == crate::npc::NpcKind::KingSlime && {
                let (nx, ny) = n.center();
                (cx - nx).abs() + (cy - ny).abs() < 4000.0
            }
        })
    }

    /// `Player.NearAnyNPCsThatBlockRespawn` (`Player.cs:38987-39018`): a live
    /// NPC inside the centred `Main.MaxWorldViewSize` (1920x1080) rect inflated
    /// by 320 px blocks the respawn skip — an invasion NPC while its invasion
    /// runs (`NPC.GetNPCInvasionGroup(type) == Main.invasionType`), or a
    /// `NPCID.Sets.DangerThatPreventsOtherDangers` member (the port has none
    /// yet, so only the invasion arm fires).
    pub fn near_any_npcs_that_block_respawn(
        &self,
        npcs: &[crate::npc::Npc],
        invasion_live: bool,
    ) -> bool {
        // `Utils.CenteredRectangle(Center, Main.MaxWorldViewSize)` then
        // `Inflate(320, 320)` (`:38992-38998`).
        const VIEW_W: f32 = 1920.0;
        const VIEW_H: f32 = 1080.0;
        const INFLATE: f32 = 320.0;
        let (cx, cy) = self.center();
        let half_w = VIEW_W * 0.5 + INFLATE;
        let half_h = VIEW_H * 0.5 + INFLATE;
        npcs.iter().any(|n| {
            let (nx, ny) = n.center();
            let (w, h) = n.size();
            let overlaps = (nx - cx).abs() <= half_w + w * 0.5
                && (ny - cy).abs() <= half_h + h * 0.5;
            if !overlaps {
                return false;
            }
            // `NPC.GetNPCInvasionGroup(nPC.type) > 0 && == Main.invasionType`
            // (`:39005-39008`); the danger-set arm (`:39009-39012`) has no
            // ported members.
            invasion_live && n.kind.is_invasion_npc()
        })
    }

    /// `Player.AllowsRespawnTimerSkip` (`Player.cs:38962-38987`): after the
    /// 90-tick lockout, with no boss hindering, no danger NPCs nearby and
    /// no Frost/Pumpkin/OOA event running (`:38984-38986`). The
    /// `ghost`/`pvpDeath` branches stay unreachable in the port.
    pub fn allows_respawn_timer_skip(
        &self,
        boss_near: bool,
        danger_near: bool,
        moon_event_live: bool,
    ) -> bool {
        !(self.ghost_like()
            || self.dead_time < 90
            || boss_near
            || danger_near
            || moon_event_live)
    }

    /// The port has no ghosts (hardcore) or PVP deaths; both gates are false.
    fn ghost_like(&self) -> bool {
        false
    }

    /// `Player.UpdateDead` (`Player.cs:16919-17070`): `deadTime++`, the
    /// respawn decrement clamped to `respawnTimerMax` (3600), and the
    /// jump-press skip while `AllowsRespawnTimerSkip`. Returns true when the
    /// player should respawn now.
    pub fn update_dead(
        &mut self,
        jump: bool,
        boss_near: bool,
        danger_near: bool,
        moon_event_live: bool,
    ) -> bool {
        self.dead_time += 1;
        self.respawn_timer = (self.respawn_timer - 1).clamp(0, 3600);
        // `Player.UpdateDead`'s counter reset (`Player.cs:17009-17010`): the
        // budget zeroes while dead — the pool stays until AI_062's dead-owner
        // flag read (M4-PROJ-8 phase 4) despawns the minions.
        self.num_minions = 0;
        self.slots_minions = 0.0;
        let jump_edge = jump && !self.prev_jump_input;
        self.prev_jump_input = jump;
        let can_skip = self.allows_respawn_timer_skip(boss_near, danger_near, moon_event_live);
        if can_skip && jump_edge && self.respawn_timer > 0 {
            return true;
        }
        self.respawn_timer <= 0
    }

    /// `Player.Spawn(ReviveFromDeath)`: the dead flags clear, then the
    /// standard spawn-point respawn.
    pub fn revive(&mut self) {
        self.dead = false;
        self.dead_time = 0;
        self.respawn_timer = 0;
        self.respawn();
    }

    /// `Main.TryRemovingBuff` (`Main.cs:43370-43389`): the buff bar's
    /// right-click removal — non-debuffs only, and not Petrified 60 /
    /// Webbed 151; the mount's own buff dismounts instead of removing the
    /// buff (`Mount.CheckBuff`, `Mount.cs` — `data.buff == b`). The
    /// `TryRemovingBuff_CheckBuffHideMisc` arms (`:43391-43402`) ride the
    /// misc-equips pets (unported). Returns whether the arm entered (the
    /// caller plays `SoundEngine.PlaySound(12)`'s MenuTick cue).
    pub fn try_removing_buff(&mut self, i: usize, b: i32) -> bool {
        if !crate::buffs::is_debuff(b) && b != 60 && b != 151 {
            if self.mount.is_active() && self.mount.data().map(|d| d.buff) == Some(b) {
                if let Some(buff) = self.mount.dismount() {
                    self.clear_buff_type(buff);
                }
            } else {
                self.buffs.del_buff(i);
            }
            return true;
        }
        false
    }

    /// `GetNurseHealCost`'s slot count (`Main.cs:40058-40065`): a debuff
    /// slot counts when `Main.debuff[b] && buffTime > 60 &&
    /// !NurseCannotRemoveDebuff[b]`.
    pub fn remountable_debuff_count(&self) -> i32 {
        let mut count = 0;
        for i in 0..crate::buffs::MAX_BUFFS {
            let b = self.buffs.buff_type[i];
            if crate::buffs::is_debuff(b)
                && self.buffs.buff_time[i] > 60
                && !crate::buffs::nurse_cannot_remove(b)
            {
                count += 1;
            }
        }
        count
    }

    /// `NPCChatText_DoNurseHeal`'s removal pass (`Main.cs:40147`): the
    /// debuff slots with `buffTime > 0` and the Nurse's protected list
    /// clear.
    pub fn clear_nurse_removable_debuffs(&mut self) {
        for i in 0..crate::buffs::MAX_BUFFS {
            let b = self.buffs.buff_type[i];
            if crate::buffs::is_debuff(b)
                && self.buffs.buff_time[i] > 0
                && !crate::buffs::nurse_cannot_remove(b)
            {
                self.buffs.clear_buff(b);
            }
        }
    }

    /// `Player.ResetProjectileCaches` (`Player.cs:12217-12226`): the recount
    /// input zeroed — called from the `ResetEffects` seam (`:18810`) and the
    /// dead path (`:24752`).
    pub fn reset_projectile_caches(&mut self) {
        self.highest_storm_tiger_gem_original_damage = 0;
        self.highest_abigail_counter_original_damage = 0;
        self.owned_projectile_counts = [0; PROJ_COUNT];
    }

    /// `Player.UpdateProjectileCaches` (`Player.cs:12184-12215`): the per-tick
    /// recount of the live pool's owned projectiles into
    /// `owned_projectile_counts` plus the 831/970 `originalDamage` maxima.
    /// The C# walks `Main.projectile[1000]` for `owner == i`; the port's
    /// whoAmI stand-in is 0 (the single-player owner the spawner folds to).
    pub fn update_projectile_caches(&mut self, pool: &ProjectilePool) {
        for p in pool.slots() {
            if !p.active || p.owner != 0 {
                continue;
            }
            // A kind at or above `ProjectileID.Count` cannot exist in the C#
            // id space; the count skips it instead of indexing out of range.
            if let Some(count) = self.owned_projectile_counts.get_mut(usize::from(p.kind)) {
                *count += 1;
            }
            match p.kind {
                831 => {
                    if self.highest_storm_tiger_gem_original_damage < p.original_damage {
                        self.highest_storm_tiger_gem_original_damage = p.original_damage;
                    }
                }
                970 => {
                    if self.highest_abigail_counter_original_damage < p.original_damage {
                        self.highest_abigail_counter_original_damage = p.original_damage;
                    }
                }
                _ => {}
            }
        }
    }

    /// `Player.UpdateBuffs`' summon chain (`Player.cs:10483-10947`): the
    /// slot-ordered keep-alive arms (with the cap effects `:10175-10183` and
    /// the buff-216 respawn arm `:10697-10726`), driven by
    /// `owned_projectile_counts`. Runs inside the fixed step at the
    /// `UpdateBuffs` seam — the pool and the inventory ride in as parameters
    /// because `Player::update` cannot see the App's.
    ///
    /// The ported order, pinned (the plan constraint — verified against the
    /// decompile this phase):
    /// - `Main.Update` → `UpdatePlayers` (`Main.cs:17754`) → `Player.Update`:
    ///   `ResetEffects` (`:25491` — `maxMinions`/`maxTurrets` = 1 at
    ///   `:18448-18449`, the summon flags `:18620-18646`+`:18741`,
    ///   `ResetProjectileCaches` `:18810`) → the buffImmune reset
    ///   (`:25562-25564`) → `UpdateProjectileCaches(i)` (`:25565`) →
    ///   `UpdateBuffs(i)` (`:25566` — ONE `for j` loop: the timer head
    ///   `:9869-9876`, then the else-if chain: the cap effects
    ///   `:10175-10183`, the summon arms `:10483-10947`, `DelBuff(j); j--`
    ///   rescanning the slot the compaction filled) → `UpdateEquips`
    ///   (`:25633` — `grant_summon_caps`) → `numMinions = 0; slotsMinions =
    ///   0f` (`:28279-28280`, before ItemCheck).
    /// - `Main.Update` → `UpdateNPCs` (`:17767`) → `UpdateProjectiles`
    ///   (`:17775` — the pool pass accumulates `numMinions`/`slotsMinions`,
    ///   `Projectile.cs:16736-16737`, M4-PROJ-8 phase 3). So the value this
    ///   chain reads for `numMinions` is the last pool pass's accumulation —
    ///   the live pool's owned-minion count (the pool is unchanged between
    ///   the passes).
    /// The port realizes the same order across `update_with_grapple` (the
    /// ResetEffects seam + the buff timer/effects), this method, the
    /// `grant_*` seams and the pool's update — each counter's read order
    /// matches. The buff timers decrement and the expired slots purge in
    /// `update_with_grapple` before the arms run here; the C# arms run
    /// before the `:25608` purge, but a summon buff's time is pinned at
    /// 18000 by the refresh while its minion lives (and a dead-keep-alive
    /// buff is DelBuff'd by the arm either way), so the swap is
    /// unobservable.
    pub fn update_summon_buff_arms(
        &mut self,
        pool: &mut ProjectilePool,
        inventory: &[ItemSlot; INV],
    ) {
        // The slot-ordered chain with the C# `j--` rescan: after `DelBuff(j)`
        // the compaction shifts the later buffs left, so index `j` is
        // re-examined instead of advanced (the shifted-in buff's arm runs in
        // the same tick, `Player.cs:10483-10947`).
        let mut j = 0usize;
        while j < MAX_BUFFS {
            let buff = self.buffs.buff_type[j];
            // The C# guard (`:9871-9874`) — the decrement already ran in
            // `update_with_grapple`'s timer tick.
            if buff <= 0 || self.buffs.buff_time[j] <= 0 {
                j += 1;
                continue;
            }
            let survives = match buff {
                // `Player.cs:10175-10183` — the cap effects share the chain,
                // slot-ordered with the arms (the buff-216 arm's `maxMinions`
                // read at `:10706` sees the earlier slots' raises).
                110 | 150 => {
                    self.max_minions += 1;
                    true
                }
                348 => {
                    self.max_turrets += 1;
                    true
                }
                216 => self.run_baby_bird_arm(pool, inventory, j),
                _ => match SUMMON_BUFF_ARMS.iter().find(|a| a.buff == buff) {
                    Some(arm) => {
                        let kept = arm
                            .kinds
                            .iter()
                            .any(|&k| self.owned_projectile_counts[usize::from(k)] > 0);
                        if kept {
                            *(arm.flag)(self) = true;
                            self.buffs.buff_time[j] = 18000;
                            true
                        } else {
                            self.buffs.del_buff(j);
                            false
                        }
                    }
                    // Not a summon arm — the chain leaves the slot alone.
                    None => true,
                },
            };
            if survives {
                // `Player.cs:10874-10877` / `:10893-10896`: the companion
                // updaters run after their arm kept the buff.
                match buff {
                    335 => self.update_abigail_status(),
                    263 => self.update_storm_tiger_status(),
                    _ => {}
                }
                j += 1;
            }
            // else: the slot was DelBuff'd — rescan index j.
        }
        // `Player.cs:28279-28280`: the per-tick zero before ItemCheck — the
        // pool pass (Phase 3) accumulates the values the next tick's arms
        // read.
        self.num_minions = 0;
        self.slots_minions = 0.0;
    }

    /// Buff 216's arm (`Player.cs:10697-10726`): keep-alive on the 759 count;
    /// otherwise the respawn arm — below cap with the Baby Bird Staff (4281)
    /// in the inventory, spawn the item's shoot at the player's top with the
    /// item's damage/knockBack (`originalDamage = item.damage` at `:10713` —
    /// the data-row spawn seam writes it for minion rows); at cap (or without
    /// the staff) `DelBuff(j)` and no refresh (the C# `flag2` guard). Returns
    /// false when the slot was deleted — the caller rescans the index.
    fn run_baby_bird_arm(
        &mut self,
        pool: &mut ProjectilePool,
        inventory: &[ItemSlot; INV],
        j: usize,
    ) -> bool {
        if self.owned_projectile_counts[usize::from(BABY_BIRD)] > 0 {
            self.baby_bird = true;
        } else {
            // `base.whoAmI == Main.myPlayer` always holds in the
            // single-player port.
            if self.num_minions < self.max_minions {
                // `Player.FindItem` (`Player.cs:55972-55981`): the first
                // inventory slot with the type and a stack.
                if let Some(idx) = inventory.iter().position(|s| s.kind == 4281 && s.count > 0) {
                    if let Some(def) = registry::get(inventory[idx].kind) {
                        // `Projectile.NewProjectile(..., base.Top,
                        // Vector2.Zero, item.shoot, item.damage,
                        // item.knockBack, whoAmI)` — `Entity.Top` is the
                        // hitbox's top center; item 4281's `Item.shoot` is
                        // 759 (the generated row), so the verbatim data-row
                        // slice (minion flag, 1.0 slots, timeLeft 18000,
                        // the local pair) rides the spawn seam. The C#'s
                        // pool-full sentinel crash stands in as "not
                        // spawned".
                        if pool
                            .spawn_with_data_ai_owned(
                                BABY_BIRD,
                                0,
                                self.x + WIDTH * 0.5,
                                self.y,
                                0.0,
                                0.0,
                                def.damage,
                                def.knock_back,
                                0.0,
                                0.0,
                            )
                            .is_some()
                        {
                            self.baby_bird = true;
                        }
                    }
                }
            }
            if !self.baby_bird {
                self.buffs.del_buff(j);
                return false;
            }
        }
        self.buffs.buff_time[j] = 18000;
        true
    }

    /// `Player.UpdateAbigailStatus` (`Player.cs:12274-12293`) — STUB. Kills
    /// the 963 companions when the 970 count drops to 0 and spawns one beside
    /// the newest Abigail otherwise; both halves need the pool spawn/AI work
    /// (`FindNewestAI_164Minion`, the aiStyle-62 family) landing in M4-PROJ-8
    /// phases 4/6. Until then an orphan companion self-despawns through
    /// AI_062's flag read (`Projectile.cs:65045-65051`, Phase 4). Deviation
    /// recorded for ANALYSIS.
    fn update_abigail_status(&mut self) {}

    /// `Player.UpdateStormTigerStatus` (`Player.cs:12295-12328`) — STUB, the
    /// 833-835 rank management (`GetDesiredStormTigerMinionRank` + the
    /// `StormTigerIds` sets); lands with phases 4/6 like the Abigail stub.
    fn update_storm_tiger_status(&mut self) {}

    /// The `ResetEffects` summon-flag clear (`Player.cs:18620-18646`,
    /// `:18741`) — table-driven so the arms table stays the single source.
    fn clear_summon_flags(&mut self) {
        for arm in &SUMMON_BUFF_ARMS {
            *(arm.flag)(self) = false;
        }
    }

    /// Contact damage / future combat. Returns true when the hit was accepted,
    /// false when it was dodged or swallowed. `rng` is the caller's active
    /// stream (the C# dodge rolls draw from `Main.rand`, the S1-A-10 contract)
    /// and `dodgeable` is `Hurt`'s parameter (`Player.cs:38197` — the
    /// contact path passes `IsDamageDodgeable`, which rides Phase 2).
    pub fn hurt(
        &mut self,
        amount: i32,
        from_x: f32,
        rng: &mut dyn tr_core::loot::DropRng,
        dodgeable: bool,
    ) -> bool {
        self.hurt_inner(amount, from_x, true, rng, dodgeable)
    }

    /// `Player.SetImmuneTimeForAllTypes` (`Player.cs:38013-38021`): the whole
    /// write rides the port's fields — `immune = true` + `immuneTime = time`
    /// (the fields R-03-5 ported; `immuneNoBlink` stays untouched, like the
    /// C#) and the per-cooldown array collapsed onto the one `hurt_cd` (the
    /// M4-HAZARD-1 admission). The dodge handlers pass the 120/80 pair
    /// directly; `iframes_for` (the normal-hurt window) is not involved.
    fn set_immune_time_for_all_types(&mut self, time: i32) {
        self.immune = true;
        self.immune_time = time;
        self.hurt_cd = time as f32 / 60.0;
    }

    /// `Player.ShadowDodge` (`Player.cs:38023-38042`) minus the NetMessage:
    /// the 120/80 immunity, buff 59 deleted by the same single-pass `for` with
    /// `DelBuff`'s compaction (`:38030-38040`), the Hallowed set-bonus
    /// cooldown armed (`PutHallowedArmorSetBonusOnCooldown` `:38037-38039` —
    /// it keeps the 30 s buff-59 cycle from re-granting).
    fn do_shadow_dodge(&mut self) {
        self.set_immune_time_for_all_types(if self.long_invince { 120 } else { 80 });
        for i in 0..crate::buffs::MAX_BUFFS {
            if self.buffs.buff_time[i] > 0 && self.buffs.buff_type[i] == 59 {
                self.buffs.del_buff(i);
            }
        }
        self.shadow_dodge_timer = 1800;
    }

    /// `Player.BrainOfConfusionDodge` (`Player.cs:38046-38055`) minus the
    /// NetMessage: the 120/80 immunity, the 300-tick animation window and the
    /// brain's own cooldown buff 321 (240 ticks — the roll's
    /// `FindBuffIndex(321)` gate).
    fn brain_of_confusion_dodge(&mut self) {
        self.set_immune_time_for_all_types(if self.long_invince { 120 } else { 80 });
        self.brain_of_confusion_dodge_animation_counter = 300;
        self.buffs.add_buff(321, 240);
    }

    /// `Player.DoMysticSashDodge` (`Player.cs:38057-38059`): the 120/80
    /// immunity; the 50x dust + gore burst is deferred to S1-E-1.
    fn do_mystic_sash_dodge(&mut self) {
        self.set_immune_time_for_all_types(if self.long_invince { 120 } else { 80 });
    }

    /// `Player.NinjaDodge` (`Player.cs:38119-38121`): the 120/80 immunity; the
    /// 100x dust + gore burst is deferred to S1-E-1.
    fn ninja_dodge(&mut self) {
        self.set_immune_time_for_all_types(if self.long_invince { 120 } else { 80 });
    }

    /// `Player.UpdateImmunity` (`Player.cs:18856-18869`): the i-frame
    /// countdown — `immune` clears at 0 and `immuneNoBlink` rides with it;
    /// the `immuneAlpha` blinkVisual half is draw-side and stays out.
    /// (R-03-5's port; the dodge handlers above write the same fields via
    /// `SetImmuneTimeForAllTypes`, so both windows decay through here.)
    pub fn update_immunity(&mut self) {
        if self.immune {
            self.immune_time -= 1;
            if self.immune_time <= 0 {
                self.immune = false;
                self.immune_no_blink = false;
            }
        }
    }

    /// The shared hurt body. `knock_back` mirrors the original's
    /// `hitDirection != 0` gate (`Player.cs:38519`): the fall-damage and
    /// petrify calls pass hitDirection 0 and skip the knock entirely (the
    /// fallStart reset inside that branch rides with it).
    /// `Player.Update`'s contact reflection (`Player.cs:31373-31415`): the
    /// damage an NPC takes for touching the player — `turtleThorns` pins the
    /// multiplier at 2, else `thorns` scales the NPC's own damage, capped at
    /// 1000; the Cactus set adds a flat 15 more.
    /// `Player.GetArmorPenetration` (`Player.cs:4365-4373`): the flat sum,
    /// plus the melee-only row when the hit is melee (`sItem.melee`).
    pub fn get_armor_penetration(&self, melee: bool) -> i32 {
        self.armor_penetration + if melee { self.melee_armor_penetration } else { 0 }
    }

    pub fn thorns_damage(&self, incoming: i32) -> i32 {
        let multiplier = if self.turtle_thorns {
            2.0
        } else {
            self.thorns
        };
        let mut total = 0;
        if multiplier > 0.0 {
            total += ((incoming as f32 * multiplier) as i32).min(1000);
        }
        if self.cactus_thorns {
            total += 15;
        }
        total
    }

    pub(crate) fn hurt_inner(
        &mut self,
        amount: i32,
        from_x: f32,
        knock_back: bool,
        rng: &mut dyn tr_core::loot::DropRng,
        dodgeable: bool,
    ) -> bool {
        // `Player.Hurt`'s creative-godmode arm (`Player.cs:38203-38206`): the
        // journey godmode power swallows the hit outright — before the
        // iframes, the defense reduction and the knockback.
        if self.creative_god_mode {
            return false;
        }
        // `Player.Hurt`'s General-cooldown gate (`Player.cs:38210-38216`):
        // the port's single damage lane rides `ImmunityCooldownID.General`,
        // whose flag is `!immune` — the spawn i-frames swallow the hit.
        if self.immune {
            return false;
        }
        if self.hurt_cd > 0.0 || self.hp <= 0 {
            return false;
        }
        // The dodge roll block (`Player.cs:38214-38241`), slotted after the
        // cooldown early-return (the port's `hurt_cd`-gate IS the C# `flag`
        // for the General counter) and before the defense reduction.
        // `whoAmI == Main.myPlayer` collapses in the single-player port. The
        // roll order is sash -> belt -> brain -> shadow; each `Next` draws
        // only while its source is armed and the previous source did not
        // dodge (the early returns short-circuit the rest). The brain's
        // `Next(6)` precedes the buff-321 check in the C# `&&` chain — the
        // draw happens whenever the item is armed, buff 321 only gates the
        // dodge itself; shadow draws nothing.
        if dodgeable {
            if self.mystic_sash_dodge && rng.next_range(0, 10) == 0 {
                self.do_mystic_sash_dodge();
                return false;
            }
            if self.black_belt && rng.next_range(0, 10) == 0 {
                self.ninja_dodge();
                return false;
            }
            if self.brain_of_confusion
                && rng.next_range(0, 6) == 0
                && self.buffs.find_buff_index(321).is_none()
            {
                self.brain_of_confusion_dodge();
                return false;
            }
            if self.shadow_dodge {
                self.do_shadow_dodge();
                return false;
            }
        }
        // `Player.Hurt`'s defense reduction (the classic
        // `damage - statDefense * 0.5`, floored at 1 — M3-EQUIP-2).
        let amount = tr_core::armor_sets::reduce_damage(amount, self.stat_defense);
        // `Player.cs:38304`: the endurance step — `(int)((1 - endurance) *
        // damage)` after the defense reduction.
        let amount = ((1.0 - self.endurance) * amount as f32) as i32;
        // `Player.cs:38378-38381`: the `num2 < 1 -> 1` floor. The endurance
        // truncation can drive a hit to 0; the C# lifts it back to 1 (which
        // also selects the 20-tick immunity window below, not 40).
        let amount = amount.max(1);
        self.hp = (self.hp - amount).max(0);
        // `Player.Hurt`'s immunity window (`Player.cs:38385`): 40 ticks
        // for normal damage (no long-invincibility source yet), 20 when
        // the damage is exactly 1; PVP 8 is unreachable in single-player.
        self.hurt_cd = tr_core::damage::iframes_for(amount, false, false) as f32 / 60.0;
        // `Player.Hurt` resets the natural-ramp timer (`Player.cs:38395`).
        self.regen.life_regen_time = 0.0;
        if knock_back && !self.no_knockback {
            let dir = if self.x + WIDTH * 0.5 >= from_x {
                1.0
            } else {
                -1.0
            };
            // The fixed contact knockback (`Player.cs` Hurt: `4.5 * dir` with
            // `-3.5` vertical unless noKnockback).
            self.vx = dir * 4.5;
            self.vy = -3.5;
            self.on_ground = false;
        }
        true
    }

    /// The fall-damage landing hit (`Player.cs:25361-25370`): the normal branch
    /// forces `immune = false` BEFORE the Hurt — the hit lands through any
    /// iframes — and hitDirection 0 means no knockback and no fallStart reset.
    fn hurt_fall(&mut self, amount: i32, rng: &mut dyn tr_core::loot::DropRng) -> bool {
        self.hurt_cd = 0.0;
        let cx = self.x + WIDTH * 0.5;
        // The landing hit keeps `Hurt`'s default `dodgeable = true` — a
        // black-belt dodge eats fall damage in the vanilla (research S1-C-1
        // §1.4).
        self.hurt_inner(amount, cx, false, rng, true)
    }

    /// `Player.tileRangeX` (`Player.cs:2496`, `DefaultTileRangeX = 5` at
    /// `:57113`) after `Player.ResetEffects`' journey block
    /// (`:18815-18828`): the base 5, or `5 * 2 + 8 = 18` while the
    /// `FarPlacementRangePower` is enabled. The C# gates the doubling on
    /// `Main.IsJourneyMode`; the port has no journey-difficulty flag, so the
    /// power's own toggle is the gate.
    pub fn tile_range_x(&self) -> f32 {
        let base = 5.0;
        if self.far_placement_range {
            base * 2.0 + 8.0
        } else {
            base
        }
    }

    /// `Player.tileRangeY` (`DefaultTileRangeY = 3`, `Player.cs:57114`) with
    /// the same journey doubling (`:18823-18825`): `3` or `14`.
    pub fn tile_range_y(&self) -> f32 {
        let base = 3.0;
        if self.far_placement_range {
            base * 2.0 + 8.0
        } else {
            base
        }
    }

    pub fn center(&self) -> (f32, f32) {
        (self.x + WIDTH * 0.5, self.y + self.height * 0.5)
    }

    /// `Player.meleeSpeed` (`Player.cs:2458`, reset to 1f at `:18335`): the
    /// base 1f plus the accumulator the equipment/buff grants write
    /// (`meleeSpeedBonus`; the C# folds its flat `+=` arms straight into
    /// `meleeSpeed`, the port keeps the same sum). The flail family scales
    /// its per-kind speeds and its 900-px leash by `1 / meleeSpeed`
    /// (`AI_015_Flails` `Projectile.cs:43824-43831`, `:43727`), and the
    /// melee shoot path's `speed /= meleeSpeed` (`Player.cs:47063`) reads it.
    pub fn melee_speed(&self) -> f32 {
        1.0 + self.melee_speed_bonus
    }

    /// `Player.MountedCenter` (`Player.cs:3335-3343`):
    /// `position + (width/2, BaseHeight/2 + HeightOffsetHitboxCenter)`. The
    /// port has no `HeightOffsetBoost`/`HeightOffsetHitboxCenter`/mount
    /// offsets (no mounts yet), so both fold to 0 and the vector equals
    /// `center()` — kept as its own accessor so the flail chain anchor cites
    /// the C# point and survives the mount port.
    pub fn mounted_center(&self) -> (f32, f32) {
        self.center()
    }

    /// `Player.killGuide` / `Player.killClothier` (`Player.cs:2770`/`:2772`):
    /// `ApplyEquipFunctional` arms them for the worn accessory slots 3..10
    /// (`:14368`, the `UpdateEquips` loop `:12948-12954` — `IsItemSlotUnlockedAndUsable`
    /// is not modelled, every slot counts as unlocked) and `ResetEffects`
    /// clears them every tick (`:18365-18366`). The melee gate
    /// (`ProcessHitAgainstNPC` `:44274`) and the projectile gate
    /// (`Damage_PVE_Inner` `:12601`) both read this pair.
    pub fn doll_flags(&self) -> (bool, bool) {
        let worn = |k: u16| self.equipped[3..10].contains(&k);
        (worn(267), worn(1307))
    }

    /// `Player.GetWeaponDamageMultiplier` (`Player.cs:53386`) narrowed to
    /// the ported classes: the classless items pass the 1f default.
    pub fn weapon_damage_multiplier(&self, class: registry::DamageClass) -> f32 {
        match class {
            registry::DamageClass::Melee => self.melee_damage,
            registry::DamageClass::Ranged => self.ranged_damage,
            registry::DamageClass::Magic => self.magic_damage,
            registry::DamageClass::Summon => self.minion_damage,
            _ => 1.0,
        }
    }

    /// `Player.GetWeaponDamageMultiplier` in full (`Player.cs:53386-53419`),
    /// the item-aware form: the ranged dispatch keys on the ammo category —
    /// arrows → `bowEffectiveDamage` (`:3982`), bullets →
    /// `gunEffectiveDamage` (`:3984`), the specialist rockets/everything
    /// else → `specialistEffectiveDamage` (`:3986`) — and falls to
    /// `rangedDamage` for the plain guns. `use_ammo` is the weapon's
    /// `useAmmo` (the `PickAmmo_PickAmmoItem` equivalence makes it the
    /// resolved ammo's `ammo` field too — the proof `is_arrow_ammo`'s doc
    /// carries); `item_type` feeds `ItemID.Sets.IsRangedSpecialistWeapon`
    /// (`ItemID.cs:1472`). The fuse sites call this; the class-only callers
    /// keep [`Player::weapon_damage_multiplier`].
    pub fn weapon_damage_multiplier_with_ammo(
        &self,
        class: registry::DamageClass,
        item_type: u16,
        use_ammo: i32,
    ) -> f32 {
        match class {
            registry::DamageClass::Melee => self.melee_damage,
            registry::DamageClass::Ranged => {
                if crate::ammo::is_arrow_ammo(use_ammo) {
                    return (self.ranged_damage / self.ranged_mult_damage
                        + self.arrow_damage_additive_stack)
                        * self.ranged_mult_damage
                        * self.arrow_damage;
                }
                if crate::ammo::is_bullet_ammo(use_ammo) {
                    return self.ranged_damage * self.bullet_damage;
                }
                if crate::ammo::is_specialist_ammo(use_ammo)
                    || crate::ammo::IS_RANGED_SPECIALIST_WEAPONS.contains(&item_type)
                {
                    return self.ranged_damage * self.rocket_damage;
                }
                self.ranged_damage
            }
            registry::DamageClass::Magic => self.magic_damage,
            registry::DamageClass::Summon => self.minion_damage,
            _ => 1.0,
        }
    }

    /// The class crit source (`Player.meleeCrit`/`rangedCrit`/`magicCrit`,
    /// `Player.cs:2430-2432`; base 4 per class).
    pub fn class_crit(&self, class: registry::DamageClass) -> i32 {
        match class {
            registry::DamageClass::Melee => self.melee_crit,
            registry::DamageClass::Ranged => self.ranged_crit,
            registry::DamageClass::Magic => self.magic_crit,
            _ => 0,
        }
    }

    /// `Player.ItemCheck`'s food consumption side (`:14353`'s gate): the
    /// eat plays through the swing; the flag refreshes next tick's scan.
    pub fn eat_food(&mut self) {
        // The vanilla re-eat gate reads `wellFed` (the scan-derived flag);
        // the caller already gated on `!well_fed`, so nothing extra here —
        // the slot's buff grant is the state change.
    }

    /// Direct health loss with no knockback or i-frames, used by drowning and fall
    /// damage (`statLife -= ...` in `Player.cs`).
    fn lose_health(&mut self, amount: i32) {
        if self.hp <= 0 {
            return;
        }
        self.hp = (self.hp - amount).max(0);
        self.hurt_cd = self.hurt_cd.max(0.8);
        // No ramp reset here: the C# direct `statLife -=` paths (drowning, fall
        // damage) leave `lifeRegenTime` untouched.
    }

    /// `UpdateDyes` (`Player.cs:9587-9614`): the per-tick dye cache, refilled
    /// after the `ResetEffects` clears and before the `UpdateEquips` grants
    /// (`Player.Update` `:25491-25633`). The C# zero-fills ~36 `c*` fields
    /// (`:9589-9592`) and `UpdateItemDye` (`:9616-9757`) re-derives the
    /// accessory effect-slot fields from the armor items; the port keeps the
    /// fields with draw consumers — the armor trio reads its dye slots
    /// directly (`cHead/cBody/cLegs = GetEffectiveDye(0..2).dye`,
    /// `:9593-9595`) and the misc five mirror `:9600-9604`. The `wearsRobe →
    /// cLegs = cBody` rule (`:9596-9599`) and the accessory mappings wait for
    /// their ported visuals (docs/ANALYSIS.md "Dye deviations").
    pub(crate) fn update_dye_cache(&mut self) {
        self.c_head = dye::shader_id_for(self.dye_slots[0]);
        self.c_body = dye::shader_id_for(self.dye_slots[1]);
        self.c_legs = dye::shader_id_for(self.dye_slots[2]);
        // `cPet/cLight/cMinecart/cMount/cGrapple = miscDyes[0..4].dye`
        // (`:9600-9604`); `cYorai = cPet` (`:9613`) has no draw consumer.
        self.c_pet = dye::shader_id_for(self.misc_dye_slots[0]);
        self.c_light = dye::shader_id_for(self.misc_dye_slots[1]);
        self.c_minecart = dye::shader_id_for(self.misc_dye_slots[2]);
        self.c_mount = dye::shader_id_for(self.misc_dye_slots[3]);
        self.c_grapple = dye::shader_id_for(self.misc_dye_slots[4]);
    }

    /// One fixed 60 Hz step.
    /// The test-facing tick wrapper: the C# `Player.Update` reads the global
    /// `Main.rand`; the port threads the caller's stream (`update_with_grapple`).
    /// Production always goes through the world-sim's `UpdatePlayers` stage
    /// stream, so this wrapper supplies a scratch seed-0 stream for the
    /// gear-less test players (an armed dodge source would draw from it
    /// deterministically — nothing else).
    pub fn update(&mut self, world: &World, input: PlayerInput) {
        let mut scratch = tr_core::UnifiedRandom::default();
        self.update_with_grapple(world, input, &[], false, false, false, None, &mut scratch)
    }

    /// The full tick with the optional grappling pull (`GrappleMovement`):
    /// while a hook is attached the pull REPLACES the normal movement
    /// (`Player.cs:27395-27399` gates the whole run/jump/wing/dash block).
    /// `item_in_use` is `player.itemAnimation > 0` and `danger_near` the
    /// `NPC.AnyDanger` stand-in — both feed the sleeping state's checks.
    pub fn update_with_grapple(
        &mut self,
        world: &World,
        input: PlayerInput,
        // `Player.grappling[]`/`grapCount` as a `(ProjectileID, center,
        // rotation)` triple per registered hook (`Player.cs:2742`/`:2744`) —
        // the pool rebuilds it on its own pass, so the caller only hands the
        // set over. The rotation is read by the Squirrel Hook's steering alone.
        grapple: &[(u16, Vec2, f32)],
        item_in_use: bool,
        danger_near: bool,
        // `Main.raining` + the wind model: the Cute Fishron's special getter
        // (`Player.cs:4048-4056`) needs both; `update` passes calm defaults.
        raining: bool,
        wind: Option<&tr_core::wind::Wind>,
        // The stage stream the C# swaps in for the player update
        // (`Main.cs:17755-17757`): the world-sim passes the `UpdatePlayers`
        // named stream; the dodge rolls and the lava `Hurt` inside the tick
        // draw from it (S1-A-10 — no new persistent stream).
        rng: &mut dyn tr_core::loot::DropRng,
    ) {
        // `Player.grappling[0] != -1` is rebuilt per tick by the attached
        // hook's AI (`Player.cs:28300` + `Projectile.cs:52860-52864`) — the
        // caller passes the registered set, so derive it here.
        self.grappling_active = !grapple.is_empty();
        // The buff-frame head of `Player.Update` (`Player.cs:25555-25616`): the
        // `buffImmune` reset precedes `UpdateBuffs` and every re-grant; the
        // end-of-update purge consumes the table at `:26011-26016`.
        self.buffs.reset_buff_immune();
        // `Player.Update` → `UpdateBuffs(i)` (`Player.cs:24535`): the timer head runs
        // early in the tick. Nothing grants buffs yet, so this only decrements.
        self.buffs.update_buffs_tick();
        // `Player.Update` → `UpdateImmunity` (`Player.cs:25456`): the spawn
        // i-frames count down before the AI.
        self.update_immunity();
        // `Player.cs:25625-25628`: the Cute Fishron's special window decays
        // one per tick; `Mount.UpdateEffects` re-arms it later in the tick.
        if self.mount_fishron_special_counter > 0.0 {
            self.mount_fishron_special_counter -= 1.0;
        }
        // `Player.cs:24591`: the tag stack's update right after the buff head —
        // the `maxTagEffects` cap plus the per-NPC mark/proc timers.
        self.tag_effects.update_tick();
        // `Player.cs:9881-9884`: a mount-type buff re-affirms its mount every
        // tick and holds `buffTime = 10` — the buff is the source of truth.
        // (The vanilla re-calls `SetMount` with the live type, which is the
        // `Mount.cs:6198` no-op guard; the real re-equip runs when the buff
        // re-appears on its own.)
        for bt in self.buffs.active_types() {
            if let (Some(m), Some(i)) = (
                crate::mount::mount_type_of_buff(bt),
                self.buffs.find_buff_index(bt),
            ) {
                self.buffs.buff_time[i] = 10;
                if !self.mount.is_active() {
                    let wet = self.wet;
                    self.set_mount(m, wet);
                }
            }
        }
        // Buffs whose timer ran out are deleted right after the tick
        // (`Player.cs:25608-25616`) — before any effect can read them.
        self.remove_expired_buffs();
        self.prev_x = self.x;
        self.prev_y = self.y;
        self.prev_gfx_off_y = self.gfx_off_y;
        self.hurt_cd = (self.hurt_cd - 1.0 / 60.0).max(0.0);
        // `Player.Update`'s Hallowed-cooldown decay (`Player.cs:25461-25464`,
        // right after `UpdateImmunity`): one tick off the 1800 shadow-dodge
        // cycle timer per frame.
        if self.shadow_dodge_timer > 0 {
            self.shadow_dodge_timer -= 1;
        }
        // `Player.Update`'s frozen/webbed control lock: the frozen flag
        // zeroes the movement controls (`Player.cs:10057`'s flag gates the
        // run keys), the webbed one zeroes velocity outright (the scan
        // above). Gravity still applies to a frozen player.
        let input = if self.frozen || self.webbed {
            PlayerInput::default()
        } else {
            input
        };
        // The run keys ride on the player like the C#'s `controlLeft`/`controlRight`
        // (input struct → fields early in `Player.Update`).
        self.control_left = input.left;
        self.control_right = input.right;
        self.control_jump = input.jump;
        self.control_down = input.down;
        // `PlayerSittingHelper.UpdateSitting` (`Player.cs:26125`) and
        // `PlayerSleepingHelper.UpdateState` (`:26126`): the stand-up/wake
        // gates run after the controls land, before the movement applies.
        self.update_sitting(world);
        self.update_sleeping(world, item_in_use, danger_near);
        let (cx, cy) = self.center();
        let mid = world.tile((cx / TILE_PX) as i32, (cy / TILE_PX) as i32);
        // `Collision.WetCollision` / `LavaCollision` set the four wet flags
        // (`Player.cs:27728-27910`); the mid-tile liquid kind picks them.
        let liquid = mid.filter(|t| t.liquid > 0).map(|t| t.liquid_kind());
        self.wet = liquid == Some(tr_core::tile::LiquidKind::Water);
        self.lava_wet = liquid == Some(tr_core::tile::LiquidKind::Lava);
        self.honey_wet = liquid == Some(tr_core::tile::LiquidKind::Honey);
        self.shimmer_wet = liquid == Some(tr_core::tile::LiquidKind::Shimmer);
        self.in_water = self.wet || self.honey_wet;
        self.in_lava = self.lava_wet;

        // The fall origin is only reset post-movement now: the landing/rest
        // block (`velocity.Y == 0`, `Player.cs:25383`) and the continuous
        // condition (`Player.cs:25379`). The old pre-movement `vy <= 0` reset
        // measured falls from every upward apex — the vanilla code never does.

        // `Player.cs:25295-25297`: `fallStart2` captures the pre-movement row
        // while rising or resting — it feeds the rail-landing gate
        // (`Minecart.cs:809-813`) and nothing else.
        if self.vy <= 0.0 {
            self.fall_start2 = (self.y / TILE_PX) as i32;
        }

        // `Player.cs:24364-24371`: the per-tick movement defaults;
        // `moveSpeed` multiplies acceleration and the primary gate while
        // `accRunSpeed` takes only the debuff (`Player.cs:26004-26010`).
        self.max_run_speed = 3.0;
        self.run_acceleration = 0.08;
        self.run_slowdown = 0.2;
        self.acc_run_speed = self.max_run_speed;
        // `Player.cs:24366-24367`: the jump statics reset per tick (the C#
        // writes the statics `Player.jumpHeight = 15` / `jumpSpeed = 5.01f`).
        self.jump_height = 15;
        self.jump_speed = 5.01;
        // `Player.cs:24380-24437`: the wet/shimmer statics rebase — the C#
        // runs it right after the defaults and BEFORE `UpdateJumpHeight`'s
        // max-lift (`:25656`), so a wet assignment (30/6.01 water) survives
        // the lift and the later max can only raise it. The rebased pair
        // feeds the plain integration arm below at full strength.
        let (tick_gravity, tick_max_fall) = {
            let flags = WetStaticsFlags {
                wet: self.wet,
                honey_wet: self.honey_wet,
                lava_wet: self.lava_wet,
                shimmer_wet: self.shimmer_wet,
                ..WetStaticsFlags::default()
            };
            let (g, mf, jh, js) =
                wet_gravity_and_cap(GRAVITY, MAX_FALL, self.jump_height, self.jump_speed, flags);
            self.jump_height = jh;
            self.jump_speed = js;
            (g, mf)
        };
        // `UpdateJumpHeight` (`:19587-19593`): the balloon/jump-boost lift is
        // a MAX over the rebased statics, not an assignment.
        if self.jump_boost {
            self.jump_height = self.jump_height.max(20);
            self.jump_speed = self.jump_speed.max(6.51);
        }
        self.move_speed = 1.0;
        self.strongest_move_speed_debuff = 1.0;
        // `Player.cs:24371-24374`: a mounted cart keeps `onWrongGround` across
        // the tick (the on-track check below rewrites it); everyone else
        // clears it at the defaults.
        if !self.cart_mounted {
            self.on_wrong_ground = false;
        }
        // `Player.ResetEffects` + `UpdateBuffs`' slot scan (`Player.cs:18640/
        // :9855`, the debuff rows `:10039-10061`, Weak `:11873-11878`, Webbed
        // `:11865-11883`, Darkness→blind `:9969-9971`): the flags derive per
        // slot, Weak's stat mods subtract per slot. The melee base recalc
        // (`UpdateDamageRecalc`) has no other ported modifier yet, so the
        // reset stands in for it (deviation until M3-EQUIP-3).
        self.slow = false;
        self.chilled = false;
        self.frozen = false;
        self.webbed = false;
        // `Player.cs:18786`: justJumped clears at the tick head; the auto-jump
        // flag re-derives per tick from the mount head-set (`Mount.cs:4797`).
        self.just_jumped = false;
        self.auto_jump = false;
        // `Mount.CheckMountBuff` via `ResetEffects` (`Player.cs:18832` ->
        // `Mount.cs:5275-5281`): the buff gone means dismount.
        if self.mount.is_active()
            && self
                .mount
                .data()
                .is_some_and(|d| self.buffs.find_buff_index(d.buff).is_none())
        {
            self.try_dismount(world);
        }
        self.blind = false;
        self.weak_stacks = 0;
        self.melee_damage = 1.0;
        self.ranged_damage = 1.0;
        self.magic_damage = 1.0;
        self.minion_damage = 1.0;
        // `:18338`/`:18396-18399` — the ranged effective-damage components.
        self.ranged_mult_damage = 1.0;
        self.arrow_damage = 1.0;
        self.arrow_damage_additive_stack = 0.0;
        self.bullet_damage = 1.0;
        self.rocket_damage = 1.0;
        // `ResetEffects`' summon budget bases (`Player.cs:18448-18449`): the
        // cap buffs 110/150/348 (`:10175-10183`, the summon arms pass) and
        // the armor/held rows (`:13258-13948`/`:15324-15329`,
        // `grant_summon_caps`) re-add per tick.
        self.max_minions = 1;
        self.max_turrets = 1;
        // The 28 summon keep-alive flags (`:18620-18646`+`:18741`) — the
        // arms table is the single source of the flag set.
        self.clear_summon_flags();
        // `ResetProjectileCaches` (`:18810`): the recount inputs zeroed;
        // `UpdateProjectileCaches` recounts them at the `:25565` seam (the
        // summon arms pass in `world_sim/player.rs`).
        self.reset_projectile_caches();
        self.mana_regen_bonus = 0;
        self.well_fed = false;
        self.pick_speed_bonus = 0.0;
        self.melee_speed_bonus = 0.0;
        // `ResetEffects`' `statDefense = 0` (`Player.cs:18327`): the buff arms
        // below and `GrantArmorBenefits` (`:13121`, the equipment pass at the
        // bottom of this block) re-add per tick. Before M3-EQUIP-5 the field
        // was assigned *outside* the tick (from `world_sim`), which silently
        // dropped every buff and prefix defense arm.
        self.stat_defense = 0;
        // `:18334`: `manaCost = 1f` — the reductions re-apply below.
        self.mana_cost = 1.0;
        // `:18497`: `endurance = 0f` — the rows below re-apply.
        self.endurance = 0.0;
        // `:18323-18324`: the penetration reset.
        self.armor_penetration = 0;
        self.melee_armor_penetration = 0;
        // `:18663`: `lavaMax = 0` — the charm rows below re-add it.
        self.lava_max = 0;
        // `:18376` + `:18726-18727`: the thorns flag clear.
        self.thorns = 0.0;
        self.cactus_thorns = false;
        self.turtle_thorns = false;
        // The dodge-family clears (`ResetEffects`): `shadowDodge = false`
        // (`:18440`), `brainOfConfusionItem = null` (`:18617`),
        // `blackBelt = false` / `mysticSashDodge = false` (`:18661-18662`),
        // `longInvince = false` (`:18719`) — the fold's equipped/buff arms
        // below re-arm from the live sources. The brain's animation counter
        // decays here like the C# (`:18794-18797`).
        self.shadow_dodge = false;
        self.brain_of_confusion = false;
        self.black_belt = false;
        self.mystic_sash_dodge = false;
        self.long_invince = false;
        if self.brain_of_confusion_dodge_animation_counter > 0 {
            self.brain_of_confusion_dodge_animation_counter -= 1;
        }
        // The luck inputs ResetEffects owns (`Player.cs:18732` stinky,
        // `:18812` equipmentBasedLuckBonus, `:18814` luckPotion): zeroed here,
        // re-derived in the buff scan below (the potion/kite/stinky arms).
        self.luck_state.stinky = false;
        self.luck_state.equipment_based_luck_bonus = 0.0;
        self.luck_state.luck_potion = 0;
        // `ResetEffects`' crit bases (`Player.cs:18640+`): the per-slot adds
        // (the fed tiers, the accessory states) stack on top per tick.
        self.melee_crit = 4;
        self.ranged_crit = 4;
        self.magic_crit = 4;
        // `GrantArmorBenefits` (' armor/accessory rows, `Player.cs:13121`):
        // every equipped wearable's `Item.defense` — the three armor slots AND
        // the seven functional accessory slots (Shackle +1, Cobalt Shield +1,
        // Ankh Shield +4, …), then the completed set's bonus
        // (`ArmorSetBonuses.GetCompleteSet(...).Effect`).
        for &kind in self.equipped[..10].iter() {
            if kind == 0 {
                continue;
            }
            if let Some(def) = crate::registry::get(kind) {
                self.stat_defense += def.defense as i32;
            }
            // The piece's own damage/crit/speed row (`GrantArmorBenefits`'
            // `switch (armorPiece.type)`, `Player.cs:13126-13950`).
            if let Some(piece) = tr_core::armor_sets::armor_stats(kind) {
                self.melee_damage += piece.melee_damage;
                self.ranged_damage += piece.ranged_damage;
                self.magic_damage += piece.magic_damage;
                self.minion_damage += piece.minion_damage;
                self.melee_crit += piece.melee_crit;
                self.ranged_crit += piece.ranged_crit;
                self.magic_crit += piece.magic_crit;
                self.move_speed += piece.move_speed;
                self.melee_speed_bonus += piece.melee_speed;
            }
            // `GrantArmorBenefits`' `switch (armorPiece.type)` rows that touch
            // `manaCost` (`Player.cs:13118-13760`): the Magic Hat and Wizard
            // Hat robes (1282-1287, −0.05…−0.15 by tier), the Crystal
            // Assassin head (4982, −0.1), the Spectre hood/mask (1003 −0.17,
            // 2189 −0.13), the Mystic Robe (2279, −0.1), the Apprentice/
            // Dark Artist rows (3797 −0.1, 3875 −0.15).
            self.mana_cost -= match kind {
                1282 => 0.05,
                1283 => 0.07,
                1284 => 0.09,
                1285 => 0.11,
                1286 => 0.13,
                1287 => 0.15,
                2279 | 3797 | 4982 => 0.1,
                2189 => 0.13,
                3875 => 0.15,
                1003 => 0.17,
                _ => 0.0,
            };
        }
        // `:12988-13003`: the Celestial Stone bundle. The flag is set by the
        // three stones (`:15412-15424`): 1865 (Celestial Stone) and 3110
        // (Celestial Shell) always, 899 (Sun Stone) by day, 900 (Moon Stone) by
        // night or during an eclipse.
        self.sky_stone_effects = self.equipped[..10].contains(&1865)
            || self.equipped[..10].contains(&3110)
            || (self.equipped[..10].contains(&899) && world.day_time)
            || (self.equipped[..10].contains(&900) && (!world.day_time || world.eclipse));
        if self.sky_stone_effects {
            self.stat_defense += 4;
            self.melee_speed_bonus += 0.1;
            self.melee_damage += 0.1;
            self.melee_crit += 2;
            self.ranged_damage += 0.1;
            self.ranged_crit += 2;
            self.magic_damage += 0.1;
            self.magic_crit += 2;
            // `pickSpeed -= 0.15f` — the port's bonus field is the negation.
            self.pick_speed_bonus += 0.15;
            self.minion_damage += 0.1;
        }
        // The immunity rows: the worn accessories (`BUFF_IMMUNITY_ROWS`) and
        // the completed Snow/Molten sets (`set_buff_immunity`).
        for (kind, ids) in BUFF_IMMUNITY_ROWS {
            if self.equipped[..10].contains(kind) {
                for id in *ids {
                    if let Some(slot) = self.buffs.buff_immune.get_mut(*id as usize) {
                        *slot = true;
                    }
                }
            }
        }
        let (armor_head, armor_body, armor_legs) = (
            self.equipped[0] as i32,
            self.equipped[1] as i32,
            self.equipped[2] as i32,
        );
        self.stat_defense +=
            tr_core::armor_sets::set_bonus_defense(armor_head, armor_body, armor_legs);
        for id in tr_core::armor_sets::set_buff_immunity(armor_head, armor_body, armor_legs) {
            if let Some(slot) = self.buffs.buff_immune.get_mut(*id as usize) {
                *slot = true;
            }
        }
        // The completed set's *stat* row (`ArmorSetBonuses.Benefits` — the
        // `ArmorSetBonuses.Initialize` triples, `:396-500`): the damage/crit/
        // speed/mana/minion scalars of Necro, Pumpkin, Ninja, Wizard, Magic Hat,
        // Molten, Frost, Mining, the Cobalt/Mythril/Adamantite head variants,
        // Crystal Assassin, Tiki, Spooky, Spider and Bee.
        if let Some((_set, set)) =
            tr_core::armor_sets::set_stats(armor_head, armor_body, armor_legs)
        {
            self.melee_damage += set.melee_damage;
            self.ranged_damage += set.ranged_damage;
            self.magic_damage += set.magic_damage;
            self.minion_damage += set.minion_damage;
            self.melee_crit += set.melee_crit;
            self.ranged_crit += set.ranged_crit;
            self.magic_crit += set.magic_crit;
            self.move_speed += set.move_speed;
            self.melee_speed_bonus += set.melee_speed;
            self.pick_speed_bonus += set.pick_speed;
            self.mana_max += set.mana_max;
            self.max_minions += set.max_minions;
            self.mana_cost += set.mana_cost;
            self.endurance += set.endurance;
            self.thorns += set.thorns;
            self.cactus_thorns |= set.cactus_thorns;
            self.turtle_thorns |= set.turtle_thorns;
            if set.dash_type != 0 {
                self.dash_type = set.dash_type;
            }
        }
        for bt in self.buffs.active_types() {
            match bt {
                22 => self.blind = true, // `BuffID.Darkness`
                32 => self.slow = true,  // `BuffID.Slow`
                // `Player.UpdateBuffs`' shadow-dodge arm (`Player.cs:10075-
                // 10077`): the Hallowed set's buff 59 arms `shadowDodge` for
                // the tick; the buff is granted by the set bonus's
                // `onHitDodge` scan, which is an armor-set row (out of scope).
                59 => self.shadow_dodge = true,
                // `:10201-10204` — `BuffID.Endurance` (114): +10%.
                114 => self.endurance += 0.1,
                // `:10015-10017` — `BuffID.Sharpened` (159): the melee-only
                // +12 armor penetration.
                159 => self.melee_armor_penetration += 12,
                // `:10461-10470` — the Ice Barrier (62, the Frozen Turtle
                // Shell): +25% while at or below half life.
                62 => {
                    if self.hp as f32 <= self.max_hp as f32 * 0.5 {
                        self.endurance += 0.25;
                    }
                }
                // `:11887-11892` — `BuffID.MagicPower` (29): the crit, the
                // damage, the 20 mana and the −0.02 cost.
                29 => {
                    self.magic_crit += 2;
                    self.magic_damage += 0.05;
                    self.mana_max += 20;
                    self.mana_cost -= 0.02;
                }
                33 => {
                    self.weak_stacks += 1; // `BuffID.Weak`
                    self.melee_damage -= 0.051;
                    self.move_speed -= 0.1;
                    // `:11915`: `statDefense -= 4` per Weak slot.
                    self.stat_defense -= 4;
                }
                // `:9962` — `BuffID.Ironskin`: `statDefense += 8`.
                5 => self.stat_defense += 8,
                // `:10064` — `BuffID.Ichor`: `statDefense -= 15`.
                69 => self.stat_defense -= 15,
                // `:10108` — buff 215 (`BuffID.Warmth`'s neighbour row in the
                // C#: `statDefense += 5`).
                215 => self.stat_defense += 5,
                // `:10306-10312` — `BuffID.DryadsWard`: `statDefense += 8`,
                // `lifeRegen += 6` (the regen half has no consumer yet) and
                // `thorns += 0.5` while under 1.
                165 => {
                    self.stat_defense += 8;
                    if self.thorns < 1.0 {
                        self.thorns += 0.5;
                    }
                }
                // `:10005-10011` — `BuffID.Thorns` (14): `thorns = 1f` if lower.
                14 => {
                    if self.thorns < 1.0 {
                        self.thorns = 1.0;
                    }
                }
                // `:11921` — `BuffID.Tipsy`: `statDefense -= 4`.
                25 => self.stat_defense -= 4,
                46 => self.chilled = true, // `BuffID.Chilled`
                47 => self.frozen = true,  // `BuffID.Frozen`
                // `:11001-11004`: the Sunflower buff's movement tier —
                // `moveSpeed += 0.1f; moveSpeed *= 1.1f` per slot.
                146 => {
                    self.move_speed += 0.1;
                    self.move_speed *= 1.1;
                }
                // `:9932-9934`: the Star in a Bottle's mana-regen add.
                158 => self.mana_regen_bonus += 10,
                // The WellFed tiers (`:11931-11969`): the per-slot stat adds —
                // the defense rows (26 +2, 206 +3, 207 +4) since M3-EQUIP-5;
                // the minion halves wait for M4-MINION.
                26 => {
                    self.well_fed = true;
                    // `:11934`: `statDefense += 2`.
                    self.stat_defense += 2;
                    self.melee_crit += 2;
                    self.melee_damage += 0.05;
                    self.melee_speed_bonus += 0.05;
                    self.ranged_crit += 2;
                    self.ranged_damage += 0.05;
                    self.magic_crit += 2;
                    self.magic_damage += 0.05;
                    self.move_speed += 0.2;
                    self.pick_speed_bonus += 0.05;
                }
                206 => {
                    self.well_fed = true;
                    // `:11950`: `statDefense += 3`.
                    self.stat_defense += 3;
                    self.melee_crit += 3;
                    self.melee_damage += 0.075;
                    self.melee_speed_bonus += 0.075;
                    self.ranged_crit += 3;
                    self.ranged_damage += 0.075;
                    self.magic_crit += 3;
                    self.magic_damage += 0.075;
                    self.move_speed += 0.3;
                    self.pick_speed_bonus += 0.1;
                }
                207 => {
                    self.well_fed = true;
                    // `:11966`: `statDefense += 4`.
                    self.stat_defense += 4;
                    self.melee_crit += 4;
                    self.melee_damage += 0.1;
                    self.melee_speed_bonus += 0.1;
                    self.ranged_crit += 4;
                    self.ranged_damage += 0.1;
                    self.magic_crit += 4;
                    self.magic_damage += 0.1;
                    self.move_speed += 0.4;
                    self.pick_speed_bonus += 0.15;
                }
                149 => self.webbed = true, // `BuffID.Webbed`
                // `:10276-10280`: buff 120 (Stinky) — the `talkNPC = -1`
                // dialogue half is not ported; the luck −0.25 flag is.
                120 => self.luck_state.stinky = true,
                // `:10285-10301`: buff 257 (Lucky) — the potion tier by
                // *remaining* time; `RecalculateLuck` weights it
                // `(int)luckPotion · 0.1` (max +0.3).
                257 => {
                    if let Some(i) = self.buffs.find_buff_index(257) {
                        let t = self.buffs.buff_time[i];
                        self.luck_state.luck_potion = if t > 36_000 {
                            3
                        } else if t > 18_000 {
                            2
                        } else {
                            1
                        };
                    }
                }
                // `:9920-9924`: buff 383 (Kite) — the tier is
                // `buffTime / 10800` clamped 0..3, assigned only while the
                // buff is active: the C# never zeroes the field, so it stays
                // stale after expiry (no `kiteLuckLevel = 0` exists).
                383 => {
                    if let Some(i) = self.buffs.find_buff_index(383) {
                        self.luck_state.kite_luck_level =
                            (self.buffs.buff_time[i] / 10_800).clamp(0, 3) as u8;
                    }
                }
                _ => {}
            }
        }
        if self.slow {
            self.strongest_move_speed_debuff = self.strongest_move_speed_debuff.min(0.5);
        }
        if self.chilled {
            self.strongest_move_speed_debuff = self.strongest_move_speed_debuff.min(0.75);
        }
        if self.webbed {
            // `:11866-11872`: velocity zeroed (airborne keeps a 1e-6 fall),
            // jump height 0, gravity 0 — the player hangs in the web.
            self.vx = 0.0;
            self.vy = if self.vy != 0.0 { 1e-6 } else { 0.0 };
        }
        self.move_speed *= self.strongest_move_speed_debuff;
        // `ResetEffects`' equipment-flag clears (`Player.cs:18348-18788`) and
        // the accessory grant pass - `UpdateEquips` -> `ApplyEquipFunctional`
        // (`Player.cs:12892`/`:14368`, the slots 3..10 loop :14602-14604):
        // the flags reset, then the equipped kinds re-set them per tick.
        // `UpdateDyes` sits between the two in `Player.Update`
        // (`:25492` before `UpdateEquips`' `:25633`), so the dye cache
        // refills here, against this tick's slots.
        self.update_dye_cache();
        self.jump_boost = false;
        self.no_fall_dmg = false;
        self.ice_skate = false;
        self.desert_boots = false;
        self.rocket_boots = 0;
        self.no_knockback = false;
        self.has_jump_options = [false; 9];
        // `:18701-18703`: `wingsLogic = 0; hasWings = false; wingTimeMax = 0`
        // — the granting wing re-sets all three below, so taking the wings off
        // ends the flight (and its budget) on the next tick.
        self.wings_logic = 0;
        self.has_wings = false;
        self.wing_time_max = 0;
        self.accessory_life_regen = 0;
        let (kinds, prefixes) = (self.equipped, self.equipped_prefix);
        for (slot, prefix) in kinds[3..10].iter().zip(&prefixes[3..10]) {
            // `UpdateEquips`' accessory arm (`Player.cs:12925-12930`):
            // `GrantPrefixBenefits` rides the accessory slot's rolled prefix
            // before the kind grants below.
            crate::prefix::grant_prefix_benefits(*prefix, self);
            match *slot {
                // The bottles (`Player.cs:14997-15001`, :14793-14811, :15001):
                53 => self.has_jump_options[ExtraJump::Cloud.index()] = true,
                857 => self.has_jump_options[ExtraJump::Sandstorm.index()] = true,
                987 => self.has_jump_options[ExtraJump::Blizzard.index()] = true,
                1724 => self.has_jump_options[ExtraJump::Fart.index()] = true,
                3201 => self.has_jump_options[ExtraJump::Sail.index()] = true,
                // The bottle combos (`:14822-14835`): 1164 (Bundle of
                // Balloons) and 5331 (its Amber/Glass variant) carry all three
                // bottles at once.
                1164 | 5331 => {
                    self.has_jump_options[ExtraJump::Cloud.index()] = true;
                    self.has_jump_options[ExtraJump::Sandstorm.index()] = true;
                    self.has_jump_options[ExtraJump::Blizzard.index()] = true;
                    self.jump_boost = true;
                }
                // Every other bottle/balloon carries its own option *and*
                // `jumpBoost` (`:14806/:15093-15103`, `:14797-14860`): `399`
                // Shiny Red Balloon and `1250` Cloud in a Balloon the Cloud
                // row, `1251`/`1163` the Blizzard one, `983`/`1252` the
                // Sandstorm one, `1863`/`3250` the Fart one, `3252`/`3241` the
                // Sail one. `3251` (honeycomb balloon) and `1249` (Honey
                // Comb) carry only the boost, and so does `159`/`3225` (the
                // Aglet-style `jumpSpeedBoost` rows).
                399 | 1250 => {
                    self.has_jump_options[ExtraJump::Cloud.index()] = true;
                    self.jump_boost = true;
                }
                1251 | 1163 => {
                    self.has_jump_options[ExtraJump::Blizzard.index()] = true;
                    self.jump_boost = true;
                }
                983 | 1252 => {
                    self.has_jump_options[ExtraJump::Sandstorm.index()] = true;
                    self.jump_boost = true;
                }
                1863 | 3250 => {
                    self.has_jump_options[ExtraJump::Fart.index()] = true;
                    self.jump_boost = true;
                }
                3252 | 3241 => {
                    self.has_jump_options[ExtraJump::Sail.index()] = true;
                    self.jump_boost = true;
                }
                159 | 1249 | 3225 | 3251 => self.jump_boost = true,
                // The boots (`:15005-15008` Hermes / `:15013-15016` Flurry /
                // `:15017-15020` Amphibian — `accRunSpeed = 6f`):
                54 | 1579 | 3200 => self.acc_run_speed = 6.0,
                // The Rocket Boots (`:15021-15024` - `rocketBoots = 1`).
                128 => self.rocket_boots = 1,
                // The Spectre Boots (`:15213-15216` - tier 2).
                405 => {
                    self.acc_run_speed = 6.0;
                    self.rocket_boots = 2;
                }
                // The Lightning Boots (`:15376-15380` - tier 2, 6.75 speed,
                // +8% move).
                898 => {
                    self.acc_run_speed = 6.75;
                    self.rocket_boots = 2;
                    self.move_speed += 0.08;
                }
                // The Frostspark Boots (`:15382-15388` - tier 3 and the ice
                // skates).
                1862 => {
                    self.acc_run_speed = 6.75;
                    self.rocket_boots = 3;
                    self.move_speed += 0.08;
                    self.ice_skate = true;
                }
                // The Terraspark Boots (`:15390-15396` - tier 4).
                5000 => {
                    self.acc_run_speed = 6.75;
                    self.rocket_boots = 4;
                    self.move_speed += 0.08;
                    self.ice_skate = true;
                    // The Terraspark row (`:15168-15174`): the water/fire walk
                    // and the lava charm's 420 ticks.
                    self.lava_max += 420;
                }
                // The Hellfire Treads (`:15397-15401` - tier 5, 6f).
                4874 => {
                    self.acc_run_speed = 6.0;
                    self.rocket_boots = 5;
                }
                _ if Self::wing_slot(*slot as i32) > 0 => {
                    // `:15593-15602`: the accessory's `wingSlot` is both the
                    // gameplay wing (`wingsLogic`) and the look (`wings`, same
                    // field in the port), and `:15453` takes the flight budget
                    // from the same id's `ArmorIDs.Wing.Sets.Stats`.
                    let slot_id = Self::wing_slot(*slot as i32);
                    self.wings_logic = slot_id;
                    self.has_wings = true;
                    self.wing_time_max = Self::wing_stats(slot_id).0;
                }
                // The Titan Glove's melee-speed row (`:15103-15107`); its
                // `autoReuseGlove` flag has no port consumer yet.
                211 => self.melee_speed_bonus += 0.12,
                // The damage/crit accessories (`ApplyEquipFunctional`, the
                // `Player.cs:15223-15560` block): the four class emblems at
                // +15% (Sorcerer 489, Warrior 490, Ranger 491, Summoner 2998),
                // the Avenger Emblem's +12% to all four (935), the melee-speed
                // gloves (897/936/1343 — the Fire Gauntlet adds +12% melee
                // damage), the Sniper Scope's +10 ranged crit/+10% ranged
                // damage (1858), the Celestial Stone-style +8 crit/+10% all
                // (1301), the Sorcerer's Magnet +15% magic (2220), the minion
                // emblems +15%/+10% with their extra slot (1167/1864/1845),
                // the Ankh/Stone rows' crit-and-damage mix (3015) and the
                // Empress Brooch's +7.5% move (4989).
                489 => self.magic_damage += 0.15,
                490 => self.melee_damage += 0.15,
                491 => self.ranged_damage += 0.15,
                2998 => self.minion_damage += 0.15,
                935 => {
                    self.melee_damage += 0.12;
                    self.ranged_damage += 0.12;
                    self.magic_damage += 0.12;
                    self.minion_damage += 0.12;
                }
                897 => self.melee_speed_bonus += 0.12,
                936 => {
                    self.melee_speed_bonus += 0.12;
                    self.melee_damage += 0.12;
                }
                1343 => {
                    self.melee_speed_bonus += 0.12;
                    self.melee_damage += 0.12;
                }
                1858 => {
                    self.ranged_crit += 10;
                    self.ranged_damage += 0.1;
                }
                1301 => {
                    self.melee_crit += 8;
                    self.ranged_crit += 8;
                    self.magic_crit += 8;
                    self.melee_damage += 0.1;
                    self.ranged_damage += 0.1;
                    self.magic_damage += 0.1;
                    self.minion_damage += 0.1;
                }
                3015 => {
                    self.melee_crit += 5;
                    self.ranged_crit += 5;
                    self.magic_crit += 5;
                    self.melee_damage += 0.05;
                    self.ranged_damage += 0.05;
                    self.magic_damage += 0.05;
                    self.minion_damage += 0.05;
                }
                2220 => self.magic_damage += 0.15,
                1167 => self.minion_damage += 0.15,
                // 1864/1845 also carry `maxMinions++` in the C#; the budget
                // half is `grant_summon_caps`' (`world_sim/player.rs`), so only
                // the damage row lands here (the C# total counts once).
                1864 => self.minion_damage += 0.15,
                1845 => self.minion_damage += 0.1,
                4989 => self.move_speed += 0.075,
                // The penetration rows: the Shark Tooth Necklace (3212,
                // `:13286-13289`) and the Honey Comb (4007, `:14485-14489`)
                // add a flat 5 each.
                3212 | 4007 => self.armor_penetration += 5,
                // The Lava Charm family (`:15168-15183`): 906 (Lava Charm),
                // 4038 (Molten Charm), 3999 (Molten Skull Rose) and 4003
                // (Lava Waders) each add 420 ticks of lava immunity; 908
                // (Lava Waders' older row) and 5000 (Terraspark Boots) also
                // set the water/fire walk and the lava rose.
                906 | 3999 | 4003 | 4038 => self.lava_max += 420,
                // (The Terraspark Boots' own row rides its boot arm above —
                // a duplicate pattern here would be unreachable.)
                908 => self.lava_max += 420,
                // The Worm Scarf (`:15081-15084`): +17% endurance.
                3224 => self.endurance += 0.17,
                // The Mana Flower (`:12962-12965`, `manaFlower` −0.08) and the
                // Nature's Gift (`:15110-15113`, −0.06).
                555 => self.mana_cost -= 0.08,
                223 => self.mana_cost -= 0.06,
                // The Ice Skates (`:15089-15092`).
                950 => self.ice_skate = true,
                // The Aglet / the Anklet of the Wind (`:15114-15121`).
                285 => self.move_speed += 0.05,
                212 => self.move_speed += 0.1,
                // `Player.cs:15029-15033` + the horseshoe balloons: the fall
                // damage immunity.
                158 | 1250 | 1251 | 1252 | 3250 | 3251 | 3252 => self.no_fall_dmg = true,
                // `Player.cs:15025-15028` - the Cobalt Shield.
                156 => self.no_knockback = true,
                // The dodge family (`ApplyEquipFunctional`): the Black Belt
                // 963 (`:15059-15062`) and the Master Ninja Gear 984
                // (`:15063-15067`, whose dash/spiked-boots rows ride the
                // dash granters) both arm `blackBelt`; the Mystic Arts Sash
                // 6189 (`:14632-14635`) arms `mysticSashDodge` and the Brain
                // of Confusion 3223 (`:15085-15089`) arms the (bool
                // stand-in for the) `brainOfConfusionItem` reference.
                963 | 984 => self.black_belt = true,
                6189 => self.mystic_sash_dodge = true,
                3223 => self.brain_of_confusion = true,
                // `UpdateLifeRegen`'s `lifeRegen` base (the Band's
                // `Item.lifeRegen = 2`, `Item.cs:1974` -> `Player.cs:13122`).
                // (The `Item.lifeRegen` rows are generic now — the loop after
                // the accessory match adds every worn item's value.)
                _ => {}
            }
        }
        // `Player.cs:25927-25930`: the defense floor — a pile of debuffs (Ichor
        // −15, Weak −4 a slot, Tipsy −4) may overshoot the armor's total, and
        // the C# never lets `statDefense` go negative (a negative value would
        // *raise* incoming damage through `CalculateDamagePlayersTake`).
        self.stat_defense = self.stat_defense.max(0);
        // `Player.lifeRegen`'s equipment rows (`UpdateLifeRegen`'s input, fed
        // through `accessory_life_regen` -> `extra_regen`): the armor pieces
        // (`:13148` Stardust/Solar +4, `:13192` Spooky +8, `:13886-13900` the
        // Solar Flare pieces +2 each — the C# also adds `Item.lifeRegen`,
        // which the item table does not carry yet), the Regeneration buff
        // (2, +4 at `:9950`) and the Dryad's Ward (165, +6 at `:10305`), and
        // the Celestial Stone bundle's +2 (`:12991`).
        for &kind in self.equipped[..10].iter() {
            // `:13122`: `lifeRegen += armorPiece.lifeRegen` — the item's own
            // row (the Band of Regeneration 2, Charm of Myths 2, Mana
            // Regeneration Band 2, Restoration Shield 2, Mystic Arts Sash 2
            // are the whole stock table).
            if let Some(def) = crate::registry::get(kind) {
                self.accessory_life_regen += def.life_regen;
            }
            self.accessory_life_regen += match kind {
                3800 => 4,
                3872 => 8,
                2763 | 2764 | 2765 => 2,
                _ => 0,
            };
        }
        for bt in self.buffs.active_types() {
            match bt {
                2 => self.accessory_life_regen += 4,
                165 => self.accessory_life_regen += 6,
                _ => {}
            }
        }
        if self.sky_stone_effects {
            self.accessory_life_regen += 2;
        }
        // The debug pins stand in for the granting equipment - re-applied
        // after the clear (the vanilla: the grants re-set per tick).
        for i in 0..9 {
            if self.pinned_jumps[i] {
                self.has_jump_options[i] = true;
            }
        }
        if self.pinned_balloon {
            self.jump_boost = true;
        }
        if self.pinned_rocket_boots > self.rocket_boots {
            self.rocket_boots = self.pinned_rocket_boots;
        }
        // The `TR_MOUNT` pin equips while nothing else did this tick — a real
        // saddle (or its still-live buff) wins, matching the refresh order.
        if let Some(m) = self.pinned_mount {
            if !self.mount.is_active() {
                let wet = self.wet;
                self.set_mount(m, wet);
            }
        }
        // `Player.UpdateBuffs`' scene-buff grants (`Player.cs:25525-25555`):
        // the 2-tick indicator buffs refresh from the scan; the effect cases
        // above read the previous tick's grant, exactly like the source.
        if self.auras.shadow_candle {
            self.buffs.add_buff(350, 2);
        }
        if self.auras.campfire {
            self.buffs.add_buff(87, 2);
        }
        if self.auras.cat_bast {
            self.buffs.add_buff(215, 2);
        }
        if self.auras.star_in_bottle {
            self.buffs.add_buff(158, 2);
        }
        if self.auras.heart_lantern {
            self.buffs.add_buff(89, 2);
        }
        if self.auras.sunflower {
            self.buffs.add_buff(146, 2);
        }
        // `Player.Update` → `UpdateLuck` (`Player.cs:25645` → `:29041`): the
        // decay timers + `RecalculateLuck`, after the buff effects derived
        // stinky/luckPotion/kiteLuckLevel above and before movement.
        self.update_luck(world);
        self.run_acceleration *= self.move_speed;
        self.max_run_speed *= self.move_speed;
        self.acc_run_speed *= self.strongest_move_speed_debuff;
        // `Player.cs:26522-26579`: the floor-tile modifiers, one else-if chain.
        // The flags come from the previous tick's `FloorVisuals` sample — one
        // tick stale, like the original's call order. The empress-brooch /
        // Magiluminescence / shadow-armor boosts land with M3-EQUIP-3.
        if self.sticky {
            self.max_run_speed *= 0.25;
            self.run_acceleration *= 0.25;
            self.run_slowdown *= 2.0;
            self.vx = self.vx.clamp(-self.max_run_speed, self.max_run_speed);
        } else if self.powerrun {
            self.max_run_speed *= 3.5;
            self.run_slowdown *= 2.0;
        } else if self.running_on_sand && self.desert_boots {
            let n = 1.75;
            self.max_run_speed *= n;
            self.acc_run_speed *= n;
            self.run_acceleration *= n;
            self.run_slowdown *= n;
        } else if self.slippy2 {
            self.run_acceleration *= 0.6;
            self.run_slowdown = 0.0;
            if self.ice_skate {
                self.run_acceleration *= 3.5;
                self.max_run_speed *= 1.25;
            }
        } else if self.slippy {
            self.run_acceleration *= 0.7;
            if self.ice_skate {
                self.run_acceleration *= 3.5;
                self.max_run_speed *= 1.25;
            } else {
                self.run_slowdown *= 0.1;
            }
        }
        // `Player.cs:25256-25287`: the hover latches — a down press edge
        // latches hover-down, an up edge swaps to hover-up; landing clears
        // both. `Settings.HoverControl == Hold` clears them every tick; the
        // vanilla default is the Toggle mode.
        if input.down && !self.down_prev {
            if self.try_keeping_hovering_up {
                self.try_keeping_hovering_up = false;
            } else {
                self.try_keeping_hovering_down = true;
            }
        }
        self.down_prev = input.down;
        if self.vy == 0.0 {
            self.try_keeping_hovering_up = false;
            self.try_keeping_hovering_down = false;
        }
        // `Player.cs:27395-27399`: the whole run/jump/wing/dash movement
        // block is bypassed while grappling — `GrappleMovement` replaces the
        // velocity below.
        let grappling = self.grappling_active;
        let dir = input.right as i32 - input.left as i32;
        // `Player.cs:26688` calls `HorizontalMovement()` unconditioned, so its
        // direction block (`:19835-19843`) applies even on the ticks the
        // port's grapple gate bypasses the call — the facing update for those
        // ticks stays here (the pull replaces the velocity afterwards, the
        // facing keeps following, with the cart lock intact).
        if grappling && dir != 0 {
            let roll = (self.vx > 0.0) as i32 - (self.vx < 0.0) as i32;
            if !self.cart_mounted || roll == dir {
                self.facing = dir;
            }
        }
        // `Player.cs:26623-26687`: the mount stat override. The base cart is
        // not additive and cannot use wings (`MovementStatsAreAdditive` false,
        // `CanUseWings` false), so the stats are replaced outright and the
        // wing/rocket gear is stripped for the tick.
        if self.cart_mounted {
            self.run_slowdown = 0.2; // `:26625`
            self.rocket_boots = 0; // `:26628`
            self.wings_logic = 0; // `:26631-26633` (`wings = 0` has no port field)
            self.max_run_speed = CART_RUN_SPEED; // `:26652` mount.RunSpeed
            self.acc_run_speed = CART_DASH_SPEED; // `:26653` mount.DashSpeed
            self.run_acceleration = CART_ACCELERATION; // `:26654` mount.Acceleration
            // `Player.cs:19628-19629` + `UpdateJumpHeight`: the cart is in no
            // `JumpHeight`/`JumpSpeed` case (`Mount.cs:2810-2858`) — fixed
            // 15/5.15, no |vx| scaling.
            self.jump_height = CART_JUMP_HEIGHT;
            self.jump_speed = CART_JUMP_SPEED;
            // `Player.cs:26665-26681`: the on-track check while resting — the
            // friction switch and the dismount-on-direction-press edge.
            if self.vy == 0.0 {
                if !minecart::on_track(
                    world,
                    Vec2::new(self.x, self.y),
                    WIDTH as i32,
                    CART_MOUNT_HEIGHT as i32,
                ) {
                    self.full_rotation = 0.0; // `:26668`
                    self.on_wrong_ground = true; // `:26669`
                    self.run_slowdown = 0.2; // `:26670`
                    // `:26672` — `releaseLeft`/`releaseRight` are `!controlX`
                    // recomputed at tick end (`:27624-27635`), so the pair is
                    // a direction-key press edge.
                    if (self.control_left && !self.left_prev)
                        || (self.control_right && !self.right_prev)
                    {
                        self.try_cart_dismount(world);
                    }
                } else {
                    self.run_slowdown = self.run_acceleration; // `:26675` — 0.04
                    self.on_wrong_ground = false; // `:26676`
                }
            }
        }
        if !grappling {
            // `:26496-26499`: the wing run-speed tweaks run before the
            // movement — vanilla gates them on `mount.CanUseWings` too (and
            // on `!merman`, which has no port counterpart).
            if self.wings_logic > 0 && self.vy != 0.0 && self.mount.can_use_wings() {
                self.wing_air_logic_tweaks();
            }
            // `Player.cs:26623-26688`: the per-tick mount block (with the
            // `UpdateEffects` head and the `UpdateJumpHeight` merge — vanilla
            // runs those at `:25656`/`:26011`, both earlier in the same tick).
            // A no-op without a mount; the cart ride's own stat block (the
            // same C# lines, `cart_mounted`-gated) runs where Phase 2 placed
            // it — the two never fire together (no cart row in the mount
            // table; M2-MOUNT-3 bridges them).
            self.update_mount(world, raining, wind);
            self.horizontal_movement();
        }

        let in_liquid = self.wet || self.lava_wet || self.honey_wet || self.shimmer_wet;
        // `Player.cs:27352-27359`: the plain arm integrates the rebased
        // gravity (the `:24380-24437` rebase above owns the liquid values)
        // at FULL strength and caps the fall at the rebased `maxFallSpeed`
        // — the `gravity / 3` / `maxFallSpeed / 3` sites (`:27008`/`:27339`)
        // are the mount/wings flight integrators, not a wet rule.
        let gravity = tick_gravity;
        let fall_cap = tick_max_fall;
        // `WetCollision`'s `movementSpeed` (`Player.cs:28168-28171` + :23436):
        // the position delta scales by the liquid speed — water/lava 0.5,
        // honey 0.25, shimmer 0.375. The scaled step feeds the collision; the
        // velocity itself keeps integrating (the exit speed is preserved).
        self.liquid_step =
            movement_speed_for(self.wet, self.lava_wet, self.honey_wet, self.shimmer_wet);

        let jump_pressed = input.jump && !self.jump_held;
        self.jump_held = input.jump;
        // `:21024` clears the latch at the END of the held block, so the
        // jump branches read the PREVIOUS tick's latch — capture it first.
        let released = self.release_jump;
        if input.jump && !grappling {
            // `:21024`: any held frame clears the release latch.
            self.release_jump = false;
            if self.on_ground && jump_pressed {
                // `Player.cs:20671-20675`: the cart nudge — the mount jump
                // lifts the magnet a hair off the rail line so the ride state
                // machine cannot re-catch the jump.
                if self.cart_mounted {
                    self.y -= 0.001;
                }
                // `Player.cs:20752-20754`: leaving the rail levels the tilt.
                if !self.on_track {
                    self.full_rotation = 0.0;
                }
                // `Player.cs:20766-20767`: the ground jump; the landing re-arm
                // follows (`:20755-20758` — `RefreshDoubleJumps`). `:20678`
                // sets `justJumped` on the grounded trigger.
                self.just_jumped = true;
                self.vy = -self.jump_speed;
                self.jump = self.jump_height;
                self.on_ground = false;
                self.refresh_double_jumps();
            } else if self.jump > 0 {
                if self.vy == 0.0 {
                    self.jump = 0;
                } else {
                    self.vy = -self.jump_speed;
                    self.jump -= 1;
                }
            } else if self.in_water {
                self.vy = -2.0;
            } else if self.jump == 0
                && released
                && self.can_jump_again.iter().any(|&c| c)
                && !self.in_water
            {
                // `Player.cs:20698-21022`: the extra-jump priority chain —
                // the mount jumps first, then Sandstorm, Blizzard, Fart,
                // Sail, Cloud as the unconditional fall-through. Each sets
                // the standard ascent velocity and its own hold-boost
                // counter; the performing flag is the visual state.
                let jumped = ExtraJump::PRIORITY
                    .iter()
                    .copied()
                    .find(|j| self.can_jump_again[j.index()]);
                if let Some(j) = jumped {
                    self.can_jump_again[j.index()] = false;
                    self.performing_jump = Some(j);
                    // `:20678`: only a grounded trigger counts as justJumped.
                    if self.vy == 0.0 {
                        self.just_jumped = true;
                    }
                    self.vy = -self.jump_speed;
                    self.jump = (self.jump_height as f32 * j.hold_multiplier()) as i32;
                } else {
                    // `:20746`: the Cloud fall-through consumed even when
                    // unarmed — the can flag clears all the same.
                    if let Some(cloud) = self.can_jump_again.get_mut(0) {
                        *cloud = false;
                    }
                }
            }
        } else {
            self.jump = 0;
            // `:21026-21031`: jump released — the rocket re-arm latch and the
            // wing/rocket ground refills gate on it.
            self.release_jump = true;
            self.rocket_release = true;
        }

        // `Player.cs:26061-26099`: the stale option clear while airborne.
        self.clear_stale_jumps();

        // `Player.Update` → `DashMovement` (`Main` call order `:26757`): the
        // velocity-only dash pass before the collision step.
        if !grappling {
            self.dash_movement(world, input);
        }
        // `Player.cs:26799-26812`: while CCed the mount dismounts and the
        // plain gravity takes over (the port's CC input zeroing and the
        // flight gate below already cover the rest of that branch).
        if !grappling && self.cced() && self.mount.is_active() {
            self.try_dismount(world);
        }

        // `Player.Update` → `GrappleMovement` (`Main` call order `:27653`,
        // `Player.cs:22613-22740`): the pull REPLACES the velocity — the
        // vector to the attached hook center clamped to the pull speed. The
        // fall origin refreshes per tick (no fall damage accrues).
        if grappling {
            self.grapple_movement(grapple, input, item_in_use);
        }

        // The flight section (`Player.cs:26744-27360`): the wings and the
        // rocket boots REPLACE the gravity integration on the ticks they run.
        // The plain gravity below applies only in the fallback. The wet ticks
        // skip the flight (the vanilla water physics dominate; the swim cap
        // stays).
        let flew = if !in_liquid && !self.webbed && !grappling {
            self.flight_section(world, input, gravity, fall_cap)
        } else {
            false
        };
        // `Player.cs:26821-26824`: a rest tick clears the ramp freeze.
        if self.vy == 0.0 {
            self.cart_ramp_time = 0;
        }
        if !flew && !self.webbed && !grappling {
            // `Player.cs:27352-27359`: the ramp freeze — gravity applies only
            // once the ramp timer has run out; otherwise it ticks down.
            if self.cart_ramp_time <= 0 {
                self.vy = (self.vy + gravity).min(fall_cap);
            } else {
                self.cart_ramp_time -= 1;
            }
        }
        self.vy = self.vy.max(if in_liquid {
            -(MAX_FALL / 3.0)
        } else {
            -MAX_FALL
        });

        // `Player.cs:28093-28161`: the `Minecart.TrackCollision` slot — it runs
        // before the integration because it snaps Y itself and rewrites the
        // velocity (pitfall §10.3: X moves in the integration below).
        let _was_on_track = self.on_track; // `:28091` `flag32` — the off-rail JumpingSound delegate is empty (see below)
        self.on_track = false; // `:28092`
        let mut hit_switch_flag = false; // `flag33`
        if self.cart_mounted {
            // `:28099-28106`: the water scale wraps only this call
            // (`ignoreWater`/`merman` have no port fields yet — treated false).
            let scale = if self.shimmer_wet {
                0.25
            } else if self.honey_wet {
                0.25
            } else if !self.wet {
                1.0
            } else {
                0.5
            };
            let body_h = self.body_height() as i32;
            // `Player.cs:28110-28111` + `Minecart.cs:809-813`: the landing
            // event fires mid-call with the pre-resolve `Position` and the
            // rotation the caller set before the call — both captured here.
            let pre_pos = Vec2::new(self.x, self.y);
            let pre_rotation = self.full_rotation;
            let mut pos = Vec2::new(self.x, self.y);
            let mut vel = Vec2::new(self.vx, self.vy);
            vel *= scale;
            // `:28109-28110` — `controlDown`/`controlUp` are the follow flags
            // (the port's input has no up bit; single-rail rides never read it).
            let flags = minecart::track_collision(
                world,
                &mut pos,
                &mut vel,
                &mut self.last_boost,
                WIDTH as i32,
                body_h,
                self.control_down,
                false,
                self.fall_start2,
                false,
            );
            vel /= scale;
            // `:28116-28119` — the non-cart position restore never fires for
            // carts (`!this.mount.Cart` gate).
            if flags.on_track() {
                self.on_track = true; // `:28113`
                // `:28114-28115` — the rail-slope visual correction.
                let (gfx, rot) = minecart::track_rotation(
                    world,
                    pos + vel,
                    WIDTH as i32,
                    body_h,
                    self.control_down,
                    false,
                );
                self.gfx_off_y = gfx;
                self.full_rotation = rot;
                self.full_rotation_origin = ((WIDTH / 2.0), body_h as f32);
            }
            // `:28117-28119`: the riding-off-rail `MinecartJumpingSound`
            // delegate is EMPTY in C# (`DelegateMethods.cs:244-248`) — the
            // `was_on_track` && !on_track edge plays nothing.
            if flags.landing {
                // `Minecart.cs:809-813`: the landing check —
                // `MinecartLandingSound` (Item53) + a 10-spark burst, both
                // with the pre-call position and rotation captured above.
                minecart::minecart_landing_sound(
                    &mut self.pending_cart_sounds,
                    pre_pos,
                    WIDTH as i32,
                    body_h,
                );
                minecart::wheel_sparks(
                    &mut self.pending_sparks,
                    &mut self.spark_rng,
                    pre_pos,
                    WIDTH as i32,
                    body_h,
                    10,
                    pre_rotation,
                );
            }
            if flags.bouncy_bumper() {
                // `:28123-28136`: the sprite flip follows the held direction
                // keys; the body direction follows the flipped roll.
                if self.control_left || self.control_right {
                    self.cart_flip = !self.cart_flip;
                }
                if vel.x > 0.0 {
                    self.facing = 1;
                } else if vel.x < 0.0 {
                    self.facing = -1;
                }
                // `:28140`: `MinecartBumperSound` → Item56, at the
                // post-TrackCollision position (`base.position`, pre-integration).
                minecart::minecart_bumper_sound(
                    &mut self.pending_cart_sounds,
                    pos,
                    WIDTH as i32,
                    body_h,
                );
            }
            if flags.hit_switch() {
                hit_switch_flag = true; // `:28138-28140`
            }
            if flags.used_ramp() {
                // `:28148-28150` — the gravity-freeze grant, truncated.
                self.cart_ramp_time = (1.0f32.min(self.vx.abs() / CART_RUN_SPEED) * 20.0) as i32;
            }
            if flags.boost_left() {
                self.track_boost -= minecart::BOOSTER_SPEED; // `:28151-28153`
            }
            if flags.boost_right() {
                self.track_boost += minecart::BOOSTER_SPEED; // `:28155-28157`
            }
            self.x = pos.x;
            self.y = pos.y;
            self.vx = vel.x;
            self.vy = vel.y;
        }
        // The AABB resolve (`TileCollision` + the step/slope helpers): the
        // velocity phase above only shaped `velocity`; this integrates it.
        self.move_vanilla(world, input.down, gravity);
        // `Player.cs:28237-28241` + `Minecart.HitTrackSwitch`
        // (`Minecart.cs:1281-1288`): the pressure-track trip fires at the
        // post-integration magnet tile (net message 59 is out of scope; the
        // caller applies `Wiring::hit_switch`).
        if hit_switch_flag {
            let m = minecart::magnet_position(
                Vec2::new(self.x, self.y),
                WIDTH as i32,
                self.body_height() as i32,
            );
            self.pending_switch_trip = Some(((m.x / 16.0) as i32, (m.y / 16.0) as i32));
        }

        // `Player.cs:25219-25224`: the tick petrification lands, a flat hit —
        // the difficulty multiplier is 1 (classic) until M6-CREATIVE lands.
        let stoned = self.buffs.find_buff_index(156).is_some();
        if stoned && !self.last_stoned {
            let cx = self.x + WIDTH * 0.5;
            self.hurt_inner(20, cx, false, rng, true);
        }
        self.last_stoned = stoned;

        // `Player.cs:25293-25385`: the fall-damage block — the whole thing runs
        // whenever the player has come to rest this tick (`velocity.Y == 0`).
        // `:26057-26060`: every rest tick re-arms the granted extra jumps.
        if self.vy == 0.0 {
            self.refresh_double_jumps();
            // `num10 = 25 + extraFall` (`Player.cs:25301`); the mount addendum
            // (`Player.cs:25301-25304`): `25 + extraFall` + the mount's own.
            let threshold = 25 + self.extra_fall + self.mount.extra_fall();
            // `num11` is int-truncated on both ends (`Player.cs:25306`).
            let mut fell = (self.y / TILE_PX) as i32 - self.fall_start;
            // `:25307-25318`: the zeroing — a flying mount and the bunny
            // (type 1) nullify.
            if self.mount.can_fly() || self.mount.kind() == 1 {
                fell = 0;
            }
            // `Player.cs:25311-25314`: a track rider resting on rails takes no
            // fall damage — `Minecart.OnTrack` reads the magnet tile.
            if self.cart_mounted
                && minecart::on_track(
                    world,
                    Vec2::new(self.x, self.y),
                    WIDTH as i32,
                    self.body_height() as i32,
                )
            {
                fell = 0;
            }
            // The dash zero of `:25319-25323` waits for its own task; the
            // cloud/soft-tile scan does
            // (`Player.cs:25325-25345`): the tile row under the feet across
            // the hitbox's x-extent.
            let tx0 = (self.x / TILE_PX) as i32;
            let tx1 = ((self.x + WIDTH) / TILE_PX) as i32;
            // `Player.cs:25321` reads `base.height` — the live hitbox height
            // (the mount framework's equip for its mounts; the cart's 52) scans
            // the tile row, not the row above it.
            let ty = ((self.y + self.body_height() + 1.0) / TILE_PX) as i32;
            for tx in tx0..=tx1 {
                if let Some(t) = world.tile(tx, ty) {
                    if t.active()
                        && (tr_core::tiledata::SET_CLOUDS.has(t.kind)
                            || t.kind == 666
                            || (t.kind == 19 && (t.frame_y / 18) == 49))
                    {
                        fell = 0;
                        break;
                    }
                }
            }
            if stoned {
                // `Player.cs:25347-25359`: petrified — a fixed 2-tile grace and
                // 20 damage per tile; extraFall and the mount multiplier are
                // ignored, and the immunity clears AFTER the hit.
                let dmg = (fell - 2) * 20;
                if dmg > 0 {
                    let cx = self.x + WIDTH * 0.5;
                    self.hurt_inner(dmg, cx, false, rng, true);
                    self.hurt_cd = 0.0;
                }
            } else if (fell > threshold) && !self.no_fall_dmg && !self.has_wings {
                // `Player.cs:25360-25374`: `(fell - threshold) * 10` damage with
                // hitDirection 0 — no knockback — and `immune = false` forced
                // BEFORE the Hurt so the hit lands through any iframes.
                // `:25363-25367`: the mount's `FallDamage` scales the hit
                // (only while a mount is active — the C# gates on
                // `mount.Active`).
                let dmg = if self.mount.is_active() {
                    (((fell - threshold) * 10) as f32 * self.mount.fall_damage()) as i32
                } else {
                    (fell - threshold) * 10
                };
                self.hurt_fall(dmg, rng);
            }
            self.fall_start = (self.y / TILE_PX) as i32;
        }
        // `Player.cs:25379-25382`: the continuous reset — jumping, liquid, the
        // slow-fall/rocket/sky-gravity/tongued terms (the four systems are not
        // ported yet) keep the fall origin at the current row.
        // `Player.cs:25379-25382`: the rocket thrust refreshes the fall
        // origin (the plain wing ascent relies on `hasWings` instead).
        if self.jump > 0 || self.rocket_delay > 0 || self.in_water {
            self.fall_start = (self.y / TILE_PX) as i32;
        }

        // `Player.cs:28269-28272`: `FloorVisuals` after this tick's movement —
        // the movement block reads the flags next tick (one tick stale).
        if self.vy == 0.0 && !grappling {
            self.floor_visuals(world);
        }

        self.update_survival(rng);

        if self.x < 0.0 {
            self.x = 0.0;
            self.vx = 0.0;
        }
        let max_x = (world.width as f32 * TILE_PX) - WIDTH;
        if self.x > max_x {
            self.x = max_x;
            self.vx = 0.0;
        }
        if self.y > world.height as f32 * TILE_PX + 300.0 {
            self.respawn();
        }

        // The immune purge runs near the end of the original update, after movement
        // and effects (`Player.cs:26012-26018`).
        self.buffs.purge_immune_buffs();

        self.update_frames(world);

        // `Player.cs:27618-27635`: the release flags recompute at the end of
        // the tick — the double-tap reads the press edges against them.
        self.right_prev = input.right;
        self.left_prev = input.left;
    }

    /// `Mount.TryLanding` (`Mount.cs:4562-4568`): while the frame state is
    /// still air/fly, a 4px-down tile probe snapping shut pulls the frame to
    /// the grounded set on the touchdown tick (the C# re-probes
    /// `TileCollision` with `UnitY * gravDir * 4`).
    fn try_landing(&mut self, world: &World) {
        if !matches!(
            self.mount.frame_state(),
            crate::mount::FRAME_FLYING | crate::mount::FRAME_IN_AIR
        ) {
            return;
        }
        // The `!controlUp && !controlDown && !controlJump` gate — the port
        // has no controlUp input yet.
        if self.control_down || self.control_jump {
            return;
        }
        let probe = collision::tile_collision(
            world,
            [self.x, self.y],
            [0.0, 4.0],
            WIDTH,
            self.height,
            false,
            false,
        );
        if probe.vel[1] == 0.0 {
            // `GetIntendedGroundedFrame` (`Mount.cs:4547-4560`).
            let state = if self.vx == 0.0
                || ((self.slippy || self.slippy2) && !self.control_left && !self.control_right)
            {
                crate::mount::FRAME_STANDING
            } else {
                crate::mount::FRAME_RUNNING
            };
            self.mount
                .update_frame(state, self.vx, self.vy, self.just_jumped, self.control_down);
        }
    }

    /// `PlayerFrame()` (`Player.cs:35755-36647`) — the legs and body rows of
    /// every pose. The chain order is the vanilla one: the swim window, the
    /// mount animator (`flag7` plus `DoesNotOverrideLegFrames`), the leg rows,
    /// then the body rows (item use → pulley → hold styles → shield → mount →
    /// grapple → water → swim → air → walk → idle).
    ///
    /// The C# reads the held item out of `inventory[selectedItem]`; the port
    /// keeps the inventory in the app, which hands the same fields in through
    /// [`FrameInputs`] before each fixed step (`world_sim/player.rs`, beside
    /// `swing_aim`).
    ///
    /// Arms whose granting equipment has no port field yet stay coded but
    /// unreachable until it lands: Sandstorm in a Bottle (`sand_storm`), the
    /// Magic Carpet (`carpet_frame`), flippers (`ShouldFloatInWater`), the
    /// pulley/rope item (`pulley`) and the raised shield (`shield_raised`).
    fn update_frames(&mut self, world: &World) {
        let held = self.frame_inputs;
        // `:35756-35762`: the swim window ticks down and dies out of water.
        if self.swim_time > 0 {
            self.swim_time -= 1;
            if !self.wet {
                self.swim_time = 0;
            }
        }
        // `:36148-36240`: `flag7` — the mount owns the leg frames unless it is
        // one of the 57-60 roller-skate mounts
        // (`MountID.Sets.DoesNotOverrideLegFrames`, `MountID.cs:39`). Every
        // mount still runs its own animator, so this block does not return.
        let mut mount_owns_legs = false;
        if self.mount.is_active() {
            let kind = self.mount.kind();
            if !crate::mount::does_not_override_leg_frames(kind) {
                mount_owns_legs = true;
                // `:36150-36161`: row 6; the 23/45/48 carve-out forces row 0.
                self.leg_frame_counter = 0.0;
                self.leg_frame = 6;
                if matches!(kind, 23 | 45 | 48) {
                    self.leg_frame = 0;
                }
            }
            if self.vy != 0.0 && self.mount.running_grace_time() <= 0 {
                if self.mount.fly_time() > 0
                    && self.jump == 0
                    && self.control_jump
                    && !self.mount.can_hover()
                {
                    // `:36199-36200`: `TryBeginningFlight` only fires for
                    // hover mounts (`Mount.cs:4537` — none in this table);
                    // the Rudolph dust trail is a visual seam.
                    self.mount.update_frame(
                        crate::mount::FRAME_FLYING,
                        self.vx,
                        self.vy,
                        self.just_jumped,
                        self.control_down,
                    );
                    self.try_landing(world);
                } else if self.wet {
                    // `:36203-36206`: the swim state (the ground mounts'
                    // swim sets reuse their in-air frames).
                    self.mount.update_frame(
                        crate::mount::FRAME_SWIMMING,
                        self.vx,
                        self.vy,
                        self.just_jumped,
                        self.control_down,
                    );
                } else {
                    // `:36207-36212`: plain air.
                    self.mount.update_frame(
                        crate::mount::FRAME_IN_AIR,
                        self.vx,
                        self.vy,
                        self.just_jumped,
                        self.control_down,
                    );
                    self.try_landing(world);
                }
            } else {
                // `:36216` -> `GetIntendedGroundedFrame` (`Mount.cs:4547-4560`):
                // the hooks-grapple arm cannot fire (no rows in CanUseHooks);
                // the `windPushed` term has no port counterpart.
                let state = if self.vx == 0.0
                    || ((self.slippy || self.slippy2) && !self.control_left && !self.control_right)
                {
                    crate::mount::FRAME_STANDING
                } else {
                    crate::mount::FRAME_RUNNING
                };
                self.mount.update_frame(
                    state,
                    self.vx,
                    self.vy,
                    self.just_jumped,
                    self.control_down,
                );
            }
        }

        // `:36219-36291`: the leg rows.
        if !mount_owns_legs && self.legs_armor != 140 {
            if self.swim_time > 0 {
                // `:36221-36233`: the swim paddle — a flat 2 per tick.
                self.leg_frame_counter += 2.0;
                while self.leg_frame_counter > 8.0 {
                    self.leg_frame_counter -= 8.0;
                    self.leg_frame += 1;
                }
                if self.leg_frame < 7 {
                    self.leg_frame = 19;
                } else if self.leg_frame > 19 {
                    self.leg_frame = 7;
                }
            } else if self.vy != 0.0 || self.grappling_active {
                // `:36234-36240`: the jump/fall/hang row 5; the three
                // always-animated wing sets keep row 0.
                self.leg_frame_counter = 0.0;
                self.leg_frame = 5;
                if matches!(self.wings_logic, 22 | 28 | 45)
                    && self.should_draw_wings_that_are_always_animated()
                {
                    self.leg_frame = 0;
                }
            } else if self.vx != 0.0 {
                // `:36242-36275`: the walk cycle. The slippery no-input arm
                // (and the roller skates, `MountID.Sets.IsRollerSkates`) pins
                // the rest row.
                let roller_skates =
                    self.mount.is_active() && crate::mount::is_roller_skates(self.mount.kind());
                if ((self.slippy || self.slippy2) | roller_skates)
                    && !self.control_left
                    && !self.control_right
                {
                    self.leg_frame_counter = 0.0;
                    self.leg_frame = 0;
                } else {
                    let mut num19 = self.vx.abs() * 1.3;
                    // `:36255-36258`: the skates lock to 2 at full speed.
                    if roller_skates && self.vx.abs() >= self.max_run_speed {
                        num19 = 2.0;
                    }
                    self.leg_frame_counter += num19;
                    while self.leg_frame_counter > 8.0 {
                        self.leg_frame_counter -= 8.0;
                        self.leg_frame += 1;
                    }
                    if self.leg_frame < 7 {
                        self.leg_frame = 19;
                    } else if self.leg_frame > 19 {
                        self.leg_frame = 7;
                    }
                }
            } else {
                // `:36276-36282`: the rest row.
                self.leg_frame_counter = 0.0;
                self.leg_frame = 0;
            }
        }
        // `:36292-36297`: the Magic Carpet glide pins the rest row.
        if self.carpet_frame >= 0 {
            self.leg_frame_counter = 0.0;
            self.leg_frame = 0;
        }
        // `:36298-36311`: the sandstorm dash pins the rest row and turns the
        // player every 4 ticks while no item is in use.
        if self.sand_storm {
            if self.grappling_active {
                self.sand_storm = false;
            }
            if self.misc_counter % 4 == 0 && !held.swing_active {
                self.facing = -self.facing;
            }
            self.leg_frame_counter = 0.0;
            self.leg_frame = 0;
        } else if held.swing_active && held.use_style_blocks_frames {
            // `:36312-36444`: the item-use rows per useStyle.
            self.apply_item_use_frame(held.swing_remaining, held.use_style, held.aim_relative);
        } else if self.pulley {
            // `:36445-36454`: the rope pose.
            self.body_frame = if self.pulley_dir == 2 { 1 } else { 2 };
        } else if held.can_visually_hold
            && held.hold_style == 1
            && held.hold_style_dry
            && held.torch_hold_ok
        {
            // `:36455-36459`: the torch hold (row 3).
            self.body_frame = 3;
        } else if held.can_visually_hold && held.hold_style == 2 && held.hold_style_dry {
            // `:36460-36464`: the held-up item (row 2).
            self.body_frame = 2;
        } else if held.can_visually_hold && held.hold_style == 3 {
            self.body_frame = 3;
        } else if held.can_visually_hold && held.hold_style == 5 {
            self.body_frame = 3;
        } else if held.can_visually_hold && held.hold_style == 7 {
            // `:36477-36481`: the holdStyle 7 row 11.
            self.body_frame = 11;
        } else if held.can_visually_hold && held.hold_style == 4 && self.vy == 0.0 {
            // `:36483-36488`: the umbrella hold rests on row 0. (The C# also
            // requires `gravDir == 1`; the port has no gravity flip.)
            self.body_frame = 0;
        } else if self.shield_raised {
            // `:36489-36492`: the raised shield (row 10).
            self.body_frame = 10;
        } else if self.mount.is_active()
            && !crate::mount::does_not_override_body_frames(self.mount.kind())
        {
            // `:36493-36497`: the mount body row.
            self.body_frame_counter = 0.0;
            self.body_frame = self.mount.data().map_or(0, |d| d.body_frame) as u32;
        } else if self.grappling_active {
            // `:36498-36535`: the grapple pose leans with the hooks' centroid.
            self.sand_storm = false;
            let dx = held.grapple_dx;
            let dy = held.grapple_dy;
            if dy < 0.0 && dy.abs() > dx.abs() {
                self.body_frame = 2;
            } else if dy > 0.0 && dy.abs() > dx.abs() {
                self.body_frame = 4;
            } else {
                self.body_frame = 3;
            }
        } else if self.wet && self.should_float_in_water() {
            // `:36536-36539`: the float pose (row 10).
            self.body_frame = 10;
        } else if self.swim_time > 0 {
            // `:36540-36558`: the swim window — row 0 at the edges, row 5 in
            // the middle.
            self.body_frame = if self.swim_time > 20 {
                0
            } else if self.swim_time > 10 {
                5
            } else {
                0
            };
        } else if self.vy != 0.0 {
            // `:36559-36600`: the airborne rows.
            if self.sliding {
                self.body_frame = 3;
            } else if self.sand_storm || self.carpet_frame >= 0 {
                self.body_frame = 6;
            } else if self.eoc_dash > 0 {
                self.body_frame = 6;
            } else if self.wings_logic > 0 {
                self.body_frame = if matches!(self.wings_logic, 22 | 28 | 45) {
                    0
                } else if self.vy > 0.0 {
                    if self.control_jump {
                        6
                    } else {
                        5
                    }
                } else {
                    6
                };
            } else {
                self.body_frame = 5;
            }
            self.body_frame_counter = 0.0;
        } else if self.vx != 0.0 {
            // `:36602-36623`: the walk rows. The Moon Lord legs (140) run
            // their own counter at `|velocity.X| * 0.5`; everything else
            // copies the leg row.
            if self.legs_armor == 140 {
                self.body_frame_counter += self.vx.abs() * 0.5;
                while self.body_frame_counter > 8.0 {
                    self.body_frame_counter -= 8.0;
                    self.body_frame += 1;
                }
                if self.body_frame < 7 {
                    self.body_frame = 19;
                } else if self.body_frame > 19 {
                    self.body_frame = 7;
                }
            } else {
                self.body_frame_counter += self.vx.abs() * 1.5;
                self.body_frame = self.leg_frame;
            }
        } else {
            // `:36624-36633`: the rest row.
            self.body_frame_counter = 0.0;
            self.body_frame = 0;
        }

        // `:36634-36644`: the Moon Lord legs override their own rows after the
        // body chain — row 5 in the air, row 0 at rest, and row 7 whenever the
        // body row's headgear offset is zero.
        if self.legs_armor == 140 {
            self.leg_frame_counter = 0.0;
            self.leg_frame = if self.vy != 0.0 { 5 } else { 0 };
            if offsets_player_headgear_y(self.body_frame as usize) == 0.0 {
                self.leg_frame = 7;
            }
            if matches!(self.wings_logic, 22 | 28) {
                self.leg_frame = 0;
            }
        }
        // `:36645-36651`: the sitting/sleeping legs (217) pin row 5.
        if self.legs_armor == 217 && (self.sitting || self.sleeping) {
            self.leg_frame_counter = 0.0;
            self.leg_frame = 5;
        }
    }

    /// `Player.ShouldDrawWingsThatAreAlwaysAnimated` (`Player.cs:30737-30748`):
    /// the three always-animated wing sets keep the rest legs while airborne.
    fn should_draw_wings_that_are_always_animated(&self) -> bool {
        self.vy != 0.0
            && !self.grappling_active
            && (!self.wet || !self.should_float_in_water())
            && self.mount.data().is_some_and(|d| d.can_use_wings)
    }

    /// `Player.ShouldFloatInWater` (`Player.cs:3872-3885`): `canFloatInWater`
    /// (flippers or the merman buff) while not holding down; mounted, only the
    /// Water Strider's mount (37) floats. No accessory grants
    /// `canFloatInWater` in the port yet, so this stays false and the pose arm
    /// is coded but unreachable.
    fn should_float_in_water(&self) -> bool {
        false
    }

    /// `PlayerFrame()`'s item-use branch (`Player.cs:36312-36444`): the body
    /// rows per useStyle while `itemAnimation > 0`. `remaining` is
    /// `itemAnimation / itemAnimationMax` (1 at the swing's start, 0 at the
    /// end — the C# compares `itemAnimation` against the fractions directly);
    /// `aim_relative` is `itemRotation * direction` (the shooting styles' rows
    /// read it). The quirks are verbatim: style 3's both branches are row 3,
    /// style 5 reads the aim, style 13 mirrors style 1.
    fn apply_item_use_frame(&mut self, remaining: f32, use_style: u8, aim_relative: f32) {
        // The inverted remaining: `itemAnimation > max * frac` ==
        // `remaining < (1 - frac)` (itemAnimation runs 1 -> 0).
        let inv = 1.0 - remaining;
        match use_style {

            // `:36312-36326` (blocks/placeables; `type == 0` shares it) —
            // rows 1 -> 2 -> 3 by thirds.
            1 | 0 => {
                self.body_frame = if remaining < 0.333 {
                    3
                } else if remaining < 0.666 {
                    2
                } else {
                    1
                };
            }
            // `:36327-36331` — DrinkOld: the first half row 3, then row 2.
            7 => {
                self.body_frame = if remaining > 0.5 { 3 } else { 2 };
            }
            // `:36332-36334` — EatFood: row 3 through the use.
            2 => self.body_frame = 3,
            // `:36335-36345` — MowTheLawn: >half row 4, >15% row 3, else 2.
            11 => {
                self.body_frame = if remaining > 0.5 {
                    4
                } else if remaining > 0.15 {
                    3
                } else {
                    2
                };
            }
            // `:36346-36348` — DrinkLiquid: row 0.
            9 => self.body_frame = 0,
            // `:36349-36362` — DrinkLong: the progress `* 6` clamped at 1,
            // >= 0.5 row 3 else row 2.
            6 => {
                let num20 = (inv * 6.0).min(1.0);
                self.body_frame = if num20 >= 0.5 { 3 } else { 2 };
            }
            // `:36363-36372` — Thrust: BOTH branches row 3 (the vanilla
            // quirk — `:36368`'s else is dead).
            3 => self.body_frame = 3,
            // `:36373-36375` — HoldUp: row 2.
            4 => self.body_frame = 2,
            // `:36376-36378` — GolfPlay: row 0.
            8 => self.body_frame = 0,
            // `:36379-36381` — Guitar: row 3.
            12 => self.body_frame = 3,
            // `:36382-36392` — Rapier: the same thirds as style 1.
            13 => {
                self.body_frame = if remaining < 0.333 {
                    3
                } else if remaining < 0.666 {
                    2
                } else {
                    1
                };
            }
            // `:36393-36444` — Shoot: the rows read `itemRotation *
            // direction` (the aim): the default row 3, past -0.75 row 2
            // (gravDir -1: row 4), past 0.6 row 4 (gravDir -1: row 2).
            5 => {
                self.body_frame = 3;
                let num21 = aim_relative;
                if num21 < -0.75 {
                    self.body_frame = 2;
                }
                if num21 > 0.6 {
                    self.body_frame = 4;
                }
            }
            _ => {}
        }
    }

    /// The expired-buff sweep of `Player.Update` (`Player.cs:25608-25616`): a
    /// slot holding a type with a non-positive timer is deleted. Like the C#,
    /// the loop does not rescan after `DelBuff` compaction.
    fn remove_expired_buffs(&mut self) {
        for i in 0..MAX_BUFFS {
            if self.buffs.buff_type[i] > 0 && self.buffs.buff_time[i] <= 0 {
                self.buffs.del_buff(i);
            }
        }
    }

    /// The slot scan behind the C# per-buff flag chain: a buff counts while it
    /// occupies a slot with a positive timer (`Player.cs:9869-9870`). The scan
    /// ignores `buffImmune` on purpose — an immune-held buff still sets its
    /// flag for the one frame before the purge deletes it (`:26011-26016`).
    fn holding_index(&self, buff_type: i32) -> Option<usize> {
        (0..MAX_BUFFS)
            .find(|&i| self.buffs.buff_time[i] > 0 && self.buffs.buff_type[i] == buff_type)
    }

    /// `true` while `buff_type` is held on a live slot (see [`Self::holding_index`]).
    fn holding(&self, buff_type: i32) -> bool {
        self.holding_index(buff_type).is_some()
    }

    /// The Wet-buff (103) player-side effects of the `UpdateBuffs` chain
    /// (`Player.cs:11740-11751`): while held, lava contact deletes the buff;
    /// otherwise it re-grants this frame's fire immunities (OnFire 24, OnFire3
    /// 323, Burning 67).
    fn refresh_wet_state(&mut self) {
        // The C# branch runs inside the per-buff slot loop and uses the
        // previous frame's `lavaWet`; the port's `in_lava` probe is this
        // frame's, so the removal lands one tick earlier than the original's
        // lagging flag (docs/ANALYSIS.md, Phase 4).
        if let Some(i) = self.holding_index(103) {
            if self.in_lava {
                self.buffs.del_buff(i);
            } else {
                self.buffs.buff_immune[24] = true;
                self.buffs.buff_immune[323] = true;
                self.buffs.buff_immune[67] = true;
            }
        }
    }

    /// The DoT branches of `Player.UpdateLifeRegen` as a pure derivation.
    ///
    /// Pseudocode (branch order = C# `Player.cs:18949→19095`; each hit branch
    /// clamps a positive regen and zeroes the ramp time, folded into
    /// `resets_time`):
    /// ```text
    /// regen ← 0; resets_time ← false
    /// poisoned(20): −4 | venom(70): −30 | chlorophyteSpore(397): −30
    /// onFire(24): −8, ×2 while drippingSlime(137)
    ///             (−100/×−100 while `Main.vampireSeed`, the `vampirism` micro seed)
    /// onFire3(323): −8 ×2 | frostburn(44): −16 ×2 | frostburn2(324): −16 ×2
    /// cursedInferno(39): −24 ×2 | burned(67): −60 ×2
    /// suffocating(68): −40
    /// electrified(144): −8, −32 more while controlLeft || controlRight
    /// tongued(38) && expert: −100
    /// bleeding(30): zeroes the ramp time only (no delta, `:19178-19181`)
    /// honey(48) while regen < 0: regen += 4, clamped at 0 (never positive)
    /// fast_lane ← burned || suffocating || (tongued && expert)      (:19313)
    /// death_fast ← suffocating → 7, else tongued → 12, else 8       (:19321-19331)
    /// death_generic ← vampire(22, unported) → poisoned||venom → 9 →
    ///                 electrified → 10 → chlorophyte → 23 → else 8  (:19362-19385)
    /// ```
    fn derive_dot_inputs(&self, expert: bool) -> DotDerivation {
        let mut regen = 0i32;
        let mut resets_time = false;
        // The DoT branches in the exact C# order (`Player.cs:18949-19095`).
        // Each branch clamps a positive regen and zeroes `lifeRegenTime`
        // before subtracting (`:18951-18956` shape) — the engine's
        // `dot_resets_time` covers the clamp+reset for the folded sum.
        if self.holding(20) {
            // Poisoned — `:18949-18957`.
            resets_time = true;
            regen -= 4;
        }
        if self.holding(70) {
            // Venom — `:18958-18966`.
            resets_time = true;
            regen -= 30;
        }
        if self.holding(397) {
            // Chlorophyte Spore — `:18967-18975`.
            resets_time = true;
            regen -= 30;
        }
        let drip = self.holding(137); // drippingSlime doubles the fire family
        if self.holding(24) {
            // OnFire — `:18976-18998`. M5-SEED-2: the `vampirism` world trades
            // the drain for its own (`Main.vampireSeed` → −100, and the
            // `drippingSlime` double is −100 instead of −8, `:18983-18992`) —
            // the flag comes from the world's micro seeds (`WorldGen.cs:555-558`).
            resets_time = true;
            if self.vampirism {
                regen -= 100;
                if drip {
                    regen -= 100;
                }
            } else {
                regen -= 8;
                if drip {
                    regen -= 8;
                }
            }
        }
        if self.holding(323) {
            // OnFire3 (hellfire) — `:19000-19012`.
            resets_time = true;
            regen -= 8;
            if drip {
                regen -= 8;
            }
        }
        if self.holding(44) {
            // Frostburn — `:19013-19025`.
            resets_time = true;
            regen -= 16;
            if drip {
                regen -= 16;
            }
        }
        if self.holding(324) {
            // Frostburn2 — `:19026-19038`.
            resets_time = true;
            regen -= 16;
            if drip {
                regen -= 16;
            }
        }
        if self.holding(39) {
            // Cursed Inferno (C# `onFire2`) — `:19039-19051`.
            resets_time = true;
            regen -= 24;
            if drip {
                regen -= 24;
            }
        }
        if self.holding(67) {
            // Burning (C# `burned`) — `:19052-19064`.
            resets_time = true;
            regen -= 60;
            if drip {
                regen -= 60;
            }
        }
        if self.holding(68) {
            // Suffocation — `:19065-19073`.
            resets_time = true;
            regen -= 40;
        }
        if self.holding(144) {
            // Electrified — `:19074-19086`; the extra 32 needs the run keys.
            resets_time = true;
            regen -= 8;
            if self.control_left || self.control_right {
                regen -= 32;
            }
        }
        let tongued = self.holding(38);
        if tongued && expert {
            // Tongued, expert-only — `:19087-19095`.
            resets_time = true;
            regen -= 100;
        }
        // Bleeding zeroes the ramp time without draining — `:19178-19181`.
        if self.holding(30) {
            resets_time = true;
        }
        // Honey softens a drain by 4, clamped at 0 so it never heals —
        // `:19096-19103`. At this point the C# `lifeRegen` holds only the DoT
        // sum (the flat adds come later), so folding here is exact.
        if self.holding(48) && regen < 0 {
            regen = (regen + 4).min(0);
        }
        // The fast-lane entry condition — `:19313`.
        let fast_lane = self.holding(67) || self.holding(68) || (tongued && expert);
        // The fast lane's kill reason — `:19319-19333` (suffocation 7 →
        // tongued 12 → the fire fallback 8).
        let death_fast = if self.holding(68) {
            DotDeath::Suffocation
        } else if tongued {
            DotDeath::Tongued
        } else {
            DotDeath::Burned
        };
        // The generic lane's kill reason — `:19361-19384`. The C# tests
        // `vampireBurningInSunlight` (22) first; nothing grants it in the
        // port, so the chain starts at poisoned/venom (9).
        let death_generic = if self.holding(20) || self.holding(70) {
            DotDeath::Poison
        } else if self.holding(144) {
            DotDeath::Electrified
        } else if self.holding(397) {
            DotDeath::ChlorophyteSpore
        } else {
            DotDeath::Burned
        };
        DotDerivation {
            regen,
            resets_time,
            fast_lane,
            death_fast,
            death_generic,
        }
    }

    /// `Player.RefreshDoubleJumps` (`Player.cs:22869-22908`): every option
    /// the equipment grants re-arms.
    fn refresh_double_jumps(&mut self) {
        for i in 0..9 {
            if self.has_jump_options[i] {
                self.can_jump_again[i] = true;
            }
        }
    }

    /// `Player.cs:26061-26099`: while airborne, an extra jump whose option
    /// vanished (an accessory swap) force-clears.
    fn clear_stale_jumps(&mut self) {
        if self.vy == 0.0 {
            return;
        }
        for i in 0..9 {
            if !self.has_jump_options[i] {
                self.can_jump_again[i] = false;
            }
        }
    }

    /// `Player.HorizontalMovement` (`Player.cs:19772-20088`): the primary
    /// acceleration gates on `maxRunSpeed`; between `maxRunSpeed` and
    /// `accRunSpeed` (the boot top speeds) acceleration continues at 0.2x —
    /// ground-only here, the wings/flying-mount terms land with PHYS-5/M2-
    /// MOUNT-1, and `dashDelay >= 0` always holds until M2-PHYS-4. No input
    /// decays by `runSlowdown` on the ground and half of it in the air. There
    /// is no hard clamp — the decay bounds the speed (knockback rides out).
    /// `Player.DashMovement` (`Player.cs:21034-21538`), dashTypes 1/2/3/6.
    /// The dash granters (Tabi 977, EoCShield 3097, Master Ninja Gear 984 →
    /// type 1/2/1) land with M3-EQUIP-3, the Solar set (type 3) and the
    /// Chillet mounts (type 6) with their own tasks — `TR_DASH=<1|2|3|6>`
    /// pins the capability for now. The dust/gore trails are visual seams;
    /// the vanilla plays no dash sounds.
    fn dash_movement(&mut self, world: &World, input: PlayerInput) {
        // `:21040-21048`: the lifecycle re-sync.
        if self.dash_delay == 0 {
            self.dash = self.dash_type;
        }
        if self.dash == 0 {
            self.dash_time = 0;
            self.dash_delay = 0;
            return;
        }
        if self.dash_delay > 0 {
            // `:21237-21248`: the cooldown ticks; the EoC grace drains with it.
            if self.eoc_dash > 0 {
                self.eoc_dash -= 1;
            }
            if self.eoc_dash == 0 {
                self.eoc_hit = -1;
            }
            self.dash_delay -= 1;
            return;
        }
        if self.dash_delay < 0 {
            // `:21249-21399`: the active dash — the duration is a speed-decay
            // model, not a tick timer.
            let mut threshold = 12.0f32;
            let mut fast = 0.992f32;
            let run_band = self.acc_run_speed.max(self.max_run_speed);
            let mut mid = 0.96f32;
            let mut cooldown = 20;
            match self.dash {
                // `:21271-21285`: the shield decays faster, cooldowns longer.
                2 => {
                    fast = 0.985;
                    mid = 0.94;
                    cooldown = 30;
                }
                // `:21286-21307`: the Solar dash rides higher and longer.
                3 => {
                    threshold = 14.0;
                    fast = 0.985;
                    mid = 0.94;
                }
                // `:21346-21351`: a direction flip or losing the capability
                // ends a mount dash at 30 with a damp.
                6 => {
                    if self.vx.signum() as i32 != self.facing || self.dash_type != 6 {
                        self.dash_delay = 30;
                        self.vx *= 0.3;
                        self.dash_time = 0;
                        return;
                    }
                }
                _ => {}
            }
            if self.dash <= 0 {
                return;
            }
            if self.vx > threshold || self.vx < -threshold {
                self.vx *= fast;
                return;
            }
            if self.vx > run_band || self.vx < -run_band {
                self.vx *= mid;
                return;
            }
            // `:21390-21398`: the dash ends — the cooldown starts and the
            // velocity pins to the run band in the held/facing direction.
            self.dash_delay = cooldown;
            if self.vx < 0.0 || (self.vx == 0.0 && input.left) {
                self.vx = -run_band;
            } else if self.vx > 0.0 || (self.vx == 0.0 && input.right) {
                self.vx = run_band;
            }
            return;
        }
        // `:21400-21537`: the dash start (`dashDelay == 0`), per type.
        let Some((dir, dashing)) = self.do_common_dash_handle(input) else {
            return;
        };
        if !dashing {
            return;
        }
        let speed = match self.dash {
            1 => 16.9,
            2 => 14.5,
            3 => 21.9,
            // `:21520-21523`: the mount hop at the start.
            6 => {
                if self.vy == 0.0 {
                    self.vy = -4.0;
                }
                16.0
            }
            _ => return,
        };
        self.vx = speed * dir as f32;
        // `:21414-21417`: a solid or sloped tile ahead halves the launch.
        if self.solid_ahead(world, dir) {
            self.vx /= 2.0;
        }
        self.dash_delay = -1;
        if self.dash == 2 {
            self.eoc_dash = 15;
        }
    }

    /// `Player.DoCommonDashHandle` (`Player.cs:21546-21605`): the 15-tick
    /// double-tap window (`Settings.DashControl == AllowDoubleTap`, the
    /// default) — the first tap arms `dashTime = ±15`, the same-direction
    /// press edge inside the window fires. The dedicated dash button
    /// (`controlDash`) has no port input binding yet.
    fn do_common_dash_handle(&mut self, input: PlayerInput) -> Option<(i32, bool)> {
        if self.dash_time > 0 {
            self.dash_time -= 1;
        }
        if self.dash_time < 0 {
            self.dash_time += 1;
        }
        let right_edge = input.right && !self.right_prev;
        let left_edge = input.left && !self.left_prev;
        if right_edge {
            if self.dash_time > 0 {
                return Some((1, true));
            }
            self.dash_time = 15;
        } else if left_edge {
            if self.dash_time < 0 {
                return Some((-1, true));
            }
            self.dash_time = -15;
        }
        None
    }

    /// `WorldGen.SolidOrSlopedTile` on the two launch-probe points
    /// (`Player.cs:21412-21414`): the hitbox's leading edge at the upper
    /// body and the center rows.
    fn solid_ahead(&self, world: &World, dir: i32) -> bool {
        let cx = self.x + WIDTH * 0.5;
        let cy = self.y + self.height * 0.5;
        let tx = ((cx + dir as f32 * (WIDTH * 0.5 + 2.0)) / TILE_PX) as i32;
        let ty0 = ((cy - self.height * 0.5 + 2.0) / TILE_PX) as i32;
        let ty1 = (cy / TILE_PX) as i32;
        [ty0, ty1].into_iter().any(|ty| {
            world.tile(tx, ty).is_some_and(|t| {
                t.active() && (tr_core::tiledata::TILE_SOLID.has(t.kind) || t.slope() != 0)
            })
        })
    }

    /// `Item.wingSlot` (`Item.cs`'s `SetDefaults` cases — the 47 wings whose
    /// `case` carries the assignment; the three that set it from a shared
    /// helper stay 0 for now). The port reads it where the C# reads
    /// `currentItem.wingSlot` (`Player.cs:15593-15602`).
    pub fn wing_slot(item: i32) -> i32 {
        match item {
            492 => 1,
            493 => 2,
            665 => 3,
            748 => 4,
            749 => 5,
            761 => 6,
            785 => 7,
            786 => 8,
            821 => 9,
            822 => 10,
            823 => 11,
            948 => 12,
            1162 => 13,
            1165 => 14,
            1515 => 15,
            1583 => 16,
            1584 => 17,
            1585 => 18,
            1586 => 19,
            1797 => 20,
            1830 => 21,
            1866 => 22,
            1871 => 23,
            2280 => 24,
            2494 => 25,
            2609 => 26,
            2770 => 27,
            3228 => 28,
            3580 => 33,
            3582 => 34,
            3588 => 35,
            3592 => 36,
            3883 => 37,
            3924 => 38,
            3928 => 39,
            4730 => 40,
            4746 => 41,
            4750 => 42,
            4754 => 43,
            4823 => 44,
            4954 => 45,
            4978 => 46,
            5586 => 47,
            5627 => 48,
            5659 => 49,
            5686 => 50,
            6140 => 51,
            _ => 0,
        }
    }

    /// `ArmorIDs.Wing.Sets.Stats` via `WingStatsInitializer.cs:11-61`:
    /// `(fly_time, acc_run_override, accel_mult, has_hover, hover_speed,
    /// hover_accel)`. `-1` = no run-speed override. The dev/misc wings share
    /// the 150/7 config.
    pub fn wing_stats(id: i32) -> (i32, f32, f32, bool, f32, f32) {
        match id {
            46 => (25, 3.0, 1.0, false, -1.0, 1.0),
            1 | 2 => (100, 6.25, 1.0, false, -1.0, 1.0),
            6 | 7 | 10 | 25 => (130, 6.75, 1.0, false, -1.0, 1.0),
            4 => (150, 6.5, 1.0, false, -1.0, 1.0),
            5 | 9 | 13 | 14 | 15 => (160, 7.5, 1.0, false, -1.0, 1.0),
            8 | 11 | 24 | 27 => (170, 7.5, 1.0, false, -1.0, 1.0),
            22 => (170, 6.5, 1.0, true, 10.0, 10.0),
            12 | 20 | 21 | 23 => (180, 7.5, 1.0, false, -1.0, 1.0),
            26 => (180, 8.0, 2.0, false, -1.0, 1.0),
            37 => (150, 7.0, 2.5, true, 12.0, 12.0),
            44 => (150, 8.0, 2.0, false, -1.0, 1.0),
            45 => (180, 8.0, 4.5, true, 16.0, 16.0),
            29 | 32 => (180, 9.0, 2.5, false, -1.0, 1.0),
            30 | 31 => (180, 6.5, 1.5, true, 12.0, 12.0),
            3 | 16..=19 | 28 | 33..=43 | 48 | 49 => (150, 7.0, 1.0, false, -1.0, 1.0),
            _ => (0, -1.0, 1.0, false, -1.0, 1.0),
        }
    }

    /// `WingMovement`'s per-tier constants (`Player.cs:22023-22243`):
    /// `(num, num2, num3, num4, num5)` — the ascent impulse, the falling
    /// brake, the upward cap multiplier, the rising brake threshold, the
    /// rising impulse.
    fn wing_constants(id: i32) -> (f32, f32, f32, f32, f32) {
        match id {
            26 | 37 => (0.125, 0.75, 2.5, 1.0, 0.15),
            8 | 11 | 22 | 24 | 27 => (0.1, 0.5, 1.66, 0.5, 0.1),
            12 | 20 | 21 | 23 => (0.1, 0.5, 1.805, 0.5, 0.1),
            29 | 32 => (0.135, 0.85, 3.0, 1.0, 0.15),
            30 | 31 => (0.15, 0.5, 2.45, 1.0, 0.1),
            44 | 45 => (0.1, 0.95, 4.5, 1.0, 0.15),
            _ => (0.1, 0.5, 1.5, 0.5, 0.1),
        }
    }

    /// The hold-down hover wing set (`Player.cs:27405`, the dampening list).
    fn hover_dampening_wing(id: i32) -> bool {
        matches!(id, 22 | 28 | 30 | 31 | 33 | 35 | 37 | 45)
    }

    /// `Player.WingAirLogicTweaks` (`Player.cs:29304-29323`): the wing's run
    /// speed override and acceleration multiplier, the hover variants while
    /// hold-down-hovering. Runs before `HorizontalMovement`.
    fn wing_air_logic_tweaks(&mut self) {
        let (_fly, acc_override, accel_mult, has_hover, hover_speed, hover_accel) =
            Self::wing_stats(self.wings_logic);
        let hovering = self.trying_to_hover_down()
            && self.control_jump
            && self.wing_time > 0.0
            && !self.is_performing_down_dash();
        if has_hover && hovering {
            if hover_speed != -1.0 {
                self.acc_run_speed = hover_speed;
            }
            self.run_acceleration *= hover_accel;
        } else {
            if acc_override != -1.0 && acc_override > self.acc_run_speed {
                self.acc_run_speed = acc_override;
            }
            self.run_acceleration *= accel_mult;
        }
    }

    /// `Player.TryingToHoverDown` (`Player.cs:3934-3941`): controlDown or the
    /// latch.
    fn trying_to_hover_down(&self) -> bool {
        self.control_down || self.try_keeping_hovering_down
    }

    /// `Player.TryingToHoverUp` (`Player.cs:3924-3933`): the up latch.
    fn trying_to_hover_up(&self) -> bool {
        self.try_keeping_hovering_up
    }

    /// The `isPerformingJump_DownDash` stand-in: the flag lands with the
    /// Dead-Cells dash (not in this task's scope), so it is always false.
    fn is_performing_down_dash(&self) -> bool {
        false
    }

    /// `Player.CCed` (`Player.cs:3377-3385`): frozen, webbed or stoned.
    pub fn cced(&self) -> bool {
        self.frozen || self.webbed || self.buffs.find_buff_index(156).is_some()
    }

    /// `Player.ClearBuff` (`Player.cs:5242-5256`): every slot carrying the
    /// buff type.
    fn clear_buff_type(&mut self, bt: i32) {
        while let Some(i) = self.buffs.find_buff_index(bt) {
            self.buffs.del_buff(i);
        }
    }

    /// `Mount.SetMount` (`Mount.cs:6196-6305`) via the player: the state
    /// swap, the buff clear/add (`:6203`/`:6226`) and the bottom-anchored
    /// hitbox growth (`:6264-6277` — no PlayerSize delegate exists in this
    /// set; the type-44 dash reset `:6228-6235` cannot fire without that row).
    pub fn set_mount(&mut self, m: i32, wet: bool) -> bool {
        let Some(equip) = self.mount.set_mount(m, wet) else {
            return false;
        };
        if let Some(old) = equip.clear_buff {
            self.clear_buff_type(old);
        }
        self.buffs.add_buff(equip.add_buff, 3600);
        self.grow_hitbox(HEIGHT + equip.height_boost as f32);
        true
    }

    /// `:6270-6275`: the height change anchors the bottom (position.Y += h;
    /// height = new; position.Y -= h).
    fn grow_hitbox(&mut self, new_height: f32) {
        self.y += self.height;
        self.height = new_height;
        self.y -= self.height;
    }

    /// `Mount.Dismount` (`Mount.cs:6142-6194`) via the player: the buff clear
    /// and the hitbox restore. `ignore_effect` stands in for the spawn dust
    /// (`:6167-6170` — no dust system yet).
    pub fn dismount(&mut self, world: &World, ignore_effect: bool) -> bool {
        let Some(buff) = self.mount.dismount() else {
            return false;
        };
        let _ = ignore_effect;
        self.clear_buff_type(buff);
        // `:6176-6193`: the restore — the 20x42 box from the bottom center;
        // on failure the Y patch accepts the overlap (the collision step
        // resolves it next tick, as in the original). The resize goes through
        // the single `TryChangingSizeFromBottomCenter` port (world-aware).
        if let Some([_, ny]) = collision::try_changing_size_from_bottom_center(
            world,
            [self.x, self.y],
            [WIDTH, self.height],
            [WIDTH, HEIGHT],
        ) {
            self.y = ny;
            self.height = HEIGHT;
        } else {
            self.y += self.height;
            self.height = HEIGHT;
            self.y -= self.height;
        }
        true
    }

    /// `Mount.TryDismount` (`Mount.cs:6127-6140`) with `CanDismountWithResult`
    /// (`:6611-6624`): the CC gate cannot fire on this table (no row sets
    /// `DontDismountWhenCCed`) and the shrink probe always fits, so the
    /// dismount runs; Phase 3's quick-mount counts the failures.
    pub fn try_dismount(&mut self, world: &World) -> bool {
        self.dismount(world, false)
    }

    /// `Mount.CanMount` (`Mount.cs:6581-6604`): the grown box must fit in
    /// space (`CanFitInSpaceWithSize` -> `Collision.IsClearSpotTest`, stand-in
    /// [`collision::collides_rect`]), the bat (56) refuses water (`:6595`) and
    /// a grapple in flight blocks every mount outside `Cart`/`CanUseHooks`
    /// (`:6599-6601`). The PlayerSize-delegate arm cannot fire on this table.
    pub(crate) fn can_mount(&self, world: &World, m: i32) -> bool {
        let Some(row) = crate::mount::mount_data(m) else {
            return false;
        };
        let size_h = self.height + row.height_boost as f32;
        if collides_rect(world, self.x, self.y + self.height - size_h, WIDTH, size_h) {
            return false;
        }
        // `:6595`: `Collision.WetCollision(Bottom - size * 0.5, size)` - the
        // probe is the grown box centred on the body's bottom point.
        if m == 56
            && (self.wet
                || self.dripping()
                // `Bottom.X - size.X / 2` = `position.X` for the fixed width.
                || wet_collision(
                    world,
                    self.x,
                    self.y + self.height - size_h / 2.0,
                    WIDTH,
                    size_h,
                ))
        {
            return false;
        }
        // `:6599`: `!Cart[m] && !CanUseHooks[m] && grappling[0] >= 0`.
        if !crate::mount::is_cart(m)
            && !crate::mount::can_use_hooks(m)
            && self.grappling_active
        {
            return false;
        }
        true
    }

    /// `Player.dripping` (`Player.cs:11743-11747`): buff 103 (`Wet`) keeps the
    /// player dripping until lava deletes it; the bat's wet gate reads it.
    fn dripping(&self) -> bool {
        self.holding(103) && !self.lava_wet
    }

    /// `WorldGen.InAPlaceWithWind(position, width, height)`
    /// (`WorldGen.cs:86841-86846`): the tile rectangle the body covers.
    fn wind_here(&self, world: &World, wind: Option<&tr_core::wind::Wind>) -> bool {
        let Some(wind) = wind else {
            return false;
        };
        let x0 = (self.x / TILE_PX) as i32;
        let y0 = (self.y / TILE_PX) as i32;
        let x1 = ((self.x + WIDTH) / TILE_PX) as i32;
        let y1 = ((self.y + self.height) / TILE_PX) as i32;
        wind.in_a_place_with_wind(world, x0, y0, 1 + x1 - x0, 1 + y1 - y0)
    }

    /// `Player.MountFishronSpecial` (`Player.cs:4044-4056`): full run stats for
    /// the Cute Fishron (12) while hurt, not in plain water (lava/honey are
    /// exempt), dripping, inside the 60/420-tick window, or under rain that
    /// reaches the body.
    fn mount_fishron_special(
        &self,
        world: &World,
        raining: bool,
        wind: Option<&tr_core::wind::Wind>,
    ) -> bool {
        if self.hp >= self.max_hp / 2
            && (!self.wet || self.lava_wet || self.honey_wet)
            && !self.dripping()
            && !(self.mount_fishron_special_counter > 0.0)
        {
            return raining && self.wind_here(world, wind);
        }
        true
    }

    /// `Player.QuickMount` (`Player.cs:6045-6100`): the toggle — dismount, or
    /// equip the first saddle in the inventory. `QuickMount_GetItemToUse`
    /// (`:6179-6198`) walks the inventory (the `miscEquips[3]` slot is not
    /// ported — M3-EQUIP); the cart items are skipped by the table itself (no
    /// cart rows) and the `QuickMinecart` fallback waits for M2-MOUNT-3. The
    /// FailedNoSpace branch (`:6052-6070` — the 3-fail force + the
    /// `PlayerNoSpaceTeleport`) is unreachable for this set: every row's
    /// `heightBoost >= 0` makes the dismount probe a shrink, which always
    /// fits. Returns the saddle item kind used (for the use animation).
    pub fn quick_mount(
        &mut self,
        world: &World,
        inventory: &[crate::item::ItemSlot],
    ) -> Option<u16> {
        if self.mount.is_active() {
            self.try_dismount(world);
            return None;
        }
        // `:6074-6077`: the guards (tongued/gravDir/noItems have no port arms).
        if self.cced() || self.dead {
            return None;
        }
        let item = inventory
            .iter()
            .find(|slot| slot.count > 0 && crate::mount::mount_type_of_item(slot.kind).is_some());
        let Some(slot) = item else {
            return None; // `:6097` `QuickMinecart` — M2-MOUNT-3
        };
        let Some(m) = crate::mount::mount_type_of_item(slot.kind) else {
            return None;
        };
        if !self.can_mount(world, m) {
            return None;
        }
        if self.set_mount(m, self.wet) {
            Some(slot.kind)
        } else {
            None
        }
    }

    /// The per-tick mount block — `Player.cs:26623-26688` with the
    /// `UpdateEffects` head (`Mount.cs:4795-4803`, `:4949-5008`) and the
    /// `UpdateJumpHeight` mount merge (`:26011` → `:19619-19637`). Vanilla
    /// runs those at :25656/:26011, both before the run-speed merge — the
    /// combined method keeps their relative order. Only reachable while not
    /// grappling (the caller's `:26622` branch).
    fn update_mount(
        &mut self,
        world: &World,
        raining: bool,
        wind: Option<&tr_core::wind::Wind>,
    ) {
        if !self.mount.is_active() {
            return;
        }
        // `Mount.cs:4797`: the mount head-sets the auto-jump flag per tick.
        self.auto_jump = self.mount.auto_jump();
        // `Mount.cs:4982-5008` (`UpdateEffects` case 12): the Cute Fishron
        // re-arms its special window when hurt or standing in water/rain.
        if self.mount.kind() == 12 {
            if self.hp <= self.max_hp / 2 {
                self.mount_fishron_special_counter = 60.0;
            }
            if self.wet || (raining && self.wind_here(world, wind)) {
                self.mount_fishron_special_counter = 420.0;
            }
        }
        // `Mount.cs:4949-4981`: the ground-dash mounts arm their extra jumps
        // (the PHYS-8 priority chain consumes them) and pin knockback off
        // above the charge speed.
        match self.mount.kind() {
            10 => self.has_jump_options[ExtraJump::Unicorn.index()] = true,
            47 => self.has_jump_options[ExtraJump::WallOfFleshGoat.index()] = true,
            14 => self.has_jump_options[ExtraJump::Basilisk.index()] = true,
            _ => {}
        }
        if matches!(self.mount.kind(), 10 | 47 | 14 | 40 | 41 | 42)
            && self.vx.abs() > self.mount.dash_speed() - self.mount.run_speed() / 2.0
        {
            self.no_knockback = true;
        }
        // `Player.cs:19619-19631`: the jump statics — replace, or additive
        // when the mount says so; `:19635-19637` the sticky clamp after (the
        // `dazed` clamp `:19638-19641` has no port counterpart).
        if self.mount.movement_stats_are_additive() {
            self.jump_height += self.mount.jump_height(self.vx);
            self.jump_speed += self.mount.jump_speed(self.vx);
        } else {
            self.jump_height = self.mount.jump_height(self.vx);
            self.jump_speed = self.mount.jump_speed(self.vx);
        }
        if self.sticky {
            self.jump_height /= 10;
            self.jump_speed /= 5.0;
        }
        // `:26625-26635`: the wings/rocket suppression + the slowdown floor.
        if !self.mount.can_use_wings() {
            self.run_slowdown = 0.2;
        }
        self.rocket_boots = 0;
        if !self.mount.can_use_wings() {
            self.has_wings = false;
            self.wings_logic = 0;
        }
        // `:26636` + `:26637-26654`: the three-way run-stat merge.
        let flag17 = self.mount.can_use_wings() && self.wings_logic > 0 && self.vy != 0.0;
        if self.mount.movement_stats_are_additive() {
            self.max_run_speed += self.mount.run_speed();
            self.acc_run_speed += self.mount.dash_speed();
            self.run_acceleration += self.mount.acceleration();
        } else if flag17 {
            self.max_run_speed = self.max_run_speed.max(self.mount.run_speed());
            self.acc_run_speed = self.acc_run_speed.max(self.mount.dash_speed());
            self.run_acceleration = self.run_acceleration.max(self.mount.acceleration());
        } else {
            self.max_run_speed = self.mount.run_speed();
            self.acc_run_speed = self.mount.dash_speed();
            self.run_acceleration = self.mount.acceleration();
        }
        // `:26655-26659`: the Cute Fishron runs at half acceleration/top
        // speed unless `MountFishronSpecial` is true.
        if self.mount.kind() == 12 && !self.mount_fishron_special(world, raining, wind) {
            self.run_acceleration /= 2.0;
            self.max_run_speed /= 2.0;
        }
        // `:26660-26663` (the roller skates), `:26665-26682` (the cart/OnTrack
        // block — M2-MOUNT-3) and `:26683-26686` (`UpdateDrill` —
        // M2-MOUNT-4) have no rows in this table yet; the ability counters
        // still tick (`:26663`).
        self.mount.ability_recovery();
    }

    /// The flight section (`Player.cs:26772-27425`): `canRocket`, the ground
    /// refills, `flag19`, `WingMovement`, the rocket trigger + thrust, the
    /// glide. Returns true when it integrated vy itself (the plain gravity
    /// is skipped this tick, like the original's else-if chain).
    fn flight_section(
        &mut self,
        world: &World,
        input: PlayerInput,
        gravity: f32,
        fall_cap: f32,
    ) -> bool {
        // `:26745-26752`: an unequipped tier zeroes its budget.
        if self.wings_logic == 0 {
            self.wing_time = 0.0;
        }
        if self.rocket_boots == 0 {
            self.rocket_time = 0;
        }
        // `:26772-26775`: airborne outside the initial jump burst.
        if self.vy > -self.jump_speed && self.vy != 0.0 {
            self.can_rocket = true;
        }
        // `:26776-26781`: the wing refill on the ground needs a jump release —
        // or an auto-jump (constant-jump) mount re-jumping this tick; the
        // mount flight budget refills with it (`mount.ResetFlightTime`).
        if (self.vy == 0.0 && self.release_jump) || (self.auto_jump && self.just_jumped) {
            self.wing_time = self.wing_time_max as f32;
            self.mount.reset_flight_time(self.vx);
        }
        // `Player.cs:24761`: every resting tick recovers 2 hover fatigue.
        if self.vy == 0.0 {
            self.mount.fatigue_recovery();
        }
        // `:26859-26862`: the rocket fuel refills on any rest tick (or an
        // auto-jump mount's re-jump).
        if self.vy == 0.0 || (self.auto_jump && self.just_jumped) {
            self.rocket_time = self.rocket_time_max;
        }
        // `:26786-26798`: flag19 — actively flying this tick. The second
        // condition is the hold-down-hover extension for the hover wings.
        let mut flag19 = self.wings_logic > 0
            && self.control_jump
            && self.wing_time > 0.0
            && self.jump == 0
            && self.vy != 0.0;
        if self.wings_logic > 0
            && self.control_jump
            && self.trying_to_hover_down()
            && self.wing_time > 0.0
            && matches!(self.wings_logic, 22 | 28 | 30 | 32 | 29 | 33 | 35 | 37 | 45)
        {
            flag19 = true;
        }
        // `:26825-26834`: with both equipped the rocket fuel converts into
        // wing time (capped past the max).
        if self.wings_logic > 0 && self.rocket_boots != 0 && self.vy != 0.0 && self.rocket_time != 0
        {
            let add = self.rocket_time * 6;
            self.wing_time =
                (self.wing_time + add as f32).min(self.wing_time_max as f32 + add as f32);
            self.rocket_time = 0;
        }
        if flag19 {
            self.wing_movement(input);
        }
        // `:26867-26902`: the rocket trigger — only once the wing budget is
        // spent (or absent); the arming needs a midair jump release.
        if (self.wing_time == 0.0 || self.wings_logic == 0)
            && self.rocket_boots != 0
            && self.control_jump
            && self.rocket_delay == 0
            && self.can_rocket
            && self.rocket_release
        {
            if self.rocket_time > 0 {
                self.rocket_time -= 1;
                self.rocket_delay = 10;
                if self.rocket_delay2 <= 0 {
                    self.rocket_delay2 = if self.rocket_boots == 1 { 30 } else { 15 };
                }
            } else {
                self.can_rocket = false;
            }
        }
        // `:26903-26914`: the flame cadence delay (the sound delay is audio).
        if self.rocket_delay2 > 0 {
            self.rocket_delay2 -= 1;
        }
        // `:26915-26955`: the rocket thrust — 10 ticks per fuel tick.
        if self.rocket_delay > 0 {
            self.rocket_delay -= 1;
            self.vy -= 0.1;
            if self.vy > 0.0 {
                self.vy -= 0.5;
            } else if self.vy > -self.jump_speed * 0.5 {
                self.vy -= 0.1;
            }
            if self.vy < -self.jump_speed * 1.5 {
                self.vy = -self.jump_speed * 1.5;
            }
            // `Player.cs:25379-25382`: the thrusting fall origin refreshes.
            self.fall_start = (self.y / TILE_PX) as i32;
            return true;
        }
        if !flag19 {
            // `:26962-26964`: the hover mounts (`Mount.Hover`, `Mount.cs:3340`)
            // — the branch carries the vertical move itself, so it is
            // consumed like the vanilla `else if` chain.
            if self.mount.can_hover() {
                self.vy = self.mount.hover_integrate(
                    self.vy,
                    false, // no `controlUp` key ported (M2-MOUNT-4)
                    self.control_jump,
                    self.control_down,
                    gravity,
                );
                // `:3459`: the fall origin refreshes every hover tick (no
                // fall damage accrues while hovering).
                self.fall_start = (self.y / TILE_PX) as i32;
                return true;
            }
            // `:26965-27015`: the mount flight thrust — the flying mounts'
            // replacement for the wing ascent while the budget lasts.
            if self.mount.can_fly() && self.control_jump && self.jump == 0 {
                if self.mount.flight() {
                    if self.trying_to_hover_down() {
                        // `:26968-26974`: the hover-down dampening.
                        self.vy *= 0.9;
                        if self.vy > -1.0 && self.vy < 0.5 {
                            self.vy = 1e-5;
                        }
                    } else {
                        // `:26975-27000`: the rise thrust — the falling
                        // brake, the rising assist, the upward cap. The
                        // type-50 half cap references the full table; only
                        // Queen Slime (50) exists here (56/61 are M2-MOUNT-2).
                        let num40 = 0.1;
                        let mut num41 = self.jump_speed;
                        if self.mount.kind() == 50 {
                            num41 *= 0.5;
                        }
                        // `:26981`: the Bat/Pixie rise cap is 2/3 of the
                        // others.
                        if matches!(self.mount.kind(), 56 | 61) {
                            num41 /= 1.5;
                        }
                        if self.vy > 0.0 {
                            self.vy -= num40 * 5.0;
                        } else if self.vy > -num41 * 1.5 {
                            self.vy -= num40;
                        }
                        if self.vy < -num41 * 1.5 {
                            self.vy = -num41 * 1.5;
                        }
                    }
                    return true;
                }
                // `:27001-27014`: the budget spent — gravity / 3 with the
                // fall cap / 3 (the upward half needs gravDir, not ported).
                self.vy += gravity / 3.0;
                if self.vy > fall_cap / 3.0 && !self.trying_to_hover_down() {
                    self.vy = fall_cap / 3.0;
                }
                return true;
            }
            // `:27034-27037 + 27339-27350`: the wing glide — jump held while
            // falling: gravity / 3, the fall cap / 3, the fall origin
            // refreshed per tick (no fall damage accrues).
            if self.wings_logic > 0 && self.control_jump && self.vy > 0.0 {
                self.vy += gravity / 3.0;
                if self.vy > fall_cap / 3.0 {
                    self.vy = fall_cap / 3.0;
                }
                self.fall_start = (self.y / TILE_PX) as i32;
                self.hover_dampening(input);
                return true;
            }
            return false;
        }
        self.hover_dampening(input);
        true
    }

    /// `Player.WingMovement` (`Player.cs:22023-22243`): the ascent physics —
    /// the impulse, the falling brake, the rising brake, the upward cap, the
    /// budget countdown (half while hover-down with no horizontal input, 2x
    /// for the jetpack), and the wings-44/45 hover thrusts.
    fn wing_movement(&mut self, input: PlayerInput) {
        let (num, mut num2, mut num3, mut num4, mut num5) = Self::wing_constants(self.wings_logic);
        let hover_down = self.trying_to_hover_down() && !input.left && !input.right;
        if self.wings_logic == 4 {
            // `:22025-22058`: the jetpack hover-up burns 2 ticks per tick.
            if self.trying_to_hover_up() {
                self.vy -= 0.2;
                if self.vy > 0.0 {
                    self.vy -= 0.5;
                } else if self.vy > -JUMP_SPEED * 0.5 {
                    self.vy -= 0.1;
                }
                if self.vy < -JUMP_SPEED * 3.0 {
                    self.vy = -JUMP_SPEED * 3.0;
                }
                self.wing_time -= 2.0;
                return;
            }
        }
        if self.wings_logic == 45 {
            // `:22137-22180`: the hover thrusts (44 shares the numbers).
            if self.trying_to_hover_up() {
                self.vy -= 0.4;
                if self.vy > 0.0 {
                    self.vy -= 0.5;
                } else if self.vy > -JUMP_SPEED * 0.5 {
                    self.vy -= 0.2;
                }
                if self.vy < -JUMP_SPEED * 3.0 {
                    self.vy = -JUMP_SPEED * 3.0;
                }
            }
            if self.trying_to_hover_down() && !input.jump && self.vy != 0.0 {
                self.vy += 0.4;
            }
        }
        let _ = num2;
        self.vy -= num;
        if self.vy > 0.0 {
            self.vy -= num2;
        } else if self.vy > -self.jump_speed * num4 {
            self.vy -= num5;
        }
        if self.vy < -self.jump_speed * num3 {
            self.vy = -self.jump_speed * num3;
        }
        // `:22230-22237`: the budget countdown.
        if hover_down && matches!(self.wings_logic, 22 | 28 | 30 | 31 | 37 | 45) {
            self.wing_time -= 0.5;
        } else {
            self.wing_time -= 1.0;
        }
    }

    /// `Player.cs:27404-27425`: the post-block hover dampening — the hover
    /// wings holding down multiply vy toward the float.
    fn hover_dampening(&mut self, input: PlayerInput) {
        let can_use = self.has_wings && !self.is_performing_down_dash();
        if Self::hover_dampening_wing(self.wings_logic)
            && self.trying_to_hover_down()
            && self.control_jump
            && self.wing_time > 0.0
            && can_use
        {
            let num67 = if self.wings_logic == 45 { 0.8 } else { 0.9 };
            self.vy *= num67;
            if self.vy > -2.0 && self.vy < 1.0 {
                self.vy = 1e-5;
            }
        }
    }

    /// `Player.GrappleMovement` (`Player.cs:22613-22740`): the pull replaces
    /// the velocity; the jump-off detaches. `hook` is `(center, type)` of the
    /// attached hook (the param already filters to the attached state).
    fn grapple_movement(
        &mut self,
        hooks: &[(u16, Vec2, f32)],
        input: PlayerInput,
        item_in_use: bool,
    ) {
        // `GetGrapplingForces` (`Player.cs:22757-22856`): zero registered hooks
        // carries the current velocity through untouched.
        if hooks.is_empty() {
            return;
        }
        // `RefreshMovementAbilities` (`Player.cs:22858-22867`): the flight
        // budgets refill every tick while grappling.
        self.wing_time = self.wing_time_max as f32;
        self.rocket_time = self.rocket_time_max;
        self.rocket_delay = 0;
        self.refresh_double_jumps();
        // `:22645`: the fall origin refreshes per tick — no fall damage.
        self.fall_start = (self.y / TILE_PX) as i32;

        // `GetGrapplingForces` (`Player.cs:22757-22856`): the steering-adjusted
        // average of every registered hook's center, clamped to
        // `grappling[0]`'s type pull speed. The tuple's third element is the
        // hook's rotation (only the Squirrel Hook reads it).
        let pull = crate::grapple::grapple_pull(
            hooks,
            Vec2::new(self.x + WIDTH * 0.5, self.y + self.height * 0.5),
            // `PlayerInput` has no `up` arm (nothing consumes it in the
            // port); the C# reads `controlUp` here.
            crate::grapple::steering_stick(input.left, input.right, false, input.down),
        );
        let Some(pull) = pull else {
            // `:22845-22847`: no registered hook carries the velocity through.
            return;
        };
        // `:22659`: `flag3` = the pull points down while the CURRENT velocity is
        // zero (a grounded pull) — read before the replacement.
        let flag3 = pull.vel.y > 0.0 && self.vy == 0.0;
        // `:22660-22661`: the pull replaces the velocity outright.
        self.vx = pull.vel.x;
        self.vy = pull.vel.y;
        // `:22655-22658`: the pull points downward → the platforms fall
        // through (`GoingDownWithGrapple`).
        self.going_down_with_grapple = pull.going_down;

        // `:22670-22676`: the direction follows the pull; an exactly-zero X
        // takes the Squirrel Hook's preferred direction instead. The C# gates
        // the block on `itemAnimation == 0`, which the sim passes in.
        if !item_in_use {
            if self.vx == 0.0 {
                if let Some(d) = pull.dir {
                    self.facing = d;
                }
            }
            if self.vx > 0.0 {
                self.facing = 1;
            } else if self.vx < 0.0 {
                self.facing = -1;
            }
        }

        // `:22705-22739`: the jump-off.
        if input.jump {
            if self.release_jump {
                // `flag` (`:22627`): nearly stationary.
                let flag = Vec2::new(self.vx, self.vy).length() < 2.0;
                // `flag2` (`:22628`): floating in water with near-zero vy.
                let flag2 = self.in_water && self.vy.abs() < 0.02;
                let mut flag4 = flag2 || flag;
                if input.down {
                    flag4 = false;
                }
                if flag3 && !self.control_jump {
                    flag4 = false;
                }
                if flag4 {
                    self.vy = -self.jump_speed;
                    self.jump = self.jump_height;
                    self.release_jump = false;
                } else {
                    self.vy += 0.01;
                    self.release_jump = false;
                }
                self.refresh_double_jumps();
                // `RemoveAllGrapplingHooks` (`Player.cs:33562-33566`): the
                // caller observes `grappling_active == false` and drops the
                // hook.
                self.grappling_active = false;
            }
        } else {
            self.release_jump = true;
        }
    }

    /// `Player.HorizontalMovement` (`Player.cs:19772-20090`), the exact if/else
    /// chain: the pending booster push applies pre-input and clamps
    /// (`:19818-19833`), the cart arms gate their accel on `velocity.Y == 0`
    /// (`:19849/:19907`), the off-track brakes pull the roll toward rest
    /// (`:19858-19865`, `:19948-19956`), the cart coast arm clamps ±max without
    /// on-track friction (`:20023-20058`), and the grounded/airborne friction
    /// tails close the chain (`:20059-20087`).
    fn horizontal_movement(&mut self) {
        // `Player.cs:19818-19833`: the boost applies before any input reads
        // the velocity, then clamps.
        if self.track_boost != 0.0 {
            self.vx += self.track_boost;
            self.track_boost = 0.0;
            if self.vx < 0.0 {
                if self.vx < -self.max_run_speed {
                    self.vx = -self.max_run_speed;
                }
            } else if self.vx > self.max_run_speed {
                self.vx = self.max_run_speed;
            }
        }
        // `Player.cs:19835-19843`: the direction lock — the facing follows the
        // run keys (`ChangeDir(num5)`); a cart keeps the roll direction, read
        // AFTER the boost above (`Math.Sign(velocity.X) == num5`; `Sign(0) =
        // 0` locks a stationary cart; `AllowDirectionChange` is true for
        // carts, `Mount.cs:630-639`).
        let dir_keys = self.control_right as i32 - self.control_left as i32;
        if dir_keys != 0 {
            let roll = (self.vx > 0.0) as i32 - (self.vx < 0.0) as i32;
            if !self.cart_mounted || roll == dir_keys {
                self.facing = dir_keys;
            }
        }
        if self.control_left && self.vx > -self.max_run_speed {
            // `:19848-19857`: the cart accel gate — airborne carts get no
            // input accel that tick.
            if !self.cart_mounted || self.vy == 0.0 {
                if self.vx > self.run_slowdown {
                    self.vx -= self.run_slowdown;
                }
                self.vx -= self.run_acceleration;
            }
            // `:19858-19865`: the off-track brake.
            if self.on_wrong_ground {
                if self.vx < -self.run_slowdown {
                    self.vx += self.run_slowdown;
                } else {
                    self.vx = 0.0;
                }
            }
            // `Player.cs:19868-19902`: the rolling wheel sparks + Item55 —
            // the arm's gate is the direction-OPPOSITE sign (`velocity.X >=
            // 0f`): the sound + sparks fire while braking a rightward roll or
            // at rest, not while accelerating left. (`itemAnimation == 0`
            // holds — the port's item swing never gates movement.)
            if self.cart_mounted
                && !self.on_wrong_ground
                && self.on_track
                && self.vy == 0.0
                && self.vx >= 0.0
            {
                let pos = Vec2::new(self.x, self.y);
                let vel = Vec2::new(self.vx, self.vy);
                let body_h = self.body_height() as i32;
                let max_run = self.max_run_speed;
                let rotation = self.full_rotation;
                minecart::rolling_wheel_sparks(
                    &mut self.pending_cart_sounds,
                    &mut self.pending_sparks,
                    &mut self.spark_rng,
                    pos,
                    WIDTH as i32,
                    body_h,
                    vel,
                    max_run,
                    rotation,
                );
            }
        } else if self.control_right && self.vx < self.max_run_speed {
            // `:19904-19916`: the mirror arm.
            if !self.cart_mounted || self.vy == 0.0 {
                if self.vx < -self.run_slowdown {
                    self.vx += self.run_slowdown;
                }
                self.vx += self.run_acceleration;
            }
            // `:19948-19956`: the off-track brake.
            if self.on_wrong_ground {
                if self.vx > self.run_slowdown {
                    self.vx -= self.run_slowdown;
                } else {
                    self.vx = 0.0;
                }
            }
            // `Player.cs:19926-19958`: the mirror arm — the gate is
            // `velocity.X <= 0f` (braking a leftward roll or at rest).
            if self.cart_mounted
                && !self.on_wrong_ground
                && self.on_track
                && self.vy == 0.0
                && self.vx <= 0.0
            {
                let pos = Vec2::new(self.x, self.y);
                let vel = Vec2::new(self.vx, self.vy);
                let body_h = self.body_height() as i32;
                let max_run = self.max_run_speed;
                let rotation = self.full_rotation;
                minecart::rolling_wheel_sparks(
                    &mut self.pending_cart_sounds,
                    &mut self.pending_sparks,
                    &mut self.spark_rng,
                    pos,
                    WIDTH as i32,
                    body_h,
                    vel,
                    max_run,
                    rotation,
                );
            }
        } else if self.control_left && self.vx > -self.acc_run_speed && self.dash_delay >= 0 {
            // `:19960-19972`: the over-max refine — the inner gate carries the
            // airborne-wings term (`wingsLogic > 0`; carts zero it above).
            if self.vy == 0.0 || self.wings_logic > 0 {
                if self.vx > self.run_slowdown {
                    self.vx -= self.run_slowdown;
                }
                self.vx -= self.run_acceleration * 0.2;
                if self.wings_logic > 0 {
                    self.vx -= self.run_acceleration * 0.2;
                }
            }
            // `:19973-19980`: the off-track brake — unreachable for carts
            // (maxRunSpeed == accRunSpeed, `:26652-26653`), kept verbatim.
            if self.on_wrong_ground {
                if self.vx < self.run_slowdown {
                    self.vx += self.run_slowdown;
                } else {
                    self.vx = 0.0;
                }
            }
        } else if self.control_right && self.vx < self.acc_run_speed && self.dash_delay >= 0 {
            // `:19992-20005`: the mirror arm.
            if self.vy == 0.0 || self.wings_logic > 0 {
                if self.vx < -self.run_slowdown {
                    self.vx += self.run_slowdown;
                }
                self.vx += self.run_acceleration * 0.2;
                if self.wings_logic > 0 {
                    self.vx += self.run_acceleration * 0.2;
                }
            }
            // `:20006-20013`: the off-track brake (see above).
            if self.on_wrong_ground {
                if self.vx > self.run_slowdown {
                    self.vx -= self.run_slowdown;
                } else {
                    self.vx = 0.0;
                }
            }
        } else if self.cart_mounted && self.vx.abs() >= 1.0 {
            // `:20023-20058`: the cart coast — friction only off-track (rails
            // have no rolling resistance; the on-track 0.04 applies below
            // 1 px/tick through the `velocity.Y == 0` arm), then the clamp.
            if self.on_wrong_ground {
                if self.vx > 0.0 {
                    if self.vx > self.run_slowdown {
                        self.vx -= self.run_slowdown;
                    } else {
                        self.vx = 0.0;
                    }
                } else if self.vx < 0.0 {
                    if self.vx < -self.run_slowdown {
                        self.vx += self.run_slowdown;
                    } else {
                        self.vx = 0.0;
                    }
                }
            }
            if self.vx > self.max_run_speed {
                self.vx = self.max_run_speed;
            }
            if self.vx < -self.max_run_speed {
                self.vx = -self.max_run_speed;
            }
        } else if self.vy == 0.0 {
            // `:20059-20072`: the grounded friction — `runSlowdown` is 0.04
            // on-track and 0.2 off-track via the on-track check.
            if self.vx > self.run_slowdown {
                self.vx -= self.run_slowdown;
            } else if self.vx < -self.run_slowdown {
                self.vx += self.run_slowdown;
            } else {
                self.vx = 0.0;
            }
        } else {
            // `:20074-20087`: the airborne friction, halved.
            // (`PortalPhysicsEnabled` has no port system.)
            if self.vx > self.run_slowdown * 0.5 {
                self.vx -= self.run_slowdown * 0.5;
            } else if self.vx < -self.run_slowdown * 0.5 {
                self.vx += self.run_slowdown * 0.5;
            } else {
                self.vx = 0.0;
            }
        }
    }

    /// `Player.FloorVisuals` (`Player.cs:23708-23743`) + `GetFloorTile`
    /// (`:23755-23783`): the tile row at the feet, center column first, then
    /// the neighbors, solid tiles only. The dust visuals (MakeFloorDust) and
    /// the slope dust carve-out stay with the particle tasks.
    fn floor_visuals(&mut self, world: &World) {
        let x = ((self.x + WIDTH * 0.5) / TILE_PX) as i32;
        // `Player.GetFloorTile` (`Player.cs:23755-23783`) reads `base.height` —
        // the live hitbox height (the mount framework's equip; the cart's 52).
        let y = ((self.y + self.body_height()) / TILE_PX) as i32;
        let mut kind = None;
        for tx in [x, x - 1, x + 1] {
            if let Some(t) = world.tile(tx, y) {
                if t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind) {
                    kind = Some(t.kind);
                    break;
                }
            }
        }
        let Some(kind) = kind else {
            self.sticky = false;
            self.slippy = false;
            self.slippy2 = false;
            self.powerrun = false;
            self.running_on_sand = false;
            return;
        };
        // `Player.cs:23727-23731`: cobweb 229, the IceSkateSlippery set
        // (`TileID.cs:481`), FrozenSlimeBlock 197, Asphalt 198, and the sand
        // conversions (`TileID.cs:92-94`).
        self.sticky = kind == 229;
        self.slippy = tr_core::tiledata::SET_ICE_SKATE_SLIPPERY.has(kind);
        self.slippy2 = kind == 197;
        self.powerrun = kind == 198;
        self.running_on_sand = tr_core::tiledata::SET_CONVERSION_SAND.has(kind)
            || tr_core::tiledata::SET_CONVERSION_SANDSTONE.has(kind)
            || tr_core::tiledata::SET_CONVERSION_HARDENED_SAND.has(kind);
    }

    /// `Player.luck` (`Player.cs:3086`) — the value the roll consumers read
    /// (`Luck.RollLuck(this.luck, range)`, `Player.cs:18188`; `DamageVar`
    /// `:44342`; `NPCLoot_DropMoney` `NPC.cs:79622`). Never clamped — the
    /// −0.7/1.0 caps are UI-only (`NormalizedLuck`, `:4107-4122`).
    pub fn luck(&self) -> f32 {
        self.luck_state.luck
    }

    /// `Player.UpdateLuck` → `UpdateLuckFactors` (`Player.cs:29041-29135`),
    /// called from the post-buff-effects position (`:25645`): the ladybug and
    /// coin decays run for every player, the torch scan + the broken-mirror
    /// decay for `Main.myPlayer` only (the port's single player). The torch
    /// slot fills with the 1-row-per-tick scan in the M2-BUFF-6 phase 2; the
    /// `luckNeedsSync` tail is net-only.
    fn update_luck(&mut self, world: &World) {
        let day_rate = world.day_rate();
        self.luck_state.update_lady_bug_decay(day_rate);
        self.luck_state.update_coin_decay(day_rate);
        // `TryRecalculatingTorchLuck` + the diff check (`Player.cs:29126-29130`
        // — the diff half is net-only): `Main.myPlayer`-only in the C#, which
        // is the port's single player.
        let center = self.center();
        crate::luck::torch_scan_step(
            &mut self.luck_state,
            world,
            center,
            self.dead,
            self.held_torch_style,
            self.zones,
        );
        self.luck_state.update_broken_mirror_decay(day_rate);
        self.luck_state.recalculate_luck();
    }

    /// `Player.AddCoinLuck` (`Player.cs:18090-18101`): the shimmer-coin feed
    /// (`WorldItem.cs:1692-1716` is its only C# caller; the coin-shimmer chain
    /// is unported — the callable is unit-tested per the plan). The
    /// `luckNeedsSync` tail is net-only.
    pub fn add_coin_luck(&mut self, coin_position: (f32, f32), coin_amount: i32) {
        let (cx, cy) = self.center();
        let dist = ((coin_position.0 - cx).powi(2) + (coin_position.1 - cy).powi(2)).sqrt();
        if !self.dead && dist < 1000.0 {
            self.luck_state.coin_luck += coin_amount as f32;
            if self.luck_state.coin_luck > 1_000_000.0 {
                self.luck_state.coin_luck = 1_000_000.0;
            }
        }
    }

    /// `NPC.LadyBugKilled`'s player half (`NPC.cs:81566-81598`) — see
    /// `luck::on_lady_bug_killed`; returns the `ladyBugRainBoost` add for the
    /// future Rain writer (the critter row, M4).
    pub fn on_lady_bug_killed(&mut self, position: (f32, f32), gold: bool) -> i32 {
        let (cx, cy) = self.center();
        crate::luck::on_lady_bug_killed(&mut self.luck_state, position, (cx, cy), self.dead, gold)
    }

    /// `NPC.LadyBugLuck`'s player half (`NPC.cs:81600-81626`) — see
    /// `luck::on_lady_bug_nearby`; the `releaseOwner == 255` wild-only gate
    /// stays with the NPC (M4 critter row).
    pub fn on_lady_bug_nearby(&mut self, position: (f32, f32), gold: bool) {
        let (cx, cy) = self.center();
        crate::luck::on_lady_bug_nearby(&mut self.luck_state, position, (cx, cy), gold);
    }

    /// `NPC.NPCLuck` (`NPC.cs:81628-81652`) — see `luck::on_town_npc_death`.
    pub fn on_town_npc_death(&mut self, npc_type: i32) {
        crate::luck::on_town_npc_death(&mut self.luck_state, npc_type, self.dead);
    }


    /// `Player.MinionNPCTargetAim` (`Player.cs:56913-56931`): the summon
    /// staff's right-click picks the nearest chaseable NPC to the mouse world
    /// point; a second click on the same target clears it.
    pub fn minion_npc_target_aim(&mut self, npcs: &[Npc], mouse_world: Vec2) {
        let mut best: i32 = -1;
        let mut best_d = f32::MAX;
        for (i, n) in npcs.iter().enumerate() {
            if !n.can_be_chased_by() {
                continue;
            }
            let (w, h) = n.size();
            let d = (Vec2::new(n.x, n.y) + Vec2::new(w, h) * 0.5 - mouse_world).length();
            if best == -1 || d < best_d {
                best = i as i32;
                best_d = d;
            }
        }
        if self.minion_attack_target_npc == best {
            self.minion_attack_target_npc = -1;
        } else {
            self.minion_attack_target_npc = best;
        }
    }

    /// `Player.MinionRestTargetAim`'s invalidation half (`Player.cs:57009-57011`):
    /// a target that is no longer chaseable or drifted past 3000 px drops.
    pub fn invalidate_minion_target(&mut self, npcs: &[Npc]) {
        if self.minion_attack_target_npc == -1 {
            return;
        }
        let ok = npcs
            .get(self.minion_attack_target_npc as usize)
            .is_some_and(|n| {
                if !n.can_be_chased_by() {
                    return false;
                }
                let (w, h) = n.size();
                let aim = Vec2::new(n.x, n.y) + Vec2::new(w, h) * 0.5;
                (aim - Vec2::new(self.x, self.y)).length() <= 3000.0
            });
        if !ok {
            self.minion_attack_target_npc = -1;
        }
    }
    fn update_survival(&mut self, rng: &mut dyn tr_core::loot::DropRng) {
        // The Wet-buff effects and the DoT derivation are the flag sources the
        // C#'s `UpdateBuffs` chain leaves behind for `UpdateLifeRegen`
        // (`Player.cs:11740-11751, 18949-19103`).
        self.refresh_wet_state();
        let dots = self.derive_dot_inputs(self.buffs.expert_mode);

        if self.in_water {
            self.breath_cd += 1;
            if self.breath_cd >= BREATH_CD {
                self.breath_cd = 0;
                self.breath -= 1;
                if self.breath <= 0 {
                    self.breath = 0;
                    self.lose_health(2);
                }
            }
        } else {
            self.breath = (self.breath + 3).min(BREATH_MAX);
            self.breath_cd = 0;
        }

        // `Player.Update` → `UpdateLifeRegen` / `UpdateManaRegen`
        // (`Player.cs:25977,25995`). The aura/item/grapple inputs stay at their
        // defaults until those systems land (`regen.rs`); the hp gate stands in
        // for the original's dead-player check (M2-DEATH).
        if self.hp > 0 {
            let in_ = RegenInputs {
                max_life: self.max_hp,
                max_mana: self.mana_max,
                mana_regen_bonus: self.mana_regen_bonus,
                well_fed: self.well_fed,
                campfire: self.auras.campfire,
                heart_lantern: self.auras.heart_lantern,
                moving_x: self.vx != 0.0,
                standing_still: self.vx.abs() < 0.05 && self.vy.abs() < 0.05,
                dot_regen: dots.regen,
                dot_resets_time: dots.resets_time,
                dot_fast_lane: dots.fast_lane,
                dot_death_fast: dots.death_fast,
                dot_death_generic: dots.death_generic,
                sitting_or_sleeping: self.sitting || self.sleeping,
                extra_regen: self.accessory_life_regen,
                ..RegenInputs::default()
            };
            if let Some(death) = update_life_regen(&mut self.regen, &in_, &mut self.hp) {
                self.dot_death = Some(death);
            }
            update_mana_regen(&mut self.regen, &in_, &mut self.mana);
        }

        // The lava hit and the water extinguish both trail `UpdateLifeRegen` in
        // the original frame (`Player.cs:27630-27761` > `:25977`): a DoT kill
        // this tick wins over the lava hit.
        // `Player.cs:27681-27722`: the lava timer. Standing in lava drains one
        // tick of the immunity budget per frame; only when it is empty does the
        // 80-damage hit land (and grant On Fire for 420). Out of the lava the
        // budget refills up to `lavaMax` — the Lava Charm family's 420 ticks.
        if self.in_lava {
            if self.lava_time > 0 {
                self.lava_time -= 1;
            } else if self.hurt_cd <= 0.0 {
                let cx = self.x + WIDTH * 0.5;
                // The lava hit rolls the dodge block like any `Hurt` — the
                // stream rides in from the `UpdatePlayers` stage through the
                // update chain, `dodgeable` keeps `Hurt`'s default `true`
                // (research S1-C-1 §1.4).
                if self.hurt(80, cx, rng, true) {
                    // The Hurt return `num81 > 0.0` gates the OnFire grant
                    // (`Player.cs:27710-27711`; remix/ashWood/lavaRose variants
                    // out of scope). `AddBuff` applies the expert ×2 itself (24
                    // is in the difficulty set, `BuffID.cs:50`).
                    self.buffs.add_buff(24, 420);
                }
            }
        } else if self.lava_time < self.lava_max {
            self.lava_time += 1;
        }
        if self.lava_time > self.lava_max {
            self.lava_time = self.lava_max;
        }
        // Water extinguishes only OnFire (24) and OnFire3 (323) — Frostburn,
        // CursedInferno and Burning are immune to it — and only while not
        // standing in lava (`Player.cs:27750-27761`). Deviation (Assumption-2
        // fallback): the C# `WetCollision` flag covers honey too; the port
        // conditions on the water-only `wet` flag, distinct from honey
        // (docs/ANALYSIS.md, Phase 4). The C# loop does not rescan after
        // `DelBuff` compaction.
        if self.wet && !self.lava_wet {
            for i in 0..MAX_BUFFS {
                let t = self.buffs.buff_type[i];
                if t == 24 || t == 323 {
                    self.buffs.del_buff(i);
                }
            }
        }
    }

    /// The original's movement pipeline (`Player.cs`): `TileCollision` resolves the
    /// AABB against solid tiles and slope triangles, `StepDown`/`StepUp` lift the
    /// hitbox over one-tile ledges while `gfxOffY` hides the pop, and
    /// `SlopeCollision` slides the feet along ramps.
    fn move_vanilla(&mut self, world: &World, down: bool, gravity: f32) {
        // `Player.cs`: the visual offset eases back to zero at walk speed.
        let decay = (1.0 + self.vx.abs() / 3.0) * self.step_speed;
        if self.gfx_off_y > 0.0 {
            self.gfx_off_y = (self.gfx_off_y - decay).max(0.0);
        } else if self.gfx_off_y < 0.0 {
            self.gfx_off_y = (self.gfx_off_y + decay).min(0.0);
        }
        self.gfx_off_y = self.gfx_off_y.clamp(-32.0, 32.0);

        let mut pos = [self.x, self.y];
        let mut vel = [self.vx, self.vy];
        // `Player.cs` runs the step helpers on the pre-collision velocity (which
        // still carries this tick's gravity), then resolves the AABB. The wet
        // path never steps — `WetCollision` goes straight to `TileCollision`
        // (`Player.cs:23443`), the step helpers are `DryCollision` internals.
        // Both helpers are skipped while riding tracks (`IsRidingTracks`,
        // `Player.cs:28057-28068` — a mounted cart always is) and inside
        // `DryCollision` for any cart (`:23529-23541`).
        let body_h = self.body_height();
        let dry = self.liquid_step == 1.0;
        if dry && vel[1] == gravity && !self.cart_mounted {
            collision::step_down(
                world,
                &mut pos,
                &mut vel,
                WIDTH,
                body_h,
                &mut self.step_speed,
                &mut self.gfx_off_y,
            );
        }
        if dry && vel[1] >= gravity && !down && !self.cart_mounted {
            collision::step_up(
                world,
                &mut pos,
                &mut vel,
                WIDTH,
                body_h,
                &mut self.step_speed,
                &mut self.gfx_off_y,
                false,
            );
        }
        // `Player.TileCollision` (`Player.cs:23596-23603`): the collision body
        // rides `height − 10` while on tracks — the cart sprite owns the bottom
        // 10 px (pitfall §10.9).
        let col_h = if self.on_track { body_h - 10.0 } else { body_h };
        // `Player.cs:28088-28092`: the cart always falls through platforms
        // (`fallThrough` and `flag31` both true).
        let (fall_through, fall2) = if self.cart_mounted {
            (true, true)
        } else {
            (false, false)
        };
        let tc = collision::tile_collision(world, pos, vel, WIDTH, col_h, fall_through, fall2);
        if tc.up {
            self.jump = 0;
        }
        // `Player.WetCollision` (`Player.cs:23436-23451`): the collision sees
        // the unscaled velocity; the free axes then integrate at the liquid
        // fraction (`liquid_step` — water/lava 0.5, honey 0.25, shimmer 0.375)
        // while an axis the collision overrode keeps its full velocity, so
        // momentum survives the water.
        let pre = vel;
        vel = tc.vel;
        let ms = self.liquid_step;
        pos[0] += if vel[0] != pre[0] {
            vel[0]
        } else {
            vel[0] * ms
        };
        pos[1] += if vel[1] != pre[1] {
            vel[1]
        } else {
            vel[1] * ms
        };
        if down {
            self.stair_fall = true;
        }
        // `Player.SlopingCollision(fallThrough, flag31)` (`Player.cs:28210`):
        // the cart's platform pass-through rides here too.
        let sc = collision::slope_collision(
            world,
            pos,
            vel,
            WIDTH,
            body_h,
            self.cart_mounted || self.stair_fall,
        );
        self.stair_fall = sc.stair_fall;
        if sc.stair && (sc.pos[1] - pos[1]).abs() > 8.0 + vel[0].abs() {
            self.gfx_off_y -= sc.pos[1] - pos[1];
            self.step_speed = 4.0;
        }
        pos = sc.pos;
        vel = sc.vel;
        self.x = pos[0];
        self.y = pos[1];
        self.vx = vel[0];
        self.vy = vel[1];
        // The original gates jumps on `velocity.Y == 0f` after collision.
        self.on_ground = self.vy == 0.0;
    }

    /// Position between the previous and current fixed steps, for smooth rendering.
    /// The visual step offset is interpolated too and baked into `y`.
    pub fn interpolated(&self, alpha: f32) -> Self {
        let a = alpha.clamp(0.0, 1.0);
        let mut out = self.clone();
        out.x = self.prev_x + (self.x - self.prev_x) * a;
        out.y = self.prev_y + (self.y - self.prev_y) * a;
        out.y += self.prev_gfx_off_y + (self.gfx_off_y - self.prev_gfx_off_y) * a;
        out.prev_x = out.x;
        out.prev_y = out.y;
        out.gfx_off_y = 0.0;
        out.prev_gfx_off_y = 0.0;
        out
    }
    pub fn ammo_flags(&self) -> crate::ammo::AmmoFlags {
        crate::ammo::AmmoFlags {
            has_molten_quiver: self.has_molten_quiver,
            magic_quiver: self.magic_quiver,
            acc_sharp_barb: self.acc_sharp_barb,
            archery: self.archery,
            ammo_box: self.ammo_box,
            ammo_potion: self.ammo_potion,
            huntress_ammo_cost90: self.huntress_ammo_cost90,
            chloro_ammo_cost80: self.chloro_ammo_cost80,
            ammo_cost80: self.ammo_cost80,
            ammo_cost75: self.ammo_cost75,
        }
    }


    /// Deterministic identity for the G6 world-range cache.
    pub fn render_key(&self) -> u32 {
        let mut h = self.x.to_bits().wrapping_mul(0x9E37_79B1);
        h ^= self.y.to_bits().wrapping_mul(0x85EB_CA77);
        h ^= (self.facing as u32) << 1;
        h ^= self.on_ground as u32;
        h ^= self.leg_frame << 8;
        h ^= self.body_frame << 16;
        // The draw bakes `gfxOffY` into the sprite's screen position
        // (`LegacyPlayerRenderer.cs:457,468`), so its ease after a step must
        // invalidate the cached range or the offset animation freezes.
        h ^= self.gfx_off_y.to_bits().wrapping_mul(0x27D4_EB2F);
        // The mount's animator state feeds the mount sprites' pixels
        // (`Mount.Draw`'s frame rect + the per-frame player offset).
        if self.mount.is_active() {
            h ^= (self.mount.kind() as u32) << 3;
            h ^= (self.mount.frame() as u32).wrapping_mul(0x9E37_79B1);
            h ^= (self.mount.frame_state() as u32) << 20;
            h ^= (self.mount.player_offset() as u32) << 26;
        }
        h
    }
}

/// Player movement collision: ground slopes (slope 1/2) are handled by `slope_floor`.
pub fn collides_player_rect(world: &World, x: f32, y: f32) -> bool {
    collides_filtered(world, x, y, WIDTH, HEIGHT, true)
}

pub fn collides_player(world: &World, x: f32, y: f32) -> bool {
    collides_player_rect(world, x, y)
}

/// Generic solid-tile AABB test used by NPCs and the player.
pub fn collides_rect(world: &World, x: f32, y: f32, w: f32, h: f32) -> bool {
    collides_filtered(world, x, y, w, h, false)
}

/// `Collision.WetCollision` (`Collision.cs:1572-1651`): true when a liquid
/// tile's surface overlaps the inset probe rectangle. The C# `honey`/`shimmer`
/// out-params have no consumer here and are not modelled. The bat mount's
/// activation gate calls this (`Mount.cs:6595`).
pub fn wet_collision(world: &World, x: f32, y: f32, w: f32, h: f32) -> bool {
    let wi = w as i32;
    let hi = h as i32;
    let inset_w = 10.min(wi);
    let inset_h = (hi / 2).min(hi);
    let px = x + (wi / 2) as f32 - (inset_w / 2) as f32;
    let py = y + (hi / 2) as f32 - (inset_h / 2) as f32;
    let max_y = (world.height - 40).max(0);
    let x0 = (((x / TILE_PX) as i32) - 1).clamp(0, world.width - 1);
    let x1 = ((((x + wi as f32) / TILE_PX) as i32) + 2).clamp(0, world.width - 1);
    let y0 = (((y / TILE_PX) as i32) - 1).clamp(0, max_y);
    let y1 = ((((y + hi as f32) / TILE_PX) as i32) + 2).clamp(0, max_y);
    for tx in x0..x1 {
        for ty in y0..y1 {
            let Some(t) = world.tile(tx, ty) else {
                continue;
            };
            let left = tx as f32 * TILE_PX;
            if t.liquid > 0 {
                // `num5 = (256 - liquid) / 32`; the surface rises by
                // `num5 * 2` and the solid part shrinks by the same amount.
                let num5 = (256 - t.liquid as i32) as f32 / 32.0;
                let top = ty as f32 * TILE_PX + num5 * 2.0;
                let depth = 16.0 - (num5 * 2.0) as i32 as f32;
                if px + inset_w as f32 > left
                    && px < left + 16.0
                    && py + inset_h as f32 > top
                    && py < top + depth
                {
                    return true;
                }
            } else if t.active()
                && t.slope() != 0
                && ty > 0
                && world.tile(tx, ty - 1).is_some_and(|above| above.liquid > 0)
            {
                // The sloped-tile branch (`:1625-1645`): the liquid above a
                // slope still counts even though this cell is dry.
                if px + inset_w as f32 > left
                    && px < left + 16.0
                    && py + inset_h as f32 > ty as f32 * TILE_PX
                    && py < ty as f32 * TILE_PX + 16.0
                {
                    return true;
                }
            }
        }
    }
    false
}

fn collides_filtered(world: &World, x: f32, y: f32, w: f32, h: f32, ignore_slopes: bool) -> bool {
    let x0 = (x / TILE_PX).floor() as i32;
    let x1 = ((x + w - 0.01) / TILE_PX).floor() as i32;
    let y0 = (y / TILE_PX).floor() as i32;
    let y1 = ((y + h - 0.01) / TILE_PX).floor() as i32;
    for ty in y0..=y1 {
        for tx in x0..=x1 {
            let Some(t) = world.tile(tx, ty) else {
                continue;
            };
            if !t.active() {
                continue;
            }
            if (t.kind as usize) >= td::TILE_COUNT || !td::TILE_SOLID.has(t.kind) {
                continue;
            }
            if ignore_slopes {
                if t.slope() == 1 || t.slope() == 2 {
                    continue; // ground ramps are handled by `slope_floor`
                }
                if t.half_brick() {
                    // Only the bottom 8 px are solid.
                    if y + h > ty as f32 * TILE_PX + 8.0 {
                        return true;
                    }
                    continue;
                }
                if t.slope() >= 3 {
                    // Ceiling ramp: solid above the diagonal, empty below it.
                    let base_x = tx as f32 * TILE_PX;
                    let base_y = ty as f32 * TILE_PX;
                    let sx0 = x.max(base_x);
                    let sx1 = (x + w).min(base_x + TILE_PX);
                    let mut cx = sx0;
                    while cx < sx1 {
                        let local = cx - base_x;
                        let boundary = if t.slope() == 3 {
                            base_y + TILE_PX - local
                        } else {
                            base_y + local
                        };
                        if y < boundary && y + h > base_y {
                            return true;
                        }
                        cx += 2.0;
                    }
                    continue;
                }
            }
            return true;
        }
    }
    false
}

#[cfg(test)]

mod tests {
    #[test]
    fn dye_cache_mirrors_update_dyes() {
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        // Empty slots → the C# zero-fill (`UpdateDyes`, `:9589-9592`).
        p.update_dye_cache();
        assert_eq!(p.c_head, 0);
        // `cHead/cBody/cLegs = GetEffectiveDye(0..2).dye` (`:9593-9595`): red
        // dye (1007) on the head slot, blue (1015) on the legs, nothing on
        // the body.
        p.dye_slots[0] = 1007;
        p.dye_slots[2] = 1015;
        p.misc_dye_slots[3] = 1007;
        p.update_dye_cache();
        assert!(p.c_head > 0);
        assert_eq!(p.c_body, 0);
        assert!(p.c_legs > 0 && p.c_legs != p.c_head);
        // The same dye lands on the same slot everywhere (`cMount` mirrors
        // `miscDyes[3].dye`, `:9600-9604`).
        assert_eq!(p.c_mount, p.c_head);
        // The misc five keep `:9600-9604`'s pet/light/cart/mount/hook order.
        p.misc_dye_slots[0] = 1015;
        p.update_dye_cache();
        assert!(p.c_pet > 0);
        assert_eq!(p.c_light, 0);
    }

    /// S1-C-4 — `GetWeaponDamageMultiplier`'s ranged dispatch
    /// (`Player.cs:53396-53398`) keys on the ammo category, and the
    /// effective-damage components (`:3982-3986`) move it off the plain
    /// `rangedDamage` once the C#'s arming sites fire: the Archery buff
    /// (`:10019`), the quivers (`:14466`/`:14484`/`:14598`/`:15334`) and the
    /// Shroomite helmets (`:13697-13712`).
    #[test]
    fn s1_c_4_the_ranged_multiplier_dispatches_by_ammo_category() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        // A plain gun (bullet ammo 97): the plain rangedDamage — every
        // component at its reset value.
        assert_eq!(
            p.weapon_damage_multiplier_with_ammo(registry::DamageClass::Ranged, 100, 97),
            1.0
        );
        // An arrow weapon (ammo 40): bowEffectiveDamage
        // `(ranged / mult + stack) * mult * arrowDamage` = 1.0 at the resets.
        assert_eq!(
            p.weapon_damage_multiplier_with_ammo(registry::DamageClass::Ranged, 100, 40),
            1.0
        );
        // The arming: the Archery buff (×1.1) and a Magic Quiver (+0.1 stack)
        // move the bow arm off 1.0 while the gun arm stays on rangedDamage.
        p.arrow_damage *= 1.1; // the buff arm (`:10019`)
        p.arrow_damage_additive_stack += 0.1; // the quiver arm (`:14466`)
        let bow = p.weapon_damage_multiplier_with_ammo(registry::DamageClass::Ranged, 100, 40);
        assert!((bow - (1.0 / 1.0 + 0.1) * 1.0 * 1.1).abs() < 1e-6, "{bow}");
        assert_eq!(
            p.weapon_damage_multiplier_with_ammo(registry::DamageClass::Ranged, 100, 97),
            1.0,
            "the gun arm does not read the arrow components"
        );
        // The Shroomite Bullet helmet (`:13704`): the gun arm reads
        // bulletDamage; the bow arm does not.
        p.bullet_damage *= 1.12;
        assert_eq!(
            p.weapon_damage_multiplier_with_ammo(registry::DamageClass::Ranged, 100, 97),
            1.12
        );
        let bow2 = p.weapon_damage_multiplier_with_ammo(registry::DamageClass::Ranged, 100, 40);
        assert!((bow2 - bow).abs() < 1e-6, "the helmet does not touch the bow");
        // The specialist arm: rocket ammo 771 and the
        // `IsRangedSpecialistWeapon` ids ride rocketDamage (`:3986`) — but
        // only after the arrow/bullet checks (the C#'s `if` order), so a
        // specialist weapon on arrow ammo stays on the bow arm.
        p.rocket_damage *= 1.12;
        assert_eq!(
            p.weapon_damage_multiplier_with_ammo(registry::DamageClass::Ranged, 100, 771),
            1.12
        );
        assert_eq!(
            p.weapon_damage_multiplier_with_ammo(registry::DamageClass::Ranged, 1156, 931),
            1.12,
            "the specialist weapon on flare ammo rides rocketDamage"
        );
        let bow3 = p.weapon_damage_multiplier_with_ammo(registry::DamageClass::Ranged, 1156, 40);
        assert!((bow3 - bow).abs() < 1e-6, "arrow ammo wins over the id");
        // The non-ranged classes keep their table (`:53390`/`:53410-53419`).
        assert_eq!(p.weapon_damage_multiplier_with_ammo(registry::DamageClass::Melee, 100, 40), 1.0);
    }

    /// M3-EQUIP-5 — `Player.statDefense` is derived inside the tick, in the
    /// vanilla order: `ResetEffects` zeroes it (`Player.cs:18327`), the buff
    /// arms add, `GrantArmorBenefits` adds the piece rows (`:13121`) and the
    /// set bonus, `GrantPrefixBenefits` the accessory prefixes, then the floor
    /// (`:25927`).
    #[test]
    fn stat_defense_sums_pieces_set_bonus_and_buffs() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        // Nothing worn, nothing buffed: 0.
        p.update(&w, PlayerInput::default());
        assert_eq!(p.stat_defense, 0);
        // The copper set (89/80/76): 1 + 2 + 1 pieces + the MetalTier1 bonus 2.
        p.equipped[0] = 89;
        p.equipped[1] = 80;
        p.equipped[2] = 76;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.stat_defense, 6, "copper set: 4 pieces + 2");
        // The accessories count too (`GetEffectiveArmor(0..10)`): the Shackle
        // (216) +1 and the Cobalt Shield (156) +1 sit in slots 3/4. (The Ankh
        // Shield would be +5 but it also grants immunity to Weak — see the
        // immunity test — which would change the debuff rows below.)
        p.equipped[3] = 216;
        p.equipped[4] = 156;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.stat_defense, 8, "8 = 4 + 2 + 1 + 1");
        // Ironskin (buff 5) +8 and the fed tier (26) +2.
        p.buffs.add_buff(5, 3600);
        p.buffs.add_buff(26, 3600);
        p.update(&w, PlayerInput::default());
        assert_eq!(p.stat_defense, 18, "8 + 8 + 2");
        // Ichor (69) −15 and Weak (33) −4: the sum can go below the armor, and
        // the floor clamps it at 0 (`:25927`) instead of inflating damage.
        p.buffs.add_buff(69, 3600);
        p.buffs.add_buff(33, 3600);
        p.update(&w, PlayerInput::default());
        assert_eq!(p.stat_defense, 0, "18 - 15 - 4 = -1, floored");
        p.equipped[0] = 0;
        p.equipped[1] = 0;
        p.equipped[2] = 0;
        p.equipped[3] = 0;
        p.equipped[4] = 0;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.stat_defense, 0, "the floor holds at 0");
    }

    /// M3-EQUIP-20 — the armor-penetration rows and the melee-only split.
    #[test]
    fn the_penetration_rows_add_up() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        p.update(&w, PlayerInput::default());
        assert_eq!(p.get_armor_penetration(true), 0);
        // The Shark Tooth Necklace (3212) and the Honey Comb (4007): +5 each,
        // both class-independent.
        p.equipped[3] = 3212;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.get_armor_penetration(true), 5);
        assert_eq!(p.get_armor_penetration(false), 5);
        p.equipped[4] = 4007;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.get_armor_penetration(false), 10);
        // The Sharpened buff (159) adds 12 that only melee hits see.
        p.buffs.add_buff(159, 3600);
        p.update(&w, PlayerInput::default());
        assert_eq!(p.get_armor_penetration(true), 22);
        assert_eq!(p.get_armor_penetration(false), 10);
    }

    /// M3-EQUIP-17 — the life-regen rows feed `extra_regen`.
    #[test]
    fn the_life_regen_rows_add_up() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        p.update(&w, PlayerInput::default());
        assert_eq!(p.accessory_life_regen_value(), 0);
        // The Band of Regeneration (49): the item's own `lifeRegen` 2.
        p.equipped[3] = 49;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.accessory_life_regen_value(), 2);
        // The Charm of Myths (860) carries the same `Item.lifeRegen` 2, and the
        // row is the item's own field now, not a hardcoded match arm.
        p.equipped[4] = 860;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.accessory_life_regen_value(), 4);
        p.equipped[4] = 0;
        // The Regeneration buff (2) +4 and the Dryad's Ward (165) +6.
        p.buffs.add_buff(2, 3600);
        p.buffs.add_buff(165, 3600);
        p.update(&w, PlayerInput::default());
        assert_eq!(p.accessory_life_regen_value(), 12);
        // The Solar Flare Breastplate (2764) +2 and the Celestial Stone +2.
        p.equipped[1] = 2764;
        p.equipped[4] = 1865;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.accessory_life_regen_value(), 16);
        // The Spooky Breastplate (3872) is the +8 row.
        p.equipped[1] = 3872;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.accessory_life_regen_value(), 22);
    }

    /// M3-EQUIP-16 — the lava immunity budget.
    #[test]
    fn the_lava_charm_buys_immunity_ticks() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        p.update(&w, PlayerInput::default());
        assert_eq!(p.lava_max, 0);
        // The Lava Charm (906) adds 420 ticks; the Terraspark Boots (5000) too.
        p.equipped[3] = 906;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.lava_max, 420);
        p.equipped[4] = 5000;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.lava_max, 840, "the rows stack");
        // Out of the lava the budget refills one tick per frame.
        p.lava_time = 0;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.lava_time, 1);
        // Standing in lava drains it instead of burning: five frames of the
        // budget leave the player untouched (`update_survival` is the sub-step
        // the lava hit lives in).
        p.lava_time = 5;
        p.hp = 100;
        p.hurt_cd = 0.0;
        for _ in 0..5 {
            p.in_lava = true;
            p.update_survival(&mut tr_core::UnifiedRandom::default());
        }
        assert_eq!(p.hp, 100, "the budget absorbed the lava");
        assert_eq!(p.lava_time, 0);
        // The next frame burns for the vanilla 80.
        p.in_lava = true;
        p.hurt_cd = 0.0;
        p.update_survival(&mut tr_core::UnifiedRandom::default());
        assert_eq!(p.hp, 20, "the 80-damage lava hit");
        assert!(p.buffs.find_buff_index(24).is_some(), "On Fire");
    }

    /// M3-EQUIP-15 — the thorns rows and the reflection arithmetic.
    #[test]
    fn the_thorns_rows_scale_the_reflection() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        p.update(&w, PlayerInput::default());
        assert_eq!(p.thorns_damage(20), 0);
        // The Thorns potion (14): the NPC's own damage comes back whole.
        p.buffs.add_buff(14, 3600);
        p.update(&w, PlayerInput::default());
        assert!((p.thorns - 1.0).abs() < 1e-4);
        assert_eq!(p.thorns_damage(20), 20);
        // The Dryad's Ward (165) pushes it to 1.5 while under 1… it is already
        // 1.0, so the half does not land — the C# `if (thorns < 1f)` guard.
        p.buffs.add_buff(165, 3600);
        p.update(&w, PlayerInput::default());
        assert!((p.thorns - 1.0).abs() < 1e-4);
        // The Turtle set pins the multiplier at 2 regardless of the potion.
        p.equipped[0] = 1316;
        p.equipped[1] = 1317;
        p.equipped[2] = 1318;
        p.update(&w, PlayerInput::default());
        assert!(p.turtle_thorns);
        assert_eq!(p.thorns_damage(20), 40);
        assert_eq!(p.thorns_damage(2000), 1000, "the 1000 cap");
        // The Cactus set adds its flat 15 on top.
        p.equipped[0] = 894;
        p.equipped[1] = 895;
        p.equipped[2] = 896;
        p.update(&w, PlayerInput::default());
        assert!(p.cactus_thorns);
        assert!(!p.turtle_thorns, "the turtle row is gone");
        assert_eq!(p.thorns_damage(20), 35, "20 from the potion + the flat 15");
    }

    /// M3-EQUIP-14 — the Celestial Stone bundle and the day/night stones.
    #[test]
    fn the_sky_stones_add_their_bundle() {
        let mut w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        // The Celestial Stone (1865): every row of the C# block.
        p.equipped[3] = 1865;
        p.update(&w, PlayerInput::default());
        assert!(p.sky_stone_effects);
        assert_eq!(p.stat_defense, 4);
        assert!((p.melee_damage - 1.1).abs() < 1e-4);
        assert!((p.minion_damage - 1.1).abs() < 1e-4);
        assert_eq!((p.melee_crit, p.ranged_crit, p.magic_crit), (6, 6, 6));
        assert!((p.pick_speed_bonus - 0.15).abs() < 1e-4);
        // The Sun Stone (899) only works by day; the Moon Stone (900) by night.
        p.equipped[3] = 899;
        w.day_time = true;
        p.update(&w, PlayerInput::default());
        assert!(p.sky_stone_effects);
        w.day_time = false;
        p.update(&w, PlayerInput::default());
        assert!(!p.sky_stone_effects);
        p.equipped[3] = 900;
        p.update(&w, PlayerInput::default());
        assert!(p.sky_stone_effects);
        w.day_time = true;
        p.update(&w, PlayerInput::default());
        assert!(!p.sky_stone_effects);
        // An eclipse makes the Moon Stone work in daylight.
        w.eclipse = true;
        p.update(&w, PlayerInput::default());
        assert!(p.sky_stone_effects);
    }

    /// M3-EQUIP-13 — the armor pieces carry their own damage/crit/speed rows
    /// (before this the worn armor only contributed defense).
    #[test]
    fn the_armor_pieces_add_their_stat_rows() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        // The Cobalt Mage Hat (371): +10% magic damage.
        p.equipped[0] = 371;
        p.update(&w, PlayerInput::default());
        assert!((p.magic_damage - 1.1).abs() < 1e-4);
        assert!((p.melee_damage - 1.0).abs() < 1e-4);
        // The Fossil Greaves (552) add 7% to every class and +8% move; the
        // Yoyo Bag-style rows stack per piece, so two pieces add twice.
        p.equipped[1] = 552;
        p.update(&w, PlayerInput::default());
        assert!((p.melee_damage - 1.07).abs() < 1e-4);
        assert!((p.minion_damage - 1.07).abs() < 1e-4);
        assert!((p.move_speed - 1.08).abs() < 1e-4);
        // The Solar Flare Breastplate (2764) alone is +29% melee damage.
        p.equipped[0] = 0;
        p.equipped[1] = 2764;
        p.update(&w, PlayerInput::default());
        assert!((p.melee_damage - 1.29).abs() < 1e-4);
    }

    /// M3-EQUIP-12 — the immunity rows of the accessories and the Snow/Molten
    /// sets, re-applied after `ResetEffects`' clear.
    #[test]
    fn the_immunity_rows_block_their_debuffs() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        p.update(&w, PlayerInput::default());
        assert!(!p.buffs.buff_immune[46]);
        // The Snow set (803/804/805) is immune to Chilled (46) and Frozen (47).
        p.equipped[0] = 803;
        p.equipped[1] = 804;
        p.equipped[2] = 805;
        p.update(&w, PlayerInput::default());
        assert!(p.buffs.buff_immune[46]);
        assert!(p.buffs.buff_immune[47]);
        // The Ankh Shield (1613): On Fire is *not* on its list, Chilled is.
        p.equipped[0] = 0;
        p.equipped[1] = 0;
        p.equipped[2] = 0;
        p.equipped[3] = 1613;
        p.update(&w, PlayerInput::default());
        for id in [46, 33, 36, 30, 20, 32, 31, 35, 23, 22, 156] {
            assert!(p.buffs.buff_immune[id], "the ankh row {}", id);
        }
        assert!(!p.buffs.buff_immune[24], "On Fire is not an ankh immunity");
        // The Molten set (231/232/233) is.
        p.equipped[3] = 0;
        p.equipped[0] = 231;
        p.equipped[1] = 232;
        p.equipped[2] = 233;
        p.update(&w, PlayerInput::default());
        assert!(p.buffs.buff_immune[24]);
        // Cheap proof the table gates the add: the debuff never lands.
        p.buffs.add_buff(24, 600);
        assert_eq!(p.buffs.find_buff_index(24), None);
    }

    /// M3-EQUIP-11 — `endurance` folds the rows and the hurt applies it.
    #[test]
    fn the_endurance_rows_cut_the_incoming_damage() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        p.update(&w, PlayerInput::default());
        assert!((p.endurance - 0.0).abs() < 1e-4);
        // The Worm Scarf (3224) is +17%.
        p.equipped[3] = 3224;
        p.update(&w, PlayerInput::default());
        assert!((p.endurance - 0.17).abs() < 1e-4);
        // The Turtle set (1316/1317/1318) is +15%.
        p.equipped[0] = 1316;
        p.equipped[1] = 1317;
        p.equipped[2] = 1318;
        p.equipped[3] = 0;
        p.update(&w, PlayerInput::default());
        assert!((p.endurance - 0.15).abs() < 1e-4);
        // The Endurance buff (114) adds its 10% on top.
        p.buffs.add_buff(114, 3600);
        p.update(&w, PlayerInput::default());
        assert!((p.endurance - 0.25).abs() < 1e-4);
        // 40 damage with 15+10% endurance: 40 * 0.75 = 30 past the defense 0.
        p.stat_defense = 0;
        p.hp = 100;
        p.hurt_cd = 0.0;
        p.hurt_inner(40, p.x - 10.0, false, &mut tr_core::UnifiedRandom::default(), true);
        assert_eq!(p.hp, 70, "the cut, not the raw 40");
        // A 1-damage hit past 25% endurance truncates to 0, then the
        // `num2 < 1 -> 1` floor (`Player.cs:38378-38381`) lifts it back to 1.
        p.hurt_cd = 0.0;
        p.hurt_inner(1, p.x - 10.0, false, &mut tr_core::UnifiedRandom::default(), true);
        assert_eq!(p.hp, 69, "the sub-one hit floors to 1, not 0");
    }

    /// S1-C-3 — the `num2 < 1 -> 1` floor (`Player.cs:38378-38381`): the
    /// endurance multiply can truncate a hit to 0; the C# floors it to 1, which
    /// also shortens the immunity window from 40 to 20 ticks (`:38385`:
    /// `num2 != 1.0 ? 40 : 20`, via `damage.rs::iframes_for`).
    #[test]
    fn a_sub_one_hit_floors_to_one_and_uses_the_short_iframe() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        // 25% endurance: (1 - 0.25) * 1 = 0.75, truncated to 0 before the floor.
        p.endurance = 0.25;
        p.stat_defense = 0;
        p.hp = 100;
        p.hurt_cd = 0.0;
        assert!(p.hurt_inner(1, p.x - 10.0, false, &mut tr_core::UnifiedRandom::default(), true));
        assert_eq!(p.hp, 99, "the hit lands for the floored 1, not 0");
        assert!(
            (p.hurt_cd - 20.0 / 60.0).abs() < 1e-6,
            "the 1-damage immunity window is 20 ticks, not 40"
        );
    }

    /// M3-EQUIP-10 — the class emblems and the crit/move accessories.
    #[test]
    fn the_damage_accessories_raise_their_class() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        // Warrior Emblem (490): melee +15%, nothing else.
        p.equipped[3] = 490;
        p.update(&w, PlayerInput::default());
        assert!((p.melee_damage - 1.15).abs() < 1e-4);
        assert!((p.magic_damage - 1.0).abs() < 1e-4);
        // Avenger Emblem (935): +12% to all four classes.
        p.equipped[3] = 935;
        p.update(&w, PlayerInput::default());
        assert!((p.melee_damage - 1.12).abs() < 1e-4);
        assert!((p.minion_damage - 1.12).abs() < 1e-4);
        // Necromantic Scroll (1301): +8 to three crits and +10% to four
        // classes on top of the 4-point crit bases.
        p.equipped[3] = 1301;
        p.update(&w, PlayerInput::default());
        assert_eq!((p.melee_crit, p.ranged_crit, p.magic_crit), (12, 12, 12));
        assert!((p.melee_damage - 1.1).abs() < 1e-4);
        // Papyrus Scarab (1864): +15% minion damage (its `maxMinions++` half
        // belongs to `grant_summon_caps`).
        p.equipped[3] = 1864;
        p.update(&w, PlayerInput::default());
        assert!((p.minion_damage - 1.15).abs() < 1e-4);
    }

    /// M3-EQUIP-9 — `manaCost` folds the armor pieces, the accessories, the
    /// casters' sets and Magic Power.
    #[test]
    fn the_mana_cost_multiplier_folds_every_row() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        p.update(&w, PlayerInput::default());
        assert!((p.mana_cost - 1.0).abs() < 1e-4);
        // Nature's Gift (223) −0.06 and the Mana Flower (555) −0.08.
        p.equipped[3] = 223;
        p.equipped[4] = 555;
        p.update(&w, PlayerInput::default());
        assert!((p.mana_cost - 0.86).abs() < 1e-4);
        // The Mystic Robe (2279) −0.1 as an armor piece.
        p.equipped[1] = 2279;
        p.update(&w, PlayerInput::default());
        assert!((p.mana_cost - 0.76).abs() < 1e-4);
        // Magic Power (buff 29) −0.02 plus its other rows.
        p.buffs.add_buff(29, 3600);
        let mana_max_before = p.mana_max;
        p.update(&w, PlayerInput::default());
        assert!((p.mana_cost - 0.74).abs() < 1e-4);
        assert!(p.mana_max > mana_max_before, "the 20-mana row");
        // The Mythril Caster set (376/379/380) −0.17.
        p.equipped[0] = 376;
        p.equipped[1] = 379;
        p.equipped[2] = 380;
        p.equipped[3] = 0;
        p.equipped[4] = 0;
        p.update(&w, PlayerInput::default());
        assert!((p.mana_cost - (1.0 - 0.17 - 0.02)).abs() < 1e-4);
    }

    /// M3-EQUIP-8 — the wings grant `wingsLogic`/`wingTimeMax` themselves (the
    /// `TR_WINGS` pin is no longer the only source).
    #[test]
    fn the_wing_accessories_grant_flight() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        p.update(&w, PlayerInput::default());
        assert_eq!(p.wings_logic, 0);
        assert_eq!(p.wing_time_max, 0);
        // Demon Wings (492 → `ArmorIDs.Wing` 1): 100 ticks of flight.
        p.equipped[3] = 492;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.wings_logic, 1);
        assert!(p.has_wings);
        assert_eq!(p.wing_time_max, Player::wing_stats(1).0);
        // Bee Wings (1866 → 22) is the hover row: its stats carry the hover
        // speeds, and the budget is its own.
        p.equipped[3] = 1866;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.wings_logic, 22);
        assert_eq!(p.wing_time_max, Player::wing_stats(22).0);
        assert!(Player::wing_stats(22).3, "the hover flag");
        // Taking the wings off ends both.
        p.equipped[3] = 0;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.wings_logic, 0);
        assert_eq!(p.wing_time_max, 0);
        assert!(!p.has_wings);
    }

    /// M3-EQUIP-7 — the movement accessories grant the live flags (the
    /// `TR_JUMPS`/`TR_BOOTS` pins stop being the only source).
    #[test]
    fn the_movement_accessories_grant_their_flags() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        // Bundle of Balloons (1164): Cloud + Sandstorm + Blizzard + jumpBoost.
        // Before M3-EQUIP-7 only the jump boost was ported, so the item gave
        // one extra jump instead of three.
        p.equipped[3] = 1164;
        p.update(&w, PlayerInput::default());
        assert!(p.has_jump_options[ExtraJump::Cloud.index()]);
        assert!(p.has_jump_options[ExtraJump::Sandstorm.index()]);
        assert!(p.has_jump_options[ExtraJump::Blizzard.index()]);
        assert!(p.jump_boost);
        // Frostspark Boots (1862): tier 3, 6.75 accel, +8% move, ice skates.
        p.equipped[3] = 1862;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.rocket_boots, 3);
        assert!((p.acc_run_speed_value() - 6.75).abs() < 1e-4);
        assert!((p.move_speed - 1.08).abs() < 1e-4);
        assert!(p.ice_skate);
        assert!(!p.has_jump_options[ExtraJump::Cloud.index()]);
        // Hellfire Treads (4874): the tier-5 boots at the plain 6f.
        p.equipped[3] = 4874;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.rocket_boots, 5);
        assert!((p.acc_run_speed_value() - 6.0).abs() < 1e-4);
        // A Sandstorm balloon (1252): the bottle's option and the boost.
        p.equipped[3] = 0;
        p.equipped[4] = 1252;
        p.update(&w, PlayerInput::default());
        assert!(p.has_jump_options[ExtraJump::Sandstorm.index()]);
        assert!(p.jump_boost);
    }

    /// M3-EQUIP-5 — `ArmorSetBonuses.Benefits`' scalar rows ride the same pass.
    #[test]
    fn the_completed_set_grants_its_stat_row() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        // Crystal Assassin (4982/4983/4984): +10% every damage, +10 every crit,
        // `dashType = 5`. The crit bases are 4 (`ResetEffects`), so 14.
        p.equipped[0] = 4982;
        p.equipped[1] = 4983;
        p.equipped[2] = 4984;
        p.update(&w, PlayerInput::default());
        // The damage scalars start at 1.0 in the reset (`:2535-2538`): the
        // set's `+= 0.1f` plus the slate's own +5% (4983, M3-EQUIP-13's piece
        // row) read as 1.15.
        assert!((p.melee_damage - 1.15).abs() < 1e-4);
        assert!((p.minion_damage - 1.15).abs() < 1e-4);
        assert_eq!((p.melee_crit, p.ranged_crit, p.magic_crit), (14, 14, 14));
        assert_eq!(p.dash_type, 5);
        // Swapping to the Ninja set (256/257/258) keeps only its own row:
        // `moveSpeed += 0.2f`.
        p.equipped[0] = 256;
        p.equipped[1] = 257;
        p.equipped[2] = 258;
        p.update(&w, PlayerInput::default());
        assert!((p.melee_damage - 1.0).abs() < 1e-4, "the base 1.0, no add");
        assert_eq!(p.melee_crit, 4);
        assert!((p.move_speed - 1.2).abs() < 1e-4);
        // The Mining set (88/410/411) is the `pickSpeed -= 0.1f` negation.
        p.equipped[0] = 88;
        p.equipped[1] = 410;
        p.equipped[2] = 411;
        p.update(&w, PlayerInput::default());
        assert!((p.pick_speed_bonus - 0.1).abs() < 1e-4);
        // A partial set grants nothing.
        p.equipped[2] = 0;
        p.update(&w, PlayerInput::default());
        assert!((p.pick_speed_bonus - 0.0).abs() < 1e-4);
    }

    #[test]
    fn dye_cache_ignores_non_dye_kinds() {
        // Dirty: a non-dye in a dye slot renders plain — the C# reads
        // `GetShaderIdFromItemId` (`Item.cs:48713`), which is 0 outside the
        // `DyeInitializer` bindings.
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        p.dye_slots[1] = 89; // the Copper Helmet
        p.update_dye_cache();
        assert_eq!(p.c_body, 0);
    }

    #[test]
    fn dot_death_survival_drain_records_the_reason() {
        // DW-1.3 — Player.cs:19361-19384: a drain kill carries its reason out
        // of UpdateLifeRegen; update_survival parks it on the player for the
        // death pipeline (the default generic attribution here = Burned, 8).
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        p.update_survival(&mut tr_core::UnifiedRandom::default());
        assert_eq!(p.dot_death, None); // a healthy tick records nothing
        p.hp = 8;
        p.regen.life_regen_count = -1200;
        p.update_survival(&mut tr_core::UnifiedRandom::default());
        assert_eq!(p.hp, 0);
        assert_eq!(p.dot_death, Some(DotDeath::Burned));
    }

    #[test]
    fn the_defense_reduces_the_hurt_damage() {
        // `Player.Hurt`'s classic reduction: the damage minus
        // `statDefense * 0.5`, floored at 1 (M3-EQUIP-2).
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        p.hurt_cd = 0.0;
        p.stat_defense = 20; // 10 damage reduction
        p.hp = 100;
        p.hurt(20, 0.0, &mut tr_core::UnifiedRandom::default(), true);
        assert_eq!(p.hp, 90, "20 - 20*0.5 = 10 taken");
        // The floor: a huge defense leaves 1.
        p.hurt_cd = 0.0;
        p.hp = 100;
        p.stat_defense = 400;
        p.hurt(20, 0.0, &mut tr_core::UnifiedRandom::default(), true);
        assert_eq!(p.hp, 99);
    }

    #[test]
    fn the_potion_sickness_buff_gates_and_expires() {
        // The buff 21 (`BuffID.PotionSickness`): the drink arm gates on it,
        // and the 3600-tick timer expires like any buff.
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        p.buffs.add_buff(21, 3600);
        assert!(p.buffs.find_buff_index(21).is_some());
        p.update(&World::new("t".into(), 64, 800), PlayerInput::default());
        assert!(p.buffs.find_buff_index(21).is_some());
        // Expire it.
        for _ in 0..3600 {
            p.update(&World::new("t".into(), 64, 800), PlayerInput::default());
        }
        assert!(p.buffs.find_buff_index(21).is_none());
    }

    #[test]
    fn the_fed_tiers_stack_their_stat_adds() {
        // `Player.cs:11931-11969`: the three tiers' per-slot adds — the
        // crits/damage/speed ride the scan, the defense waits for
        // M3-EQUIP-3, the minion halves for M4-MINION.
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        p.buffs.add_buff(26, 600);
        p.update(&World::new("t".into(), 64, 800), PlayerInput::default());
        assert!(p.well_fed);
        assert_eq!(p.melee_crit, 4 + 2);
        assert!((p.melee_damage - 1.05).abs() < 1e-4);
        assert!((p.move_speed - 1.2).abs() < 1e-4);
        assert!((p.pick_speed_bonus - 0.05).abs() < 1e-4);
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        p.buffs.add_buff(207, 600);
        p.update(&World::new("t".into(), 64, 800), PlayerInput::default());
        assert_eq!(p.melee_crit, 4 + 4);
        assert!((p.melee_damage - 1.1).abs() < 1e-4);
        assert!((p.move_speed - 1.4).abs() < 1e-4);
    }

    #[test]
    fn the_fed_flag_clears_when_the_buff_expires() {
        // ResetEffects' per-tick derivation: the flag rides the slots.
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        p.buffs.add_buff(26, 2);
        p.update(&World::new("t".into(), 64, 800), PlayerInput::default());
        assert!(p.well_fed);
        for _ in 0..3 {
            p.update(&World::new("t".into(), 64, 800), PlayerInput::default());
        }
        assert!(!p.well_fed);
    }

    #[test]
    fn the_sunflower_buff_moves_and_the_star_buffs_mana_regen() {
        // `Player.cs:11001-11004`: the 146 slot — `moveSpeed += 0.1; *= 1.1`
        // (1.0 -> 1.21); `:9932-9934`: the 158 slot — `manaRegenBonus += 10`.
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        p.buffs.add_buff(146, 60);
        p.update(&World::new("t".into(), 64, 800), PlayerInput::default());
        assert!((p.move_speed - 1.21).abs() < 1e-4);
        assert!((p.max_run_speed - 3.0 * 1.21).abs() < 1e-3);
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        p.buffs.add_buff(158, 60);
        p.update(&World::new("t".into(), 64, 800), PlayerInput::default());
        assert_eq!(p.mana_regen_bonus, 10);
    }

    #[test]
    fn the_aura_buffs_grant_from_the_scene_snapshot() {
        // `Player.cs:25525-25555`: the 2-tick indicator buffs refresh from
        // the scan; the effect cases read the previous tick's grant (the
        // sunflower's movement starts one tick after the aura appears).
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        p.auras.campfire = true;
        p.auras.heart_lantern = true;
        p.auras.star_in_bottle = true;
        p.auras.sunflower = true;
        p.update(&World::new("t".into(), 64, 800), PlayerInput::default());
        let types = p.buffs.active_types();
        assert!(types.contains(&87) && types.contains(&89) && types.contains(&158));
        // The grant happened after this tick's scan, so 146 applies NEXT
        // tick — the move_speed is still 1.0 here.
        assert!((p.move_speed - 1.0).abs() < 1e-4);
        p.update(&World::new("t".into(), 64, 800), PlayerInput::default());
        assert!((p.move_speed - 1.21).abs() < 1e-4);
    }

    #[test]
    fn movement_speed_per_liquid() {
        // Player.cs:28168-28171: water/lava 0.5, honey 0.25, shimmer 0.375,
        // dry 1.0.
        assert_eq!(movement_speed_for(true, false, false, false), 0.5);
        assert_eq!(movement_speed_for(false, true, false, false), 0.5);
        assert_eq!(movement_speed_for(false, false, true, false), 0.25);
        assert_eq!(movement_speed_for(false, false, false, true), 0.375);
        assert_eq!(movement_speed_for(false, false, false, false), 1.0);
    }

    #[test]
    #[test]
    fn the_wet_statics_rebase_feeds_the_tick() {
        // The wiring: a water tick rebase-assigns the jump statics (the C#
        // `Player.jumpHeight = 30` / `jumpSpeed = 6.01f`, `:24418-24424`) —
        // the values the jump section consumes.
        let mut w = ground_world();
        for y in 58..60 {
            for x in 0..64 {
                let i = w.index(x, y);
                w.tiles[i].liquid = 200;
            }
        }
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        assert!(p.wet, "the tick must be wet: center row {}", (p.y / TILE_PX) as i32);
        assert_eq!(p.jump_height, 30, "the water jump height");
        assert!((p.jump_speed - 6.01).abs() < 1e-5, "the water jump speed");
    }

    #[test]
    fn the_wet_statics_rebase_assigns_the_per_liquid_arms() {
        // `Player.cs:24418-24424` (water): 0.2/5 + the jump statics 30/6.01.
        let water = WetStaticsFlags {
            wet: true,
            ..Default::default()
        };
        let (g, mf, jh, js) = wet_gravity_and_cap(GRAVITY, MAX_FALL, 15, 5.01, water);
        assert!((g - 0.2).abs() < 1e-5 && (mf - 5.01).abs() < 1e-5);
        assert_eq!((jh, js), (30, 6.01));
        // `:24399-24402` (honey): 0.1/3, the jump statics stay at the reset.
        let honey = WetStaticsFlags {
            wet: true,
            honey_wet: true,
            ..Default::default()
        };
        let (g, mf, jh, js) = wet_gravity_and_cap(GRAVITY, MAX_FALL, 15, 5.01, honey);
        assert!((g - 0.1).abs() < 1e-5 && (mf - 3.01).abs() < 1e-5);
        assert_eq!((jh, js), (15, 5.01));
        // `:24404-24407` (merman): 0.3/7.
        let merman = WetStaticsFlags {
            wet: true,
            merman: true,
            ..Default::default()
        };
        let (g, mf, jh, js) = wet_gravity_and_cap(GRAVITY, MAX_FALL, 15, 5.01, merman);
        assert!((g - 0.3).abs() < 1e-5 && (mf - 7.01).abs() < 1e-5);
        assert_eq!((jh, js), (15, 5.01));
        // `:24409-24417` (trident, not lava): 0.25/6 + 25/5.51; controlUp
        // drops it to 0.1/2.
        let trident = WetStaticsFlags {
            wet: true,
            trident: true,
            ..Default::default()
        };
        let (g, mf, jh, js) = wet_gravity_and_cap(GRAVITY, MAX_FALL, 15, 5.01, trident);
        assert!((g - 0.25).abs() < 1e-5 && (mf - 6.01).abs() < 1e-5);
        assert_eq!((jh, js), (25, 5.51));
        let trident_up = WetStaticsFlags {
            control_up: true,
            ..trident
        };
        let (g, mf, jh, js) = wet_gravity_and_cap(GRAVITY, MAX_FALL, 15, 5.01, trident_up);
        assert!((g - 0.1).abs() < 1e-5 && (mf - 2.01).abs() < 1e-5);
        assert_eq!((jh, js), (25, 5.51));
        // Trident in lava falls through to the plain water arm.
        let trident_lava = WetStaticsFlags {
            lava_wet: true,
            ..trident
        };
        let (g, mf, jh, js) = wet_gravity_and_cap(GRAVITY, MAX_FALL, 15, 5.01, trident_lava);
        assert!((g - 0.2).abs() < 1e-5 && (mf - 5.01).abs() < 1e-5);
        assert_eq!((jh, js), (30, 6.01));
        // `:24389-24394` (shimmerWet, not absorbed): 0.15 + 23/5.51 — the
        // arm never touches the cap, so it stays at the 10 default.
        let shimmer = WetStaticsFlags {
            shimmer_wet: true,
            ..Default::default()
        };
        let (g, mf, jh, js) = wet_gravity_and_cap(GRAVITY, MAX_FALL, 15, 5.01, shimmer);
        assert!((g - 0.15).abs() < 1e-5 && (mf - 10.01).abs() < 1e-5);
        assert_eq!((jh, js), (23, 5.51));
    }

    #[test]
    fn the_rebase_multipliers_gates_and_creep() {
        // `:24384-24387`: the Dead Cells down-dash arm ×0.85 (wet, not
        // shimmering) — 0.4×0.85 / 10×0.85 + 0.01.
        let dash = WetStaticsFlags {
            wet: true,
            down_dash: true,
            ..Default::default()
        };
        let (g, mf, jh, js) = wet_gravity_and_cap(GRAVITY, MAX_FALL, 15, 5.01, dash);
        assert!((g - 0.4 * 0.85).abs() < 1e-5 && (mf - (10.0 * 0.85 + 0.01)).abs() < 1e-5);
        assert_eq!((jh, js), (15, 5.01));
        // `:24389-24393`: the absorbed shimmering arm ×0.9 — 0.36 / 9.01.
        let shimmering = WetStaticsFlags {
            shimmering: true,
            ..Default::default()
        };
        let (g, mf, jh, js) = wet_gravity_and_cap(GRAVITY, MAX_FALL, 15, 5.01, shimmering);
        assert!((g - 0.36).abs() < 1e-5 && (mf - 9.01).abs() < 1e-5);
        assert_eq!((jh, js), (15, 5.01));
        // The down-dash arm loses to the shimmer arm when shimmering.
        let shimmer_dash = WetStaticsFlags {
            wet: true,
            shimmering: true,
            down_dash: true,
            ..Default::default()
        };
        let (g, mf, _, _) = wet_gravity_and_cap(GRAVITY, MAX_FALL, 15, 5.01, shimmer_dash);
        assert!((g - 0.36).abs() < 1e-5 && (mf - 9.01).abs() < 1e-5);
        // `:24433-24436`: the vortex debuff zeroes the gravity AFTER the
        // wet arm assigned it.
        let vortex = WetStaticsFlags {
            wet: true,
            vortex_debuff: true,
            ..Default::default()
        };
        let (g, mf, jh, js) = wet_gravity_and_cap(GRAVITY, MAX_FALL, 15, 5.01, vortex);
        assert!(g == 0.0 && (mf - 5.01).abs() < 1e-5);
        assert_eq!((jh, js), (30, 6.01));
        // `:24380-24382`: portal physics pins the cap at 35 before the wet
        // arm re-assigns it (dry here, so 35 survives + the creep).
        let portal = WetStaticsFlags {
            portal_physics: true,
            ..Default::default()
        };
        let (g, mf, _, _) = wet_gravity_and_cap(GRAVITY, MAX_FALL, 15, 5.01, portal);
        assert!((g - 0.4).abs() < 1e-5 && (mf - 35.01).abs() < 1e-5);
        // Dry: the plain arm rides the defaults + the creep — the C#'s
        // every-tick `maxFallSpeed += 0.01f` (`:24437`).
        let (g, mf, jh, js) =
            wet_gravity_and_cap(GRAVITY, MAX_FALL, 15, 5.01, WetStaticsFlags::default());
        assert!((g - 0.4).abs() < 1e-5 && (mf - 10.01).abs() < 1e-5);
        assert_eq!((jh, js), (15, 5.01));
    }

    #[test]
    fn the_wet_rebase_runs_before_the_jump_boost_lift() {
        // The C# order: the reset (`:24366`) → the rebase (`:24380-24437`)
        // → `UpdateJumpHeight`'s max-lift (`:25656`), so the wet 30/6.01
        // survives the lift and a jump-boosted shimmer keeps the wet 23
        // height while the lift raises the speed to 6.51.
        let shimmer = WetStaticsFlags {
            shimmer_wet: true,
            ..Default::default()
        };
        let (g, mf, jh, js) = wet_gravity_and_cap(GRAVITY, MAX_FALL, 15, 5.01, shimmer);
        assert!((g - 0.15).abs() < 1e-5 && (mf - 10.01).abs() < 1e-5);
        assert_eq!((jh.max(20), js.max(6.51)), (23, 6.51));
    }

    #[test]
    fn respawn_time_base_boss_and_expert() {
        // Player.cs:39060-39082: 600 base, +600 boss-near, x1.5 expert.
        assert_eq!(Player::get_respawn_time(false, false), 600);
        assert_eq!(Player::get_respawn_time(true, false), 1200);
        assert_eq!(Player::get_respawn_time(false, true), 900);
        assert_eq!(Player::get_respawn_time(true, true), 1800);
    }

    #[test]
    fn update_dead_counts_down_and_revives_at_zero() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        p.start_death(&w, false, false);
        assert!(p.dead);
        assert_eq!(p.respawn_timer, 600);
        // 600 ticks of the dead state (no jump) -> the timer drains to 0 and
        // the last tick reports the respawn.
        let mut revived_at = None;
        for tick in 0..700 {
            if p.update_dead(false, false, false, false) {
                revived_at = Some(tick);
                break;
            }
        }
        assert_eq!(revived_at, Some(599));
        p.revive();
        assert!(!p.dead);
        assert_eq!(p.respawn_timer, 0);
    }

    #[test]
    fn respawn_timer_clamps_at_3600() {
        // Player.respawnTimerMax = 3600 (Player.cs:57109): a boss-near expert
        // death (1800) halved by nothing still clamps; the clamp matters for
        // the >3600 totals (the FTW x2 stays out). A forced high timer holds.
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        p.start_death(&w, true, true);
        p.respawn_timer = 4000; // the save-load can exceed the cap
        p.update_dead(false, false, false, false);
        // The clamp catches the decrement result (Player.cs:17049).
        assert_eq!(p.respawn_timer, 3600);
    }

    #[test]
    fn respawn_life_is_half_the_pool_floored_at_100() {
        // `Player.cs:37598-37608`: the pool is `max(statLifeMax,
        // statLifeMax2)` — the port's single `max_hp` is the effective max —
        // and the life is `max(pool / 2, 100)`: a fresh 100-life character
        // revives FULL, 400 revives at 200.
        let w = World::new("t".into(), 64, 128);
        let mut p = Player::spawn(&w);
        p.max_hp = 400;
        p.hp = 0;
        p.respawn();
        assert_eq!(p.hp, 200);
        assert_eq!(p.breath, BREATH_MAX);
        // The i-frames (`:37614-37618` + `:37661-37668`): the death revive
        // arms 180 with `immuneNoBlink` (the player is never hostile here).
        assert!(p.immune);
        assert_eq!(p.immune_time, 180);
        assert!(p.immune_no_blink);
    }

    #[test]
    fn spawn_iframes_swallow_hits_and_expire() {
        // `Player.cs:38210-38216`: the General cooldown flag is `!immune` —
        // the spawn i-frames swallow the hit; `UpdateImmunity`
        // (`:18856-18869`) clears it at 0 and the damage lands again.
        let w = World::new("t".into(), 64, 128);
        let mut p = Player::spawn(&w);
        p.respawn();
        let mut scratch = tr_core::UnifiedRandom::default();
        assert!(
            !p.hurt(50, 0.0, &mut scratch, true),
            "the i-frames swallow the hit"
        );
        p.immune_time = 1;
        p.update_immunity();
        assert!(!p.immune, "the countdown cleared the flag");
        assert!(!p.immune_no_blink);
        assert!(
            p.hurt(50, 0.0, &mut scratch, true),
            "the damage lands after the i-frames"
        );
    }

    #[test]
    fn the_nurse_skips_protected_debuffs_in_both_gates() {
        // `BuffID.Sets.NurseCannotRemoveDebuff` (`BuffID.cs:52`): the
        // protected Potion Sickness 21 neither counts in the price
        // (`Main.cs:40061-40064`, the `> 60` time gate too) nor clears at
        // the heal (`:40147`); the unprotected Poisoned 20 does both.
        let w = World::new("t".into(), 64, 128);
        let mut p = Player::spawn(&w);
        p.buffs.add_buff(21, 3600); // protected
        p.buffs.add_buff(20, 3600); // removable
        assert_eq!(p.remountable_debuff_count(), 1, "the protected one stays out of the price");
        p.clear_nurse_removable_debuffs();
        assert!(p.buffs.find_buff_index(21).is_some(), "the protected one survives");
        assert_eq!(p.buffs.find_buff_index(20), None, "the removable clears");
    }

    #[test]
    fn the_nurse_price_ignores_young_debuffs() {
        // `Main.cs:40062`: the cost counts only `buffTime > 60` slots; the
        // removal (`:40147`) takes any `buffTime > 0`.
        let w = World::new("t".into(), 64, 128);
        let mut p = Player::spawn(&w);
        p.buffs.add_buff(20, 50); // under the 60-tick price gate
        assert_eq!(p.remountable_debuff_count(), 0);
        p.buffs.buff_time[p.buffs.find_buff_index(20).unwrap()] = 1;
        p.clear_nurse_removable_debuffs();
        assert_eq!(p.buffs.find_buff_index(20), None, "the removal clears at time 1");
    }

    #[test]
    fn buff_removal_refuses_debuffs_and_dels_the_rest() {
        // `Main.cs:43370-43389`: the non-debuff arm dels the slot; a debuff
        // (or 60/151) refuses.
        let w = World::new("t".into(), 64, 128);
        let mut p = Player::spawn(&w);
        p.buffs.add_buff(5, 3600); // Ironskin — a non-debuff
        p.buffs.add_buff(20, 3600); // Poisoned — a debuff
        let poison = p.buffs.find_buff_index(20).unwrap();
        let iron = p.buffs.find_buff_index(5).unwrap();
        assert!(!p.try_removing_buff(poison, 20), "a debuff refuses");
        assert!(p.try_removing_buff(iron, 5), "the non-debuff removes");
        assert_eq!(p.buffs.find_buff_index(5), None);
    }

    #[test]
    fn buff_removal_dismounts_the_mount_buff_without_del() {
        // `Main.cs:43376-43380`: the mount's own buff (`Mount.CheckBuff` =
        // `data.buff == b`) dismounts and clears the buff — no `DelBuff`.
        let w = World::new("t".into(), 64, 128);
        let mut p = Player::spawn(&w);
        p.mount_cart();
        let mount_buff = p.mount.data().unwrap().buff;
        p.buffs.add_buff(mount_buff, 3600);
        let slot = p.buffs.find_buff_index(mount_buff).unwrap();
        assert!(p.try_removing_buff(slot, mount_buff), "the arm entered");
        assert!(!p.mount.is_active(), "the mount dismounted");
    }

    #[test]
    fn the_moon_events_block_the_respawn_skip() {
        // `Player.cs:38984-38986`: `Main.snowMoon || Main.pumpkinMoon ||
        // DD2Event.Ongoing` refuses the timer skip.
        let w = World::new("t".into(), 64, 128);
        let mut p = Player::spawn(&w);
        p.dead_time = 200;
        p.respawn_timer = 600;
        assert!(p.allows_respawn_timer_skip(false, false, false));
        assert!(!p.allows_respawn_timer_skip(false, false, true));
    }

    /// BUG-06b: `Player.NearAnyNPCsThatBlockRespawn` (`Player.cs:38987-39018`)
    /// — the live invasion's members inside the view rect inflated by 320 px
    /// block the skip; the same NPCs do not while the invasion is over, and a
    /// non-invasion NPC never does (the danger-set arm has no ported members).
    #[test]
    fn bug_06b_only_the_live_invasions_npcs_block_the_respawn_skip() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        let (px, py) = p.center();
        let goblin = crate::npc::Npc::new(crate::npc::NpcKind::GoblinWarrior, px + 200.0, py, 1, 0.0);
        let slime = crate::npc::Npc::new(crate::npc::NpcKind::Slime, px + 200.0, py, 1, 0.0);
        // Inside the rect while the invasion runs — blocked.
        assert!(p.near_any_npcs_that_block_respawn(&[goblin.clone()], true));
        // The invasion ended: the same goblin no longer blocks.
        assert!(!p.near_any_npcs_that_block_respawn(&[goblin.clone()], false));
        // A commoner NPC is not an invasion group member.
        assert!(!p.near_any_npcs_that_block_respawn(&[slime], true));
        // Out of the inflated view rect (1920x1080 + 320 px): no block.
        let far = crate::npc::Npc::new(
            crate::npc::NpcKind::GoblinWarrior,
            px + 4000.0,
            py,
            1,
            0.0,
        );
        assert!(!p.near_any_npcs_that_block_respawn(&[far], true));
        // The gate itself: a nearby danger NPC vetoes the skip.
        p.dead_time = 200;
        assert!(p.allows_respawn_timer_skip(false, false, false));
        assert!(!p.allows_respawn_timer_skip(false, true, false));
    }

    #[test]
    fn the_skip_needs_the_90_tick_lockout_and_a_jump_edge() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        p.start_death(&w, false, false);
        // Inside the 90-tick lockout: no skip even on the jump edge (the
        // held jump's edge is consumed while the lockout runs).
        for _ in 0..89 {
            assert!(!p.update_dead(true, false, false, false));
        }
        // The lockout expired but the held jump has no fresh edge; a release
        // and a new press do.
        assert!(!p.update_dead(true, false, false, false));
        assert!(!p.update_dead(false, false, false, false));
        assert!(p.update_dead(true, false, false, false));
    }

    #[test]
    fn the_skip_is_blocked_while_a_boss_hinders() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        p.start_death(&w, false, false);
        for _ in 0..95 {
            p.update_dead(false, false, false, false);
        }
        assert!(!p.allows_respawn_timer_skip(true, false, false));
        assert!(p.allows_respawn_timer_skip(false, false, false));
    }

    #[test]
    fn the_jump_edge_is_required_for_the_skip() {
        // Holding jump through the lockout does not skip on expiry; a fresh
        // press does (the JustBecameTrue semantics of PlayerInput.Triggers).
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        p.start_death(&w, false, false);
        for _ in 0..95 {
            p.update_dead(true, false, false, false);
        }
        // The held jump consumed its edge at the lockout expiry...
        assert!(!p.update_dead(true, false, false, false));
        // ...release and press again: the edge fires.
        p.update_dead(false, false, false, false);
        assert!(p.update_dead(true, false, false, false));
    }

    use super::*;
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    fn ground_world() -> World {
        let mut w = World::new("t".into(), 64, 128);
        for x in 0..64 {
            for y in 60..128 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.spawn_x = 10;
        w.spawn_y = 40;
        w
    }

    #[test]
    fn falls_and_lands_on_ground() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        assert!(p.on_ground, "player should land, y={}", p.y);
        assert!((p.y - (60.0 * TILE_PX - HEIGHT)).abs() < 1.0, "y={}", p.y);
    }

    /// Drops the player from `from_tile` onto the ground_world surface (top
    /// row 60) and simulates until rest. Returns (hp lost, landed tile row).
    fn drop_and_settle(w: &World, from_tile: i32) -> (i32, i32) {
        let mut p = Player::spawn(w);
        let hp0 = p.hp;
        p.y = (from_tile * 16) as f32;
        p.fall_start = from_tile;
        for _ in 0..600 {
            p.update(w, PlayerInput::default());
            if p.on_ground {
                break;
            }
        }
        (hp0 - p.hp, (p.y / TILE_PX) as i32)
    }

    /// A `ground_world` whose surface top row is `kind` (ice 161, cobweb 229,
    /// asphalt 198 — all solid in `TILE_SOLID`).
    fn floor_world(kind: u16) -> World {
        let mut w = ground_world();
        for x in 0..64 {
            let i = w.index(x, 60);
            w.tiles[i].kind = kind;
        }
        w
    }

    /// Walks right for `ticks` with the input held, returning the final vx.
    fn walk(w: &World, ticks: u32) -> f32 {
        let mut p = Player::spawn(w);
        let input = PlayerInput {
            right: true,
            ..Default::default()
        };
        for _ in 0..ticks {
            p.update(w, input);
        }
        p.vx
    }

    /// `Player.cs:26560-26570`: plain ice multiplies `runSlowdown` by 0.1 when
    /// nobody skates — releasing the key, the player slides ten times further
    /// than on dirt.
    #[test]
    fn ice_slides_without_skates() {
        let ice = floor_world(161);
        let mut p = Player::spawn(&ice);
        let input = PlayerInput {
            right: true,
            ..Default::default()
        };
        for _ in 0..60 {
            p.update(&ice, input);
        }
        let vx0 = p.vx;
        for _ in 0..30 {
            p.update(&ice, PlayerInput::default());
        }
        let on_ice = p.vx;
        let mut q = Player::spawn(&ground_world());
        q.vx = vx0;
        for _ in 0..30 {
            q.update(&ground_world(), PlayerInput::default());
        }
        let on_dirt = q.vx;
        assert!(vx0 > 2.5, "should have accelerated, vx={vx0}");
        assert!(on_ice > 1.5, "ice keeps sliding: {on_ice}");
        assert!(on_dirt < 0.05, "dirt friction stops the player: {on_dirt}");
    }

    /// `Player.cs:26562-26565`: with `iceSkate` the ice multiplies
    /// `runAcceleration` by 3.5 and `maxRunSpeed` by 1.25 — the top speed on
    /// ice exceeds the dirt one (3.0).
    #[test]
    fn ice_skates_raise_the_top_speed() {
        let ice = floor_world(161);
        let mut p = Player::spawn(&ice);
        p.equipped[3] = 950; // the Ice Skates (`Player.cs:15089-15092`)
        let input = PlayerInput {
            right: true,
            ..Default::default()
        };
        for _ in 0..120 {
            p.update(&ice, input);
        }
        assert!(
            p.vx > 3.7 && p.vx < 4.0,
            "skate top speed ≈ 3.75, vx={}",
            p.vx
        );
        let dirt_top = walk(&ground_world(), 120);
        assert!(
            dirt_top > 2.9 && dirt_top < 3.2,
            "dirt top speed = 3, vx={dirt_top}"
        );
    }

    /// `Player.cs:26522-26534`: the cobweb multiplies speed 0.25x, slowdown
    /// 2x, and hard-clamps the current velocity to the reduced max.
    #[test]
    fn cobweb_slow_and_clamps() {
        let web = floor_world(229);
        let mut p = Player::spawn(&web);
        for _ in 0..300 {
            p.update(&web, PlayerInput::default());
            if p.on_ground {
                break;
            }
        }
        // Standing on the web, the flags are live: an entry speed at
        // knockback strength clamps to the reduced max on the next tick.
        p.vx = 4.5;
        p.update(&web, PlayerInput::default());
        assert!(p.vx < 1.0, "web clamps and drags: vx={}", p.vx);
    }

    /// `Player.cs:23436-23451`: in water the free axes integrate at 0.5, in
    /// honey at 0.25 — momentum (vx) is untouched, the stride halves.
    #[test]
    fn water_halves_the_stride() {
        let mut w = ground_world();
        for y in 58..60 {
            for x in 0..64 {
                let i = w.index(x, y);
                w.tiles[i].liquid = 200;
            }
        }
        let mut p = Player::spawn(&w);
        let input = PlayerInput {
            right: true,
            ..Default::default()
        };
        for _ in 0..90 {
            p.update(&w, input);
        }
        // Both runners are settled into their gait; measure the stride over
        // the next 30 ticks only.
        let x0 = p.x;
        let mut q = Player::spawn(&ground_world());
        for _ in 0..90 {
            q.update(&ground_world(), input);
        }
        let q0 = q.x;
        for _ in 0..30 {
            p.update(&w, input);
            q.update(&ground_world(), input);
        }
        let ratio = (p.x - x0) / (q.x - q0);
        assert!(
            (0.4..0.65).contains(&ratio),
            "wet stride ≈ half the dry one: {ratio}"
        );
        // Momentum survives: the velocity is not scaled, only the stride.
        assert!(p.vx > 2.0, "vx preserved in water: {}", p.vx);
    }

    /// `Player.cs:20059-20087`: no input decays by `runSlowdown` on the ground
    /// and half of it in the air (0.2 vs 0.1 per tick).
    #[test]
    fn air_friction_is_half_the_ground_friction() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.vx = 2.0;
        p.on_ground = false;
        p.vy = -3.0; // airborne
        p.update(&w, PlayerInput::default());
        let air = 2.0 - p.vx;
        let mut q = Player::spawn(&w);
        q.vx = 2.0;
        q.on_ground = true;
        q.vy = 0.0;
        q.update(&w, PlayerInput::default());
        let ground = 2.0 - q.vx;
        assert!(
            (air - ground * 0.5).abs() < 1e-4,
            "air {air} vs ground {ground}"
        );
    }

    /// The port used to hard-clamp vx to ±3 every tick — the vanilla has no
    /// clamp: a knockback-speed vx (4.5) decays by friction instead.
    #[test]
    fn overspeed_rides_out() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.vx = 4.5;
        p.update(&w, PlayerInput::default());
        assert!(
            p.vx > 4.0 && p.vx < 4.5,
            "no clamp, friction only: vx={}",
            p.vx
        );
    }

    /// Simulates the double-tap: a right press edge, a release, a second
    /// press `gap` ticks later. Returns (dashDelay, eocDash, vx) one tick
    /// after the second press.
    fn double_tap(w: &World, gap: u32) -> (i32, i32, f32) {
        let mut p = Player::spawn(w);
        for _ in 0..120 {
            p.update(w, PlayerInput::default());
        }
        p.dash_type = 2;
        p.update(
            w,
            PlayerInput {
                right: true,
                ..Default::default()
            },
        );
        p.update(w, PlayerInput::default());
        for _ in 0..gap {
            p.update(w, PlayerInput::default());
        }
        p.update(
            w,
            PlayerInput {
                right: true,
                ..Default::default()
            },
        );
        p.update(w, PlayerInput::default());
        (p.dash_delay, p.eoc_dash, p.vx)
    }

    /// `Player.cs:21571-21600`: a second same-direction tap inside the 15-tick
    /// window fires the dash — velocity 14.5 (dash 2), `dashDelay = -1`, the
    /// EoC grace armed at 15.
    #[test]
    fn double_tap_fires_the_dash() {
        let w = ground_world();
        let (delay, eoc, vx) = double_tap(&w, 5);
        assert_eq!(delay, -1, "dashing");
        assert_eq!(eoc, 15, "the EoC grace arms at the start");
        // The launch tick already took the ground friction AND the first
        // decay: (14.5 - 0.2) x 0.985.
        assert!(
            (vx - 14.3 * 0.985).abs() < 1e-4,
            "shield dash speed, vx={vx}"
        );
    }

    /// The window: a second tap after 15 ticks is too late — `dashTime` has
    /// decayed to 0 and re-arms instead of firing.
    #[test]
    fn double_tap_window_is_15_ticks() {
        let w = ground_world();
        let (delay, _, _) = double_tap(&w, 20);
        assert_eq!(delay, 0, "outside the window");
    }

    /// `Player.cs:21374-21398`: the dash decays 0.985/tick above the 12-tile
    /// threshold; the end starts the 30-tick shield cooldown and pins the
    /// velocity to the run band.
    #[test]
    fn shield_dash_decays_into_cooldown() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.dash_type = 2;
        p.dash = 2;
        p.dash_delay = -1;
        p.eoc_dash = 15;
        p.vx = 14.5;
        let mut max_delay = 0;
        for _ in 0..60 {
            p.update(&w, PlayerInput::default());
            max_delay = max_delay.max(p.dash_delay);
        }
        // The dash ended into the 30-tick shield cooldown (the highest count
        // observed) and the cooldown has fully run out by now.
        assert_eq!(max_delay, 30, "the shield cooldown");
        assert_eq!(p.dash_delay, 0, "the cooldown expired");
        assert!(
            p.vx.abs() <= 3.0 + 0.09,
            "pinned to the run band: vx={}",
            p.vx
        );
        assert_eq!(p.eoc_dash, 0, "the grace drained on the cooldown");
    }

    /// `Player.cs:21414-21417`: a solid tile ahead halves the launch speed.
    #[test]
    fn wall_halves_the_launch() {
        let mut w = ground_world();
        for y in 55..62 {
            for x in [12, 13] {
                let i = w.index(x, y);
                w.tiles[i].kind = 1;
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        let mut p = Player::spawn(&w);
        p.dash_type = 2;
        let held = PlayerInput {
            right: true,
            ..Default::default()
        };
        p.update(&w, held);
        p.update(&w, PlayerInput::default());
        for _ in 0..3 {
            p.update(&w, PlayerInput::default());
        }
        p.update(&w, held);
        p.update(&w, PlayerInput::default());
        assert!(
            p.vx.abs() < 14.5,
            "the launch halved against the wall: vx={}",
            p.vx
        );
    }

    /// The wings flight: `TR_WINGS`-shaped state — wings_logic 1 (100-tick
    /// budget), jump held midair after the normal jump drains: the ascent
    /// caps at `-jumpSpeed * 1.66` and the budget drains 1/tick.
    #[test]
    fn wings_lift_and_drain_the_budget() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        let ground_y = p.y;
        // Demon Wings grant `wings_logic`/`wing_time_max` inside the tick.
        p.equipped[3] = 492;
        p.wing_time = 100.0;
        let held = PlayerInput {
            jump: true,
            ..Default::default()
        };
        // Jump first (the flight needs `jump == 0`), then hold.
        p.update(&w, held);
        let mut min_vy = 0.0f32;
        for _ in 0..60 {
            p.update(&w, held);
            min_vy = min_vy.min(p.vy);
        }
        assert!(p.y < ground_y - 200.0, "the player rose: y={}", p.y);
        assert!(
            min_vy >= -JUMP_SPEED * 1.66 - 0.5 && min_vy < 0.0,
            "the ascent cap: {min_vy}"
        );
        assert!(p.wing_time < 100.0, "the budget drains: {}", p.wing_time);
    }

    /// `Player.cs:26776-26781`: the wing refill on the ground needs the jump
    /// RELEASE — holding jump on the ground does not refill.
    #[test]
    fn wing_refill_needs_the_jump_release() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        // Demon Wings grant `wings_logic`/`wing_time_max` inside the tick.
        p.equipped[3] = 492;
        p.wing_time = 30.0;
        // Land and release: the refill.
        p.update(&w, PlayerInput::default());
        assert_eq!(p.wing_time, 100.0, "released on the ground");
        // Hold jump: no refill while grounded.
        let held = PlayerInput {
            jump: true,
            ..Default::default()
        };
        p.wing_time = 30.0;
        p.update(&w, held);
        assert_eq!(p.wing_time, 30.0, "holding jump blocks the refill");
    }

    /// `Player.cs:26867-26955`: the rocket boots — a midair jump release
    /// arms `rocketRelease`; holding jump again burns a fuel tick per 10
    /// thrust ticks at 0.1 + brake impulses, capped at `-jumpSpeed * 1.5`.
    #[test]
    fn rocket_boots_thrust_after_the_rearm() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        p.pinned_rocket_boots = 2;
        p.rocket_boots = 2;
        p.rocket_time = 7;
        // Jump and release midair: the arming.
        let held = PlayerInput {
            jump: true,
            ..Default::default()
        };
        p.update(&w, held);
        p.update(&w, PlayerInput::default());
        assert!(p.rocket_release, "released midair");
        // Hold again: the thrust (the ascent cap -5.01 * 1.5 = -7.515).
        let mut min_vy = 0.0f32;
        for _ in 0..30 {
            p.update(&w, held);
            min_vy = min_vy.min(p.vy);
        }
        assert!(
            min_vy <= -JUMP_SPEED * 1.5 + 0.5,
            "the thrust cap: {min_vy}"
        );
        assert!(p.rocket_time < 7, "the fuel drains: {}", p.rocket_time);
    }

    /// `Player.cs:26825-26834`: with wings AND boots the rocket fuel
    /// converts into wing time (+6 per fuel tick, capped past the max)
    /// before the rocket can trigger.
    #[test]
    fn rocket_fuel_converts_into_wing_time() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        // The wings come from the item now (Demon Wings = `ArmorIDs.Wing` 1,
        // 100 ticks): the grant runs inside the tick, so the budget starts
        // empty and the rocket fuel is the only source this tick.
        p.equipped[3] = 492;
        p.pinned_rocket_boots = 2;
        p.rocket_boots = 2;
        p.rocket_time = 7;
        // Airborne with both: the conversion consumed the fuel.
        p.vy = 1.0;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.rocket_time, 0, "the fuel converted");
        assert_eq!(p.wing_time, 42.0, "7 fuel ticks x 6 = 42 wing ticks");
    }

    /// `Player.GrappleMovement` (`Player.cs:22613-22740`): while a hook is
    /// attached the pull replaces the velocity — the vector to the hook
    /// center clamped to the pull speed.
    #[test]
    fn grapple_pull_replaces_the_velocity() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        let hook_center = Vec2::new(p.x + 80.0, p.y - 60.0);
        // One registered Hook — the multi-hook seam's single-member case.
        let grapple: &[(u16, Vec2, f32)] = &[(crate::projectile::HOOK, hook_center, 0.0)];
        p.update_with_grapple(&w, PlayerInput::default(), grapple, false, false, false, None, &mut tr_core::UnifiedRandom::default());
        // The pull toward the hook: dx +, dy -.
        assert!(p.vx > 0.0 && p.vy < 0.0, "pulling toward the hook");
        // The speed cap 11 px/tick for the Hook (13).
        assert!(
            Vec2::new(p.vx, p.vy).length() <= 11.0 + 1e-4,
            "the pull speed capped: {}",
            Vec2::new(p.vx, p.vy).length()
        );
        // GoingDownWithGrapple: the pull points up here.
        assert!(!p.going_down_with_grapple);
    }

    /// `Player.cs:22645`: the fall origin refreshes per tick while
    /// grappling — a long pull never accrues fall damage.
    #[test]
    fn grapple_refreshes_the_fall_origin() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        let start_row = (p.y / TILE_PX) as i32;
        let center = Vec2::new(p.x, p.y - 300.0);
        let grapple: &[(u16, Vec2, f32)] = &[(crate::projectile::HOOK, center, 0.0)];
        for _ in 0..40 {
            p.update_with_grapple(&w, PlayerInput::default(), grapple, false, false, false, None, &mut tr_core::UnifiedRandom::default());
            // GrappleMovement refreshes the origin BEFORE the position
            // integrates — one row behind the current position, like the
            // original's call order (:27645 vs the collision block).
            let row = (p.y / TILE_PX) as i32;
            assert!(
                (p.fall_start - row).abs() <= 1,
                "the origin rides the pull: {} vs {row}",
                p.fall_start
            );
        }
        let _ = start_row;
    }

    /// `Player.cs:22705-22739`: the jump-off — a fresh jump press while
    /// nearly stationary grants the full jump and detaches.
    #[test]
    fn grapple_jump_off_detaches() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        // Settle, then latch the hook right above (a near-stationary hang).
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        // The registered set (`grappling[]`/`grapCount`): one attached Hook.
        let center = Vec2::new(p.x + WIDTH * 0.5, p.y - 4.0);
        let grapple: &[(u16, Vec2, f32)] = &[(crate::projectile::HOOK, center, 0.0)];
        // The press edge: the jump-off.
        p.update_with_grapple(
            &w,
            PlayerInput {
                jump: true,
                ..Default::default()
            },
            grapple,
            false,
            false,
            false,
            None,
            &mut tr_core::UnifiedRandom::default(),
        );
        assert!(!p.grappling_active, "detached");
        assert!(p.vy < 0.0, "the full jump impulse: vy={}", p.vy);
    }

    /// `Player.cs:22713-22717`: holding DOWN cancels the stationary jump-off
    /// — the nudge path detaches without the jump.
    #[test]
    fn grapple_jump_off_down_is_a_nudge() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        // The registered set (`grappling[]`/`grapCount`): one attached Hook.
        let center = Vec2::new(p.x + WIDTH * 0.5, p.y - 4.0);
        let grapple: &[(u16, Vec2, f32)] = &[(crate::projectile::HOOK, center, 0.0)];
        p.update_with_grapple(
            &w,
            PlayerInput {
                jump: true,
                down: true,
                ..Default::default()
            },
            grapple,
            false,
            false,
            false,
            None,
            &mut tr_core::UnifiedRandom::default(),
        );
        assert!(!p.grappling_active, "detached");
        // The nudge path (`velocity.Y += 0.01f`) does NOT grant the jump
        // counter — the pull velocity carries (the vanilla does not zero it
        // in RemoveAllGrapplingHooks).
        assert_eq!(p.jump, 0, "no jump counter on the nudge path");
    }

    /// `Player.cs:22660-22661`: while pulling DOWN the platforms fall
    /// through (`GoingDownWithGrapple`).
    #[test]
    fn downward_pull_sets_going_down() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        // One attached Hook below the player.
        let center = Vec2::new(p.x + WIDTH * 0.5, p.y + 120.0);
        let grapple: &[(u16, Vec2, f32)] = &[(crate::projectile::HOOK, center, 0.0)];
        p.update_with_grapple(&w, PlayerInput::default(), grapple, false, false, false, None, &mut tr_core::UnifiedRandom::default());
        assert!(p.going_down_with_grapple, "the pull points down");
    }

    /// `Player.cs:19963,19993`: the overspeed continuation is gated on
    /// `dashDelay >= 0` — no input acceleration while dashing (the decay
    /// runs instead).
    #[test]
    fn no_input_acceleration_while_dashing() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.equipped[3] = 492; // Demon Wings
        p.fall_start = 40;
        p.vy = 5.0;
        let held = PlayerInput {
            jump: true,
            ..Default::default()
        };
        p.update(&w, held);
        // The glide cap is the wings arm's `maxFallSpeed / 3` (`:27339`) on
        // the rebased `maxFallSpeed` — `:24437`'s every-tick `+0.01f` rides
        // in (`R-03-1`).
        assert!(
            p.vy <= (MAX_FALL + 0.01) / 3.0 + 1e-4,
            "the glide cap: {}",
            p.vy
        );
        assert_eq!(p.fall_start, (p.y / TILE_PX) as i32, "the origin refreshes");
    }

    /// `Player.cs:25360`: `!hasWings` gates the fall damage — any equipped
    /// wings nullify it (PHYS-2's field, armed here).
    #[test]
    fn wings_nullify_the_fall_damage() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.equipped[3] = 492; // Demon Wings
        p.y = 25.0 * TILE_PX;
        p.fall_start = 25;
        for _ in 0..300 {
            p.update(&w, PlayerInput::default());
            if p.on_ground {
                break;
            }
        }
        assert!(p.on_ground && p.hp == 100, "no fall damage with wings");
    }

    /// `Player.cs:27404-27425`: the hover wings holding down dampen vy
    /// toward the float.
    #[test]
    fn hover_wings_dampen_the_descent() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        // Bee Wings (1866 → `ArmorIDs.Wing` 22, the `HasDownHoverStats` row).
        p.equipped[3] = 1866;
        p.wing_time = 170.0;
        p.vy = 3.0;
        let held = PlayerInput {
            jump: true,
            down: true,
            ..Default::default()
        };
        p.update(&w, held);
        assert!(
            p.vy.abs() < 3.0 && p.vy >= 0.0,
            "the descent dampened: vy={}",
            p.vy
        );
    }

    /// Places `kind` in the accessory slot 0 (= `armor[3]`), runs the tick's
    /// reset + grant pass by calling `update`, then reports the flags.
    fn player_with_accessory(kind: u16) -> Player {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.equipped[3] = kind;
        for _ in 0..2 {
            p.update(&w, PlayerInput::default());
        }
        p
    }

    #[test]
    fn the_accessory_grants_arm_the_jumps() {
        // `ApplyEquipFunctional`'s bottle grants (`Player.cs:14997-15001`,
        // :14793-14811): the bottles arm their jump options; the reset clears
        // and the pass re-arms every tick.
        let p = player_with_accessory(53);
        assert!(p.has_jump_options[ExtraJump::Cloud.index()]);
        assert!(!p.has_jump_options[ExtraJump::Sandstorm.index()]);
        let p = player_with_accessory(857);
        assert!(p.has_jump_options[ExtraJump::Sandstorm.index()]);
        let p = player_with_accessory(987);
        assert!(p.has_jump_options[ExtraJump::Blizzard.index()]);
        let p = player_with_accessory(1724);
        assert!(p.has_jump_options[ExtraJump::Fart.index()]);
        let p = player_with_accessory(3201);
        assert!(p.has_jump_options[ExtraJump::Sail.index()]);
        // No accessory: nothing armed.
        let p = player_with_accessory(0);
        assert!(p.has_jump_options.iter().all(|&b| !b));
    }

    #[test]
    fn the_hermes_boots_lift_the_acceleration_gate() {
        // `Player.cs:15005-15008`: `accRunSpeed = 6f` — the base reset (3.0)
        // is lifted, the debuff multiplier still applies afterwards.
        let p = player_with_accessory(54);
        assert!(
            (p.acc_run_speed_value() - 6.0).abs() < 1e-4,
            "{}",
            p.acc_run_speed_value()
        );
        // The vanilla leaves `maxRunSpeed` at 3.0 — the boots only widen the
        // acceleration window (the continuation to accRunSpeed).
        assert!((p.max_run_speed_value() - 3.0).abs() < 1e-4);
    }

    #[test]
    fn the_godmode_power_swallows_hits_death_and_refills() {
        // `GodmodePower` -> `Player.creativeGodMode` (`Player.cs:25493-25497`);
        // `Hurt` returns 0 before the iframes (`:38203-38206`) and `KillMe`
        // refuses outright (`:38809-38812`).
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        p.creative_god_mode = true;
        p.hurt_cd = 0.0;
        let hp = p.hp;
        assert!(!p.hurt(50, p.x + 100.0, &mut tr_core::UnifiedRandom::default(), true), "the hit is swallowed");
        assert_eq!(p.hp, hp, "no damage");
        assert_eq!(p.vx, 0.0, "no knockback");
        p.start_death(&w, false, false);
        assert!(!p.dead, "KillMe refuses while the power is on");
        // Switching the power off restores both arms.
        p.creative_god_mode = false;
        assert!(p.hurt(10, p.x + 100.0, &mut tr_core::UnifiedRandom::default(), true));
        p.hp = 0;
        p.start_death(&w, false, false);
        assert!(p.dead);
    }

    #[test]
    fn the_far_placement_power_doubles_the_tile_range() {
        // `Player.ResetEffects` (`Player.cs:18815-18828`): the default
        // `tileRangeX/Y` 5/3 (`:57113-57114`) become `* 2 + 8` = 18/14 while
        // `FarPlacementRangePower` is enabled (`CreativePowers.cs:730-767`).
        let w = ground_world();
        let mut p = Player::spawn(&w);
        assert_eq!(p.tile_range_x(), 5.0);
        assert_eq!(p.tile_range_y(), 3.0);
        p.far_placement_range = true;
        assert_eq!(p.tile_range_x(), 18.0);
        assert_eq!(p.tile_range_y(), 14.0);
    }

    #[test]
    fn the_horseshoe_and_the_shield_grant_their_flags() {
        // `Player.cs:15029-15033` (noFallDmg) + `:15025-15028` (noKnockback).
        let p = player_with_accessory(158);
        assert!(p.no_fall_dmg);
        let p = player_with_accessory(156);
        assert!(p.no_knockback);
        let w = ground_world();

        // The hurt gate: no knockback while the shield is on (the player
        // settles first so the vy assert reads the hurt, not the spawn fall).
        let mut p = player_with_accessory(156);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        let from = p.x + 100.0;
        assert!(p.hurt_inner(10, from, true, &mut tr_core::UnifiedRandom::default(), true));
        assert_eq!(p.vx, 0.0, "the shield eats the knockback");
        // Without the shield: the knockback lands.
        let mut p = player_with_accessory(0);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        let from = p.x + 100.0;
        assert!(p.hurt_inner(10, from, true, &mut tr_core::UnifiedRandom::default(), true));
        assert!(p.vx != 0.0);
    }

    #[test]
    fn the_balloon_lifts_the_jump_statics() {
        // `UpdateJumpHeight` (`Player.cs:19589-19593`): jumpBoost floors
        // jumpHeight 20 / jumpSpeed 6.51 — the Cloud Balloon's grant
        // (`Player.cs:15093-15101`).
        let p = player_with_accessory(399);
        assert!(p.jump_boost);
        let w = ground_world();
        let mut q = Player::spawn(&w);
        q.update(&w, PlayerInput::default());
        assert!(q.jump_height < p.jump_height || q.jump_speed < p.jump_speed);
    }

    #[test]
    fn the_ice_skates_and_the_boots_feed_the_floor_modifiers() {
        // `Player.cs:15089-15092` (iceSkate) — the consumer at the port's
        // HorizontalMovement (player.rs's `if self.ice_skate`).
        let p = player_with_accessory(950);
        assert!(p.ice_skate);
        let p = player_with_accessory(54);
        assert!((p.acc_run_speed_value() - 6.0).abs() < 1e-4);
    }

    #[test]
    fn the_rocket_boots_tier_applies() {
        // `Player.cs:15021-15024`: `rocketBoots = 1`.
        let p = player_with_accessory(128);
        assert_eq!(p.rocket_boots, 1);
    }

    #[test]
    fn the_band_feeds_the_regen_base() {
        // `Player.cs:13122` (`lifeRegen += armorPiece.lifeRegen`, the Band's
        // `Item.lifeRegen = 2` — `Item.cs:1974`): the regen input carries it.
        let p = player_with_accessory(49);
        assert_eq!(p.accessory_life_regen_value(), 2);
    }

    #[test]
    fn the_pins_survive_the_tick_clear() {
        // The TR_JUMPS/TR_BOOTS pins stand in for the granting equipment: the
        // pass clears the flags and re-applies the pins (the vanilla: the
        // grants re-set per tick after ResetEffects).
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.pinned_jumps[ExtraJump::Cloud.index()] = true;
        p.pinned_rocket_boots = 3;
        p.update(&w, PlayerInput::default());
        assert!(p.has_jump_options[ExtraJump::Cloud.index()]);
        assert_eq!(p.rocket_boots, 3);
        assert!(!p.has_jump_options[ExtraJump::Sandstorm.index()]);
    }

    /// DW-3.3's field half — the ten ammo-mastery flags a fresh `Player` carries
    /// (`player.rs`'s ammo block) plus the cycling struct: every one defaults
    /// false/None/0/0 like the C#'s `ResetEffects`-cleared fields
    /// (`Player.cs:18390-18664`, `:18479`), so the rolls stay inert until the
    /// per-tick grant seam in `world_sim/player.rs` derives them from the
    /// equipment and the buffs. The `ammo_flags()` projection is the single seam
    /// `ammo::not_consume` / `ammo::resolve_projectile` read.
    #[test]
    fn the_ammo_fields_default_to_false_and_none() {
        let w = crate::test_world_helpers::ground();
        let p = Player::spawn(&w);
        // The four fuse flags (Phase 1's fields).
        assert!(!p.has_molten_quiver && !p.magic_quiver);
        assert!(!p.acc_sharp_barb && !p.archery);
        // The six roll flags Phase 3 appends.
        assert!(!p.ammo_box && !p.ammo_potion && !p.huntress_ammo_cost90);
        assert!(!p.chloro_ammo_cost80 && !p.ammo_cost80 && !p.ammo_cost75);
        // The cycling state: None/0/0.
        assert_eq!(p.ammo_cycling.mode, crate::ammo::AmmoCyclingMode::None);
        assert_eq!((p.ammo_cycling.offset, p.ammo_cycling.cooldown), (0, 0));
        // The projection round-trips: an all-false Player gives an all-false
        // AmmoFlags, so the chain is the plain vanilla one.
        let flags = p.ammo_flags();
        assert_eq!(flags, crate::ammo::AmmoFlags::default());
        // A granted flag reaches the struct the rolls read.
        let mut granted = Player::spawn(&w);
        granted.ammo_box = true;
        granted.magic_quiver = true;
        assert!(granted.ammo_flags().ammo_box && granted.ammo_flags().magic_quiver);
    }

    /// Spawns on flat ground and settles (the first update is the spawn tick).
    fn settled() -> (World, Player) {
        let w = crate::test_world_helpers::ground();
        let mut p = Player::spawn(&w);
        p.update(&w, PlayerInput::default());
        (w, p)
    }

    #[test]
    fn the_leg_rows_cover_the_missing_playerframe_arms() {
        // `Player.cs:36219-36291`: the leg chain. Each arm is driven from the
        // state the C# reads, in the vanilla order.
        let (w, mut p) = settled();
        // The swim paddle (`:36221-36233`): a flat 2 per tick regardless of
        // velocity, cycling rows 7..=19.
        p.swim_time = 10;
        p.wet = true;
        p.vy = 0.0;
        p.leg_frame = 7;
        p.leg_frame_counter = 0.0;
        // `while (counter > 8.0)` is strict, so the 8.0 sum needs a fifth tick.
        for _ in 0..6 {
            p.update_frames(&w);
        }
        assert_eq!(p.leg_frame, 8, "2 per tick crosses 8.0 on the fifth tick");
        // Out of water the window dies (`:35759-35761`).
        p.wet = false;
        p.update_frames(&w);
        assert_eq!(p.swim_time, 0);
        // The hover set pins row 5 (`:36234-36240`): the grapple holds the
        // legs at the fall row even at rest.
        let (w, mut p) = settled();
        p.grappling_active = true;
        p.update_frames(&w);
        assert_eq!(p.leg_frame, 5, "the hooks hold the fall row");
        // The rest row (`:36276-36282`).
        p.grappling_active = false;
        p.vy = 0.0;
        p.update_frames(&w);
        assert_eq!(p.leg_frame, 0);
        // The slippery no-input arm (`:36244-36250`): on ice with no steering
        // the legs rest even while sliding.
        p.vy = 0.0;
        p.vx = 6.0;
        p.slippy = true;
        p.control_left = false;
        p.control_right = false;
        p.update_frames(&w);
        assert_eq!(p.leg_frame, 0, "the ice slide pins the rest row");
        // The walk cycle (`:36251-36275`) once the player steers.
        p.control_right = true;
        p.leg_frame = 7;
        p.leg_frame_counter = 0.0;
        for _ in 0..2 {
            p.update_frames(&w);
        }
        assert!(p.leg_frame > 7, "the counter advanced the row");
    }

    #[test]
    fn the_moon_lord_legs_overrun_the_body_rows() {
        // `Player.cs:36634-36644`: the 140 legs re-derive their row from the
        // body row's headgear offset after the body chain.
        let (w, mut p) = settled();
        p.legs_armor = 140;
        assert_eq!(offsets_player_headgear_y(7), 0.0);
        // Body row 7-9 has a zero headgear offset -> the leg row is pinned 7.
        p.vy = 0.0;
        p.vx = 4.0;
        p.leg_frame = 9;
        p.body_frame = 9;
        p.update_frames(&w);
        assert_eq!(p.body_frame, 9, "the legs-140 walk keeps its own rows");
        assert_eq!(p.leg_frame, 7, "the zero offset pins the leg row");
        // Airborne the leg row follows `velocity.Y != 0` -> 5.
        p.vy = 3.0;
        p.update_frames(&w);
        assert_eq!(p.leg_frame, 5);
        // The always-animated wings (22/28) still force row 0.
        p.wings_logic = 22;
        p.update_frames(&w);
        assert_eq!(p.leg_frame, 0);
    }

    #[test]
    fn the_body_chain_order_matches_playerframe() {
        // `Player.cs:36445-36633`: the body rows after the item-use arm, in the
        // vanilla `else if` order.
        let (w, mut p) = settled();
        p.vx = 0.0;
        // The rope pose (`:36445-36454`).
        p.pulley = true;
        p.pulley_dir = 2;
        p.update_frames(&w);
        assert_eq!(p.body_frame, 1);
        p.pulley_dir = 1;
        p.update_frames(&w);
        assert_eq!(p.body_frame, 2);
        p.pulley = false;
        // holdStyle 1 (the torch) -> row 3 (`:36455-36459`); it beats the walk
        // row because the arm comes first.
        p.frame_inputs = FrameInputs {
            hold_style: 1,
            can_visually_hold: true,
            hold_style_dry: true,
            torch_hold_ok: true,
            use_style_blocks_frames: true,
            ..FrameInputs::default()
        };
        p.vx = 3.0;
        p.update_frames(&w);
        assert_eq!(p.body_frame, 3, "the torch hold wins over the walk row");
        // holdStyle 2 -> row 2 (`:36460-36464`).
        p.frame_inputs.hold_style = 2;
        p.update_frames(&w);
        assert_eq!(p.body_frame, 2);
        // holdStyle 7 -> row 11 (`:36477-36481`).
        p.frame_inputs.hold_style = 7;
        p.update_frames(&w);
        assert_eq!(p.body_frame, 11);
        // The umbrella (4) only rests on the ground.
        p.frame_inputs.hold_style = 4;
        p.vy = 0.0;
        p.update_frames(&w);
        assert_eq!(p.body_frame, 0);
        // The raised shield -> row 10 (`:36489-36492`).
        p.frame_inputs = FrameInputs::default();
        p.shield_raised = true;
        p.update_frames(&w);
        assert_eq!(p.body_frame, 10);
        p.shield_raised = false;
        // With nothing held the walk row copies the legs (`:36619-36622`).
        p.vy = 0.0;
        p.leg_frame = 12;
        p.vx = 3.0;
        p.update_frames(&w);
        assert_eq!(p.body_frame, p.leg_frame);
        assert!(p.leg_frame >= 12, "the walk advanced the leg row");
        assert!(p.body_frame_counter > 0.0, "the 1.5x counter accumulated");
        // At rest the body rests (`:36624-36633`).
        p.vx = 0.0;
        p.update_frames(&w);
        assert_eq!(p.body_frame, 0);
    }

    #[test]
    fn the_airborne_body_rows_follow_the_wings() {
        // `Player.cs:36559-36600`: the airborne rows per wing state.
        let (_, mut p) = settled();
        p.vy = 2.0;
        p.wings_logic = 0;
        p.update_frames(&World::new("t".into(), 8, 8));
        assert_eq!(p.body_frame, 5, "no wings: the plain fall row");
        p.wings_logic = 4;
        p.control_jump = true;
        p.update_frames(&World::new("t".into(), 8, 8));
        assert_eq!(p.body_frame, 6, "rising under thrust: row 6");
        p.vy = -2.0;
        p.update_frames(&World::new("t".into(), 8, 8));
        assert_eq!(p.body_frame, 6, "rising: row 6");
        p.wings_logic = 22;
        p.update_frames(&World::new("t".into(), 8, 8));
        assert_eq!(p.body_frame, 0, "the always-animated set rests in the air");
    }

    #[test]
    fn the_use_style_body_dispatch_matches_the_playerframe_chain() {
        // `Player.cs:36307-36444`: the body rows per useStyle. `remaining` is
        // `itemAnimation / itemAnimationMax` (1 at the start, 0 at the end).
        let w = crate::test_world_helpers::ground();
        let mut p = Player::spawn(&w);
        // Style 1 (Swing): the thirds 1 -> 2 -> 3.
        p.apply_item_use_frame(0.9, 1, 0.0);
        assert_eq!(p.body_frame, 1, "style 1 first third");
        p.apply_item_use_frame(0.5, 1, 0.0);
        assert_eq!(p.body_frame, 2, "style 1 middle third");
        p.apply_item_use_frame(0.1, 1, 0.0);
        assert_eq!(p.body_frame, 3, "style 1 last third");
        // Style 7 (DrinkOld): the first half row 3, then 2.
        p.apply_item_use_frame(0.75, 7, 0.0);
        assert_eq!(p.body_frame, 3, "style 7 first half");
        p.apply_item_use_frame(0.25, 7, 0.0);
        assert_eq!(p.body_frame, 2, "style 7 second half");
        // Style 2 (EatFood): row 3 through the use.
        p.apply_item_use_frame(0.5, 2, 0.0);
        assert_eq!(p.body_frame, 3);
        // Style 11 (MowTheLawn): >half 4, >15% 3, else 2.
        p.apply_item_use_frame(0.75, 11, 0.0);
        assert_eq!(p.body_frame, 4, "style 11 first half");
        p.apply_item_use_frame(0.3, 11, 0.0);
        assert_eq!(p.body_frame, 3, "style 11 middle");
        p.apply_item_use_frame(0.05, 11, 0.0);
        assert_eq!(p.body_frame, 2, "style 11 tail");
        // Style 9 (DrinkLiquid): row 0.
        p.apply_item_use_frame(0.5, 9, 0.0);
        assert_eq!(p.body_frame, 0);
        // Style 6 (DrinkLong): `num20 = progress * 6` clamped at 1 - the
        // row flips to 3 once num20 passes 0.5 (progress > 1/12).
        p.apply_item_use_frame(0.99, 6, 0.0);
        assert_eq!(p.body_frame, 2, "style 6 start - num20 = 0.06");
        p.apply_item_use_frame(0.9, 6, 0.0);
        assert_eq!(p.body_frame, 3, "style 6 mid - num20 = 0.6");
        p.apply_item_use_frame(0.05, 6, 0.0);
        assert_eq!(p.body_frame, 3, "style 6 late - num20 clamps at 1");
        // Style 3 (Thrust): BOTH branches row 3 - the vanilla quirk.
        p.apply_item_use_frame(0.9, 3, 0.0);
        assert_eq!(p.body_frame, 3, "style 3 first third");
        p.apply_item_use_frame(0.1, 3, 0.0);
        assert_eq!(p.body_frame, 3, "style 3 last third");
        // Style 4 (HoldUp): row 2 through the use.
        p.apply_item_use_frame(0.5, 4, 0.0);
        assert_eq!(p.body_frame, 2);
        // Style 8 (GolfPlay): row 0. Style 12 (Guitar): row 3.
        p.apply_item_use_frame(0.5, 8, 0.0);
        assert_eq!(p.body_frame, 0);
        p.apply_item_use_frame(0.5, 12, 0.0);
        assert_eq!(p.body_frame, 3);
        // Style 13 (Rapier): the thirds mirror style 1.
        p.apply_item_use_frame(0.9, 13, 0.0);
        assert_eq!(p.body_frame, 1);
        p.apply_item_use_frame(0.1, 13, 0.0);
        assert_eq!(p.body_frame, 3);
        // Style 5 (Shoot): the aim-relative rows - the default 3, past
        // -0.75 row 2, past 0.6 row 4.
        p.apply_item_use_frame(0.5, 5, 0.0);
        assert_eq!(p.body_frame, 3, "style 5 aim centred");
        p.apply_item_use_frame(0.5, 5, -0.9);
        assert_eq!(p.body_frame, 2, "style 5 aim high");
        p.apply_item_use_frame(0.5, 5, 0.9);
        assert_eq!(p.body_frame, 4, "style 5 aim low");
    }

    #[test]
    fn the_use_style_carries_from_the_held_item() {
        // `item::use_style` reads the ItemDef's `useStyle`.
        assert_eq!(crate::item::use_style(36), 1, "the wooden sword swings");
        assert_eq!(crate::item::use_style(31), 1, "the bottle places");
        assert_eq!(crate::item::use_style(0), 0, "the empty hand");
    }

    /// Arms the Cloud + Sandstorm options on a settled player — through the
    /// pins (the grant pass clears the direct sets every tick; the pins are
    /// the stand-in for the granting equipment).
    fn settled_with_jumps(w: &World, opts: &[ExtraJump]) -> Player {
        let mut p = Player::spawn(w);
        for j in opts {
            p.pinned_jumps[j.index()] = true;
        }
        for _ in 0..120 {
            p.update(w, PlayerInput::default());
        }
        for j in opts {
            p.can_jump_again[j.index()] = true;
        }
        p
    }

    /// Fires a jump edge (press + release + gap) while airborne.
    fn tap_air(w: &World, p: &mut Player) {
        p.update(
            w,
            PlayerInput {
                jump: true,
                ..Default::default()
            },
        );
        p.update(w, PlayerInput::default());
        for _ in 0..3 {
            p.update(w, PlayerInput::default());
        }
        p.update(
            w,
            PlayerInput {
                jump: true,
                ..Default::default()
            },
        );
    }

    /// `Player.cs:20698-20748`: the priority chain — with Sandstorm + Cloud
    /// armed, the Sandstorm consumes first (hold-boost 3x).
    #[test]
    fn sandstorm_beats_cloud_in_the_chain() {
        let w = ground_world();
        let mut p = settled_with_jumps(&w, &[ExtraJump::Sandstorm, ExtraJump::Cloud]);
        // Jump off the ground; the ground jump re-arms everything.
        p.update(
            &w,
            PlayerInput {
                jump: true,
                ..Default::default()
            },
        );
        assert!(p.can_jump_again[ExtraJump::Sandstorm.index()], "re-armed");
        tap_air(&w, &mut p);
        assert!(
            !p.can_jump_again[ExtraJump::Sandstorm.index()],
            "sandstorm consumed first"
        );
        assert!(
            p.can_jump_again[ExtraJump::Cloud.index()],
            "cloud still armed"
        );
        assert_eq!(p.performing_jump, Some(ExtraJump::Sandstorm));
        // The hold-boost counter: jumpHeight (15) x 3 = 45.
        assert_eq!(p.jump, 45, "sandstorm 3x boost");
        // The next tap: the Cloud fall-through (0.75x = 11).
        tap_air(&w, &mut p);
        assert_eq!(p.performing_jump, Some(ExtraJump::Cloud));
        assert_eq!(p.jump, (15.0 * 0.75) as i32, "cloud 0.75x boost");
    }

    /// `Player.cs:20755-20758`: the ground jump re-arms every granted option.
    #[test]
    fn ground_jump_rearms_the_options() {
        let w = ground_world();
        let mut p = settled_with_jumps(&w, &[ExtraJump::Cloud]);
        // Consume the cloud.
        tap_air(&w, &mut p);
        assert!(!p.can_jump_again[ExtraJump::Cloud.index()]);
        // Land: the re-arm.
        for _ in 0..300 {
            p.update(&w, PlayerInput::default());
            if p.on_ground {
                break;
            }
        }
        p.update(&w, PlayerInput::default());
        assert!(
            p.can_jump_again[ExtraJump::Cloud.index()],
            "re-armed on land"
        );
    }

    /// `Player.cs:26061-26099`: while airborne, an option that vanished
    /// force-clears its armed flag.
    #[test]
    fn stale_options_clear_while_airborne() {
        let w = ground_world();
        let mut p = settled_with_jumps(&w, &[ExtraJump::Cloud]);
        // Airborne.
        p.update(
            &w,
            PlayerInput {
                jump: true,
                ..Default::default()
            },
        );
        p.update(&w, PlayerInput::default());
        assert!(p.vy != 0.0 || !p.on_ground);
        // The accessory vanishes mid-air (`UpdateEquips`' next pass sees no
        // granting equipment — the pin is the stand-in and it clears).
        p.pinned_jumps[ExtraJump::Cloud.index()] = false;
        p.update(&w, PlayerInput::default());
        assert!(
            !p.can_jump_again[ExtraJump::Cloud.index()],
            "the stale option clears"
        );
    }

    /// `Player.cs:21023-21031`: the second airborne jump requires the button
    /// released since the previous one — holding through does not re-fire.
    #[test]
    fn the_second_jump_needs_the_release() {
        let w = ground_world();
        let mut p = settled_with_jumps(&w, &[ExtraJump::Cloud]);
        // The ground jump.
        p.update(
            &w,
            PlayerInput {
                jump: true,
                ..Default::default()
            },
        );
        assert!(p.can_jump_again[ExtraJump::Cloud.index()]);
        // Hold through: no consumption (releaseJump stays false).
        let held = PlayerInput {
            jump: true,
            ..Default::default()
        };
        for _ in 0..20 {
            p.update(&w, held);
        }
        assert!(
            p.can_jump_again[ExtraJump::Cloud.index()],
            "holding does not consume"
        );
        assert!(p.jump > 0 || p.vy < 0.0, "still in the first jump");
    }

    /// `Player.cs:19589-19593`: the balloon floor lifts the jump statics —
    /// the ground jump height 20 instead of 15.
    #[test]
    fn balloon_lifts_the_jump_height() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        p.jump_boost = true;
        let held = PlayerInput {
            jump: true,
            ..Default::default()
        };
        p.update(&w, held);
        assert_eq!(p.jump, 20, "jumpHeight floor 20 with jumpBoost");
    }

    /// `Player.cs:19963,19993`: the overspeed continuation is gated on
    /// `dashDelay >= 0` — no input acceleration while dashing (the decay
    /// runs instead).
    #[test]
    fn dash_overspeed_continuation_gated() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.dash_type = 2;
        p.dash = 2;
        p.dash_delay = -1;
        p.vx = 14.5;
        let held = PlayerInput {
            right: true,
            ..Default::default()
        };
        p.update(&w, held);
        assert!(p.vx > 14.0, "decay, not acceleration: vx={}", p.vx);
    }

    /// `Player.cs:23755-23783`: `GetFloorTile` samples the center column
    /// first, then the neighbors — the flags come from the tile under the
    /// feet (one tick stale, so a settled player has live flags).
    #[test]
    fn floor_flags_sample_the_feet_row() {
        let w = floor_world(161);
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        assert!(p.on_ground, "must settle first");
        assert!(p.slippy, "ice floor flags the slippery set");
        let mut q = Player::spawn(&ground_world());
        for _ in 0..120 {
            q.update(&ground_world(), PlayerInput::default());
        }
        q.update(&ground_world(), PlayerInput::default());
        assert!(!q.slippy, "dirt is not slippery");
    }

    /// `Player.cs:25363`: `(fell - (25 + extraFall)) * 10`, `fell` truncated on
    /// both ends. Dropping from tile 25 lands at row 57 → fell = 32 → 70.
    #[test]
    fn fall_damage_matches_the_vanilla_formula() {
        let w = ground_world();
        let (lost, landed) = drop_and_settle(&w, 25);
        assert_eq!(landed, 57, "feet rest on row 57");
        assert_eq!(lost, (landed - 25 - 25) * 10, "fell = 57 - 25 = 32 tiles");
    }

    /// `Player.cs:25360`: `num11 > num10` is strict — a 25-tile fall hurts for
    /// nothing; the first damaging tile is 26.
    #[test]
    fn the_safe_fall_threshold_is_strict() {
        let w = ground_world();
        // From tile 32 the feet land at row 57 → fell 25 → no damage.
        let (lost, _) = drop_and_settle(&w, 32);
        assert_eq!(lost, 0, "25-tile fall is free");
    }

    /// `Player.cs:25361`: the normal branch forces `immune = false` BEFORE the
    /// Hurt — a fall lands through any iframes the contact damage granted.
    #[test]
    fn fall_damage_ignores_iframes() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.hurt_cd = 5.0; // mid-iframes
        p.y = 25.0 * TILE_PX;
        p.fall_start = 25;
        for _ in 0..600 {
            p.update(&w, PlayerInput::default());
            if p.on_ground {
                break;
            }
        }
        assert!(p.hp < 100, "fall must land through iframes");
    }

    /// The liquid reset (`Player.cs:25379`): while `wet` the fall origin rides
    /// the player, so a fall through water lands for nothing.
    #[test]
    fn water_forgives_the_fall() {
        let mut w = ground_world();
        for y in 50..58 {
            for x in 0..64 {
                let i = w.index(x, y);
                w.tiles[i].liquid = 200;
            }
        }
        let (lost, _) = drop_and_settle(&w, 20);
        assert_eq!(lost, 0, "a fall through water is free");
    }

    /// `Player.cs:25338-25345`: a landing on the cloud set (tile 189 here)
    /// zeroes the fall distance.
    #[test]
    fn clouds_forgive_the_fall() {
        let mut w = ground_world();
        for x in 0..64 {
            let i = w.index(x, 60);
            w.tiles[i].kind = 189;
        }
        let (lost, _) = drop_and_settle(&w, 25);
        assert_eq!(lost, 0, "a landing on clouds is free");
    }

    /// `Player.cs:25347-25359`: petrified replaces the whole formula — a fixed
    /// 2-tile grace at 20 damage per tile, ignoring extraFall.
    #[test]
    fn stoned_landing_uses_the_petrified_formula() {
        let mut w = ground_world();
        // Clouds out of the picture: the plain dirt surface again.
        let mut p = Player::spawn(&w);
        p.max_hp = 1000;
        p.hp = 1000;
        p.buffs.add_buff(156, 600); // Petrified
        p.y = 25.0 * TILE_PX;
        p.fall_start = 25;
        for _ in 0..600 {
            p.update(&w, PlayerInput::default());
            if p.on_ground {
                break;
            }
        }
        let landed = (p.y / TILE_PX) as i32;
        // (fell - 2) * 20 = (32 - 2) * 20 = 600, minus the flat petrify hit of
        // 20 taken the moment the buff appeared.
        assert_eq!(p.hp, 1000 - (landed - 25 - 2) * 20 - 20, "petrified hp");
    }

    /// `Player.cs:25370`: fall damage calls Hurt with hitDirection 0 — no
    /// knockback, the player stays planted where they landed.
    #[test]
    fn fall_landing_has_no_knockback() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.y = 25.0 * TILE_PX;
        p.fall_start = 25;
        for _ in 0..600 {
            p.update(&w, PlayerInput::default());
            if p.on_ground {
                break;
            }
        }
        assert!(p.on_ground, "must rest after landing");
        assert_eq!(p.vy, 0.0, "no upward knock");
    }

    #[test]
    fn idle_uses_the_rest_pose_row_zero() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        // `PlayerFrame`: standing still is leg/body frame row 0, never the jump
        // pose (row 5) that made the character look like it was about to leap.
        assert_eq!(p.leg_frame, 0, "idle legs must be row 0");
        assert_eq!(p.body_frame, 0, "idle body must be row 0");
    }

    #[test]
    fn airborne_uses_row_five() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        p.update(
            &w,
            PlayerInput {
                jump: true,
                ..Default::default()
            },
        );
        for _ in 0..8 {
            p.update(&w, PlayerInput::default());
        }
        assert!(p.vy != 0.0, "should be airborne, vy={}", p.vy);
        assert_eq!(p.leg_frame, 5, "airborne legs must be row 5");
        assert_eq!(p.body_frame, 5, "airborne body must be row 5");
    }

    #[test]
    fn walking_cycles_the_walk_rows() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        let mut seen = std::collections::BTreeSet::new();
        for _ in 0..240 {
            p.update(
                &w,
                PlayerInput {
                    right: true,
                    ..Default::default()
                },
            );
            assert!(
                (7..=19).contains(&p.leg_frame),
                "walk row {} outside 7..=19",
                p.leg_frame
            );
            assert_eq!(p.body_frame, p.leg_frame, "body must copy the leg row");
            seen.insert(p.leg_frame);
        }
        assert!(seen.len() >= 4, "walk cycle did not advance: {seen:?}");
    }

    #[test]
    fn walks_and_is_blocked_by_walls() {
        let mut w = ground_world();
        for y in 50..60 {
            let i = w.index(20, y);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        let mut p = Player::spawn(&w);
        for _ in 0..60 {
            p.update(&w, PlayerInput::default());
        }
        for _ in 0..240 {
            p.update(
                &w,
                PlayerInput {
                    right: true,
                    ..Default::default()
                },
            );
        }
        assert!(p.x < 20.0 * TILE_PX, "player passed the wall: x={}", p.x);
    }

    #[test]
    fn jump_leaves_the_ground() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        let grounded_y = p.y;
        p.update(
            &w,
            PlayerInput {
                jump: true,
                ..Default::default()
            },
        );
        for _ in 0..10 {
            p.update(&w, PlayerInput::default());
        }
        assert!(p.y < grounded_y - 4.0, "jump did not rise: {}", p.y);
    }

    #[test]
    fn walks_down_a_slope_smoothly() {
        let mut w = World::new("t".into(), 64, 128);
        for x in 0..64 {
            let top = if x < 20 { 60 } else { 61 };
            for y in top..100 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        let i = w.index(19, 60);
        w.tiles[i].set_slope(1);
        w.spawn_x = 10;
        w.spawn_y = 58;
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        let mut last = p.y + p.gfx_off_y;
        let mut max_step: f32 = 0.0;
        for _ in 0..180 {
            p.update(
                &w,
                PlayerInput {
                    right: true,
                    ..Default::default()
                },
            );
            let vis = p.y + p.gfx_off_y;
            max_step = max_step.max((vis - last).abs());
            last = vis;
        }
        assert!(p.y > 56.0 * TILE_PX, "should have descended, y={}", p.y);
        assert!(max_step < 8.0, "slope caused a {max_step} px visual jump");
    }

    #[test]
    fn walks_up_a_one_tile_step_smoothly() {
        let mut w = ground_world();
        // A one-tile step at x=20: `Collision.StepUp` lifts the hitbox while
        // `gfxOffY` keeps the sprite at the old height, so the climb is smooth.
        for x in 20..30 {
            let i = w.index(x, 59);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        let mut last = p.y + p.gfx_off_y;
        let mut max_step: f32 = 0.0;
        let mut stood_on_step = false;
        for _ in 0..240 {
            p.update(
                &w,
                PlayerInput {
                    right: true,
                    ..Default::default()
                },
            );
            let vis = p.y + p.gfx_off_y;
            max_step = max_step.max((vis - last).abs());
            last = vis;
            if p.x + WIDTH > 20.0 * TILE_PX && p.x < 30.0 * TILE_PX {
                stood_on_step |= (p.y + HEIGHT - 59.0 * TILE_PX).abs() < 1.0;
            }
        }
        assert!(
            p.x + WIDTH > 21.0 * TILE_PX,
            "player never climbed the step: x={}",
            p.x
        );
        assert!(stood_on_step, "feet never rested on the step top");
        // The original eases `gfxOffY` at `(1 + |vx| / 3) * stepSpeed` px per tick
        // (5 px/tick at walk speed with stepSpeed 2.5); what must never happen is
        // the instant 16 px teleport of a naive snap.
        assert!(max_step < 8.0, "step caused a {max_step} px visual jump");
    }

    #[test]
    fn holding_jump_rises_higher_than_tapping() {
        let w = ground_world();
        let mut tap = Player::spawn(&w);
        for _ in 0..120 {
            tap.update(&w, PlayerInput::default());
        }
        let mut tap_peak = tap.y;
        tap.update(
            &w,
            PlayerInput {
                jump: true,
                ..Default::default()
            },
        );
        for _ in 0..60 {
            tap.update(&w, PlayerInput::default());
            tap_peak = tap_peak.min(tap.y);
        }

        let mut hold = Player::spawn(&w);
        for _ in 0..120 {
            hold.update(&w, PlayerInput::default());
        }
        let mut hold_peak = hold.y;
        for _ in 0..30 {
            hold.update(
                &w,
                PlayerInput {
                    jump: true,
                    ..Default::default()
                },
            );
            hold_peak = hold_peak.min(hold.y);
        }
        for _ in 0..60 {
            hold.update(&w, PlayerInput::default());
            hold_peak = hold_peak.min(hold.y);
        }
        assert!(
            hold_peak + 20.0 < tap_peak,
            "hold peak {hold_peak} vs tap peak {tap_peak}"
        );
    }

    #[test]
    fn long_falls_hurt_but_short_ones_do_not() {
        let mut w = ground_world();
        w.spawn_y = 30;
        let mut p = Player::spawn(&w);
        for _ in 0..300 {
            p.update(&w, PlayerInput::default());
        }
        // The hitbox top lands at 60 - 42/16 = 57.4 tiles, so the drop is ~27:
        // 2.4 tiles over the safe distance.
        assert!(p.on_ground);
        assert!(p.hp < 90 && p.hp > 0, "hp {}", p.hp);

        let mut w = ground_world();
        w.spawn_y = 50;
        let mut p = Player::spawn(&w);
        for _ in 0..200 {
            p.update(&w, PlayerInput::default());
        }
        assert!(p.on_ground);
        assert_eq!(p.hp, 100, "a 10-tile fall must not hurt");
    }

    #[test]
    fn drowning_drains_breath_and_damages() {
        let mut w = ground_world();
        for x in 0..64 {
            // A pool from y=40 down to the ground, so the player settles on the floor.
            for y in 40..60 {
                let i = w.index(x, y);
                w.tiles[i].liquid = 255;
            }
        }
        w.spawn_y = 44;
        let mut p = Player::spawn(&w);
        for _ in 0..1500 {
            p.update(&w, PlayerInput::default());
        }
        assert!(p.in_water);
        assert_eq!(p.breath, 0, "breath {}", p.breath);
        assert!(p.hp < 100, "drowning should hurt, hp {}", p.hp);
    }

    #[test]
    fn health_regenerates_after_a_delay() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        p.hp = 50;
        p.regen.life_regen_time = 0.0;
        for _ in 0..1200 {
            p.update(&w, PlayerInput::default());
        }
        // The faithful engine at `statLifeMax2 = 100` is slow: the ramp scale is
        // `100/400·0.85 + 0.15 = 0.3625`, so tiers 1-2 round banker's-style to
        // 0 and the first points arrive from tick ~600 on (≈ +4 hp over 20 s
        // standing). The old placeholder granted 1 hp per half-second — the
        // `> 60` bar belonged to it.
        assert!(p.hp > 50, "regen only reached {} (sit still, 20 s)", p.hp);
        assert!(p.hp <= 56, "regen overshot the faithful pace: {}", p.hp);
    }

    #[test]
    fn stands_on_half_bricks_at_their_own_height() {
        let mut w = ground_world();
        for x in 0..64 {
            let i = w.index(x, 60);
            w.tiles[i] = Tile {
                kind: 1,
                flags: FLAG_ACTIVE | tr_core::tile::FLAG_HALF_BRICK,
                ..Default::default()
            };
        }
        let mut p = Player::spawn(&w);
        for _ in 0..200 {
            p.update(&w, PlayerInput::default());
        }
        assert!(p.on_ground);
        let expected = 60.0 * TILE_PX + 8.0 - HEIGHT;
        assert!(
            (p.y - expected).abs() < 1.0,
            "y {} expected {expected}",
            p.y
        );
    }

    #[test]
    fn half_bricks_only_block_their_bottom_half() {
        let mut w = ground_world();
        let i = w.index(20, 50);
        w.tiles[i] = Tile {
            kind: 1,
            flags: FLAG_ACTIVE | tr_core::tile::FLAG_HALF_BRICK,
            ..Default::default()
        };
        assert!(collides_player_rect(
            &w,
            20.0 * TILE_PX,
            50.0 * TILE_PX + 8.0
        ));
        assert!(!collides_player_rect(
            &w,
            20.0 * TILE_PX,
            50.0 * TILE_PX + 8.0 - HEIGHT
        ));
    }

    #[test]
    fn ceiling_slopes_block_by_their_triangle() {
        let mut w = ground_world();
        let i = w.index(20, 50);
        w.tiles[i] = Tile {
            kind: 1,
            flags: FLAG_ACTIVE,
            ..Default::default()
        };
        w.tiles[i].set_slope(3);
        // Slope 3 is solid above the diagonal from (0, 16) to (16, 0).
        assert!(collides_player_rect(
            &w,
            20.0 * TILE_PX + 1.0,
            50.0 * TILE_PX
        ));
        assert!(!collides_player_rect(
            &w,
            20.0 * TILE_PX + 1.0,
            51.0 * TILE_PX
        ));
    }

    #[test]
    fn standing_on_a_ground_slope() {
        let mut w = World::new("t".into(), 32, 64);
        for x in 0..32 {
            for y in 30..64 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        let i = w.index(10, 29);
        w.tiles[i] = Tile {
            kind: 1,
            ..Default::default()
        };
        w.tiles[i].set_slope(1);
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.spawn_x = 8;
        w.spawn_y = 20;
        w.compute_column_surface();
        w.compute_chunks();

        let mut p = Player::spawn(&w);
        // Left edge six pixels into the ramp: `SlopeCollision` rests slope 1 on
        // the diagonal at the hitbox's left edge.
        p.x = 10.0 * TILE_PX + 6.0;
        p.y = 29.0 * TILE_PX - HEIGHT;
        for _ in 0..30 {
            p.update(&w, PlayerInput::default());
        }
        let feet = p.y + HEIGHT;
        let expected = 29.0 * TILE_PX + (p.x - 10.0 * TILE_PX);
        assert!(
            (feet - expected).abs() <= 1.5,
            "feet {feet} expected {expected}"
        );
        assert!(p.on_ground);
    }

    // — Phase 2 (M2-BUFF-3): DoT derivation, water/lava/honey treatment —

    /// A player with a large life pool for 300-tick drain runs; `update_survival`
    /// only (the buff timers are not decremented there, so the buff holds).
    fn drain_rig() -> Player {
        drain_rig_in(&World::new("t".into(), 64, 800))
    }

    /// The world-parameterized twin (M5-SEED-2's vampirism rigs pass a world
    /// whose micro seeds carry `Main.vampireSeed`'s bit).
    fn drain_rig_in(world: &World) -> Player {
        let mut p = Player::spawn(world);
        p.max_hp = 1000;
        p.hp = 1000;
        p
    }

    // — M2-BUFF-6 (phase 1): the luck value + the buff-driven inputs —

    #[test]
    fn buff_257_tiers_follow_the_remaining_time() {
        // DW-1.3 — `Player.cs:10285-10301`: `>36000 → 3, >18000 → 2, else 1`,
        // read after `UpdateBuffs`' timer head decremented the slot.
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        p.buffs.add_buff(257, 36_002);
        p.update(&World::new("t".into(), 64, 800), PlayerInput::default());
        assert_eq!(p.luck_state.luck_potion, 3);

        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        p.buffs.add_buff(257, 36_001);
        p.update(&World::new("t".into(), 64, 800), PlayerInput::default());
        assert_eq!(p.luck_state.luck_potion, 2, "36000 → not > 36000");

        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        p.buffs.add_buff(257, 18_001);
        p.update(&World::new("t".into(), 64, 800), PlayerInput::default());
        assert_eq!(p.luck_state.luck_potion, 1, "18000 → not > 18000");
    }

    #[test]
    fn buff_383_tier_is_stale_after_expiry_never_zeroed() {
        // DW-1.3 — `Player.cs:9920-9924`: tier = buffTime/10800 clamp 0..3,
        // assigned only while the buff is active; no `kiteLuckLevel = 0`
        // exists in the C#, so the field keeps its last value after expiry.
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        p.buffs.add_buff(383, 32_401);
        p.update(&World::new("t".into(), 64, 800), PlayerInput::default());
        assert_eq!(p.luck_state.kite_luck_level, 3, "32400/10800");

        p.buffs.clear_buff(383);
        p.update(&World::new("t".into(), 64, 800), PlayerInput::default());
        assert_eq!(p.luck_state.kite_luck_level, 3, "stale-until-recomputed");
    }

    #[test]
    fn buff_120_sets_stinky_and_the_reset_clears_it() {
        // DW-1.3 — `Player.cs:10276-10280` sets it, `ResetEffects` `:18732`
        // clears it every frame; RecalculateLuck weights it −0.25.
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        p.buffs.add_buff(120, 600);
        p.update(&World::new("t".into(), 64, 800), PlayerInput::default());
        assert!(p.luck_state.stinky);
        assert!((p.luck() + 0.25).abs() < 1e-6);

        p.buffs.clear_buff(120);
        p.update(&World::new("t".into(), 64, 800), PlayerInput::default());
        assert!(!p.luck_state.stinky);
        assert_eq!(p.luck(), 0.0);
    }

    #[test]
    fn one_tick_order_decay_then_recalculate() {
        // DW-1.4 — `UpdateLuckFactors` (`Player.cs:29121`) drains the ladybug
        // timer before `RecalculateLuck` reads it, in the same tick the buff
        // scan derived the potion tier; `p.luck()` is the consumer seam.
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        p.buffs.add_buff(257, 20_002); // 20_001 after the timer head → tier 2
        p.luck_state.lady_bug_luck_time_left = -10_800;
        p.update(&World::new("t".into(), 64, 800), PlayerInput::default());
        assert_eq!(p.luck_state.luck_potion, 2);
        assert_eq!(p.luck_state.lady_bug_luck_time_left, -10_799, "dayRate 1");
        let want = (10_799.0f32 / -10_800.0) * 0.2 + 2.0f32 * 0.1;
        assert!((p.luck() - want).abs() < 1e-6, "{} vs {want}", p.luck());
    }

    #[test]
    fn add_coin_luck_caps_at_a_million_within_a_thousand_px() {
        // DW-1.1 — `Player.AddCoinLuck` (`Player.cs:18090-18101`): `!dead`,
        // within 1000 px of the center, capped at 1 000 000 copper
        // equivalents (the shimmer feed is the only C# writer; unported).
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        p.add_coin_luck((p.x + 500.0, p.y), 999_000);
        assert_eq!(p.luck_state.coin_luck, 999_000.0);
        p.add_coin_luck((p.x + 500.0, p.y), 1_000_000);
        assert_eq!(p.luck_state.coin_luck, 1_000_000.0, "capped");

        p.luck_state.coin_luck = 100.0;
        p.add_coin_luck((p.x + 1200.0, p.y), 5);
        assert_eq!(p.luck_state.coin_luck, 100.0, "out of range");

        p.dead = true;
        p.add_coin_luck((p.x, p.y), 5);
        assert_eq!(p.luck_state.coin_luck, 100.0, "dead bails");
    }

    #[test]
    fn dot_table_rows_drain_exact_life_regen() {
        // DW-2.1 — the §2.1 table rows as exact `lifeRegen` values, branch order
        // and constants per `Player.cs:18949-19095`; bleeding resets the ramp
        // only (`:19178-19181`). `loss` is the exact hp drop over 300 ticks
        // where the chunk math is closed-form (None = assert "drained").
        let rows: &[(i32, bool, i32, Option<i32>)] = &[
            (20, false, -4, Some(10)),   // Poisoned: −4×300 = −1200 → 10×1 hp
            (70, false, -30, Some(75)),  // Venom: −120 every 4 ticks → 75×1
            (397, false, -30, None),     // Chlorophyte Spore
            (24, false, -8, Some(20)),   // OnFire: −120 every 15 ticks → 20×1
            (323, false, -8, Some(20)),  // OnFire3
            (44, false, -16, None),      // Frostburn
            (324, false, -16, None),     // Frostburn2
            (39, false, -24, Some(60)),  // Cursed Inferno: −120 every 5 ticks
            (67, false, -60, Some(150)), // Burning: fast lane −600/10 ticks → 30×5
            (68, false, -40, Some(100)), // Suffocation: fast lane −600/15 → 20×5
            (144, false, -8, Some(20)),  // Electrified standing
            (38, true, -100, Some(250)), // Tongued expert: fast lane −600/6 → 50×5
            (30, false, 0, Some(0)),     // Bleeding: ramp reset only, no drain
        ];
        for &(id, expert, regen, loss) in rows {
            let mut p = drain_rig();
            p.buffs.expert_mode = expert;
            p.buffs.add_buff(id, 600);
            for _ in 0..300 {
                p.update_survival(&mut tr_core::UnifiedRandom::default());
            }
            assert_eq!(
                p.regen.life_regen, regen,
                "buff {id} must drain exactly {regen}/tick"
            );
            match loss {
                Some(lost) => assert_eq!(1000 - p.hp, lost, "buff {id} total loss"),
                None => assert!(p.hp < 1000, "buff {id} must drain"),
            }
            assert!(p.hp > 0, "buff {id} rig must survive 300 ticks");
        }
    }

    #[test]
    fn water_extinguishes_only_onfire_and_onfire3() {
        // DW-2.2 — `Player.cs:27750-27761`: only 24 and 323 are deleted; the
        // rest of the fire/poison family rides through the water.
        for id in [24, 323] {
            let mut p = drain_rig();
            p.buffs.add_buff(id, 600);
            p.in_water = true;
            p.wet = true;
            p.update_survival(&mut tr_core::UnifiedRandom::default());
            assert!(
                p.buffs.find_buff_index(id).is_none(),
                "water must cure buff {id}"
            );
        }
        for id in [44, 324, 39, 67, 20, 68, 144, 30] {
            let mut p = drain_rig();
            p.buffs.add_buff(id, 600);
            p.in_water = true;
            p.wet = true;
            p.update_survival(&mut tr_core::UnifiedRandom::default());
            assert!(
                p.buffs.find_buff_index(id).is_some(),
                "water must not cure buff {id}"
            );
        }
    }

    #[test]
    fn lava_wet_blocks_the_water_extinguish() {
        // DW-2.2 — `!base.lavaWet` guard (`Player.cs:27751`): standing in lava
        // the water cannot cure OnFire. (The lava hit also re-hurt: hp 70.)
        let mut p = drain_rig();
        p.buffs.add_buff(24, 600);
        p.in_water = true;
        p.wet = true;
        p.in_lava = true;
        p.lava_wet = true;
        p.update_survival(&mut tr_core::UnifiedRandom::default());
        assert!(
            p.buffs.find_buff_index(24).is_some(),
            "lavaWet blocks the cure"
        );
        assert_eq!(p.buffs.buff_time[p.buffs.find_buff_index(24).unwrap()], 600);
    }

    #[test]
    fn honey_does_not_extinguish_fire() {
        // DW-2.2 (Assumption-2 fallback) — the port conditions the cure on the
        // water-only `wet` flag; the C# `WetCollision` flag would cover honey
        // too (Collision.cs:1608-1618) — deviation recorded for
        // docs/ANALYSIS.md (Phase 4).
        let mut p = drain_rig();
        p.buffs.add_buff(24, 600);
        p.in_water = true; // the physics probe covers water | honey
        p.honey_wet = true;
        p.update_survival(&mut tr_core::UnifiedRandom::default());
        assert!(
            p.buffs.find_buff_index(24).is_some(),
            "honey must not cure OnFire"
        );
    }

    #[test]
    fn water_cures_both_fire_buffs_over_two_ticks_despite_the_no_rescan_quirk() {
        // DW-2.2 quirk — the C# extinguish loop does not rescan after `DelBuff`
        // compaction (`Player.cs:27753-27761`): with 24 and 323 both held, one
        // tick deletes the first and skips the shifted-in second.
        let mut p = drain_rig();
        p.buffs.add_buff(24, 600);
        p.buffs.add_buff(323, 600);
        p.in_water = true;
        p.wet = true;
        p.update_survival(&mut tr_core::UnifiedRandom::default());
        let left = p.buffs.active_types();
        assert_eq!(left.len(), 1, "one fire buff survives the shift-skip");
        p.update_survival(&mut tr_core::UnifiedRandom::default());
        assert!(
            p.buffs.active_types().is_empty(),
            "the second falls next tick"
        );
    }

    #[test]
    fn lava_contact_grants_onfire_420() {
        // DW-2.3 — `Player.cs:27710-27711`: a landed lava Hurt grants
        // AddBuff(24, 420); while the iframes run (`num81 == 0`) nothing lands.
        let mut p = drain_rig();
        p.in_lava = true;
        p.update_survival(&mut tr_core::UnifiedRandom::default());
        let idx = p.buffs.find_buff_index(24).expect("OnFire granted on lava");
        assert_eq!(p.buffs.buff_time[idx], 420);
        // `:27705` — the classic lava hit is 80 (`num79`), the 420-tick On
        // Fire rides it.
        assert_eq!(p.hp, 1000 - 80);
        // Inside the iframes the hit (and therefore the grant) repeats nothing.
        let hp = p.hp;
        p.update_survival(&mut tr_core::UnifiedRandom::default());
        assert_eq!(p.hp, hp, "no second lava hit inside the iframes");
        // Expert doubles the buff time through the AddBuff chain
        // (`AddBuff_DetermineBuffTimeToAdd`, 24 is in the difficulty set).
        let mut p = drain_rig();
        p.buffs.expert_mode = true;
        p.in_lava = true;
        p.update_survival(&mut tr_core::UnifiedRandom::default());
        let idx = p.buffs.find_buff_index(24).unwrap();
        assert_eq!(p.buffs.buff_time[idx], 840);
    }

    #[test]
    fn wet_immunity_removes_held_fire_within_one_tick() {
        // DW-2.4 — Wet (103) re-grants `buffImmune[24/323/67]` for the frame
        // (`Player.cs:11740-11751`) and the end-of-update purge deletes the
        // held fire buff (`:26011-26016`): one full update tick clears it.
        // The C# still drains one frame from the flags (the chain runs before
        // the purge): −8 OnFire on the counter.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        p.buffs.add_buff(24, 600);
        p.buffs.add_buff(103, 600);
        p.update(&w, PlayerInput::default());
        assert!(
            p.buffs.find_buff_index(24).is_none(),
            "OnFire purged in 1 tick"
        );
        assert!(
            p.buffs.find_buff_index(103).is_some(),
            "Wet itself survives"
        );
        assert_eq!(p.regen.life_regen_count, -8, "one verbatim drain frame");
    }

    #[test]
    fn immune_purge_skips_the_compacted_fire_buff_for_a_frame() {
        // The purge's no-rescan quirk (`Player.cs:26012-26018`, pinned in
        // buffs.rs): deleting slot 0 shifts the second fire buff into it and
        // the loop never rescans — Burning falls one tick later.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        p.buffs.add_buff(24, 600);
        p.buffs.add_buff(67, 600);
        p.buffs.add_buff(103, 600);
        p.update(&w, PlayerInput::default());
        assert!(p.buffs.find_buff_index(24).is_none());
        assert!(
            p.buffs.active_types().contains(&67),
            "the compacted-in Burning is skipped this frame (still held; \
             find_buff_index hides it behind the live immunity)"
        );
        p.update(&w, PlayerInput::default());
        assert!(
            !p.buffs.active_types().contains(&67),
            "Burning purged the next frame"
        );
        assert!(p.buffs.find_buff_index(103).is_some());
    }

    #[test]
    fn wet_immunity_vanishes_after_wet_expires() {
        // DW-2.4 — the per-frame reset (`Player.cs:25562-25565`) means the
        // immunities live exactly as long as the Wet buff does.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        p.buffs.add_buff(103, 2);
        for _ in 0..3 {
            p.update(&w, PlayerInput::default());
        }
        assert!(p.buffs.find_buff_index(103).is_none(), "Wet expired");
        assert!(!p.buffs.buff_immune[24], "OnFire immunity gone");
        assert!(!p.buffs.buff_immune[323], "OnFire3 immunity gone");
        assert!(!p.buffs.buff_immune[67], "Burning immunity gone");
        // A fresh OnFire now sticks — the immunity is really gone.
        p.buffs.add_buff(24, 100);
        p.update(&w, PlayerInput::default());
        assert!(p.buffs.find_buff_index(24).is_some());
    }

    #[test]
    fn wet_self_removes_on_lava_contact() {
        // `Player.cs:11741-11744`: the Wet branch deletes itself while
        // `lavaWet`, granting no immunities that frame.
        let mut p = drain_rig();
        p.buffs.add_buff(103, 600);
        p.in_lava = true;
        p.update_survival(&mut tr_core::UnifiedRandom::default());
        assert!(p.buffs.find_buff_index(103).is_none());
        assert!(!p.buffs.buff_immune[24], "no fire immunity without Wet");
    }

    #[test]
    fn honey_softens_a_drain_by_four_and_never_heals() {
        // DW-2.5 — `Player.cs:19096-19103`: honey +4 to a negative regen,
        // clamped at 0 — never positive, no heal from the mitigation itself.
        let mut p = drain_rig();
        p.buffs.add_buff(20, 600); // Poisoned −4
        p.buffs.add_buff(48, 600); // Honey
        for _ in 0..300 {
            p.update_survival(&mut tr_core::UnifiedRandom::default());
        }
        assert_eq!(p.regen.life_regen, 0, "−4 + 4 clamps at 0");
        assert_eq!(p.hp, 1000, "a softened −4 drains nothing");
        let mut p = drain_rig();
        p.buffs.add_buff(70, 600); // Venom −30
        p.buffs.add_buff(48, 600);
        p.update_survival(&mut tr_core::UnifiedRandom::default());
        assert_eq!(p.regen.life_regen, -26, "−30 + 4 stays negative");
        // Honey without a drain never adds anything (the +2 aura is M2-BUFF-5).
        let mut p = drain_rig();
        p.buffs.add_buff(48, 600);
        p.update_survival(&mut tr_core::UnifiedRandom::default());
        assert_eq!(p.regen.life_regen, 0);
        assert_eq!(p.hp, 1000);
    }

    #[test]
    fn fire_family_deltas_sum_with_dripping_slime() {
        // Dirty — fire debuffs coexist and sum (`Player.cs` has no replace
        // chain); drippingSlime (137) doubles the fire family only.
        let mut p = drain_rig();
        p.buffs.add_buff(24, 600); // OnFire −8
        p.buffs.add_buff(324, 600); // Frostburn2 −16
        p.update_survival(&mut tr_core::UnifiedRandom::default());
        assert_eq!(p.regen.life_regen, -24);
        let mut p = drain_rig();
        p.buffs.add_buff(24, 600);
        p.buffs.add_buff(324, 600);
        p.buffs.add_buff(137, 600); // drippingSlime
        p.update_survival(&mut tr_core::UnifiedRandom::default());
        assert_eq!(p.regen.life_regen, -48, "drip doubles both fire parts");
        // A poison part is not doubled by the drip.
        let mut p = drain_rig();
        p.buffs.add_buff(20, 600); // Poisoned −4
        p.buffs.add_buff(24, 600); // OnFire −8
        p.buffs.add_buff(137, 600);
        p.update_survival(&mut tr_core::UnifiedRandom::default());
        assert_eq!(p.regen.life_regen, -20, "−4 poison + (−8×2) fire");
    }

    // M5-SEED-2: the `vampirism` world's OnFire row (`Player.cs:18983-18992`)
    // drains −100 instead of −8, with the drippingSlime double at −100.
    #[test]
    fn the_vampirism_world_trades_the_on_fire_drain_for_its_own() {
        // The classic row first — the flag is off on a classic world.
        let mut p = drain_rig();
        p.buffs.add_buff(24, 600);
        p.update_survival(&mut tr_core::UnifiedRandom::default());
        assert_eq!(p.regen.life_regen, -8);
        // The seeded world: the flag rides the world's micro seeds, captured
        // at `Player::spawn` (`WorldGen.cs:555-558`).
        let mut w = World::new("t".into(), 64, 800);
        w.micro_seeds
            .insert(tr_core::seeds::MicroSeedFlags::VAMPIRISM);
        let mut p = drain_rig_in(&w);
        assert!(p.vampirism, "the flag reaches the player");
        p.buffs.add_buff(24, 600);
        p.update_survival(&mut tr_core::UnifiedRandom::default());
        assert_eq!(p.regen.life_regen, -100, "the vampirism OnFire drain");
        // The drip double is 100 too, and only the fire family is affected.
        let mut p = drain_rig_in(&w);
        p.buffs.add_buff(24, 600);
        p.buffs.add_buff(137, 600);
        p.update_survival(&mut tr_core::UnifiedRandom::default());
        assert_eq!(p.regen.life_regen, -200, "drip doubles the 100");
        let mut p = drain_rig_in(&w);
        p.buffs.add_buff(20, 600); // Poisoned −4 (unchanged)
        p.buffs.add_buff(24, 600); // OnFire −100
        p.buffs.add_buff(137, 600);
        p.update_survival(&mut tr_core::UnifiedRandom::default());
        assert_eq!(p.regen.life_regen, -204, "−4 poison + (−100×2) fire");
    }

    #[test]
    fn electrified_drains_more_while_moving() {
        // Dirty — `Player.cs:19074-19086`: −8, −32 more while `controlLeft ||
        // controlRight` (the run keys, not velocity).
        let mut p = drain_rig();
        p.buffs.add_buff(144, 600);
        p.update_survival(&mut tr_core::UnifiedRandom::default());
        assert_eq!(p.regen.life_regen, -8);
        p.control_left = true;
        p.update_survival(&mut tr_core::UnifiedRandom::default());
        assert_eq!(p.regen.life_regen, -40, "−8 −32 with the left key");
        let mut p = drain_rig();
        p.buffs.add_buff(144, 600);
        p.control_right = true;
        p.update_survival(&mut tr_core::UnifiedRandom::default());
        assert_eq!(p.regen.life_regen, -40, "−8 −32 with the right key");
    }

    #[test]
    fn dot_death_reasons_follow_the_derivation_priority() {
        // Decision #5 — the caller-side reason selection (fast 7→12→8,
        // generic 22→9→10→23→8) is this phase's job; unit checks on the
        // derivation against `Player.cs:19313-19331,19361-19385`.
        let derive = |buffs: &[i32], expert: bool| -> DotDerivation {
            let mut p = drain_rig();
            p.buffs.expert_mode = expert;
            for &id in buffs {
                p.buffs.add_buff(id, 600);
            }
            p.derive_dot_inputs(expert)
        };
        let d = derive(&[20], false);
        assert!(!d.fast_lane);
        assert_eq!(d.death_generic, DotDeath::Poison); // 9 before 10/23
        let d = derive(&[70, 144], false);
        assert_eq!(d.death_generic, DotDeath::Poison); // poisoned||venom first
        let d = derive(&[144, 397], false);
        assert_eq!(d.death_generic, DotDeath::Electrified); // 10 before 23
        let d = derive(&[397], false);
        assert_eq!(d.death_generic, DotDeath::ChlorophyteSpore);
        let d = derive(&[67, 20], false);
        assert!(d.fast_lane);
        assert_eq!(d.death_fast, DotDeath::Burned); // the fast fallback 8
        assert_eq!(d.death_generic, DotDeath::Poison); // the generic lane too
        let d = derive(&[68, 67], false);
        assert_eq!(d.death_fast, DotDeath::Suffocation); // 7 before 8
        let d = derive(&[68, 38], true);
        assert_eq!(d.death_fast, DotDeath::Suffocation); // 7 before 12
        let d = derive(&[38], true);
        assert!(d.fast_lane);
        assert_eq!(d.death_fast, DotDeath::Tongued); // 12
        let d = derive(&[38], false);
        assert!(!d.fast_lane, "tongued needs expert for the fast lane");
        assert_eq!(d.regen, 0, "non-expert tongued drains nothing");
    }

    #[test]
    fn fast_lane_kill_carries_the_derived_reason() {
        // The derivation → engine seam end-to-end: the drain kill returns the
        // reason the held buffs selected (world_sim resolves the text).
        let mut p = drain_rig();
        p.buffs.add_buff(68, 600); // suffocating → fast lane, reason 7
        p.regen.life_regen_count = -600;
        p.hp = 3;
        p.update_survival(&mut tr_core::UnifiedRandom::default());
        assert_eq!(p.hp, 0);
        assert_eq!(p.dot_death, Some(DotDeath::Suffocation));
        let mut p = drain_rig();
        p.buffs.add_buff(20, 600); // poisoned → generic lane, reason 9
        p.regen.life_regen_count = -120;
        p.hp = 1;
        p.update_survival(&mut tr_core::UnifiedRandom::default());
        assert_eq!(p.hp, 0);
        assert_eq!(p.dot_death, Some(DotDeath::Poison));
    }

    #[test]
    fn expired_buffs_are_swept_after_the_tick() {
        // `Player.cs:25608-25616`: slots with a type but no timer are deleted
        // right after `UpdateBuffs`, before any effect reads them.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        p.buffs.add_buff(20, 1);
        p.buffs.add_buff(70, 1);
        p.update(&w, PlayerInput::default());
        assert!(
            p.buffs.active_types().is_empty(),
            "both expired buffs swept: {:?}",
            p.buffs.active_types()
        );
        // And a swept id can be re-added fresh afterwards.
        p.buffs.add_buff(20, 600);
        p.update(&w, PlayerInput::default());
        assert!(p.buffs.find_buff_index(20).is_some());
    }

    // ------------------------------------------------------------------
    // M2-PHYS-7 Phase 2 — the minecart ride (DW-2.1 .. DW-2.5)
    // ------------------------------------------------------------------

    /// The ride-test world: open sky, no ground — every fixture hand-places
    /// its tiles.
    fn cart_world() -> World {
        World::new("cart".into(), 200, 200)
    }

    /// Hand-places one track tile with explicit frame indices — the Phase-1
    /// convention (`minecart.rs` tests): `frame_x` is the front rail frame id,
    /// `frame_y` the back rail (−1 = single rail). Track placement
    /// (`FrameTrack`) is out of scope.
    fn place_track(w: &mut World, x: i32, y: i32, frame_x: i16, frame_y: i16) {
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind: 314,
            frame_x,
            frame_y,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
    }

    /// A straight-rail run (frame 1) on `row` from `x0` to `x1`.
    fn rail_run(w: &mut World, row: i32, x0: i32, x1: i32) {
        for x in x0..=x1 {
            place_track(w, x, row, 1, -1);
        }
    }

    fn place_stone(w: &mut World, x: i32, y: i32) {
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind: 1,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
    }

    /// The mounted player position whose bottom-center magnet sits at
    /// `magnet`: `magnet = position + (width/2 − 25 + 25, 52/2 + 26)` =
    /// `position + (10, 52)` (`Minecart.GetMagnetPosition`,
    /// `Minecart.cs:1290-1294`).
    fn cart_at(magnet: Vec2) -> (f32, f32) {
        (magnet.x - 10.0, magnet.y - CART_MOUNT_HEIGHT)
    }

    /// Mounts and seats the cart on the straight rail of `row` (the frame-1
    /// rail line is `row * 16 + 6`, `Minecart.cs:117-121`) at magnet X `mx`
    /// with the given roll.
    fn cart_on_rail(w: &World, row: i32, mx: f32, vx: f32) -> Player {
        let mut p = Player::spawn(w);
        p.mount_cart();
        let (x, y) = cart_at(Vec2::new(mx, row as f32 * 16.0 + 6.0));
        p.x = x;
        p.y = y;
        p.prev_x = x;
        p.prev_y = y;
        p.vx = vx;
        p.fall_start = row;
        p.fall_start2 = row;
        p
    }

    /// The magnet's world point of the current position.
    fn magnet_of(p: &Player) -> Vec2 {
        Vec2::new(p.x + 10.0, p.y + CART_MOUNT_HEIGHT)
    }

    /// DW-2.1 — the `TR_CART` pin's body, `Mount.SetMount(MountID.Minecart)`
    /// (`Mount.cs:6196-6305`): the Produced ride-field set lands at its
    /// defaults, the mount buff is granted, and the body grows 42 -> 52
    /// bottom-center. A same-type re-mount is a no-op (`:6198`); the death
    /// path strips the ride state (`Player.cs:38907`).
    #[test]
    fn mount_cart_sets_the_ride_state_fields() {
        let w = cart_world();
        let mut p = Player::spawn(&w);
        let top = p.y;
        p.mount_cart();
        assert!(p.cart_mounted);
        assert!(!p.cart_flip);
        assert!(!p.on_track);
        assert!(!p.on_wrong_ground);
        assert_eq!(p.track_boost, 0.0);
        assert_eq!(p.last_boost, (0, 0));
        assert_eq!(p.cart_ramp_time, 0);
        assert_eq!(p.full_rotation, 0.0);
        assert_eq!(p.full_rotation_origin, (0.0, 0.0));
        assert_eq!(p.body_height(), 52.0, "the mounted body is 20x52");
        assert_eq!(p.y, top - 10.0, "the grow lifts the top, the bottom stays");
        assert!(
            p.buffs.find_buff_index(CART_MOUNT_BUFF).is_some(),
            "the mount buff 118 for 3600 ticks"
        );
        // The same-type re-mount early-returns (`Mount.cs:6198`).
        let y = p.y;
        p.mount_cart();
        assert_eq!(p.y, y);
        // The respawn (post-death) path resets the ride state.
        p.respawn();
        assert!(!p.cart_mounted);
        assert_eq!(p.body_height(), 42.0);
        assert!(p.buffs.find_buff_index(CART_MOUNT_BUFF).is_none());
    }

    /// DW-2.2 — the cart top speed is the mount's 13 (`Mount.cs:926`) on a
    /// straight rail: the input accel (`Player.cs:19904-19916`) may overshoot
    /// by one step and the coast clamp (`:20043-20057`) pins it exactly.
    #[test]
    fn cart_top_speed_is_thirteen_on_straight_track() {
        let mut w = cart_world();
        rail_run(&mut w, 40, 30, 120);
        let mut p = cart_on_rail(&w, 40, 35.0 * 16.0 + 8.0, 12.0);
        for _ in 0..30 {
            p.update(
                &w,
                PlayerInput {
                    right: true,
                    ..Default::default()
                },
            );
            assert!(p.on_track, "riding the straight");
            // The accel arm may overshoot by one step (`:19904-19916` has no
            // clamp); the coast clamp erases it the next tick.
            assert!(p.vx <= 13.0 + 0.04 + 1e-4, "bounded overshoot: {}", p.vx);
        }
        assert_eq!(p.vx, 13.0, "the coast clamp pins the top speed");
    }

    /// DW-2.2 — the booster's ±4 applies NEXT tick pre-input, then the input
    /// step, and the clamp caps at 13 (`Player.cs:19818-19833`; the detection
    /// `Minecart.cs:832-847`).
    #[test]
    fn booster_applies_four_next_tick_pre_input_and_clamps() {
        let mut w = cart_world();
        rail_run(&mut w, 40, 30, 120);
        // Right-boost booster (frame 30) mid-run.
        place_track(&mut w, 50, 40, 30, -1);
        let mut p = cart_on_rail(&w, 40, 35.0 * 16.0 + 8.0, 5.0);
        let input = PlayerInput {
            right: true,
            ..Default::default()
        };
        let mut ticks = 0;
        while p.track_boost == 0.0 && ticks < 120 {
            p.update(&w, input);
            ticks += 1;
        }
        assert!(ticks < 120, "the booster was crossed");
        assert_eq!(p.track_boost, 4.0, "the booster tile arms the push");
        let before = p.vx;
        p.update(&w, input);
        assert!(
            (p.vx - (before + 4.0 + 0.04)).abs() < 1e-4,
            "the boost applied pre-input, then the accel step: {}",
            p.vx
        );
        assert_eq!(p.track_boost, 0.0, "the push is spent");
        // The clamp: a fast cart caps at 13 instead of gaining the full 4.
        let mut p = cart_on_rail(&w, 40, 35.0 * 16.0 + 8.0, 12.9);
        let mut ticks = 0;
        while p.track_boost == 0.0 && ticks < 120 {
            p.update(&w, input);
            ticks += 1;
        }
        p.update(&w, input);
        assert_eq!(p.vx, 13.0, "the boost clamp: {}", p.vx);
        assert_eq!(p.track_boost, 0.0);
        // The left-boost booster (frame 31) pushes a leftward roll harder.
        place_track(&mut w, 50, 40, 31, -1);
        let mut p = cart_on_rail(&w, 40, 65.0 * 16.0 + 8.0, -5.0);
        let input = PlayerInput {
            left: true,
            ..Default::default()
        };
        let mut ticks = 0;
        while p.track_boost == 0.0 && ticks < 200 {
            p.update(&w, input);
            ticks += 1;
        }
        assert_eq!(p.track_boost, -4.0, "the left booster arms a negative push");
        let before = p.vx;
        p.update(&w, input);
        assert!((p.vx - (before - 4.0 - 0.04)).abs() < 1e-4, "{}", p.vx);
    }

    /// DW-2.2 — the ramp end launches at 45° (`Minecart.cs:692-701`:
    /// `(vx, −vx·√2/2)` for an up-right exit) and freezes gravity for
    /// `(int)(min(1, |vx|/13) · 20)` ticks (`Player.cs:28148-28150`,
    /// `:27352-27359`). The freeze also proves the input-accel gate
    /// (`velocity.Y == 0`, `Player.cs:19849`) — the DW-2.5 mechanism.
    #[test]
    fn ramp_launch_suppresses_gravity_and_leaves_at_45_degrees() {
        let mut w = cart_world();
        rail_run(&mut w, 41, 30, 38);
        place_track(&mut w, 39, 41, 6, -1); // corner: straight left, up right
        place_track(&mut w, 40, 40, 16, -1); // ramp rising right, ramp end
        let mut p = cart_on_rail(&w, 41, 33.0 * 16.0 + 8.0, 13.0);
        let input = PlayerInput {
            right: true,
            ..Default::default()
        };
        let mut launched = false;
        for _ in 0..20 {
            p.update(&w, input);
            if p.cart_ramp_time > 0 {
                launched = true;
                break;
            }
        }
        assert!(launched, "the ramp end launched the cart");
        let launch_vy = -13.0 * std::f32::consts::FRAC_1_SQRT_2;
        assert!((p.vy - launch_vy).abs() < 1e-3, "45 degrees up: {}", p.vy);
        assert!((p.vx - 13.0).abs() < 1e-4, "the roll carries through");
        assert_eq!(p.cart_ramp_time, 20, "full speed freezes 20 ticks");
        assert!(!p.on_track, "airborne off the ramp end");
        // The freeze: gravity and input accel both silent while vy != 0.
        let (vx0, vy0) = (p.vx, p.vy);
        for _ in 0..5 {
            p.update(&w, input);
            assert!(
                (p.vx - vx0).abs() < 1e-4,
                "no input accel airborne: {}",
                p.vx
            );
            assert!((p.vy - vy0).abs() < 1e-4, "gravity frozen: {}", p.vy);
        }
        // Gravity resumes once the timer runs out.
        for _ in 0..25 {
            p.update(&w, input);
        }
        assert!(p.vy > vy0, "gravity resumed: {} -> {}", vy0, p.vy);
    }

    /// DW-2.2 — the corner hand-off: the cart follows an up-right corner at
    /// speed (`Minecart.cs:600-614` corner switch) and ends on the upper rail
    /// line (the `:818-819` snap).
    #[test]
    fn corner_follows_the_rail_at_speed() {
        let mut w = cart_world();
        rail_run(&mut w, 41, 30, 38);
        place_track(&mut w, 39, 41, 6, -1); // corner: straight left, up right
        place_track(&mut w, 40, 40, 4, -1); // corner: from below, straight right
        rail_run(&mut w, 40, 41, 60);
        let mut p = cart_on_rail(&w, 41, 33.0 * 16.0 + 8.0, 10.0);
        let input = PlayerInput {
            right: true,
            ..Default::default()
        };
        for _ in 0..12 {
            p.update(&w, input);
            assert!(p.on_track, "the rail is never lost through the corner");
        }
        let magnet_y = magnet_of(&p).y;
        assert!(
            (magnet_y - (40.0 * 16.0 + 6.0)).abs() < 0.5,
            "snapped to the upper rail: {}",
            magnet_y
        );
    }

    /// DW-2.3 — off the rails the coast friction is `runSlowdown` = 0.2 per
    /// tick (`Player.cs:26670` + `:20024-20041`) and a direction-key press
    /// edge dismounts (`:26672-26674` → `Mount.TryDismount`).
    #[test]
    fn off_track_friction_is_point_two_and_direction_press_dismounts() {
        let mut w = cart_world();
        rail_run(&mut w, 40, 30, 44); // the run ends in open air
        for x in 36..80 {
            place_stone(&mut w, x, 43); // the landing floor 3 rows down
        }
        let mut p = cart_on_rail(&w, 40, 35.0 * 16.0 + 8.0, 6.0);
        // Ride off the end and fall to the floor; the settle is complete when
        // the off-track-at-rest flag is up with the body at rest.
        for _ in 0..90 {
            p.update(&w, PlayerInput::default());
            if !p.on_track && p.vy == 0.0 && p.on_wrong_ground {
                break;
            }
        }
        assert!(!p.on_track, "off the rails");
        assert_eq!(p.vy, 0.0, "settled on the floor");
        assert!(p.on_wrong_ground, "the on-track check flagged wrong ground");
        // The coast bleeds exactly 0.2 per tick while |vx| >= 1 (the fall
        // decayed 6 by 0.2 per fall tick — about 3 remain).
        assert!(p.vx >= 1.0, "measurable roll remains: {}", p.vx);
        p.update(&w, PlayerInput::default());
        let v1 = p.vx;
        p.update(&w, PlayerInput::default());
        let v2 = p.vx;
        assert!(
            (v1 - v2 - 0.2).abs() < 1e-4,
            "0.2/tick off-track coast: {} -> {}",
            v1,
            v2
        );
        // A direction-key press edge dismounts; the shrink keeps the bottom.
        let y_before = p.y;
        p.update(
            &w,
            PlayerInput {
                right: true,
                ..Default::default()
            },
        );
        assert!(!p.cart_mounted, "the press edge dismounted");
        assert_eq!(p.body_height(), 42.0);
        assert_eq!(p.y, y_before + 10.0, "the shrink anchors the bottom");
        assert!(p.buffs.find_buff_index(CART_MOUNT_BUFF).is_none());
    }

    /// DW-2.3 — the cart jump: the −0.001 magnet nudge
    /// (`Player.cs:20671-20675`) then the fixed cart jump stats 15/5.15
    /// (`Mount.cs:929-930`; no `JumpHeight` case for carts) — the rider
    /// leaves the rail, the mount stays, and the rider lands back.
    #[test]
    fn jump_from_the_cart_uses_the_cart_jump_stats() {
        let mut w = cart_world();
        rail_run(&mut w, 40, 30, 120);
        let mut p = cart_on_rail(&w, 40, 35.0 * 16.0 + 8.0, 6.0);
        p.update(&w, PlayerInput::default());
        assert!(p.on_track, "seated");
        p.update(
            &w,
            PlayerInput {
                jump: true,
                ..Default::default()
            },
        );
        assert_eq!(p.jump, CART_JUMP_HEIGHT, "the cart jump height 15");
        assert!(
            (p.vy - (-CART_JUMP_SPEED + GRAVITY)).abs() < 1e-4,
            "the 5.15 impulse minus this tick's gravity: {}",
            p.vy
        );
        assert!(p.cart_mounted, "still mounted");
        assert!(!p.on_track, "left the rail");
        // Hold the jump through the rise, then fall back onto the rail.
        for _ in 0..30 {
            p.update(
                &w,
                PlayerInput {
                    jump: true,
                    ..Default::default()
                },
            );
        }
        for _ in 0..60 {
            p.update(&w, PlayerInput::default());
        }
        assert!(p.on_track, "landed back on the rail");
        assert!(p.cart_mounted, "the mount survived the hop");
    }

    /// DW-2.3 — `Player.cs:25311-25314`: a track rider resting on rails takes
    /// no fall damage; the same drop onto stone does.
    #[test]
    fn landing_on_rails_zeroes_fall_damage() {
        // The rail catch: a 30-row drop lands harmlessly.
        let mut w = cart_world();
        rail_run(&mut w, 40, 30, 60);
        let mut p = cart_on_rail(&w, 40, 40.0 * 16.0 + 8.0, 0.0);
        p.y = 10.0 * 16.0;
        p.prev_y = p.y;
        p.fall_start = 10;
        p.fall_start2 = 10;
        for _ in 0..90 {
            p.update(&w, PlayerInput::default());
        }
        assert!(p.on_track, "landed on the rail");
        assert_eq!(p.hp, 100, "no fall damage on rails");
        // The same drop onto stone hurts.
        let mut w2 = cart_world();
        for x in 30..60 {
            place_stone(&mut w2, x, 41);
        }
        let mut p2 = Player::spawn(&w2);
        p2.mount_cart();
        p2.x = 40.0 * 16.0 - 10.0;
        p2.y = 10.0 * 16.0;
        p2.prev_y = p2.y;
        p2.fall_start = 10;
        p2.fall_start2 = 10;
        for _ in 0..90 {
            p2.update(&w2, PlayerInput::default());
        }
        assert!(p2.hp < 100, "the same drop onto stone hurts: {}", p2.hp);
    }

    /// DW-2.4 — riding over a pressure track (frame 21, `_trackType == 1`)
    /// trips the switch once per pass: the magnet tile is handed to
    /// `Wiring::hit_switch` by the caller (`Player.cs:28237-28241`,
    /// `Minecart.cs:1281-1288`).
    #[test]
    fn riding_over_a_pressure_track_trips_the_switch_once_per_pass() {
        let mut w = cart_world();
        rail_run(&mut w, 40, 30, 120);
        place_track(&mut w, 50, 40, 21, -1); // the pressure straight
        let mut p = cart_on_rail(&w, 40, 35.0 * 16.0 + 8.0, 6.0);
        let input = PlayerInput {
            right: true,
            ..Default::default()
        };
        let mut trips = 0;
        let mut trip_tile = None;
        for _ in 0..40 {
            p.update(&w, input);
            if let Some((i, j)) = p.take_switch_trip() {
                trips += 1;
                trip_tile = Some((i, j));
            }
        }
        assert_eq!(trips, 1, "once per pass");
        assert_eq!(trip_tile, Some((50, 40)), "the magnet tile");
        // A plain-rail pass never trips.
        let mut p = cart_on_rail(&w, 40, 35.0 * 16.0 + 8.0, 6.0);
        for _ in 0..10 {
            p.update(&w, input);
            assert!(
                p.take_switch_trip().is_none(),
                "no trip off the pressure tile"
            );
        }
    }

    /// DW-2.5 — the bouncy bumper (frame 24): the rewind flips the roll and
    /// the rider keeps the rail (`Minecart.cs:669-691`, `:914-922`;
    /// `Player.cs:28123-28136` toggles `cartFlip` on the held key). The
    /// input-accel suppression half of this item is the `velocity.Y == 0`
    /// gate, exercised on the ramp flight in `ramp_launch_...` — a flat-rail
    /// bounce itself resolves with vy == 0 (the ride snaps zero the march Y
    /// before the resolve, `Minecart.cs:920`).
    #[test]
    fn bouncy_bumper_flips_velocity_and_the_rider_keeps_the_rail() {
        let mut w = cart_world();
        rail_run(&mut w, 40, 30, 60);
        place_track(&mut w, 45, 40, 24, -1); // bouncy bumper, rail on the right half
        let mut p = cart_on_rail(&w, 40, 35.0 * 16.0 + 8.0, 8.0);
        let input = PlayerInput {
            right: true,
            ..Default::default()
        };
        let mut flipped = false;
        for _ in 0..20 {
            p.update(&w, input);
            if p.vx < 0.0 {
                flipped = true;
                break;
            }
        }
        assert!(flipped, "the bounce flipped the roll");
        assert!(p.cart_flip, "the held direction key toggled the flip");
        assert!(p.on_track, "still riding the bouncy tile's rail");
        assert_eq!(p.vy, 0.0, "the ride resolve zeroes vy");
        let magnet_y = magnet_of(&p).y;
        assert!(
            (magnet_y - (40.0 * 16.0 + 6.0)).abs() < 0.5,
            "still on the rail line: {}",
            magnet_y
        );
    }

    /// Review probe (Phase 2 gate fix 1) — the direction lock reads the
    /// POST-boost roll: the boost applies at `Player.cs:19818-19833` and the
    /// lock sits at `:19835-19843`, both inside `HorizontalMovement`. Rolling
    /// left over a right-boost booster with the right key held flips the
    /// facing on the boost-apply tick, not one tick later.
    #[test]
    fn boost_tick_flips_the_facing_to_the_boosted_direction() {
        let mut w = cart_world();
        rail_run(&mut w, 40, 30, 60);
        place_track(&mut w, 45, 40, 30, -1); // right-boost booster
        let mut p = cart_on_rail(&w, 40, 46.0 * 16.0 + 8.0, -3.0);
        p.facing = -1;
        let input = PlayerInput {
            right: true,
            ..Default::default()
        };
        // The crossing tick: the booster arms the push against the roll; the
        // lock (pre-boost roll = negative) keeps the facing.
        let mut crossed = false;
        for _ in 0..20 {
            p.update(&w, input);
            if p.track_boost != 0.0 {
                crossed = true;
                break;
            }
        }
        assert!(crossed, "the booster was crossed rolling left");
        assert_eq!(p.facing, -1, "the crossing tick keeps the roll facing");
        // The boost-apply tick: the roll turns positive pre-input and the
        // lock (post-boost) flips the facing the same tick.
        p.update(&w, input);
        assert!(p.vx > 0.0, "the boost turned the roll: {}", p.vx);
        assert_eq!(p.facing, 1, "the boosted direction, same tick");
    }

    /// Review probe (Phase 2 gate fix 1) — the dismount edge fires in the
    /// stat override (`Player.cs:26672-26674`) and the now-unmounted
    /// direction block inside `HorizontalMovement` (`:19835-19843`) flips
    /// the facing the SAME tick.
    #[test]
    fn dismount_tick_flips_the_facing_to_the_pressed_direction() {
        let mut w = cart_world();
        rail_run(&mut w, 40, 30, 44);
        for x in 36..80 {
            place_stone(&mut w, x, 43);
        }
        let mut p = cart_on_rail(&w, 40, 35.0 * 16.0 + 8.0, 6.0);
        // Roll off the end and settle off-track.
        for _ in 0..90 {
            p.update(&w, PlayerInput::default());
            if !p.on_track && p.vy == 0.0 && p.on_wrong_ground {
                break;
            }
        }
        assert!(p.on_wrong_ground && !p.on_track && p.vy == 0.0, "settled");
        assert_eq!(p.facing, 1, "the keyless roll-off kept the spawn facing");
        // The left press edge: the dismount and the facing flip land together.
        p.update(
            &w,
            PlayerInput {
                left: true,
                ..Default::default()
            },
        );
        assert!(!p.cart_mounted, "the press edge dismounted");
        assert_eq!(p.facing, -1, "the pressed direction, same tick");
    }

    /// The regular bumper (frame 2) dead-stops the cart at the rail line
    /// (`Minecart.cs:660-668` — the `stopped` resolve keeps only the travelled
    /// X). With the key held the C# vibrates in place: the accel arm grants
    /// one 0.04 step on the vy == 0 ticks and the boundary sentinel eats it
    /// when the march crosses the sub edge, so |vx| never exceeds one step.
    #[test]
    fn bumper_stops_the_cart_on_the_rail() {
        let mut w = cart_world();
        rail_run(&mut w, 40, 30, 60);
        place_track(&mut w, 45, 40, 2, -1); // bumper on the left half
        let mut p = cart_on_rail(&w, 40, 35.0 * 16.0 + 8.0, 8.0);
        let input = PlayerInput {
            right: true,
            ..Default::default()
        };
        let mut reached = false;
        for _ in 0..20 {
            p.update(&w, input);
            if p.vx.abs() < 1e-4 {
                reached = true;
            }
            if reached {
                // The held key vibrates in place: one 0.04 accel step on the
                // vy == 0 ticks, eaten by the boundary sentinel's stop.
                assert!(
                    p.vx.abs() <= 0.04 + 1e-4,
                    "held against the bumper: {}",
                    p.vx
                );
                assert!(p.on_track, "resting on the rail, never unseated");
            }
        }
        assert!(reached, "the stop zeroed the roll at the boundary");
        let magnet = magnet_of(&p);
        assert!((magnet.y - (40.0 * 16.0 + 6.0)).abs() < 0.5);
        assert!(magnet.x < 45.0 * 16.0 + 0.5, "never inside the bumper sub");
    }

    /// `Player.cs:28088-28092`: the cart always falls through platforms —
    /// `fallThrough`/`flag31` are forced for `Mount.Cart`.
    #[test]
    fn the_cart_falls_through_platforms() {
        let mut w = cart_world();
        for x in 30..60 {
            let i = w.index(x, 38);
            w.tiles[i] = Tile {
                kind: 19, // platform
                frame_y: 0,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        for x in 30..60 {
            place_stone(&mut w, x, 42);
        }
        let mut p = Player::spawn(&w);
        p.mount_cart();
        let (x, y) = cart_at(Vec2::new(40.0 * 16.0 + 8.0, 30.0 * 16.0));
        p.x = x;
        p.y = y;
        p.prev_x = x;
        p.prev_y = y;
        for _ in 0..90 {
            p.update(&w, PlayerInput::default());
        }
        let bottom = p.y + CART_MOUNT_HEIGHT;
        assert!(
            (bottom - 42.0 * 16.0).abs() < 1.0,
            "fell through to the floor: {}",
            bottom
        );
        assert!(!p.on_track, "stone is not a rail");
    }

    /// Dirty (plan P2) — `TR_CART` with no rails under the magnet: the
    /// mounted cart just falls under gravity; no ride state, no panic. The
    /// first rest tick flags `on_wrong_ground` (the C# off-track-at-rest
    /// check, `Player.cs:26665-26674`) and the cart keeps it through the fall
    /// (`:24371-24374` preserves it for carts).
    #[test]
    fn cart_with_no_rails_falls_normally() {
        let w = cart_world();
        let mut p = Player::spawn(&w);
        p.mount_cart();
        p.y = 30.0 * 16.0;
        p.update(&w, PlayerInput::default());
        assert!(p.on_wrong_ground, "resting off-track flags wrong ground");
        for _ in 0..30 {
            p.update(&w, PlayerInput::default());
            assert!(!p.on_track);
        }
        assert!(p.vy > 0.0, "falling under gravity: {}", p.vy);
    }

    // ------------------------------------------------------------------
    // M2-PHYS-7 Phase 3 — wheel sparks + sounds
    // ------------------------------------------------------------------

    /// Finds a stream seed where one rolling call at `|vx| = vel_x` queues
    /// exactly `want` sparks — the gate rolls interleave with the 14-roll
    /// sparks (the rain.rs seed-search idiom, run against the same emitter
    /// constants the tick passes).
    fn seed_with_spark_count(vel_x: f32, want: usize) -> u64 {
        for s in 1u64.. {
            let mut sparks = Vec::new();
            let mut rng = s;
            crate::minecart::rolling_wheel_sparks(
                &mut Vec::new(),
                &mut sparks,
                &mut rng,
                Vec2::new(100.0, 200.0),
                20,
                52,
                Vec2::new(vel_x, 0.0),
                13.0,
                0.0,
            );
            if sparks.len() == want {
                return s;
            }
        }
        unreachable!()
    }

    /// DW-3.2 — the landing burst (`Minecart.cs:809-813` via the Phase-1
    /// `TrackFlags::landing`): a real fall onto a rail queues the Item53 cue
    /// and exactly 10 sparks at the pre-call magnet point, with the
    /// pre-call rotation (`Player.cs:28110-28111` sets the emitter statics
    /// before `TrackCollision` runs).
    #[test]
    fn landing_on_a_rail_bursts_ten_sparks_and_item53() {
        let mut w = cart_world();
        rail_run(&mut w, 40, 30, 120);
        let mut p = cart_on_rail(&w, 40, 35.0 * 16.0 + 8.0, 0.0);
        p.y -= 64.0; // four rows above the rail line
        p.prev_y = p.y;
        p.vy = 5.0; // the gate needs `velocity.Y > defaultGravity` (0.4)
        p.fall_start2 = (p.y / 16.0) as i32 - 3; // a real fall origin
        // The fall takes a handful of ticks to cross the 64 px to the rail;
        // the landing tick queues the burst with its pre-call position.
        let mut landed_at = None;
        for _ in 0..20 {
            let (pre_x, pre_y) = (p.x, p.y);
            p.update(&w, PlayerInput::default());
            if !p.pending_sparks.is_empty() {
                landed_at = Some((pre_x, pre_y));
                break;
            }
        }
        let (pre_x, pre_y) = landed_at.expect("the landing fired");
        assert!(p.on_track, "the fall ended on the rail");
        assert_eq!(p.pending_sparks.len(), 10, "the 10-spark burst");
        assert_eq!(p.pending_cart_sounds.len(), 1);
        assert_eq!(p.pending_cart_sounds[0].cue, "Item_53");
        // Every spark sits at the pre-call magnet point (rotation 0 on the
        // flat rail): y exactly `pre_y + 52`, x = pre_x + 10 ± 13 + 4.
        for s in &p.pending_sparks {
            assert_eq!(s.pos.y, pre_y + 52.0);
            assert!(
                (s.pos.x - (pre_x + 27.0)).abs() < 1e-3 || (s.pos.x - (pre_x + 1.0)).abs() < 1e-3,
                "x {}",
                s.pos.x
            );
        }
    }

    /// DW-3.3 (dirty) — the rolling gates are direction-opposite
    /// (`Player.cs:19868`: the left arm needs `velocity.X >= 0f`; `:19926`:
    /// the right arm `velocity.X <= 0f`): riding forward is silent, braking
    /// a same-sign roll sparks.
    #[test]
    fn riding_forward_does_not_spark_the_braking_arms_do() {
        let mut w = cart_world();
        rail_run(&mut w, 40, 30, 140);
        // Settle one tick so the previous tick's `onTrack` is up.
        let mut p = cart_on_rail(&w, 40, 35.0 * 16.0 + 8.0, 12.0);
        p.update(&w, PlayerInput::default());
        // All three 50% gates would pass — the arm's sign gate is the only
        // thing that can stop them.
        p.spark_rng = seed_with_spark_count(10.4, 3);
        // Riding forward (right held, vx > 0): the right arm's `vx <= 0`
        // gate fails — no sparks, no Item55.
        p.update(
            &w,
            PlayerInput {
                right: true,
                ..Default::default()
            },
        );
        assert!(p.pending_sparks.is_empty(), "forward riding is silent");
        assert!(p.pending_cart_sounds.is_empty());
        // Braking (left held against the rightward roll): the left arm's
        // `vx >= 0` gate fires the three rolls → 3 sparks + Item55.
        let mut p = cart_on_rail(&w, 40, 35.0 * 16.0 + 8.0, 13.0);
        p.update(&w, PlayerInput::default());
        p.spark_rng = seed_with_spark_count(10.4, 3);
        p.update(
            &w,
            PlayerInput {
                left: true,
                ..Default::default()
            },
        );
        assert_eq!(p.pending_sparks.len(), 3, "the braking arm sparks");
        assert_eq!(p.pending_cart_sounds.len(), 1);
        assert_eq!(p.pending_cart_sounds[0].cue, "Item_55");
        // At rest the arm's own accel (`Player.cs:19855`: `velocity.X -=
        // runAcceleration`, unconditional for a grounded cart) flips vx to
        // −0.04 before the gate reads it — the at-rest press never sparks
        // (the C# order; the gate is the arm's post-accel velocity).
        let mut p = cart_on_rail(&w, 40, 35.0 * 16.0 + 8.0, 0.0);
        p.update(&w, PlayerInput::default());
        p.update(
            &w,
            PlayerInput {
                left: true,
                ..Default::default()
            },
        );
        assert!(
            p.pending_sparks.is_empty(),
            "the at-rest press never sparks"
        );
    }

    /// DW-3.3 (dirty) — no sparks off-track or airborne: the gate needs the
    /// previous tick's `onTrack` with `!onWrongGround` and `velocity.Y == 0`
    /// (`Player.cs:19868`).
    #[test]
    fn sparks_are_absent_off_track_and_airborne() {
        let mut w = cart_world();
        rail_run(&mut w, 40, 30, 60);
        // Off-track: a mounted cart on plain ground (no rails) never passes
        // the gate — the on-track check flags `on_wrong_ground` and the
        // previous tick's `onTrack` is down.
        let mut w2 = cart_world();
        for x in 20..80 {
            place_stone(&mut w2, x, 45);
        }
        let mut p = Player::spawn(&w2);
        p.mount_cart();
        p.y = 45.0 * 16.0 - 52.0;
        p.prev_y = p.y;
        p.vx = 10.0;
        p.spark_rng = seed_with_spark_count(10.4, 3);
        p.update(
            &w2,
            PlayerInput {
                left: true,
                ..Default::default()
            },
        );
        assert!(p.on_wrong_ground, "the off-track flag is up");
        assert!(p.pending_sparks.is_empty(), "off-track: no sparks");
        assert!(p.pending_cart_sounds.is_empty(), "off-track: no Item55");
        // Airborne: jump off the rail, then brake mid-air — `onTrack` (the
        // previous tick's) is down and `velocity.Y != 0`.
        let mut w = cart_world();
        rail_run(&mut w, 40, 30, 120);
        let mut p = cart_on_rail(&w, 40, 35.0 * 16.0 + 8.0, 6.0);
        p.update(&w, PlayerInput::default());
        p.update(
            &w,
            PlayerInput {
                jump: true,
                ..Default::default()
            },
        );
        assert!(p.vy != 0.0, "the jump launched the cart");
        p.spark_rng = seed_with_spark_count(10.4, 3);
        p.update(
            &w,
            PlayerInput {
                left: true,
                ..Default::default()
            },
        );
        assert!(p.pending_sparks.is_empty(), "airborne: no sparks");
        assert!(p.pending_cart_sounds.is_empty(), "airborne: no Item55");
    }

    /// DW-3.3 (the replay half, through the tick) — the same world, seed and
    /// inputs give byte-equal spark queues and identical stream positions.
    #[test]
    fn the_player_spark_stream_replays_identically() {
        let mut w = cart_world();
        rail_run(&mut w, 40, 30, 140);
        let mut a = cart_on_rail(&w, 40, 35.0 * 16.0 + 8.0, 12.0);
        let mut b = cart_on_rail(&w, 40, 35.0 * 16.0 + 8.0, 12.0);
        let seed = seed_with_spark_count(10.4, 3);
        a.spark_rng = seed;
        b.spark_rng = seed;
        for _ in 0..10 {
            let input = PlayerInput {
                left: true,
                ..Default::default()
            };
            a.update(&w, input);
            b.update(&w, input);
            assert_eq!(a.pending_sparks, b.pending_sparks, "byte-equal queues");
            assert_eq!(a.pending_cart_sounds, b.pending_cart_sounds);
            assert_eq!(a.spark_rng, b.spark_rng, "identical stream positions");
        }
    }

    /// Dirty (pitfall §10.13) — the dismount resets `cartFlip`/`lastBoost`/
    /// `fullRotation` and clears the mount buff (`Mount.cs:6151-6162`), and a
    /// fresh mount starts clean (`:6205-6211`).
    #[test]
    fn dismount_resets_the_ride_state() {
        let w = cart_world();
        let mut p = Player::spawn(&w);
        p.mount_cart();
        p.cart_flip = true;
        p.last_boost = (3, 4);
        p.full_rotation = 0.5;
        p.full_rotation_origin = (10.0, 52.0);
        p.track_boost = 2.0;
        p.cart_dismount();
        assert!(!p.cart_mounted);
        assert!(!p.cart_flip);
        assert_eq!(p.last_boost, (0, 0));
        assert_eq!(p.full_rotation, 0.0);
        assert_eq!(p.full_rotation_origin, (0.0, 0.0));
        assert!(p.buffs.find_buff_index(CART_MOUNT_BUFF).is_none());
        p.mount_cart();
        assert!(p.cart_mounted);
        assert_eq!(p.last_boost, (0, 0), "the fresh mount starts clean");
        assert!(!p.cart_flip);
    }
    // -- M2-MOUNT-1 phase 2: the per-tick mount block ------------------------

    #[test]
    fn the_mount_replaces_the_run_stats_and_suppresses_the_wings() {
        // DW-2.1/2.2 — the replace mode `Player.cs:26649-26654` and the
        // wings/rocket suppression `:26625-26635`.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..60 {
            p.update(&w, PlayerInput::default());
        }
        p.wings_logic = 22; // pretend wings so the suppression has a target
        p.has_wings = true;
        p.rocket_boots = 1;
        p.set_mount(10, false); // Unicorn 4/12/0.3
        p.update(&w, PlayerInput::default());
        assert_eq!(p.max_run_speed_value(), 4.0, "maxRunSpeed replaced");
        assert_eq!(p.acc_run_speed_value(), 12.0, "accRunSpeed = DashSpeed");
        assert!(!p.has_wings && p.wings_logic == 0, "wings suppressed");
        assert_eq!(p.rocket_boots, 0, "rocketBoots zeroed");
    }

    #[test]
    fn flying_with_wings_max_merges_instead_of_replacing() {
        // `:26643-26648` — flag17 keeps the better stat (Hermes' 6.0 beats
        // the Slime mount's 4.0 DashSpeed).
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..60 {
            p.update(&w, PlayerInput::default());
        }
        p.equipped[3] = 54; // Hermes — `accRunSpeed = 6f`
        p.update(&w, PlayerInput::default()); // the grant pass runs
        assert_eq!(p.acc_run_speed_value(), 6.0);
        p.set_mount(3, false);
        p.wings_logic = 22;
        p.vy = -1.0; // airborne so flag17 can arm
        p.mount.set_test_overrides(Some(true), None); // the M2-MOUNT-2 row
        p.update_mount(&w, false, None);
        assert_eq!(p.acc_run_speed_value(), 6.0, "max(6.0, 4.0)");
        // without the wings the same merge replaces: 4.0
        p.mount.set_test_overrides(Some(false), None);
        p.wings_logic = 22;
        p.update_mount(&w, false, None);
        assert_eq!(p.acc_run_speed_value(), 4.0, "replaced while grounded");
    }

    #[test]
    fn the_additive_mode_adds_instead_of_replacing() {
        // `:26637-26641` — the `MovementStatsAreAdditive` branch (roller
        // skates in vanilla; the row lands with M2-MOUNT-4).
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..60 {
            p.update(&w, PlayerInput::default());
        }
        p.set_mount(3, false);
        p.mount.set_test_overrides(None, Some(true));
        p.update_mount(&w, false, None);
        assert_eq!(p.max_run_speed_value(), 3.0 + 4.0, "3 + Slime runSpeed");
        assert_eq!(p.acc_run_speed_value(), 3.0 + 4.0, "3 + Slime dashSpeed");
    }

    #[test]
    fn the_mount_jumps_replace_and_scale_with_x_velocity() {
        // DW-2.3 — `Player.cs:19619-19631` + `Mount.cs:2810-2858`.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.set_mount(10, false); // Unicorn — no scaling branch
        p.vx = 16.0;
        p.update_mount(&w, false, None);
        assert_eq!(p.jump_height, 10);
        assert_eq!(p.jump_speed, 8.01);
        p.dismount(&w, false);
        p.set_mount(0, false); // Rudolph, type 0
        p.vx = 4.0;
        p.update_mount(&w, false, None);
        assert_eq!(p.jump_height, 18, "17 + (int)(4/4f)");
        assert_eq!(p.jump_speed, 5.31 + 4.0 / 7.0);
    }

    #[test]
    fn the_unicorn_arms_its_extra_jump_and_the_chain_consumes_it() {
        // DW-2.3 — `Mount.cs:4950-4953` arms the option; the PHYS-8 chain
        // (`Player.cs:20698-21022`) consumes it after the vanilla re-arm.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..60 {
            p.update(&w, PlayerInput::default());
        }
        p.set_mount(10, false);
        // the ground jump
        let press = PlayerInput {
            jump: true,
            ..Default::default()
        };
        p.update(&w, press);
        assert!(p.vy < 0.0, "the ground jump fired");
        // release mid-air (the else branch zeroes `jump` and latches the
        // release), then press again — the extra jump fires while airborne
        let release = PlayerInput::default();
        p.update(&w, release);
        p.update(&w, press);
        assert_eq!(
            p.performing_jump,
            Some(ExtraJump::Unicorn),
            "the mount jump consumed"
        );
        assert!(!p.can_jump_again[ExtraJump::Unicorn.index()]);
    }

    #[test]
    fn the_bunny_nullifies_fall_damage_and_the_unicorn_scales_it() {
        // DW-2.4 — `Player.cs:25315-25318` (bunny zero) + `:25363-25367`
        // (the FallDamage multiplier) + `:25307-25310` (CanFly zero).
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.set_mount(1, false); // Bunny — the nullify
        p.y = 25.0 * TILE_PX;
        p.fall_start = 25;
        for _ in 0..600 {
            p.update(&w, PlayerInput::default());
            if p.on_ground {
                break;
            }
        }
        assert_eq!(p.hp, 100, "the bunny nullifies the fall");
        // Rudolph can fly — `CanFly` zeroes the fall distance too
        p.dismount(&w, false);
        p.set_mount(0, false);
        p.y = 25.0 * TILE_PX;
        p.fall_start = 25;
        p.hp = 100;
        for _ in 0..600 {
            p.update(&w, PlayerInput::default());
            if p.on_ground {
                break;
            }
        }
        assert_eq!(p.hp, 100, "the flying mount zeroes the fall");
        // Unicorn — 0.2x: (32 - 25) * 10 = 70 -> 14
        p.dismount(&w, false);
        p.set_mount(10, false);
        p.y = 25.0 * TILE_PX;
        p.fall_start = 25;
        p.hp = 100;
        for _ in 0..600 {
            p.update(&w, PlayerInput::default());
            if p.on_ground {
                break;
            }
        }
        // The 76px unicorn hitbox rests with its top on row 55 (960-76=884),
        // so `num11` measures 55-25 = 30: (30-25)*10 = 50, x0.2 = 10.
        assert_eq!(p.hp, 90, "(50) * 0.2 = 10");
    }

    #[test]
    fn the_slime_extra_fall_lifts_the_threshold() {
        // `:25301-25304` + `Mount.cs:869` (extraFall 10): a 30-tile fall with
        // the Slime mount stays under `25 + 10`.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.set_mount(3, false);
        p.y = 30.0 * TILE_PX;
        p.fall_start = 30;
        for _ in 0..600 {
            p.update(&w, PlayerInput::default());
            if p.on_ground {
                break;
            }
        }
        assert_eq!(p.hp, 100, "35 threshold eats the 30-tile fall");
    }

    #[test]
    fn equipping_grows_the_hitbox_and_dismounting_restores_it() {
        // DW-2.5 — `Mount.cs:6264-6277` grow, `:6176-6193` restore.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..60 {
            p.update(&w, PlayerInput::default());
        }
        let y_before = p.y;
        assert_eq!(p.height, HEIGHT);
        assert!(p.set_mount(10, false)); // Unicorn boost 34
        assert_eq!(p.height, HEIGHT + 34.0);
        assert!(
            (p.y - (y_before - 34.0)).abs() < 0.01,
            "the bottom stays anchored"
        );
        assert!(
            p.buffs.find_buff_index(162).is_some(),
            "the mount buff added (:6226)"
        );
        assert!(p.dismount(&w, false));
        assert_eq!(p.height, HEIGHT);
        assert!((p.y - y_before).abs() < 0.01, "the restore returns the top");
        assert!(
            p.buffs.find_buff_index(162).is_none(),
            "the buff cleared (:6150)"
        );
    }

    #[test]
    fn cced_dismounts_the_mount_mid_tick() {
        // DW-2.6 — `Player.cs:26799-26812`: the Frozen buff CCs, the mount
        // dismounts during the same tick.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..60 {
            p.update(&w, PlayerInput::default());
        }
        p.set_mount(10, false);
        p.buffs.add_buff(47, 100); // BuffID.Frozen
        p.update(&w, PlayerInput::default());
        assert!(!p.mount.is_active(), "CCed dismounts");
        assert_eq!(p.height, HEIGHT, "the hitbox restored");
    }

    #[test]
    fn the_auto_jump_refill_uses_just_jumped() {
        // `:26776-26781` — a constant-jump mount's re-jump refills the flight
        // budget (`autoJump && justJumped`).
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..60 {
            p.update(&w, PlayerInput::default());
        }
        p.set_mount(50, false); // Queen Slime: constantJump + flightTimeMax 80
        let press = PlayerInput {
            jump: true,
            ..Default::default()
        };
        p.update(&w, press);
        assert!(p.auto_jump, "the mount head-sets autoJump (:4797)");
        assert!(p.just_jumped, "the grounded trigger sets justJumped");
        assert!(p.mount.fly_time() > 0, "ResetFlightTime refilled (:26777)");
    }

    const EMPTY_ITEM_SLOT: crate::item::ItemSlot = crate::item::ItemSlot {
        kind: 0,
        name: "",
        count: 0,
        prefix: 0,
    };

    // -- M2-MOUNT-1 phase 3: triggers, buff lifecycle, quick-mount ----------

    fn saddle(kind: u16) -> crate::item::ItemSlot {
        crate::item::ItemSlot {
            kind,
            name: "",
            count: 1,
            prefix: 0,
        }
    }

    #[test]
    fn quick_mount_equips_the_first_saddle_and_the_buff_persists() {
        // DW-3.1 — `QuickMount` (`Player.cs:6045-6100`) + the refresh
        // (`:9881-9884`, `buffTime = 10`).
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..60 {
            p.update(&w, PlayerInput::default());
        }
        let inv = [saddle(3260), EMPTY_ITEM_SLOT];
        assert_eq!(
            p.quick_mount(&w, &inv),
            Some(3260),
            "the Blessed Apple equips"
        );
        assert!(p.mount.is_active() && p.mount.kind() == 10);
        assert_eq!(p.height, HEIGHT + 34.0, "the hitbox grew");
        assert!(
            p.buffs.find_buff_index(162).is_some(),
            "the mount buff added (:6226)"
        );
        p.update(&w, PlayerInput::default());
        let i = p.buffs.find_buff_index(162).unwrap();
        assert_eq!(p.buffs.buff_time[i], 10, "the refresh holds buffTime at 10");
        assert!(p.mount.is_active(), "the buff keeps the mount alive");
    }

    #[test]
    fn the_buff_expiring_dismounts() {
        // DW-3.2 — `CheckMountBuff` (`Mount.cs:5275-5281` via `:18832`).
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.set_mount(10, false);
        assert!(p.mount.is_active());
        let i = p.buffs.find_buff_index(162).unwrap();
        p.buffs.del_buff(i);
        p.update(&w, PlayerInput::default());
        assert!(!p.mount.is_active(), "the buff gone -> dismount");
        assert_eq!(p.height, HEIGHT, "the hitbox restored");
    }

    #[test]
    fn dismount_on_item_use_respects_the_same_mount_exemption() {
        // DW-3.3 — the `DismountOnItemUse` predicate (`Mount.cs:642-652`);
        // the item gate itself is the editing.rs arm (`Player.cs:31695-31698`).
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.set_mount(10, false);
        assert!(
            !p.mount.dismount_on_item_use(),
            "no ground row sets the flag"
        );
        p.mount.set_test_dismount_on_item_use(Some(true));
        assert!(p.mount.dismount_on_item_use());
        // the exemption: the held item being the same mount does not clear
        let same = crate::mount::mount_type_of_item(3260) == Some(p.mount.kind());
        assert!(same, "Blessed Apple is the Unicorn's own saddle");
        // a different item dismounts through the vanilla gate
        assert!(p.try_dismount(&w));
        assert!(!p.mount.is_active());
    }

    #[test]
    fn quick_mount_toggles_and_guards() {
        // DW-3.4 — the toggle, the no-saddle fallback, the CC guard, the
        // space check (`Mount.CanMount`, `Mount.cs:6581-6604`).
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..60 {
            p.update(&w, PlayerInput::default());
        }
        let none = [EMPTY_ITEM_SLOT];
        assert_eq!(p.quick_mount(&w, &none), None, "no saddle -> nothing");
        let inv = [saddle(2430)]; // Slimy Saddle -> Slime 3
        assert_eq!(p.quick_mount(&w, &inv), Some(2430));
        assert!(p.mount.is_active());
        assert_eq!(p.quick_mount(&w, &inv), None, "mounted -> dismount");
        assert!(!p.mount.is_active());
        // the CC guard (`:6074-6077`)
        p.buffs.add_buff(47, 100); // Frozen
        p.update(&w, PlayerInput::default());
        assert_eq!(p.quick_mount(&w, &inv), None, "CCed cannot mount");
    }

    #[test]
    fn can_mount_needs_the_grown_box_to_fit() {
        // `Mount.cs:6593-6595` — the Unicorn's 76px box does not fit under a
        // 3-tile ceiling where the Slime's 62px does.
        let mut w = ground_world();
        // A 64px cavity: solid rows 54-55, air rows 56-59, ground row 60.
        for x in 0..64 {
            for y in 54..56 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        let mut p = Player::spawn(&w);
        p.y = 918.0; // resting on the row-60 ground inside the cavity
        assert!(!p.can_mount(&w, 10), "76px does not fit in the 64px cavity");
        assert!(p.can_mount(&w, 3), "62px fits");
    }

    #[test]
    fn s1_l_8_can_mount_applies_the_bat_and_grapple_sets() {
        // `Mount.cs:6595` (the bat wet rule) and `:6599-6601` (the
        // `Cart`/`CanUseHooks` grapple exemption).
        let mut w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        // Dry and grounded: the bat (56) fits.
        assert!(p.can_mount(&w, 56));
        // `wet` blocks it; so does the Wet buff's `dripping` flag.
        p.wet = true;
        assert!(!p.can_mount(&w, 56), "the bat refuses water");
        p.wet = false;
        p.buffs.add_buff(103, 600); // `Player.cs:11743-11747`
        assert!(!p.can_mount(&w, 56), "dripping blocks the bat");
        p.clear_buff_type(103);
        // `Collision.WetCollision` on the `Bottom - size/2` probe: a puddle
        // in the feet row still blocks while the mid tile stays dry.
        let feet = ((p.y + p.height) / TILE_PX) as i32;
        let col = ((p.x + WIDTH / 2.0) / TILE_PX) as i32;
        let i = w.index(col, feet);
        w.tiles[i].liquid = 255;
        assert!(!p.wet, "the body itself is dry");
        assert!(!p.can_mount(&w, 56), "the wet probe reaches the feet row");
        w.tiles[i].liquid = 0;
        assert!(p.can_mount(&w, 56));
        // A live grapple refuses the ordinary mounts but not the carts or the
        // CanUseHooks set.
        p.grappling_active = true;
        assert!(!p.can_mount(&w, 0), "Rudolph refuses a live grapple");
        assert!(p.can_mount(&w, 6), "the minecart keeps the grapple");
        assert!(p.can_mount(&w, 13), "the wood cart keeps it");
        assert!(p.can_mount(&w, 53), "the terra fart cart keeps it");
        assert!(crate::mount::can_use_hooks(54));
        assert!(crate::mount::can_use_hooks(57) && crate::mount::can_use_hooks(60));
        assert!(!crate::mount::can_use_hooks(56));
        assert!(!crate::mount::is_cart(56));
    }

    #[test]
    fn s1_l_8_fishron_halves_run_stats_outside_the_special() {
        // `Player.cs:26655-26659` with the getter at `:4044-4056`.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.set_mount(12, false); // Cute Fishron
        p.hp = p.max_hp;
        p.mount_fishron_special_counter = 0.0;
        p.update_mount(&w, false, None);
        let row = crate::mount::mount_data(12).unwrap();
        assert_eq!(p.max_run_speed_value(), row.run_speed / 2.0);
        assert_eq!(p.run_acceleration, row.acceleration / 2.0);
        // Hurt (`hp <= statLifeMax2 / 2`) arms the 60-tick window: no halving.
        p.hp = p.max_hp / 2;
        p.update_mount(&w, false, None);
        assert_eq!(p.mount_fishron_special_counter, 60.0);
        assert_eq!(p.max_run_speed_value(), row.run_speed);
        // The window survives the tick's one-point decay (`:25625-25628`).
        p.update_with_grapple(&w, PlayerInput::default(), &[], false, false, false, None, &mut tr_core::UnifiedRandom::default());
        assert!(p.mount_fishron_special_counter > 0.0);
        // The full tick leaves the mount in its air frame; the `+13.5` arm
        // (`Mount.cs:461-464`) would mask the halving probe, so put it back
        // on the ground state the wading case runs in.
        p.mount
            .test_set_frame_state(crate::mount::FRAME_STANDING);
        // Wading re-arms the 420-tick window (`Mount.cs:5005-5008`).
        p.mount_fishron_special_counter = 0.0;
        p.hp = p.max_hp;
        p.wet = true;
        p.update_mount(&w, false, None);
        assert_eq!(p.mount_fishron_special_counter, 420.0);
        assert_eq!(p.max_run_speed_value(), row.run_speed);
    }

    #[test]
    fn the_tr_mount_pin_equips_and_a_real_saddle_wins() {
        // DW-3.5 — the pin re-applies after the flag clear; a real mount
        // (or its buff) suppresses it, matching the refresh order.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..60 {
            p.update(&w, PlayerInput::default());
        }
        p.pinned_mount = Some(10);
        p.update(&w, PlayerInput::default());
        assert!(
            p.mount.is_active() && p.mount.kind() == 10,
            "the pin equipped"
        );
        assert!(
            p.buffs.find_buff_index(162).is_some(),
            "the buff sustains it"
        );
        // a real saddle overrides the pin while it is live
        p.set_mount(3, false);
        p.update(&w, PlayerInput::default());
        assert_eq!(p.mount.kind(), 3, "the real saddle wins");
        // removing the saddle hands control back to the pin
        let i = p.buffs.find_buff_index(130).unwrap();
        p.buffs.del_buff(i);
        p.update(&w, PlayerInput::default());
        assert_eq!(p.mount.kind(), 10, "the pin re-equipped after the clear");
    }

    // -- M2-MOUNT-1 phase 4: the frame animator ------------------------------

    #[test]
    fn the_queen_slime_run_counter_advances_at_half_speed() {
        // DW-4.1 — `Mount.cs:4277-4279` (type 50: `|vx| * 0.5`) with the
        // 16-tick delay and the 5-frame wrap (`:2194-2244` row).
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.set_mount(50, false);
        p.vx = 4.0;
        // 2.0 per tick, one crossing per tick — 8 ticks per frame; the first
        // crossing lands on tick 9, the second on 17.
        for _ in 0..17 {
            p.update_frames(&w);
        }
        assert_eq!(p.mount.frame_state(), crate::mount::FRAME_RUNNING);
        assert_eq!(p.mount.frame(), 2, "two steps in 17 ticks");
        // crossings at 25/33/41: the fourth step exits the 5-frame band...
        for _ in 0..(8 * 3) {
            p.update_frames(&w);
        }
        assert_eq!(p.mount.frame(), 0, "wrapped back into the running band");
    }

    #[test]
    fn the_rudolph_idle_pick_is_deterministic_and_reaches_the_idle_band() {
        // DW-4.2 — the idle timer (`Mount.cs:4145-4181`) through the
        // splitmix stand-in: same tick sequence, same picks.
        let w = ground_world();
        let mut a = Player::spawn(&w);
        let mut b = Player::spawn(&w);
        a.set_mount(0, false);
        b.set_mount(0, false);
        let mut reached_idle = false;
        for tick in 0..2500 {
            a.update_frames(&w);
            b.update_frames(&w);
            assert_eq!(a.mount.frame(), b.mount.frame(), "tick {tick}: same pick");
            assert!(a.mount.frame() != 1, "standing still never runs");
            if a.mount.frame() >= 2 && a.mount.frame() <= 5 {
                reached_idle = true;
            }
        }
        assert!(reached_idle, "the idle band (frames 2-5) was reached");
    }

    #[test]
    fn the_air_state_picks_the_in_air_row_and_rudolph_reaches_fly_frames() {
        // DW-4.3 — `Player.cs:36155-36216`: the in-air row right after the
        // jump, then the flying band while the budget lasts.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..60 {
            p.update(&w, PlayerInput::default());
        }
        p.set_mount(1, false); // Bunny: in-air start row 5
        let press = PlayerInput {
            jump: true,
            ..Default::default()
        };
        p.update(&w, press);
        let release = PlayerInput::default();
        p.update(&w, release);
        assert_eq!(p.mount.frame_state(), crate::mount::FRAME_IN_AIR);
        assert_eq!(p.mount.frame(), 5, "the bunny's in-air row");
        // Rudolph: the ground refill arms the budget, the held jump after the
        // jump budget runs out switches to the flying band (start row 6).
        p.dismount(&w, false);
        p.set_mount(0, false);
        for _ in 0..240 {
            p.update(&w, release); // settle back down; rest ticks refill
            if p.on_ground && p.mount.fly_time() > 0 {
                break;
            }
        }
        assert!(p.on_ground && p.mount.fly_time() > 0, "the refill armed");
        p.update(&w, press); // the ground jump
        for _ in 0..30 {
            p.update(&w, press); // hold: the jump budget (17) runs out
        }
        assert_eq!(
            p.mount.frame_state(),
            crate::mount::FRAME_FLYING,
            "fly time drives state 3"
        );
        assert!((6..=11).contains(&p.mount.frame()), "the flying band");
    }

    #[test]
    fn the_walking_grace_stays_inert_for_the_ground_rows() {
        // DW-4.4 — `Mount.cs:3620-3631`: every ground row carries
        // `walkingGraceTimeMax = 0`, so the grace never blocks the air state
        // and never goes negative.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..60 {
            p.update(&w, PlayerInput::default());
        }
        p.set_mount(1, false);
        let press = PlayerInput {
            jump: true,
            ..Default::default()
        };
        p.update(&w, press);
        let mut airborne_ticks = 0;
        for _ in 0..40 {
            p.update(&w, press);
            assert_eq!(p.mount.running_grace_time(), 0, "max 0 -> always 0");
            if p.vy != 0.0 {
                airborne_ticks += 1;
                assert_eq!(
                    p.mount.frame_state(),
                    crate::mount::FRAME_IN_AIR,
                    "the grace never blocks the air state"
                );
            }
        }
        assert!(airborne_ticks > 10, "the hop stayed airborne");
    }

    /// A 40x120 world with a dirt floor at y 20 — the sit/sleep rigs (the
    /// height clears `SolidCollision`'s `maxTilesY - 40` bottom clamp).
    fn sit_world() -> World {
        let mut w = World::new("t".into(), 40, 120);
        w.frame_important = vec![false; tr_core::tiledata::TILE_COUNT];
        for x in 0..40 {
            let i = w.index(x, 20);
            w.tiles[i] = tr_core::tile::Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
        w.compute_column_surface();
        w
    }

    #[test]
    fn the_sit_state_exits_on_a_facing_flip_or_a_gone_seat() {
        // DW-1.1 — `PlayerSittingHelper.UpdateSitting` (`:50-87`): the seat
        // re-decode at `Bottom + (0, -2)` (`:56-61`), the input mask
        // (`:62-65` — `controlUp` rides the port's merged jump binding) and
        // the facing mismatch stand the player up.
        let mut w = sit_world();
        // The chair's BOTTOM cell at (10, 19) — one row above the floor —
        // with `frameY % 40 == 18`: the decode reads the row above and
        // anchors the seat to this cell's bottom edge — `(168 - 6, 320)`
        // for the left-facing frame.
        let i = w.index(10, 19);
        w.tiles[i] = tr_core::tile::Tile {
            kind: 15,
            frame_y: 18,
            ..Default::default()
        };
        w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        let mut p = Player::spawn(&w);
        p.sit_down(&w, 162.0, 320.0, -1, (0.0, 0.0));
        assert!(p.sitting);
        // The seat re-decode still resolves (the mask holds) — the state
        // survives a quiet tick.
        p.update_sitting(&w);
        assert!(p.sitting);
        // The facing mismatch (`targetDirection != player.direction`).
        p.facing = 1;
        p.update_sitting(&w);
        assert!(!p.sitting);
        // The seat tile going away fails the decode → stand up.
        p.sit_down(&w, 162.0, 320.0, -1, (0.0, 0.0));
        assert!(p.sitting);
        w.tiles[i].flags &= !tr_core::tile::FLAG_ACTIVE;
        p.update_sitting(&w);
        assert!(!p.sitting);
    }

    #[test]
    fn the_sleep_state_arms_the_rotation_and_wakes_on_input() {
        // DW-1.5 — `PlayerSleepingHelper`'s `StartSleeping` (`:155-190`),
        // `UpdateState` (`:85-138`) and `StopSleeping` (`:140-153`): the
        // anchor snap, the π/2 rotation (`:73-74`), the fully-asleep counter
        // with the ActUp resets (`:45-64`) and the input wake (`:103-106`).
        let mut w = sit_world();
        // The bed's BOTTOM cell at (10, 19) — one row above the floor —
        // style 0 (`frameY 18`): the anchor is column `10 - 18/18 + 1 = 10`,
        // row `19` → `(168, 320)`.
        let bed = w.index(10, 19);
        w.tiles[bed] = tr_core::tile::Tile {
            kind: 79,
            frame_y: 18,
            ..Default::default()
        };
        w.tiles[bed].flags |= tr_core::tile::FLAG_ACTIVE;
        let mut p = Player::spawn(&w);
        p.sleep_down(&w, 168.0, 320.0, -1, 0);
        assert!(p.sleeping);
        assert_eq!(p.sleeping_style, 0);
        assert_eq!(p.facing, -1);
        assert_eq!(p.vx, 0.0);
        assert_eq!(p.vy, 0.0);
        // `fullRotation = PI/2 * (-direction)` = +π/2, origin `Size / 2`.
        assert!((p.full_rotation - std::f32::consts::FRAC_PI_2).abs() < 1e-6);
        assert_eq!(p.full_rotation_origin, (10.0, 21.0));
        // The counter climbs while quiet and resets on an item swing.
        p.update_sleeping(&w, false, false);
        assert_eq!(p.sleep_time, 1);
        p.update_sleeping(&w, true, false);
        assert_eq!(p.sleep_time, 0);
        for _ in 0..120 {
            p.update_sleeping(&w, false, false);
        }
        assert_eq!(p.sleep_time, 120);
        // The blood-moon night keeps resetting the counter (`:51-54`).
        w.blood_moon = true;
        w.day_time = false;
        p.update_sleeping(&w, false, false);
        assert_eq!(p.sleep_time, 0);
        assert!(p.sleeping);
        w.blood_moon = false;
        // The input mask wakes (`:103-106`).
        p.control_jump = true;
        p.update_sleeping(&w, false, false);
        assert!(!p.sleeping);
        assert_eq!(p.sleep_time, 0);
        assert_eq!(p.full_rotation, 0.0);
        // The bed going away wakes too (`:97-102`).
        p.sleep_down(&w, 168.0, 320.0, -1, 0);
        assert!(p.sleeping);
        w.tiles[bed].flags &= !tr_core::tile::FLAG_ACTIVE;
        p.update_sleeping(&w, false, false);
        assert!(!p.sleeping);
    }

    /// A minimal valid NPC room (the `housing` tests' shape): the wood
    /// frame, the background walls everywhere, the interior air ≥ the
    /// 60-tile room minimum.
    fn room_box(w: &mut World, x0: i32, y0: i32, rw: i32, rh: i32) {
        for x in x0..x0 + rw {
            for y in y0..y0 + rh {
                let i = w.index(x, y);
                w.tiles[i].wall = 27;
                let border = x == x0 || x == x0 + rw - 1 || y == y0 || y == y0 + rh - 1;
                if border {
                    w.tiles[i] = tr_core::tile::Tile {
                        kind: 9,
                        frame_x: -1,
                        frame_y: -1,
                        ..Default::default()
                    };
                    w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
                }
            }
        }
    }

    #[test]
    fn the_load_rebuilds_the_spawn_from_a_valid_bed() {
        // DW-3.2 — `FindSpawn` (`Player.cs:54592-54609`) over the port's
        // world-side bed spawn: a validated bed cell overrides the world
        // spawn; a destroyed bed falls back (the silent load-time arm).
        let mut w = sit_world();
        // The room sits on the floor row (20): the frame rows 12..20, the
        // interior 13..19. The bed's bottom cell rides (20, 19) and the
        // spawn cell = (20, 20) — the box floor.
        room_box(&mut w, 14, 12, 12, 9);
        let bed = w.index(20, 19);
        w.tiles[bed] = tr_core::tile::Tile {
            kind: 79,
            frame_y: 18,
            ..Default::default()
        };
        w.tiles[bed].flags |= tr_core::tile::FLAG_ACTIVE;
        w.bed_spawn = Some((20, 20));
        let p = Player::spawn(&w);
        assert_eq!(p.spawn_x, 20.0 * TILE_PX + 8.0 - WIDTH * 0.5);
        assert_eq!(p.spawn_y, 20.0 * TILE_PX - HEIGHT);
        // The bed gone → the world spawn (`SpawnX = -1`'s fallback).
        w.tiles[bed].flags &= !tr_core::tile::FLAG_ACTIVE;
        let p = Player::spawn(&w);
        assert_eq!(p.spawn_x, w.spawn_x as f32 * TILE_PX + 8.0 - WIDTH * 0.5);
    }

    #[test]
    fn the_snap_refuses_a_solid_box_and_respawn_clears_the_states() {
        // `Player.CanSnapToPosition` (`Player.cs:33401-33403`) and `Spawn`'s
        // `StopVanityActions` (`:37556`) via `respawn`.
        let mut w = sit_world();
        let mut p = Player::spawn(&w);
        p.x = 160.0;
        p.y = 278.0; // the box spans rows 17..20 at column 10 — all air
        assert!(p.can_snap_to(&w, 0.0, 0.0));
        let i = w.index(10, 19);
        w.tiles[i] = tr_core::tile::Tile {
            kind: 1,
            ..Default::default()
        };
        w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        assert!(!p.can_snap_to(&w, 0.0, 0.0));
        // The respawn clears the sit/sleep state (`Player.cs:37556`).
        p.sitting = true;
        p.sleeping = true;
        p.full_rotation = 1.5;
        p.respawn();
        assert!(!p.sitting);
        assert!(!p.sleeping);
        assert_eq!(p.full_rotation, 0.0);
    }
    // ---- S1-C-1: the dodge family (`Player.cs:38214-38241` + the fold) ----

    /// The scripted stream the dodge-roll tests drive: `rolls` feed
    /// `next_range` in order (exhausted -> the largest in-range value, which
    /// never rolls `== 0`), `draws` records every call so the S1-A-10
    /// draw-order contract is asserted directly.
    struct DodgeRng {
        rolls: Vec<i32>,
        draws: Vec<(i32, i32)>,
    }
    impl DodgeRng {
        fn with_rolls(rolls: &[i32]) -> Self {
            Self {
                rolls: rolls.to_vec(),
                draws: Vec::new(),
            }
        }
    }
    impl tr_core::loot::DropRng for DodgeRng {
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            self.draws.push((min, max));
            if self.rolls.is_empty() {
                return max - 1;
            }
            self.rolls.remove(0)
        }
        fn next_float(&mut self) -> f32 {
            0.5
        }
    }

    /// A ground player with `items` in the accessory slots, the fold run
    /// twice (`player_with_accessory`'s multi-source form).
    fn dodge_player_with(items: &[u16]) -> Player {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for (i, &kind) in items.iter().enumerate() {
            p.equipped[3 + i] = kind;
        }
        for _ in 0..2 {
            p.update(&w, PlayerInput::default());
        }
        p
    }

    /// Runs the dodge window out the way `UpdateImmunity` does in the C# —
    /// the dodge's `SetImmuneTimeForAllTypes` writes `immune`/`immuneTime`
    /// (R-03-5's ported fields, now gating `hurt_inner`'s General flag), so a
    /// next hit in these tests needs the expired window, not just a cleared
    /// `hurt_cd`.
    fn expire_dodge_window(p: &mut Player) {
        p.immune = false;
        p.immune_time = 0;
        p.hurt_cd = 0.0;
    }

    #[test]
    fn s1_c_1_dw_1_1_dodge_flags_arm_from_items_and_buff_and_clear() {
        // `ApplyEquipFunctional`: 963 (`:15060`) and 984 (`:15064`) arm
        // `blackBelt`, 6189 (`:14633`) arms `mysticSashDodge`, 3223
        // (`:15087`) arms the brain; `ResetEffects` clears them per frame
        // (`:18661-18662`/`:18617`) and the fold re-arms from the live
        // sources.
        let p = dodge_player_with(&[963]);
        assert!(p.black_belt);
        let p = dodge_player_with(&[984]);
        assert!(p.black_belt, "the Master Ninja Gear arms the same flag");
        let p = dodge_player_with(&[6189]);
        assert!(p.mystic_sash_dodge);
        let p = dodge_player_with(&[3223]);
        assert!(p.brain_of_confusion);
        let p = dodge_player_with(&[0]);
        assert!(!p.black_belt && !p.mystic_sash_dodge && !p.brain_of_confusion);
        // Buff 59 arms `shadowDodge` through the buff scan
        // (`UpdateBuffs`, `:10075-10077`).
        let w = ground_world();
        let mut p = Player::spawn(&w);
        assert!(!p.shadow_dodge);
        p.buffs.add_buff(59, 1800);
        p.update(&w, PlayerInput::default());
        assert!(p.shadow_dodge, "buff 59 arms the shadow dodge");
        let i = p.buffs.find_buff_index(59).unwrap();
        p.buffs.del_buff(i);
        p.update(&w, PlayerInput::default());
        assert!(!p.shadow_dodge, "the fold clears it without the buff");
        // The fold-order half: armed while equipped, cleared by the next
        // fold without the source.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.equipped[3] = 963;
        p.update(&w, PlayerInput::default());
        assert!(p.black_belt);
        p.equipped[3] = 0;
        p.update(&w, PlayerInput::default());
        assert!(!p.black_belt, "the next fold without the source clears it");
    }

    #[test]
    fn s1_c_1_dw_1_2_sash_roll_dodges_on_next_10_zero() {
        // `mysticSashDodge && Main.rand.Next(10) == 0` (`:38215-38219`).
        let mut p = dodge_player_with(&[6189]);
        let hp = p.hp;
        let mut rng = DodgeRng::with_rolls(&[0]);
        assert!(!p.hurt(20, p.x + 100.0, &mut rng, true), "the sash dodge fires");
        assert_eq!(p.hp, hp, "no damage on the dodge");
        assert_eq!(rng.draws, vec![(0, 10)]);
        // A missed roll lands the hit through the same source.
        expire_dodge_window(&mut p);
        let mut rng = DodgeRng::with_rolls(&[1]);
        assert!(p.hurt(20, p.x + 100.0, &mut rng, true));
        assert!(p.hp < hp);
    }

    #[test]
    fn s1_c_1_dw_1_2_belt_roll_dodges_on_next_10_zero() {
        // `blackBelt && Main.rand.Next(10) == 0` -> `NinjaDodge()`
        // (`:38220-38223`).
        let mut p = dodge_player_with(&[963]);
        let hp = p.hp;
        let mut rng = DodgeRng::with_rolls(&[0]);
        assert!(!p.hurt(20, p.x + 100.0, &mut rng, true), "the belt dodge fires");
        assert_eq!(p.hp, hp);
        assert_eq!(rng.draws, vec![(0, 10)]);
        expire_dodge_window(&mut p);
        let mut rng = DodgeRng::with_rolls(&[1]);
        assert!(p.hurt(20, p.x + 100.0, &mut rng, true));
        assert!(p.hp < hp);
    }

    #[test]
    fn s1_c_1_dw_1_2_brain_roll_gates_on_armed_flag_and_buff_321() {
        // `brainOfConfusionItem != null && !IsAir && Main.rand.Next(6) == 0
        // && FindBuffIndex(321) == -1` (`:38224-38227`): the armed flag folds
        // the two item checks into one bool, and the roll PRECEDES the
        // buff-321 check in the C# `&&` chain — the draw happens while armed
        // even with 321 active, the buff only gates the dodge itself.
        let mut p = dodge_player_with(&[3223]);
        let mut rng = DodgeRng::with_rolls(&[0]);
        assert!(!p.hurt(20, p.x + 100.0, &mut rng, true), "the brain dodge fires");
        assert_eq!(rng.draws, vec![(0, 6)]);
        // The dodge armed buff 321 itself (240 ticks): the next hit roll is
        // still drawn, but no dodge comes out of it.
        assert!(p.buffs.find_buff_index(321).is_some());
        expire_dodge_window(&mut p);
        let hp = p.hp;
        let mut rng = DodgeRng::with_rolls(&[0]);
        assert!(p.hurt(20, p.x + 100.0, &mut rng, true), "buff 321 gates the dodge");
        assert_eq!(rng.draws, vec![(0, 6)], "the draw happened anyway (C# order)");
        assert!(p.hp < hp);
        // Unarmed: a would-be-hitting roll draws nothing at all.
        let mut p = dodge_player_with(&[0]);
        let mut rng = DodgeRng::with_rolls(&[0]);
        assert!(p.hurt(20, p.x + 100.0, &mut rng, true));
        assert!(rng.draws.is_empty(), "no brain draw without the item");
    }

    #[test]
    fn s1_c_1_dw_1_2_shadow_dodges_without_a_roll() {
        // `shadowDodge -> ShadowDodge()` is deterministic (`:38229-38232`) —
        // no `Next` draw at all.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.buffs.add_buff(59, 1800);
        p.update(&w, PlayerInput::default());
        assert!(p.shadow_dodge);
        let hp = p.hp;
        let mut rng = DodgeRng::with_rolls(&[]);
        assert!(!p.hurt(20, p.x + 100.0, &mut rng, true));
        assert!(rng.draws.is_empty(), "the shadow dodge draws nothing");
        assert_eq!(p.hp, hp);
    }

    #[test]
    fn s1_c_1_dw_1_6_draw_order_sash_belt_brain_when_nothing_hits() {
        // The S1-A-10 draw-order contract, asserted on the recorded stream:
        // all three rolled sources armed + a stream that never hits draws
        // exactly sash -> belt -> brain, one `Next` each, in that order
        // (`:38215-38227`); shadow (not armed here) draws nothing.
        let mut p = dodge_player_with(&[6189, 963, 3223]);
        let mut rng = DodgeRng::with_rolls(&[]);
        assert!(p.hurt(20, p.x + 100.0, &mut rng, true), "no dodge, the hit lands");
        assert_eq!(rng.draws, vec![(0, 10), (0, 10), (0, 6)]);
    }

    #[test]
    fn s1_c_1_dw_1_6_first_source_hit_consumes_no_later_draw() {
        // The early return short-circuits: a sash hit consumes no belt/brain
        // draw (`:38217-38218`).
        let mut p = dodge_player_with(&[6189, 963, 3223]);
        let mut rng = DodgeRng::with_rolls(&[0]);
        assert!(!p.hurt(20, p.x + 100.0, &mut rng, true));
        assert_eq!(rng.draws, vec![(0, 10)]);
    }

    #[test]
    fn s1_c_1_dw_1_3_dodge_sets_the_80_tick_window_and_leaves_hp() {
        // The handlers' `SetImmuneTimeForAllTypes(longInvince ? 120 : 80)`
        // (`:38013-38021`, `:38025` et al.) — the dodge pair writes the
        // collapsed General cooldown directly, NOT `iframes_for`'s 40/20
        // normal-hurt window.
        let mut p = dodge_player_with(&[963]);
        let hp = p.hp;
        let mut rng = DodgeRng::with_rolls(&[0]);
        assert!(!p.hurt(50, p.x + 100.0, &mut rng, true));
        assert_eq!(p.hp, hp);
        assert!((p.hurt_cd - 80.0 / 60.0).abs() < 1e-4, "the 80-tick dodge window");
        // `longInvince` lifts the pair to 120.
        expire_dodge_window(&mut p);
        p.long_invince = true;
        let mut rng = DodgeRng::with_rolls(&[0]);
        assert!(!p.hurt(50, p.x + 100.0, &mut rng, true));
        assert!((p.hurt_cd - 120.0 / 60.0).abs() < 1e-4, "the 120-tick window");
    }

    #[test]
    fn s1_c_1_dw_1_3_brain_dodge_bookkeeping_runs_exactly_once() {
        // `BrainOfConfusionDodge` (`:38046-38055`): the 300 counter, ONE
        // buff-321 add (240 ticks), the 120/80 window.
        let mut p = dodge_player_with(&[3223]);
        let mut rng = DodgeRng::with_rolls(&[0]);
        assert!(!p.hurt(20, p.x + 100.0, &mut rng, true));
        assert_eq!(p.brain_of_confusion_dodge_animation_counter, 300);
        let count = p
            .buffs
            .active_types()
            .iter()
            .filter(|&&b| b == 321)
            .count();
        assert_eq!(count, 1, "one buff-321 add");
        let i = p.buffs.find_buff_index(321).unwrap();
        assert_eq!(p.buffs.buff_time[i], 240);
        assert!((p.hurt_cd - 80.0 / 60.0).abs() < 1e-4);
    }

    #[test]
    fn s1_c_1_dw_1_3_shadow_dodge_bookkeeping_runs_exactly_once() {
        // `ShadowDodge` (`:38023-38042`): buff 59 deleted, the Hallowed
        // cooldown set to 1800, the 120/80 window.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.buffs.add_buff(59, 1800);
        p.update(&w, PlayerInput::default());
        assert!(p.shadow_dodge);
        let mut rng = DodgeRng::with_rolls(&[]);
        assert!(!p.hurt(20, p.x + 100.0, &mut rng, true));
        assert_eq!(p.buffs.find_buff_index(59), None, "buff 59 deleted");
        assert_eq!(p.shadow_dodge_timer, 1800);
        assert!((p.hurt_cd - 80.0 / 60.0).abs() < 1e-4);
    }

    #[test]
    fn s1_c_1_dw_1_4_no_draw_no_dodge_while_hurt_cd_live() {
        // The cooldown early-return (`:38207-38213`) precedes the roll block:
        // with `hurt_cd` live no `Next` is drawn and no dodge fires (the C#
        // `flag` gate). Dirty test: the hitting stream would dodge if the
        // block ran.
        let mut p = dodge_player_with(&[963]);
        p.hurt_cd = 1.0;
        let hp = p.hp;
        let mut rng = DodgeRng::with_rolls(&[0]);
        assert!(!p.hurt(20, p.x + 100.0, &mut rng, true));
        assert!(rng.draws.is_empty(), "no draw while the cooldown is live");
        assert_eq!(p.hp, hp);
        assert!((p.hurt_cd - 1.0).abs() < 1e-6, "the cooldown untouched");
    }

    #[test]
    fn s1_c_1_dw_1_5_dodge_short_circuits_before_defense_endurance_and_iframes() {
        // The roll block sits after the cooldown gate and before the defense
        // reduction (`Player.cs:38214` > `:38304`): a dodged lethal hit
        // leaves hp/regen/knockback untouched and carries the DODGE window
        // (80/60), not `iframes_for`'s 40/60 — the early return proves the
        // defense/endurance/immunity steps never ran.
        let mut p = dodge_player_with(&[963]);
        p.hp = 5;
        p.endurance = 0.5;
        p.stat_defense = 100;
        let regen_time = p.regen.life_regen_time;
        let mut rng = DodgeRng::with_rolls(&[0]);
        assert!(
            !p.hurt(50, p.x + 100.0, &mut rng, true),
            "the belt eats the lethal hit"
        );
        assert_eq!(p.hp, 5, "no damage, no death pipeline");
        assert!(
            (p.hurt_cd - 80.0 / 60.0).abs() < 1e-4,
            "the dodge window, not the 40-tick hurt window"
        );
        assert_eq!(
            p.regen.life_regen_time, regen_time,
            "the regen reset did not run"
        );
        assert_eq!(p.vx, 0.0, "no knockback on the dodge");
    }

    #[test]
    fn s1_c_1_godmode_returns_before_the_dodge_rolls() {
        // `creativeGodMode` returns before the roll block (`:38205-38207`) —
        // an armed belt draws nothing under the power.
        let mut p = dodge_player_with(&[963]);
        p.creative_god_mode = true;
        let mut rng = DodgeRng::with_rolls(&[0]);
        assert!(!p.hurt(20, p.x + 100.0, &mut rng, true));
        assert!(rng.draws.is_empty(), "godmode precedes the rolls");
        assert_eq!(p.hurt_cd, 0.0, "no immunity window written");
    }

    #[test]
    fn s1_c_1_no_rolls_when_dodgeable_is_false() {
        // `Hurt`'s `dodgeable` parameter gates the whole block (`:38214`):
        // false -> no draw, no dodge. Phase 2 threads the real
        // `IsDamageDodgeable` values through this parameter.
        let mut p = dodge_player_with(&[963]);
        let hp = p.hp;
        let mut rng = DodgeRng::with_rolls(&[0]);
        assert!(p.hurt(20, p.x + 100.0, &mut rng, false));
        assert!(rng.draws.is_empty());
        assert!(p.hp < hp);
    }

    #[test]
    fn s1_c_1_a_dodged_lava_hit_skips_the_on_fire_grant() {
        // The lava site (`update_survival`, `Player.cs:27681-27722`): the
        // OnFire grant rides the `Hurt` return (`:27710-27711`) — a dodge
        // returns false, so no buff 24; the counterpart roll lands and
        // grants, proving the UpdatePlayers stream reaches the lava hurt.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.equipped[3] = 963;
        for _ in 0..2 {
            p.update(&w, PlayerInput::default());
        }
        p.in_lava = true;
        p.lava_time = 0;
        let hp = p.hp;
        let mut rng = DodgeRng::with_rolls(&[0]);
        p.update_survival(&mut rng);
        assert_eq!(p.hp, hp, "the belt ate the lava hit");
        assert!(p.buffs.find_buff_index(24).is_none(), "no OnFire on a dodge");
        // The counterpart: a missed roll lands the 80 damage and grants.
        expire_dodge_window(&mut p);
        let mut rng = DodgeRng::with_rolls(&[1]);
        p.update_survival(&mut rng);
        assert!(p.hp < hp);
        assert!(p.buffs.find_buff_index(24).is_some(), "OnFire on the landed hit");
    }

    #[test]
    fn s1_c_1_the_shadow_dodge_timer_decays_one_per_tick() {
        // `Player.Update`'s decay (`:25461-25464`), riding the tick head.
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.shadow_dodge_timer = 1800;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.shadow_dodge_timer, 1799);
        p.shadow_dodge_timer = 0;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.shadow_dodge_timer, 0, "no underflow");
    }

    #[test]
    fn s1_c_1_the_brain_counter_decays_and_clears_on_respawn() {
        // The fold decay (`ResetEffects`, `:18794-18797`) and `UpdateDead`'s
        // zero (`:16932`, the port's `respawn`).
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.brain_of_confusion_dodge_animation_counter = 300;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.brain_of_confusion_dodge_animation_counter, 299);
        p.respawn();
        assert_eq!(p.brain_of_confusion_dodge_animation_counter, 0);
    }
}

/// M4-PROJ-8 phase 2 — the summon keep-alive arms (`Player.cs:10483-10947`),
/// the recount caches (`:12184-12226`) and the buff-216 respawn arm, driven
/// on a bare Player + pool (the world_sim tick wires the same calls).
#[cfg(test)]
mod summon_tests {
    use super::*;
    use crate::item::EMPTY_SLOT;

    /// `Player.FindItem`'s stand-in: the inventory with the staff (4281) in
    /// slot 0 (the `ItemDef` verified this phase: shoot 759, damage 7,
    /// knockBack 4.0).
    fn staff_inventory() -> [ItemSlot; INV] {
        let mut inv = [EMPTY_SLOT; INV];
        inv[0] = ItemSlot {
            kind: 4281,
            name: registry::name(4281),
            count: 1,
            prefix: 0,
        };
        inv
    }

    fn empty_inventory() -> [ItemSlot; INV] {
        [EMPTY_SLOT; INV]
    }

    /// Deactivates a slot through the public key path (`key_of` +
    /// `kill_remote`, the `29` despawn) — the pool's `Kill` stand-in.
    fn kill_pool_slot(pool: &mut ProjectilePool, slot: usize) {
        let key = pool.key_of(slot);
        let (owner, slot, generation) = crate::net::projectile_key_parts(key);
        pool.kill_remote(slot, owner, generation);
    }

    /// DW-2.2 — buff 64 (`Slime`) tracks projectile 266 exactly: kept and
    /// refreshed to 18000 while the pool holds it, dropped on the first pass
    /// whose recount sees it gone (the C# reads the pool the previous frame's
    /// pool pass left, `Player.cs:25565` → the arms `:10521-10534`).
    #[test]
    fn test_DW_2_2_buff_64_tracks_projectile_266() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        let mut pool = ProjectilePool::new();
        let inv = empty_inventory();
        let minion = pool
            .spawn(266, 0, Vec2::new(200.0, 200.0), Vec2::ZERO, 10, 1.0, false)
            .unwrap();
        p.buffs.add_buff(64, 100);
        // The live order: `ResetProjectileCaches` (`:18810`, the ResetEffects
        // seam) then the recount (`:25565`) then the arms.
        p.reset_projectile_caches();
        p.update_projectile_caches(&pool);
        p.update_summon_buff_arms(&mut pool, &inv);
        assert!(p.slime, "266 in the pool re-affirms the flag");
        let b = p.buffs.find_buff_index(64).unwrap();
        assert_eq!(p.buffs.buff_time[b], 18000, "the arm pins the refresh");
        assert_eq!(p.owned_projectile_counts[266], 1);
        // The boundary: the minion dies between ticks (the pool pass's kill)
        // — the drop lands on exactly the next pass, not earlier.
        kill_pool_slot(&mut pool, minion);
        p.reset_projectile_caches();
        p.update_projectile_caches(&pool);
        p.update_summon_buff_arms(&mut pool, &inv);
        assert_eq!(
            p.buffs.find_buff_index(64),
            None,
            "the buff drops the tick the recount sees 266 gone"
        );
        // The flag lifecycle: the arm only SETS it — `ResetEffects` clears it
        // per tick (the full tick's seam, `:18622`). One real tick clears it.
        p.update(&w, PlayerInput::default());
        assert!(!p.slime, "the flag is only true while the arm re-affirms");
        // A buff with no minion ever spawns drops immediately.
        p.buffs.add_buff(64, 100);
        p.reset_projectile_caches();
        p.update_projectile_caches(&pool);
        p.update_summon_buff_arms(&mut pool, &inv);
        assert_eq!(p.buffs.find_buff_index(64), None);
    }

    /// DW-2.2 boundary variant — buff 216's baby-bird respawn arm
    /// (`Player.cs:10697-10726`): below cap with the staff spawns 759 from
    /// the item's shoot/damage/knockBack (+ `originalDamage = item.damage`,
    /// `:10713`); at cap it DelBuffs; without the staff it DelBuffs.
    #[test]
    fn test_DW_2_2_buff_216_respawn_arm() {
        let w = World::new("t".into(), 64, 800);
        let inv = staff_inventory();
        // Below cap: the respawn arm fires.
        let mut p = Player::spawn(&w);
        let mut pool = ProjectilePool::new();
        p.buffs.add_buff(216, 100);
        p.update_projectile_caches(&pool);
        p.update_summon_buff_arms(&mut pool, &inv);
        assert!(p.baby_bird, "the spawn re-affirms the flag");
        let bird = pool
            .slots()
            .iter()
            .find(|s| s.active && s.kind == 759)
            .expect("759 spawned from the staff");
        assert_eq!(bird.damage, 7);
        assert_eq!(bird.original_damage, 7, "Player.cs:10713");
        assert_eq!(bird.knock_back, 4.0);
        assert_eq!(bird.owner, 0);
        assert_eq!(bird.pos.x, p.x + WIDTH * 0.5);
        assert_eq!(bird.pos.y, p.y, "Entity.Top — the hitbox's top center");
        assert_eq!(bird.vel, Vec2::ZERO);
        // The verbatim 759 slice rides the data-row spawn seam (the phase-1
        // review fix): the 10×10 box and tile collide from the row, the
        // 18000 lifetime, the minion flag and its 1.0 slot budget.
        assert_eq!((bird.w, bird.h), (10.0, 10.0));
        assert!(bird.tile_collide);
        assert_eq!(bird.time_left, 18000);
        assert!(bird.minion);
        assert_eq!(bird.minion_slots, 1.0);
        let b = p.buffs.find_buff_index(216).unwrap();
        assert_eq!(p.buffs.buff_time[b], 18000);
        // At cap (`numMinions == maxMinions`): DelBuff, no spawn. The
        // counter is the pool pass's stand-in until Phase 3 writes it.
        let mut p2 = Player::spawn(&w);
        let mut pool2 = ProjectilePool::new();
        p2.num_minions = 1;
        p2.buffs.add_buff(216, 100);
        p2.update_projectile_caches(&pool2);
        p2.update_summon_buff_arms(&mut pool2, &inv);
        assert!(
            !pool2.slots().iter().any(|s| s.active && s.kind == 759),
            "at cap the arm spawns nothing"
        );
        assert_eq!(p2.buffs.find_buff_index(216), None, "Player.cs:10719-10722");
        // Below cap without the staff: DelBuff too.
        let mut p3 = Player::spawn(&w);
        let mut pool3 = ProjectilePool::new();
        p3.buffs.add_buff(216, 100);
        p3.update_projectile_caches(&pool3);
        p3.update_summon_buff_arms(&mut pool3, &empty_inventory());
        assert_eq!(p3.buffs.find_buff_index(216), None);
    }

    /// The C# `DelBuff(j); j--;` rescan (`:10483-10947`): a dropped arm's
    /// neighbour shifts into its slot and still gets its arm the same tick.
    #[test]
    fn test_edge_the_arms_rescan_after_delbuff() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        let mut pool = ProjectilePool::new();
        let inv = empty_inventory();
        pool.spawn(317, 0, Vec2::new(200.0, 200.0), Vec2::ZERO, 10, 1.0, false)
            .unwrap();
        // Buff 64 at slot 0 (no 266 — dropped), buff 83 at slot 1 (317 alive
        // — kept): after the drop, 83 shifts into slot 0 and is re-examined.
        p.buffs.add_buff(64, 100);
        p.buffs.add_buff(83, 100);
        p.update_projectile_caches(&pool);
        p.update_summon_buff_arms(&mut pool, &inv);
        assert_eq!(p.buffs.find_buff_index(64), None);
        let raven = p.buffs.find_buff_index(83).expect("the rescan kept 83");
        assert!(p.raven);
        assert_eq!(p.buffs.buff_time[raven], 18000);
    }

    /// The cap effects share the arms' slot-ordered chain
    /// (`Player.cs:10175-10183` before `:10706`): buff 110 in an earlier slot
    /// raises `maxMinions` before buff 216's arm reads it, so a minion count
    /// at the base cap still spawns.
    #[test]
    fn test_edge_cap_effects_are_slot_ordered_before_the_216_arm() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        let mut pool = ProjectilePool::new();
        let inv = staff_inventory();
        p.buffs.add_buff(110, 100); // slot 0 — the raise
        p.buffs.add_buff(216, 100); // slot 1 — the read
        p.num_minions = 1; // at the base cap of 1
        p.update_projectile_caches(&pool);
        p.update_summon_buff_arms(&mut pool, &inv);
        assert_eq!(p.max_minions, 2, "110 raised the cap");
        assert!(
            pool.slots().iter().any(|s| s.active && s.kind == 759),
            "216 read the raised cap and spawned"
        );
    }

    /// The chain's 29th arm — buff 161 (`DeadlySphereMinion`, kind 533,
    /// `Player.cs:10957-10972`, review fix): the plain keep-alive shape, kept
    /// and refreshed while 533 is owned, dropped the pass the recount sees
    /// it gone.
    #[test]
    fn test_edge_buff_161_tracks_projectile_533() {
        let w = World::new("t".into(), 64, 800);
        let mut p = Player::spawn(&w);
        let mut pool = ProjectilePool::new();
        let inv = empty_inventory();
        let sphere = pool
            .spawn(533, 0, Vec2::new(200.0, 200.0), Vec2::ZERO, 10, 1.0, false)
            .unwrap();
        p.buffs.add_buff(161, 100);
        p.reset_projectile_caches();
        p.update_projectile_caches(&pool);
        p.update_summon_buff_arms(&mut pool, &inv);
        assert!(
            p.deadly_sphere_minion,
            "533 in the pool re-affirms the flag"
        );
        let b = p.buffs.find_buff_index(161).unwrap();
        assert_eq!(p.buffs.buff_time[b], 18000);
        assert_eq!(p.owned_projectile_counts[533], 1);
        // The drop: the minion leaves the pool, the buff follows one pass
        // later — and the flag clears on the next tick's ResetEffects.
        kill_pool_slot(&mut pool, sphere);
        p.reset_projectile_caches();
        p.update_projectile_caches(&pool);
        p.update_summon_buff_arms(&mut pool, &inv);
        assert_eq!(p.buffs.find_buff_index(161), None);
        p.update(&w, PlayerInput::default());
        assert!(!p.deadly_sphere_minion);
    }
}
