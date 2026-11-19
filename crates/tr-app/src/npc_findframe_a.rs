//! S1-D-3 FindFrame coverage wave A — the LOW-id kinds of
//! `NPC.FindFrame` (`NPC.cs:66684-77370`) that the port's `npc.rs`
//! `find_frame` match does not carry yet: the nine slime variants, the
//! five demon-eye variants and the ten zombie/rare walkers.
//!
//! Wave split (2026-11-18): wave B (`task/S1-D-3-findframe-b`,
//! `npc_findframe_b.rs`) owns the 25 MID/HIGH-id kinds (31/34/75/77/78/
//! 79/80/82/85/86/104/109/110/122/140/155/167/174/177/197/206/289/294/
//! 296/630); the frost legion 338-352 ride `task/S1-D-3-frost-legion`,
//! the pumpkin family 305-330 landed with S1-F-13b, and the bosses with
//! in-flight AI machines keep their own `find_frame` arms. Already
//! ported in `npc.rs::find_frame` and NOT touched here: the base Slime
//! (1), DemonEye (2), Zombie (3), KingSlime (4), ServantOfCthulhu (5),
//! EaterOfSouls (6) / MeteorHead (23), the bats (49/51/83 + VoodooDemon
//! 67), the worms (7-15, single-frame `{}` arms) and WanderingEye (133).
//! This module owns the 24 kinds listed in [`FindFrameKind::vanilla_type`].
//!
//! Shape: the machines are pure functions over a small
//! [`FrameState`]/[`FrameInput`] pair (the `npc_findframe_b.rs` /
//! `tr-core/src/drawdata_anim.rs` S1-F-9b precedent). The wiring landed
//! with S1-D-3ff: the 24 kinds joined `NpcKind` and `npc.rs::find_frame`
//! routes them through [`find_frame_kind`] + [`find_frame_a`] (the
//! `Npc::find_frame_wave_a` bridge copies `row`/`counter`/`sprite_dir`/
//! `rotation` to/from the NPC). No RNG is touched (none of these arms
//! draws `Main.rand` — the eye group's dust spawn is type 23's, not this
//! wave's), so the wiring cannot move a stream.
//!
//! Row convention: the port tracks frame ROW indices, not pixels — the
//! C#'s `num` (the sheet height / `Main.npcFrameCount[type]`,
//! `NPC.cs:66694`) cancels out of every comparison, exactly as in
//! `npc.rs::find_frame`'s existing arms. The per-kind row counts are
//! `Main.npcFrameCount` (`Main.cs:67082`'s 697-entry initializer), counted
//! entry-by-entry in [`FindFrameKind::frame_count`].
//!
//! Deliberate skips (recorded in docs/ANALYSIS.md "S1-D-3 FindFrame wave
//! A"): the slime group's `type == 302 || type == 304` spriteDirection
//! snap (`NPC.cs:70824-70827` — the town slimes are not this wave's
//! kinds), the eye group's type-23 dust spawn (`:74007-74013`), the
//! 121/660 `localAI[0]` init (`:74014-74018`), the 660-only 6-tick
//! cadence (`:74046-74049`) and the dead second flip branch
//! (`:74032-74043` — its `type == 2` arm is shadowed by the first `if`,
//! and 121/660 are not this wave's kinds).

// The wiring landed with S1-D-3ff (`npc.rs::find_frame_wave_a`); the
// module stays a binary-crate module, so the unused-in-bin targets keep
// the allow (the `npc_findframe_b.rs` precedent).
#![allow(dead_code)]

