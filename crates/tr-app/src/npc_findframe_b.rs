//! S1-D-3 FindFrame coverage wave B — the MID/HIGH-id kinds of
//! `NPC.FindFrame` (`NPC.cs:66684-77370`) that the port's `npc.rs`
//! `find_frame` match does not carry yet: the dungeon skeletons, the
//! mummies, the hallowed/corrupt surface family (pixie/unicorn/gastropod),
//! the mimics, the wolf, the jungle hopper pair and the hardmode dungeon
//! casters' bones.
//!
//! Wave split (2026-11-18): wave A (`task/S1-D-3-findframe-a`) takes the
//! LOW-id kinds (slimes/zombies/eyes/worms); the frost legion 338-352 ride
//! `task/S1-D-3-frost-legion`, the pumpkin family 305-330 landed with
//! S1-F-13b (the Poltergeist 330 in `df4d98aa`), and the bosses with
//! in-flight AI machines keep their own `find_frame` arms. This module
//! owns the 25 kinds listed in [`FindFrameKind::vanilla_type`].
//!
//! Shape: the kinds are NOT in `NpcKind` yet (their AI/spawn machines are
//! later D-3 rounds), so — like `tr-core/src/drawdata_anim.rs` (S1-F-9b) —
//! the frame machines land first as pure functions over a small
//! [`FrameState`]/[`FrameInput`] pair and the `find_frame` wiring is a
//! separate step: when a kind joins `NpcKind`, its arm calls
//! [`find_frame_b`] with the NPC's `vx`/`vy`/`facing`/`wet`/`ai` and copies
//! `row`/`counter`/`sprite_dir`/`rotation` back. No RNG is touched (none of
//! these arms draws `Main.rand`), so the wiring cannot move a stream.
//!
//! Row convention: the port tracks frame ROW indices, not pixels — the
//! C#'s `num` (the sheet height / `Main.npcFrameCount[type]`,
//! `NPC.cs:66694`) cancels out of every comparison, exactly as in
//! `npc.rs::find_frame`'s existing arms. The per-kind row counts are
//! `Main.npcFrameCount` (`Main.cs:67082`'s 697-entry initializer), counted
//! entry-by-entry in [`FindFrameKind::frame_count`].
//!
//! Deliberate skips (recorded in docs/ANALYSIS.md "S1-D-3 FindFrame wave
//! B"): the `IsABestiaryIconDummy` branches (the port has no bestiary
//! dummy), the humanoid arm's town `ai[0]` pose zones (`NPC.cs:68662-69488`
//! — unreachable for these aiStyle-3 fighters, whose `AI_003_Fighters`
//! turn counter only ever leaves `ai[0]` at 0 or 1, `NPC.cs:57381-57397`),
//! the disabled profile-texture block (`NPC.cs:68637`, `if (false && …)`),
//! and the dust spawns of the case-48 group's OTHER members (60/634 —
//! not this wave's kinds).

// The `find_frame` wiring is a later wave (the kinds are not in
// `NpcKind` yet) — the `ammo.rs`/`drawdata_anim.rs` precedent.
#![allow(dead_code)]

/// The 25 wave-B kinds, keyed by their VANILLA `NPC.type`
/// (`ID/NPCID.cs`'s `public const short` rows — the port-local `NpcKind`
/// ids do not exist for them yet).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FindFrameKind {
    /// `NPCID.AngryBones` = 31 — the humanoid-group arm.
    AngryBones,
    /// `NPCID.CursedSkull` = 34 — `case 34`.
    CursedSkull,
    /// `NPCID.Pixie` = 75 — `case 75`.
    Pixie,
    /// `NPCID.ArmoredSkeleton` = 77 — the humanoid-group arm.
    ArmoredSkeleton,
    /// `NPCID.Mummy` = 78 — the humanoid-group arm.
    Mummy,
    /// `NPCID.DarkMummy` = 79 — the humanoid-group arm.
    DarkMummy,
    /// `NPCID.LightMummy` = 80 — the humanoid-group arm.
    LightMummy,
    /// `NPCID.Wraith` = 82 — the `case 48` floater group.
    Wraith,
    /// `NPCID.Mimic` = 85 — the `case 85` chest group.
    Mimic,
    /// `NPCID.Unicorn` = 86 — `case 86`.
    Unicorn,
    /// `NPCID.Werewolf` = 104 — the humanoid-group arm.
    Werewolf,
    /// `NPCID.Clown` = 109 — `case 109`.
    Clown,
    /// `NPCID.SkeletonArcher` = 110 — the `case 110` archer group.
    SkeletonArcher,
    /// `NPCID.Gastropod` = 122 — `case 122`.
    Gastropod,
    /// `NPCID.PossessedArmor` = 140 — the humanoid-group arm (the
    /// idle-row-1 quirk).
    PossessedArmor,
    /// `NPCID.Wolf` = 155 — `case 155`.
    Wolf,
    /// `NPCID.UndeadViking` = 167 — the humanoid-group arm.
    UndeadViking,
    /// `NPCID.Herpling` = 174 — `case 174`.
    Herpling,
    /// `NPCID.Derpling` = 177 — `case 177`.
    Derpling,
    /// `NPCID.ArmoredViking` = 197 — the humanoid-group arm.
    ArmoredViking,
    /// `NPCID.IcyMerman` = 206 — `case 206` (the archer shape).
    IcyMerman,
    /// `NPCID.GiantCursedSkull` = 289 — `case 289`.
    GiantCursedSkull,
    /// `NPCID.AngryBonesBig` = 294 — the humanoid-group arm.
    AngryBonesBig,
    /// `NPCID.AngryBonesBigHelmet` = 296 — the humanoid-group arm.
    AngryBonesBigHelmet,
    /// `NPCID.BloodMummy` = 630 — the humanoid-group arm.
    BloodMummy,
}

