//! Player mounts: the `MountData` data table, the mount instance state and the
//! equip/dismount core — `Mount.cs` (`MountData` :129-266, instance state
//! :325-383, `Reset` :675-694, per-type setup :696-2567, accessors :449-652,
//! `JumpHeight`/`JumpSpeed` :2810-2858, `CanFly` :2860-2879, `SetMount`
//! :6196-6305, `Dismount` :6142-6194, `CheckMountBuff` :5275-5281).
//!
//! The vanilla design keeps one static `MountData` per id plus a `Mount`
//! instance on the player whose accessors encode per-type overrides; the port
//! mirrors that split: this module owns the data rows and the pure state, the
//! `impl Player` blocks at the bottom wire it into the fixed step (the
//! per-tick block is `Player.cs:26623-26688`).
//!
//! Scope (M2-MOUNT-1, framework + ground mounts): the 12 ground-mount rows
//! (Rudolph, Bunny, Slime, Scutlix, Unicorn, Basilisk, the three horses,
//! PogoStick, WoFGoat, QueenSlime) and the state/equip core. Out of scope:
//! the hover/flyer fatigue pool (M2-MOUNT-2) and the minecart rows
//! (M2-MOUNT-3), everything minecart
//! (`SetAsMinecart` `Mount.cs:2747-2799`, SuperCart — M2-MOUNT-3), the
//! per-type abilities (`UpdateDrill`/`UseAbility` — M2-MOUNT-4), the net sync
//! (msg 13 — no multiplayer), and the draw path (Phase 5 of the plan).
//!
//! W4-d8 DOC-SWEEP (3)/(4) exception: the Scutlix (9) ability's
//! SELF-CONTAINED half is ported here — `AimAbility`'s head aim
//! (`Mount.cs:5340-5421`), `UseAbility`'s two-beam fire (`:3258-3289`),
//! `ClampToDeadZone` (`:5296-5333`), the eye-offset table (`:1398-1421`),
//! `StopAbilityCharge` (`:2922-2931`) and the remote charge/active flag
//! writes — the 441/453 crosshair AI (`Projectile.cs:30621-30670`) calls
//! them. The Drill (8) mining payload (`DrillMountData`, `UpdateDrill`/
//! `UseDrill` `:3036-3180`) and the Santank (46) ability stay M2-MOUNT-4.

use glam::Vec2;

/// `Mount.MountData` frame-set triple (`Mount.cs:207-249`): start/count/delay
/// (+ `idleFrameLoop` :237). A `count` of 0 means the set is unused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FrameSet {
    pub start: i32,
    pub count: i32,
    pub delay: i32,
    pub loops: bool,
}

/// `Mount.FrameStanding..FrameDashing` (`Mount.cs:277-287`).
pub(crate) const FRAME_STANDING: i32 = 0;
pub(crate) const FRAME_RUNNING: i32 = 1;
pub(crate) const FRAME_IN_AIR: i32 = 2;
pub(crate) const FRAME_FLYING: i32 = 3;
pub(crate) const FRAME_SWIMMING: i32 = 4;
pub(crate) const FRAME_DASHING: i32 = 5;

/// The static per-type row — `Mount.MountData` (`Mount.cs:129-266`). Fields the
/// framework does not consume yet (ability charges, minecart rails) are carried
/// so later tasks fill behaviour without reshaping the table. Texture sheets
/// are named content files (resolved by the atlas at draw time); an empty name
/// is the `Asset<Texture2D>.Empty` default.
#[derive(Clone, Copy, Debug)]
pub(crate) struct MountData {
    /// Port-added: the `MountID` this row fills (vanilla keys the rows by
    /// array index in `Mount.mounts`).
    pub id: i32,
    /// `MountData.buff` (:163) — the buff that keeps the mount alive.
    pub buff: i32,
    /// `heightBoost` (:161) — the player grows to `42 + heightBoost`.
    pub height_boost: i32,
    /// `flightTimeMax` (:165); 0 = the mount cannot fly (`CanFly` :2860-2879).
    pub flight_time_max: i32,
    pub uses_hover: bool,
    /// `MountData.emitsLight` (`Mount.cs:198`) — the row adds its
    /// `lightColor` to the tile light every tick (`Mount.cs:3632-3636`).
    pub emits_light: bool,
    /// `MountData.lightColor` (`Mount.cs:199`) — the RGB the row emits.
    pub light_color: (f32, f32, f32),
    pub run_speed: f32,
    pub dash_speed: f32,
    pub swim_speed: f32,
    pub acceleration: f32,
    pub jump_speed: f32,
    pub jump_height: i32,
    /// `fallDamage` (:181) — the fall-damage multiplier; the C# default is 0.
    pub fall_damage: f32,
    pub extra_fall: i32,
    pub fatigue_max: i32,
    /// `constantJump` (:187) — auto-jump while held (`Mount.AutoJump` :514).
    pub constant_jump: bool,
    pub block_extra_jumps: bool,
    pub ability_charge_max: i32,
    pub ability_cooldown: i32,
    pub ability_duration: i32,
    pub walking_grace_time_max: i32,
    pub dismounts_on_item_use: bool,
    pub spawn_dust: i32,
    pub spawn_dust_no_gravity: bool,
    pub total_frames: i32,
    pub standing: FrameSet,
    pub running: FrameSet,
    pub flying: FrameSet,
    pub in_air: FrameSet,
    pub idle: FrameSet,
    pub swim: FrameSet,
    pub dashing: FrameSet,
    pub minecart: bool,
    pub can_ride_minecart_tracks: bool,
    pub can_use_wings: bool,
    pub movement_stats_are_additive: bool,
    pub back_sheet: &'static str,
    /// `MountData.backTextureExtra` (`Mount.cs:135`) — the second back strip
    /// (the C#'s draw type 1, `Mount.cs:5516`). The port's mount renderer
    /// draws the back and front strips only (draw types 0 and 2); the field
    /// carries the row's data like `front_extra_sheet` does.
    pub back_extra_sheet: &'static str,
    /// `MountData.backTextureGlow` (`Mount.cs:133`) — the glow strip over the
    /// back texture. S1-L-8b: the data field exists; the draw pass that
    /// consumes it is still unported (the renderer draws back/front only).
    pub back_glow_sheet: &'static str,
    /// `MountData.backTextureExtraGlow` (`Mount.cs:137`) — no row sets it
    /// (the Drill's Initialize writes `Asset<Texture2D>.Empty`, `:1325`);
    /// carried so the slot mapping stays complete.
    pub back_extra_glow_sheet: &'static str,
    pub front_sheet: &'static str,
    /// `MountData.frontTextureGlow` (`Mount.cs:141`) — the glow strip over
    /// the front texture (the Drill seat, the mech cart, the Bat, the shroom
    /// cart). Data only until the glow draw pass lands (S1-L-8b).
    pub front_glow_sheet: &'static str,
    pub front_extra_sheet: &'static str,
    /// `MountData.frontTextureExtraGlow` (`Mount.cs:145`) — the Drill diode's
    /// glow (`Mount.cs:1329`). Data only (S1-L-8b).
    pub front_extra_glow_sheet: &'static str,
    pub x_offset: i32,
    pub y_offset: i32,
    /// `playerYOffsets[totalFrames]` (:155) — per-frame player draw offset.
    pub player_y_offsets: &'static [i32],
    pub body_frame: i32,
    pub player_head_offset: i32,
    pub player_x_offset: i32,
    /// `MountData.delegations.PlayerSize` (`Mount.cs:116`) — the three
    /// transformation rows override the mounted hitbox size: Rat 55 → 14×14
    /// (`DelegateMethods.Mount.RatPlayerSize`, `DelegateMethods.cs:190-194`,
    /// wired `Mount.cs:2366`), Bat 56 → 20×18 (`BatPlayerSize`,
    /// `DelegateMethods.cs:164-168`, `Mount.cs:2421`), Pixie 61 → 8×14
    /// (`PixiePlayerSize`, `DelegateMethods.cs:170-174`, `Mount.cs:2484`).
    /// `None` = no delegate (the `42 + heightBoost` growth, `Mount.cs:6264-6277`).
    pub player_size_override: Option<(f32, f32)>,
}

/// The `MountData` field defaults (C# initializer values, `Mount.cs:129-266`):
/// everything zero/false. Rows only override what `Initialize` sets.
const BASE: MountData = MountData {
    id: 0,
    buff: 0,
    height_boost: 0,
    flight_time_max: 0,
    uses_hover: false,
    emits_light: false,
    // `MountData.lightColor = Vector3.One` (`Mount.cs:259`) — the C# default
    // is white, not zero (S1-L-8b). Only the three `emitsLight` rows override
    // it (Drill `:1290`, SpookyWood `:1952`, LavaShark `:2160`).
    light_color: (1.0, 1.0, 1.0),
    run_speed: 0.0,
    dash_speed: 0.0,
    swim_speed: 0.0,
    acceleration: 0.0,
    jump_speed: 0.0,
    jump_height: 0,
    fall_damage: 0.0,
    extra_fall: 0,
    fatigue_max: 0,
    constant_jump: false,
    block_extra_jumps: false,
    ability_charge_max: 0,
    ability_cooldown: 0,
    ability_duration: 0,
    walking_grace_time_max: 0,
    dismounts_on_item_use: false,
    spawn_dust: 0,
    spawn_dust_no_gravity: false,
    total_frames: 0,
    standing: FrameSet {
        start: 0,
        count: 0,
        delay: 0,
        loops: false,
    },
    running: FrameSet {
        start: 0,
        count: 0,
        delay: 0,
        loops: false,
    },
    flying: FrameSet {
        start: 0,
        count: 0,
        delay: 0,
        loops: false,
    },
    in_air: FrameSet {
        start: 0,
        count: 0,
        delay: 0,
        loops: false,
    },
    idle: FrameSet {
        start: 0,
        count: 0,
        delay: 0,
        loops: false,
    },
    swim: FrameSet {
        start: 0,
        count: 0,
        delay: 0,
        loops: false,
    },
    dashing: FrameSet {
        start: 0,
        count: 0,
        delay: 0,
        loops: false,
    },
    minecart: false,
    can_ride_minecart_tracks: false,
    can_use_wings: false,
    movement_stats_are_additive: false,
    back_sheet: "",
    back_extra_sheet: "",
    back_glow_sheet: "",
    back_extra_glow_sheet: "",
    front_sheet: "",
    front_glow_sheet: "",
    front_extra_sheet: "",
    front_extra_glow_sheet: "",
    x_offset: 0,
    y_offset: 0,
    player_y_offsets: &[],
    body_frame: 0,
    player_head_offset: 0,
    player_x_offset: 0,
    // `MountDelegatesData.PlayerSize` defaults to null (`Mount.cs:116`).
    player_size_override: None,
};

// -- Per-type rows (Mount.Initialize, `Mount.cs:696-2567`) -------------------

/// 0 Rudolph (`Mount.cs:700-751`): partial flight, `flightTimeMax` 160.
static RUDOLPH_Y_OFFSETS: [i32; 12] = [30, 32, 30, 30, 30, 30, 30, 30, 30, 30, 30, 32];
const RUDOLPH: MountData = MountData {
    id: 0,
    buff: 90,
    height_boost: 20,
    flight_time_max: 160,
    run_speed: 5.5,
    dash_speed: 12.0,
    acceleration: 0.09,
    jump_height: 17,
    jump_speed: 5.31,
    spawn_dust: 57,
    total_frames: 12,
    standing: FrameSet {
        start: 0,
        count: 1,
        delay: 12,
        loops: false,
    },
    running: FrameSet {
        start: 6,
        count: 6,
        delay: 12,
        loops: false,
    },
    flying: FrameSet {
        start: 6,
        count: 6,
        delay: 6,
        loops: false,
    },
    in_air: FrameSet {
        start: 1,
        count: 1,
        delay: 12,
        loops: false,
    },
    idle: FrameSet {
        start: 2,
        count: 4,
        delay: 30,
        loops: true,
    },
    swim: FrameSet {
        start: 1,
        count: 1,
        delay: 12,
        loops: false,
    },
    back_sheet: "Rudolph_0",
    front_sheet: "Rudolph_1",
    front_extra_sheet: "Rudolph_2",
    x_offset: 13,
    y_offset: -7,
    player_y_offsets: &RUDOLPH_Y_OFFSETS,
    body_frame: 3,
    player_head_offset: 22,
    ..BASE
};

/// 1 Bunny (`Mount.cs:806-859`): the fall-damage nullify is `Player.cs:25315-25318`.
static BUNNY_Y_OFFSETS: [i32; 7] = [14, 14, 16, 18, 22, 22, 14];
const BUNNY: MountData = MountData {
    id: 1,
    buff: 128,
    height_boost: 20,
    fall_damage: 0.8,
    run_speed: 4.0,
    dash_speed: 7.8,
    acceleration: 0.13,
    jump_height: 15,
    jump_speed: 5.01,
    spawn_dust: 15,
    total_frames: 7,
    standing: FrameSet {
        start: 0,
        count: 1,
        delay: 12,
        loops: false,
    },
    running: FrameSet {
        start: 0,
        count: 7,
        delay: 12,
        loops: false,
    },
    flying: FrameSet {
        start: 1,
        count: 6,
        delay: 6,
        loops: false,
    },
    in_air: FrameSet {
        start: 5,
        count: 1,
        delay: 12,
        loops: false,
    },
    // `swimFrame* = inAirFrame*` (`Mount.cs:848-850`) — S1-L-8b.
    swim: FrameSet {
        start: 5,
        count: 1,
        delay: 12,
        loops: false,
    },
    back_sheet: "Mount_Bunny",
    x_offset: 1,
    y_offset: 4,
    player_y_offsets: &BUNNY_Y_OFFSETS,
    body_frame: 3,
    player_head_offset: 22,
    ..BASE
};

/// 3 Slime (`Mount.cs:861-911`): constant-jump bouncer; wet-slime handling is
/// `Player.cs:27985-27998` (M2-MOUNT-2).
static SLIME_Y_OFFSETS: [i32; 4] = [20, 22, 20, 18];
const SLIME: MountData = MountData {
    id: 3,
    buff: 130,
    height_boost: 20,
    fall_damage: 0.5,
    extra_fall: 10,
    run_speed: 4.0,
    dash_speed: 4.0,
    acceleration: 0.18,
    jump_height: 12,
    jump_speed: 8.25,
    constant_jump: true,
    spawn_dust: 56,
    total_frames: 4,
    standing: FrameSet {
        start: 0,
        count: 1,
        delay: 12,
        loops: false,
    },
    running: FrameSet {
        start: 0,
        count: 4,
        delay: 12,
        loops: false,
    },
    in_air: FrameSet {
        start: 1,
        count: 1,
        delay: 12,
        loops: false,
    },
    back_sheet: "Mount_Slime",
    x_offset: 1,
    y_offset: 11,
    player_y_offsets: &SLIME_Y_OFFSETS,
    body_frame: 3,
    player_head_offset: 22,
    ..BASE
};

/// 9 Scutlix (`Mount.cs:1343-1421`): `8/8/0.4`; the aiming projectile ability
/// is M2-MOUNT-4 (the ability fields are already in the row). The decompile
/// asserts the frame size 45x54 (`:1398-1416`).
static SCUTLIX_Y_OFFSETS: [i32; 12] = [16; 12];
const SCUTLIX: MountData = MountData {
    id: 9,
    buff: 143,
    height_boost: 16,
    fall_damage: 0.0,
    ability_charge_max: 40,
    ability_cooldown: 20,
    ability_duration: 0,
    run_speed: 8.0,
    dash_speed: 8.0,
    acceleration: 0.4,
    jump_height: 22,
    jump_speed: 10.01,
    block_extra_jumps: false,
    spawn_dust: 15,
    total_frames: 12,
    standing: FrameSet {
        start: 6,
        count: 6,
        delay: 12,
        loops: false,
    },
    running: FrameSet {
        start: 0,
        count: 6,
        delay: 12,
        loops: false,
    },
    flying: FrameSet {
        start: 0,
        count: 0,
        delay: 12,
        loops: false,
    },
    in_air: FrameSet {
        start: 1,
        count: 1,
        delay: 12,
        loops: false,
    },
    idle: FrameSet {
        start: 6,
        count: 0,
        delay: 12,
        loops: true,
    },
    swim: FrameSet {
        start: 0,
        count: 0,
        delay: 12,
        loops: false,
    },
    // `TextureAssets.ScutlixMount[0..2]` — the [1]/[2] sheet names are verified
    // against the atlas in the draw phase.
    back_sheet: "Mount_Scutlix",
    front_sheet: "Mount_ScutlixEyes",
    front_extra_sheet: "Mount_ScutlixEyeGlow",
    x_offset: 1,
    y_offset: 6,
    player_y_offsets: &SCUTLIX_Y_OFFSETS,
    body_frame: 3,
    player_head_offset: 18,
    ..BASE
};

/// 10 Unicorn (`Mount.cs:1423-1482`): the atlas "4/12/0.3" ground dasher.
static UNICORN_Y_OFFSETS: [i32; 16] = [
    28, 28, 28, 30, 30, 28, 28, 30, 30, 28, 28, 28, 30, 30, 28, 32,
];
const UNICORN: MountData = MountData {
    id: 10,
    buff: 162,
    height_boost: 34,
    fall_damage: 0.2,
    run_speed: 4.0,
    dash_speed: 12.0,
    acceleration: 0.3,
    jump_height: 10,
    jump_speed: 8.01,
    spawn_dust: 15,
    total_frames: 16,
    standing: FrameSet {
        start: 0,
        count: 1,
        delay: 12,
        loops: false,
    },
    running: FrameSet {
        start: 1,
        count: 7,
        delay: 15,
        loops: false,
    },
    dashing: FrameSet {
        start: 9,
        count: 6,
        delay: 40,
        loops: false,
    },
    flying: FrameSet {
        start: 1,
        count: 6,
        delay: 6,
        loops: false,
    },
    in_air: FrameSet {
        start: 15,
        count: 1,
        delay: 12,
        loops: false,
    },
    // `swimFrame* = inAirFrame*` (`Mount.cs:1471-1473`) — S1-L-8b.
    swim: FrameSet {
        start: 15,
        count: 1,
        delay: 12,
        loops: false,
    },
    back_sheet: "Mount_Unicorn",
    x_offset: 5,
    y_offset: 1,
    player_y_offsets: &UNICORN_Y_OFFSETS,
    body_frame: 3,
    player_head_offset: 34,
    ..BASE
};

/// 14 Basilisk (`Mount.cs:1644-1692`): `dashSpeed` is never set in Initialize
/// (stays the 0 default) — verified against this decompile.
static BASILISK_Y_OFFSETS: [i32; 8] = [8, 10, 8, 10, 8, 8, 10, 8];
const BASILISK: MountData = MountData {
    id: 14,
    buff: 193,
    height_boost: 6,
    fall_damage: 0.2,
    run_speed: 8.0,
    acceleration: 0.25,
    jump_height: 20,
    jump_speed: 8.01,
    spawn_dust: 15,
    total_frames: 8,
    standing: FrameSet {
        start: 0,
        count: 1,
        delay: 12,
        loops: false,
    },
    running: FrameSet {
        start: 2,
        count: 6,
        delay: 30,
        loops: false,
    },
    in_air: FrameSet {
        start: 1,
        count: 1,
        delay: 12,
        loops: false,
    },
    // `swimFrame* = inAirFrame*` (`Mount.cs:1681-1683`) — S1-L-8b.
    swim: FrameSet {
        start: 1,
        count: 1,
        delay: 12,
        loops: false,
    },
    back_sheet: "Mount_Basilisk",
    x_offset: 4,
    y_offset: 8,
    player_y_offsets: &BASILISK_Y_OFFSETS,
    body_frame: 3,
    player_head_offset: 10,
    ..BASE
};

/// 40/41/42 horses via `SetAsHorse` (`Mount.cs:2607-2668`, rows :1843-1850):
/// the atlas "3/9/0.25"; buffs 275/276/277.
static HORSE_Y_OFFSETS: [i32; 16] = [
    28, 28, 28, 30, 30, 28, 28, 30, 30, 28, 28, 28, 30, 30, 28, 32,
];
const fn horse(id: i32, buff: i32, back_sheet: &'static str) -> MountData {
    MountData {
        id,
        buff,
        height_boost: 34,
        fall_damage: 0.5,
        run_speed: 3.0,
        dash_speed: 9.0,
        acceleration: 0.25,
        jump_height: 6,
        jump_speed: 7.01,
        spawn_dust: 3,
        total_frames: 16,
        standing: FrameSet {
            start: 0,
            count: 1,
            delay: 12,
            loops: false,
        },
        running: FrameSet {
            start: 1,
            count: 7,
            delay: 15,
            loops: false,
        },
        dashing: FrameSet {
            start: 9,
            count: 6,
            delay: 40,
            loops: false,
        },
        flying: FrameSet {
            start: 1,
            count: 6,
            delay: 6,
            loops: false,
        },
        in_air: FrameSet {
            start: 15,
            count: 1,
            delay: 12,
            loops: false,
        },
        // `swimFrame* = inAirFrame*` (`Mount.cs:2656-2658`) — S1-L-8b.
        swim: FrameSet {
            start: 15,
            count: 1,
            delay: 12,
            loops: false,
        },
        back_sheet,
        x_offset: 5,
        y_offset: 1,
        player_y_offsets: &HORSE_Y_OFFSETS,
        body_frame: 3,
        player_head_offset: 34,
        ..BASE
    }
}
const PAINTED_HORSE: MountData = horse(40, 275, "Extra_161");
const MAJESTIC_HORSE: MountData = horse(41, 276, "Extra_162");
const DARK_HORSE: MountData = horse(42, 277, "Extra_163");

