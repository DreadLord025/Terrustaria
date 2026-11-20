//! World-side `Gore.NewGore*` sites, wave B: the `WorldGen.CheckPot`
//! gore switch (`WorldGen.cs:56878-56954`). W4-g17 (code-only mode):
//! the module + witnesses land first; the dispatch claim (the edit
//! lane's `check_pot` → `PotEvent`, `tr_core::edit::kill`) is the
//! wiring wave — the `npc_hit_effect_a.rs` (W4-d55) precedent.
//!
//! ## Scope — this wave's sites so far (source order)
//!
//! | # | C# site | arm |
//! | --- | --- | --- |
//! | 1-3 | `:56881`-`:56883` | style 0 — the clay pot 51/52/53 |
//! | 4-6 | `:56886`-`:56888` | style 1 — 166/167/168 |
//! | 7-9 | `:56891`-`:56893` | style 2 — 169/170/171 |
//! | 10-12 | `:56896`-`:56898` | style 3 — 172/173/174 |
//! | 13-14 | `:56903`-`:56904` | styles 4-6 — 197/198 |
//! | 15-16 | `:56909`-`:56910` | styles 7-9 — 199/200 |
//! | 17-18 | `:56914`-`:56915` | styles 10-12 — 201/202 |
//! | 19-20 | `:56919`-`:56920` | styles 13-15 — 203/204 |
//! | 21-22 | `:56930`-`:56931` | styles 25-27 — 2× genRand.Next(217,220) |
//! | 23-24 | `:56935`-`:56936` | styles 28-30 — 2× genRand.Next(315,317) |
//! | 25 | `:56943` | styles 31-33 — genRand.Next(2,5) gores of 698+Next(6) |
//! | 26-28 | `:56948`-`:56950` | styles 34-36 — 1122/1123/1124 |
//!
//! Styles 16-24 spawn NOTHING (`:56924-56927`'s break). The switch is
//! COMPLETE with this commit.
//!
//! ## RNG contract
//!
//! The fixed-type rows draw NOTHING at the arm level — the types are
//! data; each spawn's kicks roll inside `GorePool::new_gore` on the
//! caller's `Main.rand` stream (Y `Next(10,31)` then X `Next(-20,21)`,
//! `Gore.cs:1327-1328`). The `genRand` rows (styles 25-33) draw their
//! type picks on `WorldGen.genRand` — a SEPARATE stream from the
//! kicks' `Main.rand` (the port has no runtime genRand, `edit.rs:900`;
//! the caller hands it in). The type draw is an ARGUMENT: it rides
//! genRand BEFORE that gore's kicks; the styles 31-33 count roll
//! (`Next(2,5)`) rides genRand FIRST.
//!
//! ## Documented drops / stand-ins (this wave)
//!
//! * `CheckPot`'s 2x2 validation, the `destroyObject` latch, the
//!   break-sound pick (`:56856-56867`, already ported as
//!   `tr_core::edit::kill::pot_break_sound`), the `KillTile` loop and
//!   the `SpawnThingsFromPot` loot tail are the edit lane's — this
//!   module takes the resolved `style` (`frameY / 36`, `:56810-56816`)
//!   and the TRIGGER cell `(i, j)` (the C# spawns at `i*16, j*16`,
//!   NOT the pot's 2x2 origin).
//! * The post-spawn writes land on the returned slot, the sentinel
//!   included (the `npc_hit_effect_a.rs` convention); these rows have
//!   no post-writes.
//! * netMode: single-player port, the pool's own convention.
//!
//! See docs/ANALYSIS.md "The world gore sites (W4-g17, wave B)".
#![allow(dead_code)] // wiring deferred (code-only mode) — the CheckPot/PotEvent claim is the follow-up round

use glam::Vec2;
use tr_core::gore::GorePool;
use tr_core::{UnifiedRandom, World};

