//! The Projectile.cs `Lighting.AddLight` census freeze (W4-f7, proj batch 3).
//!
//! The full-file census (re-derived independently here from the gitignored
//! `decompiled/` junction, 79 791 lines) finds EXACTLY 107 `Lighting.AddLight`
//! call sites in `Projectile.cs` - every `AddLight` token in the file is one
//! of them (no bare/engine-direct/comment forms). The claim map:
//!
//! * Sites 1-67 (`:16655`...`:54071`) - W4-c19's slice (the `ProjLight`
//!   chain `:16153-16661` plus the AI-arm lights through `AI_001`'s head).
//!   Claimed by branch name; at probe time c19 sat clean at main `a725acb0`
//!   with nothing pushed (the in-flight probe).
//! * Sites 68-107 (`:54173`...`:78556`) - W4-e6's LANDED second wave
//!   (`origin/task/W4-e6-addlight-proj-2` `c8b072b6`, commits `3ece0525` /
//!   `66d75adb` / `fceb64db`): `proj_ai_lights.rs`, 20 ported / 23 lights,
//!   20 absent (18 unrowed kinds + the `:78556` rand-gated electrification
//!   scan). This round re-audited every ported arm against its C# window
//!   (verbatim - colours, anchors, gates, pair order) and every absent
//!   kind against `projectile/data.rs::proj_data` (no row resolves for any
//!   of the 19 unrowed kinds; all 20 ported kinds resolve, 442/819 through
//!   the numeric arms `data.rs:3148`/`:3275`).
//!
//! The unclaimed THIRD-batch remainder is therefore EMPTY (the W4-f9
//! `c2740784` Main.cs precedent). This round adds the missing WITNESS half
//! (the W4-g19 `8005116f` precedent): the 107-site table frozen line-for-line
//! against the decompilation, the negative census (no engine-internal write
//! forms; the light-map delegate forms counted and frozen - they are the
//! documented skip lanes, NOT zero), the slice-boundary pins and the e6
//! arm-structure pins. A future decomp refresh or lossy merge cannot
//! silently move/drop/add a site without this test failing.
//!
//! Reads the gitignored `decompiled/` junction at test time and FAILS when
//! absent (the S1-L-12d localization precedent, g19's
//! `main_addlight_census.rs`). Code-only round: witnesses WRITTEN NOT RUN,
//! no compiler invoked.

use std::path::PathBuf;

fn decompiled_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../decompiled")
}

fn read_source(rel: &str) -> String {
    let path = decompiled_root().join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "{} could not be read ({e}) - the C# source lives in the gitignored \
             `decompiled/` tree; create the junction \
             (`cmd /c mklink /J decompiled ..\\Terraria-Rust\\decompiled`) before \
             running the suite (the S1-L-12d precedent)",
            path.display()
        )
    })
}

/// The 1-based line, trimmed (the C# uses tab indentation).
fn line_at<'a>(lines: &[&'a str], n: usize) -> &'a str {
    assert!(n >= 1 && n <= lines.len(), "line {n} out of range");
    lines[n - 1].trim()
}

