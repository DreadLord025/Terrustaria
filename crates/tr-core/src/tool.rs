//! The pickaxe damage model — `Player.GetPickaxeDamage`
//! (`Player.cs:53902-54045`) ported verbatim: the per-tile damage multiplier
//! over the pick power, the zero-damage tier gates and the instant-break
//! rows. `PickTile` (`Player.cs:53757-53826`) adds the damage into the
//! `HitTile` book until 100 breaks the tile; the mining cadence is
//! `toolTime`/`itemTime` (`Player.cs:43576-43587`, `:45777-45780`), wired in
//! `world_sim/editing.rs`.
//!
//! The axe tiles (`Main.tileAxe`) route to their own arm in
//! `ItemCheck_UseMiningTools_ActuallyUseMiningTool` (`Player.cs:45706-45771`,
//! S1-P-i30's [`axe_damage`] + the `HitTile` accumulation in
//! `world_sim/editing.rs`); the hammer tiles (`Main.tileHammer`) keep their
//! pre-i29 pacing until the i31 task. A pick never damages either set
//! (`Player.cs:45656`). This module keeps the shared tile-set predicates,
//! the pick side and the axe damage ladder.

use crate::tiledata as td;

/// `Main.tileAxe` (`Main.cs:7208-7223`, all 16 assignments — the gem-tree
/// rows are `:7216-7223`) — the tiles an axe chops. A pick cannot mine them
/// (`Player.cs:45656`).
pub fn is_axe_tile(kind: u16) -> bool {
    matches!(
        kind,
        5 | 72 | 80 | 323 | 488 | 583 | 584 | 585 | 586 | 587 | 588 | 589 | 596 | 616 | 634 | 704
    )
}

/// `Main.tileHammer` (`Main.cs`, the `tileHammer[..] = true` assignments) —
/// the tiles only a hammer damages (`Player.cs:45665-45704`): the demon
/// altar 26, the shadow orb 31 and the crimson hearts 695/696.
pub fn is_hammer_tile(kind: u16) -> bool {
    matches!(kind, 26 | 31 | 695 | 696)
}

/// One axe hit's damage before the `HitTile` accumulation — the axe arm's
/// damage line (`Player.cs:45708-45712`) verbatim:
/// `num2 = (tile.type != 80) ? num2 + (int)((float)sItem.axe * 1.2f)
///                        : num2 + (int)((float)(sItem.axe * 3) * 1.2f)`
/// then the getGoodWorld `(int)((double)num2 * 1.3)`. `base` is the running
/// `num2` the caller carries in — the `Main.tileNoFail` 100 head
/// (`:45661-45664`); NO axe tile is `tileNoFail` (`Main.cs:7189-7203` vs
/// `:7208-7223`), so the arm always passes 0. The C# multiplies in f32
/// (`(float)sItem.axe * 1.2f`) and truncates toward zero on the int cast;
/// the cactus 80 triples the axe power FIRST (`sItem.axe * 3`, integer),
/// then the same f32 1.2 scale. The `CanKillTile` refusal (`:45717-45720`)
/// zeroes the result — the caller applies it, like the pick arm's
/// `PickTile_DetermineDamage`.
pub fn axe_damage(kind: u16, axe_power: i32, base: i32, get_good: bool) -> i32 {
    // `:45708` — the cactus triple, then the f32 1.2 scale + truncation.
    let scaled = if kind != 80 {
        axe_power as f32 * 1.2
    } else {
        (axe_power * 3) as f32 * 1.2
    };
    let mut num = base + scaled as i32;
    // `:45709-45712` — the For-the-Worthy 1.3 (f64 in the C#).
    if get_good {
        num = (num as f64 * 1.3) as i32;
    }
    num
}

/// `Main.tileDungeon` (`Main.cs:7992-7997`) — the bricks the dungeon gate
/// covers.
fn is_dungeon_tile(kind: u16) -> bool {
    matches!(kind, 41 | 43 | 44 | 677..=679)
}

