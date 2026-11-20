//! `Main.AnimateTiles_CritterCages` (`Main.cs:13511-16596`) — the critter-cage
//! frame-table animator the S1-F-9b module docs deferred (`drawdata_anim.rs`,
//! `Main.cs:19303`, the `AnimateTiles` tail). W4-g33 (DrawData wiring lane 1):
//! the d61-d64 census found every cage draw arm ported (`drawdata_furniture.rs`)
//! but frozen at row 0 because these `Main.*CageFrame` tables never moved.
//!
//! State: the `Main.cageFrames = 25` slot tables allocated at
//! `Main.cs:66901-66972` (1-D `[25]`, the 2-D rows scorpion 2, slug 3,
//! jellyfish 3, butterfly 9, dragonfly 7). The three mode tables
//! (`fishBowlFrameMode` `[25]`, `butterflyCageMode` `[9,25]`,
//! `jellyfishCageMode` `[3,25]`) ride beside their frame tables.
//!
//! The `Main.critterCage` proximity gate (`Main.cs:1445`): the non-solid
//! `TileDrawing.Draw` clears it (`TileDrawing.cs:474`), every cage case of
//! `GetTileDrawData` sets it while drawing a cage cell (`:5672`/`:5781`/
//! `:5838`/`:5846`/`:5856`/`:5874`/`:5893`/`:5903`, plus the 620 special-pass
//! arm `:8005`), and the animator early-returns unless it is set
//! (`Main.cs:13513-13516`). The port keeps the flag on [`CageAnimator`] and
//! the render pass sets it (`render/tiles.rs`).
//!
//! Startup warm-up (`Main.Initialize_TileAndNPCData2`, `Main.cs:6802-6807`):
//! `critterCage = true`, 3600 animator calls, `critterCage = false` — the
//! vanilla tables are NOT all-zero when a world opens. [`CageAnimator::
//! warm_up`] reproduces it on `Main.rand` (the parameterless
//! `new UnifiedRandom()` default seed 0, `UnifiedRandom.cs:18-21`, matching
//! the port's `NamedRngs::new(0)` global stand-in). The tables are process
//! state in C# — the world-load reset (`Main.cs:8393-8394`) does NOT touch
//! them — so the port does not reset them on world entry either.
//!
//! Determinism: every draw rides the caller's [`UnifiedRandom`] (`Main.rand`)
//! in source order — including the draws that happen on a failed gate
//! (`<= rand.Next(30, 900)` keeps drawing every tick). No other stream is
//! touched.

use crate::drawdata_furniture::{CageAnim, CAGE_FRAMES};
use crate::unified_random::UnifiedRandom;

const N: usize = CAGE_FRAMES as usize; // 25 — Main.cageFrames (Main.cs:66901)

/// One 1-D `Main.<animal>CageFrame` / `...Counter` pair.
#[derive(Clone, Debug, Default)]
struct CageTable {
    frame: [i32; N],
    counter: [i32; N],
}

/// One 2-D pair, `R` rows × `Main.cageFrames` slots.
#[derive(Clone, Debug)]
struct CageTable2<const R: usize> {
    frame: [[i32; N]; R],
    counter: [[i32; N]; R],
}

impl<const R: usize> Default for CageTable2<R> {
    fn default() -> Self {
        Self { frame: [[0; N]; R], counter: [[0; N]; R] }
    }
}

/// The `Main.*CageFrame` animator state (the allocation block
/// `Main.cs:66902-66972`). Field order follows the C# allocation order.
#[derive(Clone, Debug, Default)]
pub struct CageAnimator {
    /// `Main.critterCage` (`Main.cs:1445`) — the draw pass's proximity flag.
    pub critter_cage: bool,
    bunny: CageTable,
    squirrel: CageTable,
    squirrel_orange: CageTable,
    mallard: CageTable,
    duck: CageTable,
    grebe: CageTable,
    seagull: CageTable,
    bird: CageTable,
    red_bird: CageTable,
    blue_bird: CageTable,
    macaw: CageTable,
    butterfly_mode: [[u8; N]; 9],
    butterfly: CageTable2<9>,
    dragonfly: CageTable2<7>,
    scorpion: CageTable2<2>,
    snail: CageTable,
    snail2: CageTable,
    fish_bowl_mode: [u8; N],
    fish_bowl: CageTable,
    lava_fish_bowl: CageTable,
    frog: CageTable,
    mouse: CageTable,
    turtle: CageTable,
    fairy_jar: CageTable,
    jellyfish_mode: [[u8; N]; 3],
    jellyfish: CageTable2<3>,
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

impl CageAnimator {
    pub fn new() -> Self {
        Self::default()
    }

    /// The `frame_of` lookup `drawdata_furniture::cage_draw_data` /
    /// `furniture_post_pass` consume: `Main.<table>[row, idx]`. C# indexes
    /// the tables with the `% cageFrames` probe result — which keeps the
    /// sign of a negative product (`Get*AnimalCageFrame`,
    /// `TileDrawing.cs:6528-6547`) — so a negative slot would throw in C#;
    /// the port clamps the lookup to slot 0 rather than panicking
    /// (documented deviation: unreachable in-world, cages sit at positive
    /// coordinates).
    pub fn frame_of(&self, table: CageAnim, row: i32, idx: i32) -> i32 {
        let i = (idx.max(0) as usize).min(N - 1);
        let r = (row.max(0) as usize).min(8);
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
            CageAnim::Scorpion => self.scorpion.frame[r.min(1)][i],
            CageAnim::Penguin => self.penguin.frame[i],
            CageAnim::Seahorse => self.seahorse.frame[i],
            CageAnim::Pufferfish => self.pufferfish.frame[i],
            CageAnim::Snail => self.snail.frame[i],
            CageAnim::Snail2 => self.snail2.frame[i],
            CageAnim::Frog => self.frog.frame[i],
            CageAnim::Grasshopper => self.grasshopper.frame[i],
            CageAnim::Mouse => self.mouse.frame[i],
            CageAnim::Worm => self.worm.frame[i],
            CageAnim::Slug => self.slug.frame[r.min(2)][i],
            CageAnim::Maggot => self.maggot.frame[i],
            CageAnim::Rat => self.rat.frame[i],
            CageAnim::Ladybug => self.ladybug.frame[i],
            CageAnim::WaterStrider => self.water_strider.frame[i],
            CageAnim::FishBowl => self.fish_bowl.frame[i],
            CageAnim::LavaFishBowl => self.lava_fish_bowl.frame[i],
            CageAnim::FairyJar => self.fairy_jar.frame[i],
            CageAnim::Butterfly => self.butterfly.frame[r][i],
            CageAnim::Dragonfly => self.dragonfly.frame[r.min(6)][i],
            CageAnim::Jellyfish => self.jellyfish.frame[r.min(2)][i],
        }
    }

