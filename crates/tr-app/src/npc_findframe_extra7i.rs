//! W4-f26 slice 7i — FindFrame extra wave 7i: the NEXT 4 TRULY
//! unclaimed case ids of `NPC.FindFrame` (`NPC.cs:66684-77370`) after
//! the 7h slice — **646/647/648/649**, the first four gem bunnies
//! (`ID/NPCID.cs:11969/:11971/:11973/:11975`). SMALL batch by design
//! (the 400-storm brief: 4 kinds, not the full seven-bunny run —
//! 650/651/652 stay unclaimed for the next slice).
//!
//! The window re-probed mechanically at THIS slice's census time
//! (`git fetch origin --prune`, per-tip `git rev-parse` + mechanical
//! grep) against the d44 master census (`docs/research/68-findframe-
//! wire-status.md` §6's 130-kind UNCLAIMED worklist — the d44 tip
//! ADVANCED `c863a7a5` → `ce2c3a24a`, refresh 4: the extra7c/extra8
//! WIRE round lands OUR f26 7c claims + the f27 extra8 claims behind
//! the `cs_id` hook — no NEW independent claim; §6's worklist stays
//! the 130-row base):
//!
//! * the f26 7d/7e/7f pushed claims = 15; the f27 tip `1a24a133`
//!   UNCHANGED slice-10 claims (537-545) = 9; the d42 LANDED arms = 38
//!   (tip ADVANCED `2d7ac08c` → `33b3cfe07` — the rotation extraction
//!   + the BloodSquid-619 hover fix, NO new ids); the c4 `0df6964b` /
//!   c8 / e26 `048cca6c` INLINE arms = 8 (c8 ADVANCED `708522fc` →
//!   `7c2de9c06` — the retire-phase docs/rename, the 606 arm
//!   unchanged); the g10 tip `5dee2865` UNCHANGED town-pet arm = 11
//!   (637/638/656/670/678-684); the f26 7g pushed claims = 5; the
//!   f26 7h claims (636/639-645, commit `89cef0b45`) = 8;
//! * the d41 tip ADVANCED `5d036721` → `727578f2d` (batch 3: the
//!   desert antlions + Dune Splicer 508-512 — dedupe inside our own
//!   7c claims, no new ids);
//! * all EIGHT pushed lane tips grep ZERO for `GemBunny` +
//!   `\b64[6-9] =>` over `npc.rs` / `npc/` / `npc_findframe*.rs`
//!   (f27/d42/d41/d44/c4/c8/e26/g10); this branch's own tree greps
//!   ZERO arms — the ONLY kind-name hits are the
//!   `creative_sacrifices.rs`/`itemdata.rs`/`lang_tables.rs` data
//!   rows, not arms; the d44 refresh-4 §6 rows for 646-649 still say
//!   **UNCLAIMED** (`NPC.cs:76859-76862`).
//!
//! Census: the 7h post-state **36 unclaimed** − 4 = **32 unclaimed**
//! after this slice. The 32-id worklist: 634 (the RNG SporeBat — its
//! seam contract is published in the extra7h header), 650-655 (6),
//! 657-662 (6), 664, 667, 669, 671-677 (7), 687-689 (3), 691-696 (6).
//!
//! The arm — the hop group `case 46/47/303/337/443/464/540/614/
//! 646-652` (`NPC.cs:76851-76904`, our labels `:76859-76862`; 540 is
//! the f27-claimed bunny twin — the VERBATIM ONE-group machine copy,
//! the 7g/7h precedent; the wiring merge keeps one):
//!
//! * grounded (`vy == 0`): the ±1 direction flip (`:76868-76875`),
//!   the idle row-0 zero + break (`:76876-76881`), the
//!   `|vx| * 1f` THEN `++` accumulator with the STRICT >6.0 step
//!   (`:76882-76888`), the wrap at `row >= count` back to row **0**
//!   (`:76889-76892` — the FULL 7 rows walk, NO count−1 decrement
//!   (the squirrel's `:76497` decrement does NOT apply here) and NO
//!   lift-to-start-row);
//! * air: row **4** rising (`:76894-76898`) / row **6** falling
//!   (`:76899-76903`), both counter-zeroed with the sprite UNTOUCHED
//!   (the grounded-only flip). Unlike the squirrel arm the two air
//!   checks are `else if (Y < 0)` / `else if (Y > 0)` — the vy != 0
//!   gate makes the second condition redundant, same behaviour.
//!
//! `SetDefaults` gives all seven bunnies aiStyle 7 (`NPC.cs:17238-
//! 17248`, the `type >= 646 && type <= 652` block, aiStyle at
//! `:17242`) — the hop arm reads NO aiStyle at all (no dead branch to
//! document, unlike the squirrel's backpedal).
//!
//! Shape: the kinds are NOT in `NpcKind` yet (the roster wave owns
//! them) — the `npc_findframe_extra7h.rs` precedent: pure machines
//! over [`FrameState`]/[`FrameInput`], the wiring seam
//! [`find_frame_kind_extra7i`] hooked later at `npc.rs::find_frame`
//! by `cs_id`.
//!
//! RNG: NONE — the mechanical grep over the cited range
//! (`:76851-76904`, 54 lines) = ZERO `Main.rand`/`.Next(`/`NextFloat`/
//! `NextDouble`. World reads: NONE — the body reads only
//! velocity/direction (the extra-7b FrameInput seam needs no
//! resolved world booleans this wave).
//!
//! Row convention: frame ROW indices, not pixels (the C#'s `num`
//! cancels). Frame counts `Main.npcFrameCount` (`Main.cs:67082`'s
//! 697-entry initializer): 646-649 = **7** each (`:67148` entries
//! 8-11) — counted entry-by-entry in
//! [`FindFrameExtra7iKind::frame_count`] and cross-checked against
//! `render/entities.rs::NPC_FRAME_COUNT` (all 697 slots re-diffed =
//! ZERO mismatches, a mechanical text-parse diff).

