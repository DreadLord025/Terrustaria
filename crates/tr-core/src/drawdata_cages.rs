//! The critter-cage animator — `Main.AnimateTiles_CritterCages`
//! (`Main.cs:13511-16596`) plus its three helpers `UpdateOwlCageFrames`
//! (`:16598-16604`), `UpdateDragonflyJarFrames` (`:16606-16615`) and the
//! per-slot bodies `UpdateOwlCageFrame` (`:16617-16666`) /
//! `UpdateDragonflyJarFrame` (`:16668-16727`).
//!
//! C# advances the per-slot `Main.*CageFrame` tables (`Main.cs:66901-66972`,
//! `Main.cageFrames = 25`) once per client update at the tail of
//! `AnimateTiles` (`Main.cs:19303`), gated on `Main.critterCage` — a flag the
//! tile draw sets while a cage tile is in view (`TileDrawing.cs:5672` etc.,
//! cleared at the non-solid layer's start, `:474`). The draw side reads the
//! tables through `drawdata_furniture::cage_draw_data`'s `frame_of` lookup
//! (W4-f14 wiring: `render/tiles.rs` after `apply_cases_a`, the C#
//! first-switch position; the chest half rides the same post-pass at the
//! `CacheSpecialDraws_Part2` position, `DrawSingleTile` `:993`).
//!
//! Determinism: every roll rides the caller's [`UnifiedRandom`] — the port's
//! exact `Main.rand` Knuth subtractive stream — in source order, INCLUDING
//! the draws the decompiled C# takes on paths whose result is discarded
//! (the mallard/duck `Next(3)` short-circuit at `:13895`/`:13955`, the bird
//! frame-1 `Next(3)` at `:14397`, the butterfly `Next(3, 16)` at `:16411`,
//! the jellyfish mode rolls). No other RNG is touched.
//!
//! State lifetime: the tables are PROCESS state in C# — allocated once
//! (`Main.cs:66902-66972`) and never re-zeroed per world (the world-load
//! reset `:8393-8394` covers `tileFrame`/`tileFrameCounter` only). The
//! startup warm-up `Initialize_TileAndNPCData2` (`Main.cs:6800-6807`) runs
//! 3600 ticks with `critterCage` forced true before any world loads; the
//! port replays it in [`CritterCageAnimator::new`] on a fresh world-seed
//! `Main.rand` clone (the documented `NamedRngs::global` seed deviation) —
//! the live stream is NOT consumed, so the pre-wiring ambience/animator
//! interleave stays bit-for-bit until a cage tile enters the view.

use crate::drawdata_furniture::{CageAnim, CAGE_FRAMES};
use crate::unified_random::UnifiedRandom;

const N: usize = CAGE_FRAMES as usize;

/// One 1-D `Main.*CageFrame` table pair (`Main.cs:66902-66972`).
#[derive(Clone, Debug, Default)]
struct CageTable {
    frame: [i32; N],
    counter: [i32; N],
}

/// One 2-D table pair (`butterfly[9, N]`, `dragonfly[7, N]`, `scorpion[2, N]`,
/// `slug[3, N]`, `jellyfish[3, N]` — `Main.cs:66924-66966`).
#[derive(Clone, Debug)]
struct CageTable2<const R: usize> {
    frame: [[i32; N]; R],
    counter: [[i32; N]; R],
}

impl<const R: usize> Default for CageTable2<R> {
    fn default() -> Self {
        Self {
            frame: [[0; N]; R],
            counter: [[0; N]; R],
        }
    }
}

/// `Main.critterCage` (`Main.cs:1445`) plus every `Main.*CageFrame` /
/// `*FrameCounter` / `*Mode` array of `Main.cs:66902-66972`, in declaration
/// order. All tables start zeroed; [`Self::new`] replays the C# startup
/// warm-up so the first drawn frame is the post-init state.
#[derive(Clone, Debug)]
pub struct CritterCageAnimator {
    /// `Main.critterCage` — set by the draw while a cage is in view
    /// (`TileDrawing.cs:5672/:5781/:5838/:5846/:5856/:5874/:5893/:5903`),
    /// cleared at the non-solid layer start (`:474`); gates the tick.
    pub critter_cage: bool,
    bunny: CageTable,
    squirrel: CageTable,
    /// `Main.squirrelCageFrameOrange` (`Main.cs:1455-1457`): animated
    /// (`:13752-13885`) but read by NO draw arm — kept for stream fidelity.
    squirrel_orange: CageTable,
    mallard: CageTable,
    duck: CageTable,
    grebe: CageTable,
    seagull: CageTable,
    bird: CageTable,
    red_bird: CageTable,
    blue_bird: CageTable,
    macaw: CageTable,
    butterfly: CageTable2<9>,
    butterfly_mode: [[u8; N]; 9],
    dragonfly: CageTable2<7>,
    scorpion: CageTable2<2>,
    snail: CageTable,
    snail2: CageTable,
    fish_bowl: CageTable,
    fish_bowl_mode: [u8; N],
    lava_fish_bowl: CageTable,
    frog: CageTable,
    mouse: CageTable,
    turtle: CageTable,
    fairy_jar: CageTable,
    jellyfish: CageTable2<3>,
    jellyfish_mode: [[u8; N]; 3],
    worm: CageTable,
    maggot: CageTable,
    rat: CageTable,
    ladybug: CageTable,
    penguin: CageTable,
    water_strider: CageTable,
    seahorse: CageTable,
    slug: CageTable2<3>,
    owl: CageTable,
    grasshopper: CageTable,
    pufferfish: CageTable,
}

impl Default for CritterCageAnimator {
    fn default() -> Self {
        Self::new(0)
    }
}

impl CritterCageAnimator {
    /// The zero tables (`Main.cs:66902-66972`) plus the startup warm-up
    /// `Initialize_TileAndNPCData2` (`Main.cs:6800-6807`): `critterCage =
    /// true`, 3600 `AnimateTiles_CritterCages()` ticks, `critterCage =
    /// false`. `world_seed` seeds the warm-up's `Main.rand` clone (the C#
    /// uses the process stream seeded `DateTime.Now.Ticks`, `:6463` — the
    /// port's documented world-seed stand-in); the caller's live stream is
    /// untouched.
    pub fn new(world_seed: i32) -> Self {
        let mut a = Self::zeroed();
        let mut rng = UnifiedRandom::new(world_seed);
        a.critter_cage = true;
        for _ in 0..3600 {
            a.animate_critter_cages(&mut rng);
        }
        a.critter_cage = false;
        a
    }

    /// The bare allocation state (`Main.cs:66902-66972`) — every table and
    /// mode zero, `critterCage` false. No warm-up; the vanilla t=0 draw
    /// state the furniture tests pin.
    pub fn zeroed() -> Self {
        Self {
            critter_cage: false,
            bunny: CageTable::default(),
            squirrel: CageTable::default(),
            squirrel_orange: CageTable::default(),
            mallard: CageTable::default(),
            duck: CageTable::default(),
            grebe: CageTable::default(),
            seagull: CageTable::default(),
            bird: CageTable::default(),
            red_bird: CageTable::default(),
            blue_bird: CageTable::default(),
            macaw: CageTable::default(),
            butterfly: CageTable2::default(),
            butterfly_mode: [[0; N]; 9],
            dragonfly: CageTable2::default(),
            scorpion: CageTable2::default(),
            snail: CageTable::default(),
            snail2: CageTable::default(),
            fish_bowl: CageTable::default(),
            fish_bowl_mode: [0; N],
            lava_fish_bowl: CageTable::default(),
            frog: CageTable::default(),
            mouse: CageTable::default(),
            turtle: CageTable::default(),
            fairy_jar: CageTable::default(),
            jellyfish: CageTable2::default(),
            jellyfish_mode: [[0; N]; 3],
            worm: CageTable::default(),
            maggot: CageTable::default(),
            rat: CageTable::default(),
            ladybug: CageTable::default(),
            penguin: CageTable::default(),
            water_strider: CageTable::default(),
            seahorse: CageTable::default(),
            slug: CageTable2::default(),
            owl: CageTable::default(),
            grasshopper: CageTable::default(),
            pufferfish: CageTable::default(),
        }
    }

    /// The `frame_of(table, row, idx)` lookup
    /// [`crate::drawdata_furniture::cage_draw_data`] consumes — the C#
    /// `Main.<animal>CageFrame[row, idx]` reads of the four `case` groups
    /// (`TileDrawing.cs:5674-5753/:5783-5830/:5833-5897/:5898-5907`).
    pub fn frame_of(&self, table: CageAnim, row: i32, idx: i32) -> i32 {
        let i = idx as usize;
        let r = row as usize;
        match table {
            CageAnim::Bunny => self.bunny.frame[i],
            CageAnim::Turtle => self.turtle.frame[i],
            CageAnim::Owl => self.owl.frame[i],
            CageAnim::Squirrel => self.squirrel.frame[i],
            CageAnim::Mallard => self.mallard.frame[i],
            CageAnim::Duck => self.duck.frame[i],
            CageAnim::Grebe => self.grebe.frame[i],
            CageAnim::Seagull => self.seagull.frame[i],
            CageAnim::Bird => self.bird.frame[i],
            CageAnim::BlueBird => self.blue_bird.frame[i],
            CageAnim::RedBird => self.red_bird.frame[i],
            CageAnim::Macaw => self.macaw.frame[i],
            CageAnim::Scorpion => self.scorpion.frame[r][i],
            CageAnim::Penguin => self.penguin.frame[i],
            CageAnim::Seahorse => self.seahorse.frame[i],
            CageAnim::Pufferfish => self.pufferfish.frame[i],
            CageAnim::Snail => self.snail.frame[i],
            CageAnim::Snail2 => self.snail2.frame[i],
            CageAnim::Frog => self.frog.frame[i],
            CageAnim::Grasshopper => self.grasshopper.frame[i],
            CageAnim::Mouse => self.mouse.frame[i],
            CageAnim::Worm => self.worm.frame[i],
            CageAnim::Slug => self.slug.frame[r][i],
            CageAnim::Maggot => self.maggot.frame[i],
            CageAnim::Rat => self.rat.frame[i],
            CageAnim::Ladybug => self.ladybug.frame[i],
            CageAnim::WaterStrider => self.water_strider.frame[i],
            CageAnim::FishBowl => self.fish_bowl.frame[i],
            CageAnim::LavaFishBowl => self.lava_fish_bowl.frame[i],
            CageAnim::FairyJar => self.fairy_jar.frame[i],
            CageAnim::Butterfly => self.butterfly.frame[r][i],
            CageAnim::Dragonfly => self.dragonfly.frame[r][i],
            CageAnim::Jellyfish => self.jellyfish.frame[r][i],
        }
    }

