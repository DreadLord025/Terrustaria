//! The projectile `Lighting.AddLight` sources (`S1-E-5`).
//!
//! C# `Lighting.AddLight` funnels every overload into
//! `Lighting.AddLight(int i, int j, float r, float g, float b)`
//! (`Lighting.cs:157-163`), which appends to the active engine's per-frame
//! list behind `!Main.gamePaused && Main.netMode != 2`
//! (`LightingEngine.cs:51-54`). The engine then max-merges that list onto the
//! working light map (`LightingEngine.cs:208-235`), one frame late. There is
//! ONE list pair per engine — [`crate::entity_lights::EntityLights`] — and
//! this family feeds the same pair as the item family
//! (`entity_lights.rs::item_light`) and the NPC family
//! (`npc_lights.rs::npc_light`); the tile-anchor vocabulary is shared from
//! there too.
//!
//! Ported here: `Projectile.ProjLight` (`Projectile.cs:16153-16661`) — the
//! per-type colour chain that multiplies the slot's `light` amount into one
//! `AddLight` at the integer-halved centre (`:16659`), called from `Update`
//! (`:17031`) once per movement sub-step, after `HandleMovement` and the
//! active re-check (`:17027-17030`). The `light > 0f` early return
//! (`:16155-16158`) is [`proj_light`]'s first gate; the amounts ride
//! `projectile.rs::set_defaults_light` (the `SetDefaults` slice) onto the
//! slot at spawn, and the aiStyle-24 arm rewrites its shard's amount every
//! tick (`:25953`, `ai_crystal_storm`).
//!
//! **Which arms are absent, exactly** (of the 107 `Projectile.cs` call
//! sites, this chain's `:16659` is the one ported — see ANALYSIS "Entity
//! lights (S1-E-5)" for the full drop register):
//!
//! * `:16655` — the Jack 'O Lantern (870)'s `FloodFillTile` + `AddLight`
//!   pair: the kind has no `proj_data` row, and the flood fill has no port.
//! * The RNG arms 867 (`:16217`) and 1000 (`:16224`) draw
//!   `Main.rand.Next(28, 42)` inside the chain — neither kind has a
//!   `proj_data` row, so no slot can carry their light; the arms are not
//!   ported and the stream is untouched.
//! * The disco arms 993 (`:16194-16198`), 1089 (`:16434-16439`) and 251
//!   (`:16630-16641`) are **deferred**: their kinds have no rows, but
//!   `LightParams::disco` exists, so they are cheap follow-ups. The 79
//!   Rainbow Rod bullet (`:16505-16510`) IS a ported row — its colour rides
//!   the same params.
//! * The 211 Shadowbeam arm (`:16601-16614`) **writes** `this.light`
//!   (1.5/1.0 by `localAI[0]`); the kind has no row, and a write inside a
//!   draw-pass feed would be a state mutation — it lands with its AI.
//! * The 259 Golem eye beam's `Main.getGoodWorld` blackout (`:16234-16243`)
//!   is ported: the feed passes the world's For-the-Worthy flag.
//! * The per-tick `light` writers outside `ProjLight`: the aiStyle-24 shard
//!   (`:25953`) is live (`projectile.rs::ai_crystal_storm`); the aiStyle-5
//!   writers (`:23982`/`:24053` — kinds 12/955/92, none with rows) and the
//!   FlameArrow's water transform (`:16822-16826` — `TransformType(1)` +
//!   `light = 0f`, the projectile-water work) are dropped with their
//!   systems. Until the transform lands, a wet FlameArrow keeps its light
//!   where the C# zeroes it (ANALYSIS records the deviation).

use glam::Vec2;
use tr_core::lighting::LightParams;