// The wiring is a later round (the kinds are not in `NpcKind` yet) —
// the `npc_findframe_extra7h.rs` precedent.
#![allow(dead_code)]

/// The extra-wave-7i kinds, keyed by their VANILLA `NPC.type`
/// (`ID/NPCID.cs`'s `public const short` rows).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FindFrameExtra7iKind {
    /// `NPCID.GemBunnyAmethyst` = 646 (`NPCID.cs:11969`) — the hop
    /// group (`NPC.cs:76859`).
    GemBunnyAmethyst,
    /// `NPCID.GemBunnyTopaz` = 647 (`NPCID.cs:11971`) — the same
    /// group (`NPC.cs:76860`).
    GemBunnyTopaz,
    /// `NPCID.GemBunnySapphire` = 648 (`NPCID.cs:11973`) — the same
    /// group (`NPC.cs:76861`).
    GemBunnySapphire,
    /// `NPCID.GemBunnyEmerald` = 649 (`NPCID.cs:11975`) — the same
    /// group (`NPC.cs:76862`).
    GemBunnyEmerald,
}

impl FindFrameExtra7iKind {
    /// The vanilla `NPC.type` id.
    pub fn vanilla_type(self) -> i16 {
        match self {
            Self::GemBunnyAmethyst => 646,
            Self::GemBunnyTopaz => 647,
            Self::GemBunnySapphire => 648,
            Self::GemBunnyEmerald => 649,
        }
    }

    /// `Main.npcFrameCount[type]` (`Main.cs:67082`'s initializer,
    /// counted entry-by-entry): 646-649 = 7 (`:67148` entries 8-11).
    pub fn frame_count(self) -> u32 {
        7
    }
}

/// The vanilla-id wiring seam: `Some(kind)` for exactly this wave's
/// ids (the `find_frame_kind_extra7h` precedent). The wiring round
/// hooks it at the `npc.rs::find_frame` head by `cs_id`.
pub fn find_frame_kind_extra7i(vanilla_type: i16) -> Option<FindFrameExtra7iKind> {
    Some(match vanilla_type {
        646 => FindFrameExtra7iKind::GemBunnyAmethyst,
        647 => FindFrameExtra7iKind::GemBunnyTopaz,
        648 => FindFrameExtra7iKind::GemBunnySapphire,
        649 => FindFrameExtra7iKind::GemBunnyEmerald,
        _ => return None,
    })
}

/// The `NPC` fields `FindFrame` reads/writes for this arm:
/// `frame.Y / num` (the row), `frameCounter` and `spriteDirection`.
/// The wiring step copies them to/from `Npc`. `rotation`/`local_ai`
/// are carried for the uniform 7h wiring copy, never read/written
/// here.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FrameState {
    /// `frame.Y / num` — the sheet row.
    pub row: u32,
    /// `NPC.frameCounter` (the C# field is a `double`, `NPC.cs:6485`;
    /// the port's f32 is the existing `npc.rs` convention — the
    /// integer accumulators here are exact in both).
    pub counter: f32,
    /// `NPC.spriteDirection` (±1).
    pub sprite_dir: i32,
    /// `NPC.rotation` — the arm never writes it.
    pub rotation: f32,
    /// `NPC.localAI[0]`/`[1]` — never read/written this wave; carried
    /// for the uniform wiring copy.
    pub local_ai: [f32; 2],
}

