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
    pub back_glow_sheet: &'static str,
    pub front_sheet: &'static str,
    pub front_extra_sheet: &'static str,
    pub x_offset: i32,
    pub y_offset: i32,
    /// `playerYOffsets[totalFrames]` (:155) — per-frame player draw offset.
    pub player_y_offsets: &'static [i32],
    pub body_frame: i32,
    pub player_head_offset: i32,
    pub player_x_offset: i32,
}

/// The `MountData` field defaults (C# initializer values, `Mount.cs:129-266`):
/// everything zero/false. Rows only override what `Initialize` sets.
const BASE: MountData = MountData {
    id: 0,
    buff: 0,
    height_boost: 0,
    flight_time_max: 0,
    uses_hover: false,
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
    back_glow_sheet: "",
    front_sheet: "",
    front_extra_sheet: "",
    x_offset: 0,
    y_offset: 0,
    player_y_offsets: &[],
    body_frame: 0,
    player_head_offset: 0,
    player_x_offset: 0,
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
    back_sheet: "Extra_169",
    // `TextureAssets.GlowMask[284]` — the port's glow sheet name is verified in
    // the draw phase (the content dir has no GlowMask_284 file).
    back_glow_sheet: "GlowMask_284",
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

/// 5 Bee (`Mount.cs:1163-1217`): hover, 320 fuel + 320 fatigue, wings. The
/// C# `backTextureExtra` (the animated wing strip, `Mount_BeeWings`) rides no
/// back-extra draw slot yet (only the glow slots exist) — a documented
/// deviation until the mount layers grow one.
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

/// 8 Drill (`Mount.cs:1273-1341`): swim 4, hover, light-emitting.
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
    spawn_dust: 226,
    total_frames: 1,
    standing: FrameSet { start: 0, count: 1, delay: 12, loops: false },
    running: FrameSet { start: 0, count: 1, delay: 12, loops: false },
    flying: FrameSet { start: 0, count: 1, delay: 12, loops: false },
    in_air: FrameSet { start: 0, count: 1, delay: 12, loops: false },
    idle: FrameSet { start: 0, count: 0, delay: 12, loops: false },
    swim: FrameSet { start: 0, count: 0, delay: 12, loops: false },
    back_sheet: "Mount_DrillRing",
    front_sheet: "Mount_DrillSeat",
    front_extra_sheet: "Mount_DrillDiode",
    x_offset: 1,
    y_offset: 4,
    player_y_offsets: &[4; 1],
    body_frame: 3,
    player_head_offset: 18,
    ..BASE
};

/// 12 CuteFishron (`Mount.cs:1536-1590`): swim 16, swim frames, hover with the
/// half speed multiplier (`MountFishronSpecial` — not ported). The back-glow
/// strip `Mount_CuteFishron2` also rides no slot yet (same deviation as the
/// Bee's back-extra).
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
/// `flightTimeMax` 0, `fatigueMax` 320.
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
    spawn_dust: 35,
    total_frames: 8,
    standing: FrameSet { start: 4, count: 1, delay: 12, loops: false },
    running: FrameSet { start: 4, count: 4, delay: 14, loops: false },
    in_air: FrameSet { start: 4, count: 1, delay: 6, loops: false },
    swim: FrameSet { start: 0, count: 4, delay: 16, loops: false },
    back_sheet: "Extra_172",
    x_offset: 2,
    y_offset: 1,
    player_y_offsets: &LAVA_SHARK_Y_OFFSETS,
    body_frame: 3,
    player_head_offset: 16,
    ..BASE
};

/// 56 Bat (`Mount.cs:2416-2478`): dismounts on item use, no fuel, hover with
/// the dash-speed rise cap.
const BAT: MountData = MountData {
    id: 56,
    buff: 377,
    flight_time_max: 320,
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
    x_offset: -4,
    y_offset: -1,
    player_y_offsets: &[0; 16],
    body_frame: 3,
    player_head_offset: -24,
    ..BASE
};

