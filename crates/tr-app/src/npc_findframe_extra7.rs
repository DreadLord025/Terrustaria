//! W4-f27 — FindFrame extra wave 7 (slice 7): the FIRST 15 unclaimed
//! missing case ids of `NPC.FindFrame` (`NPC.cs:66684-77370`) in the
//! **306+ band** of the f26 census (219 unclaimed after the
//! d44-d49/e13/e14/f26/f27 claims; the early band claim is the
//! ANALYSIS "W4-f27 FindFrame extra wave 7" section — f26c keeps the
//! 278-305 band below 306). The 306..350 window's 11 unclaimed ids
//! **316/317/318/322/323/324/333/334/335/336/337** plus the next 4
//! ascending **437/438/440/442** — the Ghost, the two costume demon
//! eyes, the three costume skeletons, the four ribbon slimes, the
//! Christmas bunny, the Cultist Tablet, the Cultist Devote, the
//! Cultist Boss Clone and the Gold Bird (`ID/NPCID.cs:11309-11557`).
//! Landed 5 kinds per commit.
//!
//! Shape: the kinds are NOT in `NpcKind` yet (their AI/spawn machines
//! are later rounds), so — like `npc_findframe_a.rs` /
//! `npc_findframe_b.rs` (S1-D-3 waves A/B) and the extra-4 wave — the
//! frame machines land first as pure functions over a small
//! [`FrameState`]/[`FrameInput`] pair. The wiring seam is
//! [`find_frame_kind_extra7`]: when a kind joins `NpcKind`, its
//! `find_frame` arm copies `row`/`counter`/`sprite_dir`/`rotation`
//! out, runs one tick and copies them back (the
//! `Npc::find_frame_wave_a` bridge precedent). No arm draws
//! `Main.rand` — the floater group's only draw (`NPC.cs:74844`) is
//! gated `type == 634` and the eye group's dust (`:74007`) is gated
//! `type == 23`, neither is 316/317/318 — so the wiring cannot move a
//! stream.
//!
//! Row convention: the port tracks frame ROW indices, not pixels —
//! the C#'s `num` (the sheet height / `Main.npcFrameCount[type]`,
//! `NPC.cs:66694`) cancels out of every comparison, exactly as in
//! `npc.rs::find_frame`'s existing arms. The per-kind row counts are
//! `Main.npcFrameCount` (`Main.cs:67082`'s 697-entry initializer),
//! counted entry-by-entry in [`FindFrameExtra7Kind::frame_count`] and
//! cross-checked against the port's own `render/entities.rs::
//! NPC_FRAME_COUNT` table.
//!
//! Deliberate skips (recorded in docs/ANALYSIS.md "W4-f27 FindFrame
//! extra wave 7"):
//! * the floater group's type-60 dust (`NPC.cs:74839-74843`), the
//!   type-634 `Main.rand.Next(10)` dust (`:74844-74849`), the 210/211
//!   double-increment + `vx * 0.2` tilt (`:74859-74863`), the
//!   49/51/60/634 `num292--` (`:74867-74870`) and the type-48
//!   `num291 = 5` (`:74871-74874`) — none is 316 (it flaps the FULL
//!   4 rows every 6 ticks);
//! * the eye group's type-23 dust (`NPC.cs:74007-74013`), the
//!   121/660 `localAI[0]` one-shot row-2 latch (`:74014-74018`), the
//!   SECOND flip branch (`:74032-74043` — unreachable for 317/318,
//!   the first branch's type list carries them) and the type-660
//!   `num142 = 6` override (`:74046-74049`) — 317/318 ride the
//!   8-tick strip with the atan2 aim;
//! * the humanoid group's town `ai[0]` sit/talk/attack pose zones
//!   (`NPC.cs:68662-69488`) — 322/323/324 are aiStyle-3 fighters
//!   (`NPC.cs:13079`/`:13092`/`:13105`, no `townNPC`), the wave-B
//!   `humanoid_fighter_frame` keep-one precedent;
//! * the slime group's 302/304 town-slime sprite snap
//!   (`NPC.cs:70824-70827`) — the ribbon slimes 333-336 are not in
//!   that pair;
//! * the bunny group's air rows 4/6 (`NPC.cs:76894-76903`) — the
//!   7-row sheet's rise/fall pins, shared with 46/47 (d44's) and
//!   303/443/464/540/614/646-652 (later waves');
//! * the 439/440 arm's `IsABestiaryIconDummy` branch
//!   (`NPC.cs:72021-72032`) — a display-only dummy path, the port
//!   has no bestiary-dummy NPC entity; the live `localAI[2]` pose
//!   ladder is carried verbatim.

// The wiring is a later round (the kinds are not in `NpcKind` yet) —
// the `npc_findframe_a.rs`/`npc_findframe_b.rs` precedent.
#![allow(dead_code)]

