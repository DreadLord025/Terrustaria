//! The chat-portrait witnesses (W4-g58 retire, WRITTEN NOT RUN): the
//! provider resolution + the shimmer gate + the hop-lift anchor (the
//! draw needs the atlas — the retail-content tests' lane).

use super::chat_portrait::portrait_anchor;
use super::chat_portrait_data::{is_shimmer_variant, packed_sheet_names, portrait_sheet_name};
use crate::npc::{Npc, NpcKind};
use glam::Vec2;

/// The provider table resolves every ported talkable kind, the shimmer
/// condition swaps the stem, and the Zoologist's four-way selector
/// follows the C#'s entry order (`NPCID.cs:6010-6012`).
#[test]
fn the_provider_resolves_the_roster_and_the_conditions() {
    assert_eq!(
        portrait_sheet_name(NpcKind::Merchant, false, false).as_deref(),
        Some("Portrait_Merchant")
    );
    assert_eq!(
        portrait_sheet_name(NpcKind::Merchant, true, false).as_deref(),
        Some("Portrait_Merchant_shimmer")
    );
    // No row: the bound forms + the wild kinds return None.
    assert_eq!(portrait_sheet_name(NpcKind::BoundWizard, false, false), None);
    assert_eq!(portrait_sheet_name(NpcKind::Slime, false, false), None);
    // The Zoologist selector: shimmer-first, then plain lycanthrope.
    for (sh, ly, want) in [
        (false, false, "Portrait_Zoologista"),
        (false, true, "Portrait_Zoologistb"),
        (true, false, "Portrait_Zoologista_shimmer"),
        (true, true, "Portrait_Zoologistb_shimmer"),
    ] {
        assert_eq!(
            portrait_sheet_name(NpcKind::Zoologist, sh, ly).as_deref(),
            Some(want)
        );
    }
    // Every ported talkable kind has a row (the always-true TryGetValue
    // gate the hop port relies on, research/240 §4).
    for kind in [
        NpcKind::Guide, NpcKind::Merchant, NpcKind::Nurse, NpcKind::ArmsDealer,
        NpcKind::Dryad, NpcKind::Demolitionist, NpcKind::Clothier,
        NpcKind::GoblinTinkerer, NpcKind::Painter, NpcKind::WitchDoctor,
        NpcKind::TravellingMerchant, NpcKind::Angler, NpcKind::Stylist,
        NpcKind::TaxCollector, NpcKind::Golfer, NpcKind::Zoologist,
        NpcKind::Princess, NpcKind::DyeTrader, NpcKind::PartyGirl,
        NpcKind::Wizard, NpcKind::Mechanic,
    ] {
        assert!(
            portrait_sheet_name(kind, false, false).is_some(),
            "{kind:?} must carry an NPCPortraits row"
        );
    }
    // The packing expansion covers every resolvable name exactly once
    // (21 stems + 21 shimmer twins + the Zoologist b pair = 44).
    let packed = packed_sheet_names();
    assert_eq!(packed.len(), 44);
    let mut sorted = packed.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), packed.len(), "no duplicate sheet names");
    for kind in [NpcKind::Merchant, NpcKind::Zoologist, NpcKind::Guide] {
        for (sh, ly) in [(false, false), (true, false), (false, true), (true, true)] {
            let name = portrait_sheet_name(kind, sh, ly).unwrap();
            assert!(packed.contains(&name), "{name} must be packed");
        }
    }
}

/// `IsShimmerVariant` (`NPC.cs:6713-6723`): variation 1 AND the
/// transform set — variation 0/2 never shimmer, a non-transform kind
/// fails the set arm.
#[test]
fn the_shimmer_gate_needs_variation_one_and_the_transform_set() {
    let mut n = Npc::new(NpcKind::Merchant, 0.0, 0.0, 1, 0.0);
    n.variation_index = 1;
    assert!(is_shimmer_variant(&n), "17 is in ShimmerTownTransform");
    n.variation_index = 0;
    assert!(!is_shimmer_variant(&n));
    n.variation_index = 2;
    assert!(!is_shimmer_variant(&n));
    let mut s = Npc::new(NpcKind::Slime, 0.0, 0.0, 1, 0.0);
    s.variation_index = 1;
    assert!(!is_shimmer_variant(&s), "the Slime has no transform row");
}

/// The anchor is the panel top-left + (-62, 62) with the hop lift on y
/// (`Main.cs:40337` + `:40358`); frame 0 lifts the full -14, frame 80+
/// and the num4 gate (120+) rest at 0.
#[test]
fn the_anchor_rides_the_hop_lift() {
    let tl = Vec2::new(710.0, 100.0);
    assert_eq!(portrait_anchor(tl, 80), Vec2::new(648.0, 162.0));
    // Frame 0: `BounceEaseOut(0)` = 1 (`Utils.cs:360-363`), so the lift is
    // `-14 * (1 - 1)` = 0 - the resting anchor.
    assert_eq!(portrait_anchor(tl, 0), Vec2::new(648.0, 162.0));
    let f10 = portrait_anchor(tl, 10);
    assert!((f10.y - (162.0 - 10.71875)).abs() < 1e-4);
    assert_eq!(portrait_anchor(tl, 120), Vec2::new(648.0, 162.0));
    assert_eq!(portrait_anchor(tl, 1000), Vec2::new(648.0, 162.0));
}