/// The 24 wave-A kinds, keyed by their VANILLA `NPC.type`
/// (`ID/NPCID.cs`'s `public const short` rows; the `NpcKind` variants
/// landed with the S1-D-3ff wiring — [`find_frame_kind`] maps them).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FindFrameKind {
    // The slime group — `FindFrame`'s case 1/16/59/71/81/138/147/183/
    // 184/204/225/302/304/333-336/535/537/658/659/667/676 arm
    // (`NPC.cs:70801-70846`); this wave takes its nine LOW-id members
    // (the base Slime 1 is already in `npc.rs`).
    /// `NPCID.MotherSlime` = 16 (`NPCID.cs:10709`).
    MotherSlime,
    /// `NPCID.LavaSlime` = 59 (`NPCID.cs:10795`).
    LavaSlime,
    /// `NPCID.DungeonSlime` = 71 (`NPCID.cs:10819`).
    DungeonSlime,
    /// `NPCID.CorruptSlime` = 81 (`NPCID.cs:10839`).
    CorruptSlime,
    /// `NPCID.IlluminantSlime` = 138 (`NPCID.cs:10953`).
    IlluminantSlime,
    /// `NPCID.IceSlime` = 147 (`NPCID.cs:10971`).
    IceSlime,
    /// `NPCID.Crimslime` = 183 (`NPCID.cs:11043`).
    Crimslime,
    /// `NPCID.SpikedIceSlime` = 184 (`NPCID.cs:11045`).
    SpikedIceSlime,
    /// `NPCID.UmbrellaSlime` = 225 (`NPCID.cs:11127`).
    UmbrellaSlime,
    // The eye group — `FindFrame`'s case 2/23/121/169/190-194/317/318/
    // 660 arm (`NPC.cs:73994-74059`); this wave takes its five LOW-id
    // demon-eye variants (2/23 are already in `npc.rs`).
    /// `NPCID.CataractEye` = 190 (`NPCID.cs:11057`).
    CataractEye,
    /// `NPCID.SleepyEye` = 191 (`NPCID.cs:11059`).
    SleepyEye,
    /// `NPCID.DialatedEye` = 192 (`NPCID.cs:11061`).
    DialatedEye,
    /// `NPCID.GreenEye` = 193 (`NPCID.cs:11063`).
    GreenEye,
    /// `NPCID.PurpleEye` = 194 (`NPCID.cs:11065`).
    PurpleEye,
    // The walker group — `FindFrame`'s case 3/52/53/132/161/186-189/
    // 200/223/251/254/255/319-321/331/332/342/536/590/691 arm
    // (`NPC.cs:76324-76391`); this wave takes its ten LOW-id members
    // (the base Zombie 3 is already in `npc.rs::walker_frame`).
    /// `NPCID.DoctorBones` = 52 (`NPCID.cs:10781`).
    DoctorBones,
    /// `NPCID.TheGroom` = 53 (`NPCID.cs:10783`).
    TheGroom,
    /// `NPCID.BaldZombie` = 132 (`NPCID.cs:10941`).
    BaldZombie,
    /// `NPCID.ZombieEskimo` = 161 (`NPCID.cs:10999`).
    ZombieEskimo,
    /// `NPCID.PincushionZombie` = 186 (`NPCID.cs:11049`).
    PincushionZombie,
    /// `NPCID.SlimedZombie` = 187 (`NPCID.cs:11051`).
    SlimedZombie,
    /// `NPCID.SwampZombie` = 188 (`NPCID.cs:11053`).
    SwampZombie,
    /// `NPCID.TwiggyZombie` = 189 (`NPCID.cs:11055`).
    TwiggyZombie,
    /// `NPCID.FemaleZombie` = 200 (`NPCID.cs:11077`).
    FemaleZombie,
    /// `NPCID.ZombieRaincoat` = 223 (`NPCID.cs:11123`).
    ZombieRaincoat,
}