/// `WorldGen.CheckPot`'s gore switch (`WorldGen.cs:56878-56954`) —
/// COMPLETE. `style` is the pot's `frameY / 36` row (`:56810-56816`);
/// `(i, j)` is the trigger cell — the C# spawns every gore at
/// `(i*16, j*16)` with zero velocity and scale 1. `rand` is the
/// kicks' `Main.rand` stream; `gen_rand` is `WorldGen.genRand` — the
/// SEPARATE stream the styles 25-33 type picks draw on (the port has
/// no runtime genRand, `edit.rs:900`; the caller hands it in).
pub fn pot_break_gores(
    style: i32,
    i: i32,
    j: i32,
    gore: &mut GorePool,
    rand: &mut UnifiedRandom,
    gen_rand: &mut UnifiedRandom,
    world: &World,
) {
    let pos = Vec2::new((i * 16) as f32, (j * 16) as f32);
    // `:56878-56921` — the fixed-type rows (3-gore styles 0-3, then
    // the 2-gore bands 4-6 / 7-9 / 10-12 / 13-15).
    let types: &[i32] = match style {
        0 => &[51, 52, 53],     // :56881-56883
        1 => &[166, 167, 168],  // :56886-56888
        2 => &[169, 170, 171],  // :56891-56893
        3 => &[172, 173, 174],  // :56896-56898
        4..=6 => &[197, 198],   // :56903-56904
        7..=9 => &[199, 200],   // :56909-56910
        10..=12 => &[201, 202], // :56914-56915
        13..=15 => &[203, 204], // :56919-56920
        // `:56924-56927` — styles 16-24 break with NO gores.
        16..=24 => return,
        // `:56928-56937` — the genRand type-pick pairs. The type draw
        // is an ARGUMENT: it rides genRand BEFORE the kicks' Main.rand.
        25..=27 => {
            for _ in 0..2 {
                let ty = gen_rand.next_range(217, 220); // :56930-56931
                gore.new_gore(rand, world, pos, Vec2::ZERO, ty, 1.0);
            }
            return;
        }
        28..=30 => {
            for _ in 0..2 {
                let ty = gen_rand.next_range(315, 317); // :56935-56936
                gore.new_gore(rand, world, pos, Vec2::ZERO, ty, 1.0);
            }
            return;
        }
        // `:56938-56945` — the count roll FIRST, then per gore the
        // 698+Next(6) type (genRand) + the kicks (Main.rand).
        31..=33 => {
            let num6 = gen_rand.next_range(2, 5); // :56940
            for _ in 0..num6 {
                let ty = 698 + gen_rand.next_max(6); // :56943
                gore.new_gore(rand, world, pos, Vec2::ZERO, ty, 1.0);
            }
            return;
        }
        34..=36 => &[1122, 1123, 1124], // :56948-56950
        _ => return,
    };
    for &ty in types {
        gore.new_gore(rand, world, pos, Vec2::ZERO, ty, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn air() -> World {
        World::new("t".into(), 200, 200)
    }

    /// `WorldGen.cs:56881-56898` — the four 3-gore rows, the types in
    /// source order, every gore at the trigger cell's top-left.
    #[test]
    fn w4_g17_the_pot_rows_match_the_csharp() {
        let rows = [
            (0, [51, 52, 53]),
            (1, [166, 167, 168]),
            (2, [169, 170, 171]),
            (3, [172, 173, 174]),
        ];
        for (style, types) in rows {
            let w = air();
            let mut gore = GorePool::new();
            let mut rand = UnifiedRandom::new(4);
            let mut gen_rand = UnifiedRandom::new(100);
            pot_break_gores(style, 7, 9, &mut gore, &mut rand, &mut gen_rand, &w);
            let got: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
            assert_eq!(got, types.to_vec(), "style {style}");
            for g in &gore.slots()[..3] {
                assert_eq!(g.position, Vec2::new(112.0, 144.0));
                assert!(g.active);
            }
            // The fixed rows never touch genRand.
            let fresh = UnifiedRandom::new(100);
            assert_eq!(gen_rand.peek(), fresh.peek(), "style {style}");
        }
    }

    /// The fixed rows draw NOTHING but the kicks: 2 draws per gore on
    /// the caller's stream, in the pool's Y-then-X order.
    #[test]
    fn w4_g17_the_pot_rows_draw_kicks_only() {
        let w = air();
        let mut gore = GorePool::new();
        let mut rand = UnifiedRandom::new(8);
        let mut gen_rand = UnifiedRandom::new(100);
        pot_break_gores(2, 7, 9, &mut gore, &mut rand, &mut gen_rand, &w);
        let mut twin = UnifiedRandom::new(8);
        for _ in 0..3 {
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
        }
        assert_eq!(rand.peek(), twin.peek(), "kicks only");
    }

    /// `WorldGen.cs:56900-56920` — the 2-gore bands: styles 4-6 share
    /// 197/198, 7-9 share 199/200, 10-12 share 201/202, 13-15 share
    /// 203/204 (the C# range guards, not per-style rows).
    #[test]
    fn w4_g17_the_pot_two_gore_bands_match_the_csharp() {
        let bands = [
            (4, [197, 198]), (5, [197, 198]), (6, [197, 198]),
            (7, [199, 200]), (8, [199, 200]), (9, [199, 200]),
            (10, [201, 202]), (11, [201, 202]), (12, [201, 202]),
            (13, [203, 204]), (14, [203, 204]), (15, [203, 204]),
        ];
        for (style, types) in bands {
            let w = air();
            let mut gore = GorePool::new();
            let mut rand = UnifiedRandom::new(6);
            let mut gen_rand = UnifiedRandom::new(100);
            pot_break_gores(style, 3, 4, &mut gore, &mut rand, &mut gen_rand, &w);
            let got: Vec<i32> = gore.slots()[..2].iter().map(|g| g.gore_type).collect();
            assert_eq!(got, types.to_vec(), "style {style}");
            assert!(!gore.slot(2).active, "style {style} spawns exactly two");
        }
    }

    /// `WorldGen.cs:56924-56927` — styles 16-24 break with NO gores
    /// and draw nothing on either stream; unknown styles too.
    #[test]
    fn w4_g17_the_no_gore_styles_spawn_nothing() {
        let w = air();
        for style in [16, 17, 18, 19, 20, 21, 22, 23, 24, 99, -1] {
            let mut gore = GorePool::new();
            let mut rand = UnifiedRandom::new(8);
            let mut gen_rand = UnifiedRandom::new(100);
            pot_break_gores(style, 7, 9, &mut gore, &mut rand, &mut gen_rand, &w);
            assert!(!gore.slots().iter().any(|g| g.active), "style {style}");
            let fresh_r = UnifiedRandom::new(8);
            let fresh_g = UnifiedRandom::new(100);
            assert_eq!(rand.peek(), fresh_r.peek(), "style {style} rand");
            assert_eq!(gen_rand.peek(), fresh_g.peek(), "style {style} genRand");
        }
    }

    /// `WorldGen.cs:56928-56937` — styles 25-27 pick TWO types from
    /// genRand.Next(217,220), styles 28-30 from Next(315,317). The
    /// type draw rides genRand BEFORE the kicks' Main.rand; the twin
    /// replays both streams in that order.
    #[test]
    fn w4_g17_the_genrand_pairs_draw_the_type_before_the_kicks() {
        for (style, lo, hi) in [(25, 217, 220), (26, 217, 220), (27, 217, 220),
                                (28, 315, 317), (29, 315, 317), (30, 315, 317)] {
            let w = air();
            let mut gore = GorePool::new();
            let mut rand = UnifiedRandom::new(11);
            let mut gen_rand = UnifiedRandom::new(77);
            pot_break_gores(style, 5, 6, &mut gore, &mut rand, &mut gen_rand, &w);
            let mut twin_r = UnifiedRandom::new(11);
            let mut twin_g = UnifiedRandom::new(77);
            for slot in 0..2 {
                let ty = twin_g.next_range(lo, hi);
                assert_eq!(gore.slot(slot).gore_type, ty, "style {style} slot {slot}");
                assert!((lo..hi).contains(&gore.slot(slot).gore_type));
                let _ky = twin_r.next_range(10, 31);
                let _kx = twin_r.next_range(-20, 21);
            }
            assert_eq!(rand.peek(), twin_r.peek(), "style {style} kicks");
            assert_eq!(gen_rand.peek(), twin_g.peek(), "style {style} types");
            assert!(!gore.slot(2).active, "style {style} spawns exactly two");
        }
    }

    /// `WorldGen.cs:56938-56945` — styles 31-33 roll the COUNT first
    /// (genRand.Next(2,5) → 2..4 gores), then per gore 698+Next(6).
    #[test]
    fn w4_g17_the_genrand_count_roll_precedes_the_types() {
        for style in [31, 32, 33] {
            let w = air();
            let mut gore = GorePool::new();
            let mut rand = UnifiedRandom::new(13);
            let mut gen_rand = UnifiedRandom::new(91);
            pot_break_gores(style, 5, 6, &mut gore, &mut rand, &mut gen_rand, &w);
            let mut twin_r = UnifiedRandom::new(13);
            let mut twin_g = UnifiedRandom::new(91);
            let count = twin_g.next_range(2, 5);
            assert!((2..=4).contains(&count));
            for slot in 0..count as usize {
                let ty = 698 + twin_g.next_max(6);
                assert_eq!(gore.slot(slot).gore_type, ty, "style {style} slot {slot}");
                assert!((698..=703).contains(&gore.slot(slot).gore_type));
                let _ky = twin_r.next_range(10, 31);
                let _kx = twin_r.next_range(-20, 21);
            }
            assert!(!gore.slot(count as usize).active, "style {style} count");
            assert_eq!(rand.peek(), twin_r.peek(), "style {style} kicks");
            assert_eq!(gen_rand.peek(), twin_g.peek(), "style {style} count+types");
        }
    }

    /// `WorldGen.cs:56946-56951` — styles 34-36 spawn the fixed
    /// 1122/1123/1124 trio (no genRand draws).
    #[test]
    fn w4_g17_the_high_style_trio_is_fixed() {
        for style in [34, 35, 36] {
            let w = air();
            let mut gore = GorePool::new();
            let mut rand = UnifiedRandom::new(17);
            let mut gen_rand = UnifiedRandom::new(100);
            pot_break_gores(style, 5, 6, &mut gore, &mut rand, &mut gen_rand, &w);
            let got: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
            assert_eq!(got, vec![1122, 1123, 1124], "style {style}");
            let fresh = UnifiedRandom::new(100);
            assert_eq!(gen_rand.peek(), fresh.peek(), "style {style} no genRand");
        }
    }
}
