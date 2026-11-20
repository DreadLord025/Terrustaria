//! World-side `Gore.NewGore*` sites, wave B: the `WorldGen.CheckPot`
//! gore switch (`WorldGen.cs:56878-56954`). W4-g17 (code-only mode):
//! the module + witnesses land first; the dispatch claim (the edit
//! lane's `check_pot` → `PotEvent`, `tr_core::edit::kill`) is the
//! wiring wave — the `npc_hit_effect_a.rs` (W4-d55) precedent.
//!
//! ## Scope — this commit's sites (source order)
//!
//! | # | C# site | arm |
//! | --- | --- | --- |
//! | 1-3 | `:56881`-`:56883` | style 0 — the clay pot 51/52/53 |
//! | 4-6 | `:56886`-`:56888` | style 1 — 166/167/168 |
//! | 7-9 | `:56891`-`:56893` | style 2 — 169/170/171 |
//!
//! Style 3 (`:56896-56898`), the 2-gore rows (styles 4-15,
//! `:56903-56920`) and the `genRand` rows (styles 25-36,
//! `:56930-56950`) land in the follow-up commits of this wave.
//! Styles 16-24 spawn NOTHING (`:56924-56927`'s break).
//!
//! ## RNG contract
//!
//! The fixed-type rows draw NOTHING at the arm level — the types are
//! data; each spawn's kicks roll inside `GorePool::new_gore` on the
//! caller's `Main.rand` stream (Y `Next(10,31)` then X `Next(-20,21)`,
//! `Gore.cs:1327-1328`). The `genRand` rows (follow-up commits) draw
//! their type picks on `WorldGen.genRand` — a SEPARATE stream from the
//! kicks' `Main.rand` (the port has no runtime genRand, `edit.rs:900`;
//! the caller hands it in).
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

/// `WorldGen.CheckPot`'s gore switch (`WorldGen.cs:56878-56954`),
/// styles 0-2 (this commit): the three 3-gore rows. `style` is the
/// pot's `frameY / 36` row (`:56810-56816`); `(i, j)` is the trigger
/// cell — the C# spawns every gore at `(i*16, j*16)` with zero
/// velocity and scale 1.
pub fn pot_break_gores(
    style: i32,
    i: i32,
    j: i32,
    gore: &mut GorePool,
    rand: &mut UnifiedRandom,
    world: &World,
) {
    let pos = Vec2::new((i * 16) as f32, (j * 16) as f32);
    // `:56878-56894` — the fixed-type 3-gore rows.
    let types: &[i32] = match style {
        0 => &[51, 52, 53],     // :56881-56883
        1 => &[166, 167, 168],  // :56886-56888
        2 => &[169, 170, 171],  // :56891-56893
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

    /// `WorldGen.cs:56881-56893` — the three 3-gore rows, the types in
    /// source order, every gore at the trigger cell's top-left.
    #[test]
    fn w4_g17_the_pot_rows_match_the_csharp() {
        let rows = [
            (0, [51, 52, 53]),
            (1, [166, 167, 168]),
            (2, [169, 170, 171]),
        ];
        for (style, types) in rows {
            let w = air();
            let mut gore = GorePool::new();
            let mut rand = UnifiedRandom::new(4);
            pot_break_gores(style, 7, 9, &mut gore, &mut rand, &w);
            let got: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
            assert_eq!(got, types.to_vec(), "style {style}");
            for g in &gore.slots()[..3] {
                assert_eq!(g.position, Vec2::new(112.0, 144.0));
                assert!(g.active);
            }
        }
    }

    /// The fixed rows draw NOTHING but the kicks: 2 draws per gore on
    /// the caller's stream, in the pool's Y-then-X order.
    #[test]
    fn w4_g17_the_pot_rows_draw_kicks_only() {
        let w = air();
        let mut gore = GorePool::new();
        let mut rand = UnifiedRandom::new(8);
        pot_break_gores(2, 7, 9, &mut gore, &mut rand, &w);
        let mut twin = UnifiedRandom::new(8);
        for _ in 0..3 {
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
        }
        assert_eq!(rand.peek(), twin.peek(), "kicks only");
    }

    /// Styles without a row (this commit: 3+, incl. the 16-24 no-gore
    /// band) spawn nothing and draw nothing.
    #[test]
    fn w4_g17_the_unported_styles_spawn_nothing() {
        let w = air();
        for style in [3, 4, 7, 16, 24, 25, 34, 99, -1] {
            let mut gore = GorePool::new();
            let mut rand = UnifiedRandom::new(8);
            pot_break_gores(style, 7, 9, &mut gore, &mut rand, &w);
            assert!(!gore.slots().iter().any(|g| g.active), "style {style}");
            let fresh = UnifiedRandom::new(8);
            assert_eq!(rand.peek(), fresh.peek(), "style {style} drew nothing");
        }
    }
}