/// 43 PogoStick (`Mount.cs:1852-1899`): constant-jump hopper, `extraFall` 20,
/// `bodyFrame` 4 (the only one off row 3 in this set).
static POGO_Y_OFFSETS: [i32; 4] = [14; 4];
const POGO_STICK: MountData = MountData {
    id: 43,
    buff: 278,
    height_boost: 12,
    fall_damage: 0.25,
    extra_fall: 20,
    run_speed: 5.0,
    acceleration: 0.1,
    jump_height: 8,
    jump_speed: 8.0,
    constant_jump: true,
    spawn_dust: 15,
    total_frames: 4,
    standing: FrameSet {
        start: 0,
        count: 1,
        delay: 5,
        loops: false,
    },
    running: FrameSet {
        start: 0,
        count: 4,
        delay: 5,
        loops: false,
    },
    in_air: FrameSet {
        start: 0,
        count: 1,
        delay: 5,
        loops: false,
    },
    swim: FrameSet {
        start: 0,
        count: 1,
        delay: 5,
        loops: false,
    },
    front_sheet: "Extra_164",
    x_offset: 5,
    y_offset: 10,
    player_y_offsets: &POGO_Y_OFFSETS,
    body_frame: 4,
    player_head_offset: 10,
    ..BASE
};

/// 47 WallOfFleshGoat (`Mount.cs:2041-2100`): Unicorn stats with a glow pass.
static WOF_GOAT_Y_OFFSETS: [i32; 16] = [
    30, 30, 30, 32, 32, 30, 30, 32, 32, 30, 30, 30, 32, 32, 30, 34,
];
const WOF_GOAT: MountData = MountData {
    id: 47,
    buff: 282,
    height_boost: 34,
    fall_damage: 0.2,
    run_speed: 4.0,
    dash_speed: 12.0,
    acceleration: 0.3,
    jump_height: 10,
    jump_speed: 8.01,
    spawn_dust: 5,
    total_frames: 16,
    standing: FrameSet {
        start: 0,
        count: 1,
        delay: 12,
        loops: false,
    },
    running: FrameSet {
        start: 1,
        count: 7,
        delay: 15,
        loops: false,
    },
    dashing: FrameSet {
        start: 9,
        count: 6,
        delay: 40,
        loops: false,
    },
    flying: FrameSet {
        start: 1,
        count: 6,
        delay: 6,
        loops: false,
    },
    in_air: FrameSet {
        start: 15,
        count: 1,
        delay: 12,
        loops: false,
    },
    // `swimFrame* = inAirFrame*` (`Mount.cs:2089-2091`) — S1-L-8b.
    swim: FrameSet {
        start: 15,
        count: 1,
        delay: 12,
        loops: false,
    },
    back_sheet: "Extra_169",
    // `TextureAssets.GlowMask[284]` (`Mount.cs:2095`) loads as `Images/Glow_284`
    // (`AssetInitializer.cs:352-354`) — S1-L-8b fixes the sheet name (the old
    // `GlowMask_284` matches no content file). Data only: the glow draw pass
    // is still unported.
    back_glow_sheet: "Glow_284",
    x_offset: 5,
    y_offset: -1,
    player_y_offsets: &WOF_GOAT_Y_OFFSETS,
    body_frame: 3,
    player_head_offset: 34,
    ..BASE
};

/// 50 QueenSlime (`Mount.cs:2194-2244`): constant-jump bouncer with a short
/// flight meter (80) and the in-air +2 run boost (`RunSpeed` :472-475).
static QUEEN_SLIME_Y_OFFSETS: [i32; 8] = [20, 22, 20, 20, 22, 22, 20, 20];
const QUEEN_SLIME: MountData = MountData {
    id: 50,
    buff: 318,
    height_boost: 20,
    flight_time_max: 80,
    fall_damage: 0.5,
    run_speed: 5.5,
    dash_speed: 5.5,
    acceleration: 0.2,
    jump_height: 10,
    jump_speed: 7.25,
    constant_jump: true,
    spawn_dust: 243,
    total_frames: 8,
    standing: FrameSet {
        start: 0,
        count: 1,
        delay: 12,
        loops: false,
    },
    running: FrameSet {
        start: 0,
        count: 5,
        delay: 16,
        loops: false,
    },
    in_air: FrameSet {
        start: 5,
        count: 1,
        delay: 12,
        loops: false,
    },
    back_sheet: "Extra_204",
    x_offset: 1,
    y_offset: -1,
    player_y_offsets: &QUEEN_SLIME_Y_OFFSETS,
    body_frame: 3,
    player_head_offset: 22,
    ..BASE
};

/// The partial table (12 of `MountID.Count` 66, `MountID.cs:189`). Vanilla
/// fills all 66 in `Initialize`; ids outside this slice are rejected by
/// `set_mount` — deviation recorded in ANALYSIS "Mounts framework (M2-MOUNT-1)",
/// the remaining categories land with M2-MOUNT-2/3/4.
// -- Hover/flier rows (M2-MOUNT-2, `Mount.cs:696-2567`) -----------------------

/// 2 Pigron (`Mount.cs:752-804`): partial flight, `flightTimeMax` 160.
static PIGRON_Y_OFFSETS: [i32; 16] = [
    22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 24, 26, 24, 22,
];
const PIGRON: MountData = MountData {
    id: 2,
    buff: 129,
    height_boost: 20,
    flight_time_max: 160,
    run_speed: 5.0,
    dash_speed: 9.0,
    acceleration: 0.08,
    jump_height: 10,
    jump_speed: 6.01,
    spawn_dust: 58,
    total_frames: 16,
    standing: FrameSet { start: 7, count: 1, delay: 12, loops: false },
    running: FrameSet { start: 11, count: 5, delay: 12, loops: false },
    flying: FrameSet { start: 1, count: 6, delay: 6, loops: false },
    in_air: FrameSet { start: 0, count: 1, delay: 12, loops: false },
    idle: FrameSet { start: 8, count: 3, delay: 20, loops: false },
    swim: FrameSet { start: 0, count: 1, delay: 12, loops: false },
    back_sheet: "Mount_Pigron",
    x_offset: 1,
    y_offset: 8,
    player_y_offsets: &PIGRON_Y_OFFSETS,
    body_frame: 3,
    player_head_offset: 22,
    ..BASE
};

/// 5 Bee (`Mount.cs:1163-1217`): hover, 320 fuel + 320 fatigue, wings. S1-L-8b
/// fills the C# `backTextureExtra` (`Mount_BeeWings`, `:1212`) into the row;
/// the animated wing draw (the `UpdateFrame` bee arm) still waits for
/// M2-MOUNT-2 — the field carries the data only.
static BEE_Y_OFFSETS: [i32; 12] = [16, 16, 16, 16, 16, 16, 16, 16, 18, 16, 16, 16];
const BEE: MountData = MountData {
    id: 5,
    buff: 132,
    height_boost: 16,
    flight_time_max: 320,
    fatigue_max: 320,
    fall_damage: 0.0,
    uses_hover: true,
    run_speed: 2.0,
    dash_speed: 2.0,
    acceleration: 0.16,
    jump_height: 10,
    jump_speed: 4.0,
    block_extra_jumps: true,
    spawn_dust: 152,
    total_frames: 12,
    standing: FrameSet { start: 0, count: 1, delay: 12, loops: false },
    running: FrameSet { start: 0, count: 5, delay: 12, loops: false },
    flying: FrameSet { start: 5, count: 3, delay: 12, loops: false },
    in_air: FrameSet { start: 5, count: 3, delay: 12, loops: false },
    idle: FrameSet { start: 8, count: 4, delay: 12, loops: true },
    swim: FrameSet { start: 0, count: 0, delay: 12, loops: false },
    back_sheet: "Mount_Bee",
    // `backTextureExtra = BeeMount[1]` (`Mount.cs:1212`) — `Mount_BeeWings`
    // (`AssetInitializer.cs:443`). Data only (S1-L-8b).
    back_extra_sheet: "Mount_BeeWings",
    x_offset: 1,
    y_offset: 4,
    player_y_offsets: &BEE_Y_OFFSETS,
    body_frame: 3,
    player_head_offset: 18,
    ..BASE
};

/// 7 UFO (`Mount.cs:1218-1272`): hover, no fatigue penalty (ignores fatigue).
const UFO: MountData = MountData {
    id: 7,
    buff: 141,
    height_boost: 16,
    flight_time_max: 320,
    fatigue_max: 320,
    fall_damage: 0.0,
    uses_hover: true,
    run_speed: 8.0,
    dash_speed: 8.0,
    acceleration: 0.16,
    jump_height: 10,
    jump_speed: 4.0,
    block_extra_jumps: true,
    spawn_dust: 226,
    spawn_dust_no_gravity: true,
    total_frames: 8,
    standing: FrameSet { start: 0, count: 8, delay: 4, loops: false },
    running: FrameSet { start: 0, count: 8, delay: 4, loops: false },
    flying: FrameSet { start: 0, count: 8, delay: 4, loops: false },
    in_air: FrameSet { start: 0, count: 8, delay: 4, loops: false },
    idle: FrameSet { start: 0, count: 0, delay: 12, loops: true },
    swim: FrameSet { start: 0, count: 0, delay: 12, loops: false },
    front_sheet: "Mount_UFO",
    front_extra_sheet: "Mount_UFOGlow",
    x_offset: 1,
    y_offset: 4,
    player_y_offsets: &[16; 8],
    body_frame: 3,
    player_head_offset: 18,
    ..BASE
};

/// 8 Drill (`Mount.cs:1273-1341`): swim 4, hover, light-emitting (`:1289-1290`,
/// added by S1-L-8b). The idle band's `start` is 8 (`:1316`) even though the
/// count is 0 — the drill rotation arm of `UpdateFrame` reads it. The three
/// glow strips (`DrillMount[3..5]`, `:1323`/`:1327`/`:1329`) are data-only
/// until the glow draw pass lands.
const DRILL: MountData = MountData {
    id: 8,
    buff: 142,
    height_boost: 16,
    flight_time_max: 320,
    fatigue_max: 320,
    fall_damage: 1.0,
    uses_hover: true,
    swim_speed: 4.0,
    run_speed: 6.0,
    dash_speed: 4.0,
    acceleration: 0.16,
    jump_height: 10,
    jump_speed: 4.0,
    block_extra_jumps: true,
    emits_light: true,
    light_color: (0.3, 0.3, 0.4),
    spawn_dust: 226,
    total_frames: 1,
    standing: FrameSet { start: 0, count: 1, delay: 12, loops: false },
    running: FrameSet { start: 0, count: 1, delay: 12, loops: false },
    flying: FrameSet { start: 0, count: 1, delay: 12, loops: false },
    in_air: FrameSet { start: 0, count: 1, delay: 12, loops: false },
    // `idleFrameStart = 8` (`Mount.cs:1316`); count/delay stay 0/12.
    idle: FrameSet { start: 8, count: 0, delay: 12, loops: false },
    swim: FrameSet { start: 0, count: 0, delay: 12, loops: false },
    back_sheet: "Mount_DrillRing",
    // `backTextureGlow = DrillMount[3]` (`:1323`) = `Mount_Glow_DrillRing`
    // (`AssetInitializer.cs:449`).
    back_glow_sheet: "Mount_Glow_DrillRing",
    front_sheet: "Mount_DrillSeat",
    // `frontTextureGlow = DrillMount[4]` (`:1327`) = `Mount_Glow_DrillSeat`
    // (`AssetInitializer.cs:450`).
    front_glow_sheet: "Mount_Glow_DrillSeat",
    front_extra_sheet: "Mount_DrillDiode",
    // `frontTextureExtraGlow = DrillMount[5]` (`:1329`) =
    // `Mount_Glow_DrillDiode` (`AssetInitializer.cs:451`).
    front_extra_glow_sheet: "Mount_Glow_DrillDiode",
    x_offset: 1,
    y_offset: 4,
    player_y_offsets: &[4; 1],
    body_frame: 3,
    player_head_offset: 18,
    ..BASE
};

/// 12 CuteFishron (`Mount.cs:1536-1590`): swim 16, swim frames, hover with the
/// half speed multiplier (`MountFishronSpecial` — not ported). S1-L-8b fills
/// the `backTextureGlow` strip `Mount_CuteFishron2` (`:1585`); the glow draw
/// pass is still unported — data only.
static FISHRON_Y_OFFSETS: [i32; 23] = [12; 23];
const CUTE_FISHRON: MountData = MountData {
    id: 12,
    buff: 168,
    height_boost: 14,
    flight_time_max: 320,
    fatigue_max: 320,
    fall_damage: 0.0,
    uses_hover: true,
    swim_speed: 16.0,
    run_speed: 2.0,
    dash_speed: 1.0,
    acceleration: 0.2,
    jump_height: 4,
    jump_speed: 3.0,
    block_extra_jumps: true,
    spawn_dust: 15,
    total_frames: 23,
    standing: FrameSet { start: 8, count: 1, delay: 12, loops: false },
    running: FrameSet { start: 8, count: 7, delay: 14, loops: false },
    flying: FrameSet { start: 0, count: 8, delay: 16, loops: false },
    in_air: FrameSet { start: 0, count: 8, delay: 6, loops: false },
    idle: FrameSet { start: 0, count: 0, delay: 0, loops: false },
    swim: FrameSet { start: 15, count: 8, delay: 4, loops: false },
    back_sheet: "Mount_CuteFishron1",
    // `backTextureGlow = CuteFishronMount[1]` (`Mount.cs:1585`) =
    // `Mount_CuteFishron2` (`AssetInitializer.cs:438`). Data only (S1-L-8b).
    back_glow_sheet: "Mount_CuteFishron2",
    x_offset: 2,
    y_offset: 16,
    player_y_offsets: &FISHRON_Y_OFFSETS,
    body_frame: 3,
    player_head_offset: 16,
    ..BASE
};

/// 23 WitchBroom (`Mount.cs:1740-1794`): hover, `inAir` 6/8 band, heightBoost 0.
static BROOM_Y_OFFSETS: [i32; 6] = [6, 6, 6, 6, 6, 6];
const WITCH_BROOM: MountData = MountData {
    id: 23,
    buff: 230,
    height_boost: 0,
    flight_time_max: 320,
    fatigue_max: 320,
    fall_damage: 0.0,
    uses_hover: true,
    run_speed: 9.0,
    dash_speed: 9.0,
    acceleration: 0.16,
    jump_height: 10,
    jump_speed: 4.0,
    block_extra_jumps: true,
    spawn_dust: 43,
    spawn_dust_no_gravity: true,
    total_frames: 6,
    standing: FrameSet { start: 0, count: 1, delay: 0, loops: false },
    running: FrameSet { start: 0, count: 1, delay: 0, loops: false },
    flying: FrameSet { start: 0, count: 1, delay: 0, loops: false },
    in_air: FrameSet { start: 0, count: 6, delay: 8, loops: false },
    idle: FrameSet { start: 0, count: 0, delay: 0, loops: true },
    swim: FrameSet { start: 0, count: 0, delay: 0, loops: false },
    back_sheet: "Extra_113",
    x_offset: -2,
    y_offset: 8,
    player_y_offsets: &BROOM_Y_OFFSETS,
    body_frame: 0,
    player_head_offset: 0,
    ..BASE
};

/// 44 PirateShip (`Mount.cs:1900-1938`): hover, the running-only row.
static PIRATE_Y_OFFSETS: [i32; 10] = [9; 10];
const PIRATE_SHIP: MountData = MountData {
    id: 44,
    buff: 279,
    height_boost: 24,
    flight_time_max: 320,
    fatigue_max: 320,
    fall_damage: 0.0,
    uses_hover: true,
    run_speed: 3.0,
    dash_speed: 6.0,
    acceleration: 0.12,
    jump_height: 3,
    jump_speed: 1.0,
    swim_speed: 3.0,
    block_extra_jumps: true,
    spawn_dust: 228,
    total_frames: 10,
    running: FrameSet { start: 0, count: 10, delay: 8, loops: false },
    front_sheet: "Extra_166",
    x_offset: 0,
    y_offset: 8,
    player_y_offsets: &PIRATE_Y_OFFSETS,
    body_frame: 3,
    player_head_offset: 16,
    ..BASE
};

/// 48 DarkMageBook (`Mount.cs:2101-2143`): `usesHover` but `CanFly` false
/// (`Mount.cs:2860-2879`); hover without flight fuel, `flag4` off so the
/// acceleration term never applies.
static BOOK_Y_OFFSETS: [i32; 6] = [15, 15, 13, 13, 13, 15];
const DARK_MAGE_BOOK: MountData = MountData {
    id: 48,
    buff: 283,
    height_boost: 14,
    flight_time_max: 320,
    uses_hover: true,
    run_speed: 8.0,
    dash_speed: 8.0,
    acceleration: 0.2,
    jump_height: 5,
    jump_speed: 6.0,
    swim_speed: 8.0,
    spawn_dust: 62,
    total_frames: 6,
    running: FrameSet { start: 0, count: 6, delay: 8, loops: false },
    back_sheet: "Extra_170",
    x_offset: 1,
    y_offset: 16,
    player_y_offsets: &BOOK_Y_OFFSETS,
    body_frame: 0,
    player_head_offset: 16,
    ..BASE
};

/// 49 LavaShark (`Mount.cs:2144-2192`): `CanHover` only in the swim state,
/// `flightTimeMax` 0, `fatigueMax` 320. S1-L-8b adds the light
/// (`emitsLight`/`lightColor`, `:2159-2160`) and the `backTextureGlow`
/// `GlowMask[285]` = `Glow_285` (`:2187`, `AssetInitializer.cs:352-354`).
static LAVA_SHARK_Y_OFFSETS: [i32; 8] = [10; 8];
const LAVA_SHARK: MountData = MountData {
    id: 49,
    buff: 305,
    height_boost: 8,
    run_speed: 2.0,
    dash_speed: 1.0,
    acceleration: 0.4,
    jump_height: 4,
    jump_speed: 3.0,
    swim_speed: 14.0,
    block_extra_jumps: true,
    flight_time_max: 0,
    fatigue_max: 320,
    uses_hover: true,
    emits_light: true,
    light_color: (0.3, 0.15, 0.1),
    spawn_dust: 35,
    total_frames: 8,
    standing: FrameSet { start: 4, count: 1, delay: 12, loops: false },
    running: FrameSet { start: 4, count: 4, delay: 14, loops: false },
    in_air: FrameSet { start: 4, count: 1, delay: 6, loops: false },
    swim: FrameSet { start: 0, count: 4, delay: 16, loops: false },
    back_sheet: "Extra_172",
    // `backTextureGlow = GlowMask[285]` (`Mount.cs:2187`). Data only (S1-L-8b).
    back_glow_sheet: "Glow_285",
    x_offset: 2,
    y_offset: 1,
    player_y_offsets: &LAVA_SHARK_Y_OFFSETS,
    body_frame: 3,
    player_head_offset: 16,
    ..BASE
};

/// 56 Bat (`Mount.cs:2416-2478`): dismounts on item use, no fuel, hover with
/// the dash-speed rise cap. S1-L-8b: `fatigueMax` 320 (`:2426`) and the
/// `frontTextureGlow` `GlowMask[370]` = `Glow_370` (`:2474`,
/// `AssetInitializer.cs:352-354`) — data only.
const BAT: MountData = MountData {
    id: 56,
    buff: 377,
    flight_time_max: 320,
    fatigue_max: 320,
    fall_damage: 0.0,
    uses_hover: true,
    dismounts_on_item_use: true,
    block_extra_jumps: true,
    dash_speed: 4.5,
    run_speed: 4.5,
    acceleration: 0.2,
    jump_height: 8,
    jump_speed: 5.0,
    spawn_dust: 31,
    total_frames: 16,
    standing: FrameSet { start: 0, count: 1, delay: 6, loops: false },
    dashing: FrameSet { start: 9, count: 6, delay: 40, loops: false },
    flying: FrameSet { start: 1, count: 8, delay: 5, loops: false },
    idle: FrameSet { start: 0, count: 0, delay: 0, loops: false },
    running: FrameSet { start: 1, count: 8, delay: 5, loops: false },
    in_air: FrameSet { start: 1, count: 8, delay: 5, loops: false },
    swim: FrameSet { start: 1, count: 8, delay: 5, loops: false },
    front_sheet: "Extra_274",
    // `frontTextureGlow = GlowMask[370]` (`Mount.cs:2474`). Data only (S1-L-8b).
    front_glow_sheet: "Glow_370",
    x_offset: -4,
    y_offset: -1,
    player_y_offsets: &[0; 16],
    body_frame: 3,
    player_head_offset: -24,
    // `DelegateMethods.Mount.BatPlayerSize` (`Mount.cs:2421` →
    // `DelegateMethods.cs:164-168`): the bat body is 20×18.
    player_size_override: Some((20.0, 18.0)),
    ..BASE
};

/// 61 Pixie (`Mount.cs:2479-2542`): same flight shape as the Bat. S1-L-8b:
/// `fatigueMax` 320 (`:2490`); `frontTextureGlow` stays Empty (`:2538`).
const PIXIE: MountData = MountData {
    id: 61,
    buff: 384,
    flight_time_max: 320,
    fatigue_max: 320,
    fall_damage: 0.0,
    uses_hover: true,
    dismounts_on_item_use: true,
    block_extra_jumps: true,
    dash_speed: 4.5,
    run_speed: 4.5,
    acceleration: 0.2,
    jump_height: 8,
    jump_speed: 5.0,
    spawn_dust: 43,
    spawn_dust_no_gravity: true,
    total_frames: 4,
    standing: FrameSet { start: 0, count: 4, delay: 8, loops: false },
    dashing: FrameSet { start: 0, count: 4, delay: 2, loops: false },
    flying: FrameSet { start: 0, count: 4, delay: 6, loops: false },
    idle: FrameSet { start: 0, count: 4, delay: 8, loops: true },
    running: FrameSet { start: 0, count: 4, delay: 6, loops: false },
    in_air: FrameSet { start: 0, count: 4, delay: 6, loops: false },
    swim: FrameSet { start: 0, count: 4, delay: 6, loops: false },
    front_sheet: "Extra_290",
    front_extra_sheet: "Extra_291",
    x_offset: -2,
    y_offset: -1,
    player_y_offsets: &[0; 4],
    body_frame: 0,
    player_head_offset: -24,
    // `DelegateMethods.Mount.PixiePlayerSize` (`Mount.cs:2484` →
    // `DelegateMethods.cs:170-174`): the pixie body is 8×14.
    player_size_override: Some((8.0, 14.0)),
    ..BASE
};
// -- Minecart rows (M2-MOUNT-3, `Mount.cs:912-1110` `SetAsMinecart`) -----------

