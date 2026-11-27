//! W4-f26 slice 7h — FindFrame extra wave 7h: the NEXT 8 TRULY
//! unclaimed case ids of `NPC.FindFrame` (`NPC.cs:66684-77370`) after
//! the 7g slice — **636/639-645** — the Hallow Boss (the Empress of
//! Light) + the seven gem squirrels (`ID/NPCID.cs:11949` +
//! `:11955-11967`).
//!
//! The window re-probed mechanically at THIS slice's census time
//! against the d44 master census refresh 3 (`c863a7a5`, the 130-kind
//! UNCLAIMED worklist) minus every pushed claim:
//!
//! * the f26 7d/7e/7f pushed claims (516/521/523-530/531/532/534-536)
//!   = 15 of the 130 (the refresh-3 stale rows);
//! * the f27 tip `1a24a133` slice-10 NEW claims (537-545) = 9 (549 is
//!   a FIX of its own kind, no count change; 533 port-covered);
//! * the d42 tip `2d7ac08c` LANDED find_frame arms = 38
//!   (578/580/582-586/590/591/593-603/608-614/616-620/624-629);
//! * the c4 tip `0df6964b` (581/587 style-44 + 592/607/615 style-16),
//!   the c8 tip `708522fc` (ADVANCED past `3705670d` — the new commits
//!   are the crawler finals/docs, the 606 arm unchanged), the e26 tip
//!   `048cca6c` (604/605 ladybugs) = 8 INLINE arms;
//! * the **g10 tip `5dee2865`** (NEW this census): its
//!   `npc_findframe_pet.rs` landed the town-pet humanoid-group arm —
//!   `npc.rs::find_frame` routes TownCat/TownDog/TownBunny + the eight
//!   town slimes to `town_pet_find_frame` = **11** of the 130
//!   (637/638/656/670/678-684); the `687 BoundTownSlimeYellow` is NOT
//!   in the arm's match — it stays unclaimed;
//! * the f26 slice-7g pushed claims (435/436/631/632/635) = 5;
//! * the d41 tip `5d036721` (the salamanders) greps ZERO for this
//!   slice's eight ids + kind names; the d40 tip `8fe1d6d3` stops at
//!   490; the d43 tip `a725acb0` IS the wave-3 base; every pushed
//!   findframe module seam + the g8/g9/g10/f17/f18 wire lanes grep
//!   ZERO for the eight (mechanical `\b<id> =>` + kind-name scan —
//!   the ONLY kind-name hits are the `creative_sacrifices.rs`
//!   sacrifice-count rows, data not arms);
//! * this branch's own `npc.rs` has no kind/arm for any of the eight.
//!
//! Census: 130 − 15 − 9 − 38 − 8 = 60 − 11 (g10 pets) − 5 (7g) = **44
//! unclaimed before THIS slice**; − 8 = **36 unclaimed** after (the
//! 7g post-state 55 minus the 11 g10 pets minus this slice's 8). The
//! eight picked ids are the ascending head of the 44: 636, then
//! 639-645. **634 SporeBat stays EXCLUDED** (its flutter arm draws
//! `Main.rand.Next(10)` at `NPC.cs:74844` — the zero-RNG rule; its
//! RNG-seam contract is published below for the d44f seam-fed wave);
//! 637/638 are now g10-claimed (above).
//!
//! The arms:
//!
//! * **636 HallowBoss** — its own case (`NPC.cs:67022-67024`): the
//!   one-line phase pin `frame.Y = AI_120_HallowBoss_IsInPhase2() ?
//!   num : 0` — row 1 in phase 2, row 0 otherwise, on the 2-row sheet
//!   (`Main.cs:67147` entry 6). The phase read itself
//!   (`NPC.cs:45814-45821`) is PURE `ai[3]`: `ai[3] == 1 || ai[3] ==
//!   3` — no world read, no RNG; it rides [`FrameInput::ai3`] and the
//!   machine evaluates the two comparisons verbatim (the extra-7b
//!   resolved-input precedent — the wiring bridge copies `ai[3]`,
//!   nothing to probe). The counter AND sprite are UNTOUCHED (the
//!   C# arm writes only `frame.Y`).
//! * **639-645 the gem squirrels** — the `case 299/538/539/639-645`
//!   squirrel group (`NPC.cs:76458-76512`, labels `:76461-76467`),
//!   the VERBATIM machine copy of the f27 slice-10 squirrel arm (the
//!   7g ONE-group precedent — the wiring merge keeps one): the
//!   grounded ±1 flip (`:76468-76477`), the idle row-0 zero
//!   (`:76484-76489`), the `|vx| * 0.5` THEN `++` accumulator with
//!   the STRICT >6 step (`:76490-76496`) and the `>= count - 1` wrap
//!   back to row **1** (`:76497-76500` — NOT row 0: the walk rows are
//!   1..4 on the 6-row sheet, row 0 the idle pose, row 5 never
//!   walkable); the air row 2 rising (`:76502-76506`) / row 4
//!   falling (`:76507-76511`), both counter-zeroed with the sprite
//!   UNTOUCHED. There is NO lift-to-start-row (the bunny/antlion
//!   lift is absent — a row-0 start walks 0 → 1, the 7e lamia shape).
//!   The aiStyle-3 backpedal branch (`:76478-76483`) is DEAD for
//!   these: `SetDefaults` gives all seven aiStyle 7
//!   (`NPC.cs:17226-17236`, the `type >= 639 && type <= 645` block) —
//!   the documented dead branch, the f27 538/539 precedent (their
//!   style-7 rows `:15877`/`:15889`).
//!
//! Shape: the kinds are NOT in `NpcKind` yet (the d43 roster wave
//! owns them), so — the `npc_findframe_extra7g.rs` precedent — the
//! frame machines land first as pure functions over a small
//! [`FrameState`]/[`FrameInput`] pair. The wiring seam is
//! [`find_frame_kind_extra7h`]: when a kind joins `NpcKind`, its
//! `find_frame` arm copies the state fields out, runs one tick and
//! copies them back (the `Npc::find_frame_wave_a` bridge precedent).
//!
//! RNG: NONE — the mechanical grep over the cited ranges (each ≤ 80
//! lines: `:67022-67024`, `:45814-45821`, `:76468-76512`) = ZERO
//! `Main.rand`/`.Next(`/`NextFloat`/`NextDouble`. World reads: NONE —
//! the phase read is pure `ai[3]`; the squirrel body reads only
//! velocity/direction/aiStyle (the dead branch's aiStyle rides the
//! SetDefaults fact, not a seam).
//!
//! Row convention: the port tracks frame ROW indices, not pixels —
//! the C#'s `num` cancels out of every comparison. The per-kind row
//! counts are `Main.npcFrameCount` (`Main.cs:67082`'s 697-entry
//! initializer): 636 = 2 (`:67147` entry 6), 639-645 = 6 (`:67148`
//! entries 1-7) — counted entry-by-entry in
//! [`FindFrameExtra7hKind::frame_count`] and cross-checked against
//! `render/entities.rs::NPC_FRAME_COUNT`.
//!
//! ## The 634 SporeBat RNG-seam contract (published for the d44f
//! ## seam-fed wave — NOT ported here)
//!
//! 634 rides the floater group `case 182/210/211/253/316/634/662/664`
//! (`NPC.cs:74830-74885`, label `:74835`). The arm's ONLY RNG draw is
//! `NPC.cs:74844`: `if (this.type == 634 && Main.rand.Next(10) == 0)`
//! — a per-tick 1-in-10 spore-dust spawn (Dust type 165, `:74846`,
//! the velocity `*= 0.1f` + `noGravity` at `:74847-74848`). Contract
//! for the seam-fed wave:
//!
//! * The wiring bridge draws `Main.rand.Next(10)` EVERY tick for
//!   type 634 ONLY (the `&&` short-circuits for every other group
//!   member — 182/210/211/253/316/662/664 draw NOTHING on this arm;
//!   the draw order rides `UpdateNPC`'s AI → FindFrame sequence,
//!   `NPC.cs:90423` → `:90511`, after the AI's own draws), and feeds
//!   the RESOLVED boolean `spore_dust_draw_is_zero` into FrameInput
//!   (the 7d SolarFlare resolved-draw precedent — there the latch
//!   draws row-first/counter-second; here ONE draw per tick).
//! * The dust spawn itself (`Dust.NewDust` + the two field writes)
//!   is a bridge-side effect keyed on the same boolean — the frame
//!   machine never touches dust.
//! * The frame machine after the seam is zero-RNG: the vx-sign
//!   sprite gates (`:74850-74857` — NOT direction: sign-gated like
//!   the 7c antlion), `rotation = vx * 0.1f` (`:74858`, the npc.rs
//!   rotation-write precedent — the 7d squidhead), the bare
//!   `++frameCounter` (`:74864` — the 210/211 second ++ does NOT
//!   apply), the >= 6 step (`:74875-74879`, num291 = 6 — the type-48
//!   5 override does NOT apply), and the wrap at `num292 = count − 1`
//!   (`:74867-74870` — the 49/51/60/634 decrement DOES apply: the
//!   flap cycles rows 0..3 on the 5-row sheet, row 4 never flapped;
//!   `:74880-74883` wraps to 0). `Main.npcFrameCount[634]` = 5
//!   (`Main.cs:67147` entry 4).
//! * The `type == 60` dust block (`:74839-74843`) is NOT 634's — the
//!   floater group's other members' seams own their own blocks.