    /// G6 key input (the `TileAnimator::anim_key` convention): FNV-1a over
    /// every FRAME table's raw `i32` little-endian bytes, in struct order.
    /// The counters and modes are deliberately excluded — they are animator
    /// internals the draw never reads directly (`frame_of` is the only draw
    /// input; the owl frame IS a pure counter function and rides via its
    /// frame table). Deterministic, order-sensitive, no RNG.
    pub fn anim_key(&self) -> u64 {
        let mut h = 0xcbf2_9ce4_8422_2325u64;
        let mut feed = |v: i32| {
            for b in v.to_le_bytes() {
                h ^= b as u64;
                h = h.wrapping_mul(0x0000_0100_0000_01b3);
            }
        };
        for t in [
            &self.bunny,
            &self.squirrel,
            &self.squirrel_orange,
            &self.mallard,
            &self.duck,
            &self.grebe,
            &self.seagull,
            &self.bird,
            &self.red_bird,
            &self.blue_bird,
            &self.macaw,
            &self.snail,
            &self.snail2,
            &self.fish_bowl,
            &self.lava_fish_bowl,
            &self.frog,
            &self.mouse,
            &self.turtle,
            &self.fairy_jar,
            &self.worm,
            &self.maggot,
            &self.rat,
            &self.ladybug,
            &self.penguin,
            &self.water_strider,
            &self.seahorse,
            &self.owl,
            &self.grasshopper,
            &self.pufferfish,
        ] {
            for &v in &t.frame {
                feed(v);
            }
        }
        for row in &self.butterfly.frame {
            for &v in row {
                feed(v);
            }
        }
        for row in &self.dragonfly.frame {
            for &v in row {
                feed(v);
            }
        }
        for row in &self.scorpion.frame {
            for &v in row {
                feed(v);
            }
        }
        for row in &self.jellyfish.frame {
            for &v in row {
                feed(v);
            }
        }
        for row in &self.slug.frame {
            for &v in row {
                feed(v);
            }
        }
        h
    }

    /// One client update — `Main.AnimateTiles_CritterCages()`
    /// (`Main.cs:13511-16596`), in source order. The `critterCage` gate
    /// (`:13513-13516`) returns before ANY draw; the per-animal loops then
    /// run in the C# order (the orange squirrel loop rides between the
    /// squirrel and the mallard, `:13752`).
    pub fn animate_critter_cages(&mut self, rand: &mut UnifiedRandom) {
        if !self.critter_cage {
            return; // Main.cs:13513-13516
        }
        // Each shared-shape loop clones its table ONCE, walks all 25 slots
        // (only the loop's own table mutates inside it — same reads as the
        // C# per-slot field accesses), then writes back once. The slot
        // bodies' early `return`s are the C# `continue`s: the mutated
        // counter stays in the clone.
        let mut t = self.bunny.clone();
        for i in 0..N {
            Self::step_bunny(&mut t, i, rand);
        }
        self.bunny = t;
        let mut t = self.squirrel.clone();
        for i in 0..N {
            Self::step_squirrel(&mut t, i, rand);
        }
        self.squirrel = t;
        let mut t = self.squirrel_orange.clone();
        for i in 0..N {
            Self::step_squirrel(&mut t, i, rand);
        }
        self.squirrel_orange = t;
        let mut t = self.mallard.clone();
        for i in 0..N {
            Self::step_duck(&mut t, i, rand);
        }
        self.mallard = t;
        let mut t = self.duck.clone();
        for i in 0..N {
            Self::step_duck(&mut t, i, rand);
        }
        self.duck = t;
        for i in 0..N {
            self.step_grebe(i, rand);
        }
        for i in 0..N {
            self.step_seagull(i, rand);
        }
        let mut t = self.bird.clone();
        for i in 0..N {
            Self::step_bird(&mut t, i, rand);
        }
        self.bird = t;
        let mut t = self.blue_bird.clone();
        for i in 0..N {
            Self::step_bird(&mut t, i, rand);
        }
        self.blue_bird = t;
        let mut t = self.red_bird.clone();
        for i in 0..N {
            Self::step_bird(&mut t, i, rand);
        }
        self.red_bird = t;
        for i in 0..N {
            self.step_macaw(i, rand);
        }
        for r in 0..2 {
            for i in 0..N {
                self.step_scorpion(r, i, rand);
            }
        }
        for i in 0..N {
            self.step_penguin(i, rand);
        }
        for i in 0..N {
            self.step_turtle(i, rand);
        }
        for i in 0..N {
            self.step_seahorse(i, rand);
        }
        for i in 0..N {
            self.step_pufferfish(i, rand);
        }
        self.update_owl_cage_frames(rand); // Main.cs:15476
        let mut t = self.snail.clone();
        for i in 0..N {
            Self::step_snail(&mut t, i, 45, 3600, 60, 7200, rand);
        }
        self.snail = t;
        let mut t = self.snail2.clone();
        for i in 0..N {
            Self::step_snail(&mut t, i, 30, 2700, 45, 5400, rand);
        }
        self.snail2 = t;
        for i in 0..N {
            self.step_frog(i, rand);
        }
        for i in 0..N {
            self.step_mouse(i, rand);
        }
        for i in 0..N {
            self.step_worm(i, rand);
        }
        for r in 0..3 {
            let wrap = [24, 31, 34][r]; // Main.cs:15859-15873
            for i in 0..N {
                self.step_slug(r, i, wrap, rand);
            }
        }
        for i in 0..N {
            self.step_grasshopper(i, rand);
        }
        for i in 0..N {
            self.step_maggot(i, rand);
        }
        for i in 0..N {
            self.step_ladybug(i, rand);
        }
        for i in 0..N {
            self.step_rat(i, rand);
        }
        for i in 0..N {
            self.step_water_strider(i, rand);
        }
        self.step_fairy_jar(rand); // Main.cs:16197-16227
        for i in 0..N {
            self.step_fish_bowl(i, rand);
        }
        for i in 0..N {
            self.step_lava_fish_bowl(i, rand);
        }
        for r in 0..9 {
            for i in 0..N {
                self.step_butterfly(r, i, rand);
            }
        }
        self.update_dragonfly_jar_frames(rand); // Main.cs:16526
        for r in 0..3 {
            for i in 0..N {
                self.step_jellyfish(r, i, rand);
            }
        }
    }

    // ---- per-animal slot bodies (one C# loop body each) ----
    // The shared-shape helpers take the table by value-cloned `&mut` — the
    // clone keeps the borrow checker out of the `self` + `rand` split; the
    // mutated clone is written back before the next slot runs.

    /// The bunny loop body (`Main.cs:13517-13617`) — a free fn over the
    /// caller's table clone; the write-back is the loop's tail.
    fn step_bunny(t: &mut CageTable, i: usize, rand: &mut UnifiedRandom) {
        let f = t.frame[i];
        if f == 0 {
            t.counter[i] += 1;
            if t.counter[i] <= rand.next_range(30, 900) {
                return;
            }
            if rand.next_max(3) != 0 {
                let num = rand.next_max(7);
                t.frame[i] = if num == 0 {
                    4
                } else if num <= 2 {
                    2
                } else {
                    1
                };
            }
            t.counter[i] = 0;
        } else if f == 1 {
            t.counter[i] += 1;
            if t.counter[i] >= 10 {
                t.counter[i] = 0;
                t.frame[i] = 0;
            }
        } else if (2..=3).contains(&f) {
            t.counter[i] += 1;
            if t.counter[i] >= 10 {
                t.counter[i] = 0;
                t.frame[i] += 1;
            }
            if t.frame[i] > 3 {
                t.frame[i] = 0;
            }
        } else if (4..=10).contains(&f) {
            t.counter[i] += 1;
            if t.counter[i] >= 5 {
                t.counter[i] = 0;
                t.frame[i] += 1;
            }
        } else if f == 11 {
            t.counter[i] += 1;
            if t.counter[i] <= rand.next_range(30, 900) {
                return;
            }
            if rand.next_max(3) != 0 {
                t.frame[i] = if rand.next_max(7) == 0 { 13 } else { 12 };
            }
            t.counter[i] = 0;
        } else if f == 12 {
            t.counter[i] += 1;
            if t.counter[i] >= 10 {
                t.counter[i] = 0;
                t.frame[i] = 11;
            }
        } else if f >= 13 {
            t.counter[i] += 1;
            if t.counter[i] >= 5 {
                t.counter[i] = 0;
                t.frame[i] += 1;
            }
            if t.frame[i] > 21 {
                t.frame[i] = 0;
            }
        }
    }

    /// The squirrel loop body (`Main.cs:13618-13751`); the orange table
    /// rides the SAME body (`:13752-13885`, `squirrelCageFrameOrange`) — a
    /// free fn over the caller's table clone (the grey/orange split stays
    /// in `animate_critter_cages`). The 2..3 wrap rolls `Next(5)` (0 → idle
    /// 0, else loop 2) and the 11..12 arm counts `< 5` with the frame bump
    /// BEFORE the wrap check — both verbatim.
    fn step_squirrel(t: &mut CageTable, i: usize, rand: &mut UnifiedRandom) {
        let f = t.frame[i];
        if f == 0 {
            t.counter[i] += 1;
            if t.counter[i] <= rand.next_range(30, 900) {
                return;
            }
            if rand.next_max(3) != 0 {
                let num = rand.next_max(7);
                t.frame[i] = if num == 0 {
                    4
                } else if num <= 2 {
                    2
                } else {
                    1
                };
            }
            t.counter[i] = 0;
        } else if f == 1 {
            t.counter[i] += 1;
            if t.counter[i] >= 10 {
                t.counter[i] = 0;
                t.frame[i] = 0;
            }
        } else if (2..=3).contains(&f) {
            t.counter[i] += 1;
            if t.counter[i] >= 5 {
                t.counter[i] = 0;
                t.frame[i] += 1;
            }
            if t.frame[i] > 3 {
                t.frame[i] = if rand.next_max(5) == 0 { 0 } else { 2 };
            }
        } else if (4..=8).contains(&f) {
            t.counter[i] += 1;
            if t.counter[i] >= 5 {
                t.counter[i] = 0;
                t.frame[i] += 1;
            }
        } else if f == 9 {
            t.counter[i] += 1;
            if t.counter[i] <= rand.next_range(30, 900) {
                return;
            }
            if rand.next_max(3) != 0 {
                let num = rand.next_max(7);
                t.frame[i] = if num == 0 {
                    13
                } else if num <= 2 {
                    11
                } else {
                    10
                };
            }
            t.counter[i] = 0;
        } else if f == 10 {
            t.counter[i] += 1;
            if t.counter[i] >= 10 {
                t.counter[i] = 0;
                t.frame[i] = 9;
            }
        } else if f == 11 || f == 12 {
            t.counter[i] += 1;
            if t.counter[i] < 5 {
                return;
            }
            t.frame[i] += 1;
            if t.frame[i] > 12 {
                t.frame[i] = if rand.next_max(5) != 0 { 11 } else { 9 };
            }
            t.counter[i] = 0;
        } else if f >= 13 {
            t.counter[i] += 1;
            if t.counter[i] >= 5 {
                t.counter[i] = 0;
                t.frame[i] += 1;
            }
            if t.frame[i] > 17 {
                t.frame[i] = 0;
            }
        }
    }