impl FindFrameKind {
    /// The vanilla `NPC.type` (`ID/NPCID.cs`'s const rows).
    pub fn vanilla_type(self) -> i16 {
        match self {
            Self::AngryBones => 31,
            Self::CursedSkull => 34,
            Self::Pixie => 75,
            Self::ArmoredSkeleton => 77,
            Self::Mummy => 78,
            Self::DarkMummy => 79,
            Self::LightMummy => 80,
            Self::Wraith => 82,
            Self::Mimic => 85,
            Self::Unicorn => 86,
            Self::Werewolf => 104,
            Self::Clown => 109,
            Self::SkeletonArcher => 110,
            Self::Gastropod => 122,
            Self::PossessedArmor => 140,
            Self::Wolf => 155,
            Self::UndeadViking => 167,
            Self::Herpling => 174,
            Self::Derpling => 177,
            Self::ArmoredViking => 197,
            Self::IcyMerman => 206,
            Self::GiantCursedSkull => 289,
            Self::AngryBonesBig => 294,
            Self::AngryBonesBigHelmet => 296,
            Self::BloodMummy => 630,
        }
    }

    /// `Main.npcFrameCount[type]` — the row counts counted entry-by-entry
    /// out of the 697-entry initializer (`Main.cs:67082-67153`; entry
    /// `type` sits on line `67084 + type / 10`, slot `type % 10`).
    pub fn frame_count(self) -> u32 {
        match self {
            // Main.cs:67087 (entries 30-39): 31 = 15, 34 = 3.
            Self::AngryBones => 15,
            Self::CursedSkull => 3,
            // Main.cs:67091 (entries 70-79): 75 = 4, 77 = 15, 78/79 = 16.
            Self::Pixie => 4,
            Self::ArmoredSkeleton => 15,
            Self::Mummy => 16,
            Self::DarkMummy => 16,
            // Main.cs:67092 (entries 80-89): 80 = 16, 82 = 4, 85 = 18,
            // 86 = 16.
            Self::LightMummy => 16,
            Self::Wraith => 4,
            Self::Mimic => 18,
            Self::Unicorn => 16,
            // Main.cs:67094 (entries 100-109): 104 = 16, 109 = 12.
            Self::Werewolf => 16,
            Self::Clown => 12,
            // Main.cs:67095 (entry 110): 20.
            Self::SkeletonArcher => 20,
            // Main.cs:67096 (entry 122): 11.
            Self::Gastropod => 11,
            // Main.cs:67098 (entry 140): 15.
            Self::PossessedArmor => 15,
            // Main.cs:67099 (entry 155): 13.
            Self::Wolf => 13,
            // Main.cs:67100 (entry 167): 15.
            Self::UndeadViking => 15,
            // Main.cs:67101 (entries 170-179): 174 = 5, 177 = 3.
            Self::Herpling => 5,
            Self::Derpling => 3,
            // Main.cs:67103 (entry 197): 15.
            Self::ArmoredViking => 15,
            // Main.cs:67104 (entry 206): 20.
            Self::IcyMerman => 20,
            // Main.cs:67112 (entry 289): 6.
            Self::GiantCursedSkull => 6,
            // Main.cs:67113 (entries 290-299): 294 = 16, 296 = 15.
            Self::AngryBonesBig => 16,
            Self::AngryBonesBigHelmet => 15,
            // Main.cs:67147 (entry 630): 16.
            Self::BloodMummy => 16,
        }
    }
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
    /// `NPC.rotation` — only the arms that write it (75/82/122) touch it.
    pub rotation: f32,
}

/// The per-tick inputs the arms read: `velocity`, `direction`, `wet` and
/// the `ai` slots. `ai_action` is not carried — none of these kinds sets
/// it (the `num2` motion class of `NPC.cs:66696-66704` is unused by every
/// wave-B arm).
#[derive(Clone, Copy, Debug)]
pub struct FrameInput {
    pub vx: f32,
    pub vy: f32,
    /// `Entity.direction` (±1; 0 leaves `spriteDirection` alone in the
    /// arms that gate on `direction == 1` / `== -1`).
    pub direction: i32,
    pub wet: bool,
    pub ai: [f32; 4],
}

impl Default for FrameInput {
    fn default() -> Self {
        Self { vx: 0.0, vy: 0.0, direction: 1, wet: false, ai: [0.0; 4] }
    }
}

