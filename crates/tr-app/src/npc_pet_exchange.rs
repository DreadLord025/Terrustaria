//! W4-g10 lane 3 FINAL — the pet-license EXCHANGE chain, NPC side, over the
//! LIVE town-pet kinds (`NPC.RerollVariationForNPCType` `NPC.cs:80920-80931`,
//! `NPC.RerollVariation` `:80933-80961`, the `TownNPCProfiles` rows
//! `GameContent/TownNPCProfiles.cs:138-180` + `Profiles.cs:83-86`/`:214-222`,
//! the `Spawn_PetExchange` burst
//! `GameContent/Drawing/ParticleOrchestrator.cs:2691-2700`).
//!
//! The a6/g87 use-arm machine (`Player.ItemCheck_UsePetLicenses` →
//! `LicenseOrExchangePet` → `NPC.UnlockOrExchangePet`, `Player.cs:45352-45365`
//! /`:45438-45445`, `NPC.cs:80894-80918`) owns the bought flags, the gate and
//! the chat; its else-arm posts `Misc.PetExchangeFail` unconditionally today
//! because no kind carried 637/638/656 (research/138 §12.2's activation
//! contract, step 1). This module IS step 1's consumer: the kinds now exist
//! (W4-g10 batches 1/2), so the Success arm is live and witnessable. The
//! merged `use_pet_license` swaps its hardcoded Fail for
//! [`reroll_variation_for_kind`]'s result (contract step 4 — a ONE-LINE
//! conditional, research/138 §12.2).
//!
//! Keep-one (f43): `town_arrival_pets.rs` (branch `task/W4-f43-town-spawn-2`,
//! UNWIRED) ports the same chain as pure fns over a `PetVariationSlot` —
//! `reroll_variation`/`reroll_variation_for_npc_type`/the variant rosters/
//! `name_category_for`. Its own module note says "the wiring lane maps it
//! onto `Npc`"; this module is that mapping. At the merge the shared loop +
//! rosters dedup onto ONE copy (f43's pure fns are the shape owner; this
//! module keeps only the live-`Npc` scan, the heal/name writes and the gore
//! burst f43 leaves to the caller). The rows below are byte-identical to
//! f43's (`TownNPCProfiles.cs:139-147` re-read).
//!
//! RNG census (the whole chain, research/138 §12.2 + f43's module note):
//! * the variation roll is `Main.rand.Next(6)` (`Profiles.cs:216`) — the
//!   ONLY `Main.rand` slot before the burst; the Legacy profiles (the eight
//!   town slimes) roll the CONSTANT 0 with NO draw (`Profiles.cs:83-86`);
//! * the name draw rides `WorldGen.genRand` (`Profiles.cs:221` →
//!   `LanguageManager.RandomFromCategory` `LanguageManager.cs:419`) — a
//!   DIFFERENT stream, injected separately;
//! * the `PetExchange` burst (`NPC.cs:80951-80958`, SP arm) draws 8 per gore
//!   × 13 gores = 104 on `Main.rand`: the position's
//!   `NextVector2Circular(20, 20)` (`Utils.cs:1266-1269` = `NextFloat` angle
//!   + `NextFloat` radius), the type `Next(61, 64)`, the scale
//!   `1 + NextFloat() * 0.3`, `NewGore`'s two velocity jitters
//!   (`Gore.cs:1327-1328`, the 61-63 smoke row is sticky-false `:1341-1344`
//!   and ChildSafety-safe, so no extra draws), then the velocity overwrite's
//!   `(2π·Next()).ToRotationVector2() · NextFloat()` (2 draws).
#![allow(dead_code)] // the a6/g87 use-arm merge wires the dispatch (contract step 4)

use glam::Vec2;
use tr_core::UnifiedRandom;
use tr_core::gore::GorePool;
use tr_core::World;

use crate::npc::{Npc, NpcKind};

/// The outcome of the exchange dispatch — the merged `use_pet_license`
/// else-arm maps Success/Fail to the `Misc.PetExchangeSuccess`/`Fail` chat
/// (`NPC.cs:80910-80917`); `NoResident` is `RerollVariationForNPCType`'s
/// no-match false (`:80930`), which the C# ALSO chats Fail for (the a6 gate
/// `AnyNPCs` already refused that path, so the merged arm never sees it).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExchangeOutcome {
    Success,
    Fail,
    NoResident,
}