    /// The mallard loop body (`Main.cs:13886-14013`); the duck rides the
    /// SAME body (`:14014-14141`). The idle arms' `(f == 0 && Next(3) != 0)
    /// || (f == 4 && Next(5) == 0)` short-circuit is load-bearing for the
    /// stream: at f 4 the `Next(3)` is NOT drawn; at f 0 with `Next(3) == 0`
    /// the `Next(5)` is NOT drawn (`:13895`/`:13955`).
    fn step_duck(t: &mut CageTable, i: usize, rand: &mut UnifiedRandom) {
        let f = t.frame[i];
        if f == 0 || f == 4 {
            t.counter[i] += 1;
            if t.counter[i] <= rand.next_range(45, 2700) {
                return;
            }
            if (f == 0 && rand.next_max(3) != 0) || (f == 4 && rand.next_max(5) == 0) {
                if rand.next_max(6) == 0 {
                    t.frame[i] = 5;
                } else if rand.next_max(3) == 0 {
                    t.frame[i] = if f == 4 { 0 } else { 4 };
                } else {
                    t.frame[i] = 1;
                }
            }
            t.counter[i] = 0;
        } else if (1..=3).contains(&f) {
            t.counter[i] += 1;
            if t.counter[i] >= 5 {
                t.counter[i] = 0;
                t.frame[i] += 1;
            }
            if t.frame[i] > 3 {
                t.frame[i] = if rand.next_max(5) == 0 { 0 } else { 1 };
            }
        } else if (5..=11).contains(&f) {
            t.counter[i] += 1;
            if t.counter[i] >= 5 {
                t.counter[i] = 0;
                t.frame[i] += 1;
            }
        } else if f == 12 || f == 16 {
            t.counter[i] += 1;
            if t.counter[i] <= rand.next_range(45, 2700) {
                return;
            }
            if (f == 12 && rand.next_max(3) != 0) || (f == 16 && rand.next_max(5) == 0) {
                if rand.next_max(6) == 0 {
                    t.frame[i] = 17;
                } else if rand.next_max(3) == 0 {
                    t.frame[i] = if f == 16 { 12 } else { 16 };
                } else {
                    t.frame[i] = 13;
                }
            }
            t.counter[i] = 0;
        } else if (13..=15).contains(&f) {
            t.counter[i] += 1;
            if t.counter[i] < 5 {
                return;
            }
            t.frame[i] += 1;
            if t.frame[i] > 15 {
                t.frame[i] = if rand.next_max(5) != 0 { 12 } else { 13 };
            }
            t.counter[i] = 0;
        } else if f >= 17 {
            t.counter[i] += 1;
            if t.counter[i] >= 5 {
                t.counter[i] = 0;
                t.frame[i] += 1;
            }
            if t.frame[i] > 23 {
                t.frame[i] = 0;
            }
        }
    }

    /// The grebe loop (`Main.cs:14142-14211`): the counter bumps FIRST,
    /// the idle thresholds roll per-slot (0/7 → `Next(300, 400)`, 16/20 →
    /// `Next(480, 600)`, else the static 5), then the frame arms.
    fn step_grebe(&mut self, i: usize, rand: &mut UnifiedRandom) {
        let t = &mut self.grebe;
        t.counter[i] += 1;
        let f = t.frame[i];
        let idle = f == 0 || f == 7;
        let perch = f == 16 || f == 20;
        let mut limit = 5;
        if idle {
            limit = rand.next_range(300, 400);
        }
        if perch {
            limit = rand.next_range(480, 600);
        }
        if t.counter[i] < limit {
            return;
        }
        t.counter[i] = 0;
        if (25..=27).contains(&f) {
            t.frame[i] += 1;
            if t.frame[i] > 27 {
                t.frame[i] = if rand.next_max(5) == 0 { 7 } else { 25 };
            }
        } else if (22..=24).contains(&f) {
            t.frame[i] += 1;
            if t.frame[i] > 24 {
                t.frame[i] = if rand.next_max(5) == 0 { 0 } else { 22 };
            }
        } else if f == 0 && rand.next_max(3) == 0 {
            t.frame[i] = if rand.next_max(2) == 0 { 22 } else { 14 };
        } else if f == 7 && rand.next_max(3) == 0 {
            t.frame[i] = if rand.next_max(2) == 0 { 25 } else { 18 };
        } else if f == 13 || f == 17 {
            t.frame[i] = 0;
        } else if f == 21 {
            t.frame[i] = 7;
        } else {
            t.frame[i] += 1;
        }
    }

    /// The seagull loop (`Main.cs:14212-14371`): the counter-first shape
    /// with three threshold classes (0/14 → `Next(180, 250)`, 15..18 → 66,
    /// 9..13 → 78, else 6) and the two sub-counter ladders that re-derive
    /// the flap frame from `counter % 66` / `% 78` while below threshold.
    fn step_seagull(&mut self, i: usize, rand: &mut UnifiedRandom) {
        let t = &mut self.seagull;
        t.counter[i] += 1;
        let f = t.frame[i];
        let idle = f == 0 || f == 14;
        let preen = (15..=18).contains(&f);
        let flap = (9..=13).contains(&f);
        let mut limit = 6;
        if idle {
            limit = rand.next_range(180, 250);
        }
        if preen {
            limit = 66;
        }
        if flap {
            limit = 78;
        }
        if t.counter[i] >= limit {
            t.counter[i] = 0;
            if f == 0 && rand.next_max(3) == 0 {
                t.frame[i] = 9;
                return;
            }
            if f == 14 {
                t.frame[i] = if rand.next_max(2) == 0 { 15 } else { 19 };
                return;
            }
            if f == 9 {
                t.frame[i] = 0;
                return;
            }
            if f == 8 || f == 15 {
                t.frame[i] = 14;
                return;
            }
            t.frame[i] += 1;
            if t.frame[i] > 26 {
                t.frame[i] = 0;
            }
        } else if preen {
            // Main.cs:14269-14313 — the % 66 preen ladder.
            let n = t.counter[i] % 66;
            t.frame[i] = if n > 60 {
                15
            } else if n > 54 {
                16
            } else if n > 48 {
                17
            } else if n > 42 {
                18
            } else if n > 36 {
                17
            } else if n > 30 {
                16
            } else if n > 24 {
                17
            } else if n > 18 {
                18
            } else if n > 12 {
                17
            } else if n > 6 {
                16
            } else {
                15
            };
        } else if flap {
            // Main.cs:14317-14369 — the % 78 flap ladder.
            let n = t.counter[i] % 78;
            t.frame[i] = if n > 72 {
                9
            } else if n > 66 {
                10
            } else if n > 60 {
                11
            } else if n > 54 {
                12
            } else if n > 48 {
                13
            } else if n > 42 {
                12
            } else if n > 36 {
                11
            } else if n > 30 {
                12
            } else if n > 24 {
                13
            } else if n > 18 {
                12
            } else if n > 12 {
                11
            } else if n > 6 {
                10
            } else {
                9
            };
        }
    }

    /// The bird loop body (`Main.cs:14372-14507`); blue bird (`:14508-14643`)
    /// and red bird (`:14644-14779`) ride the SAME body. The frame-1 arm's
    /// `counter > Next(900, 18000) && Next(3) == 0` short-circuit keeps the
    /// second roll gated; the 2..5 arm rolls `Next(3)` only at frame 3.
    fn step_bird(t: &mut CageTable, i: usize, rand: &mut UnifiedRandom) {
        let f = t.frame[i];
        if f == 0 {
            t.counter[i] += 1;
            if t.counter[i] <= rand.next_range(30, 2700) {
                return;
            }
            if rand.next_max(3) != 0 {
                t.frame[i] = if rand.next_max(3) != 0 { 2 } else { 1 };
            }
            t.counter[i] = 0;
        } else if f == 1 {
            t.counter[i] += 1;
            if t.counter[i] > rand.next_range(900, 18000) && rand.next_max(3) == 0 {
                t.counter[i] = 0;
                t.frame[i] = 0;
            }
        } else if (2..=5).contains(&f) {
            t.counter[i] += 1;
            if t.counter[i] >= 5 {
                t.counter[i] = 0;
                if f == 3 && rand.next_max(3) == 0 {
                    t.frame[i] = 13;
                } else {
                    t.frame[i] += 1;
                }
            }
        } else if f == 6 {
            t.counter[i] += 1;
            if t.counter[i] <= rand.next_range(45, 2700) {
                return;
            }
            if rand.next_max(3) != 0 {
                if rand.next_max(6) == 0 {
                    t.frame[i] = 7;
                } else if rand.next_max(6) == 0 {
                    t.frame[i] = 11;
                }
            }
            t.counter[i] = 0;
        } else if (7..=10).contains(&f) {
            t.counter[i] += 1;
            if t.counter[i] >= 5 {
                t.frame[i] += 1;
                if t.frame[i] > 10 {
                    t.frame[i] = 0;
                }
                t.counter[i] = 0;
            }
        } else if (11..=13).contains(&f) {
            t.counter[i] += 1;
            if t.counter[i] >= 5 {
                t.frame[i] += 1;
                t.counter[i] = 0;
            }
        } else if f == 14 {
            t.counter[i] += 1;
            if t.counter[i] > rand.next_range(5, 600) {
                if rand.next_max(20) == 0 {
                    t.frame[i] = 16;
                } else if rand.next_max(20) == 0 {
                    t.frame[i] = 4;
                } else {
                    t.frame[i] = 15;
                }
                t.counter[i] = 0;
            }
        } else if f == 15 {
            t.counter[i] += 1;
            if t.counter[i] >= 10 {
                t.counter[i] = 0;
                t.frame[i] = 14;
            }
        } else {
            if !(16..=18).contains(&f) {
                return;
            }
            t.counter[i] += 1;
            if t.counter[i] >= 5 {
                t.frame[i] += 1;
                if t.frame[i] > 18 {
                    t.frame[i] = 0;
                }
                t.counter[i] = 0;
            }
        }
    }