// The wiring is a later round (the kinds are not in `NpcKind` yet) —
// the `npc_findframe_extra7g.rs` precedent.
#![allow(dead_code)]

/// The extra-wave-7h kinds, keyed by their VANILLA `NPC.type`
/// (`ID/NPCID.cs`'s `public const short` rows).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FindFrameExtra7hKind {
    /// `NPCID.HallowBoss` = 636 (`NPCID.cs:11949`) — its own case
    /// (`NPC.cs:67022-67024`), the ai[3] phase-2 row pin.
    HallowBoss,
    /// `NPCID.GemSquirrelAmethyst` = 639 (`NPCID.cs:11955`) — the
    /// 299/538/539/639-645 squirrel group (`NPC.cs:76461`).
    GemSquirrelAmethyst,
    /// `NPCID.GemSquirrelTopaz` = 640 (`NPCID.cs:11957`) — the same
    /// group (`NPC.cs:76462`).
    GemSquirrelTopaz,
    /// `NPCID.GemSquirrelSapphire` = 641 (`NPCID.cs:11959`) — the
    /// same group (`NPC.cs:76463`).
    GemSquirrelSapphire,
    /// `NPCID.GemSquirrelEmerald` = 642 (`NPCID.cs:11961`) — the
    /// same group (`NPC.cs:76464`).
    GemSquirrelEmerald,
    /// `NPCID.GemSquirrelRuby` = 643 (`NPCID.cs:11963`) — the same
    /// group (`NPC.cs:76465`).
    GemSquirrelRuby,
    /// `NPCID.GemSquirrelDiamond` = 644 (`NPCID.cs:11965`) — the
    /// same group (`NPC.cs:76466`).
    GemSquirrelDiamond,
    /// `NPCID.GemSquirrelAmber` = 645 (`NPCID.cs:11967`) — the same
    /// group (`NPC.cs:76467`).
    GemSquirrelAmber,
}

