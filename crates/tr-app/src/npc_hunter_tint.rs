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
//!
//! **WIRING (W4-g78 wiring phase).** The 14 debuff arms are LIVE at the
//! uncontested NPC colour site (`render/entities.rs::draw_npcs`, the
//! `style.shade` result — `Main.DrawNPC`'s `npcColor` before the per-kind
//! draw, `Main.cs:22939`): [`tint_flags_from_buffs`] folds the live
//! `Npc.buffs` bar onto [`NpcTintFlags`] and the chain runs per NPC. The
//! Hunter floor's `hunter_active` input (`CanApplyHunterPotionEffects` =
//! the perspective player's `detectCreature`) needs the `draw_npcs`
//! signature threaded with the player flag — that tail is contested by
//! c39/d70, so it stays `false` here and the exact hunk is published in
//! research/77 §2. The G6 `npc_key` is widened to hash the tint-relevant
//! buff state (`npc.rs::render_key`, the uncontested free fn) so a buff
//! gain/loss invalidates the cached world range (the d70-tint precedent).

use crate::npc::Npc;

/// Fold the live `Npc.buffs` bar onto the tint flags — the
/// `UpdateNPC_BuffSetFlags` debuff ids (`NPC.cs:91965-92149`) the chain
/// reads, via `NpcBuffs::has_buff` (the `FindBuffIndex` immune-table gate).
/// `can_display_buffs` is false for the Golem body/head/fists (the C#
/// `canDisplayBuffs = false` arms `NPC.cs:18008-18021`, types 245-248; the
/// free head 249 keeps it true). `hunter_active`/`hunter_friendly` are the
/// contested Hunter-potion inputs — `false` until the research/77 §2
/// threading lands (the debuff arms are independent of them).
pub fn tint_flags_from_buffs(n: &Npc) -> NpcTintFlags {
    let b = &n.buffs;
    NpcTintFlags {
        // `NPC.cs:18008-18021` — 245/246/247/248 clear canDisplayBuffs.
        can_display_buffs: !matches!(
            n.kind,
            crate::npc::NpcKind::Golem
                | crate::npc::NpcKind::GolemHead
                | crate::npc::NpcKind::GolemFistLeft
                | crate::npc::NpcKind::GolemFistRight
        ),
        broken_armor: b.has_buff(36),          // `:92025-92028`
        poisoned: b.has_buff(20),              // `:91981-91984`
        venom: b.has_buff(70),                 // `:91997-92000`
        chlorophyte_spore: b.has_buff(397),    // `:92001-92004`
        potent_acid: b.has_buff(395),          // `:92005-92008`
        bleeding: b.has_buff(30),              // `:91989-91992`
        hemorrhage: b.has_buff(375),           // `:91993-91996`
        midas: b.has_buff(72),                 // `:92017-92020`
        betsys_curse: b.has_buff(203),         // `:92101-92104`
        oiled: b.has_buff(204),                // `:92105-92108`
        stinky: b.has_buff(120),               // `:92061-92064`
        dripping_slime: b.has_buff(137),       // `:92049-92052`
        dripping_sparkle_slime: b.has_buff(320), // `:92053-92056`
        ichor: b.has_buff(69),                 // `:92021-92024`
        // The contested Hunter inputs (research/77 §2): false until the
        // detect_creature threading + f47's friendly predicate land.
        hunter_active: false,
        life_max_gt_1: n.max_hp > 1,
        hunter_friendly: false,
    }
}

