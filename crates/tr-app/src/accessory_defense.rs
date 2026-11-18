//! S1-C-12 wave C — the DEFENSE/IMMUNITY accessory family's
//! `ApplyEquipFunctional` rows (`Player.cs:14368-15565`), kept in a module
//! disjoint from wave A (movement, `player.rs`) and wave B (combat,
//! `accessory_combat.rs`).
//!
//! What lands here (the rows the port did not carry yet):
//! * the Obsidian Shield 397 (`Player.cs:15209-15213`): `noKnockback` +
//!   `fireWalk`;
//! * the Paladin's/Frozen/Hero shields 938/3997/3998 (`:15348-15353`):
//!   `noKnockback` always, `hasPaladinShield` only above 25% life;
//! * the Silver Shield 6183 (`:14605-14609`) and the Restoration Shield
//!   6188 (`:14629-14631`): the knockback/fire-walk halves;
//! * the Ankh Shield 1613's `noKnockback` + `fireWalk` (`:15552-15554` —
//!   its eleven `buffImmune` rows already ride `BUFF_IMMUNITY_ROWS`,
//!   `player.rs:401-420`);
//! * the `fireWalk` granters: the Obsidian Skull 193 (`:15130-15133`), the
//!   Obsidian Horseshoe 396 (`:15203-15208`), the Obsidian Water Walking
//!   Boots 907 (`:15158-15162`), the Lava Waders 908 / Terraspark Boots
//!   5000 (`:15167-15173`), the Molten Charm 4038 (`:14448-14450`), the
//!   Molten Skull Rose 4003 (`:14451-14454`), the Obsidian Skull Rose 4004
//!   (`:14469-14472`) and the Lava Skull 3999 (`:14473-14475`).
//!
//! Already ported elsewhere (pinned by the tests below, NOT re-granted
//! here): the Cobalt Shield 156's `noKnockback` (`player.rs:4129`,
//! `Player.cs:15025-15028`), every `buffImmune[...]` accessory row — the
//! Hand Warmer 1921, Pocket Mirror 3781, Blindfold 888, Nazar 891,
//! Megaphone 890, Armor Polish 886, Vitamins 892, Bezoar 887, Adhesive
//! Bandage 885, Fast Clock 889, Trifold Map 893, the 901-904 combos, the
//! Ankh Charm/Shield 1612/1613 and the Reflective Shades 5354
//! (`BUFF_IMMUNITY_ROWS`, `Player.cs:15463-15564`) — and every worn
//! item's `Item.defense` (`GrantArmorBenefits`, `Player.cs:13121`, the
//! generic sum at `player.rs:3595-3601`).
//!
//! Deliberately out (docs/ANALYSIS.md "S1-C-12 wave C"): the
//! `accSilverBracer` flag's whip-tag consumers (`Player.cs:12970-12974`,
//! wave B territory), the Frozen Shield's half-life Ice Barrier buff
//! (`:14888-14891`, buff arms = E-7), the `waterWalk`/`lavaRose` halves
//! of the boot rows (movement = wave A / unported), the Paladin's
//! Shield multiplayer team-dust arm (`:15354-15365`, single player) and
//! the Shimmer Cloak 5355's `shimmerImmune` (`:15528-15531`, no shimmer
//! float machine). The "palladium/mythril/orichalcum/titanium/chlorophyte
//! shields" and a "Plantera bulb" accessory do not exist in
//! `ItemID.cs` — the real family is the one above.
//!
//! Order: the C# runs these rows interleaved per slot inside
//! `UpdateEquips`' 3..10 loop (`Player.cs:14602-14604`); every row here
//! is a boolean OR grant with no RNG draw, so a second pass after the
//! main accessory match is behaviour-identical.

use crate::player::Player;

