//! The critter-cage animator — `Main.AnimateTiles_CritterCages`
//! (`Main.cs:13511-16596`) plus its two out-of-line helpers
//! `UpdateOwlCageFrames` (`:16598-16604` → `UpdateOwlCageFrame` `:16617-16666`)
//! and `UpdateDragonflyJarFrames` (`:16606-16615` → `UpdateDragonflyJarFrame`
//! `:16668-16727`).
//!
//! C# advances the separate `Main.*CageFrame[]` / `*CageFrameCounter[]`
//! entity arrays (NOT `Main.tileFrame[]`) once per client update, gated on
//! `Main.critterCage` (`:13513-13516`). The draw side reads them through
//! [`crate::drawdata_furniture::cage_draw_data`]'s `frame_of` lookup. This
//! module owns that state; it is the S1-F-9b deferred animator half that
//! `drawdata_anim.rs`'s module docs flag as "CritterCages still deferred".
//!
//! Determinism: every machine draws from the caller's [`UnifiedRandom`] —
//! the port's exact `Main.rand` Knuth subtractive stream — in C# source
//! order. The whole function is ported as ONE ordered sequence (the 34
//! table machines below run top-to-bottom exactly as `:13517-16595`), so a
//! later machine's draws land on the same stream position as vanilla. No
//! other RNG is touched; tile framing (`framing.rs`) is untouched.
//!
//! Wiring (W4-g34 union, research/67): the DRAW seam covers ALL 92 cage
//! ids (`drawdata_furniture::is_cage_kind` — the g33 42 + g34 50 union).
//! The ANIMATOR is ported whole because the machines share one RNG
//! stream; splitting it would desynchronise every downstream draw.
//!
//! The `Main.critterCage` proximity gate is fed by the caller (the tile
//! draw sets it when a cage cell is drawn, `TileDrawing.cs:5672` etc.; the
//! non-solid pass clears it at `:474`). The init warm-up
//! (`Initialize_TileAndNPCData2`, `Main.cs:6800-6807`: 3600 forced ticks)
//! is WIRED at App construction (research/67 §2.6/§4) via
//! [`CritterCageAnimator::warm_up`] on a FRESH world-seeded
//! `UnifiedRandom` clone (f14 semantics) — the live `Main.rand` stand-in
//! stream is untouched; the exact-stream replay of the ctor draws before
//! `:6610` (`Next(6)` `:6467` + `RandomizeBackgrounds` `:6594-6601`) is
//! the recorded residual (docs/ANALYSIS.md, research/67 §4). PROCESS
//! state: no world-entry reset (the `:8387-8395` zero loop covers
//! `tileFrame` only; the cage arrays live in `static Main()`).

use crate::drawdata_furniture::{CageAnim, CAGE_FRAMES};
use crate::unified_random::UnifiedRandom;

/// `Main.cageFrames` slot count as an array length (`Main.cs:66901`).
const N: usize = CAGE_FRAMES as usize;

/// One 1-D `Main.*CageFrame` / `*CageFrameCounter` pair
/// (`Main.cs:66902-66972`, all `new int[Main.cageFrames]`, zero-initialised).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CageTable {
    pub frame: [i32; N],
    pub counter: [i32; N],
}

/// The full `Main.*CageFrame` animator state. Field names follow the C#
/// fields (`Main.cs:66902-66972`); the 2-D tables carry the C# first index
/// (scorpion 2, slug 3, butterfly 9, dragonfly 7, jellyfish 3). The `mode`
/// arrays are the C# `byte` mode fields (`fishBowlFrameMode`,
/// `butterflyCageMode`, `jellyfishCageMode`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CritterCageAnimator {
    // ---- first half (g33 draw lane): big cages + snails ----
    pub bunny: CageTable,
    pub squirrel: CageTable,
    pub squirrel_orange: CageTable,
    pub mallard: CageTable,
    pub duck: CageTable,
    pub grebe: CageTable,
    pub seagull: CageTable,
    pub bird: CageTable,
    pub blue_bird: CageTable,
    pub red_bird: CageTable,
    pub macaw: CageTable,
    /// `Main.scorpionCageFrame[2, cageFrames]` (`:66929`).
    pub scorpion: [[i32; N]; 2],
    pub scorpion_counter: [[i32; N]; 2],
    pub penguin: CageTable,
    pub turtle: CageTable,
    pub seahorse: CageTable,
    pub pufferfish: CageTable,
    pub owl: CageTable,
    pub snail: CageTable,
    // ---- second half (g34 draw lane): small + water/jellyfish ----
    pub snail2: CageTable,
    pub frog: CageTable,
    pub mouse: CageTable,
    pub worm: CageTable,
    /// `Main.slugCageFrame[3, cageFrames]` (`:66965`).
    pub slug: [[i32; N]; 3],
    pub slug_counter: [[i32; N]; 3],
    pub grasshopper: CageTable,
    pub maggot: CageTable,
    pub ladybug: CageTable,
    pub rat: CageTable,
    pub water_strider: CageTable,
    pub fairy_jar: CageTable,
    pub fish_bowl: CageTable,
    /// `Main.fishBowlFrameMode` (`byte[cageFrames]`, `:66935`).
    pub fish_bowl_mode: [u8; N],
    pub lava_fish_bowl: CageTable,
    /// `Main.butterflyCageFrame[9, cageFrames]` (`:66925`).
    pub butterfly: [[i32; N]; 9],
    pub butterfly_counter: [[i32; N]; 9],
    /// `Main.butterflyCageMode` (`byte[9, cageFrames]`, `:66924`).
    pub butterfly_mode: [[u8; N]; 9],
    /// `Main.dragonflyJarFrame[7, cageFrames]` (`:66928`).
    pub dragonfly: [[i32; N]; 7],
    pub dragonfly_counter: [[i32; N]; 7],
    /// `Main.jellyfishCageFrame[3, cageFrames]` (`:66949`).
    pub jellyfish: [[i32; N]; 3],
    pub jellyfish_counter: [[i32; N]; 3],
    /// `Main.jellyfishCageMode` (`byte[3, cageFrames]`, `:66948`).
    pub jellyfish_mode: [[u8; N]; 3],
}