/// `Mount.SetAsMinecart` (`Mount.cs:2747-2799`): the shared cart body —
/// `Minecart`/Sparks dust 213, heightBoost 10, flightTimeMax 0, fallDamage 1,
/// run/dash 13, accel 0.04, jump 15/5.15, blockExtraJumps, 3 frames, the
/// standing 1/12/0 + running 3/12/0 bands, only the front texture.
const fn cart(id: i32, buff: i32, front_sheet: &'static str) -> MountData {
    MountData {
        id,
        buff,
        minecart: true,
        spawn_dust: 213,
        height_boost: 10,
        flight_time_max: 0,
        fall_damage: 1.0,
        run_speed: 13.0,
        dash_speed: 13.0,
        acceleration: 0.04,
        jump_height: 15,
        jump_speed: 5.15,
        block_extra_jumps: true,
        total_frames: 3,
        standing: FrameSet {
            start: 0,
            count: 1,
            delay: 12,
            loops: false,
        },
        running: FrameSet {
            start: 0,
            count: 3,
            delay: 12,
            loops: false,
        },
        flying: FrameSet {
            start: 0,
            count: 0,
            delay: 0,
            loops: false,
        },
        in_air: FrameSet {
            start: 0,
            count: 0,
            delay: 0,
            loops: false,
        },
        idle: FrameSet {
            start: 0,
            count: 0,
            delay: 0,
            loops: false,
        },
        swim: FrameSet {
            start: 0,
            count: 0,
            delay: 0,
            loops: false,
        },
        front_sheet,
        x_offset: 1,
        y_offset: 13,
        player_y_offsets: &[8; 3],
        body_frame: 3,
        player_head_offset: 14,
        ..BASE
    }
}

/// `SetAsMinecart` with the vertical offsets (`Mount.cs:1097` Fart, `:1105`
/// Terra-Fart, both `(-10, -8)`): `yOffset = 13 + verticalOffset` (= 3,
/// `:2772`), the y-offsets `8 - verticalOffset + playerVerticalOffset`
/// (= `8 + 10 - 8` = **10**, `:2767` — S1-L-8b fixes the port's old 0). The
/// shared offsets are STATIC so the row stays a promoted `const` (`'static`
/// borrow).
const CART_VO_Y_OFFSETS: [i32; 3] = [10, 10, 10];

const fn cart_vo(id: i32, buff: i32, front_sheet: &'static str) -> MountData {
    MountData {
        y_offset: 13 - 10,
        player_y_offsets: &CART_VO_Y_OFFSETS,
        ..cart(id, buff, front_sheet)
    }
}

/// `MountID.Minecart` — the base cart's row id (`MountID.cs`: `Minecart = 6`).
pub(crate) const MOUNT_MINECART: i32 = 6;

/// 6 Minecart (`Mount.cs:912-963`, hand-written; identical to the
/// `SetAsMinecart` body): the base cart, buff 118, `Mount_Minecart`.
const MINECART: MountData = MountData {
    id: 6,
    buff: 118,
    ..cart(6, 118, "Mount_Minecart")
};

/// 11 MinecartMech (`Mount.cs:1483-1535`, hand-written): heightBoost 12,
/// playerYOffsets 9, xOffset -1, yOffset 11, the mech body + glow textures.
/// S1-L-8b: `spawnDust` is 213 (`:1488` — the port had 0), and the glow strip
/// is `frontTextureGlow` (`:1531`), not a front-extra. The sheet names are the
/// content files `Mount_MinecartMech`/`Mount_MinecartMechGlow`
/// (`AssetInitializer.cs:435-436`); the old `MinecartMechMount*` names match
/// no file.
const MINECART_MECH: MountData = MountData {
    id: 11,
    buff: 166,
    minecart: true,
    spawn_dust: 213,
    height_boost: 12,
    flight_time_max: 0,
    fall_damage: 1.0,
    run_speed: 13.0,
    dash_speed: 13.0,
    acceleration: 0.04,
    jump_height: 15,
    jump_speed: 5.15,
    block_extra_jumps: true,
    total_frames: 3,
    standing: FrameSet {
        start: 0,
        count: 1,
        delay: 12,
        loops: false,
    },
    running: FrameSet {
        start: 0,
        count: 3,
        delay: 12,
        loops: false,
    },
    front_sheet: "Mount_MinecartMech",
    // `frontTextureGlow = MinecartMechMount[1]` (`Mount.cs:1531`). Data only
    // until the glow draw pass lands (S1-L-8b).
    front_glow_sheet: "Mount_MinecartMechGlow",
    x_offset: -1,
    y_offset: 11,
    player_y_offsets: &[9; 3],
    body_frame: 3,
    player_head_offset: 14,
    ..BASE
};

/// 13 MinecartWood (`Mount.cs:1591-1641`, hand-written): run/dash 10, accel
/// 0.03, jump 12, buff 184; the sheet is the content file `Mount_MinecartWood`
/// (`AssetInitializer.cs:439` — S1-L-8b name fix).
const MINECART_WOOD: MountData = MountData {
    id: 13,
    buff: 184,
    minecart: true,
    spawn_dust: 213,
    height_boost: 10,
    flight_time_max: 0,
    fall_damage: 1.0,
    run_speed: 10.0,
    dash_speed: 10.0,
    acceleration: 0.03,
    jump_height: 12,
    jump_speed: 5.15,
    block_extra_jumps: true,
    total_frames: 3,
    standing: FrameSet {
        start: 0,
        count: 1,
        delay: 12,
        loops: false,
    },
    running: FrameSet {
        start: 0,
        count: 3,
        delay: 12,
        loops: false,
    },
    front_sheet: "Mount_MinecartWood",
    x_offset: 1,
    y_offset: 13,
    player_y_offsets: &[8; 3],
    body_frame: 3,
    player_head_offset: 14,
    ..BASE
};

/// 16 FishMinecart (`Mount.cs:1044-1086`, hand-written; identical to the
/// `SetAsMinecart` body), buff 210; the sheet is the content file
/// `Mount_MinecartMineCarp` (`AssetInitializer.cs:441` — S1-L-8b name fix).
const FISH_MINECART: MountData = MountData {
    id: 16,
    buff: 210,
    ..cart(16, 210, "Mount_MinecartMineCarp")
};

/// 39 DiggingMoleMinecart (`Mount.cs:1027-1042`): `SetAsMinecart` +
/// yOffset −2, front-extra `Extra_165`, run/dash 6, accel 0.02.
const MOLE_MINECART: MountData = MountData {
    id: 39,
    buff: 272,
    ..MountData {
        y_offset: 13 - 2,
        front_extra_sheet: "Extra_165",
        run_speed: 6.0,
        dash_speed: 6.0,
        acceleration: 0.02,
        ..cart(39, 272, "Extra_155")
    }
};

// The plain `SetAsMinecart` cosmetic rows (`Mount.cs:966-1024`). The desert
// cart's sheet is the content file `Mount_MinecartDesert`
// (`AssetInitializer.cs:440` — S1-L-8b name fix).
const DESERT_MINECART: MountData = cart(15, 208, "Mount_MinecartDesert");
const BEE_MINECART: MountData = cart(18, 220, "Extra_108");
const LADYBUG_MINECART: MountData = cart(19, 222, "Extra_109");
const PIGRON_MINECART: MountData = cart(20, 224, "Extra_110");
const SUNFLOWER_MINECART: MountData = cart(21, 226, "Extra_111");
const HELL_MINECART: MountData = cart(22, 228, "Extra_112");
// 24 Shroom (`Mount.cs:983-985`): `Extra_116` is `frontTextureGlow` (`:985`),
// NOT a front-extra — S1-L-8b moves it to the glow slot (data only).
const SHROOM_MINECART: MountData = MountData {
    front_glow_sheet: "Extra_116",
    ..cart(24, 231, "Extra_115")
};
const AMETHYST_MINECART: MountData = cart(25, 233, "Extra_117");
const TOPAZ_MINECART: MountData = cart(26, 235, "Extra_118");
const SAPPHIRE_MINECART: MountData = cart(27, 237, "Extra_119");
const EMERALD_MINECART: MountData = cart(28, 239, "Extra_120");
const RUBY_MINECART: MountData = cart(29, 241, "Extra_121");
const DIAMOND_MINECART: MountData = cart(30, 243, "Extra_122");
const AMBER_MINECART: MountData = cart(31, 245, "Extra_123");
const BEETLE_MINECART: MountData = cart(32, 247, "Extra_124");
const MEOWMERE_MINECART: MountData = cart(33, 249, "Extra_125");
const PARTY_MINECART: MountData = cart(34, 251, "Extra_126");
const PIRATE_MINECART: MountData = cart(35, 253, "Extra_127");
const STEAMPUNK_MINECART: MountData = cart(36, 255, "Extra_128");
// 38 Coffin/Hellwizard (`Mount.cs:1026-1031`): `backTexture = frontTexture`
// (`:1030`) — the cart draws behind the player too (S1-L-8b fills the slot).
const HELLWIZARD_MINECART: MountData = MountData {
    back_sheet: "Extra_150",
    ..cart(38, 269, "Extra_150")
};
// 51/53 the fart carts (`Mount.cs:1096-1110`): `SetAsMinecart(..., -10, -8)`
// plus `spawnDust = 211` (`:1098`/`:1106` — S1-L-8b; the shared body's 213 is
// overwritten).
const FART_MINECART: MountData = MountData {
    spawn_dust: 211,
    ..cart_vo(51, 338, "Extra_246")
};
const TERRA_FART_MINECART: MountData = MountData {
    spawn_dust: 211,
    ..cart_vo(53, 346, "Extra_251")
};
// -- S1-L-9: the sixteen rows `Mount.Initialize` never ported -----------------

/// 4 Turtle (`Mount.cs:1112-1162`): the reef rider — buff 131, heightBoost
/// 26, fallDamage 1, run 2 / dash 5 / swim 10, accel 0.08, jump 12/3.7 and
/// twelve frames of a 26 px player offset.
const TURTLE: MountData = MountData {
    id: 4,
    buff: 131,
    height_boost: 26,
    flight_time_max: 0,
    fall_damage: 1.0,
    run_speed: 2.0,
    dash_speed: 5.0,
    swim_speed: 10.0,
    acceleration: 0.08,
    jump_height: 12,
    jump_speed: 3.7,
    spawn_dust: 56,
    total_frames: 12,
    standing: FrameSet { start: 0, count: 1, delay: 12, loops: false },
    running: FrameSet { start: 0, count: 6, delay: 12, loops: false },
    flying: FrameSet { start: 0, count: 0, delay: 0, loops: false },
    in_air: FrameSet { start: 3, count: 1, delay: 12, loops: false },
    idle: FrameSet { start: 0, count: 0, delay: 0, loops: false },
    swim: FrameSet { start: 6, count: 6, delay: 12, loops: false },
    back_sheet: "Mount_Turtle",
    x_offset: 1,
    y_offset: 13,
    player_y_offsets: &[26; 12],
    body_frame: 3,
    player_head_offset: 28,
    ..BASE
};

/// 17 GolfCartSomebodySaveMe (`Mount.cs:1694-1739`): buff 212, heightBoost
/// 16, fallDamage 0.2, run 8, accel 0.25, jump 20/8.01 — the four-frame
/// cart with the `Extra_96`/`Extra_97` sheet pair.
const GOLF_CART: MountData = MountData {
    id: 17,
    buff: 212,
    height_boost: 16,
    flight_time_max: 0,
    fall_damage: 0.2,
    run_speed: 8.0,
    acceleration: 0.25,
    jump_height: 20,
    jump_speed: 8.01,
    spawn_dust: 15,
    total_frames: 4,
    standing: FrameSet { start: 0, count: 1, delay: 12, loops: false },
    running: FrameSet { start: 0, count: 4, delay: 12, loops: false },
    flying: FrameSet { start: 0, count: 0, delay: 0, loops: false },
    in_air: FrameSet { start: 1, count: 1, delay: 12, loops: false },
    idle: FrameSet { start: 0, count: 0, delay: 0, loops: false },
    swim: FrameSet { start: 1, count: 1, delay: 12, loops: false },
    back_sheet: "Extra_97",
    back_extra_sheet: "Extra_96",
    x_offset: 2,
    y_offset: 1, // `:1713` — `17 - heightBoost`
    player_y_offsets: &[8; 4],
    body_frame: 3,
    player_head_offset: 18,
    ..BASE
};

/// 37 Flamingo (`Mount.cs:1796-1841`): buff 265, heightBoost 12, fallDamage
/// 0.2, run 6, accel 0.15, jump 14/6.01 — ten frames, body row 4.
const FLAMINGO: MountData = MountData {
    id: 37,
    buff: 265,
    height_boost: 12,
    flight_time_max: 0,
    fall_damage: 0.2,
    run_speed: 6.0,
    acceleration: 0.15,
    jump_height: 14,
    jump_speed: 6.01,
    spawn_dust: 282,
    total_frames: 10,
    standing: FrameSet { start: 0, count: 1, delay: 12, loops: false },
    running: FrameSet { start: 2, count: 7, delay: 20, loops: false },
    flying: FrameSet { start: 0, count: 0, delay: 0, loops: false },
    in_air: FrameSet { start: 1, count: 1, delay: 12, loops: false },
    idle: FrameSet { start: 0, count: 0, delay: 0, loops: false },
    // `:1830-1832`: the swim band rides the running count/start with its own
    // 10-tick delay.
    swim: FrameSet { start: 2, count: 7, delay: 10, loops: false },
    back_sheet: "Extra_149",
    x_offset: 5,
    y_offset: 1,
    player_y_offsets: &[20; 10],
    body_frame: 4,
    player_head_offset: 20,
    ..BASE
};

/// 45 SpookyWood (`Mount.cs:1940-1985`): buff 280, heightBoost 25, fallDamage
/// 0.1, run 12 / dash 16, accel 0.5, jump 14/7 — one of the three rows that
/// emit light (`:1951-1952`; the Drill `:1289-1290` and the LavaShark
/// `:2159-2160` are the others).
const SPOOKY_WOOD: MountData = MountData {
    id: 45,
    buff: 280,
    height_boost: 25,
    flight_time_max: 0,
    fall_damage: 0.1,
    emits_light: true,
    light_color: (0.6, 0.4, 0.35),
    run_speed: 12.0,
    dash_speed: 16.0,
    acceleration: 0.5,
    jump_height: 14,
    jump_speed: 7.0,
    spawn_dust: 6,
    total_frames: 8,
    standing: FrameSet { start: 0, count: 1, delay: 20, loops: false },
    running: FrameSet { start: 2, count: 6, delay: 20, loops: false },
    flying: FrameSet { start: 0, count: 0, delay: 0, loops: false },
    in_air: FrameSet { start: 1, count: 1, delay: 20, loops: false },
    idle: FrameSet { start: 0, count: 0, delay: 0, loops: false },
    swim: FrameSet { start: 2, count: 6, delay: 20, loops: false },
    back_sheet: "Extra_167",
    // `GlowMask[283]` (`Mount.cs:1980`) = `Images/Glow_283`
    // (`AssetInitializer.cs:352-354`) — S1-L-8b name fix; data only.
    back_glow_sheet: "Glow_283",
    x_offset: 2, // `:1962` — the second write wins
    y_offset: 1,
    player_y_offsets: &[30; 8],
    body_frame: 0,
    player_head_offset: 20,
    ..BASE
};

/// 46 Santank (`Mount.cs:1987-2039`): the ability mount — buff 281, the
/// 40-charge ability (`:1994-1996`), run 8 / dash 8, accel 0.4, jump 8/9.01
/// and 27 frames.
static SANTANK_Y_OFFSETS: [i32; 27] = [
    4, 6, 6, 4, 4, 4, 4, 6, 6, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
];
const SANTANK: MountData = MountData {
    id: 46,
    buff: 281,
    height_boost: 0,
    flight_time_max: 0,
    fatigue_max: 0,
    fall_damage: 0.0,
    ability_charge_max: 40,
    ability_cooldown: 40,
    ability_duration: 0,
    run_speed: 8.0,
    dash_speed: 8.0,
    acceleration: 0.4,
    jump_height: 8,
    jump_speed: 9.01,
    block_extra_jumps: false,
    spawn_dust: 15,
    total_frames: 27,
    standing: FrameSet { start: 0, count: 1, delay: 12, loops: false },
    running: FrameSet { start: 0, count: 11, delay: 12, loops: false },
    flying: FrameSet { start: 0, count: 0, delay: 0, loops: false },
    in_air: FrameSet { start: 1, count: 11, delay: 12, loops: false },
    idle: FrameSet { start: 0, count: 0, delay: 0, loops: false },
    swim: FrameSet { start: 0, count: 11, delay: 12, loops: false },
    front_sheet: "Extra_168",
    front_extra_sheet: "Extra_168",
    x_offset: 1,
    y_offset: 1,
    player_y_offsets: &SANTANK_Y_OFFSETS,
    body_frame: 3,
    player_head_offset: 2,
    ..BASE
};

/// 52 Wolf (`Mount.cs:2246-2308`): buff 342, fallDamage 0.1, run 9.5, accel
/// 0.18, jump 18/9.01 — fifteen frames with the mouth-position delegate.
static WOLF_Y_OFFSETS: [i32; 15] = [0, -14, -10, -8, 18, -2, 0, 2, 4, 4, 2, 0, 4, 2, -4];
const WOLF: MountData = MountData {
    id: 52,
    buff: 342,
    flight_time_max: 0,
    fall_damage: 0.1,
    run_speed: 9.5,
    acceleration: 0.18,
    jump_height: 18,
    jump_speed: 9.01,
    spawn_dust: 31,
    total_frames: 15,
    standing: FrameSet { start: 0, count: 1, delay: 12, loops: false },
    running: FrameSet { start: 5, count: 6, delay: 20, loops: false },
    flying: FrameSet { start: 0, count: 0, delay: 0, loops: false },
    in_air: FrameSet { start: 12, count: 3, delay: 12, loops: false },
    idle: FrameSet { start: 0, count: 0, delay: 0, loops: false },
    swim: FrameSet { start: 12, count: 3, delay: 12, loops: false },
    front_sheet: "Extra_253",
    front_extra_sheet: "Extra_254",
    x_offset: 4,
    y_offset: -1,
    player_y_offsets: &WOLF_Y_OFFSETS,
    body_frame: 3,
    ..BASE
};

/// 54 Velociraptor (`Mount.cs:2310-2360`): buff 370, fallDamage 0.1, run 4.5
/// / dash 7.5, accel 0.15, jump 15/6.01 — the only row of the new set with a
/// flying band (5 frames from 16).
const VELOCIRAPTOR: MountData = MountData {
    id: 54,
    buff: 370,
    flight_time_max: 0,
    fall_damage: 0.1,
    run_speed: 4.5,
    dash_speed: 7.5,
    acceleration: 0.15,
    jump_height: 15,
    jump_speed: 6.01,
    spawn_dust: 31,
    total_frames: 21,
    standing: FrameSet { start: 0, count: 1, delay: 12, loops: false },
    running: FrameSet { start: 8, count: 7, delay: 20, loops: false },
    flying: FrameSet { start: 16, count: 5, delay: 6, loops: false },
    in_air: FrameSet { start: 5, count: 2, delay: 6, loops: false },
    idle: FrameSet { start: 0, count: 0, delay: 0, loops: false },
    swim: FrameSet { start: 5, count: 2, delay: 6, loops: false },
    front_sheet: "Extra_271",
    front_extra_sheet: "Extra_272",
    x_offset: 4,
    y_offset: -4,
    player_y_offsets: &[0; 21],
    body_frame: 0,
    ..BASE
};

/// 55 Rat (`Mount.cs:2362-2415`): buff 374, fallDamage 0.1, run 4.5 / dash 7.5,
/// accel 0.15, jump 15/6.01 — the transformation mount that dismounts on item
/// use (`:2371`).
const RAT: MountData = MountData {
    id: 55,
    buff: 374,
    flight_time_max: 0,
    fall_damage: 0.1,
    dismounts_on_item_use: true,
    run_speed: 4.5,
    dash_speed: 7.5,
    acceleration: 0.15,
    jump_height: 15,
    jump_speed: 6.01,
    spawn_dust: 31,
    total_frames: 16,
    standing: FrameSet { start: 0, count: 1, delay: 12, loops: false },
    running: FrameSet { start: 4, count: 7, delay: 12, loops: false },
    dashing: FrameSet { start: 11, count: 5, delay: 16, loops: false },
    flying: FrameSet { start: 0, count: 0, delay: 0, loops: false },
    in_air: FrameSet { start: 12, count: 3, delay: 6, loops: false },
    idle: FrameSet { start: 0, count: 4, delay: 16, loops: false },
    swim: FrameSet { start: 12, count: 3, delay: 6, loops: false },
    front_sheet: "Extra_273",
    x_offset: 0,
    y_offset: -2,
    player_y_offsets: &[0; 16],
    body_frame: 3,
    player_head_offset: -30,
    // `DelegateMethods.Mount.RatPlayerSize` (`Mount.cs:2366` →
    // `DelegateMethods.cs:190-194`): the rat body is 14×14.
    player_size_override: Some((14.0, 14.0)),
    ..BASE
};

/// `Mount.idleFrames_Rat` (`Mount.cs:6665-6669`) — the rat's own idle-frame
/// walk (the animator reads it instead of the linear band, `Mount.cs:4194-4208`).
const RAT_IDLE_FRAMES: [i32; 11] = [0, 1, 3, 2, 3, 2, 3, 2, 1, 0, 0];

