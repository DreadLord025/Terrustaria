//! W4-f26 slice 5 — FindFrame extra wave 5: the NEXT unclaimed missing
//! case ids of `NPC.FindFrame` (`NPC.cs:66684-77370`), counted off the
//! ascending census after the d44-d49/e13/e14/f26/f27 claims (the
//! pushed branches read: d44 owns 37/43/46/47/48/56/57/61/62/63/64/67,
//! d45 69/73/74/83/84/93/94/101/102/103/116/120, d46
//! 150/151/152/156/157/158/159/160/162/163/164/165, d47
//! 166/168/169/170/171/172/173/175/176/178/179/180, d48 355-366, d49
//! 367/374/375/377/378/379/380/430-436/591, e13
//! 121/137/141/142/148/149/181/182/185/195/196/198/199/201/202, e14's
//! declared next-15 203/204/205/209/217/218/219/220/221/224/226/229/
//! 230/231/232, f26 182/185/195/196/198/199/201/202/203/204/205/209/
//! 217/218/219, f27 233/234/235/236/237/238/239/240/241/242/243/244/
//! 251/252/253 — the overlaps dedupe). The census re-derived
//! mechanically: 602 C# top-level case ids − 248 port-covered − 75
//! d44-d49 claims = 279 remaining (the e13 base, re-confirmed); minus
//! the 30 e13/e14-declared, f26's 15 and f27's 15 = 234 unclaimed (the
//! folded segment families — the Destroyer body 135, the Golem heads
//! 246/249, the Fishron hazards 371/373 — resolve through their own
//! `npc.rs::find_frame` arms, not their bestiary folds, the f27 census
//! note). THIS wave owns the FIRST 15 of those — **254/255/256/257/
//! 258/268/269/270/271/272/273/274/275/276/277** — the two mushroom
//! zombies, the Fungo Fish, the Anomura Fungus, the Mushi Ladybug, the
//! Ichor Sticker and the first nine post-Plantera Armored Bones
//! (`ID/NPCID.cs:11185-11235`). The next ids (278/279/280/281/…) ride
//! the next slice. Landed 5 kinds per commit.
//!
//! Shape: the kinds are NOT in `NpcKind` yet (their AI/spawn machines
//! are later rounds), so — like `npc_findframe_a.rs` /
//! `npc_findframe_b.rs` (S1-D-3 waves A/B) and the f26/f27 extra waves —
//! the frame machines land first as pure functions over a small
//! [`FrameState`]/[`FrameInput`] pair. The wiring seam is
//! [`find_frame_kind_extra5`] (the f27 precedent): when a kind joins
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
//! counted entry-by-entry in [`FindFrameExtra5Kind::frame_count`] and
//! cross-checked against the port's own `render/entities.rs::
//! NPC_FRAME_COUNT` table.
//!
//! Deliberate skips (recorded in docs/ANALYSIS.md "W4-f26 FindFrame
//! extra wave 5"):
//! * the jellyfish group's `ai[1] == 1` pulse gate (`NPC.cs:73921-
//!   73938`) — the 63/64/103/242 list does NOT carry 256, so the Fungo
//!   Fish always rides the plain idle bands; the gate is d44's 63/64,
//!   d45/e14's 103 and f27's 242 (the same machine, keep-one);
//! * the Armored Bones group's `type == 140` idle row-1 branch
//!   (`NPC.cs:75849-75853`) is DEAD inside the 269..280 case list (140
//!   has its own humanoid-group label, `:68562`, wave-B's
//!   PossessedArmor) — not carried; the group's idle always snaps
//!   row 0;
//! * none of this wave's arms writes `rotation` — the field is not
//!   even in [`FrameState`] (the f26 extra-3 shape before the floater
//!   batches added it).

// The wiring is a later round (the kinds are not in `NpcKind` yet) —
// the `npc_findframe_a.rs`/`npc_findframe_b.rs` precedent.
#![allow(dead_code)]