impl FindFrameExtra7hKind {
    /// The vanilla `NPC.type` id.
    pub fn vanilla_type(self) -> i16 {
        match self {
            Self::HallowBoss => 636,
            Self::GemSquirrelAmethyst => 639,
            Self::GemSquirrelTopaz => 640,
            Self::GemSquirrelSapphire => 641,
            Self::GemSquirrelEmerald => 642,
            Self::GemSquirrelRuby => 643,
            Self::GemSquirrelDiamond => 644,
            Self::GemSquirrelAmber => 645,
        }
    }

    /// `Main.npcFrameCount[type]` (`Main.cs:67082`'s initializer,
    /// counted entry-by-entry): 636 = 2 (`:67147` entry 6), 639-645
    /// = 6 (`:67148` entries 1-7).
    pub fn frame_count(self) -> u32 {
        match self {
            Self::HallowBoss => 2,
            _ => 6,
        }
    }
}

/// The vanilla-id wiring seam: `Some(kind)` for exactly this wave's
/// ids (the `find_frame_kind_extra7g` precedent). The wiring round
/// hooks it at the `npc.rs::find_frame` head by `cs_id`.
pub fn find_frame_kind_extra7h(vanilla_type: i16) -> Option<FindFrameExtra7hKind> {
    Some(match vanilla_type {
        636 => FindFrameExtra7hKind::HallowBoss,
        639 => FindFrameExtra7hKind::GemSquirrelAmethyst,
        640 => FindFrameExtra7hKind::GemSquirrelTopaz,
        641 => FindFrameExtra7hKind::GemSquirrelSapphire,
        642 => FindFrameExtra7hKind::GemSquirrelEmerald,
        643 => FindFrameExtra7hKind::GemSquirrelRuby,
        644 => FindFrameExtra7hKind::GemSquirrelDiamond,
        645 => FindFrameExtra7hKind::GemSquirrelAmber,
        _ => return None,
    })
}

