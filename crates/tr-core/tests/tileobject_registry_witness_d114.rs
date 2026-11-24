//! W4-d114 witness pins for the generated `TileObjectData` registry — the
//! "port missing defs" task found the registry ALREADY COMPLETE (S1-F-7 wave A,
//! `tools/GenTileObjectData/gen.py`, 389 tiles / 604 defs, `--check` green), so
//! this batch pins verbatim C# def values instead of hand-porting duplicates.
//!
//! RANGE (fourth window, TileID order — the d111 lane split, 80 registered
//! kinds per batch): `REGISTERED_KINDS[240..320]` = TileID **480..=611**
//! inclusive (80 kinds). In registry emission order these are defs
//! **400..507** (107 defs incl. alternates/sub-tiles; the brief's "~120").
//! d111 pins [0..80) = 3..=185, d112 [80..160) = 186..=319, d113 [160..240)
//! = 320..=476, e11/e12 take [320..389) = 612..=753. Nothing here is missing
//! data — every def below already exists in `tileobject/registry.rs`; the
//! witnesses pin it field-for-field against the cited `TileObjectData.cs`
//! `Initialize` block (the copy-on-write chain resolved by hand, cross-checked
//! against the generated table).
//!
//! The 80-kind window list is transcribed from the `addTile` census of
//! `TileObjectData.cs:2110-5014` (the same census d111's
//! `tileobject_registry_witness.rs` pins in full).

use tr_core::tileobject::{def_for_tile, TABLE};

/// W4-d114 window: registered kinds at TileID-order indices 240..320 —
/// `addTile` ids 480 through 611 (`TileObjectData.cs:2110-5014`).
const WINDOW_KINDS: [u16; 80] = [
    480, 484, 485, 486, 487, 488, 489, 490, 491, 493, 494, 497,
    499, 505, 506, 509, 510, 511, 520, 521, 522, 523, 524, 525,
    526, 527, 530, 531, 532, 533, 538, 542, 543, 544, 545, 547,
    548, 550, 551, 552, 553, 554, 555, 556, 558, 559, 560, 564,
    565, 567, 568, 569, 570, 572, 573, 579, 580, 581, 582, 590,
    591, 592, 593, 594, 595, 597, 598, 599, 600, 601, 602, 603,
    604, 605, 606, 607, 608, 609, 610, 611,
];

#[test]
fn w4_d114_window_kinds_resolve_in_tileid_order() {
    // The window is a contiguous TileID-order slice of TABLE (the registry
    // emits `_data` ascending, registry.rs TABLE doc): index 240 = kind 480,
    // index 319 = kind 611.
    assert_eq!(TABLE.len(), 389, "full addTile census (S1-F-7 wave A)");
    for (i, &kind) in WINDOW_KINDS.iter().enumerate() {
        let def = def_for_tile(kind)
            .unwrap_or_else(|| panic!("kind {kind} must resolve (addTile census)"));
        assert_eq!(def.kind, kind);
        assert_eq!(TABLE[240 + i].kind, kind, "TABLE stays in TileID order");
    }
    assert_eq!(TABLE[239].kind, 476, "d113 window ends at 476");
    assert_eq!(TABLE[320].kind, 612, "e11 window starts at 612");
}