/// `Main.tileNoFail` (`Main.cs`'s `tileNoFail[..] = true` assignments, the
/// full walk) — the tiles that start at 100 damage (`GetPickaxeDamage`'s
/// head, `Player.cs:53905-53908`): one pick hit breaks them.
pub fn is_no_fail_tile(kind: u16) -> bool {
    matches!(
        kind,
        3 | 4
            | 24
            | 32
            | 50
            | 51
            | 52
            | 61
            | 62
            | 69
            | 73
            | 74
            | 81
            | 82
            | 83
            | 84
            | 110
            | 113
            | 115
            | 129
            | 162
            | 165
            | 184
            | 185
            | 186
            | 187
            | 192
            | 201
            | 205
            | 227
            | 233
            | 254
            | 324
            | 330
            | 331
            | 332
            | 333
            | 352
            | 373
            | 374
            | 375
            | 382
            | 384
            | 461
            | 481
            | 482
            | 483
            | 484
            | 485
            | 518
            | 519
            | 528
            | 529
            | 530
            | 549
            | 624
            | 636
            | 637
            | 638
            | 654
            | 655
            | 656
            | 666
            | 697
            | 700
            | 701
            | 705
            | 709
    )
}

/// `Main.tileRope` (`Main.cs`'s `tileRope[..] = true` assignments) — the
/// ropes break instantly (`Player.cs:53980-53983`).
pub fn is_rope_tile(kind: u16) -> bool {
    matches!(kind, 213 | 214 | 353 | 365 | 366 | 449 | 450 | 451 | 504)
}

/// The world inputs `GetPickaxeDamage` reads. `Main.getGoodWorld` is the
/// For-the-Worthy seed (`SeedFlags::FOR_THE_WORTHY`, the port's
/// `worldgen_job.rs:143` mapping); `Main.infectedSeed` is always false in
/// 1.4.5.8 (`Main.cs:52266`/`:66541`, the wld save writes it false) — the
/// field keeps the C# shape for the gate at `:53914`.
#[derive(Clone, Copy, Debug, Default)]
pub struct PickWorld {
    /// `Main.worldSurface` — the demonite/crimtane + dungeon depth gates.
    pub world_surface: f64,
    /// `Main.UnderworldLayer` (`Main.cs:2790`: `maxTilesY - 200`) — the
    /// hellforge gate (`Player.cs:53934`).
    pub underworld_layer: i32,
    /// `Main.maxTilesX` — the dungeon band's 0.35/0.65 x window
    /// (`Player.cs:53955`).
    pub max_tiles_x: i32,
    /// `Main.getGoodWorld` — the tombstone quarter-power and the ×2 damage
    /// (`Player.cs:53909`, `:53836-53839`).
    pub get_good: bool,
    /// `Main.infectedSeed` — always false in 1.4.5.8 (see the type doc).
    pub infected_seed: bool,
    /// `Main.notTheBeesWorld` / `Main.noTrapsWorld` /
    /// `Main.tenthAnniversaryWorld` — the trap gate's seed clause
    /// (`Player.cs:53946`).
    pub not_the_bees: bool,
    pub no_traps: bool,
    pub tenth_anniversary: bool,
}