#[test]
fn the_projectile_cs_addlight_census_is_exactly_107_sites() {
    let text = read_source("Terraria/Projectile.cs");
    let lines: Vec<&str> = text.lines().collect();
    let sites: Vec<(usize, &str)> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.contains("Lighting.AddLight("))
        .map(|(i, l)| (i + 1, l.trim()))
        .collect();
    // The verbatim 107 - line numbers AND argument text frozen, in line
    // order (the site numbering both slices' registers use).
    assert_eq!(
        sites,
        vec![
            (16655, "Lighting.AddLight(base.Center, rgb);"),
            (16659, "Lighting.AddLight((int)((base.position.X + (float)(base.width / 2)) / 16f), (int)((base.position.Y + (float)(base.height / 2)) / 16f), num, num2, num3);"),
            (17343, "Lighting.AddLight(rgb: ((player.mount.Type == 63 || player.mount.Type == 65) ? new Vector3(0.6f, 0.3f, 0.1f) : new Vector3(0.5f, 0.1f, 0.6f)) * 0.5f, position: boxPosition + new Vector2((float)boxWidth / 2f, (float)boxHeight / 2f));"),
            (24307, "Lighting.AddLight(r: (0.5f + num73) / 2f, g: (0.5f + num74) / 2f, b: (0.5f + num75) / 2f, position: base.Center);"),
            (24877, "Lighting.AddLight(base.Center, 0.25f, 0.2f, 0f);"),
            (24881, "Lighting.AddLight(base.Center, 0.15f, 0.15f, 0.15f);"),
            (25394, "Lighting.AddLight(base.Center, 0.3f, 0.25f, 0.1f);"),
            (27920, "Lighting.AddLight(base.Center, 0.3f, 0.6f, 0.8f);"),
            (29674, "Lighting.AddLight(base.Center, 0.8f, 0.3f, 0.1f);"),
            (29678, "Lighting.AddLight(base.Center, 0.3f, 0.5f, 0.7f);"),
            (30378, "Lighting.AddLight((int)base.Center.X / 16, (int)base.Center.Y / 16, 0.1f, 0.4f, 0.6f);"),
            (31212, "Lighting.AddLight((int)base.Center.X / 16, (int)base.Center.Y / 16, 0.3f * num655, 0.4f * num655, 1f * num655);"),
            (31864, "Lighting.AddLight(base.Center, 0.3f, 0.75f, 0.9f);"),
            (31893, "Lighting.AddLight(vector89, 0.3f, 0.75f, 0.9f);"),
            (31998, "Lighting.AddLight(base.Center, 0.4f, 0.85f, 0.9f);"),
            (32049, "Lighting.AddLight(base.Center, 0.3f, 0.45f, 0.5f);"),
            (32155, "Lighting.AddLight(base.Center, 0.3f, 0.45f, 0.5f);"),
            (32384, "Lighting.AddLight(base.Center, 1.1f, 0.9f, 0.4f);"),
            (32410, "Lighting.AddLight(base.position, 0.35f, 0.35f, 0.1f);"),
            (32962, "Lighting.AddLight(base.Center, 0.8f * 0.3f, 181f / 255f * 0.3f, 72f / 255f * 0.3f);"),
            (33580, "Lighting.AddLight(base.Center, 0.5f, 0.1f, 0.3f);"),
            (34226, "Lighting.AddLight(base.Center, num832 * num835, num833 * num835, num834 * num835);"),
            (34265, "Lighting.AddLight(base.Center, newColor3.ToVector3() * 0.5f);"),
            (34402, "Lighting.AddLight(base.Center + base.velocity * 3f, portalColor.ToVector3() * 0.5f);"),
            (34416, "Lighting.AddLight(base.Center, new Vector3(0.075f, 0.3f, 0.15f));"),
            (34487, "Lighting.AddLight(base.Center, 0.9f, 0.8f, 0.6f);"),
            (34802, "Lighting.AddLight(base.Center, 0.7f, 0.2f, 0.6f);"),
            (36150, "Lighting.AddLight(base.Center, newColor4.ToVector3() * 0.5f);"),
            (37022, "Lighting.AddLight(base.Center, new Vector3(0.2f, 0.5f, 0.8f) * Utils.Remap(this.ai[0], 0f, 60f, 0.5f, 1f));"),
            (37070, "Lighting.AddLight(base.Center, new Vector3(0.3f, 0.9f, 0.7f) * this.Opacity);"),
            (37821, "Lighting.AddLight(base.Center, 0.2f, 0.5f, 0.7f);"),
            (38000, "Lighting.AddLight(base.Center, num23 * num25, num23 * num26, num23 * num27);"),
            (38458, "Lighting.AddLight(base.Center, Vector3.One * 0.3f);"),
            (38707, "Lighting.AddLight(base.Center, 0.3f, 0.3f, 0.1f);"),
            (39020, "Lighting.AddLight((int)(base.Center.X / 16f), (int)(base.Center.Y / 16f), r, g, b);"),
            (39021, "Lighting.AddLight((int)(base.Center.X / 16f), (int)(base.Center.Y / 16f) - 1, r, g, b);"),
            (39783, "Lighting.AddLight(rgb: color.ToVector3() * num9, position: base.Center);"),
            (40038, "Lighting.AddLight(base.Center, 1f, 0f, 0f);"),
            (40046, "Lighting.AddLight(base.Center, 0.2f * num, 0.275f * num, 0.075f * num);"),
            (40507, "Lighting.AddLight(base.Center, vector);"),
            (40513, "Lighting.AddLight(base.Center, vector * Utils.Remap(this.timeLeft, 120f, 0f, 1f, 0f));"),
            (40892, "Lighting.AddLight(base.Center, 0.2f, 0.6f, 0.7f);"),
            (40896, "Lighting.AddLight(base.Center, 0.8f, 0.7f, 0.4f);"),
            (41031, "Lighting.AddLight(base.Center, Vector3.One * 0.3f);"),
            (41775, "Lighting.AddLight(base.Center + this.rotation.ToRotationVector2() * 85f * this.scale, value.ToVector3());"),
            (42181, "Lighting.AddLight(base.Center, value.ToVector3());"),
            (42206, "Lighting.AddLight(base.Center, value3.ToVector3());"),
            (43172, "Lighting.AddLight(base.Center, vector4 * 0.5f * this.Opacity);"),
            (43173, "Lighting.AddLight(mountedCenter, vector5 * this.Opacity * 0.15f);"),
            (44612, "Lighting.AddLight(base.Center, 0.9f, 0.75f, 0.1f);"),
            (44662, "Lighting.AddLight(base.Center, 0.9f, 0.9f, 0.7f);"),
            (46037, "Lighting.AddLight(base.Center, 0.65f, 0.9f, 1f);"),
            (46588, "Lighting.AddLight(base.Center, Color.HotPink.ToVector3() * 0.3f);"),
            (47569, "Lighting.AddLight(rgb: new Vector3(1f, 0.6f, 0.1f) * num2, position: base.Center);"),
            (47649, "Lighting.AddLight(base.Center, color.ToVector3());"),
            (47846, "Lighting.AddLight(r2.Center.ToVector2(), new Vector3(0.2f, 0f, 0.4f));"),
            (48011, "Lighting.AddLight(r16.Center.ToVector2(), new Vector3(0.1f, 0.1f, 0.2f));"),
            (49283, "Lighting.AddLight(base.Center, Vector3.One * 0.5f);"),
            (49287, "Lighting.AddLight(base.Center, Vector3.One * 0.8f);"),
            (49291, "Lighting.AddLight(base.Center, Color.Lerp(Main.OurFavoriteColor, Color.White, 0.8f).ToVector3() * 1f);"),
            (49464, "Lighting.AddLight(base.Center, 0.9f, 0.1f, 0.3f);"),
            (51492, "Lighting.AddLight(base.Center, new Vector3(0.5f, 0.1f, 0.1f) * this.scale);"),
            (52606, "Lighting.AddLight(mountedCenter, 0f, 0.4f, 0.3f);"),
            (52641, "Lighting.AddLight(mountedCenter, vector2);"),
            (52642, "Lighting.AddLight(base.Center, vector2);"),
            (52890, "Lighting.AddLight(base.Center, this.GetCeleb2Color().ToVector3() * 0.5f);"),
            (54071, "Lighting.AddLight(base.Center + base.velocity * 3f, rgb);"),
            (54173, "Lighting.AddLight((int)base.Center.X / 16, (int)base.Center.Y / 16, 0.3f, 1.1f, 0.5f);"),
            (54262, "Lighting.AddLight(base.Center, 1.1f, 0.9f, 0.4f);"),
            (54406, "Lighting.AddLight(base.Center, 0.2f, 0.1f, 0.6f);"),
            (54571, "Lighting.AddLight(base.Center, rgb2);"),
            (54674, "Lighting.AddLight((int)base.Center.X / 16, (int)base.Center.Y / 16, 0.4f, 0.85f, 0.9f);"),
            (54714, "Lighting.AddLight((int)base.Center.X / 16, (int)base.Center.Y / 16, 0.4f, 0.85f, 0.9f);"),
            (54755, "Lighting.AddLight((int)base.Center.X / 16, (int)base.Center.Y / 16, 0.3f, 0.45f, 0.8f);"),
            (54784, "Lighting.AddLight((int)base.Center.X / 16, (int)base.Center.Y / 16, 0.25f, 0.4f, 0.7f);"),
            (54788, "Lighting.AddLight((int)base.Center.X / 16, (int)base.Center.Y / 16, 0.7f, 0.65f, 0.3f);"),
            (54792, "Lighting.AddLight(base.Center, 0.7f, 0.3f, 0.3f);"),
            (54831, "Lighting.AddLight((int)base.Center.X / 16, (int)base.Center.Y / 16, 0.4f, 0.1f, 0.2f);"),
            (54835, "Lighting.AddLight((int)base.Center.X / 16, (int)base.Center.Y / 16, 0.4f, 0.1f, 0.3f);"),
            (54858, "Lighting.AddLight((int)base.Center.X / 16, (int)base.Center.Y / 16, 0.15f, 0.15f, 0.4f);"),
            (54932, "Lighting.AddLight((int)base.Center.X / 16, (int)base.Center.Y / 16, 0.3f, 0.8f, 1.1f);"),
            (54968, "Lighting.AddLight((int)base.Center.X / 16, (int)base.Center.Y / 16, 0.5f, 0.2f, 1.1f);"),
            (54992, "Lighting.AddLight(base.Center, 1.1f, 0.3f, 0.4f);"),
            (55137, "Lighting.AddLight((int)base.Center.X / 16, (int)base.Center.Y / 16, 0.9f, 0.6f, 0.2f);"),
            (55171, "Lighting.AddLight((int)base.Center.X / 16, (int)base.Center.Y / 16, 0.9f, 0.6f, 0.2f);"),
            (56126, "Lighting.AddLight(base.Center, vector36.X, vector36.Y, vector36.Z);"),
            (56294, "Lighting.AddLight(base.Center, 0.1f, 0.3f, 0.4f);"),
            (56298, "Lighting.AddLight(base.Center, 0.4f, 0.2f, 0.4f);"),
            (56421, "Lighting.AddLight(base.Center, 0.3f, 0.05f, 0.05f);"),
            (57185, "Lighting.AddLight(base.Center, 0.6f, 0.4f, 0.3f);"),
            (58213, "Lighting.AddLight(base.Center, 0.5f, 0.3f, 0.1f);"),
            (58226, "Lighting.AddLight(base.Center, 0.5f, 0.3f, 0.1f);"),
            (61228, "Lighting.AddLight((int)base.Center.X / 16, (int)base.Center.Y / 16, 0.9f, 0.6f, 0.2f);"),
            (65120, "Lighting.AddLight(base.Center, 0.2f, 0.6f, 0.7f);"),
            (66272, "Lighting.AddLight((int)base.Center.X / 16, (int)base.Center.Y / 16, vector7.X, vector7.Y, vector7.Z);"),
            (66683, "Lighting.AddLight(vector24, 0.8f, 0.8f, 0.8f);"),
            (66736, "Lighting.AddLight(vector25, 0.8f, 0.8f, 0.8f);"),
            (68124, "Lighting.AddLight(base.Center, new Vector3(0.4f, 0.2f, 0.1f));"),
            (68125, "Lighting.AddLight(base.Bottom + new Vector2(0f, -10f), new Vector3(0.4f, 0.2f, 0.1f));"),
            (68172, "Lighting.AddLight(base.Center, new Vector3(0.4f, 0.2f, 0.1f) * 1.2f);"),
            (68173, "Lighting.AddLight(base.Bottom + new Vector2(0f, -10f), new Vector3(0.4f, 0.2f, 0.1f) * 1.2f);"),
            (68224, "Lighting.AddLight(base.Center, new Vector3(0.4f, 0.2f, 0.1f) * 1.5f);"),
            (68225, "Lighting.AddLight(base.Bottom + new Vector2(0f, -10f), new Vector3(0.4f, 0.2f, 0.1f) * 1.5f);"),
            (69332, "Lighting.AddLight(base.Center, 0.6f, 0.5f, 0.3f);"),
            (69507, "Lighting.AddLight(player.Center, 0.75f, 0.9f, 1.15f);"),
            (69885, "Lighting.AddLight(player.Center, 0.75f, 0.9f, 1.15f);"),
            (78556, "Lighting.AddLight(x, y, 1f, 1f, 1f);"),
        ],
        "the Projectile.cs AddLight census moved - re-run the W4-c19/e6/f7 \
         census and re-pin the family registers (proj_lights.rs cites \
         :16153-16661, proj_ai_lights.rs cites :54173-:78556)"
    );
    // No other AddLight spelling exists in the file: the token count equals
    // the site count (the g19 Main.cs negative-census form).
    assert_eq!(
        text.matches("AddLight").count(),
        107,
        "a 108th AddLight form appeared in Projectile.cs - census it before porting"
    );
}

