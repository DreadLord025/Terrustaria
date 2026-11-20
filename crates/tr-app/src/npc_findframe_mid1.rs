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
/// ids do not exist for them yet). Batch 1 carries the first six.
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

    /// The batch-1 `Main.npcFrameCount` rows, counted entry-by-entry out
    /// of the `Main.cs:67082` initializer and cross-checked against the
    /// port's `NPC_FRAME_COUNT` table.
    #[test]
    fn the_mid1_batch1_frame_counts_match_the_main_initializer() {
        let table = [
            (FindFrameMid1Kind::IceBat, 150, 4),
            (FindFrameMid1Kind::Lavabat, 151, 4),
            (FindFrameMid1Kind::GiantFlyingFox, 152, 4),
            (FindFrameMid1Kind::RedDevil, 156, 5),
            (FindFrameMid1Kind::Arapaima, 157, 6),
            (FindFrameMid1Kind::VampireBat, 158, 4),
        ];
        for (kind, vanilla, count) in table {
            assert_eq!(kind.vanilla_type(), vanilla, "{kind:?} id");
            assert_eq!(kind.frame_count(), count, "{kind:?} npcFrameCount");
            assert_eq!(
                crate::render::entities::npc_rows(vanilla),
                count,
                "{kind:?} vs the port's NPC_FRAME_COUNT"
            );
            assert_eq!(find_frame_kind_for_cs_id(vanilla), Some(kind), "{kind:?} cs_id map");
        }
    }

    /// The `cs_id` hook maps exactly the batch-1 ids and nothing else —
    /// the inertness seam (no current kind's `cs_id` is in the set).
    #[test]
    fn the_cs_id_hook_maps_only_the_batch1_ids() {
        for id in 0u16..=697 {
            let mapped = find_frame_kind_for_cs_id(id);
            let in_set = matches!(id, 150 | 151 | 152 | 156 | 157 | 158);
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
}
