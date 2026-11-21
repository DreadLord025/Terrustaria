//! The projectile `Lighting.AddLight` sources, FIRST wave (`S1-E-5`, W4-c19).
//!
//! C# `Lighting.AddLight` funnels every overload into
//! `Lighting.AddLight(int i, int j, float r, float g, float b)`
//! (`Lighting.cs:157-163`), which appends to the active engine's per-frame
//! list behind `!Main.gamePaused && Main.netMode != 2`
//! (`LightingEngine.cs:51-54`). The engine then max-merges that list onto the
//! working light map (`LightingEngine.cs:208-235`), one frame late. There is
//! ONE list pair per engine — [`crate::entity_lights::EntityLights`] — and
//! this family feeds the same pair as the `ProjLight` chain
//! (`proj_lights.rs::proj_light`), the second-wave AI-arm family
//! (`proj_ai_lights.rs`, W4-e6), the item family
//! (`entity_lights.rs::item_light`), the NPC family (`npc_lights.rs`) and the
//! Player family (`player_lights.rs`); the tile-anchor vocabulary is shared
//! from `entity_lights.rs` too.
//!
//! Ported here: the **first 67** of `Projectile.cs`'s 107 `Lighting.AddLight`
//! call sites (sites 1-67 in line order, `:16655`…`:54071`). Sites 68-107 are
//! W4-e6's landed second wave (`proj_ai_lights.rs`); 67 + 40 = 107 — this
//! slice CLOSES the `Projectile.cs` AddLight family. The site lines are frozen
//! verbatim in `crates/tr-app/tests/proj_addlight_census.rs` (W4-f7
//! `67110ccf`); a decomp refresh that moves any line fails that test before
//! this port can silently drift.
//!
//! Like the `ProjLight` chain and the e6 second wave, an arm whose kind has no
//! `proj_data` row is absent wholesale (no slot can carry it), and no ported
//! arm draws `Main.rand` — the feed is pure, so its placement cannot move any
//! stream. A light gated on a `Main.rand` draw is absent wholesale (the e6
//! RNG rule): the 409 Typhoon (`:30378`, inside `Next(3) == 0`), the 849
//! ScytheWhip (`:47846`) and the 912 CoolWhip (`:48011`).
//!
//! **The site register (all 67, in line order):**
//!
//! * `:16655` — site 1, the 870 Jack 'O Lantern's `FloodFillTile` +
//!   `AddLight(Center, rgb)` pair (`ProjLight`'s 870 arm). AUDIT-LANDED: the
//!   centre `AddLight` rides the `ProjLight` common tail (`proj_lights.rs`,
//!   the 870 row's `light`); the `FloodFillTile` light-map spread has no port
//!   (the documented skip). No new code.
//! * `:16659` — site 2, the `ProjLight` common tail
//!   `(int)((position + width/2) / 16)`. AUDIT-LANDED: live in
//!   `proj_lights.rs::proj_light` for every lit row. No new code.
//! * `:17343` — site 3, `EmitEnchantmentVisualsAt`'s mount light (the
//!   Chillet mounts 62-65, `(0.6,0.3,0.1)`/`(0.5,0.1,0.6)` × 0.5 at the
//!   projectile centre). Absent: the port's `mount.rs` carries no 62-65
//!   `MountData` row, so `player.mount.kind()` can never read them; the
//!   `hostile`/`noEnchantments` gates have no slot fields either.
//! * `:24307` — site 4, the 502 Meowmere's disco light
//!   (`(0.5 + DiscoX/255)/2` per channel at `Center`, aiStyle-8 tail).
//!   Absent: 502 has no `proj_data` row.
//! * `:24877`/`:24881` — sites 5/6, the 346 OrnamentHostile's frame pair
//!   (aiStyle-14): `frame = (int)ai[1]`, then `frame == 0` →
//!   `AddLight(Center, 0.25, 0.2, 0)` else `AddLight(Center, 0.15, 0.15,
//!   0.15)`. PORTED.
//! * `:25394` — site 7, the 1092 LibrarianSkeletonBook's
//!   `AddLight(Center, 0.3, 0.25, 0.1)` (aiStyle-18 head). Absent: 1092 has
//!   no row.
//! * `:27920` — site 8, the 732 ThunderSpearShot's
//!   `AddLight(Center, 0.3, 0.6, 0.8)` (aiStyle-44 type-732 arm, after the
//!   rand dust — the light itself is unconditional). PORTED.
//! * `:29674`/`:29678` — sites 9/10, the 533 DeadlySphere's pair
//!   (aiStyle-66): `(int)ai[0] % 3 != 2` → `AddLight(Center, 0.8, 0.3, 0.1)`
//!   else `AddLight(Center, 0.3, 0.5, 0.7)`. PORTED.
//! * `:30378` — site 11, the 409 Typhoon's
//!   `AddLight((int)Center/16, 0.1, 0.4, 0.6)`, INSIDE
//!   `localAI[1] > 10 && Main.rand.Next(3) == 0`. Absent wholesale (the RNG
//!   rule): the light rides a rand draw the pure feed cannot take. (The
//!   sim's `ai_typhoon` already runs the fade block unconditionally — a
//!   recorded deviation — but the light stays absent until the gate lands.)
//! * `:31212` — site 12, the 451 InfluxWaver's alpha-scaled
//!   `AddLight((int)Center/16, 0.3·a, 0.4·a, 1·a)` (aiStyle-81 tail).
//!   Absent: 451 has no row.
//! * `:31864`/`:31893` — sites 13/14, the 464 CultistBossIceMist's
//!   `AddLight(Center, 0.3, 0.75, 0.9)` (aiStyle-86, the `ai[1]==1` spin and
//!   the 6-point ring). Absent: 464 has no row.
//! * `:31998` — site 15, the 465 CultistBossLightningOrb's
//!   `AddLight(Center, 0.4, 0.85, 0.9)` (aiStyle-88). Absent: 465 no row.
//! * `:32049` — site 16, the 466 CultistBossLightningOrbArc's
//!   `AddLight(Center, 0.3, 0.45, 0.5)` (aiStyle-88). Absent: 466 no row.
//! * `:32155` — site 17, the 580 VortexLightning's
//!   `AddLight(Center, 0.3, 0.45, 0.5)` (aiStyle-88's `type == 580` else
//!   arm, after `frameCounter++`). PORTED.
//! * `:32384` — site 18, the 490 CultistRitual's
//!   `AddLight(Center, 1.1, 0.9, 0.4)` (aiStyle-89 tail). Absent: 490 no row.
//! * `:32410` — site 19, the 492 MagicLantern's
//!   `AddLight(base.position, 0.35, 0.35, 0.1)` (aiStyle-90). Absent: 492 no
//!   row.
//! * `:32962` — site 20, the 518 CoinPortal's
//!   `AddLight(Center, 0.8·0.3, 181/255·0.3, 72/255·0.3)` (aiStyle-94 tail).
//!   Absent: 518 no row.
//! * `:33580` — site 21, the 575 NebulaSphere's
//!   `AddLight(Center, 0.5, 0.1, 0.3)` (aiStyle-107). Absent: 575 no row.
//! * `:34226` — site 22, the 590 TruffleSpore's rand-flicker
//!   `AddLight(Center, …)` (aiStyle-112, `alpha == 0` + `Next(28,42)`).
//!   Absent: 590 no row (and rand-bearing).
//! * `:34265` — site 23, the 644 RainbowCrystalExplosion's
//!   `AddLight(Center, hslToRgb(ai[0],1,0.5).ToVector3()·0.5)` (aiStyle-112,
//!   `alpha == 0`). PORTED.
//! * `:34402` — site 24, the 602 PortalGunGate's
//!   `AddLight(Center + velocity·3, portalColor·0.5)` (aiStyle-114,
//!   `alpha == 0`). Absent: 602 no row.
//! * `:34416` — site 25, the 604 TerrarianBeam's
//!   `AddLight(Center, 0.075, 0.3, 0.15)` (aiStyle-115 head, unconditional).
//!   PORTED (604 is the style's only rowed kind).
//! * `:34487` — site 26, the aiStyle-117 `flag58` trio 612/953/978's
//!   `AddLight(Center, 0.9, 0.8, 0.6)`. Absent: none of 612/953/978 rowed.
//! * `:34802` — site 27, the 617 NebulaArcanum's
//!   `AddLight(Center, 0.7, 0.2, 0.6)` (aiStyle-118 tail, `alpha < 150`).
//!   PORTED.
//! * `:36150` — site 28, the 658 SandnadoHostileMark's
//!   `AddLight(Center, white·0.5)` (aiStyle-128, `alpha == 0`). Absent: 658
//!   no row.
//! * `:37022` — site 29, the 1115 GlacierFangShot's
//!   `AddLight(Center, (0.2,0.5,0.8)·Remap(ai[0],0,60,0.5,1))`
//!   (AI_208 tail). Absent: 1115 no row.
//! * `:37070` — site 30, the 641 MoonlordTurret's
//!   `AddLight(Center, (0.3,0.9,0.7)·Opacity)` (AI_123, the 641 arm).
//!   PORTED.
//! * `:37821` — site 31, the 731 ThunderStaffShot's
//!   `AddLight(Center, 0.2, 0.5, 0.7)` (AI_029's 731 arm). Absent: 731 no
//!   row.
//! * `:38000` — site 32, the gem bolts' per-kind
//!   `AddLight(Center, num25, num26, num27)` (AI_029_GemStaffs_ActuallyGemStaffs
//!   tail, `num24 = (type-121+86) - 85`): 121 (0.9,0,0.9), 122 (0.9,0.9,0),
//!   123 (0,0.1,1.3), 124 (0,1,0.1), 125 (1,0.1,0.1), 126 (1.3,1.3,1.3),
//!   597 amber (0.9,0.6,0.2). PORTED (the 7 rowed gems; the 1120
//!   ForbiddenMinionShot's (1,1,1) default has no row).
//! * `:38458` — site 33, the 1105 RemoteControlCar's
//!   `AddLight(Center, One·0.3)` (AI_205 tail). Absent: 1105 no row.
//! * `:38707` — site 34, the 1098 PalworldDigtoise's
//!   `AddLight(Center, 0.3, 0.3, 0.1)` (AI_204). Absent: 1098 no row.
//! * `:39020`/`:39021` — sites 35/36, the phaseblade/phaser pair
//!   `AddLight((int)(Center/16), r,g,b)` + the tile above (AI_003's
//!   `IsAPhaseblade` block, `Item.GetPhaseColor·0.5`). Absent: no
//!   1059-1072/1074-1076/1101/1102 row.
//! * `:39783` — site 37, the storm-lightning's
//!   `AddLight(Center, GetLightningColor·num9)` (AI_203, types 1091/1117).
//!   Absent: neither rowed.
//! * `:40038` — site 38, the 1078 MeteorOre's `AddLight(Center, 1,0,0)`
//!   (AI_199). Absent: 1078 no row.
//! * `:40046` — site 39, the 567/568 SporeTrap's
//!   `AddLight(Center, 0.2·n, 0.275·n, 0.075·n)` (AI_105 head). Absent:
//!   neither rowed.
//! * `:40507` — site 40, the AI_196_Petal early-return
//!   `AddLight(Center, vector)` (`ai[0] < 20 && ai[0] >= 10`). The reaching
//!   kinds are 1038/1116 (aiStyle 196); the rowed 248 FlowerPowPetal is
//!   aiStyle 1 and never reaches AI_196. Absent: 1038/1116 no row.
//! * `:40513` — site 41, the 1116 PetalstormShot's
//!   `AddLight(Center, vector·Remap(timeLeft,120,0,1,0))` (AI_196). Absent:
//!   1116 no row.
//! * `:40892` — site 42, the 614 StardustCellMinionShot's
//!   `AddLight(Center, 0.2, 0.6, 0.7)` (AI_113 tail). Absent: 614 no row.
//! * `:40896` — site 43, the 636 Daybreak's
//!   `AddLight(Center, 0.8, 0.7, 0.4)` (AI_113 tail). Absent: 636 no row.
//! * `:41031` — site 44, the 1020 JimsDrone's `AddLight(Center, One·0.3)`
//!   (AI_195 tail). Absent: 1020 no row.
//! * `:41775` — site 45, the 973 TrueNightsEdge's lead-anchor
//!   `AddLight(Center + rot·85·scale, (64,220,96))` (AI_191). Absent: 973 no
//!   row.
//! * `:42181` — site 46, the 972 NightsEdge's
//!   `AddLight(Center, (0,96,0) [lerp])` (AI_190). Absent: 972 no row.
//! * `:42206` — site 47, the 984 TerraBlade2's
//!   `AddLight(Center, (15,84,125))` (AI_190's 984 arm). Absent: 984 no row.
//! * `:43172`/`:43173` — sites 48/49, the 933/1100 FinalFractal's
//!   trail + mounted-centre pair (AI_182). Absent: neither rowed.
//! * `:44612` — site 50, the 535 MedusaHead's
//!   `AddLight(Center, 0.9, 0.75, 0.1)` (AI_100). Absent: 535 no row.
//! * `:44662` — site 51, the 623 StardustGuardian's
//!   `AddLight(Center, 0.9, 0.9, 0.7)` (AI_120's 623 arm). PORTED.
//! * `:46037` — site 52, the 342 NorthPole's
//!   `AddLight(Center, 0.65, 0.9, 1)` (AI_019_Spears' 342 arm). PORTED.
//! * `:46588` — site 53, the 856 SparkleGuitar's
//!   `AddLight(Center, HotPink.ToVector3()·0.3)` (AI_167's 856 arm). PORTED.
//! * `:47569` — site 54, the 855 ReleaseLantern's
//!   `AddLight(Center, (1,0.6,0.1)·(1-alpha/255))` (AI_166). Absent: 855 no
//!   row.
//! * `:47649` — site 55, the 1127 ChlorophyteSaberStab's
//!   `AddLight(Center, (32,108,0))` (AI_161). Absent: 1127 no row.
//! * `:47846` — site 56, the 849 ScytheWhip's
//!   `AddLight(r2.Center, 0.2, 0, 0.4)` (AI_165's 849 arm), INSIDE the
//!   `Next(9) != 0` rand gate. Absent wholesale (the RNG rule).
//! * `:48011` — site 57, the 912 CoolWhip's
//!   `AddLight(r16.Center, 0.1, 0.1, 0.2)` (AI_165's 912 arm), INSIDE the
//!   `NextFloat() < num28/2` rand gate. Absent wholesale (the RNG rule).
//! * `:49283`/`:49287`/`:49291` — sites 58/59/60, the Storm Tigers'
//!   `AddLight(Center, …)` (AI_067's `flag3`, `ai[0] != 4`): 833 One·0.5,
//!   834 One·0.8, 835 `Lerp(OurFavoriteColor, White, 0.8)·1`. PORTED — the
//!   tiger AI arm is deferred (the dispatch gates 833-835 out of
//!   `ai_freaking_pirates`), so the slot keeps its spawn `ai[0]` and the
//!   light reads it; the colour/gate are faithful to the slot state (the
//!   e6 LaserDrill ported-but-dark precedent, here ported-and-live at the
//!   spawn state). ANALYSIS records the deviation.
//! * `:49464` — site 61, the 500 CrimsonHeart's
//!   `AddLight(Center, 0.9, 0.1, 0.3)` (AI_067's `type == 500` arm). PORTED.
//! * `:51492` — site 62, the 756 SharpTears'
//!   `AddLight(Center, (0.5,0.1,0.1)·scale)` (AI_157's `type == 756` arm).
//!   Absent: 756 no row (the rowed 961 DeerclopsIceSpike rides the same
//!   style but the light gates on 756 specifically).
//! * `:52606` — site 63, the 446 AntiGravityHook's
//!   `AddLight(mountedCenter, 0, 0.4, 0.3)` (AI_007's 446 arm; the
//!   `PlotTileLine` rope spread is the light-map write with no port —
//!   skipped). PORTED.
//! * `:52641`/`:52642` — sites 64/65, the lunar hooks 646-649's
//!   `AddLight(mountedCenter, vector2)` + `AddLight(Center, vector2)` pair
//!   (AI_007's 646-649 arm): 646 (0.7,0.5,0.1), 647 (0,0.6,0.7),
//!   648 (0.6,0.2,0.6), 649 (0.6,0.6,0.9); the `PlotTileLine` rope spread
//!   skipped. PORTED.
//! * `:52890` — site 66, the Celebration Mk2 rockets 715-718's
//!   `AddLight(Center, GetCeleb2Color().ToVector3()·0.5)` (AI_147 head,
//!   the `(int)ai[0]` palette). PORTED.
//! * `:54071` — site 67, the 601 PortalGunBolt's
//!   `AddLight(Center + velocity·3, GetPortalColor(owner, ai[0])·0.5)`
//!   (AI_001's 601 arm). PORTED — the portal teleport AI is unported, so
//!   `ai[0]` keeps its spawn value and the colour reads it (the e6
//!   ported-but-dark precedent); the single-player `netMode == 0` fold
//!   picks the orange/blue portal hue by `ai[0] != 0`. ANALYSIS records it.
//!
//! The anchor forms: the `AddLight(Vector2, …)` overload
//! (`Lighting.cs:140-143`) truncates the QUOTIENT (`(int)(v / 16f)`,
//! [`XAnchor::Center`]); the raw-vector anchors (the hooks' `mountedCenter`,
//! the 601/602 `Center + velocity·3` lead) use the same quotient truncation
//! on the computed vector. The `(int)Center/16` form (the absent 409/451
//! arms) truncates first — kept distinct in the register, but the two agree
//! for every real coordinate (the e6 anchor-agreement invariant).