/// The extra-wave-5 kinds, keyed by their VANILLA `NPC.type`
/// (`ID/NPCID.cs`'s `public const short` rows).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FindFrameExtra5Kind {
    /// `NPCID.ZombieMushroom` = 254 (`NPCID.cs:11185`) — the case-3
    /// zombie walker group (`NPC.cs:76336`).
    ZombieMushroom,
    /// `NPCID.ZombieMushroomHat` = 255 (`NPCID.cs:11187`) — the same
    /// walker group (`NPC.cs:76337`).
    ZombieMushroomHat,
    /// `NPCID.FungoFish` = 256 (`NPCID.cs:11189`) — the jellyfish
    /// group (`NPC.cs:73920`), NOT a pulse-gate member.
    FungoFish,
    /// `NPCID.AnomuraFungus` = 257 (`NPCID.cs:11191`) — `case 257`
    /// (`NPC.cs:76179-76224`).
    AnomuraFungus,
    /// `NPCID.MushiLadybug` = 258 (`NPCID.cs:11193`) — `case 258`
    /// (`NPC.cs:76225-76275`).
    MushiLadybug,
    /// `NPCID.IchorSticker` = 268 (`NPCID.cs:11213`) — the 250/264
    /// 6-tick strip group (`NPC.cs:72884-72897`).
    IchorSticker,
    /// `NPCID.RustyArmoredBonesAxe` = 269 (`NPCID.cs:11215`) — the
    /// 269..280 Armored Bones group (`NPC.cs:75825-75882`).
    RustyArmoredBonesAxe,
    /// `NPCID.RustyArmoredBonesFlail` = 270 (`NPCID.cs:11217`) — the
    /// same group.
    RustyArmoredBonesFlail,
    /// `NPCID.RustyArmoredBonesSword` = 271 (`NPCID.cs:11219`) — the
    /// same group.
    RustyArmoredBonesSword,
    /// `NPCID.RustyArmoredBonesSwordNoArmor` = 272 (`NPCID.cs:11221`)
    /// — the same group.
    RustyArmoredBonesSwordNoArmor,
    /// `NPCID.BlueArmoredBones` = 273 (`NPCID.cs:11223`) — the same
    /// group.
    BlueArmoredBones,
    /// `NPCID.BlueArmoredBonesMace` = 274 (`NPCID.cs:11225`) — the
    /// same group.
    BlueArmoredBonesMace,
    /// `NPCID.BlueArmoredBonesNoPants` = 275 (`NPCID.cs:11227`) — the
    /// same group, the ONE 16-row sheet (`Main.cs:67111` slot 5).
    BlueArmoredBonesNoPants,
    /// `NPCID.BlueArmoredBonesSword` = 276 (`NPCID.cs:11229`) — the
    /// same group.
    BlueArmoredBonesSword,
    /// `NPCID.HellArmoredBones` = 277 (`NPCID.cs:11231`) — the same
    /// group.
    HellArmoredBones,
}

impl FindFrameExtra5Kind {
    /// The vanilla `NPC.type` — the `FindFrame` switch key.
    pub fn vanilla_type(self) -> i16 {
        match self {
            Self::ZombieMushroom => 254,
            Self::ZombieMushroomHat => 255,
            Self::FungoFish => 256,
            Self::AnomuraFungus => 257,
            Self::MushiLadybug => 258,
            Self::IchorSticker => 268,
            Self::RustyArmoredBonesAxe => 269,
            Self::RustyArmoredBonesFlail => 270,
            Self::RustyArmoredBonesSword => 271,
            Self::RustyArmoredBonesSwordNoArmor => 272,
            Self::BlueArmoredBones => 273,
            Self::BlueArmoredBonesMace => 274,
            Self::BlueArmoredBonesNoPants => 275,
            Self::BlueArmoredBonesSword => 276,
            Self::HellArmoredBones => 277,
        }
    }