impl ExchangeOutcome {
    /// `NPC.UnlockOrExchangePet`'s chat key (`NPC.cs:80912`/`:80916`).
    pub(crate) fn chat_key(self) -> &'static str {
        match self {
            ExchangeOutcome::Success => "Misc.PetExchangeSuccess",
            ExchangeOutcome::Fail | ExchangeOutcome::NoResident => "Misc.PetExchangeFail",
        }
    }
}

/// `TownNPCProfiles`' variant rosters (`TownNPCProfiles.cs:139-147`): the
/// texture-name order IS the variation-index order. f43 keep-one: identical
/// to `town_arrival_pets.rs`'s `CAT_VARIANTS`/`DOG_VARIANTS`/`BUNNY_VARIANTS`.
const CAT_VARIANTS: [&str; 6] = ["Siamese", "Black", "OrangeTabby", "RussianBlue", "Silver", "White"];
const DOG_VARIANTS: [&str; 6] = ["Labrador", "PitBull", "Beagle", "Corgi", "Dalmation", "Husky"];
const BUNNY_VARIANTS: [&str; 6] = ["White", "Angora", "Dutch", "Flemish", "Lop", "Silver"];

/// The `VariantNPCProfile` roll bound — `Some(6)` for the three license pets,
/// `None` for the eight town slimes (`LegacyNPCProfile.RollVariation` returns
/// the constant 0, `Profiles.cs:83-86`; every pet HAS a profile, so the
/// `GetProfile` gate `NPC.cs:80935-80938` passes for all eleven kinds).
fn variant_count(kind: NpcKind) -> Option<i32> {
    match kind {
        NpcKind::TownCat | NpcKind::TownDog | NpcKind::TownBunny => Some(6),
        _ => None,
    }
}

/// `Profiles.VariantNPCProfile.GetNameForVariant` (`Profiles.cs:219-222`) /
/// `LegacyNPCProfile.GetNameForVariant` → `NPC.getNewNPCName`
/// (`NPC.cs:7312-7322`): the license pets key on the CURRENT variation, the
/// town slimes on their fixed colour. The draw itself is
/// `WorldGen.genRand.Next(category_count)` (`LanguageManager.cs:419`).
fn name_category(kind: NpcKind, variation: i32) -> Option<String> {
    let (base, variants): (&str, &[&str]) = match kind {
        NpcKind::TownCat => ("CatNames", &CAT_VARIANTS),
        NpcKind::TownDog => ("DogNames", &DOG_VARIANTS),
        NpcKind::TownBunny => ("BunnyNames", &BUNNY_VARIANTS),
        NpcKind::TownSlimeBlue => return Some("SlimeNames_Blue".into()),
        NpcKind::TownSlimeGreen => return Some("SlimeNames_Green".into()),
        NpcKind::TownSlimeOld => return Some("SlimeNames_Old".into()),
        NpcKind::TownSlimePurple => return Some("SlimeNames_Purple".into()),
        NpcKind::TownSlimeRainbow => return Some("SlimeNames_Rainbow".into()),
        NpcKind::TownSlimeRed => return Some("SlimeNames_Red".into()),
        NpcKind::TownSlimeYellow => return Some("SlimeNames_Yellow".into()),
        NpcKind::TownSlimeCopper => return Some("SlimeNames_Copper".into()),
        _ => return None,
    };
    let v = variants.get(variation as usize)?;
    Some(format!("{base}_{v}"))
}