use glam::Vec2;

use crate::entity_lights::{tile_x, tile_y, XAnchor};
use crate::projectile::{
    Projectile, DEADLY_SPHERE, GEM_BOLT_AMBER, GEM_BOLT_AMETHYST, GEM_BOLT_DIAMOND,
    GEM_BOLT_EMERALD, GEM_BOLT_RUBY, GEM_BOLT_SAPPHIRE, GEM_BOLT_TOPAZ, MOONLORD_TURRET,
    NEBULA_ARCANUM, NORTH_POLE, ORNAMENT_HOSTILE, RAINBOW_CRYSTAL_EXPLOSION, SPARKLE_GUITAR,
    STARDUST_GUARDIAN, STORM_TIGER_1, STORM_TIGER_2, STORM_TIGER_3, TERRARIAN_BEAM,
    THUNDER_SPEAR_BOLT, VORTEX_RITUAL_BOLT,
};

/// The centre tile of a slot's post-step box, quotient-truncated (the
/// `AddLight(Vector2, …)` overload, `Lighting.cs:140-143`).
fn centre_tile(p: &Projectile) -> (i32, i32) {
    (
        tile_x(p.pos, p.w, XAnchor::Center),
        tile_y(p.pos, p.h, XAnchor::Center),
    )
}

/// The first-wave AI-arm `AddLight` sources this projectile contributes for
/// one frame, in C# call order (the lunar-hook arms append two), or an empty
/// list.
///
/// `p` is the slot's post-step state; `mounted_center` is the owner's
/// `MountedCenter` (the hooks' anchor, `Projectile.cs:52572`), resolved by the
/// feed for `owner == 0` to the local player (the e6 single-player fold) and
/// `None` otherwise. Pure — no arm draws RNG and none mutates the slot, so
/// the feed's placement cannot move any stream.
pub(crate) fn proj_lights_a(
    p: &Projectile,
    mounted_center: Option<Vec2>,
) -> Vec<(i32, i32, [f32; 3])> {
    let mut out = Vec::new();
    let (cx, cy) = centre_tile(p);
    // The per-type arms, in source order. An arm whose kind has no
    // `proj_data` row is absent wholesale (the register above).
    if p.kind == ORNAMENT_HOSTILE {
        // `:24874-24882` — `frame = (int)ai[1]` (the sim's `ornament` writes
        // it), then the frame-0 / else pair at the centre.
        let rgb = if p.frame == 0 {
            [0.25, 0.2, 0.0]
        } else {
            [0.15, 0.15, 0.15]
        };
        out.push((cx, cy, rgb));
    } else if p.kind == THUNDER_SPEAR_BOLT {
        // `:27920` — the aiStyle-44 type-732 arm's unconditional centre
        // light (after the rand dust, which the feed does not take).
        out.push((cx, cy, [0.3, 0.6, 0.8]));
    } else if p.kind == DEADLY_SPHERE {
        // `:29670-29679` — `(int)ai[0] % 3 != 2` picks the warm/cool pair.
        let rgb = if (p.ai[0] as i32) % 3 != 2 {
            [0.8, 0.3, 0.1]
        } else {
            [0.3, 0.5, 0.7]
        };
        out.push((cx, cy, rgb));
    } else if p.kind == VORTEX_RITUAL_BOLT {
        // `:32145-32155` — aiStyle-88's `else` arm: `type != 580` returns,
        // then `frameCounter++` and the unconditional centre light (the
        // sim's `ai_vortex_bolt` runs the wrap latch / wand around it).
        out.push((cx, cy, [0.3, 0.45, 0.5]));
    } else if p.kind == RAINBOW_CRYSTAL_EXPLOSION {
        // `:34239` + `:34263-34265` — `newColor3 =
        // Main.hslToRgb(ai[0], 1f, 0.5f)` computed at the arm head; the
        // light fires at `alpha == 0` after the fade (the sim's
        // `ai_rainbow_crystal_shard` runs it). `ToVector3()` divides the
        // bytes by 255, then × 0.5.
        if p.alpha == 0 {
            let c = crate::ui::color::hsl_to_rgb(p.ai[0] as f64, 1.0, 0.5, 255);
            out.push((
                cx,
                cy,
                [
                    c[0] as f32 / 255.0 * 0.5,
                    c[1] as f32 / 255.0 * 0.5,
                    c[2] as f32 / 255.0 * 0.5,
                ],
            ));
        }
    } else if p.kind == TERRARIAN_BEAM {
        // `:34416` — aiStyle-115's head light, unconditional (the style's
        // only rowed kind is the 604).
        out.push((cx, cy, [0.075, 0.3, 0.15]));
    } else if p.kind == NEBULA_ARCANUM {
        // `:34800-34802` — aiStyle-118's tail: `alpha < 150` gates the
        // centre light (the sim's `ai_nebula_arcanum` runs the −15 fade).
        if p.alpha < 150 {
            out.push((cx, cy, [0.7, 0.2, 0.6]));
        }
    } else if p.kind == MOONLORD_TURRET {
        // `:37070` — AI_123's 641 arm: `(0.3, 0.9, 0.7) * this.Opacity`.
        // `Opacity` is the `1 - alpha/255` view (`Projectile.cs:540-545`);
        // the sim's `ai_floating_turrets` runs the −5 fade and the scale
        // pin before this read.
        let opacity = 1.0 - p.alpha as f32 / 255.0;
        out.push((cx, cy, [0.3 * opacity, 0.9 * opacity, 0.7 * opacity]));
    } else if let Some(rgb) = gem_bolt_light(p.kind) {
        // `:37954-38000` — AI_029_GemStaffs_ActuallyGemStaffs' tail:
        // `num23 = 1f`, the per-gem switch on `num24 = (type - 121 + 86)
        // - 85` (the 597 amber remaps `num = 262` first, `:37832-37834`),
        // then `AddLight(Center, num23·num25, num23·num26, num23·num27)`.
        // The 1120 ForbiddenMinionShot's default (1,1,1) has no row.
        out.push((cx, cy, rgb));
    } else if p.kind == STARDUST_GUARDIAN {
        // `:44662` — AI_120's 623 arm: (0.9, 0.9, 0.7) at the centre,
        // unconditional inside the `flag` (623) block.
        out.push((cx, cy, [0.9, 0.9, 0.7]));
    } else if p.kind == NORTH_POLE {
        // `:46037` — AI_019_Spears' 342 arm: (0.65, 0.9, 1) at the centre,
        // unconditional (the trailing dust is the documented skip).
        out.push((cx, cy, [0.65, 0.9, 1.0]));
    } else if p.kind == SPARKLE_GUITAR {
        // `:46588` — AI_167's 856 arm: `Color.HotPink.ToVector3() * 0.3f`.
        // XNA's HotPink is (255, 105, 180); ToVector3 divides by 255.
        out.push((
            cx,
            cy,
            [
                255.0 / 255.0 * 0.3,
                105.0 / 255.0 * 0.3,
                180.0 / 255.0 * 0.3,
            ],
        ));
    } else if p.kind == STORM_TIGER_1 || p.kind == STORM_TIGER_2 || p.kind == STORM_TIGER_3 {
        // `:49271-49292` — AI_067's `flag3` block, gated on `ai[0] != 4f`
        // (the special-attack state). The tiger AI arm is deferred (the
        // dispatch gates 833-835 out of `ai_freaking_pirates`), so `ai[0]`
        // keeps its spawn value; a default spawn reads 0 (≠ 4) and the light
        // is live, a spawn into the attack state reads 4 and stays dark. The
        // colour/gate are faithful to the slot state (the e6 LaserDrill
        // ported-but-dark precedent, here state-driven; ANALYSIS records it).
        if p.ai[0] != 4.0 {
            let rgb = match p.kind {
                // `:49283` — 833: `Vector3.One * 0.5f`.
                STORM_TIGER_1 => [0.5, 0.5, 0.5],
                // `:49287` — 834: `Vector3.One * 0.8f`.
                STORM_TIGER_2 => [0.8, 0.8, 0.8],
                // `:49291` — 835: `Color.Lerp(Main.OurFavoriteColor,
                // Color.White, 0.8f).ToVector3() * 1f`. OurFavoriteColor is
                // (255, 231, 69) (`Main.cs:66748`); XNA's Color.Lerp truncates
                // each byte (`(byte)MathHelper.Lerp`), giving (255, 250, 217).
                _ => [
                    255.0 / 255.0,
                    250.0 / 255.0,
                    217.0 / 255.0,
                ],
            };
            out.push((cx, cy, rgb));
        }
    }
    out
}