/// The extra-wave-7 kinds, keyed by their VANILLA `NPC.type`
/// (`ID/NPCID.cs`'s `public const short` rows).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FindFrameExtra7Kind {
    /// `NPCID.Ghost` = 316 (`NPCID.cs:11309`) — the floater group
    /// (`NPC.cs:74834`, shared with 48/49/51/60/82/93/137/182/210/
    /// 211/253/634/662/664).
    Ghost,
    /// `NPCID.DemonEyeOwl` = 317 (`NPCID.cs:11311`) — the eye group
    /// (`NPC.cs:74003`, shared with 2/23/121/169/190-194/660).
    DemonEyeOwl,
    /// `NPCID.DemonEyeSpaceship` = 318 (`NPCID.cs:11313`) — the same
    /// eye group (`NPC.cs:74004`).
    DemonEyeSpaceship,
    /// `NPCID.SkeletonTopHat` = 322 (`NPCID.cs:11321`) — the
    /// humanoid-group fighter arm (`NPC.cs:68599`).
    SkeletonTopHat,
    /// `NPCID.SkeletonAstonaut` = 323 (`NPCID.cs:11323`, the C#'s
    /// spelling) — the same group (`NPC.cs:68600`).
    SkeletonAstonaut,
    /// `NPCID.SkeletonAlien` = 324 (`NPCID.cs:11325`) — the same
    /// group (`NPC.cs:68601`).
    SkeletonAlien,
    /// `NPCID.SlimeRibbonWhite` = 333 (`NPCID.cs:11343`) — the
    /// num2-fed slime strip (`NPC.cs:70814`).
    SlimeRibbonWhite,
    /// `NPCID.SlimeRibbonYellow` = 334 (`NPCID.cs:11345`) — the same
    /// strip (`NPC.cs:70815`).
    SlimeRibbonYellow,
    /// `NPCID.SlimeRibbonGreen` = 335 (`NPCID.cs:11347`) — the same
    /// strip (`NPC.cs:70816`).
    SlimeRibbonGreen,
    /// `NPCID.SlimeRibbonRed` = 336 (`NPCID.cs:11349`) — the same
    /// strip (`NPC.cs:70817`).
    SlimeRibbonRed,
    /// `NPCID.BunnyXmas` = 337 (`NPCID.cs:11351`) — the bunny hop
    /// group (`NPC.cs:76854`, shared with 46/47/303/443/464/540/
    /// 614/646-652).
    BunnyXmas,
    /// `NPCID.CultistTablet` = 437 (`NPCID.cs:11549`) — `case 437`
    /// (`NPC.cs:71987-71996`), the 20-tick 4-band cycle.
    CultistTablet,
    /// `NPCID.CultistDevote` = 438 (`NPCID.cs:11551`) — `case 438`
    /// (`NPC.cs:71998-72015`), the 49-tick mirrored cycle.
    CultistDevote,
    /// `NPCID.CultistBossClone` = 440 (`NPCID.cs:11555`) — the
    /// 439/440 pose ladder (`NPC.cs:72017-72101`), the clone rides
    /// the SAME machine (the 439 boss's own arm is `npc.rs`'s
    /// AncientCultist 1-frame stub — the wiring round's keep-one).
    CultistBossClone,
    /// `NPCID.GoldBird` = 442 (`NPCID.cs:11557`) — the bird group
    /// (`NPC.cs:73631`, shared with 74/297/298).
    GoldBird,
}

impl FindFrameExtra7Kind {
    /// The vanilla `NPC.type` — the `FindFrame` switch key.
    pub fn vanilla_type(self) -> i16 {
        match self {
            Self::Ghost => 316,
            Self::DemonEyeOwl => 317,
            Self::DemonEyeSpaceship => 318,
            Self::SkeletonTopHat => 322,
            Self::SkeletonAstonaut => 323,
            Self::SkeletonAlien => 324,
            Self::SlimeRibbonWhite => 333,
            Self::SlimeRibbonYellow => 334,
            Self::SlimeRibbonGreen => 335,
            Self::SlimeRibbonRed => 336,
            Self::BunnyXmas => 337,
            Self::CultistTablet => 437,
            Self::CultistDevote => 438,
            Self::CultistBossClone => 440,
            Self::GoldBird => 442,
        }
    }

    /// `Main.npcFrameCount[type]` (`Main.cs:67082`'s initializer,
    /// entry `type` on line `67084 + type/10`, slot `type % 10`) —
    /// cross-checked against `render/entities.rs::NPC_FRAME_COUNT`.
    pub fn frame_count(self) -> u32 {
        match self {
            // Main.cs:67115 (entries 316-318): 4, 2, 2.
            Self::Ghost => 4,
            Self::DemonEyeOwl => 2,
            Self::DemonEyeSpaceship => 2,
            // Main.cs:67116 (322/323/324): 16, 15, 16.
            Self::SkeletonTopHat => 16,
            Self::SkeletonAstonaut => 15,
            Self::SkeletonAlien => 16,
            // Main.cs:67117 (333-336): 2, 2, 2, 2; (337): 7.
            Self::SlimeRibbonWhite => 2,
            Self::SlimeRibbonYellow => 2,
            Self::SlimeRibbonGreen => 2,
            Self::SlimeRibbonRed => 2,
            Self::BunnyXmas => 7,
            // Main.cs:67127 (437/438): 4, 4; :67128 (440): 16, (442): 5.
            Self::CultistTablet => 4,
            Self::CultistDevote => 4,
            Self::CultistBossClone => 16,
            Self::GoldBird => 5,
        }
    }
}

