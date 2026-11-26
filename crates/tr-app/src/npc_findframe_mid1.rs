//! W4-d46 FindFrame mid-band slice 1 — the FIRST 12 missing
//! `NPC.FindFrame` cases in the vanilla-id band 150-350
//! (`NPC.cs:66684-77370`) that neither `npc.rs::find_frame` nor the
//! wave-A (`npc_findframe_a.rs`) / wave-B (`npc_findframe_b.rs`) modules
//! carry yet. The census (ids 150-350, ascending, skipping every id that
//! already has a ported arm — the same census d47's header cites) yields
//! these twelve, landed in two batches of six:
//!
//! | vanilla | kind            | C# case            | rows | batch |
//! |---------|-----------------|--------------------|------|-------|
//! | 150     | IceBat          | `:74786-74822`     | 4    | 1     |
//! | 151     | Lavabat         | `:74786-74822`     | 4    | 1     |
//! | 152     | GiantFlyingFox  | `:74786-74822`     | 4    | 1     |
//! | 156     | RedDevil        | `:73903-73915`     | 5    | 1     |
//! | 157     | Arapaima        | `:74134-74180`     | 6    | 1     |
//! | 158     | VampireBat      | `:74786-74822`     | 4    | 1     |
//! | 159     | Vampire         | humanoid `:68569`  | 15   | 2     |
//! | 160     | Truffle         | humanoid `:68570`  | 23   | 2     |
//! | 162     | Frankenstein    | humanoid `:68571`  | 15   | 2     |
//! | 163     | BlackRecluse    | `:73227-73267`     | 8    | 2     |
//! | 164     | WallCreeper     | `:73083-73111`     | 5    | 2     |
//! | 165     | WallCreeperWall | `:73112-73145`     | 4    | 2     |
//!
//! Wave split (W4): d46 (this module) takes the first 12; d47
//! (`npc_findframe_mid2.rs`) takes the next 12 (166 SwampThing onward);
//! d48/d49 take the high band (d48's file is `npc_findframe_c.rs` — this
//! module follows d47's `mid1`/`mid2` naming to stay collision-free).
//! The frost legion 338-352, the pumpkin family 305-330, the martians,
//! the DD2/lunar families and the bosses with in-flight AI machines keep
//! their own `npc.rs` arms and are NOT re-ported here. The tortoises
//! 153/154 have no `FindFrame` case at all — their frames ride the
//! aiStyle-39 post-switch tail (`NPC.cs:77292-77368`), which
//! `npc_ai_style39.rs` documents as caller-owned; they are not "missing
//! cases" and stay out of the census.
//!
//! Shape: the kinds are NOT in `NpcKind` yet (their roster rows ride the
//! W4 `npc-kinds` waves), so — like `npc_findframe_b.rs` — the frame
//! machines land first as pure functions over a small
//! [`FrameState`]/[`FrameInput`] pair. The wiring is a `cs_id`-keyed hook
//! at the head of `npc.rs::find_frame` ([`find_frame_kind_for_cs_id`] +
//! `Npc::find_frame_wave_mid1`): NO current kind's `cs_id` lands in this
//! set (the bestiary's only in-band id is 161 ZombieEskimo, a wave-A
//! walker), so the hook is stream-inert today and auto-activates when
//! the roster wave adds the kinds. No RNG is touched — see the dust
//! note below — so the wiring cannot move a stream.
//!
//! Row convention: the port tracks frame ROW indices, not pixels — the
//! C#'s `num` (the sheet height / `Main.npcFrameCount[type]`,
//! `NPC.cs:66694`) cancels out of every comparison, exactly as in
//! `npc.rs::find_frame`'s existing arms. The per-kind row counts are
//! `Main.npcFrameCount` (`Main.cs:67082`'s 697-entry initializer), counted
//! entry-by-entry in [`FindFrameMid1Kind::frame_count`] and cross-checked
//! against the port's `render::entities::NPC_FRAME_COUNT` table.
//!
//! Deliberate skips (recorded in docs/ANALYSIS.md "W4-d46 FindFrame
//! mid-band slice 1"): the bat group's dust spawns — the Lavabat's
//! unconditional `Dust.NewDust(6)` (`NPC.cs:74791-74795`) and the IceBat's
//! `Main.rand.Next(10) == 0`-gated `Dust.NewDust(67)` (`:74796-74802`).
//! The port's `find_frame` is RNG-free by construction (the
//! `npc_findframe_b.rs` precedent); adding the IceBat's `Next(10)` draw
//! would offset every downstream stream, so both dust arms are count-only
//! visual skips. The humanoid arm's town `ai[0]` pose zones
//! (`NPC.cs:68662-69488`) are not carried — the Truffle's town machine
//! does not model those states yet (the `npc.rs::humanoid_town_frame`
//! precedent for the existing town kinds); the Vampire/Frankenstein are
//! aiStyle-3 fighters whose turn counter leaves `ai[0]` at 0/1
//! (`NPC.cs:57381-57397`). The disabled profile-texture block
//! (`NPC.cs:68637`, `if (false && …)`) and the `IsTownSlime` flip
//! inversion (`:68657-68660` — none of these kinds is a town slime) are
//! likewise not carried.