    /// `Main.npcFrameCount[type]` (`Main.cs:67082`'s initializer,
    /// entry `type` on line `67084 + type/10`, slot `type % 10`) —
    /// cross-checked against `render/entities.rs::NPC_FRAME_COUNT`.
    pub fn frame_count(self) -> u32 {
        match self {
            // Main.cs:67109 (entries 254/255/256/257/258): 3, 3, 4, 5, 6.
            Self::ZombieMushroom => 3,
            Self::ZombieMushroomHat => 3,
            Self::FungoFish => 4,
            Self::AnomuraFungus => 5,
            Self::MushiLadybug => 6,
            // Main.cs:67110 (268/269): 4, 15; :67111 (270/271/272): 15, 15, 15.
            Self::IchorSticker => 4,
            Self::RustyArmoredBonesAxe => 15,
            Self::RustyArmoredBonesFlail => 15,
            Self::RustyArmoredBonesSword => 15,
            Self::RustyArmoredBonesSwordNoArmor => 15,
            // Main.cs:67111 (273..277): 15, 15, 16, 15, 15 — 275 is
            // the ONE 16-row sheet in the group.
            Self::BlueArmoredBones => 15,
            Self::BlueArmoredBonesMace => 15,
            Self::BlueArmoredBonesNoPants => 16,
            Self::BlueArmoredBonesSword => 15,
            Self::HellArmoredBones => 15,
        }
    }
}

/// The wiring seam: the vanilla type → this module's kind (the
/// `find_frame_kind_extra4` precedent). `None` outside the slice.
pub fn find_frame_kind_extra5(vanilla_type: i16) -> Option<FindFrameExtra5Kind> {
    Some(match vanilla_type {
        254 => FindFrameExtra5Kind::ZombieMushroom,
        255 => FindFrameExtra5Kind::ZombieMushroomHat,
        256 => FindFrameExtra5Kind::FungoFish,
        257 => FindFrameExtra5Kind::AnomuraFungus,
        258 => FindFrameExtra5Kind::MushiLadybug,
        268 => FindFrameExtra5Kind::IchorSticker,
        269 => FindFrameExtra5Kind::RustyArmoredBonesAxe,
        270 => FindFrameExtra5Kind::RustyArmoredBonesFlail,
        271 => FindFrameExtra5Kind::RustyArmoredBonesSword,
        272 => FindFrameExtra5Kind::RustyArmoredBonesSwordNoArmor,
        273 => FindFrameExtra5Kind::BlueArmoredBones,
        274 => FindFrameExtra5Kind::BlueArmoredBonesMace,
        275 => FindFrameExtra5Kind::BlueArmoredBonesNoPants,
        276 => FindFrameExtra5Kind::BlueArmoredBonesSword,
        277 => FindFrameExtra5Kind::HellArmoredBones,
        _ => return None,
    })
}

/// The `NPC` fields `FindFrame` reads/writes for these arms:
/// `frame.Y / num` (the row), `frameCounter`, `spriteDirection`. No
/// slice-5 arm writes `rotation` (the header's skip) — the field is
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

/// The per-tick inputs the arms read: `velocity` and `direction`
/// only — no slice-5 arm reads `wet`, the `ai` slots or `aiAction`
/// (256's pulse gate excludes it, the header's skip).
#[derive(Clone, Copy, Debug)]
pub struct FrameInput {
    pub vx: f32,
    pub vy: f32,
    /// `Entity.direction` (±1; 0 leaves `spriteDirection` alone in
    /// the arms that gate on `direction == 1` / `== -1`).
    pub direction: i32,
}

impl Default for FrameInput {
    fn default() -> Self {
        Self { vx: 0.0, vy: 0.0, direction: 1 }
    }
}