/// `ParticleOrchestrator.Spawn_PetExchange`
/// (`ParticleOrchestrator.cs:2691-2700`): 13 smoke gores (type `Next(61, 64)`
/// = `GoreID.Smoke1..3`, `GoreID.cs:45-49`), alpha 100, the velocity
/// overwrite after the spawn. Every draw rides the caller's `Main.rand`
/// (the burst runs inside `RerollVariation`, `NPC.cs:80953`, under no
/// `SwapRandom` block — the bare stream). 8 draws per gore, 104 total.
fn spawn_pet_exchange_burst(
    gore: &mut GorePool,
    world: &World,
    center: Vec2,
    movement: Vec2,
    rng: &mut UnifiedRandom,
) {
    for _ in 0..13 {
        // `NextVector2Circular(20, 20)` (`Utils.cs:1266-1269`):
        // `NextVector2Unit()` (one NextFloat angle) * (20, 20) * NextFloat.
        let angle = rng.next_float() * std::f32::consts::PI * 2.0;
        let unit = Vec2::new(angle.cos(), angle.sin());
        let radius = rng.next_float();
        let offset = unit * Vec2::new(20.0, 20.0) * radius;
        let position = center + Vec2::new(-20.0, -20.0) + offset;
        let gore_type = rng.next_range(61, 64);
        let scale = 1.0 + rng.next_float() * 0.3;
        // `Gore.NewGoreDirect(position, Vector2.Zero, type, scale)`.
        let slot = gore.new_gore_direct(rng, world, position, Vec2::ZERO, gore_type, scale);
        // `(2π·Next()).ToRotationVector2() * NextFloat() + MovementVector*0.5`
        // — the raw `Next()` (NOT NextFloat), `Utils.cs:1665-1668`. The C#
        // draws these EVEN on a refused spawn (it writes `Main.gore[600]`),
        // so the stream census keeps them outside the sentinel guard.
        let rot = std::f32::consts::PI * 2.0 * rng.next() as f32;
        let speed = rng.next_float();
        if slot != tr_core::gore::GORE_SENTINEL {
            let g = gore.slot_mut(slot);
            g.alpha = 100;
            g.velocity = Vec2::new(rot.cos(), rot.sin()) * speed + movement * 0.5;
        }
    }
}

/// `NPC.RerollVariationForNPCType` + `NPC.RerollVariation`
/// (`NPC.cs:80920-80961`) over the LIVE roster: the FIRST active NPC of the
/// kind rerolls (`:80922-80928` — slot order, one resident only), the
/// ≤100-draw loop keeps drawing while the index is unchanged (`:80941-80944`),
/// success writes the new index + `GivenName` + `life = lifeMax`
/// (`:80949-80950`) and fires the PetExchange burst (`:80951-80958`, SP).
/// `roll_rng` is `Main.rand`, `name_rng` is `WorldGen.genRand` — two streams
/// (the f43 census). `gore` is `None` on a headless caller (the burst's 104
/// draws then DO NOT run — the witness passes a pool so the stream census
/// stays honest; the production caller always has `app.gore`).
pub(crate) fn reroll_variation_for_kind(
    npcs: &mut [Npc],
    kind: NpcKind,
    roll_rng: &mut UnifiedRandom,
    name_rng: &mut UnifiedRandom,
    mut gore: Option<(&mut GorePool, &World)>,
) -> ExchangeOutcome {
    let net_id = kind.net_id();
    // `:80922-80928` — the first active resident of the type; the port's
    // liveness is the hp (the `npc_chat.rs` convention).
    let Some(slot) = npcs.iter_mut().find(|n| n.hp > 0 && n.kind.net_id() == net_id) else {
        return ExchangeOutcome::NoResident; // `:80930`
    };
    // `:80935-80938` — every town pet HAS a profile (the eleven kinds are
    // exactly the profile-bearing pets), so the gate passes here.
    let old = slot.variation_index;
    let mut index = old;
    match variant_count(kind) {
        // `VariantNPCProfile.RollVariation` = `Main.rand.Next(6)`
        // (`Profiles.cs:214-217`); `while (num2++ < 100 && index == num)`.
        Some(count) => {
            for _ in 0..100 {
                if index != old {
                    break;
                }
                index = roll_rng.next_max(count);
            }
        }
        // `LegacyNPCProfile.RollVariation` returns 0 with NO draw
        // (`Profiles.cs:83-86`): the loop spins 100 times drawing nothing.
        None => {
            for _ in 0..100 {
                if index != old {
                    break;
                }
                index = 0;
            }
        }
    }
    if index == old {
        return ExchangeOutcome::Fail; // `:80945-80948`
    }
    slot.variation_index = index;
    // `:80949` — `GivenName = profile.GetNameForVariant(this)`: the
    // genRand category draw (a DIFFERENT stream from the roll).
    if let Some(category) = name_category(kind, index) {
        let size = crate::lang::category_size(&category);
        if size > 0 {
            let draw = name_rng.next_max(size as i32) as usize;
            slot.given_name = crate::lang::random_from_category(&category, draw);
        }
    }
    // `:80950` — the exchange heals to full.
    slot.hp = slot.max_hp;
    // `:80951-80958` — the SP PetExchange burst (netMode != 1).
    if let Some((pool, world)) = gore {
        let (cx, cy) = slot.center();
        spawn_pet_exchange_burst(
            pool,
            world,
            Vec2::new(cx, cy),
            Vec2::new(slot.vx, slot.vy),
            roll_rng,
        );
    }
    // `:80959` — `TrySendData(56)` is MP-only (the port is SP).
    ExchangeOutcome::Success
}

