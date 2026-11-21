//! W4-g78 DOC-SWEEP lane 4 — `NPC.GetNPCColorTintedByBuffs`
//! (`NPC.cs:91024-91141`) + `NPC.buffColor` (`:90986-90993`): the per-NPC
//! draw-colour buff chain, whose LAST arm is the Hunter-potion highlight
//! (`:91110-91139`). `Main.DrawNPC` calls it once per NPC
//! (`Main.cs:22939`, `npcColor = rCurrentNPC.GetNPCColorTintedByBuffs(npcColor)`),
//! AFTER the behind-tiles black-out scan (`:22922-22938`) — so a hunter-lit
//! NPC stays visible inside solid tiles (the `DoDraw_WallsTilesNPCs`
//! draw-order swap `Main.cs:63290-63299` moves the behind-tiles NPC pass
//! AFTER the solid tiles while `detectCreature` is armed).
//!
//! **PORTED-UNWIRED (code-only, W4-g78).** The consumer
//! (`render/entities.rs::draw_npcs`, the `style.shade` colour) is contested
//! by the npc-kind/style lanes (c10/c4/c6/c7/c9/d126/d127/d34/d35/d38); the
//! wiring lands with their merge. The chain is a PURE colour function —
//! every NPC input rides [`NpcTintFlags`], the caller folds the live
//! `UpdateNPC_BuffSetFlags` flags (`npc_buffs.rs`) onto it.
//!
//! Keep-one: the friendly predicate `NPC.AppearsFriendlyToHunterPotion`
//! (`:91008-91017`, the 689 override `NPCID.cs:10128-10132`) is f47's lane
//! (`tr_core::shimmer::appears_friendly_to_hunter_potion`, UNWIRED there) —
//! this module takes its RESULT as `hunter_friendly` and never re-ports it.
//! `CanApplyHunterPotionEffects` (`:91019-91022`) is just the perspective
//! player's `detectCreature` — the caller passes `hunter_active`.
//!
//! RNG census: ZERO draws — the whole chain is byte/f32 colour maths.
//! `buffColor` truncates each channel toward zero (`(byte)((float)(int)c * m)`,
//! the XNA byte-multiply convention); the multipliers ACCUMULATE across arms
//! (`num *= …`) and each arm re-applies the running product, exactly like
//! the C# (a later arm's multiply compounds the earlier ones).

/// `NPC.buffColor` (`NPC.cs:90986-90993`): per-channel
/// `(byte)((float)(int)channel * mult)` — the int widen then f32 multiply
/// then byte truncate (toward zero; the product of a 0..255 byte and a
/// 0..1 multiplier never exceeds 255 here, so no wrap). Alpha rides too.
pub fn buff_color(color: [u8; 4], r: f32, g: f32, b: f32, a: f32) -> [u8; 4] {
    [
        (color[0] as f32 * r) as u8,
        (color[1] as f32 * g) as u8,
        (color[2] as f32 * b) as u8,
        (color[3] as f32 * a) as u8,
    ]
}

/// The NPC flags `GetNPCColorTintedByBuffs` reads, in the C# arm order.
/// Every field mirrors the `UpdateNPC_BuffSetFlags` flag (`NPC.cs:91965-
/// 92149`) the caller folds from the live buff bar; `can_display_buffs` is
/// the `NPC.canDisplayBuffs` field (`:6199`, default true; the 245/246/247/
/// 248 mini-bosses clear it `:18010-18020`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NpcTintFlags {
    /// `canDisplayBuffs` (`:91026`) — false returns the colour untouched.
    pub can_display_buffs: bool,
    pub broken_armor: bool,        // `:91034`
    pub poisoned: bool,            // `:91041`
    pub venom: bool,               // `:91047`
    pub chlorophyte_spore: bool,   // `:91053`
    pub potent_acid: bool,         // `:91059`
    pub bleeding: bool,            // `:91065` (bleeding || hemorrhage)
    pub hemorrhage: bool,          // `:91065`
    pub midas: bool,               // `:91071`
    pub betsys_curse: bool,        // `:91077`
    pub oiled: bool,               // `:91083`
    pub stinky: bool,              // `:91090`
    pub dripping_slime: bool,      // `:91096`
    pub dripping_sparkle_slime: bool, // `:91102`
    pub ichor: bool,               // `:91106`
    /// `CanApplyHunterPotionEffects()` (`:91110`) — the perspective
    /// player's `detectCreature` (buff 17, `Player.cs:10021-10024`).
    pub hunter_active: bool,
    /// `lifeMax > 1` (`:91110`) — the hunter arm's second gate.
    pub life_max_gt_1: bool,
    /// `AppearsFriendlyToHunterPotion()` (`:91115`) — f47's predicate
    /// result (keep-one: `shimmer::appears_friendly_to_hunter_potion`).
    pub hunter_friendly: bool,
}

