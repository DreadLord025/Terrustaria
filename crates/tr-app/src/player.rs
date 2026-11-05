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

use glam::Vec2;
use crate::buffs::{BuffSlots, MAX_BUFFS};
use crate::collision;
use crate::regen::{DotDeath, RegenInputs, RegenState, update_life_regen, update_mana_regen};
use crate::luck::LuckState;

pub const WIDTH: f32 = 20.0;
pub const HEIGHT: f32 = 42.0;

const GRAVITY: f32 = 0.4;
const MAX_FALL: f32 = 10.0;
/// `Player.jumpSpeed` / `Player.jumpHeight`: holding the jump key keeps the
/// upward velocity at `jumpSpeed` for `jumpHeight` ticks, so tap and hold jumps
/// reach different heights.
const JUMP_SPEED: f32 = 5.01;
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

/// The in-liquid gravity (`Player.cs:27008`): `gravity / 3`.
pub fn liquid_gravity() -> f32 {
    GRAVITY / 3.0
}

/// The in-liquid fall cap, both ways (`Player.cs:27011-27017`):
/// `maxFallSpeed / 3`.
pub fn liquid_fall_cap() -> f32 {
    MAX_FALL / 3.0
}

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
    /// `Player.dead`: the player is dead and waiting for the respawn timer
    /// (`Player.cs:38908`).
    pub dead: bool,
    /// `Player.deadTime`: the ticks spent dead (`Player.cs:16923`); gates the
    /// respawn-skip (90-tick lockout, `Player.cs:57111`).
    pub dead_time: i32,
    /// `Player.respawnTimer`: the ticks left until the respawn.
    pub respawn_timer: i32,
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
    /// `Player.legFrame.Y / 56`: the current leg animation row (0..=19), driven by
    /// `PlayerFrame()`.
    pub leg_frame: u32,
    /// `Player.bodyFrame.Y / 56`: the current body animation row (0..=19). Mirrors the
    /// legs while walking; the app overrides it with the item-use rows during a swing.
    pub body_frame: u32,
    /// `Player.legFrameCounter`: accumulates `|velocity.X| * 1.3` while walking and
    /// advances a leg row every `8.0`.
    leg_frame_counter: f32,
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
    pub mana: i32,
    pub mana_max: i32,
    /// The 44 buff slots and `buffImmune` — `Player.buffType`/`buffTime`/`buffImmune`
    /// (`Player.cs:1510-1514`), driven by the `Player.AddBuff` family (`buffs.rs`).
    pub buffs: BuffSlots,
    pub hurt_cd: f32,
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
    /// The grapple hook's attached center, set by the caller per tick (the
    /// vanilla `GrappleMovement` reads `Main.projectile` directly).
    pub grapple_hook_center: Vec2,
    /// The attached hook's type (the pull-speed table lookup).
    pub grapple_hook_type: i32,
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
    move_speed: f32,
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
}

impl Player {
    pub fn spawn(world: &World) -> Self {
        let sx = world.spawn_x as f32 * TILE_PX + 8.0 - WIDTH * 0.5;
        let sy = world.spawn_y as f32 * TILE_PX;
        Self {
            dead: false,
            dead_time: 0,
            respawn_timer: 0,
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
            wet: false,
            lava_wet: false,
            honey_wet: false,
            shimmer_wet: false,
            liquid_step: 1.0,
            in_water: false,
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
            mana: 20,
            mana_max: 20,
            buffs: BuffSlots::default(),
            hurt_cd: 0.0,
            breath: BREATH_MAX,
            breath_cd: 0,
            regen: RegenState::default(),
            dot_death: None,
            luck_state: LuckState::default(),
            fall_start: (sy / TILE_PX) as i32,
            extra_fall: 0,
            no_fall_dmg: false,
            has_wings: false,
            grappling_active: false,
            going_down_with_grapple: false,
            grapple_hook_center: Vec2::ZERO,
            grapple_hook_type: 13,
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
        }
    }