/// 57-60 the roller skates — `Mount.SetAsRollerSkate` (`Mount.cs:2569-2605`):
/// the Minecart-tracks rider with additive movement stats and wings.
const fn roller_skate(id: i32, buff: i32) -> MountData {
    MountData {
        id,
        buff,
        spawn_dust: 31,
        can_ride_minecart_tracks: true,
        can_use_wings: true,
        movement_stats_are_additive: true,
        dash_speed: 1.0,
        run_speed: 1.0,
        acceleration: 0.1,
        jump_height: 2,
        jump_speed: 1.0,
        fall_damage: 1.0,
        block_extra_jumps: false,
        total_frames: 1,
        standing: FrameSet { start: 0, count: 1, delay: 6, loops: false },
        x_offset: 0,
        body_frame: 3,
        y_offset: 0,
        player_y_offsets: &[2],
        ..BASE
    }
}
const ROLLER_SKATES: MountData = roller_skate(57, 378);
const ROLLER_SKATES_GREEN: MountData = roller_skate(58, 379);
const ROLLER_SKATES_WHITE: MountData = roller_skate(59, 380);
const ROLLER_SKATES_PINK: MountData = roller_skate(60, 381);

/// 62-65 the Chillemt family — `Mount.SetAsChillet`
/// (`Mount.cs:2670-2745`): buff 387/388/391/392, heightBoost 4, fallDamage
/// 0.5, run 3, accel 0.32, walkingGraceTimeMax 10 and thirteen frames; the
/// hard-mode pair (64/65) rides the faster dash and jump (`:2677-2684`).
static CHILLET_Y_OFFSETS: [i32; 13] = [10, 10, 10, 10, 12, 16, 12, 10, 12, 16, 16, 14, 10];
const fn chillet(id: i32, buff: i32, back_sheet: &'static str, hardmode: bool) -> MountData {
    MountData {
        id,
        buff,
        spawn_dust: 15,
        height_boost: 4,
        flight_time_max: 0,
        fall_damage: 0.5,
        run_speed: 3.0,
        dash_speed: if hardmode { 9.0 } else { 6.5 },
        acceleration: 0.32,
        jump_height: if hardmode { 11 } else { 6 },
        jump_speed: if hardmode { 8.1 } else { 8.01 },
        walking_grace_time_max: 10,
        total_frames: 13,
        standing: FrameSet { start: 0, count: 4, delay: 12, loops: false },
        running: FrameSet { start: 6, count: 6, delay: 30, loops: false },
        dashing: FrameSet { start: 5, count: 7, delay: 130, loops: false },
        flying: FrameSet { start: 1, count: 6, delay: 6, loops: false },
        in_air: FrameSet { start: 4, count: 1, delay: 12, loops: false },
        idle: FrameSet { start: 0, count: 4, delay: 4, loops: true },
        swim: FrameSet { start: 4, count: 1, delay: 12, loops: false },
        back_sheet,
        x_offset: 7,
        body_frame: 3,
        y_offset: -10,
        player_y_offsets: &CHILLET_Y_OFFSETS,
        player_head_offset: 10,
        ..BASE
    }
}
const CHILLET: MountData = chillet(62, 387, "Extra_301", false);
const CHILLET_IGNIS: MountData = chillet(63, 388, "Extra_302", false);
const TRUSTY_CHILLET: MountData = chillet(64, 391, "Extra_301", true);
const TRUSTY_CHILLET_IGNIS: MountData = chillet(65, 392, "Extra_302", true);

pub(crate) static MOUNTS: &[MountData] = &[
    RUDOLPH,        // 0
    BUNNY,          // 1
    SLIME,          // 3
    SCUTLIX,        // 9
    UNICORN,        // 10
    BASILISK,       // 14
    PAINTED_HORSE,  // 40
    MAJESTIC_HORSE, // 41
    DARK_HORSE,     // 42
    POGO_STICK,     // 43
    WOF_GOAT,       // 47
    QUEEN_SLIME,    // 50
    PIGRON,         // 2
    BEE,            // 5
    UFO,            // 7
    DRILL,          // 8
    CUTE_FISHRON,   // 12
    WITCH_BROOM,    // 23
    PIRATE_SHIP,    // 44
    DARK_MAGE_BOOK, // 48
    LAVA_SHARK,     // 49
    BAT,            // 56
    PIXIE,          // 61
    MINECART,          // 6
    MINECART_MECH,     // 11
    MINECART_WOOD,     // 13
    FISH_MINECART,     // 16
    DESERT_MINECART,   // 15
    BEE_MINECART,      // 18
    LADYBUG_MINECART,  // 19
    PIGRON_MINECART,   // 20
    SUNFLOWER_MINECART,// 21
    HELL_MINECART,     // 22
    SHROOM_MINECART,   // 24
    AMETHYST_MINECART, // 25
    TOPAZ_MINECART,    // 26
    SAPPHIRE_MINECART, // 27
    EMERALD_MINECART,  // 28
    RUBY_MINECART,     // 29
    DIAMOND_MINECART,  // 30
    AMBER_MINECART,    // 31
    BEETLE_MINECART,   // 32
    MEOWMERE_MINECART, // 33
    PARTY_MINECART,    // 34
    PIRATE_MINECART,   // 35
    STEAMPUNK_MINECART,// 36
    HELLWIZARD_MINECART,// 38
    MOLE_MINECART,     // 39
    FART_MINECART,     // 51
    TERRA_FART_MINECART,// 53
    // -- S1-L-9: the sixteen rows `Mount.Initialize` never ported ------------
    TURTLE,            // 4
    GOLF_CART,         // 17
    FLAMINGO,          // 37
    SPOOKY_WOOD,       // 45
    SANTANK,           // 46
    WOLF,              // 52
    VELOCIRAPTOR,      // 54
    RAT,               // 55
    ROLLER_SKATES,     // 57
    ROLLER_SKATES_GREEN, // 58
    ROLLER_SKATES_WHITE, // 59
    ROLLER_SKATES_PINK,  // 60
    CHILLET,           // 62
    CHILLET_IGNIS,     // 63
    TRUSTY_CHILLET,    // 64
    TRUSTY_CHILLET_IGNIS, // 65
];

/// The saddle items — `Item.SetDefaults`' `Item.mountType` assignments:
/// `Item.cs:20779` Reindeer Bells 1914 -> Rudolph 0, `:24044` Fuzzy Carrot
/// 2428 -> Bunny 1, `:24068` Slimy Saddle 2430 -> Slime 3, `:26465` Brain
/// Scrambler 2771 -> Scutlix 9, `:30007` Blessed Apple 3260 -> Unicorn 10,
/// `:33385` Ancient Horn 3771 -> Basilisk 14, `:39363` the horse saddles
/// 4785-4787 -> 40-42, `:39395` Pogo Stick 4791 -> 43, `:39413` Wall Of
/// Flesh Goat Mount Item 4795 -> 44 + (4795 - 4792) = 47, `:40133` Queen
/// Slime Mount Saddle 4981 -> 50.
const MOUNT_ITEMS: [(u16, i32); 65] = [
    (1914, 0),
    (2428, 1),
    (2430, 3),
    (2771, 9),
    (3260, 10),
    (3771, 14),
    (4785, 40),
    (4786, 41),
    (4787, 42),
    (4791, 43),
    (4795, 47),
    (4981, 50),
    // `Item.SetDefaults`' `mountType` assignments (M2-MOUNT-2):
    // `:24056` Scaly Truffle 2429 -> 2, `:24307` Honeyed Goggles 2502 -> 5,
    // `:26440` Cosmic Car Key 2769 -> 7, `:26452` Drill Containment Unit
    // 2768 -> 8, `:30779` Shrimpy Truffle 3367 -> 12, `:37573` Witch Broom
    // 4444 -> 23, `:39413` 4792/4796 -> 44/48, `:39535` Superheated Blood
    // 4828 -> 49, `:44035` 5597 -> 56, `:44459` 5662 -> 61.
    (2429, 2),
    (2502, 5),
    (2769, 7),
    (2768, 8),
    (3367, 12),
    (4444, 23),
    (4792, 44),
    (4796, 48),
    (4828, 49),
    (5597, 56),
    (5662, 61),
    // The cart items (`Item.DefaultToMinecart`, `Item.cs:48395-48400` +
    // the per-item call sites): 2343 -> 6, 3353 -> 11, 4066 -> 15,
    // 4067 -> 16, 4426 -> 18, 4427 -> 19, 4428 -> 20, 4429 -> 21,
    // 4443 -> 22, 4450 -> 24, 4451 -> 25, 4452 -> 26, 4453 -> 27,
    // 4454 -> 28, 4455 -> 29, 4456 -> 30, 4467 -> 31, 4468 -> 32,
    // 4469 -> 33, 4470 -> 34, 4471 -> 35, 4472 -> 36, 4745 -> 38,
    // 4763 -> 39, 5125 -> 51, 5288 -> 53.
    (2343, 6),
    (3353, 11),
    (4066, 15),
    (4067, 16),
    (4426, 18),
    (4427, 19),
    (4428, 20),
    (4429, 21),
    (4443, 22),
    (4450, 24),
    (4451, 25),
    (4452, 26),
    (4453, 27),
    (4454, 28),
    (4455, 29),
    (4456, 30),
    (4467, 31),
    (4468, 32),
    (4469, 33),
    (4470, 34),
    (4471, 35),
    (4472, 36),
    (4745, 38),
    (4763, 39),
    (5125, 51),
    (5288, 53),
    // -- S1-L-9: the items of the sixteen new rows --------------------------
    // `Item.SetDefaults`' `Item.mountType` assignments: `:24194` Turtle Mount
    // 2491 -> 4, `:36685` Golf Cart 4264 -> 17, `:38778` Flamingo Feather
    // 4716 -> 37, `:39413` the `44 + type - 4792` run (4793 -> 45, 4794 ->
    // 46), `:41600` Wolf Mount 5130 -> 52, `:43366` Velociraptor 5510 ->
    // 54, `:43512` Rat 5525 -> 55 (S1-P-i41: 5524 is the Chlorophyte Visor,
    // `:43504-43509` headSlot 283 — NOT a saddle), `:44056` Roller Skates
    // 5600 -> 57 (5599 is the Retro Monolith, `:44051-44053`), `:44300`
    // 5640 -> 58 (5638/5639 are the Torch God music boxes, `:44293-44297`),
    // `:44313` 5641 -> 59, `:44326` 5642 -> 60, `:44519` Chillet 5665 -> 62,
    // `:44531` 5666 -> 63, `:47394` 6150 -> 64, `:47407` 6151 -> 65.
    (2491, 4),
    (4264, 17),
    (4716, 37),
    (4793, 45),
    (4794, 46),
    (5130, 52),
    (5510, 54),
    (5525, 55),
    (5600, 57),
    (5640, 58),
    (5641, 59),
    (5642, 60),
    (5665, 62),
    (5666, 63),
    (6150, 64),
    (6151, 65),
];

/// `Item.mountType` for the ported rows.
/// `MountID.Sets.DoesNotOverrideLegFrames` (`MountID.cs:39`) — the roller-skate
/// mounts (57-60) let the legs keep their own rows.
pub(crate) fn does_not_override_leg_frames(id: i32) -> bool {
    matches!(id, 57..=60)
}

/// `MountID.Sets.DoesNotOverrideBodyFrames` (`MountID.cs:38`) — the same set.
pub(crate) fn does_not_override_body_frames(id: i32) -> bool {
    does_not_override_leg_frames(id)
}

/// `MountID.Sets.IsRollerSkates` (`MountID.cs:42`) — the same set again; the
/// walk branch locks their leg counter to `2` at full speed.
pub(crate) fn is_roller_skates(id: i32) -> bool {
    does_not_override_leg_frames(id)
}

/// `MountID.Sets.Cart` (`MountID.cs:43`): the 27 minecart ids. `Mount.CanMount`
/// exempts them from the grapple gate (`Mount.cs:6599`).
pub(crate) fn is_cart(id: i32) -> bool {
    matches!(
        id,
        6 | 11 | 13 | 15 | 16 | 18..=22 | 24..=36 | 38 | 39 | 51 | 53
    )
}

/// `MountID.Sets.CanUseHooks` (`MountID.cs:36`): the mounts that keep a hook
/// usable while ridden — the second half of the `CanMount` grapple exemption.
pub(crate) fn can_use_hooks(id: i32) -> bool {
    matches!(id, 54 | 57..=60)
}

/// `MountID.Sets.DontDismountWhenCCed` (`MountID.cs:47`): the Rat 55, the
/// Bat 56 and the Pixie 61 — `Mount.CanDismountWithResult` (`Mount.cs:6615`)
/// refuses the dismount while the rider is `CCed`.
pub(crate) fn dont_dismount_when_cced(id: i32) -> bool {
    matches!(id, 55 | 56 | 61)
}

/// `Mount.DismountCheckResult` (`Mount.cs:268-273`) — the dismount gate's
/// outcome; `QuickMount`'s no-space ladder reads it (`Player.cs:6049`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DismountCheck {
    Succeeded,
    FailedCCed,
    FailedNoSpace,
}

pub(crate) fn mount_type_of_item(item: u16) -> Option<i32> {
    MOUNT_ITEMS
        .iter()
        .find(|(id, _)| *id == item)
        .map(|(_, m)| *m)
}

/// `BuffID.Sets.MountType` for the ported rows — each mount keeps its own
/// buff alive (`MountData.buff`).
pub(crate) fn mount_type_of_buff(buff: i32) -> Option<i32> {
    MOUNTS.iter().find(|row| row.buff == buff).map(|row| row.id)
}

/// `MountID.Count` (`MountID.cs:181`) — the number of mount rows the table
/// carries; the renderer's per-mount sheet arrays are sized by it.
pub(crate) const MOUNT_COUNT: usize = 66;

/// The index of a mount's row in [`MOUNTS`] — the key of the renderer's
/// per-mount sheet tables.
pub(crate) fn mount_index(id: i32) -> Option<usize> {
    MOUNTS.iter().position(|row| row.id == id)
}

/// `Mount.mounts[m]` lookup; `None` for ids the partial table does not carry.
pub(crate) fn mount_data(id: i32) -> Option<&'static MountData> {
    // The rows are `const`s, so a found reference is genuinely 'static.
    if let Some(pos) = MOUNTS.iter().position(|row| row.id == id) {
        Some(&MOUNTS[pos])
    } else {
        None
    }
}

/// The SuperCart stat overrides (`Mount.cs:373-381` statics, read :478-481,
/// :490-493, :502-505, :2829-2832, :2853-2856) — the values the static
/// constructor writes (`Mount.cs:6660-6664`).
const SUPER_CART_RUN_SPEED: f32 = 20.0;
const SUPER_CART_DASH_SPEED: f32 = 20.0;
const SUPER_CART_ACCELERATION: f32 = 0.1;
const SUPER_CART_JUMP_HEIGHT: i32 = 15;
const SUPER_CART_JUMP_SPEED: f32 = 5.15;

/// What the caller must apply after a successful [`MountState::set_mount`]:
/// the buff swap plus the hitbox growth (`SetMount` `Mount.cs:6226`,
/// :6264-6277 — the PlayerSize-delegate branch does not exist in this set).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Equip {
    /// `ClearBuff(old data.buff)` on a swap (:6203).
    pub clear_buff: Option<i32>,
    /// `AddBuff(data.buff, 3600)` (:6226).
    pub add_buff: i32,
    /// The new player height is `42 + height_boost`, bottom-anchored.
    pub height_boost: i32,
    /// `Mount.SetMount`'s swap branch (`:6202-6213`) only runs when the mount
    /// was already active — the cart flip / last boost / full rotation are
    /// cleared there.
    pub was_active: bool,
    /// `Mount.AnyTrackRider` (`Mount.cs:554-567`) on the OLD row: a minecart
    /// or a minecart-track rider (the roller skates). Only meaningful with
    /// `was_active`.
    pub track_rider: bool,
    /// The NEW row's `PlayerSize` delegate result (`Mount.cs:6244-6263`):
    /// `Some((w, h))` replaces the `42 + heightBoost` growth with the
    /// bottom-center resize to that exact size (Rat 14×14, Bat 20×18,
    /// Pixie 8×14); `None` takes the height-boost growth branch.
    pub size_override: Option<(f32, f32)>,
}

/// The mount instance state — `Mount` (`Mount.cs:325-383`). Plain data so the
/// per-tick `Player` clone (the draw's raw fixed-step copy) stays cheap; the
/// per-type payload
/// `_mountSpecificData` (drill/dash-toggle/fly-permission) belongs to
/// M2-MOUNT-2/4 and is not carried yet.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MountState {
    active: bool,
    /// `Mount._type`; -1 = none (after `Reset` :683).
    kind: i32,
    flip_draw: bool,
    frame: i32,
    frame_counter: f32,
    frame_extra: i32,
    frame_extra_counter: f32,
    frame_state: i32,
    fly_time: i32,
    idle_time: i32,
    idle_time_next: i32,
    fatigue: f32,
    fatigue_max: f32,
    ability_charging: bool,
    ability_charge: i32,
    ability_cooldown: i32,
    ability_duration: i32,
    ability_active: bool,
    aiming: bool,
    should_super_cart: bool,
    walking_grace_time_left: i32,
    /// `SelectiveFlyingMountData.allowedToFly` (`Mount.cs:70-80`) — the
    /// Velociraptor's (54) `_mountSpecificData` payload: `UpdateAfterEquips`
    /// (`Mount.cs:2989-2997`) sets it to `wingsLogic > 0` every tick;
    /// `CanFly` (`:2866-2869`) reads it BEFORE the `flightTimeMax` gate.
    /// The other `_mountSpecificData` payloads (Drill 8, ExtraFrame 35 —
    /// `SetMount` `:6287-6294`) are M2-MOUNT-4 state and not carried yet.
    selective_allowed_to_fly: bool,
    #[cfg(test)]
    test_can_use_wings: Option<bool>,
    #[cfg(test)]
    test_additive: Option<bool>,
    #[cfg(test)]
    test_dismounts_on_item_use: Option<bool>,
    /// The splitmix64 stream behind the idle-pick stand-in (see
    /// [`MountState::update_frame`]); deliberately NOT reset by
    /// [`MountState::reset`] — the vanilla `Main.rand` it stands in for is
    /// global (`Mount.cs:675-694` never touches it either).
    idle_rng: u64,
}

impl Default for MountState {
    /// `Mount()` → `Reset()` (`Mount.cs:669-694`).
    fn default() -> Self {
        let mut state = MountState {
            active: false,
            kind: -1,
            flip_draw: false,
            frame: 0,
            frame_counter: 0.0,
            frame_extra: 0,
            frame_extra_counter: 0.0,
            frame_state: 0,
            fly_time: 0,
            idle_time: 0,
            idle_time_next: -1,
            fatigue: 0.0,
            fatigue_max: 0.0,
            ability_charging: false,
            ability_charge: 0,
            ability_cooldown: 0,
            ability_duration: 0,
            ability_active: false,
            aiming: false,
            should_super_cart: false,
            walking_grace_time_left: 0,
            selective_allowed_to_fly: false,
            #[cfg(test)]
            test_can_use_wings: None,
            #[cfg(test)]
            test_additive: None,
            #[cfg(test)]
            test_dismounts_on_item_use: None,
            idle_rng: IDLE_RNG_SEED,
        };
        state.reset();
        state
    }
}

/// The seed of the animator's private splitmix64 stream. The vanilla idle
/// pick reads `Main.rand.Next(900, 1500)` (`Mount.cs:4165`); the port-wide
/// `Main.rand` sequence is not reproduced (docs/ANALYSIS.md "the port-wide
/// RNG deviation"), so the animator owns a deterministic stand-in instead —
/// the same tick sequence always picks the same idle delays.
const IDLE_RNG_SEED: u64 = 0x5EED_C0FF_EE12_3456;

