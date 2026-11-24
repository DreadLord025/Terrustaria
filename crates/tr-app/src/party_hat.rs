//! The birthday-party town-NPC hats (`NPC.UsesPartyHat` / `GetPartyHatColor`,
//! `NPC.cs:66611-66659`) and their draw (`Main.DrawNPCExtras`'s
//! `!beforeDraw && n.UsesPartyHat()` block, `Main.cs:26914-27188`).
//!
//! While `BirthdayParty.PartyIsUp` every town NPC (bar the bound forms, the
//! Tax Collector and the Zoologist) wears a coloured party hat; the colour
//! is a deterministic hash of the NPC's name, slot and the moon phase, so it
//! is stable for a given world-day.
//!
//! W4-c39 exec (research/114 §1g): the death-gore half DROPPED — d55's
//! `npc_hit_effect_a.rs::party_hat_block` is the canonical survivor (the
//! full normal/shimmer table, caller-gated, the strike-stream contract).
//! `UsesPartyHat` here is life-agnostic (the C# property has no life gate;
//! the DRAW caller adds `hp > 0`, the HitEffect caller gates `hp <= 0`).
//!
//! Deviations (recorded): the shimmer-variant per-type offsets
//! (`Main.cs:27029-27060`) and the town-slime / pet special cases
//! (`:26945-27028`, types 637/638/656/663-frame/670-684) name kinds the port
//! does not ship, so only the ported town roster's rows are carried. The
//! `IsABestiaryIconDummy` / `IsAPortraitDummy` gates fold to false (the port
//! draws no bestiary/portrait dummies through this pass).

use crate::npc::Npc;
#[cfg(test)]
use crate::npc::NpcKind;

/// `PartyHatColor` (`PartyHatColor.cs:3-12`): None 0, Blue 1, Pink 2, Cyan 3,
/// Purple 4, White 5.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PartyHatColor {
    None = 0,
    Blue = 1,
    Pink = 2,
    Cyan = 3,
    Purple = 4,
    White = 5,
}

/// `NPC.UsesPartyHat` (`NPC.cs:66611-66634`): the bound old man (37), the
/// skeleton merchant (453) and the Zoologist (633) never wear one; the
/// Dryad's (20) variation 1 (the shimmer form) skips it; `ForcePartyHatOn`
/// forces it (unported — no writer); otherwise a town NPC with a non-empty
/// frame wears one iff `BirthdayParty.PartyIsUp`.
///
/// W4-c39 exec (research/114 §0.3): the C# property has NO life gate — the
/// `life <= 0` term lives in the CALLERS (`HitEffect` `:81660` gates
/// `townNPC && life <= 0 && UsesPartyHat()`; the `UpdateNPC`-drawn roster
/// supplies the living half implicitly). The earlier `hp > 0` baked in
/// here made the predicate always false on a corpse, so the death-gore
/// seam could never fire. The predicate is life-agnostic now; the DRAW
/// caller adds `n.hp > 0` (the living roster), the HitEffect caller gates
/// `hp <= 0` itself (d55's `npc_hit_effect_a.rs` shape).
///
/// The port's `is_town_npc_like` is the `townNPC` flag set; the bound forms
/// (37/453) have no ported kind, so the explicit exclusions are the Tax
/// Collector (441, in the flag set) and the Zoologist (633). The frame gate
/// (`frame.Height > 0`) is always true for a drawn NPC.
pub(crate) fn uses_party_hat(n: &Npc, party_is_up: bool) -> bool {
    let ty = n.kind.net_id();
    if ty == 441 || ty == 37 || ty == 633 {
        return false;
    }
    if ty == 20 && n.variation_index == 1 {
        return false;
    }
    // `ForcePartyHatOn` (no ported writer) and the bestiary/portrait-dummy
    // gates fold to false; the `frame.Height > 0 && townNPC` half is the
    // town-flag set. NO life term (the C# property has none — see above).
    n.kind.is_town_npc_like() && party_is_up
}