    /// The macaw loop (`Main.cs:14780-14878`): the `> 5` / `> 6` counter
    /// idioms (strict) and the two wrap rolls (`Next(3)` at 19/10).
    fn step_macaw(&mut self, i: usize, rand: &mut UnifiedRandom) {
        let t = &mut self.macaw;
        let f = t.frame[i];
        if f == 0 {
            t.counter[i] += 1;
            if t.counter[i] <= rand.next_range(300, 2700) {
                return;
            }
            if rand.next_max(3) != 0 {
                t.frame[i] = if rand.next_max(3) != 0 { 16 } else { 1 };
            }
            t.counter[i] = 0;
        } else if (1..=5).contains(&f) {
            t.counter[i] += 1;
            if t.counter[i] > 5 {
                t.counter[i] = 0;
                t.frame[i] += 1;
                if t.frame[i] > 5 {
                    t.frame[i] = 6;
                }
            }
        } else if (16..=19).contains(&f) {
            t.counter[i] += 1;
            if t.counter[i] > 6 {
                t.counter[i] = 0;
                t.frame[i] += 1;
                if t.frame[i] > 19 {
                    t.frame[i] = if rand.next_max(3) != 0 { 16 } else { 0 };
                }
            }
        } else if f == 6 {
            t.counter[i] += 1;
            if t.counter[i] <= rand.next_range(300, 2700) {
                return;
            }
            if rand.next_max(3) != 0 {
                t.frame[i] = if rand.next_max(3) != 0 { 7 } else { 11 };
            }
            t.counter[i] = 0;
        } else if (11..=15).contains(&f) {
            t.counter[i] += 1;
            if t.counter[i] > 5 {
                t.counter[i] = 0;
                t.frame[i] += 1;
                if t.frame[i] > 15 {
                    t.frame[i] = 0;
                }
            }
        } else {
            if !(7..=10).contains(&f) {
                return;
            }
            t.counter[i] += 1;
            if t.counter[i] > 6 {
                t.counter[i] = 0;
                t.frame[i] += 1;
                if t.frame[i] > 10 {
                    t.frame[i] = if rand.next_max(3) == 0 { 6 } else { 7 };
                }
            }
        }
    }

    /// The scorpion loop (`Main.cs:14879-15038`), row-major over the 2-row
    /// table. The idle arm's nested `Next(7)/Next(4)/Next(3)` cascade and
    /// the 14..15 / 16..17 arms' unconditional `Next(5)` (drawn INSIDE the
    /// `>= 5` / `>= 5` counter gate, after the wrap) are verbatim.
    fn step_scorpion(&mut self, r: usize, i: usize, rand: &mut UnifiedRandom) {
        let t = &mut self.scorpion;
        let f = t.frame[r][i];
        if f == 0 || f == 7 {
            t.counter[r][i] += 1;
            if t.counter[r][i] <= rand.next_range(30, 3600) {
                return;
            }
            if f == 7 {
                t.frame[r][i] = 0;
            } else if rand.next_max(3) == 0 {
                if rand.next_max(7) == 0 {
                    t.frame[r][i] = 1;
                } else if rand.next_max(4) == 0 {
                    t.frame[r][i] = 8;
                } else if rand.next_max(3) == 0 {
                    t.frame[r][i] = 7;
                } else {
                    t.frame[r][i] = 14;
                }
            }
            t.counter[r][i] = 0;
        } else if (1..=2).contains(&f) {
            t.counter[r][i] += 1;
            if t.counter[r][i] >= 10 {
                t.counter[r][i] = 0;
                t.frame[r][i] += 1;
            }
        } else if (8..=10).contains(&f) {
            t.counter[r][i] += 1;
            if t.counter[r][i] >= 10 {
                t.counter[r][i] = 0;
                t.frame[r][i] += 1;
            }
        } else if f == 11 {
            t.counter[r][i] += 1;
            if t.counter[r][i] > rand.next_range(45, 5400) {
                if rand.next_max(6) == 0 {
                    t.frame[r][i] = 12;
                }
                t.counter[r][i] = 0;
            }
        } else if (12..=13).contains(&f) {
            t.counter[r][i] += 1;
            if t.counter[r][i] >= 10 {
                t.counter[r][i] = 0;
                t.frame[r][i] += 1;
                if t.frame[r][i] > 13 {
                    t.frame[r][i] = 0;
                }
            }
        } else if (14..=15).contains(&f) {
            t.counter[r][i] += 1;
            if t.counter[r][i] >= 5 {
                t.counter[r][i] = 0;
                t.frame[r][i] += 1;
                if t.frame[r][i] > 15 {
                    t.frame[r][i] = 14;
                }
                if rand.next_max(5) == 0 {
                    t.frame[r][i] = 0;
                }
            }
        } else if f == 4 || f == 3 {
            t.counter[r][i] += 1;
            if t.counter[r][i] <= rand.next_range(30, 3600) {
                return;
            }
            if f == 3 {
                t.frame[r][i] = 4;
            } else if rand.next_max(3) == 0 {
                if rand.next_max(5) == 0 {
                    t.frame[r][i] = 5;
                } else if rand.next_max(3) == 0 {
                    t.frame[r][i] = 3;
                } else {
                    t.frame[r][i] = 16;
                }
            }
            t.counter[r][i] = 0;
        } else if (5..=6).contains(&f) {
            t.counter[r][i] += 1;
            if t.counter[r][i] >= 10 {
                t.counter[r][i] = 0;
                t.frame[r][i] += 1;
                if t.frame[r][i] > 7 {
                    t.frame[r][i] = 0;
                }
            }
        } else {
            if !(16..=17).contains(&f) {
                return;
            }
            t.counter[r][i] += 1;
            if t.counter[r][i] >= 5 {
                t.counter[r][i] = 0;
                t.frame[r][i] += 1;
                if t.frame[r][i] > 17 {
                    t.frame[r][i] = 16;
                }
                if rand.next_max(5) == 0 {
                    t.frame[r][i] = 4;
                }
            }
        }
    }

    /// The penguin loop (`Main.cs:15039-15203`). The 10..11 arm's
    /// `> 3` wrap check is verbatim dead code (frames 10-11 can never
    /// exceed 3 in one bump) — kept, with its `Next(3)` roll inside.
    fn step_penguin(&mut self, i: usize, rand: &mut UnifiedRandom) {
        let t = &mut self.penguin;
        let f = t.frame[i];
        if f == 0 {
            t.counter[i] += 1;
            if t.counter[i] <= rand.next_range(30, 1800) {
                return;
            }
            if rand.next_max(2) == 0 {
                if rand.next_max(10) == 0 {
                    t.frame[i] = 4;
                } else if rand.next_max(7) == 0 {
                    t.frame[i] = 15;
                } else if rand.next_max(3) == 0 {
                    t.frame[i] = 2;
                } else {
                    t.frame[i] = 1;
                }
            }
            t.counter[i] = 0;
        } else if f == 1 {
            t.counter[i] += 1;
            if t.counter[i] >= 10 {
                t.counter[i] = 0;
                t.frame[i] = 0;
            }
        } else if (2..=3).contains(&f) {
            t.counter[i] += 1;
            if t.counter[i] < 5 {
                return;
            }
            t.counter[i] = 0;
            t.frame[i] += 1;
            if t.frame[i] > 3 {
                t.frame[i] = if rand.next_max(3) == 0 { 0 } else { 2 };
            }
        } else if (4..=6).contains(&f) {
            t.counter[i] += 1;
            if t.counter[i] >= 10 {
                t.counter[i] = 0;
                t.frame[i] += 1;
            }
        } else if f == 15 {
            t.counter[i] += 1;
            if t.counter[i] > rand.next_range(10, 1800) {
                if rand.next_max(2) == 0 {
                    t.frame[i] = 0;
                }
                t.counter[i] = 0;
            }
        } else if f == 8 {
            t.counter[i] += 1;
            if t.counter[i] <= rand.next_range(30, 3600) {
                return;
            }
            if rand.next_max(2) == 0 {
                if rand.next_max(10) == 0 {
                    t.frame[i] = 12;
                } else if rand.next_max(7) == 0 {
                    t.frame[i] = 7;
                } else if rand.next_max(3) == 0 {
                    t.frame[i] = 10;
                } else {
                    t.frame[i] = 9;
                }
            }
            t.counter[i] = 0;
        } else if f == 9 {
            t.counter[i] += 1;
            if t.counter[i] >= 10 {
                t.counter[i] = 0;
                t.frame[i] = 8;
            }
        } else if (10..=11).contains(&f) {
            t.counter[i] += 1;
            if t.counter[i] < 5 {
                return;
            }
            t.counter[i] = 0;
            t.frame[i] += 1;
            if t.frame[i] > 3 {
                // Main.cs:15166 — verbatim (> 3 never fires from 10/11).
                t.frame[i] = if rand.next_max(3) == 0 { 8 } else { 10 };
            }
        } else if (12..=14).contains(&f) {
            t.counter[i] += 1;
            if t.counter[i] >= 10 {
                t.counter[i] = 0;
                t.frame[i] += 1;
            }
        } else {
            if f != 7 {
                return;
            }
            t.counter[i] += 1;
            if t.counter[i] > rand.next_range(10, 3600) {
                if rand.next_max(2) == 0 {
                    t.frame[i] = 8;
                }
                t.counter[i] = 0;
            }
        }
    }

    /// The turtle loop (`Main.cs:15204-15233`): counter-first, the idle
    /// frames 0/15/31 roll `Next(180, 250)`, else the static 8.
    fn step_turtle(&mut self, i: usize, rand: &mut UnifiedRandom) {
        let t = &mut self.turtle;
        t.counter[i] += 1;
        let f = t.frame[i];
        let idle = f == 0 || f == 15 || f == 31;
        let mut limit = 8;
        if idle {
            limit = rand.next_range(180, 250);
        }
        if t.counter[i] < limit {
            return;
        }
        t.counter[i] = 0;
        if f == 29 {
            t.frame[i] = 0;
            return;
        }
        if f == 15 && rand.next_max(3) == 0 {
            t.frame[i] = 30;
            return;
        }
        t.frame[i] += 1;
        if t.frame[i] > 32 {
            t.frame[i] = 15;
        }
    }

    /// The seahorse loop (`Main.cs:15234-15273`): counter-first with three
    /// threshold classes (0/13 → `Next(220, 340)`, 4/9/17/22 → 10, ≥25 → 6,
    /// else 7).
    fn step_seahorse(&mut self, i: usize, rand: &mut UnifiedRandom) {
        let t = &mut self.seahorse;
        t.counter[i] += 1;
        let f = t.frame[i];
        let idle = f == 0 || f == 13;
        let sway = f == 4 || f == 9 || f == 17 || f == 22;
        let bubble = f >= 25;
        let mut limit = 7;
        if idle {
            limit = rand.next_range(220, 340);
        }
        if sway {
            limit = 10;
        }
        if bubble {
            limit = 6;
        }
        if t.counter[i] < limit {
            return;
        }
        t.counter[i] = 0;
        if f == 0 && rand.next_max(2) == 0 {
            t.frame[i] = 25;
            return;
        }
        if f == 24 {
            t.frame[i] = 0;
            return;
        }
        t.frame[i] += 1;
        if t.frame[i] > 33 {
            t.frame[i] = 0;
        }
    }