#[test]
fn projectile_cs_has_no_engine_direct_light_writes() {
    let text = read_source("Terraria/Projectile.cs");
    // No engine internals are touched directly - every light rides the
    // Lighting.AddLight funnel (Lighting.cs:157-163).
    for token in ["_activeEngine", "perFrameLights", "LightingEngine", "DynamicLight"] {
        assert_eq!(
            text.matches(token).count(),
            0,
            "`{token}` appeared in Projectile.cs - the census closed with no \
             engine-direct forms; a new form needs a census row"
        );
    }
    // The light-MAP delegate writes are a DIFFERENT family (the documented
    // skip lanes: the 870 FloodFillTile pair, the 445 >=30f PlotTileLine
    // flood half, the laser/held-item CastLightOpen trails). Their counts
    // are frozen so a decomp refresh cannot silently grow the skip lane.
    for (token, count) in [
        ("FloodFillTile", 2),       // :16654 (the 870 spread) + :70535 (helper)
        ("PlotTileLine", 44),       // the CutTiles/CastLight* trail family
        ("CastLight", 40),          // 31 CastLightOpen + 9 CastLight delegate refs
        ("SpreadLight", 1),         // :16654 SpreadLightOpen_StopForSolids
    ] {
        assert_eq!(
            text.matches(token).count(),
            count,
            "`{token}` count moved in Projectile.cs - re-census the light-map \
             skip lane before touching the entity-light registers"
        );
    }
}

