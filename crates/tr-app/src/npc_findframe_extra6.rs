//! W4-f26 slice 6 — FindFrame extra wave 6: the NEXT unclaimed missing
//! case ids of `NPC.FindFrame` (`NPC.cs:66684-77370`), counted off the
//! ascending census after the d44-d49/e13/e14/f26/f27 claims (the
//! pushed branches read: d44 owns 37/43/46/47/48/56/57/61/62/63/64/67,
//! d45 69/73/74/83/84/93/94/101/102/103/116/120, d46
//! 150/151/152/156/157/158/159/160/162/163/164/165, d47
//! 166/168/169/170/171/172/173/175/176/178/179/180, d48 355-366, d49
//! 367/374/375/377/378/379/380/430-436/591, e13
//! 121/137/141/142/148/149/181/182/185/195/196/198/199/201/202, e14's
//! declared next-15 203/204/205/209/217/218/219/220/221/224/226/229/
//! 230/231/232, f26 slice-3 182/185/195/196/198/199/201/202/203/204/
//! 205/209/217/218/219, f27 233/234/235/236/237/238/239/240/241/242/
//! 243/244/251/252/253, f26 slice-5 254/255/256/257/258/268/269/270/
//! 271/272/273/274/275/276/277 — the overlaps dedupe). The census
//! re-derived mechanically: 602 C# top-level case ids − 248
//! port-covered − 75 d44-d49 claims = 279 remaining (the e13 base,
//! re-confirmed); minus the 30 e13/e14-declared, f26's 15 + 15 and
//! f27's 15 = 219 unclaimed (the folded segment families — the
//! Destroyer body 135, the Golem heads 246/249, the Fishron hazards
//! 371/373 — resolve through their own `npc.rs::find_frame` arms, not
//! their bestiary folds, the f27 census note). THIS wave owns the
//! FIRST 15 of those from 278 up — **278/279/280/281/282/283/284/285/
//! 286/287/290/291/292/293/295** — the last three Hell Armored Bones,
//! the six post-Plantera dungeon casters, the Bone Lee, the Paladin,
//! the Sniper/Tactical/Commando skeletons and the Angry Bones Big
//! Muscle (`ID/NPCID.cs:11233-11267`). The band skips 288 (the port's
//! DungeonSpirit `npc.rs` arm), 289/294/296 (wave-B's
//! GiantCursedSkull/AngryBonesBig/AngryBonesBigHelmet) and 305 (the
//! port's Scarecrow1 arm) — all port-covered. The f27 co-lane takes
//! the 306+ band (its pushed claim doc read); the in-band remainder
//! 297..304 rides a later slice. Landed 5 kinds per commit.
//!
//! Shape: the kinds are NOT in `NpcKind` yet (their AI/spawn machines
//! are later rounds), so — like `npc_findframe_a.rs` /
//! `npc_findframe_b.rs` (S1-D-3 waves A/B) and the f26/f27 extra waves —
//! the frame machines land first as pure functions over a small
//! [`FrameState`]/[`FrameInput`] pair. The wiring seam is
//! [`find_frame_kind_extra6`] (the f27 precedent): when a kind joins
//! `NpcKind`, its `find_frame` arm copies `row`/`counter`/`sprite_dir`
//! out, runs one tick and copies them back (the
//! `Npc::find_frame_wave_a` bridge precedent). No arm draws
//! `Main.rand`, so the wiring cannot move a stream.
//!
//! Row convention: the port tracks frame ROW indices, not pixels —
//! the C#'s `num` (the sheet height / `Main.npcFrameCount[type]`,
//! `NPC.cs:66694`) cancels out of every comparison, exactly as in
//! `npc.rs::find_frame`'s existing arms. The per-kind row counts are
//! `Main.npcFrameCount` (`Main.cs:67082`'s 697-entry initializer),
//! counted entry-by-entry in [`FindFrameExtra6Kind::frame_count`] and
//! cross-checked against the port's own `render/entities.rs::
//! NPC_FRAME_COUNT` table.
//!
//! Deliberate skips (recorded in docs/ANALYSIS.md "W4-f26 FindFrame
//! extra wave 6"):
//! * the Armored Bones group's `type == 140` idle row-1 branch
//!   (`NPC.cs:75849-75853`) is DEAD inside the 269..280 case list (140
//!   has its own humanoid-group label, `:68562`, wave-B's
//!   PossessedArmor) — not carried; the group's idle always snaps
//!   row 0 (the slice-5 note);
//! * the caster group's `type == 32 && localAI[3] == 1` +3-row bank
//!   (`NPC.cs:77050-77053`) is DEAD inside the 281..286 case list (32
//!   is the DarkCaster, its own label `:77020` — the port's
//!   `npc.rs::find_frame` DarkCaster arm carries the bank) — not
//!   carried;
//! * the humanoid group's town `ai[0]` pose zones (`NPC.cs:68662-
//!   69488`) are unreachable for the aiStyle-3 fighters 287/295 (the
//!   wave-B `humanoid_fighter_frame` keep-one, the extra-3 delegate
//!   precedent);
//! * none of this wave's arms writes `rotation` — the field is not
//!   even in [`FrameState`] (the slice-5 shape).

