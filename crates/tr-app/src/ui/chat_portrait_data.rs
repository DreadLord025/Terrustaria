//! The chat-portrait provider half (W4-g58 retire) — the
//! `NPCID.Sets.NPCPortraits` rows (`NPCID.cs:5894-6073`) for the ported
//! roster + the selector resolution (`NPCPortraitSelector.
//! GetFirstValidatedEntry`, `NPCID.cs:123-128`) + the shimmer condition
//! (`NPC.IsShimmerVariant`, `NPC.cs:6713-6723`). The DRAW rides
//! [`super::chat_portrait`]; the hop STATE is `ui::npc_chat`'s
//! (`portrait_hop_lift`, research/240 §4).

use crate::npc::{Npc, NpcKind};

/// The `NPCPortraits` rows for the ported roster: `(NPCID, base sheet
/// stem, has a _shimmer variant)`. The bound/sleeping forms, the pets,
/// the town slimes and the unported town humans (OldMan 37, Santa 142,
/// SkeletonMerchant 453, Steampunker 178, Truffle 160, Pirate 229,
/// Tavernkeep 550, Cyborg 209) have no ported kind — their rows ride the
/// roster lanes. The Zoologist's four-way lycanthrope selector
/// (`:6010-6012`) resolves in [`portrait_sheet_name`].
pub(crate) const PORTRAIT_ROWS: &[(u16, &str, bool)] = &[
    (17, "Portrait_Merchant", true),
    (18, "Portrait_Nurse", true),
    (19, "Portrait_ArmsDealer", true),
    (20, "Portrait_Dryad", true),
    (22, "Portrait_Guide", true),
    (38, "Portrait_Demolitionist", true),
    (54, "Portrait_Clothier", true),
    (107, "Portrait_GoblinTinkerer", true),
    (108, "Portrait_Wizard", true),
    (124, "Portrait_Mechanic", true),
    (207, "Portrait_DyeTrader", true),
    (208, "Portrait_PartyGirl", true),
    (227, "Portrait_Painter", true),
    (228, "Portrait_WitchDoctor", true),
    (353, "Portrait_Stylist", true),
    (368, "Portrait_TravellingMerchant", true),
    (369, "Portrait_Angler", true),
    (441, "Portrait_TaxCollector", true),
    (588, "Portrait_Golfer", true),
    (633, "Portrait_Zoologista", true),
    (663, "Portrait_Princess", true),
];

/// The provider's resolved sheet name — the FIRST entry whose condition
/// holds, else the default (`NPCID.cs:123-128`). `shimmered` is the
/// `ShimmeredPortraitCondition` (`NPCID.cs:383-391`); `lycanthrope` is
/// the Zoologist's `ShouldBestiaryGirlBeLycantrope` arm (`:6010-6012` —
/// shimmer first, then the plain `Portrait_Zoologistb`, then defaults).
/// `None` = no row (the C#'s `npcChatPortrait == null` early return,
/// `Main.cs:40326-40329`).
pub(crate) fn portrait_sheet_name(
    kind: NpcKind,
    shimmered: bool,
    lycanthrope: bool,
) -> Option<String> {
    let id = kind.cs_id();
    let (_, stem, has_shimmer) = PORTRAIT_ROWS.iter().find(|(row, _, _)| *row == id)?;
    if id == 633 {
        // The four-way selector's entry order (`NPCID.cs:6010-6012`).
        return Some(match (shimmered, lycanthrope) {
            (true, false) => "Portrait_Zoologista_shimmer".to_string(),
            (true, true) => "Portrait_Zoologistb_shimmer".to_string(),
            (false, true) => "Portrait_Zoologistb".to_string(),
            (false, false) => "Portrait_Zoologista".to_string(),
        });
    }
    Some(if shimmered && *has_shimmer {
        format!("{stem}_shimmer")
    } else {
        stem.to_string()
    })
}

/// `IsShimmerVariant` for a live NPC (`NPC.cs:6713-6723`): variation 1
/// AND the `ShimmerTownTransform` set (`npc_ai_town`'s port). The
/// persisted `App.shimmered_npcs` bitmap is the save-side mirror.
pub(crate) fn is_shimmer_variant(npc: &Npc) -> bool {
    npc.variation_index == 1
        && crate::npc_ai_town::shimmer_town_transform(npc.kind.cs_id() as i32)
}

/// Every sheet name the loader must pack for [`PORTRAIT_ROWS`] — the
/// base stems + their `_shimmer` twins + the Zoologist `b` pair (the
/// assets loader and the Lookups builder share this expansion).
pub(crate) fn packed_sheet_names() -> Vec<String> {
    PORTRAIT_ROWS
        .iter()
        .flat_map(|(_, stem, has_shimmer)| {
            let mut names = vec![stem.to_string()];
            if *has_shimmer {
                names.push(format!("{stem}_shimmer"));
            }
            if *stem == "Portrait_Zoologista" {
                names.push("Portrait_Zoologistb".to_string());
                names.push("Portrait_Zoologistb_shimmer".to_string());
            }
            names
        })
        .collect()
}