/// `NPC.FindFrame` (`NPC.cs:66684`) for the extra-wave-5 kinds — one
/// tick.
pub fn find_frame_extra5(kind: FindFrameExtra5Kind, s: &mut FrameState, i: &FrameInput) {
    match kind {
        // `case 254/255` in the zombie walker group (`NPC.cs:76324-
        // 76391`, shared with 3/52/53/132/161/186-189/200/223/251/
        // 319-321/331/332/342/536/590/691 — the wave-A zombie arm and
        // f27's 251 Eyezor are the same machine, keep-one at the
        // merge): the grounded flip writes only on direction ±1
        // (`:76347-76357`); the air OR the facing-against-motion turn
        // pins row 2 + zeroes the counter (`:76358-76363` — the 3-row
        // sheet's jump row); the idle snaps row 0 (`:76364-76369`);
        // the walk accumulates |vx| over the <8/<16/<24/<32 bands
        // rows 0/1/2/1 with the >= 32 tail zeroing the counter and
        // HOLDING the row (`:76370-76390`).
        FindFrameExtra5Kind::ZombieMushroom | FindFrameExtra5Kind::ZombieMushroomHat => {
            // `:76347-76357`.
            if i.vy == 0.0 {
                if i.direction == 1 {
                    s.sprite_dir = 1;
                }
                if i.direction == -1 {
                    s.sprite_dir = -1;
                }
            }
            // `:76358-76363`.
            if i.vy != 0.0
                || (i.direction == -1 && i.vx > 0.0)
                || (i.direction == 1 && i.vx < 0.0)
            {
                s.counter = 0.0;
                s.row = 2;
                return;
            }
            // `:76364-76369`.
            if i.vx == 0.0 {
                s.counter = 0.0;
                s.row = 0;
                return;
            }
            // `:76370-76390`.
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
                s.counter = 0.0;
            }
        }
        // `case 256` in the jellyfish group (`NPC.cs:73916-73960`,
        // shared with 63/64/103/242 — d44's 63/64, d45/e14's 103 and
        // f27's 242 are the same machine, keep-one at the merge): the
        // `ai[1] == 1` pulse gate (`:73921`) lists 63/64/103/242 and
        // NOT 256, so the Fungo Fish ALWAYS rides the idle path —
        // counter++ then the <6/<12/<18 bands over rows 0/1/2 with
        // the row-3 tail resetting at >= 23 (`:73939-73959`). No
        // sprite/rotation write.
        FindFrameExtra5Kind::FungoFish => {
            // `:73939-73959`.
            s.counter += 1.0;
            if s.counter < 6.0 {
                s.row = 0;
                return;
            }
            if s.counter < 12.0 {
                s.row = 1;
                return;
            }
            if s.counter < 18.0 {
                s.row = 2;
                return;
            }
            s.row = 3;
            if s.counter >= 23.0 {
                s.counter = 0.0;
            }
        }
        // `case 257` (`NPC.cs:76179-76224`): the Anomura Fungus — the
        // zombie-walker shape on the 5-row sheet with the air pin at
        // row 4 and the FOUR-row walk cycle: the grounded ±1 flip
        // (`:76180-76190`), the air OR turn pins row 4 + zeroes the
        // counter (`:76191-76196`), the idle snaps row 0 (`:76197-
        // 76202`), the walk accumulates |vx| over the <8/<16/<24/<32
        // bands rows 0/1/2/3 with the >= 32 tail zeroing the counter
        // and HOLDING the row (`:76203-76223`).
        FindFrameExtra5Kind::AnomuraFungus => {
            // `:76180-76190`.
            if i.vy == 0.0 {
                if i.direction == 1 {
                    s.sprite_dir = 1;
                }
                if i.direction == -1 {
                    s.sprite_dir = -1;
                }
            }
            // `:76191-76196`.
            if i.vy != 0.0
                || (i.direction == -1 && i.vx > 0.0)
                || (i.direction == 1 && i.vx < 0.0)
            {
                s.counter = 0.0;
                s.row = 4;
                return;
            }
            // `:76197-76202`.
            if i.vx == 0.0 {
                s.counter = 0.0;
                s.row = 0;
                return;
            }
            // `:76203-76223`.
            s.counter += i.vx.abs();
            if s.counter < 8.0 {
                s.row = 0;
            } else if s.counter < 16.0 {
                s.row = 1;
            } else if s.counter < 24.0 {
                s.row = 2;
            } else if s.counter < 32.0 {
                s.row = 3;
            } else {
                s.counter = 0.0;
            }
        }
        // `case 258` (`NPC.cs:76225-76275`): the Mushi Ladybug — the
        // grounded ±1 flip (`:76226-76236`); the air OR turn branch
        // does NOT zero the counter — it ticks it (++ then > 1 steps
        // a row) over the rows 3..5 flight strip with the
        // `> 5 || < 3` clamp lifting ANY out-of-band row to 3
        // (`:76237-76249` — the first air tick from row 0 steps to 1
        // then clamps to 3); the idle snaps row 0 + zeroes the
        // counter (`:76250-76254`); the ground walk is the THREE-band
        // |vx| accumulator rows 0/1/2 with the >= 24 tail zeroing the
        // counter and HOLDING the row (`:76255-76274`).
        FindFrameExtra5Kind::MushiLadybug => {
            // `:76226-76236`.
            if i.vy == 0.0 {
                if i.direction == 1 {
                    s.sprite_dir = 1;
                }
                if i.direction == -1 {
                    s.sprite_dir = -1;
                }
            }
            // `:76237-76249`.
            if i.vy != 0.0
                || (i.direction == -1 && i.vx > 0.0)
                || (i.direction == 1 && i.vx < 0.0)
            {
                s.counter += 1.0;
                if s.counter > 1.0 {
                    s.row += 1;
                    s.counter = 0.0;
                }
                if s.row > 5 || s.row < 3 {
                    s.row = 3;
                }
            } else if i.vx == 0.0 {
                // `:76250-76254`.
                s.counter = 0.0;
                s.row = 0;
            } else {
                // `:76255-76274`.
                s.counter += i.vx.abs();
                if s.counter < 8.0 {
                    s.row = 0;
                } else if s.counter < 16.0 {
                    s.row = 1;
                } else if s.counter < 24.0 {
                    s.row = 2;
                } else {
                    s.counter = 0.0;
                }
            }
        }
        // `case 268` in the 250/264 strip group (`NPC.cs:72884-72897`
        // — 250 is the port's AngryNimbus, 264 the PlanterasTentacle,
        // both already carry the same machine in `npc.rs::find_frame`
        // `:8354-8364`; keep-one at the merge): counter++ then >= 6
        // steps a row + zeroes the counter, the row wraps at
        // `npcFrameCount[268]` = 4 back to 0. No sprite/rotation
        // write; velocity is ignored.
        FindFrameExtra5Kind::IchorSticker => {
            // `:72887-72896`.
            s.counter += 1.0;
            if s.counter >= 6.0 {
                s.row += 1;
                s.counter = 0.0;
            }
            if s.row >= kind.frame_count() {
                s.row = 0;
            }
        }
        // `case 269..277` in the Armored Bones group (`NPC.cs:75825-
        // 75882`, the 269..280 list — 278..280 ride the next slice):
        // the grounded ±1 flip (`:75839-
        // 75846`); the grounded idle snaps row 0 + zeroes the counter
        // (`:75847-75859` — the `type == 140` row-1 branch is DEAD
        // inside this case list, the header's skip); the walk lifts
        // any row < 2 to 2 FIRST (`:75861-75864` — the first walk
        // tick from idle jumps straight to row 2), then accumulates
        // `|vx| * 2` PLUS 1 (`:75865-75866`), the > 6 counter steps a
        // row (`:75867-75871`), wrapping at the sheet's full count
        // back to row 2 (`:75872-75875` — rows 0/1 are idle/air-only
        // on the walk); the AIR branch zeroes the counter and pins
        // row 0 (`:75877-75881`). No rotation write.
        FindFrameExtra5Kind::RustyArmoredBonesAxe
        | FindFrameExtra5Kind::RustyArmoredBonesFlail
        | FindFrameExtra5Kind::RustyArmoredBonesSword
        | FindFrameExtra5Kind::RustyArmoredBonesSwordNoArmor
        | FindFrameExtra5Kind::BlueArmoredBones
        | FindFrameExtra5Kind::BlueArmoredBonesMace
        | FindFrameExtra5Kind::BlueArmoredBonesNoPants
        | FindFrameExtra5Kind::BlueArmoredBonesSword
        | FindFrameExtra5Kind::HellArmoredBones => {
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(vx: f32, vy: f32) -> FrameInput {
        FrameInput { vx, vy, ..Default::default() }
    }

    fn rows_over(kind: FindFrameExtra5Kind, i: FrameInput, ticks: usize) -> Vec<u32> {
        let mut s = FrameState::default();
        let mut out = Vec::new();
        for _ in 0..ticks {
            find_frame_extra5(kind, &mut s, &i);
            out.push(s.row);
        }
        out
    }

    /// The frame-count table agrees with the port's verbatim
    /// `NPC_FRAME_COUNT` (`Main.cs:67082` initializer cross-check).
    #[test]
    fn f26s5_frame_counts_match_the_port_table() {
        let table = crate::render::entities::NPC_FRAME_COUNT;
        for k in [
            FindFrameExtra5Kind::ZombieMushroom,
            FindFrameExtra5Kind::ZombieMushroomHat,
            FindFrameExtra5Kind::FungoFish,
            FindFrameExtra5Kind::AnomuraFungus,
            FindFrameExtra5Kind::MushiLadybug,
            FindFrameExtra5Kind::IchorSticker,
            FindFrameExtra5Kind::RustyArmoredBonesAxe,
            FindFrameExtra5Kind::RustyArmoredBonesFlail,
            FindFrameExtra5Kind::RustyArmoredBonesSword,
            FindFrameExtra5Kind::RustyArmoredBonesSwordNoArmor,
            FindFrameExtra5Kind::BlueArmoredBones,
            FindFrameExtra5Kind::BlueArmoredBonesMace,
            FindFrameExtra5Kind::BlueArmoredBonesNoPants,
            FindFrameExtra5Kind::BlueArmoredBonesSword,
            FindFrameExtra5Kind::HellArmoredBones,
        ] {
            assert_eq!(
                k.frame_count(),
                table[k.vanilla_type() as usize],
                "type {}",
                k.vanilla_type()
            );
        }
    }

    /// 254/255 the mushroom zombies: the case-3 walker bands
    /// 0/1/1/2/2/1/1 then the reset tick HOLDS row 1 (`NPC.cs:76370-
    /// 76390` — the else writes no frame); the air pin is row 2
    /// (`:76361`). Expectation re-derived by mechanical replay of
    /// the C# order.
    #[test]
    fn f26s5_mushroom_zombies_walk_the_zombie_bands() {
        for k in [FindFrameExtra5Kind::ZombieMushroom, FindFrameExtra5Kind::ZombieMushroomHat] {
            let rows = rows_over(k, input(4.0, 0.0), 16);
            assert_eq!(rows, vec![0, 1, 1, 2, 2, 1, 1, 1, 0, 1, 1, 2, 2, 1, 1, 1]);
            let mut s = FrameState { row: 1, counter: 12.0, ..Default::default() };
            find_frame_extra5(k, &mut s, &input(4.0, 2.0));
            assert_eq!((s.row, s.counter), (2, 0.0), "the air pin row 2");
            // The facing-against-motion turn pins row 2 on the ground too.
            let mut s = FrameState::default();
            let i = FrameInput { vx: -1.0, vy: 0.0, direction: 1 };
            find_frame_extra5(k, &mut s, &i);
            assert_eq!((s.row, s.counter), (2, 0.0), "the turn pin");
            // The grounded flip writes only on direction +/-1.
            let mut s = FrameState { sprite_dir: -1, ..Default::default() };
            let i = FrameInput { vx: 1.0, vy: 0.0, direction: 0 };
            find_frame_extra5(k, &mut s, &i);
            assert_eq!(s.sprite_dir, -1, "direction 0 leaves the flip");
        }
    }

    /// 256 the Fungo Fish: ALWAYS the idle bands (the `ai[1]` pulse
    /// gate `NPC.cs:73921` excludes 256) — the 23-tick cycle
    /// 0x5/1x6/2x6/3x6 (`:73939-73959`).
    #[test]
    fn f26s5_fungo_fish_pulses_the_idle_bands_only() {
        let rows = rows_over(FindFrameExtra5Kind::FungoFish, input(2.0, -1.0), 24);
        assert_eq!(
            rows,
            vec![0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 0]
        );
        let mut s = FrameState { sprite_dir: -1, ..Default::default() };
        find_frame_extra5(FindFrameExtra5Kind::FungoFish, &mut s, &input(-3.0, 2.0));
        assert_eq!(s.sprite_dir, -1, "no sprite write");
    }

    /// 257 the Anomura Fungus: the FOUR-row walk cycle 0/1/2/3 with
    /// the reset-tick row-HOLD and the air pin at row 4 (`NPC.cs:
    /// 76191-76223`).
    #[test]
    fn f26s5_anomura_fungus_walks_four_rows() {
        let rows = rows_over(FindFrameExtra5Kind::AnomuraFungus, input(4.0, 0.0), 16);
        assert_eq!(rows, vec![0, 1, 1, 2, 2, 3, 3, 3, 0, 1, 1, 2, 2, 3, 3, 3]);
        let mut s = FrameState { row: 2, counter: 9.0, ..Default::default() };
        find_frame_extra5(FindFrameExtra5Kind::AnomuraFungus, &mut s, &input(4.0, -2.0));
        assert_eq!((s.row, s.counter), (4, 0.0), "the air pin row 4");
    }

    /// 258 the Mushi Ladybug: the air strip ticks the counter (NOT
    /// zeroed) over rows 3..5 — from row 0 the first air tick steps
    /// to 1 then the `< 3` clamp lifts to 3 (`NPC.cs:76237-76249`);
    /// the ground walk is the THREE-band cycle with the >= 24
    /// reset-tick row-HOLD (`:76255-76274`).
    #[test]
    fn f26s5_mushi_ladybug_flies_rows_three_to_five() {
        let rows = rows_over(FindFrameExtra5Kind::MushiLadybug, input(4.0, -2.0), 12);
        assert_eq!(rows, vec![3, 4, 4, 5, 5, 3, 3, 4, 4, 5, 5, 3]);
        let rows = rows_over(FindFrameExtra5Kind::MushiLadybug, input(4.0, 0.0), 12);
        assert_eq!(rows, vec![0, 1, 1, 2, 2, 2, 0, 1, 1, 2, 2, 2]);
        // The idle snaps row 0 + zeroes the counter.
        let mut s = FrameState { row: 4, counter: 1.0, ..Default::default() };
        find_frame_extra5(FindFrameExtra5Kind::MushiLadybug, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (0, 0.0));
    }

    /// 268 the Ichor Sticker: the shared 6-tick strip over the FULL
    /// 4 rows (`NPC.cs:72887-72896` — the 250/264 twins in `npc.rs`
    /// are the same machine); velocity is ignored, no sprite write.
    #[test]
    fn f26s5_ichor_sticker_cycles_the_six_tick_strip() {
        let rows = rows_over(FindFrameExtra5Kind::IchorSticker, input(3.0, -2.0), 24);
        assert_eq!(
            rows,
            vec![0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 0]
        );
        let mut s = FrameState { sprite_dir: -1, ..Default::default() };
        find_frame_extra5(FindFrameExtra5Kind::IchorSticker, &mut s, &input(-3.0, 0.0));
        assert_eq!(s.sprite_dir, -1, "no sprite write");
    }

    /// 269..272 the rusty Armored Bones: the first walk tick lifts
    /// row 0 to 2 THEN steps (`NPC.cs:75861-75871` — vx 4 gives
    /// |vx|*2+1 = 9 > 6 every tick, so the walk strides a row per
    /// tick from row 3); the wrap returns to row 2, not 0
    /// (`:75872-75875`); the air pins row 0 (`:75877-75881`).
    #[test]
    fn f26s5_rusty_armored_bones_stride_from_row_two() {
        for k in [
            FindFrameExtra5Kind::RustyArmoredBonesAxe,
            FindFrameExtra5Kind::RustyArmoredBonesFlail,
            FindFrameExtra5Kind::RustyArmoredBonesSword,
            FindFrameExtra5Kind::RustyArmoredBonesSwordNoArmor,
        ] {
            let rows = rows_over(k, input(4.0, 0.0), 13);
            assert_eq!(rows, vec![3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 2]);
            // The slow walk: vx 0.5 gives 2.0/tick — the lift to 2
            // holds three ticks, the 4th steps (counter 8 > 6).
            let rows = rows_over(k, input(0.5, 0.0), 8);
            assert_eq!(rows, vec![2, 2, 2, 3, 3, 3, 3, 4]);
            // The air pin.
            let mut s = FrameState { row: 7, counter: 3.0, ..Default::default() };
            find_frame_extra5(k, &mut s, &input(4.0, -1.0));
            assert_eq!((s.row, s.counter), (0, 0.0), "the air pin row 0");
            // The grounded idle snaps row 0 (the dead 140 branch is
            // not carried).
            let mut s = FrameState { row: 5, counter: 3.0, ..Default::default() };
            find_frame_extra5(k, &mut s, &input(0.0, 0.0));
            assert_eq!((s.row, s.counter), (0, 0.0), "the idle row 0");
        }
    }

    /// 273..277 the blue/hell Armored Bones ride the SAME group
    /// machine — the 275 NoPants' 16-row sheet is the ONE count
    /// variant: its walk reaches row 15 before the wrap to 2, the
    /// 15-row twins wrap at 14 (`NPC.cs:75872-75875` reads
    /// `Main.npcFrameCount[type]`; `Main.cs:67111` slot 5 = 16).
    #[test]
    fn f26s5_blue_hell_bones_wrap_at_their_own_counts() {
        for k in [
            FindFrameExtra5Kind::BlueArmoredBones,
            FindFrameExtra5Kind::BlueArmoredBonesMace,
            FindFrameExtra5Kind::BlueArmoredBonesSword,
            FindFrameExtra5Kind::HellArmoredBones,
        ] {
            let rows = rows_over(k, input(4.0, 0.0), 13);
            assert_eq!(rows, vec![3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 2]);
        }
        let rows = rows_over(FindFrameExtra5Kind::BlueArmoredBonesNoPants, input(4.0, 0.0), 15);
        assert_eq!(rows, vec![3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 2, 3]);
    }

    /// The 15-kind strip sweep: a 5x3x3 vx/vy/direction input grid,
    /// 60 ticks each — every row stays inside its sheet (the f27
    /// precedent).
    #[test]
    fn f26s5_fifteen_kind_strip_sweep_stays_inside_the_sheets() {
        let kinds = [
            FindFrameExtra5Kind::ZombieMushroom,
            FindFrameExtra5Kind::ZombieMushroomHat,
            FindFrameExtra5Kind::FungoFish,
            FindFrameExtra5Kind::AnomuraFungus,
            FindFrameExtra5Kind::MushiLadybug,
            FindFrameExtra5Kind::IchorSticker,
            FindFrameExtra5Kind::RustyArmoredBonesAxe,
            FindFrameExtra5Kind::RustyArmoredBonesFlail,
            FindFrameExtra5Kind::RustyArmoredBonesSword,
            FindFrameExtra5Kind::RustyArmoredBonesSwordNoArmor,
            FindFrameExtra5Kind::BlueArmoredBones,
            FindFrameExtra5Kind::BlueArmoredBonesMace,
            FindFrameExtra5Kind::BlueArmoredBonesNoPants,
            FindFrameExtra5Kind::BlueArmoredBonesSword,
            FindFrameExtra5Kind::HellArmoredBones,
        ];
        for k in kinds {
            let count = k.frame_count();
            for vx in [-4.0f32, -1.0, 0.0, 1.0, 4.0] {
                for vy in [-3.0f32, 0.0, 3.0] {
                    for direction in [-1i32, 0, 1] {
                        let i = FrameInput { vx, vy, direction };
                        let mut s = FrameState::default();
                        for tick in 0..60 {
                            find_frame_extra5(k, &mut s, &i);
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

    /// The seam maps exactly the batch's vanilla types and nothing
    /// else.
    #[test]
    fn f26s5_seam_covers_exactly_the_slice() {
        for t in 0..697i16 {
            let k = find_frame_kind_extra5(t);
            let claimed = [
                254, 255, 256, 257, 258, 268, 269, 270, 271, 272, 273, 274, 275, 276, 277,
            ];
            assert_eq!(k.is_some(), claimed.contains(&t), "type {t}");
            if let Some(k) = k {
                assert_eq!(k.vanilla_type(), t);
            }
        }
    }
}