/// `NPC.FindFrame` (`NPC.cs:66684`) for the wave-B kinds — one tick.
pub fn find_frame_b(kind: FindFrameKind, s: &mut FrameState, i: &FrameInput) {
    let count = kind.frame_count();
    match kind {
        // The humanoid-group case list (`NPC.cs:68543-68634` carries
        // 31/77/78/79/80/104/140/167/197/294/296/630) → the shared
        // fighter walk (`NPC.cs:68647-69655`). None is `isLikeATownNPC`
        // (`NPC.cs:6856-6866` — only 453 + the town flag), so `num57` = 0
        // and the walk wraps at the FULL `npcFrameCount`; none is a town
        // slime, so `num108` = 6 and `num109` = row 2 for all of them.
        FindFrameKind::AngryBones
        | FindFrameKind::ArmoredSkeleton
        | FindFrameKind::Mummy
        | FindFrameKind::DarkMummy
        | FindFrameKind::LightMummy
        | FindFrameKind::Werewolf
        | FindFrameKind::PossessedArmor
        | FindFrameKind::UndeadViking
        | FindFrameKind::ArmoredViking
        | FindFrameKind::AngryBonesBig
        | FindFrameKind::AngryBonesBigHelmet
        | FindFrameKind::BloodMummy => {
            // The airborne pin list (`NPC.cs:69647`): 31/77/140/167/197/
            // 294/296 snap to row 0 in the air; the mummies (78/79/80/
            // 630) and the werewolf (104) keep the group's row 1.
            let air_row0 = matches!(
                kind,
                FindFrameKind::AngryBones
                    | FindFrameKind::ArmoredSkeleton
                    | FindFrameKind::PossessedArmor
                    | FindFrameKind::UndeadViking
                    | FindFrameKind::ArmoredViking
                    | FindFrameKind::AngryBonesBig
                    | FindFrameKind::AngryBonesBigHelmet
            );
            // The idle quirk (`NPC.cs:69510-69514`): the PossessedArmor
            // (140) stands on row 1, not row 0.
            let idle_row1 = kind == FindFrameKind::PossessedArmor;
            humanoid_fighter_frame(s, i, count, air_row0, idle_row1);
        }
        // `case 34` (`NPC.cs:77055-77066`): the Cursed Skull's plain
        // 4-tick strip over its 3 rows — no direction/rotation write.
        FindFrameKind::CursedSkull => {
            s.counter += 1.0;
            if s.counter >= 4.0 {
                s.row += 1;
                s.counter = 0.0;
            }
            if s.row >= count {
                s.row = 0;
            }
        }
        // `case 75` (`NPC.cs:74113-74133`): the Pixie faces by VELOCITY
        // (vx > 0 → 1, else -1 — even at vx == 0), tilts `vx * 0.1`, and
        // runs the 4-tick strip over its 4 rows.
        FindFrameKind::Pixie => {
            s.sprite_dir = if i.vx > 0.0 { 1 } else { -1 };
            s.rotation = i.vx * 0.1;
            s.counter += 1.0;
            if s.counter >= 4.0 {
                s.row += 1;
                s.counter = 0.0;
            }
            if s.row >= count {
                s.row = 0;
            }
        }
        // `case 82` in the floater group (`NPC.cs:74823-74884`, shared
        // with 48/49/51/60/93/137/182/210/211/253/316/634/662/664): the
        // Wraith faces by velocity sign (0 leaves the flip alone), tilts
        // `vx * 0.1`, and steps every 6 ticks (`num291` = 6 — the 5 is
        // type 48's) over the full 4 rows (`num292` — the `--` list
        // 49/51/60/634 does not carry 82). The group's dust spawns
        // (60/634) and the 210/211 double-increment are other kinds'.
        FindFrameKind::Wraith => {
            if i.vx > 0.0 {
                s.sprite_dir = 1;
            }
            if i.vx < 0.0 {
                s.sprite_dir = -1;
            }
            s.rotation = i.vx * 0.1;
            s.counter += 1.0;
            if s.counter >= 6.0 {
                s.row += 1;
                s.counter = 0.0;
            }
            if s.row >= count {
                s.row = 0;
            }
        }
        // `case 85` (`NPC.cs:73443-73515`, shared with the mimics 341/
        // 629): the closed chest (`ai[0] == 0`) pins row 0; the open
        // chest's counter walks DOWN on the ground and UP in the air,
        // clamped to 0..12 (`num189 * 4`), and the bands pick rows
        // 1/2/3/4/5/4/3/2. The clamp makes the `>= num189 * 8` reset
        // (`:73501-73504`) unreachable — a vanilla quirk kept verbatim.
        // `ai[3]` selects the biome bank: 2 → +6 rows, 3 → +12 (the
        // 18-row sheet is three 6-row banks).
        FindFrameKind::Mimic => {
            if i.ai[0] == 0.0 {
                s.counter = 0.0;
                s.row = 0;
            } else {
                let num189 = 3.0f32;
                if i.vy == 0.0 {
                    s.counter -= 1.0;
                } else {
                    s.counter += 1.0;
                }
                if s.counter < 0.0 {
                    s.counter = 0.0;
                }
                if s.counter > num189 * 4.0 {
                    s.counter = num189 * 4.0;
                }
                if s.counter < num189 {
                    s.row = 1;
                } else if s.counter < num189 * 2.0 {
                    s.row = 2;
                } else if s.counter < num189 * 3.0 {
                    s.row = 3;
                } else if s.counter < num189 * 4.0 {
                    s.row = 4;
                } else if s.counter < num189 * 5.0 {
                    s.row = 5;
                } else if s.counter < num189 * 6.0 {
                    s.row = 4;
                } else if s.counter < num189 * 7.0 {
                    s.row = 3;
                } else {
                    s.row = 2;
                    if s.counter >= num189 * 8.0 {
                        s.counter = num189;
                    }
                }
            }
            if i.ai[3] == 2.0 {
                s.row += 6;
            } else if i.ai[3] == 3.0 {
                s.row += 12;
            }
        }
        // `case 86` (`NPC.cs:74404-74464`): the Unicorn's two gaits. The
        // flip rides the ground (or wet) with the ±2 velocity deadband;
        // the air pins row 15, the stand row 0; the trot (`|vx| < 3`)
        // accumulates `|vx|` to 6 over rows 1..8, the gallop to 10 over
        // rows 9..14 — both wrap INSIDE their band, never to row 0.
        FindFrameKind::Unicorn => {
            if i.vy == 0.0 || i.wet {
                if i.vx < -2.0 {
                    s.sprite_dir = -1;
                } else if i.vx > 2.0 {
                    s.sprite_dir = 1;
                } else {
                    s.sprite_dir = i.direction;
                }
            }
            if i.vy != 0.0 {
                s.row = 15;
                s.counter = 0.0;
            } else if i.vx == 0.0 {
                s.counter = 0.0;
                s.row = 0;
            } else if i.vx.abs() < 3.0 {
                s.counter += i.vx.abs();
                if s.counter >= 6.0 {
                    s.counter = 0.0;
                    s.row += 1;
                    if s.row >= 9 {
                        s.row = 1;
                    }
                    if s.row == 0 {
                        s.row = 1;
                    }
                }
            } else {
                s.counter += i.vx.abs();
                if s.counter >= 10.0 {
                    s.counter = 0.0;
                    s.row += 1;
                    if s.row >= 15 {
                        s.row = 9;
                    }
                    if s.row <= 8 {
                        s.row = 9;
                    }
                }
            }
        }
        // `case 109` (`NPC.cs:74606-74621`): the Clown flips only while
        // grounded AND moving with the facing (the `vx <= 0 && dir < 0`
        // / `vx >= 0 && dir > 0` gate), and the `|vx|` accumulator keeps
        // its remainder (`-= 7`) across the 12-row wrap.
        FindFrameKind::Clown => {
            if i.vy == 0.0
                && ((i.vx <= 0.0 && i.direction < 0) || (i.vx >= 0.0 && i.direction > 0))
            {
                s.sprite_dir = i.direction;
            }
            s.counter += i.vx.abs();
            if s.counter >= 7.0 {
                s.counter -= 7.0;
                s.row += 1;
                if s.row >= count {
                    s.row = 0;
                }
            }
        }
        // `case 110` (`NPC.cs:76017-76059`, shared with 214/215/216/293)
        // and `case 206` (`NPC.cs:76060-76098`): two identical archer
        // arms — the shot pose pins row `(int)ai[2]` while grounded, the
        // walk climbs rows 6..count-1 at `|vx| * 3` (step at > 6), the
        // air holds row 0.
        FindFrameKind::SkeletonArcher | FindFrameKind::IcyMerman => {
            if i.vy == 0.0 {
                if i.direction == 1 {
                    s.sprite_dir = 1;
                }
                if i.direction == -1 {
                    s.sprite_dir = -1;
                }
                if i.ai[2] > 0.0 {
                    s.sprite_dir = i.direction;
                    s.row = i.ai[2] as u32;
                    s.counter = 0.0;
                } else {
                    if s.row < 6 {
                        s.row = 6;
                    }
                    s.counter += i.vx.abs() * 3.0;
                    if s.counter > 6.0 {
                        s.row += 1;
                        s.counter = 0.0;
                    }
                    if s.row >= count {
                        s.row = 6;
                    }
                }
            } else {
                s.counter = 0.0;
                s.row = 0;
            }
        }
        // `case 122` (`NPC.cs:73607-73627`): the Gastropod faces by
        // `direction`, tilts `vx * 0.05`; the laser charge (`ai[3] > 0`)
        // pins row `3 + (int)(ai[3] / 8)` (rows 3..10 of the 11-row
        // sheet) with the counter parked; the flight runs the 8-tick
        // strip over rows 0..2.
        FindFrameKind::Gastropod => {
            s.sprite_dir = i.direction;
            s.rotation = i.vx * 0.05;
            if i.ai[3] > 0.0 {
                let num129 = (i.ai[3] / 8.0) as i32;
                s.counter = 0.0;
                s.row = (num129 + 3) as u32;
            } else {
                s.counter += 1.0;
                if s.counter >= 8.0 {
                    s.row += 1;
                    s.counter = 0.0;
                }
                if s.row >= 3 {
                    s.row = 0;
                }
            }
        }
        // `case 155` (`NPC.cs:74311-74374`): the Wolf. The air pins rows
        // 10 (rising) / 11 (falling) WITHOUT touching the counter; the
        // stand row 0; the turn-in-place band (facing against a slow
        // `|vx| < 4` velocity) walks rows 0 → 1 → 2 at 6 ticks a row and
        // parks at 2; the run accumulates `|vx| * 0.4` (step at > 8,
        // remainder kept) over rows 3..9, and landing from the air rows
        // enters the run at row 12.
        FindFrameKind::Wolf => {
            let mut row = s.row;
            if i.vy < 0.0 {
                row = 10;
            } else if i.vy > 0.0 {
                row = 11;
            } else if i.vx == 0.0 {
                row = 0;
                s.counter = 0.0;
            } else if ((i.direction > 0 && i.vx < 0.0) || (i.direction < 0 && i.vx > 0.0))
                && i.vx.abs() < 4.0
            {
                s.sprite_dir = i.direction;
                if row > 2 {
                    row = 0;
                    s.counter = 0.0;
                }
                if row < 2 {
                    s.counter += 1.0;
                    if s.counter > 5.0 {
                        s.counter = 0.0;
                        row += 1;
                    }
                } else {
                    s.counter = 0.0;
                }
            } else {
                s.sprite_dir = if i.vx < 0.0 { -1 } else { 1 };
                s.counter += i.vx.abs() * 0.4;
                if row == 10 || row == 11 {
                    row = 12;
                    s.counter = 0.0;
                } else if s.counter > 8.0 {
                    s.counter -= 8.0;
                    row += 1;
                    if row > 9 {
                        row = 3;
                    }
                }
            }
            s.row = row;
        }
        // `case 174` (`NPC.cs:73172-73202`): the Herpling's hop poses —
        // grounded row 0, rising faster than 1.5 row 1, falling faster
        // than 1.5 row 3, the slow arc row 2. The ±1.5 comparisons ride
        // the C#'s `(double)` cast (`:73190`/`:73194`) — the exact
        // literals compare identically in f32. The bestiary-dummy strip
        // (`:73173-73185`) is not ported (no dummy in the port).
        FindFrameKind::Herpling => {
            if i.vy == 0.0 {
                s.row = 0;
            } else if i.vy < -1.5 {
                s.row = 1;
            } else if i.vy > 1.5 {
                s.row = 3;
            } else {
                s.row = 2;
            }
        }
        // `case 177` (`NPC.cs:73203-73226`): the Derpling's crouch — 10
        // grounded ticks on row 1, then row 0 with the counter PARKED at
        // 30 (every later grounded tick re-enters the `>= 10` branch, so
        // the crouch holds until it jumps); the rise keeps the counter,
        // the fall resets it; both air legs ride row 2.
        FindFrameKind::Derpling => {
            if i.vy == 0.0 {
                s.counter += 1.0;
                if s.counter >= 10.0 {
                    s.counter = 30.0;
                    s.row = 0;
                } else {
                    s.row = 1;
                }
            } else if i.vy < 0.0 {
                s.row = 2;
            } else {
                s.counter = 0.0;
                s.row = 2;
            }
        }
        // `case 289` (`NPC.cs:72652-72675`): the Giant Cursed Skull's
        // 8-tick cycle — rows 0 (counter 1..3), 1 (4..7), 2 (the 8th
        // tick, which resets). The C#'s follow-up `if (frameCounter >
        // 12)` (`:72666-72669`) is DEAD (the counter was just zeroed)
        // and is not carried — the `npc.rs` GolemHeadFree precedent.
        // `ai[3] == 1` (the spin attack) shifts the whole cycle +3 rows
        // onto the sheet's second bank (6 rows).
        FindFrameKind::GiantCursedSkull => {
            s.counter += 1.0;
            if s.counter < 4.0 {
                s.row = 0;
            } else if s.counter < 8.0 {
                s.row = 1;
            } else {
                s.row = 2;
                s.counter = 0.0;
            }
            if i.ai[3] == 1.0 {
                s.row += 3;
            }
        }
    }
}