/// The `NPC` fields `FindFrame` reads/writes for these arms:
/// `frame.Y / num` (the row), `frameCounter` and `spriteDirection`.
/// The wiring step copies them to/from `Npc`. `rotation`/`local_ai`
/// are carried for the uniform 7g wiring copy, never read/written
/// here.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FrameState {
    /// `frame.Y / num` — the sheet row.
    pub row: u32,
    /// `NPC.frameCounter` (the C# field is a `double`, `NPC.cs:6485`;
    /// the port's f32 is the existing `npc.rs` convention — the
    /// integer/half accumulators here are exact in both).
    pub counter: f32,
    /// `NPC.spriteDirection` (±1).
    pub sprite_dir: i32,
    /// `NPC.rotation` — no arm writes it this wave.
    pub rotation: f32,
    /// `NPC.localAI[0]`/`[1]` — never read/written this wave; carried
    /// for the uniform wiring copy.
    pub local_ai: [f32; 2],
}

/// The per-tick inputs the arms read: `velocity`, `direction` and
/// the Hallow Boss `ai[3]` phase slot (the pure `NPC.cs:45814-45821`
/// read — no world probe). No RNG draws this wave; the squirrel
/// group never reads `ai3`.
#[derive(Clone, Copy, Debug)]
pub struct FrameInput {
    pub vx: f32,
    pub vy: f32,
    /// `Entity.direction` (±1; 0 leaves `spriteDirection` alone —
    /// every copy this wave is ±1-gated).
    pub direction: i32,
    /// `NPC.ai[3]` — the Hallow Boss phase slot: 1 or 3 = phase 2
    /// (`NPC.cs:45816-45820`). The squirrels never read it.
    pub ai3: f32,
}

impl Default for FrameInput {
    fn default() -> Self {
        Self {
            vx: 0.0,
            vy: 0.0,
            direction: 1,
            ai3: 0.0,
        }
    }
}

/// `NPC.AI_120_HallowBoss_IsInPhase2` (`NPC.cs:45814-45821`) — the
/// VERBATIM pure read: `ai[3] == 1f || ai[3] == 3f`.
fn hallow_boss_in_phase2(ai3: f32) -> bool {
    ai3 == 1.0 || ai3 == 3.0
}