/// One pick hit's damage before the `HitTile` accumulation —
/// `Player.GetPickaxeDamage` (`Player.cs:53902-54045`) verbatim. The C#
/// re-reads `Main.tile[x, y]` after the mannequin/weapon-rack redirects
/// (`:53984-54043`); the port passes a `tile_at` closure for those reads
/// (the hit buffer's `UpdatePosition` side rides the caller). `frame_x` /
/// `frame_y` are the HIT cell's frames.
///
/// The returned damage is the pre-`CanKillTile` value;
/// `PickTile_DetermineDamage` (`:53828-53844`) zeroes it on a `CanKillTile`
/// refusal and doubles it on `getGoodWorld` — the caller applies both.
pub fn pickaxe_damage(
    x: i32,
    y: i32,
    pick_power: i32,
    kind: u16,
    frame_x: i32,
    frame_y: i32,
    w: &PickWorld,
    tile_at: impl Fn(i32, i32) -> Option<crate::tile::Tile>,
) -> i32 {
    let p = pick_power;
    // `:53904-53908` — the noFail head.
    let mut num = if is_no_fail_tile(kind) { 100 } else { 0 };
    // `:53909` — the multiplier ladder (the decompiled nested ternary,
    // unrolled arm by arm; every `/` is the C# integer division).
    if is_dungeon_tile(kind) || kind == 58 || kind == 25 || kind == 117 || kind == 203 {
        num += p / 2;
    } else if kind == 85 {
        // Tombstones (`TileID.cs:85`).
        num += if !w.get_good { p } else { p / 4 };
    } else if kind == 48 || kind == 232 || td::SET_CLOUDS.has(kind) {
        // Spikes 48 / WoodenSpikes 232 / `TileID.Sets.Clouds`
        // (`TileID.cs:505`: 189, 196, 460, 717-719).
        num += p * 2;
    } else if kind == 226 {
        // LihzahrdBrick.
        num += p / 4;
    } else if kind == 107 || kind == 221 {
        // Cobalt / Palladium.
        num += p / 2;
    } else if kind == 108 || kind == 222 {
        // Mythril / Orichalcum.
        num += p / 3;
    } else if kind == 111 || kind == 223 {
        // Adamantite / Titanium.
        num += p / 4;
    } else if kind == 211 {
        // Chlorophyte.
        num += p / 5;
    } else {
        num += p;
    }
    // `:53910-53913` — the chlorophyte 200 gate (its own `if`, NOT part of
    // the else-chain below — a verbatim quirk).
    if kind == 211 && p < 200 {
        num = 0;
    }
    // `:53914-53971` — the zero-damage tier gates, the else-if chain in
    // the C# order.
    if !w.infected_seed && (kind == 25 || kind == 203) && p < 65 {
        num = 0;
    } else if kind == 117 && p < 65 {
        num = 0;
    } else if kind == 37 && p < 50 {
        // Meteorite.
        num = 0;
    } else if (kind == 22 || kind == 204) && (y as f64) > w.world_surface && p < 55 {
        // Demonite / Crimtane — below the surface only.
        num = 0;
    } else if kind == 56 && p < 55 {
        // Obsidian.
        num = 0;
    } else if kind == 77 && p < 65 && y >= w.underworld_layer {
        // Hellforge — in the underworld only.
        num = 0;
    } else if kind == 58 && p < 65 {
        // Hellstone.
        num = 0;
    } else if (kind == 226 || kind == 237) && p < 210 {
        // LihzahrdBrick / LihzahrdAltar.
        num = 0;
    } else if kind == 137
        && p < 210
        && (!w.not_the_bees || !w.no_traps || w.tenth_anniversary)
    {
        // Traps — the dart/geyser/spear rows `frameY / 18` 1..=4 (the
        // `(uint)(frameY / 18 - 1) <= 3u` wrap, `:53948`).
        if ((frame_y / 18 - 1) as u32) <= 3 {
            num = 0;
        }
    } else if is_dungeon_tile(kind) && p < 100 && (y as f64) > w.world_surface {
        // The dungeon bricks — below the surface, outside the middle
        // 0.35..0.65 x band (`:53955`).
        if (x as f64) < (w.max_tiles_x as f64) * 0.35
            || (x as f64) > (w.max_tiles_x as f64) * 0.65
        {
            num = 0;
        }
    } else if (kind == 107 || kind == 221) && p < 100 {
        num = 0;
    } else if (kind == 108 || kind == 222) && p < 110 {
        num = 0;
    } else if (kind == 111 || kind == 223) && p < 150 {
        num = 0;
    }
    // `:53972-53975` — the soft tiles take a SECOND pick-power add (dirt 0,
    // snow 147, clay 40, sand 53, ash 57, mud 59, silt 123, slush 224,
    // hardened sand 397).
    if matches!(kind, 147 | 0 | 40 | 53 | 57 | 59 | 123 | 224 | 397) {
        num += p;
    }
    // `:53976-53979` — the desert fossil's flat +5.
    if kind == 404 {
        num += 5;
    }
    // `:53980-53983` — the stalactite 165, the ropes and the crimson grass
    // 199 break instantly.
    if kind == 165 || is_rope_tile(kind) || kind == 199 {
        num = 100;
    }
    // `:53984-53997` — the mannequins 128/269: the right-half cells
    // (frameX 18/54) redirect the hit to the left cell; a dressed frame
    // (frameX >= 100) refuses the pick (`Main.blockMouse` — the port has
    // no blockMouse, the zero damage is the observable half).
    if kind == 128 || kind == 269 {
        let mut tx = x;
        let mut t_fx = frame_x;
        if frame_x == 18 || frame_x == 54 {
            tx -= 1;
            if let Some(t) = tile_at(tx, y) {
                t_fx = i32::from(t.frame_x);
            }
        }
        if t_fx >= 100 {
            num = 0;
        }
    }
    // `:53998-54043` — the weapon rack 334: the frameY 0/36 rows redirect
    // down/up (the SECOND check reads the redirected tile's frameY, a
    // verbatim re-read), the frameX third redirects left, and a racked
    // weapon (frameX >= 5000) refuses the pick.
    if kind == 334 {
        let mut tx = x;
        let mut ty = y;
        let mut t_fx = frame_x;
        let mut t_fy = frame_y;
        if t_fy == 0 {
            ty += 1;
            if let Some(t) = tile_at(tx, ty) {
                t_fx = i32::from(t.frame_x);
                t_fy = i32::from(t.frame_y);
            }
        }
        if t_fy == 36 {
            ty -= 1;
            if let Some(t) = tile_at(tx, ty) {
                t_fx = i32::from(t.frame_x);
                t_fy = i32::from(t.frame_y);
            }
        }
        let mut racked = t_fx >= 5000;
        if !racked {
            tx -= t_fx / 18 % 3;
            if let Some(t) = tile_at(tx, ty) {
                if i32::from(t.frame_x) >= 5000 {
                    racked = true;
                    t_fx = i32::from(t.frame_x);
                }
            }
        }
        if racked {
            let mut fx = t_fx;
            let mut count = 0;
            while fx >= 5000 {
                fx -= 5000;
                count += 1;
            }
            if count != 0 {
                num = 0;
            }
        }
    }
    num
}