#[test]
fn the_two_claimed_slices_cover_all_107_sites() {
    let text = read_source("Terraria/Projectile.cs");
    let lines: Vec<&str> = text.lines().collect();
    let nums: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.contains("Lighting.AddLight("))
        .map(|(i, _)| i + 1)
        .collect();
    assert_eq!(nums.len(), 107);
    // W4-c19's claimed slice: sites 1-67, :16655 (the 870 ProjLight arm)
    // through :54071 (the last pre-AI_001-tail site).
    assert_eq!(nums[0], 16655);
    assert_eq!(nums[66], 54071);
    // W4-e6's LANDED slice: sites 68-107, :54173 (the 436 arm) through
    // :78556 (the rand-gated electrification scan).
    assert_eq!(nums[67], 54173);
    assert_eq!(nums[106], 78556);
    // The ProjLight boundary: the chain's two sites sit inside the method
    // (:16153-16661); every later site is an AI-arm light.
    assert_eq!(line_at(&lines, 16153), "public void ProjLight()");
    assert_eq!(nums[1], 16659);
    assert!(nums[2] > 16661, "a third ProjLight-internal site appeared");
    // The remainder beyond both claims is EMPTY by construction: 67 + 40
    // = 107 = the whole census. Nothing is left for a fourth batch.
    assert_eq!(67 + 40, nums.len());
}