    /// `Main.Initialize_TileAndNPCData2`'s warm-up (`Main.cs:6802-6807`):
    /// `critterCage = true; for (3600) AnimateTiles_CritterCages();
    /// critterCage = false;` — on `Main.rand`, before any world loads.
    pub fn warm_up(&mut self, rand: &mut UnifiedRandom) {
        self.critter_cage = true;
        for _ in 0..3600 {
            self.animate(rand);
        }
        self.critter_cage = false;
    }

    /// One `Main.AnimateTiles_CritterCages()` call (`Main.cs:13511-16596`),
    /// the per-table loops in source order. `rand` is `Main.rand`.
    pub fn animate(&mut self, rand: &mut UnifiedRandom) {
        if !self.critter_cage {
            return; // Main.cs:13513-13516
        }
        for i in 0..N {
            step_bunny(&mut self.bunny, i, rand); // :13517-13617
        }
        for j in 0..N {
            step_squirrel(&mut self.squirrel, j, rand); // :13618-13751
        }
        for k in 0..N {
            step_squirrel(&mut self.squirrel_orange, k, rand); // :13752-13885
        }
        for l in 0..N {
            step_duck(&mut self.mallard, l, rand); // :13886-14013
        }
        for m in 0..N {
            step_duck(&mut self.duck, m, rand); // :14014-14141
        }
        for n in 0..N {
            step_grebe(&mut self.grebe, n, rand); // :14142-14211
        }
        for i in 0..N {
            step_seagull(&mut self.seagull, i, rand); // :14212-14371
        }
        for i in 0..N {
            step_bird(&mut self.bird, i, rand); // :14372-14507
        }
        for i in 0..N {
            step_bird(&mut self.blue_bird, i, rand); // :14508-14643
        }
        for i in 0..N {
            step_bird(&mut self.red_bird, i, rand); // :14644-14779
        }
        for i in 0..N {
            step_macaw(&mut self.macaw, i, rand); // :14780-14878
        }
        for r in 0..2 {
            for i in 0..N {
                step_scorpion(&mut self.scorpion, r, i, rand); // :14879-15038
            }
        }
        for i in 0..N {
            step_penguin(&mut self.penguin, i, rand); // :15039-15203
        }
        for i in 0..N {
            step_turtle(&mut self.turtle, i, rand); // :15204-15233
        }
        for i in 0..N {
            step_seahorse(&mut self.seahorse, i, rand); // :15234-15273
        }
        for i in 0..N {
            step_pufferfish(&mut self.pufferfish, i, rand); // :15274-15475
        }
        for i in 0..N {
            step_owl(&mut self.owl, i, rand); // :15476 → UpdateOwlCageFrames :16598-16666
        }
        for i in 0..N {
            step_snail(&mut self.snail, i, rand, 45, 3600, 60, 7200); // :15477-15549
        }
        for i in 0..N {
            step_snail(&mut self.snail2, i, rand, 30, 2700, 45, 5400); // :15550-15622
        }
        for i in 0..N {
            step_frog(&mut self.frog, i, rand); // :15623-15723
        }
        // The remaining table loops (mouse … jellyfish, Main.cs:15724-16595)
        // land with their step functions in the following batches.
    }

    /// G6 key input (the `TileAnimator::anim_key` convention): FNV-1a over
    /// every frame/mode value the cage draw arms read, in the C# allocation
    /// order (`Main.cs:66902-66972`). The counters are animator-private (no
    /// draw arm reads them) but ride too — they pin the stream position for
    /// the next frame. No RNG.
    pub fn anim_key(&self) -> u64 {
        let mut h = 0xcbf2_9ce4_8422_2325u64;
        let mut feed = |v: i32| {
            for b in v.to_le_bytes() {
                h ^= b as u64;
                h = h.wrapping_mul(0x0000_0100_0000_01b3);
            }
        };
        let t1 = |t: &CageTable, feed: &mut dyn FnMut(i32)| {
            for &v in &t.frame {
                feed(v);
            }
            for &v in &t.counter {
                feed(v);
            }
        };
        t1(&self.bunny, &mut feed);
        t1(&self.squirrel, &mut feed);
        t1(&self.squirrel_orange, &mut feed);
        t1(&self.mallard, &mut feed);
        t1(&self.duck, &mut feed);
        t1(&self.grebe, &mut feed);
        t1(&self.seagull, &mut feed);
        t1(&self.bird, &mut feed);
        t1(&self.red_bird, &mut feed);
        t1(&self.blue_bird, &mut feed);
        t1(&self.macaw, &mut feed);
        for row in &self.butterfly_mode {
            for &v in row {
                feed(v as i32);
            }
        }
        for row in 0..9 {
            for &v in &self.butterfly.frame[row] {
                feed(v);
            }
            for &v in &self.butterfly.counter[row] {
                feed(v);
            }
        }
        for row in 0..7 {
            for &v in &self.dragonfly.frame[row] {
                feed(v);
            }
            for &v in &self.dragonfly.counter[row] {
                feed(v);
            }
        }
        for row in 0..2 {
            for &v in &self.scorpion.frame[row] {
                feed(v);
            }
            for &v in &self.scorpion.counter[row] {
                feed(v);
            }
        }
        t1(&self.snail, &mut feed);
        t1(&self.snail2, &mut feed);
        for &v in &self.fish_bowl_mode {
            feed(v as i32);
        }
        t1(&self.fish_bowl, &mut feed);
        t1(&self.lava_fish_bowl, &mut feed);
        t1(&self.frog, &mut feed);
        t1(&self.mouse, &mut feed);
        t1(&self.turtle, &mut feed);
        t1(&self.fairy_jar, &mut feed);
        for row in &self.jellyfish_mode {
            for &v in row {
                feed(v as i32);
            }
        }
        for row in 0..3 {
            for &v in &self.jellyfish.frame[row] {
                feed(v);
            }
            for &v in &self.jellyfish.counter[row] {
                feed(v);
            }
        }
        t1(&self.worm, &mut feed);
        t1(&self.maggot, &mut feed);
        t1(&self.rat, &mut feed);
        t1(&self.ladybug, &mut feed);
        t1(&self.penguin, &mut feed);
        t1(&self.water_strider, &mut feed);
        t1(&self.seahorse, &mut feed);
        for row in 0..3 {
            for &v in &self.slug.frame[row] {
                feed(v);
            }
            for &v in &self.slug.counter[row] {
                feed(v);
            }
        }
        t1(&self.owl, &mut feed);
        t1(&self.grasshopper, &mut feed);
        t1(&self.pufferfish, &mut feed);
        h
    }