/// The pick-power tier gate — `pickaxe_damage(...) > 0` for the plain
/// tiles (no frame redirects). The depth-only shape the mining gate and
/// `cursor_cell_is_tile_target` read; the dungeon x band, the hellforge
/// underworld gate and the trap rows need the real world inputs — those
/// callers pass [`pickaxe_damage`] with a full [`PickWorld`] directly.
/// Here the dungeon band is the pre-i29 depth-only approximation
/// (`max_tiles_x` at the sentinel keeps the middle-band arm free), recorded
/// in ANALYSIS "The pickaxe mining census (S1-P-i29)".
pub fn pick_can_mine(kind: u16, pick_power: i32, below_surface: bool) -> bool {
    let w = PickWorld {
        // A below-surface target sits one row past the surface; an
        // above-surface one on it.
        world_surface: if below_surface { 0.0 } else { 1.0e9 },
        underworld_layer: i32::MAX,
        max_tiles_x: i32::MAX,
        ..PickWorld::default()
    };
    pickaxe_damage(0, if below_surface { 1 } else { 0 }, pick_power, kind, 0, 0, &w, |_, _| None)
        > 0
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The full-damage path with a plain world (no seed flags, the surface
    /// at y 100, the underworld at y 600, a 4200-wide world).
    fn dmg(kind: u16, p: i32, x: i32, y: i32) -> i32 {
        let w = PickWorld {
            world_surface: 100.0,
            underworld_layer: 600,
            max_tiles_x: 4200,
            ..PickWorld::default()
        };
        pickaxe_damage(x, y, p, kind, 0, 0, &w, |_, _| None)
    }

    #[test]
    fn the_copper_pick_mines_the_world_but_not_the_gated_tiles() {
        // The copper pick's 35: the plain ores/dirt/stone pass.
        for kind in [0u16, 1, 2, 6, 7, 8, 9, 5] {
            assert!(pick_can_mine(kind, 35, true), "kind {kind}");
        }
        // The gates: the meteorite 50, the ebonstone 65, the hellstone 65,
        // the obsidian 55, the dungeon 100 below the surface.
        assert!(!pick_can_mine(37, 35, false));
        assert!(pick_can_mine(37, 50, false));
        assert!(!pick_can_mine(25, 35, true));
        assert!(pick_can_mine(25, 65, true));
        assert!(!pick_can_mine(58, 35, false));
        assert!(pick_can_mine(58, 65, false));
        assert!(!pick_can_mine(56, 35, true));
        assert!(!pick_can_mine(41, 35, true));
        assert!(pick_can_mine(41, 100, true));
        // The demonite gate only bites below the surface.
        assert!(pick_can_mine(22, 35, false));
        assert!(!pick_can_mine(22, 35, true));
        assert!(pick_can_mine(22, 55, true));
    }

    #[test]
    fn the_axe_and_hammer_tile_sets() {
        assert!(is_axe_tile(5));
        assert!(!is_axe_tile(1));
        // The gem trees joined tileAxe (`Main.cs:7216-7223`).
        assert!(is_axe_tile(583));
        assert!(is_axe_tile(589));
        // The hammer tiles: the altar 26, the orb 31, the hearts 695/696.
        assert!(is_hammer_tile(26));
        assert!(is_hammer_tile(31));
        assert!(is_hammer_tile(695));
        assert!(!is_hammer_tile(1));
    }

    /// S1-P-i30 — `Main.tileAxe` is the FULL 16-kind table
    /// (`Main.cs:7208-7223`, every `tileAxe[..] = true` assignment): the
    /// pre-fix port list dropped the gem trees 583-589, so the mining gate
    /// refused an axe on a gem tree. WRITTEN, NOT RUN (code-only mode).
    #[test]
    fn s1_p_i30_the_axe_set_is_the_full_vanilla_table() {
        for kind in [5u16, 72, 80, 488, 704, 323, 596, 616, 634, 589, 584, 588, 586, 587, 585, 583]
        {
            assert!(is_axe_tile(kind), "Main.tileAxe[{kind}]");
        }
        // The non-axe kinds stay out (the mushroom plants 71 are not the
        // mushroom tree 72; the dirt/stone/sand are pick tiles; the gemcorn
        // saplings 590 are placement tiles).
        for kind in [0u16, 1, 2, 71, 53, 590] {
            assert!(!is_axe_tile(kind), "not tileAxe[{kind}]");
        }
    }

    /// S1-P-i30 — the axe damage ladder (`Player.cs:45708-45712`): the f32
    /// 1.2 scale with int truncation, the cactus 80's integer triple FIRST,
    /// and the getGoodWorld f64 1.3 on the summed value. WRITTEN, NOT RUN.
    #[test]
    fn s1_p_i30_the_axe_damage_ladder() {
        // The Iron Axe (axe 9, `Item.cs:1524`): (int)(9f * 1.2f) = 10.
        assert_eq!(axe_damage(5, 9, 0, false), 10);
        // The cactus triple: (int)((9 * 3)f * 1.2f) = (int)32.4 = 32.
        assert_eq!(axe_damage(80, 9, 0, false), 32);
        // Lucy the Axe (axe 30, `Item.cs:40832`): (int)36.0 = 36; the
        // cactus (int)(90f * 1.2f) = (int)108.0 = 108 — one hit breaks it.
        assert_eq!(axe_damage(5, 30, 0, false), 36);
        assert_eq!(axe_damage(80, 30, 0, false), 108);
        // The getGoodWorld 1.3 on the SUM: (int)(10.0 * 1.3) = 13; the
        // cactus (int)(32.0 * 1.3) = (int)41.6 = 41.
        assert_eq!(axe_damage(5, 9, 0, true), 13);
        assert_eq!(axe_damage(80, 9, 0, true), 41);
        // The base adds BEFORE the scale of the axe term but the 1.3 hits
        // the whole sum (the C# `num2` carry-in — the noFail head is 0 for
        // every axe tile, kept for the shape).
        assert_eq!(axe_damage(5, 9, 100, false), 110);
        assert_eq!(axe_damage(5, 9, 100, true), 143);
        // The Cobalt Chainsaw (axe 14, `Item.cs:5840`): (int)16.8 = 16.
        assert_eq!(axe_damage(5, 14, 0, false), 16);
        // The Chlorophyte Chainsaw (axe 23, `Item.cs:15241`): (int)27.6 = 27.
        assert_eq!(axe_damage(5, 23, 0, false), 27);
    }

    /// S1-P-i29 — the `GetPickaxeDamage` multiplier ladder
    /// (`Player.cs:53909`): stone takes the plain power, the soft tiles a
    /// double add, the ores the integer-divided fractions, the spikes and
    /// clouds double.
    #[test]
    fn s1_p_i29_the_damage_multiplier_ladder() {
        let p = 100;
        assert_eq!(dmg(1, p, 2100, 200), 100, "stone: plain power");
        assert_eq!(dmg(0, p, 2100, 200), 200, "dirt: the second add (:53972)");
        assert_eq!(dmg(53, p, 2100, 200), 200, "sand: the second add");
        assert_eq!(dmg(59, p, 2100, 200), 200, "mud: the second add");
        assert_eq!(dmg(147, p, 2100, 200), 200, "snow: the second add");
        assert_eq!(dmg(404, p, 2100, 200), 105, "desert fossil: +5 (:53976)");
        assert_eq!(dmg(58, p, 2100, 200), 50, "hellstone: p/2");
        assert_eq!(dmg(25, p, 2100, 200), 50, "ebonstone: p/2");
        assert_eq!(dmg(117, p, 2100, 200), 50, "pearlstone: p/2");
        assert_eq!(dmg(41, p, 2100, 200), 50, "dungeon brick: p/2 (mid band)");
        assert_eq!(dmg(107, p, 2100, 200), 50, "cobalt: p/2");
        assert_eq!(dmg(221, p, 2100, 200), 50, "palladium: p/2");
        assert_eq!(dmg(108, 110, 2100, 200), 36, "mythril: p/3 (110/3=36)");
        assert_eq!(dmg(222, 110, 2100, 200), 36, "orichalcum: p/3");
        assert_eq!(dmg(111, 150, 2100, 200), 37, "adamantite: p/4 (150/4=37)");
        assert_eq!(dmg(223, 150, 2100, 200), 37, "titanium: p/4");
        assert_eq!(dmg(211, 200, 2100, 200), 40, "chlorophyte: p/5");
        assert_eq!(dmg(226, 210, 2100, 200), 52, "lihzahrd brick: p/4 (210/4=52)");
        assert_eq!(dmg(48, p, 2100, 200), 200, "spikes: p*2");
        assert_eq!(dmg(232, p, 2100, 200), 200, "wooden spikes: p*2");
        assert_eq!(dmg(189, p, 2100, 200), 200, "cloud: p*2");
        assert_eq!(dmg(85, p, 2100, 200), 100, "tombstone: plain power");
    }

    /// S1-P-i29 — the zero-damage gates the old `pick_can_mine` missed:
    /// chlorophyte 200, mythril 110, adamantite 150, lihzahrd 210, the
    /// hellforge underworld gate, the trap rows and the dungeon x band.
    #[test]
    fn s1_p_i29_the_missing_tier_gates() {
        // Chlorophyte needs 200 (`:53910`).
        assert_eq!(dmg(211, 190, 2100, 200), 0);
        assert!(dmg(211, 200, 2100, 200) > 0);
        // Mythril/orichalcum need 110 (`:53964`), adamantite/titanium 150
        // (`:53968`).
        assert_eq!(dmg(108, 100, 2100, 200), 0);
        assert!(dmg(108, 110, 2100, 200) > 0);
        assert_eq!(dmg(222, 100, 2100, 200), 0);
        assert_eq!(dmg(111, 110, 2100, 200), 0);
        assert!(dmg(111, 150, 2100, 200) > 0);
        assert_eq!(dmg(223, 110, 2100, 200), 0);
        // Lihzahrd brick/altar need 210 (`:53942`).
        assert_eq!(dmg(226, 200, 2100, 200), 0);
        assert!(dmg(226, 210, 2100, 200) > 0);
        assert_eq!(dmg(237, 200, 2100, 200), 0);
        // The hellforge gate only bites in the underworld (`:53934`).
        assert!(dmg(77, 35, 2100, 200) > 0, "above the underworld: free");
        assert_eq!(dmg(77, 35, 2100, 600), 0, "underworld: needs 65");
        assert!(dmg(77, 65, 2100, 600) > 0);
        // The dungeon band (`:53953-53958`): below the surface, outside
        // 0.35..0.65 of the width, needs 100; inside the band it is free.
        assert_eq!(dmg(41, 35, 100, 200), 0, "x < 0.35*4200");
        assert_eq!(dmg(41, 35, 4100, 200), 0, "x > 0.65*4200");
        assert!(dmg(41, 35, 2100, 200) > 0, "the middle band is free");
        assert!(dmg(41, 35, 100, 50) > 0, "above the surface: free");
        // The depth-only helper keeps the pre-i29 dungeon approximation
        // (no x band — the full path above carries it).
        assert!(!pick_can_mine(41, 35, true));
        assert!(pick_can_mine(41, 100, true));
        // The trap rows (`:53946-53951`): frameY/18 1..=4 need 210.
        let w = PickWorld {
            world_surface: 100.0,
            underworld_layer: 600,
            max_tiles_x: 4200,
            ..PickWorld::default()
        };
        for fy in [18, 36, 54, 72] {
            assert_eq!(
                pickaxe_damage(2100, 200, 100, 137, 0, fy, &w, |_, _| None),
                0,
                "trap frameY {fy}"
            );
        }
        assert!(
            pickaxe_damage(2100, 200, 100, 137, 0, 0, &w, |_, _| None) > 0,
            "frameY 0 is not a gated row"
        );
        assert!(
            pickaxe_damage(2100, 200, 210, 137, 0, 18, &w, |_, _| None) > 0,
            "210 clears the trap gate"
        );
    }

    /// S1-P-i29 — the instant-break rows: noFail 100, the ropes, the
    /// stalactite and the crimson grass (`:53905-53908`, `:53980-53983`).
    #[test]
    fn s1_p_i29_the_instant_break_rows() {
        // Gate V3: the noFail head sets num=100, then the multiplier ladder
        // ADDS pickPower (`:53909` else arm `num + pickPower`), so a noFail
        // tile at power 1 answers 101 — still an instant break (>= 100). The
        // rope/stalactite/crimson arm (`:53980-53983`) OVERWRITES num=100.
        assert_eq!(dmg(4, 1, 2100, 200), 101, "torch: noFail 100 + power 1");
        assert_eq!(dmg(3, 1, 2100, 200), 101, "grass: noFail 100 + power 1");
        assert_eq!(dmg(213, 1, 2100, 200), 100, "rope");
        assert_eq!(dmg(365, 1, 2100, 200), 100, "silk rope");
        assert_eq!(dmg(165, 1, 2100, 200), 100, "stalactite");
        assert_eq!(dmg(199, 1, 2100, 200), 100, "crimson grass");
        // The mystic snake rope 504 is a rope — but PickTile refuses it
        // outright (`:53760-53763`); the damage model still answers 100.
        assert_eq!(dmg(504, 1, 2100, 200), 100);
    }

    /// S1-P-i29 — the getGoodWorld arms: the tombstone quarter-power
    /// (`:53909`) and the ×2 damage (`PickTile_DetermineDamage :53836-53839`
    /// — the caller doubles; the ladder itself only changes the tombstone).
    #[test]
    fn s1_p_i29_the_get_good_world_tombstone() {
        let w = PickWorld {
            world_surface: 100.0,
            underworld_layer: 600,
            max_tiles_x: 4200,
            get_good: true,
            ..PickWorld::default()
        };
        assert_eq!(
            pickaxe_damage(2100, 200, 100, 85, 0, 0, &w, |_, _| None),
            25,
            "the tombstone takes p/4 on getGoodWorld"
        );
    }

    /// S1-P-i29 — the mannequin redirect (`:53984-53997`): the right-half
    /// cell reads the left cell's frames; a dressed mannequin refuses.
    #[test]
    fn s1_p_i29_the_mannequin_redirect() {
        let w = PickWorld::default();
        let left = crate::tile::Tile {
            kind: 128,
            frame_x: 100, // dressed
            ..Default::default()
        };
        // The right-half cell (frameX 18) redirects to the dressed left
        // cell → zero damage.
        assert_eq!(
            pickaxe_damage(10, 10, 100, 128, 18, 0, &w, |x, _| {
                (x == 9).then_some(left)
            }),
            0
        );
        // The undressed left cell keeps the plain power.
        let bare = crate::tile::Tile {
            kind: 128,
            frame_x: 0,
            ..Default::default()
        };
        assert_eq!(
            pickaxe_damage(10, 10, 100, 128, 18, 0, &w, |x, _| {
                (x == 9).then_some(bare)
            }),
            100
        );
    }
}