fn idle_next(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn idle_range(state: &mut u64, lo: i32, hi: i32) -> i32 {
    lo + (idle_next(state) % (hi - lo) as u64) as i32
}

impl MountState {
    /// `Mount.Reset` (`Mount.cs:675-694`) — note the vanilla quirk that
    /// `_fatigue`, `_abilityCooldown` and `_abilityDuration` are NOT zeroed.
    pub(crate) fn reset(&mut self) {
        self.active = false;
        self.kind = -1;
        self.flip_draw = false;
        self.frame = 0;
        self.frame_counter = 0.0;
        self.frame_extra = 0;
        self.frame_extra_counter = 0.0;
        self.frame_state = 0;
        self.fly_time = 0;
        self.idle_time = 0;
        self.idle_time_next = -1;
        self.fatigue_max = 0.0;
        self.ability_charging = false;
        self.ability_charge = 0;
        self.aiming = false;
        self.should_super_cart = false;
        self.walking_grace_time_left = 0;
        // `SetMount` builds a fresh `SelectiveFlyingMountData`
        // (`Mount.cs:6295-6298` → `:76-80`, `allowedToFly = false`); the
        // reset mirrors the fresh-payload default.
        self.selective_allowed_to_fly = false;
    }

    pub(crate) fn is_active(&self) -> bool {
        self.active
    }

    /// `Mount.UpdateEffects` (`Mount.cs:4798`): the Super Cart flag rides the
    /// cart set (`MountID.Sets.Cart`) and the player's unlock/enable pair.
    /// The flag then overrides the run/dash/acceleration/jump getters
    /// (`:478-505`, `:2829-2856`).
    pub(crate) fn set_should_super_cart(&mut self, value: bool) {
        self.should_super_cart = value;
    }

    pub(crate) fn kind(&self) -> i32 {
        self.kind
    }

    pub(crate) fn frame(&self) -> i32 {
        self.frame
    }

    pub(crate) fn frame_state(&self) -> i32 {
        self.frame_state
    }

    /// `Mount.RunningGraceTime` (:403).
    pub(crate) fn running_grace_time(&self) -> i32 {
        self.walking_grace_time_left
    }

    /// `Mount.UpdateFrame` (`Mount.cs:3581-4506`) — the ground subset: the
    /// head (state change, the idle timer, the walking grace) plus cases 0-4
    /// (standing/idle, running, in-air, flying, swimming). Case 5 (dashing)
    /// waits for the dash mounts (M2-MOUNT-4) — the state selector below
    /// never emits it for this table. The per-type extras of
    /// `:3637-4142` (drill rotation, bee wings, chillet dust) have no rows
    /// in this table yet.
    pub(crate) fn update_frame(
        &mut self,
        state: i32,
        vx: f32,
        vy: f32,
        just_jumped: bool,
        control_down: bool,
        sliding: bool,
        dash: i32,
        dash_delay: i32,
    ) {
        let Some(row) = self.data() else { return };
        // `:3583-3611`: a state change resets the counter — unless the row is
        // one of the four that keep their counter across the switch (48/56/61
        // and the rat 55, whose sliding arm rides `Player.sliding`).
        if self.frame_state != state {
            // `:3585-3588`: the bat (56) resets on the dash-state switch.
            if self.kind == 56
                && ((self.frame_state == 5 && (dash <= 0 || dash_delay >= 0))
                    || (self.frame_state != 5 && dash > 0 && dash_delay < 0))
            {
                self.frame_counter = 0.0;
            }
            self.frame_state = state;
            let mut flag = true;
            if self.kind == 48 && (state == 1 || state == 2) {
                flag = false; // :3591-3594
            }
            if self.kind == 56 && (state == 1 || state == 4 || state == 2) {
                flag = false; // :3595-3598
            }
            if self.kind == 61 && (state == 1 || state == 4 || state == 2) {
                flag = false; // :3599-3602
            }
            if self.kind == 55 && (state == 1 || state == 5 || state == 4 || (state == 2 && sliding))
            {
                flag = false; // :3603-3606
            }
            if flag {
                self.frame_counter = 0.0;
            }
        }
        // `:3612-3615`: any non-standing state clears the idle timer.
        if state != FRAME_STANDING {
            self.idle_time = 0;
        }
        // `:3620-3631`: the walking grace — full on any rest tick, decays in
        // the air, zeroed by a jump or a held downward fall.
        if vy == 0.0 {
            self.walking_grace_time_left = row.walking_grace_time_max;
        } else if self.walking_grace_time_left > 0 {
            self.walking_grace_time_left -= 1;
        }
        if just_jumped || (control_down && vy > 0.0) {
            self.walking_grace_time_left = 0;
        }
        // `:3632-3636`: `emitsLight` — no row in this table sets it.
        match state {
            FRAME_STANDING => {
                // `:4145-4181`: the idle timer. The per-type ranges are the
                // C#'s (`Main.rand.Next(min, max)`): 17 → 120-300, 2 → 600-900
                // (the pigron row is absent from this table), 55 → 120-420;
                // every other row picks 900-1500. The stand-in stream
                // replaces `Main.rand` (docs/ANALYSIS.md "the port-wide RNG
                // deviation"), so the extra vanilla draw of the per-type
                // overrides is not reproduced.
                if row.idle.count != 0 && self.idle_time == 0 {
                    let (lo, hi) = match self.kind {
                        17 => (120, 300),
                        55 => (120, 420),
                        _ => (900, 1500),
                    };
                    self.idle_time_next = idle_range(&mut self.idle_rng, lo, hi);
                }
                if row.idle.count != 0 {
                    self.idle_time += 1;
                }
                self.frame_counter += 1.0;
                if row.idle.count != 0 && self.idle_time >= self.idle_time_next {
                    // `:4194-4208`: the rat (55) walks its own idle-frame table
                    // instead of the linear band.
                    if self.kind == 55 {
                        let num32 = ((self.idle_time - self.idle_time_next) as f32
                            / row.idle.delay as f32
                            / row.idle.count as f32
                            * RAT_IDLE_FRAMES.len() as f32) as i32;
                        if num32 >= RAT_IDLE_FRAMES.len() as i32 {
                            self.frame_counter = 0.0;
                            self.frame = row.standing.start;
                            self.idle_time = 0;
                        } else {
                            self.frame = row.idle.start + RAT_IDLE_FRAMES[num32 as usize];
                        }
                        return;
                    }
                    // `:4198-4214`: the idle band; type 55's rat table is
                    // outside this set.
                    let num33 = ((self.idle_time - self.idle_time_next) as f32
                        / row.idle.delay as f32) as i32;
                    if num33 >= row.idle.count {
                        if row.idle.loops {
                            // `:4208-4211`: loop back (the chillet re-pick
                            // `Main.rand.Next(20)` is outside this set).
                            self.idle_time = self.idle_time_next;
                            self.frame = row.idle.start;
                        } else {
                            // `:4212-4217`: back to standing.
                            self.frame_counter = 0.0;
                            self.frame = row.standing.start;
                            self.idle_time = 0;
                        }
                    } else {
                        self.frame = row.idle.start + num33;
                        // `:4222-4236`: the out-of-range guards.
                        if self.frame < row.idle.start
                            || self.frame >= row.idle.start + row.idle.count
                        {
                            self.frame = if row.idle.loops {
                                row.idle.start
                            } else {
                                row.standing.start
                            };
                        }
                    }
                } else {
                    // `:4242-4250`: the standing loop.
                    if self.frame_counter > row.standing.delay as f32 {
                        self.frame_counter -= row.standing.delay as f32;
                        self.frame += 1;
                    }
                    if self.frame < row.standing.start
                        || self.frame >= row.standing.start + row.standing.count
                    {
                        self.frame = row.standing.start;
                    }
                }
            }
            FRAME_RUNNING => {
                // `:4262-4321`: the counter speed — the Queen Slime (50)
                // half-speed override fires here; the 9/46/44/48/55/56 arms
                // reference rows outside this table.
                let num25 = if self.kind == 50 {
                    vx.abs() * 0.5
                } else {
                    vx.abs()
                };
                self.frame_counter += num25;
                // `num25 >= 0` always holds without the Scutlix flipDraw
                // reversal (`:4299-4320`'s negative branch).
                if self.frame_counter > row.running.delay as f32 {
                    self.frame_counter -= row.running.delay as f32;
                    self.frame += 1;
                }
                if self.frame < row.running.start
                    || self.frame >= row.running.start + row.running.count
                {
                    self.frame = row.running.start;
                }
            }
            FRAME_IN_AIR => {
                // `:4347-4373`: one step per in-air delay (the 56/61 speed
                // and the 62-65/4/52/55 specials are outside this set).
                self.frame_counter += 1.0;
                if self.frame_counter > row.in_air.delay as f32 {
                    self.frame_counter -= row.in_air.delay as f32;
                    self.frame += 1;
                }
                if self.frame < row.in_air.start
                    || self.frame >= row.in_air.start + row.in_air.count
                {
                    self.frame = row.in_air.start;
                }
            }
            FRAME_FLYING => {
                // `:4323-4345`: the flight loop (the 56/61 speed clamp and
                // the 54 budget shortcut are outside this set).
                self.frame_counter += 1.0;
                if self.frame_counter > row.flying.delay as f32 {
                    self.frame_counter -= row.flying.delay as f32;
                    self.frame += 1;
                }
                if self.frame < row.flying.start
                    || self.frame >= row.flying.start + row.flying.count
                {
                    self.frame = row.flying.start;
                }
            }
            FRAME_SWIMMING => {
                // `:4460-4475`: the swim loop paces on the speed average.
                self.frame_counter += (vx.abs() + vy.abs()) / 2.0;
                if self.frame_counter > row.swim.delay as f32 {
                    self.frame_counter -= row.swim.delay as f32;
                    self.frame += 1;
                }
                if self.frame < row.swim.start
                    || self.frame >= row.swim.start + row.swim.count
                {
                    self.frame = row.swim.start;
                }
            }
            _ => {
                // `:4476-4505` (dashing) waits for the dash mounts.
            }
        }
    }

    pub(crate) fn fly_time(&self) -> i32 {
        self.fly_time
    }

    pub(crate) fn data(&self) -> Option<&'static MountData> {
        if !self.active {
            return None;
        }
        mount_data(self.kind)
    }

    /// `Mount.AllowDirectionChange` (`Mount.cs:630-639`): true for every
    /// mount except the Scutlix (9), whose ability cooldown must be under
    /// half the row's (`_abilityCooldown < _data.abilityCooldown / 2`).
    /// An inactive mount allows the change (`_type == -1` never equals 9).
    /// S1-P-i05: the run-key direction lock (`Player.cs:19837`) reads it.
    pub(crate) fn allow_direction_change(&self) -> bool {
        if self.kind == 9 {
            return self
                .data()
                .is_some_and(|d| self.ability_cooldown < d.ability_cooldown / 2);
        }
        true
    }

    /// `Mount.SetMount` (`Mount.cs:6196-6305`) — the state transitions; the
    /// buff add/clear and the hitbox growth are returned for the caller
    /// (the net SendData(13) :6222-6225 and the sound/dust effects have no port
    /// counterpart yet). `wet` is `Player.wet` (the `m == 5` guard :6198).
    pub(crate) fn set_mount(&mut self, m: i32, wet: bool) -> Option<Equip> {
        let old_buff = self.data().map(|d| d.buff);
        // :6202-6213 — the swap branch reads the OLD row: `AnyTrackRider`
        // (`:554-567`) is a minecart or a minecart-track rider.
        let was_active = self.active;
        let track_rider = self
            .data()
            .map(|d| d.minecart || d.can_ride_minecart_tracks)
            .unwrap_or(false);
        if self.kind == m || m <= -1 || m >= 66 || (m == 5 && wet) || mount_data(m).is_none() {
            return None;
        }
        let row = mount_data(m).unwrap();
        if !self.active {
            self.active = true; // :6214-6217
        }
        self.fly_time = 0; // :6218
        self.kind = m;
        self.fatigue_max = row.fatigue_max as f32; // :6221
                                                   // :6227 `_flipDraw = false`
        self.flip_draw = false;
        // :6236-6243 — the Scutlix/Santank ability-cooldown floors.
        if self.kind == 9 && self.ability_cooldown < 20 {
            self.ability_cooldown = 20;
        }
        if self.kind == 46 && self.ability_cooldown < 40 {
            self.ability_cooldown = 40;
        }
        Some(Equip {
            clear_buff: old_buff,
            add_buff: row.buff,
            height_boost: row.height_boost,
            was_active,
            track_rider,
            size_override: row.player_size_override,
        })
    }

    /// `Mount.Dismount` (`Mount.cs:6142-6194`) — the state transitions; returns
    /// the buff to clear. The dust effect (:6167-6170) and the hitbox restore
    /// (:6176-6193, via `Collision.TryChangingSizeFromBottomCenter`) are the
    /// caller's (Phase 2 wires both; the noItems restore :6152-6155 only
    /// concerns transformation mounts 55/56).
    pub(crate) fn dismount(&mut self) -> Option<i32> {
        if !self.active {
            return None; // :6144-6147
        }
        let buff = self.data().map(|d| d.buff);
        self.active = false; // :6149
        self.reset(); // :6171
        buff
    }

    /// `Mount.CheckMountBuff` (`Mount.cs:5275-5281`): the buff expiring is a
    /// dismount request — `true` when the caller must run the dismount path
    /// (the `TryDismount` gates live on the Player side, `Mount.cs:6606-6624`).
    pub(crate) fn check_mount_buff(&self, has_mount_buff: bool) -> bool {
        self.kind != -1 && !has_mount_buff
    }

    // -- Accessors (the per-type overrides live here, as in C#) --------------

    /// `Mount.RunSpeed` (`Mount.cs:449-484`). The swim/special-state overrides
    /// reference the full type set; only the rows this table carries can fire.
    pub(crate) fn run_speed(&self) -> f32 {
        let Some(row) = self.data() else { return 0.0 };
        if (self.kind == 4 || self.kind == 12 || self.kind == 44 || self.kind == 49)
            && self.frame_state == 4
        {
            return row.swim_speed;
        }
        if self.kind == 12 && self.frame_state == 2 {
            return row.run_speed + 13.5;
        }
        if self.kind == 44 && self.frame_state == 2 {
            return row.run_speed + 4.0;
        }
        if self.kind == 5 && self.frame_state == 2 {
            let num = self.fatigue / self.fatigue_max;
            return row.run_speed + 4.0 * (1.0 - num);
        }
        if self.kind == 50 && self.frame_state == 2 {
            return row.run_speed + 2.0;
        }
        if self.should_super_cart {
            return SUPER_CART_RUN_SPEED;
        }
        row.run_speed
    }

    /// `Mount.DashSpeed` (`Mount.cs:486-496`).
    pub(crate) fn dash_speed(&self) -> f32 {
        let Some(row) = self.data() else { return 0.0 };
        if self.should_super_cart {
            return SUPER_CART_DASH_SPEED;
        }
        row.dash_speed
    }

    /// `Mount.Acceleration` (`Mount.cs:498-508`).
    pub(crate) fn acceleration(&self) -> f32 {
        let Some(row) = self.data() else { return 0.0 };
        if self.should_super_cart {
            return SUPER_CART_ACCELERATION;
        }
        row.acceleration
    }

    /// `Mount.JumpHeight` (`Mount.cs:2810-2834`) — the |xVelocity| scaling for
    /// types 0/1 (the C# `(int)` cast truncates toward zero) and the swim +5.
    pub(crate) fn jump_height(&self, x_velocity: f32) -> i32 {
        let Some(row) = self.data() else { return 0 };
        let mut num = row.jump_height;
        match self.kind {
            0 => num += (x_velocity.abs() / 4.0) as i32,
            1 => num += (x_velocity.abs() / 2.5) as i32,
            4 | 49 => {
                if self.frame_state == 4 {
                    num += 5;
                }
            }
            _ => {}
        }
        if self.should_super_cart {
            num = SUPER_CART_JUMP_HEIGHT;
        }
        num
    }

    /// `Mount.JumpSpeed` (`Mount.cs:2836-2858`).
    pub(crate) fn jump_speed(&self, x_velocity: f32) -> f32 {
        let Some(row) = self.data() else { return 0.0 };
        let mut num = row.jump_speed;
        match self.kind {
            0 | 1 => num += x_velocity.abs() / 7.0,
            4 | 49 => {
                if self.frame_state == 4 {
                    num += 2.5;
                }
            }
            _ => {}
        }
        if self.should_super_cart {
            num = SUPER_CART_JUMP_SPEED;
        }
        num
    }

    /// `Mount.CanFly` (`Mount.cs:2860-2879`) — the type-54 selective-fly
    /// branch reads the `SelectiveFlyingMountData.allowedToFly` payload
    /// (`UpdateAfterEquips` arms it from `wingsLogic > 0`, `:2989-2992`)
    /// BEFORE the `flightTimeMax == 0` refusal.
    /// Gate merge (i40 ⨉ i41): the i41 latched-payload shape wins over the
    /// i40 derive-live `can_fly(wings_logic)` copy — same semantics, but the
    /// latch is the C# structure (`_mountSpecificData`).
    pub(crate) fn can_fly(&self) -> bool {
        if !self.active {
            return false;
        }
        // `:2866-2869` — the raptor flies while wings are equipped even
        // though its own `flightTimeMax` is 0.
        if self.kind == 54 && self.selective_allowed_to_fly {
            return true;
        }
        let Some(row) = self.data() else { return false };
        if row.flight_time_max == 0 {
            return false;
        }
        if self.kind == 48 {
            return false;
        }
        true
    }

    /// `Mount.UpdateAfterEquips`' type-54 arm (`Mount.cs:2989-2997`): the
    /// selective-fly permission mirrors `wingsLogic > 0`; with the Empress
    /// Brooch (`Player.cs:14771-14774` grants `empressBrooch`), the flight
    /// budget refills to `wingTimeMax` every tick while wings are equipped
    /// (`:2993-2996`).
    pub(crate) fn update_selective_flight(
        &mut self,
        wings_logic: i32,
        empress_brooch: bool,
        wing_time_max: i32,
    ) {
        if !self.active || self.kind != 54 {
            return;
        }
        self.selective_allowed_to_fly = wings_logic > 0;
        if self.selective_allowed_to_fly && empress_brooch {
            self.fly_time = wing_time_max;
        }
    }

    /// `Mount.ResetFlightTime` (`Mount.cs:5262-5273`) — the type-0 |v|*20
    /// bonus and the type-54 wings borrow (`_flyTime = wingTimeMax`).
    pub(crate) fn reset_flight_time(&mut self, x_velocity: f32, wing_time_max: i32) {
        self.fly_time = if self.active {
            mount_data(self.kind).map_or(0, |d| d.flight_time_max)
        } else {
            0
        };
        if self.kind == 0 {
            self.fly_time += (x_velocity.abs() * 20.0) as i32;
        }
        // `:5269-5272` — the Velociraptor borrows the wing budget outright.
        if self.kind == 54 {
            self.fly_time = wing_time_max;
        }
    }

    pub(crate) fn flight_time_max(&self) -> i32 {
        self.data().map_or(0, |d| d.flight_time_max)
    }

    // Gate merge (i40 ⨉ i41): the i40 `refill_wing_flight` duplicate is
    // folded into `update_selective_flight` above — the i41 copy is the
    // canonical `UpdateAfterEquips` type-54 arm (permission latch + brooch
    // refill in one call, `Mount.cs:2989-2997`); one grant path.

    /// `Mount.Flight` (`Mount.cs:2973-2981`): one tick of budget per call.
    pub(crate) fn flight(&mut self) -> bool {
        if self.fly_time <= 0 {
            return false;
        }
        self.fly_time -= 1;
        true
    }

    /// `Mount.CanHover` (`Mount.cs:2881-2896`): `usesHover`; the lava shark
    /// (49) only in its swim state. No table row sets `usesHover` yet
    /// (M2-MOUNT-2), so this is false for every current id.
    pub(crate) fn can_hover(&self) -> bool {
        let Some(row) = self.data() else { return false };
        if !row.uses_hover {
            return false;
        }
        if self.kind == 49 {
            return self.frame_state == 4;
        }
        true
    }


    /// `Mount.DoesHoverIgnoresFatigue` (`Mount.cs:3553-3560`): the fatigue-free
    /// hover set — the UFO, Drill, CuteFishron, WitchBroom, PirateShip,
    /// LavaShark, Bat and Pixie never build fatigue.
    pub(crate) fn does_hover_ignore_fatigue(&self) -> bool {
        matches!(self.kind, 7 | 8 | 12 | 23 | 44 | 49 | 56 | 61)
    }

    /// `Mount.FatigueRecovery` (`Mount.cs:2961-2971`): -2 per resting tick,
    /// called by the player whenever `velocity.Y == 0`.
    pub(crate) fn fatigue_recovery(&mut self) {
        if self.fatigue > 2.0 {
            self.fatigue -= 2.0;
        } else {
            self.fatigue = 0.0;
        }
    }

    /// `Mount.Hover` (`Mount.cs:3340-3543`): the hover branch of the mounted
    /// vertical movement. Returns the integrated `velocity.Y`; the caller
    /// treats the branch as consumed (the gravity arm is inside, like the
    /// original's `else if` chain). `control_up` has no port key yet (the
    /// wings' hold-up), so the ascent is `control_jump` only.
    pub(crate) fn hover_integrate(
        &mut self,
        vy: f32,
        control_up: bool,
        control_jump: bool,
        control_down: bool,
        gravity: f32,
    ) -> f32 {
        let Some(row) = self.data() else { return vy };
        // `:3553` — the flag: the free-hover set never drains fatigue.
        let flag = self.does_hover_ignore_fatigue();
        // `:3377-3381` — the active-hover frame states; the lava shark (49)
        // only in its swim state, the Bat (56) and Pixie (61) also dashing.
        let flag2 = match self.kind {
            49 => self.frame_state == 4,
            56 | 61 => matches!(self.frame_state, 2 | 3),
            _ => matches!(self.frame_state, 2 | 4),
        };
        let mut num3 = if self.fatigue_max > 0.0 {
            (self.fatigue / self.fatigue_max).clamp(0.0, 1.0)
        } else {
            0.0
        };
        if flag {
            num3 = 0.0;
        }
        let num = if self.kind == 12 { 0.5 } else { 1.0 };
        if flag2 {
            // `:3369-3383` — the fuel drain lives INSIDE the hover arm: the
            // flight seconds first, then the fatigue pool builds (1.0 per
            // tick, gravity/defaultGravity); empties the hover.
            let mut fuel = true;
            if !flag {
                if self.fly_time > 0 {
                    self.fly_time -= 1;
                } else if self.fatigue < self.fatigue_max {
                    self.fatigue += 1.0;
                } else {
                    fuel = false;
                }
            }
            // `:3384` — the acceleration applies unless the DarkMageBook.
            let flag4 = self.kind != 48;
            let mut num4 = 4.0 * num3;
            let mut num5 = 4.0 * num3;
            if self.kind == 48 {
                num4 = 0.0;
                num5 = 0.0;
            }
            if num4 == 0.0 {
                num4 = -0.001;
            }
            if num5 == 0.0 {
                num5 = -0.001;
            }
            let mut vy = vy;
            if flag4 && (control_up || control_jump) && fuel {
                // `:3392-3401` — the ascend target: fatigue slows the rise;
                // the Bat/Pixie cap at their dash speed.
                num4 = -2.0 - 6.0 * (1.0 - num3);
                if self.kind == 48 {
                    num4 /= 3.0;
                }
                if matches!(self.kind, 56 | 61) {
                    num4 = -row.dash_speed;
                }
                vy -= row.acceleration * num;
            } else if control_down {
                num5 = 8.0;
                if matches!(self.kind, 56 | 61) {
                    num5 = row.dash_speed;
                }
                vy += row.acceleration * num;
            } else if num5 != -0.001 || num4 != -0.001 {
                // `:3418-3421` — the idle drift: gravity with a 4*num3 fall
                // cap (only when a real cap exists).
                vy += gravity;
                num5 = 4.0 * num3;
            }
            // `:3454-3472` — the vertical clamp toward num4.
            if vy < num4 {
                vy += row.acceleration * num;
                if vy > num4 {
                    vy = num4;
                }
            }
            return vy;
        }
        // `:3472-3486` — not hovering.
        vy + gravity
    }

    /// `Mount.AbilityRecovery` (`Mount.cs:2938-2959`): the charge fills while
    /// charging, decays one per tick otherwise; the cooldown and the duration
    /// count down. (The ability consumers themselves are M2-MOUNT-4.)
    pub(crate) fn ability_recovery(&mut self) {
        if self.ability_charging {
            if let Some(row) = self.data() {
                if self.ability_charge < row.ability_charge_max {
                    self.ability_charge += 1;
                }
            }
        } else if self.ability_charge > 0 {
            self.ability_charge -= 1;
        }
        if self.ability_cooldown > 0 {
            self.ability_cooldown -= 1;
        }
        if self.ability_duration > 0 {
            self.ability_duration -= 1;
        }
    }

    /// `Mount.StartAbilityCharge`'s REMOTE branch (`Mount.cs:2916-2919`):
    /// the non-local owner only sets the flag — the local-owner 441 spawn
    /// (`:2905-2914`) is the auto-aim's `UpdateEffects` path (M2-MOUNT-4).
    /// The 441 crosshair's remote gate (`Projectile.cs:30660-30663`) calls
    /// it when not already charging.
    pub(crate) fn start_ability_charge_remote(&mut self) {
        if self.kind == 9 {
            self.ability_charging = true;
        }
    }

    /// `Mount.AbilityActive` (`Mount.cs:626`) — the Drill's mining gate
    /// (`UseDrill` `:3061`, the 453 crosshair's remote check
    /// `Projectile.cs:30665`).
    pub(crate) fn ability_active(&self) -> bool {
        self.ability_active
    }

    /// `Mount.UseAbility`'s Drill (8) `_abilityActive` write
    /// (`Mount.cs:3317-3335`): `toggleOn: false` clears it (the local
    /// release / the remote 453 gate), `toggleOn: true` sets it (the
    /// `ItemCheck_HandleMount` press, `Player.cs:52944` — the 453 spawn
    /// itself is the mining layer's, M2-MOUNT-4).
    pub(crate) fn set_ability_active(&mut self, value: bool) {
        if self.kind == 8 {
            self.ability_active = value;
        }
    }

    /// `Mount.StopAbilityCharge` (`Mount.cs:2922-2931`): the Scutlix (9) /
    /// Santank (46) drop the charge flag and re-arm the row's cooldown +
    /// duration. W4-d8 DOC-SWEEP (3): the 441 crosshair's `Kill()` arm
    /// (`Projectile.cs:73394-73397`) calls it on EVERY 441 death.
    pub(crate) fn stop_ability_charge(&mut self) {
        if matches!(self.kind, 9 | 46) {
            self.ability_charging = false;
            if let Some(row) = self.data() {
                self.ability_cooldown = row.ability_cooldown;
                self.ability_duration = row.ability_duration;
            }
        }
    }

    /// `Mount.AbilityCharging` (`Mount.cs:624`) / `Mount.AbilityCharge`
    /// (`:620`) — the crosshair's remote-owner gate (`Projectile.cs:30660`)
    /// and the aim's eye-glow read (`Mount.cs:5394`).
    pub(crate) fn ability_charging(&self) -> bool {
        self.ability_charging
    }

    pub(crate) fn ability_charge(&self) -> i32 {
        self.ability_charge
    }

    /// `Mount._frameExtra` — the Scutlix head's aim frame (6/7/8/9/10, the
    /// `AimAbility` buckets `Mount.cs:5367-5393`); the eye-position index
    /// `(frameExtra - 6) * 2` reads it (`:3269`/`:5400`).
    pub(crate) fn frame_extra(&self) -> i32 {
        self.frame_extra
    }

    /// The Scutlix eye offsets — `Mount.scutlixEyePositions`
    /// (`Mount.cs:1398-1421`): the ten raw pairs MINUS the 45×54 texture
    /// size (`:1418-1420`), so index 0 = (60−45, 2−54) = (15, −52). The
    /// aim/fire reads `[(frameExtra − 6) * 2 + i]` for the two eyes.
    pub(crate) fn scutlix_eye_positions() -> [(f32, f32); 10] {
        // `:1399-1408` — the raw pairs, verbatim.
        const RAW: [(f32, f32); 10] = [
            (60.0, 2.0),
            (70.0, 6.0),
            (68.0, 6.0),
            (76.0, 12.0),
            (80.0, 10.0),
            (84.0, 18.0),
            (74.0, 20.0),
            (76.0, 24.0),
            (70.0, 34.0),
            (76.0, 34.0),
        ];
        // `:1409` + `:1418-1420` — the (45, 54) subtract.
        let mut out = [(0.0f32, 0.0f32); 10];
        for i in 0..10 {
            out[i] = (RAW[i].0 - 45.0, RAW[i].1 - 54.0);
        }
        out
    }

    /// `Mount.ClampToDeadZone` (`Mount.cs:5296-5333`) — the Scutlix (9) /
    /// Drill (8) half: a cursor INSIDE the mount's texture box (±45×±54
    /// for the Scutlix, `scutlixTextureSize` `:1409`; ±80×±80 for the
    /// Drill, `drillTextureSize` `:1333`) projects onto the box edge
    /// along the smaller scale; outside, the point stands. The Santank
    /// (46) row is its own ability's (M2-MOUNT-4). `center` is the
    /// player's Centre; the return is the clamped world point.
    pub(crate) fn clamp_to_dead_zone(&self, center: Vec2, position: Vec2) -> Vec2 {
        let (half_w, half_h) = match self.kind {
            9 => (45.0f32, 54.0f32), // `:5303-5304` (Y, X) = (54, 45)
            8 => (80.0, 80.0),       // `:5311-5312`
            _ => return position,    // `:5314-5315`
        };
        let mut rel = position - center;
        if rel.x > -half_w && rel.x < half_w && rel.y > -half_h && rel.y < half_h {
            // `:5321-5330` — the two edge scales; the SMALLER wins (the
            // C# `num3 > num4 → *= num4` else `*= num3`). The abs-zero
            // divide is the C#'s own infinity (a centred cursor projects
            // to the box edge on the other axis).
            let sx = half_w / rel.x.abs();
            let sy = half_h / rel.y.abs();
            rel *= if sx > sy { sy } else { sx };
        }
        rel + center
    }

    /// `Mount.AimAbility`'s Scutlix (9) half (`Mount.cs:5340-5421`) — the
    /// head's aim: the dead-zone-clamped cursor angle picks the facing
    /// (±90° fold), the flip-draw flag (facing against the velocity) and
    /// the `_frameExtra` bucket (8 level / 9,10 up / 7,6 down, the 22.5° /
    /// 67.5° / 112.5° gates). Returns the C#'s `bool` — true when the
    /// frame or the facing changed (`:5416-5420`). The eye-glow light
    /// (`:5394-5414`, `Lighting.AddLight` per eye while the charge > 0)
    /// is the light layer's — skipped. W4-d8 DOC-SWEEP (4): the 441/453
    /// crosshair's local + remote aim calls (`Projectile.cs:30648`/
    /// `:30669`) reach this; the Drill (8) half writes the unported
    /// `DrillMountData` diode rotation (`:5466-5494`, M2-MOUNT-4) and
    /// folds out with it.
    pub(crate) fn aim_ability_scutlix(
        &mut self,
        center: Vec2,
        facing: &mut i32,
        vx: f32,
        mouse: Vec2,
    ) -> bool {
        // `:5337` — `_aiming = true` for EVERY type (the switch's head).
        self.aiming = true;
        if self.kind != 9 {
            return false; // `:5496-5497` — the default arm
        }
        let old_frame = self.frame_extra;
        let old_facing = *facing;
        // `:5344` — the clamped cursor's angle in DEGREES.
        let clamped = self.clamp_to_dead_zone(center, mouse);
        let mut deg = (clamped - center).y.atan2((clamped - center).x).to_degrees();
        // `:5345-5358` — the ±90° facing fold.
        if deg > 90.0 {
            *facing = -1;
            deg = 180.0 - deg;
        } else if deg < -90.0 {
            *facing = -1;
            deg = -180.0 - deg;
        } else {
            *facing = 1;
        }
        // `:5359-5366` — the flip-draw: facing against the velocity sign.
        self.flip_draw = (*facing > 0 && vx < 0.0) || (*facing < 0 && vx > 0.0);
        // `:5367-5393` — the frame buckets. The C# compares the f32
        // degree value against the widened doubles (22.5 / 67.5 / 112.5).
        if deg >= 0.0 {
            if (deg as f64) < 22.5 {
                self.frame_extra = 8;
            } else if (deg as f64) < 67.5 {
                self.frame_extra = 9;
            } else if (deg as f64) < 112.5 {
                self.frame_extra = 10;
            }
        } else if (deg as f64) > -22.5 {
            self.frame_extra = 8;
        } else if (deg as f64) > -67.5 {
            self.frame_extra = 7;
        } else if (deg as f64) > -112.5 {
            self.frame_extra = 6;
        }
        // `:5416-5420` — the change verdict.
        if self.frame_extra == old_frame {
            return *facing != old_facing;
        }
        true
    }

    /// `Mount.UseAbility`'s Scutlix (9) half (`Mount.cs:3258-3289`) — the
    /// laser fire: TWO 606 beams from the head's eyes toward the
    /// dead-zone-clamped cursor, each at velocity 14 (the normalized
    /// eye→cursor delta × 14), damage 150, knockback 0, spawned 14 px
    /// along the aim from the eye. Returns the two `(x, y, vx, vy)` spawn
    /// tuples (the eye anchor + the beam velocity); the caller spawns
    /// them through the pool (`NewProjectile` centres the box, the port's
    /// seam takes the top-left — the half-box subtract is the caller's).
    /// The eye anchor: `(position.X + width/2, position.Y + height)` +
    /// the eye offset (X mirrored by the facing, ± xOffset; Y + yOffset).
    /// W4-d8 DOC-SWEEP (3): the 441 crosshair's channel-release
    /// `UseAbility` (`Projectile.cs:30651`) reaches this. The Santank
    /// (46) / Drill (8) halves are their own abilities (M2-MOUNT-4).
    pub(crate) fn scutlix_laser_fire(
        &self,
        pos: Vec2,
        width: f32,
        height: f32,
        facing: i32,
        mouse: Vec2,
    ) -> Option<[(f32, f32, f32, f32); 2]> {
        if self.kind != 9 {
            return None; // `:3256-3258` — the switch's case 9 only
        }
        let row = self.data()?;
        let center = Vec2::new(pos.x + width * 0.5, pos.y + height * 0.5);
        let mouse = self.clamp_to_dead_zone(center, mouse);
        // `:3266-3268` — the head anchor (the player's bottom-centre).
        let anchor = Vec2::new(pos.x + (width as i32 / 2) as f32, pos.y + height);
        let eyes = Self::scutlix_eye_positions();
        let base = (self.frame_extra - 6) * 2; // `:3269`
        let mut out = [(0.0f32, 0.0f32, 0.0f32, 0.0f32); 2];
        for i in 0..2 {
            // `:3273-3281` — the eye point: Y + yOffset, X ± xOffset by
            // the facing.
            let eye = Vec2::new(
                if facing == -1 {
                    anchor.x - eyes[base as usize + i].0 - row.x_offset as f32
                } else {
                    anchor.x + eyes[base as usize + i].0 + row.x_offset as f32
                },
                anchor.y + eyes[base as usize + i].1 + row.y_offset as f32,
            );
            // `:3282-3286` — the normalized aim × 14, the spawn 14 px out.
            let mut aim = mouse - eye;
            aim = aim.normalize_or_zero();
            aim *= 14.0;
            let spawn = eye + aim;
            out[i] = (spawn.x, spawn.y, aim.x, aim.y);
        }
        Some(out)
    }

    /// `Mount.ExtraFall` / `Mount.FallDamage` (:183/:181 read via properties).
    pub(crate) fn extra_fall(&self) -> i32 {
        self.data().map_or(0, |d| d.extra_fall)
    }

    pub(crate) fn fall_damage(&self) -> f32 {
        self.data().map_or(0.0, |d| d.fall_damage)
    }

    /// `Mount.AutoJump` (:514) / `Mount.BlockExtraJumps` (:516).
    pub(crate) fn auto_jump(&self) -> bool {
        self.data().map_or(false, |d| d.constant_jump)
    }

    pub(crate) fn block_extra_jumps(&self) -> bool {
        self.data().map_or(false, |d| d.block_extra_jumps)
    }

    /// `Mount.CanUseWings` (`Mount.cs:570-580`) — an inactive mount leaves the
    /// player's wings untouched (`true`).
    pub(crate) fn can_use_wings(&self) -> bool {
        #[cfg(test)]
        if let Some(v) = self.test_can_use_wings {
            return v;
        }
        self.data().map_or(true, |d| d.can_use_wings)
    }

    /// `Mount.MovementStatsAreAdditive` (`Mount.cs:582-592`).
    pub(crate) fn movement_stats_are_additive(&self) -> bool {
        #[cfg(test)]
        if let Some(v) = self.test_additive {
            return v;
        }
        self.data().map_or(false, |d| d.movement_stats_are_additive)
    }

    /// `Mount.DismountOnItemUse` (:642-652).
    pub(crate) fn dismount_on_item_use(&self) -> bool {
        #[cfg(test)]
        if let Some(v) = self.test_dismounts_on_item_use {
            return self.active && v;
        }
        self.active && self.data().map_or(false, |d| d.dismounts_on_item_use)
    }

    /// `Mount.IsConsideredASlimeMount` (:518-528) — types 3 and 50.
    pub(crate) fn is_considered_a_slime_mount(&self) -> bool {
        self.kind == 3 || self.kind == 50
    }

    /// `Mount.PlayerOffset` (:407-421) — the per-frame player draw offset.
    pub(crate) fn player_offset(&self) -> i32 {
        if !self.active {
            return 0;
        }
        let Some(row) = self.data() else { return 0 };
        if self.frame >= row.total_frames {
            return 0;
        }
        row.player_y_offsets[self.frame as usize]
    }

    /// `Mount.PlayerOffsetHitbox` (:423-433).
    pub(crate) fn player_offset_hitbox(&self) -> i32 {
        if !self.active {
            return 0;
        }
        -self.player_offset() + self.data().map_or(0, |d| d.height_boost)
    }

    /// `Mount.IsAtIdle` (:594-608): the idle band active — Rudolph's draw
    /// uses it to hide the front strip (`Mount.cs:5520-5523`).
    pub(crate) fn is_at_idle(&self) -> bool {
        let Some(row) = self.data() else { return false };
        row.idle.count != 0 && self.idle_time >= self.idle_time_next
    }

    /// Test-only: the frame state the hover gates read.
    #[cfg(test)]
    pub(crate) fn test_set_frame_state(&mut self, state: i32) {
        self.frame_state = state;
    }

    /// Test-only: the flight budget value (`reset_flight_time` writes it too).
    #[cfg(test)]
    pub(crate) fn test_set_fly_time(&mut self, t: i32) {
        self.fly_time = t;
    }

    /// Test-only: the fatigue pool value.
    #[cfg(test)]
    pub(crate) fn test_set_fatigue(&mut self, v: f32) {
        self.fatigue = v;
    }

    // The M2-MOUNT-2 rows (wing-allowed, additive-stat mounts) do not exist
    // yet; these test-only overrides let the merge branches be exercised now.
    #[cfg(test)]
    pub(crate) fn set_test_overrides(
        &mut self,
        can_use_wings: Option<bool>,
        additive: Option<bool>,
    ) {
        self.test_can_use_wings = can_use_wings;
        self.test_additive = additive;
    }

    #[cfg(test)]
    pub(crate) fn set_test_dismount_on_item_use(&mut self, v: Option<bool>) {
        self.test_dismounts_on_item_use = v;
    }
}