/// The gem bolts' per-kind light tint (`Projectile.cs:37959-38000`): the
/// switch arms on `num24 = num - 85` where `num = type - 121 + 86` (the 597
/// amber is `num = 262`, `:37832-37834`), `num23 = 1f`. `None` for every
/// non-gem kind (the 1120's `default` (1,1,1) arm has no row — absent).
fn gem_bolt_light(kind: u16) -> Option<[f32; 3]> {
    match kind {
        // case 1 (`:37976-37980`): num25 *= 0.9, num26 *= 0, num27 *= 0.9.
        GEM_BOLT_AMETHYST => Some([0.9, 0.0, 0.9]),
        // case 2 (`:37986-37990`): 0.9 / 0.9 / 0.
        GEM_BOLT_TOPAZ => Some([0.9, 0.9, 0.0]),
        // case 3 (`:37961-37965`): 0 / 0.1 / 1.3.
        GEM_BOLT_SAPPHIRE => Some([0.0, 0.1, 1.3]),
        // case 4 (`:37971-37975`): 0 / 1 / 0.1.
        GEM_BOLT_EMERALD => Some([0.0, 1.0, 0.1]),
        // case 5 (`:37966-37970`): 1 / 0.1 / 0.1.
        GEM_BOLT_RUBY => Some([1.0, 0.1, 0.1]),
        // case 6 (`:37981-37985`): 1.3 on all three.
        GEM_BOLT_DIAMOND => Some([1.3, 1.3, 1.3]),
        // default with num == 262 (`:37991-37998`): 0.9 / 0.6 / 0.2.
        GEM_BOLT_AMBER => Some([0.9, 0.6, 0.2]),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectile::{proj_data, Projectile};

    fn slot(kind: u16) -> Projectile {
        let mut p = Projectile::default();
        p.active = true;
        p.kind = kind;
        p.pos = Vec2::new(160.0, 176.0);
        p.w = 16.0;
        p.h = 16.0;
        p
    }

    #[test]
    fn the_ornament_frame_pair_picks_by_frame() {
        // `:24877`/`:24881` — frame 0 → (0.25, 0.2, 0); else (0.15, 0.15,
        // 0.15). The 16x16 box at (160, 176) centres on (168, 184) → (10, 11).
        let mut p = slot(ORNAMENT_HOSTILE);
        p.frame = 0;
        assert_eq!(proj_lights_a(&p, None), vec![(10, 11, [0.25, 0.2, 0.0])]);
        p.frame = 1;
        assert_eq!(
            proj_lights_a(&p, None),
            vec![(10, 11, [0.15, 0.15, 0.15])]
        );
    }

    #[test]
    fn the_thunder_spear_bolt_lights_a_fixed_blue() {
        // `:27920` — (0.3, 0.6, 0.8) at the centre, unconditional.
        let p = slot(THUNDER_SPEAR_BOLT);
        assert_eq!(proj_lights_a(&p, None), vec![(10, 11, [0.3, 0.6, 0.8])]);
    }

    #[test]
    fn the_deadly_sphere_picks_by_ai0_mod_3() {
        // `:29674`/`:29678` — `(int)ai[0] % 3 != 2` → warm, else cool.
        let mut p = slot(DEADLY_SPHERE);
        p.ai[0] = 0.0;
        assert_eq!(proj_lights_a(&p, None)[0].2, [0.8, 0.3, 0.1]);
        p.ai[0] = 1.0;
        assert_eq!(proj_lights_a(&p, None)[0].2, [0.8, 0.3, 0.1]);
        p.ai[0] = 2.0;
        assert_eq!(proj_lights_a(&p, None)[0].2, [0.3, 0.5, 0.7]);
        p.ai[0] = 5.0;
        assert_eq!(proj_lights_a(&p, None)[0].2, [0.3, 0.5, 0.7]);
    }

    #[test]
    fn the_vortex_ritual_bolt_lights_a_fixed_teal() {
        // `:32155` — (0.3, 0.45, 0.5) at the centre, unconditional inside
        // the aiStyle-88 type-580 else arm.
        let p = slot(VORTEX_RITUAL_BOLT);
        assert_eq!(proj_lights_a(&p, None), vec![(10, 11, [0.3, 0.45, 0.5])]);
    }

    #[test]
    fn the_rainbow_crystal_shard_lights_its_hsl_hue_at_alpha_zero() {
        // `:34265` — hslToRgb(ai[0], 1, 0.5) · 0.5, gated on alpha == 0.
        // Hue 0 → (255, 0, 0) → (0.5, 0, 0).
        let mut p = slot(RAINBOW_CRYSTAL_EXPLOSION);
        p.alpha = 0;
        p.ai[0] = 0.0;
        assert_eq!(proj_lights_a(&p, None), vec![(10, 11, [0.5, 0.0, 0.0])]);
        // The fade gate: alpha > 0 → dark.
        p.alpha = 8;
        assert!(proj_lights_a(&p, None).is_empty());
    }

    #[test]
    fn the_terrarian_beam_lights_a_fixed_green() {
        // `:34416` — (0.075, 0.3, 0.15), unconditional.
        let p = slot(TERRARIAN_BEAM);
        assert_eq!(
            proj_lights_a(&p, None),
            vec![(10, 11, [0.075, 0.3, 0.15])]
        );
    }

    #[test]
    fn the_nebula_arcanum_gates_on_alpha_under_150() {
        // `:34802` — (0.7, 0.2, 0.6) only while alpha < 150.
        let mut p = slot(NEBULA_ARCANUM);
        p.alpha = 149;
        assert_eq!(proj_lights_a(&p, None)[0].2, [0.7, 0.2, 0.6]);
        p.alpha = 150;
        assert!(proj_lights_a(&p, None).is_empty());
    }

    #[test]
    fn the_moonlord_turret_scales_by_opacity() {
        // `:37070` — (0.3, 0.9, 0.7) · (1 − alpha/255). alpha 0 → full.
        let mut p = slot(MOONLORD_TURRET);
        p.alpha = 0;
        assert_eq!(proj_lights_a(&p, None)[0].2, [0.3, 0.9, 0.7]);
        // alpha 255 → Opacity 0 → a zeroed light is still appended (the
        // C# calls AddLight unconditionally inside the 641 arm).
        p.alpha = 255;
        assert_eq!(proj_lights_a(&p, None)[0].2, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn the_gem_bolts_take_their_per_kind_tints() {
        // `:38000` — the switch on num24 = type − 120 (the 597 amber remaps
        // to 262 → the default arm). Each tint at the centre.
        let cases = [
            (GEM_BOLT_AMETHYST, [0.9, 0.0, 0.9]),
            (GEM_BOLT_TOPAZ, [0.9, 0.9, 0.0]),
            (GEM_BOLT_SAPPHIRE, [0.0, 0.1, 1.3]),
            (GEM_BOLT_EMERALD, [0.0, 1.0, 0.1]),
            (GEM_BOLT_RUBY, [1.0, 0.1, 0.1]),
            (GEM_BOLT_DIAMOND, [1.3, 1.3, 1.3]),
            (GEM_BOLT_AMBER, [0.9, 0.6, 0.2]),
        ];
        for (kind, rgb) in cases {
            let p = slot(kind);
            assert_eq!(proj_lights_a(&p, None), vec![(10, 11, rgb)], "kind {kind}");
        }
    }

    #[test]
    fn the_guardian_north_pole_and_guitar_light_fixed_colours() {
        // `:44662` — 623 (0.9, 0.9, 0.7); `:46037` — 342 (0.65, 0.9, 1);
        // `:46588` — 856 HotPink(255,105,180)/255 · 0.3.
        assert_eq!(
            proj_lights_a(&slot(STARDUST_GUARDIAN), None)[0].2,
            [0.9, 0.9, 0.7]
        );
        assert_eq!(
            proj_lights_a(&slot(NORTH_POLE), None)[0].2,
            [0.65, 0.9, 1.0]
        );
        assert_eq!(
            proj_lights_a(&slot(SPARKLE_GUITAR), None)[0].2,
            [255.0 / 255.0 * 0.3, 105.0 / 255.0 * 0.3, 180.0 / 255.0 * 0.3]
        );
    }

    #[test]
    fn the_storm_tigers_light_by_tier_and_gate_on_ai0() {
        // `:49283`/`:49287`/`:49291` — 833 One·0.5, 834 One·0.8, 835
        // Lerp(OurFavoriteColor(255,231,69), White, 0.8) = (255,250,217)/255.
        let mut p = slot(STORM_TIGER_1);
        p.ai[0] = 0.0;
        assert_eq!(proj_lights_a(&p, None)[0].2, [0.5, 0.5, 0.5]);
        p.kind = STORM_TIGER_2;
        assert_eq!(proj_lights_a(&p, None)[0].2, [0.8, 0.8, 0.8]);
        p.kind = STORM_TIGER_3;
        assert_eq!(
            proj_lights_a(&p, None)[0].2,
            [255.0 / 255.0, 250.0 / 255.0, 217.0 / 255.0]
        );
        // The `ai[0] != 4` gate: the attack state stays dark.
        p.ai[0] = 4.0;
        assert!(proj_lights_a(&p, None).is_empty());
    }

    #[test]
    fn every_ported_kind_has_a_proj_data_row() {
        // The absent-wholesale rule's invariant: every kind this feed serves
        // resolves a `proj_data` row (the spawn gate).
        for kind in [
            ORNAMENT_HOSTILE,
            THUNDER_SPEAR_BOLT,
            DEADLY_SPHERE,
            VORTEX_RITUAL_BOLT,
            RAINBOW_CRYSTAL_EXPLOSION,
            TERRARIAN_BEAM,
            NEBULA_ARCANUM,
            MOONLORD_TURRET,
            GEM_BOLT_AMETHYST,
            GEM_BOLT_TOPAZ,
            GEM_BOLT_SAPPHIRE,
            GEM_BOLT_EMERALD,
            GEM_BOLT_RUBY,
            GEM_BOLT_DIAMOND,
            GEM_BOLT_AMBER,
            STARDUST_GUARDIAN,
            NORTH_POLE,
            SPARKLE_GUITAR,
            STORM_TIGER_1,
            STORM_TIGER_2,
            STORM_TIGER_3,
        ] {
            assert!(
                proj_data(kind).is_some(),
                "kind {kind} must have a row"
            );
        }
    }

    #[test]
    fn an_unrowed_kind_contributes_nothing() {
        // The 502 Meowmere (site 4), the 1092 book (site 7), the 451
        // InfluxWaver (site 12), the 464/465/466 cultist orbs (sites 13-16),
        // the 490 ritual (site 18), the 492 lantern (site 19), the 518
        // coin portal (site 20), the 575 nebula sphere (site 21), the 590
        // truffle spore (site 22), the 602 portal gate (site 24), the
        // 612/953/978 trio (site 26), the 658 sandnado mark (site 28), the
        // 1115 glacier fang (site 29), the 731 thunder staff (site 31), the
        // 1105 RC car (site 33), the 1098 digtoise (site 34), the
        // phaseblades (sites 35-36), the 1091/1117 storm lightning (site
        // 37), the 1078 meteor ore (site 38), the 567/568 spore traps (site
        // 39) have no rows; the rowed 248 FlowerPowPetal is aiStyle 1 and
        // never reaches AI_196 (site 40). Sites 41-50 add the 1116
        // petalstorm (41), the 614 cell shot (42), the 636 daybreak (43),
        // the 1020 drone (44), the 973/972/984 sword beams (45-47), the
        // 933/1100 fractals (48-49), the 535 medusa head (50), the 855
        // release lantern (54), the 1127 saber stab (55) and the 756 sharp
        // tears (62) have no rows; the 849/912 whip lights (56-57) are
        // rand-gated absent. A slot carrying any of them feeds no light.
        for kind in [
            502u16, 1092, 451, 464, 465, 466, 490, 492, 518, 575, 590, 602, 612, 953,
            978, 658, 1115, 731, 1105, 1098, 1059, 1074, 1091, 1117, 1078, 567, 568,
            248, 1038, 1116, 614, 636, 1020, 973, 972, 984, 933, 1100, 535, 855, 1127,
            849, 912, 756,
        ] {
            let p = slot(kind);
            assert!(proj_lights_a(&p, None).is_empty());
        }
    }
}