// The wiring is a later round (the kinds are not in `NpcKind` yet) —
// the `npc_findframe_a.rs`/`npc_findframe_b.rs` precedent.
#![allow(dead_code)]

/// The extra-wave-6 kinds, keyed by their VANILLA `NPC.type`
/// (`ID/NPCID.cs`'s `public const short` rows).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FindFrameExtra6Kind {
    /// `NPCID.HellArmoredBonesSpikeShield` = 278 (`NPCID.cs:11233`) —
    /// the 269..280 Armored Bones group (`NPC.cs:75834`).
    HellArmoredBonesSpikeShield,
    /// `NPCID.HellArmoredBonesMace` = 279 (`NPCID.cs:11235`) — the
    /// same group (`NPC.cs:75835`).
    HellArmoredBonesMace,
    /// `NPCID.HellArmoredBonesSword` = 280 (`NPCID.cs:11237`) — the
    /// same group (`NPC.cs:75836`).
    HellArmoredBonesSword,
    /// `NPCID.RaggedCaster` = 281 (`NPCID.cs:11239`) — the caster
    /// pose group (`NPC.cs:77023`).
    RaggedCaster,
    /// `NPCID.RaggedCasterOpenCoat` = 282 (`NPCID.cs:11241`) — the
    /// same caster group (`NPC.cs:77024`).
    RaggedCasterOpenCoat,
    /// `NPCID.Necromancer` = 283 (`NPCID.cs:11243`) — the same caster
    /// group (`NPC.cs:77025`).
    Necromancer,
    /// `NPCID.NecromancerArmored` = 284 (`NPCID.cs:11245`) — the same
    /// caster group (`NPC.cs:77026`).
    NecromancerArmored,
    /// `NPCID.DiabolistRed` = 285 (`NPCID.cs:11247`) — the same
    /// caster group (`NPC.cs:77027`).
    DiabolistRed,
    /// `NPCID.DiabolistWhite` = 286 (`NPCID.cs:11249`) — the same
    /// caster group (`NPC.cs:77028`).
    DiabolistWhite,
    /// `NPCID.BoneLee` = 287 (`NPCID.cs:11251`) — the humanoid-group
    /// fighter walk (`NPC.cs:68590`, the wave-B keep-one delegate).
    BoneLee,
}

impl FindFrameExtra6Kind {
    /// The vanilla `NPC.type` — the `FindFrame` switch key.
    pub fn vanilla_type(self) -> i16 {
        match self {
            Self::HellArmoredBonesSpikeShield => 278,
            Self::HellArmoredBonesMace => 279,
            Self::HellArmoredBonesSword => 280,
            Self::RaggedCaster => 281,
            Self::RaggedCasterOpenCoat => 282,
            Self::Necromancer => 283,
            Self::NecromancerArmored => 284,
            Self::DiabolistRed => 285,
            Self::DiabolistWhite => 286,
            Self::BoneLee => 287,
        }
    }

