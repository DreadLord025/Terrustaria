//! The Main.cs `Lighting.AddLight` census freeze (W4-g19, Main batch 2 of 2).
//!
//! The full-file census (W4-f9 `c2740784`, re-derived independently here)
//! finds EXACTLY FIVE `Lighting.AddLight` call sites in `Main.cs` — all in
//! `DrawProj`'s grapple-chain draws:
//! * `:29016/29020/29024/29028` — the type-332 Christmas-hook chain's four
//!   rotating colour rows (`num112` 0..3, wrap at `:29039-29043`), anchored
//!   `(int)vector19.X / 16` — truncate the coordinate FIRST (`:29012-29013`).
//! * `:29186` — the type-488 Illuminant-hook chain's per-stepped-link light,
//!   via the `AddLight(Vector2, …)` overload's QUOTIENT anchor
//!   `(int)(position.X / 16f)` (`Lighting.cs:140-143`).
//!
//! All five are ported by W4-e8's `tile_lights.rs`
//! (`christmas_hook_chain_lights` / `illuminant_hook_chain_lights`); the
//! Main.cs tile/ambient remainder is EMPTY (no update-side/ambient sites —
//! tile ambience is scanner-seeded, `tr-core/tile_light`; no
//! `PlotTileLine`/`CastLight*` in Main.cs — those live in
//! `DelegateMethods.cs`/`Utils.cs`, also W4-e8 scope).
//!
//! This test freezes that census against the decompilation so a future
//! decomp refresh or a lossy merge cannot silently move/drop a site or
//! introduce a sixth one without the family registers noticing. It reads
//! the gitignored `decompiled/` junction at test time and FAILS when the
//! junction is absent (the S1-L-12d localization precedent,
//! `tr-core/localization/mod.rs::shipped_english_resources_match_the_game`).
//!
//! Code-only round: witnesses WRITTEN NOT RUN, no compiler invoked.

use std::path::PathBuf;

fn decompiled_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../decompiled")
}

fn read_source(rel: &str) -> String {
    let path = decompiled_root().join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "{} could not be read ({e}) — the C# source lives in the gitignored \
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
fn the_main_cs_addlight_census_is_exactly_five_sites() {
    let text = read_source("Terraria/Main.cs");
    let lines: Vec<&str> = text.lines().collect();
    let sites: Vec<(usize, &str)> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.contains("Lighting.AddLight("))
        .map(|(i, l)| (i + 1, l.trim()))
        .collect();
    // The verbatim five — line numbers AND argument text frozen.
    assert_eq!(
        sites,
        vec![
            (29016, "Lighting.AddLight(i2, j2, 0f, 0.2f, 0.2f);"),
            (29020, "Lighting.AddLight(i2, j2, 0.1f, 0.2f, 0f);"),
            (29024, "Lighting.AddLight(i2, j2, 0.2f, 0.1f, 0f);"),
            (29028, "Lighting.AddLight(i2, j2, 0.2f, 0f, 0.2f);"),
            (29186, "Lighting.AddLight(center2, 0.2f, 0f, 0.175f);"),
        ],
        "the Main.cs AddLight census moved — re-run the W4-f9/g19 census and \
         re-pin the family registers (e8 tile_lights.rs cites these lines)"
    );
    // No other AddLight spelling exists in the file (bare `AddLight(`,
    // engine-direct writes, comments): the token count equals the site count.
    assert_eq!(
        text.matches("AddLight").count(),
        5,
        "a sixth AddLight form appeared in Main.cs — census it before porting"
    );
}

#[test]
fn main_cs_has_no_other_light_write_forms() {
    let text = read_source("Terraria/Main.cs");
    // The negative census (W4-f9): Main.cs feeds no tile-cast delegates and
    // no engine internals directly; ambience is scanner-seeded, not here.
    for token in [
        "PlotTileLine",
        "CastLight",
        "SpreadLight",
        "_activeEngine",
        "perFrameLights",
        "DynamicLight",
        "LightingEngine",
    ] {
        assert_eq!(
            text.matches(token).count(),
            0,
            "`{token}` appeared in Main.cs — the tile/ambient remainder was \
             closed EMPTY (W4-f9 c2740784); a new form needs a census row"
        );
    }
}