#[test]
fn the_second_waves_ported_arm_structure_is_pinned() {
    let text = read_source("Terraria/Projectile.cs");
    let lines: Vec<&str> = text.lines().collect();
    // The e6 audit (this round, verbatim against <=100-line windows): the
    // gates/anchors/colours proj_ai_lights.rs replays. The AddLight lines
    // themselves are frozen in the census table above; these are their
    // structural contexts.
    // AI_001's per-type arms.
    assert_eq!(line_at(&lines, 54150), "if (this.type == 436)");
    assert_eq!(line_at(&lines, 54470), "Vector3 rgb2 = new Vector3(0.7f, 0.1f, 0.5f);");
    assert_eq!(line_at(&lines, 54481), "rgb2 = new Vector3(0.4f, 0.6f, 0.9f);");
    // The 459 box rewrite keeps the centre invariant (position = Center,
    // width = height = (int)(22*scale), Center = position).
    assert_eq!(line_at(&lines, 54670), "base.position = base.Center;");
    assert_eq!(line_at(&lines, 54672), "base.width = (base.height = (int)(22f * this.scale));");
    assert_eq!(line_at(&lines, 54673), "base.Center = base.position;");
    // The 442 alpha gate + the localAI[1] >= 120 kill BEFORE the light.
    assert_eq!(line_at(&lines, 54747), "if (this.alpha == 0)");
    assert_eq!(line_at(&lines, 54750), "if (this.localAI[1] >= 120f)");
    // The 440/449/606 shared arm's 449 sub-if.
    assert_eq!(line_at(&lines, 54772), "if (this.type == 440 || this.type == 449 || this.type == 606)");
    assert_eq!(line_at(&lines, 54786), "if (this.type == 449)");
    // The twin orange arms' gates; the rand-bearing dust halves follow.
    assert_eq!(line_at(&lines, 55130), "if (this.type == 350)");
    assert_eq!(line_at(&lines, 55164), "if (this.type == 325)");
    // The 819 else-if chain tail.
    assert_eq!(line_at(&lines, 56388), "else if (this.type == 819)");
    // AI_026's foxsparks pair - unconditional inside each type block.
    assert_eq!(line_at(&lines, 58203), "if (this.type == 1094)");
    assert_eq!(line_at(&lines, 58216), "if (this.type == 1113)");
    // AI_062's StardustCell block.
    assert_eq!(line_at(&lines, 65110), "if (this.type == 613)");
    // AI_075's LaserDrill: the hand anchor, the localAI[0] cycle, the
    // distance gate, the triangle wave, the >=30f flood half (skipped),
    // and the TAIL pin AFTER the light (the ported-but-dark deviation).
    assert_eq!(line_at(&lines, 66135), "Vector2 vector = player.RotatedRelativePoint(mountedCenter);");
    assert_eq!(line_at(&lines, 66251), "this.localAI[0]++;");
    assert_eq!(line_at(&lines, 66252), "if (this.localAI[0] >= 60f)");
    assert_eq!(line_at(&lines, 66256), "if (Vector2.Distance(vector, base.Center) >= 5f)");
    assert_eq!(line_at(&lines, 66258), "float num12 = this.localAI[0] / 60f;");
    assert_eq!(line_at(&lines, 66259), "if (num12 > 0.5f)");
    assert_eq!(line_at(&lines, 66261), "num12 = 1f - num12;");
    assert_eq!(
        line_at(&lines, 66263),
        "Vector3 vector7 = Vector3.Lerp(new Vector3(0f, 1f, 0.7f), new Vector3(0f, 0.7f, 1f), 1f - num12 * 2f) * 0.5f;"
    );
    assert_eq!(line_at(&lines, 66264), "if (Vector2.Distance(vector, base.Center) >= 30f)");
    assert_eq!(
        line_at(&lines, 67329),
        "base.position = player.RotatedRelativePoint(mountedCenter) - base.Size / 2f;"
    );
    // AI_130's tower cases - the Centre light then the Bottom+(0,-10) light.
    assert_eq!(line_at(&lines, 68122), "case 663:");
    assert_eq!(line_at(&lines, 68170), "case 665:");
    assert_eq!(line_at(&lines, 68222), "case 667:");
    // AI_138's shared tail AFTER the sentry-backpack reposition arm.
    assert_eq!(line_at(&lines, 69330), "this.AI_Sentries_HeavySlingReposition();");
    // The electrification scan's rand gate (the absent-wholesale reason).
    assert_eq!(line_at(&lines, 78549), "if (Main.rand.Next(4) == 0)");
    // Entity.cs: Center halves FLOAT (width/2f), Bottom = pos + full height
    // - the feed's `w * 0.5` centre and `pos.y + h` bottom are exact.
    let entity = read_source("Terraria/Entity.cs");
    let elines: Vec<&str> = entity.lines().collect();
    assert_eq!(
        line_at(&elines, 54),
        "return new Vector2(this.position.X + (float)this.width / 2f, this.position.Y + (float)this.height / 2f);"
    );
    assert_eq!(
        line_at(&elines, 126),
        "return new Vector2(this.position.X + (float)this.width / 2f, this.position.Y + (float)this.height);"
    );
}