impl FindFrameKind {
    /// The vanilla `NPC.type` (`ID/NPCID.cs`'s const rows).
    pub fn vanilla_type(self) -> i16 {
        match self {
            Self::MotherSlime => 16,
            Self::LavaSlime => 59,
            Self::DungeonSlime => 71,
            Self::CorruptSlime => 81,
            Self::IlluminantSlime => 138,
            Self::IceSlime => 147,
            Self::Crimslime => 183,
            Self::SpikedIceSlime => 184,
            Self::UmbrellaSlime => 225,
            Self::CataractEye => 190,
            Self::SleepyEye => 191,
            Self::DialatedEye => 192,
            Self::GreenEye => 193,
            Self::PurpleEye => 194,
            Self::DoctorBones => 52,
            Self::TheGroom => 53,
            Self::BaldZombie => 132,
            Self::ZombieEskimo => 161,
            Self::PincushionZombie => 186,
            Self::SlimedZombie => 187,
            Self::SwampZombie => 188,
            Self::TwiggyZombie => 189,
            Self::FemaleZombie => 200,
            Self::ZombieRaincoat => 223,
        }
    }

    /// `Main.npcFrameCount[type]` — the row counts counted entry-by-entry
    /// out of the 697-entry initializer (`Main.cs:67082-67153`; entry
    /// `type` sits on line `67084 + type / 10`, slot `type % 10`). Every
    /// slime and eye variant shares the base kind's 2-row hop/aim strip;
    /// every walker shares the Zombie's 3-row stand/walk strip.
    pub fn frame_count(self) -> u32 {
        match self {
            // Main.cs:67085 (entry 16): 2.
            Self::MotherSlime => 2,
            // Main.cs:67089 (entry 59): 2.
            Self::LavaSlime => 2,
            // Main.cs:67091 (entry 71): 2.
            Self::DungeonSlime => 2,
            // Main.cs:67092 (entry 81): 2.
            Self::CorruptSlime => 2,
            // Main.cs:67097 (entry 138): 2.
            Self::IlluminantSlime => 2,
            // Main.cs:67098 (entry 147): 2.
            Self::IceSlime => 2,
            // Main.cs:67102 (entries 183/184): 2, 2.
            Self::Crimslime => 2,
            Self::SpikedIceSlime => 2,
            // Main.cs:67106 (entry 225): 2.
            Self::UmbrellaSlime => 2,
            // Main.cs:67103 (entries 190-194): 2, 2, 2, 2, 2.
            Self::CataractEye => 2,
            Self::SleepyEye => 2,
            Self::DialatedEye => 2,
            Self::GreenEye => 2,
            Self::PurpleEye => 2,
            // Main.cs:67089 (entries 52/53): 3, 3.
            Self::DoctorBones => 3,
            Self::TheGroom => 3,
            // Main.cs:67097 (entry 132): 3.
            Self::BaldZombie => 3,
            // Main.cs:67100 (entry 161): 3.
            Self::ZombieEskimo => 3,
            // Main.cs:67102 (entries 186-189): 3, 3, 3, 3.
            Self::PincushionZombie => 3,
            Self::SlimedZombie => 3,
            Self::SwampZombie => 3,
            Self::TwiggyZombie => 3,
            // Main.cs:67104 (entry 200): 3.
            Self::FemaleZombie => 3,
            // Main.cs:67106 (entry 223): 3.
            Self::ZombieRaincoat => 3,
        }
    }
}