#[cfg(test)]
impl Default for TestOverrides {
    fn default() -> Self {
        TestOverrides {
            can_use_wings: None,
            additive: None,
        }
    }
}

/// Test-only row-value overrides (see `set_test_overrides`).
#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TestOverrides {
    can_use_wings: Option<bool>,
    additive: Option<bool>,
}

#[cfg(test)]
mod tests {
    // Both gate witnesses survive the M1 keep-one (i05's `allow_direction_change`
    // is the single fn): the i04 boundary probe (direct cooldown writes) and
    // the i12 recovery-decrement mirror.

    /// S1-P-i04 — `Mount.AllowDirectionChange` (`Mount.cs:630-640`): only
    /// the Scutlix (9) gates on the ability cooldown's half-max window
    /// (`abilityCooldown = 20`, `Mount.cs:1343-1421` — the row's
    /// `ability_cooldown`); every other mount allows the turn.
    #[test]
    fn s1_p_i04_only_the_scutlix_gates_the_direction_change() {
        let mut m = MountState::default();
        assert!(m.allow_direction_change(), "no mount allows the turn");
        m.set_mount(10, false); // the Unicorn
        assert!(m.allow_direction_change(), "the Unicorn allows it");
        m.set_mount(9, false); // the Scutlix
        // `SetMount` floors the Scutlix cooldown at 20 (`Mount.cs:6236-
        // 6243`, the port's `set_mount`): 20 >= 20/2 → LOCKED on equip.
        assert!(!m.allow_direction_change(), "the equip floor locks it");
        m.ability_cooldown = 11;
        assert!(!m.allow_direction_change(), "11 >= 10 stays locked");
        m.ability_cooldown = 10;
        assert!(!m.allow_direction_change(), "10 >= 10 locks");
        m.ability_cooldown = 9;
        assert!(m.allow_direction_change(), "9 < 10 opens the window");
        m.ability_cooldown = 0;
        assert!(m.allow_direction_change());
    }

    /// S1-P-i12 — `Mount.AllowDirectionChange` (`Mount.cs:630-640`): the
    /// Scutlix (type 9) reads `_abilityCooldown < _data.abilityCooldown / 2`
    /// (20/2 = 10, integer division); every other type returns true. The
    /// witness drives the cooldown through the real `SetMount` floor
    /// (`:6236-6243`) and the `AbilityRecovery` per-tick decrement
    /// (`:2938-2959`). Mirror clone, written not run.
    #[test]
    fn i12_allow_direction_change_is_the_scutlix_half_cooldown_gate() {
        let mut m = super::MountState::default();
        // No mount / a non-Scutlix type: always true (`:638`).
        assert!(m.allow_direction_change());
        assert!(m.set_mount(10, false).is_some());
        assert!(m.allow_direction_change(), "the Unicorn always allows");
        // The Scutlix floors its cooldown at 20 on equip (`:6236-6243`):
        // 20 < 10 is false — the fresh mount locks the turn.
        assert!(m.set_mount(9, false).is_some());
        assert!(!m.allow_direction_change(), "the fresh Scutlix locks");
        // Tick the cooldown down past the half-max boundary: 10 still
        // locks (`<` is strict), 9 allows.
        for _ in 0..10 {
            m.ability_recovery();
        }
        assert!(!m.allow_direction_change(), "10 is not < 10");
        m.ability_recovery();
        assert!(m.allow_direction_change(), "9 < 10 allows the turn");
    }