/// The wiring seam: the vanilla type → this module's kind (the
/// `find_frame_kind_extra4` precedent). `None` outside the slice.
pub fn find_frame_kind_extra7(vanilla_type: i16) -> Option<FindFrameExtra7Kind> {
    Some(match vanilla_type {
        316 => FindFrameExtra7Kind::Ghost,
        317 => FindFrameExtra7Kind::DemonEyeOwl,
        318 => FindFrameExtra7Kind::DemonEyeSpaceship,
        322 => FindFrameExtra7Kind::SkeletonTopHat,
        323 => FindFrameExtra7Kind::SkeletonAstonaut,
        324 => FindFrameExtra7Kind::SkeletonAlien,
        333 => FindFrameExtra7Kind::SlimeRibbonWhite,
        334 => FindFrameExtra7Kind::SlimeRibbonYellow,
        335 => FindFrameExtra7Kind::SlimeRibbonGreen,
        336 => FindFrameExtra7Kind::SlimeRibbonRed,
        337 => FindFrameExtra7Kind::BunnyXmas,
        437 => FindFrameExtra7Kind::CultistTablet,
        438 => FindFrameExtra7Kind::CultistDevote,
        440 => FindFrameExtra7Kind::CultistBossClone,
        442 => FindFrameExtra7Kind::GoldBird,
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
    /// `NPC.rotation` — the floater/eye/bird arms write it; the
    /// skeleton/slime/bunny/cultist arms never touch it.
    pub rotation: f32,
}

/// The per-tick inputs the arms read: `velocity`, `direction`, the
/// `ai` slots, `aiAction` and `localAI[2]`. `wet` is not carried —
/// none of these arms reads it.
#[derive(Clone, Copy, Debug)]
pub struct FrameInput {
    pub vx: f32,
    pub vy: f32,
    /// `Entity.direction` (±1; 0 leaves `spriteDirection` alone in
    /// the arms that gate on `direction == 1` / `== -1`, but the
    /// bird arm's unconditional copy writes it verbatim).
    pub direction: i32,
    pub ai: [f32; 4],
    /// `NPC.aiAction` — 0 the motion class, 1 the attack (`num2 = 4`,
    /// `NPC.cs:66696-66704`); only the ribbon slimes read it.
    pub ai_action: i32,
    /// `NPC.localAI[2]` — the cultist pose selector (0/1/2 the idle
    /// poses, 10/11/12 the attack poses, 13 the ritual;
    /// `NPC.cs:64616-64644`'s AI_084 writes); only 440 reads it.
    pub local_ai2: f32,
}

impl Default for FrameInput {
    fn default() -> Self {
        Self {
            vx: 0.0,
            vy: 0.0,
            direction: 1,
            ai: [0.0; 4],
            ai_action: 0,
            local_ai2: 0.0,
        }
    }
}

/// `NPC.FindFrame` (`NPC.cs:66684`) for the extra-wave-7 kinds — one
/// tick.
pub fn find_frame_extra7(kind: FindFrameExtra7Kind, s: &mut FrameState, i: &FrameInput) {
    match kind {
        // `case 316` in the floater group (`NPC.cs:74823-74884`,
        // shared with 48/49/51/60/82/93/137/182/210/211/253/634/662/
        // 664 — wave-B's Wraith 82 and extra-4's Reaper 253 are the
        // same machine, keep-one at the merge): the vx-sign flip
        // (0 leaves it), the `vx * 0.1` tilt, the 6-tick strip over
        // the FULL 4 rows — 316 is not in the `num292--` list
        // (49/51/60/634, `:74867-74870`), not the type-48
        // `num291 = 5` override, not the 210/211 double-increment.
        // The group's dust spawns are types 60/634's (the header's
        // skip — the only `Main.rand` draw `:74844` is 634-gated).
        FindFrameExtra7Kind::Ghost => {
            // `:74850-74857`.
            if i.vx > 0.0 {
                s.sprite_dir = 1;
            }
            if i.vx < 0.0 {
                s.sprite_dir = -1;
            }
            // `:74858`.
            s.rotation = i.vx * 0.1;
            // `:74864-74883` — num291 = 6, num292 = the full count.
            s.counter += 1.0;
            if s.counter >= 6.0 {
                s.row += 1;
                s.counter = 0.0;
            }
            if s.row >= kind.frame_count() {
                s.row = 0;
            }
        }
        // `case 317/318` in the eye group (`NPC.cs:73994-74059`,
        // shared with 2/23/121/169/190-194/660 — wave-A's five eye
        // variants are the same machine, keep-one at the merge):
        // the FIRST aim branch (`:74019-74031` — the type list
        // carries 317/318): `spriteDirection` mirrors the flight
        // sign and `rotation = atan2(vy, vx)`, `+ 3.14f` (the
        // literal, not PI) when moving left; both freeze while
        // `vx == 0`. The 8-tick strip (`num142` = 8; the 6 is type
        // 660's) cycles the 2 rows. The 23 dust, the 121/660
        // `localAI[0]` latch and the SECOND flip branch are other
        // kinds' (the header's skips).
        FindFrameExtra7Kind::DemonEyeOwl | FindFrameExtra7Kind::DemonEyeSpaceship => {
            // `:74019-74031`.
            if i.vx > 0.0 {
                s.sprite_dir = 1;
                s.rotation = i.vy.atan2(i.vx);
            }
            if i.vx < 0.0 {
                s.sprite_dir = -1;
                // The original adds the literal `3.14f`, not PI.
                s.rotation = i.vy.atan2(i.vx) + 3.14;
            }
            // `:74044-74058`.
            s.counter += 1.0;
            if s.counter >= 8.0 {
                s.row += 1;
                s.counter = 0.0;
            }
            if s.row >= kind.frame_count() {
                s.row = 0;
            }
        }
        // `case 322/323/324` in the humanoid-group case list
        // (`NPC.cs:68599-68601`) → the shared fighter walk
        // (`NPC.cs:68647-69655`) via the wave-B keep-one helper (the
        // extra-3 SnowFlinx/Nymph/Lihzahrd delegate precedent, the
        // pub(crate) lift). None is `isLikeATownNPC`
        // (`NPC.cs:6856-6866` — only 453 + the town flag; the three
        // SetDefaults arms `:13075-13114` set aiStyle 3, no
        // townNPC), so `num57` = 0 and the walk wraps at the FULL
        // `npcFrameCount`; none is a town slime, so `num108` = 6 and
        // `num109` = row 2. The airborne pin list (`:69647`) carries
        // NONE of 322/323/324 (the group's row 1 stands); none is
        // the 140/489 idle-row-1 pair. The 323 astronaut is the ONE
        // 15-row sheet of the trio (322/324 walk to row 15 before
        // the wrap, 323 to row 14).
        FindFrameExtra7Kind::SkeletonTopHat
        | FindFrameExtra7Kind::SkeletonAstonaut
        | FindFrameExtra7Kind::SkeletonAlien => {
            let bi = crate::npc_findframe_b::FrameInput {
                vx: i.vx,
                vy: i.vy,
                direction: i.direction,
                wet: false,
                ai: i.ai,
            };
            let mut bs = crate::npc_findframe_b::FrameState {
                row: s.row,
                counter: s.counter,
                sprite_dir: s.sprite_dir,
                rotation: s.rotation,
            };
            crate::npc_findframe_b::humanoid_fighter_frame(
                &mut bs,
                &bi,
                kind.frame_count(),
                false,
                false,
            );
            s.row = bs.row;
            s.counter = bs.counter;
            s.sprite_dir = bs.sprite_dir;
            s.rotation = bs.rotation;
        }
        // `case 333/334/335/336` in the slime strip group
        // (`NPC.cs:70801-70846`, shared with 1/16/59/71/81/138/147/
        // 183/184/204/225/302/304/535/537/658/659/667/676 — the
        // port's Slime arm `npc.rs:7772` and wave-A's slime roster
        // are the same machine, keep-one at the merge): the counter
        // feeds on the `num2` motion class (`NPC.cs:66696-66704`):
        // +1 always, +1 more while moving/airborne (num2 > 0), +1
        // more on the attack class (num2 == 4, aiAction 1); the
        // >= 8 step walks the FULL 2 rows with the >= count wrap.
        // The 302/304 sprite snap (`:70824-70827`) is the town
        // slimes', not the ribbons'. No sprite/rotation write.
        FindFrameExtra7Kind::SlimeRibbonWhite
        | FindFrameExtra7Kind::SlimeRibbonYellow
        | FindFrameExtra7Kind::SlimeRibbonGreen
        | FindFrameExtra7Kind::SlimeRibbonRed => {
            // `:66696-66704` — the num2 class.
            let num2 = if i.ai_action == 0 {
                if i.vy < 0.0 {
                    2
                } else if i.vy > 0.0 {
                    3
                } else if i.vx != 0.0 {
                    1
                } else {
                    0
                }
            } else if i.ai_action == 1 {
                4
            } else {
                0
            };
            // `:70828-70845`.
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
            if s.row >= kind.frame_count() {
                s.row = 0;
            }
        }
        // `case 337` in the bunny hop group (`NPC.cs:76851-76904`,
        // shared with 46/47/303/443/464/540/614/646-652 — d44's
        // 46/47 are the same machine, keep-one at the merge): the
        // grounded ±1 flip (0 leaves it), the idle snaps row 0 +
        // zeroes the counter (`:76876-76881`), the walk accumulates
        // `|vx| * 1` PLUS 1 (`:76882-76883`), the > 6 step walks the
        // FULL 7 rows with the >= count wrap to 0
        // (`:76884-76892`); the air pins row 4 rising / row 6
        // falling, both zeroing the counter (`:76894-76903`).
        FindFrameExtra7Kind::BunnyXmas => {
            if i.vy == 0.0 {
                // `:76868-76875`.
                if i.direction == 1 {
                    s.sprite_dir = 1;
                }
                if i.direction == -1 {
                    s.sprite_dir = -1;
                }
                if i.vx == 0.0 {
                    // `:76876-76881`.
                    s.row = 0;
                    s.counter = 0.0;
                    return;
                }
                // `:76882-76892`.
                s.counter += i.vx.abs();
                s.counter += 1.0;
                if s.counter > 6.0 {
                    s.row += 1;
                    s.counter = 0.0;
                }
                if s.row >= kind.frame_count() {
                    s.row = 0;
                }
            } else if i.vy < 0.0 {
                // `:76894-76898`.
                s.counter = 0.0;
                s.row = 4;
            } else {
                // `:76899-76903`.
                s.counter = 0.0;
                s.row = 6;
            }
        }
        // `case 437` (`NPC.cs:71987-71996`): the Cultist Tablet's
        // 20-tick cycle — counter++ then >= 20 resets to 0 FIRST,
        // the row is `(int)counter % 20 / 5` (four 5-tick bands over
        // the 4 rows). No sprite/rotation write, velocity ignored.
        FindFrameExtra7Kind::CultistTablet => {
            s.counter += 1.0;
            if s.counter >= 20.0 {
                s.counter = 0.0;
            }
            s.row = (s.counter as i32 % 20 / 5) as u32;
        }
        // `case 438` (`NPC.cs:71998-72015`): the Cultist Devote's
        // 49-tick mirrored cycle — counter++ (+1 more while
        // `ai[1] == 1`, the ritual chant), >= 49 resets to 0, the
        // band `(int)counter % 49 / 7` runs 0..6 and the >= 4 tail
        // MIRRORS it (`6 - band`: 0/1/2/3/2/1/0 over the 4 rows).
        // No sprite/rotation write.
        FindFrameExtra7Kind::CultistDevote => {
            s.counter += 1.0;
            if i.ai[1] == 1.0 {
                s.counter += 1.0;
            }
            if s.counter >= 49.0 {
                s.counter = 0.0;
            }
            let mut band = s.counter as i32 % 49 / 7;
            if band >= 4 {
                band = 6 - band;
            }
            s.row = band as u32;
        }
        // `case 439/440` (`NPC.cs:72017-72101`): the cultist pose
        // ladder on `localAI[2]` (the AI_084 pose selector,
        // `:64616-64644`): 0/1/2 the idle poses (rows 4-6 / 10-12 /
        // 7-9 while the row is OFF the pose's home row, the >= 15
        // tail snaps the home row 0/1/2), 10/11/12 the attack poses
        // (the SAME bands but the >= 15 tail only zeroes the
        // counter — the row keeps cycling), 13 the ritual (rows
        // 13-15, the >= 15 tail zeroes the counter); any other
        // selector writes NO frame. The counter++ runs AFTER the
        // switch (`:72100`). The `IsABestiaryIconDummy` branch
        // (`:72021-72032`) is the display-only skip (the header).
        // No sprite/rotation write. 439 itself is the port's
        // AncientCultist (the wiring round's keep-one note).
        FindFrameExtra7Kind::CultistBossClone => {
            // `:72020` — num195 = (int)localAI[2].
            let pose = i.local_ai2 as i32;
            // `:72035-72098` — the pose switch.
            match pose {
                0 => {
                    if s.counter >= 15.0 {
                        s.counter = 0.0;
                        s.row = 0;
                    } else if s.row != 0 {
                        s.row = (s.counter as i32 / 5 + 4) as u32;
                    }
                }
                10 => {
                    if s.counter >= 15.0 {
                        s.counter = 0.0;
                    }
                    s.row = (s.counter as i32 / 5 + 4) as u32;
                }
                1 => {
                    if s.counter >= 15.0 {
                        s.counter = 0.0;
                        s.row = 1;
                    } else if s.row != 1 {
                        s.row = (s.counter as i32 / 5 + 10) as u32;
                    }
                }
                11 => {
                    if s.counter >= 15.0 {
                        s.counter = 0.0;
                    }
                    s.row = (s.counter as i32 / 5 + 10) as u32;
                }
                2 => {
                    if s.counter >= 15.0 {
                        s.counter = 0.0;
                        s.row = 2;
                    } else if s.row != 2 {
                        s.row = (s.counter as i32 / 5 + 7) as u32;
                    }
                }
                12 => {
                    if s.counter >= 15.0 {
                        s.counter = 0.0;
                    }
                    s.row = (s.counter as i32 / 5 + 7) as u32;
                }
                13 => {
                    if s.counter >= 15.0 {
                        s.counter = 0.0;
                    }
                    s.row = (s.counter as i32 / 5 + 13) as u32;
                }
                _ => {}
            }
            // `:72100`.
            s.counter += 1.0;
        }
        // `case 442` in the bird group (`NPC.cs:73628-73652`, shared
        // with 74/297/298 — f26c's 297/298 are the same machine,
        // keep-one at the merge): the UNCONDITIONAL direction copy
        // (0 included, `:73633`), the `vx * 0.1` tilt (`:73634`),
        // the still-air snap to row 4 + counter zero (`:73635-
        // 73640` — the 5-row sheet's perch row), the 4-tick strip
        // over rows 0..count-2 (`num124` = count - 1, `:73641-
        // 73651` — row 4 is perch-only on the flap).
        FindFrameExtra7Kind::GoldBird => {
            // `:73633-73634`.
            s.sprite_dir = i.direction;
            s.rotation = i.vx * 0.1;
            if i.vx == 0.0 && i.vy == 0.0 {
                // `:73635-73640`.
                s.row = 4;
                s.counter = 0.0;
                return;
            }
            // `:73641-73651` — num124 = count - 1.
            s.counter += 1.0;
            if s.counter >= 4.0 {
                s.row += 1;
                s.counter = 0.0;
            }
            if s.row >= kind.frame_count() - 1 {
                s.row = 0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(vx: f32, vy: f32) -> FrameInput {
        FrameInput { vx, vy, ..Default::default() }
    }

    fn rows_over(kind: FindFrameExtra7Kind, i: FrameInput, ticks: usize) -> Vec<u32> {
        let mut s = FrameState::default();
        let mut out = Vec::new();
        for _ in 0..ticks {
            find_frame_extra7(kind, &mut s, &i);
            out.push(s.row);
        }
        out
    }

    /// The frame-count table agrees with the port's verbatim
    /// `NPC_FRAME_COUNT` (`Main.cs:67082` initializer cross-check).
    #[test]
    fn f27s7_frame_counts_match_the_port_table() {
        let table = crate::render::entities::NPC_FRAME_COUNT;
        for k in [
            FindFrameExtra7Kind::Ghost,
            FindFrameExtra7Kind::DemonEyeOwl,
            FindFrameExtra7Kind::DemonEyeSpaceship,
            FindFrameExtra7Kind::SkeletonTopHat,
            FindFrameExtra7Kind::SkeletonAstonaut,
            FindFrameExtra7Kind::SkeletonAlien,
            FindFrameExtra7Kind::SlimeRibbonWhite,
            FindFrameExtra7Kind::SlimeRibbonYellow,
            FindFrameExtra7Kind::SlimeRibbonGreen,
            FindFrameExtra7Kind::SlimeRibbonRed,
            FindFrameExtra7Kind::BunnyXmas,
            FindFrameExtra7Kind::CultistTablet,
            FindFrameExtra7Kind::CultistDevote,
            FindFrameExtra7Kind::CultistBossClone,
            FindFrameExtra7Kind::GoldBird,
        ] {
            assert_eq!(
                k.frame_count(),
                table[k.vanilla_type() as usize],
                "type {}",
                k.vanilla_type()
            );
        }
    }

    /// 316 Ghost: the floater 6-tick strip over the FULL 4 rows, the
    /// vx-sign flip (0 holds) and the vx*0.1 tilt
    /// (`NPC.cs:74850-74883`). Expectation re-derived by mechanical
    /// replay of the C# order.
    #[test]
    fn f27s7_ghost_flaps_the_full_four_rows() {
        let rows = rows_over(FindFrameExtra7Kind::Ghost, input(2.0, -1.0), 25);
        assert_eq!(
            rows,
            vec![0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 0, 0]
        );
        let mut s = FrameState { sprite_dir: -1, ..Default::default() };
        find_frame_extra7(FindFrameExtra7Kind::Ghost, &mut s, &input(0.0, 0.0));
        assert_eq!(s.sprite_dir, -1, "vx == 0 leaves the flip alone");
        assert_eq!(s.rotation, 0.0);
        find_frame_extra7(FindFrameExtra7Kind::Ghost, &mut s, &input(-3.0, 0.0));
        assert_eq!(s.sprite_dir, -1);
        assert_eq!(s.rotation, -0.3);
    }

    /// 317/318 the costume eyes: the atan2 aim (the literal 3.14f
    /// left-turn offset), the vx == 0 freeze, the 8-tick 2-row strip
    /// (`NPC.cs:74019-74058`).
    #[test]
    fn f27s7_costume_eyes_aim_and_flap() {
        for k in [FindFrameExtra7Kind::DemonEyeOwl, FindFrameExtra7Kind::DemonEyeSpaceship] {
            let rows = rows_over(k, input(2.0, -1.0), 17);
            assert_eq!(rows, vec![0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0]);
            let mut s = FrameState::default();
            find_frame_extra7(k, &mut s, &input(3.0, 4.0));
            assert_eq!(s.sprite_dir, 1);
            assert_eq!(s.rotation, 4.0f32.atan2(3.0));
            find_frame_extra7(k, &mut s, &input(-3.0, 4.0));
            assert_eq!(s.sprite_dir, -1);
            assert_eq!(s.rotation, 4.0f32.atan2(-3.0) + 3.14);
            // vx == 0 freezes BOTH the flip and the aim.
            let mut s = FrameState { sprite_dir: -1, rotation: 0.5, ..Default::default() };
            find_frame_extra7(k, &mut s, &input(0.0, 2.0));
            assert_eq!((s.sprite_dir, s.rotation), (-1, 0.5));
        }
    }

    /// 322/323/324 the costume skeletons: the wave-B fighter walk —
    /// the |vx|*2+1 accumulator, the > 6 step, the lift-to-2 and the
    /// wrap-to-2 at the FULL count; the 323 astronaut's 15-row sheet
    /// wraps one row earlier (`NPC.cs:69521-69585`).
    #[test]
    fn f27s7_costume_skeletons_walk_the_fighter_bands() {
        for k in [
            FindFrameExtra7Kind::SkeletonTopHat,
            FindFrameExtra7Kind::SkeletonAstonaut,
            FindFrameExtra7Kind::SkeletonAlien,
        ] {
            // vx 1: +3 a tick, the row steps every 3rd tick.
            let rows = rows_over(k, input(1.0, 0.0), 6);
            assert_eq!(rows, vec![2, 2, 3, 3, 3, 4]);
            // The idle snap + the air row 1 (none is in the pin list).
            let mut s = FrameState { row: 5, counter: 4.0, ..Default::default() };
            find_frame_extra7(k, &mut s, &input(0.0, 0.0));
            assert_eq!((s.row, s.counter), (0, 0.0));
            find_frame_extra7(k, &mut s, &input(1.0, -2.0));
            assert_eq!((s.row, s.counter), (1, 0.0));
        }
        // The count variant: 322/324 reach row 15, 323 wraps at 14.
        for (k, last) in [
            (FindFrameExtra7Kind::SkeletonTopHat, 15u32),
            (FindFrameExtra7Kind::SkeletonAstonaut, 14),
            (FindFrameExtra7Kind::SkeletonAlien, 15),
        ] {
            let mut s = FrameState { row: last, counter: 6.0, ..Default::default() };
            find_frame_extra7(k, &mut s, &input(1.0, 0.0));
            assert_eq!(s.row, 2, "type {} wraps at {last}", k.vanilla_type());
        }
    }

    /// 333-336 the ribbon slimes: the num2-fed 8-tick strip over the
    /// 2 rows — idle 8 ticks a step, moving 4, attacking ~3
    /// (`NPC.cs:70828-70845` + `:66696-66704`).
    #[test]
    fn f27s7_ribbon_slimes_feed_on_the_motion_class() {
        for k in [
            FindFrameExtra7Kind::SlimeRibbonWhite,
            FindFrameExtra7Kind::SlimeRibbonYellow,
            FindFrameExtra7Kind::SlimeRibbonGreen,
            FindFrameExtra7Kind::SlimeRibbonRed,
        ] {
            let rows = rows_over(k, input(0.0, 0.0), 17);
            assert_eq!(rows[7], 1, "idle steps on tick 8");
            assert_eq!(rows[15], 0, "the wrap returns to 0 on tick 16");
            let rows = rows_over(k, input(2.0, 0.0), 9);
            assert_eq!(rows[3], 1, "moving steps on tick 4");
            let i = FrameInput { ai_action: 1, ..Default::default() };
            let rows = rows_over(k, i, 7);
            assert_eq!(rows[2], 1, "attacking steps on tick 3 (+3/tick)");
            // No sprite write (the 302/304 snap is the town slimes').
            let mut s = FrameState { sprite_dir: -1, rotation: 0.5, ..Default::default() };
            find_frame_extra7(k, &mut s, &input(2.0, 0.0));
            assert_eq!((s.sprite_dir, s.rotation), (-1, 0.5));
        }
    }

    /// 337 BunnyXmas: the |vx|+1 accumulator with the > 6 step over
    /// the FULL 7 rows, the idle row-0 snap, the air rows 4/6 and
    /// the grounded-only ±1 flip (`NPC.cs:76866-76903`).
    #[test]
    fn f27s7_bunny_hops_the_seven_rows() {
        // vx 2: +3 a tick, steps on ticks 3/6/9...
        let rows = rows_over(FindFrameExtra7Kind::BunnyXmas, input(2.0, 0.0), 22);
        assert_eq!(
            rows,
            vec![0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4, 4, 5, 5, 5, 6, 6, 6, 0, 0]
        );
        let mut s = FrameState { row: 3, counter: 5.0, sprite_dir: -1, ..Default::default() };
        let i = FrameInput { direction: 0, ..Default::default() };
        find_frame_extra7(FindFrameExtra7Kind::BunnyXmas, &mut s, &i);
        assert_eq!((s.row, s.counter, s.sprite_dir), (0, 0.0, -1), "idle snap, dir 0 holds");
        find_frame_extra7(FindFrameExtra7Kind::BunnyXmas, &mut s, &input(2.0, -1.0));
        assert_eq!((s.row, s.counter), (4, 0.0), "rising pins row 4");
        find_frame_extra7(FindFrameExtra7Kind::BunnyXmas, &mut s, &input(2.0, 1.0));
        assert_eq!((s.row, s.counter), (6, 0.0), "falling pins row 6");
    }

    /// 437 CultistTablet: the 20-tick 4-band cycle — the >= 20 reset
    /// runs BEFORE the band read, so tick 20 shows row 0
    /// (`NPC.cs:71989-71995`).
    #[test]
    fn f27s7_tablet_cycles_the_four_bands() {
        let rows = rows_over(FindFrameExtra7Kind::CultistTablet, input(0.0, 0.0), 21);
        assert_eq!(
            rows,
            vec![0, 0, 0, 0, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 0, 0]
        );
    }

    /// 438 CultistDevote: the 49-tick mirrored cycle 0/1/2/3/2/1/0,
    /// the ai[1]==1 chant doubles the tick (`NPC.cs:72000-72014`).
    #[test]
    fn f27s7_devote_mirrors_the_seven_bands() {
        let rows = rows_over(FindFrameExtra7Kind::CultistDevote, input(0.0, 0.0), 50);
        let mut want = Vec::new();
        for b in [0u32, 1, 2, 3, 2, 1, 0] {
            want.extend(std::iter::repeat(b).take(7));
        }
        want.push(0); // tick 50: counter 49 resets to 0, band 0.
        assert_eq!(rows, want);
        // The chant: +2 a tick — tick 24 lands counter 48 (band 6,
        // mirrored to 0), tick 25's 50 >= 49 resets to band 0.
        let i = FrameInput { ai: [0.0, 1.0, 0.0, 0.0], ..Default::default() };
        let rows = rows_over(FindFrameExtra7Kind::CultistDevote, i, 25);
        assert_eq!(rows[23], 0, "counter 48 on tick 24 → band 6 → mirror 0");
        assert_eq!(rows[24], 0, "tick 25: 50 >= 49 resets, band 0");
    }

    /// 440 CultistBossClone: the pose ladder — pose 0's off-home
    /// bands 4/5/6 then the >= 15 home-row-0 snap; pose 10 keeps
    /// cycling 4/5/6 through the reset; pose 13 rides rows 13-15;
    /// an unknown selector writes no frame but still ticks
    /// (`NPC.cs:72035-72100`).
    #[test]
    fn f27s7_clone_rides_the_pose_ladder() {
        // Pose 0 from row 4 (off home): bands 4/5/6, then home 0.
        let i = FrameInput { local_ai2: 0.0, ..Default::default() };
        let mut s = FrameState { row: 4, ..Default::default() };
        let mut rows = Vec::new();
        for _ in 0..17 {
            find_frame_extra7(FindFrameExtra7Kind::CultistBossClone, &mut s, &i);
            rows.push(s.row);
        }
        assert_eq!(rows, vec![4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 6, 6, 6, 6, 6, 0, 0]);
        // Pose 10: the reset does NOT snap home — the cycle repeats.
        let i = FrameInput { local_ai2: 10.0, ..Default::default() };
        let rows = rows_over(FindFrameExtra7Kind::CultistBossClone, i, 17);
        assert_eq!(rows, vec![4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 6, 6, 6, 6, 6, 4, 4]);
        // Pose 13: rows 13-15, the reset keeps cycling.
        let i = FrameInput { local_ai2: 13.0, ..Default::default() };
        let rows = rows_over(FindFrameExtra7Kind::CultistBossClone, i, 16);
        assert_eq!(rows, vec![13, 13, 13, 13, 13, 14, 14, 14, 14, 14, 15, 15, 15, 15, 15, 13]);
        // Unknown selector: no frame write, the counter still ticks.
        let i = FrameInput { local_ai2: 7.0, ..Default::default() };
        let mut s = FrameState { row: 3, counter: 2.0, ..Default::default() };
        find_frame_extra7(FindFrameExtra7Kind::CultistBossClone, &mut s, &i);
        assert_eq!((s.row, s.counter), (3, 3.0));
    }

    /// 442 GoldBird: the unconditional direction copy (0 included),
    /// the vx*0.1 tilt, the still-air row-4 perch, the 4-tick strip
    /// over rows 0..3 (row 4 is perch-only) (`NPC.cs:73633-73651`).
    #[test]
    fn f27s7_gold_bird_flaps_rows_zero_through_three() {
        let rows = rows_over(FindFrameExtra7Kind::GoldBird, input(2.0, -1.0), 17);
        assert_eq!(rows, vec![0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 0, 0]);
        let mut s = FrameState { sprite_dir: -1, ..Default::default() };
        find_frame_extra7(FindFrameExtra7Kind::GoldBird, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter, s.sprite_dir), (4, 0.0, 1), "perch + dir copy");
        let i = FrameInput { direction: 0, ..input(1.0, 0.0) };
        find_frame_extra7(FindFrameExtra7Kind::GoldBird, &mut s, &i);
        assert_eq!(s.sprite_dir, 0, "the copy is unconditional");
        assert_eq!(s.rotation, 0.1);
        // The perch row falls into the flap strip on the first move.
        let mut s = FrameState { row: 4, ..Default::default() };
        find_frame_extra7(FindFrameExtra7Kind::GoldBird, &mut s, &input(1.0, 0.0));
        assert_eq!(s.row, 0, "row 4 >= count-1 wraps immediately");
    }

    /// The 15-kind strip sweep: a 5x3x3 vx/vy/direction input grid
    /// (plus the ai[1]/aiAction/localAI[2] variants for 438/333-336/
    /// 440), 40 ticks each — every row stays inside its sheet (the
    /// extra-4 precedent).
    #[test]
    fn f27s7_fifteen_kind_strip_sweep_stays_inside_the_sheets() {
        let kinds = [
            FindFrameExtra7Kind::Ghost,
            FindFrameExtra7Kind::DemonEyeOwl,
            FindFrameExtra7Kind::DemonEyeSpaceship,
            FindFrameExtra7Kind::SkeletonTopHat,
            FindFrameExtra7Kind::SkeletonAstonaut,
            FindFrameExtra7Kind::SkeletonAlien,
            FindFrameExtra7Kind::SlimeRibbonWhite,
            FindFrameExtra7Kind::SlimeRibbonYellow,
            FindFrameExtra7Kind::SlimeRibbonGreen,
            FindFrameExtra7Kind::SlimeRibbonRed,
            FindFrameExtra7Kind::BunnyXmas,
            FindFrameExtra7Kind::CultistTablet,
            FindFrameExtra7Kind::CultistDevote,
            FindFrameExtra7Kind::CultistBossClone,
            FindFrameExtra7Kind::GoldBird,
        ];
        for k in kinds {
            let count = k.frame_count();
            for vx in [-4.0f32, -1.0, 0.0, 1.0, 4.0] {
                for vy in [-3.0f32, 0.0, 3.0] {
                    for direction in [-1i32, 0, 1] {
                        for &(ai1, ai_action, local_ai2) in &[
                            (0.0, 0, 0.0),
                            (1.0, 0, 10.0),
                            (0.0, 1, 13.0),
                        ] {
                            let i = FrameInput {
                                vx,
                                vy,
                                direction,
                                ai: [0.0, ai1, 0.0, 0.0],
                                ai_action,
                                local_ai2,
                            };
                            let mut s = FrameState::default();
                            for tick in 0..40 {
                                find_frame_extra7(k, &mut s, &i);
                                assert!(
                                    s.row < count,
                                    "type {} tick {tick} row {} >= {count} (vx {vx}, vy {vy}, dir {direction})",
                                    k.vanilla_type(),
                                    s.row
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    /// The seam maps exactly the slice's vanilla types and nothing
    /// else.
    #[test]
    fn f27s7_seam_covers_exactly_the_slice() {
        for t in 0..697i16 {
            let k = find_frame_kind_extra7(t);
            let claimed = [
                316, 317, 318, 322, 323, 324, 333, 334, 335, 336, 337, 437, 438, 440, 442,
            ];
            assert_eq!(k.is_some(), claimed.contains(&t), "type {t}");
            if let Some(k) = k {
                assert_eq!(k.vanilla_type(), t);
            }
        }
    }
}