// The `find_frame` wiring is a `cs_id` hook that is inert until the
// roster wave adds the kinds — the `npc_findframe_b.rs` precedent.
#![allow(dead_code)]

/// The wave mid-1 kinds, keyed by their VANILLA `NPC.type`
/// (`ID/NPCID.cs`'s `public const short` rows — the port-local `NpcKind`
/// ids do not exist for them yet).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FindFrameMid1Kind {
    /// `NPCID.IceBat` = 150 (`NPCID.cs:10977`) — the bat flap group.
    IceBat,
    /// `NPCID.Lavabat` = 151 (`NPCID.cs:10979`) — the bat flap group.
    Lavabat,
    /// `NPCID.GiantFlyingFox` = 152 (`NPCID.cs:10981`) — the bat flap group.
    GiantFlyingFox,
    /// `NPCID.RedDevil` = 156 (`NPCID.cs:10989`) — `case 156`.
    RedDevil,
    /// `NPCID.Arapaima` = 157 (`NPCID.cs:10991`) — `case 157`.
    Arapaima,
    /// `NPCID.VampireBat` = 158 (`NPCID.cs:10993`) — the bat flap group.
    VampireBat,
    /// `NPCID.Vampire` = 159 (`NPCID.cs:10995`) — the humanoid-group arm.
    Vampire,
    /// `NPCID.Truffle` = 160 (`NPCID.cs:10997`) — the humanoid-group arm
    /// (the town member, `ExtraFramesCount` 7).
    Truffle,
    /// `NPCID.Frankenstein` = 162 (`NPCID.cs:11001`) — the humanoid-group arm.
    Frankenstein,
    /// `NPCID.BlackRecluse` = 163 (`NPCID.cs:11003`) — `case 163`.
    BlackRecluse,
    /// `NPCID.WallCreeper` = 164 (`NPCID.cs:11005`) — `case 164`.
    WallCreeper,
    /// `NPCID.WallCreeperWall` = 165 (`NPCID.cs:11007`) — `case 165`.
    WallCreeperWall,
}

impl FindFrameMid1Kind {
    /// The vanilla `NPC.type` (`ID/NPCID.cs`'s const rows).
    pub fn vanilla_type(self) -> i16 {
        match self {
            Self::IceBat => 150,
            Self::Lavabat => 151,
            Self::GiantFlyingFox => 152,
            Self::RedDevil => 156,
            Self::Arapaima => 157,
            Self::VampireBat => 158,
            Self::Vampire => 159,
            Self::Truffle => 160,
            Self::Frankenstein => 162,
            Self::BlackRecluse => 163,
            Self::WallCreeper => 164,
            Self::WallCreeperWall => 165,
        }
    }

    /// `Main.npcFrameCount[type]` — the row counts counted entry-by-entry
    /// out of the 697-entry initializer (`Main.cs:67082-67153`; entry
    /// `type` sits on line `67084 + type / 10`, slot `type % 10`).
    pub fn frame_count(self) -> u32 {
        match self {
            // Main.cs:67099 (entries 150-159): 150/151/152 = 4, 156 = 5,
            // 157 = 6, 158 = 4.
            Self::IceBat => 4,
            Self::Lavabat => 4,
            Self::GiantFlyingFox => 4,
            Self::RedDevil => 5,
            Self::Arapaima => 6,
            Self::VampireBat => 4,
            // Main.cs:67099-67100 (entries 159-165): 159 = 15, 160 = 23,
            // 162 = 15, 163 = 8, 164 = 5, 165 = 4.
            Self::Vampire => 15,
            Self::Truffle => 23,
            Self::Frankenstein => 15,
            Self::BlackRecluse => 8,
            Self::WallCreeper => 5,
            Self::WallCreeperWall => 4,
        }
    }
}

