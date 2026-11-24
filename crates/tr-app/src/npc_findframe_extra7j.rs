//! W4-f26 slice 7j — FindFrame extra wave 7j: the NEXT 6 TRULY
//! unclaimed case ids of `NPC.FindFrame` (`NPC.cs:66684-77370`) after
//! the 7i slice — **650/651/652** (the gem-bunny tail deferred by 7i:
//! Ruby/Diamond/Amber, `ID/NPCID.cs:11977/:11979/:11981`) plus
//! **653/654/655** (HellButterfly/Lavafly/MagmaSnail,
//! `NPCID.cs:11983/:11985/:11987`). SMALL batch by design (the
//! 400-storm brief: the 650-655 window, 6 kinds).
//!
//! The brief's "same hop-group-arm extension" premise holds ONLY for
//! 650-652 (`NPC.cs:76863-76865`); the hell-critter tail is three
//! DIFFERENT arms: 653 the butterfly group (`:72741`), 654 its own
//! firefly-blink case (`:72690`), 655 the snail body (`:72240`).
//! **655's machine is ALREADY PORTED on this branch** —
//! `npc_ai_style67.rs::snail_find_frame` (`:414`, its case-359/360/655
//! doc at `:404`) is the verbatim twin; this module's MagmaSnail arm
//! DELEGATES to it (the d44 wave-C Snail/GlowingSnail precedent:
//! "one machine, no second copy", the house shared-helper rule). The
//! claim here is the id-keyed SEAM entry, not a second machine.
//!
//! Census re-probe at THIS slice's time (`git fetch origin --prune`,
//! per-tip `git rev-parse` + mechanical grep over `npc.rs` / `npc/` /
//! `npc_findframe*.rs`):
//!
//! * f27 ADVANCED `1a24a133` → `77ebe4e93` (slice 11: 435/436 — a
//!   keep-one dup vs OUR pushed 7g claims, no new independent id —
//!   plus the 550/552-563 v1-arm FIXES, outside our window);
//! * d42 ADVANCED `33b3cfe07` → `bac408be0` (doc-only finals: the
//!   keep-one drop-hunks + the 578-630 band declaration, no new ids);
//! * d41 `727578f2d` / d44 `ce2c3a24a` / c4 `0df6964b` / c8
//!   `7c2de9c06` / e26 `048cca6c` / g10 `5dee2865` UNCHANGED;
//! * all EIGHT tips grep ZERO for `GemBunnyRuby|GemBunnyDiamond|
//!   GemBunnyAmber|HellButterfly|Lavafly|MagmaSnail` + `65[0-5] =>`
//!   arms — the ONLY d44 hits are wave-C's header cut-list comments
//!   naming 654/653/655 as deliberately OUT of its 350-550 band;
//!   the advanced f27/d42 tips grep ZERO `=>` arms for the remaining
//!   worklist ids too.
//!
//! Census: the 7i post-state **32 unclaimed** − 6 = **26 unclaimed**
//! after this slice. The 26-id worklist: 634 (the RNG SporeBat — its
//! seam contract is published in the extra7h header), 657-662 (6),
//! 664, 667, 669, 671-677 (7), 687-689 (3), 691-696 (6).
//!
//! The four arms, ported verbatim:
//!
//! * 650-652 — the hop group `case 46/47/303/337/443/464/540/614/
//!   646-652` (`NPC.cs:76851-76904`, our labels `:76863-76865`): the
//!   VERBATIM 7i machine copy (the f27-540 ONE-group precedent; the
//!   wiring merge keeps one) — the grounded ±1 flip
//!   (`:76868-76875`), the idle row-0 zero + break (`:76876-76881`),
//!   the `|vx| * 1f` THEN `++` STRICT >6.0 step (`:76882-76888`),
//!   the `>= count` wrap to row **0** (`:76889-76892` — the FULL 7
//!   rows walk, NO count−1 decrement, NO lift), the air row 4 rising
//!   (`:76894-76898`) / row 6 falling (`:76899-76903`) counter-zeroed
//!   sprite-UNTOUCHED (the `else if (Y > 0)` redundant behind the
//!   vy != 0 gate). All three bunnies aiStyle 7 (`:17238-17248`,
//!   aiStyle at `:17242`) — the arm reads no aiStyle, no dead branch.
//! * 653 HellButterfly — the butterfly group `case 356/444/653/661`
//!   (`NPC.cs:72739-72773`, our label `:72741`): `rotation =
//!   vx * 0.3f` (`:72745`), the UNCONDITIONAL sprite copy (`:72746`
//!   — no ±1 gate, unlike the hop arm), the velocity-fed accumulator
//!   `counter + 1.0 + (|vx| + |vy|) / 2f` (`:72747`), the 7-tick
//!   bands rows 0/1/2/1 (`:72748-72762`) with the reset at
//!   `7*4-1 = 27` (`:72763-72766`), and NO colour-bank offset —
//!   the `:72768` gate EXCLUDES 653 (356-only, the d44 wave-C
//!   Butterfly arm keeps it; aiStyle 65 `:17254`).
//! * 661 EmpressButterfly (`NPCID.cs:11999`) — the SAME butterfly
//!   group, its own case label `:72742`: ONE machine, the claim is
//!   the id-keyed seam row (the 655 delegation precedent). The
//!   `:72768` gate EXCLUDES 661 EXACTLY like 653
//!   (`this.type != 444 && this.type != 653 && this.type != 661`),
//!   so no colour bank either.
//! * 654 Lavafly — its own `case 654` (`NPC.cs:72690-72709`): the
//!   UNCONDITIONAL sprite copy (`:72691`), the bare `++` (`:72692`),
//!   the <4 row-0 / else row-1 blink with the `>= 7` reset
//!   (`:72693-72704`), then the `localAI[2] <= 0f` glow-off shift
//!   `+= 2` rows (`:72705-72708` — the 4-row sheet is two 2-row
//!   banks: lit on top, unlit below; the d44 wave-C Firefly arm is
//!   the 355/358 twin). The `localAI[2]` glow timer is AI-side —
//!   the style-64 machine ALREADY on this branch
//!   (`npc_ai_style64.rs`, the roll `NPC.cs:34184`
//!   `Main.rand.Next(10, 30)` behind the night/depth gate, the
//!   per-tick decrement `:34173`) — so it rides [`FrameInput`] as
//!   the RESOLVED `local_ai2` value (the 7d resolved-draw seam
//!   precedent): the frame machine itself draws NOTHING.
//! * 655 MagmaSnail — the snail body (`NPC.cs:72240-72295`, its own
//!   `case 655` at `:72240`, shared with 359/360): DELEGATED to
//!   `npc_ai_style67::snail_find_frame` (the d44 wave-C precedent) —
//!   the `vy > 1f` fall (`:72241-72255`: the 6-tick row-4 phase,
//!   row 5 after, reset at `6*2-1 = 11`), the zero-velocity freeze
//!   (`:72258-72261`'s `!(velocity != Vector2.Zero)` break — frame
//!   AND counter untouched), the 10-tick six-phase walk rows
//!   0/1/2/3/2/1 (`:72262-72293`, reset at `10*6-1 = 59`). aiStyle
//!   67 (`:17284`); the arm reads no ai/local slots.
//!
//! Shape: the kinds are NOT in `NpcKind` yet (the roster wave owns
//! them) — the `npc_findframe_extra7i.rs` precedent: pure machines
//! over [`FrameState`]/[`FrameInput`], the wiring seam
//! [`find_frame_kind_extra7j`] hooked later at `npc.rs::find_frame`
//! by `cs_id`.
//!
//! RNG: NONE — the mechanical grep over the four cited ranges
//! (`:76851-76904`, `:72739-72773`, `:72690-72709`, `:72240-72295`)
//! = ZERO `Main.rand`/`.Next(`/`NextFloat`/`NextDouble` (the same
//! grep also covered `Main.player`/`Main.npc[`/`Collision.`/
//! `WorldGen.`/`Framing.`/`Dust.` = ZERO world reads).
//!
//! Row convention: frame ROW indices, not pixels (the C#'s `num`
//! cancels). Frame counts `Main.npcFrameCount` (`Main.cs:67082`'s
//! 697-entry initializer): 650-652 = **7** (`:67149` entries 1-3),
//! 653 = **3** (entry 4), 654 = **4** (entry 5), 655 = **6** (entry
//! 6) — counted entry-by-entry in
//! [`FindFrameExtra7jKind::frame_count`] and cross-checked against
//! `render/entities.rs::NPC_FRAME_COUNT` (all 697 slots re-diffed =
//! ZERO mismatches, a mechanical text-parse diff). 661's add-on:
//! **3** (`Main.cs:67150` entry 2, the same 3-row sheet as 653).