impl CritterCageAnimator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Back to the bare zero allocation (`Main.cs:66902-66972`). NOT a
    /// world-load hook — the cage tables are PROCESS state (research/67
    /// §2.5: the static-ctor allocation runs once per process; the C#
    /// world-load zero loop `:8387-8395` covers `tileFrame` only). Kept
    /// for tests that need the vanilla t=0 draw state.
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// `Main.AnimateTiles_CritterCages` (`Main.cs:13511-16596`): the whole
    /// ordered sequence, gated on `Main.critterCage` (`:13513-13516`).
    /// `critter_cage` is the proximity flag the tile draw sets; `rand` is
    /// `Main.rand`. The machines run in C# source order so the shared RNG
    /// stream stays aligned.
    pub fn animate(&mut self, critter_cage: bool, rand: &mut UnifiedRandom) {
        if !critter_cage {
            return; // :13513-13516
        }
        // :13517-13617 — bunny.
        step_bunny(&mut self.bunny, rand);
        // :13618-13751 — squirrel.
        step_squirrel(&mut self.squirrel, rand);
        // :13752-13885 — squirrelOrange (the same machine, orange variant).
        step_squirrel(&mut self.squirrel_orange, rand);
        // :13886-14013 — mallard.
        step_mallard_duck(&mut self.mallard, rand);
        // :14014-14141 — duck (byte-identical to mallard).
        step_mallard_duck(&mut self.duck, rand);
        // :14142-14211 — grebe.
        step_grebe(&mut self.grebe, rand);
        // :14212-14371 — seagull.
        step_seagull(&mut self.seagull, rand);
        // :14372-14507 — bird.
        step_bird(&mut self.bird, rand);
        // :14508-14643 — blueBird (byte-identical to bird).
        step_bird(&mut self.blue_bird, rand);
        // :14644-14779 — redBird (byte-identical to bird).
        step_bird(&mut self.red_bird, rand);
        // :14780-14878 — macaw.
        step_macaw(&mut self.macaw, rand);
        // :14879-15038 — scorpion (2 rows).
        for row in 0..2 {
            step_scorpion(&mut self.scorpion[row], &mut self.scorpion_counter[row], rand);
        }
        // :15039-15203 — penguin.
        step_penguin(&mut self.penguin, rand);
        // :15204-15233 — turtle.
        step_turtle(&mut self.turtle, rand);
        // :15234-15273 — seahorse.
        step_seahorse(&mut self.seahorse, rand);
        // :15274-15475 — pufferfish (num22 = 40).
        step_pufferfish(&mut self.pufferfish, rand);
        // :15476 — UpdateOwlCageFrames.
        for i in 0..N {
            step_owl(&mut self.owl.frame, &mut self.owl.counter, i, rand);
        }
        // :15477-15549 — snail.
        step_snail(&mut self.snail, rand);
        // :15550-15622 — snail2.
        step_snail2(&mut self.snail2, rand);
        // :15623-15723 — frog.
        step_frog(&mut self.frog, rand);
        // :15724-15829 — mouse.
        step_mouse(&mut self.mouse, rand);
        // :15830-15858 — worm.
        step_worm(&mut self.worm, rand);
        // :15859-15885 — slug (3 rows, per-row wrap 24/31/34).
        for row in 0..3 {
            let wrap = match row {
                0 => 24,
                1 => 31,
                _ => 34,
            };
            step_slug(&mut self.slug[row], &mut self.slug_counter[row], wrap, rand);
        }
        // :15886-15948 — grasshopper.
        step_grasshopper(&mut self.grasshopper, rand);
        // :15949-15985 — maggot.
        step_maggot(&mut self.maggot, rand);
        // :15986-16049 — ladybug.
        step_ladybug(&mut self.ladybug, rand);
        // :16050-16156 — rat.
        step_rat(&mut self.rat, rand);
        // :16157-16196 — waterStrider.
        step_water_strider(&mut self.water_strider, rand);
        // :16197-16227 — fairyJar (slot 0 ticks, slots 1.. derive; no RNG).
        step_fairy_jar(&mut self.fairy_jar);
        // :16228-16384 — fishBowl (mode byte).
        step_fish_bowl(&mut self.fish_bowl, &mut self.fish_bowl_mode, rand);
        // :16385-16397 — lavaFishBowl.
        step_lava_fish_bowl(&mut self.lava_fish_bowl, rand);
        // :16398-16525 — butterfly (9 rows, mode byte).
        for row in 0..9 {
            step_butterfly(
                &mut self.butterfly[row],
                &mut self.butterfly_counter[row],
                &mut self.butterfly_mode[row],
                rand,
            );
        }
        // :16526 — UpdateDragonflyJarFrames (7 rows).
        for row in 0..7 {
            step_dragonfly(&mut self.dragonfly[row], &mut self.dragonfly_counter[row], rand);
        }
        // :16527-16595 — jellyfish (3 rows, mode byte).
        for row in 0..3 {
            step_jellyfish(
                &mut self.jellyfish[row],
                &mut self.jellyfish_counter[row],
                &mut self.jellyfish_mode[row],
                rand,
            );
        }
    }

    /// `Initialize_TileAndNPCData2`'s warm-up (`Main.cs:6802-6807`): force
    /// `critterCage = true` and tick 3600 times so the cages start mid-cycle
    /// rather than all-zero. WIRED at App construction on a fresh
    /// world-seeded clone (research/67 §2.6/§4 — see module docs).
    pub fn warm_up(&mut self, rand: &mut UnifiedRandom) {
        for _ in 0..3600 {
            self.animate(true, rand);
        }
    }

    /// The `frame_of(table, row, idx)` lookup
    /// [`crate::drawdata_furniture::cage_draw_data`] consumes: reads the
    /// animator's `Main.*CageFrame` value for a cage kind's table pick.
    /// `idx` is the `Get*AnimalCageFrame` probe result (0..cageFrames for
    /// validly-framed tiles); an out-of-range probe reads 0 (the C# would
    /// throw — it cannot happen for placed tiles, whose anchor fold keeps
    /// `x - fx/18 >= 0`).
    pub fn frame_of(&self, table: CageAnim, row: i32, idx: i32) -> i32 {
        if idx < 0 || idx >= CAGE_FRAMES {
            return 0;
        }
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
            CageAnim::Scorpion => self.scorpion[r][i],
            CageAnim::Penguin => self.penguin.frame[i],
            CageAnim::Seahorse => self.seahorse.frame[i],
            CageAnim::Pufferfish => self.pufferfish.frame[i],
            CageAnim::Snail => self.snail.frame[i],
            CageAnim::Snail2 => self.snail2.frame[i],
            CageAnim::Frog => self.frog.frame[i],
            CageAnim::Grasshopper => self.grasshopper.frame[i],
            CageAnim::Mouse => self.mouse.frame[i],
            CageAnim::Worm => self.worm.frame[i],
            CageAnim::Slug => self.slug[r][i],
            CageAnim::Maggot => self.maggot.frame[i],
            CageAnim::Rat => self.rat.frame[i],
            CageAnim::Ladybug => self.ladybug.frame[i],
            CageAnim::WaterStrider => self.water_strider.frame[i],
            CageAnim::FishBowl => self.fish_bowl.frame[i],
            CageAnim::LavaFishBowl => self.lava_fish_bowl.frame[i],
            CageAnim::FairyJar => self.fairy_jar.frame[i],
            CageAnim::Butterfly => self.butterfly[r][i],
            CageAnim::Dragonfly => self.dragonfly[r][i],
            CageAnim::Jellyfish => self.jellyfish[r][i],
        }
    }

    /// G6 key input: a hash of every animator value that rides a cage
    /// cell's pixels (the `frame_of` lookups). FNV-1a over the raw `i32`
    /// little-endian bytes in field order — deterministic, order-sensitive,
    /// no RNG (the `TileAnimator::anim_key` convention). The mode bytes ride
    /// too: they gate future frame writes, so two states differing only in
    /// mode must not share a cached range.
    pub fn anim_key(&self) -> u64 {
        let mut h = 0xcbf2_9ce4_8422_2325u64;
        let mut feed = |v: i32| {
            for b in v.to_le_bytes() {
                h ^= b as u64;
                h = h.wrapping_mul(0x0000_0100_0000_01b3);
            }
        };
        let mut table = |t: &CageTable| {
            for &v in &t.frame {
                feed(v);
            }
            for &v in &t.counter {
                feed(v);
            }
        };
        table(&self.bunny);
        table(&self.squirrel);
        table(&self.squirrel_orange);
        table(&self.mallard);
        table(&self.duck);
        table(&self.grebe);
        table(&self.seagull);
        table(&self.bird);
        table(&self.blue_bird);
        table(&self.red_bird);
        table(&self.macaw);
        for row in 0..2 {
            for &v in &self.scorpion[row] {
                feed(v);
            }
            for &v in &self.scorpion_counter[row] {
                feed(v);
            }
        }
        table(&self.penguin);
        table(&self.turtle);
        table(&self.seahorse);
        table(&self.pufferfish);
        table(&self.owl);
        table(&self.snail);
        table(&self.snail2);
        table(&self.frog);
        table(&self.mouse);
        table(&self.worm);
        for row in 0..3 {
            for &v in &self.slug[row] {
                feed(v);
            }
            for &v in &self.slug_counter[row] {
                feed(v);
            }
        }
        table(&self.grasshopper);
        table(&self.maggot);
        table(&self.ladybug);
        table(&self.rat);
        table(&self.water_strider);
        table(&self.fairy_jar);
        table(&self.fish_bowl);
        for &m in &self.fish_bowl_mode {
            feed(m as i32);
        }
        table(&self.lava_fish_bowl);
        for row in 0..9 {
            for &v in &self.butterfly[row] {
                feed(v);
            }
            for &v in &self.butterfly_counter[row] {
                feed(v);
            }
            for &m in &self.butterfly_mode[row] {
                feed(m as i32);
            }
        }
        for row in 0..7 {
            for &v in &self.dragonfly[row] {
                feed(v);
            }
            for &v in &self.dragonfly_counter[row] {
                feed(v);
            }
        }
        for row in 0..3 {
            for &v in &self.jellyfish[row] {
                feed(v);
            }
            for &v in &self.jellyfish_counter[row] {
                feed(v);
            }
            for &m in &self.jellyfish_mode[row] {
                feed(m as i32);
            }
        }
        h
    }
}

// ---- first-half machines (g33 draw lane) ----