    /// The pufferfish loop (`Main.cs:15274-15475`): frames 0..40 swim with
    /// a `Next(75)` puff roll per step; frames ≥41 run the inflate/deflate
    /// counter machine (`num24 = 7`, `num25 = 183`, `num26 = 175`).
    fn step_pufferfish(&mut self, i: usize, rand: &mut UnifiedRandom) {
        let t = &mut self.pufferfish;
        let num22 = 40; // Main.cs:15274
        let f = t.frame[i];
        if (0..=num22).contains(&f) {
            t.counter[i] += 1;
            if t.counter[i] < 7 {
                return;
            }
            t.counter[i] = 0;
            t.frame[i] += 1;
            if t.frame[i] > num22 {
                t.frame[i] = 0;
            }
            if rand.next_max(75) == 0 {
                // Main.cs:15292-15325 — the puff start-frame switch.
                t.frame[i] = match t.frame[i] {
                    0..=2 => 41,
                    8..=10 => 46,
                    17..=19 => 51,
                    20..=22 => 56,
                    30..=32 => 61,
                    37..=40 => 66,
                    v => v,
                };
            }
            return;
        }
        let num24 = 7; // Main.cs:15329
        let num25 = 5 * num24 * 5 + 8; // 183 (:15330)
        let num26 = num25 - 8; // 175 (:15331)
        t.counter[i] += 1;
        if t.counter[i] < num26 {
            let mut num27 = t.counter[i] / num24;
            if num27 > 3 {
                num27 = 3;
            }
            // Main.cs:15340-15380 — the inflate ladder.
            t.frame[i] = match t.frame[i] {
                46..=50 => 46 + num27,
                51..=55 => 51 + num27,
                56..=60 => 56 + num27,
                61..=65 => 61 + num27,
                66..=70 => 66 + num27,
                _ => 41 + num27,
            };
        } else {
            // Main.cs:15384-15424 — the fully-puffed hold frame.
            t.frame[i] = match t.frame[i] {
                46..=50 => 50,
                51..=55 => 55,
                56..=60 => 60,
                61..=65 => 65,
                66..=70 => 70,
                _ => 45,
            };
        }
        if t.counter[i] >= num25 {
            t.counter[i] = 0;
            // Main.cs:15429-15473 — the deflate exit back to the swim row.
            t.frame[i] = match t.frame[i] {
                41..=45 => 1,
                46..=50 => 9,
                51..=55 => 18,
                56..=60 => 21,
                61..=65 => 31,
                66..=70 => 38,
                v => v,
            };
        }
    }

    /// `Main.UpdateOwlCageFrames` (`Main.cs:16598-16604`) — the 25-slot
    /// loop over `UpdateOwlCageFrame`.
    fn update_owl_cage_frames(&mut self, rand: &mut UnifiedRandom) {
        for i in 0..N {
            self.update_owl_cage_frame(i, rand);
        }
    }

    /// `Main.UpdateOwlCageFrame` (`Main.cs:16617-16666`): the counter is a
    /// pure state machine (`% 40` phases × 4 blocks) — the frame is a
    /// function of the counter, no separate frame memory in the C# logic.
    fn update_owl_cage_frame(&mut self, i: usize, rand: &mut UnifiedRandom) {
        let t = &mut self.owl;
        if t.counter[i] % 40 != 0 || rand.next_max(80) == 0 {
            t.counter[i] += 1;
        }
        if (t.counter[i] + 1) % 40 == 39 {
            t.counter[i] = 40 * rand.next_max(4);
        }
        let mut num = t.counter[i] % 40 / 10;
        let num2 = t.counter[i] / 40;
        let mut num3 = 0;
        match num2 {
            0 => {
                if num == 3 {
                    num = 1;
                }
                num3 = num;
            }
            1 => {
                if num == 3 {
                    num = 1;
                }
                num3 = 0;
                if num != 0 {
                    num3 = 8 - num;
                }
            }
            2 => {
                num3 = 0;
                if num != 0 {
                    num3 = 7 + num;
                }
            }
            3 => {
                num3 = 0;
                if num != 0 {
                    num3 = 11 + num;
                }
            }
            _ => {}
        }
        t.frame[i] = num3;
    }

    /// The snail loop body (`Main.cs:15477-15549`); snail2 rides the SAME
    /// body with its own constants (`:15550-15622`): idle `Next(lo1, hi1)`
    /// = (45, 3600) / (30, 2700), hide `Next(lo2, hi2)` = (60, 7200) /
    /// (45, 5400). The DOUBLE `f == 1 && Next(3) == 0` arm is verbatim —
    /// the second roll only draws when the first missed.
    fn step_snail(
        t: &mut CageTable,
        i: usize,
        lo1: i32,
        hi1: i32,
        lo2: i32,
        hi2: i32,
        rand: &mut UnifiedRandom,
    ) {
        let f = t.frame[i];
        if (0..=13).contains(&f) {
            t.counter[i] += 1;
            if t.counter[i] <= rand.next_range(lo1, hi1) {
                return;
            }
            if f == 8 && rand.next_max(2) == 0 {
                t.frame[i] = 14;
            } else if f == 1 && rand.next_max(3) == 0 {
                t.frame[i] = 19;
            } else if f == 1 && rand.next_max(3) == 0 {
                t.frame[i] = 20;
            } else {
                t.frame[i] += 1;
                if t.frame[i] > 13 {
                    t.frame[i] = 0;
                }
            }
            t.counter[i] = 0;
        } else if (14..=18).contains(&f) {
            t.counter[i] += 1;
            if t.counter[i] >= 5 {
                t.counter[i] = 0;
                t.frame[i] += 1;
            }
            if t.frame[i] > 18 {
                t.frame[i] = 20;
            }
        } else {
            if f != 19 && f != 20 {
                return;
            }
            t.counter[i] += 1;
            if t.counter[i] <= rand.next_range(lo2, hi2) {
                return;
            }
            t.counter[i] = 0;
            if rand.next_max(4) == 0 {
                if rand.next_max(3) == 0 {
                    t.frame[i] = 2;
                } else if f == 19 {
                    t.frame[i] = 20;
                } else {
                    t.frame[i] = 19;
                }
            }
        }
    }

    /// The frog loop (`Main.cs:15623-15723`). The 18..23 arm's `> 17`
    /// wrap fires EVERY 5-tick step (the frames are already past 17) —
    /// verbatim, with its `Next(3)` roll.
    fn step_frog(&mut self, i: usize, rand: &mut UnifiedRandom) {
        let t = &mut self.frog;
        let f = t.frame[i];
        if f == 0 {
            t.counter[i] += 1;
            if t.counter[i] > rand.next_range(45, 3600) {
                t.frame[i] = if rand.next_max(10) == 0 { 1 } else { 12 };
                t.counter[i] = 0;
            }
        } else if (1..=5).contains(&f) {
            t.counter[i] += 1;
            if t.counter[i] >= 5 {
                t.frame[i] += 1;
                t.counter[i] = 0;
            }
        } else if (12..=17).contains(&f) {
            t.counter[i] += 1;
            if t.counter[i] >= 5 {
                t.counter[i] = 0;
                t.frame[i] += 1;
            }
            if t.frame[i] > 17 {
                t.frame[i] = if rand.next_max(3) == 0 { 0 } else { 12 };
            }
        } else if f == 6 {
            t.counter[i] += 1;
            if t.counter[i] > rand.next_range(45, 3600) {
                t.frame[i] = if rand.next_max(10) == 0 { 7 } else { 18 };
                t.counter[i] = 0;
            }
        } else if (7..=11).contains(&f) {
            t.counter[i] += 1;
            if t.counter[i] >= 5 {
                t.frame[i] += 1;
                t.counter[i] = 0;
                if t.frame[i] > 11 {
                    t.frame[i] = 0;
                }
            }
        } else {
            if !(18..=23).contains(&f) {
                return;
            }
            t.counter[i] += 1;
            if t.counter[i] >= 5 {
                t.counter[i] = 0;
                t.frame[i] += 1;
            }
            if t.frame[i] > 17 {
                t.frame[i] = if rand.next_max(3) == 0 { 6 } else { 18 };
            }
        }
    }

    /// The mouse loop (`Main.cs:15724-15829`).
    fn step_mouse(&mut self, i: usize, rand: &mut UnifiedRandom) {
        let t = &mut self.mouse;
        let f = t.frame[i];
        if (0..=1).contains(&f) {
            t.counter[i] += 1;
            if t.counter[i] >= 5 {
                t.frame[i] += 1;
                if t.frame[i] > 1 {
                    t.frame[i] = 0;
                }
                t.counter[i] = 0;
                if rand.next_max(15) == 0 {
                    t.frame[i] = 4;
                }
            }
        } else if (4..=7).contains(&f) {
            t.counter[i] += 1;
            if t.counter[i] >= 5 {
                t.counter[i] = 0;
                t.frame[i] += 1;
            }
            if t.frame[i] > 7 {
                t.frame[i] = 2;
            }
        } else if (2..=3).contains(&f) {
            t.counter[i] += 1;
            if t.counter[i] >= 5 {
                t.frame[i] += 1;
                if t.frame[i] > 3 {
                    t.frame[i] = 2;
                }
                t.counter[i] = 0;
                if rand.next_max(15) == 0 {
                    t.frame[i] = 8;
                } else if rand.next_max(15) == 0 {
                    t.frame[i] = 12;
                }
            }
        } else if (8..=11).contains(&f) {
            t.counter[i] += 1;
            if t.counter[i] >= 5 {
                t.counter[i] = 0;
                t.frame[i] += 1;
            }
            if t.frame[i] > 11 {
                t.frame[i] = 0;
            }
        } else if (12..=13).contains(&f) {
            t.counter[i] += 1;
            if t.counter[i] >= 5 {
                t.counter[i] = 0;
                t.frame[i] += 1;
            }
        } else if (14..=17).contains(&f) {
            t.counter[i] += 1;
            if t.counter[i] >= 5 {
                t.counter[i] = 0;
                t.frame[i] += 1;
                if t.frame[i] > 17 && rand.next_max(20) != 0 {
                    t.frame[i] = 14;
                }
            }
        } else {
            if !(18..=19).contains(&f) {
                return;
            }
            t.counter[i] += 1;
            if t.counter[i] >= 5 {
                t.counter[i] = 0;
                t.frame[i] += 1;
                if t.frame[i] > 19 {
                    t.frame[i] = 0;
                }
            }
        }
    }

    /// The worm loop (`Main.cs:15830-15858`): every tick rolls
    /// `Next(30, 91)` as the threshold, then `Next(4)` gates the bump.
    fn step_worm(&mut self, i: usize, rand: &mut UnifiedRandom) {
        let t = &mut self.worm;
        t.counter[i] += 1;
        if t.counter[i] < rand.next_range(30, 91) {
            return;
        }
        t.counter[i] = 0;
        if rand.next_max(4) != 0 {
            return;
        }
        t.frame[i] += 1;
        if t.frame[i] == 9 && rand.next_max(2) == 0 {
            t.frame[i] = 0;
        }
        if t.frame[i] > 18 {
            t.frame[i] = if rand.next_max(2) == 0 { 9 } else { 0 };
        }
    }