/// 61 Pixie (`Mount.cs:2479-2542`): same flight shape as the Bat.
const PIXIE: MountData = MountData {
    id: 61,
    buff: 384,
    flight_time_max: 320,
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
/// Terra-Fart): `yOffset = 13 + verticalOffset` (= 3), the y-offsets
/// `8 - verticalOffset + playerVerticalOffset` (= 0). The shared offsets are
/// STATIC so the row stays a promoted `const` (`'static` borrow).
const CART_VO_Y_OFFSETS: [i32; 3] = [0, 0, 0];

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
const MINECART_MECH: MountData = MountData {
    id: 11,
    buff: 166,
    minecart: true,
    spawn_dust: 0,
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
    front_sheet: "MinecartMechMount",
    front_extra_sheet: "MinecartMechMountGlow",
    x_offset: -1,
    y_offset: 11,
    player_y_offsets: &[9; 3],
    body_frame: 3,
    player_head_offset: 14,
    ..BASE
};

/// 13 MinecartWood (`Mount.cs:1591-1641`, hand-written): run/dash 10, accel
/// 0.03, jump 12, buff 184, `MinecartWoodMount`.
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
    front_sheet: "MinecartWoodMount",
    x_offset: 1,
    y_offset: 13,
    player_y_offsets: &[8; 3],
    body_frame: 3,
    player_head_offset: 14,
    ..BASE
};

/// 16 FishMinecart (`Mount.cs:1044-1086`, hand-written; identical to the
/// `SetAsMinecart` body), buff 210, `FishMinecartMount`.
const FISH_MINECART: MountData = MountData {
    id: 16,
    buff: 210,
    ..cart(16, 210, "Mount_FishMinecart")
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

// The plain `SetAsMinecart` cosmetic rows (`Mount.cs:966-1024`).
const DESERT_MINECART: MountData = cart(15, 208, "Mount_DesertMinecart");
const BEE_MINECART: MountData = cart(18, 220, "Extra_108");
const LADYBUG_MINECART: MountData = cart(19, 222, "Extra_109");
const PIGRON_MINECART: MountData = cart(20, 224, "Extra_110");
const SUNFLOWER_MINECART: MountData = cart(21, 226, "Extra_111");
const HELL_MINECART: MountData = cart(22, 228, "Extra_112");
const SHROOM_MINECART: MountData = MountData {
    front_extra_sheet: "Extra_116",
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
const HELLWIZARD_MINECART: MountData = cart(38, 269, "Extra_150");
const FART_MINECART: MountData = cart_vo(51, 338, "Extra_246");
const TERRA_FART_MINECART: MountData = cart_vo(53, 346, "Extra_251");
// -- Special-ability rows (M2-MOUNT-4, `Mount.cs:1987-2044` Santank,
//    `Mount.cs:2540-2605` roller skates, `Mount.cs:2544-2746` chillets) -------

/// 46 Santank (`Mount.cs:1987-2044`): the cannon mount — ability 40/40/0
/// (the auto-targeting fire is the follow-up), run/dash 8, accel 0.4,
/// jump 8/9.01, 27 frames, the `Extra_168` strip front + extra.
static SANTANK_Y_OFFSETS: [i32; 27] = [
    4, 6, 6, 4, 4, 4, 4, 6, 6, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
];
const SANTANK: MountData = MountData {
    id: 46,
    buff: 281,
    ability_charge_max: 40,
    ability_cooldown: 40,
    ability_duration: 0,
    height_boost: 0,
    flight_time_max: 0,
    fall_damage: 0.0,
    run_speed: 8.0,
    dash_speed: 8.0,
    acceleration: 0.4,
    jump_height: 8,
    jump_speed: 9.01,
    block_extra_jumps: false,
    spawn_dust: 15,
    total_frames: 27,
    standing: FrameSet {
        start: 0,
        count: 1,
        delay: 12,
        loops: false,
    },
    running: FrameSet {
        start: 0,
        count: 11,
        delay: 12,
        loops: false,
    },
    in_air: FrameSet {
        start: 1,
        count: 11,
        delay: 12,
        loops: false,
    },
    swim: FrameSet {
        start: 0,
        count: 11,
        delay: 12,
        loops: false,
    },
    front_sheet: "Extra_168",
    front_extra_sheet: "Extra_168",
    x_offset: 1,
    y_offset: 1,
    player_y_offsets: &SANTANK_Y_OFFSETS,
    body_frame: 3,
    player_head_offset: 2,
    ..BASE
};

const ROLLER_SKATE_BLUE: MountData = roller_skate(57, 378);
const ROLLER_SKATE_GREEN: MountData = roller_skate(58, 379);
const ROLLER_SKATE_CLASSIC: MountData = roller_skate(59, 380);
const ROLLER_SKATE_PARTY: MountData = roller_skate(60, 381);
const CHILLET: MountData = chillet(62, 387, "Extra_301", false);
const CHILLET_IGNIS: MountData = chillet(63, 388, "Extra_302", false);
const CHILLET_TRUSTY: MountData = chillet(64, 391, "Extra_301", true);
const CHILLET_TRUSTY_IGNIS: MountData = chillet(65, 392, "Extra_302", true);

/// `Mount.SetAsChillet` (`Mount.cs:2670-2746`): the Palworld ride — the
/// shared body plus the hardmode branch (64/65: dash 9, jump 11/8.1).
const CHILLET_Y_OFFSETS: [i32; 13] = [10, 10, 10, 10, 12, 16, 12, 10, 12, 16, 16, 14, 10];

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
        standing: FrameSet {
            start: 0,
            count: 4,
            delay: 12,
            loops: false,
        },
        running: FrameSet {
            start: 6,
            count: 6,
            delay: 30,
            loops: false,
        },
        dashing: FrameSet {
            start: 5,
            count: 7,
            delay: 130,
            loops: false,
        },
        flying: FrameSet {
            start: 1,
            count: 6,
            delay: 6,
            loops: false,
        },
        in_air: FrameSet {
            start: 4,
            count: 1,
            delay: 12,
            loops: false,
        },
        idle: FrameSet {
            start: 0,
            count: 4,
            delay: 12,
            loops: true,
        },
        swim: FrameSet {
            start: 4,
            count: 1,
            delay: 12,
            loops: false,
        },
        back_sheet,
        x_offset: 7,
        y_offset: -10,
        player_y_offsets: &CHILLET_Y_OFFSETS,
        body_frame: 3,
        player_head_offset: 10,
        ..BASE
    }
}

