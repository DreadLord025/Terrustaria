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
//! the hover/flyer fatigue pool (M2-MOUNT-2), everything minecart
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
];

/// The saddle items — `Item.SetDefaults`' `Item.mountType` assignments:
/// `Item.cs:20779` Reindeer Bells 1914 -> Rudolph 0, `:24044` Fuzzy Carrot
/// 2428 -> Bunny 1, `:24068` Slimy Saddle 2430 -> Slime 3, `:26465` Brain
/// Scrambler 2771 -> Scutlix 9, `:30007` Blessed Apple 3260 -> Unicorn 10,
/// `:33385` Ancient Horn 3771 -> Basilisk 14, `:39363` the horse saddles
/// 4785-4787 -> 40-42, `:39395` Pogo Stick 4791 -> 43, `:39413` Wall Of
/// Flesh Goat Mount Item 4795 -> 44 + (4795 - 4792) = 47, `:40133` Queen
/// Slime Mount Saddle 4981 -> 50.
const MOUNT_ITEMS: [(u16, i32); 12] = [
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
        // unknown id (2 Pigron is not in the partial table) is rejected
        assert_eq!(m.set_mount(2, false), None);
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