// The wiring is a later round (the kinds are not in `NpcKind` yet) —
// the `npc_findframe_extra7i.rs` precedent.
#![allow(dead_code)]

/// The extra-wave-7j kinds, keyed by their VANILLA `NPC.type`
/// (`ID/NPCID.cs`'s `public const short` rows).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FindFrameExtra7jKind {
    /// `NPCID.GemBunnyRuby` = 650 (`NPCID.cs:11977`) — the hop group
    /// (`NPC.cs:76863`).
    GemBunnyRuby,
    /// `NPCID.GemBunnyDiamond` = 651 (`NPCID.cs:11979`) — the same
    /// group (`NPC.cs:76864`).
    GemBunnyDiamond,
    /// `NPCID.GemBunnyAmber` = 652 (`NPCID.cs:11981`) — the same
    /// group (`NPC.cs:76865`).
    GemBunnyAmber,
    /// `NPCID.HellButterfly` = 653 (`NPCID.cs:11983`) — the butterfly
    /// group (`NPC.cs:72741`), no colour bank.
    HellButterfly,
    /// `NPCID.Lavafly` = 654 (`NPCID.cs:11985`) — its own firefly
    /// blink case (`NPC.cs:72690`).
    Lavafly,
    /// `NPCID.MagmaSnail` = 655 (`NPCID.cs:11987`) — the snail body
    /// (`NPC.cs:72240`), DELEGATED to
    /// `npc_ai_style67::snail_find_frame` (no second copy).
    MagmaSnail,
    /// `NPCID.EmpressButterfly` = 661 (`NPCID.cs:11999`) — the SAME
    /// butterfly group as 653 (`NPC.cs:72742`), no colour bank (the
    /// `:72768` gate excludes 661 exactly like 653).
    EmpressButterfly,
}