/// `Mount.SetAsRollerSkate` (`Mount.cs:2569-2605`): the grindable rides —
/// additive stats, run/dash 1, accel 0.1, jump 2/1, wings allowed, no
/// texture of its own (the player draws).
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
        standing: FrameSet {
            start: 0,
            count: 1,
            delay: 6,
            loops: false,
        },
        x_offset: 0,
        y_offset: 0,
        player_y_offsets: &[2; 1],
        body_frame: 3,
        player_head_offset: 2,
        ..BASE
    }
}
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
    SANTANK,           // 46
    ROLLER_SKATE_BLUE, // 57
    ROLLER_SKATE_GREEN,// 58
    ROLLER_SKATE_CLASSIC,// 59
    ROLLER_SKATE_PARTY, // 60
    CHILLET,           // 62
    CHILLET_IGNIS,     // 63
    CHILLET_TRUSTY,    // 64
    CHILLET_TRUSTY_IGNIS, // 65
];

/// The saddle items — `Item.SetDefaults`' `Item.mountType` assignments:
/// `Item.cs:20779` Reindeer Bells 1914 -> Rudolph 0, `:24044` Fuzzy Carrot
/// 2428 -> Bunny 1, `:24068` Slimy Saddle 2430 -> Slime 3, `:26465` Brain
/// Scrambler 2771 -> Scutlix 9, `:30007` Blessed Apple 3260 -> Unicorn 10,
/// `:33385` Ancient Horn 3771 -> Basilisk 14, `:39363` the horse saddles
/// 4785-4787 -> 40-42, `:39395` Pogo Stick 4791 -> 43, `:39413` Wall Of
/// Flesh Goat Mount Item 4795 -> 44 + (4795 - 4792) = 47, `:40133` Queen
/// Slime Mount Saddle 4981 -> 50.
const MOUNT_ITEMS: [(u16, i32); 58] = [
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
    // M2-MOUNT-4: the Santank item (`Item.cs:39400` 4794 -> 46), the four
    // roller-skate items (`:44056` 5600 -> 57, `:44300` 5640 -> 58,
    // `:44313` 5641 -> 59, `:44326` 5642 -> 60) and the four Chillets
    // (`:44519` 5665 -> 62, `:44531` 5666 -> 63, `:47394` 6150 -> 64,
    // `:47407` 6151 -> 65).
    (4794, 46),
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
/// :490-493, :502-505, :2829-2832, :2853-2856). Only the minecart system writes
/// them (M2-MOUNT-3); until then the values stay 0 and the branches are dead.
const SUPER_CART_RUN_SPEED: f32 = 0.0;
const SUPER_CART_DASH_SPEED: f32 = 0.0;
const SUPER_CART_ACCELERATION: f32 = 0.0;
const SUPER_CART_JUMP_HEIGHT: i32 = 0;
const SUPER_CART_JUMP_SPEED: f32 = 0.0;

/// What the caller must apply after a successful [`MountState::set_mount`]:
/// the buff swap plus the hitbox growth (`SetMount` `Mount.cs:6226`,
/// :6264-6277 — the PlayerSize-delegate branch does not exist in this set).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Equip {
    /// `ClearBuff(old data.buff)` on a swap (:6203).
    pub clear_buff: Option<i32>,
    /// `AddBuff(data.buff, 3600)` (:6226).
    pub add_buff: i32,
    /// The new player height is `42 + height_boost`, bottom-anchored.
    pub height_boost: i32,
}