    /// The slug loop (`Main.cs:15874-15884`): `++counter >= Next(5, 15)`
    /// (a fresh roll EVERY tick), `++frame >= wrap` (24/31/34 per row).
    fn step_slug(&mut self, r: usize, i: usize, wrap: i32, rand: &mut UnifiedRandom) {
        let t = &mut self.slug;
        t.counter[r][i] += 1;
        if t.counter[r][i] >= rand.next_range(5, 15) {
            t.counter[r][i] = 0;
            t.frame[r][i] += 1;
            if t.frame[r][i] >= wrap {
                t.frame[r][i] = 0;
            }
        }
    }

    /// The grasshopper loop (`Main.cs:15886-15948`).
    fn step_grasshopper(&mut self, i: usize, rand: &mut UnifiedRandom) {
        let t = &mut self.grasshopper;
        let f = t.frame[i];
        if (0..=1).contains(&f) {
            t.counter[i] += 1;
            if t.counter[i] >= 5 {
                t.frame[i] += 1;
                if t.frame[i] > 1 {
                    t.frame[i] = 0;
                }
                t.counter[i] = 0;
                if rand.next_max(15) == 0 {
                    t.frame[i] = 2;
                }
            }
        } else if (2..=5).contains(&f) {
            t.counter[i] += 1;
            if t.counter[i] >= 5 {
                t.counter[i] = 0;
                t.frame[i] += 1;
            }
            if t.frame[i] > 5 {
                t.frame[i] = 6;
            }
        } else if (6..=7).contains(&f) {
            t.counter[i] += 1;
            if t.counter[i] >= 5 {
                t.frame[i] += 1;
                if t.frame[i] > 7 {
                    t.frame[i] = 6;
                }
                t.counter[i] = 0;
                if rand.next_max(15) == 0 {
                    t.frame[i] = 8;
                }
            }
        } else if (8..=11).contains(&f) {
            t.counter[i] += 1;
            if t.counter[i] >= 5 {
                t.counter[i] = 0;
                t.frame[i] += 1;
            }
            if t.frame[i] > 11 {
                t.frame[i] = 0;
            }
        }
    }

    /// The maggot loop (`Main.cs:15949-15985`): counter-first, idle frames
    /// 0/6/18 roll `Next(100, 140)`, else the static 8.
    fn step_maggot(&mut self, i: usize, rand: &mut UnifiedRandom) {
        let t = &mut self.maggot;
        t.counter[i] += 1;
        let f = t.frame[i];
        let idle = f == 0 || f == 6 || f == 18;
        let mut limit = 8;
        if idle {
            limit = rand.next_range(100, 140);
        }
        if t.counter[i] < limit {
            return;
        }
        t.counter[i] = 0;
        if f == 0 {
            t.frame[i] = if rand.next_max(3) == 0 { 13 } else { 1 };
            return;
        }
        if f == 12 {
            t.frame[i] = 0;
            return;
        }
        t.frame[i] += 1;
        if t.frame[i] > 23 {
            t.frame[i] = 6;
        }
    }

    /// The ladybug loop (`Main.cs:15986-16049`): the threshold roll is
    /// UNCONDITIONAL (`Next(10, 12)` first, then overridden by the frame
    /// class) — the base roll is drawn even when a later arm replaces it.
    fn step_ladybug(&mut self, i: usize, rand: &mut UnifiedRandom) {
        let t = &mut self.ladybug;
        t.counter[i] += 1;
        let f = t.frame[i];
        let long_idle = f == 3 || f == 7 || f >= 17;
        let mut limit = rand.next_range(10, 12);
        if f == 18 {
            limit = rand.next_range(160, 241);
        } else if long_idle {
            limit = rand.next_range(198, 206);
        } else if (8..=16).contains(&f) {
            limit = 5;
        }
        if t.counter[i] < limit {
            return;
        }
        t.counter[i] = 0;
        if f < 18 {
            if (f == 2 || f == 5) && rand.next_max(3) == 0 {
                t.frame[i] = 17;
            } else if f == 3 || f == 12 {
                t.frame[i] = match rand.next_max(3) {
                    0 | 1 => 4,
                    _ => 13,
                };
            } else if f == 7 || f == 16 {
                t.frame[i] = match rand.next_max(3) {
                    0 | 1 => 0,
                    _ => 8,
                };
            } else {
                t.frame[i] += 1;
            }
        } else {
            t.frame[i] = if rand.next_max(2) == 0 { 13 } else { 4 };
        }
    }

    /// The rat loop (`Main.cs:16050-16156`): counter-first with three
    /// threshold classes (0/4 → `Next(40, 70)`, 8..11 → `Next(70, 110)`,
    /// >11 → 180, else 5), the % 90 grooming ladder and the % 10 ear-twitch.
    fn step_rat(&mut self, i: usize, rand: &mut UnifiedRandom) {
        let t = &mut self.rat;
        t.counter[i] += 1;
        let f = t.frame[i];
        let idle = f == 0 || f == 4;
        let groom = (8..=11).contains(&f);
        let sleep = f > 11;
        let mut limit = 5;
        if idle {
            limit = rand.next_range(40, 70);
        }
        if groom {
            limit = rand.next_range(70, 110);
        }
        if sleep {
            limit = 180;
        }
        if t.counter[i] >= limit {
            t.counter[i] = 0;
            if f == 0 && rand.next_max(2) == 0 {
                t.frame[i] = if rand.next_max(3) != 0 { 12 } else { 8 };
                return;
            }
            if f == 4 && rand.next_max(2) == 0 {
                t.frame[i] = 10;
                return;
            }
            if f == 7 || f == 9 || f == 12 {
                t.frame[i] = 0;
                return;
            }
            if f == 11 {
                t.frame[i] = 4;
                return;
            }
            t.frame[i] += 1;
            if t.frame[i] > 16 {
                t.frame[i] = 0;
            }
        } else if f > 11 {
            // Main.cs:16107-16143 — the % 90 sleep-breath ladder.
            let n = t.counter[i] % 90;
            t.frame[i] = if n > 80 {
                12
            } else if n > 70 {
                13
            } else if n > 60 {
                14
            } else if n > 50 {
                15
            } else if n > 40 {
                16
            } else if n > 30 {
                15
            } else if n > 20 {
                14
            } else if n > 10 {
                13
            } else {
                12
            };
        } else if t.counter[i] % 10 == 0 {
            if f == 8 || f == 10 {
                t.frame[i] += 1;
            } else if f == 9 || f == 11 {
                t.frame[i] -= 1;
            }
        }
    }

    /// The water-strider loop (`Main.cs:16157-16196`): counter-first,
    /// idle 0/8 → `Next(110, 210)`, 17/20 → `Next(150, 260)`, else 5.
    fn step_water_strider(&mut self, i: usize, rand: &mut UnifiedRandom) {
        let t = &mut self.water_strider;
        t.counter[i] += 1;
        let f = t.frame[i];
        let idle = f == 0 || f == 8;
        let perch = f == 17 || f == 20;
        let mut limit = 5;
        if idle {
            limit = rand.next_range(110, 210);
        }
        if perch {
            limit = rand.next_range(150, 260);
        }
        if t.counter[i] < limit {
            return;
        }
        t.counter[i] = 0;
        if f == 0 && rand.next_max(2) == 0 {
            t.frame[i] = 16;
            return;
        }
        if f == 8 && rand.next_max(2) == 0 {
            t.frame[i] = 19;
            return;
        }
        if f == 15 || f == 18 {
            t.frame[i] = 0;
            return;
        }
        t.frame[i] += 1;
        if t.frame[i] > 21 {
            t.frame[i] = 8;
        }
    }

    /// The fairy-jar block (`Main.cs:16197-16227`): slot 0 advances on its
    /// own `> 5` counter (wrap 11); slots 1..24 are pure offsets of slot 0
    /// (`+ i % 12`, +1 at `% 2 == 0`, `% 5 == 0`, `% 8 == 0`, wrapped 12).
    fn step_fairy_jar(&mut self, _rand: &mut UnifiedRandom) {
        let t = &mut self.fairy_jar;
        t.counter[0] += 1;
        if t.counter[0] > 5 {
            t.counter[0] = 0;
            t.frame[0] += 1;
            if t.frame[0] > 11 {
                t.frame[0] = 0;
            }
        }
        for i in 1..N {
            let mut num50 = t.frame[0] + (i as i32) % 12;
            if i % 2 == 0 {
                num50 += 1;
            }
            if i % 5 == 0 {
                num50 += 1;
            }
            if i % 8 == 0 {
                num50 += 1;
            }
            while num50 >= 12 {
                num50 -= 12;
            }
            t.frame[i] = num50;
        }
    }

    /// The fish-bowl loop (`Main.cs:16228-16384`): the mode byte picks the
    /// swim cadence (1 → 5 ticks, 2 → 20, 3 → `Next(5, 3600)` per tick,
    /// else ≤10 frames at 10 ticks); each mode arm rolls its own
    /// mode-refresh `Next(900)/Next(3600)` + `Next(5)`.
    fn step_fish_bowl(&mut self, i: usize, rand: &mut UnifiedRandom) {
        let t = &mut self.fish_bowl;
        let mode = &mut self.fish_bowl_mode;
        let max_value = 5; // Main.cs:16230
        let f = t.frame[i];
        if mode[i] == 1 {
            if rand.next_max(900) == 0 {
                mode[i] = rand.next_max(max_value) as u8;
            }
            t.counter[i] += 1;
            if t.counter[i] < 5 {
                return;
            }
            t.counter[i] = 0;
            if f == 10 {
                if rand.next_max(20) == 0 {
                    t.frame[i] = 11;
                    mode[i] = 0;
                } else {
                    t.frame[i] = 1;
                }
            } else {
                t.frame[i] += 1;
            }
        } else if mode[i] == 2 {
            if rand.next_max(3600) == 0 {
                mode[i] = rand.next_max(max_value) as u8;
            }
            t.counter[i] += 1;
            if t.counter[i] < 20 {
                return;
            }
            t.counter[i] = 0;
            if f == 10 {
                if rand.next_max(20) == 0 {
                    t.frame[i] = 11;
                    mode[i] = 0;
                } else {
                    t.frame[i] = 1;
                }
            } else {
                t.frame[i] += 1;
            }
        } else if mode[i] == 3 {
            if rand.next_max(3600) == 0 {
                mode[i] = rand.next_max(max_value) as u8;
            }
            t.counter[i] += 1;
            if t.counter[i] < rand.next_range(5, 3600) {
                return;
            }
            t.counter[i] = 0;
            if f == 10 {
                if rand.next_max(20) == 0 {
                    t.frame[i] = 11;
                    mode[i] = 0;
                } else {
                    t.frame[i] = 1;
                }
            } else {
                t.frame[i] += 1;
            }
        } else if f <= 10 {
            if rand.next_max(3600) == 0 {
                mode[i] = rand.next_max(max_value) as u8;
            }
            t.counter[i] += 1;
            if t.counter[i] < 10 {
                return;
            }
            t.counter[i] = 0;
            if f == 10 {
                if rand.next_max(12) == 0 {
                    t.frame[i] = 11;
                } else {
                    t.frame[i] = 1;
                }
            } else {
                t.frame[i] += 1;
            }
        } else if f == 12 || f == 13 {
            t.counter[i] += 1;
            if t.counter[i] < 10 {
                return;
            }
            t.counter[i] = 0;
            t.frame[i] += 1;
            if t.frame[i] > 13 {
                t.frame[i] = if rand.next_max(20) == 0 { 14 } else { 12 };
            }
        } else {
            if f < 11 {
                return;
            }
            t.counter[i] += 1;
            if t.counter[i] >= 10 {
                t.counter[i] = 0;
                t.frame[i] += 1;
                if t.frame[i] > 16 {
                    t.frame[i] = 4;
                }
            }
        }
    }