    /// W4-d8 DOC-SWEEP (3)/(4) — the Scutlix ability's self-contained half
    /// (WRITTEN NOT RUN, code-only): the eye table, the dead-zone clamp,
    /// the aim buckets, the two-beam fire and the charge-stop. Every
    /// expected value is hand-derived from the cited C# lines.
    #[test]
    fn w4_d8_the_scutlix_ability_math() {
        use super::*;
        let mut m = MountState::default();
        assert!(m.set_mount(9, false).is_some());
        // The eye table (`Mount.cs:1399-1420`): raw (60, 2) − (45, 54).
        let eyes = MountState::scutlix_eye_positions();
        assert_eq!(eyes[0], (15.0, -52.0), "index 0 = (60−45, 2−54)");
        assert_eq!(eyes[9], (31.0, -20.0), "index 9 = (76−45, 34−54)");
        // The dead-zone clamp (`:5319-5330`): a cursor inside the ±45×±54
        // box projects onto the edge. Center (1000, 500), cursor
        // (1010, 510): rel (10, 10), sx = 45/10 = 4.5, sy = 54/10 = 5.4;
        // sx < sy → *= sx → rel (45, 45) → (1045, 545).
        let c = Vec2::new(1000.0, 500.0);
        let clamped = m.clamp_to_dead_zone(c, Vec2::new(1010.0, 510.0));
        assert_eq!(clamped, Vec2::new(1045.0, 545.0), "the smaller scale wins");
        // Outside the box: the point stands.
        let far = Vec2::new(1200.0, 500.0);
        assert_eq!(m.clamp_to_dead_zone(c, far), far);
        // The aim buckets (`:5367-5393`): a level-right cursor → frame 8,
        // facing +1. The player centre (1000, 500), the cursor (1200, 500)
        // is outside the dead zone → angle 0°.
        let mut facing = -1;
        let changed = m.aim_ability_scutlix(c, &mut facing, 0.0, Vec2::new(1200.0, 500.0));
        assert_eq!(facing, 1, "the right cursor faces right (`:5357`)");
        assert_eq!(m.frame_extra(), 8, "0° < 22.5 → frame 8 (`:5371`)");
        assert!(changed, "the facing flipped");
        assert!(!m.flip_draw, "vx 0 → no flip (`:5359-5366`)");
        // A straight-up cursor (1000, 300): angle −90° → the < −90 branch
        // is NOT taken (−90 is not < −90) → facing +1, deg −90 → the
        // > −112.5 bucket → frame 6.
        let mut facing2 = 1;
        let _ = m.aim_ability_scutlix(c, &mut facing2, 0.0, Vec2::new(1000.0, 300.0));
        assert_eq!(m.frame_extra(), 6, "−90° > −112.5 → frame 6 (`:5392`)");
        // A left cursor (800, 500): angle 180° > 90 → facing −1,
        // deg = 180 − 180 = 0 → frame 8.
        let mut facing3 = 1;
        let _ = m.aim_ability_scutlix(c, &mut facing3, 0.0, Vec2::new(800.0, 500.0));
        assert_eq!(facing3, -1, "the left fold (`:5347`)");
        assert_eq!(m.frame_extra(), 8);
        // The fire (`:3266-3287`): frame 8 → eye base (8−6)×2 = 4, the
        // eyes[4] = (80−45, 10−54) = (35, −44), eyes[5] = (39, −36).
        // Player pos (980, 458), width 20, height 42 → anchor
        // (980 + 20/2, 458 + 42) = (990, 500). Facing +1, xOffset 1,
        // yOffset 6: eye0 = (990 + 35 + 1, 500 − 44 + 6) = (1026, 462).
        // The cursor (1200, 500) is outside the dead zone (rel 200, 0).
        // aim = normalize((1200, 500) − (1026, 462)) × 14.
        let mut fire_facing = 1;
        m.aim_ability_scutlix(c, &mut fire_facing, 0.0, Vec2::new(1200.0, 500.0));
        let beams = m
            .scutlix_laser_fire(
                Vec2::new(980.0, 458.0),
                20.0,
                42.0,
                1,
                Vec2::new(1200.0, 500.0),
            )
            .expect("the Scutlix fires");
        let d = Vec2::new(1200.0 - 1026.0, 500.0 - 462.0).normalize_or_zero() * 14.0;
        assert!((beams[0].0 - (1026.0 + d.x)).abs() < 1e-4, "eye0 + the aim");
        assert!((beams[0].1 - (462.0 + d.y)).abs() < 1e-4);
        assert!((beams[0].2 - d.x).abs() < 1e-4, "the ×14 velocity");
        assert!((beams[0].3 - d.y).abs() < 1e-4);
        // The second eye: eyes[5] = (39, −36) → (990 + 39 + 1, 500 − 36 + 6)
        // = (1030, 470).
        let d2 = Vec2::new(1200.0 - 1030.0, 500.0 - 470.0).normalize_or_zero() * 14.0;
        assert!((beams[1].0 - (1030.0 + d2.x)).abs() < 1e-4, "eye1 + the aim");
        // A non-Scutlix mount never fires.
        let mut u = MountState::default();
        assert!(u.set_mount(10, false).is_some());
        assert!(u.scutlix_laser_fire(Vec2::ZERO, 20.0, 42.0, 1, Vec2::X).is_none());
        // StopAbilityCharge (`:2922-2931`): the flag drops, the row's
        // cooldown 20 / duration 0 re-arm.
        m.start_ability_charge_remote();
        assert!(m.ability_charging());
        m.ability_cooldown = 3;
        m.stop_ability_charge();
        assert!(!m.ability_charging());
        assert_eq!(m.ability_cooldown, 20, "the row's cooldown (`:1351`)");
        assert_eq!(m.ability_duration, 0, "the row's duration (`:1352`)");
    }

    #[test]
    fn the_cart_rows_carry_the_setas_minecart_body() {
        // `Mount.SetAsMinecart` (`Mount.cs:2747-2799`): the shared body plus
        // the base cart's own fields.
        let m = mount_data(6).expect("the base minecart row");
        assert!(m.minecart);
        assert_eq!(m.buff, 118);
        assert_eq!(m.height_boost, 10);
        assert_eq!(m.flight_time_max, 0);
        assert_eq!(m.fall_damage, 1.0);
        assert_eq!(m.run_speed, 13.0);
        assert_eq!(m.acceleration, 0.04);
        assert_eq!(m.jump_height, 15);
        assert_eq!(m.jump_speed, 5.15);
        assert!(m.block_extra_jumps);
        assert_eq!(m.total_frames, 3);
        assert_eq!(m.player_y_offsets.len(), m.total_frames as usize);
        assert_eq!(m.standing.count, 1);
        assert_eq!(m.running.count, 3);
        assert_eq!(m.flying.count, 0);
        assert_eq!(m.front_sheet, "Mount_Minecart");
        // The cosmetic variants (`Mount.cs:966-1110`): all 27 cart rows.
        for id in [6, 11, 13, 15, 16, 18, 19, 20, 21, 22, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 38, 39, 51, 53] {
            let row = mount_data(id).unwrap_or_else(|| panic!("cart row {id}"));
            assert!(row.minecart, "row {id} is a cart");
            assert_eq!(row.flight_time_max, 0, "row {id} cannot fly");
            assert!(!row.can_use_wings, "row {id} suppresses wings");
        }
        // The mech/wood/mole overrides.
        let mech = mount_data(11).unwrap();
        assert_eq!(mech.height_boost, 12);
        assert_eq!(mech.x_offset, -1);
        assert_eq!((mech.y_offset, mech.player_head_offset), (11, 14));
        assert_eq!(mech.spawn_dust, 213, "S1-L-8b: `Mount.cs:1488` (was 0)");
        let wood = mount_data(13).unwrap();
        assert_eq!((wood.run_speed, wood.acceleration), (10.0, 0.03));
        let mole = mount_data(39).unwrap();
        assert_eq!((mole.run_speed, mole.acceleration), (6.0, 0.02));
        assert_eq!(mole.y_offset, 13 - 2);
        // The fart carts' vertical offsets (`Mount.cs:1097`, `:1105`, both
        // `SetAsMinecart(..., -10, -8)`): `yOffset = 13 + (-10)` (`:2772`) and
        // `playerYOffsets = 8 - (-10) + (-8) = 10` (`:2767` — S1-L-8b fixes
        // the port's old 0).
        let fart = mount_data(51).unwrap();
        assert_eq!(fart.y_offset, 13 - 10);
        assert_eq!(fart.player_y_offsets[0], 10);
        assert_eq!(fart.spawn_dust, 211, "`Mount.cs:1098` overwrites the 213");
        assert_eq!(mount_data(53).unwrap().spawn_dust, 211, ":1106");
    }

    #[test]
    fn the_cart_items_summon_their_rows() {
        // `Item.DefaultToMinecart` (`Item.cs:48395-48400` + the per-item sites).
        let pairs = [
            (2343, 6),
            (3353, 11),
            (4066, 15),
            (4067, 16),
            (4426, 18),
            (4443, 22),
            (4450, 24),
            (4469, 33),
            (4472, 36),
            (4745, 38),
            (4763, 39),
            (5125, 51),
            (5288, 53),
        ];
        for (item, id) in pairs {
            assert_eq!(mount_type_of_item(item), Some(id), "item {item}");
        }
        // A saddle still routes to its own row (the shared table grew).
        assert_eq!(mount_type_of_item(1914), Some(0));
        assert_eq!(mount_type_of_buff(118), Some(6), "the cart buff maps back to its row");
    }

    #[test]
    fn the_flyer_rows_answer_the_flight_gates() {
        // `Mount.CanFly` (`:2860-2879`): `flightTimeMax > 0`, the Dark Mage
        // Book (48) excluded; `CanHover` (`:2881-2896`) needs `usesHover`
        // (the lava shark only while swimming).
        let mut m = MountState::default();
        // The Pigron flies (160 partial) and does not hover.
        m.set_mount(2, false);
        assert!(m.can_fly());
        assert!(!m.can_hover());
        m.dismount();
        // The Book hovers but never flies.
        m.set_mount(48, false);
        assert!(!m.can_fly());
        assert!(m.can_hover());
        m.dismount();
        // The lava shark hovers only in the swim frame state.
        m.set_mount(49, false);
        assert!(!m.can_fly(), "flightTimeMax is 0");
        m.test_set_frame_state(2);
        assert!(!m.can_hover());
        m.test_set_frame_state(4);
        assert!(m.can_hover());
        m.dismount();
        // The UFO ignores fatigue; the Pigron builds it.
        m.set_mount(7, false);
        assert!(m.does_hover_ignore_fatigue());
        m.dismount();
        m.set_mount(2, false);
        assert!(!m.does_hover_ignore_fatigue());
    }

    #[test]
    fn the_hover_fuel_drains_and_the_fatigue_builds() {
        let mut m = MountState::default();
        m.set_mount(5, false);
        assert!(m.can_hover());
        m.test_set_frame_state(2);
        // A fresh mount has an empty flight budget (`:6218`), so the very
        // first hover tick burns fatigue (gravity/defaultGravity = 1).
        let vy = m.hover_integrate(-4.0, false, true, false, 0.4);
        assert_eq!(m.fly_time(), 0);
        assert_eq!(m.fatigue, 1.0);
        // Ascending (control_jump) subtracts the acceleration: vy gets more
        // negative, i.e. upward, and saturates at the fatigue-scaled cap.
        assert!(vy < -4.0);
        // The grounded refill restores the budget (`ResetFlightTime`), then
        // the flight seconds burn before the fatigue builds again.
        m.test_set_fatigue(0.0);
        m.reset_flight_time(0.0, 0);
        assert_eq!(m.fly_time(), 320);
        let vy = m.hover_integrate(-4.0, false, true, false, 0.4);
        assert_eq!(m.fly_time(), 319);
        assert_eq!(m.fatigue, 0.0);
        assert!(vy < -4.0);
        // Not hovering (frame state 0): pure gravity, no fuel drain.
        m.test_set_frame_state(0);
        let vy = m.hover_integrate(1.0, false, false, false, 0.4);
        assert_eq!(vy, 1.4);
        assert_eq!(m.fly_time(), 319);
    }

    #[test]
    fn the_hover_fatigue_recovers_on_rest() {
        // `Mount.FatigueRecovery` (`Mount.cs:2961-2971`): -2 to 0.
        let mut m = MountState::default();
        m.set_mount(5, false);
        m.test_set_fatigue(7.0);
        m.fatigue_recovery();
        assert_eq!(m.fatigue, 5.0);
        m.test_set_fatigue(1.0);
        m.fatigue_recovery();
        assert_eq!(m.fatigue, 0.0);
    }

    use super::*;

    /// S1-P-i05 — the Scutlix's `AllowDirectionChange` (`Mount.cs:630-639`):
    /// the ability cooldown must drop under half the row's 20; every other
    /// mount (and the inactive state) allows the change. The run-key
    /// direction lock (`Player.cs:19837`) reads it. WRITTEN NOT RUN.
    #[test]
    fn s1_p_i05_the_scutlix_cooldown_gates_the_direction_change() {
        let mut m = MountState::default();
        assert!(m.allow_direction_change(), "no mount allows the change");
        m.set_mount(9, false);
        m.ability_cooldown = 20;
        assert!(!m.allow_direction_change(), "the full cooldown locks it");
        m.ability_cooldown = 10;
        assert!(!m.allow_direction_change(), "10 == 20/2 is still locked");
        m.ability_cooldown = 9;
        assert!(m.allow_direction_change(), "under half unlocks it");
        // A non-Scutlix mount always allows it (the `_type == 9` gate).
        // The Drill (8, `Mount.cs:1273-1341`) — a ported row with no
        // ability-cooldown direction gate; even a pinned cooldown never
        // locks it.
        let mut drill = MountState::default();
        drill.set_mount(8, false);
        drill.ability_cooldown = 100;
        assert!(drill.allow_direction_change(), "the drill never locks");
    }

    #[test]
    fn the_twelve_ground_rows_carry_the_exact_c_constants() {
        // DW-1.2 — values read from Mount.Initialize (`Mount.cs:696-2567`).
        let unicorn = mount_data(10).unwrap();
        assert_eq!(
            (unicorn.run_speed, unicorn.dash_speed, unicorn.acceleration),
            (4.0, 12.0, 0.3)
        );
        assert_eq!(unicorn.buff, 162);
        assert_eq!(unicorn.height_boost, 34);
        let horse = mount_data(40).unwrap();
        assert_eq!(
            (horse.run_speed, horse.dash_speed, horse.acceleration),
            (3.0, 9.0, 0.25)
        );
        assert_eq!(horse.buff, 275);
        assert_eq!(mount_data(41).unwrap().buff, 276);
        assert_eq!(mount_data(42).unwrap().buff, 277);
        let rudolph = mount_data(0).unwrap();
        assert_eq!(rudolph.flight_time_max, 160);
        assert_eq!(rudolph.buff, 90);
        assert!(mount_data(3).unwrap().constant_jump);
        assert!(mount_data(43).unwrap().constant_jump);
        assert!(mount_data(50).unwrap().constant_jump);
        assert_eq!(mount_data(3).unwrap().extra_fall, 10); // Mount.cs:869
        assert_eq!(mount_data(43).unwrap().extra_fall, 20); // :1861
        assert_eq!(mount_data(1).unwrap().fall_damage, 0.8); // :815
        assert_eq!(mount_data(14).unwrap().dash_speed, 0.0); // never set, :1644-1692
        assert_eq!(mount_data(9).unwrap().ability_charge_max, 40); // :1350
        assert_eq!(mount_data(43).unwrap().body_frame, 4); // :1876
                                                           // every row's buff id resolves and its offsets cover total_frames
        for &id in [0, 1, 3, 9, 10, 14, 40, 41, 42, 43, 47, 50].iter() {
            let row = mount_data(id).unwrap();
            assert!(row.buff > 0, "row {id} has no buff");
            assert_eq!(
                row.player_y_offsets.len() as i32,
                row.total_frames,
                "row {id}"
            );
        }
    }

    #[test]
    fn set_mount_dismount_and_swap_follow_the_c_guards() {
        // DW-1.3 — `SetMount` (:6196-6305) / `Dismount` (:6142-6194).
        let mut m = MountState::default();
        assert!(!m.is_active());
        assert_eq!(
            m.dismount(),
            None,
            "dismount while inactive is a no-op (:6144)"
        );
        // The wet gate: the Bee (5) cannot mount in water (`Mount.cs:6205`).
        assert_eq!(m.set_mount(5, true), None);
        // The Pigron (2) equips normally on dry land.
        assert_eq!(
            m.set_mount(2, false).map(|e| e.add_buff),
            Some(129)
        );
        m.dismount();
        let equip = m.set_mount(10, false).expect("unicorn equips");
        assert_eq!(
            equip,
            Equip {
                clear_buff: None,
                add_buff: 162,
                height_boost: 34,
                was_active: false,
                track_rider: false,
                size_override: None,
            }
        );
        assert!(m.is_active());
        // same type is a no-op (:6198) — the buff refresh must not re-add
        assert_eq!(m.set_mount(10, false), None);
        // type 5 while wet is guarded (:6198) even though the row is absent
        assert_eq!(m.set_mount(5, true), None);
        // a swap clears the old buff (:6203)
        let swap = m.set_mount(40, false).expect("horse swap");
        assert_eq!(
            swap,
            Equip {
                clear_buff: Some(162),
                add_buff: 275,
                height_boost: 34,
                was_active: true,
                track_rider: false,
                size_override: None,
            }
        );
        assert_eq!(m.kind(), 40);
        // dismount returns the buff to clear
        assert_eq!(m.dismount(), Some(275));
        assert!(!m.is_active());
        assert_eq!(m.kind(), -1, "Reset sets _type = -1 (:683)");
        // the Reset quirk: fatigue/abilityCooldown/abilityDuration survive (:675-694)
        let mut m2 = MountState::default();
        m2.set_mount(9, false);
        m2.ability_cooldown = 7;
        m2.ability_duration = 3;
        m2.fatigue = 5.0;
        m2.dismount();
        assert_eq!((m2.ability_cooldown, m2.ability_duration), (7, 3));
        assert_eq!(m2.fatigue, 5.0);
    }

    #[test]
    fn jump_height_and_speed_scale_with_x_velocity_like_the_c() {
        // DW-1.4 — `JumpHeight`/`JumpSpeed` (`Mount.cs:2810-2858`).
        let mut m = MountState::default();
        m.set_mount(0, false); // Rudolph, type 0
        assert_eq!(m.jump_height(0.0), 17);
        assert_eq!(m.jump_height(4.0), 18, "(int)(4/4f) = 1");
        assert_eq!(m.jump_height(-8.0), 19, "Abs first, then the cast");
        assert_eq!(m.jump_speed(7.0), 5.31 + 1.0, "|v|/7f");
        m.dismount();
        m.set_mount(1, false); // Bunny, type 1
        assert_eq!(m.jump_height(4.0), 16, "(int)(4/2.5f) = 1");
        assert_eq!(m.jump_speed(14.0), 5.01 + 2.0);
        m.dismount();
        m.set_mount(10, false); // Unicorn — no scaling branch
        assert_eq!(m.jump_height(16.0), 10);
        assert_eq!(m.jump_speed(16.0), 8.01);
    }

    #[test]
    fn run_speed_queen_slime_air_boost_and_can_fly() {
        // `RunSpeed` (:449-484) type-50 branch + `CanFly` (:2860-2879).
        let mut m = MountState::default();
        m.set_mount(50, false);
        assert_eq!(m.run_speed(), 5.5);
        m.frame_state = 2; // in air
        assert_eq!(m.run_speed(), 7.5, "type 50 in-air +2 (:472-475)");
        m.frame_state = 0;
        assert!(m.can_fly(), "flightTimeMax 80 (:2197)");
        m.dismount();
        m.set_mount(1, false);
        assert!(!m.can_fly(), "flightTimeMax 0 (:2870-2873)");
        assert_eq!(m.run_speed(), 4.0);
    }