use crate::entity_lights::{tile_x, tile_y, XAnchor};
use crate::projectile::{
    ANTI_GRAVITY_HOOK, BALL_OF_FIRE, BALL_OF_FROST, BULLET_SNOWMAN, CHRISTMAS_HOOK,
    CHLOROPHYTE_BULLET, CRYSTAL_BULLET, CRYSTAL_SHARD, CRYSTAL_SHARD_MELEE, CRYSTAL_STORM,
    CURSED_BULLET, CURSED_FLAME, CURSED_FLAME_FRIENDLY, DEMON_SCYTHE,
    DEMON_SICKLE,
    DESTROYER_LASER, DUAL_HOOK_BLUE, DUAL_HOOK_RED, ENCHANTED_BEAM, EXPLODING_BULLET, EYE_LASER,
    FLAMELASH,
    FLAME_ARROW, FROST_HYDRA, GOLDEN_BULLET, GOLEM_EYE_BEAM, GREEN_LASER, HIGH_VELOCITY_BULLET,
    ICE_BLOCK, ICE_BOOMERANG, ICHOR_BULLET,
    ILLUMINANT_HOOK, LIGHT_BEAM, MAGIC_MISSILE, MAGNET_SPHERE_BALL, METEOR_SHOT, MINI_RETINA_LASER,
    MUSKET_BALL, NANO_BULLET, NIGHT_BEAM, PARTY_BULLET, PIRATE_ARROW, PURPLE_LASER,
    RAINBOW_ROD_BULLET, SWORD_BEAM,
    TERRA_BEAM, UNHOLY_TRIDENT, UNHOLY_TRIDENT_HOSTILE, VENOM_BULLET, WATER_BOLT,
};