/// `NPC.GetPartyHatColor` (`NPC.cs:66636-66659`): the name-hash colour.
/// `num = name.Length + name[0]`, `num2 = moonPhase` (−1 in the day), then
/// `num += whoAmI + num2 * (whoAmI % 2 == 0 ? 1 : -1)`; the `< 0 → +5`,
/// `% 5`, `== 0 → ++` chain lands in 1..=5. `name` is `GivenOrTypeName`
/// (the given name, else the type name); an empty name answers None.
pub(crate) fn party_hat_color(
    name: &str,
    who_am_i: i32,
    moon_phase: u8,
    day_time: bool,
) -> PartyHatColor {
    if name.is_empty() {
        return PartyHatColor::None;
    }
    // C# `name[0]` is a UTF-16 code unit; the ported names are ASCII, so the
    // first byte's value matches. `Length` is the char count.
    let mut num = name.chars().count() as i32 + name.chars().next().unwrap() as i32;
    let mut num2 = moon_phase as i32;
    if day_time {
        num2 -= 1;
    }
    // `(whoAmI % 2 == 0).ToDirectionInt()` — true → 1, false → −1.
    let dir = if who_am_i % 2 == 0 { 1 } else { -1 };
    num += who_am_i + num2 * dir;
    if num < 0 {
        num += 5;
    }
    num %= 5;
    if num == 0 {
        num += 1;
    }
    match num {
        1 => PartyHatColor::Blue,
        2 => PartyHatColor::Pink,
        3 => PartyHatColor::Cyan,
        4 => PartyHatColor::Purple,
        _ => PartyHatColor::White,
    }
}

/// The hat strip's frame column (`Main.cs:26923-26939`): Blue 0 (the default
/// `num2 = 0`), Pink 16, Cyan 17, Purple 18, White 19; `Frame(20, 1, num2 % 20)`.
pub(crate) fn hat_frame_column(color: PartyHatColor) -> u32 {
    match color {
        PartyHatColor::Pink => 16,
        PartyHatColor::Cyan => 17,
        PartyHatColor::Purple => 18,
        PartyHatColor::White => 19,
        _ => 0,
    }
}

/// `NPCID.Sets.HatOffsetY` (`NPCID.cs:10462`) for the ported town roster —
/// the per-type Y nudge added to the hat anchor (`Main.cs:27066`). Types
/// absent from the set answer the factory default 0.
fn hat_offset_y(ty: i32) -> i32 {
    match ty {
        227 => 4,
        107 | 108 | 208 | 124 | 588 | 663 => 2,
        17 => 2,
        38 => 8,
        54 => 4,
        369 => 6,
        353 => -2,
        _ => 0,
    }
}