/// The `cs_id` → [`FindFrameMid1Kind`] mapping the `npc.rs::find_frame`
/// hook reads. Returns `None` for every id outside this wave's set — so
/// the hook is inert for all current kinds (none has a `cs_id` in the
/// set) and fires only once the roster wave adds them.
pub fn find_frame_kind_for_cs_id(cs_id: u16) -> Option<FindFrameMid1Kind> {
    Some(match cs_id {
        150 => FindFrameMid1Kind::IceBat,
        151 => FindFrameMid1Kind::Lavabat,
        152 => FindFrameMid1Kind::GiantFlyingFox,
        156 => FindFrameMid1Kind::RedDevil,
        157 => FindFrameMid1Kind::Arapaima,
        158 => FindFrameMid1Kind::VampireBat,
        159 => FindFrameMid1Kind::Vampire,
        160 => FindFrameMid1Kind::Truffle,
        162 => FindFrameMid1Kind::Frankenstein,
        163 => FindFrameMid1Kind::BlackRecluse,
        164 => FindFrameMid1Kind::WallCreeper,
        165 => FindFrameMid1Kind::WallCreeperWall,
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
    /// `NPC.rotation` — only the bat group + RedDevil write it.
    pub rotation: f32,
}

/// The per-tick inputs the arms read: `velocity`, `direction`, `wet` and
/// the `ai` slots. `wet` is the Arapaima's dry double-increment gate
/// (`NPC.cs:74138`); `ai` is carried for the humanoid arm's future town
/// pose zones (not read today — the zones are a documented skip).
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

/// `NPC.FindFrame` (`NPC.cs:66684`) for the mid-1 kinds — one tick.
pub fn find_frame_mid1(kind: FindFrameMid1Kind, s: &mut FrameState, i: &FrameInput) {
    match kind {
        // The bat flap group (`case 150/151/152/158/226`,
        // `NPC.cs:74786-74822`): the velocity-sign flip (0 leaves it),
        // the `vx * 0.1` tilt, and the 6-tick strip over the full 4 rows.
        // The Lavabat's unconditional dust (`:74791-74795`) and the
        // IceBat's `Next(10)`-gated dust (`:74796-74802`) are count-only
        // visual skips (the module is RNG-free — see the header). 226
        // FlyingSnake is a later wave's kind, not carried here.
        FindFrameMid1Kind::IceBat
        | FindFrameMid1Kind::Lavabat
        | FindFrameMid1Kind::GiantFlyingFox
        | FindFrameMid1Kind::VampireBat => {
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
            if s.row >= 4 {
                s.row = 0;
            }
        }
        // `case 156` (`NPC.cs:73903-73915`): the RedDevil faces by
        // `direction` (unconditional), tilts `vx * 0.1`, and the counter
        // wraps at 20 with the row read off `(int)counter / 4` (rows
        // 0..4 of the 5-row sheet).
        FindFrameMid1Kind::RedDevil => {
            s.sprite_dir = i.direction;
            s.rotation = i.vx * 0.1;
            s.counter += 1.0;
            if s.counter >= 20.0 {
                s.counter = 0.0;
            }
            s.row = (s.counter as i32 / 4) as u32;
        }
        // `case 157` (`NPC.cs:74134-74180`): the Arapaima faces by
        // `direction`; the counter ticks once wet, TWICE dry (`:74138`);
        // the `num50 = 5` bands pick rows 0/1/2/1/3/4/5/4 and the `else`
        // resets the counter WITHOUT writing the row (a vanilla quirk —
        // the row holds 4 for that tick). The `(double)` casts
        // (`:74143`…) compare identically in f32 for these literals.
        FindFrameMid1Kind::Arapaima => {
            s.sprite_dir = i.direction;
            s.counter += 1.0;
            if !i.wet {
                s.counter += 1.0;
            }
            let num50 = 5.0f32;
            if s.counter < num50 {
                s.row = 0;
            } else if s.counter < num50 * 2.0 {
                s.row = 1;
            } else if s.counter < num50 * 3.0 {
                s.row = 2;
            } else if s.counter < num50 * 4.0 {
                s.row = 1;
            } else if s.counter < num50 * 5.0 {
                s.row = 3;
            } else if s.counter < num50 * 6.0 {
                s.row = 4;
            } else if s.counter < num50 * 7.0 {
                s.row = 5;
            } else if s.counter < num50 * 8.0 {
                s.row = 4;
            } else {
                s.counter = 0.0;
            }
        }
        // The humanoid-group case list (`NPC.cs:68569-68571` carries
        // 159/160/162) → the shared fighter/town walk
        // (`NPC.cs:68647-69655`). None is `IsTownSlime`, so `num108` = 6
        // and `num109` = row 2 for all three. `num57` =
        // `isLikeATownNPC ? ExtraFramesCount[type] : 0` (`:68635`): the
        // Truffle (160) is `townNPC` (`SetDefaults` `:11005`) with
        // `ExtraFramesCount[160]` = 7 (`NPCID.cs:10464`), so its walk
        // wraps at 23 - 7 = 16; the Vampire (159) and Frankenstein (162)
        // are not town-like, so `num57` = 0 and they wrap at the full 15.
        // The airborne pin list (`:69647`) carries 159 (row 0) but NOT
        // 160/162 (the group's row 1).
        FindFrameMid1Kind::Vampire
        | FindFrameMid1Kind::Truffle
        | FindFrameMid1Kind::Frankenstein => {
            let extra = if kind == FindFrameMid1Kind::Truffle { 7 } else { 0 };
            let air_row0 = kind == FindFrameMid1Kind::Vampire;
            humanoid_frame_mid1(s, i, kind.frame_count(), extra, air_row0);
        }
        // `case 163` (`NPC.cs:73227-73267`): the BlackRecluse. The air
        // pins the counter to -12 and the row to 5 (rising) / 6
        // (falling); the ground faces by `direction`, accumulates
        // `|vx| * 1.1`, and the bands pick rows 6/7/1/2/3 with the
        // `else` resetting the counter WITHOUT writing the row (the
        // vanilla quirk — the row holds for that tick). Rows 0/4 of the
        // 8-row sheet are never written by this machine.
        FindFrameMid1Kind::BlackRecluse => {
            if i.vy != 0.0 {
                s.counter = -12.0;
                s.row = if i.vy < 0.0 { 5 } else { 6 };
            } else {
                s.sprite_dir = i.direction;
                s.counter += i.vx.abs() * 1.1;
                if s.counter < -6.0 {
                    s.row = 6;
                } else if s.counter < 0.0 {
                    s.row = 7;
                } else if s.counter < 6.0 {
                    s.row = 1;
                } else if s.counter < 12.0 {
                    s.row = 2;
                } else if s.counter < 18.0 {
                    s.row = 3;
                } else {
                    s.counter = 0.0;
                }
            }
        }
        // `case 164` (`NPC.cs:73083-73111`, shared with 236/239/530):
        // the WallCreeper. The air zeroes the counter and pins row 4
        // (rising) / 0 (falling); the ground faces by `direction`,
        // accumulates `|vx| * 1.1`, and the WHILE loop steps a row per
        // 6 (keeping the remainder, possibly several rows a tick),
        // wrapping past row 3 back to 0.
        FindFrameMid1Kind::WallCreeper => {
            if i.vy != 0.0 {
                s.counter = 0.0;
                s.row = if i.vy < 0.0 { 4 } else { 0 };
            } else {
                s.sprite_dir = i.direction;
                s.counter += i.vx.abs() * 1.1;
                while s.counter >= 6.0 {
                    s.counter -= 6.0;
                    s.row += 1;
                    if s.row > 3 {
                        s.row = 0;
                    }
                }
            }
        }
        // `case 165` (`NPC.cs:73112-73145`, shared with 237/238/240/531):
        // the WallCreeperWall. `num194` = 0.5 (the 0.4 is type 531's,
        // not this wave's); the counter accumulates
        // `(|vx| + |vy|) * 0.5` and the bands pick rows 0/1/2/3 with the
        // `else` resetting the counter WITHOUT writing the row (the
        // vanilla quirk). No sprite/rotation write.
        FindFrameMid1Kind::WallCreeperWall => {
            let num194 = 0.5f32;
            s.counter += (i.vx.abs() + i.vy.abs()) * num194;
            if s.counter < 6.0 {
                s.row = 0;
            } else if s.counter < 12.0 {
                s.row = 1;
            } else if s.counter < 18.0 {
                s.row = 2;
            } else if s.counter < 24.0 {
                s.row = 3;
            } else {
                s.counter = 0.0;
            }
        }
    }
}

/// The humanoid-group walk (`NPC.cs:68647-69655`) for the mid-1
/// members: the grounded flip (`:68649-68656` — only `direction` ±1
/// writes), the idle snap to row 0 (`:69515-69519` — none of these is
/// the 140/489 idle-row-1 quirk), the `|vx| * 2 + 1` accumulator
/// (`:69552-69553`) stepping every > 6 (`num108` = 6) over rows
/// 2..count-1-extra (`num109` = row 2, the wrap at
/// `npcFrameCount - num57`, `:69581`), and the airborne row 1 → pin-0
/// tail (`:69642-69650`). The town `ai[0]` pose zones
/// (`:68662-69488`) are a documented skip (see the header).
fn humanoid_frame_mid1(
    s: &mut FrameState,
    i: &FrameInput,
    count: u32,
    extra: u32,
    air_row0: bool,
) {
    if i.vy == 0.0 {
        if i.direction == 1 {
            s.sprite_dir = 1;
        }
        if i.direction == -1 {
            s.sprite_dir = -1;
        }
        if i.vx == 0.0 {
            // `NPC.cs:69515-69519`: the idle row 0.
            s.row = 0;
            s.counter = 0.0;
        } else {
            // `NPC.cs:69552-69553`: the accumulator.
            s.counter += i.vx.abs() * 2.0;
            s.counter += 1.0;
            // `:69572-69575`: the walk band's floor (row 2).
            if s.row < 2 {
                s.row = 2;
            }
            // `:69576-69580`: the step.
            if s.counter > 6.0 {
                s.row += 1;
                s.counter = 0.0;
            }
            // `:69581-69584`: the wrap at `npcFrameCount - num57`.
            if s.row >= count.saturating_sub(extra) {
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

    fn rows_over(kind: FindFrameMid1Kind, i: FrameInput, ticks: usize) -> Vec<u32> {
        let mut s = FrameState::default();
        let mut out = Vec::new();
        for _ in 0..ticks {
            find_frame_mid1(kind, &mut s, &i);
            out.push(s.row);
        }
        out
    }

    /// The 12 `Main.npcFrameCount` rows, counted entry-by-entry out of
    /// the `Main.cs:67082` initializer and cross-checked against the
    /// port's `NPC_FRAME_COUNT` table.
    #[test]
    fn the_mid1_frame_counts_match_the_main_initializer() {
        let table = [
            (FindFrameMid1Kind::IceBat, 150, 4),
            (FindFrameMid1Kind::Lavabat, 151, 4),
            (FindFrameMid1Kind::GiantFlyingFox, 152, 4),
            (FindFrameMid1Kind::RedDevil, 156, 5),
            (FindFrameMid1Kind::Arapaima, 157, 6),
            (FindFrameMid1Kind::VampireBat, 158, 4),
            (FindFrameMid1Kind::Vampire, 159, 15),
            (FindFrameMid1Kind::Truffle, 160, 23),
            (FindFrameMid1Kind::Frankenstein, 162, 15),
            (FindFrameMid1Kind::BlackRecluse, 163, 8),
            (FindFrameMid1Kind::WallCreeper, 164, 5),
            (FindFrameMid1Kind::WallCreeperWall, 165, 4),
        ];
        for (kind, vanilla, count) in table {
            assert_eq!(kind.vanilla_type(), vanilla, "{kind:?} id");
            assert_eq!(kind.frame_count(), count, "{kind:?} npcFrameCount");
            assert_eq!(
                crate::render::entities::npc_rows(vanilla as u16),
                count,
                "{kind:?} vs the port's NPC_FRAME_COUNT"
            );
            assert_eq!(find_frame_kind_for_cs_id(vanilla as u16), Some(kind), "{kind:?} cs_id map");
        }
    }

    /// The `cs_id` hook maps exactly the twelve ids and nothing else —
    /// the inertness seam (no current kind's `cs_id` is in the set).
    #[test]
    fn the_cs_id_hook_maps_only_the_twelve_mid1_ids() {
        for id in 0u16..=697 {
            let mapped = find_frame_kind_for_cs_id(id);
            let in_set = matches!(
                id,
                150 | 151 | 152 | 156 | 157 | 158 | 159 | 160 | 162 | 163 | 164 | 165
            );
            assert_eq!(mapped.is_some(), in_set, "cs_id {id}");
        }
    }

    /// The bat flap group (`NPC.cs:74786-74822`): the velocity-sign flip
    /// (0 leaves it), the 0.1 tilt, the 6-tick strip over the full 4
    /// rows. The dust spawns are count-only skips (no RNG).
    #[test]
    fn the_bat_group_flaps_four_rows_every_six_ticks() {
        for kind in [
            FindFrameMid1Kind::IceBat,
            FindFrameMid1Kind::Lavabat,
            FindFrameMid1Kind::GiantFlyingFox,
            FindFrameMid1Kind::VampireBat,
        ] {
            let mut s = FrameState::default();
            let i = input(-2.0, 0.0);
            find_frame_mid1(kind, &mut s, &i);
            assert_eq!(s.sprite_dir, -1, "{kind:?} flip");
            assert_eq!(s.rotation, -0.2, "{kind:?} tilt");
            assert_eq!(
                rows_over(kind, i, 24),
                [
                    0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 0
                ],
                "{kind:?} strip"
            );
            // vx == 0 leaves the flip alone.
            let mut s = FrameState { sprite_dir: -1, ..Default::default() };
            find_frame_mid1(kind, &mut s, &input(0.0, 0.0));
            assert_eq!(s.sprite_dir, -1, "{kind:?} zero-velocity flip hold");
        }
    }

    /// The RedDevil (`NPC.cs:73903-73915`): the `direction` flip, the
    /// 0.1 tilt, the 20-tick counter wrap with the row off
    /// `(int)counter / 4` (rows 0..4).
    #[test]
    fn the_red_devil_reads_its_row_off_the_twenty_tick_counter() {
        let mut i = input(3.0, 0.0);
        i.direction = -1;
        let mut s = FrameState::default();
        find_frame_mid1(FindFrameMid1Kind::RedDevil, &mut s, &i);
        assert_eq!(s.sprite_dir, -1);
        assert_eq!(s.rotation, 0.3);
        // counter 1..20 → rows 0,0,0,1,1,1,1,2,2,2,2,3,3,3,3,4,4,4,4,0.
        assert_eq!(
            rows_over(FindFrameMid1Kind::RedDevil, i, 20),
            [0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 0]
        );
    }

    /// The Arapaima (`NPC.cs:74134-74180`): the dry double-increment,
    /// the `num50 = 5` bands (rows 0/1/2/1/3/4/5/4), and the `else`
    /// counter-reset that leaves the row at 4 for one tick.
    #[test]
    fn the_arapaima_doubles_its_clock_dry_and_holds_the_reset_row() {
        // Dry: +2 a tick. The 5-wide bands hold 2 or 3 even counters:
        // 2,4 → 0; 6,8 → 1; 10,12,14 → 2; 16,18 → 1; 20,22,24 → 3;
        // 26,28 → 4; 30,32,34 → 5; 36,38 → 4; 40 hits the else-reset
        // (row holds 4) and the cycle restarts at 2.
        let dry = input(0.0, 0.0);
        assert_eq!(
            rows_over(FindFrameMid1Kind::Arapaima, dry, 22),
            [0, 0, 1, 1, 2, 2, 2, 1, 1, 3, 3, 3, 4, 4, 5, 5, 5, 4, 4, 4, 0, 0]
        );
        // The reset tick (counter hits 40): the row holds 4, counter 0.
        let mut s = FrameState::default();
        for _ in 0..16 {
            find_frame_mid1(FindFrameMid1Kind::Arapaima, &mut s, &dry);
        }
        // After 16 dry ticks counter = 32 → row 5 (the [30,35) band);
        // tick 17 → 34 row 5; tick 18 → 36 row 4; tick 19 → 38 row 4;
        // tick 20 → 40 reset.
        assert_eq!((s.row, s.counter), (5, 32.0));
        for _ in 0..4 {
            find_frame_mid1(FindFrameMid1Kind::Arapaima, &mut s, &dry);
        }
        assert_eq!((s.row, s.counter), (4, 0.0), "the else holds row 4");
        // Wet: +1 a tick — the bands stretch to twice the length.
        let mut wet = input(0.0, 0.0);
        wet.wet = true;
        let mut s = FrameState::default();
        for _ in 0..5 {
            find_frame_mid1(FindFrameMid1Kind::Arapaima, &mut s, &wet);
        }
        assert_eq!((s.row, s.counter), (1, 5.0), "wet ticks once");
        // The direction flip.
        let mut i = input(0.0, 0.0);
        i.direction = -1;
        let mut s = FrameState::default();
        find_frame_mid1(FindFrameMid1Kind::Arapaima, &mut s, &i);
        assert_eq!(s.sprite_dir, -1);
    }

    /// The humanoid group's idle/air split: the Vampire's air row is
    /// pinned to 0 (`NPC.cs:69647`), the Truffle/Frankenstein keep the
    /// group's row 1; the idle stands row 0; the airborne tick never
    /// writes `spriteDirection`.
    #[test]
    fn the_humanoid_group_idle_and_air_rows_follow_the_pin_list() {
        let mut s = FrameState { sprite_dir: -1, ..Default::default() };
        find_frame_mid1(FindFrameMid1Kind::Vampire, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (0, 0.0));
        // The idle tick ran with direction 1 → the grounded flip wrote
        // +1; the air tick itself never writes the flip.
        find_frame_mid1(FindFrameMid1Kind::Vampire, &mut s, &input(0.0, 2.0));
        assert_eq!((s.row, s.counter, s.sprite_dir), (0, 0.0, 1));
        for kind in [FindFrameMid1Kind::Truffle, FindFrameMid1Kind::Frankenstein] {
            find_frame_mid1(kind, &mut s, &input(0.0, 1.0));
            assert_eq!(s.row, 1, "{kind:?} air row");
        }
    }

    /// The Vampire/Frankenstein walk (`NPC.cs:69552-69584`): `|vx|*2+1`
    /// = 3 a tick at vx 1, the step at > 6 (every 3rd tick), the floor
    /// row 2 and the wrap at the full 15 back to 2 (num57 = 0).
    #[test]
    fn the_vampire_walk_wraps_at_the_full_fifteen_rows() {
        for kind in [FindFrameMid1Kind::Vampire, FindFrameMid1Kind::Frankenstein] {
            let mut s = FrameState::default();
            let i = input(1.0, 0.0);
            let mut rows = Vec::new();
            for _ in 0..6 {
                find_frame_mid1(kind, &mut s, &i);
                rows.push(s.row);
            }
            assert_eq!(rows, [2, 2, 3, 3, 3, 4], "{kind:?}");
            // Row 14 + one step = 15 → the wrap to 2.
            s.row = 14;
            s.counter = 6.0;
            find_frame_mid1(kind, &mut s, &i);
            assert_eq!(s.row, 2, "{kind:?} wrap");
        }
    }

    /// The Truffle walk (`NPC.cs:69581`'s `num57` = ExtraFramesCount[160]
    /// = 7): the wrap band stops at 23 - 7 = 16, so row 15 + one step =
    /// 16 wraps to 2 — the attack rows 16..22 are never walked.
    #[test]
    fn the_truffle_walk_wraps_before_its_seven_attack_rows() {
        let mut s = FrameState::default();
        let i = input(1.0, 0.0);
        s.row = 15;
        s.counter = 6.0;
        find_frame_mid1(FindFrameMid1Kind::Truffle, &mut s, &i);
        assert_eq!(s.row, 2, "the wrap at 23 - 7 = 16");
        // The grounded flip rides direction ±1 only.
        let mut s = FrameState { sprite_dir: -1, ..Default::default() };
        let mut i = input(1.0, 0.0);
        i.direction = 0;
        find_frame_mid1(FindFrameMid1Kind::Truffle, &mut s, &i);
        assert_eq!(s.sprite_dir, -1, "direction 0 leaves the flip");
    }

    /// The BlackRecluse (`NPC.cs:73227-73267`): the air pins counter -12
    /// and rows 5/6; the ground's `|vx|*1.1` bands pick rows 6/7/1/2/3
    /// from the -12 start, and the `else` reset holds the row.
    #[test]
    fn the_black_recluse_lands_from_minus_twelve_through_the_bands() {
        // Air rising → row 5, counter -12.
        let mut s = FrameState::default();
        find_frame_mid1(FindFrameMid1Kind::BlackRecluse, &mut s, &input(0.0, -1.0));
        assert_eq!((s.row, s.counter), (5, -12.0));
        // Air falling → row 6.
        find_frame_mid1(FindFrameMid1Kind::BlackRecluse, &mut s, &input(0.0, 1.0));
        assert_eq!((s.row, s.counter), (6, -12.0));
        // Ground at vx 0 adds NOTHING (`|vx| * 1.1` = 0): the -12
        // landing counter parks in the `< -6` band (row 6) forever.
        let ground = input(0.0, 0.0);
        let mut s = FrameState { counter: -12.0, ..Default::default() };
        for _ in 0..8 {
            find_frame_mid1(FindFrameMid1Kind::BlackRecluse, &mut s, &ground);
            assert_eq!((s.row, s.counter), (6, -12.0));
        }
        // The band boundaries (`:73243-73266`): preset counters ride
        // rows 6 (<-6), 7 (-6..0), 1 (0..6), 2 (6..12), 3 (12..18);
        // >= 18 resets the counter and HOLDS the row.
        for (counter, row) in
            [(-12.0, 6u32), (-6.0, 7), (-0.5, 7), (0.0, 1), (6.0, 2), (12.0, 3)]
        {
            let mut s = FrameState { counter, row: 0, ..Default::default() };
            find_frame_mid1(FindFrameMid1Kind::BlackRecluse, &mut s, &ground);
            assert_eq!((s.row, s.counter), (row, counter), "counter {counter}");
        }
        let mut s = FrameState { counter: 18.0, row: 3, ..Default::default() };
        find_frame_mid1(FindFrameMid1Kind::BlackRecluse, &mut s, &ground);
        assert_eq!((s.row, s.counter), (3, 0.0), "the else holds the row");
        // The walk at vx 10 (`10 * 1.1f` = 11.0 exactly in f32): from
        // the -12 landing, tick 1 → -1 (row 7), tick 2 → 10 (row 2),
        // tick 3 → 21 → the reset (row holds 2, counter 0).
        let run = input(10.0, 0.0);
        let mut s = FrameState { counter: -12.0, ..Default::default() };
        find_frame_mid1(FindFrameMid1Kind::BlackRecluse, &mut s, &run);
        assert_eq!((s.row, s.counter), (7, -1.0));
        find_frame_mid1(FindFrameMid1Kind::BlackRecluse, &mut s, &run);
        assert_eq!((s.row, s.counter), (2, 10.0));
        find_frame_mid1(FindFrameMid1Kind::BlackRecluse, &mut s, &run);
        assert_eq!((s.row, s.counter), (2, 0.0));
        // The direction flip on the ground.
        let mut i = input(0.0, 0.0);
        i.direction = -1;
        let mut s = FrameState::default();
        find_frame_mid1(FindFrameMid1Kind::BlackRecluse, &mut s, &i);
        assert_eq!(s.sprite_dir, -1);
    }

    /// The WallCreeper (`NPC.cs:73083-73111`): the air pins rows 4/0
    /// with counter 0; the ground's WHILE loop steps a row per 6 keeping
    /// the remainder, wrapping past row 3 to 0.
    #[test]
    fn the_wall_creeper_while_loop_keeps_the_remainder() {
        // Air rising → row 4; falling → row 0.
        let mut s = FrameState::default();
        find_frame_mid1(FindFrameMid1Kind::WallCreeper, &mut s, &input(0.0, -1.0));
        assert_eq!((s.row, s.counter), (4, 0.0));
        find_frame_mid1(FindFrameMid1Kind::WallCreeper, &mut s, &input(0.0, 1.0));
        assert_eq!((s.row, s.counter), (0, 0.0));
        // Ground at vx 6: 6.6 a tick → one step a tick (remainder 0.6
        // accumulates); from row 0 the rows climb 1,2,3,0,1,…
        let run = input(6.0, 0.0);
        let mut s = FrameState::default();
        let mut rows = Vec::new();
        for _ in 0..5 {
            find_frame_mid1(FindFrameMid1Kind::WallCreeper, &mut s, &run);
            rows.push(s.row);
        }
        assert_eq!(rows, [1, 2, 3, 0, 1]);
        // A fast tick steps several rows at once: counter 0 + |vx|*1.1
        // = 26.4 → four steps (26.4-6*4 = 2.4), row 0 → 1 → 2 → 3 → 0.
        let mut s = FrameState::default();
        find_frame_mid1(FindFrameMid1Kind::WallCreeper, &mut s, &input(24.0, 0.0));
        assert_eq!(s.row, 0);
        assert!((s.counter - 2.4).abs() < 1e-4, "remainder {}", s.counter);
    }

    /// The WallCreeperWall (`NPC.cs:73112-73145`): the
    /// `(|vx|+|vy|)*0.5` accumulator's 6/12/18/24 bands over rows
    /// 0/1/2/3, the `else` reset holding the row, no sprite write.
    #[test]
    fn the_wall_creeper_wall_climbs_four_bands_and_holds_the_reset() {
        let mut s = FrameState { sprite_dir: -1, rotation: 0.5, ..Default::default() };
        // (|vx|+|vy|) = 4 → +2 a tick: counter 2,4,6,…,24.
        let i = input(2.0, 2.0);
        let mut rows = Vec::new();
        for _ in 0..13 {
            find_frame_mid1(FindFrameMid1Kind::WallCreeperWall, &mut s, &i);
            rows.push(s.row);
        }
        // 2,4 → 0; 6,8,10 → 1; 12,14,16 → 2; 18,20,22 → 3; 24 → reset
        // (row holds 3); then 2 → 0.
        assert_eq!(rows, [0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3, 3, 0]);
        assert_eq!(s.sprite_dir, -1, "no flip write");
        assert_eq!(s.rotation, 0.5, "no rotation write");
    }
}