    #[test]
    fn the_sixteen_missing_rows_carry_the_exact_c_constants() {
        // S1-L-9 — the rows `Mount.Initialize` (`Mount.cs:696-2567`) never
        // ported: the table carried 50 of the 66 `MountID` ids. Every value
        // below is read off the decompiled source.

        // 4 Turtle (`Mount.cs:1112-1162`).
        let turtle = mount_data(4).unwrap();
        assert_eq!((turtle.buff, turtle.height_boost), (131, 26));
        assert_eq!(
            (turtle.run_speed, turtle.dash_speed, turtle.swim_speed),
            (2.0, 5.0, 10.0)
        );
        assert_eq!(
            (turtle.acceleration, turtle.jump_height, turtle.jump_speed),
            (0.08, 12, 3.7)
        );
        assert_eq!(
            (turtle.fall_damage, turtle.spawn_dust, turtle.total_frames),
            (1.0, 56, 12)
        );
        assert_eq!(turtle.back_sheet, "Mount_Turtle");
        assert_eq!(
            (
                turtle.x_offset,
                turtle.y_offset,
                turtle.body_frame,
                turtle.player_head_offset
            ),
            (1, 13, 3, 28)
        );

        // 17 GolfCartSomebodySaveMe (`Mount.cs:1694-1739`).
        let golf = mount_data(17).unwrap();
        assert_eq!((golf.buff, golf.height_boost), (212, 16));
        assert_eq!((golf.run_speed, golf.acceleration), (8.0, 0.25));
        assert_eq!((golf.jump_height, golf.jump_speed), (20, 8.01));
        assert_eq!(
            (golf.fall_damage, golf.spawn_dust, golf.total_frames),
            (0.2, 15, 4)
        );
        assert_eq!(
            (golf.back_sheet, golf.back_extra_sheet),
            ("Extra_97", "Extra_96")
        );
        assert_eq!(
            (
                golf.x_offset,
                golf.y_offset,
                golf.body_frame,
                golf.player_head_offset
            ),
            (2, 1, 3, 18)
        );

        // 37 Flamingo (`Mount.cs:1796-1841`).
        let flamingo = mount_data(37).unwrap();
        assert_eq!((flamingo.buff, flamingo.height_boost), (265, 12));
        assert_eq!((flamingo.run_speed, flamingo.acceleration), (6.0, 0.15));
        assert_eq!((flamingo.jump_height, flamingo.jump_speed), (14, 6.01));
        assert_eq!(
            (flamingo.fall_damage, flamingo.spawn_dust, flamingo.total_frames),
            (0.2, 282, 10)
        );
        assert_eq!(flamingo.back_sheet, "Extra_149");
        assert_eq!(
            (
                flamingo.x_offset,
                flamingo.y_offset,
                flamingo.body_frame,
                flamingo.player_head_offset
            ),
            (5, 1, 4, 20)
        );

        // 45 SpookyWood (`Mount.cs:1940-1985`).
        let spooky = mount_data(45).unwrap();
        assert_eq!((spooky.buff, spooky.height_boost), (280, 25));
        assert_eq!(
            (spooky.run_speed, spooky.dash_speed, spooky.acceleration),
            (12.0, 16.0, 0.5)
        );
        assert_eq!((spooky.jump_height, spooky.jump_speed), (14, 7.0));
        assert_eq!(
            (spooky.fall_damage, spooky.spawn_dust, spooky.total_frames),
            (0.1, 6, 8)
        );
        assert!(spooky.emits_light, ":1951");
        assert_eq!(spooky.light_color, (0.6, 0.4, 0.35), ":1952");
        assert_eq!(
            (spooky.back_sheet, spooky.back_glow_sheet),
            // S1-L-8b: `GlowMask[283]` loads as `Glow_283`
            // (`AssetInitializer.cs:352-354`), not `GlowMask_283`.
            ("Extra_167", "Glow_283")
        );

        // 46 Santank (`Mount.cs:1987-2039`).
        let santank = mount_data(46).unwrap();
        assert_eq!(santank.buff, 281);
        assert_eq!(santank.height_boost, 0);
        assert_eq!(
            (
                santank.ability_charge_max,
                santank.ability_cooldown,
                santank.ability_duration
            ),
            (40, 40, 0)
        );
        assert_eq!(
            (santank.run_speed, santank.dash_speed, santank.acceleration),
            (8.0, 8.0, 0.4)
        );
        assert_eq!((santank.jump_height, santank.jump_speed), (8, 9.01));
        assert_eq!((santank.spawn_dust, santank.total_frames), (15, 27));
        assert_eq!(
            (santank.front_sheet, santank.front_extra_sheet),
            ("Extra_168", "Extra_168")
        );

        // 52 Wolf (`Mount.cs:2246-2308`).
        let wolf = mount_data(52).unwrap();
        assert_eq!(wolf.buff, 342);
        assert_eq!(
            (
                wolf.run_speed,
                wolf.acceleration,
                wolf.jump_height,
                wolf.jump_speed
            ),
            (9.5, 0.18, 18, 9.01)
        );
        assert_eq!(
            (wolf.fall_damage, wolf.spawn_dust, wolf.total_frames),
            (0.1, 31, 15)
        );
        assert_eq!(
            (wolf.front_sheet, wolf.front_extra_sheet),
            ("Extra_253", "Extra_254")
        );
        assert_eq!((wolf.x_offset, wolf.y_offset, wolf.body_frame), (4, -1, 3));

        // 54 Velociraptor (`Mount.cs:2310-2360`).
        let raptor = mount_data(54).unwrap();
        assert_eq!(raptor.buff, 370);
        assert_eq!(
            (raptor.run_speed, raptor.dash_speed, raptor.acceleration),
            (4.5, 7.5, 0.15)
        );
        assert_eq!((raptor.jump_height, raptor.jump_speed), (15, 6.01));
        assert_eq!(
            (raptor.fall_damage, raptor.spawn_dust, raptor.total_frames),
            (0.1, 31, 21)
        );
        assert_eq!(
            (raptor.front_sheet, raptor.front_extra_sheet),
            ("Extra_271", "Extra_272")
        );
        assert_eq!((raptor.x_offset, raptor.y_offset, raptor.body_frame), (4, -4, 0));
        assert_eq!(
            (raptor.flying.start, raptor.flying.count, raptor.flying.delay),
            (16, 5, 6)
        );

        // 55 Rat (`Mount.cs:2362-2415`).
        let rat = mount_data(55).unwrap();
        assert_eq!(rat.buff, 374);
        assert!(rat.dismounts_on_item_use, ":2371");
        assert_eq!(
            (rat.run_speed, rat.dash_speed, rat.acceleration),
            (4.5, 7.5, 0.15)
        );
        assert_eq!((rat.jump_height, rat.jump_speed), (15, 6.01));
        assert_eq!(
            (rat.fall_damage, rat.spawn_dust, rat.total_frames),
            (0.1, 31, 16)
        );
        assert_eq!(rat.front_sheet, "Extra_273");
        assert_eq!(
            (
                rat.x_offset,
                rat.y_offset,
                rat.body_frame,
                rat.player_head_offset
            ),
            (0, -2, 3, -30)
        );
        assert_eq!(
            (rat.dashing.start, rat.dashing.count, rat.dashing.delay),
            (11, 5, 16)
        );

        // 57-60 the roller skates (`Mount.SetAsRollerSkate`, `Mount.cs:2569-2605`).
        for (id, buff) in [(57, 378), (58, 379), (59, 380), (60, 381)] {
            let skate = mount_data(id).unwrap();
            assert_eq!(skate.buff, buff, "row {id}");
            assert_eq!(skate.spawn_dust, 31, "row {id}");
            assert!(skate.can_ride_minecart_tracks, "row {id}");
            assert!(skate.can_use_wings, "row {id}");
            assert!(skate.movement_stats_are_additive, "row {id}");
            assert_eq!(
                (skate.run_speed, skate.dash_speed, skate.acceleration),
                (1.0, 1.0, 0.1),
                "row {id}"
            );
            assert_eq!((skate.jump_height, skate.jump_speed), (2, 1.0), "row {id}");
            assert_eq!(skate.fall_damage, 1.0, "row {id}");
            assert_eq!(skate.total_frames, 1, "row {id}");
            assert_eq!(skate.body_frame, 3, "row {id}");
        }

        // 62-65 the Chillemt family (`Mount.SetAsChillet`, `Mount.cs:2670-2745`).
        for (id, buff, sheet) in [
            (62, 387, "Extra_301"),
            (63, 388, "Extra_302"),
            (64, 391, "Extra_301"),
            (65, 392, "Extra_302"),
        ] {
            let chillet = mount_data(id).unwrap();
            assert_eq!(chillet.buff, buff, "row {id}");
            assert_eq!(chillet.spawn_dust, 15, "row {id}");
            assert_eq!(chillet.height_boost, 4, "row {id}");
            assert_eq!(chillet.fall_damage, 0.5, "row {id}");
            assert_eq!(chillet.run_speed, 3.0, "row {id}");
            assert_eq!(chillet.acceleration, 0.32, "row {id}");
            assert_eq!(chillet.walking_grace_time_max, 10, "row {id}");
            assert_eq!(chillet.total_frames, 13, "row {id}");
            assert_eq!(chillet.back_sheet, sheet, "row {id}");
            assert_eq!(
                (
                    chillet.x_offset,
                    chillet.y_offset,
                    chillet.player_head_offset
                ),
                (7, -10, 10),
                "row {id}"
            );
            assert_eq!(
                (
                    chillet.dashing.start,
                    chillet.dashing.count,
                    chillet.dashing.delay
                ),
                (5, 7, 130),
                "row {id}"
            );
            assert!(chillet.idle.loops, "row {id}: :2732");
        }
        // The hard-mode pair (64/65) rides the faster numbers (:2677-2684).
        assert_eq!(
            (
                mount_data(64).unwrap().dash_speed,
                mount_data(64).unwrap().jump_height
            ),
            (9.0, 11)
        );
        assert_eq!(mount_data(64).unwrap().jump_speed, 8.1);
        assert_eq!(
            (
                mount_data(62).unwrap().dash_speed,
                mount_data(62).unwrap().jump_height
            ),
            (6.5, 6)
        );
        assert_eq!(mount_data(62).unwrap().jump_speed, 8.01);

        // Every row's offsets cover its frame count.
        for id in [4, 17, 37, 45, 46, 52, 54, 55, 57, 58, 59, 60, 62, 63, 64, 65] {
            let row = mount_data(id).unwrap();
            assert!(row.buff > 0, "row {id} has no buff");
            assert_eq!(
                row.player_y_offsets.len() as i32,
                row.total_frames,
                "row {id}"
            );
        }
    }

    #[test]
    fn the_new_mount_items_map_to_their_rows() {
        // S1-L-9 — `Item.SetDefaults`' `Item.mountType` assignments.
        assert_eq!(mount_type_of_item(2491), Some(4)); // :24194
        assert_eq!(mount_type_of_item(4264), Some(17)); // :36685
        assert_eq!(mount_type_of_item(4716), Some(37)); // :38778
        assert_eq!(mount_type_of_item(4793), Some(45)); // :39413 (44 + type - 4792)
        assert_eq!(mount_type_of_item(4794), Some(46)); // :39413
        assert_eq!(mount_type_of_item(5130), Some(52)); // :41600
        assert_eq!(mount_type_of_item(5510), Some(54)); // :43366
        // S1-P-i41: only 5525 is the rat saddle (`:43511-43512`); 5524 is
        // the Chlorophyte Visor (`:43504-43509`).
        assert_eq!(mount_type_of_item(5524), None);
        assert_eq!(mount_type_of_item(5525), Some(55)); // :43512
        // 5599 is the Retro Monolith (`:44051-44053`); 5600 the skates.
        assert_eq!(mount_type_of_item(5599), None);
        assert_eq!(mount_type_of_item(5600), Some(57)); // :44056
        // 5638/5639 are the Torch God music boxes (`:44293-44297`); only
        // 5640 is the green-skate saddle.
        assert_eq!(mount_type_of_item(5638), None);
        assert_eq!(mount_type_of_item(5639), None);
        assert_eq!(mount_type_of_item(5640), Some(58)); // :44300
        assert_eq!(mount_type_of_item(5641), Some(59)); // :44313
        assert_eq!(mount_type_of_item(5642), Some(60)); // :44326
        assert_eq!(mount_type_of_item(5665), Some(62)); // :44519
        assert_eq!(mount_type_of_item(5666), Some(63)); // :44531
        assert_eq!(mount_type_of_item(6150), Some(64)); // :47394
        assert_eq!(mount_type_of_item(6151), Some(65)); // :47407
    }

    #[test]
    fn the_rat_uses_its_own_idle_band_and_frame_table() {
        // S1-L-9 — `Mount.UpdateFrame` (`Mount.cs:4194-4208`) with
        // `Mount.idleFrames_Rat` (`:6665-6669`).
        let mut m = MountState::default();
        m.set_mount(55, false);
        m.update_frame(FRAME_STANDING, 0.0, 0.0, false, false, false, 0, 0);
        assert!(
            m.idle_time_next >= 120 && m.idle_time_next < 420,
            "the rat's idle band (:4196-4197): {}",
            m.idle_time_next
        );
        while m.idle_time < m.idle_time_next {
            m.update_frame(FRAME_STANDING, 0.0, 0.0, false, false, false, 0, 0);
        }
        assert_eq!(m.frame, 0, "the table's first entry (:4205)");
        // 64 ticks later the table index leaves the range and the C# resets
        // to the standing frame (`:4197-4202`).
        for _ in 0..64 {
            m.update_frame(FRAME_STANDING, 0.0, 0.0, false, false, false, 0, 0);
        }
        assert_eq!(m.idle_time, 0, "the out-of-range pick resets the timer");
        assert_eq!(m.frame, m.data().unwrap().standing.start);
    }

    #[test]
    fn the_super_cart_statics_are_the_c_values() {
        // S1-L-9 — `Mount.SuperCart*` (`Mount.cs:6660-6664`), read by the
        // speed getters when `_shouldSuperCart` is set (:478-505, :2829-2856).
        let mut m = MountState::default();
        m.set_mount(6, false); // a cart row
        assert_eq!(m.run_speed(), 13.0, "the vanilla cart speed");
        m.set_should_super_cart(true);
        assert_eq!(m.run_speed(), 20.0);
        assert_eq!(m.dash_speed(), 20.0);
        assert_eq!(m.acceleration(), 0.1);
        assert_eq!(m.jump_height(0.0), 15);
        assert_eq!(m.jump_speed(0.0), 5.15);
        m.set_should_super_cart(false);
        assert_eq!(m.run_speed(), 13.0);
    }

    #[test]
    fn player_offset_follows_the_frame_table() {
        // `PlayerOffset` (:407-421) / `PlayerOffsetHitbox` (:423-433).
        let mut m = MountState::default();
        assert_eq!(m.player_offset(), 0, "inactive");
        m.set_mount(10, false);
        m.frame = 15;
        assert_eq!(m.player_offset(), 32);
        m.frame = 16; // >= totalFrames
        assert_eq!(m.player_offset(), 0);
        m.frame = 3;
        assert_eq!(m.player_offset(), 30);
        assert_eq!(m.player_offset_hitbox(), -30 + 34);
    }

    // -- S1-L-8b: the field-level fidelity pass over the 66-row table --------
    // WRITTEN, NOT RUN (code-only mode): every pin below is read off
    // `Mount.Initialize` / the `SetAs*` helpers in `Mount.cs`.

    #[test]
    fn s1_l_8b_the_light_color_default_is_white() {
        // `MountData.lightColor = Vector3.One` (`Mount.cs:259`) — the BASE
        // default, not zero. Only the three `emitsLight` rows override it.
        assert_eq!(BASE.light_color, (1.0, 1.0, 1.0));
        assert!(!BASE.emits_light, "`Mount.cs:261` defaults false");
        // The Drill (`:1289-1290`).
        let drill = mount_data(8).unwrap();
        assert!(drill.emits_light, ":1289");
        assert_eq!(drill.light_color, (0.3, 0.3, 0.4), ":1290");
        // The Spooky Wood (`:1951-1952`).
        let spooky = mount_data(45).unwrap();
        assert!(spooky.emits_light, ":1951");
        assert_eq!(spooky.light_color, (0.6, 0.4, 0.35), ":1952");
        // The Lava Shark (`:2159-2160`).
        let shark = mount_data(49).unwrap();
        assert!(shark.emits_light, ":2159");
        assert_eq!(shark.light_color, (0.3, 0.15, 0.1), ":2160");
        // Every other row keeps the white default and emits nothing.
        for row in MOUNTS {
            if !matches!(row.id, 8 | 45 | 49) {
                assert!(!row.emits_light, "row {} emits no light", row.id);
                assert_eq!(row.light_color, (1.0, 1.0, 1.0), "row {}", row.id);
            }
        }
    }

    #[test]
    fn s1_l_8b_the_drill_row_carries_its_idle_start_and_glow_slots() {
        // 8 Drill (`Mount.cs:1273-1341`).
        let d = mount_data(8).unwrap();
        // `idleFrameStart = 8` (`:1316`) with count 0 / delay 12 (`:1314-1315`).
        assert_eq!((d.idle.start, d.idle.count, d.idle.delay), (8, 0, 12));
        // The six texture slots (`:1322-1329`), named by their content files
        // (`AssetInitializer.cs:446-451`).
        assert_eq!(d.back_sheet, "Mount_DrillRing", "DrillMount[0], :1322");
        assert_eq!(d.back_glow_sheet, "Mount_Glow_DrillRing", "DrillMount[3], :1323");
        assert_eq!(d.front_sheet, "Mount_DrillSeat", "DrillMount[1], :1326");
        assert_eq!(d.front_glow_sheet, "Mount_Glow_DrillSeat", "DrillMount[4], :1327");
        assert_eq!(d.front_extra_sheet, "Mount_DrillDiode", "DrillMount[2], :1328");
        assert_eq!(
            d.front_extra_glow_sheet, "Mount_Glow_DrillDiode",
            "DrillMount[5], :1329"
        );
        // `backTextureExtraGlow` stays Empty (`:1325`).
        assert_eq!(d.back_extra_glow_sheet, "");
    }

    #[test]
    fn s1_l_8b_the_glow_and_extra_slots_match_the_c_assignments() {
        // 5 Bee: `backTextureExtra = BeeMount[1]` (`Mount.cs:1212`).
        let bee = mount_data(5).unwrap();
        assert_eq!(bee.back_sheet, "Mount_Bee", ":1211");
        assert_eq!(bee.back_extra_sheet, "Mount_BeeWings", ":1212");
        // 12 CuteFishron: `backTextureGlow = CuteFishronMount[1]` (`:1585`).
        let fishron = mount_data(12).unwrap();
        assert_eq!(fishron.back_sheet, "Mount_CuteFishron1", ":1584");
        assert_eq!(fishron.back_glow_sheet, "Mount_CuteFishron2", ":1585");
        // 7 UFO: the glow is `frontTextureExtra = UfoMount[1]` (`:1269`) —
        // the C# puts it in the EXTRA slot, not a glow slot.
        let ufo = mount_data(7).unwrap();
        assert_eq!(ufo.front_sheet, "Mount_UFO", ":1268");
        assert_eq!(ufo.front_extra_sheet, "Mount_UFOGlow", ":1269");
        assert_eq!(ufo.front_glow_sheet, "", "the UFO has no frontTextureGlow");
        // 9 Scutlix: `frontTextureExtra = ScutlixMount[2]` (`:1394`).
        let scutlix = mount_data(9).unwrap();
        assert_eq!(scutlix.front_extra_sheet, "Mount_ScutlixEyeGlow", ":1394");
        // 11 mech cart: `frontTextureGlow = MinecartMechMount[1]` (`:1531`).
        let mech = mount_data(11).unwrap();
        assert_eq!(mech.front_sheet, "Mount_MinecartMech", ":1530");
        assert_eq!(mech.front_glow_sheet, "Mount_MinecartMechGlow", ":1531");
        assert_eq!(mech.front_extra_sheet, "", ":1532 Empty");
        // 24 shroom cart: `frontTextureGlow = Extra[116]` (`:985`).
        let shroom = mount_data(24).unwrap();
        assert_eq!(shroom.front_sheet, "Extra_115", ":984");
        assert_eq!(shroom.front_glow_sheet, "Extra_116", ":985");
        assert_eq!(shroom.front_extra_sheet, "", "SetAsMinecart :2795 Empty");
        // 38 coffin cart: `backTexture = frontTexture` (`:1030`).
        let coffin = mount_data(38).unwrap();
        assert_eq!(coffin.front_sheet, "Extra_150", ":1027");
        assert_eq!(coffin.back_sheet, "Extra_150", ":1030");
        // 45 SpookyWood / 47 WoF goat / 49 LavaShark: the GlowMask strips load
        // as `Glow_{id}` (`AssetInitializer.cs:352-354`).
        assert_eq!(mount_data(45).unwrap().back_glow_sheet, "Glow_283", ":1980");
        assert_eq!(mount_data(47).unwrap().back_glow_sheet, "Glow_284", ":2095");
        assert_eq!(mount_data(49).unwrap().back_glow_sheet, "Glow_285", ":2187");
        // 56 Bat: `frontTextureGlow = GlowMask[370]` (`:2474`); 61 Pixie's
        // stays Empty (`:2538`).
        assert_eq!(mount_data(56).unwrap().front_glow_sheet, "Glow_370", ":2474");
        assert_eq!(mount_data(61).unwrap().front_glow_sheet, "", ":2538");
        assert_eq!(mount_data(61).unwrap().front_extra_sheet, "Extra_291", ":2539");
    }

    #[test]
    fn s1_l_8b_the_swim_bands_copy_their_source_bands() {
        // The rows whose `swimFrame*` reads another band in Initialize.
        // 1 Bunny = inAir (`Mount.cs:848-850`).
        let bunny = mount_data(1).unwrap();
        assert_eq!(bunny.swim, bunny.in_air, ":848-850");
        assert_eq!((bunny.swim.start, bunny.swim.count, bunny.swim.delay), (5, 1, 12));
        // 10 Unicorn = inAir (`:1471-1473`).
        let unicorn = mount_data(10).unwrap();
        assert_eq!(unicorn.swim, unicorn.in_air, ":1471-1473");
        assert_eq!((unicorn.swim.start, unicorn.swim.count), (15, 1));
        // 14 Basilisk = inAir (`:1681-1683`).
        let basilisk = mount_data(14).unwrap();
        assert_eq!(basilisk.swim, basilisk.in_air, ":1681-1683");
        // 40-42 horses = inAir (`SetAsHorse` `:2656-2658`).
        for id in [40, 41, 42] {
            let horse = mount_data(id).unwrap();
            assert_eq!(horse.swim, horse.in_air, "row {id}, :2656-2658");
            assert_eq!((horse.swim.start, horse.swim.count), (15, 1));
        }
        // 47 WoF goat = inAir (`:2089-2091`).
        let goat = mount_data(47).unwrap();
        assert_eq!(goat.swim, goat.in_air, ":2089-2091");
        // 37 Flamingo = running count/start with its OWN 10-tick delay
        // (`:1830-1832`).
        let flamingo = mount_data(37).unwrap();
        assert_eq!(
            (flamingo.swim.start, flamingo.swim.count, flamingo.swim.delay),
            (2, 7, 10),
            ":1830-1832"
        );
        // 45 SpookyWood = running count/start, delay 20 (`:1974-1976`).
        let spooky = mount_data(45).unwrap();
        assert_eq!((spooky.swim.start, spooky.swim.count, spooky.swim.delay), (2, 6, 20));
        // 46 Santank = running (`:2027-2029`).
        let santank = mount_data(46).unwrap();
        assert_eq!(santank.swim, santank.running, ":2027-2029");
        // 52 Wolf / 54 Velociraptor / 55 Rat = inAir (`:2297-2299`,
        // `:2349-2351`, `:2404-2406`).
        for id in [52, 54, 55] {
            let row = mount_data(id).unwrap();
            assert_eq!(row.swim, row.in_air, "row {id}");
        }
        // 62-65 Chillemts = inAir (`SetAsChillet` `:2733-2735`).
        for id in [62, 63, 64, 65] {
            let row = mount_data(id).unwrap();
            assert_eq!(row.swim, row.in_air, "row {id}, :2733-2735");
        }
    }

    #[test]
    fn s1_l_8b_the_bat_and_pixie_carry_their_fatigue_max() {
        // `fatigueMax = 320` (`Mount.cs:2426` Bat, `:2490` Pixie) — the port
        // rows left it at the 0 default.
        assert_eq!(mount_data(56).unwrap().fatigue_max, 320, ":2426");
        assert_eq!(mount_data(61).unwrap().fatigue_max, 320, ":2490");
    }

    #[test]
    fn s1_l_8b_the_cart_sheet_names_are_the_content_files() {
        // `AssetInitializer.cs:424-441` — the port's old `MinecartMechMount*`/
        // `MinecartWoodMount`/`Mount_FishMinecart`/`Mount_DesertMinecart`
        // spellings matched no `.xnb` in `assets/Content/Images`.
        assert_eq!(mount_data(6).unwrap().front_sheet, "Mount_Minecart", ":959");
        assert_eq!(mount_data(11).unwrap().front_sheet, "Mount_MinecartMech", ":1530");
        assert_eq!(mount_data(13).unwrap().front_sheet, "Mount_MinecartWood", ":1638");
        assert_eq!(mount_data(15).unwrap().front_sheet, "Mount_MinecartDesert", ":966");
        assert_eq!(mount_data(16).unwrap().front_sheet, "Mount_MinecartMineCarp", ":1090");
    }

    #[test]
    fn s1_l_8b_every_named_mount_sheet_is_packed() {
        // The rows name content files; the atlas packing list must carry each
        // one or the draw silently falls through (`sky_tex` → None). The glow
        // slots are data-only today but their sheets are packed for the draw
        // pass that lands with M2-MOUNT.
        use crate::assets::STATIC_SHEET_NAMES;
        let packed = |name: &str| STATIC_SHEET_NAMES.contains(&name);
        for row in MOUNTS {
            for name in [
                row.back_sheet,
                row.back_extra_sheet,
                row.back_glow_sheet,
                row.back_extra_glow_sheet,
                row.front_sheet,
                row.front_glow_sheet,
                row.front_extra_sheet,
                row.front_extra_glow_sheet,
            ] {
                if !name.is_empty() {
                    assert!(packed(name), "row {} names {name}, never packed", row.id);
                }
            }
        }
    }
}