/// The `NpcKind` → [`FindFrameKind`] mapping (the S1-D-3ff wiring): the
/// 24 wave-A kinds map 1:1, every other kind is `None` (its arm stays in
/// `npc.rs::find_frame`).
pub fn find_frame_kind(kind: crate::npc::NpcKind) -> Option<FindFrameKind> {
    use crate::npc::NpcKind as K;
    Some(match kind {
        K::MotherSlime => FindFrameKind::MotherSlime,
        K::LavaSlime => FindFrameKind::LavaSlime,
        K::DungeonSlime => FindFrameKind::DungeonSlime,
        K::CorruptSlime => FindFrameKind::CorruptSlime,
        K::IlluminantSlime => FindFrameKind::IlluminantSlime,
        K::IceSlime => FindFrameKind::IceSlime,
        K::Crimslime => FindFrameKind::Crimslime,
        K::SpikedIceSlime => FindFrameKind::SpikedIceSlime,
        K::UmbrellaSlime => FindFrameKind::UmbrellaSlime,
        K::CataractEye => FindFrameKind::CataractEye,
        K::SleepyEye => FindFrameKind::SleepyEye,
        K::DialatedEye => FindFrameKind::DialatedEye,
        K::GreenEye => FindFrameKind::GreenEye,
        K::PurpleEye => FindFrameKind::PurpleEye,
        K::DoctorBones => FindFrameKind::DoctorBones,
        K::TheGroom => FindFrameKind::TheGroom,
        K::BaldZombie => FindFrameKind::BaldZombie,
        K::ZombieEskimo => FindFrameKind::ZombieEskimo,
        K::PincushionZombie => FindFrameKind::PincushionZombie,
        K::SlimedZombie => FindFrameKind::SlimedZombie,
        K::SwampZombie => FindFrameKind::SwampZombie,
        K::TwiggyZombie => FindFrameKind::TwiggyZombie,
        K::FemaleZombie => FindFrameKind::FemaleZombie,
        K::ZombieRaincoat => FindFrameKind::ZombieRaincoat,
        _ => return None,
    })
}

/// The `NPC` fields `FindFrame` reads/writes for these arms:
/// `frame.Y / num` (the row), `frameCounter`, `spriteDirection`,
/// `rotation`. The wiring step copies them to/from `Npc`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FrameState {
    /// `frame.Y / num` — the sheet row.
    pub row: u32,
    /// `NPC.frameCounter`.
    pub counter: f32,
    /// `NPC.spriteDirection` (±1).
    pub sprite_dir: i32,
    /// `NPC.rotation` — only the eye group writes it (the atan2 aim).
    pub rotation: f32,
}

/// The per-tick inputs the arms read: `velocity`, `direction` and
/// `aiAction`. Unlike wave B, the slime group DOES read the `num2`
/// motion class (`NPC.cs:66696-66704`), so `ai_action` is carried; the
/// eye/walker arms never read it (nor `ai`/`wet`/`localAI` — none of
/// this wave's kinds touches those in `FindFrame`).
#[derive(Clone, Copy, Debug)]
pub struct FrameInput {
    pub vx: f32,
    pub vy: f32,
    /// `Entity.direction` (±1; 0 leaves `spriteDirection` alone in the
    /// walker arm, which gates on `direction == 1` / `== -1`).
    pub direction: i32,
    /// `NPC.aiAction` — 0 the motion class, 1 the attack (`num2 = 4`).
    pub ai_action: i32,
}

impl Default for FrameInput {
    fn default() -> Self {
        Self { vx: 0.0, vy: 0.0, direction: 1, ai_action: 0 }
    }
}