/// `NPC.GetNPCColorTintedByBuffs` (`NPC.cs:91024-91141`), verbatim arm
/// order. The running multipliers `num/num2/num3/a` start at 1 and each
/// debuff arm multiplies then re-applies `buffColor`; ichor REPLACES the
/// colour with (255,255,0,255); the hunter arm then floors each channel to
/// the friendly (50,255,50) or hostile (255,50,50) tint.
pub fn npc_color_tinted_by_buffs(color: [u8; 4], f: &NpcTintFlags) -> [u8; 4] {
    if !f.can_display_buffs {
        return color; // `:91026-91029`
    }
    let mut num = 1.0f32;
    let mut num2 = 1.0f32;
    let mut num3 = 1.0f32;
    let a = 1.0f32;
    let mut c = color;
    if f.broken_armor {
        num *= 0.75;
        num2 *= 0.75;
        num3 *= 0.75;
        c = buff_color(c, num, num2, num3, a); // `:91034-91040`
    }
    if f.poisoned {
        num *= 0.65;
        num3 *= 0.75;
        c = buff_color(c, num, num2, num3, a); // `:91041-91046`
    }
    if f.venom {
        num2 *= 0.45;
        num *= 0.75;
        c = buff_color(c, num, num2, num3, a); // `:91047-91052`
    }
    if f.chlorophyte_spore {
        num *= 0.65;
        num3 *= 0.75;
        c = buff_color(c, num, num2, num3, a); // `:91053-91058`
    }
    if f.potent_acid {
        num2 *= 0.45;
        num *= 0.75;
        c = buff_color(c, num, num2, num3, a); // `:91059-91064`
    }
    if f.bleeding || f.hemorrhage {
        num2 *= 0.85;
        num3 *= 0.85;
        c = buff_color(c, num, num2, num3, a); // `:91065-91070`
    }
    if f.midas {
        num3 *= 0.3;
        num *= 0.85;
        c = buff_color(c, num, num2, num3, a); // `:91071-91076`
    }
    if f.betsys_curse {
        num *= 0.8;
        num2 *= 0.6;
        c = buff_color(c, num, num2, num3, a); // `:91077-91082`
    }
    if f.oiled {
        num *= 0.4;
        num2 *= 0.4;
        num3 *= 0.4;
        c = buff_color(c, num, num2, num3, a); // `:91083-91089`
    }
    if f.stinky {
        num *= 0.7;
        num3 *= 0.55;
        c = buff_color(c, num, num2, num3, a); // `:91090-91095`
    }
    if f.dripping_slime {
        num *= 0.8;
        num2 *= 0.8;
        c = buff_color(c, num, num2, num3, a); // `:91096-91101`
    }
    if f.dripping_sparkle_slime {
        // `:91102-91105` — NOTE: no `num2 *= …`/`num3 *= …` accumulator
        // write; the arm passes `num2 * 0.75f, num3 * 0.85f` DIRECTLY (the
        // running multipliers stay unchanged for the hunter floor below).
        c = buff_color(c, num, num2 * 0.75, num3 * 0.85, a);
    }
    if f.ichor {
        c = [255, 255, 0, 255]; // `:91106-91109`
    }
    if f.hunter_active && f.life_max_gt_1 {
        // `:91110-91139` — the Hunter-potion floor: friendly green
        // (50,255,50), hostile red (255,50,50); each channel is a `max`.
        let (b, b2, b3) = if f.hunter_friendly {
            (50u8, 255u8, 50u8)
        } else {
            (255u8, 50u8, 50u8)
        };
        c[0] = c[0].max(b);
        c[1] = c[1].max(b2);
        c[2] = c[2].max(b3);
    }
    c
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `buffColor` (`:90986-90993`): the byte truncate toward zero —
    /// 200 * 0.65f32: the exact product 129.99999523… rounds to 130.0f
    /// (within half an f32 ulp of 130), so the truncate gives 130;
    /// 100 * 0.75 = 75 exact; 255 * 0.45f32 = 114.74999… → 114.
    #[test]
    fn w4_g78_buff_color_truncates_toward_zero() {
        assert_eq!(buff_color([200, 100, 255, 255], 0.65, 0.75, 0.45, 1.0), [130, 75, 114, 255]);
        // Identity multipliers keep the colour.
        assert_eq!(buff_color([10, 20, 30, 40], 1.0, 1.0, 1.0, 1.0), [10, 20, 30, 40]);
    }

    /// `:91026-91029`: `canDisplayBuffs == false` returns the colour
    /// untouched — even with every debuff + the hunter arm armed.
    #[test]
    fn w4_g78_no_display_returns_untouched() {
        let f = NpcTintFlags {
            can_display_buffs: false,
            hunter_active: true,
            life_max_gt_1: true,
            hunter_friendly: false,
            ichor: true,
            ..Default::default()
        };
        assert_eq!(npc_color_tinted_by_buffs([10, 20, 30, 255], &f), [10, 20, 30, 255]);
    }

    /// `:91110-91139`: the hunter floor — a dark hostile NPC lifts to
    /// (255,50,50), a friendly one to (50,255,50); a channel already above
    /// the floor is kept (the `max`).
    #[test]
    fn w4_g78_the_hunter_floor_lifts_each_channel() {
        let hostile = NpcTintFlags {
            can_display_buffs: true,
            hunter_active: true,
            life_max_gt_1: true,
            hunter_friendly: false,
            ..Default::default()
        };
        assert_eq!(npc_color_tinted_by_buffs([0, 0, 0, 255], &hostile), [255, 50, 50, 255]);
        // A bright green channel (200) stays above the hostile G floor 50.
        assert_eq!(npc_color_tinted_by_buffs([0, 200, 0, 255], &hostile), [255, 200, 50, 255]);
        let friendly = NpcTintFlags { hunter_friendly: true, ..hostile };
        assert_eq!(npc_color_tinted_by_buffs([0, 0, 0, 255], &friendly), [50, 255, 50, 255]);
    }

    /// `:91110`: the hunter arm needs BOTH `detectCreature` AND
    /// `lifeMax > 1` — a 1-life critter (the gold critters' lifeMax 5 pass,
    /// but a 1-life dummy does not) or an inactive potion floors nothing.
    #[test]
    fn w4_g78_the_hunter_arm_gates() {
        let base = NpcTintFlags { can_display_buffs: true, hunter_friendly: false, ..Default::default() };
        // lifeMax 1: no floor even with the potion active.
        let f = NpcTintFlags { hunter_active: true, life_max_gt_1: false, ..base };
        assert_eq!(npc_color_tinted_by_buffs([0, 0, 0, 255], &f), [0, 0, 0, 255]);
        // Potion inactive: no floor.
        let f = NpcTintFlags { hunter_active: false, life_max_gt_1: true, ..base };
        assert_eq!(npc_color_tinted_by_buffs([0, 0, 0, 255], &f), [0, 0, 0, 255]);
    }

    /// `:91106-91109`: ichor REPLACES the colour with (255,255,0,255)
    /// BEFORE the hunter floor — a hostile hunter-lit ichored NPC keeps
    /// G 255 (above the 50 floor) and lifts B to 50.
    #[test]
    fn w4_g78_ichor_replaces_then_hunter_floors() {
        let f = NpcTintFlags {
            can_display_buffs: true,
            ichor: true,
            hunter_active: true,
            life_max_gt_1: true,
            hunter_friendly: false,
            ..Default::default()
        };
        assert_eq!(npc_color_tinted_by_buffs([10, 20, 30, 255], &f), [255, 255, 50, 255]);
    }

    /// The accumulator compounding: brokenArmor (×0.75 all) THEN poisoned
    /// (num ×0.65, num3 ×0.75) re-applies the RUNNING product to the
    /// ALREADY-TRUNCATED colour — first arm [200,200,200] → [150,150,150];
    /// second arm R 150·(0.75f32·0.65f32) = 73.12499… → 73, G 150·0.75 =
    /// 112.5 → 112, B 150·0.5625 = 84.375 → 84 (the per-arm truncation
    /// compounds, exactly like the C#'s repeated buffColor calls).
    #[test]
    fn w4_g78_the_multipliers_compound_in_arm_order() {
        let f = NpcTintFlags {
            can_display_buffs: true,
            broken_armor: true,
            poisoned: true,
            ..Default::default()
        };
        assert_eq!(npc_color_tinted_by_buffs([200, 200, 200, 255], &f), [73, 112, 84, 255]);
    }
}
