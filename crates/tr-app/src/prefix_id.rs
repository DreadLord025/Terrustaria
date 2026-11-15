//! `PrefixID` (`PrefixID.cs`) — the vanilla prefix id constants, the S1-F-4
//! completion of the port's id space: the roll tables and the save/netcode
//! slots carry the ids verbatim, and every id 1..=97 names a C# constant
//! (`PrefixID.Count` = 98 — the array bounds the C# allocates `Lang.prefix`
//! with, `Lang.cs:1218`). The names are the C# constant spellings, which is
//! also the localization key (`Lang.FillNameCacheArray("Prefix", ...)`,
//! `Lang.cs:474` — `Prefix.{const name}`).

/// `PrefixID`'s constants, id ascending — `(id, const name)`, 1..=97 with no
/// gaps (the coverage test pins the walk).
pub const PREFIX_ID: &[(u16, &str)] = &[
    (1, "Large"),
    (2, "Massive"),
    (3, "Dangerous"),
    (4, "Savage"),
    (5, "Sharp"),
    (6, "Pointy"),
    (7, "Tiny"),
    (8, "Terrible"),
    (9, "Small"),
    (10, "Dull"),
    (11, "Unhappy"),
    (12, "Bulky"),
    (13, "Shameful"),
    (14, "Heavy"),
    (15, "Light"),
    (16, "Sighted"),
    (17, "Rapid"),
    (18, "Hasty"),
    (19, "Intimidating"),
    (20, "Deadly"),
    (21, "Staunch"),
    (22, "Awful"),
    (23, "Lethargic"),
    (24, "Awkward"),
    (25, "Powerful"),
    (26, "Mystic"),
    (27, "Adept"),
    (28, "Masterful"),
    (29, "Inept"),
    (30, "Ignorant"),
    (31, "Deranged"),
    (32, "Intense"),
    (33, "Taboo"),
    (34, "Celestial"),
    (35, "Furious"),
    (36, "Keen"),
    (37, "Superior"),
    (38, "Forceful"),
    (39, "Broken"),
    (40, "Damaged"),
    (41, "Shoddy"),
    (42, "Quick"),
    (43, "Deadly2"),
    (44, "Agile"),
    (45, "Nimble"),
    (46, "Murderous"),
    (47, "Slow"),
    (48, "Sluggish"),
    (49, "Lazy"),
    (50, "Annoying"),
    (51, "Nasty"),
    (52, "Manic"),
    (53, "Hurtful"),
    (54, "Strong"),
    (55, "Unpleasant"),
    (56, "Weak"),
    (57, "Ruthless"),
    (58, "Frenzying"),
    (59, "Godly"),
    (60, "Demonic"),
    (61, "Zealous"),
    (62, "Hard"),
    (63, "Guarding"),
    (64, "Armored"),
    (65, "Warding"),
    (66, "Arcane"),
    (67, "Precise"),
    (68, "Lucky"),
    (69, "Jagged"),
    (70, "Spiked"),
    (71, "Angry"),
    (72, "Menacing"),
    (73, "Brisk"),
    (74, "Fleeting"),
    (75, "Hasty2"),
    (76, "Quick2"),
    (77, "Wild"),
    (78, "Rash"),
    (79, "Intrepid"),
    (80, "Violent"),
    (81, "Legendary"),
    (82, "Unreal"),
    (83, "Mythical"),
    (84, "Legendary2"),
    (85, "Fabled"),
    (86, "Loyal"),
    (87, "Worthy"),
    (88, "Focused"),
    (89, "Patient"),
    (90, "Rabid"),
    (91, "IllTempered"),
    (92, "Petty"),
    (93, "Feeble"),
    (94, "Skittish"),
    (95, "Eager"),
    (96, "Ballistic"),
    (97, "Scraggling"),
];

/// `PrefixID.Count` (`PrefixID.cs:214`): 98 — 0 is the no-prefix slot.
pub const PREFIX_COUNT: usize = 98;

/// The constant spelling for `id`, or `"None"` for the 0 slot (`Lang.prefix`'s
/// array bounds leave 0 unnamed; `Item.Prefix`'s 0 return means "no prefix").
pub fn prefix_const_name(id: u16) -> &'static str {
    if id == 0 {
        return "None";
    }
    PREFIX_ID
        .iter()
        .find(|&&(i, _)| i == id)
        .map_or("None", |&(_, n)| n)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// S1-F-4 — the id walk: every id 1..=97 carries its C# constant, no
    /// gaps (`Count` = 98 with the unnamed 0 slot), and the C#'s sampled
    /// spellings ride the walk (the previously-missing 62..80 band and the
    /// 88..97 tail included).
    #[test]
    fn the_prefix_id_walk_is_gapless_and_names_the_csharp_constants() {
        assert_eq!(PREFIX_ID.len(), 97, "the 1..=97 constants");
        for (i, (id, name)) in PREFIX_ID.iter().enumerate() {
            assert_eq!(*id, i as u16 + 1, "id {} walks in order", i + 1);
        }
        assert_eq!(PREFIX_COUNT, 98, "`PrefixID.Count = 98` (`PrefixID.cs:214`)");
        assert_eq!(prefix_const_name(0), "None");
        assert_eq!(prefix_const_name(62), "Hard");
        assert_eq!(prefix_const_name(65), "Warding");
        assert_eq!(prefix_const_name(80), "Violent");
        assert_eq!(prefix_const_name(88), "Focused");
        assert_eq!(prefix_const_name(97), "Scraggling");
        // The every-id probe: the walk never answers "None" for 1..=97.
        for id in 1..=97u16 {
            assert_ne!(prefix_const_name(id), "None");
        }
    }
}