/// A stable hash of the tint-relevant buff state for the G6 `npc_key`
/// (`npc.rs::render_key`): two frames that differ only in an NPC's active
/// tint-buffs must not share a cached world range. Bit `i` = the i-th
/// `has_buff` flag in [`tint_flags_from_buffs`] order. The Golem
/// `canDisplayBuffs` refusal does NOT ride here — it is a static property
/// of the kind, and the kind already rides the key (the `<< 5` arm), so
/// buff-free worlds (Golem included) keep the pre-wiring key bit-for-bit.
/// ZERO RNG.
pub fn tint_key(n: &Npc) -> u32 {
    let f = tint_flags_from_buffs(n);
    let bits = [
        f.broken_armor,
        f.poisoned,
        f.venom,
        f.chlorophyte_spore,
        f.potent_acid,
        f.bleeding,
        f.hemorrhage,
        f.midas,
        f.betsys_curse,
        f.oiled,
        f.stinky,
        f.dripping_slime,
        f.dripping_sparkle_slime,
        f.ichor,
    ];
    let mut h = 0u32;
    for (i, b) in bits.iter().enumerate() {
        if *b {
            h |= 1 << i;
        }
    }
    h
}

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

    // ---- the WIRING witnesses (the buff fold + the G6 tint key) ----

    /// `UpdateNPC_BuffSetFlags`' ids (`NPC.cs:91981-92108`): the fold maps
    /// each buff id to its flag; the `FindBuffIndex` immune gate hides an
    /// immune buff (the slime's Poisoned-20 row, `NPCID.cs:6079-6084`).
    #[test]
    fn w4_g78_the_buff_fold_maps_the_csharp_ids() {
        let mut n = Npc::new(crate::npc::NpcKind::DemonEye, 0.0, 0.0, 1, 0.0);
        let f = tint_flags_from_buffs(&n);
        assert!(f.can_display_buffs);
        assert!(!f.poisoned && !f.ichor && !f.midas);
        n.buffs.add_buff(20, 600); // Poisoned
        n.buffs.add_buff(69, 600); // Ichor
        n.buffs.add_buff(72, 600); // Midas
        n.buffs.add_buff(320, 600); // drippingSparkleSlime
        let f = tint_flags_from_buffs(&n);
        assert!(f.poisoned && f.ichor && f.midas && f.dripping_sparkle_slime);
        assert!(!f.venom && !f.oiled, "unadded ids stay false");
        // The immune gate: a blue slime is immune to Poisoned 20 — the
        // AddBuff refuses, so the fold sees nothing.
        let mut s = Npc::new(crate::npc::NpcKind::Slime, 0.0, 0.0, 1, 0.0);
        s.buffs.add_buff(20, 600);
        assert!(!tint_flags_from_buffs(&s).poisoned);
    }

    /// `NPC.cs:18008-18021`: the Golem body/head/fists (245-248) clear
    /// `canDisplayBuffs`; the FREE head (249, `:18023-18026`) keeps it true.
    #[test]
    fn w4_g78_the_golem_parts_refuse_the_display() {
        use crate::npc::NpcKind::*;
        for k in [Golem, GolemHead, GolemFistLeft, GolemFistRight] {
            let n = Npc::new(k, 0.0, 0.0, 1, 0.0);
            assert!(!tint_flags_from_buffs(&n).can_display_buffs, "{k:?}");
            // The refusal returns the colour untouched even with ichor.
            let mut n = n;
            n.buffs.add_buff(69, 600);
            let f = tint_flags_from_buffs(&n);
            assert_eq!(npc_color_tinted_by_buffs([10, 20, 30, 255], &f), [10, 20, 30, 255]);
        }
        let n = Npc::new(GolemHeadFree, 0.0, 0.0, 1, 0.0);
        assert!(tint_flags_from_buffs(&n).can_display_buffs, "249 keeps true");
    }

    /// The G6 `npc_key` widening: an ichor gain changes the tint key, an
    /// expiry (time 0) restores it — two frames differing only in the buff
    /// bar must not share the cached world range (the d70-tint precedent).
    #[test]
    fn w4_g78_the_tint_key_tracks_the_buff_bar() {
        let mut n = Npc::new(crate::npc::NpcKind::DemonEye, 0.0, 0.0, 1, 0.0);
        let k0 = tint_key(&n);
        assert_eq!(k0, 0, "a clean bar hashes to zero");
        n.buffs.add_buff(69, 600);
        let k1 = tint_key(&n);
        assert_ne!(k0, k1, "the ichor gain must change the key");
        assert_eq!(k1, 1 << 13, "ichor is bit 13 of the fold order");
        n.buffs.buff_time[n.buffs.find_buff_index(69).unwrap()] = 0;
        assert_eq!(tint_key(&n), k0, "the expiry restores the key");
        // The Golem refusal does NOT ride the key (the kind already does):
        // a clean Golem hashes to zero like a clean DemonEye — buff-free
        // worlds keep the pre-wiring key bit-for-bit.
        let g = Npc::new(crate::npc::NpcKind::Golem, 0.0, 0.0, 1, 0.0);
        assert_eq!(tint_key(&g), 0);
    }

    /// The LIVE chain end-to-end: a poisoned DemonEye's light colour
    /// [200,200,200] rides the fold + the chain → R 200·0.65f32 = 130 (the
    /// 129.99999523 product rounds to 130.0f), G untouched 200, B
    /// 200·0.75 = 150. The hunter floor stays OFF (hunter_active false —
    /// the contested threading, research/77 §2).
    #[test]
    fn w4_g78_the_wired_chain_tints_a_poisoned_npc() {
        let mut n = Npc::new(crate::npc::NpcKind::DemonEye, 0.0, 0.0, 1, 0.0);
        n.buffs.add_buff(20, 600);
        let f = tint_flags_from_buffs(&n);
        assert!(!f.hunter_active, "the contested input stays false");
        assert_eq!(npc_color_tinted_by_buffs([200, 200, 200, 255], &f), [130, 200, 150, 255]);
    }
}
