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
//! RNG rule): the 409 Typhoon (`:30378`, inside `Next(3) == 0`), the 590
//! TruffleSpore (`:34226`, the `Next(28,42)` colour flicker), the 849
//! ScytheWhip (`:47846`) and the 912 CoolWhip (`:48011`).
//!
//! **Round 2 (the 42-absent re-census):** the d-lane AI ports pushed since the
//! first wave rowed ~20 of the absent kinds (W4-d12/d13/d17/d19/d20/d21); their
//! light arms are activated here, each citing its source lane. The collector
//! matches `p.kind` directly, so an arm is live as soon as its lane's row lets
//! a slot carry the kind — the row itself stays the lane's to merge (the
//! keep-one-dedupe convention; this branch copies no rows). Two first-wave
//! misattributions are corrected: site 45 (`:41775`) is the 985 arm (not 973),
//! site 46 (`:42181`) the 1126 arm (not 972). The tiger/601 state-driven
//! deviations are unchanged (no lane ported their AI). See ANALYSIS.
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
//!   projectile centre). PORTED by [`chillet_mount_light`]: the gate's
//!   `hostile` / `noEnchantments` / `noEnchantmentVisuals` fields land as
//!   kind-derived `SetDefaults` census helpers (`data.rs` — the 179/4/21-kind
//!   lists; the only runtime `hostile` writes are the two classless
//!   bomb-death flips the `(melee || ranged)` gate already excludes). The
//!   aiStyle-190/191 swords emit the two rotated swing rects (`:17299-17305`).
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
//!   Absent wholesale (the RNG rule): the colour's `num831` rides a
//!   `Main.rand.Next(28,42)` draw the pure feed cannot take. (The 590 row
//!   landed with W4-d12, but the rand gate keeps the light absent.)
//! * `:34265` — site 23, the 644 RainbowCrystalExplosion's
//!   `AddLight(Center, hslToRgb(ai[0],1,0.5).ToVector3()·0.5)` (aiStyle-112,
//!   `alpha == 0`). PORTED.
//! * `:34402` — site 24, the 602 PortalGunGate's
//!   `AddLight(Center + velocity·3, portalColor·0.5)` (aiStyle-114,
//!   `alpha == 0`). PORTED — rowed by W4-d12; the gate reads
//!   `GetPortalColor(owner, ai[1])` (ai[1], the portal index, not ai[0]).
//! * `:34416` — site 25, the 604 TerrarianBeam's
//!   `AddLight(Center, 0.075, 0.3, 0.15)` (aiStyle-115 head, unconditional).
//!   PORTED (604 is the style's only rowed kind).
//! * `:34487` — site 26, the aiStyle-117 `flag58` trio 612/953/978's
//!   `AddLight(Center, 0.9, 0.8, 0.6)`. PORTED for the 612/953 (rowed by
//!   W4-d12); the 978 Volcano keeps no row (aiStyle 189, d19's decade), so
//!   its term stays unreachable.
//! * `:34802` — site 27, the 617 NebulaArcanum's
//!   `AddLight(Center, 0.7, 0.2, 0.6)` (aiStyle-118 tail, `alpha < 150`).
//!   PORTED.
//! * `:36150` — site 28, the 658 SandnadoHostileMark's
//!   `AddLight(Center, white·0.5)` (aiStyle-128, `alpha == 0`). PORTED —
//!   rowed by W4-d13.
//! * `:37022` — site 29, the 1115 GlacierFangShot's
//!   `AddLight(Center, (0.2,0.5,0.8)·Remap(ai[0],0,60,0.5,1))`
//!   (AI_208 tail). PORTED — rowed by W4-d21.
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
//!   `AddLight(Center, One·0.3)` (AI_205 tail). PORTED — rowed by W4-d21.
//! * `:38707` — site 34, the 1098 PalworldDigtoise's (and the 1123 Trusty
//!   twin's) `AddLight(Center, 0.3, 0.3, 0.1)` (AI_204, the `ai[0] in {1,2,3}`
//!   mining states). PORTED — rowed by W4-d21.
//! * `:39020`/`:39021` — sites 35/36, the phaseblade/phaser pair
//!   `AddLight((int)(Center/16), r,g,b)` + the tile above (AI_003's
//!   `IsAPhaseblade` block, `Item.GetPhaseColor·0.5`). Absent: no
//!   1059-1072/1074-1076/1101/1102 row.
//! * `:39783` — site 37, the storm-lightning's
//!   `AddLight(Center, GetLightningColor·num9)` (AI_203, types 1091/1117).
//!   PORTED — rowed by W4-d21; the 1091's `moreLightning && noLightning`
//!   seed-green folds to the teal default (the feed carries no micro-seed
//!   flags; ANALYSIS records the fold).
//! * `:40038` — site 38, the 1078 MeteorOre's `AddLight(Center, 1,0,0)`
//!   (AI_199). PORTED — rowed by W4-d20.
//! * `:40046` — site 39, the 567/568 SporeTrap's
//!   `AddLight(Center, 0.2·n, 0.275·n, 0.075·n)` (AI_105 head). Absent:
//!   neither rowed.
//! * `:40507` — site 40, the AI_196_Petal early-return
//!   `AddLight(Center, vector)` (`ai[0] < 20 && ai[0] >= 10`). The reaching
//!   kinds are 1038/1116 (aiStyle 196); the rowed 248 FlowerPowPetal is
//!   aiStyle 1 and never reaches AI_196. PORTED — 1038/1116 rowed by W4-d20.
//! * `:40513` — site 41, the 1116 PetalstormShot's
//!   `AddLight(Center, vector·Remap(timeLeft,120,0,1,0))` (AI_196, the
//!   `ai[0] >= 20` half). PORTED — rowed by W4-d20.
//! * `:40892` — site 42, the 614 StardustCellMinionShot's
//!   `AddLight(Center, 0.2, 0.6, 0.7)` (AI_113 tail). PORTED — rowed by
//!   W4-d12 F1.
//! * `:40896` — site 43, the 636 Daybreak's
//!   `AddLight(Center, 0.8, 0.7, 0.4)` (AI_113 tail). PORTED — rowed by
//!   W4-d12 F1.
//! * `:41031` — site 44, the 1020 JimsDrone's `AddLight(Center, One·0.3)`
//!   (AI_195 tail). PORTED — rowed by W4-d20.
//! * `:41775` — site 45, the 985 TerraBlade2Shot's lead-anchor
//!   `AddLight(Center + rot·85·scale, (64,220,96))` (AI_191's `type == 985`
//!   arm, AFTER the `type != 985` return `:41726`). PORTED — rowed by
//!   W4-d20. (The wave-1 register misattributed this to the 973, which
//!   returns before the light and has none.)
//! * `:42181` — site 46, the 1126 ChlorophyteClaymoreBlade's
//!   `AddLight(Center, (0,96,0) [lerp toward (34,128,0) when localAI[2]==1])`
//!   (AI_190's `type == 1126` arm). PORTED — rowed by W4-d19. (The wave-1
//!   register misattributed this to the 972, which has no light arm.)
//! * `:42206` — site 47, the 984 TerraBlade2's
//!   `AddLight(Center, (15,84,125))` (AI_190's 984 arm). PORTED — rowed by
//!   W4-d19.
//! * `:43172`/`:43173` — sites 48/49, the 933/1100 FinalFractal's
//!   trail + mounted-centre pair (AI_182, the profile `trailColor`, the
//!   Opacity window). PORTED — rowed by W4-d19.
//! * `:44612` — site 50, the 535 MedusaHead's
//!   `AddLight(Center, 0.9, 0.75, 0.1)` (AI_100). Absent: 535 no row.
//! * `:44662` — site 51, the 623 StardustGuardian's
//!   `AddLight(Center, 0.9, 0.9, 0.7)` (AI_120's 623 arm). PORTED.
//! * `:46037` — site 52, the 342 NorthPole's
//!   `AddLight(Center, 0.65, 0.9, 1)` (AI_019_Spears' 342 arm). PORTED.
//! * `:46588` — site 53, the 856 SparkleGuitar's
//!   `AddLight(Center, HotPink.ToVector3()·0.3)` (AI_167's 856 arm). PORTED.
//! * `:47569` — site 54, the 855 ReleaseLantern's
//!   `AddLight(Center, (1,0.6,0.1)·(1-alpha/255))` (AI_166). PORTED — rowed
//!   by W4-d17.
//! * `:47649` — site 55, the 1127 ChlorophyteSaberStab's
//!   `AddLight(Center, (32,108,0))` (AI_161). PORTED — rowed by W4-d17.
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
    proj_data, set_defaults_hostile, set_defaults_no_enchantment_visuals,
    set_defaults_no_enchantments, Projectile, ANTI_GRAVITY_HOOK, CANNON_EXPLOSIVE, CRIMSON_HEART,
    DEADLY_SPHERE, GEM_BOLT_AMBER, GEM_BOLT_AMETHYST, GEM_BOLT_DIAMOND, GEM_BOLT_EMERALD,
    GEM_BOLT_RUBY, GEM_BOLT_SAPPHIRE, GEM_BOLT_TOPAZ, LUNAR_HOOK_NEBULA, LUNAR_HOOK_SOLAR,
    LUNAR_HOOK_STARDUST, LUNAR_HOOK_VORTEX, MOONLORD_TURRET, NEBULA_ARCANUM, NORTH_POLE,
    ORNAMENT_HOSTILE, RAINBOW_CRYSTAL_EXPLOSION, SPARKLE_GUITAR, STARDUST_GUARDIAN,
    STORM_TIGER_1, STORM_TIGER_2, STORM_TIGER_3, TERRARIAN_BEAM, THUNDER_SPEAR_BOLT,
    VORTEX_RITUAL_BOLT,
};