/// `NPCID.Sets.NPCFramingGroup` (`NPCID.cs:10486`) → the
/// `TownNPCsFramingGroups` row (`:10488-10537`). The ported kinds land in
/// groups 0/1/2; the per-frame Y offset is `array[frame]` (clamped to 0 when
/// the frame index runs past the row, `Main.cs:26918-26921`).
fn framing_group_offset(ty: i32, frame: u32) -> i32 {
    const GROUP0: [i32; 26] = [
        0, 0, 0, -2, -2, -2, 0, 0, 0, 0, -2, -2, -2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    const GROUP1: [i32; 25] = [
        0, 0, 0, -2, -2, -2, 0, 0, 0, -2, -2, -2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    const GROUP2: [i32; 25] = [
        0, 0, 0, -2, -2, -2, 0, 0, -2, -2, -2, -2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    let group = match ty {
        18 | 20 | 208 | 124 | 353 | 633 => 1,
        369 => 2,
        _ => 0,
    };
    let row: &[i32] = match group {
        1 => &GROUP1,
        2 => &GROUP2,
        _ => &GROUP0,
    };
    let idx = frame as usize;
    if idx >= row.len() {
        0
    } else {
        row[idx]
    }
}

/// The hat anchor's per-type X/Y nudges and the `ai[0] == 5` (attack-pose)
/// lift (`Main.cs:26942-27163`), reduced to the ported town roster. Returns
/// `(num3, num4, num6, num7)`:
/// * `num3` — the base X offset (Princess 663 → 1, else 0);
/// * `num4` — the base Y offset (town slimes only — 0 for the port);
/// * `num6` — the `ai[0] == 5` Y lift (per-type, else 0 when not attacking);
/// * `num7` — the closing X offset (`Main.cs:27110-27162`).
fn hat_type_offsets(ty: i32, attacking: bool) -> (i32, i32, i32, i32) {
    let num3 = if ty == 663 { 1 } else { 0 };
    let num4 = 0; // town-slime rows (682/683) are unported.
    let num6 = if attacking {
        match ty {
            38 => -8,
            124 => -2,
            588 => -4,
            108 => -6,
            663 => -8,
            _ => -4,
        }
    } else {
        0
    };
    let num7 = match ty {
        588 => 0,
        227 => -4,
        228 => -2,
        17 | 18 | 19 | 20 | 22 | 124 | 353 | 633 => -1,
        38 | 54 | 107 | 108 | 207 => -3,
        208 | 369 => 1,
        _ => 0,
    };
    (num3, num4, num6, num7)
}

/// The hat's world-space draw anchor + scale (`Main.cs:27061-27188`), for a
/// drawn town NPC. `frame_h` is the NPC sheet's per-frame height (the
/// `n.frame.Y / n.frame.Height` row index is `n.frame()`); `sheet_w`/`sheet_h`
/// the hat strip's native size. Returns the anchor (the C# `bottom` after the
/// offset fold, world pixels), the source-frame size (the 2-px-trimmed hat
/// cell) and the origin, all in the NPC's `scale` units — the caller applies
/// the screen transform and the `npcColor * Opacity` tint.
///
/// The C# draws with `rotation = 0`, `origin = rectangle.Size() -
/// (rectangle.Width/2, 12)`, `scale = n.scale` and the NPC's sprite flip.
pub(crate) struct HatDraw {
    /// The anchor in world pixels (the C# `bottom`, pre-`screenPosition`).
    pub anchor: [f32; 2],
    /// The trimmed source cell size (native px).
    pub cell: [f32; 2],
    /// The rotation origin (native px).
    pub origin: [f32; 2],
    /// `n.scale`.
    pub scale: f32,
    /// The horizontal flip (`spriteDirection == 1` flips, the NPC draw rule).
    pub flip: bool,
    /// The strip column.
    pub column: u32,
}

/// Build the hat draw for `n` (assumes [`uses_party_hat`] is true). `frame_h`
/// is the NPC's per-frame sheet height; `cell_w`/`cell_h` the Extra_72 strip's
/// FULL per-column cell size (the 2-px trim is applied here, the C#
/// `Width -= 2; Height -= 2`).
pub(crate) fn hat_draw(
    n: &Npc,
    color: PartyHatColor,
    frame_h: f32,
    cell_w: f32,
    cell_h: f32,
) -> HatDraw {
    let ty = n.kind.net_id();
    let (_, h) = n.size();
    let num5 = n.sprite_dir; // `n.spriteDirection`
    // The port's `frame` is the row index already (the C# `n.frame.Y /
    // n.frame.Height`); a zero-height frame answers row 0.
    let frame_row = if frame_h > 0.0 { n.frame() } else { 0 };
    let (num3, num4, num6, num7) = hat_type_offsets(ty, n.ai[0] == 5.0);
    // `vector = -new Vector2(0, n.height) + new Vector2(-2*num5, n.gfxOffY)`.
    let mut vx = -2.0 * num5 as f32;
    let mut vy = -h + n.gfx_off_y;
    vx += num3 as f32 * num5 as f32;
    vy += framing_group_offset(ty, frame_row) as f32;
    vy += num4 as f32;
    vy += hat_offset_y(ty) as f32;
    vy += num6 as f32;
    // The closing X fold (`:27163-27164`): `num7 * num5 + 4 * num5`.
    vx += num7 as f32 * num5 as f32;
    vx += 4.0 * num5 as f32;
    // `bottom = n.Bottom; bottom += vector * n.scale`.
    let bottom_x = n.x + n.size().0 * 0.5;
    let bottom_y = n.y + h;
    let anchor = [bottom_x + vx * n.scale, bottom_y + vy * n.scale];
    // `rectangle = value.Frame(20, 1, num2 % 20); Width -= 2; Height -= 2`.
    let cell_w = cell_w - 2.0;
    let cell_h = cell_h - 2.0;
    // `origin = rectangle.Size() - new Vector2(rectangle.Width / 2, 12)`.
    let origin = [cell_w - cell_w * 0.5, cell_h - 12.0];
    HatDraw {
        anchor,
        cell: [cell_w, cell_h],
        origin,
        scale: n.scale,
        flip: num5 == 1,
        column: hat_frame_column(color),
    }
}

// W4-c39 exec (research/114 §1g): the death-gore table DROPPED — d55's
// `npc_hit_effect_a.rs::party_hat_block` is the canonical survivor (the
// full normal/shimmer pairs incl. 229's spawn-NOTHING shimmer arm, the
// strike-stream RNG contract, caller-gated `town_npc && hp <= 0 &&
// uses_party_hat`). c39's base-rows-only `party_death_gores` + its
// `update_npcs` death-site block were the wrong shape (the baked hp>0
// gate made them never fire, and the stream was UpdateNpcs not the
// strike's). Wiring d55's `hit_effect` into the strike path is its lane's
// declared follow-up — the merge must NOT keep c39's block as a
// "temporary" wire.

#[cfg(test)]
mod tests {
    use super::*;

    /// W4-c39 — `GetPartyHatColor`'s name-hash chain (`NPC.cs:66642-66658`):
    /// the `< 0 → +5`, `% 5`, `== 0 → ++` fold lands every input in 1..=5,
    /// and the day-time moon-phase decrement shifts the colour.
    #[test]
    fn w4_c39_the_hat_colour_hash_lands_in_one_to_five() {
        // A spread of names/slots/phases — every result is a real colour.
        for (name, slot, phase, day) in [
            ("Andrew", 0, 0u8, true),
            ("Merchant", 1, 3, false),
            ("Zoologist", 2, 7, true),
            ("A", 255, 4, false),
            ("LongGivenName", 12, 0, true),
            ("", 3, 2, false),
        ] {
            let c = party_hat_color(name, slot, phase, day);
            if name.is_empty() {
                assert_eq!(c, PartyHatColor::None, "an empty name answers None");
            } else {
                assert!(
                    matches!(
                        c,
                        PartyHatColor::Blue
                            | PartyHatColor::Pink
                            | PartyHatColor::Cyan
                            | PartyHatColor::Purple
                            | PartyHatColor::White
                    ),
                    "{name}/{slot}/{phase}/{day} -> {c:?}"
                );
            }
        }
        // The day-time decrement changes the phase term: same name/slot,
        // phase 0 day vs night differ by the `num2` step.
        let day = party_hat_color("Andrew", 0, 0, true);
        let night = party_hat_color("Andrew", 0, 0, false);
        // num2 = -1 (day) vs 0 (night); slot 0 is even (dir 1), so the day
        // colour is one step lower in the hash (mod the fold).
        assert_ne!(day, night, "the day-time phase decrement shifts the colour");
    }

    /// W4-c39 — the hat strip columns (`Main.cs:26923-26939`): Blue 0,
    /// Pink 16, Cyan 17, Purple 18, White 19.
    #[test]
    fn w4_c39_the_hat_columns_match_the_strip() {
        assert_eq!(hat_frame_column(PartyHatColor::Blue), 0);
        assert_eq!(hat_frame_column(PartyHatColor::Pink), 16);
        assert_eq!(hat_frame_column(PartyHatColor::Cyan), 17);
        assert_eq!(hat_frame_column(PartyHatColor::Purple), 18);
        assert_eq!(hat_frame_column(PartyHatColor::White), 19);
    }

    // W4-c39 exec (research/114 §1g): the death-gore-rows witness DROPPED
    // with `party_death_gores` — d55's 7 gore witnesses cover the table
    // (incl. the shimmer variants) in the surviving `npc_hit_effect_a.rs`.

    /// W4-c39 exec (research/114 §0.3) — `UsesPartyHat` has NO life gate:
    /// the predicate answers the same on a corpse as on a living NPC (the
    /// `life <= 0` term is the CALLER's, `HitEffect` `:81660`). The earlier
    /// baked `hp > 0` made it always false on a corpse.
    #[test]
    fn w4_c39_uses_party_hat_is_life_agnostic() {
        let mut n = Npc::new(NpcKind::Merchant, 0.0, 0.0, 1, 0.0);
        n.hp = n.max_hp;
        assert!(uses_party_hat(&n, true), "a living town NPC hats up");
        n.hp = 0;
        assert!(
            uses_party_hat(&n, true),
            "the predicate has no life gate — the corpse still answers true"
        );
        assert!(!uses_party_hat(&n, false), "no party, no hat");
        // The exclusions still hold on a corpse.
        let mut tax = Npc::new(NpcKind::TaxCollector, 0.0, 0.0, 1, 0.0);
        tax.hp = 0;
        assert!(!uses_party_hat(&tax, true), "the Tax Collector never hats");
    }
}