    /// `Main.npcFrameCount[type]` (`Main.cs:67082`'s initializer,
    /// entry `type` on line `67084 + type/10`, slot `type % 10`) —
    /// cross-checked against `render/entities.rs::NPC_FRAME_COUNT`.
    pub fn frame_count(self) -> u32 {
        match self {
            // Main.cs:67111 (entries 278/279): 15, 15; :67112 (280): 15.
            Self::HellArmoredBonesSpikeShield => 15,
            Self::HellArmoredBonesMace => 15,
            Self::HellArmoredBonesSword => 15,
            // Main.cs:67112 (entries 281..286): 3, 3, 3, 3, 3, 3.
            Self::RaggedCaster => 3,
            Self::RaggedCasterOpenCoat => 3,
            Self::Necromancer => 3,
            Self::NecromancerArmored => 3,
            Self::DiabolistRed => 3,
            Self::DiabolistWhite => 3,
            // Main.cs:67112 (entry 287): 16 — the ONE 16-row sheet in
            // this slice's humanoid pair (295 is 15).
            Self::BoneLee => 16,
        }
    }
}

/// The wiring seam: the vanilla type → this module's kind (the
/// `find_frame_kind_extra5` precedent). `None` outside the slice.
pub fn find_frame_kind_extra6(vanilla_type: i16) -> Option<FindFrameExtra6Kind> {
    Some(match vanilla_type {
        278 => FindFrameExtra6Kind::HellArmoredBonesSpikeShield,
        279 => FindFrameExtra6Kind::HellArmoredBonesMace,
        280 => FindFrameExtra6Kind::HellArmoredBonesSword,
        281 => FindFrameExtra6Kind::RaggedCaster,
        282 => FindFrameExtra6Kind::RaggedCasterOpenCoat,
        283 => FindFrameExtra6Kind::Necromancer,
        284 => FindFrameExtra6Kind::NecromancerArmored,
        285 => FindFrameExtra6Kind::DiabolistRed,
        286 => FindFrameExtra6Kind::DiabolistWhite,
        287 => FindFrameExtra6Kind::BoneLee,
        _ => return None,
    })
}

/// The `NPC` fields `FindFrame` reads/writes for these arms:
/// `frame.Y / num` (the row), `frameCounter`, `spriteDirection`. No
/// slice-6 arm writes `rotation` (the header's skip) — the field is
/// not carried. The wiring step copies them to/from `Npc`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FrameState {
    /// `frame.Y / num` — the sheet row.
    pub row: u32,
    /// `NPC.frameCounter`.
    pub counter: f32,
    /// `NPC.spriteDirection` (±1).
    pub sprite_dir: i32,
}

/// The per-tick inputs the arms read: `velocity`, `direction` and the
/// `ai` slots (the casters' `ai[1]` windup, the Paladin's `ai[1]`/
/// `ai[2]` hammer ladder) — no slice-6 arm reads `wet` or `aiAction`.
#[derive(Clone, Copy, Debug)]
pub struct FrameInput {
    pub vx: f32,
    pub vy: f32,
    /// `Entity.direction` (±1; 0 leaves `spriteDirection` alone in
    /// the arms that gate on `direction == 1` / `== -1`).
    pub direction: i32,
    pub ai: [f32; 4],
}

impl Default for FrameInput {
    fn default() -> Self {
        Self { vx: 0.0, vy: 0.0, direction: 1, ai: [0.0; 4] }
    }
}