/// `Main.bunnyCageFrame` loop (`Main.cs:13517-13617`).
fn step_bunny(t: &mut CageTable, rand: &mut UnifiedRandom) {
    for i in 0..N {
        if t.frame[i] == 0 {
            t.counter[i] += 1;
            if t.counter[i] <= rand.next_range(30, 900) {
                continue;
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
        } else if t.frame[i] == 1 {
            t.counter[i] += 1;
            if t.counter[i] >= 10 {
                t.counter[i] = 0;
                t.frame[i] = 0;
            }
        } else if (2..=3).contains(&t.frame[i]) {
            t.counter[i] += 1;
            if t.counter[i] >= 10 {
                t.counter[i] = 0;
                t.frame[i] += 1;
            }
            if t.frame[i] > 3 {
                t.frame[i] = 0;
            }
        } else if (4..=10).contains(&t.frame[i]) {
            t.counter[i] += 1;
            if t.counter[i] >= 5 {
                t.counter[i] = 0;
                t.frame[i] += 1;
            }
        } else if t.frame[i] == 11 {
            t.counter[i] += 1;
            if t.counter[i] <= rand.next_range(30, 900) {
                continue;
            }
            if rand.next_max(3) != 0 {
                if rand.next_max(7) == 0 {
                    t.frame[i] = 13;
                } else {
                    t.frame[i] = 12;
                }
            }
            t.counter[i] = 0;
        } else if t.frame[i] == 12 {
            t.counter[i] += 1;
            if t.counter[i] >= 10 {
                t.counter[i] = 0;
                t.frame[i] = 11;
            }
        } else if t.frame[i] >= 13 {
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
}

/// `Main.squirrelCageFrame` loop (`Main.cs:13618-13751`); the orange variant
/// (`:13752-13885`) is byte-identical and reuses this machine.
fn step_squirrel(t: &mut CageTable, rand: &mut UnifiedRandom) {
    for i in 0..N {
        if t.frame[i] == 0 {
            t.counter[i] += 1;
            if t.counter[i] <= rand.next_range(30, 900) {
                continue;
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
        } else if t.frame[i] == 1 {
            t.counter[i] += 1;
            if t.counter[i] >= 10 {
                t.counter[i] = 0;
                t.frame[i] = 0;
            }
        } else if (2..=3).contains(&t.frame[i]) {
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
        } else if (4..=8).contains(&t.frame[i]) {
            t.counter[i] += 1;
            if t.counter[i] >= 5 {
                t.counter[i] = 0;
                t.frame[i] += 1;
            }
        } else if t.frame[i] == 9 {
            t.counter[i] += 1;
            if t.counter[i] <= rand.next_range(30, 900) {
                continue;
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
        } else if t.frame[i] == 10 {
            t.counter[i] += 1;
            if t.counter[i] >= 10 {
                t.counter[i] = 0;
                t.frame[i] = 9;
            }
        } else if t.frame[i] == 11 || t.frame[i] == 12 {
            t.counter[i] += 1;
            if t.counter[i] < 5 {
                continue;
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
        } else if t.frame[i] >= 13 {
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
}

/// `Main.mallardCageFrame` loop (`Main.cs:13886-14013`); duck
/// (`:14014-14141`) is byte-identical and reuses this machine.
fn step_mallard_duck(t: &mut CageTable, rand: &mut UnifiedRandom) {
    for i in 0..N {
        if t.frame[i] == 0 || t.frame[i] == 4 {
            t.counter[i] += 1;
            if t.counter[i] <= rand.next_range(45, 2700) {
                continue;
            }
            if (t.frame[i] == 0 && rand.next_max(3) != 0)
                || (t.frame[i] == 4 && rand.next_max(5) == 0)
            {
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
        } else if (1..=3).contains(&t.frame[i]) {
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
        } else if (5..=11).contains(&t.frame[i]) {
            t.counter[i] += 1;
            if t.counter[i] >= 5 {
                t.counter[i] = 0;
                t.frame[i] += 1;
            }
        } else if t.frame[i] == 12 || t.frame[i] == 16 {
            t.counter[i] += 1;
            if t.counter[i] <= rand.next_range(45, 2700) {
                continue;
            }
            if (t.frame[i] == 12 && rand.next_max(3) != 0)
                || (t.frame[i] == 16 && rand.next_max(5) == 0)
            {
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
        } else if (13..=15).contains(&t.frame[i]) {
            t.counter[i] += 1;
            if t.counter[i] < 5 {
                continue;
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
        } else if t.frame[i] >= 17 {
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
}

/// `Main.grebeCageFrame` loop (`Main.cs:14142-14211`).
fn step_grebe(t: &mut CageTable, rand: &mut UnifiedRandom) {
    for i in 0..N {
        t.counter[i] += 1;
        let flag = t.frame[i] == 0 || t.frame[i] == 7;
        let num2 = t.frame[i] == 16 || t.frame[i] == 20;
        let mut num3 = 5;
        if flag {
            num3 = rand.next_range(300, 400);
        }
        if num2 {
            num3 = rand.next_range(480, 600);
        }
        if t.counter[i] < num3 {
            continue;
        }
        t.counter[i] = 0;
        if (25..=27).contains(&t.frame[i]) {
            t.frame[i] += 1;
            if t.frame[i] > 27 {
                if rand.next_max(5) == 0 {
                    t.frame[i] = 7;
                } else {
                    t.frame[i] = 25;
                }
            }
        } else if (22..=24).contains(&t.frame[i]) {
            t.frame[i] += 1;
            if t.frame[i] > 24 {
                if rand.next_max(5) == 0 {
                    t.frame[i] = 0;
                } else {
                    t.frame[i] = 22;
                }
            }
        } else if t.frame[i] == 0 && rand.next_max(3) == 0 {
            t.frame[i] = if rand.next_max(2) == 0 { 22 } else { 14 };
        } else if t.frame[i] == 7 && rand.next_max(3) == 0 {
            t.frame[i] = if rand.next_max(2) == 0 { 25 } else { 18 };
        } else if t.frame[i] == 13 || t.frame[i] == 17 {
            t.frame[i] = 0;
        } else if t.frame[i] == 21 {
            t.frame[i] = 7;
        } else {
            t.frame[i] += 1;
        }
    }
}

/// `Main.seagullCageFrame` loop (`Main.cs:14212-14371`).
fn step_seagull(t: &mut CageTable, rand: &mut UnifiedRandom) {
    for i in 0..N {
        t.counter[i] += 1;
        let flag2 = t.frame[i] == 0 || t.frame[i] == 14;
        let flag3 = (15..=18).contains(&t.frame[i]);
        let num5 = (9..=13).contains(&t.frame[i]);
        let mut num6 = 6;
        if flag2 {
            num6 = rand.next_range(180, 250);
        }
        if flag3 {
            num6 = 66;
        }
        if num5 {
            num6 = 78;
        }
        if t.counter[i] >= num6 {
            t.counter[i] = 0;
            if t.frame[i] == 0 && rand.next_max(3) == 0 {
                t.frame[i] = 9;
                continue;
            }
            if t.frame[i] == 14 {
                if rand.next_max(2) == 0 {
                    t.frame[i] = 15;
                } else {
                    t.frame[i] = 19;
                }
                continue;
            }
            if t.frame[i] == 9 {
                t.frame[i] = 0;
                continue;
            }
            if t.frame[i] == 8 || t.frame[i] == 15 {
                t.frame[i] = 14;
                continue;
            }
            t.frame[i] += 1;
            if t.frame[i] > 26 {
                t.frame[i] = 0;
            }
        } else if (15..=18).contains(&t.frame[i]) {
            let num7 = t.counter[i] % 66;
            if num7 > 60 {
                t.frame[i] = 15;
            } else if num7 > 54 {
                t.frame[i] = 16;
            } else if num7 > 48 {
                t.frame[i] = 17;
            } else if num7 > 42 {
                t.frame[i] = 18;
            } else if num7 > 36 {
                t.frame[i] = 17;
            } else if num7 > 30 {
                t.frame[i] = 16;
            } else if num7 > 24 {
                t.frame[i] = 17;
            } else if num7 > 18 {
                t.frame[i] = 18;
            } else if num7 > 12 {
                t.frame[i] = 17;
            } else if num7 > 6 {
                t.frame[i] = 16;
            } else {
                t.frame[i] = 15;
            }
        } else if (9..=13).contains(&t.frame[i]) {
            let num8 = t.counter[i] % 78;
            if num8 > 72 {
                t.frame[i] = 9;
            } else if num8 > 66 {
                t.frame[i] = 10;
            } else if num8 > 60 {
                t.frame[i] = 11;
            } else if num8 > 54 {
                t.frame[i] = 12;
            } else if num8 > 48 {
                t.frame[i] = 13;
            } else if num8 > 42 {
                t.frame[i] = 12;
            } else if num8 > 36 {
                t.frame[i] = 11;
            } else if num8 > 30 {
                t.frame[i] = 12;
            } else if num8 > 24 {
                t.frame[i] = 13;
            } else if num8 > 18 {
                t.frame[i] = 12;
            } else if num8 > 12 {
                t.frame[i] = 11;
            } else if num8 > 6 {
                t.frame[i] = 10;
            } else {
                t.frame[i] = 9;
            }
        }
    }
}

/// `Main.birdCageFrame` loop (`Main.cs:14372-14507`); blueBird
/// (`:14508-14643`) and redBird (`:14644-14779`) are byte-identical.
fn step_bird(t: &mut CageTable, rand: &mut UnifiedRandom) {
    for i in 0..N {
        if t.frame[i] == 0 {
            t.counter[i] += 1;
            if t.counter[i] <= rand.next_range(30, 2700) {
                continue;
            }
            if rand.next_max(3) != 0 {
                if rand.next_max(3) != 0 {
                    t.frame[i] = 2;
                } else {
                    t.frame[i] = 1;
                }
            }
            t.counter[i] = 0;
        } else if t.frame[i] == 1 {
            t.counter[i] += 1;
            if t.counter[i] > rand.next_range(900, 18000) && rand.next_max(3) == 0 {
                t.counter[i] = 0;
                t.frame[i] = 0;
            }
        } else if (2..=5).contains(&t.frame[i]) {
            t.counter[i] += 1;
            if t.counter[i] >= 5 {
                t.counter[i] = 0;
                if t.frame[i] == 3 && rand.next_max(3) == 0 {
                    t.frame[i] = 13;
                } else {
                    t.frame[i] += 1;
                }
            }
        } else if t.frame[i] == 6 {
            t.counter[i] += 1;
            if t.counter[i] <= rand.next_range(45, 2700) {
                continue;
            }
            if rand.next_max(3) != 0 {
                if rand.next_max(6) == 0 {
                    t.frame[i] = 7;
                } else if rand.next_max(6) == 0 {
                    t.frame[i] = 11;
                }
            }
            t.counter[i] = 0;
        } else if (7..=10).contains(&t.frame[i]) {
            t.counter[i] += 1;
            if t.counter[i] >= 5 {
                t.frame[i] += 1;
                if t.frame[i] > 10 {
                    t.frame[i] = 0;
                }
                t.counter[i] = 0;
            }
        } else if (11..=13).contains(&t.frame[i]) {
            t.counter[i] += 1;
            if t.counter[i] >= 5 {
                t.frame[i] += 1;
                t.counter[i] = 0;
            }
        } else if t.frame[i] == 14 {
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
        } else if t.frame[i] == 15 {
            t.counter[i] += 1;
            if t.counter[i] >= 10 {
                t.counter[i] = 0;
                t.frame[i] = 14;
            }
        } else {
            if !(16..=18).contains(&t.frame[i]) {
                continue;
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
}

/// `Main.macawCageFrame` loop (`Main.cs:14780-14878`).
fn step_macaw(t: &mut CageTable, rand: &mut UnifiedRandom) {
    for i in 0..N {
        if t.frame[i] == 0 {
            t.counter[i] += 1;
            if t.counter[i] <= rand.next_range(300, 2700) {
                continue;
            }
            if rand.next_max(3) != 0 {
                if rand.next_max(3) != 0 {
                    t.frame[i] = 16;
                } else {
                    t.frame[i] = 1;
                }
            }
            t.counter[i] = 0;
        } else if (1..=5).contains(&t.frame[i]) {
            t.counter[i] += 1;
            if t.counter[i] > 5 {
                t.counter[i] = 0;
                t.frame[i] += 1;
                if t.frame[i] > 5 {
                    t.frame[i] = 6;
                }
            }
        } else if (16..=19).contains(&t.frame[i]) {
            t.counter[i] += 1;
            if t.counter[i] > 6 {
                t.counter[i] = 0;
                t.frame[i] += 1;
                if t.frame[i] > 19 {
                    t.frame[i] = if rand.next_max(3) != 0 { 16 } else { 0 };
                }
            }
        } else if t.frame[i] == 6 {
            t.counter[i] += 1;
            if t.counter[i] <= rand.next_range(300, 2700) {
                continue;
            }
            if rand.next_max(3) != 0 {
                if rand.next_max(3) != 0 {
                    t.frame[i] = 7;
                } else {
                    t.frame[i] = 11;
                }
            }
            t.counter[i] = 0;
        } else if (11..=15).contains(&t.frame[i]) {
            t.counter[i] += 1;
            if t.counter[i] > 5 {
                t.counter[i] = 0;
                t.frame[i] += 1;
                if t.frame[i] > 15 {
                    t.frame[i] = 0;
                }
            }
        } else {
            if !(7..=10).contains(&t.frame[i]) {
                continue;
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
}

/// `Main.scorpionCageFrame[row, ..]` inner loop (`Main.cs:14881-15037`).
fn step_scorpion(frame: &mut [i32; N], counter: &mut [i32; N], rand: &mut UnifiedRandom) {
    for i in 0..N {
        if frame[i] == 0 || frame[i] == 7 {
            counter[i] += 1;
            if counter[i] <= rand.next_range(30, 3600) {
                continue;
            }
            if frame[i] == 7 {
                frame[i] = 0;
            } else if rand.next_max(3) == 0 {
                if rand.next_max(7) == 0 {
                    frame[i] = 1;
                } else if rand.next_max(4) == 0 {
                    frame[i] = 8;
                } else if rand.next_max(3) == 0 {
                    frame[i] = 7;
                } else {
                    frame[i] = 14;
                }
            }
            counter[i] = 0;
        } else if (1..=2).contains(&frame[i]) {
            counter[i] += 1;
            if counter[i] >= 10 {
                counter[i] = 0;
                frame[i] += 1;
            }
        } else if (8..=10).contains(&frame[i]) {
            counter[i] += 1;
            if counter[i] >= 10 {
                counter[i] = 0;
                frame[i] += 1;
            }
        } else if frame[i] == 11 {
            counter[i] += 1;
            if counter[i] > rand.next_range(45, 5400) {
                if rand.next_max(6) == 0 {
                    frame[i] = 12;
                }
                counter[i] = 0;
            }
        } else if (12..=13).contains(&frame[i]) {
            counter[i] += 1;
            if counter[i] >= 10 {
                counter[i] = 0;
                frame[i] += 1;
                if frame[i] > 13 {
                    frame[i] = 0;
                }
            }
        } else if (14..=15).contains(&frame[i]) {
            counter[i] += 1;
            if counter[i] >= 5 {
                counter[i] = 0;
                frame[i] += 1;
                if frame[i] > 15 {
                    frame[i] = 14;
                }
                if rand.next_max(5) == 0 {
                    frame[i] = 0;
                }
            }
        } else if frame[i] == 4 || frame[i] == 3 {
            counter[i] += 1;
            if counter[i] <= rand.next_range(30, 3600) {
                continue;
            }
            if frame[i] == 3 {
                frame[i] = 4;
            } else if rand.next_max(3) == 0 {
                if rand.next_max(5) == 0 {
                    frame[i] = 5;
                } else if rand.next_max(3) == 0 {
                    frame[i] = 3;
                } else {
                    frame[i] = 16;
                }
            }
            counter[i] = 0;
        } else if (5..=6).contains(&frame[i]) {
            counter[i] += 1;
            if counter[i] >= 10 {
                counter[i] = 0;
                frame[i] += 1;
                if frame[i] > 7 {
                    frame[i] = 0;
                }
            }
        } else {
            if !(16..=17).contains(&frame[i]) {
                continue;
            }
            counter[i] += 1;
            if counter[i] >= 5 {
                counter[i] = 0;
                frame[i] += 1;
                if frame[i] > 17 {
                    frame[i] = 16;
                }
                if rand.next_max(5) == 0 {
                    frame[i] = 4;
                }
            }
        }
    }
}

/// `Main.penguinCageFrame` loop (`Main.cs:15039-15203`).
fn step_penguin(t: &mut CageTable, rand: &mut UnifiedRandom) {
    for i in 0..N {
        if t.frame[i] == 0 {
            t.counter[i] += 1;
            if t.counter[i] <= rand.next_range(30, 1800) {
                continue;
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
        } else if t.frame[i] == 1 {
            t.counter[i] += 1;
            if t.counter[i] >= 10 {
                t.counter[i] = 0;
                t.frame[i] = 0;
            }
        } else if (2..=3).contains(&t.frame[i]) {
            t.counter[i] += 1;
            if t.counter[i] < 5 {
                continue;
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
        } else if (4..=6).contains(&t.frame[i]) {
            t.counter[i] += 1;
            if t.counter[i] >= 10 {
                t.counter[i] = 0;
                t.frame[i] += 1;
            }
        } else if t.frame[i] == 15 {
            t.counter[i] += 1;
            if t.counter[i] > rand.next_range(10, 1800) {
                if rand.next_max(2) == 0 {
                    t.frame[i] = 0;
                }
                t.counter[i] = 0;
            }
        } else if t.frame[i] == 8 {
            t.counter[i] += 1;
            if t.counter[i] <= rand.next_range(30, 3600) {
                continue;
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
        } else if t.frame[i] == 9 {
            t.counter[i] += 1;
            if t.counter[i] >= 10 {
                t.counter[i] = 0;
                t.frame[i] = 8;
            }
        } else if (10..=11).contains(&t.frame[i]) {
            t.counter[i] += 1;
            if t.counter[i] < 5 {
                continue;
            }
            t.counter[i] = 0;
            t.frame[i] += 1;
            // :15166 — the C# compares `> 3` here (not `> 11`); faithful.
            if t.frame[i] > 3 {
                if rand.next_max(3) == 0 {
                    t.frame[i] = 8;
                } else {
                    t.frame[i] = 10;
                }
            }
        } else if (12..=14).contains(&t.frame[i]) {
            t.counter[i] += 1;
            if t.counter[i] >= 10 {
                t.counter[i] = 0;
                t.frame[i] += 1;
            }
        } else {
            if t.frame[i] != 7 {
                continue;
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
}

/// `Main.turtleCageFrame` loop (`Main.cs:15204-15233`).
fn step_turtle(t: &mut CageTable, rand: &mut UnifiedRandom) {
    for i in 0..N {
        t.counter[i] += 1;
        let num17 = t.frame[i] == 0 || t.frame[i] == 15 || t.frame[i] == 31;
        let mut num18 = 8;
        if num17 {
            num18 = rand.next_range(180, 250);
        }
        if t.counter[i] < num18 {
            continue;
        }
        t.counter[i] = 0;
        if t.frame[i] == 29 {
            t.frame[i] = 0;
            continue;
        }
        if t.frame[i] == 15 && rand.next_max(3) == 0 {
            t.frame[i] = 30;
            continue;
        }
        t.frame[i] += 1;
        if t.frame[i] > 32 {
            t.frame[i] = 15;
        }
    }
}

/// `Main.seahorseCageFrame` loop (`Main.cs:15234-15273`).
fn step_seahorse(t: &mut CageTable, rand: &mut UnifiedRandom) {
    for i in 0..N {
        t.counter[i] += 1;
        let flag4 = t.frame[i] == 0 || t.frame[i] == 13;
        let num20 = t.frame[i] == 4 || t.frame[i] == 9 || t.frame[i] == 17 || t.frame[i] == 22;
        let flag5 = t.frame[i] >= 25;
        let mut num21 = 7;
        if flag4 {
            num21 = rand.next_range(220, 340);
        }
        if num20 {
            num21 = 10;
        }
        if flag5 {
            num21 = 6;
        }
        if t.counter[i] < num21 {
            continue;
        }
        t.counter[i] = 0;
        if t.frame[i] == 0 && rand.next_max(2) == 0 {
            t.frame[i] = 25;
            continue;
        }
        if t.frame[i] == 24 {
            t.frame[i] = 0;
            continue;
        }
        t.frame[i] += 1;
        if t.frame[i] > 33 {
            t.frame[i] = 0;
        }
    }
}

/// `Main.pufferfishCageFrame` loop (`Main.cs:15274-15475`), `num22 = 40`.
fn step_pufferfish(t: &mut CageTable, rand: &mut UnifiedRandom) {
    let num22 = 40;
    for i in 0..N {
        if (0..=num22).contains(&t.frame[i]) {
            t.counter[i] += 1;
            if t.counter[i] < 7 {
                continue;
            }
            t.counter[i] = 0;
            t.frame[i] += 1;
            if t.frame[i] > num22 {
                t.frame[i] = 0;
            }
            if rand.next_max(75) == 0 {
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
            continue;
        }
        let num24 = 7;
        let num25 = 5 * num24 * 5 + 8; // 133
        let num26 = num25 - 8; // 125
        t.counter[i] += 1;
        if t.counter[i] < num26 {
            let mut num27 = t.counter[i] / num24;
            if num27 > 3 {
                num27 = 3;
            }
            match t.frame[i] {
                46..=50 => t.frame[i] = 46 + num27,
                51..=55 => t.frame[i] = 51 + num27,
                56..=60 => t.frame[i] = 56 + num27,
                61..=65 => t.frame[i] = 61 + num27,
                66..=70 => t.frame[i] = 66 + num27,
                _ => t.frame[i] = 41 + num27,
            }
        } else if t.counter[i] >= num26 {
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
}

/// `Main.UpdateOwlCageFrame` (`Main.cs:16617-16666`), called per slot by
/// `UpdateOwlCageFrames` (`:16598-16604`, invoked at `:15476`).
fn step_owl(frames: &mut [i32; N], counters: &mut [i32; N], style: usize, rand: &mut UnifiedRandom) {
    if counters[style] % 40 != 0 || rand.next_max(80) == 0 {
        counters[style] += 1;
    }
    if (counters[style] + 1) % 40 == 39 {
        counters[style] = 40 * rand.next_max(4);
    }
    let mut num = counters[style] % 40 / 10;
    let num2 = counters[style] / 40;
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
    frames[style] = num3;
}

/// `Main.snailCageFrame` loop (`Main.cs:15477-15549`).
fn step_snail(t: &mut CageTable, rand: &mut UnifiedRandom) {
    for i in 0..N {
        if (0..=13).contains(&t.frame[i]) {
            t.counter[i] += 1;
            if t.counter[i] <= rand.next_range(45, 3600) {
                continue;
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
        } else if (14..=18).contains(&t.frame[i]) {
            t.counter[i] += 1;
            if t.counter[i] >= 5 {
                t.counter[i] = 0;
                t.frame[i] += 1;
            }
            if t.frame[i] > 18 {
                t.frame[i] = 20;
            }
        } else {
            if t.frame[i] != 19 && t.frame[i] != 20 {
                continue;
            }
            t.counter[i] += 1;
            if t.counter[i] <= rand.next_range(60, 7200) {
                continue;
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
}

// ---- second-half machines (g34 draw lane) ----

/// `Main.snail2CageFrame` loop (`Main.cs:15550-15622`) — the snail machine
/// with the 30/2700 + 45/5400 cadences (snail uses 45/3600 + 60/7200).
fn step_snail2(t: &mut CageTable, rand: &mut UnifiedRandom) {
    for i in 0..N {
        if (0..=13).contains(&t.frame[i]) {
            t.counter[i] += 1;
            if t.counter[i] <= rand.next_range(30, 2700) {
                continue;
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
        } else if (14..=18).contains(&t.frame[i]) {
            t.counter[i] += 1;
            if t.counter[i] >= 5 {
                t.counter[i] = 0;
                t.frame[i] += 1;
            }
            if t.frame[i] > 18 {
                t.frame[i] = 20;
            }
        } else {
            if t.frame[i] != 19 && t.frame[i] != 20 {
                continue;
            }
            t.counter[i] += 1;
            if t.counter[i] <= rand.next_range(45, 5400) {
                continue;
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
}

/// `Main.frogCageFrame` loop (`Main.cs:15623-15723`).
fn step_frog(t: &mut CageTable, rand: &mut UnifiedRandom) {
    for i in 0..N {
        if t.frame[i] == 0 {
            t.counter[i] += 1;
            if t.counter[i] > rand.next_range(45, 3600) {
                if rand.next_max(10) == 0 {
                    t.frame[i] = 1;
                } else {
                    t.frame[i] = 12;
                }
                t.counter[i] = 0;
            }
        } else if (1..=5).contains(&t.frame[i]) {
            t.counter[i] += 1;
            if t.counter[i] >= 5 {
                t.frame[i] += 1;
                t.counter[i] = 0;
            }
        } else if (12..=17).contains(&t.frame[i]) {
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
        } else if t.frame[i] == 6 {
            t.counter[i] += 1;
            if t.counter[i] > rand.next_range(45, 3600) {
                if rand.next_max(10) == 0 {
                    t.frame[i] = 7;
                } else {
                    t.frame[i] = 18;
                }
                t.counter[i] = 0;
            }
        } else if (7..=11).contains(&t.frame[i]) {
            t.counter[i] += 1;
            if t.counter[i] >= 5 {
                t.frame[i] += 1;
                t.counter[i] = 0;
                if t.frame[i] > 11 {
                    t.frame[i] = 0;
                }
            }
        } else {
            if !(18..=23).contains(&t.frame[i]) {
                continue;
            }
            t.counter[i] += 1;
            if t.counter[i] >= 5 {
                t.counter[i] = 0;
                t.frame[i] += 1;
            }
            // :15711 — the C# compares `> 17` here (not `> 23`); faithful.
            if t.frame[i] > 17 {
                if rand.next_max(3) == 0 {
                    t.frame[i] = 6;
                } else {
                    t.frame[i] = 18;
                }
            }
        }
    }
}

/// `Main.mouseCageFrame` loop (`Main.cs:15724-15829`).
fn step_mouse(t: &mut CageTable, rand: &mut UnifiedRandom) {
    for i in 0..N {
        if (0..=1).contains(&t.frame[i]) {
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
        } else if (4..=7).contains(&t.frame[i]) {
            t.counter[i] += 1;
            if t.counter[i] >= 5 {
                t.counter[i] = 0;
                t.frame[i] += 1;
            }
            if t.frame[i] > 7 {
                t.frame[i] = 2;
            }
        } else if (2..=3).contains(&t.frame[i]) {
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
        } else if (8..=11).contains(&t.frame[i]) {
            t.counter[i] += 1;
            if t.counter[i] >= 5 {
                t.counter[i] = 0;
                t.frame[i] += 1;
            }
            if t.frame[i] > 11 {
                t.frame[i] = 0;
            }
        } else if (12..=13).contains(&t.frame[i]) {
            t.counter[i] += 1;
            if t.counter[i] >= 5 {
                t.counter[i] = 0;
                t.frame[i] += 1;
            }
        } else if (14..=17).contains(&t.frame[i]) {
            t.counter[i] += 1;
            if t.counter[i] >= 5 {
                t.counter[i] = 0;
                t.frame[i] += 1;
                if t.frame[i] > 17 && rand.next_max(20) != 0 {
                    t.frame[i] = 14;
                }
            }
        } else {
            if !(18..=19).contains(&t.frame[i]) {
                continue;
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
}

/// `Main.wormCageFrame` loop (`Main.cs:15830-15858`).
fn step_worm(t: &mut CageTable, rand: &mut UnifiedRandom) {
    for i in 0..N {
        t.counter[i] += 1;
        if t.counter[i] < rand.next_range(30, 91) {
            continue;
        }
        t.counter[i] = 0;
        if rand.next_max(4) != 0 {
            continue;
        }
        t.frame[i] += 1;
        if t.frame[i] == 9 && rand.next_max(2) == 0 {
            t.frame[i] = 0;
        }
        if t.frame[i] > 18 {
            if rand.next_max(2) == 0 {
                t.frame[i] = 9;
            } else {
                t.frame[i] = 0;
            }
        }
    }
}

/// `Main.slugCageFrame[row, ..]` inner loop (`Main.cs:15874-15884`); the
/// outer per-row wrap pick is 24/31/34 (`:15859-15873`).
fn step_slug(frame: &mut [i32; N], counter: &mut [i32; N], wrap: i32, rand: &mut UnifiedRandom) {
    for i in 0..N {
        counter[i] += 1;
        if counter[i] >= rand.next_range(5, 15) {
            counter[i] = 0;
            frame[i] += 1;
            if frame[i] >= wrap {
                frame[i] = 0;
            }
        }
    }
}

/// `Main.grasshopperCageFrame` loop (`Main.cs:15886-15948`).
fn step_grasshopper(t: &mut CageTable, rand: &mut UnifiedRandom) {
    for i in 0..N {
        if (0..=1).contains(&t.frame[i]) {
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
        } else if (2..=5).contains(&t.frame[i]) {
            t.counter[i] += 1;
            if t.counter[i] >= 5 {
                t.counter[i] = 0;
                t.frame[i] += 1;
            }
            if t.frame[i] > 5 {
                t.frame[i] = 6;
            }
        } else if (6..=7).contains(&t.frame[i]) {
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
        } else if (8..=11).contains(&t.frame[i]) {
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
}

/// `Main.maggotCageFrame` loop (`Main.cs:15949-15985`).
fn step_maggot(t: &mut CageTable, rand: &mut UnifiedRandom) {
    for i in 0..N {
        t.counter[i] += 1;
        let num38 = t.frame[i] == 0 || t.frame[i] == 6 || t.frame[i] == 18;
        let mut num39 = 8;
        if num38 {
            num39 = rand.next_range(100, 140);
        }
        if t.counter[i] < num39 {
            continue;
        }
        t.counter[i] = 0;
        if t.frame[i] == 0 {
            if rand.next_max(3) == 0 {
                t.frame[i] = 13;
            } else {
                t.frame[i] = 1;
            }
            continue;
        }
        if t.frame[i] == 12 {
            t.frame[i] = 0;
            continue;
        }
        t.frame[i] += 1;
        if t.frame[i] > 23 {
            t.frame[i] = 6;
        }
    }
}

/// `Main.ladybugCageFrame` loop (`Main.cs:15986-16049`).
fn step_ladybug(t: &mut CageTable, rand: &mut UnifiedRandom) {
    for i in 0..N {
        t.counter[i] += 1;
        let flag6 = t.frame[i] == 3 || t.frame[i] == 7 || t.frame[i] >= 17;
        let mut num41 = rand.next_range(10, 12);
        if t.frame[i] == 18 {
            num41 = rand.next_range(160, 241);
        } else if flag6 {
            num41 = rand.next_range(198, 206);
        } else if (8..=16).contains(&t.frame[i]) {
            num41 = 5;
        }
        if t.counter[i] < num41 {
            continue;
        }
        t.counter[i] = 0;
        if t.frame[i] < 18 {
            if (t.frame[i] == 2 || t.frame[i] == 5) && rand.next_max(3) == 0 {
                t.frame[i] = 17;
            } else if t.frame[i] == 3 || t.frame[i] == 12 {
                match rand.next_max(3) {
                    0 | 1 => t.frame[i] = 4,
                    _ => t.frame[i] = 13,
                }
            } else if t.frame[i] == 7 || t.frame[i] == 16 {
                match rand.next_max(3) {
                    0 | 1 => t.frame[i] = 0,
                    _ => t.frame[i] = 8,
                }
            } else {
                t.frame[i] += 1;
            }
        } else {
            t.frame[i] = if rand.next_max(2) == 0 { 13 } else { 4 };
        }
    }
}

/// `Main.ratCageFrame` loop (`Main.cs:16050-16156`).
fn step_rat(t: &mut CageTable, rand: &mut UnifiedRandom) {
    for i in 0..N {
        t.counter[i] += 1;
        let flag7 = t.frame[i] == 0 || t.frame[i] == 4;
        let num43 = t.frame[i] == 8 || t.frame[i] == 9 || t.frame[i] == 10 || t.frame[i] == 11;
        let flag8 = t.frame[i] > 11;
        let mut num44 = 5;
        if flag7 {
            num44 = rand.next_range(40, 70);
        }
        if num43 {
            num44 = rand.next_range(70, 110);
        }
        if flag8 {
            num44 = 180;
        }
        if t.counter[i] >= num44 {
            t.counter[i] = 0;
            if t.frame[i] == 0 && rand.next_max(2) == 0 {
                if rand.next_max(3) != 0 {
                    t.frame[i] = 12;
                } else {
                    t.frame[i] = 8;
                }
                continue;
            }
            if t.frame[i] == 4 && rand.next_max(2) == 0 {
                t.frame[i] = 10;
                continue;
            }
            if t.frame[i] == 7 || t.frame[i] == 9 || t.frame[i] == 12 {
                t.frame[i] = 0;
                continue;
            }
            if t.frame[i] == 11 {
                t.frame[i] = 4;
                continue;
            }
            t.frame[i] += 1;
            if t.frame[i] > 16 {
                t.frame[i] = 0;
            }
        } else if t.frame[i] > 11 {
            let num45 = t.counter[i] % 90;
            if num45 > 80 {
                t.frame[i] = 12;
            } else if num45 > 70 {
                t.frame[i] = 13;
            } else if num45 > 60 {
                t.frame[i] = 14;
            } else if num45 > 50 {
                t.frame[i] = 15;
            } else if num45 > 40 {
                t.frame[i] = 16;
            } else if num45 > 30 {
                t.frame[i] = 15;
            } else if num45 > 20 {
                t.frame[i] = 14;
            } else if num45 > 10 {
                t.frame[i] = 13;
            } else {
                t.frame[i] = 12;
            }
        } else if t.counter[i] % 10 == 0 {
            if t.frame[i] == 8 || t.frame[i] == 10 {
                t.frame[i] += 1;
            } else if t.frame[i] == 9 || t.frame[i] == 11 {
                t.frame[i] -= 1;
            }
        }
    }
}

/// `Main.waterStriderCageFrame` loop (`Main.cs:16157-16196`).
fn step_water_strider(t: &mut CageTable, rand: &mut UnifiedRandom) {
    for i in 0..N {
        t.counter[i] += 1;
        let flag9 = t.frame[i] == 0 || t.frame[i] == 8;
        let num47 = t.frame[i] == 17 || t.frame[i] == 20;
        let mut num48 = 5;
        if flag9 {
            num48 = rand.next_range(110, 210);
        }
        if num47 {
            num48 = rand.next_range(150, 260);
        }
        if t.counter[i] < num48 {
            continue;
        }
        t.counter[i] = 0;
        if t.frame[i] == 0 && rand.next_max(2) == 0 {
            t.frame[i] = 16;
            continue;
        }
        if t.frame[i] == 8 && rand.next_max(2) == 0 {
            t.frame[i] = 19;
            continue;
        }
        if t.frame[i] == 15 || t.frame[i] == 18 {
            t.frame[i] = 0;
            continue;
        }
        t.frame[i] += 1;
        if t.frame[i] > 21 {
            t.frame[i] = 8;
        }
    }
}

/// `Main.fairyJarFrame` block (`Main.cs:16197-16227`): slot 0 ticks on a
/// 6-tick counter through 12 rows; slots 1.. derive from slot 0 with the
/// parity offsets — NO RNG is drawn here.
fn step_fairy_jar(t: &mut CageTable) {
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

/// `Main.fishBowlFrame` loop (`Main.cs:16228-16384`) — the mode byte
/// (`Main.fishBowlFrameMode`) picks the cadence family; `maxValue = 5`.
fn step_fish_bowl(t: &mut CageTable, mode: &mut [u8; N], rand: &mut UnifiedRandom) {
    for i in 0..N {
        let max_value = 5;
        if mode[i] == 1 {
            if rand.next_max(900) == 0 {
                mode[i] = rand.next_max(max_value) as u8;
            }
            t.counter[i] += 1;
            if t.counter[i] < 5 {
                continue;
            }
            t.counter[i] = 0;
            if t.frame[i] == 10 {
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
                continue;
            }
            t.counter[i] = 0;
            if t.frame[i] == 10 {
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
                continue;
            }
            t.counter[i] = 0;
            if t.frame[i] == 10 {
                if rand.next_max(20) == 0 {
                    t.frame[i] = 11;
                    mode[i] = 0;
                } else {
                    t.frame[i] = 1;
                }
            } else {
                t.frame[i] += 1;
            }
        } else if t.frame[i] <= 10 {
            if rand.next_max(3600) == 0 {
                mode[i] = rand.next_max(max_value) as u8;
            }
            t.counter[i] += 1;
            if t.counter[i] < 10 {
                continue;
            }
            t.counter[i] = 0;
            if t.frame[i] == 10 {
                // :16332 — the mode-0 arm rolls Next(12), NOT Next(20),
                // and does NOT clear the mode on the 11 jump.
                if rand.next_max(12) == 0 {
                    t.frame[i] = 11;
                } else {
                    t.frame[i] = 1;
                }
            } else {
                t.frame[i] += 1;
            }
        } else if t.frame[i] == 12 || t.frame[i] == 13 {
            t.counter[i] += 1;
            if t.counter[i] < 10 {
                continue;
            }
            t.counter[i] = 0;
            t.frame[i] += 1;
            if t.frame[i] > 13 {
                if rand.next_max(20) == 0 {
                    t.frame[i] = 14;
                } else {
                    t.frame[i] = 12;
                }
            }
        } else {
            if t.frame[i] < 11 {
                continue;
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
}

/// `Main.lavaFishBowlFrame` loop (`Main.cs:16385-16397`).
fn step_lava_fish_bowl(t: &mut CageTable, rand: &mut UnifiedRandom) {
    for i in 0..N {
        t.counter[i] += 1;
        if t.counter[i] > 4 + rand.next_max(3) {
            t.counter[i] = 0;
            t.frame[i] += 1;
            if t.frame[i] > 26 {
                t.frame[i] = 0;
            }
        }
    }
}

/// `Main.butterflyCageFrame[row, ..]` inner loop (`Main.cs:16400-16524`) —
/// the mode byte's >= 10 half walks the rows BACKWARD (the jarred-variant
/// mirror), the < 10 half forward; both hop ±8 rows on the landmark frames.
fn step_butterfly(
    frame: &mut [i32; N],
    counter: &mut [i32; N],
    mode: &mut [u8; N],
    rand: &mut UnifiedRandom,
) {
    for i in 0..N {
        counter[i] += 1;
        if rand.next_max(3600) == 0 {
            mode[i] = rand.next_max(5) as u8;
            if rand.next_max(2) == 0 {
                mode[i] += 10;
            }
        }
        let mut num55 = rand.next_range(3, 16);
        if mode[i] == 1 || mode[i] == 11 {
            num55 = 3;
        }
        if mode[i] == 2 || mode[i] == 12 {
            num55 = 5;
        }
        if mode[i] == 3 || mode[i] == 13 {
            num55 = 10;
        }
        if mode[i] == 4 || mode[i] == 14 {
            num55 = 15;
        }
        if mode[i] >= 10 {
            if frame[i] <= 7 {
                if counter[i] < num55 {
                    continue;
                }
                counter[i] = 0;
                frame[i] -= 1;
                if frame[i] < 0 {
                    frame[i] = 7;
                }
                if frame[i] != 1 && frame[i] != 4 && frame[i] != 6 {
                    continue;
                }
                if rand.next_max(20) == 0 {
                    frame[i] += 8;
                } else if rand.next_max(6) == 0 {
                    if mode[i] >= 10 {
                        mode[i] -= 10;
                    } else {
                        mode[i] += 10;
                    }
                }
            } else {
                if counter[i] < num55 {
                    continue;
                }
                counter[i] = 0;
                frame[i] -= 1;
                if frame[i] < 8 {
                    frame[i] = 14;
                }
                if frame[i] != 9 && frame[i] != 12 && frame[i] != 14 {
                    continue;
                }
                if rand.next_max(20) == 0 {
                    frame[i] -= 8;
                } else if rand.next_max(6) == 0 {
                    if mode[i] >= 10 {
                        mode[i] -= 10;
                    } else {
                        mode[i] += 10;
                    }
                }
            }
        } else if frame[i] <= 7 {
            if counter[i] >= num55 {
                counter[i] = 0;
                frame[i] += 1;
                if frame[i] > 7 {
                    frame[i] = 0;
                }
                if (frame[i] == 1 || frame[i] == 4 || frame[i] == 6) && rand.next_max(10) == 0 {
                    frame[i] += 8;
                }
            }
        } else if counter[i] >= num55 {
            counter[i] = 0;
            frame[i] += 1;
            if frame[i] > 15 {
                frame[i] = 8;
            }
            if (frame[i] == 9 || frame[i] == 12 || frame[i] == 14) && rand.next_max(10) == 0 {
                frame[i] -= 8;
            }
        }
    }
}

/// `Main.UpdateDragonflyJarFrame` (`Main.cs:16668-16727`), called per
/// (style, variation) by `UpdateDragonflyJarFrames` (`:16606-16615`,
/// invoked at `:16526`).
fn step_dragonfly(frame: &mut [i32; N], counter: &mut [i32; N], rand: &mut UnifiedRandom) {
    for i in 0..N {
        counter[i] += 1;
        match frame[i] {
            0 => {
                if counter[i] >= 300 && rand.next_max(60) == 0 {
                    counter[i] = 0;
                    frame[i] = rand.next_from_slice(&[1, 5, 9]);
                }
            }
            1 | 5 | 9 => {
                if counter[i] >= 100 {
                    counter[i] = 0;
                    if frame[i] == 1 {
                        frame[i] = rand.next_from_slice(&[0, 5, 5, 9, 9]);
                    } else {
                        frame[i] = 1;
                    }
                } else if counter[i] >= 6 {
                    counter[i] = 0;
                    frame[i] += 1;
                }
            }
            2 | 3 | 4 | 6 | 7 | 8 | 10 | 11 | 12 => {
                let num = frame[i] - 1;
                let num2 = num % 4 - 1;
                let mut num3 = counter[i] / 4 % 4;
                if num3 > 2 {
                    num3 = 4 - num3;
                }
                frame[i] = num - num2 + 1 + num3;
                if counter[i] >= 40 && rand.next_max(30) == 0 {
                    counter[i] = 100;
                    frame[i] = num - num2 + 1 - 1;
                }
            }
            _ => {}
        }
    }
}

/// `Main.jellyfishCageFrame[row, ..]` inner loop (`Main.cs:16529-16594`) —
/// the mode byte walks 0 → 1 → 2 → 3 → 0 with per-mode row bands.
fn step_jellyfish(
    frame: &mut [i32; N],
    counter: &mut [i32; N],
    mode: &mut [u8; N],
    rand: &mut UnifiedRandom,
) {
    for i in 0..N {
        counter[i] += 1;
        if mode[i] == 0 && rand.next_max(1800) == 0 {
            mode[i] = 1;
        }
        if mode[i] == 2 && rand.next_max(60) == 0 {
            mode[i] = 3;
        }
        let mut num58 = 1;
        if mode[i] == 0 {
            num58 = rand.next_range(10, 20);
        }
        if mode[i] == 1 {
            num58 = rand.next_range(15, 25);
        }
        if mode[i] == 2 {
            num58 = rand.next_range(4, 9);
        }
        if mode[i] == 3 {
            num58 = rand.next_range(15, 25);
        }
        if mode[i] == 0 && frame[i] <= 3 && counter[i] >= num58 {
            counter[i] = 0;
            frame[i] += 1;
            if frame[i] >= 4 {
                frame[i] = 0;
            }
        }
        if mode[i] == 1 && frame[i] <= 7 && counter[i] >= num58 {
            counter[i] = 0;
            frame[i] += 1;
            if frame[i] >= 7 {
                mode[i] = 2;
            }
        }
        if mode[i] == 2 && frame[i] <= 9 && counter[i] >= num58 {
            counter[i] = 0;
            frame[i] += 1;
            if frame[i] >= 9 {
                frame[i] = 7;
            }
        }
        if mode[i] == 3 && frame[i] <= 10 && counter[i] >= num58 {
            counter[i] = 0;
            frame[i] += 1;
            if frame[i] >= 10 {
                frame[i] = 3;
                mode[i] = 0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// W4-g34 witness (WRITTEN NOT RUN — code-only mode): the fresh state
    /// is the vanilla allocation — every table zeroed (`Main.cs:66902-66972`
    /// allocates `new int[...]`, zero-initialised), so a t=0 draw through
    /// `frame_of` reproduces the static cage arms byte-for-byte.
    #[test]
    fn fresh_state_is_the_vanilla_zero_allocation() {
        let a = CritterCageAnimator::new();
        assert_eq!(a, CritterCageAnimator::default());
        for table in [
            CageAnim::Bunny, CageAnim::Snail, CageAnim::Snail2, CageAnim::Frog,
            CageAnim::Mouse, CageAnim::Worm, CageAnim::Slug, CageAnim::Grasshopper,
            CageAnim::Maggot, CageAnim::Ladybug, CageAnim::Rat, CageAnim::WaterStrider,
            CageAnim::FishBowl, CageAnim::LavaFishBowl, CageAnim::FairyJar,
            CageAnim::Butterfly, CageAnim::Dragonfly, CageAnim::Jellyfish,
        ] {
            for row in 0..9 {
                for idx in 0..CAGE_FRAMES {
                    assert_eq!(a.frame_of(table, row, idx), 0, "{table:?} {row} {idx}");
                }
            }
        }
    }

    /// W4-g34 witness (WRITTEN NOT RUN): the `critterCage` gate
    /// (`Main.cs:13513-13516`) — a false flag draws NO RNG and changes no
    /// state; the same seed's stream position is untouched.
    #[test]
    fn the_critter_cage_gate_skips_every_draw() {
        let mut a = CritterCageAnimator::new();
        let mut r = UnifiedRandom::new(7);
        let mut probe = UnifiedRandom::new(7);
        for _ in 0..50 {
            a.animate(false, &mut r);
        }
        assert_eq!(a, CritterCageAnimator::new(), "gated ticks change nothing");
        assert_eq!(r.next(), probe.next(), "gated ticks draw no RNG");
    }

    /// W4-g34 witness (WRITTEN NOT RUN): determinism — two same-seeded
    /// animators ticked identically stay bit-equal, and `anim_key` tracks
    /// the state (the G6 coverage convention).
    #[test]
    fn same_seed_replays_bit_equal_and_the_key_tracks_state() {
        let mut a = CritterCageAnimator::new();
        let mut b = CritterCageAnimator::new();
        let mut ra = UnifiedRandom::new(42);
        let mut rb = UnifiedRandom::new(42);
        let fresh_key = a.anim_key();
        for _ in 0..400 {
            a.animate(true, &mut ra);
            b.animate(true, &mut rb);
            assert_eq!(a, b);
        }
        assert_ne!(fresh_key, a.anim_key(), "400 ticks move the tables");
        assert_eq!(a.anim_key(), b.anim_key());
        a.reset();
        assert_eq!(fresh_key, a.anim_key(), "reset restores the fresh key");
    }

    /// W4-g34 witness (WRITTEN NOT RUN): the fairy-jar derivation
    /// (`Main.cs:16197-16227`) — slot 0 cycles 0..11 on a >5 counter, slots
    /// 1.. derive from slot 0 with the parity offsets, NO RNG. Hand-derived
    /// from the C# for the first ticks (counter hits 6 on tick 6).
    #[test]
    fn fairy_jar_slot0_cycles_and_slots_derive_without_rng() {
        let mut t = CageTable::default();
        let mut r = UnifiedRandom::new(1);
        let mut probe = UnifiedRandom::new(1);
        // Ticks 1-5: counter climbs, frame stays 0.
        for _ in 0..5 {
            step_fairy_jar(&mut t);
        }
        assert_eq!((t.frame[0], t.counter[0]), (0, 5));
        // Tick 6: counter 6 > 5 -> frame 1, counter 0.
        step_fairy_jar(&mut t);
        assert_eq!((t.frame[0], t.counter[0]), (1, 0));
        // Slot derivation at frame[0] = 1 (i = 1..24):
        // num50 = 1 + i%12, +1 when i%2==0, +1 when i%5==0, +1 when i%8==0,
        // wrapped below 12.
        for i in 1..N {
            let mut want = 1 + (i as i32) % 12;
            if i % 2 == 0 { want += 1; }
            if i % 5 == 0 { want += 1; }
            if i % 8 == 0 { want += 1; }
            while want >= 12 { want -= 12; }
            assert_eq!(t.frame[i], want, "slot {i}");
        }
        // The whole block draws no RNG.
        assert_eq!(r.next(), probe.next());
        // 12 full cycles (72 ticks) wrap frame[0] back to 0.
        for _ in 0..66 {
            step_fairy_jar(&mut t);
        }
        assert_eq!(t.frame[0], 0);
    }

    /// W4-g34 witness (WRITTEN NOT RUN): the lava fish bowl
    /// (`Main.cs:16385-16397`) — counter > 4 + Next(3) advances the frame
    /// through 27 rows (0..26); the frame only steps +1 or wraps 26 -> 0.
    #[test]
    fn lava_fish_bowl_wraps_at_26() {
        let mut t = CageTable::default();
        let mut r = UnifiedRandom::new(11);
        let mut seen_wrap = false;
        let mut prev = 0;
        for _ in 0..2000 {
            step_lava_fish_bowl(&mut t);
            let f = t.frame[0];
            assert!((0..=26).contains(&f), "frame {f} out of band");
            if prev == 26 && f == 0 {
                seen_wrap = true;
            }
            assert!(f == prev || f == prev + 1 || (prev == 26 && f == 0));
            prev = f;
        }
        assert!(seen_wrap, "2000 ticks must cross the 26 -> 0 wrap");
    }

    /// W4-g34 witness (WRITTEN NOT RUN): the slug per-row wraps
    /// (`Main.cs:15859-15884`) — row 0 wraps at 24, row 1 at 31, row 2 at 34;
    /// the counter cadence is `>= Next(5, 15)` per tick.
    #[test]
    fn slug_rows_wrap_at_24_31_34() {
        let mut a = CritterCageAnimator::new();
        let mut r = UnifiedRandom::new(5);
        let mut max = [0i32; 3];
        for _ in 0..4000 {
            a.animate(true, &mut r);
            for row in 0..3 {
                for i in 0..N {
                    let f = a.slug[row][i];
                    let wrap = [24, 31, 34][row];
                    assert!((0..wrap).contains(&f), "slug row {row} frame {f}");
                    max[row] = max[row].max(f);
                }
            }
        }
        // 4000 ticks at a 5-15-tick cadence cross every row's full cycle.
        assert_eq!(max, [23, 30, 33], "each row reaches its last frame");
    }

    /// W4-g34 witness (WRITTEN NOT RUN): the jellyfish mode machine
    /// (`Main.cs:16529-16594`) — mode stays in 0..3, frames stay in 0..10.
    #[test]
    fn jellyfish_mode_and_frame_bands_hold() {
        let mut a = CritterCageAnimator::new();
        let mut r = UnifiedRandom::new(23);
        for _ in 0..6000 {
            a.animate(true, &mut r);
            for row in 0..3 {
                for i in 0..N {
                    assert!(a.jellyfish_mode[row][i] <= 3);
                    assert!((0..=10).contains(&a.jellyfish[row][i]));
                }
            }
        }
    }

    /// W4-g34 witness (WRITTEN NOT RUN): the butterfly mode byte
    /// (`Main.cs:16400-16524`) — mode stays in 0..14 (the +10 mirror), frames
    /// stay in 0..15.
    #[test]
    fn butterfly_mode_mirror_and_frame_band_hold() {
        let mut a = CritterCageAnimator::new();
        let mut r = UnifiedRandom::new(31);
        for _ in 0..6000 {
            a.animate(true, &mut r);
            for row in 0..9 {
                for i in 0..N {
                    assert!(a.butterfly_mode[row][i] <= 14);
                    assert!((0..=15).contains(&a.butterfly[row][i]));
                }
            }
        }
    }

    /// W4-g34 witness (WRITTEN NOT RUN): the dragonfly idle/hop machine
    /// (`Main.cs:16668-16727`) — frames stay in 0..12.
    #[test]
    fn dragonfly_frames_stay_in_band() {
        let mut a = CritterCageAnimator::new();
        let mut r = UnifiedRandom::new(17);
        for _ in 0..6000 {
            a.animate(true, &mut r);
            for row in 0..7 {
                for i in 0..N {
                    assert!((0..=12).contains(&a.dragonfly[row][i]));
                }
            }
        }
    }

    /// W4-g34 witness (WRITTEN NOT RUN): the fish-bowl mode byte
    /// (`Main.cs:16228-16384`) — mode stays in 0..4 (`Next(5)`), frames stay
    /// in 0..16.
    #[test]
    fn fish_bowl_mode_and_frame_bands_hold() {
        let mut a = CritterCageAnimator::new();
        let mut r = UnifiedRandom::new(13);
        for _ in 0..6000 {
            a.animate(true, &mut r);
            for i in 0..N {
                assert!(a.fish_bowl_mode[i] <= 4);
                assert!((0..=16).contains(&a.fish_bowl.frame[i]));
            }
        }
    }

    /// W4-g34 witness (WRITTEN NOT RUN): `frame_of` reads the right table —
    /// a forced value in each second-half table is read back through the
    /// `CageAnim` pick the draw seam uses (the d64 batch 2-3 table map).
    #[test]
    fn frame_of_reads_the_second_half_tables() {
        let mut a = CritterCageAnimator::new();
        a.snail2.frame[3] = 7;
        a.frog.frame[3] = 8;
        a.mouse.frame[3] = 9;
        a.worm.frame[3] = 10;
        a.slug[1][3] = 11;
        a.grasshopper.frame[3] = 12;
        a.maggot.frame[3] = 13;
        a.ladybug.frame[3] = 14;
        a.rat.frame[3] = 15;
        a.water_strider.frame[3] = 16;
        a.fairy_jar.frame[3] = 17;
        a.fish_bowl.frame[3] = 18;
        a.lava_fish_bowl.frame[3] = 19;
        a.butterfly[8][3] = 20;
        a.dragonfly[6][3] = 21;
        a.jellyfish[2][3] = 22;
        assert_eq!(a.frame_of(CageAnim::Snail2, 0, 3), 7);
        assert_eq!(a.frame_of(CageAnim::Frog, 0, 3), 8);
        assert_eq!(a.frame_of(CageAnim::Mouse, 0, 3), 9);
        assert_eq!(a.frame_of(CageAnim::Worm, 0, 3), 10);
        assert_eq!(a.frame_of(CageAnim::Slug, 1, 3), 11);
        assert_eq!(a.frame_of(CageAnim::Grasshopper, 0, 3), 12);
        assert_eq!(a.frame_of(CageAnim::Maggot, 0, 3), 13);
        assert_eq!(a.frame_of(CageAnim::Ladybug, 0, 3), 14);
        assert_eq!(a.frame_of(CageAnim::Rat, 0, 3), 15);
        assert_eq!(a.frame_of(CageAnim::WaterStrider, 0, 3), 16);
        assert_eq!(a.frame_of(CageAnim::FairyJar, 0, 3), 17);
        assert_eq!(a.frame_of(CageAnim::FishBowl, 0, 3), 18);
        assert_eq!(a.frame_of(CageAnim::LavaFishBowl, 0, 3), 19);
        assert_eq!(a.frame_of(CageAnim::Butterfly, 8, 3), 20);
        assert_eq!(a.frame_of(CageAnim::Dragonfly, 6, 3), 21);
        assert_eq!(a.frame_of(CageAnim::Jellyfish, 2, 3), 22);
        // Out-of-range probes read 0 (the C# cannot index them for placed
        // tiles; the port guards rather than panics).
        assert_eq!(a.frame_of(CageAnim::Frog, 0, -1), 0);
        assert_eq!(a.frame_of(CageAnim::Frog, 0, CAGE_FRAMES), 0);
    }

    /// W4-g34 witness (WRITTEN NOT RUN): the warm-up
    /// (`Main.cs:6802-6807`) ticks exactly 3600 forced passes — the state
    /// after `warm_up` equals 3600 manual `animate(true, ..)` ticks on the
    /// same stream.
    #[test]
    fn warm_up_is_3600_forced_ticks() {
        let mut a = CritterCageAnimator::new();
        let mut b = CritterCageAnimator::new();
        let mut ra = UnifiedRandom::new(99);
        let mut rb = UnifiedRandom::new(99);
        a.warm_up(&mut ra);
        for _ in 0..3600 {
            b.animate(true, &mut rb);
        }
        assert_eq!(a, b);
        assert_eq!(ra.next(), rb.next(), "the streams stay aligned");
    }
}