/// `Projectile.ProjLight` (`Projectile.cs:16153-16661`): the `AddLight`
/// source this projectile contributes for one movement sub-step, or `None`.
///
/// `light` is the slot's current amount (`SetDefaults` at spawn, rewritten
/// per tick by the aiStyle-24 arm); `pos`/`width`/`height` are the slot's
/// post-movement box; `good_world` is `Main.getGoodWorld` (the 259 blackout);
/// `p` carries `Main.DiscoR/G/B` for the 79 arm. Pure — no arm draws RNG
/// (the rand-bearing 867/1000 arms have no ported rows and are absent), so
/// the feed's placement cannot move any stream.
///
/// The tail (`:16659`) positions the light at
/// `(int)((position.X + (float)(width / 2)) / 16f)` — the C# halves the
/// **integer** width/height *before* the float cast. [`XAnchor::Center`]
/// uses `width / 2f`; the two forms agree whenever the box sides are even,
/// which every lit `proj_data` row's are (the odd-sided rows carry no
/// light). `proj_center_matches_the_float_anchor_for_every_lit_row` pins
/// that invariant, so the shared anchor is exact here, not approximate.
pub(crate) fn proj_light(
    kind: u16,
    light: f32,
    pos: Vec2,
    width: f32,
    height: f32,
    good_world: bool,
    p: &LightParams,
) -> Option<(i32, i32, [f32; 3])> {
    // `:16155-16158` — `if (!(this.light > 0f)) return;`.
    if !(light > 0.0) {
        return None;
    }
    // `:16159-16161` — num/num2/num3 all start at `this.light`.
    let mut r = light;
    let mut g = light;
    let mut b = light;
    // The `switch` (`:16162-16199`): only the 987-993 glowstick bobbers,
    // none with a `proj_data` row — the 993 disco arm is deferred (module
    // docs). The `if`-chain below is `:16200-16647` in source order; every
    // arm whose kind has no row is absent (the register in ANALYSIS).
    if kind == ANTI_GRAVITY_HOOK {
        // `:16200-16204` — 446: `num *= 0f; num3 *= 0.8f;`.
        r *= 0.0;
        b *= 0.8;
    } else if kind == GOLEM_EYE_BEAM {
        // `:16234-16243` — 259: `num3 *= 0.1f`, then the getGoodWorld
        // blackout zeroes all three.
        b *= 0.1;
        if good_world {
            r = 0.0;
            g = 0.0;
            b = 0.0;
        }
    } else if kind == FLAME_ARROW {
        // `:16249-16253` — 2/82 (the hostile 82 has no row): `num2 *=
        // 0.75f; num3 *= 0.55f;`.
        g *= 0.75;
        b *= 0.55;
    } else if kind == CHRISTMAS_HOOK {
        // `:16229-16233` — 332: `num3 *= 0.1f; num2 *= 0.6f;`.
        b *= 0.1;
        g *= 0.6;
    } else if kind == FROST_HYDRA {
        // `:16259-16263` — 308: `num2 *= 0.7f; num *= 0.1f;`.
        g *= 0.7;
        r *= 0.1;
    } else if kind == MAGNET_SPHERE_BALL {
        // `:16279-16282` — 254: `num *= 0.1f;`.
        r *= 0.1;
    } else if kind == CRYSTAL_STORM {
        // `:16283-16287` — 94: `num *= 0.5f; num2 *= 0f;`.
        r *= 0.5;
        g *= 0.0;
    } else if kind == CURSED_FLAME_FRIENDLY || kind == CURSED_FLAME || kind == CURSED_BULLET {
        // `:16294-16299` — 95/96/103/104 (the 103 water bolt has no row; the
        // 104 CursedBullet joins via S1-P-i18): `num *= 0.35f; num2 *= 1f;
        // num3 *= 0f;`.
        r *= 0.35;
        g *= 1.0;
        b *= 0.0;
    } else if kind == ILLUMINANT_HOOK {
        // `:16315-16320` — 488: absolute `num = 0.3f; num3 = 0.25f;
        // num2 = 0f;` — the amount is ignored past the gate.
        r = 0.3;
        b = 0.25;
        g = 0.0;
    } else if kind == MUSKET_BALL || kind == BULLET_SNOWMAN || kind == PIRATE_ARROW
        || kind == HIGH_VELOCITY_BULLET
    {
        // `:16338-16342` — 14/110/180/242/302 (the 302 sniper bullet has no
        // row; the 242 HighVelocity joins via S1-P-i18): `num2 *= 0.7f;
        // num3 *= 0.1f;`.
        g *= 0.7;
        b *= 0.1;
    } else if kind == BALL_OF_FIRE {
        // `:16348-16353` — 15: `num2 *= 0.4f; num3 *= 0.1f; num = 1f;`.
        g *= 0.4;
        b *= 0.1;
        r = 1.0;
    } else if kind == MAGIC_MISSILE {
        // `:16354-16359` — 16: `num *= 0.1f; num2 *= 0.4f; num3 = 1f;`.
        r *= 0.1;
        g *= 0.4;
        b = 1.0;
    } else if kind == GREEN_LASER {
        // `:16370-16374` — 20: `num *= 0.1f; num3 *= 0.3f;`.
        r *= 0.1;
        b *= 0.3;
    } else if kind == WATER_BOLT {
        // `:16380-16385` — 27: `num *= 0f; num2 *= 0.3f; num3 = 1f;`.
        r *= 0.0;
        g *= 0.3;
        b = 1.0;
    } else if kind == FLAMELASH {
        // `:16386-16390` — 34: `num2 *= 0.1f; num3 *= 0.1f;`.
        g *= 0.1;
        b *= 0.1;
    } else if kind == METEOR_SHOT {
        // `:16391-16396` — 36 (S1-P-i18): `num = 0.8f; num2 *= 0.2f;
        // num3 *= 0.6f;` — the red channel is absolute.
        r = 0.8;
        g *= 0.2;
        b *= 0.6;
    } else if kind == DEMON_SICKLE || kind == DEMON_SCYTHE {
        // `:16402-16407` — 44/45: `num3 = 1f; num *= 0.6f; num2 *= 0.1f;`.
        b = 1.0;
        r *= 0.6;
        g *= 0.1;
    } else if kind == DUAL_HOOK_BLUE {
        // `:16458-16463` — 73: `num *= 0.4f; num2 *= 0.6f; num3 *= 1f;`.
        r *= 0.4;
        g *= 0.6;
        b *= 1.0;
    } else if kind == DUAL_HOOK_RED {
        // `:16464-16469` — 74: `num *= 1f; num2 *= 0.4f; num3 *= 0.6f;`.
        r *= 1.0;
        g *= 0.4;
        b *= 0.6;
    } else if kind == PARTY_BULLET {
        // `:16470-16475` — 284 (S1-P-i18): `num *= 1f; num2 *= 0.1f;
        // num3 *= 0.8f;`.
        r *= 1.0;
        g *= 0.1;
        b *= 0.8;
    } else if kind == NANO_BULLET {
        // `:16476-16481` — 285 (S1-P-i18): `num *= 0.1f; num2 *= 0.5f;
        // num3 *= 1f;`.
        r *= 0.1;
        g *= 0.5;
        b *= 1.0;
    } else if kind == EXPLODING_BULLET {
        // `:16482-16487` — 286 (S1-P-i18): `num *= 1f; num2 *= 0.5f;
        // num3 *= 0.1f;`.
        r *= 1.0;
        g *= 0.5;
        b *= 0.1;
    } else if kind == GOLDEN_BULLET {
        // `:16488-16493` — 287 (S1-P-i18): `num *= 0.9f; num2 *= 1f;
        // num3 *= 0.4f;`.
        r *= 0.9;
        g *= 1.0;
        b *= 0.4;
    } else if kind == VENOM_BULLET {
        // `:16494-16498` — 283 (S1-P-i18): `num *= 0.8f; num2 *= 0.1f;` — the
        // blue channel keeps the amount (no `num3` line in the C# block).
        r *= 0.8;
        g *= 0.1;
    } else if kind == RAINBOW_ROD_BULLET {
        // `:16505-16510` — 79: absolute `Main.DiscoR/G/B / 255f` — the
        // amount is ignored past the gate.
        r = p.disco[0] as f32 / 255.0;
        g = p.disco[1] as f32 / 255.0;
        b = p.disco[2] as f32 / 255.0;
    } else if kind == ICE_BLOCK {
        // `:16511-16516` — 80: `num *= 0f; num2 *= 0.8f; num3 *= 1f;`.
        r *= 0.0;
        g *= 0.8;
        b *= 1.0;
    } else if kind == EYE_LASER || kind == PURPLE_LASER {
        // `:16517-16522` — 83/88: `num *= 0.7f; num2 *= 0f; num3 *= 1f;`.
        r *= 0.7;
        g *= 0.0;
        b *= 1.0;
    } else if kind == DESTROYER_LASER {
        // `:16523-16528` — 100: `num *= 1f; num2 *= 0.5f; num3 *= 0f;`.
        r *= 1.0;
        g *= 0.5;
        b *= 0.0;
    } else if kind == MINI_RETINA_LASER {
        // `:16529-16534` — 84/389 (84 has no row): `num *= 0.8f;
        // num2 *= 0f; num3 *= 0.5f;`.
        r *= 0.8;
        g *= 0.0;
        b *= 0.5;
    } else if kind == CRYSTAL_BULLET || kind == CRYSTAL_SHARD {
        // `:16535-16540` — 89/90 (S1-P-i18): `num2 *= 0.2f; num3 *= 1f;
        // num *= 0.05f;`.
        g *= 0.2;
        b *= 1.0;
        r *= 0.05;
    } else if kind == CRYSTAL_SHARD_MELEE {
        // `:16541-16546` — 1134: `num2 *= 0.2f; num3 *= 0.9f; num *= 0.5f;`.
        g *= 0.2;
        b *= 0.9;
        r *= 0.5;
    } else if kind == ICE_BOOMERANG {
        // `:16553-16558` — 113: `num *= 0.25f; num2 *= 0.75f; num3 *= 1f;`.
        r *= 0.25;
        g *= 0.75;
        b *= 1.0;
    } else if kind == UNHOLY_TRIDENT || kind == UNHOLY_TRIDENT_HOSTILE {
        // `:16559-16564` — 114/115: `num *= 0.5f; num2 *= 0.05f;
        // num3 *= 1f;`.
        r *= 0.5;
        g *= 0.05;
        b *= 1.0;
    } else if kind == SWORD_BEAM {
        // `:16565-16568` — 116: `num3 *= 0.25f;`.
        b *= 0.25;
    } else if kind == TERRA_BEAM || kind == NIGHT_BEAM {
        // `:16574-16578` — 132/157: `num *= 0.2f; num3 *= 0.6f;`.
        r *= 0.2;
        b *= 0.6;
    } else if kind == LIGHT_BEAM {
        // `:16579-16584` — 156: `num *= 1f; num3 *= 0.6f; num2 = 0f;`.
        r *= 1.0;
        b *= 0.6;
        g = 0.0;
    } else if kind == ENCHANTED_BEAM {
        // `:16585-16590` — 173: `num *= 0.3f; num3 *= 1f; num2 = 0.4f;`.
        r *= 0.3;
        b *= 1.0;
        g = 0.4;
    } else if kind == CHLOROPHYTE_BULLET {
        // `:16591-16595` — 207 (S1-P-i18): `num *= 0.4f; num3 *= 0.4f;` — the
        // green channel keeps the amount (no `num2` line in the C# block).
        r *= 0.4;
        b *= 0.4;
    } else if kind == BALL_OF_FROST {
        // `:16596-16600` — 253: `num = 0f; num2 *= 0.4f;`.
        r = 0.0;
        g *= 0.4;
    } else if kind == ICHOR_BULLET {
        // `:16642-16647` — 278/279 (the 278 ichor arrow has no row; the 279
        // IchorBullet joins via S1-P-i18): `num *= 1f; num2 *= 1f; num3 *= 0f;`.
        r *= 1.0;
        g *= 1.0;
        b *= 0.0;
    }
    // Kinds with no chain arm (the 6 Enchanted Boomerang, the plain hooks
    // 13/32 and their riders) keep num/num2/num3 at the amount on all three
    // channels — the C# fall-through leaves them untouched.
    //
    // `:16648-16660` — the 870 flood-fill pair is absent (module docs); the
    // common tail is the integer-halved centre (the doc comment above pins
    // why XAnchor::Center is exact for every lit row).
    Some((
        tile_x(pos, width, XAnchor::Center),
        tile_y(pos, height, XAnchor::Center),
        [r, g, b],
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectile::{proj_data, spawnable_kinds};

    fn at(kind: u16, light: f32) -> Option<(i32, i32, [f32; 3])> {
        proj_light(
            kind,
            light,
            Vec2::new(160.0, 176.0),
            16.0,
            16.0,
            false,
            &LightParams::default(),
        )
    }

    #[test]
    fn the_zero_amount_early_returns() {
        // `:16155-16158` — `!(this.light > 0f)` returns before the chain,
        // even for the absolute-colour kinds (488/79).
        assert!(at(MUSKET_BALL, 0.0).is_none());
        assert!(at(ILLUMINANT_HOOK, 0.0).is_none());
        assert!(at(RAINBOW_ROD_BULLET, 0.0).is_none());
        // Negative and NaN amounts fail the `> 0f` test too.
        assert!(at(MUSKET_BALL, -0.5).is_none());
        assert!(at(MUSKET_BALL, f32::NAN).is_none());
    }

    #[test]
    fn the_bullets_take_the_common_14_110_180_arm() {
        // `:16338-16342` — (l, 0.7l, 0.1l); the musket ball's 0.5
        // (`Projectile.cs:957`) gives (0.5, 0.35, 0.05). The box (160, 176,
        // 16, 16) centres on (168, 184) → tile (10, 11).
        let l = 0.5f32;
        let (x, y, rgb) = at(MUSKET_BALL, l).expect("the musket ball lights");
        assert_eq!((x, y), (10, 11));
        assert_eq!(rgb, [l, l * 0.7, l * 0.1]);
        assert_eq!(at(BULLET_SNOWMAN, l).unwrap().2, [l, l * 0.7, l * 0.1]);
        assert_eq!(at(PIRATE_ARROW, l).unwrap().2, [l, l * 0.7, l * 0.1]);
    }

    #[test]
    fn the_flame_arrow_arm_and_the_armless_boomerang() {
        // `:16249-16253` — 2/82: (l, 0.75l, 0.55l).
        let l = 1.0f32; // `Projectile.cs:824`
        assert_eq!(at(FLAME_ARROW, l).unwrap().2, [l, l * 0.75, l * 0.55]);
        // The 6 Enchanted Boomerang has NO chain arm — the amount rides
        // all three channels (`:16159-16161`'s start values).
        let e = 0.4f32; // `:872`
        assert_eq!(
            at(crate::projectile::ENCHANTED_BOOMERANG, e).unwrap().2,
            [e, e, e]
        );
    }

    #[test]
    fn the_absolute_colour_arms_ignore_the_amount() {
        // 15 (`:16348-16353`): r pinned to 1, g/b scale the amount.
        let l = 0.8f32; // `:970`
        assert_eq!(at(BALL_OF_FIRE, l).unwrap().2, [1.0, l * 0.4, l * 0.1]);
        // 16 (`:16354-16359`): b pinned to 1.
        assert_eq!(at(MAGIC_MISSILE, l).unwrap().2, [l * 0.1, l * 0.4, 1.0]);
        // 488 (`:16315-16320`): fully absolute — a doubled amount changes
        // nothing.
        assert_eq!(at(ILLUMINANT_HOOK, 0.3).unwrap().2, [0.3, 0.0, 0.25]);
        assert_eq!(at(ILLUMINANT_HOOK, 0.6).unwrap().2, [0.3, 0.0, 0.25]);
        // 27 (`:16380-16385`): r zeroed, b pinned to 1.
        assert_eq!(at(WATER_BOLT, 1.0).unwrap().2, [0.0, 0.3, 1.0]);
        // 45 (`:16402-16407`): b pinned to 1.
        let d = 0.2f32; // `:1288`
        assert_eq!(at(DEMON_SCYTHE, d).unwrap().2, [d * 0.6, d * 0.1, 1.0]);
        // 173 (`:16585-16590`): g pinned to 0.4.
        assert_eq!(at(ENCHANTED_BEAM, d).unwrap().2, [d * 0.3, 0.4, d]);
        // 253 (`:16596-16600`): r zeroed absolutely.
        let f = 0.8f32; // `:3364`
        assert_eq!(at(BALL_OF_FROST, f).unwrap().2, [0.0, f * 0.4, f]);
    }

    #[test]
    fn the_rainbow_rod_bullet_rides_the_disco_params() {
        // `:16505-16510` — absolute Main.DiscoR/G/B / 255f.
        let p = LightParams {
            disco: [51, 102, 204],
            ..Default::default()
        };
        let got = proj_light(
            RAINBOW_ROD_BULLET,
            0.8, // `:1600`
            Vec2::new(160.0, 176.0),
            32.0,
            32.0,
            false,
            &p,
        )
        .expect("the rainbow rod bullet lights");
        assert_eq!(got.2, [51.0 / 255.0, 102.0 / 255.0, 204.0 / 255.0]);
        // The 32x32 box centres on (176, 192) → tile (11, 12).
        assert_eq!((got.0, got.1), (11, 12));
    }

    #[test]
    fn the_golem_eye_beam_blacks_out_on_the_good_world() {
        // `:16234-16243` — 259: (0.3, 0.3, 0.03) normally, all-zero under
        // Main.getGoodWorld (the source still appends the zero light).
        let l = 0.3f32; // `:3436`
        assert_eq!(at(GOLEM_EYE_BEAM, l).unwrap().2, [l, l, l * 0.1]);
        let dark = proj_light(
            GOLEM_EYE_BEAM,
            l,
            Vec2::new(160.0, 176.0),
            8.0,
            8.0,
            true,
            &LightParams::default(),
        )
        .expect("the zeroed light is still appended");
        assert_eq!(dark.2, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn the_crystal_storm_arm_halves_red_and_kills_green() {
        // `:16283-16287` — 94: (0.5l, 0, l). The aiStyle-24 arm rewrites
        // the amount to `scale * 0.5f` per tick (`:25953`): at scale 1.2
        // the shard contributes (0.3, 0, 0.6).
        let l = 1.2f32 * 0.5;
        assert_eq!(at(CRYSTAL_STORM, l).unwrap().2, [l * 0.5, 0.0, l]);
        // 1134 (`:16541-16546`): the yoyo shard's (0.5l, 0.2l, 0.9l).
        let s = 0.5f32; // `:10644`
        assert_eq!(
            at(CRYSTAL_SHARD_MELEE, s).unwrap().2,
            [s * 0.5, s * 0.2, s * 0.9]
        );
    }

    #[test]
    fn the_lasers_and_hooks_keep_their_arm_colours() {
        // 83/88 (`:16517-16522`): (0.7l, 0, l).
        let l = 0.75f32; // `:1639`/`:1709`
        assert_eq!(at(EYE_LASER, l).unwrap().2, [l * 0.7, 0.0, l]);
        assert_eq!(at(PURPLE_LASER, l).unwrap().2, [l * 0.7, 0.0, l]);
        // 100 (`:16523-16528`): (l, 0.5l, 0).
        assert_eq!(at(DESTROYER_LASER, l).unwrap().2, [l, l * 0.5, 0.0]);
        // 389 (`:16529-16534`): (0.8l, 0, 0.5l).
        assert_eq!(at(MINI_RETINA_LASER, l).unwrap().2, [l * 0.8, 0.0, l * 0.5]);
        // 73/74 (`:16458-16469`): the blue/red mirror pair at 0.4 (`:1546`).
        let h = 0.4f32;
        assert_eq!(at(DUAL_HOOK_BLUE, h).unwrap().2, [h * 0.4, h * 0.6, h]);
        assert_eq!(at(DUAL_HOOK_RED, h).unwrap().2, [h, h * 0.4, h * 0.6]);
        // 446 (`:16200-16204`): (0, l, 0.8l) at 0.7 (`:5186`).
        let a = 0.7f32;
        assert_eq!(at(ANTI_GRAVITY_HOOK, a).unwrap().2, [0.0, a, a * 0.8]);
        // 332 (`:16229-16233`): (l, 0.6l, 0.1l) at 0.5 (`:4256`).
        let c = 0.5f32;
        assert_eq!(at(CHRISTMAS_HOOK, c).unwrap().2, [c, c * 0.6, c * 0.1]);
    }

    #[test]
    fn the_negative_position_truncates_the_quotient() {
        // `(int)(v / 16f)` truncates toward zero: the 16x16 box at
        // (-20, -20) centres on (-12, -12); -12/16 = -0.75 → 0 on both axes.
        let got = proj_light(
            MUSKET_BALL,
            0.5,
            Vec2::new(-20.0, -20.0),
            16.0,
            16.0,
            false,
            &LightParams::default(),
        )
        .unwrap();
        assert_eq!((got.0, got.1), (0, 0));
    }

    #[test]
    fn proj_center_matches_the_float_anchor_for_every_lit_row() {
        // `:16659` halves the INTEGER width/height before the float cast;
        // XAnchor::Center halves in float. The forms differ only for odd
        // sides — this pins that every lit `proj_data` row has even sides,
        // so the shared anchor is exact (module docs).
        for kind in spawnable_kinds() {
            let Some(d) = proj_data(kind) else { continue };
            if d.light <= 0.0 {
                continue;
            }
            assert_eq!(d.w % 2.0, 0.0, "kind {kind} has an odd width");
            assert_eq!(d.h % 2.0, 0.0, "kind {kind} has an odd height");
            // And the anchors agree at a fractional position, where the
            // truncation boundary would expose a half-pixel drift.
            let pos = Vec2::new(159.5, 175.25);
            let ix = ((pos.x + (d.w as i32 / 2) as f32) / 16.0) as i32;
            let iy = ((pos.y + (d.h as i32 / 2) as f32) / 16.0) as i32;
            assert_eq!(tile_x(pos, d.w, XAnchor::Center), ix, "kind {kind} x");
            assert_eq!(tile_y(pos, d.h, XAnchor::Center), iy, "kind {kind} y");
        }
    }
}