    /// The lava-fish-bowl loop (`Main.cs:16385-16397`): `counter >
    /// 4 + Next(3)` (a fresh roll every tick), wrap 26.
    fn step_lava_fish_bowl(&mut self, i: usize, rand: &mut UnifiedRandom) {
        let t = &mut self.lava_fish_bowl;
        t.counter[i] += 1;
        if t.counter[i] > 4 + rand.next_max(3) {
            t.counter[i] = 0;
            t.frame[i] += 1;
            if t.frame[i] > 26 {
                t.frame[i] = 0;
            }
        }
    }

    /// The butterfly loop (`Main.cs:16398-16525`), row-major over 9 rows.
    /// EVERY tick draws the mode-refresh `Next(3600)` (+ `Next(5)`/`Next(2)`
    /// on a hit) AND the cadence `Next(3, 16)` — the cadence roll is drawn
    /// even when the mode overrides it (`:16411`, the module-doc note).
    fn step_butterfly(&mut self, r: usize, i: usize, rand: &mut UnifiedRandom) {
        let t = &mut self.butterfly;
        let mode = &mut self.butterfly_mode;
        t.counter[r][i] += 1;
        if rand.next_max(3600) == 0 {
            mode[r][i] = rand.next_max(5) as u8;
            if rand.next_max(2) == 0 {
                mode[r][i] += 10;
            }
        }
        let m = mode[r][i];
        let mut num55 = rand.next_range(3, 16);
        if m == 1 || m == 11 {
            num55 = 3;
        }
        if m == 2 || m == 12 {
            num55 = 5;
        }
        if m == 3 || m == 13 {
            num55 = 10;
        }
        if m == 4 || m == 14 {
            num55 = 15;
        }
        let f = t.frame[r][i];
        if m >= 10 {
            // The reverse-flight modes (:16428-16494).
            if f <= 7 {
                if t.counter[r][i] < num55 {
                    return;
                }
                t.counter[r][i] = 0;
                t.frame[r][i] -= 1;
                if t.frame[r][i] < 0 {
                    t.frame[r][i] = 7;
                }
                let f = t.frame[r][i];
                if f != 1 && f != 4 && f != 6 {
                    return;
                }
                if rand.next_max(20) == 0 {
                    t.frame[r][i] += 8;
                } else if rand.next_max(6) == 0 {
                    if mode[r][i] >= 10 {
                        mode[r][i] -= 10;
                    } else {
                        mode[r][i] += 10;
                    }
                }
            } else {
                if t.counter[r][i] < num55 {
                    return;
                }
                t.counter[r][i] = 0;
                t.frame[r][i] -= 1;
                if t.frame[r][i] < 8 {
                    t.frame[r][i] = 14;
                }
                let f = t.frame[r][i];
                if f != 9 && f != 12 && f != 14 {
                    return;
                }
                if rand.next_max(20) == 0 {
                    t.frame[r][i] -= 8;
                } else if rand.next_max(6) == 0 {
                    if mode[r][i] >= 10 {
                        mode[r][i] -= 10;
                    } else {
                        mode[r][i] += 10;
                    }
                }
            }
        } else if f <= 7 {
            if t.counter[r][i] >= num55 {
                t.counter[r][i] = 0;
                t.frame[r][i] += 1;
                if t.frame[r][i] > 7 {
                    t.frame[r][i] = 0;
                }
                let f = t.frame[r][i];
                if (f == 1 || f == 4 || f == 6) && rand.next_max(10) == 0 {
                    t.frame[r][i] += 8;
                }
            }
        } else if t.counter[r][i] >= num55 {
            t.counter[r][i] = 0;
            t.frame[r][i] += 1;
            if t.frame[r][i] > 15 {
                t.frame[r][i] = 8;
            }
            let f = t.frame[r][i];
            if (f == 9 || f == 12 || f == 14) && rand.next_max(10) == 0 {
                t.frame[r][i] -= 8;
            }
        }
    }

    /// `Main.UpdateDragonflyJarFrames` (`Main.cs:16606-16615`) — the
    /// 7-row × 25-slot walk over `UpdateDragonflyJarFrame`.
    fn update_dragonfly_jar_frames(&mut self, rand: &mut UnifiedRandom) {
        for r in 0..7 {
            for i in 0..N {
                self.update_dragonfly_jar_frame(r, i, rand);
            }
        }
    }

    /// `Main.UpdateDragonflyJarFrame` (`Main.cs:16668-16727`): frame 0
    /// idles 300 ticks then picks a hover row via `NextFromList(1, 5, 9)`;
    /// the hover rows 1/5/9 step every 6 ticks and re-pick at 100
    /// (`NextFromList(0, 5, 5, 9, 9)` from row 1); the wing rows derive the
    /// frame from `counter / 4 % 4` (mirrored at >2) with a `Next(30)`
    /// early-exit at counter ≥ 40.
    fn update_dragonfly_jar_frame(&mut self, r: usize, i: usize, rand: &mut UnifiedRandom) {
        let t = &mut self.dragonfly;
        t.counter[r][i] += 1;
        match t.frame[r][i] {
            0 => {
                if t.counter[r][i] >= 300 && rand.next_max(60) == 0 {
                    t.counter[r][i] = 0;
                    t.frame[r][i] = rand.next_from_slice(&[1, 5, 9]);
                }
            }
            1 | 5 | 9 => {
                if t.counter[r][i] >= 100 {
                    t.counter[r][i] = 0;
                    if t.frame[r][i] == 1 {
                        t.frame[r][i] = rand.next_from_slice(&[0, 5, 5, 9, 9]);
                    } else {
                        t.frame[r][i] = 1;
                    }
                } else if t.counter[r][i] >= 6 {
                    t.counter[r][i] = 0;
                    t.frame[r][i] += 1;
                }
            }
            2..=4 | 6..=8 | 10..=12 => {
                let num = t.frame[r][i] - 1;
                let num2 = num % 4 - 1;
                let mut num3 = t.counter[r][i] / 4 % 4;
                if num3 > 2 {
                    num3 = 4 - num3;
                }
                t.frame[r][i] = num - num2 + 1 + num3;
                if t.counter[r][i] >= 40 && rand.next_max(30) == 0 {
                    t.counter[r][i] = 100;
                    t.frame[r][i] = num - num2 + 1 - 1;
                }
            }
            _ => {}
        }
    }