/// W4-d44 wire — the wiring seam: the vanilla type → this module's
/// kind (the `find_frame_kind_extra6` in-module-lookup precedent).
/// The d38 residual pair ONLY (research/125 §14, the SEAL-RECEIPT
/// re-probed at this tip): 289 GiantCursedSkull is EXCLUDED (the E3
/// dedupe keeps the d38 inline arm — the dropped twin), the other
/// wave-B kinds await their roster lanes. `None` outside.
pub fn find_frame_kind_b(vanilla_type: i16) -> Option<FindFrameKind> {
    Some(match vanilla_type {
        294 => FindFrameKind::AngryBonesBig,
        296 => FindFrameKind::AngryBonesBigHelmet,
        _ => return None,
    })
}

/// The humanoid-group fighter walk (`NPC.cs:68647-69655`) for the
/// non-town members: the grounded flip (`:68649-68656` — only `direction`
/// ±1 writes), the idle snap (`:69489-69519` — row 0, or row 1 for the
/// PossessedArmor), the `|vx| * 2 + 1` accumulator stepping every > 6
/// over rows 2..count-1 (`:69521-69585` — `num108` = 6, `num109` = row 2,
/// `num57` = 0), and the airborne row 1 → pin-0 tail (`:69642-69650`).
/// The town `ai[0]` pose zones (`:68662-69488`) are unreachable for these
/// aiStyle-3 fighters (their turn counter leaves `ai[0]` at 0/1,
/// `NPC.cs:57381-57397`) and are not carried — the `humanoid_town_frame`
/// precedent in `npc.rs`.
pub(crate) fn humanoid_fighter_frame(
    s: &mut FrameState,
    i: &FrameInput,
    count: u32,
    air_row0: bool,
    idle_row1: bool,
) {
    if i.vy == 0.0 {
        if i.direction == 1 {
            s.sprite_dir = 1;
        }
        if i.direction == -1 {
            s.sprite_dir = -1;
        }
        if i.vx == 0.0 {
            // `NPC.cs:69510-69514`: type 140 idles on row 1.
            s.row = if idle_row1 { 1 } else { 0 };
            s.counter = 0.0;
        } else {
            if s.row < 2 {
                s.row = 2;
            }
            s.counter += i.vx.abs() * 2.0;
            s.counter += 1.0;
            if s.counter > 6.0 {
                s.row += 1;
                s.counter = 0.0;
            }
            if s.row >= count {
                s.row = 2;
            }
        }
    } else {
        // `NPC.cs:69642-69650`: the group's air row 1, then the pin list.
        s.counter = 0.0;
        s.row = if air_row0 { 0 } else { 1 };
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
            find_frame_b(kind, &mut s, &i);
            out.push(s.row);
        }
        out
    }

    /// The 25 `Main.npcFrameCount` rows, counted entry-by-entry out of
    /// the `Main.cs:67082` initializer (the line/slot arithmetic in
    /// `frame_count`'s doc).
    #[test]
    fn the_wave_b_frame_counts_match_the_main_initializer() {
        let table = [
            (FindFrameKind::AngryBones, 31, 15),
            (FindFrameKind::CursedSkull, 34, 3),
            (FindFrameKind::Pixie, 75, 4),
            (FindFrameKind::ArmoredSkeleton, 77, 15),
            (FindFrameKind::Mummy, 78, 16),
            (FindFrameKind::DarkMummy, 79, 16),
            (FindFrameKind::LightMummy, 80, 16),
            (FindFrameKind::Wraith, 82, 4),
            (FindFrameKind::Mimic, 85, 18),
            (FindFrameKind::Unicorn, 86, 16),
            (FindFrameKind::Werewolf, 104, 16),
            (FindFrameKind::Clown, 109, 12),
            (FindFrameKind::SkeletonArcher, 110, 20),
            (FindFrameKind::Gastropod, 122, 11),
            (FindFrameKind::PossessedArmor, 140, 15),
            (FindFrameKind::Wolf, 155, 13),
            (FindFrameKind::UndeadViking, 167, 15),
            (FindFrameKind::Herpling, 174, 5),
            (FindFrameKind::Derpling, 177, 3),
            (FindFrameKind::ArmoredViking, 197, 15),
            (FindFrameKind::IcyMerman, 206, 20),
            (FindFrameKind::GiantCursedSkull, 289, 6),
            (FindFrameKind::AngryBonesBig, 294, 16),
            (FindFrameKind::AngryBonesBigHelmet, 296, 15),
            (FindFrameKind::BloodMummy, 630, 16),
        ];
        for (kind, vanilla, count) in table {
            assert_eq!(kind.vanilla_type(), vanilla, "{kind:?} id");
            assert_eq!(kind.frame_count(), count, "{kind:?} npcFrameCount");
        }
    }

    /// The mummy walk (`NPC.cs:69521-69585`): `|vx| * 2 + 1` = 3 a tick
    /// at vx 1, the row steps at > 6 (every 3rd tick), the band floor is
    /// row 2 and the wrap at 16 returns to 2.
    #[test]
    fn the_mummy_walk_steps_every_three_ticks_and_wraps_to_row_two() {
        let mut s = FrameState::default();
        let i = input(1.0, 0.0);
        let mut rows = Vec::new();
        for _ in 0..6 {
            find_frame_b(FindFrameKind::Mummy, &mut s, &i);
            rows.push(s.row);
        }
        assert_eq!(rows, [2, 2, 3, 3, 3, 4]);
        assert_eq!(s.counter, 0.0);
        assert_eq!(s.sprite_dir, 1);
        // Row 15 + one step = 16 → the wrap to 2 (`:69581-69584`).
        s.row = 15;
        s.counter = 6.0;
        find_frame_b(FindFrameKind::Mummy, &mut s, &i);
        assert_eq!(s.row, 2);
    }

    /// The idle/air split of the humanoid group: the mummy idles row 0
    /// and flies row 1 (`NPC.cs:69517`/`:69645`); the AngryBones' air
    /// row is pinned to 0 (`NPC.cs:69647-69650`); the airborne tick
    /// never writes `spriteDirection`.
    #[test]
    fn the_humanoid_group_idle_and_air_rows_follow_the_pin_list() {
        let mut s = FrameState { sprite_dir: -1, ..Default::default() };
        find_frame_b(FindFrameKind::Mummy, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (0, 0.0));
        find_frame_b(FindFrameKind::Mummy, &mut s, &input(0.0, 2.0));
        // The idle tick above ran with the default `direction = 1`, so the
        // grounded flip (`NPC.cs:68648-68651`) already wrote +1; the air
        // tick itself never writes `spriteDirection` and leaves it at 1.
        assert_eq!((s.row, s.counter, s.sprite_dir), (1, 0.0, 1));
        find_frame_b(FindFrameKind::AngryBones, &mut s, &input(0.0, 2.0));
        assert_eq!(s.row, 0);
        find_frame_b(FindFrameKind::Werewolf, &mut s, &input(0.0, -2.0));
        assert_eq!(s.row, 1);
        for kind in [
            FindFrameKind::ArmoredSkeleton,
            FindFrameKind::UndeadViking,
            FindFrameKind::ArmoredViking,
            FindFrameKind::AngryBonesBig,
            FindFrameKind::AngryBonesBigHelmet,
        ] {
            find_frame_b(kind, &mut s, &input(0.0, 1.0));
            assert_eq!(s.row, 0, "{kind:?} air pin");
        }
        for kind in [
            FindFrameKind::DarkMummy,
            FindFrameKind::LightMummy,
            FindFrameKind::BloodMummy,
        ] {
            find_frame_b(kind, &mut s, &input(0.0, 1.0));
            assert_eq!(s.row, 1, "{kind:?} air row");
        }
    }

    /// The PossessedArmor quirk (`NPC.cs:69510-69514`): the idle stands
    /// on row 1, and the walk's floor clamp lifts it to row 2.
    #[test]
    fn the_possessed_armor_idles_on_row_one() {
        let mut s = FrameState::default();
        find_frame_b(FindFrameKind::PossessedArmor, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (1, 0.0));
        find_frame_b(FindFrameKind::PossessedArmor, &mut s, &input(1.0, 0.0));
        assert_eq!(s.row, 2);
    }

    /// The grounded flip rides `direction` ±1 only (`NPC.cs:68649-68656`);
    /// direction 0 leaves the sprite alone.
    #[test]
    fn the_humanoid_flip_gates_on_the_direction_sign() {
        let mut s = FrameState { sprite_dir: -1, ..Default::default() };
        let mut i = input(1.0, 0.0);
        i.direction = 0;
        find_frame_b(FindFrameKind::Mummy, &mut s, &i);
        assert_eq!(s.sprite_dir, -1);
        i.direction = 1;
        find_frame_b(FindFrameKind::Mummy, &mut s, &i);
        assert_eq!(s.sprite_dir, 1);
    }

    /// The Pixie (`NPC.cs:74113-74133`): the velocity flip (even at
    /// vx 0 → -1), the 0.1 tilt, the 4-tick strip over 4 rows.
    #[test]
    fn the_pixie_flips_by_velocity_and_cycles_four_rows() {
        let mut i = input(2.0, 0.0);
        i.direction = -1;
        let mut s = FrameState::default();
        find_frame_b(FindFrameKind::Pixie, &mut s, &i);
        assert_eq!(s.sprite_dir, 1);
        assert_eq!(s.rotation, 0.2);
        // c hits 4 on ticks 4/8/12/16 → rows 1/2/3/0 (the wrap).
        assert_eq!(
            rows_over(FindFrameKind::Pixie, i, 16),
            [0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 0]
        );
        let mut s = FrameState::default();
        find_frame_b(FindFrameKind::Pixie, &mut s, &input(0.0, 0.0));
        assert_eq!(s.sprite_dir, -1);
    }

    /// The Cursed Skull (`NPC.cs:77055-77066`): the plain 4-tick strip
    /// over 3 rows, no sprite/rotation write.
    #[test]
    fn the_cursed_skull_cycles_three_rows_every_four_ticks() {
        let mut s = FrameState { sprite_dir: -1, rotation: 0.5, ..Default::default() };
        let i = input(3.0, 0.0);
        let mut rows = Vec::new();
        for _ in 0..12 {
            find_frame_b(FindFrameKind::CursedSkull, &mut s, &i);
            rows.push(s.row);
        }
        assert_eq!(rows, [0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 0]);
        assert_eq!(s.sprite_dir, -1);
        assert_eq!(s.rotation, 0.5);
    }

    /// The Wraith (`NPC.cs:74823-74884`): the 6-tick strip over the full
    /// 4 rows, the velocity-sign flip (0 leaves it), the 0.1 tilt.
    #[test]
    fn the_wraith_floats_on_the_six_tick_four_row_strip() {
        let mut s = FrameState::default();
        let i = input(-2.0, 0.0);
        find_frame_b(FindFrameKind::Wraith, &mut s, &i);
        assert_eq!(s.sprite_dir, -1);
        assert_eq!(s.rotation, -0.2);
        assert_eq!(
            rows_over(FindFrameKind::Wraith, i, 24),
            [
                0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 0
            ]
        );
        let mut s = FrameState { sprite_dir: -1, ..Default::default() };
        find_frame_b(FindFrameKind::Wraith, &mut s, &input(0.0, 0.0));
        assert_eq!(s.sprite_dir, -1);
    }

    /// The Mimic (`NPC.cs:73443-73515`): the closed chest pins row 0;
    /// the open chest's counter climbs in the air to the 12 clamp (row
    /// 5) and walks back down on the ground; the `ai[3]` biome banks
    /// shift +6/+12.
    #[test]
    fn the_mimic_chest_opens_by_the_clamped_counter_and_biome_bank() {
        let mut s = FrameState::default();
        let mut closed = input(0.0, -1.0);
        closed.ai[0] = 0.0;
        find_frame_b(FindFrameKind::Mimic, &mut s, &closed);
        assert_eq!((s.row, s.counter), (0, 0.0));
        let mut open = input(0.0, -1.0);
        open.ai[0] = 1.0;
        let mut rows = Vec::new();
        for _ in 0..13 {
            find_frame_b(FindFrameKind::Mimic, &mut s, &open);
            rows.push(s.row);
        }
        // c = 1..12 then clamped: rows 1,1,2,2,2,3,3,3,4,4,4,5,5.
        assert_eq!(rows, [1, 1, 2, 2, 2, 3, 3, 3, 4, 4, 4, 5, 5]);
        assert_eq!(s.counter, 12.0);
        // One grounded tick: c 11 → the `< num189 * 4` band, row 4.
        let ground = FrameInput { vy: 0.0, ..open };
        find_frame_b(FindFrameKind::Mimic, &mut s, &ground);
        assert_eq!((s.row, s.counter), (4, 11.0));
        // The biome banks (`:73507-73514`).
        let mut hallow = FrameInput { vy: -1.0, ..open };
        hallow.ai[3] = 2.0;
        let mut s2 = FrameState::default();
        find_frame_b(FindFrameKind::Mimic, &mut s2, &hallow);
        assert_eq!(s2.row, 1 + 6);
        hallow.ai[3] = 3.0;
        let mut s3 = FrameState::default();
        find_frame_b(FindFrameKind::Mimic, &mut s3, &hallow);
        assert_eq!(s3.row, 1 + 12);
    }

    /// The Unicorn (`NPC.cs:74404-74464`): the stand row 0, the trot's
    /// 6-accumulator over rows 1..8, the gallop's 10-accumulator over
    /// rows 9..14, the air row 15, and the ±2 flip deadband.
    #[test]
    fn the_unicorn_trots_and_gallops_in_their_own_bands() {
        // Trot at vx 2: step every 3 ticks, first step lands row 1.
        assert_eq!(
            rows_over(FindFrameKind::Unicorn, input(2.0, 0.0), 6),
            [0, 0, 1, 1, 1, 2]
        );
        // The trot wrap: row 8 + one step = 9 → row 1 (`:74439-74442`).
        let mut s = FrameState { row: 8, counter: 4.0, ..Default::default() };
        find_frame_b(FindFrameKind::Unicorn, &mut s, &input(2.0, 0.0));
        assert_eq!(s.row, 1);
        // Gallop at vx 5: step every 2 ticks; from the stand the first
        // step lands row 9 (the `<= 8` floor, `:74459-74462`).
        assert_eq!(
            rows_over(FindFrameKind::Unicorn, input(5.0, 0.0), 4),
            [0, 9, 9, 10]
        );
        // The gallop wrap: row 14 + one step = 15 → row 9.
        let mut s = FrameState { row: 14, counter: 5.0, ..Default::default() };
        find_frame_b(FindFrameKind::Unicorn, &mut s, &input(5.0, 0.0));
        assert_eq!(s.row, 9);
        // The air pins row 15 and never flips.
        let mut s = FrameState { sprite_dir: -1, ..Default::default() };
        find_frame_b(FindFrameKind::Unicorn, &mut s, &input(5.0, 1.0));
        assert_eq!((s.row, s.counter, s.sprite_dir), (15, 0.0, -1));
        // The flip deadband: grounded vx 1 rides `direction`.
        let mut i = input(1.0, 0.0);
        i.direction = -1;
        find_frame_b(FindFrameKind::Unicorn, &mut s, &i);
        assert_eq!(s.sprite_dir, -1);
        // Wet keeps the flip alive in the air (`:74405`).
        let mut wet = input(5.0, 1.0);
        wet.wet = true;
        find_frame_b(FindFrameKind::Unicorn, &mut s, &wet);
        assert_eq!(s.sprite_dir, 1);
    }

    /// The Clown (`NPC.cs:74606-74621`): the `|vx|` accumulator keeps
    /// its remainder across steps, and the flip gate needs ground +
    /// moving-with-the-facing.
    #[test]
    fn the_clown_keeps_the_accumulator_remainder_and_gates_the_flip() {
        let mut s = FrameState::default();
        let i = input(3.0, 0.0);
        find_frame_b(FindFrameKind::Clown, &mut s, &i);
        find_frame_b(FindFrameKind::Clown, &mut s, &i);
        find_frame_b(FindFrameKind::Clown, &mut s, &i);
        assert_eq!((s.row, s.counter, s.sprite_dir), (1, 2.0, 1));
        find_frame_b(FindFrameKind::Clown, &mut s, &i);
        find_frame_b(FindFrameKind::Clown, &mut s, &i);
        assert_eq!((s.row, s.counter), (2, 1.0));
        // Airborne: no flip write, the strip still runs.
        let mut s = FrameState { sprite_dir: -1, ..Default::default() };
        find_frame_b(FindFrameKind::Clown, &mut s, &input(3.0, 1.0));
        assert_eq!(s.sprite_dir, -1);
        // Grounded against the facing: no flip either.
        let mut against = input(-3.0, 0.0);
        against.direction = 1;
        let mut s = FrameState { sprite_dir: -1, ..Default::default() };
        find_frame_b(FindFrameKind::Clown, &mut s, &against);
        assert_eq!(s.sprite_dir, -1);
    }

    /// The Skeleton Archer / Icy Merman (`NPC.cs:76017-76098`): the shot
    /// pose pins `(int)ai[2]`, the walk climbs rows 6..19 at `|vx| * 3`,
    /// the air holds row 0.
    #[test]
    fn the_archers_walk_from_row_six_and_pin_the_shot_pose() {
        for kind in [FindFrameKind::SkeletonArcher, FindFrameKind::IcyMerman] {
            let mut s = FrameState::default();
            let i = input(1.0, 0.0);
            let mut rows = Vec::new();
            for _ in 0..6 {
                find_frame_b(kind, &mut s, &i);
                rows.push(s.row);
            }
            assert_eq!(rows, [6, 6, 7, 7, 7, 8], "{kind:?}");
            // The wrap at 20 returns to 6.
            s.row = 19;
            s.counter = 6.0;
            find_frame_b(kind, &mut s, &i);
            assert_eq!(s.row, 6, "{kind:?} wrap");
            // The shot pose.
            let mut shot = input(0.0, 0.0);
            shot.ai[2] = 3.0;
            shot.direction = -1;
            find_frame_b(kind, &mut s, &shot);
            assert_eq!((s.row, s.counter, s.sprite_dir), (3, 0.0, -1), "{kind:?}");
            // The air row 0.
            find_frame_b(kind, &mut s, &input(1.0, 2.0));
            assert_eq!((s.row, s.counter), (0, 0.0), "{kind:?} air");
        }
    }

    /// The Gastropod (`NPC.cs:73607-73627`): the 8-tick flight strip
    /// over rows 0..2, the laser charge pins `3 + ai[3]/8`, the 0.05
    /// tilt and the `direction` flip.
    #[test]
    fn the_gastropod_flies_three_rows_and_pins_the_laser_charge() {
        let mut i = input(2.0, 0.0);
        i.direction = -1;
        let mut s = FrameState::default();
        find_frame_b(FindFrameKind::Gastropod, &mut s, &i);
        assert_eq!(s.sprite_dir, -1);
        assert_eq!(s.rotation, 0.1);
        assert_eq!(
            rows_over(FindFrameKind::Gastropod, i, 24),
            [
                0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 0
            ]
        );
        let mut charge = input(0.0, 0.0);
        charge.ai[3] = 16.0;
        find_frame_b(FindFrameKind::Gastropod, &mut s, &charge);
        assert_eq!((s.row, s.counter), (5, 0.0));
        charge.ai[3] = 63.0;
        find_frame_b(FindFrameKind::Gastropod, &mut s, &charge);
        assert_eq!(s.row, 10);
    }

    /// The Wolf (`NPC.cs:74311-74374`): the air rows 10/11 keep the
    /// counter, the landing enters the run at row 12, the run's 0.4
    /// accumulator steps at > 8 over rows 3..9, and the turn-in-place
    /// band walks 0 → 1 → 2 at 6 ticks a row then parks.
    #[test]
    fn the_wolf_runs_turns_and_lands_through_row_twelve() {
        let mut s = FrameState { counter: 5.0, ..Default::default() };
        find_frame_b(FindFrameKind::Wolf, &mut s, &input(0.0, -1.0));
        assert_eq!((s.row, s.counter), (10, 5.0));
        find_frame_b(FindFrameKind::Wolf, &mut s, &input(0.0, 1.0));
        assert_eq!((s.row, s.counter), (11, 5.0));
        // Landing from the air row: row 12, counter reset.
        find_frame_b(FindFrameKind::Wolf, &mut s, &input(3.0, 0.0));
        assert_eq!((s.row, s.counter, s.sprite_dir), (12, 0.0, 1));
        // The run at vx 4: 1.6 a tick, the 6th tick steps (9.6 > 8) and
        // keeps the 1.6 remainder; row 12 > 9 wraps to 3 on the step.
        let run = input(4.0, 0.0);
        let mut s = FrameState { row: 3, ..Default::default() };
        let mut rows = Vec::new();
        for _ in 0..6 {
            find_frame_b(FindFrameKind::Wolf, &mut s, &run);
            rows.push(s.row);
        }
        assert_eq!(rows, [3, 3, 3, 3, 3, 4]);
        // The C# keeps the remainder (`-= 8`): 6 × 1.6f = 9.599999f,
        // minus 8 = 1.5999994f — the exact f32 the vanilla carries.
        assert_eq!(s.counter, 1.6f32 * 6.0 - 8.0);
        // The run wrap: row 9 + one step → 3.
        let mut s = FrameState { row: 9, counter: 7.0, ..Default::default() };
        find_frame_b(FindFrameKind::Wolf, &mut s, &run);
        assert_eq!(s.row, 3);
        // The idle.
        find_frame_b(FindFrameKind::Wolf, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (0, 0.0));
        // The turn: facing 1 against vx -1 — from row 5 snap to 0, then
        // 6 ticks a row up to the row-2 park.
        let turn = FrameInput { vx: -1.0, direction: 1, ..Default::default() };
        let mut s = FrameState { row: 5, ..Default::default() };
        find_frame_b(FindFrameKind::Wolf, &mut s, &turn);
        // The snap (`NPC.cs:74332-74336`) zeroes the counter, then the SAME
        // tick falls through to `if (num283 < 2) frameCounter++` (`:74337`)
        // — the counter is 1.0, not 0.0 (the 14-tick list below is derived
        // from that 1.0 start).
        assert_eq!((s.row, s.counter, s.sprite_dir), (0, 1.0, 1));
        let mut rows = Vec::new();
        for _ in 0..14 {
            find_frame_b(FindFrameKind::Wolf, &mut s, &turn);
            rows.push(s.row);
        }
        assert_eq!(rows, [0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2]);
        assert_eq!(s.counter, 0.0);
        // A fast against-facing velocity (|vx| >= 4) is a RUN, not a
        // turn — the sprite follows the velocity.
        let mut s = FrameState { row: 3, ..Default::default() };
        let fast = FrameInput { vx: -5.0, direction: 1, ..Default::default() };
        find_frame_b(FindFrameKind::Wolf, &mut s, &fast);
        assert_eq!(s.sprite_dir, -1);
    }

    /// The Herpling (`NPC.cs:73186-73201`): the four hop poses with the
    /// exact ±1.5 boundaries (the slow arc rides row 2).
    #[test]
    fn the_herpling_picks_its_hop_pose_by_the_vertical_speed() {
        let mut s = FrameState::default();
        for (vy, row) in [(0.0, 0), (-2.0, 1), (2.0, 3), (1.0, 2), (-1.5, 2), (1.5, 2)] {
            find_frame_b(FindFrameKind::Herpling, &mut s, &input(0.0, vy));
            assert_eq!(s.row, row, "vy {vy}");
        }
    }

    /// The Derpling (`NPC.cs:73203-73226`): 9 grounded ticks on row 1,
    /// the 10th crouches to row 0 with the counter parked at 30 (the
    /// crouch holds); the rise keeps the counter, the fall resets it.
    #[test]
    fn the_derpling_crouches_after_ten_grounded_ticks_and_holds() {
        let mut s = FrameState::default();
        let ground = input(0.0, 0.0);
        let mut rows = Vec::new();
        for _ in 0..12 {
            find_frame_b(FindFrameKind::Derpling, &mut s, &ground);
            rows.push(s.row);
        }
        assert_eq!(rows, [1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0]);
        assert_eq!(s.counter, 30.0);
        // The rise keeps the parked counter; the fall resets it.
        find_frame_b(FindFrameKind::Derpling, &mut s, &input(0.0, -3.0));
        assert_eq!((s.row, s.counter), (2, 30.0));
        find_frame_b(FindFrameKind::Derpling, &mut s, &input(0.0, 3.0));
        assert_eq!((s.row, s.counter), (2, 0.0));
    }

    /// The Giant Cursed Skull (`NPC.cs:72652-72675`): the 8-tick cycle
    /// rows 0/0/0/1/1/1/1/2, and the `ai[3] == 1` spin bank shifts +3.
    #[test]
    fn the_giant_cursed_skull_cycles_eight_ticks_and_shifts_the_spin_bank() {
        assert_eq!(
            rows_over(FindFrameKind::GiantCursedSkull, input(0.0, 0.0), 8),
            [0, 0, 0, 1, 1, 1, 1, 2]
        );
        let mut spin = input(0.0, 0.0);
        spin.ai[3] = 1.0;
        assert_eq!(
            rows_over(FindFrameKind::GiantCursedSkull, spin, 8),
            [3, 3, 3, 4, 4, 4, 4, 5]
        );
    }
}