/// `NPC.FindFrame` (`NPC.cs:66684`) for the extra-wave-6 kinds — one
/// tick.
pub fn find_frame_extra6(kind: FindFrameExtra6Kind, s: &mut FrameState, i: &FrameInput) {
    match kind {
        // `case 278..280` in the Armored Bones group (`NPC.cs:75825-
        // 75882`, the 269..280 list — the slice-5 nine are the SAME
        // machine, keep-one at the merge): the grounded ±1 flip
        // (`:75839-75846`); the grounded idle snaps row 0 + zeroes
        // the counter (`:75847-75859` — the `type == 140` row-1
        // branch is DEAD inside this case list, the header's skip);
        // the walk lifts any row < 2 to 2 FIRST (`:75861-75864`),
        // then accumulates `|vx| * 2` PLUS 1 (`:75865-75866`), the
        // > 6 counter steps a row (`:75867-75871`), wrapping at the
        // sheet's full count back to row 2 (`:75872-75875`); the AIR
        // branch zeroes the counter and pins row 0 (`:75877-75881`).
        // No rotation write.
        FindFrameExtra6Kind::HellArmoredBonesSpikeShield
        | FindFrameExtra6Kind::HellArmoredBonesMace
        | FindFrameExtra6Kind::HellArmoredBonesSword => {
            if i.vy == 0.0 {
                // `:75839-75846`.
                if i.direction == 1 {
                    s.sprite_dir = 1;
                }
                if i.direction == -1 {
                    s.sprite_dir = -1;
                }
                // `:75847-75859` — the dead 140 branch not carried.
                if i.vx == 0.0 {
                    s.row = 0;
                    s.counter = 0.0;
                    return;
                }
                // `:75861-75864`.
                if s.row < 2 {
                    s.row = 2;
                }
                // `:75865-75871`.
                s.counter += i.vx.abs() * 2.0;
                s.counter += 1.0;
                if s.counter > 6.0 {
                    s.row += 1;
                    s.counter = 0.0;
                }
                // `:75872-75875`.
                if s.row >= kind.frame_count() {
                    s.row = 2;
                }
            } else {
                // `:75877-75881`.
                s.counter = 0.0;
                s.row = 0;
            }
        }
        // `case 281/282` in the caster pose group (`NPC.cs:77020-
        // 77054`, shared with 32/45/172/693 — the port's
        // GoblinSorcerer/Tim/DarkCaster arm `npc.rs:8066-8078` is the
        // same machine, keep-one at the merge): the grounded ±1 flip
        // (`:77030-77040`); the row snaps to 0 EVERY tick
        // (`:77041`), then the air lifts to row 2 (`:77042-77045`)
        // ELSE the `ai[1] > 0` cast windup lifts to row 1
        // (`:77046-77049`). The `type == 32 && localAI[3] == 1`
        // +3-row bank (`:77050-77053`) is DEAD inside this case list
        // (the header's skip). The counter is NEVER touched; no
        // rotation write.
        FindFrameExtra6Kind::RaggedCaster
        | FindFrameExtra6Kind::RaggedCasterOpenCoat
        | FindFrameExtra6Kind::Necromancer
        | FindFrameExtra6Kind::NecromancerArmored
        | FindFrameExtra6Kind::DiabolistRed
        | FindFrameExtra6Kind::DiabolistWhite => {
            // `:77030-77040`.
            if i.vy == 0.0 {
                if i.direction == 1 {
                    s.sprite_dir = 1;
                }
                if i.direction == -1 {
                    s.sprite_dir = -1;
                }
            }
            // `:77041-77049` — the dead 32 bank not carried.
            s.row = 0;
            if i.vy != 0.0 {
                s.row = 2;
            } else if i.ai[1] > 0.0 {
                s.row = 1;
            }
        }
        // The humanoid-group case list (`NPC.cs:68590` carries 287)
        // → the shared fighter walk (`NPC.cs:68647-69655`) via the
        // wave-B keep-one helper (the extra-3 SnowFlinx/Nymph/
        // Lihzahrd delegate precedent). Bone Lee is NOT
        // `isLikeATownNPC` (`NPC.cs:6856-6866` — only 453 + the town
        // flag; its SetDefaults arm `:12724-12737` sets aiStyle 3, no
        // townNPC), so `num57` = 0 and the walk wraps at the FULL
        // `npcFrameCount` = 16; not a town slime, so `num108` = 6 and
        // `num109` = row 2. NOT in the airborne pin list (`:69647` —
        // the air holds row 1), NOT the 140/489 idle-row-1 pair (the
        // idle snaps row 0, `:69510-69518`).
        FindFrameExtra6Kind::BoneLee => {
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
                rotation: 0.0,
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(vx: f32, vy: f32) -> FrameInput {
        FrameInput { vx, vy, ..Default::default() }
    }

    fn rows_over(kind: FindFrameExtra6Kind, i: FrameInput, ticks: usize) -> Vec<u32> {
        let mut s = FrameState::default();
        let mut out = Vec::new();
        for _ in 0..ticks {
            find_frame_extra6(kind, &mut s, &i);
            out.push(s.row);
        }
        out
    }

    /// The frame-count table agrees with the port's verbatim
    /// `NPC_FRAME_COUNT` (`Main.cs:67082` initializer cross-check).
    #[test]
    fn f26s6_frame_counts_match_the_port_table() {
        let table = crate::render::entities::NPC_FRAME_COUNT;
        for k in [
            FindFrameExtra6Kind::HellArmoredBonesSpikeShield,
            FindFrameExtra6Kind::HellArmoredBonesMace,
            FindFrameExtra6Kind::HellArmoredBonesSword,
            FindFrameExtra6Kind::RaggedCaster,
            FindFrameExtra6Kind::RaggedCasterOpenCoat,
            FindFrameExtra6Kind::Necromancer,
            FindFrameExtra6Kind::NecromancerArmored,
            FindFrameExtra6Kind::DiabolistRed,
            FindFrameExtra6Kind::DiabolistWhite,
            FindFrameExtra6Kind::BoneLee,
        ] {
            assert_eq!(
                k.frame_count(),
                table[k.vanilla_type() as usize],
                "{k:?} ({})",
                k.vanilla_type()
            );
        }
    }

    /// The Hell bones walk at vx 3 steps EVERY tick (3*2+1 = 7 > 6):
    /// the first tick lifts idle row 0 to 2 then steps to 3, the
    /// stride reaches row 14 and the 13th tick wraps 15 → 2
    /// (`NPC.cs:75861-75875`, replay-derived).
    #[test]
    fn f26s6_the_hell_bones_stride_from_2_and_wrap_at_15() {
        for k in [
            FindFrameExtra6Kind::HellArmoredBonesSpikeShield,
            FindFrameExtra6Kind::HellArmoredBonesMace,
            FindFrameExtra6Kind::HellArmoredBonesSword,
        ] {
            assert_eq!(
                rows_over(k, input(3.0, 0.0), 13),
                vec![3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 2],
                "{k:?}"
            );
        }
    }

    /// The Hell bones' air pins row 0 + zeroes the counter; the
    /// grounded idle snaps row 0; the grounded flip gates on
    /// direction ±1 (0 leaves the flip alone).
    #[test]
    fn f26s6_the_hell_bones_air_idle_pins_and_flip_gate() {
        let k = FindFrameExtra6Kind::HellArmoredBonesSword;
        let mut s = FrameState { row: 9, counter: 4.0, sprite_dir: 1 };
        find_frame_extra6(k, &mut s, &input(2.0, -1.0));
        assert_eq!((s.row, s.counter), (0, 0.0));
        find_frame_extra6(k, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (0, 0.0));
        let mut i = input(1.0, 0.0);
        i.direction = 0;
        find_frame_extra6(k, &mut s, &i);
        assert_eq!(s.sprite_dir, 1, "direction 0 leaves the flip");
        i.direction = -1;
        find_frame_extra6(k, &mut s, &i);
        assert_eq!(s.sprite_dir, -1);
    }

    /// The caster pose machine (`NPC.cs:77041-77049`): row 0 stands,
    /// `ai[1] > 0` winds up on row 1, ANY nonzero vy pins row 2 (the
    /// air wins over the windup); the counter is NEVER touched.
    #[test]
    fn f26s6_the_casters_pose_ladder_keeps_the_counter() {
        for k in [
            FindFrameExtra6Kind::RaggedCaster,
            FindFrameExtra6Kind::RaggedCasterOpenCoat,
            FindFrameExtra6Kind::Necromancer,
            FindFrameExtra6Kind::NecromancerArmored,
            FindFrameExtra6Kind::DiabolistRed,
            FindFrameExtra6Kind::DiabolistWhite,
        ] {
            let mut s = FrameState { row: 2, counter: 7.0, sprite_dir: 1 };
            find_frame_extra6(k, &mut s, &input(3.0, 0.0));
            assert_eq!((s.row, s.counter), (0, 7.0), "{k:?} stand");
            let mut windup = input(0.0, 0.0);
            windup.ai[1] = 0.5;
            find_frame_extra6(k, &mut s, &windup);
            assert_eq!((s.row, s.counter), (1, 7.0), "{k:?} windup");
            let mut air = input(0.0, -2.0);
            air.ai[1] = 3.0;
            find_frame_extra6(k, &mut s, &air);
            assert_eq!((s.row, s.counter), (2, 7.0), "{k:?} air wins");
            find_frame_extra6(k, &mut s, &input(0.0, 2.0));
            assert_eq!(s.row, 2, "{k:?} falling air too");
        }
    }

    /// The Bone Lee rides the wave-B humanoid fighter walk on the
    /// 16-row sheet (`NPC.cs:69521-69585`): vx 3 steps every tick
    /// (3*2+1 = 7 > 6), the stride lifts idle row 0 to 2 then steps
    /// to 3, reaches row 15 and the 14th tick wraps 16 → 2; the air
    /// holds row 1 (287 is NOT in the `:69647` pin list); the idle
    /// snaps row 0 (not the 140/489 pair).
    #[test]
    fn f26s6_the_bone_lee_walks_the_16_row_sheet() {
        let k = FindFrameExtra6Kind::BoneLee;
        assert_eq!(
            rows_over(k, input(3.0, 0.0), 14),
            vec![3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 2]
        );
        let mut s = FrameState { row: 9, counter: 4.0, sprite_dir: 1 };
        find_frame_extra6(k, &mut s, &input(2.0, -1.0));
        assert_eq!((s.row, s.counter), (1, 0.0), "air row 1, not 0");
        find_frame_extra6(k, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (0, 0.0), "idle row 0");
        // The slow walk keeps the accumulator: vx 1 → +3/tick, the
        // step fires on ticks 3/6 (9 > 6, remainder zeroed).
        assert_eq!(rows_over(k, input(1.0, 0.0), 7), vec![2, 2, 3, 3, 3, 4, 4]);
    }

    /// The seam maps exactly this slice's ids — the covered 288 and
    /// the slice-5 277 stay `None`.
    #[test]
    fn f26s6_the_seam_maps_exactly_the_slice() {
        for (id, k) in [
            (278, FindFrameExtra6Kind::HellArmoredBonesSpikeShield),
            (279, FindFrameExtra6Kind::HellArmoredBonesMace),
            (280, FindFrameExtra6Kind::HellArmoredBonesSword),
            (281, FindFrameExtra6Kind::RaggedCaster),
            (282, FindFrameExtra6Kind::RaggedCasterOpenCoat),
            (283, FindFrameExtra6Kind::Necromancer),
            (284, FindFrameExtra6Kind::NecromancerArmored),
            (285, FindFrameExtra6Kind::DiabolistRed),
            (286, FindFrameExtra6Kind::DiabolistWhite),
            (287, FindFrameExtra6Kind::BoneLee),
        ] {
            assert_eq!(find_frame_kind_extra6(id), Some(k));
            assert_eq!(k.vanilla_type(), id);
        }
        for id in [277, 288, 290, 305] {
            assert_eq!(find_frame_kind_extra6(id), None, "{id}");
        }
    }
}