    /// The jellyfish loop (`Main.cs:16527-16595`), row-major over 3 rows.
    /// The mode rolls are sequential `if`s (NOT else-if — a mode set to 1
    /// this tick does not re-test the `== 2` arm until next tick anyway,
    /// but a 2 → 3 transition CAN draw both rolls in one tick); the
    /// cadence roll runs per mode class; the four frame arms are also
    /// sequential `if`s, so a mode flip inside one arm can gate the next.
    fn step_jellyfish(&mut self, r: usize, i: usize, rand: &mut UnifiedRandom) {
        let t = &mut self.jellyfish;
        let mode = &mut self.jellyfish_mode;
        t.counter[r][i] += 1;
        if mode[r][i] == 0 && rand.next_max(1800) == 0 {
            mode[r][i] = 1;
        }
        if mode[r][i] == 2 && rand.next_max(60) == 0 {
            mode[r][i] = 3;
        }
        let mut num58 = 1;
        if mode[r][i] == 0 {
            num58 = rand.next_range(10, 20);
        }
        if mode[r][i] == 1 {
            num58 = rand.next_range(15, 25);
        }
        if mode[r][i] == 2 {
            num58 = rand.next_range(4, 9);
        }
        if mode[r][i] == 3 {
            num58 = rand.next_range(15, 25);
        }
        let f = t.frame[r][i];
        if mode[r][i] == 0 && f <= 3 && t.counter[r][i] >= num58 {
            t.counter[r][i] = 0;
            t.frame[r][i] += 1;
            if t.frame[r][i] >= 4 {
                t.frame[r][i] = 0;
            }
        }
        if mode[r][i] == 1 && t.frame[r][i] <= 7 && t.counter[r][i] >= num58 {
            t.counter[r][i] = 0;
            t.frame[r][i] += 1;
            if t.frame[r][i] >= 7 {
                mode[r][i] = 2;
            }
        }
        if mode[r][i] == 2 && t.frame[r][i] <= 9 && t.counter[r][i] >= num58 {
            t.counter[r][i] = 0;
            t.frame[r][i] += 1;
            if t.frame[r][i] >= 9 {
                t.frame[r][i] = 7;
            }
        }
        if mode[r][i] == 3 && t.frame[r][i] <= 10 && t.counter[r][i] >= num58 {
            t.counter[r][i] = 0;
            t.frame[r][i] += 1;
            if t.frame[r][i] >= 10 {
                t.frame[r][i] = 3;
                mode[r][i] = 0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drawdata_furniture::{cage_draw_data, CAGE_FRAMES};

    /// W4-f14 gate witness: `critterCage == false` returns before ANY draw
    /// (Main.cs:13513-13516) — the stream is untouched and no table moves.
    #[test]
    fn w4_f14_the_gate_returns_before_any_draw() {
        let mut a = CritterCageAnimator::zeroed();
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        a.animate_critter_cages(&mut rng);
        assert_eq!(rng.peek(), peek, "the closed gate draws nothing");
        assert_eq!(a.anim_key(), CritterCageAnimator::zeroed().anim_key());
    }

    /// W4-f14 stream witness: one open-gate tick draws on the CALLER's
    /// stream (the exact Main.rand interleave), and two runs from the same
    /// seed are bit-identical (state + stream position).
    #[test]
    fn w4_f14_one_tick_is_deterministic_and_consumes_the_caller_stream() {
        let run = || {
            let mut a = CritterCageAnimator::zeroed();
            a.critter_cage = true;
            let mut rng = UnifiedRandom::new(42);
            a.animate_critter_cages(&mut rng);
            (a.anim_key(), rng.peek())
        };
        let (k1, p1) = run();
        let (k2, p2) = run();
        assert_eq!(k1, k2, "same seed, same tables");
        assert_eq!(p1, p2, "same seed, same stream position");
        assert_ne!(p1, UnifiedRandom::new(42).peek(), "the tick drew");
        assert_ne!(k1, CritterCageAnimator::zeroed().anim_key());
    }

    /// W4-f14 warm-up witness: `new(seed)` replays the 3600-tick
    /// Initialize_TileAndNPCData2 warm-up (Main.cs:6800-6807) on a FRESH
    /// clone — the caller's live stream is not consumed — and ends with
    /// the gate closed. Same seed → same post-warm-up tables.
    #[test]
    fn w4_f14_the_warmup_replays_on_a_clone_not_the_live_stream() {
        let a = CritterCageAnimator::new(123);
        let b = CritterCageAnimator::new(123);
        assert!(!a.critter_cage, "the warm-up closes the gate (:6807)");
        assert_eq!(a.anim_key(), b.anim_key(), "the warm-up is seeded");
        assert_ne!(
            a.anim_key(),
            CritterCageAnimator::zeroed().anim_key(),
            "3600 ticks moved the tables"
        );
        // The live-stream stand-in: a caller rng seeded the same is NOT
        // advanced by the constructor (it owns its clone).
        let mut live = UnifiedRandom::new(123);
        let peek = live.peek();
        let _ = CritterCageAnimator::new(123);
        assert_eq!(live.peek(), peek, "the live stream is untouched");
    }

    /// W4-f14 anim_key witness: the key covers every FRAME table (the draw
    /// input) — a moved slot changes it, an unchanged animator does not,
    /// and the 2-D tables ride it too.
    #[test]
    fn w4_f14_anim_key_tracks_the_frame_tables() {
        let mut a = CritterCageAnimator::zeroed();
        let k0 = a.anim_key();
        assert_eq!(k0, CritterCageAnimator::zeroed().anim_key());
        a.bunny.frame[3] = 5;
        let k1 = a.anim_key();
        assert_ne!(k0, k1, "a 1-D frame slot rides the key");
        assert_eq!(k1, a.anim_key(), "unchanged state, unchanged key");
        a.butterfly.frame[8][2] = 9; // the 620 jar's row
        assert_ne!(k1, a.anim_key(), "a 2-D frame slot rides the key");
        let k2 = a.anim_key();
        a.bunny.counter[3] = 99; // counters are NOT draw input
        assert_eq!(k2, a.anim_key(), "the counters stay out of the key");
    }

    /// W4-f14 frame_of witness: the lookup routes every CageAnim to its
    /// table/row — the four 2-D tables read the C# first index, the 1-D
    /// tables ignore the row.
    #[test]
    fn w4_f14_frame_of_routes_table_and_row() {
        let mut a = CritterCageAnimator::zeroed();
        a.bunny.frame[6] = 3;
        a.scorpion.frame[1][6] = 8;
        a.slug.frame[2][16] = 11;
        a.butterfly.frame[8][24] = 14;
        a.dragonfly.frame[6][24] = 5;
        a.jellyfish.frame[2][16] = 7;
        assert_eq!(a.frame_of(CageAnim::Bunny, 0, 6), 3);
        assert_eq!(a.frame_of(CageAnim::Bunny, 1, 6), 3, "1-D ignores row");
        assert_eq!(a.frame_of(CageAnim::Scorpion, 1, 6), 8);
        assert_eq!(a.frame_of(CageAnim::Slug, 2, 16), 11);
        assert_eq!(a.frame_of(CageAnim::Butterfly, 8, 24), 14);
        assert_eq!(a.frame_of(CageAnim::Dragonfly, 6, 24), 5);
        assert_eq!(a.frame_of(CageAnim::Jellyfish, 2, 16), 7);
    }

    /// W4-f14 draw-feed witness: the animator's tables drive the census
    /// `cage_draw_data` rewrite — a moved bunny slot at the (12,12) big
    /// probe (idx 6) shifts addFrY by frame × 54, and the 620 jar reads
    /// butterfly row 8 through the WATER probe (idx 24).
    #[test]
    fn w4_f14_the_tables_feed_the_census_cage_rewrite() {
        let mut a = CritterCageAnimator::zeroed();
        a.bunny.frame[6] = 2;
        a.butterfly.frame[8][24] = 3;
        let d = cage_draw_data(275, 12, 12, 0, 0, CAGE_FRAMES, |t, r, i| a.frame_of(t, r, i))
            .unwrap();
        assert_eq!((d.top, d.add_fr_y), (2, 2 * 54));
        let d = cage_draw_data(620, 12, 12, 0, 0, CAGE_FRAMES, |t, r, i| a.frame_of(t, r, i))
            .unwrap();
        assert_eq!((d.top, d.add_fr_y), (2, 3 * 36));
        // The zeroed tables reproduce the static arms (the census pin).
        let z = CritterCageAnimator::zeroed();
        let d = cage_draw_data(275, 13, 7, 18, 18, CAGE_FRAMES, |t, r, i| z.frame_of(t, r, i))
            .unwrap();
        assert_eq!(d, crate::drawdata::draw_data(275, 13, 7, 18, 18, 0, false));
    }

    /// W4-f14 owl witness: the owl frame is a pure counter function
    /// (Main.cs:16617-16666). Driving one tick from counter-1 with the
    /// `% 40 != 0` bump arm (which short-circuits the Next(80) roll):
    /// 9 → 10 → block 0 phase 1 → frame 1; 29 → 30 → phase 3 folds to 1;
    /// 49 → 50 → block 1 → 8 - 1 = 7; 89 → 90 → block 2 → 7 + 1 = 8;
    /// 129 → 130 → block 3 → 11 + 1 = 12.
    #[test]
    fn w4_f14_the_owl_frame_is_the_counter_function() {
        let cases = [(9, 1), (29, 1), (49, 7), (89, 8), (129, 12)];
        for (counter, frame) in cases {
            let mut a = CritterCageAnimator::zeroed();
            a.owl.counter[0] = counter;
            let mut rng = UnifiedRandom::new(1);
            a.update_owl_cage_frame(0, &mut rng);
            assert_eq!(a.owl.counter[0], counter + 1, "the bump arm fired");
            assert_eq!(a.owl.frame[0], frame, "counter {counter} → frame");
        }
        // The block-1 phase-0 rest: counter 39 → 40 → num 0 → frame 0.
        let mut a = CritterCageAnimator::zeroed();
        a.owl.counter[0] = 39;
        a.update_owl_cage_frame(0, &mut UnifiedRandom::new(1));
        assert_eq!(a.owl.frame[0], 0);
    }

    /// W4-f14 fairy-jar witness: slots 1..24 are pure offsets of slot 0
    /// (Main.cs:16207-16227) — no RNG, the +1 bumps at % 2/% 5/% 8 and the
    /// wrap-12 fold.
    #[test]
    fn w4_f14_the_fairy_jar_slots_are_slot0_offsets() {
        let mut a = CritterCageAnimator::zeroed();
        a.fairy_jar.frame[0] = 11;
        a.step_fairy_jar(&mut UnifiedRandom::new(1));
        // slot 0: counter 1 (<= 5) keeps frame 11.
        assert_eq!(a.fairy_jar.frame[0], 11);
        // slot 1: 11 + 1 = 12 → 0 (no bumps).
        assert_eq!(a.fairy_jar.frame[1], 0);
        // slot 2: 11 + 2 + 1 = 14 → 2.
        assert_eq!(a.fairy_jar.frame[2], 2);
        // slot 5: 11 + 5 + 1 = 17 → 5.
        assert_eq!(a.fairy_jar.frame[5], 5);
        // slot 8: 11 + 8 + 1 + 1 = 21 → 9.
        assert_eq!(a.fairy_jar.frame[8], 9);
        // slot 10: 11 + 10 + 1 + 1 = 23 → 11.
        assert_eq!(a.fairy_jar.frame[10], 11);
        // slot 24: 11 + 0 + 1 + 1 = 13 → 1 (% 12 == 0, % 2/% 8 bump).
        assert_eq!(a.fairy_jar.frame[24], 1);
    }

    /// W4-f14 pufferfish witness: the puff roll's start-frame switch
    /// (Main.cs:15292-15325) and the inflate ladder's 7-tick phases —
    /// the frame stays in 0..70 and the deflate exits to the swim row.
    #[test]
    fn w4_f14_the_pufferfish_machine_stays_in_range() {
        let mut a = CritterCageAnimator::zeroed();
        a.critter_cage = true;
        let mut rng = UnifiedRandom::new(9);
        for _ in 0..4000 {
            a.animate_critter_cages(&mut rng);
            for &f in &a.pufferfish.frame {
                assert!((0..=70).contains(&f), "pufferfish frame {f} out of range");
            }
            for &f in &a.bunny.frame {
                assert!((0..=21).contains(&f), "bunny frame {f} out of range");
            }
            for row in &a.butterfly.frame {
                for &f in row {
                    assert!((0..=15).contains(&f), "butterfly frame {f}");
                }
            }
            for row in &a.jellyfish.frame {
                for &f in row {
                    assert!((0..=10).contains(&f), "jellyfish frame {f}");
                }
            }
        }
    }

    /// W4-f14 order witness (include_str): the driver walks the loops in
    /// the C# source order — the orange squirrel between grey and mallard
    /// (:13752), the owl between pufferfish and snail (:15476), the
    /// dragonfly between butterfly and jellyfish (:16526).
    #[test]
    fn w4_f14_the_driver_order_matches_the_csharp() {
        let src = include_str!("drawdata_cages.rs");
        let body = &src[src.find("pub fn animate_critter_cages").unwrap()..];
        let body = &body[..body.find("// ---- per-animal slot bodies").unwrap()];
        let pos = |needle: &str| body.find(needle).expect(needle);
        let order = [
            "self.bunny = t;",
            "self.squirrel = t;",
            "self.squirrel_orange = t;",
            "self.mallard = t;",
            "self.duck = t;",
            "self.step_grebe",
            "self.step_seagull",
            "self.bird = t;",
            "self.blue_bird = t;",
            "self.red_bird = t;",
            "self.step_macaw",
            "self.step_scorpion",
            "self.step_penguin",
            "self.step_turtle",
            "self.step_seahorse",
            "self.step_pufferfish",
            "self.update_owl_cage_frames",
            "self.snail = t;",
            "self.snail2 = t;",
            "self.step_frog",
            "self.step_mouse",
            "self.step_worm",
            "self.step_slug",
            "self.step_grasshopper",
            "self.step_maggot",
            "self.step_ladybug",
            "self.step_rat",
            "self.step_water_strider",
            "self.step_fairy_jar",
            "self.step_fish_bowl",
            "self.step_lava_fish_bowl",
            "self.step_butterfly",
            "self.update_dragonfly_jar_frames",
            "self.step_jellyfish",
        ];
        let mut last = 0;
        for needle in order {
            let p = pos(needle);
            assert!(p > last, "{needle} must follow the previous loop");
            last = p;
        }
        // The gate precedes every loop (Main.cs:13513-13516).
        assert!(pos("if !self.critter_cage") < pos("self.bunny = t;"));
    }
}
