//! The `GoreID` sets — port of `Terraria.ID.GoreID.Sets`
//! (C# `decompiled/Terraria/ID/GoreID.cs:5-25`).
//!
//! The C# builds the four tables with `SetFactory` (`SetFactory.cs:97-130`):
//! `CreateIntSet(default, id, value, …)` fills the whole array with `default`
//! and then writes the listed `(id, value)` pairs; `CreateBoolSet(false, id, …)`
//! flips the listed ids to `true`. The port keeps the same default-then-pairs
//! shape — the rows below are the C#'s literal pairs, ranges collapsed.
//!
//! - **Count:** 1407 (`GoreID.Count`, `GoreID.cs:1615`).
//! - **SpecialAI** (`GoreID.cs:20`): default `0`; `1` for 860/892/893/1014-1016,
//!   `2` for 825-827, `3` for 910-925/1113-1121/1248-1255/1257/1278/1007/1008,
//!   `4` for 1087-1090, `5` for 1091-1093, `6` for 1202-1204, `7` for 1225.
//! - **DisappearSpeed** (`GoreID.cs:21`): default `1`; `3` for 1007, `10` for
//!   1008, `7` for 1024-1026.
//! - **DisappearSpeedAlpha** (`GoreID.cs:22`): default `1`; the same rows plus
//!   `3` for 1218.
//! - **IsDrip** (`GoreID.cs:23`): `true` for 706-717 (the water/lava/honey
//!   drips), 943, 1147, 1160-1162, 1383.

/// Total number of gore types (`GoreID.Count`, `GoreID.cs:1615`).
pub const GORE_COUNT: usize = 1407;

/// `GoreID.Sets.SpecialAI[Type]` (`GoreID.cs:20`) — the per-type special
/// behaviour of `Gore.NewGore` (`Gore.cs:1390-1456`) and `Gore.Update`.
pub fn special_ai(gore_type: usize) -> i32 {
    match gore_type {
        860 | 892 | 893 | 1014..=1016 => 1,
        825..=827 => 2,
        910..=925 | 1113..=1121 | 1248..=1255 | 1257 | 1278 | 1007 | 1008 => 3,
        1087..=1090 => 4,
        1091..=1093 => 5,
        1202..=1204 => 6,
        1225 => 7,
        _ => 0,
    }
}

/// `GoreID.Sets.DisappearSpeed[Type]` (`GoreID.cs:21`) — the per-tick decay of
/// `timeLeft` (`Gore.cs:112`).
pub fn disappear_speed(gore_type: usize) -> i32 {
    match gore_type {
        1007 => 3,
        1008 => 10,
        1024..=1026 => 7,
        _ => 1,
    }
}

/// `GoreID.Sets.DisappearSpeedAlpha[Type]` (`GoreID.cs:22`) — the per-tick
/// alpha decay (`Gore.cs:158`).
pub fn disappear_speed_alpha(gore_type: usize) -> i32 {
    match gore_type {
        1007 => 3,
        1008 => 10,
        1024..=1026 => 7,
        1218 => 3,
        _ => 1,
    }
}

/// `GoreID.Sets.IsDrip[Type]` (`GoreID.cs:23`) — the drips draw behind the
/// tiles on a 15-frame sheet and live three times as long (`Gore.cs:1355-1360`).
pub fn is_drip(gore_type: usize) -> bool {
    matches!(
        gore_type,
        706..=717 | 943 | 1147 | 1160..=1162 | 1383
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_special_ai_table_matches_the_csharp_rows() {
        // `GoreID.cs:20` — the default is 0 and each listed id carries its row.
        assert_eq!(special_ai(0), 0);
        assert_eq!(special_ai(860), 1);
        assert_eq!(special_ai(892), 1);
        assert_eq!(special_ai(893), 1);
        assert_eq!(special_ai(1014), 1);
        assert_eq!(special_ai(1016), 1);
        assert_eq!(special_ai(825), 2);
        assert_eq!(special_ai(827), 2);
        assert_eq!(special_ai(910), 3);
        assert_eq!(special_ai(925), 3);
        assert_eq!(special_ai(1007), 3);
        assert_eq!(special_ai(1008), 3);
        assert_eq!(special_ai(1113), 3);
        assert_eq!(special_ai(1121), 3);
        assert_eq!(special_ai(1248), 3);
        assert_eq!(special_ai(1255), 3);
        assert_eq!(special_ai(1257), 3);
        assert_eq!(special_ai(1278), 3);
        assert_eq!(special_ai(1087), 4);
        assert_eq!(special_ai(1090), 4);
        assert_eq!(special_ai(1091), 5);
        assert_eq!(special_ai(1093), 5);
        assert_eq!(special_ai(1202), 6);
        assert_eq!(special_ai(1204), 6);
        assert_eq!(special_ai(1225), 7);
        // The neighbours of every run keep the default.
        assert_eq!(special_ai(859), 0);
        assert_eq!(special_ai(861), 0);
        assert_eq!(special_ai(824), 0);
        assert_eq!(special_ai(828), 0);
        assert_eq!(special_ai(909), 0);
        assert_eq!(special_ai(926), 0);
        assert_eq!(special_ai(1006), 0);
        assert_eq!(special_ai(1009), 0);
        assert_eq!(special_ai(1086), 0);
        assert_eq!(special_ai(1094), 0);
        assert_eq!(special_ai(1201), 0);
        assert_eq!(special_ai(1205), 0);
        assert_eq!(special_ai(1224), 0);
        assert_eq!(special_ai(1226), 0);
    }

    #[test]
    fn the_disappear_speed_tables_match_the_csharp_rows() {
        // `GoreID.cs:21-22` — the default is 1 for both tables.
        assert_eq!(disappear_speed(0), 1);
        assert_eq!(disappear_speed(1007), 3);
        assert_eq!(disappear_speed(1008), 10);
        assert_eq!(disappear_speed(1024), 7);
        assert_eq!(disappear_speed(1026), 7);
        assert_eq!(disappear_speed(1027), 1);
        assert_eq!(disappear_speed_alpha(0), 1);
        assert_eq!(disappear_speed_alpha(1007), 3);
        assert_eq!(disappear_speed_alpha(1008), 10);
        assert_eq!(disappear_speed_alpha(1024), 7);
        assert_eq!(disappear_speed_alpha(1218), 3);
        assert_eq!(disappear_speed_alpha(1219), 1);
    }

    #[test]
    fn the_is_drip_table_matches_the_csharp_rows() {
        // `GoreID.cs:23` — only the listed ids are drips.
        assert!(is_drip(706));
        assert!(is_drip(713));
        assert!(is_drip(717));
        assert!(is_drip(943));
        assert!(is_drip(1147));
        assert!(is_drip(1160));
        assert!(is_drip(1162));
        assert!(is_drip(1383));
        assert!(!is_drip(705));
        assert!(!is_drip(718));
        assert!(!is_drip(942));
        assert!(!is_drip(1146));
        assert!(!is_drip(1148));
        assert!(!is_drip(1159));
        assert!(!is_drip(1163));
        assert!(!is_drip(1382));
        assert!(!is_drip(1384));
    }
}