/// `NPC.FindFrame` (`NPC.cs:66684`) for the extra-wave-7h kinds — one
/// tick.
pub fn find_frame_extra7h(kind: FindFrameExtra7hKind, s: &mut FrameState, i: &FrameInput) {
    match kind {
        // `case 636` (`NPC.cs:67022-67024`): the one-line phase pin —
        // `frame.Y = AI_120_HallowBoss_IsInPhase2() ? num : 0`. Row 1
        // in phase 2 (ai[3] 1 or 3), row 0 otherwise; the counter and
        // sprite are UNTOUCHED (the arm writes only frame.Y).
        FindFrameExtra7hKind::HallowBoss => {
            s.row = if hallow_boss_in_phase2(i.ai3) { 1 } else { 0 };
        }
        // `case 639-645` in the squirrel group (`NPC.cs:76461-76467`
        // → `:76458-76512`, shared with 299/538/539 — 538/539 are
        // f27-s10-claimed, the VERBATIM machine copy, the 7g ONE-group
        // precedent; 299 rides the wave-B roster): the grounded ±1
        // flip (`:76468-76477`), the idle row-0 zero (`:76484-76489`),
        // the `|vx| * 0.5` THEN `++` accumulator with the STRICT >6
        // step (`:76490-76496`) and the `>= count - 1` wrap back to
        // row 1 (`:76497-76500` — rows 1..4 walkable on the 6-row
        // sheet, row 5 never); the air row 2 rising (`:76502-76506`)
        // / row 4 falling (`:76507-76511`), both counter-zeroed. NO
        // lift-to-start-row (a row-0 start walks 0 → 1 — the 7e lamia
        // shape). The aiStyle-3 backpedal branch (`:76478-76483`) is
        // DEAD (all seven aiStyle 7, `NPC.cs:17226-17236`) — the
        // documented dead branch, the f27 538/539 precedent.
        _ => {
            if i.vy == 0.0 {
                // `:76468-76477`.
                if i.direction == 1 {
                    s.sprite_dir = 1;
                }
                if i.direction == -1 {
                    s.sprite_dir = -1;
                }
                // `:76484-76489`.
                if i.vx == 0.0 {
                    s.row = 0;
                    s.counter = 0.0;
                    return;
                }
                // `:76490-76496`.
                s.counter += i.vx.abs() * 0.5;
                s.counter += 1.0;
                if s.counter > 6.0 {
                    s.row += 1;
                    s.counter = 0.0;
                }
                // `:76497-76500`.
                if s.row >= kind.frame_count() - 1 {
                    s.row = 1;
                }
            } else if i.vy < 0.0 {
                // `:76502-76506`.
                s.counter = 0.0;
                s.row = 2;
            } else {
                // `:76507-76511`.
                s.counter = 0.0;
                s.row = 4;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SQUIRRELS: [FindFrameExtra7hKind; 7] = [
        FindFrameExtra7hKind::GemSquirrelAmethyst,
        FindFrameExtra7hKind::GemSquirrelTopaz,
        FindFrameExtra7hKind::GemSquirrelSapphire,
        FindFrameExtra7hKind::GemSquirrelEmerald,
        FindFrameExtra7hKind::GemSquirrelRuby,
        FindFrameExtra7hKind::GemSquirrelDiamond,
        FindFrameExtra7hKind::GemSquirrelAmber,
    ];

    fn input(vx: f32, vy: f32) -> FrameInput {
        FrameInput { vx, vy, ..Default::default() }
    }

    fn facing(vx: f32, vy: f32, direction: i32) -> FrameInput {
        FrameInput { vx, vy, direction, ..Default::default() }
    }

    fn rows_over(kind: FindFrameExtra7hKind, i: FrameInput, ticks: usize) -> Vec<u32> {
        let mut s = FrameState::default();
        let mut out = Vec::new();
        for _ in 0..ticks {
            find_frame_extra7h(kind, &mut s, &i);
            out.push(s.row);
        }
        out
    }

    /// The frame-count table agrees with the port's verbatim
    /// `NPC_FRAME_COUNT` (`Main.cs:67082` initializer cross-check).
    #[test]
    fn f26s7h_frame_counts_match_the_port_table() {
        let table = crate::render::entities::NPC_FRAME_COUNT;
        for k in [FindFrameExtra7hKind::HallowBoss]
            .into_iter()
            .chain(SQUIRRELS)
        {
            assert_eq!(
                k.frame_count(),
                table[k.vanilla_type() as usize],
                "{k:?} ({})",
                k.vanilla_type()
            );
        }
    }

    /// The Hallow Boss phase pin: ai[3] 1/3 → row 1, every other
    /// value → row 0 (the VERBATIM `:45816-45820` two-equality read —
    /// 2 is NOT phase 2, the IsGenuinelyEnraged slot); the counter
    /// and sprite are UNTOUCHED, velocity/direction irrelevant.
    #[test]
    fn f26s7h_the_hallow_boss_phase_pin() {
        let k = FindFrameExtra7hKind::HallowBoss;
        for (ai3, row) in [
            (0.0, 0), (1.0, 1), (2.0, 0), (3.0, 1), (4.0, 0), (-1.0, 0), (1.5, 0),
        ] {
            let mut s = FrameState { row: 1, counter: 7.0, sprite_dir: -1, ..Default::default() };
            find_frame_extra7h(k, &mut s, &FrameInput { ai3, ..facing(3.0, -2.0, -1) });
            assert_eq!((s.row, s.counter, s.sprite_dir), (row, 7.0, -1), "ai3 {ai3}");
        }
    }

    /// The squirrel walk: vx 2 adds EXACTLY 2.0 a tick (2.0f * 0.5f
    /// == 1.0f, then ++), the STRICT >6 step lands every 3rd tick —
    /// NO lift, the row-0 start walks 0 → 1; the >= 5 wrap goes back
    /// to row 1 (NOT 0), so rows 1..4 cycle and row 5 never appears.
    #[test]
    fn f26s7h_the_squirrel_walk_wrap_to_one() {
        for k in SQUIRRELS {
            // `NPC.cs:76490-76497`: +|vx|*0.5 then ++ = +2/tick with the
            // STRICT > 6 step -- rows 0 (3 ticks), 1 (4), 2 (4), 3 (4),
            // 4 (tick 16).
            assert_eq!(
                rows_over(k, input(2.0, 0.0), 16),
                vec![0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4]
            );
            // The wrap edge: row 4 + counter 6 → +2 = 8 > 6 → row 5
            // → >= 5 → 1, counter 0.
            let mut s = FrameState { row: 4, counter: 6.0, ..Default::default() };
            find_frame_extra7h(k, &mut s, &input(2.0, 0.0));
            assert_eq!((s.row, s.counter), (1, 0.0));
        }
    }

    /// The squirrel's slower feed: vx 1 adds 1.5 a tick — the >6
    /// step lands every 4th tick (6.0 is NOT > 6.0 — the half-step
    /// edges exact in f32 and the C# double alike).
    #[test]
    fn f26s7h_the_squirrel_half_step_cadence() {
        let k = FindFrameExtra7hKind::GemSquirrelAmethyst;
        // `NPC.cs:76490-76497`: +1.5/tick with the strict > 6 step (the
        // 6.0 tick does NOT step) -- row 0 holds ticks 1..4, row 1
        // ticks 5..9, row 2 ticks 10..12.
        assert_eq!(
            rows_over(k, input(1.0, 0.0), 12),
            vec![0, 0, 0, 0, 1, 1, 1, 1, 1, 2, 2, 2]
        );
        // The strict edge: counter 4.5 + 1.5 = 6.0 does NOT step.
        let mut s = FrameState { row: 2, counter: 4.5, ..Default::default() };
        find_frame_extra7h(k, &mut s, &input(1.0, 0.0));
        assert_eq!((s.row, s.counter), (2, 6.0));
    }

    /// The squirrel idle/air pins: the idle zeroes BOTH row and
    /// counter; the air pins row 2 rising / row 4 falling with the
    /// counter zeroed and the sprite UNTOUCHED (the grounded-only
    /// ±1 flip); direction 0 leaves the sprite alone.
    #[test]
    fn f26s7h_the_squirrel_idle_and_air_pins() {
        for k in SQUIRRELS {
            let mut s = FrameState { row: 3, counter: 5.0, sprite_dir: -1, ..Default::default() };
            find_frame_extra7h(k, &mut s, &facing(0.0, 0.0, 0));
            assert_eq!((s.row, s.counter, s.sprite_dir), (0, 0.0, -1));
            let mut s = FrameState { row: 1, counter: 5.0, sprite_dir: 1, ..Default::default() };
            find_frame_extra7h(k, &mut s, &facing(2.0, -2.0, -1));
            assert_eq!((s.row, s.counter, s.sprite_dir), (2, 0.0, 1));
            let mut s = FrameState { row: 1, counter: 5.0, sprite_dir: -1, ..Default::default() };
            find_frame_extra7h(k, &mut s, &facing(-2.0, 2.0, 1));
            assert_eq!((s.row, s.counter, s.sprite_dir), (4, 0.0, -1));
            // The grounded flip is ±1-gated.
            let mut s = FrameState { sprite_dir: 1, ..Default::default() };
            find_frame_extra7h(k, &mut s, &facing(-2.0, 0.0, -1));
            assert_eq!(s.sprite_dir, -1);
        }
    }

    /// The seam maps exactly this wave's 8 ids — the g10-claimed
    /// pets 637/638/656, the RNG-excluded 634, the 7g-claimed
    /// 631/635, the f27-claimed 538/539, the wave-B 299 and the
    /// no-case 663 stay `None`.
    #[test]
    fn f26s7h_the_seam_maps_exactly_the_slice() {
        for k in [FindFrameExtra7hKind::HallowBoss]
            .into_iter()
            .chain(SQUIRRELS)
        {
            let id = k.vanilla_type();
            assert_eq!(find_frame_kind_extra7h(id), Some(k));
        }
        for id in [299, 538, 539, 631, 634, 635, 637, 638, 646, 652, 656, 663] {
            assert_eq!(find_frame_kind_extra7h(id), None, "type {id}");
        }
    }

    /// The 8-kind strip sweep: every kind ticks 27 input combos
    /// (vx 0/1/3 × vy 0/-2/2 × direction 1/-1/0) for 40 ticks each
    /// from a fresh state — every row stays inside its own sheet
    /// (the 7g sweep shape; the squirrel wrap exercises inside the
    /// window).
    #[test]
    fn f26s7h_the_8_kind_strip_sweep_stays_in_sheet() {
        for k in [FindFrameExtra7hKind::HallowBoss]
            .into_iter()
            .chain(SQUIRRELS)
        {
            let mut combo = 0;
            for vx in [0.0f32, 1.0, 3.0] {
                for vy in [0.0f32, -2.0, 2.0] {
                    for direction in [1i32, -1, 0] {
                        let mut s = FrameState::default();
                        let i = FrameInput { vx, vy, direction, ..Default::default() };
                        for _ in 0..40 {
                            find_frame_extra7h(k, &mut s, &i);
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

    /// The Hallow Boss phase sweep: ai3 -1/0/1/2/3/4 × 18 velocity/
    /// direction combos — the row is the pure phase pin, the counter
    /// and sprite never touched (the arm's verbatim invariant).
    #[test]
    fn f26s7h_the_hallow_boss_phase_sweep_is_pure() {
        let k = FindFrameExtra7hKind::HallowBoss;
        for ai3 in [-1.0f32, 0.0, 1.0, 2.0, 3.0, 4.0] {
            for vx in [0.0f32, 2.0, -2.0] {
                for vy in [0.0f32, -2.0, 2.0] {
                    for direction in [1i32, -1] {
                        let mut s = FrameState { row: 1, counter: 5.0, sprite_dir: -1, ..Default::default() };
                        let i = FrameInput { vx, vy, direction, ai3 };
                        find_frame_extra7h(k, &mut s, &i);
                        assert_eq!(s.row, if ai3 == 1.0 || ai3 == 3.0 { 1 } else { 0 });
                        assert_eq!((s.counter, s.sprite_dir), (5.0, -1));
                    }
                }
            }
        }
    }
}
