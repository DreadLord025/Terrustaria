//! W4-g64 (the d13 314fa6cd machine port, dust-wired) — `AI()`'s
//! `else if (this.aiStyle == 124)` arm (`Projectile.cs:34970-35696`): the
//! seven pet rows — the Suspicious Looking Tentacle light pet (650) and the
//! six flying pets (882 Eye of Cthulhu, 888 Twins, 895 Fairy Queen, 896
//! Pumpking, 898 Ice Queen, 957 Glommer). This branch rows 895/898/957 (the
//! trail-dust lane's kinds); the other four rows + the buff-spawn seam stay
//! d13's (the machine covers all seven so the merge keeps one).
//!
//! **RNG census (by construction).** The arm-level `Main.rand` draws, in the
//! C#'s order:
//! * 896 idle frame wrap (`:35431`): `Next(15)` — only when the wrap fires.
//! * 957 ground-bounce cycle end (`:35519`): `Next(150)` (the 200+ rest pin).
//! * 650 ore scan (`:35571`): the `Next(15)` gate; on a 0 and the pet within
//!   `Main.screenWidth` of the owner, ONE `Next(8)` per tile of the 35×35
//!   window (`:35587`, unconditional inside the loop — the `< 4` test reads
//!   the drawn value).
//! * 650 point-at-ore branch (`:35679`): `Next(10)` — only when `ai[0] > 0`
//!   and no NPC/velocity target won.
//! W4-g64 dust wiring (d13 skipped the bodies whole): the 895/898 flight
//! sparkles (`:35231`/`:35295`, `trail_895`/`trail_898` — zero explicit
//! draws, the pool internals ride the stream on the spawn), the 650 scan's
//! <2 sparkle (`:35601`, `trail_650_scan_sparkle`) and the point-at-ore 204
//! (`:35681`, `trail_650_point_sparkle`). The 957 arm draws NO dust — its
//! behaviour-coupled half is the `SolidTileAllowBottomSlope` column scan
//! (`:35483-35497`) gating the bounce cycle's `Next(150)` rest roll.
//!
//! Deviations (the standing pool precedents, ANALYSIS): the 650/895/896
//! `Utils.PlotTileLine` light casts (`:35008-35043`) have no per-frame port
//! layer; the 957's `SoundID.GlommerBounce` (`:35524`) is a TRACKABLE cue
//! deferred with the D3 trackable pool (the Item42 precedent);
//! `Main.screenWidth` rides the pool's `set_screen_width` seam (the
//! `set_disco` pattern). The pet flags ride `Player.pet_flags[kind]` (the
//! `buff_effects_accessory` granters: buff 190 → 650, the `PET_FLAG_ARMS`
//! table → the six flyers). `customData` is the pool's dropped field.

use glam::Vec2;
use tr_core::World;
use tr_core::loot::DropRng;
use tr_core::particles::{DustPool, SpawnCtx};

use crate::npc::Npc;
use crate::player::Player;
use crate::projectile::{
    EOC_PET, FAIRY_QUEEN_PET, GLOMMER_PET, ICE_QUEEN_PET, PUMPKING_PET, Projectile,
    SUSPICIOUS_TENTACLE, TWINS_PET, csign, npc_center, proj_frames,
};
use crate::projectile_ai026_c::solid_tile_allow_bottom_slope;

/// `Kill()` (`Projectile.cs:70644-70648`) — the sibling-module deactivation
/// (the `projectile_ai_spear` precedent; `Projectile::kill` is private).
fn kill(p: &mut Projectile) {
    p.kill_time_left = p.time_left;
    p.active = false;
    p.time_left = 0;
}

/// `Main.IsTileSpelunkable` (`Main.cs:21783-21798`): the `tileSpelunker`
/// table (`Main.cs:9420-9467` + `:9651`) plus the two frame-window arms —
/// the 185 statues' `frameY == 18 && frameX in 576..=882` and the 186
/// pressure plates' `frameX in 864..=1170`.
fn is_tile_spelunkable(kind: u16, frame_x: i16, frame_y: i16) -> bool {
    // The verbatim `Main.tileSpelunker[...] = true` id list.
    const SPELUNKER: &[u16] = &[
        6, 7, 8, 9, 12, 21, 467, 441, 468, 28, 107, 108, 111, 63, 64, 65, 66, 67, 68, 566, 166,
        167, 168, 169, 178, 211, 221, 222, 223, 236, 702, 37, 407, 227, 84, 83, 404, 242, 240,
        246, 245, 105, 349, 337, 531, 506, 751, 752, 639,
    ];
    if SPELUNKER.contains(&kind) {
        return true;
    }
    if kind == 185 && frame_y == 18 && (576..=882).contains(&frame_x) {
        return true;
    }
    if kind == 186 && (864..=1170).contains(&frame_x) {
        return true;
    }
    false
}

/// The flight machines' rotation blend (`:35128-35141` and its five twins):
/// the ±π wrap of the CURRENT rotation toward the target, then the
/// `(rot * (12 - 1) + target) / 12` one-twelfth step.
fn flight_rotation(rot: f32, target: f32) -> f32 {
    use std::f32::consts::{PI, TAU};
    let mut rot = rot;
    if (rot - target).abs() >= PI {
        if target < rot {
            rot -= TAU;
        } else {
            rot += TAU;
        }
    }
    (rot * (12.0 - 1.0) + target) / 12.0
}