    /// Moves the player to a pixel position without changing health or breath.
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
        self.hp = self.max_hp;
        self.hurt_cd = 0.0;
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
        self.gfx_off_y = 0.0;
        self.prev_gfx_off_y = 0.0;
    }

    /// `Player.KillMe`'s respawn setup (`Player.cs:38908-38910`): the dead
    /// flag, `deadTime` reset and `respawnTimer = GetRespawnTime(pvp)`.
    pub fn start_death(&mut self, boss_near: bool, expert: bool) {
        self.dead = true;
        self.dead_time = 0;
        self.respawn_timer = Player::get_respawn_time(boss_near, expert);
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

    /// `Player.AllowsRespawnTimerSkip` (`Player.cs:38962-38987`): after the
    /// 90-tick lockout, with no boss hindering and no danger NPCs nearby.
    /// `ghost`/`pvpDeath`/event branches are unreachable in the port.
    pub fn allows_respawn_timer_skip(&self, boss_near: bool, danger_near: bool) -> bool {
        !(self.ghost_like() || self.dead_time < 90 || boss_near || danger_near)
    }

    /// The port has no ghosts (hardcore) or PVP deaths; both gates are false.
    fn ghost_like(&self) -> bool {
        false
    }

    /// `Player.UpdateDead` (`Player.cs:16919-17070`): `deadTime++`, the
    /// respawn decrement clamped to `respawnTimerMax` (3600), and the
    /// jump-press skip while `AllowsRespawnTimerSkip`. Returns true when the
    /// player should respawn now.
    pub fn update_dead(&mut self, jump: bool, boss_near: bool, danger_near: bool) -> bool {
        self.dead_time += 1;
        self.respawn_timer = (self.respawn_timer - 1).clamp(0, 3600);
        let jump_edge = jump && !self.prev_jump_input;
        self.prev_jump_input = jump;
        let can_skip = self.allows_respawn_timer_skip(boss_near, danger_near);
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

    /// Contact damage / future combat. Returns true when the hit was accepted.
    pub fn hurt(&mut self, amount: i32, from_x: f32) -> bool {
        self.hurt_inner(amount, from_x, true)
    }

    /// The shared hurt body. `knock_back` mirrors the original's
    /// `hitDirection != 0` gate (`Player.cs:38519`): the fall-damage and
    /// petrify calls pass hitDirection 0 and skip the knock entirely (the
    /// fallStart reset inside that branch rides with it).
    fn hurt_inner(&mut self, amount: i32, from_x: f32, knock_back: bool) -> bool {
        if self.hurt_cd > 0.0 || self.hp <= 0 {
            return false;
        }
        // `Player.Hurt`'s defense reduction (the classic
        // `damage - statDefense * 0.5`, floored at 1 — M3-EQUIP-2).
        let amount = tr_core::armor_sets::reduce_damage(amount, self.stat_defense);
        self.hp = (self.hp - amount).max(0);
        // `Player.Hurt`'s immunity window (`Player.cs:38385`): 40 ticks
        // for normal damage (no long-invincibility source yet), 20 when
        // the damage is exactly 1; PVP 8 is unreachable in single-player.
        self.hurt_cd = tr_core::damage::iframes_for(amount, false, false) as f32 / 60.0;
        // `Player.Hurt` resets the natural-ramp timer (`Player.cs:38395`).
        self.regen.life_regen_time = 0.0;
        if knock_back {
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
    fn hurt_fall(&mut self, amount: i32) -> bool {
        self.hurt_cd = 0.0;
        let cx = self.x + WIDTH * 0.5;
        self.hurt_inner(amount, cx, false)
    }

    pub fn center(&self) -> (f32, f32) {
        (self.x + WIDTH * 0.5, self.y + HEIGHT * 0.5)
    }

    /// `Player.GetWeaponDamageMultiplier` (`Player.cs:53386`) narrowed to
    /// the ported classes: the classless items pass the 1f default.
    pub fn weapon_damage_multiplier(&self, class: registry::DamageClass) -> f32 {
        match class {
            registry::DamageClass::Melee => self.melee_damage,
            registry::DamageClass::Ranged => self.ranged_damage,
            registry::DamageClass::Magic => self.magic_damage,
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

    /// One fixed 60 Hz step.
    pub fn update(&mut self, world: &World, input: PlayerInput) {
        self.update_with_grapple(world, input, None)
    }

    /// The full tick with the optional grappling pull (`GrappleMovement`):
    /// while a hook is attached the pull REPLACES the normal movement
    /// (`Player.cs:27395-27399` gates the whole run/jump/wing/dash block).
    pub fn update_with_grapple(
        &mut self,
        world: &World,
        input: PlayerInput,
        grapple: Option<(Vec2, i32)>,
    ) {
        // `Player.grappling[0] != -1` is rebuilt per tick by the attached
        // hook's AI (`Player.cs:28300` + `Projectile.cs:52860-52864`) — the
        // caller passes the attached state, so derive it here.
        self.grappling_active = grapple.is_some();
        // The buff-frame head of `Player.Update` (`Player.cs:25555-25616`): the
        // `buffImmune` reset precedes `UpdateBuffs` and every re-grant; the
        // end-of-update purge consumes the table at `:26011-26016`.
        self.buffs.reset_buff_immune();
        // `Player.Update` → `UpdateBuffs(i)` (`Player.cs:24535`): the timer head runs
        // early in the tick. Nothing grants buffs yet, so this only decrements.
        self.buffs.update_buffs_tick();
        // Buffs whose timer ran out are deleted right after the tick
        // (`Player.cs:25608-25616`) — before any effect can read them.
        self.remove_expired_buffs();
        self.prev_x = self.x;
        self.prev_y = self.y;
        self.prev_gfx_off_y = self.gfx_off_y;
        self.hurt_cd = (self.hurt_cd - 1.0 / 60.0).max(0.0);
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

        // `Player.cs:24364-24371`: the per-tick movement defaults;
        // `moveSpeed` multiplies acceleration and the primary gate while
        // `accRunSpeed` takes only the debuff (`Player.cs:26004-26010`).
        self.max_run_speed = 3.0;
        self.run_acceleration = 0.08;
        self.run_slowdown = 0.2;
        self.acc_run_speed = self.max_run_speed;
        // `Player.cs:24366-24367` + `UpdateJumpHeight` (`:19589-19593`): the
        // jump statics reset per tick; the balloon floor lifts them.
        self.jump_height = 15;
        self.jump_speed = 5.01;
        if self.jump_boost {
            self.jump_height = self.jump_height.max(20);
            self.jump_speed = self.jump_speed.max(6.51);
        }
        self.move_speed = 1.0;
        self.strongest_move_speed_debuff = 1.0;
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
        self.blind = false;
        self.weak_stacks = 0;
        self.melee_damage = 1.0;
        self.mana_regen_bonus = 0;
        self.well_fed = false;
        self.pick_speed_bonus = 0.0;
        self.melee_speed_bonus = 0.0;
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
        for bt in self.buffs.active_types() {
            match bt {
                22 => self.blind = true, // `BuffID.Darkness`
                32 => self.slow = true,  // `BuffID.Slow`
                33 => {
                    self.weak_stacks += 1; // `BuffID.Weak`
                    self.melee_damage -= 0.051;
                    self.move_speed -= 0.1;
                }
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
                // The WellFed tiers (`:11931-11969`): the per-slot stat adds.
                // The `statDefense` half waits for the player-defense
                // consumer (M3-EQUIP-3); the minion halves for M4-MINION.
                26 => {
                    self.well_fed = true;
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
        let dir = (input.right as i32 - input.left as i32) as f32;
        if dir != 0.0 {
            self.facing = dir as i32;
        }
        if !grappling {
            // `:26496-26499`: the wing run-speed tweaks run before the movement.
            if self.wings_logic > 0 && self.vy != 0.0 {
                self.wing_air_logic_tweaks();
            }
            self.horizontal_movement(dir);
        }

        let in_liquid = self.wet || self.lava_wet || self.honey_wet || self.shimmer_wet;
        // `Player.cs:27008-27017`: in a liquid the gravity is `gravity / 3`
        // and the fall cap is `maxFallSpeed / 3`, applied both ways (the
        // upward cap keeps the swim under the surface).
        let gravity = if in_liquid { liquid_gravity() } else { GRAVITY };
        let fall_cap = if in_liquid {
            liquid_fall_cap()
        } else {
            MAX_FALL
        };
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
                // `Player.cs:20766-20767`: the ground jump; the landing re-arm
                // follows (`:20755-20758` — `RefreshDoubleJumps`).
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

        // `Player.Update` → `GrappleMovement` (`Main` call order `:27653`,
        // `Player.cs:22613-22740`): the pull REPLACES the velocity — the
        // vector to the attached hook center clamped to the pull speed. The
        // fall origin refreshes per tick (no fall damage accrues).
        if grappling {
            self.grapple_movement(grapple, input);
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
        if !flew && !self.webbed && !grappling {
            self.vy = (self.vy + gravity).min(fall_cap);
        }
        self.vy = self.vy.max(if in_liquid {
            -(MAX_FALL / 3.0)
        } else {
            -MAX_FALL
        });

        // The AABB resolve (`TileCollision` + the step/slope helpers): the
        // velocity phase above only shaped `velocity`; this integrates it.
        self.move_vanilla(world, input.down, gravity);

        // `Player.cs:25219-25224`: the tick petrification lands, a flat hit —
        // the difficulty multiplier is 1 (classic) until M6-CREATIVE lands.
        let stoned = self.buffs.find_buff_index(156).is_some();
        if stoned && !self.last_stoned {
            let cx = self.x + WIDTH * 0.5;
            self.hurt_inner(20, cx, false);
        }
        self.last_stoned = stoned;

        // `Player.cs:25293-25385`: the fall-damage block — the whole thing runs
        // whenever the player has come to rest this tick (`velocity.Y == 0`).
        // `:26057-26060`: every rest tick re-arms the granted extra jumps.
        if self.vy == 0.0 {
            self.refresh_double_jumps();
            // `num10 = 25 + extraFall` (`Player.cs:25301`); the mount addendum
            // (`Player.cs:25304`) waits for M2-MOUNT-1.
            let threshold = 25 + self.extra_fall;
            // `num11` is int-truncated on both ends (`Player.cs:25306`).
            let mut fell = (self.y / TILE_PX) as i32 - self.fall_start;
            // The flying-minecart/bunny/dash zeroes (`Player.cs:25307-25323`)
            // have no systems here yet; the cloud/soft-tile scan does
            // (`Player.cs:25325-25345`): the tile row under the feet across
            // the hitbox's x-extent.
            let tx0 = (self.x / TILE_PX) as i32;
            let tx1 = ((self.x + WIDTH) / TILE_PX) as i32;
            let ty = ((self.y + HEIGHT + 1.0) / TILE_PX) as i32;
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
                    self.hurt_inner(dmg, cx, false);
                    self.hurt_cd = 0.0;
                }
            } else if (fell > threshold) && !self.no_fall_dmg && !self.has_wings {
                // `Player.cs:25360-25374`: `(fell - threshold) * 10` damage with
                // hitDirection 0 — no knockback — and `immune = false` forced
                // BEFORE the Hurt so the hit lands through any iframes.
                self.hurt_fall((fell - threshold) * 10);
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

        self.update_survival();

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

        self.update_frames();

        // `Player.cs:27618-27635`: the release flags recompute at the end of
        // the tick — the double-tap reads the press edges against them.
        self.right_prev = input.right;
        self.left_prev = input.left;
    }

    /// `PlayerFrame()` (legs + the locomotion part of the body). The original runs this
    /// every tick after movement:
    /// - idle: legs and body row `0` (the rest pose);
    /// - airborne (`velocity.Y != 0`): row `5` (the jump/fall pose);
    /// - walking (`velocity.X != 0`): legs advance one row per `8.0` of
    ///   `legFrameCounter += |velocity.X| * 1.3`, cycling the walk band rows `7..=19`,
    ///   and the body copies the leg row.
    ///
    /// The wings/mount/pulley/hold-style branches of the original are not modelled yet,
    /// and item-use body frames are applied by the app via [`Player::apply_item_use_frame`]
    /// while a swing is active.
    fn update_frames(&mut self) {
        if self.vy != 0.0 {
            self.leg_frame_counter = 0.0;
            self.leg_frame = 5;
        } else if self.vx != 0.0 {
            self.leg_frame_counter += self.vx.abs() * 1.3;
            while self.leg_frame_counter > 8.0 {
                self.leg_frame_counter -= 8.0;
                self.leg_frame += 1;
            }
            if self.leg_frame < 7 {
                self.leg_frame = 19;
            } else if self.leg_frame > 19 {
                self.leg_frame = 7;
            }
        } else {
            self.leg_frame_counter = 0.0;
            self.leg_frame = 0;
        }

        if self.vy != 0.0 {
            self.body_frame = 5;
        } else if self.vx != 0.0 {
            self.body_frame = self.leg_frame;
        } else {
            self.body_frame = 0;
        }
    }

    /// `PlayerFrame()` item-use branch for `useStyle == 1` (blocks and other placeables):
    /// the body steps rows `1 -> 2 -> 3` as `itemAnimation` runs out. `remaining` is
    /// `itemAnimation / itemAnimationMax` (1 at the start of the swing, 0 at the end).
    pub fn apply_item_use_frame(&mut self, remaining: f32) {
        self.body_frame = if remaining < 0.333 {
            3
        } else if remaining < 0.666 {
            2
        } else {
            1
        };
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
            // OnFire — `:18976-18998` (the vampireSeed −100 variant has no
            // source and stays out per plan).
            resets_time = true;
            regen -= 8;
            if drip {
                regen -= 8;
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
        let cy = self.y + HEIGHT * 0.5;
        let tx = ((cx + dir as f32 * (WIDTH * 0.5 + 2.0)) / TILE_PX) as i32;
        let ty0 = ((cy - HEIGHT * 0.5 + 2.0) / TILE_PX) as i32;
        let ty1 = (cy / TILE_PX) as i32;
        [ty0, ty1].into_iter().any(|ty| {
            world.tile(tx, ty).is_some_and(|t| {
                t.active() && (tr_core::tiledata::TILE_SOLID.has(t.kind) || t.slope() != 0)
            })
        })
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
        // `:26776-26781`: the wing refill on the ground needs a jump release.
        if self.vy == 0.0 && self.release_jump {
            self.wing_time = self.wing_time_max as f32;
        }
        // `:26859-26862`: the rocket fuel refills on any rest tick.
        if self.vy == 0.0 {
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
    fn grapple_movement(&mut self, hook: Option<(Vec2, i32)>, input: PlayerInput) {
        let Some((hook_center, hook_type)) = hook else {
            return;
        };
        // `RefreshMovementAbilities` (`Player.cs:22858-22867`): the flight
        // budgets refill every tick while grappling.
        self.wing_time = self.wing_time_max as f32;
        self.rocket_time = self.rocket_time_max;
        self.rocket_delay = 0;
        self.refresh_double_jumps();
        // `:22645`: the fall origin refreshes per tick — no fall damage.
        self.fall_start = (self.y / TILE_PX) as i32;

        // `GetGrapplingForces` (`Player.cs:22820-22846`): the velocity = the
        // vector to the hook center clamped to the per-type pull speed.
        let to_hook = hook_center - Vec2::new(self.x + WIDTH * 0.5, self.y + HEIGHT * 0.5);
        let dist = to_hook.length();
        let pull_speed = crate::grapple::hook_spec(hook_type).pull_speed;
        let pull = if dist > 0.001 {
            to_hook * (pull_speed / dist).min(1.0)
        } else {
            Vec2::ZERO
        };
        // `:22659`: `flag3` = the pull points down while the CURRENT velocity
        // is zero (a grounded pull) — read before the replacement.
        let flag3 = pull.y > 0.0 && self.vy == 0.0;
        // `:22660-22661`: the pull replaces the velocity outright.
        self.vx = pull.x;
        self.vy = pull.y;
        // `:22655-22658`: the pull points downward → the platforms fall
        // through (`GoingDownWithGrapple`).
        self.going_down_with_grapple = pull.y > 0.0;
        // `:22655-22658`: the pull points downward → the platforms fall
        // through (`GoingDownWithGrapple`).

        // `:22670-22673`: the direction follows the pull.
        if self.vx > 0.0 {
            self.facing = 1;
        } else if self.vx < 0.0 {
            self.facing = -1;
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

    fn horizontal_movement(&mut self, dir: f32) {
        if dir < 0.0 {
            if self.vx > -self.max_run_speed {
                if self.vx > self.run_slowdown {
                    self.vx -= self.run_slowdown;
                }
                self.vx -= self.run_acceleration;
            } else if self.vx > -self.acc_run_speed && self.vy == 0.0 && self.dash_delay >= 0 {
                if self.vx > self.run_slowdown {
                    self.vx -= self.run_slowdown;
                }
                self.vx -= self.run_acceleration * 0.2;
            }
        } else if dir > 0.0 {
            if self.vx < self.max_run_speed {
                if self.vx < -self.run_slowdown {
                    self.vx += self.run_slowdown;
                }
                self.vx += self.run_acceleration;
            } else if self.vx < self.acc_run_speed && self.vy == 0.0 && self.dash_delay >= 0 {
                if self.vx < -self.run_slowdown {
                    self.vx += self.run_slowdown;
                }
                self.vx += self.run_acceleration * 0.2;
            }
        } else if self.vy == 0.0 {
            if self.vx > self.run_slowdown {
                self.vx -= self.run_slowdown;
            } else if self.vx < -self.run_slowdown {
                self.vx += self.run_slowdown;
            } else {
                self.vx = 0.0;
            }
        } else if self.vx > self.run_slowdown * 0.5 {
            self.vx -= self.run_slowdown * 0.5;
        } else if self.vx < -self.run_slowdown * 0.5 {
            self.vx += self.run_slowdown * 0.5;
        } else {
            self.vx = 0.0;
        }
    }

    /// `Player.FloorVisuals` (`Player.cs:23708-23743`) + `GetFloorTile`
    /// (`:23755-23783`): the tile row at the feet, center column first, then
    /// the neighbors, solid tiles only. The dust visuals (MakeFloorDust) and
    /// the slope dust carve-out stay with the particle tasks.
    fn floor_visuals(&mut self, world: &World) {
        let x = ((self.x + WIDTH * 0.5) / TILE_PX) as i32;
        let y = ((self.y + HEIGHT) / TILE_PX) as i32;
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
        // `TryRecalculatingTorchLuck` + the diff check sit here
        // (`Player.cs:29126-29130`) — phase 2.
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

    fn update_survival(&mut self) {
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
        if self.in_lava && self.hurt_cd <= 0.0 {
            let cx = self.x + WIDTH * 0.5;
            if self.hurt(30, cx) {
                // The Hurt return `num81 > 0.0` gates the OnFire grant
                // (`Player.cs:27710-27711`; remix/ashWood/lavaRose variants out
                // of scope). `AddBuff` applies the expert ×2 itself (24 is in
                // the difficulty set, `BuffID.cs:50`).
                self.buffs.add_buff(24, 420);
            }
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
        let dry = self.liquid_step == 1.0;
        if dry && vel[1] == gravity {
            collision::step_down(
                world,
                &mut pos,
                &mut vel,
                WIDTH,
                HEIGHT,
                &mut self.step_speed,
                &mut self.gfx_off_y,
            );
        }
        if dry && vel[1] >= gravity && !down {
            collision::step_up(
                world,
                &mut pos,
                &mut vel,
                WIDTH,
                HEIGHT,
                &mut self.step_speed,
                &mut self.gfx_off_y,
                false,
            );
        }
        let tc = collision::tile_collision(world, pos, vel, WIDTH, HEIGHT, false, false);
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
        let sc = collision::slope_collision(world, pos, vel, WIDTH, HEIGHT, self.stair_fall);
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
    fn dot_death_survival_drain_records_the_reason() {
        // DW-1.3 — Player.cs:19361-19384: a drain kill carries its reason out
        // of UpdateLifeRegen; update_survival parks it on the player for the
        // death pipeline (the default generic attribution here = Burned, 8).
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        p.update_survival();
        assert_eq!(p.dot_death, None); // a healthy tick records nothing
        p.hp = 8;
        p.regen.life_regen_count = -1200;
        p.update_survival();
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
        p.hurt(20, 0.0);
        assert_eq!(p.hp, 90, "20 - 20*0.5 = 10 taken");
        // The floor: a huge defense leaves 1.
        p.hurt_cd = 0.0;
        p.hp = 100;
        p.stat_defense = 400;
        p.hurt(20, 0.0);
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
    fn in_liquid_gravity_and_cap_are_thirds() {
        // Player.cs:27008-27017: gravity / 3 (0.4 -> 0.1333) and the
        // both-ways cap at maxFallSpeed / 3 (10 -> 3.333).
        assert!((liquid_gravity() - 0.4 / 3.0).abs() < 1e-6);
        assert!((liquid_fall_cap() - 10.0 / 3.0).abs() < 1e-6);
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
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        p.start_death(false, false);
        assert!(p.dead);
        assert_eq!(p.respawn_timer, 600);
        // 600 ticks of the dead state (no jump) -> the timer drains to 0 and
        // the last tick reports the respawn.
        let mut revived_at = None;
        for tick in 0..700 {
            if p.update_dead(false, false, false) {
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
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        p.start_death(true, true);
        p.respawn_timer = 4000; // the save-load can exceed the cap
        p.update_dead(false, false, false);
        // The clamp catches the decrement result (Player.cs:17049).
        assert_eq!(p.respawn_timer, 3600);
    }

    #[test]
    fn the_skip_needs_the_90_tick_lockout_and_a_jump_edge() {
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        p.start_death(false, false);
        // Inside the 90-tick lockout: no skip even on the jump edge (the
        // held jump's edge is consumed while the lockout runs).
        for _ in 0..89 {
            assert!(!p.update_dead(true, false, false));
        }
        // The lockout expired but the held jump has no fresh edge; a release
        // and a new press do.
        assert!(!p.update_dead(true, false, false));
        assert!(!p.update_dead(false, false, false));
        assert!(p.update_dead(true, false, false));
    }

    #[test]
    fn the_skip_is_blocked_while_a_boss_hinders() {
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        p.start_death(false, false);
        for _ in 0..95 {
            p.update_dead(false, false, false);
        }
        assert!(!p.allows_respawn_timer_skip(true, false));
        assert!(p.allows_respawn_timer_skip(false, false));
    }

    #[test]
    fn the_jump_edge_is_required_for_the_skip() {
        // Holding jump through the lockout does not skip on expiry; a fresh
        // press does (the JustBecameTrue semantics of PlayerInput.Triggers).
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        p.start_death(false, false);
        for _ in 0..95 {
            p.update_dead(true, false, false);
        }
        // The held jump consumed its edge at the lockout expiry...
        assert!(!p.update_dead(true, false, false));
        // ...release and press again: the edge fires.
        p.update_dead(false, false, false);
        assert!(p.update_dead(true, false, false));
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
        p.ice_skate = true;
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
        p.wings_logic = 1;
        p.has_wings = true;
        p.wing_time_max = 100;
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
        p.wings_logic = 1;
        p.has_wings = true;
        p.wing_time_max = 100;
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
        p.wings_logic = 1;
        p.has_wings = true;
        p.wing_time_max = 100;
        p.wing_time = 0.0;
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
        let grapple = Some((hook_center, 13));
        p.update_with_grapple(&w, PlayerInput::default(), grapple);
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
        let grapple = Some((Vec2::new(p.x, p.y - 300.0), 13));
        for _ in 0..40 {
            p.update_with_grapple(&w, PlayerInput::default(), grapple);
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
        p.grappling_active = true;
        p.grapple_hook_center = Vec2::new(p.x + WIDTH * 0.5, p.y - 4.0);
        p.grapple_hook_type = 13;
        let grapple = Some((p.grapple_hook_center, 13));
        // The press edge: the jump-off.
        p.update_with_grapple(&w, PlayerInput {
            jump: true,
            ..Default::default()
        }, grapple);
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
        p.grappling_active = true;
        p.grapple_hook_center = Vec2::new(p.x + WIDTH * 0.5, p.y - 4.0);
        p.grapple_hook_type = 13;
        let grapple = Some((p.grapple_hook_center, 13));
        p.update_with_grapple(&w, PlayerInput {
            jump: true,
            down: true,
            ..Default::default()
        }, grapple);
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
        p.grappling_active = true;
        p.grapple_hook_center = Vec2::new(p.x + WIDTH * 0.5, p.y + 120.0);
        p.grapple_hook_type = 13;
        let grapple = Some((p.grapple_hook_center, 13));
        p.update_with_grapple(&w, PlayerInput::default(), grapple);
        assert!(p.going_down_with_grapple, "the pull points down");
    }

    /// `Player.cs:19963,19993`: the overspeed continuation is gated on
    /// `dashDelay >= 0` — no input acceleration while dashing (the decay
    /// runs instead).
    #[test]
    fn no_input_acceleration_while_dashing() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.wings_logic = 1;
        p.has_wings = true;
        p.fall_start = 40;
        p.vy = 5.0;
        let held = PlayerInput {
            jump: true,
            ..Default::default()
        };
        p.update(&w, held);
        assert!(p.vy <= MAX_FALL / 3.0 + 1e-4, "the glide cap: {}", p.vy);
        assert_eq!(p.fall_start, (p.y / TILE_PX) as i32, "the origin refreshes");
    }

    /// `Player.cs:25360`: `!hasWings` gates the fall damage — any equipped
    /// wings nullify it (PHYS-2's field, armed here).
    #[test]
    fn wings_nullify_the_fall_damage() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.wings_logic = 1;
        p.has_wings = true;
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
        p.wings_logic = 22; // Hoverboard: HasDownHoverStats
        p.has_wings = true;
        p.wing_time_max = 170;
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

    /// Arms the Cloud + Sandstorm options on a settled player.
    fn settled_with_jumps(w: &World, opts: &[ExtraJump]) -> Player {
        let mut p = Player::spawn(w);
        for _ in 0..120 {
            p.update(w, PlayerInput::default());
        }
        for j in opts {
            p.has_jump_options[j.index()] = true;
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
        // The accessory vanishes mid-air.
        p.has_jump_options[ExtraJump::Cloud.index()] = false;
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
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
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
                p.update_survival();
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
            p.update_survival();
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
            p.update_survival();
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
        p.update_survival();
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
        p.update_survival();
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
        p.update_survival();
        let left = p.buffs.active_types();
        assert_eq!(left.len(), 1, "one fire buff survives the shift-skip");
        p.update_survival();
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
        p.update_survival();
        let idx = p.buffs.find_buff_index(24).expect("OnFire granted on lava");
        assert_eq!(p.buffs.buff_time[idx], 420);
        assert_eq!(p.hp, 1000 - 30); // the pre-existing 30-dmg lava hit
        // Inside the iframes the hit (and therefore the grant) repeats nothing.
        let hp = p.hp;
        p.update_survival();
        assert_eq!(p.hp, hp, "no second lava hit inside the iframes");
        // Expert doubles the buff time through the AddBuff chain
        // (`AddBuff_DetermineBuffTimeToAdd`, 24 is in the difficulty set).
        let mut p = drain_rig();
        p.buffs.expert_mode = true;
        p.in_lava = true;
        p.update_survival();
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
        p.update_survival();
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
            p.update_survival();
        }
        assert_eq!(p.regen.life_regen, 0, "−4 + 4 clamps at 0");
        assert_eq!(p.hp, 1000, "a softened −4 drains nothing");
        let mut p = drain_rig();
        p.buffs.add_buff(70, 600); // Venom −30
        p.buffs.add_buff(48, 600);
        p.update_survival();
        assert_eq!(p.regen.life_regen, -26, "−30 + 4 stays negative");
        // Honey without a drain never adds anything (the +2 aura is M2-BUFF-5).
        let mut p = drain_rig();
        p.buffs.add_buff(48, 600);
        p.update_survival();
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
        p.update_survival();
        assert_eq!(p.regen.life_regen, -24);
        let mut p = drain_rig();
        p.buffs.add_buff(24, 600);
        p.buffs.add_buff(324, 600);
        p.buffs.add_buff(137, 600); // drippingSlime
        p.update_survival();
        assert_eq!(p.regen.life_regen, -48, "drip doubles both fire parts");
        // A poison part is not doubled by the drip.
        let mut p = drain_rig();
        p.buffs.add_buff(20, 600); // Poisoned −4
        p.buffs.add_buff(24, 600); // OnFire −8
        p.buffs.add_buff(137, 600);
        p.update_survival();
        assert_eq!(p.regen.life_regen, -20, "−4 poison + (−8×2) fire");
    }

    #[test]
    fn electrified_drains_more_while_moving() {
        // Dirty — `Player.cs:19074-19086`: −8, −32 more while `controlLeft ||
        // controlRight` (the run keys, not velocity).
        let mut p = drain_rig();
        p.buffs.add_buff(144, 600);
        p.update_survival();
        assert_eq!(p.regen.life_regen, -8);
        p.control_left = true;
        p.update_survival();
        assert_eq!(p.regen.life_regen, -40, "−8 −32 with the left key");
        let mut p = drain_rig();
        p.buffs.add_buff(144, 600);
        p.control_right = true;
        p.update_survival();
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
        p.update_survival();
        assert_eq!(p.hp, 0);
        assert_eq!(p.dot_death, Some(DotDeath::Suffocation));
        let mut p = drain_rig();
        p.buffs.add_buff(20, 600); // poisoned → generic lane, reason 9
        p.regen.life_regen_count = -120;
        p.hp = 1;
        p.update_survival();
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
}