/// `NPC.FindFrame` (`NPC.cs:66684`) for the wave-A kinds — one tick.
pub fn find_frame_a(kind: FindFrameKind, s: &mut FrameState, i: &FrameInput) {
    let count = kind.frame_count();
    // The `num2` motion class (`NPC.cs:66696-66704`): aiAction 0 maps
    // the velocity (rising 2, falling 3, moving 1, idle 0), aiAction 1
    // is the attack class 4; any other aiAction leaves it 0. Only the
    // slime group consumes it.
    let mut num2 = 0;
    if i.ai_action == 0 {
        num2 = if i.vy < 0.0 {
            2
        } else if i.vy > 0.0 {
            3
        } else if i.vx != 0.0 {
            1
        } else {
            0
        };
    } else if i.ai_action == 1 {
        num2 = 4;
    }
    match kind {
        // The slime group (`NPC.cs:70801-70846`): the 8-tick hop strip
        // over the 2-row sheet, doubled while moving/airborne and
        // tripled while attacking. The `type == 302 || type == 304`
        // spriteDirection snap (`:70824-70827`) belongs to the town
        // slimes — not this wave's kinds — so no flip/rotation write.
        FindFrameKind::MotherSlime
        | FindFrameKind::LavaSlime
        | FindFrameKind::DungeonSlime
        | FindFrameKind::CorruptSlime
        | FindFrameKind::IlluminantSlime
        | FindFrameKind::IceSlime
        | FindFrameKind::Crimslime
        | FindFrameKind::SpikedIceSlime
        | FindFrameKind::UmbrellaSlime => {
            s.counter += 1.0;
            if num2 > 0 {
                s.counter += 1.0;
            }
            if num2 == 4 {
                s.counter += 1.0;
            }
            if s.counter >= 8.0 {
                s.row += 1;
                s.counter = 0.0;
            }
            if s.row >= count {
                s.row = 0;
            }
        }
        // The eye group (`NPC.cs:73994-74059`): the five variants ride
        // the FIRST aim branch (`:74019-74031` — the type list carries
        // 190-194): `spriteDirection` mirrors the flight sign and
        // `rotation = atan2(vy, vx)`, `+ 3.14f` (the literal, not PI)
        // when moving left; both freeze while `vx == 0`. The 8-tick
        // strip (`num142` = 8; the 6 is type 660's, `:74046-74049`)
        // cycles the 2 rows. The 23 dust spawn and the 121/660
        // `localAI[0]` init are other kinds'; the second flip branch
        // (`:74032-74043`) is unreachable for these five.
        FindFrameKind::CataractEye
        | FindFrameKind::SleepyEye
        | FindFrameKind::DialatedEye
        | FindFrameKind::GreenEye
        | FindFrameKind::PurpleEye => {
            if i.vx > 0.0 {
                s.sprite_dir = 1;
                s.rotation = i.vy.atan2(i.vx);
            }
            if i.vx < 0.0 {
                s.sprite_dir = -1;
                // The original adds the literal `3.14f`, not PI.
                s.rotation = i.vy.atan2(i.vx) + 3.14;
            }
            s.counter += 1.0;
            if s.counter >= 8.0 {
                s.row += 1;
                s.counter = 0.0;
            }
            if s.row >= count {
                s.row = 0;
            }
        }
        // The walker group (`NPC.cs:76324-76391`): the plain 3-row
        // stand/walk strip shared with the base Zombie (`npc.rs`'s
        // `walker_frame` ports the same arm for type 3).
        FindFrameKind::DoctorBones
        | FindFrameKind::TheGroom
        | FindFrameKind::BaldZombie
        | FindFrameKind::ZombieEskimo
        | FindFrameKind::PincushionZombie
        | FindFrameKind::SlimedZombie
        | FindFrameKind::SwampZombie
        | FindFrameKind::TwiggyZombie
        | FindFrameKind::FemaleZombie
        | FindFrameKind::ZombieRaincoat => {
            zombie_walk_frame(s, i);
        }
    }
}

