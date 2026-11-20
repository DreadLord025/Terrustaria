//! W4-g35 — the NPC glow-mask census: every `TextureAssets.GlowMask[...]`
//! overlay `Main.DrawNPC` draws (`Main.cs:23406-26297`). There is NO
//! `NPC.glowMask` field and no static table in the C# — each type's draw arm
//! hard-codes its sheet index(es), gate and colour; this module ports those
//! RECORDS so the entity glow-draw pass binds exact data (the port's pillar
//! ghosts `render/entities.rs:3347-3392` already draw; this table pins their
//! ids and adds the missing census).
//!
//! Census notes (verbatim source state, not port gaps):
//! - The Firefly 355 / LightningBug 358 arms draw the NAMED sheets
//!   `Firefly`/`Lightningbug` (`:25699`/`:25703`), not GlowMask ids.
//! - The SolarFlare 516 arm draws `TextureAssets.Npc[type]` copies
//!   (`:26223-26227`), not a glow sheet.
//! - The Prime family (125/127/131/139/120) uses `EyeLaser`/`Probe`/
//!   `Bone_Eyes`/`Bone_Laser` named sheets (`:25577-25642`) — already bound
//!   in `render/entities.rs` (prime_eyes/bone_laser).
//! - The Golem 245/246/249 arms draw `Golem[1..3]`/`Extra[106/107]`
//!   (`:25729-25768`), not GlowMask ids.
//! - 393 (SaucerTurret) shares the 392-395 arm but draws NO glow (only the
//!   −8 origin shift, `:24594-24597`); 382 (RayGunner) draws `Extra[1]`
//!   only (`:25932`) — its glow is not a GlowMask sheet.

/// One NPC glow record: the type, its GlowMask sheet id(s) and the C# draw
/// site. `masks` holds every sheet the arm draws (the town variants and the
/// trail alternates included); `dynamic` marks the ids computed at draw time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NpcGlowRecord {
    pub npc: u16,
    pub masks: &'static [u16],
    /// The `Main.cs` draw line (the first GlowMask draw of the arm).
    pub line: u32,
}