impl FindFrameExtra7jKind {
    /// The vanilla `NPC.type` id.
    pub fn vanilla_type(self) -> i16 {
        match self {
            Self::GemBunnyRuby => 650,
            Self::GemBunnyDiamond => 651,
            Self::GemBunnyAmber => 652,
            Self::HellButterfly => 653,
            Self::Lavafly => 654,
            Self::MagmaSnail => 655,
            Self::EmpressButterfly => 661,
        }
    }

    /// `Main.npcFrameCount[type]` (`Main.cs:67082`'s initializer,
    /// counted entry-by-entry): 650-652 = 7 (`:67149` entries 1-3),
    /// 653 = 3 (entry 4), 654 = 4 (entry 5), 655 = 6 (entry 6);
    /// 661 = 3 (`:67150` entry 2).
    pub fn frame_count(self) -> u32 {
        match self {
            Self::GemBunnyRuby | Self::GemBunnyDiamond | Self::GemBunnyAmber => 7,
            Self::HellButterfly | Self::EmpressButterfly => 3,
            Self::Lavafly => 4,
            Self::MagmaSnail => 6,
        }
    }
}

/// The vanilla-id wiring seam: `Some(kind)` for exactly this wave's
/// ids (the `find_frame_kind_extra7i` precedent). The wiring round
/// hooks it at the `npc.rs::find_frame` head by `cs_id`. The 655
/// entry routes to the ALREADY-PORTED
/// `npc_ai_style67::snail_find_frame` — the seam claim, not a second
/// machine.
pub fn find_frame_kind_extra7j(vanilla_type: i16) -> Option<FindFrameExtra7jKind> {
    Some(match vanilla_type {
        650 => FindFrameExtra7jKind::GemBunnyRuby,
        651 => FindFrameExtra7jKind::GemBunnyDiamond,
        652 => FindFrameExtra7jKind::GemBunnyAmber,
        653 => FindFrameExtra7jKind::HellButterfly,
        654 => FindFrameExtra7jKind::Lavafly,
        655 => FindFrameExtra7jKind::MagmaSnail,
        661 => FindFrameExtra7jKind::EmpressButterfly,
        _ => return None,
    })
}

/// The `NPC` fields `FindFrame` reads/writes for these arms:
/// `frame.Y / num` (the row), `frameCounter`, `spriteDirection` and
/// (653/654 only) `rotation`. The wiring step copies them to/from
/// `Npc`. `local_ai` is carried for the uniform wiring copy, never
/// read/written here (654's glow timer rides [`FrameInput`]).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FrameState {
    /// `frame.Y / num` — the sheet row.
    pub row: u32,
    /// `NPC.frameCounter` (the C# field is a `double`, `NPC.cs:6485`;
    /// the port's f32 is the existing `npc.rs` convention — the
    /// accumulator feeds are documented per witness).
    pub counter: f32,
    /// `NPC.spriteDirection` (±1).
    pub sprite_dir: i32,
    /// `NPC.rotation` — 653 writes `vx * 0.3f` (`:72745`); the hop
    /// and blink arms never touch it.
    pub rotation: f32,
    /// `NPC.localAI[0]`/`[1]` — never read/written this wave; carried
    /// for the uniform wiring copy.
    pub local_ai: [f32; 2],
}