/// The plain walker arm (`NPC.cs:76347-76390`): the grounded flip
/// (`:76347-76357` — only `direction` ±1 writes), the air/turn pin to
/// row 2 (`:76358-76363`), the stand row 0 (`:76364-76369`), and the
/// `|vx|` accumulator's 8/16/24/32 bands stepping rows 0/1/2/1
/// (`:76370-76390`). In the final `else` the C# only zeroes the counter
/// and leaves `frame.Y` alone (the lap holds row 1) — kept verbatim,
/// the `npc.rs::walker_frame` precedent.
fn zombie_walk_frame(s: &mut FrameState, i: &FrameInput) {
    if i.vy == 0.0 {
        if i.direction == 1 {
            s.sprite_dir = 1;
        }
        if i.direction == -1 {
            s.sprite_dir = -1;
        }
    }
    if i.vy != 0.0 || (i.direction == -1 && i.vx > 0.0) || (i.direction == 1 && i.vx < 0.0) {
        s.counter = 0.0;
        s.row = 2;
    } else if i.vx == 0.0 {
        s.counter = 0.0;
        s.row = 0;
    } else {
        s.counter += i.vx.abs();
        if s.counter < 8.0 {
            s.row = 0;
        } else if s.counter < 16.0 {
            s.row = 1;
        } else if s.counter < 24.0 {
            s.row = 2;
        } else if s.counter < 32.0 {
            s.row = 1;
        } else {
            // The C# leaves `frame.Y` alone here (`:76387-76390`).
            s.counter = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(vx: f32, vy: f32) -> FrameInput {
        FrameInput { vx, vy, ..Default::default() }
    }

    fn rows_over(kind: FindFrameKind, i: FrameInput, ticks: usize) -> Vec<u32> {
        let mut s = FrameState::default();
        let mut out = Vec::new();
        for _ in 0..ticks {
            find_frame_a(kind, &mut s, &i);
            out.push(s.row);
        }
        out
    }

    /// The 24 `Main.npcFrameCount` rows, counted entry-by-entry out of
    /// the `Main.cs:67082` initializer (the line/slot arithmetic in
    /// `frame_count`'s doc), cross-checked against the port's existing
    /// anchors (slime 1 = 2, demon eye 2 = 2, zombie 3 = 3).
    #[test]
    fn the_wave_a_frame_counts_match_the_main_initializer() {
        let table = [
            (FindFrameKind::MotherSlime, 16, 2),
            (FindFrameKind::LavaSlime, 59, 2),
            (FindFrameKind::DungeonSlime, 71, 2),
            (FindFrameKind::CorruptSlime, 81, 2),
            (FindFrameKind::IlluminantSlime, 138, 2),
            (FindFrameKind::IceSlime, 147, 2),
            (FindFrameKind::Crimslime, 183, 2),
            (FindFrameKind::SpikedIceSlime, 184, 2),
            (FindFrameKind::UmbrellaSlime, 225, 2),
            (FindFrameKind::CataractEye, 190, 2),
            (FindFrameKind::SleepyEye, 191, 2),
            (FindFrameKind::DialatedEye, 192, 2),
            (FindFrameKind::GreenEye, 193, 2),
            (FindFrameKind::PurpleEye, 194, 2),
            (FindFrameKind::DoctorBones, 52, 3),
            (FindFrameKind::TheGroom, 53, 3),
            (FindFrameKind::BaldZombie, 132, 3),
            (FindFrameKind::ZombieEskimo, 161, 3),
            (FindFrameKind::PincushionZombie, 186, 3),
            (FindFrameKind::SlimedZombie, 187, 3),
            (FindFrameKind::SwampZombie, 188, 3),
            (FindFrameKind::TwiggyZombie, 189, 3),
            (FindFrameKind::FemaleZombie, 200, 3),
            (FindFrameKind::ZombieRaincoat, 223, 3),
        ];
        for (kind, vanilla, count) in table {
            assert_eq!(kind.vanilla_type(), vanilla, "{kind:?} id");
            assert_eq!(kind.frame_count(), count, "{kind:?} npcFrameCount");
        }
    }

    /// The idle slime hop (`NPC.cs:70828-70845`): +1 a tick, the row
    /// steps at >= 8 and wraps at the 2-row sheet — 8 ticks a row.
    #[test]
    fn the_idle_slime_hops_every_eight_ticks() {
        for kind in [
            FindFrameKind::MotherSlime,
            FindFrameKind::LavaSlime,
            FindFrameKind::DungeonSlime,
            FindFrameKind::CorruptSlime,
            FindFrameKind::IlluminantSlime,
            FindFrameKind::IceSlime,
            FindFrameKind::Crimslime,
            FindFrameKind::SpikedIceSlime,
            FindFrameKind::UmbrellaSlime,
        ] {
            let rows = rows_over(kind, input(0.0, 0.0), 16);
            assert_eq!(
                rows,
                [0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0],
                "{kind:?} idle hop"
            );
        }
    }

    /// The airborne/moving slime doubles the clock (`NPC.cs:70829-70832`
    /// — `num2 > 0` adds a second tick): rising (num2 = 2), falling
    /// (num2 = 3) and grounded-moving (num2 = 1) all step every 4 ticks.
    #[test]
    fn the_moving_slime_doubles_its_frame_clock() {
        for i in [input(0.0, -2.0), input(0.0, 2.0), input(1.0, 0.0)] {
            let rows = rows_over(FindFrameKind::MotherSlime, i, 8);
            assert_eq!(rows, [0, 0, 0, 1, 1, 1, 1, 0], "vx {} vy {}", i.vx, i.vy);
        }
    }

    /// The attacking slime triples the clock (`NPC.cs:70833-70836` —
    /// `aiAction == 1` gives num2 = 4, which adds BOTH extra ticks):
    /// +3 a tick, the step at 9 zeroes the counter (no remainder kept).
    #[test]
    fn the_attacking_slime_triples_its_frame_clock() {
        let i = FrameInput { ai_action: 1, ..input(0.0, 0.0) };
        let mut s = FrameState::default();
        let mut rows = Vec::new();
        for _ in 0..6 {
            find_frame_a(FindFrameKind::DungeonSlime, &mut s, &i);
            rows.push(s.row);
        }
        assert_eq!(rows, [0, 0, 1, 1, 1, 0]);
        assert_eq!(s.counter, 0.0);
    }

    /// The slime arm writes neither `spriteDirection` nor `rotation`
    /// (`NPC.cs:70824-70827`'s snap is types 302/304 only).
    #[test]
    fn the_slime_arm_leaves_the_flip_and_rotation_alone() {
        let mut s = FrameState { sprite_dir: -1, rotation: 0.5, ..Default::default() };
        find_frame_a(FindFrameKind::UmbrellaSlime, &mut s, &input(3.0, -1.0));
        assert_eq!(s.sprite_dir, -1);
        assert_eq!(s.rotation, 0.5);
    }

    /// The eye aim (`NPC.cs:74019-74031`): vx > 0 faces +1 with
    /// `atan2(vy, vx)`; vx < 0 faces -1 with `atan2 + 3.14f` (the
    /// literal); vx == 0 freezes BOTH the flip and the rotation.
    #[test]
    fn the_eye_aim_mirrors_and_freezes_like_the_csharp() {
        for kind in [
            FindFrameKind::CataractEye,
            FindFrameKind::SleepyEye,
            FindFrameKind::DialatedEye,
            FindFrameKind::GreenEye,
            FindFrameKind::PurpleEye,
        ] {
            let mut s = FrameState { sprite_dir: -1, rotation: 0.5, ..Default::default() };
            find_frame_a(kind, &mut s, &input(0.0, 2.0));
            assert_eq!((s.sprite_dir, s.rotation), (-1, 0.5), "{kind:?} freeze");
            find_frame_a(kind, &mut s, &input(1.0, 1.0));
            assert_eq!(s.sprite_dir, 1, "{kind:?} right flip");
            assert!((s.rotation - std::f32::consts::FRAC_PI_4).abs() < 1e-6);
            find_frame_a(kind, &mut s, &input(-1.0, 0.0));
            assert_eq!(s.sprite_dir, -1, "{kind:?} left flip");
            assert!(
                (s.rotation - (std::f32::consts::PI + 3.14)).abs() < 1e-5,
                "{kind:?} the 3.14f literal, not PI"
            );
        }
    }

    /// The eye strip (`NPC.cs:74044-74058`): +1 a tick regardless of
    /// motion (no num2 read), the row steps at >= 8 over the 2 rows.
    #[test]
    fn the_eye_strip_cycles_every_eight_ticks() {
        let rows = rows_over(FindFrameKind::CataractEye, input(2.0, -1.0), 16);
        assert_eq!(rows, [0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0]);
    }

    /// The walker stand (`NPC.cs:76364-76369`): vx == 0 on the ground
    /// pins row 0 and zeroes the counter; the grounded flip only writes
    /// on `direction` ±1 (`:76349-76356`).
    #[test]
    fn the_walker_stands_on_row_zero_and_flips_only_on_pm_one() {
        let mut s = FrameState { row: 2, counter: 12.0, sprite_dir: -1, ..Default::default() };
        find_frame_a(FindFrameKind::TheGroom, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter, s.sprite_dir), (0, 0.0, 1));
        let mut s = FrameState { sprite_dir: -1, ..Default::default() };
        let i = FrameInput { direction: 0, ..input(0.0, 0.0) };
        find_frame_a(FindFrameKind::TheGroom, &mut s, &i);
        assert_eq!(s.sprite_dir, -1, "direction 0 leaves the flip alone");
    }

    /// The air/turn pin (`NPC.cs:76358-76363`): airborne OR facing
    /// against the velocity pins row 2 and zeroes the counter; the
    /// airborne tick never writes `spriteDirection`.
    #[test]
    fn the_walker_pins_row_two_in_the_air_and_while_turning() {
        let mut s = FrameState { sprite_dir: 1, ..Default::default() };
        let i = FrameInput { direction: -1, ..input(0.0, 2.0) };
        find_frame_a(FindFrameKind::ZombieRaincoat, &mut s, &i);
        assert_eq!((s.row, s.counter, s.sprite_dir), (2, 0.0, 1));
        // Turning in place: direction -1 while moving right.
        let i = FrameInput { direction: -1, ..input(0.5, 0.0) };
        find_frame_a(FindFrameKind::ZombieRaincoat, &mut s, &i);
        assert_eq!((s.row, s.sprite_dir), (2, -1));
        // And the mirror: direction 1 while moving left.
        let i = FrameInput { direction: 1, ..input(-0.5, 0.0) };
        find_frame_a(FindFrameKind::DoctorBones, &mut s, &i);
        assert_eq!(s.row, 2);
    }

    /// The walk bands (`NPC.cs:76370-76390`): at vx 2 the counter laps
    /// 2..32 — rows 0 (2/4/6), 1 (8..14), 2 (16..22), 1 (24..30), and
    /// the 32-tick only zeroes the counter, HOLDING row 1 (the vanilla
    /// no-write quirk); the next lap restarts at row 0.
    #[test]
    fn the_walker_bands_lap_and_hold_row_one_at_the_reset() {
        let mut s = FrameState::default();
        let i = input(2.0, 0.0);
        let mut rows = Vec::new();
        for _ in 0..16 {
            find_frame_a(FindFrameKind::BaldZombie, &mut s, &i);
            rows.push(s.row);
        }
        assert_eq!(rows, [0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 1, 1, 1, 1, 1]);
        assert_eq!(s.counter, 0.0, "the 32-tick zeroes the counter");
        assert_eq!(s.row, 1, "and leaves the row alone");
        find_frame_a(FindFrameKind::BaldZombie, &mut s, &i);
        assert_eq!((s.row, s.counter), (0, 2.0), "the next lap restarts");
    }

    /// Every walker shares the one arm — pin the band entry row across
    /// all ten kinds (`NPC.cs:76324-76346`'s case list).
    #[test]
    fn all_ten_walkers_share_the_zombie_arm() {
        for kind in [
            FindFrameKind::DoctorBones,
            FindFrameKind::TheGroom,
            FindFrameKind::BaldZombie,
            FindFrameKind::ZombieEskimo,
            FindFrameKind::PincushionZombie,
            FindFrameKind::SlimedZombie,
            FindFrameKind::SwampZombie,
            FindFrameKind::TwiggyZombie,
            FindFrameKind::FemaleZombie,
            FindFrameKind::ZombieRaincoat,
        ] {
            let rows = rows_over(kind, input(1.0, 0.0), 16);
            // At vx 1: rows 0 for counters 1..7, 1 for 8..15, and the
            // 16th tick lands in the 16..23 band → row 2.
            assert_eq!(
                rows,
                [0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 2],
                "{kind:?} walk bands"
            );
        }
    }
}