/// The wave-C hook: one `ApplyEquipFunctional` pass over the seven
/// functional accessory slots (`Player.cs:14602-14604`), applying only
/// the defense/immunity rows listed in the module doc. Called from
/// `Player::update` right after the main accessory match.
pub fn apply_defense_accessories(p: &mut Player) {
    for &kind in &p.equipped[3..10] {
        match kind {
            // `Player.cs:15209-15213` — the Obsidian Shield: the Cobalt
            // Shield's knockback immunity plus the Obsidian Skull's fire
            // walk. Its +2 defense rides the generic `Item.defense` sum
            // (`Item.cs:6038`, `Player.cs:13121`).
            397 => {
                p.no_knockback = true;
                p.fire_walk = true;
            }
            // `Player.cs:15348-15353` — the Paladin's Shield (938), the
            // Frozen Shield (3997) and the Hero Shield (3998):
            // `noKnockback` unconditionally; `hasPaladinShield` only
            // while `statLife > statLifeMax2 * 0.25f` (strict >). The
            // multiplayer team-dust arm (`:15354-15365`) is out; the
            // hurt-side team redirect (`:38190`) rides C-7.
            938 | 3997 | 3998 => {
                p.no_knockback = true;
                if p.hp as f32 > p.max_hp as f32 * 0.25 {
                    p.has_paladin_shield = true;
                }
            }
            // `Player.cs:14605-14609` — the Silver Shield: knockback
            // immunity + fire walk. The `accSilverBracer` flag's whip-tag
            // consumers (`:12970-12974`) ride wave B.
            6183 => {
                p.no_knockback = true;
                p.fire_walk = true;
            }
            // `Player.cs:14629-14631` — the Restoration Shield: knockback
            // immunity. Its `lifeRegen = 2` rides the generic
            // `Item.lifeRegen` loop (`player.rs:4181-4195`).
            6188 => p.no_knockback = true,
            // `Player.cs:15552-15554` — the Ankh Shield's non-immunity
            // half: knockback immunity + fire walk (the eleven
            // `buffImmune` rows are `BUFF_IMMUNITY_ROWS`' 1613 entry).
            1613 => {
                p.no_knockback = true;
                p.fire_walk = true;
            }
            // The bare `fireWalk` granters: the Obsidian Skull
            // (`:15130-15133`), the Obsidian Horseshoe (`:15203-15208`,
            // its `noFallDmg`/luck halves ride wave A), the Obsidian
            // Water Walking Boots (`:15158-15162`, `waterWalk2` = A),
            // the Lava Waders / Terraspark Boots (`:15167-15173`, the
            // `lavaMax += 420` half already rides `player.rs:4110-4113`),
            // the Molten Charm (`:14448-14450`), the Molten Skull Rose
            // (`:14451-14454`), the Obsidian Skull Rose (`:14469-14472`)
            // and the Lava Skull (`:14473-14475`). `lavaRose` has no
            // ported consumer and stays out.
            193 | 396 | 907 | 908 | 5000 | 3999 | 4003 | 4004 | 4038 => {
                p.fire_walk = true;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::PlayerInput;
    use tr_core::world::World;

    fn rig() -> (World, Player) {
        let w = World::new("t".into(), 64, 800);
        let p = Player::spawn(&w);
        (w, p)
    }

    fn equip(w: &World, p: &mut Player, kind: u16) {
        p.equipped[3] = kind;
        p.update(w, PlayerInput::default());
    }

    /// `Player.cs:15025-15028` + `Item.cs:3249` — the Cobalt Shield's
    /// knockback immunity and +1 defense (the pre-existing port, pinned).
    #[test]
    fn the_cobalt_shield_grants_knockback_immunity_and_one_defense() {
        let (w, mut p) = rig();
        equip(&w, &mut p, 156);
        assert!(p.no_knockback);
        assert!(!p.fire_walk, "the Cobalt Shield is not the Obsidian one");
        assert_eq!(p.stat_defense, 1);
    }

    /// `Player.cs:15209-15213` + `Item.cs:6038` — the Obsidian Shield.
    #[test]
    fn the_obsidian_shield_grants_knockback_immunity_and_fire_walk() {
        let (w, mut p) = rig();
        equip(&w, &mut p, 397);
        assert!(p.no_knockback);
        assert!(p.fire_walk);
        assert_eq!(p.stat_defense, 2);
    }

    /// `Player.cs:15348-15353` — the three paladin-family shields: the
    /// flag is life-gated at a strict `> 25%`, the immunity is not.
    #[test]
    fn the_paladin_family_shields_gate_the_flag_at_a_quarter_life() {
        for (kind, defense) in [(938u16, 6i32), (3997, 6), (3998, 10)] {
            let (w, mut p) = rig();
            equip(&w, &mut p, kind);
            assert!(p.no_knockback, "{kind}");
            assert!(p.has_paladin_shield, "{kind} at full life");
            assert_eq!(p.stat_defense, defense, "{kind} Item.defense");
            // Exactly 25% of the 100 max: `>` is strict, the flag stays off.
            p.hp = 25;
            p.update(&w, PlayerInput::default());
            assert!(p.no_knockback, "{kind} immunity is not life-gated");
            assert!(!p.has_paladin_shield, "{kind} at exactly 25%");
            p.hp = 26;
            p.update(&w, PlayerInput::default());
            assert!(p.has_paladin_shield, "{kind} just above 25%");
        }
    }

    /// `Player.cs:14605-14609` + `Item.cs:47616-47620` — the Silver Shield.
    #[test]
    fn the_silver_shield_grants_knockback_immunity_and_fire_walk() {
        let (w, mut p) = rig();
        equip(&w, &mut p, 6183);
        assert!(p.no_knockback);
        assert!(p.fire_walk);
        assert_eq!(p.stat_defense, 2);
    }

    /// `Player.cs:14629-14631` + `Item.cs:47642-47647` — the Restoration
    /// Shield: knockback immunity and +1 defense (its `lifeRegen = 2`
    /// rides the private `accessory_life_regen` loop, `player.rs` tests).
    #[test]
    fn the_restoration_shield_grants_knockback_immunity() {
        let (w, mut p) = rig();
        equip(&w, &mut p, 6188);
        assert!(p.no_knockback);
        assert!(!p.fire_walk);
        assert_eq!(p.stat_defense, 1);
    }

    /// `Player.cs:15550-15564` — the Ankh Shield: the eleven immunities
    /// (the pre-existing table) plus the new knockback/fire-walk half.
    #[test]
    fn the_ankh_shield_grants_the_full_immunity_bundle_and_fire_walk() {
        let (w, mut p) = rig();
        equip(&w, &mut p, 1613);
        assert!(p.no_knockback);
        assert!(p.fire_walk);
        for id in [46, 33, 36, 30, 20, 32, 31, 35, 23, 22, 156] {
            assert!(p.buffs.buff_immune[id], "the ankh row {id}");
        }
        assert!(!p.buffs.buff_immune[24], "On Fire is not an ankh immunity");
        // The Ankh Charm (1612, `:15537-15548`) has NO knockback/fire half.
        p.equipped[3] = 1612;
        p.update(&w, PlayerInput::default());
        assert!(!p.no_knockback);
        assert!(!p.fire_walk);
        assert!(p.buffs.buff_immune[156]);
    }

    /// `Player.cs:15463-15501` — the single-immunity accessories, one
    /// debuff each (the pre-existing `BUFF_IMMUNITY_ROWS`, pinned per
    /// accessory): Armor Polish 886 → Broken Armor 36, Bezoar 887 →
    /// Poisoned 20, Blindfold 888 → Darkness 22, Megaphone 890 → Silenced
    /// 35, Nazar 891 → Cursed 23, Vitamins 892 → Weak 33, Pocket Mirror
    /// 3781 → Stoned 156.
    #[test]
    fn the_single_immunity_accessories_block_exactly_their_debuff() {
        for (kind, immune) in [
            (886u16, 36usize),
            (887, 20),
            (888, 22),
            (890, 35),
            (891, 23),
            (892, 33),
            (3781, 156),
        ] {
            let (w, mut p) = rig();
            equip(&w, &mut p, kind);
            assert!(p.buffs.buff_immune[immune], "{kind} → {immune}");
            // Exactly one immunity: the rest of the ankh list stays open.
            let count = p.buffs.buff_immune.iter().filter(|b| **b).count();
            assert_eq!(count, 1, "{kind} grants only buff {immune}");
            // The gate works: the debuff never lands (`Player.cs:5249`).
            p.buffs.add_buff(immune as i32, 600);
            assert_eq!(p.buffs.find_buff_index(immune as i32), None);
        }
    }

    /// `Player.cs:15532-15536` — the Hand Warmer: Chilled 46 + Frozen 47,
    /// and nothing else (On Fire 24 stays open).
    #[test]
    fn the_hand_warmer_blocks_chilled_and_frozen() {
        let (w, mut p) = rig();
        equip(&w, &mut p, 1921);
        assert!(p.buffs.buff_immune[46]);
        assert!(p.buffs.buff_immune[47]);
        assert!(!p.buffs.buff_immune[24]);
        p.buffs.add_buff(46, 600);
        assert_eq!(p.buffs.find_buff_index(46), None);
    }

    /// The `fireWalk` granters (`Player.cs:14448-14475`, `:15130-15133`,
    /// `:15158-15173`, `:15203-15208`): each fires the flag alone.
    #[test]
    fn the_fire_walk_family_each_grant_the_flag() {
        for kind in [193u16, 396, 907, 908, 5000, 3999, 4003, 4004, 4038] {
            let (w, mut p) = rig();
            equip(&w, &mut p, kind);
            assert!(p.fire_walk, "{kind} grants fireWalk");
        }
        // The Lava Waders' 420-tick lava budget still rides the main
        // match (`player.rs:4110-4113`) — the second pass adds no double.
        let (w, mut p) = rig();
        equip(&w, &mut p, 908);
        assert_eq!(p.lava_max, 420);
    }

    /// `ResetEffects` (`Player.cs:18355-18356`, `:18777`): taking the
    /// accessory off ends every grant on the next tick.
    #[test]
    fn unequipping_clears_the_grants_next_tick() {
        let (w, mut p) = rig();
        equip(&w, &mut p, 397);
        assert!(p.no_knockback && p.fire_walk);
        p.equipped[3] = 0;
        p.update(&w, PlayerInput::default());
        assert!(!p.no_knockback);
        assert!(!p.fire_walk);
        equip(&w, &mut p, 938);
        assert!(p.has_paladin_shield);
        p.equipped[3] = 0;
        p.update(&w, PlayerInput::default());
        assert!(!p.has_paladin_shield);
    }

    /// The consumer seam: `Collision.CanTileHurt`'s hot arm
    /// (`Collision.cs:3220`) through `apply_touch_damage` — the Obsidian
    /// Shield's `fireWalk` stops the hellstone's burn, and the skipped
    /// scan draws NO random numbers (the determinism invariant).
    #[test]
    fn fire_walk_stops_the_hellstone_burn_without_touching_the_rng() {
        use tr_core::tile::FLAG_ACTIVE;
        let mut w = World::new("hot".into(), 60, 200);
        let i = w.index(11, 30);
        w.tiles[i].kind = 58; // TileID.Hellstone — Sets.TouchDamageHot
        w.tiles[i].flags |= FLAG_ACTIVE;
        // Baseline: no accessory, the burning debuff lands.
        let mut p = Player::spawn(&w);
        p.x = 11.0 * 16.0;
        p.y = 30.0 * 16.0 - 30.0;
        let mut rng = tr_core::UnifiedRandom::new(0);
        let height = p.body_height() as i32;
        let out = crate::touch_damage::apply_touch_damage(
            &mut w,
            &mut p,
            &mut rng,
            crate::player::WIDTH as i32,
            height,
        );
        assert!(out.tile.is_some(), "the hellstone is a hurt tile");
        assert!(p.buffs.find_buff_index(67).is_some(), "Burning lands");
        // With the Obsidian Shield: the scan finds nothing, no buff, and
        // the stream is byte-identical to its pristine state.
        let mut p = Player::spawn(&w);
        p.x = 11.0 * 16.0;
        p.y = 30.0 * 16.0 - 30.0;
        equip(&w, &mut p, 397);
        p.x = 11.0 * 16.0;
        p.y = 30.0 * 16.0 - 30.0;
        let mut rng = tr_core::UnifiedRandom::new(0);
        let height = p.body_height() as i32;
        let out = crate::touch_damage::apply_touch_damage(
            &mut w,
            &mut p,
            &mut rng,
            crate::player::WIDTH as i32,
            height,
        );
        assert!(out.tile.is_none(), "fireWalk skips the hot tile");
        assert_eq!(p.buffs.find_buff_index(67), None);
        assert_eq!(rng, tr_core::UnifiedRandom::new(0), "no draws were made");
    }
}