#[test]
fn the_lighting_cs_funnel_and_gate_are_pinned() {
    let text = read_source("Terraria/Lighting.cs");
    let lines: Vec<&str> = text.lines().collect();
    // The Vector3 overload (the rgb2/colour-vector arms): QUOTIENT truncation.
    assert_eq!(
        line_at(&lines, 135),
        "public static void AddLight(Vector2 position, Vector3 rgb)"
    );
    assert_eq!(
        line_at(&lines, 137),
        "Lighting.AddLight((int)(position.X / 16f), (int)(position.Y / 16f), rgb.X, rgb.Y, rgb.Z);"
    );
    // The Vector2 float overload: the same QUOTIENT truncation.
    assert_eq!(
        line_at(&lines, 140),
        "public static void AddLight(Vector2 position, float r, float g, float b)"
    );
    assert_eq!(
        line_at(&lines, 142),
        "Lighting.AddLight((int)(position.X / 16f), (int)(position.Y / 16f), r, g, b);"
    );
    // Every overload funnels into the int form behind the pause/server gate.
    assert_eq!(
        line_at(&lines, 157),
        "public static void AddLight(int i, int j, float r, float g, float b)"
    );
    assert_eq!(line_at(&lines, 159), "if (!Main.gamePaused && Main.netMode != 2)");
    assert_eq!(
        line_at(&lines, 161),
        "Lighting._activeEngine.AddLight(i, j, new Vector3(r, g, b));"
    );
}