/// The per-tick inputs the arms read. No RNG draws, no world reads.
#[derive(Clone, Copy, Debug)]
pub struct FrameInput {
    pub vx: f32,
    pub vy: f32,
    /// `Entity.direction` (±1; for the hop arm 0 leaves
    /// `spriteDirection` alone — the copy is ±1-gated; 653/654 copy
    /// it UNCONDITIONALLY).
    pub direction: i32,
    /// The RESOLVED `NPC.localAI[2]` — 654's glow timer, owned by the
    /// style-64 AI machine (`npc_ai_style64.rs`; the AI-side roll
    /// `NPC.cs:34184` draws on `Main.rand` THERE, never in
    /// `FindFrame`). Read by 654 ONLY; the other kinds ignore it.
    pub local_ai2: f32,
}

impl Default for FrameInput {
    fn default() -> Self {
        Self { vx: 0.0, vy: 0.0, direction: 1, local_ai2: 0.0 }
    }
}

/// `NPC.FindFrame` (`NPC.cs:66684`) for the extra-wave-7j kinds — one
/// tick.
pub fn find_frame_extra7j(kind: FindFrameExtra7jKind, s: &mut FrameState, i: &FrameInput) {
    match kind {
        // The hop group (`NPC.cs:76866-76903`) — the VERBATIM 7i
        // machine copy (the f27-540 ONE-group precedent).
        FindFrameExtra7jKind::GemBunnyRuby
        | FindFrameExtra7jKind::GemBunnyDiamond
        | FindFrameExtra7jKind::GemBunnyAmber => {
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
                // `:76889-76892` — the FULL-count wrap to 0.
                if s.row >= kind.frame_count() {
                    s.row = 0;
                }
            } else if i.vy < 0.0 {
                // `:76894-76898`.
                s.counter = 0.0;
                s.row = 4;
            } else {
                // `:76899-76903` (the C#'s `else if (Y > 0)` — the
                // vy != 0 gate makes the condition redundant).
                s.counter = 0.0;
                s.row = 6;
            }
        }
        // The butterfly group (`NPC.cs:72743-72772`), 653/661's
        // slice — NO colour bank (the `:72768` gate excludes
        // 444/653/661). ONE machine, two claim rows (the 655
        // delegation precedent).
        FindFrameExtra7jKind::HellButterfly | FindFrameExtra7jKind::EmpressButterfly => {
            let num143 = 7.0f32;
            // `:72745-72746` — the tilt + the UNCONDITIONAL copy.
            s.rotation = i.vx * 0.3;
            s.sprite_dir = i.direction;
            // `:72747` — the velocity-fed accumulator.
            s.counter += 1.0 + (i.vx.abs() + i.vy.abs()) / 2.0;
            // `:72748-72767` — rows 0/1/2/1, reset at 7*4-1 = 27.
            if s.counter < num143 {
                s.row = 0;
            } else if s.counter < num143 * 2.0 {
                s.row = 1;
            } else if s.counter < num143 * 3.0 {
                s.row = 2;
            } else {
                s.row = 1;
                if s.counter >= num143 * 4.0 - 1.0 {
                    s.counter = 0.0;
                }
            }
        }
        // The firefly blink (`NPC.cs:72690-72708`) — the d44 wave-C
        // Firefly twin, 654's own case.
        FindFrameExtra7jKind::Lavafly => {
            // `:72691-72692`.
            s.sprite_dir = i.direction;
            s.counter += 1.0;
            // `:72693-72704` — the <4 / >=7 two-row blink.
            if s.counter < 4.0 {
                s.row = 0;
            } else {
                s.row = 1;
                if s.counter >= 7.0 {
                    s.counter = 0.0;
                }
            }
            // `:72705-72708` — the glow-off bank shift (the RESOLVED
            // AI-side timer rides the input).
            if i.local_ai2 <= 0.0 {
                s.row += 2;
            }
        }
        // The snail body (`NPC.cs:72240-72295`) — ALREADY PORTED by
        // the aiStyle-67 module: delegate, no second copy (the d44
        // wave-C Snail/GlowingSnail precedent, the house
        // shared-helper rule).
        FindFrameExtra7jKind::MagmaSnail => {
            let (row, counter) =
                crate::npc_ai_style67::snail_find_frame(i.vx, i.vy, s.row, s.counter);
            s.row = row;
            s.counter = counter;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BUNNIES: [FindFrameExtra7jKind; 3] = [
        FindFrameExtra7jKind::GemBunnyRuby,
        FindFrameExtra7jKind::GemBunnyDiamond,
        FindFrameExtra7jKind::GemBunnyAmber,
    ];

    const ALL: [FindFrameExtra7jKind; 7] = [
        FindFrameExtra7jKind::GemBunnyRuby,
        FindFrameExtra7jKind::GemBunnyDiamond,
        FindFrameExtra7jKind::GemBunnyAmber,
        FindFrameExtra7jKind::HellButterfly,
        FindFrameExtra7jKind::Lavafly,
        FindFrameExtra7jKind::MagmaSnail,
        FindFrameExtra7jKind::EmpressButterfly,
    ];

    fn input(vx: f32, vy: f32) -> FrameInput {
        FrameInput { vx, vy, ..Default::default() }
    }

    fn facing(vx: f32, vy: f32, direction: i32) -> FrameInput {
        FrameInput { vx, vy, direction, ..Default::default() }
    }

    fn rows_over(kind: FindFrameExtra7jKind, i: FrameInput, ticks: usize) -> Vec<u32> {
        let mut s = FrameState::default();
        let mut out = Vec::new();
        for _ in 0..ticks {
            find_frame_extra7j(kind, &mut s, &i);
            out.push(s.row);
        }
        out
    }

    /// The frame-count table agrees with the port's verbatim
    /// `NPC_FRAME_COUNT` (`Main.cs:67082` initializer cross-check).
    #[test]
    fn f26s7j_frame_counts_match_the_port_table() {
        let table = crate::render::entities::NPC_FRAME_COUNT;
        for k in ALL {
            assert_eq!(
                k.frame_count(),
                table[k.vanilla_type() as usize],
                "{k:?} ({})",
                k.vanilla_type()
            );
        }
    }

    /// The bunny-tail walk: vx 2 adds EXACTLY 3.0 a tick (|2| * 1f
    /// then ++), the STRICT >6 step lands every 3rd tick (6.0 is NOT
    /// >6); NO lift, the row-0 start walks 0 → 1; the >= 7 wrap goes
    /// back to row 0 — the FULL 7 rows cycle (the 7i vectors verbatim).
    #[test]
    fn f26s7j_the_bunny_tail_walk_full_wrap_to_zero() {
        for k in BUNNIES {
            assert_eq!(
                rows_over(k, input(2.0, 0.0), 22),
                vec![0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4, 4, 5, 5, 5, 6, 6, 6, 0, 0]
            );
            // The wrap edge: row 6 + counter 6 → +3 = 9 > 6 → row 7
            // → >= 7 → 0, counter 0.
            let mut s = FrameState { row: 6, counter: 6.0, ..Default::default() };
            find_frame_extra7j(k, &mut s, &input(2.0, 0.0));
            assert_eq!((s.row, s.counter), (0, 0.0));
            // The strict half-step edge: vx 1 → +2.0/tick, counter
            // 4 + 2 = 6.0 does NOT step.
            let mut s = FrameState { row: 2, counter: 4.0, ..Default::default() };
            find_frame_extra7j(k, &mut s, &input(1.0, 0.0));
            assert_eq!((s.row, s.counter), (2, 6.0));
        }
    }

    /// The bunny-tail idle/air pins: the idle zeroes BOTH row and
    /// counter; the air pins row 4 rising / row 6 falling with the
    /// counter zeroed and the sprite UNTOUCHED (the grounded-only ±1
    /// flip); direction 0 leaves the sprite alone.
    #[test]
    fn f26s7j_the_bunny_tail_idle_and_air_pins() {
        for k in BUNNIES {
            let mut s = FrameState { row: 3, counter: 5.0, sprite_dir: -1, ..Default::default() };
            find_frame_extra7j(k, &mut s, &facing(0.0, 0.0, 0));
            assert_eq!((s.row, s.counter, s.sprite_dir), (0, 0.0, -1));
            let mut s = FrameState { row: 1, counter: 5.0, sprite_dir: 1, ..Default::default() };
            find_frame_extra7j(k, &mut s, &facing(2.0, -2.0, -1));
            assert_eq!((s.row, s.counter, s.sprite_dir), (4, 0.0, 1));
            let mut s = FrameState { row: 1, counter: 5.0, sprite_dir: -1, ..Default::default() };
            find_frame_extra7j(k, &mut s, &facing(-2.0, 2.0, 1));
            assert_eq!((s.row, s.counter, s.sprite_dir), (6, 0.0, -1));
            // The grounded flip is ±1-gated.
            let mut s = FrameState { sprite_dir: 1, ..Default::default() };
            find_frame_extra7j(k, &mut s, &facing(-2.0, 0.0, -1));
            assert_eq!(s.sprite_dir, -1);
        }
    }

    /// The HellButterfly still-air flap: the feed is EXACTLY +1.0 a
    /// tick (vx = vy = 0) — rows 0 (ticks 1-6), 1 (7-13), 2 (14-20),
    /// 1 (21-26), the reset at counter >= 27 lands tick 27 SHOWING
    /// row 1 (the reset runs AFTER the row pick), tick 28 back to
    /// row 0. The rotation pins vx*0.3 (0 at rest); the sprite copy
    /// is UNCONDITIONAL (direction 0 copies 0, unlike the hop arm).
    #[test]
    fn f26s7j_the_hell_butterfly_still_flap_and_reset_tick() {
        let k = FindFrameExtra7jKind::HellButterfly;
        assert_eq!(
            rows_over(k, input(0.0, 0.0), 28),
            vec![
                0, 0, 0, 0, 0, 0, // ticks 1-6
                1, 1, 1, 1, 1, 1, 1, // 7-13
                2, 2, 2, 2, 2, 2, 2, // 14-20
                1, 1, 1, 1, 1, 1, // 21-26
                1, // 27 — the reset tick still shows row 1
                0, // 28
            ]
        );
        let mut s = FrameState { sprite_dir: 1, rotation: 9.0, ..Default::default() };
        find_frame_extra7j(k, &mut s, &facing(0.0, 0.0, 0));
        assert_eq!((s.sprite_dir, s.rotation), (0, 0.0));
        // The reset edge from a pinned state: counter 26 + 1 = 27
        // >= 27 → row 1, counter 0.
        let mut s = FrameState { row: 2, counter: 26.0, ..Default::default() };
        find_frame_extra7j(k, &mut s, &input(0.0, 0.0));
        assert_eq!((s.row, s.counter), (1, 0.0));
    }

    /// The HellButterfly velocity feed + tilt: vx 2, vy 2 feeds
    /// EXACTLY 1 + (2+2)/2 = 3.0 a tick — the <7/<14/<21 bands step
    /// every 3rd/2nd/2nd tick; the rotation is vx*0.3 (the f32
    /// product, compared against the SAME expression). NO colour
    /// bank: the row never leaves 0..2 on the 3-row sheet even at
    /// high feed (the `:72768` gate excludes 653 — a 356-style
    /// ai[2] bank would escape).
    #[test]
    fn f26s7j_the_hell_butterfly_velocity_feed_and_tilt() {
        let k = FindFrameExtra7jKind::HellButterfly;
        let mut s = FrameState::default();
        let i = input(2.0, 2.0);
        let mut rows = Vec::new();
        for _ in 0..9 {
            find_frame_extra7j(k, &mut s, &i);
            rows.push(s.row);
        }
        // counters 3,6,9,12,15,18,21,24,27 → rows 0,0,1,1,2,2,1,1,1
        // (tick 9 resets).
        assert_eq!(rows, vec![0, 0, 1, 1, 2, 2, 1, 1, 1]);
        assert_eq!(s.counter, 0.0);
        assert_eq!(s.rotation, 2.0f32 * 0.3);
        // The sprite copy is unconditional even airborne.
        let mut s = FrameState::default();
        find_frame_extra7j(k, &mut s, &facing(-2.0, -2.0, -1));
        assert_eq!(s.sprite_dir, -1);
        assert_eq!(s.rotation, -2.0f32 * 0.3);
    }

    /// The Lavafly blink, glow ON (local_ai2 > 0): the bare ++ feeds
    /// +1.0 — rows 0 (ticks 1-3), 1 (4-6), the >= 7 reset lands tick
    /// 7 SHOWING row 1, tick 8 back to row 0. The sprite copy is
    /// UNCONDITIONAL.
    #[test]
    fn f26s7j_the_lavafly_lit_blink() {
        let k = FindFrameExtra7jKind::Lavafly;
        let lit = FrameInput { local_ai2: 10.0, ..Default::default() };
        let mut s = FrameState::default();
        let mut rows = Vec::new();
        for _ in 0..8 {
            find_frame_extra7j(k, &mut s, &lit);
            rows.push(s.row);
        }
        assert_eq!(rows, vec![0, 0, 0, 1, 1, 1, 1, 0]);
        assert_eq!(s.counter, 1.0);
        let mut s = FrameState { sprite_dir: 1, ..Default::default() };
        find_frame_extra7j(k, &mut s, &FrameInput { direction: 0, local_ai2: 10.0, ..Default::default() });
        assert_eq!(s.sprite_dir, 0);
    }

    /// The Lavafly glow-off bank: local_ai2 <= 0 shifts EVERY tick's
    /// row +2 — the unlit bank rows 2/3 on the 4-row sheet; the
    /// boundary local_ai2 == 0 IS glow-off (the C#'s `<=`); the
    /// timer value itself is NEVER written by the frame machine
    /// (the AI-side style-64 machine owns it).
    #[test]
    fn f26s7j_the_lavafly_glow_off_bank_shift() {
        let k = FindFrameExtra7jKind::Lavafly;
        assert_eq!(
            rows_over(k, input(0.0, 0.0), 8),
            vec![2, 2, 2, 3, 3, 3, 3, 2]
        );
        // The <= boundary: exactly 0 is glow-off, 0.5 is lit.
        let mut s = FrameState::default();
        find_frame_extra7j(k, &mut s, &FrameInput { local_ai2: 0.0, ..Default::default() });
        assert_eq!(s.row, 2);
        let mut s = FrameState::default();
        find_frame_extra7j(k, &mut s, &FrameInput { local_ai2: 0.5, ..Default::default() });
        assert_eq!(s.row, 0);
    }

    /// The MagmaSnail delegation: the arm's (row, counter) equals the
    /// already-ported `snail_find_frame` twin EXACTLY (one machine).
    /// The pinned vectors hand-re-derived from `NPC.cs:72240-72295`:
    /// the fall (vy 2 > 1) feeds +1 — rows 4 at counters 1..5, row 5
    /// at 6..10, the >= 11 reset lands tick 11 SHOWING row 5, tick 12
    /// back to row 4; the walk (vx 1, vy 0) feeds +1 — row 0 at
    /// counters 1..9, row 1 at 10..19, row 2 at 20..29, row 3 at
    /// 30..39, row 2 at 40..49, row 1 at 50..58, the >= 59 reset
    /// lands tick 59 SHOWING row 1; the zero-velocity tick freezes
    /// frame AND counter (`:72258-72261`'s break) and never touches
    /// the sprite.
    #[test]
    fn f26s7j_the_magma_snail_delegates_to_the_snail_twin() {
        let k = FindFrameExtra7jKind::MagmaSnail;
        // The fall cycle: 12 ticks from fresh.
        let mut s = FrameState::default();
        let mut rows = Vec::new();
        for _ in 0..12 {
            find_frame_extra7j(k, &mut s, &input(0.0, 2.0));
            rows.push(s.row);
        }
        assert_eq!(rows, vec![4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 5, 4]);
        assert_eq!(s.counter, 1.0);
        // The walk cycle: the six phases + the reset tick, sampled at
        // the phase edges (ticks 9/10, 19/20, 29/30, 39/40, 49/50,
        // 58/59/60).
        let mut s = FrameState::default();
        let mut edge_rows = Vec::new();
        for tick in 1..=60 {
            find_frame_extra7j(k, &mut s, &input(1.0, 0.0));
            if [9, 10, 19, 20, 29, 30, 39, 40, 49, 50, 58, 59, 60].contains(&tick) {
                edge_rows.push(s.row);
            }
        }
        assert_eq!(edge_rows, vec![0, 1, 1, 2, 2, 3, 3, 2, 2, 1, 1, 1, 0]);
        assert_eq!(s.counter, 1.0);
        // The zero-velocity freeze: frame + counter + sprite untouched.
        let mut s = FrameState { row: 3, counter: 7.0, sprite_dir: -1, ..Default::default() };
        find_frame_extra7j(k, &mut s, &facing(0.0, 0.0, 1));
        assert_eq!((s.row, s.counter, s.sprite_dir), (3, 7.0, -1));
        // The delegation is EXACT: same (row, counter) as the twin
        // over a probe grid.
        for vx in [0.0f32, 1.0, -2.0] {
            for vy in [0.0f32, 0.5, 1.0, 2.0, -1.0] {
                for row in [0u32, 3, 5] {
                    for counter in [0.0f32, 5.0, 10.0, 58.0] {
                        let mut s = FrameState { row, counter, ..Default::default() };
                        find_frame_extra7j(k, &mut s, &input(vx, vy));
                        let (r, c) = crate::npc_ai_style67::snail_find_frame(vx, vy, row, counter);
                        assert_eq!((s.row, s.counter), (r, c), "vx {vx} vy {vy} row {row} c {counter}");
                    }
                }
            }
        }
    }

    /// The seam maps exactly this wave's 7 ids — the hop group's
    /// other members 46/47/303/337/443/464/540/614, the 7i-claimed
    /// 646-649, the snail siblings 359/360 (the twin's own kinds),
    /// the butterfly-group siblings 356/444 and the firefly
    /// siblings 355/358 stay `None`.
    #[test]
    fn f26s7j_the_seam_maps_exactly_the_slice() {
        for k in ALL {
            let id = k.vanilla_type();
            assert_eq!(find_frame_kind_extra7j(id), Some(k));
        }
        for id in [46, 47, 303, 337, 443, 464, 540, 614, 646, 647, 648, 649, 359, 360, 355, 356, 358, 444] {
            assert_eq!(find_frame_kind_extra7j(id), None, "type {id}");
        }
    }

    /// The EmpressButterfly witness: 661 rides the SAME butterfly
    /// machine as 653 (one machine, the claim-row add-on). The still
    /// flap walks the verbatim 653 vector — rows 0/1/2/1 with the
    /// reset tick 27 still showing row 1 — and the velocity-fed
    /// tilt + the reset edge match the 653 twins exactly; no colour
    /// bank (the `:72768` gate excludes 661), so the row never
    /// leaves 0..2 on the 3-row sheet even at high feed.
    #[test]
    fn f26s7j_the_empress_butterfly_rides_the_hell_butterfly_machine() {
        let k = FindFrameExtra7jKind::EmpressButterfly;
        assert_eq!(
            rows_over(k, input(0.0, 0.0), 28),
            vec![
                0, 0, 0, 0, 0, 0, // ticks 1-6
                1, 1, 1, 1, 1, 1, 1, // 7-13
                2, 2, 2, 2, 2, 2, 2, // 14-20
                1, 1, 1, 1, 1, 1, // 21-26
                1, // 27 — the reset tick still shows row 1
                0, // 28
            ]
        );
        // The velocity feed: vx 2, vy 2 feeds +3.0 a tick — the same
        // 653 vector; the tilt is vx*0.3; no colour bank escape.
        let mut s = FrameState::default();
        let i = input(2.0, 2.0);
        let mut rows = Vec::new();
        for _ in 0..9 {
            find_frame_extra7j(k, &mut s, &i);
            rows.push(s.row);
        }
        assert_eq!(rows, vec![0, 0, 1, 1, 2, 2, 1, 1, 1]);
        assert_eq!(s.counter, 0.0);
        assert_eq!(s.rotation, 2.0f32 * 0.3);
        // One machine: state-for-state identical to the 653 twin.
        let mut a = FrameState { row: 2, counter: 26.0, ..Default::default() };
        let mut b = a;
        find_frame_extra7j(k, &mut a, &input(0.0, 0.0));
        find_frame_extra7j(FindFrameExtra7jKind::HellButterfly, &mut b, &input(0.0, 0.0));
        assert_eq!(a, b);
        assert_eq!((a.row, a.counter), (1, 0.0));
    }

    /// The 7-kind strip sweep: every kind ticks 54 input combos
    /// (vx 0/1/3 × vy 0/-2/2 × direction 1/-1/0 × local_ai2 0/10)
    /// for 40 ticks each from a fresh state — every row stays inside
    /// its own sheet (the 7i sweep shape widened by the glow axis;
    /// the butterfly's 27-reset and the blink's 7-reset both cycle
    /// inside the window).
    #[test]
    fn f26s7j_the_7_kind_strip_sweep_stays_in_sheet() {
        for k in ALL {
            let mut combo = 0;
            for vx in [0.0f32, 1.0, 3.0] {
                for vy in [0.0f32, -2.0, 2.0] {
                    for direction in [1i32, -1, 0] {
                        for local_ai2 in [0.0f32, 10.0] {
                            let mut s = FrameState::default();
                            let i = FrameInput { vx, vy, direction, local_ai2 };
                            for _ in 0..40 {
                                find_frame_extra7j(k, &mut s, &i);
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
}