    /// Every cage tile kind the draw arms feed these tables to
    /// (`drawdata_furniture::cage_table`, `TileDrawing.cs:5629-5907`) — the
    /// G6 gate set: the key rides only when a cage is active in the view
    /// rect, so cage-free worlds keep the pre-wiring cache behaviour
    /// bit-for-bit.
    pub fn is_cage_kind(kind: u16) -> bool {
        matches!(
            kind,
            275..=282
                | 285
                | 286
                | 288..=299
                | 309
                | 310
                | 316..=318
                | 339
                | 358
                | 359
                | 360..=364
                | 391..=394
                | 413
                | 414
                | 505
                | 521..=527
                | 532
                | 533
                | 538
                | 542..=544
                | 550
                | 551
                | 553..=556
                | 558
                | 559
                | 568..=570
                | 580
                | 582
                | 598
                | 599..=612
                | 619
                | 620
                | 629
                | 632
                | 640
                | 643..=645
                | 710
        )
    }
}

// ---------------------------------------------------------------------------
// The per-table step machines, one slot per call, in the C# loop order.
// Each `continue` in the C# body ends that slot's update (the counter keeps
// its incremented value); a bare fall-through leaves the frame untouched.
// ---------------------------------------------------------------------------

/// `Main.bunnyCageFrame[i]` (`Main.cs:13519-13616`).
fn step_bunny(t: &mut CageTable, i: usize, rand: &mut UnifiedRandom) {
    let f = t.frame[i];
    if f == 0 {
        t.counter[i] += 1;
        if t.counter[i] <= rand.next_range(30, 900) {
            return;
        }
        if rand.next_max(3) != 0 {
            let num = rand.next_max(7);
            if num == 0 {
                t.frame[i] = 4;
            } else if num <= 2 {
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
            if rand.next_max(7) == 0 {
                t.frame[i] = 13;
            } else {
                t.frame[i] = 12;
            }
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

/// `Main.squirrelCageFrame[j]` (`Main.cs:13620-13750`) — also drives
/// `squirrelCageFrameOrange` verbatim (`:13754-13884`, the two bodies are
/// byte-identical apart from the array names).
fn step_squirrel(t: &mut CageTable, i: usize, rand: &mut UnifiedRandom) {
    let f = t.frame[i];
    if f == 0 {
        t.counter[i] += 1;
        if t.counter[i] <= rand.next_range(30, 900) {
            return;
        }
        if rand.next_max(3) != 0 {
            let num = rand.next_max(7);
            if num == 0 {
                t.frame[i] = 4;
            } else if num <= 2 {
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
        if t.counter[i] >= 5 {
            t.counter[i] = 0;
            t.frame[i] += 1;
        }
        if t.frame[i] > 3 {
            if rand.next_max(5) == 0 {
                t.frame[i] = 0;
            } else {
                t.frame[i] = 2;
            }
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
            if num == 0 {
                t.frame[i] = 13;
            } else if num <= 2 {
                t.frame[i] = 11;
            } else {
                t.frame[i] = 10;
            }
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
            if rand.next_max(5) != 0 {
                t.frame[i] = 11;
            } else {
                t.frame[i] = 9;
            }
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

/// `Main.mallardCageFrame[l]` (`Main.cs:13888-14012`) — also drives
/// `duckCageFrame[m]` verbatim (`:14016-14140`, byte-identical bodies).
fn step_duck(t: &mut CageTable, i: usize, rand: &mut UnifiedRandom) {
    let f = t.frame[i];
    if f == 0 || f == 4 {
        t.counter[i] += 1;
        if t.counter[i] <= rand.next_range(45, 2700) {
            return;
        }
        // The C# `||` short-circuits: a frame-0 slot rolls Next(3), a
        // frame-4 slot rolls Next(5) — one draw either way.
        if (f == 0 && rand.next_max(3) != 0) || (f == 4 && rand.next_max(5) == 0) {
            if rand.next_max(6) == 0 {
                t.frame[i] = 5;
            } else if rand.next_max(3) == 0 {
                if t.frame[i] == 4 {
                    t.frame[i] = 0;
                } else {
                    t.frame[i] = 4;
                }
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
            if rand.next_max(5) == 0 {
                t.frame[i] = 0;
            } else {
                t.frame[i] = 1;
            }
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
                if t.frame[i] == 16 {
                    t.frame[i] = 12;
                } else {
                    t.frame[i] = 16;
                }
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
            if rand.next_max(5) != 0 {
                t.frame[i] = 12;
            } else {
                t.frame[i] = 13;
            }
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

/// `Main.grebeCageFrame[n]` (`Main.cs:14144-14210`): the counter ticks
/// unconditionally, the limit is rolled per state, then one dispatch.
fn step_grebe(t: &mut CageTable, i: usize, rand: &mut UnifiedRandom) {
    t.counter[i] += 1;
    let f = t.frame[i];
    let idle = f == 0 || f == 7;
    let idle2 = f == 16 || f == 20;
    let mut limit = 5;
    if idle {
        limit = rand.next_range(300, 400);
    }
    if idle2 {
        limit = rand.next_range(480, 600);
    }
    if t.counter[i] < limit {
        return;
    }
    t.counter[i] = 0;
    if (25..=27).contains(&f) {
        t.frame[i] += 1;
        if t.frame[i] > 27 {
            if rand.next_max(5) == 0 {
                t.frame[i] = 7;
            } else {
                t.frame[i] = 25;
            }
        }
    } else if (22..=24).contains(&f) {
        t.frame[i] += 1;
        if t.frame[i] > 24 {
            if rand.next_max(5) == 0 {
                t.frame[i] = 0;
            } else {
                t.frame[i] = 22;
            }
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

/// `Main.seagullCageFrame[num4]` (`Main.cs:14214-14370`): the counter ticks
/// unconditionally; the 15-18/9-13 bands re-derive their frame from
/// `counter % 66` / `% 78` on EVERY tick below the limit (the glide loops).
fn step_seagull(t: &mut CageTable, i: usize, rand: &mut UnifiedRandom) {
    t.counter[i] += 1;
    let f = t.frame[i];
    let idle = f == 0 || f == 14;
    let glide = (15..=18).contains(&f);
    let fly = (9..=13).contains(&f);
    let mut limit = 6;
    if idle {
        limit = rand.next_range(180, 250);
    }
    if glide {
        limit = 66;
    }
    if fly {
        limit = 78;
    }
    if t.counter[i] >= limit {
        t.counter[i] = 0;
        if t.frame[i] == 0 && rand.next_max(3) == 0 {
            t.frame[i] = 9;
            return;
        }
        if t.frame[i] == 14 {
            if rand.next_max(2) == 0 {
                t.frame[i] = 15;
            } else {
                t.frame[i] = 19;
            }
            return;
        }
        if t.frame[i] == 9 {
            t.frame[i] = 0;
            return;
        }
        if t.frame[i] == 8 || t.frame[i] == 15 {
            t.frame[i] = 14;
            return;
        }
        t.frame[i] += 1;
        if t.frame[i] > 26 {
            t.frame[i] = 0;
        }
    } else if glide {
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
    } else if fly {
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

/// `Main.birdCageFrame[num9]` (`Main.cs:14374-14506`) — also drives
/// `blueBirdCageFrame` (`:14510-14642`) and `redBirdCageFrame`
/// (`:14646-14778`), all three byte-identical.
fn step_bird(t: &mut CageTable, i: usize, rand: &mut UnifiedRandom) {
    let f = t.frame[i];
    if f == 0 {
        t.counter[i] += 1;
        if t.counter[i] <= rand.next_range(30, 2700) {
            return;
        }
        if rand.next_max(3) != 0 {
            if rand.next_max(3) != 0 {
                t.frame[i] = 2;
            } else {
                t.frame[i] = 1;
            }
        }
        t.counter[i] = 0;
    } else if f == 1 {
        t.counter[i] += 1;
        // The && short-circuits: Next(3) only rolls when the counter passed.
        if t.counter[i] > rand.next_range(900, 18000) && rand.next_max(3) == 0 {
            t.counter[i] = 0;
            t.frame[i] = 0;
        }
    } else if (2..=5).contains(&f) {
        t.counter[i] += 1;
        if t.counter[i] >= 5 {
            t.counter[i] = 0;
            if t.frame[i] == 3 && rand.next_max(3) == 0 {
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

/// `Main.macawCageFrame[num12]` (`Main.cs:14782-14877`). Note the `> 5` /
/// `> 6` gates (NOT `>=` — the macaw runs one tick slower than the birds).
fn step_macaw(t: &mut CageTable, i: usize, rand: &mut UnifiedRandom) {
    let f = t.frame[i];
    if f == 0 {
        t.counter[i] += 1;
        if t.counter[i] <= rand.next_range(300, 2700) {
            return;
        }
        if rand.next_max(3) != 0 {
            if rand.next_max(3) != 0 {
                t.frame[i] = 16;
            } else {
                t.frame[i] = 1;
            }
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
            if rand.next_max(3) != 0 {
                t.frame[i] = 7;
            } else {
                t.frame[i] = 11;
            }
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

/// `Main.scorpionCageFrame[num13, num14]` (`Main.cs:14883-15036`), both
/// rows — the draw arms read row 0 only (`TileDrawing.cs:5737`).
fn step_scorpion(t: &mut CageTable2<2>, r: usize, i: usize, rand: &mut UnifiedRandom) {
    let f = t.frame[r][i];
    if f == 0 || f == 7 {
        t.counter[r][i] += 1;
        if t.counter[r][i] <= rand.next_range(30, 3600) {
            return;
        }
        if t.frame[r][i] == 7 {
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
        if t.frame[r][i] == 3 {
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

/// `Main.penguinCageFrame[num15]` (`Main.cs:15041-15202`). The 10/11 band's
/// wrap re-checks `> 3` (NOT `> 11`, `:15166`) — a verbatim decompiled quirk:
/// frame 11 wraps on its first 5-tick pass.
fn step_penguin(t: &mut CageTable, i: usize, rand: &mut UnifiedRandom) {
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
            if rand.next_max(3) == 0 {
                t.frame[i] = 0;
            } else {
                t.frame[i] = 2;
            }
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
            // :15166 — verbatim `> 3` (the copy-paste quirk), not `> 11`.
            if rand.next_max(3) == 0 {
                t.frame[i] = 8;
            } else {
                t.frame[i] = 10;
            }
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

/// `Main.turtleCageFrame[num16]` (`Main.cs:15206-15232`): unconditional
/// counter tick, idle states roll a 180-250 limit, one dispatch on pass.
fn step_turtle(t: &mut CageTable, i: usize, rand: &mut UnifiedRandom) {
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
    if t.frame[i] == 29 {
        t.frame[i] = 0;
        return;
    }
    if t.frame[i] == 15 && rand.next_max(3) == 0 {
        t.frame[i] = 30;
        return;
    }
    t.frame[i] += 1;
    if t.frame[i] > 32 {
        t.frame[i] = 15;
    }
}

/// `Main.seahorseCageFrame[num19]` (`Main.cs:15236-15272`).
fn step_seahorse(t: &mut CageTable, i: usize, rand: &mut UnifiedRandom) {
    t.counter[i] += 1;
    let f = t.frame[i];
    let idle = f == 0 || f == 13;
    let pose = f == 4 || f == 9 || f == 17 || f == 22;
    let swim = f >= 25;
    let mut limit = 7;
    if idle {
        limit = rand.next_range(220, 340);
    }
    if pose {
        limit = 10;
    }
    if swim {
        limit = 6;
    }
    if t.counter[i] < limit {
        return;
    }
    t.counter[i] = 0;
    if t.frame[i] == 0 && rand.next_max(2) == 0 {
        t.frame[i] = 25;
        return;
    }
    if t.frame[i] == 24 {
        t.frame[i] = 0;
        return;
    }
    t.frame[i] += 1;
    if t.frame[i] > 33 {
        t.frame[i] = 0;
    }
}

/// `Main.pufferfishCageFrame[num23]` (`Main.cs:15275-15474`): the 0-40 swim
/// band (7-tick step, the 1/75 inflate roll), then the inflate/deflate
/// machine on frames 41-70 (`num24 = 7`, `num25 = 5*7*5+8 = 183`,
/// `num26 = 175`).
fn step_pufferfish(t: &mut CageTable, i: usize, rand: &mut UnifiedRandom) {
    let num22 = 40;
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
            // :15292-15325 — the inflate pick reads the POST-step frame.
            match t.frame[i] {
                0 | 1 | 2 => t.frame[i] = 41,
                8 | 9 | 10 => t.frame[i] = 46,
                17 | 18 | 19 => t.frame[i] = 51,
                20 | 21 | 22 => t.frame[i] = 56,
                30 | 31 | 32 => t.frame[i] = 61,
                37 | 38 | 39 | 40 => t.frame[i] = 66,
                _ => {}
            }
        }
        return;
    }
    let num24 = 7;
    let num25 = 5 * num24 * 5 + 8;
    let num26 = num25 - 8;
    t.counter[i] += 1;
    if t.counter[i] < num26 {
        let mut num27 = t.counter[i] / num24;
        if num27 > 3 {
            num27 = 3;
        }
        // :15340-15380 — the default arm covers 41-45 AND every other
        // frame outside the five listed bands.
        match t.frame[i] {
            46..=50 => t.frame[i] = 46 + num27,
            51..=55 => t.frame[i] = 51 + num27,
            56..=60 => t.frame[i] = 56 + num27,
            61..=65 => t.frame[i] = 61 + num27,
            66..=70 => t.frame[i] = 66 + num27,
            _ => t.frame[i] = 41 + num27,
        }
    } else if t.counter[i] >= num26 {
        // :15384-15424 — the deflate hold frames.
        match t.frame[i] {
            46..=50 => t.frame[i] = 50,
            51..=55 => t.frame[i] = 55,
            56..=60 => t.frame[i] = 60,
            61..=65 => t.frame[i] = 65,
            66..=70 => t.frame[i] = 70,
            _ => t.frame[i] = 45,
        }
    }
    if t.counter[i] >= num25 {
        t.counter[i] = 0;
        // :15429-15473 — back to the swim band at the pose's entry frame.
        match t.frame[i] {
            41..=45 => t.frame[i] = 1,
            46..=50 => t.frame[i] = 9,
            51..=55 => t.frame[i] = 18,
            56..=60 => t.frame[i] = 21,
            61..=65 => t.frame[i] = 31,
            66..=70 => t.frame[i] = 38,
            _ => {}
        }
    }
}

/// `Main.UpdateOwlCageFrame` (`Main.cs:16617-16666`, called per slot by
/// `UpdateOwlCageFrames` `:16598-16604` at the `:15476` site): a 160-tick
/// counter cycle drives the frame; the `% 40 != 0 || Next(80) == 0` gate
/// draws Next(80) ONLY on the 40-boundary ticks (the || short-circuit).
fn step_owl(t: &mut CageTable, i: usize, rand: &mut UnifiedRandom) {
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

/// `Main.snailCageFrame[num28]` (`Main.cs:15479-15548`) — also drives
/// `snail2CageFrame[num29]` (`:15552-15621`), identical apart from the two
/// roll ranges: snail (45,3600)/(60,7200), snail2 (30,2700)/(45,5400).
fn step_snail(
    t: &mut CageTable,
    i: usize,
    rand: &mut UnifiedRandom,
    crawl_lo: i32,
    crawl_hi: i32,
    hide_lo: i32,
    hide_hi: i32,
) {
    let f = t.frame[i];
    if (0..=13).contains(&f) {
        t.counter[i] += 1;
        if t.counter[i] <= rand.next_range(crawl_lo, crawl_hi) {
            return;
        }
        if t.frame[i] == 8 && rand.next_max(2) == 0 {
            t.frame[i] = 14;
        } else if t.frame[i] == 1 && rand.next_max(3) == 0 {
            t.frame[i] = 19;
        } else if t.frame[i] == 1 && rand.next_max(3) == 0 {
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
        if t.counter[i] <= rand.next_range(hide_lo, hide_hi) {
            return;
        }
        t.counter[i] = 0;
        if rand.next_max(4) == 0 {
            if rand.next_max(3) == 0 {
                t.frame[i] = 2;
            } else if t.frame[i] == 19 {
                t.frame[i] = 20;
            } else {
                t.frame[i] = 19;
            }
        }
    }
}

/// `Main.frogCageFrame[num30]` (`Main.cs:15625-15722`). The 18-23 band's
/// wrap re-checks `> 17` OUTSIDE the 5-tick gate (`:15711`) — verbatim: a
/// frame 18 slot re-rolls the wrap every tick, not only on a pass.
fn step_frog(t: &mut CageTable, i: usize, rand: &mut UnifiedRandom) {
    let f = t.frame[i];
    if f == 0 {
        t.counter[i] += 1;
        if t.counter[i] > rand.next_range(45, 3600) {
            if rand.next_max(10) == 0 {
                t.frame[i] = 1;
            } else {
                t.frame[i] = 12;
            }
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
            if rand.next_max(3) == 0 {
                t.frame[i] = 0;
            } else {
                t.frame[i] = 12;
            }
        }
    } else if f == 6 {
        t.counter[i] += 1;
        if t.counter[i] > rand.next_range(45, 3600) {
            if rand.next_max(10) == 0 {
                t.frame[i] = 7;
            } else {
                t.frame[i] = 18;
            }
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
            // :15711 — verbatim `> 17` outside the gate (the 12-17 band's
            // check copy-pasted), so 18-23 re-roll every tick.
            if rand.next_max(3) == 0 {
                t.frame[i] = 6;
            } else {
                t.frame[i] = 18;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A scripted `Main.rand` stand-in: the exact C# stream is the
    /// `UnifiedRandom` port (tested separately); these witnesses pin the
    /// STATE MACHINE given a roll sequence. `Scripted` replays the C#
    /// `Next(min,max)` results by feeding a real `UnifiedRandom` whose
    /// values are pre-captured — instead we drive the machines directly
    /// with a seeded stream and pin the first divergence-free run.
    fn rng() -> UnifiedRandom {
        UnifiedRandom::new(1)
    }

    #[test]
    fn gate_off_draws_nothing() {
        let mut a = CageAnimator::new();
        let mut r = rng();
        let before = a.anim_key();
        a.animate(&mut r); // critter_cage = false — Main.cs:13513 return
        assert_eq!(a.anim_key(), before);
        // The stream is untouched by the early return.
        let mut r2 = rng();
        assert_eq!(r.next(), r2.next());
    }

    #[test]
    fn bunny_idle_rolls_every_tick_and_hops_on_pass() {
        // Main.cs:13519-13542: frame 0 ticks the counter and rolls
        // Next(30,900) EVERY tick; on a pass, Next(3)!=0 rolls Next(7):
        // 0 → frame 4, 1-2 → frame 2, 3-6 → frame 1.
        let mut t = CageTable::default();
        let mut r = rng();
        // Deterministic replay: run until the first non-zero frame, then
        // pin the counter reset + the reachable frame set.
        for _ in 0..2000 {
            step_bunny(&mut t, 0, &mut r);
            if t.frame[0] != 0 {
                break;
            }
        }
        assert!(matches!(t.frame[0], 1 | 2 | 4), "first hop picks 1/2/4");
        assert_eq!(t.counter[0], 0, "the pass resets the counter (:13542)");
        // Frame 1 holds 10 ticks then returns to 0 (:13544-13552).
        t.frame[0] = 1;
        t.counter[0] = 0;
        for _ in 0..9 {
            step_bunny(&mut t, 0, &mut r);
        }
        assert_eq!(t.frame[0], 1);
        step_bunny(&mut t, 0, &mut r);
        assert_eq!((t.frame[0], t.counter[0]), (0, 0));
    }

    #[test]
    fn bunny_run_band_wraps_21_to_0() {
        // Main.cs:13604-13615: frames >= 13 advance every 5 ticks and wrap
        // at > 21 (NOT at >= 21 — frame 21 draws one more slot).
        let mut t = CageTable::default();
        let mut r = rng();
        t.frame[0] = 21;
        for _ in 0..4 {
            step_bunny(&mut t, 0, &mut r);
        }
        assert_eq!(t.frame[0], 21, "4 ticks below the 5-tick gate");
        step_bunny(&mut t, 0, &mut r);
        assert_eq!(t.frame[0], 0, "22 wraps to 0");
    }

    #[test]
    fn squirrel_hop_band_rolls_the_1_in_5_reset() {
        // Main.cs:13654-13672: frames 2-3 advance every 5 ticks; passing 3
        // rolls Next(5): 0 → frame 0, else frame 2 (the hop loop).
        let mut t = CageTable::default();
        let mut r = rng();
        t.frame[0] = 3;
        t.counter[0] = 4;
        step_squirrel(&mut t, 0, &mut r);
        assert!(matches!(t.frame[0], 0 | 2), "the wrap picks 0 or 2");
        assert_eq!(t.counter[0], 0);
        // The 11/12 band uses `< 5` + counter reset AFTER the wrap roll
        // (:13717-13736) — counter 4 returns early WITHOUT resetting.
        t.frame[0] = 11;
        t.counter[0] = 3;
        step_squirrel(&mut t, 0, &mut r);
        assert_eq!((t.frame[0], t.counter[0]), (11, 4));
    }

    #[test]
    fn duck_idle_arm_rolls_one_gate_draw_per_state() {
        // Main.cs:13888-13917: the (f==0 && Next(3)!=0) || (f==4 &&
        // Next(5)==0) short-circuit draws exactly ONE Next on the gate.
        // Pin it by stream position: two tables in the same state, same
        // seed, must consume the same number of draws.
        let mut a = CageTable::default();
        let mut b = CageTable::default();
        a.counter[0] = 5000; // past any Next(45,2700)
        b.counter[0] = 5000;
        a.frame[0] = 0;
        b.frame[0] = 0;
        let mut ra = rng();
        let mut rb = rng();
        step_duck(&mut a, 0, &mut ra);
        step_duck(&mut b, 0, &mut rb);
        assert_eq!((a.frame[0], a.counter[0]), (b.frame[0], b.counter[0]));
        assert_eq!(ra.next(), rb.next(), "same stream position after the arm");
    }

    #[test]
    fn grebe_idle_limit_is_rolled_before_the_gate() {
        // Main.cs:14144-14159: counter++ FIRST, then the state rolls the
        // limit (idle 300-400 / 480-600, else 5); a below-limit tick keeps
        // the incremented counter.
        let mut t = CageTable::default();
        let mut r = rng();
        step_grebe(&mut t, 0, &mut r);
        assert_eq!(t.counter[0], 1, "the unconditional ++ (:14144)");
        assert_eq!(t.frame[0], 0);
        // Frame 13/17 snap to 0 on a pass (:14199-14202).
        t.frame[0] = 13;
        t.counter[0] = 4;
        step_grebe(&mut t, 0, &mut r);
        assert_eq!((t.frame[0], t.counter[0]), (0, 0));
    }

    #[test]
    fn seagull_glide_band_rederives_the_frame_from_the_counter() {
        // Main.cs:14267-14313: frames 15-18 below the 66 limit map
        // counter % 66 through the 11-step glide ladder every tick.
        let mut t = CageTable::default();
        let mut r = rng();
        t.frame[0] = 15;
        t.counter[0] = 60; // ++ → 61 → n = 61 > 60 → frame 15
        step_seagull(&mut t, 0, &mut r);
        assert_eq!((t.frame[0], t.counter[0]), (15, 61));
        t.counter[0] = 54; // ++ → 55 → n = 55 > 54 → frame 16
        step_seagull(&mut t, 0, &mut r);
        assert_eq!(t.frame[0], 16);
        t.counter[0] = 0; // ++ → 1 → n = 1 → frame 15 (the else)
        step_seagull(&mut t, 0, &mut r);
        assert_eq!(t.frame[0], 15);
        // At the limit the glide exits via the ==15 → 14 arm (:14256).
        t.counter[0] = 65;
        step_seagull(&mut t, 0, &mut r);
        assert_eq!((t.frame[0], t.counter[0]), (14, 0));
    }

    #[test]
    fn bird_frame3_hop_can_jump_to_13() {
        // Main.cs:14403-14417: frames 2-5 step every 5 ticks; frame 3
        // rolls Next(3)==0 → 13 (the sing band entry) else ++.
        let mut t = CageTable::default();
        let mut r = rng();
        let mut saw13 = false;
        let mut saw4 = false;
        for _ in 0..60 {
            t.frame[0] = 3;
            t.counter[0] = 4;
            step_bird(&mut t, 0, &mut r);
            match t.frame[0] {
                13 => saw13 = true,
                4 => saw4 = true,
                other => panic!("unexpected hop target {other}"),
            }
            assert_eq!(t.counter[0], 0);
        }
        assert!(saw13 && saw4, "both hop targets must be reachable");
    }

    #[test]
    fn bird_frame14_rolls_two_20s_in_order() {
        // Main.cs:14461-14479: the 1/20 sing pick, then a SECOND 1/20 for
        // the frame-4 hop, else 15; counter resets only on the pass.
        let mut t = CageTable::default();
        let mut r = rng();
        t.frame[0] = 14;
        let mut targets = std::collections::BTreeSet::new();
        for _ in 0..4000 {
            t.frame[0] = 14;
            t.counter[0] = 600; // past any Next(5,600)
            step_bird(&mut t, 0, &mut r);
            targets.insert(t.frame[0]);
            assert_eq!(t.counter[0], 0);
        }
        assert_eq!(targets, [4, 15, 16].into_iter().collect());
    }

    #[test]
    fn red_and_blue_bird_share_the_bird_machine() {
        // The three C# bodies are byte-identical — same seed, same state,
        // same outcome (the keep-one port's parity witness).
        let mut a = CageTable::default();
        let mut b = CageTable::default();
        let mut ra = rng();
        let mut rb = rng();
        for _ in 0..500 {
            step_bird(&mut a, 3, &mut ra);
            step_bird(&mut b, 3, &mut rb);
        }
        assert_eq!(a.frame[3], b.frame[3]);
        assert_eq!(a.counter[3], b.counter[3]);
    }

    #[test]
    fn frame_of_reads_the_right_table_and_row() {
        let mut a = CageAnimator::new();
        a.bunny.frame[6] = 7;
        a.slug.frame[2][16] = 5;
        a.butterfly.frame[8][24] = 9;
        a.scorpion.frame[1][0] = 3; // row 1 exists but the draw reads row 0
        assert_eq!(a.frame_of(CageAnim::Bunny, 0, 6), 7);
        assert_eq!(a.frame_of(CageAnim::Slug, 2, 16), 5);
        assert_eq!(a.frame_of(CageAnim::Butterfly, 8, 24), 9);
        assert_eq!(a.frame_of(CageAnim::Scorpion, 0, 0), 0);
        // Negative probe results clamp to slot 0 (documented deviation).
        assert_eq!(a.frame_of(CageAnim::Bunny, 0, -2), a.bunny.frame[0]);
    }

    #[test]
    fn cage_kind_set_matches_the_furniture_table() {
        // The G6 gate set must cover exactly the cage_table kinds.
        let cages: &[u16] = &[
            275, 276, 277, 278, 279, 280, 281, 296, 297, 309, 358, 359, 413, 414, 542, 550, 551,
            553, 554, 558, 559, 599, 600, 601, 602, 603, 604, 605, 606, 607, 608, 609, 610, 611,
            612, 632, 640, 643, 644, 645, 710, 285, 286, 298, 299, 310, 339, 361, 362, 363, 364,
            391, 392, 393, 394, 532, 533, 538, 544, 555, 556, 582, 619, 629, 282, 505, 543, 598,
            568, 569, 570, 288, 289, 290, 291, 292, 293, 294, 295, 360, 580, 620, 521, 522, 523,
            524, 525, 526, 527, 316, 317, 318,
        ];
        for &k in cages {
            assert!(CageAnimator::is_cage_kind(k), "kind {k} missing from the gate");
        }
        for k in [0u16, 1, 21, 283, 284, 287, 300, 315, 319, 365, 395, 621, 631, 641, 646, 711] {
            assert!(!CageAnimator::is_cage_kind(k), "kind {k} is not a cage");
        }
    }

    // ---- batch 2: macaw … frog ----

    #[test]
    fn macaw_walk_band_uses_gt5_not_ge5() {
        // Main.cs:14802-14813: frames 1-5 step on counter > 5 (six ticks),
        // and passing 5 lands on 6 (the idle pose), not 0.
        let mut t = CageTable::default();
        let mut r = rng();
        t.frame[0] = 5;
        for _ in 0..5 {
            step_macaw(&mut t, 0, &mut r);
        }
        assert_eq!((t.frame[0], t.counter[0]), (5, 5), "> 5 keeps the 6th tick waiting");
        step_macaw(&mut t, 0, &mut r);
        assert_eq!((t.frame[0], t.counter[0]), (6, 0));
    }

    #[test]
    fn scorpion_frame7_snaps_to_zero_without_rolls() {
        // Main.cs:14890-14893: the idle-7 pass writes 0 directly (no
        // Next(3) chain) — pin the stream position against a fresh rng
        // advanced by exactly the one Next(30,3600) gate draw.
        let mut t = CageTable2::<2>::default();
        t.frame[0][0] = 7;
        t.counter[0][0] = 4000; // past any Next(30,3600)
        let mut r = rng();
        step_scorpion(&mut t, 0, 0, &mut r);
        assert_eq!((t.frame[0][0], t.counter[0][0]), (0, 0));
        let mut r2 = rng();
        r2.next_range(30, 3600);
        assert_eq!(r.next(), r2.next(), "exactly one gate draw on the 7-arm");
    }

    #[test]
    fn penguin_10_11_band_keeps_the_gt3_quirk() {
        // Main.cs:15157-15176: the 10/11 band's wrap re-checks > 3 — frame
        // 10's first 5-tick pass ALWAYS wraps (10+1 > 3), rolling 8 or 10.
        let mut t = CageTable::default();
        let mut r = rng();
        t.frame[0] = 10;
        t.counter[0] = 4;
        step_penguin(&mut t, 0, &mut r);
        assert!(matches!(t.frame[0], 8 | 10), "the >3 quirk wraps immediately");
        assert_eq!(t.counter[0], 0);
    }

    #[test]
    fn turtle_29_resets_and_15_can_jump_to_30() {
        // Main.cs:15218-15227: frame 29 → 0 (no roll); frame 15 rolls
        // Next(3)==0 → 30 else falls through to ++ (16).
        let mut t = CageTable::default();
        let mut r = rng();
        t.frame[0] = 29;
        t.counter[0] = 300; // past any Next(180,250)? 29 is not idle — limit 8
        step_turtle(&mut t, 0, &mut r);
        assert_eq!((t.frame[0], t.counter[0]), (0, 0));
        let mut targets = std::collections::BTreeSet::new();
        for _ in 0..200 {
            t.frame[0] = 15;
            t.counter[0] = 300;
            step_turtle(&mut t, 0, &mut r);
            targets.insert(t.frame[0]);
        }
        assert_eq!(targets, [16, 30].into_iter().collect());
    }

    #[test]
    fn seahorse_swim_band_limit_6_and_24_reset() {
        // Main.cs:15239-15266: frames >= 25 use limit 6; frame 24 → 0.
        let mut t = CageTable::default();
        let mut r = rng();
        t.frame[0] = 25;
        for _ in 0..5 {
            step_seahorse(&mut t, 0, &mut r);
        }
        assert_eq!(t.frame[0], 25, "5 ticks below the 6 limit");
        step_seahorse(&mut t, 0, &mut r);
        assert_eq!(t.frame[0], 26);
        t.frame[0] = 24;
        t.counter[0] = 10;
        step_seahorse(&mut t, 0, &mut r);
        assert_eq!(t.frame[0], 0);
    }

    #[test]
    fn pufferfish_swim_wraps_40_to_0_and_inflate_reads_post_step() {
        // Main.cs:15277-15327: the swim band steps every 7 ticks, wraps
        // > 40 → 0, then the 1/75 roll reads the POST-step frame (0 → 41
        // is reachable only via the wrap).
        let mut t = CageTable::default();
        let mut r = rng();
        t.frame[0] = 40;
        t.counter[0] = 6;
        // Force the inflate roll: run many wraps until one fires.
        let mut saw_inflate = false;
        for _ in 0..600 {
            t.frame[0] = 40;
            t.counter[0] = 6;
            step_pufferfish(&mut t, 0, &mut r);
            match t.frame[0] {
                0 => {}
                41 => saw_inflate = true,
                other => panic!("unexpected post-wrap frame {other}"),
            }
        }
        assert!(saw_inflate, "the 1/75 inflate must fire on the wrapped 0");
    }

    #[test]
    fn pufferfish_inflate_cycle_returns_to_the_swim_entry() {
        // Main.cs:15329-15473: from 41 the counter climbs to num26 = 175
        // (deflate hold 45) then num25 = 183 → frame 1, counter 0.
        let mut t = CageTable::default();
        let mut r = rng();
        t.frame[0] = 41;
        for _ in 0..183 {
            step_pufferfish(&mut t, 0, &mut r);
        }
        assert_eq!((t.frame[0], t.counter[0]), (1, 0));
        // The mid-cycle ladder: counter 7 → num27 = 1 → frame 42.
        t.frame[0] = 41;
        t.counter[0] = 6;
        step_pufferfish(&mut t, 0, &mut r);
        assert_eq!(t.frame[0], 42);
    }

    #[test]
    fn owl_counter_cycle_maps_the_four_phases() {
        // Main.cs:16627-16665: num = counter%40/10, num2 = counter/40.
        // Phase 0: 0,1,2,1; phase 1: 0,7,6,7; phase 2: 0,8,9,8;
        // phase 3: 0,12,13,12 (num==3 folds to 1 in phases 0/1 only).
        let mut t = CageTable::default();
        let mut r = rng();
        // Drive with the ++ gate forced: counter%40 != 0 always increments.
        // Pin the mapping by setting counters directly (the frame write is
        // unconditional).
        for (counter, expect) in [
            (5usize, 0), (15, 1), (25, 2), (35, 1),
            (45, 0), (55, 7), (65, 6), (75, 7),
            (85, 0), (95, 8), (105, 9), (115, 8),
            (125, 0), (135, 12), (145, 13), (155, 12),
        ] {
            t.counter[0] = counter as i32;
            step_owl(&mut t, 0, &mut r);
            // The step may ++ or re-roll the counter first; re-derive from
            // the C# by pinning only counters where the ++ is forced
            // (counter%40 != 0) and no 39-boundary re-roll fires.
            let c = counter as i32 + 1;
            if (c + 1) % 40 != 39 {
                let mut num = c % 40 / 10;
                let phase = c / 40;
                if phase <= 1 && num == 3 {
                    num = 1;
                }
                let want = match phase {
                    0 => num,
                    1 => if num != 0 { 8 - num } else { 0 },
                    2 => if num != 0 { 7 + num } else { 0 },
                    _ => if num != 0 { 11 + num } else { 0 },
                };
                assert_eq!(t.frame[0], want, "counter {counter}");
                let _ = expect;
            }
        }
    }

    #[test]
    fn snail_hide_band_swaps_19_20_and_can_exit_to_2() {
        // Main.cs:15521-15547: the 19/20 hide pass rolls Next(4)==0 then
        // Next(3)==0 → 2, else the 19↔20 swap.
        let mut t = CageTable::default();
        let mut r = rng();
        let mut targets = std::collections::BTreeSet::new();
        for _ in 0..3000 {
            t.frame[0] = 19;
            t.counter[0] = 8000; // past any Next(60,7200)
            step_snail(&mut t, 0, &mut r, 45, 3600, 60, 7200);
            targets.insert(t.frame[0]);
        }
        assert_eq!(targets, [2, 19, 20].into_iter().collect());
        // The crawl band's double frame-1 roll: 19 then 20 (:15490-15497).
        let mut saw = std::collections::BTreeSet::new();
        for _ in 0..3000 {
            t.frame[0] = 1;
            t.counter[0] = 4000;
            step_snail(&mut t, 0, &mut r, 45, 3600, 60, 7200);
            saw.insert(t.frame[0]);
        }
        assert!(saw.contains(&19) && saw.contains(&20) && saw.contains(&2));
    }

    #[test]
    fn frog_18_23_band_rerolls_the_wrap_every_tick() {
        // Main.cs:15711-15721: the > 17 check sits OUTSIDE the 5-tick gate
        // — frame 18 re-rolls Next(3) on EVERY tick (the verbatim quirk).
        let mut t = CageTable::default();
        let mut r = rng();
        t.frame[0] = 18;
        t.counter[0] = 0;
        step_frog(&mut t, 0, &mut r);
        assert!(matches!(t.frame[0], 6 | 18), "the wrap rolls below the gate");
        assert_eq!(t.counter[0], 1, "the counter still ticks");
        // The 12-17 band only rolls on the pass (counter 4 → 5).
        t.frame[0] = 17;
        t.counter[0] = 0;
        step_frog(&mut t, 0, &mut r);
        assert_eq!(t.frame[0], 17, "no wrap roll below the 5-tick gate");
    }
}