/// The centre tile of a slot's post-step box, quotient-truncated (the
/// `AddLight(Vector2, …)` overload, `Lighting.cs:140-143`).
fn centre_tile(p: &Projectile) -> (i32, i32) {
    (
        tile_x(p.pos, p.w, XAnchor::Center),
        tile_y(p.pos, p.h, XAnchor::Center),
    )
}

/// `Utils.Remap(fromValue, fromMin, fromMax, toMin, toMax)` (`Utils.cs:345-348`)
/// in the clamped default — `Lerp(toMin, toMax, GetLerpValue(fromMin, fromMax,
/// fromValue, clamped: true))`. The glacier fang's charge scale (`:37022`)
/// and the petalstorm's life fade (`:40513`) read it.
fn remap(from_value: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
    to_min + (to_max - to_min) * crate::projectile::get_lerp_value(from_min, from_max, from_value)
}

/// Site 3 (`:17343`) — `EmitEnchantmentVisualsAt`'s Chillet-mount light, the
/// one first-wave site OUTSIDE the AI arms (`Update` calls
/// `UpdateEnchantmentVisuals` at `:16702`, before the AI). Collected FIRST,
/// ahead of [`proj_lights_a`]'s AI-arm lights, in the C# call order.
///
/// The gate chain, faithful to the slot state:
/// * the outer `Update` gate `!noEnchantmentVisuals` (`:16700`) and the
///   `npcProj` early-return (`:17292`) — `npcProj` has no port field (the
///   hostile shots ride `friendly = false`, the `projectile.rs:22545`
///   precedent), so it folds false;
/// * the arm gate (`:17341`): `mount.Active && mount.Type in 62..=65 &&
///   (melee || ranged) && friendly && !hostile && !noEnchantments`.
///
/// `hostile` / `noEnchantments` / `noEnchantmentVisuals` are `SetDefaults`
/// facts derived from the kind (the `data.rs` census helpers): the only
/// runtime `hostile` writes are the two classless bomb-death flips
/// (`:28247` / `:75798`), which the `(melee || ranged)` gate already excludes,
/// so the kind-derived flags are exact for this consumer. `melee` / `ranged` /
/// `friendly` ride the slot's live flags.
///
/// `owner_mount` is the owner's `Mount.Type` when `Mount.Active` (the feed
/// resolves `owner == 0` to the local player, the e6 single-player fold);
/// `None` = no active mount / no owner, so the arm skips. Pure — no RNG.
pub(crate) fn chillet_mount_light(
    p: &Projectile,
    owner_mount: Option<i32>,
) -> Vec<(i32, i32, [f32; 3])> {
    let mut out = Vec::new();
    let Some(mount_type) = owner_mount else {
        return out;
    };
    // `:17341` — the Chillet mounts 62-65, the class + friendly gates, and
    // the three kind-derived enchantment flags.
    if !matches!(mount_type, 62 | 63 | 64 | 65)
        || !(p.melee || p.ranged)
        || !p.friendly
        || set_defaults_hostile(p.kind)
        || set_defaults_no_enchantments(p.kind)
        || set_defaults_no_enchantment_visuals(p.kind)
    {
        return out;
    }
    // `:17343` — `(type == 63 || type == 65) ? (0.6,0.3,0.1) : (0.5,0.1,0.6)`,
    // then `* 0.5f`.
    let rgb = if mount_type == 63 || mount_type == 65 {
        [0.6 * 0.5, 0.3 * 0.5, 0.1 * 0.5]
    } else {
        [0.5 * 0.5, 0.1 * 0.5, 0.6 * 0.5]
    };
    let (cx, cy) = centre_tile(p);
    let style = proj_data(p.kind).map_or(0, |d| d.ai_style);
    if style == 190 || style == 191 {
        // `:17299-17305` — the big-sword swings emit at TWO rotated rects
        // (`num = -π/4` then `+π/4`); each rect's centre is
        // `Center + (rotation + num).ToRotationVector2() * 70 * scale`, and
        // `EmitEnchantmentVisualsAt(r.TopLeft, r.Width, r.Height)` anchors at
        // `r.TopLeft + (r.Width/2, r.Height/2)` = that centre. The slot's
        // `rot`/`scale` are the spawn state until the sword AI arms land (the
        // state-driven deviation, ANALYSIS).
        let centre = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
        for num in [-std::f32::consts::FRAC_PI_4, std::f32::consts::FRAC_PI_4] {
            let (s, c) = (p.rot + num).sin_cos();
            let anchor = centre + Vec2::new(c, s) * 70.0 * p.scale;
            out.push(((anchor.x / 16.0) as i32, (anchor.y / 16.0) as i32, rgb));
        }
    } else {
        // `:17316` — the general box: `boxPosition + (boxWidth/2, boxHeight/2)`
        // = `position + (w/2, h/2)` = the centre. (The whip branch `:17307`
        // never reaches here — `DefaultToWhip` sets no melee/ranged flag.)
        out.push((cx, cy, rgb));
    }
    out
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
    } else if p.kind == 602 {
        // `:34394-34402` — aiStyle-114's gate (602 PortalGunGate): the GATE
        // reads `GetPortalColor(owner, (int)ai[1])` (ai[1], not ai[0]), the
        // −25 fade, then the light at `alpha == 0` anchored
        // `Center + velocity*3`. Rowed by W4-d12 (`:6511-6520`); its
        // `ai_portal_gate` runs the fade. The colour is the same single-player
        // portal hue the 601 bolt reads.
        if p.alpha == 0 {
            let centre = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
            let lead = centre + p.vel * 3.0;
            let c = portal_color(p.ai[1] as i32);
            out.push((
                (lead.x / 16.0) as i32,
                (lead.y / 16.0) as i32,
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
    } else if p.kind == 612 || p.kind == 953 {
        // `:34461` + `:34485-34487` — aiStyle-117's `flag58` trio
        // (612/953/978): (0.9, 0.8, 0.6) at the centre, unconditional inside
        // the flag. The 612 SolarWhipSwordExplosion / 953 DaybreakExplosion
        // are rowed by W4-d12; the 978 Volcano keeps no row (its aiStyle-189
        // arm is d19's decade, still unported), so the third term stays
        // unreachable. The d12 explosion arm runs the −63 fade; the light
        // itself is ungated.
        out.push((cx, cy, [0.9, 0.8, 0.6]));
    } else if p.kind == NEBULA_ARCANUM {
        // `:34800-34802` — aiStyle-118's tail: `alpha < 150` gates the
        // centre light (the sim's `ai_nebula_arcanum` runs the −15 fade).
        if p.alpha < 150 {
            out.push((cx, cy, [0.7, 0.2, 0.6]));
        }
    } else if p.kind == 658 {
        // `:36088` + `:36148-36150` — aiStyle-128's SandnadoHostileMark:
        // `newColor4 = new Color(255,255,255)` at the head, the light at
        // `alpha == 0` = `white.ToVector3() * 0.5f`. Rowed by W4-d13
        // (`:7071-7082`); its `ai_sandnado_mark` runs the −8/+16 fade.
        if p.alpha == 0 {
            out.push((cx, cy, [0.5, 0.5, 0.5]));
        }
    } else if p.kind == MOONLORD_TURRET {
        // `:37070` — AI_123's 641 arm: `(0.3, 0.9, 0.7) * this.Opacity`.
        // `Opacity` is the `1 - alpha/255` view (`Projectile.cs:540-545`);
        // the sim's `ai_floating_turrets` runs the −5 fade and the scale
        // pin before this read.
        let opacity = 1.0 - p.alpha as f32 / 255.0;
        out.push((cx, cy, [0.3 * opacity, 0.9 * opacity, 0.7 * opacity]));
    } else if p.kind == 1115 {
        // `:37022` — AI_208_GlacierFangShot's tail:
        // `(0.2,0.5,0.8) * Utils.Remap(ai[0], 0, 60, 0.5, 1)`, unconditional.
        // Rowed by W4-d21 (`:10401-10411`); its `ai_glacier_fang_shot` runs
        // the channel charge that advances ai[0].
        let n = remap(p.ai[0], 0.0, 60.0, 0.5, 1.0);
        out.push((cx, cy, [0.2 * n, 0.5 * n, 0.8 * n]));
    } else if let Some(rgb) = gem_bolt_light(p.kind) {
        // `:37954-38000` — AI_029_GemStaffs_ActuallyGemStaffs' tail:
        // `num23 = 1f`, the per-gem switch on `num24 = (type - 121 + 86)
        // - 85` (the 597 amber remaps `num = 262` first, `:37832-37834`),
        // then `AddLight(Center, num23·num25, num23·num26, num23·num27)`.
        // The 1120 ForbiddenMinionShot's default (1,1,1) has no row.
        out.push((cx, cy, rgb));
    } else if p.kind == 1105 {
        // `:38458` — AI_205_RemoteControlCar's tail: `Vector3.One * 0.3f`
        // at the centre, unconditional (after the speed clamp). Rowed by
        // W4-d21 (`:10294-10301`); its `ai_remote_control_car` runs the
        // input-bit drive. The light is RNG-free.
        out.push((cx, cy, [0.3, 0.3, 0.3]));
    } else if p.kind == 1098 || p.kind == 1123 {
        // `:38585-38589` + `:38707` — AI_204_Digtoise: the light lives in
        // the `flag8 = ai[0] in {1,2,3}` block (the mining states); the
        // appear (0) / despawn (4) / petting (5) states return before it.
        // (0.3, 0.3, 0.1) at the centre. Rowed by W4-d21 (`:10241-10250` /
        // `:10504-10513`); its `ai_digtoise` drives the ai[0] state machine.
        if matches!(p.ai[0] as i32, 1 | 2 | 3) {
            out.push((cx, cy, [0.3, 0.3, 0.1]));
        }
    } else if p.kind == 1091 || p.kind == 1117 {
        // `:39780-39783` — AI_203_StormLightning's tail:
        // `GetLightningColor(type).ToVector3() * num9`, where
        // `num9 = sqrt(1 - (Remap(ai[1]/num, 0, 0.5, 0, 1) - 1)^2) * 0.3`
        // and `num = 240` (1091 trap) / `30` (1117 weapon). Rowed by W4-d21
        // (`:10175-10185` / `:10427-10436`); its `ai_storm_lightning` runs
        // the ai[1] age counter. The 1091's `moreLightning && noLightning`
        // green (`:39871-39874`) reads two micro-seed flags the feed does not
        // carry — folded to the teal default (the colour is faithful for
        // every classic world; ANALYSIS records the seed fold).
        let num = if p.kind == 1117 { 30.0 } else { 240.0 };
        let num7 = p.ai[1] / num;
        let num8 = remap(num7, 0.0, 0.5, 0.0, 1.0);
        let num9 = (1.0 - (num8 - 1.0) * (num8 - 1.0)).sqrt() * 0.3;
        let c = lightning_color(p.kind);
        out.push((
            cx,
            cy,
            [
                c[0] as f32 / 255.0 * num9,
                c[1] as f32 / 255.0 * num9,
                c[2] as f32 / 255.0 * num9,
            ],
        ));
    } else if p.kind == 1078 {
        // `:40038` — AI_199_MeteorOre's light: (1, 0, 0) at the centre,
        // unconditional (the surrounding dust draws rand, the light does
        // not). Rowed by W4-d20 (`:10067-10077`); its `ai_meteor_ore` runs
        // the spin. The hostile+friendly trap keeps both flags.
        out.push((cx, cy, [1.0, 0.0, 0.0]));
    } else if p.kind == 1038 || p.kind == 1116 {
        // `:40466-40513` — AI_196_Petal. `vector` = (0.5, 0, 0.2) for the
        // 1038 FlowerWhipPetal, (0, 0.6, 0.6) for the 1116 PetalstormShot.
        // Site 40 (`:40507`): the early-return branch `ai[0] < 20` lights
        // once `ai[0] >= 10` (the pre-increment `flag`), full `vector`.
        // Site 41 (`:40513`): the 1116's `ai[0] >= 20` half scales by
        // `Remap(timeLeft, 120, 0, 1, 0)`. Rowed by W4-d20 (`:9793-9806` /
        // `:10413-10425`); its `ai_petal` runs the age counter. The 248
        // FlowerPowPetal is aiStyle 1 and never reaches this style.
        let vector = if p.kind == 1116 {
            [0.0, 0.6, 0.6]
        } else {
            [0.5, 0.0, 0.2]
        };
        if p.ai[0] < 20.0 {
            if p.ai[0] >= 10.0 {
                out.push((cx, cy, vector));
            }
        } else if p.kind == 1116 {
            let n = remap(p.time_left as f32, 120.0, 0.0, 1.0, 0.0);
            out.push((cx, cy, [vector[0] * n, vector[1] * n, vector[2] * n]));
        }
    } else if p.kind == 614 {
        // `:40890-40892` — AI_113_TargetSticker's tail: (0.2, 0.6, 0.7) at
        // the centre, unconditional (after the state machine). Rowed by
        // W4-d12 F1 (`:6611-6619`).
        out.push((cx, cy, [0.2, 0.6, 0.7]));
    } else if p.kind == 636 {
        // `:40894-40896` — AI_113's 636 Daybreak arm: (0.8, 0.7, 0.4) at
        // the centre, unconditional. Rowed by W4-d12 F1 (`:6838-6849`).
        out.push((cx, cy, [0.8, 0.7, 0.4]));
    } else if p.kind == 1020 {
        // `:41031` — AI_195_JimsDrone's tail: `Vector3.One * 0.3f` at the
        // centre, unconditional (after the speed clamp). Rowed by W4-d20
        // (`:9659-9665`); its `ai_jims_drone` runs the input-bit flight.
        out.push((cx, cy, [0.3, 0.3, 0.3]));
    } else if p.kind == 985 {
        // `:41773-41775` — AI_191_TrueNightsEdge's 985 TerraBlade2Shot arm
        // (AFTER the `type != 985` return at `:41726`): the lead anchor
        // `Center + rotation.ToRotationVector2() * 85 * scale`, colour
        // `new Color(64, 220, 96)`. Rowed by W4-d20 (`:9405-9421`); its
        // `ai_true_nights_edge` runs the rot/scale choreography. The 973
        // TrueNightsEdge returns before this light (it has none).
        let centre = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
        let (s, c) = p.rot.sin_cos();
        let lead = centre + Vec2::new(c, s) * 85.0 * p.scale;
        out.push((
            (lead.x / 16.0) as i32,
            (lead.y / 16.0) as i32,
            [64.0 / 255.0, 220.0 / 255.0, 96.0 / 255.0],
        ));
    } else if p.kind == 1126 {
        // `:42176-42181` — AI_190_NightsEdge's 1126 ChlorophyteClaymoreBlade
        // arm: `new Color(0, 96, 0)`, lerped toward (34, 128, 0) by
        // `num = localAI[0] / ai[1]` when `localAI[2] == 1` (the ground-bloom
        // latch the first-tick block sets at `vel.y == 0`). Rowed by W4-d19
        // (`:10543-10558`); its `ai_nights_edge` runs the latch + age. The 972
        // NightsEdge has no light arm (the wave-1 register misattributed it).
        let mut rgb = [0.0, 96.0 / 255.0, 0.0];
        if p.local_ai[2] == 1.0 {
            let num = if p.ai[1] != 0.0 { p.local_ai[0] / p.ai[1] } else { 0.0 };
            // XNA's Color.Lerp truncates each byte (`(byte)MathHelper.Lerp`,
            // the 835 tiger precedent): R = (byte)(34·num), G = (byte)(96+32·num).
            rgb = [
                (34.0 * num) as u8 as f32 / 255.0,
                (96.0 + (128.0 - 96.0) * num) as u8 as f32 / 255.0,
                0.0,
            ];
        }
        out.push((cx, cy, rgb));
    } else if p.kind == 984 {
        // `:42204-42206` — AI_190's 984 TerraBlade2 arm: `new Color(15, 84,
        // 125)` at the centre, unconditional. Rowed by W4-d19 (`:9388-9404`).
        out.push((cx, cy, [15.0 / 255.0, 84.0 / 255.0, 125.0 / 255.0]));
    } else if p.kind == 933 || p.kind == 1100 {
        // `:43156-43173` — AI_182_FinalFractal's first-second trail: the
        // profile colour `trailColor` (the `(int)ai[1]` sword-id lookup),
        // `vector4 * 0.5 * Opacity` at the centre + `Lerp(One, vector4, 0.7)
        // * Opacity * 0.15` at the owner's MountedCenter. Rowed by W4-d19
        // (`:8857-8873` / `:10265-10281`); its `ai_final_fractal` runs the
        // orbit + the opacity window. The light reads the slot's Opacity
        // (`1 - alpha/255`); the mounted half needs the owner anchor.
        if p.local_ai[0] < 60.0 {
            let opacity = 1.0 - p.alpha as f32 / 255.0;
            let tc = final_fractal_trail_color(p.ai[1] as i32);
            let v4 = [
                tc[0] as f32 / 255.0,
                tc[1] as f32 / 255.0,
                tc[2] as f32 / 255.0,
            ];
            out.push((cx, cy, [v4[0] * 0.5 * opacity, v4[1] * 0.5 * opacity, v4[2] * 0.5 * opacity]));
            if let Some(mc) = mounted_center {
                let v5 = [
                    1.0 + (v4[0] - 1.0) * 0.7,
                    1.0 + (v4[1] - 1.0) * 0.7,
                    1.0 + (v4[2] - 1.0) * 0.7,
                ];
                out.push((
                    (mc.x / 16.0) as i32,
                    (mc.y / 16.0) as i32,
                    [v5[0] * opacity * 0.15, v5[1] * opacity * 0.15, v5[2] * opacity * 0.15],
                ));
            }
        }
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
    } else if p.kind == 855 {
        // `:47566-47569` — AI_166_Dove's 855 ReleaseLantern arm:
        // `(1, 0.6, 0.1) * (1 - alpha/255)` at the centre, unconditional
        // inside the lantern flag. Rowed by W4-d17 (`:8197-8205`); its
        // `ai_dove` runs the +3 fade after timeLeft < 90.
        let num2 = 1.0 - p.alpha as f32 / 255.0;
        out.push((cx, cy, [num2, 0.6 * num2, 0.1 * num2]));
    } else if p.kind == 1127 {
        // `:47646-47649` — AI_161_RapierStabs' 1127 ChlorophyteSaberStab
        // arm: `new Color(32, 108, 0, 0)` at the centre (the A=0 does not
        // touch ToVector3's RGB). Rowed by W4-d17 (`:10559-10564`,
        // DefaultToShortsword); its `ai_rapier_stabs` runs the thrust.
        out.push((cx, cy, [32.0 / 255.0, 108.0 / 255.0, 0.0]));
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
    } else if p.kind == CRIMSON_HEART {
        // `:49462-49464` — AI_067's `type == 500` arm: (0.9, 0.1, 0.3) at
        // the centre, unconditional.
        out.push((cx, cy, [0.9, 0.1, 0.3]));
    } else if p.kind == ANTI_GRAVITY_HOOK {
        // `:52604-52606` — AI_007's 446 arm: `AddLight(mountedCenter, 0,
        // 0.4, 0.3)`. The `PlotTileLine` rope spread (`:52613`) is the
        // light-map write with no port — skipped (the 870 FloodFillTile
        // precedent). The anchor is the owner's MountedCenter, not the
        // hook's centre.
        if let Some(mc) = mounted_center {
            out.push(((mc.x / 16.0) as i32, (mc.y / 16.0) as i32, [0.0, 0.4, 0.3]));
        }
    } else if p.kind == LUNAR_HOOK_SOLAR
        || p.kind == LUNAR_HOOK_VORTEX
        || p.kind == LUNAR_HOOK_NEBULA
        || p.kind == LUNAR_HOOK_STARDUST
    {
        // `:52623-52644` — AI_007's 646-649 arm: the per-kind colour, then
        // `AddLight(mountedCenter, vector2)` + `AddLight(Center, vector2)`
        // (the pair, in that order). The `PlotTileLine` rope spread
        // (`:52644`) skipped.
        let rgb = match p.kind {
            LUNAR_HOOK_SOLAR => [0.7, 0.5, 0.1],   // `:52629`
            LUNAR_HOOK_VORTEX => [0.0, 0.6, 0.7],  // `:52632`
            LUNAR_HOOK_NEBULA => [0.6, 0.2, 0.6],  // `:52635`
            _ => [0.6, 0.6, 0.9],                  // `:52638` (649)
        };
        if let Some(mc) = mounted_center {
            out.push(((mc.x / 16.0) as i32, (mc.y / 16.0) as i32, rgb));
        }
        out.push((cx, cy, rgb));
    } else if (715..=718).contains(&p.kind) {
        // `:52890` — AI_147_Celeb2Rocket's head:
        // `AddLight(Center, GetCeleb2Color().ToVector3() * 0.5f)`. The
        // palette is the `(int)ai[0]` switch (`:52550-52563`); the
        // Transparent default contributes a zeroed light (still appended).
        let c = celeb2_color(p.ai[0] as i32);
        out.push((
            cx,
            cy,
            [
                c[0] as f32 / 255.0 * 0.5,
                c[1] as f32 / 255.0 * 0.5,
                c[2] as f32 / 255.0 * 0.5,
            ],
        ));
    } else if p.kind == CANNON_EXPLOSIVE {
        // `:54066-54071` — AI_001's 601 arm:
        // `AddLight(Center + velocity*3, GetPortalColor(owner, ai[0]) * 0.5)`.
        // The portal teleport AI is unported, so `ai[0]` keeps its spawn
        // value (the e6 ported-but-dark precedent); the single-player
        // `netMode == 0` fold (`PortalHelper.cs:381-384`) picks the orange
        // hue at portal 0 and the blue at portal != 0. The lead anchor is
        // `Center + velocity*3`, quotient-truncated.
        let centre = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
        let lead = centre + p.vel * 3.0;
        let c = portal_color(p.ai[0] as i32);
        out.push((
            (lead.x / 16.0) as i32,
            (lead.y / 16.0) as i32,
            [
                c[0] as f32 / 255.0 * 0.5,
                c[1] as f32 / 255.0 * 0.5,
                c[2] as f32 / 255.0 * 0.5,
            ],
        ));
    }
    out
}

/// `Projectile.GetCeleb2Color` (`Projectile.cs:52550-52563`): the
/// `(int)ai[0]` palette — XNA's named colours as bytes. The default is
/// `Color.Transparent` (0,0,0,0).
fn celeb2_color(state: i32) -> [u8; 3] {
    match state {
        0 => [255, 0, 0],       // Color.Red
        1 => [255, 140, 0],     // Color.DarkOrange
        2 => [255, 215, 0],     // Color.Gold
        3 => [50, 205, 50],     // Color.LimeGreen
        4 => [65, 105, 225],    // Color.RoyalBlue
        5 => [80, 20, 180],     // `new Color(80, 20, 180)`
        6 => [255, 105, 180],   // Color.HotPink
        _ => [0, 0, 0],         // Color.Transparent
    }
}

/// `PortalHelper.GetPortalColor(player, portal)` (`PortalHelper.cs:378-392`)
/// in the single-player `netMode == 0` fold: portal 0 is
/// `hslToRgb(0.12, 1, 0.5)` (orange), any other portal is
/// `hslToRgb(0.52, 1, 0.6)` (blue). The `A = 66` write does not touch RGB.
fn portal_color(portal: i32) -> [u8; 3] {
    let c = if portal != 0 {
        crate::ui::color::hsl_to_rgb(0.52, 1.0, 0.6, 255)
    } else {
        crate::ui::color::hsl_to_rgb(0.12, 1.0, 0.5, 255)
    };
    [c[0], c[1], c[2]]
}

/// `Projectile.GetLightningColor(type)` (`Projectile.cs:39862-39878`): the
/// 1117 LightningStrikeShot is (60, 120, 255), the 1122 ArcSurge (255, 60, 60),
/// the 1091 StormLightning trap (80, 220, 80) ONLY under
/// `moreLightningSeed && noLightningSeed` else the (80, 220, 220) teal default.
/// The feed carries no micro-seed flags, so the 1091's seed-green folds to the
/// teal default (faithful for every classic world; ANALYSIS records the fold).
fn lightning_color(kind: u16) -> [u8; 3] {
    match kind {
        1117 => [60, 120, 255],
        1122 => [255, 60, 60],
        _ => [80, 220, 220],
    }
}

/// `FinalFractalHelper.GetFinalFractalProfile(usedSwordId).trailColor`
/// (`Graphics/FinalFractalHelper.cs:104-111` + the static table `:128-215`):
/// the per-sword trail colour the Zenith illusion's light reads (`:43170`).
/// The 1123 key is `Main.OurFavoriteColor` (255, 231, 69, `Main.cs:66748`);
/// unlisted ids return the default profile's `Color.White` (`:215`).
fn final_fractal_trail_color(sword_id: i32) -> [u8; 3] {
    match sword_id {
        65 => [236, 62, 192],
        1123 => [255, 231, 69], // Main.OurFavoriteColor
        46 => [122, 66, 191],
        121 => [254, 158, 35],
        190 => [107, 203, 0],
        368 | 674 => [236, 200, 19],
        273 | 675 => [179, 54, 201],
        2880 => [84, 234, 245],
        989 => [91, 158, 232],
        1826 => [252, 95, 4],
        3063 => [254, 194, 250],
        3065 => [237, 63, 133],
        757 => [80, 222, 122],
        155 => [56, 78, 210],
        795 => [237, 28, 36],
        3018 => [143, 215, 29],
        4144 | 4956 => [178, 255, 180],
        3507 => [235, 166, 135],
        _ => [255, 255, 255], // _defaultProfile, Color.White
    }
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
    fn the_crimson_heart_lights_a_fixed_red() {
        // `:49464` — (0.9, 0.1, 0.3), unconditional.
        assert_eq!(
            proj_lights_a(&slot(CRIMSON_HEART), None)[0].2,
            [0.9, 0.1, 0.3]
        );
    }

    #[test]
    fn the_antigravity_hook_lights_the_mounted_centre_only() {
        // `:52606` — (0, 0.4, 0.3) at mountedCenter, NOT the hook centre.
        let p = slot(ANTI_GRAVITY_HOOK);
        // No owner anchor → no light (the feed's None fold).
        assert!(proj_lights_a(&p, None).is_empty());
        // mountedCenter (320, 480) → tile (20, 30); the hook's own centre
        // (168, 184) → (10, 11) must NOT appear.
        let got = proj_lights_a(&p, Some(Vec2::new(320.0, 480.0)));
        assert_eq!(got, vec![(20, 30, [0.0, 0.4, 0.3])]);
    }

    #[test]
    fn the_lunar_hooks_light_the_mounted_centre_then_the_centre() {
        // `:52641`/`:52642` — the pair, mounted first then centre, same
        // per-kind colour. 646 (0.7,0.5,0.1), 647 (0,0.6,0.7),
        // 648 (0.6,0.2,0.6), 649 (0.6,0.6,0.9).
        let cases = [
            (LUNAR_HOOK_SOLAR, [0.7, 0.5, 0.1]),
            (LUNAR_HOOK_VORTEX, [0.0, 0.6, 0.7]),
            (LUNAR_HOOK_NEBULA, [0.6, 0.2, 0.6]),
            (LUNAR_HOOK_STARDUST, [0.6, 0.6, 0.9]),
        ];
        for (kind, rgb) in cases {
            let p = slot(kind);
            let got = proj_lights_a(&p, Some(Vec2::new(320.0, 480.0)));
            assert_eq!(got, vec![(20, 30, rgb), (10, 11, rgb)], "kind {kind}");
            // Without the owner anchor only the centre half lands.
            assert_eq!(proj_lights_a(&p, None), vec![(10, 11, rgb)]);
        }
    }

    #[test]
    fn the_celeb2_rockets_take_the_ai0_palette() {
        // `:52890` — GetCeleb2Color()/255 · 0.5. State 0 Red → (0.5, 0, 0);
        // state 6 HotPink; the default Transparent → a zeroed light.
        let mut p = slot(715);
        p.ai[0] = 0.0;
        assert_eq!(proj_lights_a(&p, None)[0].2, [0.5, 0.0, 0.0]);
        p.ai[0] = 6.0;
        assert_eq!(
            proj_lights_a(&p, None)[0].2,
            [255.0 / 255.0 * 0.5, 105.0 / 255.0 * 0.5, 180.0 / 255.0 * 0.5]
        );
        p.ai[0] = 9.0;
        assert_eq!(proj_lights_a(&p, None)[0].2, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn the_portal_bolt_lights_the_lead_anchor_in_the_portal_hue() {
        // `:54071` — Center + velocity*3, GetPortalColor(owner, ai[0])·0.5.
        // The 16x16 box at (160,176) centres (168,184); velocity (2,0) →
        // lead (174,184) → tile (10,11). ai[0]=0 → orange hsl(0.12,1,0.5)
        // = (255,184,0); ai[0]=1 → blue hsl(0.52,1,0.6) = (51,231,255).
        let mut p = slot(CANNON_EXPLOSIVE);
        p.vel = Vec2::new(2.0, 0.0);
        p.ai[0] = 0.0;
        let got = proj_lights_a(&p, None);
        assert_eq!(got[0].0, 10);
        assert_eq!(got[0].1, 11);
        assert_eq!(
            got[0].2,
            [255.0 / 255.0 * 0.5, 184.0 / 255.0 * 0.5, 0.0]
        );
        p.ai[0] = 1.0;
        assert_eq!(
            proj_lights_a(&p, None)[0].2,
            [51.0 / 255.0 * 0.5, 231.0 / 255.0 * 0.5, 255.0 / 255.0 * 0.5]
        );
    }

    #[test]
    fn the_chillet_mount_light_gates_and_pairs() {
        // `:17341-17343` — mount 62/64 → (0.5,0.1,0.6)·0.5; 63/65 →
        // (0.6,0.3,0.1)·0.5, at the centre for a general-box melee shot.
        let mut p = slot(TERRARIAN_BEAM); // 604, aiStyle 115, melee, friendly
        p.melee = true;
        assert!(chillet_mount_light(&p, None).is_empty(), "no mount → dark");
        assert!(chillet_mount_light(&p, Some(61)).is_empty(), "not a Chillet");
        assert_eq!(
            chillet_mount_light(&p, Some(62)),
            vec![(10, 11, [0.25, 0.05, 0.3])]
        );
        assert_eq!(
            chillet_mount_light(&p, Some(63))[0].2,
            [0.3, 0.15, 0.05]
        );
        assert_eq!(
            chillet_mount_light(&p, Some(65))[0].2,
            [0.3, 0.15, 0.05]
        );
        // The friendly gate: a hostile-lane slot stays dark.
        p.friendly = false;
        assert!(chillet_mount_light(&p, Some(62)).is_empty());
        p.friendly = true;
        // The class gate: no melee/ranged → dark (the whip precedent).
        p.melee = false;
        assert!(chillet_mount_light(&p, Some(62)).is_empty());
    }

    #[test]
    fn the_chillet_mount_light_respects_the_enchantment_census() {
        // The kind-derived flags: a hostile SetDefaults kind (346 ornament,
        // `:4366`) stays dark even with melee forced; a noEnchantmentVisuals
        // kind (732 thunder spear, `:7171`) too.
        let mut p = slot(ORNAMENT_HOSTILE);
        p.melee = true;
        assert!(chillet_mount_light(&p, Some(62)).is_empty(), "hostile census");
        let mut p = slot(THUNDER_SPEAR_BOLT);
        p.melee = true;
        assert!(
            chillet_mount_light(&p, Some(62)).is_empty(),
            "noEnchantmentVisuals census"
        );
        // The census helpers themselves: the four noEnchantments kinds, the
        // 21 noEnchantmentVisuals kinds, and a sample of the hostile list.
        for k in [480u16, 1121, 1131, 1132] {
            assert!(set_defaults_no_enchantments(k), "kind {k}");
        }
        assert!(!set_defaults_no_enchantments(604));
        for k in [181u16, 732, 933, 1100, 1128, 1129] {
            assert!(set_defaults_no_enchantment_visuals(k), "kind {k}");
        }
        assert!(!set_defaults_no_enchantment_visuals(604));
        for k in [12u16, 346, 658, 1078, 1091, 1092] {
            assert!(set_defaults_hostile(k), "kind {k}");
        }
        assert!(!set_defaults_hostile(604));
        assert!(!set_defaults_hostile(342));
    }

    #[test]
    fn the_chillet_sword_swings_emit_two_rotated_rects() {
        // `:17299-17305` — aiStyle 190/191 emit at num = −π/4 then +π/4;
        // each anchor = Center + (rot + num).ToRotationVector2()·70·scale.
        // The 984 TerraBlade2 (aiStyle 190) at (160,176) 16x16, rot 0,
        // scale 1: centre (168,184); the −π/4 anchor (217.497, 134.503) →
        // tile (13,8); the +π/4 anchor (217.497, 233.497) → tile (13,14).
        let mut p = slot(984);
        p.melee = true;
        p.rot = 0.0;
        p.scale = 1.0;
        let got = chillet_mount_light(&p, Some(62));
        assert_eq!(got.len(), 2);
        assert_eq!((got[0].0, got[0].1), (13, 8));
        assert_eq!((got[1].0, got[1].1), (13, 14));
        assert_eq!(got[0].2, [0.25, 0.05, 0.3]);
        assert_eq!(got[1].2, got[0].2);
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
            CRIMSON_HEART,
            ANTI_GRAVITY_HOOK,
            LUNAR_HOOK_SOLAR,
            LUNAR_HOOK_VORTEX,
            LUNAR_HOOK_NEBULA,
            LUNAR_HOOK_STARDUST,
            715,
            716,
            717,
            718,
            CANNON_EXPLOSIVE,
        ] {
            assert!(
                proj_data(kind).is_some(),
                "kind {kind} must have a row"
            );
        }
    }

    #[test]
    fn an_unrowed_kind_contributes_nothing() {
        // Round 2: the d-lanes rowed ~20 of the first wave's absent kinds and
        // their arms are now live (the activation witnesses below). What stays
        // absent: the still-unrowed kinds — the 502 Meowmere (site 4), the
        // 1092 book (7), the 451 InfluxWaver (12), the 464/465/466 cultist orbs
        // (13-16), the 490 ritual (18), the 492 lantern (19), the 518 coin
        // portal (20), the 575 nebula sphere (21), the 978 Volcano (the 26
        // trio's third term), the 731 thunder staff (31), the phaseblades
        // (35-36), the 567/568 spore traps (39), the 535 medusa head (50), the
        // 756 sharp tears (62); the rand-gated 590 truffle spore (22), the
        // 409 Typhoon (11), the 849/912 whips (56-57); and the 248
        // FlowerPowPetal (aiStyle 1, never reaches AI_196) plus the 973/972
        // sword beams (rowed by d19/d20 but with NO light arm — the wave-1
        // register misattributed sites 45/46 to them). A slot carrying any of
        // them feeds no light.
        for kind in [
            502u16, 1092, 451, 464, 465, 466, 490, 492, 518, 575, 590, 978, 731,
            1059, 1074, 567, 568, 248, 973, 972, 535, 849, 912, 756, 409,
        ] {
            let p = slot(kind);
            assert!(proj_lights_a(&p, None).is_empty(), "kind {kind}");
        }
    }

    #[test]
    fn the_d12_activated_arms_light_their_colours() {
        // Site 24 (`:34402`) — the 602 gate: alpha==0, the ai[1] portal hue,
        // the Center+vel*3 lead. Site 26 (`:34487`) — the 612/953 flag58 pair.
        // Sites 42/43 (`:40892`/`:40896`) — the 614/636 AI_113 tail.
        let mut p = slot(602);
        p.alpha = 0;
        p.ai[1] = 0.0; // portal 0 → orange hsl(0.12,1,0.5) = (255,184,0)
        p.vel = Vec2::new(2.0, 0.0);
        let got = proj_lights_a(&p, None);
        assert_eq!(got[0].0, 10); // lead (174,184) → tile 10
        assert_eq!(got[0].1, 11);
        assert_eq!(got[0].2, [255.0 / 255.0 * 0.5, 184.0 / 255.0 * 0.5, 0.0]);
        p.alpha = 25; // the fade gate → dark
        assert!(proj_lights_a(&p, None).is_empty());
        for k in [612u16, 953] {
            assert_eq!(proj_lights_a(&slot(k), None), vec![(10, 11, [0.9, 0.8, 0.6])]);
        }
        assert_eq!(proj_lights_a(&slot(614), None), vec![(10, 11, [0.2, 0.6, 0.7])]);
        assert_eq!(proj_lights_a(&slot(636), None), vec![(10, 11, [0.8, 0.7, 0.4])]);
    }

    #[test]
    fn the_d13_d17_activated_arms_light_their_colours() {
        // Site 28 (`:36150`) — the 658 mark: alpha==0 → white·0.5.
        let mut p = slot(658);
        p.alpha = 0;
        assert_eq!(proj_lights_a(&p, None), vec![(10, 11, [0.5, 0.5, 0.5])]);
        p.alpha = 8;
        assert!(proj_lights_a(&p, None).is_empty());
        // Site 54 (`:47569`) — the 855 lantern: (1,0.6,0.1)·(1-alpha/255).
        let mut p = slot(855);
        p.alpha = 0;
        assert_eq!(proj_lights_a(&p, None)[0].2, [1.0, 0.6, 0.1]);
        p.alpha = 255;
        assert_eq!(proj_lights_a(&p, None)[0].2, [0.0, 0.0, 0.0]);
        // Site 55 (`:47649`) — the 1127 saber: (32,108,0)/255.
        assert_eq!(
            proj_lights_a(&slot(1127), None)[0].2,
            [32.0 / 255.0, 108.0 / 255.0, 0.0]
        );
    }

    #[test]
    fn the_d19_sword_arms_light_their_colours() {
        // Site 47 (`:42206`) — the 984 TerraBlade2: (15,84,125)/255.
        assert_eq!(
            proj_lights_a(&slot(984), None)[0].2,
            [15.0 / 255.0, 84.0 / 255.0, 125.0 / 255.0]
        );
        // Site 46 (`:42181`) — the 1126 Claymore: (0,96,0)/255 base; the
        // localAI[2]==1 ground-bloom lerps toward (34,128,0) by localAI[0]/ai[1].
        let mut p = slot(1126);
        p.local_ai[2] = 0.0;
        assert_eq!(proj_lights_a(&p, None)[0].2, [0.0, 96.0 / 255.0, 0.0]);
        p.local_ai[2] = 1.0;
        p.local_ai[0] = 5.0;
        p.ai[1] = 10.0; // num = 0.5 → R = (byte)17 = 17, G = (byte)112 = 112
        assert_eq!(
            proj_lights_a(&p, None)[0].2,
            [17.0 / 255.0, 112.0 / 255.0, 0.0]
        );
    }

    #[test]
    fn the_d19_fractal_pair_lights_the_profile_colour() {
        // Sites 48/49 (`:43172`/`:43173`) — the 933/1100 fractal: the profile
        // trailColor·0.5·Opacity at the centre + Lerp(One,trail,0.7)·Opacity·0.15
        // at the mounted centre. ai[1]=65 → (236,62,192); alpha 0 → Opacity 1.
        let mut p = slot(933);
        p.local_ai[0] = 10.0; // < 60 → the first-second window
        p.alpha = 0;
        p.ai[1] = 65.0;
        let got = proj_lights_a(&p, Some(Vec2::new(320.0, 480.0)));
        assert_eq!(got.len(), 2);
        assert_eq!(got[0].0, 10);
        assert_eq!(got[0].1, 11);
        assert_eq!(
            got[0].2,
            [236.0 / 255.0 * 0.5, 62.0 / 255.0 * 0.5, 192.0 / 255.0 * 0.5]
        );
        assert_eq!(got[1].0, 20); // mounted (320,480) → tile (20,30)
        assert_eq!(got[1].1, 30);
        // The mounted half without an owner anchor: only the centre lands.
        assert_eq!(proj_lights_a(&p, None).len(), 1);
        // Past the first second (localAI[0] >= 60) → dark.
        p.local_ai[0] = 60.0;
        assert!(proj_lights_a(&p, None).is_empty());
        // The unlisted sword id → the White default profile.
        p.local_ai[0] = 10.0;
        p.ai[1] = 99999.0;
        assert_eq!(proj_lights_a(&p, None)[0].2, [0.5, 0.5, 0.5]);
    }

    #[test]
    fn the_d20_activated_arms_light_their_colours() {
        // Site 38 (`:40038`) — the 1078 meteor ore: (1,0,0).
        assert_eq!(proj_lights_a(&slot(1078), None), vec![(10, 11, [1.0, 0.0, 0.0])]);
        // Site 44 (`:41031`) — the 1020 drone: One·0.3.
        assert_eq!(proj_lights_a(&slot(1020), None), vec![(10, 11, [0.3, 0.3, 0.3])]);
        // Site 45 (`:41775`) — the 985 TerraBlade2Shot: the rot·85·scale lead,
        // (64,220,96)/255. rot 0, scale 1 → lead (168+85,184) = (253,184) → (15,11).
        let mut p = slot(985);
        p.rot = 0.0;
        p.scale = 1.0;
        let got = proj_lights_a(&p, None);
        assert_eq!(got[0].0, 15);
        assert_eq!(got[0].1, 11);
        assert_eq!(got[0].2, [64.0 / 255.0, 220.0 / 255.0, 96.0 / 255.0]);
    }

    #[test]
    fn the_d20_petal_arms_gate_on_the_age_counter() {
        // Site 40 (`:40507`) — the 1038 petal: ai[0] in [10,20) → (0.5,0,0.2).
        let mut p = slot(1038);
        p.ai[0] = 9.0;
        assert!(proj_lights_a(&p, None).is_empty());
        p.ai[0] = 10.0;
        assert_eq!(proj_lights_a(&p, None)[0].2, [0.5, 0.0, 0.2]);
        p.ai[0] = 20.0; // the 1038 has no >=20 light
        assert!(proj_lights_a(&p, None).is_empty());
        // Site 41 (`:40513`) — the 1116 petalstorm: ai[0]>=20 scales
        // (0,0.6,0.6) by Remap(timeLeft,120,0,1,0). timeLeft 60 → 0.5.
        let mut p = slot(1116);
        p.ai[0] = 25.0;
        p.time_left = 60;
        assert_eq!(proj_lights_a(&p, None)[0].2, [0.0, 0.3, 0.3]);
        p.ai[0] = 15.0; // the early-return half uses the 1116 colour
        assert_eq!(proj_lights_a(&p, None)[0].2, [0.0, 0.6, 0.6]);
    }

    #[test]
    fn the_d21_activated_arms_light_their_colours() {
        // Site 33 (`:38458`) — the 1105 RC car: One·0.3.
        assert_eq!(proj_lights_a(&slot(1105), None), vec![(10, 11, [0.3, 0.3, 0.3])]);
        // Site 34 (`:38707`) — the 1098/1123 digtoise: ai[0] in {1,2,3} only.
        for k in [1098u16, 1123] {
            let mut p = slot(k);
            p.ai[0] = 0.0; // the appear state → dark
            assert!(proj_lights_a(&p, None).is_empty());
            p.ai[0] = 2.0;
            assert_eq!(proj_lights_a(&p, None)[0].2, [0.3, 0.3, 0.1]);
            p.ai[0] = 4.0; // the despawn state → dark
            assert!(proj_lights_a(&p, None).is_empty());
        }
        // Site 29 (`:37022`) — the 1115 glacier fang: (0.2,0.5,0.8)·Remap(ai0,0,60,0.5,1).
        let mut p = slot(1115);
        p.ai[0] = 0.0; // Remap → 0.5
        assert_eq!(proj_lights_a(&p, None)[0].2, [0.1, 0.25, 0.4]);
        p.ai[0] = 60.0; // Remap → 1
        assert_eq!(proj_lights_a(&p, None)[0].2, [0.2, 0.5, 0.8]);
    }

    #[test]
    fn the_d21_storm_lightning_scales_by_the_age_envelope() {
        // Site 37 (`:39783`) — the 1091/1117: GetLightningColor·num9, where
        // num9 = sqrt(1-(Remap(ai1/num,0,0.5,0,1)-1)^2)·0.3. At ai1=0 → num9=0
        // (a zeroed light is still appended); at ai1=num/4 → num8=0.5 →
        // num9 = sqrt(0.75)·0.3.
        let mut p = slot(1117); // num = 30, colour (60,120,255)
        p.ai[1] = 0.0;
        assert_eq!(proj_lights_a(&p, None)[0].2, [0.0, 0.0, 0.0]);
        p.ai[1] = 7.5; // num7 = 0.25 → num8 = 0.5
        let n9 = (0.75f32).sqrt() * 0.3;
        assert_eq!(
            proj_lights_a(&p, None)[0].2,
            [60.0 / 255.0 * n9, 120.0 / 255.0 * n9, 255.0 / 255.0 * n9]
        );
        // The 1091 trap: num = 240, the teal default (80,220,220).
        let mut p = slot(1091);
        p.ai[1] = 60.0; // num7 = 0.25 → num8 = 0.5
        assert_eq!(
            proj_lights_a(&p, None)[0].2,
            [80.0 / 255.0 * n9, 220.0 / 255.0 * n9, 220.0 / 255.0 * n9]
        );
    }

    #[test]
    fn the_d_lane_activated_kinds_resolve_rows_when_their_lanes_land() {
        // The collector matches `p.kind` directly, so the arms above are live
        // regardless; this pins the spawn-gate invariant conditionally (the
        // W4-d12 `if let Some(d) = proj_data(978)` precedent): once a d-lane's
        // row merges, the kind MUST resolve. On this branch the rows are the
        // lanes' to merge, so the check is a no-op until then.
        for kind in [
            602u16, 612, 953, 614, 636, 658, 855, 1127, 984, 1126, 933, 1100,
            1078, 1038, 1116, 1020, 985, 1115, 1105, 1098, 1123, 1091, 1117,
        ] {
            if let Some(d) = proj_data(kind) {
                assert!(d.ai_style > 0, "kind {kind} rowed with a style");
            }
        }
    }
}
