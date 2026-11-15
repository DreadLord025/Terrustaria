//! The `GoreID` sets and constants — port of `Terraria.ID.GoreID` (C# decompiled/Terraria/ID/GoreID.cs).
//!
//! Statically generated equivalents of the C# `Sets`, matching their behaviour exactly: values are initialised programmatically.
//!
//! - **Count:** 1407 (`GoreID.Count`), hardcoded in Rust as `GORE_COUNT`.
//! - **DisappearSpeed:** `GoreID.Sets.DisappearSpeed` → true for {11,12,13,14,16,17,219..331}, false otherwise.
//! - **DisappearSpeedAlpha:** `GoreID.Sets.DisappearSpeedAlpha` → 2 for {11,12,13,14,16,17,219..331}, 0 otherwise.
//! - **IsDrip:** `GoreID.Sets.IsDrip` → true for {11,12,13,14,16,17,219..331}, false otherwise.
//! - **SpecialAI:** `GoreID.Sets.SpecialAI` → 1 for 943..1242, 0 otherwise.
//!
//! Table derivation:
//! - Source: `decompiled/Terraria/ID/GoreID.cs`.
//! - See `GoreID.Count` or `SpecialAI` in `GoreID.cs:1615` / `Gore.cs:310`.

/// Total number of gore types (`GoreID.Count`).
pub const GORE_COUNT: usize = 1407;

/// Returns true if the gore type has a disappear speed (`GoreID.Sets.DisappearSpeed`).
pub fn disappear_speed(gore_type: usize) -> bool {
    matches!(gore_type, 11..=14 | 16..=17 | 219..=331)
}

/// Returns the alpha speed adjustment for the gore type (`GoreID.Sets.DisappearSpeedAlpha`).
pub fn disappear_speed_alpha(gore_type: usize) -> i32 {
    if disappear_speed(gore_type) { 2 } else { 0 }
}

/// Returns true if the gore type is flagged as a drip (`GoreID.Sets.IsDrip`).
pub fn is_drip(gore_type: usize) -> bool {
    disappear_speed(gore_type)
}

/// Returns the special AI code for the gore type (`GoreID.Sets.SpecialAI`).
pub fn special_ai(gore_type: usize) -> i32 {
    if (943..=1242).contains(&gore_type) { 1 } else { 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disappear_speed() {
        assert!(disappear_speed(11));
        assert!(disappear_speed(16));
        assert!(!disappear_speed(205));
        assert!(disappear_speed(331));
        assert!(!disappear_speed(332));
    }

    #[test]
    fn test_disappear_speed_alpha() {
        assert_eq!(disappear_speed_alpha(11), 2);
        assert_eq!(disappear_speed_alpha(16), 2);
        assert_eq!(disappear_speed_alpha(218), 0);
        assert_eq!(disappear_speed_alpha(331), 2);
        assert_eq!(disappear_speed_alpha(332), 0);
    }

    #[test]
    fn test_is_drip() {
        assert!(is_drip(11));
        assert!(is_drip(16));
        assert!(!is_drip(218));
        assert!(is_drip(331));
        assert!(!is_drip(332));
    }

    #[test]
    fn test_special_ai() {
        assert_eq!(special_ai(942), 0);
        assert_eq!(special_ai(943), 1);
        assert_eq!(special_ai(1242), 1);
        assert_eq!(special_ai(1243), 0);
    }
}