/// The mount instance state — `Mount` (`Mount.cs:325-383`). Plain data so the
/// per-tick `Player` clone (interpolated) stays cheap; the per-type payload
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
    }

    pub(crate) fn is_active(&self) -> bool {
        self.active
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
    pub(crate) fn update_frame(&mut self, state: i32, vx: f32, vy: f32, just_jumped: bool, control_down: bool) {
        let Some(row) = self.data() else { return };
        // `:3583-3611`: a state change resets the counter (the exceptions for
        // types 48/56/61/55 reference rows outside this table).
        if self.frame_state != state {
            self.frame_state = state;
            self.frame_counter = 0.0;
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
                // `:4145-4181`: the idle timer. The per-type ranges (17:
                // 120-300, 2: 600-900, 55: 120-420) reference rows outside
                // this table; the base pick is `Main.rand.Next(900, 1500)`
                // through the stand-in stream.
                if row.idle.count != 0 && self.idle_time == 0 {
                    self.idle_time_next = idle_range(&mut self.idle_rng, 900, 1500);
                }
                if row.idle.count != 0 {
                    self.idle_time += 1;
                }
                self.frame_counter += 1.0;
                if row.idle.count != 0 && self.idle_time >= self.idle_time_next {
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
            FRAME_DASHING => {
                // `:4476-4505`: the dash loop paces on |velocity.X|, the
                // flip sign for Scutlix (9) alone not modelled.
                self.frame_counter += vx.abs();
                if self.frame_counter > row.dashing.delay as f32 {
                    self.frame_counter -= row.dashing.delay as f32;
                    self.frame += 1;
                }
                if self.frame < row.dashing.start
                    || self.frame >= row.dashing.start + row.dashing.count
                {
                    self.frame = row.dashing.start;
                }
            }
            _ => {}
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

    /// `Mount.SetMount` (`Mount.cs:6196-6305`) — the state transitions; the
    /// buff add/clear and the hitbox growth are returned for the caller
    /// (the net SendData(13) :6222-6225 and the sound/dust effects have no port
    /// counterpart yet). `wet` is `Player.wet` (the `m == 5` guard :6198).
    pub(crate) fn set_mount(&mut self, m: i32, wet: bool) -> Option<Equip> {
        let old_buff = self.data().map(|d| d.buff);
        if self.kind == m || m <= -1 || m >= 66 || (m == 5 && wet) || mount_data(m).is_none() {
            return None;
        }
        let row = mount_data(m).unwrap();
        if !self.active {
            self.active = true; // :6214-6217
        }
        // :6202-6213 — the clear-buff/cart/rotation resets only concern an
        // already-active swap (the clear travels in `Equip`) and carts; neither
        // carries more state in this set.
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

    /// `Mount.CanFly` (`Mount.cs:2860-2879`) — the type-54 selective-fly branch
    /// needs `_mountSpecificData` (M2-MOUNT-4) and cannot fire on this table.
    pub(crate) fn can_fly(&self) -> bool {
        if !self.active {
            return false;
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

    /// `Mount.ResetFlightTime` (`Mount.cs:5262-5273`) — the type-0 |v|*20 bonus
    /// and the type-54 wings borrow need the Player (Phase 2 passes velocity).
    pub(crate) fn reset_flight_time(&mut self, x_velocity: f32) {
        self.fly_time = if self.active {
            mount_data(self.kind).map_or(0, |d| d.flight_time_max)
        } else {
            0
        };
        if self.kind == 0 {
            self.fly_time += (x_velocity.abs() * 20.0) as i32;
        }
    }

    pub(crate) fn flight_time_max(&self) -> i32 {
        self.data().map_or(0, |d| d.flight_time_max)
    }

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
    #[test]
    fn the_special_ability_rows_carry_the_c_constants() {
        // `Mount.cs:1987-2044` Santank: the cannon's 40/40/0 ability + the
        // run/jump stats; `SetAsChillet`/`SetAsRollerSkate` shapes.
        let s = mount_data(46).expect("the Santank row");
        assert_eq!((s.buff, s.spawn_dust), (281, 15));
        assert_eq!(
            (s.ability_charge_max, s.ability_cooldown, s.ability_duration),
            (40, 40, 0)
        );
        assert_eq!((s.run_speed, s.dash_speed, s.acceleration), (8.0, 8.0, 0.4));
        assert_eq!((s.jump_height, s.jump_speed), (8, 9.01));
        assert!(!s.block_extra_jumps);
        assert_eq!((s.front_sheet, s.front_extra_sheet), ("Extra_168", "Extra_168"));

        // The Chillets: 62/63 vanilla, 64/65 the hardmode branch (dash 9,
        // jump 11/8.1), both with walking grace + the dashing band.
        for (id, buff, dash, jh, js, hm) in [
            (62, 387, 6.5, 6, 8.01, false),
            (63, 388, 6.5, 6, 8.01, false),
            (64, 391, 9.0, 11, 8.1, true),
            (65, 392, 9.0, 11, 8.1, true),
        ] {
            let c = mount_data(id).expect("the chillet row");
            assert_eq!(c.buff, buff);
            assert_eq!((c.run_speed, c.dash_speed), (3.0, dash));
            assert_eq!((c.jump_height, c.jump_speed), (jh, js));
            assert_eq!(c.walking_grace_time_max, 10);
            assert_eq!(c.dashing.start, 5);
            assert_eq!(c.dashing.count, 7);
            assert_eq!((c.x_offset, c.y_offset, c.player_head_offset), (7, -10, 10));
            assert!(c.height_boost == 4 && c.flight_time_max == 0);
            let _ = hm;
        }
        // The roller skates: additive stats, wing-allowed, grindable, no strip.
        for (id, buff) in [(57, 378), (58, 379), (59, 380), (60, 381)] {
            let r = mount_data(id).expect("the skate row");
            assert_eq!(r.buff, buff);
            assert!(r.movement_stats_are_additive);
            assert!(r.can_use_wings);
            assert!(r.can_ride_minecart_tracks);
            assert_eq!((r.run_speed, r.dash_speed, r.acceleration), (1.0, 1.0, 0.1));
            assert_eq!((r.jump_height, r.jump_speed), (2, 1.0));
            assert!(r.front_sheet.is_empty() && r.back_sheet.is_empty());
        }
        // The summon items reach their rows.
        assert_eq!(mount_type_of_item(4794), Some(46));
        assert_eq!(mount_type_of_buff(281), Some(46));
        for (item, id) in [(5600, 57), (5640, 58), (5641, 59), (5642, 60)] {
            assert_eq!(mount_type_of_item(item), Some(id));
        }
        for (item, id) in [(5665, 62), (5666, 63), (6150, 64), (6151, 65)] {
            assert_eq!(mount_type_of_item(item), Some(id));
        }
    }

    #[test]
    fn the_dashing_frame_advances_the_band() {
        // `Mount.cs:4476-4505` (FRAME_DASHING) and the row's 7/130/5 band.
        let mut m = MountState::default();
        m.set_mount(62, false);
        m.test_set_frame_state(crate::mount::FRAME_DASHING);
        m.update_frame(crate::mount::FRAME_DASHING, 6.5, 0.0, false, false);
        assert_eq!(m.frame, 5, "the band starts at dashingFrameStart");
        for _ in 0..140 {
            m.update_frame(crate::mount::FRAME_DASHING, 6.5, 0.0, false, false);
        }
        assert!(
            (5..12).contains(&m.frame),
            "the frame stays inside the 7-wide band, got {}",
            m.frame
        );
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
        let wood = mount_data(13).unwrap();
        assert_eq!((wood.run_speed, wood.acceleration), (10.0, 0.03));
        let mole = mount_data(39).unwrap();
        assert_eq!((mole.run_speed, mole.acceleration), (6.0, 0.02));
        assert_eq!(mole.y_offset, 13 - 2);
        // The fart carts' vertical offsets (`Mount.cs:1097`, `:1105`).
        let fart = mount_data(51).unwrap();
        assert_eq!(fart.y_offset, 13 - 10);
        assert_eq!(fart.player_y_offsets[0], 0);
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
        m.reset_flight_time(0.0);
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
                height_boost: 34
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
                height_boost: 34
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
        assert!(!m.can_fly(), "flightTimeMax 0 (:2867-2869)");
        assert_eq!(m.run_speed(), 4.0);
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
}