/// The census, in `Main.DrawNPC` source order. Colour/gate detail rides the
/// per-row comments; the draw pass (f15-adjacent entity work) consumes it.
pub const NPC_GLOW_RECORDS: &[NpcGlowRecord] = &[
    // :23409 — VortexHornet, (255-alpha) grey a0.
    NpcGlowRecord { npc: 427, masks: &[152], line: 23409 },
    // :23412 — VortexHornetQueen, (128-alpha/2) grey a0.
    NpcGlowRecord { npc: 426, masks: &[153], line: 23412 },
    // :23425/:23427 — SandElemental, gold-multiplied (127-alpha) grey:
    // num55 ring copies *(1-num56) + the centre copy.
    NpcGlowRecord { npc: 541, masks: &[216], line: 23425 },
    // :23452/:23454 — DD2WitherBeastT2/T3, gate localAI[3] >= 60 (:23442),
    // colour9 = (127-alpha) grey * color5; ring + centre copies.
    NpcGlowRecord { npc: 568, masks: &[224], line: 23452 },
    NpcGlowRecord { npc: 569, masks: &[224], line: 23452 },
    // :23596 — DD2Betsy, Lerp(npcColor, White, 0.6) A=66 * (0.7+0.3*lerp2);
    // the lerpValue2=2f overwrite (:23579) makes the factor 1.3 verbatim.
    NpcGlowRecord { npc: 551, masks: &[226], line: 23596 },
    // :23941/:23953 — DD2DarkMageT1/T3, gate Opacity > 0 (:23936):
    // White A/2 *Opacity + 4 ring copies *0.35 *(1-num111).
    NpcGlowRecord { npc: 564, masks: &[225], line: 23941 },
    NpcGlowRecord { npc: 565, masks: &[225], line: 23941 },
    // :24039 — DD2EterniaCrystal, gate alpha < 255 (:24017):
    // (140,50,255,0)*0.6, 3 pulse copies at scale*0.75.
    NpcGlowRecord { npc: 548, masks: &[239], line: 24039 },
    // :24174 — LunarTowerStardust, 4 ghosts (64,64,64,0)*Opacity,
    // offset 4+|light-0.5|*4 (already drawn: entities.rs glow_stardust).
    NpcGlowRecord { npc: 493, masks: &[132], line: 24174 },
    // :24184 — LunarTowerNebula, same ghost form (entities.rs glow_nebula).
    NpcGlowRecord { npc: 507, masks: &[143], line: 24184 },
    // :24194 — LunarTowerVortex (entities.rs glow_vortex).
    NpcGlowRecord { npc: 422, masks: &[149], line: 24194 },
    // :24204 — LunarTowerSolar, offset 2+|light-0.5|*9 + UnitX*2
    // (entities.rs glow_solar).
    NpcGlowRecord { npc: 517, masks: &[162], line: 24204 },
    // :24306/:24312 — StardustWormHead 402: the oldPos trail alternates
    // GlowMask[133]/[134] per copy parity (:24295 Npc[403]/[404] twin),
    // the head draws [135]; colour (255,255,255,0)*(1-num339*i/2)*num343.
    NpcGlowRecord { npc: 402, masks: &[133, 134, 135], line: 24306 },
    // :24601 — MartianSaucer, (200,200,200,0) steady.
    NpcGlowRecord { npc: 392, masks: &[48], line: 24601 },
    // :24605 — MartianSaucerCore (the type==395 arm), same colour.
    NpcGlowRecord { npc: 395, masks: &[49], line: 24605 },
    // :24609 — MartianSaucerCannon (type==394), same colour.
    NpcGlowRecord { npc: 394, masks: &[50], line: 24609 },
    // :25070/:25074 — SolarDrakomireRider 416: (255-alpha) grey a=255-alpha
    // + 4 ghosts (64,64,64,0) at rider-velocity rotations (the 415 mount
    // sync block :25051-25063 feeds position9/spinningpoint6).
    NpcGlowRecord { npc: 416, masks: &[156], line: 25070 },
    // :25103 — MartianProbe 399 body, (127-alpha/2) grey a0.
    NpcGlowRecord { npc: 399, masks: &[100, 101], line: 25103 },
    // :25152/:25169 — SolarSolenian 419, gate ai[2] <= -9: the oldPos trail
    // re-frames GlowMask[154] rows (num205-3-i)%3 + 5, white *(10-i)/20 a0,
    // scale Lerp(0.75, 1.2, (10-i)/10).
    NpcGlowRecord { npc: 419, masks: &[154], line: 25152 },
    // :25381 — NebulaHeadcrab 421, gate ai[0] == 5 (the latch-on-player
    // arm): mask 146 at the PLAYER's Top/Bottom + direction*4, GetAlpha.
    NpcGlowRecord { npc: 421, masks: &[146], line: 25381 },
    // :25563 — FireImp 24, GetAlpha(White) full-frame overlay.
    NpcGlowRecord { npc: 24, masks: &[360], line: 25563 },
    // :25567 — the gem squirrels 639..645, GetAlpha(White).
    NpcGlowRecord { npc: 639, masks: &[286], line: 25567 },
    NpcGlowRecord { npc: 640, masks: &[286], line: 25567 },
    NpcGlowRecord { npc: 641, masks: &[286], line: 25567 },
    NpcGlowRecord { npc: 642, masks: &[286], line: 25567 },
    NpcGlowRecord { npc: 643, masks: &[286], line: 25567 },
    NpcGlowRecord { npc: 644, masks: &[286], line: 25567 },
    NpcGlowRecord { npc: 645, masks: &[286], line: 25567 },
    // :25571 — the gem bunnies 646..652, GetAlpha(White).
    NpcGlowRecord { npc: 646, masks: &[287], line: 25571 },
    NpcGlowRecord { npc: 647, masks: &[287], line: 25571 },
    NpcGlowRecord { npc: 648, masks: &[287], line: 25571 },
    NpcGlowRecord { npc: 649, masks: &[287], line: 25571 },
    NpcGlowRecord { npc: 650, masks: &[287], line: 25571 },
    NpcGlowRecord { npc: 651, masks: &[287], line: 25571 },
    NpcGlowRecord { npc: 652, masks: &[287], line: 25571 },
    // :25707 — Lavafly 654, (255,255,255,0) full-frame (the aiStyle-64
    // family's draw twin; the Firefly/LightningBug use NAMED sheets).
    NpcGlowRecord { npc: 654, masks: &[290], line: 25707 },
    // :25711 — HellButterfly 653, (255,255,255,0), the +3f (not +4f) Y.
    NpcGlowRecord { npc: 653, masks: &[288], line: 25711 },
    // :25772 — MartianOfficer 383, (255,255,255,0); the ForceBubble 384
    // companion copy (:25776) is Npc[384], not a glow sheet.
    NpcGlowRecord { npc: 383, masks: &[11], line: 25772 },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn w4_g35_the_landed_npc_records_match_the_source() {
        let r = |npc| NPC_GLOW_RECORDS.iter().find(|r| r.npc == npc).unwrap();
        assert_eq!(r(427).masks, &[152]);
        assert_eq!(r(426).masks, &[153]);
        assert_eq!(r(493).masks, &[132]);
        assert_eq!(r(568).masks, r(569).masks); // the shared WitherBeast arm
    }
}