/// The idle machines' rotation decay (`:35328-35339` and its five twins):
/// the >π unwrap, the ±0.005 dead band snap, else the ×0.96 decay.
fn idle_rotation(rot: f32) -> f32 {
    use std::f32::consts::{PI, TAU};
    let mut rot = rot;
    if rot > PI {
        rot -= TAU;
    }
    if rot > -0.005 && rot < 0.005 {
        0.0
    } else {
        rot * 0.96
    }
}

impl Projectile {
    /// `AI()`'s aiStyle-124 arm (`Projectile.cs:34970-35696`). The owner
    /// reads ride the dispatch's `player` link (the pool only simulates
    /// local-owned slots); a `None` link holds the slot (the orphan guard,
    /// the spear precedent — unreachable in game: the pets spawn owner 0).
    /// `screen_width` is `Main.screenWidth` (the 650 scan's distance gate,
    /// `:35577`). The movement is the dispatch's free integrate
    /// (`tileCollide = false`, `UpdatePosition` `:20445` — 124 is not in the
    /// `:20435` skip list). W4-g64: `dust`/`dust_ctx` carry the live pool
    /// through the `update_dust` seam (the d13 signature + the two params).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn ai_pets_style124(
        &mut self,
        world: &World,
        npcs: &[Npc],
        player: Option<&Player>,
        rng: &mut dyn DropRng,
        screen_width: f32,
        dust: &mut DustPool,
        dust_ctx: &SpawnCtx,
    ) {
        use std::f32::consts::{PI, TAU};
        // `:34972-34978` — the 650 flag and the dead-owner kill.
        let flag65 = self.kind == SUSPICIOUS_TENTACLE;
        let Some(pl) = player else {
            return; // the orphan hold (the module doc)
        };
        if pl.dead {
            kill(self);
            return;
        }
        // `:34979-35006` — the seven keep-alive pins: each type's buff flag
        // pins `timeLeft = 2`. The port's `pet_flags[kind]` IS the flag map
        // (buff 190 → `pet_flags[650]`, the `PET_FLAG_ARMS` rows → the six
        // flyers, `buff_effects_accessory.rs`).
        if pl.pet_flags[self.kind as usize] {
            self.time_left = 2;
        }
        // `:35007` — `base.direction = (this.spriteDirection = player.direction)`.
        self.direction = pl.facing;
        self.sprite_direction = pl.facing;
        // `:35008-35043` — the 650/895/896 `PlotTileLine` light casts are
        // SKIPPED (no per-frame light layer, the module doc).
        // `:35044-35091` — the home-spot ladder.
        let mut num882 = 30.0f32;
        let mut y10 = -20.0f32;
        let mut num883 = pl.facing;
        let counts = &pl.owned_projectile_counts;
        match self.kind {
            EOC_PET => {
                if counts[usize::from(SUSPICIOUS_TENTACLE)] > 0 {
                    num883 *= -1;
                }
            }
            TWINS_PET => {
                num882 = 30.0;
                y10 = -50.0;
                if counts[usize::from(SUSPICIOUS_TENTACLE)] > 0 {
                    y10 = -70.0;
                }
            }
            FAIRY_QUEEN_PET => {
                num882 = -36.0;
                y10 = -50.0;
            }
            PUMPKING_PET => {
                num882 = 30.0;
                y10 = -60.0;
                if counts[usize::from(TWINS_PET)] > 0 {
                    num882 = -30.0;
                }
            }
            ICE_QUEEN_PET => {
                num882 = -30.0;
                y10 = -50.0;
                if counts[usize::from(FAIRY_QUEEN_PET)] > 0 {
                    num882 = 30.0;
                }
            }
            GLOMMER_PET => {
                num882 = -40.0;
                y10 = -40.0;
                if counts[usize::from(FAIRY_QUEEN_PET)] > 0 {
                    num882 = 40.0;
                }
            }
            _ => {}
        }
        // `:35092-35098` — the spot off `MountedCenter`; the >1000 px
        // teleport snaps to `player.Center + offset` (NOT MountedCenter).
        let offset = Vec2::new(num883 as f32 * num882, y10);
        let (mcx, mcy) = pl.mounted_center();
        let home = Vec2::new(mcx, mcy) + offset;
        let mut center = Vec2::new(self.pos.x + self.w * 0.5, self.pos.y + self.h * 0.5);
        let num884 = center.distance(home);
        if num884 > 1000.0 {
            let (pcx, pcy) = pl.center();
            center = Vec2::new(pcx, pcy) + offset;
            self.pos = center - Vec2::new(self.w * 0.5, self.h * 0.5);
        }
        // `:35099-35115` — the approach: the <4 px drag, then the snap/0.1
        // pull (the distance read is the PRE-teleport one, the C# order).
        let vector154 = home - center;
        if num884 < 4.0 {
            self.vel *= 0.25;
        }
        if vector154 != Vec2::ZERO {
            if vector154.length() < 4.0 {
                self.vel = vector154;
            } else {
                self.vel = vector154 * 0.1;
            }
        }
        // `:35116-35123` — the three far flyers face their flight.
        if num884 > 50.0
            && matches!(self.kind, FAIRY_QUEEN_PET | ICE_QUEEN_PET | GLOMMER_PET)
        {
            self.direction = 1;
            self.sprite_direction = 1;
            if self.vel.x < 0.0 {
                self.direction = -1;
                self.sprite_direction = -1;
            }
        }
        let frames = proj_frames(self.kind) as i32;
        if self.vel.length() > 6.0 {
            // `:35124-35325` — the flight frame machines.
            match self.kind {
                SUSPICIOUS_TENTACLE => {
                    // `:35126-35150`
                    let target = self.vel.y.atan2(self.vel.x) + PI / 2.0;
                    self.rot = flight_rotation(self.rot, target);
                    self.frame_counter += 1;
                    if self.frame_counter >= 4 {
                        self.frame_counter = 0;
                        self.frame += 1;
                        if self.frame >= frames {
                            self.frame = 0;
                        }
                    }
                }
                EOC_PET => {
                    // `:35151-35170`
                    self.rot = self.vel.x * 0.125;
                    self.frame_counter += 1;
                    if self.frame_counter >= 3 {
                        self.frame_counter = 0;
                        if self.frame < 6 {
                            self.frame = 6;
                        } else {
                            self.frame += 1;
                            if self.frame > 15 {
                                self.frame = 10;
                            }
                        }
                    }
                }
                TWINS_PET => {
                    // `:35171-35203`
                    let target = self.vel.y.atan2(self.vel.x) + PI / 2.0;
                    self.rot = flight_rotation(self.rot, target);
                    self.frame_counter += 1;
                    if self.frame_counter >= 3 {
                        self.frame_counter = 0;
                        if self.frame < 6 {
                            self.frame = 6;
                        } else {
                            self.frame += 1;
                            if self.frame > 17 {
                                self.frame = 6;
                            }
                        }
                    }
                }
                FAIRY_QUEEN_PET => {
                    // `:35204-35234` — W4-g64: the 242 sparkle (`:35231`)
                    // RE-ARMED on the frame-reset tick (the d13 skip replaced).
                    let target = self.vel.x * 0.1;
                    self.rot = flight_rotation(self.rot, target);
                    self.frame_counter += 1;
                    if self.frame_counter >= 3 {
                        self.frame_counter = 0;
                        self.frame += 1;
                        if self.frame >= frames {
                            self.frame = 0;
                        }
                    }
                    // `:35229` — the frameCounter == 0 read AFTER the cycle.
                    crate::proj_trail_dust::trail_895(
                        self.pos,
                        self.w as i32,
                        self.h as i32,
                        self.frame_counter == 0,
                        rng,
                        dust,
                        dust_ctx,
                        world,
                    );
                }
                PUMPKING_PET => {
                    // `:35235-35267`
                    let target = self.vel.y.atan2(self.vel.x) + PI / 2.0;
                    self.rot = flight_rotation(self.rot, target);
                    self.frame_counter += 1;
                    if self.frame_counter >= 4 {
                        self.frame_counter = 0;
                        if self.frame < 8 {
                            self.frame = 8;
                        } else {
                            self.frame += 1;
                            if self.frame >= frames {
                                self.frame = 8;
                            }
                        }
                    }
                }
                ICE_QUEEN_PET => {
                    // `:35268-35298` — W4-g64: the 80 sparkle (`:35295`)
                    // RE-ARMED on the frame-reset tick (the d13 skip replaced).
                    let target = self.vel.x * 0.1;
                    self.rot = flight_rotation(self.rot, target);
                    self.frame_counter += 1;
                    if self.frame_counter >= 3 {
                        self.frame_counter = 0;
                        self.frame += 1;
                        if self.frame >= frames {
                            self.frame = 0;
                        }
                    }
                    crate::proj_trail_dust::trail_898(
                        self.pos,
                        self.w as i32,
                        self.h as i32,
                        self.frame_counter == 0,
                        rng,
                        dust,
                        dust_ctx,
                        world,
                    );
                }
                GLOMMER_PET => {
                    // `:35299-35324` — the wrap is `projFrames * 2`. NO dust
                    // in the C# flight arm (the ground machine below draws
                    // only the Next(150) rest roll).
                    let target = self.vel.x * 0.05;
                    self.rot = flight_rotation(self.rot, target);
                    self.frame_counter += 1;
                    if self.frame_counter >= 4 {
                        self.frame_counter = 0;
                        self.frame += 1;
                        if self.frame >= frames * 2 {
                            self.frame = 0;
                        }
                    }
                }
                _ => {}
            }
        } else {
            // `:35326-35565` — the idle frame machines.
            match self.kind {
                SUSPICIOUS_TENTACLE => {
                    // `:35326-35348`
                    self.rot = idle_rotation(self.rot);
                    self.frame_counter += 1;
                    if self.frame_counter >= 6 {
                        self.frame_counter = 0;
                        self.frame += 1;
                        if self.frame >= frames {
                            self.frame = 0;
                        }
                    }
                }
                EOC_PET => {
                    // `:35349-35361`
                    self.rot = self.vel.x * 0.125;
                    self.frame_counter += 1;
                    if self.frame_counter >= 5 {
                        self.frame_counter = 0;
                        self.frame += 1;
                        if self.frame == 6 || self.frame >= frames {
                            self.frame = 0;
                        }
                    }
                }
                TWINS_PET => {
                    // `:35362-35385` — the wrap is the literal 18.
                    self.rot = idle_rotation(self.rot);
                    self.frame_counter += 1;
                    if self.frame_counter >= 5 {
                        self.frame_counter = 0;
                        self.frame += 1;
                        if self.frame == 6 || self.frame >= 18 {
                            self.frame = 0;
                        }
                    }
                }
                FAIRY_QUEEN_PET => {
                    // `:35386-35409`
                    self.rot = idle_rotation(self.rot);
                    self.frame_counter += 1;
                    if self.frame_counter >= 5 {
                        self.frame_counter = 0;
                        self.frame += 1;
                        if self.frame >= frames {
                            self.frame = 0;
                        }
                    }
                }
                PUMPKING_PET => {
                    // `:35410-35437` — the wrap's 1/15 frame-8 re-roll.
                    self.rot = idle_rotation(self.rot);
                    self.frame_counter += 1;
                    if self.frame_counter >= 5 {
                        self.frame_counter = 0;
                        self.frame += 1;
                        if self.frame == 8 || self.frame >= frames {
                            self.frame = 0;
                            if rng.next_range(0, 15) == 0 {
                                self.frame = 8;
                            }
                        }
                    }
                }
                ICE_QUEEN_PET => {
                    // `:35438-35461`
                    self.rot = idle_rotation(self.rot);
                    self.frame_counter += 1;
                    if self.frame_counter >= 5 {
                        self.frame_counter = 0;
                        self.frame += 1;
                        if self.frame >= frames {
                            self.frame = 0;
                        }
                    }
                }
                GLOMMER_PET => {
                    // `:35462-35565` — the ground-bounce machine. W4-g64:
                    // ported FAITHFULLY from d13 (behaviour-coupled — the
                    // column scan gates the bounce cycle + its Next(150)).
                    self.rot = idle_rotation(self.rot);
                    if self.vel.length() <= 0.01 {
                        // `:35479-35497` — the 4-air-cells + 1-solid probe
                        // (`WorldGen.SolidTileAllowBottomSlope`, the shared
                        // `projectile_ai026_c` helper; the (num904 < 4) &
                        // flag67 conjunction is the C#'s non-short-circuit
                        // `&` — both operands always evaluated).
                        let mut flag66 = true;
                        let num901 = center.x as i32 / 16;
                        let mut num902 = center.y as i32 / 16;
                        let num903 = 4;
                        for num904 in 0..num903 + 1 {
                            if num901 < 0
                                || num901 >= world.width
                                || num902 < 0
                                || num902 >= world.height
                            {
                                flag66 = false;
                                break;
                            }
                            let flag67 =
                                solid_tile_allow_bottom_slope(world, num901, num902);
                            if (num904 == num903 && !flag67) || ((num904 < num903) & flag67) {
                                flag66 = false;
                                break;
                            }
                            num902 += 1;
                        }
                        if flag66 {
                            // `:35498-35537` — the grounded bounce cycle.
                            self.local_ai[0] -= 1.0;
                            if self.local_ai[0] <= 0.0 {
                                self.local_ai[0] = 0.0;
                                if self.frame < frames * 2 {
                                    self.frame = frames * 2;
                                }
                                let mut num905 = 3;
                                if self.frame <= 30 && self.frame <= 33 {
                                    num905 = 2;
                                }
                                self.frame_counter += 1;
                                if self.frame_counter >= num905 {
                                    self.frame_counter = 0;
                                    self.frame += 1;
                                    if self.frame >= frames * 3 {
                                        // `:35519` — the rest-length roll.
                                        self.local_ai[0] = (200 + rng.next_range(0, 150)) as f32;
                                        self.frame = 0;
                                    }
                                    if self.frame == 32 {
                                        // `:35524` — GlommerBounce is a
                                        // trackable cue: deferred (the
                                        // module doc).
                                    }
                                }
                            } else {
                                self.frame_counter += 1;
                                if self.frame_counter >= 4 {
                                    self.frame_counter = 0;
                                    self.frame += 1;
                                    if self.frame >= frames * 2 {
                                        self.frame = 0;
                                    }
                                }
                            }
                        } else {
                            // `:35538-35550` — airborne: the 300 rest pin.
                            self.local_ai[0] = 300.0;
                            self.frame_counter += 1;
                            if self.frame_counter >= 4 {
                                self.frame_counter = 0;
                                self.frame += 1;
                                if self.frame >= frames * 2 {
                                    self.frame = 0;
                                }
                            }
                        }
                    } else {
                        // `:35552-35564` — moving: the same 300 pin + cycle.
                        self.local_ai[0] = 300.0;
                        self.frame_counter += 1;
                        if self.frame_counter >= 4 {
                            self.frame_counter = 0;
                            self.frame += 1;
                            if self.frame >= frames * 2 {
                                self.frame = 0;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        // `:35566-35570` — the 650 point timer: the pre-increment only runs
        // while `ai[0] > 0`; the 60-wrap clears both ai slots.
        if flag65 && self.ai[0] > 0.0 {
            self.ai[0] += 1.0;
            if self.ai[0] >= 60.0 {
                self.ai[0] = 0.0;
                self.ai[1] = 0.0;
            }
        }
        // `:35571-35609` — the 650 ore-highlight scan.
        if flag65 && rng.next_range(0, 15) == 0 {
            let mut num906 = -1i32;
            let mut num907 = -1i32;
            let mut num908 = -1.0f32;
            let num909 = 17i32;
            let (pcx, pcy) = pl.center();
            if (center - Vec2::new(pcx, pcy)).length() < screen_width {
                let num910 =
                    (center.x as i32 / 16).clamp(num909 + 1, world.width - num909 - 1);
                let num911 =
                    (center.y as i32 / 16).clamp(num909 + 1, world.height - num909 - 1);
                for num912 in num910 - num909..=num910 + num909 {
                    for num913 in num911 - num909..=num911 + num909 {
                        // `:35587` — the draw is UNCONDITIONAL per tile.
                        let num914 = rng.next_range(0, 8);
                        let in_radius = Vec2::new(
                            (num910 - num912) as f32,
                            (num911 - num913) as f32,
                        )
                        .length()
                            < num909 as f32;
                        let spelunkable = world.tile(num912, num913).is_some_and(|t| {
                            t.active() && is_tile_spelunkable(t.kind, t.frame_x, t.frame_y)
                        });
                        if num914 < 4 && in_radius && spelunkable {
                            let tile_center = Vec2::new(
                                num912 as f32 * 16.0 + 8.0,
                                num913 as f32 * 16.0 + 8.0,
                            );
                            // `:35590` — `base.Distance` reads the Centre.
                            let num915 = center.distance(tile_center);
                            if num915 < num908 || num908 == -1.0 {
                                num908 = num915;
                                num906 = num912;
                                num907 = num913;
                                self.ai[0] = 1.0;
                                self.ai[1] = (tile_center.y - center.y)
                                    .atan2(tile_center.x - center.x);
                            }
                            // `:35599-35604` — W4-g64: the <2 sparkle dust
                            // RE-ARMED (the d13 body-skip replaced).
                            if num914 < 2 {
                                crate::proj_trail_dust::trail_650_scan_sparkle(
                                    Vec2::new(num912 as f32 * 16.0, num913 as f32 * 16.0),
                                    rng,
                                    dust,
                                    dust_ctx,
                                    world,
                                );
                            }
                        }
                    }
                }
            }
            // The C# writes num906/num907 but never reads them back.
            let _ = (num906, num907);
        }
        // `:35610-35613` — the steering tail is the 650's alone.
        if !flag65 {
            return;
        }
        // `:35614-35629` — the angle/extension decode of the two localAI
        // stores. `Math.IEEERemainder(x, 1)` is the round-TIES-EVEN
        // fractional part (f64, the C# double overload).
        let f = self.local_ai[0] % TAU - PI;
        let mut num917 = {
            let x = self.local_ai[1] as f64;
            let mut r = (x - x.round_ties_even()) as f32;
            if r < 0.0 {
                r += 1.0;
            }
            r
        };
        let mut num918 = (self.local_ai[1] as f64).floor() as f32;
        let max = 0.999f32;
        let mut num919 = 0.0f32;
        let mut num920 = 0i32;
        let mut amount2 = 0.1f32;
        let flag68 = Vec2::new(pl.vx, pl.vy).length() > 3.0;
        // `:35630-35651` — the two closest-NPC scans (the friendly/small
        // scan only feeds while the owner stands still).
        let mut num921 = -1i32;
        let mut num922 = -1i32;
        let mut num923 = 300.0f32;
        let mut num924 = 500.0f32;
        for (i, n) in npcs.iter().enumerate() {
            if !(n.hp > 0 && n.chaseable && !n.dont_take_damage && !n.immortal) {
                continue;
            }
            let num926 = center.distance(npc_center(n));
            if n.friendly || n.max_hp <= 5 {
                if num926 < num923 && !flag68 {
                    num923 = num926;
                    num922 = i as i32;
                }
            } else if num926 < num924 {
                num924 = num926;
                num921 = i as i32;
            }
        }
        // `:35652-35692` — the target ladder.
        if flag68 {
            // `AngleTo(Center + velocity)` = the velocity's own angle.
            num919 = pl.vy.atan2(pl.vx);
            num920 = 1;
            num917 = (num917 + 0.05).clamp(0.0, max);
            num918 += csign(-10.0 - num918);
        } else if num921 != -1 {
            let c = npc_center(&npcs[num921 as usize]);
            num919 = (c.y - center.y).atan2(c.x - center.x);
            num920 = 2;
            num917 = (num917 + 0.05).clamp(0.0, max);
            num918 += csign(-12.0 - num918);
        } else if num922 != -1 {
            let c = npc_center(&npcs[num922 as usize]);
            num919 = (c.y - center.y).atan2(c.x - center.x);
            num920 = 3;
            num917 = (num917 + 0.05).clamp(0.0, max);
            num918 += csign(6.0 - num918);
        } else if self.ai[0] > 0.0 {
            num919 = self.ai[1];
            num917 = (num917 + csign(0.75 - num917) * 0.05).clamp(0.0, max);
            num920 = 4;
            num918 += csign(10.0 - num918);
            if rng.next_range(0, 10) == 0 {
                // `:35681-35683` — W4-g64: the 204 point dust RE-ARMED
                // (the d13 body-skip replaced). `f` is the DECODED angle
                // (pre-lerp), num917 the POST-step extension.
                crate::proj_trail_dust::trail_650_point_sparkle(
                    center, f, num917, rng, dust, dust_ctx, world,
                );
            }
        } else {
            num919 = if pl.facing == 1 { 0.0 } else { 3.1416028 };
            num917 = (num917 + csign(0.75 - num917) * 0.05).clamp(0.0, max);
            num918 += csign(0.0 - num918);
            amount2 = 0.12;
        }
        // `:35693-35695` — the vector lerp back to an angle, then the
        // state-tagged re-encode (`num920 * 2π + π` offset).
        let (fs, fc) = f.sin_cos();
        let (ts, tc) = num919.sin_cos();
        let blended = Vec2::new(fc, fs).lerp(Vec2::new(tc, ts), amount2);
        num919 = blended.y.atan2(blended.x);
        self.local_ai[0] = num919 + num920 as f32 * TAU + PI;
        self.local_ai[1] = num918 + num917;
    }
}

#[cfg(test)]
mod w4_g64_style124_tests {
    use super::*;
    use crate::projectile::{STYLE124_PETS, proj_data};
    use tr_core::tile::FLAG_ACTIVE;

    fn world() -> World {
        World::new("t".into(), 64, 64)
    }

    fn player(w: &World) -> Player {
        let mut p = Player::spawn(w);
        p.facing = 1;
        p
    }

    fn pet(kind: u16, pos: Vec2) -> Projectile {
        let mut p = Projectile::default();
        p.active = true;
        p.kind = kind;
        p.owner = 0;
        p.w = 20.0;
        p.h = 20.0;
        p.pos = pos;
        p.tile_collide = false;
        p
    }

    /// A draw-recording `Main.rand` stand-in (the d13 `CensusRng` shape):
    /// the census proves count AND order; every range draw returns its min.
    struct CensusRng {
        ranges: Vec<(i32, i32)>,
        floats: usize,
    }

    impl DropRng for CensusRng {
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            self.ranges.push((min, max));
            min
        }
        fn next_float(&mut self) -> f32 {
            self.floats += 1;
            0.0
        }
    }

    fn scratch(w: &World) -> (DustPool, SpawnCtx) {
        crate::projectile::scratch_dust(w)
    }

    /// The three rowed kinds' `SetDefaults` slice (`Projectile.cs:8553-8565`,
    /// `:8576-8588`, `:9034-9046`) + the frame sheets (`Main.cs:8683/8686/
    /// 8697`). The other four rows stay d13's (the merge keep-one).
    #[test]
    fn w4_g64_the_rowed_124_pets_carry_the_csharp_set_defaults() {
        for kind in [FAIRY_QUEEN_PET, ICE_QUEEN_PET, GLOMMER_PET] {
            let d = proj_data(kind).unwrap_or_else(|| panic!("kind {kind} rowed"));
            assert_eq!((d.w, d.h), (20.0, 20.0), "kind {kind}");
            assert_eq!(d.ai_style, 124, "kind {kind}");
            assert_eq!(d.penetrate, -1, "kind {kind}");
            assert_eq!(d.time_left, 18000, "kind {kind}: `timeLeft *= 5`");
            assert!(!d.tile_collide, "kind {kind}");
            assert!(d.ignore_water, "kind {kind}");
            assert!(d.net_important, "kind {kind}");
            assert!(!d.minion, "kind {kind}: the pets are not minions");
        }
        for (kind, frames) in [(FAIRY_QUEEN_PET, 6), (ICE_QUEEN_PET, 16), (GLOMMER_PET, 12)] {
            assert_eq!(proj_frames(kind), frames, "kind {kind}'s sheet");
        }
        // The full seven-kind ladder set is named (d13's rows complete it).
        assert_eq!(STYLE124_PETS.len(), 7);
    }

    /// `:34974-35006` — the dead-owner kill and the buff-flag keep-alive pin.
    #[test]
    fn w4_g64_the_keep_alive_pins_ride_pet_flags() {
        let w = world();
        let (mut sd, sc) = scratch(&w);
        for kind in [FAIRY_QUEEN_PET, ICE_QUEEN_PET, GLOMMER_PET] {
            let mut pl = player(&w);
            pl.pet_flags[kind as usize] = true;
            let mut p = pet(kind, Vec2::new(pl.x + 30.0, pl.y - 20.0));
            p.time_left = 500;
            let mut rng = CensusRng { ranges: Vec::new(), floats: 0 };
            p.ai_pets_style124(&w, &[], Some(&pl), &mut rng, 1280.0, &mut sd, &sc);
            assert_eq!(p.time_left, 2, "kind {kind}: the pin");
            pl.dead = true;
            p.time_left = 500;
            p.ai_pets_style124(&w, &[], Some(&pl), &mut rng, 1280.0, &mut sd, &sc);
            assert!(!p.active, "kind {kind}: the dead-owner kill");
            assert_eq!(p.time_left, 0, "kind {kind}: Kill zeroes");
        }
    }

    /// `:35477-35550` — the 957 ground probe (the d13 witness, ported):
    /// four air cells above a solid floor ground the pet (the rest
    /// countdown decrements, ZERO draws before the cycle end); a solid
    /// cell inside the span keeps it airborne (the 300 pin).
    #[test]
    fn w4_g64_the_957_ground_probe_splits_grounded_and_airborne() {
        let mut w = world();
        let (mut sd, sc) = scratch(&w);
        let pl = player(&w);
        // Park the pet exactly on its home spot so the approach leaves
        // zero velocity (the ground machine only runs at <= 0.01 speed).
        let (mcx, mcy) = pl.mounted_center();
        let home = Vec2::new(mcx - 40.0, mcy - 40.0); // facing 1, no 895 owned
        let cx = home.x as i32 / 16;
        let cy = home.y as i32 / 16;
        // Floor four cells below the centre row.
        for x in 0..w.width {
            let t = w.tile_mut(x, cy + 4).unwrap();
            t.kind = 1; // dirt
            t.flags |= FLAG_ACTIVE;
        }
        let mut p = pet(GLOMMER_PET, home - Vec2::new(10.0, 10.0));
        p.local_ai[0] = 5.0;
        let mut rng = CensusRng { ranges: Vec::new(), floats: 0 };
        p.ai_pets_style124(&w, &[], Some(&pl), &mut rng, 1280.0, &mut sd, &sc);
        assert_eq!(p.local_ai[0], 4.0, "grounded: the rest countdown");
        assert!(rng.ranges.is_empty(), "the 957 draws nothing before the cycle end");
        // A solid cell inside the air span breaks the probe → the 300 pin.
        let t = w.tile_mut(cx, cy + 2).unwrap();
        t.kind = 1;
        t.flags |= FLAG_ACTIVE;
        p.local_ai[0] = 5.0;
        p.ai_pets_style124(&w, &[], Some(&pl), &mut rng, 1280.0, &mut sd, &sc);
        assert_eq!(p.local_ai[0], 300.0, "airborne: the 300 pin");
        assert!(rng.ranges.is_empty(), "still no draws off the cycle end");
    }

    /// `:35498-35527` — the grounded bounce cycle's END: the frame walks
    /// `projFrames*2 → *3` (the 2-tick cadence while frame <= 30), the
    /// wrap rolls `200 + Next(150)` and resets frame 0. Stream-order
    /// witness: the ONE Next(150) lands exactly at the wrap tick, after
    /// the countdown's zero-draw ticks.
    #[test]
    fn w4_g64_the_957_bounce_cycle_end_rolls_the_rest_pin() {
        let mut w = world();
        let (mut sd, sc) = scratch(&w);
        let pl = player(&w);
        let (mcx, mcy) = pl.mounted_center();
        let home = Vec2::new(mcx - 40.0, mcy - 40.0);
        let cy = home.y as i32 / 16;
        for x in 0..w.width {
            let t = w.tile_mut(x, cy + 4).unwrap();
            t.kind = 1;
            t.flags |= FLAG_ACTIVE;
        }
        let mut p = pet(GLOMMER_PET, home - Vec2::new(10.0, 10.0));
        // localAI[0] = 1: this tick decrements to 0 and pins frame 24
        // (projFrames*2); the 2-tick cadence walks 24→36 in 12 steps
        // (frame <= 30 keeps num905 = 2 up to 30, then 3 for 31..35 —
        // the C# `frame <= 30 && frame <= 33` conjunction).
        p.local_ai[0] = 1.0;
        let mut rng = CensusRng { ranges: Vec::new(), floats: 0 };
        // Tick 1: countdown hits 0, frame pinned to 24, counter 1 < 2.
        p.ai_pets_style124(&w, &[], Some(&pl), &mut rng, 1280.0, &mut sd, &sc);
        assert_eq!(p.frame, 24, "the pin to projFrames*2");
        assert!(rng.ranges.is_empty());
        // Run to the wrap: the frame reaches 36 (12*3) and rolls.
        let mut guard = 0;
        while rng.ranges.is_empty() && guard < 100 {
            p.ai_pets_style124(&w, &[], Some(&pl), &mut rng, 1280.0, &mut sd, &sc);
            guard += 1;
        }
        assert_eq!(rng.ranges, vec![(0, 150)], "ONE Next(150) at the cycle end");
        assert_eq!(p.frame, 0, "the wrap resets the frame");
        assert!(p.local_ai[0] >= 200.0 && p.local_ai[0] < 350.0, "the 200+ rest pin");
    }

    /// `:35204-35234`/`:35268-35298` — the 895/898 flight sparkles fire on
    /// the frame-reset tick (frameCounter hits 3 → 0) and NOWHERE else.
    /// The pet flies (100 px off the spot → the 0.1 pull = speed 10 > 6).
    /// Stream-order: two identical runs land identical pools + tails (the
    /// pool internals ride the same stream — the 642 parity shape).
    #[test]
    fn w4_g64_the_895_898_flight_sparkles_fire_on_the_frame_reset() {
        let w = world();
        for (kind, dust_type) in [(FAIRY_QUEEN_PET, 242), (ICE_QUEEN_PET, 80)] {
            let mut pl = player(&w);
            pl.x = 512.0;
            pl.y = 512.0;
            let (mcx, mcy) = pl.mounted_center();
            let off = if kind == FAIRY_QUEEN_PET { -36.0 } else { -30.0 };
            let home = Vec2::new(mcx + off, mcy - 50.0);
            let mk = || pet(kind, home + Vec2::new(100.0, 0.0) - Vec2::new(10.0, 10.0));
            let (mut sd, sc) = scratch(&w);
            let mut p = mk();
            p.frame_counter = 2; // the next tick is the reset tick
            let mut rng = tr_core::UnifiedRandom::new(124);
            p.ai_pets_style124(&w, &[], Some(&pl), &mut rng, 1280.0, &mut sd, &sc);
            assert!(p.vel.length() > 6.0, "the flight branch");
            assert_eq!(p.frame_counter, 0, "the reset tick");
            let spawned: Vec<usize> = (0..6000).filter(|&i| sd.slot(i).active).collect();
            assert_eq!(spawned.len(), 1, "one sparkle on the reset tick");
            let d = sd.slot(spawned[0]);
            assert_eq!(d.dust_type, dust_type);
            assert!(d.no_gravity);
            assert_eq!(d.alpha, 50);
            // The parity twin: identical pool + stream tail.
            let (mut sd2, sc2) = scratch(&w);
            let mut p2 = mk();
            p2.frame_counter = 2;
            let mut rng2 = tr_core::UnifiedRandom::new(124);
            p2.ai_pets_style124(&w, &[], Some(&pl), &mut rng2, 1280.0, &mut sd2, &sc2);
            for &i in &spawned {
                assert_eq!(sd.slot(i), sd2.slot(i), "slot {i} deterministic");
            }
            for _ in 0..8 {
                assert_eq!(rng.next_range(0, 1000), rng2.next_range(0, 1000), "tail");
            }
            // A NON-reset tick spawns nothing (the frameCounter == 0 gate).
            let (mut sd3, sc3) = scratch(&w);
            let mut p3 = mk();
            p3.frame_counter = 0; // increments to 1 — no reset
            let mut rng3 = tr_core::UnifiedRandom::new(124);
            p3.ai_pets_style124(&w, &[], Some(&pl), &mut rng3, 1280.0, &mut sd3, &sc3);
            assert!((0..6000).all(|i| !sd3.slot(i).active), "off-reset: zero dust");
        }
    }

    /// `:35571-35609` + `:35679-35684` — the 650 ore scan with the dust
    /// bodies LIVE: a copper ore two tiles right of the pet's centre arms
    /// the point state AND spawns the <2 scan sparkle (the CensusRng's
    /// min-returning Next(8) = 0 < 2); the steering tail's Next(10) = 0
    /// spawns the point-at-ore 204. Stream-order parity twin + the 204
    /// shape (fadeIn 0.75, the ×0.1 velocity).
    #[test]
    fn w4_g64_the_650_scan_sparkles_ride_the_live_pool() {
        let mut w = world();
        let mut pl = player(&w);
        pl.x = 512.0;
        pl.y = 512.0;
        let (pcx, pcy) = pl.center();
        let ore_x = (pcx as i32 / 16) + 2;
        let ore_y = pcy as i32 / 16;
        let t = w.tile_mut(ore_x, ore_y).unwrap();
        t.kind = 7; // copper, `Main.cs:9421`
        t.flags |= FLAG_ACTIVE;
        let mk = || pet(SUSPICIOUS_TENTACLE, Vec2::new(pcx - 10.0, pcy - 10.0));
        let (mut sd, sc) = scratch(&w);
        let mut p = mk();
        let mut rng = CensusRng { ranges: Vec::new(), floats: 0 };
        p.ai_pets_style124(&w, &[], Some(&pl), &mut rng, 1280.0, &mut sd, &sc);
        assert_eq!(p.ai[0], 1.0, "the ore arms the point state");
        assert_eq!(rng.ranges[0], (0, 15), "the scan gate");
        assert_eq!(rng.floats, 0, "no explicit float draws on this path");
        // The window's per-tile Next(8)s are interleaved with the ore
        // tile's pool internals (the <2 sparkle) — the gate draw ORDER
        // stays: the first 1 + (ore tile index) draws are the gate + the
        // per-tile rolls up to the ore.
        let spawned: Vec<usize> = (0..6000).filter(|&i| sd.slot(i).active).collect();
        assert_eq!(spawned.len(), 2, "the scan sparkle + the point sparkle");
        for &i in &spawned {
            let d = sd.slot(i);
            assert_eq!(d.dust_type, 204);
            assert_eq!(d.fade_in, 0.75);
        }
        // The scan sparkle sits in the ore tile's 16x16 box.
        let scan = sd.slot(spawned[0]);
        assert!(scan.position.x >= ore_x as f32 * 16.0
            && scan.position.x < ore_x as f32 * 16.0 + 16.0);
        // The point sparkle: Center + f.ToRotationVector2()*6*num917 - 4.
        // localAI starts 0 → f = -π, num917 = 0 + sign(0.75)*0.05 = 0.05.
        let point = sd.slot(spawned[1]);
        let f = 0.0f32 % std::f32::consts::TAU - std::f32::consts::PI;
        let (s, c) = f.sin_cos();
        let want = Vec2::new(pcx, pcy) + Vec2::new(c, s) * 6.0 * 0.05 - Vec2::ONE * 4.0;
        assert!((point.position.x - want.x).abs() < 16.0
            && (point.position.y - want.y).abs() < 16.0, "the 8x8 box off the anchor");
        // The parity twin on a concrete stream.
        let (mut sd2, sc2) = scratch(&w);
        let mut p2 = mk();
        let mut rng2 = tr_core::UnifiedRandom::new(650);
        p2.ai_pets_style124(&w, &[], Some(&pl), &mut rng2, 1280.0, &mut sd2, &sc2);
        let (mut sd3, sc3) = scratch(&w);
        let mut p3 = mk();
        let mut rng3 = tr_core::UnifiedRandom::new(650);
        p3.ai_pets_style124(&w, &[], Some(&pl), &mut rng3, 1280.0, &mut sd3, &sc3);
        for i in 0..6000 {
            assert_eq!(sd2.slot(i), sd3.slot(i), "slot {i} deterministic");
        }
        for _ in 0..8 {
            assert_eq!(rng2.next_range(0, 1000), rng3.next_range(0, 1000), "tail");
        }
    }
}