#[cfg(test)]
mod tests {
    //! W4-g10 lane 3 FINAL (code-only): the exchange-chain witnesses,
    //! WRITTEN NOT RUN. These are the POSITIVE Success witnesses the a6/g87
    //! lanes could not write pre-activation (research/138 §12.2's boundary:
    //! "the positive Success witness lands WITH step 1 (a kind carrying
    //! 637/638/656)") — the kinds now exist, so the chain is witnessed end
    //! to end against mirrored `UnifiedRandom` streams.
    use super::*;

    fn init_lang() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        crate::lang::init(tr_core::localization::GameCulture::English);
    }

    /// A resident pet at a known position, hp below max (the heal witness).
    fn pet(kind: NpcKind, variation: i32) -> Npc {
        let mut n = Npc::new(kind, 320.0, 480.0, 1, 0.0);
        n.variation_index = variation;
        n.hp = 10;
        n.vx = 2.0;
        n.vy = -1.0;
        n
    }

    fn world() -> World {
        World::new("t".into(), 64, 64)
    }

    /// `NPC.cs:80922-80930` — no resident of the type: false, no draws.
    #[test]
    fn w4_g10_no_resident_answers_no_resident_and_draws_nothing() {
        let mut npcs = [pet(NpcKind::TownDog, 0)];
        let mut roll = UnifiedRandom::new(7);
        let mut name = UnifiedRandom::new(9);
        let before = (roll.peek(), name.peek());
        let out = reroll_variation_for_kind(
            &mut npcs,
            NpcKind::TownCat,
            &mut roll,
            &mut name,
            None,
        );
        assert_eq!(out, ExchangeOutcome::NoResident);
        assert_eq!((roll.peek(), name.peek()), before, "zero draws");
        assert_eq!(out.chat_key(), "Misc.PetExchangeFail");
    }

    /// The dead resident is not active — the scan skips it (`nPC.active`).
    #[test]
    fn w4_g10_a_dead_resident_is_not_active() {
        let mut n = pet(NpcKind::TownCat, 0);
        n.hp = 0;
        let mut npcs = [n];
        let mut roll = UnifiedRandom::new(7);
        let mut name = UnifiedRandom::new(9);
        let out = reroll_variation_for_kind(
            &mut npcs,
            NpcKind::TownCat,
            &mut roll,
            &mut name,
            None,
        );
        assert_eq!(out, ExchangeOutcome::NoResident);
    }

    /// `NPC.cs:80941-80950` — the cat at variation 0 rerolls to a NEW index
    /// on `Main.rand`, heals to full, names from the NEW variation's
    /// category on `genRand` (a different stream). The roll mirror: keep
    /// drawing `Next(6)` while the index is unchanged.
    #[test]
    fn w4_g10_the_cat_exchange_rolls_heals_and_names_on_two_streams() {
        init_lang();
        // A seed whose first Next(6) is non-zero (one draw, then success).
        let mut seed = 0;
        loop {
            let mut r = UnifiedRandom::new(seed);
            if r.next_max(6) != 0 {
                break;
            }
            seed += 1;
        }
        let mut npcs = [pet(NpcKind::TownCat, 0)];
        let mut roll = UnifiedRandom::new(seed);
        let mut name = UnifiedRandom::new(123);
        let out = reroll_variation_for_kind(
            &mut npcs,
            NpcKind::TownCat,
            &mut roll,
            &mut name,
            None,
        );
        assert_eq!(out, ExchangeOutcome::Success);
        assert_eq!(out.chat_key(), "Misc.PetExchangeSuccess");
        // The mirrored roll stream: one Next(6).
        let mut witness = UnifiedRandom::new(seed);
        let new_index = witness.next_max(6);
        assert_ne!(new_index, 0);
        assert_eq!(npcs[0].variation_index, new_index);
        assert_eq!(roll.peek(), witness.peek(), "one Main.rand draw");
        // The heal (`:80950`).
        assert_eq!(npcs[0].hp, npcs[0].max_hp);
        assert_eq!(npcs[0].hp, 250);
        // The name: genRand.Next(category_size) over CatNames_<variant>.
        let variants = ["Siamese", "Black", "OrangeTabby", "RussianBlue", "Silver", "White"];
        let cat = format!("CatNames_{}", variants[new_index as usize]);
        let size = crate::lang::category_size(&cat);
        assert!(size > 0, "the shipped CatNames categories exist");
        let mut name_witness = UnifiedRandom::new(123);
        let draw = name_witness.next_max(size as i32) as usize;
        assert_eq!(npcs[0].given_name, crate::lang::random_from_category(&cat, draw));
        assert_eq!(name.peek(), name_witness.peek(), "one genRand draw");
    }

    /// `NPC.cs:80941-80948` — 100 unchanged draws = Fail: no heal, no name,
    /// no index write. A `Next(6)` stream pinned to 0 by a mirrored oracle
    /// is impossible with UnifiedRandom, so the FAIL path is witnessed on
    /// the LEGACY profile instead (the constant-0 roll) plus a cat whose
    /// 100 draws all repeat — the loop-cap witness uses a stub roller via
    /// the legacy arm; the cat's 100-draw cap is pinned by the loop shape
    /// (the `for _ in 0..100` bound) and the legacy fail below.
    #[test]
    fn w4_g10_the_legacy_slime_at_zero_fails_with_no_draws() {
        // `Profiles.cs:83-86` — LegacyNPCProfile.RollVariation returns 0
        // with NO draw; a slime at variation 0 can never change → Fail.
        let mut npcs = [pet(NpcKind::TownSlimeBlue, 0)];
        let mut roll = UnifiedRandom::new(7);
        let mut name = UnifiedRandom::new(9);
        let before = (roll.peek(), name.peek());
        let out = reroll_variation_for_kind(
            &mut npcs,
            NpcKind::TownSlimeBlue,
            &mut roll,
            &mut name,
            None,
        );
        assert_eq!(out, ExchangeOutcome::Fail);
        assert_eq!((roll.peek(), name.peek()), before, "the legacy roll draws NOTHING");
        assert_eq!(npcs[0].variation_index, 0);
        assert_eq!(npcs[0].hp, 10, "no heal on failure");
        assert!(npcs[0].given_name.is_empty(), "no name on failure");
    }

    /// `NPC.cs:80949-80950` — a legacy slime at a NON-zero index succeeds
    /// (the constant-0 roll changes it), names from its FIXED colour
    /// category, heals — still zero Main.rand draws before the burst.
    #[test]
    fn w4_g10_the_legacy_slime_from_nonzero_succeeds_without_a_roll_draw() {
        init_lang();
        let mut npcs = [pet(NpcKind::TownSlimeCopper, 3)];
        let mut roll = UnifiedRandom::new(7);
        let mut name = UnifiedRandom::new(55);
        let out = reroll_variation_for_kind(
            &mut npcs,
            NpcKind::TownSlimeCopper,
            &mut roll,
            &mut name,
            None,
        );
        assert_eq!(out, ExchangeOutcome::Success);
        assert_eq!(npcs[0].variation_index, 0);
        assert_eq!(npcs[0].hp, 250);
        let size = crate::lang::category_size("SlimeNames_Copper");
        assert!(size > 0);
        let mut name_witness = UnifiedRandom::new(55);
        let draw = name_witness.next_max(size as i32) as usize;
        assert_eq!(
            npcs[0].given_name,
            crate::lang::random_from_category("SlimeNames_Copper", draw)
        );
        assert_eq!(roll.peek(), UnifiedRandom::new(7).peek(), "zero roll draws (no burst)");
    }

    /// `NPC.cs:80922-80928` — the FIRST active resident rerolls, the second
    /// stays untouched (the C# returns after one).
    #[test]
    fn w4_g10_only_the_first_resident_rerolls() {
        init_lang();
        let mut seed = 0;
        loop {
            let mut r = UnifiedRandom::new(seed);
            if r.next_max(6) != 0 {
                break;
            }
            seed += 1;
        }
        let mut npcs = [pet(NpcKind::TownDog, 0), pet(NpcKind::TownDog, 0)];
        let mut roll = UnifiedRandom::new(seed);
        let mut name = UnifiedRandom::new(1);
        let out = reroll_variation_for_kind(
            &mut npcs,
            NpcKind::TownDog,
            &mut roll,
            &mut name,
            None,
        );
        assert_eq!(out, ExchangeOutcome::Success);
        assert_ne!(npcs[0].variation_index, 0);
        assert_eq!(npcs[1].variation_index, 0, "the second resident is untouched");
        assert_eq!(npcs[1].hp, 10, "the second resident is not healed");
    }

    /// `ParticleOrchestrator.cs:2691-2700` — the burst's 104-draw census
    /// (8 per gore × 13) against a mirrored oracle, plus the alpha-100 /
    /// velocity-overwrite writes. The roll draws ONE Next(6) first (the
    /// seed forces a non-zero first draw), then the burst's 104.
    #[test]
    fn w4_g10_the_pet_exchange_burst_draws_104_and_writes_the_gores() {
        init_lang();
        let mut seed = 0;
        loop {
            let mut r = UnifiedRandom::new(seed);
            if r.next_max(6) != 0 {
                break;
            }
            seed += 1;
        }
        let w = world();
        let mut npcs = [pet(NpcKind::TownCat, 0)];
        let (cx, cy) = npcs[0].center();
        let (mvx, mvy) = (npcs[0].vx, npcs[0].vy);
        let mut roll = UnifiedRandom::new(seed);
        let mut name = UnifiedRandom::new(3);
        let mut gore = GorePool::new();
        let out = reroll_variation_for_kind(
            &mut npcs,
            NpcKind::TownCat,
            &mut roll,
            &mut name,
            Some((&mut gore, &w)),
        );
        assert_eq!(out, ExchangeOutcome::Success);
        // The mirrored stream: the roll, then 13 × (angle, radius, type,
        // scale, NewGore's two jitters, rotation Next(), speed NextFloat).
        let mut witness = UnifiedRandom::new(seed);
        let _ = witness.next_max(6); // the variation roll
        let mut expected: Vec<(f32, f32, i32, f32, f32, f32)> = Vec::new();
        for _ in 0..13 {
            let angle = witness.next_float() * std::f32::consts::PI * 2.0;
            let radius = witness.next_float();
            let off = Vec2::new(angle.cos(), angle.sin()) * Vec2::new(20.0, 20.0) * radius;
            let pos = Vec2::new(cx, cy) + Vec2::new(-20.0, -20.0) + off;
            let ty = witness.next_range(61, 64);
            let scale = 1.0 + witness.next_float() * 0.3;
            // NewGore's default-block velocity jitters (`Gore.cs:1327-1328`)
            // — drawn, then OVERWRITTEN by the burst's velocity write.
            let _jy = witness.next_range(10, 31);
            let _jx = witness.next_range(-20, 21);
            let rot = std::f32::consts::PI * 2.0 * witness.next() as f32;
            let speed = witness.next_float();
            let vel = Vec2::new(rot.cos(), rot.sin()) * speed + Vec2::new(mvx, mvy) * 0.5;
            expected.push((pos.x, pos.y, ty, scale, vel.x, vel.y));
        }
        assert_eq!(roll.peek(), witness.peek(), "the 1 + 104 draw census");
        for (i, (px, py, ty, scale, vx, vy)) in expected.iter().enumerate() {
            let g = gore.slot(i);
            assert!(g.active, "gore {i} active");
            assert_eq!(g.gore_type, *ty, "gore {i} type 61..63");
            assert_eq!(g.alpha, 100, "gore {i} alpha 100");
            assert!((g.position.x - px).abs() < 1e-3, "gore {i} position.x");
            assert!((g.position.y - py).abs() < 1e-3, "gore {i} position.y");
            assert!((g.scale - scale).abs() < 1e-5, "gore {i} scale");
            assert!((g.velocity.x - vx).abs() < 1e-3, "gore {i} velocity.x");
            assert!((g.velocity.y - vy).abs() < 1e-3, "gore {i} velocity.y");
            assert!(!g.sticky, "the smoke row is sticky-false");
        }
        assert!(!gore.slot(13).active, "exactly 13 gores");
    }
}