/// The per-tick inputs the arm reads: `velocity` and `direction`.
/// No RNG draws, no world reads, no ai slots this wave.
#[derive(Clone, Copy, Debug)]
pub struct FrameInput {
    pub vx: f32,
    pub vy: f32,
    /// `Entity.direction` (±1; 0 leaves `spriteDirection` alone —
    /// the copy is ±1-gated).
    pub direction: i32,
}

impl Default for FrameInput {
    fn default() -> Self {
        Self { vx: 0.0, vy: 0.0, direction: 1 }
    }
}

/// `NPC.FindFrame` (`NPC.cs:66684`) for the extra-wave-7i kinds — one
/// tick, the hop-group body (`NPC.cs:76866-76903`).
pub fn find_frame_extra7i(kind: FindFrameExtra7iKind, s: &mut FrameState, i: &FrameInput) {
    if i.vy == 0.0 {
        // `:76868-76875`.
        if i.direction == 1 {
            s.sprite_dir = 1;
        }
        if i.direction == -1 {
            s.sprite_dir = -1;
        }
        // `:76876-76881`.
        if i.vx == 0.0 {
            s.row = 0;
            s.counter = 0.0;
            return;
        }
        // `:76882-76888` — |vx| * 1f THEN ++, STRICT > 6.0.
        s.counter += i.vx.abs() * 1.0;
        s.counter += 1.0;
        if s.counter > 6.0 {
            s.row += 1;
            s.counter = 0.0;
        }
        // `:76889-76892` — the FULL-count wrap to 0 (no decrement).
        if s.row >= kind.frame_count() {
            s.row = 0;
        }
    } else if i.vy < 0.0 {
        // `:76894-76898`.
        s.counter = 0.0;
        s.row = 4;
    } else {
        // `:76899-76903` (the C#'s `else if (Y > 0)` — the vy != 0
        // gate makes the condition redundant).
        s.counter = 0.0;
        s.row = 6;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BUNNIES: [FindFrameExtra7iKind; 4] = [
        FindFrameExtra7iKind::GemBunnyAmethyst,
        FindFrameExtra7iKind::GemBunnyTopaz,
        FindFrameExtra7iKind::GemBunnySapphire,
        FindFrameExtra7iKind::GemBunnyEmerald,
    ];

    fn input(vx: f32, vy: f32) -> FrameInput {
        FrameInput { vx, vy, ..Default::default() }
    }

    fn facing(vx: f32, vy: f32, direction: i32) -> FrameInput {
        FrameInput { vx, vy, direction }
    }

    fn rows_over(kind: FindFrameExtra7iKind, i: FrameInput, ticks: usize) -> Vec<u32> {
        let mut s = FrameState::default();
        let mut out = Vec::new();
        for _ in 0..ticks {
            find_frame_extra7i(kind, &mut s, &i);
            out.push(s.row);
        }
        out
    }

    /// The frame-count table agrees with the port's verbatim
    /// `NPC_FRAME_COUNT` (`Main.cs:67082` initializer cross-check).
    #[test]
    fn f26s7i_frame_counts_match_the_port_table() {
        let table = crate::render::entities::NPC_FRAME_COUNT;
        for k in BUNNIES {
            assert_eq!(
                k.frame_count(),
                table[k.vanilla_type() as usize],
                "{k:?} ({})",
                k.vanilla_type()
            );
        }
    }

    /// The bunny walk: vx 2 adds EXACTLY 3.0 a tick (|2| * 1f then
    /// ++), the STRICT >6 step lands every 3rd tick (6.0 is NOT >6);
    /// NO lift, the row-0 start walks 0 → 1; the >= 7 wrap goes back
    /// to row 0 — the FULL 7 rows cycle (row 6 IS walkable, unlike
    /// the squirrel's count−1 strip).
    #[test]
    fn f26s7i_the_bunny_walk_full_wrap_to_zero() {
        for k in BUNNIES {
            assert_eq!(
                rows_over(k, input(2.0, 0.0), 22),
                vec![0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4, 4, 5, 5, 5, 6, 6, 6, 0, 0]
            );
            // The wrap edge: row 6 + counter 6 → +3 = 9 > 6 → row 7
            // → >= 7 → 0, counter 0.
            let mut s = FrameState { row: 6, counter: 6.0, ..Default::default() };
            find_frame_extra7i(k, &mut s, &input(2.0, 0.0));
            assert_eq!((s.row, s.counter), (0, 0.0));
        }
    }

    /// The slower feed: vx 1 adds 2.0 a tick — the >6 step lands
    /// every 4th tick; counter 4 + 2 = 6.0 does NOT step (the strict
    /// edge exact in f32 and the C# double alike).
    #[test]
    fn f26s7i_the_bunny_strict_six_cadence() {
        let k = FindFrameExtra7iKind::GemBunnyAmethyst;
        assert_eq!(
            rows_over(k, input(1.0, 0.0), 12),
            vec![0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3]
        );
        let mut s = FrameState { row: 2, counter: 4.0, ..Default::default() };
        find_frame_extra7i(k, &mut s, &input(1.0, 0.0));
        assert_eq!((s.row, s.counter), (2, 6.0));
    }

    /// The idle/air pins: the idle zeroes BOTH row and counter; the
    /// air pins row 4 rising / row 6 falling with the counter zeroed
    /// and the sprite UNTOUCHED (the grounded-only ±1 flip);
    /// direction 0 leaves the sprite alone.
    #[test]
    fn f26s7i_the_bunny_idle_and_air_pins() {
        for k in BUNNIES {
            let mut s = FrameState { row: 3, counter: 5.0, sprite_dir: -1, ..Default::default() };
            find_frame_extra7i(k, &mut s, &facing(0.0, 0.0, 0));
            assert_eq!((s.row, s.counter, s.sprite_dir), (0, 0.0, -1));
            let mut s = FrameState { row: 1, counter: 5.0, sprite_dir: 1, ..Default::default() };
            find_frame_extra7i(k, &mut s, &facing(2.0, -2.0, -1));
            assert_eq!((s.row, s.counter, s.sprite_dir), (4, 0.0, 1));
            let mut s = FrameState { row: 1, counter: 5.0, sprite_dir: -1, ..Default::default() };
            find_frame_extra7i(k, &mut s, &facing(-2.0, 2.0, 1));
            assert_eq!((s.row, s.counter, s.sprite_dir), (6, 0.0, -1));
            // The grounded flip is ±1-gated.
            let mut s = FrameState { sprite_dir: 1, ..Default::default() };
            find_frame_extra7i(k, &mut s, &facing(-2.0, 0.0, -1));
            assert_eq!(s.sprite_dir, -1);
        }
    }

    /// The seam maps exactly this wave's 4 ids — the hop group's
    /// other members 46/47/303/337/443/464/540/614 (540 f27-claimed),
    /// the DEFERRED bunny tail 650-652, the 7h-claimed 645, the
    /// RNG-excluded 634 and the g10-claimed 656 stay `None`.
    #[test]
    fn f26s7i_the_seam_maps_exactly_the_slice() {
        for k in BUNNIES {
            let id = k.vanilla_type();
            assert_eq!(find_frame_kind_extra7i(id), Some(k));
        }
        for id in [46, 47, 303, 337, 443, 464, 540, 614, 634, 645, 650, 651, 652, 656] {
            assert_eq!(find_frame_kind_extra7i(id), None, "type {id}");
        }
    }

    /// The 4-kind strip sweep: every kind ticks 27 input combos
    /// (vx 0/1/3 × vy 0/-2/2 × direction 1/-1/0) for 40 ticks each
    /// from a fresh state — every row stays inside its own sheet
    /// (the 7h sweep shape; the full-count wrap exercises inside the
    /// window).
    #[test]
    fn f26s7i_the_4_kind_strip_sweep_stays_in_sheet() {
        for k in BUNNIES {
            let mut combo = 0;
            for vx in [0.0f32, 1.0, 3.0] {
                for vy in [0.0f32, -2.0, 2.0] {
                    for direction in [1i32, -1, 0] {
                        let mut s = FrameState::default();
                        let i = FrameInput { vx, vy, direction };
                        for _ in 0..40 {
                            find_frame_extra7i(k, &mut s, &i);
                            assert!(
                                s.row < k.frame_count(),
                                "{k:?} combo {combo} row {} escapes the {}-row sheet",
                                s.row,
                                k.frame_count()
                            );
                        }
                        combo += 1;
                    }
                }
            }
        }
    }
}