#[test]
fn the_christmas_hook_arm_structure_is_pinned() {
    let text = read_source("Terraria/Main.cs");
    let lines: Vec<&str> = text.lines().collect();
    // The 332 arm gate and the walk's exits/anchor/step/wrap — the exact
    // statements e8's `christmas_hook_chain_lights` replays.
    assert_eq!(line_at(&lines, 28991), "else if (proj.type == 332)");
    assert_eq!(line_at(&lines, 29002), "if (num115 < 30f)");
    assert_eq!(line_at(&lines, 29007), "if (float.IsNaN(num115))");
    // Truncate-then-divide anchor — NOT the quotient form (differs < 0).
    assert_eq!(line_at(&lines, 29012), "int i2 = (int)vector19.X / 16;");
    assert_eq!(line_at(&lines, 29013), "int j2 = (int)vector19.Y / 16;");
    assert_eq!(line_at(&lines, 29030), "num115 = 16f / num115;");
    assert_eq!(line_at(&lines, 29040), "if (num112 > 3)");
    assert_eq!(line_at(&lines, 29042), "num112 = 0;");
    // The light is sampled at the PRE-step position: the anchor (:29012)
    // precedes the step (:29030-29034) in source order — pinned by the
    // line numbers above; the colour rows sit between them (:29014-29029).
    assert_eq!(line_at(&lines, 29014), "if (num112 == 0)");
    assert_eq!(line_at(&lines, 29018), "if (num112 == 1)");
    assert_eq!(line_at(&lines, 29022), "if (num112 == 2)");
    assert_eq!(line_at(&lines, 29026), "if (num112 == 3)");
}

#[test]
fn the_illuminant_hook_arm_structure_is_pinned() {
    let text = read_source("Terraria/Main.cs");
    let lines: Vec<&str> = text.lines().collect();
    // The shared chain arm's gate lists 486..489; the 488 sheet is
    // Chains[type - 486] = Chains[2], num116 = its Height.
    assert!(line_at(&lines, 29046).contains("proj.type >= 486 && proj.type <= 489"));
    assert_eq!(line_at(&lines, 29071), "if (proj.type >= 486 && proj.type <= 489)");
    assert_eq!(
        line_at(&lines, 29073),
        "value11 = TextureAssets.Chains[proj.type - 486].Value;"
    );
    assert_eq!(line_at(&lines, 29088), "float num116 = value11.Height;");
    // The exit is checked BEFORE the step; the light rides the STEPPED
    // center2 through the Vector2 overload (quotient anchor).
    assert_eq!(line_at(&lines, 29146), "if (vector21.Length() < num116 + 1f)");
    assert_eq!(line_at(&lines, 29173), "center2 += vector22 * num116;");
    assert_eq!(line_at(&lines, 29184), "if (proj.type == 488)");
    // The num117 offset switch lists 383/446/489/487 only — 488 has none.
    assert_eq!(line_at(&lines, 29099), "case 383:");
    assert_eq!(line_at(&lines, 29102), "case 446:");
    assert_eq!(line_at(&lines, 29105), "case 489:");
    assert_eq!(line_at(&lines, 29108), "case 487:");
}

#[test]
fn the_lighting_cs_funnel_and_gate_are_pinned() {
    let text = read_source("Terraria/Lighting.cs");
    let lines: Vec<&str> = text.lines().collect();
    // The Vector2 float overload: QUOTIENT truncation `(int)(v / 16f)`.
    assert_eq!(
        line_at(&lines, 140),
        "public static void AddLight(Vector2 position, float r, float g, float b)"
    );
    assert_eq!(
        line_at(&lines, 142),
        "Lighting.AddLight((int)(position.X / 16f), (int)(position.Y / 16f), r, g, b);"
    );
    // The torch overload: truncate-then-divide `(int)v / 16` — the THIRD
    // anchor shape, distinct from both above below zero.
    assert_eq!(
        line_at(&lines, 154),
        "Lighting.AddLight((int)position.X / 16, (int)position.Y / 16, R, G, B);"
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
